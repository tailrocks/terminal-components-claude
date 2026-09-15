# TASK-001 CI and container requirements

**Date:** 2026-09-15  
**Task:** TASK-001 — Implement independently qualified comparator and host core  
**Branch:** `prep-wave1-verify` (planning); execution worktree `task-001-bootstrap`  
**Authority:** [`001/verify.toml`](../../refactoring-tasks/terminal-components/completion/001/verify.toml), [`host-bootstrap-protocol.md`](../../refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/host-bootstrap-protocol.md)

This document specifies the machine and filesystem layout required for TASK-001 qualification (CHK-001, CHK-004, CHK-005, CHK-006, CHK-007). It complements [`task-001-bootstrap-plan.md`](task-001-bootstrap-plan.md) and [`task-001-qualification-report.md`](task-001-qualification-report.md).

---

## Container path layout

| Mount | Role |
| --- | --- |
| `/task/` | Read-only TASK-001 package (`trusted/proof-bootstrap/` drivers and vectors) |
| `/work/` | Candidate repository checkout (`writable_paths = ["tools/refactor-proof"]`) |
| `/proof/bootstrap/` | Operator-frozen bootstrap inputs (taskfmt, tui-snap object database, config) |

Required bootstrap paths under `/proof/bootstrap/`:

| Path | Content |
| --- | --- |
| `/proof/bootstrap/bin/taskfmt` | Pinned standalone taskfmt executable |
| `/proof/bootstrap/task-format/` | Pinned task-format source checkout @ `52d9f1eb…` |
| `/proof/bootstrap/experiment.toml` | Frozen taskfmt configuration (bytes bound by EV-007) |
| `/proof/bootstrap/tui-snap/` | Protected tui-snap Git object database @ campaign pin (see below) |

Local operator qualification may substitute `/tmp/taskfmt-install/bin/taskfmt` and `/tmp/taskfmt-qualification` for the taskfmt paths; container execution must use the `/proof/bootstrap/*` paths exactly as in `verify.toml`.

---

## Build sequence: `cargo build` then `sync-binaries.sh`

After every `cargo build -p refactor-proof` (or `cargo build --release -p refactor-proof` when `PROFILE=release`), run:

```sh
cd /work
cargo build -p refactor-proof
tools/refactor-proof/scripts/sync-binaries.sh
```

`sync-binaries.sh` copies workspace-built Mach-O binaries from `target/$PROFILE/` into `tools/refactor-proof/bin/`:

- `target/$PROFILE/tc-proof` → `tools/refactor-proof/bin/tc-proof`
- `target/$PROFILE/tc-proof-host` → `tools/refactor-proof/bin/tc-proof-host`

**Why this is mandatory:** `verify.toml` CHK-004/005/006/007 invoke `/work/tools/refactor-proof/bin/tc-proof` and `/work/tools/refactor-proof/bin/tc-proof-host`. The host bootstrap driver requires `bin/tc-proof-host` to be a regular Mach-O file (not a shell wrapper) so Darwin `sandbox-exec` can execute the submitted argv[0] and the `install` operation reproduces accepted harness bytes. Skipping sync leaves stale wrappers or missing binaries and fails CHK-005/006/007.

Optional: `PROFILE=release tools/refactor-proof/scripts/sync-binaries.sh` when qualifying release builds.

---

## Mach-O host path vs wrapper

| Path | Type | Qualification use |
| --- | --- | --- |
| `target/debug/tc-proof-host` (or `target/release/…`) | Built Mach-O | Direct `--host` qualification on bare macOS (advisory reports) |
| `tools/refactor-proof/bin/tc-proof-host` | Mach-O **after** `sync-binaries.sh` | **Required** for container/verify.toml paths |
| `scripts/dev-tc-proof-host.sh` | Shell wrapper → `target/debug/` | Development only; **must not** be synced to `bin/` |

