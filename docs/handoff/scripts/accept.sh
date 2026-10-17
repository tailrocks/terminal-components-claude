#!/bin/sh
# Re-approve (tuiscotti accept) every failing capture named in $GATE_DIR/vis-*.txt, one name per call; run only after cells.py attribution.
# accept every failing capture of the vis-*.txt logs, one name per call
# (each from its own partition run); run from the worktree root
D=${GATE_DIR:?set GATE_DIR to your own scratch dir (archive, logs)}
TS=$(mise which tuiscotti)
grep -hE "^\s+FAIL \[" "$D"/vis-*.txt | awk '{print $NF}' | sort -u | sed 's#::#/#g' > "$D/accept-names.txt"
n=0; bad=0
while read -r name; do
  if "$TS" accept --store snapshots --actual target/tuiscotti/actual "$name" > "$D/accept-last.txt" 2>&1; then
    n=$((n+1))
  else
    bad=$((bad+1)); echo "FAILED $name"; cat "$D/accept-last.txt"
  fi
done < "$D/accept-names.txt"
echo "accepted $n failed $bad"
