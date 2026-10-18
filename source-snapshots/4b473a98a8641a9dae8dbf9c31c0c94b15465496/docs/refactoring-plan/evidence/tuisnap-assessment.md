# tui-snap capability evidence, 2026-09-11

## Source identity

Remote `donbeave/tui-snap` main: `5036cf87e621e6beb66deffe3224abdbefc955cb` (queried live with `git ls-remote`). Initially no GitHub PRs in any state and no remote branch other than main. Existing terminal-components main bundles these repaired commits:

- `7d5d62cd44d430ff5d2fc4006cc776b30ee487e1`: canonical terminal attributes and geometry.
- `02102e943cf933ce4acf56aae8c25622a31fda0f`: independently reviewed, reference-derived schema-3 migration of the tool's 24 approvals.
- `e45d3fae2ecc628e294c0b5796a8775ad2d7f0e2`: physical cursor coordinate while preserving pending wrap.

All three retain DCO and Codex trailers. Worktrees were isolated under `/tmp/tui-snap-audit.656lHG`; no terminal-components production code was edited. No tui-snap repository AGENTS.md or CLAUDE.md was present in the inspected source. Live verification used Rust 1.98.0 (`88d9e12ae178fab0fb5cc050a94da85685d449ea`), `aarch64-apple-darwin`, Darwin arm64. This is distinct from the historical qualification's recorded Rust 1.98.1; current outcomes below are new evidence.

## Concrete upstream failures

Upstream's existing suite passes 38 tests. Two independently authored regression probes fail at upstream main:

```text
DIM foreground RGB(255,255,255), background RGB(0,0,0):
  actual RGB(25,25,25), expected RGB(153,153,153)
Raw ANSI wide glyph 界 on background RGB(4,5,6):
  continuation cell background Default, expected RGB(4,5,6)
0 passed; 2 failed
```

The first defect narrows the weighted color sum to `u8` before dividing by ten. The second constructs a blank continuation cell without copying its style. Both defects undermine exact visual evidence for fades, styled wide text and reverse/selection backgrounds. Upstream frame schema also omits hidden/blink flags; raw replay loses strike and bold+DIM combination. These are verified source/API deficiencies, not missing high-level mouse/keyboard/resize helpers.

Minimal upstream reproductions use existing APIs:

```rust
let mut c = tuisnap::Cell::blank(0, 0);
c.fg = tuisnap::Color::Rgb(tuisnap::Rgb::new(255,255,255));
c.bg = tuisnap::Color::Rgb(tuisnap::Rgb::new(0,0,0));
c.mods.dim = true;
assert_eq!(tuisnap::Frame::resolve_cell(&c,
    tuisnap::Rgb::new(0,0,0), tuisnap::Rgb::new(0,0,0)).0,
    tuisnap::Rgb::new(153,153,153));
let frame = tuisnap::ansi::replay_raw(
    "\x1b[48;2;4;5;6m界".as_bytes(), 8, 3, 0,
    tuisnap::Provenance::now("probe", "ansi", vec![])).unwrap();
assert_eq!(frame.get(1,0).unwrap().bg,
    tuisnap::Color::Rgb(tuisnap::Rgb::new(4,5,6)));
```

## Rerun results at repaired e45d3fa

| Exact command | Observed result |
| --- | --- |
| `rtk cargo build --examples --locked` | Pass. This prerequisite is required before the PTY test uses `fixture_app`. |
| `rtk cargo test --locked --all-targets --all-features` | 49 tests pass, 9 suites, 140.74 s. |
| `rtk cargo test --locked --no-default-features` | 40 pass across integration suites and doctest, 139.08 s. |
| `rtk cargo test --locked --doc --all-features` | 1 doctest passes. |
| `rtk cargo fmt --all -- --check` | Pass. |
| `rtk cargo clippy --locked --all-targets --all-features -- -D warnings` | Pass. |
| `rtk cargo test --locked --test tool_qualification` | 11 tests pass. |
| `rtk proxy python3 -m unittest discover -s tools/qualified-capture/tests -v` (terminal-components main) | 2 integrity tests pass: payload hashes and missing/tampered repair rejection before acquisition. |

An initial all-target test invocation failed because the fixture executable had not yet been built. Building examples and rerunning resolved that documented prerequisite; no assertion or baseline was changed.

