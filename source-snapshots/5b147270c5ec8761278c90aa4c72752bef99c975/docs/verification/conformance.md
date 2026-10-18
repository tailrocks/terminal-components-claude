# Conformance

Conformance is the closure gate for the future in-place Termrock refactor. It
proves that the proposed reusable library preserves the frozen applications'
observable behavior while satisfying the new caller-owned API and shared
runtime contracts. It is a test and review contract, not an implementation.

The [test-only registry foundation](../foundations/conformance.md) owns the
case-index shape and production dependency boundary. This document owns
coverage closure and acceptance.

## Required inventory

The source-pack registry contains 45 component/API surfaces, 12 foundations,
54 legacy families, 45 capture plans, and 222 component-specific required
case descriptions. The final documentation must reconcile each registry entry
to one canonical component, foundation, verification, or explicit disposition:

```text
W01 Brand             W02 Button          W03 Checkbox        W04 Toggle
W05 RadioGroup        W06 ChipBar         W07 Field           W08 TextInput
W09 TextArea          W10 Select          W11 Form            W12 List
W13 FilterList        W14 NavList         W15 Tree            W16 Steps
W17 Tabs              W18 Picker          W19 CommandPalette  W20 PickerChain
W21 Completion        W22 Dialog          W23 Menu            W24 ContextMenu
W25 MenuBar           W26 HelpOverlay     W27 Wizard          W28 Grid
W29 CodeEditor        W30 DiffView        W31 TextViewport    W32 Panel
W33 SplitPane         W34 Props           W35 PropsList       W36 Empty
W37 ProgressBar       W38 Spinner         W39 Meter           W40 StatusBar
W41 HintBar           W42 KeyHint         W43 TooSmall        W44 TerminalView
W45 ScrollRegion
```

The canonical component documents under [`../components/`](../components/),
foundation documents under [`../foundations/`](../foundations/), and machine
readable registries under [`../reference/`](../reference/) are the places to
resolve the inventory. This document owns completeness and gates; it does not
restate each component's behavior.

All 54 legacy families remain traceable through
[`../reference/family-disposition.json`](../reference/family-disposition.json),
including families consolidated into shared Grid, Menu, Picker, text-editing,
scroll, panel/viewport, status, and hint mechanisms. Consolidation removes
parallel mechanisms while preserving distinct visual and interaction recipes.

## Public-surface conformance

External consumer tests compile against the future Termrock public API and
exercise `update`, `draw`, and `measure` with caller-owned durable state,
borrowed props, immutable draw state, typed actions, controlled values, stable
semantic IDs, stable item/column keys, and explicit reconciliation. They do
not include candidate internals, use `pub(crate)` shortcuts, copy baseline
painters, or create size-specific snapshot replicas.

The production library is independent of the test registry, PTY process
spawner, fonts, PNG/HTML libraries, discovery manifests, and approval store.
Compile-fail and protocol tests cover read-only editing, illegal secret
operations, private runtime access, and draw-state mutation. Public examples
must compile as external consumers after implementation.

`TerminalView` conformance is a prepared-cell presentation boundary. It checks
styles, continuation cells, cursor/selection, and typed copy/link/interaction
requests. It does not add an emulator, PTY runtime, shell, daemon, session
manager, or escape parser to Termrock.

## Generic composition fixtures

The source-pack recipes are inert library fixtures with typed synthetic data
and deterministic UI scripts. They are not application implementations and do
not contain routes, service effects, configuration files, account discovery,
launch decisions, authentication, or daemon behavior. Candidate fixture code
uses only the Termrock public API. The pinned applications may be used as
oracle sources, but their product worlds, route enums, account models, launch
simulators, and session behavior are not copied into a fixture.

The eight required composition cases are:

| Case | Composition | Required proof |
| --- | --- | --- |
| R01 List/detail shell | Brand, MenuBar, Panel, Tree/List/NavList, SplitPane, PropsList, Button, HintBar, TooSmall | Identity, current versus cursor, detail focus, drawer/narrow behavior, and source reorder |
| R02 Tabbed configuration form | Tabs, Form, Field, TextInput/TextArea, Checkbox/Toggle/Select, Grid, ChipBar | Draft lifecycle, errors, controlled values, row/cell presentation, and shared public components |
| R03 Nested selection dialog | Dialog, Picker, PickerChain, Completion, Menu, TextInput, HelpOverlay | Modal-first paste, one-level Escape, exact focus restoration, retry/readiness, and stable target |
| R04 Status/measurement panel | Props, List, Meter, Spinner, StatusBar, Empty | Unknown/stale/error/refreshing states, thresholds, and narrow priority dropping |
| R05 Progress and output | Steps, ProgressBar, TextViewport, Panel, Button, HintBar | Deterministic phases, cancel intent only, tail/reading retention, and logs versus prose |
| R06 Tabbed cell surfaces | Tabs, ContextMenu, SplitPane, TerminalView, StatusBar | Prepared terminal-cell composition, cursor/input ownership, per-tab focus, and resize |
| R07 Inspect/review surface | Tree/List, DiffView, read-only CodeEditor, PropsList, Dialog | Source-preserving selection/copy, review fallback, and caller-owned actions |
| R08 Lifecycle/size harness | Tiny Scene, TooSmall, optional TerminalSession and author paint | Cleanup, resize recovery, and custom-art clipping without a product animation engine |

