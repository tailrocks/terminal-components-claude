# Colour as meaning: resolved tokens and follow-ups

This file records what the library shipped from
[`colour-showcase-tablepro.md`](colour-showcase-tablepro.md) and
[`colour-holla-jackin.md`](colour-holla-jackin.md), where the two reviews
disagreed and what was decided, and the per-app wiring that is left to
the app follow-ups.

Doctrine (also on the showcase Overview page): one accent hue for focus
and primary action; status hues only where they carry meaning; syntax
hues only in code. Every status keeps a glyph, so colour is never the
only cue.

## Resolved tokens

Contrast is WCAG 2.2 against canvas `#000` / surface `#111` / card
`#18181b` / field `#1e1e22` / raised `#27272a` / popover `#3f3f46`.
Indexed and named colours are measured with the xterm defaults (the same
`contrast_ratio` the theme uses). Every value is a `Theme` field, and
each level can be pinned through `LevelOverrides`.

| Token | Truecolor | 256 | 16 | Mono cue | Contrast (truecolor) |
|---|---|---|---|---|---|
| `info` | `#79b8ff` | 111 | LightBlue | `i` glyph (`Theme::status`) | 10.1 / 9.1 / 8.5 / 8.0 / 7.2 / 5.0 |
| `link` | `info`, underlined on hover or focus | 111 | LightBlue | underline (always in mono) | as `info` |
| `error` (text) | `#f05d5d` | 203 | LightRed | `✗` bold, `!` | 6.4 / 5.8 / 5.4 / 5.1 / 4.6 / 3.2 |
| `error_strong` (popover) | `#ff8a8a` | 210 | LightRed | as `error` | 4.6 on popover (256: 4.9) |
| `success` | `#3fb950` | 71 | Green | `✓` | 8.3 / 7.4 / 7.0 / 6.5 / 5.9 / 4.1 |
| `success_strong` (popover) | `#56d364` | 77 | Green | `✓` | 5.4 on popover (256: 6.2) |
| `warning` | `#f59e09` | 214 (pinned) | Yellow | `▲`, `•` | 9.8 / 8.8 / 8.3 / 7.7 / 6.9 / 4.9 |
| `text_muted` | `#8a8a8a` | 245 | Gray | none | 6.1 / 5.5 / 5.1 / 4.8 / 4.3 / 3.0 |
| `syntax_keyword` | `#79b8ff` bold | 111 | LightBlue | bold | as `info` |
| `syntax_string` | `#8bd5ca` | 116 | Cyan | none | 12.5 / 11.2 / 10.5 / 9.9 / 8.8 / 6.2 |
| `syntax_literal` | `#c6a0f6` italic | 183 | LightMagenta | italic | 9.8 / 8.8 / 8.2 / 7.7 / 6.9 / 4.9 |
| `danger_label` (`Tone::Danger`, production) | `#ff6b6b` bold | 203 | LightRed | `◆` + bold | 7.6 / 6.8 / 6.4 / 6.0 / 5.4 / 3.8 |

Where a token is under 4.5:1 (raised and popover for muted, popover for
error, success and danger), `Theme::tone_on` and `Theme::legible` pick
the readable step: the strong error or success step, or one step up the
text ladder (muted to secondary). The tests
`semantic_hues_read_on_every_plane_they_can_land_on`,
`semantic_hues_read_at_256_colours`,
`semantic_hues_stay_apart_at_reduced_levels`,
`statuses_carry_a_glyph_at_every_level_and_glyph_set` and
`syntax_classes_differ_without_colour` in `src/theme.rs` hold these.

### Library API

- `Tone::Info`, `Tone::Active` (the accent as a foreground for live or
  running state, never "done"), `Tone::Danger`.
- `Theme::status(StatusKind) -> (glyph, Style)`, `Theme::status_at` (the
  running spinner frame), `Theme::status_label`. Ok `✓` success, Fail `✗`
  error bold, Warn `▲` warning, Pending `•` warning, Running spinner in
  the accent with the label in info, Queued `○` muted, Info `i` info,
  Skipped `–` faint. Glyphs come from `Theme::glyphs` (`Glyphs::ASCII`:
  `+ X ! * | o i -`).
- `Theme::env(EnvKind) -> (glyph, Style)`: production `◆` danger label
  bold, staging `◇` warning, development `·` info, local muted without a
  glyph (ASCII `# ~ .`).
