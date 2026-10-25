# Termrock API rules

## Status

These rules specify the PR #38 target. They are not a claim that current code satisfies every rule.

Target source: `e08ac57d10ae54c87337eae0be08b1f5e8f4bda4`.
Product reference: `5b3dbd7ae799f325efbb4f4cc11b2f5518dec948`.

Use [ARCHITECTURE.md](ARCHITECTURE.md) for owners and feature rules. Use [REVIEW.md](REVIEW.md) for findings and [PLAN.md](PLAN.md) for sequence. Implement the units in [TASKS.md](TASKS.md). Prove them through [VALIDATION.md](VALIDATION.md). Keep the public inventory in [COMPONENT-MATRIX.md](COMPONENT-MATRIX.md). Follow [AGENT-RUNBOOK.md](AGENT-RUNBOOK.md), [SOURCES.md](SOURCES.md), and [TESTING-RULES.md](TESTING-RULES.md). The testing rules govern conflicts about tests and source comments. Keep the other API contracts in this file.

## 1. Keep one normal path and one advanced path

The normal path constructs a component, supplies caller-owned state, handles a typed action, and composes the result. It must not require raw key matching or hit-region construction for standard behavior.

The advanced path uses `termrock::author`. It permits custom painting, parts, row adapters, bindings, and composition leaves. It keeps the same runtime rules as built-in components.

Keep current public root aliases during migration. Prefer family namespaces in new examples. Do not export every internal helper into a new prelude. Add a small prelude only when a compiled beginner example shows that it removes real import noise.

## 2. Keep the current component shapes

Do not add a universal Widget trait. Current components have different state requirements.

Keep these existing API patterns:

| Operation | Existing shape |
|---|---|
| Construct a Button | `Button::new(id, label).variant(Variant::PRIMARY).disabled(disabled).checked(checked)` |
| Update a Button | `update(&self, cx: &mut Cx<'_>) -> Response<Activated>` |
| Update a List | `update(&self, cx: &mut Cx<'_>, state: &mut ListState, items: &[T]) -> Response<ListAction>` |
| Draw a List | `draw(&self, ui: &mut Ui<'_>, area: Rect, state: &ListState, items: &[T]) -> Rect` |
| Measure | `measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size` |

These signatures describe existing patterns. Use the [current public consumer source](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-consumer-example/src/app.rs) as the starting example. Update and compile it during implementation.

A Button can have controlled checked state without a ButtonState. A List has persistent cursor, chosen, checked, and scroll state. A Form borrows field values through FormData. Keep these distinctions explicit.

Keep `new` constructors small. Use builders for optional policy. Give defaults a product meaning. Do not add booleans whose interactions are unclear. Use the existing policy enums when more than two states are possible.

## 3. Keep caller-owned state and borrowed data

The caller owns domain data. A component borrows it for update, measurement, or draw. A component may own explicit interaction state when the caller stores that state.

Do not clone an entire source merely to render visible rows. Keep Grid cell views, Diff rows, text spans, and labels borrowed where the current API supports this.

Keep callbacks borrowed for their actual lifetime. Do not require `'static`, Send, Sync, or cloning without a concrete owner requirement. A callback may capture a local borrowed value. It must not retain Ui, Cx, Buffer references, or other frame borrows after the call.

Keep the present static binding-table and Overlay rule-storage contracts unless a separate task changes them. These storage contracts do not require ordinary row or slot callbacks to be static.

Use generic callbacks where the existing API already needs their concrete type. Use borrowed trait objects for heterogeneous slots where they simplify ownership. Do not replace every callback with Box or Arc for uniformity.

If an owned convenience API is necessary, add it beside the borrowed path. State its allocation and lifetime behavior. Prove the need with a real consumer.

## 4. Keep semantic state out of measurement and draw

Update may change semantic component state. It may request a domain action through a typed response.

Measure must not paint, handle input, change domain state, or register focus and hit regions.

Draw may paint and register frame facts. It may update diagnostics and runtime-derived caches. It must not commit an edit, cancel an edit, change chosen data, or run a domain action.

Do not describe draw as entirely free of effects. Ui is mutable because frame registration is necessary. Test the semantic state separately from the runtime frame state.

Repeated rendering of unchanged state, time, theme, and geometry must preserve semantic state. Reference rendering must remain inert. Synthetic focused or pressed flags do not grant live interaction permission.

## 5. Preserve Response channels

Keep `Response<A>`. Keep Flow and Invalidate separate.

