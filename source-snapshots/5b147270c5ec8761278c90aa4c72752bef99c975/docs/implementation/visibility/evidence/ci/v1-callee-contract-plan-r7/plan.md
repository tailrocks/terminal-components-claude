# VIS-11 external implementation addition: V1 callee contract — R7

Status: proposal pending final independent review and Root queue admission. This is an addition to the existing VIS-11 external-implementation plan path. Root records it through the existing queue CLI/evidence mechanism. It creates no new work ID, queue field, claim framework, or owner reassignment.

## Fixed source subject

Use only the selected Git-less R3 source snapshot at `/private/tmp/termrock-vis11-velnor-observer-tag-reusable-source-r3-fixture-fix-20261009/source`. Its inventory is `/private/tmp/termrock-vis11-velnor-observer-tag-reusable-source-r3-fixture-fix-20261009/evidence/combined-r3-tree-inventory.json`, SHA-256 `612bbbcf127a64ee4e44d709aea92dbf3d688140a944bdb69e54a72153065319`. No Git commit identity is asserted. The snapshot is not claimed as executed or qualified. The provenance-only source-fix review file has SHA-256 `57149da728b12ce060faa0794e5049fbc2b7c40e4756aa587037206bf0091671` and records `SOURCE_CANDIDATE_REVIEW_PENDING`; it is not a qualification receipt.

This slice follows `V1_CALLEE_CONTRACT` from `/private/tmp/termrock-vis11-reusable-publisher-split-r2-20261009/plan.json` (SHA-256 `d038e674baa6d69c22b05ad22a836183c262bb3605a0353ac44b0ac5c34e7b3f`). It does not include V2 caller or observer workflow generation.

## Owner, reviewer, and admission

- External implementation owner: `/root/subjects_runner_review_luna`.
- Independent reviewer: `/root/velnor_current_refs_luna`.
- Root remains the integration owner and alone records admission.
- No producer-source edit begins until Root records this exact reviewed plan addition in the accepted queue state.
- The existing VIS-11 owner, reviewer, token, base, allowed paths, and history remain unchanged.

## Exact eight paths

1. `crates/core/velnor-actions-contract-workflow/src/workflow/reusable_callee.rs` — new.
2. `crates/core/velnor-actions-contract-workflow/src/workflow/mod.rs`.
3. `crates/core/velnor-actions-contract-workflow/tests/impl_contract_reusable_callee.rs` — new.
4. `crates/core/velnor-actions-contract-workflow/tests/main.rs`.
5. `crates/services/velnor-actions-workflow-document/src/document.rs`.
6. `crates/services/velnor-actions-workflow-document/src/document_reusable_callee.rs` — new.
7. `crates/services/velnor-actions-workflow-render-strict/src/lib.rs`.
8. `crates/services/velnor-actions-workflow-render-strict/src/reusable_callee.rs` — new.

Keep substantive contract, document, and renderer code in the three new modules. Existing files should only add the declarations, exports, and test registration needed to connect them. Baseline existing-file sizes are 94, 17, 361, and 105 lines, respectively; `document.rs` has 39 lines of remaining room under the general cap. Apply the frozen source `AGENTS.md` limits too: 150 lines for the existing `lib.rs` and test `main.rs`, and 80 lines per function. Keep all files under the general 400-line limit.

## Exact input contract

Expose exactly these six `workflow_call` inputs, all with type `string`, `required: true`, no `default`, and these direct expression mappings:

| Input | Type | Required | Default | Fixed expression |
| --- | --- | --- | --- | --- |
| `repository` | string | true | absent | `github.repository` |
| `event_name` | string | true | absent | `github.event_name` |
| `ref` | string | true | absent | `github.ref` |
| `sha` | string | true | absent | `github.sha` |
| `run_id` | string | true | absent | `github.run_id` |
| `run_attempt` | string | true | absent | `github.run_attempt` |

The official GitHub Actions contexts reference lists `github.run_id` and `github.run_attempt` as strings. GitHub workflow syntax requires values passed in `jobs.<job_id>.with` to match the called workflow's declared input type. The direct string mappings above preserve that requirement. Set `required: true` for all six inputs and emit no `default` property; GitHub otherwise treats them as optional, and an omitted string input defaults to the empty string. The [contexts reference](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts) and [workflow syntax reference](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax) are the primary-source basis.

All six fields forward data only. The identity guard checks actual GitHub context and trusted renderer policy; it never treats caller-supplied input values as identity attestation or overrides.

## Contract, identity, and permissions

Implement only the strict typed reusable-callee contract, document mapping, strict rendering, and Rust contract/render tests. Preserve exactly six required string inputs with no defaults, their direct mappings, the closed schema, and permissions. Trusted caller repository, workflow path, and exact source/publication branch come from explicit typed renderer policy. Callers cannot choose or override them. If a required policy value is absent or ambiguous, generated output remains disabled.

