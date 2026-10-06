# Termrock public API contract

Status: proposed target for the in-place refactor. This is a Rustdoc-style design contract, not a claim that the current crate compiles these signatures. The implementation phase starts by qualifying the contract against external consumer examples in P1.

## Current versus proposed signatures

Signatures in this document are the proposed target unless stated otherwise.
They do not describe current source.
For example, the `TextInput::new(id, value, revision)` constructor below is proposed.
The current source at `4d117b48` uses `TextInput::new(id)` with phase-specific value access
(`crates/termrock-fields/src/input.rs:980`).
Ordinary consumer examples with verified status labels live in
[consumer recipes](consumer-recipes.md) and its [example catalog](example-catalog.json).
Reconcile one signature per component in its canonical contract before implementation.

## Design boundary

Termrock is a reusable Rust terminal UI library. A caller supplies domain data, durable component state, event timing, and application effects. A component interprets normalized runtime input, publishes a typed intent, measures itself, and paints from immutable inputs. It does not own a product router, persistence, provider client, PTY, shell, or database.

The API is being introduced in this repository in place. The existing source is the starting implementation and the four current applications are conformance consumers. The target library identity is Termrock even while current source modules retain legacy names during migration.

Every public component contract follows the same conceptual phases:

```rust
component.update(cx, state) -> Response<Action>
component.measure(measure_cx, constraints[, state, model]) -> Size
component.draw(ui, area, state[, model]) -> Rect
```

The exact receiver and model parameters vary where a component is stateless or consumes a borrowed source. The ownership rule does not vary: semantic mutation belongs to `update`, geometry calculation is pure measurement, and `draw` consumes immutable semantic inputs.

## Ownership and phases

### Caller-owned domain data

The caller owns committed values, row and column models, eligibility, validation policy, provider results, and all side effects. Props borrow this data for one component call. A props value never stores a borrowed model in durable component state and never requires a `'static` callback.

### Caller-owned durable component state

State exists only where interaction must survive a frame: draft text, cursor/current key, scroll anchor, selection anchor, expanded tree keys, picker stage, edit phase, or split preference. State does not contain terminal rectangles, theme colors, runtime registries, callbacks, domain service clients, or borrowed model data. Geometry and hit regions are republished each frame by the runtime.

Stateless controls do not receive empty state types merely for symmetry. `Button`, `Checkbox`, `Toggle`, `Brand` in its static mode, `Field`, `Panel`, `Empty`, `ProgressBar`, `Spinner`, `Meter`, and similar controls expose no fake persistent state.

