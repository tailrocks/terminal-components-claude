# ContextMenu

Status: proposed target.
Owner: termrock-overlays.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W24 · Group: Overlays · Phase: P4 · Legacy family: C32 (`menu-context-menubar`).
Capture plan: [`../reference/capture-plans/context-menu.json`](../reference/capture-plans/context-menu.json).
Current source: [`src/widgets/menu.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/menu.rs).

Canonical Termrock contract for W24. `ContextMenu` is a thin anchored
composition over the shared [`Menu`](./menu.md) engine; it does not create a
second item interaction mechanism. This page describes the future API on
`termrock-implementation`; the current applications remain unchanged.

## Purpose and exclusions

`ContextMenu` presents a command list at a pointer or keyboard anchor. The
caller supplies the stable target `ItemKey`, current action data and menu
revision. Runtime owns the popover layer, pointer barrier, focus/capture,
outside dismissal and placement. The menu reports an action against the
captured target; it never recomputes that target from the current visual row
index at invocation time.

The surface supports the same disabled, separator, danger, shortcut and
submenu behavior as [`Menu`](./menu.md). It may add a title or anchor metadata
for the caller, but it does not fork row painting or navigation.

Exclusions:

- executing a context command, mutating caller data or deciding what a target
  means;
- a global popup ID shared by unrelated owners;
- retargeting to the currently selected row when the captured row is removed or
  reordered;
- replacing modal layers or allowing an obscured context menu to activate;
- changing product behavior or visual design in Showcase, TablePro, Jackin
  Preview or Holla.

## Public API

Rustdoc-style target declarations; exact spelling is frozen by the P1
external-consumer probes.

```rust
ContextMenu::new(
    id: Id,
    target: ItemKey,
    items: &'a [MenuItem<'a>],
    revision: Revision,
) -> ContextMenu<'a>

ContextMenu::anchor(self, anchor: Anchor) -> Self
ContextMenu::patch(self, patch: StylePatch) -> Self

ContextMenu::update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut MenuState,
) -> Response<MenuAction>

ContextMenu::draw(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &MenuState,
) -> Rect

ContextMenu::measure(
    &self,
    cx: &MeasureCx<'_>,
    constraints: Constraints,
) -> Size
```

`target`, `items`, anchor and `revision` are borrowed/current props. `MenuState`
is shared with Menu and stores only highlighted key, submenu path and scroll.
There is no separate context-menu cursor or item engine. The typed action is
the shared `MenuAction`:

```rust
enum MenuAction {
    Invoke { action: ActionKey, origin: ActivationOrigin },
    Dismissed { reason: DismissReason },
}
```

The owner must associate the emitted `ActionKey` with the captured `ItemKey`.
If the target's revision makes it unavailable, the action is dismissed or
disabled according to the declared policy; it is never transferred to an old
display index.

The current source calls the legacy surface `ContextMenu` and stores a
positional `cursor`, `Rect` anchor and `WidgetId`. Those fields document the
frozen starting behavior only. The Termrock target uses stable item identity,
borrowed props, shared `MenuState`, runtime-owned placement and typed actions.

## Ordinary use

Consumer recipe: [EX-09 — Menus, context menus, and a command palette](../api/consumer-recipes.md) (`proposed_target`).

The frozen source uses ContextMenu in Showcase Chrome. A secondary click on a
session row selects that row, captures its anchor and opens a rounded popover
with a title and Change title, Move left/right, and danger Close commands.
Keyboard `m` opens the selected row's menu; disabled edge moves stay visible
and inert. Jackin Preview's capsule menu is another preserved context-like
composition, opened by the prefix command and anchored to the active tab.

Read-only evidence includes:

- [`src/widgets/menu.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/menu.rs), including
  `anchor`, pointer-position placement, title, row IDs, hover cursor movement,
  disabled barriers and edge flip/clamp;
- Showcase [`src/bin/showcase/pages/chrome.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/showcase/pages/chrome.rs)
  and its context-menu tests;
- Jackin Preview capsule source and
  [`src/bin/jackin_preview/app_tests_chrome.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/app_tests_chrome.rs);
