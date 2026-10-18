# Termrock refactoring task catalog

This catalog is the executable future implementation plan for the in-place Termrock refactor.
Execution is structured under a canonical two-stage model:

- **Stage A (Preparation on `termrock-refactor`):** Snapshot, interaction, verification, and CI preparation without refactoring production behavior (`src/**` strictly read-only).
- **Stage B (Implementation on `termrock-implementation`):** Created only from the accepted Stage A commit. All implementation tasks (P1–P7) execute exclusively on `termrock-implementation`. No task creates a clean-room repository or a second project.

The four existing applications (`showcase`, `tablepro`, `jackin-preview`, `holla`) remain parity consumers and integration fixtures. They are not product targets. Every package protects `snapshots/**` and `tests/visual_baseline/**`. Rather than deferring application migration to a single late step, candidate components are integrated incrementally through bounded consumer-adoption checkpoints at each phase (P2 basic controls/chrome, P3 fields/editing/forms, P4 menus/dialogs/pickers, P5 grid/output, P6 terminal view, P7 full audit). TASK-015 completes and consolidates public-API application adoption across all four apps and finalizes the TablePro `preview_sql` adapter. All application changes require exact frozen output and interaction parity; the SQL adapter must match the baseline's sealed statement vectors byte-for-byte.

## Format and execution

Packages use [task-format revision `6348230a7197ddbe3db164c53c3e8ee1ba256f06`](https://github.com/donbeave/task-format/blob/6348230a7197ddbe3db164c53c3e8ee1ba256f06/reference/FORMAT.md)
and `taskfmt 0.2.0`, with `task/v5` README contracts and `verify/v2` verifier contracts. The current format has no `task.toml`,
`task-meta/v1`, project command, or predecessor field. Lint all packages with:

```sh
for task in refactoring-tasks/P*/TASK-*; do
  taskfmt lint "$task" || exit
done
```

The verifier commands are future implementation checks. They use `rtk cargo nextest`; no Rust
implementation or test run is part of this documentation-only change. Each package has one final
gate check. The final gate intentionally runs the workspace suite because shared runtime and
rendering changes cross component boundaries. Incremental consumer adoption at each phase verifies
that candidate components work in real application compositions without altering scenarios,
fixtures, product behavior, or rendered output. The TablePro `preview_sql` input adapter required to consume
caller-owned Grid state is finalized in P6/TASK-015; it cannot change fixtures, scenarios, product behavior, or output. The
adapter boundary and parity rule are owned by
[`migration.md`](../docs/implementation/migration.md).

## Dependency DAG

Dependencies are catalog relationships, not fabricated Git SHAs. An implementation task can start only from the
actual committed descendant accepted for its dependencies on `termrock-implementation`.

| Phase | Package | Outcome | Depends on |
| --- | --- | --- | --- |
| P0 | [TASK-001](P0/TASK-001/README.md) | Qualify the oracle, conformance package, pinned toolchain, and independent comparator | — |
| P1 | [TASK-002](P1/TASK-002/README.md) | Identity, events, responses, and constrained public API | TASK-001 |
| P1 | [TASK-003](P1/TASK-003/README.md) | Runtime focus, hit testing, capture, time, and layers | TASK-001, TASK-002 |
| P1 | [TASK-004](P1/TASK-004/README.md) | Measurement, layout, theme, text, and authoring foundations | TASK-002, TASK-003 |
| P2 | [TASK-005](P2/TASK-005/README.md) | Basic controls, semantic chrome, and consumer-adoption checkpoint | TASK-004 |
| P2 | [TASK-006](P2/TASK-006/README.md) | Keyed collections, reconciliation, and consumer-adoption checkpoint | TASK-004 |
| P3 | [TASK-007](P3/TASK-007/README.md) | Shared text editing core, fields, forms, and consumer-adoption checkpoint | TASK-004, TASK-005 |
| P3 | [TASK-008](P3/TASK-008/README.md) | Choices, controlled values, secrets, validation, and consumer-adoption checkpoint | TASK-006, TASK-007 |
| P4 | [TASK-009](P4/TASK-009/README.md) | Shared overlay, menu engine, and consumer-adoption checkpoint | TASK-003, TASK-005 |
| P4 | [TASK-010](P4/TASK-010/README.md) | Picker, command palette, chains, completion, and consumer-adoption checkpoint | TASK-006, TASK-009 |
| P5 | [TASK-011](P5/TASK-011/README.md) | Panel, split, scroll, and text viewport infrastructure | TASK-004, TASK-006 |
| P5 | [TASK-012](P5/TASK-012/README.md) | Grid, code editing, diff presentation, and consumer-adoption checkpoint | TASK-007, TASK-011 |
| P5 | [TASK-013](P5/TASK-013/README.md) | Progress, spinner, meter, status, hints, help, and consumer-adoption checkpoint | TASK-005, TASK-006, TASK-011 |
| P6 | [TASK-014](P6/TASK-014/README.md) | Prepared-cell TerminalView and terminal edge contracts | TASK-011, TASK-012, TASK-013 |
| P6 | [TASK-015](P6/TASK-015/README.md) | Complete application adoption and composed conformance | TASK-008, TASK-010, TASK-014 |
| P7 | [TASK-016](P7/TASK-016/README.md) | Public API and architecture closure audit | TASK-015 |
| P7 | [TASK-017](P7/TASK-017/README.md) | Complete visual/interaction parity and negative gates across all applications | TASK-016 |
| P7 | [TASK-018](P7/TASK-018/README.md) | Performance evidence and independent release review | TASK-017 |

## Canonical contract ownership

Task packages identify the work boundary and proof. They do not redefine component behavior. Read
the linked canonical contracts before implementation:

- `docs/architecture/overview.md` owns the target architecture and ownership boundaries.
- `docs/api/public-api.md` owns `update`, `draw`, `measure`, props, state, actions, and IDs.
- `docs/foundations/` owns shared identity, events, runtime, layout, theme, text, collections,
  secrets, authoring, and conformance mechanisms.
- `docs/components/` owns the complete public component/API inventory and component behavior.
- `docs/design/` owns visual and interaction parity.
- `docs/verification/` owns oracle provenance, exact comparison, state coverage, and mutation gates.
- `docs/implementation/plan.md` owns phase sequencing and in-place migration strategy.
- `docs/implementation/quality-gates.md` owns the pinned toolchain, package/dependency boundaries, implementation-quality blockers, performance evidence, and independent-review protocol.

Application migration is performed incrementally through bounded consumer-adoption checkpoints
at each phase while preserving the exact frozen Showcase, TablePro, Jackin Preview, and Holla
behavior. TASK-015 completes and consolidates the migration across all four applications and
finalizes the TablePro preview adapter. No package authorizes services, providers, Docker, account
authorization, Git operations, database backends, shell/PTY runtime, or product redesign.
