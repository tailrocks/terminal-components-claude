# Card heading styles

Owner request (2026-10-17): on a standard (unframed) card the heading row
(`Titled card … surface`, `Card · scrollable … 1–7 of 60`) is drawn in the
same plane and nearly the same ink as the body, so it is unclear where the
heading ends and the body starts. Framed panels are fine (the title sits in
the frame) and stay as they are. The current look stays the default; a
distinct heading is an option.

## Where the card stands today

`Panel::card` (`src/widgets/panel.rs`) fills `surface`, draws the title on
row 0 at `pad_x` (secondary ink; primary + bold while focus is inside, the
focus-within cue), the meta right-aligned in faint ink, keeps row 1 blank
and starts the content on row 2. Only the ink step (secondary vs the body's
secondary/muted text) separates heading from body; at 16 colours and in
monochrome even that collapses (secondary, muted and faint are all `Gray`).

## Candidates

Every candidate keeps the content area exactly where `Plain` puts it, so a
style can be switched app-wide without moving any layout, and none touches
the framed variant.

| Style | Truecolor / 256 | 16 colours | Monochrome (`NO_COLOR`) | Notes |
|---|---|---|---|---|
| **Plain** (default) | today's look | today's look | bold only while focused | byte-identical to the baseline |
| **Rule** | `Glyphs::h_divider` (`─`) across the blank row 1 in `Theme::rule_on(surface)` | `DarkGray` rule on `Black` | the glyph shape carries it; ASCII `-` | no new plane, no focus interaction |
| **Band** | row 0 on the plane one step up (`Plane::up` → raised), inks via `Theme::legible` | `DarkGray` band, ink kept ≥ 4.5:1 (black) | fill is stripped: title underlined | raised plane is also the plane of buttons and chips |
| **Emphasis** | title bold in every state (secondary at rest, primary focused) | same | bold is already the focus-within cue, so the focused title also underlines | weakest separation; changes the focus cue's contrast |

Contrast: titles stay ≥ 4.5:1 at every level (unit test
`every_style_reads_and_sets_the_heading_apart_at_every_level`). The rule is
a non-text separator, not information-bearing (WCAG 1.4.11 does not apply),
but it is still visible at every level and in monochrome by its shape.

```
Plain (default)                       Rule (recommended)
  Titled card                surface    Titled card                surface
                                        ──────────────────────────────────
  A card is a filled surface. Its       A card is a filled surface. Its
  title sits in the top-left and        title sits in the top-left and

Band                                  Emphasis
▓▓Titled card▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓surface▓▓  **Titled card**            surface
                                      
  A card is a filled surface. Its       A card is a filled surface. Its
```

## Recommendation: Rule

- **Reference.** macOS inspectors and grouped settings sections separate a
  section's header from its content with a hairline separator, not with a
  fill; a filled header strip reads as a toolbar or a control.
- **No new plane.** The raised plane (`Band`) is where secondary buttons,
  chips and the active tab sit (`Plane::up`): a heading on it suggests it is
  clickable. At 16 colours its `DarkGray` fill also forces black ink.
- **Focus untouched.** `Emphasis` makes every title bold, which is the
  monochrome focus-within cue of a plain card; it then needs an extra
  underline to show focus. `Rule` leaves the title and its focus cue alone,
  and the single `▎` rule of the shell is unaffected.
- **Every level.** A glyph survives `NO_COLOR` and ASCII (`-`), so the
  separation is the same in monochrome as in truecolor.
- **Zero layout cost.** It uses the blank row a titled card already keeps.

`Band` and `Emphasis` stay available for apps that want them.

## API

- `CardHeading { Plain (default), Rule, Band, Emphasis }` in
  `widgets::panel`, with `CardHeading::ALL` and `label()`.
- App-wide: `Theme::card_heading` / `Theme::with_card_heading(style)`.
- Per card: `Panel::heading_style(style)`, which wins over the theme.
- A framed panel ignores both; an untitled card has no heading and draws
  as `Plain`.
- Showcase: Layout › Panels option `panels.heading`
  (`theme | plain | rule | band | emphasis`, `theme` = the shell's choice),
  shell option `shell.card_heading` (every card in the showcase), and a
  States column per style.