- frozen snapshot roots under `snapshots/showcase/pages/chrome` and
  `snapshots/jackin/capsule`, plus the baseline drivers for Showcase and
  Jackin.

## Ownership

- `update` receives pointer/keyboard context intent through `Cx`, reconciles
  target and items at `Revision`, and returns at most one typed action. The
  runtime resolves the anchor and layer before dispatch.
- `draw` is read-only. It paints the shared menu and registers only the current
  surface/rows. It does not select a row because it is hovered, capture a
  pointer, retarget the target, or dismiss the layer.
- `measure` is pure display-cell measurement. It accounts for title, labels,
  shortcuts, separators, nested affordances and available edge space.
- Runtime owns focus, hover, capture, outside clicks and layer order. The
  wrapper owns only the semantic captured target and anchor prop.
- Menu, ContextMenu and MenuBar use the same identity, eligibility, submenu and
  scroll contracts; only anchoring and chrome differ.

Shared dependencies:

- [`menu`](./menu.md), [`runtime`](../foundations/runtime.md),
  [`layers`](../foundations/layers.md),
  [`input-actions`](../foundations/input-actions.md),
  [`identity`](../foundations/identity.md), [`layout`](../foundations/layout.md),
  [`theme`](../foundations/theme.md),
  [`collections`](../foundations/collections.md),
  [`author`](../foundations/author.md),
  [`conformance`](../foundations/conformance.md),
  [`scroll-region`](./scroll-region.md).

## Customization

Advertised parts are `container`, `row`, `marker`, `label`, `shortcut`,
`submenu` and `separator`. A supported part patch changes only its reserved
cells under the shared [`theme`](../foundations/theme.md) precedence.

## Behavior

### Focus and keyboard

- A keyboard or pointer context action opens the menu with the target captured
  as an `ItemKey`. Focus and layer ownership are registered with runtime; the
  underlying list does not keep receiving menu keys.
- Up/Down and `j`/`k` navigate shared keyed menu state, skipping disabled and
  separator rows. Home/`g` and End/`G` select the first/last eligible key.
- Enter/Space or an effective shortcut invokes the current enabled action with
  the captured target. Unassigned modified chords never become plain actions.
- Escape closes nested submenu levels first, then dismisses the context layer.
  Closing restores the prior owner focus. A modal layer above the context menu
  always wins input and prevents activation underneath it.

### Pointer, hover and activation

- Right-click/secondary-click opens at the numeric pointer position after the
  trusted oracle resolves the hit. The candidate cannot choose its own hitbox
  and call that the same sealed gesture.
- A row activates only on matching press/release. Release outside, target
  removal, or disabling during the held press cancels. An outside click is
  consumed exactly once and cannot fall through.
- Hover moves the shared cursor only while pointer hover is current. Keyboard
  input suppresses stale hover until pointer motion restores it. Disabled rows
  neither hover nor activate.
- Read-only target metadata may be shown in the title; it is not editable and
  does not grant a command action. A disabled context command remains a visible
  barrier.

### Resize and reconciliation

- `measure` and the shared layout foundation place the popover below, above or
  beside its anchor, clamping or flipping at all edges. Resize reanchors from
  the same semantic anchor and preserves the target/key path.
- Long menus use shared `ScrollRegion` behavior: top/middle/end, wheel
  boundaries, thumb capture, edge fade and selected-row protection where
  applicable.
- A target/item revision reconciles by stable key. If the target disappears or
  is no longer eligible, dismiss or disable it by policy. Never use the new
  occupant of the target's former index, even if its label matches.

## Visual matrix

Preserve the baseline's rounded popover surface, title text, row spacing,
selected highlight, hover lift, danger tone, disabled styling, separator
glyph, shortcut alignment and clipping. The pointer anchor may be a one-cell
position or a caller rectangle; placement flips/clamps at terminal boundaries
without changing the rendered menu recipe. Unicode labels and shortcuts use
display-cell width, preserving wide-cell continuation.

Color capability lanes are truecolor, 256, 16, explicit `none` and `NO_COLOR`.
No component painter hardcodes RGB values. Context menus have no independent
animation; child activation feedback uses the shared 140 ms clock where the
baseline supplies it. Reduced/paused motion must not change action semantics.

