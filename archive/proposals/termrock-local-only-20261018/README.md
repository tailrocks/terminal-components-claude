# Termrock local-only proposals — 2026-10-18

These three source proposals were recovered from local Git blob objects and are retained as historical artifacts. They are **incompatible with the current base and unreviewed for production**. Their presence here does not approve, install, or merge their behavior into the production tree.

| Archived source | Original blob object | Historical role |
| --- | --- | --- |
| `tools/visibility/status.py` | `8b1521fe5f101e8f44be818432d8eb81b5b66d67` | Python visibility status renderer and evidence checks |
| `crates/termrock-visibility-tests/tests/status.rs` | `c59c532a838e35afac0d90e79e2b0023fda011c6` | Rust status CLI contract tests |
| `crates/termrock-visibility-tests/tests/queue_v2.rs` | `d954a1c37c9993464d87ea2478e55943a0409011` | Q48 queue v2 integration tests |

The supplied IDs are blob IDs, not commit IDs, so this archive preserves only the three requested file proposals. It does not import their surrounding history or snapshots. The proposals are based in this branch on rescue commit `77fc91a82aeb145be4efc4403b18380ff40d21fb`.

The original DCO signatory is preserved verbatim. Sanitization is limited to machine-specific temporary paths and actor identifiers: test path values use generic fixture roots or the runtime temporary directory, while fixture actor identities use consistent role-based identifiers. The actor-specific review artifact basename is also generalized. These substitutions preserve the fixtures' path and role checks; the proposal source otherwise remains from the listed blobs.

Compatibility evidence is limited. A prior local status integration run against the current rescue base (`77fc91a82aeb145be4efc4403b18380ff40d21fb`) passed 38 of 61 tests; 23 failed because of fixture/current-base incompatibilities. That run used the anonymized DCO fixture identity, which has since been restored to the original signatory; no status test was rerun after that attribution-only restoration. The Q48 queue v2 integration test was not run because its matching snapshot was approximately 13.4 GB. These results do not establish that either proposal is compatible with the current base.

Review these files in their archived context before considering any production use. No production source or test file was changed by this proposal archive.
