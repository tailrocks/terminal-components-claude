#!/usr/bin/env python3
"""Align refactoring-tasks catalog docs with taskfmt container path contract."""

from __future__ import annotations

import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "refactoring-tasks" / "terminal-components"
COMPLETION = CATALOG / "completion"
ROOT_CAMPAIGN = CATALOG / "CAMPAIGN_AGENTS.md"

VERIFY_ENV_MARKER = "## Verify environment (container paths)"

BOOTSTRAP_VERIFY_ENV = """\
## Verify environment (container paths)

`verify.toml` subprocess argv is literal; taskfmt `--task-dir` / `--root` do **not** rewrite check paths. Host must expose these read-only mounts before checks run:

| Path | Purpose |
| --- | --- |
| `/proof/bootstrap/bin/taskfmt` | Pinned taskfmt gate |
| `/proof/bootstrap/task-format/` | Pinned task-format source @ `52d9f1eb…` |
| `/proof/bootstrap/experiment.toml` | Frozen taskfmt config |
| `/work/tools/refactor-proof/bin/tc-proof` | Comparator (synced Mach-O after `sync-binaries.sh`) |
| `/work/tools/refactor-proof/bin/tc-proof-host` | Host core (TASK-001; synced Mach-O) |

Bootstrap drivers live under `/task/trusted/proof-bootstrap/`. See `/work/docs/refactoring-plan/path-contract.md`.
"""

BOOTSTRAP_070_VERIFY_ENV = """\
## Verify environment (container paths)

`verify.toml` subprocess argv is literal; taskfmt `--task-dir` / `--root` do **not** rewrite check paths. Host must expose these read-only mounts before checks run:

| Path | Purpose |
| --- | --- |
| `/proof/bootstrap/bin/taskfmt` | Pinned taskfmt gate |
| `/proof/bootstrap/task-format/` | Pinned task-format source @ `52d9f1eb…` |
| `/proof/bootstrap/experiment.toml` | Frozen taskfmt config |
| `/work/tools/refactor-proof/bin/tc-proof` | Comparator/runner (synced Mach-O) |

Bootstrap drivers live under `/task/trusted/proof-bootstrap/`. See `/work/docs/refactoring-plan/path-contract.md`.
"""

HYBRID_VERIFY_ENV = """\
## Verify environment (container paths)

`verify.toml` subprocess argv is literal; taskfmt `--task-dir` / `--root` do **not** rewrite check paths. Hybrid band:

| Path | Purpose |
| --- | --- |
| `/proof/bin/tc-proof` | CHK-001 preflight only |
| `/run/tc-proof/context-index.json` | Frozen context index |
| `/run/tc-proof/contexts/CHK-001.json` | CHK-001 frozen context |
| `/proof/bootstrap/bin/taskfmt` | Bootstrap driver checks |
| `/proof/bootstrap/task-format/` | Pinned task-format source |
| `/proof/bootstrap/experiment.toml` | Frozen taskfmt config |
| `/work/tools/refactor-proof/bin/tc-proof` | Bootstrap drivers (CHK-002+) |

See `/work/docs/refactoring-plan/path-contract.md`.
"""

PRODUCTION_VERIFY_ENV = """\
## Verify environment (container paths)

`verify.toml` subprocess argv is literal; taskfmt `--task-dir` / `--root` do **not** rewrite check paths. Host must expose these read-only mounts before checks run:

| Path | Purpose |
| --- | --- |
| `/proof/bootstrap/bin/taskfmt` | Pinned taskfmt gate |
| `/proof/bootstrap/experiment.toml` | Frozen taskfmt config |
| `/proof/bin/tc-proof` | Harness worker invoked by every check in `verify.toml` |
| `/run/tc-proof/context-index.json` | Frozen context index |
| `/run/tc-proof/contexts/CHK-NNN.json` | One frozen context per declared check id |

See `/work/docs/refactoring-plan/path-contract.md`.
"""

STEP7_OLD = (
    "7. When implementation leaves are complete, run `taskfmt verify --progress \"\"` "
    "from `/work`; fix and rerun until it exits 0 with last line `DONE`. Then append "
    "the terminal progress event and run full `taskfmt verify`. Its complete output is "
    "completion evidence."
)
STEP7_NEW = (
    "7. When implementation leaves are complete, run `taskfmt verify --progress \"\"` "
    "from `/work`; fix and rerun until it exits 0 with last line `DONE`. Then append "
    "the terminal progress event and run full `taskfmt verify`. Its complete output is "
    "completion evidence. **Campaign:** `/task/CAMPAIGN_AGENTS.md` supersedes this step "
    "for completion authority — do not use `--progress \"\"` or claim `DONE` without "
    "host `tc-proof-host verify`."
)