- Ignored input can require repaint.
- Consumed input can require no repaint.
- An action is separate from both.
- `Invalidate::Layout` is stronger than Paint.
- StateFlags describe the named component owner.

Use the existing `map_action`, `on_action`, `on_activated`, and `erase` methods. They preserve response metadata. Do not add a second action-drop API.

Add one small method only where the parent must preserve its own action and merge child effects. Proposed signature:

| Owner | Proposed method |
|---|---|
| `Response<A>` | `merge_effects<B>(self, other: &Response<B>) -> Self` |

Its exact contract is:

1. Preserve `self.id`, `self.state`, and `self.action`.
2. Set flow to the maximum of both flow values.
3. Set invalidation to the maximum of both invalidation values.
4. Do not copy or erase `other.action`.
5. Do not merge owner-specific StateFlags.

Migrate the completing-editor and form-dialog combiners through this contract. The parent still selects its domain action. Do not add a generic action bus.

Keep existing `Response<()>` BitOr semantics during this change. Do not silently change its state or owner behavior. Document its aggregation limits. Read individual component state before aggregation.

Test all Flow and Invalidate combinations. Include Paint plus Layout in both orders. Preserve typed action, owner, and state. Test that erase and action mapping preserve metadata.

## 6. Validate before commit

A fallible operation must validate first or leave the prior state intact. Preserve the value, selection, cursor, and revision when a commit fails.

Use typed errors for invalid data, stale frame tokens, depth failures, and invalid state transitions. Add Display and Error implementations where callers need them. Do not flatten every failure into a string.

Do not add Result to an infallible builder only for consistency. Keep documented layout clamps. Do not silently clamp identity failures or turn an invalid composition into empty content.

An error path must not partially register focus or hits. A failed edit must not leak its candidate value through copy, logs, or secret rendering.

## 7. Keep the prepared-frame protocol

Retain these steps:

1. Update semantic state or explicit time.
2. Prepare against the current area.
3. Project with a matching runtime, generation, and area token.
4. Complete transport.
5. Acknowledge that exact presentation.
6. Accept input under the published frame contract.

Reject stale or mismatched tokens. Retry preparation when the actual area changes. App and theme mutation invalidate prepared output. Do not accept input against unpublished geometry.

Keep monotonic time explicit. Keep Moment and ClockError available without a backend feature. A backward clock request must preserve the current time and return an error.

Keep focus scopes, modal order, capture, clipboard ownership, and wheel routing in runtime. A widget uses those services. An application maps typed actions into domain work.

### Keyboard delivery snapshot

T09 adds `KeyboardDelivery` in `termrock_core::keys::keyboard`. This is a small `Clone + Copy + Debug + Default + PartialEq + Eq` value. It holds two private Boolean answers: Alt delivery and Super delivery. It contains no protocol stack, environment cache, observation counter, or terminal handle.

Use these exact public methods:

| Owner | Proposed method |
|---|---|
| `KeyboardDelivery` | `new() -> Self` |
| `KeyboardDelivery` | `with_alt(self, delivered: bool) -> Self` |
| `KeyboardDelivery` | `with_super(self, delivered: bool) -> Self` |
| `KeyboardDelivery` | `alt_delivered(self) -> bool` |
| `KeyboardDelivery` | `super_delivered(self) -> bool` |
| `KeyboardDelivery` | `allows(self, chord: Chord) -> bool` |
| `KeyboardDelivery` | `label_deliverable(self, label: &str) -> bool` |
| `Runtime<A>` | `keyboard_delivery(&self) -> KeyboardDelivery` |
| `Runtime<A>` | `set_keyboard_delivery(&mut self, delivery: KeyboardDelivery)` |
| `Cx<'_>`, `Ui<'_>` | `keyboard_delivery(&self) -> KeyboardDelivery` |
| `Footer` | `compose_with_delivery(&self, delivery: KeyboardDelivery) -> Vec<Hint>` |

Use const methods where supported. `new()` and Default allow neither Alt nor Super. The host supplies confirmed answers. Ordinary chords remain allowed. `allows` checks modifier bits, not formatted text. It requires each requested Alt or Super modifier to be deliverable.

Keep `label_deliverable` only for existing literal hint labels. Move the existing label policy into this pure method. Preserve its declared pointer-label rules. New typed hints and tooltip shortcuts use `allows` before formatting.

Store one snapshot in each Runtime. Changing it invalidates prepared output and affected hint and tooltip caches. Request layout so changed shortcut width gets measured again. Setting the same value has no effect. This setter must not enable a binding, emit input, or change focus.

