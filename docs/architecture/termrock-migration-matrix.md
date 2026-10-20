# Termrock migration and acceptance matrix

## Current acceptance status — 2026-10-21

This section is the current decision summary. Detailed chronological checkpoints below remain evidence; their intermediate candidate statuses and branch tips are superseded by later exact-commit records. PR #38 was remotely verified at `6d1587e160d2f317436163d997cdc40f21c7263a` before this record update. The frozen source, tag object, Tuiscotti revision and read-only design reference are unchanged.

| Application | Product gate | Library gate and next dependency |
| --- | --- | --- |
| Showcase | Incomplete; complete paired comparison has not passed twice. | Remaining replay/correction paths and capability coverage require verification. Shared button/state-matrix adoption remains required. |
| TablePro | Incomplete; latest parent-validated source ecb271501 has 20 failing tests; form reference conflicts and genuine defects remain unresolved; full paired coverage is outstanding. | Real editor migration is blocked on introduced all-target compilation failures, shared focus-entry lifecycle handling and unselected completion match emphasis. The frozen default focuses directly into editing; no extra Enter is permitted. Grid painter removal is blocked on shared embedded-input geometry. History still has private widget painting and missing functional controls. |
| Jackin (`jackin-preview`) | Incomplete; replay-backed frames are not acceptable parity proof. | Read-only inventory at `adc6d0e3f668818f6f8321310452db6806c123cc` found 41 fixed-cell historical replay modules, a separate historical palette helper, and two presentation RUNS frames. Replace them by shared components; preserve legitimate brand-specific animations and domain transitions. |
| Holla | Incomplete; fixed-size/time plate output is not acceptable parity proof. | Same inventory found 46 plate builders, private Finder painting and scenario/time/viewport selectors. Shared searchable sectioned results and responsive master-detail composition must replace them while preserving domain scenarios/actions. |

The Holla/Jackin inventory found their application source subtrees identical between fixed baseline `4169783ae883788186d161af09ba9c481bd84e5c` and fixed design reference `f4867e9b5f8a08a3bb61ce80d644198a29f51984`; those reference subtrees contain no historical/RUNS/plate mechanisms. This is source evidence, not a functional comparison. Holla's frozen Finder has one search/navigation owner and a responsive master-detail policy: detail share 34%, minimum 34 cells, maximum 48 cells, gap 2, collapse below 108 columns and one summary row. General shared contracts must express these constraints without an approved-size table or application-specific library variant.

Baseline rebuilding remains diagnostic and unadmitted. The reviewed nonincremental fixed-path recipe is being independently repeated; neither a successful build nor executable-byte reproducibility establishes product parity. Ordered real-process receipts, immutable reference decisions, complete frozen capture/replay, candidate comparisons, functional outcomes, cursor/mode/cleanup checks and deliberate-defect controls remain required. Full workspace nextest, Clippy, formatting, CI, executable public consumer examples and independent final reviews are not complete. No application or library acceptance is claimed.

## Form data-aware measurement — 2026-10-21

`Form::measure_with_data` measures from the same `Placements` iterator consumed
by Form update and draw. It observes field visibility and option counts, then
applies the same group filtering, radio height, text-area height and paired
`Half` row rules. It does not invoke validation or control update handlers. The
existing `Form::measure` signature remains source compatible and is documented
as a data-independent estimate.

The public `termrock-forms` integration test uses notification-preferences data
with unrelated field IDs, three and ten transport choices, a hidden endpoint,
disabled locale, paired fields, a four-row text area, long help, and wide Unicode
labels and values. It verifies live measurement against registered rendered
field bounds and option rows, visibility, pairing, narrow and zero-width
constraints, and a narrow draw. The regression first ran against the legacy
method and failed with estimated height 24 versus rendered height 27. The
data-aware method passes with height 27; three-option and empty-option data
measure 20 and 18 rows. Measurement invoked no validators. On this worktree,
`cargo nextest run -p termrock-forms --all-targets` passed 52/52; the affected
layout/runtime/control/field/overlay packages passed 644 tests with three
existing skips. Strict `termrock-forms` Clippy and scoped rustfmt passed.

**Showcase capability coverage allowlist:** `Form::measure_with_data` is covered
by the executable public consumer test
`crates/termrock-forms/tests/form_measurement.rs`. Reason: this pure sizing API
adds no rendered control or Showcase screen, and adding a screen would change
the frozen Showcase flow. The refactor branch has no machine-checked Showcase
catalog/coverage gate; this records the disposition for gate restoration, not
catalog acceptance.

This increment proves the library API only. It does not migrate TablePro's
connection form or establish application parity.


## TablePro shared Grid row-to-cell styling — 2026-10-21

Frozen Grid behavior resolves ordinary cells over their row style and replaces
that style only for the ACTIVE cursor cell. The reusable Grid now resolves
row recipes with explicit provenance, passes the resolved row channels into
cell defaults, and preserves per-cell recipe overrides and modifier removal.
Existing caller `part_defaults` are included for both ROW and CELL, including
explicit `Clear` provenance when no higher-precedence recipe supplies that
channel. Empty ACTIVE cells use the same complete-style replacement as
populated ACTIVE cells. No TablePro-specific style or inset API
was added.

TablePro had been painting BOLD across the selected data row after Grid drew
it, and repainted five cells at the beginning of the second visible column
with the surface style. Both caller-side corrections are removed; Grid owns
row/cell styling and its existing column geometry. The real-app regression
`unfocused_grid_cursor_does_not_style_the_cursor_row` failed before removal
(on the exact `9b4aae59b5c006e64b3791fa15804f6411633e4d` source) and passes
after. Grid-focused styling, ACTIVE precedence, cell channel overrides and
modifier removal pass in all four color levels.

Validation on merged source `df30dc0`: `cargo nextest run -p termrock-grid
--all-targets` passes 133/133. Real Grid buffer regressions cover unfocused/active cursor row inheritance,
empty cells, explicit row and cell `Clear`, recipe-over-default precedence,
and deleted-row strike-through.
The TablePro unfocused-cursor and deleted-row strike regressions pass; the
latter initially failed with inherited styling, was corrected to preserve the
row's deletion strike unless CELL explicitly removes it, and now passes. `cargo nextest run -p
tablepro-ui --all-targets` runs 184 tests: 173 pass and 11 fail. Remaining
failures are `explain_orders_shows_a_plan_cell_the_error_frame_hides`,
`close_only_binds_target_but_quit_binds_every_tab`,
`frame_tablepro_grid_500x14_120x40`, `query_and_inline_drafts_survive_cancel_without_commit`,
`result_confirmation_rejects_changed_sql_and_vanished_target_is_noop`,
`inserting_after_five_hundred_rows_reveals_the_new_owned_record`,
`result_confirmation_cancel_preserves_inline_draft_and_confirm_targets_captured_tab`,
`uppercase_u_requires_confirmation_and_preserves_sql_and_other_tab`,
`plain_u_in_sql_editor_is_text_and_leaves_inactive_table_pending`,
`tablepro_visual_baseline`, and
`focused_query_identity_and_draft_survive_earlier_insert_close_and_reorder`.
Against the exact `df30dc0` source with this change's four files restored,
TablePro has 183 tests: 172 pass and 11 fail. The candidate has 184 tests:
173 pass and 11 fail, with the exact same failure names; the added real-app
regression passes. Grid strict Clippy passes. Combined Grid/TablePro strict
Clippy is blocked by `tablepro-domain::Tab` large-enum-variant lint at
`tabs.rs:315` (this task changed no domain source). Scoped formatting is
checked separately. Full
product parity remains incomplete; no paired visual approval is claimed.

An independent Grid review then found semantic underline provenance was lost
when the resolved row underline was converted to a raw Custom color for CELL
defaults. `PaintStyle` now retains the bound underline role and surface through
canonical resolution; Grid reuses that role for ordinary cell inheritance,
while raw underline replacement clears the provenance. ACTIVE cells explicitly
reset an inherited underline only when their own resolved style has none. No
global downgrade behavior changed. The regression covers populated and empty
cells at TrueColor, ANSI-256, ANSI-16, and Mono. With provenance propagation
disabled, the test fails in Mono (White cell underline vs Gray row underline);
with the fix, focused Grid tests pass. `cargo nextest run -p termrock-theme -p
termrock-grid --all-targets` passes 263/263; strict scoped Clippy and fmt pass.

After normal-merging PR #38 source `25f5792666cefcb1d51d56fcb973fd06a15b8741`,
`cargo nextest run -p tablepro-ui --all-targets` was run on the merged candidate
and on the merged PR source with the four Grid-owned source/test files restored.
The candidate had 184 tests (173 pass, 11 fail); the source base had 183
(172 pass, 11 fail). Both runs failed the exact same 11 test names listed
above. The TablePro Grid-cursor regression is the one added passing test. This
comparison is limited to TablePro nextest and does not establish product parity.

Superseding the 25f attribution above, PR source `1dbf3b2caa429a49ec1a38fba641735a3890b918`
removes the historical completion-popup case and has 181 TablePro tests (170
pass, 11 fail) with this Grid change's four files restored. The merged candidate
has 182 tests (171 pass, 11 fail); the 11 failing names are identical. The new
single passing test is `unfocused_grid_cursor_does_not_style_the_cursor_row`.
This remains a focused regression comparison, not application acceptance.

Superseding the `1dbf3b2c` attribution above, PR source
`d371d8f4460d940c2fc460bc3b0a1e79924e9265` removes the historical scrolled
TablePro grid module. With the four Grid-owned source/test files restored,
`cargo nextest run -p tablepro-ui --all-targets` runs 180 tests (169 pass, 11
fail). The merged candidate runs 181 tests (170 pass, 11 fail); both runs have
the exact same 11 failure names above. The added passing test is
`unfocused_grid_cursor_does_not_style_the_cursor_row`. Grid/theme remains 263/263
on this merged source. No product parity acceptance is implied.

Updated: 2026-10-21. Status: **in progress; all four applications incomplete**. This is the current migration record. Older audit reports remain evidence, not a complete inventory or acceptance declaration.

## CI feature-gate increment — 2026-10-21

Velnor Actions 0.1.29 now has a named `testing` Rust configuration while the
existing `default` configuration stays unchanged. The generated `termrock`
test task enables the facade `testing` feature and discovers the gated button,
text-input, slots, and split-block consumer binaries. The generated command
ran 13 tests across six binaries (13 passed, 0 skipped); actionlint and zizmor
also passed. Generator 0.1.29 reports explicit default-feature fallbacks for
packages with no matching feature and does not put `testing` in their Cargo
argv. Its generated doctest jobs still use `cargo test --doc`, which conflicts
with the repository's nextest-only rule and remains unresolved. See the
[CI testing-configuration audit](../audits/2026-10-21-velnor-testing-configuration.md).
This does not establish full CI or product acceptance.

## TablePro query results 120×40 truecolor — 2026-10-21

`tablepro/query/results/120x40` truecolor and nocolor match with the live
results card. The ANSI stamp is gone. A slow frame coalesces Esc and Ctrl+R
into Alt+Ctrl+R; that chord runs the query. In mono the Run label is bold
reverse. Duration comes from the executor (`15 ms` for this statement).

Still red on this scenario: `120x40` 256 (569 cells, tab rule `#444444` vs
`#3a3a3a`), 16 (306), none (1570). Truecolor other sizes: `80x24` 893,
`100x30` 1472, `160x50` 2284, `72x20` footer clips `Ctrl+X Explain`. `40x12`
and `60x18` boot on the 72×20 too-small gate, so `Query 1` never appears.
Full visual runs and `origin/main` are not done.

## TablePro query completion 120×40 truecolor — 2026-10-21

`tablepro/query/completion/120x40` truecolor matches. The editor line, the
four-row catalog menu, and the EDIT footer come from the live completion
batch. The ANSI stamp module is removed. Results `120x40` truecolor stays
green on the same tree. The menu geometry is still the 120×40 truecolor
frame. Nocolor reaches the menu and differs by 125 cells (no reverse
video). Other completion sizes and color levels are not claimed.

## TablePro scrolled-right 120×40 truecolor — 2026-10-21

`tablepro/table/scrolled_right/120x40` truecolor matches with the live grid.
The ANSI stamp is removed. The cursor stays on `updated_at` and the window
shows `‹11`. Other sizes of this scenario are not claimed.

## TablePro history tab 120×40 truecolor — 2026-10-21

`tablepro/overlays/history_tab/120x40` truecolor matches. Ctrl+Y focuses the
history list. The card paints the seeded log: search row, ten oldest entries,
and the detail card. The ANSI stamp is removed. Other sizes are not claimed.

## TablePro data 120×40 color levels — 2026-10-21

`tablepro/table/data/120x40` truecolor matches. A right-aligned value used to
clear its pad to Default; the vacated cells keep the row fill. The cursor row
is bold through the gutter tail, the gaps, the preview arrow, and the slack
past the active value. `tablepro/table/scrolled_right/120x40` truecolor matches,
including that gutter tail. `tablepro/table/data/120x40` nocolor and 256 match. Legacy inks use the
frozen 256 indexes (`#3a3a3a` → 238, `#474747` → 239, `#737373` → 242,
`#808080` → 245, `#173f1c` → 22), not the nearest gray ramp step. Other sizes
are not claimed. Full visual runs and `origin/main` are not done.

## Palette and independently diagnosed reference corrections — 2026-10-20

This increment follows style delivery `e597618d6576a725538a70d71250fd81c6599c28`.
The only production changes are the three Junie palette entries independently
reviewed at `95cdc7bdb111dc1b683a638b421dae718bb29643`: ANSI-256 muted/faint
245/242 and Mono faint `Gray`, all explicitly assigned by frozen `4169783`.
No application painter, captured frame, baseline tag, or test-tool pin changed.

Reference corrections were independently diagnosed and reviewed before merging:

- Empty title isolation: frozen widget execution proves muted RGB 138 rather
  than 128. All ownership and override assertions remain.
- Empty aggregate: immutable historical executions reproduced both old
  fingerprints. Projecting their 32 cases/5,376 cells through frozen palette
  assignments changes only 32 foreground channels; every other cell field is
  identical. Historical values and artifact hashes remain in the audit. This
  proves palette correction plus pre-inheritance compatibility, not that new
  Loading/Partial APIs reproduce frozen application screens or cursor state.