CONTAINER_MOUNTS_BULLET = (
    "- **Container mounts:** Read-only `/proof/bin/tc-proof`, "
    "`/run/tc-proof/context-index.json`, and `/run/tc-proof/contexts/CHK-NNN.json` "
    "per `verify.toml`; taskfmt `--task-dir` / `--root` do not substitute for these mounts."
)

CONTEXTS_OLD = "immutable per-check contexts"
CONTEXTS_NEW = (
    "`/run/tc-proof/context-index.json` and `/run/tc-proof/contexts/CHK-NNN.json` "
    "(immutable per-check contexts)"
)


def task_num(path: Path) -> int:
    return int(path.name)


def verify_env_for(num: int) -> str:
    if num == 1:
        return BOOTSTRAP_VERIFY_ENV
    if num == 70:
        return BOOTSTRAP_070_VERIFY_ENV
    if num in (71, 72):
        return HYBRID_VERIFY_ENV
    return PRODUCTION_VERIFY_ENV


def patch_readme(readme: Path, num: int) -> bool:
    text = readme.read_text()
    changed = False

    production_band = num >= 2 and num not in (70, 71, 72)
    if production_band and "**Container mounts:**" not in text:
        if "## Preconditions" in text and "## Scope" in text:
            pre, rest = text.split("## Scope", 1)
            if "/run/tc-proof/contexts/" not in pre:
                lines = pre.splitlines(keepends=True)
                insert_at = len(lines)
                for i, line in enumerate(lines):
                    if line.startswith("- **P-"):
                        insert_at = i + 1
                lines.insert(insert_at, CONTAINER_MOUNTS_BULLET + "\n")
                text = "".join(lines) + "## Scope" + rest
                changed = True

    if CONTEXTS_OLD in text and CONTEXTS_NEW not in text:
        text = text.replace(CONTEXTS_OLD, CONTEXTS_NEW)
        changed = True

    # Bare docs/refactoring-plan/ in Read before editing (051-064 style)
    def repl_doc_link(match: re.Match[str]) -> str:
        name = match.group(1)
        return f"[`/work/docs/refactoring-plan/{name}`](/work/docs/refactoring-plan/{name})"

    new_text = re.sub(
        r"`docs/refactoring-plan/([a-z0-9\-]+\.md)`",
        repl_doc_link,
        text,
    )
    if new_text != text:
        text = new_text
        changed = True

    # Visual regression host paths
    vis_old = "`docs/baseline/snapshots-v2.md`, `refactoring-tasks/visual-validation.md`"
    vis_new = (
        "`/work/docs/baseline/snapshots-v2.md` "
        "(host catalog: `refactoring-tasks/visual-validation.md`)"
    )
    if vis_old in text:
        text = text.replace(vis_old, vis_new)
        changed = True

    if changed:
        readme.write_text(text)
    return changed


def sync_campaign_agents(task_dir: Path) -> bool:
    dest = task_dir / "CAMPAIGN_AGENTS.md"
    if not ROOT_CAMPAIGN.exists():
        return False
    if dest.exists() and dest.read_text() == ROOT_CAMPAIGN.read_text():
        return False
    shutil.copy2(ROOT_CAMPAIGN, dest)
    return True


def main() -> None:
    stats = {"readme": 0, "campaign": 0}
    for task_dir in sorted(COMPLETION.iterdir()):
        if not task_dir.is_dir() or not task_dir.name.isdigit():
            continue
        num = task_num(task_dir)
        readme = task_dir / "README.md"
        if readme.is_file() and patch_readme(readme, num):
            stats["readme"] += 1
        if sync_campaign_agents(task_dir):
            stats["campaign"] += 1

    completion_readme = COMPLETION / "README.md"
    if completion_readme.is_file():
        text = completion_readme.read_text()
        note = (
            "\n> Container verify paths: see "
            "[`/work/docs/refactoring-plan/path-contract.md`](/work/docs/refactoring-plan/path-contract.md). "
            "Host catalog paths below are for planning checkout only.\n"
        )
        if "Container verify paths:" not in text:
            text = text.replace("# Completion tasks\n", "# Completion tasks\n" + note, 1)
            completion_readme.write_text(text)

    print(
        f"Patched README: {stats['readme']}, CAMPAIGN_AGENTS: {stats['campaign']}"
    )


if __name__ == "__main__":
    main()
