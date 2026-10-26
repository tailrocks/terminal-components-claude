# T37 configuration comment correction

This correction starts from commit `383ad59b44baf95232005551b4d70f136bef3fea`, tree `402f9324abc73332978a0d03ee1c80f6b8ffa477`. The tracked configuration diff SHA-256 is `84d2084e46811d527c23dbef273c2b937597223487668966f5b1c474038dad2c`.

## Changes

The six approved config files lost 34 ordinary comment spans: root `Cargo.toml` (14), `.github-gen/velnor-workflow.toml` (5), `crates/termrock-consumer-example/Cargo.toml` (3), `external-consumer-demo/Cargo.toml` (6), `external-consumer-demo/.config/nextest.toml` (1), and `rustfmt.toml` (5). The exact old and new file hashes are in [parsed-values.json](parsed-values.json). All six parsed TOML values, arrays, and table order match the fixed input. All six raw file hashes changed.

The root lint rationale remains in the [development guide](../../../../development.md#workspace-lint-rationale). [build-configuration.md](../../../../contracts/build-configuration.md) records Velnor ownership and history, the 2024 format setting, Nextest profile context, and the reviewed consumer boundary. It labels the R-43 size note as source-pinned history, not a current status.

The old metadata runbook remains unchanged. Its lines 89–90 incorrectly state that root lint comments and other manifest comments were retained. This runbook supersedes that statement.

## Decisions and comment scan

Sol high decision `T37-CONSUMER-BOUNDARY-01` preserves `termrock` as each consumer's only direct normal Termrock dependency. The facade has 19 production leaf dependencies. The decision does not preserve a zero-transitive-crate claim. The application and test-tool exclusion remains under T06 and T30. The external path consumer does not prove registry packaging; T34 owns that work.

Sol high decision `T37-RECOVERED-NEXTEST-01` accepts one exact historical exception: `docs/implementation/visibility/recovered-nextest.toml`, 993 bytes, SHA-256 `c69638050ddfec1b6be98a60fdb4ab269e0b851af7a13ea5e20523a694329d6b`. Its origin records an absent harness and keeps the file outside conventional Nextest discovery. Its embedded commands remain historical. This exception does not exclude other documentation.

The scan parsed 68 tracked `.toml` files and 7 TOML lockfiles. It found zero ordinary comments in the six target files, zero other ordinary comments in active TOML, zero parse errors, and zero unclassified comments. It records exact generated lock headers and oracle comments. The tracked `baselines/oracle/ORACLE.toml` blob is not materialized in this sparse checkout; its blob hash equals the present immutable oracle file. This is not a product failure. See [toml-comment-scan.json](toml-comment-scan.json).

## Verification

Velnor Actions 0.1.48 ran twice. Both commands exited 0. All 12 managed output files match the fixed input and each other byte-for-byte. The manifest still has 360 tasks: 90 each for build, Clippy, documentation, and formatting. Its 180 default/testing pairs have 162 identical argv/environment pairs and 18 feature-flag differences. The 13 prior-reviewed action and local-action references remain unchanged because every managed output file is identical. The `cargo_nextest` runner, `run_ignored = "default"`, and default/testing configurations remain in generator inputs. The generated manifest has no eligible Nextest run task; the lane is `ABSENT / NOT VERIFIED`.

The prior accepted generator record keeps `--no-tests fail` for eligible tasks. No eligible task exists in this generated manifest, so this correction adds no run gate and reports no behavior pass.

Guarded no-deps metadata passed for the root workspace and external consumer. Both outputs are byte-identical to the fixed input. Counts remain 47 packages and 68 targets in the root, and one package and two targets in the external consumer. Each consumer declares only `termrock` directly. All seven Cargo and mise lock hashes match the fixed input.

The root locked all-target/all-feature check and external consumer locked all-target/all-feature check passed. Root and external `cargo fmt --all -- --check` passed. The build logs report the existing `unused_braces` warning in `crates/termrock-feedback/src/hintbar.rs:193`; the prior accepted build log has the same warning. The three other standalone manifests and locks are byte-identical, so their accepted checks were not repeated.

All 22 E2E digest inputs match their prior Git blobs. The recorded digest remains `1777d5a93db4e1884aa1df1ef51323f4577e8d0347170ad2ba361dc39e4070ce`. This correction changed raw config hashes and does not claim those raw hashes are unchanged. It did not repeat the E2E build.

Exact command argv, workdirs, output hashes, lock hashes, and source binding are in [command-source-binding.json](command-source-binding.json). Build, format, metadata, generator, and disk logs are in this directory. `git diff --check` passed after the edits.

## Limits and status

No tests or product behavior cases ran. Selected cases: 0. Product behavior remains `NOT VERIFIED`. T37 remains active until the overall independent acceptance review. This correction does not self-approve T37, start T38, add a test, or add a replacement gate.