T20 derives the snapshot from its existing `AltKeys` policy, explicit probe answers, observations, and actual protocol state. Preserve the current refusal and ambiguous-Alt rules. Read the environment at the session adapter boundary. A custom host can supply a snapshot directly.

Keep the actual protocol lifetime in one session-owned `KeyboardNegotiation` instance. Keep core ProtocolState as its neutral value model. Do not copy its pushed or enabled state into Runtime. The snapshot describes delivery only. Refresh it after a probe, accepted observation, suspend, or resume changes that answer.

T12 reads this snapshot for tooltip shortcuts. T19 reads it for shell hints. `Footer::compose_with_delivery` keeps the existing composition order and changes only the delivery input. Keep `compose()` temporarily as a conservative-default wrapper. Update first-party callers to pass the owning runtime's snapshot.

Retire the core and shell global capability accessors through explicit migration entries. T20 removes core's global protocol and delivery mirrors. T19 removes shell's global delivery mirror. Preserve pure encoders, decoders, protocol values, and `AltKeys::from_env`.

Test two runtimes with opposite delivery snapshots. A mutation in one must not alter the other. Test Alt, Super, both modifiers, ordinary chords, remapping, and the literal pointer-label cases. Test that an equal setter preserves a prepared token. Test that a changed setter rejects the old token and refreshes hint width. Test explicit refusal, ambiguous Alt observations, and suspend/resume at the session boundary.

## 8. Use one composition engine

Keep layout plans in termrock-layout. Keep measured execution and clipping in runtime. Keep typed component adapters in the facade. Keep generic multi-family blocks in the facade or their established owner.

An adapter supplies placement only. It calls the component's existing measure and draw paths. It does not create another selection state, row renderer, hit map, or input handler.

Keep Group row, column, and wrap APIs. Keep Panel, SplitPane, MasterDetail, and AppShell composition. Do not introduce a second declarative tree language.

Use existing `composition::menu_bar` and `composition::status_bar` in the public consumer. Remove its handwritten MenuLeaf and StatusLeaf adapters after equivalent reviewed functional proof passes.

### Nested validation

Extend the existing CompositionLeaf trait with a backward-compatible validation hook. Proposed shape:

| Owner | Proposed method |
|---|---|
| `CompositionLeaf` | `validate_composition(&self, parent_depth: usize) -> Result<(), CompositionError>` |

The default implementation accepts an atomic leaf. Built-in Child, ShellSlots, and other nested adapters validate their actual children. A custom nested leaf must implement the same contract.

Validate before measurement, draw, or registration. Use saturating or checked depth arithmetic. Preserve the bounded depth rule and safe drop. Propagate errors through checked public show and measure calls.

Do not discard a Child error when it is passed as `&dyn CompositionLeaf`. Do not return zero size as the normal error response. For a direct low-level invalid draw, emit a diagnostic and keep the subtree inert.

Keep the existing infallible Measure trait. A checked composition entry point validates before it calls that trait. A direct caller must validate a nested leaf first. Retain the legacy empty-size fallback only for an invalid direct compatibility call. This fallback is not a successful checked result. Do not add interior mutation to immutable Ui only to report this misuse.

Test the depth boundary through every built-in wrapper. Verify that rejection leaves buffer, hit map, focus ring, and bindings unchanged. Include owned ContentSlot and direct trait coercion cases.

## 9. Make data changes explicit

Use domain ItemKey values for reorderable data. Use positional keys only when position is the identity.

Keep cursor, chosen item, checked set, expansion, and edit target separate. Preserve each surviving key across sort, filter, resize, and replacement. Define fallback when a key disappears.

The current CollectionCore shortcut compares length and endpoint keys. It cannot detect all internal mutations. Current callers can invoke `invalidate()`. Keep that path and document it precisely.

T14 uses the existing `termrock_core::Revision` newtype. Add this builder to List, NavList, and ChipBar:

| Owners | Proposed const builder |
|---|---|
| `List`, `NavList`, `ChipBar` | `source_revision(self, revision: Revision) -> Self` |

Keep each component's current update signature. Store the optional revision in its configuration. Pass it through to one CollectionCore path:

| Owner | Proposed method |
|---|---|
| `CollectionCore` | `reconcile_with_revision(&mut self, revision: Revision, len: usize, extent: usize, key: impl Fn(usize) -> ItemKey, enabled: impl Fn(usize) -> bool) -> Reconciliation` |

