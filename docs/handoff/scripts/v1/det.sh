#!/bin/sh
# Run `cargo xtask visual determinism` for one V1 group filter and copy the showcase actuals to $V1_DIR/actual; run from the worktree root.
S=${V1_DIR:?set V1_DIR to your own scratch dir}
# usage: det.sh <filterfile> <tag>
F=$(cat "$1")
TAG=$2
start=$(date +%s)
cargo xtask visual determinism "$F" > $S/det-$TAG.out 2> $S/det-$TAG.err
rc=$?
echo "rc=$rc secs=$(( $(date +%s) - start ))"
tail -3 $S/det-$TAG.err
grep -c . $S/det-$TAG.out
mkdir -p $S/actual
rsync -a target/tuiscotti/actual/showcase $S/actual/
