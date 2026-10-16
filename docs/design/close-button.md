# Close buttons: one padded plate for every `×`

Status: accepted (2026-10-16). Owner: component library (`src/widgets/icon_button.rs`).

## Problem

Product feedback: close buttons are visually too small, worst on hover.

- Showcase › Dialogs › "Rename task": the frame's top border reads `… × ─╮`.
  Hovering the `×` lights exactly one cell; the two padding cells around it
  keep the border plane, so the hover reads as a one-cell blip.
- tablepro tab strip `≡ Query 1 ×    ≡ Query 2 ×`: the tab `×` has a 3-cell
  hit target (`Tabs::close_target_w = 3`) but only the glyph cell changes on
  hover, so the target the user can actually hit is invisible.
- The `+` (new tab) at the right end of the same strip already does it
  right: ` + ` is three cells, all three take the hover plane.

The hit area was already wider than one cell in most places; the problem is
that the **drawn** target was not. A target the user cannot see is a target
they aim at as if it were one cell.

## Inventory (before)

| Control | Where | Drawn cells | Hit cells | Rest | Hover | Pressed |
|---|---|---|---|---|---|---|
| Modal frame `×` | `ui::modal::close_button` (used by `modal::Frame`, `Dialog`, `HelpOverlay`, `FormDialog`, `ChoiceDialog`, `InfoDialog`, jackin build log / app modals, holla modals) | ` × ` on the top border, `×` styled, pads in border style | 3 | secondary ink on the card | `×` cell only: card hover plane | `×` cell only: card pressed plane + bold |
| Tab `×` | `widgets::tabs::Tabs` (tablepro query tabs, holla tabs, jackin capsule tabs) | `×` (1 cell) | `close_target_w` = 3, centred | secondary ink on the tab plane | glyph cell only | glyph cell only |
| Chip `×` | `widgets::chips::ChipBar` (tablepro filters, holla activity, showcase chips) | `×` (1 cell) | `close_target_w` = 3, centred | as tab | glyph cell only | glyph cell only |
| Tab `+` (new) | `Tabs::allow_new` | ` + ` (3 cells) | 3 | title ladder rest | 3-cell hover plane | 3-cell pressed plane |
| Tab `+N` (overflow) | `Tabs` / `ChipBar` | ` +N` | same | title ladder | full | full |

There is no app-local close drawing: all four apps route the library's ids
(`modal::CLOSE`, `Tabs::close_id`, `ChipBar::close_id`). Panels, drawers and
toasts have no `×` (they close by Esc, focus leaving, or time).

At 16 colours the grey planes cannot always step (hover = rest), so hover is
the `hover_flat` underline; in `NO_COLOR` hover underlines and pressed is
reversed. On a one-cell target both read as "the glyph flickered".

## Standards

- **macOS** (reference): window close is a circular control whose glyph and
  fill appear on hover of the traffic-light group; Safari/Finder tab close
  is an `×` on a rounded hover plate larger than the glyph. No red tint on
  tab close; red only for the window's traffic light.
- **Windows**: window close is a full-height rectangle that fills red on
  hover. Tab close (Edge, Terminal) is a square plate with a neutral hover.
- **Browsers** (Chrome, Firefox, Safari): tab `×` sits on a rounded hover
  plate; on narrow tabs it shows only on the active / hovered tab.
- **Fitts's law**: time to hit grows as the target shrinks; a target should
  be visibly as big as it acts.
- **WCAG 2.2 SC 2.5.8 Target Size (Minimum)**: 24×24 CSS px. A terminal cell
  is roughly 8–10 px wide and 16–20 px tall, so the analogue is **3 cells
  wide × 1 row** (24–30 px × 16–20 px). Hit area must equal what is drawn.
- **WCAG 1.4.11**: the hover plate is a non-text state indicator (≥ 3:1
  where colour carries it; the modifier carries it where colour cannot).

## Approaches considered

**A. Padded plate (` × `), like `+` — chosen.** Three cells (glyph plus one
cell of padding each side). Rest: no plate, secondary ink on the host, so a
dialog or a tab strip stays quiet. Hover: all three cells take the hover
step of the plane the button sits on (the `Kind::Close` ladder). Pressed:
the pressed step plus weight (16 colours: reversed; mono: reversed). The hit
area is exactly the plate. Matches the `+` the PO pointed at, macOS's hover
plate and WCAG 2.5.8.

