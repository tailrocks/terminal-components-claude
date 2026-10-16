**Colour review: showcase and tablepro (branch `design/interaction-polish` @ a3e367e8f)**

The PO's concern holds, and tablepro is the worst case. I measured the frames the current code renders (`VISUAL_FULL=1` nextest run, 120×40, truecolor and 256), not the committed `snapshots/`. About 110 showcase and tablepro rows differ from what's committed, so those PNGs are out of date.

- **tablepro (55 frames):** median chromatic ink is 0.2% of glyph cells, and 39 of 55 frames are under 1%. A typical workbench frame has two coloured cells: the green ▪ brand and the focus `▎`.
- **showcase (115 frames):** median is 1.8%. Each frame has about 10 distinct foreground colours, nearly all of them grey steps.
- **Root causes:**
  - The design doctrine in `src/bin/showcase/pages/overview.rs:47-48` says: "Everything else is achromatic."
  - `Theme::syntax` deliberately uses greys only (`theme.rs:1486`).
  - The `info` token (#8787ff) is defined and used by no widget or app page (only the Overview swatch shows it).
  - Blue appears only on menu cursor rows and text selection.
  - tablepro passes no meaning into colour: environment, connection status and statement risk all render grey.

## 1. Proposed semantic colour roles

Contrast is measured on canvas #000 / card #18181b / field #1e1e22 / popover #3f3f46.

| Token | Truecolor | 256 | 16 | Mono cue | Contrast | Use |
|---|---|---|---|---|---|---|
| `info` (re-hue; replaces PURPLE) | #79b8ff | 111 | LightBlue | `i` glyph | 10.1 / 8.5 / 8.0 / 5.0 | info, links, running, development env |
| `link` (new style) | `info` + underline | 111 | LightBlue | underline | as `info` | links, clickable references |
| `syntax_keyword` | #79b8ff, bold | 111 | LightBlue | bold | 10.1 / 8.5 / 8.0 | SQL and Rust keywords |
| `syntax_string` | #8bd5ca | 116 | Cyan | none | 12.5 / 10.5 / 9.9 | string literals, JSON values |
| `syntax_literal` | #c6a0f6 | 183 | LightMagenta | none | 9.8 / 8.2 / 7.7 | numbers, true/false, NULL (stays italic) |
| syntax: comment / operator / identifier | keep faint italic / muted / primary | — | — | italic | — | unchanged |
| `env_production` | `danger_label` #ff6b6b, bold | 203 | LightRed | ◆ + bold | 7.6 / 6.4 / 6.0 | production badge, tree glyph, write gates |
| `env_staging` | `warning` #f59e09 | 214 | Yellow | ◇ | 9.8 / 8.3 | staging |
| `env_development` | `info` | 111 | LightBlue | · | as `info` | development |
| `env_local` | `text_muted` | 245 | Gray | none | ≥4.8 | local |
| `error` as text (split from fill) | #f05454 | 203 | LightRed | `!` / ✗ | 6.1 / 5.15 / 4.83 | Today #e44545 is only 4.42 on card and 4.14 on field, which fails 4.5. Keep #e44545 for fills and glyphs. |
| `text_muted` (fix) | #8a8a8a | 245 (already) | Gray | none | 6.1 / 5.13 / 4.81 | Today #808080 is 4.49 on card and 4.21 on field, which fails, and it doesn't match the 256 value. |

A new `Theme::status(StatusKind) -> (glyph, Style)`, so no app hand-paints status rows:

| Status | Glyph | Token |
|---|---|---|
| Ok | ✓ | `success` |
| Fail | ✗ | `error` text |
| Warn | ▲ | `warning` |
| Pending / dirty | • | `warning` |
| Running | spinner | `accent` (keeps the statusbar rule); label in `info` |
| Queued | ○ | `text_muted` |
| Info | `i` | `info` |
| Skipped | – | `text_faint` |

Because glyphs are mandatory, colour is never the only cue.

- **Keys in hints:** stay primary bold, no new token. Adding colour there would be decoration.
- **Accent:** stays reserved for focus and the primary action.
- **Doctrine:** change it to "one accent hue for focus/primary; status hues only where there is meaning; syntax hues only in code."

## 2. Ranked findings

| # | Sev | App / capture | Element | Today | Proposed | Lib or app (file:line) |
|---|---|---|---|---|---|---|
| 1 | High | tp workbench: `query/results`, `table/*`, `ack/*` | `◆ production` header badge | bold #fff | `env_production` | app `tablepro/app.rs:2766-2773`; lib: add `Tone::Danger`/`Info` to `theme.rs:1513` |
| 2 | High | tp `query/*`, `history_tab`, `ack/gate`; sc `pages/codeeditor`, `flows/editor/*` | syntax colouring | keyword bold white; string and number #b3b3b3 | `syntax_*` tokens | lib `theme.rs:1486-1499` |
| 3 | High | library-wide | `info` token | #8787ff, never used | re-hue to #79b8ff and wire into `Tone`, links and running state | lib `theme.rs:383,539` |
| 4 | High | tp all frames (top bar) | Faint segment `truecolor · 120×40` | renders #fff, the brightest item on the bar | `legible()` should step up the text ladder (faint→muted→secondary), not jump to primary. #737373 is 4.43:1 on #000 so it is promoted to white. | lib `theme.rs:1280-1295` |
| 5 | High | tp `ack/gate`, `ack/armed` | "Execute write query?" on production | all neutral; armed Execute is a green accent fill (reads as "go") | ▲ `warning` before the title; Target env in `env_production`; Execute uses the danger kind when env is production | app `app.rs:1138-1212` |
| 6 | Med-High | tp `connections/default`, `production_detail` | env glyphs ◆◇· in the tree; "Environment" value | grey; `local` faint (3.7 on card) | `env_*` tokens | app `connections.rs:306-312, 1329` |
| 7 | Med | tp all | status message "Connected to Production" | #b3b3b3, untyped | `set_status(msg, Tone)` and Ok/Fail glyphs: ✓ `success` for connect/save/execute, ✗ `error` on failure | app `app.rs:284, 238` |
| 8 | Med | sc `flows/taskrunner/running`, `pages/taskrunner` | task states | ✓ in secondary grey; "queued" faint (3.74 on card); running label primary | `Theme::status`; "· running" in `info` | app `showcase/pages/taskrunner.rs:462-500`; lib new helper |
| 9 | Med | sc `pages/overview` | principles and token swatches | "One hue… achromatic"; no swatches for success, highlight, selection, danger; labels wrong (text.faint says "white 30%" but is #737373 ≈45%; border.subtle says "15%" but is #3a3a3a ≈23%) | update copy, swatches and labels | app `overview.rs:20-50` |
| 10 | Med | sc `flows/forms/invalid`, `audit/inputs`, `audit/textareas` | error text on card/field | #e44545 (4.42 / 4.14) | `error` text #f05454 | lib `theme.rs:369, 544` |
| 11 | Med | all card panels (Engine/Host labels, placeholders) | `text_muted` | #808080 (4.49 card / 4.21 field) | #8a8a8a | lib `theme.rs:362` |
| 12 | Med | tp `table/*`, `query/results`, explorer | active tab underline, result tab, explorer's current table | all grey in current code (the snapshots still show green) | accent underline while the page is active (the change already in flight) | lib `widgets/tabs.rs` |
| 13 | Med | tp `table/data`, `structure`; sc `pages/datagrid` | typed literals: true/false, NULL, JSON | plain; NULL faint italic | `syntax_literal` for bool/NULL; JSON values in `syntax_string`, only in typed columns | lib `widgets/grid.rs` cell tone |
| 14 | Low-Med | sc `pages/settings`, `flows/forms/*` | required `*` | accent green (the accent means focus) | `text_muted`; `error` text only once invalid | lib `widgets/input.rs:791` |
| 15 | Low-Med | tp `query/results` editor toolbar | Run / Run all / Explain | all subtle grey | Run gets accent ink (subtle-primary) or a ▶ accent glyph | app `tablepro/tabs.rs` / `workbench.rs` |
| 16 | Low-Med | tp `connections/form_*` | Test connection result, SSL on/off | text only | `Theme::status` Ok/Fail; "on" in `success` | app `connections.rs:1340` |
| 17 | Low | tp `query/explain*` | hot plan node | `93 ▲` amber is good; node name "Seq Scan" plain | node name in `warning` when it is the hot node | app `tabs.rs` |
| 18 | Low | tp `overlays/history_tab` | slow durations (412 ms) | plain | `warning` above a threshold (failures already get a red `!`) | app |
| 19 | Low | sc `flows/help/overlay`, tp `overlays/help_overlay` | key column | same weight as the description | keys primary bold, description secondary (like the hint bar) | lib `widgets/help.rs` |
| 20 | Low | tp header | `safe` badge | bold #fff | `success` plus a glyph; Silent-on-production amber is already right | app `app.rs:2793-2797` |
| 21 | Low | tp vs sc | brand ▪ | tp uses `Tone::Success` #3fb950, sc uses accent #48e054 | accent in both | app `app.rs:2749` |
| 22 | Low | 256 level | warning, info and syntax not pinned | nearest-match (info lands on 105) | pin 111 / 116 / 183 / 214 / 203 | lib `theme.rs:666-716` |
| 23 | Low | 16 level | `highlight` and `selection_bg` | both Blue, so menu cursor and text selection collide | selection uses reverse video (Mono-style) | lib `theme.rs:719-` |
| 24 | Low (overuse) | sc `pages/progress`, `flows/progress/done` | meters | 19–24% chromatic; 61 red background cells | red fill only on the failed bar; others get a coloured label and a neutral track | app `showcase/pages/progress.rs` |
| 25 | Info | sc diff pages | +/− lines | `success` / `error` | already correct; use as the reference pattern | — |

## 3. Top 10 quick wins

1. Re-hue `info` to #79b8ff (256: 111, 16: LightBlue) and add `Tone::Info` and `Tone::Danger`. That is two enum arms and one palette constant.
2. Fix `legible()` to step up the text ladder. One function; it fixes the white debug badge.
3. Add colour to `Theme::syntax`: keyword blue bold, string teal, literal lavender. One function, and both apps' editors pick it up.
4. Colour the production header badge `env_production` (`app.rs:2768`), and point `env_tone` at the `env_*` tokens (`connections.rs:306`).
5. On a production write gate: ▲ warning title glyph and Execute as the danger kind (`app.rs:1138, 1210-1212`).
6. Raise `text_muted` to #8a8a8a and split error text to #f05454. Both clear AA on card and field.
7. Taskrunner: ✓ in `success`, "queued" in `text_muted`, "running" in `info` (`taskrunner.rs:464, 494`).
8. Make `set_status` take a tone, and show "Connected / Saved / Executed" with ✓ `success`.
9. Make the required `*` neutral (`input.rs:791`).
10. Fix the Overview swatches, labels and principle copy so the doc teaches purposeful colour.

Measurement script: `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/ansistat.py`. Per-frame stats: `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/stats_act.txt`. Current-code captures: `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/colour-a/target/tuiscotti/actual/`. Nothing in the repo was edited, and the `visual-baseline` tag was not touched.