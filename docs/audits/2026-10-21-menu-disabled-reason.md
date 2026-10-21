# Shared ContextMenu disabled reasons (WIP)

This audit records a bounded shared-widget implementation on PR #38's WIP
branch, based on read-only contract
`menu-disabled-reason-contract-20261021/contract.json` (SHA-256
`95a36ea84f66c7fa5bb5f7c0fb082f8259e10604475e3a4d0b01844d84fd660f`). The
frozen source is `visual-baseline` commit
`4169783ae883788186d161af09ba9c481bd84e5c`; no frozen artifact was modified.

`MenuItem::reason` borrows explanatory text, marks the item disabled, and is
independent of its executable chord and display hint. A later
`disabled(false)` re-enables activation and suppresses the reason. The shared
ContextMenu renders the selected disabled reason as a noninteractive `HELP`
decoration in its bottom border. It does not add a row or affect measurement.
The only theme change adds the muted semantic `MENU/HELP` recipe. Opt-in
`skip_disabled(false)` and `clear_on_leave(true)` model the frozen menu's
disabled-row keyboard and hover lifecycle while existing menus retain their
defaults.

Focused widget tests cover the borrowed reason/chord contract, unchanged
natural width, disabled pointer and keyboard activation, bottom-border
placement, all four color profiles, pointer-to-keyboard priority, clear-on-leave,
and `HELP` style/slot overrides without a hit target. Before the implementation,
the focused public API probes failed to compile because the reason API and
status decoration did not exist; after implementation the focused tests passed.
The pre-existing natural-width assertion remains exact (`16` cells), and passed
against the base and after the change.

The integration test `crates/termrock-overlays/tests/context_menu_disabled_reason.rs`
also compiles a consumer against public crate/runtime APIs with its own actions
and data. It verifies disabled-hover reason output, nonactivation from Enter or
the displayed chord, and reason clearing after keyboard movement.

The unfiltered overlays library suite reports one failure:
`menu::tests::default_menu_bar_scope_keeps_title_patches_off_the_padding_cells`.
It reproduces at this WIP's base `6841e67ed4d240d9b39478b2e99f39ff48b8189b`,
which contains incoming commit `262ac80ed138f14a204f8a1a32d4a48fe0cde111`.
An independent before/after run confirms it is introduced by `262ac80`:
before that commit, 125/125 overlays tests passed; after it, 124/125 passed and
this was the sole failure (logs at
`/private/tmp/incoming262-review-20261021/{before,after}-overlays.log`). Review
traced it to `262ac80`'s `MenuBar::draw` change: padding was
added to the label string and received the `Part::TITLE` style under the default
title-only hover policy, violating the assertion that the default title patch
applies only to title cells. A separate authorized fix restored painting the
label through its inset rect, keeping padding outside the title part. The exact
regression test was red at the WIP base and green after the fix; the complete
80-test overlays library suite, scoped Clippy, and formatting all pass. This
records the 262-introduced issue as fixed on this WIP, not as globally accepted
until integration and review.

**Not accepted as application parity:** Jackin/Capsule adoption, end-to-end CLI
comparison, full frozen flow inventory, resize/reanchor behavior, all-disabled
and zero/narrow Unicode matrix, independent review, full workspace gates, and
PR #38 integration are still outstanding. This WIP is only a shared component
increment. The branch does not claim the full product or library acceptance
gates.
