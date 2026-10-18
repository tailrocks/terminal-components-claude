# Engineering coordination

## Work ownership

Before making changes, identify the task's owner, source branch, paths in scope, and acceptance evidence. Keep one active writer per file. Split independent work by path or use separate worktrees when changes overlap; review the resulting integration as a whole.

Preserve other contributors' uncommitted and committed work. Do not reset, clean, restore, or check out files outside your ownership. Before integration, inspect the full diff and resolve conflicts by comparing intended behavior with current source and project decisions, not by selecting a whole side mechanically.

## Evidence and handoff

For each completed scope, report the exact source revision, paths changed, commands run, results, failures, and unverified behavior. A passing check applies only to the code and targets it exercised. Product behavior claims require the corresponding acceptance in [the plan](docs/plan.md) and comparisons against [the design authority](docs/design.md).

Keep one canonical home for each current contract. Generated publisher records, hash-pinned capture payloads, safety rationales, and historical evidence have distinct purposes; update them only within their owning workflow and preserve their recorded identities. Do not use an old task diary or a generated status page as current project direction.
