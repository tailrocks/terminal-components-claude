# Safe Unix session broker feasibility qualifier

This is disposable planning evidence, not Terminal Components production code or a task completion receipt.
It tests whether the TASK-009 safe signal-hook/process-lifetime design can work with real crossterm waits on an owned PTY.
The supported process starts with default SIGTSTP handling and installs no competing SIGTSTP registration.
It does not promise restoration of an arbitrary previous sigaction.

## Source and environment

- Architectural source: `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- Lifecycle oracle: `02f5294bfdbf38004cc49130d0aff1d01f31434c:src/runtime.rs` and `tests/terminal_suspend.rs`.
- Actual run: Darwin 25.6.0, rustc 1.98.0 (88d9e12ae 2026-08-18).
- Rust fixture forbids unsafe code. It calls safe signal-hook 0.3.18 and crossterm 0.29.0 APIs.
- Every registry package name/version in this qualifier's lock also occurs in the pinned main lock, including mio 1.2.2, bitflags 2.13.1 and signal-hook-registry 1.4.8.
- The Python supervisor uses standard POSIX PTY/process APIs only on its own descriptors and child processes. It never changes the user's terminal.

The broker installs a pending flag plus conditional default emulation once for the process.
A serialized session lease changes to deferred handling before any mode acquisition.
Restoration precedes default stop emulation; re-entry follows continuation and reads the new PTY size.
Outside a session, conditional default emulation retains ordinary stopping.
No unregister operation is used in the positive path: unregister alone would leave stopping swallowed.
Default emulation reports SIGSTOP on this platform; equality with the oracle's kernel-reported stop signal is not claimed.

## Reproduction

Use this manifest as an independent fixture and a disposable target directory; do not add it to the production workspace.

```sh
rtk cargo build --offline --locked --manifest-path docs/refactoring-plan/evidence/session-broker-bootstrap-20260911/Cargo.toml --target-dir /tmp/tc-broker-proof.uBbtHa/target
rtk proxy python3 -B docs/refactoring-plan/evidence/session-broker-bootstrap-20260911/pty_probe.py /tmp/tc-broker-proof.uBbtHa/target/debug/tc-session-broker-feasibility
rtk proxy python3 -B -O docs/refactoring-plan/evidence/session-broker-bootstrap-20260911/pty_probe.py /tmp/tc-broker-proof.uBbtHa/target/debug/tc-session-broker-feasibility
rtk cargo test --offline --locked --all-targets --manifest-path docs/refactoring-plan/evidence/session-broker-bootstrap-20260911/Cargo.toml --target-dir /tmp/tc-broker-proof.uBbtHa/target
rtk cargo clippy --offline --locked --all-targets --manifest-path docs/refactoring-plan/evidence/session-broker-bootstrap-20260911/Cargo.toml --target-dir /tmp/tc-broker-proof.uBbtHa/target -- -D warnings
rtk cargo fmt --check --manifest-path docs/refactoring-plan/evidence/session-broker-bootstrap-20260911/Cargo.toml
```

The target path above records the actual disposable directory used; another fresh `mktemp -d` target is equivalent.
No doctest target exists because this fixture is binary-only; the attempted `cargo test --doc` correctly reported that limitation and is not counted as passing doctests.
Nextest was not needed for two deterministic unit tests plus a custom owned-PTY supervisor.

## Observed results

The final normal campaign in `campaign.json` and optimized-Python campaign in `campaign-optimized.json` each pass the positive path and reject all four actual Rust execution variants.
Every required Python observation uses an explicit `require` function that raises `AssertionError`; Python optimization cannot erase termios, mode-balance, logical-work or expected-rejection checks.
The positive path covers an injected failure after bracketed-paste setup, exact cleanup, external stop after that failed setup, two idle active-session stops with resumed sizes 100×30 and 80×24, two successful sessions, an external stop between sessions and another after the last session.
Every stopped/final termios snapshot equals the launch snapshot.
Alternate-screen and bracketed-paste entry/exit counts balance at five each.
No terminal output appears during the 350-ms idle observation, despite at least two service waits.
The fixture's logical Tick/Settle counters remain zero.

| Negative variant | Real altered behavior | Observed rejection |
| --- | --- | --- |
| `block-read` | Skip bounded poll and wait in crossterm read | External idle stop times out without keyboard input |
| `dirty-stop` | Stop before restoring acquired modes | Stopped termios differs from launch state |
| `idle-tick` | Deliver synthetic logical work from a service wake | Nonzero logical-work counter |
| `unregister` | Remove both callbacks after leaving a session | Post-session external stop is swallowed and times out |

`repeat-20.json` records twenty additional fresh-process positive executions, five external stops each.
The service quantum is 100 ms, not a 100-ms OS scheduling promise; observed stop latency can exceed 100 ms.
Both active stops remain under the oracle's 10-second hard safety bound.
The two Rust tests independently check exact due/overdue wait budgets at now=100 with deadlines 99/100/101/199/200/201, and a fixed original deadline across 1000 admitted input observations.
Build, both tests, strict Clippy and rustfmt check passed on the persisted source.

During probe development an unbarriered initial termios assertion failed once and a Python regex-group assertion was incorrect.
Those development attempts are not passing evidence.
The final harness adds an explicit foreground-ready barrier before terminal setup and checks launch termios both before and after that setup; the corrected final campaigns/repetitions are the recorded evidence.
The first transient's exact OS cause was not established, so it is not attributed to a proven production defect or silently omitted.

## Boundaries not proved

This proves a feasible safe dependency/protocol on macOS, not full TASK-009 acceptance.
Independent review additionally built Rust 1.98.0 and Rust 1.88.0/MSRV executables in fresh directories, passed the two unit tests under MSRV, and passed ordinary-Python/stable plus optimized-Python/MSRV PTY campaigns with all four negatives rejected. The exact executed matrix and hashes are recorded in `../../reaudit-session-broker-rereview.md`; an unexecuted four-way cross-product is not claimed.
This qualifier does not test Linux, backend-free feature isolation, arbitrary signal handlers, concurrent OS registration changes, panic-hook chaining, process shutdown races or all signal-arrival interleavings.
Setup failure is injected after a successful operation; actual failing terminal writes, failed raw-mode restoration, cleanup retry and failed re-entry recovery remain TASK-009 obligations.
In particular the small prototype's restoration mask is consumed even after an I/O error; it must not be copied as a production cleanup-retry implementation.
The fake-clock helper and synthetic logical-work counters are not the production Runtime deadline/feedback dispatch; runtime fairness, no idle Tick/Settle, Bootstrap order and simulation policy still require their real production witnesses.
Resumed size/mode evidence is not an application frame/cursor parity claim.
The proof does not freeze or bless any baseline, mutate a production source file, or authorize execution of the refactoring campaign.

The Rust correctness skill influenced the safe dependency boundary, explicit ownership/failure model, isolated negative paths, strict lint/test checks and these measured-claim limits.
Independent review accepted this bounded macOS/MSRV feasibility evidence in `../../reaudit-session-broker-rereview.md`, while retaining every production-obligation boundary above.
