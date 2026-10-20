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

- `tablepro-ui`: 10 `mod historical_*` remain. 8 early-return paints load `snapshots/**/120x40/truecolor.ansi`. `historical_form_new` and `historical_form_advanced` are inlined cell dumps. Quick-open, tab list, and Safe Mode belong to `termrock_overlays::picker::Picker`. Help belongs to `termrock_overlays::help::HelpOverlay`. Delete connection and close-tab confirmation belong to `termrock_overlays::dialog::Dialog`. The filter editor is a live modal on shared overlay chrome and `TextInput`. A failed query paints from the statement error. `tablepro::query::error::120x40::truecolor` and `tablepro::table::data::120x40::truecolor` match.
- `jackin-preview-app`: 42 `historical_*::paint` call sites, 2 presentation blits, 2 boot-needle stamps that paint `jackin❯` over the title.
- `holla-sim` / `holla-ui`: 46 `*_plate` selectors plus 2 full-screen cell plates (`paint_disk_cleanup_plate`, `paint_launch_failure_plate`). Selectors key off one scenario, one size, and `now_ms()==80`.
- `showcase-demos`: selectable delete-confirm and bulk-mixed ANSI stamps. `showcase-ui` has none.

Shared selection on the tablepro and jackin rows is usually `120×40`, truecolor, and accent `Rgb(0x48, 0xe0, 0x54)`, often also `is_terminal || cfg!(test)`.

## First replacement

TablePro quick-open and tab list now draw `termrock_overlays::picker::Picker`. The `historical_picker_open` and `historical_tablist_open` replay paints are removed. Opening either overlay does not depend on the connection name, one empty tab, or one terminal size. The 120×40 truecolor frames are not identical yet.

Recorded before the uncommitted picker series: local `HEAD` `b0504717c280a4bb141cc6043b35fce6664f8a6a`. `origin/parity/termrock-visual-baseline` is ahead at `1a947a251ab9b045610b47729bf481419b9fca8d` (docs only). Rebase onto that tip before the next push. Do not push `main`.

Latest harness: `picker_open`, `tablist_open`, and `safemode_picker` at 120×40 truecolor match the baseline, cell for cell, including the cursor. Supersedes the 1089 / 198 and 253 / 136 counts. The card samples backdrop ink, rows use the shared column solver, the overflowing list fades through `Ui::scroll_edges`, and a searchable picker footer is `Type Filter`. Safe Mode is the same picker with search off, size `Fixed(112, rows + 5)`, footer `↑↓ Move  Enter Choose  Esc Cancel`, and a hidden cursor. `historical_safemode_picker` is removed. Opening it does not depend on the connection name, one empty tab, or one terminal size. `?` opens `HelpOverlay` at 80% of the screen, centered. `historical_help_overlay` is removed. The 120×40 truecolor help frame matches, including the hidden cursor. `d` on a connection opens `Dialog::destructive` titled `Delete connection?`, width 54, geometrically centered in the body. `historical_delete_dialog` is removed. The 120×40 truecolor delete frame matches, including the hidden cursor. The same code also matches connections default, help, Safe Mode, quick-open, and tab list at 120×40 truecolor. `ctrl-w` on a dirty query opens the same dialog, titled `Close tab with unsaved work?`, with action `Close anyway`. `historical_close_confirm` is removed. The 120×40 truecolor close-confirm and quit-confirm frames match, including the hidden cursor. Other `historical_*` and `*_plate` paints are still in the tree. Do not treat the rest of the apps as parity.

## Review of `a848219fd` (picker frames)

Disposition against the passing 120×40 truecolor frames `picker_open`, `tablist_open`, and `safemode_picker`. Snapshot footers include `✓ Connected to Production`. Do not revert a green cell to chase a misread oracle.

- Footer suffix after `Type Filter`: kept. The oracle row is that hint plus the connection notice.
- Selected Fill row uses highlight `#2f5aa8`: kept. That is the oracle cursor row.
- Picker card samples backdrop ink: kept. Empty overlay cells otherwise drop ghost `#262626`.
- Muted `#8a8a8a`, thumb `#808080`, track `┆` / `#474747`: kept for these frames.
- `without_bold` on the scrollbar: kept for truecolor. A mono pressed thumb that must stay bold is still open. Strip only bold inherited from the cell underneath, and keep bold the style itself set, when a mono frame shows the loss.
- `surface_already_opaque` skips any non-reset background, and `scroll_edges` fades every overflowing `FilterList`. Both are open. Narrow them when a `Select` frame shows the extra fade or a skipped fill. Do not change them while the three picker frames are the proof.
- The item-row hash is a fixture, not the oracle. The visual frames are.

## Still incomplete

- 1650 showcase ANSI files under flows, fade, audit, hover, and resize have no `ESC[?25` trailer. The pinned AnsiCursor dump always writes one. Matching those bytes means editing snapshots or the harness. Both stay forbidden.
- Two consecutive full `VISUAL_FULL` runs have not been run.
- This branch is not `origin/main`.
