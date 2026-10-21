# Feedback field-row private context refactor

Source increment based on PR #38 tip `45760ca43689cbaac6145b99361fc1ebe838e1ee` (`origin/parity/termrock-visual-baseline`). Before edit, `crates/termrock-feedback/src/settings_page.rs` matched SHA-256 `a2686aa692452b236c122cbe86a6419bc4b92ceaeb5eeb1f576552622ff14836` from the reviewed plan.

`paint_field_parts` now receives a private `FieldRowPaint` context holding its shared `Rect`, row y, selected word style, and selected padding style. The selected branch creates one context and reuses it for scope, kind, and source. The expectation on `clippy::too_many_arguments` is removed. Width calculation, saturating gap, paint ordering, clipping, exact styles, optional fields, and public APIs remain unchanged.

Before and after, an ephemeral in-module corpus rendered complete `Cell` debug records plus area, cursor, and registry count for 48 combinations: widths 79/80/81 and 119/120/121, shifted origins, selected/unselected rows, absent/present kind and source, empty/short/overlong values, and wide/combining Unicode. The temporary harness was removed after both captures. Both complete outputs contain 76,848 lines and have identical SHA-256 `a783726be915d4dc2387dd66329b3d5a9324d137001bcc116cd06035a4310a51`. Refactor evidence only; not an application approval or frozen-reference artifact:

- `before.txt` and `after.txt` in `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/feedback-field-row-private-context-20261021/`

Checks:

- `cargo nextest list -p jackin-preview-app --test app_tests` found the three relevant tests; no zero-test filter.
- `cargo nextest run -p jackin-preview-app --test app_tests -E 'test(settings_mounts_80_paints_the_approved_destination_cell) or test(settings_mounts_two_enter_paints_the_approved_mounts_frame) or test(settings_tabs_and_save_preview_stay_on_the_settings_route)'`: 3 passed, 48 skipped.
- `cargo clippy -p termrock-feedback --all-targets -- -D warnings`: passed.
- `cargo fmt --all --check`: passed.
- `cargo nextest run -p termrock-feedback --all-targets`: 123 passed, 7 failed, 0 skipped. Repeated these same seven failures against the unmodified source from the increment base: `derived_hintbar_metadata::metadata_without_a_hint_context_still_renders_and_redraw_allocates_nothing`, `feedback_clock::bound_scene_freezes_feedback_without_clock_or_update_effects_and_allocates_nothing`, `feedback_clock::warm_equal_time_sync_and_live_publication_allocate_nothing`, `feedback_clock::synchronization_is_equal_time_idempotent_and_backwards_or_wrong_policy_atomic`, `meter_defaults::capture_unchanged_default_cells`, `meter_semantic_tokens::quantized_aliases_follow_pinned_ordered_lift`, and `meter_semantic_tokens::junie_block_rest_matches_pinned_lift_on_every_surface_and_capability`. This increment does not touch those paths. Log: `/private/tmp/feedback-field-row-baseline-failures.log`.

This private parameter refactor makes no visual parity claim and does not approve the surrounding application settings rendering paths.
