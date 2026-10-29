use termrock_xtask::doc_inventory::{inspect_fixture, scan_workspace};

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("xtask crate is nested under crates/")
        .to_path_buf()
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
