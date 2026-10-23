use std::path::{Path, PathBuf};

use termrock_xtask::ownership::{check_workspace, exceptions, module_graph, rules};

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|crates| crates.parent())
        .expect("xtask crate lives two levels below the workspace root")
        .to_path_buf()
}

#[test]
fn ownership_check_passes_on_workspace() {
    let root = workspace_root();
    let exceptions = root
        .join("crates")
        .join("termrock-xtask")
        .join("exceptions")
        .join("v1.json");
    let report = check_workspace(&root, &exceptions);
    assert!(
        report.files_scanned > 250,
        "the check must scan the whole tree, got {} files",
        report.files_scanned
    );
    assert!(
        report.passed(),
        "ownership check failed:\n{}",
        termrock_xtask::ownership::render_human(&report)
    );
}

fn scratch_workspace(member_src: &str, exceptions_json: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("scratch dir");
    let root = dir.path();
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/demo-preview\"]\n",
    )
    .expect("write manifest");
    let member = root.join("crates").join("demo-preview");
    std::fs::create_dir_all(&member).expect("create member");
    std::fs::write(
        member.join("Cargo.toml"),
        "[package]\nname = \"demo-preview\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write member manifest");
    let src = member.join("src");
    std::fs::create_dir_all(&src).expect("create src");
    std::fs::write(src.join("lib.rs"), member_src).expect("write lib.rs");
    std::fs::write(root.join("exceptions.json"), exceptions_json).expect("write exceptions");
    dir
}

fn empty_exceptions() -> String {
    format!(
        r#"{{"schema":"{}","exceptions":[]}}"#,
        exceptions::SCHEMA_VERSION
    )
}

fn write_member_file(dir: &tempfile::TempDir, relative: &str, text: &str) {
    let path = dir.path().join("crates/demo-preview").join(relative);
    std::fs::create_dir_all(path.parent().expect("fixture has a parent"))
        .expect("create fixture parent");
    std::fs::write(path, text).expect("write fixture file");
}

fn own14_findings(report: &termrock_xtask::ownership::Report) -> Vec<(String, usize)> {
    let mut findings: Vec<_> = report
        .findings
        .iter()
        .filter(|finding| finding.rule == "OWN-14")
        .map(|finding| (finding.file.clone(), finding.line))
        .collect();
    findings.sort();
    findings
}

fn empty_workspace() -> tempfile::TempDir {
    scratch_workspace("", &empty_exceptions())
}

fn check_scratch(dir: &tempfile::TempDir) -> termrock_xtask::ownership::Report {
    let root = std::fs::canonicalize(dir.path()).expect("canonical scratch workspace");
    check_workspace(&root, &root.join("exceptions.json"))
}

fn assert_skipped_body_attribute_is_conservative(attribute: &str, local_item: &str) {
    let dir = empty_workspace();
    let source = format!(
        "#[cfg(test)] mod candidate;\n\
         #[path = \"../before.rs\"] mod before;\n\
         fn production() {{\n\
             {attribute}\n\
             {local_item}\n\
         }}\n\
         include!(\"../after.inc.rs\");\n\
         #[path = \"../after.rs\"] mod after;\n"
    );
    write_member_file(&dir, "src/lib.rs", &source);
    for (path, name) in [
        ("src/candidate.rs", "candidate"),
        ("before.rs", "before"),
        ("after.inc.rs", "included_after"),
        ("after.rs", "module_after"),
    ] {
        write_member_file(
            &dir,
            path,
            &format!("fn {name}() {{ None::<u8>.expect(\"{name}\"); }}\n"),
        );
    }

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    for path in ["before.rs", "after.inc.rs", "after.rs"] {
        let external = std::fs::canonicalize(member.join(path)).expect("external route");
        assert!(
            analysis.sources.contains(&external),
            "known sibling route {path} must remain inventoried: {analysis:?}"
        );
    }
    let findings = own14_findings(&check_scratch(&dir));
    assert!(
        findings
            .iter()
            .any(|(path, _)| path == "crates/demo-preview/src/candidate.rs"),
        "the cfg(test) candidate must remain in the production ownership inventory: {findings:?}"
    );
    assert!(
        !analysis.complete,
        "unmodeled skipped-body attribute {attribute} must make provenance incomplete"
    );
    assert!(
        analysis.test_only.is_empty(),
        "unmodeled skipped-body attributes never authorize pruning"
    );
    assert_eq!(
        findings,
        [
            ("crates/demo-preview/after.inc.rs".to_string(), 1),
            ("crates/demo-preview/after.rs".to_string(), 1),
            ("crates/demo-preview/before.rs".to_string(), 1),
            ("crates/demo-preview/src/candidate.rs".to_string(), 1),
        ]
    );
}

fn assert_uncertain_module_attribute_keeps_candidate(attribute: &str) {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        &format!("#[cfg(test)]\n{attribute}\nmod candidate;\n"),
    );
    write_member_file(
        &dir,
        "src/candidate.rs",
        "fn body() { None::<u8>.expect(\"candidate\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let candidate =
        std::fs::canonicalize(member.join("src/candidate.rs")).expect("candidate source");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(
        !analysis.complete,
        "uncertain attribute {attribute} must make provenance incomplete"
    );
    assert!(
        analysis.test_only.is_empty(),
        "uncertain attribute {attribute} must not prove test-only sources"
    );
    assert!(
        analysis.sources.contains(&candidate),
        "the uncertain candidate must stay in the source graph: {analysis:?}"
    );
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/candidate.rs".to_string(), 1)],
        "the candidate's actual OWN-14 finding must remain"
    );
}

fn assert_shadowed_test_attribute_keeps_candidate(imports: &str, attribute: &str) {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        &format!("{imports}\n#[cfg(test)]\n{attribute}\nmod candidate;\n"),
    );
    write_member_file(
        &dir,
        "src/candidate.rs",
        "fn body() { None::<u8>.expect(\"candidate\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let candidate =
        std::fs::canonicalize(member.join("src/candidate.rs")).expect("candidate source");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(
        !analysis.complete,
        "possibly shadowed attribute {attribute} must make provenance incomplete"
    );
    assert!(analysis.test_only.is_empty());
    assert!(analysis.sources.contains(&candidate));
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/candidate.rs".to_string(), 1)],
        "the potentially shadowed candidate must retain its actual OWN-14 finding"
    );
}

fn assert_unshadowed_test_import_keeps_test_provenance(imports: &str) {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        &format!("{imports}\n#[cfg(test)]\n#[test]\nmod candidate;\n"),
    );
    write_member_file(&dir, "src/candidate.rs", "fn body() {}\n");

    let member = dir.path().join("crates/demo-preview");
    let candidate =
        std::fs::canonicalize(member.join("src/candidate.rs")).expect("candidate source");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(analysis.complete, "non-shadowing import: {imports}");
    assert!(
        analysis.test_only.contains(&candidate),
        "the exact test-only route should remain proven: {analysis:?}"
    );
}

fn assert_skipped_body_import_keeps_candidate(imports: &str) {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        &format!(
            "#[cfg(test)] mod candidate;\n\
             fn production() {{ {imports} #[test] fn generated() {{}} }}\n"
        ),
    );
    write_member_file(
        &dir,
        "src/candidate.rs",
        "fn body() {{ None::<u8>.expect(\"candidate\"); }}\n",
    );
    let member = dir.path().join("crates/demo-preview");
    let candidate =
        std::fs::canonicalize(member.join("src/candidate.rs")).expect("candidate source");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(
        !analysis.complete,
        "local import must be unresolved: {imports}"
    );
    assert!(analysis.test_only.is_empty());
    assert!(analysis.sources.contains(&candidate));
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/candidate.rs".to_string(), 1)]
    );
}

fn assert_enclosing_import_keeps_skipped_body_candidate(
    imports_before: &str,
    body: &str,
    imports_after: &str,
) {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        &format!(
            "#[cfg(test)] mod candidate;\n\
             {imports_before}\n\
             fn production() {{\n{body}\n}}\n\
             {imports_after}\n"
        ),
    );
    write_member_file(
        &dir,
        "src/candidate.rs",
        "fn body() { None::<u8>.expect(\"candidate\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let candidate =
        std::fs::canonicalize(member.join("src/candidate.rs")).expect("candidate source");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(
        !analysis.complete,
        "an enclosing import may shadow a skipped-body attribute: {analysis:?}"
    );
    assert!(
        analysis.test_only.is_empty(),
        "uncertain enclosing imports must not authorize test-only pruning"
    );
    assert!(analysis.sources.contains(&candidate));
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/candidate.rs".to_string(), 1)],
        "the actual OWN-14 finding must remain"
    );
}

fn assert_unshadowed_skipped_body_attribute_proves_candidate(body: &str) {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        &format!(
            "#[cfg(test)] mod candidate;\n\
             fn production() {{\n{body}\n}}\n"
        ),
    );
    write_member_file(&dir, "src/candidate.rs", "fn body() {}\n");

    let member = dir.path().join("crates/demo-preview");
    let candidate =
        std::fs::canonicalize(member.join("src/candidate.rs")).expect("candidate source");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(
        analysis.complete,
        "built-in attribute context: {analysis:?}"
    );
    assert!(analysis.test_only.contains(&candidate));
    assert!(own14_findings(&check_scratch(&dir)).is_empty());
}

