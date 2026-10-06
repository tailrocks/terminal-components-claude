# Implementation plan — Termrock only

Every phase is a library/conformance task. No phase creates or migrates Jackin.

| Phase | Work | Acceptance |
|---|---|---|
| P0 | Freeze source/tool refs; import existing oracle artifacts; inventory exports, parts, caller-specific behavior and missing states; qualify comparator | Immutable source manifest; required cases cannot disappear; deliberate one-cell/action mutations fail |
| P1 | Establish facade and external consumer tests; identities, input/response, runtime focus/capture/time, layers, theme, shared text/model contracts | Boot/event/draw purity, stale-geometry, duplicate-key, overlay and color tests pass |
| P2 | Button/Brand, Panel/SplitPane/ScrollRegion, List/NavList/Tree/Tabs, Empty, KeyHint/HintBar/TooSmall | A generic list-detail shell plus nested/resize/pointer fixtures match source; no domain modules |
| P3 | Field/Input/TextArea, choices, Select, ChipBar, Form, Props/PropsList and secret validation | Draft/commit/cancel, Unicode, controlled value, secret negatives and form composition pass |
| P4 | Menu/ContextMenu/MenuBar, Dialog, FilterList/Picker/CommandPalette/PickerChain, Completion and Wizard | Nested overlay typing/paste/dismissal and stable-target behaviors pass |
| P5 | TextViewport, Grid(table+cells), CodeEditor/DiffView, Steps/Progress/Spinner/Meter, StatusBar and HelpOverlay | Output retention, source projection, diff fallback, read-only capability and full animation cycles pass |
| P6 | TerminalView borrowed-cell adapter; remaining generic composed fixtures; optional crossterm session tests | Cell/cursor/input ownership and cleanup pass without a PTY/emulator dependency in core |
| P7 | Complete public API/parts audit, per-case exact comparison, mutation gates, performance measurements and independent reviews | Zero unresolved required cases; no approved-oracle edits; no duplicate painters; all examples compile |

## Sequencing within phases

Source and capture inventories are independent from candidate implementation. Build the comparator first and negatively qualify it. Do not let candidate workers create expected files.

Within P1, settle identities/responses/clock and model signatures before parallel widget work. Theme, geometry and shared text contracts can be developed in parallel with narrow ownership, then integrated before accepting downstream components.

Within P2, ScrollRegion and Button precede List/Tree and composed rows. Basic KeyHint precedes HintBar. Within P3, Field/shared text/secret storage precede Input/TextArea, then choices and Form. Within P4, Dialog and FilterList precede Picker; Menu precedes its wrappers. Within P5, Spinner precedes Steps/Meter/StatusBar, TextViewport precedes CodeEditor/DiffView/HelpOverlay. The machine-readable dependency graph in reference/components.json is acyclic.

## Parallel work packets

Use separate owners for foundation/runtime, theme/layout, text/secrets, collections, fields/forms, overlays, output/grid/diff and conformance. Delegate research and review when the execution environment supports actual agents. Do not claim subprocesses or scripts are independent reviewers. Pin any mandated agent model/effort exactly and fail closed rather than silently substituting it.

Each worker receives its component references, shared contracts, read-only oracle and an explicit writable path set. Shared API changes require the relevant contract owner. Prefer one working branch with small coherent commits; coordinate file ownership rather than uncontrolled concurrent edits to the same file. Preserve unrelated changes; do not create a branch per widget by default.

A work packet ends with implementation, separate-file tests, source references, external-consumer example, exact actual/diff artifacts, and a factual results summary. No progress diary, generated coordination system, stub renderer or temporary compatibility layer is a shipped deliverable.

## Mandatory proof for each component

Implement update/draw/measure and its applicable controlled state semantics. Run headless action/state tests and draw-purity checks. Capture required states against the trusted oracle. Exercise mouse and keyboard in the generic lab. Verify all documented slots actually affect the correct cells. Run at least one negative mutation of its paint and one of its action behavior. Review API consistency, ownership and scoped complexity.

A missing oracle state is capture work, not a reason to approve the candidate. A nonvisual helper gets a justified non-frame disposition and behavioral tests. A newly specified state gets an Extension label and separate review.

## Definition of done

All 45 component/API references and 12 foundation contracts are dispositioned with passing evidence or an explicitly reviewed change of scope. All 54 old ledger rows retain traceability. Applicable baseline cases are exact; new adapter/theme/robustness cases have separately accepted contracts. No snapshots are updated automatically. The core crate builds without conformance/backend dependencies. Public examples compile against only public exports. Library-only composed fixtures cover every required generic building block.

Actual Jackin integration begins in a later, separately approved plan. It is deliberately absent from this dependency graph.
