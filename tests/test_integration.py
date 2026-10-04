# SPDX-License-Identifier: GPL-3.0-only
import importlib.util
from pathlib import Path
import tempfile
import unittest

path=Path(__file__).resolve().parents[1]/'scripts/integrate.py'
spec=importlib.util.spec_from_file_location('integrate',path)
integration=importlib.util.module_from_spec(spec);spec.loader.exec_module(integration)
class IntegrationTests(unittest.TestCase):
    def tree(self, path, family=(5,10,198)):
        (path/'drivers').mkdir();(path/'Makefile').write_text(f'VERSION = {family[0]}\nPATCHLEVEL = {family[1]}\nSUBLEVEL = {family[2]}\n')
        for file in ['Makefile','Kconfig']: (path/'drivers'/file).write_text('# existing kernel content\n')
    def test_repeat_preserves_tree(self):
        with tempfile.TemporaryDirectory() as tmp:
            tree=Path(tmp);self.tree(tree);integration.integrate(tree);integration.integrate(tree)
            for file in ['Makefile','Kconfig']:
                text=(tree/'drivers'/file).read_text();self.assertEqual(text.count(integration.MARK),1);self.assertIn('# existing kernel content',text)
            self.assertTrue((tree/'drivers/kilasu/uapi/include/kilasu.h').exists())
    def test_all_target_families(self):
        for family in [(5,10,1),(5,15,1),(6,1,1)]:
            with tempfile.TemporaryDirectory() as tmp:
                tree=Path(tmp);self.tree(tree,family);self.assertEqual(integration.version(tree),family)
    def test_unknown_tree_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            tree=Path(tmp);self.tree(tree);(tree/'drivers/kilasu').mkdir()
            with self.assertRaises(ValueError):integration.integrate(tree)
    def test_unsupported_kernel(self):
        with tempfile.TemporaryDirectory() as tmp:
            tree=Path(tmp);self.tree(tree,(4,19,0))
            with self.assertRaises(ValueError):integration.integrate(tree)
