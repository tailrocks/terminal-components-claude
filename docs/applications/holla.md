# Holla

Holla composes reusable Termrock components under the [component-only contract](../architecture/component-composition.md). Its ordinary consumer shape is [EX-18](../api/consumer-recipes.md#ex-18--holla-finder-and-activity-composition).

## Role

Holla is the context-adaptive composition reference. It exercises a finder,
preview and action flow together with plans, activities, streamed output,
file and disk views, trust and confirmation gates, and host-aware context.
It proves that Termrock components compose under changing context, nested
layers and long-running fixture phases.

The frozen visual and observable-interaction oracle is commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Preserve the Holla fixture
worlds, routes, keyboard and pointer behavior, virtual timing, literal
`holla❯` chrome, and every approved artifact under `snapshots/holla/` exactly.

## Current implementation

The current binary is `holla` with source under `src/bin/holla/`. The package
and library still use the current identifiers `junie-tui` and `junie_tui`.
These are implementation names scheduled for the future Termrock refactor;
the visible Holla label and fixture content are baseline data.

Holla is deterministic and in-memory. It does not spawn a host command, call
Docker/Git/provider services, launch a shell, or persist real account or
filesystem state. The simulated world records the effects needed to exercise
the UI and interaction contract. This application remains a conformance
consumer; it is not a product redevelopment scope.

## Run and inspect

```sh
# Empty-folder finder fixture.
cargo run --release --bin holla

# A named concept or parity fixture, frozen at a deterministic tick.
cargo run --release --bin holla -- \
  --scenario rust-dirty --motion paused --frame 40
cargo run --release --bin holla -- \
  --scenario upgrade-plan --motion paused --frame 60 --color 256

# List the current 34 scenario names and option values.
cargo run --release --bin holla -- --help

# Environment controls used by the current fixture runner.
HOLLA_NO_MOTION=1 cargo run --release --bin holla
HOLLA_NO_HISTORY=1 cargo run --release --bin holla
```

The 11 concept scenarios are:

```text
first-use       rust-dirty       monorepo-root     monorepo-child
docker-cleanup  disk-cleanup     upgrade-plan      activities-multi
remote-host     launch-failure   hard-cases
```

The 23 capability fixtures are:

```text
parity-discovery       parity-history          parity-files
parity-browser         parity-git-current      parity-git-batch
parity-task-sources    parity-cargo            parity-docker
parity-brew-services   parity-gradle           parity-idea
parity-upgrade-managers parity-executor        parity-task-input
parity-custom-actions  parity-disk-scan        parity-disk-navigation
parity-insights        parity-delete-safety    parity-cleanup-results
parity-platforms       parity-platforms-linux
```

`--motion` accepts `full`, `reduced`, and `paused`; `--frame N` selects a
fixture tick for paused captures. `HOLLA_NO_MOTION=1` selects reduced motion
when no explicit motion is supplied. `HOLLA_NO_HISTORY=1` disables usage
learning in the fixture world. The standard capture geometry is 72×20,
80×24, 100×30, 120×40 and 160×50, with truecolor, ANSI 256, ANSI 16,
explicit `none`, and `NO_COLOR` where a capture declares that path.

## Baseline scenarios and evidence

The approved snapshot taxonomy is:

```text
snapshots/holla/concept/   the 11 concept worlds
snapshots/holla/parity/    the 23 capability worlds
snapshots/holla/flows/     keyboard, gate, overlay and route journeys
snapshots/holla/fade/      scroll, output and boundary fade cases
snapshots/holla/audit/     representative multi-size/color audits
```

The preserved shell begins with the `holla❯` menu bar and host identity, then
a blank row, a two-row document-tab strip with permanent `Here`, a body with
one-cell margins, status bar, and hint bar. The Here tab hosts stacked decision
pages with a breadcrumb; activity tabs remain separate run/output surfaces.

The finder keeps its live query and clickable scope readout above fixed result
columns. Rows preserve label, type, scope, reason, and risk slots computed over
the whole result set; narrow layouts drop the reason first. At 110 columns the
preview card sits beside the rows. Below that, it becomes a drawer over rows
while focused and a one-line summary remains visible. Keep row ordering and
risk wording from the caller fixture.

When a fixture presents a broad destructive action, preserve its two existing
review steps: Gate 1 shows the target, scope, risk, reversibility, and full
sequence with Cancel focused; Gate 2 shows condensed facts and requires the
target-bound acknowledgement phrase. Execution re-resolves the target; drift
returns to Gate 1 with a Changed fact. These exact interactions are protected
reference behavior and do not expand the Termrock library into an executor.
The `docker-cleanup` oracle includes the literal `I UNDERSTAND: REMOVE ALL
DOCKER DATA ON devbox`; narrower actions keep their unprefixed target phrase.

Plan fixtures retain their current outline columns and consequence summary:
optional inclusion, ordinal, label, prerequisite ordinals, lane, and supplied
status/meta. The selected step facts appear beside the outline when space
allows and behind the existing `p` action when narrow. When execution ends,
the result cursor lands on the failed step or last step run and the existing
`Next` row remains present.

The shell key surface remains caller-owned. Finder uses type-to-search,
Up/Down, Enter, Alt+Enter, Tab for preview, Ctrl+Up/Down for scope, and the
`@parent`, `@children`, `@system`, and `@all` tokens. Global navigation keeps
Ctrl+G for activities, Alt+0 for Here, Alt+1–9 for tabs, Ctrl+W to close a tab,
F10 for menus, F1 for help, and Ctrl+Q to quit. The query editor keeps its
baseline select/undo/redo/word-delete/clear bindings. Page-specific activity,
file, disk, and plan bindings remain those in the frozen fixture traces.

The source evidence is:

- `src/bin/holla/app.rs`, `scenario.rs`, `screens/`, `domain/`, and `sim/` for
  shell ownership, pages, actions, plans, activities and fixture transitions;
- `tests/visual_baseline/holla.rs` for exact scenario, frame, geometry, color
  and interaction invocations;
- `src/bin/holla/README.md` for the current scenario walkthroughs;
- `snapshots/holla/` for immutable `.ansi`, `.txt`, `.png`, and `.html`
  output.

Run the Holla lane with:

```sh
cargo nextest run --run-ignored only \
  -E 'binary(visual_baseline) & test(holla_)'
```

Parity must retain the `Here` tab and activity/plan tabs, finder query and
scope semantics, preview and argument pages, trust and two-gate confirmation,
output retention/find/input/cancel behavior, nested modal ownership, pointer
capture, scroll/fade policy, resize recovery, host identity, and all typed
action targets. A candidate cannot discover its own hit coordinates and call
that parity; replay uses the frozen oracle coordinates and key sequences.

## Boundary

Future migration work may move Holla onto Termrock's shared Menu, Picker,
TextInput, TextViewport, Dialog, StatusBar, ScrollRegion, Tree and runtime
contracts. Preserve the current 34 fixture worlds, routes, output and
interactions. Do not add real command execution, provider integrations,
filesystem mutation, account services, new Holla behavior, or visual redesign.
