# Termrock status

| Conclusion | State | Evidence |
| --- | --- | --- |
| Visibility / Complete | NOT_RUN | The shared suite, paired results, and publication checks have not been accepted as a complete current run. |
| Refactor / Ready | NOT_RUN | No complete current evidence covers required visual, interaction, API, ownership, and review checks. |
| Reference / Qualified | NOT_RUN | The tag identity is recorded, but no paired reference execution is qualified. |
| Command / Ready | NOT_RUN | No exact root command has current execution evidence. |
| Evidence freshness | NOT_RUN | No validated paired run receipt is available for the measured source pair. |

## Required commands

The table reports each metric separately for the reference and candidate. The Ready column is separate from Build, Launch, First frame, Interaction, Exit, Restoration, Visual, and Ownership.
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

The status reporter generated this report from measured facts in [source-facts.json](tools/visibility/source-facts.json). A run receipt records execution results tied to an exact source pair and required test set. No validated paired run receipt is available, so this report cannot show a product pass. Product requirements remain in [CHECKLIST.md](CHECKLIST.md) and [checklist.json](checklist.json).

## Recorded source pair

This is the earlier source pair captured at the timestamp below. The branch tips observed later are listed separately; neither table is product-test evidence.

| Identity | Value |
| --- | --- |
| Local role | Candidate (termrock-implementation) |
| Candidate observed commit (recorded pair) | [d7c03139d0131157f1bd40eaf379d46c2e3814f8](https://github.com/tailrocks/terminal-components-claude/commit/d7c03139d0131157f1bd40eaf379d46c2e3814f8) |
| Reference observed commit (recorded pair) | [2ae351dd27f0ee7d7d52ff6093208b0a57f62698](https://github.com/tailrocks/terminal-components-claude/commit/2ae351dd27f0ee7d7d52ff6093208b0a57f62698) |
| Immutable visual tag object | 1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5 |
| Immutable visual tag commit | [4a79c0a2d40fca46fc406b77157ce3b3f12ec16b](https://github.com/tailrocks/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b) |
| Pair observed at | UNKNOWN |
| Report commit | Set after this report is committed; the generated page cannot include its own commit ID. |
| Shared suite / case-set / expected-generation digests | NOT_RECORDED |

## Latest source and local checkout observations

| Observation | Value |
| --- | --- |
| Candidate remote branch tip | [d7c03139d0131157f1bd40eaf379d46c2e3814f8](https://github.com/tailrocks/terminal-components-claude/commit/d7c03139d0131157f1bd40eaf379d46c2e3814f8) |
| Reference remote branch tip | [2ae351dd27f0ee7d7d52ff6093208b0a57f62698](https://github.com/tailrocks/terminal-components-claude/commit/2ae351dd27f0ee7d7d52ff6093208b0a57f62698) |
| Source observation method | read-only GitHub branch API observation |
| Remote branch tips observed at | 2026-10-10T22:26:30Z |
| Source timestamp meaning | 2026-10-10T22:26:30Z; branch last-update time unknown; remote commit committer dates are metadata only (candidate 2026-10-10T22:15:34Z, reference 2026-10-10T22:24:41Z). |
| Earlier source observations | No earlier candidate tip recorded; its retrieval time was not retained, and this tip is superseded by the observation above. |
| Captured local checkout (source facts) | termrock-implementation at `d7c03139d0131157f1bd40eaf379d46c2e3814f8`; compare this recorded identity with the selected source commit above. |
| Captured local cryptographic commit signature | `N` from `git show -s --format='%G?' HEAD` (Git reported no cryptographic signature). |
| Captured local Developer Certificate of Origin (DCO) trailer | Expected line `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>` is present in commit `d7c03139d0131157f1bd40eaf379d46c2e3814f8`. |
| Parsed local DCO trailers | Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com> |
| Captured local commit subject | docs(visibility): authorize current reference suite tip |
| Captured local commit observation time | 2026-10-10T22:26:30Z |
| Local publication | Published. Local HEAD and tree were read from the existing checkout; the working tree is dirty, so this is a commit identity observation and does not qualify dirty product files. |

The DCO observations above describe the captured local commit message. They do not report a remote DCO check.

The two latest branch tips are separate observations, not a paired execution. No paired product run is inferred from them.

## Previous source observations

These source snapshots remain historical. Their CI and Developer Certificate of Origin (DCO) captures are bound to the exact pair in each row. They do not qualify product results or change the fixed comparison pair.

| Candidate / reference source pair | Source observed at | Method | CI and DCO observation (source evidence) | CI capture times | Next source observation at |
| --- | --- | --- | --- | --- | --- |
| `931bbba00bad43de79731912f48e29fe88751cb8` / `68d98ac99238c8580464f238a65b0f862ecc3dbd` | 2026-10-08T08:18:00Z | read-only GitHub branch API observation | run 37747564884 failure (0 jobs, 0 artifacts); reference query returned 0 runs; candidate/reference DCO 113212666161 / 113195062894; product execution NOT_RUN (evidence SHA-256 `4aac6e1d76aa2f7db40588f34f443ee6f0bd23aed11064cb7fc5bb48d183f3d1`) | API 2026-10-08T08:33:53Z; provider page 2026-10-08T09:02:19Z | 2026-10-08T10:05:21Z |
| `bebb60a7948f9ee190483c48545d6bc35c5437df` / `68d98ac99238c8580464f238a65b0f862ecc3dbd` | 2026-10-08T10:05:21Z | read-only exact-source comparison | run 37758149777 failure (0 jobs, 0 artifacts); reference query returned 0 runs; candidate/reference DCO 113247788492 / 113195062894; product execution NOT_RUN (evidence SHA-256 `318e4d127122b4eac303c21d6e48e0028121cbc6e69998483e9a454a34d04e2f`) | API 2026-10-08T10:04:14Z; provider page 2026-10-08T10:03:39Z | 2026-10-08T10:13:57Z |
| `e22d54708fe58670d92c2dac7c1ace6c97fac164` / `68d98ac99238c8580464f238a65b0f862ecc3dbd` | 2026-10-08T10:13:57Z | read-only GitHub branch API observation | No CI/DCO capture recorded; source-only observation. (evidence SHA-256 `ba6862438a415fcccc04a756be15a06e63b9eee17c4e0edf7442bf6fe982f301`) | No CI capture | 2026-10-08T10:19:24Z |
| `cee7e2e307514e49a70d8b8fcb028923ccdc7322` / `68d98ac99238c8580464f238a65b0f862ecc3dbd` | 2026-10-08T10:19:24Z | explicit HTTPS refs/heads fetch | run 37762481719 failure (0 jobs, 0 artifacts); reference query returned 0 runs; candidate/reference DCO 113262062732 / 113195062894; product execution NOT_RUN (evidence SHA-256 `6645a0d40d4777d8fc8af0f74e64bf2c701986ed9bf2d5d30aa1d73a82e1db3c`) | API 2026-10-08T10:22:44Z; provider page 2026-10-08T10:22:44Z | 2026-10-08T15:11:33Z |
| `1ea1c17707f0a8f1639af506be179013d5e2d52a` / `b682cb26d68b353aeeccf9e51653eddf097b39f5` | 2026-10-08T15:11:33Z | explicit HTTPS refs/heads fetch | run 37793991551 failure (0 jobs, 0 artifacts); reference query returned 0 runs; candidate/reference DCO 113368579050 / 113326850483; product execution NOT_RUN (evidence SHA-256 `367121e3eb643453d46d33b598388752c41b1c5fa23e9c099e751c280e0568aa`) | API 2026-10-08T15:22:16Z; provider page 2026-10-08T15:22:16Z | 2026-10-08T23:25:02Z |
| `1d797d41c8141fcbdc3f69d7f11eb8875ab54712` / `b274dd57f4dd078ade6e424d546d83efbd2e8526` | 2026-10-08T23:25:02Z | read-only GitHub branch API observation; local fetched refs verified afterward | No CI/DCO capture recorded; source-only observation. (evidence SHA-256 `fcf943687ae5c398e58ff67a57eba112ea23310fb6cbe1e08738a559ad3eab3d`) | No CI capture | 2026-10-10T22:26:30Z |

Historical raw-evidence coverage gap: normalized source/CI summaries for 6 retained records remain, but their original raw source and CI capture files are not in the repository archive. The gap does not change the fixed source pair or the current readiness assessment; current candidate measurements and paired-result gaps determine readiness separately.


## Recorded CI and repository checks

| Check | Observation | Scope |
| --- | --- | --- |
| [CI run 38090751977](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977) | failure at d7c03139d0131157f1bd40eaf379d46c2e3814f8; 51 jobs; 49 artifacts. Failure reason is not recorded. | The run head matches the latest observed candidate branch tip. This recorded workflow result is historical and is not a product test result. |
| [DCO check 114326526502](https://github.com/tailrocks/terminal-components-claude/commit/d7c03139d0131157f1bd40eaf379d46c2e3814f8/checks) | success; No commits are reported with sign-off problems. | The check head matches the latest observed candidate branch tip. This is a repository gate, separate from product results; no remote history change is inferred. |

## Current CI and Developer Certificate of Origin (DCO) observations

| Check | Observation | Scope |
| --- | --- | --- |
| Candidate Actions run [38090751977](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977) | failure at `d7c03139d0131157f1bd40eaf379d46c2e3814f8`; 51 jobs; 49 artifacts. No correlated provider annotation is recorded, so the failure cause is not stated. | Workflow-level result; product execution remains NOT_RUN. |
| Candidate job execution snapshot | At `2026-10-10T22:54:12Z`: 51 jobs; 51 completed, 0 in progress, 0 queued; 37 successful, 13 failed, and 1 skipped. Failed jobs: [Rust / jackin-preview-app](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114327636875); [Rust / jackin-preview](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114327636971); [Rust / termrock-conformance](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114327637296); [Rust / termrock-e2e](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114327637331); [Rust / termrock](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114327637337); [Rust / tablepro](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114327637341); [Rust / termrock-visibility-publisher](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114327637345); [Rust / tablepro-ui](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114327637350); [Rust / jackin-preview-host-ui](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114327637389); [Rust / termrock-xtask](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114327637393); [Rust / termrock-visibility-tests](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114327637423); [Rust / termrock-viewport](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114327637504); [Required](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114329714696). Skipped jobs: [Publish baseline](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977/job/114329813325). | Final repository workflow job snapshot; these job results do not qualify product checks and product readiness remains NOT_RUN. |
| Reference Actions query | The reference Actions API query completed successfully and returned 0 matching Actions runs for this source tip. | Query source SHA `2ae351dd27f0ee7d7d52ff6093208b0a57f62698`; this is not a paired execution. |
| Candidate DCO status | [success](https://github.com/tailrocks/terminal-components-claude/runs/114326526502); No commits are reported with sign-off problems. | Repository gate at `d7c03139d0131157f1bd40eaf379d46c2e3814f8`; separate from product results. |
| Reference DCO status | [action_required](https://github.com/tailrocks/terminal-components-claude/runs/114328215608); 14 commits are reported with sign-off problems. | Repository gate at `2ae351dd27f0ee7d7d52ff6093208b0a57f62698`; separate from product results. |

GitHub API capture at 2026-10-10T22:54:12Z: manifest SHA-256 `cf5e6ae37a17bdbc81e25e04bc52e942d200eea1c309e89e628c150902a8cadb`. Its source pair is candidate `d7c03139d0131157f1bd40eaf379d46c2e3814f8` and reference `2ae351dd27f0ee7d7d52ff6093208b0a57f62698`. The candidate workflow source is `.github/workflows/ci.yml` at `d7c03139d0131157f1bd40eaf379d46c2e3814f8`, 115446 bytes, SHA-256 `664fe57510323c3d148cdad3f9a416bf383fa54276fa40efb4af95bd3b57804f`.

Workflow execution is repository-level evidence and supplies no additional product results. Paired, visual, interaction, API, ownership, Ready, and Visibility Complete results remain NOT_RUN.


## Current E2E subtree parity

| Observation | Value | Scope |
| --- | --- | --- |
| `crates/termrock-e2e` subtree identity | Candidate `140d74e6552b7cec6db30dd4c32c3e5cbc675aef`; reference `140d74e6552b7cec6db30dd4c32c3e5cbc675aef`; **equal** | Structural `git rev-parse` comparison of candidate `d7c03139d0131157f1bd40eaf379d46c2e3814f8` and reference `2ae351dd27f0ee7d7d52ff6093208b0a57f62698` at `2026-10-10T22:26:30Z`. It is not a paired execution and does not establish visual, interaction, API, ownership, or product readiness. |






## Report publication observations

The pinned root publication record reports commit `b9e34b13ef47401865f1511b9babaab6e020eedc` as a normal fast-forward from `169380c7d6a4cff9f1c43ecef592abc715e6df6a` (tree `3d7c68dcf6724799c38aefb774ac25447ea630b3`) with 3 queue/test/documentation paths. Its postcommit review matched the local commit tree and publication record; the reviewer could not independently refresh the remote branch tip. This administrative publication is separate from product execution.

Publication record `docs/implementation/visibility/evidence/reports/status-source-archive-20261009/raw/termrock-vis10-foundation-publication-20261009.json` (SHA-256 `6dbaf2e84e7befa9e6eda15f6a4885f706e657d33496bd4f1aa4d36dc64912c2`); postcommit review `docs/implementation/visibility/evidence/reports/status-source-archive-20261009/raw/termrock-vis10-foundation-postcommit-review-technical.json` (SHA-256 `5591df8eeeb9250f7cc50135d5cdb0ed11d3b569e7f6f591b0fcac63e1a0edd5`).

| Publication commit | Actions run | Result | Jobs | Artifacts | Failure cause | Product execution |
| --- | --- | --- | ---: | ---: | --- | --- |
| `169380c7d6a4cff9f1c43ecef592abc715e6df6a` | [37862074205](https://github.com/tailrocks/terminal-components-claude/actions/runs/37862074205) | completed / failure (created 2026-10-08T23:55:22Z) | 0 | UNKNOWN_NOT_QUERIED | UNKNOWN_NOT_CAPTURED | NOT_RUN |
| `b9e34b13ef47401865f1511b9babaab6e020eedc` | [37864720297](https://github.com/tailrocks/terminal-components-claude/actions/runs/37864720297) | completed / failure (created 2026-10-09T00:25:40Z) | 0 | UNKNOWN_NOT_QUERIED | UNKNOWN_NOT_CAPTURED | NOT_RUN |

These workflow runs belong to their exact report publication commits. Neither run provides product checks on the selected 1d/b274 source pair; no failure cause or artifact count is inferred.


Tool versions recorded at 2026-10-10T22:54:12Z: Python 3.14.8, Rust 1.98.1, and cargo-nextest 0.9.143 through mise on aarch64-apple-darwin. This records installed tools only; it does not show that a product check ran.

## Applications

Visual, interaction, API, and ownership remain separate. API and ownership are candidate-only obligations.

| Application | Reference visual | Candidate visual | Reference interaction | Candidate interaction | Reference API | Candidate API | Reference ownership | Candidate ownership | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Showcase | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_APPLICABLE | NOT_RUN | NOT_APPLICABLE | NOT_RUN | No current run receipt. |
| Jackin Preview | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_APPLICABLE | NOT_RUN | NOT_APPLICABLE | NOT_RUN | No current run receipt. |
| Holla | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_APPLICABLE | NOT_RUN | NOT_APPLICABLE | NOT_RUN | No current run receipt. |
| TablePro | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_APPLICABLE | NOT_RUN | NOT_APPLICABLE | NOT_RUN | No current run receipt. |

## Historical bounded execution observations

These local temporary records are bound only to historical source pair `1ea1c17707f0a8f1639af506be179013d5e2d52a` / `b682cb26d68b353aeeccf9e51653eddf097b39f5`. That pair was observed at 2026-10-08T15:11:33Z and superseded at 2026-10-08T23:25:02Z. These results are not current for the selected source pair. They do not establish the complete required set, all applications, exact root commands, API, ownership, or readiness. Their raw evidence remains outside this repository, so this report is local-only until those artifacts are archived.

| Observation | Result | Evidence |
| --- | --- | --- |
| Deferred conformance subset | 22 passed; 1 failed of 23 terminal tests; build completed; execution exited 100 after an assertion failure | Seal `c517a00e2b66d252c18c2c5c7737caff9d6832a4b4764b0dd38e3ed12515c8aa`; review `ca856719c5096aaedae96dbdb223817763d14a517f222788a200ac3da366e270` |
| Holla HELP-HOLLA-004 diagnostic | 80 checks: 56 PASS, 8 BLOCKED, 16 NOT_APPLICABLE | Receipt `8fa7329ffad3130f03f1c3025bc7e47807c927e66bff61b3c45b52f778619882`; seal `c2f75d9387a639d7afd05dd4c269f88ef760ab063ca1b6a2c3b635470938ac89`; review `ffff027f91e309778aaf8c0b1e261ff8f23a4f88a5291ec33d84d8b46f50ddc3` |

Per-role outcomes:

| Role | Measured outcomes |
| --- | --- |
| Candidate | build: PASS (4), launch: PASS (4), first_frame: PASS (4), interaction: PASS (16), visual: BLOCKED (4), exit: NOT_APPLICABLE (4), restoration: NOT_APPLICABLE (4) |
| Reference | build: PASS (4), launch: PASS (4), first_frame: PASS (4), interaction: PASS (16), visual: BLOCKED (4), exit: NOT_APPLICABLE (4), restoration: NOT_APPLICABLE (4) |

Aggregate status counts:

| Scope | Count |
| --- | --- |
| Holla results | 80 rows: 56 PASS, 8 BLOCKED, 16 NOT_APPLICABLE |

Aggregate dimension counts:

| Status | Dimensions |
| --- | --- |
| PASS | build_hash_checks: 8; first_frame: 8; interaction: 32; launch_checks: 8 |
| BLOCKED | visual: 8 |
| NOT_APPLICABLE | exit: 8; restoration: 8 |

The deferred failure is BD-21 / W13-05 (w13_filter_wide_trail_cells_clear).
Holla recorded 80 total result rows: 56 PASS, 8 BLOCKED, 16 NOT_APPLICABLE.
The harness selected 1 test and exited 100: 0 passed, 1 failed, and 1 skipped.
The test failed at tests/holla_help_overlay.rs:24-37 because receipt_has_blocking_result found eight BLOCKED visual checks. The raw panic states exact comparison remains blocked because no shared expected generation was supplied. The PTY interaction assertions passed; this is not evidence of a visual FAIL or PASS.
Recorded FAIL/BLOCKED rows: candidate: visual: BLOCKED (4); reference: visual: BLOCKED (4).
The review records 12 registry assertion IDs and 4 aggregate interaction checks per subject across 4 checkpoints.
This covers one 120x40 truecolor case only.
The independent artifact review matched the hash and size of all 80 expected artifact files.
The candidate and reference release-build commands each exited with code 0.
No API or ownership result is recorded.

This partial case does not cover the 293-case, 421-checkpoint, or 7,550-snapshot/profile inventory. The proposed R5 denominator is not accepted as the active required set.


## Current local package control

| Observation | Result | Scope |
| --- | --- | --- |
| Package `termrock-e2e-2026-10-09.2`; 29 files | SHA-256 `9f7e2298aded253dcfccfb6b56f8377be889aadfe477dfd40cb1620267a2ddd6` | Current source commit `d7c03139d0131157f1bd40eaf379d46c2e3814f8`; revision `termrock-e2e-2026-10-09.2`; package files only. |
| `cargo nextest` digest control | PASS (1/1 selected; 1 passed, 0 failed, 0 skipped) | Run `0266dbb7-5fbc-4c83-8021-f5c9af2b65ca`; compiled/current digest sensitivity only; NOT product execution and NOT a paired run. |

Package source identity and one local Rust control passed. This is not a paired run and the repository remains globally dirty; product behavior and readiness are NOT_RUN.


## Local package and reporter controls

| Observation | Result | Evidence |
| --- | --- | --- |
| Historical local E2E package | `f61abd3dfba1b4f867a539ab18ed7e4760b24a18`; tree `15b0c9c2b24b773844a99c84b3839b4e98ef101f`; 21 files; revision `termrock-e2e-2026-10-08.1`; package SHA-256 `f307376a1e1d3dca04642875fdf98e9b43cdee47e6988ed0952c30c62c447361` | Package review `5c45cf6060a154099d5b97dea31066b211ceea3ee985ae80e66c73ba26b4acca` |
| f307 contract control | 1 non-product registry/precondition test passed of 1 selected; no Holla journey ran | Receipt `11bbc2548897ff83126aa6585266e21dd657ce7556dc4195362489d64f81e4a8`; independent review `d371e97c7ecb48770b6a04a1f77fc3279085b2115b3f26ca4d324d2873782041` |
| Holla suite relation | Historical Holla receipt uses `a914f8e34280f55d6a868bfee8777f771fb0354e96e5a2291a91b86c65a62ea0`; it differs from local f307 `f307376a1e1d3dca04642875fdf98e9b43cdee47e6988ed0952c30c62c447361`. The f307 Holla product result remains NOT_RUN. | Holla receipt `8fa7329ffad3130f03f1c3025bc7e47807c927e66bff61b3c45b52f778619882` |
| Status reporter controls | 35 unique test identities reconciled as 26 + 7 + 1 + 1 across R4/R6/R7/R9 source revisions; this is not one full green run. | Cumulative review `65cde1ebac37588ce31983c78bc589242ab7cd3f8be3f9c787537212a9f64d7a` |

These package and reporter observations are local controls. A historical package remains bound to its recorded commit even when the local HEAD has advanced. They do not qualify product behavior or change Visibility / Complete, Refactor / Ready, or current paired product results.


## Component and coverage inventory

The required shared-case and per-component checkpoint sets are unknown until the case inventory is audited and bound to the common suite. component-ownership.json is a proposal map, not an executed checkpoint set. No per-component result is inferred from imports, checklist text, or historical captures.

No compatible previous complete required-set run is recorded, so trend changes are not measured.

## Next work

Highest open priority: **P0**, from accepted queue revision 77.

| Work | State | Owner | Reviewer | Priority reason |
| --- | --- | --- | --- | --- |
| VIS-01 | claimed | /root/reference_claim_luna | /root/ci_parity_review_luna | Publish source facts and show missing paired execution and the current CI failure. |
| VIS-02 | claimed | /root/controls_wrapper_review_luna | /root/subjects_runner_review_luna | A shared driver for real binaries is needed before visibility can be measured. |
| VIS-03 | review | /root/showcase_owner | /root/technical_review | Direct agents to the visibility tasks and accepted work paths before further product refactoring. |
| VIS-04 | claimed | /root | /root/jackin_inventory_current_luna | Run the deferred assertions and preserve their failure results. |
| VIS-06 | in_progress | /root/rust_test_execution | /root/subjects_runner_review_luna | Resolve and build actual release binaries from pinned source before paired observations. |
| VIS-07 | in_progress | /root/language_review | /root/technical_review | Keep actual independent review and commit bindings in repository evidence. |
| VIS-08 | in_progress | /root/rust_test_infrastructure | /root/subjects_runner_review_luna | Use Rust tests to preserve and verify the reporting, queue, deferred, and binary-build requirements. |
| VIS-09 | in_progress | /root/rust_test_infrastructure | /root/technical_review | Preserve all queue assertions in Rust tests before further queue changes. |
| VIS-10 | in_progress | /root/coordination_luna | /root/jackin_inventory_current_luna | Implement the repository-aware amendment prerequisite before reference and isolated tool claims. |
| VIS-11 | in_progress | /root/coordination_luna | /root/velnor_current_refs_luna | The user explicitly requested repair of PR #17 CI/CD; preserve required visibility and product gates. |
| VIS-12 | review | /root/coordination_luna | /root/technical_review | The user explicitly authorized limited DCO commit repairs and guarded force push for PR #17. |
| VIS-13 | claimed | /root/tag_capture_boundary_luna | /root/ci_parity_review_luna | Provide the repository-portable, hash-checked current-observation archive needed by VIS-01 status evidence; retain current failure and NOT_RUN states without promoting readiness. |
| VIS-14 | in_progress | /root/jackin_inventory_current_luna | /root/observer_code_review_luna | Provide bounded read-only Rust ingestion that resolves main-CI and observer identities, distinguishes failures/missing or cancelled work, and validates observer artifacts as inert data. This is an independently useful prerequisite to trusted report projection; it does not publish STATUS or write Git refs. |
| VIS-16 | claimed | /root/api_ownership_luna | /root/subjects_runner_review_luna | Add a narrow Rust command-line boundary for the accepted read-only observer-ingestion API, with process-level tests for caller and observer identity validation. Keep CLI source work independent from VIS-14 while requiring its transport source to be merged before build or test execution. |
| VIS-17 | claimed | /root/reference_claim_luna | /root/ci_parity_review_luna | Implement a bounded Rust status publication command with trusted-run validation and optimistic ref update controls. This claim covers code and fake-transport tests only. |
| VIS-18 | claimed | /root/rust_test_infrastructure | /root/subjects_runner_review_luna | Provide the bounded read-only audit needed to assess the 153 unmanifested Cargo registry source package roots observed after the actual VIS-06 R11 Holla build. This audit is a prerequisite input to the VIS-06 cache-provenance decision; it cannot change or qualify the original R11 result. |
| VIS-19 | claimed | /root/tag_capture_boundary_luna | /root/current_branch_comparison_luna | Document and review the shared Tuiscotti observation and cursor correction needed for VIS-02 to use truthful live and replay capture state. This claim grants only the Termrock evidence path below. |

See [WORK_QUEUE.md](WORK_QUEUE.md) for dependencies and claim details.

## Evidence navigation

- [Work queue](WORK_QUEUE.md)
- [Implementation PR #17](https://github.com/tailrocks/terminal-components-claude/pull/17)
- [Recorded CI run](https://github.com/tailrocks/terminal-components-claude/actions/runs/38090751977)
- Product requirements: [CHECKLIST.md](CHECKLIST.md) and [checklist.json](checklist.json)
- [Source facts and observation commands](tools/visibility/README.md)

If no current run receipt is present, the product result remains NOT_RUN. NOT_RUN is a result status, not a run-receipt status. No historical pass is promoted to current evidence.
