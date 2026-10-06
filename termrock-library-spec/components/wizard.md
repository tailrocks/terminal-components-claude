# W27 · Wizard

**Group:** Composition helpers · **Phase:** P4 · **Classification:** baseline-composition  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Generic navigation/retention helper; no clone operation, process execution, automatic persistence or domain-specific fields.

Legacy family mapping: `C34`  
Proposed implementation home: `crates/termrock/src/components/wizard.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/bin/jackin_preview/screens/prelude.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/prelude.rs)
- [src/bin/jackin_preview/screens/modals.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/modals.rs)
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: wizard.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/wizard.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/jackin](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
Wizard::new(id: Id, steps: &'a [WizardStep<'a>], revision: Revision) -> Wizard<'a>

update(&self, cx: &mut Cx<'_>, state: &mut WizardState) -> Response<WizardAction>
draw<R>(&self, ui: &mut Ui<'_>, area: Rect, state: &WizardState, body: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> R
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
can_advance(bool)
can_finish(bool)
busy(bool)
patch(StylePatch)
```

**Durable state:** WizardState: current stable step and visited path. Values and per-step widget states are caller-owned.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** WizardAction::{Next { from: ItemKey }, Back { from: ItemKey }, Finish, Cancel}

## Visual parts and composition

Advertised styled parts: `container`, `step-heading`, `body`, `actions`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [form](./form.md), [button](./button.md)

## Behavioral contract

1. Next/Finish respects current validity and eligibility. Back retains previous field drafts unless the caller explicitly invalidates them.
2. Removing/branching a step reconciles by key; disabled steps are not blindly treated as completed.
3. Step body is a slot of actual components, using the source composition rather than introducing a redesigned wizard screen.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W27-01 | invalid Next, valid Next, Back and final Finish |
| W27-02 | busy step and cancellation eligibility |
| W27-03 | change earlier choice removes a later step without stale focus |
| W27-04 | nonlinear enabled steps and terminal-size reduction |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/wizard.json](../capture-plans/wizard.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
