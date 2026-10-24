# Runtime correctness review — Termrock delivery branch, round 2026-10-22

- Reviewed commit: `bea5350ec99b0652335714275fd3f0d078399bca` (`bea5350ec9 feat(holla): width-band layout policy and a real history setting`)
- Read-only worktree: `/Users/donbeave/Projects/review-runtime-20261022` (detached at the reviewed sha)
- Baseline worktree for red/green attribution: `/Users/donbeave/Projects/review-runtime-base-20261022` at `5dfff7f20f` (parent of `ef35a7f6b`, the first commit of the stream)
- Private `CARGO_TARGET_DIR`: `/private/var/folders/8p/.../opencode/review-runtime-target` (and `.../review-runtime-base-target` for the baseline)
- Reviewer: independent runtime-correctness reviewer (read-only; no code changed in the reviewed tree)

Stream under review (all reachable from the reviewed sha): `ef35a7f6b`, `493f9054f`, `849d26dad7`, `150320ab3`, `fc2c3bd4e`, `b4b1a33468`, `ec783f6c5`, `a972caaf1d`, `bea5350ec9` and the docs commits between them.

---

## Findings

### Accounts page (jackin-preview-app)

`crates/jackin-preview-app/src/app/accounts_page.rs:581`: 🔴 blocker: the accounts tree hit regions are off by one row — the shared `List` owner is drawn into `list_area()` at `area.y+2` while the shared painter draws the card title at `area.y+2` and the tree rows from `area.y+3`, so a click lands on the row below the one painted (measured: clicking the painted "▾ Claude" row selects `ItemKey::Index(2)` "★ Personal"; clicking the painted "Work" row selects `ItemKey::Index(5)` "Codex"). Fix: draw the list owner into the tree row band (`area.y+3`) or let the shared painter own the row hit regions.

`crates/jackin-preview-app/src/app/accounts_page.rs:1508`: 🔴 blocker: the narrow accounts list never scrolls with the cursor — `detail_rows()` hands every row to `paint_accounts_detail`, which paints `page.rows` from index 0 and uses `offset` only to place the scrollbar thumb (`crates/termrock-feedback/src/accounts_detail.rs:115`); with the cursor at index 20 the pane still shows rows 1–11 while the thumb and the header report row 16. Fix: slice the rows by `offset` before handing them to the painter, or make the painter honour `offset`.

`crates/jackin-preview-app/src/app/accounts_page.rs:518`: 🔴 blocker: the narrow range indicator is not clamped to the total — it prints `offset+1 … offset+visible` and the measured frame reads "16–26 of 23" for 23 rows. Fix: clamp the upper bound to `total`.

`crates/jackin-preview-app/src/app.rs:3936`: 🔴 blocker: a committed filter leaves the accounts list without key handling — `update_accounts` returns early while `accounts.filtering` is true, and committing only calls `cx.focus(ACCOUNTS_LIST)` without clearing `filtering`, so `List::update` is never called again; the filter field is not painted either (the focus left it), so the page is key-dead (measured: after `/` + "Claude" + Enter the cursor stays `Index(0)` and Down does nothing). Fix: clear `filtering` on commit, or keep updating the list while the filter is committed.

`crates/jackin-preview-app/src/app/accounts_page.rs:92`: 🟠 major: the new accounts hint bar advertises keys that do not perform the advertised action — "e Edit…" (e is `CMD_EDITOR_OPEN`, handled only on the Manager route), "t Disable" (no binding at all), "d Remove…" (d is `CMD_CAPSULE_DETACH`; the remove chord is `x`), "m Usage" (m is `CMD_MANAGER`; Usage is `u`), and "r Refresh all" on the overview (`CMD_ACCOUNT_REFRESH` refreshes the selected account and does nothing when the selection is Overview). Measured: after `e`, `t` and `d` the app state is unchanged; after `m` the route is `Manager`; on the overview `r` does nothing. Fix: correct the hint lists to the real chords.