#[test]
fn cfg_test_external_module_is_excluded_but_neighboring_production_is_scanned() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "fn before() { None::<u8>.expect(\"before\"); }\n\
         #[cfg(test)]\n\
         mod picker_contract;\n\
         fn between() { None::<u8>.expect(\"between\"); }\n\
         #[cfg(test)]\n\
         mod extra_contract;\n\
         fn after() { None::<u8>.expect(\"after\"); }\n",
    );
    write_member_file(
        &dir,
        "src/picker_contract.rs",
        "#[test]\nfn fixture_assertion() { None::<u8>.expect(\"fixture\"); }\n",
    );
    write_member_file(
        &dir,
        "src/extra_contract.rs",
        "fn helper() { None::<u8>.expect(\"fixture\"); }\n",
    );

    let report = check_scratch(&dir);
    assert_eq!(
        own14_findings(&report),
        [
            ("crates/demo-preview/src/lib.rs".to_string(), 1),
            ("crates/demo-preview/src/lib.rs".to_string(), 4),
            ("crates/demo-preview/src/lib.rs".to_string(), 7),
        ]
    );
}

#[test]
fn mixed_test_spans_never_mask_production_on_shared_physical_lines() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "fn before() { None::<u8>.expect(\"before\"); } #[cfg(test)] mod first {}\n\
         #[cfg(test)] mod second {} fn after() { None::<u8>.expect(\"after\"); }\n\
         #[cfg(test)] mod multiline {\n\
             fn hidden() { None::<u8>.expect(\"test-only\"); }\n\
         } fn after_close() { None::<u8>.expect(\"after-close\"); }\n",
    );

    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [
            ("crates/demo-preview/src/lib.rs".to_string(), 1),
            ("crates/demo-preview/src/lib.rs".to_string(), 2),
            ("crates/demo-preview/src/lib.rs".to_string(), 5),
        ]
    );
}

#[test]
fn literal_inline_cfg_test_opacity_preserves_other_production_rules() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "fn before() { buffer.set_string(0, 0, \"before\", Style::default()); None::<u8>.expect(\"before\"); }\n\
         #[allow(dead_code)]\n\
         #[cfg(test)]\n\
         mod test_helpers {\n\
             use super::*;\n\
             use super::{Buffer as Alias};\n\
             #[test]\n\
             fn helper() { buffer.set_string(0, 0, \"inside\", Style::default()); None::<u8>.expect(\"inside\"); }\n\
             #[derive(Clone)] struct Derived;\n\
             #[custom_rewrite] fn rewritten() {}\n\
             mod deeper { #[test] fn nested() { None::<u8>.expect(\"nested\"); } }\n\
         }\n\
         fn after() { buffer.set_string(0, 0, \"after\", Style::default()); None::<u8>.expect(\"after\"); }\n",
    );

    let report = check_scratch(&dir);
    let mut own14 = own14_findings(&report);
    own14.sort();
    assert_eq!(
        own14,
        [
            ("crates/demo-preview/src/lib.rs".to_string(), 1),
            ("crates/demo-preview/src/lib.rs".to_string(), 13),
        ],
        "only the literal cfg(test) body should be excluded: {:?}",
        report.findings
    );
    let mut paint_lines: Vec<_> = report
        .findings
        .iter()
        .filter(|finding| finding.rule == "OWN-02")
        .map(|finding| (finding.file.clone(), finding.line))
        .collect();
    paint_lines.sort();
    assert_eq!(
        paint_lines,
        [
            ("crates/demo-preview/src/lib.rs".to_string(), 1),
            ("crates/demo-preview/src/lib.rs".to_string(), 13),
        ],
        "production_lines based paint findings must use the same ranges"
    );
}

fn assert_inline_cfg_candidate_stays_production(attributes: &str) {
    let dir = empty_workspace();
    let source = format!(
        "{attributes}\nmod candidate {{\n    use super::*;\n    #[test]\n    fn body() {{ None::<u8>.expect(\"candidate\"); }}\n}}\n"
    );
    let body_line = source
        .lines()
        .position(|line| line.contains("fn body()"))
        .map(|index| index + 1)
        .unwrap_or(0);
    write_member_file(&dir, "src/lib.rs", &source);
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/lib.rs".to_string(), body_line)],
        "outer attributes must not grant opacity: {attributes}"
    );
}

#[test]
fn inline_cfg_test_opacity_requires_complete_compatible_outer_attributes() {
    for attributes in [
        "mod tests",
        "#[test]",
        "#[bench]",
        "#[cfg(any(test, feature = \"x\"))]",
        "#[cfg(not(test))]",
        "#[cfg(test) trailing]",
        "#[cfg::test]",
        "#[test::helper]\n#[cfg(test)]",
        "#[derive::Clone]\n#[cfg(test)]",
        "#[custom_rewrite]\n#[cfg(test)]",
        "#[cfg(test)]\n#[custom_rewrite]",
        "#[cfg_attr(test, path = \"candidate.rs\")]\n#[cfg(test)]",
        "#[path = \"candidate.rs\"]\n#[cfg(test)]",
        "#[cfg(test)]\n#[derive(Clone)]",
    ] {
        if attributes == "mod tests" {
            let dir = empty_workspace();
            write_member_file(
                &dir,
                "src/lib.rs",
                "mod tests { use super::*; #[test] fn body() { None::<u8>.expect(\"ordinary\"); } }\n",
            );
            assert_eq!(
                own14_findings(&check_scratch(&dir)),
                [("crates/demo-preview/src/lib.rs".to_string(), 1)],
                "an ordinary mod tests is not test-only"
            );
        } else {
            assert_inline_cfg_candidate_stays_production(attributes);
        }
    }
    for attribute in ["#[test]", "#[bench]"] {
        let dir = empty_workspace();
        let source = format!(
            "use custom::*;\n{attribute}\nmod candidate {{ #[test] fn body() {{ None::<u8>.expect(\"shadowed\"); }} }}\n"
        );
        write_member_file(&dir, "src/lib.rs", &source);
        assert_eq!(
            own14_findings(&check_scratch(&dir)).len(),
            1,
            "a possibly shadowed bare {attribute} cannot prove cfg(test)"
        );
    }
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)] mod malformed { fn body() { None::<u8>.expect(\"open\");\n",
    );
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/lib.rs".to_string(), 1)],
        "unclosed delimiters remain fail-closed"
    );
}

#[test]
fn inline_cfg_test_range_mode_does_not_change_member_graph_routes() {
    let dir = empty_workspace();
    let member = std::fs::canonicalize(dir.path().join("crates/demo-preview"))
        .unwrap_or_else(|error| panic!("canonicalize member root: {error}"));
    std::fs::write(
        member.join("Cargo.toml"),
        "[package]\nname = \"demo-preview\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\
         [[test]]\nname = \"ownership-root\"\npath = \"tests/ownership_root.rs\"\n",
    )
    .unwrap_or_else(|error| panic!("write explicit test target: {error}"));
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)] mod ambiguous { use super::*; #[test] fn body() {} }\n\
         #[path = \"../outside/shared_path.rs\"] mod shared;\n\
         include!(\"../outside/lib.inc.rs\");\n\
         unknown_routes!({ #[path = \"../outside/macro_only.rs\"] mod hidden; });\n\
         #[path = \"../outside/later.rs\"] mod later;\n",
    );
    write_member_file(
        &dir,
        "src/main.rs",
        "#[path = \"../outside/shared_path.rs\"] mod shared_again;\n",
    );
    write_member_file(
        &dir,
        "tests/ownership_root.rs",
        "#[cfg(test)] mod ambiguous { use super::*; #[test] fn body() {} }\n\
         #[path = \"../outside/test_path.rs\"] mod test_path;\n\
         include!(\"../outside/test.inc.rs\");\n",
    );
    for relative in [
        "outside/shared_path.rs",
        "outside/lib.inc.rs",
        "outside/later.rs",
        "outside/test_path.rs",
        "outside/test.inc.rs",
        "outside/macro_only.rs",
    ] {
        write_member_file(&dir, relative, "fn routed() {}\n");
    }

    let analysis = module_graph::analyze_member(&member);
    let mut actual: Vec<_> = analysis
        .sources
        .iter()
        .map(|path| {
            path.strip_prefix(&member)
                .unwrap_or_else(|_| panic!("source escaped member: {}", path.display()))
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        [
            "outside/later.rs",
            "outside/lib.inc.rs",
            "outside/shared_path.rs",
            "outside/test.inc.rs",
            "outside/test_path.rs",
            "src/lib.rs",
            "src/main.rs",
            "tests/ownership_root.rs",
        ],
        "known later siblings, dual routes, and explicit test roots stay inventoried"
    );
    assert!(!analysis.complete, "ambiguous routes remain conservative");
    assert!(
        analysis.test_only.is_empty(),
        "explicit test roots and ambiguous full-graph bodies stay production"
    );
}

#[test]
fn literal_path_test_child_is_pruned() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)]\n#[path = \"test_helpers/picker.rs\"]\nmod picker_contract;\n\
         fn production() { None::<u8>.expect(\"production\"); }\n",
    );
    write_member_file(
        &dir,
        "src/test_helpers/picker.rs",
        "#[test]\nfn picker() { None::<u8>.expect(\"fixture\"); }\n",
    );

    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/lib.rs".to_string(), 4)]
    );
}

