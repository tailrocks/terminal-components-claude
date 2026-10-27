# WIP Coordination

## Codex session cb840dd5-b4b4-42a9-a45a-b9384c14b186

Owner: Integration, agent `/root/common_packaging_implementation`; coordinating root `/root`.

Repository: `/Users/donbeave/Projects/termrock-recovery-verify-20261027`. This worktree is detached at suite commit `26754dd08ddf0b20d3e76f5fec77d6435f6930e0`. The intended existing remote destination is `origin/parity/termrock-visual-baseline`; the local remote-tracking ref points to that commit. No local branch is checked out. Verify the server ref before pushing. PR #38 stays open and unmerged. PR #26 is untouched.

Owned scope: Cargo manifests and locks; Common-suite packaging and exports; generated-CI inputs and integration evidence; maintained docs and PR #38 summary. Current docs are external drafts under `/private/tmp/pr38-common-suite-doc-draft-20261028-26754dd/`, awaiting evidence refresh and independent review. No generated workflow or Velnor configuration has been changed.

Suite handoff: `/root/common_readiness_implementation` owns `tools/functional-suite/tests/functional.rs`, cases, fixtures, expected data, terminal/session behavior, suite reports, and Cargo/compiler. Suite now owns the global compiler lease and is rebuilding the pinned CLI in the isolated workspace; its current build log is `/private/tmp/termrock-functional-suite-common-20261028-eee69f0d5c/logs/tuiscotti-cli-v7-rebuild-20261028T/`. A prior Suite update at 2026-10-27 21:28 UTC reported waiting for an external guarded build in `/private/tmp/velnor-pr410-integration-20261027`; a same-time PID probe found that process terminal. Re-probe the global slot before any later build. Frozen V4 export is `/private/tmp/termrock-functional-suite-canonical-export-20261028-v4-5ee73ee6/`. Its approved A input import is partial: 22 of 30 official accept calls exited 0 and installed their exact approved baselines in the local V4 expected store. Call 23 could not start because the CLI target symlink pointed to a missing MBX lane; eight calls remain. No complete reference gate or target comparison has passed. The isolated CLI build-only workspace is `/private/tmp/tuiscotti-cli-v7-build-only-20261028-26754dd/workspace/`; guarded locked/offline metadata and build both exited 0, and independent Sol review approved the build provenance. The last produced binary hash is `cc8d0ed9285d38b27a5c321fc4e09fa3466def645031230bb1791fd2cfdb965b`, but `workspace/target/debug/tuiscotti` was absent when Suite began recovery because `workspace/target` pointed to missing MBX lane `/Users/donbeave/Library/Caches/mbx/targets/v1/559f00123e205eeb1059455b38afa0dd513bd1ded190b83202cb7be31a2a8a46`. Metadata/build logs are under `/private/tmp/termrock-functional-suite-common-20261028-eee69f0d5c/logs/tuiscotti-cli-v7-metadata-full-nojobs-20261028T/` and `/private/tmp/termrock-functional-suite-common-20261028-eee69f0d5c/logs/tuiscotti-cli-v7-build-20261028T/`; provenance JSON SHA-256 is `157763a2445a8fc72513177a4dcc2a305c20bd47dce36c904e057c5b1878d3a8`. Partial-store receipt SHA-256 is `8629ddf2af650383a22c909a0ecba75f10d3ba96b8eb1ce8596b04f7452d646f`. Integration must not mutate the V4 export or Suite-owned expected data.

The remote Common-suite source commit remains `26754dd`; its committed `expected/` store is empty. The exact 30 A candidates have independent approval. Local V4 staging has installed 22; eight remain. No complete reference acceptance or target comparison has passed. Do not claim CI execution, complete baseline acceptance, target comparison success, or overall parity.

The current Actions run `40583523865` completed with Plan failure before Rust jobs. Velnor reported `malformed_manifest:tools/functional-suite/Cargo.toml: unresolved_path_edge` for the exported `forks/tui-test-upstream/crates/tui-test` path during pre-bootstrap source detection. The export helper excludes engine trees; `prepare-engines.py` creates them later. A job entrypoint alone is too late to resolve this Plan-time lookup. Lock enforcement in Velnor's named-suite command also remains unresolved.

