#!/usr/bin/env python3
"""TASK-077 CHK-002: functional bundle gate behavior for namespaces.

Loads the TRACKED BUNDLE (never the repo sources) as a module in this
disposable process and asserts the exact gate contract: 'components'
with a native family is accepted, the four app namespaces stay
accepted, unknown/empty/missing native namespaces still raise
Reject("PROTOCOL"), and the synthetic family still requires exactly
'synthetic'. Fails on the current tree (components rejected); passes
once the bundle is regenerated from the fixed source. Runs from the
worktree root.
"""

from __future__ import annotations

import importlib.machinery
import importlib.util
from pathlib import Path
from typing import Any

BUNDLE = Path("tools/refactor-proof/bin/tc-proof")
EXPECTED = frozenset({"showcase", "holla", "jackin", "tablepro", "components"})
APPS = ("showcase", "holla", "jackin", "tablepro")


def load_bundle() -> Any:
    loader = importlib.machinery.SourceFileLoader(
        "tc_proof_bundle_077", str(BUNDLE)
    )
    spec = importlib.util.spec_from_loader(loader.name, loader)
    if spec is None:
        raise RuntimeError(f"cannot build spec for {BUNDLE}")
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    return module


def main() -> int:
    failures: list[str] = []
    try:
        bundle = load_bundle()
    except Exception as error:
        print(f"FAIL: cannot load tracked bundle as module: {error!r}")
        return 1
    print("PASS: tracked bundle loads as a module in a disposable process")
    Reject = bundle.Reject
    gate = bundle.validate_oracle_namespace

    allowlist = bundle.NATIVE_ORACLE_NAMESPACES
    if allowlist != EXPECTED:
        failures.append(
            f"bundled allowlist is {sorted(allowlist)}, want {sorted(EXPECTED)}"
        )
    else:
        print(f"PASS: bundled allowlist is exactly {sorted(EXPECTED)}")

    def expect_accept(namespace: Any, family: Any) -> None:
        try:
            result = gate(namespace, family)
        except Exception as error:
            failures.append(
                f"gate rejected ({namespace!r}, {family!r}): "
                f"{type(error).__name__}({getattr(error, 'category', '?')})"
            )
        else:
            if result is not None:
                failures.append(
                    f"gate returned {result!r} for ({namespace!r}, {family!r}), want None"
                )
            else:
                print(f"PASS: gate accepts ({namespace!r}, {family!r})")

    def expect_reject(namespace: Any, family: Any) -> None:
        try:
            result = gate(namespace, family)
        except Exception as error:
            if (
                type(error).__name__ != "Reject"
                or getattr(error, "category", None) != "PROTOCOL"
            ):
                failures.append(
                    f"gate raised {type(error).__name__}("
                    f"{getattr(error, 'category', '?')}) for "
                    f"({namespace!r}, {family!r}), want Reject(PROTOCOL)"
                )
            else:
                print(f"PASS: gate rejects ({namespace!r}, {family!r}) with PROTOCOL")
        else:
            failures.append(
                f"gate accepted ({namespace!r}, {family!r}) -> {result!r}, "
                "want Reject(PROTOCOL)"
            )

    expect_accept("components", "native")
    for app in APPS:
        expect_accept(app, "native")
    expect_reject("bogus", "native")
    expect_reject("", "native")
    expect_reject(None, "native")
    expect_accept("synthetic", "synthetic")
    expect_reject("components", "synthetic")
    expect_reject("bogus", "synthetic")

    for failure in failures:
        print(f"FAIL: {failure}")
    if failures:
        return 1
    print("gate-behavior probe: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
