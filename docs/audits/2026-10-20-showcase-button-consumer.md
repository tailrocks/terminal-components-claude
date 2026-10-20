# Showcase Button consumer migration

Status: focused migration only; Showcase parity and Termrock acceptance remain incomplete.

## Reference and implementation

The component contract is based on reviewed Button source `2d65740b717a86fbc5701c0286f7b35454ea5332`, layered over PR #38 integration commit `407d727f9f952bafcbb842e8be341ffb32120e4e`. The Button now owns its gutter, toggle marker, label, pressed treatment, state resolution, and hit registration. `ButtonsPage` draws the same `Button` instance it updates. `BusyMarkerPolicy::Replace` is selected through the public builder for the playground and inert state matrix. The policy does not alter current non-toggle busy sample; it preserves the frozen contract when a toggle is busy.

Removed `legacy_gutter`, caller-painted toggle markers, and the pressed-state cell erasure. The state matrix still uses `Ui::reference` for noninteractive state previews; the real Button renderer owns all preview cells, and reference rows remain inert. Exported `BusyMarkerPolicy` from `termrock-controls` and the public `termrock` facade after consumer compilation exposed the missing export.

## Remaining baseline difference

The frozen source `4169783ae883788186d161af09ba9c481bd84e5c` Buttons page includes separate Toolbar, Button row, Icon buttons, Options, and States sections and their option-controlled layout/interaction flows. The current Showcase Buttons page has only the playground and state matrix. This increment does not restore those sections: current public composition/API does not provide the baseline ButtonRow overflow/wrapping behavior, IconButton, or the page Options/States section navigation. Recreating those mechanics in this page would violate shared ownership. Track these as open Showcase parity work for shared layout/widgets and page composition.

The paired real-CLI verifier must still compare the frozen and candidate Button page at 120x40 and the adjacent/supported scenarios. This record reports no full visual or functional parity result.

## Focused checks

- `showcase-demos` harness test activates the Auto-approve control and verifies state change through the live Button, while confirming state-preview controls are not registered as interactive targets.
- `button_public_consumer.rs` compiles public `BusyMarkerPolicy`, Button parts, and builder use through the `termrock` facade as an external integration-test crate.
- `cargo nextest run -p showcase-demos --no-fail-fast`: 44 passed, 0 skipped.
- Deliberate defect proof: temporarily restored the old post-draw literal `▎` gutter repaint. The focused Harness test failed at the focused-gutter cell (`left: "▎"`, `right: "f"`); restoring the shared-renderer path passed.
- Direct `rustfmt --edition 2024 --check` on the changed Rust files passed. Workspace `cargo fmt --check` still fails on unrelated pre-existing fixed-frame formatting in `crates/showcase-demos/src/pages/chrome.rs`.
- `cargo clippy -p showcase-demos --all-targets` exited successfully with existing warnings in `pages/grid.rs` and `pages/settings.rs`. Adding `-D warnings` fails on those same unrelated warnings.
- Full application paired comparison and all Showcase flows remain pending.
