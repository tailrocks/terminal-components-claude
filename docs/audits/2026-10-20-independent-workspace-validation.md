# Independent workspace validation, 2026-10-20

## Source and evidence

The independent verifier tested commit `da911656e168b65cb0e4667138cbe0708ebbcd2d`. No product sources were changed during these runs. Machine-readable package selections, exact commands, outcomes, and logs are retained under `/Users/donbeave/Projects/termrock-cli-evidence-20261020/independent-library-gates/`; `independent-validation-summary.json` is the consolidated receipt.

The immutable visual oracle remains source commit `4169783ae883788186d161af09ba9c481bd84e5c`, tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`, and Tuiscotti revision `a1ea27f757d75d07b05499276cd932e9f8e067ad`. Native migrated digest fixtures are not a substitute for comparison with these frozen real CLI applications.

## Results and limits

| Scope | Result | Limit |
| --- | --- | --- |
| Root default members, all targets | 1,479 passed; 615 skipped; clippy and formatting passed | Six default packages only; this is not full-workspace evidence. |
| Nineteen production termrock packages | 1,313 passed; four skipped; package-scoped clippy passed | Exact package list is recorded in `packages.json`; application targets were excluded. |
| All 47 workspace packages, four disjoint no-fail-fast partitions | 3,760 passed; 35 failed; 29,515 ignored or skipped | All 3,795 default-selected tests ran. Ignored visual scenarios remain unverified. |
| Workspace clippy | Failed | Denied `expect_used` in `crates/tablepro-ui/src/app.rs:9284`. No lint suppression was added. |
| Workspace formatting | Passed | Formatting does not establish functional or visual parity. |

The six default members are `termrock`, `showcase`, `jackin-preview`, `holla`, `tablepro`, and `termrock-conformance`. Required root commands passed on this commit, but their default package selection must never be described as a green full workspace.

## Failure investigation

The 35 initial workspace failures are retained rather than hidden by selection or baseline updates. They include the ownership audit, Showcase interaction and presentation contracts, Jackin and Holla visual contracts, Tablepro interaction/performance/visual contracts, and nine CLI startup or terminal-mode checks.

Nine initial CLI failures reported missing executable paths because the frozen harness defaults to workspace `target/debug` while the verifier used an external `CARGO_TARGET_DIR`. An explicit four-application build and `VISUAL_BIN_DIR` rerun at the same source commit made seven of those checks pass. Two startup-input checks still failed: queued pre-start `q` exited Showcase and Tablepro. The identical nine tests passed against the frozen binaries. Shared startup delay and input draining are therefore a demonstrated compatibility gap; they are being repaired in `termrock-session`, not by changing test inputs or selectors.

The Jackin mono label test also failed on clean pre-change commit `3600c0eeb0596ad61a5eef3bcb6ddab5a8020150`. Independent real frozen CLI captures place the complete `FAILED` label at `(103,7)`, while the native assertion probed `(103,8)`, which contains `o`. A separately reviewed correction must retain the other probes and strengthen the complete label/readability check. Product coordinates and frozen results must not be moved to satisfy an incorrect native assertion. Persistent capture provenance is under `../jackin-mono-baseline/` relative to the evidence root above.

Showcase investigation found disconnected live table/list state and fixed footer painting that masks live status. It also found native navigation expectations inconsistent with the frozen Overview/Chrome ordering. These findings require separate baseline-driven repairs or evidence-backed assertion corrections; no digest has been blessed. Tablepro investigation found a completion session that reopens on drawing/focus, while the frozen editor closes accepted completion and distinguishes navigation from explicit editing. Editing-mode Tab indents in the frozen application; changing it into focus traversal would be another regression.

The ownership audit reports unexcepted production historical renderers. Existing exception ratchets are not proof that application behavior belongs to reusable crates. Production captured-frame replay remains prohibited by the migration goal, whether or not a legacy exception admits it.

## Acceptance

No application has complete 1:1 acceptance from these runs. The independent library checks establish a bounded library result. Full native workspace validation is red, the registered frozen CLI suite has not completed, and architecture review remains incomplete. Preserve all failures, ignored-case coverage gaps, and evidence until the corresponding source changes pass the unchanged frozen comparisons.

## Integration formatting checkpoint

A later fetch/rebase brought in two formatting differences in Showcase list and scrolling helpers. Rustfmt changed line wrapping and removed optional function-call trailing commas. The diff was inspected and normalized source was compared with the pre-format contents. These formatting corrections preserve rendering and input behavior. The broader native failures above remain recorded independently.

## Cleared integration checkpoint

The corrected ordinary-check partition is 11 source-only checks and nine real-CLI checks for each binary version. `isolation::env_isolation` runs against both explicit binary directories. The runner guard suite has 18 tests, including a candidate-isolation failure regression. The same tooling change is being applied to both comparison branches.

The default MenuBar and Picker-title style changes remain outside this integration checkpoint. Their real-baseline evidence is retained, but the combined native gate has 15 component-digest failures (seven MenuBar and eight Picker). No frozen approvals were modified and no failing comparisons were weakened. The full deferred source is preserved at local ref `wip/style-conformance-20261020` (29ff641f4598755d6897442a9191b3c031529646). Resolving this native-fixture authority conflict remains work, not accepted parity.

The cleared checkpoint passed 1,479 default-member tests with 615 skipped before its formatting-only follow-up. Independent workspace failures documented above and full paired-CLI coverage remain incomplete. Neither branch is accepted as complete.