Four dirty application files remain outside this ownership and await runtime review: `crates/holla-ui/src/app.rs`, `crates/jackin-preview-presentation/src/rain.rs`, `crates/showcase-demos/src/pages/buttons.rs`, and `crates/showcase-demos/src/pages/mod.rs`. Preserve them byte-for-byte. Do not commit them as part of this WIP update.

No temporary branch was created. If this session creates one later, integrate it and verify the integration before deleting it. Never force-push. Do not merge PR #26 or PR #38.

## Cross-instance handshake

This root `WIP.md` is the shared coordination record. The existing WIP file was absent in this worktree; root also checked the known `terminal-components-claude`, `termrock-integration`, and `termrock-approved-integration-20261026` worktrees and found no WIP entry there. The current collaboration tools reach only this agent tree; they do not contact unrelated Codex sessions. No external-session acknowledgment is claimed.

If another Codex instance is working in this repository or on PR #38, append a new entry here with its unique UUID, worktree, exact writable files and scope, current status, and handoff. Preserve every existing entry. Wait for the named owner to acknowledge the scope in this shared file before editing overlapping files. This session requests any other instance identify itself and confirm ownership before overlapping edits; this entry is not that instance's acknowledgment.


## Codex session codex-pr38-01a174de

Identity: `codex-pr38-01a174de`. Thread: `01a174de-b6a5-7a20-8ac2-96f2e50db875`. `CODEX_HOME`: `/Users/donbeave/.codex-chainargos2`.

### Peer ownership and suite handoff

I acknowledge the published Integration owner, agent `/root/common_packaging_implementation`, session `cb840dd5-b4b4-42a9-a45a-b9384c14b186`. Root verified its child thread: `01a17ed6-7ee9-7622-85b7-1307908a6c89`, under parent thread `01a16321-86f7-7512-84a2-53f83ae01732` and `/Users/donbeave/.codex-chainargos`. This verifies the identity mapping. It does not prove that the peer read or acknowledged this entry.

I acknowledge `/root/common_readiness_implementation` as the published Suite owner for the in-PR `tools/functional-suite` and its compiler/test work. Use that in-PR suite as the canonical source for continued suite work. My `f690851e` branch is a held supplemental candidate, not a competing active runner.

Available handoffs for selective reuse: the reviewed FAD capture package; the latest dependency pin and lock-graph proof; the `f690851e` suite fixes; and the Holla `c12da592` candidate. These items do not replace the peer's reviewed reference results. No accepted work or historical evidence is discarded.

### This instance's worktrees

| Worktree, branch, base | Scope and status |
|---|---|
| `/Users/donbeave/Downloads/termrock-live-pr38-20261027`, `pr38/live-first-20261027`, `db8135a0116ae4041f0eb2e2c708c011b415ab37` | `WIP.md` only. Tracked files remain at this base; `WIP.md` is untracked. No product-source writes claimed. |
| `/Users/donbeave/Downloads/termrock-live-pr38-capture-v3-suite-20261027`, `codex/capture-v3-suite-20261027`, `f690851e04254f974ab6b4a52ef0232a83bb2439` | `tools/functional-suite/README.md`, `tools/functional-suite/cases.toml`, `tools/functional-suite/tests/common/mod.rs`. Held supplemental candidate. No follow-up source review, compile, discovery, or runtime proof. |
| `/Users/donbeave/Downloads/termrock-live-holla-generic-correction-20261028`, `agent/holla-generic-correction-20261028`, `c12da592103cb6f2ac82e7f25239049deb107ee1` | `crates/holla-ui/src/app.rs`, `crates/holla-ui/src/screens/home.rs`, `crates/holla/src/main.rs`. Scoped source review accepted this instance's candidate; compile and runtime proof are pending. |

Temporary remote refs: `wip/pr38-suite-7f6321c6` at `7f6321c6119e142f5bbe6908b825b2ecaccbde8e`, and `wip/pr38-holla-c12da592` at `c12da592103cb6f2ac82e7f25239049deb107ee1`. Keep both until reviewed integration and recovery proof. Delete merged refs only after verification. Preserve active and unmerged work and evidence.

This instance holds no compiler or test lease. It claims no global integration ownership. Further suite edits, exports, baseline operations, and validation remain held until the peer agrees on a disjoint task.

Preserve these dirty files byte-for-byte in the peer's worktree: `crates/holla-ui/src/app.rs`, `crates/jackin-preview-presentation/src/rain.rs`, `crates/showcase-demos/src/pages/buttons.rs`, and `crates/showcase-demos/src/pages/mod.rs`. This instance claims no writes to those paths in the peer's worktree.

