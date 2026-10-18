# Input, responses, and effective key bindings

**Foundation F02 · canonical owner of normalized input, semantic response metadata, and binding resolution**

**Legacy family:** C02.

This page defines how Termrock turns terminal events into typed component actions and shared command metadata. It owns the contract between caller, component update code, and runtime dispatch. Components may describe their key-specific behavior, but they do not invent alternate event/result protocols.

Termrock is the in-place future of this repository's `termrock-implementation` branch. The frozen oracle is commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Current source uses `Outcome` and `Input` in [`src/core/event.rs`](../../src/core/event.rs), and `keyhint`/`hintbar` render legacy hint data in [`src/widgets/keyhint.rs`](../../src/widgets/keyhint.rs) and [`src/widgets/hintbar.rs`](../../src/widgets/hintbar.rs). Those names describe the migration starting point; the target public response and binding vocabulary is below. `RenderCtx` is likewise a current implementation name, not a target event API.

## Source and oracle references

- Migration input: `termrock-library-spec/foundations/input-actions.md`, the F02 entry in `termrock-library-spec/reference/foundations.json`, and the input sections of `termrock-library-spec/PUBLIC-API.md`/`reference/TYPES.md`.
- Frozen source: [`src/core/event.rs`](../../src/core/event.rs), [`src/widgets/keyhint.rs`](../../src/widgets/keyhint.rs), and [`src/widgets/hintbar.rs`](../../src/widgets/hintbar.rs) at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.
- Identity ownership: [`identity.md`](identity.md).
- Runtime dispatch and phase flow: [`runtime.md`](runtime.md).
- Component update/draw boundary: [`../api/public-api.md`](../api/public-api.md) and [`../foundations/input-actions.md`](../foundations/input-actions.md).
- Interaction parity and exact action observations: [`../design/interaction-contract.md`](../design/interaction-contract.md) and [`../verification/interaction-parity.md`](../verification/interaction-parity.md).

## Target public vocabulary

```rust
enum Flow { Bubble, Consumed }
enum Invalidate { None, Paint, Layout }

enum ActivationOrigin { Keyboard, Pointer, Programmatic }
struct Activated { origin: ActivationOrigin }
struct ValueChanged<T> { value: T, origin: ActivationOrigin }

struct Binding<'a> {
    action: ActionKey,
    chord: Option<Chord>,
    label: &'a str,
    enabled: bool,
    visible: bool,
    priority: u16,
}

BindingView::resolve(scope: ScopeId, bindings: &[Binding<'_>]) -> BindingView
Cx::intents(&mut self, owner: Id) -> IntentIter<'_>
```

