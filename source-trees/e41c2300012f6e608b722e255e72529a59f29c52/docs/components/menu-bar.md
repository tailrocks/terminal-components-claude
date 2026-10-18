# MenuBar

Status: proposed target.
Owner: termrock-overlays.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W25 · Group: Chrome · Phase: P4 · Legacy family: C32 (`menu-context-menubar`).
Capture plan: [`../reference/capture-plans/menu-bar.json`](../reference/capture-plans/menu-bar.json).
Current source: [`src/widgets/menu.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/menu.rs).

Canonical Termrock contract for W25. `MenuBar` is the one-row chrome wrapper
around the shared [`Menu`](./menu.md) engine. This is a target contract for the
in-place refactor on `termrock-implementation`; it does not rename current source or
redesign any frozen application.

## Purpose and exclusions

`MenuBar` renders caller supplied top-level menu labels, an optional leading
brand lockup and optional trailing metadata. It delegates dropdown navigation,
shortcuts, separators and action eligibility to the shared Menu engine. The
application supplies menu vocabulary, bindings and metadata; the library owns
measurement, one-row layout, focus/hover state, anchors and composition.

It does not prescribe File/Go/Help names, execute menu effects, own routes,
or become a product shell.

Exclusions:

- a second command dispatcher or menu-row implementation;
- assuming a fixed brand string or application identity;
- allowing trailing metadata to overlap menu hitboxes;
- hiding disabled actions or changing application routing semantics;
- redesigning Showcase, TablePro, Jackin Preview or Holla.

## Public API

These Rustdoc-style signatures describe the target shape; external-consumer
probes in P1 freeze exact spelling.

```rust
MenuBar::new(
    id: Id,
    menus: &'a [TopMenu<'a>],
    revision: Revision,
) -> MenuBar<'a>

MenuBar::leading(self, brand: Option<Brand<'a>>) -> Self
MenuBar::trailing(self, metadata: StyledText<'a>) -> Self
MenuBar::patch(self, patch: StylePatch) -> Self

MenuBar::update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut MenuBarState,
) -> Response<MenuAction>

MenuBar::draw(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &MenuBarState,
) -> Rect

MenuBar::measure(
    &self,
    cx: &MeasureCx<'_>,
    constraints: Constraints,
) -> Size
```

`menus`, brand, metadata and effective bindings are borrowed props. The caller
advances `Revision` when menu eligibility or labels change. `MenuBarState` owns
the selected top-menu key and the shared `MenuState` for an open dropdown path;
it does not own app routes, labels, geometry or command effects.

The returned typed action is the shared `MenuAction` from [`Menu`](./menu.md):

```rust
enum MenuAction {
    Invoke { action: ActionKey, origin: ActivationOrigin },
    Dismissed { reason: DismissReason },
}
```

When the caller makes the brand clickable, it supplies an `ActionKey` and the
bar reports it through `MenuAction::Invoke`; the library never hardcodes a
product route.

The current implementation is the legacy `MenuBar` with `MenuBarEvent`,
`WidgetId`, `Outcome`, `RenderCtx` and `Lockup`. Those identifiers remain only
as source/oracle evidence until the implementation phase.

## Ordinary use

Consumer recipe: [EX-09 — Menus, context menus, and a command palette](../api/consumer-recipes.md) (`proposed_target`).

Unchanged evidence includes:

- Showcase [`src/bin/showcase/pages/chrome.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/showcase/pages/chrome.rs),
  with File/View/Help menus, `app❯` brand, status metadata and context menu;
