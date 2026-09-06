# holla — context-adaptive action launcher

"this folder, this host, right now" — one root surface that already knows
what matters here, and never runs anything without showing you first.

Fully simulated, like `jackin-preview`: mise, git, gh, docker, btm,
pg_activity, ssh and the filesystem are deterministic in-memory fixtures.
The real stack commands are **never executed**. SSH identity files are
filenames only — key contents never enter fixtures, UI, history or plans.

## Run

```sh
cargo run --release --bin holla                      # first-use scenario
cargo run --release --bin holla -- --scenario remote-host
cargo run --release --bin holla -- --scenario hard-cases --color 256
cargo run --release --bin holla -- --motion paused --frame 4000
```

Flags: `--scenario NAME` · `--motion full|reduced|paused` · `--frame N` ·
`--color truecolor|256|16|none`. `HOLLA_NO_MOTION=1` selects reduced motion.

Scenarios: `first-use` `rust-dirty` `monorepo-root` `monorepo-child`
`docker-cleanup` `disk-cleanup` `upgrade-plan` `activities-multi`
`remote-host` `launch-failure` `hard-cases`.

## The model

- **One root experience.** Home is an always-armed query row over the
  priority stack: *Suggested here* (top-ranked four), *Recent here* (memory),
  *Explore* (everything discovered). No mode to learn; the query is armed
  from the first frame.
- **Context rings, cwd primary.** Every action carries a scope —
  Here → Project → Workspace → Host → Personal — rendered as an explicit tag
  on every nonlocal row. `Ctrl+S` cycles a scope filter; the current ring is
  always visible next to the query.
- **Capability ≠ action ≠ recommendation.** Discovery finds capabilities;
  the catalogue derives actions; ranking recommends. A reason rides on every
  row (`used 6 times in this project · alias gs`, `branch is 3 commits
  behind`), so a recommendation is always explainable.
- **Destructive is first-class.** Risk changes treatment, never
  availability. Read-only runs directly (simulated); Bounded previews first;
  Broad opens a plan: a reviewable DAG (Space excludes, dependents
  recalculate), then a typed phrase bound to the target host
  (`UPDATE ALL CHILD PROJECTS IN ~/work/monorepo`).
- **Activities survive navigation.** Long-running work becomes a named
  activity in the strip under the header — running/waiting/succeeded/failed/
  detached with per-state tones. `Ctrl+A` cycles them from anywhere, digits
  `1–9` jump on activity pages, `0` returns home. Multi-service log follows
  merge into one stream that keeps service identity.
- **Ranking is inspectable and yours.** `Ctrl+O` on any row: pin here, set
  an alias the query will match, hide here, reset ranking — each item states
  its why. Hidden rows resurface when you type their exact command (the only
  way back to Unhide/Reset).

## Keys

`Type` filter · `↑↓` results · `Enter` run top match · `Ctrl+P` preview ·
`Ctrl+O` alternatives · `Ctrl+S` scope · `Ctrl+A` activities · `?` help ·
`q`/`Ctrl+Q` quit (confirm; on production hosts the dialog names the remote
identity) · `F10` menu · mouse everywhere, never required.

## Verifying

Logic is test-covered; visuals need eyes. Three layers, cheap to thorough:

**1. Automated — logic and regression:**

```sh
cargo fmt --check                            # formatting
cargo clippy --all-targets -- -D warnings    # lint
cargo test                                   # every flow below, on TestBackend
```

**2. Interactive — one scenario at a time:**

```sh
cargo run --bin holla -- --scenario hard-cases
```

| Scenario | What to verify |
|---|---|
| `first-use` | discovery runs, empty state, footer hints |
| `rust-dirty` | type to filter, `↑↓` move, `Enter` runs top match (simulated), footer status confirms |
| `monorepo-root` | "Update all child projects" opens a plan DAG; gates ask for the typed phrase; run shows per-child ✓/✗ (billing diverges, legacy fast-forwards) |
| `monorepo-child` | cwd-aware ranking; workspace scope tags on rows |
| `docker-cleanup` | blocked rows state why; cleanup plan matches its apply effect |
| `disk-cleanup` | freshness labels: `active today` is never removed, `inactive N days` is reclaimable |
| `upgrade-plan` | remote host identity in the header; compound upgrade plan |
| `activities-multi` | strip `● … − ✗`; `Ctrl+A` cycles from anywhere; digits `1–9` open activity pages; `0` returns home |
| `remote-host` | ssh context; quit confirm names the remote identity |
| `launch-failure` | strip shows `✗ 1 seed db` from the first frame; a failing run reports honestly |
| `hard-cases` | detached HEAD, `▲ docker: discovery failed`, no-identity ssh host, long breadcrumb truncates before chrome drops |

Cross-cutting, in any scenario: `F1`/`?` help, `Ctrl+S` scope cycle, `F10`
menus, ranking controls via `Ctrl+O` (pin/alias/hide/reset — a hidden row
resurfaces only when you type its exact command), live window resize, and
`--color none` (mono must lose nothing but hue: warnings keep `▲`, errors a
bold `!`) and `--motion reduced`.

**3. Capture matrix — the full visual pass:**

```sh
tools/p6_matrix.sh           # 132 PNGs: 11 scenarios × 4 sizes × 3 color levels
open shots/h_p6_*.png        # names: h_p6_<scenario>_<size>_<color>
```

After changing a surface, rerun only what moved, then look at the PNGs —
rendered output is the evidence:

```sh
SCENARIOS=hard-cases SIZES="80x24 120x40" COLORS=mono tools/p6_matrix.sh
```

**Simulation guarantee (adversarial):** click through every plan and gate
while watching `ps` for git/docker/ssh/mise/btm/pg_activity. Nothing spawns;
every effect lands in the in-memory `World` and every run ends with a
`simulated` status.

## Evidence

`shots/h_*.png` — reviewed captures per phase (strip, activity pages, merged
logs, plan reviews and results, pg lock tree, ssh resolution, monitor
snapshot, alias prompt, hard-cases legibility matrix), plus the full
`h_p6_*` matrix: 11 scenarios × 4 sizes × 3 color levels.
`src/bin/holla/app_tests.rs` drives the real app on a TestBackend through
every flow above (270+ assertions).
