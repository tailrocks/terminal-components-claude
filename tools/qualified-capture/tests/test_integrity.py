"""Payload corruption must fail before acquiring or executing tool sources."""
import importlib.util
import json
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

    def assert_manifest_rejected(self, mutate, message):
        with tempfile.TemporaryDirectory() as temp:
            package = Path(temp) / "package"
            shutil.copytree(PACKAGE, package)
            pin_path = package / "pins.json"
            pins = json.loads(pin_path.read_text())
            mutate(pins)
            pin_path.write_text(json.dumps(pins))
            root = Path(temp) / "must-not-exist"
            result = subprocess.run([sys.executable, package / "acquire.py", "--root", root],
                                    capture_output=True, text=True, timeout=10)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(message, result.stderr)
            self.assertFalse(root.exists())

    def test_operational_payload_cannot_be_retired_or_unpinned(self):
        self.assert_manifest_rejected(
            lambda pins: pins["payload_sha256"].pop("fixture.py"),
            "Qualified operational payload membership changed")

        def retire_executable(pins):
            pins["retired_documentation_sha256"]["fixture.py"] = pins["payload_sha256"].pop("fixture.py")

        self.assert_manifest_rejected(retire_executable, "Invalid qualified documentation retirement")

    def test_documentation_retirement_requires_exact_provenance(self):
        self.assert_manifest_rejected(
            lambda pins: pins["retired_documentation_sha256"].update({"README.md": "0" * 64}),
            "Invalid qualified documentation retirement")
        self.assert_manifest_rejected(
            lambda pins: pins.update(supersedes_pins_sha256="0" * 64),
            "Invalid qualified documentation retirement")

    def test_unknown_payload_schema_rejected(self):
        self.assert_manifest_rejected(
            lambda pins: pins.update(schema=3),
            "Unsupported qualified payload schema")

    def test_missing_and_tampered_repairs_fail_before_acquisition(self):
        for name in ["repairs/tuisnap.bundle", "repairs/tuisnap-code.patch", "harness/main.rs",
                     "fixture.py", "licenses/tui-snap-LICENSE"]:
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
