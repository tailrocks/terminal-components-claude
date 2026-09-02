# Junie TUI — a Ratatui design-system laboratory

A runnable, interactive prototype that answers one question: *if the
[Junie](https://junie.jetbrains.com) website had been designed for a terminal
instead of a browser, what would its components look and feel like?*

The application is the specification. Every component page exposes real
interactive states (default, hover, focus, pressed, selected, disabled, error,
editing), two composed screens show the parts working together, and the
architecture is shaped so the widgets can later be extracted into a reusable
Ratatui library.

## Run

```sh
cargo run --release
cargo run --release -- --page tables        # start on a page (overview, buttons, inputs, … settings, taskrunner)
cargo run --release -- --color 256          # force a colour level: truecolor|256|16|none
```

Requirements: Rust 1.88+, a terminal with mouse support. Truecolor is the
primary target (`COLORTERM=truecolor`); 256/16-colour terminals get a mapped
palette, `NO_COLOR` gives a monochrome fallback. Minimum size is 72×20; below
that a reduced state is shown until the terminal grows.

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
src/widgets   button, input, textarea, choice, list, tree, table, panel, tabs, dialog, progress, scrollbar
src/pages     one showcase page per component + two composed screens
src/app.rs    shell layout, event routing, modal stack, footer hints, inspector
src/data.rs   demo data
tools/        headless capture harness (tmux → ANSI → PNG) used for visual review
```

Widgets are plain state structs with `render(area, buf, ctx)` — which draws
*and* registers hit regions and focus stops — and small `on_key` / `on_click`
/ `on_wheel` handlers returning `Outcome::{Ignored, Consumed, Changed}`.
Pages own their widgets and route events; the app owns focus, hover, pressed
state and dialogs.

## Towards a reusable library

- `core/` and `theme.rs` are already free of demo code and can move to a
  crate as-is. `ui::ctx::RenderCtx` is the seam between library and app.
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

`tools/capture.sh` runs the binary in a fixed-size tmux pane, sends keys and
SGR mouse events, and captures the pane with colours; `tools/ansi2png.py`
rasterises the capture with JetBrains Mono so rendered output can be inspected
as an image. Screens in `shots/` were produced this way.
