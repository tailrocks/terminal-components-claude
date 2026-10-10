# Visibility status reporter

Source-facts schema version 2 preserves the original source pair and its
CI/DCO observations, then records later source observations and the local
checkout separately. `source_observation_history` retains the six
previous source snapshots in order, including their exact CI captures or an
explicit `null` when no capture exists. CI and DCO checks apply only to their
recorded head SHAs. Source observations, repository gates, and product results
remain separate.

The retained 931 branch API record uses the legacy `updated_at` field. Its
recorded command extracts `.commit.commit.committer.date`, which is commit
metadata rather than the branch last-update time. New source records name this
value `commit_committer_at` and store an unknown branch last-update time as an
explicit `null`.

`status.py` renders the root `STATUS.md` from source facts, accepted task
records in `docs/implementation/visibility/tasks.json`, and the narrowly
supported execution observations described below. It does not execute a
product or infer a result from a checklist. Exact root-command rows stay
`NOT_RUN` unless evidence records that exact command.

## Recorded baseline

The original recorded source pair is candidate
`85b51da2e9832dba642abf7d64d032f848cade0e` and reference
`5f6e52f31861f9f4281f1db264ab012457b9bc2e`. Its observation time is not
separately recorded. CI run
[37736017446](https://github.com/tailrocks/terminal-components-claude/actions/runs/37736017446)
failed before any job or artifact was recorded. Its run-page annotation reports
that the workflow file exceeds the 500 KB limit. DCO check `113175645948`
reported two sign-off problems. Both are historical repository observations,
not product test results. Older README versions listed run `37721476033` and
check `113129904697`; those IDs were superseded before this source-facts
recording.

The visual-baseline branch and immutable tag are recorded separately. The
tag object and peeled commit remain the identities in `source-facts.json`.
The installed Python, Rust, and cargo-nextest versions describe the environment;
they do not show that a product check ran.

## Previous source observations

The source facts retain these previous observations in order. Each CI
capture is bound to the candidate and reference SHAs in its row. The e22
observation is source-only and has an explicit `current_ci_observation: null`.

- Candidate `931bbba00bad43de79731912f48e29fe88751cb8` / reference
  `68d98ac99238c8580464f238a65b0f862ecc3dbd`, observed at
  `2026-10-08T08:18:00Z`; superseded at `2026-10-08T10:05:21Z`. Its API CI
  capture was recorded at `08:33:53Z` (manifest SHA-256
  `96fab9bbb37943d69c27e9e57d6b053ac7674de5bff2ca5f5f3087429e226d50`) and
  provider page capture at `09:02:19Z` (manifest SHA-256
  `f1eb6e6dc17014481e898b750053e29f278fcf39f70738c5d851e73d0e695e80`). Run
  `37747564884` failed before jobs or artifacts. The reference API query
  completed successfully and returned zero matching runs. DCO checks
  `113212666161` and `113195062894` reported two and fourteen sign-off problems.
- Candidate `bebb60a7948f9ee190483c48545d6bc35c5437df` / reference
  `68d98ac99238c8580464f238a65b0f862ecc3dbd`, observed at
  `2026-10-08T10:05:21Z`; superseded at `2026-10-08T10:13:57Z`. Its API capture
  ended at `10:04:14Z` (manifest SHA-256
  `3f419da7b1b4be673f550c68601f10e9ca9605fddb05bce0e4353349062c529f`); its
  provider page was captured at `10:03:39Z`. Run `37758149777` failed with zero
  jobs and zero artifacts. The reference API query completed successfully and
  returned zero matching runs. DCO checks `113247788492` and `113195062894`
  reported two and fourteen sign-off problems.
- Candidate `e22d54708fe58670d92c2dac7c1ace6c97fac164` / reference
  `68d98ac99238c8580464f238a65b0f862ecc3dbd`, observed by the branch API at
  `2026-10-08T10:13:57Z`; superseded at `2026-10-08T10:19:24Z`. The source
  record preserves the returned heads and commit committer dates. The API did
  not supply branch last-update time.
- Candidate `cee7e2e307514e49a70d8b8fcb028923ccdc7322` / reference
  `68d98ac99238c8580464f238a65b0f862ecc3dbd`, selected by explicit refs fetch
  at `2026-10-08T10:19:24Z`; superseded at `2026-10-08T15:11:33Z`. Its fetch
  record is `/private/tmp/termrock-fixed-source-pair-20261008-101924.json`
  (SHA-256 `6645a0d40d4777d8fc8af0f74e64bf2c701986ed9bf2d5d30aa1d73a82e1db3c`).
  Its commit committer dates were `2026-10-08T17:17:11+07:00` and
  `2026-10-08T14:13:13+07:00`; they are metadata, and branch last-update time
  is unknown. The API and provider capture manifest is
  `/private/tmp/termrock-ci-fixed-pair.20261008-102210Z/manifest.json`
  (SHA-256 `819ee505aeceb2d4f55de56e6845765c6f3189dee01be605ed694bdd19f005a1`).
  It records candidate run `37762481719` as failure with zero jobs and zero
  artifacts, a reference query with zero matching runs, and DCO checks
  `113262062732` / `113195062894` with two / fourteen sign-off problems. The
  provider page's workflow-size annotation and 537,470-byte workflow (SHA-256
  `607ed7980b6c18f95cfc08e1f8dee1aac12477b5d1e678f26c84a75b8f238c24`) are
  historical evidence, not product results.
