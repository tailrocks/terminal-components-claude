# Button shared-contract increment (2026-10-21)

Scope: `termrock-controls::Button` focus visibility and busy-prefix geometry. This is a bounded component change, not a Showcase migration or an application parity approval.

## Decisions and implementation

- Preserve the public constructors, IDs, update/activation path, and the existing default `Always` behavior. Add `ButtonFocusVisibility::{Always, KeyboardOnly}` as a per-button override. `Always` only promotes the runtime's existing eligible `FOCUSED` state to `FOCUS_VISIBLE`; it never manufactures focus, promotes disabled/busy controls, or promotes a control drawn inside an inert reference scope. `KeyboardOnly` uses the visibility bit delivered by the runtime.
- Keep the shared runtime as the sole input and focus engine. Tests exercise pointer down/up, keyboard focus, Enter/Space activation, and inert references through `termrock-test-support::Harness`.
- A busy button has one leading spinner prefix. Its checked marker is suppressed during busy state and its measurement no longer reserves a second marker lane. Busy/disabled controls remain non-activating.
- Owner-wide `Ui::reference` styles the actual Button parts without registering a focus stop, action binding, cursor, or control hit area. No second reference renderer was added.

## Marker contract from the first Button-only increment (superseded below)

The first Button-only increment intentionally left the marker unchanged: frozen source uses a one-cell dot and spacer, while current RadioOn/Off glyphs are three-cell strings. The later Theme contract below resolves this with distinct one-cell DotOn/DotOff roles; it does not crop or widen the Radio roles.

## Evidence

- Reference diagnosis inspected: `showcase-buttons-reference-diagnosis.json`, frozen source `4169783ae883788186d161af09ba9c481bd84e5c`; inspected candidate diagnosis source `67ebf35cdc2ae6c51169ab7939b51e70d045fadd`.
- Implementation branch base before this increment: `6f68e22a6666c44c9f9cf726b4ddb895e6daf68a`.
- A source-backed pre-change run of the focused contract test failed on the busy measured-width assertion (8 versus expected 6) and the attempted marker-role assertion (`nextest` run ID `9dc3d965-701c-47d2-bdde-806d70293544`). The marker assertion was revised to preserve frozen one-cell geometry following the diagnosis above; it is not claimed as a newly fixed behavior.
- After the bounded change, `cargo nextest run -p termrock-controls --test button_contract --no-fail-fast` passed 7/7 (run ID `870deb75-6343-422c-922c-6db263002730`).
- `cargo nextest run -p termrock-controls --all-targets` passed 120/120 (run ID `6a7407c9-e502-4eda-b178-3ede0ec6126b`).
- `cargo clippy -p termrock-controls --all-targets -- -D warnings` passed.
- Targeted `rustfmt --edition 2024 --check` is recorded with this change. Workspace formatting was not applied because it would modify unrelated files.

## Independent review correction

Independent review found that `KeyboardOnly` removed `FOCUSED` from the same local flags used to publish keyboard bindings. The added no-Tab regression reproduces the failure: after pointer down/up focuses the button, Enter left activation count at 1 instead of 2 (`nextest` run `7bf46007-e0fd-4e7c-871e-51ab2bd09c3c`, red before the fix). Binding publication now receives the runtime focus flags before a separate `style_flags` view applies the focus-visibility policy. The same regression checks both Enter and Space without a Tab round trip. The focused suite is green after the fix (`0c939a68-1e6a-4e14-9b34-d232961a434c`).

## Shared Theme contract increment

The reviewed follow-up agreement `button-dot-focus-recipe-agreement-5cd18153.json` adds `GlyphRole::DotOn` and `DotOff` as discriminants 42/43 while preserving `GlyphSet::new([&'static str; 41], ...)`. The new semantic storage lives in private string fields; the original 41-entry array is unchanged. Unicode defaults are `●`/`○`; `.ascii_glyphs()` configures `*`/`o`, and later explicit builder overrides still win. RadioOn/Off remain `(●)`/`(○)` and are not clipped or reused as Button dots.

Button marker fallback now uses DotOn/DotOff when MARKER inherits. Explicit Set, Clear, and slot handling remain intact. The shared color recipe changes only the Button gutter focus indicator to match `FOCUS_VISIBLE`; `FOCUSED` remains available for container/body emphasis and binding publication. The Mono fallback replaces the default Button-only gutter rule with a `FOCUS_VISIBLE` rule, while retaining generic FOCUSED gutter behavior for other families and custom Button mono rules. Focus policy no longer removes FOCUSED from `style_flags`.

