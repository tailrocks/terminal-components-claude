# Frozen-source build provenance (observability only)

This increment records inputs for future frozen-source builds. It does not admit a rebuilt baseline, change a reference decision, relax the existing four-binary hash guard, or change frozen source, snapshots, tag, or Tuiscotti pin.

Each successful `build` writes `build-provenance.json` beside the legacy `build.json`. The sidecar has a versioned schema, a timestamp-independent cohort fingerprint, and a content self-digest. The legacy record keeps `source_sha256` and the four binary hashes and adds an optional digest pointer; existing validators continue to require the pinned source and exact binaries for the current stale-approval exception. A new hash set therefore remains unadmitted.

The sidecar records the exact frozen tag/commit/tree, source and archive hashes, Cargo.lock and toolchain-file hashes, runner and harness patch/helper manifest/lock/source/extension/decision hashes, build stage commands/order, executable hashes after each stage (including after Nextest's `--no-run` phase), host/target observations, Cargo/Rust tool paths and hashes, Rust/LLVM version output and sysroot identity files, plus best-available SDK/compiler/linker observations. Effective build environment values are represented only by SHA-256 for a fixed allowlist. Unknown environment values and configuration contents are not serialized. Since `clean_env` still inherits host `PATH` and other unreviewed variables, the sidecar explicitly says `observed-not-hermetic`; it is not a reproducibility or admission certificate. Link invocation and Cargo config semantics remain unobserved, and target override resolution may be incomplete.

Focused guards cover schema/version and required fields, self-digest and cohort-digest integrity, timestamp exclusion from the cohort fingerprint, allowlist-only environment recording, and secret non-leakage. Existing reference-decision and exact binary identity checks remain unchanged.

## Still required before any replacement-build decision

- Recover the original approved executable set if possible; otherwise a separate, independently reviewed reference decision must authorize a new, fully source-bound build cohort. Preserve the historical decision and all frozen approvals.
- Qualify the exact compiler distribution, linker/SDK, flags, Cargo configuration, effective environment, dependency inputs, and build stages in a controlled recipe; repeat independent builds and compare all four executable identities.
- Add per-child executable path/hash, argv, environment/profile, test binary and Tuiscotti identity receipts. The historical receipts do not provide this process binding.
- Capture, replay, and cross-compare full baseline inventories and ordinary CLI checks from independent source builds before candidate comparison. Diagnose old-approval conflicts separately; preserve both frames and require a documented independent reference decision.
- Add negative controls for wrong source/tool/harness, changed environment or argv, tampered binaries, candidate substitution, missing/reordered receipts, and deliberate behavioral/frame defects.

Until those gates and the separate decision are complete, provenance reports remain diagnostic-only and product parity remains unverified.
