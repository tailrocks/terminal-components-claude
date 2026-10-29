# Button Mono checked-marker regression

## Finding

The shared `Button` marker inherited the generic Mono `Part::MARKER` rules for `SELECTED` and `CHECKED`. Those rules replaced the button's semantic `DotOn` role with `Chosen` or `Checked`. A real Showcase Buttons page rendered `✓` where the frozen Button contract renders `●`.

Independent probe: `/private/tmp/button-2d66-independent-probes/tests/mono_showcase.rs`; public `ButtonsPage`, real `Harness`, checked Verbose button, 120×40, `Theme::junie().for_level(ColorLevel::Mono)`. Before `913a74b879039fcf115318edee303c02d395869f` passed; `2d66ea9be9b55b3f3a3b3c960b9acfeeff9c566` failed with `✓`.

## Change

Mono fallback now skips only generic selected/checked marker glyph substitutions for `Family::BUTTON`. Button `DotOn` and `DotOff` remain the component fallback. Other families retain the generic Mono rules. Theme recipe, overlay, instance `Set`, `Clear`, and slot precedence remain in the normal resolver.

## Evidence

`crates/termrock-controls/tests/button_contract.rs::checked_button_marker_keeps_its_semantic_glyph_in_every_color_profile` failed before the change at Mono (`✓` versus `●`) and passed after. It also checks unchecked markers, custom `DotOn`/`DotOff`, authored theme glyphs, instance `Set`, and instance `Clear`.

`downgrade::tests::mono_rule_counts_match_the_documented_totals` pins the generic rule identity set and the Button effective identity set. The generic contract remains 20 rules. Button applies 18: exactly `(MARKER, SELECTED)` and `(MARKER, CHECKED)` are omitted; its existing focused-gutter substitution and all other effective rule identities stay pinned. All other family count expectations remain unchanged.

Focused `cargo nextest run -p termrock-controls --test button_contract`: 12 passed, 0 skipped. Full Theme + Controls `cargo nextest run -p termrock-theme -p termrock-controls --all-targets`: 283 passed, 1 failed, 0 skipped. The sole failure is the known `all_73_junie_slots_match_reference_capability_output`; it predates this Button change and remains a visible unresolved palette/reference issue. No other failures appeared. Final full-suite log: `/private/tmp/button-mono-marker-20261021-theme-controls-final.log`, SHA-256 `cd3a5a66af753072c68259021c63e4707fad4f7a33a02d3c4a0aa054d6e1d0c9`. Independent unchanged real Showcase probes `mono_showcase` and `showcase_live`: 2 passed, 0 skipped after the change; log SHA-256 `4368953082663088c0030d0bef2ffa47333da051336f003da697a4dfcaaf0b40`.

This is a shared Button/theme regression fix; it does not establish Showcase or four-application parity. Pressed color ladder failures and full application comparisons remain separate work.
