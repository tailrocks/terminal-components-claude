# Runtime underlay fill repair

Status: bounded runtime repair complete on the WIP branch. Application parity and the Termrock acceptance gate remain incomplete.

## Scope

`Ui::fill_keep_ink(area, Color)` now writes ordinary spaces and replaces only the background. It preserves the underlay cell's foreground, underline color, modifiers, and foreground role provenance. Since a raw `Color` carries no semantic role, the prior background role is cleared. Layer cells are marked written through the existing `FrameState` or `LayerDraw` role-index path. No cell symbol or transport metadata is copied.

The change is limited to `crates/termrock-runtime/src/ui/paint.rs`. It does not change layer order, input modes, cursor requests, focus registration, or the existing nested-layer foreground behavior.

## Regression evidence

The regression test failed against the pre-fix `fill_keep_ink` implementation in this isolated worktree: the underlay cell's Green underline color became Reset. Preserved pre-fix log SHA-256: `10a214f340c57697b475aa32d8a1d56e98e2b650723ca738503447186ff38847`. The independently reviewed incoming e4 public `Runtime` fixture reported the same defect. That review also confirmed a nested lower/upper overlay foreground probe already passed; no foreground compositor defect was claimed.

The new runtime regression test, `ui::paint::tests::underlay_fill_preserves_all_ink_channels_and_foreground_role`, exercises a real layer buffer. It checks all copied style channels, foreground/background role handling, ordinary-space output, a wide grapheme adjacent to the filled region, and the clipped boundary.

## Checks

- `cargo nextest run -p termrock-runtime --lib ui::paint::tests::underlay_fill_preserves_all_ink_channels_and_foreground_role`: 1 passed, 0 skipped. Log SHA-256: `594cd889e8b3eddb835770ba49560a00b455539d75742fe34ec5dc4b4b22719e`.
- `cargo nextest run -p termrock-runtime --lib`: 260 passed, 0 skipped. Log SHA-256: `dde3f2882e6b9c802dc2ac06a27eef28cd1de40cdfa1ef127f77ed6f7221c729`.
- `cargo clippy -p termrock-runtime --all-targets -- -D warnings`: passed. Log SHA-256: `11ef5e93fa0b24223468feefba7e6ccac5cfa1044fd0209d42708400bee2253e`.
- `cargo fmt --check` and `git diff --check`: passed.

These checks establish this runtime crate's focused behavior only. They do not establish menu unification, full public-consumer acceptance, frozen parity, or any application's completion.