#[test]
fn non_mod_rs_path_attribute_uses_source_file_directory_not_module_directory() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)] #[path = \"shared.rs\"] mod test_helper;\nmod outer;\n",
    );
    write_member_file(
        &dir,
        "src/outer.rs",
        "#[path = \"shared.rs\"] mod production;\n",
    );
    write_member_file(
        &dir,
        "src/shared.rs",
        "fn production() { None::<u8>.expect(\"shared production\"); }\n",
    );
    write_member_file(&dir, "src/outer/shared.rs", "fn unrelated() {}\n");

    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/shared.rs".to_string(), 1)]
    );
}

#[test]
fn inline_path_attributes_follow_mod_rs_and_non_mod_rs_logical_directories() {
    let dir = empty_workspace();
    write_member_file(&dir, "src/lib.rs", "mod outer;\n");
    write_member_file(
        &dir,
        "src/outer.rs",
        "fn production() { None::<u8>.expect(\"outer\"); }\n\
         mod inline { #[cfg(test)] #[path = \"helpers/child.rs\"] mod helper; }\n",
    );
    write_member_file(
        &dir,
        "src/outer/inline/helpers/child.rs",
        "fn fixture() { None::<u8>.expect(\"non-mod-rs test helper\"); }\n",
    );
    write_member_file(&dir, "src/nested_root.rs", "mod outer_mod;\n");
    write_member_file(
        &dir,
        "src/outer_mod/mod.rs",
        "fn production() { None::<u8>.expect(\"mod-rs\"); }\n\
         mod inline { #[cfg(test)] #[path = \"helpers/child.rs\"] mod helper; }\n",
    );
    write_member_file(
        &dir,
        "src/outer_mod/inline/helpers/child.rs",
        "fn fixture() { None::<u8>.expect(\"mod-rs test helper\"); }\n",
    );
    write_member_file(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"demo-preview\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\
         [[bin]]\nname = \"nested-root\"\npath = \"src/nested_root.rs\"\n",
    );

    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [
            ("crates/demo-preview/src/outer.rs".to_string(), 1),
            ("crates/demo-preview/src/outer_mod/mod.rs".to_string(), 1),
        ]
    );
}

#[test]
fn expression_macros_do_not_disable_test_module_proof() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "const VALUES: Vec<u8> = vec![1, 2, 3];\n\
         #[cfg(test)]\nmod test_helpers;\n\
         fn production() { None::<u8>.expect(\"production\"); }\n",
    );
    write_member_file(
        &dir,
        "src/test_helpers.rs",
        "fn fixture() { None::<u8>.expect(\"fixture\"); }\n",
    );

    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/lib.rs".to_string(), 4)]
    );
}

#[test]
fn test_names_any_cfg_and_not_test_remain_production_capable() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "mod tests;\nmod test;\n#[cfg(any(test, feature = \"extra\"))]\nmod any_test;\n#[cfg(not(test))]\nmod not_test;\n",
    );
    for module in ["tests", "test", "any_test", "not_test"] {
        write_member_file(
            &dir,
            &format!("src/{module}.rs"),
            "fn production() { None::<u8>.expect(\"production\"); }\n",
        );
    }

    let report = check_scratch(&dir);
    assert_eq!(own14_findings(&report).len(), 4);
}

#[test]
fn complete_cfg_predicates_and_repeated_attributes_prove_test_only() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(all(test, feature = \"extra\",))]\nmod all_test;\n\
         #[cfg(not(not(test)))]\nmod double_not;\n\
         #[cfg(test)]\n#[cfg(not(feature = \"extra\"))]\nmod repeated;\n\
         #[cfg(not(feature = \"extra\"))]\n#[cfg(test)]\nmod reversed;\n\
         #[cfg(all(test, false))]\nmod always_false;\n\
         #[cfg(\n             all(test,\n                 feature = \"extra\",\n             )\n         )]\nmod multiline;\n\
         #[cfg(test)] mod same_line;\n\
         #[cfg(test)]\n#[cfg(not(test))]\nmod contradictory;\n",
    );
    for module in [
        "all_test",
        "double_not",
        "repeated",
        "reversed",
        "always_false",
        "multiline",
        "same_line",
        "contradictory",
    ] {
        write_member_file(
            &dir,
            &format!("src/{module}.rs"),
            "fn body() { None::<u8>.expect(\"body\"); }\n",
        );
    }

    let report = check_scratch(&dir);
    assert_eq!(
        own14_findings(&report),
        [
            ("crates/demo-preview/src/always_false.rs".to_string(), 1),
            ("crates/demo-preview/src/contradictory.rs".to_string(), 1),
        ]
    );
}

#[test]
fn dual_test_and_production_parents_keep_shared_source_production() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)]\n#[path = \"shared.rs\"]\nmod test_parent;\n",
    );
    write_member_file(
        &dir,
        "src/main.rs",
        "#[path = \"shared.rs\"]\nmod production_parent;\n",
    );
    write_member_file(
        &dir,
        "src/shared.rs",
        "fn shared() { None::<u8>.expect(\"shared\"); }\n",
    );

    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/shared.rs".to_string(), 1)]
    );
}

#[test]
fn test_module_descendant_with_a_production_route_remains_scanned() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)]\nmod outer;\n\
         fn production() { None::<u8>.expect(\"production\"); }\n",
    );
    write_member_file(&dir, "src/outer.rs", "mod grandchild;\n");
    write_member_file(
        &dir,
        "src/outer/grandchild.rs",
        "fn shared() { None::<u8>.expect(\"shared\"); }\n",
    );
    write_member_file(
        &dir,
        "src/main.rs",
        "#[path = \"outer/grandchild.rs\"]\nmod live;\n",
    );

    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [
            ("crates/demo-preview/src/lib.rs".to_string(), 3),
            ("crates/demo-preview/src/outer/grandchild.rs".to_string(), 1),
        ]
    );
}

#[test]
fn unknown_cfg_attr_include_and_unreferenced_sources_fail_closed() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg_attr(feature = \"extra\", cfg(test))]\nmod cfg_attr_child;\n\
         include!(\"dynamic_child.rs\");\n\
         #[cfg(test)]\nmod unreachable_child;\n\
         fn root() { None::<u8>.expect(\"root\"); }\n",
    );
    for module in [
        "cfg_attr_child",
        "dynamic_child",
        "unreachable_child",
        "orphan",
    ] {
        write_member_file(
            &dir,
            &format!("src/{module}.rs"),
            "fn body() { None::<u8>.expect(\"body\"); }\n",
        );
    }

    let report = check_scratch(&dir);
    assert_eq!(own14_findings(&report).len(), 5);
}

#[test]
fn custom_target_and_nested_inline_module_paths_are_graph_roots() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)]\nmod nested {\n    mod child;\n}\n\
         #[cfg(test)]\n#[path = \"shared.rs\"]\nmod test_parent;\n",
    );
    write_member_file(
        &dir,
        "src/custom_root.rs",
        "#[path = \"shared.rs\"]\nmod production_parent;\n",
    );
    write_member_file(
        &dir,
        "src/nested/child.rs",
        "fn nested() { None::<u8>.expect(\"nested\"); }\n",
    );
    write_member_file(
        &dir,
        "src/shared.rs",
        "fn shared() { None::<u8>.expect(\"shared\"); }\n",
    );
    write_member_file(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"demo-preview\"\n\
         [[bin]]\nname = \"custom-root\"\npath = \"src/custom_root.rs\"\n",
    );

    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/shared.rs".to_string(), 1)]
    );
}

#[test]
fn ambiguous_module_files_are_not_pruned() {
    let dir = empty_workspace();
    write_member_file(&dir, "src/lib.rs", "#[cfg(test)]\nmod duplicate;\n");
    write_member_file(
        &dir,
        "src/duplicate.rs",
        "fn one() { None::<u8>.expect(\"one\"); }\n",
    );
    write_member_file(
        &dir,
        "src/duplicate/mod.rs",
        "fn two() { None::<u8>.expect(\"two\"); }\n",
    );

    assert_eq!(own14_findings(&check_scratch(&dir)).len(), 2);
}

#[test]
fn literal_include_contexts_propagate_test_and_production_routes() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)]\ninclude!(\"shared.inc.rs\");\n\
         #[cfg(test)]\ninclude!(\"test_only.inc.rs\");\n",
    );
    write_member_file(&dir, "src/main.rs", "include!(\"shared.inc.rs\");\n");
    write_member_file(
        &dir,
        "src/shared.inc.rs",
        "fn shared() { None::<u8>.expect(\"shared\"); }\n",
    );
    write_member_file(
        &dir,
        "src/test_only.inc.rs",
        "fn test_only() { None::<u8>.expect(\"test only\"); }\n",
    );

    let report = check_scratch(&dir);
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(
        &dir.path().join("crates/demo-preview"),
    );
    assert_eq!(
        own14_findings(&report),
        [("crates/demo-preview/src/shared.inc.rs".to_string(), 1)],
        "analysis={analysis:?}, files_scanned={}, failures={:?}, findings={:?}",
        report.files_scanned,
        report.failures,
        report.findings
    );
}

#[test]
fn block_local_module_keeps_dual_parent_source_production() {
    let module = empty_workspace();
    write_member_file(
        &module,
        "src/lib.rs",
        "#[cfg(test)] #[path = \"shared.rs\"] mod test_parent;\n\
         fn production() { #[path = \"shared.rs\"] mod live; }\n",
    );
    write_member_file(
        &module,
        "src/shared.rs",
        "fn body() { None::<u8>.expect(\"block module\"); }\n",
    );
    assert_eq!(
        own14_findings(&check_scratch(&module)),
        [("crates/demo-preview/src/shared.rs".to_string(), 1)]
    );
}

