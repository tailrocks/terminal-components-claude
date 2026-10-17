# HelpOverlay

Status: proposed target.
Owner: termrock-overlays.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W26 · Group: Overlays · Phase: P5 · Legacy family: C33 (`help-overlay`).
Capture plan: [`../reference/capture-plans/help-overlay.json`](../reference/capture-plans/help-overlay.json).
Current source: [`src/bin/jackin_preview/screens/modals.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/modals.rs).

Canonical Termrock contract for W26. `HelpOverlay` is a read-only, scrollable
modal composition over shared Dialog, TextViewport and KeyHint infrastructure.
It describes the target API on `termrock-implementation`; current app-local help
surfaces and frozen output remain unchanged.

## Purpose and exclusions

`HelpOverlay` explains the currently effective commands in a modal, scrollable
surface. The caller supplies explanatory sections and a resolved
`BindingView`; Termrock lays out columns, key hints, scrolling, fades,
scrollbar and modal chrome. It emits dismissal intent only. It does not
execute commands, infer application routes, own editor state or decide what a
binding does.

Descriptions may be explanatory text rather than runnable actions, but any
displayed chord must agree with the same effective binding resolution used by
menus, HintBar and handlers.

Exclusions:

- a second keybinding source, command registry or application help database;
- editable text, selection ownership, domain state or provider/network data;
- a generic whole-surface activation/pressed state;
- a product-specific route, shell or modal manager;
- redesigning the help appearance or content of Showcase, TablePro, Jackin
  Preview or Holla.

## Public API

Rustdoc-style target declarations; exact spelling is frozen by P1 external
consumer probes.

```rust
HelpOverlay::new(
    id: Id,
    sections: &'a [HelpSection<'a>],
) -> HelpOverlay<'a>

HelpOverlay::bindings(self, bindings: &'a BindingView) -> Self
HelpOverlay::patch(self, patch: StylePatch) -> Self

HelpOverlay::update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut HelpOverlayState,
) -> Response<HelpAction>

HelpOverlay::draw(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &HelpOverlayState,
) -> Rect

HelpOverlay::measure(
    &self,
    cx: &MeasureCx<'_>,
    constraints: Constraints,
) -> Size
```

`sections`, descriptions, scope metadata and `BindingView` are borrowed props.
`HelpOverlayState` owns only viewport scroll. Runtime owns modal layer, focus,
hover, pointer capture and prior-focus restoration.

```rust
enum HelpAction {
    Dismissed,
}
```

There is no generic Help action that executes the displayed command. A caller
may reopen the effective binding owner after dismissal.

The current source type is app-local `HelpOverlay` and still uses legacy
`WidgetId`, `Outcome`, `RenderCtx`, `ScrollState` and hardcoded section tuples.
Those names are source/oracle evidence. The target borrows sections and the
effective binding view from the caller; it does not preserve a second static
keybinding catalog as a public requirement.

## Ordinary use

The frozen Jackin Preview overlay uses `modal_frame` to dim the page, center a
rounded elevated frame titled “Keyboard shortcuts”, show a scope label, lay
out section blocks in one to three 36-cell columns, and provide a scrollbar,
edge fades and “↑↓ Scroll · Esc Close” hint. `Esc`, `?`, `q` and Enter close
the current app-local overlay; wheel and PageUp/PageDown/Up/Down scroll. The
overlay is opened only when no other modal is active and restores the prior
focus when dismissed.

Other baseline applications use Dialog compositions for their help pages:

- Jackin [`src/bin/jackin_preview/app.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/app.rs),
  including manager/capsule scope-specific sections;
- Showcase `src/bin/showcase/app.rs` “Keyboard & mouse” dialog;
- TablePro `src/bin/tablepro/app.rs` “Keyboard” dialog;
- Holla `src/bin/holla/app.rs` context-adaptive help modal;
- frozen help snapshots under `snapshots/jackin`, `snapshots/showcase`,
  `snapshots/tablepro` and `snapshots/holla` where present, plus the baseline
  drivers for all four applications.