| Axis | Required cases | Applicability |
|---|---|---|
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide | popover and anchor |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer-motion restoration | eligible rows only |
| Activation | pointer down, held press, release inside/outside, removed/disabled target, keyboard activation, feedback active/expired | actual command rows |
| Target/source | stable target, nonselected target, reorder, insertion, removal, filtered/stale revision | caller keyed source |
| Layer | closed/open, nested, outside, Escape ladder, owner removed, resize/reanchor, focus restore, modal above | runtime layer stack |
| Scroll | no overflow, start/middle/end, wheel boundary, thumb drag, resize while scrolled, edge fade, protected row | long menu only |
| Entries | enabled, disabled, danger, separator, shortcut fit/clip, title/empty | supplied variants |
| Capability/motion | truecolor, 256, 16, none, NO_COLOR; reduced/paused and feedback expiry | every applicable variant |

Editing and modal acknowledgement states are not applicable to ContextMenu
itself. If a menu action opens an editor, that child owns its editing states.

## Verification

The frozen baseline is the only visual authority. New stable-target and
reorder safety behavior is an `Extension` until an unchanged source state is
captured and approved.

Use the plan sizes `72x20`, `80x24`, `100x30`, `120x40`, `160x50` and all five
capability lanes. Classify expected output as `ExistingOracle`, `ExtractedOracle`
or `Extension` before candidate comparison; expected artifacts are read-only
to candidate code.

| Case | Required observation |
|---|---|
| W24-01 | Open on a nonselected row; invocation remains bound to that stable target, not the selected row. |
| W24-02 | Remove/reorder the target while open; no stale index retarget or duplicate action occurs. |
| W24-03 | Keyboard and pointer anchors near every terminal border clamp/flip and preserve cells/target. |
| W24-04 | A context menu beneath a modal cannot receive keys, clicks or action dispatch. |

Trace exact cells, wide continuations, foreground/background/modifiers,
cursor, focus owner, capture owner, layer path, target `ItemKey`, selected
menu key, navigation key and action count/target. Invalid oracle setup is
blocked, never a pass.

Required negative tests include:

- the captured target cannot change because a row moved, disappeared or reused
  its old index;
- modal-first routing blocks a lower ContextMenu, including pointer, keyboard,
  wheel and outside-click leakage;
- release outside, disabled target and removed target produce no invocation;
- draw twice does not mutate cursor, target, scroll, layer or source data;
- duplicate identity, out-of-bounds anchor/hit registration and zero/tiny
  geometry fail closed;
- Unicode labels never split a grapheme or continuation cell;
- part patches affect their advertised cells and cannot replace the runtime
  barrier; dead-call/custom-paint mutation is rejected;
- one cell, cursor, focus/capture owner, target key or action trace difference
  fails exact comparison.

## Rejected use

Forbidden: preview draws an anchored option box with padded rows and resolves
the target from the visual row index instead of using `ContextMenu`.

```rust
// Forbidden: preview owns the popup and retargets by row position.
ui.paint_str(popup_row, x, "  Change title   ");
ui.paint_str(popup_row + 1, x, "  Close          ");
model.invoke(selected_row_index);
```

Rule: [ARC-012](../architecture/component-composition.md). Preview code must
not recreate component rendering or generic interaction. The following
patterns are prohibited in preview drawing paths:

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

Use `ContextMenu::new` with a captured `ItemKey` target and handle the shared
typed `MenuAction`.

## Known gaps

- [FIX-002](../implementation/code-remediation-backlog.md) (pending): replace
  historical Manager, menu, inspect, help, and quit screens; the context-menu
  slice must use the real `ContextMenu` over the shared `Menu` engine.
- [FIX-007](../implementation/code-remediation-backlog.md) (pending): resolve
  public signature drift against this contract; compile external examples and
  check the accepted state/action model.
- Capture cases W24-01–W24-04 await expected-artifact binding before candidate
  comparison; new stable-target and reorder safety behavior is an `Extension`.
