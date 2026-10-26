# T37 metadata result

## Final E2E follow-up on source `953fbd76ac47019b0161d86a9a462d04f51b0c7f`

The one-line `crates/termrock-e2e/build.rs` formatting unit (`d0f7387ee44898f5587b28fdfbc70d1ef9c0313b`) received independent Sol-high ACCEPT from `/root/t37_apps_review`; it preserves tokens and literals. On the integrated source, the guarded locked E2E all-target/all-feature check and standalone formatter check both exited 0. Exact commands and log hashes are recorded in `final-verification-953fbd/final-verification.json`.

The retained `Profile` warning for required `readiness_timeout_ms` and `input_pacing_ms` remains visible. `build.rs` hashes itself as one of 22 E2E suite inputs, so its source-format change refreshes the compiled suite digest. Applying the unchanged algorithm gives `f38701e25e4d70121eb3934cd4426771809bc48e8c1f50f7c62fdc84df6575d9` before the formatting commit and `6c24418e8060ca31056576dd7db377f27fccc31f18738f0802b4ba90250b12e8` at the current source; these values are not asserted equal. The earlier formatter failure remains preserved as historical evidence.

This follow-up extends evidence commit `243487b3dd05816d90b818d299f576d893db62f7`; no tests ran, selected behavior cases remain zero, and product behavior remains **NOT VERIFIED**. Overall T37 documentation and integrated acceptance are still pending.

## Final follow-up on source `560f3dca487247b1068c45a0d2b469c8eabe3165`

This section supersedes the earlier pending-source statements below. On this accepted combined source, all five guarded locked build checks passed: the root workspace and the e2e, visibility, external-consumer, and publisher standalone workspaces. Root and three standalone format checks passed. The e2e standalone format check exited 1 on a one-line formatting difference in `crates/termrock-e2e/build.rs:23`; the file SHA-256 (`d0c867ee75175e6464cd260fd658544c97926fb2c0dc0e69d82af3a4c4ceaa57`) matches product baseline `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`, so the drift predates T37. A formatting-only fix and e2e rerun are assigned to the source owner and remain pending. No Rust source was changed by this metadata owner.

The permitted guarded no-deps metadata command passed in the root and all four standalone workspaces. It reports 47 root packages and 68 retained targets (42 libraries, 14 binaries, 12 examples), with no enabled test, bench, or doctest targets. The five lock hashes remained unchanged during metadata inspection. The exact 62-path metadata graph still matches SHA-256 `9a39580c733b65fe02ac28a691188838daef4957235bd658fc305d1d5bdaf2a5`. Eight scoped active-reference checks are clear; the 24-file preservation hash set matches.

Pinned Velnor 0.1.48 generation ran twice successfully and was byte-identical. The final manifest has 360 tasks (90 each build, Clippy, doc, and fmt), zero Nextest run tasks, and 45 discovered packages. Testing config remains `cargo_nextest`, `run_ignored = "default"`, with default/testing configurations and `--no-tests fail` retained for eligible future tasks. The Nextest lane is **ABSENT / NOT VERIFIED**, not passing. Both `rust-visual-harness` jobs remain generated. Generated output hashes and exact command logs are in `final-verification-560f/final-verification.json`.

The scoped metadata/evidence commit `0d47da019d0ace072558bdce9916824eb67c9180` received Sol-high scoped ACCEPT with no findings from `/root/t37_apps_review`; that review did not rerun Cargo or Velnor. The follow-up checks above independently bind the current source. The two corrected caller-classification strings in `t37-port-conflict-orphan/runbook.json` are authorized by `/root/t00_independent_review` Sol-high under T37-ORPHAN-CAPTURE-01: the historical caller was the free function `crates/holla-sim/src/fixtures.rs::port_conflict_case`.

No tests, doctests, benches, or Nextest cases ran. Selected behavior cases remain zero and product behavior is **NOT VERIFIED**. T37 remains subject to integrated independent acceptance.

## Status

Metadata, lockfile, generated-workflow, and guarded build checks were completed on accepted combined source `55c75d292f5288b07f4a779575a42431259e21c4`. This is structural and build evidence only; zero-case behavior is **NOT VERIFIED**. No tests or Nextest runs were executed.

T37-EXPECTED-CACHE-01 and the exact 202-path Rust formatter correction are independently accepted and integrated. The current source is `865073462331552897822d65ede8d673f9c870d7`. A final source unit removes the unused `PortConflict::capture` helper; its independent review/integration is pending. The generator output and locked build checks below are bound to source `55c75` and predate the accepted source updates. Per coordinator direction, regenerate through pinned Velnor and rerun final formatting and locked build checks once on the accepted combined source after the listener-helper unit is integrated, rather than at intermediate heads. This result is not a T37 self-approval.

