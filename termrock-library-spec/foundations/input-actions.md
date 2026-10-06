# F02 · Input, responses and effective key bindings

**Scope:** shared generic library/test infrastructure, not a product subsystem.  
**Legacy families:** C02  
**Visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; interface spellings below are proposals.

## Source references

- [src/core/event.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/core/event.rs)
- [src/widgets/keyhint.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/keyhint.rs)
- [src/widgets/hintbar.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/hintbar.rs)

[Target architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) · [refactoring task catalog](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv)

## Proposed public surface

Rustdoc-style declaration notation. This is not a compiled implementation. Read with [PUBLIC-API.md](../PUBLIC-API.md) and the [shared type dictionary](../reference/TYPES.md).

```rust
enum Flow { Bubble, Consumed }
enum Invalidate { None, Paint, Layout }
struct Response<A> { id: Id, flow: Flow, invalidate: Invalidate, state: VisualState, action: Option<A> }
enum ActivationOrigin { Keyboard, Pointer, Programmatic }
struct Activated { origin: ActivationOrigin }
struct ValueChanged<T> { value: T, origin: ActivationOrigin }
struct Binding<'a> { action: ActionKey, chord: Option<Chord>, label: &'a str, enabled: bool, visible: bool, priority: u16 }
BindingView::resolve(scope: ScopeId, bindings: &[Binding<'_>]) -> BindingView
Cx::intents(&mut self, owner: Id) -> IntentIter<'_>
```

## Contract

1. Normalize terminal input once. Preserve modifiers, repeat/release information, paste boundaries and pointer coordinates. InputToken can preserve an opaque link to original bytes when the host needs lossless forwarding.
2. Consumption, visual invalidation, state change and domain intent are independent. A boundary wheel event can be consumed without paint; a display tick can repaint without an action.
3. A widget update processes one dispatched input/cause, yielding at most one caller-facing action. Focus traversal, layer closing and geometry invalidation are runtime requests, not duplicate business actions. Replayed/repeated releases cannot activate twice.
4. Use one effective binding catalog for handlers, hints, menus, command palette and help. Resolve modal/editor scopes before ordinary application commands; global emergency quit is explicit policy. Keep text input from accidentally invoking background y/n, q or other shortcuts.
5. The uniform naming/signature convention does not imply a universal dynamic Widget trait. Keep typed actions; do not erase all actions to strings or Any.

## Required proof

- keyboard and pointer yield same action target with correct origin
- repeat key policy versus duplicate release
- consumed-without-repaint case
- modal-first paste and editor-first typing
- remapping changes handler and every displayed chord together

This foundation has no invented independent hover/pressed screenshot. Its visible effects are proved through the components and composed fixtures that use it. Nonvisual invariants have headless/state/compile-fail/protocol tests. Implementation and independent verification are not executed in this document pack.
