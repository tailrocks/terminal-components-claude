#!/usr/bin/env bash
# Build the native proof comparator required by verifier-subagent checks.
# This is a host-local build step; taskfmt remains verifier-only.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
TC_TARGET_DIR="${TC_PROOF_TARGET_DIR:-}"
CARGO_TARGET="${CARGO_TARGET_DIR:-}"

# The verifier owns the target directory. An empty CARGO_TARGET_DIR is not a
# target selection; with no explicit override, inspect the repository default
# and reject it if it is the shared-cache symlink used by this checkout.
TARGET_DIR="$({
	python3 - "$ROOT" "$TC_TARGET_DIR" "$CARGO_TARGET" <<'PY'
import os
import stat
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
tc_raw, cargo_raw = sys.argv[2:]


def check_parent_components(path: Path, name: str) -> None:
    allowed_system_symlinks = {
        Path("/etc"): Path("/private/etc"),
        Path("/home"): Path("/System/Volumes/Data/home"),
        Path("/tmp"): Path("/private/tmp"),
        Path("/var"): Path("/private/var"),
    }
    current = Path(path.anchor)
    for component in path.parts[1:-1]:
        current /= component
        try:
            metadata = os.lstat(current)
        except FileNotFoundError:
            break
        except OSError as error:
            raise SystemExit(
                f"{name} parent path component is unreadable: {current}: {error}"
            ) from error
        if stat.S_ISLNK(metadata.st_mode):
            if current.resolve() != allowed_system_symlinks.get(current):
                raise SystemExit(
                    f"{name} parent path component must not be a symlink: {current}"
                )
            continue
        if not stat.S_ISDIR(metadata.st_mode):
            raise SystemExit(
                f"{name} parent path component is not a directory: {current}"
            )


def checked_path(raw: str, name: str) -> Path:
    path = Path(raw)
    if not path.is_absolute():
        raise SystemExit(f"{name} must be an absolute path: {raw or '<empty>'}")
    check_parent_components(path, name)
    if path.is_symlink():
        raise SystemExit(f"{name} must not be a symlink: {path}")
    return path

if tc_raw and cargo_raw:
    tc_path = checked_path(tc_raw, "TC_PROOF_TARGET_DIR")
    cargo_path = checked_path(cargo_raw, "CARGO_TARGET_DIR")
    if tc_path.resolve() != cargo_path.resolve():
        raise SystemExit(
            "TC_PROOF_TARGET_DIR and CARGO_TARGET_DIR select ambiguous target roots"
        )
    selected = tc_path
elif tc_raw:
    selected = checked_path(tc_raw, "TC_PROOF_TARGET_DIR")
elif cargo_raw:
    selected = checked_path(cargo_raw, "CARGO_TARGET_DIR")
else:
    selected = root / "target"
    check_parent_components(selected, "proof target directory")

if selected.is_symlink():
    raise SystemExit(f"proof target directory must not be a symlink: {selected}")
if not selected.exists():
    raise SystemExit(f"proof target directory is missing: {selected}")
if not selected.is_dir():
    raise SystemExit(f"proof target path is not a directory: {selected}")

resolved = selected.resolve()
if resolved == root or root in resolved.parents:
    raise SystemExit("proof target directory must be external to the worktree")
print(resolved)
PY
})"

cd "$ROOT"
[[ -z "$(git status --porcelain)" ]] ||
	{
		echo "campaign-build-proof: worktree must be clean at the verifier commit" >&2
		exit 1
	}

COMMIT="$(git rev-parse HEAD)"
TREE="$(git rev-parse 'HEAD^{tree}')"
DEBUG_DIR="$TARGET_DIR/debug"
BINARY="$DEBUG_DIR/tc-proof"
RECEIPT="$DEBUG_DIR/tc-proof.build.json"

