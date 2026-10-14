import json
import os
from pathlib import Path
import pty
import select
import subprocess
import tempfile
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, HTTPServer

ROOT = Path(__file__).resolve().parents[1]


class TerminalChatTest(unittest.TestCase):
    def test_terminal_followup_keeps_history_and_ctrl_d_exits(self):
        subprocess.run(['cargo', 'build', '--locked'], cwd=ROOT, check=True, capture_output=True)
        target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target'))
        binary = (target / 'debug/crabloop').resolve()
        requests = []

        class Handler(BaseHTTPRequestHandler):
            def do_POST(self):
                requests.append(json.loads(self.rfile.read(int(self.headers['Content-Length']))))
                answer = 'First answer' if len(requests) == 1 else 'Followup answer'
                body = json.dumps({'status': 'completed', 'output': [{
                    'type': 'message', 'role': 'assistant', 'phase': 'final_answer',
                    'content': [{'type': 'output_text', 'text': answer}]
                }]}).encode()
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Content-Length', str(len(body)))
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, *args):
                pass

        server = HTTPServer(('127.0.0.1', 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        master, slave = pty.openpty()
        child = None
        pending = b''

        def read_until(needle):
            nonlocal pending
            deadline = time.monotonic() + 10
            while needle not in pending:
                self.assertLess(time.monotonic(), deadline, pending.decode(errors='replace'))
                ready, _, _ = select.select([master], [], [], 0.1)
                if not ready and child.poll() is not None:
                    self.fail(f'Process exited before {needle!r}: {pending!r}')
                if ready:
                    try:
                        chunk = os.read(master, 4096)
                    except OSError as error:
                        self.fail(f'Terminal closed before {needle!r}: {error}; {pending!r}')
                    self.assertTrue(chunk, pending)
                    pending += chunk
            end = pending.index(needle) + len(needle)
            seen, pending = pending[:end], pending[end:]
            return seen

        try:
            with tempfile.TemporaryDirectory() as directory:
                env = {'PATH': os.environ['PATH'], 'CRABLOOP_PROVIDER': 'openai',
                       'OPENAI_API_KEY': 'test-key', 'CRABLOOP_MODEL': 'test-model',
                       'CRABLOOP_API_URL': f'http://127.0.0.1:{server.server_port}/responses'}
                child = subprocess.Popen([str(binary)], cwd=directory, env=env,
                                         stdin=slave, stdout=slave, stderr=slave)
                read_until(b'crab> ')
                os.write(master, b'First question\n')
                read_until(b'First answer')
                read_until(b'crab> ')
                os.write(master, b'\n')
                read_until(b'crab> ')
                self.assertEqual(len(requests), 1)
                os.write(master, b'Followup question\n')
                read_until(b'Followup answer')
                read_until(b'crab> ')
                os.write(master, b'\x04')
                self.assertEqual(child.wait(timeout=5), 0)
            self.assertEqual(len(requests), 2)
            history = requests[1]['input']
            self.assertEqual(history[0]['content'], 'First question')
            self.assertEqual(history[1]['content'][0]['text'], 'First answer')
            self.assertEqual(history[2]['content'], 'Followup question')
        finally:
            if child is not None and child.poll() is None:
                child.kill()
                child.wait()
            os.close(master)
            os.close(slave)
            server.shutdown()
            server.server_close()
            thread.join()


if __name__ == '__main__':
    unittest.main()
