# TextInput

Status: current/proposed split. `TextInput::new` and its core editing entry points are source-checked at `4d117b48`; the secret specialization, conflict policy, and remaining builders below are the proposed target.
Owner: termrock-fields.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W08 · Group: Forms · Phase: P3.

## Purpose and exclusions

`TextInput` is a single-line controlled editor. The caller owns the committed value and revision. Durable state owns the active draft, rollback snapshot, edit phase, caret, selection, and horizontal offset. Geometry, palette, and committed domain ownership never enter state, and an editing string is not copied on every paint.

Plain and secret specializations share the editing core while keeping secret data out of debug, logs, clipboard, snapshots, and rollback after close.

Exclusions:

- No form schema, persistence, domain validation side effect, or product-specific parser.
- No geometry or theme in `TextInputState`.
- No universal editor policy: TextArea intentionally differs in Escape and paste semantics.
- No requirement to retain current legacy `WidgetId`, `Outcome`, `RenderCtx`, or mutable `TextInput` API.

## Public API

Current source-checked declarations at `4d117b48` (`crates/termrock-fields/src/input.rs`):

```rust
TextInput::new(id)       // input.rs:980 `new`
TextInput::value(v)      // input.rs:1000 `value`
TextInput::read_only(y)  // input.rs:1056 `read_only`
input.update(cx, state, value)  // input.rs:1137 `update`
input.draw(ui, area, state)     // input.rs:1318 `draw`
```

Proposed target on top of the current declarations:

```rust
TextInput::new(id: Id, value: &'a str, revision: Revision)
    -> TextInput<'a, Plain>
TextInput::secret(id: Id, value: &'a Secret, revision: Revision)
    -> TextInput<'a, SecretText>

input.update(&mut cx, &mut state) -> Response<TextAction<M::Value>>
input.draw(&mut ui, area, &state) -> Rect
input.measure(&measure_cx, constraints) -> Size
```

Builders:

```rust
label(&'a str)
placeholder(&'a str)
help(&'a str)
required(bool)
disabled(bool)
read_only(bool)
validation(Option<&'a ValidationMessage>)
validator(&Validator)
blur(BlurPolicy)                 // baseline: Commit
secret_policy(SecretPolicy)      // secret specialization only
patch(StylePatch)
conflict_policy(ConflictPolicy)  // default PreserveDraftAndReport
```

`TextInputState<Plain>` contains `EditPhase`, draft, rollback value, caret, selection, and horizontal offset. `TextInputState<SecretText>` is redacted in `Debug` and not `Clone`. Safe observations are `phase()`, `caret()`, `selection()`, and `resolve_conflict(ConflictResolution)`; only Plain exposes `draft() -> &str`. Typed actions are `TextAction<V>::{ Edited, Commit { value: V }, Cancelled, Conflict }`. `Conflict` has no plaintext payload; resolution is explicit.

Update, draw, and measure:

- `update` owns editing transitions and key/paste/pointer input. Navigation focus is distinct from edit mode. Enter/F2 or completed click enters edit; Enter commits and validates; Tab/Shift+Tab commits before traversal; blur commit runs in update, never draw.
- `draw` is semantically immutable. It composes Field chrome, text/placeholder, selection, cursor, help/error, and optional secret marker from state. It must not begin/commit/cancel edits, copy strings, validate, or alter caller value.
- `measure` returns the baseline three-row Field geometry and horizontal text viewport. Click-to-caret mapping uses the same grapheme/display-width geometry as draw.

## Ordinary use

Consumer recipe: [EX-02 — Current controlled text input](../api/consumer-recipes.md) (`current_source_checked`).

Showcase input captures and Jackin editor consumers are protected references.

Ordinary use passes a stable `Id`, committed `value`, and `revision`. The caller keeps `TextInputState` across frames, routes `TextAction` (Edited, Commit, Cancelled, Conflict), and applies committed values itself. No silent overwrite occurs on source revision change; conflict resolution is explicit.

## Ownership

The stable component `Id` owns runtime focus/hit/capture. State survives frames only for the same semantic identity. A source revision change during an active draft follows `ConflictPolicy` (default `PreserveDraftAndReport`); no silent overwrite occurs. Removal releases runtime ownership and clears secret material. Reordering fields preserves IDs and does not move drafts between fields.

Shared dependencies:

- [`identity`](../foundations/identity.md) — stable semantic ID and revision identity.
- [`input-actions`](../foundations/input-actions.md) — keymap, paste, responses, and typed text actions.
- [`runtime`](../foundations/runtime.md) — focus, hit, capture, hover, cursor, and time.
- [`layout`](../foundations/layout.md), [`text`](../foundations/text.md) — Field geometry, grapheme editing, display widths, and selection.
- [`theme`](../foundations/theme.md) — semantic field, cursor, selection, error, and capability styles.
- [`secret-validation`](../foundations/secret-validation.md) — validation and secret lifetime/redaction rules.
- [`author`](../foundations/author.md), [`conformance`](../foundations/conformance.md) — controlled authoring and oracle proof.
- [`Field`](field.md) — shared chrome; [`TextArea`](text-area.md) intentionally differs in Escape/paste.

## Customization

