# Feedback for tuiscotti: findings from a 7,550-scenario consumer

You are working on **tuiscotti** (<https://github.com/tailrocks/tuiscotti>).
A downstream consumer ported its whole visual suite to tuiscotti and hit the
problems below. This prompt contains everything you need. You have no
access to the conversation that produced it.

## 1. Context: who we are and how we use tuiscotti

- Consumer: <https://github.com/tailrocks/terminal-components-claude>, branch
  `design/interaction-polish`, a ratatui TUI component library with four
  binaries. We pin tuiscotti as a dev-dependency to `rev =
  "a47c9aaefb34e4c00026f99d8a8dd7ee5916b274"` (our `Cargo.toml:54`; this is
  `main` as of 2026-10-16). Every file:line below is at that rev. Re-check
  them against current `main`.
- Our own deps: `ratatui = { version = "0.30", features = ["crossterm_0_29"] }`,
  `unicode-width = "0.2"`.
- Usage, Workflow 2 as a library (inspect `tests/visual_baseline/`):
  `tui::Tui::new([CARGO_BIN_EXE_<bin>, args…])` with `env_clear(true)` and
  an explicit environment, then scripted keys, mouse and resize, a content
  wait, `wait_stable_quiet(400 ms)`, and `render::frame_from_screen`. Each
  scenario is one nextest test (`harness = false`, libtest-mimic): about
  11,000 tests in the full tier and about 890 in the per-PR smoke tier.
- Files that work around tuiscotti, which we want to delete:
  - `tests/visual_baseline/support.rs`: `gate()` (around line 280) replaces
    `GroupedStore::check_with`. The `COMMITTED` set holds `.ansi` only.
    `write_expected_evidence` rebuilds expected PNG/HTML from `.ansi` on a
    mismatch. `Session::send_key` (around line 529) has a lone-`f`
    workaround.
  - `tests/visual_baseline/ansi_cursor.rs`: our `encode`/`decode` for
    `ansi_dump` plus a cursor trailer.
  - `xtask/src/main.rs`: `cargo xtask visual accept|list|render|import`,
    because the CLI cannot accept grouped `.ansi`-only stores.
- Docs: `docs/visual-testing/tuiscotti-migration.md` (measurements and the
  upstream list), `docs/visual-testing/plan.md` §G (U1–U8).

## 2. Problems, in priority order

### P1. A lone `f` or `F` key is rejected as a malformed function key

`crates/tuiscotti-runtime/src/tui/input_types.rs:165-169`:
`name.strip_prefix('F').or_else(|| name.strip_prefix('f'))` matches `"f"`
with `rest = ""`, and the result is `bad function key "f"`. Any one-letter
name starting with f/F, and any `f…` word, is routed to the function-key
branch before the single-char branch at `:170`. Fix: treat `F<1..=12>`
as a function key only when `rest` is non-empty and all digits. Otherwise
fall through to the single-char and `unknown key` branches, and keep the
error for `F0` and `F13`. Add tests for `f`, `F`, `ctrl+f`, `shift+F`, `F1`,
`F12`, `F13`, `fx`.

### P2. Exact dependency pins force consumers' lockfiles

Root `Cargo.toml:22` `crossterm = "=0.28.1"`, `:31` `ratatui = "=0.29.0"`,
`:32` `unicode-width = "=0.2.0"`. Effect in our `Cargo.lock`:
`unicode-width` is held at 0.2.0 (down from 0.2.2), and the dev graph
carries crossterm 0.28.1 + 0.29.0 and ratatui 0.29.0 + 0.30.2. Desired:
compatible ranges (`^`). Determinism stays guarded by your committed
`Cargo.lock`, golden tests and a CI lane on minimal and latest versions.
Where ratatui types are public API, consider a feature per ratatui major
version, or an isolated adapter crate.

Evidence from our dependency upgrade (2026-10-16, `docs/dependencies.md`),
`cargo update -p <crate> --precise <latest> --dry-run` at this rev:

| Crate | Held at | Latest | tuiscotti pin |
|---|---|---|---|
| `unicode-width` | 0.2.0 | 0.2.2 | `Cargo.toml:32` `=0.2.0` (via `tuiscotti-core`) |
| `insta` | 1.48.0 | 1.49.0 | `Cargo.toml:27` `=1.48.0` (via `tuiscotti-insta`) |
| `serde_json` | 1.0.151 | 1.0.154 | `Cargo.toml:29` `=1.0.151` (via `tuiscotti-core`) |
| `sha2` | 0.11.0 | 0.11.1 | `Cargo.toml:30` `=0.11.0` (via `tuiscotti-insta`) |
| `ratatui` | 0.29.0 (second copy) | 0.30.2 | `Cargo.toml:31` `=0.29.0` |
| `crossterm` | 0.28.1 (second copy) | 0.29.0 | `Cargo.toml:22` `=0.28.1` |

`unicode-width` is the one that reaches our shipped binaries: Cargo keeps a
single 0.2.x in the lockfile, so the library and all four apps measure
text with the 0.2.0 tables only because a dev-dependency pins them.

### P3. Minimal artifact mode: one `.ansi` plus a cursor trailer per scenario

`GroupedStore` always reads, renders, accepts and seals four artifacts:

- `grouped/check.rs:84-104`: a missing `.ansi`, `.txt`, `.html` or `.png`
  is `MissingApproval`; `:132-135` always renders actual PNG/HTML;
- `grouped/check_gates.rs:66` (`missing_approved_names`, fixed 4 labels);
- `grouped/mutate.rs:22-27`: `accept` copies all four;
- `grouped/seal.rs:60-75,110-135`: it hashes all four plus `frame.json`;
- `render/text.rs:46` (`ansi_dump`) writes no cursor, so the caret is
  gated only through the PNG/HTML.

We measured on the full corpus: `ansi_dump` plus the trailer (`ESC[?25l`
when hidden, else `ESC[?25h ESC[<y>;<x>H ESC[<Ps> q`) round-trips the
cells and the visible cursor for all **7,550/7,550** frames. A PNG
re-rendered from the decoded file is byte-identical to one from the full
frame in all 7,550. The store shrinks from **13.2 GB to 71 MB**, and the
`.ansi` caught 14 classes of change the PNG missed (NBSP, indexed vs RGB,
default vs explicit colour, underline style and colour, and others).
1,950 of the 7,550 frames (25.8%) have a visible cursor. Ask:

1. `render::ansi_dump_with_cursor(&Frame) -> String`, and the inverse
   `formats::ansi::frame_from_ansi(&str, Provenance) -> Result<Frame, FormatError>`.
2. `formats/ansi.rs:30` `assert_normalized_sgr` accepts exactly one
   trailing cursor trailer.
3. `GroupedStore::with_artifacts(ArtifactSet::AnsiCursor)` (types at
   `grouped/types.rs:71`). In that mode it requires, reads, gates, accepts,
   seals and hashes only the configured files. PNG/HTML/diff are written
   only on a mismatch, as review evidence: expected is rebuilt from the
   approved `.ansi`, reusing the diff block at `check.rs:244-251`. The
   status stays cell-based.
4. CLI: `tuiscotti accept` / `review` for a grouped store in that mode,
   with a name filter or glob (`crates/tuiscotti-cli/src/cli.rs:155-163`
   is per-name and classic-store only). Keep explicit-only approval:
   no env-var bless, refuse under `CI`.

### P4. `expect_visible` requires a unique match

`bound_locator.rs:162-181`: `expect_visible_within` uses `resolve_unique`.
Boot needles that repeat on screen (a title also present in a list) need
`Locator::resolve` inside `wait_predicate`. Add `first()`, `nth(n)`,
`any()` and `expect_count_at_least(n)` (or a `Match` policy), keeping
unique as the default.

### P5. `WaitError` omits the last screen

`tui/error.rs:107-111`: `Timeout` prints only `waited` and
`evidence.revision`. The README (line 121) promises "Timeouts fail WITH the
last screen". Include a bounded text dump of the evidence screen (for
example the first N rows, capped at K bytes, trailing blanks trimmed) in
`Display`, the same for `Cancelled`, `Unsupported` and `Closed` when
evidence exists. `evidence()` stays as the structured accessor.

