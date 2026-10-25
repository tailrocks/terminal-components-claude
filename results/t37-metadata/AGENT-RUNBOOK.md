# T37 metadata result

## Status

This work has a structural manifest pass only. The final source fragment from the tools owner is not in this worktree. The root lock and four active standalone locks still need updates. CI output generation and the integrated build check are pending. This file does not claim a build pass or product behavior proof.

## Input and ownership

- Fixed input: `d402f893d159ac26da5db4d18c13510afdbce12f`.
- Original fixed source: `c817b99e9e43abbbf3519d56b2f5ee86924243d8`.
- Product source baseline: `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`.
- Accepted documentation baseline: `5bcc79b971ae30ee244b8fef841c6fbafdb918d0`.
- This worktree owns Cargo manifests and locks, `.velnor`, `.config`, generator inputs, CI output generation, inventory JSON, acceptance policy JSON, component ownership JSON, and `results/t37-metadata/**`.
- Foundation source fragments: `7368d4fc5a71e46cae5ee92963bcf3a6943953e9` and `2fee2c277d1213fc5fad5d81eb77479600182b69`.
- Component source fragments: `96fe6a2a1430967831777da2da6402206c4716ff` and `f03e22c14a5817272cd5643094e974f9c25f1b6a`.
- Application source fragments: `4f502be854a4d4c7a7585344bd9915b2a26fae79` and `fb667ed7ce6d58d204e49ad0dd10a66b9a9bef0a`.
- The tools source worktree is still active. Its deletion proposal SHA-256 is `b61d0374d87c1a17cd639f796562b614b8b19fe0d78d605ef49fb7f570032772`. Later inventory/schema proposal evidence SHA-256 is `c4f5ce52766a1d82782bbd6670e49297271307af9a292a2a2408debc1038bc13`. These are proposal inputs, not commit IDs or a combined-source build result.
- Do not self-approve this result.

Metadata commits:

- Cleanup: `bb0ea4d7a156f2f5d48d7b0d544281876f2ef4f5`.
- Evidence: `545b0cca194823fa73f3fbaca86ff60c6085f690`.
- Pending-source status correction: `5ccb1a25cee50b9a6f8e60278257e57271e6d4c0`.
- Corrected SHA references: `ef4753617fcc1e93cd43201b8d7b35f6dd80f88d`.
- Retained feature branch scan: `572212bf7b32813eb4ab3ddd3dde6df7b01370b6`.

## Metadata changes

- Removed `termrock-test-support` and `termrock-conformance` from workspace membership, default membership where applicable, and workspace dependencies. Their package manifests are retired.
- Removed their test-only dependency edges. Removed test-only development dependencies from the retained component packages, `visual-harness`, `termrock-e2e`, the publisher packages, `refactor-proof`, the visibility tools, and the external consumer package. Kept `termrock-shell`'s `ratatui` and facade development dependencies because its retained example uses them.
- Removed the orphan root workspace dependency entries listed in `dependency-retirement-map.json`. The remaining root workspace dependency keys have retained manifest consumers.
- Removed the ten explicit facade test targets. Kept all twelve examples. Set `autotests = false` and `autobenches = false` on all retained packages. Set `test = false` and `bench = false` on retained libraries, binaries, and examples. Set `doctest = false` only on library targets. No `harness = false` setting was added.
- Kept the visual-harness package and its four production binaries. Removed its `visual_baseline` test target and test-only development dependencies.
- Kept the e2e `admit_expected_generation` binary and its normal `tuiscotti` dependency. Kept the visibility `capture-command` and `nextest-result-reader` binaries. Removed the visibility `fixture` target table and `tempfile` dependency; the fixture source still exists in the fixed source input and must disappear with the reviewed tools source fragment.
- Retired the exact workbench-tabs-privacy and paint-style-pilot package manifests and locks, the frozen helper package and lock, and the qualified-capture harness lock. The qualified-capture harness manifest was absent from the fixed input. The workbench retirement uses the accepted application deletion map for the two fixture bins, README, and no-retained-consumer result. It does not infer retirement for other nested packages.
- Removed only `tools/test-inventory/profiles.json`. Kept `historical.json` byte-identical as a historical obligations catalog and `required.json` byte-identical as a pending, unapproved proposal.
- Reduced `.config/nextest.toml` to the active `ci` profile. Generated tasks use `--profile ci`; the profile remains required. The generated Nextest commands retain `--no-tests fail`.
- Removed the retired package paths from `.velnor` discovery exclusions. The global `testing` configuration remains unchanged pending the conditional high review.
- Did not change `tools/acceptance_policy.json` or `component-ownership.json`. The forbidden package names remain policy guards. The W01–W45 ownership map remains byte-identical.
- Removed stale Cargo manifest comments that cited the deleted doc-inventory test, reach harness, and public-only test. Root lint rationale comments remain unchanged pending the docs owner's accepted transfer.

See `dependency-retirement-map.json`, `retirement-input-hashes.json`, `preservation-hashes.json`, and `target-survivor-map.json` for exact paths and hashes.

## High-review decisions

