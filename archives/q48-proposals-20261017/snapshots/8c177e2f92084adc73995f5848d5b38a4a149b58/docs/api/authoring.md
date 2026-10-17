# Public component-author surface

This document owns the caller-facing names and use of Termrock's constrained
author API. The shared implementation and safety rules are owned by the
[authoring foundation](../foundations/author.md). Read this with the [public
API contract](public-api.md), [types](types.md), and
[architecture overview](../architecture/overview.md).

## Purpose and boundary

Standard Termrock components are the default. The public author surface is for
a generic reusable component or a part explicitly marked replaceable by its
component contract. It provides borrowed content, semantic styles, clipped
painting, geometry publication, cursor requests, and normalized owner intents.

It does not expose unrestricted buffer writes, mutable runtime registries,
product routing, services, executors, filesystem, PTY/session handles, or a
universal boxed `Widget` trait. There is no `show()` method mixing event
semantics with painting. The API applies to this repository's future in-place
Termrock library; it does not claim a completed source or package rename.

Only a reusable declared component or a constrained visual part inside its
owning library crate may use this surface.
Preview route, shell, domain, and simulation code must not use it.
Raw buffer access through `termrock::author` or an alias in preview code
violates the [component-only contract](../architecture/component-composition.md).
A component renderer may legitimately draw cells; that permission never extends
to arbitrary preview screen painting.

## Proposed calls

These Rustdoc-style declarations name the public extension surface. P1 must
compile them through an external-consumer example before the spelling is
frozen.

```rust
pub mod author {
    pub fn register(ui: &mut Ui<'_>, region: RegionSpec);
    pub fn paint(
        ui: &mut Ui<'_>,
        area: Rect,
        text: StyledText<'_>,
        part: Part,
    );
    pub fn request_cursor(ui: &mut Ui<'_>, owner: Id, cursor: CursorSpec);
    pub fn blit_terminal(
        ui: &mut Ui<'_>,
        area: Rect,
        source: &dyn TerminalSource,
    );
}

impl Ui<'_> {
    pub fn part<R>(
        &mut self,
        owner: Id,
        part: Part,
        area: Rect,
        painter: impl FnOnce(&mut PartUi<'_>) -> R,
    ) -> R;
}
```

The author calls participate in the same component lifecycle as standard
components. Use the public [update/measure/draw and response contract](public-api.md)
for state ownership, typed actions, and phase semantics. Use the
[foundation contract](../foundations/author.md) for clipping, reserved
rectangles, callback lifetimes, style resolution, row/cell parts, and
forbidden runtime access.

## Standard parts and customization

Components expose only their documented `Part` values. The public builder
offers an instance `StylePatch`, a per-part `StylePatch`, or an immediate
`SlotPainter` only when the component explicitly allows that operation.
Unsupported names are diagnosed. Style customization changes semantic roles;
it does not redraw over an unrelated child or alter the component's size or
interaction ownership.

For collection components, a component contract may expose a borrowed,
typed row or cell painter. The painter receives the item key and declared
part, not a display index. Its exact constraints are defined by the
[authoring foundation](../foundations/author.md).

## TerminalView edge

`author::blit_terminal` is reserved for the documented `TerminalView` path. It
accepts caller-provided prepared cells, including styles and continuation
cells, and stays inside the assigned clip. Termrock returns typed interaction,
copy, and link requests; it does not parse terminal escapes or own a PTY,
shell, emulator, daemon, or agent session. See the [TerminalView contract](../components/terminal-view.md)
and optional [session edge](../foundations/session.md).
