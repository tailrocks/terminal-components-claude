"""Validate T00 source record consistency without running product code."""
import gzip
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent

EXPECTED_LEDGERS = {
    "tracked-tree.jsonl.gz": (94_896, 16_734_722_741, 69_405),
    "test-files.jsonl.gz": (93_917, 16_711_374_993, 68_470),
    "tracked-tree-31f.jsonl.gz": (94_896, 16_734_721_880, 69_405),
    "test-files-31f.jsonl.gz": (93_917, 16_711_374_132, 68_470),
    "tracked-tree-1d6.jsonl.gz": (94_896, 16_734_723_891, 69_406),
    "test-files-1d6.jsonl.gz": (93_917, 16_711_376_143, 68_471),
}


def read_rows(name: str) -> list[dict]:
    with gzip.open(ROOT / name, "rt", encoding="utf-8") as stream:
        return [json.loads(line) for line in stream]


def main() -> None:
    json_paths = list(ROOT.rglob("*.json"))
    for path in json_paths:
        json.loads(path.read_text(encoding="utf-8"))

    ledgers = {}
    for name, expected in EXPECTED_LEDGERS.items():
        rows = read_rows(name)
        actual = (
            len(rows),
            sum(row["size_bytes"] for row in rows),
            len({row["blob_oid"] for row in rows}),
        )
        assert actual == expected, (name, actual, expected)
        assert len({row["path"] for row in rows}) == len(rows), name
        ledgers[name] = rows

    tree_paths = {row["path"] for row in ledgers["tracked-tree-1d6.jsonl.gz"]}
    test_paths = {row["path"] for row in ledgers["test-files-1d6.jsonl.gz"]}
    assert test_paths <= tree_paths

    ci = json.loads((ROOT / "ci-head-1d6-rest.json").read_text())
    assert ci["total_count"] == 100
    assert ci["jobs_returned"] == 100
    assert ci["pagination_complete"] is True
    assert ci["job_status_counts"] == {
        "completed/cancelled": 95,
        "completed/failure": 1,
        "completed/skipped": 2,
        "completed/success": 2,
    }

    inventory = json.loads((ROOT / "test-inventory.json").read_text())
    directives = inventory["required_comment_directives"]
    assert len(directives) == 3_025
    script_paths = {row["path"] for row in inventory["source_scan_boundary"]["extensionless_executable_scripts"]}
    assert script_paths == {"tools/refactor-proof/bin/tc-proof", "tools/refactor-proof/bin/tc-proof-host"}
    for path in script_paths:
        assert any(row["path"] == path and row["source"].startswith("#!") for row in directives)
    script_record = json.loads((ROOT / "script-directives.json").read_text())
    assert {row["path"] for row in script_record["scripts"]} == script_paths

    findings = json.loads((ROOT / "finding-deltas-e08-to-1d6.json").read_text())
    assert len(findings["f13"]["moved_files"]) == 7
    assert findings["f13"]["behavior_proof_rows"] == 6
    assert not list(ROOT.rglob("__pycache__"))

    print(
        f"JSON files valid ({len(json_paths)}); six ledgers reconciled; "
        "1d6 test paths are a subset of the full tree; CI pagination is "
        "complete at 100 jobs; F13 records seven moves and six proofs; "
        "no __pycache__."
    )


if __name__ == "__main__":
    main()
