# Historical Termrock recovery ledger

## Status

Archived branch-recovery record. It does not direct current recovery, merges, branch deletion, API changes, migration, or readiness decisions.

## Authority

Current direction: [architecture](../architecture.md), [design](../design.md), [status](../status.md), and [plan](../plan.md). Current workspace source and manifests are authoritative. `design/interaction-polish` supplies accepted product behavior and appearance; `main` supplies split architecture direction. Refactoring remains incomplete and not ready.

## Historical evidence

The original ledger compared frozen `main`, target, and PR branch heads for a particular integration at `integrate/unique-0722f4f9`. Its branch names, SHAs, row dispositions, commands, and test results refer only to that snapshot. They are not present-day branch or source claims.

Immutable source: [ledger at cleanup base](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/integration/recovery-ledger.md).

## Remaining work

Re-evaluate any useful API, test, or recovery finding against current source before recording it in the current plan. No ledger row authorizes changing code or deleting a compatibility path today.

## Verification

Use current `git` refs and source files for current state. Verify each proposed code or API change with present consumer evidence.
