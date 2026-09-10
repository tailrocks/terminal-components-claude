#!/usr/bin/env bash
# Reproducible interaction evidence; no baselines are modified.
# Requires tmux, rg, and PY pointing to Python with Pillow (capture.sh).
# Example: PY=/tmp/holla-venv/bin/python tools/audit_flows.sh
# Optional subsets: AUDIT_FLOW_SIZES="120x40" AUDIT_FLOW_COLORS="no_color".
# TablePro's Production connection is an in-memory fixture, with no drivers.
set -euo pipefail
cd "$(dirname "$0")/.."

export PY=${PY:-python3}
export SHOT_DIR=${SHOT_DIR:-shots/audit-flows}
export CAPTURE_SOCKET=${CAPTURE_SOCKET:-junie-audit-flows}
export CAPTURE_SESSION=${CAPTURE_SESSION:-flows}
"$PY" -B -c 'import PIL'
cargo build --bin showcase --bin tablepro
trap 'tools/capture.sh stop' EXIT

keys() { tools/capture.sh keys "$@"; }
shot() { tools/capture.sh shot "$1"; }
require_text() {
    if ! rg -Fq -- "$2" "$SHOT_DIR/$1.txt"; then
        echo "expected '$2' in $SHOT_DIR/$1.txt" >&2
        exit 1
    fi
}
require_reverse() {
    # A buffer-style assertion cannot prove real NO_COLOR backend output.
    python3 -B - "$SHOT_DIR/$1.ansi" <<'PY'
import pathlib, re, sys
ansi = pathlib.Path(sys.argv[1]).read_text()
assert any("7" in match.split(";") for match in re.findall(r"\x1b\[([0-9;]*)m", ansi)), "selection lost reverse video"
PY
}
start() {
    export BIN=$1
    ARGS="$2 $color_args" tools/capture.sh start "$cols" "$rows"
}
set_color() {
    export PRESERVE_NO_COLOR=0
    export NO_COLOR=
    case "$color" in
        truecolor) color_args="--color truecolor" ;;
        none) color_args="--color none" ;;
        no_color)
            color_args=""
            export PRESERVE_NO_COLOR=1 NO_COLOR=1
            ;;
        *) echo "unknown audit colour: $color" >&2; exit 1 ;;
    esac
}

for size in ${AUDIT_FLOW_SIZES:-80x24 120x40 160x50}; do
    cols=${size%x*}
    rows=${size#*x}
    for color in ${AUDIT_FLOW_COLORS:-truecolor none no_color}; do
        set_color
        suffix="${size}_${color}"

        start target/debug/showcase "--page inputs"
        # Each key waits for the next render/focus reconciliation.
        keys Tab Enter C-l
        name="inputs_selected_$suffix"
        shot "$name"
        require_text "$name" "EDIT"
        require_text "$name" "payments-gateway"
        if [ "$color" = no_color ]; then
            require_reverse "$name"
        fi

        start target/debug/showcase "--page forms"
        keys Tab C-s
        name="forms_invalid_$suffix"
        shot "$name"
        require_text "$name" "Required"

        start target/debug/showcase "--page diff"
        keys Tab Enter
        name="diff_review_$suffix"
        shot "$name"
        require_text "$name" "● Review"
        require_text "$name" "attempts = 3"

        # Locate a fixture run in the actual rendered pane. Prefix cells on
        # this ASCII fixture row all occupy one terminal column. Coordinates
        # are converted to SGR mouse's one-based convention.
        read -r drag_x drag_y < <(python3 -B - "$SHOT_DIR/$name.txt" <<'PY'
import pathlib, sys
for y, line in enumerate(pathlib.Path(sys.argv[1]).read_text().splitlines()):
    x = line.find("attempts = 3")
    if x >= 0:
        print(x + 1, y + 1)
        break
else:
    raise SystemExit("diff fixture line missing")
PY
        )
        tools/capture.sh mouse "$drag_x" "$drag_y" down
        tools/capture.sh mouse "$((drag_x + 10))" "$drag_y" drag
        tools/capture.sh mouse "$((drag_x + 10))" "$drag_y" up
        keys y
        name="diff_drag_$suffix"
        shot "$name"
        require_text "$name" "Selection copied in demo"
        if [ "$color" = no_color ]; then
            require_reverse "$name"
        fi

        # Mouse selection focuses the viewport; the preceding stop is Empty.
        keys BTab Enter
        name="diff_empty_$suffix"
        shot "$name"
        require_text "$name" "No file selected"
        tools/capture.sh stop
    done
done

# The acknowledgment interaction is independently regression-tested at all
# supported geometry boundaries. Persist its full fixture flow at normal size
# in both truecolor and actual NO_COLOR to show editable and armed states.
cols=120
rows=40
for color in truecolor no_color; do
    set_color
    suffix="120x40_$color"
    start target/debug/tablepro "--connect Production"
    keys Tab i "UPDATE orders SET status = 'paid' WHERE id = 'x'" Escape C-r
    name="tablepro_ack_gate_$suffix"
    shot "$name"
    require_text "$name" "Type orders to confirm"
    keys Enter orders Enter Right
    name="tablepro_ack_armed_$suffix"
    shot "$name"
    require_text "$name" "orders"
    require_text "$name" "Execute"
    keys Enter
    sleep 1
    name="tablepro_ack_executed_$suffix"
    shot "$name"
    require_text "$name" "rows affected"
    tools/capture.sh stop
done
echo "Interaction captures verified under $SHOT_DIR"
