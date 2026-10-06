# F04 · Layers, modal ownership and anchored popups

**Scope:** shared generic library/test infrastructure, not a product subsystem.  
**Legacy families:** C09  
**Visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; interface spellings below are proposals.

## Source references

- [src/ui/popup.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/popup.rs)
- [src/widgets/dialog.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/dialog.rs)
- [src/widgets/menu.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/menu.rs)

[Target architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) · [refactoring task catalog](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv)

## Proposed public surface

Rustdoc-style declaration notation. This is not a compiled implementation. Read with [PUBLIC-API.md](../PUBLIC-API.md) and the [shared type dictionary](../reference/TYPES.md).

```rust
Cx::open_layer(id: Id, spec: LayerSpec) -> Result<(), LayerError>
Cx::close_layer(id: Id, reason: DismissReason)
Ui::layer<R>(id: Id, body: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> Option<R>
struct LayerSpec { owner: Id, kind: LayerKind, anchor: Anchor, size: LayerSize, dismiss: DismissPolicy, backdrop: Backdrop, inert_below: bool }
enum LayerKind { Modal, Popover, Menu }
enum DismissReason { Escape, OutsidePointer, OwnerRemoved, Programmatic }
```

## Contract

1. One runtime layer stack provides placement, Z order, capture barriers and focus restoration for Dialog, Picker, Select, Menu, Completion and help.
2. Component measures a size; runtime resolves the rectangle against current viewport and anchor. No shared popup.surface ID. Popovers can trap pointer without trapping normal editor typing; modal focus stays confined.
3. App content is a borrowed slot closure invoked immediately in draw. Do not box a static callback or require a cloned application model simply to open a dialog.
4. Backdrop and overlay rendering preserve baseline cell colors/dimming; layer compositing uses written-cell ownership and proper clipping so transparency cannot overwrite unrelated cells.
5. Click outside is one dismissal gesture, not dismissal plus activation of a button below. Escape closes only the top applicable level. Remove the owner and release capture/keyboard ownership deterministically.

## Required proof

- nested modal -> picker -> submenu with one-level Escape
- outside click never passes through after close
- reanchor on resize and deleted anchor
- popover completion retains editor cursor/typing
- transparent/partial layer leaves lower cells correct

This foundation has no invented independent hover/pressed screenshot. Its visible effects are proved through the components and composed fixtures that use it. Nonvisual invariants have headless/state/compile-fail/protocol tests. Implementation and independent verification are not executed in this document pack.
