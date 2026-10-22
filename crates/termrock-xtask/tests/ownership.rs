use std::path::{Path, PathBuf};

use termrock_xtask::ownership::{check_workspace, exceptions, rules};

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
    check_workspace(dir.path(), &dir.path().join("exceptions.json"))
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

    assert_eq!(
        own14_findings(&check_scratch(&dir)),
        [("crates/demo-preview/src/shared.inc.rs".to_string(), 1)]
    );
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