## Input and ownership

- Original source input: `c817b99e9e43abbbf3519d56b2f5ee86924243d8`.
- Product source baseline: `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`.
- Accepted documentation baseline: `5bcc79b971ae30ee244b8fef841c6fbafdb918d0`.
- Initial metadata input: `d402f893d159ac26da5db4d18c13510afdbce12f`.
- Source used for the recorded metadata, generator, and locked build checks: `55c75d292f5288b07f4a779575a42431259e21c4`.
- T37-EXPECTED-CACHE-01 source: `ed9d246ae51515e88bce0b0bc2a8545fb32efcd1`; exact 202-path formatter source: `865073462331552897822d65ede8d673f9c870d7`. These source edits do not alter the manifest graph, but the generated task digests and final checks must be refreshed after the pending listener-helper source unit.
- Accepted source inputs include the foundation, component, application, selection-binding, tools, runtime-compatibility, admission-compatibility, and facade-comment fragments recorded in the sibling T37 result records. The final tools source head is `7fee71b6bd5091ce670d36644273ed1ffda1481d`.
- This metadata owner changes Cargo manifests and locks, Velnor/Nextest inputs, generated output through its producer, and `results/t37-metadata/**`. The two exact compatibility-evidence JSON string corrections were separately authorized by the coordinator and are included in this evidence unit.

## Metadata cleanup and retained targets

- Retired workspace packages `termrock-test-support` and `termrock-conformance`, their test targets, dependency edges, and test-only workspace dependency keys. Preserved their immutable performance baseline and conformance component baseline byte-identically.
- Removed only the approved manual API workbench/probe packages and their generated discovery exclusions, the frozen helper package, and its lock. Did not infer retirement for other nested packages.
- Removed test-only development dependencies whose only remaining uses were in deleted suites. The exact manifests and removed dependency names are in `dependency-retirement-map.json`. Retained normal dependencies and development dependencies used by production/examples, including shell example dependencies and the e2e admission binary's `tuiscotti` dependency.
- Kept `visual-harness` and its four production app binaries; removed only its `visual_baseline` test target and test-only dependencies. Kept e2e `admit_expected_generation`; kept visibility `capture-command` and `nextest-result-reader`; removed the obsolete visibility `fixture` target and unused `tempfile` dependency.
- Removed the root inventory runner's exclusive `profiles.json`. Kept `historical.json` byte-identically as historical obligations and `required.json` byte-identically as an unapproved proposal. No inventory runner or test replacement was added.
- Removed the visibility-tools nested Nextest config because it had no retained test target consumer. Root `.config/nextest.toml` retains its active `ci` profile. The external-consumer profile remains and Velnor reports that it shadows the root profile for that standalone consumer.
- Restored `crates/termrock-forms/Cargo.toml`'s development edge `termrock-runtime = { workspace = true, features = ["testing"] }` under T37-BUILD-COMPAT-01. The retained `form_lanes` and `form_intrinsic_content` examples call the restored, testing-gated `Runtime::draw_scene` method.
- Retained target identities, names, paths, crate types, required features, and documentation flags. All 47 workspace packages set `autotests = false` and `autobenches = false`; all 68 retained root targets have effective `test = false` and `bench = false`; all 42 libraries have effective `doctest = false`. The root target kinds are 42 libraries, 14 binaries, and 12 examples. There are zero separate test or bench targets. No `harness = false` was used.
- Four retained standalone workspaces also passed metadata inspection: e2e (1 package/3 targets), visibility tools (1/3), external consumer (1/2), and visibility publisher (1/1). All have `autotests = false` and `autobenches = false`, with zero enabled test/bench/doctest behavior targets.

See `dependency-retirement-map.json`, `retirement-input-hashes.json`, `target-survivor-map.json`, and `metadata-final.json` for exact paths, identity rows, and structural counts.

The metadata-only structural reference scan covers 63 active manifests, locks, configs, generated workflows, and the Velnor task manifest. It finds no retired package, target, or dependency references. Historical/policy evidence is excluded from that scan and its retained references are labeled in `structural-reference-scan.json`.

## Lockfiles