- Nested modal cells: frozen backdrop semantics retain ordinary text and dim
  its style. Full X/R/P/U cell and cursor assertions preserve both paint paths
  and draw orders. Independent execution against the genuine old compositor
  fails the corrected test, as recorded in the audit.
- Item-row fixture: its original hash remains asserted under an explicit
  historical palette. The independently established 174 muted foreground
  corrections produce the expected complete buffer; a wrong-color negative
  control fails. The candidate hash was not copied into the fixture.

Reviewed worker sources are `45b4ca679272b4b3c8948139cd87c1473d652bbf`,
`d3e5647c87aeb33cab03b378fe0f7111718ff3a0`, and
`23a3fac940f24a499b532b0ef580f19355715043`. Their detailed decisions remain
in the corresponding Empty, nested-layer, item-row and palette audits.
The integrated repeat runs 675 tests: 669 pass, the same six pre-existing
Dialog layout tests fail, and three remain skipped. Those failures are not
reported as green. Logs are retained under
`/Users/donbeave/Projects/termrock-cli-evidence-20261020/pr38-palette-reviewed/`.
Full library, application, format, CI and paired acceptance gates remain open.

## Reviewed shared-style delivery — 2026-10-20

The source series reviewed at `2863bf6e7f011c321088263cce12d423b16d73d3`
starts from PR #38 commit `1d7b8eb291e37cb4806ce1369a8a0abab08a81ac`.
Independent review verified all ten changed Rust files are byte-identical
to previously cleared `e72dbb15369d541ee86095f6caaee3f7e4680828`; current
TablePro fixes, Dialog backdrop handling, and focus-bar suppression remain
intact. Explicit theme, overlay and instance style channels reach the shared
components without application repainting. No application, CI, captured frame,
expected hash, or frozen reference is changed by this series.

The same four-package nextest command runs 675 cases after the change versus
662 on the parent: all 13 new regressions pass, two parent failures are fixed,
and the same 14 existing failures remain. The full gate is **not green**.
Those failures comprise six Dialog layout cases and eight theme capability,
empty-style, or nested-backdrop cases. The palette corrections and reference
repairs are separate review candidates, not silently included here. Strict
package Clippy and formatting pass. Detailed source mapping and original logs
are in [the style audit](../audits/2026-10-20-shared-style-regressions.md);
independent evidence is at `/tmp/termrock-style-review-2863`, and the parent
repeat is retained under
`/Users/donbeave/Projects/termrock-cli-evidence-20261020/pr38-style-delivery/`.

Fresh real-CLI evidence on unchanged parent `1d7b8eb` captures and replays all
four selected TablePro checkpoints. Picker-open and tab-list-open match the
candidate exactly, but both Esc-dismissed flows differ in 82 footer cells;
cursor state matches. This limited sample remains incomplete and does not
approve the runtime/Picker integration. See the preserved
`tablepro-picker-1d7/paired-tablepro-picker-result.json` evidence index beneath
the same external evidence root. All four applications remain incomplete.

## Fixed sources and starting tips

| Source | Identity recorded before this work | Use |
| --- | --- | --- |
| `parity/termrock-visual-baseline` | `0b8f4e32b24a137987307600b10d892bec0619c3` (remote observation 2026-10-20 10:52:50 UTC) | PR #38 candidate branch |
| PR #26 fixed source | `f4867e9b5f8a08a3bb61ce80d644198a29f51984` | Product/contract reference; do not substitute a moving branch |
| `visual-baseline` tag object | `2718a6b8127a522a3d351d0b4bdb7542fc8da542` | Frozen oracle; never move |
| Frozen source commit | `4169783ae883788186d161af09ba9c481bd84e5c`, tree `ad0f7cf8899df8c5fea3ced3f60adf0ac9a2d297` | Isolated baseline archive and CLI binaries |
| Tuiscotti | `a1ea27f757d75d07b05499276cd932e9f8e067ad` | Fixed PTY and frame-comparison tool |

Record exact tested source commits in every run. Product refactoring remains scoped to PR #38; PR #26 at the fixed revision above and the frozen baseline remain product references. A later explicit instruction authorized generated workflow upgrades on `main`, `parity/termrock-visual-baseline`, and `design/interaction-polish`; main delivery must use CI-only PR #39, and design changes are limited to generated CI configuration and migration evidence. No application/test-tool refactor or baseline change is authorized there. The current PR #38 runner can test separate builds of each fixed reference without modifying those references. Do not update frozen snapshots to make a candidate green.

## Application migration matrix

| Application | Baseline screen and flow families to preserve | Shared owner required | Current evidence / missing work | Product | Library |
| --- | --- | --- | --- | --- | --- |
| Showcase | Shell/chrome and page navigation; component catalog pages for buttons, inputs/forms, lists, tables, trees, tabs, menus, dialogs/popups, feedback, text/code, layout; each demonstrated state/configuration and keyboard/pointer route | `termrock` widgets, overlays, runtime, text/layout/theme; Showcase supplies demo data and actions | Native root gate at recorded checkpoint: 1,479 passed, 615 skipped. Current external component checkpoint `e7c4be2e5` reports 1,479 root tests, 48 code-editor checks, clippy/fmt; not full workspace or frozen CLI acceptance. Live table/list state and footer ownership remain open. | INCOMPLETE | INCOMPLETE |
| Jackin (`jackin-preview`) | Intro/outro and first-use; menu and capsule; manager/cockpit; workspace, split panes, editor and completion; picker, drawer, settings, accounts, usage, launch, help, save/cancel and resize flows | `termrock` menu, picker, editor/completion, drawer, master-detail/layout, shell and shared input/focus; Jackin retains simulation/domain actions and branded intro/outro/fade | Frozen mono test coordinate correction independently verified; no full candidate journeys. Manager/picker consumer still has full-frame failure; generic shell disappears under open picker unless shared owner layering is fixed. | INCOMPLETE | INCOMPLETE |
| TablePro | Connection/catalogue, create/edit/reconnect; multiple query tabs and tab-list/picker; explorer/workbench; SQL editing/completion; results, error, explain/analyze, history; filter editor, dialogs, help, scroll and resize | `termrock` Picker and tab list, table/grid, editor/completion, dialogs, shell/layout; TablePro owns SQL, connection and query effects | Live Picker/tab-list implementation is missing; a prior SQL editor Tab mismatch remains. Eighteen historical ANSI replay modules remain. The shared sorting component work currently targets Showcase Tasks, not an accepted TablePro conversion. | INCOMPLETE | INCOMPLETE |
| Holla | Home/catalogue and Finder; first-use, monorepo, activities, Docker/disk cleanup, launch failure; parity discovery/history/files/browser/Git/Cargo/task sources; menus, prompts, help, resize and terminal modes | `termrock` shell, menu, picker/search, list, dialog, status, layout/scroll; Holla owns simulation/domain actions and data | Single Holla menu-hover live case passed three phases. Existing Holla native digest and broad CLI captures remain red. `*_plate` model dispatch and application paint helpers are production render paths pending replacement and proof. | INCOMPLETE | INCOMPLETE |

A component-level buffer test, migrated digest, or same-source control does not satisfy product acceptance. A screen that paints through a large private UI crate does not satisfy the library gate.

## Paired test system and coverage

The shared runner is `tools/frozen_baseline.py`; its harness patch and copied helper remain test-only. The test suite drives real frozen and candidate CLIs through the pinned Tuiscotti PTY, captures baseline frames, replays the frozen binary, then compares the candidate. Canonical decoded frames include every cell/style attribute and cursor state. Original expected data is immutable. Each successful case must produce ordered checkpoint receipts in every phase.

- Original frozen inventory: 28,895 ignored scenarios plus 20 ordinary checks. The original stored corpus has 7,675 approvals and 21,220 pending rows. `--all` fails closed on the missing references; a selected case is partial.
- Ordinary checks: 11 source-only, 9 frozen-CLI, 9 candidate-CLI. `isolation::env_isolation` belongs in both real-binary groups. The same guard suite and runner files must exist byte-for-byte on both comparison branches.
- Registered extension inventory: 174 cases. The current pilot exercised 36 cases and emitted 174 ordered receipts in each reported phase. Extension coverage supplements the original inventory; it does not replace or rename original scenarios.
- Latest recorded candidate full-corpus attempt: 240 scenarios red. It is not acceptance. Most recent candidate reports also include a first Holla menu-hover cell mismatch before the shared title-hover correction; the one-case Holla rerun then passed. Re-run exact current branch heads.
- Independent diagnosis confirmed a stale grown-resize approval. The exact frozen tuple and immutable old/new hashes are recorded in `tools/frozen_cli_references.json`; only baseline capture can apply the documented decision. Frozen replay and candidate comparison remain strict. The old approval and original red reports remain preserved. Shrunk resize has no reference exception.
- Required next expansion: meaningful intermediate checkpoints after input, resize and controlled time; adjacent/breakpoint sizes, changed/reordered data, empty/large data, repeated open/close, pointer/pressed states, text edits, supported color modes and wide/combining text. Existing registered checkpoints are not proof of unregistered flows.

Evidence details and limits: [frozen CLI parity](../audits/2026-10-20-frozen-cli-parity.md), [full CLI runner](../audits/2026-10-20-full-cli-runner.md), [live frame support](../audits/2026-10-20-live-frame-support.md). Audit files are superseded only where this matrix says so; underlying red evidence remains active.

## Production replay-path inventory

Audit searched production crates for `historical_*`, `*_plate`, `RUNS`, ANSI includes, frame decoding, raw-buffer writes and direct paint. Findings below are confirmed replay or fixture-selected render families, not just name matches. Ordinary semantic demo data, icons, terminal viewers, and genuine layout breakpoints require individual review and must not be removed by naming heuristic.

| Production path | Observed mechanism and purpose | Replacement owner / status |
| --- | --- | --- |
| `crates/jackin-preview-app/src/app/historical_*.rs` (40 replay files: `historical_accounts_add_form`, `historical_accounts_add_form_required`, `historical_accounts_detail`, `historical_accounts_detail_80`, `historical_accounts_drawer`, `historical_accounts_filter`, `historical_accounts_filter_80`, `historical_audit_accounts_80`, `historical_capsule_menu`, `historical_capsule_menu_80`, `historical_capsule_new_tab`, `historical_capsule_palette`, `historical_capsule_split_vertical`, `historical_capsule_zoom`, `historical_cockpit_cancel`, `historical_cockpit_info`, `historical_editor`, `historical_editor_auth`, `historical_editor_env`, `historical_editor_mounts`, `historical_editor_mounts_dirty`, `historical_editor_roles`, `historical_editor_save`, `historical_hard_cases_80`, `historical_hard_launch_locked`, `historical_hard_launch_picker`, `historical_inspect`, `historical_intro_f300`, `historical_launch_picker`, `historical_manager_menu`, `historical_settings_agents`, `historical_settings_env`, `historical_settings_mounts`, `historical_settings_mounts_80`, `historical_settings_route`, `historical_settings_route_80`, `historical_settings_save`, `historical_settings_trust`, `historical_usage_detail`, `historical_usage_overview`) | `RUNS` arrays encode positioned text and colors; paint those cells after scenario/state/viewport selection. These are production captured-frame replays. `historical_paint.rs` is a separate historical style-recipe helper, not a `RUNS` frame; review it as theme ownership, not as proven replay. Likewise Showcase `historical_palette` functions return semantic theme styles; the name alone does not establish frame replay. | Move behavior into shared widgets/blocks and feed Jackin domain data/actions. All 40 replay files remain unresolved unless removed with a paired journey. Manager `?` paints `HelpOverlay` over the dimmed manager. `historical_help_overlay` is removed. `jackin/manager/help_overlay/120x40` truecolor matches, including the hidden cursor. `t` on the paused launch paints the launch trace label on the activity row. `historical_cockpit_debug` is removed. `jackin/cockpit/debug` and `jackin/scenarios/launch-running` match at 72×20, 80×24, 100×30, 120×40, and 160×50 for truecolor, 256, 16, nocolor, and `--color none`. 120×40 still blits the presentation frame. The other sizes compose the baseline cockpit (strip, hero, stage rail, activity, cancel, footer) and downgrade that frame. Pending 40×12 and 60×18 have no approved snapshot (`missing-approval`); they are not cell diffs and must not be accepted. `jackin/cockpit/info` and `cancel_confirm` match at 72×20, 80×24, 100×30, 120×40, and 160×50 truecolor. Empty padding keeps the dimmed cockpit ink (`fill_keep_fg`); chrome is the centered block. 120×40 truecolor still uses the historical stamp. 256, 16, nocolor, and `--color none` for those overlays still differ. |
| `crates/jackin-preview-presentation/src/{capsule.rs,launch_running.rs}` | Separate presentation-level positioned `RUNS` renderers; proves replay is not limited to app crate. | Shared component composition; unresolved. |
| `crates/tablepro-ui/src/app.rs` | Production predicates select fixed scenario/size/state frames; modules parse `RUNS` from `include_str!` ANSI or hold fixed runs; app repaints after normal UI. | Quick-open, tab list, and Safe Mode use `Picker`. Help uses `HelpOverlay`. Delete connection and close-tab confirmation use `Dialog`. The filter editor uses shared overlay chrome plus `TextInput`. A failed query paints the editor, result tab, and error card from the statement error. Explain and explain-analyze paint the editor and the plan strip from the plan tree. Query results paints the editor, chip, and grid from the executed result. Query completion paints the editor line, catalog menu, and EDIT footer from the completion batch. Those replay modules are removed. Scrolled-right orders uses the live grid window. History paints the search row, oldest-first list, and detail card from the seeded log. The new-connection card paints Basic fields from the empty draft. The Advanced page paints the SSL, SSH, and startup placeholders from that same card. Hiding the explorer paints the vertical Explorer rail and the inset query card. Zooming an empty query paints the blank results card, the tab focus mark, and the zoom footer. `tablepro/workbench/maximized/120x40` truecolor matches. No `historical_*.rs` replay module remains. An empty SQL statement keeps the idle editor. The EDIT footer is shown only while that editor is focused. With the explorer focused, the footer is the explorer hint row. `tablepro/audit/production/120x40` truecolor matches. |
| `crates/showcase-demos/src/pages/chrome.rs` | At source `0b8f4e32`, `MEDIUM_100X30` is explicitly copied from an approved snapshot; `paint_medium_100x30` writes its positioned cell tuples and dispatch selects exact 79×26 page dimensions. Additional narrow/wide render paths need the same purpose audit. | Real shared chrome/menu/workspace components and responsive composition; unresolved. |
| `crates/showcase-demos/src/pages/selectable/{frames.rs,delete_confirm.ansi,bulk_mixed.ansi}` | Imports two approved ANSI frames, decodes cells, resets and writes raw buffer for selectable demo cases. | Real shared selectable/table/list components with inert reference-state data; unresolved. |
| `crates/holla-sim/src/catalogue.rs` plus `crates/holla-ui/src/screens/home.rs` | Scenario and exact viewport/time `*_plate` dispatch selects named fixture view; home helpers paint dedicated fixed families, including parity files/browser and tag lists. Some catalogue functions build semantic domain view data, so review each function before removing it; exact dimension/time selection plus direct plate painters are not accepted as final architecture. | Shared Holla screens/components driven by simulation state and domain view data; unresolved. |
| `crates/termrock-e2e/src/lib.rs` | Test/evidence frame conversion and fixture-generation helpers. Search hit is test support, not by itself production replay; verify dependency/build reachability before any removal. | Test-only harness ownership, never app render path. Audit reachability remains open. |
| `tools/parity_replay.py`, `tools/parity_approve.py`, `tools/test-inventory/historical.json`, `crates/termrock-e2e/**`, `crates/visual-harness/**`, `crates/termrock-conformance/tests/visual_baseline/**`, `crates/showcase/tests/tuiscotti_qualification.rs` | Reference frame management, source inventories, and real-CLI/test harness checks. These are verification paths, not app render paths, and reference data belongs here. Verify dependency edges so no app binary links test-only capture code. | Keep reference artifacts and capture in test system; no production dependency permitted. Dependency/build graph proof pending. |

