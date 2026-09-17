#!/usr/bin/env bash
# Retired fail-closed compatibility shim.
#
# bin/tc-proof is the host-local Python dispatcher used by current task
# contracts. Overwriting it with the Rust comparator silently removes
# preflight/capture/accounting/architecture operations. Build and test the
# comparator through cargo nextest; never install it over the dispatcher.
set -euo pipefail

echo "sync-binaries: retired; refusing to overwrite bin/tc-proof" >&2
exit 78