The canonical [`Response<A>` shape and field names](../api/types.md#typed-response) live in the public type dictionary: `id` identifies the semantic owner and `state` carries derived `VisualState`. This foundation defines response flow and dispatch behavior, not a second response representation. A response has at most one caller-facing typed action for one dispatched input or update cause. `Flow` answers whether routing continues. `Invalidate` answers whether a repaint or layout publication is needed. Visual state and action metadata are observations, not alternate side effects. These axes are independent: a boundary wheel event may be consumed without painting, while a tick may request paint without a business action.

The shared type dictionary defines `Input`, `Intent`, `InputToken`, `Chord`, `Binding`, `BindingView`, `ScopeId`, `UpdateCause`, `VisualState`, and `Response<A>` in [`../api/types.md`](../api/types.md). The target API keeps typed actions; it does not erase them to strings or `Any`.

## Normalization boundary

The host/session adapter normalizes terminal input exactly once before component dispatch. It preserves:

- key code, modifiers, repeat/release information, and the original event order;
- pointer coordinates and down/up/drag/move/wheel distinctions;
- paste boundaries and text payloads;
- resize dimensions and tick/model-change causes;
- an optional opaque `InputToken` when a host needs lossless forwarding of original bytes.

Normalization must not synthesize business actions. A token cannot reconstruct bytes already discarded by a decoder. Session/PTY concerns remain outside this foundation and the generic Termrock core.

Current `Input::from_crossterm` maps press/repeat keys, supported mouse events, resize, paste, and omits unsupported terminal events. During refactoring, preserve the baseline observable behavior while moving normalization behind the target API. Release handling, repeat policy, and any source-specific modifier exceptions remain explicit component/runtime policy; they cannot be inferred by treating every repeated or released event as a fresh activation.

## Dispatch and response lifecycle

For each update cause:

1. The runtime selects the top eligible layer and key scope, then resolves normalized input to owner intents.
2. The component's `update` receives `Cx`, caller-owned durable state, and borrowed props/model as required.
3. The component applies only its semantic state transition and returns one typed `Response<A>`.
4. Runtime requests such as focus traversal, layer dismissal, capture release, and geometry invalidation are recorded through `Cx`/runtime metadata; they are not duplicated as domain actions.
5. The caller reads the typed action and applies domain changes before the next frame. The component does not persist, perform IO, call providers, or execute the action itself.

Consumption, state change, invalidation, and business intent remain separate. Replayed or repeated releases must not activate twice. If a component changes state but has no domain action, it may return `Consumed` plus `Paint`/`Layout`. If a boundary event is handled solely to prevent propagation, it may return `Consumed` plus `None`.

The runtime owns focus, hover, pointer capture, layer scope, and activation feedback. Stable control identity comes from [`identity.md`](identity.md); frame and time rules come from [`runtime.md`](runtime.md). Component-specific contracts own which key means cancel, commit, navigate, or accept. Do not add a global shortcut that preempts an editor or modal.

## Effective bindings

One binding catalog feeds handlers and every presentation of a command:

- component/keymap resolution;
- `KeyHint` and `HintBar` metadata;
- menus and menu bars;
- command palette and completion metadata;
- help overlays and application context hints.

`BindingView::resolve` evaluates scopes from most specific to least specific. Modal and editor scopes win over ordinary application commands. A global emergency quit exists only when the runtime/application policy explicitly declares it. Remapping a chord updates both dispatch and all displayed hints from the same effective catalog; no string-to-key round trip or duplicated hard-coded hint list is allowed. Disabled or hidden bindings remain available as metadata only when their component contract says so.

## Intent and text ownership

An intent is an owner-resolved semantic input, not a raw key string. Text editors receive text/paste before background character shortcuts. Modal paste is offered to the top applicable modal before page content. The existing baseline's TextInput and TextArea intentionally differ in Escape and paste behavior; shared normalization must preserve those documented component policies rather than forcing one universal editor rule. See [`../foundations/text.md`](text.md) and the per-component contracts.

Pointer activation follows the runtime gesture grammar: down establishes a possible target, drag may update capture, and up activates only if the stable target is still eligible and the release is inside. A pointer and keyboard path aimed at the same control produce the same typed action target with distinct `ActivationOrigin` metadata. Click-through after outside dismissal is forbidden.

## Required proof and parity coverage

The implementation and conformance suites must include:

- keyboard and pointer activation producing one action for the same stable target with correct origins;
- repeat-key behavior and duplicate-release suppression;
- consumed-without-repaint and repaint-without-action cases;
- modal-first paste and editor-first typing, including unassigned `y`/`n`/`q` characters;
- remapping changing the handler and every rendered chord together;
- disabled/hidden bindings not dispatching actions;
- outside pointer dismissal not activating content below;
- at most one caller-facing action per input/cause, even when runtime focus/layer requests also occur;
- typed action target and action count included in exact interaction traces.

Visible feedback is verified by components/composed fixtures against the frozen snapshots: focus gutter, hover suppression, pressed/activation phase, selected/current markers, and hint truncation belong to the visual contract. This foundation does not invent an independent response screenshot. Expected artifacts remain in the trusted oracle lane; candidate code cannot approve its own baseline.

## Implementation acceptance

P1 is complete only when input is normalized once, `Response<A>` cleanly separates flow/invalidation/state/action, effective bindings are shared by dispatch and presentation, and typed actions remain visible through the public facade. Legacy `Outcome` may be an internal migration adapter for a short transition, but it is not a future compatibility promise. Do not introduce a universal boxed `Widget` trait, a render-and-update `show()` API, or an untyped event bus to make signatures look uniform.