Use the existing reconciliation implementation underneath this path. Do not copy its cursor, checked-set, or fallback algorithms. Keep existing reconcile methods and `invalidate()`. They remain the supported legacy path.

The data owner advances the revision when ordered keys or eligibility change. The revision applies to one collection owner and state. A replacement source must advance it or call `invalidate()`. Reusing a revision for different ordered data violates this contract.

Compare extent separately from source identity. Update and clamp scroll extent even when source revision and endpoint keys match. Include the transition between revisioned and legacy paths in the cache stamp. It must not reuse an unrelated warm stamp.

`Revision::next()` saturates at `u64::MAX`. Do not treat that value as proof that data stayed unchanged. At the saturated value, run reconciliation instead of the revision shortcut. Keep this fallback local to the exhausted revision. Do not change the Revision type or its current public methods.

Keep Tree's existing source lineage and query revisions. Keep GridModel::revision. This task does not replace either contract with the new collection builder.

Do not claim that an unchanged endpoint stamp proves unchanged data. Do not replace the warm shortcut with an unconditional full scan. With an unchanged valid revision, preserve the current bounded key-access contract.

Preserve these observable outcomes: internal reorder and replacement with unchanged length and endpoints; removal of a checked key; changed eligibility and row extent; semantic key and cached index; scroll extent and fallback; invalidation; revision-mode transitions; and saturated revisions. T00 records existing warm-path counter tests as historical source evidence. Do not retain or run those tests after T37. After T38, use reviewed functional scenarios to prove the unchanged-revision bounded key-access contract and these state changes.

## 10. Preserve bounded work

Keep Tree's runtime-derived index separate from TreeState. Preserve clone lineage, equal-generation replacement, and saturation behavior. Do not copy the derived index into semantic state.

Keep GridModel's borrowed cell and row-key API. Keep structural holes as None. Do not call decoration, action, or edit hooks for a missing cell. Keep sort requests distinct from domain ordering.

Keep deterministic access, visit, callback, and allocation bounds as observable performance contracts. T00 records existing warm-cache tests as historical source evidence. Do not retain or run those tests after T37. After T38, prove the required bounds with reviewed scenarios. State which operation pays initial O(n) preparation and which changes invalidate it.

A visible draw must not scan the full collection without an explicit justified contract. A content revision may require a rebuild. A paint-only change must not force that rebuild.

Use wall-clock performance checks only for the remaining measured risk. Keep unrelated performance subjects separate when the reviewed test architecture requires separate targets.

## 11. Keep customization composable

Retain the theme, scoped overlay, instance patch, part patch, and slot paths. Give patch merge one pure implementation owner.

The intended precedence is the existing theme resolution followed by scoped overrides and instance/part overrides. Verify the exact existing resolution order before moving code. Preserve `Slot::Inherit`, `Slot::Set`, and `Slot::Clear` as distinct operations.

Apply matching part patches in declaration order. Preserve each explicit style channel. Do not reset unrelated channels when a caller overrides one channel.

A visual slot uses the parent's clipped area. It must preserve component identity and standard interaction unless its documented contract explicitly replaces that behavior. It must not register an extra live target under reference rendering.

Test sibling isolation. Test two parts of one component. Test repeated patches to one part. Test clear and inherit separately. Test a borrowed local callback. Test Unicode, narrow geometry, and each supported color capability.

Keep paint and hit geometry derived from one layout result. Do not fix clipping separately in every application.

## 12. Restore reference behavior without restoring its structure

PR #26 Button has tooltip, shortcut, and mnemonic behavior. PR #38 Button lacks those builders and fields.

Add the missing behavior through shared owners. Keep the current borrowed label, controlled checked prop, runtime input, and style parts. Do not restore a mutable public `area`, raw Buffer rendering, or per-widget raw key handling.

Use these additive Button builders:

| Owner | Proposed const builder |
|---|---|
| `Button<'a>` | `mnemonic(self, key: char) -> Self` |
| `Button<'a>` | `tooltip(self, text: &'a str) -> Self` |
| `Button<'a>` | `tooltip_config(self, tip: Tooltip<'a>) -> Self` |

### Mnemonic display

`mnemonic` selects a display mark. It does not register a binding or consume a key. Use the first matching label grapheme. Match ASCII letters without case sensitivity. Match other characters exactly. Underline the complete displayed grapheme. Do not use a byte offset as a cell position.

