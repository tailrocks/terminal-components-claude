#!/usr/bin/env bash
# Execute one capture application without putting its argv through a shell.
# The first two arguments are owned state paths used for stderr and exit status.
set -u

if (( $# < 3 )); then
  echo "capture_exec: usage: capture_exec.sh STDERR_PATH EXIT_PATH PROGRAM [ARG ...]" >&2
  exit 64
fi

stderr_path=$1
exit_path=$2
shift 2

# These paths are created by capture.sh inside a mode-0700 run directory.  Do
# not follow an attacker-supplied symlink if a state file was tampered with.
if [[ -L "$stderr_path" || -L "$exit_path" ]]; then
  echo "capture_exec: state path is a symlink" >&2
  exit 1
fi

exit_tmp=
# shellcheck disable=SC2329 # invoked by the EXIT trap below
cleanup_exit_tmp() {
  if [[ -n "$exit_tmp" && -e "$exit_tmp" && ! -L "$exit_tmp" ]]; then
    rm -f "$exit_tmp" || true
  fi
}
trap cleanup_exit_tmp EXIT

# Keep the application argv opaque to this script.  In particular, neither
# the binary path nor an argument is ever interpolated into shell source.
"$@" 2>"$stderr_path"
rc=$?

if [[ -L "$exit_path" ]]; then
  echo "capture_exec: exit state path became a symlink" >&2
  exit 1
fi
if ! exit_tmp=$(mktemp "${exit_path}.tmp.XXXXXX"); then
  echo "capture_exec: cannot create atomic exit state" >&2
  exit 1
fi
chmod 600 "$exit_tmp"
printf '%s\n' "$rc" > "$exit_tmp"
if [[ -L "$exit_path" ]]; then
  echo "capture_exec: exit state path became a symlink" >&2
  exit 1
fi
if ! mv -f "$exit_tmp" "$exit_path"; then
  echo "capture_exec: cannot publish atomic exit state" >&2
  exit 1
fi
exit_tmp=
trap - EXIT
exit "$rc"