These consumers are reference applications and conformance fixtures. The
future shared component must reproduce their approved output when migrated;
this page does not authorize product help rewrites.

Displayed chords resolve through the same effective binding view as the
[EX-09](../api/consumer-recipes.md) menu surfaces, HintBar and handlers.

## Ownership

- `update` consumes one input/cause. Escape and the effective close bindings
  request dismissal; Up/Down, PageUp/PageDown and wheel alter scroll state;
  scrollbar thumb gestures use the shared ScrollRegion runtime path.
- `draw` is read-only. It lays out section columns, paints key/description
  cells, registers the scroll surface and renders fade/scrollbar, but never
  changes scroll, focus, bindings or section data.
- `measure` is pure and derives one-to-three column layout from display-cell
  widths, body height and constraints. It handles zero/tiny dimensions and
  narrow collapse without mutating state.
- Runtime opens HelpOverlay as a modal layer, saves the current owner focus,
  focuses the overlay surface, consumes outside input, and restores the exact
  owner/draft after close.
- Binding resolution is shared with Menu, MenuBar and HintBar. A remap or
  disabled action is reflected consistently in every surface.

Shared dependencies:

- [`dialog`](./dialog.md), [`text-viewport`](./text-viewport.md),
  [`key-hint`](./key-hint.md), [`scroll-region`](./scroll-region.md),
  [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md),
  [`input-actions`](../foundations/input-actions.md),
  [`identity`](../foundations/identity.md),
  [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md),
  [`text`](../foundations/text.md), [`author`](../foundations/author.md),
  [`conformance`](../foundations/conformance.md).

## Customization

Advertised parts are `container`, `title`, `section`, `chord`, `description`,
`scrollbar` and `fade`. Parts use shared [`theme`](../foundations/theme.md)
roles and reserved rectangles; a patch cannot change layer ownership or
install a command dispatcher.

## Behavior

### Focus and keyboard

- Opening creates a modal scope and focuses the overlay's stable `Id`. The
  underlying editor/page cannot receive keys, paste or commands while it is
  open. Close restores the saved owner focus and draft.
- Escape and the resolved close binding dismiss. The current baseline also
  accepts `?`, `q` and Enter as close aliases; compatibility capture must
  preserve whichever effective bindings the application supplies.
- Up/Down/`j`/`k`, PageUp/PageDown, Home/End where supplied, and wheel scroll
  the viewport. Boundary events are consumed without changing focus.
- There is no editable or selectable text state in HelpOverlay. If a future
  help entry becomes an action, it must be a real child component with its own
  action contract rather than an implicit activation of prose.

### Pointer, hover and activation

- A scrollbar thumb press captures the overlay's scroll owner; matching drag
  updates scroll, while release outside ends capture without activating prose.
- The whole surface has no generic pressed, hovered or activation state. The
  capture plan marks activation not applicable; child scroll gestures are the
  applicable pointer states.
- Outside click dismisses the help layer once and never clicks the underlying
  editor/page. Keyboard input suppresses stale hover on any future interactive
  child; decorative section text does not receive a focus stop.

### Resize, scroll and reconciliation

- `measure` recomputes column count and body viewport on resize. A narrow
  layout collapses wide columns, while a later wider resize restores them and
  preserves scroll position as far as content permits.
- Scroll uses shared ScrollRegion semantics: no overflow, top/middle/bottom,
  wheel boundaries, thumb drag, resize while scrolled, edge fades and
  protected rows.
- Source sections/bindings reconcile only during `update` at an explicit
  revision. Removed rows cannot leave stale scroll or focus targets; displayed
  bindings update atomically with handler eligibility.

## Visual matrix

The default recipe preserves the baseline:

- dimmed page and rounded elevated frame with title and scope metadata;
- section headings and key/description hierarchy, including the exact
  display-cell column spacing and surface;
- responsive columns: wide layouts may show up to three columns; narrow
  layouts collapse columns while keeping rows readable and in source order;
