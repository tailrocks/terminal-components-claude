# Handoff

## PropsList sheet contract correction

- Branch: `wip/props-dialog-contract-stage1-20261021`
- Exact local tip: `386b895597b220d949304e506bd5c629792b8b25`
- Worktree: `/private/tmp/termrock-props-dialog-stage1-20261021`
- State: implementation and audit are committed; worktree clean. The author attempted the shared push lock once and found it owned by `termrock-fix-findings workstream 2026-10-21`. No fetch, rebase, push, or lock modification was performed. Parent assumed publication and was sent the exact commit SHA.
- Focused regression: `cargo nextest run -p termrock-controls --test props_sheet_regressions --target-dir /tmp/termrock-props-stage-target` — 5 passed, run `b83f3be9-3ad4-40e5-9474-cbf8be8ccf17`.
- Full affected crate: `cargo nextest run -p termrock-controls --target-dir /tmp/termrock-props-stage-target` — 151 passed, 0 skipped, run `24ccb375-63a4-4635-810c-79163d92b911`.
- Scoped strict Clippy, `rustfmt --edition 2024 --check` on touched Rust files, and `git diff --check` passed before commit.
- Regression source: `crates/termrock-controls/tests/props_sheet_regressions.rs`. The five probes cover bounds-clipped copy hints, wrapped-value bottom clipping, per-row copy-hint space, measured label width, and explicit part-style Set/Clear precedence. Red evidence from the pre-correction implementation is recorded in `docs/audits/2026-10-21-props-sheet-shared-contract.md`.
- Independent probe: `/private/tmp/termrock-props-sheet-independent-e4b919-20261021/crates/termrock-controls/tests/independent_props_sheet.rs`; immutable packet `/Users/donbeave/Projects/termrock-cli-evidence-20261021/props-sheet-independent-review-e4b919-20261021/review.json`, SHA-256 `2d156d1e8d6dba332d2844869481bc075ad3cec580319fb71e1a96d93c1685e7`. These files belong to the reviewer and must remain read-only.
- Next: parent publishes/merges this exact commit when the shared lock is available, then asks the independent reviewer to rerun the immutable probe against the merged exact commit. Do not claim independent re-review, application parity, or the remaining large-list alternate-theme, Dialog chrome/scope, Jackin app migration, or full Tuiscotti gates as complete.
