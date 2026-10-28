# Final source-custody metadata addendum plan

Target branch/head for the draft: parity/termrock-visual-baseline at 49c6fe5444e08071c4630d19a61220b4c271e572.

## Files to add after review

- docs/consolidation/2026-10-29/CURRENT-CUSTODY.md: concise scope, current verified status, exact holds, and cleanup preconditions.
- docs/consolidation/2026-10-29/source-custody-index.jsonl: one record per discovered source repository, worktree, Git store, staging root, and non-Git project root. Keep the source inventories immutable; this index links them instead of copying or rewriting their contents.

## Required record fields

Each JSONL record should carry a stable source-root ID; source inventory and seal hashes; sanitized repository identity and captured HEAD/ref/stash/worktree set; per-store reflog path/hash and disposition (raw bytes captured, logical state reconstructed, explicitly empty, or raw bytes not retained); physical file/path coverage; dirty, untracked, ignored, sparse, and symlink coverage; exact PR-relative locator(s); object or payload dependency group IDs; source and transport digests/sizes; Safety/technical review decision IDs; remote verification head/tree and exact path/blob/mode/size results; state (verified, pending, partial, or concrete exclusion); and explicit limitations.

## Evidence that must be joined

Bind the home, tmp, global, and peer inventories and all sealed addenda, including the 13-worktree/1,598-ref/22-stash home inventory, the peer-custody intake at /private/tmp/pr38-consolidation-inventory-20261029/peer-custody-intake-64f810f8-7867-4020-9dd8-c4631dc6c88a.txt (SHA-256 cb58cf5b60e7f284608954c7593ede299361298162aa8bda220de6ad94114e80), raw reflog payloads where captured and explicit logical-only/missing-byte limits, the 83-store and Downloads TCC groups, the 37-Downloads-worktree closure and v1-v7 addenda, the source-delta-v4 routing map (seal SHA-256 bd3c46e6761499af10e51274f0fab8a801dd56d0edbe59be2355af6cad1e6d13; 49 route rows, 11 unresolved non-cache occurrences across six unique targets, eight payloads pending Safety; route-row SHA-256 547b804de7e6bb4dc83fafe4b4de247782e8618c76cf30f28854dbe6a5ded84b; summary SHA-256 bc2a4fd6845e16a1c5b84feff19db0d49e90ca4bdda0263000213d3b9c9321f9), the V1/V2 staging archive once eligible, and the V3 transport/privacy decision when available. Include the canonical d354 physical-worktree closure and correction, the V4g scanner source/self-check approvals plus final corpus result, and the remote-byte receipts for every published group. Preserve old source snapshots, ref names/OIDs, conflict sets, and archived alternative trees as data; do not collapse distinct roots merely because object or payload bytes deduplicate.

## Explicit limits and closeout gates

- Preserve the raw f629/c337 index-byte gap: logical stage/tree maps do not prove original raw-index bytes, flags, or extensions.
- Keep the invalid tmp_pack as opaque raw-byte custody only. Keep target caches separate and make no rebuildability claim.
- Do not label an archive verified based only on local staging, a GitHub tree listing, or a passing pack import. Each group needs its own exact remote-byte recovery decision.
- Use the exact-head DCO result: 35 missing sign-offs at check 123137595744. Record the current Actions failure separately; do not rewrite history or infer product parity from custody proofs.
- The final all-work audit must account for every discovered root/ref/reflog/stash/index/file occurrence and every unresolved item. Immediately before cleanup, revalidate each target against its captured source identity and manifest; if it changed, recapture and update remote custody. Only after independent remote recovery, fresh pre-deletion source-identity/state checks, and a separately reviewed cleanup allowlist may local project copies be deleted, and only within the user's already-authorized cleanup scope.
