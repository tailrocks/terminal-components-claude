# FieldHover capability-profile repair

Status: **The source-qualified Junie FieldHover surface is repaired and verified through a real TextInput consumer in all four color profiles. The all-73 fixture remains intentionally red; it is not an acceptance oracle for unrelated semantic roles.**

## Contract and implementation

The frozen baseline (`4169783:src/theme.rs`) authors FieldHover as `#2f2f34` for truecolor, ANSI256 index `237`, ANSI16 `DarkGray`, and mono `Black`. The current Junie source had truecolor `#232328` and ANSI256 `234`; ANSI16 fell through generic conversion without an authored semantic palette. These are real FieldHover defects, distinct from the all-73 fixture's generic-projection assumptions.

The built-in truecolor token now uses `#2f2f34`, and the ANSI256 authored surface is `237`. Junie's ANSI16 palette is derived from the existing generic source projection, then only the independently source-qualified `field_hover` slot is set to `DarkGray`. Other tokens retain generic ANSI16 conversion and the existing ANSI16 foreground repair. The existing mono field-hover value remains Black. No button recipe, legacy mono rule, raw quantizer, or other token expectation was changed.

The public `TextInput` consumer tests cover the actual hovered field background plus direct `Theme::resolve` of `Role::Surface(Surface::FieldHover)` for all four profiles. They also exercise editing+hover, disabled hover suppression, read-only hover, focused read-only hover, empty narrow input, an explicitly customized field token, and an explicit ANSI256 capability palette. A separate assertion proves the authored ANSI16 palette differs from generic conversion only at the qualified FieldHover token.

## RED/GREEN and verification

- RED before product edits: `cargo nextest run -p termrock-fields --test field_hover_palette --status-level fail`, run `2d534502-1942-4c9f-a2c4-09d8a3f4ba7b`, failed through the real TextInput consumer: actual truecolor hover `#232328`, expected frozen `#2f2f34`.
- GREEN focused consumer: same command, run `8fdacea0-626c-4d83-8e36-03d110939a55`: 4 passed, 0 skipped.
- `cargo nextest run -p termrock-theme --all-targets --no-fail-fast --status-level fail`, run `cae0d752-ef60-44ca-8ac8-2f39f8224562`: 139 passed, 1 failed, 0 skipped. The only failure is the unchanged `all_73_junie_slots_match_reference_capability_output` fixture. It reports ANSI256 candidate `field_hover=237` against fixture expectation `236`, computed by raw projection from the candidate TrueColor token. Its unrelated palette-role failures remain unapproved and unmodified, as required by the semantic-reference diagnosis in `/Users/donbeave/Projects/termrock-cli-evidence-20261020/branch-consolidation/theme-semantic-reference-decision-plan-4ae4cb9.json`.
- The source-qualified `theme_holla_states` Field and FieldHover expectations and the built-in Junie token assertion were updated from the old incorrect surface to the frozen source values; the remaining cases in those suites passed in the 139 green tests.
- Focused field consumer was rerun after the implementation and profile-preservation additions: `cargo nextest run -p termrock-fields --test field_hover_palette --status-level fail`: 5 passed, 0 skipped.
- `cargo nextest run -p termrock-fields --all-targets --no-fail-fast --status-level fail`: 80 passed, 0 skipped (run `fc82c7f0-1a46-4d51-9f82-6d5bb35a04d2`).
- `cargo clippy -p termrock-theme -p termrock-fields --all-targets -- -D warnings`: passed.
- `cargo fmt -p termrock-fields -p termrock-theme --check` and `git diff --check`: passed.

No change to the frozen reference or the all-73 fixture is approved. The broader 73-token semantic inventory, button recipe cohort, unrelated profile roles, application parity, and architecture acceptance remain unresolved.
