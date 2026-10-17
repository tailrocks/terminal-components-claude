# Dialog

Status: proposed target.
Owner: termrock-overlays.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W22 · Group: Overlays · Phase: P4 · Legacy family: C31 (`dialog`).
Capture plan: [`../reference/capture-plans/dialog.json`](../reference/capture-plans/dialog.json).
Current source: [`src/widgets/dialog.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/dialog.rs).

Canonical Termrock contract for W22. This page describes the target API on
`termrock-implementation`; it does not rename the current implementation or change
any baseline application.

## Purpose and exclusions

`Dialog` presents a modal frame around caller supplied content and typed action
metadata. The caller owns facts, wording, acknowledgement models, child
widgets and action eligibility. Termrock owns modal layer placement, the
backdrop, focus trapping, pointer barriers, and the dialog chrome.

The body is an open slot of real components. Information, prompt, confirmation,
error and destructive presentations are recipes over the same engine. A typed
acknowledgement is a child `TextInput` plus caller eligibility; it is not a
second dialog engine or a closed body enum.

Exclusions:

- executing application effects, persistence, network work or shell commands;
- a universal boxed `Widget` trait or a `show()` call that mixes update and
  painting;
- owning child text drafts, domain facts, or the meaning of an action;
- exposing runtime geometry, hit registries, focus handles or mutable domain rows;
- making every overlay a dialog or imposing one application-specific modal copy;
- redesigning Showcase, TablePro, Jackin Preview or Holla.

## Public API

These are Rustdoc-style target declarations. They are proposed signatures until
the P1 external-consumer probes freeze the spelling.

```rust
Dialog::new(
    id: Id,
    title: &'a str,
    actions: &'a [ActionMeta<'a>],
) -> Dialog<'a>

Dialog::tone(self, tone: DialogTone) -> Self
Dialog::dismiss(self, policy: DismissPolicy) -> Self
Dialog::default_action(self, action: Option<ActionKey>) -> Self
Dialog::body_size(self, size: Size) -> Self
Dialog::patch(self, patch: StylePatch) -> Self

Dialog::update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut DialogState,
) -> Response<DialogAction>