- `Theme::link_style(hovered_or_focused)`.
- `SyntaxTone::Literal` for booleans and NULL; numbers share its style.
- `legible()` steps up the text ladder (faint, muted, secondary) and
  never jumps to primary unless no step reads (a coloured fill).
- 16 colours: `selection_bg` is no longer the highlight's Blue; text
  selection is reversed video. Ink on a highlight fill stays primary
  (`Theme::row_ink`).
- Meters: the healthy run is `info`, medium `warning` with `▲`, high
  `error` with `!` (never green for a quota). `MeterTone::Level { warn,
  crit }` takes owner thresholds; the old explicit level is
  `MeterTone::Fixed(MeterLevel)`.
- Tabs: `TabItem::done` draws `✓` in success after busy, error and dirty.
- Data grid: NULL and booleans in the literal hue (NULL stays italic),
  JSON values in the string hue.
- Text input: the required `*` is muted, and takes the error tone only
  once the field is invalid.
- Help overlay: keys primary bold, descriptions secondary.
- Picker group labels: muted, stepped up through `legible` on the
  popover (faint was 2.2:1 there).
- Added for the app follow-ups: `HintLayer::status_parts` (a hint-bar
  status whose parts keep their own tones; the message tone still picks
  the `!` / `▲` mark) and `viewport::render_spans` (one line of `Span`s
  outside a viewport, each tone legible on its plane, cut with `…`).

### Decisions where the reviews differed

- `info`: `#79b8ff` / 111 (showcase review) over `#6cb6ff` / 75
  (holla review). 111 is the closer 256 match and the same hue serves
  keywords, so code and information read as one family.
- `error`: `#f05d5d` (holla review) over `#f05454`; both clear 4.5:1 on
  card and field; `#f05d5d` also clears raised (4.56). 256 is 203
  everywhere instead of 167 plus 203 on raised planes: 203 reads on
  every plane but the popover, which takes `error_strong` (210).
- `text_muted`: `#8a8a8a`, matching its 256 pin (245).
- `active`: a `Tone` (accent as foreground), not a new colour token.

### Formerly deferred (library), now done

