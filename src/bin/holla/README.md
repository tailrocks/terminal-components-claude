# holla — context-adaptive action launcher

"This folder, this host, right now": one finder that already knows what
matters here and never runs anything without showing you first.

Fully simulated, like `jackin_preview`: mise, git, gh, docker, brew,
gradle, btm, pg_activity, ssh and a virtual filesystem are deterministic
in-memory fixtures on a virtual clock (80 ms ticks). The real stack commands
are **never executed**; every runnable row carries an exact argument vector
(program, arguments, working directory, host), an outcomes table decides
what that command does to the world, an unmodeled command fails visibly,
and every effect of a finished run lands in the in-memory world (a cancelled
backend frees its waiters, a cleanup removes exactly what it claimed).

## Run

```sh
cargo run --release --bin holla                          # first-use scenario
cargo run --release --bin holla -- --scenario docker-cleanup
cargo run --release --bin holla -- --scenario hard-cases --color 16
cargo run --release --bin holla -- --motion paused --frame 40
```

Flags: `--scenario NAME` · `--motion full|reduced|paused` · `--frame N` ·
`--color truecolor|256|16|none`. `HOLLA_NO_MOTION=1` selects reduced motion.

Concept scenarios: `first-use` `rust-dirty` `monorepo-root`
`monorepo-child` `docker-cleanup` `disk-cleanup` `upgrade-plan`
`activities-multi` `remote-host` `launch-failure` `hard-cases`.

Parity scenarios (one per legacy capability slice, HP01–HP23 in
`docs/improvements/`): `parity-discovery` `parity-history` `parity-files`
`parity-browser` `parity-git-current` `parity-git-batch`
`parity-task-sources` `parity-cargo` `parity-docker` `parity-brew-services`
`parity-gradle` `parity-idea` `parity-upgrade-managers` `parity-executor`
`parity-task-input` `parity-custom-actions` `parity-disk-scan`
`parity-disk-navigation` `parity-insights` `parity-delete-safety`
`parity-cleanup-results` `parity-platforms` `parity-platforms-linux`.
`HOLLA_NO_HISTORY=1` disables usage learning: nothing is read or written.

## The model

A tab is something that runs; a page is something you are deciding. `Here`
is the permanent first tab: a query row with a scope readout, grouped rows
(`label · type · scope · reason`) and a preview that answers what will
happen, where, why it is suggested, what changes and which gate stands in
the way. Flows push pages inside Here with a breadcrumb; only activities
and executing plans become tabs. A committed cleanup is owned by the world,
not by the gate that started it: it runs one item per tick, its report page
fills in as it goes, and quitting waits for it to settle. The design note in
`docs/product/notes/04-design-note.md` records the model, the rejected
alternatives and how each CONCEPT §13 principle is met.

## Keys

`Type` search · `↑↓` move · `Enter` run (`Review…` when a gate stands in the
way) · `Alt+Enter` alternatives · `Tab` preview · `Ctrl+↑/↓` scope ·
`@parent @children @system @all` scope tokens · `Ctrl+G` activities ·
`Alt+0` Here · `Alt+1–9` tabs · `Ctrl+W` close tab · `F10` menu · `F1` help
· `Ctrl+Q` quit. Query editing: `Ctrl+A` selects the query (typing replaces
it), `Ctrl+Z` undo, `Ctrl+Y` redo, `Ctrl+Backspace` or `Alt+Backspace`
deletes a word, `Ctrl+U` clears. Letter-hot pages use `s` stop, `r` retry or
restart, `p` facts, `u` undo, `y` copy, `c` confirm, `Space` select,
`Ctrl+]` detach. Activity tabs: `i` types to the program's stdin (a prompt
shows `waiting for input`; `Enter` sends the line, `Ctrl+D` end of input,
`Ctrl+C` stop, `Esc` returns the keyboard), `/` finds in the output
(`Enter`/`↓` next, `↑` previous). Files pages: `g` jumps to a path,
`Ctrl+H` shows hidden entries, `Alt+Enter` lists the file actions, `/`
finds in the preview. Disk pages: `s` allocated/apparent, `f` folds noise,
`Space` selects with a parent dominating its descendants, `a` selects the
visible entries, `d` reviews the deletion, `r` rescans, `x` cancels, `t`
opens Spotlight top files. Cleanup pages: `Space` selects, `m` toggles
Trash/permanent, `n` toggles a dry run, `d` reviews.