Parts are `container`, `label`, `gutter`, `text`, `placeholder`, `selection`, `cursor`, `help`, `error`, and `marker`. [`Field`](field.md) owns shared chrome; this page owns single-line editing and viewport rules. Preserve baseline field rows, focus gutter, editing background, placeholder/error/help tones, selection modifiers, cursor position/visibility, masked bullet geometry, reveal-tail policy, clipping, and narrow layout. Style patches cannot replace geometry, focus/capture, or the whole surface.

Ordinary example: `TextInput::new(id, value, rev).label("Name").placeholder("Ada")`.

## Behavior

| Area | Required behavior |
|---|---|
| Keyboard | Navigation Enter/F2 starts editing. Editing Enter commits/validates; Escape restores the start-of-edit value; Tab/Shift+Tab commits and traverses. Arrow/word/home/end/delete grammar follows shared text foundation. |
| Paste | A tagged paste starts editing before insertion even from navigation mode. It is routed only after modal priority. Disabled/read-only paste is ignored. This behavior is intentionally different from TextArea. |
| Pointer | One completed click focuses, enters editing, and positions the caret by displayed grapheme width. A second click only moves the caret; it does not restart the edit snapshot. Press/release and capture are runtime-owned. |
| Focus/hover | Focus is navigation state, not edit state. Losing focus while editing commits per baseline blur policy. Keyboard suppresses stale hover. |
| Disabled/read-only | Disabled input cannot edit, paste, or capture. Read-only may focus and display selection/cursor policy but cannot mutate or emit editing actions. |
| Validation | Required/custom validation updates error state after commit and live corrections. Invalid text may remain a draft/committed value with error; Form submission validates separately. |
| Secret | Mask one bullet per grapheme, including wide characters. Tail reveal is explicit and only outside edit mode. Secret values never appear in `Debug`, clipboard, logs, snapshots, or rollback after close. |
| Resize/Unicode | Preserve source offsets and selection across width changes. Use grapheme boundaries and display-cell widths for combining, CJK, emoji, and masked text. |
| Capability/color | Resolve field, selection, cursor, error, disabled, and masked styles semantically across color capabilities. |
| Motion | No editor-owned animation loop; shared activation/fade policy only where baseline requires. |

## Visual matrix

| Axis | Cases |
|---|---|
| Geometry | normal, zero area, tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover |
| Activation | not applicable as a generic whole-surface action; pointer/edit gestures are covered below |
| Editing | navigation, editing, selected text, invalid, read-only, commit, cancel, blur, Tab traversal, source revision conflict |

Capture every applicable case at 72×20, 80×24, 100×30, 120×40, and 160×50 with truecolor, 256, 16, none, and nocolor. Record dimensions, symbols/styles/modifiers, cursor, focus/capture/layer owner, draft/committed values (redacted for secrets), typed action count/target, phase, and revision.

## Verification

The frozen output/interaction oracle is [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). Current implementation is [`src/widgets/input.rs`](../../src/widgets/input.rs), legacy `TextInput`, source blob `f440b74c7d305e987ec31bf04d34afd993844bb0`; shared key/edit helpers are [`src/widgets/field_common.rs`](../../src/widgets/field_common.rs), blob `bd41cdd85e9c2605c356f7468dc0de3e3c50e54b`. Current source tests include required-error correction, one-click editing, masked click geometry, narrow fields, and wide-grapheme cursor alignment.

Use [`capture-plans/text-input.json`](../reference/capture-plans/text-input.json), W08. It is planned and has no expected artifacts; source output must be bound before candidate testing.

Required capture cases:

| Case | Exact requirement |
|---|---|
| W08-01 | Focus then Enter; edit then Escape restores original. |
| W08-02 | Completed click on wide/combining/masked grapheme; Down-only does not edit. |
| W08-03 | Paste while navigation starts editing; disabled paste is ignored. |
| W08-04 | Edit then Tab/Shift+Tab/blur emits one commit and correct traversal. |
| W08-05 | Required empty and custom validation error; correction clears prior error. |
| W08-06 | Secret mode has no plaintext in Debug, clipboard, logs, snapshots, or rollback after close. |
| W08-07 | External value changes during draft; no silent overwrite. |

Negative tests:

- Navigation-mode Down/arrow cannot enter edit or mutate the value.
- Escape restores the start-of-edit value exactly; Tab/blur emits one commit and one traversal response.
- Paste begins edit for TextInput but never bypasses disabled/read-only/modal ownership.
- Source revision changes cannot silently overwrite an active draft.
- Secret text cannot appear in debug, logs, clipboard, snapshots, or post-close state.
- Draw/measure cannot mutate draft, validate, commit, or allocate a second editor.
- Candidate code cannot accept its own snapshots as expected output.

Accept after external API, source-state, exact snapshot, action, Unicode, validation, secret, conflict, and independent review gates pass.

## Rejected use

Forbidden: painting the draft text with `ui.paint_str` and owning key handling instead of using TextInput.

```rust
// Forbidden: preview owns editor rendering and key handling.
ui.paint_str(row, 0, &draft);
if let Key::Char(c) = key { draft.push(c); }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and app-local generic input handling in preview drawing paths. Use `TextInput::new` and route the typed `TextAction`.

## Known gaps

- Capture plan W08 is planned and uncaptured; no expected artifacts are bound.
- Implementation of the secret specialization, conflict policy, and remaining builders is future work on `termrock-implementation`.
- [FIX-001 prelude/forms](../implementation/code-remediation-backlog.md): the remaining contract signatures are proposed targets and not source-checked.
