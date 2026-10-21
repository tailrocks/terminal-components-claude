# Migration handoff

Start with the current [Termrock migration and acceptance matrix](../architecture/termrock-migration-matrix.md). It records fixed source identities, application status, paired CLI coverage, architecture findings, unresolved differences, and next evidence. Treat every partial or missing result as incomplete work.

## Palette and shared-grid handoff (2026-10-21)

Preservation branch: `wip/palette-grid-repair-20261021`, tip `148c32c1bd7b646b032b3eaa739d1ee1338ecf2e` before this handoff update. The work is based on PR #38 commit `5c0075a09eec51b805f15d79ad945d4fb51fb5bf` and consists of:

- `7d5246b1b487dccfca76648bf13be13e2577026d`: remove exact-RGB global conversion exceptions; keep the frozen Junie ANSI-256 semantic values for `on_accent=232`, `border_strong=242`, and `disabled_fg=239`.
- `3a8b18362954da068e1c3418817fcb9d25c0ab37`: apply existing `Ui::scroll_edges` behavior in shared Grid and remove TablePro's private bottom-edge repaint.
- `148c32c1bd7b646b032b3eaa739d1ee1338ecf2e`: state the unresolved app-level repaint limitation accurately in the audit.

Focused checks: `cargo nextest run --locked -p termrock-grid` passed 125/125. The filtered theme check passed 2 tests, with 121 filtered. `cargo nextest run --locked -p tablepro-ui` passed 167/180 and failed 13; failures were not reproduced against the pre-change source, so their attribution remains unresolved. This is not application parity evidence.

Important limitation: TablePro's `draw_painted_grid` / `paint_scrolled_cells` runs after the shared Grid renderer and can overwrite the new shared fade. Thus direct Grid behavior is verified, but the application-visible final cells are not proven fixed. Remove that repaint only after mapping its row-number, cell-style, hit/focus and scrolling contracts to shared components, then compare the real CLI against the frozen baseline. An attempted merge into the existing WIP and theme-palette branches conflicts; integrate these commits through the PR #38 source branch after review and revalidation.

Next: independently review the exact commits, integrate onto current PR #38, repair the actual TablePro post-render repaint in its owning component migration, and only then continue the compatible per-color-level Split/Seam recipe work. Do not claim visual parity or treat the 13 failures as pre-existing.

## Fields integration blocked by incoming runtime change (2026-10-21)

Preservation branch: `wip/parent-fields-runtime-blocker-20261021`. The exact Fields source `332f40d07e38512f91cfb038eb9174c413755686` and its independent review are committed. Before synchronizing incoming PR #38 changes, scoped Fields/Forms/Runtime verification passed 504 tests with three existing skips, strict Clippy and scoped formatting. After normally merging shared PR #38 through `adf334b21745b9f713c26275987d09bf22eeda16`, the same suite has 503 passes and one allocation-test failure. Publication to PR #38 stopped.

The unchanged isolated allocation test fails with inherited `NO_COLOR` present (one allocation versus zero expected), and passes when that variable is absent. Incoming `modal_backdrop` now reads `std::env::var_os("NO_COLOR")` on each paint. Independent diagnosis and a shared-layer repair are required; removing the variable is diagnostic evidence, not a gate workaround. Preserve incoming history and all reference artifacts. See [exact blocker evidence](../audits/2026-10-21-fields-publication-incoming-runtime-blocker.json).

Next: independently review the incoming runtime change, fix the per-paint environment dependency without changing the test or hiding failures, rerun the complete scoped gate under the original environment, normally synchronize/push PR #38, and verify its remote SHA. All four applications remain incomplete.
## Showcase Selection bypass removal (2026-10-21)

