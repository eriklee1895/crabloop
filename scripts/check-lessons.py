#!/usr/bin/env python3
"""Replay the v1 teaching patches from an empty directory, without API calls.

Each stage must apply cleanly, stay rustfmt-clean and pass its tests. After the last stage,
every file the patches created must equal the reference tree, and src/ must match exactly.
"""
from pathlib import Path
import argparse
import json
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
PATCHES = ROOT / 'tutorial/v1/patches'
STAGES = ['01-responses', '02-file-tools', '03-bash', '04-agent-loop', '05-coding-task',
          '06-conversation']
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--v1-ref', help='compare with this Git ref instead of the working tree')
args = parser.parse_args()


def reference(relative):
    if args.v1_ref:
        return subprocess.check_output(['git', 'show', f'{args.v1_ref}:{relative}'], cwd=ROOT)
    return (ROOT / relative).read_bytes()


def reference_sources():
    if args.v1_ref:
        listing = subprocess.check_output(
            ['git', 'ls-tree', '-r', '--name-only', args.v1_ref, 'src'], cwd=ROOT, text=True)
        return set(listing.splitlines())
    return {p.relative_to(ROOT).as_posix() for p in (ROOT / 'src').rglob('*.rs')}


def run(command, cwd):
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    if result.returncode:
        print(result.stdout + result.stderr)
        raise SystemExit(f'{" ".join(command)} failed')
    return result.stdout


with tempfile.TemporaryDirectory(prefix='crabloop-lessons-') as directory:
    workspace = Path(directory)
    run(['git', 'init', '--quiet'], workspace)
    shutil.copyfile(ROOT / 'Cargo.lock', workspace / 'Cargo.lock')
    for stage in STAGES:
        patch = str(PATCHES / f'{stage}.patch')
        run(['git', 'apply', '--check', patch], workspace)
        run(['git', 'apply', patch], workspace)
        run(['cargo', 'fmt', '--check'], workspace)
        output = run(['cargo', 'test', '--locked', '--offline'], workspace)
        results = [line for line in output.splitlines() if line.startswith('test result:')]
        print(json.dumps({'stage': stage, 'passed': True, 'test_results': results},
                         ensure_ascii=False), flush=True)

    created = [p.relative_to(workspace).as_posix() for p in workspace.rglob('*')
               if p.is_file() and not {'.git', 'target'} & set(p.relative_to(workspace).parts)]
    for relative in sorted(created):
        if relative != 'Cargo.lock' and (workspace / relative).read_bytes() != reference(relative):
            raise SystemExit(f'lesson file differs from reference: {relative}')
    sources = {p for p in created if p.startswith('src/')}
    if sources != reference_sources():
        raise SystemExit(f'src file set differs: {sorted(sources ^ reference_sources())}')
    run(['python3', '-m', 'unittest', 'discover', '-s', 'tests', '-p', '*_test.py'], workspace)
    print(run(['python3', 'scripts/count-lines.py'], workspace))
