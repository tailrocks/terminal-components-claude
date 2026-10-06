# Proposed public API contract

## Reading these references

Every file under `components/` declares a constructor, available builder families, durable state, typed actions, update/draw/measure signatures, styled parts and behavior cases. These are **Rustdoc-style signature declarations**, not compilable implementations. Builder entries omit `self -> Self` for readability; they are consuming, `#[must_use]` builders unless noted. No code in this pack claims to be the implemented crate.

The three lifetimes are simple: caller domain data outlives a component call; ephemeral props borrow it; durable state never borrows it. Borrowed row/part callbacks are immediate `Fn`/`FnOnce` calls, never background tasks or mandatory `'static` callbacks.

## Normal calling pattern

```rust
// Proposed usage; type-check after implementing the documented API.
let input = TextInput::new(name_id, model.name.as_str(), model.revision)
    .label("Name")
    .required(true);
let response = input.update(cx, &mut view.name);
match response.action {
    Some(TextAction::Commit { value }) => {
        model.name = value;
        model.revision = next_revision(model.revision);
    }
    Some(TextAction::Cancelled) | None => {}
    _ => {}
}

// Separate draw callback: model and view are read-only here.
TextInput::new(name_id, model.name.as_str(), model.revision)
    .label("Name")
    .required(true)
    .draw(ui, name_area, &view.name);
```

`next_revision` is an application helper, not a promised library export. Update and draw construct equivalent current props. Actions are applied before the next frame; drawing an old props snapshot after mutating its model is not a supported shortcut.

```rust
// Proposed stateless control use.
let result = Button::new(save_id, "Save")
    .variant(ButtonVariant::Primary)
    .disabled(!valid || saving)
    .update(cx);
if result.action.is_some() {
    // Caller records an intent; no persistence is run in Button.
    requested_save = true;
}
```

## Rendering and mutation boundary

All stateful components accept immutable state during draw. Grid's read-only and editing capability is intentionally explicit:

```rust
Grid::update(&self, cx: &mut Cx<'_>, state: &mut GridState,
             model: &dyn GridModel) -> Response<GridAction>
Grid::update_editable(&self, cx: &mut Cx<'_>, state: &mut GridState,
                      model: &mut dyn GridEditor) -> Response<GridAction>
Grid::draw(&self, ui: &mut Ui<'_>, area: Rect, state: &GridState,
           model: &dyn GridModel) -> Rect
```

GridEditor controls mutation of a local supplied model only. No persistence/network effects are permitted in a component callback. FormControls is a caller adapter, not an untyped field database: it commits/binds child values and validates current values before FormAction::Submit is emitted.

## Part customization

`patch(StylePatch)` applies an instance override. `patch_part(Part, StylePatch)` specializes one advertised part. `slot(Part, &SlotPainter)` exists only for replaceable parts documented by that component. Each part has one reserved rectangle, style resolution path and clip. A callback cannot restyle an unrelated sibling or change action ownership.

The reference `parts` lists name the proposed parts that must be implemented/tested. Not every part is replaceable. Surface/container fills, hit geometry, capture and hidden disabled barriers are not arbitrary replacement slots. Accept only implemented slot names and reject unsupported parts rather than silently ignoring them.

Per-family default variant and visual metrics come from the pinned source, not a generic CSS-like cascade. Global/subtree overrides must reach custom row/cell renderers and empty/loading placeholders as well as the stock painter.

## Reconciliation and safe state access

Collection sources are keyed and revisioned. Their update entry points perform explicit source reconciliation before interpreting input; an optional public `reconcile` operation can be exposed by the Reconcile trait, but it must call the same algorithm. No semantic reconciliation in draw.

Expose state through narrow methods: cursor/current key, scroll position, edit phase and safe selection information. Private fields stay private. Plain text state may expose a draft string for caller validation; secret state may not. Secret commits use TextAction<Secret>; those actions are not generally Clone or serializable.

Text editing uses the supplied source revision. If an external revision arrives during editing and conflicts with the captured starting value, default to preserving draft and reporting `TextAction::Conflict`. Explicit `resolve_conflict(KeepDraft|Reload)` chooses the next source/draft relationship. Never silently discard either value. The response enum includes `Edited`, `Commit { value }`, `Cancelled`, and `Conflict`; nonconflicting source refresh may reconcile without a domain action.

## Keymap and runtime behavior

Widget updates consume normalized owner intents via Cx. Runtime owns focus/capture, key scope, layers and feedback. Response metadata is registered once with the dispatch context by the component's shared response helper; applications read the returned action but do not need to manually update a second focus/hover engine. Runtime does not execute the returned application action.

A Scene implementation supplies update and read-only draw callbacks for a generic consumer. Scene is not a universal Widget trait or a product router. Stateless widgets have no persistent state solely to satisfy Scene. A separate optional TerminalSession adapter owns raw-mode/crossterm integration.

## Deliberate API decisions relative to the refactoring references

| Decision | Rationale |
|---|---|
| Retain typed props/state/update/draw, stable keys, borrowed models, layers and part customization | These are the intended architectural direction |
| `Theme::termrock()` instead of a Junie-branded default constructor | Rename library identity without changing oracle tokens |
| Small Scene driver instead of importing a complete existing application/runtime facade | Only generic component dispatch/presentation is needed now |
| Explicit source Revision on dynamic collections and controlled editor inputs | Make stale projections/draft conflicts a documented caller contract |
| Secret-specialized text state/actions | Avoid an unsafe universal Clone/Debug state requirement |
| Table absorbed into Grid; wrappers share engines | Preserve behavior with fewer independent mechanisms |
| TerminalView is a new borrowed-cell adapter API | Cover future terminal-pane presentation without a parser/PTY product |
| Paper/custom themes and newly added robustness cases use an Extension lane | No invented baseline provenance |

The exact public spelling can change once before API freeze in P1 when external usage probes are compiled. Changes must update this reference and source-family mapping. After that, component workers may not invent incompatible per-widget response shapes or silently widen the public facade.
