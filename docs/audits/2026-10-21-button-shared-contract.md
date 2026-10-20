# Button shared-contract increment (2026-10-21)

Scope: `termrock-controls::Button` focus visibility and busy-prefix geometry. This is a bounded component change, not a Showcase migration or an application parity approval.

## Decisions and implementation

- Preserve the public constructors, IDs, update/activation path, and the existing default `Always` behavior. Add `ButtonFocusVisibility::{Always, KeyboardOnly}` as a per-button override. `Always` only promotes the runtime's existing eligible `FOCUSED` state to `FOCUS_VISIBLE`; it never manufactures focus, promotes disabled/busy controls, or promotes a control drawn inside an inert reference scope. `KeyboardOnly` uses the visibility bit delivered by the runtime.
- Keep the shared runtime as the sole input and focus engine. Tests exercise pointer down/up, keyboard focus, Enter/Space activation, and inert references through `termrock-test-support::Harness`.
- A busy button has one leading spinner prefix. Its checked marker is suppressed during busy state and its measurement no longer reserves a second marker lane. Busy/disabled controls remain non-activating.
- Owner-wide `Ui::reference` styles the actual Button parts without registering a focus stop, action binding, cursor, or control hit area. No second reference renderer was added.

## Marker contract left open

The checked Button fallback is intentionally not changed to `GlyphRole::RadioOn/RadioOff`. Frozen source `4169783ae883788186d161af09ba9c481bd84e5c` uses a one-cell dot and a following spacer in this control. The current `RadioOn/RadioOff` glyphs are three-cell strings (`(●)` / `(○)`); widening this Button marker would alter its measured geometry, while forcing either role into its one-cell marker clips the glyph. This increment preserves the current one-cell `SwitchKnob` checked fallback and blank unchecked fallback; explicit marker `Set`, `Clear`, and slot precedence remain tested. A generic one-cell dot/toggle semantic role needs a separate shared Theme API decision and owner. No baseline frame or stored approval was modified.

## Evidence

- Reference diagnosis inspected: `showcase-buttons-reference-diagnosis.json`, frozen source `4169783ae883788186d161af09ba9c481bd84e5c`; inspected candidate diagnosis source `67ebf35cdc2ae6c51169ab7939b51e70d045fadd`.
- Implementation branch base before this increment: `6f68e22a6666c44c9f9cf726b4ddb895e6daf68a`.
- A source-backed pre-change run of the focused contract test failed on the busy measured-width assertion (8 versus expected 6) and the attempted marker-role assertion (`nextest` run ID `9dc3d965-701c-47d2-bdde-806d70293544`). The marker assertion was revised to preserve frozen one-cell geometry following the diagnosis above; it is not claimed as a newly fixed behavior.
- After the bounded change, `cargo nextest run -p termrock-controls --test button_contract --no-fail-fast` passed 7/7 (run ID `870deb75-6343-422c-922c-6db263002730`).
- `cargo nextest run -p termrock-controls --all-targets` passed 120/120 (run ID `6a7407c9-e502-4eda-b178-3ede0ec6126b`).
- `cargo clippy -p termrock-controls --all-targets -- -D warnings` passed.
- Targeted `rustfmt --edition 2024 --check` is recorded with this change. Workspace formatting was not applied because it would modify unrelated files.

## Limits

These tests establish the changed Button contract within the controls package only. They do not establish full Showcase behavior, CLI parity for any application, visual-baseline inventory coverage, or acceptance of stored expected artifacts. The marker-role design remains unresolved as described above.