#[test]
fn block_local_include_keeps_dual_parent_source_production() {
    let include = empty_workspace();
    write_member_file(
        &include,
        "src/lib.rs",
        "#[cfg(test)] include!(\"shared.inc.rs\");\n\
         fn production() { let _values = vec![1, 2, 3]; include!(\"shared.inc.rs\"); }\n",
    );
    write_member_file(
        &include,
        "src/shared.inc.rs",
        "fn body() { None::<u8>.expect(\"block include\"); }\n",
    );
    assert_eq!(
        own14_findings(&check_scratch(&include)),
        [("crates/demo-preview/src/shared.inc.rs".to_string(), 1)]
    );
}

#[test]
fn block_local_path_module_discovers_only_its_out_of_src_target() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "fn production() { #[path = \"../outside/only.rs\"] mod local; }\n",
    );
    write_member_file(
        &dir,
        "outside/only.rs",
        "fn body() { None::<u8>.expect(\"local module\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/unreferenced.rs",
        "fn orphan() { None::<u8>.expect(\"orphan\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    let reached = std::fs::canonicalize(member.join("outside/only.rs")).expect("route target");
    let orphan = std::fs::canonicalize(member.join("outside/unreferenced.rs")).expect("orphan");
    assert!(analysis.sources.contains(&reached), "analysis={analysis:?}");
    assert!(!analysis.sources.contains(&orphan), "analysis={analysis:?}");
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/outside/only.rs".to_string(), 1)]
    );
}

#[test]
fn block_local_include_discovers_out_of_src_include_chain_only() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "fn production() {\n\
             if true { { include!(\"../outside/entry.inc.rs\"); } }\n\
             let _expression = include!(\"../outside/expression.inc.rs\");\n\
         }\n",
    );
    write_member_file(
        &dir,
        "outside/entry.inc.rs",
        "include!(\"nested/child.inc.rs\");\n",
    );
    write_member_file(
        &dir,
        "outside/nested/child.inc.rs",
        "fn body() { None::<u8>.expect(\"included child\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/expression.inc.rs",
        "None::<u8>.expect(\"included expression\")\n",
    );
    write_member_file(
        &dir,
        "outside/unreferenced.inc.rs",
        "fn orphan() { None::<u8>.expect(\"orphan\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    for relative in [
        "outside/entry.inc.rs",
        "outside/nested/child.inc.rs",
        "outside/expression.inc.rs",
    ] {
        let reached = std::fs::canonicalize(member.join(relative)).expect("route target");
        assert!(
            analysis.sources.contains(&reached),
            "{relative}: {analysis:?}"
        );
    }
    let orphan = std::fs::canonicalize(member.join("outside/unreferenced.inc.rs")).expect("orphan");
    assert!(!analysis.sources.contains(&orphan), "analysis={analysis:?}");
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [
            (
                "crates/demo-preview/outside/expression.inc.rs".to_string(),
                1
            ),
            (
                "crates/demo-preview/outside/nested/child.inc.rs".to_string(),
                1
            ),
        ]
    );
}

#[test]
fn included_expression_fragments_reach_nested_literal_include_edges() {
    for (kind, expression) in [
        ("direct", "include!(\"nested/leaf.rs\")"),
        ("parenthesized", "(include!(\"nested/leaf.rs\"))"),
        ("closure", "|| include!(\"nested/leaf.rs\")"),
        ("block", "{ include!(\"nested/leaf.rs\") }"),
    ] {
        let dir = empty_workspace();
        write_member_file(
            &dir,
            "src/lib.rs",
            "fn production() { let _value = include!(\"../outside/expr.rs\"); }\n",
        );
        write_member_file(&dir, "outside/expr.rs", expression);
        write_member_file(
            &dir,
            "outside/nested/leaf.rs",
            "None::<u8>.expect(\"nested leaf\");\n",
        );
        write_member_file(
            &dir,
            "outside/nested/unreferenced.rs",
            "None::<u8>.expect(\"unreferenced\");\n",
        );

        let member = dir.path().join("crates/demo-preview");
        let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
        let expression_source =
            std::fs::canonicalize(member.join("outside/expr.rs")).expect("expression source");
        let leaf =
            std::fs::canonicalize(member.join("outside/nested/leaf.rs")).expect("nested leaf");
        let orphan = std::fs::canonicalize(member.join("outside/nested/unreferenced.rs"))
            .expect("unreferenced leaf");
        assert!(analysis.complete, "{kind}: {analysis:?}");
        assert!(analysis.sources.contains(&expression_source), "{kind}");
        assert!(analysis.sources.contains(&leaf), "{kind}: {analysis:?}");
        assert!(!analysis.sources.contains(&orphan), "{kind}: {analysis:?}");
        assert_eq!(
            own14_findings(&check_scratch(&dir)),
            [("crates/demo-preview/outside/nested/leaf.rs".to_string(), 1)],
            "{kind}"
        );
    }
}

#[test]
fn module_item_include_uses_included_filesystem_directory_and_predicate() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)] include!(\"../outside/items.rs\");\n",
    );
    write_member_file(
        &dir,
        "src/child.rs",
        "fn decoy() { None::<u8>.expect(\"caller-directory decoy\"); }\n",
    );
    write_member_file(&dir, "outside/items.rs", "mod child;\n");
    write_member_file(
        &dir,
        "outside/child.rs",
        "#[path = \"nested/path_child.rs\"] mod explicit;\n\
         include!(\"nested/chain.inc.rs\");\n",
    );
    write_member_file(
        &dir,
        "outside/nested/path_child.rs",
        "fn fixture() { None::<u8>.expect(\"path child\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/nested/chain.inc.rs",
        "include!(\"leaf.rs\");\n",
    );
    write_member_file(
        &dir,
        "outside/nested/leaf.rs",
        "fn fixture() { None::<u8>.expect(\"nested include leaf\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    let caller_decoy = std::fs::canonicalize(member.join("src/child.rs")).unwrap();
    let physical_child = std::fs::canonicalize(member.join("outside/child.rs")).unwrap();
    let path_child = std::fs::canonicalize(member.join("outside/nested/path_child.rs")).unwrap();
    let leaf = std::fs::canonicalize(member.join("outside/nested/leaf.rs")).unwrap();

    assert!(analysis.complete, "{analysis:?}");
    assert!(analysis.sources.contains(&physical_child), "{analysis:?}");
    assert!(analysis.sources.contains(&path_child), "{analysis:?}");
    assert!(analysis.sources.contains(&leaf), "{analysis:?}");
    assert!(analysis.test_only.contains(&physical_child), "{analysis:?}");
    assert!(analysis.test_only.contains(&path_child), "{analysis:?}");
    assert!(analysis.test_only.contains(&leaf), "{analysis:?}");
    assert!(!analysis.test_only.contains(&caller_decoy), "{analysis:?}");
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/child.rs".to_string(), 1)]
    );
}

#[test]
fn expression_include_uses_physical_module_directory_and_keeps_caller_predicate() {
    let dir = empty_workspace();
    write_member_file(&dir, "src/lib.rs", "mod caller;\n");
    write_member_file(
        &dir,
        "src/caller.rs",
        "mod inline {\n\
             #[cfg(test)] fn test_fragment() {\n\
                 let _value = include!(\"../outside/expr.rs\");\n\
             }\n\
         }\n",
    );
    write_member_file(
        &dir,
        "src/caller/inline/child.rs",
        "fn decoy() { None::<u8>.expect(\"caller namespace decoy\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/expr.rs",
        "{ mod child; let _leaf = include!(\"nested/leaf.rs\"); None::<u8> }",
    );
    write_member_file(
        &dir,
        "outside/child.rs",
        "fn fixture() { None::<u8>.expect(\"physical expression child\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/nested/leaf.rs",
        "None::<u8>.expect(\"expression nested leaf\")",
    );

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    let caller_decoy = std::fs::canonicalize(member.join("src/caller/inline/child.rs")).unwrap();
    let physical_child = std::fs::canonicalize(member.join("outside/child.rs")).unwrap();
    let leaf = std::fs::canonicalize(member.join("outside/nested/leaf.rs")).unwrap();

    assert!(analysis.complete, "{analysis:?}");
    assert!(analysis.sources.contains(&physical_child), "{analysis:?}");
    assert!(analysis.sources.contains(&leaf), "{analysis:?}");
    assert!(analysis.test_only.contains(&physical_child), "{analysis:?}");
    assert!(analysis.test_only.contains(&leaf), "{analysis:?}");
    assert!(!analysis.test_only.contains(&caller_decoy), "{analysis:?}");
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [(
            "crates/demo-preview/src/caller/inline/child.rs".to_string(),
            1
        )]
    );
}

