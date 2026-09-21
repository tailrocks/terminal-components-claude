#!/usr/bin/env python3
"""Source-discovery mutation proofs. Parsing is not execution."""
import json
from pathlib import Path
import tempfile
import unittest

import source_discovery as disc
from inventory import Invalid


def write_tree(base, files):
    for name, text in files.items():
        path = base / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)


def fixture_workspace(files, seed_lines=None):
    temp = tempfile.TemporaryDirectory(prefix="source-discovery-")
    root = Path(temp.name) / "repo"
    root.mkdir()
    write_tree(root, files)
    seed = None
    if seed_lines is not None:
        seed = "\n".join(f"- `{path}`" for path in seed_lines) + "\n"
    result = disc.discover(root, seed, seed_expected=None if seed_lines is None else len(seed_lines))
    return temp, root, result


WORKSPACE = '''[workspace]
resolver = "3"
members = ["crate"]
'''

CRATE = '''[package]
name = "demo"
version = "0.0.0"
edition = "2024"
'''


class Parser(unittest.TestCase):
    def names(self, files, seed=None):
        self.temp, self.root, result = fixture_workspace(files, seed)
        self.addCleanup(self.temp.cleanup)
        return result

    def identities(self, result, origin=None):
        rows = result["identities"]
        if origin:
            rows = [row for row in rows if row["origin"] == origin]
        return [row["identity"] for row in rows]

    def test_comments_and_strings_are_not_tests(self):
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE,
            "crate/src/lib.rs": '''
/// ```
/// #[test]
/// fn doctest_real() {}
/// ```
// #[test]
// fn commented() {}
const S: &str = "#[test] fn in_string() {}";
#[test]
fn real() { assert_eq!(1, 1); }
''',
        })
        self.assertEqual(self.identities(result, "fn-test"), ["real"])
        self.assertTrue(any(row["origin"] == "rustdoc" for row in result["identities"]))

    def test_cfg_test_module_and_external_mod(self):
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE,
            "crate/src/lib.rs": '''
mod app;
''',
            "crate/src/app.rs": '''
#[cfg(test)]
mod tests {
    #[test]
    fn inline() { assert!(true); }
}
#[cfg(test)]
mod historical_tests;
''',
            "crate/src/app/historical_tests.rs": '''
#[test]
fn restored() { assert_eq!(2, 2); }
''',
        })
        self.assertEqual(sorted(self.identities(result, "fn-test")), [
            "app::historical_tests::restored",
            "app::tests::inline",
        ])

    def test_macro_definition_is_not_an_identity(self):
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE,
            "crate/src/lib.rs": '''
macro_rules! suite {
    ($name:ident) => {
        mod $name {
            #[test]
            fn generated() { assert!(true); }
        }
    };
}
suite!(alpha);
''',
        })
        self.assertEqual(self.identities(result, "fn-test"), [])
        self.assertEqual(self.identities(result, "macro"), ["alpha::generated"])

    def test_exported_suite_macro_expands_in_other_crate(self):
        result = self.names({
            "Cargo.toml": '[workspace]\nresolver="3"\nmembers=["testing","crate"]\n',
            "testing/Cargo.toml": '[package]\nname="testing"\nversion="0.0.0"\nedition="2024"\n',
            "testing/src/lib.rs": '''
#[macro_export]
macro_rules! conformance_suite {
    ($($name:ident => $case:ty),+ $(,)?) => {
        $(
            mod $name {
                #[test] fn disabled_cannot_activate() {}
            }
        )+
    };
}
''',
            "crate/Cargo.toml": CRATE,
            "crate/src/lib.rs": "",
            "crate/tests/conformance.rs": '''
conformance_suite!(probe => ProbeCase, button => ButtonCase);
''',
        })
        names = self.identities(result, "macro")
        self.assertIn("probe::disabled_cannot_activate", names)
        self.assertIn("button::disabled_cannot_activate", names)

    def test_conformance_suite_expands_each_entry(self):
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE,
            "crate/src/lib.rs": '''
macro_rules! conformance_suite {
    ($($name:ident => $case:ty),+ $(,)?) => {
        $(
            mod $name {
                #[test] fn name_matches_the_module() {}
                #[test] fn disabled_cannot_activate() {}
            }
        )+
    };
}
conformance_suite!(
    probe => ProbeCase,
    button => ButtonCase,
);
''',
        })
        names = self.identities(result, "macro")
        self.assertIn("probe::disabled_cannot_activate", names)
        self.assertIn("button::name_matches_the_module", names)
        self.assertEqual(len(names), 4)

    def test_matrix_and_nested_modules(self):
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE,
            "crate/src/lib.rs": '''
macro_rules! matrix {
    ($comp:ident, $c:expr) => {
        mod $comp {
            #[test] fn default() {}
            #[test] fn focused() {}
        }
    };
}
mod render {
    mod components {
        matrix!(button, Comp::Button);
    }
}
''',
        })
        self.assertEqual(sorted(self.identities(result, "macro")), [
            "render::components::button::default",
            "render::components::button::focused",
        ])

    def test_ignore_reason_and_oracle_conflict_classification(self):
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE.replace('name = "demo"', 'name = "holla"'),
            "crate/src/lib.rs": '''
mod app;
mod scenario;
''',
            "crate/src/app.rs": '''
#[cfg(test)]
mod tests {
    #[test]
    fn home_hint_casing_preserves_lowercase_physical_shortcuts() {
        assert!(true);
    }
    #[test]
    #[ignore = "pty helper"]
    fn helper() { assert!(false); }
}
''',
            "crate/src/scenario.rs": '''
#[cfg(test)]
mod tests {
    #[test]
    fn names_round_trip() { assert_eq!(1, 1); }
}
''',
        })
        by_id = {row["identity"]: row for row in result["identities"] if row["origin"] == "fn-test"}
        self.assertEqual(by_id["app::tests::home_hint_casing_preserves_lowercase_physical_shortcuts"]["classification"],
                         "oracle-conflict")
        self.assertEqual(by_id["app::tests::home_hint_casing_preserves_lowercase_physical_shortcuts"]["assertions"][0]["classification"],
                         "oracle-conflict")
        self.assertEqual(by_id["scenario::tests::names_round_trip"]["classification"], "oracle-conflict")
        self.assertEqual(by_id["app::tests::helper"]["ignored"], "pty helper")
        self.assertEqual(by_id["app::tests::helper"]["classification"], "preserve")

    def test_integration_crate_root_loads_sibling_mod(self):
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE,
            "crate/src/lib.rs": "",
            "crate/tests/app_tests.rs": '''
mod support;
#[test]
fn uses_support() { assert_eq!(support::SEED, 1); }
''',
            "crate/tests/support.rs": "pub const SEED: u8 = 1;\n",
        })
        self.assertIn("crate/tests/support.rs", result["scanned"])
        self.assertFalse(any("missing module support" in b.get("reason", "") for b in result["blockers"]))
        self.assertIn("uses_support", self.identities(result, "fn-test"))

    def test_trybuild_and_path_attr_and_include_docs(self):
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE,
            "crate/README.md": '''# Demo
```rust
assert_eq!(2 + 2, 4);
```
''',
            "crate/src/lib.rs": '''
#![doc = include_str!("../README.md")]
#[test]
fn compile_fail_cases_hold() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
''',
            "crate/tests/render.rs": '''
#[path = "fixtures/text.rs"]
mod fixtures_text;
#[test]
fn uses_fixture() { assert_eq!(fixtures_text::SEED, "x"); }
''',
            "crate/tests/fixtures/text.rs": "pub const SEED: &str = \"x\";\n",
            "crate/tests/ui/must_use.rs": "fn main() {}\n",
        })
        names = {(row["origin"], row["identity"]) for row in result["identities"]}
        self.assertIn(("fn-test", "compile_fail_cases_hold"), names)
        self.assertIn(("fn-test", "uses_fixture"), names)
        self.assertIn(("trybuild", "must_use"), names)
        self.assertTrue(any(row["origin"] == "rustdoc" and row["path"].endswith("README.md")
                            for row in result["identities"]))
        self.assertIn("crate/tests/fixtures/text.rs", result["scanned"])

    def test_unknown_test_generating_macro_is_a_blocker(self):
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE,
            "crate/src/lib.rs": '''
mystery!(alpha);
''',
        })
        # mystery! has no definition and no #[test] in the invocation body.
        self.assertEqual(result["identities"], [])
        self.assertEqual(result["blockers"], [])
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE,
            "crate/src/lib.rs": '''
mystery! {
    #[test]
    fn hidden() {}
}
''',
        })
        self.assertTrue(any("unexpanded" in b.get("reason", "") for b in result["blockers"]))

    def test_duplicate_identity_fails(self):
        with self.assertRaises(Invalid):
            self.names({
                "Cargo.toml": WORKSPACE,
                "crate/Cargo.toml": CRATE,
                "crate/src/lib.rs": '''
#[test] fn twice() {}
#[test] fn twice() {}
''',
            })

    def test_seed_path_must_exist_and_unreached_is_blocker(self):
        with self.assertRaises(Invalid):
            self.names({
                "Cargo.toml": WORKSPACE,
                "crate/Cargo.toml": CRATE,
                "crate/src/lib.rs": "#[test] fn a() {}\n",
            }, seed=["crate/src/missing.rs"])
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE,
            "crate/src/lib.rs": "#[test] fn a() {}\n",
            "crate/src/orphan.rs": "#[test] fn unused() {}\n",
        }, seed=["crate/src/lib.rs", "crate/src/orphan.rs"])
        self.assertTrue(any(b.get("path") == "crate/src/orphan.rs" for b in result["blockers"]))

    def test_doctest_is_nextest_blocker_not_execution(self):
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE,
            "crate/src/lib.rs": '''
/// ```
/// assert_eq!(2 + 2, 4);
/// ```
pub fn example() {}
#[test]
fn unit() {}
''',
        })
        self.assertTrue(any("doctest runner" in b.get("reason", "") for b in result["blockers"]))
        self.assertEqual(self.identities(result, "fn-test"), ["unit"])

    def test_baseline_combo_is_ignored(self):
        result = self.names({
            "Cargo.toml": WORKSPACE,
            "crate/Cargo.toml": CRATE,
            "crate/src/lib.rs": '''
macro_rules! baseline_combo_tests {
    ($mod_name:ident, |$cols:ident, $rows:ident, $color:ident| $body:expr) => {
        mod $mod_name {
            #[test]
            #[ignore = "visual baseline capture; run with --ignored"]
            fn c_72x20_truecolor() { $body }
        }
    };
}
macro_rules! baseline_case {
    ($fn_name:ident => $case:expr) => {
        baseline_combo_tests!($fn_name, |cols, rows, color| { $case; });
    };
}
baseline_case!(showcase_pages_overview_default_120x40_truecolor => Case::new());
''',
        })
        rows = [row for row in result["identities"] if row["origin"] == "macro"]
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["identity"], "showcase_pages_overview_default_120x40_truecolor::c_72x20_truecolor")
        self.assertIsNotNone(rows[0]["ignored"])


