I found one real-binary PTY suite in `tests/visual_baseline/` (via tuisnap). It is not run in CI. Its committed `snapshots/` (12G) is stale in two ways:

- **The HTML artifact records the checkout's absolute path,** so every HTML comparison fails anywhere but `/Users/donbeave/Projects/terminal-components-claude`.
- **The tablepro connections screens show old content.** The approved frames predate the hint-text fix in 8049ca930.

Everything below ran in an isolated `git archive HEAD` export under `.../scratchpad/infra/head` with its own target dir. Nothing in the repo was touched.

## 1. What exists

**`tests/visual_baseline/`** (main, support, audit, holla, jackin, pointer, showcase, tablepro)
- **Real binaries in a PTY.** It spawns the four built binaries through tuisnap's PTY session, which uses the termpane/termlens terminal emulators. It does not render in-process. Mouse, wheel, drag, hover and resize go through the live session in `pointer.rs`; hover is sent as raw SGR motion bytes `CSI <35;c;r M`.
- **Each capture:** spawn, wait for a boot text, send steps 120 ms apart (`key`, `type:`, `sleep:`, `wait:`), wait for 400 ms of quiet (`SETTLE`), then gate. Default timeout is 8 s; slow screens override it.
- **Four artifacts per capture,** gated at four levels: `.ansi` (cell-exact), `.txt` (content), `.html` (byte-exact, with a base64 PNG and frame JSON inside) and `.png` (pixel threshold 1.0, glyphs drawn with the fonts vendored in tuisnap).
- **Matrix:** every root runs 5 sizes (72x20, 80x24, 100x30, 120x40, 160x50) × 5 colours (`--color truecolor|256|16|none`, plus `nocolor` = `NO_COLOR=1` with no flag).
- **Storage:** `snapshots/<app>/<group>/<leaf>/<cols>x<rows>/<color>.{ansi,txt,html,png}`, committed. Actuals, diffs and the report go to `target/tuisnap/{actual,diff,report.html}`.
- **Updating baselines:** only by hand with `tuisnap accept --grouped --store snapshots --all` (installed at `~/.cargo/bin/tuisnap`). There is no auto-bless.
- **Default run:** all 303 capture tests are `#[ignore]`. A plain run only does `store_integrity`, which checks that names match the suite and that each scenario has exactly the four files.
- **Config:** `.config/nextest.toml` puts them in a `pty` group (8 threads), with a `ci` profile. The suite replaced `tools/tuisnap_baseline.sh`, which is now deleted.

**`showcase_visual_baseline`** (`src/bin/showcase/app_tests.rs:641`)
- In-process ratatui `TestBackend`. It renders every showcase page after one Tab, at 5 sizes × 4 levels (TrueColor/256/16/Mono).
- It stores an FNV hash per page in `tests/showcase_baseline.txt`, leaving the sidebar out. `UPDATE_BASELINE=1` rewrites it.
- Passes at HEAD.

**`tests/holla_pty.rs`, `tests/terminal_suspend.rs`**
- Real holla binary on a hand-rolled `openpty`. They check behaviour, not images: the colour/`NO_COLOR` policy, input floods, and SIGTSTP/fg termios restore. They are not image baselines.

**`focus_gutter.rs`, `choice_containment.rs`**
- In-process buffer assertions across colour levels.

**tuisnap** (rev 2d43458)
- A library used as a dev-dependency (`tuisnap::pty::{Session, run_once}`, `grouped::GroupedStore`, `VENDORED_FACES`).
- It also offers an in-process `tuisnap::ratatui::draw_frame` path, which this repo does not use.

**Runtime**
- `cargo nextest run --all-targets`: 23.7 s for the tests (557 run, 303 skipped).
- Visual subset: 6 root tests (150 captures) took 360 s wall-clock on a loaded machine. That is about 14 s per capture, since the 25 combos in each test run one after another.
- Extrapolating to all 303 tests on 8 slots gives roughly 3–4 hours. The docs say the scrolling captures alone take about 2.5 min each (`docs/baseline/tuisnap-coverage.md:138`).

