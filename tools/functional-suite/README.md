# Common functional suite

This standalone Cargo workspace runs real-process journeys for Showcase, Holla, Jackin Preview, and TablePro. It owns one case manifest, lockfile, `functional` Nextest target, and pinned patched engine sources. It does not build the application binaries or update expected samples.

`cases/cases.toml` registers 89 cases, 10 with plan availability and 79 without plans, plus 276 coverage IDs. Coverage IDs describe requirements; they are not frame checkpoints. The current expected store has 35 reference samples: the original 30 plus five Status & Hints samples. The first-delivery selection still covers four journeys and 30 checkpoints; that subset does not establish full project acceptance.

## Export and prepare

Run the exporter from the application checkout. Give it a new absolute destination outside both application worktrees; it excludes the engine forks from the source export. The engine helper then creates and verifies the pinned full source trees and applies their recorded patches.

```sh
APP_ROOT="$PWD"
EXPORT="/absolute/path/to/new-functional-suite-export"
OTHER_APP_ROOT="/absolute/path/to/other-application-worktree"
python3 tools/functional-suite/scripts/export-source.py "$EXPORT" \
  --forbid-root "$APP_ROOT" \
  --forbid-root "$OTHER_APP_ROOT"
cd "$EXPORT"
python3 scripts/prepare-engines.py
```

Build `showcase`, `holla`, `jackin-preview`, and `tablepro` from the same application source and profile. Keep the binaries and each run's output directory outside the application worktrees.

## Run the first-delivery subset

From the exported suite root, use a new output directory for each run. Keep the source export, lockfile, prepared engines, binaries, and profile fixed across paired runs.

```sh
BIN_DIR="/absolute/path/to/the-four-production-binaries"
OUT_DIR="/absolute/path/to/new-output-A"
CARGO_BUILD_JOBS=2 TERMROCK_PARITY_BIN_DIR="$BIN_DIR" \
TERMROCK_PARITY_OUT_DIR="$OUT_DIR" \
"$HOME/.local/bin/rust-build-guard" -- mise exec -- cargo nextest run \
  --locked --offline --test functional --no-tests fail --retries 0 \
  --no-fail-fast --test-threads 2 -E 'test(/^first_delivery_/)'
```

Use a different fresh `OUT_DIR` for the paired run. This exact filter selects four tests and leaves 85 registered cases unrun. A nonzero exit keeps the run failed; review its saved case and frame receipts before drawing a result. Do not copy target output into the expected store as approval.

## Results at 2026-10-28

The original four-case first-delivery reference run passed 30/30 exact state-and-pixel comparisons. All four selected processes exited cleanly, drained output, and restored the terminal. Its receipt is `97c6d1d58526adce64f487d7e3db5e27e1b000e37abbd75092ac690e40880845`; the independently accepted checks establish those 30 samples only.

The expected store now contains 35 samples: those original 30 plus five Status & Hints samples. The prior 30 hashes were preserved. The five additional samples passed the bounded reference check (5/5); receipt `96a0826447d3e0f812274ecaeac585320142069e5b9bb6e86c37ed8eab290bc5`, independent review `07840e9d1d6e4ac0d6c306c6d29979fa10c8f55b36b4b5f6192b5f84e89376b`. The current approval record is `expected/BASELINE-APPROVAL.json` (SHA-256 `d2472258c6f9f8ece6ee5a63795f9f303470c750bf8f2c553cdaace66607d41b`); its additive setup block retains the earlier `reference-check-pending` label despite the separate accepted check. The 35-sample inventory digest is `cdd7ec53ae25a1bf2685d69e187bbaf5b26159145f93be544792e4a186ba49d4`. The persistent empty approval lock is runtime-only and excluded from Git and source exports.

The current Showcase pair covers two journeys and 12 checkpoints. The reference side matched 12/12. The target reached all 12 checkpoints and exited cleanly, drained output, and restored the terminal, but matched 0/12 exact state-and-pixel comparisons. All 24 header-row captures matched; full routes and visual parity remain open. Receipt `0b824f4225063f2b9d66439aa9ff433661e257c1783b3444e8cbdd262289bcd0` and correction addendum `a9324139b9de43995e470b2f0786642ad5b44d075db3d3b464142afa3197a4d9` bind the run; independent review `8117092a28481fcea12346ed29d3df286b115a1d7cefa26cdfb8d819929ea5dd` accepted the bounded evidence. This does not establish full Showcase parity.

An earlier four-application target run reached 16/30 checkpoints: two matched, 14 differed visually, and 14 were not reached. Those results remain bound to the earlier binaries and are not replaced by the two-journey Showcase result. The registered suite still has 79 cases without plans; 276 coverage IDs are not the frame denominator, and full-suite parity is not established.

