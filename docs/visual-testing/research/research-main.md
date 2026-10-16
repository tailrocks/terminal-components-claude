I set up detached worktrees for `main` (032e9faa3) and for `design/interaction-polish` (fb70b7ddb) under the scratchpad, ran everything there, and removed both with `git worktree remove`. No ref changed: `visual-baseline` is still 4a79c0a2d and `main` is still 032e9faa3. I read AGENTS.md on both refs.

**Bottom line:** the two branches share no commits after Sep 3. Main has the full termrock crate structure, and its harness is already pointed at the visual-baseline frames. But main fails every sampled capture, and some of its apps are different products. The reference matches its frozen snapshots exactly; only its HTML gate fails, because of a path recorded inside the file.

## 1. Main's structure

**History**
- `main` and `visual-baseline` split at cc14dd6be (2026-09-03). That is also the merge-base of `main` and `design/interaction-polish`.
- `main` is 831 commits past that point (395 first-parent, 11 merges). The other side has 77 commits up to the tag (4a79c0a2d), which `main` does not contain.
- `design/interaction-polish` is `visual-baseline` plus 18 commits (overlay layer, menus, dismiss policy, `cursor_row`/`CursorStyle`, AGENTS rules). That is fewer than the ~25 you were told.
- Main's phases:
  - Sep 3–5: component foundations (7899678da), Slice 2–4 components, conformance and digest gates, the `tui`/`apps` workspace.
  - Then integration and consolidation PRs #13–#22: holla/jackin integration (c3108b579), "port reviewed Termrock behavior" (c8a231f60), refactor-proof harness (c14252517).
  - Finally 032e9faa3 "land the termrock crate split" (#24, 2026-10-15): one squash touching 77,622 files, about 14.2M insertions, mostly evidence.

**Workspace** (root `Cargo.toml`)
- 46 members, lib name `termrock`:
  - Library crates: `termrock-{core,layout,theme,text,render,collections,runtime,controls,fields,feedback,navigation,viewport,overlays,grid,editors,forms,terminal,session}`, plus the `termrock` facade.
  - Each app is split into crates: `showcase-{data,demos,ui}`, `jackin-preview-{domain,sim,presentation,host-ui,capsule-ui,app}`, `holla-{domain,catalog,plan,sim,ui}`, `tablepro-{domain,sql,demo,ui}`.
  - Tooling: `termrock-test-support`, `termrock-conformance`, `termrock-xtask`, `termrock-visibility-publisher`, `tools/refactor-proof`.
- `crates/termrock-e2e` and `crates/termrock-visibility-tests` exist on disk but are not workspace members.
- Strict lints: deny `panic`, `indexing_slicing`, `unwrap`/`expect`, `print_stdout`.

**API shape**
- `App` trait (`crates/termrock-runtime/src/runtime.rs:43`): `update(&mut Cx) -> Response<()>`, `draw(&self, &mut Ui)`, `keymap()`, `min_size()`, `on_esc()`.
- Widgets are generic over key and row types and come as `XState` + `XAction`/`XCmd` + a widget struct with `update(cx)` / `draw(ui, area)` / `measure()`. Examples: `Tabs<'a,T,K,R>`, `ChipBar`, `SplitPane`; see `crates/termrock-controls/src/button.rs:280-516`.
- Layers: `LayerSpec`, `Anchor`, `Backdrop`, `Dismiss`.
- `termrock::author` is the prelude.

**Build and tests**
- `cargo build --all-targets` at the root builds only the 6 default-members. It passes.
- `cargo build --workspace --all-targets` passes (1 warning).
- `cargo nextest run --workspace --all-targets`: 3,576 passed, 0 failed, 620 skipped. The skipped ones are the ignored PTY visual captures, so a green run says nothing about appearance.
- The default-member nextest run: 1,479 passed.

**Binaries and flags on main**

| Binary | Flags | Notes |
|---|---|---|
| `showcase` | `--theme`, `--color`, `--page`, `--motion full\|paused`, `--frame` | Hand-parsed in `crates/showcase-ui/src/app.rs:1339`. No `--help`: it tries to open the terminal. An unknown `--page` or `--color` value is silently ignored, so you get the default page or colour level instead of an error. |
| `tablepro` | `--color`, `--connect`, `--theme junie\|paper` | |
| `holla` | `--scenario`, `--motion`, `--frame`, `--color` | 11 scenarios. |
| `jackin-preview` | `--scenario`, `--motion`, `--frame`, `--color`, `--theme` | |

The reference uses clap with `-c/-p/-m/-f` everywhere, and `--motion reduced` on all four binaries.

**Plans describing the target**
- On main: `GOAL.md` (restore historical UI while keeping the refactor), `REFACTORING_GOAL.md` (marked historical), `docs/REFACTORING_AUDIT_REPORT.md`, `STATUS.md` (Refactor NOT_READY, reference not qualified), `docs/plans/*`.
- On the reference: `docs/refactoring/{goal.md,plan.md,main-port-analysis.md}`, `docs/refactoring-plan/` (branch-diff ledgers, BF01–BF14 library defects), and `refactoring-tasks/terminal-components/completion/` (770 files).
- `main-port-analysis.md` already recommends porting the library and harness and not taking main's apps "as-is" (it is dated at main 7b27732a).

## 2. Mapping: reference to main

| Reference | Main equivalent |
|---|---|
| `src/core/{event,id}` | `termrock-core/{event,id,keys,intent,response}` |
| `src/core/{focus,hit}` | `termrock-runtime/{focus,hit}` |
| `src/core/scroll` | `termrock-collections/scroll`, `termrock-runtime/scroll_region` |
| `src/core/text`, `src/ui/text` | `termrock-text/*` |
| `src/ui/ctx` (`RenderCtx`, `overlay`/`flush_overlays`/`modal_backdrop`/`begin_modal`) | `termrock-runtime/ui` + `layer` (`Ui`/`Cx`, `LayerSpec`, `Backdrop`). Those `RenderCtx` method names are missing. |
| `src/ui/popup` (`popup::Dismiss`, `backdrop_id`) | `layer::Dismiss`/`DismissReason`, with different semantics |
| `src/ui/fade` | `termrock-runtime/ui/paint.rs` (fade) |
| `src/ui/layout` | `termrock-layout` |
| `theme.rs` | `termrock-theme/*` (tokens, recipe, downgrade, glyph). **Missing:** `gutter_symbol`, `cursor_row`, `CursorStyle`. |
| `runtime.rs` (`TerminalSession`, suspend) | `termrock-session` + `termrock-runtime`. **No job-control suspend** (no SIGTSTP). |
| widgets `button`, `brand`, `empty`, `panel`, `props`, `choice` | `termrock-controls/*` (+ `radio_field`) |
| `splitter` | `termrock-controls/split` (`SplitPane`) |
| `input`, `textarea`, `field_common` | `termrock-fields/*`, `termrock-runtime/field_control` |
| `hintbar`, `keyhint`, `progress`, `statusbar` | `termrock-feedback/*` (+ `meter`) |
| `list`, `tree`, `tabs`, `chips`, `steps` | `termrock-navigation/*` (+ `nav_list`, `filter_list`; `ChipBar`) |
| `menu`, `dialog`, `select`, `picker`, `completion` | `termrock-overlays/*` (`MenuBar`, `ContextMenu`, `help`, `picker_chain`, `select_field`). **Missing:** `hover_switch`, `popup_width`. |
| `grid` | `termrock-grid` |
| `table` | **No `Table` type**; grid is used instead |
| `code`, `diff` | `termrock-editors/*` |
| `viewport` | `termrock-viewport` |
| `scrollbar` | Internal to list/props/steps; no standalone widget |
| `segments` | **Missing** |
| `bin/showcase` (23 pages) | `showcase-ui` + `showcase-demos` (22 pages). **No Diff page.** |
| `bin/tablepro` | `tablepro-ui` (+ domain/sql/demo) |
| `bin/holla` (11 scenarios + 23 `parity-*`) | `holla-ui`/`sim`/`domain`. Only the 11 scenarios; `parity-*` is rejected with "unknown scenario". |
| `bin/jackin_preview` | `jackin-preview-app` + `host-ui` + `capsule-ui` + `sim` |

## 3. Visual and functional divergence

I ran the same 4 roots on both refs through the existing PTY suites (5 sizes × 5 colours each, 100 combos per ref): showcase overview, showcase lists, holla discovery, jackin usage overview.

**Reference** (`cargo nextest run --test visual_baseline --run-ignored only`):
- 100/100 combos are cell-, ansi-, txt- and pixel-identical to `snapshots/` (pixel similarity 1.000000).
- They still FAIL, only on the HTML gate. The HTML embeds the binary's absolute path, `/Users/donbeave/Projects/terminal-components-claude/target/debug/holla`, so any other checkout or worktree fails.
- The 18 design commits changed no file under `snapshots/` (`git diff visual-baseline design/interaction-polish -- snapshots` is empty).

**Main** (`-p termrock-conformance --test visual_baseline`): 4/4 roots fail, 100/100 combos.
- **Showcase overview** (`target/tuiscotti/actual/showcase/overview/default/120x40/truecolor.txt` vs `baselines/tuiscotti-v1/...`): 428 cells differ.
  - The focus gutter `▎` is painted on every sidebar row.
  - Sidebar labels are bold `#b3b3b3`; the reference has muted `#4d4d4d`.
  - "Diff" is missing from the nav.
  - The legend row "editing: cursor + underline" is missing.
  - An extra "Author component · click or Tab" line appears.
- **Showcase lists**: the same `▎` on every row of the nav and of each list. The current-page marker `› Lists` is missing, as are the list metadata "1–13 of 20" and "2 selected".
- **Holla**: the boot needle `holla❯` times out because `--scenario parity-discovery` does not exist on main. Even `first-use` is a different product:

  | | Reference | Main |
  |---|---|---|
  | Location | "no project" | "~/scratch/empty" |
  | Host | "mbp · local" | "devbox · local" |
  | Header | "Here" scope tabs | "discovering…" spinner |
  | Hints | Tab/Ctrl+G/F1 | F10 Menu, Ctrl+S Scope |

- **Jackin**: the needle `jackin❯` never appears. The reference chrome is "jackin❯ File Go Help · Accounts › Overview" with a bordered Accounts tree. Main renders "Jackin Preview scenario · accounts-mixed / Account & Usage Center" with a different layout and no menubar.
- **TablePro** (manual PTY peek; closest to the reference):
  - `▎` on every row.
  - "Scratch" is listed under Acme instead of Personal.
  - The Explorer pane is about 2× wider.
  - Schema icons differ (`▣`/`▾` vs `D`/`S`).
  - The query editor has an extra "SQL query" frame.
- These match main's own records:
  - `STATUS.md`: Refactor NOT_READY.
  - `main-port-analysis.md`: the parity gaps, BF01–BF14, and the note that main's app baselines are self-referential.

## 4. Visual-test and capture infrastructure

**Reference**
- `tests/visual_baseline/{main,support,showcase,holla,jackin,tablepro,pointer,audit}.rs`: 302 ignored PTY roots.
- Store: `snapshots/<app>/<group>/<leaf>/<cols>x<rows>/<color>.{ansi,txt,png,html}`, 7,550 captures.
- Driven by the `tuisnap` dev-dependency (`donbeave/tui-snap` @2d43458a).
- CI is only `holla-fable-audit*.yml`, which runs `cargo test`, against the project rule.

**Main**
- `crates/termrock-conformance/tests/visual_baseline/`: the same case files ported verbatim (same argv, needles and sends), plus `*_pending_*`, `*_journeys` and negative controls. 614 ignored tests.
- Driven by `tuiscotti` (`tailrocks/tuiscotti` @a47c9aae).
- Store: `baselines/tuiscotti-v1/`, 7,550 captures × 10 artifacts. These are the visual-baseline 4a79c0a2d frames converted, per `admission-record.json` ("legacy_replayed_conversion"), and renamed screen-first (for example `showcase/pages/overview` becomes `showcase/overview/default`) via `tests/conformance/required_cases.json`.
- `VISUAL_BASELINE_STORE=snapshots` can point the suite at the legacy layout.
- The harness already masks argv0 in the HTML, so it avoids the reference's path problem.
- CI (`.github/workflows/ci.yml`) runs per-crate fmt, clippy and nextest only. No ignored PTY captures run in CI.
- Older evidence also exists: `baseline/before/` (499 captures from d5e7075), `parity/`, `tools/parity_replay.py`.

**Can one scenario script drive both?** Mostly yes for showcase, tablepro and jackin, since the same flags and frames exist. The exceptions:
- Holla on main lacks all 23 `parity-*` scenarios, which carry about 190 argv uses in the suite.
- Showcase on main lacks `--page diff`, and the silent fallback means that case quietly renders Overview.
- Showcase on main lacks `--motion reduced`; the suite only uses it for holla and pointer cases, so this matters only for parity.
- `--help` differs.
- `NO_COLOR` handling exists on both sides.

## 5. Risks for a 1:1 port, and what main needs

1. **The oracle is not the design branch.** The frozen frames are `visual-baseline`, and the 18 design commits never re-blessed snapshots. Before anything is ported, the design branch's own suite must be re-run, and any intentional visual changes approved, or the shared suite will gate against the pre-polish look.
2. **Make the CLI identical.** Use clap with `--help`. Make unknown `--page`/`--color` values an error instead of a silent fallback. Accept `--motion reduced` in showcase, add the `diff` page, and add the 23 holla `parity-*` scenarios with the same `--frame 40` determinism. Keep page slugs and title aliases (`editabletables`, `chipsselects`, `codeeditor`, `datagrid`), and keep boot needles (`Junie Design system`, `holla❯`, `jackin❯`) on the first frame.
3. **Fix systemic library defects, not per screen:**
   - The focus gutter painted on every row: main has no single focus indicator and no `cursor_row`/`CursorStyle`.
   - Sidebar emphasis colours: bold where the reference uses the muted tone.
   - Missing list metadata ("1–13 of 20", "2 selected").
   - Plus BF01–BF14 from the reference's `docs/refactoring-plan/branch-diff-foundation.md`.
4. **Missing reference API features:** `popup::Dismiss`/`backdrop_id` semantics, `MenuBar::hover_switch`, `Select::popup_width`, the overlay flush and modal backdrop, `segments`, a `Table` widget, and terminal suspend. Main's layer `Dismiss` has different semantics and must be reconciled with the "every behaviour configurable" rules in AGENTS.md.
5. **Holla and Jackin need re-implementation, not polish.** They are a different product generation (copy, chrome, layout).
6. **Harness:** add an argv0 mask (or relative argv) on the reference side, or run the suite only from main's harness. Main's PTY suite is not in CI, so a passing nextest run proves nothing visually; a CI job running `--run-ignored only` is needed.
7. **Process risk:** `main-port-analysis.md` warns against cherry-picking from main's 831-commit lineage, and the #24 squash makes per-change provenance hard. Main's AGENTS.md also lacks the reference's visual-state rules and the visual-baseline rule; merge those into main's AGENTS.md (keeping `CLAUDE.md` as a symlink).

Logs are in the scratchpad: `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/nextest-ws.log`, `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/vb-run.log` (main captures) and `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/vb-ref-run.log` (reference captures). The actual frames lived inside the worktrees and were deleted with them.