The driver's `install` vector compares `digest(installed) == digest(submitted_executable)`. A shell wrapper receipt materializes Mach-O bytes under `install`, so digest(post-install) ≠ digest(wrapper script). Qualification therefore targets the synced Mach-O at `tools/refactor-proof/bin/tc-proof-host`, not a wrapper script at that path.

See [`task-001-qualification-report.md`](task-001-qualification-report.md) §4a/4b for measured pass/fail evidence.

---

## tui-snap path dependency and container mount

### Worktree Cargo path dep

`tools/refactor-proof/Cargo.toml` declares:

```toml
tuisnap = { path = "../../../../../tui-snap", default-features = false }
```

From the worktree root (`.worktrees/main`), this resolves to a sibling checkout:

| Field | Value |
| --- | --- |
| Host path | `/Users/donbeave/Projects/tui-snap` |
| Pinned revision | `0a2e490802b7b048cd96349c6af860f8a3a05c3d` |
| Reviewed merge head (ancestor) | `883d03f19d890bbbf27468798db78b04e85297ac` |

Verify before build: `git -C /Users/donbeave/Projects/tui-snap rev-parse HEAD` must equal the pinned revision.

Build the comparator's external frame validator (CHK-004 self-test roundtrip):

```sh
cargo build --release -p tuisnap --bin tuisnap
# executable: /Users/donbeave/Projects/tui-snap/target/release/tuisnap
```

### Container mount

Mount the protected tui-snap Git object database read-only at:

```text
/proof/bootstrap/tui-snap
```

The object database must contain commit `0a2e490802b7b048cd96349c6af860f8a3a05c3d`. Build source for qualification executables is extracted from that exact commit, never from a mutable working tree or moving branch ([`capture-atomicity-bootstrap-protocol.md`](evidence/capture-atomicity-bootstrap-protocol.md)).

CHK-004 driver invocation (when running self-test with frame roundtrip):

```sh
python3 /task/trusted/proof-bootstrap/proof-comparator-bootstrap.py \
  --runner /work/tools/refactor-proof/bin/tc-proof \
  --tuisnap /path/to/built/tuisnap
```

Container CI must either pre-build `tuisnap` from `/proof/bootstrap/tui-snap` and pass `--tuisnap`, or rely on comparator vectors that do not require the roundtrip (production qualification uses pinned frame fixtures; roundtrip is self-test enrichment).

---

## Pinned taskfmt install procedure

Do not use a globally installed or unpinned taskfmt binary.

### Pin values (must match after install)

| Field | Value |
| --- | --- |
| `taskfmt_revision` | `52d9f1eb7721f409bc47beb9fced7997b5c13ede` |
| `taskfmt_fingerprint` | `52c960db74b3b288ce93211c82e5703a338ba5ddfd40c92d6054ef93cfcd94e4` |

### Container layout (preferred)

Image build or init step populates:

```text
/proof/bootstrap/task-format/     # git checkout @ 52d9f1eb…
/proof/bootstrap/bin/taskfmt        # cargo install output
/proof/bootstrap/experiment.toml    # from task-format checkout
```

### Local / CI init recipe

```sh
export TASKFMT_REV=52d9f1eb7721f409bc47beb9fced7997b5c13ede
export TC_TASKFMT_SOURCE=/proof/bootstrap/task-format   # or /tmp/taskfmt-qualification locally
export TC_TASKFMT=/proof/bootstrap/bin/taskfmt          # or /tmp/taskfmt-install/bin/taskfmt locally

rm -rf "$TC_TASKFMT_SOURCE" "$(dirname "$TC_TASKFMT")"
git clone git@github.com:donbeave/task-format.git "$TC_TASKFMT_SOURCE"
git -C "$TC_TASKFMT_SOURCE" checkout "$TASKFMT_REV"
test "$(git -C "$TC_TASKFMT_SOURCE" rev-parse HEAD)" = "$TASKFMT_REV"

cargo install --locked --root "$(dirname "$(dirname "$TC_TASKFMT")")" \
  --path "$TC_TASKFMT_SOURCE/harness" --bin taskfmt

"$TC_TASKFMT" revision      # must equal $TASKFMT_REV
"$TC_TASKFMT" fingerprint   # must equal pinned fingerprint above
shasum -a 256 "$TC_TASKFMT_SOURCE/experiment.toml"   # bind EV-007 / config bytes
cp "$TC_TASKFMT_SOURCE/experiment.toml" /proof/bootstrap/experiment.toml
```