This is a scoped source inventory from the recorded branch tree, not a proof that *all* helper, presentation, build-script or generated-code paths were exhausted. Inspection of `crates/termrock-e2e/build.rs` found suite-file hashing and `rerun-if-changed` generation only; it does not generate application render code. The earlier 35-site ownership review and 85/88 audit finding counts are findings, not path counts or a complete coverage denominator. Continue repository-wide dependency/build/generated-source tracing. Do not claim replay-free architecture until each production path above has a replacement and the full audit records zero remaining paths.

## Architecture and independent validation

| Gate | Evidence | Status |
| --- | --- | --- |
| Independent production crates | Prior exact checkpoint: 19 production packages, 1,313 passed, 4 skipped; package clippy green | Partial; rerun at current integrated commit |
| Root default members | 1,479 passed, 615 skipped at prior checkpoint | Partial; six members only |
| Full workspace | 3,795 tests ran, 3,760 passed, 35 failed; workspace clippy failed at `crates/tablepro-ui/src/app.rs:9284` (`expect_used`) | RED; failures recorded in [independent validation](../audits/2026-10-20-independent-workspace-validation.md) |
| Code editor completion | 48 focused tests green at `e7c4be2e5` per independent checkpoint | Component only; consumer and outside-click ownership remain open |
| Master-detail | Shared geometry/state API and 374 tests, 3 skipped at recorded checkpoint; does not include live Manager composition | Component only; app use and full frame evidence absent |
| External public consumer | Not yet demonstrated for proposed composition API with unrelated domain data and no private app helpers | RED / missing |
| Ownership/replay audit | Existing exception ratchet and 35-site report; 85/88 prior audit findings are not a complete path inventory | RED / partial |

The new `MasterDetail` owns layout geometry/state. Frozen narrow detail replaces body content; it is not a floating modal. Live Manager composition and actual dialog/picker layering remain separate work. The completion adapter owns editor/popup lifecycle; outside clicks currently dismiss without forwarding to owner. Picker policy/title-style and MenuBar changes with native fixture conflicts are deferred WIP at the cited checkpoints; do not infer parity from tests that only assert a proposed style.

## Decisions and next evidence

1. Keep frozen source/tag/Tuiscotti IDs fixed. Record commit, tree, binary hashes, environment, size, mode, scenario inventory, receipts and result for every candidate run.
2. Use the same PR #38 runner, test helper and manifests when testing each fixed reference and candidate. Earlier byte-identical dual-branch publication is historical; the latest instruction prohibits further PR #26 changes. A test inventory mismatch still blocks comparison.
3. Keep each app INCOMPLETE until original plus extension inventory and all required ordinary checks pass on the exact same candidate commit, with no skipped or missing case and a reviewed reference-data decision.
4. Keep library gate separate: remove replay from production, prove public shared APIs own mechanics, compile an external consumer, and complete source/dependency/build/generated-code review.
5. Resolve red behavior in shared components or domain model; retain baseline flows. Never select a renderer by test label, exact viewport/clock, terminal detector or captured accent. Never repaint a widget in app code or register absent controls to move focus/cursor.
6. Continue in order: TablePro Picker/tab-list real consumer and replay removal; Showcase real button/catalogue paths; Jackin semantic shell/workspace/picker/editor; Holla semantic catalogue/layout. Expand shared blocks only for demonstrated consumers.

No application has full product or library acceptance. This matrix becomes current at its merge commit; append exact commits and evidence without relabeling partial checks green.

## Latest bounded evidence

- Exact PR checkpoint `e7c4be2e52bc9877ba5edc9930235cceab19a336`: independently built candidate; Holla File-menu hover and four stored application controls passed all three phases with zero checkpoint differences. Evidence: `/Users/donbeave/Projects/termrock-current-cli-proof-20261020/live-paired/1792493484804932000-1666732d0b3c/report.json` and `/Users/donbeave/Projects/termrock-current-cli-proof-20261020/live-paired/1792493572222308000-16335f5b43ea/report.json`. Five selected checkpoints are not full parity.
- Showcase Tasks WIP `fa02fa54a6151c43cccd56d61e096f4a87808f13`: seven focused consumer checks passed. Fresh real CLI comparison of six sorting workflows at 120×40 truecolor failed at every initial checkpoint, with 1,952 differing cells each, including cursor shape. Frozen capture and replay matched 24 receipts each. Evidence: `/Users/donbeave/Projects/termrock-live-cli-table-consumer-20261020/live-paired/1792493587115197000-c06708be5d1e/report.json`. Earlier consumer evidence is superseded for this WIP. It is not accepted.
- Shared extension manifest hash `b995c00f0dc14d4e0a81d1d56140e1c10ba7936a043e0135196f8fe8421cabb8`: all 174 registered sequences are checked exactly. Thirty incorrect 100×30 checkpoint lists were repaired before integration; the omitted-middle guard is mandatory. Twenty-three guards pass; frozen capture/replay for 36 cases matched 174 receipts each. Remaining registered scenarios and required workflow obligations remain unverified.
- Whole-repository formatting at source `0b8f4e32` is red in the pre-existing Chrome fixed-cell constants. The new test module passes its own formatting check; root nextest and clippy passed. No formatting or architecture failure is relabeled successful.
- CI planning was independently observed failing before Rust jobs ran because standalone helper discovery failed. Skipped Rust jobs are blocked evidence, not passing validation. The reviewed discovery correction and generated symlink correction were pushed to PR commit `2de0d01255390d2a02b7775ee452268782802a38`, with remote SHA verified. Remote run `39415797723` passed Actionlint but failed Check generated files in Plan; downstream Rust jobs were skipped and provide no passing validation. DCO remained action-required. The local preview success does not supersede this remote failure; diagnosis is active.

## Current integration obligations

The staged contract work above is not all present in the candidate branch. Source identities in each evidence entry remain authoritative.

- CI discovery correction `445c7e9af00bdbc5f9a2de86172b3e91b31eab74` received independent generated-tree and plan review. It discovers 47 packages and 44 crate jobs and adds five missing visual-harness tasks. It does not establish CI acceptance: 39 generated doctest commands conflict with nextest-only policy; no generated task enables the `testing` feature; standalone helper tests and paired-runner tests lack a normal generated execution path. The generated `.github/CLAUDE.md` symlink is corrected through `[agents] claude_md = "symlink"` in source configuration and regeneration (`8beaf2865b922fbea0ae082f914367d1fed64609`). The remaining execution gaps need supported generator corrections, not hand edits. See [CI planning audit](../audits/2026-10-20-ci-planning.md).
- Independent reference review confirmed the grown resize approval is stale: two real frozen captures and overview controls have identical 4,800 cells and cursor, ANSI SHA `f2e07ed284673b1d543441767460193e987034e163ca7ea7d3d4496473a0d442`, full frame JSON SHA `01e9f7d349fc8075df7b64188db97a05705792aac8c781184db14a6d1ecc253a`. Legacy grown/shrunk SHA `7769a6686dfc5cb5c69facb03f54a0fa57e963b35fcf315b1c4625745e2b7708` predates the October Components card. Preserve both artifacts. The documented decision authorizes only the independently reviewed grown tuple in paired capture, with exact hashes and an explicit legacy-red verdict; the exact capture-only decision is implemented and the two-case frozen capture/replay pilot passes, and the deliberate hidden-cursor-only negative control has been verified. Shrunk120×40 separately fails its no-change wait before any checkpoint and has no approved reference exception. Evidence: `/Users/donbeave/Projects/termrock-reference-review-20261020`.
- Test integrity: canonical ANSI does not retain hidden cursor position, shape, or blink. The reviewed helper adds full semantic frame equality, mandatory semantic receipts, and preserved actual-frame diagnostics. A fresh two-chunk pilot matched frozen capture/replay and rejected the candidate overview for hidden cursor `Block` versus frozen `Bar`; its second candidate case was not run. The suite was published identically on both comparison branches, and a deliberately mutated hidden cursor was rejected with zero cell differences. Earlier ANSI-only pilots cannot establish complete cursor-state parity.
- TablePro priority: replace both tab-list and quick-switcher replay paths, phantom editor/cursor registrations and fixture predicates with the real Picker. Shared field editing, presentation preservation, content slots, menu/clipboard/selection contracts and unrelated-data consumer proof precede consumer acceptance. Do not restore a captured background when live workbench defects become visible.

- Shared contract review remains active: field command execution, selection invalidation, scrolled drag coordinates, secret clipboard temporaries, and duplicated command mechanics require fixes before the field API can be accepted. The strict architecture scanner is also unaccepted: referenced modules behind Cargo-target aliases and missing live includes exposed false-green source accounting. These findings do not authorize production replay paths.

- Same-source paired-runner positive control: a clean, separately built frozen-source candidate passed Holla Files and Showcase Overview at 120×40 truecolor, with chunk size one. All three phases emitted both ordered checkpoints and matched complete semantic frames. Evidence: `/Users/donbeave/Projects/termrock-live-self-control-20261020/live-paired/1792496449799955000-02f47d78355a`. This validates the bounded harness path, not the refactored applications.

- Shared test-system integration checks: 28 Python guards and 25 standalone semantic-helper nextest tests pass, with helper clippy and formatting and extension formatting clean. The optional underline-color and `Default`/`Indexed`/`Rgb` variants match the pinned serialization schema. Exact per-phase inventory unions remain mandatory across multiple chunks. These checks do not accept application parity; deliberate candidate-defect verification is recorded below.

- Deliberate candidate defect: isolated frozen source changed only the default caret from `Bar` to `Block`; the runner recorded its dirty source fingerprint and separate candidate binary hashes. Frozen capture/replay passed and candidate comparison failed solely on hidden cursor shape, with zero differing cells. Evidence: `/Users/donbeave/Projects/termrock-live-hidden-cursor-proof-20261020/live-paired/1792496903298882000-d4f64b7e5c12`. The immutable baseline and approvals were unchanged.
- Shared publication checkpoint: PR `0168ccc15246343e6a1f846a30bf7964991b439c` and design `46ff0ebbf457ed46da301706229d941ee985a102` were pushed and remote IDs verified. Thirteen shared test/helper/manifest/documentation files were byte-identical; the identity manifest is `/Users/donbeave/Projects/termrock-cli-evidence-20261020/shared-semantic-branch-identity.json`. These remain partial test-system checks, not application acceptance.

## Latest instruction checkpoint and delivery boundary

Fixed product references remain PR #26 source `f4867e9b5f8a08a3bb61ce80d644198a29f51984`, baseline source `4169783ae883788186d161af09ba9c481bd84e5c`, tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`, and Tuiscotti `a1ea27f757d75d07b05499276cd932e9f8e067ad`. No ref changed.

Before the workflow-only upgrade, recorded tips were `main` `a97e351df736b576904bc450619ca347ea64d65b`, remote PR #38 `1a947a251ab9b045610b47729bf481419b9fca8d`, and observed design branch `8681124d2cbbc75ef8ed998b5918883039a022c5`. The shared PR #38 remote later advanced to `a848219fd858ed6352fcb403b2cc5d5953083e6f`. Worker integration commit `90cd26402ffb421fd13db4418a8f9201c3a31d5e` was local only and is not a recorded remote PR #38 tip. The design comparison branch at `8681124d2cbbc75ef8ed998b5918883039a022c5` is an observed branch tip, not the fixed PR #26 reference; fixed PR #26 source remains `f4867e9b5f8a08a3bb61ce80d644198a29f51984`. Design workflow author tip `8533344520cfdf12734c1c149fc89ec946b3ea46` is based on `8681124d2cbbc75ef8ed998b5918883039a022c5`.

Latest workflow instruction explicitly authorizes updating all three branches with Velnor Actions generated files. Main upgrade is isolated in CI-only PR #39; do not push directly to `main`. Parity generated-file regeneration and audit are worker changes for integration into PR #38. Design upgrade is limited to workflow source/config, generated files, and migration evidence. Do not add application or test-tool changes to design, retarget or rewrite PR #26, or change frozen references. This scoped workflow authorization supersedes the earlier blanket ban on additional design/main workflow updates only.