The PR #38 Actions run `40631500020` at previous head `6510719682d1020b5e634ca4d8589b732de11544` completed with failure. The Plan job checked out synthetic merge commit `4acf6dd779522f46c6f3834561049352c3ca4e19`, merging the head into base `a97e351df736b576904bc450619ca347ea64d65b`. The 50-job run had one Jactionlint success, three failures (Plan and two Required aggregates), and 46 skipped jobs. Plan failed before application Rust jobs on unresolved path edge `tools/functional-suite/forks/tui-test-upstream/crates/tui-test`; the aggregate failures add no application-level result. A prior DCO report of 38 historical missing sign-offs is not a fresh DCO readback for this head.

The [PR #38 evidence entry for head 1e3ab9e](../../docs/architecture/termrock-migration-record.md#current-evidence-at-pr-38-head-1e3ab9e-2026-10-28) is historical; the current entry at head 2b6f8fa is linked below.

At published PR #38 head `1e3ab9e502cf93d8f941179436d9bdd7dc865df4`, Actions run [40643651467](https://github.com/tailrocks/terminal-components-claude/actions/runs/40643651467) was in progress at the recorded 2026-10-28 06:26:17 UTC observation. Jactionlint had passed; Plan was still running and its checkout/result had not been inspected. No application result is known for this head. The migration record contains the source-bound Jackin pair and the latest CI observation.


## Current evidence at published PR #38 head `2b6f8fa` (2026-10-28)

The published suite remains at 89 cases, 10 with plan availability and 79 without. Its 276 coverage IDs are requirements, not frame checkpoints. The approved store remains 35 samples, and the full frame denominator is incomplete. The runtime approval lock remains excluded from Git and source exports.

The published Showcase Buttons V5 change has a bounded two-case result: reference 12/12 exact frames; target 12/12 reached and 0/12 exact state-and-pixel matches. Both child processes exited cleanly, drained output, and restored terminal modes. The direct target Nextest exit is unknown because the zsh wrapper failed on its reserved `status` variable. This is incremental evidence, not full Showcase parity. Review SHA-256: `f7c017a26e7f95be9b7a82a46f0e1cb4589c4af64baa5f8758c02177cdcda522`.

An unpublished Suite V2 candidate adds one case plan while retaining 89 cases: 11 with plan availability, 78 without; its 276 IDs remain requirements. Candidate hashes: harness `5c57284d9cf726c924852a1908cfa1b7de39234e28cc3c4eea848aec235ee1fe`, case manifest `9a445c17cee018708b287427b98d7a289e68cd5bd35b2f0d3d37c56ded7c2f0b`. Export and engine preparation succeeded; guarded locked/offline metadata succeeded. The functional target build was interrupted before completion when the observed total reached seven actual compiler workers. No Nextest LIST or test run completed. The candidate is not published or validated.

Actions run [40652202431](https://github.com/tailrocks/terminal-components-claude/actions/runs/40652202431) at this head failed before application jobs: Plan stopped at unresolved path edge `tools/functional-suite/forks/tui-test-upstream/crates/tui-test`; the run had 1 success, 3 failures (Plan plus two Required aggregates), and 46 skipped jobs. The aggregate failures are not application results. See the [current migration record entry](../../docs/architecture/termrock-migration-record.md#current-evidence-at-pr-38-head-2b6f8fa-2026-10-28) for receipts and source bindings.

Velnor PR #426 merged at `2026-10-28T07:31:47Z` (`e5286b78ef031018ecfe0920eeba34973103915e`). A separate read-only release-binding check remains in progress; merged source does not prove release binding, hosted artifact behavior, or combined native-and-RustSuite execution.

## Current correction at PR #38 head `2b6f8fa` — 2026-10-28

The Suite V2 candidate remains unpublished and unvalidated. Its export, engine preparation, and locked offline metadata succeeded, but both guarded functional-target builds were interrupted for resource admission (exit 130). One attempt observed seven compiler workers; the resumed attempt observed three, including two workers on `/tmp/velnor-bump-target`, and interrupted only its own process. No successful compile, Nextest LIST, or test run exists. The partial target and both full receipts/log sets are preserved under `/private/tmp/termrock-functional-suite-phases-v2-evidence-20261028/`; details and hashes are in the [migration record](../../docs/architecture/termrock-migration-record.md#correction-to-suite-v2-and-velnor-release-status-at-pr-38-head-2b6f8fa--2026-10-28).

Velnor PR #426 merged, but the latest published release `v0.1.66` points to the pre-merge base and was published before the merge. No approved exact release pin contains the feature. Hosted artifact behavior, combined native-and-RustSuite execution, and release binding remain unproven. The merge is not release acceptance.
