# Wizard

Status: proposed target.
Owner: termrock-forms.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W27 · Group: Composition helpers · Phase: P4 · Legacy family: C34 (`wizard`).
Capture plan: [`../reference/capture-plans/wizard.json`](../reference/capture-plans/wizard.json).
Current source evidence: [`src/bin/jackin_preview/screens/prelude.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/prelude.rs), [`screens/modals.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/modals.rs).

Canonical Termrock contract for W27. `Wizard` is a generic step navigation and
retention helper with a body slot. It is a target Termrock component for the
in-place refactor on `termrock-implementation`; it is not a product workflow.

## Purpose and exclusions

`Wizard` retains a stable current step and visited path while the caller
supplies borrowed step descriptors, per-step validation, child components and
durable field values. It emits typed navigation intent. The caller owns the
meaning of a step, values, persistence, effects and whether a step is eligible.

The body is a slot of real Termrock components. `Wizard` supplies shared
heading/chrome and actions while runtime supplies focus/capture/layer behavior.

Exclusions:

- clone operations, process execution, account/provider calls or persistence;
- domain-specific fields, automatic saving, route transitions or launch logic;
- treating a disabled step as completed or using visual order as identity;
- creating a product-specific prelude screen or redesigning Jackin Preview;
- a universal boxed widget abstraction or semantic work in `draw`.

## Public API

Rustdoc-style target declarations; P1 external-consumer probes freeze the
precise spelling.

```rust
Wizard::new(
    id: Id,
    steps: &'a [WizardStep<'a>],
    revision: Revision,
) -> Wizard<'a>

Wizard::can_advance(self, eligible: bool) -> Self
Wizard::can_finish(self, eligible: bool) -> Self
Wizard::busy(self, busy: bool) -> Self
Wizard::patch(self, patch: StylePatch) -> Self

Wizard::update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut WizardState,
) -> Response<WizardAction>

Wizard::draw<R>(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &WizardState,
    body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
) -> R

Wizard::measure(
    &self,
    cx: &MeasureCx<'_>,
    constraints: Constraints,
) -> Size
```

`steps`, eligibility, busy state and body props are borrowed. Each step has a
stable `ItemKey` and may expose read-only status/validation metadata. The
caller owns child widget state and all values. `WizardState` owns only the
current stable step key and visited path.

```rust
enum WizardAction {
    Next { from: ItemKey },
    Back { from: ItemKey },
    Finish,
    Cancel,
}
```

The action reports intent with stable step identity and activation origin where
the shared response contract requires it. It never persists or executes the
result.

No Rust `Wizard` implementation exists in the frozen source under that name.
The Jackin Preview five-step prelude and its Dialog, ChoiceDialog, Picker and
FileBrowser composition are the visual/interaction oracle. That source still
uses legacy `junie_tui`, `WidgetId`, `Outcome` and app-owned modal types. They
are evidence for parity, not future public API requirements.

## Ordinary use

Consumer recipe: [EX-16 — Jackin workspace prelude composition](../api/consumer-recipes.md) (`proposed_target`).

The frozen Jackin Preview prelude is a five-step modal chain:

1. Source (file browser or Git URL),
2. Destination (choice),
3. Edit destination (prompt/validation),
4. Working directory (picker),
5. Name (prompt/validation).

Titles identify the current step (`step 1 of 5` through `step 5 of 5`). Back
retains prior values, invalid Next leaves the current step open with the error,
and a valid final Create hands the caller a pending workspace. The chain can be
rewound and reopened without losing accepted values. The same target contracts
also cover nested form/choice flows where earlier choices alter later steps.

Evidence includes:

- prelude construction and result handling in
  [`src/bin/jackin_preview/screens/prelude.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/prelude.rs);
- modal chrome and child composition in
  [`src/bin/jackin_preview/screens/modals.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/modals.rs);
- prelude application tests and frozen snapshots under `snapshots/jackin`;
- baseline case `jackin/manager/new_workspace_prelude` at 120x40 and 80x24.

## Ownership

- `update` reconciles `steps` at `Revision`, dispatches child responses and
  applies eligibility/busy policy. Next/Finish emit only when the caller says
  the current value is valid and the action is enabled. Back retains child
  drafts unless the caller explicitly invalidates them.
