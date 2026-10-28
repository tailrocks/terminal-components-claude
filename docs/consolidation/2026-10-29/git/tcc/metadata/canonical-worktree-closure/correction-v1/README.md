# Canonical worktree physical closure correction

This correction supplements the sealed no-follow scan in `canonical-worktree-closure-v2`. Its physical-file rows and hashes remain authoritative. It fixes only mode interpretation: the two `0o755` lstat values belong to Git symlinks (mode `120000`), so they are not executable-bit mismatches.

At HEAD d3546b, all 3,596 regular files and both tracked symlinks outside the published archive subtree exactly match commit-tree blobs. The only extra path is the ignored `target` symlink, whose 110-byte absolute cache-reference string is stored as a content-addressed metadata object. The destination under `Library/Caches/mbx/targets` was not inspected or followed. No project regular-file bytes outside the commit tree were found.

The historical f629 physical checkout is not retroactively proven by this current scan; only its 36 frozen paths have explicit mapping.
