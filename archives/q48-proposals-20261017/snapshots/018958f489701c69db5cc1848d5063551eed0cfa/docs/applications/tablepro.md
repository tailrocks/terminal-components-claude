# TablePro

TablePro composes reusable Termrock components under the [component-only contract](../architecture/component-composition.md). Its ordinary consumer shape is [EX-17](../api/consumer-recipes.md#ex-17--tablepro-workbench-composition).

## Role

TablePro is the complex workbench conformance consumer. It exercises Grid and
table-row behavior together with CodeEditor/TextInput, overlays, selection,
forms, tabs, scrolling, completion, status and safety feedback. Its
deterministic in-memory catalog makes the full workbench reproducible without
a database driver or network service.

The frozen visual and observable-interaction oracle is commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Preserve the TablePro fixture
world, keyboard and pointer routes, all modal behavior, and every approved
artifact under `snapshots/tablepro/` exactly.

## Current implementation

The current binary is `tablepro` (`src/bin/tablepro/main.rs`) with the
workbench split across `app.rs`, `connections.rs`, `workbench.rs`, `model.rs`,
`tabs.rs`, `sql.rs`, and `db.rs`. The package is still `junie-tui` and the
library import path is still `junie_tui`; these are current legacy source
identifiers, not the target Termrock identity. The visible `TablePro` product
label remains part of the oracle.

The preserved composition contains:

- connections, grouped connection details, filtering, duplication, deletion,
  and a new connection form with basic and advanced fields;
- explorer navigation for databases, schemas, tables, views, functions,
  sequences, columns, indexes and keys;
- workbench tabs for table Data/Structure, SQL query and History, including
  tab creation, close confirmation, switching, and quick switching;
- typed table cells, sorting, filters, horizontal and vertical scrolling,
  cell editing, row selection, inserts, deletes, duplication and pending
  changes;
- query editing, completion, results, errors, EXPLAIN and EXPLAIN ANALYZE;
- Safe Mode levels, read-only refusal, write/destructive confirmation, typed
  acknowledgement, help, tab-list, history and picker overlays.

These are fixture surfaces for the library. They do not authorize a real
database backend or changes to the TablePro product.

## Run and inspect

```sh
# Connections screen.
cargo run --release --bin tablepro

# Deterministic production fixture, already in the workbench.
cargo run --release --bin tablepro -- --connect Production

# Explicit color paths and CLI help.
cargo run --release --bin tablepro -- --color 256
cargo run --release --bin tablepro -- --color none
cargo run --release --bin tablepro -- --help
```

`--color` accepts `truecolor`, `256`, `16`, and `none` (plus the current
aliases). The workbench has a 72×20 minimum and is captured at 72×20, 80×24,
100×30, 120×40, and 160×50. The approved store also records declared color
variants and interaction states; inspect the exact inventory in
`tests/visual_baseline/tablepro.rs` and `snapshots/tablepro/`.

## Preserved layout and binding behavior

The shell keeps a one-row identity strip, a two-row tab strip, then a body with
one-cell side margins. The workbench body is a framed explorer at one quarter
of body width, clamped to 28–40 cells, a one-cell gap, and a framed tab body.
Below 100 columns the explorer becomes a drawer over the body while focused;
`0` opens it, and Tab or opening an object puts it away. The connections screen
uses a list at one third width, clamped to 26–40 cells, with a two-cell gap and
a detail/form card; below 80 columns it shows only the list. In the query tab,
the editor/results split starts at 38%, with minima of four and six rows and a
blank row between. `Ctrl+↑/↓` resizes it and `z` maximizes the focused half.

Preserve the current workbench chords: `Ctrl+R`/`F5` runs the statement at the
cursor, `Alt+R` runs all, `Ctrl+X`/`Alt+X` explains, `Ctrl+T` creates a tab,
`Ctrl+W` closes it, `Ctrl+O`/`Ctrl+P` opens quickly, `Ctrl+G` opens the tab
list, `Ctrl+Y` opens history, `Ctrl+B` toggles the explorer, `Ctrl+L` changes
Safe Mode, `Ctrl+D` switches Data/Structure, `Ctrl+S` saves pending changes,
and `Ctrl+F` finds or filters. `Alt+D` duplicates a grid row so the app's
`Ctrl+D` retains its Data/Structure meaning. The connections screen uses
`Ctrl+N` for a new connection and `/` to filter.

Escape unwinds the existing local sequence: cancel a running query,
unmaximize, move to the tab strip, then move to the explorer and clear its
filter on arrival. `q` quits; `Ctrl+C` first cancels running work and then
asks when work is unsaved. These are baseline application bindings, not a
Termrock product roadmap.

The Data grid uses the exact shared database-grid recipe in the [Grid
contract](../components/grid.md). The application-owned two-row pending bar
shows count and operation breakdown with Preview SQL, Discard, and Save stops;
a rejected row edit replaces the breakdown with its reason. Preserve the
existing composition and supplied fixture data.

## Baseline scenarios and evidence

The baseline capture groups cover the following scenario families:

```text
connections  default, form creation/editing, advanced fields, filtering,
             production detail, deletion and duplication
table        default, sorting/filtering, cell editing, dirty rows,
             duplication, filter editor and horizontal scroll
query        completion, column completion, results, error, EXPLAIN,
             EXPLAIN ANALYZE, new tab and close confirmation
overlays     help, history, Open Quickly, Safe Mode, tab list and table list
ack          gate, armed, delete gate and executed safety transitions
workbench    composed explorer/tab/body/footer and resize behavior
```

The authoritative evidence locations are:

- `src/bin/tablepro/app.rs` and the screen/workbench modules — route, focus,
  hit, modal, editing and deterministic catalog behavior;
- `tests/visual_baseline/tablepro.rs` — executable capture programs and
  needles for the scenario families above;
- `snapshots/tablepro/` — approved grouped output, including audit, table,
  query, overlay, acknowledgement and resize/fade captures.

Run the TablePro lane with:

```sh
cargo nextest run --run-ignored only \
  -E 'binary(visual_baseline) & test(tablepro_)'
```

Parity must compare the visible grid with the same model used for hit testing
and updates. Preserve selected stable row/column keys through sort, filter,
insert, delete and source refresh. Preserve draft/committed cell values,
typed actions, editor cursor, completion ownership, scroll offsets, modal
focus restoration, safety target text, and all pointer down/up/capture cases.

## Boundary

Future work may migrate TablePro onto Termrock's consolidated Grid, Menu,
Picker, shared text editing, ScrollRegion, Dialog, and StatusBar mechanisms.
It must preserve the exact current application, fixture data, routes, output
and interactions. No real SQL engine, provider integration, database backend,
new workbench flow, or visual redesign belongs in this refactor.
