# Current migration decision — 2026-10-21

This is the sole current summary. All later sections are preserved historical evidence; their intermediate tips, counts, local status and author-reported green results do not establish current acceptance.

| Application | Exact product gate | Reusable library gate |
| --- | --- | --- |
| Showcase | Incomplete; full paired inventory and repeated comparison missing | Incomplete; Button/theme adoption, capability coverage and remaining application mechanics need proof |
| TablePro | Incomplete; four independently identified form/footer/reference-data issues remain | Incomplete; cleared Picker/keymap changes need integration; form, tabs and composition contracts remain |
| Jackin (`jackin-preview`) | Incomplete; full paired inventory, modal/property flows and intermediate states remain | Incomplete; shared Dialog/Props/Button adoption and remaining presentation paths require review |
| Holla | Incomplete; unchanged real application probes fail and native fixture refresh lacks frozen authority | Incomplete; private list/presentation selectors and application mechanics remain |

Delivery remains PR #38, `parity/termrock-visual-baseline`. Main, PR #26/design and the frozen tag are read-only. Baseline source `4169783ae883788186d161af09ba9c481bd84e5c`, tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`, fixed design source `f4867e9b5f8a08a3bb61ce80d644198a29f51984` and Tuiscotti `a1ea27f757d75d07b05499276cd932e9f8e067ad` remain unchanged.

The coordinator's last verified PR #38 push is `34f73e3066da7a77eac2d4719a697dc085906dda`: Mono Button marker repair, 284 of 285 scoped tests passing, one retained 73-slot failure and zero skips. The separately published runtime correction `36b9a06509c991600956679e43295de27a2fd1ca` subsequently gained independent bounded clearance: 505 frames/49,728 cells match last-good output and warm redraw allocations remain zero. Neither result is four-application parity.

Ready source contracts awaiting coordinator integration include Button focusability `05875f028f27b45805805b0529cd8b66b2dd1051`, Props `8ff521660eb9297791264f7858fa3eeb493be4b2`, Theme profiles `847ccc74208e5ed3277bc4f3dd8fa6958e682713` and Picker/keymap `d40f8e5fb7e47dca8c971c718559f840e9ba693d`. Each has independent bounded clearance, not application acceptance. Preserve incoming exports, Dismiss defaults and current Theme additions during integration. The Picker WIP tip is remotely verified; other local source commits remain queued when the shared push lock prevents publication.

Velnor 0.1.31 and the empty-Shell-doctest fix were independently verified and generated 450 tasks, but the latest-release guard stopped publication after discovering 0.1.32. Upgrade `bd517d7cda5a4178b84542d07fa7db7c61d48701` has independent source/generator clearance and now awaits synchronized official regeneration and publication. No direct main/design push is authorized. Latest-release checks must run again before delivery.

Frozen run `58b38fb0b1a1007f14230337f8f1a225` retains nine authenticated capture steps and 288 checkpoints. No replay, ordinary CLI check or candidate comparison has run. The 707-file failed partial attempt is preserved. Capture nine is held because wrapper prechecks cannot atomically bind the official next-step choice. The agreed canonical runner guard requires a new sealed plan/run after review and publication; preserve this old run unchanged and transfer no checkpoint credit automatically. This changes neither frozen reference revision nor expected frames.

Required unresolved checks include the 73-slot reference decision, incoming Feedback Clippy diagnostic, ignored Esc scanner/reference defects, Shell measurement/API/routing/testing-feature findings, executable capability coverage, all-branch consolidation, full independent library consumer validation, full workspace Nextest/Clippy/formatting, hosted required CI and two complete paired application comparisons. Fresh source synchronization is not approval of incoming private painters or candidate-derived fixture refreshes.

Detailed clearances and blocked reviews are preserved under `docs/audits/evidence/2026-10-21-current-blockers/`; later exact integration records must explicitly supersede this summary.

---

## Historical checkpoint:  parent integration gate (2026-10-21)

Fields `332f40d`, Form window Stage1 `ae32d3f` and modal repair `02742e4` are integrated locally after independent clearances. The parent's scoped Fields/Forms/Runtime all-targets nextest, strict Clippy and package formatting now pass; the exact summary and remaining ignored tests are recorded in [parent evidence](../audits/2026-10-21-modal-fields-form-parent-integration.json). Modal's three review defects are resolved with 36,864 frozen-derived style cases and 10,092 legacy pair cases. This supersedes the earlier mapping rejections only for that bounded source contract. [Modal clearance](../audits/2026-10-21-modal-02742-independent-clearance.json).

NavList `9a550ed` and the corrected historical ItemRow fixture `eb82d6e` also have independent bounded clearances but are not yet adopted by this checkpoint. [NavList](../audits/2026-10-21-nav-list-9a550-independent-clearance.json), [fixture](../audits/2026-10-21-itemrow-eb82-independent-clearance.json).

Incoming `93b3065` made six expected-value overrides without repairing the palette. Independent review rejects that as frozen proof; the old universal raw projection is also not a semantic oracle. A separate test-integrity restoration removes the six new overrides while preserving the full inventory and documented unresolved red result. No reported Theme-green result authorizes acceptance of those expectations. [Review](../audits/2026-10-21-theme-expectation-overrides-rejection.json).

All four applications, the complete architecture gate, public API coverage and complete paired CLI checks remain incomplete. Source synchronization with other writers does not approve their incoming replay/painter paths or weakened checks.

## Historical checkpoint:  source and review update (2026-10-21)

Shared PR #38 advanced through `efda9593a905e3d462a62852748cf43254276dca`. Its history explicitly reverts the Jackin replay deletion (`c99dcef10`) and adds another private 120×40 painter (`8c7624537`). This reverses the earlier source-removal progress; neither retained replay nor a renamed application painter satisfies the library gate. New MasterDetail, runtime binding/hint and TablePro picker/footer changes require independent exact-commit review. Synchronizing that shared history preserves collaboration; it does not approve it.

The independent review of historical `49da9ae` proves two functional improvements but retains Debug-info navigation/copy, modal isolation and captured-frame painter defects; its one new strict-Clippy error is attributed separately from nine existing diagnostics. Two capsule cursor failures also reproduce its predecessor. [Review](../audits/2026-10-21-jackin-49da-independent-review.json).

Modal `65092d9` fixes the first two findings but remains rejected: explicit `Style.bg = Some(Color::Reset)` is conflated with `None`, producing Black instead of frozen overlay `#27272a`. Eleven probes have nine passes and two failures, while all 10,092 legacy pair outputs remain unchanged. Four environment runs preserve zero-allocation behavior. The same author is repairing Option semantics without changing the legacy pair endpoint. [Review](../audits/2026-10-21-modal-65092-independent-rejection.json).

Recolor primitives `dfb54cf` are independently cleared: unchanged public probes improve from four of fourteen to fourteen of fourteen twice, preserving complete cells, semantic channel provenance, transparent layers and interaction registries. Native runtime retains only the separately reproduced incoming modal allocation failure; this is bounded primitive clearance, not product acceptance. [Review](../audits/2026-10-21-recolor-dfb54-independent-clearance.json).

The persistent ItemRow historical fixture failure has a source-backed hypothesis: it reconstructs old foreground values but omits old disabled ink 238 after production was correctly changed to frozen index 239. A test-only worker must prove complete-cell equality before repairing that historical reconstruction. Production index 239 and the existing expected checksum must remain unchanged. [Diagnosis](../audits/2026-10-21-itemrow-historical-fixture-diagnosis.json).

## Historical checkpoint:  review checkpoint (2026-10-21)

All four applications and the complete architecture gate remain incomplete. Parent source integration is preserved on `wip/parent-fields-runtime-blocker-20261021`; it has not passed publication to PR #38 after incoming modal changes. Main, design/PR #26, the frozen tag and fixed source revisions remain unchanged.

- Fields `332f40d` and Form window Stage1 `ae32d3f` have independent bounded clearances. Form Stage1's eight public probes and 149 native tests pass; caret-follow, edit-menu and application form replacement remain outside that clearance. [Form review](../audits/2026-10-21-form-ae32-independent-clearance.json), [next follow contract](../audits/2026-10-21-form-follow-stage2-plan.json).
- Modal repair `95d21da` is **rejected**: independent public consumer has one pass and three failures. Reduced-color text retains faint/ghost instead of frozen muted-or-primary with DIM, and early collision fallback loses the DIM decision in TrueColor/ANSI256. An actual Harness cell also reproduces the wrong ink. The unchanged allocation gate is repaired, but these visual defects block adoption. The original author is fixing the shared mapping order. [Exact review](../audits/2026-10-21-modal-95d21-independent-rejection.json).
- Runner failure repair `ca4aff6`, delivered at `482cdaf`, is independently cleared within its failure-handling scope: five focused cases and all 75 Python tests pass. The unchanged tests against the old runner produce four failures and two errors. Child and timeout cases traverse real subprocesses with test-owned fake cargo; expensive source/build trust checks remain mocked. Existing six capture chunks remain paused; no plan, reference approval or candidate eligibility was changed. [Review](../audits/2026-10-21-runner-ca4-independent-clearance.json).
- FieldHover `4507e7d` is independently cleared: nine public probes pass; native Theme/Fields results retain the unresolved all-73 palette failure. Only two of 292 semantic outputs change (TrueColor and ANSI256 FieldHover). ANSI16 was already DarkGray. This is not a broad palette-oracle approval. [Review](../audits/2026-10-21-field-hover-4507-independent-clearance.json).
- NavList `0333cde` and recolor provenance `dfb54cf` are protected WIPs undergoing independent review. Neither has application acceptance. The NavList author's discovered default-Bar regression was repaired and pinned against `fa589153`; broader independent equality remains required.
- Incoming Jackin `49da9ae` removes replay files and uses shared Dialog, but remains under independent review for private frame-40 painting and functional property/modal behavior. The earlier `adf334b` review is historical evidence, not approval of its successor. [Historical review](../audits/2026-10-21-jackin-adf334-independent-review.json), [shared Steps replacement plan](../audits/2026-10-21-jackin-steps-live-adoption-plan.json).

## Historical checkpoint:  blocker update: runtime and capture integrity (2026-10-21)

Fields integration passed 504 scoped tests before incoming shared changes. Publication is now blocked by incoming `adf334b` introducing a deterministic modal snapshot allocation under inherited `NO_COLOR` (503/504 passed). The unchanged isolated test also fails; removing the variable passes only as diagnosis. No environment workaround is accepted. [Evidence](../audits/2026-10-21-fields-publication-incoming-runtime-blocker.json).

Frozen captures have six verified contiguous chunks, ordinals 0–5, totaling 192 cases/checkpoints/process receipts. Hash verification passed and final directories exactly match receipts. Capture is paused because independent review found that a post-rename verification/journal failure can leave an unreceipted directory with no failure marker, blocking retry. The missing injected-failure test is a coverage defect. A separate shared runner repair is underway; no replay, candidate comparison or product approval is authorized by these receipts. [Progress](../audits/2026-10-21-fresh-capture-paused-progress.json), [finding](../audits/2026-10-21-fresh-advance-failure-review.json).


## Historical checkpoint:  integration checkpoint: Fields window repair (2026-10-21)

Adopted source `332f40d07e38512f91cfb038eb9174c413755686` after independent review and parent verification: 504 scoped native tests passed, three existing tests skipped; strict scoped Clippy and formatting passed. This supersedes the earlier rejected horizontal-origin, captured-drag, and placeholder projections only within the reviewed contract. Forms adapter, caret-follow, edit-menu, clipped thumb policy, and all application parity remain incomplete. Evidence: [parent integration](../audits/2026-10-21-textarea-332f-parent-integration.json), [independent clearance](../audits/2026-10-21-textarea-332f-clearance.json).

The 1,301 missing tracked conformance artifacts were restored from exact Git HEAD, with their identities preserved. Cause remains unknown; controlled independent checkout did not reproduce loss. This is recovery, not a product fix or a diagnosed cleaner defect. [Diagnosis](../audits/2026-10-21-artifact-absence-independent-diagnosis.json).

# Termrock migration and acceptance matrix

## Historical checkpoint:  acceptance status — 2026-10-21

This section is the current decision summary. Detailed chronological checkpoints below remain evidence; their intermediate candidate statuses and branch tips are superseded by later exact-commit records. Parent delivery was remotely verified at `f2d66896f3341b115a217a1bdad6379badebccbb`; the subsequently observed shared PR #38 tip is `42fb861c07228db39f40228b5ecaa2e3521d6239`. Reviewed Grid and runtime-fixture integration was subsequently pushed and remote-verified at `71124871981c1138002b05bb9e7c2083cdd03836`, with the recorded semantic palette failure retained. The frozen source, tag object, Tuiscotti revision and read-only design reference are unchanged.

