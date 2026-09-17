#!/usr/bin/env bash
# Copy workspace-built refactor-proof binaries to tools/refactor-proof/bin/.
# The comparator is the only accepted proof executable. Keep this script limited
# to copying that standalone binary; verifier lifecycle orchestration is retired.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
workspace="$(cd "$root/../.." && pwd)"
profile="${PROFILE:-debug}"
target="$workspace/target/$profile"

built="$target/tc-proof"
dest="$root/bin/tc-proof"
if [[ ! -f "$built" ]]; then
  echo "sync-binaries: missing $built (run: cargo build -p refactor-proof)" >&2
  exit 1
fi
cp "$built" "$dest"
chmod 755 "$dest"

echo "sync-binaries: installed $(shasum -a 256 "$dest" | awk '{print $1}') -> bin/tc-proof"
