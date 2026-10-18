# PickerChain

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-overlays.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W20 · Group: Overlays · Phase: P4.

## Purpose and exclusions

Canonical Termrock contract for W20. `PickerChain` composes the shared Picker mechanism across keyed stages; it does not know any backend or product domain.

The caller owns stage availability, keyed result data, readiness and asynchronous generation. `PickerChain` remembers a stable stage path plus each stage's query, highlighted key and scroll, and returns stage/key actions. It does not know vaults, providers, directories, credentials, network state or backend loaders.

## Public API

```rust
PickerChain::new(
    id: Id,
    stages: &'a [PickerStage<'a>],
    revision: Revision,
) -> PickerChain<'a>;

PickerChain::current(ItemKey)
    .readiness(Readiness<'a>)
    .patch(StylePatch);

PickerChain::update(&self, cx: &mut Cx<'_>, state: &mut PickerChainState)
    -> Response<PickerChainAction>;
PickerChain::draw(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &PickerChainState,
) -> Rect;
PickerChain::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size;
```

`PickerChainState` owns stable stage path and per-stage query/cursor/scroll. No backend result, credential or loader state is stored. Typed actions are `Accept { stage: ItemKey, key: ItemKey }`, `Back`, `Retry { stage: ItemKey }` and `Cancel`. Read-only current stage/result observations are allowed.

## Ordinary use

The picker-chain flows in Jackin Preview modals are the baseline consumers.

The caller supplies borrowed stages, a source revision, and readiness, then applies the typed `PickerChainAction` results.

## Ownership

- Stage and result data are borrowed caller props. Caller generation/revision checks decide whether returned asynchronous data is current; the chain accepts only the supplied accepted revision.
- `Back` restores the previous stage's query, highlighted stable key and scroll. `Cancel` exits without resetting a dirty underlying form or other caller state.
- A changed earlier choice invalidates incompatible later-stage view state. It must clear or reinitialize later stages according to the caller's stage revision; it must never bind an old key to a new result.
- Retry emits the stable stage key. Loading, partial, error and retry are presentation/readiness states, not backend behavior.
- `update` performs stage navigation and typed requests. `draw` is read-only and composes one Picker surface per current stage; `measure` is pure. Runtime owns modal layer, focus trap, capture, placement, time and restoration.

Shared dependencies: [`picker`](./picker.md), [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`collections`](../foundations/collections.md), [`author`](../foundations/author.md), and [`conformance`](../foundations/conformance.md).

## Customization

Styled parts are `container`, `breadcrumb`, `title`, `query`, `row`, `footer`, `empty` and `status`. Use [`theme`](../foundations/theme.md) and Picker styling; no chain-local palette or query engine.

Preserve the baseline modal surface, breadcrumb/stage path, query field, row hierarchy, status/loading/error/empty text, footer actions, clipping, backdrop, focus trap and narrow-width collapse. Breadcrumbs may collapse at narrow widths only according to the specified baseline recipe. Stage-specific result styling remains the Picker recipe.

## Behavior

- The current stage owns query focus and result navigation through Picker semantics. Enter emits `Accept` with both stable stage and result keys. Search-disabled stages retain navigation/dismiss behavior.
- Back (including empty-query Backspace where supplied) rewinds one stage and restores its durable view state. Cancel closes the chain and restores the prior owner focus without clearing that owner's drafts.
- Retry returns the stable stage key. A stale completion for an abandoned stage is ignored by caller generation checks and cannot alter selection, breadcrumb or scroll.
- Pointer row activation uses completed captured gestures. Outside click/Escape follows the runtime modal policy. Wheel/thumb drag stay in the current stage and do not transfer focus. Resize reanchors/clamps while preserving stage path.
- Keyboard activity suppresses stale hover until pointer motion. Disabled/unavailable rows cannot activate; changed earlier choices invalidate later keys deterministically.
- Where a stage action has baseline feedback, sample the 140 ms window and expiry explicitly; redraw frequency never advances it.

## Visual matrix

| Axis | Required cases |
|---|---|
| Geometry | normal, zero/tiny area, non-zero origin, exact fit, one-cell short, long Unicode content, narrow breadcrumb collapse |
| Focus/hover | unfocused, focused query/row, hovered, focus+hover, keyboard-suppressed hover, pointer restoration |
| Activation | stage/result pointer down/held/release inside/outside, removed/disabled target, keyboard accept, applicable feedback |
| Source/readiness | empty, ready, selected differs from cursor, disabled entry, insert/remove/reorder/filter, stale target, loading/partial/error/retry at every stage |
| Scroll/layer | no overflow/start/middle/end, wheel boundary, thumb drag, resize/reanchor, edge fade/protected row; closed/open/nested/Escape/outside/owner removed/focus restore/modal-first-paste |

Only states supplied by a stage apply. Shared dimensions/capabilities are in [`../verification/visual-parity.md`](../verification/visual-parity.md).

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen composition (legacy family C29) is represented by the picker-chain flows in [`src/bin/jackin_preview/screens/modals.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/modals.rs), with Picker behavior from [`src/widgets/picker.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/picker.rs). Current source identifiers are legacy evidence only.

Use [`../reference/capture-plans/picker-chain.json`](../reference/capture-plans/picker-chain.json). Bind expected output before candidate testing with [`../verification/oracle-and-provenance.md`](../verification/oracle-and-provenance.md). Compare exact cells/continuation, styles/modifiers supported by the tool, cursor, focus/capture/layer owner, stage path, query drafts, selected keys, readiness and typed action count/target. Invalid oracle setup is blocked.

| Case | Required observation |
|---|---|
| W20-01 | Stage 1 → stage 2 → Back restores query, cursor key and scroll. |
| W20-02 | Loading/error/retry is represented at each applicable stage. |
| W20-03 | Stale completion for an abandoned stage produces no selection change. |
| W20-04 | Nested form retains all drafts after chain Cancel. |
| W20-05 | Breadcrumb collapse at narrow widths preserves stage identity and action reachability. |

Negative tests must prove: no credentials/backend/network behavior; stale generation cannot mutate an abandoned stage; Back restores the prior stage by key; changed earlier choices cannot rebind later state; Cancel cannot clear the underlying form; disabled/stale results cannot activate; draw cannot advance stages or mutate drafts; outside/Escape focus restoration is stable; and one shared Picker engine handles each stage.

## Rejected use

Forbidden: painting a stage breadcrumb with rows and advancing stages in preview instead of using PickerChain.

```rust
// Forbidden: preview paints its own stages and owns the path.
ui.paint_str(row, 0, "vault › folder");
if clicked { stage = 1; }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and padded strings that simulate menus. Use `PickerChain::new` with stable stage keys and handle the typed `PickerChainAction`.

## Known gaps

- Capture plan W20 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-002 historical modal screens](../implementation/code-remediation-backlog.md): pending; the Jackin Preview modal picker-chain flows still use historical paths.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