The guard denied `cargo update --workspace --offline` with status 125; no bypass was used. Guard-permitted non-locked Cargo resolution refreshed the pruned graphs, then every final compile check used `--locked`. The unsupported guarded `cargo metadata --locked --format-version 1 --no-deps --manifest-path Cargo.toml` probe was also denied with status 125 before Cargo; it left the root lock unchanged. The permitted guarded no-deps metadata command passed in the root and all four standalone workspaces.

| Lockfile | Entries before → after | Removed entries | Added entries | SHA-256 after |
|---|---:|---:|---:|---|
| `Cargo.lock` | 477 → 387 | 90 | 0 | `b7604fa542eeead232ee5503a6c1871ef74ca5aed4447bd4d782e975fd3c91ba` |
| `crates/termrock-e2e/Cargo.lock` | 264 → 233 | 31 | 0 | `406a25455aa28fb34d35a3b46aef6ef76b7af8ef52472a52c6e2edbc8c3cb3f5` |
| `crates/termrock-visibility-tests/Cargo.lock` | 41 → 30 | 11 | 0 | `34d3f9b1aa926b8209fec670122325720bb548e8f697cc9e1a280b03bdb46de8` |
| `external-consumer-demo/Cargo.lock` | 207 → 207 | 0 | 0 | `3d97fd71235062fb1ae6a833aed92a76d8d918050116fb75ed1b4560766c2d5b` |
| `tools/termrock-visibility-publisher/Cargo.lock` | 175 → 150 | 25 | 0 | `e58bcfca71d427b6eb7604cf1db7cadd0ba3847d7b5b97e8579d47cc6331ba2d` |

Package versions and sources for every retained lock entry are unchanged; lock changes prune dependency closure only. The external-consumer lock's entry count stays constant while its dependency graph changes. `lockfiles-final.json` contains the full removed-package lists and before/after hashes.

## Velnor and generated output

Generation used `mise exec -- velnor-actions generate` twice; both runs exited 0 and were byte-idempotent. Velnor 0.1.48 generated the workflow and task manifest. No generated workflow was hand-edited.

- `.github/workflows/ci.yml` SHA-256: `b2a2747b49052bbec9c565d1436e4d19c6f7aadb6cb3dd3fa18376127ca83107`.
- `.github/velnor/task-execution-manifest-v1.json` SHA-256: `2c72c5bc66b5bed549e4322389ef099ee9d1e749ed4b055e3eb2f45eaf7b8305`.
- The workflow preamble through `jobs:` is byte-identical to the prior generated workflow (SHA-256 `2592a3fb0162fae984f0bff0752e81841a6c1cbf999fac2218a7b318e399e261`), and all 13 unique action/local-action references are unchanged, including the pinned external action SHAs. Only generated task/job membership changes.
- The prior generated manifest had 470 tasks: 94 each for build, Clippy, doc, fmt, and Nextest. The final manifest has 360 tasks: 90 each for build, Clippy, doc, and fmt; it has zero Nextest run tasks. It covers 45 discovered packages; the root has 47 members, with e2e and visibility tools intentionally excluded from discovery and checked as standalone workspaces.
- The final 180 default/testing task pairs include 162 identical argv/environment pairs and 18 feature-flag differences across `termrock-core`, `termrock-runtime`, `termrock-shell`, `termrock-theme`, `termrock-viewport`, and `termrock`. All 180 default-task argv/environment records match the prior generated output.
- `rust-visual-harness` and `rust-visual-harness-testing` remain generated jobs. The Velnor testing matrix retains `default` and `testing`, `test_runner = "cargo_nextest"`, and `run_ignored = "default"`. Previously generated eligible Nextest tasks all carried `--no-tests fail`; the final generated set has no eligible Nextest task, so the lane is **ABSENT / NOT VERIFIED**, not green. The policy remains in the generator input for future approved targets.
- `.velnor/config.toml` changes only its discovery exclusions for retired paths; the feature matrix and runner settings are unchanged. `mise.toml`, `mise.lock`, and `rust-toolchain.toml` are byte-identical to the fixed input.

Exact task differences and input/output hashes are in `generator-final.json`. The root Nextest `ci` profile is unchanged; visual-only profiles and their stale test-lane comments were removed with the retired visual suite. No replacement gate was added.

## Preservation and comments