#[test]
fn nested_expression_includes_keep_predicates_paths_and_shared_reachability() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "fn production() { let _value = include!(\"../outside/shared/expr.rs\"); }\n\
         #[cfg(test)] fn test_shared() { let _value = include!(\"../outside/shared/expr.rs\"); }\n\
         #[cfg(test)] fn test_only() { let _value = include!(\"../outside/test/expr.rs\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/shared/expr.rs",
        "include!(\"nested/shared.rs\")",
    );
    write_member_file(
        &dir,
        "outside/shared/nested/shared.rs",
        "None::<u8>.expect(\"shared leaf\");\n",
    );
    write_member_file(&dir, "outside/test/expr.rs", "include!(\"nested/test.rs\")");
    write_member_file(
        &dir,
        "outside/test/nested/test.rs",
        "None::<u8>.expect(\"test leaf\");\n",
    );
    write_member_file(
        &dir,
        "outside/test/nested/unreferenced.rs",
        "None::<u8>.expect(\"unreferenced test leaf\");\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    let shared_expr = std::fs::canonicalize(member.join("outside/shared/expr.rs")).unwrap();
    let shared_leaf =
        std::fs::canonicalize(member.join("outside/shared/nested/shared.rs")).unwrap();
    let test_expr = std::fs::canonicalize(member.join("outside/test/expr.rs")).unwrap();
    let test_leaf = std::fs::canonicalize(member.join("outside/test/nested/test.rs")).unwrap();
    let orphan = std::fs::canonicalize(member.join("outside/test/nested/unreferenced.rs")).unwrap();
    assert!(analysis.complete, "{analysis:?}");
    for source in [&shared_expr, &shared_leaf, &test_expr, &test_leaf] {
        assert!(
            analysis.sources.contains(source),
            "{source:?}: {analysis:?}"
        );
    }
    assert!(!analysis.sources.contains(&orphan), "{analysis:?}");
    assert!(!analysis.test_only.contains(&shared_expr), "{analysis:?}");
    assert!(!analysis.test_only.contains(&shared_leaf), "{analysis:?}");
    assert!(analysis.test_only.contains(&test_expr), "{analysis:?}");
    assert!(analysis.test_only.contains(&test_leaf), "{analysis:?}");
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [(
            "crates/demo-preview/outside/shared/nested/shared.rs".to_string(),
            1
        )]
    );
}

#[test]
fn nested_expression_fragments_keep_caller_namespace_and_use_included_file_directory() {
    let dir = empty_workspace();
    write_member_file(&dir, "src/lib.rs", "mod caller;\n");
    write_member_file(
        &dir,
        "src/caller.rs",
        "fn production() { let _value = include!(\"../outside/expr.rs\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/expr.rs",
        "{ mod child; let _leaf = include!(\"nested/leaf.rs\"); None::<u8> }",
    );
    write_member_file(
        &dir,
        "src/caller/child.rs",
        "fn decoy() { None::<u8>.expect(\"caller-directory decoy\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/child.rs",
        "fn child() { None::<u8>.expect(\"physical child\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/nested/leaf.rs",
        "None::<u8>.expect(\"physical nested leaf\")",
    );

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    let caller_decoy =
        std::fs::canonicalize(member.join("src/caller/child.rs")).expect("caller decoy");
    let included_child =
        std::fs::canonicalize(member.join("outside/child.rs")).expect("included child");
    let nested_leaf =
        std::fs::canonicalize(member.join("outside/nested/leaf.rs")).expect("nested leaf");
    assert!(analysis.complete, "{analysis:?}");
    assert!(analysis.sources.contains(&included_child), "{analysis:?}");
    assert!(analysis.sources.contains(&nested_leaf), "{analysis:?}");
    assert!(!analysis.test_only.contains(&caller_decoy), "{analysis:?}");
    assert!(
        !analysis.test_only.contains(&included_child),
        "{analysis:?}"
    );
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [
            ("crates/demo-preview/outside/child.rs".to_string(), 1),
            ("crates/demo-preview/outside/nested/leaf.rs".to_string(), 1),
            ("crates/demo-preview/src/caller/child.rs".to_string(), 1),
        ]
    );
}

#[test]
fn nested_expression_unknowns_keep_later_routes_and_cycles_conservative() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "fn production() { let _value = include!(\"../outside/expr.rs\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/expr.rs",
        "(\n\
             unknown_macro!(include!(\"nested/hidden.rs\")),\n\
             include!(concat!(\"nested/\", \"dynamic.rs\")),\n\
             include!(\"nested/later.rs\"),\n\
             include!(\"nested/loop_a.rs\"),\n\
             include!(\"nested/loop_a.rs\")\n\
         )",
    );
    write_member_file(
        &dir,
        "outside/nested/later.rs",
        "None::<u8>.expect(\"later known route\");\n",
    );
    write_member_file(&dir, "outside/nested/loop_a.rs", "include!(\"loop_b.rs\")");
    write_member_file(&dir, "outside/nested/loop_b.rs", "include!(\"loop_a.rs\")");
    for relative in [
        "outside/nested/hidden.rs",
        "outside/nested/dynamic.rs",
        "outside/nested/unreferenced.rs",
    ] {
        write_member_file(
            &dir,
            relative,
            "None::<u8>.expect(\"must stay unresolved\");\n",
        );
    }

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(!analysis.complete, "{analysis:?}");
    for relative in [
        "outside/expr.rs",
        "outside/nested/later.rs",
        "outside/nested/loop_a.rs",
        "outside/nested/loop_b.rs",
    ] {
        let source = std::fs::canonicalize(member.join(relative)).expect("known source");
        assert!(
            analysis.sources.contains(&source),
            "{relative}: {analysis:?}"
        );
        assert_eq!(
            analysis
                .sources
                .iter()
                .filter(|path| *path == &source)
                .count(),
            1
        );
    }
    for relative in [
        "outside/nested/hidden.rs",
        "outside/nested/dynamic.rs",
        "outside/nested/unreferenced.rs",
    ] {
        let source = std::fs::canonicalize(member.join(relative)).expect("unresolved source");
        assert!(
            !analysis.sources.contains(&source),
            "{relative}: {analysis:?}"
        );
    }
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/outside/nested/later.rs".to_string(), 1)]
    );
}

#[test]
fn block_local_path_routes_use_source_and_inline_module_directories() {
    let dir = empty_workspace();
    write_member_file(&dir, "src/lib.rs", "mod outer;\n");
    write_member_file(
        &dir,
        "src/outer/inline/placeholder.rs",
        "fn placeholder() {}\n",
    );
    write_member_file(
        &dir,
        "src/outer.rs",
        "fn production() {\n\
             #[path = \"../outside/source_parent/anchor.rs\"] mod source_relative;\n\
             if true { mod inline {\n\
                 #[path = \"../../../outside/inline_parent/anchor.rs\"] mod inline_relative;\n\
             } }\n\
         }\n",
    );
    write_member_file(
        &dir,
        "outside/source_parent/anchor.rs",
        "fn body() { None::<u8>.expect(\"source relative\"); }\nmod child;\n",
    );
    write_member_file(
        &dir,
        "outside/source_parent/child.rs",
        "fn body() { None::<u8>.expect(\"source child\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/inline_parent/anchor.rs",
        "fn body() { None::<u8>.expect(\"inline relative\"); }\nmod child;\n",
    );
    write_member_file(
        &dir,
        "outside/inline_parent/child.rs",
        "fn body() { None::<u8>.expect(\"inline child\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    for relative in [
        "outside/source_parent/anchor.rs",
        "outside/source_parent/child.rs",
        "outside/inline_parent/anchor.rs",
        "outside/inline_parent/child.rs",
    ] {
        let reached = std::fs::canonicalize(member.join(relative)).expect("route target");
        assert!(
            analysis.sources.contains(&reached),
            "{relative}: {analysis:?}"
        );
    }
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [
            (
                "crates/demo-preview/outside/inline_parent/anchor.rs".to_string(),
                1
            ),
            (
                "crates/demo-preview/outside/inline_parent/child.rs".to_string(),
                1
            ),
            (
                "crates/demo-preview/outside/source_parent/anchor.rs".to_string(),
                1
            ),
            (
                "crates/demo-preview/outside/source_parent/child.rs".to_string(),
                1
            ),
        ]
    );
}

#[test]
fn block_local_include_cycles_and_duplicate_routes_reach_each_source_once() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "fn first() { include!(\"../outside/entry.inc.rs\"); }\n\
         fn duplicate() { if true { include!(\"../outside/entry.inc.rs\"); } }\n",
    );
    write_member_file(
        &dir,
        "outside/entry.inc.rs",
        "fn entry() { None::<u8>.expect(\"entry\"); }\
         include!(\"nested/loop.inc.rs\");\n",
    );
    write_member_file(
        &dir,
        "outside/nested/loop.inc.rs",
        "fn loop_file() { None::<u8>.expect(\"loop\"); }\
         include!(\"../entry.inc.rs\");\n",
    );
    write_member_file(
        &dir,
        "outside/unreferenced.inc.rs",
        "fn orphan() { None::<u8>.expect(\"orphan\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    for relative in ["outside/entry.inc.rs", "outside/nested/loop.inc.rs"] {
        let reached = std::fs::canonicalize(member.join(relative)).expect("route target");
        assert!(
            analysis.sources.contains(&reached),
            "{relative}: {analysis:?}"
        );
        assert_eq!(
            analysis
                .sources
                .iter()
                .filter(|source| *source == &reached)
                .count(),
            1
        );
    }
    let orphan = std::fs::canonicalize(member.join("outside/unreferenced.inc.rs")).expect("orphan");
    assert!(!analysis.sources.contains(&orphan), "analysis={analysis:?}");
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [
            ("crates/demo-preview/outside/entry.inc.rs".to_string(), 1),
            (
                "crates/demo-preview/outside/nested/loop.inc.rs".to_string(),
                1
            ),
        ]
    );
}