- Candidate `1ea1c17707f0a8f1639af506be179013d5e2d52a` / reference
  `b682cb26d68b353aeeccf9e51653eddf097b39f5`, observed at
  `2026-10-08T15:11:33Z`; superseded at `2026-10-08T23:25:02Z`. Candidate run
  `37793991551` failed with zero jobs and zero artifacts; the reference query
  returned zero runs. DCO checks `113368579050` / `113326850483` reported two
  / fourteen sign-off problems. The provider page recorded the 500 KB
  workflow-size annotation. This capture applies only to this historical pair
  (fetch record SHA-256
  `367121e3eb643453d46d33b598388752c41b1c5fa23e9c099e751c280e0568aa`).
- Candidate `1d797d41c8141fcbdc3f69d7f11eb8875ab54712` / reference
  `b274dd57f4dd078ade6e424d546d83efbd2e8526`, observed at
  `2026-10-08T23:25:02Z`; superseded at `2026-10-10T06:44:23Z`. Its normalized
  source record and the 2026-10-09 current-status/publication records remain
  preserved, while the 1d/b274 CI snapshot is retained separately as a
  historical snapshot. It has no current product qualification.

The 931 record retains the earlier candidate tip
`b07540df4f7a97fcca8c4d3396102651be459503`, which was relayed without a
retained fetch time or API response. It carries no test or product
qualification.

## Current source pair and CI/DCO snapshot

The current comparison pair is candidate
`38a897b179b6f01cf07dd6d889f7a278886e5226` and reference
`4b473a98a8641a9dae8dbf9c31c0c94b15465496`, observed at
`2026-10-10T06:44:23Z` through read-only GitHub branch APIs. The API supplied
commit heads and committer timestamps but no branch-update timestamp. The
candidate commit date is `2026-10-10T06:30:48Z`; the reference commit date is
`2026-10-09T18:51:08Z`. Commit dates are metadata, not branch-update times.