- Jackin Preview host/capsule `MenuBar` compositions in
  [`src/bin/jackin_preview/app.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/app.rs)
  and `screens/capsule.rs`, including `jackin❯` as literal baseline text;
- visual baseline drivers and snapshot roots under Showcase Chrome and Jackin
  manager/capsule;
- current interaction tests for F10, left/right switching, pointer switching,
  dynamic eligibility and brand activation.

## Ownership

- `update` handles one normalized key/pointer/cause. It changes the selected
  top-menu key or delegates item behavior to shared Menu state, returning at
  most one typed action.
- `draw` is read-only. It lays out the one-row bar, paints menu labels/brand/
  metadata, registers label hit regions, and draws the shared dropdown after
  the bar. It never opens a menu solely because a label is hovered or mutates
  selection during rendering.
- `measure` is pure and uses one geometry calculation for paint and hits. It
  reserves brand, labels and trailing metadata without overlap and handles
  zero/tiny allocations.
- Runtime owns focus, hover, pointer capture, layer order and feedback clock.
  MenuBar owns only its selected top-menu key and shared child state.
- MenuBar, Menu and ContextMenu must resolve item identity, bindings,
  disabled barriers, submenu paths and dismissal through the shared engine.

Shared dependencies:

- [`menu`](./menu.md), [`brand`](./brand.md),
  [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md),
  [`input-actions`](../foundations/input-actions.md),
  [`identity`](../foundations/identity.md),
  [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md),
  [`collections`](../foundations/collections.md),
  [`author`](../foundations/author.md),
  [`conformance`](../foundations/conformance.md).

## Customization

Advertised parts are `container`, `brand`, `title`, `menu-label`, `dropdown`
and `metadata`. Theme patches affect only declared rectangles. A brand slot
cannot install a new focus router; metadata cannot claim a menu hitbox.

## Behavior

### Focus and keyboard

- When closed and focused, Left/Right (and the bound equivalents) move the
  selected top-menu key. Tab traversal enters/leaves the bar according to the
  runtime focus ring.
- F10 or a registered accelerator opens the selected menu. Enter, Down or
  Space may open according to the effective binding policy. Menu navigation
  then delegates to shared Menu.
- While open, Left/Right switches top-menu titles and preserves the active
  MenuBar layer. Up/Down/Enter/Space/shortcuts use Menu. Escape closes the
  dropdown and restores the previous owner focus.
- Remapped/disabled menu actions are resolved from the effective binding view;
  labels and displayed chords must change with the same binding revision.

### Pointer, hover and activation

- Clicking a label opens it, or toggles it closed when already open. Hovering a
  different top label while a dropdown is open switches to that title without
  an accidental close. Pointer transitions into the dropdown retain the layer.
- Press/release completes a label or row action only when the target and
  release match. Release outside closes according to layer policy and cannot
  activate a page control underneath. Held press and 140 ms feedback remain
  observable where the baseline provides them.
- Disabled menu rows are visible barriers; separator rows are not targets.
  Hover suppression after keyboard input follows runtime policy.
- The brand, when clickable, emits its own typed intent. It is not treated as a
  menu command and has no library-owned product behavior.

### Resize and reconciliation

- One-row layout recomputes brand, labels and trailing metadata from measured
  display-cell widths on resize. No label hitbox may disagree with paint.
- Open dropdowns reanchor to the current label rectangle and use shared Menu
  edge clamping/flipping and scrolling. The selected top-menu key survives
  resize and available-space changes.
- At `Revision` change, reconcile labels and menu action data by stable keys.
  Removed labels close their child path; removed item actions do not transfer
  to a new index or label.

## Visual matrix

The frozen `src/widgets/menu.rs` paints a one-row canvas bar. The optional
`Lockup` is the only accent-filled control. Menu labels use secondary text,
hover lift, and a shared popover plane when open. A keyboard-focused bar shows
the focus gutter; an open label shares the popover surface with its dropdown.
The right-hand host context is measured after menu labels and must collapse
without changing left hit rectangles.

The default Termrock theme preserves:

- one row, exact label padding, brand spacing and baseline background plane;
- secondary labels, focus gutter for the keyboard-selected closed bar, hover
  lift, and open-label popover continuity;
- the optional brand lockup and literal application-visible marks (`app❯`,
  `jackin❯`, `holla❯`) exactly where the frozen consumers supply them;
- dropdown surface, border, row highlight, danger/disabled/separator styling
  through shared Menu recipes;
- trailing metadata truncation/collapse without painted and hit-test geometry
  disagreeing;
- Unicode and wide-cell measurement, truecolor/256/16/none/NO_COLOR role
  resolution, and the shared 140 ms activation feedback where applicable.

MenuBar has no independent animation. Reduced/paused motion changes only
declared feedback timing and never action identity or routing.

| Axis | Required cases | Applicability |
|---|---|---|
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide | one-row bar and open dropdown |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer-motion restoration | label, brand and menu child where interactive |
| Activation | pointer down, held press, release inside/outside, removed/disabled target, keyboard activation, feedback active/expired | clickable brand/labels/menu rows |
| Menu/layer | closed, open, nested, F10/accelerator, left/right switch, pointer switch, Escape, outside, owner removed, focus restore | runtime-owned dropdown |
| Metadata | full trailing context, narrow collapse, Unicode truncation and re-expansion | supplied trailing prop |
| Source | action eligibility change after open, insert/remove/reorder labels/items, stale key | revisioned caller source |
| Scroll | no overflow, start/middle/end, wheel boundary, thumb drag, resize while scrolled, edge fade | long dropdown only |
| Capability/motion | truecolor, 256, 16, none, NO_COLOR; normal/reduced/paused feedback | every applicable visual variant |

The bar itself has no text-editing or acknowledgement state. Such states belong
to the child component opened by a menu action.

## Verification

The visual target is exactly the pinned output. A paper theme or newly exposed
robustness behavior is an `Extension`, not baseline parity.

Use the plan's `72x20`, `80x24`, `100x30`, `120x40`, `160x50` dimensions and
`truecolor`, `256`, `16`, `none`, `nocolor` capabilities. Bind expected output
before candidate comparison as `ExistingOracle`, `ExtractedOracle` or explicit
`Extension`.

| Case | Required observation |
|---|---|
| W25-01 | F10 opens the bar; Left/Right switch titles; Escape closes and restores owner focus. |
| W25-02 | Pointer-open then pointer-hover/click switch to another top menu without losing the layer. |
| W25-03 | Narrow bar collapses trailing metadata while preserving label geometry and clickable brand. |
| W25-04 | Dynamic action eligibility changes after open and is reflected in paint, binding and dispatch. |

Trace exact cells, continuations, colors/modifiers, cursor, focus/capture/layer
owners, selected top-menu/item keys, navigation key, metadata visibility and
typed action count/target. Missing oracle setup is blocked, never a pass.

Required negative tests include:

- draw twice cannot open/switch menus or mutate selected keys;
- label paint and hit geometry remain identical under narrow/trailing metadata
  pressure;
- disabled/separator rows, release-outside and stale item keys cannot invoke;
- a removed label/menu cannot receive a pending action after reconciliation;
- brand activation cannot dispatch a menu command or product-side effect;
- keyboard-suppressed hover, pointer switch, nested Escape and focus restore
  produce the declared trace exactly;
- zero/tiny geometry and Unicode/truncation cannot panic or register outside
  cells;
- part patches affect their promised cells and dead-call/custom-paint mutation
  fails; one cell, cursor, focus/capture owner, layer path or action target
  difference fails exact comparison.

## Rejected use

Forbidden: preview paints its own top row of padded labels with a hand-tracked
open index instead of using `MenuBar`.

```rust
// Forbidden: preview owns the bar row and the open-menu index.
ui.paint_str(0, 0, "  File    View    Help  ");
if clicked_label { open_menu = Some(label_index); }
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

Use `MenuBar::new` with borrowed top menus and handle the shared typed
`MenuAction`.

## Known gaps

- [FIX-002](../implementation/code-remediation-backlog.md) (pending): replace
  historical Manager, menu, inspect, help, and quit screens; the menu-bar
  slice must use the real `MenuBar` over the shared `Menu` engine.
- [FIX-007](../implementation/code-remediation-backlog.md) (pending): resolve
  public signature drift against this contract; compile external examples and
  check the accepted state/action model.
- Capture cases W25-01–W25-04 await expected-artifact binding before candidate
  comparison; newly exposed robustness behavior is an `Extension`.