#[test]
fn block_local_module_routes_preserve_test_production_and_shared_owners() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)] fn test_only() {\n\
             #[path = \"../outside/test_only.rs\"] mod fixture;\n\
         }\n\
         fn production() { #[path = \"../outside/shared.rs\"] mod shared; }\n\
         #[cfg(test)] fn shared_test() {\n\
             #[path = \"../outside/shared.rs\"] mod shared;\n\
         }\n",
    );
    write_member_file(
        &dir,
        "outside/test_only.rs",
        "fn fixture() { None::<u8>.expect(\"test only\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/shared.rs",
        "fn shared() { None::<u8>.expect(\"shared\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    let test_only = std::fs::canonicalize(member.join("outside/test_only.rs")).unwrap();
    let shared = std::fs::canonicalize(member.join("outside/shared.rs")).unwrap();
    assert!(analysis.complete, "analysis={analysis:?}");
    assert!(
        analysis.sources.contains(&test_only),
        "analysis={analysis:?}"
    );
    assert!(analysis.sources.contains(&shared), "analysis={analysis:?}");
    assert!(
        analysis.test_only.contains(&test_only),
        "analysis={analysis:?}"
    );
    assert!(
        !analysis.test_only.contains(&shared),
        "analysis={analysis:?}"
    );
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/outside/shared.rs".to_string(), 1)]
    );
}

#[test]
fn block_local_unknown_routes_keep_later_known_siblings_discoverable() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "fn routes() {\n\
             unknown_routes!({ #[path = \"../outside/macro_only.rs\"] mod hidden; });\n\
             #[path = concat!(\"../outside/\", \"dynamic.rs\")] mod dynamic;\n\
             #[path(\"../outside/malformed.rs\")] mod malformed;\n\
             include!(concat!(\"../outside/\", \"dynamic.inc.rs\"));\n\
             #[path = \"../outside/later.rs\"] mod later;\n\
             include!(\"../outside/later.inc.rs\");\n\
         }\n",
    );
    for relative in [
        "outside/macro_only.rs",
        "outside/dynamic.rs",
        "outside/dynamic.inc.rs",
        "outside/malformed.rs",
    ] {
        write_member_file(
            &dir,
            relative,
            "fn not_reached() { None::<u8>.expect(\"unknown route\"); }\n",
        );
    }
    write_member_file(
        &dir,
        "outside/later.rs",
        "fn later() { None::<u8>.expect(\"later module\"); }\n",
    );
    write_member_file(
        &dir,
        "outside/later.inc.rs",
        "fn later_include() { None::<u8>.expect(\"later include\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(!analysis.complete, "analysis={analysis:?}");
    for relative in ["outside/later.rs", "outside/later.inc.rs"] {
        let reached = std::fs::canonicalize(member.join(relative)).expect("route target");
        assert!(
            analysis.sources.contains(&reached),
            "{relative}: {analysis:?}"
        );
    }
    for relative in [
        "outside/macro_only.rs",
        "outside/dynamic.rs",
        "outside/dynamic.inc.rs",
        "outside/malformed.rs",
    ] {
        let unresolved = std::fs::canonicalize(member.join(relative)).expect("unresolved target");
        assert!(
            !analysis.sources.contains(&unresolved),
            "{relative}: {analysis:?}"
        );
    }
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [
            ("crates/demo-preview/outside/later.inc.rs".to_string(), 1),
            ("crates/demo-preview/outside/later.rs".to_string(), 1),
        ]
    );
}

#[cfg(unix)]
#[test]
fn block_local_routes_reject_member_and_symlink_escapes_without_losing_known_siblings() {
    use std::os::unix::fs::symlink;

    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "fn routes() {\n\
             #[path = \"../../../outside.rs\"] mod outside;\n\
             #[path = \"../outside/escape.rs\"] mod symlinked;\n\
             include!(\"../../../outside.inc.rs\");\n\
             include!(\"../outside/escape.inc.rs\");\n\
             #[path = \"../outside/known.rs\"] mod known;\n\
         }\n",
    );
    let outside_rs = dir.path().join("outside.rs");
    let outside_inc = dir.path().join("outside.inc.rs");
    std::fs::write(
        &outside_rs,
        "fn outside() { None::<u8>.expect(\"outside member\"); }\n",
    )
    .unwrap();
    std::fs::write(
        &outside_inc,
        "fn outside_include() { None::<u8>.expect(\"outside member include\"); }\n",
    )
    .unwrap();
    write_member_file(
        &dir,
        "outside/known.rs",
        "fn known() { None::<u8>.expect(\"known\"); }\n",
    );
    let outside = dir.path().join("outside.rs");
    let outside_include = dir.path().join("outside.inc.rs");
    let member = dir.path().join("crates/demo-preview");
    std::fs::create_dir_all(member.join("outside")).unwrap();
    symlink(outside, member.join("outside/escape.rs")).unwrap();
    symlink(outside_include, member.join("outside/escape.inc.rs")).unwrap();

    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(!analysis.complete, "analysis={analysis:?}");
    let known = std::fs::canonicalize(member.join("outside/known.rs")).unwrap();
    assert!(analysis.sources.contains(&known), "analysis={analysis:?}");
    assert!(
        !analysis
            .sources
            .contains(&std::fs::canonicalize(dir.path().join("outside.rs")).unwrap())
    );
    assert!(
        !analysis
            .sources
            .contains(&std::fs::canonicalize(dir.path().join("outside.inc.rs")).unwrap())
    );
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/outside/known.rs".to_string(), 1)]
    );
}

#[test]
fn unknown_derive_and_ambiguous_imported_derive_keep_test_routes_production() {
    for derive in ["FromStr", "Debug"] {
        let dir = empty_workspace();
        let import = if derive == "Debug" {
            "use custom::Debug;\n"
        } else {
            ""
        };
        write_member_file(
            &dir,
            "src/lib.rs",
            &format!(
                "#[cfg(test)] mod candidate;\n{import}#[derive({derive})] struct Generated;\n"
            ),
        );
        write_member_file(
            &dir,
            "src/candidate.rs",
            "fn generated_edge() { None::<u8>.expect(\"candidate\"); }\n",
        );
        assert_eq!(
            own14_findings(&check_scratch(&dir)),
            [("crates/demo-preview/src/candidate.rs".to_string(), 1)],
            "derive {derive} must not prove that a custom derive is inert"
        );
    }
}

#[test]
fn explicit_harness_false_test_target_root_remains_production() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/custom_root.rs",
        "#[cfg(test)] fn helper() { None::<u8>.expect(\"test cfg helper\"); }\n\
         fn root_contract() { None::<u8>.expect(\"test root is production\"); }\n",
    );
    write_member_file(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"demo-preview\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\
         [[test]]\nname = \"custom-root\"\npath = \"src/custom_root.rs\"\nharness = false\n",
    );

    let report = check_scratch(&dir);
    assert_eq!(
        own14_findings(&report),
        [("crates/demo-preview/src/custom_root.rs".to_string(), 2)],
        "files_scanned={}, failures={:?}, findings={:?}",
        report.files_scanned,
        report.failures,
        report.findings
    );
}

#[test]
fn explicit_test_root_does_not_change_lib_or_bin_production_reachability() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[path = \"shared.rs\"] mod from_lib;\n\
         fn library() { None::<u8>.expect(\"library root\"); }\n",
    );
    write_member_file(
        &dir,
        "src/main.rs",
        "#[path = \"shared.rs\"] mod from_bin;\n\
         fn binary() { None::<u8>.expect(\"binary root\"); }\n",
    );
    write_member_file(
        &dir,
        "src/custom_test.rs",
        "#[path = \"shared.rs\"] mod from_test;\n\
         fn explicit_test() { None::<u8>.expect(\"explicit test root\"); }\n",
    );
    write_member_file(
        &dir,
        "src/shared.rs",
        "fn shared() { None::<u8>.expect(\"shared from all roots\"); }\n",
    );
    write_member_file(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"demo-preview\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\
         [[test]]\nname = \"explicit\"\npath = \"src/custom_test.rs\"\nharness = false\n",
    );

    let mut findings = own14_findings(&check_scratch(&dir));
    findings.sort();
    assert_eq!(
        findings,
        [
            ("crates/demo-preview/src/custom_test.rs".to_string(), 2),
            ("crates/demo-preview/src/lib.rs".to_string(), 2),
            ("crates/demo-preview/src/main.rs".to_string(), 2),
            ("crates/demo-preview/src/shared.rs".to_string(), 1),
        ]
    );
}

#[test]
fn skipped_block_item_macro_is_not_assumed_to_be_an_expression_macro() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)] mod candidate;\n\
         fn production() { declare_modules!(); }\n",
    );
    write_member_file(
        &dir,
        "src/candidate.rs",
        "fn generated() { None::<u8>.expect(\"candidate\"); }\n",
    );

    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/candidate.rs".to_string(), 1)]
    );
}

#[test]
fn incomplete_root_still_enumerates_already_known_external_routes() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)] #[path = \"../external.rs\"] mod test_parent;\n\
         fn production() { declare_modules!(); }\n",
    );
    write_member_file(
        &dir,
        "external.rs",
        "fn reachable() { None::<u8>.expect(\"external route\"); }\n",
    );
    let member = dir.path().join("crates/demo-preview");
    let external = std::fs::canonicalize(member.join("external.rs")).expect("external source");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);

    assert!(
        !analysis.complete,
        "the block item macro makes the graph incomplete"
    );
    assert!(
        analysis.test_only.is_empty(),
        "incomplete provenance never prunes"
    );
    assert!(
        analysis.sources.contains(&external),
        "the module edge parsed before the unknown route must still be inventoried: {analysis:?}"
    );
}