The read-only V1 guard is push-only. It checks caller `github.repository`, `github.event_name == push`, `github.ref`, and `github.workflow_ref` against explicit trusted renderer policy. It checks dynamic equality `github.workflow_sha == github.sha`; it embeds no expected caller commit SHA and adds no workflow input. The guard separately checks callee `job.workflow_repository` against the expected callee repository.

Use these exact permissions:

- Reusable caller job: `actions: read`, `contents: write`, `pull-requests: none`, `id-token: none`.
- Read-only identity guard job: `{}` (no token scopes).
- Callee write-capable job: `actions: read`, `contents: write`, `pull-requests: none`, `id-token: none`.
- Every omitted permission is denied. Do not add scopes.

The guard reads GitHub context only. It cannot fetch or verify caller workflow source bytes. V1 does not claim branch protection or workflow-file ownership controls are installed or verified. V2/T3 must separately verify actual branch/ruleset and workflow-file trust controls and exact reviewed caller workflow bytes through externally trusted configuration before an operating publisher is enabled. Do not assume those controls exist.

V1 must not compare callee `job.workflow_sha` to a value embedded in the callee. After V1 is committed, V2 binds callee source through the trusted caller workflow's exact full-SHA `uses` reference. V2/T3 separately verify exact caller workflow bytes before enablement. This contract is scoped to GitHub.com; it makes no GHES support claim because the `job.workflow_*` identity properties used here are not available on GHES.

## Required V1 Rust regressions

- Positive contract and rendered-schema cases cover exactly the six fields, each `type: string`, `required: true`, and with no `default` property, plus the direct expression mappings above and the exact reusable-caller permission map.
- Negative contract cases reject each omitted input, malformed/unknown fields, non-string values, any input not marked `required: true`, any emitted default, altered permissions, caller-selected repository/branch/workflow path/callee reference, and using the six data inputs to override identity policy.
- Strict rendering contains `workflow_call` only, exactly the six string inputs with `required: true`, no default, fixed mappings and permissions, and no candidate checkout, extraction, or execution steps. The render snapshot asserts `required` is exactly true and `default` is absent for each input.
- Identity guard positive case accepts only the expected caller repository, push event, exact ref/workflow_ref, `github.workflow_sha == github.sha`, and expected callee `job.workflow_repository`.
- Negative cases reject non-push events, missing/mismatched caller repository/ref/workflow_ref, `github.workflow_sha != github.sha`, missing/mismatched callee repository, absent/ambiguous policy, and caller overrides.
- Permission tests prove the guard has an empty permission map and the write job has exactly the stated read/write map; omitted scopes are denied. Reject any additional caller/callee scope, missing required writer scope, or token scope on the guard.
- Structure tests prove the write-capable job depends on guard success and that the guard is read-only. Missing/ambiguous explicit repository/path/branch policy keeps generated output disabled.
- V1 tests do not claim branch/ruleset or workflow-file controls are installed. V2/T3 own their verification, exact caller-byte verification, and full-SHA callee `uses` binding after V1 commit.

## R6 correction recorded in R7

The R6 plan and review bound six required fields by intent but did not require `required: true` in the rendered `workflow_call` schema. GitHub workflow syntax makes inputs optional unless `required: true`; an omitted optional string input without a default is `""`. R7 makes the rendered schema and Rust render snapshot assert `required: true` and no `default` for each input, and includes negative cases for omitted inputs and altered required/default fields. See the [workflow syntax reference](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax).

## Follow-on trusted Rust publisher requirement

The trusted Rust publisher is a separate later slice, outside these eight paths. It receives `run_id` and `run_attempt` as strings, validates each as a positive decimal string, then parses with checked `u64` conversion before any API, artifact, or Git write side effect. It rejects empty, zero, negative, fractional, nondecimal, whitespace-padded, and overflowing values. It must not use `fromJSON`, JavaScript number conversion, or other precision-losing coercion. Its later Rust tests must accept canonical positive decimal values within `u64`, reject every invalid form before side effects, and reject values above `u64::MAX` without wrapping.

## Exclusions and gates

This V1 slice excludes V2 caller/workflow generation, observer identity or ingestion, publisher runtime, report projection, runtime-schema changes, queue/CAS changes, Termrock edits, and all remote/tag/release actions. Do not add observer IDs to the V1 contract.

After queue admission, source edits remain limited to these paths and require exact independent source review. Cargo/Nextest/build/generation remain `NOT_RUN` until Root grants the bounded Rust gate. This proposal grants no test execution and makes no source-qualification claim.
