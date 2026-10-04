# Proposed functionality-based crate map

This is a target architecture proposal, not the current repository layout. It supersedes the earlier one-library-crate limit. All packages are immediate children of `crates/`; all production, app, test and tooling Rust source lives within those packages. The root is a virtual Cargo workspace; documentation/configuration/baseline artifacts may remain at root.

The 44 packages below are not a quota to satisfy with empty wrappers. Freeze a source-reconciled ownership map before moving code. Refine a boundary only with documented dependency/consumer evidence, matching checklist updates and independent review; no refinement may collapse the requested multiple library crates or multiple functionality crates per application. Extra crate splits need real ownership or measured build benefits.

Facades and binary entry points are intentionally thin; every other crate must own meaningful code, tests and responsibilities. Consumer crates use the public `termrock` facade; lower library crates use narrow sibling contracts and never depend back on the facade. All app-only packages and conformance/tools are `publish = false` unless an explicit later release policy says otherwise.

The listed dependency edges are an allowed internal direction, not an instruction to retain unused dependencies. External dependencies must also be justified. A publish=false test crate may have normal dependencies on Tuiscotti or app crates; production crates must never depend on that test crate.


## library

### `termrock-core`

Path: `crates/termrock-core`.

Stable control/item/column identities, revisions, normalized input/intents, typed responses, invalidation and explicit time vocabulary.

**Boundary:** No widgets, application domain, backend, capture toolkit, or circular dependency on the facade.

**Allowed internal dependencies:** none.

**Source hints, to reconcile:** `src/core/id.rs`, `src/core/event.rs`, `src/termrock/identity.rs`, `src/termrock/response.rs`.

### `termrock-layout`

Path: `crates/termrock-layout`.

Pure constraints, tracks, measurement, clipping and split geometry.

**Boundary:** No events dispatched, painting, retained application state, or IO.

**Allowed internal dependencies:** `termrock-core`.

**Source hints, to reconcile:** `src/ui/layout.rs`, `src/termrock/layout.rs`.

### `termrock-theme`

Path: `crates/termrock-theme`.

Semantic colors, surfaces, glyphs, motion/design tokens, recipes and override resolution.

**Boundary:** Preserve baseline recipes. No runtime or widget dependency.

**Allowed internal dependencies:** `termrock-core`.

**Source hints, to reconcile:** `src/theme.rs`, `src/termrock/theme.rs`.

### `termrock-text`

Path: `crates/termrock-text`.

Unicode measurement, grapheme editing, revisioned text sources, selection/copy mapping, protected text storage and bounded edit history.

**Boundary:** Text rules do not import widgets, session IO or application credentials. Protected state has restricted exposure/Debug/clone semantics.

**Allowed internal dependencies:** `termrock-core`, `termrock-theme`.

**Source hints, to reconcile:** `src/core/text.rs`, `src/ui/text.rs`, `src/termrock/text.rs`, `src/termrock/secret.rs`.

### `termrock-render`

Path: `crates/termrock-render`.

Constrained cell painting, clipping, surface/style application and paint/layout fact primitives.

**Boundary:** No rasterizer, fonts, PNG/HTML export, event router or product painter. Runtime wraps these primitives in the public Ui.

**Allowed internal dependencies:** `termrock-core`, `termrock-layout`, `termrock-theme`, `termrock-text`.

**Source hints, to reconcile:** `src/ui/ctx.rs`, `src/termrock/author.rs`.

### `termrock-collections`

Path: `crates/termrock-collections`.

Keyed borrowed models, reconciliation, selection and shared ScrollState/scrollbar geometry.

**Boundary:** No application rows or independent focus/gesture engine.

**Allowed internal dependencies:** `termrock-core`, `termrock-layout`, `termrock-text`.

**Source hints, to reconcile:** `src/core/scroll.rs`, `src/termrock/collections.rs`, `src/termrock/scroll.rs`.

### `termrock-runtime`

Path: `crates/termrock-runtime`.

Frame publication, Cx/Ui, focus, hit regions, pointer capture, layers, cursor arbitration and interaction timing.

