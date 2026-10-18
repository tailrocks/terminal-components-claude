# Qualified capture tooling

Reproduce the reviewed capture tools outside this repository's Rust MSRV build.
This package qualifies terminal tooling against an independent ANSI fixture; it
does not establish application parity or approve application snapshots.

1. Install Rust **1.98.1**, Zig **0.16.0**, Git, Python 3.10+ and the platform's
   native C/C++ build tools. A Unix PTY is required. Qualification was executed on
   macOS arm64; other platforms must run these same checks before claiming support.
2. Run from this repository, with Zig 0.16.0 on `PATH`:

   ```sh
   rtk proxy python3 -m unittest discover -s tools/qualified-capture/tests -v
   rtk proxy python3 tools/qualified-capture/acquire.py --root /tmp/qualified-capture-new
   ```

   With mise, use `rtk proxy mise exec zig@0.16.0 -- python3
   tools/qualified-capture/acquire.py --root /tmp/qualified-capture-new`.
   The output directory must not exist. Network access to GitHub and Cargo
   dependencies is required; this is exact source acquisition, not an offline
   dependency mirror.
3. Require `commands.json` to report `status: complete`. It binds every command,
   exit code, source commit/tree, compiler, generated binary hash and raw smoke
   artifact hash. Failure remains `failed`; a partial build cannot claim success.

`pins.json` is the versioned trust root. Every bundled repair, proof, fixture and
harness payload is checked before network access or execution. Acquisition checks
both upstream and final Git trees, exact commits, locked dependency hashes, and
clean source status after building. Cargo always uses `+1.98.1 --locked`, external
target directories and an external harness with relative source dependencies.
No application manifest, toolchain selection or CI configuration is changed.
Inherited color suppression and Rust build overrides are removed and their names
recorded. Ambient Cargo configuration, native toolchains and platform libraries
can still affect binaries; per-build hashes are evidence, not a promise of
cross-platform bit-identical debug executables.

## Source and review provenance

| Tool | Upstream commit | Qualified commit |
| --- | --- | --- |
| tui-snap | `5036cf87e621e6beb66deffe3224abdbefc955cb` | `e45d3fae2ecc628e294c0b5796a8775ad2d7f0e2` |
| tui-test | `2d5f8020ba6eb77b9c4da38892c2f6f200311b90` | unchanged |

`repairs/tuisnap.bundle` contains three reviewed commits, preserving author,
signoff and Codex trailers. The prerequisite is the exact upstream commit above.
The compact Git bundle is necessary because the equivalent mail patch is about
51 MB of snapshot JSON; the bundle is about 261 KB. The readable
`repairs/tuisnap-code.patch` contains the complete source repair, excluding only
approved visual fixture files. Those fixture changes remain in the bundle and
their exact old/new hashes are in `proof/migration-reviewed-hashes.json`.

The repairs preserve bold+DIM, continuation-cell styles, hidden/blink fields,
DECAWM clipping and physical cursor coordinates; fix DIM arithmetic; and add
literal bracketed paste with LF preservation. Schema 3 intentionally rejects old
schema 2 frames. The fixture migration was reconstructed from existing approvals
and reviewed independently; it was not generated from application captures.
The 24 tool fixtures have explicit review provenance in
`proof/migration-review.md` and `proof/cursor-review.md`.

Copies of the upstream tool and vendored-engine licenses are in `licenses/`.
Upstream license files also remain in each acquired tree.
The vendored vt100 0.16.2 source retains its LICENSE. Its original crate URL,
archive SHA-256 and original per-file hashes are in
`repairs/vt100-upstream.json`; the final qualified Git tree binds the modifications.
No third-party binary is redistributed. Historical command logs retain their
original absolute paths as evidence only; no runtime command depends on them.

`proof/final-tool-gates.json`, `full-*.log` and `test-identities.json` bind the
previous full tool matrix: 49 all-feature/all-target tests, 39 backend-free tests,
one doctest, formatting, clippy, and both visual suites covering all 24 approvals.
The clean acquisition command builds both tools again and executes the portable
smoke described below. It does not substitute a smoke run for those full gates.

## Qualification and explicit limits

`fixture.py` and `expected-cells.json` predate repaired captures. The independent
oracle reconstructs and checks all 384 cells: RGB, ANSI-256, ANSI-16, default
colors, bold, DIM, reverse, underline, hidden, blink, strike, wide and combining
text, clipping, cursor, style-only frame settling, zero-based mouse coordinates,
drag, resize, literal LF paste and terminal ECHO/ICANON restoration. Five injected
cell mutations and a live impossible text assertion must fail. Missing/tampered
bundle and readable patch tests must fail before any acquisition directory exists.

Repaired tui-snap must have zero missing fields or cell differences. tui-test's
complementary oracle permits exactly its observed blink loss and absent
width/continuation fields; any additional difference fails. Raw captures are
retained without removing those losses. tui-test cursor CLI exposes only x/y,
not visibility, shape or blinking. Its direct-run exit-code assertion lacks
command metadata; the smoke instead checks `wait exit` and `state.exited == 0`.
Its CLI has no high-level paste command; the smoke writes bracketed bytes.

Neither engine proves every terminal mode. Raw ANSI replay in tui-snap does not
preserve cursor appearance. Blink phase/rate is not animated; canonical blink is
a boolean. Hidden text remains in canonical frames and is not redacted, although
rendered glyphs are omitted. Ordinary `Session.paste` retains termlens newline
conversion; only `paste_literal` proves LF preservation and requires bracketed
paste mode. Wide glyphs that cannot fit with autowrap disabled are suppressed.
Exact termios restoration is not claimed: the oracle records all lflag changes,
including macOS PENDIN, while requiring ECHO/ICANON restoration. The tui-test
daemon sequence runs inside one orchestrator process so tool-call teardown cannot
invalidate its lifecycle evidence.
