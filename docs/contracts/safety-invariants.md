# Safety contracts for retained unsafe operations

This page keeps source-pinned safety rationale and caller rules for the session ESC bridge and retained diagnostics. It is a contract record, not a safety audit or proof of product behavior. It does not authorize a change to unsafe code or lint policy.

## Session bridge source record

The source mapping uses the frozen product input `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`. The safety inventory was prepared at work input `c817b99e9e43abbbf3519d56b2f5ee86924243d8`. The source file is `crates/termrock-session/src/esc_bridge.rs`; its SHA-256 at the frozen product input is `2c406c21b3f9d97149bad97b5455dde72dc80439de6e81097d0503fe3b27f316`. The source API inventory contains 1,122 groups. Its SHA-256 is `6ead70836c20e3ad1c2dc15f0de44daf35d74c6507e1bbfd086de5fa6e85b619`. The safety rationale inventory contains 42 records. Its SHA-256 is `6d4173b5b8b59af2f7bde4dc680bbd0e67acb73cedaafc4823803d95684fa7c5`.

The line numbers below refer to the frozen source input. They remain historical source locations after T37 removes the comments. The 28 production rationale groups all map to unsafe operations in `esc_bridge.rs`. The 14 test-only groups are removed with their legacy test code and are not part of this production contract.

## Caller and behavior contract

The bridge preserves an ESC suffix when Crossterm 0.29 reads a double ESC in one input chunk. Crossterm can return one Escape and clear the remaining bytes. The bridge keeps the real terminal input, sends the first Escape to a side pseudo-terminal, and holds the suffix until Session receives the standalone Escape. This preserves the Esc-then-Alt+key input sequence.

A bridged terminal capability query uses a private nonblocking endpoint tied to the real terminal. A query error does not prove that the terminal lacks support. The caller declines to enable the protocol when the query fails. The bridge checks its private two-second deadline during setup and query I/O. The unbridged Session path keeps its existing reader and deadline behavior. The bridge does not fall back to that detector.

Install the bridge before any Crossterm event `poll` or `read` call. Those calls bind the process-wide reader to stdin. A later stdin swap leaves the reader on the wrong descriptor. Restore Crossterm raw mode before restoring the real terminal to stdin and restoring the saved termios. The raw-mode restore writes the side slave's saved mode to the fd that is stdin at that time.

A lone Escape remains held until another byte arrives, so the bridge does not split Alt+x into Escape and `x`. It releases a safe split only after Crossterm returns the standalone Escape to Session. Other event kinds leave the queued suffix untouched. `wait(None)` waits for an event. Discarding queued bytes affects the real terminal and does not call Crossterm.

| Contract source | Required behavior |
|---|---|
| `esc_bridge.rs:1–8` | The module description explains the double-Escape case. Keep the first Escape on the side PTY and hold the suffix until Session receives standalone Escape. |
| `esc_bridge.rs:21–21` | Historical test-only helper: the stored index identifies the second Escape in the first double-Escape pair. This helper is removed with the legacy tests. |
| `esc_bridge.rs:252–252` | The saved termios value is the real terminal state captured before the bridge switches it to raw mode. |
| `esc_bridge.rs:293–299` | A query error is inconclusive; decline protocol enablement. The bridge uses a private two-second deadline and never falls back to the unbridged detector. |
| `esc_bridge.rs:796–800` | Install before Crossterm poll/read. Those calls bind the process-wide reader to stdin; a later swap leaves it on the wrong descriptor. |
| `esc_bridge.rs:852–852` | Discard queued bytes from the real terminal only. Do not touch Crossterm. |
| `esc_bridge.rs:866–869` | Restore after Crossterm raw-mode restore so the real terminal receives the saved termios. |
| `esc_bridge.rs:881–885` | Wait for one Crossterm event or timeout; `None` waits for an event. Split a double Escape only at a known event boundary and keep its suffix until Session reads Escape. |
| `esc_bridge.rs:934–935` | Keep a lone Escape until another byte arrives so Alt+x is not split. |
| `esc_bridge.rs:970–971` | Release a safe split only after standalone Escape reaches Session. Other event kinds keep the suffix queued. |

## Production unsafe-operation rationale map

Each row identifies the original `SAFETY` comment lines and the next unsafe operation in the frozen source. The rationale text is retained from the source record. Keep each invariant synchronized with its operation.

