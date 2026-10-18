# Termrock implementation plan

This directory owns the future execution sequence for the in-place Termrock
refactor. It does not define a second component API, a second visual contract,
or product roadmaps for the four reference applications.

## Read in this order

1. [`plan.md`](plan.md) — the P0–P7 implementation DAG, phase gates, and work
   packet rules.
2. [`migration.md`](migration.md) — how the current `junie-tui` implementation
   becomes the Termrock library while preserving the frozen applications.
3. [`quality-gates.md`](quality-gates.md) — toolchain, package, implementation
   quality, performance, and independent-review gates.
4. [`../architecture/overview.md`](../architecture/overview.md) — the target
   architecture and ownership boundaries.
5. [`../api/public-api.md`](../api/public-api.md) — the target caller-facing
   API.
6. [`../foundations/README.md`](../foundations/README.md) and
   [`../components/README.md`](../components/README.md) — the shared contracts
   and complete component inventory.
7. [`../verification/README.md`](../verification/README.md) — oracle,
   conformance, and parity gates.

The task catalog under `refactoring-tasks/` turns this plan into executable
work packages. Tasks link to these documents and to the canonical foundation,
API, component, and verification contracts; they must not copy those
contracts into task prose.

## Two-stage execution model and destination

Execution is divided into two canonical stages:

1. **Stage A (Preparation on `termrock-refactor`):**
   - Snapshot, interaction, verification, and CI preparation.
   - Production behavior is not refactored; `src/**` is strictly read-only.
   - Preserves start and final commits descended from annotated `visual-baseline` (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
2. **Stage B (Refactor on `termrock-implementation`):**
   - The Rust Termrock library refactor is executed exclusively on `termrock-implementation`.
   - Branch `termrock-implementation` is created only from the exact accepted Stage A commit.
   - Implements the canonical P1–P7 sequence, bounded consumer-adoption checkpoints, and final parity verification. Never continue production refactoring on `termrock-refactor`.

The current package and source names remain true until a future implementation
step changes them:

- Cargo package: `junie-tui`;
- Rust library name: `junie_tui`;
- current internals may contain `WidgetId`, `Outcome`, `RenderCtx`, and other
  legacy names.

Those names are current implementation facts, not the target public identity.
The future library identity is Termrock. A package, crate, or module rename is
an implementation-phase change with its own compile and consumer gates. This
documentation phase does not perform that rename.

The earlier source pack described a separate `termrock-new` destination. That
assumption is historical and superseded. No phase in this plan creates,
copies, or migrates into another repository.

## Scope and incremental consumer adoption

Every phase is limited to the reusable library, its generic conformance
consumer, and the proof needed to preserve the baseline. The four binaries
remain preserved consumers:

- `showcase` — component laboratory;
- `tablepro` — complex grid/editor/workbench composition;
- `jackin-preview` — host-management and terminal-pane composition;
- `holla` — finder, preview, action, plan, output, and context composition.

Rather than deferring all application migration to a late phase, candidate
components are integrated through bounded consumer-adoption checkpoints at each
phase:
- **P2:** basic controls and semantic chrome (`showcase`, `tablepro`, `jackin-preview`, `holla`);
- **P3:** fields, editing, forms, and choices;
- **P4:** menus, dialogs, pickers, and completion;
- **P5:** grid, output viewports, code editor, diff view, and status;
- **P6:** terminal view and completed public-API application adoption;
- **P7:** full audit, closure, and cross-application visual/interaction parity.

Such incremental adoptions must preserve the frozen output, observable
keyboard/pointer behavior, scenarios, and snapshots. No application product
feature, service, backend, route, or visual redesign belongs in this plan.

## Canonical ownership

| Question | Canonical owner |
|---|---|
| Mission, immutable boundaries, and definition of success | [`GOAL.md`](../../GOAL.md) |
| Target architecture and concern ownership | [`../architecture/`](../architecture/) |
| Public signatures, types, authoring, and parts | [`../api/`](../api/) |
| Shared runtime/foundation contracts | [`../foundations/`](../foundations/) |
| Component behavior and state matrices | [`../components/`](../components/) |
| Visual and interaction contracts | [`../design/`](../design/) |
| Oracle provenance and exact parity gates | [`../verification/`](../verification/) |
| Implementation quality and independent review | [`quality-gates.md`](quality-gates.md) |
| Preserved application roles and scenarios | [`../applications/`](../applications/) |
| Executable dependency graph and task metadata | [`../../refactoring-tasks/`](../../refactoring-tasks/) |

This directory records sequencing, dependencies, migration mechanics, and
phase acceptance. If a phase needs a behavior rule, it links to the owner
above instead of defining a competing rule.

Machine-readable coverage remains beside the canonical contracts under
`docs/reference/`: component inventory, foundation inventory, legacy-family
disposition, source/tool locks, and capture-plan metadata. These manifests
describe requirements and provenance; they do not contain approved candidate
output.

## Stop line

The plan is complete when the same repository exposes the Termrock public
library contract, every applicable baseline case is compared against the
trusted oracle, generic consumers compile using public exports, and the four
applications still match the frozen visual and interaction reference. Product
integration remains a separately approved effort after this library plan.
