#!/usr/bin/env python3
"""Retired runner-only bundler.

The runner-only bundle could overwrite the canonical dispatcher with a binary
that lacked accounting and architecture operations. Keep this path as an
explicit fail-closed tombstone; use architecture/rebundle.py only when a
reviewed native build needs a bundled artifact.
"""

raise SystemExit(
    "runner/bundle.py is retired; use the native verifier workflow and the "
    "canonical architecture/rebundle.py only"
)
