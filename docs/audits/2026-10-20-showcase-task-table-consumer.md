# Showcase task table consumer

Frozen source: `visual-baseline` commit `4169783ae883788186d161af09ba9c481bd84e5`.

The baseline task table has columns ID, Task, Owner, Status, Branch, Changes,
and Duration. Minimum widths are 5, 24, 7, 9, 20, 9, and 9 cells. Task is the
flex column; Changes and Duration are right aligned. The first, sixth, and
seventh columns sort numerically. All other sortable columns compare text.
Sorting cycles unsorted, ascending, descending, then unsorted. Initial row
order is the task data order, with the cursor at the first task row. Arrow
keys move by row; left and right do not move the cursor between columns.

Current task domain data lives in `showcase-data::TASKS` as 24 `TaskRow`
values. `TaskRow.id` supplies stable identity. Cell formatting is presentation
data: IDs use `#` plus four digits; duration uses seconds below one minute and
`Xm SSs` at one minute or above. Status labels and tones follow the baseline
mapping: Running is `▸ Running`, Failed is error red, Paused is warning amber,
Queued is muted, and Done is secondary. Zero changes is muted.

The public `termrock-grid::Table` and `TableColumn` APIs are available through
the shared component commits through `a1b55a5eff2f3fde7b63e478379f76151f525532`.
The task consumer now keeps task data in source order, uses task IDs as stable
keys, formats all seven cells, declares baseline column sizing/alignment and
numeric display sorting, and delegates task rendering, input, focus, hit
testing, sorting, scrolling and activation to `Table`. Activation maps the
selected task ID to the existing page status response. The former private task
row painter and task horizontal scrollbar were removed. Reviewer and Checks
cards still use their existing private rendering and are outside this bounded
consumer change.

The Table consumer configures `Role::Danger` through the public per-table
`tone_styles` builder, mapping only the Failed task status to baseline RGB
`#f05d5d`. This leaves theme resolution and table focus/hover/selection
overlays in the shared component. The shared Table default is ActiveOnly sort
indicators, matching the frozen header state.

Focused integration tests now verify a real click on the Owner header changes
the shared table sort state and moves a task row, and Enter activation emits
the expected domain result. These are widget-harness checks; the pinned real
CLI paired workflows are still required for acceptance.

`cargo nextest run -p showcase-demos tables::tests --no-fail-fast` passes all
7 focused tests. Source order, IDs, cell values/tones, table column
configuration, exact Failed status RGB, Owner header sorting, and Enter
activation pass. The sort indicator default and per-table tone override come
from the shared Table API.

The pinned paired extension runner captured and replayed all six frozen
120x40 truecolor workflows (24 ordered receipts per workflow), but its candidate
pilot used the earlier candidate before this consumer integration. That
candidate differed in all six workflows; evidence is recorded in
`/Users/donbeave/Projects/termrock-live-extension-pair-20261020/live-paired/1792492720783767000-c06708be5d1e/report.json`.
It is not proof of the current integration. Current consumer parity remains
unverified and incomplete until a fresh candidate build passes paired frames.
