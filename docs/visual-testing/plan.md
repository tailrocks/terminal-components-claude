# Visual regression testing and refactor completion plan

Status: proposed, 2026-10-16. Owner: main session (coordinates); all work is
done by subagents (AGENTS.md, Way of working).

Evidence for every claim below is in [`research/`](research/):
`research-tuiscotti.md` (tool), `research-infra.md` (this branch's suite),
`research-main.md` (the `main` branch), `research-formats.md` (snapshot
formats). Design backlog: `docs/audits/design-review-2026-10/`.

## Binding decisions

| # | Decision | Consequence |
|---|---|---|
| D1 | tuiscotti latest `main`, exact `rev =`, bumped as `main` moves | Harness migration from `tuisnap` is **T0, in flight** (another agent: `Cargo.toml`, `tests/visual_baseline/`, `.config/nextest.toml`). This plan builds on it and does not redo it. |
| D2 | Workflow 2: the real binaries in a PTY, scripted keys/mouse/resize, compared with a committed store, in CI | Spawn `CARGO_BIN_EXE_<bin>` (the binary `cargo run --bin X` builds), never in-process rendering. |
| D3 | Minimal committed format: **one `.ansi` + cursor trailer per scenario** | Lossless for all 7,550 current frames; 71 MB instead of 13.2 GB. PNG/HTML are generated only as review artifacts. Terminal modes are covered by one boot test per app. |
| D4 | `design/interaction-polish` is the visual and behavioural reference; `main` is the target structure | `main` must render every scenario 1:1 identically, keeping AGENTS.md's configurable API. The `visual-baseline` tag is never touched; the PO decides separately when this branch's oracle replaces it. |
| D5 | Small commits, pushed often, subagents do the work | Every task in §H is one PR-sized slice with the visual gate. |

## A. Goals, non-goals, acceptance

**Goals**

1. A CI-run visual + behavioural suite on this branch that gates every PR.
2. An approved, versioned oracle store produced from this branch after the
   design-review backlog lands.
3. The same harness and the same oracle gating `main` until `main` matches
   it 1:1.

**Non-goals**

- Gating PNG or HTML bytes (review artifacts only).
- Wall-clock or mid-animation frames (spinners, jackin rain, streaming TTY),
  as already dropped in `docs/baseline/snapshots-v2.md`.
- Windows. tuiscotti has never run there.
- Rewriting git history to drop the 7 GB of old `snapshots/` blobs. No
  force-push, ever.

**"1:1" means all of these hold on `main`:**

| Check | Pass condition |
|---|---|
| Visual | For every scenario in `baselines/oracle/`, `main`'s `.ansi` (cells, styles, grid size, cursor trailer) is byte-identical. No allow-list, no tolerance. |
| Scenario set | `main` runs the identical harness (hash in `ORACLE.toml`), so the scenario list cannot be narrowed. |
| Terminal modes | One boot test per app passes: alternate screen 1049, mouse 1003/1006 (or the app's documented mode), bracketed paste 2004 on; all restored after quit; exit code 0. |
| Functional journeys | Assertion-based PTY journeys pass (quit paths, Ctrl+Z suspend/resume, `NO_COLOR`/`--color` policy, input floods, resize). |
| CLI | Every argv used by the suite is accepted; `--help` exits 0 and lists the reference flags; an unknown `--page`/`--color`/`--scenario` exits non-zero with an error. |
| Repo gate | `cargo nextest run --all-targets`, `cargo clippy --all-targets`, `cargo fmt --check` green. |

## B. Suite architecture

### B1. Harness (Workflow 2)

```
scenario table ──expand──> Scenario{name, bin, argv, env, size, steps, ready}
   Tui::new([bin]).args(argv).env_clear(true).envs(PINNED).size(c, r).spawn()
   wait: Locator::expect_visible(ready needle)   # content-gated, never a sleep
   steps: press / send_text / mouse_down|up|move|wheel / resize, each followed
          by a content wait or wait_stable_quiet(200 ms)
   snapshot() → frame_from_screen(..) → encode_ansi_cursor(frame)
   gate: bytes == baselines/oracle/<name>.ansi
   on mismatch: write actual .ansi + frame.json, expected/actual PNG+HTML,
                diff PNG, report row  (status unchanged: still a failure)
```

| Item | Choice |
|---|---|
| Test granularity | **One nextest test per scenario** (not per root). A `harness = false` test binary (`libtest-mimic`) lists generated trials named `<app>::<group>::<leaf>[::<state>]::<cols>x<rows>::<color>`. Today one root runs 25 combos in series (about 14 s each under load, one drift fails all 25). |
| Binary resolution | `bin(name)`: `VISUAL_BIN_DIR/<name>` if set, else `env!("CARGO_BIN_EXE_<name>")`. The same harness file then works on `main`, where binaries live in other crates. |
| Renderer | One shared `Renderer` per process, used only to build review artifacts. |
| Gate code | Local wrapper in `tests/visual_baseline/support.rs` (replaces `gate()`), about 150 lines on public APIs: `render::ansi_dump`, cursor trailer encode/decode (from the formats probe `src/minimal.rs`), `Renderer::{render_png, render_html}`, `diff::compare_png`. Switch to the upstream `ArtifactSet::AnsiCursor` when it lands (§G). |
| Hover | tuiscotti `mouse_move`, which fails closed with `ModeNotEnabled`. Drop the raw `CSI <35;c;r M` bytes in `pointer.rs`. |
| Pressed | `mouse_down` with no `mouse_up`, in `--motion paused`. For keyboard activation, the press flash is frozen at `--frame N`. |
| Coordinates | Resolved from locators (`get_by_text(..).region()`) at capture time, never hardcoded, so a layout change shows up as a frame diff rather than a mis-click. |

### B2. Scenario definition: generated from state tables

Hand-written `Case::new` lines (302 roots today) are replaced by typed Rust
tables in `tests/visual_baseline/scenarios/<app>.rs`:

| Table | Row | Expands to |
|---|---|---|
| `SURFACES` | `{id, argv, ready, steps}`: one reachable screen or overlay | `<app>/<group>/<leaf>` base scenario |
| `ELEMENTS` | `{surface, id, locate, states: &[Hover, Pressed, Focus, Disabled, SelectedHover]}`: each interactive element | `<…>/<leaf>/<element>-<state>` |
| `OVERLAYS` | `{surface, open_steps, id}`: every popup, menu, dialog and picker per app | `<…>/overlay/<id>` plus `<id>-dismissed-esc` and `<id>-dismissed-outside` |
| `JOURNEYS` | ordered `{steps, checkpoint}` lists (existing `flows`) | one scenario per checkpoint |
| `RESIZES` | `{surface, from, to}` | `<…>/resize/<from>-<to>` |
| `MATRIX` | tier → sizes × colours | the per-scenario combo suffix |

Rule: the generated set must contain all 302 existing root names unchanged,
so the first re-capture is reviewable as a diff against old frames.

### B3. Environment pinning (every spawn)

| Var / flag | Value | Why |
|---|---|---|
| `env_clear(true)` | always | The child must not inherit the developer's or CI's environment. |
| `TERM` | `xterm-256color` | tuiscotti default, made explicit. |
| `COLORTERM` | `truecolor` only for the truecolor level; unset otherwise | Colour detection. |
| `--color` | `truecolor\|256\|16\|none`; level `nocolor` = no flag + `NO_COLOR=1` | Both paths are policies under test. |
| `LANG`, `LC_ALL` | `C.UTF-8` | Width and locale. |
| `TZ` | `UTC` | Any rendered time. |
| `HOME`, cwd | `target/tuiscotti/home/<scenario-hash>` (created empty), cwd = repo root | No history or config leaks. |
| App noise | `HOLLA_NO_HISTORY=1`; never set `HOLLA_NO_MOTION`, `JACKIN_NO_MOTION`, `CLICOLOR_FORCE`, `FORCE_COLOR` | Same policy as today. |
| Motion | `--motion paused --frame N` wherever the binary has motion | Deterministic frames. |
| **New: tablepro** | add `--motion full\|reduced\|paused` and `--frame N` | tablepro has no determinism flags today (T6). |
| **New: clock** | `JUNIE_NOW=<RFC 3339>` read by a single `core::clock` seam, only if the audit (T6) finds a rendered wall-clock value | Removes settle races caused by time. |

### B4. Gate, artifacts, store

| Path | Content | Committed |
|---|---|---|
| `baselines/oracle/<app>/<group>/<leaf>[/<state>]/<cols>x<rows>/<color>.ansi` | normalized SGR dump + cursor trailer (`ESC[?25l`, or `ESC[?25h ESC[y;xH ESC[Ps q`) | yes |
| `baselines/oracle/ORACLE.toml` | `version`, `source_commit`, `tuiscotti_rev`, `harness_sha256`, scenario count per app, `approved_by`, date | yes |
| `target/tuiscotti/actual/<name>.{ansi,frame.json}` | every run | no |
| `target/tuiscotti/review/<name>/{expected,actual}.{png,html}`, `diff.png` | only on mismatch | no (CI artifact) |
| `target/tuiscotti/report.html` | table of statuses, links to review files | no (CI artifact) |

The old `snapshots/` tree (4 files × 7,550, 12 GB, stale, and HTML with an
absolute path) is removed from the working tree in T18, after the v0 import
has diffed against it. History keeps it.

**`store_integrity`** (default, non-ignored test, runs in the plain
`cargo nextest run`):

- each generated scenario has exactly one `.ansi`;
- no orphan files, and nothing but `.ansi` and `ORACLE.toml`;
- each file parses (normalized SGR + exactly one valid trailer);
- the counts in `ORACLE.toml` match;
- `harness_sha256` matches `tests/visual_baseline/**`.

### B5. Approval flow: explicit and local only

| Step | Command | Rule |
|---|---|---|
| 1. Run | `cargo nextest run --run-ignored only -E 'binary(visual_baseline)'` (filters in README) | Failures write review artifacts. |
| 2. Review | open `target/tuiscotti/report.html` | Look at every changed scenario. |
| 3. Accept | `cargo xtask visual accept <filterset>` | Copies actual `.ansi` files into the store and updates `ORACLE.toml` counts. **Refuses when `CI` is set.** There is no env-var bless (tuiscotti principle). |
| 4. Commit | one commit per intended change | The message names the visual change and the scenarios touched (AGENTS.md: a deliberate visual change is called out). |
| Render | `cargo xtask visual render <name>` | Approved `.ansi` to PNG/HTML for local viewing. |

## C. Capture matrix and runtime

### C1. Tiers

| Tier | Trigger | Scenarios | Sizes × colours |
|---|---|---|---|
| **smoke** | every PR, every push | all generated scenarios | `120x40/truecolor` + `72x20/nocolor` (widest common layout + smallest canonical size in mono, which catches layout and state-distinction bugs) |
| **full** | nightly 02:00 UTC, manual | all | canonical 5 sizes (72x20, 80x24, 100x30, 120x40, 160x50) × 5 colours (truecolor, 256, 16, none, nocolor) |
| **narrow** | nightly, manual | `SURFACES` + `OVERLAYS` only | `40x12`, `60x18` × truecolor, nocolor |
| **boot + journeys** | every PR (plain `cargo nextest run`) | 4 boot tests + journeys | 80x24 |

### C2. Coverage gaps to close (from research-infra §2)

| Gap | Today | Target | Task |
|---|---|---|---|
| Hover | 3 showcase roots | every `ELEMENTS` row in all 4 apps: menus, tabs, chips, close `×`, scrollbar thumbs, splitters, buttons, rows | T12 |
| Pressed | never captured | every element: press-without-release (mouse) and frozen keyboard flash (paused) | T13 |
| Disabled | ad hoc | every control with a disabled variant; check that hover and press do not change it | T14 |
| Selected + hover | ad hoc | every list or table with selection | T14 |
| Popups and modals | piecemeal | complete `OVERLAYS` inventory per app, open plus both dismiss paths | T15 |
| Sizes below 72x20 | none | narrow tier (40x12, 60x18) | T16 |
| Resize | 2 roots per app, jackin 0 | at least 4 per app, grow and shrink across a breakpoint | T16 |
| Colour levels | 5 | unchanged; every state checked at every level | — |

### C3. Runtime estimate and sharding

The planning figure is about **3 s per capture**: boot + steps at 120 ms +
content wait + 200 ms quiet, with binaries built at `opt-level = 1`. Today's
14 s is serial, under load, with a 400 ms settle. T9 measures the real
figure and this table is updated.

| Tier | Captures (est.) | CPU-s | Shards | Wall per shard (4 slots) |
|---|---|---|---|---|
| smoke | ~800 (302 roots + new coverage, × 2) | 2,400 | 2 | ~5 min + build |
| full | ~10,000 (7,550 + new coverage) | 30,000 | 12 | ~11 min + build |
| narrow | ~1,200 | 3,600 | 2 | ~8 min |

- **Sharding:** build once with `cargo nextest archive`; each shard runs
  `cargo nextest run --archive-file … --partition hash:k/N`.
- **Runner:** `ubuntu-24.04`. The `.ansi` gate is OS-independent. PNG and
  HTML bytes were identical on macOS arm64, Linux arm64 and Linux x86_64
  across 755 scenarios (research-formats), so review images also match
  local ones.
- **Slots:** `pty` test-group `max-threads = 4` on 4-vCPU runners. If the
  repo gets 2-vCPU private runners, double the shard count instead of
  oversubscribing, since settle races grow with load.
- **Upload:** `actions/upload-artifact@v4`, `if: failure()`, path
  `target/tuiscotti/{actual,review,report.html}`, 14-day retention, one
  artifact per shard (`visual-<tier>-<k>`).

## D. CI/CD

| Workflow | Trigger | Jobs | Required on PR |
|---|---|---|---|
| `ci.yml` (new) | PR, push to `design/interaction-polish`, `main` | `fmt` (`cargo fmt --check`), `clippy` (`--all-targets -D warnings`), `test` (`cargo nextest run --all-targets --profile ci`: unit + `store_integrity` + boot + journeys) | yes, all three |
| `visual.yml` (new) | PR, push | `build` (archive) → `smoke` × 2 shards → `report` (merge shard reports into the job summary) | yes: `visual-smoke` |
| `visual.yml` | `schedule` nightly | `build` → `full` × 12 + `narrow` × 2 | no; a red nightly is P1 and opens an issue via `gh issue create` |
| `visual.yml` | `workflow_dispatch` with input `tier: smoke\|full\|narrow\|capture` | `capture` runs full + narrow with failures allowed and uploads the whole `actual/` as `oracle-candidate` (input to E2) | — |
| `holla-fable-audit*.yml` (existing) | `holla-fable` only | replace `cargo test` with `cargo nextest run`; `ubuntu-22.04` → `ubuntu-24.04` | no |

- **Caching:** `Swatinem/rust-cache@v2`, keyed on `Cargo.lock` + toolchain;
  it covers the tuiscotti git checkout. Never cache `target/tuiscotti`.
- **Toolchain:** `dtolnay/rust-toolchain` at the version T0 sets (tuiscotti
  needs 1.98; ours is 1.88). Install nextest with
  `taiki-e/install-action@nextest`.
- **Never in CI:** `xtask visual accept`, or writes to `baselines/`.
- **Branch protection:** require `fmt`, `clippy`, `test` and `visual-smoke`
  on `design/interaction-polish`. On `main`, see §F (ratchet).

## E. Baseline lifecycle

### E1. Versions

| Version | When | Content |
|---|---|---|
| `v0` (interim) | right after T0 + T4 land | Full capture at that HEAD. Gives every design-backlog PR a visible diff. The committed `snapshots/` are pre-polish frames (16+ `src/` commits and 18 design commits newer; the tablepro hint copy already drifts), so they cannot be the gate. |
| `v0.n` | each design-fix PR | That PR re-approves only its own affected scenarios, named in the commit. |
| `v1` (published oracle) | the design-review backlog is closed (every finding fixed or recorded with a reason) and the PO signs off | Full re-capture at HEAD on `ubuntu-24.04` (dispatch `tier=capture`); diff against `v0.n` must be empty, or explained. Then `ORACLE.toml` `version = 1`, `source_commit = <sha>`. |
| `vN` | any later intended visual change on this branch | Same flow; `version` increments. Git history of `baselines/oracle/` is the version history. **No new tags; `visual-baseline` is untouched.** |

### E2. Capture procedure (v0, v1)

1. Dispatch `visual.yml` with `tier=capture` and download `oracle-candidate`.
2. Run `cargo xtask visual import <dir>`. It writes `baselines/oracle/` and
   prints a change list grouped by app/group against the previous store
   (for v0: the old `snapshots/*.ansi`, read through the cursor-less
   compare).
3. Reviewer: every change group maps to a design commit or audit finding.
   Write the mapping into the commit message.
4. Accept locally, commit, push (small commits: one per app).

### E3. Publishing to `main`: committed copy, synced by PR

**Choice: `main` keeps a byte-for-byte copy at `baselines/oracle/`, updated
only by a sync PR from an approved commit on this branch.**

| Option | Verdict | Reason |
|---|---|---|
| **Committed copy in `main`, synced by PR** | **chosen** | 71 MB working tree, about 10 MB packed: cheap. CI has no cross-branch fetch. Each oracle bump is a reviewable diff on `main`. The gate stays reproducible offline at any `main` commit. |
| Orphan `visual-oracle` branch | rejected | Needs a cross-ref fetch in every job. A `main` commit's gate is not self-contained, and history and rollback are harder. |
| CI artifact / release asset | rejected | Artifacts expire. Release assets add mutable state, and the tag is reserved. Not reviewable in a PR. |
| Git LFS | rejected | Not needed at 71 MB, and it adds a dependency. |

**Sync mechanics** (T20):

- `cargo xtask visual sync-oracle --from <sha>` checks `ORACLE.toml` at that
  SHA, then copies `baselines/oracle/` and `tests/visual_baseline/` into
  `main`'s tree.
- On `main` the harness lives at `crates/termrock-conformance/tests/visual_baseline/`
  with `VISUAL_BIN_DIR` set.
- `main`'s `store_integrity` verifies `harness_sha256`, so `main` cannot
  edit the harness or the store except by a sync.
- `main`'s existing `baselines/tuiscotti-v1/` (converted pre-polish frames,
  10 artifacts per scenario) and `required_cases.json` renaming are deleted
  in the same PR. One naming scheme: this branch's.

## F. Refactor execution on `main`

### F1. Gate: the ratchet

- `main` runs the same `visual.yml` with the synced harness and oracle.
- `tests/visual_baseline/progress/main-passing.txt` lists the scenarios that
  pass on `main`.
- **Required check `visual-ratchet`:** no scenario on the list may fail.
  Newly passing scenarios must be added (`cargo xtask visual ratchet`
  updates the list; it refuses in CI).
- The job summary prints passing/total per app. The milestone is met when
  `main-passing.txt` equals the full scenario list. At that point the
  ratchet becomes the plain `visual-smoke` / `visual-full` gate.

Baseline today (research-main): 0 of the sampled roots pass on `main`.
Totals per app follow the current store: tablepro 1,100, showcase 2,100,
jackin 1,225, holla 3,125 (they grow with §C2).

### F2. CLI parity (land first: without it most scenarios never reach their ready needle)

| Item | Reference behaviour | `main` today | Task |
|---|---|---|---|
| Parser | clap on all four binaries, short flags `-c/-p/-m/-f` | hand-parsed | T22 |
| `--help` | prints usage, exit 0 | tries to open the terminal | T22 |
| Unknown `--page` / `--color` / `--scenario` | error, non-zero exit | silent fallback to the default | T22 |
| `--motion reduced` | all four binaries | showcase lacks it | T22 |
| Showcase `diff` page and slug aliases (`editabletables`, `chipsselects`, `codeeditor`, `datagrid`) | present | Diff missing (22 vs 23 pages) | T23 |
| Holla `parity-*` scenarios (23, deterministic at `--frame 40`) | present | rejected "unknown scenario" | T27 |
| Boot needles on the first frame (`Junie Design system`, `holla❯`, `jackin❯`, tablepro title) | yes | holla and jackin never show them | T25–T27 |
| tablepro `--motion` / `--frame` | added by T6 | absent | T22 |
| `main`-only flags (`--theme`) | — | keep; the default must render the reference | — |

### F3. Library defects to port into termrock (one PR each, behaviour configurable per AGENTS.md)

| # | Reference behaviour (this branch) | `main` gap | Crate on `main` |
|---|---|---|---|
| L1 | Exactly one focus indicator `▎` (`Theme::gutter_symbol`/`gutter`), in the topmost layer only | `▎` painted on every row of sidebar and lists | `termrock-theme`, `termrock-navigation` |
| L2 | `Theme::cursor_row`, `CursorStyle::Fill` for owned lists | missing | `termrock-theme`, `termrock-overlays` |
| L3 | Neutral "active" treatment (current page `›`, active tab, chosen value: accent only while focused) | bold `#b3b3b3` vs muted `#4d4d4d`; `› Lists` missing | `termrock-theme`, `termrock-navigation` |
| L4 | State model: rest/hover/pressed/focused/selected/disabled resolver + per-plane ladder (`states.md` §1), explicit 256/16/mono tables | `lift()`, BF12 disabled-vs-hover defect | `termrock-theme` |
| L5 | `popup::Dismiss` + `backdrop_id(owner)` light dismiss; dialog dismiss resolves to cancel | `layer::Dismiss` with different semantics | `termrock-runtime` (layer), `termrock-overlays` |
| L6 | Overlay layer: `overlay` / `flush_overlays` / `begin_modal` / `modal_backdrop`, independent of draw order | `LayerSpec`/`Backdrop`, no flush API | `termrock-runtime` |
| L7 | `MenuBar::hover_switch`, `Select::{max_rows, placement, popup_width}` | `hover_switch`, `popup_width` missing | `termrock-overlays` |
| L8 | `segments` widget | missing | `termrock-navigation` or `termrock-controls` |
| L9 | `Table` widget (distinct from grid) | missing | `termrock-grid` |
| L10 | Job-control suspend (SIGTSTP/fg, termios restore) + panic restore shows cursor (BF07) | missing | `termrock-session` |
| L11 | Reveal for select values/options and grid cells (2f8749074) | check | `termrock-overlays`, `termrock-grid` |
| L12 | List metadata ("1–13 of 20", "2 selected"), standalone scrollbar | missing / internal only | `termrock-navigation` |
| L13 | Pressed rows on cells and keyboard flash (d5e1595a8); disabled menu items with a reason (00643499a) | check | `termrock-controls`, `termrock-overlays` |
| L14 | BF01–BF14 (`docs/refactoring-plan/branch-diff-foundation.md`): text normalization, grapheme selection and word motion, paste outcome, fuzzy match, wrap, paint clusters, split collapse, ASCII glyphs, chord identity, input re-delivery | as listed there | various |

Every row is checked against the oracle: the PR names the scenarios it
flips to passing.

### F4. App order and milestones

| M | Scope | Exit criterion (ratchet) |
|---|---|---|
| M0 | Harness + oracle synced to `main`; `visual-ratchet` required; `main`'s AGENTS.md gains the reference's visual-state and tuiscotti rules (`CLAUDE.md` stays a symlink) | job reports passing/total; list may be empty |
| M1 | CLI parity (F2) | 100% of scenarios reach their ready needle (a "booted" metric, reported separately) |
| M2 | L1–L7 shared defects | sharp rise in every app; no app-level code changed |
| M3 | **tablepro** (closest: `▎`, Scratch grouping, Explorer width, schema icons, extra "SQL query" frame) | tablepro 100% |
| M4 | **showcase** (Diff page, legend row, nav labels, list metadata) | showcase 100% |
| M5 | **jackin** (re-implement chrome: `jackin❯ File Go Help`, Accounts tree; a different product generation today) | jackin 100% |
| M6 | **holla** (re-implement: copy, scope tabs, hints, 23 `parity-*`) | holla 100% |
| M7 | Nightly full + narrow green on `main`; boot + journeys green | 1:1 per §A. The PO then decides about the tag. |

**Slice rule:** one PR = one library defect (L*) or one surface group of one
app. It must flip at least 1 scenario, regress none, and pass the repo gate.
Re-implementation follows the reference *behaviour* through the termrock
API; no cherry-picks from either lineage (`main-port-analysis.md`).

## G. Risks and upstream work

| Risk | Impact | Mitigation |
|---|---|---|
| tuiscotti churn: single maintainer, research grade, breaking `main` | re-port on every bump | Exact `rev` pin. Each bump is its own PR: re-run full, and only review-artifact changes are allowed without approval. `.ansi` did not churn across tool versions (7,550/7,550 identical). |
| Settle races / flakes (no clock control) | red CI with no cause | Content-gated waits, paused motion, tablepro flags, `JUNIE_NOW`, 4 slots per runner. nextest `retries = 0` (a flake is a bug); quarantine list with an owner and expiry. |
| Pressed is a 140 ms flash | cannot be captured | Mouse: hold without release. Keyboard: frozen by paused motion. If the app does not hold pressed while the button is down, fix the component (T13). |
| Oracle captured before the design backlog closes | `main` chases a moving target | `main` syncs only `ORACLE.toml` `version ≥ 1`. Until then `main` works on F2 + L1–L7 (stable intent). |
| Harness divergence between branches | `main` gates a weaker set | `harness_sha256` in `store_integrity`; sync-only edits on `main`. |
| Hidden-cursor coordinate noise | false failures | The trailer encodes position only when the cursor is visible (decided in research-formats). |
| Terminal modes not in any frame | regressions invisible | Boot tests (`Observation.state.modes`). |
| CI minutes (full ≈ 8 runner-hours/night) | cost | Full runs nightly only; skip it if there are no commits since the last green nightly. |
| `main` needs the 1.98 toolchain like this branch | build breaks | T0 sets it; T19 applies the same pin on `main`. |

**Upstream tuiscotti PRs** (file a PR, then delete our wrapper; file:line
from research-formats):

| # | Change | Location |
|---|---|---|
| U1 | `ansi_dump_with_cursor(&Frame)` and inverse `formats::ansi::frame_from_ansi` | `tuiscotti-render/src/render/text.rs:46` |
| U2 | Allow the cursor trailer in `assert_normalized_sgr` | `formats/ansi.rs:30` |
| U3 | `GroupedStore::with_artifacts(ArtifactSet::AnsiCursor)` | `grouped/types.rs:71` |
| U4 | Require and read only configured artifacts | `check.rs:84-104`, `check_gates.rs:66` |
| U5 | PNG/HTML as evidence only; rebuild expected from `.ansi` on mismatch | `check.rs:132-135`, `248-251` |
| U6 | Copy and hash only configured files | `mutate.rs:22-27`, `seal.rs:60-75,110-135` |
| U7 | Provenance stores `argv[0]` file name only (no absolute path) | `frame_from_screen` provenance |
| U8 | CLI `accept` for the grouped store with a filter | `crates/tuiscotti-cli/src/cli.rs:159` |

## P. Performance (table scrolling)

PO request (2026-10-16): table components lag when scrolling. Status: P1–P4
fixed with guards; remaining risks recorded. Findings,
measurements and status live in
`docs/audits/design-review-2026-10/performance.md`.

| Rule | Detail |
|---|---|
| Measure first | `tests/table_perf.rs` (widgets alone, 500 / 10k / 100k rows) and `src/bin/tablepro/perf_tests.rs` (real app: customers table, query results) print per-event and per-frame cost; run them with `--cargo-profile release --run-ignored only --no-capture`. |
| Fix in the library | Runtime input coalescing, bounded text measurement, grid width sampling; every knob configurable with a default. |
| Guard deterministically | Counting assertions (frames per burst, regions per frame, bytes read) over wall-clock thresholds; any time budget is generous and documented. |
| No visual change | Perf work must leave every rendered cell identical; a visual diff is a bug. |

## H. Task list

Owner types: **harness** (tests/visual_baseline, xtask), **app** (one binary),
**lib** (shared layer), **ci** (workflows), **main-lib** / **main-app**
(on `main`, termrock crates). Every task: one worktree subagent, small
commits, `cargo nextest run --all-targets` + clippy + fmt green before push.

| # | Task | Owner | Files | Acceptance |
|---|---|---|---|---|
| T0 | *(in flight, other agent)* Migrate the harness from tuisnap to tuiscotti latest `main` | harness | `Cargo.toml`, `tests/visual_baseline/*`, `.config/nextest.toml` | builds; `store_integrity` green; one root passes |
| T1 | Cursor-trailer `.ansi` gate wrapper + review artifacts on mismatch | harness | `tests/visual_baseline/support.rs`, `tests/visual_baseline/ansi_cursor.rs` | the caret-move demo fails the gate; round-trip unit tests; review files written |
| T2 | One test per scenario via `libtest-mimic`, `harness = false` | harness | `Cargo.toml` `[[test]]`, `tests/visual_baseline/main.rs` | `cargo nextest list` shows 7,550 named trials |
| T3 | Environment pinning (B3) + `VISUAL_BIN_DIR` resolution | harness | `support.rs` | an env-leak test (`FORCE_COLOR=1` set in the parent) has no effect |
| T4 | `xtask` crate: `visual accept\|render\|import\|ratchet\|sync-oracle`; refuses under `CI` | harness | `xtask/`, `.cargo/config.toml`, root `[workspace]` | `CI=1 cargo xtask visual accept` exits non-zero |
| T5 | New store `baselines/oracle/` + `ORACLE.toml` + new `store_integrity` | harness | `baselines/`, `tests/visual_baseline/main.rs` | integrity green on the empty store only while `ORACLE.toml` says `pending = true`; captures fail as missing approval |
| T6 | Determinism: tablepro `--motion`/`--frame`; audit rendered time; `JUNIE_NOW` seam if needed | app | `src/bin/tablepro/*`, `src/core/` | two runs of the full tablepro set are byte-identical |
| T7 | `ci.yml` (fmt, clippy, nextest `ci` profile) | ci | `.github/workflows/ci.yml` | green on PR |
| T8 | `visual.yml`: build archive, smoke × 2, nightly full × 12 + narrow × 2, dispatch `capture`, artifacts | ci | `.github/workflows/visual.yml`, `.config/nextest.toml` (`pty` 4 slots) | a deliberate 1-cell change fails smoke and uploads review PNGs |
| T9 | Measure per-capture time on CI; update §C3 | ci | `docs/visual-testing/plan.md` | numbers from a real run |
| T10 | Fix `holla-fable-audit*.yml`: `cargo test` → `cargo nextest run`, ubuntu-24.04 | ci | `.github/workflows/holla-fable-audit*.yml` | `grep -r "cargo test" .github` empty |
| T11 | Convert 302 hand-written roots to `SURFACES`/`JOURNEYS`/`RESIZES` tables, names unchanged | harness | `tests/visual_baseline/scenarios/*.rs` | generated name set ⊇ old root names |
| T12 | `ELEMENTS` inventory + hover for all 4 apps (one PR per app) | harness | `scenarios/<app>.rs`, `pointer.rs` removed | every interactive element listed has a hover scenario |
| T13 | Pressed: press-without-release + paused keyboard flash; component fix if pressed is not held | harness + lib | `scenarios/*`, `src/widgets/*` | pressed frames differ from hover and rest at all 5 levels |
| T14 | Disabled and selected+hover rows | harness | `scenarios/*` | each disabled control: hover/press frames equal rest |
| T15 | `OVERLAYS` inventory per app: open, Esc dismiss, outside-click dismiss | harness | `scenarios/*` | every `RenderCtx::overlay`/`begin_modal` call site maps to ≥1 row (grep check in a test) |
| T16 | Narrow tier (40x12, 60x18) + ≥4 resize rows per app | harness | `scenarios/*`, `MATRIX` | jackin has resize rows |
| T17 | Boot tests (modes 1049, 1003/1006, 2004; restore; exit 0) + port `holla_pty.rs`, `terminal_suspend.rs` to tuiscotti | harness | `tests/boot.rs`, `tests/journeys/*` | no hand-rolled `openpty` remains |
| T18 | **Oracle v0**: dispatch `capture`, import, review vs old `snapshots`, commit per app | harness | `baselines/oracle/`; `git rm -r snapshots/` | smoke + full green on HEAD; `snapshots/` gone |
| T19 | Design-review backlog (existing audits): each fix re-approves its scenarios | lib / app | per audit | audit files record each finding as fixed or with a reason |
| T20 | **Oracle v1** + `sync-oracle` PR to `main` (deletes `baselines/tuiscotti-v1`, `required_cases.json`; toolchain pin) | harness | `main`: `baselines/oracle/`, `crates/termrock-conformance/tests/visual_baseline/` | `main`'s `store_integrity` green; harness hash matches |
| T21 | `main`: `visual.yml` + `visual-ratchet` required; merge visual-state rules into `main`'s AGENTS.md | ci | `main`: `.github/workflows/*`, `AGENTS.md` | ratchet job reports passing/total |
| T22 | `main` CLI parity: clap, `--help`, unknown-value errors, `--motion reduced`, tablepro flags | main-app | `crates/*-ui/src/app.rs`, app bins | journey tests for help and errors pass |
| T23 | `main` showcase Diff page + slug aliases | main-app | `crates/showcase-*` | `--page diff` scenarios reach ready |
| T24 | `main` L1–L7, one PR each | main-lib | termrock crates per F3 | each PR flips scenarios, regresses none |
| T25 | `main` tablepro to 100% (surface-group slices) | main-app | `crates/tablepro-*` | ratchet: tablepro N/N |
| T26 | `main` showcase, then jackin, to 100% | main-app | `crates/showcase-*`, `crates/jackin-preview-*` | ratchet N/N per app |
| T27 | `main` holla re-implementation + 23 `parity-*` scenarios | main-app | `crates/holla-*` | ratchet N/N |
| T28 | `main` L8–L14 | main-lib | termrock crates | as T24 |
| T29 | M7 sign-off: nightly full + narrow green on `main`; report to PO | main session | `docs/visual-testing/` | §A table all green |
| T31 | Table scrolling performance (§P): coalesce input bursts in the runtime, bound per-cell text work, configurable width sampling, regression guards | lib | `src/runtime.rs`, `src/ui/text.rs`, `src/ui/reveal.rs`, `src/widgets/grid.rs`, `tests/table_perf.rs` | a wheel burst draws one frame; frame cost independent of row count and cell length; before/after in `performance.md` |
| T30 | Upstream U1–U8 to tuiscotti; switch our wrapper to `ArtifactSet::AnsiCursor`; bump the pin | harness | tuiscotti fork PRs, `support.rs` | wrapper deleted; gate unchanged (full run green) |

**Order and parallelism:** T0 → T1–T5 (parallel; disjoint files) → T6–T10
(parallel) → T11 → T12–T17 (one agent per app) → T18 → T19 (ongoing) →
T20 → T21 → T22–T23 → T24 → T25 → T26 → T27; T28 and T30 run alongside.
