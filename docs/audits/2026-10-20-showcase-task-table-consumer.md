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

The public `termrock-grid::Table` and `TableColumn` API is available in
component commit `caf8eb20ff0dd706debc1c7f1681440eea36960a`. The consumer model
now keeps task data in source order, uses task IDs as stable keys, formats all
seven cells, and maps baseline tones. A focused nextest verifies source order,
IDs, cell text, and all status tones. Component stage two is adding visible-row
range and status metadata accessors needed by the frozen task card. The consumer
must delegate layout, sorting, scrolling, focus, hit testing, and activation to
`Table`. Activation must map the selected task's domain ID into the existing
page status response. Existing private geometry and painting remain in place
until paired real-CLI frames prove the public component renders the same cells
and cursor state.

Current gap: `TablesPage::update` ignores input, and its task table paints
coordinates and row order privately. Existing in-process tests cover static
content and private geometry only. They do not prove header sorting, keyboard
navigation, pointer hit regions, scrolling, or baseline frame parity. No parity
claim is made by this audit.