If no grapheme matches, show no mark. Preserve the reference suppression rules for a truncated label, a busy button, and a checked/toggle marker. Keep ordinary focus, disabled, and pressed treatment unchanged. Activation still uses the published Binding table.

### Borrowed tooltip request

Runtime owns `Tooltip<'a>`. It contains borrowed text, an optional ActionKey, and presentation policy. Add these configuration methods:

| Owner | Proposed constructor or builder |
|---|---|
| `Tooltip<'a>` | `new(text: &'a str) -> Self` |
| `Tooltip<'a>` | `action(self, action: ActionKey) -> Self` |
| `Tooltip<'a>` | `enabled(self, enabled: bool) -> Self` |
| `Tooltip<'a>` | `delay(self, delay: core::time::Duration) -> Self` |
| `Tooltip<'a>` | `max_width(self, cells: u16) -> Self` |

This table specifies method shapes. It is not implementation code. Use private fields and const builders where the values permit them. Keep defaults enabled, 600 ms delay, and 48 cells maximum width. These defaults come from the reference Tooltips policy.

`Button::tooltip(text)` uses that default request and shows text only. An absent action means no shortcut. `Tooltip::action(ActionKey)` opts into a shortcut for that action. `tooltip_config` supplies this request and any policy changes. Empty text or disabled tooltip policy publishes no visible tip.

Controls passes its existing Binding table with the request. Runtime owns this operation:

| Owner | Proposed method |
|---|---|
| `Ui<'_>` | `tooltip<C: Copy + 'static>(&mut self, owner: Id, anchor: Rect, tip: Tooltip<'_>, bindings: &'static [Binding<C>])` |

When the request has an action, find that ActionKey in the supplied table. Resolve its action and default chord through `Ui::effective_chord`. Apply the runtime's current terminal-deliverability policy. If the request has no action, omit the shortcut. Also omit it when the action is missing or no effective deliverable chord exists. Do not add `shortcut(&str)` or invent a display chord.

The supplied table is the same table that the component uses for activation. Reading it must not publish bindings or enable an unfocused command. A hovered owner can show its remapped shortcut without keyboard focus. Do not require an entry in the focused-owner binding registry.

Use the existing LayerKind::Tooltip, anchor/clipping logic, monotonic time, and binding services. This is one request type and one runtime presentation path. Do not add a second overlay framework or a controls-to-overlays dependency.

Borrowed text is valid only during the draw callback. Runtime may retain owner, anchor, policy, and timing metadata. It must not retain the borrowed text or a closure that borrows it. Paint or project tooltip text during the current frame. Keep only the resulting owned frame output after the callback.

Runtime owns hover delay and dismissal state per runtime and owner. Do not copy the reference's thread-local state. Preserve one visible tooltip, hover transfer from owner to tip, Escape dismissal, and press dismissal. After dismissal, require the reference's leave-and-reenter condition before showing it again. A tooltip must not move keyboard focus or trap background input.

The tooltip must not make its own owner inactive. Its passive layer differs from a modal or popover that covers that owner. Keep the owner eligible during hover transfer to its tip.

Reference rendering and covered owners publish no live tooltip. Resize reclamps its geometry. Owner removal clears its pending and visible state. Register its timer through the existing earliest-deadline mechanism. Drawing does not advance the timer or change semantic Button state.

Confirm the full #26 behavior inventory before closing T12. Test Unicode mnemonic positions, repeated letters, missing letters, narrow truncation, and busy and checked suppression. Test disabled owners, remapped and unbound shortcuts, top-layer ownership, and reference inertness. Test hover transfer, dismissal, resize, deterministic timing, and Mono rendering.

Test that `Button::tooltip(text)` shows no shortcut by default. Add an unfocused-hover test with an explicitly configured action and its remapped activation chord. The tooltip must show that chord without publishing a command for the hovered owner. Keyboard activation must still reach the focused owner.

## 13. Complete the public inventory

Keep W01–W45 as requirement IDs. Do not treat them as the complete public API.

The current facade adapter list adds RadioField, Note, TextSection, SelectField, Table, CompletingCodeEditor, FormDialog, AppShell, and MasterDetail. Also inventory SplitBlock, Notice, DerivedHintBar, and TextReport. Inventory all application-specific feedback exports before retirement.

Classify each item as control, block, primitive, descriptor, alias, or adapter. For example, CommandPalette is an alias and Menu is a descriptor. Do not count them as independent render implementations.

Record these fields for each item:

