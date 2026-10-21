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

### D-007 — Relocating a bypass is not removing it

Commits on the delivery branch that move an approval predicate from the
application dispatch into a view module, for example "jackin: gate the
accounts frames in their view modules" (`8a9b28807`) and "holla: fold
remaining size-keyed plate gates into frame predicates" (`8689fe270`),
relocate the size, fixture-label and theme-identity guards. They do not
remove them. The task requires the guard to be gone and the live chrome to
produce the frame from real state, so a relocated guard is still open work
and must not be counted as done.

### D-008 — Expected digests are a product signal, not an output to bless

`crates/termrock-conformance/tests/baselines/components.txt` is treated as
an acceptance signal. The migration matrix records the standing decision
that "the product differences remain work to fix, not output to bless", and
that no pinned frozen capture or documented independent reference decision
authorises replacing an expected digest. So the 231 red component digests
are a product regression to be fixed in the library, not a baseline to
re-bless. No stream may set `BLESS=1`, edit that file, or widen, skip or
weaken an assertion.

**Jackin — partially landed.** `fc2c3bd4e` replaces the four hard-coded
accounts view modules with a real `accounts_page.rs` driven by the world
state, and takes the 256-colour downgrade from the shared theme ladder
instead of an app-local table. 188 tests run, 187 passed, 1 failed
(`visual::jackin_visual_baseline`), which was already failing before the
merge.
The stream also re-blessed 8 app-local digests in
`crates/jackin-preview-app/tests/baselines/jackin.txt` for the
`accounts-mixed` scenario. That file is a candidate-side regression pin, not
the frozen oracle, but the re-blessing still needs independent evidence
that the new rendering matches the frozen reference. The test-integrity
review owns that check.
The remaining Jackin bypasses are open: the help overlay, the hard-launch
chrome, the narrow hard-cases manager, the capsule split and zoom views,
the settings and prelude pages, the editor tab strip, the detail drawer
correction painter, the harness boot needles and the `historical_*` name
space.

### The theme ramp drift, decided against the frozen rendered reference