**Boundary:** Exactly one interaction owner. Platform terminal IO lives in termrock-session; no dependency on component crates.

**Allowed internal dependencies:** `termrock-core`, `termrock-layout`, `termrock-theme`, `termrock-render`, `termrock-collections`.

**Source hints, to reconcile:** `src/runtime.rs`, `src/core/focus.rs`, `src/core/hit.rs`, `src/termrock/runtime.rs`, `src/termrock/layers.rs`.

### `termrock-controls`

Path: `crates/termrock-controls`.

Basic actionable/decorative controls: Button, Brand, choices, Panel, SplitPane, Props, Empty and TooSmall.

**Boundary:** No per-control event loop, cloned legacy renderer, or application policy.

**Allowed internal dependencies:** `termrock-core`, `termrock-runtime`, `termrock-layout`, `termrock-theme`, `termrock-text`, `termrock-collections`.

**Source hints, to reconcile:** `src/widgets/button.rs`, `src/widgets/choice.rs`, `src/widgets/brand.rs`, `src/widgets/panel.rs`, `src/widgets/splitter.rs`, `src/widgets/props.rs`, `src/widgets/empty.rs`.

### `termrock-fields`

Path: `crates/termrock-fields`.

Field chrome, TextInput and TextArea behavior, drafts, editing policy, generic validation and protected input adapters.

**Boundary:** Single-line/multiline policies remain distinct. No popup dependency: Select is an overlay composition.

**Allowed internal dependencies:** `termrock-core`, `termrock-runtime`, `termrock-controls`, `termrock-text`, `termrock-collections`.

**Source hints, to reconcile:** `src/widgets/field_common.rs`, `src/widgets/input.rs`, `src/widgets/textarea.rs`.

### `termrock-feedback`

Path: `crates/termrock-feedback`.

Spinner, ProgressBar, Meter, StatusBar, HintBar and KeyHint using shared motion and style contracts.

**Boundary:** Unknown quota is not zero, progress is not quota, derived hints are not a second painter.

**Allowed internal dependencies:** `termrock-core`, `termrock-runtime`, `termrock-theme`, `termrock-text`, `termrock-controls`.

**Source hints, to reconcile:** `src/widgets/progress.rs`, `src/widgets/statusbar.rs`, `src/widgets/segments.rs`, `src/widgets/hintbar.rs`, `src/widgets/keyhint.rs`.

### `termrock-navigation`

Path: `crates/termrock-navigation`.

List, NavList, Tree, Tabs, ChipBar, Steps, PropsList and filtered-list composition over shared collections.

**Boundary:** Stable item identities and one collection/scroll mechanism; no product routing decisions.

**Allowed internal dependencies:** `termrock-core`, `termrock-runtime`, `termrock-collections`, `termrock-controls`, `termrock-fields`, `termrock-feedback`, `termrock-text`.

**Source hints, to reconcile:** `src/widgets/list.rs`, `src/widgets/tree.rs`, `src/widgets/tabs.rs`, `src/widgets/chips.rs`, `src/widgets/steps.rs`.

### `termrock-viewport`

Path: `crates/termrock-viewport`.

Revisioned TextViewport, read/selection/follow state, shared ScrollRegion and edge fades.

**Boundary:** No editor, popup or terminal emulator dependency; prose and log follow policies are explicit.

**Allowed internal dependencies:** `termrock-core`, `termrock-runtime`, `termrock-collections`, `termrock-text`, `termrock-controls`.

**Source hints, to reconcile:** `src/widgets/viewport.rs`, `src/widgets/scrollbar.rs`, `src/ui/fade.rs`.

### `termrock-overlays`

Path: `crates/termrock-overlays`.

Dialog, Menu/ContextMenu/MenuBar, Select, Picker/CommandPalette/PickerChain, Completion and HelpOverlay.

**Boundary:** Shared runtime layer ownership; open body slots avoid a reverse dependency on Grid/CodeEditor/Form.

**Allowed internal dependencies:** `termrock-core`, `termrock-runtime`, `termrock-controls`, `termrock-fields`, `termrock-navigation`, `termrock-viewport`, `termrock-feedback`.

