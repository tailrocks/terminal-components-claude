# TextArea

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-fields.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W09 · Group: Forms · Phase: P3.

## Purpose and exclusions

`TextArea` is a multiline controlled editor. The caller owns committed text and revision. Durable state owns multiline draft/editor positions, selection, vertical and horizontal scroll, edit phase, and follow-caret policy. Field chrome and text operations are shared with TextInput; they are not copied into a second editor.

Exclusions:

- No document persistence, product editor semantics, schema, or domain side effects.
- No generic widget trait or universal editor policy.
- No paste or Escape behavior copied from TextInput: TextArea intentionally differs.
- No geometry or palette stored in editor state.

## Public API

Proposed target. No part of this API is source-checked against current code.

```rust
TextArea::new(id: Id, value: &'a str, revision: Revision)
    -> TextArea<'a, Plain>
TextArea::secret(id: Id, value: &'a Secret, revision: Revision)
    -> TextArea<'a, SecretText>

area.update(&mut cx, &mut state) -> Response<TextAction<M::Value>>
area.draw(&mut ui, area, &state) -> Rect
area.measure(&measure_cx, constraints) -> Size
```

Builders: `label(&'a str)`, `help(&'a str)`, `rows(u16)`, `disabled(bool)`, `read_only(bool)`, `validation(Option<&'a ValidationMessage>)`, `wrap(WrapMode)`, `patch(StylePatch)`, and `conflict_policy(ConflictPolicy)` (default `PreserveDraftAndReport`).

`TextAreaState<Plain>` contains shared editor state, multiline caret/selection, vertical/horizontal scroll, edit phase, and follow-caret. Secret specialization is redacted and non-Clone like TextInput. Safe observations are `phase()`, `caret()`, `selection()`, `resolve_conflict()`, and Plain-only `draft()`. Typed actions are `TextAction<V>::{ Edited, Commit { value: V }, Cancelled, Conflict }`; an explicit cancel command is distinct from the baseline Escape binding.

Update, draw, and measure:

- `update` handles navigation scrolling, edit grammar, wheel, paste, and focus transitions. Enter/F2 or completed click enters edit. Enter while editing inserts a newline. Escape finishes and commits TextArea editing; Tab/Shift+Tab commits and traverses.
- `draw` is semantically immutable. It paints Field chrome, visible lines, selection, cursor, scrollbar, and fades from state. It must not commit, mutate scroll, or recalculate semantic values as a side effect.
- `measure` accounts for label, body rows, help/error line, text viewport, and scrollbar. It preserves source offsets across wrapped lines, empty final lines, and resize.

## Ordinary use

No consumer recipe covers TextArea yet.

Showcase textarea captures are protected baseline consumers.

Ordinary use passes a stable `Id`, committed `value`, and `revision`. The caller keeps `TextAreaState` across frames and routes `TextAction` (Edited, Commit, Cancelled, Conflict). Enter while editing inserts a newline; Escape finishes and commits (the intentional TextArea exception to TextInput rollback). Paste requires editing already active.

## Ownership

Stable `Id` owns runtime focus/hit/capture. Draft/editor state follows that ID and source revision. Reorder does not move a draft to a neighboring field; removal releases runtime ownership and clears secret material. External revision changes during edit use explicit conflict policy and never silently overwrite.

Shared dependencies:

- [`identity`](../foundations/identity.md) — semantic ID and revision.
- [`input-actions`](../foundations/input-actions.md) — multiline keymap, paste, and typed responses.
- [`runtime`](../foundations/runtime.md) — focus, hit, capture, hover, cursor, and time.
- [`layout`](../foundations/layout.md), [`text`](../foundations/text.md) — line/column measurement, graphemes, selection, and resize.
- [`theme`](../foundations/theme.md) — field, cursor, selection, scrollbar, fade, and capability styles.
- [`secret-validation`](../foundations/secret-validation.md) — validation, redaction, and secret lifetime.
- [`author`](../foundations/author.md), [`conformance`](../foundations/conformance.md) — API ownership and oracle proof.
- [`Field`](field.md), [`ScrollRegion`](scroll-region.md), and [`TextInput`](text-input.md) — shared mechanisms and intentional differences.

## Customization

Parts are `container`, `label`, `gutter`, `text`, `selection`, `cursor`, `help`, `error`, `scrollbar`, and `fade`. [`Field`](field.md) owns shared chrome; [`ScrollRegion`](scroll-region.md) owns scrolling/thumb/fade mechanism. Preserve baseline body fill, focus gutter, cursor, selection modifiers, line clipping, horizontal ellipsis, vertical scrollbar, edge fades, disabled/error colors, and narrow height behavior. Supported patches cannot replace geometry, focus/capture, or whole surface.

The baseline body contains the configured text rows with a two-cell inset. An
optional scrollbar takes one column. The footer keeps help/error at left and
`ln 3/12` or `a–b of N` at right. While editing, the current line has a
`border-strong` underline. Long lines scroll horizontally to reveal the
insertion point; an offscreen hardware cursor is hidden rather than painted
over unrelated text.

