I reviewed the theme tokens, AGENTS.md "Visual states" and the audit docs, and computed contrast ratios from the token values. I did not render fresh screens. Paths: `/Users/donbeave/Projects/terminal-components-claude/src/theme.rs` (palette around lines 350–420; `StateModifiers` around lines 200–250), `/Users/donbeave/Projects/terminal-components-claude/AGENTS.md`, and `/Users/donbeave/Projects/terminal-components-claude/docs/audits/design-review-2026-10/`.

Three findings matter most:
- **Faint text fails your own 4.5:1 rule.** The token comment only claims "at least 3:1".
- **Selection fills fail non-text contrast** (about 1.7:1 against the canvas), and nothing but colour marks a selected row.
- **"Delete…" breaks the ellipsis convention**, and a destructive action should never be the default (primary) button.

| # | Verdict | One-line correction |
|---|---|---|
| 1 | Agree with adjustment | Keep one accent. The neutral "active" state must not rely on colour alone (add bold or `•`) and must reach 3:1 against its siblings. Return focus to the control that opened a modal when it closes. |
| 2 | Agree with adjustment | Hover must stay weaker than focus. Drop underline on hover in mono and 16-colour; use DIM/BOLD instead and keep underline for links. Fix the mono pressed-fill clash. |
| 3 | **Contradicts** (your own rule) | `#737373` faint text is 3.74:1 on card, 3.14 on overlay, 2.2 on popover. Faint text needs its own value per plane so each reaches at least 4.5:1. |
| 4 | Agree with adjustment | Keep `▎` plus BOLD. Panel title focus needs more than a style change (marker or reversed title), and a tab strip needs a non-colour cue. |
| 5 | Agree with adjustment | Hover reveal must be dismissable, hoverable and persistent (WCAG 1.4.13), and also reachable by keyboard. Default on, with an option. |
| 6 | Agree | Thumb states need a glyph difference, not only colour. Know the content-height cost of the bar row up front so the layout doesn't flicker. |
| 7 | Agree with adjustment | No group should contain a leaf with its own name ("Tables ▸ Tables"). Overview sits alone at the top. Rename "Chrome" to "App shell". Two levels at most. |
| 8 | Agree with adjustment | Choose the control by option count. Show `▾` only on the active or hovered cell. Booleans use `☐/☑`, not a toggle. |
| 9 | Agree with adjustment | Destructive is never the default button. Cancel is the default and is focused first; the red button is filled but not the primary. |
| 10 | Agree | The dim must work at 16 colours and mono (DIM modifier, the colour.md #2 fix). The footer shows the modal's hints, not the page's. |
| 11 | Agree with adjustment | Errors take precedence over hints and stay until dismissed or replaced (WCAG 4.1.3, 2.2.1). Compact mode off by default is right. |
| 12 | **Contradicts** (ellipsis); adjustment (keys) | "Delete…" is wrong when only a confirmation follows. Key names must be one consistent set: `Del`, not "Delete", next to "Esc". |
| 13 | Agree with adjustment | A `│` track looks like a panel border. Draw the track in `border_subtle` or don't draw it; the thumb carries the state. |
| 14 | **Contradicts** (WCAG 1.4.11 / 1.4.1) | `#173f1c` and `#1f3460` are about 1.7:1 against black, so selection needs a glyph or modifier cue. `SUCCESS` vs `GREEN` is 1.46:1, so it is not a usable distinction. |
| 15 | Agree with adjustment | Disabled items stay reachable by the cursor. Show the reason in the hint or status row when the cursor is on the item, and keep the shortcut dimmed. |
| 16 | Agree | Same 1.4.13 rules as #5. Clamp to the screen edge, with the 1-cell margin from #9. |

### Detailed notes

**1. Single focus indicator.** WCAG 2.4.7 and the WAI-ARIA APG patterns support exactly one focus point.
- **Blue popup cursor is correct.** In a combobox or menu the owner keeps focus and the cursor row is the "active descendant" (APG Combobox). That justifies blue for the row.
- **Neutral active state needs two fixes.** "Active but unfocused" (active tab, current page) must still be readable without colour (1.4.1) and reach 3:1 against neighbouring items (1.4.11). Change: active = text_primary + BOLD (or a `•` marker); inactive = text_secondary.
- **Focus return after a modal** (APG Dialog). Restore focus to the control that opened the modal. Default behaviour, no option.

**2. Rest / hover / pressed everywhere.**
- **Not a standard requirement.** Material 3 state layers (hover 8%, focus and pressed 10%) and HIG treat hover as a weak, optional mouse cue. Many keyboard-first TUIs (lazygit, helix) have no hover at all. Keep it, but hover must always read weaker than focus.
- **Underline means "link".** Underline on hover in 16-colour and mono collides with links.
- **Mono clash.** `MONO.pressed_fill = BOLD|UNDERLINED` makes a pressed fill look like a bold hover.
- **Change:**
  - Mono: hover = BOLD; pressed = REVERSED; pressed on a fill = REVERSED removed + BOLD (no underline).
  - 16-colour: hover moves from DarkGray to Gray where it can; otherwise BOLD.
  - Add a knob: `StateModifiers.hover_enabled` (default true).

**3. Text contrast.** WCAG 1.4.3 requires 4.5:1 for all non-incidental text, faint metadata included. Disabled text is exempt (inactive components), so `#4d4d4d` is fine.
- **Measured.** `#737373` reaches 4.43:1 on black, 3.74 on card, 3.14 on overlay and 2.2 on popover.
- **Change:** make `text_faint` plane-relative (as the ladders already are). Floor values are about `#7a7a7a` on canvas, `#868686` on card, `#959595` on overlay and `#b3b3b3` on popover. Extend the contrast test to faint text on every plane it can land on.
- **16-colour danger fallback.** White is acceptable only if the label text itself says "Delete" (1.4.1). Keep it.

**4. Focus marks.**
- **Gutter is fine.** `▎` (U+258E) is in every common font. Green on black is 12:1, which passes 1.4.11 and the spirit of 2.4.11.
- **Panel title.** A title colour or BOLD change alone is a weak cue. WCAG 2.4.13 (AAA) wants a change of area and contrast. Change: focused panel title = accent + `▎` before the title, or reversed title text; also bring the panel border to `border_strong`.
- **Tabs.** Keep the accent rule, and also apply BOLD to the focused tab's label so mono shows a difference.

**5 / 16. Reveal on demand.** WCAG 1.4.13 requires that content shown on hover or focus can be dismissed (Esc), can be hovered without vanishing, and persists until dismissed.
- **Keyboard trigger:** a key (for example `Space` or `?`) on a truncated focused cell expands it.
- **Default:** `reveal: OnHoverAndKey`, with an option to turn it off.
- **Truncation marker:** keep the ellipsis, and use one glyph (`…`) only, never `...`.

**7. Showcase tree.**
- **Naming.** Storybook naming (Category / Component / Story) forbids a child with its parent's name. Use Tables ▸ {Basic table, Editable, Data grid, Selection, Cell editors}.
- **Placement.** Put "Option editors" under Inputs if it also covers forms; otherwise rename it "Cell editors".
- **Structure.** Overview is a top-level leaf, not part of "Chrome". Expand only the current page's group by default, offer type-to-filter, and stop at two levels.

**8. Table editors.** Follow the option-count ladder from Material and HIG:
- 2 options: checkbox `☐/☑` (Material data tables use checkboxes, not switches).
- 2–5: segmented control or radio popover.
- 6–15: dropdown.
- More than 15: searchable combobox (APG Combobox).
- `▾` on every cell is visual noise; Excel shows its arrow only on the active cell. Default: show `▾` on the focused or hovered cell, with option `enum_affordance: Always`.
- Status pills are read-only. Never let them look editable.

**9. Dialogs.**
- **Default button.** Apple HIG Alerts and Windows guidelines both say not to make a destructive action the default button. Cancel takes the default and initial focus; the destructive button is red with an outline or red text. Not accent-filled, and not the Enter target.
- **Layout.** Rightmost primary and centred placement are fine (HIG alerts).
- **Default and option.** Destructive dialogs default to Cancel. Option: `default_button`.

**11. Footer status.** Status messages must reach the user without taking focus (WCAG 4.1.3), and errors must not time out (WCAG 2.2.1). Change:
- Priority: error > hint > info.
- Errors persist until the next action.
- Info times out after 5 seconds or more.

**12. Ellipsis and key names.**
- **Ellipsis.** Apple HIG Menus and Windows UX guidelines: use an ellipsis only when the command needs more input. Never use one when a confirmation is all that follows, so "Delete…" becomes "Delete", while "Rename…" and "Export…" keep theirs.
- **Key names.** Footer keycap set: `Esc Enter Tab Space Del Bksp PgUp/PgDn Home/End ↑↓ ←→`. Show "Delete" spelled out only as the action word, never as the key name.

**13. Scrollbars.** The track glyph `│` is identical to panel borders. Change: track in `border_subtle` (or a dotted `┆` / no track at all); thumb in `┃`/`━` in text_secondary; held = `█`. The glyph difference already carries state in mono, which is good.

*Done 2026-10-16:* the dotted track (`┆` / `┄`, its own `Theme::scroll_track` ink). Decision log in [hscroll.md](hscroll.md#decision-the-track-is-a-dotted-rail-2026-10-16). The thumb keeps `┃`/`━` in every state (AGENTS.md); held stays the accent, not `█`.

**14. Selection colour.**
- **Selection fill.** Selection is information needed to identify state, so it needs 3:1 against adjacent colours (1.4.11). `#173f1c` and `#1f3460` are about 1.7:1. Change: always pair the fill with a non-colour cue (a `✓`/`●` marker column or BOLD) and keep the fill as reinforcement.
- **Two greens.** Success (`#3fb950`) vs accent (`#48e054`) is 1.46:1 and almost the same hue, so they can't be told apart, especially with deuteranopia. Change: success always carries a `✓` glyph, or shift it toward teal (around `#2fb5a0`).
- **Green selection tint.** It also weakens #1's "only one green" rule. Consider `GREEN_25` → a neutral (`#2a2a2f`) + marker.

**15. Disabled menu items.**
- **Reachable.** The WAI-ARIA APG Menu pattern keeps disabled items focusable but not activatable.
- **Reason placement.** Swapping the shortcut for the reason truncates long reasons and hides the shortcut. Default: dimmed shortcut stays in place; the full reason appears in the hint or status row when the cursor is on the item. Option: `reason_inline`.

### Gaps not yet raised

1. **Field and control boundaries fail WCAG 1.4.11.** `border_subtle #3a3a3a` is 1.85:1 on black and 1.56:1 on card; the `INPUT` fill is 1.26:1 on canvas. Input outlines need `border_strong` (at least 3:1).
2. **Pressed direction is inconsistent across ladders.** Neutral planes get lighter when pressed, while highlight and red get darker (`HIGHLIGHT_PRESSED #244a8c`, `RED_45_PRESSED`). Pick one rule for every ladder (Fluent: hover lighter, pressed darker).
3. **Two blues with different meanings.** `highlight` (menu cursor) and `selection_bg` (text or range selection) are both blue. Where both appear in a combobox list, a cursor row inside a selected range is ambiguous. Define a combined token (`highlight_on_selection`).
4. **DIM (SGR 2) support is uneven.** Older Windows conhost and some multiplexers ignore it, which leaves mono disabled and backdrop undistinguished. Pair it with a glyph or a reason; document a fallback.
5. **BOLD-as-bright.** Many 16-colour terminals render BOLD as the bright colour. Then BOLD for focus and pressed changes hue unpredictably, and BOLD on Gray text can equal White. Test against a bold-is-bright profile.
6. **Wide-glyph and font risk.** `▾ ☐ ☑ ✓ ↵` vary in width (East Asian ambiguous width) and in font coverage. Provide an ASCII fallback set (`v [ ] [x] * Enter`) behind `Theme::glyphs` and select it automatically for `TERM=linux` or the Windows console.
7. **Colour variables, not a light theme.** No light theme or terminal-palette-following mode exists. Users of light terminals get a black canvas forced on them. At minimum, honour `COLORFGBG` or offer a `follow_terminal` option at the 16-colour level.
8. **Reduced motion.** Fade edges, any animated spinners or progress, and possible blinking cursors need a `reduce_motion` knob (WCAG 2.3.3 / 2.2.2).
9. **Focus order and visibility after scrolling.** Focus that scrolls off-screen inside a scroll view violates 2.4.11. Auto-scroll focused items into view with at least one row of context.
10. **Hint bar alignment per app.** `Theme::hintbar.align` is centred in Holla and Jackin and left in TablePro and Showcase. One library visual language should pick one; left-aligned is the convention in lazygit, k9s and helix.

## Product owner decision

Defaults must follow the standards cited in this review. Every behaviour
stays configurable, so users who prefer different behaviour can override
the defaults (app-wide profile, per-widget setting, or key map; see
`AGENTS.md`, "Component architecture").
