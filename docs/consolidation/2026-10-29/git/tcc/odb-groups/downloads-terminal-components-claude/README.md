# Downloads TCC new-object delta (partial custody)

This group preserves the 3,747 loose Git objects in `/Users/donbeave/Downloads/terminal-components-claude/.git/objects` whose typed OIDs are absent from the canonical TCC object manifest. All bytes are the exact compressed source loose-object bytes; they are not decoded/recompressed.

The group depends on the canonical TCC archive at `../..` and its `inputs/physical-objects.tsv` (257,216 typed OIDs, SHA-256 `1e1094407122bfe6814f028cc360101490093ae7a3c4d5af589c808ff887beab`). The full Downloads loose inventory has 5,012 objects: 1,265 have matching canonical type/size and are represented by that dependency; the remaining 3,747 are stored here. The Safety review found no type/size conflicts or missing graph links for the canonical-plus-local object union.

Objects at or below 48 MiB are stored at their Git fanout paths under `objects/loose/`. The three larger raw encodings are split into chunks no larger than 48 MiB; `portable-object-map.tsv` and `objects/chunks/index.tsv` bind ordered chunk paths and hashes so the exact raw source bytes can be reassembled. The staged group reassembled and hash-checked all three.

This commit contains only the ODB object delta. Refs, reflogs, 37 linked-worktree indexes/admin state, sparse-checkout patterns, branch/upstream metadata, main worktree files, and the remaining 21 untracked/ignored content groups are separate pending groups. It is not an all-work custody or cleanup-complete claim.
