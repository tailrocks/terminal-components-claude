#!/usr/bin/env python3
"""Check T37 API prose identities, cleanup references, and document excerpts.

This checks structure and provenance only. It does not score or prove semantic
correctness of any disposition.
"""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]
INPUT = "e164e7b5b17e9c2d78bf30cfbbfe6bdb36e7b7ab"
RAW_COMMIT = "2fee2c277d1213fc5fad5d81eb77479600182b69"
PRODUCT = "1d6ee92adf6ccca38eeee64b1371cba1497e8fcd"
CLEANUP = "7368d4fc5a71e46cae5ee92963bcf3a6943953e9"
PLAN_ROOT = Path("/Users/donbeave/Downloads/termrock-pr38-plan")
DECISIONS = Path(__file__).with_name("decisions.json")
FOUNDATION = ROOT / "docs/contracts/foundation-values.md"


def git_bytes(commit: str, path: str) -> bytes:
    return subprocess.check_output(["git", "show", f"{commit}:{path}"], cwd=ROOT)


def git_json(commit: str, path: str) -> dict:
    return json.loads(git_bytes(commit, path))


def text_hash(lines: list[str]) -> str:
    compact = json.dumps(lines, ensure_ascii=False, separators=(",", ":"))
    return hashlib.sha256(compact.encode()).hexdigest()


def slug(text: str) -> str:
    text = re.sub(r"[`*_]", "", text.lower())
    text = re.sub(r"[^a-z0-9 -]", "", text)
    return re.sub(r"\s+", "-", text.strip())


def check_markdown_sentences(path: Path) -> list[str]:
    failures: list[str] = []
    for line_no, line in enumerate(path.read_text().splitlines(), 1):
        if line.startswith("#") or line.startswith("|"):
            continue
        plain = re.sub(r"\[[^\]]+\]\([^)]*\)", "link", line)
        plain = re.sub(r"https?://\S+", "url", plain)
        plain = re.sub(r"`[^`]*`", "code", plain)
        for sentence in re.split(r"(?<=[.!?])\s+(?=[A-Z`])", plain):
            words = re.findall(r"\b[\w]+(?:[-'][\w]+)*\b", sentence)
            if not words:
                continue
            # Descriptive and procedural clauses use the stricter 20-word
            # ceiling here; this structural scan never validates meaning.
            if len(words) > 20:
                failures.append(f"{path}:{line_no}: {len(words)} words: {sentence.strip()}")
    return failures


def main() -> None:
    raw = git_json(RAW_COMMIT, "docs/implementation/pr38-final/results/t37-foundations/source-api-prose.json")
    crosswalk = git_json(INPUT, "docs/implementation/pr38-final/results/t37-shared/foundation-api-prose-crosswalk.json")
    data = json.loads(DECISIONS.read_text())
    raw_by_record = {i: record for i, record in enumerate(raw["records"])}
    cross_by_record = {record["source_map_record"]: record for record in crosswalk["records"]}
    expected = {
        i: record
        for i, record in enumerate(raw["records"])
        if record["path"].startswith(("crates/termrock-core/", "crates/termrock-layout/", "crates/termrock-text/"))
    }
    records = data["records"]
    assert len(records) == 322, len(records)
    assert len({record["id"] for record in records}) == 322
    assert len({record["source_map_record"] for record in records}) == 322
    assert {record["source_map_record"] for record in records} == set(expected)

    counts: Counter[str] = Counter()
    for record in records:
        index = record["source_map_record"]
        original = raw_by_record[index]
        identity = cross_by_record[index]
        for key in ("id", "source_map_record", "path", "line_range", "source_sha256", "text_sha256"):
            assert record[key] == identity[key], (index, key)
        assert record["path"] == original["path"]
        assert record["line_range"] == {"start": original["start_line"], "end": original["end_line"]}
        assert record["source_text"] == original["text"]
        assert text_hash(original["text"]) == record["text_sha256"]
        source = git_bytes(PRODUCT, record["path"])
        assert hashlib.sha256(source).hexdigest() == record["source_sha256"]

        cleaned = git_bytes(CLEANUP, record["path"]).decode().splitlines()
        item = record["attached_item"]
        if item["kind"] == "module_documentation":
            assert item["anchor_declaration"]
            assert item["anchor_declaration"] in "\n".join(cleaned), index
        else:
            assert item["signature"] in [line.strip() for line in cleaned], index
        assert record["cleanup_target_status"]["status"] == "present_after_accepted_cleanup"

        disposition = record["disposition"]
        counts[disposition] += 1
        if record["destination"]:
            dest, anchor = record["destination"].split("#", 1)
            if dest == "API-RULES.md":
                page = PLAN_ROOT / dest
            else:
                page = ROOT / dest
            content = page.read_text()
            assert record["supporting_excerpt"] in content, (index, dest)
            headings = {slug(m.group(1)) for m in re.finditer(r"^#{1,6}\s+(.+?)\s*$", content, re.M)}
            assert anchor in headings, (index, anchor, headings)
        else:
            assert record["specific_reason"], index
        if disposition in {
            "required_maintained_clause",
            "source_wording_conflict",
            "source_observation",
        }:
            assert record["required_clause"], index
            assert record["destination"], index

    paths = Counter(record["path"].split("/")[1] + "/" for record in records)
    assert paths == Counter({"termrock-core/": 268, "termrock-layout/": 32, "termrock-text/": 22}), paths

    sentence_failures = check_markdown_sentences(FOUNDATION)
    assert not sentence_failures, "\n".join(sentence_failures)
    print("322 identities, source/text hashes, cleaned declarations, destinations, anchors, and excerpts: PASS")
    print("cohort counts:", dict(sorted(paths.items())))
    print("dispositions:", dict(sorted(counts.items())))
    print("foundation-values sentence-length scan (20-word ceiling): PASS")
    print("semantic correctness: NOT CHECKED")


if __name__ == "__main__":
    main()
