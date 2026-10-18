# Jackin Preview

Jackin Preview composes reusable Termrock components under the [component-only contract](../architecture/component-composition.md). Its ordinary consumer shape is [EX-16](../api/consumer-recipes.md#ex-16--jackin-workspace-prelude-composition).

## Role

Jackin Preview is the conformance consumer for a large composed host-control
surface. It exercises host management, account and usage views, workspace
editing, launch progress, overlays, settings, and a terminal-pane composition
with tabs and splits. It is a deterministic preview fixture, not a service
implementation.

The frozen visual and observable-interaction oracle is commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Preserve every route, fixture
world, virtual-clock phase, keyboard and pointer path, and approved artifact
under `snapshots/jackin/` exactly.

## Current implementation

The binary name is `jackin-preview`, with source under
`src/bin/jackin_preview/`. The current package/library identifiers remain
`junie-tui` and `junie_tui`; the source module name `jackin_preview` is also a
current implementation identifier. The visible `jackin❯` lockup is a literal
baseline string. Future Termrock library renaming must not alter it.

The current route set is:

```text
Intro → Manager → Prelude → Editor → Settings → Accounts → Usage
       → Cockpit → Handoff → Capsule → Outro
```

The fixture world is entirely simulated. It does not contact the real Jackin
CLI, providers, account services, containers, 1Password, a daemon, or a live
PTY. Its virtual clock and motion modes exist to make the reference frames
repeatable. This documentation does not authorize adding any of those
services.

## Run and inspect

```sh
# First-use intro and manager fixture.
cargo run --release --bin jackin-preview

# Select a fixture world and freeze a deterministic phase.
cargo run --release --bin jackin-preview -- \
  --scenario accounts-mixed --motion paused --frame 40
cargo run --release --bin jackin-preview -- \
  --scenario launch-failure --motion paused --frame 240

# Motion and color options.
cargo run --release --bin jackin-preview -- \
  --scenario launch-running --motion reduced --color 256
JACKIN_NO_MOTION=1 cargo run --release --bin jackin-preview
cargo run --release --bin jackin-preview -- --help
```

Current scenario names and fixture meanings:

| Scenario | Reference state |
| --- | --- |
| `first-use` | Intro followed by an empty manager |
| `returning` | Existing running instance and populated manager |
| `accounts-mixed` | Multiple providers/accounts with mixed health, entering Accounts |
| `launch-running` | Active launch cockpit |
| `launch-failure` | Launch stage failure and failure dialog |
| `capsule-multi` | Attached Capsule with several tabs and nested panes |
| `outro-last` | Last running instance and exit/outro ritual |
| `hard-cases` | Long labels, missing daemon data, discovery failure and many rows |

`--motion` accepts `full`, `reduced`, and `paused`; `--frame N` selects a
fixture tick for paused captures. `JACKIN_NO_MOTION=1` selects reduced motion
when no explicit motion is supplied. The standard geometry matrix is 72×20,
80×24, 100×30, 120×40 and 160×50, with the declared truecolor, 256, 16,
explicit `none`, and `NO_COLOR` paths where present.

## Preserved shell and motion composition

The first-use and outro scenarios include the existing seeded Intro/Outro
ritual. Its virtual animation uses 33 ms ticks; phrase typing and the starfield
warp are pure functions of the fixture tick and seed. Preserve the existing
phase transitions and the pinned frame values used by the capture programs
(including intro boundary frames 300/400 and the launch-failure frame 240).
Reduced motion and paused motion have distinct output; `Enter`/`Esc` skip the
same baseline ritual stages, and a paused frame remains reproducible. These
facts describe reference output only; they do not create a product feature
plan.

The settings, accounts, usage, launch, and Capsule scenes also preserve their
existing supplied registry/policy, effective-account, launch-stage, session,
tab, split, and terminal-pane compositions. All service and terminal data stay
in their current deterministic fixture worlds.

## Baseline scenarios and evidence

The approved store covers intro, manager, prelude/editor, settings, accounts,
usage, cockpit, and Capsule compositions, plus audit fixtures for
`accounts-mixed` and `capsule-multi`. Capture programs live in
`tests/visual_baseline/jackin.rs`; approved artifacts live in
`snapshots/jackin/`.

The source evidence is split between:

- `src/bin/jackin_preview/app.rs`, `scenario.rs`, `screens/`, `domain/`, and
  `sim/` for routes, fixture data, virtual time, overlays, focus and input;
- `tests/visual_baseline/jackin.rs` for exact scenario/frame/size/color
  invocations and interaction checkpoints;
- `snapshots/jackin/` for the immutable grouped output.

Run the Jackin Preview lane with:

```sh
cargo nextest run --run-ignored only \
  -E 'binary(visual_baseline) & test(jackin_)'
```

Parity must retain the current route and layer semantics: focus trapping and
restoration in dialogs, menu and picker ownership, completed click versus
canceled release, terminal-pane selection/copy and tab behavior, scroll and
split geometry, virtual-clock transitions, reduced/paused motion, secrets
masked in every artifact, and the exact `jackin❯` chrome.

## Boundary

Future migration work may replace the current internal widgets with Termrock
components and shared runtime mechanisms. Preserve this preview's fixture
worlds and observable behavior while doing so. Do not add real host
management, authorization, accounts, providers, container launch, terminal
daemon, or new product flows; do not redesign Jackin Preview.
