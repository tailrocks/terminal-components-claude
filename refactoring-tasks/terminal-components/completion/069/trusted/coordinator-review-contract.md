# TASK-069 coordinator adversarial review (non-machine)

Binding: campaign [execution prompt](/work/docs/refactoring-plan/campaign-execution-prompt.md) §“Final adversarial campaign review” and §“Definition of done”.

TASK-069 **CHK-007** success (`tc-proof close`) is **necessary but not sufficient** for campaign completion. It establishes machine merge-readiness closure only: joining host-authentic passed records for CHK-001–CHK-006 on the exact frozen tree.

The coordinator must record, per perspective, reviewer identity (fresh context), tree SHA reviewed, verdict (`VERIFIED` | `REJECTED` | `BLOCKED`), and disposition of every material finding.

## Required perspectives (minimum)

- Architecture / reusable API
- Showcase parity
- Holla parity
- Jackin parity
- TablePro parity
- Visual parity
- Test preservation
- Proof-chain integrity
- Git ancestry and receipts

## Evidence path

Campaign ledger entry `adversarial-review/<tree-sha>.json` (operator-owned; never writable by task executors or `artifacts/refactoring-completion`).

Historical sources with human-review portions: HIST:A130 (review clause), HIST:EARLY-AMEND-053, HIST:RG51 (review challenge portion). Machine gate portions remain on CHK-005/006/007 as specified in `host-context/*.template.json`.