**CI**
- No visual suite runs in CI. `.github/workflows/holla-fable-audit*.yml` only trigger on `holla-fable`, and they run `cargo test`, which breaks the nextest-only rule.

**Scripts**
- The only scripts are `docs/refactoring-plan/evidence/*.py` and their copies under `refactoring-tasks/`. They are plan, proof and bootstrap tooling, not capture generators.

## 2. Scenario coverage

- **Totals:** 302 roots × 25 combos = 7550 scenarios = 30200 files.
- **Naming:** `<app>/<group>/<surface>[/<state>]/<cols>x<rows>/<color>`. Test function names come from the representative capture, e.g. `holla_parity_discovery_default_120x40_truecolor`, or `*_matrix` in `pointer.rs`.

| App | Roots | Scenarios | Groups (roots) |
|---|---|---|---|
| holla | 125 | 3125 | flows 85, parity 23 (all `parity-*` scenarios), concept 9, fade 4, audit 2, resize 2 |
| showcase | 84 | 2100 | flows 46, pages 18, fade 10, audit 5, hover 3, resize 2 |
| jackin | 49 | 1225 | manager 10, editor 7, capsule 5, accounts 5, scenarios 6, settings 6, intro 3, cockpit 3, usage 2, audit 2 |
| tablepro | 44 | 1100 | table 10, connections 8, query 8, overlays 6, workbench 4, ack 4, resize 2, fade 1, audit 1 |

The full list of roots is in `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/infra/roots.txt`.

**Gaps**
- **Hover:** only 3 roots (showcase buttons, lists, tables). None in tablepro, jackin or holla, and none for menus, tabs, chips, close `×`, scrollbar handles or splitters.
- **Pressed:** never captured. tuisnap has `click` (press and release together) and `drag`, but no press-and-hold. Pressed is a 140 ms flash in `docs/audits/design-review-2026-10/states.md`, so it is timing-dependent.
- **Disabled, and selected + hover combined:** not systematically covered.
- **Popups and modals:** covered piecemeal (dialogs, pickers, context and menus, help overlays, tablepro overlays, confirms). There is no rule like "every overlay × every app".
- **Sizes:** nothing below 72x20 and no odd or very wide sizes. Resize only has 2 roots per app (jackin has none).
- **Dropped on purpose:** wall-clock and mid-animation states, as listed in `docs/baseline/snapshots-v2.md` §drops (spinners, jackin rain, streaming TTY).

## 3. Determinism

**What keeps captures reproducible**
- **Paused motion:** `--motion paused --frame N` is used in 243 holla cases, 25 jackin and 9 showcase cases (scrolling at frame 1600, progress).
- **Seeded data:** the simulations are seeded (`MOTION_SEED`).
- **Clean environment:** each capture strips `NO_COLOR`, `HOLLA_NO_MOTION`, `JACKIN_NO_MOTION`, `CLICOLOR_FORCE` and `FORCE_COLOR`, and sets `HOLLA_NO_HISTORY=1`.
- **Explicit colour:** `--color` is always passed, and `nocolor` uses the real `NO_COLOR=1`.
- **Fixed inputs:** HTML provenance has `created_unix: 0`, the fonts are vendored in tuisnap, and readiness waits on text rather than sleeps (e.g. `wait:739.63s`, `wait:7 of 7`, `of 23`).

**CLI flags per binary**
- `showcase`: `-c/--color`, `-p/--page`, `-m/--motion full|reduced|paused`, `-f/--frame`
- `jackin-preview`: `--color`, `-s/--scenario`, `--motion`, `--frame`
- `holla`: `--color`, `--scenario`, `--motion`, `--frame`
- `tablepro`: only `--color` and `--connect`. It has no motion or frame pinning.

**Risks**
1. **Path in the HTML gate (proven).** The approved HTML has `"argv":["/Users/donbeave/Projects/terminal-components-claude/target/debug/showcase",...]`. My clean export failed the HTML gate on 150 of 150 captures even where ansi, txt and png were byte-identical. Any CI checkout or other target dir will fail every HTML gate.
2. **Settle timing under load.** The 400 ms quiet window has already caused races (see `snapshots-v2.md` notes on jackin-accounts and scrolling). Showcase mostly runs with full motion. tablepro cannot be pinned.
3. **Very slow, long-running captures.** About 14 s per capture under load, with timeouts up to 60 s and `terminate-after = 10`.
4. **PNG fonts across OSes.** Pixel gating across macOS and Linux CI is unverified, and the baselines were made on macOS.
5. **Approval is all or nothing.** It is fail-closed: a single drift fails all 25 combos of a root's test.

