# Work queue

Rendered by the integration owner from [accepted task records](docs/implementation/visibility/tasks.json).
Queue revision: 6. Acceptance owner: `/root` (single integrator).

Highest open priority: **P0 critical**. Product refactoring waits for the visibility milestone.

| Work | Priority | State | Owner | Reviewer |
| --- | --- | --- | --- | --- |
| VIS-01 | P0 | review | `/root/status_luna` | `/root/technical_review` |
| VIS-02 | P0 | in_progress | `/root/suite_luna` | `/root/technical_review` |
| VIS-03 | P0 | review | `/root/showcase_owner` | `/root/technical_review` |
| VIS-04 | P0 | in_progress | `/root/deferred_execution` | `/root/technical_review` |
| VIS-05 | P0 | in_progress | `/root/coordination_luna` | `/root/technical_review` |

The accepted record defines paths, the starting commit, the claim token, the expiry time, dependencies, and evidence.
The expiry time does not permit a new owner. Record a safe handoff first.

Queue validation and automatic claim acceptance are **not implemented** in this first increment.
This record grants work scope. It does not prove test results or requirement completion.
Product requirements remain in [CHECKLIST.md](CHECKLIST.md) and `checklist.json`.
