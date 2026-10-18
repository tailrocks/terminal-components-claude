# Menu

Status: proposed target.
Owner: termrock-overlays.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W23 · Group: Overlays · Phase: P4 · Legacy family: C32 (`menu-context-menubar`).
Capture plan: [`../reference/capture-plans/menu.json`](../reference/capture-plans/menu.json).
Current source: [`src/widgets/menu.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/menu.rs).

Canonical Termrock contract for W23. `Menu` is the one command-list engine
used by [`ContextMenu`](./context-menu.md) and [`MenuBar`](./menu-bar.md).
This page describes the future API on `termrock-implementation`; current source and
the four baseline applications remain unchanged in this documentation phase.

## Purpose and exclusions

`Menu` renders a keyed command list with separators, shortcuts, optional
submenus and live action eligibility. It owns menu-local cursor, submenu path
and overflow scroll state. The caller supplies labels, action metadata,
enabled/disabled state, stable keys and any child menu data. Runtime owns
focus, hover, pointer capture, layer placement, outside dismissal and the
active key scope.

Menus report intent. They never execute commands, navigate application routes,
mutate domain data, or build a second keybinding catalog.

Exclusions:

- a universal boxed `Widget` trait or a `show()` method that mixes update and
  painting;
- application-specific File/Go/Help vocabulary or command side effects;
- a second engine for submenus, context menus or menu bars;
- treating disabled rows as absent or allowing events to click through them;
- redesigning the frozen Showcase, TablePro, Jackin Preview or Holla chrome.

## Public API

These declarations are the target shape. The exact spelling is frozen only
after P1 external-consumer probes.

```rust
Menu::new(
    id: Id,
    items: &'a [MenuItem<'a>],
    revision: Revision,
) -> Menu<'a>

Menu::patch(self, patch: StylePatch) -> Self
Menu::patch_part(self, part: MenuPart, patch: StylePatch) -> Self

Menu::update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut MenuState,
) -> Response<MenuAction>

Menu::draw(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &MenuState,
) -> Rect

