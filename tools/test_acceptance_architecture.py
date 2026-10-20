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
        write(helper / "src/lib.rs", "// captured terminal frame\npub fn helper() {}\n")
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
        self.assertTrue(any(item["file"] == "crates/helper/src/lib.rs" for item in report["findings"]))

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

    def test_out_of_root_directory_symlink_fails_closed(self) -> None:
        outside = self.root.parent / f"{self.root.name}-outside-dir"
        (outside / "external").mkdir(parents=True)
        (outside / "external/mod.rs").write_text("// captured terminal frame\n", encoding="utf-8")
        link = self.root / "crates/termrock-core/src/external"
        link.symlink_to(outside / "external", target_is_directory=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("out-of-root production input" in error and "src/external" in error for error in report["errors"]))
        self.assertFalse(any(row["path"].endswith("src/external/mod.rs") for row in report["source_inputs"]))
        import shutil

        shutil.rmtree(outside)

    def test_in_root_directory_symlink_contents_are_scanned(self) -> None:
        write(self.root / "crates/termrock-core/external/mod.rs", "pub fn included() {}\n")
        link = self.root / "crates/termrock-core/src/external"
        link.symlink_to(self.root / "crates/termrock-core/external", target_is_directory=True)
        write(self.root / "crates/termrock-core/src/lib.rs", '#[path = "external/mod.rs"]\nmod external;\n')
        report = self.run_gate()
        self.assertTrue(report["architecture_checks_passed"], report["errors"])
        self.assertFalse(report["errors"])
        self.assertTrue(any(row["path"] == "crates/termrock-core/src/external/mod.rs" for row in report["source_inputs"]))

    def test_in_root_directory_symlink_cycle_fails_closed(self) -> None:
        link = self.root / "crates/termrock-core/src/cycle"
        link.symlink_to(self.root / "crates/termrock-core/src", target_is_directory=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("recursive production source directory symlink cycle" in error for error in report["errors"]))

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

    def test_custom_lib_target_path_is_hashed_and_scanned(self) -> None:
        write(self.root / "crates/termrock-core/Cargo.toml", '[package]\nname="termrock-core"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="other.rs"\n')
        write(self.root / "crates/termrock-core/other.rs", "// captured terminal frame\npub fn shared() {}\n")
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any(row["path"] == "crates/termrock-core/other.rs" for row in report["source_inputs"]))
        self.assertTrue(any(item["file"] == "crates/termrock-core/other.rs" for item in report["findings"]))

    def test_custom_target_explicit_tests_module_is_scanned(self) -> None:
        write(self.root / "crates/termrock-core/Cargo.toml", '[package]\nname="termrock-core"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="other.rs"\n')
        write(self.root / "crates/termrock-core/other.rs", "pub mod tests;\n")
        write(self.root / "crates/termrock-core/tests/mod.rs", "// captured terminal frame\n")
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any(row["path"] == "crates/termrock-core/tests/mod.rs" for row in report["source_inputs"]))
        self.assertTrue(any(item["file"] == "crates/termrock-core/tests/mod.rs" for item in report["findings"]))

    def test_untracked_tests_module_under_src_is_scanned(self) -> None:
        write(self.root / "crates/termrock-core/src/lib.rs", "pub mod tests;\n")
        write(self.root / "crates/termrock-core/src/tests/mod.rs", "// captured terminal frame\n")
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any(row["path"] == "crates/termrock-core/src/tests/mod.rs" for row in report["source_inputs"]))
        self.assertTrue(any(item["file"] == "crates/termrock-core/src/tests/mod.rs" for item in report["findings"]))

    def test_actual_cargo_target_directory_symlink_is_pruned(self) -> None:
        write(self.root / "target/generated.rs", "// captured terminal frame\n")
        link = self.root / "crates/termrock-core/src/target"
        link.symlink_to(self.root / "target", target_is_directory=True)
        report = self.run_gate()
        self.assertTrue(report["architecture_checks_passed"], report["errors"])
        self.assertFalse(any("target/generated.rs" in row["path"] for row in report["source_inputs"]))

    def test_declared_module_into_pruned_target_directory_is_unaccounted(self) -> None:
        write(self.root / "target/mod.rs", "// captured terminal frame\n")
        link = self.root / "crates/termrock-core/src/target"
        link.symlink_to(self.root / "target", target_is_directory=True)
        write(self.root / "crates/termrock-core/src/lib.rs", "pub mod target;\n")
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("production module resolves inside Cargo target directory" in item and "src/lib.rs:1" in item for item in report["unaccounted_inputs"]))
        self.assertFalse(any(row["path"] == "target/mod.rs" for row in report["source_inputs"]))

    def test_custom_cargo_root_module_alias_into_target_is_unaccounted(self) -> None:
        crate = self.root / "crates/termrock-core"
        write(crate / "Cargo.toml", '[package]\nname="termrock-core"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="other.rs"\n')
        write(crate / "other.rs", "pub mod target;\n")
        write(self.root / "target/mod.rs", "// captured terminal frame\n")
        (crate / "target").symlink_to(self.root / "target", target_is_directory=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("production module may resolve through a Cargo target-directory alias" in item and "other.rs:1" in item for item in report["unaccounted_inputs"]))

    def test_out_of_package_cargo_root_and_raw_identifier_alias_are_unaccounted(self) -> None:
        crate = self.root / "crates/termrock-core"
        write(crate / "Cargo.toml", '[package]\nname="termrock-core"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="../../shared/other.rs"\n')
        write(self.root / "shared/other.rs", "pub mod r#target;\n")
        write(self.root / "target/mod.rs", "pub fn generated() {}\n")
        (self.root / "shared/target").symlink_to(self.root / "target", target_is_directory=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("production module may resolve through a Cargo target-directory alias" in item and "shared/other.rs:1" in item for item in report["unaccounted_inputs"]))

    def test_out_of_package_cargo_root_with_ordinary_module_tree_fails_closed(self) -> None:
        crate = self.root / "crates/termrock-core"
        write(crate / "Cargo.toml", '[package]\nname="termrock-core"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="../../shared/lib.rs"\n')
        write(self.root / "shared/lib.rs", 'pub mod hidden;\ninclude!("support.inc");\n')
        write(self.root / "shared/hidden.rs", 'pub mod nested;\ninclude!("hidden.inc");\n')
        write(self.root / "shared/hidden/nested.rs", "// captured terminal frame in module child\n")
        write(self.root / "shared/hidden.inc", "// captured terminal frame in nested module include\n")
        write(self.root / "shared/support.inc", 'include!("nested.inc");\n')
        write(self.root / "shared/nested.inc", "// captured terminal frame in included source\n")
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("production Cargo target source is outside its package; module tree is not fully accounted" in item and "shared/lib.rs" in item for item in report["unaccounted_inputs"]))
        self.assertTrue(any(row["path"] == "shared/lib.rs" for row in report["source_inputs"]))
        self.assertTrue(any(row["path"] == "shared/support.inc" for row in report["source_inputs"]))
        self.assertTrue(any(row["path"] == "shared/nested.inc" for row in report["source_inputs"]))
        self.assertFalse(any(row["path"] == "shared/hidden.rs" for row in report["source_inputs"]))
        self.assertTrue(any(item["file"] == "shared/nested.inc" for item in report["findings"]))

    def test_out_of_package_path_module_with_children_fails_closed(self) -> None:
        crate = self.root / "crates/termrock-core"
        write(crate / "src/lib.rs", '#[path = "../../../shared/entry.rs"]\npub mod external;\n')
        write(self.root / "shared/entry.rs", "pub mod hidden;\n")
        write(self.root / "shared/hidden.rs", "// captured terminal frame in path child\n")
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("Rust source input is outside its package; module tree is not fully accounted" in item and "src/lib.rs:1" in item for item in report["unaccounted_inputs"]))
        self.assertTrue(any(row["path"] == "shared/entry.rs" for row in report["source_inputs"]))
        self.assertFalse(any(row["path"] == "shared/hidden.rs" for row in report["source_inputs"]))

    def test_out_of_package_rust_include_with_children_fails_closed(self) -> None:
        crate = self.root / "crates/termrock-core"
        write(crate / "src/lib.rs", 'include!("../../../shared/entry.rs");\n')
        write(self.root / "shared/entry.rs", "pub mod hidden;\n")
        write(self.root / "shared/hidden.rs", "// captured terminal frame in include child\n")
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("Rust source input is outside its package; module tree is not fully accounted" in item and "src/lib.rs:1" in item for item in report["unaccounted_inputs"]))
        self.assertTrue(any(row["path"] == "shared/entry.rs" for row in report["source_inputs"]))
        self.assertFalse(any(row["path"] == "shared/hidden.rs" for row in report["source_inputs"]))

    def test_transitive_cross_package_rust_include_fails_closed_and_scans_known_inputs(self) -> None:
        crate = self.root / "crates/termrock-core"
        write(crate / "src/lib.rs", 'include!("../../../shared/entry.inc");\n')
        write(self.root / "shared/entry.inc", 'include!("nested.inc");\n')
        write(self.root / "shared/nested.inc", "// captured terminal frame in transitive source\n")
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("Rust source input is outside its package; module tree is not fully accounted" in item and "src/lib.rs:1" in item for item in report["unaccounted_inputs"]))
        self.assertTrue(any(row["path"] == "shared/entry.inc" for row in report["source_inputs"]))
        self.assertTrue(any(row["path"] == "shared/nested.inc" for row in report["source_inputs"]))
        self.assertTrue(any(item["file"] == "shared/nested.inc" for item in report["findings"]))

    def test_out_of_package_include_str_is_accounted_as_data(self) -> None:
        crate = self.root / "crates/termrock-core"
        write(crate / "src/lib.rs", 'pub const DATA: &str = include_str!("../../../shared/data.txt");\npub const BYTES: &[u8] = include_bytes!("../../../shared/data.txt");\n')
        write(self.root / "shared/data.txt", "pub mod hidden;\n")
        report = self.run_gate()
        self.assertTrue(report["architecture_checks_passed"], report["errors"])
        self.assertTrue(any(row["path"] == "shared/data.txt" for row in report["source_inputs"]))

    def test_include_macro_families_comments_and_all_delimiters(self) -> None:
        crate = self.root / "crates/termrock-core"
        delimiters = (("(", ")"), ("{", "}"), ("[", "]"))
        lines = []
        for macro in ("include", "include_str", "include_bytes"):
            for opening, closing in delimiters:
                path = "../../../shared/source.rs" if macro == "include" else "../../../shared/data.txt"
                if macro == "include_str" and opening == "{":
                    lines.append(f'{macro} // head comment\n! // delimiter comment\n {opening}"{path}"{closing};')
                else:
                    lines.append(f'{macro} /* head */ ! /* before delimiter */ {opening}"{path}"{closing};')
        write(crate / "src/lib.rs", "\n".join(lines) + "\n")
        write(self.root / "shared/source.rs", "pub fn included() {}\n")
        write(self.root / "shared/data.txt", "static text payload\n")
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        outside_rust_refs = [item for item in report["unaccounted_inputs"] if "Rust source input is outside its package" in item]
        self.assertEqual(len(outside_rust_refs), 3)
        self.assertFalse(any("invocation syntax is unsupported" in item for item in report["unaccounted_inputs"]))
        self.assertTrue(any(row["path"] == "shared/source.rs" for row in report["source_inputs"]))
        self.assertTrue(any(row["path"] == "shared/data.txt" for row in report["source_inputs"]))

    def test_unsupported_include_literals_and_delimiters_fail_closed(self) -> None:
        write(self.root / "crates/termrock-core/src/lib.rs", '\n'.join((
            'include!(concat!("nested.inc"));',
            'include!(r#"raw.inc"#);',
            'include!("escaped\\\\path.inc");',
            'include!{"unterminated.inc"',
            'include /* unterminated comment',
            '',
        )))
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        unsupported = [item for item in report["unaccounted_inputs"] if "invocation syntax is unsupported" in item]
        self.assertEqual(len(unsupported), 5)
        self.assertTrue(any(":1 " in item for item in unsupported))
        self.assertTrue(any(":2 " in item for item in unsupported))
        self.assertTrue(any(":3 " in item for item in unsupported))
        self.assertTrue(any(":4 " in item for item in unsupported))
        self.assertTrue(any(":5 " in item for item in unsupported))

    def test_valid_rust_literals_do_not_hide_following_include(self) -> None:
        source = '''\
const MULTILINE: &str = "first line
second line";
const ESCAPED_NEWLINE: &str = "first line\\
    second line";
const QUOTE: char = '"';
const BYTE_QUOTE: u8 = b'"';
const RAW: &str = r#"include!("not-real.inc")"#;
fn borrow<'a>(value: &'a str) -> &'a str { value }
include!("after_literals.inc");
'''
        write(self.root / "crates/termrock-core/src/lib.rs", source)
        write(self.root / "crates/termrock-core/src/after_literals.inc", "// captured terminal frame after literals\n")
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any(row["path"] == "crates/termrock-core/src/after_literals.inc" for row in report["source_inputs"]))
        self.assertTrue(any(item["file"] == "crates/termrock-core/src/after_literals.inc" for item in report["findings"]))
        self.assertFalse(any("Rust lexical uncertainty" in item for item in report["unaccounted_inputs"]))
        self.assertFalse(any("not-real.inc" in item for item in report["errors"]))

    def test_unterminated_string_reports_lexical_uncertainty(self) -> None:
        write(self.root / "crates/termrock-core/src/lib.rs", 'const BROKEN: &str = "unterminated\ninclude!(hidden.inc);\n')
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("Rust lexical uncertainty prevents complete source scan" in item for item in report["unaccounted_inputs"]))

    def test_path_attribute_file_alias_into_target_is_unaccounted(self) -> None:
        crate = self.root / "crates/termrock-core"
        write(crate / "src/lib.rs", '#[path = "generated.rs"]\npub mod generated;\n')
        write(self.root / "target/generated.rs", "pub fn generated() {}\n")
        (crate / "src/generated.rs").symlink_to(self.root / "target/generated.rs")
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("path module resolves inside Cargo target directory" in item and "src/lib.rs:1" in item for item in report["unaccounted_inputs"]))

    def test_raw_path_attribute_outside_package_fails_closed(self) -> None:
        write(self.root / "crates/termrock-core/src/lib.rs", '#[path = r"../../../shared/entry.rs"]\npub mod external;\n')
        write(self.root / "shared/entry.rs", "pub mod hidden;\n")
        write(self.root / "shared/hidden.rs", "// captured terminal frame\n")
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any(row["path"] == "shared/entry.rs" for row in report["source_inputs"]))
        self.assertTrue(any("outside its package; module tree is not fully accounted" in item and "src/lib.rs:1" in item for item in report["unaccounted_inputs"]))

    def test_commented_nested_cfg_attr_path_fails_closed(self) -> None:
        write(self.root / "crates/termrock-core/src/lib.rs", '#[cfg_attr(all(), path /* comment */ = /* path */ "../../../shared/entry.rs")]\npub mod external;\n')
        write(self.root / "shared/entry.rs", "pub mod hidden;\n")
        write(self.root / "shared/hidden.rs", "// captured terminal frame\n")
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any(row["path"] == "shared/entry.rs" for row in report["source_inputs"]))
        self.assertTrue(any("outside its package; module tree is not fully accounted" in item and "src/lib.rs:1" in item for item in report["unaccounted_inputs"]))

    def test_dynamic_path_attribute_is_explicitly_unaccounted(self) -> None:
        write(self.root / "crates/termrock-core/src/lib.rs", '#[path = concat!("generated", ".rs")]\npub mod generated;\n')
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("path attribute cannot be statically enumerated" in item and "src/lib.rs:1" in item for item in report["unaccounted_inputs"]))

    def test_nested_inline_module_alias_into_target_is_unaccounted(self) -> None:
        crate = self.root / "crates/termrock-core"
        write(crate / "src/lib.rs", "pub mod inner {\n    pub mod target;\n}\n")
        write(self.root / "target/mod.rs", "// captured terminal frame\n")
        (crate / "src/inner").mkdir(parents=True, exist_ok=True)
        (crate / "src/inner/target").symlink_to(self.root / "target", target_is_directory=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("production module may resolve through a Cargo target-directory alias" in item and "src/lib.rs:2" in item for item in report["unaccounted_inputs"]))

    def test_missing_live_include_does_not_fall_back_to_git_index(self) -> None:
        include = self.root / "crates/termrock-core/generated.inc"
        write(self.root / "crates/termrock-core/src/lib.rs", 'include!("../generated.inc");\n')
        write(include, "// captured terminal frame\n")
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        include.unlink()
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("unreadable or out-of-root include" in item and "generated.inc" in item for item in report["errors"]))
        self.assertFalse(any(row["path"] == "crates/termrock-core/generated.inc" for row in report["source_inputs"]))

    def test_custom_build_target_outside_src_is_hashed_and_unresolved(self) -> None:
        write(self.root / "crates/termrock-core/Cargo.toml", '[package]\nname="termrock-core"\nversion="0.1.0"\nedition="2024"\nbuild="build/custom.rs"\n')
        write(self.root / "crates/termrock-core/build/custom.rs", "fn main() {}\n")
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any(row["path"] == "crates/termrock-core/build/custom.rs" for row in report["source_inputs"]))
        self.assertTrue(any("build-generated production inputs" in item and "build/custom.rs" in item for item in report["unaccounted_inputs"]))

    def test_nested_include_and_path_module_are_followed(self) -> None:
        write(self.root / "crates/termrock-core/src/lib.rs", 'include!("../first.inc");\n')
        write(self.root / "crates/termrock-core/first.inc", '#[path = "generated/second.rs"]\nmod nested;\n')
        write(self.root / "crates/termrock-core/generated/second.rs", "// captured terminal frame\n")
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any(row["path"] == "crates/termrock-core/generated/second.rs" and row.get("included_by") == "crates/termrock-core/first.inc" for row in report["source_inputs"]))
        self.assertTrue(any(item["file"] == "crates/termrock-core/generated/second.rs" for item in report["findings"]))

    def test_recursive_include_cycle_fails_closed(self) -> None:
        write(self.root / "crates/termrock-core/src/lib.rs", 'include!("../first.inc");\n')
        write(self.root / "crates/termrock-core/first.inc", 'include!("second.inc");\n')
        write(self.root / "crates/termrock-core/second.inc", 'include!("first.inc");\n')
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)
        report = self.run_gate()
        self.assertFalse(report["architecture_checks_passed"])
        self.assertTrue(any("recursive production source include cycle" in error for error in report["errors"]))


if __name__ == "__main__":
    unittest.main()
