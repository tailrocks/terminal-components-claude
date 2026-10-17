# Q48 proposal snapshot archive (local only)

This commit preserves five complete proposal snapshot trees as Git trees under `archives/q48-proposals-20261017/snapshots/<tree-object-id>`. The snapshot trees are retained byte-for-byte; they are not checked out or edited.

This ref is intentionally local-only. Do not push it to the public origin without a specific sanitization review. The preserved evidence includes personal-identifying material: a personal name, a machine hostname, and local user-home paths. The source evidence remains unchanged here to preserve its provenance.

The source object database was left intact. This rescue repository imported only the 28 snapshot-closure objects that were missing locally (27 trees and one blob). All imported payloads were checked against the source object database. The unique proposal `queue_v2.rs` blob is `d954a1c37c9993464d87ea2478e55943a0409011` (178,690 bytes).

## Preserved snapshot tree IDs

- `018958f489701c69db5cc1848d5063551eed0cfa`
- `563422bd05dba31043cc12c93c750634e65b3c82`
- `748f54fca6b93bb5e1f716edd0836ca8b83a94c1`
- `8c177e2f92084adc73995f5848d5b38a4a149b58`
- `d303814b8d8c919116e7dd2afa820aed7be0ff4a`

This archive is a preservation container, not a reviewed or publishable code change.
