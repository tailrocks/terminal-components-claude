---
schema: task/v5
id: TASK-015
title: "Complete application adoption and prove composed conformance"
kind: refactor
---

# TASK-015 — Complete application adoption and prove composed conformance

## Goal

The four preserved applications complete adoption of the refactored reusable components and prove full composed conformance while retaining their frozen observable behavior.

## Context

The applications are reference consumers and integration fixtures, not product targets. Building upon the bounded consumer-adoption checkpoints established across phases P2–P6, this package completes public-API application adoption across all four reference applications (`showcase`, `tablepro`, `jackin-preview`, and `holla`), finalizes the TablePro `preview_sql` adapter, and validates end-to-end composed conformance against the frozen oracle. Its verifier enumerates exact existing presentation files that call reusable TUI APIs; it does not grant an entire application directory. The files may change only where needed to adopt the public Termrock components and caller-owned state. Scenarios, product behavior, source data, and outputs remain unchanged.

Dependencies: TASK-008, TASK-010, TASK-014. Canonical contracts: [in-place migration](../../../docs/implementation/migration.md), [applications/README.md](../../../docs/applications/README.md), [applications/showcase.md](../../../docs/applications/showcase.md), [applications/tablepro.md](../../../docs/applications/tablepro.md), [applications/jackin-preview.md](../../../docs/applications/jackin-preview.md), [applications/holla.md](../../../docs/applications/holla.md), [verification/conformance.md](../../../docs/verification/conformance.md), [verification/visual-parity.md](../../../docs/verification/visual-parity.md).

## Preconditions

- **P-001:** The caller has supplied a committed descendant of annotated `visual-baseline` (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) on `termrock-implementation`.
- **P-002:** The linked canonical architecture, API, component, design, and verification documents are present.
- **P-003:** The future implementation environment provides `rtk` and `cargo nextest`.

## Scope

In scope:

- Consolidating and completing public-API application adoption across the four reference applications (`showcase`, `tablepro`, `jackin-preview`, and `holla`).
- The exact application presentation call-site files listed in `verify.toml`: Showcase app/page composition; TablePro app, connection, tab, and workbench composition; Jackin Preview app/screen composition; Holla app/screen composition; library-level composition probes; preserved application conformance traces; and integration evidence against the immutable oracle.
- Generic additions under `tests/termrock/` and `tests/conformance/` needed to exercise these compositions.
- The exact `src/bin/tablepro/model.rs` path is writable only for the `preview_sql` adapter and its directly related fixture/interface update in `preview_sql_orders_updates_inserts_deletes`, if the new caller-owned Grid model requires that fixture change. Preserve the test's existing behavioral assertions.

Out of scope:

- Any shared library implementation path (`src/lib.rs`, `src/core/`, `src/runtime.rs`, `src/theme.rs`, `src/ui/`, or `src/widgets/`); implementation gaps return to their owning component/foundation task.
- Every application file absent from the exact `verify.toml` allowlist, including app entrypoints/CLI scenario selection, app tests, Showcase data, Jackin/Holla domains, simulators, scenarios, clocks, and art helpers. In `model.rs`, everything outside the narrowly allowed `preview_sql` adapter and its directly related fixture remains out of scope: `sql_literal`, history, completion, switcher, database, query execution, and SQL generation or preview semantics.
- New routes, services, providers, databases, Docker, authorization, Git operations, product redesign, fixture/data changes, and application behavior changes.

## Requirements

- **R-001 (MUST):** Deliver the scoped mechanism according to the linked canonical contracts.
- **R-002 (MUST):** Preserve caller-owned state, borrowed props, typed actions, stable identities, and shared foundation ownership required by the API.
- **R-003 (MUST):** Application edits are limited to completing component adoption in the exact existing presentation files enumerated in `verify.toml` and the specific `preview_sql` adapter exception in R-006. App entrypoints, unrelated app tests, fixture/data/domain/simulation/scenario files, database/SQL modules, snapshots, and visual-baseline tests remain unchanged. The four applications retain their scenarios, observable interaction, and rendered output.
- **R-004 (MUST NOT):** No product implementation, service integration, repository migration, or application redesign is added to this library task.
- **R-005 (MUST):** The completion gate succeeds.
- **R-006 (MUST):** Adapt only `preview_sql`'s data boundary so it consumes TablePro's caller-owned Termrock Grid rows and pending-edit state. Preserve the full ordered SQL output byte-for-byte for equivalent table, column, original-row, and edit inputs, including dirty-row and changed-column order, original-key predicates, NULL/DEFAULT behavior, no-op revert behavior, composite primary keys, and the no-primary-key fallback. Preserve SQL literal formatting and escaping. The existing inline preview regression assertions must remain and may change only as needed to construct the new input fixture. `sql_literal`, query SQL/execution, Safe Mode policy, history, completion, switcher, database fixtures, and all other application-model behavior stay unchanged. An independent diff review must confirm that `model.rs` changes are limited to `preview_sql` and its directly related fixture construction.