Ordinary example: `TextArea::new(id, value, rev).label("Notes").rows(6)`.

## Behavior

| Area | Required behavior |
|---|---|
| Keyboard | Navigation arrows/HJK/PageUp/PageDown/Home/End scroll. Enter/F2 enters edit. Editing arrows and movement change caret; PageUp/PageDown move caret by rows. Enter inserts newline. Escape commits (the intentional TextArea exception). Tab/Shift+Tab commits and traverses. |
| Paste | Paste requires editing already active; navigation-mode paste is ignored. Bracketed multiline paste preserves newlines. This is intentionally different from TextInput. |
| Pointer/wheel | One click enters editing and maps to line/column; wheel scrolls without changing focus. Runtime owns pointer capture and clipped hit areas. |
| Focus/hover | Focus is separate from edit mode. Losing focus commits per shared blur policy. Keyboard suppresses stale hover. |
| Disabled/read-only | Disabled cannot edit/paste/capture. Read-only can navigate/scroll and show selection per policy but cannot mutate text. |
| Scroll | Navigation arrows/page keys scroll the viewport. A manual wheel or thumb move preserves its offset and may leave the caret offscreen; wheel alone does not ensure it. During editing, a later keyboard step first moves the caret, then minimally scrolls to reveal that new position. Cursor movement, edits, and resize may reveal it; an offscreen hardware cursor stays hidden. Test boundaries and thumb capture. |
| Resize/Unicode | Keep source offsets over wrapped lines, tabs, newlines, empty final lines, combining/wide graphemes; clip writes to body. |
| Capability/color | Resolve field, selection, cursor, scrollbar, fade, error, and disabled styles semantically across capabilities. |
| Motion | Shared scroll fade policy only; no private animation loop. |

## Visual matrix

| Axis | Cases |
|---|---|
| Geometry | normal, zero area, tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover |
| Activation | not applicable as generic whole-surface activation; edit/scroll/selection gestures apply |
| Editing | navigation, editing, selected text, invalid, read-only, commit, finish via Escape, blur, Tab traversal, source revision conflict |
| Scroll | no overflow, start, middle, end, wheel boundary, thumb drag, resize while scrolled, edge fade, protected row |

Capture all applicable cases at 72×20, 80×24, 100×30, 120×40, and 160×50 under truecolor, 256, 16, none, and nocolor. Record exact cells, cursor, focus/capture/layer owner, scroll offset, caret/selection/source offsets, draft/committed values (redacted for secrets), and typed action count/target.

## Verification

The immutable oracle is [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). Current source is [`src/widgets/textarea.rs`](../../src/widgets/textarea.rs), legacy `TextArea`, source blob `5bfa5148a8b335c28ed52a5af305fa9df39917e5`; shared Field/edit helpers are [`src/widgets/field_common.rs`](../../src/widgets/field_common.rs), blob `bd41cdd85e9c2605c356f7468dc0de3e3c50e54b`. Current source tests cover read-only scrolling, boundary wheels, Unicode click/cursor geometry, manual scroll, resize, and narrow allocations.

Use [`capture-plans/text-area.json`](../reference/capture-plans/text-area.json), W09, planned and without expected artifacts. Bind source output before candidates.

Required capture cases:

| Case | Exact requirement |
|---|---|
| W09-01 | Edit Enter inserts newline; Escape commits and leaves edit mode. |
| W09-02 | Navigation-mode paste is ignored; edit-mode bracketed paste preserves newlines. |
| W09-03 | PageUp/PageDown and Home/End in navigation versus editing. |
| W09-04 | Selection spans newline, tab, and grapheme boundaries. |
| W09-05 | Wheel while editing followed by caret movement; follow-caret resumes correctly. |
| W09-06 | Height 0/1 and `rows+2` Field layout. |

Negative tests:

- Escape commits TextArea and never rolls back as TextInput does.
- Navigation paste never inserts; edit paste preserves newlines.
- Draw cannot mutate text, caret, scroll, or commit state.
- Manual wheel scroll remains until the documented follow-caret trigger.
- Resize, empty final lines, and Unicode boundaries cannot lose source offsets or write outside body.
- Secret text never appears in debug/log/clipboard/snapshot/post-close state.
- Candidate output cannot become expected output.

Accept after API, state/action, exact visual, scrolling, Unicode, secret, conflict, and independent review gates pass.

## Rejected use

Forbidden: painting text lines with `ui.paint_str` and owning newline insertion instead of using TextArea.

```rust
// Forbidden: preview owns multiline rendering and editing.
for (i, line) in draft.lines().enumerate() { ui.paint_str(i, 0, line); }
if enter { draft.insert(caret, '\n'); }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and app-local generic input handling in preview drawing paths. Use `TextArea::new` and route the typed `TextAction`.

## Known gaps

- Capture plan W09 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-001 prelude/forms](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
