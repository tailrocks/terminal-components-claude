# KeyHint

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-feedback.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W42 · Group: Chrome · Phase: P2.

## Purpose and exclusions

KeyHint renders a typed Chord and its label using baseline modifier glyphs, multi-key notation and exact display width. It is a decorative leaf used by menus, help, hints and other chrome.

Exclusions:

- Parsing display strings into event bindings.
- Handling input or owning a focus/hit target.
- Creating a second shortcut formatter in a parent component.
- Splitting graphemes or key glyph groups during clipping.

## Public API

Signatures are the target shape; builders may use the repository's consuming-builder convention.

```rust
KeyHint::new(chord: &'a Chord, label: &'a str) -> KeyHint<'a>

KeyHint::draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
KeyHint::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size

KeyHint::patch(StylePatch);
```

KeyHint has no durable state, update method or typed action. The Chord and label are borrowed. The same Chord formatter is used in every parent context.

## Ordinary use

Existing consumers include Showcase chrome, menus, help and hint rows; their baseline output remains protected.

The caller supplies the borrowed typed Chord and label. One Chord renders identically in Menu, HelpOverlay and HintBar contexts. Disabled/descriptive binding notation is supplied by the parent.

## Ownership

| Concern | Owner |
| --- | --- |
| Typed chord and label | Caller/binding view |
| Chord formatting and width | KeyHint |
| Focus, hover, hit and capture | None; parent metadata remains decorative |
| Theme and part styles | Termrock theme |
| Clipping and geometry | Shared layout/text foundations |

Exact-fit and one-cell-short measurement must agree with painting.

Shared dependencies: [`identity`](../foundations/identity.md), [`input/actions`](../foundations/input-actions.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`text`](../foundations/text.md), [`authoring`](../foundations/author.md) and [`conformance`](../foundations/conformance.md).

## Customization

Parts are chord, modifier, key and label. Unicode labels, combining marks, grapheme-safe clipping and capability fallback follow [text](../foundations/text.md) and [theme](../foundations/theme.md).

Ordinary example: `KeyHint::new(&chord, "Save")`.

## Behavior

- KeyHint is decorative. It handles no input and owns no focus/hit target. KeyHint cannot register focus, hit or pointer capture.
- Display text cannot be parsed as a replacement binding.
- Clipping cannot split graphemes or key glyph groups.
- Width returned by measure cannot differ from draw width.
- A Chord cannot render differently solely because its parent is Menu or Help.

## Visual matrix

| Axis | Required states |
| --- | --- |
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, 1-cell short, Unicode label, narrow then wide |
| Focus/hover | Not applicable; no independent focus stop |
| Activation | Not applicable; decorative |
| Binding | single key, modified key, multi-key prefix, disabled/descriptive parent notation |
| Composition | identical rendering in Menu, HelpOverlay and HintBar |
| Color/capability | parent/theme fallback in every capture capability |

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen widget source is frozen-at-4a79c0a2 ([keyhint.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/keyhint.rs)). Use [`../reference/capture-plans/key-hint.json`](../reference/capture-plans/key-hint.json), the [input/action contract](../foundations/input-actions.md) and [visual parity proof](../verification/visual-parity.md); candidate code cannot create expected output.

| Case | Required observation |
| --- | --- |
| W42-01 | Single key, modified key, multi-key prefix and Unicode label |
| W42-02 | Exact fit and one-cell-short measure/paint consistency |
| W42-03 | Disabled/descriptive binding notation supplied by parent |
| W42-04 | One Chord renders identically in menu, help and hint contexts |

Record exact cells/styles, chord input, label input, dimensions/capability, cursor (none), focus/capture (none) and action count (zero).

Required negative tests:

- display text cannot be parsed as a replacement binding;
- clipping cannot split graphemes or key glyph groups;
- width returned by measure cannot differ from draw width;
- KeyHint cannot register focus, hit or pointer capture;
- a Chord cannot render differently solely because its parent is Menu or Help.

## Rejected use

Forbidden: a parent formatting and painting its own shortcut string instead of using the shared KeyHint formatter.

```rust
// Forbidden: parent owns its own shortcut format.
ui.paint_str(row, 0, "[ctrl-k]  Save");
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits "direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls". Use `KeyHint::new` with the typed `Chord` so every parent formats identically.

## Known gaps

- Capture plan W42 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
