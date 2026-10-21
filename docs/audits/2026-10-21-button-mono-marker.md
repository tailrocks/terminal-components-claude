# Button Mono checked-marker regression

## Finding

The shared `Button` marker inherited the generic Mono `Part::MARKER` rules for `SELECTED` and `CHECKED`. Those rules replaced the button's semantic `DotOn` role with `Chosen` or `Checked`. A real Showcase Buttons page rendered `✓` where the frozen Button contract renders `●`.

Independent probe: `/private/tmp/button-2d66-independent-probes/tests/mono_showcase.rs`; public `ButtonsPage`, real `Harness`, checked Verbose button, 120×40, `Theme::junie().for_level(ColorLevel::Mono)`. Before `913a74b879039fcf115318edee303c02d395869f` passed; `2d66ea9be9b55b3f3a3b3c960b9acfeeff9c566` failed with `✓`.

## Change

Mono fallback now skips only generic selected/checked marker glyph substitutions for `Family::BUTTON`. Button `DotOn` and `DotOff` remain the component fallback. Other families retain the generic Mono rules. Theme recipe, overlay, instance `Set`, `Clear`, and slot precedence remain in the normal resolver.

## Evidence

`crates/termrock-controls/tests/button_contract.rs::checked_button_marker_keeps_its_semantic_glyph_in_every_color_profile` failed before the change at Mono (`✓` versus `●`) and passed after. It also checks unchecked markers, custom `DotOn`/`DotOff`, authored theme glyphs, instance `Set`, and instance `Clear`.

Focused `cargo nextest run -p termrock-controls --test button_contract`: 12 passed, 0 skipped. This is a shared Button/theme regression fix; it does not establish Showcase or four-application parity. Pressed color ladder failures and full application comparisons remain separate work.
