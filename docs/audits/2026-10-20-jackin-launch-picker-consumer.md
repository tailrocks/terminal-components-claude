# Jackin launch Picker consumer audit — 2026-10-20

Status: incomplete; do not remove the frozen launch-picker stamp until the live component matches the full frame.

Frozen authority: source commit `4169783ae883788186d161af09ba9c481bd84e5c`; tuiscotti pin `a1ea27f757d75d07b05499276cd932e9f8e067ad`.

The Jackin manager consumer now projects each launch candidate into the shared Picker item model with the frozen account title, resolution level, additional-account count, availability tag, glyph, group, and disabled state. Its scope caption uses the workspace and resolved role. The Picker uses a fixed 84×9 layer, `GeometricCenter`, and column rows. In the 120×40 Harness, the live layer rectangle is `(18,15,84,9)`, matching the frozen frame geometry. Showcase scope API coverage remains excepted: the baseline Quick Picker demo invokes `Picker::draw` and then the page painter, whose content fill erases the shared caption. Removing the existing page painter would expose unrelated Picker visual gaps. Keep the baseline page output unchanged and record this API in the showcase coverage allowlist until shared renderer composition can render the caption without a private duplicate painter.

Application regression coverage exercises Escape dismissal without launch and Down followed by Enter launching the selected Codex agent. The latter passes with the default Jackin keymap intact. An earlier hypothesis that the global Down binding intercepted Picker navigation was not confirmed and was removed; no shared routing change is justified by the current test.

The 120×40 full-cell assertion remains red: the live title cell at `(20,16)` has background `Rgb(39,39,42)` while the frozen expected background is `Rgb(24,24,27)`. The frozen ANSI frame has no row focus gutter: `(20,17)` is blank and `(21,17)` is `▪`. The new live assertion fails because `(20,17)` contains `▎`. Live row columns also differ: Claude detail clips to `provider d…` while the frozen row clips to `provider …`; the ItemColumns offsets differ as well. These are shared Picker style/layout gaps; no screen-local rendering workaround is acceptable. The legacy `historical_launch_picker` source file remains in the worktree, but its app wiring is currently removed in this uncommitted, failing experiment. Do not commit that removal until the shared Picker owns an exact full-cell match. No parity completion is claimed.

Checks run:

- `cargo nextest run -p jackin-preview-app --test app_tests manager_launch_picker_down_then_enter_launches_selected_account_agent` — passed (1/1).
- `cargo nextest run -p jackin-preview-app --test app_tests manager_launch_picker_escape_dismisses_without_launching manager_launch_picker_down_then_enter_launches_selected_account_agent` — passed (2/2).
- `cargo nextest run -p jackin-preview-app --test app_tests manager_launch_picker_paints_the_approved_agent_cell` — failed on the full-cell title background as described above (1/1).
- `cargo nextest run -p jackin-preview-app --test app_tests manager_launch_picker_has_no_extra_row_focus_gutter` — failed because the live Picker paints `▎` at frozen blank cell `(20,17)` (1/1).
- `cargo nextest run -p showcase-demos 'pages::pickers::tests::'` — passed (2/2), including after duplicate scope painting was removed.

## Real CLI and tuiscotti coverage

The frozen runner defines the launch picker in `crates/visual-harness/tests/visual_baseline/scenarios/jackin.rs` as `jackin/manager/launch_picker`, with Enter, semantic wait on `Launch · choose Agent`, and dismissal phases. The hard-cases counterpart is `jackin/manager/hard_launch_picker`. Both are Overlay scenarios in the full matrix: 72×20, 80×24, 100×30, 120×40, and 160×50 at truecolor, 256-color, 16-color, `--color none`, and `NO_COLOR` (25 frames per scenario root). With `VISUAL_FULL=1`, the narrow overlay tier adds 40×12 and 60×18 at truecolor and `NO_COLOR` (4 frames per root). The runner pins tuiscotti at the frozen revision above and uses its real PTY CLI sessions, fixed environment, and `AnsiCursor` artifacts.

To compare binaries with identical tests, build the frozen and migrated Jackin binaries in separate worktrees, then run the same current `visual-harness` test binary twice, changing only `VISUAL_BIN_DIR`:

```sh
VISUAL_FULL=1 VISUAL_BIN_DIR=/path/to/frozen/target/debug cargo nextest run -p visual-harness --test visual_baseline --run-ignored all 'jackin::manager::launch_picker'
VISUAL_FULL=1 VISUAL_BIN_DIR=/path/to/migrated/target/debug cargo nextest run -p visual-harness --test visual_baseline --run-ignored all 'jackin::manager::launch_picker'
```

Repeat for `jackin::manager::hard_launch_picker`. The ignored scenario trials require `--run-ignored all`; both runs use the frozen snapshot root and same fixed PTY setup. Do not approve or regenerate snapshots to make the migrated run pass. The frozen baseline source commit and tuiscotti commit remain fixed above.
