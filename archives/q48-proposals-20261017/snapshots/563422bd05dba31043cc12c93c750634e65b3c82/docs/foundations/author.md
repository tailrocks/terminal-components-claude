# F10 · Public component-author surface

**Status:** canonical Termrock foundation contract; implementation is future work.

**Legacy families:** C52.

**Visual authority:** the unchanged `visual-baseline` commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Author APIs preserve the frozen applications' cells and interactions; they do not authorize product redesign.

**Scope:** the constrained surface for implementing a new reusable Termrock component or a narrowly scoped product artwork fixture. Standard components remain the preferred surface for application authors.

This surface belongs to the future in-place Termrock library on `termrock-implementation` in this repository. It does not define a separate repository or a second application framework.

See the [public API contract](../api/public-api.md), [API authoring contract](../api/authoring.md), [architecture overview](../architecture/overview.md), [runtime contract](../foundations/runtime.md), [component index](../components/README.md), and [conformance contract](../verification/conformance.md).

## Source evidence

- [`src/ui/ctx.rs`](../../src/ui/ctx.rs) is the frozen implementation's current render-context boundary. Its `RenderCtx`, hit registration, focus registration, cursor request, modal barrier, and theme access are legacy implementation names, not future public compatibility requirements.
- [`src/bin/jackin_preview/rain.rs`](../../src/bin/jackin_preview/rain.rs) is application artwork evidence. Rain, warp phrases, logos, and instance-entry/exit rules remain application-owned; they are not Termrock standard components.
- The baseline applications and snapshots are read-only conformance consumers. A custom author surface must preserve their observable output when used by a future migration.

## Surface implementation boundary

The public names and signatures are owned by the
[API authoring contract](../api/authoring.md). This foundation owns their
implementation constraints: borrowed inputs, immediate callbacks, reserved
rectangles, clipping, normalized intents, and no access to mutable runtime
registries. Type spelling may change during the P1 public API freeze.

## Responsibilities and ownership

1. Application authors use the curated standard component API for Button, List, Grid, Menu, text editors, overlays, and the other documented surfaces. They do not reimplement a standard control with a custom paint callback.
2. A custom component author owns the component's caller-provided model, durable state, stable `Id`/`ItemKey` values, update policy, and typed actions. The author surface never owns a domain service, process, terminal session, or persistence operation.
3. `draw` callbacks receive read-only state and borrowed props. They may paint cells inside the reserved rectangle, request a cursor, publish layout/hit facts through the constrained `Ui`, and invoke immediate row/part callbacks. They must not mutate semantic state, read a clock or environment variable, perform IO, or execute a returned action.
4. Runtime owns focus, hover, press/capture, layers, hit dispatch, cursor arbitration, and normalized input. A custom author cannot write directly to those registries or add a second hit-testing engine.
5. Theme resolves semantic roles and capability conversion; exact visual values belong to the visual contract. Author callbacks receive resolved/borrowed style vocabulary or an explicit `StylePatch`; they do not invent a parallel global palette or bypass disabled/read-only recipes.
6. Callback lifetimes are immediate. `Fn`/`FnOnce` bounds do not make a callback pure by themselves; state-before/after checks and API review must catch hidden interior mutability, IO, or retained data.

## Parts, slots, and clipping

Each advertised `Part` has one reserved rectangle, a documented default painter, a style-resolution path, and a clip. `patch_part` changes the style of that part; `slot` replaces only parts explicitly marked replaceable by the component contract. Unsupported part names are rejected or diagnosed. Silently ignoring a typo is not acceptable.

The runtime applies the same owner/part identity and item key rules to stock and custom painters:

- Custom row/cell callbacks receive a reserved row/cell area, not the whole frame buffer.
- A callback cannot paint outside its rectangle, register an unrelated hit target, move a sibling's focus ring, or change layer ownership.
- Surface/container fills, disabled barriers, focus geometry, capture geometry, and modal backdrops are runtime/component-owned unless the component explicitly documents a replaceable part.
- Custom style patches flow through nested standard components according to the documented precedence order. A nested standard component cannot be restyled by painting over it after the fact.
- Reorder, insertion, filtering, and removal use stable `ItemKey` values; a displayed index cannot become a custom action identity.

