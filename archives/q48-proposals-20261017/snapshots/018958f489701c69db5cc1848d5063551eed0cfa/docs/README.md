# Termrock documentation

This tree is the repository's current specification for its in-place refactor into Termrock.

## Two-Stage Execution Model

1. **Stage A: Preparation on `termrock-refactor`**
   - Snapshot, interaction, verification, and CI preparation without refactoring production behavior (`src/**` strictly read-only).
   - Establishes qualified baseline captures, machine-checkable requirements registry, and CI infrastructure.
2. **Stage B: Implementation on `termrock-implementation`**
   - In-place Rust Termrock library refactor executed exclusively on `termrock-implementation` (forked only from the accepted Stage A commit).
   - Executes the canonical P1–P7 phase sequence with bounded consumer-adoption checkpoints across all four applications (`showcase`, `tablepro`, `jackin-preview`, `holla`).

## Authority

1. [`GOAL.md`](../GOAL.md) and [`termrock-preparation-and-refactoring-goal.md`](../termrock-preparation-and-refactoring-goal.md) own the current mission, scope, protected baseline, and success conditions.
2. [`architecture/overview.md`](architecture/overview.md) owns the high-level map. Detailed contracts live once under `api/`, `design/`, `foundations/`, `components/`, and `verification/`.
3. [`implementation/plan.md`](implementation/plan.md) owns phase ordering and phase gates; [`refactoring-tasks/`](../refactoring-tasks/README.md) turns it into task packages that link back to contracts.
4. Frozen source and approved output at `visual-baseline` commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` own visual and observable interaction evidence.
5. Git history retains superseded planning decisions. Old working-tree plans are removed after migration.

The supplied `termrock-library-spec/` directory was migration input. The refactor stays in this repository across the two canonical branches (`termrock-refactor` for preparation and `termrock-implementation` for implementation); no separate Termrock repository is planned. Machine-readable component, family, provenance, and capture references are under `reference/`.

## Reading map

- [Architecture overview](architecture/overview.md) — target ownership, boundaries, and two-stage model.
- [Public API](api/README.md) — props, state, phases, types, and authoring.
- [Visual and interaction contracts](design/visual-contract.md) — baseline presentation and input grammar.
- [Foundations](foundations/README.md) — one owner for each shared mechanism.
- [Components](components/README.md) — all 45 public component/API surfaces.
- [Verification](verification/README.md) — frozen oracle, parity, provenance, and conformance.
- [Applications](applications/README.md) — preserved conformance consumers and current run commands.
- [Implementation](implementation/README.md) — in-place P0–P7 sequence, two-stage model, and bounded consumer-adoption checkpoints.
- [Task catalog](../refactoring-tasks/README.md) — task packages, implementation preconditions on `termrock-implementation`, and incremental consumer adoption.

## Consumer migration model

Rather than deferring all application changes to a single late task, candidate Termrock components are integrated through bounded consumer-adoption checkpoints at each phase:
- **P2:** basic controls and semantic chrome (`showcase`, `tablepro`, `jackin-preview`, `holla`).
- **P3:** fields, editing, forms, and choices.
- **P4:** menus, dialogs, pickers, and completion.
- **P5:** grid, output viewports, code editor, diff view, and status.
- **P6:** terminal view and completed public-API application adoption.
- **P7:** full audit, closure, and cross-application visual/interaction parity.

## Current source names

The frozen source still builds package `junie-tui` and library `junie_tui`. Names such as `WidgetId`, `Outcome`, and `RenderCtx` describe current implementation only. They are not future Termrock compatibility obligations. Literal app-visible strings such as `jackin❯` and `holla❯` remain part of the oracle.
