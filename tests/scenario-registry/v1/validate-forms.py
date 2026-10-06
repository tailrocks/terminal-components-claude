#!/usr/bin/env python3
"""Machine validation for scenario-registry.forms.json (FORMS group).

Checks: required fields per row, ID uniqueness + format, cross-file join
with v1/registry.json, ref_source symbols present in cited files,
provenance paths + line ranges, snapshot paths, coverage shape,
legacy_map roots/sizes/status. Exit 0 on PASS, 1 on FAIL.
Run from the repo root: python3 tests/scenario-registry/v1/validate-forms.py
"""
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
GROUP_FILE = os.path.join(ROOT, "tests/scenario-registry/v1/scenario-registry.forms.json")
REGISTRY_FILE = os.path.join(ROOT, "tests/scenario-registry/v1/registry.json")
SCHEMA_FILE = os.path.join(ROOT, "tests/scenario-registry/v1/schema.json")

ERRORS = []
CHECKS = []


def fail(msg):
    ERRORS.append(msg)


def check(name, cond, detail=""):
    CHECKS.append((name, bool(cond)))
    if not cond:
        fail(f"{name}: {detail}")


ID_RE = re.compile(r"^[A-Z0-9]+(-[A-Z0-9]+)*-[0-9]{3}$")
V1_REQUIRED = ["id", "app", "component", "part", "ref_symbols", "cand_symbols",
               "data", "state", "env", "viewport", "color", "motion", "inputs",
               "checkpoints", "assertions", "provenance", "snapshots",
               "adapters", "applicability", "results"]
GROUP_REQUIRED = V1_REQUIRED + ["ref_source", "cand_source"]
COMPONENTS = {"Form", "Wizard"}
APPS = {"showcase", "tablepro", "jackin-preview", "holla"}
COLORS = {"truecolor", "256", "16", "none", "nocolor"}
SIZE_RE = re.compile(r"^\d+x\d+$")
LINES_RE = re.compile(r"^\d+(-\d+)?(,\d+(-\d+)?)*$")


def load_json(path):
    try:
        with open(path) as f:
            return json.load(f)
    except Exception as e:  # noqa: BLE001 - report as check failure
        fail(f"parse {path}: {e}")
        return None


