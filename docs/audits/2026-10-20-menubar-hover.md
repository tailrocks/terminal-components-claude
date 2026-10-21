# Menu bar pointer hover parity

## Change

With title-scoped hover selected, `MenuBar::draw` removes owner-level `HOVERED` before resolving its container style. A hovered title resolves its own title style and paints its full padded title rectangle, so the bar background and neighboring titles remain unchanged. Title defaults come from a private recipe variant, then resolve under the public `MENU/DEFAULT/TITLE` identity with `Ui::style_defaults`, preserving family default variants, recipe overrides, overlays, and local part patches. ContextMenu keeps its original live owner state, so caller-defined hover recipes and overlays still apply to headings.

Holla no longer overrides the title foreground or paints title pads after the shared component. Keyboard bindings, click handling, pressed brackets, and menu state transitions remain in the shared component.

## Component-default compatibility

Native `MenuBar` rendering keeps the pre-hover-refactor `Bar` policy by default. This preserves the existing MENU/DEFAULT/TITLE recipe behavior and whole-bar owner hover, including the seven native `menu_bar` render fixtures. Applications opt into title-scoped hover through the public `MenuBar::hover_scope(MenuBarHoverScope::Title)` policy; Holla does so explicitly to match its frozen padded-title hover. The title-scoped policy uses the theme's `termrock.menu-bar-title` recipe, with local title patches and runtime overlays still resolved by the public MENU/DEFAULT/TITLE identity. Native component fixtures are not changed.

## Evidence

The added real Holla `Harness<App>` pointer test failed before the component and recipe changes: the first hovered title pad at column 11 had `#111111` instead of `#1c1c1f`. After the change, it passes. It checks all six padded File-title cells, primary white foreground, every other cell in the header against the resting frame, full-frame restoration after pointer exit, and unchanged cursor position.

Focused checks passed:

- `cargo nextest run -p holla-ui --lib --filter-expr 'test(menu_bar_pointer_hover_changes_only_the_file_title_and_clears_on_exit) | test(menu_opens_before_publication_and_survives_focus_settlement) | test(interrupt_quits_outside_modals_and_quit_chord_precedes_open_menu)'`: 3 passed.
- `cargo nextest run -p termrock-overlays --lib`: 57 passed, including the public default-variant/overlay/local-patch test, ContextMenu heading and hover-overlay tests, all-color interaction test, and existing menu keyboard, press, mono bracket, and popup lifecycle tests.
- The added `menu_bar_keyboard_and_pointer_activation_survive_all_color_levels` test passes at TrueColor, Ansi256, Ansi16, and Mono. It exercises keyboard open/Esc close, pointer press brackets, and click activation.
- `cargo nextest run -p holla-ui --lib`: 139 passed, including existing keyboard menu flows.
- `cargo clippy -p termrock-overlays -p holla-ui --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.

The frozen tag and paired harness source were prepared with source commit `4169783ae883788186d161af09ba9c481bd84e5c` and Tuiscotti revision `a1ea27f757d75d07b05499276cd932e9f8e067ad`. After materializing the tracked `snapshots` tree in the sparse worktree, the isolated candidate build succeeded. An intermediate live run exposed that the private defaults variant needed its own resting title recipe; a later run caught a missing default style source when resolving against the public variant identity. Both were fixed before the passing run. Live comparison `1792489418241711000-1666732d0b3c` ran `holla::elements::menubar::file-hover::120x40::truecolor` through real baseline and candidate CLI processes. Baseline capture, frozen replay, and candidate comparison all passed; each emitted one identical ordered checkpoint receipt (`79c2daaf1058e144160d3a20a5dff026ea52f8796e26d4b9d9cb5d45e66570bc`). Runtime expectation hash: `603cbca006aa889f48282ad81cb87d481bbaaf6a645e812a8afd884f7bc64e82`. This is a single-case pilot; `acceptance_complete` remains false. No baseline snapshot was edited or accepted.

A second regression test first failed against the initial hover commit: with the ContextMenu owner hovered, its heading changed from muted foreground on `#3f3f46` to white on `#1c1c1f`. ContextMenu's stock title remains unchanged on owner hover, while a separate test proves an explicit title hover overlay still applies. MenuBar tests also exercise a custom family default variant, a matching overlay rule, and local title patch precedence together. These preservation checks do not prove frozen ContextMenu visual parity: the frozen `src/widgets/menu.rs` heading is muted without hover, while the earlier shared implementation had a separate pre-existing heading-style mismatch.

After the explicit hover-scope policy and native-padding fix, the real CLI comparison was repeated at candidate commit `372177c3ee3b00f421b7facd7cfc0d93ba18f448`. Run `1792492225016917000-1666732d0b3c` passed baseline capture, frozen replay, and candidate comparison for `holla::elements::menubar::file-hover::120x40::truecolor`; all three phases produced checkpoint receipt `79c2daaf1058e144160d3a20a5dff026ea52f8796e26d4b9d9cb5d45e66570bc`. Candidate Holla binary SHA-256: `27cf7ee5c2b55777cd336e701691b92b3a5271058fe434a44bb1b944186d08f3`. This remains a single-case pilot; `acceptance_complete` remains false.

The existing Holla digest test remains red for `parity-discovery 80x24 junie truecolor`: expected `08e346d897de21b6`, actual `491485d3b0a74eb1`. The failure is recorded by the separate existing screen-fixture investigation; this hover work does not change its digest. The full Holla visual gate remains incomplete.
