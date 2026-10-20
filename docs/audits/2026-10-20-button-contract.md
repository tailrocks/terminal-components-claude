# Button contract audit — 2026-10-20

Status: **focused contract fixes implemented; independent review complete; application integration and frozen product parity remain open.**

## Source and implementation

- Candidate starting source: `99b719d85a0714c88484240dbc9184f110b518d4`.
- Initial implementation: `7dab012ee6c0094f7a2a0264a41d6530984c6820`.
- This follow-up adds the independent review fixes and focused regression tests.
- Frozen behavior source: `4169783ae883788186d161af09ba9c481bd84e5c`.
- Read-only independent reviewer: `/root/button_contract_audit`.

The shared button now owns busy spinner scheduling through the runtime controlled clock; the button's `BusyMarkerPolicy` retains Termrock's existing two-lane default and offers a single-lane baseline mode. `GlyphRole::SwitchOff` supplies the baseline unchecked toggle glyph. Button theme recipes supply busy, disabled, checked, pressed marker, and focus-gutter styling. Primary pressed-state foreground is explicitly `OnAccent` to override the generic family pressed role; danger uses `OnDanger`, while other variants use Canvas. The label and marker resolve atop the same container foreground, with component and theme overrides still applied afterward.

## Review findings and disposition

1. **P2 marker override loss:** the initial pressed-marker draw selected the container's resolved style, overriding marker-specific theme and widget patches. Fixed by moving the pressed foreground into `Part::MARKER` theme resolution and drawing from the resolved marker style. Added a test for explicit foreground and modifier patches while pressed.
2. **P1 focus contract regression:** the initial shared gutter recipe used `FOCUS_VISIBLE` only and lost the frozen button's default focus bar for `FOCUSED` specimens, including pressed+focused. Fixed to key the default gutter recipe on `FOCUSED`; disabled buttons still remove both focus flags. Added a focused-only gutter regression test.
3. **P2 pressed-marker variant mismatch:** a universal Canvas role did not match primary and danger on-fill ink. The pressed marker now resolves against the container style with variant-specific `OnAccent`, `OnDanger`, or Canvas roles; label and marker styles inherit their container before their own resolved overrides. Added primary/danger pressed-marker foreground checks while retaining the explicit marker-patch test.
4. **Remaining parity limits:** component-only tests do not establish application integration, spinner phase parity through real CLI journeys, or complete frozen visual/functional parity. Those remain acceptance blockers.

## Verification

- `cargo nextest run -p termrock-controls --lib button::tests`: follow-up 11/11 passed.
- `cargo nextest run -p termrock-theme --lib builtin::tests`: follow-up 13/13 passed.
- `cargo nextest run -p termrock-controls --all-targets`: initial implementation 107/107 passed.
- `cargo nextest run -p termrock-theme --all-targets`: 122 total, 114 passed, 8 failed on initial implementation.
- Independent review reran the full theme set at base `99b719d85` and implementation `7dab012ee6c0094f7a2a0264a41d6530984c6820`; both produced the same 114 pass / 8 fail result and identical failure identities and assertion values. Logs: `/tmp/button-review-theme-base-nextest.log` and `/tmp/button-review-theme-all-nextest.log`. These eight theme failures predate the button implementation.
- `cargo clippy -p termrock-controls -p termrock-theme --all-targets -- -D warnings`: passed after the follow-up fixes.
- Full workspace formatting has an unrelated pre-existing failure in `crates/showcase-demos/src/pages/chrome.rs`; do not treat it as a pass.

## Integration requirement

An application consumer must select the baseline busy-marker policy where appropriate and remove its repaint workaround. Independent application and visual reviewers must verify the live control, focus behavior, and spinner timing against the frozen binary. This audit does not claim either product or library acceptance for the four applications.
