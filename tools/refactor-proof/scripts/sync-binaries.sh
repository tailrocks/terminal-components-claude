#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
workspace="$(cd "$root/../.." && pwd)"
profile="${PROFILE:-debug}"
target="$workspace/target/$profile"

for name in tc-proof tc-proof-host; do
  built="$target/$name"
  dest="$root/bin/$name"
  if [[ ! -f "$built" ]]; then
    echo "sync-binaries: missing $built (run: cargo build -p refactor-proof)" >&2
    exit 1
  fi
  cp "$built" "$dest"
  chmod 755 "$dest"
done

echo "sync-binaries: installed $(shasum -a 256 "$root/bin/tc-proof-host" | awk '{print $1}') -> bin/tc-proof-host"
