# Design review, October 2026

## Status

Historical evidence. Its dated findings and source pins describe the recorded review only.
## Authority

Current authority: main defines the intended split-crate architecture; design/interaction-polish defines UI/UX, colors, and behavior.
## Current evidence

Use only findings tied to the revisions and methods recorded in this document; they do not establish current implementation state.
## Remaining work

Preserve concrete requirements and unresolved items in docs/remaining-work.md; superseded procedural directions are not current instructions.
## Verification

Verification is limited to the recorded commands, revisions, and results. Re-run against current sources before closure.


Findings from parallel audits of every interface on the
`design/interaction-polish` branch, one file per category. They are the
backlog for that branch: each fix lands in the shared component or theme
layer (see `AGENTS.md`), with a regression test, in its own commit.

What is still open across these files (and the owner questions they
raise) is collected in [`docs/remaining-work.md`](../../remaining-work.md).

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
| `colour-showcase-tablepro.md` | Colour as meaning in showcase and tablepro: semantic roles, syntax and environment hues |
| `colour-holla-jackin.md` | Colour as meaning in holla and jackin: live, done, health and quota states |
| `colour-follow-ups.md` | The resolved colour tokens the library shipped, decisions between the two colour reviews, and the colour follow-ups per app |
| `showcase-coverage-library.md` | Showcase coverage of every library widget, variant, state and option (244-row matrix) |
| `showcase-coverage-apps.md` | Showcase coverage of the UI patterns the apps use (92 patterns), and which are still app-local |
| `responsive.md` | Every app screen and overlay keeps its content reachable at every size (the reachability gate, `tests/reach_allow.txt`) |

The three duplication reviews are consolidated into one plan:
[`docs/architecture/shared-components-plan.md`](../../architecture/shared-components-plan.md).
The two showcase coverage inventories feed its showcase coverage track
(SC, §7 of the same plan).

Line numbers refer to the code at the time of each audit and drift as
fixes land.
