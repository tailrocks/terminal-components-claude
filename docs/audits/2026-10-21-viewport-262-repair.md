# TextViewport repair for reviewed 262ac80ed

## Scope and decision

The independent review of `262ac80ed138f14a204f8a1a32d4a48fe0cde111` found viewport regressions R3, R6, R7, and R8 in `crates/termrock-viewport/src/viewport.rs`. Review packet: `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/incoming-262ac80ed-independent-review-20261021/review.json`, SHA-256 `8703b69e78fe5056e41fdb40629f6f35ea481e83accb2bf5f101c0e037ea482e`.

This bounded repair keeps the existing vertical-only viewport contract. It removes the newly painted horizontal bar, which had no horizontal state or input handling, and restores a stable text width and row count. The public `reserve_gutter`, `scroll_gap`, and `bar_cursor` builders remain. Pointer mapping uses the committed logical wrapping width and row origin alongside the visible hit origin; cursor placement under horizontal clipping was not covered by the first repair and is documented below. Width and preferred-size calculations saturate.

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

## Follow-up: clipped wrapped pointer layout (795 review)

The independent review of `795badb0adb2a9d118bb2cce6eb467b13e35be36` found that update rebuilt its wrapping key from the clipped `Part::TEXT` hit width. The pinned review packet is `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/viewport-795badb0-independent-review-20261021/review.json`, SHA-256 `49df6d35a24ed29cbbe4c77672be35024b0e453e7166b07febc963fead6f292f`.

The real-Runtime regression uses logical area `24x4`, clip `10x4`, wrapping enabled, and `"abcdefghijklmnopqrstuv WORD"`. The independent packet's consumer passed on source `4cc10c3a7158123500fbe38435f2b68874b50561`; at reviewed `795` it rendered `W` at `(2,1)` but selected `0..22` instead of `23..27`. This new in-crate regression failed on reviewed `795` with the same wrong selection before the fix. The fix records the un-clipped area and resolved text width in the viewport's existing `LayoutFacts`, then maps pointer coordinates using the visible horizontal hit origin, logical row origin, and committed logical wrap width. No extra layout owner or application workaround was added.

The regression now passes with both top clipping and the original `10x4` clip. A horizontal clip test checks the visible hit origin against the actually rendered cell. Both tests exercise real mouse down/up, selection cells, and TrueColor, ANSI 256, ANSI 16, and Mono themes. The existing `resize_updates_current_viewport_before_replacing_published_geometry` test continues to cover the runtime resize publication guard.

Follow-up verification at the repaired source: `cargo nextest run -p termrock-viewport --all-targets --no-fail-fast` and the same command with `--features testing` each passed 58/58, 0 skipped. `cargo clippy -p termrock-viewport --all-targets --features testing -- -D warnings`, `cargo fmt --check --package termrock-viewport`, and `git diff --check` passed. The new wrapped regression failed on reviewed source `795` with the independently reported wrong selection and passes after this fix. This remains a crate-level result; the application-level coverage gaps above are unchanged.

## Follow-up: restore default frames and clipped caret geometry (72b review)

The independent review of `72b47055598317e0b0671e938b5bb8d066c35827` found two remaining issues. Packet: `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/viewport-72b47055-independent-review-20261021/review.json`, SHA-256 `83b8e06705a7e62066af40b79edf8b9f9c1210ef376bbd253b1e3969c598f70d`.

`VP72-R1` was an incoming 262 default visual change, not introduced by the 72b pointer fix: `draw_lines` called `Ui::scroll_edges` for every viewport, while pre-262 did not. I removed that implicit viewport fade and kept the shared explicit `Ui::scroll_edges` facility for consumers that choose it. An independently copied real-Runtime consumer at exact pre-262 source `4cc10c3a7158123500fbe38435f2b68874b50561` and the candidate now produce byte-identical complete default matrix artifacts: 288 TrueColor/ANSI256/ANSI16/Mono, wrap, size, and height configurations; 9 ordered checkpoints each; 2,592 checkpoints; 56,114,040 bytes each; SHA-256 `7272621df736dee66932e7f8cec325f0b80fb3640e467c8de44b87681fd26b50`; `cmp` exit 0. The output includes full cell values and runtime metadata. This compares component source to pre-262 behavior, not to the frozen application tag.

`VP72-R2` is an inherited pre-262 clipping gap exposed by the expanded contract, not newly introduced by 72b. With logical viewport `24x4`, clip `(4,1,8,3)`, wrapped text `"abcdefghijklmnopqrstuv WORD"`, and caret cell `(0,23)`, `W` renders at `(5,1)` but the runtime cursor was absent on both 4cc and 72b. The clipped caret fix now uses the visible horizontal portion of the text rect for the cursor x origin, the same logical row origin and wrap width as drawing, and rejects positions outside the visible intersection. It does not change `Ui::paint_str` or app coordinates.

Own copies of the independent clipping fixture were run against a detached exact-4cc worktree and the candidate. Before: both `visible_caret_matches_horizontally_clipped_rendered_character` and the pointer clipping matrix were red. After: both pass, including 32 TrueColor/ANSI256/ANSI16/Mono × wrap × top/left/right clipping pointer cases and the focused caret at `(5,1)`. The in-crate caret regression also checks all four profiles. Fixture copies and outputs are under `/private/tmp/viewport-contract-external-20261021`; the reviewer's original fixture was left untouched.

Verification after the 72b follow-up: `cargo nextest run -p termrock-viewport --all-targets --no-fail-fast` passed 59/59 with 0 skipped; the same command with `--features testing` passed 59/59 with 0 skipped. `cargo clippy -p termrock-viewport --all-targets --features testing -- -D warnings`, `cargo fmt --check --package termrock-viewport`, and `git diff --check` passed. The exact-pre-262 complete default-frame matrix remained byte-identical as recorded above. Frozen application captures, full workspace checks, and hosted CI remain outside this bounded viewport repair.
