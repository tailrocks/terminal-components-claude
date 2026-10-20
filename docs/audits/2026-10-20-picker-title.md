# Picker title baseline style

Frozen baseline direct CLI evidence is recorded at `/Users/donbeave/Projects/termrock-cli-evidence-20261020/independent-library-gates/picker-title-real-baseline/record.json`: the `jackin/manager/launch_picker/120x40/truecolor` frame at `(20,16)` contains `L`, bold, white foreground, and `#18181b` background. The frozen `Theme::title` recipe applies primary foreground and bold; Picker calls that shared title recipe. The shared `Family::PICKER` recipe lacked `Part::TITLE`, so Picker text did not inherit those attributes.

`picker_widget_title_uses_primary_bold_style_for_every_title_cell` renders the reusable Picker through the actual widget in a full terminal buffer. It locates the full `Launch Target` text and checks every title glyph for the theme's primary foreground, overlay surface background, and bold; it also confirms the cursor stays unset. Before the recipe change, the regression failed because the rendered title cells lacked bold. The baseline's exact reference cell is separately captured by the direct CLI evidence above.

`picker_title_keeps_frozen_primary_bold_style_and_public_precedence` checks the built-in recipe at TrueColor, Ansi256, Ansi16, and Mono for both Junie and Paper, and verifies a public family override can replace the foreground and clear bold. The integration test `picker_title_theme_override_overlay_and_local_patch_remain_overridable` verifies a custom family default variant and its state style, matching overlay precedence, and a local style patch that clears bold.

Validation passed: `cargo nextest run -p termrock-theme` (125 tests), `cargo clippy -p termrock-theme --all-targets -- -D warnings`, `cargo fmt --all -- --check`, and `git diff --check`.

This verifies the Picker title attributes and style precedence. It does not claim full Jackin frame parity; the separate full-frame comparison still has known background differences.