`crates/jackin-preview-app/src/app.rs:7514`: 🟠 major: the accounts route still keys on a fixture label and a captured status string — `accounts_work_drawer_enter()` requires `selected_id == "acct-claude-work"`, `list.cursor() == ItemKey::index(4)`, `status == "Accounts › Claude › Work"`, `motion == Motion::Paused` and `scenario == AccountsMixed`, and it still gates the Enter binding in `sync_workspace_keymap` and the `CMD_EXIT_CONFIRM` handler; the stream removed the paint gates but left this key gate, and the new guard test cannot see it. Fix: drop the fixture predicate and route Enter through the list.

`crates/jackin-preview-app/src/app.rs:10425`: 🟠 major: the accounts route returns before `draw_content`, so the shell panel, the app footer and `Self::account_add_button().draw(...)` are never drawn on that route; the add control survives only through the hand-rolled `register_control` at `crates/jackin-preview-app/src/app/accounts_page.rs:573`, whose rect starts at `area.x+1+name_x` = column 8 while the painted label "+ Add account…" starts at column 7, so the leading "+" glyph is not clickable (measured: click at x=7 does nothing, x=8…22 open the picker). Fix: draw the add button from the page, or align the registered rect with the painted label.

`crates/jackin-preview-app/src/app/accounts_page.rs:553`: 🟡 minor: `register_add_row` is only called from `paint_wide`; the narrow path never registers the add control and never paints the "+ Add account…" row, so the tree's last row is unreachable in the narrow layout. Fix: register and paint the row in both layouts.

`crates/jackin-preview-app/src/app/accounts_page.rs:1438`: 🟡 minor: `truncate` counts `chars()`, not terminal cells, and cuts inside a grapheme cluster, so a wide or combining account name is cut to the wrong column width; `list_title` (`:1592`) and `narrow_name_width` (`:1539`) repeat the `chars().count()` mistake. Fix: cut with `termrock::width` over grapheme clusters.

`crates/jackin-preview-app/src/app.rs:2202`: 🟡 minor: `accounts_page::tree(self)` rebuilds the whole tree (a `Vec<TreeRow>` with several `String`s per row) on every call and is called from `account_rows()` (every frame), `selected_account_id()` (twice per frame) and `account_row_index()`; the replaced `selected_account_id(world, key)` was allocation-free. Fix: build the tree once per frame and share it.

### Theme oracle

`crates/termrock-theme/src/builtin/junie.rs:48`: 🔴 blocker: the theme oracle commit `ec783f6c5` reverts two corpus-measured restorations and rewrites the tests that pinned them — muted/faint go back to `0x8a8a8a`/`0x737373` (which `b4b1a33468` measured as never occurring in the approved corpus, against `0x808080`/`0x4d4d4d`), the scrollbar track goes back to `┆` (measured U+2502 12600 times against 6 of U+2506) and the SQL syntax tokens go back to blue/teal/purple (`849d26dad7`: "a blue keyword is not in it"); the same commit edits `crates/termrock-theme/tests/capability_palettes.rs:68`, `crates/termrock-theme/src/builtin/mod.rs:1207` and `crates/termrock-theme/src/glyph.rs:335` so the pinning expectations follow the new values. Fix: restore the corpus-measured values, or land the reference decision with a re-blessed corpus before moving the tokens.

`crates/termrock-overlays/src/picker.rs:1228`: 🟠 major: the theme change collapses two distinct inks and breaks the picker query-field contract — the test asserts the placeholder, the family theme, the themed and the patched query ink are all different, and with `fg[2] == 0x8a8a8a` the placeholder ink and the theme ink are now the same value (`left: Rgb(138,138,138)`, `right: Rgb(138,138,138)`). This test is green at the base and red at the reviewed sha. Fix: keep the query placeholder on its own ink, or restore the ramp.

`crates/termrock-overlays/tests/context_menu_danger_profile.rs:205`: 🟠 major: the mono tier of the danger profile regresses — the active/press row now paints `White` where the frozen ladder expects `DarkGray`, and the disabled row resolves to the active style (`black().on_white().bold()`) instead of `gray().on_dark_gray()`. Both are green at the base and red at the reviewed sha. Fix: restore the mono ladder values the danger profile expects.

