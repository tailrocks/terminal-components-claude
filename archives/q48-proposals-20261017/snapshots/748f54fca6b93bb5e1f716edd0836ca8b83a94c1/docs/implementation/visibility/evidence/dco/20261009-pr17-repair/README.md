# PR #17 Developer Certificate of Origin (DCO) repair evidence

This packet records the limited DCO repair of branch `refs/heads/termrock-implementation` on 2026-10-09. It records a DCO check and a limited history repair. It does not report product-test success or visibility completion.

## Published result

The branch moved from `28058beecf63acee3b65bab847e507fe8e1c539d` to `6819529c2d24d871ba1c251988d8544cfaa82c51`. Both commits have the same tree, `3a58036c595688898fc989ef6453e6d9d512f230`. The reviewed R2 (revision 2) old-to-new commit map covers 83 commit objects and records four message edits. The independent post-push reviewer verified all 83 actual commit objects and 82 distinct trees against that map in the external Git mirror.

The repair added the required `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>` line to two non-merge commits reported by the DCO check. It also added the line to two merge commits among the descendant commits recreated during the rewrite. The DCO check did not report those merge commits. The repair changed them so every newly recreated commit object has the required line. The repair kept the author and committer headers, tree IDs, and parent links unchanged. `old-to-new-map.json` lists each old and new commit ID.

GitHub DCO check 113639531062 completed with conclusion `success` for the new head at `2026-10-09T02:24:09Z`. The audit of all history before the repair found 20 commits without a sign-off line. The repair changed four commits in this limited scope. Sixteen older omissions remain unchanged. This packet preserves the GitHub DCO check result and the all-history audit. A passing check does not remove those 16 historical omissions.

## Captured records

- `publication.json` records the root-observed commit-rewrite tool and push exit codes. It records the old and new heads, equal tree, and unchanged local HEAD and index. It also records the frozen tag object and target commit IDs, and the DCO check ID.
- `dco-check-runs.json` is the GitHub check-runs API response for DCO check 113639531062.
- `remote-head-and-tag.raw` records the post-push remote branch head and the frozen tag object and target commit.
- `rewrite-audit.json`, `old-to-new-map.json`, and `rewrite-source-manifest.json` bind the scope, source inventory, and complete 83-object rewrite map.
- `rewrite-independent-review.json` reviews the old-to-new commit map. `materializer-independent-review.json` reviews the commit-rewrite helper and its successful read-only preflight check. `post-push-actual-review.json` reviews the 83 objects after publication.
- `materializer-preflight-receipt.json` and `materializer-preflight.stdout.raw` record the preflight check. It verified the inputs and 83 planned objects. It wrote zero Git objects and left Git references unchanged.
- `prior-materializer-failure-observation.json` preserves the earlier R1 (revision 1) attempt to load the `.mjs` file. That attempt failed before Git ran and wrote no objects.
- `original-head-backup-record.json` binds the `q31` incremental recovery bundle to the previously verified complete `e44` backup.

The `e44` full backup bundle is 5,757,777,217 bytes. The `q31` incremental bundle is 3,550 bytes. `original-head-backup-record.json` lists their external paths. This repository packet does not contain either bundle. The two bundles are sufficient to reconstruct the original q31 history. No restore test was run.

The commit-rewrite tool's successful write output exists only in Root's tool execution record. This packet has no raw copy. The exact push command and output were also not saved as raw files. This packet does not recreate either capture. `publication.json`, the post-push remote snapshot, and the independent actual-object review preserve the available observations.

## Scope limits

The frozen `visual-baseline` tag remains at object `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5` and target commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. The DCO check is a repository gate only. No product visibility result has been accepted. The refactor is not ready.
