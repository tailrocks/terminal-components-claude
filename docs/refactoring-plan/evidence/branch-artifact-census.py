#!/usr/bin/env python3
"""Decode every changed artifact blob; inventory is not visual parity approval."""
from __future__ import annotations
import argparse
from collections import Counter
import csv
import difflib
import hashlib
from html.parser import HTMLParser
import io
import json
from pathlib import Path
import re
import subprocess
from PIL import Image

MAIN = "7b27732a8c3c131760ec3438f641cb3c11343a42"
HOLLA = "2e2401393c47360741ebd321679de08982dca50a"

def sha(data):
    return hashlib.sha256(data).hexdigest()

class PreText(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.depth = 0
        self.parts = []
    def handle_starttag(self, tag, attrs):
        if tag == "pre":
            self.depth += 1
    def handle_endtag(self, tag):
        if tag == "pre":
            self.depth -= 1
    def handle_data(self, data):
        if self.depth:
            self.parts.append(data)

def decode(path, data):
    result = {"bytes": len(data), "sha256": sha(data)}
    name = Path(path).name
    kind = name.rsplit(".", 1)[-1]
    result["format"] = kind
    if data.startswith(b"\x89PNG\r\n\x1a\n"):
        with Image.open(io.BytesIO(data)) as im:
            im.verify()
        with Image.open(io.BytesIO(data)) as im:
            im.load()
            result.update(format="png", width=im.width, height=im.height,
                          mode=im.mode, rgba_sha256=sha(im.convert("RGBA").tobytes()))
        return result
    text = data.decode("utf-8")
    result["lines"] = len(text.splitlines())
    if kind == "json":
        value = json.loads(text)
        result["json_root"] = type(value).__name__
        result["json_canonical_sha256"] = sha(json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode())
        counts = Counter()
        def visit(node):
            counts[type(node).__name__] += 1
            if isinstance(node, dict):
                for value in node.values():
                    visit(value)
            elif isinstance(node, list):
                for value in node:
                    visit(value)
        visit(value)
        result["json_nodes"] = dict(counts)
        records = value if isinstance(value, list) else [value]
        provenance = []
        for record in records:
            if not isinstance(record, dict):
                continue
            source = record.get("source", {})
            if not isinstance(source, dict):
                source = {}
            provenance.append({key: value for key, value in {
                "capture": record.get("capture"), "app": record.get("app"),
                "git": source.get("git", record.get("git_commit", record.get("git"))),
                "dirty": source.get("dirty", record.get("dirty")),
                "review": record.get("review"), "geometry": record.get("geometry", record.get("dimensions")),
            }.items() if value is not None})
        result["provenance"] = provenance
    elif kind == "html":
        parser = PreText()
        parser.feed(text)
        parser.close()
        result["pre_text_sha256"] = sha("".join(parser.parts).encode())
        result["pre_chars"] = sum(map(len, parser.parts))
    elif kind == "ansi":
        # Inventory SGR-bearing pane dumps; do not pretend this is a VT parser.
        controls = Counter(re.findall(r"\x1b\[[0-?]*[ -/]*[@-~]", text))
        result["csi_total"] = sum(controls.values())
        result["csi_finals"] = dict(Counter({final: sum(n for code,n in controls.items() if code.endswith(final)) for final in sorted({code[-1] for code in controls})}))
        stripped = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", text)
        result["non_csi_escape_count"] = stripped.count("\x1b")
        result["csi_stripped_sha256"] = sha(stripped.encode())
    elif kind == "cursor":
        result["cursor_tokens"] = text.split()
        result["integer_triple"] = bool(re.fullmatch(r"\s*\d+\s+\d+\s+[01]\s*", text))
    elif kind == "tsv":
        rows = list(csv.reader(io.StringIO(text), delimiter="\t"))
        result["records"] = len(rows)
        result["column_counts"] = dict(Counter(len(row) for row in rows))
        result["duplicate_records"] = len(rows) - len({tuple(row) for row in rows})
    elif kind == "log":
        result["panic_lines"] = sum("panicked" in line or "index out of bounds" in line for line in text.splitlines())
    return result


def relationships(records, blobs):
    """Check every generated relationship; never execute historical recipes."""
    lookup = {(r["side"], r["path"]): r for r in records}
    issues, counts = [], Counter()
    def data(side, path):
        row = lookup.get((side, path))
        if row is None:
            issues.append({"kind": "missing-changed-artifact", "side": side, "path": path})
            return None
        return blobs[row["git_blob"]]
    def check(condition, kind, side, path):
        counts[kind] += 1
        if not condition:
            issues.append({"kind": kind, "side": side, "path": path})
    def trimmed_lines(text):
        # Diagnostic classification only, never a cell-parity comparator.
        return [line.rstrip(" ") for line in text.splitlines()]
    for row in records:
        side, path = row["side"], row["path"]
        if row["format"] == "png":
            base = path[:-4] if path.endswith(".png") else path.rsplit("/", 1)[0]
            separator = "." if path.endswith(".png") else "/"
            companions = {kind: lookup.get((side, base + separator + kind)) for kind in ("ansi", "txt", "html", "cursor")}
            check(all(companions.values()), "complete-five-format-set", side, path)
            if all(companions.values()):
                check(companions["ansi"]["csi_stripped_sha256"] == companions["txt"]["sha256"], "ansi-stripped-versus-text", side, path)
                # HTML serializers may add a final newline; retain strict byte
                # comparison here rather than silently normalizing discrepancies.
                check(companions["html"]["pre_text_sha256"] == companions["txt"]["sha256"], "html-pre-versus-text", side, path)
                ansi_text = blobs[companions["ansi"]["git_blob"]].decode()
                plain = blobs[companions["txt"]["git_blob"]].decode()
                stripped = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", ansi_text)
                parser = PreText()
                parser.feed(blobs[companions["html"]["git_blob"]].decode())
                parser.close()
                check(trimmed_lines(stripped) == trimmed_lines(plain), "ansi-text-nonpadding-content", side, path)
                check(trimmed_lines("".join(parser.parts)) == trimmed_lines(plain), "html-text-nonpadding-content", side, path)
        if path.endswith(".manifest.json"):
            value = json.loads(blobs[row["git_blob"]])
            ansi = data(side, path.removesuffix(".manifest.json") + ".ansi")
            check(ansi is not None and sha(ansi) == value.get("ansi_sha256"), "manifest-ansi-digest", side, path)
        if path.endswith(".png.fidelity.json"):
            value = json.loads(blobs[row["git_blob"]])
            png = lookup.get((side, path.removesuffix(".fidelity.json")))
            cell = value.get("cell", [])
            # Exact Holla tools/ansi2png.py constant PAD=12 on each edge.
            check(png is not None and len(cell) == 2 and png["width"] == value["cols"] * cell[0] + 24 and png["height"] == value["rows"] * cell[1] + 24, "fidelity-raster-dimensions", side, path)
            counts["fidelity-approximate"] += bool(value.get("approximate"))
            for kind in ("missing", "unshaped", "clipped", "controls"):
                counts["fidelity-" + kind] += len(value.get(kind, []))
    manifest = list(csv.reader(io.StringIO(data("main", "baseline/before/manifest.tsv").decode()), delimiter="\t"))
    recipes = list(csv.DictReader(io.StringIO(data("main", "parity/recipes.tsv").decode()), delimiter="\t"))
    by_id = {r[0]: r for r in manifest}
    check(len(by_id) == len(manifest), "baseline-unique-ids", "main", "baseline/before/manifest.tsv")
    check(len({r["recipe_id"] for r in recipes}) == len(recipes) and {r["recipe_id"] for r in recipes} == set(by_id), "recipe-exact-membership", "main", "parity/recipes.tsv")
    for row in recipes:
        old = by_id.get(row["recipe_id"])
        check(old is not None and [row["viewport"], row["historical_command"], row["steps"]] == old[1:4], "recipe-source-fields", "main", row["recipe_id"])
        argv = json.loads(row["current_argv"])
        check(isinstance(argv, list) and bool(argv) and all(isinstance(arg, str) for arg in argv), "recipe-argv-shape", "main", row["recipe_id"])
        for kind in ("txt", "ansi", "cursor", "html", "png"):
            check(("main", row["expected_" + kind]) in lookup, "recipe-artifact-exists", "main", row["expected_" + kind])
    provenance = json.loads(data("main", "shots/capture-provenance.json"))
    matrix = list(csv.DictReader(io.StringIO(data("main", "shots/capture-matrix.tsv").decode()), delimiter="\t"))
    for row in matrix:
        content = data("main", row["path"])
        check(content is not None and len(content) == int(row["bytes"]) and sha(content) == row["sha256"], "capture-matrix-bytes", "main", row["path"])
    for row in provenance:
        for kind, artifact in row["artifacts"].items():
            content = data("main", artifact["path"])
            check(content is not None and len(content) == artifact["bytes"] and sha(content) == artifact["sha256"], "capture-provenance-bytes", "main", artifact["path"])
    check({r["path"] for r in matrix} == {r["artifacts"]["png"]["path"] for r in provenance}, "matrix-provenance-membership", "main", "shots/capture-matrix.tsv")
    return {"counts": dict(counts), "issues": issues,
            "boundary": "Historical byte/relationship inspection only; no recipe execution, frame blessing, font re-render or immutable-oracle acceptance"}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--text-differences", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    docs = root / "docs/refactoring-plan"
    if args.text_differences:
        relations = json.loads((docs / "branch-artifact-relationships.json").read_text())
        differences = []
        for row in relations["issues"]:
            if row["kind"] != "ansi-text-nonpadding-content":
                continue
            pin = {"main": MAIN, "holla": HOLLA}[row["side"]]
            base = row["path"].removesuffix(".png")
            ansi = subprocess.check_output(["git", "show", pin + ":" + base + ".ansi"], cwd=root).decode()
            plain = subprocess.check_output(["git", "show", pin + ":" + base + ".txt"], cwd=root).decode()
            stripped = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", ansi)
            left = [line.rstrip(" ") for line in stripped.splitlines()]
            right = [line.rstrip(" ") for line in plain.splitlines()]
            differences.append({**row, "ansi_sha256": sha(ansi.encode()), "text_sha256": sha(plain.encode()),
                                "diff": list(difflib.unified_diff(left, right, fromfile="ansi-without-SGR", tofile="text", n=1, lineterm=""))})
        if args.write:
            (docs / "branch-artifact-text-differences.json").write_text(json.dumps(differences, indent=2, ensure_ascii=False) + "\n")
        print(json.dumps(differences, indent=2, ensure_ascii=False))
        return
    with (docs / "branch-diff-inventory.tsv").open() as stream:
        rows = [row for row in csv.DictReader(stream, delimiter="\t") if row["partition"] == "artifacts"]
    keys = sorted({row[side + "_blob"] for row in rows for side in ("holla", "main") if set(row[side + "_blob"]) != {"0"}})
    batch = subprocess.check_output(["git", "cat-file", "--batch"], input=("\n".join(keys) + "\n").encode(), cwd=root)
    stream = io.BytesIO(batch)
    blobs = {}
    for key in keys:
        actual, kind, size = stream.readline().decode().split()
        if actual != key or kind != "blob":
            raise ValueError("Unexpected Git object")
        data = stream.read(int(size))
        if len(data) != int(size) or stream.read(1) != b"\n":
            raise ValueError("Truncated Git object")
        if hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest() != key:
            raise ValueError("Git blob identity mismatch")
        blobs[key] = data
    if stream.read():
        raise ValueError("Unexpected batch output")
    cache, records, errors = {}, [], []
    for row in rows:
        for side in ("holla", "main"):
            key = row[side + "_blob"]
            if key not in blobs:
                continue
            identity = (key, Path(row["path"]).name.rsplit(".", 1)[-1])
            if identity not in cache:
                try:
                    cache[identity] = decode(row["path"], blobs[key])
                except Exception as error:
                    cache[identity] = {"bytes": len(blobs[key]), "sha256": sha(blobs[key]), "decode_error": type(error).__name__ + ": " + str(error)}
            record = {"path": row["path"], "side": side, "git_blob": key, **cache[identity]}
            records.append(record)
            if "decode_error" in record:
                errors.append({key: record[key] for key in ("path", "side", "decode_error")})
    summary = {"main": MAIN, "holla": HOLLA, "changed_paths": len(rows), "side_records": len(records),
               "unique_blobs": len(blobs), "decoded_bytes": sum(r["bytes"] for r in records),
               "formats": dict(Counter(r.get("format", "error") for r in records)), "decode_errors": errors,
               "scope": "Complete byte and format inspection; not manual visual review, VT replay, or parity approval"}
    relations = relationships(records, blobs)
    if args.write:
        (docs / "branch-artifact-census.json").write_text(json.dumps(summary, indent=2) + "\n")
        (docs / "branch-artifact-records.jsonl").write_text("".join(json.dumps(record, sort_keys=True) + "\n" for record in records))
        (docs / "branch-artifact-relationships.json").write_text(json.dumps(relations, indent=2) + "\n")
    summary["relationship_counts"] = relations["counts"]
    summary["relationship_issue_count"] = len(relations["issues"])
    print(json.dumps(summary, indent=2))

if __name__ == "__main__":
    main()
