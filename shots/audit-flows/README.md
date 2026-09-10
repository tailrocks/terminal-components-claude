# Interaction capture evidence

Regenerate from current sources with `PY=/path/to/python-with-pillow tools/audit_flows.sh`.
The script builds `showcase` and `tablepro`, uses a private tmux socket, and records
ANSI, text, cursor metadata, HTML, and PNG. It modifies no baselines.

The 51 capture sets cover:

| Flow | Sizes | Colour modes | Evidence |
|---|---|---|---|
| Inputs: Tab, Enter, Ctrl+L selects text while editing | 80×24, 120×40, 160×50 | truecolor, `--color none`, actual `NO_COLOR=1` | `inputs_selected_*` |
| Forms: Ctrl+S rejects empty required name and restores focus | Same | Same | `forms_invalid_*` |
| Diff: toggle Review, drag actual text with mouse, release and copy, toggle Empty | Same | Same | `diff_review_*`, `diff_drag_*`, `diff_empty_*` |
| TablePro Facts: open gate, type required acknowledgment, focus Execute, complete query | 120×40 | truecolor, actual `NO_COLOR=1` | `tablepro_ack_gate_*`, `tablepro_ack_armed_*`, `tablepro_ack_executed_*` |

TablePro's “Production” connection is an in-memory fixture with no database drivers.
The script checks resulting text and reverse-video ANSI attributes for actual
`NO_COLOR` selections; these captures exercise terminal event routing, not injected
application states. `AUDIT_FLOW_SIZES` and `AUDIT_FLOW_COLORS` select showcase
subsets; TablePro's two colour variants still run.

Representative reviewed frames:

- [Small invalid form](forms_invalid_80x24_truecolor.png)
- [Colourless text selection](inputs_selected_120x40_no_color.png)
- [Small diff mouse selection](diff_drag_80x24_truecolor.png)
- [Wide colourless diff selection](diff_drag_160x50_no_color.png)
- [Empty diff](diff_empty_120x40_no_color.png)
- [Acknowledgment armed](tablepro_ack_armed_120x40_no_color.png)
- [Acknowledged fixture query completed](tablepro_ack_executed_120x40_truecolor.png)

Rasterized PNGs use the existing capture font, which lacks some CJK and emoji
glyphs. The matching ANSI/text/HTML retain `東京`, combining accents, and `☕`;
use those artifacts when assessing Unicode data rather than font fallback.
