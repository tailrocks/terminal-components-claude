import hashlib
import json
import re
import runpy
import subprocess
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path.cwd()
RESULTS = ROOT / "docs/implementation/pr38-final/results/t37-foundations"
AUDIT = runpy.run_path(str(RESULTS / "audit_source.py"))
COMMIT = AUDIT["COMMIT"]
SOURCE_API = json.loads((RESULTS / "source-api-prose.json").read_text())


def git_blob(path):
    return subprocess.check_output(["git", "show", f"{COMMIT}:{path}"], cwd=ROOT)


def line_offsets(text):
    offsets = [0]
    for line in text.splitlines(keepends=True):
        offsets.append(offsets[-1] + len(line))
    return offsets


def next_attribute_scope(text, masked, offset):
    while offset < len(masked) and masked[offset].isspace():
        offset += 1
    while masked.startswith("#[", offset):
        end = AUDIT["attr_end"](masked, offset)
        attribute = text[offset:end]
        if re.search(r"cfg\s*\(\s*test\s*\)", attribute):
            return "cfg_test_scope"
        if re.search(r'cfg\s*\(.*feature\s*=\s*"testing"', attribute):
            return "approved_testing_feature_scope"
        offset = end
        while offset < len(masked) and masked[offset].isspace():
            offset += 1
    return None


def explicit_visibility(text, masked, offset):
    while offset < len(masked) and masked[offset].isspace():
        offset += 1
    while masked.startswith("#[", offset):
        offset = AUDIT["attr_end"](masked, offset)
        while offset < len(masked) and masked[offset].isspace():
            offset += 1
    target = masked[offset:offset + 500]
    if re.match(r"pub\s*\(\s*(?:crate|super|in\s+crate::[^)]+)\s*\)", target):
        return "restricted_visibility"
    if re.match(r"pub\b", target):
        return "explicit_public_visibility"
    return "visibility_unresolved_by_lexical_scan"


def main():
    grouped = defaultdict(list)
    for index, row in enumerate(SOURCE_API["records"]):
        grouped[row["path"]].append((index, row))

    records = []
    per_file = []
    for path, rows in sorted(grouped.items()):
        original = git_blob(path).decode()
        offsets = line_offsets(original)
        masked = AUDIT["mask_rust"](original)
        input_sha = hashlib.sha256(original.encode()).hexdigest()
        exists_after_cleanup = (ROOT / path).is_file()
        test_ranges = []
        feature_ranges = []

        if exists_after_cleanup:
            test_ranges = AUDIT["cfg_test_ranges"](path, original)
            for line_numbers in AUDIT["APPROVED_FEATURE_SCOPE_LINES"].get(path, {}).values():
                for line_number in line_numbers:
                    offset = offsets[line_number - 1]
                    while offset < len(original) and original[offset].isspace() and original[offset] != "\n":
                        offset += 1
                    match = AUDIT["CFG_TESTING"].match(original, offset)
                    if match is None:
                        match = AUDIT["CFG_ANY_TESTING"].match(original, offset)
                    if match is None:
                        continue
                    end = AUDIT["attr_item_end"](masked, match.end())
                    feature_ranges.append((offset, end))

        counts = Counter()
        for index, row in rows:
            start = offsets[row["start_line"] - 1]
            end = offsets[min(row["end_line"], len(offsets) - 1)]
            if not exists_after_cleanup:
                category = "removed_with_deleted_test_or_fixture_source"
                visibility = "not_applicable"
            elif any(left <= start < right for left, right in feature_ranges):
                category = "inside_approved_testing_feature_scope"
                visibility = "not_applicable"
            elif any(left <= start < right for left, right, _line in test_ranges):
                category = "inside_cfg_test_scope"
                visibility = "not_applicable"
            else:
                attached_scope = next_attribute_scope(original, masked, end)
                if attached_scope:
                    category = "attached_to_test_only_scope"
                    visibility = "not_applicable"
                else:
                    category = "surviving_production_source"
                    visibility = explicit_visibility(original, masked, end)

            counts[category] += 1
            if category == "surviving_production_source":
                counts[visibility] += 1
            records.append({
                "source_map_record": index,
                "path": path,
                "source_sha256": input_sha,
                "start_line": row["start_line"],
                "end_line": row["end_line"],
                "category": category,
                "visibility": visibility,
            })

        per_file.append({
            "path": path,
            "source_sha256": input_sha,
            "doc_group_count": len(rows),
            "categories": dict(sorted(counts.items())),
        })

    category_counts = Counter(record["category"] for record in records)
    visibility_counts = Counter(
        record["visibility"]
        for record in records
        if record["category"] == "surviving_production_source"
    )
    result = {
        "schema": "t37.source-api-prose-disposition.v1",
        "task_id": "T37",
        "work_input_commit": COMMIT,
        "product_source_commit": SOURCE_API["product_source_commit"],
        "source_map_path": "docs/implementation/pr38-final/results/t37-foundations/source-api-prose.json",
        "source_map_sha256": hashlib.sha256((RESULTS / "source-api-prose.json").read_bytes()).hexdigest(),
        "record_count": len(records),
        "path_count": len(per_file),
        "category_counts": dict(sorted(category_counts.items())),
        "production_visibility_counts": dict(sorted(visibility_counts.items())),
        "method": "A static Rust-aware scan maps each input doc group to a deleted source path, a cfg(test) scope, an approved testing-feature scope, or surviving production source. The lexical visibility scan only marks explicit pub tokens. It does not classify contract need.",
        "required_api_prose_status": "Pending shared docs owner review. This map is raw provenance, not maintained API guidance.",
        "caller_contract_review": "No caller contract outside the reviewed esc_bridge safety rationale was separately identified by this source-only pass. This is not proof that the remaining API prose is unnecessary.",
        "per_file_counts": per_file,
        "records": records,
    }
    (RESULTS / "source-api-prose-disposition.json").write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps({
        "record_count": len(records),
        "path_count": len(per_file),
        "category_counts": result["category_counts"],
        "production_visibility_counts": result["production_visibility_counts"],
        "required_api_prose_status": result["required_api_prose_status"],
    }, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
