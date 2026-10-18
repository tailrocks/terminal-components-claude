# PR 17 DCO minimal rewrite projection — q31 refresh

Captured **2026-10-09T02:05:24Z** for `tailrocks/terminal-components-claude#17`, base `81a8bf15cd3042f80649e2b48fed479829518dbd`, observed branch head `28058beecf63acee3b65bab847e507fe8e1c539d` (tree `3a58036c595688898fc989ef6453e6d9d512f230`). The exact merge base remains `cc14dd6beae526884aabdf897e309be837b4f504`; the range contains **473 commits**. Read-only `git ls-remote` observed both the unchanged base branch and the q31 PR branch. No checkout, index, ref, object database, or remote branch was changed.

## Audit result

The repository rule requires `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>`. The refreshed full history has **20** commits without a sign-off and **453** with exactly one required trailer; there are no wrong or duplicate sign-offs. q31 is signed and is an additional descendant of e44. The saved DCO app run remains evidence for e44 only; this package makes no current check-run claim for q31.

## Projection

The two previously reported non-merge DCO failures remain `cb89f1d61a0d8d78a1b7b7c872de16c12131e8b1` and `99c9524772f717111489db5324ccd8632138c8d9`. Their descendant closures now have sizes **82** and **81**, with an **83-object** union. The projection appends the q31 commit as one additional affected descendant: its message and tree stay unchanged, while its parent maps from e44 to the projected e44 object. The four message edits remain the two red commits and recovery merges `1f6411c7d10f16506e73f6463bd09865d9bac239` and `cc3ce8f6ac149aaee406047c5180d1a658d6bb9c`. The other 79 projected messages already contain the exact trailer. The projected head is `6819529c2d24d871ba1c251988d8544cfaa82c51`. No Git objects were written.

This minimal projection leaves **16** unrelated unsigned PR commits unchanged, including 14 older omissions outside the DCO bot's 250-row view and two earlier skipped merges. It does not claim full AGENTS compliance. Local-only commit `186c28774ae6abe740fc7663a3db2e17444f086a` remains excluded from the remote PR range; local checkout facts in the e44 audit are historical and were not refreshed.

## Recovery and limits

The previously verified full bundle for e44 remains unchanged at `/private/tmp/termrock-pr17-dco-backup-e44-20261009-r1/termrock-implementation-e44.bundle` (5,757,777,217 bytes; SHA-256 `da8d1b85c40625b86f6c49f843ab221ff34211d7bfa3fa8b23f08496a284b80e`). It covers e44, not q31. Its original record is preserved as `source/e44-backup-record.json`. No rewrite or push is part of this package; any later operation must take a fresh exact-head lease against `28058beecf63acee3b65bab847e507fe8e1c539d` and regenerate if the branch moves.

## Package

- `audit.json`: refreshed 473-commit counts and explicitly historical e44 check evidence.
- `old-to-new-map.json`: 83 deterministic projections, including the q31 parent remap.
- `source/old-commit-objects/`: all 473 exact raw source commit payloads, each verified against its Git OID.
- `source/remote-head-observation.json`: fresh read-only branch/base observation.
- `source/e44-backup-record.json`: unchanged record for the separate e44 recovery bundle.
- `source-manifest.json` and `SHA256SUMS`: hashes for the new R2 package.
