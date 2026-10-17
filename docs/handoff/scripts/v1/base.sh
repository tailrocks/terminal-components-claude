#!/bin/sh
# Extract the read-only visual-baseline tag snapshots into $V1_DIR/base (never moves or rewrites the tag); run from the worktree root.
S=${V1_DIR:?set V1_DIR to your own scratch dir}
mkdir -p "$S/base"
git archive visual-baseline snapshots/showcase | tar -x -C "$S/base"
echo "extracted to $S/base/snapshots/showcase"
