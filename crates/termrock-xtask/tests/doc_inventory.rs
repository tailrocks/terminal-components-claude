use std::{fs, process::Command};

use termrock_xtask::doc_inventory::{inspect_fixture, scan_workspace};

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("xtask crate is nested under crates/")
        .to_path_buf()
}

fn package_readme_fixture(readme: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary workspace");
    let package = root.path().join("tools/refactor-proof");
    fs::create_dir_all(package.join("src")).expect("package source directory");
    fs::write(
        root.path().join("Cargo.toml"),
        "[workspace]\nresolver = \"3\"\nmembers = [\"tools/refactor-proof\"]\n",
    )
    .expect("workspace manifest");
    fs::write(
        root.path().join("Cargo.lock"),
        "version = 4\n\n[[package]]\nname = \"readme-fixture\"\nversion = \"0.1.0\"\n",
    )
    .expect("workspace lockfile");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"readme-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("package manifest");
    fs::write(package.join("src/lib.rs"), "pub fn fixture() {}\n").expect("package target");
    let reviewed_ignore = root.path().join("crates/termrock-runtime/src/ui/paint.rs");
    fs::create_dir_all(reviewed_ignore.parent().expect("ignore parent"))
        .expect("reviewed-ignore directory");
    fs::write(
        reviewed_ignore,
        "/// ```ignore\n/// let view = ScrollRegion::view(st.core.scroll(), content, visible_len);\n/// ui.fade_picker_overflow(content, &view);\n/// ```\n",
    )
    .expect("reviewed illustrative ignore");
    fs::write(root.path().join("README.md"), "Workspace-root decoy.\n")
        .expect("workspace README decoy");
    fs::write(package.join("README.md"), readme).expect("package README");

    let init = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(root.path())
        .output()
        .expect("initialize temporary tracked workspace");
    assert!(init.status.success(), "git init failed: {init:?}");
    let add = Command::new("git")
        .args(["add", "--all"])
        .current_dir(root.path())
        .output()
        .expect("track temporary workspace inputs");
    assert!(add.status.success(), "git add failed: {add:?}");
    root
}

fn cargo_targets_fixture(targets: &[(&str, &str, &str)]) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary workspace");
    let package = root.path().join("crates/visual-harness");
    fs::create_dir_all(&package).expect("fixture package directory");
    fs::write(
        root.path().join("Cargo.toml"),
        "[workspace]\nresolver = \"3\"\nmembers = [\"crates/visual-harness\"]\n",
    )
    .expect("workspace manifest");
    fs::write(
        root.path().join("Cargo.lock"),
        "version = 4\n\n[[package]]\nname = \"doc-target-fixture\"\nversion = \"0.1.0\"\n",
    )
    .expect("workspace lockfile");

    let mut manifest = String::from(
        "[package]\nname = \"doc-target-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\nautobins = false\nreadme = false\n",
    );
    for &(name, path, source) in targets {
        manifest.push_str(&format!(
            "\n[[bin]]\nname = \"{name}\"\npath = \"{path}\"\n"
        ));
        let target_path = package.join(path);
        fs::create_dir_all(target_path.parent().expect("target parent")).expect("target directory");
        fs::write(target_path, source).expect("target source");
    }
    fs::write(package.join("Cargo.toml"), manifest).expect("package manifest");

    let reviewed_ignore = root.path().join("crates/termrock-runtime/src/ui/paint.rs");
    fs::create_dir_all(reviewed_ignore.parent().expect("ignore parent"))
        .expect("reviewed-ignore directory");
    fs::write(
        reviewed_ignore,
        "/// ```ignore\n/// let view = ScrollRegion::view(st.core.scroll(), content, visible_len);\n/// ui.fade_picker_overflow(content, &view);\n/// ```\n",
    )
    .expect("reviewed illustrative ignore");

    let init = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(root.path())
        .output()
        .expect("initialize Cargo target fixture");
    assert!(init.status.success(), "git init failed: {init:?}");
    let add = Command::new("git")
        .args(["add", "--all"])
        .current_dir(root.path())
        .output()
        .expect("track Cargo target fixture");
    assert!(add.status.success(), "git add failed: {add:?}");
    root
}

