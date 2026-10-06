# Props

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-controls.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Inventory: W34 · data and text · P3 · baseline component.

## Purpose and exclusions

Props paints a static label/value sheet. The caller supplies already-safe display values, tones, wrapping policy and any protected-value presentation. It preserves the baseline two-column alignment and continuation rows.

Props is display-only. It has no focus stop, pointer target, cursor, selection, copy action or domain introspection. PropsList owns the interactive variant and shares this row painter.

Props must not inspect a domain struct or derive labels from reflection; call Debug or Display on secret values; copy, activate, scroll or change caller-owned rows; or hide a second row layout engine inside a parent component.

## Public API

The signatures below are the proposed target, not a claim about current Rust exports. No signature is source-checked. Current Rust identifiers are implementation names and do not define the future public API.

```rust
Props::new(id: Id, rows: &'a [PropsRow<'a>]) -> Props<'a>

draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Builders:

```rust
label_width(LabelWidth)
patch(StylePatch)
patch_part(Part, StylePatch)
```

PropsRow contains a stable row key, borrowed label, typed PropsValue and display tone. PropsValue must distinguish ordinary text, styled text, an explicit empty value and a protected/redacted value. A protected value carries only its safe display form.

There is no update method because the surface has no durable interaction. If a caller needs an action or navigation, it uses PropsList.

## Ordinary use

No consumer recipe owns static Props directly; the interactive composition appears in [EX-06 — Tree and detail layout](../api/consumer-recipes.md) (`proposed_target`) through PropsList.

Real preview consumers are the Jackin manager and accounts views at the frozen commit. An independent synthetic fixture supplies borrowed rows with mixed plain, styled, masked, and empty values.

The caller supplies already-safe display values, tones, and wrapping policy for the frame. Props measures and paints the sheet; it never registers focus, hit, or capture ownership.

## Ownership

| Concern | Owner |
| --- | --- |
| Rows, values, tones and wrap policy | Caller, borrowed for the frame |
| Label width and row measurement | Props::measure and the shared layout foundation |
| Theme resolution and part patches | Termrock theme |
| Focus, hit testing and pointer capture | Runtime; unused by static Props |
| Painting | Props::draw, with no semantic mutation |

measure and draw use the same label-width and wrapping calculation. The label column is shared across rows; wrapped values create continuation rows with no repeated label. Clipping is constrained to area, including zero and nonzero-origin rectangles.

The default label column is the widest label plus two cells, measured in display width. Keep values aligned to that column; wrapped continuation lines omit the label rather than repeating or indenting a second label column.

Shared dependencies: [`identity`](../foundations/identity.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`text`](../foundations/text.md), and [`authoring`](../foundations/author.md).

## Customization

Parts with supported style patches are container, label, value, separator and status.

Ordinary customization patches declared label or value tones. Preserve the frozen baseline's label alignment, continuation spacing, value emphasis, empty-value treatment, masking glyphs, surface color and clipping. Part patches affect the cells they name and cannot replace geometry or focus ownership.

Advanced customization keeps correct rendering for Unicode labels and values, grapheme boundaries, long wrapped values, narrow widths and all declared terminal color capabilities (truecolor, 256, 16, no color and nocolor). Theme fallback follows [theme](../foundations/theme.md); text width and wrapping follow [text](../foundations/text.md).

## Behavior

Props has no focus stop or hover state, no pointer or keyboard action, no text editing or selection, no component-owned animation, and no motion behavior. Values are caller-controlled.

Resize re-measures and wraps from the new constraints without mutating rows. Semantic tones and baseline fallback are preserved in every capture capability.

## Visual matrix

| Axis | Required behavior |
| --- | --- |
| Geometry | normal, zero area, 1-cell area, nonzero origin, exact fit, 1-cell short, long Unicode content, narrow then wide |
| Focus/hover | Not applicable; no focus stop or hover state |
| Activation | Not applicable; no pointer or keyboard action |
| Editing/selection | Not applicable; values are caller-controlled |
| Resize | Re-measure and wrap from the new constraints without mutating rows |
| Motion | Not applicable |
| Color/capability | Preserve semantic tones and baseline fallback in every capture capability |

## Verification

Oracle evidence is the [frozen widget source](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/props.rs) at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Snapshot discovery is Jackin manager and accounts views at the frozen commit. See the [Props capture plan](../reference/capture-plans/props.json), the [visual contract](../design/visual-contract.md), and the [visual parity proof](../verification/visual-parity.md). The capture plan is planned with no approved expected artifacts. The frozen `src/widgets/props.rs` at `4a79c0a2` is an oracle only.

| Case | Required observation |
| --- | --- |
| W34-01 | Long label and wrapped value at narrow width |
| W34-02 | Mixed styled, plain, masked and empty values |
| W34-03 | Nonzero origin and clipped continuation line |
| W34-04 | Static Props creates no focus or pointer activation |

Each capture records dimensions, exact cells and styles, cursor visibility (none), focus owner (none), capture owner (none), action count (zero) and all source values. Use the dimensions and capabilities in the capture plan.

Required negative tests:

- secret references cannot leak raw values through formatting or debug paths;
- draw cannot mutate rows or semantic state;
- measured height equals painted height for wrapping and clipping;
- no cell outside area is written;
- static Props never registers focus, hit or capture ownership;
- unsupported part patches cannot replace geometry or surface ownership.

## Rejected use

Forbidden: painting padded label/value strings instead of using Props.

```rust
// Forbidden: preview owns label/value rendering by hand.
ui.paint_str(row, 0, "Host:   db-01         ");
ui.paint_str(row + 1, 0, "Port:   5432          ");
```

Rule: [ARC-012](../architecture/component-composition.md). Preview code must not recreate component rendering or generic interaction. The following patterns are prohibited in preview drawing paths:

```text
historical full-screen painters
snapshot files or exported frame data used as application output
hardcoded answer tables for 72x20, 80x24, 100x30, 120x40, or 160x50
fixture/scenario IDs that select a different painter
paused mode that bypasses component updates or layers
raw buffer access through termrock::author or an alias
direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls
padded strings that simulate columns, selection, fields, menus, or buttons
manual borders, focus gutters, scrollbars, or widget cursors
normal component drawing followed by a historical overlay
app-specific screens moved into a termrock crate under a generic name
fake state flags that display success without the normal component action
```

Use `Props::new` with borrowed `PropsRow` values; use PropsList when interaction is needed.

## Known gaps

- Capture plan W34 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
