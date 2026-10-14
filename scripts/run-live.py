#!/usr/bin/env python3
"""Run one controlled live repair. Credentials and raw logs stay local."""
import argparse
from datetime import datetime, timezone
import difflib
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--provider', choices=['deepseek', 'luna'], default='deepseek')
parser.add_argument('--stage', choices=['v1'], default='v1')
parser.add_argument('--scenario', choices=['coding', 'report'], default='coding')
args = parser.parse_args()
settings = dict(os.environ)
if (ROOT / '.env').exists():
    for line in (ROOT / '.env').read_text().splitlines():
        line = line.strip()
        if line and not line.startswith('#') and '=' in line:
            name, value = line.removeprefix('export ').split('=', 1)
            settings[name.strip()] = value.strip().strip('\"\'')
if args.provider == 'deepseek':
    model, base = 'deepseek-flash', 'https://api.deepseek.com'
    key = settings.get('DEEPSEEK_API_KEY')
    if not key and settings.get('OPENAI_BASE_URL', '').rstrip('/') == base:
        key = settings.get('OPENAI_API_KEY')
else:
    model, base, key = 'gpt-6-luna', settings.get('OPENAI_BASE_URL'), settings.get('OPENAI_API_KEY')
if not key or not base:
    raise SystemExit('Missing selected provider configuration in .env or environment')
stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
workspace = ROOT / 'playground' / f'{args.stage}-{args.scenario}-{args.provider}-{stamp}'
fixture_name = 'buggy-rust' if args.scenario == 'coding' else 'local-report'
shutil.copytree(ROOT / 'fixtures' / fixture_name, workspace)
raw_dir = ROOT / '.crabloop' / 'live' / workspace.name
raw_dir.mkdir(parents=True)

def verify():
    command = ['bash', str(ROOT / 'scripts/verify-playground.sh')] if args.scenario == 'coding' else ['python3', str(ROOT / 'scripts/verify-report.py')]
    return subprocess.run(command + [str(workspace)], capture_output=True, text=True, cwd=ROOT, timeout=30)

baseline = verify()
if baseline.returncode == 0:
    raise SystemExit('Fixture unexpectedly passes before repair')
env = os.environ.copy()
env.update(CRABLOOP_PROVIDER="openai", OPENAI_API_KEY=key, OPENAI_BASE_URL=base, CRABLOOP_MODEL=model)
env.pop('CRABLOOP_API_URL', None)
task = 'Read README.md and inspect the source. Fix last_index in lib.rs. Run the existing tests before editing and after the fix. Preserve the public signature and tests.rs. Write NOTES.md describing the change and actual verification. Report only observed results.'
subprocess.run(['cargo', 'build', '--locked'], cwd=ROOT, check=True, capture_output=True)
if args.scenario == 'report':
    task = 'Follow README.md: inspect all inbox documents, produce report.json with exact facts, source paths and counts, and a Chinese SUMMARY.md citing every source. Preserve the original inbox files. Verify your output against the input documents before reporting completion.'
start = datetime.now(timezone.utc)
proc = subprocess.Popen([str(ROOT / 'target/debug/crabloop')], env=env, cwd=workspace, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
timed_out = False
try:
    stdout, stderr = proc.communicate(input=task, timeout=300)
except subprocess.TimeoutExpired:
    timed_out = True
    os.killpg(proc.pid, signal.SIGKILL)
    stdout, stderr = proc.communicate()
(raw_dir / 'stdout.txt').write_text(stdout)
(raw_dir / 'stderr.txt').write_text(stderr)
final = verify()
original = (ROOT / 'fixtures/buggy-rust/lib.rs').read_text() if args.scenario == 'coding' else ''
actual = (workspace / 'lib.rs').read_text() if (workspace / 'lib.rs').is_file() else ''
tests_unchanged = (workspace / 'tests.rs').is_file() and (workspace / 'tests.rs').read_bytes() == (ROOT / 'fixtures/buggy-rust/tests.rs').read_bytes()
record = {
    'date_utc': start.isoformat(), 'scenario': args.scenario, 'source_commit': subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'source_sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((ROOT/'src').glob('*.rs'))}, 'stage': args.stage, 'provider': args.provider, 'model': model,
    'elapsed_seconds': round((datetime.now(timezone.utc) - start).total_seconds(), 2),
    'agent_exit_code': proc.returncode, 'driver_timeout': timed_out,
    'baseline_verifier_exit_code': baseline.returncode, 'final_verifier_exit_code': final.returncode,
    'original_tests_unchanged': tests_unchanged, 'notes_created': (workspace / 'NOTES.md').exists(),
    'source_diff': ''.join(difflib.unified_diff(original.splitlines(True), actual.splitlines(True), fromfile='before/lib.rs', tofile='after/lib.rs')),
    'workspace': str(workspace.relative_to(ROOT)),
}
record['passed'] = proc.returncode == 0 and final.returncode == 0 and tests_unchanged and record['notes_created']
if args.scenario == 'report':
    record['passed'] = proc.returncode == 0 and final.returncode == 0
    for field in ('original_tests_unchanged','notes_created','source_diff'):
        record.pop(field, None)
report = ROOT / 'evidence' / args.stage / f'{stamp}-{args.scenario}-{args.provider}.json'
report.parent.mkdir(parents=True, exist_ok=True)
report.write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'report': str(report.relative_to(ROOT)), 'passed': record['passed'], 'agent_exit_code': proc.returncode, 'verifier_exit_code': final.returncode, 'elapsed_seconds': record['elapsed_seconds']}))
raise SystemExit(0 if record['passed'] else 1)
