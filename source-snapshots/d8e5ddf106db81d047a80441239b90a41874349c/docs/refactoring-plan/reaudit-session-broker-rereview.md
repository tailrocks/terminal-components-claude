# Independent safe session-broker feasibility review

The coordinator independently read the complete persisted README, Rust program, PTY supervisor and Cargo manifest for `evidence/session-broker-bootstrap-20260911`. This is bounded planning feasibility evidence, not a production implementation, replacement oracle or TASK-009 acceptance. Verify-and-stop confined this review to the stated signal protocol and proof claims.

## Finding and repair

The first Python supervisor used optimization-strippable `assert` for termios equality, mode counts, logical-work absence and expected negative rejection classes. Ambient Python optimization could remove required observations even without an explicit `-O` in the documented command. This is an observation-integrity defect; no claim is made that the entire four-mutant campaign necessarily returned success under optimization, because its explicit `mutant survived` raise remained.

The author replaced each required observation with explicit `require`/`AssertionError`, including supervisor child ownership and expected negative reason. The coordinator reread every changed check and independently ran the corrected campaign in ordinary and optimized Python. Both retained the actual positive and all four distinct negative rejections. No product source was changed.

## Fresh independent execution

Fresh output directories: `/tmp/session-broker-rereview.QaKnn9/stable` and `/tmp/session-broker-rereview.QaKnn9/msrv`. The actual persisted fixture was built offline and locked using Rust 1.98.0 and installed Rust 1.88.0. Both builds passed. The two deterministic due/overdue and non-rebased-deadline unit tests additionally passed under 1.88.0.

The coordinator parsed the fixture and actual pinned main Cargo locks independently. Every one of the fixture's 37 registry package name/version/checksum triples occurs in `7b27732a8c3c131760ec3438f641cb3c11343a42:Cargo.lock`. This is exact dependency identity, not merely matching names or a fresh resolver's preferred versions.

Actual PTY campaigns:

| Build / Python mode | Positive | Required negative results | Five measured stop latencies, milliseconds |
| --- | --- | --- | --- |
| Rust 1.98.0 / ordinary | Pass | block-read timeout; dirty-stop termios mismatch; idle-tick logical-work mismatch; unregister post-session timeout | 0.062, 40.928, 102.061, 0.049, 0.029 |
| Rust 1.88.0 / `-O` | Pass | Same four exact rejection classes | 0.063, 55.803, 100.225, 0.031, 0.031 |

Commands used `rtk cargo build` / `rtk proxy cargo +1.88.0 build` and `cargo +1.88.0 test`, all with `--offline --locked --manifest-path docs/refactoring-plan/evidence/session-broker-bootstrap-20260911/Cargo.toml` and the explicit disposable target directories above. PTY commands were `rtk proxy python3 -B [-O] docs/refactoring-plan/evidence/session-broker-bootstrap-20260911/pty_probe.py <fresh-binary>` with the exact matrix shown, not an unexecuted four-way cross-product.

Each positive performed five real external stops around failed setup, active idle sessions, continuation/resize and post-session default behavior. The supervisor measured stop state using its own child and PTY, compared actual termios, and checked balanced alternate-screen/bracketed-paste sequences. No user's terminal or unrelated process was targeted. The measured values above 100 ms confirm why the service budget must not be advertised as a 100-ms OS scheduling guarantee. They remain within the oracle's separate ten-second stop safety bound.

## Exact build identities

| Input/output | SHA-256 |
| --- | --- |
| `src/main.rs` | `827cc69cc3679b9f6175063ad25cf68a273d26aee1579a5523d22b760c6d8f42` |
| `Cargo.toml` | `031941e8adf6a6e5567aa834d08d1a9c045434e0e158ef53f176a2577f377e73` |
| `Cargo.lock` | `61a852911931655b9a08ea8eac48ad18e0b52aa46f8f0688e13a65f7e975f71b` |
| Assert-free `pty_probe.py` | `52f7df0b152c9094861c6257039d2318aaa057d835ec1da4026e939917267726` |
| Fresh Rust 1.98.0 binary | `acbdaf7e348a9a963c81a1bcaeb94e8129ae8b3d69d4c0f2398d4e3a4d78fb4a` |
| Fresh Rust 1.88.0 binary | `74bccd2e21ddc5c2daf04056940c8c7ca1f7f0b4da4d5a06671508a6b4f0736f` |

## Accepted boundary and remaining obligations

The safe `signal-hook=0.3.18` process-lifetime broker with bounded crossterm service polling is feasible on the measured macOS platform, including an actual MSRV-built executable. The positive source neither unregisters after each session nor assumes unregister restores a handler. It defers active-session stop until cleanup and preserves ordinary stopping outside a session under the explicit initial-default/no-competing-registration process contract.

The fixture deliberately does not claim real Runtime fairness, feedback/deadline delivery, application frame/cursor parity, backend-free feature isolation, Linux behavior, arbitrary prior handler restoration, panic-hook chaining, shutdown races, all signal interleavings or actual I/O-error cleanup/re-entry recovery. Its restoration mask consumes failed cleanup operations, as the author explicitly documents; it is not approved as production cleanup-retry code. Logical-work counters and the small wait-budget helper are fixture observations, not proof that the future TC Runtime dispatch is correct. TASK-009 must satisfy those remaining source-qualified production witnesses. The author's twenty repetitions are additional author evidence, not twenty independent coordinator runs.