Landed on `parity/termrock-visual-baseline`: the Selection page's ANSI
frame-replay bypass (`selectable/frames.rs` + two `.ansi` fixtures) is
replaced by the shared overlay `Dialog` and real selection rendering; the
shell no longer calls `selectable.stamp(ui)`. Evidence, the two
fixture-masked 120x40 captures that now fail on out-of-scope shell/nav/
footer drift, and the remaining lower-tier backdrop question are recorded
in `docs/audits/2026-10-21-selectable-bypass-removal.md`. Next steps:
have the shell owners reconcile nav/sidebar/footer painting with the
frozen captures (then re-approve on purpose), and hand the tier-dependent
modal-backdrop mapping to the runtime owners.

## Landing-queue integration state (2026-10-21)

Queue integration onto `parity/termrock-visual-baseline` is complete through step 7; pushed tip `bfbb72516` (sequence `8a8e28300` → `499cec504` → `efda9593a` → `2d66ea9be` → `bfbb72516`, all gated and pushed under the push lock). Full landing/skip/hold record: [migration matrix, PR #38 landing-queue section](../architecture/termrock-migration-matrix.md).

Resume points:

1. `parity/buttons-reference-decision-20261020` executable pins: re-advance the Buttons reference decision through the cohort-review machinery (six reviewed audit docs pin `frozen_cli_references.json` at `2a43fb11…`; branch pins `d5d8d648…`). Docs commits already landed.
2. `parity/tablepro-picker-live-20261020`: re-port 10 commits from the old `query_editor` model onto the landed `query_input` filter-list design, then run its frozen tab-picker geometry assertions; land only if green.
3. Still held per queue: `wip/textarea-fields-stage3-20261021`, `wip/steps-shared-contract-20261021`.

## PropsList sheet contract correction (2026-10-21)

- Branch: `wip/props-dialog-contract-stage1-20261021`
- Exact local tip: `7c738acca9c0f51758e6a4331075ba5aba4775e8`
- Worktree: `/private/tmp/termrock-props-dialog-stage1-20261021`
- State: implementation correction is commit `386b895597b220d949304e506bd5c629792b8b25`; this handoff is committed at the tip above. The author attempted the shared push lock once and found it owned by `termrock-fix-findings workstream 2026-10-21`. No fetch, rebase, push, or lock modification was performed. Parent assumed publication and was sent the implementation SHA.
- Focused regression: `cargo nextest run -p termrock-controls --test props_sheet_regressions --target-dir /tmp/termrock-props-stage-target` — 5 passed, run `b83f3be9-3ad4-40e5-9474-cbf8be8ccf17`.
- Full affected crate: `cargo nextest run -p termrock-controls --target-dir /tmp/termrock-props-stage-target` — 151 passed, 0 skipped, run `24ccb375-63a4-4635-810c-79163d92b911`.
- Scoped strict Clippy, `rustfmt --edition 2024 --check` on touched Rust files, and `git diff --check` passed before the implementation commit.
- Regression source: `crates/termrock-controls/tests/props_sheet_regressions.rs`. Five probes cover bounds-clipped copy hints, wrapped-value bottom clipping, per-row copy-hint space, measured label width, and explicit part-style Set/Clear precedence. Red evidence is recorded in `docs/audits/2026-10-21-props-sheet-shared-contract.md`.
- Independent probe: `/private/tmp/termrock-props-sheet-independent-e4b919-20261021/crates/termrock-controls/tests/independent_props_sheet.rs`; immutable packet `/Users/donbeave/Projects/termrock-cli-evidence-20261021/props-sheet-independent-review-e4b919-20261021/review.json`, SHA-256 `2d156d1e8d6dba332d2844869481bc075ad3cec580319fb71e1a96d93c1685e7`. These reviewer files remain read-only.
- Next: parent publishes/merges the implementation commit when the shared lock is available, then asks the independent reviewer to rerun the immutable probe against the merged exact commit. Independent re-review, application parity, large-list alternate-theme coverage, Dialog chrome/scope, Jackin app migration, and full Tuiscotti gates remain incomplete.
