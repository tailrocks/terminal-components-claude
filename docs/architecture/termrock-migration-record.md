# Termrock migration record — coordinator ledger

Single committed ledger for the Termrock refactor delivered through PR #38
(`parity/termrock-visual-baseline`). Progress, decisions, coverage gaps and
evidence links live here. Detailed artefacts stay in their own files; this
record links them instead of restating them.

Historical narrative and per-increment receipts stay in
[termrock-migration-matrix.md](./termrock-migration-matrix.md). Where this
record and the matrix disagree, this record states the current decision and
the matrix section is marked superseded by it.

## Delivery path

| Item | Value |
| --- | --- |
| Delivery branch | `parity/termrock-visual-baseline` (PR #38) |
| Delivery PR | https://github.com/tailrocks/terminal-components-claude/pull/38 |
| Base branch | `main` (never pushed to directly) |
| Read-only product reference | `design/interaction-polish` (PR #26) |
| Frozen visual oracle | tag `visual-baseline` |

## Fixed reference revisions

Unchanged. A change to any of these needs a documented reference decision
recorded in this file.

| Reference | Revision |
| --- | --- |
| PR #26 source | `f4867e9b5f8a08a3bb61ce80d644198a29f51984` |
| Baseline commit | `4169783ae883788186d161af09ba9c481bd84e5c` |
| Baseline tag object | `2718a6b8127a522a3d351d0b4bdb7542fc8da542` |
| Tuiscotti | `a1ea27f757d75d07b05499276cd932e9f8e067ad` |

## Starting state of this coordinator run

| Record | Value |
| --- | --- |
| PR #38 tip when work started | `3196f4ef6` (chore(xtask): register shell OWN-14 invariants, clear stale editor exceptions, fix doc-inventory fences) |
| Audit base commit used by this run's read-only auditors | `40bc297e5` (docs: record independent form chooser review) |
| Branch tip observed at coordinator resume | `5478ecfc9` (Merge PR #38 chooser fix into picker verification) |

`origin/parity/termrock-visual-baseline` advanced during this run from
concurrent writers already active on the branch. Every increment in this
record is rebased onto the then-current remote tip before it is pushed, and
every push takes the shared push lock described in `AGENTS.md`.

## Acceptance gates

Both requirements must pass. Neither substitutes for the other.

| Requirement | Gate | Status |
| --- | --- | --- |
| Exact product match | Paired reference capture, reference replay, candidate comparison over the full reference inventory of all four applications, twice, same result | pending |
| Reusable library | Termrock builds and tests with no application dependency; a separate consumer compiles against public APIs only; shared components demonstrated in real application flows | pending |

## Reference inventory (the acceptance surface)

From `baselines/tuiscotti-v1/corpus-index.json` (schema
`termrock-spec/tuiscotti-corpus-index-v1`):

| Application | Screens | Captures | Artefacts |
| --- | --- | --- | --- |
| holla | 125 | 3125 | 3125 x 10 |
| showcase | 84 | 2100 | 2100 x 10 |
| jackin | 49 | 1225 | 1225 x 10 |
| tablepro | 44 | 1100 | 1100 x 10 |
| Total | 302 | 7550 | 75500 |

Every screen is captured at 5 sizes (`72x20`, `80x24`, `100x30`, `120x40`,
`160x50`) and 5 colour modes (`truecolor`, `256`, `16`, `nocolor`, `none`).
Layout of one capture directory:
`baselines/tuiscotti-v1/<app>/<screen>/<variant>/<size>/<mode>.{ansi,ascii,frame.json,html,manifest.json,observations.json,png,txt}`.

Oracle provenance recorded in the index: `historical_oracle_tag =
visual-baseline`, `historical_oracle_commit = 4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`,
`reference_app_sha = 7bd6a331721737514a2477c894d922cb262ef07b`,
`tuiscotti_source_sha = a47c9aaefb34e4c00026f99d8a8dd7ee5916b274`.

`baselines/oracle/ORACLE.toml` is still `pending = true` with zero imported
scenarios; the frozen baseline therefore still resolves through `snapshots/`.
Closing that gap is part of the "frozen artefacts stay outside production
dependencies" decision D-003 and the oracle import path
`cargo xtask visual import --oracle | seal`.

## Machine-checked acceptance state

`python3 tools/acceptance_architecture.py` is the fail-closed architecture
gate. At revision `64afb8a1f` it reports:

| Field | Value |
| --- | --- |
| `architecture_checks_passed` | true |
| `findings` | 0 |
| `dependency_violations` | 0 |
| `unaccounted_inputs` | 0 |
| `source_inputs` | 631 |
| `application_shared_closure_missing` | 0 |
| `acceptance_complete` | false (the bounded checker never attests acceptance) |
| `unresolved` | 5 |

The five unresolved requirements are declared in
`tools/acceptance_policy.json` and are the concrete work plan for this run:

1. full real-CLI workflow inventory and exact paired comparisons
2. independent source-bound semantic review of application thinness and
   behavior ownership
3. full workspace nextest, clippy, and formatting results bound to this source
   manifest
4. isolated production-library build and nextest result
5. reviewed complete production-input coverage including generated inputs

Requirement 1 is bounded by the reference inventory above: 302 screens across
four applications, each at 5 sizes and 5 colour modes.

The paired comparison harness lives in
`crates/termrock-conformance/tests/visual_baseline/`. Its store integrity
check (`main.rs`), the ten-artefact bundle writer, the decoded-frame
comparison (`support.rs::compare_bundle_dirs`, `check_baseline_bundle`) and
the per-application journey and pending suites are the mechanism. The
`#[ignore = "… run with --ignored"]` marker on a test means the case exists
in the inventory but has not been admitted yet.

Current inventory backlog: 351 ignored tests across
`crates/termrock-conformance/tests/visual_baseline/`, of which the per-app
pending suites are:

| App | Pending files | Ignored tests | Reference screens |
| --- | --- | --- | --- |
| holla | `holla_pending_h1..h7` | 124 | 125 |
| jackin | `jackin_pending_j1..j2` | 47 | 49 |
| showcase | `showcase_pending_s1..s5` | 94 | 84 |
| tablepro | `tablepro_pending_t1..t2` | 28 | 44 |

The remaining ignores are journeys, pointer, audit and support checks.

### Workspace gate baseline

`cargo nextest run --all-targets --workspace` at the coordinator baseline
(4687 tests): **4357 passed, 330 failed, 29516 skipped**. The failure set
is dominated by one systemic cause: component render digests in
`crates/termrock-conformance/tests/baselines/components.txt` no longer match
the rendered output, because that baseline was last written at `032e9faa3`
and later theme and render changes were never re-blessed. A diagnosis stream
owns the bisect, the classification and the capture-or-revert decision.

Other named failures at the same baseline:
`termrock-xtask::ownership`, `termrock-xtask::doc_inventory`,
`termrock-overlays menu::tests::menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence`,
`termrock-navigation::item_row_columns constructor_defaults_match_the_pre_policy_full_cell_fixture`,
`termrock-editors` focus-loss cases,
`termrock-theme::capability_palettes all_73_junie_slots_match_reference_capability_output`,
`visual-harness::visual_baseline determinism::sources_are_deterministic`.

## Library gaps (read-only audit, revision `40bc297e5`)

### The composition API exists and is unused

`Group`, `AppShellSlots` and `ui.compose` appear in application code exactly
once, in `crates/showcase-demos/src/pages/editor.rs:574-635`. `termrock-shell`
(`Footer`, `StatusLine`, `Nav`, `Router`, `Page`, `QuitGuard`) has ZERO
callers in application code: only `crates/termrock-shell/examples/shell_consumer.rs`
and its own tests. Every application re-implements shell chrome, routing and
quit handling.

Raw geometry counts in application crates (`Rect::new` / `Rect {`):
showcase-demos 675, tablepro-ui 291, holla-ui 214, jackin-preview-app 189,
showcase-ui 36. Direct painting counts (`ui.paint_str` / `ui.fill` / cell
writes): showcase-demos 414, tablepro-ui 265, holla-ui 216,
jackin-preview-app 107, showcase-ui 32.

### Highest-impact library gaps

| # | Gap | Unblocks | Crate |
| --- | --- | --- | --- |
| G1 | Composition adapters for the remaining ~50 components (only 11 of ~60 can enter a `Group`) | every screen of every app | `termrock` |
| G2 | Closure-based named slot builder for the shell and blocks | every `AppShell::regions` call site | `termrock-shell` |
| G3 | Library-owned scroll area / scrollbar block | 12 hand-painted scrollbars | `termrock-runtime` |
| G4 | `Panel` owns its body/title rect and title painting | 5 app re-implementations of panel chrome | `termrock-controls` |
| G5 | Content-rect queries on `Tree`, `List`, `NavList`, `Table` | every app that subtracts a chrome band by hand | `termrock-navigation`, `termrock-grid` |
| G6 | Adopt `Footer`, `StatusLine`, `Nav`, `Router`, `Page`, `QuitGuard` | 4 apps' hand-rolled shell | `termrock-shell` |
| G7 | Library returns resolved grid column placement | tablepro `ColBudget`/`ColPlace`/`GridChrome` | `termrock-grid` |
| G8 | Semantic theme tokens for app chrome | 531 hardcoded `Color::Rgb` in app crates | `termrock-theme` |
| G9 | Runtime-owned predicate recolor; fix the ANSI-256 ladder | 6 app `ui.raw()` correction painters | `termrock-runtime`, `termrock-theme` |
| G10 | Bind the declared deny lints to every `termrock-*` crate | the no-panic contract; 941 `expect`, 201 `unwrap`, 64 `panic!` in library `src/` | workspace root + all libraries |
| G11 | `termrock-session` owns the cursor-shape escape | 2 apps writing `\x1b[6 q` to stdout | `termrock-session` |
| G12 | `inactive-layer` and `invalid` as first-class flags, plus a full combination matrix | consistent chosen ink under modals | `termrock-core`, `termrock-theme` |

### API-contract scorecard

| Contract element | Status |
| --- | --- |
| `Picker::new(id).title().placeholder().filter(FilterPolicy::Label)`, `update`, `PickerAction::Chosen` | PRESENT |
| Composition API above the runtime reusing `update`/`measure`/`draw`; no input during measure or draw | PRESENT but narrow: 11 of ~60 components are adaptable; `Measure` takes `&Ui` and `draw` takes `&mut Ui`, so no `Cx` is reachable during either, and dispatch is structurally impossible |
| One resolved layout for render, clip, hit regions, focus bounds, cursor | PARTIAL: composition resolves once, but app-composed regions have no resolved layout, so hit regions come from hand-computed rects |
| Named slots, style variants, semantic tokens, one owner per state | PARTIAL: violated by apps (correction painters, parallel grid state) |
| Full state set hover/pressed/focus-visible/selected/checked/editing/disabled/read-only/busy/invalid/inactive-layer | PARTIAL: `invalid` exists only as `ERROR`; `inactive-layer` is not a flag, it is `Ui::is_inert` plus `Theme::inactive_look`; no combination matrix for `READ_ONLY`, `BUSY`, `EDITING`, `ERROR` |
| Overlays own input scope, cursor, focus restoration; grapheme-safe text; no panics | PARTIAL: overlay scope and focus restore are implemented and tested; grapheme safety is implemented and tested; panic-freedom is declared but NOT enforced, because no `termrock-*` crate opts into the workspace deny lints |
| Raw geometry stays inside components | FAIL in applications |

### Dependency direction

No production `termrock-*` crate depends on an application crate. The only
inverted edges are application DATA crates depending on widget types:
`showcase-data` -> `termrock` (a UI tree type in a data crate),
`tablepro-domain` -> `termrock` (`GridState`, `GridModel`, `ItemKey`,
`CompletingCodeEditorState`, `FieldError`), `jackin-preview-domain` ->
`termrock` (`Secret`), `jackin-preview-sim` -> `termrock` (`Rect`, `Role`,
`SplitModel`). `crates/holla-domain` declares `termrock-core` and `ratatui`
but uses neither; that edge is stale.

`tools/acceptance_policy.json` lists 19 production libraries and omits
`termrock-shell`, so the lexical architecture gate does not see it.

### Migration-matrix overstatements

- The matrix claims the existing `MasterDetail` policy expresses the frozen
  34% / 34-48-cell / gap-two / 108-column constraints. It does not:
  `crates/termrock-controls/src/master_detail.rs:149-153` ships
  `master_share: 32`, `min_master: 28`, `min_detail: 40`, `gap: 2`,
  `collapse_below: 100`, with no max clamp, so a 34-48 band is inexpressible.
  No application crate references `MasterDetail` at all.
- The matrix lists the breadcrumb trail as ported. Only the MODEL is in
  `crates/termrock-shell/src/nav.rs`; that file states the widget "is owned
  by another crate wave". The responsive, clickable trail does not exist.
- The matrix lists the footer extraction as complete. The same crate states
  that the status variant and the reveal/peek integration are not ported.
- The matrix claims the showcase has catalog pages for tabs, menus,
  popups/tooltips, meters, notices, status hints, master-detail, pane tree,
  splits/stacks, toolbars, breadcrumbs/rails and help keys.
  `crates/showcase-ui/src/app.rs:53-76` exposes 22 `PageId` variants and has
  none of them. They cannot exist until the corresponding widgets land.
- The matrix claims the menu command map and `Chrome::fit` dialog inset stay
  with their owners. `crates/termrock-overlays/src/menu.rs` exposes no menu
  command map and `crates/termrock-shell/src/chrome.rs:5-8` says the same.
  The chrome-owned dialog inset has no owner anywhere in PR #38.

## PR #26 extraction priorities (read-only audit)

PR #26 (`f4867e9b5`) capabilities still missing from PR #38, ranked by how
many application bypasses each unblocks, with the parity risk of landing it:

1. Button/action row overflow folding — LOW risk, purely additive.
2. Toolbar with overflow — LOW risk.
3. Master/detail width policy (proportional share, min and max clamp, natural
   width query) — LOW to MED; keep the shipped defaults.
4. Collapsible sidebar rail — LOW to MED.
5. Breadcrumb trail widget — LOW; the shell already owns the model.
6. Pane tree — LOW; nothing renders it today.
7. Segmented control and icon button — LOW.
8. Hint bar compact mode and reserved status share — MED risk, affects every
   app footer row; must land opt-in with the current behaviour as default.

Then: reveal (MED to HIGH parity risk), file browser widget, edit-menu
owner, diff narrow fallback, context-menu typeahead and overflow, key
reference sheet, horizontal scrollbar thumb states, column sizing with a
draggable seam.

Not to be ported: the PR #26 application implementations, `src/core/**` and
`src/shell/**` (already extracted), `src/theme.rs` (the PR #38 palette owns
the frozen colour contract), `Chrome::fit` (it would move every dialog
inside the frozen frames), the PR #26 `Splitter`, `Grid` and `Table`
widgets wholesale, and the PR #26 specimen and coverage harness.

### Measured parity state, candidate against the frozen approval

Candidate built from `fcc748646` (before this run's stream merges),
reference from the sealed cohort, comparison through the frozen harness.

| Application | Cases | Passed | Failed | Failure classes |
| --- | --- | --- | --- | --- |
| tablepro | 2762 | 48 | 2714 | 1735 frame mismatch against the approved snapshot, 627 interaction timeout, 332 boot needle never appeared, 20 other |
| showcase | 14549 | measurement in progress | | |
| jackin | 5818 | measurement pending | | |
| holla | 5766 | measurement pending | | |

The three failure classes map to `tests/visual_baseline/support.rs:322`
("baseline `{name}` failed", a decoded-cell difference against the approved
snapshot), `support.rs:700` ("`wait:{needle}` timed out") and
`support.rs:667` ("boot needle ... never appeared").

### Stream results

**Showcase — landed.** `crates/showcase-demos/src/pages/{chrome,taskrunner,terminal}.rs`
and `pages/mod.rs`, plus the new guard test
`crates/showcase-demos/tests/no_false_parity.rs`. 6914 lines deleted, 1309
added.

Removed: 11 captured cell tables (about 2400 cells) copied from the approved
snapshot or the tuiscotti oracle; `taskrunner::paint_historical`; 11 exact
page-rect size branches that each selected a different screen; the
`UpdateCause::Tick` "settled" switch in the terminal page; the
`historical_palette` style map together with the narrow-rail replay it
served. Each page now has one responsive layout policy.

The chrome page now matches the frozen approval cell for cell at every
reference size:

| size | cells differing before | after |
| --- | --- | --- |
| 72x20 | 56 | 0 |
| 80x24 | 243 | 0 |
| 100x30 | 758 | 0 |
| 120x40 | 0 | 0 |
| 160x50 | 2212 | 0 |

The task runner and the terminal page no longer replay anything, but they
still differ from their approved captures, because those captures show an
older page design (the old "Targets/Pipeline/Log" sheet and the old launch
transcript) while the pages now use the newer widget design. The two
captures also disagree with each other. This part is NOT complete and is
reported as a possible reference defect for an independent decision.

Gate: `cargo nextest run -p showcase-demos -p showcase-ui --all-targets` gives
135 run, 129 passed, 6 failed. The 6 failures are identical on the base
commit: `editor_shell_completion_accepts_tab_then_next_tab_indents`,
`editor_shell_completion_accepts_enter_then_tab_indents`,
`editor_playground_keeps_live_find_completion_and_run_contracts`,
`page_headings::all_page_headings_match_the_reference_capture_matrix`,
`page_headings::all_shell_headers_match_reference_captures_and_visibility_thresholds`,
`visual::showcase_visual_baseline`.

**Theme and overlays — landed.** `849d26dad` restores the frozen SQL syntax
ink (keywords bold white, operators mid grey, strings and numbers light grey,
errors in the theme red) and the public MENU/DEFAULT/TITLE identity of the
menu bar title, both of which had drifted from the frozen corpus. The theme
and overlay suites are green: 366 tests run, 366 passed.

## Coverage gaps

### False-parity inventory (read-only audit, revision `40bc297e5`)

No production code replays a captured ANSI frame and no production code
loads a snapshot at runtime. The replay is source-level: literal cell tables
copied from the approved snapshot, and screen-selection guards keyed on
exact size, fixture label, captured clock or theme identity.

Ranked by remaining false-parity surface:

| Rank | Application | Surface |
| --- | --- | --- |
| 1 | jackin-preview | ~20 exact-size/paused/accent-identity screen gates, 4 raw absolute-coordinate screen painters, 6 hard-coded data modules, 2 correction painters, 3 harness-needle hacks, 1 baseline colour index table |
| 2 | holla | 17 captured-clock (`now_ms() == 80`) tag frames, 14 hard-coded row builders, 13 width-band plate predicates, 18 hard-coded status painters, 2 exact-size plates that replace whole screens |
| 3 | showcase | 11 literal cell tables (~2200 cells) copied from the approved snapshot or oracle, 11 exact page-rect size branches selecting 4 different screens per page, a `Tick`-timed settled switch |
| 4 | tablepro | one exact-size focus branch; no cell tables, no clock gates, no snapshot replay |

Root causes, in order of leverage:

- **Captured clock.** `holla-sim` selects screens when `w.now_ms() == 80`,
  the harness tick. Every holla plate and tag frame hangs off it.
- **Theme identity as a selector.** Every jackin guard asserts
  `theme.color.accent == Color::Rgb(0x48, 0xe0, 0x54)` in addition to the
  size. A colour level is a capability; a specific RGB value is an identity
  of one frozen capture, and it must never select a screen.
- **Size as a selector.** Exact `120x40` / `100x30` / `80x24` comparisons
  select a screen instead of a layout policy. The policy must be a width
  band or an explicit breakpoint, owned by the layout component.
- **Harness needles.** `draw_accounts_boot_needle*` paints `jackin❯` at
  (0,0) only so a test can find it. The brand chip must come from the
  theme and the real header.
- **Frozen geometry in the library.**
  `crates/termrock-feedback/src/manager_split.rs` paints the brand chip and
  a card at `Rect::new(40, y+2, w-41, 36)` and buttons at row 36 — the
  120x40 frame baked into a shared component.

What stays: live catalogue data built from the world, legitimate width-band
breakpoints that still paint real widgets, real terminal-content viewers
with live scrollback, the demo table data, and the test-only harness code
under `crates/termrock-e2e`, `crates/termrock-conformance` and
`crates/visual-harness`.

## Streams

One writer per working copy. Application writers own only their own
application crates; the library writer owns only `crates/termrock-*/**`. A
library gap an application writer hits is recorded, not patched locally.

| Stream | Owns | Branch |
| --- | --- | --- |
| Jackin | `crates/jackin-preview-app/**` | `wip/jackin-false-parity-20261021` |
| Holla | `crates/holla-ui/**`, `crates/holla-sim/**` | `wip/holla-false-parity-20261021` |
| Showcase | `crates/showcase-ui/**`, `crates/showcase-demos/**`, `crates/showcase/**` | `wip/showcase-false-parity-20261021` |
| TablePro | `crates/tablepro-ui/**`, `crates/tablepro-demo/**` | `wip/tablepro-false-parity-20261021` |
| Termrock | `crates/termrock-*/**` | `wip/termrock-library-gaps-20261021` |

The coordinator merges each stream into `parity/termrock-visual-baseline`
under the shared push lock, and owns the final gates.

## Verification infrastructure (audit, revision `40bc297e5`)

The live paired runner is `tools/frozen_baseline.py live-compare`. The
legacy `tools/parity_replay.py` / `tools/parity_approve.py` path (499
recipes) is not the acceptance runner.

Paired run, in order:

```bash
python3 tools/frozen_baseline.py prepare  --repo . --root target/frozen-baseline
python3 tools/frozen_baseline.py build    --repo . --root target/frozen-baseline
python3 tools/frozen_baseline.py build-candidate --repo . --root target/frozen-baseline \
    --target-dir /abs/path/to/candidate-target
python3 tools/frozen_baseline.py prepare-live --repo . --root target/frozen-baseline \
    --fresh-cohort frozen-20261021-serial-nonincr-r1-r2 --baseline-bin-dir <sealed cohort bin>
python3 tools/frozen_baseline.py live-compare --repo . --root target/frozen-baseline \
    --candidate-bin-dir <candidate-target>/debug --candidate-repo <candidate checkout> \
    --case 'showcase::pages::overview::120x40::truecolor' | --all
```

Phases: `baseline-capture` (frozen binary captures frames into a sealed
expected store), `baseline-check` (frozen binary replays against the same
store — determinism proof), `candidate-check` (candidate binary replays
against the same store). Comparison is decoded cells, not ANSI bytes: each
checkpoint carries an ANSI sha256 and a semantic `Frame::to_json()` digest
covering every ordered cell plus cursor position, visibility, style and
blink. SGR modifiers are compared cell by cell. A candidate that renders the
same cells with a different cursor style still fails.

`live-compare --all` loops every scenario in one command, which breaks the
bounded-command rule. The resumable protocol is the guard:
`fresh-plan` -> `verify-fresh-plan` -> `fresh-run-init` -> `fresh-run-verify`
-> `fresh-run-advance`, each bound to an expected plan sha256.

Runner guards: `python3 tools/test_frozen_baseline.py` (119 tests),
`python3 tools/test_acceptance_architecture.py` (39 tests),
`python3 tools/test_inventory/test_inventory.py` (13 tests). All green at
`40bc297e5`.

### Reference inventory is fragmented

Nine parallel inventories exist and no cross-check asserts they agree:

| Inventory | Count | Scope |
| --- | --- | --- |
| frozen CLI scenarios in the frozen source (`EXPECTED_SCENARIOS`) | 28 895 | the only one the paired runner enforces, and only via `--all` |
| `tools/frozen_cli_extensions.json` | 174 | registered extension cases |
| `baselines/tuiscotti-v1/` | 7 550 captures = 302 screens x 25 | the stored corpus |
| `snapshots/` | 7 675 `.ansi` | visual-harness store |
| `tests/conformance/required_cases.json` | 524 | 222-case `W*` vocabulary |
| `crates/termrock-conformance/tests/visual_baseline/` | 473 cases + 41 variants | the runnable suite |
| `parity/recipes.tsv` | 499 | legacy path; zero holla rows |
| `crates/termrock-e2e/cases/registry.json` | 4 | one per app |
| `tools/app-inventory.json` | app-level | themes, startup args, perf targets |

The runnable conformance suite pins most scenarios at 120x40 truecolor only,
so "5 sizes x 5 colours" coverage exists in the corpus and the extension
manifest but not in the suite that runs.

### Blockers to clear before acceptance

0. **CI blocker, fixed.** The generated `Plan` job failed at "Check generated
   files" because velnor's crate discovery scanned
   `docs/audits/evidence/2026-10-21-picker-contrast-integration/independent-review/consumer/Cargo.toml`,
   a standalone consumer crate whose path dependencies point at an absolute
   `/private/tmp/...` sparse copy. The detection error
   `malformed_manifest:...:unresolved_path_edge` aborted generation, so every
   downstream Rust job was skipped and `required` failed closed: hosted CI
   provided zero validation for PR #38.
   Fix: `[discovery] exclude` in `.velnor/config.toml` now also lists
   `docs/audits/evidence/**`. Evidence trees are not crates. Verified
   locally with velnor-actions 0.1.32: generation succeeds and
   `diff -r --brief .github <generated>/.github` is empty, so the committed
   generated files are already correct and needed no regeneration.
   Commit `76a1a1094`.

1. `baselines/oracle/ORACLE.toml` is `pending = true`, version 0, zero
   scenarios. The corpus was acquired by `legacy_replayed_conversion`, not by
   live capture. The official oracle store is empty.
2. `frozen-baseline corpus-check` is red in both modes: the corpus ANSI and
   `frame.json` disagree on SGR modifiers, and the corpus disagrees with the
   snapshot store. About 78 tier cases.
3. The capture-eligible binary cohort is not admitted as a reference
   (`tools/frozen_cli_build_cohorts.json` says
   `reproduced-diagnostic-not-reference-approved`). The four app binaries
   are not byte-reproducible across build roots, so no candidate comparison
   has a legal baseline yet.
4. No full paired comparison has ever been green. Latest recorded attempt:
   240 scenarios red.
5. CI runs none of the real gates. The generated `Plan` job's "Check
   generated files" step is red, so every downstream Rust job is skipped and
   `required` fails closed. The fix belongs in `.velnor/config.toml` plus a
   regeneration, never in the generated workflow files.
6. `crates/termrock-e2e`, `tools/frozen-cli-frame-support`,
   `tools/visibility`, `tools/termrock-visibility-publisher`,
   `crates/holla-catalog`, `crates/holla-plan` and `crates/showcase-data`
   are excluded from velnor discovery and get no CI job at all.
7. `cargo run -p termrock-xtask -- check-ownership` is red with about 100
   findings owned by other streams.
8. `tools/test_perf_workflow.py` cannot run: `.github/workflows/perf.yml`
   does not exist in the generated workflow set.

## Decisions

See `## Decisions log`.

## Evidence

See `## Evidence index`.

## Decisions log

### D-001 — Delivery stays on PR #38

All increments land on `parity/termrock-visual-baseline` through the shared
push lock. `main` is never pushed to directly; `design/interaction-polish`
(PR #26) and the `visual-baseline` tag stay unchanged.

### D-002 — False parity is removed, not renamed

Application paths that reproduce a reference screen by replaying captured
output, by branching on exact test sizes, fixture labels or captured clock
values, or by repainting a shared widget, are deleted and replaced by real
component composition. They are not moved into Termrock and not renamed.

### D-003 — Frozen artefacts are test-only

Reference frames stay outside production dependencies. A production build
must not be able to consume reference frames through source, generated code,
resources or build scripts.

### D-004 — A screen is never selected by identity

A screen may be selected by logical application state and by a layout
policy. It may never be selected by an exact terminal size, a fixture
label, a captured clock value, or a specific theme colour value. A colour
level is a capability; an exact RGB is the identity of one frozen capture.

### D-005 — Reference defects are recorded, not papered over

When a frozen frame and the live output genuinely conflict, both results
are recorded, the case is marked as a reference defect, and the requirement
that depends on it stays open until an independent review clears the
conflict. The 15 holla no-reference parity families are the known instance:
they fail the frozen oracle on both the reference and the candidate build.

### D-006 — Missing library capabilities are reported, not forked

An application stream that needs a Termrock capability records it with a
proposed public API and the call site. The library stream implements it
once. No application may fork a widget or hand-paint cells to work around a
missing component.

## Evidence index

Populated as increments land.
