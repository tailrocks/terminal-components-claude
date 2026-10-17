# Remaining work on `design/interaction-polish`

## Priority 0 (owner, 2026-10-17): fill every empty showcase page

The owner's first priority: no showcase page may still render the planned
stub ("<Page> arrives with SCnn"). Each page gets real, interactive content
on the Playground | Options | States template; a library component the page
needs and the library lacks is extracted now (its S-step part only) and the
apps' hand-rolled copies move onto it. Agents edit their own rows here.

| Page | Slug | Step | Owning agent (branch) | Status |
|---|---|---|---|---|
| Feedback › Meters | `meters` | SC36 (`MeterList`, S17 part) | p0-meters-notices | **Done** |
| Feedback › Notices | `notices` | SC36 (`Notice`, S17 part) | p0-meters-notices | **Done** |
| Overlays › File browser | `filebrowser` | SC41 (`FileBrowser`, S22 part) | p0-filebrowser | **Done** |
| Layout › Master-detail | `masterdetail` | SC38 (`MasterDetail`, S19 part) | p0-layout | Open |
| Layout › Pane tree | `panetree` | SC45 (`PaneTree`, S26 part) | p0-layout | Open |
| Screens › Query workbench | `queryworkbench` | SC46 | p0-screens-a | Open |
| Screens › Connections | `connections` | SC47 | p0-screens-a | Open |
| Screens › Finder | `finder` | SC48 | p0 screens agent | Open |
| Screens › Disk cleanup | `diskcleanup` | SC49 | p0-screens-b | **Done** (gaps P0-G1, P0-G2 below) |
| Screens › Activity | `activity` | SC51 | p0-screens-b | **Done** (gap P0-G4 below) |
| Screens › Workspaces | `workspaces` | SC52 | p0-screens-b | **Done** (gap P0-G5 below) |
| Screens › Accounts & usage | `accountsusage` | SC53 | p0-screens-c | **Done** (gap P0-G8 below). Composed: a card with the health line as meta over a `MasterDetail` (account `TreeView` with provider folds and default / revoked / disabled metas, inspector with the account's `Notice`, `PropsList` facts, `MeterList` quota windows, Refresh / Set default), and a `DataTable` of the last 7 days' usage; narrow, the library summary line and drawer. Gaps: the add/edit `FormDialog`, the secret-reference picker chain and the per-account usage dialog with tabs are jackin-local (`screens/modals.rs`), not library components yet (S25), so the page does not show them |
| Screens › Launch cockpit | `launchcockpit` | SC55 | p0-screens-c | **Done** (gap P0-G7 below). Composed: `PropsList` launch facts in a card, `StepRail` (11 jackin stages, one skipped as cached), `TextViewport` build log following its tail with the activity in its panel meta, Launch / Cancel launch… (`Dialog::destructive`), the credential `Dialog` (Retry / Cancel launch) and the build failure `Dialog::facts`. No handoff fade (owner decision). Gap: no library activity chip / `ActivityLine` (the activity is the log panel's meta) and no log overlay layer; jackin opens its log as an overlay on `b` |
| Screens › Read-only report | `readonlyreport` | SC56 | p0-screens-c | **Done** (gap P0-G6 below). Composed: `Panel` card with the `live · ago` meta, `MeterList`, copyable `PropsList` facts, `Notice`s, `ButtonRow` actions (Refresh / `r`, Copy report, Next). Gaps: no `PageHeader`/`Heading` yet (S17), so the meta header is a card's meta; the Next buttons post a status because the showcase has no cross-page route request |
| Screens › Plan runner, Editor shell | — | SC50, SC54 | p0-screens-c | **Not stubs**: no page is registered for either (they extend the Task runner and Settings pages, which are real), so Priority 0 has nothing to fill; they stay open as SC50 / SC54 |
| All pages on the template | every slug | SC2 | p0-template | **Done**. `app_tests_template::every_page_is_on_the_template` fails if a registered page does not build through `Template`, open on its Playground and draw the Playground \| Options \| States strip, so no page can fall back to a single view; `options::page_options` has an arm for every page (no catch-all). Converted: Navigation › Sidebars (options `sidebars.*`, States: `NavList` open and collapsed, `SidebarRail`; `NavList` now implements `Specimen`); Inputs & forms › Forms (options `forms.*`: required, validation timing, blur action, description rows, help, placeholders, managed toggle; States: required field, optional field, description; on a short page the form is one column that scrolls part by part; still composed of existing widgets, `FormPanel` is S18); Screens › Task runner (keeps its tree, pipeline and log; options `taskrunner.*`: the tree's folds, indent, environment glyphs and folder counts, the log's tail and wrap; States: targets tree, log, task bar; `r` runs from the strip and the pipeline keeps running under another section); Screens › Settings (keeps its own General / Members / Environment tabs under the section strip; options `settings.*`: the section it opens on, the members table's cell navigation, the environment list's selection mode, description rows, the dirty mark on the tab, the save confirmation; States: section tabs, members table, environment list). `Sidebars` shows the library `NavList` and `SidebarRail`; `Forms` composes existing widgets until `FormPanel` (S18) |

Gaps the Priority 0 Screens pages record instead of drawing locally
(each closes with the step named; the page then adopts the component):

| Id | Page | Gap | Closes with |
|---|---|---|---|
| P0-G1 | Disk cleanup | A per-row tri-state check (a category with some of its candidates checked, holla `[–]`): `ListBox` multi mode has two states per row, so the page checks whole categories. | S16 (`RowMark`) |
| P0-G2 | Disk cleanup | The usage tree's size and share column: sizes ride in `TreeNode::meta`, so they are right-aligned text, not a measured column. | S16 (`TreeView::columns`) |
| P0-G3 | Disk cleanup | Closed: the usage \| details and categories \| reclaim pairs are `MasterDetail`s (S19 landed), each collapsing into a summary and a drawer (`diskcleanup.collapse_below`). | — |
| P0-G4 | Activity | Typing a find query and stepping through its matches (`/`, `n`/`N`, the `n of m` meta): the page shows a fixed query's marks through `TextViewport::set_marks` (option `activity.find`); holla's find code is still app-local. Attach/detach and stdin input mode stay app-level (plan §7.7 row 68). | S22 (`FindState`) |
| P0-G5 | Workspaces | jackin's third column at 150+ columns (Running, Preserved and Daemon cards beside the detail) and its tree's selection strip: the page shows the roster inside the detail card (beside the facts when the card is 84+ wide, under them otherwise). A roster column beside a `MasterDetail` is a `PaneTree`/`MasterDetail` composition the library has no builder for yet. | S19 follow-up (a third pane), S26 |
| P0-G6 | Read-only report | A page header with its `live · ago` meta and section headings: `PageHeader` / `Heading` are not extracted, so the meta rides on the snapshot card (`Panel::meta`). **Next buttons done (S14, `81457c101`)**: "Disk cleanup →" and "Activity →" push those pages through `shell::Router`; Esc and the header trail go back. **Note for p0-screens-c (plan-s17, 2026-10-17):** `widgets::page_header::{PageHeader, Heading}` and `widgets::action_row::ActionRow` have landed; adopt them here (header with `.right("live · ago")`, section `Heading`s, the Next buttons as an `ActionRow`) and close this row. | S17 (`PageHeader`, `Heading`): landed |
| P0-G7 | Launch cockpit | jackin's activity chip and the build log as an overlay on `b`: the library has no activity chip / `ActivityLine` and no log overlay layer, so the activity (with the theme spinner) is the build log panel's meta and the log is inline. | S22, S26 |
| P0-G8 | Accounts & usage | The add/edit `FormDialog`, the secret-reference picker chain and the per-account usage dialog with tabs are jackin-local (`screens/modals.rs`), so the page shows the inspector and an inline usage table only. | S25 |

## Priority 0b (owner, 2026-10-17): the whole content is reachable

The owner's rule: "always make sure there is a way to see the whole
content". Whenever content does not fit its area, scrollbars appear on the
axis that overflows (vertical and horizontal) and all of it is reachable by
keyboard, wheel (Shift+wheel and the horizontal wheel included) and
scrollbar drag.

Done (branch `responsive-lib`):

- **Library**: `ui::scroll_view::ScrollView`, a two-axis scroll container
  over content laid out at its natural size (`ui::measure::Measure`, the
  minimal part of S15's trait; `StateGrid` implements it). Bars appear
  only on the axes that overflow (dotted tracks, blank corner), wheel /
  Shift+wheel / horizontal wheel scroll it, both bars take press and drag
  through the S3 `ScrollPane`, focus moving onto a clipped control (or a
  caret or the focus gutter moving out of view) brings it in with one
  row/column of context, and unused Page Up/Down, Alt+Page Up/Down and
  Home/End scroll it. Content that fits draws in place, unchanged. Scroll
  chaining: a `ScrollPane` or `Stack` whose rows fit passes the vertical
  notch on (it consumed it). Content inside a scrolled view needs no
  change: hits, pointer positions, overlays, the reveal tip and cut text
  follow the view's offset (`HitRegistry::scroll_since`).
- **Template**: every templated showcase page draws its section body
  (Playground, Options, States) inside the template's `ScrollView`, from
  `pages::adapter` (no page code); the Inspector's body scrolls in one too.
  Options and States measure themselves, so they reach everything at every
  size. The Playground is laid out at `Page::measure` (default: the room it
  gets).
- **Gate**: `app_tests_reach.rs` walks every page (all are on the template) and section at
  80x24, 100x25, 106x29 and 120x40, scrolls the body through every window,
  and compares the words seen with those a 220x70 screen shows. Failures
  are listed in `tests/showcase_reach_allow.txt` against the rows below;
  an unlisted failure, or a listed entry that passes, fails the gate.

Open (the audit agent): each row is a page whose Playground squeezes
(cuts or drops content) instead of overflowing, because it has no
`Page::measure` yet. Fix: give the page a `measure` that returns the size
its Playground needs to show everything (its sections' natural heights,
its widest row), so the template scrolls instead of cutting; where a
missing word is only an inner list's later rows (reachable by scrolling
that list), say so and keep the line with that reason. Delete the page's
lines in the allowlist with the fix. Words shown are a sample of what is
never in view at the smallest failing size.

| Id | Page | Section | Fails at | Never in view (sample) |
|---|---|---|---|---|
| R1 | Overview | Playground | 80x24, 100x25, 106x29, 120x40 | `0a1c0c 173f1c 19191c 1e5024 256 262626 2e0f0f 2e9a38` |
| R2 | Chrome | Playground | 80x24, 100x25, 106x29, 120x40 | `7f3a 9c41 Brand Center Chosen Closed Ctrl Hide` |
| R3 | StatusHints | Playground | **Done** (reach-a): `measure`: width from `keyhint::width` (every layer and the resolved bar whole after the label column) and the captions, height from the sections with their notes wrapped (the notes and the options' help wrap instead of being cut); "all fit" draws at the bar's natural width | — |
| R4 | BreadcrumbsRails | Playground | **Done** (reach-a): `measure` from `Breadcrumb: Measure`, `Panel::outer`, `props::width`; the Trails card spans the page (the deep trail is ~90 cells, so 80-column screens scroll sideways); captions say `full width` instead of a size-dependent number | — |
| R5 | HelpKeys | Playground | **Done** (reach-a): `measure` through `tiles`: the reference card keeps its 58 columns (`Tile::fixed`, so "full" wraps at 54 at every size) beside the overlay card and its note, stacked when they do not fit; long facts wrap | — |
| R6 | QuitLeave | Playground | **Done** (reach-a): `measure` through `tiles`: the document card and the classify card side by side when both fit whole (buttons on one row, the classify table at its columns), stacked otherwise; facts wrap; the notes wrap to the card | — |
| R7 | FocusNavigation | Playground | **Done** (reach-a): `measure` through `ui::measure::tiles` (ring and readout, list and `Nav` map side by side when they fit, stacked otherwise; the keys card under them); the readouts are `props` sheets whose long values wrap; the mailbox list keeps an 8-row viewport and scrolls itself | — |
| R8 | Buttons | Playground | 80x24, 100x25, 106x29, 120x40 | `190 Button Close Copy Danger Duplicate Edit Explain` |
| R9 | Inputs | Playground | 80x24, 100x25, 106x29, 120x40 | `Alt Apply Cancel Cmd Copy Ctrl Cut Deleting` |
| R10 | TextAreas | Playground | 80x24, 100x25, 106x29, 120x40 | `API Commit Junie Plan Reading Review Use add` |
| R11 | Chips | Playground | 80x24, 100x25, 106x29, 120x40 | `100 120 5432 Acme Add Applies Empty Engine` |
| R12 | ChecksRadios | Playground | 80x24, 100x25, 106x29 | `Auto Beta Isolation Radio Read Region Repeatable Serializable` |
| R13 | Toolbars | Playground | 80x24, 100x25, 106x29 | `Action Activated End Explain Home Keys Last Line` |
| R14 | Tables | Playground | 80x24, 100x25, 106x29, 120x40 | `00s 01s 05s 08s 1044 1045 1046 1047` |
| R15 | Editable | Playground | 80x24, 100x25, 106x29, 120x40 | `1050 1051 1052 1053 118 Activated Alt Basic` |
| R16 | Grid | Playground | 80x24, 100x25, 106x29, 120x40 | `1011 1012 1013 120 150 1920 201 202` |
| R17 | Selectable | Playground | 80x24, 100x25, 106x29, 120x40 | `APAC Billing Brandt Contract Ctrl Details Docs Eli` |
| R18 | CellEditors | Playground | 80x24, 100x25, 106x29, 120x40 | `Ana Aziz Bea Berg Canary Cancel Checkbox Chen` |
| R19 | Columns | Playground | 80x24, 100x25, 106x29, 120x40 | `00s 01s 05s 08s 1040 1041 1042 1043` |
| R20 | Lists | Playground | 80x24, 100x25, 106x29, 120x40 | `1000 BoxOnly Ctrl Double Elixir Haskell ListCmd Page` |
| R21 | Trees | Playground | 80x24, 100x25, 106x29 | `Activate Cargo CollapseAll Connections Double Expand ExpandAll None` |
| R22 | Tabs | Playground | 80x24, 100x25, 106x29, 120x40 | `Checks DDL Data Export Foreign Import Indexes Migrate` |
| R23 | Menus | Playground | 80x24, 100x25 | `Action Chosen Control Dismissed Enter F10 Keys Menu` |
| R24 | Dialogs | Playground | 80x24, 100x25, 106x29 | `Action Bodies Buttons Cancel Cancelled Close DialogResult Edit` |
| R25 | Pickers | Playground | 80x24, 100x25, 106x29, 120x40 | `Back Cancelled Chosen ChosenAlt Commands Commit Detail Export` |
| R26 | DialogKinds | Playground | 80x24, 100x25, 106x29, 120x40 | `Action Bool Cancel Changed Check Choose Chooser Closed` |
| R27 | PopupsTooltips | Playground | 80x24, 100x25, 106x29, 120x40 | `300 600 Below Center Chosen Connected Dismiss Dismissed` |
| R28 | FileBrowser | Playground | 80x24, 100x25, 106x29, 120x40 | `Cancelled Choose Ctrl Enter File FileCmd FileEntry Filter` |
| R29 | Progress | Playground | 80x24, 100x25, 106x29, 120x40 | `100 BarLayout Build Capacity Deploy Error Exhausted High` |
| R30 | EmptyError | Playground | 80x24, 100x25, 106x29 | `1041 1042 1204 Acme Connections Corp EmptyState Enter` |
| R31 | Meters | Playground | 80x24, 100x25, 106x29 | `1000 310 500 512 840 Backup CPU Data` |
| R32 | Notices | Playground | 80x24, 100x25, 106x29 | `200 65535 DESC Enter FORM LIMIT LINE New` |
| R33 | Panels | Playground | 80x24, 100x25, 106x29, 120x40 | `03s 07s 11s 14s 18s 210 22s 25s` |
| R34 | Scrolling | Playground | 80x24, 100x25, 106x29, 120x40 | `008 009 010 011 012 013 014 015` |
| R35 | SplitsStacks | Playground | 80x24, 100x25, 106x29 | `12s Advanced Connection Credentials Danger Network Running Stack` |
| R36 | MasterDetail | Playground | 80x24, 100x25, 106x29, 120x40 | `014 194 204 377 655 Analyzed Columns Indexes` |
| R37 | PaneTree | Playground | 80x24, 100x25, 106x29 | `12s WARN across binaries masterdetail minimum nextest page` |
| R38 | Editor | Playground | 80x24, 100x25, 106x29, 120x40 | `200 Client Completion Diagnostics Err Error Exhausted Last` |
| R39 | Diff | Playground | 80x24, 100x25, 106x29, 120x40 | `api cafe enabled endpoint example exponential health https` |
| R40 | Terminal | Playground | 80x24, 100x25, 106x29, 120x40 | `LIVE Viewport container credentials failure image launch queued` |
| R41 | TextUtilities | Playground | 80x24, 100x25, 106x29, 120x40 | `234 512 567 Activate Connection Copy Ctrl CursorPos` |
| R42 | QueryWorkbench | Playground | 80x24, 100x25, 106x29, 120x40 | `10312 10319 10326 10333 10340 10347 10354 10361` |
| R43 | Connections | Playground | 80x24, 100x25 | `5432 Last SSH SSL ago bastion connections hour` |
| R44 | Finder | Playground | 80x24, 100x25, 106x29, 120x40 | `Alternatives Data Gate Host Risk Runs State Why` |
| R45 | DiskCleanup | Playground | 80x24, 100x25, 106x29, 120x40 | `100 2000 201 2026 700 810 900 926` |
| R46 | Activity | Playground | 80x24, 100x25, 106x29, 120x40 | `184 201 404 412 4821 4822 Command Exit` |
| R47 | Workspaces | Playground | 80x24, 100x25, 106x29, 120x40 | `04d7 9b02 Accounts Claude Codex Delete Edit Environments` |
| R48 | AccountsUsage | Playground | 80x24, 100x25, 106x29, 120x40 | `100 1Password 212 Account Cost Credential Engineering Mon` |
| R49 | LaunchCockpit | Playground | 80x24, 100x25, 106x29 | `Binari Build Cancel Capsule Construct Derived Hardline Identity` |
| R50 | ReadOnlyReport | Playground | 80x24, 100x25, 106x29 | `08Z 120 17T09 2026 24B83 412 500 Activity` |
| R51 | Forms | Playground | 80x24, 100x25, 106x29, 120x40 | `Options Review and does done like look off` |
| R52 | Sidebars | Playground | 80x24, 100x25 | `Appearance Ctrl Disabled Keyboard Last and are behind` |
| R53 | Settings | Playground | 80x24, 100x25, 106x29, 120x40 | `PRs and for invoicing off refunds storefront the` |
| R54 | TaskRunner | Playground | 80x24, 100x25 | `Cancel Log Press Ready following libs publish shared` |

## Priority order (owner, 2026-10-17, budget-constrained)

Budget is nearly spent (weekly 89%, session 59%). Order: (1) every showcase
page real on Playground | Options | States: DONE; (2) every screen fully
reachable at any size (scroll both axes); (3) the rest of section 1.
This is the single triage of the open items in section 3.

### NOW (in progress, must finish)

- Priority 0b reach R1-R54 (two agents), `tests/showcase_reach_allow.txt` to empty.
- R-apps: app screens (jackin, holla, tablepro) responsive and reachable audit, then fixes.
- R-apps-1: strips and status bars drop items with no way to reach them (overflow `…`/menu).
- R-apps-6: fixed-size page bodies move onto `ScrollView`.
- Flake fixes that make the gate reliable (a red or flaky gate costs more budget than any feature).

### NEXT (max 8, best user-visible value per effort)

1. A20: `Strings::not_in_preview` unused, apps hard-code "Not in this preview". Tiny, one shared string.
2. A15: terminal viewport SGR 7 draws a white flood. Visible bug, one widget.
3. T (SV3, SVg7, SV2): `text_faint` contrast and mono hover/press. Hard rule (4.5:1; states differ at every colour level).
4. A11: no-room sheet and focusable status bar peek line. Hard rule (no hidden content), pairs with 0b.
5. A19: `MenuBar::render` mutates hover while drawing. Correctness bug, one function.
6. A12: hscroll reveal for cut labels in Tree/List/Picker/DataTable. Hard rule (no hidden content); same family as 0b.
7. V10: framed panel title cut below `title_min`. Visible clipping on every framed panel in all apps.
8. S23-Q: `QuotaTone` into the library meter API (D5=a). Removes an app-local rule, small, unblocks jackin usage/capsule cleanup.

### LATER (grouped, one line each)

- S-steps: S15 (`Component`/`Draw` traits; big, unblocks CO8, M28, K19), S16, S18, S19 (rest), S20, S21, S22 (rest), S23, S24, S25, S26 (rest), S27.
- Carry-overs: CO1, CO8, CO10, CO11, CO12, CO13, CO16, CO17.
- Showcase coverage: SC31-SC58 and the allowlist (729 pending lines), SC50/SC54 (no stub pages).
- Visual oracle: V1, V2, V3, V5, V7, V8 (oracle v0), V11 (oracle v1 and `main` port).
- Audits: A3-A9, A10, A13, A14, A17, A21 (the other A items are done).
- Upstream: U1-U7 (U1 blocks C3). CI: C1, C2, C3 (all wait on V8).

### OWNER DECISIONS

D1-D8 are in section 2; D9-D11 are added there too. Current behaviour stays until answered.

| # | Recommended | Blocks |
|---|---|---|
| D1 | (b) measure label width via `ui::columns` | jackin `usage/overview` re-approval |
| D2 | (b) pin the commit action while dirty (library `Footer` rule) | A11 footer work, `editor/mounts_dirty` |
| D3 | (a) Save-time validation for every `FormDialog` | S18 `FormPanel`, form journeys |
| D4 | (a) fixed-choice pickers show fill only; add wording to AGENTS.md | picker re-approvals, V2 |
| D5 | (a) domain status decides; move `QuotaTone` into the library | S23-Q (NEXT 8) |
| D6 | (a) keep `Group > Page` crumb | showcase header re-approval, V1 |
| D7 | (a) file the crossterm issue upstream | U4 |
| D8 | (a) keep `visual-baseline` frozen until oracle v1 | V11 end state only |
| D9 | keep soft frames (plane separates; WCAG 1.4.11 not required when the fill identifies the component) | menu frame contrast work |
| D10 | yes: durations count script time only | holla `input_cancelled` flake (fix on `v1-showcase` c1a49af0f) |
| D11 | second row action in the picker context menu / Shift+F10 | picker Alt+Enter on legacy terminals |

### Which "done when" criteria (section 1) fit the remaining budget

| # | Criterion | Verdict |
|---|---|---|
| 1 | All S-steps and T Done | Not realistic: S15-S27 are large. Do T only. |
| 2 | All SC steps Done, allowlist permanent-only | Not realistic: 729 pending lines. |
| 3 | Every audit finding fixed or decided | Partly: NEXT items plus owner answers D1-D8; record the rest with a reason. |
| 4 | Oracle v0 imported, `snapshots/` removed | Not realistic: 1,650 legacy scenarios, V1-V3, V8. Defer. |
| 5 | CI visual jobs required, `Publish baseline` green | Not realistic: needs V8 and U1 first. |
| 6 | Full gate green on branch head | Realistic and required: do this last, after NOW. |

Realistic scope: NOW, the 8 NEXT items, owner answers D1-D8, green gate.
Owner decision: criteria 1, 2, 4, 5 move to a later budget.

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
2. Every SC-step (SC30–SC58) is "Done" and
   `tests/showcase_coverage_allow.txt` holds only `permanent` lines
   (today: 729 pending, 32 permanent; 684 functions covered via their
   owner in `coverage.rs` `VIA`).
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
| D9 | Truecolor/256 menu frames are soft (~1.1:1 against the plane). | (a) keep soft; (b) raise to 3:1. | Recommend (a): the plane separates and WCAG 1.4.11 is not required when the fill identifies the component. | contrast audit |
| D10 | holla task durations: count script time only, excluding prompt wait? | (a) yes; (b) wall time. | Recommend (a); also fixes the `input_cancelled` flake (fix committed on `v1-showcase` c1a49af0f). | holla |
| D11 | Picker alternate choice (Alt+Enter) has no deliverable non-Alt key on legacy terminals. | (a) second row action in the picker context menu / Shift+F10; (b) leave Alt only. | Recommend (a); hints never advertise undeliverable keys. | A29 |

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

Unblocked now: S12 (its dependencies are done; S8a, S8b, S9, S10 and S11 are done) and
the theme track T. The §4 groups F–K still apply; the "Depends on" column
below is the dependency each step must wait for.

| Id | What (expand; contract) | Findings it closes | Depends on | Acceptance beyond G | Demo |
|---|---|---|---|---|---|
| S8a | **Done 2026-10-17** (plan "S8a outcome"; K28, K35, K37, K38 recorded there and in `keyboard.md`, CO1 moved to S8b). `shell::{KeyRouter, ShellKeys, ShellAction}`, `Page::on_interrupt`; delete the 4 `on_key` functions, `esc_ladder`, `workbench_chord` | K11, K20, K21 (remaining apps), K26, K28, K35, K37, K38, SI10, SI16, SI23; CO1, CO2, CO9 | S7 (done) | one cross-app key-order table test | SC33 part, SC46 |
| S9 | **Done** 2026-10-17 (plan "S9 outcome"): `SelectGesture`, `RowSelection::apply`, `ClickPolicy`, `Activate`, `CheckClick`; List `checked`/`anchor`, Grid `selected_rows`, OptionEditor `checked`, tablepro `connections.rs` empty double-click branch and holla `disk.rs` toggle-on-click deleted. Left for SC33: the Data grid and Text utilities click knobs, `SelectGesture` and `ClickPolicy::edit_on` (allowlisted) | M9, M10, M15, M16, M32, SI5, SI14, SI24, SIg2; CO10 (activation); A24 | S4, S6 (done) | the M15 harness (holla Disk) flips; a selection-model table test (both landed) | SC33 |
| S10 | **Done 2026-10-17** (plan "S10 outcome"; popup rows move with S12's `OptionList`, K12 Menu/Picker/Select with S12, StatusBar with K38, a dedicated peek key open). `ui::row::RowUi`, `RowPaint::{mark, label}`, `RowLook`; delete 11 painting skeletons | K12, SI8, M25, SV14/SVg (visual), SV1 (visual), SVg3 | S4 (done) | digest unchanged before the visual commits; a contrast test per new token | SC33 |
| S8b | **Done 2026-10-17** (plan "S8b outcome"; CO1 moved to S16, deviations 1–3 there). `HasHints`, `Bindings`, `HintCx`, `shell::Footer`, `HelpOverlay::from_keymaps`, `HelpSection`, the component docs generator (`cargo xtask docs components`, `docs/components/`); 4 `draw_footer`, modal hints in T/H/J, 4 help renderers and the `"Enter","Edit"` literals are gone | K10, K23, K25, K39, SI7, SIg4, SVg10 (visual); the "no keyboard-only state" coverage test; CO3, CO4, CO5; D2 if answered (b) | S8a, S4 | hint snapshot per widget; coverage test walks every page's `KeyMap`; `docs_components_fresh` test | SC33 |
| S11 | **Done** 2026-10-17 (plan "S11 outcome"): `widgets::edit::{EditField, Validate, Validator, ValidateWhen, FieldError, Editable}`; `PointerField` once for every `Editable`; TextInput, TextArea, CodeEditor, table and grid cells, Picker, Combo, find and holla finder queries on `EditField`; `TextValidator`, `table::CellValidator`, `grid::Validator` deleted. Left: the Combo query's pointer gestures (S12), holla Files query (S22), F6 hint (S8b), the new API allowlisted for SC32 | M2, M21, M31, M39, K1 walk, K5, K7, K17, K36, SI6, SIg5, SIg9; Select All in picker/combo/find queries; D3 unchanged (owner) | S4, S6 (done) | the K1 walk, the M2 CodeEditor drag test (both landed) | SC32 |
| S12 | **Done** 2026-10-17 (plan "S12 outcome"): `OptionList`, `Item`, and the bodies of Completion, Picker, ContextMenu, Select and ChoiceEditor on it; SI13, SI21/SV15, K38 (chip lead), A23, CO15, the tablepro tab list, the cell-popup wheel and the Combo query pointer. Left: the popup peek line (K12, S10 deviation 4) and the deviations in the plan. `widgets::option_list::OptionList`, `Item`; bodies of Select, OptionEditor, Picker, Completion, ContextMenu; tablepro `PickerItem`/`CompletionItem` literals | M19, M38, SI13, SI21/SV15; tablepro tab-list glyph mismatch (`app.rs:1655` vs `1682`); CO6, CO7 | S3, S4, S5 (done) | popup journeys; wheel tests in all five | SC32 |
| S13 | **Done** 2026-10-17 (plan "S13 outcome" and "S13 (Edit menu part) outcome"): `ListCore`, `ContentState`, List, Tree, Props, Steps on `ListCore`, K37; the standard Edit context menu on every text widget (M3, `ui::edit_menu`; a right click now focuses the control, `PointerConfig::secondary_focuses`); default context menus from key maps (M22: `command_items`, DataGrid, ChipBar, TreeView Expand/Collapse all); M6 and M30 were already fixed. Left: tablepro/holla text panes and tablepro's hand-listed grid menu (S22), PropsList "Copy value" (S15), page options (SC34). | M3, M6, M22, M30, K37 | S9, S10, S11 | a right-click test on every text widget | SC34 (States grids: Lists, Trees, Text utilities done) |
| S14 | **Done** 2026-10-17 (plan "S14 outcome"; deviations 1–7 there): `shell::{Router, Nav, Chrome, Breadcrumb, Clipboard}`, `App::stakes`, typed menu `Cmd`; delete S `goto`, T `Screen`, H `go`, J `go`/`Screens`/`ROUTES`, label-string re-dispatch (H:1569, J:1667), `copy_osc52`, `request_quit`/`open_quit_confirm` ×3 | M5, M7, M8, M23, M40, K4, K6, SI17, SIg3 | S7, S8a | quit and leave journeys per app | SC34 |
| S15 | `Component`, `Draw`, `Measure`, `Container::visit`, `AnyComponent`; builder renames; delete `MouseTarget` adapters, the `Outcome` adapter, free render fns (`empty::render`, `props::render`, `keyhint::render*`), per-widget `on_*` | M28, K19; CO8 | S6–S13 | `rg 'fn on_click\b\|fn on_wheel\b' src/widgets` = 0; `WidgetDoc` required by `Component` (§5) | — |
| S16 | `Column::marker`, `RowMark`, `ListItem::{header, status}`, `TreeNode::{meta_tone, trailing, label_tone}`, `TreeView::columns`; S16a library, S16b holla rows, S16c jackin rows and trees, S16d tablepro EXPLAIN tree-table | 69 screen list-key arms; CO11; CO1 (page key maps, then one router guard); page help sections from `Page::bindings` (plan "S8b deviations" 2) | S10, S13 | per-app captures pixel-equal; the jackin tree hit test | SC35 |
| S17 | **Done 2026-10-17** (P0; plan-s17 branch): `MeterList`, `Notice` (earlier), then `Tabs::{layout, layout_gap, passive, passive_focused}` (tablepro table sections, Structure sub-tabs, connection form, workbench strip, jackin capsule usage dialog, showcase Settings / Query workbench / Tabs), `TextInput::{readout, readout_link, set_readout}` (Inputs page, Finder scope), `NavList` moved into `widgets::nav_list` (Sidebars page, with `SidebarRail`), `PageHeader` / `Heading` (`widgets::page_header`; holla `screens::heading` deleted, activity, cleanup, plan, review gate 1 and snapshot headers), `ActionRow` (holla review ×3 and plan confirm rows, tablepro connection form footer and filter editor), `DetailCard` (tablepro connection card, showcase Connections). `Item` builder: already landed with S12 (`Item::value`); the string-index parsing left (`TreeNode::meta` in the EXPLAIN tree, `LayerTag::key` in holla plan/disk menus) belongs to S16d and S22. Captures of the four apps unchanged. Left by hand, moved to CO17: holla disk header (its subtitle switches per view), plan runner and review gate 2 / run headers, about 17 jackin headings, the showcase app header, tablepro result sub-tabs (`y += 2` inside a running cursor), the connection form's test result (a status glyph and label after the buttons) and the jackin capsule usage dialog's undrawn per-row detail (see the MeterList note). | M34; CO12 | S8b, S10 | captures unchanged | SC36 |
| S18 | `FormPanel`, `FormField::pair`; tablepro connections form and filter editor, showcase forms | K6 | S11, S17 | form journeys | SC37 |
| S19 | `MasterDetail` (14 sites, `SPLIT_MIN`). Library part done with SC38 (`ui::master_detail`, `SplitKeys`, K18); holla finder/cleanup/disk/files and jackin accounts/manager migrated; open: `SplitKeys` in the app screens, holla plan (review, runner zoom with S26); the rest are not master-details (plan "S19 outcome (partial)") | M20, K18 | S17 | narrow-tier captures | SC38 |
| S20 | `EditorShell` (jackin settings and editor) | K4 (leave guard) | S14, S18 | editor journeys | SC39 |
| S21 | `FilterTree`, `FilterList` | — | S11, S16 | filter journeys | SC40 |
| S22 | `FileBrowser` + `FileSource` **done** (Priority 0, plan "S22 outcome (partial)": `widgets::file_browser`, jackin prelude/config/accounts on it). Open: `CommandMenu`, `ContextAnchor::row`, `StepLine`/`Wizard` (the browser keeps a `stepper` line until then), `FindState` + `line_numbers`; showcase `pages/row_menu.rs` (`RowMenu<T>`) and its 7 users; 12 wall-clock spinner sites; jackin dead code | `showcase-pages-navigation.md` SC16 1 | S5, S11, S13 | jackin and holla journeys | SC41 |
| S23 | `Tabs` preview and pinned tabs, `FooterNote`, `ActivityLine` | apps inventory 5, 41, 50, 51 | S2, S17 | §7.3 rules | SC42 |
| S23-Q | Move jackin `QuotaTone`/`quota_tone`/`meter_tone` (`src/bin/jackin_preview/screens/accounts.rs:2036-2068`, used by `capsule.rs:2691`, `usage.rs:22`) into the library meter API (a domain-level tone a `Meter` takes); no app keeps a tone rule | known item (owner brief); D5 | S23, D5 | jackin usage/accounts/capsule captures unchanged; a library unit test per level | SC42 or SC36 (`MeterList`) |
| S24 | `TreeView` tri-state checks and trailing action row; `DataTable` group rows, match ranges, row marks, group folds; `StepRail` table variant | apps 57, 59–62; CO13 | S16 | §7.3 rules | SC43 |
| S25 | `PickerChain` (jackin OpFlow `modals.rs:551-948`, holla go-to-path) | app 34 | S12, S22 | §7.3 rules | SC44 |
| S26 | `PaneTree`, zoom (`Split::toggle_max`/`Maximized`); no cross-fade (owner decision 2026-10-17). Library part done with SC45 (`ui::pane_tree`; jackin's tree types now re-exported from it); open: the capsule's resize, zoom and seam drag and holla's plan runner zoom onto `PaneTree` | apps 8, 69 | S2, S19 | §7.3 rules | SC45 |
| S27 | Strategy B nav policies `core::nav::{FocusPolicy, AdjacentMove, AdjacentKeys, Scope, Bounds}` on Tabs, TreeView, ListBox, Select, Picker, ContextMenu/MenuBar, Dialog, Toolbar; `KeyRouter` honours `AdjacentMove` | `showcase-pages-navigation.md` SC28 6; Strategy B rows of `showcase-coverage-library.md` | S4, S7, S8a | §7.3: a widget test per `AdjacentMove` variant at both edges, a cross-app key-order test, `Scope` inside a modal without a trap, digest unchanged | SC57 |
| T | Theme track: plane-relative `text_faint` (SV3), mono hover and press (SV2), panel title marker (SV4), `border_strong` field outlines (SVg1), one pressed direction (SVg2), success vs accent (SV14), `COLORFGBG` (SVg7) | all visual, each re-approved alone | S0 (done); can run now | contrast test on every plane | — |

### 3.2 Carry-overs from finished steps

Deviations and "Notes for later steps" of S0–S7 that are still open in the
code. Each is done inside the step named.

| Id | What | Source | Done in | Acceptance |
|---|---|---|---|---|
| CO1 | Call `keymap::unassigned_chord` once in `KeyRouter` before the focused widget, then delete the per-widget calls. Widgets now expose `Bindings::chords`; the pages' own chords are still handled in code, so the router cannot skip the page yet (plan "S8b deviations" 1). | plan "S4 deviations" 2; "S4 notes" S8a; "S8b deviations" 1 | S16 (moved from S8b: pages declare their chords on key maps first) | `rg 'unassigned_chord' src/widgets` only in the router |
| CO2 | **Done (S8a)**: shortcuts first, then type-ahead (`TypeAheadPrecedence::Shortcuts`), owned by `KeyRouter` (it wraps the whole order in `route_key`; no app calls it); test `a_letter_shortcut_wins_over_a_focused_lists_type_ahead`. Decide where page letter shortcuts sit relative to a focused list (type-ahead consumes a miss, so a page letter no longer reaches the page while a list has focus); record the rule. | "S4 notes" S8a | S8a | a key-order test row for a letter over a focused list |
| CO3 | **Done (S8b)**: `core::hints::nav_hints` reads each widget's `NavKeys` and `type_ahead`; Picker and Completion hint from their own keys (`widget_hints_tests.rs`, Desktop and Vim rows). Pages that still hand-spell nav hints adopt widget hints with S16. | "S4 notes" S8b | S8b | hint snapshot per widget under Desktop and Vim |
| CO4 | **Done (S8b)**: the library modals implement `HasHints` and report it through `Layer::hints`; jackin `layer_hints` and holla's downcasts are gone; re-approved. | "S5 deviations" 3; "S7 deviations in part 2" 3 | S8b | `rg 'fn layer_hints' src/bin` = 0 |
| CO5 | **Done (S8b)**: `shell::Footer::render` calls `Shell::footer_drawn`; an error status wins over every hint. | "S7 notes" S8b; "S2 deviations" 3 | S8b | `rg 'HintKeys::register' src/bin` = 0 |
| CO6 | **Done (S12)** except plan "S12 deviations" 1 (ChoiceEditor keeps its option-index cursor and its `Shown` row adapter over the filtered options). `OptionList` holds a `Cursor`, implements `RowSource for [Item]`, gets a `ScrollPane` (`draw_v`) and the `Overlay` for Select, Completion, ContextMenu and ChoiceEditor; delete `Select::bar`, `bar_track`, `v_click`, the `Shown`/`[MenuItem]`/`[PickerItem]` adapters, Picker/Completion `cursor_dirty`. | "S3 deviations" 4, 6; "S3 notes" S12; "S4 notes" S12; "S5 deviations" 7 | S12 | the five popups share one wheel and bar path (wheel tests) |
| CO7 | **Done (S12)**: the ChoiceEditor half (`MouseTarget` and `Overlay`; DataTable, DataGrid and the Cell editors page route to it; `rg` check 0 hits; the Combo query takes the pointer). Menu half: `ContextMenu` routes its pointer by the event's part (a secondary release on a row arrives as a click and chooses it), its inherent `on_wheel` is gone (the wheel goes through `OptionList`'s `Scrollable` for the menu and the MenuBar, Tabs and ChipBar menus); the showcase `RowMenu` sends an outside click through `layer::dismiss`, jackin's capsule tab and pane menus route clicks and the wheel through the menu's `MouseTarget` (`capsule_pane_menu_routes_the_pointer_by_part`). Left: `ChoiceEditor` gets a `MouseTarget` through `OptionList`; the showcase Cell editors page stops calling `ChoiceEditor::on_wheel`; jackin's own `RowMenu` (`screens/rowmenu.rs`) still takes an id from its screens' id-taking `click` (S22, with those screens' pointer routing). | "S6 deviations" 9; "S6 notes" S12 | S12 | `rg 'ChoiceEditor::on_wheel\|\.on_wheel\(' src/bin` = 0 |
| CO8 | Delete `PartMap` (DataTable, DataGrid), the 14 `WidgetId::item_of` helpers, `contains_id`, `pointer_in`, the per-gesture `Layer` handlers and the id-taking `on_click`/`on_press`/`on_drag`. | "S6 deviations" 8; "S6 notes" S15 | S15 | `rg 'PartMap\|item_of\|contains_id' src` = 0 |
| CO9 | **Done (S8a)**: holla `deliver_all`; test `a_background_tabs_tick_status_shows_when_the_tab_is_shown`. holla's background tabs drop their tick and message requests (only blur statuses wait); unify with the shell's background-request policy. | "S7 deviations in part 2" 4 | S8a | a holla test: a background tab's request is applied when shown |
| CO10 | Activation half done in S9 (the screens ask `ClickPolicy::DATA`, `rg 'ev\.count\(\) >= 2' src/bin` = 0). Left: hand-drawn list bars page on click (`scroll_press`, `offset_for_click`). | "S6 notes" S9; plan "S9 deviations" 6 | S16 (bars) | `rg 'scroll_press\|offset_for_click' src/bin` = 0 |
| CO11 | Screens still scanning derived ids or drawing lists by hand move onto widgets: holla disk, files, finder, plan, review, snapshot, cleanup; jackin manager tree, accounts tree, capsule panes and seams, config, editor, inspect, settings, usage; tablepro `app.rs`; showcase Scrolling/Chrome/Menus/Buttons/EmptyState actions, NavList; drop `render_with_fade` and holla `scroll_press`/`scroll_drag` there. | "S1 notes" S16; "S3 deviations" 7, "S3 notes" S16/S17; "S6 notes" S16 | S16, S17 | `rg 'scroll_press\|scroll_drag\|\.child\(' src/bin/<app>/screens` = 0 per app |
| CO12 | `ui::text::center_x` adopters (jackin capsule, cockpit, prelude, rain centre by `chars().count()`) move with their screens. | "S2 deviations" 7 | S17/S22 | no `chars().count()` centring in `src/bin/jackin_preview` |
| CO13 | holla plan header (`src/bin/holla/screens/plan.rs`, "k of n done · running · blocked · failed") builds and tones its clauses by hand; draw `progress::Tally` with a blocked count. | plan "SC21 outcome", "Recorded, not migrated" | S24/SC50 (or the next touch of plan.rs) | holla plan captures unchanged except the failed-clause tone |
| CO14 | **Done (S14, `87738119e`)**: both sites use `Chrome::fit`. tablepro dialogs still use `outer_margin` (`src/bin/tablepro/app.rs:99`, `:1431`); move to `Dialog::chrome(top, bottom)`. | "S5 deviations" 4 | S14 or S17 | `rg 'outer_margin' src/bin` = 0; tablepro captures unchanged |
| CO15 | **Done (S12)**: a K14 pass-on key closes jackin's row menus (`RowMenu` on manager, accounts, editor, settings) and the Capsule's tab and pane menus and reaches the page (`editor_shift_f10_opens_under_the_cursor_row_and_removes`: `]` moves to the next tab; `capsule_pane_menu_passes_an_unused_key_to_the_page`: F10 opens the menu bar). The menus still belong to the screens, not the layer stack. | "S5 deviations" 6 | S12 or S22 | a jackin test: a pass-on key reaches the page |
| CO17 | S17 leftovers still drawn by hand: holla disk header (a subtitle per view), plan runner header (`plan.rs` render_runner), review gate 2 and run headers (`review.rs`); about 17 jackin headings (settings, manager, inspect, editor); the showcase app header; tablepro result sub-tabs (`tabs.rs`, strip inside a running `y`); the connection form test result after its buttons (a `StatusKind` glyph and label: give `ActionRow` a status message or leave it). Move each onto `PageHeader` / `Heading` / `Tabs::layout` / `ActionRow` with its screen. | S17 outcome (plan-s17) | S16, S22 | captures unchanged per app |
| CO16 | Type-ahead takes letters and digits only, so a label starting with punctuation (`.config`) cannot be typed to; record if a screen needs it. | "S4 deviations" 3 | S16 (screens) | either a screen test or a recorded "not needed" |

### 3.3 Showcase coverage track (shared-components-plan.md §7)

SC0–SC29 are done. Acceptance for every SC step is §7.4: harness tests for
every option and state (Inspector and `--opt`), the digest regenerated
(`UPDATE_BASELINE=1`) with each changed page listed, `.pending()` visual
rows for new surfaces, the step's allowlist lines deleted with the gate
green, no app-local code in `src/bin/showcase`, and the §7.8 row updated.

Allowlist today (`tests/showcase_coverage_allow.txt`, `id | reason | step`):
761 lines, 32 `permanent`, 729 pending: SC34 347, SC58 210, SC31 76, SC41 47,
SC33 38, SC38 7, SC32 4. Besides, 684 functions are covered via their owner
(`VIA` in `src/bin/showcase/coverage.rs`, SC-G3).

| Id | What | Depends on | Allowlist lines | Source |
|---|---|---|---|---|
| SC-G1 | **Done.** The scan lists free `pub fn`s and inherent methods of public types (not constructors) as `CapKind::Function`; 1207 entries registered; 30 declared as existing demos, 854 `permanent` (since SC-G3: `Via` or owner-pending), the rest pending on SC31/SC33/SC34/SC38/SC41 or the new SC58; plan "SC-G1 outcome". | — | +1177 | owner brief; plan "SC0 outcome" (scan scope) |
| SC-G2 | **Done.** `TabCmd::ShowHidden` is shown (the Tabs keys readout names "↓ hidden tabs" while the strip has an overflow menu; `the_keys_readout_names_the_hidden_tabs_key_while_the_strip_has_a_menu`). Re-pointed with reasons: `TextInput::validate_on`, `ValidateOn` and its two variants to SC32 (no playground field is an untouched empty required field, so the two variants look the same until `ValidateWhen` lands on Inputs); `ChipCmd::ShowHidden` to SC33 (named only in the `+N` tooltip; the generated keyboard reference lists it; the Chips playground has approved captures). `Theme::centred_hints` stays on SC30, which shows it on Status & hints. The `type_ahead_timeout` lines moved under `# widgets::menu`; `# core::list` is gone. | — | 7 → 0 on finished steps | allowlist |
| SC-G3 | **Done.** `Shown::Via(owner)`: a function covered by the demo of its owning type, from one compact `VIA` table in `coverage.rs` with `ViaRole` (Protocol, Operation, Plumbing) reasons; valid only while the owner has its own `Demo`/`Option`/`StateRow`. The 880 `permanent` function lines (854 from SC-G1, 26 from S9) became 684 `Via`, 176 pending on the owner's step (SC34 171, SC33 3, SC31 2) and 20 specific runtime exclusions; `permanent` 892 → 32; plan "SC-G3 outcome". | SC-G1 | −684 | coordinator brief |
| SC30 | **Done.** Status & hints shows the footer `StatusLine` (TTL, error, paused), the `Motion` levels on a running job, `Theme::centred_hints`, `TooSmall` with its builders as options and `CommonArgs` read from clap; plan "SC30 outcome". | S2 (done) | 5 → 0 | §7.5 SC-F table |
| SC31 | Stacked modals through `ShellLayers`/`ModalStack`, `Dismiss::{focus_out, refuse_feedback}`, `Frame::close_button`, `DismissReason` in the State log; fix the recorded SC10 finding (a select's list closes on dead space under `Dismiss::ESCAPE` because `Select::close_on_blur`, not `Dismiss::focus_out`, decides blur) | S5, S7 (done): can run now | 76 (incl. 2 `ModalStack` functions that move to `VIA` once it is shown) | §7.5; "SC10 outcome" recorded finding; `interaction.md` "SC10 finding" |
| SC32 | `EditField`/`Validate`/`ValidateWhen`/`FieldError` on Inputs and Dialog kinds; `OptionList` `WheelMode`, `SelectArrows`, `Item` reason | S11, S12 | — | §7.5 |
| SC33 | `HelpOverlay::from_keymaps` replaces the hand-listed `?` (SC6 note 1), `Footer` precedence, `HasHints` in State tab, generated `KeyReference` incl. `GridCmd`, `ClickPolicy`/`Activate`/`CheckClick`/`SelectGesture`, `RowUi`/`RowLook`/`CursorStyle::Bar`, peek line | S8b, S9, S10 | — | §7.5; `showcase-pages-navigation.md` SC6 1 |
| SC34 | `ContentState` in every list-like States grid, Edit context menu on text widgets, the showcase shell on `Shell`/`Router`/`Chrome` with a clickable crumb and page stack, truthful clipboard results, Chrome toolbar, the State-tab dispatcher readout and widget action logs, `MenuBar::type_ahead_timeout` | S7, S13, S14 | 347 (incl. 171 functions of owners SC34 shows: `core::{event, hit, id, intent, pointer, response, shape}`, `RenderCtx`, `EditAction`, runtime readouts; they move to `VIA` with their owner) | §7.5; allowlist reasons |
| SC35 | Row and tree extensions demo | S16 | — | §7.5 |
| SC36 | **Done 2026-10-17** (P0): Meters (`MeterList`, now under a `PageHeader` whose status follows the offline option and a `Heading`), Notices (`Notice`), Tabs (`tabs.passive`, `tabs.gap` for `Tabs::layout_gap`, the nested strips on `Tabs::layout`), Inputs (`inputs.readout`), Sidebars (library `NavList` with `NavKeys::VIM`, `SidebarRail` collapse / rail / Ctrl+B, SC5 1), Buttons (the Next row is an `ActionRow` with its message, `buttons.next_message`, and its own ←/→ through `ActionRow::step`), Connections (`DetailCard`). `Item` is SC32's (allowlisted there). | S17 | — | §7.5; `showcase-pages-navigation.md` SC5 1 |
| SC37 | `FormPanel`, filter editor through `FormDialog` | S18 | — | §7.5 |
| SC38 | **Done.** Master-detail page on the library `MasterDetail` (wide split with a draggable seam, narrow summary, drawer, hidden pane as a focus stop) and `SplitKeys`; plan "SC38 outcome". Open: `SplitKeys` on Splits & stacks (its panes take no focus). | S19 | — | §7.5; `showcase-pages-navigation.md` SC5 2 |
| SC39 | `EditorShell` on Settings | S20 | — | §7.5 |
| SC40 | `FilterTree`/`FilterList` | S21 | — | §7.5 |
| SC41 | File browser page **done** (plan "SC41 outcome (partial)"). Open: `CommandMenu`, `StepLine`/`Wizard`, `TextViewport::find`, `ContextMenu::type_ahead_timeout` | S22 | 1 | §7.5 |
| SC42 | Preview and pinned tabs, `FooterNote`, `ActivityLine` | S23 | — | §7.5 |
| SC43 | Tri-state checklist tree, trailing action row, group rows, match ranges, row marks, plan outline | S24 | — | §7.5 |
| SC44 | `PickerChain` | S25 | — | §7.5 |
| SC45 | **Done.** Pane tree page on the library `PaneTree` (nested splits, frames, focus moves, resize chords, zoom through `Split::toggle_max`; no cross-fade); plan "SC45 outcome". | S26 | — | §7.5 |
| SC57 | Strategy B options on Focus & navigation and Toolbars, `toolbars.adjacent` | S27 | — | §7.3, §7.5 |
| SC58 | Function coverage added by SC-G1: theme style roles, widget queries and labels, render helpers on their finished pages (plan §7.5 SC58) | SC-G1 (done) | 210 | plan "SC-G1 outcome" |
| SC59 | `ScrollView` and `Measure` on the Scrolling page as their own demo: `Axes`, `ScrollbarVisibility` per view, padding, gap, `ScrollCmd` keys, follow and context as options (every templated section already scrolls in a default view; Priority 0b) | responsive-lib (done) | 40 | Priority 0b; allowlist `# ui::scroll_view, ui::measure` |
| SC46 | Screen: query workbench. **First pass done (P0, 2026-10-17)**: `pages/query_workbench.rs` composes explorer (`TextInput` filter over a lazy `TreeView`), query `Tabs` (`×`, `+`, busy/error flags), `Toolbar`, `CodeEditor` (SQL tones, error `Diagnostic`) over results (`DataTable`, 0-row `EmptyState`, plan grid, `EmptyState::error` card, running bar) in two `Split`/`Splitter`s, `segments` status line; options `queryworkbench.*`; tests `app_tests_workbench.rs`. **Gaps, shown with the closest widget until their step lands**: explorer `FilterTree` (S21; today `TextInput` + `TreeView::set_filter`), sidebar rail collapse, EXPLAIN tree-table (S16d; today a `DataTable` with indented node names), result sub-tabs and `ActivityLine` (S23; today the results card's meta), tab preview/pinned tabs (S23), the table tab (Data/Structure strip, DDL card, filter chips, `FooterNote`, pending bar, Save review with typed token: S17/S18/S23), Open Quickly and the tab list (S22 `CommandMenu`), Safe Mode picker and env-toned gate dialog, zoom (`Split::toggle_max` chord). | S8a, S16, S17, S19, S21, S23 | — | §7.5 "Screens" |
| SC47 | Screen: connections. **First pass done (P0, 2026-10-17)**: `pages/connections.rs` composes a list card (`TextInput` filter over a `TreeView` grouped by folder or environment, env glyphs `Glyphs::env` in `EnvKind::tone`, engine meta), a detail card (`props` with the production Safe Mode Warning row, connect bar, `EmptyState::error` failure block with Reconnect, a wrapping `ButtonRow` of Connect / Edit… / Duplicate / Delete…), the empty list's `EmptyState` with New connection…, the library `FormDialog` (Test connection, Save) and a destructive `Dialog` for Delete; options `connections.*`; tests `app_tests_connections.rs`. **S17 adoption (plan-s17, 2026-10-17)**: the list and the card are a `MasterDetail` (resizable seam, `connections.list_width` is the list's share 40/30/50 %, a one-line summary and a drawer below 80 columns), the card a `DetailCard` (fitted: the actions follow the facts) with an `ActionRow`, and a failure a `Notice` under the facts (Connect becomes Reconnect, as in tablepro). **Gaps, shown with the closest widget until their step lands**: the inline `FormPanel` form (S18; today the modal `FormDialog`), `FilterTree` (S21), the row context menu and the quit guard for a dirty form. | S17, S18, S19, S21 | — | same |
| SC48 | Screen: finder. **First pass done (P0, 2026-10-17)**: `pages/finder.rs` composes a typing-hot query (`TextInput`, ranked live with `ui::text::fuzzy`, then recency, grouped by kind), a scope strip (`Tabs`: Here, Project, Everywhere), the ranked `DataTable` (tone cells, `EmptyState` for no match), a preview card (`props`, Alternatives… and Why? buttons) that folds to a one-line `segments` summary on a narrow page, the alternatives `ContextMenu` (button, Shift+F10 and right-click through `ContextConfig`) and Why? as an `InfoDialog`; options `finder.*`; tests `app_tests_finder.rs`. **S17 adoption (plan-s17, 2026-10-17)**: the scope is the query's `TextInput::readout` link (`scope ‹ here ›`; a click widens it, Alt+↑/↓ step it), the preview a fitted `DetailCard` with Alternatives… and Why? as its action row. **Gaps, shown with the closest widget until their step lands**: the query owning the list (`FilterList`, S21: ↑↓ and Enter from the query; today Tab into the table) and `DataTable` group rows and match ranges (S24; today a Group column and plain names). | S11, S17, S19, S24 | — | same |
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
| V4 | **Done** (visual-v4-v6, 2026-10-17): 121 of 125 overlay sites have rows (pending `OVERLAYS` rows with both dismissals, each run and read at 120x40; five journeys and surfaces moved to `OVERLAYS`, names and approvals unchanged). The 4 sites left (jackin F2/F3, layers that open only from a job on virtual time) got rows in V12: 0 gaps. Approving the rows is V2. Was: 67 of 108 overlay sites had no `OVERLAYS` row. | — | `overlay_sites` reports 0 gaps (4 left, each a recorded finding) | plan Progress T15; §C2; `visual-reapproval.md` "Overlay coverage (V4)" |
| V5 | Complete the element inventories T12–T14: a hover scenario for every interactive element, pressed frames that differ from hover and rest at all 5 levels, disabled hover/press equal to rest. T16: approve jackin's resize rows. | V2 | plan §H T12–T14, T16 acceptance | plan §H |
| V6 | **Done** (visual-v4-v6, 2026-10-17): `holla_pty.rs` and `terminal_suspend.rs` run on `tuiscotti::tui::Tui` sessions; colour classes come from the emulated cells, terminal modes from the observation, termios through the helper's tty device; the resumed frame must equal a fresh launch's frame at the new geometry. The flood test still fails on Linux with the `src/runtime.rs` re-arm disabled (checked in a Linux container). Byte-level counts (`ESC[?1049l` once per suspension) became mode-state checks: `tuiscotti-feedback.md` P9 (output tap), P10 (termios), not posted. Was: T17 port; no hand-rolled `openpty` remains. | — | `rg openpty tests` = 0 | plan §H T17 |
| V7 | T9: measure per-capture time on a real CI run and update §C3 (still estimates). T10: holla-fable-audit workflows, `grep -r "cargo test" .github` empty. | C-items run | numbers from a real run | plan §H T9, T10, §C3 |
| V8 | T18 oracle v0: after the UI work lands, capture everything with `VISUAL_FULL=1` on ubuntu-24.04 (now through the `visual-capture` suite, T8 done), `cargo xtask visual import <actual dir> --oracle --source <commit>`, review the change list per group, commit per app, then `git rm -r snapshots/` in the same series. | V1, V2 (or their rows deleted), the S/SC steps that change captures | smoke and full green on HEAD; `snapshots/` gone; `ORACLE.toml` `pending = false` | plan Progress T18; §B4; §H T18 |
| V9 | **Done** (audit-sync branch, 2026-10-17): plan Progress rows T0/T4/T18/T30/T31, D1, §B5, §D and §H T0 updated; migration status table and legacy paragraph at 1,650; `tuiscotti-feedback.md` P3 row. Was: plan Progress T18 row says "T8 is still blocked" (T8 done); §H T0 still "in flight"; §B5 and §D name `cargo xtask visual accept` (now `tuiscotti accept`); T30 (upstream U1–U8) is done through tuiscotti PR 11 but unmarked; T31 is not in the Progress table; migration status table counts (6,000 / 5,975) predate the holla and jackin approvals. | — | the docs match the repository | plan.md lines 11, 20, 31, 174, 256, 430, 460–461; `tuiscotti-migration.md` status table |
| V10 | Framed panel title cut below `title_min`: `Panel::title_row` counts the frame padding once; fix to `title_min + pad + 1` so the meta takes its short form. Moves framed title rows in all four apps, so it needs its own re-approval commit. | after V8 or with a planned re-approval of all four apps | regression test on the Panels States grid (24-column framed `Preview` with `1–9 of 24` keeps `Preview`); four apps re-approved | `layout.md` "SC22 finding (2026-10-17)" |
| V11 | T19–T21: close the design backlog (§3.5), cut oracle v1 and the `sync-oracle` PR to `main`, then `main`'s `visual.yml` and `visual-ratchet`. T22–T29 are work on `main` (out of this branch). | V8, §3.5 | plan §H T19–T21 | plan §H, order line |
| V12 | **Done** (v12-timed, 2026-10-17): paused captures run a stopped virtual clock for an explicit step: `clock::ClockStep` (in-band paste request), `clock::run_step`, `shell::App::advance_clock` (asked only with the test opt-in `JUNIE_TUI_TEST_CLOCK=1` and under `Motion::Paused`), harness step `advance:<ms>`. Four pending `OVERLAYS` rows with both dismissals: `jackin/overlays/{editor_save_failed,settings_save_failed,editor_role_load_error,cockpit_credentials}`; `overlay_sites` 125 of 125, 0 gaps. Findings F4 (A27) and F5 (A28), seen in these rows, are fixed. Was: the three jackin layers (and A26's role-load error) opened only from a job on virtual time, which never fires under `--motion paused`. | — | rows for `jackin_preview/screens/{editor,settings}.rs::finish_save`, `editor.rs::finish_role_load` and `cockpit.rs::step`; `overlay_sites` 0 gaps | `visual-reapproval.md` F3 resolution |

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
| A1 | **Done 2026-10-17** (`color.md` "A1" section: before/after contrast per pair; fixes d720a1c2e, 3b5badf1b, 1728f3fbb; snapshot-only re-approvals holla 96ce248f9, jackin 930688b88, tablepro 919d8e9d1, showcase 42939dd99). `Theme::frame_on` (16/`none` popup frames Gray, 3.2:1), `Theme::legible_mark` (tab, status-bar, step, tree and activity spinners and marks, and the `▎` focus bar on DarkGray, ≥ 3:1), `Theme::beside_label` (active-tab prefix ≥ 4.5:1 at every level); test `frames_focus_bars_and_tab_marks_read_on_their_plane_at_every_level`. A scan of every accepted 16/`none` capture of the four apps finds no `fg == bg` cell and nothing under 3:1. Recorded: truecolor/256 keep the soft menu frame; the status-bar busy chip gap; showcase captures wait for V1 (digest regenerated). Was: 16-colour readability pass across all four apps: popup menu frames draw `DarkGray` on the `DarkGray` popover plane at 16 colours and `none` (invisible; also in the baseline); on the selected tab's DarkGray plane holla's running spinner `⠋` is 2.9:1 and a queued `○` 3.2:1, jackin's capsule tab number `1` 3.2:1 and the editor dirty `•` 2.4:1. Library fix: Tabs runs its extra inks through `Theme::legible` on the selected fill; popup frames get a distinct 16/none border ink. | one planned re-approval of all four apps (after V1 for the showcase) | every graphic ≥ 3:1, text ≥ 4.5:1, no `fg == bg` cell at 16 and `none` in the accepted captures; a contrast unit test per token | `visual-reapproval.md` holla and jackin "Recorded, not fixed" |
| A2 | **Done** (audit-sync branch, 2026-10-17): every audit file except `performance.md` (already complete) ends with "Status (T19 sync, 2026-10-17)"; each finding reads Fixed (commit), Open → owner, Superseded or Decision (reason); leftovers no step owned became A15–A24. Note: `api.md` #28 was not fixed as listed below, the apps still spell "Not in this preview" (A20). Was: give every finding a status line. No status at all: `color.md` (22), `states.md` (30), `focus.md` #1–3, #5–10, #12–16, `standards-interaction.md` verdicts 1, 3, 7, 8, 10, 13, 14, 16, 21–24 and gaps 1–3, 6, 8, 9, `standards-visual.md` verdicts 1–8, 10–12, 14–16 and gaps 1–8, 10, and the three `duplication-*.md` (mark each row "consolidated into the plan" with its step). Fixed elsewhere but not marked: `keyboard.md` #8, #9, #13, #14, #27, #30, #31, #32 (S5/S7 commits in the plan's Step status); `interaction.md` #1, #8, #10, #11, #14, #17; `api.md` #1, #11, #28 (`not_in_preview` is gone from `src/bin`); `hscroll.md` bar routing (S6); `colour-follow-ups.md` `TabItem::done` (now declared in `src/bin/showcase/coverage.rs`) and its stale "Visual (holla and jackin)" paragraph; the `keyboard.md` caret-pass list (tablepro, holla, jackin since re-approved). | — | each listed id reads "Fixed (commit)", "Open → S*n*/A*n*" or "Decision (reason)" | every file in `docs/audits/design-review-2026-10/`; plan "Step status" |
| A3 | Mouse "Open from this pass": the I-beam shows over a field's label, and TextViewport and diff text declare no I-beam. (The `+N` overflow menu half is fixed: `77fd861c6`, `Interaction::focused_under`, tablepro and chips tests.) | — | pointer-shape unit tests | `mouse.md` "Open from this pass" |
| A4 | Mouse "Left for later": holla `TextModal` (why, about) has no close button (#13; the key reference is the library HelpOverlay since S8b; with layout #11, holla/jackin help onto `KeyReference`); tablepro filter editor flashes nothing on a refused click (#24, needs `Frame`; with S18); jackin cockpit's blocked stage explains itself only in the status line (#6); a two-part crumb (`Disk › Usage`) is one link (#5); `ScrollState::follow` is opt-in (#4); the showcase draws its footer before a dialog's barrier, so a dialog's hints are not clickable there (S6 #1; with S8b); ~~ChoiceDialog's Enter and its double click (#39, S11) fire the primary button even when it is disabled~~ **Done** (dialog-a27-a28): a disabled or busy default button refuses them and the frame flashes in the warning tone until the next input (`choice_dialog::a_disabled_default_button_is_not_fired_by_enter_or_a_double_click`). | S8b, S18 as noted | per item a regression test or a recorded decision | `mouse.md` "Left for later" and S6 #1 |
| A5 | Layout "Not done": `Panel::render` returns `Rect` (#1, `try_render`); `status_min_share` instead of a cap, compact mode off (#6); `spacing.panel`/`spacing.popup` not wired (#12, its own re-approval); tables and grids do not use `scrollbar::split` (#14); `row_layout` callers without `RowOverflow::More` owners (#15); `Sections` only on the audited pages (#17); the quick switcher Picker centres in the whole screen (#4). | #12 after V8 | per item: fixed with a test, or recorded as a decision with a reason | `layout.md` "Not done, with reasons" |
| A6 | Keyboard deferred passes: tablepro workbench quit is still a destructive "Quit TablePro?" instead of Save / Discard / Cancel, and jackin capsule and cockpit keep their own exit dialogs (both with S14); viewports and lists do not report a selection through `KeyContext::selection` (`has_selection()`, with S8b); Cmd+Backspace and Option+Delete unbound; footers spell `y Copy` literally instead of `hint_key` (the InfoDialog reads its key map since S8b; 11 app sites remain, with S16); jackin editor hand-codes `j`/`k` (S16/S17); dialogs show the underlined mnemonic and need re-approval (V1). | S8b, S14, S16 | per item | `keyboard.md` data-loss, Select All, macOS chords and type-ahead passes; #36 |
| A7 | Interaction deferred: `return_activates: Focused\|Default` (#20); Tabs opting in to `FocusRing::{Always, KeyboardOnly}` (#24) together with the focus audit's per-widget strip-gutter override (needs a replacement cue, WCAG 2.4.7); holla files and finder keep `Alt+Enter Alternatives…` as their menu hint (#13 round 2); navigation and modals requested by a background screen are not acted on (#15 round 2, with CO9). | S8a for #15 | per item | `interaction.md` #13, #15, #20, #24; `focus.md` decision log "Follow-up (not done)" |
| A8 | API remainders: `ui::text::{truncate, truncate_middle, fit, slice_cells, wrap}` still cut with a hard-coded `…` (about 300 call sites) (#26); grid and scrollbar position labels, CodeEditor `ln/col` and Meter `refreshing` need a formatter hook (#28); Dialog and Picker keep their own chrome (#3, a visual change). | — | the ellipsis comes from `Glyphs`; formatter hooks with defaults unchanged (digest unchanged) | `api.md` #3, #26, #28 |
| A9 | Copy #5: with legacy key encoding a terminal reports Ctrl+\ as Ctrl+4; add the shared key normalisation. | — | `core::keyboard` unit test | `copy.md` #5 |
| A10 | Colour rows with no follow-up: holla `activities/jump` toast "exit 0" in `success` and the URL in `link`; holla `docker/done` "reclaimed 18.4 GB" in `success`; jackin `manager/tree_expanded` unlabelled amber `•` ("dirty" in the meta or a tooltip); jackin Limits low run in `info` (app row); jackin running-instance panel metas need a per-part tone (`Panel::meta`, `manager.rs:1177, 1194`); tablepro `set_status` Fail has no caller; showcase Overview swatches and labels. The T19 sync found two of these probably done: the showcase Overview swatches and labels (`0610904df`, `e92b78fc6`, `157f9206e`) and the jackin Limits low run (`a942f1081`, `57562ede2`, `colour-holla-jackin.md` R4); confirm against the captures and drop them. | re-approval per app | each row in `colour-follow-ups.md` with a commit or a reason | `colour-holla-jackin.md` M/L/H rows; `colour-follow-ups.md`; `colour-showcase-tablepro.md` #9 |
| A11 | Standards gaps no step owns: a focusable status bar whose items take Enter and show a cut item on the peek line (K38, K12 StatusBar; S10 left it); no room shows as a full-width sheet, never clipped (SI1); app-level profiles and a keymap file (SI3); Alt+mnemonics for menus with F10 shown (SI10 beyond S8a); Tab cycling inside modals (SI22); copy under mouse capture (SIg3 beyond S14); wide-character truncation (SIg10 remainder); showcase tree naming "Chrome" → "App shell" (SV7); editor choice by option count, `▾` only on the active cell, `☐/☑` (SV8); backdrop dim at 16/mono (SV10; the footer showing the modal's hints is done, CO4); `Delete…` ellipsis only when more input follows and `Del` as the key name (SV12; contradicts shipped copy, so the standard becomes the default); DIM fallback (SVg4) and BOLD-as-bright (SVg5). | T, S8b as fitting | per item, with the standard named in the test | `standards-interaction.md`; `standards-visual.md` |
| A12 | hscroll follow-ups (the Shift+wheel knob is already one, `InputConfig::shift_wheel_horizontal`, since cd4515ddc): reveal for cut labels in Tree, List, Picker and DataTable (with S10); jackin capsule pane bars routed by position (`hbar()`). | S10 | reveal tests; a jackin pane bar routing test | `hscroll.md` follow-ups and "Still open" |
| A13 | Showcase navigation leftovers: primary press-drag-release on a button menu is not offered (`release_from` is private) (SC16 2); Toolbar, Tabs `+N`, ChipBar and MenuBar `»` each own their overflow menu, to move onto one (SC27 7); legacy `showcase/flows/settings/toggled_off` is re-recorded on purpose with V1 (SC27 1). | V1 for SC27 1 | one overflow-menu implementation; a press-drag-release test | `showcase-pages-navigation.md` SC16, SC27 |
| A14 | Showcase coverage notes: the Inspector is hidden below 100 columns and only reads state; there is no in-app colour switch (`--motion reduced` is SC30). | — | fixed or recorded as a decision | `showcase-coverage-library.md` notes |
| A15 | Terminal viewport reversed cells (SGR 7) still draw as a white flood (`fg canvas` / `bg text_primary`, `src/widgets/viewport.rs` `c.style.reversed`), the treatment the table and grid cell cursor dropped. Either map them through a theme resolver or record that reverse video in terminal content is content, not a state. | T if fixed | a viewport test on a reversed cell, or a recorded decision | `states.md` #27 |
| A16 | **Done (S8a)**: `KeyRouter::input`/`paste`, `KeyHost::layer_paste`/`page_paste`; test `a_paste_goes_to_the_top_layer_else_the_page`. Paste is still routed by each app (`Input::Paste` in all four `app.rs`); move it into the shared input chain. | S8a (`KeyRouter` on `App::on_input`) | no `Input::Paste` match in `src/bin/*/app.rs`; a paste test through the router | `duplication-shell.md` row 16 |
| A17 | **Done (S14)**: every app takes its header, body and footer from `shell::Chrome::regions`. Header/body/footer `Rect` arithmetic is still done per app; no step names the `AppShell` regions (the too-small screen is shared since S2). | S14 (`Chrome`) | apps take their regions from the shell; no per-app split | `duplication-screens.md` F22 |
| A18 | **Done (S8b)**: `describe_key` is gone; the showcase spells keys with `KeyBinding::label`. | S8b | `describe_key` gone; labels from `core::keyboard` | `duplication-screens.md` F25 |
| A19 | `MenuBar::render` still calls `on_hover(live_hover())` (`src/widgets/menu.rs`), so drawing changes state; hover should arrive through the pointer router (S6) like every other widget. | S15 (render becomes pure) | a test that rendering twice without input leaves the menu bar state unchanged | `api.md` #11 |
| A20 | `Strings::not_in_preview` exists but no app reads it: holla `src/bin/holla/app.rs` and jackin `screens/capsule.rs` still hard-code "Not in this preview". | — | `rg '"Not in this preview"' src/bin` = 0 (tests aside) | `api.md` #28 |
| A21 | Undo in tables (SIg7 remainder): inside a cell edit Ctrl+Z / Ctrl+Shift+Z undo and redo typing since S11 (`EditField`); outside an edit DataGrid undoes pending changes only with `u` and DataTable has no undo. | the grid's undo command (a `GridCmd::Undo` binding), with S15 or alone | Ctrl+Z / Ctrl+Shift+Z on a committed grid edit, with a test; `u` kept as an option | `standards-interaction.md` gap 7 |
| A22 | **Done** (c4-a22 branch, 2026-10-17, `da7d3550f`): `theme::Charset` (`Unicode`, `Ascii`) is detected beside `ColorLevel::detect`: `TERM=linux`, `vt<N>`, `cons25`, `dumb`, `pcansi` (and their `-variant`s) and the legacy Windows console (no `TERM`, `WT_SESSION`, `TERM_PROGRAM` or `ConEmuANSI=ON`) select `Glyphs::ASCII` and `Strings::ASCII`; everything else stays Unicode. `CommonArgs::theme` applies it, so all four apps follow; overrides, strongest first: `--glyphs unicode|ascii`, `JUNIE_TUI_GLYPHS`, detection, then `Theme::with_glyphs`/`with_strings`. The showcase seeds `shell.glyphs`/`shell.strings` from the detected theme. Tests: `theme::tests::charset_follows_the_terminal`, `cli::tests::explicit_glyphs_win_and_reach_the_theme`, showcase `a_detected_ascii_charset_shows_as_the_options_and_survives_a_rebuild`. No locale check (`LANG` is unset in many Unicode terminals). Was: `Glyphs::ASCII` and `Strings::ASCII` are opt-in only; nothing selects them for `TERM=linux` or the Windows console. | T or S17 | a detection unit test (`TERM=linux` picks ASCII) with an override | `standards-visual.md` gap 6 |
| A23 | **Done (S12)**: a Picker without a query type-selects through its `OptionList` (`OptionList::type_ahead`, default from the `NavProfile`: on under Desktop, off under Vim); a searchable one types into its query. Test `a_picker_without_a_query_type_selects`. Left: its footer does not say `Type to jump` yet (the picker's `HasHints` hides the type-ahead hint), a hint-only follow-up with a capture re-approval (S16). | S12 (Picker body on `OptionList`) | a type-ahead test on a non-searchable Picker | `interaction.md` #11 |
| A24 | Done in S9: Alt+click on a picker row reports `ChosenAlt` (`an_alt_click_on_a_row_chooses_its_alternative`, "Alt+click on a row chooses its alternative"). | S9 | an Alt+click test reporting `ChosenAlt` | `interaction.md` #6 |
| A25 | **Done** (dd13d9c2a): jackin Container info (`Ctrl+B i`) panicked on `capsule-multi` by byte-slicing a 7-character run id to 8; every fixed byte slice of a display id goes through the new `ui::text::clip`. | — | `container_info_opens_on_a_short_run_id_without_panicking`; `jackin/overlays/capsule_info` captures (still pending) | `visual-reapproval.md` "Overlay coverage (V4)" F1 |
| A26 | **Done** (6098669b2; decision: reachable, kept): the dialog could not open because only an untrusted Role failed. A trusted Role's load can fail in the product (fetch, I/O, manifest parse), so the fixture's `RoleEntry::fetch_error` fails the trusted `writer` in `hard-cases`, and Enter on the failed row retries. Row `jackin/overlays/editor_role_load_error` since V12 (an explicit clock step). | — | `a_trusted_role_that_fails_to_load_reports_the_error_and_retries` | `visual-reapproval.md` "Overlay coverage (V4)" F2 |
| A27 | **Done**: after a job failed, its progress status ("Saving payments-platform…", "Saving settings…", "Loading role writer…") stayed in the footer under the error dialog until the TTL. A progress status now belongs to its job: `shell::JobKey`, `StatusLine::progress`/`end_job`, `Cx::progress` (`Request::Progress`, allowed in the background). The job's result ends its own progress status before the owner handles it (jackin `dispatch_msg` through `Msg::job`), so a failure that only opens a dialog leaves nothing behind and a newer message is never cleared. Every jackin job-backed progress status moved onto it (workspace and settings save, role load, prewarm, stop, purge, account validation); `Refreshing…` has no job and stays a plain status. | — | `status::a_progress_status_ends_with_its_job`; jackin `a_failed_job_leaves_no_progress_status_under_its_error` | `visual-reapproval.md` "Overlay coverage (V4)" F4 |
| A28 | **Done**: a dialog whose dismissal resolves to a danger action (jackin cockpit credential error, cancel = "Cancel launch") still light-dismissed, so a stray backdrop click cancelled the launch. `Dialog::guard_destructive_dismiss` (default on; showcase option `dialogs.guard_destructive`): while the cancel action is a `ButtonKind::Danger` button, the policy in force refuses the outside click (the frame flashes) and focus out; Esc and the `×` still follow `Dialog::dismiss`. Standard: macOS alert sheets never close on an outside click; WAI-ARIA `alertdialog` needs an explicit answer. | — | `dialog::a_destructive_cancel_refuses_light_dismissal_but_not_esc`; jackin `a_stray_click_does_not_cancel_a_launch_held_on_credentials` | `visual-reapproval.md` "Overlay coverage (V4)" F5 |
| A29 | **Done 2026-10-17** (`standards-interaction.md` "Alt (Option) chords (A29)"). `keyboard::alt_delivered` / `AltKeys` (macOS Terminal and iTerm2 wait for an unambiguous Alt key, WezTerm delivers, other macOS terminals deliver under the kitty protocol, `JUNIE_TUI_ALT_KEYS=on|off`), read by `KeyBinding::deliverable`, `hint_key`, `SplitKeys::hint`, help rows, menu shortcuts and picker hints. Deliverable paths: Ctrl+Shift+F12 zoom, Ctrl+Shift+arrows resize stand-in (`KeyBinding::alt_stand_in`), jackin Ctrl+B H J K L, tablepro Shift+F5 / Shift+F7. Open: picker alternate choice has no deliverable key (recorded). | — | `keyboard` and `keymap` unit tests per platform/terminal; a footer test per app (macOS Terminal vs kitty protocol); re-approvals attributed | AGENTS.md "Hints never advertise keys the terminal cannot deliver"; `standards-interaction.md` Gap 4 |

Not tasks (watch list): `performance.md` "Remaining risks" (stale-frame
wheel notches, reveal wrap cost, a full render per frame,
`TreeView::flatten` O(rows), the wall-clock guard).

### 3.6 Upstream and dependencies

| Id | What | Depends on | Acceptance | Source |
|---|---|---|---|---|
| U1 | velnor Problem D (§9): file a velnor-new PR so `publish_gate` (`baseline_publish.rs:224-244` at v0.1.9, unchanged through v0.1.12) accepts the configured default branch instead of `repository.default_branch`; then adopt the release, regenerate and confirm `Publish baseline` is green on a push. Not fixable by config (`default_branch = "main"` would stop push CI here). | — | the three §9 acceptance boxes checked; tracker row closed | `upstream/velnor-new-feedback.md` tracker row "Problem D" and §9 |
| U2 | velnor Problem C follow-up (2): without a toolchain file the default is still `rust@1.98.1`; catalog zizmor 1.30.1 (1.31.1 out). Non-blocking; raise upstream. Re-checked on v0.1.12 (adopted 2026-10-17): still open. | — | velnor release with the new defaults adopted | `velnor-new-feedback.md` tracker, Problem C (2); `ci.md` line 56 |
| U3 | velnor Problem C follow-up (3): `[tools]` versions are only shape-checked (`zizmor = "99.99.99"` generates). Then raise `[tools] zizmor = "1.31.1"` in `.velnor/config.toml` (and `mise.toml`) once a CI run proves it installs. Re-checked on v0.1.12 (adopted 2026-10-17): still only shape-checked, zizmor held at 1.30.1. | a velnor fix, or a CI run that proves the pin installs | zizmor 1.31.1 in CI; `dependencies.md` row updated | `velnor-new-feedback.md` Problem C (3); `dependencies.md` versions table; `ci.md` lines 58–60 |
| U4 | crossterm 0.29 Linux input starvation: remove the re-arm in `src/runtime.rs` (`poll_input`, `tty_input::rearm`, and review the related `SizeWatch` SIGWINCH note) once crossterm reads until `WouldBlock`; file the upstream issue if D7 says so and track it in `docs/upstream/`. | D7; a crossterm release with the fix | `holla_pty` flood test passes on Linux without the re-arm | `dependencies.md` crossterm paragraph; `src/runtime.rs` `poll_input` doc, `rearm`, `SizeWatch` doc |
| U5 | Keep the tuiscotti pin on the newest `main` rev (AGENTS.md hard rule): on each move, bump `rev =`, port, re-run the gate and re-approve on purpose. Tracker is otherwise all done; its only open line is V1. Last bump 2026-10-17: `3382e57` → `a1ea27f` (CI-only upstream commits, no capture changed); recurring, re-check on every upstream move. | — | `Cargo.toml` rev = tuiscotti `main` head | AGENTS.md "Tooling"; `upstream/tuiscotti-feedback.md` tracker |
| U6 | Transitive holds `generic-array` 0.14.7 and `psm` 0.1.24 (optional `termwiz` path, not built): no action until their pinning crates move; re-check on the monthly update. | upstream crates | `dependencies.md` row removed | `dependencies.md` transitive table |
| U7 | velnor lock audit: `rust@<channel>(no_checksums)` stays in `tool_install_unverified` with a complete `mise.lock`, because mise's `core:rust` backend (rustup) never records checksums. Raise upstream: exempt or separately label `core:rust` (rustup verifies its own downloads). | — | a velnor release whose `generate` is silent with our `mise.lock` | C4; `ci.md` "Tool lock" |

### 3.7 CI (docs/ci.md, .velnor/config.toml)

| Id | What | Depends on | Acceptance | Source |
|---|---|---|---|---|
| C1 | Enable `visual-smoke` on `["pull_request", "push", "workflow_dispatch"]` with `required = true`; regenerate with `velnor-actions generate` (never hand-edit `.github/`). | V8 | `visual-smoke` is part of `Required` and green | `ci.md` "Visual suites"; `.velnor/config.toml` lines 35–38, 57 |
| C2 | Give `visual-full` and `visual-narrow` `"schedule"` with `schedule = ["0 2 * * *"]`. | V8; V3 for narrow | nightly runs green | `ci.md` "Visual suites"; config lines 70, 83 |
| C3 | `Publish baseline` green on push (red on every push today, not part of `Required`; still so with v0.1.12). | U1 | job green on a push | `ci.md` jobs table, `Publish baseline` row |
| C4 | **Done** (c4-a22 branch, 2026-10-17): `mise.lock` committed, produced with `mise lock --platform linux-x64,macos-arm64` (mise 2026.10.11); it locks every `mise.toml` pin and has a checksum for both platforms on every artifact tool (actionlint, shellcheck, zizmor, gh, cargo-nextest, velnor-actions; the velnor checksums equal `.velnor/release-manifest.json`). `velnor-actions generate` (0.1.12) now reports `tool_install_unverified:1 installs skip verification: rust@1.99.1(no_checksums)` instead of 6 installs; generated files unchanged, drift check and actionlint clean; xtask `mise_lock_covers_every_mise_pin` guards the lock. Residual, not fixable in this repository: mise's `core:rust` backend installs through rustup and records no checksum on any platform (also not when installed on the CI platform), so the Rust line stays until velnor treats `core:rust` as unverifiable by design (U7). Was: every `velnor-actions generate` warned `tool_install_unverified:6 installs skip verification (no mise.lock)`. | — | `generate` lists no artifact tool as unverified | `ci.md` "Tool lock"; `dependencies.md` "Tools via mise" |

### 3.8 Responsive apps (owner priority 2026-10-17)

Every app screen keeps all content reachable at any size
([`audits/design-review-2026-10/responsive.md`](audits/design-review-2026-10/responsive.md)).
The reachability gate (`src/shell/reach.rs`, `app_tests_reach.rs` per
app) runs in the default nextest run; `tests/reach_allow.txt` holds each
known gap with its row id here, and a fix drops its lines. Branch
`responsive-apps`; the two-axis scroll container comes from
`responsive-lib`.

| Id | What | Depends on | Acceptance | Source |
|---|---|---|---|---|
| R-apps-1 | Segments and StatusBar drop whole items by priority with no `…` and no reveal (tablepro and jackin top strips, holla and jackin status bars). | — | dropped items collapse into a revealing `…` item; their allow lines gone | `responsive.md` R-apps-1 |
| R-apps-2 | Hand-rolled master-detail drops the detail or aside below a width (tablepro connections and EXPLAIN card, jackin manager aside, editor/settings detail, usage swap; holla plan summary over the drawer). | `MasterDetail` | screens on `MasterDetail`; allow lines gone | `responsive.md` R-apps-2 |
| R-apps-3 | Tree/List meta columns and hand-rolled tables drop columns without `…` (explorer counts, history durations, holla files and plan, jackin mounts). | — | meta cut with `…` and revealed; tables on `widgets::table` | `responsive.md` R-apps-3 |
| R-apps-4 | Text cut with `…` but no reveal (input placeholder and help, picker hint, PageHeader, Heading, app line lists). | — | one cut helper that registers the reveal | `responsive.md` R-apps-4 |
| R-apps-5 | Line clamps drop text without `…` (Notice `max_lines`, tablepro safe-mode description). | — | clamps marked and revealed or scrolled | `responsive.md` R-apps-5 |
| R-apps-6 | Fixed-size bodies drop rows with no scrollbar (`props::render`, fact sheets, detail cards, holla disk compact mode, jackin settings and cockpit). | responsive-lib (scroll container) | bodies scroll; allow lines gone | `responsive.md` R-apps-6 |
| R-apps-7 | Button rows drop the buttons that do not fit (`button::row_layout`). | — | wrap or `…` menu | `responsive.md` R-apps-7 |
| R-apps-8 | Cut cells reveal on hover only; Picker has no `→` reveal (WCAG keyboard gap). | — | keyboard reveal in Picker; audit drives it | `responsive.md` R-apps-8 |
| R-apps-9 | **Done 2026-10-17**: a scroll pane registers its whole body under its children (`HitRegistry::register_scroll_under`, from `ScrollPane::draw_with_fade`), so the wheel scrolls a dialog or overlay over a field, a label or the padding beside its bar; FormDialog follows focus only when focus moves, so the wheel can scroll away from the focused field. Tests `the_wheel_over_a_form_field_scrolls_the_form` (jackin), `the_wheel_beside_the_help_scrollbar_scrolls_the_help` (tablepro). | — | done | `responsive.md` R-apps-9 |
| R-apps-10 | **Done 2026-10-17**: a Dialog with a code preview shrinks the preview (down to one line, the rest "… N more") before its facts get fewer than 2 rows; at 72×20 holla start-plan shows its facts again. Test `a_short_screen_keeps_a_dialogs_facts_beside_its_code_preview` (holla). | — | done | `responsive.md` R-apps-10 |
| R-apps-14 | A Dialog's code preview past its shown lines is only summarised ("… N more"): those lines are reachable at no size (holla start-plan; at 72×20 its shrunk preview hides `apt-get update`). | — | the preview scrolls vertically (or joins the body scroll) | `responsive.md` R-apps-14 |
| R-apps-11 | Tabs behind the `+N` overflow (reachable). | — | recorded (kind b) | `responsive.md` R-apps-11 |
| R-apps-12 | Audit limits (3-row grid sweep budget, mid-word wrap, page changed under a modal). | — | recorded (kind c) | `responsive.md` R-apps-12 |
| R-apps-13 | Too-small screen below 72×20 in every app. | R-apps-2, R-apps-6 | each app's true minimum | `responsive.md` R-apps-13 |

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
