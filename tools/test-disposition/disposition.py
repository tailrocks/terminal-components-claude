#!/usr/bin/env python3
"""Test-disposition engine (TASK-008 product `test-disposition`).

Reads the accepted TASK-007 inventory (conflicts/listing) plus this task's
hand-reviewed verdicts, stage map and span-patch manifest, and:

- `decide`: cross-checks verdict coverage against the reconciled inventory
  and emits the reviewable disposition record;
- `apply`: applies the span-patch manifest to a tree with strict
  old-byte/hash/offset checks (mechanical reproduction, never approval);
- `verify`: proves a candidate tree equals archive originals plus exactly
  the manifest spans, with nothing else changed in scope.

Stdlib only. Parsing and byte checks never substitute for execution:
failing/succeeding is decided by `cargo nextest`, never by this tool.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path

SCHEMA = 1
ORACLE_TAG_COMMIT = "4a79c0a2d40fca46fc406b77157ce3b3f12ec16b"


class Invalid(Exception):
    pass


def require(cond, message):
    if not cond:
        raise Invalid(message)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def encoded(value) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def load_json(path: Path):
    try:
        return json.loads(path.read_bytes())
    except FileNotFoundError:
        raise Invalid(f"missing required input: {path}")
    except json.JSONDecodeError as exc:
        raise Invalid(f"invalid JSON in {path}: {exc}")


def identity_key(identity: dict) -> tuple:
    for field in ("package", "kind", "target", "name"):
        require(field in identity, f"identity missing {field!r}: {identity!r}")
    return (
        str(identity["package"]),
        str(identity["kind"]),
        str(identity["target"]),
        str(identity["name"]),
    )


def conflict_key(entry: dict) -> tuple:
    return identity_key(
        {
            "package": entry["package"],
            "kind": entry["kind"],
            "target": entry["target"],
            "name": entry["identity"],
        }
    )


# --------------------------------------------------------------------------
# decide
# --------------------------------------------------------------------------


def listing_index(listing: dict) -> dict:
    """(package, kind, target) -> list of per-row identity sets."""
    require(isinstance(listing.get("targets"), list), "listing.targets must be a list")
    index: dict = {}
    for row in listing["targets"]:
        key = (str(row["package"]), str(row["kind"]), str(row["target"]))
        listed = row.get("listed", [])
        require(isinstance(listed, list), f"listing row listed must be a list: {key}")
        names = set()
        for name in listed:
            require(isinstance(name, str), f"non-string listed identity in {key}")
            require(name not in names, f"duplicate listed identity {name} in {key}")
            names.add(name)
        index.setdefault(key, []).append(names)
    return index


def cmd_decide(args) -> int:
    root = Path(args.root)
    verdicts = load_json(Path(args.verdicts))
    stage = load_json(Path(args.stage))
    manifest = load_json(Path(args.manifest))
    conflicts = load_json(Path(args.conflicts))
    listing = load_json(Path(args.listing))

    for label, doc in (
        ("verdicts", verdicts),
        ("stage", stage),
        ("manifest", manifest),
    ):
        require(doc.get("schema") == SCHEMA, f"{label}.schema must be {SCHEMA}")
    base = verdicts.get("base_commit")
    require(isinstance(base, str) and len(base) == 40, "verdicts.base_commit must be a commit sha")
    for label, doc in (("stage", stage), ("manifest", manifest)):
        require(doc.get("base_commit") == base, f"{label}.base_commit must match verdicts")

    # Every 007 conflict needs exactly one hand decision.
    conflict_keys = set()
    for entry in conflicts.get("conflicts", []):
        key = conflict_key(entry)
        require(key not in conflict_keys, f"duplicate conflict entry: {key}")
        conflict_keys.add(key)

    decisions = verdicts.get("decisions", [])
    require(isinstance(decisions, list) and decisions, "verdicts.decisions must be non-empty")
    seen_ids: set = set()
    seen_keys: set = set()
    by_id: dict = {}
    for decision in decisions:
        vid = decision.get("id")
        require(isinstance(vid, str) and vid, "decision needs a string id")
        require(vid not in seen_ids, f"duplicate decision id {vid}")
        seen_ids.add(vid)
        key = identity_key(decision["identity"])
        require(key not in seen_keys, f"duplicate decision identity: {key}")
        seen_keys.add(key)
        require(
            decision.get("verdict") in ("preserve", "replace", "relocate", "retire"),
            f"{vid}: verdict must be preserve/replace/relocate/retire",
        )
        require(
            isinstance(decision.get("evidence"), list) and decision["evidence"],
            f"{vid}: every disposition needs non-empty evidence",
        )
        if decision["verdict"] == "relocate":
            require(
                isinstance(decision.get("replacement_identity"), dict),
                f"{vid}: relocate needs replacement_identity",
            )
            identity_key(decision["replacement_identity"])
        elif "replacement_identity" in decision:
            raise Invalid(f"{vid}: replacement_identity only valid for relocate")
        spans = decision.get("spans", [])
        require(isinstance(spans, list), f"{vid}: spans must be a list")
        span_keys = set()
        for span in spans:
            skey = span.get("key")
            require(isinstance(skey, str) and skey, f"{vid}: span needs a string key")
            require(skey not in span_keys, f"{vid}: duplicate span key {skey}")
            span_keys.add(skey)
            require(
                span.get("disposition")
                in ("preserve", "replace", "relocate", "retire", "retire-procedure"),
                f"{vid}/{skey}: bad span disposition",
            )
            require(
                isinstance(span.get("evidence"), list) and span["evidence"],
                f"{vid}/{skey}: every span needs non-empty evidence",
            )
        decision["_span_keys"] = span_keys
        by_id[vid] = decision

    missing = sorted(conflict_keys - seen_keys)
    require(not missing, f"conflicts without a hand decision: {missing[:5]}")

    # Every manifest patch references a real verdict; every non-preserve
    # verdict span is implemented by a manifest patch.
    patches = manifest.get("patches", [])
    require(isinstance(patches, list) and patches, "manifest.patches must be non-empty")
    implemented: dict = {}
    for patch in patches:
        pid = patch.get("id")
        require(isinstance(pid, str) and pid, "patch needs a string id")
        vref = patch.get("verdict_ref")
        require(vref in by_id, f"{pid}: unknown verdict_ref {vref!r}")
        keys = patch.get("span_keys", [])
        require(isinstance(keys, list) and keys, f"{pid}: span_keys must be non-empty")
        for skey in keys:
            require(
                skey in by_id[vref]["_span_keys"],
                f"{pid}: span key {skey!r} not in verdict {vref}",
            )
            require(
                (vref, skey) not in implemented,
                f"span {vref}/{skey} implemented twice",
            )
            implemented[(vref, skey)] = pid
    for vid, decision in by_id.items():
        for span in decision["spans"]:
            if span["disposition"] == "preserve":
                continue
            require(
                (vid, span["key"]) in implemented,
                f"non-preserve span {vid}/{span['key']} has no manifest patch",
            )

    # Stage map: future-owned identities resolve to exact closing tasks.
    entries = stage.get("entries", [])
    require(isinstance(entries, list) and entries, "stage.entries must be non-empty")
    for entry in entries:
        require(
            isinstance(entry.get("owner"), str) and entry["owner"].startswith("TASK-"),
            f"stage entry needs an exact TASK- owner: {entry!r}",
        )
        require(
            isinstance(entry.get("evidence"), list) and entry["evidence"],
            f"stage entry needs non-empty evidence: {entry!r}",
        )

    # Roll up the full listing: every listed identity is either hand-decided
    # or bulk-preserved by rule. Nothing is silently omitted.
    index = listing_index(listing)
    total = 0
    preserved = 0
    rollups = []
    for target in listing["targets"]:
        tkey = (str(target["package"]), str(target["kind"]), str(target["target"]))
        decided_here = sorted(
            vid
            for vid, decision in by_id.items()
            if identity_key(decision["identity"])[:3] == tkey
            and identity_key(decision["identity"])[3] in set(target.get("listed", []))
        )
        # A hand decision whose identity is absent from every listing row is
        # either a relocation source (renamed: old name still listed) or an error.
        listed = target.get("listed", [])
        total += len(listed)
        preserved += len(listed) - len(
            [n for n in listed if (tkey + (n,)) in seen_keys]
        )
        rollups.append(
            {
                "profile": target.get("profile"),
                "package": tkey[0],
                "kind": tkey[1],
                "target": tkey[2],
                "listed": len(listed),
                "decided": decided_here,
            }
        )
    # File-level reviews with a listing target must name real identities.
    for review in verdicts.get("reviews", []):
        rid = review.get("id", "?")
        require(
            isinstance(review.get("evidence"), list) and review["evidence"],
            f"review {rid}: every review needs non-empty evidence",
        )
        target = review.get("target")
        if target is None:
            continue
        tkey = (str(target["package"]), str(target["kind"]), str(target["target"]))
        prefix = str(target.get("prefix", ""))
        rows = index.get(tkey, [])
        require(rows, f"review {rid}: target {tkey} absent from listing")
        for test in review.get("tests", []):
            full = prefix + test["name"]
            require(
                any(full in names for names in rows),
                f"review {rid}: {full} absent from the reconciled listing",
            )

    # Every decision identity must occur in at least one listing row.
    for vid, decision in by_id.items():
        key = identity_key(decision["identity"])
        rows = index.get(key[:3], [])
        require(
            any(key[3] in names for names in rows),
            f"decision {vid}: identity {key} absent from the reconciled listing",
        )
        if decision["verdict"] == "relocate":
            new_key = identity_key(decision["replacement_identity"])
            rows = index.get(new_key[:3], [])
            require(
                not any(new_key[3] in names for names in rows),
                f"decision {vid}: replacement {new_key} already listed (not a rename)",
            )

    relocations = [
        {
            "id": vid,
            "old": decision["identity"],
            "new": decision["replacement_identity"],
        }
        for vid, decision in by_id.items()
        if decision["verdict"] == "relocate"
    ]

    out = {
        "schema": SCHEMA,
        "producer": "TASK-008",
        "base_commit": base,
        "oracle_tag_commit": ORACLE_TAG_COMMIT,
        "inputs": {
            name: digest(Path(path).read_bytes())
            for name, path in (
                ("verdicts", args.verdicts),
                ("stage", args.stage),
                ("manifest", args.manifest),
                ("conflicts", args.conflicts),
                ("listing", args.listing),
            )
        },
        "counts": {
            "listing_identities": total,
            "conflicts": len(conflict_keys),
            "decisions": len(decisions),
            "bulk_preserved": preserved,
            "relocations": len(relocations),
            "manifest_patches": len(patches),
        },
        "rules": verdicts.get("rules", []),
        "decisions": [
            {k: v for k, v in d.items() if not k.startswith("_")} for d in decisions
        ],
        "relocations": relocations,
        "rollups": rollups,
    }
    output = Path(args.output)
    require(not output.exists(), f"refusing to overwrite existing output: {output}")
    output.write_text(json.dumps(out, indent=2, sort_keys=True) + "\n")

    if args.expand_tsv:
        tsv = Path(args.expand_tsv)
        require(not tsv.exists(), f"refusing to overwrite existing output: {tsv}")
        lines = ["profile\tpackage\tkind\ttarget\tidentity\tverdict\tdecision\n"]
        for target in listing["targets"]:
            tkey = (str(target["package"]), str(target["kind"]), str(target["target"]))
            for name in target.get("listed", []):
                key = tkey + (name,)
                vid = next(
                    (
                        v
                        for v, d in by_id.items()
                        if identity_key(d["identity"]) == key
                    ),
                    "",
                )
                verdict = by_id[vid]["verdict"] if vid else "preserve"
                lines.append(
                    "\t".join(
                        [
                            str(target.get("profile", "")),
                            tkey[0],
                            tkey[1],
                            tkey[2],
                            name,
                            verdict,
                            vid,
                        ]
                    )
                    + "\n"
                )
        tsv.write_text("".join(lines))
    return 0


# --------------------------------------------------------------------------
# apply / verify
# --------------------------------------------------------------------------


def check_patch(patch: dict, files: dict) -> tuple:
    pid = patch.get("id", "?")
    path = patch.get("path")
    require(isinstance(path, str) and path, f"{pid}: patch needs a path")
    require(path in files, f"{pid}: path {path!r} not under --root")
    old = patch.get("old")
    new = patch.get("new")
    require(isinstance(old, str) and old, f"{pid}: old bytes must be non-empty text")
    require(isinstance(new, str) and new, f"{pid}: new bytes must be non-empty text")
    require(old != new, f"{pid}: old and new bytes are identical")
    content = files[path]
    first = content.find(old)
    require(first >= 0, f"{pid}: old bytes absent in {path} (already applied or drifted)")
    require(
        content.find(old, first + 1) < 0,
        f"{pid}: old bytes occur more than once in {path} (ambiguous span)",
    )
    require(
        digest(old.encode()) == patch.get("old_sha256"),
        f"{pid}: old_sha256 mismatch",
    )
    require(
        digest(new.encode()) == patch.get("new_sha256"),
        f"{pid}: new_sha256 mismatch",
    )
    if "old_start" in patch:
        require(
            patch["old_start"] == first,
            f"{pid}: old_start {patch['old_start']} != actual {first}",
        )
    return path, first, old, new


def cmd_apply(args) -> int:
    root = Path(args.root)
    manifest = load_json(Path(args.manifest))
    require(manifest.get("schema") == SCHEMA, "manifest.schema mismatch")
    patches = manifest.get("patches", [])
    require(isinstance(patches, list) and patches, "manifest.patches must be non-empty")
    files: dict = {}
    for patch in patches:
        path = patch.get("path")
        if path not in files:
            full = root / path
            require(full.is_file(), f"patch path missing under --root: {path}")
            files[path] = full.read_text()
    # Validate every span before changing any file.
    planned = [check_patch(patch, files) for patch in patches]
    # Non-overlapping spans per file; apply back to front.
    by_path: dict = {}
    for path, start, old, new in planned:
        by_path.setdefault(path, []).append((start, old, new))
    for path, spans in by_path.items():
        spans.sort()
        for (s1, o1, _), (s2, o2, _) in zip(spans, spans[1:]):
            require(s1 + len(o1) <= s2, f"overlapping spans in {path}")
        content = files[path]
        for start, old, new in reversed(spans):
            content = content[:start] + new + content[start + len(old):]
        (root / path).write_text(content)
    return 0


def git_changed_files(root: Path, base: str) -> set:
    proc = subprocess.run(
        ["git", "diff", "--name-only", f"{base}", "--"],
        cwd=root,
        capture_output=True,
        text=True,
    )
    require(proc.returncode == 0, f"git diff failed: {proc.stderr[:300]}")
    staged = subprocess.run(
        ["git", "diff", "--name-only", "--cached", f"{base}", "--"],
        cwd=root,
        capture_output=True,
        text=True,
    )
    require(staged.returncode == 0, f"git diff --cached failed: {staged.stderr[:300]}")
    out = set(proc.stdout.split()) | set(staged.stdout.split())
    # Untracked in-scope files are candidate additions: list them too.
    untracked = subprocess.run(
        ["git", "ls-files", "--others", "--exclude-standard"],
        cwd=root,
        capture_output=True,
        text=True,
    )
    require(untracked.returncode == 0, f"git ls-files failed: {untracked.stderr[:300]}")
    return out | set(untracked.stdout.split())


def cmd_verify(args) -> int:
    root = Path(args.root)
    manifest = load_json(Path(args.manifest))
    require(manifest.get("schema") == SCHEMA, "manifest.schema mismatch")
    base = manifest.get("base_commit")
    require(isinstance(base, str) and len(base) == 40, "manifest.base_commit invalid")
    archive = load_json(Path(args.archive) / "manifest.json")
    require(archive.get("base_commit") == base, "archive base_commit != manifest base")
    archived = {entry["path"]: entry for entry in archive.get("files", [])}

    patches = manifest.get("patches", [])
    by_path: dict = {}
    for patch in patches:
        by_path.setdefault(patch["path"], []).append(patch)

    for path, spans in by_path.items():
        require(path in archived, f"{path}: changed file lacks archive originals")
        entry = archived[path]
        original = (Path(args.archive) / path).read_bytes()
        require(
            digest(original) == entry["sha256"],
            f"{path}: archive bytes != recorded sha256 (archive mutated?)",
        )
        try:
            current = (root / path).read_bytes()
        except FileNotFoundError:
            raise Invalid(f"{path}: missing under --root")
        # Re-derive the candidate from archive originals plus exactly the
        # manifest spans; anything else is an outside-span edit.
        text = original.decode()
        planned = []
        for patch in spans:
            _path, start, old, new = check_patch(patch, {path: text})
            planned.append((start, old, new))
        planned.sort()
        for (s1, o1, _), (s2, o2, _) in zip(planned, planned[1:]):
            require(s1 + len(o1) <= s2, f"overlapping spans in {path}")
        for start, old, new in reversed(planned):
            text = text[:start] + new + text[start + len(old):]
        require(
            text.encode() == current,
            f"{path}: candidate differs from archive+manifest (outside-span edit?)",
        )

    changed = git_changed_files(root, base)
    # The candidate may only touch manifest files plus this task's own
    # product directories.
    allowed_prefixes = ("tools/test-disposition/", "archives/test-authority/")
    allowed = set(by_path) | {
        c for c in changed if c.startswith(allowed_prefixes)
    }
    unexpected = sorted(set(changed) - allowed)
    require(not unexpected, f"files changed outside the manifest scope: {unexpected[:10]}")
    return 0


# --------------------------------------------------------------------------
# main
# --------------------------------------------------------------------------


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="TASK-008 test-disposition engine")
    sub = parser.add_subparsers(dest="command", required=True)

    decide = sub.add_parser("decide", help="emit the disposition record")
    decide.add_argument("--root", required=True)
    decide.add_argument("--verdicts", required=True)
    decide.add_argument("--stage", required=True)
    decide.add_argument("--manifest", required=True)
    decide.add_argument("--conflicts", required=True)
    decide.add_argument("--listing", required=True)
    decide.add_argument("--output", required=True)
    decide.add_argument("--expand-tsv", default=None)

    apply = sub.add_parser("apply", help="apply the span-patch manifest")
    apply.add_argument("--root", required=True)
    apply.add_argument("--manifest", required=True)

    verify = sub.add_parser("verify", help="verify candidate == archive + manifest")
    verify.add_argument("--root", required=True)
    verify.add_argument("--manifest", required=True)
    verify.add_argument("--archive", required=True)

    args = parser.parse_args(argv)
    try:
        if args.command == "decide":
            return cmd_decide(args)
        if args.command == "apply":
            return cmd_apply(args)
        if args.command == "verify":
            return cmd_verify(args)
    except Invalid as exc:
        print(f"test-disposition:rejected: {exc}", file=sys.stderr)
        return 1
    raise AssertionError("unreachable")


if __name__ == "__main__":
    sys.exit(main())