The parity and design upgrades use stable Velnor Actions `0.1.29` at source commit `e7b2f088715379c22be473fab896a43d50cc16fc`. Independent complete-input parity generation found the previously committed manifest and CI workflow used nextest profile `default` in 88 commands while tracked `.config/nextest.toml` configures `ci`. Regeneration changed only `.github/velnor/task-execution-manifest-v1.json` and `.github/workflows/ci.yml`; a fresh preview matched all 21 `.github` entries by path, kind, bytes, mode, and symlink target. Plan: 47 workspace crates, 44 Rust execution jobs, 259 task IDs. Design verification at author tip `8533344520cfdf12734c1c149fc89ec946b3ea46` independently confirmed all 22 `.github` entries against a fresh full-input preview; its plan selected the configured `ci` profile and found two workspaces and two jobs. Both results establish generator consistency, not successful remote CI. Existing blockers remain: generated doctest tasks invoke `cargo test --doc` contrary to the nextest-only instruction; DCO is action-required in observed remote runs; full product parity and full coverage remain incomplete. Details: [Velnor parity regeneration](../audits/2026-10-20-velnor-0.1.29-parity.md), [CI planning audit](../audits/2026-10-20-ci-planning.md).

All product changes remain on `parity/termrock-visual-baseline`; workflow-only PR #39 is not a replacement for PR #38. PR #38 must reach acceptance and required repository review before merge. Full application comparison must pass twice on the same integrated commit; no complete passing run has been established.

The newly requested replay-origin audit starts from `6923311119e060176ee3cf07c585416556416024`, `f067d3f088621ec2a0fe4e7eaf7ae2c3a7c5957c`, `fd3991049709f08c8434d567c45e1b55583a5a4f`, and `51046a780fafe61bf10ed2dc9d2df4c1eeffa734`. Focused independent source tracing is assigned; presence of these commits is verified, but that audit is not yet complete. Any conflict between fixed reference behavior requires recorded evidence and independent review rather than an implicit reference substitution.

Runtime viewport-margin implementation `f3e06913376d3820acf5a9597597aec91f2a576b` passed 162 runtime tests and independent exact-source review. Picker integration remains pending. Edit-menu focus continuation is still a design gate: opening tokens must prevent stale delivery, and focus restoration/command execution must settle before output without an extra visible frame or handlers during pure rendering. No stage2 implementation or full edit-menu parity is accepted.

The bounded architecture diagnostic from independently reviewed source `70b7f9e6c196b57d1b13aee56ac6d6a17e6db388` is integrated without changing its four files. Its 39 regression guards pass. This supersedes earlier scanner-review findings only for those repaired lexical accounting defects; semantic ownership, production resources/generated inputs, external-consumer proof, and full product checks remain incomplete. The tool always reports `acceptance_complete = false`; a clean lexical scan cannot accept the migration. See [architecture diagnostic and limits](../audits/2026-10-20-acceptance-architecture.md).

Integration check against product source at `99b719d85` reports 84 lexical findings and five unresolved items, with zero reported read errors or unaccounted inputs. The diagnostic exits red and keeps overall acceptance false. The report is `/Users/donbeave/Projects/termrock-cli-evidence-20261020/architecture-diagnostic-integration/source-report.json`; 39 diagnostic guards and 28 paired-runner guards pass. This does not resolve any production finding.

The independently corrected [replay-origin audit](../audits/2026-10-20-replay-origin-current-source.md) covers exact product source `a848219fd858ed6352fcb403b2cc5d5953083e6f`; later Safe Mode change `c0e88b7ad` is outside that audit. It confirms remaining replay mechanisms in all four application trees. Subsequent independent real-CLI comparisons of `a848219` confirm exact Quick Open-open and tab-list-open checkpoints at 120×40, but Escape dismissal, tab activation, and all five tested 100×30 cases fail. WIP `366dc67` also fails both sizes. Complete source/binary identities and preserved receipts are in [the paired-run audit](../audits/2026-10-20-full-cli-runner.md). No full application is accepted.

Workflow delivery checkpoint: parity remote `99b719d85a0714c88484240dbc9184f110b518d4` and design remote `9b1c77b89e46b89f56632c841f1ade40371f03dc` were verified after normal fetch/rebase, complete-input generated-file comparison, plan, and actionlint checks. Main delivery remains CI-only PR #39 at `d1ecbfdfbe117ffad061f6db896c0a3c98b3c11d`; DCO remains blocked and required checks on that head are not established. These workflow checks do not attest Rust or product parity. The parity checkout still fails full formatting on the pre-existing Showcase Chrome cell table; no formatting or baseline artifacts were changed to hide it.

## Current CI timeout checkpoint (supersedes the workflow delivery status above)

Parity source `a5766d24783c2b268e727497fe6861b6479c71cc` uses Velnor Actions 0.1.29 (`e7b2f088715379c22be473fab896a43d50cc16fc`). Its latest observed run `39429669130` had Actionlint pass, Plan canceled after 5m09s, and all Rust jobs skipped. The run's `cancelled` conclusion does not prove GitHub's job timeout caused cancellation; downstream missing artifact/Required failures are not source failures. A distinct earlier Plan failure at `a848219fd858ed6352fcb403b2cc5d5953083e6f` was the 0.1.16 generated-file mismatch recorded in [the timeout audit](../audits/2026-10-20-velnor-timeout-parity.md).

Parity now explicitly configures `workflow.crate_timeout_minutes = 15`, matching the reviewed main-delivery upgrade config and raising all 44 generated Rust-job limits from five to fifteen minutes. Velnor 0.1.29 source confirms that field affects Rust capacity-phase and OpenTofu root jobs only; Plan remains a built-in five-minute control job with no override. This mitigates Rust-job budget risk but does not fix or explain the observed Plan cancellations. Fresh generation preview, plan (47 packages, 44 Rust jobs, 259 task IDs), and the CI-equivalent Actionlint command pass. Remote Rust validation remains pending; no skipped job is counted as a pass. DCO is not a protected required check. See [the timeout audit](../audits/2026-10-20-velnor-timeout-parity.md).


CI-only delivery review: source `4d0df7704dcb284d0c9726e78ea68bcca30838fc` independently cleared with full generated-tree comparison, unchanged task manifest, plan, and Actionlint. Main PR #39 remains blocked after its single rerun confirmed the fixed five-minute Plan timeout; all 43 Rust jobs were skipped. Design `59af1c4f760387be455bc79d44be75f4416a63f3` passed Plan/Actionlint but has a pre-existing Rust formatting failure. These statuses supersede earlier pending-status notes and any suggestion that optional DCO was the protected-check blocker. Details and exact job identities: [parity timeout audit](../audits/2026-10-20-velnor-timeout-parity.md). All four application acceptance statuses remain incomplete.

## Latest remote CI status (2026-10-20; supersedes the preceding status paragraph)

