# Note

Status: implemented on `termrock-implementation`.
Owner: termrock-fields.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W07-note · Group: Forms · Phase: P3.

## Purpose and exclusions

`Note` paints a short wrapped help note under a control in the field-help
tone (Muted). It exists for copy that does not fit `Field`'s single
truncated help row: the frozen connection form paints its safe-mode
description as two wrapped rows, which one truncated row cannot carry.

Exclusions:

- No state, no update phase, no focus stop, no pointer target.
- No scrolling, no truncation marker, no title or icon chrome.
- No error semantics: errors stay on `Field::error`.

## Public API

```rust
Note::new(id: Id, text: &str) -> Note<'_>

note.draw(&mut ui, area) -> Rect
note.measure(&measure_cx, constraints) -> Size
```

Builders: `patch(&StylePatch)`, `patch_part(&[(Part, StylePatch)])`, and
`slot(Part::CONTAINER, f)` (replaces the whole surface).

`Note` is stateless. The caller owns the text. Parts are `container` (the
fill) and `help` (the text) under `Family::FIELD`; the text resolves the
`FIELD` `HELP` tone with zero recipe change.

Update, draw, and measure:

- `draw` fills `area` with `CONTAINER`, registers `Decorative` regions
  under `id` (the `Field` chrome precedent, so a note may share its
  control's id), and paints the wrapped lines left-aligned at `area.x`,
  at most one line per row of `area`. A degenerate rect paints nothing.
- `measure` is the unwrapped width by the wrapped row count
  (`termrock_text::wrapped_rows`).

## Ordinary use

```rust
Note::new(field::SAFE_MODE, SafeMode::ALL[draft.safe_mode].description())
    .patch_part(&[(Part::CONTAINER, StylePatch::new().set_bg(card))])
    .draw(ui, Rect { x: rc.x + 2, w: rc.w - 2, h: 2, .. });
```

The caller clamps the height to the rows it can show; `Note` clips to
`area` internally too.

## Ownership

`Id` owns style attribution only. Two `Note`s (or a note and its control)
may share one id: `DuplicateId` fires only on a second `CONTROL` region,
never on `Decorative` ones.

Shared dependencies:

- [`identity`](../foundations/identity.md) — decorative registration.
- [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md) — measured rows, clipping, semantic surfaces.
- [`author`](../foundations/author.md), [`conformance`](../foundations/conformance.md) — constrained customization and oracle proof.

## Customization

Parts are `container` and `help`. Preserve the help tone, left alignment,
wrapping, and clipping. [`theme`](../foundations/theme.md) owns semantic
style precedence; the `CONTAINER` slot is the documented way to replace
the surface.

## Behavior

| Area | Required behavior |
|---|---|
| Keyboard | None. |
| Pointer | None; the chrome is `Decorative`. |
| Focus/hover | Never a focus stop; wears no runtime state. |
| Disabled/read-only | Not applicable. |
| Editing | Not applicable. |
| Scroll/resize | No scrolling; re-wraps from props every frame. |
| Unicode | Wraps and paints by display cells. |
| Capability/color | The help tone resolves for truecolor, 256, 16, none, and nocolor. |
| Motion | None. |

## Visual matrix

| Axis | Cases |
|---|---|
| Geometry | normal, zero area, one row (second line dropped), narrow wrap, nonzero origin |
| Focus/hover | unfocused only (no other state exists) |
| Source | one line, two lines, overlong words (hard wrap) |

## Verification

The frozen oracle is [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The authority bytes are the
safe-mode description rows of the frozen `form-new` frames (120x40 lines
24-25, 100x30 lines 24-25).

Required cases:

- Wraps to the area width; takes at most the area height; clips.
- Muted tone on the caller-supplied surface.
- Never writes outside `area`; never registers a focus stop.

## Rejected use

Forbidden: painting description rows with `ui.paint_str` loops in preview
code instead of using `Note`.

```rust
// Forbidden: preview owns note rendering.
for (i, line) in wrap(desc, w).iter().take(2).enumerate() {
    ui.paint_str(row(i), line, muted_style);
}
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct
`ui.paint_str` rendering of controls. Use `Note::new` with the caller-owned
text.

## Known gaps

None.