class RepoInventory(unittest.TestCase):
    def test_pinned_seed_is_reached_and_named_conflicts_exist(self):
        root = Path(__file__).resolve().parents[2]
        seed = root / "docs/refactoring-plan/inline-test-source-scope.md"
        if not seed.is_file():
            self.skipTest("campaign checkout not present")
        result = disc.discover(root, seed.read_text())
        self.assertEqual(len(result["seed_paths"]), 146)
        self.assertFalse(any("seed path not reached" in b.get("reason", "") for b in result["blockers"]))
        names = {(row["package"], row["identity"]) for row in result["identities"]}
        self.assertIn(("holla", "app::tests::home_hint_casing_preserves_lowercase_physical_shortcuts"), names)
        self.assertIn(("holla", "app::tests::home_escape_clears_scope_before_canonical_query"), names)
        self.assertIn(("holla", "scenario::tests::names_round_trip"), names)
        self.assertTrue(any(row["identity"] == "probe::disabled_cannot_activate" for row in result["identities"]))
        self.assertTrue(any(row["identity"] == "render::components::button::default" for row in result["identities"]))
        self.assertTrue(any(b.get("reason", "").startswith("cargo-nextest") for b in result["blockers"]))
        required = json.loads((root / "tools/test-inventory/required.json").read_text())
        self.assertEqual(required["approval"], "pending")
        conflicts = json.loads((root / "tools/test-inventory/conflicts.json").read_text())
        self.assertEqual(conflicts["approval"], "pending")
        conflict_ids = {(row["package"], row["identity"]) for row in conflicts["conflicts"]}
        self.assertIn(("holla", "app::tests::home_hint_casing_preserves_lowercase_physical_shortcuts"), conflict_ids)
        self.assertIn(("holla", "app::tests::home_escape_clears_scope_before_canonical_query"), conflict_ids)
        self.assertIn(("holla", "scenario::tests::names_round_trip"), conflict_ids)


class SeedLoader(unittest.TestCase):
    def test_seed_count_is_exact(self):
        text = "\n".join(f"- `p{i}.rs`" for i in range(146)) + "\n"
        self.assertEqual(len(disc.load_seed_paths(text)), 146)
        with self.assertRaises(Invalid):
            disc.load_seed_paths("- `only.rs`\n")


if __name__ == "__main__":
    unittest.main()