## Acceptance criteria

### AC-001 — Composed library scenarios match expected focus, capture, selection, navigation, state, and rendering outcomes for all four applications.
```gherkin
Given the linked canonical contracts and the accepted dependency work
When the scoped Termrock mechanism is exercised
Then its update, measure, draw, and typed-action behavior matches the contract
```

**Verification**

- **Type:** scenario
- **Covers:** `R-001`
- **Check:** `CHK-001`

### AC-002 — Nested overlays, resize, narrow geometry, motion phases, source reconciliation, and cross-component event routing preserve parity.
```gherkin
Given an applicable boundary, interaction, or reconciliation state
When the future implementation is exercised
Then the documented state, identity, geometry, and ownership invariants hold
```

**Verification**

- **Type:** scenario
- **Covers:** `R-002`
- **Check:** `CHK-002`

### AC-003 — Frozen references and scope remain protected
```gherkin
Given the exact presentation-file allowlist and immutable visual-baseline outputs
When the candidate tree is checked after implementation
Then only component-adoption call sites and the R-006 preview input adapter changed; application behavior, rendered output, scenarios, snapshots, baseline tests, and product integrations remain unchanged
```

**Verification**

- **Type:** invariant
- **Covers:** `R-003, R-004`
- **Check:** `CHK-004`

### AC-004 — TablePro preview adapter matches the sealed baseline vectors
```gherkin
Given the sealed TablePro SQL preview ExistingOracle vectors from TASK-001
When the caller-owned Grid model contains the same base rows and pending edits
Then preview_sql returns the same complete ordered SQL strings and the existing preview interaction retains its frozen output
```

**Verification**

- **Type:** invariant
- **Covers:** `R-006`
- **Check:** `CHK-003`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-005`

## Fixed decisions

- **D-001:** Applications remain reference consumers; building on phase checkpoints (P2–P6), this task consolidates and completes public-API adoption across presentation files and typed component-action wiring, plus the TablePro preview input adapter in R-006. It excludes app entrypoints, unrelated app tests, application models, fixtures, domains, simulators, scenarios, and product/backend modules.
- **D-002:** The work stays in this repository on `termrock-implementation`; it never creates a separate implementation repository.
- **D-003:** The package's verifier scope is the enumerated presentation files, the exact `model.rs` file for R-006 only, and generic library/conformance tests; no whole application directory or library implementation path is writable. Task-format grants file-level scope; R-006 defines the required source-range boundary.
- **D-004:** The SQL preview oracle test compares full strings and covers update/insert/delete order, changed-column ordering, original/composite keys, NULL and DEFAULT handling, `DEFAULT VALUES`, no-primary-key fallback, no-op reverts, integer/two-decimal/boolean/text/JSON literals and escaping, and keyed reorder/filter/removal so pending edits cannot retarget another row.

## Checklist

<!-- checklist:start -->
- [x] **1** Prepare.
    - [x] **1.1** Read the linked canonical contracts and confirm the accepted starting tree. (`R-001`, `AC-001`, `CHK-001`)
- [x] **2** Implement.
    - [x] **2.1** Deliver the scoped Termrock mechanism. (`R-001`, `AC-001`, `CHK-001`)
    - [x] **2.2** Prove shared ownership, identity, and applicable edge behavior. (`R-002`, `AC-002`, `CHK-002`)
    - [x] **2.3** Keep edits inside the exact presentation-file allowlist; limit `model.rs` to the reviewed `preview_sql` adapter and its focused fixture; preserve all frozen behavior/output. (`R-003`, `R-004`, `R-006`, `AC-003`, `AC-004`, `CHK-004`, `CHK-003`)
- [x] **3** Verify.
    - [x] **3.1** Run the one completion gate. (`R-005`, `AC-005`, `CHK-005`)
<!-- checklist:end -->