Dialog::draw<R>(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &DialogState,
    body: impl FnOnce(&mut Ui<'_>, Rect) -> R,
) -> R

Dialog::measure(
    &self,
    cx: &MeasureCx<'_>,
    constraints: Constraints,
) -> Size
```

`actions`, title, tone, dismissal policy, body size, and acknowledgement
eligibility are short-lived borrowed props. `DialogState` owns only dialog-local
durable view state: body scroll and generic dialog navigation. Child drafts and
acknowledgement values stay in caller-owned child state.

`DialogAction` is typed and carries semantic identity:

```rust
enum DialogAction {
    Choose { action: ActionKey, origin: ActivationOrigin },
    Dismiss { reason: DismissReason },
}
```

No public action is an array index. `ActionKey` survives action-list refresh;
an action removed or disabled while pressed cannot be silently replaced by the
new occupant of its row.

The source module currently exposes legacy `DialogBody`, `DialogResult`,
`WidgetId`, `Outcome` and `RenderCtx` names. They are implementation evidence,
not Termrock compatibility requirements. The future component is a reusable
Termrock dialog in this same repository and branch.

## Ordinary use

Consumer recipe: [EX-08 — A dialog that keeps normal input behavior](../api/consumer-recipes.md) (`proposed_target`).

Relevant unchanged consumers and evidence include:

- Showcase `src/bin/showcase/pages/dialogs.rs`, with confirm, prompt,
  three-action unsaved-change and destructive flows;
- Jackin Preview `src/bin/jackin_preview/screens/prelude.rs` and
  `screens/modals.rs`, including the five-step prelude, forms and nested
  modal routing;
- TablePro safety dialogs and Holla review/trust dialogs in the frozen source;
- visual cases under `snapshots/showcase` and `snapshots/jackin`, including
  `showcase/flows/dialogs/*` and `jackin/*/quit_confirm` where present;
- baseline drivers [`tests/visual_baseline/showcase.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/tests/visual_baseline/showcase.rs)
  and [`tests/visual_baseline/jackin.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/tests/visual_baseline/jackin.rs).

## Ownership

- `update` consumes one normalized input or runtime cause. The topmost layer
  receives it first. An active child editor receives text, paste and editing
  keys before dialog-level `y`, `n`, Enter or Escape policy.
- `update` may reconcile action metadata and child identity at an explicit
  revision boundary, retain surviving keys, change `DialogState`, and return at
  most one caller-facing `DialogAction`.
- `draw` is semantically read-only. It may register the surface and child hit
  regions and invoke the body slot, but it never commits drafts, advances focus,
  arms an action, dismisses a layer, or reconciles rows.
- `measure` is pure. It accounts for title, wrapped body, acknowledgement and
  pinned actions under constraints. It does not register focus, capture or
  hits and is safe for zero and tiny allocations.
- Runtime owns focus, hover, pointer capture, layer order, outside-click
  routing and feedback time. Dialog owns only its local navigation/scroll view.

Shared dependencies:

- [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md),
  [`input-actions`](../foundations/input-actions.md),
  [`identity`](../foundations/identity.md), [`layout`](../foundations/layout.md),
  [`theme`](../foundations/theme.md), [`text`](../foundations/text.md),
  [`author`](../foundations/author.md),
  [`conformance`](../foundations/conformance.md);
- [`button`](./button.md), [`text-input`](./text-input.md),
  [`scroll-region`](./scroll-region.md).

## Customization

Advertised parts are `container`, `border`, `title`, `body`, `actions` and
`status`. A `StylePatch` follows the shared Termrock precedence; a supported
part slot can replace only its reserved rectangle. It cannot change geometry,
focus/capture ownership, backdrop coverage or action ownership.

## Behavior

### Focus, keyboard and editing

- Opening a dialog creates a runtime modal layer, saves the prior focus, and
  focuses the declared initial child. Only the dialog and its children are in
  the focus scope. Closing restores the exact saved owner when it still exists;
  otherwise runtime applies its fail-closed focus rule.
- Tab and reverse Tab traverse live children. Left/Right navigate eligible
  action buttons when the action row owns focus. Disabled actions are skipped
  for keyboard traversal and remain barriers to activation.
- Escape follows `DismissPolicy`, normally invoking the declared Cancel action.
  `y`/`n` are effective bindings for plain text confirmation bodies only; they
  cannot confirm while a text field or acknowledgement is editing.
- Enter in a prompt commits/validates its child and then emits the primary
  action only when valid. Enter in a typed acknowledgement advances focus; it
  does not bypass the token requirement. Read-only facts and code are never
  treated as editable input.
- Paste is routed to the topmost active child. A modal over a picker or form
  must not leak paste or text keys to the layer below.

### Pointer, hover and activation

- A pointer press captures the actual child target; only a matching release
  inside completes activation. Release outside, target removal, or target
  disabling cancels the pending action. A held press is observable separately
  from post-release feedback.
- Clicking the dialog surface or outside it is consumed by the modal barrier.
  Outside cancellation follows the policy; a typed acknowledgement dialog and
  any non-cancelable dialog remain open. Outside cancellation never clicks the
  underlying application.
- Hover may lift the actual button or child row. Keyboard input suppresses
  stale hover until pointer motion restores it. The backdrop and whole dialog
  have no fake pressed state.
- Disabled action controls render disabled and consume attempts without an
  action. Read-only body content remains visible and scrollable; it is not
  equivalent to disabled controls.

### Layers, scroll, resize and reconciliation

- A dialog may be nested above a picker or form. Escape closes the topmost
  eligible layer one level at a time; focus restoration follows the saved
  layer path. The first paste/key event belongs to the topmost editor.
- Body scrolling uses the shared `ScrollRegion` when the body overflows,
  including wheel boundaries, scrollbar thumb capture, edge fades and resize
  while scrolled. Pinned actions remain visible.
- Placement and size are recomputed from `measure` on resize. Zero/tiny areas
  fail closed without panics or hits outside the allocation; narrow-to-wide
  recovery retains child drafts, focus semantics and stable action keys.
- Reconcile title/body/action props only during `update` at an explicit source
  revision. Preserve surviving `ActionKey`s and child IDs; a removed target
  invalidates pending activation rather than retargeting by position.

## Visual matrix

The frozen implementation is the visual and observable interaction oracle. The
current source renders a rounded elevated frame, dims the application area,
keeps the footer live, places a title and body above a right-aligned action row,
and registers a surface barrier before child controls. Its prompt, destructive,
facts and typed-acknowledgement paths are all part of the reference.

The default `Theme::termrock()` recipe must reproduce the frozen output:

- elevated rounded frame, title hierarchy, body wrapping and right-aligned
  action row with the same glyphs, spacing, clipping and surfaces;
- information, error, prompt, confirmation and destructive tones as observed;
  destructive action uses the danger recipe while Cancel remains a distinct
  safe action;
- backdrop dims the foreground surface ladder while preserving relationships;
  the live footer remains outside the dimmed area where the oracle does so;
- a typed acknowledgement reserves the field and code preview above actions;
  facts clip first on short screens, while the acknowledgement and actions stay
  inside the frame;
- `input_after_body(true)` reserves the acknowledgement field below the body
  slot, above the actions; facts clip first;
- Unicode body and title content are measured by display cells and never split
  a grapheme or wide-cell continuation;
- truecolor, 256-color, 16-color, explicit `none` and `NO_COLOR` lanes resolve
  through semantic roles without hardcoded component colors;
- child action activation uses the shared 140 ms feedback clock where the
  baseline exposes feedback. Motion paused/reduced policy must remain explicit.

| Axis | Required cases | Applicability |
|---|---|---|
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, one cell short, long Unicode, narrow then wide | all dialog recipes |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer-motion restoration | real child controls; no decorative focus stop |
| Activation | pointer down, held press, release inside/outside, removed/disabled target, keyboard activation, feedback active/expired | eligible child action/button only |
| Editing | navigation, editing, valid/invalid, commit, cancel, blur/tab, external revision conflict | prompt and acknowledgement children; not plain facts |
| Scroll | no overflow, start/middle/end, wheel boundary, thumb drag, resize while scrolled, edge fade, protected rows | body/content region only |
| Layer | closed/open, nested, outside, Escape ladder, owner removed, resize/reanchor, focus restoration, modal-first paste | runtime-owned modal layer |
| Data/tone | information, error, prompt, confirm, destructive; empty/loading/partial only where body supplies them | caller supplied body/status |
| Capability/motion | truecolor, 256, 16, none, NO_COLOR; normal/reduced/paused feedback | every visual variant that uses the lane |

Do not fabricate hover, pressed or editing states for a dialog whose body has no
such child. Attribute composite activation to the real child control.

## Verification

Use the five capture sizes `72x20`, `80x24`, `100x30`, `120x40`, `160x50` and
capabilities `truecolor`, `256`, `16`, explicit `none` and `nocolor`, as listed
by the capture plan and shared verification contract. Expected artifacts are
bound to `ExistingOracle` or `ExtractedOracle` before candidate comparison;
new robustness/API cases are `Extension`. Candidate code cannot bless its own
expected output.

| Case | Required observation |
|---|---|
| W22-01 | Information, error, prompt, confirmation and destructive bodies use actual child controls and preserve each baseline geometry. |
| W22-02 | Typed acknowledgement transitions unarmed/armed; changing the target or token invalidates the pending confirmation. |
| W22-03 | Dialog above picker/form receives paste first, traps focus, and restores the exact owner after close. |
| W22-04 | Long body resizes while scrolled; facts clip before acknowledgement and pinned actions. |
| W22-05 | Outside click cancels only when policy permits and never clicks through to the page. |

Every trace records dimensions, symbols, wide-cell continuation,
foreground/background and supported modifiers, cursor coordinates/visibility,
focus owner, capture owner, layer path, selected/action stable keys, draft and
committed values, and typed action count/target. Missing oracle setup is
blocked, never a passing capture.

The current source has a known historical acknowledgement hit path; the target
must route clicks through the same input owner used for keyboard and paste, and
the negative test below must prevent regression. This documentation does not
edit that source during the documentation phase.

Required negative tests include:

- draw twice produces identical semantic state and cells;
- disabled actions cannot activate or click through to an underlying target;
- a release outside, removed target or changed action list produces no action;
- acknowledgement clicks, keyboard edits and paste share the same child owner;
  wrong token never enables the confirming action;
- `y` while editing cannot submit; stale action keys cannot invoke a new row;
- nested modal paste and Escape never reach a lower layer;
- zero/tiny allocations, long Unicode and narrow widths do not panic or split
  continuation cells;
- backdrop, action-row and body part patches change only their reserved cells;
  a dead dialog call followed by custom paint fails the ownership mutation gate;
- one changed cell, cursor, focus owner, capture owner or typed action target
  fails exact comparison.

## Rejected use

Forbidden: preview paints its own modal frame and padded action labels instead
of using `Dialog`.

```rust
// Forbidden: preview owns the modal frame and the confirmation flag.
ui.fill(dim_area, 0, 0, dim_style);
ui.paint_str(action_row, 2, "  [ OK ]    [ Cancel ]  ");
if clicked_ok { confirmed = true; }
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

Use `Dialog::new` with a body slot and handle the typed `DialogAction`.

## Known gaps

- [FIX-002](../implementation/code-remediation-backlog.md) (pending): replace
  historical Manager, menu, inspect, help, and quit screens; the dialog slice
  must use the real `Dialog` with actual child controls.
- [FIX-007](../implementation/code-remediation-backlog.md) (pending): resolve
  public signature drift against this contract; compile external examples and
  check the accepted state/action model.
- Capture cases W22-01–W22-05 await expected-artifact binding before candidate
  comparison; candidate code cannot bless its own expected output.
