# Reconciled execution graph

This graph is generated from the canonical task index, every direct and nested task contract, and traceability. It is structural planning data only: it contains no task status, acceptance result, ledger mutation, or dispatch authorization.

The graph contains 73 direct tasks, 77 recursive verify.toml contracts, 506 direct checks, 526 recursive checks, and 276 dependency edges.

The maximum dependency depth is 35 with 24 equally deepest dependency paths. Complete machine-checkable metadata is in task-graph.json.

## Catalog audit

| Measure | Derived count | Authority |
| --- | ---: | --- |
| Direct task packages | 73 | task-index.tsv and direct package directories |
| Recursive verify.toml contracts | 77 | all package descendants |
| Direct checks | 506 | direct contracts |
| Recursive checks | 526 | direct plus nested contracts |
| Dependency edges | 276 | task.toml dependencies |
| Shared traceability interfaces | 584 | traceability.tsv source/task membership |
| Writable-scope conflict pairs | 193 | direct verify.toml writable paths |
| Incomparable writable-scope locks | 0 | conflicts without a dependency ordering |

The four nested contracts are retained as catalog members, not extra dispatchable tasks:

| Owner | Declared nested task | Checks | Contract |
| --- | --- | ---: | --- |
| TASK-001 | TASK-901 | 5 | refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/host-bootstrap-fixture/verify.toml |
| TASK-070 | TASK-901 | 5 | refactoring-tasks/terminal-components/completion/070/trusted/proof-bootstrap/host-bootstrap-fixture/verify.toml |
| TASK-071 | TASK-901 | 5 | refactoring-tasks/terminal-components/completion/071/trusted/proof-bootstrap/host-bootstrap-fixture/verify.toml |
| TASK-072 | TASK-901 | 5 | refactoring-tasks/terminal-components/completion/072/trusted/proof-bootstrap/host-bootstrap-fixture/verify.toml |

## Execution contract

- Every canonical dependency is also a verification dependency requiring an accepted, ancestry-bound dependency receipt.
- Acceptance joins require all declared dependencies and the direct contract gate check.
- A task's own receipt is never required to start that task; final-product success is never a prerequisite for the first implementation task.
- Equal-depth tasks are only parallel candidates. File conflicts, isolated worktrees/run directories, independent verification, and fresh join verification still apply.
- Migration boundaries are derived from dependency edges whose canonical task-index workstream changes. File ownership and rollback scopes are taken directly from each direct verify.toml.

## One exact longest dependency path

TASK-001 -> TASK-070 -> TASK-071 -> TASK-072 -> TASK-002 -> TASK-006 -> TASK-008 -> TASK-073 -> TASK-009 -> TASK-010 -> TASK-014 -> TASK-015 -> TASK-018 -> TASK-019 -> TASK-023 -> TASK-024 -> TASK-027 -> TASK-028 -> TASK-030 -> TASK-031 -> TASK-040 -> TASK-041 -> TASK-042 -> TASK-043 -> TASK-044 -> TASK-045 -> TASK-046 -> TASK-047 -> TASK-048 -> TASK-049 -> TASK-050 -> TASK-065 -> TASK-066 -> TASK-068 -> TASK-069

## Dependency layers and parallel candidates

Tasks in one layer have the same dependency depth. They are not automatically safe to dispatch together.