## 4. Is `snapshots/` current?

**Size**
- `du -sh snapshots` = 12G. All 30200 files are tracked (no LFS), and `.git` is 7.1G.
- HTML is 10.5 GB (about 1.4 MB each), PNG 2.6 GB, ansi 0.07 GB, txt 0.02 GB.

**Age**
- Last changed in 892186260 (2026-09-14).
- There are 16 `src/` commits since then, including 8049ca930 (hint copy), 5bf3f4311 (modal backdrop), ba849cd89 (pressed rows), fd58bb7c9 (busy button), 36ce96588 (narrow dialogs) and a67e7c787 (cursor_row).

**Regeneration of 6 roots at HEAD** (150 captures; actuals in `.../scratchpad/infra/head/target/tuisnap/actual/`)

| Root | txt | ansi | png | html |
|---|---|---|---|---|
| showcase/pages/overview | same | same | same | 25 differ (path only) |
| showcase/pages/lists | same | same | same | 25 differ (path only) |
| showcase/hover/buttons | same | same | same | 25 differ (path only) |
| jackin/manager/menu_open | same | same | same | 25 differ (path only) |
| holla/parity/discovery | same | same | same | 25 differ (path only) |
| tablepro/connections/default | 25 differ | 25 differ | 25 differ | 25 differ |

The tablepro change is real content drift from 8049ca930: `E Edit  D Delete` became `e Edit…  d Delete…`.

So the store is stale wherever the post-Sep-14 fixes touch a rendered screen. I did not measure how many roots that is; it needs a full regeneration.

## 5. Recommendations for a CI suite that runs the real binaries

1. **Remove the path from what gets gated.** Patch tuisnap provenance (or the `support.rs` gate) so argv is stored as the binary name only, or so provenance is left out of the HTML byte gate. Then re-bless once. Without this no CI or cross-branch comparison works.
2. **Gate on ansi and txt; publish png and html for review.** ansi and txt are tiny and exact. Keep png and html as review artifacts, or gate png per OS. That drops about 13 GB of blobs from git. The 4-file `store_integrity` rule would need relaxing to match.
3. **Keep large artifacts out of the main tree.** Move `snapshots/` to Git LFS, a separate `visual-baselines` branch, or a release asset per commit. Do not reuse the `visual-baseline` tag, which is frozen.
4. **Pin a Linux runner image** (for example a container with a fixed Rust and tuisnap rev). Generate the baseline there for `main` first, then capture the other branch with the same harness and image, and diff with `tuisnap report --grouped` (actuals vs a checked-out baseline folder). The harness is unchanged across branches, so the scenario set is identical as long as `tests/visual_baseline` is taken from one ref.
5. **Run a tiered CI job.**
   - Every PR: one size × 2 colours (120x40 truecolor and nocolor) per root, about 600 captures.
   - Nightly or on demand: the full 25-combo matrix, sharded across jobs with `nextest --partition`.
   - Upload `target/tuisnap/{actual,diff,report.html}` as an artifact. Use the existing `ci` nextest profile.
6. **Add determinism hooks.** Give tablepro `--motion` and `--frame`, and default captures to `--motion paused` where motion exists. Optionally add an env var for a fixed clock so settle races disappear and the 400 ms waits can shrink.
7. **Close the coverage gaps.**
   - A press-without-release mouse step (`CSI <0;c;r M`) captured in paused mode, to get pressed states.
   - Hover across all four apps.
   - Disabled, and selected + hover.
   - Sizes below 72x20.
   - A generated "every overlay" list per app.
   - Generate cases from a state table, like the `audit.rs` matrices, instead of 300 hand-written `Case::new` lines.
8. **Fix the existing workflows.** They must use `cargo nextest`, not `cargo test`.