The latest push CI run is
[38031202825](https://github.com/tailrocks/terminal-components-claude/actions/runs/38031202825),
created at `2026-10-10T06:30:56Z`. It is completed/failure with zero jobs and
zero artifacts. Its workflow remains 537,470 bytes with SHA-256
`607ed7980b6c18f95cfc08e1f8dee1aac12477b5d1e678f26c84a75b8f238c24`; the run
page records the 500 KB workflow-size annotation. The raw page SHA-256 is
`f12d83f345f875c2cfc7abd88b4d728ffd13b8d74b5a2b3a054719f684b6e21b`, and the
ephemeral capture-manifest SHA-256 is
`9da3c771c29012a1bb2d1f9de09dbab144958cf51e64361832f307245326b2b8`. Raw API and
page bodies were not archived. The reference Actions query returned zero
matching runs. Candidate DCO check `114152359863` passed; reference DCO check
`113974968877` is action_required with 14 reported commits. These are
repository observations only.

PR #17 is an open draft at the current candidate head with base `main` at
`81a8bf15cd3042f80649e2b48fed479829518dbd`. GitHub reports `mergeable=false`
and `mergeable_state=dirty`. No conflict resolution, merge, rebase, reset, or
readiness inference is included.

The latest local checkout is the published commit above, tree
`3efa46acfcc9612c6e752dc0d5f958fbf57a28ba`, signature status `N`, and the exact
required DCO trailer. The shared worktree is dirty from another workstream.
Those dirty files are outside this measurement and are not qualified. The 766
and f61 local checkout observations remain in local history.

Visibility / Complete, Refactor / Ready, Reference / Qualified, Command /
Ready, Evidence freshness, paired product execution, visual parity, interaction
parity, API adoption, ownership proof, and admission all remain NOT_RUN. Zero
CI jobs and the package-only control below cannot promote any of those states.

## Report publication observations

The root publication record reports commit
`b9e34b13ef47401865f1511b9babaab6e020eedc` as a normal fast-forward from
`169380c7d6a4cff9f1c43ecef592abc715e6df6a` (tree
`3d7c68dcf6724799c38aefb774ac25447ea630b3`) with three queue, test, and
documentation paths. Its postcommit review matched the local commit tree and
publication record, but could not independently refresh the remote branch
tip. This administrative publication is separate from product execution.
Publication record `/private/tmp/termrock-vis10-foundation-publication-20261009.json`
(SHA-256 `6dbaf2e84e7befa9e6eda15f6a4885f706e657d33496bd4f1aa4d36dc64912c2`);
postcommit review `/private/tmp/termrock-vis10-foundation-postcommit-review-technical.json`
(SHA-256 `5591df8eeeb9250f7cc50135d5cdb0ed11d3b569e7f6f591b0fcac63e1a0edd5`).

| Publication commit | Actions run | Result | Jobs | Artifacts | Failure cause | Product execution |
| --- | --- | --- | ---: | ---: | --- | --- |
| `169380c7d6a4cff9f1c43ecef592abc715e6df6a` | [37862074205](https://github.com/tailrocks/terminal-components-claude/actions/runs/37862074205) | completed / failure | 0 | UNKNOWN_NOT_QUERIED | UNKNOWN_NOT_CAPTURED | NOT_RUN |
| `b9e34b13ef47401865f1511b9babaab6e020eedc` | [37864720297](https://github.com/tailrocks/terminal-components-claude/actions/runs/37864720297) | completed / failure | 0 | UNKNOWN_NOT_QUERIED | UNKNOWN_NOT_CAPTURED | NOT_RUN |

The P169 run was created at `2026-10-08T23:55:22Z`; the b9 run was created at
`2026-10-09T00:25:40Z`. Their manifests bind each run to its exact publication
commit: P169 SHA-256
`10ecf34f05dad57d128f273f78cadaebac1089e25d037d688a292d5dced33cd4`, and b9
SHA-256 `591bf6e3de2c7aedbd68d5e59c822a5f3c4b32b5c7b5adcd7b4f159a3ba5104b`.
Neither run provides product checks on the selected 1d/b274 pair; their
artifact counts and failure causes are unknown.

## Historical bounded execution observations

`source-facts.json` includes a provisional
`termrock-status-execution-observations-v1` record nested under the superseded
1ea/b682 source pair. This narrow reader supports
the pinned deferred Nextest seal/case files and one paired Holla diagnostic
receipt with its sealed harness run and independent review records. It checks
raw JSON SHA-256 values, strict JSON, source-pair identity, event-to-case-map
counts, the exact Holla check ID/dimension set, the eight role/checkpoint by ten
format artifact matrix, and the review records' matching pins. It accepts and
renders each supported row status, including `FAIL` and `NOT_RUN`; a failure
row remains visible as a failure. For Holla, the expected and actual executable
SHA-256 and builder-receipt SHA-256 must agree across the paired receipt, the
journey review, and the separate build review before build hash rows are shown.
The nested builder receipt's source commit must match the role's selected
source, and its target triple must match the paired receipt's build record.
It does not execute a product or treat these partial observations as a complete
required set.

The historical deferred subset recorded 22 passing and 1 failing test of 23;
the failed row is `BD-21` / `W13-05`. The Holla `HELP-HOLLA-004` diagnostic recorded
56 PASS, 8 visual BLOCKED, and 16 exit/restoration NOT_APPLICABLE checks across
candidate and reference. It covers one case at 120×40 with truecolor, not the
full application inventory. The actual Holla harness assertion failed after
its visual checks were BLOCKED; the report preserves that state as BLOCKED
rather than relabeling it FAIL.

The execution records and independent reviews are under `/private/tmp` and are
local-only until their evidence files are archived with the repository. The
reader opens evidence JSON one path component at a time without following
symlinks below `/private/tmp` or the report root. It rehashes the JSON record
files; artifact-file hash/size verification is attributed to the pinned
independent Holla review. API, ownership, the active common
required set, exact root commands, and overall readiness remain incomplete.
The proposed R5 denominator is not treated as an accepted required set.

The fetch and capture records are stored under `/private/tmp` outside this
repository. Their hashes identify the reviewed local inputs, but the files are
provisional until archived and are not CI-portable by those paths. Later branch
observations remain separate and do not silently advance the selected pair.

The current local checkout observation is described in “Current source pair and
CI/DCO snapshot.” The 766 and f61 checkout/DCO observations are retained in
local checkout history. They are distinct from remote CI/DCO checks and do not
qualify the dirty worktree.

## Local package and reporter controls

`source-facts.json` includes a historical local E2E package observation bound
to commit `f61abd3dfba1b4f867a539ab18ed7e4760b24a18` and tree
`15b0c9c2b24b773844a99c84b3839b4e98ef101f`. That record identifies 21 files with package
SHA-256 `f307376a1e1d3dca04642875fdf98e9b43cdee47e6988ed0952c30c62c447361`.
Its gate ran one non-product registry/precondition contract test; it did not run
the Holla journey. The historical Holla receipt uses suite SHA-256
`a914f8e34280f55d6a868bfee8777f771fb0354e96e5a2291a91b86c65a62ea0`, which
differs from f307. The f307 Holla product result remains NOT_RUN.

The current package-only observation binds commit
`38a897b179b6f01cf07dd6d889f7a278886e5226`, 28 files, revision
`termrock-e2e-2026-10-09.2`, and package SHA-256
`5867f043a6b3e2913871770e70ac55553c235617852a646bd3fb525dd8940637`. A focused
`cargo nextest` digest-control run (`e3a31e85-4c4f-4004-ac68-ab8f73819ec0`)
passed 1/1. This checks compiled/current digest sensitivity only. It is not a
product execution, not a paired run, and does not qualify the globally dirty
checkout.

The status reporter control observation reconciles 35 unique test identities
as 26 + 7 + 1 + 1 across the reviewed R4/R6/R7/R9 source revisions and exact
failed-test reruns. Those outcomes are not one full green run and are not
product readiness evidence. The package, contract-gate, and reporter-control
records remain local-only until their evidence files are accepted and archived.

The reporter does not fetch GitHub. It verifies the pinned execution JSON
records when rendering local partial observations; it does not rehash the
artifact files named inside the paired Holla receipt.

## Typed execution attempt history

An optional top-level `execution_attempt_history` object uses
`termrock-status-execution-attempt-history/v1`. The `attempts` array is ordered
by contiguous sequence number and unique attempt ID. Each row binds to a
candidate commit, candidate/reference pair, immutable visual tag, or tool
source, and declares `CURRENT` or `HISTORICAL`. `CURRENT` is accepted only when
the binding matches the latest recorded source observation; an older or
unrelated source must remain `HISTORICAL`.

Every row pins its evidence files by repository-relative path, format, and
SHA-256. Non-provider lanes include a pinned
`termrock-status-attempt-result/v1` JSON record whose attempt ID, lane, source
binding, reconciled counts, and outcome states must equal the row. The
`ci_provider` lane instead validates the captured run, failed check suite,
suite-specific check-runs response, jobs, artifacts, and failed-log CLI output.
When that CLI reports “log not found” without a captured HTTP response, the
record must say `diagnostic.status=NOT_EXPOSED` and `http_status=NOT_CAPTURED`;
the reporter does not infer a provider cause or HTTP code.

Counts reconcile inventory, selection, filtered cases, started tests, terminal
results, and incomplete tests. Test, child process, wrapper, collector,
postflight, cleanup, capture, qualification, and admission outcomes render in
separate columns. Attempt history is informational: it cannot qualify a
product result, admit data, or change readiness. It also does not replace the
existing paired product observations.

## Published report and provider evidence

The current-publication record keeps report commits separate from the fixed
product comparison pair. Candidate report commit 20f2d485 and reference report
commit 28c6940 are pinned with their postcommit reviews. The candidate
revision-59 and reference revision-56 task records are historical snapshots.
Later queue edits are outside this evidence record.

The archived f0a provider run and DCO check apply to candidate source commit
f0a05fa, before those report commits. The run page records the 500 KB workflow
annotation. The retained Root record has no raw Actions run or jobs response, so
the report does not state job or artifact counts. The DCO result links to its
recorded check page.

The paired-source publication records suite SHA-256
b67efe786fb0c0f64db2aca62c572b700f2a5247d7a2258deab1313bdb581a5d and common
package tree c962085b9c9e6b80ddba31d8c82e01c8d2e97179. Paired execution remains
NOT_RUN and the corpus remains NOT_ADMITTED. The product comparison pair stays
1d797d41c8141fcbdc3f69d7f11eb8875ab54712 /
b274dd57f4dd078ade6e424d546d83efbd2e8526. Both the package identity and those
report records are historical; neither qualifies current product behavior.

The source evidence archive contains 70 raw members and 6,171,844 bytes. Its
manifest pins each file by repository-relative path, byte count, and SHA-256.
For --role reference, the report links checklist and source-instructions files
at the immutable candidate report commit because those files are absent from
the reference tree.

## Generate and check

Run the report CLI from the repository root with Python 3.9 or later:

    python3 tools/visibility/status.py --write
    python3 tools/visibility/status.py --check

Run the black-box CLI contract tests with Rust nextest:

    mise exec -- cargo nextest run --manifest-path crates/termrock-visibility-tests/Cargo.toml --test status --locked

The `--role reference` option changes the local-role presentation and uses
immutable candidate links for files that are absent from the reference tree.
Each branch reads its checked-in task record. The archived revision-59 and
revision-56 records remain historical snapshots.
The default role is `candidate`; `--write` writes the role selected for the
current checkout.

Ready has its own result column. It is separate from build, launch, first frame,
input and interaction, exit, restoration, visual, and ownership results. The
exact root-command rows remain `NOT_RUN`; the Holla case-specific rows display
their recorded partial statuses separately.

`--check` is read-only and requires byte-for-byte agreement. The reporter has no
publication credentials, does not update task records or the work queue, and
writes only `STATUS.md` when called with `--write`.

## Current limits

The reporter does not validate an accepted complete case registry or active
required set, enforce monotonic receipt freshness across product runs, calculate
trends, produce component checkpoint pages, or publish reports. Those results
remain incomplete until their records and evidence are accepted.
Current CI had zero runner jobs, so no CI product result exists. The current
package digest control is local and unpaired. All product readiness conclusions
remain NOT_RUN.