- `T37-TARGETS-01` sets the target flags above. Production and example target identities, paths, crate types, required features, and documentation settings stay fixed.
- `T37-PROBE-01` retires only the reviewed workbench-tabs-privacy and paint-style-pilot probe packages. The workbench decision uses the two deleted fixture bins, README, and no-retained-consumer evidence. Other nested packages are not inferred from directory names.
- `T37-PROBE-02` retires the viewport testing hooks and facade testing re-exports. Existing compatibility feature edges remain. Required work bounds remain assigned to T17/T33.
- `T37-INVENTORY-01` retires only `profiles.json` and its capture/verify lane. This metadata commit removes the profile file. The historical catalog and pending proposal remain unchanged. The inventory runner deletion is pending the tools source closure. No replacement is added before T38.
- `T37-DOC-INVENTORY-01` approves retirement of the xtask documentation-inventory helper and its `pulldown-cmark` dependency. This metadata commit removes the dependency edge. The source deletion is pending the tools source closure. `serde_json`, ownership checks, and the xtask CLI remain.
- `E2E-01` keeps the `admit_expected_generation` CLI and its production publication path. `T37-E2E-SCHEMA-02` keeps all twelve schema files and the active registry/profile data byte-identical. The preservation hashes record all twelve schemas and three related data files.
- `T37-FROZEN-01`, `T37-QUALIFIED-01`, and `CAPTURE-01` approve retirement of their runnable helper closures. This metadata commit removes the frozen manifest and the frozen/qualified lock files. The remaining source deletions are pending the tools source closure. Qualified-capture pins, proof outputs, licenses, and repair provenance stay with the tools source owner.
- `T37-CI-TESTING-01` is conditional and currently blocked by `TextViewport::selection_binding_chords`. The coordinator requested `T37-SELECTION-BINDING-01`. Keep the global testing configuration until that decision and final combined-source proof resolve the branch. Do not treat an empty suite as a pass.

## Structural evidence

- Fixed-input root metadata had 49 packages, 44 libraries, 14 binaries, 12 examples, and 18 separate test targets. The metadata cleanup removed the two retired packages and those 18 separate test targets.
- The post-cleanup root metadata has 47 packages, 42 libraries, 14 binaries, and 12 examples. All 68 retained non-test targets match the fixed-input identity map by package, target name, kind, path, crate types, required features, and doc value.
- The structural scan found zero target-flag errors, zero orphan workspace dependency keys, and zero changes to retained package feature tables.
- Standalone no-deps metadata passed for e2e, visibility tools, external-consumer-demo, and tools visibility publisher.
- The fixed Velnor output has 235 default/testing task pairs. It has 211 identical pairs and 24 feature-flag differences across `termrock-core`, `termrock-runtime`, `termrock-shell`, `termrock-theme`, `termrock-viewport`, and `termrock`. Preserve the configuration until the final combined-source proof shows that all non-test feature branches are gone and the retained aliases add no production dependency or backend choice. Keep default build, Clippy, doc, and format tasks. Keep `--no-tests fail`. Keep T05/T32 and F03/T03 open.
- A read-only fixed-input scan found one retained testing-gated item: `TextViewport::selection_binding_chords` at `crates/termrock-viewport/src/viewport.rs:1566`. Sol high review confirmed this is a real testing branch and blocks the current CI-config retirement condition. It found four callers in conformance tests scheduled for removal and no established production caller. Sol requested `T37-SELECTION-BINDING-01` to decide whether to retire only this inspection method and cfg while preserving `SELECTION_BINDINGS` and selection/input behavior. The global Velnor testing configuration remains unchanged. No `CARGO_FEATURE_TESTING` build-script use was found.

## Commands and lock state

- `/Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo metadata --format-version 1 --no-deps --manifest-path Cargo.toml` passed with status 0.
- The same guarded no-deps metadata command passed with status 0 from each retained standalone workspace directory: `crates/termrock-e2e`, `crates/termrock-visibility-tests`, `external-consumer-demo`, and `tools/termrock-visibility-publisher`.
- `/Users/donbeave/.local/bin/rust-build-guard -- mise exec -- cargo metadata --locked --format-version 1 --no-deps --manifest-path Cargo.toml` was denied by the guard with status 125. Do not bypass this guard.
- `git diff --check` passed.
- Root `Cargo.lock` was `722145d6e1d713be9096727af7e83b5775c99b62095100c4fa8cc9a2b41bf36c` before and after the denied probe. The no-deps metadata command did not update it. The active standalone lock hashes before cleanup are in `lockfiles-before.json`.
- No tests, Cargo build, lock update, or Velnor generation ran in this worktree. The final lock update, generated output, and build check must run after the reviewed tools closure is in the combined input.

## Required integrated completion

1. Add the reviewed tools source fragment to the combined input. Recheck all manifest dependency removals against retained source and examples. Preserve immutable files and recorded hashes.
2. Regenerate root and active standalone locks with the guarded, scoped Cargo update command. Record removed lock entries and before/after hashes.
3. Resolve `T37-CI-TESTING-01` only from final combined-source evidence. If any meaningful non-test feature branch remains, keep the testing configuration and request a narrower decision.
4. Run `mise exec -- velnor-actions generate`. Review the complete generated diff and record workflow/task-manifest hashes. Do not hand-edit generated YAML.
5. Run `git diff --check`, the allowed guarded no-deps metadata command, and the guarded `cargo check --locked --workspace --all-targets --all-features` command. Compare failures with the accepted baseline at `c01a2553`; name the pre-existing owner for any retained failure. Do not run legacy tests.
6. Record `zero-case behavior: NOT VERIFIED`. T37 structural and build checks do not prove product behavior. T38 and later reviewed test plans remain open.