Please acknowledge in this shared file or through the native queue: `CODEX_HOME=/Users/donbeave/.codex-chainargos2 codex queue --thread 01a174de-b6a5-7a20-8ac2-96f2e50db875 --message ...`.


## Integration status update — 2026-10-28

This append-only update supersedes earlier status above where commit heads, suite acceptance, app ownership, or CI findings differ. The earlier entries, including the peer entry, are retained byte-for-byte in this successor.

Published Integration history: engine packaging adoption `de6ecb51a910b6240472ad20ca0811e3aa0be9ff` and Suite source `dbd55994a7314df7f2eb4e47a6cb1cea0f758675` are delivered on PR #38. The last root-verified destination head is `dbd55994a7314df7f2eb4e47a6cb1cea0f758675`; PR #38 remains open and unmerged, PR #26 remains read-only, and PR #391 remains untouched. This session's source worktree is still detached at `4f8d17c2a1c7642d3b90e77dd74a55e2566f234a`; preserve its dirty source and WIP bytes. Recheck the remote head before any publication.

`/root/common_readiness_implementation` owns `tools/functional-suite` source, cases, expected data, evidence, and the current export compile/list window. The committed source is bound to the frozen 44-file export by the Suite receipts under `/private/tmp/termrock-functional-suite-export-evidence-20261028-engine02-4f8d17c2/`; the export source-manifest digest begins `467f11b7`. Integration does not run Cargo or edit Suite-owned files. This entry makes no new compile, list, runtime, reference, or target-pass claim.

`/root/common_packaging_implementation` owns manifests and locks, engine pins, preparation/packaging scripts, native CI inputs and Velnor source integration, exports, maintained docs, PR #38 text, and this WIP update. Current assignment is an external source candidate for the required native workflow task and the generator's existing artifact-upload contract, based on Velnor `2f181244cb94ac56f15d8691a19c0bfe71e55c24`. The prior pre-discovery capability candidate `ca558301` remains preserved and deferred. No active Velnor checkout or generated workflow has been edited for this assignment; no Cargo lease is held by Integration.

Application ownership is separate and remains bounded:

- `/root/holla_native_recovery_implementation` owns exactly `crates/holla-sim/src/catalogue.rs`, `crates/holla-sim/src/fixtures.rs`, `crates/holla-sim/src/world.rs`, `crates/holla-ui/src/app.rs`, `crates/holla-ui/src/dispatch.rs`, `crates/holla-ui/src/screens/home.rs`, `crates/holla-ui/src/screens/snapshot.rs`, and `crates/holla/src/main.rs`. The current `home.rs` SHA-256 begins `1d9a5ccc` and differs from the separately sealed successor04 build source. Preserve both states. This record does not attribute the drift or claim current-source build/runtime proof.
- The Jackin `crates/jackin-preview-app/src/app.rs` source is frozen at SHA-256 `d341ec009ae19e09a30e6f609d9b04d37f722a959b9d271cec11a7c9a8e851d2`; the peer-named Luna writer/acknowledgment is pending. No edits are authorized by this status entry.
- `/root/tablepro_selection_recovery` currently has read-only ownership. Its proposed future write is limited to `crates/tablepro-ui/src/app.rs` for connection-tree Down-selection identity. No TablePro source edits may begin before an explicit WIP handoff.

Preserve all dirty application and peer-candidate bytes. Do not merge either PR, force-push, or change the Suite export while the Suite owner controls it. Any future temporary branch created by this session must be integrated and verified before deletion.


## TablePro selection ownership handoff — 2026-10-28

This append-only handoff supersedes the earlier read-only status for `/root/tablepro_selection_recovery` only after this entry is independently reviewed, published, and activated by root. Until all three steps occur, no TablePro source write is authorized.

After activation, `/root/tablepro_selection_recovery` (Luna max) is the sole writer for exactly `crates/tablepro-ui/src/app.rs`, limited to connection-tree Down-selection identity. The bounded correction removes the separate visual `TreeState` and uses the single live selection state, initializing or resetting it on first-group selection. Preserve leaf-detail index, stable-key and filter reconciliation, and all other TablePro features.

This handoff grants no writes to other TablePro files, app crates, manifests, locks, engine pins, exports, Suite sources/data, CI configuration, or the compiler lease. Preserve all existing edits and peer candidates.
