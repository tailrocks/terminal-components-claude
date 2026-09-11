# Independent final architecture/API plan review

Review date: 2026-09-11. Perspective: accepted architecture, API ownership, PARTS/slots, semantic verification and executable task boundaries. This reviewer did not author the reviewed task packages. No production code or task contract was changed.

## Reviewed identities and limits

The checkout is `2e2401393c47360741ebd321679de08982dca50a`. The locally resolved immutable oracle is `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural authority remains `7b27732a8c3c131760ec3438f641cb3c11343a42`. Planning files are untracked, so the checkout SHA does not identify their contents.

Reviewed the complete planning goal and all 24 root-plan sections; architecture assessment and all eight adjudications; the 620 canonical historical clauses and reconciliation; 32 architecture contracts and 54 component families; targeted traceability edges; proof-stage contracts; reusable task requirements, fixed decisions and writable scopes for TASK-009–031, TASK-065–068 and TASK-073. Examined TASK-006 and TASK-072 contracts where their products control architectural proof. This is a semantic plan review, not execution of future production tests or certification of the independent Rust qualification fixtures under concurrent preparation.

Scope fingerprint: SHA-256 `cf28faac0cd8911c980179deb783958c95f7d81108a30667252f6bdf17d97562` over the sorted newline-terminated manifest `sha256 + two spaces + relative path` for these ten documents and every file under the 28 task directories named above: `PLANNING_GOAL.md`, `REFACTORING_COMPLETION_PLAN.md`, `architecture.md`, `architecture-adjudication.md`, `historical-obligations-canonical.tsv`, `history-ledger-reconciliation.md`, `architecture-matrix.tsv`, `component-parity.tsv`, `traceability.tsv`, `proof-contract.md` (the last eight under `docs/refactoring-plan`). There were 178 files.

Individual fingerprints at review:

- Root plan: `91e0aa45b6ca74fd6fdaf9776776709dd5c9dde263894956c509b731cddc17ef`.
- Traceability: `c1ba0f0c4e1ceaed1e6b7882d7039ac492e1271023097ae6800c2064edbda73f`.
- TASK-031 README: `0597c759d07261db6ebff7bf5d170d3fc99cb08a7eefd7e30c7049859d8ad608`.
- TASK-031 verify.toml: `5133ae3251cbd43e90f2c31f0296bc60fcbbb86de49e2538d0eca51411fa90ba`.
- TASK-073 README: `b5298c7a2923b682297c305285f2b34e80238ec751a8048961595b70124ab143`.

## ARCH-FINAL-01 — P2: TASK-031's primary proof still asserts an unspecified historical testing-registry oracle

**Sources.** `refactoring-tasks/terminal-components/completion/031/README.md:54–65` requires “sealed testing-registry oracle cases” and assigns its R-001 conformance outcome to CHK-004. Lines 113–124 additionally require direct-lane testing-registry actions. `031/verify.toml:30–36` defines CHK-004 exclusively as the oracle comparator. `docs/refactoring-plan/traceability.tsv:1592` explicitly claims that this edge proves “exact owned PARTS and advertised painted slots.” The component matrix's `testing-registry` row identifies old module/integration tests as its reference; it does not identify a historical production registry widget or an oracle PARTS/slot implementation. TASK-006 R-001 still asks to expand all 54 rows into production widget/view fixtures; its R-002 permits honest old-to-new composition mappings but does not supply one for this architectural family.

**Why this matters.** Exact unchanged cells cannot prove equality of the declared PARTS set, registry membership or documented slot promises. Adding an unreachable declared part or removing a registry invocation can leave every oracle frame unchanged. Those obligations are architectural metadata and executed-conformance properties of main. An executor must currently infer what the undefined historical testing-registry fixture means, or assign an arbitrary component screenshot to it. Neither is the specified mechanical proof of R-001.

This is not a claim that the whole task can bypass every gate: HIST:A43/HM05/HM06/HM07/HM48 correctly map to CHK-006 through R-002, and those retained gates must still run. The defect is the contradictory primary component-to-proof mapping and unnecessary undefined oracle precondition. TASK-073's corrected D-006 already recognizes this distinction, but TASK-031 and the COMP baseline edge retain the old pattern.

**Required repair.** Make TASK-031's primary PARTS/registry/painted-slot acceptance an architecture check with explicit nonempty real-state sweeps and the protected mutation controls. Update its requirement/acceptance/check mappings and the COMP traceability edge. Give `COMP:testing-registry` an explicit architecture-only baseline disposition in TASK-006. If TASK-031 additionally protects real production-view parity, name that separately using existing concrete component or application oracle identities; do not call architecture fixtures historical oracle frames. Preserve complete regression accounting and the later full application closure. Verify that a phantom PARTS member and a dropped registry invocation fail the primary architecture acceptance while unchanged legitimate instrumentation passes.

**Re-review.** Recheck TASK-006, TASK-031, TASK-073 and both directions of COMP/HIST traceability after correction. Re-run the structural validator, but do not treat its success as proof of this semantic repair.

## Other adversarial checks

No additional independently substantiated rejected-design revival was found in the reviewed clauses. In particular, the plan retains receiver-free configured Props construction, real identity in provenance, separate component/row ownership, the limited seed-reset policy, private Form bridges, exact ASCII opt-in mapping, backend-free consumers and the prohibition on app-owned generic painters. The borrowed row-author override channel in `components.md` explicitly forbids broadcasting component patches into arbitrary row-owned parts; this does not revive the rejected propagation scheme.

No approval is implied for the concurrently open host/runner/Rust-probe qualification work. Material repairs to that proof authority or the reviewed task semantics require another independent review. The finding above remains open until repaired or rejected with a concrete source-qualified oracle mapping and check contract.