- Stable requirement or inventory ID.
- Canonical implementation owner.
- All supported public paths.
- State and action types.
- Borrowed inputs and callback lifetimes.
- Parts and customization points.
- Data revision or invalidation rule.
- Focus, pointer, keyboard, and layer behavior.
- Measurement and empty/tiny behavior.
- Consumer example and approved functional case IDs.
- Keep, replace, deprecate, or retire disposition.

PropsList is implemented by controls and re-exported by navigation. ScrollRegion is implemented by runtime and re-exported by viewport. Record both facts. Do not move code merely to satisfy a stale owner name.

## 14. Keep examples executable and source comments functional

Keep API prose in maintained Markdown. Follow [TESTING-RULES.md](TESTING-RULES.md) for source comments. Put essential safety and invariant rationale in maintained contracts and design records. Keep only tool-required comments and text allowed by those rules. Use explicit Clap help attributes.

Keep real `.rs` consumer examples. Link them from the Markdown guide. Compile and exercise them through supported public paths after T38 and an approved task-level plan. Do not copy an example into a second code block and let it drift.

The beginner consumer must use only the facade as a normal dependency. It must compose a shell, list, detail view, action buttons, menu, and status. It must support keyboard and pointer input plus a narrow layout.

That consumer must not require custom Rect math, raw event matching, or custom leaf types for existing stock components. Advanced examples may use the author path and must say why.

Keep feature examples outside workspace feature unification when they prove minimal dependency claims. Use production APIs. A testing feature may support approved behavior cases, but it cannot establish production API availability.

## 15. Review each API change as a contract change

Every API task must name the owner, caller, source behavior, target behavior, and migration effect. It must identify a real consumer and a missing or failing functional proof. Plan each test before writing it, as required by [TESTING-RULES.md](TESTING-RULES.md).

Keep working behavior unless the task explicitly changes it. Do not replace a known interaction policy with a simpler one merely to reduce code.

Implementation uses GPT-6 Luna at max effort. GPT-6.1-Sol coordinates and reviews.
Use Sol high for product API, feature, runtime, identity, focus, lifetime, error, and performance changes.
Use Sol medium for test-only plans, diffs, runtime or performance test design, and evidence verification.
Use Sol high if a test review finds a required product architecture or contract change.
Use Sol medium for bounded application work. Use Sol low for text, links, and file-name checks after technical acceptance.
Follow the full assignment table in AGENT-RUNBOOK.md.

No implementation task is complete from compilation alone. It needs approved behavior evidence and the required integration evidence. Final acceptance must name the final commit and exact case scope.

## Source basis

These sources were inspected. Any tests associated with them are historical evidence for T00. They are not active targets after T37 and were not executed in this review.

- [App and runtime contract](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-runtime/src/runtime.rs#L95-L160).
- [Response methods and aggregation](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-core/src/response.rs#L188-L260).
- [Composition leaf error suppression](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock/src/composition.rs#L758-L767).
- [Revision newtype and saturation](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-core/src/id.rs#L13-L27).
- [Collection stamp and reconciliation](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-collections/src/collection/reconcile.rs#L172-L315).
- [Borrowed part style and slot callbacks](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-runtime/src/author.rs#L38-L142).
- [Target Button](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-controls/src/button.rs#L76-L213) and [reference Button](https://github.com/tailrocks/terminal-components-claude/blob/5b3dbd7ae799f325efbb4f4cc11b2f5518dec948/src/widgets/button.rs#L14-L79).
- [Reference tooltip policy and lifecycle](https://github.com/tailrocks/terminal-components-claude/blob/5b3dbd7ae799f325efbb4f4cc11b2f5518dec948/src/ui/tooltip.rs#L49-L193).
- [Core global protocol and delivery mirrors](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-core/src/keys.rs#L220-L306).
- [Existing Alt policy and literal label policy](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-core/src/keys.rs#L313-L423).
- [Duplicate shell delivery globals](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-shell/src/keyboard.rs#L8-L78) and [Footer composition](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-shell/src/footer.rs#L172-L193).
- [Focused-owner publication guard](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-runtime/src/ui/mod.rs#L881-L900).
- [Existing Tooltip layer spec](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-runtime/src/layer.rs#L512-L525) and [effective chord lookup](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/crates/termrock-runtime/src/ui/mod.rs#L1073-L1079).
- [Rust comment and Markdown policy](https://github.com/tailrocks/terminal-components-claude/blob/e08ac57d10ae54c87337eae0be08b1f5e8f4bda4/docs/development.md#L51-L53).