**Source hints, to reconcile:** `src/widgets/dialog.rs`, `src/widgets/menu.rs`, `src/widgets/picker.rs`, `src/widgets/select.rs`, `src/widgets/completion.rs`.

### `termrock-grid`

Path: `crates/termrock-grid`.

One tabular engine for row-table and editable cell-grid presentation with separate read and edit contracts.

**Boundary:** No SQL, database schema, pending-write persistence or TablePro domain ownership. Caller adapters map domain values.

**Allowed internal dependencies:** `termrock-core`, `termrock-runtime`, `termrock-collections`, `termrock-controls`, `termrock-fields`, `termrock-navigation`, `termrock-overlays`, `termrock-text`.

**Source hints, to reconcile:** `src/widgets/table.rs`, `src/widgets/grid.rs`.

### `termrock-editors`

Path: `crates/termrock-editors`.

CodeEditor and DiffView presentation, selections, completion/highlighting contracts and responsive diff modes.

**Boundary:** Borrow highlighting/diff inputs; no SQL engine, source-control process or duplicate text core.

**Allowed internal dependencies:** `termrock-core`, `termrock-runtime`, `termrock-text`, `termrock-viewport`, `termrock-navigation`, `termrock-overlays`.

**Source hints, to reconcile:** `src/widgets/code.rs`, `src/widgets/diff.rs`.

### `termrock-forms`

Path: `crates/termrock-forms`.

Form and Wizard orchestration, field order/eligibility, generic validation and typed submit/navigation requests.

**Boundary:** No account/workspace schemas or persistence. Composition consumes public child APIs.

**Allowed internal dependencies:** `termrock-core`, `termrock-runtime`, `termrock-controls`, `termrock-fields`, `termrock-navigation`, `termrock-overlays`.

**Source hints, to reconcile:** `src/bin/jackin_preview/screens/editor.rs`, `src/bin/jackin_preview/screens/prelude.rs`.

### `termrock-terminal`

Path: `crates/termrock-terminal`.

TerminalView for caller-prepared terminal cells, cursor, continuation, selection and typed input/link/copy requests.

**Boundary:** No escape parser, PTY spawner, daemon or process manager. Verify exact source locations during census.

**Allowed internal dependencies:** `termrock-core`, `termrock-runtime`, `termrock-render`, `termrock-collections`, `termrock-text`.

**Source hints, to reconcile:** `src/termrock/terminal_view.rs`, `src/bin/jackin_preview/screens/capsule.rs`.

### `termrock-session`

Path: `crates/termrock-session`.

Optional platform session/backend edge: terminal enter/leave, input conversion, presentation and restoration/suspend-resume.

**Boundary:** Terminal IO is allowed here, not in pure components. No CLI product, provider, or app business rules.

**Allowed internal dependencies:** `termrock-core`, `termrock-runtime`, `termrock-render`.

**Source hints, to reconcile:** `src/runtime.rs`.


## facade

### `termrock`

Path: `crates/termrock`.

Curated ergonomic public facade and author/theme/layout APIs, re-exporting the actual owning crates and exact shared types.

**Boundary:** Intentionally thin; no legacy adapter dependency, duplicate implementation, or child dependency back to this facade. Session feature stays optional.

**Allowed internal dependencies:** `termrock-core`, `termrock-layout`, `termrock-theme`, `termrock-text`, `termrock-render`, `termrock-collections`, `termrock-runtime`, `termrock-controls`, `termrock-fields`, `termrock-feedback`, `termrock-navigation`, `termrock-viewport`, `termrock-overlays`, `termrock-grid`, `termrock-editors`, `termrock-forms`, `termrock-terminal`, `termrock-session`.

**Source hints, to reconcile:** `src/lib.rs`, `src/termrock/mod.rs`, `main:crates/tui/src/lib.rs`.


## showcase

### `showcase-data`

Path: `crates/showcase-data`.

Deterministic demo records and scenario values independent of TUI widgets.

**Boundary:** No Termrock/Ratatui/backend dependency.

**Allowed internal dependencies:** none.