- Removing or branching a step reconciles by `ItemKey`. A removed current step
  selects the nearest valid retained step according to the shared collection
  rule, and no pending action/focus is transferred to a new occupant.
- `draw` is read-only. It paints step heading/actions and invokes the body slot
  using current immutable props/state. It cannot validate, advance, persist,
  change focus, or mutate child values.
- `measure` is pure and accounts for heading, body slot and action row under
  constraints. Zero/tiny and reduced terminal sizes fail closed without
  panics; child components own their own scroll measurement.
- Runtime owns focus, hover, pointer capture, layer order and feedback time.
  Wizard owns current/visited semantic state only.

Shared dependencies:

- [`form`](./form.md), [`button`](./button.md),
  [`dialog`](./dialog.md), [`picker`](./picker.md),
  [`runtime`](../foundations/runtime.md),
  [`layers`](../foundations/layers.md),
  [`input-actions`](../foundations/input-actions.md),
  [`identity`](../foundations/identity.md),
  [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md),
  [`collections`](../foundations/collections.md),
  [`author`](../foundations/author.md),
  [`conformance`](../foundations/conformance.md).

## Customization

Advertised parts are `container`, `step-heading`, `body` and `actions`. A
supported patch/slot is confined to its declared rectangle and cannot replace
step identity, action ownership, focus/capture, or runtime chrome.

## Behavior

### Focus and keyboard

- Opening a wizard layer focuses the first eligible child in the current step
  and traps focus to the body/actions. Runtime saves and restores the owner
  focus when the wizard closes.
- Next, Back, Finish and Cancel are real action controls. Keyboard traversal
  uses shared focus and binding rules; `Enter` does not bypass validity or
  busy policy. Escape maps to Cancel only when the caller policy permits it.
- The body receives editor/picker/form keys first. A child editing Escape or
  paste must not be interpreted as wizard navigation.
- Nonlinear enabled steps may be navigated only through explicit step actions;
  disabled steps are not silently treated as completed.

### Pointer, hover and activation

- Action buttons and child controls use completed press/release semantics.
  Release outside, removed step, or disabled action cancels activation; there
  is no whole-wizard pressed state or click-through.
- Hover belongs to actual child controls. Keyboard input suppresses stale hover
  until pointer motion restores it. Decorative step headings do not become
  focus stops unless the caller supplies a real navigable step control.
- Read-only child fields remain readable and navigable; disabled fields/actions
  are inert barriers. Busy policy may disable Next/Finish/Cancel as declared,
  but never fabricates completion.

### Resize, reconciliation and motion

- Resize recomputes heading/body/action geometry and preserves current step,
  visited path, child drafts and focus semantics. A smaller terminal may reduce
  body allocation; it must not drop values or change the step key.
- Revisioned step changes reconcile before input interpretation. If an earlier
  choice removes a later step, stale focus, pending activation and later-step
  values are handled by the caller's explicit retention policy; no index reuse
  can submit the new occupant.
- Wizard itself has no timer. Child feedback and caller-provided busy progress
  use shared time/motion policy; paused mode still accepts supplied data and
  completion/cancellation updates.

## Visual matrix

The default recipe must preserve the prelude oracle and each migrated consumer:

- modal/elevated surface, title/step count, body spacing and action row;
- current/visited/available step representation supplied by the recipe;
- child Dialog, Picker, Choice and Form visuals exactly as their own contracts
  specify; Wizard must not paint over them;
- disabled/busy/error status hierarchy, narrow-width clipping and terminal
  minimum/degenerate geometry;
