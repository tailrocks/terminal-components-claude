# Jackin Capsule menus: first live-component migration

## Scope and revisions

- Work branch: `wip/jackin-live-capsule-menu-20261021` (review before integration).
- Starting commit: `1dee009130ca11876a8d5e53e04fe1046edb9888`.
- Frozen source: `visual-baseline` commit `4169783ae883788186d161af09ba9c481bd84e5c`.
- Shared menu API contract: `branch-consolidation/capsule-live-menu-contract-0b524f16-20261021/contract.json`, SHA-256 `e6589dbfc5f80a7282bcf1ff35bd93e98de65c829b6c1ea524cfd304ef8934f2`.

## Replaced mechanisms

- Removed Capsule's scenario/80×24/theme-dependent context-menu frame selector, its static menu view, post-paint and undrawn cursor registration. The static 19-row view was not driven by current tabs, pane selection, or clipboard state.
- Tab context menu now uses the selected tab part as its anchor, derives its title from the selected tab, and exposes title, split-right, and close-tab actions.
- Pane context menu now anchors at the pointer on the actual pane and derives enabled state, hints, and zoom label from live selection, clipboard, leaf count, and zoom state. It routes copy, paste, selection clearing, split, zoom, and close through the app's existing command/domain handlers; close uses the shared destructive dialog and removes the selected pane only after confirmation.
- The pane menu and tab menu request cursor retention from the real `TextViewport` owner. A focused test supplies a terminal caret in domain data and confirms that the actual rendered viewport cursor is retained through the menu. No cursor is synthesized or registered for an absent visual control.

## Checks

- `cargo nextest run -p jackin-preview-app --filter-expr 'test(capsule_tab_menu_uses_live_tab_data_and_real_actions) | test(capsule_pane_context_uses_viewport_owner_and_live_availability)' --no-capture`: **2 passed**.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- `cargo clippy -p jackin-preview-app --all-targets -- -D warnings -A clippy::needless_borrow`: passed; the allowance covers an unrelated pre-existing warning at `crates/jackin-preview-app/src/app/launch_cockpit.rs:256`.
- `cargo nextest run -p jackin-preview-app`: **170 passed, 15 failed**. Failures include `jackin_visual_baseline`, existing captured-frame/cursor assertions, menu-bar/context-menu integration, and launch flows. These are unresolved and prevent claiming application parity. Full failure output was captured in the command log; no expected artifacts were changed.

## Open contract gaps

- The focused paired frozen/candidate flow comparison has not been run. This increment does not establish full Capsule visual or behavioral parity.
- The existing 120×40 zoom frame test expects a cursor at `(3,34)` which came from the removed test-selected painter path. The domain pane model in the default Capsule fixture has no terminal caret. Resolving visible input/caret ownership requires a real shared terminal-view/editor contract and frozen-flow evidence; restoring a screenshot-specific cursor or row is not acceptable.
- Shared context-menu layout/input behavior and stale cursor ownership remain under independent review. Full crate test failures are not waived by the two focused tests.

No baseline tag, expected result, or test was modified to bless candidate output.