**Source hints, to reconcile:** `src/bin/showcase/data.rs`.

### `showcase-demos`

Path: `crates/showcase-demos`.

Component demonstration pages and their page-local state/update/draw compositions.

**Boundary:** Every demonstrated standard control uses Termrock public APIs.

**Allowed internal dependencies:** `showcase-data`, `termrock`.

**Source hints, to reconcile:** `src/bin/showcase/pages/`.

### `showcase-ui`

Path: `crates/showcase-ui`.

Showcase shell, navigation, inspector, help and page composition controller.

**Boundary:** No duplicate component widgets or generic runtime owner.

**Allowed internal dependencies:** `showcase-data`, `showcase-demos`, `termrock`.

**Source hints, to reconcile:** `src/bin/showcase/app.rs`.

### `showcase`

Path: `crates/showcase`.

Thin existing showcase CLI binary: parse options, initialize data/session and run the UI.

**Boundary:** Preserve binary name/CLI behavior and rendered title strings; do not place all pages back in main.rs.

**Allowed internal dependencies:** `showcase-data`, `showcase-ui`, `termrock-session`.

**Source hints, to reconcile:** `src/bin/showcase/main.rs`.


## jackin-preview

### `jackin-preview-domain`

Path: `crates/jackin-preview-domain`.

Preview-only account/agent/workspace/instance/usage/configuration values and rules.

**Boundary:** Not the real Jackin product. No visual widgets or IO.

**Allowed internal dependencies:** none.

**Source hints, to reconcile:** `src/bin/jackin_preview/domain/`.

### `jackin-preview-sim`

Path: `crates/jackin-preview-sim`.

Deterministic worlds, jobs, launch, credentials, changes and simulated terminal/process output.

**Boundary:** Keep simulated execution semantics; no provider/Docker/credential-service calls.

**Allowed internal dependencies:** `jackin-preview-domain`.

**Source hints, to reconcile:** `src/bin/jackin_preview/sim/`, `src/bin/jackin_preview/clock.rs`.

### `jackin-preview-presentation`

Path: `crates/jackin-preview-presentation`.

Shared presentation contracts, view projections, generic chrome compositions and product art/animation.

**Boundary:** No host/capsule module dependency; shared contracts avoid circular routing imports.

**Allowed internal dependencies:** `jackin-preview-domain`, `termrock`.

**Source hints, to reconcile:** `src/bin/jackin_preview/screens/mod.rs`, `src/bin/jackin_preview/rain.rs`.

### `jackin-preview-host-ui`

Path: `crates/jackin-preview-host-ui`.

Manager/prelude/editor/settings/accounts/usage/cockpit screen compositions and host overlays.

**Boundary:** Consume projections and return typed app actions; do not rebuild reusable widgets.

**Allowed internal dependencies:** `jackin-preview-domain`, `jackin-preview-presentation`, `termrock`.

**Source hints, to reconcile:** `src/bin/jackin_preview/screens/manager.rs`, `src/bin/jackin_preview/screens/editor.rs`, `src/bin/jackin_preview/screens/accounts.rs`, `src/bin/jackin_preview/screens/cockpit.rs`.

### `jackin-preview-capsule-ui`

Path: `crates/jackin-preview-capsule-ui`.

Capsule tab/pane compositions, inspect/exit/export overlays and product input-prefix coordination.

**Boundary:** Termrock owns reusable input widgets/regions; preview owns product modes and simulated topology.

**Allowed internal dependencies:** `jackin-preview-domain`, `jackin-preview-presentation`, `termrock`.

**Source hints, to reconcile:** `src/bin/jackin_preview/screens/capsule.rs`, `src/bin/jackin_preview/screens/inspect.rs`.

### `jackin-preview-app`

Path: `crates/jackin-preview-app`.

Top-level preview routing, arbiter and effect dispatch joining sim and presentation packages.

**Boundary:** No child crate imports this coordinator; preserve complete fixture-world state transitions.

**Allowed internal dependencies:** `jackin-preview-domain`, `jackin-preview-sim`, `jackin-preview-presentation`, `jackin-preview-host-ui`, `jackin-preview-capsule-ui`, `termrock`.

