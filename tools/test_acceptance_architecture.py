from __future__ import annotations

from pathlib import Path
import subprocess
import tempfile
import unittest

import acceptance_architecture as gate


LIBRARIES = gate.json.loads(gate.POLICY_PATH.read_text())["production_library_packages"]
APPS = gate.json.loads(gate.POLICY_PATH.read_text())["application_binary_targets"]


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


class ArchitectureGateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="termrock-architecture-")
        self.root = Path(self.temp.name)
        members = []
        for name in LIBRARIES + APPS:
            crate = self.root / "crates" / name
            members.append(f'"crates/{name}"')
            write(crate / "Cargo.toml", f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2024"\n')
            if name in APPS:
                write(crate / "Cargo.toml", f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2024"\n[dependencies]\ntermrock = {{ path = "../termrock" }}\n')
            source = "fn main() {}\n" if name in APPS else "pub fn shared() {}\n"
            write(crate / "src" / ("main.rs" if name in APPS else "lib.rs"), source)
        write(self.root / "Cargo.toml", "[workspace]\nresolver = \"3\"\nmembers = [\n" + ",\n".join(members) + "\n]\n")
        generated = subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=self.root, capture_output=True, text=True)
        if generated.returncode:
            self.fail(f"could not prepare locked metadata fixture: {generated.stderr}")
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        subprocess.run(["git", "-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "fixture"], cwd=self.root, check=True)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def run_gate(self) -> dict:
        return gate.check(self.root)

    def test_complete_roles_do_not_attest_overall_acceptance(self) -> None:
        report = self.run_gate()
        self.assertTrue(report["architecture_checks_passed"], report["errors"])
        self.assertFalse(report["acceptance_complete"])
        self.assertTrue(report["unresolved"])
        self.assertEqual(set(report["packages"]["production_observed"]), set(LIBRARIES))
        self.assertEqual(report["packages"]["applications_missing"], [])

    def test_monolith_fails_missing_library_roles(self) -> None:
        write(self.root / "crates/termrock-terminal/Cargo.toml", '[package]\nname="junie-tui"\nversion="0.1.0"\nedition="2024"\n')
        report = self.run_gate()
        self.assertIn("termrock-terminal", report["packages"]["missing_production"])
        self.assertFalse(report["architecture_checks_passed"])

    def test_transitive_renamed_target_dependency_fails(self) -> None:
        helper = self.root / "crates/helper"
        write(helper / "Cargo.toml", '[package]\nname="termrock-helper"\nversion="0.1.0"\nedition="2024"\n')
        write(helper / "src/lib.rs", "pub fn helper() {}\n")
        write(self.root / "crates/showcase-domain/Cargo.toml", '[package]\nname="showcase-domain"\nversion="0.1.0"\nedition="2024"\n')
        write(self.root / "crates/showcase-domain/src/lib.rs", "pub fn domain() {}\n")
        write(self.root / "crates/termrock-core/Cargo.toml", '[package]\nname="termrock-core"\nversion="0.1.0"\nedition="2024"\n[target.\'cfg(unix)\'.dependencies]\ninternal = { package = "termrock-helper", path = "../helper" }\n')
        write(helper / "Cargo.toml", '[package]\nname="termrock-helper"\nversion="0.1.0"\nedition="2024"\n[dependencies]\napp_alias = { package = "showcase-domain", path = "../showcase-domain" }\n')
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        violation = next(item for item in report["dependency_violations"] if item["dependency"] == "showcase-domain" and item["from"] == "termrock-core")
        self.assertEqual(violation["file"], "crates/helper/Cargo.toml")
        self.assertIsInstance(violation["line"], int)
        self.assertEqual(len(violation["sha256"]), 64)

    def test_missing_role_fails_closed(self) -> None:
        path = self.root / "crates/termrock-terminal/Cargo.toml"
        write(path, path.read_text().replace('name = "termrock-terminal"', 'name = "renamed-terminal"'))
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertIn("termrock-terminal", report["packages"]["missing_production"])
        self.assertFalse(report["architecture_checks_passed"])

    def test_out_of_root_source_symlink_fails(self) -> None:
        outside = self.root.parent / f"{self.root.name}-outside.rs"
        outside.write_text("pub fn shared() {}\n", encoding="utf-8")
        source = self.root / "crates/termrock-core/src/lib.rs"
        source.unlink()
        source.symlink_to(outside)
        report = self.run_gate()
        self.assertTrue(any("out-of-root production input" in error for error in report["errors"]))
        outside.unlink()

    def test_renamed_static_stamp_in_library_fails(self) -> None:
        write(self.root / "crates/termrock-controls/src/moved.rs", """\
struct CellRecord { x: u16, y: u16, text: &'static str, fg: u32, bg: u32 }
const FRAME_PIECES: &[CellRecord] = &[
    CellRecord { x: 0, y: 0, text: "A", fg: 1, bg: 0 },
    CellRecord { x: 1, y: 0, text: "B", fg: 1, bg: 0 },
];
fn render(ui: &mut Ui) { for cell in FRAME_PIECES { ui.paint(cell.x, cell.y, cell.text); } }
""")
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any(item["file"].endswith("moved.rs") for item in report["findings"]))

    def test_exception_file_cannot_admit_stamp(self) -> None:
        write(self.root / "crates/termrock-controls/src/lib.rs", "// captured terminal frame\npub fn shared() {}\n")
        write(self.root / "crates/termrock-xtask/exceptions/v1.json", '{"allow_all":true}\n')
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any(item["rule"] == "frame-marker" for item in report["findings"]))

    def test_ordinary_ansi_controls_are_not_frame_replay(self) -> None:
        write(self.root / "crates/termrock-controls/src/lib.rs", 'pub const RED: &str = "\\u{1b}[31m";\npub const COLOR_MODE: &str = "Ansi256";\n')
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(any(item["rule"] == "ansi-frame-replay" for item in report["findings"]))

    def test_dynamic_generated_include_is_unresolved(self) -> None:
        write(self.root / "crates/termrock-core/src/lib.rs", 'include!(concat!(env!("OUT_DIR"), "/generated.rs"));\n')
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("dynamic include path cannot be enumerated" in item for item in report["unaccounted_inputs"]))

    def test_build_script_inputs_remain_unresolved(self) -> None:
        write(self.root / "crates/termrock-core/build.rs", "fn main() {}\n")
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("build-generated production inputs" in item for item in report["unaccounted_inputs"]))

    def test_application_without_shared_dependency_fails(self) -> None:
        write(self.root / "crates/showcase/Cargo.toml", '[package]\nname = "showcase"\nversion = "0.1.0"\nedition = "2024"\n')
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertIn("showcase", report["application_shared_closure_missing"])


if __name__ == "__main__":
    unittest.main()