- scrollbar, edge fade and protected footer/hint rows only when content
  overflows;
- Unicode descriptions and key labels clip at grapheme/display-cell
  boundaries, preserving wide-cell continuation;
- truecolor, 256, 16, explicit `none` and `NO_COLOR` capability lanes use
  semantic roles. Help has no invented animation; paused/reduced motion keeps
  content and scroll behavior identical.

| Axis | Required cases | Applicability |
|---|---|---|
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide | overlay and columns |
| Focus/hover | unfocused, focused, keyboard-suppressed hover, pointer-motion restoration | overlay surface or real child; no decorative focus |
| Activation | not applicable to whole surface; thumb press/drag/release outside applies to scrollbar | no prose activation |
| Scroll | no overflow, start/middle/end, wheel boundary, thumb drag, resize while scrolled, edge fade, protected row | content viewport |
| Layer | closed/open, nested policy, outside, Escape, owner removed, resize/reanchor, focus restore, modal-first paste | runtime-owned overlay |
| Bindings/data | effective remap, disabled action display, section reorder/removal, empty/partial content | caller supplied sections/view |
| Capability/motion | truecolor, 256, 16, none, NO_COLOR; paused/reduced | every visual variant |

Do not fabricate pointer activation, editing, disabled-control or animation
states for explanatory prose. Record explicit non-applicability.

## Verification

Use the plan dimensions `72x20`, `80x24`, `100x30`, `120x40`, `160x50` and
capabilities `truecolor`, `256`, `16`, `none`, `nocolor`. Expected artifacts
must be classified as `ExistingOracle`, `ExtractedOracle` or `Extension` before
candidate comparison; candidate code cannot create its own baseline.

| Case | Required observation |
|---|---|
| W26-01 | Wide columns collapse at narrow width and recover without row/key drift. |
| W26-02 | Remapped and disabled action representation matches HintBar and Menu. |
| W26-03 | Help scroll receives wheel/keys while an underlying editor remains inactive. |
| W26-04 | Close restores the exact owner focus and draft. |

Trace exact cells, wide continuations, colors/modifiers supported by the tool,
cursor, focus/capture/layer owners, viewport offset, binding revision and
typed dismissal count/target. Missing or invalid oracle setup is blocked,
never a pass.

Required negative tests include:

- draw twice cannot mutate scroll, source sections, bindings or focus;
- no prose row activates, and a thumb release outside cannot click through;
- modal-first paste/key/wheel routing blocks the underlying editor/page;
- close emits at most one dismissal and restores owner focus even if the owner
  moved or was removed, following runtime's fail-closed policy;
- a binding remap changes handler and display together; disabled actions cannot
  become reachable through stale text;
- zero/tiny areas and narrow Unicode columns cannot panic or split cells;
- part patches affect their advertised cells and cannot overwrite scrollbar,
  backdrop or focus barriers; dead-call/custom-paint mutation fails;
- one cell, scroll offset, cursor, focus/capture/layer owner or dismissal
  trace difference fails exact comparison.

## Rejected use

Forbidden: preview prints a padded shortcut table with its own scroll offset
instead of using `HelpOverlay`.

```rust
// Forbidden: preview owns the help grid and its scroll position.
ui.paint_str(row, 2, "  Esc      Close          ");
ui.paint_str(row + 1, 2, "  ?        Show help      ");
if pressed_down { help_offset += 1; }
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

Use `HelpOverlay::new` with borrowed sections and the shared `BindingView`;
it emits dismissal intent only.

## Known gaps

- [FIX-002](../implementation/code-remediation-backlog.md) (pending): replace
  historical Manager, menu, inspect, help, and quit screens; the help slice
  must use the real `HelpOverlay` with the shared binding view.
- [FIX-007](../implementation/code-remediation-backlog.md) (pending): resolve
  public signature drift against this contract; compile external examples and
  check the accepted state/action model.
- Capture cases W26-01–W26-04 await expected-artifact binding before candidate
  comparison; candidate code cannot create its own baseline.