**Source hints, to reconcile:** `src/bin/jackin_preview/app.rs`, `src/bin/jackin_preview/arbiter.rs`.

### `jackin-preview`

Path: `crates/jackin-preview`.

Thin existing jackin-preview binary and CLI/session bootstrap.

**Boundary:** Do not introduce a real Jackin service dependency or change binary/scenario flags.

**Allowed internal dependencies:** `jackin-preview-app`, `jackin-preview-sim`, `termrock-session`.

**Source hints, to reconcile:** `src/bin/jackin_preview/main.rs`.


## holla

### `holla-domain`

Path: `crates/holla-domain`.

Actions, context, typed effects, resources, policy/trust and plain domain state.

**Boundary:** No TUI widgets, external executors or UI-backedge.

**Allowed internal dependencies:** none.

**Source hints, to reconcile:** `src/bin/holla/domain/action.rs`, `src/bin/holla/domain/context.rs`, `src/bin/holla/domain/effect.rs`.

### `holla-catalog`

Path: `crates/holla-catalog`.

Catalog construction/discovery projections, manifests, ranking and usage learning rules.

**Boundary:** Preserve exact simulated discovery/ranking; no direct shell execution.

**Allowed internal dependencies:** `holla-domain`.

**Source hints, to reconcile:** `src/bin/holla/domain/catalog.rs`, `src/bin/holla/domain/ranking.rs`, `src/bin/holla/domain/manifest.rs`, `src/bin/holla/domain/usage.rs`.

### `holla-plan`

Path: `crates/holla-plan`.

Plan dependency rules, activity/output lifecycle, cancellation and target-bound safety decisions.

**Boundary:** No UI painting or second Termrock runtime; domain plan scheduling is not widget event dispatch.

**Allowed internal dependencies:** `holla-domain`.

**Source hints, to reconcile:** `src/bin/holla/domain/plan.rs`, `src/bin/holla/domain/activity.rs`.

### `holla-sim`

Path: `crates/holla-sim`.

Deterministic worlds, virtual filesystem and existing fixture service outcomes.

**Boundary:** Implement caller-owned effect/fixture ports; no UI dependency or new real operations.

**Allowed internal dependencies:** `holla-domain`, `holla-catalog`, `holla-plan`.

**Source hints, to reconcile:** `src/bin/holla/sim/`, `src/bin/holla/domain/fixtures.rs`, `src/bin/holla/domain/outcomes.rs`.

### `holla-ui`

Path: `crates/holla-ui`.

Finder, plan/activity/resource views, app navigation and typed effect-port coordination.

**Boundary:** Inject sim at composition root; never import sim painters or generic widget clones.

**Allowed internal dependencies:** `holla-domain`, `holla-catalog`, `holla-plan`, `termrock`.

**Source hints, to reconcile:** `src/bin/holla/app.rs`, `src/bin/holla/screens/`.

### `holla`

Path: `crates/holla`.

Thin holla binary wiring CLI options, simulated effect provider and terminal session.

**Boundary:** Keep fixture behavior/flags; no shell command spawns beyond terminal test transport.

**Allowed internal dependencies:** `holla-domain`, `holla-ui`, `holla-sim`, `termrock-session`.

**Source hints, to reconcile:** `src/bin/holla/main.rs`.


## tablepro

### `tablepro-domain`

Path: `crates/tablepro-domain`.

Connection/schema/query/result/history/safety and pending-change domain values.

**Boundary:** Remove any domain dependency on Junie/Termrock Grid CellValue; explicit UI adapters own conversions.

**Allowed internal dependencies:** none.

**Source hints, to reconcile:** `src/bin/tablepro/model.rs`, `src/bin/tablepro/tabs.rs`.

### `tablepro-sql`

Path: `crates/tablepro-sql`.

Existing SQL parsing/classification/completion analysis and deterministic pending-change SQL generation.

**Boundary:** Preserve sealed SQL preview ordering/escaping/null/key vectors; no widget imports.

