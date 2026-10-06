# Review and evidence status

## Completed here

The deliverable narrows scope to the library, expands the earlier 54-family inventory into 45 per-component API references and 12 shared-foundation references, corrects identified reference-path/behavior ambiguities, supplies per-component capture plans and reconciles old families with the new destinations.

A structural validator checks unique IDs, complete old-family mapping, existing local links, source pins, component dependency acyclicity/phase ordering, per-component API/capture-plan presence, JSON schema conformity, absence of bundled font files and archive integrity. See reference/validation.json for the executed result. Those checks validate this document pack, not Rust implementation behavior.

## Not executed here

No Rust compiler/toolchain was available in the local runtime used to assemble this pack. No proposed API was compiled, no Termrock implementation was created, no baseline/PTY executable was run, no new screenshots or animation captures were generated, and no performance numbers were measured. Existing source code and metadata were read through GitHub; full repositories were not cloned locally. Source links/blob metadata are references, not bundled repository mirrors.

No independent reviewer agent was launched in this environment. Do not interpret generated checklists or automated document checks as independent review. All implementation, capture, API compile and reviewer statuses remain pending.

## Independent review packets for implementation

**API reviewer:** compare each reference with the pinned architecture and refactoring task catalog. Verify caller-owned state, borrowed props, two phases, typed actions, stable keys, shared text/scroll/layers, constrained parts, secret exceptions and no domain-specific exports. Compile real external consumers; reject elaborate abstractions without a needed component capability.

**Visual/interaction reviewer:** compare exact old/new output and event traces, including Unicode, colors, hover suppression, completed-click semantics, field-specific Escape/paste behavior, narrow widths, fade protection and every animation phase. Do not approve from candidate-only screenshots.

**Coverage/adversarial reviewer:** verify every legacy family, every advertised variant/part and every applicable state/transition is accounted for. Remove an expected file, mutate a pixel, retarget an action, duplicate an activation and hide a case; the suite must fail. Reject size-specific replicas, dead controls, paint-over and secret leaks.

**Simplicity/runtime reviewer:** inspect actual dependency graphs, file complexity, lifecycle cleanup, geometry freshness and measured compile/frame costs. Verify headless operation, optional backend and test-only toolkit isolation. Additional production crates require measured benefit, not stylistic preference.

## Findings already reflected in this specification

| Finding | Resolution |
|---|---|
| Earlier plan included product integration | All current phases are library/conformance only |
|54 families could be mistaken for 54 widgets | Explicit 45 component/API surfaces,12 foundations and full mapping |
| Single-line and multiline Escape/paste differ | Separate field policies retained from direct source reads |
| Meter source was listed only as app-local | Primary origin is the baseline progress.rs widget module |
| Earlier op_flow.rs path is not the baseline source location | PickerChain reference uses screens/modals.rs |
| Dynamic source and draft conflict contracts were implicit | Explicit revisions and preserve/report conflict policy |
| Secret states cannot follow blanket Clone/Debug rules | Sealed secret specialization and leak/compile-fail tests |
| Draw signatures cannot prove purity alone | Callback restrictions plus state/mutation tests |
| New terminal adapter has no standalone old widget | Separate adapter-extension evidence lane |
| Renderer version drift could pollute approvals | Qualify original/new toolkit separately from product output |

## Acceptance statement

This is a detailed proposed specification, not a statement that all required behavior has already been implemented or independently approved. The milestone is accepted only after the implementation evidence described above exists, with no live Jackin implementation required in the current scope.