This prevents a custom slot from hiding a broken standard widget. A standard Button/List/Grid/Menu mutation must change every composition that contains it, and its keyboard/pointer semantics must remain the shared runtime semantics.

## TerminalView exception

`TerminalView` is the narrow raw-cell exception. It consumes caller-provided prepared terminal cells and may preserve cell styles, wide-cell continuation, cursor, selection, and typed copy/link requests. `author::blit_terminal` is available only through that documented component boundary and only inside the assigned clip.

The raw-cell path still participates in runtime clipping, layer ownership, cursor arbitration, and input routing. It cannot write through a modal barrier, overwrite an outer layer, install an escape parser, or claim focus/capture without the same runtime protocol as other components. Standard UI surfaces must not use raw blitting to conceal a missing component implementation.

Termrock is not a terminal emulator, PTY runtime, shell, daemon, agent-session manager, or terminal escape parser. A caller may prepare cells using an external terminal/session system; Termrock renders the prepared presentation and returns typed requests.

## Product boundary

Product rain, warp phrases, logos, instance-entry/exit rules, host/service clients, account state, launch flows, and product-specific routing remain outside the core author surface. A test-only baseline artwork fixture may exercise clipping, supplied time, and style parts, but it must not grow into a particle engine or a Jackin-specific public widget.

The same boundary applies to the preserved applications:

- `showcase` remains the generic component laboratory;
- `tablepro`, `jackin-preview`, and `holla` remain frozen conformance consumers;
- future migration work may replace library internals while preserving their source behavior and snapshots;
- no author task may turn this foundation into a product redevelopment project.

## Interaction with shared foundations

| Concern | Author surface provides | Shared owner |
| --- | --- | --- |
| Paint | Clipped text/style/part helpers | Theme and component recipe |
| Geometry | Reserved region publication | Layout/runtime |
| Focus and capture | No raw access; normalized intent only | Runtime |
| Stable identity | Caller-supplied `Id`, `ItemKey`, `Part` | Identity foundation |
| Actions | Typed action returned through response | Input/response foundation |
| Layers | Child drawing within a supplied layer/clip | Layer foundation |
| Time | Supplied phase/sample for deterministic paint | Runtime/time foundation |
| Terminal cells | Prepared borrowed source through `TerminalView` | Session remains external/optional |

The author API is therefore an extension point for generic capability, not a universal `Widget` trait and not a back door into runtime internals. Standard component authors get a smaller, stable vocabulary so custom code cannot fork focus, scrolling, hover, capture, text editing, or layer behavior.

## Required verification

Implementation must add these gates. The current baseline does not claim they are already implemented.

### Public-surface and safety gates

- An external consumer crate defines a keyed custom component using only public Termrock exports; no private imports, `include!`, `pub(crate)` shortcut, or copied baseline painter is allowed.
- Compile-fail tests reject access to mutable focus/hit/layer storage, unbounded buffer writes, unsupported parts, and product-only internals.
- A custom painter cannot write outside its reserved rectangle. Clipped zero/one-cell regions and nonzero frame origins are covered.
- Draw twice with unchanged props/state leaves model, action queue, focus, capture, and runtime-owned geometry observably unchanged.
- A custom callback with hidden mutation/IO is caught by before/after model checks and review, acknowledging that a function type alone cannot prove purity.

### Interaction and parity gates

- Reorder/removal of keyed custom rows shares standard runtime hit semantics; a stale display index cannot activate a new item.
- Standard Button/List/Grid/Menu mutation changes every composition using it. A dead standard call plus hand-painted replacement fails a mutation/ownership test.
- TerminalView preserves prepared cell symbols, continuation semantics, styles supported by the comparator, cursor/selection requests, and typed copy/link actions without overwriting an outer modal or layer.
- Baseline applications exercise custom parts only where their frozen output requires them. New themes or robust clipping cases are marked `Extension`, not claimed as baseline parity.
- No standalone hover/pressed screenshot is invented for the author foundation. Visible states are verified through the owning components and composed application fixtures.

## Migration note

The current `RenderCtx` API exposes more implementation detail than the target author facade. It may remain as a temporary internal bridge during the in-place refactor, but it is not a Termrock public compatibility promise. This document is the single canonical owner for custom component authorship, constrained slots, and the raw-cell exception; component pages link here instead of defining competing author rules.