Each recipe is tested as a composed scene with the exact frame and semantic
gates in [`visual-parity.md`](visual-parity.md) and
[`interaction-parity.md`](interaction-parity.md). Synthetic labels may make a
baseline frame recognizable, but must not become library API or production
logic. A fixture does not introduce a second paint path for a standard
component: changing Button in the library must change every recipe that uses
Button and fail its exact gate.

## Negative mutation gates

Each applicable mutation must fail the specific gate named in its evidence:

| Mutation | Required failure |
| --- | --- |
| One glyph, combining/wide symbol, continuation cell, dimension, foreground/background, or preserved modifier | Exact frame comparison |
| Cursor position, visibility, or supported cursor property | Cursor observation/frame gate |
| Focus owner, capture owner, active layer, selected stable key, navigation key, draft/commit value | Semantic checkpoint comparison |
| Hitbox edge, sealed pointer coordinate, release target, action order, duplicate activation, stale key after reorder | Interaction trace gate |
| Timer boundary or animation phase | Deterministic time/motion gate |
| Disabled click-through, read-only mutation, invalid controlled change, secret marker | Component/state invariant and leak gate |
| Missing, deleted, duplicate, stale, corrupt, or unexpected expected artifact | Fail-closed artifact gate |
| Changed oracle SHA, source/binary/adapter digest, profile/font/tool pin, manifest, or comparator | Provenance/integrity gate |
| Candidate-generated expected file, approval write, symlink escape, comparator replacement, failure filtering | Trust-boundary gate |
| Standard component replaced by dead call plus custom paint | Ownership/mutation gate |

Draw repeatability must fail when draw mutates state. A green Rust build or a
matching screenshot without these negative probes is insufficient evidence.

## Gate sequence

The future task catalog follows the library phases and attaches evidence at
each gate:

1. **P0 reference qualification:** verify the exact baseline commit/tag,
   original renderer/tool pin, expected artifact integrity, source/tool
   provenance, and comparator mutations.
2. **P1 runtime/API:** establish external-consumer checks, identity,
   events/responses, focus/capture, layers, layout, theme, text/model and the
   pure/PTY harness boundary.
3. **P2 collections/basic components:** prove stable keys, reconciliation,
   controlled choices, geometry and applicable visual states.
4. **P3 editing/forms/secrets:** prove Unicode editing, field-specific policy,
   revisions, validation, read-only behavior and secret non-leakage.
5. **P4 overlays/menus/pickers/completion:** prove layer stack, capture,
   Escape precedence, anchoring, focus restoration and action targets.
6. **P5 rich output/layout:** prove Grid, viewport, editor, diff, progress,
   spinner, meter, status/help, scroll/fade and motion phases.
7. **P6 terminal edge/composition:** prove prepared-cell TerminalView, PTY
   input/resize/cleanup edge cases, and generic composed scenes.
8. **P7 closure:** audit the public API and component/foundation inventory,
   run complete parity, all negative mutations, performance checks, and an
   independent review.

Unavailable oracle states, missing expected artifacts, and incomplete
provenance block the gate. They never become an implicit skip or reduce the
required denominator.

## Current baseline commands

The current repository suite is frozen evidence. Rust validation uses
`cargo nextest`; no `cargo test` command belongs in this contract. Read-only
checks for the future implementation include:

```sh
cargo nextest run --locked
cargo nextest run --locked --run-ignored only \
  -E 'binary(visual_baseline)'
cargo nextest run --locked --run-ignored only \
  --ignore-default-filter -E 'test(rebuild_review_html)'
```

The PTY capture suite is ignored by default and only passes when every approved
scenario is `matched`. Missing approval, drift, corrupt approval, missing
artifact, or capture error fails. A trusted reviewer may generate and review
reference captures outside candidate authority; candidate execution must never
bless its own output.

## Application closure

The final conformance run covers `showcase`, `tablepro`, `jackin-preview`, and
`holla` as unchanged reference consumers. It preserves their fixtures,
keyboard/mouse scenarios, output, and snapshots. Application migration work in
the future implementation is limited to wiring these consumers onto reusable
Termrock components while retaining exact parity. Product services, provider
integrations, account authorization, Docker, database backends, Git operations,
new routes, and visual redesigns are outside this conformance plan.

## Completion evidence

P7 is complete only when all of the following are attached to the reviewed
result:

- bidirectional registry reconciliation for 45 components, 12 foundations,
  54 families, 45 plans, and all 222 required case IDs;
- exact frame and semantic comparison results for every bound case;
- PTY results for each applicable transport/routing/cleanup family;
- negative mutation results showing every required drift fails;
- source, adapter, binary, renderer, profile, font, fixture, manifest, and
  artifact digests;
- no candidate approval writes and no expected-set change during execution;
- external-consumer API, compile-fail, leak, repeatability, and performance
  evidence;
- independent adversarial review with no unresolved blocking finding.