`crates/termrock-theme/tests/empty_style_inheritance.rs:289`: 🟡 minor: the standalone empty-cell digest changed (`6494524449778408503` against the expected `312677554172290677`); green at the base, red at the reviewed sha. Fix: re-bless with the reference decision or restore the tokens.

`crates/holla-ui/src/app.rs:2574`: 🟡 minor: the hovered menu title now paints `Rgb(138,138,138)` where the hover contract expects `Rgb(255,255,255)`; green at the base, red at the reviewed sha (the menu title identity change in `849d26dad7` changed the resolved ink). Fix: keep the hovered title on the primary ink.

`crates/termrock-theme/src/downgrade.rs:700`: ⚪ nit: the commit message says "the crate did not compile before it"; only the test target did — `cargo build -p termrock-theme` succeeded, and the test module called `viewport_mono_rules()` and six siblings that did not exist. Fix: correct the message.

### Showcase pages

`crates/showcase-demos/src/pages/terminal.rs:71`: 🟠 major: `paint_rail_count` writes the stage count over the rail panel's title row — at 80x24 the title reads "Step rail0 of 7 · running" and at 72x20 the title is cut to "Step r0 of 7 · running" because the count overwrites "ail". Fix: give the count the panel's meta slot or reserve the title width.

`crates/showcase-demos/src/pages/terminal.rs:280`: 🟠 major: the hard-coded "0.7 s" label is painted over the Steps widget's own state label — the label is drawn at `inner.right()-5` on the first step row, and a full-width paint of the page shows "ru0.7 s" at 120x40, 80x24 and 72x20 (the captured 120x40 shell frame shows the same cell range as "  0.7 s", i.e. the step's state column is covered). Fix: drop the label or place it where the step row has no state text.

`crates/showcase-demos/src/pages/terminal.rs:317`: 🟠 major: the two footer lines are painted on the viewport panel's bottom border row — the frame reads "status: ready──────────────────────────────╯", i.e. the border is overwritten. Fix: reserve a row inside the panel or drop the footer.

`crates/showcase-demos/src/pages/terminal.rs:334`: 🟠 major: the terminal hints advertise keys the `TextArea` does not implement — "f Follow", "y Copy" and "Esc Clear" (the `TextArea` binds `Esc` to `TextCmd::Commit`, labelled "Done") and "Home End — Oldest / live" (`Home`/`End` move to line start/end; only `Ctrl+Home`/`Ctrl+End` reach the document). Fix: drop or correct the hints.

`crates/showcase-demos/src/pages/taskrunner.rs:399`: 🟠 major: the task runner hints advertise "← → Fold" on the steps rail, but `Steps` (`crates/termrock-navigation/src/steps.rs:116-186`) binds only Up/Down/k/j/PageUp/PageDown/Home/End/g/G/Enter and has no fold. Fix: drop the fold hint or implement folding.

`crates/showcase-demos/src/pages/chrome.rs:1083`: 🟠 major: the chrome page hints advertise "m Context menu" and "right-click Context menu", but the page's `update` only handles `Brand` — there is no menu bar, no context menu and no ↑/↓ handling on the page. Fix: drive the page from real widgets or drop the hints.

`crates/showcase-demos/src/pages/chrome.rs:274`: 🟠 major: the section strip is implemented twice — the page's private `paint_section_strip` and the new shared `pages::section_strip` (`crates/showcase-demos/src/pages/mod.rs:136`) — with the same tab list and the same hard-coded junie RGBs (canvas `0x000000`, accent `0x48e054`, tab `0x27272a`); the shared copy carries seven unused constants silenced by `let _ = (CANVAS, WHITE, …)`. Fix: keep one implementation and resolve the colours through the theme roles.

