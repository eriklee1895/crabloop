import os
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]

class LiveDriverScopeTest(unittest.TestCase):
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
