import os
import json
from pathlib import Path
import runpy
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[1]

class LiveDriverScopeTest(unittest.TestCase):
    def test_task_is_sent_through_stdin(self):
        for scenario in ('coding', 'report'):
            with self.subTest(scenario=scenario), tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                (root / 'scripts').mkdir()
                script = root / 'scripts/run-live.py'
                shutil.copyfile(ROOT / 'scripts/run-live.py', script)
                shutil.copytree(ROOT / 'fixtures', root / 'fixtures')
                process = Mock(returncode=1)
                process.communicate.return_value = ('', '')
                with patch.dict(os.environ, {'DEEPSEEK_API_KEY': 'test-key'}, clear=True), \
                     patch.object(sys, 'argv', [str(script), '--scenario', scenario]), \
                     patch('subprocess.run', side_effect=[Mock(returncode=1), Mock(returncode=0), Mock(returncode=1)]), \
                     patch('subprocess.check_output', return_value='test-revision\n'), \
                     patch('subprocess.Popen', return_value=process) as popen, \
                     patch('builtins.print'):
                    with self.assertRaises(SystemExit) as result:
                        runpy.run_path(str(script), run_name='__main__')
                self.assertEqual(result.exception.code, 1)
                self.assertEqual(popen.call_args.args[0], [str(root / 'target/debug/crabloop')])
                self.assertEqual(popen.call_args.kwargs['stdin'], subprocess.PIPE)
                self.assertNotIn('CRABLOOP_TRACE_TOOLS', popen.call_args.kwargs['env'])
                record = json.loads(next((root / 'evidence/v1').glob('*.json')).read_text())
                for field in ('tool_events', 'failed_test_seen', 'passed_test_seen'):
                    self.assertNotIn(field, record)
                process.communicate.assert_called_once()
                sent = process.communicate.call_args.kwargs
                self.assertEqual(sent['timeout'], 300)
                self.assertIn('last_index' if scenario == 'coding' else 'report.json', sent['input'])

    def test_later_stage_options_are_rejected_before_credentials(self):
        for args in (["--context-bytes", "4096"], ["--stage", "v2"]):
            with self.subTest(args=args):
                result = subprocess.run(
                    [sys.executable, str(ROOT / "scripts/run-live.py"), *args],
                    env={"PATH": os.environ["PATH"]}, capture_output=True, text=True,
                )
                self.assertEqual(result.returncode, 2)
                self.assertNotIn("Missing selected provider", result.stderr)

if __name__ == "__main__":
    unittest.main()
