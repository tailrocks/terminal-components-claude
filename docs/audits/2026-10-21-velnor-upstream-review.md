# Velnor upstream investigation — 2026-10-21

Read-only investigation for Termrock CI blockers. No repository files, refs, PRs, or dependencies were changed.

## Revision and release identities

- Velnor `main`: `a6b1d7873151dc62711844158490e8687d191943` (commit subject `fix(release): allow bounded product compiler completion (#247)`; source package version 0.1.30).
- Latest stable GitHub release: `v0.1.29`, published `2026-10-20T11:52:07Z`, target/source `e7b2f088715379c22be473fab896a43d50cc16fc`. No `v0.1.30` tag/release found. The `runner-*` GitHub releases are source-bound runner artifacts, not a stable Velnor Actions generator release.
- `docs/upstream/README.md` is absent in the Termrock source checkout, so its requested PR loop procedure cannot be followed from repository instructions.

## Relevant source contracts on Velnor main

- `crates/velnor-actions-contract/src/config/workflow.rs:30-70`: `WorkflowConfig` has `crate_timeout_minutes`, `checkout_lfs`, task/verification options, but no Plan timeout or checkout depth/filter policy. Struct rejects unknown fields (`deny_unknown_fields`). The docs at lines 42-50 explicitly say plan/required/validator controls retain built-in defaults.
- `crates/velnor-actions-contract/src/workflow/timeout.rs:27-40`: `JobTimeout::PLAN = DEFAULT`, and default is five minutes. `JobTimeout::CRATE` is separately overrideable.
- `crates/velnor-actions-orchestrator/src/wire_w1.rs:106-121`: planner checkout is intentionally full history (`fetch-depth: 0`); comments cite merge-commit parent validation and `base...head` affected-work diff requirements, with shallow checkout previously failing closed (`checkout_head_mismatch`, CI 36749240499).
- `crates/velnor-actions-orchestrator/src/workflow_jobs.rs:312` uses `checkout_step_full`; `crates/velnor-actions-orchestrator/src/check_jobs.rs:48` also uses it for checks.
- `crates/velnor-actions-rust/src/tasks.rs:18-44, 173-210, 351-355`: doctest is an independent task kind and is derived for package groups based on Cargo target metadata.
- `crates/velnor-actions-rust/src/argv.rs:140-143, 225-230`: doctests deliberately remain separate and emit `cargo test --locked --offline --manifest-path … --package … --doc` even when the package's normal runner is Cargo Nextest.
- Velnor main has no config mechanism to disable or convert doctest jobs; no `doctest`/Plan timeout/checkout policy fields were found. The current generator contract preserves doctests independently.

## Official upstream PR inventory

Open PRs at observation time were #240 (Gradle stack), #235 (service containers), and #167 (private Scale Set runner); none concern Plan timeout/checkout or doctests. Remote branch names matching `plan|doctest|checkout|timeout` were absent. Relevant merged history:

- [PR #247](https://github.com/tailrocks/velnor-new/pull/247), merged at main `a6b1d787`: increases only three official product compiler jobs and `velnor-host` compiler job to ten minutes. It does not alter Plan timeout or consumer config. Tests require exact four exceptions.
- [PR #175](https://github.com/tailrocks/velnor-new/pull/175): configurable `crate_timeout_minutes` up to 360; control jobs explicitly retain five minutes.
- [PR #181](https://github.com/tailrocks/velnor-new/pull/181): one job per crate with parallel check steps; doctests are still a separate check.
- [PR #173](https://github.com/tailrocks/velnor-new/pull/173): earlier five-minute Rust capacity phases, superseded by #181.
- [PR #231](https://github.com/tailrocks/velnor-new/pull/231): optional Git LFS checkout; not a filtered-history/Plan checkout optimization.

## Doctest decision boundary

The pinned/current Nextest architecture cannot run Rust doctests. Official Nextest running docs say: “Doctests are currently not supported because of limitations in stable Rust,” and direct users to a separate `cargo test --doc` step: https://github.com/nextest-rs/nextest/blob/main/site/src/docs/running.md (also https://nexte.st/docs/running/).

Therefore no Velnor config-only change can both retain live doctest execution and obey Termrock's `cargo nextest`-only hard rule. A generator `doctest=false` switch would silently omit obligations and is not acceptable by itself. Smallest policy-compliant repo route: inventory all doctest examples, port each to executable examples/integration tests run under nextest (retain compile/documentation validation), then set Cargo target `doctest = false` only for packages whose doctest inventory has been fully migrated; let Velnor's existing Cargo metadata logic omit the now-inapplicable doctest task. Require a red/green inventory test ensuring no doc snippets were dropped. Do not relax policy or disable doctests before that work.

## Plan timeout and checkout proposal

The current five-minute timeout is a fixed control-job default and cannot be changed by `crate_timeout_minutes`; adding `workflow.plan_timeout_minutes` is the smallest backwards-compatible upstream feature. Proposed contract: optional/default-five `u16` with the existing `1..=360` validation, explicit serde compatibility and docs; propagate from `WorkflowConfig` through plan-job construction into `Job.timeout_minutes`; include tests for omitted/default output preserving bytes, boundary validation, custom Plan job literal, and unchanged timeouts for other controls. At the Termrock consumer, set a measured explicit budget (initial proposal 15 minutes) only after release/pin availability, regenerate with Velnor, and observe an uninterrupted exact PR run. Existing evidence proves only that five minutes is insufficient; it does not establish that fifteen is sufficient.

This timeout alone buys bounded execution time; it does not reduce the observed expensive full-history/full-tree checkout. A distinct checkout-optimization project is needed if 15 minutes is insufficient. Do not reduce depth blindly: the upstream code documents required merge/base/head ancestry. Safest proposal is instrument stage time, then design/test a narrowly fetched ref set or partial/sparse checkout that preserves exact `base...head`, merge-parent validation, package/tool input discovery, and reference-resource availability. Any checkout mode must compare plan JSON and affected-work selections against full checkout on both simple and divergent PR histories, including deleted/renamed files and forks. The current checkout step does not expose such a setting, and this investigation found no upstream PR implementing it.

## External evidence

Termrock's already-committed external diagnosis is `docs/audits/ci-plan-checkout-diagnosis-20261021.json` (SHA-256 `cbaab4f103d8ee67ab500a41acd4c9ebc4c765c0fc7dd21bc429a0d5aa6b9ba4`). It records explicit run `39468344965`, Plan job `118786018355`, timed out during Checkout at the five-minute limit, and separate runs cancelled by concurrency. Public-consumer plan is in `docs/audits/ci-public-consumer-plan-f076.json` (SHA-256 `b54fd5de9939425096d41e146e715abd716c09a2dd96c16263374ff50fa0aa63`).