validate_receipt() {
	python3 - "$RECEIPT" "$ROOT" "$TARGET_DIR" "$COMMIT" "$TREE" "$BINARY" <<'PY'
import hashlib
import json
import os
import stat
import sys
from pathlib import Path

receipt, root, target, commit, tree, binary = sys.argv[1:]
try:
    with Path(receipt).open(encoding="utf-8") as stream:
        value = json.load(stream)
except (OSError, ValueError) as error:
    raise SystemExit(f"invalid proof build receipt: {error}") from error

expected_root = str(Path(root).resolve())
expected_target = str(Path(target).resolve())
expected_binary = str(Path(binary).resolve())


def require_single_link_file(path: Path, label: str) -> None:
    try:
        metadata = os.lstat(path)
    except OSError as error:
        raise SystemExit(f"{label} is unreadable: {error}") from error
    if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
        raise SystemExit(f"{label} is not a regular single-link file")


require_single_link_file(Path(binary), "proof binary")
require_single_link_file(Path(receipt), "proof build receipt")
if value.get("schema") != "tc-proof-native-build/v1":
    raise SystemExit("stale proof receipt schema")
if value.get("worktree") != expected_root:
    raise SystemExit("stale proof receipt worktree binding")
if value.get("target_dir") != expected_target or value.get("cargo_target_dir") != expected_target:
    raise SystemExit("stale proof receipt target binding")
if value.get("commit") != commit or value.get("tree") != tree:
    raise SystemExit("stale proof receipt source binding")
if value.get("binary") != expected_binary:
    raise SystemExit("stale proof receipt binary binding")
try:
    actual = hashlib.sha256(Path(binary).read_bytes()).hexdigest()
except OSError as error:
    raise SystemExit(f"proof binary is unreadable: {error}") from error
if value.get("binary_sha256") != actual:
    raise SystemExit("stale proof binary hash")
PY
}

if [[ -L "$DEBUG_DIR" ]]; then
	echo "campaign-build-proof: debug target directory must not be a symlink: $DEBUG_DIR" >&2
	exit 1
fi
if [[ -L "$BINARY" || -L "$RECEIPT" || -e "$BINARY" || -e "$RECEIPT" ]]; then
	[[ -f "$BINARY" && ! -L "$BINARY" && -f "$RECEIPT" && ! -L "$RECEIPT" ]] ||
		{
			echo "campaign-build-proof: stale or linked proof target artifacts" >&2
			exit 1
		}
	validate_receipt
fi

CARGO_TARGET_DIR="$TARGET_DIR" cargo build --locked --offline -p refactor-proof --bin tc-proof

[[ -d "$DEBUG_DIR" && ! -L "$DEBUG_DIR" ]] ||
	{
		echo "campaign-build-proof: native debug target directory missing or linked: $DEBUG_DIR" >&2
		exit 1
	}
[[ -f "$BINARY" && ! -L "$BINARY" && -x "$BINARY" ]] ||
	{
		echo "campaign-build-proof: native comparator missing: $BINARY" >&2
		exit 1
	}
[[ ! -L "$RECEIPT" ]] ||
	{
		echo "campaign-build-proof: native build receipt must not be a symlink: $RECEIPT" >&2
		exit 1
	}

SHA256="$(shasum -a 256 "$BINARY" | awk '{print $1}')"
python3 - "$RECEIPT" "$ROOT" "$TARGET_DIR" "$COMMIT" "$TREE" "$BINARY" "$SHA256" <<'PY'
import json
import sys
from pathlib import Path

receipt, root, target, commit, tree, binary, sha256 = sys.argv[1:]
Path(receipt).write_text(json.dumps({
    "schema": "tc-proof-native-build/v1",
    "worktree": str(Path(root).resolve()),
    "target_dir": str(Path(target).resolve()),
    "cargo_target_dir": str(Path(target).resolve()),
    "commit": commit,
    "tree": tree,
    "binary": str(Path(binary).resolve()),
    "binary_sha256": sha256,
    "command": ["cargo", "build", "--locked", "--offline", "-p", "refactor-proof", "--bin", "tc-proof"],
}, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

validate_receipt

echo "native tc-proof comparator: $BINARY"
echo "native tc-proof build receipt: $RECEIPT"
