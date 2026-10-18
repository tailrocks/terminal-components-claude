# Baseline migration analysis — adopt-vs-improve from tui-test and terminal-control

Which patterns from two external projects the Rust baseline suite (or tuisnap
itself) should borrow, which to reject, and the suite architecture that falls
out of the answers. Companion to `tuisnap-coverage.md` (the capture matrix);
this document is the *how*, not the *what*.

## Provenance

Inspected 2026-09-12, source (not READMEs), all three local checkouts:

| Project | Commit | Notes |
|---|---|---|
| `microsoft/tui-test` | `9b21fd15ba318efcb6da138a2d7e4b3037142fb9` (2026-09-11) | workspace: `crates/tui-test` (lib) + `crates/tui-test-cli` |
| `anomalyco/terminal-control` | `c1d4f95e4f1b7638f6229e9bfc9599a955f95ce2` (2026-09-04, v1.2.1) | single crate, lib + `termctrl` bin |
| `tailrocks/tui-snap` (ours) | `263eeeb1800fd273898a9a13eaa39abefe87d10a` (2026-09-12) | vendored `termlens` 0.9 at `vendor/termlens` |
| this repo | `e5cfac95e45f9391ee9d07647d1b79f129138052` | 367-capture baseline, `tools/tuisnap_baseline.sh` |

Neither external project may become a dependency — patterns only.

---

## 1. microsoft/tui-test — pattern notes

### 1.1 Lazy text/style locator model

**How it works.** A `Locator` is "a lazy query resolved against the current
terminal grid before every read, wait, or action" (`runtime.rs:62`). Resolution
(`terminal/locator.rs`):

- The grid is flattened row-major into a `chars` vector plus a `sources`
  position map, so every match maps back to exact `(x, y)` cells. Optional
  whitespace normalization collapses whitespace runs to one space *without*
  losing cell positions.
- Text matching is plain substring (non-overlapping scan) or regex with
  one-pass byte→char offset mapping.
- Occurrence selection: `Any | Unique | First | Last | Nth(n)`
  (`MatchOccurrence`). `Unique` with >1 match is a hard error naming the count
  ("expected 'same' to match once, but found 2 matches") — ambiguity is loud,
  never silently first-matched.
- Scoping: `after`/`before` text anchors restrict the search region; queries
  nest (`within` + `Within/After/Before` direction) by computing allowed
  flat-index regions from parent matches.
- Style queries (`StyleSelector` over a `TextStyle` of
  bold/italic/colors/underline/link/…) find contiguous per-row runs of
  matching cells, compose with text in either order, and treat blanks
  deliberately: appearance styles skip blank cells (a space shows no
  foreground), link styles check them (a space can carry a URI)
  (`locator.rs:160-211`).

**Worth borrowing?** Partially.

- **Yes:** occurrence selection (`Unique` especially) and `after`/`before`
  anchors. Our four apps repeat strings across regions (help lines, status
  bars), and a boot needle that matches the wrong occurrence — or a stale
  region — is a classic flake shape. "Wait until `X` appears uniquely" and
  "wait until `X` after anchor `Y`" cover every real need.
- **No:** the full nested locator algebra (multi-level `within` chains,
  direction composition). Nothing in the 367-capture matrix needs it; the
  gate is the whole frame anyway.
