# holla — context-adaptive action launcher

"This folder, this host, right now": one finder that already knows what
matters here and never runs anything without showing you first.

Fully simulated, like `jackin_preview`: mise, git, gh, docker, btm,
pg_activity, ssh and the filesystem are deterministic in-memory fixtures on
a virtual clock (80 ms ticks). The real stack commands are **never
executed**; every effect of a finished run lands in the in-memory world
(a cancelled backend frees its waiters, a cleanup removes what it claimed).

## Run

```sh
cargo run --release --bin holla                          # first-use scenario
cargo run --release --bin holla -- --scenario docker-cleanup
cargo run --release --bin holla -- --scenario hard-cases --color 16
cargo run --release --bin holla -- --motion paused --frame 40
```

Flags: `--scenario NAME` · `--motion full|reduced|paused` · `--frame N` ·
`--color truecolor|256|16|none`. `HOLLA_NO_MOTION=1` selects reduced motion.

Scenarios: `first-use` `rust-dirty` `monorepo-root` `monorepo-child`
`docker-cleanup` `disk-cleanup` `upgrade-plan` `activities-multi`
`remote-host` `launch-failure` `hard-cases`.

## The model

A tab is something that runs; a page is something you are deciding. `Here`
is the permanent first tab: a query row with a scope readout, grouped rows
(`label · type · scope · reason`) and a preview that answers what will
happen, where, why it is suggested, what changes and which gate stands in
the way. Flows push pages inside Here with a breadcrumb; only activities
and executing plans become tabs. The design note in
`holla-project/notes/04-design-note.md` records the model, the rejected
alternatives and how each CONCEPT §13 principle is met.

## Keys

`Type` search · `↑↓` move · `Enter` run (`Review…` when a gate stands in the
way) · `Alt+Enter` alternatives · `Tab` preview · `Ctrl+↑/↓` scope ·
`@parent @children @system @all` scope tokens · `Ctrl+G` activities ·
`Alt+0` Here · `Alt+1–9` tabs · `Ctrl+W` close tab · `F10` menu · `F1` help
· `Ctrl+Q` quit. Letter-hot pages use `s` stop, `r` retry or restart, `p`
facts, `u` undo, `y` copy, `c` confirm, `Space` select, `Ctrl+]` detach.

## Verifying

Logic is test-covered; visuals need eyes. Three layers, cheap to thorough.

**1. Automated:**

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --bin holla        # 53 tests: domain, ranking, plans, journeys
```

**2. Interactive — one scenario at a time:**

| Scenario | What to verify |
|---|---|
| `first-use` | discovery streams by source; Suggested from live pressure; Explore cells |
| `rust-dirty` | `branch is 3 commits behind` leads; `Enter` on Pull runs an activity tab; status bar branch and state |
| `monorepo-root` | `pull` offers a plan across projects; primary branches resolved per repo, never `main` hard-coded |
| `monorepo-child` | `test` binds to the child's task; `From acme · the parent root` section; trust page before the untrusted task |
| `docker-cleanup` | typed query pre-filled; plan review, Gate 1, Gate 2 typed phrase; first execute drifts back to Gate 1 |
| `disk-cleanup` | scan streams largest-first; `active today` never selected; `c` builds the plan; two gates on the path |
| `upgrade-plan` | Space excludes and dependents recompute; mise upgrade fails; `r` retries; verification runs after |
| `activities-multi` | four tabs alive; merged logs with stream chips; `btm` attaches and `Ctrl+]` detaches; `Ctrl+G` picker |
| `remote-host` | `◆ prod-eu-1 · production` identity; `on prod-eu-1` scope words; restart phrase bound to the host |
| `launch-failure` | failed activity tab with honest output and `Next` follow-ups |
| `hard-cases` | detached and rebase states, unreadable children (partial), docker and github unavailable |

Cross-cutting, in any scenario: `F1` is the full key reference; `Alt+Enter`
on a row lists the alternatives (pin, alias, hide, why is this here?);
`--color none` must lose nothing but hue (warnings keep `▲`, errors a bold
`!`, focus keeps `▎`); `--motion paused --frame N` is byte-identical across
runs; resizing keeps type and scope on every row.

Which capture shows what: `h_<scenario>_<WxH>.png` are the base frames,
`h_<scenario>_mono.png` the monochrome pass, `h_*_16.png` sixteen colours,
`h_flow_*.png` the journeys (trust → arguments, alternatives, help,
activities and merged logs, `btm` attached, the upgrade plan from exclusion
to failure and retry, the disk page and its gate, the remote restart gates,
the pg blocking tree, scope pages) and `h_p3_docker_*.png` the Docker
cleanup from the root query through both gates, the drift revalidation,
execution and completion.

**3. Capture matrix — the full visual pass:**

```sh
python3 -m venv .venv && .venv/bin/pip install pillow   # once; PNG rendering
source tools/env.sh                                    # exports PY
cargo build --bin holla                                # captures run the debug binary
tools/holla_shots.sh          # 11 scenarios × 4 sizes + mono + 16 colours
tools/holla_flows.sh          # gates, plans, activities, snapshots, scopes
open shots/h_*.png
```

Captures need `tmux` on the path. Each frame lands as `.txt` (plain),
`.ansi`, `.html` and `.png`; the `.txt` is what the tests compare against.

After changing one surface, rerun only what moved and look at the PNGs:

```sh
SCENARIOS=hard-cases SIZES="80x24 120x40" COLORS=mono tools/holla_shots.sh
```

**Simulation guarantee (adversarial):** walk every plan and gate while
watching `ps` for git, docker, ssh, mise, btm or pg_activity. Nothing
spawns; every effect is a mutation of `World`, and a finished run's effect
is asserted in `app_tests_flows.rs`.
