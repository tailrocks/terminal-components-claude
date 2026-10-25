"""Hash every T00 source/tool evidence file except this index itself."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
INDEX = ROOT / "artifact-index.json"


def main() -> None:
    artifacts = []
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or path == INDEX:
            continue
        data = path.read_bytes()
        artifacts.append(
            {
                "path": path.relative_to(ROOT).as_posix(),
                "size_bytes": len(data),
                "sha256": hashlib.sha256(data).hexdigest(),
            }
        )
    record = {
        "schema": "t00.artifact-index.v1",
        "source_commit": "1d6ee92adf6ccca38eeee64b1371cba1497e8fcd",
        "integration_input_commit": "381c88c2f1a26bb727ab9658e8b694bea5915aa6",
        "artifact_count": len(artifacts),
        "hash": "SHA-256 over the committed byte content of each listed file.",
        "exclusion": "artifact-index.json does not contain its own digest. Verify its file hash from Git after commit.",
        "artifacts": artifacts,
    }
    INDEX.write_text(
        json.dumps(record, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(f"indexed {len(artifacts)} T00 evidence files")


if __name__ == "__main__":
    main()