fn cargo_target_admission_issues(
    report: &termrock_xtask::doc_inventory::Report,
) -> Vec<&termrock_xtask::doc_inventory::Issue> {
    report
        .issues
        .iter()
        .filter(|issue| {
            issue.detail.contains("Cargo target source")
                || issue.detail.contains("Rust source root is invalid")
                || issue
                    .detail
                    .contains("Rust source or include input is not tracked")
        })
        .collect()
}

fn package_readme_issue(report: &termrock_xtask::doc_inventory::Report) -> bool {
    report.issues.iter().any(|issue| {
        issue.path == "tools/refactor-proof/Cargo.toml" && issue.detail.contains("package README")
    })
}

fn package_readme_example_issue(report: &termrock_xtask::doc_inventory::Report) -> bool {
    report.issues.iter().any(|issue| {
        issue.path == "tools/refactor-proof/README.md"
            && issue
                .detail
                .contains("unregistered Rust documentation example")
    })
}

fn assert_package_fixture_inventory(report: &termrock_xtask::doc_inventory::Report) {
    assert_eq!(
        report.package_count,
        1,
        "unexpected package inventory: {}",
        report.describe()
    );
    assert_eq!(
        report.target_count,
        1,
        "unexpected target inventory: {}",
        report.describe()
    );
    assert_eq!(
        report.package_readmes,
        1,
        "unexpected README inventory: {}",
        report.describe()
    );
    assert!(
        !report.issues.iter().any(|issue| {
            issue.path == "Cargo.toml" || issue.path == "crates/termrock-runtime/src/ui/paint.rs"
        }),
        "fixture has an unrelated workspace/ignore issue: {}",
        report.describe()
    );
}

#[test]
fn tracked_workspace_documentation_has_no_unregistered_examples() {
    let report = scan_workspace(&workspace_root());
    assert!(
        report.passed(),
        "Rust documentation inventory failed:\n{}",
        report.describe()
    );
}

#[test]
fn package_relative_readme_is_resolved_from_manifest_directory_and_scanned() {
    let root = package_readme_fixture("```rust\nfn main() {}\n```\n");
    let workspace = root
        .path()
        .canonicalize()
        .expect("canonical workspace root");
    let report = scan_workspace(&workspace);

    assert_package_fixture_inventory(&report);
    assert!(
        !package_readme_issue(&report),
        "package-relative README was not resolved from its manifest directory:\n{}",
        report.describe()
    );
    assert!(
        package_readme_example_issue(&report),
        "the inventoried package README bypassed the existing content policy:\n{}",
        report.describe()
    );

    fs::write(
        workspace.join("tools/refactor-proof/README.md"),
        "```bash\nfn main() {}\n```\n",
    )
    .expect("replace README with a shell-only example");
    let report = scan_workspace(&workspace);
    assert_package_fixture_inventory(&report);
    assert!(
        !package_readme_issue(&report),
        "package-relative README failed after a content-only change:\n{}",
        report.describe()
    );
    assert!(
        !package_readme_example_issue(&report),
        "a shell fence was treated as a Rust example:\n{}",
        report.describe()
    );
}

#[test]
fn termrock_shell_examples_remain_covered_after_empty_doctests_are_disabled() {
    let shell_source = include_str!("../../termrock-shell/src/lib.rs");
    let with_example = format!(
        "{shell_source}\n/// # Examples\n/// ```rust\n/// fn shell_example() {{}}\n/// ```\n"
    );
    let issues = inspect_fixture("crates/termrock-shell/src/lib.rs", &with_example);
    assert!(
        !issues.is_empty(),
        "the documentation guard accepted a Rust example in the target with doctests disabled"
    );
}

#[test]
fn rust_and_edition_fences_fail_closed() {
    for info in [
        "rust",
        "rust,edition2024",
        "rust,no_run",
        "compile_fail",
        "should_panic",
    ] {
        let source = format!("```{info}\nfn main() {{}}\n```");
        assert!(
            !inspect_fixture("README.md", &source).is_empty(),
            "accepted {info}"
        );
    }
}

