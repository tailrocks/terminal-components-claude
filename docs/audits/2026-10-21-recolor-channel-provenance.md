# Recolor channel provenance

## Scope

Repair `Ui::recolor_rgb`, `Ui::recolor_pair`, and `Ui::recolor_fg` so color transformations update semantic paint provenance only when a channel's color changes. Public signatures and callback eligibility/order remain unchanged.

## Implementation

The three public methods delegate to one private per-cell transform. It clips against the active UI area, computes both output colors before mutating each cell, skips all writes and metadata updates when colors are unchanged, and marks a partial `PaintStyle` for changed channels only. A foreground change clears only foreground role provenance; a background change clears only background role provenance. Glyphs, modifiers, underline color, and other cell data remain untouched. Existing target dispatch handles pages and layers; no-op recoloring leaves unwritten layer cells unwritten.

## Regression evidence

`tests/recolor_provenance.rs` exercises public rendering APIs through the runtime harness:

- Identity foreground recoloring preserves the full semantic cell through dimming.
- Identity pair recoloring over an unpainted modal layer preserves the underlying glyph.
- RGB mapping invokes its callback once for each RGB channel, in foreground then background order, and writes mapped colors.

Focused test command:

```text
rustup run 1.99.1 cargo nextest run -p termrock-runtime --all-targets --features termrock-runtime/testing --test recolor_provenance
```

Result: 3/3 recolor regression tests passed. The full scoped runtime run in the same command selected 361 tests: 360 passed, 1 failed, 3 skipped. The retained failure is `scene_snapshot::warmed_modal_snapshot_projection_performs_zero_allocations` (observed allocation count 1, expected 0); this change does not edit that test or `paint.rs`. The three recolor tests themselves passed in that run.

## Remaining proof gaps

This increment does not establish full per-channel/profiles/Unicode coverage for all three APIs, allocation invariants, application call-site migration, or product parity. Those remain required before acceptance. The separate modal allocation failure remains unresolved and is not waived by this audit.
