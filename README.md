# Junie TUI — a Ratatui design system and its first real application

Three binaries share one library:

- **`showcase`** — the approved design-system laboratory: every component in
  every interaction state, two composed screens, and the visual baseline that
  keeps the language from drifting.
- **`tablepro`** — a database workbench in the spirit of
  [TablePlus](https://tableplus.com): connections, schema explorer, SQL editor
  with completion, result tabs, an editable data grid with a pending-change
  queue, filters, table structure, query history, a quick switcher, EXPLAIN
  plans, and TablePlus' Safe Mode levels. It runs against a deterministic
  in-memory demo database (no drivers), so every flow is reproducible.
- **`jackin-preview`** — an interactive, fully simulated redesign of the
  Jackin agent-container CLI (built from a read-only reading of its source): the
  Construct intro and outro rituals, the host Workspace Manager, the Create
  Workspace prelude, the Workspace Editor, Global Settings, the Account &
  Usage Center, the launch cockpit and a Capsule terminal multiplexer. Every
  scenario is a fixture world with a virtual clock, so any frame can be
  reproduced. It never touches the real Jackin CLI, containers, 1Password or
  provider APIs.

The application is the specification: *if the
[Junie](https://junie.jetbrains.com) website had been designed for a terminal
instead of a browser, what would its components look and feel like — and does
the language hold up when a real tool is built from it?*

## Run

```sh
cargo run --release                                  # the showcase (default binary)
cargo run --release -- --page datagrid               # start on a page (overview, buttons, … codeeditor, datagrid, chipsselects, pickers)
cargo run --release -- --color 256                   # force a colour level: truecolor|256|16|none

cargo run --release --bin tablepro                   # the workbench, starting on the connections screen
cargo run --release --bin tablepro -- --connect Production   # connect straight away

cargo run --release --bin jackin-preview             # Jackin redesign, first-use scenario (intro → manager)
cargo run --release --bin jackin-preview -- --scenario accounts-mixed   # first-use | returning | accounts-mixed |
                                                     # launch-running | launch-failure | capsule-multi | outro-last | hard-cases
cargo run --release --bin jackin-preview -- --scenario launch-running --motion reduced   # full | reduced | paused
cargo run --release --bin jackin-preview -- --scenario first-use --motion paused --frame 282    # freeze one frame
JACKIN_NO_MOTION=1 cargo run --release --bin jackin-preview   # same as --motion reduced
```

The preview's scenarios are deterministic: the same `--scenario`, `--motion`,
`--frame` and terminal size always render the same picture, which is what
`src/bin/jackin_preview/app_tests.rs` and the `j_*` captures rely on. No
secret ever reaches a frame — 1Password references resolve only inside the
simulated credential service, plain-text keys live in transient edit state
and render masked with a synthetic four-character tail.

Requirements: Rust 1.88+, a terminal with mouse support. Truecolor is the
primary target (`COLORTERM=truecolor`); 256/16-colour terminals get a mapped
palette, `NO_COLOR` gives a monochrome fallback. Minimum size is 72×20; below
that a reduced state is shown until the terminal grows. The workbench shows
the explorer beside the tabs from 100 columns up; below that it becomes a
drawer that covers the tab body while it has focus (`0` opens it, opening an
object or pressing Tab puts it away).

Verify:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Keyboard and mouse

| Key | Action |
|---|---|
| `Tab` / `Shift+Tab` | Move keyboard focus through every control, in reading order, wrapping |
| `↑ ↓ ← →` (`j k h l`) | Move inside the focused control (rows, cells, tree, tabs, radios) |
| `Enter` / `Space` | Activate a button, choose a row, toggle a checkbox, start editing a field |
| `Esc` | Cancel editing (inputs, cells) or finish (text areas); at top level, return to navigation |
| `[` `]` | Previous / next page · `0` jumps to the navigation |
| `s` | Sort the current table column (asc → desc → none) |
| `*` `-` | Expand / collapse every tree node |
| `f` | Follow the tail of a log panel |
| `Ctrl+S` | Submit a form |
| `i` | Toggle the state inspector · `?` help · `q` quit (`Ctrl+C` always quits) |

While editing: `Ctrl+A/E` line start/end, `Ctrl+← →` or `Alt+B/F` by word,
`Shift+arrows` select, `Ctrl+U/K` delete to start/end, `Ctrl+W` delete word,
`Ctrl+L` select all, `Tab` commits and moves on (next field, or next editable
cell in a table). Bracketed paste inserts into whichever control is editing.

Mouse: hover previews any row, button, field or tab; click focuses and
activates; a second click on a focused field or cell starts editing; the wheel
scrolls the container under the pointer without moving focus; the scrollbar
thumb can be clicked and dragged; clicking outside a dialog cancels it.

## TablePro workbench

### Information architecture

```
identity strip   ▪ TablePro · connection · ◆ production · database › schema · safe-mode token · running · pending · ? help
tab strip        ≡ Query 1 ×   T orders ×   H History ×                                              +
explorer         framed pane: filter, database › schema › Tables / Views / Functions / Sequences (lazy)
tab body         framed pane: table tab (Data / Structure), query tab (editor over result tabs), history tab
footer           key hints for the focused control · EDIT badge while editing · status
```

Modals sit above everything and dim the page: the safety dialog, the filter
editor, and the pickers (Open Quickly, tab list, Safe Mode level).

### Key map

| Key | Where | Action |
|---|---|---|
| `Tab` / `Shift+Tab` | everywhere | next / previous control (strip → explorer → tab body, in render order) |
| `0` · `Ctrl+B` | workbench | focus the explorer · show / hide it |
| `Esc` | workbench | cancel a running query → leave maximised → tab strip → explorer |
| `Ctrl+T` · `Ctrl+W` · `[` `]` · `Ctrl+G` | workbench | new query · close tab · previous / next tab · tab list |
| `Ctrl+O` / `Ctrl+P` | workbench | Open Quickly (tables, views, schemas, queries; `Tab` cycles the scope; `Alt+Enter` opens in a new tab) |
| `Ctrl+Y` · `Ctrl+L` · `Ctrl+D` · `z` · `Ctrl+↑` `Ctrl+↓` | workbench | history · Safe Mode level · Data ⇄ Structure · maximise the tab · resize the editor / results split |
| `Ctrl+R` / `F5` · `Alt+R` · `Ctrl+X` · `Alt+X` | query tab | run the statement under the cursor · run all · EXPLAIN · EXPLAIN ANALYZE |
| `Ctrl+C` | query tab | cancel the running query (quits when idle) |
| `i` `a` · `{` `}` · `/` `n` `N` · `Ctrl+Space` | editor | edit · previous / next statement · find · complete |
| `p` · `x` | result tabs | pin (pinned tabs survive the next run) · close |
| `s` · `f` · `Ctrl+F` · `F` | grid | sort column · filter on this cell · filter editor · clear filters |
| `Enter` · `Space` · `+` `-` · `u` · `y` `Y` | grid | edit cell · select row · insert / delete row · undo · copy cell / row |
| `p` · `Ctrl+S` | grid with pending changes | preview the SQL · save (Preview / Discard / Save are also buttons in the pending bar) |
| `Enter` · `r` · `y` · `/` · `c` `s` | history | open in a new tab · rerun · copy · search (terms are ANDed) · scope / status filter |

Mouse: hover lifts rows, tabs and buttons; click focuses and activates; a
second click on the current cell edits it; column headers sort; the wheel
scrolls the pane under the pointer without moving focus; clicking outside a
modal cancels it.

### Safe Mode and the safety gate

The levels are TablePlus' own, with the same defaults and semantics:

| Level | Reads | Writes | Dangerous (`DROP`, `TRUNCATE`, `ALTER … DROP`, `DELETE` without `WHERE`) |
|---|---|---|---|
| Silent (default) | run | run | confirm |
| Alert | run | confirm | confirm |
| Alert (Full) | confirm | confirm | confirm |
| Safe Mode | run | confirm + deliberate acknowledgement | confirm + acknowledgement |
| Safe Mode (Full) | confirm + acknowledgement | confirm + acknowledgement | confirm + acknowledgement |
| Read-Only | run | refused (`Cannot execute write queries: … Safe Mode is set to read-only for this connection`) | refused |

`UPDATE` without `WHERE` is a plain write, as in TablePlus. The deliberate
acknowledgement replaces Touch ID: the dialog asks for the target table's
name, and the confirming button stays disabled until it matches. The
confirmation dialog states the action, target (connection · environment ·
database · table), scope in rows, risk, reversibility and the level that
triggered it, and shows the statement. On a production connection the strip
paints the level amber when it is Silent.

### Data grid semantics

Result sets are capped at 500 rows with a *fetch more* row and an
extrapolated total; table tabs page the same way. Edits, inserts and
deletions queue in a pending-change bar (`•` marker per row, `!` for a row the
save rejected); reverting a cell to its original value removes it from the
queue; *Preview SQL* shows the statements that *Save* would run. Sorting a
loaded table asks the source to re-query with `ORDER BY`; sorting a result
set sorts locally.

### Library boundary

The library (`junie_tui`) knows nothing about SQL. The workbench supplies:
the tokeniser and statement splitter as `Highlighter` / `Segmenter` functions
for the code editor; completion items ranked by kind (column < alias < table
< view < function < keyword < schema); the column-type → `CellKind` mapping and
a validator for the grid; filter operators ordered by column type for the
chip bar; and the safety classifier that decides which dialog to open. The
same widgets appear in the showcase with static demo data.

## Visual principles extracted from Junie

From the live site's CSS custom properties and computed styles:

1. **One hue.** `#48e054` is the only chromatic colour. It marks focus, the
   primary action, the current item and a live status. Everything else is
   achromatic, so green always means "this is the thing".
2. **Alpha ladder, not gray ramp.** Text steps down in white opacity
   (100 / 70 / 50 / 30 %), borders are white at 15 / 30 %. Tiers stay
   harmonious on any surface.
3. **State is geometry, not paint.** On the web, hover adds 10 % white, pressed
   adds a 2 px ring, selected is a border. Fills never flood.
4. **Three planes.** Canvas `#000`, chrome `#111`, cards `#18181b`. Depth is
   lightness; borders appear only where a pane needs an edge.
5. **Restraint as personality.** No uppercase labels, no weight above 600,
   generous negative space, quiet chrome.

## Design tokens

All values live in `src/theme.rs`; rendering code never spells an RGB value.

| Token | Value | Source on junie.jetbrains.com |
|---|---|---|
| `canvas` | `#000000` | `--colors-bg` |
| `surface` | `#111111` | header / footer / panels |
| `surface_elevated` | `#18181b` | `--color-card` (zinc-900) |
| `surface_overlay` | `#27272a` | `--color-input` / secondary (zinc-800) |
| `field` / `field_hover` | `#1e1e22` / `#232328` | terminal mock input box, stepped for a filled canvas |
| `popover` | `#3f3f46` | `--color-popover` (zinc-700) |
| `border_subtle` / `border_strong` | white 15 % / 30 % | `border-white/10`, `/30` |
| `text_primary` / `secondary` / `muted` / `faint` | white 100 / 70 / 50 / 30 % | `text-white/70`, `/50`, Rescui `pale` |
| `accent` | `#48e054` | `--colors-primary` |
| `accent_hover` | `#3ab343` | primary at 80 % over black |
| `accent_pressed` | `#2b8632` | darker step for the press flash |
| `accent_bg` / `accent_bg_subtle` | green 20 % / 10 % | `primary-t-fog`, `bg-primary/10` |
| `error` | `#e44545` | `--color-destructive` (red-400) |
| `warning` | `#f59e09` | amber-500 |
| `info` | `#8787ff` | Rescui purple, used only for reference |

## Component state model

Every row-like or control-like element has three glyph slots that never
collide, so any combination of states reads instantly:

| Slot | Meaning | Glyph |
|---|---|---|
| gutter (column 0) | keyboard focus | `▎` in accent (white on a primary button) |
| marker (column 1) | selection / current | `›` chosen or current, `✓` checked |
| trailing | status | `!` error, spinner while busy |

| State | Visual |
|---|---|
| hover | background lifts exactly one plane (`canvas → elevated`, `surface → overlay`); never a colour, never on disabled |
| focus | accent gutter bar + bold text; the containing frame brightens its border and title |
| focus + hover | bar and lift together — visibly different from either alone |
| pressed | reversed for 140 ms after any activation (mouse or keyboard) |
| selected | marker glyph (`›` chosen, `✓` checked); the accent tint appears only on the row that also has keyboard focus |
| disabled | faint text, no bar, no hover, skipped by Tab |
| error | trailing `!` and a message line in `error`; the field keeps its focus bar |
| editing | hardware cursor placed in the control, accent underline, `EDIT` badge in the footer; the field keeps its surface |

Navigation focus and editing are separate modes: a focused input shows the
bar, `Enter` starts editing, `Esc` reverts, `Enter` commits, `Tab` commits and
moves on. Table cells show a reversed cell for navigation and a cursor for
editing, so the two never look alike.

## Interaction conventions

- **Containers** are filled cards that hug their content; a frame (rounded,
  subtle border) is used only where a pane needs an edge, and never inside a
  card. A scrollable container that is itself the focus stop shows the same
  `▎` bar in its title row plus a bold title.
- **Green budget** per screen: the focus bar, one primary button, the chosen
  marker, the tab underline, and live activity (spinner, indeterminate sweep,
  completed bar). Row status, counts, running bars and status text stay on
  the white ladder.
- **Focus ring** is rebuilt every frame in render order, so Tab order is
  reading order and always deterministic. Disabled controls never enter it.
- **Hit-testing** is rebuilt every frame too: containers register first,
  rows and cells after, so the topmost region wins. A modal pushes a barrier
  that makes everything below unreachable for both mouse and keyboard.
- **Keyboard beats hover**: a key press suppresses hover until the pointer
  moves again, so a stale lift never competes with the focus bar.
- **Scrolling** is owned by the container: keys scroll the focused container,
  the wheel scrolls the one under the pointer, the scrollbar (`│` track,
  `┃` thumb, brighter when its container has focus) only appears on overflow,
  and titles show `12–24 of 120`.
- **Dialogs** dim the page by scaling the alpha ladder (hierarchy survives)
  while keeping its surfaces, trap focus, open on the sensible default (primary for confirmations, Cancel for
  destructive ones, the field for prompts), answer to `y`/`n`, and restore
  focus on close.

## Architecture

```
src/core      framework primitives — ids, focus ring, hit registry, scroll state, text buffer, events
src/theme.rs  design tokens + component style resolvers
src/ui        render context (interaction snapshot, hit/focus registration), text helpers
src/widgets   button, input, textarea, choice, list, tree, table, panel, tabs, dialog, progress, scrollbar,
              code (editor), completion, grid (data grid), chips, select, picker, segments, props, empty, keyhint
src/runtime   the shared event loop (raw mode, mouse, bracketed paste, coalesced input, ticks on demand)
src/bin/showcase   pages/ (one per component + two composed screens), app.rs shell, data.rs, app_tests.rs (+ visual baseline)
src/bin/tablepro   db.rs (demo catalog + row generator), sql.rs (tokens, statements, safety tiers, runner, EXPLAIN),
                   model.rs (history, completion, switcher index), tabs.rs (table / query / history tabs),
                   workbench.rs, connections.rs, app.rs (modals, safety gate, chords), app_tests.rs
tools/        headless capture harness (tmux → ANSI → PNG) used for visual review
```

Widgets are plain state structs with `render(area, buf, ctx)` — which draws
*and* registers hit regions and focus stops — and small `on_key` / `on_click`
/ `on_wheel` handlers returning `Outcome::{Ignored, Consumed, Changed}`.
Pages own their widgets and route events; the app owns focus, hover, pressed
state and dialogs.

## Towards a reusable library

- `core/`, `theme.rs`, `ui/`, `widgets/` and `runtime.rs` are the library
  today (`junie_tui`); both binaries consume it only through its public API.
  `ui::ctx::RenderCtx` is the seam between library and app.
- Widgets need three small generalisations: a trait over the `render` /
  `on_*` pair so pages can hold `Vec<Box<dyn Widget>>`, a `Theme` trait (or a
  second concrete theme) to prove the tokens are not Junie-only, and builder
  options for the few hard-coded choices (gutter glyph, marker glyphs).
- The page-level routing (`locate`/`owns` helpers on lists, trees and tables)
  should become a `Container` helper so a page can dispatch clicks with one
  call.
- Keep the rule that made this prototype coherent: **no widget chooses a
  colour; it asks the theme for a style given its `VisualState`.**

## Visual review tooling

`tools/capture.sh` runs a binary in a fixed-size tmux pane (`BIN=target/debug/tablepro
ARGS="--connect Production"`), sends keys and SGR mouse events, and captures the
pane with colours; `tools/ansi2png.py` rasterises the capture with JetBrains
Mono so rendered output can be inspected as an image. Screens in `shots/`
were produced this way: `f_*` are the showcase, `s_*` the new component
pages, `t_*` the workbench, `j_*` the Jackin preview (`BIN=target/debug/jackin-preview
ARGS="--scenario returning --motion reduced"`; use the tmux key name `Escape`,
not `Esc`, when scripting).

The showcase also carries a visual baseline (`tests/showcase_baseline.txt`):
a digest of every page at 120×40 and 80×24, excluding the navigation sidebar.
`cargo test` fails when a page changes; regenerate deliberately with
`UPDATE_BASELINE=1 cargo test --bin showcase showcase_visual_baseline`.