| Application | Product gate | Library gate and next dependency |
| --- | --- | --- |
| Showcase | Incomplete; complete paired comparison has not passed twice. | Selection-page ANSI-replay bypass removed at this tip: the delete-confirm dialog renders through the shared overlay `Dialog` and bulk-mixed selection through real selection rendering (audit `docs/audits/2026-10-21-selectable-bypass-removal.md`); two frozen 120x40 captures still show stale shell/nav/footer rendering and stay recorded, not re-approved. Remaining replay/correction paths and capability coverage require verification. Shared Button focus, dot glyph and narrow-label contracts are integrated; real state-matrix adoption remains required. |
| TablePro | Incomplete; protected Grid integration runs 195 TablePro/domain tests with 183 passing, twelve failing, zero skipped and no new failure names. Full paired coverage and reference conflicts remain outstanding. | Real shared editor and completion lifecycle are integrated. Grid/embedded-input painter removal is integrated and independently reviewed. Form lanes and Radio windows are integrated; Fields TextArea window regressions are repaired and independently cleared at `332f40d`; the separate Form adapter, caret-follow and edit-menu adoption remain pending. History and application form painting still require shared replacement. |
| Jackin (`jackin-preview`) | Incomplete; replay-backed frames are not acceptable parity proof. | Read-only inventory at `adc6d0e3f668818f6f8321310452db6806c123cc` found 41 fixed-cell historical replay modules, a separate historical palette helper, and two presentation RUNS frames. Replace them by shared components; preserve legitimate brand-specific animations and domain transitions. |
| Holla | Incomplete; fixed-size/time plate output is not acceptable parity proof. | Same inventory found 46 plate builders, private Finder painting and scenario/time/viewport selectors. Shared searchable sectioned results and responsive master-detail composition must replace them while preserving domain scenarios/actions. |

The Holla/Jackin inventory found their application source subtrees identical between fixed baseline `4169783ae883788186d161af09ba9c481bd84e5c` and fixed design reference `f4867e9b5f8a08a3bb61ce80d644198a29f51984`; those reference subtrees contain no historical/RUNS/plate mechanisms. This is source evidence, not a functional comparison. Holla's frozen Finder has one search/navigation owner and a responsive master-detail policy: detail share 34%, minimum 34 cells, maximum 48 cells, gap 2, collapse below 108 columns and one summary row. Existing public MasterDetail composition and prepared-layout contracts must express these constraints without an approved-size table or application-specific library variant; real Holla adoption remains required.

The independently repeated frozen nonincremental fixed-path cohort is admitted only for fresh baseline capture/replay, with four sealed binary hashes and no candidate privilege or checkpoint approval. Prepared plan integrity and authenticated-run custody are independently reviewed; bounded capture advancement is under implementation. Neither executable-byte reproducibility nor metadata verification establishes product parity. Ordered real-process receipts, immutable reference decisions, complete frozen capture/replay, candidate comparisons, functional outcomes, cursor/mode/cleanup checks and deliberate-defect controls remain required. Full workspace nextest, Clippy, formatting, CI, executable public consumer examples and independent final reviews are not complete. No application or library acceptance is claimed.

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
| `crates/jackin-preview-app/src/app/historical_*.rs` (17 replay files: `historical_accounts_add_form`, `historical_accounts_detail`, `historical_accounts_detail_80`, `historical_accounts_drawer`, `historical_accounts_filter`, `historical_audit_accounts_80`, `historical_capsule_menu`, `historical_capsule_menu_80`, `historical_capsule_new_tab`, `historical_capsule_palette`, `historical_capsule_split_vertical`, `historical_hard_launch_picker`, `historical_inspect`, `historical_launch_picker`, `historical_manager_menu`, `historical_usage_detail`, `historical_usage_overview`) | `RUNS` arrays encode positioned text and colors; paint those cells after scenario/state/viewport selection. These are production captured-frame replays. `historical_paint.rs` is a separate historical style-recipe helper, not a `RUNS` frame; review it as theme ownership, not as proven replay. Likewise Showcase `historical_palette` functions return semantic theme styles; the name alone does not establish frame replay. | Move behavior into shared widgets/blocks and feed Jackin domain data/actions. 17 replay files remain unresolved unless removed with a paired journey. `historical_capsule_zoom` is removed. Ctrl+B z on the paused 120×40 capsule paints `termrock::feedback::paint_zoomed_pane` from the tab, transcript, and status. `jackin/capsule/zoom/120x40` truecolor matches. `historical_intro_f300` is removed. Paused first-use frame 300 paints the live rain warp. `jackin/intro/f300/120x40` truecolor matches. Frame 400 stays the empty manager. `historical_accounts_filter_80` is removed. `/`, `work`, then Enter on the paused 80×24 accounts list paints `termrock::feedback::paint_accounts_filter`. `jackin/accounts/filter/80x24` truecolor matches. The 120×40 filter stays on its stamp. `historical_settings_mounts_80` is removed. Mounts on the 80×24 settings page reuses `paint_settings_page` with the mount table and the Mounts tab selected. `jackin/settings/mounts/80x24` truecolor matches. The 120×40 mounts page uses the same painter with Kind, Source, and the status cluster. `jackin/settings/mounts/120x40` truecolor matches. `historical_settings_route_80` is removed. `s` on the paused 80×24 accounts list paints `termrock::feedback::paint_settings_page`: tabs, commit checks, trailers, and the disabled Save. `jackin/settings/route/80x24` truecolor matches. The 120×40 general page uses the same `paint_settings_page` plus `SettingsCluster` (`inside the Construct`, `●`, `2 running`) and the full trailer address. `jackin/settings/route/120x40` truecolor matches. `historical_hard_cases_80` is removed. The paused 80×24 hard-cases manager paints `termrock::feedback::paint_manager_frame` from the workspace rows, the selected directory, and the detail band. The last list row keeps 55% of its ink. `jackin/scenarios/hard-cases/80x24` truecolor matches. `historical_hard_launch_locked` is removed. The hard-cases attach paints `termrock::feedback::paint_session_chrome`: menu, blank body, status, and hints from the session words. `jackin/manager/hard_launch_locked/120x40` truecolor matches. The 7 editor replay files (`historical_editor*`) and 8 settings replay files (`historical_settings_*`) were replaced with live termrock components (`editor_view.rs`, `settings_view.rs`). `historical_accounts_add_form_required` is removed. Backtab then Enter on the empty add form paints `termrock::fields::paint_field_error` over `historical_accounts_add_form`: the label mark, the field padding, `!`, and `Required`. `jackin/accounts/add_form_required/120x40` truecolor matches. `jackin/accounts/add_form/120x40` truecolor stays free of Required. `historical_cockpit_info` and `historical_cockpit_cancel` are removed. Manager `?` paints `HelpOverlay` over the dimmed manager. `historical_help_overlay` is removed. `jackin/manager/help_overlay/120x40` truecolor matches, including the hidden cursor. `t` on the paused launch paints the launch trace label on the activity row. `historical_cockpit_debug` is removed. `jackin/cockpit/debug` and `jackin/scenarios/launch-running` match at 72×20, 80×24, 100×30, 120×40, and 160×50 for truecolor, 256, 16, nocolor, and `--color none`. 120×40 still blits the presentation frame. The other sizes compose the baseline cockpit (strip, hero, stage rail, activity, cancel, footer) and downgrade that frame. Pending 40×12 and 60×18 have no approved snapshot (`missing-approval`); they are not cell diffs and must not be accepted. `jackin/cockpit/info` and `cancel_confirm` match at 72×20, 80×24, 100×30, 120×40, and 160×50 truecolor. Empty padding keeps the dimmed cockpit ink (`fill_keep_fg`); chrome is the centered block. 120×40 truecolor uses the presentation frame. Info uses `termrock::overlays::info_card`. Cancel uses `termrock::overlays::cancel_card`. 256 matches at 72×20, 80×24, 100×30, and 160×50: dialog inks use the baseline index table, and rain that the dim rewrote to index 235 is restored to 238 on black and on the card. 120×40 256 matches: the presentation frame is dimmed once, the dialog uses the index table, and the footer is the overlay hint row. 16, nocolor, and `--color none` match at 72×20, 80×24, 100×30, 120×40, and 160×50. Blank cells stay undimmed. Ansi16 folds a dark-gray button onto black. Mono keeps that face and grays the label. The stage-rail column stays undimmed. 120×40 still starts from the presentation frame. The other sizes compose the cockpit, then the same dim. Pending 40×12 and 60×18 stay `missing-approval`. |
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

## Historical checkpoint:  integration obligations

- **Grid embedded editor error styling (2026-10-21):** frozen source `visual-baseline:src/widgets/grid.rs:2872–2875` assigns the active inline editor the theme accent underline, then substitutes the theme error role when validation fails. The shared `TextInput` previously forced a fixed Accent underline, so failed commits stayed green. `TextInput::text_style_target` now transfers only the host's resolved text underline decoration; Grid's `GRID/TEXT` theme recipe owns Accent/Danger state, while `INPUT/FIELD` continues to own text ink and surface. This preserves profile downgrades and allows normal `GRID/TEXT` theme overrides. Real Grid-buffer tests exercise successful edit, rejected commit, correction, all four color profiles, and a custom Warning override. External review of the preceding WIP commit found the error-state mismatch; this follow-up addresses it. Focused checks: Grid 137/137, Fields 74/74, Theme 130/130; scoped Clippy and direct rustfmt checks pass. Exact TablePro base `6375fcee053d7859279ce436efde98c258ae649f` ran 181 tests (161 pass, 20 fail, zero skipped); merged candidate `5a7ac2d94` ran 182 (162 pass, 20 fail, zero skipped). All 20 failure identifiers match exactly; the candidate adds one passing inline-editor regression. Logs: `/private/tmp/grid-error-base-6375.log` and `/private/tmp/grid-error-candidate-5a7ac2.log`. This comparison establishes no newly introduced TablePro test failure, not product parity. Independent review of the follow-up commit remains pending.
- Independent review then found `GRID/TEXT` underline `Slot::Clear` did not override a pre-existing `INPUT/TEXT` underline. The shared resolver now preserves the final bound underline role or Clear provenance in private `PaintStyle` metadata; `TextInput` resolves the host decoration once and applies Set/Clear while leaving Inherit alone. This preserves `ResolvedStyle.explicit` semantics and public struct construction compatibility. Real Grid-buffer regressions cover input Warning plus host Clear and no-input-default Clear over prepainted cells; theme resolver tests cover Set/Inherit/Clear, local precedence, raw replacement, patch precedence, and all four color profiles. The synthetic-color probe implementation was rejected by independent review and removed. A subsequent external probe showed explicit caller Clear also needed to emit Reset over prepaint; the fields regression now checks local Set/Clear/Inherit precedence and that ordinary inputs retain their own theme. All five external probes pass; focused Fields/Grid/Theme gates pass 345/345 with scoped Clippy, rustfmt and diff checks. No PR integration or parity claim.

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

## Historical checkpoint:  CI timeout checkpoint (supersedes the workflow delivery status above)

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


## Historical checkpoint:  package gate, form reference conflicts and upstream CI dependency (2026-10-21)

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


## Historical checkpoint:  parent package gate remains red (2026-10-21)

Parent exact `124a61a035eeff34767edc8d9aa7e61f3007ebf8` runs 180 all-target TablePro tests: 159 pass, 21 fail, zero skipped. [Hashed log and name-set comparison](../audits/2026-10-21-tablepro-124a61-base-results.json) show one additional failure, `unfocused_grid_cursor_does_not_style_the_cursor_row`, assigned to the shared Grid owner. The hidden-explorer check was renamed; the old maximized replay-module frame check was deleted with the replay module. Actual maximized real-CLI coverage remains a required explicit verification rather than assuming the smaller native inventory is equivalent. None of the existing failed expectations was changed by this coordinator. All four applications and the complete architecture gate remain incomplete.


The bounded Grid follow-up at `e250d6e2d214a03d4d833ada2034504aabaca6d0` removes TablePro's scrolled-cell, cursor-row, and inline-editor after-paint paths. Focused row weight, gutter-tail/padding fill, and active-cell text now come from the shared Grid/theme implementation; the frozen TablePro sizing inputs use Grid's existing 200-row/95th-percentile sample and minimum-header padding/cap. The real unfocused-row and PageDown+End followed by horizontal-scroll regressions pass. On the exact merged source, TablePro has 20 failures in 181 tests, with no candidate-only failure names versus the exact 124a61 base (which had 21 failures in 180 tests); the sole base-only failure is the unfocused cursor-row assertion fixed by this increment. Grid/theme/fields pass 346 tests; scoped strict Clippy and formatting pass. See the [source and verification record](../audits/2026-10-21-grid-e250-review.json). Independent review of the final increment and full frozen CLI/maximized-flow coverage remain pending, so this does not establish TablePro or application parity.

The [generic RGB downgrade repair](../audits/2026-10-21-generic-rgb-downgrade.json) removes the exact-color lookup from ANSI-256 conversion. Frozen source `4169783` uses a general nearest-color algorithm; built-in semantic roles retain their approved profile indexes through `CapabilityPalettes`. A public probe and the native regression both reproduced #3a3a3a incorrectly mapping to index 238 before the fix; the corrected custom-theme and semantic-profile tests pass. Termrock Theme passes 133 tests and scoped strict Clippy/formatting. Independent review is pending.

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