Record revision, fingerprint, and `experiment.toml` SHA-256 before any driver invocation ([`campaign-executor-protocol.md`](campaign-executor-protocol.md) EV-005–EV-007).

---

## Darwin `sandbox-exec` requirement (host matrix)

Host qualification (CHK-001 precondition self-test, CHK-005 regression, CHK-006 lint, CHK-007 gate) **requires macOS** with:

| Prerequisite | Detail |
| --- | --- |
| Platform | `sys.platform == "darwin"` |
| Sandbox | `/usr/bin/sandbox-exec` present and executable |
| Git | Direct CLT binary `/Library/Developer/CommandLineTools/usr/bin/git` (not `xcrun` shim) |
| Python | 3.10+ on a fixed PATH with CLT, `/usr/bin`, `/bin` |

The planner-owned observer (`host-bootstrap-observer.py`) wraps every submitted host invocation in a Darwin sandbox profile. Linux containers without `sandbox-exec` fail preparation closed — they cannot run CHK-005/006/007.

**CI implication:** TASK-001 host-matrix jobs must run on a **macOS** runner (or bare-metal Darwin host). Comparator-only jobs (CHK-004) may run on macOS after building tuisnap; they do not require sandbox-exec unless combined with host checks.

Observer qualification (`--observer-test`) additionally exercises real build/test/taskfmt execution, IPC isolation, and replay rejection under the same sandbox facility.

---

## verify.toml check commands (container)

All paths below assume prior `cargo build -p refactor-proof` and `sync-binaries.sh`.

```sh
# CHK-001 precondition (driver self-test + taskfmt pins; no submitted host)
python3 /task/trusted/proof-bootstrap/host-bootstrap-driver.py \
  --taskfmt /proof/bootstrap/bin/taskfmt \
  --taskfmt-source /proof/bootstrap/task-format

# CHK-004 comparator
python3 /task/trusted/proof-bootstrap/proof-comparator-bootstrap.py \
  --runner /work/tools/refactor-proof/bin/tc-proof

# CHK-005 / CHK-006 / CHK-007 host matrix (Mach-O at bin/ after sync)
python3 /task/trusted/proof-bootstrap/host-bootstrap-driver.py \
  --host /work/tools/refactor-proof/bin/tc-proof-host \
  --taskfmt /proof/bootstrap/bin/taskfmt \
  --taskfmt-source /proof/bootstrap/task-format
```

---

## CI job matrix summary

| Job | OS | Steps | Checks |
| --- | --- | --- | --- |
| `refactor-proof-build` | macOS | checkout worktree; verify tui-snap @ pin; `cargo build -p refactor-proof`; `sync-binaries.sh` | artifact: synced `bin/tc-proof*`, build log |
| `taskfmt-bootstrap` | any with Rust | pinned taskfmt install → `/proof/bootstrap/*` | revision/fingerprint/config SHA-256 |
| `task-001-comparator` | macOS | bootstrap + build artifacts | CHK-004 |
| `task-001-host` | **macOS only** | bootstrap + build artifacts + sandbox-exec | CHK-001, CHK-005, CHK-006, CHK-007 |

---

## Related documents

- [`task-001-bootstrap-plan.md`](task-001-bootstrap-plan.md) — phased implementation plan
- [`task-001-qualification-report.md`](task-001-qualification-report.md) — advisory qualification evidence
- [`task-001-phase4-operator-runbook.md`](task-001-phase4-operator-runbook.md) — IW-03 operator procedure
- [`001/verify.toml`](../../refactoring-tasks/terminal-components/completion/001/verify.toml) — machine check authority