| Depth | Tasks | Workstreams | Same-depth conflicts |
| ---: | --- | --- | ---: |
| 1 | TASK-001 | B | 0 |
| 2 | TASK-070 | B | 0 |
| 3 | TASK-071 | B | 0 |
| 4 | TASK-072 | B | 0 |
| 5 | TASK-002, TASK-003, TASK-004, TASK-005, TASK-007 | B | 0 |
| 6 | TASK-006 | B | 0 |
| 7 | TASK-008 | B | 0 |
| 8 | TASK-073 | B | 0 |
| 9 | TASK-009, TASK-011, TASK-012 | C | 0 |
| 10 | TASK-010, TASK-013, TASK-029 | C | 0 |
| 11 | TASK-014 | C | 0 |
| 12 | TASK-015, TASK-016, TASK-017, TASK-021 | C | 0 |
| 13 | TASK-018, TASK-022, TASK-026 | C | 0 |
| 14 | TASK-019, TASK-020 | C | 0 |
| 15 | TASK-023 | C | 0 |
| 16 | TASK-024 | C | 0 |
| 17 | TASK-025, TASK-027 | C | 0 |
| 18 | TASK-028 | C | 0 |
| 19 | TASK-030 | C | 0 |
| 20 | TASK-031 | C | 0 |
| 21 | TASK-032, TASK-040, TASK-051, TASK-058 | S, H, J, T | 0 |
| 22 | TASK-033, TASK-041, TASK-052, TASK-059 | S, H, J, T | 0 |
| 23 | TASK-034, TASK-042, TASK-053, TASK-060 | S, H, J, T | 0 |
| 24 | TASK-035, TASK-043, TASK-054, TASK-061 | S, H, J, T | 0 |
| 25 | TASK-036, TASK-044, TASK-055, TASK-062 | S, H, J, T | 0 |
| 26 | TASK-037, TASK-045, TASK-056, TASK-063 | S, H, J, T | 0 |
| 27 | TASK-038, TASK-046, TASK-057, TASK-064 | S, H, J, T | 0 |
| 28 | TASK-039, TASK-047 | S, H | 0 |
| 29 | TASK-048 | H | 0 |
| 30 | TASK-049 | H | 0 |
| 31 | TASK-050 | H | 0 |
| 32 | TASK-065, TASK-067 | X | 0 |
| 33 | TASK-066 | X | 0 |
| 34 | TASK-068 | X | 0 |
| 35 | TASK-069 | X | 0 |

## Migration boundaries

| From workstream | To workstream | Dependency edges |
| --- | --- | ---: |
| B | C | 50 |
| B | X | 1 |
| C | S | 12 |
| C | H | 10 |
| C | J | 11 |
| C | T | 9 |
| C | X | 27 |
| S | X | 3 |
| H | X | 3 |
| J | X | 3 |
| T | X | 3 |

## Integration checkpoints

These checkpoints are selected only from named task-index key suffixes: COMPONENTS, CONFORMANCE, CLOSE, OWNERSHIP, TESTS, PERF, API, and FINAL. Their checks remain task-owned.

| Task | Key | Depth | Gate checks |
| --- | --- | ---: | --- |
| TASK-006 | B-COMPONENTS | 6 | CHK-007 |
| TASK-031 | C-CONFORMANCE | 20 | CHK-007 |
| TASK-039 | S-CLOSE | 28 | CHK-007 |
| TASK-050 | H-CLOSE | 31 | CHK-007 |
| TASK-057 | J-CLOSE | 27 | CHK-007 |
| TASK-064 | T-CLOSE | 27 | CHK-007 |
| TASK-065 | X-OWNERSHIP | 32 | CHK-007 |
| TASK-066 | X-TESTS | 33 | CHK-007 |
| TASK-067 | X-PERF | 32 | CHK-007 |
| TASK-068 | X-API | 34 | CHK-007 |
| TASK-069 | X-FINAL | 35 | CHK-007 |
| TASK-073 | B-CONFORMANCE-FOUNDATION | 8 | CHK-007 |

## File ownership and recovery

Each task's writable and forbidden paths, direct contract, shared traceability interfaces, risk/parity source IDs, acceptance join, recursive contracts, and rollback boundary are machine-readable per task in task-graph.json. A failed candidate is recovered by discarding the unintegrated candidate and retrying from its accepted parent; widening scope or mutating forbidden paths is not a recovery path.

## Shared interfaces and conflicts

traceability.tsv yields 584 shared source/task interfaces. It also yields 193 writable-scope conflict pairs; all currently have a transitive dependency ordering, so no incomparable pair is dispatchable concurrently. The full path pairs and ordering proof are in file_conflicts and serialization_locks.

At every parallel join, materialize a fresh combined tree and rerun the union of impacted contracts, complete test accounting, and workspace gates. Individually accepted siblings are not proof of the combined result. Compare-and-swap integration rejects a changed parent; it never silently attaches a tested tree to a different parent.