The component-digest failure was bisected by the session-consolidation
stream to `a848219fd` ("fix: match tablepro quick-open and tab list to the
baseline"), which changed the Junie foreground ramp from
`0x808080 / 0x4d4d4d` to `0x8a8a8a / 0x737373` and the scrollbar track
glyph from `│` to `┆`.

That change is contradicted by the frozen rendered reference, measured
directly from the approved corpus:

| Evidence | Result |
| --- | --- |
| `0x8a8a8a` (ANSI-256 index 245) in the approved corpus at 120x40 | 0 occurrences |
| `0x808080` (index 244) in the approved corpus at 120x40 | 254 268 occurrences |
| `0x737373` (index 242) in the approved corpus at 120x40 | 0 occurrences |
| `0x4d4d4d` (index 238) in the approved corpus at 120x40 | 60 552 occurrences |
| scrollbar track `│` in the approved corpus at 120x40, every colour mode | 12 600 occurrences |
| scrollbar track `┆` in the approved corpus at 120x40, every colour mode | 6 occurrences |

The adjudication recorded on the session-consolidation branch states that
"the reference values are 245, 242", but it also records that those values
came from the tag's token TABLE and that the reviewer's own corpus check
found no such rendering. The rendered reference is the oracle, so the ramp
and the track glyph are restored to `0x808080 / 0x4d4d4d` and `│`, and the
two expectations that pinned the drifted values are corrected with the
measurement above.

Effect: the component render gate went from 89 passed / 231 failed to
178 passed / 142 failed. The remaining 142 failures are 99 truecolor and 43
mono, spread over 15 component groups, so a second drift is still open and
the bisect continues.

### D-009 — The oracle is the frozen source and its approved frames

`baselines/tuiscotti-v1/` is NOT the oracle. Its own `corpus-index.json`
records `acquisition_method: legacy_replayed_conversion` and a corpus
commit of `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, and it disagrees with
both the frozen source and the approved frames. The oracle is the frozen
source at `4169783ae` and the frames it produced, which are in the frozen
source's own snapshot store after `prepare`.

A coordinator decision (`b4b1a3346`) used the converted corpus to "restore"
a muted and faint ramp of 0x808080 and 0x4d4d4d. The frozen source says
`text_muted` is `MUTED` (0x8a8a8a) and `text_faint` is `WHITE_43`
(0x737373), and the approved frames paint those values. `493cc57d6` restores
the frozen source values and reverts that decision. The component render
gate returns to 89 passed / 231 failed, and the reference decision request
in `docs/audits/2026-10-22-component-digest-reference-decision.md` states
why the gate stays red until an independent decision lands.

### D-010 — The component digests were never stale; the palette was

The expected digests in `crates/termrock-conformance/tests/baselines/components.txt`
are byte-identical to `032e9faa3` and were never updated. The 231 failing
component digests were caused by the palette drifting away from the frozen
source. With the theme aligned to the frozen source (`ec783f6c5`) the
component render gate reports **320 tests run: 318 passed, 2 failed**, and
no expected value was updated, no baseline was re-blessed and `BLESS=1` was
never used. `docs/audits/2026-10-22-component-digest-reference-decision.md`
records the evidence and the two remaining failures.

### Independent review: UI/UX (verdict HELD)

Reviewer: an independent read-only reviewer on
`wip/review-ui-ux-20261021`, tip `3226b9b40`, report at
`docs/audits/evidence/2026-10-22-ui-ux-review/report.md`. Verdict **HELD**
on `ef35a7f6b`, `493f9054f`, `849d26dad`, `150320ab3`, `fc2c3bd4e`.

Blockers it raised, all open:

1. The live showcase terminal page makes the stored
   `showcase::pages::terminal` frame unreachable: the launch advances one
   stage per update, the settled frame needs about 36 updates, and the
   page has no `seek_paused`, so the harness times out after 60.6 s where
   the base captured the same frame in 6.0 s.
2. The live jackin accounts page breaks three stored frames that the base
   matched exactly: `jackin::accounts::detail::120x40::truecolor`,
   `jackin::accounts::drawer::120x40::truecolor` and
   `jackin::accounts::filter::120x40::truecolor` (946, 936 and 201
   differing cells). The same seven scenarios report 3 passed / 4 failed on
   the base and 0 passed / 7 failed on the reviewed tip.
3. The live showcase task runner regresses the stored frame at every size:
   120x40 goes from 23 differing cells to 1125, 160x50 from 23 to 1605,
   72x20 from 18 to 333. The new page drops the panel rule and paints the
   "Targets" title bold white where the approved frame has it secondary.

Its two major theme findings are the ones this record now acts on: the
syntax tokens must stay blue, and the 256 ladder must keep the corpus
index.

Majors and minors it raised, all open: the new hint bar advertises keys
that do not perform the advertised action on the jackin accounts route; the
shared 256 ladder changes four jackin card inks; the chrome rule ink is
0x262626 against 0x3a3a3a in the approved frame; the accounts hit region
ignores the list scroll offset; the filtered pane title truncates one
character later; the tree always appends the "Unsupported" and
"+ Add account…" rows; the drawer blanks the header the approved frame
keeps; a committed filter leaves the list without key handling; the inline
filter replaced the reference dialog's dismissal contract; the showcase
guard greps three literal strings; the showcase page paints its own
scrollbar with hard-coded inks and no states.

Measured fidelity table and the command list are in the review report.

### Workspace gate, final measurement this round

`cargo nextest run --all-targets --workspace --no-fail-fast` on the
coordinator source: **4768 tests run: 4740 passed, 28 failed, 29516
skipped**. The session started at 4687 run, 4357 passed, 330 failed, so
302 tests moved from red to green.

The 28 remaining failures, by crate:

| Crate | Failures |
| --- | --- |
| showcase-ui | 6 (`editor_playground_keeps_live_find_completion_and_run_contracts`, two `editor_shell_completion_*`, two `page_headings::*`, `showcase_visual_baseline`) |
| tablepro-ui | 4 (`query_editor_keeps_multiline_unicode_and_cursor_through_resize`, two `perf::frame_*`, `tablepro_visual_baseline`) |
| holla-ui / holla | 3 (`menu_bar_pointer_hover_changes_only_the_file_title_and_clears_on_exit`, `holla_visual_baseline`, `holla_pty::a_bounded_input_flood_is_drained_fairly_and_a_quit_behind_it_is_honoured`) |
| jackin-preview-app | 1 (`jackin_visual_baseline`) |
| termrock-controls | 3 (button focus-visibility policies) |
| termrock-overlays | 3 (context menu danger profile ×2, picker query field) |
| termrock-conformance | 2 (`render::components::picker_chain::pressed`, `::disabled`) |
| termrock-theme | 1 (`empty_style_inheritance::standalone_empty_cells_match_pre_inheritance_reference`) |
| termrock-navigation | 1 (`item_row_columns::constructor_defaults_match_the_pre_policy_full_cell_fixture`) |
| termrock-feedback | 1 (`meter_semantic_tokens::low_and_stale_tokens_match_reference_roles_but_custom_colors_remain_authoritative`) |
| termrock-xtask | 1 (`ownership_check_passes_on_workspace`) |
| visual-harness | 2 (`determinism::sources_are_deterministic`, `overlay_sites`) |

The four `*_visual_baseline` failures are the per-application visual
baselines, and they are the ones the reference comparison measures at
application level.

### Independent reviews: runtime correctness, API boundaries, test integrity

Three more independent reviewers worked in separate read-only worktrees
against `bea5350ec9`:

| Reviewer | Branch | Verdict |
| --- | --- | --- |
| Runtime correctness | `wip/review-runtime-correctness-20261022` | HELD, 33 findings (5 blockers, 14 majors, 12 minors, 2 nits) |
| API boundaries | `wip/review-api-boundaries-20261022` | see the review report |
| Test integrity | `wip/review-test-integrity-20261022` | HELD, 4 blockers and 4 majors |

The runtime review's blockers are all in the jackin accounts page and the
holla history default, and are being repaired:

1. the accounts tree hit regions are off by one row, because the list owner
   is drawn at `area.y + 2` while the painter starts at `area.y + 3`;
2. the narrow accounts list never scrolls, so the painter ignores the scroll
   offset and the cursor at index 20 still shows rows 1-11;
3. the range indicator is unclamped and reads "16-26 of 23";
4. after a filter query is committed the accounts list is key-dead, because
   `filtering` stays true and `update_accounts` returns early;
5. the accounts page regresses three oracle-approved frames.

The theme change in `ec783f6c5` was itself raised as a blocker by the
runtime reviewer, on the strength of the converted corpus. The converted
corpus is not the oracle: its own index records
`acquisition_method: legacy_replayed_conversion` and it disagrees with the
frozen source and the approved frames. An independent adjudication stream
is running to settle the conflict with both results preserved, and this
record states the conflict rather than resolving it unilaterally.

The test-integrity review's blockers:

1. the 8 re-blessed `accounts-mixed` digests in
   `crates/jackin-preview-app/tests/baselines/jackin.txt` have no
   frozen-oracle counterpart;
2. the new accounts page regresses the oracle-approved
   `jackin::accounts::detail/120x40/truecolor` frame, which the old
   application rendered digest-identical to the oracle;
3. the holla harness moved to `for_scenario_without_history` without the
   digest baseline being re-blessed;
4. the holla width-band change made the architecture gate report
   `architecture_checks_passed: false`.

Findings 3 and 4 are fixed in this round: the history default is restored to
the reference behaviour, off unless the environment asks for it, and the two
holla comments that tripped the gate's frame-marker rule are reworded on the
delivery tip. The gate now reports 0 findings and 0 dependency violations.

The test-integrity review also ran the detection experiments: the new
`accounts_page.rs` test fails on the pre-increment parent (3 of 4 fail) and
the new `no_false_parity.rs` test fails on its parent (3 of 7 fail), so the
new tests do detect the old behaviour.

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

### D-011 — Restore the pre-37f55475 component expectations pending adjudication

The 1,060 expectation replacements introduced by `37f55475f` are
superseded pending an independent reference review that adjudicates those
exact keys. Restore `crates/termrock-conformance/tests/baselines/components.txt`
to pre-37f55475 blob `3343d127cfe998069e28037b9abaa95a77f466a4`
(SHA256 `942219d9ad335ac892ad9658a85a57972fa6cf18afa33220af3b0705cdee474f`)
without reverting the product or test changes from that commit.

The restored values are preserved expectations, not a semantic approval of
each old digest. The candidate values remain preserved as unapproved
observations. The independent test-integrity review retains both blobs and
the exact changed-key census at
`/Users/donbeave/Projects/termrock-review-20261021-1536-9plajc9k/source-refresh-census-postfsck-20261022/source-quality-reviews-v10/reviewer-4/A021/test-integrity-e5077810/`;
A021 evidence preserves both blobs; a separate provenance review independently
confirmed the pre-change blob identity. Neither review approves candidate
values or adjudicates the reference for the changed keys.
Neither blob is authorized for further change until the separate review
adjudicates the exact affected keys. Red comparisons against the restored
expectations remain open product/reference conflicts; they are not grounds
to regenerate, bless, skip or weaken expectations.

The independent integrity review also records the finding that
`23790b23f` removes CodeEditor `EDITING` binding-state coverage and adds broad
legacy chord exemptions. That finding remains open for a separate focused
component-contract review; this expectation repair does not change its
conformance implementation or authorize those exceptions.

### D-012 — Consolidation evidence and remaining holds (2026-10-22)

This checkpoint records the current bounded evidence and open holds. It does not select or approve a source packet, transfer source changes, or establish application or library acceptance.

#### Published-tip verification

The independent verification is bound to published source commit `4d7a358d3ced4d5260ced349d26974aae6a9dd12` (tree `320ae4cf28ed282a7ef886d44c211f440fdc3442`). The public parity branch has since advanced to `bb4bbb6eb9c7d4f546283d5db2f978964a335f9b`; these results do not validate that later tip.

At 4d7, the bounded conformance command passed 940/940 in 210.098 seconds under a 215-second limit. This target does not load component digests. The directly affected brand-focused render check failed 1 test with 331 filtered: expected digest `9d2d64860f2f8023`, actual `754d865a10dc13ed`. The baseline file remained unchanged. Full workspace nextest, Clippy, formatting, and visual-baseline checks remain unverified. See the independent receipt at `/Users/donbeave/Projects/termrock-review-20261021-1536-9plajc9k/restored-expectations-independent-gate-receipt-2.md` (SHA-256 `3782326322e58ce0b493056c9481f9ff98cb4acef92079841e7fe607c9f4685a`) for commands, run IDs, logs, and hashes.

#### Export and source-review state

The export ledger currently accounts for 75 of 180 jobs across the five partitions: p0 1, p1 13, p2 13, p3 36, and p4 12. These are export counts only; they are not review or acceptance counts.

- **p0 / A006:** 610 source-diff chunks completed, covering 39,040 of 77,466 paths. The next chunk failed during process cleanup and remains held with its partial evidence; it is not a completed export.
- **p4 / A068:** 256 chunks completed, covering 16,384 of 141,796 paths; 125,412 paths remain.
- **p1 / A070:** the 710,474-path scoped union exceeds the current in-memory scope limits; its source status and source-tree inventories also exceed the generic metadata cap. Completion requires a disk-backed streaming exporter change; the proposed v8 work is not approved.

Completed artifacts and preserved export attempts remain under the pinned capture directory at `/Users/donbeave/Projects/termrock-review-20261021-1536-9plajc9k/source-refresh-census-postfsck-20261022/source-exports-v10-lossless-v6/`. Input identities and completed artifact hashes remain immutable. Do not treat partial reports or chunks as a complete packet.

Source review remains incomplete. A009's primary review is a draft with unresolved semantic review of large archive bodies and other listed paths; no packet is selected or approved. Its report is `/Users/donbeave/Projects/termrock-review-20261021-1536-9plajc9k/source-refresh-census-postfsck-20261022/source-quality-reviews-v10/reviewer-4/A009/primary-review.md` (SHA-256 `96fd61de318cf368c63d01fa9a9505baa8a4e7b594528f32ea4f95e19eca0633`).

A021's four-line Ready-to-Busy scheduling candidate remains held. An independently repeated real-Runtime probe reproduces the missing Busy deadline on Enter, Space and pointer activation; the candidate passes those four focused tests. This establishes bounded Busy scheduling behavior only. Fixed-reference source inspection shows application-wide idle Tick intervals of 200/400 ms, while the candidate unconditionally requests a Tick after about 80 ms even for Ready activation. Guard tests are still checking that timing and domain-effect risk. No full-frame comparison or consumer journey establishes parity, and no packet is selected. See the A021 primary review at `/Users/donbeave/Projects/termrock-review-20261021-1536-9plajc9k/source-refresh-census-postfsck-20261022/source-quality-reviews-v10/reviewer-4/A021/primary-review.md` and the independent probe review at `/Users/donbeave/Projects/termrock-review-20261021-1536-9plajc9k/button-ready-busy-independent-review/REVIEW.md` (SHA-256 `e943bccd41e0c4c2f3e835c4c43bb1017ebbab4e555cff75a70294549adf4b30`).

All app and library acceptance remains incomplete. No reviewed packet transfer or branch cleanup is authorized by this checkpoint. Continue from the pinned source and target identities, preserve the open reference conflict, and require the outstanding reviews and acceptance gates before integration.

### 2026-10-22 — TablePro expected-output restoration

Restore only `crates/tablepro-ui/tests/baselines/tablepro.txt` to the
pre-`3baa961531` and published `67212b678` blob
`e4c7211e95ba021bc88cf4fd039133051dac56ae` (SHA-256
`eaddd6b3099a1af851641d156f5aa754a5d2a85fb859bbcc92350092eb420ecb`). The
`3baa961531` rewrite replaced 136 of 168 existing digests without a separate
exact-key reference adjudication. This restoration keeps all 168 prior keys
and does not revert the production filter change or the adjacent glyph
assertions.

The restored digests are retained expectations, not semantic approval of
their values. The 136 candidate digests remain preserved as observations in
the live TablePro integrity evidence at
`/Users/donbeave/Projects/termrock-review-20261021-1536-9plajc9k/source-refresh-census-postfsck-20261022/source-quality-reviews-v10/reviewer-4/live-tablepro-expectation-integrity-b27/`.
The reference conflict remains open pending independent adjudication of
those exact keys. Any red comparison after restoration remains evidence to
review; it does not authorize regeneration, BLESS, skips, or weakened
assertions.

Independent provenance review: `tablepro-baseline-provenance-independent-20261022.json`, SHA-256
`63d06bec3e1e846abe3ac72df7f81e814ec53620a34fb99a55e454bdc35d226f`, in the external review root
above. On isolated commit `e67e7f9defd67a41ee6c8ef3b9a117f54561db01`, locked nextest ran the
existing visual target and failed its first comparison: `workbench-default 120 40 junie
truecolor`, expected `691b9e8e2f4422a7`, actual `cb84ccedef019170`. The log SHA-256 is
`09866027c27438b2c5a380a61f64bf02ba9717e690f3ef3d07b151ecfc7b8e84`. This result predates later
concurrent production fixes and is not a gate result for the integration commit. The same
focused check was repeated on live base `de6b004f6ca4697f04b768b7aa6be8d91f47d88d` with only the
restored baseline changed: one test failed with the same first mismatch; the baseline remained
unchanged. Rebased log: `tablepro-rebase-nextest-de6.log`, SHA-256
`e19af003b69740e2a27d76232f97cffa01e1149503d27ff88768459906acf53f`.


## Evidence index

Populated as increments land.
