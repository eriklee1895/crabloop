import contextlib
import io
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parents[1] / 'scripts/run-live.py'

class LiveConfigTest(unittest.TestCase):
    def run_case(self, inherited, explicit, expected):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'scripts').mkdir()
            shutil.copyfile(SOURCE, root / 'scripts/run-live.py')
            (root / 'fixtures/local-report').mkdir(parents=True)
            (root / 'src').mkdir()
            (root / '.env').write_text('DEEPSEEK_API_KEY=test-key\n')
            environment = {} if inherited is None else {'CRABLOOP_CONTEXT_BYTES': str(inherited)}
            argv = ['run-live.py', '--stage', 'v2', '--scenario', 'report']
            if explicit is not None:
                argv += ['--context-bytes', str(explicit)]
            results = iter([subprocess.CompletedProcess([], 1), subprocess.CompletedProcess([], 0), subprocess.CompletedProcess([], 0)])
            child = SimpleNamespace(returncode=0, pid=1, communicate=lambda timeout: ('done', ''))
            with patch.dict(os.environ, environment, clear=True), patch.object(sys, 'argv', argv), \
                 patch('subprocess.run', side_effect=lambda *a, **k: next(results)), \
                 patch('subprocess.check_output', return_value='test-commit'), \
                 patch('subprocess.Popen', return_value=child) as spawn, \
                 contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaises(SystemExit) as exit_status:
                    runpy.run_path(str(root / 'scripts/run-live.py'), run_name='__main__')
                self.assertEqual(exit_status.exception.code, 0)
            passed_env = spawn.call_args.kwargs['env']
            actual = int(passed_env.get('CRABLOOP_CONTEXT_BYTES', '49152'))
            record = json.loads(next((root / 'evidence/v2').glob('*.json')).read_text())
            self.assertEqual(actual, expected)
            self.assertEqual(record['context_bytes'], actual)

    def test_inherited_budget_is_recorded(self):
        self.run_case(10240, None, 10240)

    def test_explicit_budget_overrides_inherited_value(self):
        self.run_case(10240, 16384, 16384)

    def test_default_budget_is_recorded(self):
        self.run_case(None, None, 49152)
