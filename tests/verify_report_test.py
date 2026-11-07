import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]

class ReportVerifierTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.workspace = Path(self.temp.name) / 'report'
        shutil.copytree(ROOT / 'fixtures/local-report', self.workspace)
        self.data = {'items': [{'id': ident, 'owner': owner, 'status': status, 'source': f'inbox/{ident}.md'} for ident,owner,status in [('alpha','\u6797','done'),('beta','\u9648','pending'),('gamma','\u5468','done')]], 'counts': {'done':2,'pending':1}}
        (self.workspace / 'SUMMARY.md').write_text('inbox/alpha.md inbox/beta.md inbox/gamma.md')
    def tearDown(self):
        self.temp.cleanup()
    def verify(self):
        (self.workspace / 'report.json').write_text(json.dumps(self.data))
        return subprocess.run(['python3',str(ROOT/'scripts/verify-report.py'),str(self.workspace)],capture_output=True).returncode
    def test_expected_owners_match_the_fixture_documents(self):
        for item in self.data['items']:
            text = (self.workspace / item['source']).read_text()
            self.assertIn(item['owner'], text)
            self.assertIn(item['status'], text)
        self.assertEqual(self.verify(), 0)
    def test_accepts_required_facts_and_an_extra_correct_total(self):
        self.data['counts']['total'] = 3
        self.data['items'][0]['note'] = 'optional descriptive metadata'
        self.assertEqual(self.verify(),0)
    def test_rejects_wrong_counts_and_boolean_counts(self):
        self.data['counts']['done'] = 1
        self.assertNotEqual(self.verify(),0)
        self.data['counts'] = {'done':2,'pending':True}
        self.assertNotEqual(self.verify(),0)
    def test_rejects_changed_source_and_wrong_owner(self):
        self.data['items'][0]['owner'] = 'wrong'
        self.assertNotEqual(self.verify(),0)
        self.data['items'][0]['owner'] = '\u6797'
        (self.workspace/'inbox/alpha.md').write_text('modified source')
        self.assertNotEqual(self.verify(),0)

if __name__ == '__main__':
    unittest.main()
