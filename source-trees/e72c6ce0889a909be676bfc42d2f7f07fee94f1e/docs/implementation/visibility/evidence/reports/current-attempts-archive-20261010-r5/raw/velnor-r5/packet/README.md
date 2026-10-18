# Velnor prerelease v2: exact 48-case gate (R5)

Status: prepared for focused independent review; **NOT RUN**. R4 is preserved unchanged after static review found that its PASS-row collector deduplicated identical selected IDs. R5 preserves each matching raw PASS row so a duplicate cannot satisfy the exact selected-ID/count check. R3’s failed preflight remains preserved below. Only Root may execute the frozen runner after review and an exclusive Cargo slot.

## Preserved R3 preflight

R3 preflight exited 2 before spawning Cargo because `source.root_mode` was omitted. The R8 source directory is physically mode 0755; the R8 full-tree JSON does not carry a `root_mode` field. R4 records that observed mode explicitly. R3 stdout/stderr remain pinned in `plan.json`; no tests or Cargo ran.

## Source and selection

The source is the Git-less admitted R8 snapshot at `/private/tmp/termrock-vis11-tool-fixture-admission-r8-source-tag-snapshot-luna-20261010/source`. Its full tree pin is `646f52a778ec53f1edd6c23b3906443cee7aa052e066b2f01cf17905c3a1f3e2` (3,457 entries: 2,608 files, 842 directories, 7 symlinks); the post-metadata file inventory pin is `2f9d0616c41c06e2cd7986551aa934dec26c6e3a0d559f12decffac735a2d62b` (2,615 non-directories). The R8 fixture-admission record is `2ac63643ad9f7a5a5b948f2bfc2081967138bbbe5039d8f8a37afab95a70ad12`. No Git commit, tag, release, or remote identity is asserted.

The six existing groups remain [2, 2, 23, 11, 6, 4], with 48 unique test IDs. Relative to the R1 source map, 16 test rows were rebound to current source hashes across three files; six function line numbers were refreshed in `impl_ci_observer.rs`. No test IDs were added, removed, or excluded. The complete tree, source map, and relevant source rows are verified by the runner before and after each child.

## Build inputs

The R9 target at `/private/tmp/termrock-vis11-tool-fixture-gate-r9-source-tag-fixture-luna-20261010/run-root/target` is reused as a Cargo build input only. R9 ran one selected case against the exact same R8 source root, full tree, and lockfile; the prior 1/1 PASS is not carried forward. The pinned dependency file, fingerprint, R9 receipts, ready record, output log, and independent one-test review bind that provenance. The target root’s observed mode is `755`.

The Cargo registry archive and source-index closure retain their prior pins and remain **NOT_QUALIFIED** under their review. Offline use does not promote them to qualified cache evidence.

## Historical zero-job case

The prior R9 21-case observer run recorded 20 PASS / 1 FAIL for `impl_ci_observer::rendered_runtime_reports_zero_jobs_and_api_errors_without_success_or_missing_claims`. A separate R10 one-case diagnostic retry recorded 1/1 PASS against a different source tree. The cause of the earlier failure is not inferred. The case remains in this 48-case selection and will be executed normally; neither historical result is counted or treated as an expected-failure exception.

## Invocation and limits

Runner: `/private/tmp/termrock-vis11-prerelease-v2-48case-gate-luna-r5-r8-20261010/run-root/runner.cjs` (SHA-256 `0096bffe42a95984693c035cd00e562d2bfe346eac0b28841742caeddd92a083`). Root invocation:

`/opt/homebrew/Cellar/node/26.11.0_1/bin/node /private/tmp/termrock-vis11-prerelease-v2-48case-gate-luna-r5-r8-20261010/run-root/runner.cjs --execute`

Each of six bounded groups uses Cargo Nextest with `--locked --offline`, `--build-jobs 2`, `--test-threads 1`, `--no-tests fail`, `--user-config-file none`, `--ignore-default-filter`, `--retries 0`, and `--no-fail-fast`. The whole gate is limited to 1,200 seconds, with 15 seconds for TERM, 10 seconds for post-KILL close, and a 4 MiB combined output cap. Each group has a separate raw log and receipt. The only execution output root is `/private/tmp/termrock-vis11-prerelease-v2-48case-gate-luna-r5-r8-20261010/run-root/execution`, currently absent.

The no-Cargo Node preflight PASSed with exit 0. Its stdout is pinned at `/private/tmp/termrock-vis11-prerelease-v2-48case-gate-luna-r5-r8-20261010/evidence/preflight.stdout` (SHA-256 `76ee106f332f43afe9ab74f27b2700116fbdffb8e5bdf3c57ff10f3a62e763fb`); stderr at `/private/tmp/termrock-vis11-prerelease-v2-48case-gate-luna-r5-r8-20261010/evidence/preflight.stderr` is empty (SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`). It checked source, input/cache/tool pins, target provenance inputs, output absence, and all 48 command selections. Preparation ran no Cargo, Nextest, build, test, or Velnor CLI. This packet does not establish full workspace, tool, release, tag, or product qualification.
