# Derived execution graph

This graph is generated from canonical task.toml dependencies and verify.toml scopes. Task numbers do not define execution order. Dependency depth is an unweighted critical-path measure; no invented duration estimate is used.

The graph contains 73 tasks, maximum depth 35, and 24 equally deepest dependency paths. The complete predecessor representation is in [task-graph.json](task-graph.json).

## One exact longest dependency path

`TASK-001 → TASK-070 → TASK-071 → TASK-072 → TASK-002 → TASK-006 → TASK-008 → TASK-073 → TASK-009 → TASK-010 → TASK-014 → TASK-015 → TASK-018 → TASK-019 → TASK-023 → TASK-024 → TASK-027 → TASK-028 → TASK-030 → TASK-031 → TASK-040 → TASK-041 → TASK-042 → TASK-043 → TASK-044 → TASK-045 → TASK-046 → TASK-047 → TASK-048 → TASK-049 → TASK-050 → TASK-065 → TASK-066 → TASK-068 → TASK-069`

## Earliest dependency layers

Tasks in one layer are only candidates for parallel execution. Apply the shared-file locks below and require actual prerequisite code/trust receipts, not metadata status alone.

| Depth | Tasks |
| --- | --- |
| 1 | `TASK-001` |
| 2 | `TASK-070` |
| 3 | `TASK-071` |
| 4 | `TASK-072` |
| 5 | `TASK-002`, `TASK-003`, `TASK-004`, `TASK-005`, `TASK-007` |
| 6 | `TASK-006` |
| 7 | `TASK-008` |
| 8 | `TASK-073` |
| 9 | `TASK-009`, `TASK-011`, `TASK-012` |
| 10 | `TASK-010`, `TASK-013`, `TASK-029` |
| 11 | `TASK-014` |
| 12 | `TASK-015`, `TASK-016`, `TASK-017`, `TASK-021` |
| 13 | `TASK-018`, `TASK-022`, `TASK-026` |
| 14 | `TASK-019`, `TASK-020` |
| 15 | `TASK-023` |
| 16 | `TASK-024` |
| 17 | `TASK-025`, `TASK-027` |
| 18 | `TASK-028` |
| 19 | `TASK-030` |
| 20 | `TASK-031` |
| 21 | `TASK-032`, `TASK-040`, `TASK-051`, `TASK-058` |
| 22 | `TASK-033`, `TASK-041`, `TASK-052`, `TASK-059` |
| 23 | `TASK-034`, `TASK-042`, `TASK-053`, `TASK-060` |
| 24 | `TASK-035`, `TASK-043`, `TASK-054`, `TASK-061` |
| 25 | `TASK-036`, `TASK-044`, `TASK-055`, `TASK-062` |
| 26 | `TASK-037`, `TASK-045`, `TASK-056`, `TASK-063` |
| 27 | `TASK-038`, `TASK-046`, `TASK-057`, `TASK-064` |
| 28 | `TASK-039`, `TASK-047` |
| 29 | `TASK-048` |
| 30 | `TASK-049` |
| 31 | `TASK-050` |
| 32 | `TASK-065`, `TASK-067` |
| 33 | `TASK-066` |
| 34 | `TASK-068` |
| 35 | `TASK-069` |

## Shared-file serialization

Disjoint ready work may run in isolated worktrees. The following incomparable tasks have overlapping writable scope and must not execute concurrently. When both are ready, dispatch the lower task ID first, integrate its verified tree, and start the other from that accepted parent. This is a declared soft scheduling constraint, not an invented task.toml field. If only the higher task is ready, it may run first; the later task must still start after its verified integration. No two such tasks share an executor or target/output directory.

| Tasks | Overlapping scopes |
| --- | --- |
| None | Hard dependencies already serialize every scope overlap |

At every parallel join, materialize a fresh combined tree and rerun the union of impacted contracts, complete test accounting and workspace gates. Individually accepted siblings are not proof of the combined result. Compare-and-swap integration rejects a changed parent; it never silently attaches a tested tree to a different parent.
