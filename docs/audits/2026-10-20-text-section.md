# TextSection extraction record

`termrock_feedback::TextSection` centralizes the live inert title/subtitle/body
painting duplicated by holla Files, Review, and Snapshot screens. It preserves
the established one-cell horizontal inset, primary bold title, muted text,
single-row clipping, files body at row 3, and untouched cells outside painted
text. It adds builder overrides for inset, body row, and style patches.

The `visual-baseline` tag object is
`2718a6b8127a522a3d351d0b4bdb7542fc8da542`; its peeled source commit is
`4169783ae883788186d161af09ba9c481bd84e5c`. Pinned tuiscotti commit is
`a1ea27f757d75d07b05499276cd932e9f8e067ad`. This bounded extraction does not
establish full application parity. Historical `RUNS` stamps remain application
domain behavior and are not moved into a reusable section.

Showcase has no existing plain title/subtitle/body composition: its header is a
breadcrumb with actions, while its sidebar title shares a row with a count and
precedes a rule and tree. The existing sidebar title now uses TextSection with
its resolved title style and zero inset, preserving the same painted cells.
The production inset-zero/title-style configuration has a cell and cursor
comparison against direct painting, including prefilled background and style
modifiers. The current checkout has no modular API catalog or coverage
allowlist; `crates/showcase-ui/tests/app_tests.rs` covers application flows
only. Scoped coverage exemptions:

| Capability | Reason |
| --- | --- |
| `TextSection.subtitle`, `TextSection.body` | Existing holla screens consume both; adding visible showcase text changes frozen output. |
| `TextSection.body_row` | Only baseline row 3 is used; configurable override is cell-tested without adding a line. |
| `TextSection.title_patch`, `TextSection.text_patch` | Style overrides are cell-tested; a styled showcase variant would change frozen colors or attributes. |

These exemptions are recorded here because no executable catalog/allowlist
exists in this checkout. Adding a screen or visible line would violate frozen
baseline parity.

Paired tuiscotti proof remains an open parity blocker: scenario
`holla::parity::files::120x40::truecolor` (`initialHome`) differs in 1,226
cells. It launches Holla at HOME with `--scenario parity-files` and sends no
inputs, so this is a Home/cataloguefixture frame, not the Files browser and
not evidence about `TextSection` or `screens/files.rs`. Evidence includes
`/tmp/termrock-paired-proof-20261020/baseline/target/tuiscotti/actual/holla/parity/files/120x40/truecolor.verdict.json`,
the corresponding `expected/` frame, `diff/holla/parity/files/120x40/truecolor.png`,
and
`/tmp/termrock-paired-proof-20261020/candidate-initial/holla/parity/files/120x40/truecolor.frame.json`.
The Holla cleanup flow has no candidate capture under
`/tmp/termrock-paired-proof-20261020/candidate-initial/holla/flows/cleanup/`,
so that flow remains unverified. This extraction does not resolve either
finding; neither evidence nor approved baseline snapshots were changed.