### P6. Review rendering is slow

On the mismatch path, actual + expected PNG/HTML + diff cost **2–9 s per
capture** (macOS arm64, debug test profile, 120x40 and larger frames). A
showcase pilot took 1,129 s when everything mismatched and 262 s when
everything passed. `docs/PERFORMANCE.md:176-183` reports about 15 ms per
render. Find the gap. Suspects: dependencies at opt-level 0 in consumers'
dev profile, per-call face parsing, glyph cache misses, HTML embedding a
base64 PNG plus frame JSON. Wanted: profile and fix; a shared and reused
`Renderer` (cache faces and glyphs across checks and threads); HTML
optional or lightweight; and, if opt-level matters, a documented
`[profile.dev.package.tuiscotti-render] opt-level = 3` recommendation.

### P7. No absolute paths in gated or committed artifacts

`render::frame_from_screen` leaves `provenance.argv` empty
(`render/frame.rs:47`), which is good. But `Provenance::now(profile, source,
argv)` (`tuiscotti-core/src/frame/provenance.rs:25`) and the session and
registry paths (`proto/session_ops/stop.rs:108`) carry the raw argv. Under
tuisnap, an absolute `argv[0]` in the HTML failed every gate on any other
checkout. Make it a guarantee: no artifact the grouped store gates, seals
or accepts may contain absolute paths (store `argv[0]` as a file name, or
leave provenance out of byte gates), and add a test for it.

