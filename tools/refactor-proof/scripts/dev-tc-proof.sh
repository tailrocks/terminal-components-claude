#!/usr/bin/env bash
set -euo pipefail
self="$(realpath "$0")"
scripts_dir="$(cd "$(dirname "$self")" && pwd)"
crate_root="$(cd "$scripts_dir/.." && pwd)"
workspace_root="$(cd "$crate_root/../.." && pwd)"
exec "$workspace_root/target/debug/tc-proof" "$@"