`crates/showcase-demos/src/pages/chrome.rs:46`: 🟠 major: the chrome "playground" is a hand-painted mock, not live widgets — the page paints a menu bar, a status bar with its own priority-dropping planner (`status_plan`, `:653`), a session card, meters and a hint bar from raw cells with hard-coded junie RGBs, so the page demonstrates none of the components its blurb claims; `READOUT[2]` ("last MenuBarAction: —") is a frozen literal that can never change. Fix: build the playground from the `MenuBar`, `StatusBar` and `HintBar` widgets.

`crates/showcase-demos/src/pages/taskrunner.rs:343`: 🟡 minor: the pipeline card's inner geometry is hard-coded to 13/6/7/10 rows, so at 72x20 the steps area collapses to zero height and the log is cut; the page's own guard test only covers 120x40 and 80x24. Fix: derive the rows from the panel height and add the small sizes to the guard.

`crates/showcase-demos/src/pages/terminal.rs:150`: 🟡 minor: `advance()` runs on every `Page::update` whatever the update cause, so a mouse move or a key press also moves the launch one stage forward, and with `--motion paused` the showcase skips the page update on tick causes (`crates/showcase-ui/src/app.rs:1860`) so the terminal page never advances and `--frame 60` cannot reach the settled frame; the page has no `seek_paused`. Fix: advance on a clock, and implement `seek_paused`.

`crates/showcase-demos/src/pages/terminal.rs:83`: 🟡 minor: the rail count hard-codes `Color::Rgb(115,115,115)` on `Color::Rgb(17,17,17)` instead of resolving theme roles, so it does not follow the active theme and its background patch does not match the panel surface; the same bypass is in `crates/showcase-demos/src/pages/taskrunner.rs:74` (`ink(0x8a8a8a, 0x111111)`) and in the chrome colour constants. Fix: resolve through `Theme`/`FgStep`/`Surface`.

`crates/showcase-demos/src/pages/taskrunner.rs:429`: ⚪ nit: `"─".repeat(...)` allocates a `String` on every draw in the paint path (`chrome.rs:320`, `chrome.rs:326`, `chrome.rs:420` do the same). Fix: paint the rule cell by cell or cache it.

`crates/showcase-demos/tests/no_false_parity.rs:29`: 🟡 minor: the guard greps three source files for three literal strings — a size gate written as `area.width >= 120`, `matches!(area.width, 120)` or moved into a helper would pass the guard; the guard also does not cover the showcase shell or the jackin app. Fix: assert on behaviour (render the page at several sizes and compare the frames) instead of on source text.

### Holla

`crates/holla-ui/tests/visual.rs:25`: 🟠 major: the frozen holla visual baseline now captures `App::for_scenario_without_history(...)` while the shipped default (`crates/holla-ui/src/app.rs:1222`, `history_enabled()` → `true` unless `HOLLA_NO_HISTORY=1`) has `home.history = true`; the oracle therefore covers a configuration the app never ships in, and the default path has no baseline at all. Fix: capture the default configuration, or add a second baseline set for it.

`crates/holla-sim/src/catalogue.rs:897`: 🟡 minor: `launch_failure_activity_frame` widens the exact-size gate to `80..=160` but does not use the new `layout::Band` policy and still keys on the captured clock value `w.now_ms() == 80`, so the "width-band layout policy" is not applied on this path. Fix: express the gate with `layout::Band` and a state predicate instead of a fixture millisecond.

`crates/holla-ui/src/app.rs:625`: 🟡 minor: `paint_hard_cases_status` and the sibling `narrow`/`mid` switches dropped the height condition when moving to `Band`, so 81x24, 80x40 and 99x30 now take the narrow branch where they took the generic one; none of those sizes is in a captured set, so the change is unverified. Fix: keep a height floor or capture the new sizes.

### Test integrity

`crates/jackin-preview-app/tests/accounts_page.rs:104`: 🟡 minor: the accounts guard greps `accounts_page.rs` for literal needles and `app.rs` for five names; it cannot see the surviving fixture gate at `crates/jackin-preview-app/src/app.rs:7514` nor a size gate spelled differently. Fix: assert behaviourally (drive the app and compare the frames).