#[test]
fn unlabeled_and_indented_rust_fail_closed() {
    for source in [
        "```\nfn main() {}\n```",
        "```\nui.button(&button);\n```",
        "    fn main() {}\n    println!(\"example\");",
    ] {
        assert!(
            !inspect_fixture("README.md", source).is_empty(),
            "accepted {source:?}"
        );
    }
}

#[test]
fn nested_list_paragraph_continuations_are_not_indented_code() {
    let source = concat!(
        "2. Failing-first proof (rendered cells through the real `Harness`):\n",
        "   - `button_cells_carry_container_ink_and_bold_like_the_frozen_frame` —\n",
        "     red at tip (trailing BOLD dropped), green after.\n",
        "\n",
        "11. `1c17d8f262` (current tip) — re-authors the 256 danger family\n",
        "    indexes and pins the matching expected overrides.\n",
        "\n",
        "* New coverage added by the range:\n",
        "  - `crates/termrock-conformance/tests/conformance.rs`: a new\n",
        "    `activation_probe_guards` module with 5 tests, including\n",
        "    `Conformance::fixture()` and the shared driver migration.\n",
    );

    assert!(
        inspect_fixture("README.md", source).is_empty(),
        "list continuation prose is not an indented code block"
    );
}

#[test]
fn nested_list_indented_rust_remains_fail_closed() {
    for source in [
        "- example:\n\n      fn main() {}\n      println!(\"list code\");\n",
        "- outer item\n  - nested item\n\n        fn main() {}\n        println!(\"nested list code\");\n",
    ] {
        assert!(
            !inspect_fixture("README.md", source).is_empty(),
            "accepted nested-list Rust {source:?}"
        );
    }
}

#[test]
fn all_atx_heading_levels_close_paragraphs_before_indented_rust() {
    for level in 1..=6 {
        let heading = format!("{} Title", "#".repeat(level));
        let source = format!("{heading}\n    fn main() {{}}\n");
        assert!(
            !inspect_fixture("README.md", &source).is_empty(),
            "accepted Rust after heading level {level}: {source:?}"
        );
    }

    for heading in ["##\tTabbed", "###", "#not a heading", "####### Too many"] {
        let source = format!("{heading}\n    fn main() {{}}\n");
        let issues = inspect_fixture("README.md", &source);
        if heading.starts_with('#') && (heading == "###" || heading.starts_with("##\t")) {
            assert!(
                !issues.is_empty(),
                "accepted Rust after valid heading {heading:?}"
            );
        } else {
            assert!(
                issues.is_empty(),
                "misread paragraph text as a heading: {heading:?}"
            );
        }
    }
}

#[test]
fn first_list_child_code_and_list_headings_remain_fail_closed() {
    for source in [
        "-     fn main() {}\n",
        "1.     fn main() {}\n",
        "-\t    fn main() {}\n",
        "1.\t   fn main() {}\n",
        "- outer\n  -     fn main() {}\n",
        "- ## Heading\n      fn main() {}\n",
        "- outer\n  - ### Nested heading\n        fn main() {}\n",
    ] {
        assert!(
            !inspect_fixture("README.md", source).is_empty(),
            "accepted list-contained Rust: {source:?}"
        );
    }

    for source in ["-    fn main() {}\n", "1.    fn main() {}\n"] {
        assert!(
            inspect_fixture("README.md", source).is_empty(),
            "treated four-space list padding as an indented code block: {source:?}"
        );
    }
}

#[test]
fn commonmark_container_transitions_keep_indented_rust_fail_closed() {
    for (source, line) in [
        ("- item\n# Heading\n    fn main() {}\n", 3),
        ("- item\n## Heading\n    fn main() {}\n", 3),
        (
            "- outer\n  - nested\n    ## Heading\n        fn main() {}\n",
            4,
        ),
        ("- item\n  ## Heading\n      fn main() {}\n", 3),
    ] {
        let issues = inspect_fixture("README.md", source);
        assert!(
            issues.iter().any(|issue| issue.line == line),
            "accepted indented Rust at line {line}: {source:?}; got {issues:?}"
        );
    }
}

