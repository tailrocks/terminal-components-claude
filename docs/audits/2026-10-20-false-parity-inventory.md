# False parity inventory

Date: 2026-10-20. Branch `parity/termrock-visual-baseline` at `0b8f4e32b`.

## Pins

- Baseline tag object: `2718a6b8127a522a3d351d0b4bdb7542fc8da542`
- Baseline source: `4169783ae883788186d161af09ba9c481bd84e5c`
- Tuiscotti: `a1ea27f757d75d07b05499276cd932e9f8e067ad`
- `design/interaction-polish` local: `859e44e97dd39c42d203838c60bf47e9f7b5b24e`
- `origin/design/interaction-polish`: `f4867e9b5`
- `origin/main`: `a97e351df736b576904bc450619ca347ea64d65b`

## Decision

Production code that paints a captured frame is false parity. A matching image does not satisfy the library gate.

Remove these paths. Do not rename them into `termrock`. Replace each one with the shared component that owns that interaction. Snapshots stay in the visual harness only.

Do not add another `historical_*` paint or size-specific `*_plate` cell dump. Two uncommitted dumps were kept out of the branch:

- stash `wip-commit-dialog-stamp-not-landed` (tablepro commit dialog)
- stash `wip-terminal-100-cell-plate-not-landed` (showcase terminal 100×30)

## Counts on this commit

- `tablepro-ui`: 18 historical paints. 17 load `snapshots/**/120x40/truecolor.ansi`. `historical_form_new` is an inlined cell dump. Quick-open and tab list both belong to `termrock_overlays::picker::Picker` (`crates/termrock-overlays/src/picker.rs`).
- `jackin-preview-app`: 42 `historical_*::paint` call sites, 2 presentation blits, 2 boot-needle stamps that paint `jackin❯` over the title.
- `holla-sim` / `holla-ui`: 46 `*_plate` selectors plus 2 full-screen cell plates (`paint_disk_cleanup_plate`, `paint_launch_failure_plate`). Selectors key off one scenario, one size, and `now_ms()==80`.
- `showcase-demos`: selectable delete-confirm and bulk-mixed ANSI stamps. `showcase-ui` has none.

Shared selection on the tablepro and jackin rows is usually `120×40`, truecolor, and accent `Rgb(0x48, 0xe0, 0x54)`, often also `is_terminal || cfg!(test)`.

## First replacement

TablePro quick-open and tab list now draw `termrock_overlays::picker::Picker`. The `historical_picker_open` and `historical_tablist_open` replay paints are removed. Opening either overlay does not depend on the connection name, one empty tab, or one terminal size. The 120×40 truecolor frames are not identical yet.

Recorded before the uncommitted picker series: local `HEAD` `b0504717c280a4bb141cc6043b35fce6664f8a6a`. `origin/parity/termrock-visual-baseline` is ahead at `1a947a251ab9b045610b47729bf481419b9fca8d` (docs only). Rebase onto that tip before the next push. Do not push `main`.

Latest harness: `picker_open` and `tablist_open` at 120×40 truecolor both match the baseline, cell for cell, including the cursor. Supersedes the 1089 / 198 and 253 / 136 counts. The card samples backdrop ink, rows use the shared column solver, the overflowing list fades through `Ui::scroll_edges`, and the footer is the layer hint `Type Filter` while either picker is open. Other `historical_*` and `*_plate` paints are still in the tree. Do not treat the rest of the apps as parity.

## Still incomplete

- 1650 showcase ANSI files under flows, fade, audit, hover, and resize have no `ESC[?25` trailer. The pinned AnsiCursor dump always writes one. Matching those bytes means editing snapshots or the harness. Both stay forbidden.
- Two consecutive full `VISUAL_FULL` runs have not been run.
- This branch is not `origin/main`.
