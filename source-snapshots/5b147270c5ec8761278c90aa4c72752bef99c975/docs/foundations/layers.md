# Layers, modal ownership, and anchored popups

**Foundation F04 · canonical owner of Z order, placement, barriers, dismissal, and focus restoration**

**Legacy family:** C09.

This page defines the one shared layer mechanism used by Dialog, Picker, Select, Menu, ContextMenu, Completion, HelpOverlay, and other overlay components. It is not a second widget renderer and it does not own product commands or application state.

Termrock will be built by refactoring this repository in place on `termrock-implementation` from the frozen visual baseline commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Current source evidence is [`src/ui/popup.rs`](../../src/ui/popup.rs), [`src/widgets/dialog.rs`](../../src/widgets/dialog.rs), [`src/widgets/menu.rs`](../../src/widgets/menu.rs), and the barrier helpers in [`src/ui/ctx.rs`](../../src/ui/ctx.rs). Current `WidgetId`/`RenderCtx` names are migration names; they do not define the future public API.

## Source and oracle references

- Migration input: `termrock-library-spec/foundations/layers.md`, the F04 entry in `termrock-library-spec/reference/foundations.json`, and the layer/composition sections of `termrock-library-spec/SPECIFICATION.md`, `PUBLIC-API.md`, and `reference/TYPES.md`.
- Frozen source: [`src/ui/popup.rs`](../../src/ui/popup.rs), [`src/widgets/dialog.rs`](../../src/widgets/dialog.rs), [`src/widgets/menu.rs`](../../src/widgets/menu.rs), and [`src/ui/ctx.rs`](../../src/ui/ctx.rs) at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.
- Runtime focus/hit/capture and frame publication: [`runtime.md`](runtime.md).
- Stable layer/component identity: [`identity.md`](identity.md).
- Layout/measurement and clipping: [`../foundations/layout.md`](../foundations/layout.md).
- Overlay visual and interaction parity: [`../design/visual-contract.md`](../design/visual-contract.md), [`../design/interaction-contract.md`](../design/interaction-contract.md), and [`../verification/visual-parity.md`](../verification/visual-parity.md).

## Target public surface

```rust
Cx::open_layer(id: Id, spec: LayerSpec) -> Result<(), LayerError>
Cx::close_layer(id: Id, reason: DismissReason)

Ui::layer<R>(
    id: Id,
    body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
) -> Option<R>

struct LayerSpec {
    owner: Id,
    kind: LayerKind,
    anchor: Anchor,
    size: LayerSize,
    dismiss: DismissPolicy,
    backdrop: Backdrop,
    inert_below: bool,
}

enum LayerKind { Modal, Popover, Menu }
enum DismissReason { Escape, OutsidePointer, OwnerRemoved, Programmatic }
```

`LayerSpec` is declarative placement and ownership metadata. The component supplies measured size and borrowed body content; runtime resolves the final rectangle against the current viewport and anchor. `Ui::layer` invokes its body immediately during draw. It does not box a static callback, clone an application model, or schedule background work merely to paint a layer.

## Single stack and ownership

One runtime layer stack owns:

- stack order and Z ordering;
- anchor resolution and viewport-constrained placement;
- pointer/focus/key barriers;
- backdrop and clipping composition;
- outside dismissal routing;
- restoration of the surviving prior focus owner;
- deterministic release of capture/keyboard ownership when a layer closes.

The caller owns whether a layer is present, its borrowed content/model, typed actions, and the domain effects of those actions. A layer does not own persistence, navigation, a service client, an executor, or a product-specific workflow. `Dialog`, `Picker`, `Menu`, and `Completion` use this mechanism; they do not each implement their own popup stack.

Stable `Id` identifies the layer owner. Child controls derive identities from stable semantic keys according to [`identity.md`](identity.md). There is no shared magic `popup.surface` ID in the target architecture: a surface may register a frame-local hit barrier, but it does not masquerade as the semantic owner of every nested control.

## Layer kinds and interaction policy

### Modal

A modal is topmost, blocks lower pointer targets, and traps focus to visible enabled controls in its scope. The backdrop follows the baseline cell-color/dimming recipe. Escape closes only the top applicable modal level according to `DismissPolicy`; it does not broadcast to every open layer. On close, runtime restores the previous focus owner if it still exists and is eligible, otherwise the documented fallback or `None`. Removing the owner while open closes the layer and releases capture deterministically.