Normal state types should provide useful `Debug`, `Default`, equality, and cloning only where the data is safe. Secret-bearing states are the explicit exception described in [Secret state](#secret-state).

### Borrowed props

Props are short-lived, immutable inputs. Builders are consuming and `#[must_use]` unless a component documents otherwise. Examples:

```rust
let input = TextInput::new(name_id, model.name.as_str(), model.revision)
    .label("Name")
    .required(true)
    .read_only(model.locked);

let response = input.update(&mut cx, &mut view.name);
input.draw(&mut ui, name_area, &view.name);
```

The caller applies `response.action` to its model before constructing the next frame's props. Drawing an old props snapshot after mutating its source is not a supported shortcut.

### Update

`update` consumes normalized owner intents from `Cx`. It may mutate only the supplied component state and request runtime effects through the constrained context: focus, capture, layer operations, invalidation, cursor intent, or a typed semantic response. It may reconcile a revisioned source before interpreting input. It does not perform persistence or network/clipboard/PTY effects.

```rust
let response: Response<TextAction<String>> = input.update(&mut cx, &mut text_state);
if let Some(TextAction::Commit { value }) = response.action {
    model.name = value;
    model.revision = model.revision.next();
}
```

Actions are intents. Runtime records focus/hit/capture metadata and does not execute application actions. The application decides whether and how to persist an action.

### Measure

`measure` computes a finite size from props, immutable state/model, theme metrics, and `Constraints`. It must not mutate text, focus, current selection, model data, or time. It handles zero and one-cell constraints, nonzero origins through layout helpers, Unicode width, and exhausted space without wrapping arithmetic.

Stateful measurement takes an explicit immutable state/model view when required; it must not hide semantic reconciliation in measurement. A component can request layout facts during draw, but the next semantic update owns any resulting decision.

### Draw

`draw` receives immutable state and source/model data. It may paint cells through `Ui`, publish current geometry and layout facts, and request cursor presentation. It must not commit values, validate by mutating domain data, select an item, close a layer, execute a command, advance time, read the environment, perform IO, or reconcile source data.

Repeated draw with equal inputs is observably idempotent. Immutable references alone are not a complete purity proof: interior mutability and arbitrary callbacks are restricted by [the authoring contract](authoring.md) and checked with before/after model assertions.

## Responses and event flow

All semantic component output uses the canonical [`Response<A>` shape and field names](types.md#typed-response). That type dictionary owns its representation; the concrete fields may remain private, with public observations exposed narrowly and through typed accessors.

`Response<A>` carries at most one typed action for an update call. `Flow` (`Bubble` or `Consumed`) and `Invalidate` (`None`, `Paint`, or `Layout`) are independent. Response metadata is registered once by the shared runtime response helper; callers do not maintain a second focus/hover engine.

Typical flow:

1. The runtime normalizes a terminal event into `Input` and resolves its owner using current valid geometry, focus scope, layer order, and capture.
2. The component receives an owner-scoped intent through `Cx`.
3. `update` mutates caller-owned component state, requests runtime transitions, and returns a typed action when a semantic intent completes.
4. The caller applies the action to its domain model and advances the relevant `Revision`.
5. `measure` and `draw` receive the new props/state/model. `draw` publishes geometry for the next coordinate-sensitive dispatch.

Pointer activation is down → optional drag → up → conditional action. Down alone, release outside, removed target, disabled target, and click-through overlays emit no application action. Keyboard input suppresses stale hover until real pointer motion. Focus, hover, pressed, activation feedback, selected/current, and editing are separate visual axes.

Actions use semantic identities and origin metadata rather than display positions:

```rust
enum ActivationOrigin { Keyboard, Pointer, Programmatic }
struct Activated { origin: ActivationOrigin }
struct ValueChanged<T> { value: T, origin: ActivationOrigin }
enum TextAction<V> { Edited, Commit { value: V }, Cancelled, Conflict }
```

Component-specific actions are documented beside each component. They must remain typed and keyed; an untyped event bus is not part of the public API.

## Controlled values and reconciliation

Checkbox, Toggle, RadioGroup, Select, Tabs, and collection selection are controlled values. The caller owns the selected value. A component may own cursor and anchor state, but it must not silently replace the caller's selected entity with an internal copy.

Dynamic collections and source-backed editors carry a caller-supplied `Revision` or obtain one from a source trait. A revision monotonically identifies accepted source content/order. The caller advances it before update/draw after changing a source. Overflow and stale ranges fail closed.

Reconciliation occurs during `update`, before input interpretation. It retains surviving `ItemKey`, `ColumnKey`, `FieldKey`, `CellKey`, and line identities across reorder, insertion, filtering, rename, and refresh. A removed cursor may fall back to the nearest eligible successor, then predecessor, then no cursor, according to the component contract. A pending activation, acknowledgement, capture, or draft never silently transfers to that fallback item.

An optional public `reconcile` operation may exist through a shared `Reconcile` helper, but it must call the same update algorithm. `draw` never performs semantic reconciliation.

Collections borrow row records immediately. A row callback cannot retain a borrowed row, spawn work, mutate the model, or dispatch through an executor. Noninteractive headings and separators are not fake selectable rows.

## Text, editing, and secret state

TextInput, TextArea, and CodeEditor share one grapheme-aware editing core. The component selects policy for newline, Escape, blur, completion, wrapping, and commit:

- single-line TextInput keeps the baseline Escape rollback behavior;
- TextArea keeps its baseline editing and Escape commit behavior;
- CodeEditor adds source-keyed lines, marks, completion ranges, and diagnostic display;
- a stale completion/edit range is rejected when its `Revision` no longer matches.

When an external revision conflicts with an active draft, preserve the draft and return `TextAction::Conflict` by default. The caller explicitly chooses `KeepDraft` or `Reload`; neither value is silently discarded. Plain text may expose a draft string for caller validation through a narrow observation. It must not expose mutable internal buffers.

### Secret state

Secret input uses the same editing behavior with a specialized mode and value type:

```rust
TextInput::secret(id, secret, revision) -> TextInput<'a, SecretText>
TextInputState<SecretText>
TextAction<Secret>
```

Secret values and secret-bearing state are not generally `Clone`, ordinary `Debug`, or serializable. There is no plain draft getter. Copy and tail reveal require an explicit `SecretPolicy`; the default is neither. Secret commits are typed and not freely cloneable. Replacement, cancel, and drop clear owned secret buffers through the reviewed zeroization path. Validators return safe display messages and must not echo input.

## Grid capability boundary

Grid is one engine with table-row and cell/grid presentations. Its capability boundary is explicit:

```rust
Grid::update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut GridState,
    model: &dyn GridModel,
) -> Response<GridAction>

Grid::update_editable(
    &self,
    cx: &mut Cx<'_>,
    state: &mut GridState,
    model: &mut dyn GridEditor,
) -> Response<GridAction>

Grid::measure(
    &self,
    cx: &MeasureCx<'_>,
    constraints: Constraints,
    state: &GridState,
    model: &dyn GridModel,
) -> Size

Grid::draw(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &GridState,
    model: &dyn GridModel,
) -> Rect
```

The read-only path cannot mutate a model. The editable path can request a revision-valid local edit through `GridEditor`; it does not perform persistence, SQL, provider, or application work. `GridState` stores keyed cursor/selection/edit phase, never row indexes as identity. `GridColumn` uses a stable `ColumnKey`. Table presentation and cell editing keep distinct visual recipes while sharing geometry, scrolling, reconciliation, and keyboard ownership.

## Parts, patches, and slots

Components expose curated customization rather than a general repaint escape hatch:

```rust
component.patch(StylePatch)
component.patch_part(Part, StylePatch)
component.slot(Part, &SlotPainter)
```

`patch` changes advertised semantic roles for one instance. `patch_part` targets one documented part. `slot` replaces only a replaceable part with a borrowed immediate painter. Each part has one reserved rectangle, clip, ownership, and style resolution path. Unsupported part names are diagnosed; they are not silently ignored.

Container/surface fills, geometry, focus gutter, hit regions, pointer capture, disabled barriers, layer backdrop, and cursor arbitration are not arbitrary slots. A slot cannot restyle a sibling, change an action target, register a second focus router, or paint outside its rectangle. The stock component remains the owner of behavior.

Part and row/cell callbacks receive immutable typed data, immutable visual state, a reserved area, and constrained `PartUi`/`RowUi`/`CellUi`. They never receive a mutable domain model, global buffer, runtime registry, or executor. Global and subtree patches reach stock and custom row/cell painters and empty/loading placeholders through the same explicit theme resolution.

## Runtime boundary

The runtime owns focus, focus visibility, hover, pointer capture, press/activation feedback, key scope, hit geometry, layer ordering, cursor arbitration, and supplied time. Components request transitions through `Cx`; they do not read or mutate runtime registries directly. The runtime dispatches against the last valid published geometry and rejects coordinate-sensitive input after a resize/topology change until geometry is republished.

`Scene` may be a small driver for a generic consumer that sequences update and read-only draw callbacks. It is not a universal widget trait, product router, or reason to add persistent state to stateless controls. The optional terminal/session adapter owns raw-mode and crossterm cleanup; it is outside the core component API.

See [runtime](../foundations/runtime.md), [state and events](../foundations/input-actions.md), and [layers](../foundations/layout.md) for ownership details.

## Deliberate exclusions

The target public API does not require or promise:

- a universal boxed `Widget` trait for all components;
- a `show()` API that mixes event semantics and painting;
- an untyped event bus or universal mutable `RenderCtx`;
- compatibility aliases for current `WidgetId`, `Outcome`, `RenderCtx`, or `junie_tui` names;
- component-owned persistence, provider clients, filesystem, database, clipboard emission, PTYs, shell commands, or product routes;
- a terminal emulator, PTY runtime, shell, daemon, agent-session manager, or terminal escape parser;
- product-specific widgets such as account pickers, workspace launchers, Jackin shells, Holla workflows, or TablePro backends.

The current implementation may contain these names or broader internals while it is being refactored. That is migration evidence only. The target Termrock facade is the contract above.

## Component coverage

The component directory owns detailed constructors, actions, states, parts, applicable interaction matrices, dependencies, and parity cases. The 45 API surfaces are:

| Group | Surfaces |
| --- | --- |
| Chrome | Brand, Panel, MenuBar, StatusBar, HintBar, KeyHint, TooSmall |
| Controls | Button, Checkbox, Toggle, RadioGroup |
| Forms | Field, TextInput, TextArea, Select, Form |
| Collections | ChipBar, List, FilterList, NavList, Tree, Tabs |
| Overlays and composition | Picker, CommandPalette, PickerChain, Completion, Dialog, Menu, ContextMenu, HelpOverlay, Wizard |
| Data and text | Grid, CodeEditor, DiffView, TextViewport, Props, PropsList |
| Layout | SplitPane, ScrollRegion |
| Feedback | Steps, Empty, ProgressBar, Spinner, Meter |
| Adapter boundary | TerminalView |

Consolidated mechanisms remain behaviorally distinct where the frozen applications require it: one Grid engine serves table and cell modes; one Menu engine serves Menu/ContextMenu/MenuBar; one Picker mechanism serves Picker/CommandPalette/chained selection; one text core serves TextInput/TextArea/CodeEditor; one ScrollRegion serves scrolling, thumb capture, and fade policy; Panel plus TextViewport replaces a separate ScrollPanel; StatusBar absorbs segment-style presentation; DerivedHintBar is metadata, not a second renderer.

## Theme entry points

The default theme entry point is `Theme::termrock()`, which selects the pinned baseline theme. `Theme::paper()` is an optional extension palette for detecting hardcoded colors; it is not an alternate visual target for the preserved applications. Exact baseline token values and rendered states belong to the [visual contract](../design/visual-contract.md); theme resolution belongs to the [theme foundation](../foundations/theme.md).

Theme selection and capability are explicit. The target API covers TrueColor, Ansi256, Ansi16, Mono, explicit color absence, and NO_COLOR policy as separate inputs. A caller-requested capability cannot exceed the terminal ceiling. Component code resolves semantic Role and Surface values through Theme; it does not construct ad-hoc RGB styles.