#[test]
fn incomplete_root_continues_to_later_out_of_src_module_and_include_routes() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[path = \"../before.rs\"] mod before;\n\
         fn uncertain() { declare_modules!(); }\n\
         include!(\"../between.inc.rs\");\n\
         #[path = \"../after.rs\"] mod after;\n",
    );
    for (path, name) in [
        ("before.rs", "before"),
        ("between.inc.rs", "between"),
        ("after.rs", "after"),
    ] {
        write_member_file(
            &dir,
            path,
            &format!("fn {name}() {{ None::<u8>.expect(\"{name}\"); }}\n"),
        );
    }

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(
        !analysis.complete,
        "the function block is an unknown module route"
    );
    assert!(
        analysis.test_only.is_empty(),
        "incomplete provenance never prunes"
    );
    for path in ["before.rs", "between.inc.rs", "after.rs"] {
        let external = std::fs::canonicalize(member.join(path)).expect("external route");
        assert!(
            analysis.sources.contains(&external),
            "later sibling route {path} must remain inventoried: {analysis:?}"
        );
    }
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [
            ("crates/demo-preview/after.rs".to_string(), 1),
            ("crates/demo-preview/before.rs".to_string(), 1),
            ("crates/demo-preview/between.inc.rs".to_string(), 1),
        ]
    );
}

#[test]
fn incomplete_nested_custom_target_continues_to_later_routes() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"demo-preview\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\
         [[bin]]\nname = \"custom\"\npath = \"targets/main.rs\"\n",
    );
    write_member_file(
        &dir,
        "targets/main.rs",
        "mod nested {\n\
             #[path = \"before.rs\"] mod before;\n\
             fn uncertain() { declare_modules!(); }\n\
             #[path = \"external.rs\"] mod external;\n\
             include!(\"nested/include.inc.rs\");\n\
         }\n",
    );
    write_member_file(
        &dir,
        "targets/nested/before.rs",
        "fn nested_before() { None::<u8>.expect(\"nested before\"); }\n",
    );
    write_member_file(
        &dir,
        "targets/nested/external.rs",
        "fn nested_external() { None::<u8>.expect(\"nested external\"); }\n",
    );
    write_member_file(
        &dir,
        "targets/nested/include.inc.rs",
        "fn nested_include() { None::<u8>.expect(\"nested include\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(!analysis.complete, "the nested function block is unknown");
    assert!(
        analysis.test_only.is_empty(),
        "incomplete provenance never prunes"
    );
    for path in [
        "targets/nested/before.rs",
        "targets/nested/external.rs",
        "targets/nested/include.inc.rs",
    ] {
        let external = std::fs::canonicalize(member.join(path)).expect("external route");
        assert!(
            analysis.sources.contains(&external),
            "nested custom-target route {path} must remain inventoried: {analysis:?}"
        );
    }
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [
            (
                "crates/demo-preview/targets/nested/before.rs".to_string(),
                1
            ),
            (
                "crates/demo-preview/targets/nested/external.rs".to_string(),
                1
            ),
            (
                "crates/demo-preview/targets/nested/include.inc.rs".to_string(),
                1,
            ),
        ]
    );
}

#[test]
fn function_local_derive_import_keeps_test_route_production() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)] mod candidate;\n\
         fn production() {\n\
             use custom::Clone;\n\
             #[derive(Clone)]\n\
             struct Local;\n\
         }\n",
    );
    write_member_file(
        &dir,
        "src/candidate.rs",
        "fn generated_edge() { None::<u8>.expect(\"potential production edge\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(
        !analysis.complete,
        "a derive inside the skipped function body can generate module items"
    );
    assert!(
        analysis.test_only.is_empty(),
        "uncertain derives never prune"
    );
    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/candidate.rs".to_string(), 1)]
    );
}

#[test]
fn skipped_unknown_bare_attribute_keeps_candidate_and_later_routes() {
    assert_skipped_body_attribute_is_conservative("#[make_modules]", "struct Local;");
}

#[test]
fn skipped_unknown_qualified_attribute_keeps_candidate_and_later_routes() {
    assert_skipped_body_attribute_is_conservative("#[custom::make_module]", "fn local() {}");
}

#[test]
fn skipped_cfg_attr_unknown_attribute_keeps_candidate_and_later_routes() {
    assert_skipped_body_attribute_is_conservative(
        "#[cfg_attr(not(test), custom::make_module)]",
        "struct Local;",
    );
}

#[test]
fn skipped_malformed_attribute_keeps_candidate_and_later_routes() {
    assert_skipped_body_attribute_is_conservative("#[custom::]", "struct Local;");
}

#[test]
fn skipped_cfg_attr_derive_keeps_candidate_and_later_routes() {
    assert_skipped_body_attribute_is_conservative(
        "#[cfg_attr(not(test), derive(Clone))]",
        "struct Local;",
    );
}

#[test]
fn skipped_supported_attributes_and_attribute_lookalikes_remain_proven() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)] mod candidate;\n\
         fn production() {\n\
             // #[custom::make_module]\n\
             #[allow(dead_code)]\n\
             #[deprecated(note = \"literal #[custom::make_module]\")]\n\
             #[doc = concat!(\"attribute text #[cfg_attr(test, custom)]\")]\n\
             struct Local;\n\
             #[cold]\n\
             fn local_cold() {}\n\
             let _values = vec![1, 2, 3];\n\
         }\n",
    );
    write_member_file(
        &dir,
        "src/candidate.rs",
        "fn test_only() { None::<u8>.expect(\"candidate\"); }\n",
    );

    let member = dir.path().join("crates/demo-preview");
    let candidate =
        std::fs::canonicalize(member.join("src/candidate.rs")).expect("test-only candidate source");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(
        analysis.complete,
        "known inert attributes and vec! are understood"
    );
    assert!(analysis.test_only.contains(&candidate));
    assert!(own14_findings(&check_scratch(&dir)).is_empty());
}

#[test]
fn qualified_recognized_prefixes_and_malformed_test_shapes_fail_closed() {
    for attribute in [
        "#[test::make_module]",
        "#[bench::make_module]",
        "#[test(extra)]",
        "#[bench = \"x\"]",
        "#[test::]",
        "#[::test]",
        "#[cfg::test]",
        "#[cfg_attr::test]",
        "#[path::make_module]",
        "#[derive::Clone]",
        "#[cfg(test) trailing]",
        "#[path = \"candidate.rs\" trailing]",
        "#[derive(Clone) trailing]",
    ] {
        assert_uncertain_module_attribute_keeps_candidate(attribute);
    }
}

#[test]
fn qualified_and_malformed_inert_prefixes_fail_closed_in_module_and_body() {
    for attribute in ["#[allow::make_module]", "#[doc::make_module]"] {
        assert_uncertain_module_attribute_keeps_candidate(attribute);
        assert_skipped_body_attribute_is_conservative(attribute, "struct Local;");
    }
    for (attribute, item) in [
        ("#[allow(dead_code) trailing]", "struct Local;"),
        ("#[doc = \"x\" trailing]", "fn local() {}"),
        ("#[allow = \"x\"]", "struct Local;"),
        ("#[inline(unknown)]", "fn local() {}"),
    ] {
        assert_uncertain_module_attribute_keeps_candidate(attribute);
        assert_skipped_body_attribute_is_conservative(attribute, item);
    }
}

#[test]
fn only_complete_bare_test_and_bench_attributes_grant_test_provenance() {
    let ranges = termrock_xtask::ownership::module_graph::cfg_test_ranges(
        "#[test]\nfn test_case() {}\n#[bench]\nfn bench_case() {}\n",
    );
    assert_eq!(
        ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(1, 4)]
    );
    assert!(
        termrock_xtask::ownership::module_graph::cfg_test_ranges(
            "#[test(extra)]\nfn malformed() {}\n#[bench = \"x\"]\nfn malformed_bench() {}\n"
        )
        .is_empty()
    );
}

#[test]
fn imported_or_glob_test_attributes_do_not_prove_builtin_test_provenance() {
    assert_shadowed_test_attribute_keeps_candidate("use custom::test;", "#[test]");
    assert_shadowed_test_attribute_keeps_candidate("use custom::*;", "#[bench]");
    assert_shadowed_test_attribute_keeps_candidate("use custom::{test};", "#[test]");
    assert_shadowed_test_attribute_keeps_candidate("use custom::{*};", "#[bench]");
    assert_shadowed_test_attribute_keeps_candidate("use custom::make as test;", "#[test]");
    assert_shadowed_test_attribute_keeps_candidate("use custom::make as bench;", "#[bench]");
    assert_shadowed_test_attribute_keeps_candidate("use custom::{test as };", "#[test]");
}

#[test]
fn test_attribute_imports_match_only_proven_local_bindings() {
    for imports in [
        "use custom::test::helper;",
        "use custom::make as test_helper;",
        "use custom::{test as helper};",
        "use custom::{make as test_helper};",
        "use {custom::test::helper, custom::make as test_helper};",
        "use custom::{test::helper, make as test_helper};",
        "use custom::test as _;",
    ] {
        assert_unshadowed_test_import_keeps_test_provenance(imports);
    }
}

