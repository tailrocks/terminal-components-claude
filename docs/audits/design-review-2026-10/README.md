# Design review, October 2026

Findings from parallel audits of every interface on the
`design/interaction-polish` branch, one file per category. They are the
backlog for that branch: each fix lands in the shared component or theme
layer (see `AGENTS.md`), with a regression test, in its own commit.

| File | Category |
|---|---|
| `focus.md` | One focus indicator: focus, selection and "active" hierarchy |
| `color.md` | Colour, contrast and state styling at every colour level |
| `interaction.md` | Mouse and keyboard parity with desktop conventions |
| `layout.md` | Layout, truncation, overflow and responsiveness |
| `api.md` | Hardcoded behaviour that must become configuration |
| `copy.md` | Copy, key hints and feedback |

Line numbers refer to the code at the time of each audit and drift as
fixes land.