## Completed CI ref cleanup and unresolved exact reviews (2026-10-21)

Removed two obsolete task-owned remote refs after literal-ancestor verification, exact tip leases, unprotected status, no open dependent PR, owner release, and the independently cleared complete-input CI integration: `wip/velnor-0130-pr38-20261021` and `wip/pr38-ci-integration-review-20261021`. [Deletion evidence](../audits/2026-10-21-ci-task-branch-cleanup.json) verifies both absent. [Current remaining-head inventory](../audits/2026-10-21-current-branch-inventory.json) links back to the original consolidation dispositions; it does not approve newly added or unreviewed changes. Main, design reference and frozen tag remain untouched. Needed component WIPs stay protected until reviewed integration.

The [Stage3A integrity review](../audits/2026-10-21-fresh-run-ee879-review.json) rejects permissive key custody and unbound/empty journal completion. Actual prepared metadata init/verify succeeds without applications, but that does not clear those boundaries. Both fixes are assigned before capture advancement. The [ScrollRegion review](../audits/2026-10-21-scroll-window-59e-review.json) reproduces a rendered logical track versus truncated input mismatch above 65,535 logical rows; source repair is assigned and this adapter is not yet adopted.

The [all-73 theme contract map](../audits/2026-10-21-junie-token-contract-map.json) separates 48 qualified reference mappings from 25 contextual, ambiguous or extension tokens. Global raw projection is not a semantic oracle; unresolved cases remain required. The [Button dot/focus agreement](../audits/2026-10-21-button-dot-focus-recipe-agreement.json) and [Jackin live component plan](../audits/2026-10-21-jackin-live-cockpit-component-plan.json) define shared replacements without restoring captured painters. The [independent doctest inventory](../audits/2026-10-21-doctest-policy-independent-review.json) permits considering empty-task removal only after a durable nextest doc guard and exact generated diff; guard implementation is assigned, with no metadata check disabled yet.


## Windowed ScrollRegion adapter integrated (2026-10-21)

Integrated protected source `67377aad3c3ab9b36acac47cb48f6c594172907a`. The [independent clearance](../audits/2026-10-21-scroll-window-67377-clearance.json) supersedes the 65,535-row truncation finding: rendering and input now use the same usize logical viewport, with checked source clipping and existing resize cancellation. Sixty legacy scenes preserve complete cells, styles and registered regions exactly. Unsupported partially clipped legacy custom slots report a bounded diagnostic instead of registering invisible controls.

Parent [integration checks](../audits/2026-10-21-scroll-window-parent-integration-checks.json) pass 406 Core/Runtime tests, retain three existing ignored fixtures, and pass strict scoped Clippy, formatting and whitespace checks. TextArea and Form adoption remain separate required increments. No application parity or complete library acceptance is claimed.


## Latest independent findings and synchronized runtime delivery (2026-10-21)

Windowed ScrollRegion integration is pushed and remote-verified at `dc3584eca42cd1a8ef961b44e831039c02bab944`. After synchronizing incoming source `d591748ac`, the exact merged HEAD again passes 406 Core/Runtime tests with three existing ignored fixtures, strict scoped Clippy, formatting and whitespace checks. Incoming Jackin changes were preserved as shared history, not accepted as parity or architecture proof.

The [independent dialog/fill review](../audits/2026-10-21-jackin-dialog-fill-review.json) rejects the scenario/size-selected dialog cell tables and suppressed live controls. Its public buffer probe also reproduces lost semantic foreground provenance in `Ui::fill_keep_fg` after dimming; a narrow shared-helper repair is assigned. Incoming `d591748ac` adds application recoloring and public raw recolor helpers, which remain unreviewed and do not resolve the prohibited mechanism.

The [independent documentation guard review](../audits/2026-10-21-doc-guard-400c-review.json) finds a concrete fail-open for cfg_attr doc attributes separated by newlines or comments. Its five focused tests and strict Clippy pass, but this guard must be repaired before it can justify any empty doctest-task metadata change. Exact base/current ownership findings are identical; untouched Showcase chrome formatting remains failing. The guard repair and authenticated-run foundation rereview are assigned separately. All four application parity gates remain incomplete.


## Shared Button focus and dot contracts integrated (2026-10-21)

Integrated protected source `57ca512b3796b04e518a73b5fdfe7391f312c6f3`: actual focused state remains observable, keyboard-only focus display is a shared Button policy, one-cell DotOn/DotOff roles preserve existing glyph constructors, and explicit marker parts retain precedence. The [independent clearance](../audits/2026-10-21-button-57ca-clearance.json) confirms sixteen public probes and the repaired authored Scrollbar-container rule filter; no application repaint implements these mechanics.

Parent [checks](../audits/2026-10-21-button-parent-integration-checks.json) run all 259 scoped tests without exclusions: 258 pass and the independently reproduced existing all-73 palette test fails at Indexed(78) versus Indexed(77). Strict scoped Clippy, formatting and whitespace checks pass. The palette failure is retained and remains required, not waived. Narrow Unicode ellipsis, actual Showcase state-matrix migration, application parity and complete library acceptance remain incomplete.


## Steps intrinsic-size repair requires API compatibility follow-up (2026-10-21)

The [exact 17841 review](../audits/2026-10-21-steps-17841-review.json) confirms four original measurement probes now pass, including live metadata and expanded ordinals. It also proves a new public API regression: an unchanged non-Display custom-row consumer compiles on c040 and fails with E0277 on 17841 because composition now adds a Display bound. This source remains protected on its WIP and is not adopted. Keep custom row rendering generic, and add pure intrinsic measurement through an optional row contract or explicit measured-row adapter without invoking painting or state projection. The author must repair this before adoption.

The Button integration is remote-verified at `e4c3a79d5f17469787b520388deceeda153e2459`; its complete scoped run retains the single known palette failure without exclusions. All four application parity gates remain incomplete.


## Grid integration protected pending expanded runtime diagnosis (2026-10-21)

Local integration `57e803cf7` combines independently reviewed Grid source 5cd with current shared Button contracts. Product sources merge automatically; the sole documentation conflict retains current records instead of an obsolete Velnor status. [Expanded checks](../audits/2026-10-21-grid-parent-integration-pending.json) run 834 shared tests: 832 pass, two fail, three existing fixtures are ignored. Strict scoped Clippy and formatting pass. The all-73 semantic palette assertion remains required; the separate natural-width test assumes focused Button gutters without focus-visible input state and now fails. Independent diagnosis must establish the correct input/measurement contract before any test change or PR38 adoption. This integration is protected separately and is not accepted.


The protected Grid integration also runs all 195 TablePro/domain tests: 183 pass, twelve fail, none are skipped. [Exact failure comparison](../audits/2026-10-21-grid-parent-tablepro-checks.json) finds no new failure names against the recorded Form integration run; the unfocused Grid cursor failure is no longer observed. This is a regression observation only. Existing functional, visual, performance and form failures remain acceptance blockers.


## Authenticated-run foundation cleared; documentation guard still incomplete (2026-10-21)

The [independent fa57 review](../audits/2026-10-21-fresh-run-fa57-clearance.json) clears the bounded Stage 3A custody/journal foundation on its protected WIP: three focused tests pass, anchored inventory remains 29,069 visual and twenty ordinary cases, and manifest HMAC plus 0700/0600 custody protections are independently verified without printing secrets. Empty/noninventory journals cannot claim execution. No applications were launched; baseline capture/replay, candidate comparison, finalization and product approval remain unperformed. Parent source adoption remains a separate increment.

The [7c397 documentation guard review](../audits/2026-10-21-doc-guard-7c397-review.json) confirms repaired cfg_attr trivia and tracked non-Rust-extension include scanning with six passing tests. It finds a remaining bypass for inner Rustdoc block comments (`/*! ... */`), currently absent from this repository but required for a durable guard. The author is assigned a regression and repair. No doctest task metadata is disabled or accepted by this record.


## Exact bounded clearances and required component repairs (2026-10-21)

The [runtime fixture review](../audits/2026-10-21-runtime-focus-measure-067a-clearance.json) clears source 067a: keyboard-visible focus preserves the 8/7 themed width comparison, while pointer focus preserves actual FOCUSED state and widths 6/6. Production focus rules are unchanged. The [Button narrow-label review](../audits/2026-10-21-button-narrow-38a375-clearance.json) independently reproduces parent failure and candidate success for the same Unicode consumer, with all 124 native Controls tests passing. Parent published the corrected-audit/source merge at `be1ef0e320332b6ce32b8b0f8e3e401389451bb6` on its protected WIP after exact tests and strict Clippy. The [fill provenance review](../audits/2026-10-21-fill-79667-clearance.json) clears the existing-background-only fill delegation at 79667. These bounded sources still require parent adoption and affected integration checks.

The [TextArea review](../audits/2026-10-21-textarea-61426-review.json) rejects adoption: three public failures expose changed ordinary horizontal origin/caret, lost captured selection outside the body, and a placeholder painted on the wrong clipped source row. Native 74/74 does not cover these failures. The author is assigned unchanged-probe regressions and repair before any Form migration. The [Steps review](../audits/2026-10-21-steps-8ac27-review.json) confirms restored non-Display compilation but finds DefaultRow long labels measured as a fixed twelve cells; a source-compatible pure row intrinsic hook is assigned. Neither source is accepted.

The [latest Jackin audit](../audits/2026-10-21-jackin-42fb861-review.json) rejects new exact-size replay, real-modal suppression and fixed-column footer painting. Its no-op recolor probes independently reproduce erased semantic foreground/background provenance; those helpers and application plates are not cleared by the fill repair. Preserve valid domain data and frozen wording while replacing the mechanism through real shared controls.

The [final documentation guard review](../audits/2026-10-21-docguard-fcb0-clearance.json) clears seven independent focused cases including full tracked source inventory, inner/outer block docs, nested comments, cfg_attr trivia and literal includes. Parent pushed and remote-verified exact fcb0 after all 34 scoped tests and strict Clippy. This clears the guard only; target metadata changes and official regenerated-task comparisons remain separate work. All four application parity and full architecture gates remain incomplete.


## Expanded Grid/focus checks after fixture repair (2026-10-21)

[Dedicated-target integration checks](../audits/2026-10-21-grid-focus-integration-checks.json) run all 834 affected shared tests after reviewed fixture source 067a: 833 pass, the retained semantic all-73 palette test fails, and three existing fixtures are ignored. Strict affected Clippy and formatting pass. A prior build using a target directory reused across two worktrees failed to see a method present in unchanged source; its logs are preserved. A fresh dedicated target builds that exact source successfully. No product source or assertion was changed to hide the build failure.

A corrected read-only Holla source map at exact shared tip 42fb861 confirms existing MasterDetail policy already expresses the frozen 34%/34–48-cell/gap-two/108-column constraints. The next identified consumer gap is a NavList composition adapter delegating to its existing measure/draw contracts, not another master-detail engine. AppShell availability was not established by that source investigation and is not claimed here. Holla application adoption, source-index data and actual paired behavior remain required.


## Shared background-fill provenance repair integrated (2026-10-21)

Integrated independently cleared source 79667 at `1fac27d0fc44387eb8fe78236693f1286417e198`. `Ui::fill_keep_fg` delegates to the existing canonical background-only fill, preserving retained foreground semantic roles under subsequent dimming. Parent [checks](../audits/2026-10-21-fill-parent-integration-checks.json) pass 358 runtime tests, retain three existing ignored fixtures, and pass strict runtime Clippy, formatting and whitespace checks. No application plate is approved by this helper repair; independently reproduced no-op recolor provenance loss and all application parity work remain unresolved.


## Narrow Unicode Button label contract integrated (2026-10-21)

Integrated reviewed source 38a375, including the independently corrected frozen-source audit identity, at `759858e6f46dee746fe202a988e9716777db2fdd`. Button labels use existing shared grapheme/cell truncation and retain explicit label-slot ownership; five-cell `世界` renders `世…` as the frozen contract requires. Parent [exact integration checks](../audits/2026-10-21-button-narrow-parent-checks.json) run 482 Controls/Runtime tests successfully, retain three existing ignored fixtures, and pass strict scoped Clippy, formatting and whitespace checks. Actual Showcase state-matrix migration and complete application parity remain required. The separate all-73 semantic palette test remains failing and is not part of this bounded green scope.


## Completed component task-branch consolidation (2026-10-21)

Removed five completed task-owned refs after exact-tip leases, literal-ancestor checks against PR38, unprotected status and no open dependent PR: the Grid protection ref, runtime focus fixture ref, fill provenance ref, original Button contract ref and narrow Button ref. [Deletion evidence](../audits/2026-10-21-component-task-branch-cleanup.json) verifies all absent and observes main, design reference and frozen tag unchanged across the operation. The [current head inventory](../audits/2026-10-21-current-branch-inventory.json) retains necessary unintegrated WIPs and marks newly observed heads as unreviewed; the original all-branch disposition ledger remains authoritative for earlier exclusions. No protected, reference, user-backup or unreviewed branch was deleted.

