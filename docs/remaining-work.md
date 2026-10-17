# Remaining work on `design/interaction-polish`

Status: 2026-10-17, derived from the repository at `42f0b372e`
(`origin/design/interaction-polish`). This is the one list of everything
started on this branch and not finished. Every item names the document and
section it comes from; when that document and this list disagree, fix the
document or this list in the same commit, never only one of them.

Sources read: [`architecture/shared-components-plan.md`](architecture/shared-components-plan.md)
(§4 step table and "Step status", every "S*n* outcome", deviations and
"Notes for later steps"; §7 SC track, §7.3, §7.5, §7.8),
[`audits/design-review-2026-10/`](audits/design-review-2026-10/README.md)
(every file), [`visual-testing/plan.md`](visual-testing/plan.md),
[`visual-testing/tuiscotti-migration.md`](visual-testing/tuiscotti-migration.md),
[`upstream/`](upstream/README.md), [`dependencies.md`](dependencies.md),
[`ci.md`](ci.md), `.velnor/config.toml`, `tests/showcase_coverage_allow.txt`,
`snapshots/` (file counts) and `src/runtime.rs`.

## 1. Goal and "done when"

Goal: finish the shared-components refactor and the design-review backlog
on this branch, approve the branch's own visual oracle, and leave the
branch ready to port to `main` (termrock) 1:1 (visual-testing plan §F).

The branch is done when all of these hold:

1. Every S-step (S8a–S27) and the theme track T in the plan's §4 / §7.3
   table is "Done" in "Step status" / §7.8, with its `rg` contract checks
   at 0 hits and its deviations recorded.
2. Every SC-step (SC30–SC57) is "Done" and
   `tests/showcase_coverage_allow.txt` holds only `permanent` lines
   (today: 226 pending, 12 permanent).
3. Every finding in `docs/audits/design-review-2026-10/` is "Fixed" or
   recorded with a reason the owner accepted; no "open", "recorded, not
   fixed" or "owner question" entry is left without a decision (§2).
4. Oracle v0 is imported (`baselines/oracle/ORACLE.toml`
   `pending = false`), `snapshots/` is removed with `git rm -r`, no
   scenario holds tuisnap's four files, and the narrow tier and the
   `.pending()` coverage rows are approved or deleted with a reason.
5. CI runs `visual-smoke` on push and pull request as a required job,
   `visual-full` and `visual-narrow` nightly, and `Publish baseline` is
   green on a push.
6. The full gate passes on the branch head: `cargo nextest run
   --all-targets`, `cargo clippy --all-targets`, `cargo fmt --check`, the
   tuiscotti pilot and smoke tiers (plan §4 "The gate (G)").

## 2. Owner decisions pending

Only the product owner answers these. Until an answer is recorded in the
source document, the current behaviour stays; work items that depend on an
answer say so.

| # | Question | Options | Current state and recommendation | Source |
|---|---|---|---|---|
| D1 | jackin `usage/overview`: the worst-window meter label keeps a fixed 10 cells, so `Weekly · Opus` reads `Weekly · …` while the status column has room at 120x40. | (a) keep the fixed 10 cells; (b) let the label measure its width through `ui::columns`, as the usage limits do since `d09f244c9`. | Fixed 10 today. Recommend (b): one width rule for both usage tables; it is a jackin re-approval of `usage/overview` at every size. | `audits/design-review-2026-10/visual-reapproval.md`, "Owner confirmation needed (jackin)", item 1 |
| D2 | A footer that runs out of room drops normal hints from the right while `Esc` and `?` stay pinned (`f0cb83a02`); `editor/mounts_dirty` at 160x50 drops `Ctrl+S Save` beside `• 1 change`. Should `Ctrl+S` be pinned while a screen has unsaved changes? | (a) keep: only `Esc`/`?` pinned; (b) pin the save binding while the page reports dirty state (a footer rule in the library, S8b `Footer`). | Not pinned today. Recommend (b), as a library `Footer` rule ("pin the commit action while dirty"), not a jackin special case. | same, item 2 |
| D3 | Forms validate on Save: `accounts/add_form_required` now shows the Save outcome (the field marked, focus on it) instead of a validated Enter on the field (`2a71f8dcf`). | (a) Save-time validation is the standard for every `FormDialog`; (b) validate on Enter per field (the baseline). | Save-time today. Recommend (a) (macOS sheets validate on the default button; SI24 "commit on selection change, ask only on invalid"); per-field `ValidateWhen` stays an override (S11). | same, item 3 |
| D4 | Fixed-choice pickers (`manager/launch_picker`, `hard_launch_picker`, tablepro Safe Mode picker, holla Activities) mark their cursor with a row fill and no `▎`, so no focus bar shows while one is open. The baseline drew the bar. | (a) confirm the menu exception covers fixed-choice pickers and add that wording to AGENTS.md "Visual states" (the list-owned-by-an-owner rule); (b) give them `CursorStyle::Bar`. | Fill only today, locked by tablepro `the_switcher_is_the_only_layer_showing_focus` and showcase `the_chooser_stacks_a_picker_over_the_form_and_returns_to_it`. Recommend (a): it is the focus audit's recorded design (`focus.md` #1, "matching the menu"); then AGENTS.md says "a menu or a non-searchable picker marks its cursor with a fill". | same, item 4; `focus.md` #1 |
| D5 | jackin `quota_tone`: available windows are always healthy (`57562ede2`), so the domain's `QuotaStatus` (`from_pct` thresholds, overridable per window) alone decides a quota meter's level. | (a) the provider/domain status is the authority; (b) the library's `MeterTone::Level` 60/85 defaults decide. | (a) today. Recommend (a), and move `QuotaTone` into the library (work item S23-Q) so the rule lives once. | same, item 5; `colour-follow-ups.md` |
| D6 | Showcase header crumb: SC5 replaced the baseline's `/ Group / Page` with the library `Breadcrumb` (`Group › Page`, current page bold), re-approved as a deliberate visual change. | (a) keep `Group › Page` (one crumb style across apps); (b) restore `/ Group / Page` as a `Breadcrumb` separator option for the showcase. | (a) today. Recommend (a); confirm so the change against `visual-baseline` is accepted. | `shared-components-plan.md` "SC5 outcome" (Shell line) |
| D7 | crossterm 0.29 input starvation: the runtime re-arms the reader (`tcsetattr TCSANOW`) in `src/runtime.rs` `poll_input`/`tty_input::rearm`. File the bug upstream? | (a) file a crossterm issue (reader should read until `WouldBlock`) and track it in `docs/upstream/`; (b) keep the workaround untracked. | Not filed. Recommend (a): `dependencies.md` already says "Drop the re-arm once crossterm reads until `WouldBlock`", which needs an upstream fix to ever happen. | `dependencies.md` (crossterm paragraph); `src/runtime.rs` `poll_input` doc, `rearm` |
| D8 | `visual-baseline` tag replacement: when (and whether) the branch's oracle replaces the frozen tag. | (a) keep the tag frozen and let `baselines/oracle/` (v0 then v1) be the reference; (b) the owner creates a new tag or release after oracle v1. | Agents never touch the tag. Recommend (a) until oracle v1 and M7 sign-off; the owner alone acts on the tag. | `visual-testing/plan.md` D4, E1, §F4 M7 |

