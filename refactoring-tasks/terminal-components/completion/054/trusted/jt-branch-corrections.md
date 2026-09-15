# Jackin branch-review interaction obligations — TASK-054

## Authority and evidence boundary

This supplement binds BD-JT-10/14/21/25 Cockpit and launch lifecycle families owned primarily by TASK-054. Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Planning branches only.

## JT-COCKPIT-CONTRACTS: independent log/output contracts — BD-JT-25

`COCKPIT-FAILURE` reaches Cockpit with a failed run visible.

1. **cockpit-failure-runid-copy-reopens-ack:** copy failure run id. Require acknowledgement dialog reopens701–706 after copy.
2. **cockpit-log-before-output-status:** during build. Require log-before-output ordering605–618 and wrapped styled spans509–528.
3. **cockpit-effective-account-set:** launch with multiple agents. Require complete effective account set99–113, not first agent only.

Sources: oracle `screens/cockpit.rs:509–528,605–618,701–706,740–772`.

## JT-HARDCASES-PLANS; JT-RAIN-CHOREOGRAPHY; JT-WORLD-POLICY

HardCases launch plan producers and rain/handoff choreography bind through JA-002/003/040/041/043/062 with full-frame witnesses from BD-JT-14/21.

## Verification binding

R-001/CHK-004 and R-002/CHK-006 bind named branches; TASK-057 closes Jackin union.
