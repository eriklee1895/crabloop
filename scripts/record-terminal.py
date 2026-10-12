#!/usr/bin/env python3
r"""Record a real command in a fixed-size PTY as asciicast v2 (stdlib only).

Manual: capture.py -o demo.cast --record-input -- /path/to/crabloop
Steps:  capture.py -o demo.cast --steps inputs.json -- /path/to/crabloop

inputs.json is a list of {"wait_for": "prompt", "send": "task\r", ...}.
send is literal terminal input, not shell code; include \r to press Enter.
Optional fields: delay (seconds before typing), type_delay (seconds/character),
timeout (seconds waiting for prompt). Only the child's actual output is rendered.
"""

import argparse
import base64
import codecs
import errno
import fcntl
import json
import os
from pathlib import Path
import pty
import re
import select
import shlex
import signal
import struct
import sys
import termios
import time
import tty


ANSI = re.compile(r"\x1b\][^\x07]*(?:\x07|\x1b\\)|\x1b\[[0-?]*[ -/]*[@-~]")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("-o", "--output", type=Path, required=True)
    parser.add_argument("--cols", type=int, default=96)
    parser.add_argument("--rows", type=int, default=28)
    parser.add_argument("--cwd", type=Path, default=Path.cwd())
    parser.add_argument("--title", default="CrabLoop terminal recording")
    parser.add_argument("--record-input", action="store_true", help="also record actual sent input events")
    parser.add_argument("--raw-events", type=Path, help="optional timestamped base64 byte-event sidecar")
    parser.add_argument("--steps", type=Path, help="JSON array of prompt-gated input steps; disables manual input")
    parser.add_argument("--max-seconds", type=float, default=600)
    parser.add_argument("--overwrite", action="store_true")
    parser.add_argument("--quiet", action="store_true", help="do not mirror child's output to this terminal")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command or args.cols < 1 or args.rows < 1 or args.max_seconds <= 0:
        parser.error("a command, positive geometry, and positive max-seconds are required")
    steps = json.loads(args.steps.read_text()) if args.steps else []
    if not isinstance(steps, list):
        parser.error("steps must be a JSON array")
    for step in steps:
        if not isinstance(step, dict) or not isinstance(step.get("send"), str):
            parser.error("each step must be an object with a string send field")
        if not isinstance(step.get("wait_for", ""), str):
            parser.error("wait_for must be a literal string")
        if any(float(step.get(key, default)) < 0 for key, default in (("delay", 0.3), ("type_delay", 0.025), ("timeout", 300))):
            parser.error("step delays and timeout must be non-negative")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    cast = args.output.open("w" if args.overwrite else "x", encoding="utf-8", buffering=1)
    raw = None
    if args.raw_events:
        args.raw_events.parent.mkdir(parents=True, exist_ok=True)
        raw = args.raw_events.open("w" if args.overwrite else "x", encoding="utf-8", buffering=1)
    timestamp = int(time.time())
    env = dict(os.environ, TERM="xterm-256color", COLORTERM="truecolor", COLUMNS=str(args.cols), LINES=str(args.rows))
    header = {"version": 2, "width": args.cols, "height": args.rows, "timestamp": timestamp,
              "title": args.title, "command": shlex.join(command),
              "env": {"TERM": env["TERM"], "SHELL": env.get("SHELL", "/bin/bash")}}
    cast.write(json.dumps(header, ensure_ascii=False) + "\n")
    if raw:
        raw.write(json.dumps({"format": "pty-base64-v1", "width": args.cols, "height": args.rows,
                              "timestamp": timestamp}) + "\n")
    start = time.monotonic()
    decoders = {code: codecs.getincrementaldecoder("utf-8")("replace") for code in ("o", "i")}

    def record(code, data):
        elapsed = round(time.monotonic() - start, 6)
        if raw:
            raw.write(json.dumps([elapsed, code, base64.b64encode(data).decode("ascii")]) + "\n")
        text = decoders[code].decode(data)
        if text:
            cast.write(json.dumps([elapsed, code, text], ensure_ascii=False) + "\n")
        return text

    pid, master = pty.fork()
    if pid == 0:
        try:
            fcntl.ioctl(0, termios.TIOCSWINSZ, struct.pack("HHHH", args.rows, args.cols, 0, 0))
            os.chdir(args.cwd)
            os.execvpe(command[0], command, env)
        except Exception as exc:
            print(f"capture child: {exc}", file=sys.stderr, flush=True)
            os._exit(127)

    saved_tty = None
    status = None
    failure = None
    observed = ""
    step_index = 0
    step_started = time.monotonic()
    pending = None
    next_key_at = None
    stdin_open = not args.steps

    def send(data):
        while data:
            count = os.write(master, data)
            if args.record_input:
                record("i", data[:count])
            data = data[count:]

    def interrupted(signum, _frame):
        raise InterruptedError(f"recorder received signal {signum}")

    previous_signals = {signum: signal.signal(signum, interrupted) for signum in (signal.SIGTERM, signal.SIGINT)}
    try:
        if stdin_open and sys.stdin.isatty():
            saved_tty = termios.tcgetattr(sys.stdin.fileno())
            tty.setraw(sys.stdin.fileno())
        while True:
            now = time.monotonic()
            if now - start > args.max_seconds:
                raise TimeoutError(f"recording exceeded {args.max_seconds:g} seconds")
            if step_index < len(steps):
                step = steps[step_index]
                if pending is None:
                    if step.get("wait_for", "") in ANSI.sub("", observed):
                        pending = list(step["send"])
                        next_key_at = now + float(step.get("delay", 0.3))
                    elif now - step_started > float(step.get("timeout", 300)):
                        raise TimeoutError(f"step {step_index + 1} never saw {step.get('wait_for', '')!r}")
                if pending is not None and now >= next_key_at:
                    if pending:
                        send(pending.pop(0).encode("utf-8"))
                        next_key_at = now + float(step.get("type_delay", 0.025))
                    if not pending:
                        observed = ""
                        pending = None
                        step_index += 1
                        step_started = now
            descriptors = [master] + ([sys.stdin.fileno()] if stdin_open else [])
            timeout = 0.05
            if pending is not None:
                timeout = min(timeout, max(0, next_key_at - time.monotonic()))
            readable, _, _ = select.select(descriptors, [], [], timeout)
            if master in readable:
                try:
                    chunk = os.read(master, 65536)
                except OSError as exc:
                    if exc.errno != errno.EIO:
                        raise
                    chunk = b""
                if not chunk:
                    break
                observed = (observed + record("o", chunk))[-1048576:]
                if not args.quiet:
                    sys.stdout.buffer.write(chunk)
                    sys.stdout.buffer.flush()
            if stdin_open and sys.stdin.fileno() in readable:
                chunk = os.read(sys.stdin.fileno(), 65536)
                if chunk:
                    send(chunk)
                else:
                    stdin_open = False
                    send(b"\x04")
        _, status = os.waitpid(pid, 0)
        if step_index < len(steps):
            raise RuntimeError(f"child exited before input step {step_index + 1} of {len(steps)}")
    except (Exception, KeyboardInterrupt) as exc:
        failure = exc
    finally:
        if saved_tty is not None:
            termios.tcsetattr(sys.stdin.fileno(), termios.TCSADRAIN, saved_tty)
        for signum, handler in previous_signals.items():
            signal.signal(signum, handler)
        os.close(master)
        if status is None:
            try:
                os.killpg(pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            until = time.monotonic() + 1
            while time.monotonic() < until:
                result, status = os.waitpid(pid, os.WNOHANG)
                if result:
                    break
                status = None
                time.sleep(0.02)
            if status is None:
                try:
                    os.killpg(pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                _, status = os.waitpid(pid, 0)
        for code, decoder in decoders.items():
            text = decoder.decode(b"", final=True)
            if text:
                cast.write(json.dumps([round(time.monotonic() - start, 6), code, text], ensure_ascii=False) + "\n")
        cast.close()
        if raw:
            raw.close()
    if failure:
        print(f"capture: {failure}; partial recording retained at {args.output}", file=sys.stderr)
        return 1
    exit_code = os.waitstatus_to_exitcode(status)
    print(f"capture: saved {args.output} ({time.monotonic() - start:.2f}s, child exit {exit_code})", file=sys.stderr)
    return exit_code if exit_code >= 0 else 128 - exit_code


if __name__ == "__main__":
    raise SystemExit(main())