`tests/tool_qualification.rs` checks exhaustive 65,536 DIM color pairs, hidden rendering, modifier/continuation retention in both adapters, raw ANSI autowrap, formatted flag transitions, schema-2 rejection, literal LF paste, wide clipping and pending-wrap cursor behavior. Existing snapshot suites separately prove missing/corrupt approvals and mutation failure, deterministic PNG regeneration and cursor/style differences.

## Independent PTY smoke

The portable source [tuisnap-pty-probe.rs](tuisnap-pty-probe.rs) was executed against repaired e45d3fa, using the independently authored existing `tools/qualified-capture/fixture.py` and `expected-cells.json` from pinned terminal-components main. It compares every field in all 384 cells; it does not derive expected cells from tool output. It then verifies the full cursor, clicks/drags, raw SGR hover and wheel transport, literal multiline paste including a wide character, resize, style-aware settling and successful exit.

Observed result:

```text
PASS: 384 independent cells; all modifiers, wide styles, clipping, cursor;
click/drag/hover/wheel transport; literal LF paste; resize;
style-aware settle; child exit
```

To reproduce, create an external Cargo harness with this source as `src/main.rs`, depend on the final pinned `tuisnap` checkout and `serde_json = "1"`, and pass the qualified-capture directory and an existing isolated output directory as the two arguments. Final head `883d03f` needs no Cargo root patch. The initial e45d3fa run required a root `[patch.crates-io]` entry; that packaging defect was corrected after fresh review and the same probe passed again without it. The probe lock was newly resolved, so repaired-repository `--locked` tests remain the authority for the release dependency closure.

PTY transport proof does not claim application action semantics. The application scenarios must additionally assert focus, edits, scroll routing, hitboxes, overlay capture and state outcomes after these inputs.

## Publication status

Published dependency: [donbeave/tui-snap PR #1](https://github.com/donbeave/tui-snap/pull/1), branch `codex/qualification-repairs`, head `883d03f19d890bbbf27468798db78b04e85297ac`, tree `dadbaa70facc317cfabb52f0374c1f3cdceb46a1`, base `5036cf87e621e6beb66deffe3224abdbefc955cb`. The PR is open and unmerged. Pin this exact reviewed head for reference capture and implementation gates; do not follow a moving branch or silently fall back to upstream schema 2. Fresh review is recorded in [tuisnap-review.md](../tuisnap-review.md). All four material findings were fixed, then independently rechecked before publication.

## Independent-review corrections

The first fresh review found four material issues and verified the original 24-frame migration independently. All are accepted for correction in the dedicated tool branch:

1. Ordinary downstream Cargo consumption ignored the dependency-local vt100 patch and failed with six missing-method errors. The correction wires both tuisnap and an unchanged, licensed termlens 0.9 source copy to the same vendored vt100 by direct paths. `tuisnap::termlens` exposes the matching engine type identity. The new `tests/fixtures/consumer` uses no root patch; its build, execution and single-engine dependency tree pass. A future crates.io release needs separately published fork packages; current required Git/path consumption is complete.
2. Vendor `state_diff`/`state_formatted` painted wrapped rows before enabling DECAWM, losing `IJ` from a width-eight `ABCDEFGHIJ` replay when the destination disabled wrap. Content serialization now enables wrap before painting and restores the target mode afterward. New tests exercise full/delta and enabled/disabled target modes.
3. Hidden glyph replacement inserted spaces into SVG text without preserving whitespace, shifting visible text. SVG text nodes now specify `xml:space="preserve"`; a hidden-prefix/interior regression test verifies the geometry-preserving run.
4. Migration read current schema-3 approvals and therefore could not reproduce its own documented command; `assert` guards vanished under Python `-O`. It now reads only pinned upstream Git blobs, validates with explicit errors, stages a complete output, rejects existing paths and records source revision. `tools/test_migration.py` runs under `-O`, reproduces all 24 existing approved hashes, rejects replacement/source drift and verifies approvals unchanged.

Corrections are committed as `883d03f19d890bbbf27468798db78b04e85297ac` with DCO and Codex trailers. The worktree is clean. The new targeted suite passes 13 tests; the independent 384-cell PTY probe passes again without any root patch. Repeated final gates pass: 51 all-feature/all-target tests (138.72 s), 41 no-default tests including doctest (137.92 s), one explicit doctest, formatting, clippy with `-D warnings`, ordinary consumer execution, migration rejection/hash tests and `git diff --check`. No application baseline or existing tool approval changed during these corrections.