| Comment line(s) | Unsafe line | Original rationale |
|---:|---:|---|
| 271–271 | 272 | // SAFETY: `real` is an owned duplicate of the tty stdin captured by install. |
| 564–564 | 565 | // SAFETY: `fd` is the bridge-owned tty and `path` is writable C-string storage. |
| 624–626 | 627 | // SAFETY: `path` was validated as an absolute NUL-terminated path. The<br>// flags create an independent nonblocking descriptor without acquiring a<br>// controlling tty. |
| 631–631 | 632 | // SAFETY: the successful `open` returned a new descriptor owned by this call. |
| 638–638 | 639 | // SAFETY: `stat` is valid writable storage for `fstat`. |
| 642–642 | 643 | // SAFETY: successful `fstat` initialized the structure. |
| 656–656 | 657 | // SAFETY: `endpoint_fd` is the owned descriptor created with O_NONBLOCK. |
| 689–689 | 690 | // SAFETY: `pollfd` points to one initialized descriptor owned by the probe. |
| 701–701 | 702 | // SAFETY: `query` is readable and `fd` is the owned O_NONBLOCK endpoint. |
| 711–711 | 712 | // SAFETY: `buffer` is writable and `fd` is the owned O_NONBLOCK endpoint. |
| 874–874 | 875 | // SAFETY: `bridge.real` is the dup of the tty stdin from `install`. |
| 991–991 | 992 | // SAFETY: `pollfd` points at one struct for this fd, which `install` opened. |
| 1010–1010 | 1011 | // SAFETY: `buf` is a writable buffer and `fd` is the real tty. |
| 1033–1033 | 1035 | // SAFETY: `bytes` is the caller's chunk and `fd` is the side pty master. |
| 1049–1049 | 1050 | // SAFETY: duplicating stdin. Ownership of the new descriptor is transferred below. |
| 1054–1054 | 1055 | // SAFETY: `dup` returned a new valid descriptor owned by this function. |
| 1056–1056 | 1057 | // SAFETY: `fd` is an open descriptor returned by `dup`. |
| 1072–1073 | 1074 | // SAFETY: openpty writes the two fds. Null name, termios, and size keep<br>// the implementation defaults. |
| 1086–1086 | 1087 | // SAFETY: successful `openpty` returned two new open descriptors owned here. |
| 1088–1088 | 1089 | // SAFETY: successful `openpty` returned two new open descriptors owned here. |
| 1094–1094 | 1095 | // SAFETY: `tcgetattr` fills `attrs` for this open tty fd. |
| 1105–1105 | 1106 | // SAFETY: `attrs` is a complete termios value and `fd` is an open tty. |
| 1113–1113 | 1114 | // SAFETY: `tcgetattr` fills `attrs` for this open tty fd. |
| 1121–1121 | 1122 | // SAFETY: `cfmakeraw` only rewrites the termios flags in `attrs`. |
| 1123–1123 | 1124 | // SAFETY: `attrs` came from `tcgetattr` on this same fd. |
| 1131–1131 | 1132 | // SAFETY: `fd` is one of the bridge fds just opened. |
| 1136–1136 | 1137 | // SAFETY: same fd, setting the close-on-exec bit only. |
| 1144–1144 | 1145 | // SAFETY: `fd` is the open side-pty slave descriptor owned by `install`. |

## Retained Unix process-diagnostic operations

The following locations refer to frozen work input `c817b99e9e43abbbf3519d56b2f5ee86924243d8`. The hashes bind the full source files at that input. The line numbers are historical. The maintained rules below describe source preconditions. They do not prove that a caller meets them or that the operation is safe on every supported platform.

| Source and SHA-256 | Source lines | Maintained rule and limit |
|---|---:|---|
| `crates/termrock-visibility-tests/src/lib.rs` — `a964a9c0cf495b542eab8f763f339f23d576c50d339728ec941b58666a4852c0` | 592–602 | `set_nonblocking` borrows a descriptor from a live pipe `File`. `F_GETFL` must succeed. `F_SETFL` uses those flags with `O_NONBLOCK` added. The call does not transfer the descriptor. The source sets these flags before it moves the pipe into its reader or writer thread. |
| Same file and hash | 650–668 | The child check gives `waitid` a live, writable `siginfo_t` that the source zero-initializes. It uses `P_PID`, `WEXITED`, `WNOHANG`, and `WNOWAIT`. Read `si_pid` only after `waitid` succeeds; zero means that no event was reported. `WNOWAIT` leaves the child waitable. An unrelated reaper can still race with this check. This record does not prove exclusive child ownership. |
| Same file and hash | 324–330, 696–700 | Before spawning, the source requests `process_group(0)`. It then passes the negative child ID to `kill` to signal that process group. The helper ignores the `kill` result. Later `Child::wait` handles the direct child; it does not prove that every descendant exited. This record does not prove group membership or exclusive process ownership. |
| `crates/termrock-visibility-tests/src/bin/capture-command.rs` — `6cd9c64d85dcda7392ba234f06f54af816f3859b0147759e0420df94c9de0867` | 473–490 | The signal installer zero-initializes a live local `sigaction`, sets the handler and flags, initializes `sa_mask` with `sigemptyset`, and checks each result. The handler only records a signal in `AtomicI32`. The mask and action remain live while libc reads them. This record does not prove platform ABI or signal-delivery behavior. |
| `tools/refactor-proof/src/observer/client.rs` — `903ecfadb486a388239726e07e450c00a66f1b1fda650deb0b6b89184d1db7c7` | 41–60, 163–168 | The host must supply two live, valid, distinct inherited pipe descriptors and transfer exclusive ownership of each to `ObserverClient` exactly once. `OwnedFd::from_raw_fd` takes ownership; the `File` values close the descriptors when dropped. Environment parsing checks integer syntax and a nonempty nonce. It does not check that a descriptor is open, that the two descriptors differ, or that ownership was transferred exclusively. The current source caller found at this input is `tools/refactor-proof/src/host/verify.rs:88`; no current producer is established. Keep those host conditions as an open assumption. The older fixture-lifetime statement in `SAFETY.md` is historical and does not establish these conditions for this client. |