- **Later:** style predicates for waits (e.g. "wait until the status bar is
  red"). The full-frame gate already asserts final styles; waits rarely need
  style. Defer until a concrete flake demands it.

**Where it would live.** First as suite helpers (predicate over
`Session::snapshot()` text/region). If it proves out, a small tuisnap
addition: expose `Session::wait_until` (see §5b) plus an occurrence/region
wrapper — tuisnap already re-exports `termlens`, so the predicate can run on
`termlens::Screen` without new conversions. Do **not** port the `LocatorQuery`
struct hierarchy.

### 1.2 Wait taxonomy

Declared in `api.rs` (`Operation::{WaitTitle, WaitClipboard,
WaitClipboardMatch, WaitIdle, WaitCommand, WaitExit, WaitReady, WaitBell,
WaitLocator}` plus `Expect{ExitCode, Mode, Colors, Cursor, BellCount}`),
implemented in `engine.rs:1745-2193`:

| Wait | Mechanism | Useful for our simulated apps? |
|---|---|---|
| `WaitLocator` (visible text/style, `not`) | polls `locate_query` against the live grid; escapes early if the session exits | **Yes — core.** This is the principled version of our `wait:<needle>`. |
| `WaitIdle` | 250 ms since last *screen change* (`last_change.elapsed()`, `engine.rs:1950`) | Yes, but tuisnap's `wait_stable` is strictly stronger (style-aware, not just text/bytes). Keep ours. |
| `WaitExit` | child exit flag, with exit-query error path | Yes, for quit-path captures. tuisnap has it. |
| `WaitReady` (shell-ready) | OSC 133/633 shell-integration tracker: prompt marks (`terminal/integration.rs`) | **No — misleading.** Our apps are full-screen TUIs with no shell integration; against them this wait can never fire. |
| `WaitCommand` (command-complete) | same tracker's started/finished counts; **falls back to 300 ms quiet when no shell integration** (`engine.rs:1979-1999`) | **No — misleading.** Against a non-shell app it silently degrades to a quiet-period heuristic wearing a shell-shaped name and shell-shaped error messages. |
| `WaitClipboard` / match | OSC 52 clipboard events | No. Our apps don't touch the clipboard. |
| `WaitBell` | bell sequence counter | No. |
| `WaitTitle` | window-title watcher | No. |

The lesson that *does* transfer: tui-test's taxonomy separates **content
waits** (locator), **settle waits** (idle), and **lifecycle waits** (exit),
and each wait's error says which of the three gave up. tuisnap already covers
this subset: `wait_for_text` (content, error embeds the screen at timeout),
`wait_stable`/`wait_idle` (settle), `wait_exit` (lifecycle). Nothing to add
except occurrence/region variants under §1.1.

### 1.3 Renderer font catalog + glyph-outline caching

`render/font.rs` + `render/raster/font.rs`:

- **Process-global catalog**: `static CATALOG: OnceLock<Catalog>` — one
  `fontdb::Database` (bundled JetBrains Mono tier via cargo features + Nerd
  Font + system fonts), a deterministic `face_score` sort producing
  `candidates[4]` indexed by `(bold, italic)`, Nerd-Font faces segregated for
  PUA codepoints, env-var family override. Loaded once per process, shared by
  every renderer.
- **Per-renderer outline cache**: `FontSystem { glyphs: HashMap<GlyphKey,
  Option<GlyphOutline>> }` keyed by `{character, bold, italic}` — tiny-skia
  `Path` outlines cached **including negative results** (`None` = no face
  covers it, never re-looked-up). Synthetic bold/italic is decided once at
  load and carried on the cached outline.

**Worth borrowing?** **Yes — this is the exact shape tuisnap needs.** tuisnap
currently re-parses 4–5 fontdue faces *per frame* and re-rasterizes every
glyph *per frame* (§5d). tui-test's two-level split (process-global face
catalog + per-renderer glyph cache with negative entries) is the model; the
tuisnap version is simpler because the font set is pinned and vendored.

### 1.4 Public `GridRenderer` / `FrameRenderer` boundary

`render/raster.rs:47-62`: `pub trait FrameRenderer { fn render(&mut self,
frame: &Frame) -> Result<RgbaFrame>; fn pixel_size(&self) }`, implemented by
`pub struct GridRenderer`, which is constructed once per
`(cols, rows, zoom, background)` and **owns** the pixmap and the `FontSystem`
(hence the glyph cache). The video encoders in `render/encode.rs` take
`&mut dyn FrameRenderer` and reuse one renderer across a whole frame sequence,
keeping the cache hot.

**Worth borrowing?** **Yes.** It validates the `Renderer`-object proposal for
tuisnap (§5d): construct once, `&mut self` render per frame, cache rides
inside, free-function one-shots remain as thin wrappers. The trait itself is
more than we need (one renderer impl, no encoders) — a concrete struct
suffices.

### 1.5 Native snapshot auto-write-on-missing — confirmed anti-pattern, reject

`assert/snapshot.rs:176-202`, `compare(base, name, content, update)`:

```rust
if !path.exists() {
    std::fs::create_dir_all(snapshot_dir(base))?;
    std::fs::write(&path, format!("{trimmed}\n"))?;
    return Ok(SnapshotStatus::Written);
}
```

