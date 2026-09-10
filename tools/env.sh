# Python interpreter with Pillow installed (used by tools/capture.sh for PNG
# output). Create one once at the repository root:
#   python3 -m venv .venv && .venv/bin/pip install pillow
# then `source tools/env.sh` before running the capture scripts.
export PY=${PY:-$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.." && pwd)/.venv/bin/python}
