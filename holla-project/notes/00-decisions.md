# Holla working notes: accepted decisions and open questions

Living log kept by the executor while building `holla`. Decisions are
numbered so later notes can cite them (D-n). Open questions are Q-n and are
closed in place when answered.

## Accepted decisions

- D-1 Recipe. `holla` copies the jackin-preview shell recipe (Application
  trait in main.rs, `App` with modal stack + focus/hit registries rebuilt per
  frame, `Screen` trait per surface with `Cx`/`Request`, virtual `Clock`,
  `Scenario` + `Motion` + `--frame`) and TablePro's `Dialog::facts` typed
  acknowledgement for the second gate.
- D-2 Simulation. Every stack system is an in-memory fixture under `domain/`
  with a `sim/` world that advances only on virtual ticks. No process is ever
  spawned; the binary imports nothing from `std::process`.
- D-3 Frame semantics. `--motion paused --frame N` builds the scenario world
  and advances it N ticks (80 ms each) through the scripted timeline, then
  freezes. Same N, same size, same picture.
- D-4 Chrome. Menu bar with ` holla❯ ` lockup (menus File · Go · Help), host
  identity right-aligned on the same row; blank row; a document tab strip
  (Root is the permanent first tab; every activity, plan and flow is a tab);
  body; status bar (path strong on the left, discovery in the centre, live
  facts on the right); hint bar last. Too-small notice below 72×20.
- D-5 Interaction model: the "Workbench" hybrid from
  `02-interaction-models.md`. Rule: a tab is something that runs; a page is
  something you are deciding. The permanent first tab `Here` holds a finder
  (query row, grouped fixed-column results, preview card); flows (Disk,
  Cleanup, Arguments, Trust, Plan review, Gate 1) push pages inside the
  Here tab with a breadcrumb (`Here › Disk › Usage`), Esc pops. Activities
  and executing plans are the only other tabs.
- D-6 Type column. Result rows read `label · type · scope · reason` in fixed
  columns computed over every row; `type` is the domain word (`task`,
  `git`, `docker`, `disk`, `system`, `postgres`, `ssh`, `file`, `plan`,
  `activity`). No glyph-table symbol gains a second meaning.
- D-7 Risk rendering. Risk is a word in the row's right column and in the
  preview: `read-only` faint, `mutating` muted, `destructive` in the error
  tone (canvas rows take the alarm red; menu rows keep the design system's
  rose). Nothing else about a destructive row changes: same rank, same
  keys, same discoverability.
- D-8 Scope. Scope directions form one vertical axis
  `Children ↓ · Here · ↑ Parent · ↑↑ System`. `Ctrl+↑`/`Ctrl+↓` move along
  it, the scope readout at the right end of the query row is clickable, the
  query tokens `@parent @children @system @all` set it deterministically,
  and Explore rows switch it with Enter. `Here` is a bias, not a filter:
  urgent or query-matched results from other scopes still appear, always
  with their scope word.
- D-9 Typing always searches on finder pages (typing-hot). Every printable
  key goes to the query, so finder chords are non-printable: `↑↓` move,
  `Enter` primary, `Alt+Enter` / right-click alternatives, `Tab` focus ring
  (finder › preview › tab strip), `Ctrl+↑↓` scope, `Ctrl+G` activity list,
  `Alt+0` Here, `Alt+1–9` tabs, `Ctrl+W` close tab, `Ctrl+Q` quit, `F10`
  menu, `F1` help, `Esc` ladder. Letter-hot pages (Disk, plans, activities)
  use the contextual letters `s f p x u y r` and `?`; the hint bar declares
  which mode a page is in.
- D-10 Esc ladder. Modal › menu › clear query › scope back to Here › pop
  page › (on a non-Here tab) back to Here › quit when nothing is running,
  otherwise the status says what is still running and `Ctrl+Q` asks.
