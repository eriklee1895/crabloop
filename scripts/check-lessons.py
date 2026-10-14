#!/usr/bin/env python3
"""Replay the published v1 teaching patches in a disposable workspace, without API calls."""
from pathlib import Path
import argparse
import json
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
PATCHES = ROOT / 'tutorial/v1/patches'
STAGES = ['00-probe', '01-responses', '02-file-tools', '03-bash', '04-agent-loop', '05-v1', '06-v1-entry']
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--v1-ref', default='milestone/v1', help='Git ref for the final v1 source tree')
args = parser.parse_args()

with tempfile.TemporaryDirectory(prefix='crabloop-lessons-') as directory:
    workspace = Path(directory)
    subprocess.run(['git', 'init', '--quiet', str(workspace)], check=True)
    for stage in STAGES:
        patch = PATCHES / f'{stage}.patch'
        subprocess.run(['git', 'apply', '--check', str(patch)], cwd=workspace, check=True)
        subprocess.run(['git', 'apply', str(patch)], cwd=workspace, check=True)
        reference = {'05-v1': 'v1', '06-v1-entry': args.v1_ref}.get(stage)
        if reference:
            expected = set(subprocess.check_output(
                ['git', 'ls-tree', '-r', '--name-only', reference, 'src'], cwd=ROOT,
                text=True).splitlines())
            actual = {source.relative_to(workspace).as_posix()
                      for source in (workspace / 'src').rglob('*.rs')}
            if actual != expected:
                raise SystemExit(f'{reference} source file set differs')
            for relative in sorted(actual):
                original = subprocess.check_output(['git', 'show', f'{reference}:{relative}'], cwd=ROOT)
                source = workspace / relative
                if source.read_bytes() != original:
                    raise SystemExit(f'{reference} source differs: {relative}')
        command = ['cargo', 'check' if patch.stem == '00-probe' else 'test', '--locked', '--offline']
        result = subprocess.run(command, cwd=workspace, capture_output=True, text=True)
        if result.returncode:
            print(result.stdout + result.stderr)
            raise SystemExit(result.returncode)
        lines = result.stdout.splitlines()
        print(json.dumps({'stage': patch.stem, 'passed': True,
                          'test_results': [line for line in lines if line.startswith('test result:')]}, ensure_ascii=False), flush=True)
    result = subprocess.run(['cargo', 'fmt', '--check'], cwd=workspace, capture_output=True, text=True)
    if result.returncode:
        raise SystemExit(result.stdout + result.stderr)
    subprocess.run(['python3', '-m', 'unittest', 'discover', '-s', 'tests', '-p', '*_test.py'], cwd=workspace, check=True)
    subprocess.run(['python3', 'scripts/count-lines.py', '--max', '399'], cwd=workspace, check=True)
    for source in sorted((workspace / 'src').rglob('*.rs')):
        lines = source.read_text().splitlines()
        print(json.dumps({'source': source.relative_to(workspace).as_posix(), 'physical': len(lines),
                          'code': sum(bool(line.strip()) and not line.lstrip().startswith('//') for line in lines)}))
