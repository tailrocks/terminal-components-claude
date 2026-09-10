# Holla design note

What `holla` is on this design system, why this interaction model won,
what was rejected, and how each CONCEPT.md product principle (§13) is met.
Companion notes: `00-decisions.md` (numbered decisions), `01-concept-constraints.md`,
`02-interaction-models.md` (the brainstorm), `03-recipe-audit.md`,
`05-visual-critique.md`. The binary lives in `src/bin/holla/`.

## The model in one paragraph

A tab is something that runs; a page is something you are deciding. The
permanent first tab, `Here`, is a finder: a live query row with a scope
readout, grouped fixed-column rows (`label · type · scope · reason`) and a
preview card that answers CONCEPT §7 in a fixed order. Flows push pages
inside the Here tab with a breadcrumb (`Here › Disk › Usage`,
`Here › Upgrade everything › Review`) and Esc pops them. Only activities
(dev servers, merged logs, monitors, one-shot commands with output) and
executing plans become tabs, so the strip shows exactly what is alive. The
shell is the family's: menu bar with the ` holla❯ ` lockup and the host
identity on the right, blank row, tab strip, body, status bar (path strong,
git state, discovery or page state in the centre, live facts on the right),
one hint bar.

## Why this model won

Five models were brainstormed (`02-interaction-models.md` §1): a palette
root with drawer flows, a persistent split root, a context board with a
palette overlay, a breadcrumb focus stack, and a TablePro-like tabbed
workspace. None satisfied the activity multiplexer (§8.14) and the plan
lifecycle (§8.15) together with a small root (§16):

- The palette is fast but has nowhere for running work to live and makes
  every flow a "close the modal, open a page" step.
- The split root is the right empty state but cannot hold two open flows
  or a running server while you search.
- The board makes scope legible by geometry but compensates for ranking
  with regions, which §16 forbids, and collapses at 80×24.
- The focus stack deepens flows perfectly (`d` then `u` is push, push) but
  pushed pages die on pop, so activities need a second structure anyway.
- The tabbed workspace multiplexes activities exactly right and would turn
  every decision into a tab that outlives its purpose.

The hybrid keeps the two strong halves and gives them one rule: running
work is a tab, deciding is a page. It reuses compositions the system
already proves (explorer + tabbed workspace, master/detail, searchable
list with a scope readout) and keeps the root small.

## Rejected alternatives worth recording

- Gate 1 as a `Dialog::facts`. Rejected because a 66-column dialog cannot
  hold a truthful full command sequence plus every affected resource class;
  Gate 1 is a page with a facts block and a framed, scrollable sequence, and
  Gate 2 is the facts dialog with the typed phrase (D-11). Both gates keep
  Cancel focused; the Enter that selected the row never reaches either.
- Kind letters in the picker glyph slot (`M G D …`). Rejected for a type
  word column: a letter is compact but not legible without a legend.
- Alt+arrows for scope. Rejected because the shared edit keymap owns
  `Alt+←→`; `Ctrl+↑/↓` walk the scope axis and `@parent @children @system
  @all` tokens give a deterministic text route.
- Explore as six rows. Rejected for one row of cells so the empty state
  fits 80×24 with Suggested, Recent, the parent section and Running.
- A plan graph drawn with connector lines. Rejected for a stage outline
  with `needs`, a lane letter and `join`: edges are read from the numbers,
  parallelism from the letters, convergence from the word, and it survives
  monochrome and 80 columns.

## Screens and their Esc ladders

| Surface | Communicates | Esc |
|---|---|---|
| Here (finder) | path and host in chrome; Suggested here with reasons; Recent here; From the parent root (nested children); Running; Explore cells and scope rows; preview | clear query › scope to here › back a page › quit (asks while work runs) |
| Domain page (`Here › Git`) | the same finder filtered to one group | clear › pop |
| Disk › Usage | filesystems with meters, largest-first entries as the scan streams, rebuildable artifacts by project with freshness-aware selection, system families, candidate facts | drawer › pop |
| Plan review | stage outline with inclusion, needs, lane, meta; consequence line; step facts; Cancel / Confirm plan (or Continue…) | pop |
| Review (Gate 1) | plain action, host, cwd, resource classes, inclusions, recoverable, privilege, provenance, uncertainty, sequence frame | pop |
| Trust | exact file, what it defines, environment effect, trust scope, provenance, definition body | pop |
| Arguments | fields with name, expected value, default, validation, masking; live command | revert field › pop |
| Snapshot | read-only facts and lines (git, system, processes, port, docker, compose, postgres blocking tree, mise, cleanup history, service unit, ssh -G) with follow-up buttons | pop |
| Activity tab | name, state, elapsed, exit, scope, cwd, host, insights, retained output, stream chips for merged logs, follow-ups when finished, attach/detach for monitors | detach › Here |
| Plan tab | aggregate progress, outline with live states and durations, per-step retained output, retry/skip/cancel, final summary | un-maximise › Here |