`crates/jackin-preview-app/tests/app_tests.rs:1780`: 🟡 minor: the drawer test lost its colour assertions — the crumb cell's `fg`/`bg`/`BOLD` checks were replaced by a negative assertion that the crumb is absent, so a drawer colour regression is no longer caught. Fix: assert the drawer's own ink instead of deleting the check.

---

## Verification of the findings raised by the other reviewer

Confirmed by this review, with the measurements above:

All six were re-measured where a measurement was possible; the tuiscotti PTY captures were run against the stored `baselines/tuiscotti-v1` frames (one combo each: `120x40/truecolor`), with `TERMROCK_ALLOW_MIDTEST_BUILD=1 TERMROCK_PARTIAL_RUN=1 COMBO_FILTER=...`, so they are **non-acceptance** runs.

1. "the accounts hit region ignores the list scroll offset" — **confirmed in substance and sharpened**: the live list owner is one row above the painted rows, so every tree click selects the row below the clicked one (`accounts_page.rs:581`). The scroll-offset half of the claim is a different defect: the narrow list never scrolls at all (`accounts_page.rs:1508`).
2. "a committed filter leaves the list without key handling" — **confirmed**, measured (`app.rs:3936`).
3. "the new jackin hint bar advertises keys that do not perform the advertised action" — **confirmed**: `e`, `t`, `d`, `m` and the overview's `r Refresh all` (`accounts_page.rs:92`).
4. "the live showcase task runner regresses the stored frame at every size" — **confirmed at 120x40** (`showcase/taskrunner/default/120x40/truecolor`): the capture completes in 2.82 s and the frame differs from the stored corpus frame in **2507 cells** (pixel similarity 0.570). The other sizes were not re-measured. Note that `docs/architecture/termrock-migration-record.md` D-009 records the `baselines/tuiscotti-v1` corpus as *not* the oracle, so the corpus mismatch is expected on its own and the corpus frames were not regenerated with the stream.
5. "the live jackin accounts page breaks three stored frames" — **confirmed**: all three captures complete and differ from the stored corpus frames — `jackin/accounts/detail/120x40/truecolor` **1211 cells** (similarity 0.887), `jackin/accounts/drawer/120x40/truecolor` **1268 cells** (0.884), `jackin/accounts/filter/120x40/truecolor` **1178 cells** (0.963). The waits ("Accounts › Claude › Work", "Tab Actions", "Accounts · filt") all matched, so the live page is navigable and the capture flow works.
6. "the showcase terminal page is not capturable (the harness times out after 60.6 s)" — **refuted at the reviewed sha**: `showcase/terminal/default/120x40/truecolor` captured successfully in about 3 s of capture time (the case's 82 s was dominated by a 79 s mid-test `cargo build --bin showcase`) and the captured frame shows the settled page with **"7 of 7"** on the rail title row, so the `wait:7 of 7` condition is met and no timeout is reported. The underlying mechanism the reviewer names is real but harmless for that case: the page advances one stage per `Page::update` (36 updates to settle) and has no `seek_paused`. It does bite in paused motion, where the showcase skips the page update on tick causes (`showcase-ui/src/app.rs:1860`), so `--motion paused --frame 60` can never reach the settled frame (`terminal.rs:174`) — that is a separate defect (see the findings).

Not reproduced as stated: finding 6 (the terminal page is capturable).

### PTY capture runs (tuiscotti, one combo each, non-acceptance)

```
TERMROCK_ALLOW_MIDTEST_BUILD=1 TERMROCK_PARTIAL_RUN=1 COMBO_FILTER="showcase/terminal/default/120x40/truecolor" \
  cargo nextest run -p termrock-conformance --test visual_baseline --run-ignored only -E 'test(/showcase_pages_terminal_default/)'
  -> capture ok, "7 of 7" reached, 2704 differing cell(s) vs baselines/tuiscotti-v1, similarity 0.587

COMBO_FILTER="showcase/taskrunner/default/120x40/truecolor" \
  cargo nextest run -p termrock-conformance --test visual_baseline --run-ignored only -E 'test(/showcase_pages_taskrunner_default/)'
  -> capture ok (2.82 s), 2507 differing cell(s), similarity 0.570

TERMROCK_ALLOW_MIDTEST_BUILD=1 TERMROCK_PARTIAL_RUN=1 COMBO_FILTER="120x40/truecolor" \
  cargo nextest run -p termrock-conformance --test visual_baseline --run-ignored only -E 'test(/jackin_accounts_(detail|drawer|filter)_/)'
  -> detail 1211, drawer 1268, filter 1178 differing cell(s); all waits matched
```

The differences include the theme ramp (`expected fg=#808080` against `actual fg=#8a8a8a`) that `ec783f6c5` moved, and the old shell breadcrumb (`expected "/" fg=#808080` against `actual " " fg=#ffffff`), i.e. the stored corpus frames predate the current showcase shell. Per D-009 the corpus is not the oracle; these numbers are reported as facts, not as a parity verdict.

Excerpt of the captured terminal frame (`showcase/terminal/default/120x40/truecolor`, settled):

```
 Components           54  Terminal  Viewport with scrollback, selection and copy · drag the seam · step rail reports a…
    Overview              ╭ Viewport ──────────────────────────────────────────────╮    Step rail               7 of 7
  ▸ Inputs & forms        │       0.37s  info   Fetching crates.io index        ┆  │     ⠋ 01 Resolve workspace  0.7 s
  ▸ Tables                │       0.74s  info   Compiling proc-macro2 v1.0.86   ┆  │     ✓ 02 Pull base image     done
```

The `wait:7 of 7` condition is satisfied, and the "0.7 s" label sits on the first step row over the step's own state column.


---

## Commands run and results

Reviewed tree (`bea5350ec9`), private target dir:

| command | result |
|---|---|
| `cargo build --workspace --all-targets` | EXIT=0 |
| `cargo nextest run -p showcase-demos -p showcase-ui -p jackin-preview-app -p holla-ui -p holla-sim -p termrock-theme -p termrock-overlays --all-targets` | 925 tests, **912 passed, 13 failed** (413.978 s) |
| `cargo clippy --all-targets -p showcase-demos -p showcase-ui -p jackin-preview-app -p holla-ui -p holla-sim -p termrock-theme -p termrock-overlays` | clean, no warnings, no errors |
| `cargo fmt --check -p showcase-demos -p showcase-ui -p jackin-preview-app -p holla-ui -p holla-sim -p termrock-theme -p termrock-overlays` | EXIT=0, no diff |
| `cargo build -p termrock-theme` | EXIT=0 |

Baseline tree (`5dfff7f20f`), separate private target dir:

| command | result |
|---|---|
| `cargo nextest run -p showcase-demos -p showcase-ui -p jackin-preview-app -p holla-ui -p holla-sim -p termrock-theme -p termrock-overlays --all-targets` | 909 tests, **10 failed** |

### Failing tests at the reviewed sha (13)

```
holla-ui app::tests::menu_bar_pointer_hover_changes_only_the_file_title_and_clears_on_exit
holla-ui::visual holla_visual_baseline
jackin-preview-app::visual jackin_visual_baseline
showcase-ui::app_tests editor_playground_keeps_live_find_completion_and_run_contracts
showcase-ui::app_tests editor_shell_completion_accepts_enter_then_tab_indents
showcase-ui::app_tests editor_shell_completion_accepts_tab_then_next_tab_indents
showcase-ui::page_headings all_page_headings_match_the_reference_capture_matrix
showcase-ui::page_headings all_shell_headers_match_reference_captures_and_visibility_thresholds
showcase-ui::visual showcase_visual_baseline
termrock-overlays picker::tests::query_field_resolves_picker_family_theme_and_part_patch
termrock-overlays::context_menu_danger_profile disabled_active_danger_profile_returns_to_popover_surface
termrock-overlays::context_menu_danger_profile keyboard_active_danger_row_uses_frozen_background_ladder
termrock-theme::empty_style_inheritance standalone_empty_cells_match_pre_inheritance_reference
```

### Failing tests at the baseline sha (10)

```
holla-ui::visual holla_visual_baseline
jackin-preview-app::visual jackin_visual_baseline
showcase-ui::app_tests editor_playground_keeps_live_find_completion_and_run_contracts
showcase-ui::app_tests editor_shell_completion_accepts_enter_then_tab_indents
showcase-ui::app_tests editor_shell_completion_accepts_tab_then_next_tab_indents
showcase-ui::page_headings all_page_headings_match_the_reference_capture_matrix
showcase-ui::page_headings all_shell_headers_match_reference_captures_and_visibility_thresholds
showcase-ui::visual showcase_visual_baseline
termrock-overlays menu::tests::menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence
termrock-theme::capability_palettes all_73_junie_slots_match_reference_capability_output
```

### Red/green delta introduced by the stream

Newly red at `bea5350ec9` (green at `5dfff7f20f`):

```
holla-ui app::tests::menu_bar_pointer_hover_changes_only_the_file_title_and_clears_on_exit
termrock-overlays picker::tests::query_field_resolves_picker_family_theme_and_part_patch
termrock-overlays::context_menu_danger_profile disabled_active_danger_profile_returns_to_popover_surface
termrock-overlays::context_menu_danger_profile keyboard_active_danger_row_uses_frozen_background_ladder
termrock-theme::empty_style_inheritance standalone_empty_cells_match_pre_inheritance_reference
```

Newly green at `bea5350ec9`:

```
termrock-overlays menu::tests::menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence
termrock-theme::capability_palettes all_73_junie_slots_match_reference_capability_output
```

`docs/architecture/termrock-migration-record.md` D-010 sanctions only the **component digest** gate to stay red pending a reference decision; the five newly red tests above are behavioural assertions outside that gate and are not covered by the decision record.

### Probe tests written for this review (not committed, run in the reviewed worktree)

```
crates/jackin-preview-app/tests/zz_review_probe.rs
crates/jackin-preview-app/tests/zz_review_probe2.rs
crates/jackin-preview-app/tests/zz_review_probe3.rs
crates/jackin-preview-app/tests/zz_review_probe4.rs
crates/jackin-preview-app/tests/zz_review_probe5.rs
crates/showcase-demos/tests/zz_review_probe.rs
```

They were used only to measure the behaviours quoted above (list cursor after a committed filter, hint-key effects, tree click targets, the add-row hit sweep, and the rendered showcase frames at 120x40 / 80x24 / 72x20). They are not part of the delivered suite and were removed after the measurement.

---

## Verdict

**HELD.**

Scope of the verdict: the commits `ef35a7f6b`, `493f9054f`, `849d26dad7`, `150320ab3`, `fc2c3bd4e`, `b4b1a33468`, `ec783f6c5`, `a972caaf1d` and `bea5350ec9` on the delivery branch, reviewed at `bea5350ec99b0652335714275fd3f0d078399bca`, read-only.

The false-parity removals themselves are real: the exact-size branches are gone from the three showcase pages (`chrome.rs` had four `area.width == … && area.height == …` gates, `terminal.rs` had four, the jackin accounts views had size + theme + fixture-state gates and are deleted), and the captured cell tables are gone. But the removal is incomplete and the replacement pages carry runtime defects that the stream's own gate does not catch:

- the live jackin accounts page has a wrong pointer target, a list that never scrolls, an out-of-range range indicator and a key-dead committed-filter state;
- the new accounts hint bar advertises five keys that do not do what they say;
- the live showcase terminal and task runner pages overlap their own text ("ru0.7 s", "Step r0 of 7 · running", "status: ready────╯") and advertise keys their widgets do not implement;
- the chrome page is still a hand-painted mock, and the stream moved part of that mock into shared app code with hard-coded theme colours;
- the theme oracle commit reverts two corpus-measured restorations and rewrites the tests that pinned them, leaving five behavioural tests newly red.

No fix was applied by this reviewer; nothing was pushed to `parity/termrock-visual-baseline` or `main`.