#[test]
fn markdown_code_blocks_preserve_contexts_and_source_lines() {
    let cases = [
        ("preamble\n> ```rust\n> fn quoted() {}\n> ```\n", 2),
        ("- item\n  ```rust\n  fn listed() {}\n  ```\n", 2),
        ("Heading\n---\n    fn after_setext() {}\n", 3),
        ("Préface\r\n```rust\r\nfn crlf() {}\r\n```\r\n", 2),
        ("<!--\n```rust\nfn in_comment() {}\n```\n-->\n", 2),
        (
            "<details>\n```rust\nfn in_html_block() {}\n```\n</details>\n",
            2,
        ),
        ("before\n```rust\nfn unclosed() {}\n", 2),
    ];

    for (source, line) in cases {
        let issues = inspect_fixture("README.md", source);
        assert!(
            issues.iter().filter(|issue| issue.line == line).count() == 1,
            "expected one code-block finding on line {line}: {source:?}; got {issues:?}"
        );
    }
}

#[test]
fn rustdoc_markdown_code_block_reports_original_source_line() {
    let source = "/// Overview.\n/// ```rust\n/// fn documented() {}\n/// ```\n";
    let issues = inspect_fixture("src/lib.rs", source);
    assert!(
        issues.iter().any(|issue| issue.line == 2),
        "reported the wrong source line for a rustdoc fence: {issues:?}"
    );
}

#[test]
fn rustdoc_block_and_attribute_inputs_fail_closed() {
    let block = "/**\n * ```rust\n * fn main() {}\n * ```\n */";
    assert!(!inspect_fixture("src/lib.rs", block).is_empty());

    for attribute in [
        "#[doc = \"```rust\\nfn main() {}\\n```\"]",
        "#[doc = include_str!(\"examples.md\")]",
        "#[cfg_attr(feature = \"docs\", doc = include_str!(\"examples.md\"))]",
        "#[cfg_attr(\n    feature = \"docs\",\n    doc = include_str!(\"examples.md\")\n)]",
        "macro_rules! generated_docs { () => { #[doc = \"new docs\"] }; }",
    ] {
        assert!(
            !inspect_fixture("src/lib.rs", attribute).is_empty(),
            "accepted {attribute}"
        );
    }
}

#[test]
fn non_rust_markdown_and_ordinary_rust_strings_do_not_trigger() {
    let markdown = "```bash\nfn main() {}\n```\n\nPlain prose mentioning `let value = 3`.";
    assert!(inspect_fixture("README.md", markdown).is_empty());

    let fenced_body = "~~~~text\n```rust\nfn not_a_nested_example() {}\n```\n~~~~\n";
    assert!(inspect_fixture("README.md", fenced_body).is_empty());

    let rust = r###"fn example_text() {
    let text = r#"
/// ```rust
/// fn main() {}
/// ```
"#;
    let _ = text;
}"###;
    assert!(inspect_fixture("src/lib.rs", rust).is_empty());

    let cooked = r###"fn example_text() {
    let text = "continued\
    /// ```rust
    /// fn main() {}
    /// ```";
    let _ = text;
}"###;
    assert!(inspect_fixture("src/lib.rs", cooked).is_empty());
}

#[test]
fn cfg_attr_doc_assignment_ignores_trivia_bypass() {
    for source in [
        "#[cfg_attr(feature = \"docs\", doc\n = include_str!(\"examples.md\"))]",
        "#[cfg_attr(feature = \"docs\", doc /* gap */ = include_str!(\"examples.md\"))]",
    ] {
        assert!(
            !inspect_fixture("src/lib.rs", source).is_empty(),
            "accepted {source:?}"
        );
    }
}

