# Termrock interaction contract

This document owns keyboard, pointer, focus, capture, editing, layer, and
state-transition behavior for the in-place Termrock refactor. It preserves the
observable interaction of the frozen applications while assigning future
implementation ownership to the Termrock runtime and typed component actions.

The oracle is `visual-baseline` commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Read this with the
[runtime foundation](../foundations/runtime.md), [input and actions
foundation](../foundations/input-actions.md), [identity foundation](../foundations/identity.md),
[layers foundation](../foundations/layers.md), [text foundation](../foundations/text.md),
the [public API](../api/public-api.md), and the [interaction parity proof](../verification/interaction-parity.md).

The current source still contains legacy implementation types such as
`WidgetId`, `Outcome`, `RenderCtx`, and `junie_tui`. They are evidence of the
starting implementation, not future public API requirements. The target public
surface uses stable semantic IDs, borrowed props, caller-owned state, and typed
actions.

Baseline interaction evidence is preserved in [`src/ui/ctx.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/ctx.rs),
[`src/core/focus.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/core/focus.rs),
[`src/core/hit.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/core/hit.rs),
[`src/core/event.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/core/event.rs),
and [`src/runtime.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/runtime.rs).
Component-specific input evidence includes [`src/widgets/input.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/input.rs),
[`src/widgets/textarea.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/textarea.rs),
[`src/widgets/dialog.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/dialog.rs),
and [`src/ui/popup.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/popup.rs).

## Event ownership and frame order

Each interaction cycle has explicit ownership:

| Concern | Owner |
| --- | --- |
| application/domain data and committed values | caller |
| draft text, cursor/anchor, collection cursor, scroll model | caller-owned component state |
| focus, hover, pointer capture, held press, feedback timing | runtime |
| hit geometry, layers, cursor arbitration, capability projection | runtime/`Ui` frame publication |
| semantic style, glyph, spacing, and recipe | theme/component |
| filesystem, services, PTYs, clipboard emission, persistence | application edge adapter |
| meaning of an activation | typed component action |

The runtime first reconciles/update state, then measures and draws, then
publishes hit regions, focus stops, cursor intent, and layer ownership. Pointer
events use the last committed valid geometry. A boot frame without geometry
cannot dispatch coordinate-dependent input. Resize or topology changes
invalidate coordinate-dependent geometry; a fresh frame must publish it before
dispatch resumes.

`draw`/`measure` are semantically read-only. They may publish geometry, cursor
intent, and layout facts but may not commit values, choose an item, close a
layer, validate by mutation, read the clock, execute a callback with side
effects, or perform I/O. Repeated draw must not change focus, selection, draft,
scroll, or caller data. Use [typed update/draw contracts](../architecture/overview.md)
and [conformance checks](../verification/conformance.md) to prove this.

Each update processes one normalized cause and emits at most one caller-facing
typed action. Consumption, visual invalidation, state mutation, and business
intent are distinct results. A wheel at a boundary can be consumed without a
paint; a timer can invalidate the frame without a component action. Duplicate
releases, repeated input, and stale geometry cannot produce duplicate actions.

## Focus, hover, and capture

### Focus ring

The focus ring is rebuilt in visible render order. `Tab` and `Shift+Tab` visit
enabled controls in that order and wrap. Composite components such as lists,
trees, grids, tab strips, pickers, and text viewports are one focus stop with an
internal cursor. Disabled controls may remain in hit geometry for containment
but are never focus stops.

Focus is independent from current selection, hover, held press, and editing.
The focused visual is normally the `▎` gutter plus bold text; container focus
may use a documented border or title treatment. The selected/current marker
remains visible when focus moves elsewhere.

### Hover suppression

After every keyboard input, the runtime marks hover stale and suppresses its
visual effect. The previous pointer position does not lift a newly focused row
or field. The next real pointer move clears suppression and resolves hover
against the current hit map, even when the pointer did not change widgets.
Wheel scrolling re-resolves the row under the stationary pointer after the new
frame publishes geometry. Keyboard input never moves pointer capture.

### Pointer capture and activation

An eligible pointer activation is a gesture, not a down event:

```text
pointer down -> capture target -> optional drag/held state -> pointer up
```

The runtime records the stable target at down. Release activates only when the
pointer is still over that same live, enabled, eligible target. Release outside,
target removal, target disablement, a changed identity, or a covered layer
produces no business action. A click outside a cancelable top layer dismisses
that layer once and never falls through to content below.

Pointer down may show a held-press state while capture is valid. A drag may
update selection, a scrollbar, a grid range, or a split seam according to the
captured component. Dragging cannot change the target of a pending button
activation. Right-click/context activation follows the owning menu contract and
does not bypass modal ownership.

### Activation feedback

Completed keyboard and pointer activation use the same feedback contract. The
runtime presents the activation frame before processing a later event that could
erase it. The baseline feedback interval is **140 ms** under the characterized
clock; verification samples 0, 139, 140, and 141 ms. Held pointer press and
post-release feedback are separate states. Feedback expires without creating a
second action. Busy or disabled controls do not emit feedback or activate.

The exact phase/time source is supplied by the runtime; components do not read a
wall clock in `draw`. See the [visual motion rules](visual-contract.md#capabilities-and-motion)
and [verification contract](../verification/interaction-parity.md).

## Keyboard grammar

The effective binding catalog drives handlers, key hints, menus, command
palettes, and help. A remapped binding must change every displayed and handled
occurrence together. Modal and editing scopes take precedence over ordinary
application commands; a text field must consume typing-conflicting `q`, `y`,
`n`, `Ctrl+L`, `Ctrl+N`, `Ctrl+D`, or similar chords before a background
shortcut can see them.

The shared baseline grammar is:

| Input | Meaning when applicable |
| --- | --- |
| `Tab` / `Shift+Tab` | next/previous enabled focus stop |
| arrows / `h j k l` | move the focused component cursor |
| `PageUp` / `PageDown` | page the focused scrollable component |
| `Home` / `End`, `g` / `G` | jump to start/end where the component offers it |
| `Enter` / `Space` | activate, toggle, select, or enter editing per component |
| `Esc` | cancel or finish the nearest owned editing/layer state |
| `0` | jump to the host navigation when the host binds it |
| `[` / `]` | previous/next page or tab when the host binds them |
| `?` | help layer when the host binds it |
| `q`, `Ctrl+C` | host quit policy, with editing/modal/running-work precedence |

Component-specific bindings remain in each [component contract](../components/README.md)
and are rendered in contextual hints. Navigation keys never mutate a caller's
controlled value without the component's typed action and caller acceptance.

### Escape precedence

Escape is handled by the nearest active owner, one level per keypress. Preserve
these baseline policies:

1. `TextInput` cancels editing and restores its entry snapshot.
2. `TextArea` finishes and commits its current document; Escape does not roll
   it back.
3. `Completion` closes before the editor receives a later Escape.
4. A searchable `Picker` clears a non-empty query first; a later Escape
   dismisses it.
5. `Select` closes and restores its highlight to the committed choice.
6. The top modal or cancelable layer closes; it cannot skip a surviving parent
   layer or deliver the same Escape to content below.
7. Only after local edit/layer state is clear does the containing application
   handle its navigation or running-work policy. Showcase and TablePro
   sequences are preserved in their [application contracts](../applications/README.md).

### Shared editing keys and host chords

Where enabled by the component, the baseline text map uses `Ctrl+A/E` for line
start/end, `Ctrl`/`Alt+Left/Right` and `Alt+B/F` for word motion,
`Shift+arrows` for selection, `Ctrl+U/K` to delete to line start/end,
`Ctrl+W` to delete a word, `Ctrl+L` for select all, and `Ctrl+Home/End` for
document start/end. A host-owned chord takes precedence in its application;
the text component uses the host's documented alternative. TablePro's query
editor uses `Ctrl+Backspace`/`Alt+Backspace` for word deletion and `Ctrl+A`
for select all. Single-line input discards pasted line breaks; multiline input
normalizes CRLF and CR to LF. Paste reaches only the active editing owner.

Application chords cannot steal typing-conflicting `q`, `z`, `Ctrl+L`,
`Ctrl+N`, or `Ctrl+D` from an active editor. Contextual verbs remain
consistent (`s` sort, `f` filter, `p` preview, `x` close, `u` undo, `y` copy,
`*`/`-` expand/collapse); the host binds an available alternate when a key is
already used.

## Editing versus navigation

Editing and navigation are distinct observable modes. In navigation, a focused
field shows its gutter and responds to traversal; Enter/F2 or a click enters
editing. In editing, the shared text engine owns grapheme motion, word motion,
selection, deletion, paste, undo/history where offered, and source/display
projection. Positions are valid grapheme boundaries, wide cells are not split,
and copy returns source text according to the component's copy policy.

The baseline intentionally preserves different policies:

| Component | Enter | Escape | Paste before editing |
| --- | --- | --- | --- |
| `TextInput` | commit | cancel and restore the entry snapshot | starts editing and inserts |
| `TextArea` | insert newline while editing; commit when the component's submit path requests it | commit/finish the document | ignored until editing is already active |

These differences are part of parity. A shared `TextEditorCore` may own text
operations, but it must not erase the component-level commit/cancel/paste
policy. Losing focus commits according to the component contract. Invalid
submission retains focus, value, and an error message; it does not silently
move to another field. Form submission focuses the first invalid visible field.

Read-only text may navigate, scroll, select, copy, and place a caret where
advertised; it never opens a mutation draft. A disabled field consumes or
ignores input according to the host route but cannot focus, hover, edit, or
activate. Secret fields retain their redaction and lifecycle rules; secrets
must not appear in action traces, debug output, or visual artifacts.

## Controlled choices and source reconciliation

Checkbox, Toggle, RadioGroup, Select, Tabs, Lists, Trees, Pickers, Grid, and
similar collections separate:

- caller-controlled chosen/checked/committed value;
- component-owned cursor and draft/anchor;
- runtime-owned focus, hover, and capture;
- typed action requesting a change.

Dynamic data uses stable `Id`, `ItemKey`, `ColumnKey`, `FieldKey`, and
`Revision` identities. Labels and display indices are not identities. On reorder,
filter, insertion, refresh, or removal, update reconciles surviving keys and
invalidates stale derived ranges. If a cursor disappears, a documented nearest
eligible fallback may be selected for navigation; a pending press, activation,
acknowledgement, or edit draft must never retarget to that fallback. A stale
completion or response is rejected by revision before it can change the scene.

Controlled callers may accept or reject a requested value. The component does
not silently maintain a second authoritative copy. Source reordering must leave
the selected stable key selected and must not activate the new occupant of an
old row index.

## Scroll, selection, and pointer interaction

ScrollRegion is the shared owner of vertical/horizontal bounds, edge fades,
thumb capture, wheel routing, and tail/follow policy where applicable. Up/down
and wheel movement clamp at boundaries; a boundary no-op can be consumed
without a redraw. The wheel scrolls the topmost container under the pointer and
does not move keyboard focus. Scrollbar track click jumps; thumb drag preserves
its grab offset; release clears capture. Rebuild hit geometry after content
movement and re-resolve stationary-pointer hover.

The baseline wheel step is three visual rows. It changes viewport offset
without moving a list/tree/grid cursor, selected stable key, or text caret. The
next keyboard movement advances from that existing semantic position and only
then minimally scrolls to reveal it; drawing alone never pulls a cursor into
view. Grid horizontal-wheel input moves columns through the same bounded
scroll owner. The wheel targets the innermost registered region beneath the
pointer, including without keyboard focus; a modal/popover takes it when the
pointer is over that layer. A boundary event is consumed with no repaint.

Tail following is a capability of log/output recipes only. A log pauses when
manual scroll leaves the real tail and resumes when the tail is reached by
wheel, key, page, or thumb. Prose `End` is a jump and never enables follow.

Text selection, grid range selection, and picker/list cursors are distinct from
focus. A selection is not a click. A mouse drag selects only after the component
has established its anchor; a pointer release outside the captured content does
not activate an unrelated row. Protected cursor/selection rows remain visible
through edge fading.

## Layers and modal ownership

The runtime owns one stack for dialogs, menus, context menus, select popups,
completion, pickers, and help. Each layer declares its owner, kind, placement,
dismissal, backdrop, pointer barrier, and focus policy. Components measure their
content; the runtime resolves the rectangle against the current viewport and
anchor.

### Modal

Opening a modal:

1. save the current surviving focus identity;
2. clear hover and press capture;
3. push both focus and hit barriers;
4. choose the documented initial focus;
5. route all input and paste to the modal until it closes.

The baseline initial target depends on the dialog recipe: routine confirmation
starts on the primary action, destructive confirmation starts on Cancel, and
prompt/typed acknowledgement starts in the field. Opening clears hover and
press so an old pointer state cannot show through.

Closing a modal restores the saved focus if it still exists and is reachable;
otherwise it chooses the first valid stop. Escape closes only the top applicable
level. A cancelable outside click is one dismissal event. A disabled underlying
control cannot receive click-through activation.

### Anchored popup

Menus, completion, select, and anchored pickers claim the pointer barrier and
are drawn above their siblings. They may keep keyboard focus with the owner so
typing continues, but their own key policy decides which events they consume.
They reanchor on resize and close when the owner disappears. Nested popups
close one level at a time; an outside click or Escape cannot skip a surviving
parent layer.

Focus restoration and layer ownership are semantic observations in parity tests,
not inferred from pixels alone.

## Readiness and status transitions

The component contract owns applicable state transitions; the runtime routes
them without inventing product behavior:

- empty content exposes the documented fill action or remains inert;
- loading shows the documented spinner and rejects duplicate activation;
- partial data keeps its loaded/total identity and fetch-more action;
- errors retain a recoverable value/focus and expose a typed retry action when
  supplied;
- success emits one action/result and shows the owning status treatment;
- disabled blocks all live interaction;
- read-only permits only the documented inspection interactions.

Busy, invalid, and error states cannot make a stale pointer target eligible.
Changing readiness while a pointer is held revalidates the target at release.

## Resize, geometry, and lifecycle edges

Layout and hit registration use the same pure measured rectangles. Test zero and
tiny allocations, nonzero origins, exact fit, one-cell-short boundaries, wide
Unicode content, scrollbar appearance/disappearance, and narrow-to-wide
recovery. Below a composed application's 72×20 baseline threshold, the host
renders its too-small state and does not dispatch hidden application controls;
individual components have no global 72×20 requirement.

Resize during a drag invalidates old geometry. The runtime must release a
disappeared capture target, revalidate an anchor, and preserve surviving state,
draft, selection, and scroll according to the component contract. Terminal
session cleanup is an optional edge adapter; component code does not spawn
processes or own PTYs.

## Required interaction proof

Every applicable component state is verified through both semantic interaction
traces and rendered frames. At minimum cover:

- normal, focus, hover, focus+hover, keyboard-suppressed hover, and restored hover;
- pointer down, held press, release inside, release outside, removed/disabled target;
- keyboard activation and exactly one 140 ms feedback interval;
- editing/navigation, selection, valid/invalid, read-only, commit/cancel/blur/tab;
- empty/loading/partial/ready/error/retry/success where supplied;
- top/middle/bottom scroll, wheel boundary, thumb drag, edge fade, and tail/read mode;
- nested layers, outside dismissal, Escape ladder, reanchor, owner removal, and focus restoration;
- resize, source reorder/removal/filter, revision conflicts, and stale responses;
- every unique motion phase and 0/139/140/141 ms deadline boundary.

Do not fabricate inapplicable states for decorative components. Record
non-applicability and its reason in the component capture plan. Compare exact
action count, target identity, focus owner, capture owner, active layer path,
selected key, navigation key, draft/committed values, cursor, and dimensions in
addition to cells and supported style attributes. The full procedure is in
[interaction parity](../verification/interaction-parity.md), [visual parity](../verification/visual-parity.md),
and [conformance](../verification/conformance.md).

## Boundary of this contract

Termrock components emit generic typed actions and preserve the four frozen
application consumers. They do not implement Jackin/Holla/TablePro product
services, accounts, databases, Docker, real Git operations, shells, daemons,
agent sessions, terminal emulation, PTY ownership, or escape parsing. The
`TerminalView` boundary consumes caller-provided prepared cells and emits typed
interaction/copy/link requests; those external terminal concerns stay outside
the library.