- All twelve e2e schemas, `cases/registry.json`, and `profile.json` match the fixed input byte-for-byte. The oracle descriptor, historical/pending inventory JSON, policy/ownership maps, and the immutable performance/component baseline files also match their fixed-input hashes. See `final-preservation-checks.json` and `preservation-hashes.json`.
- `cases/deferred-obligations.json` was source-retired under the accepted E2E-01/tools closure (`697a0353e5c84485b5dc36aa8058a4c71a1c236e`, `8bd63d96ab6be07f14b570e434df3d376fc8020b`). Its input hash remains recorded as provenance. The removed rows do not close the listed behavior obligations; zero-case behavior stays **NOT VERIFIED**.
- `profile.json` keeps `readiness_timeout_ms = 8000` and `input_pacing_ms = 120`. T37-PROFILE-FORMAT-01 retains both as required `u64` fields under `deny_unknown_fields` when admission parses `PROFILE_JSON` (`crates/termrock-e2e/src/lib.rs:799-800`, parse at line 3345). They validate retained profile format/admission data; they make no executor or timing guarantee. Their compiler warning is retained without fake reads, serde changes, or lint allowances.
- Root `[workspace.lints.rust]` and `[workspace.lints.clippy]` tables, including explanatory safety comments, are byte-identical to the fixed input. No blanket lint allowance or weakened unsafe-code lint was added.
- Stale manifest comments tied to the removed doc-inventory test, public-only test, and retired testing package were removed. Other retained manifest/API comments were kept. Generated file headers and pinned workflow action references remain producer-owned.

## Checks and open findings

All Cargo commands used `/Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo ...`. Disk preflight reported `free_bytes=2136941903872 status=ok` before final metadata inspection.

`metadata-output-hashes.json` binds the manifest/lock/config/generated-output graph used for these checks (`metadata_graph_sha256 = 9a39580c733b65fe02ac28a691188838daef4957235bd658fc305d1d5bdaf2a5`). `metadata-final.json` records the five permitted no-deps metadata commands and target flags. `build-final.json` records exact final command, source, metadata graph, exit status, and log hash for every locked check.

| Check | Command | Workdir | Status | Log |
|---|---|---|---:|---|
| Root compile | `cargo check --locked --workspace --all-targets --all-features` | `.` | 0 | `build-root-locked-final.log` |
| e2e compile | `cargo check --locked --all-targets --all-features` | `crates/termrock-e2e` | 0 | `build-e2e-locked-final.log` |
| visibility tools compile | same | `crates/termrock-visibility-tests` | 0 | `build-visibility-locked-final.log` |
| consumer compile | same | `external-consumer-demo` | 0 | `build-consumer-locked-final.log` |
| publisher compile | same | `tools/termrock-visibility-publisher` | 0 | `build-publisher-locked-final.log` |
| whitespace check | `git diff --check` | `.` | 0 | clean |
| formatting | `cargo fmt --all -- --check` | `.` | 1 | `format-check-final.log` |

The root locked check compiled the 47-member workspace and all targets/features successfully on source `55c75`. The e2e and root checks emitted warnings. The `Profile` warning is intentionally retained by T37-PROFILE-FORMAT-01. T37-EXPECTED-CACHE-01 independently accepted and integrated dropping only the four private `LoadedExpectedCheckpoint` cached fields/initializers while preserving all file/hash/frame/provenance/PNG/decode/dimension/alpha/tree validation, errors, and order; the final locked e2e check is pending. The old root log reports `PortConflict::capture` as unused at `crates/holla-sim/src/listeners.rs:397`; a source-only removal unit is pending independent review and integration.

The pre-correction formatting check on source `55c75` affected 202 Rust paths, with 1,792 diff blocks, 3,940 blank-line edits, and 66 nonblank edits in ten paths. It was not blank-line-only. The exact path list is `format-check-affected-paths.txt` (SHA-256 `ab5517c9ec853ff081d4ec465048a7588e53ebb6a211a61bda8892b760f25a7c`); full pre-correction output is preserved as `format-check-final.log.gz`. Its decompressed SHA-256 is `b48a9855e94b3ac1528495875df6a78a217fb9b6e1507d3257c8547dc8ecb97d` and compressed SHA-256 is `5de1c0d57a35f157a9e595162f3d791f45eb438c9741ffd6b9034150fb2fd909`. `set -o pipefail; gzip -dc docs/implementation/pr38-final/results/t37-metadata/format-check-final.log.gz | shasum -a 256` exited 0 and returned the original SHA. The exact formatter correction is now accepted and integrated; the final formatter check is pending the listener-helper source unit. This metadata owner made no `.rs` formatting changes.

No tests, doctests, benches, or Nextest runs were executed. There are zero selected behavior cases; all product behavior remains **NOT VERIFIED**. Structural and build evidence does not prove product behavior. T05/T32 and later reviewed test-plan work remain open. Independent Sol-high review of this metadata/evidence commit is required.