Keep this unresolved obligation as `T20-OBS-FD-01`. The maintained [observer transport note](../../tools/refactor-proof/observer-transport.md) records the open producer and ownership questions. T01 must inventory them. T38 must approve public-path cases before test edits. T20 owns the bounded client and supported-launcher contract extension. It must decide whether the launcher transfers or lends each endpoint, how it enforces distinct ownership and one-time construction, and how it handles a probe followed by construction. T06 tracks the existing lint scope. T36 must check the final source and evidence. A parseable descriptor number or an open descriptor does not prove exclusive ownership. Do not claim that environment values enforce these rules.

Before T20 closes this item, the design and approved plan must cover a valid request/response pipe pair and nonce; absent, malformed, negative, closed, out-of-range, or equal descriptor values; partial endpoint acquisition; repeated construction; and `require_transport` followed by construction. They must also cover unrelated open descriptors, stale environment values, unsupported platforms, and a transport-kind mismatch. The supported launcher must define whether it transfers or lends each endpoint, and the client must enforce the agreed contract before converting raw descriptors to owners. These are future review obligations, not T37 tests or passing cases. Behavior remains `NOT VERIFIED`.

The source record for retained and retired visibility safety rationale is [`t37-tools/safety-rationale.json`](../implementation/pr38-final/results/t37-tools/safety-rationale.json), SHA-256 `4dd097c5697b9b6757f855aa48bd2b9a2fdeb201a536a8a14e650712d4bcfebf`. It binds the source input, file hashes, original line text, and current caller. Its retained-operation map is evidence of source locations, not fresh platform, signal, process-ownership, or descriptor-ownership proof. The observer module's existing `unsafe_code` expectation and the existing `refactor-proof` lint policy remain unchanged.

## Maintenance rules

- Keep `[workspace.lints.rust] unsafe_code = "deny"` and the existing `undocumented_unsafe_blocks` policy. Do not add or widen a lint allowance.
- Sol high decision `T37-SAFETY-01` found no active `undocumented_unsafe_blocks` conflict in the foundation manifests. Keep the current lint policy. Stop and request a new scoped Sol high decision if another active lint conflict appears.
- Keep the source mapping and rationale when the corresponding operation remains. Update the source commit, file hash, and line map when the operation moves or changes.
- This page maps nine production `esc_bridge.rs` API groups that define the bridge contract and one historical test-only helper group. The foundation source API inventory is broader. The bounded caller contracts selected for T37 are in [authoring guidance](authoring.md#runtime-routing-and-layer-receipts), with source hashes and line spans retained in [`source-api-prose.json`](../../docs/implementation/pr38-final/results/t37-foundations/source-api-prose.json). That archive is provenance, not maintained guidance or per-group semantic equivalence. T01 must classify the cleaned-source API obligations; T35 must publish current caller guidance.
- The safety inventory also records test-only allocator, fixture process-termination, and visibility queue-lock operations retired with their legacy tests. Their rationale is historical evidence, not an active production guarantee. Retained diagnostic rationale is mapped above before T37 removes its inline comments.
- T37 runs no replacement tests. Product behavior remains `NOT VERIFIED` until an approved later task proves it through real production paths.

## Evidence limits

The inventory records source comments and source hashes. It does not prove that the current implementation is safe or that the described behavior passed a test. T37 must preserve the operations and contracts. A later safety review must inspect the exact implementation at its reviewed source revision.

Finding F17 and task T20 remain open. The source record does not establish one exclusive owner for the session lifetime. Cleanup after a session ends can still run from the panic hook. Restore consumes saved state before `dup2` succeeds. Keep these as open source risks and recheck them in T20. This contract does not certify them as resolved.