A missing baseline is **silently written and counts as a pass** (`Written`,
not a failure); `update: true` rewrites on mismatch. This turns the snapshot
suite into a snapshot *recorder*: a regression that lands before the first run
on a fresh machine (or after a baseline file is deleted/lost from git) is
blessed as truth, and CI goes green on it. This is why we reject it, and the
doc trail should say so explicitly:

- tuisnap's `Store::check` returns `Status::MissingApproval` (a failure) when
  `approved/` has no entry and **never writes under `approved/`**
  (`snapshot.rs:421-425` comment: approvals change solely through explicit
  `accept`, "so parallel gates cannot race on approval files and renderer
  upgrades cannot silently heal them").
- `Store::accept` is an explicit human act with "deliberately no
  environment-variable auto-accept" (`snapshot.rs:467-469`).

One adjacent idea worth noting (already covered): tui-test's serializer
records the palette **slot** rather than the resolved RGB so baselines survive
profile recolors. tuisnap's `Frame` stores `Color::Indexed` vs `Color::Rgb`
natively and pixels are gated against the pinned profile — equivalent
protection, no action.

### 1.6 How tui-test's own Rust tests are organized

- Heavy inline unit tests (`#[cfg(test)] mod tests` per module; `locator.rs`
  alone carries ~500 lines built on a 10-line `grid(&[&str])` fixture
  constructor that turns string lines into cell grids).
- Integration tests per concern: `crates/tui-test/tests/runtime.rs` (sessions,
  registry, operations) and `crates/tui-test/tests/render-snapshots/`
  (blessed PNG/GIF fixtures for the renderer itself);
  `crates/tui-test-cli/tests/session_lifecycle.rs` for the CLI layer. No
  `examples/` directory.

**Worth copying structurally?** Two things: (1) a tiny text-fixture
constructor for grid-level unit tests of helpers (our region/occurrence
predicates can be unit-tested without any PTY); (2) renderer golden fixtures
living in their own directory with the test that regenerates/compares them —
maps to our `shots/tuisnap/` store layout. The per-concern-file split (rather
than one mega `tests/baseline.rs`) is already our plan (§6).

---

## 2. anomalyco/terminal-control — pattern notes

### 2.1 Embedded `Session` reuse model

`session.rs`: `Session::start(&argv, ...)` spawns one PTY-backed child
in-process, starts one reader thread feeding an mpsc queue, and then serves
*many* operations — `send`, `wait_for_text`, `wait_for_idle`,
`capture(settle, deadline)`, `status`, `resize`, `semantic_snapshot` — over
the session's life. The named-session CLI daemon and the TypeScript driver are
adapters over this same implementation (AGENTS.md: "Named sessions own one
terminal process"). The documented library flow (`docs/rust-library.md`) is
exactly one-session-many-captures: boot → `wait_for_text` → `send` →
`wait_for_idle` → `capture` → … → `wait_for_exit` → `stop`.

**Worth borrowing?** **Yes, with argv-scoped bounds** (§5a). It is the proven
shape for amortizing boot+settle across a scenario group, and tuisnap's
`Session` (spawn/send_key/type_text/wait_*/snapshot) already supports it —
`run_once` is the only one-shot-shaped piece, and it's a convenience wrapper,
not a constraint.

### 2.2 Reusable `PngRenderer` (font-db init once)

`render.rs:105-194`: `pub struct PngRenderer { options:
resvg::usvg::Options<'static> }`; `new()` loads the system font db **once**;
`render(&self, …)` reuses the options for every SVG→PNG conversion.
`recording.rs:836`'s `VideoRenderer` holds one `PngRenderer` across all video
frames. The one-shot `render::png()` free function exists but constructs a
fresh renderer per call — the same free-function-vs-object split proposed for
tuisnap.

**Worth borrowing?** **Yes** — independent confirmation of the §5d pattern
from a second codebase: load fonts once into a long-lived renderer object;
per-frame work shrinks to rasterize+encode.

### 2.3 Capture-completion semantics (idle vs deadline vs exit vs output-closed)

`session.rs:85-100, 490-527`. `capture(settle, deadline)` loops at 10 ms and
returns `CaptureResult { shot, reason }` with the reason exposed:

```rust
pub enum CaptureReason { Idle, Deadline, Exited, OutputClosed }
```

Priority: `Exited` > `OutputClosed` > `Idle` (settle elapsed since last
output) > `Deadline`. Why it matters: a `Deadline` capture is evidence the app
may still be moving — the caller can (and their CLIs do) treat it as "do not
trust this frame for a baseline". A bare screenshot API cannot express that;
the reason makes unsaved/mid-motion frames distinguishable from settled ones.

tuisnap's answer to the same problem is fail-closed: `wait_stable` errors on
timeout and the error embeds the screen at timeout — equivalent protection,
different shape (exception vs reason-tagged value).

**Worth borrowing?** The **vocabulary**, not the mechanism. In the Rust suite,
every capture path ends in a settle wait that *fails the test* on deadline
(tuisnap behavior — keep it). Borrow the habit of recording *which* completion
produced the frame in test diagnostics ("settled via wait_stable(400ms)" vs
"boot timeout"), so a near-deadline capture reads as a flake risk at review
time. No tuisnap change required. (Their `wait_for_text` also distinguishes
"session ended before text appeared" from "timed out" (`session.rs:448-452`)
— a one-line error-quality idea worth copying into tuisnap's wrappers
eventually.)

### 2.4 Public `Frame` construction — external Ratatui→Frame adapter feasibility

`frame.rs` is all-public plain data: `Frame { version, cols, rows, foreground,
background, cursor: Option<Cursor>, cells: Vec<Cell> }` with `Cell { x, y,
text, width, foreground, background, attributes }` and resolved-RGB `Color`.
An external adapter is: iterate the Ratatui `Buffer`, map `ratatui::Color` →
RGB, set `width = 2` on wide leads and skip/blank the continuation, set the
cursor. The research claim is **confirmed with one caveat**: the xterm
indexed→RGB palette table (`indexed_color`, `frame.rs:121`) is `pub(crate)`,
so an external adapter re-implements ~40 lines (16 ANSI colors + 6×6×6 cube +
grayscale ramp). For us the question is academic — tuisnap already ships its
own adapter (`src/ratatui.rs`, `from_buffer`/`capture`) and PTY conversion
(`pty.rs frame_from_screen`) against *its* canonical `Frame` — but it confirms
that adapting a foreign, simple `Frame` type would not have been a project.

### 2.5 Thread-confinement of Ghostty state

`Session` doc (`session.rs:28-33`): "Ghostty terminal state is
thread-confined, so a session must remain on the thread where it was created."
AGENTS.md: sessions "remain single-threaded because Ghostty terminal objects
are not `Send` or `Sync`." Parallelism in their world is therefore *many
sessions on many threads/processes*, never one session shared across threads.

**Impact on us:** as a design rule, adopt the same confinement: each
`tuisnap::pty::Session` is created, used, and dropped on one test thread.
cargo test runs each `#[test]` on its own thread, so per-test sessions comply
naturally; a shared `static SESSION` would not. (Note the constraint is
*their* engine's; tuisnap's vendored termlens has no `!Send` marker and no
thread-affinity API — but confining sessions costs us nothing and keeps the
mental model identical.)

---

## 3. Cross-cutting answers

### 3a. One PTY per capture, or one session per scenario group?

**Answer: hybrid, bounded by argv identity.** The matrix dimensions
(`--page`/`--size`/`--color`/env) are launch arguments — different cells of
the matrix are different processes no matter what, and most of the 367
captures are distinct argv cells. Session reuse can only ever apply *within*
one argv cell, i.e. interactive sub-states reached by key sequences from the
same boot (e.g. `showcase --page inputs` at 120×40 truecolor →
`inputs_editing`, `inputs_selected`, …).

Within an argv cell:

- **Reuse when** the group shares a boot and each scenario returns to a
  verifiable neutral screen (esc-spam + boot-needle wait). Saves
  boot+`wait_idle(200ms)`+`wait_stable(400ms)` ≈ 0.6–1 s per capture —
  significant across dozens of interactive sub-states; this is
  terminal-control's embedded model (§2.1) working as designed.
- **Fresh process when** a scenario mutates hard-to-reset state (scrollback,
  modal stacks, quit/restart paths), or when the neutral-point wait itself
  would be as slow as a boot. Determinism is seeded, but residue leaks are
  real; flake isolation also argues for fresh processes — one crashed app
  costs one capture, not the rest of the group.

Practical encoding: `Case::reuse_group: Option<&str>`; cases with the same
group run sequentially in one test body sharing a session; everything else is
its own test with its own process (see §6).

### 3b. Locator-style waits vs fixed needles — and is tuisnap there today?

**Answer: yes to locator-style waits; tuisnap needs one small addition.**

- Today tuisnap's library exposes `wait_for_text` (whole-screen substring via
  `termlens::Screen::text()`), `wait_stable`, `wait_idle`, `wait_exit`
  (`pty.rs:256-275`). There is **no** generic predicate passthrough and no
  occurrence/region/style variant. `Session.term` is private, so the suite
  cannot reach `termlens::Terminal::wait_until` (which exists and takes
  `impl FnMut(&Screen) -> bool`, `vendor/termlens/src/terminal.rs:2818`).
- The minimal tuisnap addition (≈3 lines in `src/pty.rs`, next to
  `wait_for_text`):

  ```rust
  /// Fail unless `pred` holds on the live screen before the deadline.
  pub fn wait_until(&mut self, pred: impl FnMut(&termlens::Screen) -> bool) -> Result<()> {
      Ok(self.term.wait_until(pred)?)
  }
  ```

  Everything from §1.1 that we actually want — unique occurrence, after-anchor
  scoping, region-restricted match — then composes in suite helpers over
  `Screen` text/cells, unit-testable with tui-test's `grid(&[&str])` fixture
  trick (§1.6). Style-aware waits stay deferred; add only if a real flake
  demands them.
- Keep fixed `sleep:` steps out of the matrix entirely; the only timed steps
  should be settle waits (`wait_stable`) and the small post-key pacing that
  already lives in `run_once` (120 ms) — in the suite, replace pacing with
  content/stable waits wherever a needle exists.

### 3c. Parallel execution — are parallel tuisnap Sessions safe?

**Answer: yes, at cargo-test default parallelism, including on macOS. This was
engineered and stress-tested in vendored termlens.**

- tuisnap's `pty.rs` holds no global state; `Session` owns a
  `termlens::Terminal`; `ansi.rs`'s termpane use is per-instance
  (`DamageGrid::new`).
- The vendored termlens has exactly one process-global:
  `static PTY_LIFECYCLE: Mutex<()>` (`vendor/termlens/src/terminal.rs:190`),
  held across PTY **lifecycle edges** (open+spawn on one side,
  kill+reap+master-close on the other) because macOS tears PTYs down with
  `revoke()` and recycles device numbers immediately — a concurrent teardown
  could revoke a just-opened device (~1-in-800 spawn deaths observed on loaded
  macOS runners). Steady-state I/O never touches the lock.
- `open_pty` additionally retries transient `ENXIO` exhaustion (12 attempts,
  25 ms backoff, **lock dropped between attempts** so teardowns can free
  devices), explicitly motivated by "a suite running one test per core —
  which is what `cargo test` does by default" at sixteen threads
  (`terminal.rs:206-224`). Linux ran the same stress suite clean.
- Store side: `Store::check` writes only under `actual/` and `diff/` (atomic
  writes, distinct names per capture) and never under `approved/`
  (`snapshot.rs:421-425`), so parallel gates cannot race; `accept` runs
  outside the test process.
- App side: all four binaries run seeded simulated state; holla's history
  side effects are already suppressed via `HOLLA_NO_HISTORY=1` (the baseline
  script exports it — the suite must set it per-session env). No sockets, no
  shared files. If any app ever grows a per-user state dir, pin that app's
  tests behind a `static Mutex` rather than serializing the whole suite.

Concurrency ceiling: default `--test-threads` (one per core). Each test = 1
PTY + 1 child + 1 reader thread; a dozen concurrent is well inside the envelope
termlens was stress-tested at.

### 3d. Renderer caching for bulk re-render — concrete tuisnap proposal

**Problem.** `Store::check` → `render::render_png_report` (`render.rs:327`)
does, *per frame*: `load_font(faces.regular, …)` + `verify_geometry`
(lines 337–338), then `FontSet::load` — four `fontdue::Font::from_bytes`
parses of the ~2 MB vendored faces (line 342) — then per-cell
`face.font.rasterize(c, px)` with **no glyph cache** (`draw_symbol`,
line 265). Across 367 captures (×2 renders whenever an approved PNG is
regenerated to memory), that is ~3k redundant font parses and ~10⁶ redundant
glyph rasterizations of the same few hundred glyphs.

**Proposal** (mirrors tui-test's `FontSystem`, §1.3, and terminal-control's
`PngRenderer`, §2.2):

1. New `pub struct Renderer` in `tuisnap/src/render.rs`, next to `FontSet`:
   owns `profile: Profile`, `set: FontSet` (loaded once), and
   `glyphs: HashMap<GlyphKey, Option<(fontdue::Metrics, Vec<u8>)>>` where
   `GlyphKey { ch: char, face: FaceIdx }` (`FaceIdx =
   Regular|Bold|Italic|BoldItalic`; px is fixed per profile at
   `font_px * scale`, so it needn't be in the key). Cache **negative**
   entries (tofu codepoints) exactly like tui-test.
2. `Renderer::new(profile, faces)` performs the one-time
   `load_font`+`verify_geometry` (moving lines 337–338 off the per-frame
   path) and `FontSet::load` (line 342).
3. `Renderer::render(&mut self, frame: &Frame) -> Result<Rendered,
   RenderError>` is today's `render_png_report` body minus the loads;
   `draw_symbol` consults the cache instead of calling
   `face.font.rasterize` directly (insertion point: `render.rs:265`).
4. Keep `render_png` / `render_png_report` as thin one-shot wrappers
   (construct, render once) so CLI behavior is byte-identical.
5. Store integration: add `Store::check_with(&self, renderer: &mut Renderer,
   name, actual, pixel_threshold)` (or a `check_rendered` variant taking a
   pre-rendered `Rendered`) so the suite gates through the cached renderer
   while `Store` keeps owning the approval policy. `profile.rs` needs no
   structural change; optionally add `Profile::renderer(&self, faces)`.
6. Threading: `Renderer` is used via `&mut self`, so give each test thread
   its own (`thread_local!` in the suite helper) — no locks, deterministic,
   and matches the per-test-thread session confinement from §2.5/§3c.

Expected effect: font parsing drops from O(frames) to O(threads);
rasterization drops to O(distinct glyphs × faces) for the whole run.

---

## 4. Recommended suite architecture

### Test file layout (under `tests/`)

```
tests/
  baseline_support/mod.rs     # helper crate-module compiled into each test target
  baseline_showcase.rs        # one integration target per app binary
  baseline_holla.rs
  baseline_tablepro.rs
  baseline_jackin.rs
```

(Existing focused tests — `holla_pty.rs`, `terminal_suspend.rs`,
`choice_containment.rs`, `focus_gutter.rs` — stay as they are; the baseline
suite is additive.) Rationale for one target per app, borrowed from tui-test's
per-concern split (§1.6): independent parallel scheduling, per-app filtering
(`cargo test --test baseline_holla`), and a crashed app never takes another
app's target down with it.

### Matrix encoding in Rust

The 367 names stop being bash positional args and become typed data. A
`macro_rules!` generates one `#[test]` per case (real test names → per-case
isolation, default parallel scheduling, `cargo test <name>` filtering):

```rust
struct Case {
    name: &'static str,                  // "showcase_overview_default_80x24_truecolor"
    args: &'static [&'static str],       // ["--page", "overview"]
    cols: u16, rows: u16,
    color: ColorMode,                    // Truecolor | Ansi256 | Ansi16 | NoColorFlag | NoColorEnv
    steps: &'static [Step],
    settle: Duration,                    // usually 400 ms
    timeout: Option<Duration>,           // CAP_TIMEOUT override (scrolling/terminal)
    reuse_group: Option<&'static str>,   // Some("inputs") → shares a session within its test
}

enum Step {
    Key(&'static str),        // send_key names, as today
    Type(&'static str),
    Wait(&'static str),       // needle (whole screen)
    WaitUnique(&'static str), // needle that must occur exactly once (§3b)
    WaitAfter { anchor: &'static str, needle: &'static str }, // scoped (§3b)
    Sleep(u64),               // last resort only
    Neutral,                  // return to the group's verifiable boot screen (§3a)
}
```

`ColorMode::NoColorEnv` wraps argv in `NO_COLOR=1` (today's `env NO_COLOR=1`
trick), and the suite always strips ambient `NO_COLOR` for non-nocolor cases —
the script's `PRESERVE_NO_COLOR` contract moves into `session_for`.
`HOLLA_NO_HISTORY=1` is set on every holla session's env (§3c).

Cases that share a `reuse_group` are emitted into **one** `#[test]` that runs
them sequentially over one session with `Step::Neutral` between captures;
all other cases get their own `#[test]` and their own process.

### Helper API sketch (`baseline_support/mod.rs`)

```rust
pub fn store() -> tuisnap::snapshot::Store;              // shots/tuisnap
pub fn session_for(case: &Case) -> tuisnap::pty::Session; // argv + env + geometry
pub fn drive(s: &mut Session, steps: &[Step]);           // Step → session calls (waits fail closed)
pub fn run_case(case: &Case, s: &mut Session) -> CompareOutcome; // drive + wait_stable + store.check_with(renderer)
thread_local! { static RENDERER: RefCell<tuisnap::render::Renderer> } // §5d, one per test thread
pub fn wait_unique(s: &mut Session, needle: &str);       // over Session::wait_until (§3b)
pub fn wait_after(s: &mut Session, anchor: &str, needle: &str);
pub fn neutral_point(s: &mut Session, boot_needle: &str); // esc-spam + boot needle
```

Grid-level unit tests for `wait_unique`/`wait_after` predicates live inline in
the support module using a `grid(&[&str])`-style fixture (§1.6) — no PTY
needed.

### Accept/report workflow (unchanged policy, new driver)

1. `cargo test --test baseline_showcase …` — captures write `actual/`
   (frame + PNG + fidelity sidecar), gate cell-exact + pixel-exact, and
   **fail closed** on `missing-approval`, `cells-differ`, `pixels-differ`,
   dimension mismatch, or any wait timeout. No auto-write of approvals —
   that tui-test policy is explicitly rejected (§1.5).
2. Review: `tuisnap report --store shots/tuisnap` (HTML expected/actual/diff).
   Loose per-capture artifacts (`ansi/txt/png/html` under `shots/tuisnap/frames/`)
   become an opt-in review render (env-gated ignored test or a small
   `cargo test -- --ignored` pass), not an always-on second render.
3. Bless: `tuisnap accept --store shots/tuisnap [--all | <name>]` — still the
   only mutation path for `approved/`; CI never accepts (tuisnap enforces:
   no env auto-accept).
4. CI: runs the four baseline targets with default `--test-threads`; a
   missing or drifting baseline is red, exactly as today.

### Parallelism stance

Default cargo test threads (one per core). Per-case process isolation by
default; session reuse only sequentially inside a `reuse_group` test. One
`Renderer` per test thread via `thread_local!`. Sessions are
created/used/dropped on their test thread (§2.5). Backed by evidence:
termlens's `PTY_LIFECYCLE` lock + `ENXIO` retry were built and stress-tested
for exactly this load shape on macOS (§3c); `Store` never writes `approved/`
during checks (§3c); apps are seeded and side-effect-free under
`HOLLA_NO_HISTORY=1`.

---

## 5. Summary of adopt / improve / reject

**Adopt into tuisnap (library additions):**

1. `Renderer` object: load `FontSet` once, cache glyph rasterizations keyed by
   `(char, face)` incl. negatives; free functions stay as one-shot wrappers.
   Modeled on tui-test `FontSystem` + terminal-control `PngRenderer`. Spots:
   `render.rs:265` (cache), `render.rs:337-342` (hoist), `snapshot.rs`
   `check_with` (use).
2. `Session::wait_until` passthrough to termlens (3 lines, `pty.rs`) —
   unlocks occurrence/region waits.

**Adopt into the suite (patterns):**

3. Occurrence selection (`Unique`) + after/before anchors as wait helpers —
   from tui-test's locator model, minus the nested-query algebra.
4. Hybrid session model: argv-scoped reuse groups with a verifiable neutral
   point; fresh process per matrix cell — terminal-control's embedded model,
   bounded by determinism/flake-isolation realities.
5. Capture-reason vocabulary in diagnostics (settled vs deadline) — a
   deadline-settled baseline capture is a flake flag at review time.
6. tui-test test-organization: per-concern integration targets, tiny
   text-grid fixtures for helper unit tests.

**Reject (with evidence, for the doc trail):**

7. Auto-write-on-missing snapshots (tui-test `compare`): a missing baseline is
   a `missing-approval` failure; blessing is explicit `accept` only.
8. Shell-integration waits (`WaitReady`/`WaitCommand`) and clipboard/bell/title
   waits: meaningless or silently heuristic against full-screen TUI apps.
9. Full locator algebra and `FrameRenderer` trait generality: more machinery
   than a 4-app, single-renderer suite justifies.