def main():
    doc = load_json(GROUP_FILE)
    reg = load_json(REGISTRY_FILE)
    if doc is None or reg is None:
        print("FAIL")
        for e in ERRORS:
            print(" ", e)
        return 1

    check("header.version", doc.get("version") == "1", repr(doc.get("version")))
    check("header.group", doc.get("group") == "forms", repr(doc.get("group")))
    ref_commit = doc.get("ref_commit")
    check("header.ref_commit", isinstance(ref_commit, str) and len(ref_commit) == 40, repr(ref_commit))

    scenarios = doc.get("scenarios", [])
    check("scenarios.nonempty", isinstance(scenarios, list) and len(scenarios) > 0, type(scenarios).__name__)

    reg_ids = {s["id"] for s in reg.get("scenarios", [])}
    reg_forms = {s["id"] for s in reg.get("scenarios", []) if s.get("component") in COMPONENTS}
    seen = set()
    for s in scenarios:
        sid = s.get("id", "<missing>")
        for field in GROUP_REQUIRED:
            check(f"row.{sid}.field.{field}", field in s, "missing")
        if not ID_RE.match(sid):
            check(f"row.{sid}.id.format", False, sid)
        check(f"row.{sid}.id.unique", sid not in seen, "duplicate")
        seen.add(sid)
        check(f"row.{sid}.id.in_registry", sid in reg_ids, "not in v1/registry.json")
        check(f"row.{sid}.app", s.get("app") in APPS, repr(s.get("app")))
        check(f"row.{sid}.component", s.get("component") in COMPONENTS, repr(s.get("component")))
        check(f"row.{sid}.color", s.get("color") in COLORS, repr(s.get("color")))
        check(f"row.{sid}.motion", s.get("motion") in ("playing", "paused"), repr(s.get("motion")))
        vp = s.get("viewport", {})
        check(f"row.{sid}.viewport", isinstance(vp.get("width"), int) and isinstance(vp.get("height"), int), repr(vp))
        check(f"row.{sid}.inputs", isinstance(s.get("inputs"), list) and len(s.get("inputs")) > 0, "empty")
        check(f"row.{sid}.checkpoints", isinstance(s.get("checkpoints"), list) and len(s.get("checkpoints")) > 0, "empty")
        a = s.get("assertions", {})
        ok_a = all(isinstance(a.get(k), list) for k in ("visual", "state", "action", "negative"))
        check(f"row.{sid}.assertions", ok_a, repr(sorted(a)))
        prov = s.get("provenance", {})
        check(f"row.{sid}.provenance.commit", prov.get("commit") == ref_commit, repr(prov.get("commit")))
        for src in prov.get("sources", []):
            p = src.get("path", "")
            full = os.path.join(ROOT, p)
            ok = os.path.isfile(full)
            check(f"row.{sid}.prov.exists:{p}", ok, "missing file")
            lines = src.get("lines", "")
            ok_lines = bool(LINES_RE.match(lines))
            check(f"row.{sid}.prov.lines:{p}@{lines}", ok_lines, "bad range syntax")
            if ok and ok_lines:
                with open(full, errors="replace") as f:
                    n = sum(1 for _ in f)
                for part in lines.split(","):
                    lo, _, hi = part.partition("-")
                    lo, hi = int(lo), int(hi) if hi else int(lo)
                    check(f"row.{sid}.prov.bounds:{p}@{part}", 1 <= lo <= hi <= n, f"file has {n} lines")
        for p in s.get("snapshots", {}).get("ref", []):
            check(f"row.{sid}.snapshot:{p}", os.path.isfile(os.path.join(ROOT, p)), "missing file")
        for entry in s.get("ref_source", []):
            p = entry.get("path", "")
            full = os.path.join(ROOT, p)
            ok = os.path.isfile(full)
            check(f"row.{sid}.ref_source.exists:{p}", ok, "missing file")
            if ok:
                text = open(full, errors="replace").read()
                for sym in entry.get("symbols", []):
                    hit = re.search(r"\b" + re.escape(sym) + r"\b", text) is not None
                    check(f"row.{sid}.ref_source.sym:{p}#{sym}", hit, "symbol not found")
        check(f"row.{sid}.ref_source.nonempty", len(s.get("ref_source", [])) > 0, "empty")

    missing_here = sorted(reg_forms - seen)
    check("join.registry_forms_covered", not missing_here, f"missing here: {missing_here}")

    cov = doc.get("coverage", [])
    check("coverage.count", len(cov) == 2, f"got {len(cov)}")
    cov_names = set()
    for c in cov:
        name = c.get("component", "<missing>")
        cov_names.add(name)
        for field in ("component", "rows", "handlers", "flags", "thresholds", "timers", "early_returns", "notes"):
            check(f"coverage.{name}.field.{field}", field in c, "missing")
        for rid in c.get("rows", []):
            check(f"coverage.{name}.row.{rid}", rid in seen, "unknown row id")
        check(f"coverage.{name}.rows.nonempty", len(c.get("rows", [])) > 0, "empty")
    check("coverage.components", cov_names == COMPONENTS, f"extra/missing: {sorted(cov_names ^ COMPONENTS)}")

    legacy = doc.get("legacy_map", [])
    check("legacy.nonempty", isinstance(legacy, list) and len(legacy) > 0, "empty")
    for e in legacy:
        old = e.get("old", "<missing>")
        for field in ("old", "app", "sizes", "new", "status"):
            check(f"legacy.{old}.field.{field}", field in e, "missing")
        full = os.path.join(ROOT, "snapshots", old)
        check(f"legacy.{old}.dir", os.path.isdir(full), "missing dir")
        if os.path.isdir(full):
            actual = sorted(d for d in os.listdir(full) if SIZE_RE.match(d) and os.path.isdir(os.path.join(full, d)))
            check(f"legacy.{old}.sizes", sorted(e.get("sizes", [])) == actual, f"actual {actual}")
        st = e.get("status")
        check(f"legacy.{old}.status", st in ("mapped", "pending"), repr(st))
        if st == "mapped":
            check(f"legacy.{old}.new", e.get("new") in seen, repr(e.get("new")))
        else:
            check(f"legacy.{old}.new_null", e.get("new") is None, repr(e.get("new")))
        for rid in e.get("also", []):
            check(f"legacy.{old}.also.{rid}", rid in seen, "unknown row id")

    # Optional: strict v1 schema validation when jsonschema is installed.
    try:
        import jsonschema  # noqa: F401
    except ImportError:
        print("SKIP strict schema validation (jsonschema not installed)")
    else:
        import jsonschema
        schema = load_json(SCHEMA_FILE)
        if schema is not None:
            adapted = {"version": "1", "scenarios": scenarios, "legacy_roots": []}
            try:
                jsonschema.validate(adapted, schema)
            except jsonschema.ValidationError as ve:
                fail(f"schema: {ve.message}")
            else:
                CHECKS.append(("schema.v1_strict", True))

    passed = sum(1 for _, ok in CHECKS if ok)
    print(f"checks: {passed}/{len(CHECKS)} passed")
    if ERRORS:
        print("FAIL")
        for e in ERRORS[:40]:
            print(" ", e)
        if len(ERRORS) > 40:
            print(f" ... +{len(ERRORS) - 40} more")
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