Button tests cover real pointer focus for Always and KeyboardOnly, keyboard focus and no-Tab Enter activation across all eight Button variants and four color profiles, focused body-label styling, inert reference state, busy/checked prefix geometry, Unicode clipping, explicit glyph customization, and marker part override precedence. Theme tests cover the old public constructor shape and every role get/set, stable old role indices, ASCII/Unicode dots, all variants/profiles for the Button gutter recipe, non-Button List behavior, and authored Button Mono overrides.

The body-style regression was run at pre-change commit `536ac2603663bff241fe4f11ac121a4045a8843e` using the real Harness and a KeyboardOnly button with pointer focus. It failed because the label's FOCUSED bold recipe was lost (`nextest` run `271f1227-ecae-4d29-8348-834a708b0094`). The same contract now passes as part of the green Button suite; the temporary baseline worktree and test were removed.

The focused Button tests pass 8/8 (`nextest` run `26c20059-5492-40d7-ad1f-ee6b478141ac`). Controls all-targets passed 121/121 (`3f9c3bd3-52e0-4660-9bed-89dfa1a33d06`). Theme tests pass with one known test excluded: 136 passed, 1 skipped (`8c3528ee-47f4-4958-92c4-4272156c5a36`). The unfiltered theme run had 136 passes and one failure, `all_73_junie_slots_match_reference_capability_output`, where the reference formula expects ANSI256 index 77 and implementation returns 78. This is confirmed pre-existing at `536ac2603663bff241fe4f11ac121a4045a8843e` (run `35a265ae-2ec9-4132-b246-4fc440aad5dc`) and reproduced unchanged on this worktree (`776914ae-a451-400e-af8c-e50b81a9d0f6`, before the other theme fixes, then `19d0a244-4be6-4100-9bbb-6930311ed7b5`). No palette or approval data was altered. Strict Clippy for both packages and targeted rustfmt passed.

## Limits

These tests establish this Button/Theme contract only. They do not establish full Showcase behavior, CLI parity for any application, visual-baseline inventory coverage, or acceptance of stored expected artifacts. The unrelated ANSI256 semantic-palette test remains a known failure; the separate narrow-label fit issue remains a later Button increment.

## Exact merged-tip Theme check

On merged WIP tip `be4c5f14a953dd7f26f04123964102508e0e97b0`, an unfiltered `cargo nextest run -p termrock-theme --no-fail-fast --status-level fail` ran 137 tests: 136 passed and `capability_palettes::all_73_junie_slots_match_reference_capability_output` failed (nextest run `fe493d79-c2f7-40ef-988c-84d32580c332`). The assertion reports actual `Indexed(78)` versus expected `Indexed(77)` at `crates/termrock-theme/tests/capability_palettes.rs:68`. This remains a required unresolved Theme gate failure; the filtered 136-pass run is focused evidence only and is not a Theme suite pass. The same unchanged failure was reproduced at base `536ac2603663bff241fe4f11ac121a4045a8843e` in run `35a265ae-2ec9-4132-b246-4fc440aad5dc`. No expected output or palette logic was modified.

## Independent review correction: preserve non-Button Mono filtering

The independent review of `9cf76f4653b48d5f8e6049a63e7f26ea6ef9d1c4` found a P2 regression: splitting default and targeted Mono loops applied a `SCROLLBAR / CONTAINER / PRESSED` authored rule that the prior implementation filtered out. The reviewer’s public-resolution probe was green on base `536ac2603663bff241fe4f11ac121a4045a8843e` and red on the candidate; their native Theme suite had 257 passes and this finding, alongside the separately known palette failure. I reproduced the underlying patch leak locally before correcting it (nextest run `e4d8a916-2be5-48bc-b133-76ef8c5469f2`, actual `glyph: Set(WarningMark)` versus inherited).

The targeted Mono loop now applies the same Scrollbar container-press predicate as the generic loop. A regression test builds a public `Theme` with an authored Scrollbar Mono rule and verifies the resolved pressed container does not receive `WarningMark`. Button still resolves its default Mono keyboard-focus gutter to `FocusBar`, and authored Button overrides keep precedence. After the correction, Theme library tests passed 92/92 (run `1695bf7d-4809-43ab-8f1f-b9754b8a29ab`), and the Theme package passed 137 tests with only the known palette test excluded (run `fa2976d1-bfd7-4540-bd29-10503b4cdc08`). The unfiltered package run remains a required failure: 137 passed, 1 failed at `all_73_junie_slots_match_reference_capability_output`, actual ANSI256 `Indexed(78)` versus expected `Indexed(77)` (run `9b77611d-eb33-4217-af7f-d31f2099e165`). Strict Theme Clippy, targeted rustfmt, and `git diff --check` passed. No palette or expected data changed.
