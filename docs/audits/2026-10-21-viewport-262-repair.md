# TextViewport repair for reviewed 262ac80ed

## Scope and decision

The independent review of `262ac80ed138f14a204f8a1a32d4a48fe0cde111` found viewport regressions R3, R6, R7, and R8 in `crates/termrock-viewport/src/viewport.rs`. Review packet: `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/incoming-262ac80ed-independent-review-20261021/review.json`, SHA-256 `8703b69e78fe5056e41fdb40629f6f35ea481e83accb2bf5f101c0e037ea482e`.

This bounded repair keeps the existing vertical-only viewport contract. It removes the newly painted horizontal bar, which had no horizontal state or input handling, and restores a stable text width and row count. The public `reserve_gutter`, `scroll_gap`, and `bar_cursor` builders remain. The text rectangle now supplies the same origin and width to update-time pointer mapping, draw-time clipping, registered hit regions, and cursor placement. Width and preferred-size calculations saturate.

No horizontal scrolling was added. A shared horizontal scrolling contract still needs explicit state, actions, pointer regions, cursor behavior, and resolved layout before any horizontal bar can be shown.

## Regression proof

The added viewport tests were copied into a temporary worktree at exact pre-repair source `262ac80ed138f14a204f8a1a32d4a48fe0cde111`. `CARGO_TARGET_DIR=/private/tmp/termrock-viewport-prefx-target-262 cargo nextest run -p termrock-viewport --lib --no-fail-fast` reported 28 passed, 3 failed, 0 skipped. The failures were:

- `maximum_scroll_gap_saturates_for_measurement_update_and_draw`: debug overflow in the prior width calculation.
- `gutterless_pointer_selection_uses_the_drawn_text_origin`: click at visible column 3 produced no `bb` selection.
- `the_text_width_does_not_depend_on_whether_the_bar_is_painted`: short content width 23 versus long content width 22.

After the repair, on latest PR #38 source base `7716ad775c57b3c3ff2cbdcf1aaa393f44330ee3` plus this working change:

- `CARGO_TARGET_DIR=/private/tmp/termrock-viewport-repair-target-e4eae58 cargo nextest run -p termrock-viewport --all-targets --no-fail-fast`: 56 passed, 0 failed, 0 skipped.
- `CARGO_TARGET_DIR=/private/tmp/termrock-viewport-repair-target-e4eae58 cargo clippy -p termrock-viewport --all-targets -- -D warnings`: passed.
- With the same latest source and a distinct target: `CARGO_TARGET_DIR=/private/tmp/termrock-viewport-target-7716ad775 cargo nextest run -p termrock-viewport --all-targets --no-fail-fast` and the same command with `--features testing`: both 56 passed, 0 failed, 0 skipped.
- `CARGO_TARGET_DIR=/private/tmp/termrock-viewport-target-7716ad775 cargo clippy -p termrock-viewport --all-targets -- -D warnings`: passed.
- `cargo fmt --check -- crates/termrock-viewport/src/viewport.rs`: passed.
- `git diff --check`: passed.

The width regression test uses long, unwrapped lines and checks equal returned text dimensions, the full row count, and that text remains in the last row instead of being replaced by an inert bar. Width and gutterless pointer tests run at TrueColor, ANSI 256, ANSI 16, and Mono. The size matrix covers zero, one-cell, one-row, and ordinary regions. The gutterless test uses the real `Runtime`, mouse input, rendered cell, and resulting word selection. The maximum-gap test covers saturated measurement, update through an `App`, and draw at zero, narrow, and ordinary areas. The `widest_line` full-content scan has been removed; no horizontal width pass remains, including in wrapped mode.

## Remaining verification

These checks cover the viewport crate only. No Jackin paired CLI/Tuiscotti comparison, complete application inventory, full workspace Nextest/Clippy run, or hosted CI result is claimed. Those application and acceptance checks remain pending, and the viewport change is not approval of the incoming 262 application/menu changes.
