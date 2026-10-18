# Brand

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-controls.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W01 · Group: Chrome · Phase: P2.

## Purpose and exclusions

`Brand` renders an application supplied identity lockup. It owns the lockup's measured label and optional metadata, while the application owns the name, artwork, and meaning. The same component is used wherever an application exposes its identity so spacing, accent treatment, and clipping stay stable.

Static mode is decorative. Optional interactive mode exposes one click action and uses the shared Button pointer feedback. The lockup never invents a product name, route, version lookup, or logo asset.

Exclusions:

- No built-in Jackin, Holla, TablePro, or Showcase branding.
- No navigation, routing, command dispatch, persistence, focus stop, or pointer hit target in static mode.
- No second press timer, animation loop, or bespoke interaction router.
- No redesign of the four baseline applications.

## Public API

Proposed target. No part of this API is source-checked against current code.

```rust
Brand::new(id: Id, label: &'a str) -> Brand<'a>

brand.update(&mut cx) -> Response<Activated>
brand.draw(&mut ui, area) -> Rect
brand.measure(&measure_cx, constraints) -> Size
```

Builders:

```rust
meta(&'a str)
interactive(bool)       // false by default
patch(StylePatch)
```

Borrowed props are `label`, optional `meta`, interactive eligibility, and style patch. There is no durable component state. Runtime owns focus, hover, pointer capture, and the shared 140 ms activation feedback when interactive mode is enabled. The typed action is `Activated { origin: ActivationOrigin }`.

Update, draw, and measure:

- `update` consumes only runtime events relevant to this identity and returns at most one typed activation. Static mode returns no event and never registers focus or hit ownership.
- `draw` is read-only with respect to component semantics. It paints the complete lockup inside the supplied rectangle and returns the consumed rectangle. It must not request focus, mutate props, or start timers.
- `measure` uses grapheme display width, metadata, baseline spacing, and compact/available geometry. It reports a zero or clipped result for zero area without writing outside the allocation.

## Ordinary use

No consumer recipe covers Brand yet.

Existing consumers include the Showcase chrome page and Jackin Preview chrome. These references describe observed starting behavior; they do not authorize changing those applications.

Ordinary use passes borrowed `label`, optional `meta`, and `interactive(false)` for a decorative lockup. The caller keeps the stable `Id` across frames and renders the lockup inside its chrome row. Interactive mode registers focus and hit ownership through the runtime and handles at most one typed `Activated` per gesture.

## Ownership

`Id` is stable across frames and unique per lockup. Runtime interaction is keyed by that semantic identity, never by paint order. Removing a lockup releases its focus, hover, and capture records. Reordering two lockups cannot transfer a pending press or action to the other lockup. Static lockups have no focus identity.

Shared dependencies:

- [`identity`](../foundations/identity.md) — stable `Id` and semantic ownership.
- [`input-actions`](../foundations/input-actions.md) — activation origin and response rules.
- [`runtime`](../foundations/runtime.md) — focus, hit testing, pointer capture, hover suppression, and time.
- [`layout`](../foundations/layout.md) — measured/clipped rectangles and Unicode cell geometry.
- [`theme`](../foundations/theme.md) — semantic surfaces, accent, and capability mapping.
- [`author`](../foundations/author.md) — constrained parts/slots and update/draw/measure authoring.
- [`conformance`](../foundations/conformance.md) — oracle classification and negative gates.

## Customization

Parts are `container`, `label`, and `meta`. Style resolution follows [`theme`](../foundations/theme.md); geometry, focus ownership, capture, and whole-surface replacement are not slots. Preserve the baseline filled accent surface, on-accent bold text, label/meta tones, padding, glyph widths, one-row height, and clipping. Rust namespace changes must never rename application-visible text in an oracle fixture. Compact mode removes only the outer padding prescribed by the baseline.

Interactive mode uses the same hover lift, pressed surface, eligibility rectangle, and activation timing as [`Button`](button.md). It remains click-only unless the caller explicitly binds a keyboard activation through the runtime. A decorative lockup is absent from Tab traversal.

Ordinary example: `Brand::new(id, "Jackin").meta("v2").patch(accent_patch)`.

## Behavior

| Area | Required behavior |
|---|---|
| Focus | Not applicable in static mode. Interactive mode may be focused only when registered by the caller/runtime. |
| Hover | Interactive hover lifts the accent surface. Keyboard input suppresses stale hover until pointer motion restores it. |
| Pointer | `pointer_down` captures only the clipped lockup rectangle; release inside activates once; release outside or removed target cancels. |
| Disabled | A disabled interactive lockup is not hit eligible, does not activate, and cannot retain pressed feedback. |
| Editing/read-only | Not applicable. Label and metadata are borrowed read-only text. |
| Resize | Re-measure and clip to the new rectangle; never retain geometry as state. |
| Unicode | Measure and clip by display cells, including combining marks, wide glyphs, and empty labels. |
| Capability/color | Resolve semantic accent/on-accent styles through the theme for truecolor, 256, 16-color, none, and `NO_COLOR` modes. |
| Motion | Use shared activation feedback only; no component-specific animation. |

## Visual matrix

| Axis | Required states |
|---|---|
| Geometry | normal, zero area, tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover (interactive mode only) |
| Activation | pointer down, held press, release inside, release outside, removed target, disabled target, typed activation, feedback at 0/139/140/141 ms and expired |

Decorative mode does not receive inapplicable focus or activation states. Every applicable visual variant is checked at 72×20, 80×24, 100×30, 120×40, and 160×50 under truecolor, 256, 16, none, and nocolor capabilities.

## Verification

The immutable visual and interaction oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). Current source at that commit is [`src/widgets/brand.rs`](../../src/widgets/brand.rs), whose legacy type is `Lockup` and whose source blob is `709dfe6233f5cc16042c5f70432ede57ffd09024`.

The source capture plan is [`capture-plans/brand.json`](../reference/capture-plans/brand.json) (W01, planned and uncaptured). Expected artifacts are intentionally absent until a trusted baseline capture is bound. ExistingOracle and ExtractedOracle remain distinct from future Extension cases; candidate rendering cannot approve its own expected output.

Required capture cases:

| Case | Exact requirement |
|---|---|
| W01-01 | Static label plus metadata at exact width and one column too narrow. |
| W01-02 | Interactive lockup: hover, press, release, canceled release, and one typed activation. |
| W01-03 | Two independent IDs; pointer targeting cannot cross between lockups. |
| W01-04 | Static lockup is absent from Tab traversal. |

Each case records exact cells, cursor, focus/capture/layer owner, action count and target. An unavailable source setup is a blocked capture, never a passing result.

Negative tests:

- Static mode registers no focus or hit target.
- Two lockups cannot share a pending capture or activation.
- Release outside, removed target, disabled target, and repeated draw emit no activation.
- `draw` and `measure` cannot mutate semantic state or start feedback.
- Custom part patches cannot replace the whole surface or bypass clipping.
- Candidate output cannot become an expected artifact.

Accept only after an external-consumer API example compiles, source-state tests and all applicable exact captures pass, documented parts change the intended cells, and an independent reviewer confirms the evidence.

## Rejected use

Forbidden: painting the product name with `ui.paint_str` in a chrome row and handling clicks with app-local hit math instead of using Brand.

```rust
// Forbidden: preview recreates the lockup and its click handling.
ui.paint_str(row, 0, "Jackin");
if clicked(row_cells) { open_home(); }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and app-local interaction in preview drawing paths. Use `Brand::new` with `interactive(true)` and handle the typed `Activated` action.

## Known gaps

- Capture plan W01 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