#[test]
fn inner_and_outer_block_doc_examples_survive_nested_comments() {
    for doc in ["/*!", "/**"] {
        let source = format!(
            "{doc}\n * Examples:\n * /* a nested comment */\n * ```rust\n * fn hidden_example() {{}}\n * ```\n */"
        );
        assert!(
            !inspect_fixture("src/lib.rs", &source).is_empty(),
            "accepted block doc {source:?}"
        );
    }

    let ordinary = "/*\n * ```rust\n * fn ordinary_comment() {}\n * ```\n */";
    assert!(inspect_fixture("src/lib.rs", ordinary).is_empty());
}

#[test]
fn shared_sibling_cargo_targets_are_normalized_and_still_scanned() {
    let rustdoc =
        "/// # Examples\n/// ```rust\n/// fn tracked_target_example() {}\n/// ```\nfn main() {}\n";
    let root = cargo_targets_fixture(&[
        ("showcase", "../showcase/src/main.rs", rustdoc),
        ("tablepro", "../tablepro/src/main.rs", "fn main() {}\n"),
        (
            "jackin-preview",
            "../jackin-preview/src/main.rs",
            "fn main() {}\n",
        ),
        ("holla", "../holla/src/main.rs", "fn main() {}\n"),
    ]);
    let workspace = root
        .path()
        .canonicalize()
        .expect("canonical Cargo workspace root");
    let report = scan_workspace(&workspace);

    assert_eq!(report.package_count, 1, "{}", report.describe());
    assert_eq!(report.target_count, 4, "{}", report.describe());
    assert_eq!(report.package_readmes, 0, "{}", report.describe());
    assert_eq!(report.ignored_examples, 1, "{}", report.describe());
    assert!(
        cargo_target_admission_issues(&report).is_empty(),
        "valid sibling target roots were not normalized: {}",
        report.describe()
    );
    assert!(
        report.issues.iter().any(|issue| {
            issue.path == "crates/showcase/src/main.rs"
                && issue
                    .detail
                    .contains("unregistered Rust documentation example")
        }),
        "normalized target content was skipped instead of scanned: {}",
        report.describe()
    );

    fs::write(
        workspace.join("crates/showcase/src/main.rs"),
        "/// ```bash\n/// fn shell_example() {}\n/// ```\nfn main() {}\n",
    )
    .expect("replace Rustdoc example with shell-only documentation");
    let shell_only = scan_workspace(&workspace);
    assert_eq!(shell_only.target_count, 4, "{}", shell_only.describe());
    assert_eq!(shell_only.ignored_examples, 1, "{}", shell_only.describe());
    assert!(
        cargo_target_admission_issues(&shell_only).is_empty(),
        "source-only replacement changed target admission: {}",
        shell_only.describe()
    );
    assert!(
        shell_only.issues.is_empty(),
        "shell-only fixture should remove only the content-policy issue: {}",
        shell_only.describe()
    );
}

#[test]
fn duplicate_cargo_targets_keep_counts_and_non_rs_sources_are_scanned() {
    let root = cargo_targets_fixture(&[
        ("first", "../showcase/src/main.rs", "fn main() {}\n"),
        ("second", "../showcase/src/main.rs", "fn main() {}\n"),
        (
            "module-source",
            "../tablepro/src/entry.module",
            "/// ```rust\n/// fn non_rs_target_example() {}\n/// ```\n",
        ),
    ]);
    let workspace = root
        .path()
        .canonicalize()
        .expect("canonical Cargo workspace root");
    let report = scan_workspace(&workspace);

    assert_eq!(report.package_count, 1, "{}", report.describe());
    assert_eq!(
        report.target_count,
        3,
        "duplicate targets were deduplicated: {}",
        report.describe()
    );
    assert_eq!(report.package_readmes, 0, "{}", report.describe());
    assert_eq!(report.ignored_examples, 1, "{}", report.describe());
    assert!(
        cargo_target_admission_issues(&report).is_empty(),
        "valid duplicate/non-Rust target source was rejected: {}",
        report.describe()
    );
    assert!(
        report.issues.iter().any(|issue| {
            issue.path == "crates/tablepro/src/entry.module"
                && issue
                    .detail
                    .contains("unregistered Rust documentation example")
        }),
        "non-.rs Cargo target was not scanned: {}",
        report.describe()
    );
}