**B. Notch cut into the frame (`┤ × ├`).** Reads as a "real" button at rest
and frames the target, but adds two border glyphs to every dialog (busier
than the baseline's visual language), breaks the clean `─` run, needs 5
cells, and does nothing for tabs and chips. Rejected: a frame-only fix and a
visible change to every modal at rest.

**C. Labelled `× Close` / `Esc` chip on wide dialogs.** Discoverable, but it
duplicates the footer's Cancel / Close action and the `Esc Close` hint,
costs 9+ cells of title room, and does not apply to tabs or chips.
Rejected: redundant; the tooltip (below) carries the label and the key.

**D. Tab `×` only on hover / active tab, with a plate.** The browser idiom
for narrow tabs; less noise on a long strip. But in a terminal the pointer
is often absent (keyboard users), so a hidden `×` hides the affordance, and
a strip that reflows on hover moves targets under the pointer. Kept as an
**option** (`Tabs::close_reveal(CloseReveal::Hover)`): the space stays
reserved, nothing reflows; default stays `Always`.

**Danger tint (Windows red) on hover**: off by default (macOS: closing a tab
or a dialog is not destructive; the dialog resolves to its cancel action).
Available as `IconButton::danger_hover(true)`: hover takes the danger ink,
pressed the danger fill.

## The system

One shared primitive, `widgets::icon_button::IconButton`, draws every
padded glyph button: the close `×` (modal frames, tabs, chips) and the tab
strip's `+`. Nothing else draws a close glyph.

- **Geometry**: plate = `pad` + glyph + `pad` cells (default `pad = 1`,
  3 cells). `IconButton::width()` reports it; the hit region registered is
  exactly the plate.
- **Styling**: through `Theme::interactive` only — `Kind::Close` for close
  buttons (`Theme::close_plate`), `Kind::Title` for `+`. Rest keeps the
  host plane with secondary ink; hover / pressed paint the whole plate.
  The theme's ladder tests already keep rest / hover / pressed distinct for
  `Kind::Close` on every plane and colour level, and distinct from focus
  (a close button never carries the `▎`: keyboard focus stays on the owner).
- **Glyph**: `Glyphs::close` (`×`, ASCII `x`); the `+` passes its own.
- **Frame**: the plate sits two cells in from the right corner on the top
  border (`╭─ Title ───── × ─╮`): the corner and its `─` lead-in stay, so
  the `╮` is never displaced. Title and meta reserve `width + 1`.
- **Tabs / chips**: the plate is the cell before the glyph, the glyph and
  the cell after it, inside the tab or chip; the tab or chip itself is not
  hovered while the pointer is on the plate (`child_state`).
- **Tooltip** through `ui::tooltip`: `Close · Esc` on frames, `Close tab · x`
  on tabs (first deliverable `TabCmd::Close` binding), `Remove · Delete` on
  chips, `New tab · n` on `+`. Texts from `Strings`.
- **Keyboard equivalence**: frames close with Esc (the `Dismiss` policy:
  the `×` only shows when Esc may close); tabs with `x` / Delete; chips with
  Delete / Backspace / `x`; `+` with `n`. Clicking the `×` means exactly what
  the key means.

### Knobs

| Knob | Type | Default |
|---|---|---|
| `IconButton::pad` | `u16` | `1` (3-cell plate) |
| `IconButton::danger_hover` | `bool` | `false` |
| `IconButton::tip` | `Option<Tip>` | per use (see above) |
| `Tabs::close_button` / `ChipBar::close_button` | `IconButton` template | `IconButton::close`, `pad 1` |
| `Tabs::close_reveal` | `CloseReveal` (`Always`, `Hover`) | `Always` |
| `modal::Frame::close_button`, `Dialog::close_button` | `IconButton` | `IconButton::close(modal::CLOSE)` |

The `close_target_w(cells)` builder (tabs, chips) stays and now sets the
plate width (`pad = (cells - 1) / 2`), so a plate never draws smaller than
it hits; the separate hit-width field is gone.

## Visual change (intended)

Every close `×` shows a 3-cell plate on hover and press instead of a lit
glyph. At rest nothing changes except where a hit area was narrower than
the plate. Re-approved captures must differ only at close buttons.
