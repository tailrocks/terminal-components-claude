# HintBar

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-feedback.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W41 · Group: Chrome · Phase: P2.

## Purpose and exclusions

HintBar paints the one shell-owned bottom hint row from effective action and binding metadata. The same binding view drives input, Menu and HelpOverlay. DerivedHintBar is an adapter/factory that selects metadata; it is not a second painter.

Exclusions:

- Parsing user-visible strings back into event bindings.
- Creating controls or duplicate actions from descriptive hints.
- Allowing each child/modal to paint another footer row.
- Reimplementing KeyHint formatting.

## Public API

Signatures are the target shape; builders may use the repository's consuming-builder convention.

```rust
HintBar::new(id: Id, hints: &'a [Hint<'a>]) -> HintBar<'a>
HintBar::from_bindings(id: Id, bindings: &'a BindingView) -> HintBar<'a>

HintBar::draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
HintBar::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size

HintBar::alignment(Alignment)
    .patch(StylePatch);
```

Effective binding metadata is borrowed; there is no durable state or typed action by default. Hints describe actions already handled by the active owner.

## Ordinary use

Existing consumers include Showcase chrome and Jackin compositions; their baseline output remains protected.

The caller supplies borrowed effective binding metadata. The active owner wins in this order: modal/editor/prefix context, then temporary mode, then active screen, then global fallback. Preserve one global bottom row.

## Ownership

| Concern | Owner |
| --- | --- |
| Effective binding view and labels | Caller/runtime layer resolver |
| Highest-priority owner selection | Layers/runtime |
| Chord formatting | KeyHint |
| Bottom-row placement | Application shell |
| Painting and fit/drop order | HintBar |

In narrow widths, drop lowest-priority complete hint groups and never split a key glyph group.

Shared dependencies: [`layers`](../foundations/layers.md), [`input/actions`](../foundations/input-actions.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`text`](../foundations/text.md), [`authoring`](../foundations/author.md) and [`conformance`](../foundations/conformance.md).

## Customization

Parts are container, chord, label, separator and editing-badge. Preserve shortcut notation, label spacing, final separator, status/badge placement and the baseline narrow-width marker. Descriptive hints are noninteractive.

Unicode labels and chord display widths follow [text](../foundations/text.md); theme and color fallback follow [theme](../foundations/theme.md). There is no component-owned motion.

Ordinary example: `HintBar::from_bindings(id, &bindings)`.

## Behavior

- Keyboard, mouse and focus do not target the bar itself. Descriptive metadata has no focus stop; hints do not duplicate controls.
- Editing badges reflect the active owner metadata.
- A lower-priority layer cannot override the active owner.
- Descriptive hints cannot create actions.
- Narrow fit cannot split a chord or create a second footer.
- Drawing cannot mutate binding metadata; remapped display cannot diverge from the effective keymap.

## Visual matrix

| Axis | Required states |
| --- | --- |
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, 1-cell short, Unicode label, narrow then wide |
| Focus/hover | Not applicable; decorative metadata has no focus stop |
| Activation | Not applicable; hints do not duplicate controls |
| Owner | navigation, editing, nested modal and prefix contexts |
| Composition | one footer in composite fixtures; no duplicate child bars |
| Color/capability | semantic fallback across capture capabilities |

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen widget source is frozen-at-4a79c0a2 ([hintbar.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/hintbar.rs)). Use [`../reference/capture-plans/hint-bar.json`](../reference/capture-plans/hint-bar.json), the [KeyHint contract](./key-hint.md), [runtime layers](../foundations/layers.md) and [visual parity proof](../verification/visual-parity.md); candidate code cannot create expected output.

| Case | Required observation |
| --- | --- |
| W41-01 | Normal navigation, editing and nested modal owner selection |
| W41-02 | Remapped chord is shown consistently with actual handling |
| W41-03 | Narrow priority drops do not split key glyph groups |
| W41-04 | Editing badge and final separator spacing |
| W41-05 | Composite fixture has no duplicate bottom hint rows |

Record exact cells/styles, active owner, hint order, dimensions/capability, cursor (none), focus/capture (none) and action count (zero).

Required negative tests:

- DerivedHintBar cannot paint independently of HintBar;
- a lower-priority layer cannot override the active owner;
- descriptive hints cannot create actions;
- narrow fit cannot split a chord or create a second footer;
- drawing cannot mutate binding metadata;
- remapped display cannot diverge from the effective keymap.

## Rejected use

Forbidden: a child screen painting its own footer row instead of letting the shell-owned HintBar render effective bindings.

```rust
// Forbidden: child paints a second footer row.
ui.paint_str(last_row, 0, "^S save   ^Q quit");
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits "direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls". Use the one shell-owned `HintBar::from_bindings` with the effective `BindingView`.

## Known gaps

- Capture plan W41 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
