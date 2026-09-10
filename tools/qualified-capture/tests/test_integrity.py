"""Payload corruption must fail before acquiring or executing tool sources."""
import importlib.util
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

PACKAGE = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("qualified_acquire", PACKAGE / "acquire.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class IntegrityTests(unittest.TestCase):
    def test_versioned_payloads(self):
        module.validate_payloads()

    def test_missing_and_tampered_repairs_fail_before_acquisition(self):
        for name in ["repairs/tuisnap.bundle", "repairs/tuisnap-code.patch"]:
            for mutation in ["missing", "tampered"]:
                with self.subTest(payload=name, mutation=mutation), tempfile.TemporaryDirectory() as temp:
                    package = Path(temp) / "package"
                    shutil.copytree(PACKAGE, package)
                    path = package / name
                    if mutation == "missing":
                        path.unlink()
                    else:
                        with path.open("ab") as stream:
                            stream.write(b"injected corruption\n")
                    root = Path(temp) / "must-not-exist"
                    result = subprocess.run([sys.executable, package / "acquire.py", "--root", root],
                                            capture_output=True, text=True, timeout=10)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn("Missing or tampered qualified payload: " + name, result.stderr)
                    self.assertFalse(root.exists())


if __name__ == "__main__":
    unittest.main()
