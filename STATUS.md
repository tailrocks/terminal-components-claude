# Termrock status

| Conclusion | State | Evidence |
| --- | --- | --- |
| Visibility / Complete | NOT_RUN | The shared suite, paired results, and publication checks have not been accepted as a complete current run. |
| Refactor / Ready | NOT_RUN | No complete current evidence covers required visual, interaction, API, ownership, and review checks. |
| Reference / Qualified | NOT_RUN | The tag identity is recorded, but no paired reference execution is qualified. |
| Command / Ready | NOT_RUN | No exact root command has current execution evidence. |
| Evidence freshness | NOT_RUN | No validated paired run receipt is available for the measured source pair. |

## Required commands

Each result cell shows reference and candidate separately. Ready has its own column, separate from Build, Launch, First frame, input and interaction, Exit, Restoration, Visual, and Ownership.
Reference ownership is NOT_APPLICABLE because the reference is not required to use the candidate Termrock architecture.

| ID | Exact root command | Build | Launch | First frame | Interaction | Exit | Restoration | Visual | Ownership | Ready |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| RUN-01 | cargo run --release --bin showcase | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_APPLICABLE; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN |
| RUN-02 | cargo run --release --bin jackin-preview | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_APPLICABLE; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN |
| RUN-03 | cargo run --release --bin holla | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_APPLICABLE; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN |
| RUN-04 | cargo run --release --bin tablepro | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_APPLICABLE; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN |
| RUN-05 | cargo run --release --bin tablepro -- --connect Production | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_APPLICABLE; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN |
| RUN-06 | cargo run --release --bin jackin-preview -- --scenario accounts-mixed | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_APPLICABLE; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN |
| RUN-07 | cargo run --release --bin holla -- --scenario remote-host | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN | Reference: NOT_APPLICABLE; Candidate: NOT_RUN | Reference: NOT_RUN; Candidate: NOT_RUN |

This bootstrap was generated from measured facts in [source-facts.json](tools/visibility/source-facts.json). A run receipt is a record of execution results tied to an exact source pair and required test set. No validated paired run receipt is available, so this report cannot show a product pass. Product requirements remain in [CHECKLIST.md](CHECKLIST.md) and [checklist.json](checklist.json).

## Measured source identity

| Identity | Value |
| --- | --- |
| Local role | Candidate (termrock-implementation) |
| Candidate observed commit | [5572f09f57053adb74c886cdab748650452531b6](https://github.com/tailrocks/terminal-components-claude/commit/5572f09f57053adb74c886cdab748650452531b6) |
| Reference observed commit | [91aba5102f5b4dabaff506894f8b19166afb27a6](https://github.com/tailrocks/terminal-components-claude/commit/91aba5102f5b4dabaff506894f8b19166afb27a6) |
| Immutable visual tag object | 1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5 |
| Immutable visual tag commit | [4a79c0a2d40fca46fc406b77157ce3b3f12ec16b](https://github.com/tailrocks/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b) |
| Facts observed at | 2026-10-08T03:27:36Z |
| Report commit | Set after this report is committed; the generated page cannot include its own commit ID. |
| Shared suite / case-set / expected-generation digests | NOT_RECORDED |

## Current CI and repository checks

| Check | Observation | Scope |
| --- | --- | --- |
| [CI run 37721476033](https://github.com/tailrocks/terminal-components-claude/actions/runs/37721476033) | failure at 5572f09f57053adb74c886cdab748650452531b6; 0 jobs; 0 artifacts. Failure reason: Workflow file exceeds the maximum allowed size of 500 KB.; [source](https://github.com/tailrocks/terminal-components-claude/actions/runs/37721476033). | The run head matches the measured candidate SHA. This workflow result is not a product test result. |
| [DCO check 113129904697](https://github.com/tailrocks/terminal-components-claude/commit/5572f09f57053adb74c886cdab748650452531b6/checks) | action_required; 2 commits are reported with sign-off problems. | PR gate status; separate from product results. Existing history is unchanged. |

Available in this environment: Python 3.9.6, Rust 1.98.1, and cargo-nextest 0.9.146 through mise on aarch64-apple-darwin. This records installed tools only; it does not show that a product check ran.

## Applications

Visual, interaction, API, and ownership remain separate. API and ownership are candidate-only obligations.

| Application | Reference visual | Candidate visual | Reference interaction | Candidate interaction | Reference API | Candidate API | Reference ownership | Candidate ownership | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Showcase | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_APPLICABLE | NOT_RUN | NOT_APPLICABLE | NOT_RUN | No current run receipt. |
| Jackin Preview | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_APPLICABLE | NOT_RUN | NOT_APPLICABLE | NOT_RUN | No current run receipt. |
| Holla | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_APPLICABLE | NOT_RUN | NOT_APPLICABLE | NOT_RUN | No current run receipt. |
| TablePro | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_APPLICABLE | NOT_RUN | NOT_APPLICABLE | NOT_RUN | No current run receipt. |

## Component and coverage inventory

The required shared-case and per-component checkpoint sets are Unknown until the case inventory is audited and bound to the common suite. component-ownership.json is a proposal map, not an executed checkpoint set. No per-component result is inferred from imports, checklist text, or historical captures.

No compatible previous paired run is recorded, so trend changes are not measured.

## Next work

Highest open priority: **P0**, from accepted queue revision 6.

| Work | State | Owner | Reviewer | Priority reason |
| --- | --- | --- | --- | --- |
| VIS-01 | review | /root/status_luna | /root/technical_review | Publish source facts and show missing paired execution and the current CI failure. |
| VIS-02 | in_progress | /root/suite_luna | /root/technical_review | A shared driver for real binaries is needed before visibility can be measured. |
| VIS-03 | review | /root/showcase_owner | /root/technical_review | Direct agents to the visibility tasks and accepted work paths before further product refactoring. |
| VIS-04 | in_progress | /root/deferred_execution | /root/technical_review | Run the deferred assertions and preserve their failure results. |
| VIS-05 | in_progress | /root/coordination_luna | /root/technical_review | Queue safety is needed for independent work. Its code does not depend on status publication. |

See [WORK_QUEUE.md](WORK_QUEUE.md) for dependencies and claim details.

## Evidence navigation

- [Work queue](WORK_QUEUE.md)
- [Implementation PR #17](https://github.com/tailrocks/terminal-components-claude/pull/17)
- [Current CI run](https://github.com/tailrocks/terminal-components-claude/actions/runs/37721476033)
- [Product checklist](CHECKLIST.md)
- [Source facts and observation commands](tools/visibility/README.md)

If no current run receipt is present, the product result remains NOT_RUN. NOT_RUN is a result status, not a run-receipt status. No historical pass is promoted to current evidence.