## Key grammar

Finder pages are typing-hot: every printable key edits the query. `↑↓`
move, `Enter` primary, `Alt+Enter` or right-click alternatives, `Tab`
finder › preview › tab strip, `Ctrl+↑/↓` scope, `Ctrl+G` activities,
`Alt+0` Here, `Alt+1–9` tabs, `Ctrl+W` close tab, `F10` menu, `F1` help,
`Ctrl+Q` quit. Letter-hot pages use the contextual letters the system
already owns: `s` stop, `f` follow, `p` facts drawer, `x` close, `u` undo,
`y` copy, `r` retry or restart, `a` all, `c` confirm or cleanup plan,
`Space` select, `Ctrl+]` detach. The hint bar declares the mode.

## Product principles (CONCEPT §13)

1. Here first. The catalogue tags every row with a direction and the
   effective directory; `Here` is a bias, not a filter, and local rows get
   a rank bonus. Nested children get a "From ‹root› · the parent root"
   section.
2. Intent before syntax. Search matches labels, keywords and intent phrases
   (`why disk full`, `checkout main`, `sync projects`, `service logs`); the
   exact command is in the preview and the gate, never required as input.
3. One root experience. One finder from any path; domains are pages of the
   same finder, not separate apps.
4. Useful before typing. Suggested here carries a reason on every row;
   first-use suggests disk analysis and system resources from live pressure.
5. Search across domains. `logs` returns a Compose log stream, a container
   log, a nearby log file and a journal side by side, each with type and
   scope.
6. Resources plus actions. A container, a database, a config file and an
   activity are rows with a primary action and alternatives.
7. Primary action plus discoverable alternatives. Enter runs; `Alt+Enter`
   lists run, copy, insert, arguments, the resource's alternatives, pin,
   alias, hide, reset and "why is this here?"; the preview lists them so
   the chord is not a secret.
8. Adaptive but predictable. Ranking tiers (pin › live state › context ›
   used here › used anywhere › default) sit below match quality; an exact
   alias always wins and crosses scope; `test` binds to the test task
   here, never to a fixed id.
9. Explain every recommendation. The row shows the strongest reason, the
   preview lists three, "Why is this here?" lists all with the ranking
   order and the note that risk never changes the rank.
10. Keep scope visible. `type · scope` never drops from a row; remote hosts
    write `on prod-eu-1`; the preview says where it runs and where it was
    defined; the status bar keeps the path; the menu bar keeps the host.
11. Feel immediate. Discovery streams by tick with a status-bar spinner and
    per-row freshness; selection is kept across rebuilds.
12. Make safety structural. Risk classes and confirmation levels are data on
    every row; destructive rows change only the hint (`Enter Review…`) and
    the gate, never the rank; two gates bind the phrase to the target and
    revalidate before executing (the docker plan drifts once on purpose).
13. Coordinate instead of recreating. `btm`, `pg_activity`, `lazygit` and
    `ssh` are handoffs that become attachable activities.
14. Zero configuration, optional mastery. Nothing is configured; aliases,
    pins and hides are offered from the alternatives menu.
15. Keyboard-first, not shortcut-secret. Every chord is in the hint bar or
    the key reference; the menu bar carries the same commands.
16. Local-first and privacy-conscious. Memory is per path and host in the
    fixture; secrets never reach a command line (`password never shown`).

## Conventions added to DESIGN.md on purpose

Finder rows, the query row with a scope readout, the stage outline, and
the two-gate pages are recorded under Composed patterns. No glyph gained a
second meaning; lane letters and `join` are words.

## Truth after an action

A finished run changes the fixture world it claimed to change, so the root
experience is honest after an action as well as before it: a cancelled
backend frees its waiters, a restarted unit is healed, a cleanup removes
its candidates and writes history, the Docker plan empties what each
succeeded step touched and a partial run leaves a partial world
(`domain/effect.rs`, `World::apply_effect`, `apply_plan_effects`).

## Known limits

- Attach mode is a simulation: keys are swallowed, `q` exits the program,
  `Ctrl+]` detaches. crossterm reports `Ctrl+]` as `Ctrl+5`, which is
  accepted too.
- The `apt` command strings for the Debian plan are not in the concept
  sources; they are the standard ones and flagged in `00-decisions.md`.
- Durations are virtual ticks (80 ms); estimates read in seconds so a plan
  finishes within a demo.