### P8. Nice-to-haves

- **Clock and time control.** There are no hooks today (`wait_stable` uses
  real deadlines; `DEFAULT_STABLE_QUIET` is in `tui/limits.rs:32`).
  Offer a documented approach (env-var convention, a `Tui::clock` hook
  the app can opt into) or explain why not.
- **Pointer.** `Session::mouse_down` and `mouse_move`
  (`tui/session_input.rs:162,203`) exist and check modes, and we moved
  off raw SGR bytes onto them. Document which DEC modes each one needs
  (1000/1002/1003/1006) and add a hover-via-move and a press-and-hold
  example.
- **Docs.** `docs/ARCHITECTURE.md:68,74,110` and DECISIONS §6 still say
  `portable-pty` + `alacritty_terminal`, while `Cargo.toml:33-34` and
  LIMITATIONS say `termpane`. Make them agree.
- **Example.** Add a Workflow-2-to-files example: spawn a real binary, wait,
  `frame_from_screen`, then a grouped check into a directory store, with a
  CI artifact upload snippet.

## 3. Acceptance criteria

- [ ] `press("f")`, `press("F")`, `press("ctrl+f")` send the char; `F1`–`F12`
      unchanged; `F13`/`F0` still error.
- [ ] A consumer on ratatui 0.30 / crossterm 0.29 / unicode-width 0.2.2
      resolves a single copy of `unicode-width` and does not get a forced
      downgrade. Duplicate ratatui/crossterm are gone or behind a feature.
- [ ] `ansi_dump_with_cursor` → `frame_from_ansi` round-trips the cells and
      the cursor (property test plus fixtures with wide chars, combining
      marks, every SGR attribute, hidden and visible cursor and every
      DECSCUSR shape).
- [ ] `GroupedStore::with_artifacts(ArtifactSet::AnsiCursor)` passes with
      only `<name>.ansi` approved, fails closed when it is missing, writes
      expected/actual PNG/HTML/diff only on a mismatch, and `accept` copies
      only `.ansi`. The default `ArtifactSet` keeps today's behaviour and
      bytes exactly.
- [ ] CLI accept/review works on such a store with a filter.
- [ ] Locator `first`/`nth`/`any`/`count_at_least`; unique stays default.
- [ ] `WaitError::Timeout` `Display` includes a bounded screen dump.
- [ ] Mismatch-path render cost is measured before and after, in a bench
      or a PERFORMANCE.md table, for 80x24, 120x40 and 160x50.
- [ ] A test proves no absolute path reaches gated or sealed artifacts.
- [ ] Every existing test still passes. Existing public APIs keep their
      signatures and defaults (additive changes, or a feature flag).

## 4. How to work

1. **Research** the crates (`tuiscotti-runtime` grouped/tui/bound_locator,
   `tuiscotti-render` render/formats/diff, `tuiscotti-cli`), DECISIONS.md,
   SNAPSHOTS.md, TESTING.md and PERFORMANCE.md. Read our usage on the branch
   above. Re-verify every file:line here against current `main`.
2. **Analyse and brainstorm** at least two designs for P2 (ranges vs
   adapter crate vs features), P3 (artifact-set enum vs per-artifact
   flags vs a pluggable gate trait) and P6. Choose the one that best keeps
   tuiscotti's invariants (explicit approval, fail-closed, determinism).
   Record the decision in DECISIONS.md.
3. **Implement** additively. Existing consumers must keep compiling and
   produce identical artifacts by default. Where a break is unavoidable,
   gate it behind a feature or a new constructor, and write a migration
   note.
4. **Test** each item (unit, property and golden), plus one end-to-end
   test that drives a fixture binary through the `.ansi`-only store:
   pass, mismatch with evidence, accept, pass again.
5. **Document:** API.md, SNAPSHOTS.md (the new artifact mode and trailer
   grammar), CLI.md, TESTING.md (CI wiring for the minimal mode),
   LIMITATIONS.md, and `docs/MIGRATION.md`.
6. **Land on `main`.**

## 5. Report back

Reply with:

- the new `main` commit SHA to pin;
- the API we should call to delete our wrappers: the replacement for
  `tests/visual_baseline/support.rs` `gate()` and
  `write_expected_evidence`, for `ansi_cursor.rs` `encode`/`decode`, and
  for the lone-`f` workaround in `send_key`; also say whether the CLI can
  replace `cargo xtask visual accept`;
- confirmation that our committed `.ansi` files (`ansi_dump` + trailer, as
  in `ansi_cursor.rs`) are byte-compatible with your format, or the exact
  difference and a one-shot conversion;
- the dependency ranges and what changes in our lockfile;
- measured render timings before and after;
- anything declined or changed, and why.

Do not open issues or PRs against our repository. We will bump the pin
ourselves.
