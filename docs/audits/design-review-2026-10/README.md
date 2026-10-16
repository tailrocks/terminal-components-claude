# Design review, October 2026

Findings from parallel audits of every interface on the
`design/interaction-polish` branch, one file per category. They are the
backlog for that branch: each fix lands in the shared component or theme
layer (see `AGENTS.md`), with a regression test, in its own commit.

| File | Category |
|---|---|
| `focus.md` | One focus indicator: focus, selection and "active" hierarchy |
| `states.md` | Rest, hover, pressed, focused, selected and disabled for every control |
| `color.md` | Colour, contrast and state styling at every colour level |
| `interaction.md` | Mouse and keyboard parity with desktop conventions |
| `layout.md` | Layout, truncation, overflow and responsiveness |
| `api.md` | Hardcoded behaviour that must become configuration |
| `copy.md` | Copy, key hints and feedback |
| `hscroll.md` | Horizontal scrolling and position for wide content |
| `performance.md` | Scrolling cost of tables, grids, lists and trees |
| `keyboard.md` | Keyboard-only use: focus order, caret, Esc, quit safety and hints |
| `mouse.md` | Mouse-only use: routing, dead ends, navigation, menus, drag and wheel |
| `standards-interaction.md` | Standards review of interaction decisions (HIG, WAI-ARIA APG, WCAG 2.2) |
| `standards-visual.md` | Standards review of visual tokens and states (contrast, colour, glyphs) |
| `duplication-shell.md` | Duplicated shell logic across the four apps: routing, modals, focus, hints, help, quit |
| `duplication-widgets.md` | Duplicated interaction and state logic inside the library widgets |
| `duplication-screens.md` | Hand-built screen UI that belongs in library components |

The three duplication reviews are consolidated into one plan:
[`docs/architecture/shared-components-plan.md`](../../architecture/shared-components-plan.md).

Line numbers refer to the code at the time of each audit and drift as
fixes land.
