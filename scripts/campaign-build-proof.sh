#!/usr/bin/env bash
# Build the native proof comparator required by verifier-subagent checks.
# This is a host-local build step; taskfmt remains verifier-only.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"

cd "$ROOT"
[[ -z "$(git status --porcelain)" ]] \
  || { echo "campaign-build-proof: worktree must be clean at the verifier commit" >&2; exit 1; }
cargo build --locked --offline -p refactor-proof --bin tc-proof

BINARY="$ROOT/target/debug/tc-proof"
[[ -f "$BINARY" && ! -L "$BINARY" && -x "$BINARY" ]] \
  || { echo "campaign-build-proof: native comparator missing: $BINARY" >&2; exit 1; }

RECEIPT="$ROOT/target/debug/tc-proof.build.json"
COMMIT="$(git rev-parse HEAD)"
SHA256="$(shasum -a 256 "$BINARY" | awk '{print $1}')"
python3 - "$RECEIPT" "$ROOT" "$COMMIT" "$BINARY" "$SHA256" <<'PY'
import json
import sys
from pathlib import Path

receipt, root, commit, binary, sha256 = sys.argv[1:]
Path(receipt).write_text(json.dumps({
    "schema": "tc-proof-native-build/v1",
    "worktree": str(Path(root).resolve()),
    "commit": commit,
    "binary": str(Path(binary).resolve()),
    "binary_sha256": sha256,
    "command": ["cargo", "build", "--locked", "--offline", "-p", "refactor-proof", "--bin", "tc-proof"],
}, sort_keys=True, indent=2) + "\n")
PY

echo "native tc-proof comparator: $BINARY"
echo "native tc-proof build receipt: $RECEIPT"
