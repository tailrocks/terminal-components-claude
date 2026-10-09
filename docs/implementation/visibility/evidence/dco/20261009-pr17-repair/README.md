# PR #17 DCO repair evidence

This packet records the limited DCO repair of `refs/heads/termrock-implementation` on 2026-10-09. It records a repository check result and a narrowly scoped history repair. It does not report product-test success or visibility completion.

## Published result

The branch moved from `28058beecf63acee3b65bab847e507fe8e1c539d` to `6819529c2d24d871ba1c251988d8544cfaa82c51`. The old and new commits have the same tree, `3a58036c595688898fc989ef6453e6d9d512f230`. The R2 projection maps 83 commit objects and records four message edits. The independent post-push reviewer verified all 83 actual commit objects and 82 distinct trees against that mapping in the external mirror.

The repair added the required `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>` line to the two DCO-reported non-merge commits and to two merge commits in the rewritten descendant closure. The merge commits were not reported as DCO failures; they were edited so every newly recreated commit object has the canonical line. Author and committer headers, trees, and parent topology were preserved. `old-to-new-map.json` provides each old/new object binding.

GitHub DCO check 113639531062 completed with conclusion `success` for the new head at `2026-10-09T02:24:09Z`. The pre-repair all-history audit found 20 commits without a sign-off line. Four were changed in this limited repair; 16 older omissions remain unchanged. The provider check result and the all-history audit are both preserved here; the check conclusion does not erase those 16 historical omissions.

## Captured records

- `publication.json` records the root-observed materializer and push exit codes, old/new heads, equal tree, unchanged local HEAD/index, immutable tag IDs, and DCO check ID.
- `dco-check-runs.json` is the GitHub check-runs API response for DCO check 113639531062.
- `remote-head-and-tag.raw` records the post-push remote branch head and frozen tag object/peeled commit.
- `rewrite-audit.json`, `old-to-new-map.json`, and `rewrite-source-manifest.json` bind the scope, source inventory, and complete 83-object projection.
- `rewrite-independent-review.json` reviews the R2 projection; `materializer-independent-review.json` reviews the R2 helper and successful read-only preflight; `post-push-actual-review.json` reviews the 83 objects after publication.
- `materializer-preflight-receipt.json` and `materializer-preflight.stdout.raw` record the preflight. Preflight verified the inputs and 83 planned objects, wrote zero Git objects, and left refs unchanged.
- `prior-materializer-failure-observation.json` preserves the earlier R1 `.mjs` module-load failure. That attempt failed before invoking Git and wrote no objects.
- `original-head-backup-record.json` binds the q31 incremental recovery bundle to the previously verified complete e44 backup.

The e44 full backup bundle (5,757,777,217 bytes) and q31 incremental bundle (3,550 bytes) remain at the external paths recorded in `original-head-backup-record.json`; they are not copied into this repository packet. The pair is sufficient to reconstruct the original q31 history. No restore rehearsal was run.

The materializer’s successful write-mode stdout was available only in Root’s tool execution record and was not saved as a raw file. The exact push command/stdout was also not saved as a raw file. This packet does not recreate either capture. `publication.json`, the post-push remote snapshot, and the independent actual-object review preserve the available observations.

## Scope limits

The frozen `visual-baseline` tag remains at object `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5` and peeled commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. The DCO check is a repository gate only. Product visibility remains unaccepted and refactor readiness remains not ready.