#[test]
fn local_imports_cannot_shadow_test_attributes_inside_skipped_bodies() {
    for imports in [
        "use custom::test;",
        "use custom::{test};",
        "use custom::make as test;",
    ] {
        assert_skipped_body_import_keeps_candidate(imports);
    }
}

#[test]
fn enclosing_direct_import_shadows_a_skipped_body_test_attribute() {
    assert_enclosing_import_keeps_skipped_body_candidate(
        "use custom::test;",
        "#[test]\nstruct Local;",
        "",
    );
}

#[test]
fn enclosing_alias_import_shadows_a_skipped_body_test_attribute() {
    assert_enclosing_import_keeps_skipped_body_candidate(
        "use custom::make as test;",
        "#[test]\nstruct Local;",
        "",
    );
}

#[test]
fn enclosing_group_import_shadows_a_skipped_body_test_attribute() {
    assert_enclosing_import_keeps_skipped_body_candidate(
        "use custom::{test};",
        "#[test]\nstruct Local;",
        "",
    );
}

#[test]
fn enclosing_glob_import_shadows_a_skipped_body_bench_attribute() {
    assert_enclosing_import_keeps_skipped_body_candidate(
        "use custom::*;",
        "#[bench]\nstruct Local;",
        "",
    );
}

#[test]
fn import_after_a_skipped_function_still_shadows_its_test_attribute() {
    assert_enclosing_import_keeps_skipped_body_candidate(
        "",
        "#[test]\nstruct Local;",
        "use custom::test;",
    );
}

#[test]
fn enclosing_import_reaches_test_attributes_in_deeper_skipped_blocks() {
    assert_enclosing_import_keeps_skipped_body_candidate(
        "use custom::bench;",
        "if true { { #[bench]\nstruct Local; } }",
        "",
    );
}

#[test]
fn unshadowed_skipped_body_test_and_bench_attributes_keep_builtin_provenance() {
    assert_unshadowed_skipped_body_attribute_proves_candidate(
        "#[test]\nfn generated_test() {}\n#[bench]\nfn generated_bench() {}",
    );
}

#[test]
fn enclosing_test_import_does_not_cross_into_a_nested_module_scope() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "use custom::test;\n\
         mod nested {\n\
             fn production() { #[test] struct Local; }\n\
             #[cfg(test)]\n\
             mod candidate;\n\
         }\n",
    );
    write_member_file(&dir, "src/nested/candidate.rs", "fn body() {}\n");

    let member = dir.path().join("crates/demo-preview");
    let candidate = std::fs::canonicalize(member.join("src/nested/candidate.rs"))
        .expect("nested candidate source");
    let analysis = termrock_xtask::ownership::module_graph::analyze_member(&member);
    assert!(
        analysis.complete,
        "nested module has its own imports: {analysis:?}"
    );
    assert!(analysis.test_only.contains(&candidate));
}

#[test]
fn skipped_block_expression_macro_is_not_treated_as_an_item_macro() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)] mod candidate;\n\
         fn production() { let _values = vec![1, 2, 3]; }\n",
    );
    write_member_file(
        &dir,
        "src/candidate.rs",
        "fn test_only() { None::<u8>.expect(\"candidate\"); }\n",
    );

    assert!(own14_findings(&check_scratch(&dir)).is_empty());
}

#[test]
fn missing_modules_and_item_macros_disable_test_only_pruning() {
    let dir = empty_workspace();
    write_member_file(
        &dir,
        "src/lib.rs",
        "#[cfg(test)]\nmod test_child;\n\
         #[cfg(test)]\nmod missing_child;\n\
         declare_modules!();\n",
    );
    write_member_file(
        &dir,
        "src/test_child.rs",
        "fn test_child() { None::<u8>.expect(\"test child\"); }\n",
    );

    assert_eq!(own14_findings(&check_scratch(&dir)).len(), 1);
}

#[cfg(unix)]
#[test]
fn symlink_module_aliases_disable_pruning() {
    let dir = empty_workspace();
    write_member_file(&dir, "src/lib.rs", "#[cfg(test)]\nmod child;\n");
    write_member_file(
        &dir,
        "src/child.rs",
        "fn child() { None::<u8>.expect(\"child\"); }\n",
    );
    let src = dir.path().join("crates/demo-preview/src");
    std::os::unix::fs::symlink(src.join("child.rs"), src.join("alias.rs"))
        .expect("create source alias");

    assert_eq!(own14_findings(&check_scratch(&dir)).len(), 2);

    let cycle = empty_workspace();
    write_member_file(&cycle, "src/lib.rs", "#[cfg(test)]\nmod child;\n");
    write_member_file(
        &cycle,
        "src/child.rs",
        "fn child() { None::<u8>.expect(\"child\"); }\n",
    );
    let src = cycle.path().join("crates/demo-preview/src");
    std::os::unix::fs::symlink(&src, src.join("cycle")).expect("create directory cycle");

    assert_eq!(own14_findings(&check_scratch(&cycle)).len(), 1);
}

#[test]
fn negative_probe_forbidden_pattern_fails() {
    let dir = scratch_workspace(
        "use ratatui::buffer::Buffer as Buf;\n\
         fn paint(buf: &mut Buf) {\n\
         \x20   buf.set_string(0, 0, \"x\", s);\n\
         }\n\
         fn load(o: Option<u32>) -> u32 {\n\
         \x20   o.expect(\"present\")\n\
         }\n",
        &empty_exceptions(),
    );
    let report = check_workspace(dir.path(), &dir.path().join("exceptions.json"));
    assert!(!report.passed(), "a forbidden pattern must fail the check");
    let kinds: Vec<String> = report.failures.iter().map(|f| f.message.clone()).collect();
    assert!(
        kinds.iter().any(|m| m.contains("OWN-01")),
        "raw buffer import must fail: {kinds:?}"
    );
    assert!(
        kinds.iter().any(|m| m.contains("OWN-02")),
        "buffer paint call must fail: {kinds:?}"
    );
    assert!(
        kinds.iter().any(|m| m.contains("OWN-14")),
        "production expect must fail: {kinds:?}"
    );

    let import = report
        .findings
        .iter()
        .find(|f| f.rule == "OWN-01")
        .expect("an OWN-01 finding");
    assert!(
        import.detail.contains("ratatui::buffer::Buffer"),
        "alias must resolve to the canonical path: {}",
        import.detail
    );
}

#[test]
fn negative_probe_growth_beyond_max_fails() {
    let capped = format!(
        r#"{{"schema":"{}","exceptions":[{{"path":"crates/demo-preview/src/lib.rs","rule":"OWN-02","max":1,"reason":"probe","review":"probe"}}]}}"#,
        exceptions::SCHEMA_VERSION
    );
    let dir = scratch_workspace(
        "use termrock::Ui;\n\
         fn a(ui: &mut Ui) {\n\
         \x20   ui.fill(area, s);\n\
         \x20   ui.fill(area, s);\n\
         }\n",
        &capped,
    );
    let report = check_workspace(dir.path(), &dir.path().join("exceptions.json"));
    assert!(!report.passed(), "growth beyond max must fail");
    assert!(
        report
            .failures
            .iter()
            .any(|f| f.kind == exceptions::FailureKind::Growth),
        "expected a growth failure: {:?}",
        report.failures
    );
}

#[test]
fn negative_probe_stale_exception_fails() {
    let stale = format!(
        r#"{{"schema":"{}","exceptions":[{{"path":"crates/demo-preview/src/lib.rs","rule":"OWN-02","max":3,"reason":"probe","review":"probe"}}]}}"#,
        exceptions::SCHEMA_VERSION
    );
    let dir = scratch_workspace("fn clean() {}\n", &stale);
    let report = check_workspace(dir.path(), &dir.path().join("exceptions.json"));
    assert!(!report.passed(), "a stale exception must fail");
    assert!(
        report
            .failures
            .iter()
            .any(|f| f.kind == exceptions::FailureKind::Stale),
        "expected a stale failure: {:?}",
        report.failures
    );
}

#[test]
fn negative_probe_unknown_schema_fails() {
    let dir = scratch_workspace("fn clean() {}\n", r#"{"schema":"nope/v9","exceptions":[]}"#);
    let report = check_workspace(dir.path(), &dir.path().join("exceptions.json"));
    assert!(!report.passed(), "an unknown schema must fail");
}

#[test]
fn rule_catalog_is_stable() {
    let ids: Vec<&str> = rules::RULES.iter().map(|r| r.id).collect();
    assert_eq!(
        ids,
        [
            "OWN-01", "OWN-02", "OWN-03", "OWN-04", "OWN-05", "OWN-06", "OWN-07", "OWN-08",
            "OWN-09", "OWN-10", "OWN-11", "OWN-12", "OWN-13", "OWN-14",
        ]
    );
}

#[test]
fn shipped_exceptions_parse() {
    let root = workspace_root();
    let path: PathBuf = root
        .join("crates")
        .join("termrock-xtask")
        .join("exceptions")
        .join("v1.json");
    let text = std::fs::read_to_string(&path).expect("read shipped exceptions");
    let parsed = exceptions::load_exceptions(&text).expect("shipped exceptions parse");
    assert!(
        !parsed.exceptions.is_empty(),
        "the G1 corpus is non-empty by construction"
    );
    for exception in &parsed.exceptions {
        assert!(
            Path::new(&root).join(&exception.path).exists(),
            "exception path must exist: {}",
            exception.path
        );
        assert!(!exception.reason.is_empty(), "reason required");
        assert!(!exception.review.is_empty(), "review required");
    }
}