- display-cell-safe Unicode labels and body content;
- semantic colors across truecolor, 256, 16, explicit `none` and `NO_COLOR`;
- no invented animation. Child feedback (such as a button's 140 ms activation)
  follows the shared clock, and paused/reduced motion remains deterministic.

| Axis | Required cases | Applicability |
|---|---|---|
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide | wizard frame/body/actions |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer-motion restoration | real child/action controls |
| Activation | not applicable to whole surface; pointer down/held/release inside/outside, removed/disabled target, keyboard activation and feedback apply to real children | child actions only |
| Navigation | invalid Next, valid Next, Back retention, Finish, Cancel, nonlinear enabled steps | caller step model |
| Data/eligibility | busy, disabled action, source reorder/removal, earlier-choice branch removal, stale revision | revisioned steps/props |
| Layer/focus | closed/open, nested child modal, outside policy, Escape, owner removed, resize, focus restore, modal-first paste | runtime layer stack |
| Capability/motion | truecolor, 256, 16, none, NO_COLOR; normal/reduced/paused child feedback | every applicable visual variant |

Do not assign a generic pressed state to the wizard frame or fabricate a
scroll/animation state when only a child component supports it.

## Verification

Use plan dimensions `72x20`, `80x24`, `100x30`, `120x40`, `160x50` and
capabilities `truecolor`, `256`, `16`, `none`, `nocolor`. Bind expected output
before candidate comparison as `ExistingOracle`, `ExtractedOracle` or explicit
`Extension`.

| Case | Required observation |
|---|---|
| W27-01 | Invalid Next stays on the step; valid Next, Back and final Finish emit the correct typed actions. |
| W27-02 | Busy step and cancellation eligibility match the caller policy; disabled controls cannot activate. |
| W27-03 | Changing an earlier choice removes a later step without stale focus, value loss outside policy or action retargeting. |
| W27-04 | Nonlinear enabled steps and terminal-size reduction preserve step keys, drafts and exact geometry. |

Trace exact cells, continuations, colors/modifiers, cursor, focus/capture/layer
owners, current/visited step keys, child draft/commit state, busy/eligibility
flags and typed action count/target. Missing oracle setup is blocked, never a
pass.

Required negative tests include:

- draw and measure cannot advance, validate, persist or mutate child state;
- invalid/disabled/busy Next or Finish emits no action; Cancel follows policy;
- a removed/reordered step cannot retarget a pending action or focus to a new
  occupant; stable keys and visited-path reconciliation are preserved;
- Back retains drafts unless explicit caller invalidation says otherwise;
- child Escape/paste/modal-first routing cannot leak into wizard navigation;
- release outside, removed target and disabled child do not activate;
- zero/tiny geometry, Unicode labels and narrow resize cannot panic or split
  continuation cells;
- supported part patches affect reserved cells and cannot paint over child
  controls; dead-call/custom-paint mutation fails;
- one cell, cursor, step key, focus/capture owner or action trace difference
  fails exact comparison.

## Rejected use

Forbidden: preview implements its own step screens with padded fields and a
step counter instead of using `Wizard` with real child components.

```rust
// Forbidden: preview owns step screens and the step counter.
ui.paint_str(title_row, 2, "step 2 of 5: Destination");
ui.paint_str(field_row, 2, "  Path: /tmp/demo          ");
if next_pressed { step += 1; }
```

Rule: [ARC-012](../architecture/component-composition.md). Preview code must
not recreate component rendering or generic interaction. The following
patterns are prohibited in preview drawing paths:

```text
historical full-screen painters
snapshot files or exported frame data used as application output
hardcoded answer tables for 72x20, 80x24, 100x30, 120x40, or 160x50
fixture/scenario IDs that select a different painter
paused mode that bypasses component updates or layers
raw buffer access through termrock::author or an alias
direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls
padded strings that simulate columns, selection, fields, menus, or buttons
manual borders, focus gutters, scrollbars, or widget cursors
normal component drawing followed by a historical overlay
app-specific screens moved into a termrock crate under a generic name
fake state flags that display success without the normal component action
```

Use `Wizard::new` with a body slot of real components and handle the typed
`WizardAction`.

## Known gaps

- This component is a library-only contract. Application migration onto it is
  a future parity task and must preserve all four frozen consumers.
- [FIX-001](../implementation/code-remediation-backlog.md) (pending): replace
  reachable historical Prelude painters with component composition using
  Picker, fields, choices, Wizard/Dialog, and actions.
- [FIX-007](../implementation/code-remediation-backlog.md) (pending): resolve
  public signature drift against this contract; compile external examples and
  check the accepted state/action model.
- Capture cases W27-01–W27-04 await expected-artifact binding before candidate
  comparison; candidate code cannot bless its own expected output.