Narrow Button integration is pushed and remote-verified at `4ae4cb9dd7feb17f8835f9588d80bcedac0497fd`, after the same 482 Controls/Runtime tests pass on exact HEAD. This does not resolve the retained palette failure, production application replay mechanisms, or four-application paired acceptance.


## Steps intrinsic and custom-row contracts integrated (2026-10-21)

Integrated protected source `04a37e1bb63583a34948620897655dd057d7dd86` at `9ce228c1b952697aa9250eb43535bf16d1c9934d`. The [independent clearance](../audits/2026-10-21-steps-04a-clearance.json) supersedes both prior Steps findings: generic non-Display custom rows compile unchanged, built-in Display rows contribute actual cell widths through a source-compatible pure RowFn hook, and explicit custom measurement wins. Numbered ordinals, optional metadata and the composition adapter use the real shared Steps renderer and unchanged state owner. Measurement does not invoke row painters or state projections. Eight independent probes and all ten facade consumers pass.

Parent [integration checks](../audits/2026-10-21-steps-parent-integration-checks.json) run all 579 selected Runtime/Navigation/facade tests: 578 pass, the same pre-existing ItemRow fingerprint fails, three existing fixtures are ignored. Strict scoped Clippy, formatting and whitespace checks pass. The automatic RowUi merge preserves Grid padding and adds only the reviewed trait hook. Spinner timing, frozen focus/interaction details, Jackin live adoption and all application parity remain required; this is not a green full library gate.


## Core-interaction extraction from PR #26 (2026-10-21)

Keyboard-protocol decision (extract-kbd wave, 2026-10-21): the protocol ships capability-only, default off. Evidence: the frozen baseline captures (`baselines/tuiscotti-v1/**`) contain no `CSI >` push, `CSI ?u` query or `CSI <u` pop anywhere; the boot mode check (`crates/visual-harness/tests/visual_baseline/modes.rs`) pins exactly 1049/1003/1006/2004 under the runner's pinned `TERM=xterm-256color`; and PR #26 itself never negotiated at startup (only transient widget push/pop, e.g. `src/ui/pane_tree.rs:1087`). `TerminalSession::enter` therefore writes no protocol bytes, and negotiation happens only through `enter_with_keyboard_protocol` / the `TERMROCK_KEYBOARD_PROTOCOL` policy. Adoption handoff for the hint wave: render owners call `LabelOn::label_on(ClientPlatform)` for chord labels and `termrock_core::keys::keyboard::{label_deliverable, super_delivered, alt_delivered}` for delivery answers; `termrock-shell/src/keyboard.rs`'s simplified delivery copies should retire onto the core engine then, and `termrock-feedback`/overlay hints pick the labels up.

Worker core-extract landed the PR #26 interaction core (fixed source `f4867e9b5f8a08a3bb61ce80d644198a29f51984`) into the shared termrock crates on `parity/termrock-visual-baseline`. All landings are additive: no application, render path or frozen visual output changed, and every PR #26 default that would alter current behavior was retuned to the parity default.

| Extraction | PR #26 source (commit f4867e9b5) | Landing | Ported tests |
|---|---|---|---|
| Client platform detection | `src/core/nav.rs` platform block | `crates/termrock-core/src/platform.rs` | `env_override_wins`, `term_program_detects_macos_hosts` |
| Selection model | `src/core/selection.rs` | `crates/termrock-core/src/selection.rs` | 12: `toggle_flips_one_row_and_skips_disabled`, `select_only_replaces_the_selection`, `extend_selects_the_display_range_from_the_anchor`, `extend_keeps_rows_selected_before_the_anchor`, `extend_without_anchor_starts_at_the_cursor_and_skips_disabled`, `tri_state_and_select_all_ignore_disabled_rows`, `tri_state_glyphs`, `apply_follows_the_selection_model_table`, `context_select_keeps_a_selection_it_lands_in`, `gestures_come_from_the_click_modifiers_and_keys`, `click_policies_fire_on_their_click`, `remap_follows_removed_rows` |
| Step index helper | `src/core/nav.rs` | `crates/termrock-core/src/nav.rs` | `steps_clamp_or_wrap` |
| List typeahead + Nav profiles | `src/core/typeahead.rs`, `src/core/list.rs` | `crates/termrock-core/src/typeahead.rs`, `crates/termrock-core/src/list.rs` | 12: `apply_steps_pages_and_jumps_over_enabled_rows`, `a_horizontal_cursor_takes_left_and_right`, `skip_disabled_off_reaches_disabled_rows_but_type_ahead_never_does`, `an_empty_source_ignores_everything`, `all_disabled_stays_put`, `type_ahead_finds_cycles_and_a_miss_is_consumed`, `select_places_the_cursor_in_range_only`, `key_step_types_letters_when_type_ahead_is_on`, `the_profile_sets_nav_keys_and_type_ahead_defaults`, `a_bound_letter_wins_over_type_ahead_and_the_rest_type`, `type_ahead_first_is_an_app_wide_and_a_per_widget_choice`, `outside_route_key_and_in_popups_a_letter_types_at_once` |
| Outcome | `src/core/event.rs` | `crates/termrock-core/src/outcome.rs` | `or_keeps_the_stronger_outcome` |
| Nav key profiles | `src/core/keymap.rs` | `crates/termrock-core/src/keymap.rs` | `desktop_maps_arrows_paging_and_ends_only`, `hints_follow_the_profile` |
| UndoStack | `src/core/edit.rs:278-365` | `crates/termrock-text/src/undo.rs` | `typing_folds_into_one_step_and_redo_reapplies_it`, `a_sealed_run_starts_a_new_step`, `an_other_edit_breaks_the_run_and_the_limit_holds` |
| Scroll policies | `src/core/scroll.rs` (`reveal_span`, `FollowCursor`, `TrackClick`) | `crates/termrock-collections/src/scroll.rs`; `crates/termrock-viewport/src/viewport.rs` (`FollowCursor` enum replacing the bool) | 7: `a_paging_track_press_moves_one_viewport_toward_the_pointer`, `motion_after_a_paging_press_never_jumps_the_thumb`, `follow_on_move_leaves_a_wheeled_view_alone_until_the_cursor_moves`, `follow_always_pins_the_cursor`, `reveal_span_keeps_context_and_aligns_a_long_span_to_the_top`, `a_zero_context_keeps_the_frozen_minimal_reveal`, `pointer_move_centers_exactly_without_context` |
| Hit registry completeness | `src/core/hit.rs` (`register_backdrop`/`register_backdrop_of`, `Bar`, `ReleaseFrom`, `accepts_release`) | `crates/termrock-runtime/src/hit.rs` | 4: `a_bar_resolves_to_its_owners_track_and_the_thumb_is_its_own_part`, `a_thumb_without_its_bar_is_ignored_and_an_empty_track_never_registers`, `releases_are_accepted_per_widget_and_source`, `a_backdrop_catches_clicks_for_its_owner` |
| Pointer policy | `src/core/pointer.rs` (`SecondaryOutside` :40-67, secondary drag :501-511, bar phase routing :843-857); `src/core/hit.rs::backdrop_at` :469 | `crates/termrock-runtime/src/capture.rs` (`SecondaryOutside`, `SecondaryRoute`, `SecondaryTracker`, `backdrop_at`, `BarInput`, `bar_input_for`) | 3: `a_secondary_release_on_an_accepting_row_is_a_click`, `a_secondary_outside_a_backdrop_reopens_the_layer_beneath`, `bar_input_routes_the_press_phases` |
| Pointer/caret shapes | `src/core/shape.rs` | `crates/termrock-core/src/shape.rs` (inert: nothing in the workspace emits a sequence; apps opt in via `ShapeSync`) | 3: `the_sequences_are_byte_exact`, `support_is_detected_from_the_terminal`, `shapes_are_written_on_change_only` |
| Keyboard-protocol engine | `src/core/keyboard.rs:35-297` (`push`/`parse_flags_reply`, `encode`/`decode`, `KeyboardProtocol`, `ProtocolState`, session atomics, `AltKeys`, `observe`, `super_delivered`/`alt_delivered`, `label_deliverable`) | `crates/termrock-core/src/keys.rs` (`pub mod keyboard`; env overrides renamed `TERMROCK_KEYBOARD_PROTOCOL` / `TERMROCK_ALT_KEYS`) | 9: `push_pop_and_query_sequences`, `super_a_encodes_and_decodes`, `the_runtime_decodes_super_from_crossterm` (crossterm feature), `enter_leave_suspend_and_continue`, `the_env_override`, `alt_detection_per_platform_and_terminal`, `macos_terminal_delivers_alt_only_once_an_unambiguous_alt_key_arrives`, `another_macos_terminal_delivers_alt_with_the_kitty_protocol`, `alt_refused_or_delivered_by_detection` |
| Chord labels per platform | `src/core/keymap.rs:292-380` (`KeyBinding::label`/`label_on(ClientPlatform)`) | `crates/termrock-runtime/src/keymap.rs` (additive `LabelOn` trait over `Chord`; nothing calls it yet — the hint layer adopts it later) | 3: `labels_follow_the_pr26_rules`, `super_labels_per_platform`, `shift_is_part_of_the_char_it_names` |
| Keyboard-protocol negotiation I/O | PR #26 had no session wiring (only transient widget push/pop, e.g. `src/ui/pane_tree.rs:1087`) | `crates/termrock-terminal/src/keyboard.rs` (`KeyboardNegotiation` write side) and `crates/termrock-session/src/session.rs` (`TerminalSession::enter_with_keyboard_protocol`, `suspend_keyboard`, `resume_keyboard`; pop precedes the mode restore, including the panic hook) | 7: `enter_pushes_once_and_leave_pops_once`, `off_and_refused_auto_write_nothing`, `auto_pushes_when_the_terminal_answers`, `sequences_come_from_the_core_module`, `the_default_restore_writes_no_protocol_bytes`, `a_pushed_protocol_pops_first_on_restore`, `enter_terminal_modes_is_byte_stable_without_a_push` |

