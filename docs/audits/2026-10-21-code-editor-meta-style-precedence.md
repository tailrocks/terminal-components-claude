# CodeEditor META style precedence

## Finding

Independent review `code-editor-meta-precedence-09aa-20261021` found that `CodeEditor::draw` resolved `Part::META` and then replaced its foreground with a line-number color. The focused cursor line also added bold after resolution. Those late writes erased consumer-owned `META` foregrounds and explicit modifier clears in focused and read-only editors. Follow-up review `code-editor-meta-9cc4059-independent-review-20261021` found the same defect in the position footer: its post-resolution faint patch erased consumer foregrounds.

## Change

`CodeEditor` now supplies the existing line-number and position-footer presentation as `StyleDefaults` to the canonical `Ui::style_defaults_with_explicit` resolver. Defaults remain faint for read-only lines and the position footer, primary plus bold for the focused cursor line, secondary for other lines in the current block, and muted otherwise. Theme rules, overlays, and per-widget `PartStyle` patches resolve above these defaults, including explicit same-value foregrounds and modifier removals. Native profile tests use `Theme::downgrade`, keeping the Junie palette consistent with each capability level. No public API or application-specific code changed.

## Evidence

- Review packet: `code-editor-meta-precedence-09aa-20261021/review.json`, SHA-256 `3950fedec68aa02fa9a7a965b7faa19e7edbe37341f441d21040a91093ebfbba`.
- Follow-up review packet: `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/code-editor-meta-9cc4059-independent-review-20261021/review.json`, SHA-256 `c501c654e129c347c50836055b25ef29882b21b8f60425dcf867609471467d95`. It recorded the inherited footer override and the direct-capability test fixture gap.
- The unchanged external probe source SHA-256 is `6ada49133bcfb5549d867f2dfac1be8ed80427c0e8e730385f8c443c52ef1454`. Review recorded it red against `09aa3a18aede41df9cd478859bed5bec519ef445`; the same source passed against this worktree through public `termrock` and `termrock-test-support` dependencies.
- `CARGO_TARGET_DIR=/private/tmp/code-editor-meta-precedence-final-target-20261021 rustup run 1.99.1 cargo nextest run -p termrock-editors --lib`: 37 passed, 0 skipped.
- `CARGO_TARGET_DIR=/private/tmp/code-editor-meta-precedence-final-target-20261021 rustup run 1.99.1 cargo nextest run --manifest-path /Users/donbeave/Projects/code-editor-meta-probe-current/Cargo.toml`: unchanged external probe source SHA-256 `6ada49133bcfb5549d867f2dfac1be8ed80427c0e8e730385f8c443c52ef1454` passed, 1 passed, 0 skipped.
- `CARGO_TARGET_DIR=/private/tmp/code-editor-meta-precedence-final-target-20261021 rustup run 1.99.1 cargo clippy -p termrock-editors --all-targets -- -D warnings`: passed.
- `rustup run 1.99.1 cargo fmt --all -- --check`: passed.

Native regressions cover genuine runtime focus, editable and read-only consumers, all four downgraded color profiles, authored theme foreground and italic/underline rules on both line numbers and the rendered footer, explicit bold removal, same-value per-widget foreground ownership on both surfaces, the default faint footer and line-number presentation, current-block styling, and selection suppressing current-block styling. The independent public-API footer probe changed from red to green.

This bounded change does not establish frozen CLI parity or complete PR #38 acceptance.
