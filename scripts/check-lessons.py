#!/usr/bin/env python3
"""Replay the published v1 teaching patches in a disposable workspace, without API calls."""
from pathlib import Path
import json
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
PATCHES = ROOT / 'tutorial/v1/patches'
STAGES = ['00-probe', '01-responses', '02-file-tools', '03-bash', '04-agent-loop', '05-v1', '06-readable']

with tempfile.TemporaryDirectory(prefix='crabloop-lessons-') as directory:
    workspace = Path(directory)
    subprocess.run(['git', 'init', '--quiet', str(workspace)], check=True)
    for stage in STAGES:
        patch = PATCHES / f'{stage}.patch'
        subprocess.run(['git', 'apply', '--check', str(patch)], cwd=workspace, check=True)
        subprocess.run(['git', 'apply', str(patch)], cwd=workspace, check=True)
        if patch.stem == '05-v1':
            for source in (workspace / 'src').glob('*.rs'):
                relative = source.relative_to(workspace).as_posix()
                original = subprocess.check_output(['git', 'show', f'v1:{relative}'], cwd=ROOT)
                if source.read_bytes() != original:
                    raise SystemExit(f'v1 source differs: {relative}')
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
    for source in sorted((workspace / 'src').glob('*.rs')):
        lines = source.read_text().splitlines()
        print(json.dumps({'source': source.relative_to(workspace).as_posix(), 'physical': len(lines),
                          'code': sum(bool(line.strip()) and not line.lstrip().startswith('//') for line in lines)}))