- **Done: stale meters keep their level.** A stale meter used to draw a
  faint run, so a stale 76% hid its level (holla-jackin finding "Session
  meter at 76%"). Now `MeterTone::Stale` derives the level from the
  shared thresholds, and `Meter::stale(true)` marks any levelled tone
  (`Normal`, `Level`, `Fixed`, `Warning`, `Exhausted`) as last-good
  data. The default `StaleStyle::Dashed` keeps the level hue on the run
  (info 10.1, warning 9.8, error 6.4:1 on canvas; graphics need 3:1)
  and the level glyph (`▲`, `!`). It draws the run dashed
  (`Glyphs::bar_stale`, `╍`, ASCII `~`) and the value muted and italic
  (muted `#8a8a8a`: 6.1 / 5.5 / 5.1 / 4.8:1 on canvas, surface, card,
  field; `legible` steps it up where it would fall under 4.5:1, such as
  the lifted rest of a block meter, where it was 4.4:1). The dashes and
  the italic are the stale cue without colour, so level (glyph, run
  length) and staleness (dashes) both read under `NO_COLOR`.
  `StaleStyle::Faint` keeps the old faint look as an override for
  owners that would rather not show a level they cannot vouch for.
  Tests: `stale_meter_keeps_its_level_with_a_cue_that_survives_mono`,
  `stale_composes_with_tones_and_the_faint_style_is_an_override` in
  `src/widgets/progress.rs`. The jackin owners can now pass their quota
  tone with `.stale(true)` instead of `MeterTone::Stale`, so a stale
  quota warning stays a warning (usage, accounts and capsule meters).
- **Done: a running progress bar is `Tone::Active`.** A determinate
  `ProgressStatus::Active` run used the secondary grey; it now takes the
  accent as live state, the hue of the indeterminate sweep and of the
  spinner, so a task keeps its colour when its progress becomes known.
  Done stays the success hue with `✓`: the hues differ at every colour
  level (truecolor `#48e054` against `#3fb950`, 256 78 against 71, 16
  LightGreen against Green) and the glyph differs at all of them,
  including mono. The run is 12.1:1 on canvas (256: 11.6:1). Test:
  `active_bar_runs_in_the_active_tone_apart_from_done`.

## Colour follow-ups per app

Line numbers are for the code at the time of the library change. Each
item uses the library API above; none needs a new token.

### tablepro

All done. Each row is held by a test in
`src/bin/tablepro/app_tests_colour.rs` that drives the real app and
reads the rendered cells (each failed on the code before the change).

| Element | Where | Change | Status |
|---|---|---|---|
| `◆ production` header badge (staging, development, local segments next to it) | `src/bin/tablepro/app.rs:2799-2803` | `Theme::env(EnvKind::…)` glyph and style instead of Normal / Secondary / Muted / Faint | Done: glyph `Glyphs::env`, tone `EnvKind::tone`, bold where `Theme::env` is bold (`strip_brand_is_the_accent_and_the_environment_badge_comes_from_theme_env`, `strip_environment_badges_take_each_environment_tone`) |
| `env_tone` | `src/bin/tablepro/connections.rs:306` | map to `Tone::Danger` / `Warning` / `Info` / `Muted` (or use `Theme::env`) | Done: `Environment::kind().tone()` |
| Tree environment glyphs `◆ ◇ ·` | `src/bin/tablepro/connections.rs:352-357` | glyphs from `Theme::glyphs.env_*`, ink from `Theme::env` | Done: `TreeNode::glyph_tone`, glyphs synced from the theme's set at render (`connection_tree_and_detail_show_environments_in_their_tones`) |
| Detail "Environment" value | `src/bin/tablepro/connections.rs:1329` | follows `env_tone` | Done (same test) |
| Write gate title "Execute write query?" | `src/bin/tablepro/app.rs:1164-1170` | `▲` warning glyph before the title on production | Done: `Dialog::title_status(StatusKind::Warn)` (`write_gate_on_production_warns_names_the_target_and_arms_a_danger_button`) |
| Write gate "Target" line | `src/bin/tablepro/app.rs:1178-1189` | `env_production` tone when the target is production | Done (same test) |
| Write gate Execute button | `src/bin/tablepro/app.rs:1240-1244` | `Button::danger` whenever the env is production (not only for destructive statements) | Done (same test; `write_gate_off_production_keeps_a_plain_title_and_primary_execute` holds the other environments) |
| `set_status` takes no tone | `src/bin/tablepro/app.rs:290` | take a `StatusKind`: Ok for "Connected to" (`app.rs:244-245`), "Saved" (`app.rs:1425-1426`, `1970`, `connections.rs:944`); Fail for failures; Info or Warn for cancelled (`app.rs:396, 848, 1086, 2164`) and the Safe Mode block (`app.rs:1155`) | Done: `set_status_as` / `Cx::status_as`, drawn by `keyhint::render_status`. Cancelled is Info (the user asked for it); the Safe Mode blocks (run and save), "save or discard before disconnecting" and "Fix the highlighted fields" are Warn. No footer status reports a failure today (failures show in the result pane and the connection card), so nothing takes Fail yet (`footer_status_carries_its_kind`) |
| Statement results "ok" / "failed" | `src/bin/tablepro/tabs.rs:1170-1189` | `Theme::status` Ok / Fail | Done on the result pane's status line, where they are read: `✓ 3 rows · …` (glyph success, text muted), `✗ failed · …` (glyph bold error, text `status_label(Fail)`) (`statement_results_lead_with_their_status`). `last_status` also feeds the tab picker's detail text, which is plain text |
| Toolbar Run | `src/bin/tablepro/tabs.rs:1613` | Run as the primary action (accent ink or `▶` accent glyph); Run all and Explain stay subtle (`1617`, `1621`) | Done: `Button::primary` (`toolbar_run_is_the_primary_action`) |
| Test connection result | `src/bin/tablepro/connections.rs:1642-1646` | `Theme::status(Ok/Fail)` (today `✓` in secondary) | Done: glyph from `Theme::status`, message in `status_label` (`test_connection_result_takes_the_status_glyph_and_tone`) |
| SSL on/off | `src/bin/tablepro/connections.rs:1301, 1340` | "on" in success with `✓` | Done: SSL and SSH are separate rows so only SSL takes the tone (`ssl_on_is_a_passed_check_in_the_detail_card`) |
| Explain hot node name | `src/bin/tablepro/tabs.rs:1942-1970` | node name in `warning` when it is the hot node (the `▲` share is already right) | Done: `TreeNode::label_tone(Tone::Warning)`; the `▲` now comes from `Theme::glyphs` (`explain_names_the_hot_node_in_warning`) |
| History slow durations | `src/bin/tablepro/tabs.rs:2495`, detail `2672` | `warning` above a threshold | Done: `HistoryTab::slow_ms` (default 1000) and `ListItem::meta_tone`; the detail's Duration follows (`history_marks_slow_durations_in_warning`) |
| Safe badge | `src/bin/tablepro/app.rs:2823-2828` | `safe` in success with a glyph; Silent on production stays warning | Done: `✓ safe` / `✓ safe+` from `Theme::status(Ok)`; Silent on production also gains the `▲` of `Theme::status(Warn)`, so its warning keeps a glyph like every status (`strip_safe_badge_is_a_passed_check_and_silent_production_a_warning`) |
| Brand `▪` | `src/bin/tablepro/app.rs:2780` | `Tone::Active` (the accent, as in showcase), not `Tone::Success` | Done (strip test above) |
| SQL `TRUE`/`FALSE`/`NULL` | `src/bin/tablepro/sql.rs:35, 100-101, 193-196`; mapping `src/bin/tablepro/tabs.rs:338-344` | a `TokKind::Literal` mapped to `SyntaxTone::Literal` (numbers already take the literal hue) | Done: the parser still reads them as words (`sql_booleans_and_null_take_the_literal_hue`) |

Library additions for these rows: `StatusKind::tone`, `EnvKind::tone`,
`Glyphs::env`, `keyhint::render_status`, `Dialog::title_status`,
`TreeNode::glyph_tone` / `label_tone`, `ListItem::meta_tone`, each with
a unit test beside it. Defaults draw exactly as before.

Visual: the tablepro captures are all still pending tuiscotti
re-approval (legacy tuisnap approvals, skipped unless `VISUAL_FULL=1`),
so no tablepro capture gates these changes and none was re-approved.

### showcase

All three are done; the regression tests are in
`src/bin/showcase/app_tests_colour.rs`.

| Element | Where | Change |
|---|---|---|
| Task runner states | `src/bin/showcase/pages/taskrunner.rs:464, 468, 474, 494` | **Done.** Every row leads with its `Theme::status_at` glyph: queued `○` muted, running the accent spinner with the task name in info (`status_label`), done `✓` success, failed `✗` error bold, cancelled `–` (Skipped). The state word takes `tone_on(kind.tone())`, so "cancelled" (faint, 3.7:1 on the card) steps up to muted. Test `taskrunner_states_take_the_status_roles`. |
| Progress page meters and failed bar | `src/bin/showcase/pages/progress.rs:52, 108-109, 142-151` | **Done.** The block visual (whose fill is the level hue as a background) is shown once, on a low meter, so red appears only as a run or a glyph (the failed bar, high, exhausted), never as a filled block. Meters take the library levels; the right column shows the options: owner thresholds (`MeterTone::Level { warn: 80, crit: 95 }` at 72% and 88%), `MeterTone::Fixed`, a stale domain warning (`Meter::stale`) and `StaleStyle::Faint`. The caption now says low is blue. Test `progress_page_shows_meter_levels_and_staleness_without_red_fills`. |
| Editor sample `true` / `false` | `src/bin/showcase/pages/editor.rs:57, 119, 130-131` | **Done.** Booleans are `SyntaxTone::Literal`; the sample gains `.gzip(true)` so a literal is on screen. Test `editor_booleans_are_literals`. |

### holla

All done; the regression tests are in `src/bin/holla/app_tests_colour.rs`.

| Element | Where | Change | Status |
|---|---|---|---|
| Batch toast "N ok · N failed · N cancelled" | `src/bin/holla/app.rs:548, 553` | per-segment tones: ok success, failed error, cancelled muted; an all-ok batch in success | Done: `App::set_status_parts` through `HintLayer::status_parts`; a zero count stays muted (`a_batch_summary_colours_each_count_by_what_it_says`) |
| Output line tones, failure output | `src/bin/holla/screens/activity.rs:90-98` (errors from `src/bin/holla/domain/activity.rs:173, 433`) | `LineTone::Success` to `Tone::Success`; stderr body primary with a red `!` gutter, red only on the status chip | Done: error lines get a bold error `! ` gutter and a primary body (`failure_output_is_primary_text_behind_a_red_gutter`) |
| "succeeded" meta | `src/bin/holla/screens/plan.rs:196` | success | Done: the outcome line is spans (`viewport::render_spans`), failed in error (`a_finished_plan_says_succeeded_in_success_and_failed_in_error`) |
| Required vs optional step checkboxes | `src/bin/holla/screens/plan.rs:303-309` | required: faint with a lock cue; toggleable: `t.chosen(focused)` | Done: toggleable already took `t.chosen(focused)`; the lock cue is the row note `required` (or `required · <privilege>`) beside the faint marker, words rather than a new glyph (`a_required_plan_step_says_why_it_cannot_be_toggled`) |
| Cleanup history outcome "trashed" | `src/bin/holla/screens/snapshot.rs:784, 1137-1145` (label `src/bin/holla/domain/cleanup.rs:849`) | `Theme::status(Ok)` for trashed | Done: history rows and report items draw `Theme::status(Ok)` before `trashed`, the row in success (`a_trashed_cleanup_record_carries_the_done_status`) |
| Checked boxes always accent | `src/bin/holla/screens/cleanup.rs:646, 718` | `t.chosen(focused)` | Done: the category mark too (`a_checked_cleanup_category_is_primary_until_its_list_has_focus`) |
| Status bar `↓ ↑ •` counts | `src/bin/holla/app.rs:3519-3524` (text `src/bin/holla/domain/stack.rs:178-184`) | `↓↑` info, dirty `•` warning, conflicts error | Done: `GitState::state_parts`, one status item per part; conflicts read `N conflicts` (`the_git_state_in_the_status_bar_is_toned_per_part`) |
| Finder section headings | `src/bin/holla/screens/mod.rs:489` (`fn heading`, 474) | `t.muted()` (faint is 4.4:1 on canvas) | Done: `Theme::tone_on(Muted, bg)` (`section_headings_are_muted_not_faint`) |
| Disk filesystem meter | `src/bin/holla/screens/disk.rs:1670` | `MeterTone::Level { warn: 80, crit: 95 }` | Done: `disk::FS_METER`, also on the status-bar root filesystem meter (`a_filesystem_meter_warns_at_its_own_thresholds`) |
| Activity tabs | `src/bin/holla/app.rs:3389-3401` | `Succeeded` sets `TabItem::done` instead of the `✓` suffix | Done: also finished plan tabs (`a_succeeded_activity_tab_is_done`) |

### jackin

| Element | Where | Change |
|---|---|---|
| Usage provider rows (whole row in one tone) | `src/bin/jackin_preview/screens/usage.rs:395-419` | a table with per-cell tone: name primary, window as a `Meter`, only the status cell in error / warning with its glyph, "current" `Theme::status(Ok)` |
| Usage Health line | `src/bin/jackin_preview/screens/usage.rs:314-319` | per-segment tones: degraded error bold, warnings and stale warning, refreshing info, counts primary |
| Accounts Health line | `src/bin/jackin_preview/screens/accounts.rs:1457-1459` | as the usage Health line |
| Accounts warnings rows | `src/bin/jackin_preview/screens/accounts.rs:1520-1531` | glyph and status phrase coloured; account primary, reset time muted |
| Provider group labels | `src/bin/jackin_preview/screens/accounts.rs:1386-1387, 1409-1410` (and the usage tree) | `text_muted` instead of `text_faint` |
| Running instance `◉` and "running" | `src/bin/jackin_preview/screens/manager.rs:220` (also `1349, 1636, 2839`), meta `1177, 1194` | `Tone::Active` on the glyph and the word |
| "N running" counts | `src/bin/jackin_preview/app.rs:1046, 2788` | count secondary with an `Active` dot |
| Run-id chip in warning | `src/bin/jackin_preview/screens/cockpit.rs:971-976` | `Theme::link_style` / info |
| Session meter while stale | `src/bin/jackin_preview/screens/capsule.rs:1826, 1843-1845` | library done (stale keeps the level, dashed run); pass the quota tone with `Meter::stale(true)` to keep a domain warning |
| Capsule tab suffixes | `src/bin/jackin_preview/screens/capsule.rs:1512-1528` | finished agents use `TabItem::done` (today `○` for Done) |
