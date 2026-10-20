# Menu bar pointer hover parity

## Change

`MenuBar::draw` now removes owner-level `HOVERED` before resolving its container style. A hovered title resolves its own title style and paints its full padded title rectangle, so the bar background and neighboring titles remain unchanged. MenuBar title styling uses a dedicated private theme variant, so its secondary rest ink and primary hover ink with Junie canvas-hover `#1c1c1f` do not change generic ContextMenu headings. ContextMenu title rendering removes owner-level `HOVERED` from heading style resolution.

Holla no longer overrides the title foreground or paints title pads after the shared component. Keyboard bindings, click handling, pressed brackets, and menu state transitions remain in the shared component.

## Evidence

The added real Holla `Harness<App>` pointer test failed before the component and recipe changes: the first hovered title pad at column 11 had `#111111` instead of `#1c1c1f`. After the change, it passes. It checks all six padded File-title cells, primary white foreground, every other cell in the header against the resting frame, full-frame restoration after pointer exit, and unchanged cursor position.

Focused checks passed:

- `cargo nextest run -p holla-ui --lib --filter-expr 'test(menu_bar_pointer_hover_changes_only_the_file_title_and_clears_on_exit) | test(menu_opens_before_publication_and_survives_focus_settlement) | test(interrupt_quits_outside_modals_and_quit_chord_precedes_open_menu)'`: 3 passed.
- `cargo nextest run -p termrock-overlays --lib`: 55 passed, including the context-menu heading preservation test, all-color interaction test, and existing menu keyboard, press, mono bracket, and popup lifecycle tests.
- The added `menu_bar_keyboard_and_pointer_activation_survive_all_color_levels` test passes at TrueColor, Ansi256, Ansi16, and Mono. It exercises keyboard open/Esc close, pointer press brackets, and click activation.
- `cargo nextest run -p holla-ui --lib`: 139 passed, including existing keyboard menu flows.
- `cargo clippy -p termrock-overlays -p holla-ui --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.

The frozen tag and paired harness source were prepared with source commit `4169783ae883788186d161af09ba9c481bd84e5c` and Tuiscotti revision `a1ea27f757d75d07b05499276cd932e9f8e067ad`. After materializing the tracked `snapshots` tree in the sparse worktree, the isolated candidate build succeeded. One intermediate live run exposed that the new private variant needed its own resting title recipe; that was added before the passing run. Live comparison `1792488891593369000-1666732d0b3c` ran `holla::elements::menubar::file-hover::120x40::truecolor` through real baseline and candidate CLI processes. Baseline capture, frozen replay, and candidate comparison all passed; each emitted one identical ordered checkpoint receipt (`79c2daaf1058e144160d3a20a5dff026ea52f8796e26d4b9d9cb5d45e66570bc`). Runtime expectation hash: `603cbca006aa889f48282ad81cb87d481bbaaf6a645e812a8afd884f7bc64e82`. This is a single-case pilot; `acceptance_complete` remains false. No baseline snapshot was edited or accepted.

A second regression test first failed against the initial hover commit: with the ContextMenu owner hovered, its heading changed from muted foreground on `#3f3f46` to white on `#1c1c1f`. The test now compares every full-width heading cell in rest and owner-hover frames and passes after scoping hover to MenuBar. This is a preservation regression check, not proof of frozen ContextMenu visual parity: the frozen `src/widgets/menu.rs` heading is muted without hover, while the earlier shared implementation had a separate pre-existing heading-style mismatch.

The existing Holla digest test remains red for `parity-discovery 80x24 junie truecolor`: expected `08e346d897de21b6`, actual `491485d3b0a74eb1`. The failure is recorded by the separate existing screen-fixture investigation; this hover work does not change its digest. The full Holla visual gate remains incomplete.