- D-11 Two gates. Gate 1 is a pushed review page (`… › Review`): plain
  action, absolute path/host/cwd, resource classes with counts and sizes,
  inclusions, the full command sequence in a viewport, recoverable vs
  permanent, uncertainty, privilege, provenance, trust; `Cancel` focused,
  danger `Continue…`. Gate 2 is `Dialog::facts` with the condensed facts,
  six command lines, and the typed phrase; the danger button stays disabled
  until the phrase matches exactly. Broad actions prefix `I UNDERSTAND: `.
  Execute re-resolves; a changed target closes the dialog with a status
  line and re-renders Gate 1. Justification: the dialog cannot hold a
  truthful full sequence plus every resource class at 66 columns; the page
  can, and the dialog then binds intent to the target.
- D-12 Plans render as a stage outline: `[✓] NN label · needs · lane` rows
  with the step-rail state glyphs, a lane letter for parallel branches,
  `join` for convergence, a consequence sentence under the list after any
  exclusion, and the selected step's facts beside it. Confirming turns the
  review page into a plan tab (rail + per-step output).
- D-13 Activities are tabs backed by `TextViewport`; each keeps name, origin,
  scope, cwd, host, state, start, duration, exit, and actions. Multi-service
  logs are one viewport with bold service prefixes and a chip bar to show or
  hide a stream.
- D-14 Preview is a card (surface plane, no border) of `Prop` rows answering
  CONCEPT §7 in fixed order: What · Where · Runs in · Why · Changes ·
  Freshness · Confirmation, then the command lines. Below 100 columns it is
  a drawer that covers the results while it has focus.

- D-15 Snapshots (git status, system resources, pg blocking tree, ssh
  resolution) are pushed pages, not tabs: they are read, not run.
- D-16 Attach mode. Monitors (`btm`, `pg_activity`, `ssh`) are activities
  whose tab shows the tool's simulated screen; `Enter`/`i` attaches (keys go
  to the program, status bar shows an `attached` chip), `Ctrl+]` detaches.

- D-17 Nested children get a `From ‹root› · the parent root` section on the
  empty state with the root's ecosystem tasks (`dev`, `up`, `setup`) first,
  so parent contributions are visible without a scope change.
- D-18 `test` is a context-aware alias bound to the test task defined here,
  never to a fixed item id; sibling projects under the workspace root carry
  the scope word `sibling` and the Parent direction.
- D-19 Match quality outranks every learned signal when a query is typed:
  a prefix or substring match always beats a scattered subsequence; an
  exact alias crosses the scope filter.
- D-20 The preview splits beside the rows from a 110-column terminal; the
  reason column gives way first and the risk column never drops (revised
  after the visual critique, B2).
- D-21 Derived alternatives (dry runs, fetch, per-container restart and
  stop, `ssh -G` resolution, activity stop and restart, volume analysis,
  alias prompts) resolve in the shell without being rows of their own.

- D-22 Visual critique responses (`05-visual-critique.md`): spinners are a
  `StatusItem::busy` flag so every status spinner is primary; the risk
  word is a plain text-secondary column at rest; preview paths and
  commands never wrap (`truncate_middle`, `Runs` capped at six); the host
  identity in the menu bar is text-primary with the `◆` glyph, the git
  state after the branch name is the only warning-toned status item; mode
  labels are text-secondary; failure paints only the failed row and the
  `N failed` clause; the run lands on its result and offers `Next`
  follow-ups; the body keeps one blank row above the status bar; cards end
  after their content; the disk page drops Filesystems below 22 rows.
- D-23 Kept against the critique: the preview splits at 110 columns (at
  100 the reason column would starve, D-20); the plan outline's cursor row
  keeps `▎` + tint because it is the focused control (the finder's focus
  is its query row); `Uncertainty` and `Privilege` stay warning-toned on
  gates because they are safety facts; the attached chip is text-secondary.

## Open questions

- Q-1 Closed: the tab strip always shows `Here` so the model is visible from
  the first frame.
- Q-2 Closed: tmux `C-Up`/`C-Down` reach the app as Ctrl+arrows (see the
  `h_flow_scope_*` captures). `Ctrl+]` arrives as `Ctrl+5` through
  crossterm; both are accepted for detach.
- Q-3 Debian apt command strings are not in the sources; invented as
  `apt-get update`, `apt list --upgradable`, `apt-get upgrade -y`,
  `apt-get autoremove --purge` and flagged in the design note.
