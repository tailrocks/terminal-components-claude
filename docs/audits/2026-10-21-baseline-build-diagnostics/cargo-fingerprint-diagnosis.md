# Diagnostic frozen-build A2/B2: Cargo fingerprint review

Status: **diagnostic failure; no build cohort admitted**. This note supplements (and does not replace) the immutable A2/B2 receipts and independent comparison report.

## Fixed evidence

- Recipe: `2409404d35f3d92424c9a416436a91fe1578045ecc45110dd9c509b97ae09f62`.
- A2/B2 independent comparison: `independent-ab-comparison.json`, SHA-256 `851a89d1a193e8d20f3adf0b94c873153bab55cf0dd9fa80c488b84b2aa1b093`.
- Source, live-source and provenance records are equal; metadata dependency graph is equal. The comparison reports all four application binary hashes differ at stage 1 and after the pinned harness stage 2.
- A2 prepare and prepare-live argv used `--timeout 220`; the reviewed recipe and B2 used `--timeout 225`. Their receipts are preserved as recorded. This is a second independent reason the A2 attempt does not satisfy the exact recipe.

## Cargo fingerprint observations

Read-only inspection of the two controlled targets found 184 matching `.fingerprint/**/*.json` relative paths. After replacing only each exact A2/B2 root prefix with `<ROOT>`, 112 files still differ. In those files, Cargo records dependency fingerprint values that differ. For example, `anstream` has the same relative fingerprint filename in both targets, while its dependency fingerprint numbers differ.

The fingerprint JSON also stores the `rustflags` array verbatim. A2 contains `--remap-path-prefix=/Users/donbeave/Projects/termrock-cli-evidence-20261021/controlled-baseline-a2/{live-source,controlled-target}=...`; B2 contains the corresponding `controlled-baseline-b2` prefixes. Both remap to the same canonical destinations, but the input strings differ. Thus the target-root normalization used for report comparison does not make Cargo's stored fingerprint input equal. This is evidence that Cargo observed path-distinct build flags and computed distinct fingerprints; it does not, by itself, prove these flags are the sole cause of the executable byte differences.

The current receipts/logs do not capture verbose rustc command lines. Therefore this review cannot directly report or compare rustc `-C metadata`, `-C extra-filename`, or exact per-unit stable crate disambiguators. Fingerprint directory names and unit artifact names are insufficient proof of the actual command arguments. No build was rerun to obtain them.

A source scan of the frozen tree found `env!("CARGO_MANIFEST_DIR")` only in test-only code (integration tests, `#[cfg(test)]` modules, or test macros) and no direct use in the four production application code paths. It provides no current evidence that production application binaries embed the two source root paths through that macro. The harness does compile tests and the paths may appear in test artifacts; this diagnosis does not claim otherwise.

## Independent Mach-O correlation

A separate read-only sealed-binary review found all 17 named section hashes equal across A2/B2, including `__text`; load-command bytes, UUIDs, and file sizes also match. Its byte attribution found every differing non-signature byte in N_OSO debug-map string-table entries: source/target-root paths and `.rcgu.o` filename suffixes differ (examples reported: showcase `07nhpnx` vs `0cr07r4`; `junie_tui0wv81bk` vs `0np0c1h`). All remaining differences are CodeDirectory SHA-256 page-hash slots, independently checked against the unchanged raw bytes. This supports a debug-map/signature explanation for the observed executable mismatch and provides no evidence that instruction/data sections differ. The per-byte report is `/Users/donbeave/Projects/termrock-cli-evidence-20261021/controlled-baseline-b2/macho-diagnosis/summary.json`, SHA-256 `0ec2ac722994d09522619a721983429a61045c59763c28e63a384866e8f07a8c`. It confirms zero N_OSO timestamps, valid ad-hoc signatures, and independently recomputed page hashes. Differing byte counts by binary (N_OSO root / object suffix / signature page-hash slots): Showcase 585 / 3054 / 1401; TablePro 548 / 3075 / 1311; Jackin 565 / 2940 / 1339; Holla 556 / 2886 / 1305. These findings explain the observed byte deltas but do not establish an accepted reference cohort or authorize normalization.

## Pinned Rust compiler condition for object suffixes

The exact rustc source at commit `58cf8f9c35b355199727627d07ae34a4e2846d67` was inspected and preserved under `compiler-source-evidence/` (raw source SHA-256: `util.rs` `f2b1d1937e728f00f8d62ead507e61b662d1ac8cfba6aa83e8dc3db71a813133`; `config.rs` `bae41228cdaa3122b80f9dd90f8d4f5d3bd09c2922d23d1f8ba64a5ea2579f14`). `rustc_interface/src/util.rs` sets `invocation_temp` to a random value only when the incremental option is present. `rustc_session/src/config.rs` appends that value to codegen-unit temporary object filenames. Therefore a shared physical source/target path fixes path prefixes and raw RUSTFLAGS but does not fix these random suffixes when incremental compilation is enabled. `save-temps` and `--emit` do not disable the constructor condition.

The pinned Rustup distribution's bundled Cargo profile reference is preserved by hash `2178bd9b63c5be9e3546b765d1ed6aad92efefee50923e4f54dc292b50954773`. It documents that `incremental` maps to `-C incremental`; the default codegen-unit count is 256 for incremental and 16 for non-incremental builds. The frozen manifest also has `[profile.dev.package."*"] opt-level = 3`; do not overwrite this package override with a blanket optimization/profile setting. A future diagnostic recipe could explicitly disable incremental while leaving all debug/profile settings untouched, but that changes build strategy and the codegen-unit default. It needs a fresh exact independent review and parent authorization. No flag experiment or rebuild has been run.

The serial-fixed-path proposal (`serial-fixed-path-proposal.json`, SHA-256 `a095304ca8dfd7528efa9e9fe385f15974d6d4c49c9542bc2817ee2fadb71caf`) notes this source condition but does not disable incremental compilation; it was never executed and is insufficient to promise identical object suffixes. Treat it only as preserved proposal evidence, not an approved recipe.

## Interpretation and next step

The A/B runs are not a reproducibility pass. The exact root-bearing `RUSTFLAGS` are a confirmed non-equal Cargo input; they can invalidate fingerprints even though rustc remaps source and target paths to the same virtual locations. The binary hash mismatches remain unexplained beyond that observed discrepancy. Independent Mach-O inspection is separate evidence and must be retained alongside this note.

Any new diagnostic attempt requires a newly reviewed recipe that makes the complete effective compiler arguments and Cargo fingerprints comparable, or records their exact differences as a deliberate red result. Do not edit these receipts, normalize executable bytes, change the source/tag/tool pins, change expected data, relax immutable admission, or treat a new cohort as accepted based on this hypothesis.