## Verifying

Logic is test-covered; visuals need eyes. Three layers, cheap to thorough.

**1. Automated:**

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --bin holla        # domain, ranking, plans, concept journeys, parity journeys (HP01–HP23), row proofs, F23 proofs
cargo test --test terminal_suspend   # owned-pty job-control suspension (F21)
cargo test --test holla_pty          # fresh-process palette / NO_COLOR matrix and the bounded input flood
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
| `parity-*` | one scenario per capability slice; `src/bin/holla/app_tests_parity.rs` names the decisive rows and the tuisnap baseline captures every parity scenario at four sizes plus mono |

Cross-cutting, in any scenario: `F1` is the full key reference; `Alt+Enter`
on a row lists the alternatives (pin, alias, hide, why is this here?);
`--color none` must lose nothing but hue (warnings keep `▲`, errors a bold
`!`, focus keeps `▎`); `--motion paused --frame N` is byte-identical across
runs; resizing keeps type and scope on every row.

Which frozen legacy capture shows what (the `shots/h_*` corpus was captured
with the retired tmux harness and is now historical evidence):
`h_<scenario>_<WxH>.png` are the base frames,
`h_<scenario>_mono.png` the monochrome pass, `h_*_16.png` sixteen colours,
`h_flow_*.png` the journeys (trust → arguments, alternatives, help,
activities and merged logs, `btm` attached, the upgrade plan from exclusion
to failure and retry, the disk page and its gate, the remote restart gates,
the pg blocking tree, scope pages), `h_p3_docker_*.png` the Docker cleanup
from the root query through both gates, the drift revalidation, execution
and completion, and `h_hp<NN>_*.png` the parity flows (discovery, query
editing, find and browse, git, batches, task adapters, cargo, docker, brew
services, gradle and IDEA cleanups, upgrade managers, output streams, prompts
and cancellation, trust, disk scans and trees, insights, the deletion gates,
reports and the platform cases). Every frame has a `.manifest.json` (source
revision, binary digest, arguments, geometry, colour environment, tools and
fonts) and a `.png.fidelity.json` that says whether the raster is exact or
approximate and why; the `.txt` capture is authoritative for content.
`--color none` is a four-grey palette; actual `NO_COLOR` proof is the
fresh-process test in `tests/holla_pty.rs`, not a capture.

**3. Baseline matrix — the full visual pass:**

```sh
tools/tuisnap_baseline.sh                      # builds the binaries, captures the matrix into shots/tuisnap/
open shots/tuisnap/report.html                 # review every actual
tuisnap accept --store shots/tuisnap --all     # approve after review
tuisnap report --store shots/tuisnap           # re-verify: every approved frame must report matched
```

The holla slice is 194 captures: all 34 scenarios at four sizes plus mono,
256/16-colour and real-`NO_COLOR` spots, the 72×20 minimum-size boundary,
and 14 journeys (finder, files, browser, cleanup gates, upgrade plan,
remote gates, help, activities). The matrix and its rationale live in
`docs/baseline/tuisnap-coverage.md`; `tools/tuisnap_baseline.sh` is the
executable source of truth.

After changing one surface, rerun only what moved and look at the report:

```sh
APPS=holla ONLY='holla_hard-cases' SKIP_BUILD=1 tools/tuisnap_baseline.sh
```

**Simulation guarantee (adversarial):** walk every plan and gate while
watching `ps` for git, docker, ssh, mise, btm or pg_activity. Nothing
spawns; every effect is a mutation of `World`, and a finished run's effect
is asserted in `app_tests_flows.rs`.