**Allowed internal dependencies:** `tablepro-domain`.

**Source hints, to reconcile:** `src/bin/tablepro/sql.rs`, `src/bin/tablepro/model.rs`.

### `tablepro-demo`

Path: `crates/tablepro-demo`.

In-memory demo database/catalog, deterministic query outcomes and scenario data.

**Boundary:** Keep the app simulated; no production database drivers.

**Allowed internal dependencies:** `tablepro-domain`, `tablepro-sql`.

**Source hints, to reconcile:** `src/bin/tablepro/db.rs`.

### `tablepro-ui`

Path: `crates/tablepro-ui`.

Connection/workbench/editor/result/table/history/safety presentation and controller interfaces.

**Boundary:** Use generic Termrock Grid, editors and overlays via adapters; no duplicate table engine.

**Allowed internal dependencies:** `tablepro-domain`, `tablepro-sql`, `termrock`.

**Source hints, to reconcile:** `src/bin/tablepro/app.rs`, `src/bin/tablepro/screens/`, `src/bin/tablepro/tabs.rs`.

### `tablepro`

Path: `crates/tablepro`.

Thin tablepro binary wiring CLI, demo provider and session.

**Boundary:** Binary behavior and all six safety modes remain unchanged.

**Allowed internal dependencies:** `tablepro-ui`, `tablepro-demo`, `termrock-session`.

**Source hints, to reconcile:** `src/bin/tablepro/main.rs`.


## verification

### `termrock-test-support`

Path: `crates/termrock-test-support`.

Test-only stable case/checkpoint contracts, synthetic fixture helpers and semantic observation projections.

**Boundary:** Not a dependency of production packages; no reference-side dependency on candidate Termrock types, approval mutation or expected painter.

**Allowed internal dependencies:** none.

**Source hints, to reconcile:** `tests/conformance/registry.rs`.

### `termrock-conformance`

Path: `crates/termrock-conformance`.

Actual public-component and real-app integration/PTY parity tests, artifact verification and negative tests.

**Boundary:** Own Tuiscotti dependency and external oracle process boundary. No old Junie package in candidate workspace/dependency graph.

**Allowed internal dependencies:** `termrock`, `termrock-test-support`, `showcase-ui`, `jackin-preview-app`, `holla-ui`, `holla-sim`, `tablepro-ui`, `tablepro-demo`.

**Source hints, to reconcile:** `tests/visual_baseline/`, `tests/conformance/`, `tests/termrock_*.rs`.

### `termrock-xtask`

Path: `crates/termrock-xtask`.

Small Rust repository gate/coverage/bundle/checklist utilities and explicit reference-capture orchestration.

**Boundary:** Tool implementation, not a general agent/workflow platform. No normal test may admit the real baseline.

**Allowed internal dependencies:** `termrock-test-support`.

**Source hints, to reconcile:** `tests/stage_corpus.rs`.

## Cycle-breaking rules

`termrock-runtime` exposes the component contexts but never imports component crates. `termrock-render` is a cell-painting primitive layer, not a PNG renderer. Shared region/layout fact records belong to lower contracts so painting can report to the runtime without a backedge.

Fields do not depend on popup implementations; Select lives in overlays. Menus/dialogs accept slots so they do not import forms/grid/editors. Editors depend on overlay/viewport contracts, never the reverse. Form orchestration consumes children and returns actions rather than owning product persistence.

App domain/simulation libraries own plain domain types. UI adapters translate to generic Grid/Tree/Props/Text interfaces; importing Grid cell kinds into a SQL model does not count as domain separation. Shared app presentation/action contracts are below host/capsule view crates and the app coordinator. Extract a small shared type into its existing lower owner instead of making a new god-crate or boxing away a cycle.

## Verification

Use Cargo metadata for package names, physical manifests, targets and all relevant feature/target dependency graphs. Parse import/source relationships too: Cargo alone will not catch cross-crate `include!` or a renamed copy of an old painter. Verify binary link paths and mutate standard controls in disposable builds to demonstrate real consumers. Record representative change-rebuild scope and compile/link time; many crates do not automatically imply faster CI.