### Popover

A popover is anchored to a screen rect, control part, or owner identity and is reanchored after resize or anchor change. It may block pointer events below without trapping ordinary keyboard/editor typing. Completion and picker popovers retain the editor cursor and typing ownership when their policy says so. A popover never forwards a completed outside dismissal click to the lower page.

### Menu

Menu, ContextMenu, and MenuBar share the same engine and layer rules. A menu has a topmost command scope, pointer barrier, stable row keys, disabled-row policy, and one-level Escape dismissal. Placement may be below, above, or beside an anchor, then flips/clamps to the current viewport. A chosen command returns a typed action to the caller; the layer stack does not execute it.

## Placement, geometry, and resize

The runtime resolves `Anchor` and `LayerSize` using current measured content and viewport constraints. It must keep the final rectangle inside the viewport where possible, flip an anchored popup when there is insufficient room, and clamp oversized content without unsigned-coordinate underflow. Anchor deletion produces the documented fallback (close or reanchor to owner/screen); it never dereferences a stale rectangle.

Layer geometry is frame data, not durable state. A resize invalidates old placement and pointer geometry; the runtime measures and republishes before accepting coordinate-dependent input. Nested layers resolve in stack order so a picker inside a modal, or submenu inside a menu, cannot register under its parent.

## Compositing and barriers

Backdrop and overlay painting preserve baseline cell ownership:

- clip every layer to the current viewport;
- write only cells owned by the layer or its declared backdrop policy;
- preserve lower cells through transparent/partial surfaces;
- keep border, fill, cursor, selection, and dimming roles in the semantic theme;
- publish pointer/focus barriers together with the frame so a visible layer cannot be interactive only after a later draw.

An inert layer below an active modal remains blocked where the modal policy requires it, even when the modal has holes or transparent cells. A click outside is one dismissal gesture. Closing a layer releases capture before lower layers become eligible; the same pointer-up cannot activate a newly exposed control.

## Runtime lifecycle for a layer

1. Caller state requests/open layer with stable owner `Id` and `LayerSpec`.
2. Runtime records the prior focus/capture owner and pushes the layer scope.
3. Component measures content; runtime resolves anchor and size.
4. Draw publishes the layer surface, child geometry, focus registrations, and barriers transactionally.
5. Input is offered to the top applicable layer first. Modal/editor scopes take paste/text before ordinary commands.
6. Typed actions and dismissal reasons return through `Response<A>`/runtime requests; runtime does not run domain effects.
7. Close, owner removal, or replacement pops only the intended level, releases ownership, and restores a surviving fallback. The next frame publishes the resulting geometry.

See [`input-actions.md`](input-actions.md) for action/flow semantics and [`runtime.md`](runtime.md) for the shared capture/focus/time lifecycle. This page does not redefine those contracts.

## Required proof and parity coverage

The implementation and conformance suites must include:

- nested modal → picker → submenu with Escape closing exactly one applicable level;
- outside click dismissing a popover/modal without activating content below;
- anchor deletion and viewport resize reanchoring or closing according to policy;
- popover completion retaining editor cursor/typing ownership;
- modal close restoring a surviving prior focus target, then selecting the documented fallback when it was removed;
- transparent/partial layer composition leaving lower cells, styles, and continuation cells correct;
- disabled/inert lower controls remaining unreachable while the layer is open;
- capture release on owner removal and no stale pointer-up activation;
- layer geometry and barrier publication being atomic on draw/registration failure;
- exact comparison of backdrop, border, clipping, focus, cursor, selected key, capture owner, dismissal reason, and typed action count against the frozen oracle.

No independent baseline screenshot is invented for the layer foundation. Dialog/Menu/Picker/Completion and the four preserved applications provide the applicable visual states. New robustness cases are marked `Extension` and reviewed separately from `ExistingOracle`/`ExtractedOracle` cases.

## Implementation acceptance

P4 is complete only when Dialog, Picker, Select, Menu, ContextMenu, MenuBar, Completion, HelpOverlay, and related components use one runtime layer stack; placement, barriers, dismissal, and restoration are runtime-owned; body content remains borrowed and immediate; and all applicable baseline overlay frames/actions pass exact conformance. No task may introduce parallel popup registries, a shared magic surface owner, click-through dismissal, or product-specific service behavior.