| Page router (Nav/Router/PageKey/Crumb) | `src/shell/nav.rs` | `crates/termrock-shell/src/nav.rs` | 8: `push_back_and_pop_to_walk_one_stack`, `replace_and_reset_drop_what_they_cover`, `tabs_keep_their_own_stacks`, `apply_builds_new_routes_and_reports_what_it_dropped`, `dropped_by_names_the_pages_a_move_leaves`, `a_breadcrumb_click_pops_to_its_level`, `every_move_that_drops_a_page_is_flagged_for_its_stakes`, `a_page_key_can_be_hashed_from_any_value` |
| Chrome geometry | `src/shell/chrome.rs` | `crates/termrock-shell/src/chrome.rs` | 2: `regions_split_the_frame_once`, `rows_reports_the_reserved_strips` |
| Status line + jobs | `src/shell/status.rs` | `crates/termrock-shell/src/status.rs` (tones map to `termrock_theme::Role`; `TONE_*` consts) | 7: the PR #26 `status_tests` suite (priority stays with the strongest message, errors outlast information, a newer message replaces an error, information expires after five seconds on the clock, plus the input/paste tests) |
| Quit guard | `src/ui/leave.rs` | `crates/termrock-shell/src/quit.rs` (`Stakes`, `QuitKeys`, `LeaveGuard`; confirmations described by `LeaveDialog`+`LeaveDialogKind` instead of building the overlays `Dialog`) | 5: `quit_keys_respect_text_focus_dirt_and_selection`, `a_clean_leave_goes_now_and_a_dirty_one_asks_save_discard_cancel`, `a_guard_that_cannot_save_offers_cancel_and_discard`, `running_work_asks_stop_or_stay`, `stakes_combine_to_the_strongest` |
| Keyboard deliverability | `src/core/keyboard.rs` | `crates/termrock-shell/src/keyboard.rs` (Super/Alt delivery answers; Kitty mode push/pop stays with the session crate); the full protocol engine now also lives in `crates/termrock-core/src/keys.rs` (`keys::keyboard`, see the core-interaction table) — the shell's simplified delivery answers are a pending adoption item for the hint wave | 4: `pointer_shift_labels_are_never_deliverable`, `cmd_and_alt_labels_follow_their_delivery_answers`, the observe test, `labels_carry_state_without_colour` |
| Key router | `src/shell/router.rs` | `crates/termrock-shell/src/router.rs` (`ShellKeys` priority/keys/fallback tiers over exact `Key` chords, `KeyHost` returning `Response`) | `the_key_order_table` (33 rows), `tab_reaches_the_fallback_tier_when_the_runtime_passes_it`, `a_paste_goes_to_the_top_layer_else_the_page`, `a_layer_that_passes_still_consumes_the_key`, `unbind_removes_a_key_from_every_tier`, `the_quit_letter_note_honours_the_configured_letter` |
| Page contract | `src/shell/page.rs` | `crates/termrock-shell/src/page.rs` (`Page` updating against the runtime `Cx`, `PageLeaf` composition adapter, `replay_key`, `rebuild_on_tick`) | 2: `replay_key_names_the_keys_a_menu_choice_names`, `rebuild_on_tick_rebuilds_only_when_the_view_moves` |
| App shell block | `src/shell/app.rs` chrome split | `crates/termrock-shell/src/shell.rs` (`AppShell` named header/body/footer slots over `CompositionSlots`) | 3: `the_shell_lays_three_named_slots_out_as_one_column`, `a_strip_slot_keeps_its_rows_and_the_body_flexes`, `the_spec_carries_the_gap_and_inset` |
| Footer ownership | `src/shell/footer.rs` | `crates/termrock-shell/src/footer.rs` (compose rules only; rendering goes through `termrock_feedback::HintBar` at adoption) | 6: `the_topmost_context_with_hints_wins_then_the_fallback`, `tab_next_is_added_once_and_never_while_editing`, `hints_never_name_shift_click_or_an_undelivered_cmd_chord`, `an_error_demotes_every_hint_and_takes_the_status_line`, `the_commit_key_stays_pinned_while_dirty_only_when_asked`, `a_status_message_reads_back_with_its_tone` |
| Reachability harness | `src/shell/reach.rs` | `crates/termrock-shell/src/reach.rs` (`Words`, `Reach`, `gate`, `reach_tests!`, probe over `Runtime` + test buffer) | 2: `words_split_cut_and_skip_the_footer`, `against_sorts_missing_from_cut` |
| Esc vocabulary | `tests/esc_vocabulary.rs` | `crates/termrock-shell/tests/esc_vocabulary.rs` (scoped to the shell's own vocabulary) | 3: `esc_never_quits_by_default`, `the_shell_notes_come_from_one_vocabulary`, `a_silent_esc_note_names_nothing` |

Shell-layer deviations (deliberate): focus movement (Tab/BackTab), too-small gating, hover suppression and flashing are runtime-owned on parity, so the router keeps `FocusNext`/`FocusPrev` as vocabulary only and the too-small order-table rows stay with the runtime; `row_anchor`/`open_context`/`row_choice` are unported because the parity overlay layer replaces `LayerTag` context menus; footer `StatusKind` and the peek (`ui::reveal`) integration map to the feedback wave's `HintBar`/status components; `Chrome::fit` and the menu command map stay with the overlays/menu-bar owners; `Words::add_overflowed` and the reveal-driven `Reveal` steps await the reveal owner. Open for the app-adoption wave: `tests/keymap_leftovers.rs` is widget-scoped (ChoiceDialog/FormDialog/DataGrid/HelpOverlay/PropsList/DataTable APIs absent on parity) and the skipped suites (`mouse_targets`, `focus_gutter`, `scroll_panes`, `choice_containment`, `terminal_suspend`, `table_perf`, `naming_conventions`, `strings`, `ascii_glyphs`, `cli`) stay with their widget-crate owners.

Frozen-behavior deviations (deliberate): `ScrollState::context` defaults to 0 (PR #26 default 1) so the existing `ensure_visible` reveal is byte-identical; `ViewportState` keeps its `follow()`/`set_follow(bool)` accessors over the new `FollowCursor` enum (`Always` equals the old `true`); `ShapeSync` is unwired, so frozen apps still send no OSC 22 / DECSCUSR output.

Handoffs for other owners: `MouseKind::SecondaryDrag` is missing in `termrock-core` `event.rs` `from_crossterm` (right-button drag maps to `None`), needed before `SecondaryTracker::on_secondary_drag` can be driven from real input; the full PR #26 `tests/mouse_targets.rs` port needs the widget-crate owners (it exercises per-widget targets); in parity the bar thumb region is owned by the bar id (PR #26 answered hits with the owner), recorded as the registry's documented adaptation.

Gate on pushed HEAD: `cargo nextest run` for core, text, collections, viewport, runtime (573 passed), `cargo nextest run -p termrock-conformance` failure set byte-identical to the parent commit (288 pre-existing failures from other sessions' landings, zero added), strict clippy `--all-targets` clean and `cargo fmt --check` clean on all touched crates.


## 2026-10-21: reviewed navigation row integration

Integrated NavList/RowUi source `9a550edecc55ee99b170ff31ee51957895aeb1ee` and historical test-fixture repair `eb82d6e1526cd55f21cdc66590b9ff36d7606deb`. Independent reviews are preserved under `docs/audits/evidence/2026-10-21-nav-row-integration/`. The row API preserves default rendering and exposes shared composition parts, cursor style, and configurable content gaps. The historical fixture explicitly reconstructs its original disabled palette; production colors and existing checksum literals remain unchanged. Its negative guard now isolates the intended muted-color defect. These changes do not establish Holla or other application parity.

The preceding modal/field/form integration was pushed and remotely verified at `913a74b879039fcf115318edee303c02d395869f`: scoped Fields/Forms/Runtime nextest passed 520 tests with three skipped; scoped strict Clippy and formatting passed. Incoming source from concurrent authors remains subject to independent architecture and product review. All four application acceptance gates remain incomplete.


## 2026-10-21: typed dialog and property slots

Integrated `a1903462d2abf972741f15d4b5873a953e5df471`, retaining the existing Dialog renderer, state registration, and PropsList behavior through public typed adapters. The independent clearance packet is under `docs/audits/evidence/2026-10-21-typed-slots/`; its SHA-256 is `9e7253efd9192745bdc0a3d8bc3acc8f5e29ef903961f6231abfd77ac06d2c2f`. The committed clipping regression fails on the previous implementation and the corrected external suite passes 24 tests without skips. Application adoption, InfoDialog chrome, scoped commands, and complete product parity remain unverified.

The preceding navigation increment was pushed and remotely verified at `b77445a6da757045f80cf793adc7812952aaeab9`. Navigation/runtime nextest passed 579 tests with three skips, and the public composition consumer passed three tests without skips; scoped strict Clippy and formatting passed. The first consumer invocation omitted its required `testing` feature and stopped before execution; the corrected invocation passed. Incoming concurrent Button/theme/Showcase changes were preserved through normal merge and still require independent review.


## 2026-10-21: recolor channel provenance

Integrated `dfb54cfc5dd9511b214a68677eb213492df897fd`. Existing recolor APIs keep their signatures, callback order, and eligibility; changed channels now preserve unrelated style provenance, while unchanged channels do not acquire new paint metadata. The independent packet under `docs/audits/evidence/2026-10-21-recolor-provenance/` has SHA-256 `50295af7cffefb3a0c25aeee72810bf631490cd10b1d7495b770dba43e75b05b`: the unchanged external suite failed ten of fourteen checks before the fix and passed fourteen twice afterward. This repairs a rendering primitive; it does not authorize application-owned widget repainting or screen-specific recolor passes.

The preceding typed-slot integration was pushed and remotely verified at `7a104258b020f717625848e7582d19ea6bdaf6ed`: scoped Runtime/Overlays nextest passed 479 tests with three skips, and five public slot tests passed without skips; scoped strict Clippy and formatting passed. Independent review additionally exercises Tab, ShiftTab, Escape, and repeated focus restoration across four profiles; this supersedes the earlier authored audit's statement that these independent flows had not yet been exercised. Application parity remains incomplete.


## 2026-10-21: palette test integrity and field-hover repair

Integrated reviewed `c60830f8a3b7279aa2046be7d3e7b6ca9984896a` to remove six unsupported expected overrides. Its original 73-slot inventory and three older overrides remain unchanged. This intentionally exposes an existing semantic mismatch; the retained universal raw projection is also not a valid frozen oracle for every semantic token. Neither its old expected values nor the removed candidate-matching overrides establish baseline authority. Independent clearance SHA-256: `4fa3e1161fb0801da119c3aea265c06d75150bb8b5c062b491eb555ca69ec9ab`. Corrected the requested minor audit wording.

Integrated separately reviewed `4507e7de2b730985bc3e8f5478f6cd68be9b51a3`: field-hover ink matches the frozen TrueColor and Ansi256 contract, preserving the other profiles and generic quantizer. Independent clearance SHA-256: `ec8ebfe6fe563bb0c980e33c22615f0ba8091e6b31a53165c56aa9b200255982`. Both packets are preserved under `docs/audits/evidence/2026-10-21-theme-integrity/`. Full theme/product acceptance remains incomplete; any known failing assertion must be reported explicitly, never excluded from the result.

The preceding recolor integration was pushed and remotely verified at `992953a9eb8092558b1de3043d40a0d4265f7f11`: Runtime nextest passed 377 tests with three skips; scoped strict Clippy and formatting passed.


## 2026-10-21: current capture evidence and rejected InfoCard mechanism

Independent exact-environment verification passed for the new runner-bound Stage3C plan on source `482cdaf48c42d8355a173e1faeee74e159c46d0b`; plan ID `58b38fb0b1a1007f14230337f8f1a225`, run ID `8470dddf86b24e1e8dcf3293d2625b1f`. The first capture increment completed 32 checkpoints at journal ordinal zero, and official post-capture verification passed. Its result SHA-256 is `2f520fa3342402946bcc415120c0d49408e345d737a78cc5ee770a0a2491430d`. No replay, ordinary checks, candidate comparison, or product acceptance occurred. The six older runner310 capture chunk trees and 192 receipts remain preserved and hash-matched. New source and old evidence are not interchangeable.

Evidence custody incident: an independent reviewer overwrote the earlier preparation summary at `fresh-plan-stage3c-independent-review-packet.json`. The original summary SHA-256 was `aca941c5817febacce2676493c36af4992faa0fa7f30c26f9ed25f81f1d12591`; its original bytes remain unrecovered. Plan, custody key, expected frames, and journal were not altered. Subsequent readiness verification uses the current immutable plan and official verifier, with separate exclusively created reports; it does not claim the lost summary was preserved. The replacement readiness result SHA-256 is `66821d5b848c1683b68dc9cff52fa2e4d81e832d8d4a81bf12ad660d3b63bc77`.

The exact `93b306503581d0c90665ca884af78000e3a29258` InfoCard audit rejects the production painter: it transcribes prior plate geometry/colors into a library function, omits real control regions/focus, loses or overflows fact data, bypasses theme profiles, and retains application repainting. Real Jackin probes distinguish frame40 from frame41: at frame40 Tab/Enter cannot activate Close, and clicking a visible fact dismisses the modal. Painted Close can dismiss through apparent light-dismiss; it is not reported as a dead click. Replacement owners are the existing real Dialog, PropsList, Button, shared theme, and typed adapters. Live domain fact extraction is useful and should be retained. The independent packet SHA-256 is `1d63ffe81e6005d82851afc3526fac6a59af306ca65c69a47d043a1ed9e1c984`. These findings remain open. Reports are preserved under `docs/audits/evidence/2026-10-21-stage3c-and-info-review/`.

During palette test restoration, a worker used symbolic shared stash operations and surfaced an unrelated `app.rs` edit. Ownership remains uncertain. The edit was excluded and preserved; stash object `8a8b5b5ca22235728602a0aa77d8eac0f81d2732` is protected by local safety ref `refs/termrock-safety/stash-8a8b5b5c`, without publishing its unknown contents. Future work must use scoped commits before switching and never symbolic stash pop/drop/clear in this shared repository.

Theme/Fields integration precheck: 228 tests executed, 227 passed, one failed, zero skipped. The sole failure is `all_73_junie_slots_match_reference_capability_output`; the full output is preserved as `parent-nextest-known-red.log` (SHA-256 `95430b39fadbf472750f23157bfc2c37dd739466c921c2fb5f78e0dabc6886da`). It includes the separately frozen-qualified field-hover value237 versus generic projection236 as well as the previously exposed semantic mismatches. Scoped strict Clippy and formatting passed. The red assertion is not waived, filtered, or reported as acceptance; the next reference decision must distinguish actual frozen component contracts from contextual or extension tokens. Publication was initially deferred because the MasterDetail writer held the shared push lock.

Independent CancelCard audit at exact `913a74b879039fcf115318edee303c02d395869f` also requires replacement. In all twelve frame40 size/profile cases, clicking painted Cancel launch dismisses as keep and leaves the launch running; Tab/Enter cannot activate the confirmation. The alternate frame41 path cancels but emits `UndeliveredIntent(cancel-dialog)`, a separate unresolved observation. The library painter lacks real control registration/state/action contracts, mishandles long/Unicode content, and retains application afterpaint recolor. Working dismiss clicks can be light-dismiss and are not evidence of typed activation. The initial diagnostic assumption about a missing bottom border was disproved and is excluded. Existing Dialog/Action/Button/shared theme are the replacement owners; retain useful borrowed domain labels/data. Packet SHA-256 `d22851254c4d3be49b7de88c1e91747af9bf8c65359be10e5813a4c65a4713f0`, preserved with the InfoCard report. No application gate passes on this evidence.


## 2026-10-21: CI inventory and runner failure integrity integration

Integrated reviewed source `482cdaf48c42d8355a173e1faeee74e159c46d0b`, including the empty-doctest task removal `69c1ed6aa2f18255dd92167d30f8534ad316bcd3` and runner failure-integrity fix `ca4aff6fb5c0dc22416e88e6a1d6d90abea34375`. The single Cargo merge conflict retained both the new public navigation consumer target and the independently guarded library doctest setting. Generated workflow inputs came from the reviewed official generator output; no generated file was hand-edited. Current-source prechecks passed 75 Python runner tests, seven doc-inventory guards, and 32 facade tests, all without skips; scoped strict Clippy and full workspace formatting passed. Earlier review packets are preserved under `docs/audits/evidence/2026-10-21-ci-runner-integration/`.

Freshness remains blocked: independent release discovery found official Velnor Actions `0.1.31`, published `2026-10-20T22:37:05Z`, source `9102646d80d0aadd05cac536b94ae830c4855e05`. The currently integrated generated output remains `0.1.30`; this is not described as latest or accepted. An official upgrade and fresh exact-output validation are required next. Hosted CI, required-check migration, full workspace nextest/Clippy, strict Rustdoc, and complete product acceptance remain unverified or unresolved.

The preceding palette integrity/field-hover increment and modal/capture audit records were pushed and remotely verified at `3bb1d1586e7af9bc6e625d6fdf6a27bc4dddd5cb`. Its Theme/Fields run remains honestly red: 227 passed, one known assertion failed, zero skipped; scoped strict Clippy and formatting passed.


## PR #38 landing-queue integration onto parity/termrock-visual-baseline (2026-10-21)

Imported author report for its historical landing-queue snapshot, not independent acceptance evidence. Later parent integration records supersede its hold list: the reviewed Fields, Steps, and other listed sources have since landed where explicitly recorded above. This report does not clear current application parity or its referenced changes without independent review.

Ordered cherry-pick integration of the cleared landing queue. Pushed tip sequence: `8a8e28300` → `499cec504` → `efda9593a` → `2d66ea9be` → `bfbb72516`; each push gated (`cargo nextest run --all-targets` on touched packages, `cargo fmt --check`, `cargo clippy --all-targets`) on the exact pushed HEAD and serialized through the shared push lock.

Landings:

- Four bounded fixes plus `wip/palette-grid-repair-20261021` (grid tests kept intact) at `8a8e28300`. `fill-keep-fg` and `button-narrow` were already landed (empty `git cherry`) and skipped.
- Style chain `empty-style-reference-fix-20261020` then `item-row-palette-reference-20261020` at `499cec504`; `wip/pr38-palette-style-integration` never landed separately. `item-row-palette-reference`'s only other unlanded commit was a matrix-doc note, folded here.
- Master-detail, `tablepro-live-contracts`, `tablepro-footer-focus` at `efda9593a`.
- Button chain (`showcase-button-consumer` with `button-contract-review` squashed into it) at `2d66ea9be` with three semantic merges, all stated in the queue: `BusyMarkerPolicy` defaults to `Replace` (frozen-contract behavior kept, `Keep` available as override); theme keeps the landed `Dot`/`DotOff` glyph roles (showcase overrides adapted); button focus gutter keyed on `FOCUSED | FOCUS_VISIBLE` per the landed FOCUS_VISIBLE semantics.
- `wip/table-consumer-baseline` (15 commits plus one compile adaptation: `press_track`/`drag_track` now return `bool`, two `let _ =` bindings added) at `bfbb72516`. `parity/table-component-20261020` not landed — superseded by this branch.

Skips (verified, not dropped work): `runtime-fields` remainder identical to the step-3 commits; `strict-accounting` already integrated via ancestor `407d727f9` (tools byte-identical, tip superset); `picker-family-contracts` and `picker-geometry` superseded by the landed hosted-query-ownership lineage (`7861b05ea`/`5c50c1776`/`35f623525`); `edit-menu-overlay` already landed as `39169524e`; `wip/termrock-reviewed-contract-integration-20261020` skipped per queue.

Holds:

- `parity/buttons-reference-decision-20261020` executable pins (`0f93b80b1`, `88bc8cdf2`): the branch pins `tools/frozen_cli_references.json` at `d5d8d648…` and adds the Buttons reference decision validator, but the landed cohort-review architecture freezes the same file at `2a43fb11…` inside six reviewed audit documents (`fresh-baseline-capture-eligibility`, `fresh-baseline-capture-evidence-recheck`, `cohort-capture-metadata-de156-review`, `baseline-cohort-adapter-review`, `baseline-cohort-schema-review`, `baseline-adapter-source-equivalence`), several themselves hash-pinned in `frozen_baseline.py`. Landing would rewrite frozen review records. The branch's pure-docs commits (paired Buttons CLI proof, TablePro delete-dialog parity proof) are landed. The Buttons pin must be re-advanced through the cohort-review machinery, not by editing reviewed decisions.
- `parity/frozen-table-workflows-20261020` (`b6d75a22d`): only its audit note landed; the executable table-sort extension content is byte-identical to content already at the tip (superseded), verified by hash.
- `parity/tablepro-picker-live-20261020` (10 commits): resolved 2026-10-21 by re-port, not a merge — see "tablepro-picker-live re-port" below. The hold is cleared.
- Per queue: `wip/textarea-fields-stage3-20261021` and `wip/steps-shared-contract-20261021` remain held.


## 2026-10-21: shared MasterDetail state and responsive resolver

Integrated independently reviewed `ababd6e737c8913ea718e4c37cb56117a3ddf8f3`, including the draw-purity/layout repair reviewed at `912681032b137dc6aa91757fada4abe60c572840`. Controls and facade now share responsive mode, summary sizing, and drawer/focus-return state while retaining distinct public defaults and input policies. Draw no longer changes drawer state through interior mutability, and update/layout/draw use the common resolver. The original public purity probe fails before the repair and passes afterward. The MD-2 independent review passed 177 native tests, five unchanged public probes, and 2,016 complete-frame/region/focus/cursor compatibility checkpoints, including narrow and Unicode content. Review packets are preserved under `docs/audits/evidence/2026-10-21-master-detail/`; SHA-256 `a085449a8647c4fe3aba3e0f688e4bc832f41f2fae5269dcbb741787d61c0102` and `a54333713da692ad2b683dd6723fe540c56e4b718e9838d8771f1ba625280a84`.

Showcase/public-API coverage remains incomplete: the current split source has no executable catalog/coverage gate equivalent to the former catalog, and no inspected Showcase MasterDetail consumer. The existing public file-catalogue consumer is useful but does not satisfy that missing gate. Application adoption and frozen four-app parity remain incomplete; neither component compatibility nor this integration is full acceptance.

The preceding CI inventory/runner integration was pushed and remotely verified at `3be8fb60b100253f7c0e9fd6f075a33563819694`: 75 Python tests, seven doc guards, and 121 facade/Fields tests passed without skips; scoped strict Clippy and full workspace formatting passed. Official Velnor0.1.31 upgrade and strict Rustdoc link repair are in progress. Concurrent incoming Holla changes include a native fixture update and require separate source/reference-integrity review; preserving their history does not approve them.


## 2026-10-21: public command scope adapter

Integrated WIP `6cc98ab0fbd2f5dc404115d8470a868a8d981665`, containing the independently cleared adapter `92a438867abbd23c4c2c78574070a13114b816e5`. `Ui::command_scope(owner, table, render)` uses existing focus scopes and scoped binding publication, with no new routing engine, fake control, geometry, or domain effects. The no-flags consumer API avoids promising disabled eligibility that the existing registry does not implement. Existing component remapping and removal remain effective. Real PropsList/TextInput/empty-modal probes cover four profiles, narrow/zero regions, repeated reopen/focus restoration, pointer and keyboard input.

The independent review passed ten authored slot tests, 377 Runtime tests with three existing skipped PTY entry fixtures, and 62 positive external checks; four deliberately unsafe global-routing variants were explicitly excluded from that positive run and separately confirmed to fail. Global Capture and Bubble semantics are unchanged and still leak background commands when used that way. Applications must actually replace the inappropriate global bindings with rendered scopes. Jackin additionally needs a genuine shared focusable rail separate from Steps navigation; no invisible control may substitute for it. This API-only integration does not establish application parity. Review packet SHA-256 `ca342f9fc43d3345b6ad8801c55a697cd3bd5266d983bb30ebd27886653103de`, preserved with the source plan under `docs/audits/evidence/2026-10-21-command-scope/`.

The preceding shared MasterDetail increment was pushed and remotely verified at `d29a834b7a73cee6208b6a23767fac70a8b638ac`: 177 Controls/facade tests passed without skips; scoped strict Clippy and full workspace formatting passed. Showcase/API coverage and all application acceptance gates remain incomplete.


## 2026-10-21: completed task-reference cleanup and current independent findings

Removed eight completed task-owned remote refs after fresh locked exact-tip, ancestry, open-PR, branch-protection, effective-rules, and owner-ended checks. Every deleted tip is reachable from the verified PR38 integration history. The deletion was atomic and compared each expected old tip; no commit history was rewritten. Deleted refs: `wip/form-data-measure-20261021`, `wip/form-textarea-window-20261021`, `wip/nav-list-composition-20261021`, `wip/runtime-modal-backdrop-20261021`, `wip/steps-shared-contract-20261021`, `wip/tablepro-grid-scroll-embedded-input-20261021`, `wip/textarea-fields-stage3-20261021`, and `wip/textarea-parent-window-stage1`. The fresh remote inventory had 98 remaining heads. No worktree, user file, unreviewed branch, protected branch, PR26, main, tag, or release was deleted or modified. Only PR38 and read-only PR26 were open in the independent inventory; no superseded open PR was closed. Detailed before/after evidence is preserved under `docs/audits/evidence/2026-10-21-consolidation-and-current-reviews/`. Necessary active and preservation refs remain pending review, delivery, or operational capture.

The command-scope integration was pushed and remotely verified at `6abc95d37044ae87dd698daac3da19d29ca15318`: 415 Runtime/facade tests passed with three existing skipped fixtures; scoped strict Clippy and full workspace formatting passed. Scoped APIs are now available, but actual application migration remains required.

Independent Button review of historical exact `2d66ea9be9b55b3f3a3b3c960b9acfeeff9c566e` requires changes: a new real Showcase Mono checked-marker regression, sixteen frozen variant/profile ladder mismatches, two inherited focus-state gaps, and an unused ASCII glyph defect. Its native 327-pass result used the then-present unsupported palette expectations and is not current product acceptance. The real inert state-matrix controls remain unregistered, a useful contract to retain. Historical paired receipts referenced another candidate and cannot establish this exact source. A bounded shared Mono marker repair is under independent review; no private repaint may be restored.

The current-source CI review at `d0845c0321b7c1819d49ff9e9f6dce3a8ee28789` independently confirmed seven doc guards, 75 Python tests, and exact official 0.1.30 generated output across all 21 entries after including `.config/nextest.toml`. Its initial sparse-input mismatch is superseded, not a candidate defect. Official Velnor 0.1.31 freshness and the strict Rustdoc link failure remain required fixes in that reviewed revision.

Fresh capture evidence now records 256 checkpoints across new-plan ordinals zero through seven. Every increment passed official exact-environment post-verification. The older runner310 capture's 192 checkpoints remain a separate preserved run and are not added to the new count. Replay, ordinary checks, candidate comparison, complete repeated application comparisons, and product acceptance remain unrun. Capture result packets are preserved with the earlier Stage3C evidence; later checkpoints will supersede this count explicitly.

Capture ordinal eight subsequently hit the 205-second supervisor deadline. No receipt eight or final chunk eight was published; the authenticated journal remains contiguous zero through seven, with 256 new-plan checkpoints. The partial `.attempt-00000008-8c17ea3b1cd28b72` and its 707 files were preserved without retry or manual cleanup. Ordinal nine was not attempted. Timeout packet SHA-256 `12eeab4ab93a13f91e919495996db69c8f3a24a993df40d946dcc58001f8dc52` is preserved with the Stage3C results. This is incomplete execution requiring diagnosis and verified resume policy; it is not a pass or checkpoint credit. Component, architecture, and CI work continue independently.


## 2026-10-21: shared Mono Button marker repair

Integrated independently cleared `2ba9652954ee34479288a3d7aa65656e17fecce5`, including production repair `f761f67b09c0a08fdcf7f9476e46b3175f532866`. Generic Mono selected/checked glyph substitutions no longer override the Button family's semantic checked-dot role. This fixes the real Showcase regression without restoring private repainting, changing RGB quantization, or selecting by application/theme color. Explicit theme, overlay, instance, Set/Clear, live/inert and other-marker behavior is covered by unchanged public probes. The original real-page probes fail before the repair and pass afterward.

The first review correctly blocked integration because the documented Mono rule-count contract still expected twenty Button rules. The correction explicitly pins all twenty generic identities, the exact two Button exclusions, and all eighteen remaining Button identities; other families retain their exact prior counts. Independent verification passes seventeen unchanged external probes and reports 283 of 284 native Theme/Controls tests passing, with only the separately retained `all_73_junie_slots_match_reference_capability_output` failure. Scoped strict Clippy and full workspace formatting pass. Both review packets are preserved under `docs/audits/evidence/2026-10-21-button-mono/`; final clearance SHA-256 `fd2edc9a94a86f5936bba000febcb20649087382bb8165b13b94f09e95a0145d`.

This is bounded regression repair, not Button or Showcase parity. Sixteen frozen variant/profile ladder mismatches, inherited focus-state gaps, the unused ASCII glyph issue, current semantic-palette assertion, missing public API/Showcase coverage, and full paired CLI verification remain unresolved. The preceding evidence/branch-cleanup record was pushed and remotely verified at `5f82c4d35d12e0fb77b0f580a8ef200240661602`; that documentation-only push did not claim new Rust or product validation.


## 2026-10-21: Velnor 0.1.31 integration review

The shared Mono Button repair was pushed and remotely verified at `34f73e3066da7a77eac2d4719a697dc085906dda`. The exact integration gate ran 285 Theme/Controls tests: 284 passed, one retained semantic-palette assertion failed, and none skipped. Scoped strict Clippy and full workspace formatting passed. This supersedes the source-only native count, not its unresolved product status.

Integrated independently reviewed Velnor upgrade `2e81cb9ee64f0073e849d862bbc0c85f6647fe15` into the local PR #38 integration copy only. Official release 0.1.31 pins and generated output replace 0.1.30; one redundant Rustdoc link is corrected without Form behavior changes. Independent review SHA-256 `e19a352e486edd6138fc9e2c608d0510eb55080a3a15ef0e840c91196a2add18` and actionlint supplement `a7590cbd15d527717b51d1520988d0a0777e99fa0dfbeca6d2b3cc595adbbcf2` are preserved in the adjacent evidence directory. Main already requires the generated `Required` context through ruleset 24883153; the prior migration warning does not establish a current context failure. Rust tool checksum verification remains unresolved.

Publication is pending the shared push lock. After normal synchronization, the coordinator must regenerate with the official pinned CLI, compare generated bytes, modes and symlinks, run planning and focused runner/doc/strict Rust checks, push and verify the remote commit. Hosted CI, full workspace checks and twice-repeated four-application comparison remain incomplete. Main, PR #26 and the frozen tag remain unchanged. Incoming application fixture refreshes and runtime painting edits are not approved by this CI increment.


## 2026-10-21: independent blockers and controlled capture resume

Independent Holla review at `3be8fb60b100253f7c0e9fd6f075a33563819694` rejects the claim that removing old plate files completed semantic migration. Exact scenario/tick and viewport paths, separate painted and interactive lists, private painting and cursor output remain. Three unchanged real application probes fail before and after the removal. Of 1,088 native expected entries, 696 hashes changed without a frozen-reference decision; neither historical nor refreshed data is accepted as the product oracle. Both versions remain preserved in Git and external evidence. Native sample checks do not replace frozen paired checks.

Independent review of Button focusability prerequisite `6509b91e721057b9d5ec4f5c998ded752f29066c` finds two real event-time regressions: changing to accepted `Focusability::Disabled` before consuming an already-routed Enter or pointer release still activates. Seventeen of nineteen external probes pass, but both required probes fail; native Controls 145/145 does not clear the change. Integration is held pending a component correction and independent re-review. Props corrections `386b895597b220d949304e506bd5c629792b8b25` and Picker changes `d911e577a1342729719a746a1b314bea7880be46` likewise await independent clearance and publication. Picker package tests retain twelve identical pre-existing failures.

Ordinal eight timed out after its 32 capture cases passed, during verification/publication, not during a stuck scenario. No authenticated receipt or checkpoint credit exists for that partial attempt. The first external supervisor revision had three independently confirmed defects and is rejected. Corrected supervisor SHA-256 `2765ef46ecb834225b30c75686c8f44610a81559a66e80b31afbb8d0a574804b` is independently cleared for one complete ordinal-eight retry under an inclusive 220-second limit and a quiet resource window. Its review SHA-256 is `4e5775397945ecdb7e3f16eeeb8b0ce37ce91e226dcd47d4942366866589cf3d`. Separate-session descendant cleanup is not proven. At this record, no retry has launched; the journal remains ordinals zero through seven and 256 checkpoints. Ordinal nine is blocked until receipt eight authenticates and official verification passes. Failed attempts, the 707 partial files, immutable plan, key custody and original reports are preserved.

Exact review packets are preserved in `docs/audits/evidence/2026-10-21-current-blockers/`. All four applications remain incomplete under both acceptance gates.


The incoming TablePro/runtime source review at integrated `34f73e3066da7a77eac2d4719a697dc085906dda` raises five required findings against `d0e7ec9bf` and `05d63fe9c`: fixed eight-color majority detection drops later colors; new-connection fields copy active connection values; the connection form changes frozen composition; one exact SQL string selects an unsupported footer; and 157 of 168 native snapshot digests changed without a frozen-reference decision. Removing the redundant `tab_ownership` match arm is separately accepted as harmless cleanup. Prepared unchanged public probes are preserved beside the source review, but compilation was withheld during the capture resource window. These are source-review findings awaiting behavioral execution, not reported product-test results.

Props correction `386b895597b220d949304e506bd5c629792b8b25` remains blocked: independent default compatibility comparison found 96 of 480 changed checkpoints and 696 changed cells inside the component host. Intersecting the value rectangle before wrapping restarts logical text when scrolled; an unchanged Down-event trace shows the wrong visible substring despite identical state. The author is correcting clipping without changing logical layout. Two older style probes also contain raw-color expectations that require independent Mono qualification; they must not drive production palette exceptions.


Ordinal eight subsequently passed one complete retry under the independently cleared v2 supervisor: exit zero in 145.589 seconds, within the inclusive 220-second bound. Official plan and run verification passed; receipt eight authenticates 32 selected cases and 32 checkpoints. The new-plan run now has nine contiguous authenticated capture steps and 288 checkpoints. The earlier partial attempt's 707 files remain unchanged. Result packet SHA-256 `b804d2defce25f6a27247a05fa43759ea6cfdf48dfe8bc1f5bbdda5499be1bb2` is preserved beside the review decision. This supersedes only the prior ordinal-eight execution hold; no ordinal nine, replay, ordinary CLI check or candidate comparison has run.


The first synchronized Velnor delivery attempt reached exact generated-preview equality (21 entries), plan discovery of 48 crates, actionlint, 75 Python checks, seven doc-inventory checks, scoped strict Clippy, strict Forms Rustdoc, full workspace formatting and diff checks. Publication was correctly aborted when the coordinator's push-lock owner file disappeared before the final ownership assertion. A subsequent Button workstream reported successful acquisition only after the directory was absent, an interrupted fetch and no push; it released its own lock. The actor or cause of the coordinator's lock loss is unproven. No lock was bypassed and no shared history was force-pushed.

Further independent inspection found the newly arrived `termrock-shell` library has no runnable doc examples but its implicit Cargo target enables doctests. Official regenerated 0.1.31 output therefore includes empty default/testing doctest tasks despite the seven current inventory tests passing. This is a real CI planning gap requiring a source manifest correction and guard coverage before publication. Generated files will be regenerated only through the official CLI after that correction; neither passing narrow checks nor the current generated plan is full CI acceptance.

One coordinator handoff briefly activated six child agents before the completed Button author's final status had settled, exceeding the five-child ceiling. The coordinator immediately interrupted that author and checked its pending fetch/lock cleanup. Future activations require a fresh live-status count first; completed source reports alone do not establish a free slot.


## 2026-10-21: bounded Button and Props clearance decisions

Button correction `05875f028f27b45805805b0529cd8b66b2dd1051` is independently cleared for integration: all nineteen unchanged external probes pass, including both previously failing already-routed Disabled intents. A separate four-profile live/inert disabled-focus probe passes; Controls 149/149 and strict scoped checks pass. API catalog, Showcase coverage, broader ReadOnly semantics and application adoption remain outside this clearance. Independent packet SHA-256 `c79fc62b75883cb79fe990a22a85f767754f43ef80b381829cfb1ac54f13b458` is preserved.

Props `8ff521660eb9297791264f7858fa3eeb493be4b2` clears all seven independently raised source findings, including wrapped continuation and explicit Theme/overlay/instance container foreground inheritance through Label Clear. Native 347/347 and scoped strict checks pass. The preserved default comparison covers 480 checkpoints: zero inside-host cell changes and identical focus/cursor/state tails; 584 removed cells lie outside the component host. The coordinator accepts these removals as a component clipping correction: a component must not paint outside its allocated host, and wrapping retains the original logical geometry. This decision does not approve a change to an application baseline frame.

Two earlier external style assertions expected raw Blue/Cyan through Mono conversion. Independent source qualification confirms canonical downgrade to Black or Reset/inherited style; separate corrected probes pass across all four profiles while the original two failures remain preserved as test-reference defects. No frozen artifact or production palette is changed to satisfy them. Final Props clearance SHA-256 `0653c1f37408b3ff713771211e52d3985c2bf900efb9559fbf249d8367e77058` and both intervening blocked reviews are preserved. Publication and exact synchronized integration checks remain pending; no application parity is claimed.

The incoming runtime/TablePro behavior review now has unchanged before/after execution: prior `5f82c4d35d12e0fb77b0f580a8ef200240661602` passes both probes, integrated `34f73e3066da7a77eac2d4719a697dc085906dda` fails both, with no skipped cases. Scroll fading leaves foreground 255 instead of 140; New connection copies active Name/Database/User instead of blanks. Packet SHA-256 `f9f30a36ace1b7aca6639a0a8a0abf3679e6313cc9d1e928a3f889bdb6874963` is preserved. A shared runtime correction is assigned; other TablePro findings remain unresolved.

The following incoming workstream report is preserved for provenance. Its native visual-baseline and CLI claims are author-reported sample checks, not frozen paired-run or independent architecture acceptance; the independent blockers and current acceptance status above remain controlling.

## 2026-10-21: shell extraction landing and tablepro-picker-live re-port

### Shell-layer extraction landed

`wip/termrock-extract-shell-20261021` (13 commits, tip `9892d22a0`) merged into
`parity/termrock-visual-baseline` as merge `70229e698` onto the then-origin tip
`01ed92ca9`, pushed as `fab2e44dd`. Adds `crates/termrock-shell` (nav router,
chrome, status line, quit guard, keyboard deliverability, tiered key router,
page contract, AppShell slots, footer compose rules, reach harness, Esc
vocabulary, `shell_consumer` example) plus the additive `termrock::shell`
facade re-export and the matrix shell rows from `6b4248812`. No shared-crate
render path and no app crate source changed.

Gate on the exact pushed HEAD: `cargo nextest run -p termrock-shell` 48/48,
`cargo clippy --all-targets -p termrock-shell` clean, `cargo fmt --check`
clean, `cargo check --workspace` exit 0.

Note: an interim run of the tablepro visual baseline on the pre-`01ed92ca9`
base (`5f82c4d35` + shell merge) failed the frozen digest; the digest passes
at `01ed92ca9` and at the pushed HEAD, so the red belonged to the superseded
base, not to the shell merge. Evidence: `--test visual` PASS at `01ed92ca9`
and `fab2e44dd`, FAIL reproduced only at the stale base.

### tablepro-picker-live re-port

`parity/tablepro-picker-live-20261020` (10 commits) was re-ported onto the
landed `query_input` design per the hold note above. The branch's shared-Picker
intent for TablePro's tab list had already landed through the hosted-query-
ownership lineage (`tab_list.rs` shared `Picker`, `Ctrl+G` surface, picker
Delete reopen, focus-to-editor); the port therefore landed the branch's
missing live contracts rather than its `query_editor` internals:

- Landed in `crates/tablepro-ui/tests/quick_switcher_contract.rs` (commit
  `fab2e44dd`, test commit on top of merge `70229e698`):
  `tab_list_picker_uses_stable_tab_keys_for_filter_activation_and_delete`
  (includes the three-tab frozen geometry `(28, 15, 64, 10)`),
  `query_tab_metadata_uses_the_latest_domain_execution_summary` (adapted from
  the branch's `history.push` fixture to the landed `run_query` ->
  `last_status` mechanism; asserts `ok · ` and `error` in the picker text),
  and `tab_list_picker_keeps_the_frozen_single_tab_geometry`
  (`(28, 16, 64, 8)`). Both frozen geometry cases pass at both sizes through
  the real shared Picker path: no size, theme, or tick special-casing and no
  expected-frame updates.
- Dropped as duplicate: `chosen_query_tab_moves_focus_to_its_live_editor` —
  covered by
  `tab_list_contract::choosing_a_query_tab_moves_focus_to_its_editor`.
- Dropped as superseded: `deleting_a_dirty_query_uses_the_domain_confirmation_flow`
  — the branch's destructive-confirm flow for picker Delete contradicts the
  recorded landed decision that picker Delete discards dirty edits
  unconditionally (`docs/audits/2026-10-20-tablepro-live-contracts.md`,
  commit `c5d60225a`); the landed
  `delete_discards_dirty_query_tab_from_picker` contract stays authoritative.
- Discarded branch internals superseded by landed work: the `query_editor`
  filter-list model (branch commits `5e0981dae`, `06750c565`, `34d1ecd12`
  theme/repaint halves owned by the theme lineage), and `d11975020`
  viewport-margin layer constraint (runtime/layer.rs, not ported here).

Gate on the exact pushed HEAD: `cargo nextest run -p tablepro-ui --test
quick_switcher_contract --test tab_list_contract` 19/19, full
`cargo nextest run -p tablepro-ui` 194/194 (visual baseline included),
`cargo clippy --all-targets -p tablepro-ui` exit 0, `cargo fmt --check`
exit 0.


Integrated source correction `b2bcf918cd013874199b4aaaa8d53d9aa43966ab` for the new empty Shell doctest jobs. Independent review confirms eight doc guards, 48 Shell tests, strict checks and metadata with doctests disabled. The new guard mutates only an in-memory fixture derived from real library source. Official isolated 0.1.31 generation removes exactly the two empty Shell doctest tasks, 452 to 450, without changing common task definitions. Review SHA-256 `d3886e975594de180e36283f3399221c4aacdfd412c664ba47b06b0e6b2b4b82` is preserved. The combined source/tool upgrade still requires synchronized regeneration, exact checks, push and remote verification; no hosted CI or product acceptance is claimed.


## 2026-10-21: further independent contract findings

The new Shell source architecture review records six required findings: duplicate incompatible Shell/Chrome implementations, region/slot geometry disagreement, body minimums that can eliminate the footer, raw key routing without demonstrated runtime scope integration, unconditional production testing-feature exposure, and missing real thin-consumer/capability proof. Generic Router/Nav, typed leave policy, Moment status and existing CompositionSlots are useful contracts to retain. No second renderer or application-domain import was found. A bounded canonical geometry implementation is committed at `6b73822c361e3a253709ad71fe6c885693fd1135` and awaits independent API/runtime review; other findings remain open.

Picker correction `1e30bd458dafc3f493cba207acb135ac964f85ce` resolves real BackTab+SHIFT and explicit-reset defects, but independent review found global keymap remove/remap still used raw chord equality. Its follow-up `d40f8e5fb7e47dca8c971c718559f840e9ba693d` is remotely verified on its WIP branch and awaits re-review. The three known skipped native fixtures remain explicit. Current landing must retain incoming facade Shell exports and the Picker Dismiss default tail.

Theme profile correction `734c8662a015b85bcddf9f85081124ae5d1ae785` fixes the introduced profile-free Mono quantization regression. A separate same-valued authored-channel provenance gap also exists in the previous family profile API; it is inherited, not a newly introduced regression. Final correction `847ccc74208e5ed3277bc4f3dd8fa6958e682713` uses targeted channel ownership rather than value equality and awaits final review. The 73-slot assertion remains red and has not been weakened or blessed.

Runtime correction `25777df94dd802869833389b6a11164877350b4d` reproduced last-good rendering across 480 scenes and 47,328 cells but introduced warm redraw allocations beyond eight colors. The author corrected reusable scratch ownership and pushed `36b9a06509c991600956679e43295de27a2fd1ca` to PR #38 before the independent re-review completed. Publication is not acceptance; exact-head rendering/allocation re-review is underway. Future component authors are explicitly restricted to WIP publication until independent clearance and coordinator integration.

Capture supervisor v3 is held. It directly invokes a private real-key reader rather than delegating key access to the official runner, and its requested ordinal/phase precheck is not atomic with the official runner's selection of the next step. No secret bytes were printed or committed and no exposure is inferred. No ordinal nine ran. The authenticated capture count remains nine steps and 288 checkpoints; resolving ordinal binding must not mutate or silently replace the current plan or old results.

Official generation after the reviewed Shell manifest fix produced 450 tasks with no empty doctest tasks at local `51a3601c0`. Before publication, the release freshness guard detected official latest `v0.1.32`, so the 0.1.31 transaction stopped and released its own lock without pushing. A separate bounded 0.1.32 upgrade is assigned; main, PR #26, the baseline and fixed Tuiscotti revision remain unchanged.

Velnor Actions `0.1.32` supersedes the `0.1.31` tool pin. The release manifest, platform checksums and source revision are recorded in [the 0.1.32 upgrade audit](../audits/2026-10-21-velnor-0.1.32-upgrade.md); the earlier 0.1.31 audit remains historical. Official generation and preview match across all 21 `.github` entries. The 450 task IDs and semantic definitions match the parent source's 0.1.31 output, with no empty Shell doctest tasks or `cargo test` commands. The 75 Python runner tests, eight doc guards, Shell tests, scoped strict Clippy, strict Rustdoc, full formatting and pinned actionlint pass. This records source and generator checks only; hosted CI and the four-app frozen comparison remain incomplete. Independent review is pending.


Independent 0.1.32 review clears exact `bd517d7cda5a4178b84542d07fa7db7c61d48701`, with all 21 generated entries matching official output and preview, 450 semantic tasks unchanged, 75 runner tests, eight doc guards, 48 Shell tests and scoped strict checks passing. Exact diff repairs only three Rust documentation links beyond tool/generated inputs; no behavior or public API changes. Review SHA-256 `091b4b04b48deb91b580c8e1680aaa0f5c70bd0331462410394234b4a7f3a29a` is preserved. The isolated reviewer cache cleanup was rejected by its command guard and remains preserved; no user files were removed. Publication still requires fresh synchronization, generator freshness and exact-head checks; hosted CI and application gates remain incomplete.

The next incoming workstream report is preserved as historical author-reported evidence. Current acceptance remains incomplete and separate independent review controls source acceptance.

## 2026-10-21: reviewer-2 shell hygiene pass

PR #38 review findings fixed on `crates/termrock-shell` and the facade's
`blocks::shell`, behaviour-preserving, one commit per finding
(`72d38dbc7`..`a7ebefdcf`):

- Facade `blocks::shell` re-exports `termrock_shell::{Chrome, Regions,
  AppShell}` instead of carrying a verbatim copy (two same-named
  `AppShell` types existed); the header/gap/footer/inset chrome builders
  moved into `termrock-shell`'s `AppShell`, and the facade keeps only
  `EmptyLeaf`, `AppShellSlots` and the `AppShellExt` slot-builder trait.
- `termrock-shell` no longer force-enables `termrock-runtime`'s
  `testing` feature in `[dependencies]`; it is dev-only, so
  `runtime::stub` stays out of production builds (`cargo tree -p
  termrock -e features`: one `testing` edge before, none after).
- `reach.rs` (the 723-line reachability harness) compiles only under the
  crate's default-off `testing` feature; a self dev-dependency keeps the
  crate's own 49 tests green under `cargo nextest run -p termrock-shell`.
- `keyboard::observe` now ports PR26's Alt arm as well as Super, so
  `alt_delivered()` can be proven by observed Alt chords (regression
  test fails against the old observe: 4/5).
- `KeyRouter::once` folds a passing layer's response with
  `out | Response::consumed()` instead of returning a fresh consumed, so
  the layer's invalidate/state payload survives; `route`'s doc no longer
  claims hover suppression (the runtime owns hover on parity).
- `PageLeaf::measure` follows the sibling `min: 0` convention instead of
  `min: constraints.max`; no consumer relied on the outlier.

Gate on the exact pushed HEAD (`a7ebefdcf`): `cargo nextest run -p
termrock-shell -p termrock` 71/71, `cargo clippy --all-targets -p
termrock -p termrock-shell` clean, `cargo fmt --check` exit 0,
`cargo check --workspace` exit 0.
## Modal backdrop DIM adjudication — 2026-10-21

Decision record: `docs/audits/2026-10-21-modal-backdrop-adjudication.md`
(independent adjudication of
`docs/audits/2026-10-21-modal-backdrop-dim-vs-frozen-baseline.md`).
Classification: **NO-DEFECT** for parity's `Theme::backdrop_style`
(`crates/termrock-theme/src/lib.rs:237`) and `Paint::modal_backdrop`
(`crates/termrock-runtime/src/ui/paint.rs:459`); **REFERENCE-DEFECT** for
the frozen expected frames. The tag source (4169783ae) contains exactly one
backdrop mechanism — `Theme::backdrop` sets `Modifier::DIM` on nonblank
cells iff `fg == bg` or the level is Ansi16/Mono (`src/theme.rs:1243`,
introduced by `d720a1c2e` on 2026-10-17), and both the showcase Dialog
(`src/widgets/dialog.rs:1264`) and the tablepro Picker path
(`src/ui/modal.rs:425`) route through it. Fresh runs of the freshly built
frozen binaries (showcase ca3a3114…, tablepro 9c82d9af…, provenance
matching `controlled-baseline-a`) confirm it live: dialog-open 920 dim
cells at 16/none and 0 at 256/truecolor; picker-open full-underlay DIM at
16/none, collision-only cells at 256, none at truecolor. The conflicting
frozen artifacts are defective data, not divergent mechanisms: the
snapshots bundled in the tag were recorded at `4a79c0a2d`, before
`d720a1c2e` existed (dialogs show the pre-fix no-DIM look at 16/none), and
the admitted corpus replay additionally dropped the picker underlay DIM at
16/none/nocolor (4656 dim cells in the raw stream vs 83 in the corpus
frame). Consequently `backdrop_style` stays unchanged, the reverted
no-DIM experiment stays reverted, and the green mono digest
`quick-switcher 120 40 junie mono af463bd8d299cf6b` encodes the correct
mechanism — it must not be re-blessed to a no-DIM value. Owner-gated
follow-up: regenerate the affected expected frames (showcase
`dialogs/open` and tablepro `overlay-picker-open` /
`overlay-safemode-picker`, tiers 16/none/nocolor) from the tag-source
build through the pinned runner and add a modifier-aware
(`mods.dim`) parity check to the corpus admission; parity dialog-open
captures that fail on 16/mono backdrop DIM are correct failures against
defective data.

## Backdrop frame regeneration + modifier-aware corpus check — 2026-10-21

Executes the adjudication's owner-gated follow-up. Decision record:
`docs/audits/2026-10-21-backdrop-frame-regeneration.md` (receipts per tier:
`docs/audits/2026-10-21-backdrop-frame-regeneration-receipts.json`).
Re-verified from the frozen binaries (showcase `ca3a3114…`, tablepro
`9c82d9af…`, provenance-matched) before touching anything: dialogs/open
80x24 DIM present at 16/none/nocolor, absent at 256/truecolor (the
adjudication's 920 count is a trim-metric variant, same signature); the
picker "1847" figure was a Ctrl+P capture — `TpCmd::Switcher`, a different
overlay — while the ctrl-o picker_open fresh capture is full-screen DIM
(4660 cells / 59 runs), matching the raw tag snapshot class (4656/156).
The tag-era `snapshots/` picker frames are byte-identical to fresh frozen
captures: the snapshots were never defective, only the corpus conversion
was. Regenerated 33 corpus tiers (dialogs/open 80x24, overlay-picker-open
and overlay-safemode-picker at all five sizes; tiers 16/none/nocolor)
through the pinned runner, re-approved the matching snapshots tiers to
committed `.ansi`-only, refreshed `corpus-index.json` +
`admission-record.json`, and preserved every replaced artifact under
`baselines/regen-sidecars/2026-10-21-backdrop-frame-regeneration/`.
Truecolor/256 corpus tiers were not defective and are untouched; dialogs
truecolor snapshot drift vs the tag build is page-content drift in the tag
build itself, recorded not fixed. Hardening: `frozen-baseline corpus-check`
(`tools/frozen_baseline.py`) decodes per-cell SGR modifier state from each
`<tier>.ansi` and fails closed against the frame JSON `mods`; because the
defective conversion was self-consistent, a cross-store pass against the
conversion-independent snapshot dump (via the manifest `legacy_name`) is
what detects the class — run store-wide it flags ~1170/1510 scenarios
(dropped DIM or bold; 5 `showcase/diff/fade-wheel` tiers fail even
intra-store), a defect surface far larger than the picker families, to be
regenerated by their owners. Harness/conformance store gates green at the
worktree base. Tier cases re-measured at base and at the branch tip
(ff0005a84): 78 still red (`showcase::pages::dialogs`,
`showcase::flows::dialogs::open::80x24`, `tablepro::overlays` pickers;
control without the regeneration: 75 — the three extra are the
previously-skipped dialogs/open tiers now comparing), each failing with
the candidate missing or differently-mapping the dimmed backdrop. That is
the app/theme streams' still-open parity gap to close in source; the
regenerated frames are correct against the frozen oracle and need no
further edits when it lands.
