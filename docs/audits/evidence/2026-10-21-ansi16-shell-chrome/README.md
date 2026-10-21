# The 16-colour Overview frame: root cause and fix

Branch: `wip/ansi16-shell-chrome-20261021` (base `9770084a9`).
Failing test: `cargo nextest run -p showcase-ui -E 'test(shell_chrome_frames_are_unchanged_by_the_shared_layout)'`
→ `Overview 100x30 inspector=false Ansi16`: actual `17536421411540798214`
(`0xf35de25ca2718b06`), pinned `5554472809535586743` (`0x4d15739766ef41b7`).

## 1. The verdict

The candidate frame at `HEAD` was **wrong** and the pin was **not** re-blessed.
The frame carried truecolour literals where the 16-colour tier has to carry
palette entries. The code is fixed in the shared block; the pin stays at
`0x4d15739766ef41b7`, and the fixed frame reproduces that digest cell for
cell.

## 2. Which cells changed

Only the section strip of the Overview page. 126 cells (row 4: 47 cells,
row 5: 79 cells; columns 21 to 99), in two runs of the 100x30 frame:

| cells | row | before (`9770084a9`) | pinned frame (`d32408843`) | reference (`visual-baseline`) |
|---|---|---|---|---|
| 21–33 | 4 | `Rgb(255,255,255)` on `Rgb(39,39,42)`, bold | `White` on `DarkGray`, bold | `Black` on `DarkGray`, bold |
| 35–44, 46–54 | 4 | `Rgb(179,179,179)` on `Rgb(0,0,0)` | `Gray` on `Black` | `Gray` on `Black` |
| 84–98 | 4 | `Rgb(115,115,115)` on `Rgb(0,0,0)` | `Gray` on `Black` | `Gray` on `Black` |
| 21–33 | 5 | `Rgb(72,224,84)` on `Rgb(0,0,0)` | `LightGreen` on `Black` | `LightGreen` on `Black` |
| 34–99 | 5 | `Rgb(58,58,58)` on `Rgb(0,0,0)` | `DarkGray` on `Black` | `DarkGray` on `Black` |

Nothing else in the frame moved: the symbols, the geometry and every other
cell are identical in all three frames. The four truecolour rows of
`CHROME_FRAMES` never failed, because truecolour is the tier the literals
were authored for.

Evidence:

* `candidate-overview-100x30-16-before-fix.cells.tsv` — the frame at `HEAD`.
* `candidate-overview-100x30-16-after-fix.cells.tsv` — the frame after the
  fix; byte-identical to the pinned frame.
* `candidate-overview-100x30-16-pinned-frame.cells.tsv` — the frame the pin
  was recorded from (`d32408843`).
* `reference-overview-100x30-16.ansi` — the frozen reference capture
  (`visual-baseline:snapshots/showcase/pages/overview/100x30/16.ansi`).

## 3. The code path