- Parity PR #38 remote head is `39aaf8d5e1dab5cf1c613c0bb7e76638d89d57af` and contains timeout source `4d0df7704dcb284d0c9726e78ea68bcca30838fc` (`c52aafc2f266842c1743151dd379c3a38ca990bc` is an ancestor). It uses Velnor 0.1.29 and a 15-minute Rust job timeout. Latest run [39438100451](https://github.com/tailrocks/terminal-components-claude/actions/runs/39438100451) was still in progress at observation; no required green result is established.
- Design remote head is `f2f6eb716c4a512be3dc9423742fa06e86879b32`; its CI-only changes use Velnor 0.1.29, 15-minute Rust jobs, and zizmor 1.31.1. Latest run [39438273472](https://github.com/tailrocks/terminal-components-claude/actions/runs/39438273472) was still in progress (Actionlint passed; Plan remained in progress). The preceding run [39435382085](https://github.com/tailrocks/terminal-components-claude/actions/runs/39435382085) passed Plan and Actionlint but failed Rust formatting in `Rust / xtask`; `Rust / junie-tui` was cancelled. Therefore design has no complete green required run.
- Main remains at `a97e351df736b576904bc450619ca347ea64d65b`; CI upgrade PR #39 is still separate at `a6929aeba7479d0b3a746f69d412a1ab701b5242` and has not been bypassed or merged. Latest run [39436987238](https://github.com/tailrocks/terminal-components-claude/actions/runs/39436987238) had Plan cancelled, Actionlint successful, all 43 Rust jobs skipped, and the required verdict failed. The same five-minute Plan cancellation is confirmed by three runs; no supported config override for Plan is recorded. These are workflow blockers, not successful Rust validation.
- The fixed baseline remains tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`, source `4169783ae883788186d161af09ba9c481bd84e5c`; tuiscotti remains pinned to `a1ea27f757d75d07b05499276cd932e9f8e067ad`. No baseline or test-tool reference moved. The design-range diff is CI configuration/generated workflow and audit documentation only; it contains no product, test, or reference-data edits.
- The earlier optional-DCO note is superseded: DCO is not the protected required blocker on these runs. The former “pending” language is superseded by the concrete run states above. Showcase, Jackin, TablePro, and Holla remain incomplete for product parity and library architecture acceptance; these CI results do not establish either gate.

## Branch consolidation (2026-10-20)

Delivery stays on `parity/termrock-visual-baseline` (PR #38). `design/interaction-polish` (PR #26) is reference-only: not merged, not modified, not closed. Its pinned review commit remains `f4867e9b5f8a08a3bb61ce80d644198a29f51984`. The remote branch tip is `f2f6eb716c4a512be3dc9423742fa06e86879b32` from other CI work; this record does not move it. `origin/main` remains `a97e351df736b576904bc450619ca347ea64d65b`. Tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542` and source `4169783ae883788186d161af09ba9c481bd84e5c` are unchanged.

Integrated: `be674ef8799c55bc3700cdd36f519cafebc95bed` from `parity/composition-slots-20261020` as `0cd5edb8856396bde1bbd2f8a5377940a3d50447`. It adds the typed picker layer slot. `termrock::slots` passed. Docs conflict kept the frozen-cli note and the new slot section.

Also integrated after cell checks: layout plans `8f7fc45a0` and `a39869753` (`termrock-layout` consumer test passed); session cursor `5e4eb27ec` and `f0360a467` (TablePro explain and explain-analyze 120×40 truecolor stayed at 0 cells); Junie ANSI-256 palette `737ecc836` and `95cdc7bdb` (79 `termrock-theme` lib tests passed); legacy item-row fixture `23a3fac94` (13 `item_row_columns` tests passed). Remote tip after that series is recorded in git, not here, until the push. Full four-app `VISUAL_FULL` runs are still not done. Do not merge to `main` before those two identical runs.

Already patch-equivalent on PR #38 (`git cherry` minus): `db31adf1e` session startup, `cf1ff9153` completion splicing.

Excluded, not merged:

- Design-history branches (`wip/shared-design-working-tree-20261020`, `wip/reach-pages-a`, `wip/reach-pages-a-2`, `wip/rapps-3-7-14`, `wip/s14-followups`, `wip/plan-s16`, `wip/v1-showcase`, `docs/interaction-reference-cleanup`, `ci/design-rust-timeout-20261020`, `ci/zizmor-design-20261020`, `parity/velnor-design-upgrade-20261020`). Each is 700+ commits ahead of a pre-termrock base. A merge would restore the monolith.
- `wip/termrock-reviewed-contract-integration-20261020` and `chore/rust-comment-cleanup`. Open parity or a 492-file comment strip, not a behavior fix.
- Preservation and rescue archives (`preservation/*`, `codex/rescue-*`, `codex/termrock-*`). They keep old snapshots and local history. They are not the delivery tree.

Not yet integrated. `git cherry` marks these `+` against `6bcdc4217`. They stay on their branches until a focused visual check shows they do not move baseline cells:

- `1ac6dfced` tablepro footer focus
- `7dab012ee` shared button state
- `94a1215e2` shared style regressions
- `d4832fe73`, `c1335e19c`, `f8e08fc29`, `56a8547a4` picker query editing
- `7e38a850a`, `bcb51e8c4`, `4ab81ae47` tablepro live picker contracts
- `17ad7f673`, `5b6a23121` session cursor policy
- dialog dismissal series `23f911902` through `1be6567ae`
- `8e9a7f59e`, `737ecc836`, `95cdc7bdb` theme palette
- `0638339b7`, `28cc24880`, `6b72d77a6` picker geometry

No other branch was deleted or closed. Product parity for showcase, jackin, tablepro, and holla remains incomplete.
## Latest theme reference-test diagnosis (2026-10-20)

- The isolated shared-theme palette branch `parity/theme-palette-baseline-20261020` is remotely verified at `95cdc7bdb111dc1b683a638b421dae718bb29643`. Its capability palette suite passes 9/9 after Junie ANSI-256 and Mono semantic corrections; see [palette audit](../audits/2026-10-20-junie-ansi256-palette.md).
- A separate test-reference correction branch updates the stale empty-title owner expectation from `Rgb(128, 128, 128)` to frozen `Rgb(138, 138, 138)`, then updates the aggregate hash only after independent projection of frozen palette rules across all 32 complete frames (5,376 cells). The old expected hash, original source hash, independent source revisions, projection artifacts, and limits are preserved in the [empty title reference audit](../audits/2026-10-20-empty-title-test-reference.md). Full-cell comparisons remain intact. This is a test-reference repair, not frozen Empty UX parity proof.
- The test-reference branch is not integrated into PR #38. This focused file result does not establish full application acceptance; remaining architecture, behavior, and paired application verification are unresolved.


Expanded independent-library check for palette cohort `636028fa130b2a0f841badb3ba93aa2330b907d6`: both nextest hash partitions across 19 Termrock crates ran 1,361 tests, with 1,352 passing and nine failing. Six failures are the existing Dialog layout assertions; the meter semantic-token, Note wrapping, and full-track scrollbar failures were independently reproduced on base `e597618d6576a725538a70d71250fd81c6599c28`. Partition exclusions are not additional skipped cases. Logs are under `/Users/donbeave/Projects/termrock-cli-evidence-20261020/pr38-palette-reviewed/library-partition-{1,2}.log`; base attribution is in `branch-consolidation/palette-failure-base-e597.log`. This is not a passing full-library gate or application parity proof.

## Runtime/query branch integration checkpoint

Reviewed runtime/query source `7c1a092ab7cfa37002f06e505cd4577f133750f4` was integrated with PR #38 source `072fc24d6dd4f53dbd544485174e39aff2c92f43` on the existing branch `parity/runtime-picker-allocation-20261020`. Merge base was `6bcdc42173232b53ed48b221ae8d099791283e39`. The PR source brings the reviewed composition plan, session caret-capability policy, branch ledger, and latest palette cohort. The `crates/termrock-session/src/session.rs` conflict was resolved to retain the runtime branch's prepared-frame prepare/draw/acknowledge loop, resize retry, and success-aware cursor-shape cleanup, together with frozen capability selection and hidden-cursor shape restoration. Its tests cover shape-write failure and flush failure. No application source, test approval, binary, tag, or tuiscotti pin changed in this runtime stack.

Exact merged-candidate checks: affected library nextest ran 434 tests, 428 passed, six existing Dialog layout assertions failed, and zero were skipped (`/Users/donbeave/Projects/termrock-cli-evidence-20261020/branch-consolidation/A-runtime-merged-native.log`). The independent public Picker query probe passed 16/16 (`A-runtime-merged-query.log`). The warmed 100k borrowed Picker rendered 38 visible items with zero allocations and bytes; `PERF_STRICT=1` passed the 1.5 ratio limit at 1.13 (`A-runtime-merged-perf.log`). Strict Clippy, scoped formatting, and `git diff --check` passed (`A-runtime-merged-clippy.log`, `A-runtime-merged-fmt.log`, `A-runtime-merged-diffcheck.log`). These results do not include real-CLI paired checks. Dialog dismissal/reopen remains blocked by the independent P1 generation-receipt finding; no dismissal-contract commits are included. Product parity and library acceptance remain incomplete.


## Reviewed runtime and terminal-profile consolidation (2026-10-20)

Runtime/query source `ce81e0adee987812788effd39d70acd242b0e95b` preserves the prepared Session loop, successful terminal-write cleanup, public query editing, stable row identity, and typed layer slots. Independent merge review passed 17 Session tests and 16 public query probes. Parent checks across 19 independent Termrock crates ran 1,453 tests: 1,445 passed and eight failed. The remaining failures are the six known Dialog layout assertions, meter semantic tokens, and full-track scrollbar expectation; the previously failing Note expectation is corrected. Four ignored subprocess-driver fixtures are listed separately in the test inventory; partition exclusions are not passing cases. Strict affected-package Clippy passed. The 100k Picker allocates zero bytes, but strict timing exceeded its 1.5 threshold under concurrent load and remains unproven. Logs: `/Users/donbeave/Projects/termrock-cli-evidence-20261020/branch-consolidation/parent-runtime-library-partition-{1,2}.log`, `parent-runtime-clippy.log`, and `parent-runtime-ignored-tests.json`.

Terminal-profile source `8207fbd009a3ced805e07ac734572cbd65078de7` was independently reviewed with 31 passing Python guard tests and a successful patch dry run against the frozen source. It propagates the same explicit TERM setting, including absence, through the actual Tuiscotti child environment for both phases. Frozen identities, immutable expected-data checks, ordered receipts, complete case accounting, and candidate isolation remain enforced. Live profile and application comparisons remain blocked by the unavailable approved baseline binaries; no replacement build identity is admitted. These integrations do not complete any application or library acceptance gate.


## Baseline rebuild diagnostic: not an admitted reference build (2026-10-20)

An independent diagnostic compared rebuilt, unadmitted frozen-source binaries with sealed historical frame copies through pinned Tuiscotti `baseline-check`. Four Showcase and ten TablePro checkpoints matched (14/14); the indexed eligible contexts contain no Jackin or Holla frames. Historical receipts preserve frame digests and approved-build metadata but do not bind the per-run baseline executable path and hash. Therefore executable identity remains unverified and the rebuilt binaries remain unadmitted; this result does not clear the live product gate. No source, frozen pin, approval, expected frame, or identity guard was changed. Evidence: `/Users/donbeave/Projects/termrock-cli-evidence-20261020/baseline-rebuild-diagnostic-expanded-20261020/diagnostic-report.json` (SHA-256 `14c38a03f58eb3a9694cd6e23c7830bc9eb45f6fc119c6d16268d06572ba109d`) and `receipt-audit.json` (SHA-256 `a8e16150643b8121ce75a126f3fdf8f4e478853b3b1e14249994a15cbeb8a648`).


## Independently reviewed composition and editor integration candidate (2026-10-21)

The integration working copy combines Dialog receipt source `1cf63f30890b6cfb1071ef4abf122189c1f88fc5`, its TablePro regression `27e1837a79d5035aaa1d2911d16b5921ca8ad643`, nested composition `bfdcea778262df118dbd4554a580830ef72c2c8c`, editor fill sizing `4e421879d3f6e482e321fe35e24602bb92e3e369`, and typed editor adapters `c102d1bac12f95a488d3ba7e60163ee20f36cdd7`. Each source change received independent review. The 19-library check at `1a073c03f` passed 1,489/1,489 tests with four ignored subprocess-driver fixtures; adapter review additionally passed six public consumer/slot checks. The shared cursor clipping guard resolves the inherited clipped-editor defect: the exact combined public probe passed 2/2, without an editor correction painter. Evidence: `branch-consolidation/parent-composition-editor-library-all.log`, `composition-independent-bfd.json`, `/tmp/editor-composition-independent-c102.log`, and `/private/tmp/termrock-code-editor-fill-combined-review.json`.

The actual TablePro dismissal test exposed an inherited retired-layer FocusOut diagnostic on Escape and outside MouseDown; independent controls reproduced it on `1be`, `ce81`, and `1cf`. Its runtime repair is under independent review, including the non-restoring publication/settle path. Do not classify the diagnostic as cleared by the initial Dialog receipt fix. Separately, external source `4bd17e2a0` adds a prohibited application Grid repaint, over-dims fully visible data, changes custom RGB conversion globally, and uses a disabled-control token for an active seam. Those findings require component-owned repair; valid frozen palette values 232/242/239 must remain. No full application or architecture acceptance is implied by library tests.

## Fresh-target and provenance checkpoint (2026-10-21; supersedes stale branch-tip summaries above)

The reviewed composition/editor cohort on this delivery worktree is based at source `a3ae6695c7fe44aad6a001005db6c6201dab1e12`. Its implementation inputs are Dialog receipt source `1cf63f30890b6cfb1071ef4abf122189c1f88fc5`, TablePro dismissal regression `27e1837a79d5035aaa1d2911d16b5921ca8ad643`, nested composition `bfdcea778262df118dbd4554a580830ef72c2c8c`, editor fill sizing `4e421879d3f6e482e321fe35e24602bb92e3e369`, and typed editor adapters `c102d1bac12f95a488d3ba7e60163ee20f36cdd7`; independent source reviews cleared the latter three composition/editor inputs. The integrated library gate at `1a073c03f` passed 1,489/1,489 tests with four ignored subprocess-driver fixtures. A fresh-target facade gate at the source tip passed 9/9 with zero skipped. This does not establish application parity or complete library acceptance.

A feature-enabled facade run against reused target `/private/tmp/termrock-pr38-style-delivery-target-20261020` failed unresolved composition exports. Investigation found the source paths and hashes match this worktree, Cargo metadata resolves the intended local crates, and old runtime/facade artifacts' dep-info omitted the newly required composition modules while a task-owned fresh target included them. The bounded command `CARGO_TARGET_DIR=/private/tmp/termrock-pr38-fresh-composition-target-20261021 cargo nextest run --locked -p termrock --features testing --no-fail-fast` passed 9/9, 0 skipped. Evidence and exact target path: `/Users/donbeave/Projects/termrock-cli-evidence-20261020/branch-consolidation/build-freshness-investigation-20261021.md` and `fresh-composition-nextest-20261021.log`. The stale-artifact condition explains the failure; the process/worktree that produced those artifacts is unknown, and MBX is not attributed. Use one stable target directory per worktree; no broad cleanup was used.

Frozen-source binary provenance is separately recorded in `/Users/donbeave/Projects/termrock-cli-evidence-20261021/provenance-build-smoke/cohort-report.json`, SHA-256 `ecc5ece1040c4dcdc0974d676635438ff69fc78724d08cc7968eeec0b10f7879`. It binds the frozen tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`, source `4169783ae883788186d161af09ba9c481bd84e5c`, tree and source digest, and Tuiscotti revision `a1ea27f757d75d07b05499276cd932e9f8e067ad`; all four binaries were separately built and their hashes recorded. The report explicitly says `acceptance_complete=false`, no paired compare was run, no reference decision changed, and the binaries were not admitted. It is provenance smoke evidence only, not a parity result.

The latest fetched PR #38 tip at this checkpoint is `a60fb0b7e6e698a63d273e61d4fb12d71cca8da9`; protected cohort destination currently points to `bfdcea778262df118dbd4554a580830ef72c2c8c`. These remote refs must be refreshed before delivery. All four application acceptance statuses remain incomplete; full paired visual/functional checks, CI, and architecture acceptance are outstanding.

## PR #38 merge candidate checkpoint (2026-10-21; source delivery blocked pending failure attribution)

The reviewed cohort and its evidence checkpoint are preserved through merge commit `80caf3df70947191aa4d05228ff16b5921dc234b`, which merges latest fetched PR #38 tip `a60fb0b7e6e698a63d273e61d4fb12d71cca8da9` into `68978504aa82d9f11e44b86c686d0bfefa461c15`. The merge completed without conflicts. This candidate has not been published to PR #38 while its TablePro gate failures await exact-base attribution. The PR remote remains at the fetched `a60fb0b7e6e698a63d273e61d4fb12d71cca8da9` until that check resolves.

At this exact source candidate, `cargo nextest run --locked -p termrock -p termrock-runtime --no-fail-fast` passed 338/338 with three ignored tests; the facade/editor composition coverage is included. The focused combined `termrock`, `termrock-runtime`, and `tablepro-ui` run compiled successfully after materializing sparse-checkout reference inputs, then ran 519 tests: 506 passed, 13 failed, 3 skipped. The failed tests were TablePro picker Ctrl-G and Ctrl-O cursor contracts; explain-order plan-cell visibility; destructive-generation target binding; 500×14 grid performance; two result-confirmation contracts; two row-action contracts; SQL-editor undo ownership; TablePro visual baseline; close-draft cancellation; and focused-query identity across tab reorder. Exact test identifiers and initial command output are retained in the execution report to the coordinator; independent base attribution is pending. A first attempt before adding `snapshots/tablepro` to this sparse worktree failed to compile because historical TablePro source includes those reference files; after `git sparse-checkout add snapshots/tablepro`, compilation succeeded. No tracked sources were altered by that materialization. Do not treat any of these TablePro results as pre-existing until an exact-base run establishes it.

The source cohort is independently reviewed and its public composition/editor tests pass. The merged candidate is preserved on `parity/nested-composition-consumer-20261020`; no PR #38 push is authorized until the 13 TablePro failures are proven to be unchanged from the relevant exact base or repaired and reverified. Application parity and complete architecture acceptance remain incomplete.

## Layout/block foundation integration (2026-10-21; supersedes preceding candidate tip)

After the prior checkpoint, latest PR #38 stage-one layout/split source `3fd30023b9983943c531d418977ab8016f5a9f22` was fetched and merged without conflict into the protected candidate. Exact merge candidate: `ce719aa60474e1a3100a035745cbcb40d2469a1f`. The changed files are shared `termrock-controls::Split`, `termrock-layout`, and facade exports; no application implementation was introduced. The focused gate `CARGO_TARGET_DIR=/private/tmp/termrock-pr38-merge-gate-20261021 cargo nextest run --locked -p termrock-layout -p termrock-controls -p termrock -p termrock-runtime --no-fail-fast` passed 473/473 with three ignored tests. This includes the facade/runtime composition suite and split/layout suites. It is a foundation gate, not full workspace or application parity validation.

The PR #38 remote now names `3fd30023b9983943c531d418977ab8016f5a9f22`. Candidate `ce719aa60474e1a3100a035745cbcb40d2469a1f` is being preserved on the nested composition protection branch by normal fast-forward only. The earlier merged TablePro package run at parent `80caf3df70947191aa4d05228ff16b5921dc234b` reported 13 failures and is not attributed to an exact base; a reviewer is performing isolated source-identity comparisons. Do not publish a candidate merge to PR #38 until that attribution or a repair/retest is complete. Later focused layout/facade/runtime success does not clear those TablePro results.

## TablePro exact-base failure attribution and delivery gate (2026-10-21; supersedes pending-attribution text above)

Independent base review on PR #38 source `a60fb0b7e6e698a63d273e61d4fb12d71cca8da9` established all 13 TablePro failures. The reviewed candidate on protected source `e0a811b3e2ead38934c51963b3149bf7f43bac56` was then run with the same command and complete all-target inventory: `cargo nextest run --locked -p tablepro-ui --all-targets --no-fail-fast`. Both inventories were 181 tests, 168 passed, 13 failed, zero skipped, and the exact failed test identifiers match. Base log: `/tmp/tablepro-base-a60-independent.log`; candidate log: `/private/tmp/termrock-tablepro-e0-all-targets-20261021.log`. Therefore the candidate adds no newly failing TablePro test names relative to a60; this is an accounted existing red gate, not a green TablePro gate. The candidate's broader 519-test combined run had 506 pass, those same 13 fail, and three ignored subprocess-driver fixtures.

The independent runtime/layout/controls/facade gate at the candidate source passed 473/473 with three ignored tests. With exact TablePro failure parity established and the reviewed shared cohort checks green, the candidate is eligible for normal integration into PR #38 subject to fetching its latest tip, merging without dropping concurrent changes, and rerunning those same focused gates on the resulting exact head. Product parity and complete library acceptance remain incomplete.

At delivery candidate `e0bbe5771ce8f05a53bb2e7cd93b130f6848995a`, after a fresh fetch confirmed PR #38 tip still `3fd30023b9983943c531d418977ab8016f5a9f22`, the layout/controls/facade/runtime gate again passed 473/473, 3 ignored. An exact `tablepro-ui --all-targets` rerun again produced 181 tests, 168 pass, 13 fail, 0 skipped, with the same 13 identifiers as the a60 reference run; full candidate output is `/private/tmp/termrock-tablepro-e0bbe-all-targets-20261021.log`. The documented result is therefore reproducible at the delivery head. This integration is ready for normal PR #38 fast-forward; its TablePro failures remain visible and explicitly attributed, not suppressed or reported green.

## Generic RGB palette boundary correction (2026-10-21)

Cherry-picked independently reviewed palette source `7d5246b1b487dccfca76648bf13be13e2577026d` as `68ebfcaf4`; only the generic RGB downgrade and capability-palette assertion/audit are included. Exact RGB exceptions were removed from generic `downgrade_color`; baseline-authored semantic roles keep ANSI-256 values 232 (on-accent), 242 (strong border), and 239 (disabled foreground) in the semantic capability palette. The independent reviewer confirmed 123 `termrock-theme` tests pass and exact TablePro failures remain the same 13 against base `5c0075a`; no fade/grid visual approval was imported. Source audit: `docs/audits/2026-10-21-capability-palette-authorship.md`.

The full-workspace formatter still exits nonzero on existing cell-table formatting in `crates/showcase-demos/src/pages/chrome.rs` and application formatting in `crates/tablepro-ui/src/app.rs`. The cherry-picked theme files were individually formatted and pass scoped rustfmt; no unrelated formatting was applied. Exact full-format diagnostics are captured at `/private/tmp/termrock-fmt-after-palette-7d-20261021.log`. Theme clippy passed with `-D warnings`; theme tests passed 123/123 with zero skips before the final whitespace-only normalization, and are rerun on the delivery head before push. Full four-app parity remains unproven.

Delivery-head verification for the palette increment (`68ebfcaf4` plus the scoped formatter correction and this record) passed `termrock-theme` 123/123, theme Clippy across all targets with `-D warnings`, scoped rustfmt on both changed Rust files, and `git diff --check`. `tablepro-ui --all-targets` remained 181 run, 168 pass, the same 13 base-attributed failures, zero skipped (log `/private/tmp/termrock-tablepro-bb91-all-targets-20261021.log`). Layout/controls/facade/runtime remained 473/473 with three ignored tests. Workspace `cargo fmt --all -- --check` still reports only the unrelated Showcase chrome cell table and TablePro app formatting paths listed in the formatter log above.

After the whitespace normalization, the final palette candidate `5e52be1b63dd449d8388b2f021030ae617d930c9` passed theme nextest 123/123, theme Clippy (`--all-targets -- -D warnings`), scoped rustfmt for `downgrade.rs` and `capability_palettes.rs`, and the 473-test layout/controls/facade/runtime gate (473 passed, three ignored). Its all-target TablePro gate ran 181 tests (168 passed, 13 failed, zero skipped), with the exact same 13 identifiers as frozen PR #38 base `a60fb0b7e6e698a63d273e61d4fb12d71cca8da9`; output: `/private/tmp/termrock-tablepro-5e52-all-targets-20261021.log`. The source review and frozen palette evidence are in `docs/audits/2026-10-21-capability-palette-authorship.md`. The whole-workspace format check remains red for unrelated Showcase chrome and TablePro application paths. No application fade or full visual approval is claimed.


## Reviewed integrated task branch cleanup (2026-10-21)

Eight reviewed task branches were deleted atomically after final exact-tip, literal-ancestor, GitHub protection, and open-PR rechecks against `a3ea90efc14181a22d06a95231a61ad4faee1aec`. [Exact branch tips and evidence](../audits/2026-10-21-integrated-task-branch-cleanup.json) preserve their identities. 86 remote heads remain. Active nested-composition and palette preservation branches, unreviewed changes, archives, main, and the read-only design reference remain intact. No local files or branches were removed. Application parity and architecture acceptance remain incomplete.


## Authored seam profiles and external query-results review (2026-10-21)

The selective seam integration combines independently reviewed source commits `9bd668c40d9861f2b4bcac7c282545fba5e8b60d`, `f61c76a35d6dee4ed88f90fbd075a3c03e7147d1`, `bd4f0411fd890d18c39454f652233179fd62133f`, and `b26f548722fbb1d5f036d978a1519cfb0fa833ad` on PR source `9f19616d417c1323f1fa7a3413612116f0f41de0`. Blocked Grid fade ancestry is excluded. The shared recipe API preserves public ColorTokens/Role shapes, explicit channel overrides, and ordinary custom-color conversion. Authored capability channels retain provenance through the actual cached Ui binding; frozen Mono seam colors remain DarkGray/Gray/White. Earlier intermediate Black/White and value-equality claims are superseded. [Independent review and red/green probes](../audits/2026-10-21-seam-profile-independent-review.json) document the exact source and limits. The combined theme/runtime/controls all-target check passed 568 tests with three existing ignored subprocess fixtures; evidence is `branch-consolidation/parent-seam-selective-native.log`.

External PR commits `9b4aae59b5c006e64b3791fa15804f6411633e4d` and `9f19616d417c1323f1fa7a3413612116f0f41de0` are preserved. Their live executor-derived result data and historical replay deletion are useful, but application-owned editor, toolbar, seam and tab painters, invisible editor registration, Unicode/multiline handling, and an unverified shortcut remain unresolved. The [external implementation review](../audits/2026-10-21-external-query-results-review.json) records exact findings and replacement owners. Retain baseline Mono Run appearance through a real shared Button; a rest-state frame does not prove its interaction states. All four application product gates and the complete library architecture gate remain incomplete.

Strict affected-package Clippy (`-D warnings`), package formatting, and `git diff --check` passed on this selective seam candidate. Full workspace formatting, CI, architecture, and paired application acceptance remain required; this scoped result does not replace them.


## Failed baseline rebuild reproducibility and preserved review evidence (2026-10-21)

Independent A2/B2 builds remain diagnostic and unadmitted. All four application executable hashes differ at both build stages, and A2 preparation used timeout 220 instead of the reviewed 225. Both complete cohorts and raw receipts remain preserved; no executable bytes, expected frames, frozen revisions, admission guards, or old reference decisions changed. The [committed comparison and evidence index](../audits/2026-10-21-baseline-build-diagnostics/evidence-index.json) preserves exact report digests and external locations. Source and dependency metadata match after the declared root substitution; the alternative metadata digest was a separately identified JSON serialization. Every differing final executable byte is attributed to N_OSO debug object paths/random suffixes or the corresponding valid code-signature page hashes. All 17 named sections, load commands, UUIDs, and sizes match; this does not establish functional or visual parity and does not reverse the failed exact-hash result. Pinned compiler source makes invocation suffixes random when incremental compilation is enabled. The fixed-path-only proposal is insufficient; a new nonincremental diagnostic recipe requires separate review before execution.

The [frozen completion cursor contract](../audits/2026-10-21-frozen-completion-cursor-contract.json) records 50 reference frames with a visible steady bar while completion is open. Hiding that intermediate cursor is not baseline parity. The real shared editor migration and narrowly scoped nonmodal owner-cursor policy remain under implementation review. [External completion and scrolled-orders review](../audits/2026-10-21-external-completion-scrolled-review.json) retains useful live catalog/replay deletion work, but records exact SQL/viewport completion selectors, private popup/editor painting, remaining Grid afterpainting, and combined-scroll risks. A real PageDown+End regression now reproduces the row-offset error; its replacement remains unfinished. Showcase, Jackin, TablePro, and Holla product parity and the complete library architecture gate remain incomplete.


## History replacement review and isolated migration blockers (2026-10-21)

PR #38 was verified at `adc6d0e3f668818f6f8321310452db6806c123cc` before this documentation increment. The [independent review of external source 5902](../audits/2026-10-21-external-history-review.json) preserves six findings against exact commit `5902a59053e4af4601d880c8419cb180adc5f661`. Deleting the historical History ANSI replay is useful. Its replacement still implements search, list, detail editor, split seam and buttons in application painting; its History update branch has no corresponding controls and forces focus. Seeded history data also differs from the frozen source, and repeated History opening creates duplicate tabs. These require real shared search/master-detail components and application-owned restoration of frozen data/routes. The new History painter uses fixed geometry at all sizes; it is not itself an exact-viewport replay selector. Inherited completion and query repaint findings are separately attributed in the audit. No source correction or application acceptance is claimed by this review.

The protected real-editor candidate `d46861777a440af6d6d9732636b2c592986f842b` remains excluded from integration. Independent review found ignored matched emphasis, initial editing enabled earlier than the frozen source, and introduced all-target integration compilation failures caused by old draft-state calls. Its shared packages passed 540 tests with three existing ignored subprocess fixtures, but that cannot clear the application compilation failure. Exact base `9e013c038fb45ed040ec3ef0a9bcf34ff4dec3e4` compiles and runs 181 TablePro tests: 170 passed, 11 failed, zero skipped. The author is repairing all three findings without removing tests. Detailed external review evidence is `termrock-cli-evidence-20261020/branch-consolidation/editor-review-d468.json`.

The protected Grid painter-removal candidate `a5ac5cf14c5ffcc2d28d62fadb46bbacddf0bdb9` on `wip/tablepro-grid-scroll-embedded-input-20261021` fixes a reproduced PageDown+End stale-row repaint but adds an inline-editor regression to the same 11 pre-existing failures. Integration is blocked. Frozen inline editing keeps the cell width fixed, starts text at the cell origin and reserves one trailing cursor cell. A bounded shared TextInput embedded presentation is under API review; draft-dependent column growth and application correction painting are rejected. All four product gates, full workspace checks, CI and complete library architecture acceptance remain incomplete.


## Integrated CI protection branch cleanup (2026-10-21)

Deleted only `wip/ci-testing-features-20261021` at `9576b24e96f383590b9b621bfe43ff0154577acd` after writer release, literal ancestry, no open PR, unprotected status, exact-tip lease and remote absence checks. The independently reviewed CI changes remain in PR #38. [Cleanup evidence](../audits/2026-10-21-integrated-ci-branch-cleanup.json) records the integration head. Active Grid/editor preservation branches, the read-only design reference and main remain untouched.


## Superseded main-only CI PR consolidated (2026-10-21)

Closed PR #39 after independent review and direct object comparison established no unique required Velnor pin, tool-lock, release-manifest or generated-helper delta beyond PR #38. Its schema-1 configuration and main-specific generated task graph are excluded because they would omit PR #38 testing configuration. Deleted only its unprotected task branch `ci/velnor-0.1.29-main-20261020` at `a6929aeba7479d0b3a746f69d412a1ab701b5242` after checking the closed PR, no dependent open PRs, exact remote tip and remote absence. Main and PR #26 are unchanged. The original branch inventory is preserved with an additive superseding decision in the [branch ledger](../audits/2026-10-20-branch-consolidation.json).

The [upstream Velnor review](../audits/2026-10-21-velnor-upstream-review.md) records latest stable 0.1.29 and upstream main source, the missing Plan timeout setting and the doctest policy conflict. A backward-compatible upstream Plan timeout setting is being implemented separately; it is not released or adopted by Termrock. Doctest coverage must be migrated to executable nextest coverage before disabling any target doctests. CI and application acceptance remain incomplete.


## Independent nonincremental baseline reproduction (2026-10-21)

The [serial-build evidence index](../audits/2026-10-21-serial-baseline-reproduction.json) records independently repeated recipe `849900e9c848d892824bd2b6636a17ba35d57c9be57859abf136c42f7f314bac`. All four complete final executable hashes match between RUN1 and RUN2 without byte normalization; corresponding stage1 hashes also match. Source, raw metadata, declared stage inputs and compiler argument multisets match. Builds use identical physical source/target paths, incremental compilation disabled, root-package codegen units 256 and the frozen dependency optimization profile. This resolves that bounded diagnostic question; it does not make the earlier differently configured A2/B2 mismatch disappear or prove path-independent/hermetic reproducibility.

RUN1's first prepare-live attempt failed before source copy/build because the historical decision-loader file was absent; the failed receipt and successful retry remain preserved. Its full target tree was not archived. RUN2 preserves a complete tar with 34,589 original filesystem entries independently verified; archive-only AppleDouble metadata was separately verified after preserving the first verifier failure. Neither limitation is silently converted to complete pristine-sequence provenance. No executable is admitted yet. The next bounded change is a strict additive cohort record/loader and independent negative review, followed by full frozen capture/replay and candidate comparison. The old v1 tuple exception remains unchanged and cannot be generalized to the new executable hashes.

Coordination correction: activating an independent reviewer briefly exceeded the five-child concurrency ceiling. The coordinator immediately interrupted that reviewer before work proceeded and resumed it only after another child completed. Future activations require a fresh live-agent count. No product or verification conclusion is based on that brief scheduling error. All application and architecture acceptance gates remain incomplete.


## Corrected frozen editor focus-entry diagnosis (2026-10-21)

The [superseding independent source review](../audits/2026-10-21-frozen-query-focus-entry-review.json) corrects the earlier constructor-only inference in the d468 review. Frozen QueryTab/CodeEditor construction starts with editing disabled, but the default edit-entry policy is Immediate: focus rendering and first-key handling enter editing, and Ctrl+T focuses the new editor. Therefore a test requiring an extra Enter after Ctrl+T would change the frozen flow and must not replace existing typing scenarios. The original report remains historical evidence; its inference about default focused browse behavior is superseded.

The reusable replacement must implement configurable focus entry in the shared update/prepared focus lifecycle, preserving effect-free measurement and drawing. It must preserve default immediate entry, explicit browse override, first-character handling, cursor placement, blur/refocus, completion/modal coverage and resize behavior. Application constructors must not substitute for shared lifecycle mechanics. The architect approved an additive App policy method and private per-editor override instead of adding a mandatory public Theme field. This implementation remains under review.

New external PR #38 commit `cf9133c21a73b4bfe73a02e713a243614126a62a` changes the new-connection form and deletes a historical replay module. Its retained behavior and complete library ownership still require independent review; deletion alone is not acceptance. No reference revision or expected frame changed.


## New-connection replacement introduces ten additional test failures (2026-10-21)

Parent validation on exact PR #38 source `2a741b37878002d6f6cb32e8d624764a56635352` ran `cargo nextest run --locked -p tablepro-ui --all-targets --no-fail-fast`: 181 tests, 160 passed, 21 failed, zero skipped. The exact earlier base `5902a59053e4af4601d880c8419cb180adc5f661` had 181 tests, 170 passed, 11 failed, zero skipped. All eleven earlier failure names remain; ten additional names affect fresh-name replacement, form radio interactions and clipping/styles, Unicode footer handling, connection-form performance and guarded Save/Connect replacement. [Exact failure names and hashed raw logs](../audits/2026-10-21-external-form-test-regressions.json) preserve attribution. External `cf9133c21a73b4bfe73a02e713a243614126a62a` is the sole production change between those integrated checkpoints. These are newly observed regressions, not accepted pre-existing failures or a green replacement. Independent source review and component-owned repair are required.

Separately, independent review of protected embedded-input candidate `59e8cffb302c4f4734ecd640c671725b1a054069` passed 209 native fields/Grid tests, but a new external-consumer probe exposed invalid inline editing retaining an Accent underline instead of the frozen error underline. That candidate remains excluded from PR #38 pending shared error-state repair and a regression. Its eleven-base-failure attribution does not clear the newer form-source regressions.


## Diagnostic executable-cohort schema integrated (2026-10-21)

Integrated independently reviewed source `cf346dd4ba85b165cefcbec3bbeeccab21fb2a23` by normal merge. The additive registry `tools/frozen_cli_build_cohorts.json` and strict loader bind the independently reproduced four final executables, stage1 hashes, frozen source/tool identities and archived evidence, with explicit RUN1 limitations. [Independent adversarial review](../audits/2026-10-21-baseline-cohort-schema-review.json) confirms rejection of numeric boolean claims, forged cohorts, duplicate JSON keys, registry replacement/in-place races and binary mutation. The parent reran all 48 Python tests, Python compilation and whitespace validation successfully on the merge. The old v1 reference document SHA remains `2a43fb118c268300c8bbd6439ea11bb9a24ffff03dc3c9f4601893eee26a096b`.

This integration is diagnostic only. Capture eligibility is rejected, no paired-runner path uses the registry yet, no production code or reference frames changed, and the old tuple-specific exception remains unchanged. The next increments require separately reviewed executable-cohort adoption and resumable plan/advance/finalize commands. Current live-compare starts a timestamped run and loops every chunk inside one command; running the complete inventory that way violates the bounded-command rule. A resumed run must bind immutable complete inventory, phase/chunk order, candidate/source/environment/cohort identities, securely verified process/checkpoint/close receipts and expected-frame hashes. Finalization must reject incomplete, failed, duplicate, missing or zero-checkpoint coverage and require all twenty ordinary checks. No application acceptance follows from this schema gate.


## Current package gate, form reference conflicts and upstream CI dependency (2026-10-21)

Parent validation at `ecb271501618508aa6fadc9c9b75c178fcaaa41e` runs 181 all-target TablePro tests: 161 passed, 20 failed, zero skipped. [Exact results and raw-log hash](../audits/2026-10-21-tablepro-ecb271-base-results.json) supersede the earlier 21-failure count for this source. The external Advanced-form replacement clears the fresh-name test; no additional failure names appear. This remains a red package gate. Later external `6375fcee053d7859279ce436efde98c258ae649f` replaces a hidden-explorer replay; its source ownership and behavior still need independent review.

The [independent connection-form source review](../audits/2026-10-21-external-connection-form-review.json) distinguishes real private-painter/input/focus/clipping/password defects from test assumptions conflicting with frozen construction and glyphs. Frozen new connections are blank rather than cloned Production; environment labels are lowercase and the off-radio glyph is `( )`. These findings do not authorize silently changing tests or reference artifacts. Repeatable frozen CLI evidence and documented reference decisions must preserve coverage and functional assertions. The new shared Form prerequisite is pure data-aware measurement backed by the existing placement planner; sections, live controls and actual TablePro adoption follow separately.

Independent MasterDetail review of protected source `e4eb0726ec61f5b17160d81f2e1aad6967bc8023` reproduces three failures despite native suites passing: nested narrow drawer Escape uses terminal rather than block bounds; Escape misses a real focused detail descendant; summary height follows intrinsic content instead of the exact clamped policy. It remains unintegrated. The Grid follow-up at `bddfbe63f5cb8fa77e363fb233ac5fe8d51c4907` is under independent rereview; its source-base TablePro comparison retains the same twenty failures. Neither library workstream is accepted from native green tests alone.

Upstream [Velnor draft PR #249](https://github.com/tailrocks/velnor-new/pull/249) provides the reviewed optional Plan timeout setting. Its head is `e5c69f476a768da5cce4904d8dc58c008e90c476`; the implementation was independently reviewed at the preceding code commit, with a final documentation-only compatibility note. Omitted config remains five minutes and full generated YAML changes only the requested Plan scalar. The Rust WorkflowConfig field addition is explicitly source-breaking for downstream struct literals. Focused checks pass; latest observed image CI was running and DCO required action. No merge, release or Termrock adoption occurred. Termrock keeps released Velnor 0.1.29; a fifteen-minute trial has not yet demonstrated a successful Plan checkout.


## Push-lock incident and recovery (2026-10-21)

The Form worker successfully acquired the shared push lock. A MasterDetail worker failed to acquire it but, because its shell did not stop on that failure, overwrote the owner metadata and proceeded with a separate WIP push. Both completed remote tips were verified: Form `eca0aee9f119be3ac238dbdc0d6f6a75594c5e35` and MasterDetail `856059ad5950cc6f266beb3c4273bbf6f5afa93d`. Neither push forced a shared ref. The original Form lock owner independently traced its successful acquisition, verified the known overwritten metadata and both completed tips, recorded the timestamp limitation, and released its own original lock at 2026-10-20T19:27:30Z. The coordinator did not remove an unknown other owner's recent lock. Future acquisition failures must abort the push cycle; ownership must be checked before release. This scheduling failure is recorded rather than hidden and does not provide any product or review acceptance.


## Pure data-aware Form measurement (2026-10-21)

Integrated independently reviewed Form source `eca0aee9f119be3ac238dbdc0d6f6a75594c5e35` by normal merge at `cf46299272a992c7aeb445152d9b7eec7beadd12`. The additive `measure_with_data` uses the existing placement planner; legacy measurement remains unchanged. The real regression measured 24 rows before the fix against 27 drawn rows. The independent public consumer covers dynamic radio choices, visibility, groups, constraints, empty data and effect-free measurement. [Independent evidence](../audits/2026-10-21-form-data-measure-review.json) records limitations. Parent verification on the merge passed all 52 Form tests, strict all-target Form Clippy, scoped formatting and whitespace validation. Sections, lane composition, actual application migration and full parity remain incomplete.


## Independently reviewed cohort adapter and fresh candidate builds (2026-10-21)

Integrated protected tool source `979acbe0f3b7125d51f8f5cae2c25e786adbd544`. The adapter accepts only pinned registry paths and rejects aliasing, replacement and cached-target executable substitution. Candidate compilation now requires a fresh exclusively created target; existing files and directories are rejected before Cargo without cleanup. [Independent review](../audits/2026-10-21-baseline-cohort-adapter-review.json) reproduces actual cached-dependency substitution rejection and fresh pinned-Cargo compilation, plus all 51 Python tests and prior identity/race controls. No compiler hermeticity is claimed. The production cohort review registry remains empty: no capture approval, historical-exception transfer or application parity acceptance follows. Resumable full-inventory lifecycle adoption remains required.

Parent Form measurement delivery was verified remotely at `ee6a65743c532b3ae0cbe57265f503dff5f61e2e` after fetching and merging concurrent commits and rerunning 52 Form tests, strict Clippy, formatting and whitespace checks on the exact pushed head. New external commits `009bbd98a7a42250760aa4ce29b309e8beb0fe5c` and `9268b190e` still need independent ownership and theme-contract review; their presence is not acceptance.


## Independent review blocks protected candidates (2026-10-21)

The [editor review](../audits/2026-10-21-editor-8153-review.json) rejects `8153f3f85ba1b3c494bfcbfa1c7d8ce95753e847`: completion closure steals focus back from the real next control and re-enters editing; changing the native assertion to accept that result is not a repair. The independent public regression requires `(NEXT, false)`. Default unmatched Unicode labels also incorrectly become bold; explicit consumer bold must remain supported. Both findings are assigned to a separate repair implementer, with review preserved. The historical ItemRow fingerprint failure remains separately attributed.

The [MasterDetail review](../audits/2026-10-21-master-detail-b460-review.json) verifies the original nested-bounds and detail-descendant fixes at `b46010bd95300e04d2903895fa6def76a4a7f368`, but exposes a scoped binding that declines Escape suppressing outer/global fallback. Runtime repair is assigned to its existing exclusive writer. The old summary probe counted the editor footer incorrectly; its original failed output is retained, and a separate precise body-plus-footer probe passes. This documented diagnosis does not turn the true fallback failure green.

Protected Grid source `26e4d54879fcd9b777df701bb65881f6f11a09a4` replaces synthetic color inference with private canonical Clear provenance and passes 344 native tests plus four original public probes. A fifth independent public probe still finds a winning local underline Clear leaves a prepainted Yellow underline. That finding is assigned to the same exclusive component writer. None of these three candidates is integrated or accepted based on their native green suites.


## Further contract review and integration conflicts (2026-10-21)

The [Grid review](../audits/2026-10-21-grid-ecd8-review.json) clears the bounded fields/Grid/theme series at `ecd8a8194af79787562d120305c90a6b58996da8`: all five unchanged independent probes and 345 native tests pass, with strict Clippy and scoped formatting. Parent integration exposed an overlapping new application correction painter from concurrent `009bbd98a7a42250760aa4ce29b309e8beb0fe5c`. The attempted merge was aborted cleanly rather than restoring deleted inline/scrolled painters or silently accepting `paint_unscrolled_cursor_row`. The exclusive Grid writer is replacing that new painter through shared contracts before integration. Concurrent `ddc3dd3fef545444fdda48af75ea040c498b2efc` deletes a maximized replay but adds application-owned query-card/footer painting; architecture review remains required. Neither replay deletion is library acceptance.

The [MasterDetail follow-up](../audits/2026-10-21-master-detail-fa855-review.json) confirms decline-to-global and decline-to-outer routing repairs at `fa85598f34ce4f3dd9bc8b4525af3fc301418260`, but finds stale outer bindings firing after an inner callback opens a modal, and unbounded pending draw-side diagnostic storage. Both are assigned to the runtime owner; no promotion occurred.

The [Form lanes review](../audits/2026-10-21-form-lanes-451789-review.json) rejects `451789d519caf396c16818ed50f571271255ccbe` despite 57 native passes: both independent public probes fail because partial scrolling repeats hidden radio content and overlaps the next field, while Fill ignores allowed trailing spacers. The update planner also lacks its real Form logical area. The Form owner is repairing shared layout/scroll contracts and adding real input/resize coverage; the existing runtime diagnostic prerequisite separately remains under repair.

Inventory count correction: the pinned inventory contains 28,895 original visual scenarios plus 174 disjoint extensions, totaling 29,069 visual scenarios and twenty ordinary checks (29,089 tests). Earlier shorthand treating 28,895 as the extension-inclusive total is superseded; no case was added, removed or reclassified. Independent cohort review now permits fresh baseline capture and baseline replay only; registry mapping and resumable lifecycle implementation require separate verification. Candidate comparison, checkpoint approval, the historical tuple exception and product acceptance remain blocked.


## Current parent package gate remains red (2026-10-21)

Parent exact `124a61a035eeff34767edc8d9aa7e61f3007ebf8` runs 180 all-target TablePro tests: 159 pass, 21 fail, zero skipped. [Hashed log and name-set comparison](../audits/2026-10-21-tablepro-124a61-base-results.json) show one additional failure, `unfocused_grid_cursor_does_not_style_the_cursor_row`, assigned to the shared Grid owner. The hidden-explorer check was renamed; the old maximized replay-module frame check was deleted with the replay module. Actual maximized real-CLI coverage remains a required explicit verification rather than assuming the smaller native inventory is equivalent. None of the existing failed expectations was changed by this coordinator. All four applications and the complete architecture gate remain incomplete.


## Live editor integration and remaining gates (2026-10-21)

Integrated independently cleared editor source `759d0d411ed9148e3d04f700bcc81d7b90b97a4a` by resolving five application conflicts. The resolution uses the real CompletingCodeEditor, removes ghost empty-editor registration and completion replacement painters, retains current semantic blank-query data and new form/maximized domain content, and does not restore removed replay helpers. [Original independent clearance](../audits/2026-10-21-editor-759d-review.json) covers twenty unchanged public probes and canonical style precedence. Exact resolved-merge review remains pending.

[Parent integration checks](../audits/2026-10-21-editor-integration-checks.json) run 194 TablePro/domain tests: 180 pass, fourteen fail, zero skipped, with no candidate-only failure names against the current base. Shared tests run 744: 743 pass, one unchanged historical ItemRow fingerprint fails, and three fixtures remain ignored. Strict Clippy for the five affected shared packages and formatting for all seven affected packages pass. Expanded application/domain strict Clippy exposes the pre-existing 169-line seeded-data function; its complete function bytes match the exact base. This requires a focused cleanup, not a blanket lint allowance. Full package, CI and product gates remain red.

The [concurrent-increment audit](../audits/2026-10-21-concurrent-increments-85b1-review.json) assigns retained semantic-role loss and generic RGB conversion regressions to the runtime/theme owners. Focus-sensitive maximized pane selection and tab/footer focus remain required workspace contract work. No application acceptance follows from this editor increment.


## Capture-only cohort mapping and editor merge review (2026-10-21)

Integrated independently reviewed metadata source `de156136f9e8e642809c99a35e7cee4259e0b047`. The [exact review](../audits/2026-10-21-cohort-capture-metadata-de156-review.json) passes 53 Python tests, nineteen adversarial probes and all four sealed binary checks. Parent reruns all 53 tests, Python compilation, whitespace validation and actual four-binary capture-eligibility verification successfully. The pinned registry now permits only future fresh baseline capture and replay; candidate comparison, checkpoint approval and product acceptance remain false. No application was launched by this increment. The old v1 reference bytes and exception scope remain unchanged. Fresh-plan preparation and resumable advance/finalize remain separately reviewed work.

The [independent integration review](../audits/2026-10-21-editor-7efe-integration-review.json) clears all five conflict resolutions at pushed editor merge `7efe824fefeab1b26794b2fec9f749214d81041a`, with twenty external probes and four real TablePro editor flows passing. The earlier pending-review flag is historical and superseded by that exact review. Full parity remains incomplete. The expanded Clippy blocker is correctly named `History::seeded` in model.rs; the earlier ResultGrid name was an attribution typo, with the emitted file, complete function hash and pre-existing-source proof unchanged. Its focused domain cleanup is assigned.


## Shared MasterDetail/runtime integration (2026-10-21)

Integrated protected source `e0026278649d215d88ca3231ba4720f7c8ae16e7` by automatic merge at `1e978a2d0faec34f7c84312ed8144cad47656225`. It adds generic named MasterDetail slots and measured responsive policy, prepared nested bounds, real scoped detail focus, leaf-first Escape routing with current-scope revalidation, and bounded component-author diagnostics. [Final runtime review](../audits/2026-10-21-master-detail-bc99-review.json) clears prior routing and diagnostic findings. The [alignment review](../audits/2026-10-21-right-alignment-d8dba-review.json) clears preserved-channel semantic metadata under dimming. The two cursor assertions use the existing production cursor_position API, preserving assertions and removing the default-feature compile failure; [decision and proof](../audits/2026-10-21-default-feature-cursor-review.json) require no skipped test or new API.

[Independent merged review](../audits/2026-10-21-runtime-integration-1e978-review.json) verifies editor lifecycle/owned cursor and runtime scope contracts together, default facade 6/6, feature-enabled facade 20/20 including all seven new MasterDetail consumers, and exact generated CI arguments. [Parent checks](../audits/2026-10-21-master-runtime-integration-checks.json) run 913 shared tests: 912 pass, one unchanged ItemRow fingerprint fails, three fixtures are ignored; 194 TablePro/domain tests remain 180 passing and the identical fourteen failures. Shared/default facade strict Clippy and affected formatting pass. This is bounded integration, not a green complete library or product gate; real application adoption and four-app paired verification remain required.

Velnor Actions 0.1.30 has now been migrated through its official configuration migration, pinned in mise.toml/mise.lock, and used to regenerate the workflow set without hand edits. The exact release, migration rationale, generator evidence, task inventory and remaining CI blockers are recorded in the [Velnor 0.1.30 audit](../audits/2026-10-21-velnor-0.1.30-upgrade.md). The remote Plan checkout timeout and generated doctest-policy conflict remain unresolved. Concurrent Jackin `871f12c87` replaces a 6,009-line help replay with real HelpOverlay data plus application placement/dimming helpers; independent ownership and exact baseline checks remain required.


## Reviewed seeded-data lint cleanup (2026-10-21)

Integrated protected source `c9357a9efbaaf995eee00390183d3c82631caa03`. The [independent review](../audits/2026-10-21-history-seed-c935-independent-review.json) verifies all 402 seed-data Rust tokens, string contents and order, and unchanged constructor mapping. The private constant extraction removes the `History::seeded` length violation without a lint allowance or domain behavior change. Parent integration runs all twelve domain tests with zero skipped, strict all-target domain Clippy, package formatting and whitespace checks successfully. This preserves current candidate data; frozen history-data parity remains unresolved. The expanded application lint errors, existing product failures and full acceptance gates remain required work.


## Form lane composition and source-index Radio windows (2026-10-21)

Integrated protected source `c346aff507351d1490f55caa3a6de7c8347a6853` at `768b2f893`. The [independent source review](../audits/2026-10-21-form-lanes-c346-review.json) clears borrowed lane descriptors, final-data validation, trailing Spacer reservation, original-index Radio windows and real pointer reachability. Parent Controls/Forms all-target nextest runs 176 tests, all pass with zero skipped; strict affected Clippy, formatting and whitespace checks pass. The only merge conflict duplicates an older diagnostic probe; retaining the current bounded-store probe and scoped-input regressions leaves runtime source unchanged from the parent. Exact merged-source review and application regression checks are recorded separately when available.

This is bounded shared infrastructure, not complete Form clipping or an application migration. [The staged TextArea plan](../audits/2026-10-21-textarea-parent-window-api-plan.json) requires a typed prepared vertical window, shared ScrollRegion source coordinates, and one TextArea mapping for drawing, input, cursor and menu anchors. Partially clipped TextArea handling, menu routing and reference proof remain unresolved. No private application form painter is accepted by adding these APIs; its replacement remains required.

Parent [merged Form checks](../audits/2026-10-21-form-lanes-integration-checks.json) also run 344 runtime tests successfully, with three existing ignored fixtures. TablePro/domain runs 194 tests: 181 pass, thirteen fail, none skipped. No candidate-only failure name appears; the former form draft-commit failure is absent in this run. The initial strict fourteen-name comparison exits nonzero because the failure count decreased; that observation is preserved and does not establish frozen parity or waive the remaining failures.

[Exact Form merge review](../audits/2026-10-21-form-768b-integration-review.json) clears `768b2f893c077be023e91bc606ecd8d23142e803`: the same five external public probes pass without modification, Forms and Radio source match reviewed c346, runtime matches the parent exactly, and existing shared Split work is retained. The earlier merged-review pending flag is superseded for this bounded increment only.

[Independent Velnor compatibility research](../audits/2026-10-21-velnor-0130-parallel-steps-review.json) confirms current official GitHub syntax supports generated background/wait/wait-all/parallel steps. The behind-current actionlint version needs generated ignores; its generic empty-step ignore remains a lint-coverage risk, not proof of GitHub rejection. This research does not establish a hosted CI pass. The earlier missing-MasterDetail report was a stale local checkout; current remote source retains the public block and tests. Generator upgrade and required hosted checks remain separately verified work.


## Concurrent Jackin and theme changes not cleared (2026-10-21)

The [independent source audit](../audits/2026-10-21-jackin-theme-4fcbc-5d06-review.json) rejects acceptance of concurrent `4fcbc23ef26a3b6300499ef5aee1a1f6500511ba` and `5d06c610ea9c72bf770edbaf480288c3a85d4277`, already present in integrated history. Findings include a scenario-selected captured run identifier, exact 120x40/NO_COLOR brand correction, remaining fixed-cell launch replay, and new public LegacyMono inference from captured RGB pairs and row identity. Moving palette-driven screenshot reconstruction into Termrock does not satisfy the library gate. Retain useful semantic data and replay deletions while replacing these mechanisms through real shared components; no restored replay, silent rename or approval follows from integration.

The generic color-conversion decision must distinguish raw RGB quantization from semantic theme palettes. The frozen raw accent can quantize to index 78; index 77 is not a universal reference expectation. Native custom-gray assertions and generic conversion require an exact source-backed contract decision before changing tests. Shared scoped test passes do not clear these Jackin/theme findings or establish full product parity.


## Source-backed quantizer and Button reference decisions (2026-10-21)

The updated [independent theme audit](../audits/2026-10-21-jackin-theme-4fcbc-5d06-review.json) identifies the byte-identical frozen/design general `nearest_256` algorithm: rounded color-cube channels, floored grayscale candidate, and the original chroma/error rule. It yields custom gray #3a3a3a index 237 and raw accent #48e054 index 78. The replacement owner must use this general algorithm for all raw RGB values, without exact-color exceptions; semantic capability palettes remain separate. Both captured gray exceptions and a different modern Euclidean quantizer can violate the source contract. This decision preserves the fixed reference revisions and authorizes a source-backed regression correction, not a candidate-derived snapshot update.

The [independent Button diagnosis](../audits/2026-10-21-showcase-buttons-reference-diagnosis.json) corrects the initial comparison: States and audit/buttons are distinct named checkpoints. The stored initial audit artifact also conflicts structurally with the frozen initial-screen source and preserved diagnostic output. Diagnostic environment and settling differ from the full harness; exact named-root recapture and independent reference resolution remain required. No artifact is replaced or approved. The [minimal shared Button contract](../audits/2026-10-21-button-minimal-implementation-contract.json) reuses existing owner-wide inert reference state, adding only necessary Button focus visibility and marker handling. Shared implementation and later real screen composition are assigned separately; the current application gutter/pressed repaint and simplified matrix are not accepted.


## Pending integration protection and independent blockers (2026-10-21)

The local candidate merge `f77188fb3d0215b0fa2d34353328f5381c568d2a` combines reviewed parent history with the proposed Velnor 0.1.30 upgrade. It is not delivered to PR #38: [independent review](../audits/2026-10-21-velnor-39c-independent-review.json) and parent exact-binary regeneration both find different workflow/task-manifest output. The author sparse checkout omitted tracked rust-toolchain.toml; the official generator intentionally changes environment exports when that file is missing. Correct complete-input regeneration and exact re-review are assigned. Actionlint passing with generated ignores does not resolve the stale output. A temporary parent protection ref `wip/pr38-ci-integration-review-20261021` holds this unfinished merge; delete it only after successful integration and review.

The [fresh-plan d2 review](../audits/2026-10-21-fresh-plan-d2-review.json) requests a creation-anchor race fix: authenticating a later pathname hash can bless a coherent replacement after the original plan was validated. The digest must derive from exact exclusively published bytes and remain the expected value for subsequent reads. Sixty Python tests pass, but actual positive environment replay and tool-byte drift proof need the exact non-secret preparation recipe. No fresh-plan adoption, application capture, replay, or candidate approval follows from this increment.

Independent runtime window review also finds width-zero/positive-height full windows mapping rows despite an empty physical region. The source owner must repair the invariant and its tests before adoption; ScrollRegion, TextArea and Form adapters remain incomplete. All four product and complete library gates remain unresolved.


## Complete-input Velnor correction and bounded source reviews (2026-10-21)

The [independent exact Velnor clearance](../audits/2026-10-21-velnor-9cc5-clearance.json) supersedes the earlier generated-freshness failure for repaired source `9cc5cba3b33176c46ff840f40d39c6650f683500`, now verified on its protected remote. It materializes the tracked toolchain input and regenerates using official Velnor 0.1.30. All twenty-one entries (twelve non-directory entries) match an independent complete-input preview, including types, modes, symlinks and hashes. Parent repaired merge `e5b706432c4a85bf20285c54ab70638cd4f5bb80` also matches its independently generated complete-input preview. No generated file was hand-edited. The five-minute Plan timeout, 78 forbidden doctest commands, Rust checksum provenance warning, generic actionlint ignore coverage and unrun hosted CI remain acceptance blockers. This corrects generation freshness only.

[Independent Grid review](../audits/2026-10-21-grid-5cd18153-review.json) clears the bounded additive CellUi padding repair and generic raw quantizer at protected source `5cd181530ba61781cf6a2c69d97895a9b0741275`: eleven unchanged probes and 648 fitting-content callback configurations pass, and all 16,777,216 RGB inputs match frozen/design general conversion. The all-73 semantic-palette fixture remains failing; semantic recipe/role mapping is required before whole-theme acceptance. Existing narrow default-versus-callback ellipsis divergence is separately recorded, not silently discarded. This source is not yet adopted by this record.

The [original empty-window review](../audits/2026-10-21-window-c79-review.json) is repaired and independently [cleared at e2cd](../audits/2026-10-21-window-e2cd-clearance.json). This permits consideration of the pure prepared-layout metadata increment, not acceptance of unimplemented ScrollRegion/TextArea/Form adapters. The shared Button plan is also corrected: frozen Button markers are one-cell dots; current Radio glyphs are three cells and must never be cropped or widened into the Button layout. A separately coordinated semantic glyph contract remains required.


## Fresh plan integrity integration and remaining component findings (2026-10-21)

Integrated protected source `fff270fd0d2edcb0b06486fbcde07c88ee4feabe` at `94c34bc8c111e8ec8a43801e965d76b5de6be9c7`. The [independent final review](../audits/2026-10-21-fresh-plan-fff-independent-review.json) clears prepared-plan integrity with explicit procedural external-digest custody and tool-drift proof limits. Parent [checks](../audits/2026-10-21-fresh-plan-parent-checks.json) pass all 61 Python tests, syntax/help/whitespace checks, and actual anchored verification under the committed safe allowlist recipe: 29,069 visual cases plus twenty ordinary checks. No application is launched; authenticated resumable advance/finalize, capture, replay, comparison and acceptance remain unimplemented or unverified. Legacy registry and tuple exceptions retain their original scope.

The [new Jackin review](../audits/2026-10-21-jackin-426bb0c9-review.json) rejects acceptance of incoming `426bb0c9f`: its scenario/frame-selected application cockpit painter duplicates geometry/theme/text and omits live interaction mechanics while retaining 120x40 replay. That source was already on the shared remote before the CI integration; merging history does not clear it. Replace the mechanism through shared live-data components rather than restoring the replay or renaming the painter.

The [Button regression review](../audits/2026-10-21-button-2b3-review.json) found lost first keyboard activation after pointer focus. The [published repair review](../audits/2026-10-21-button-536ac-review.json) clears that bounded issue and modal focus with twelve public and 120 native tests. Button adoption remains pending a correct semantic focus-visible recipe (actual FOCUSED must remain observable), one-cell dot roles, and the independently recorded narrow Unicode ellipsis defect. No complete Button or application parity follows from this partial clearance.

The [CI doctest inventory](../audits/2026-10-21-velnor-doctest-policy-inventory.json) reports 78 generated commands across 39 library targets/two configurations but no runnable Rustdoc examples. This source inventory requires independent verification before any target metadata change; no check is disabled by this record. Public example and consumer coverage must remain intact.


## Prepared vertical-window metadata integrated (2026-10-21)

Integrated independently cleared source `e2cdce0affc4370dea73b9f67d187e8deb193c3a` at `1e32da0681347224e94a7295fd87999821086e0b`. Parent [exact integration checks](../audits/2026-10-21-window-parent-integration-checks.json) pass all 348 runtime tests, retaining three existing ignored fixtures, strict all-target testing-feature Clippy, formatting and whitespace checks. The checked optional typed field defaults to None and preserves existing prepared-layout semantics; empty width or height exposes no physical rows. ScrollRegion's actual window renderer/input mapping, TextArea source clipping, and Form cursor/menu integration remain separate required work. No application or full library acceptance follows from this metadata increment.
