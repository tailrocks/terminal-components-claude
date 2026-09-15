# TASK-001 GitHub Actions CI sketch (optional)

**Date:** 2026-09-15  
**Task:** TASK-001 — Implement independently qualified comparator and host core  
**Branch:** `prep-wave1-verify` (planning); execution PR [#4](https://github.com/donbeave/terminal-components-claude/pull/4) on `task-001-bootstrap`  
**Status:** Design sketch only — no workflow file is committed by this document.  
**Authority:** [`task-001-ci-requirements.md`](task-001-ci-requirements.md), [`001/verify.toml`](../../refactoring-tasks/terminal-components/completion/001/verify.toml)

This document proposes an **optional** GitHub Actions job set for PR #4 (`tools/refactor-proof` bootstrap). It complements the container/bootstrap spec in [`task-001-ci-requirements.md`](task-001-ci-requirements.md) with a concrete workflow shape. Operators may adopt, defer, or replace it; merge of PR #4 does not require this workflow to land.

---

## Goals

| Goal | Rationale |
| --- | --- |
| Prove `refactor-proof` builds on CI | Catches Cargo/path regressions early |
| Run `cargo nextest run -p refactor-proof` | 17 unit/integration tests gate the harness |
| Run comparator CHK-004 when `tuisnap` is available | Frame roundtrip enrichment; skip cleanly when sibling checkout absent |
| Run host matrix only on macOS | CHK-005/006/007 require Darwin `sandbox-exec`; Linux must not fail the workflow |

Non-goals for this sketch: full `taskfmt verify` (OB-006), pinned taskfmt bootstrap image, or workspace-wide nextest (3201 tests remain advisory on operator hardware).

---

## Trigger and scope

```yaml
on:
  pull_request:
    paths:
      - 'tools/refactor-proof/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
      - 'refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/**'
  workflow_dispatch:
```

Run on PRs that touch the harness or its trusted drivers. `workflow_dispatch` allows manual re-run after operator provisions `tui-snap`.

---

## Job matrix

| Job | `runs-on` | Required | Checks |
| --- | --- | --- | --- |
| `refactor-proof-build` | `ubuntu-latest` | yes | `cargo build -p refactor-proof`; artifact: build log |
| `refactor-proof-nextest` | `ubuntu-latest` | yes | `cargo nextest run -p refactor-proof` |
| `refactor-proof-comparator` | `macos-latest` | optional | CHK-004 via `proof-comparator-bootstrap.py` when `tuisnap` resolvable |
| `refactor-proof-host` | `macos-latest` | optional (macOS-only) | CHK-001/005/006/007 via `host-bootstrap-driver.py`; **skipped on Linux** |

`refactor-proof-build` and `refactor-proof-nextest` may be merged into one job; split here for clearer failure attribution.

---

## Shared setup (all jobs)

```yaml
env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -D warnings

steps:
  - uses: actions/checkout@v4
  - uses: dtolnay/rust-toolchain@stable
    with:
      components: rustfmt, clippy
  - uses: taiki-e/install-action@nextest
  - uses: Swatinem/rust-cache@v2
    with:
      workspaces: tools/refactor-proof
```

Install `nextest` on every job that runs tests. Use `cargo metadata --locked` if the workspace lockfile is present on the PR branch.

---

## Job: `refactor-proof-build`

Linux is sufficient — no Mach-O sync required for compile-only.

```yaml
refactor-proof-build:
  name: refactor-proof build
  runs-on: ubuntu-latest
  steps:
    # … shared setup …
    - name: Build refactor-proof
      run: cargo build --locked -p refactor-proof
```

On macOS qualification paths, follow build with `tools/refactor-proof/scripts/sync-binaries.sh` before any driver that reads `tools/refactor-proof/bin/*`.

---

## Job: `refactor-proof-nextest`

```yaml
refactor-proof-nextest:
  name: refactor-proof nextest
  runs-on: ubuntu-latest
  needs: refactor-proof-build
  steps:
    # … shared setup …
    - name: nextest -p refactor-proof
      run: cargo nextest run --locked -p refactor-proof
```

Expected: **17 passed** (per [`task-001-qualification-report.md`](task-001-qualification-report.md) @ `3ed51570`).

---

## Job: `refactor-proof-comparator` (macOS, tuisnap-gated)

Comparator CHK-004 invokes `tools/refactor-proof/bin/tc-proof`. Frame roundtrip cases need a built `tuisnap` binary at campaign pin `0a2e490802b7b048cd96349c6af860f8a3a05c3d`.

### tuisnap availability probe

```yaml
    - name: Resolve tuisnap checkout
      id: tuisnap
      run: |
        set -euo pipefail
        for candidate in \
          "${{ github.workspace }}/../tui-snap" \
          "${TUI_SNAP_PATH:-}"; do
          if [ -n "$candidate" ] && [ -d "$candidate/.git" ]; then
            echo "path=$candidate" >> "$GITHUB_OUTPUT"
            echo "available=true" >> "$GITHUB_OUTPUT"
            exit 0
          fi
        done
        echo "available=false" >> "$GITHUB_OUTPUT"
```

Default GitHub-hosted runners **do not** have a sibling `tui-snap` checkout. Without `TUI_SNAP_PATH` (repository secret or self-hosted layout), the job skips comparator execution and reports success with an explicit skip annotation.

### Comparator steps (when available)

```yaml
refactor-proof-comparator:
  name: refactor-proof comparator (CHK-004)
  runs-on: macos-latest
  needs: refactor-proof-build
  steps:
    # … shared setup …
    - name: Resolve tuisnap checkout
      id: tuisnap
      run: |
        # … probe above …

    - name: Skip comparator (no tuisnap)
      if: steps.tuisnap.outputs.available != 'true'
      run: |
        echo "::notice::Skipping CHK-004 — tuisnap checkout not available. Set TUI_SNAP_PATH or use self-hosted macOS with sibling checkout."

    - name: Build refactor-proof and sync bins
      if: steps.tuisnap.outputs.available == 'true'
      run: |
        cargo build --locked -p refactor-proof
        tools/refactor-proof/scripts/sync-binaries.sh

    - name: Build tuisnap @ pin
      if: steps.tuisnap.outputs.available == 'true'
      env:
        TUI_SNAP: ${{ steps.tuisnap.outputs.path }}
        TUI_SNAP_PIN: 0a2e490802b7b048cd96349c6af860f8a3a05c3d
      run: |
        git -C "$TUI_SNAP" fetch --depth 1 origin "$TUI_SNAP_PIN"
        git -C "$TUI_SNAP" checkout "$TUI_SNAP_PIN"
        test "$(git -C "$TUI_SNAP" rev-parse HEAD)" = "$TUI_SNAP_PIN"
        cargo build --locked --release -p tuisnap --bin tuisnap

    - name: CHK-004 comparator bootstrap
      if: steps.tuisnap.outputs.available == 'true'
      env:
        TUI_SNAP: ${{ steps.tuisnap.outputs.path }}
      run: |
        python3 refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/proof-comparator-bootstrap.py \
          --runner tools/refactor-proof/bin/tc-proof \
          --tuisnap "$TUI_SNAP/target/release/tuisnap"
```

Without `--tuisnap`, the driver still runs vectors that do not require frame roundtrip; with `--tuisnap`, expect **141/141** invocations pass ([`task-001-qualification-report.md`](task-001-qualification-report.md) §2).

---

## Job: `refactor-proof-host` (macOS only)

Host matrix checks **require** macOS `sandbox-exec`. This job must never run on `ubuntu-latest`.

```yaml
refactor-proof-host:
  name: refactor-proof host matrix
  runs-on: macos-latest
  needs: refactor-proof-comparator
  if: runner.os == 'macOS'
  steps:
    # … shared setup …
    - name: Guard — macOS only
      run: |
        test "$(uname -s)" = Darwin
        test -x /usr/bin/sandbox-exec

    - name: Build and sync Mach-O harness bins
      run: |
        cargo build --locked -p refactor-proof
        tools/refactor-proof/scripts/sync-binaries.sh

    - name: CHK-001 precondition (driver self-test)
      run: |
        python3 refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/host-bootstrap-driver.py \
          --self-test

    - name: CHK-005/006/007 host matrix
      run: |
        python3 refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/host-bootstrap-driver.py \
          --host tools/refactor-proof/bin/tc-proof-host
```

Pinned taskfmt paths (`/proof/bootstrap/bin/taskfmt`) are **not** wired in this sketch. Full OB-006 qualification remains operator-driven per [`task-001-phase4-operator-runbook.md`](task-001-phase4-operator-runbook.md). CI host job here runs driver self-test + `--host` matrix only when operator supplies taskfmt via follow-up workflow secrets or container init.

### Linux skip policy

Do **not** add a Linux leg for host matrix. If a monolithic workflow runs on `ubuntu-latest`, gate host steps:

```yaml
    - name: Host matrix (macOS only)
      if: runner.os == 'macOS'
      run: |
        # host-bootstrap-driver.py --host …
```

On Linux, the step is skipped (`skipped` in Actions UI), not failed.

---

## Full workflow skeleton

Optional file: `.github/workflows/refactor-proof.yml` (not committed by this planning doc).

```yaml
name: refactor-proof (TASK-001 sketch)

on:
  pull_request:
    paths:
      - 'tools/refactor-proof/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
  workflow_dispatch:

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -D warnings

jobs:
  refactor-proof-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@nextest
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --locked -p refactor-proof

  refactor-proof-nextest:
    runs-on: ubuntu-latest
    needs: refactor-proof-build
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@nextest
      - uses: Swatinem/rust-cache@v2
      - run: cargo nextest run --locked -p refactor-proof

  refactor-proof-comparator:
    runs-on: macos-latest
    needs: refactor-proof-build
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      # tuisnap probe + conditional CHK-004 (see sections above)

  refactor-proof-host:
    runs-on: macos-latest
    needs: refactor-proof-comparator
    if: runner.os == 'macOS'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      # sync-binaries + host-bootstrap-driver (see sections above)
```

---

## Adoption checklist

| Step | Owner | Notes |
| --- | --- | --- |
| Add `.github/workflows/refactor-proof.yml` from skeleton | Integrator | After PR #4 review |
| Provision `TUI_SNAP_PATH` on self-hosted macOS or accept comparator skip | Operator | Roundtrip cases optional on GitHub-hosted |
| Wire pinned taskfmt bootstrap for full CHK-001 gate | Operator | Deferred — see [`task-001-ci-requirements.md`](task-001-ci-requirements.md) § pinned taskfmt |
| Confirm macOS runner has CLT git at fixed path | Operator | Required for observer/host drivers |
| Record workflow SHA in operator evidence | Operator | [`task-001-operator-evidence-draft.md`](task-001-operator-evidence-draft.md) |

---

## Related documents

- [`task-001-ci-requirements.md`](task-001-ci-requirements.md) — container paths, sync-binaries, sandbox-exec, full job matrix
- [`task-001-qualification-report.md`](task-001-qualification-report.md) — measured pass counts and reproduction commands
- [`task-001-verify-container.md`](task-001-verify-container.md) — `/task`, `/work`, `/proof/bootstrap` layout for `taskfmt verify`
- [`task-001-bootstrap-plan.md`](task-001-bootstrap-plan.md) — phased implementation plan
- PR [#4](https://github.com/donbeave/terminal-components-claude/pull/4) — `tools/refactor-proof` bootstrap handoff