Already decided by the owner (not open): holla captures keep
`HOLLA_NO_HISTORY=1` (`visual-reapproval.md`, 2026-10-17); jackin's intro,
outro and handoff fade stay jackin-only (allowlist `permanent` lines,
2026-10-16 and 2026-10-17).

## 3. Work items

Ids: **S**/**SC** steps keep the plan's ids; **CO** a carry-over from a
done step's deviations or notes; **V** visual oracle and migration; **A**
an audit follow-up; **U** upstream and dependencies; **C** CI. Within each
group, items are ordered by dependency, then priority.

### 3.1 Plan S-steps (shared-components-plan.md §4 and §7.3)

Every step: expand, migrate, contract; the gate G; each drift fix its own
commit with a test that fails on the parent; each visual change named and
re-approved in its own commit (§4 "The rules that go with the gate"). The
paired SC step lands in the same change (§7.6, group SC-F). "Findings"
are the ids in the §4 table (M mouse, K keyboard, SI/SV standards, SIg/SVg
gaps).

Unblocked now: S8a, S9, S10, S11, S12 (their dependencies are done) and
the theme track T. The §4 groups F–K still apply; the "Depends on" column
below is the dependency each step must wait for.

| Id | What (expand; contract) | Findings it closes | Depends on | Acceptance beyond G | Demo |
|---|---|---|---|---|---|
| S8a | `shell::{KeyRouter, ShellKeys, ShellAction}`, `Page::on_interrupt`; delete the 4 `on_key` functions, `esc_ladder`, `workbench_chord` | K11, K20, K21 (remaining apps), K26, K28, K35, K37, K38, SI10, SI16, SI23; CO1, CO2, CO9 | S7 (done) | one cross-app key-order table test | SC33 part, SC46 |
| S9 | `SelectGesture`, `RowSelection::apply`, `ClickPolicy`, `Activate`, `CheckClick`; delete List `checked`/`anchor`, Grid `selected_rows`, OptionEditor `checked`, tablepro `connections.rs` empty double-click branch, holla `disk.rs` toggle-on-click | M9, M10, M15, M16, M32, SI5, SI14, SI24, SIg2; CO10 | S4, S6 (done) | the M15 harness (holla Disk) flips; a selection-model table test | SC33 |
| S10 | `ui::row::RowUi`, `RowPaint::{mark, label}`, `RowLook`; delete 11 painting skeletons | K12, SI8, M25, SV14/SVg (visual), SV1 (visual), SVg3 | S4 (done) | digest unchanged before the visual commits; a contrast test per new token | SC33 |
| S8b | `HasHints`, `Bindings`, `HintCx`, `shell::Footer`, `HelpOverlay::from_keymaps`, `HelpSection`, the component docs generator (`cargo xtask docs components`, §5; `docs/components/` does not exist yet); delete 4 `draw_footer`, modal hints in T/H/J, 4 help renderers, 16 `"Enter","Edit"` literals | K10, K23, K25, K39, SI7, SIg4, SVg10 (visual); the "no keyboard-only state" coverage test; CO3, CO4, CO5; D2 if answered (b) | S8a, S4 | hint snapshot per widget; coverage test walks every page's `KeyMap`; `docs_components_fresh` test | SC33 |
| S11 | `widgets::edit::EditField`, `Validate`, `Validator`, `ValidateWhen`, `FieldError`; `PointerField` on `EditField`; contract `TextValidator`, `table::CellValidator`, `grid::Validator` | M2, M21, M31, M39, K1 walk check (`DEFAULT_EDIT_ENTRY` is already `Immediate` since 1da20cc01), K5, K7, K17, K36, SI6, SIg5, SIg9; Select All in picker/combo/find queries; D3 | S4, S6 (done) | the K1 walk (a caret at every text stop); the M2 CodeEditor drag test; intended re-approval of field captures (caret shown) | SC32 |
| S12 | `widgets::option_list::OptionList`, `Item`; bodies of Select, OptionEditor, Picker, Completion, ContextMenu; tablepro `PickerItem`/`CompletionItem` literals | M19, M38, SI13, SI21/SV15; tablepro tab-list glyph mismatch (`app.rs:1655` vs `1682`); CO6, CO7 | S3, S4, S5 (done) | popup journeys; wheel tests in all five | SC32 |
| S13 | `ListCore`, `ContentState`, the standard Edit `ContextSource`; List, Tree, Props, Steps on `ListCore` | M3, M6, M22, M30 | S9, S10, S11 | a right-click test on every text widget | SC34 |
| S14 | `shell::{Router, Nav, Chrome, Breadcrumb, Clipboard}`, `App::stakes`, typed menu `Cmd`; delete S `goto`, T `Screen`, H `go`, J `go`/`Screens`/`ROUTES`, label-string re-dispatch (H:1569, J:1667), `copy_osc52`, `request_quit`/`open_quit_confirm` ×3 | M5, M7, M8, M23, M40, K4, K6, SI17, SIg3 | S7, S8a | quit and leave journeys per app | SC34 |
| S15 | `Component`, `Draw`, `Measure`, `Container::visit`, `AnyComponent`; builder renames; delete `MouseTarget` adapters, the `Outcome` adapter, free render fns (`empty::render`, `props::render`, `keyhint::render*`), per-widget `on_*` | M28, K19; CO8 | S6–S13 | `rg 'fn on_click\b\|fn on_wheel\b' src/widgets` = 0; `WidgetDoc` required by `Component` (§5) | — |
| S16 | `Column::marker`, `RowMark`, `ListItem::{header, status}`, `TreeNode::{meta_tone, trailing, label_tone}`, `TreeView::columns`; S16a library, S16b holla rows, S16c jackin rows and trees, S16d tablepro EXPLAIN tree-table | 69 screen list-key arms; CO11 | S10, S13 | per-app captures pixel-equal; the jackin tree hit test | SC35 |
| S17 | `PageHeader`, `Heading`, `ActionRow`, `DetailCard`, `NavList` (moved), `Notice`, `MeterList`, `Tabs::{layout, passive}`, `TextInput::readout`, `Item` builder | M34; CO12 | S8b, S10 | captures unchanged | SC36 |
| S18 | `FormPanel`, `FormField::pair`; tablepro connections form and filter editor, showcase forms | K6 | S11, S17 | form journeys | SC37 |
| S19 | `MasterDetail` (14 sites, `SPLIT_MIN`) | M20, K18 | S17 | narrow-tier captures | SC38 |
| S20 | `EditorShell` (jackin settings and editor) | K4 (leave guard) | S14, S18 | editor journeys | SC39 |
| S21 | `FilterTree`, `FilterList` | — | S11, S16 | filter journeys | SC40 |
| S22 | `CommandMenu`, `ContextAnchor::row`, `FileBrowser` + `FileSource`, `StepLine`/`Wizard`, `FindState` + `line_numbers`; showcase `pages/row_menu.rs` (`RowMenu<T>`) and its 7 users; jackin `modals.rs:30-526`; 12 wall-clock spinner sites; jackin dead code | `showcase-pages-navigation.md` SC16 1 | S5, S11, S13 | jackin and holla journeys | SC41 |
| S23 | `Tabs` preview and pinned tabs, `FooterNote`, `ActivityLine` | apps inventory 5, 41, 50, 51 | S2, S17 | §7.3 rules | SC42 |
| S23-Q | Move jackin `QuotaTone`/`quota_tone`/`meter_tone` (`src/bin/jackin_preview/screens/accounts.rs:2036-2068`, used by `capsule.rs:2691`, `usage.rs:22`) into the library meter API (a domain-level tone a `Meter` takes); no app keeps a tone rule | known item (owner brief); D5 | S23, D5 | jackin usage/accounts/capsule captures unchanged; a library unit test per level | SC42 or SC36 (`MeterList`) |
| S24 | `TreeView` tri-state checks and trailing action row; `DataTable` group rows, match ranges, row marks, group folds; `StepRail` table variant | apps 57, 59–62; CO13 | S16 | §7.3 rules | SC43 |
| S25 | `PickerChain` (jackin OpFlow `modals.rs:551-948`, holla go-to-path) | app 34 | S12, S22 | §7.3 rules | SC44 |
| S26 | `PaneTree`, zoom (`Split::toggle_max`/`Maximized`); no cross-fade (owner decision 2026-10-17) | apps 8, 69 | S2, S19 | §7.3 rules | SC45 |
| S27 | Strategy B nav policies `core::nav::{FocusPolicy, AdjacentMove, AdjacentKeys, Scope, Bounds}` on Tabs, TreeView, ListBox, Select, Picker, ContextMenu/MenuBar, Dialog, Toolbar; `KeyRouter` honours `AdjacentMove` | `showcase-pages-navigation.md` SC28 6; Strategy B rows of `showcase-coverage-library.md` | S4, S7, S8a | §7.3: a widget test per `AdjacentMove` variant at both edges, a cross-app key-order test, `Scope` inside a modal without a trap, digest unchanged | SC57 |
| T | Theme track: plane-relative `text_faint` (SV3), mono hover and press (SV2), panel title marker (SV4), `border_strong` field outlines (SVg1), one pressed direction (SVg2), success vs accent (SV14), `COLORFGBG` (SVg7) | all visual, each re-approved alone | S0 (done); can run now | contrast test on every plane | — |

### 3.2 Carry-overs from finished steps

Deviations and "Notes for later steps" of S0–S7 that are still open in the
code. Each is done inside the step named.

| Id | What | Source | Done in | Acceptance |
|---|---|---|---|---|
| CO1 | Call `keymap::unassigned_chord` once in `KeyRouter` before the focused widget, then delete the per-widget calls. | plan "S4 deviations" 2; "S4 notes" S8a | S8a | `rg 'unassigned_chord' src/widgets` only in the router |
| CO2 | Decide where page letter shortcuts sit relative to a focused list (type-ahead consumes a miss, so a page letter no longer reaches the page while a list has focus); record the rule. | "S4 notes" S8a | S8a | a key-order test row for a letter over a focused list |
| CO3 | Hints stop advertising `j/k`, `g/G` unless `NavProfile::current()` is `Vim` or `nav.vim`; advertise "Type to jump" where `type_ahead` is on; `Completion::NAV` and the searchable Picker read from the widget. | "S4 notes" S8b | S8b | hint snapshot per widget under Desktop and Vim |
| CO4 | Library modals' fallback footer hints (Picker, Choice, Form, Info, Help) move from jackin `layer_hints` (`src/bin/jackin_preview/app.rs:1520`) and holla's downcasts into `Layer::hints`; a visual change in tablepro, holla and showcase footers, re-approved. | "S5 deviations" 3; "S7 deviations in part 2" 3 | S8b | `rg 'fn layer_hints' src/bin` = 0 |
| CO5 | `shell::Footer` calls `Shell::footer_drawn` so no app touches `HintKeys::register`; error-over-hints is footer composition. | "S7 notes" S8b; "S2 deviations" 3 | S8b | `rg 'HintKeys::register' src/bin` = 0 |
| CO6 | `OptionList` holds a `Cursor`, implements `RowSource for [Item]`, gets a `ScrollPane` (`draw_v`) and the `Overlay` for Select, Completion, ContextMenu and ChoiceEditor; delete `Select::bar`, `bar_track`, `v_click`, the `Shown`/`[MenuItem]`/`[PickerItem]` adapters, Picker/Completion `cursor_dirty`. | "S3 deviations" 4, 6; "S3 notes" S12; "S4 notes" S12; "S5 deviations" 7 | S12 | the five popups share one wheel and bar path (wheel tests) |
| CO7 | `ChoiceEditor` gets a `MouseTarget` through `OptionList`; the showcase Cell editors page stops calling `ChoiceEditor::on_wheel`; widgets stop ignoring `Phase::Secondary` and `RowMenu`/jackin capsule menus stop routing by id. | "S6 deviations" 9; "S6 notes" S12 | S12 | `rg 'ChoiceEditor::on_wheel\|\.on_wheel\(' src/bin` = 0 |
| CO8 | Delete `PartMap` (DataTable, DataGrid), the 14 `WidgetId::item_of` helpers, `contains_id`, `pointer_in`, the per-gesture `Layer` handlers and the id-taking `on_click`/`on_press`/`on_drag`. | "S6 deviations" 8; "S6 notes" S15 | S15 | `rg 'PartMap\|item_of\|contains_id' src` = 0 |
| CO9 | holla's background tabs drop their tick and message requests (only blur statuses wait); unify with the shell's background-request policy. | "S7 deviations in part 2" 4 | S8a | a holla test: a background tab's request is applied when shown |
| CO10 | Double click is a per-screen `count ≥ 2` check (holla finder/files/disk/plan/activity, jackin manager/capsule); hand-drawn list bars page on click (`scroll_press`, `offset_for_click`). | "S6 notes" S9 | S9 (activation), S16 (bars) | `rg 'count\(\) >= 2' src/bin` = 0 |
| CO11 | Screens still scanning derived ids or drawing lists by hand move onto widgets: holla disk, files, finder, plan, review, snapshot, cleanup; jackin manager tree, accounts tree, capsule panes and seams, config, editor, inspect, settings, usage; tablepro `app.rs`; showcase Scrolling/Chrome/Menus/Buttons/EmptyState actions, NavList; drop `render_with_fade` and holla `scroll_press`/`scroll_drag` there. | "S1 notes" S16; "S3 deviations" 7, "S3 notes" S16/S17; "S6 notes" S16 | S16, S17 | `rg 'scroll_press\|scroll_drag\|\.child\(' src/bin/<app>/screens` = 0 per app |
| CO12 | `ui::text::center_x` adopters (jackin capsule, cockpit, prelude, rain centre by `chars().count()`) move with their screens. | "S2 deviations" 7 | S17/S22 | no `chars().count()` centring in `src/bin/jackin_preview` |
| CO13 | holla plan header (`src/bin/holla/screens/plan.rs`, "k of n done · running · blocked · failed") builds and tones its clauses by hand; draw `progress::Tally` with a blocked count. | plan "SC21 outcome", "Recorded, not migrated" | S24/SC50 (or the next touch of plan.rs) | holla plan captures unchanged except the failed-clause tone |
| CO14 | tablepro dialogs still use `outer_margin` (`src/bin/tablepro/app.rs:99`, `:1431`); move to `Dialog::chrome(top, bottom)`. | "S5 deviations" 4 | S14 or S17 | `rg 'outer_margin' src/bin` = 0; tablepro captures unchanged |
| CO15 | jackin row context menus belong to screens, not the layer stack: a K14 pass-on key closes the menu but does not reach the page (`editor_shift_f10_opens_under_the_cursor_row_and_removes`). | "S5 deviations" 6 | S12 or S22 | a jackin test: a pass-on key reaches the page |
| CO16 | Type-ahead takes letters and digits only, so a label starting with punctuation (`.config`) cannot be typed to; record if a screen needs it. | "S4 deviations" 3 | S16 (screens) | either a screen test or a recorded "not needed" |

### 3.3 Showcase coverage track (shared-components-plan.md §7)

SC0–SC29 are done. Acceptance for every SC step is §7.4: harness tests for
every option and state (Inspector and `--opt`), the digest regenerated
(`UPDATE_BASELINE=1`) with each changed page listed, `.pending()` visual
rows for new surfaces, the step's allowlist lines deleted with the gate
green, no app-local code in `src/bin/showcase`, and the §7.8 row updated.

Allowlist today (`tests/showcase_coverage_allow.txt`, `id | reason | step`):
238 lines, 12 `permanent`, 226 pending: SC34 172, SC31 43, SC30 5, SC32 4,
SC33 1, SC41 1.

| Id | What | Depends on | Allowlist lines | Source |
|---|---|---|---|---|
| SC-G1 | Extend the coverage-gate scan (`src/bin/showcase/catalog_scan.rs`) to public non-builder functions and methods: free `pub fn` (e.g. `keyhint::render_status`, `src/widgets/keyhint.rs:355`; `keyhint::render_toned`, `columns::solve`, `ui::text::*`) and `&self` methods (e.g. `Theme::status`, `src/theme.rs:1712`) fall into the scanner's `_ => {}` arm today. Add a `CapKind` for them, list them in `src/catalog.rs`, declare or allowlist each. | — | adds lines | owner brief; plan "SC0 outcome" (scan scope) |
| SC-G2 | **Done.** `TabCmd::ShowHidden` is shown (the Tabs keys readout names "↓ hidden tabs" while the strip has an overflow menu; `the_keys_readout_names_the_hidden_tabs_key_while_the_strip_has_a_menu`). Re-pointed with reasons: `TextInput::validate_on`, `ValidateOn` and its two variants to SC32 (no playground field is an untouched empty required field, so the two variants look the same until `ValidateWhen` lands on Inputs); `ChipCmd::ShowHidden` to SC33 (named only in the `+N` tooltip; the generated keyboard reference lists it; the Chips playground has approved captures). `Theme::centred_hints` stays on SC30, which shows it on Status & hints. The `type_ahead_timeout` lines moved under `# widgets::menu`; `# core::list` is gone. | — | 7 → 0 on finished steps | allowlist |
| SC30 | Real `Motion` Full/Reduced/Paused, `TooSmall` demo, `StatusLine` TTL, `CommonArgs`, `ui::text` helpers | S2 (done): can run now | 5 | §7.5 SC-F table |
| SC31 | Stacked modals through `ShellLayers`/`ModalStack`, `Dismiss::{focus_out, refuse_feedback}`, `Frame::close_button`, `DismissReason` in the State log; fix the recorded SC10 finding (a select's list closes on dead space under `Dismiss::ESCAPE` because `Select::close_on_blur`, not `Dismiss::focus_out`, decides blur) | S5, S7 (done): can run now | 43 | §7.5; "SC10 outcome" recorded finding; `interaction.md` "SC10 finding" |
| SC32 | `EditField`/`Validate`/`ValidateWhen`/`FieldError` on Inputs and Dialog kinds; `OptionList` `WheelMode`, `SelectArrows`, `Item` reason | S11, S12 | — | §7.5 |
| SC33 | `HelpOverlay::from_keymaps` replaces the hand-listed `?` (SC6 note 1), `Footer` precedence, `HasHints` in State tab, generated `KeyReference` incl. `GridCmd`, `ClickPolicy`/`Activate`/`CheckClick`/`SelectGesture`, `RowUi`/`RowLook`/`CursorStyle::Bar`, peek line | S8b, S9, S10 | — | §7.5; `showcase-pages-navigation.md` SC6 1 |
| SC34 | `ContentState` in every list-like States grid, Edit context menu on text widgets, the showcase shell on `Shell`/`Router`/`Chrome` with a clickable crumb and page stack, truthful clipboard results, Chrome toolbar, the State-tab dispatcher readout and widget action logs, `MenuBar::type_ahead_timeout` | S7, S13, S14 | 172 | §7.5; allowlist reasons |
| SC35 | Row and tree extensions demo | S16 | — | §7.5 |
| SC36 | `PageHeader`, `ActionRow`, `DetailCard`, library `NavList` (and `SidebarRail` on the Sidebars page, SC5 1), `Notice`, `MeterList` page, `Tabs::{layout, passive}`, `TextInput::readout`, `Item` builder | S17 | — | §7.5; `showcase-pages-navigation.md` SC5 1 |
| SC37 | `FormPanel`, filter editor through `FormDialog` | S18 | — | §7.5 |
| SC38 | Master-detail page (and the narrow drawer, SC5 2), `SplitKeys` | S19 | — | §7.5; `showcase-pages-navigation.md` SC5 2 |
| SC39 | `EditorShell` on Settings | S20 | — | §7.5 |
| SC40 | `FilterTree`/`FilterList` | S21 | — | §7.5 |
| SC41 | `CommandMenu`, File browser page, `StepLine`/`Wizard`, `TextViewport::find`, `ContextMenu::type_ahead_timeout` | S22 | 1 | §7.5 |
| SC42 | Preview and pinned tabs, `FooterNote`, `ActivityLine` | S23 | — | §7.5 |
| SC43 | Tri-state checklist tree, trailing action row, group rows, match ranges, row marks, plan outline | S24 | — | §7.5 |
| SC44 | `PickerChain` | S25 | — | §7.5 |
| SC45 | Pane tree page (no cross-fade) | S26 | — | §7.5 |
| SC57 | Strategy B options on Focus & navigation and Toolbars, `toolbars.adjacent` | S27 | — | §7.3, §7.5 |
| SC46 | Screen: query workbench | S8a, S16, S17, S19, S21, S23 | — | §7.5 "Screens" |
| SC47 | Screen: connections | S17, S18, S19, S21 | — | same |
| SC48 | Screen: finder | S11, S17, S19, S24 | — | same |
| SC49 | Screen: disk cleanup | S16, S17, S24 | — | same |
| SC50 | Screen: plan runner (with CO13) | S23, S24, S26 | — | same |
| SC51 | Screen: activity | S2, S14, S17, S22 | — | same |
| SC52 | Screen: workspaces | S16, S19, S26 | — | same |
| SC53 | Screen: accounts and usage | S17, S19, S25 | — | same |
| SC54 | Screen: editor shell | S20, S24 | — | same |
| SC55 | Screen: launch cockpit (no handoff fade) | S22, S26 | — | same |
| SC56 | Screen: read-only report | S17 | — | same |

Housekeeping: §7.8 lists the SC6 and SC15 rows twice; merge them when the
next SC row is updated.

### 3.4 Visual oracle and migration (visual-testing/plan.md, tuiscotti-migration.md)

Counted from `snapshots/` at `42f0b372e`: 7,675 `.ansi` files; 1,650
scenarios still hold tuisnap's four files (`.ansi`, `.txt`, `.html`,
`.png`), all in `showcase/`: `flows` 1,150, `fade` 250, `audit` 125,
`hover` 75, `resize` 50. tablepro (1,100), holla (3,125 accepted) and
jackin (1,200) are re-approved (`visual-reapproval.md`); the migration
doc's status table carries these counts since V9.

| Id | What | Depends on | Acceptance | Source |
|---|---|---|---|---|
| V1 | Re-approve the 1,650 legacy showcase scenarios group by group (`flows/*` per page: datagrid 100, chrome 100, settings, pickers, editor, diff, dialogs 75 each, …; `fade`, `audit`, `hover`, `resize`) with `VISUAL_FULL=1`, review, attribute every changed cell, `tuiscotti accept`; never rewrite files in bulk. Their `.html` files carry absolute paths (tuiscotti P7) and go with them. | ideally after the S-steps that move showcase pages, to approve once | `find snapshots -name '*.txt'` empty | `tuiscotti-migration.md` "Legacy four-file scenarios"; `tuiscotti-feedback.md` P3, P7 |
| V2 | Approve the `.pending()` coverage rows: 29 element, 7 overlay and 10 resize roots, 92 overlay dismissals (3,450 captures), including the quit-guard `OVERLAYS` rows and the 15 stale-step roots. | V1 for showcase rows | rows gate (no `.pending()` left, or deleted with a reason) | `tuiscotti-migration.md` status table, "stale steps", quit-guard rows |
| V3 | Narrow tier (`40x12`, `60x18`, 1,056 captures): fix the targets it misses (`jackin/cockpit/debug`, `jackin/accounts/filter` and others), then approve. | S19 (narrow layouts) | narrow tier approved; `visual-narrow` suite green | `tuiscotti-migration.md` narrow tier; plan T12–T16 row |
| V4 | Overlay coverage: 67 of 108 overlay sites have no `OVERLAYS` row (open plus both dismissals); add them per app. | — | `overlay_sites` reports 0 gaps | plan Progress T15; §C2 |
| V5 | Complete the element inventories T12–T14: a hover scenario for every interactive element, pressed frames that differ from hover and rest at all 5 levels, disabled hover/press equal to rest. T16: approve jackin's resize rows. | V2 | plan §H T12–T14, T16 acceptance | plan §H |
| V6 | T17: port `holla_pty.rs` and `terminal_suspend.rs` to tuiscotti; no hand-rolled `openpty` remains. | — | `rg openpty tests` = 0 | plan §H T17 |
| V7 | T9: measure per-capture time on a real CI run and update §C3 (still estimates). T10: holla-fable-audit workflows, `grep -r "cargo test" .github` empty. | C-items run | numbers from a real run | plan §H T9, T10, §C3 |
| V8 | T18 oracle v0: after the UI work lands, capture everything with `VISUAL_FULL=1` on ubuntu-24.04 (now through the `visual-capture` suite, T8 done), `cargo xtask visual import <actual dir> --oracle --source <commit>`, review the change list per group, commit per app, then `git rm -r snapshots/` in the same series. | V1, V2 (or their rows deleted), the S/SC steps that change captures | smoke and full green on HEAD; `snapshots/` gone; `ORACLE.toml` `pending = false` | plan Progress T18; §B4; §H T18 |
| V9 | **Done** (audit-sync branch, 2026-10-17): plan Progress rows T0/T4/T18/T30/T31, D1, §B5, §D and §H T0 updated; migration status table and legacy paragraph at 1,650; `tuiscotti-feedback.md` P3 row. Was: plan Progress T18 row says "T8 is still blocked" (T8 done); §H T0 still "in flight"; §B5 and §D name `cargo xtask visual accept` (now `tuiscotti accept`); T30 (upstream U1–U8) is done through tuiscotti PR 11 but unmarked; T31 is not in the Progress table; migration status table counts (6,000 / 5,975) predate the holla and jackin approvals. | — | the docs match the repository | plan.md lines 11, 20, 31, 174, 256, 430, 460–461; `tuiscotti-migration.md` status table |
| V10 | Framed panel title cut below `title_min`: `Panel::title_row` counts the frame padding once; fix to `title_min + pad + 1` so the meta takes its short form. Moves framed title rows in all four apps, so it needs its own re-approval commit. | after V8 or with a planned re-approval of all four apps | regression test on the Panels States grid (24-column framed `Preview` with `1–9 of 24` keeps `Preview`); four apps re-approved | `layout.md` "SC22 finding (2026-10-17)" |
| V11 | T19–T21: close the design backlog (§3.5), cut oracle v1 and the `sync-oracle` PR to `main`, then `main`'s `visual.yml` and `visual-ratchet`. T22–T29 are work on `main` (out of this branch). | V8, §3.5 | plan §H T19–T21 | plan §H, order line |

### 3.5 Audit follow-ups (docs/audits/design-review-2026-10/)

Findings already placed in an S-step (the "Findings" column of §3.1) are
not repeated here; they close with that step. This section holds what no
step owns yet, and the status bookkeeping the audits still need (plan
T19: "audit files record each finding as fixed or with a reason").
Where a standard contradicts shipped code (standards-interaction 13, 14,
24; standards-visual 12), AGENTS.md already decides: the standard becomes
the default and the old behaviour an option, so these are work, not owner
questions.

| Id | What | Depends on | Acceptance | Source |
|---|---|---|---|---|
| A1 | 16-colour readability pass across all four apps: popup menu frames draw `DarkGray` on the `DarkGray` popover plane at 16 colours and `none` (invisible; also in the baseline); on the selected tab's DarkGray plane holla's running spinner `⠋` is 2.9:1 and a queued `○` 3.2:1, jackin's capsule tab number `1` 3.2:1 and the editor dirty `•` 2.4:1. Library fix: Tabs runs its extra inks through `Theme::legible` on the selected fill; popup frames get a distinct 16/none border ink. | one planned re-approval of all four apps (after V1 for the showcase) | every graphic ≥ 3:1, text ≥ 4.5:1, no `fg == bg` cell at 16 and `none` in the accepted captures; a contrast unit test per token | `visual-reapproval.md` holla and jackin "Recorded, not fixed" |
| A2 | **Done** (audit-sync branch, 2026-10-17): every audit file except `performance.md` (already complete) ends with "Status (T19 sync, 2026-10-17)"; each finding reads Fixed (commit), Open → owner, Superseded or Decision (reason); leftovers no step owned became A15–A24. Note: `api.md` #28 was not fixed as listed below, the apps still spell "Not in this preview" (A20). Was: give every finding a status line. No status at all: `color.md` (22), `states.md` (30), `focus.md` #1–3, #5–10, #12–16, `standards-interaction.md` verdicts 1, 3, 7, 8, 10, 13, 14, 16, 21–24 and gaps 1–3, 6, 8, 9, `standards-visual.md` verdicts 1–8, 10–12, 14–16 and gaps 1–8, 10, and the three `duplication-*.md` (mark each row "consolidated into the plan" with its step). Fixed elsewhere but not marked: `keyboard.md` #8, #9, #13, #14, #27, #30, #31, #32 (S5/S7 commits in the plan's Step status); `interaction.md` #1, #8, #10, #11, #14, #17; `api.md` #1, #11, #28 (`not_in_preview` is gone from `src/bin`); `hscroll.md` bar routing (S6); `colour-follow-ups.md` `TabItem::done` (now declared in `src/bin/showcase/coverage.rs`) and its stale "Visual (holla and jackin)" paragraph; the `keyboard.md` caret-pass list (tablepro, holla, jackin since re-approved). | — | each listed id reads "Fixed (commit)", "Open → S*n*/A*n*" or "Decision (reason)" | every file in `docs/audits/design-review-2026-10/`; plan "Step status" |
| A3 | Mouse "Open from this pass": the I-beam shows over a field's label, and TextViewport and diff text declare no I-beam. (The `+N` overflow menu half is fixed: `77fd861c6`, `Interaction::focused_under`, tablepro and chips tests.) | — | pointer-shape unit tests | `mouse.md` "Open from this pass" |
| A4 | Mouse "Left for later": holla `TextModal` (help, why, about) has no close button (#13; with layout #11, holla/jackin help onto `KeyReference`); tablepro filter editor flashes nothing on a refused click (#24, needs `Frame`; with S18); jackin cockpit's blocked stage explains itself only in the status line (#6); a two-part crumb (`Disk › Usage`) is one link (#5); `ScrollState::follow` is opt-in (#4); the showcase draws its footer before a dialog's barrier, so a dialog's hints are not clickable there (S6 #1; with S8b). | S8b, S18 as noted | per item a regression test or a recorded decision | `mouse.md` "Left for later" and S6 #1 |
| A5 | Layout "Not done": `Panel::render` returns `Rect` (#1, `try_render`); `status_min_share` instead of a cap, compact mode off (#6); `spacing.panel`/`spacing.popup` not wired (#12, its own re-approval); tables and grids do not use `scrollbar::split` (#14); `row_layout` callers without `RowOverflow::More` owners (#15); `Sections` only on the audited pages (#17); the quick switcher Picker centres in the whole screen (#4). | #12 after V8 | per item: fixed with a test, or recorded as a decision with a reason | `layout.md` "Not done, with reasons" |
| A6 | Keyboard deferred passes: tablepro workbench quit is still a destructive "Quit TablePro?" instead of Save / Discard / Cancel, and jackin capsule and cockpit keep their own exit dialogs (both with S14); viewports and lists do not report a selection through `KeyContext::selection` (`has_selection()`, with S8b); Cmd+Backspace and Option+Delete unbound; footers spell `y Copy` literally instead of `hint_key` (S8b); jackin editor hand-codes `j`/`k` (S16/S17); holla finder query keeps its own string (#36, HP02; S11); dialogs show the underlined mnemonic and need re-approval (V1). | S8b, S11, S14, S16 | per item | `keyboard.md` data-loss, Select All, macOS chords and type-ahead passes; #36 |
| A7 | Interaction deferred: `return_activates: Focused\|Default` (#20); Tabs opting in to `FocusRing::{Always, KeyboardOnly}` (#24) together with the focus audit's per-widget strip-gutter override (needs a replacement cue, WCAG 2.4.7); holla files and finder keep `Alt+Enter Alternatives…` as their menu hint (#13 round 2); navigation and modals requested by a background screen are not acted on (#15 round 2, with CO9). | S8a for #15 | per item | `interaction.md` #13, #15, #20, #24; `focus.md` decision log "Follow-up (not done)" |
| A8 | API remainders: `ui::text::{truncate, truncate_middle, fit, slice_cells, wrap}` still cut with a hard-coded `…` (about 300 call sites) (#26); grid and scrollbar position labels, CodeEditor `ln/col` and Meter `refreshing` need a formatter hook (#28); Dialog and Picker keep their own chrome (#3, a visual change). | — | the ellipsis comes from `Glyphs`; formatter hooks with defaults unchanged (digest unchanged) | `api.md` #3, #26, #28 |
| A9 | Copy #5: with legacy key encoding a terminal reports Ctrl+\ as Ctrl+4; add the shared key normalisation. | — | `core::keyboard` unit test | `copy.md` #5 |
| A10 | Colour rows with no follow-up: holla `activities/jump` toast "exit 0" in `success` and the URL in `link`; holla `docker/done` "reclaimed 18.4 GB" in `success`; jackin `manager/tree_expanded` unlabelled amber `•` ("dirty" in the meta or a tooltip); jackin Limits low run in `info` (app row); jackin running-instance panel metas need a per-part tone (`Panel::meta`, `manager.rs:1177, 1194`); tablepro `set_status` Fail has no caller; showcase Overview swatches and labels. The T19 sync found two of these probably done: the showcase Overview swatches and labels (`0610904df`, `e92b78fc6`, `157f9206e`) and the jackin Limits low run (`a942f1081`, `57562ede2`, `colour-holla-jackin.md` R4); confirm against the captures and drop them. | re-approval per app | each row in `colour-follow-ups.md` with a commit or a reason | `colour-holla-jackin.md` M/L/H rows; `colour-follow-ups.md`; `colour-showcase-tablepro.md` #9 |
| A11 | Standards gaps no step owns: no room shows as a full-width sheet, never clipped (SI1); app-level profiles and a keymap file (SI3); Alt+mnemonics for menus with F10 shown (SI10 beyond S8a); Tab cycling inside modals (SI22); copy under mouse capture (SIg3 beyond S14); wide-character truncation (SIg10 remainder); showcase tree naming "Chrome" → "App shell" (SV7); editor choice by option count, `▾` only on the active cell, `☐/☑` (SV8); backdrop dim at 16/mono and the footer showing the modal's hints (SV10); `Delete…` ellipsis only when more input follows and `Del` as the key name (SV12; contradicts shipped copy, so the standard becomes the default); DIM fallback (SVg4) and BOLD-as-bright (SVg5). | T, S8b as fitting | per item, with the standard named in the test | `standards-interaction.md`; `standards-visual.md` |
| A12 | hscroll follow-ups (the Shift+wheel knob is already one, `InputConfig::shift_wheel_horizontal`, since cd4515ddc): reveal for cut labels in Tree, List, Picker and DataTable (with S10); jackin capsule pane bars routed by position (`hbar()`). | S10 | reveal tests; a jackin pane bar routing test | `hscroll.md` follow-ups and "Still open" |
| A13 | Showcase navigation leftovers: primary press-drag-release on a button menu is not offered (`release_from` is private) (SC16 2); Toolbar, Tabs `+N`, ChipBar and MenuBar `»` each own their overflow menu, to move onto one (SC27 7); legacy `showcase/flows/settings/toggled_off` is re-recorded on purpose with V1 (SC27 1). | V1 for SC27 1 | one overflow-menu implementation; a press-drag-release test | `showcase-pages-navigation.md` SC16, SC27 |
| A14 | Showcase coverage notes: the Inspector is hidden below 100 columns and only reads state; there is no in-app colour switch (`--motion reduced` is SC30). | — | fixed or recorded as a decision | `showcase-coverage-library.md` notes |
| A15 | Terminal viewport reversed cells (SGR 7) still draw as a white flood (`fg canvas` / `bg text_primary`, `src/widgets/viewport.rs` `c.style.reversed`), the treatment the table and grid cell cursor dropped. Either map them through a theme resolver or record that reverse video in terminal content is content, not a state. | T if fixed | a viewport test on a reversed cell, or a recorded decision | `states.md` #27 |
| A16 | Paste is still routed by each app (`Input::Paste` in all four `app.rs`); move it into the shared input chain. | S8a (`KeyRouter` on `App::on_input`) | no `Input::Paste` match in `src/bin/*/app.rs`; a paste test through the router | `duplication-shell.md` row 16 |
| A17 | Header/body/footer `Rect` arithmetic is still done per app; no step names the `AppShell` regions (the too-small screen is shared since S2). | S14 (`Chrome`) | apps take their regions from the shell; no per-app split | `duplication-screens.md` F22 |
| A18 | The showcase's `describe_key` (`src/bin/showcase/app.rs`) formats keys itself instead of the library key label. | S8b | `describe_key` gone; labels from `core::keyboard` | `duplication-screens.md` F25 |
| A19 | `MenuBar::render` still calls `on_hover(live_hover())` (`src/widgets/menu.rs`), so drawing changes state; hover should arrive through the pointer router (S6) like every other widget. | S15 (render becomes pure) | a test that rendering twice without input leaves the menu bar state unchanged | `api.md` #11 |
| A20 | `Strings::not_in_preview` exists but no app reads it: holla `src/bin/holla/app.rs` and jackin `screens/capsule.rs` still hard-code "Not in this preview". | — | `rg '"Not in this preview"' src/bin` = 0 (tests aside) | `api.md` #28 |
| A21 | Undo in tables (SIg7 remainder): fields have Ctrl+Z / Ctrl+Shift+Z (`d5ad6ffb6`), but DataGrid undoes pending changes only with `u` and DataTable has no undo. | S11 (`EditField`) if it fits | Ctrl+Z / Ctrl+Shift+Z on a grid edit, with a test; `u` kept as an option | `standards-interaction.md` gap 7 |
| A22 | Automatic ASCII glyphs (SVg6 remainder): `Glyphs::ASCII` and `Strings::ASCII` are opt-in only; nothing selects them for `TERM=linux` or the Windows console. The plan's §4 "Already done" and S17 rows now say SVg6 is partial (was "SVg6 done, S17 verify only"). | T or S17 | a detection unit test (`TERM=linux` picks ASCII) with an override | `standards-visual.md` gap 6 |
| A23 | A non-searchable Picker ignores typed letters (no type-ahead); Tree, ContextMenu, DataTable and DataGrid already type-select. | S12 (Picker body on `OptionList`) | a type-ahead test on a non-searchable Picker | `interaction.md` #11 |
| A24 | Alt+click does not trigger the Picker's `ChosenAlt` action (Alt+Enter does). | S9 (modifier clicks) or S12 | an Alt+click test reporting `ChosenAlt` | `interaction.md` #6 |

Not tasks (watch list): `performance.md` "Remaining risks" (stale-frame
wheel notches, reveal wrap cost, a full render per frame,
`TreeView::flatten` O(rows), the wall-clock guard).

### 3.6 Upstream and dependencies

| Id | What | Depends on | Acceptance | Source |
|---|---|---|---|---|
| U1 | velnor Problem D (§9): file a velnor-new PR so `publish_gate` (`baseline_publish.rs:224-244` at v0.1.9) accepts the configured default branch instead of `repository.default_branch`; then adopt the release, regenerate and confirm `Publish baseline` is green on a push. Not fixable by config (`default_branch = "main"` would stop push CI here). | — | the three §9 acceptance boxes checked; tracker row closed | `upstream/velnor-new-feedback.md` tracker row "Problem D" and §9 |
| U2 | velnor Problem C follow-up (2): without a toolchain file the default is still `rust@1.98.1`; catalog zizmor 1.30.1 (1.31.1 out). Non-blocking; raise upstream. | — | velnor release with the new defaults adopted | `velnor-new-feedback.md` tracker, Problem C (2); `ci.md` line 56 |
| U3 | velnor Problem C follow-up (3): `[tools]` versions are only shape-checked (`zizmor = "99.99.99"` generates). Then raise `[tools] zizmor = "1.31.1"` in `.velnor/config.toml` once a CI run proves it installs. | a velnor fix, or a CI run that proves the pin installs | zizmor 1.31.1 in CI; `dependencies.md` row updated | `velnor-new-feedback.md` Problem C (3); `dependencies.md` versions table; `ci.md` lines 58–60 |
| U4 | crossterm 0.29 Linux input starvation: remove the re-arm in `src/runtime.rs` (`poll_input`, `tty_input::rearm`, and review the related `SizeWatch` SIGWINCH note) once crossterm reads until `WouldBlock`; file the upstream issue if D7 says so and track it in `docs/upstream/`. | D7; a crossterm release with the fix | `holla_pty` flood test passes on Linux without the re-arm | `dependencies.md` crossterm paragraph; `src/runtime.rs` `poll_input` doc, `rearm`, `SizeWatch` doc |
| U5 | Keep the tuiscotti pin on the newest `main` rev (AGENTS.md hard rule): on each move, bump `rev =`, port, re-run the gate and re-approve on purpose. Tracker is otherwise all done; its only open line is V1. | — | `Cargo.toml` rev = tuiscotti `main` head | AGENTS.md "Tooling"; `upstream/tuiscotti-feedback.md` tracker |
| U6 | Transitive holds `generic-array` 0.14.7 and `psm` 0.1.24 (optional `termwiz` path, not built): no action until their pinning crates move; re-check on the monthly update. | upstream crates | `dependencies.md` row removed | `dependencies.md` transitive table |

### 3.7 CI (docs/ci.md, .velnor/config.toml)

| Id | What | Depends on | Acceptance | Source |
|---|---|---|---|---|
| C1 | Enable `visual-smoke` on `["pull_request", "push", "workflow_dispatch"]` with `required = true`; regenerate with `velnor-actions generate` (never hand-edit `.github/`). | V8 | `visual-smoke` is part of `Required` and green | `ci.md` "Visual suites"; `.velnor/config.toml` lines 35–38, 57 |
| C2 | Give `visual-full` and `visual-narrow` `"schedule"` with `schedule = ["0 2 * * *"]`. | V8; V3 for narrow | nightly runs green | `ci.md` "Visual suites"; config lines 70, 83 |
| C3 | `Publish baseline` green on push (red on every push today, not part of `Required`). | U1 | job green on a push | `ci.md` jobs table, `Publish baseline` row |

## 4. Working rules (summary; AGENTS.md is authoritative)

- Subagents do the work; the main session coordinates, reviews and
  integrates. Never more than 5 subagents in parallel, a hard ceiling that
  counts helpers subagents start; extra work waits in a queue.
  Implementation agents work in isolated worktrees on disjoint files
  (AGENTS.md "Way of working").
- Small portions: commit and push after every coherent step as soon as the
  gate passes. Before each push: `git fetch`, rebase onto the remote
  branch, re-run the gate, push. Never force-push.
- Push lock: one pusher at a time. Take it with
  `mkdir "$(git rev-parse --git-common-dir)/push.lock"` and write an owner
  file inside it; if `mkdir` fails, the lock is held: keep working and
  retry at the next checkpoint. Inside the lock: fetch, rebase, check the
  diff stat, push, then `rm -rf` the lock, also on failure (AGENTS.md "Way
  of working").
- No command runs longer than 4 minutes: set a timeout and split long jobs
  (nextest partitions, visual groups). Long commands run in the
  background while the agent keeps working; an agent with only waiting
  left ends its run and reports where to resume, rather than polling.
- Never `pkill -f`, `killall` or kill processes by name: other sessions
  and agents share the machine; stop only PIDs and tasks you started
  (AGENTS.md "Way of working").
- The gate on every step: `cargo nextest run --all-targets` (never `cargo
  test`), `cargo clippy --all-targets`, `cargo fmt --check`, the tuiscotti
  pilot (`--run-ignored only -E 'binary(visual_baseline) &
  test(/^showcase::pages/)'`) and smoke tier, and the step's regression
  tests, which fail on the parent (plan §4 "The gate (G)"; S1 deviation 6
  for the filter).
- Visual changes: named in the commit message, re-approved with
  `tuiscotti accept` in a separate commit, and every changed cell
  attributed to a cause (as in `visual-reapproval.md`); `cargo xtask visual
  check-touched` before the push. Never touch the `visual-baseline` tag or
  its release.
- Behaviour lives once in `src/core`, `src/ui` or `src/widgets`; apps and
  the showcase are thin consumers; every new public capability is shown in
  the showcase or allowlisted with a step (AGENTS.md "Component
  architecture", "Showcase").
- Upstream PRs (velnor-new, tuiscotti): follow the Upstream PR loop in
  `docs/upstream/README.md` and answer with one relay line per PR.
- Every finding is fixed or recorded with a reason; when an item here is
  finished, mark it in its source document and delete or strike it here in
  the same commit.