Menu::measure(
    &self,
    cx: &MeasureCx<'_>,
    constraints: Constraints,
) -> Size
```

`MenuItem` is a borrowed prop with an `ItemKey`, label, effective `Binding`,
enabled/disabled or read-only status, danger tone, separator metadata and an
optional keyed submenu reference. A submenu is data for this engine, not a new
dispatcher.

`MenuState` owns the highlighted item key, open submenu path and scroll offset.
It does not own command rows, domain values, geometry, focus handles or
application effects. `MenuAction` is typed:

```rust
enum MenuAction {
    Invoke { action: ActionKey, origin: ActivationOrigin },
    Dismissed { reason: DismissReason },
}
```

`ActionKey` and `ItemKey` are distinct. A display label or positional index is
never an action identity.

The current module uses legacy `MenuItem`, `ContextMenu`, `MenuBar`,
`MenuEvent`, `MenuBarEvent`, `WidgetId`, `Outcome` and `RenderCtx` types. Those
names and positional indices are source evidence only. The target is one
Termrock menu mechanism with thin surface wrappers and typed stable actions.

## Ordinary use

Consumer recipe: [EX-09 — Menus, context menus, and a command palette](../api/consumer-recipes.md) (`proposed_target`).

Baseline consumers include:

- Showcase `src/bin/showcase/pages/chrome.rs`, with File/View/Help menus and a
  row context menu containing disabled, separator and danger entries;
- Jackin Preview host and capsule menus in `src/bin/jackin_preview/app.rs` and
  `screens/capsule.rs`, including F10, prefix and dynamic route actions;
- corresponding menu states in the frozen Showcase and Jackin snapshot trees;
- baseline drivers [`tests/visual_baseline/showcase.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/tests/visual_baseline/showcase.rs)
  and [`tests/visual_baseline/jackin.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/tests/visual_baseline/jackin.rs).

## Ownership

- `update` consumes one normalized event/cause, reconciles borrowed item props
  at the declared `Revision`, advances the keyed cursor/path, and returns at
  most one `MenuAction`.
- `draw` is read-only with respect to `MenuState` and source data. It paints
  rows, registers hit regions, and reports geometry, but does not move the
  cursor from hover, open a submenu, or dismiss the layer as a side effect.
- `measure` is pure. It computes display-cell width for labels, shortcuts,
  title/separator rows and submenu affordances under constraints, including
  zero/tiny allocations and edge clamping.
- Runtime owns focus/capture, hover, active layer, outside-click routing and
  feedback time. A menu row is a command target; it is not a separate focus
  scope with a decorative focus gutter.
- Menu, ContextMenu and MenuBar share this state machine. Their wrappers supply
  anchors and chrome composition; they must not fork navigation, eligibility,
  submenu or scroll behavior.

Shared dependencies:

- [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md),
  [`input-actions`](../foundations/input-actions.md),
  [`identity`](../foundations/identity.md), [`layout`](../foundations/layout.md),
  [`theme`](../foundations/theme.md),
  [`collections`](../foundations/collections.md),
  [`author`](../foundations/author.md),
  [`conformance`](../foundations/conformance.md);
- [`scroll-region`](./scroll-region.md) and [`key-hint`](./key-hint.md).

## Customization

Advertised parts are `container`, `row`, `marker`, `label`, `shortcut`,
`submenu`, `separator` and `status`. Part patches follow the shared theme
precedence and are confined to the reserved cells. They cannot change hit
geometry, layer ownership, focus/capture or disabled barriers.

## Behavior

### Focus and keyboard

- Opening a menu establishes a runtime layer and makes its keyed cursor the
  active command position. The menu does not consume editor text intended for
  a higher modal layer.
- Up/Down and `j`/`k` skip disabled and separator entries and wrap among
  eligible item keys. Home/`g` and End/`G` select the first/last eligible key.
- Enter and Space invoke the selected enabled action. An effective shortcut
  resolves through the shared binding catalog; remapping changes both handler
  and displayed chord. Unassigned modified chords never fall through to a
  plain action.
- Left/Right and `h`/`l` close/open submenu paths as appropriate. Escape
  closes the deepest submenu first; outer layer dismissal follows runtime
  policy. Closing the final menu emits one dismissal and restores the owner.

### Pointer, hover and activation

- A row activates only on a matching completed press/release. Release outside,
  target removal, or disabling during a held press cancels; there is no
  click-through to the layer below.
- Pointer hover may move the keyed cursor only through the runtime's current
  hover observation. Keyboard input suppresses stale hover until pointer
  motion restores it.
- Moving from a parent row into its submenu retains the open path and pointer
  capture; moving back does not accidentally dismiss the parent. An outside
  click is consumed once by the top menu layer.
- Disabled rows are visible barriers. They cannot focus, hover, invoke or pass
  a click to an underlying control. Read-only command metadata may remain
  inspectable while its action is absent or disabled.

### Layers, resize and reconciliation

- A menu is a runtime-owned popover. Placement is derived from its anchor and
  measured size, clamped or flipped at terminal edges. Resize reanchors the
  current path without changing item identity.
- Overflow uses the shared `ScrollRegion` mechanism, including wheel boundary,
  thumb capture, protected selected row and edge fade where content requires it.
- At `Revision` change, reconcile by `ItemKey`. Retain a surviving current key;
  if it is removed, choose the documented nearest eligible successor then
  predecessor for navigation, but never transfer a pending activation to that
  fallback row. A removed submenu owner closes its descendant path.

## Visual matrix

The frozen source paints a rounded popover on the strongest neutral plane. The
cursor row receives a solid highlight; hover lifts an unselected row; danger
rows use a soft error tone at rest and a danger highlight under the cursor;
disabled rows are faint and inert; shortcuts are right-aligned when they fit;
separators occupy their own row. Keyboard navigation skips disabled rows and
wraps among eligible entries. The current source's `ContextMenu` and `MenuBar`
share these painting and input rules; the target preserves them through one
engine.

The Termrock default recipe must preserve the oracle's output:

- popover surface, rounded border, title/metadata rows where supplied, row
  padding and clipping;
- one solid selection highlight for the keyed cursor, with no extra focus
  gutter; hover lifts only an unselected eligible row;
- danger rows retain their error-soft rest tone and use the baseline danger
  highlight when selected;
- disabled rows are faint, non-hovering and non-activating; separators keep
  their border glyph and do not become command targets;
- effective shortcuts come from the shared binding view and are right-aligned
  only when the measured row can fit them;
- nested submenus flip at right/bottom edges while preserving the selected
  path and surface ladder;
- labels, Unicode width, wide-cell continuation, foreground/background and
  supported modifiers are exact across truecolor, 256, 16, none and `NO_COLOR`;
- pointer activation uses the shared 140 ms feedback where the baseline
  exposes it. Menus have no invented animation or time-owned state.

| Axis | Required cases | Applicability |
|---|---|---|
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide | menu surface and each submenu |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer-motion restoration | eligible rows; separator/decorative rows are N/A |
| Activation | pointer down, held press, release inside/outside, removed/disabled target, keyboard activation, feedback active/expired | actionable rows only |
| Entries | enabled, disabled, danger, separator, shortcut fit/clip, empty and dynamic eligibility | supplied item variants |
| Submenu/layer | closed, open, nested, right/bottom flip, parent-to-child hover, Escape one level, outside, owner removed, focus restore | when a submenu/anchor exists |
| Scroll | no overflow, start/middle/end, wheel boundary, thumb drag, resize while scrolled, edge fade, protected row | long menus only |
| Data | reorder, insert, remove, filter and stale source revision | dynamic item sources |
| Capability/motion | truecolor, 256, 16, none, NO_COLOR; paused/reduced and feedback expiry | every visual variant that uses it |

Only instantiate states supported by the supplied item data. Do not invent
editing or modal acknowledgement states for a command list.

## Verification

The baseline has no requirement that every existing menu render a submenu.
The target engine must nevertheless cover submenu placement and pointer
handoff in Behavior; such new robustness or API evidence is classified
as `Extension` until an unchanged oracle state is bound.

Use the five plan dimensions `72x20`, `80x24`, `100x30`, `120x40`, `160x50`
and capabilities `truecolor`, `256`, `16`, `none`, `nocolor`. Bind expected
artifacts to `ExistingOracle` or `ExtractedOracle` before candidate comparison;
new submenu/robustness behavior is an explicit `Extension`.

| Case | Required observation |
|---|---|
| W23-01 | Disabled and separator entries remain visible barriers between actionable rows. |
| W23-02 | Three-level submenu opens and flips at right/bottom edges without losing the keyed path. |
| W23-03 | Pointer movement parent → submenu → parent does not close or retarget the menu. |
| W23-04 | A remapped shortcut invokes the expected `ActionKey` and displays the same effective binding. |
| W23-05 | Outside click is consumed once and nested Escape closes one level at a time. |

Every trace records exact cells, wide continuations, colors/modifiers supported
by the capture tool, cursor, focus/capture/layer owners, selected stable key,
navigation key and typed action count/target. Missing oracle setup is blocked,
never a pass.

Required negative tests include:

- draw twice cannot move the cursor, open/close a submenu or change source data;
- duplicate IDs/keys and duplicate activation are rejected before dispatch;
- disabled/separator/outside targets cannot invoke or click through;
- stale keys after reorder/removal cannot activate the new occupant of an old
  index;
- keyboard-suppressed hover, release-outside and removed-target traces match
  the declared response without a phantom action;
- submenu edge placement and zero/tiny geometry cannot register out-of-bounds
  hits or panic on Unicode width;
- a supported part patch changes its advertised cells and is not overwritten;
  a dead menu call followed by custom paint fails the ownership mutation gate;
- one cell, cursor, focus owner, capture owner, layer path or action-target
  difference fails exact comparison.

## Rejected use

Forbidden: preview renders a padded command list with its own cursor instead
of using the shared `Menu` engine.

```rust
// Forbidden: preview owns menu rows and the cursor index.
ui.paint_str(row, 2, "> Open file        Ctrl+O  ");
ui.paint_str(row + 1, 2, "  Save             Ctrl+S  ");
if pressed_down { cursor = (cursor + 1) % items.len(); }
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

Use `Menu::new` with keyed items and handle the typed `MenuAction`.

## Known gaps

- [FIX-002](../implementation/code-remediation-backlog.md) (pending): replace
  historical Manager, menu, inspect, help, and quit screens; the menu slice
  must use the real `Menu` engine with stable keys.
- [FIX-007](../implementation/code-remediation-backlog.md) (pending): resolve
  public signature drift against this contract; compile external examples and
  check the accepted state/action model.
- Capture cases W23-01–W23-05 await expected-artifact binding before candidate
  comparison; new submenu/robustness behavior is an explicit `Extension`.