The migration `9770084a9` ("merge(showcase): integrate the shared page
chrome block") replaced the Overview page's own `frame()` and
`paint_section_strip()` with the shared `PageFrame` + `SectionStrip`
(`crates/termrock-feedback/src/page_chrome.rs`).

Before the migration the page painted the strip with its own level-aware
mapping (`crates/showcase-demos/src/pages/overview.rs`, `token_color`):

```rust
ColorLevel::Ansi16 => match hex {
    CANVAS | SURFACE => Color::Black,
    WHITE => Color::White,
    SECONDARY | MUTED | FAINT | THUMB => Color::Gray,
    RULE | TAB_ACTIVE | TRACK => Color::DarkGray,
    ACCENT => Color::LightGreen,
    _ => rgb(hex),
},
```

After the migration the page hands the block a **truecolour** palette
(`showcase_demos::pages::chrome_ink()`, the shared app policy) and the block
painted it literally:

```rust
// crates/termrock-feedback/src/page_chrome.rs, before the fix
let style = if index == self.active {
    self.ink.pair(self.ink.ink, self.ink.tab_active).add_modifier(Modifier::BOLD)
} else {
    self.ink.pair(self.ink.secondary, self.ink.canvas)
};
```

Nothing in the paint path reduces a painted colour to the frame's colour
level: `Ui::paint_str` writes the style into the buffer as it is
(`crates/termrock-runtime/src/ui/paint.rs`), and the theme downgrade only
touches theme tokens (`crates/termrock-theme/src/downgrade.rs`,
`Theme::downgrade`). So at `Ansi16` the strip carried
`Color::Rgb(255, 255, 255)` on `Color::Rgb(39, 39, 42)` while the rest of the
frame carried ANSI entries.

That is the root cause: **the shared page chrome block painted a truecolour
palette at a reduced colour level.** It is a component defect, not an
Overview defect: every page that uses `PageFrame`, `SectionStrip` or
`CardChrome` paints the same literals at the same reduced tiers.

The pre-migration page was the only demo page with a level-aware mapping;
the other demo pages paint their own literals directly (for example
`crates/showcase-demos/src/pages/lists.rs`, `fn ink`), which is why the
defect is invisible in the truecolour rows.

## 4. The reference evidence

The authority is the frozen tag `visual-baseline`
(`2718a6b8127a522a3d351d0b4bdb7542fc8da542`, baseline commit
`4169783ae883788186d161af09ba9c481bd84e5c`).

* Declared table: `visual-baseline:src/theme.rs`, `apply_16` (lines
  983–1032) declares `text_primary = White`, `text_secondary = Gray`,
  `text_muted = Gray`, `text_faint = Gray`, `text_ghost = DarkGray`,
  `border_subtle = DarkGray`, `accent = LightGreen`, `canvas = Black`,
  `surface = Black`, `surface_overlay = DarkGray`. The tag's own generic
  conversion is `nearest_16` (lines 3037–3060): a colour with a chroma below
  40 maps to `Black` (0–30), `DarkGray` (31–110), `Gray` (111–200) or
  `White`.
* Rendered oracle: `visual-baseline:snapshots/showcase/pages/overview/100x30/16.ansi`
  paints the strip as `38;5;0` on `38;5;8` (active tab), `38;5;7` on
  `38;5;0` (inactive tabs and the caption), `38;5;10` on `38;5;0` (the
  active share of the rule) and `38;5;8` on `38;5;0` (the rest of the rule).
  That is the 16-colour ink the candidate must reproduce.
* The candidate's truecolour source inks are the reference's own Junie
  palette values (`visual-baseline:src/theme.rs` lines 493–514:
  `WHITE = 0xffffff`, `WHITE_70 = 0xb3b3b3`, `TEXT_MUTED = 0x8a8a8a`,
  `TEXT_FAINT = 0x737373`, `WHITE_23 = 0x3a3a3a`, `GREEN = 0x48e054`,
  `OVERLAY = 0x27272a`), and the reference's truecolour capture paints the
  strip with exactly those values (`rgb(255,255,255)` on `rgb(39,39,42)`,
  `rgb(179,179,179)`, `rgb(115,115,115)`, `rgb(58,58,58)`, `rgb(72,224,84)`).
  The candidate therefore matched the reference in truecolour and lost the
  mapping in the reduced tiers.

## 5. The fix

`crates/termrock-feedback/src/page_chrome.rs`: the block reduces the page's
palette to the frame's colour level before it paints.

```rust
fn pair_at(&self, level: ColorLevel, fg: Color, bg: Color) -> PaintStyle {
    let (fg, bg) = match level {
        ColorLevel::Mono => legacy_mono(fg, bg, false)
            .map(|mono| (mono.fg, mono.bg))
            .unwrap_or_else(|| (downgrade_color(fg, level), downgrade_color(bg, level))),
        _ => (downgrade_color(fg, level), downgrade_color(bg, level)),
    };
    self.pair(fg, bg)
}
```

* `downgrade_color` is the library's shared conversion
  (`crates/termrock-theme/src/downgrade.rs`), the same algorithm as the
  tag's `downgrade()` (`visual-baseline:src/theme.rs` lines 3002–3060).
* The `--color none` tier uses the frozen mono table (`legacy_mono`), which
  carries the reference's own ink for the Junie colours the chrome paints.
* The level comes from the theme the frame renders with
  (`ui.theme_ref().capability.color`), so the block follows the theme and
  not a hard-coded tier.

`SectionStrip::draw` and `CardChrome::draw` paint through `pair_at`.
`PageFrame` paints theme-resolved styles only and is unchanged.

The fix is in the shared block, so it reaches every screen and every app
that uses the page chrome, as the component rules require.

## 6. Result

* `Overview 100x30` at 16 colours is cell-for-cell the pinned frame again
  (`0x4d15739766ef41b7`). The pin is unchanged and the test passes.
* The 16-colour strip now matches the reference capture in every ink of the
  strip except the active tab's foreground (see the residual findings).

## 7. Residual findings (recorded, not fixed here)

1. **The 16-colour active tab is `White` on `DarkGray`; the reference
   capture paints `Black` on `DarkGray`.** The reference repairs the pair at
   paint time: `visual-baseline:src/theme.rs`, `Theme::legible` (lines
   1624–1656) replaces a foreground whose contrast against its background
   is below 4.5:1. `White` on `DarkGray` (`127,127,127` in the tag's 16
   colour table) is 3.95:1, so the repair picks `text_on_accent` (`Black`,
   5.32:1). The library carries that repair in
   `termrock::theme::downgrade_pair` ("on 16 colors, white on dark gray
   becomes black so the label keeps the baseline contrast"). Adopting it for
   the strip's pairs would make the fixed frame match the reference capture
   exactly, but it moves the pinned digest: the 13 active-tab cells of the
   100x30 frame change from `White` to `Black`. The pin therefore stays as
   it is, and the choice is recorded here for the reference owner.
2. **The 256-colour rule ink is `Indexed(237)`, the reference declares
   `Indexed(238)`.** The reference's rule is the theme's declared
   `border_subtle` slot (`visual-baseline:src/theme.rs`, `apply_256`:
   `self.border_subtle = i(238)`), while the page's literal `0x3a3a3a` maps
   to the nearest 256 grey, which is 237. The candidate's own theme declares
   `border_subtle = Indexed(235)` at 256
   (`crates/termrock-theme/src/palettes.rs`), so neither the theme slot nor
   the literal reaches the reference's 238. This is a theme-table question
   for the theme workstream.
3. **The mono rule ink is `Black` on `Black`; the reference paints
   `DarkGray` on `Black`.** The frozen mono table
   (`termrock::theme::legacy_mono`) carries an entry for the theme's
   `border_subtle` literal `0x262626`, not for the page's `0x3a3a3a`, so the
   pair falls back to the generic mono conversion. The other eight inks of
   the strip match the reference capture exactly at the mono tier.

## 8. Commands

```sh
git worktree add /private/tmp/ansi16-shell-chrome-20261021 \
  -b wip/ansi16-shell-chrome-20261021 9770084a9
export CARGO_TARGET_DIR=/private/tmp/ansi16-target-20261021

# reproduce
cargo nextest run -p showcase-ui --no-fail-fast \
  -E 'test(shell_chrome_frames_are_unchanged_by_the_shared_layout)'

# the frame dumps
cargo nextest run -p showcase-ui --no-fail-fast -E 'test(dump_ansi16_cells)' --no-capture

# the regression tests
cargo nextest run -p termrock-feedback --no-fail-fast \
  -E 'test(/reduced_frame|sixteen_colour_strip|two_hundred_fifty_six|mono_strip/)'

# the gate
cargo nextest run --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

The reference captures were read with

```sh
git -C /Users/donbeave/Projects/terminal-components-claude \
  show visual-baseline:snapshots/showcase/pages/overview/100x30/16.ansi
git -C /Users/donbeave/Projects/terminal-components-claude \
  show visual-baseline:src/theme.rs
```
