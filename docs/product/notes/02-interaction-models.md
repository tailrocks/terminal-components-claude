# 02 · Interaction models for Holla

Brainstorm of root interaction models, evaluation, one chosen direction, and
the screen inventory + key grammar derived from it. Sketches are design
proposals at 100×30 on the Junie TUI system (DESIGN.md), not CONCEPT examples.

Shorthand: *typing-hot* page = letters go to a live query; *letter-hot* page =
letters are contextual verbs (`s f p x u y`). A page is never both.

---

## 1. Candidate models

### M1 · Palette root, drawer flows
Root is the Picker (centred, upper third) over an otherwise empty canvas. Flows
(Disk, Cleanup, Plan) open as drawers or full pages behind it; activities live
in a second picker (`Ctrl+G`).

- Empty state: weak. A modal picker has ~10 rows; Suggested/Recent/Explore/
  Running fight for them. Path/host live only in the picker title.
- Scope: readout in the picker title (`scope › here`), `Tab` cycles. OK, but
  scope hides when the picker closes.
- Progressive disclosure: every flow is "close palette, open page". Returning
  to root re-opens the modal; the page behind it goes dim. Two-layer feel.
- Two-gate: fine (Dialog::facts).
- Activities: no persistent strip. Switching = picker every time. Fails
  "fast switching, survive navigation" (§8.14) on feel, not on function.
- 80×24: best of all models (modal is small). 160×50: canvas mostly empty.
- Green discipline: easy. Keyboard destination: easy.
- Verdict: fast launcher, poor workbench. Contradicts §8.14/§8.15 weight.

### M2 · Persistent split root, flows replace body
Body = results list (left) + preview card (right), always. Flows replace the
whole body; `Esc` returns to the split.

- Empty state: strong. Sections + preview of the first suggestion answer §7
  before any key.
- Scope: readout on the query row; nonlocal rows carry a scope column.
- Disclosure: good for one flow at a time. Two open flows are impossible
  without a stack or tabs.
- Two-gate: fine.
- Activities: nowhere to live. Needs a bolt-on (picker or hidden list); output
  of a running task is invisible while you search.
- 80×24: preview must become a drawer (<100 cols, TablePro rule).
- Verdict: right root, no multiplexer.

### M3 · Context board (rings as regions) + palette overlay
Full-screen board: Here / Project / Workspace / Host quadrants each listing
their actions; `/` or typing opens a palette overlay.

- Empty state: visually rich, cognitively loud. Contradicts §16 "small result
  sets", §6.2 "not a catalog". Weak ranking would be masked by regions.
- Scope: excellent legibility (region = scope) — its only clear win.
- Search: overlay must flatten regions into one list anyway, so the board's
  model does not survive into search. Two mental models (§15).
- 80×24: four regions collapse to a single stacked column = M4 with worse
  ranking. 160×50: the one size it is designed for.
- Activities: another region. Now five.
- Verdict: reject. Layout compensates for ranking; exactly what §16 forbids.

### M4 · Focus stack, breadcrumb single column
Everything is a pushed page: root → Disk → Usage → Cleanup → Gate 1. One
column, breadcrumb on top, `Esc` pops.

- Empty state: strong (whole width for sections + inline reason lines).
- Scope: a breadcrumb crumb (`here › parent monorepo`). Clean.
- Disclosure: perfect. `d` then `u` is literally push, push.
- Two-gate: Gate 1 is a page, Gate 2 a dialog. Natural.
- Preview: no room beside the list; must be an inline expansion under the
  focused row or a page (`p`). Slower to compare candidates.
- Activities: pushed pages die on pop. Activities must escape the stack, so
  the model needs a second structure anyway.
- 80×24: best-behaved of the page models (single column by construction).
- Verdict: right way to deepen flows, wrong way to hold running work.

### M5 · Tabbed workspace (TablePro-like)
Root is a permanent first tab; every flow, plan and activity opens a tab.

- Empty state: as M2 inside the tab body.
- Activities: exactly right — named, retained, `Alt+N`, survive navigation,
  the strip shows state (`⠋ ! ✓`). Plans execute as tabs with a step rail.
- Disclosure: too much. `Disk › Usage › Cleanup` as three tabs is noise; tabs
  for things that do not run outlive their purpose and need closing.
- Scope: unchanged from M2.
- 80×24: strip holds ~4 tabs before `‹ ›` overflow; acceptable.
- Verdict: right multiplexer, wrong flow container.

### Comparison

| criterion | M1 palette | M2 split | M3 board | M4 stack | M5 tabs |
|---|---|---|---|---|---|
| useful empty state | – | ++ | + (loud) | ++ | ++ |
| scope visibility | + | + | ++ | ++ | + |
| disclosure Disk/Cleanup/Plan | – | 0 | – | ++ | – |
| two-gate ergonomics | + | + | + | ++ | + |
| activity switching | – | –– | – | –– | ++ |
| 80×24 | ++ | + | –– | ++ | + |
| destination <1 s | ++ | + | – | ++ | + |
| one green | ++ | + | 0 | ++ | + |
| §15 one mental model | 0 | + | –– | + | + |

No single seed satisfies §8.14 and §8.15 together with §6/§16. The two
strong halves are M2+M4 (root and flows) and M5 (running work).

---

## 2. Chosen direction · "Workbench": tabs hold what runs, the Here tab holds a finder with a page stack

Rule that makes the hybrid coherent: **a tab is something that runs; a page is
something you are deciding.** Concretely:

- Tab strip (Tabs widget, DESIGN Tabs): first tab `Here` is permanent. Other
  tabs are activities (dev servers, logs, `btm`, `pg_activity`, one-shot
  commands with output) and executing plans. Nothing else ever becomes a tab.
- The `Here` tab body is a **finder**: query row + grouped results + preview
  (Master/detail composition). Flows push pages *inside* the Here tab (M4);
  the tab label becomes a breadcrumb (`Here › Disk › Usage`). `Esc` pops.
- Gate 1 is a pushed page; Gate 2 is Dialog::facts on top of it. Plan review is
  a pushed page; `Confirm plan` turns it into a plan tab and pops the page.
- Every result row reads `label · type · scope · reason` in fixed columns; the
  preview answers §7 as a Props block.

Why this and not the alternatives: it uses two compositions the design system
already proves (Explorer+tabbed workspace, Master/detail), it keeps the root
small (one finder, one preview), it keeps activities one keystroke away and
visibly alive (the strip's spinner is the one live-activity green), and flows
still deepen linearly with a breadcrumb so `d` `u` reads as push, push. The
cost: at <100 cols the preview is a drawer, and the Here tab's breadcrumb can
get long (truncate_middle keeps the tail).

### §15 answers in this model
- context vs overwhelm: menu bar carries path/host; StatusBar carries git
  state, discovery, capacity; the body carries only ranked rows.
- recommendation vs match vs navigation: section headers (`Suggested here`,
  `Results`, `Explore`, `Running`), never colour.
- primary vs alternatives: `Enter` vs `Alt+Enter` ContextMenu on the row.
- keyboard discovery: hint bar per focused control; `?` only when letter-hot,
  `F1`/Help menu always.
- live discovery: rows append below the cursor; the selected identity is
  kept; StatusBar centre shows `⠋ discovering children 3/7`.
- one model across representations: finder pages (root, domain pages, files)
  are the same widget with a different source and breadcrumb.
- risk/confidence/trust/scope distinct: four separate Props lines (`Gate`,
  `Why`, `Trust`, `Scope`) — never merged into one badge.
- long work returning to root: it never leaves; it is a tab, and the root's
  `Running` section lists it with state.
- tabs vs multiplexer: tabs, with a `Ctrl+]` attach/detach seam so an
  interactive program can own the keyboard deliberately.
- graph without overwhelming simple actions: single actions never show a
  plan; plans show a lane outline (stages + lanes), collapsible per lane.
- exclusion consequences: recalculated in place, one sentence under the list.
- aggregate ↔ per-step: one plan tab: rail left, selected step's output right.

---

## 3. Shell and key grammar

Shell rows (100×30): menu bar · blank · tab strip (2) · body (23) · blank ·
StatusBar · hint bar. At 80×24 the body is 17 rows.

- Menu bar: ` holla❯ ` lockup, `File Go Help`, right-aligned `project ·
  path   host` (host bold when remote; `◆` before a production host).
- StatusBar: left = git state of the current scope (`api  main · 4 changed ·
  1 behind`, strong first item); centre = discovery or the focused activity
  (`⠋ discovering children 3/7` / `frontend dev · running 12 min`); right =
  capacity meters (`disk 82%`, `load 3.1`) and `attached` chip when a PTY
  owns the keyboard.
- Hint bar: single, centred layer; `EDIT` badge only while a form field edits.

### Global chords (work on every page, before the focused widget)
| key | action |
|---|---|
| `Alt+0` | Here tab (pops nothing; the stack is kept) |
| `Alt+1–9` | activity tab N |
| `Ctrl+G` | activities picker (Picker: state · scope · elapsed) |
| `Ctrl+↑` / `Ctrl+↓` | scope outward (here → parent → system) / inward (here → children) |
| `Alt+Enter` | alternatives menu for the focused row |
| `Tab` / `Shift+Tab` | next / previous focus stop (finder → preview → tab strip) |
| `Ctrl+]` | detach from an attached program (returns to observe mode) |
| `Ctrl+C` | in an activity: interrupt it; elsewhere: quit (dialog if activities run) |
| `F1` | help; `?` too on letter-hot pages |
| `Esc` | climbs the ladder of the current page (below) |

Letter chords (`q`, `?`, `0`, `[ ]`) exist only on letter-hot pages and on the
tab strip; on typing-hot pages they are text. The hint bar says which mode
the page is in because it lists either `Type to search` or the verbs.

### Contextual letters (letter-hot pages and the tab strip)
`s` sort · `f` filter (or follow-tail in a viewport) · `p` preview · `x` close
· `u` undo (restore a hidden/demoted row, un-exclude a step) · `y` copy
(command or path) · `a` toggle all · `*`/`-` expand/collapse · `/` filter
field · `r` retry/refresh · `i`/`Enter` attach (activity).

### Typing, Tab, Enter, Alt+Enter on a finder page
- Typing always edits the query; the query has no separate edit mode (Picker
  model). Backspace on an empty query does nothing; `Esc` clears it.
- `↑↓` move the row cursor; `PgUp/PgDn`, `Home/End` as usual. `j k` are text.
- `Enter` = primary action of the row. Destructive rows open Gate 1 instead
  of running; the preview's `Gate` line says so before you press.
- `Alt+Enter` = ContextMenu anchored right of the row: Run now · Insert into
  shell · Copy command · Arguments… · Change scope › · Pin · Alias… · Hide ·
  Why is this here? · Open with ‹specialist›. Blue cursor row, per DESIGN.
- `Tab` leaves the finder for the preview (a scroll stop), then the tab strip.
  The finder is one composite stop: the query's accent underline and the
  row's `▎` are the same focus; when focus leaves, the underline drops to
  border-strong.
- Deviation from the modal Picker: `Tab` does *not* cycle scope here because
  the page has three stops. Scope has its own chord (`Ctrl+↑/↓`).

### Aliases and two-step entry
- Exact alias = ranking tier 1 (CONCEPT §9). Query `gp` shows `Pull` first
  with tag `alias gp`; `Enter` runs it. Learned ranking cannot move it.
- `du` = alias of Disk › Usage; `Enter` pushes `Here › Disk › Usage` directly.
- `d` `Enter` `u` `Enter`: `d` is the alias of the Disk domain page (a finder
  page with a fresh query); inside it `u` is the alias of Usage. Every domain
  page is typing-hot, so two-letter paths are just two short queries.
- Prefix chords are avoided on purpose: the query is always live.

### Scope invocation
- `Ctrl+↑/↓` walk the ring; the query row's right end reads `scope ‹ here ›`,
  clickable.
- `@parent`, `@children`, `@system`, `@all` query tokens (deterministic, text
  route): `@system docker clean`.
- `Esc` (ladder step 2) resets scope to `here` — returning to Current is
  always immediate.
- Nonlocal rows show up in `here` scope only under `From other scopes` at the
  bottom when urgent or an exact match (§5.1). Their scope column is
  text-secondary (louder than the muted `here`).

---

## 4. Screen inventory

Format per screen: communicates · focus stops · Esc ladder · hint layer ·
80×24.

### 4.1 Here (root finder, empty)
- Communicates: path/host (menu bar), git state and discovery (StatusBar),
  Suggested here with reasons, Recent here, Running activities with state,
  Explore domains, preview of the cursor row.
- Stops: finder (query+rows), preview, tab strip.
- Esc: clear query → scope to here → pop page (none) → tab strip → `q` quits.
- Hints: `Type to search  ↑↓ Move  Enter Run  Alt+Enter More  Tab Preview
  Ctrl+↑↓ Scope  Ctrl+G Activities`.
- 80×24: preview becomes a drawer (Tab opens it over the body; Tab/Esc puts
  it away); a one-line summary under the list stands in (`→ cargo nextest
  run · here · read-only`). Explore collapses to one row. Running shows 2
  rows max. Row meta drops all-or-none below 12-cell labels.

### 4.2 Here (with query)
- Communicates: `Results · N` in one ranked list, each row `label · type ·
  scope · reason`; alias rows first with `alias` tag; `From other scopes`
  section when the scope is narrowed; `No matches` empty state with the
  scope hint (`Ctrl+↑ widens to parent`).
- Stops/Esc/hints: as 4.1; `Esc` first clears the query.
- 80×24: reason column drops first, then type is kept (type+scope are never
  dropped — §6.3).

### 4.3 Preview pane (right of the finder; drawer <100)
- Communicates (Props): `Does` · `Runs` (exact command, code tone) · `In`
  (cwd) · `Host` · `Scope` (here/parent/child/host + defining file) · `Why`
  (strongest signal) · `Changes` · `Data` (live/cached 40 s/partial) · `Gate`
  (none / confirm / two gates · typed phrase) · `Trust` (for contributed
  workflows). Alternatives are listed in muted text so `Alt+Enter` is not a
  secret.
- Stops: one (scroll). `y` copies the command when the pane has focus.
- Esc: back to the finder.
- 80×24: drawer covering the body, framed, title = row label; `Tab`/`Esc`
  close.

### 4.4 Domain page (`Here › Disk`, `› Git`, `› Tasks`, `› Services`, `› System`, `› Files`)
- Same finder widget with a domain source and a breadcrumb; System sets scope
  to host and says so in the StatusBar (`scope host · devbox`).
- Esc: clear query → pop to Here.
- 80×24: as 4.1.

### 4.5 Disk usage (`Here › Disk › Usage`) — letter-hot
- Communicates: Tree of largest-first entries with size, age, kind
  (`target · rebuildable`, `node_modules · owned by apps/web`), scan
  progress (`⠋ 61% · 3.2 GB so far`), protected paths marked.
- Stops: tree; `f` filter; `s` sort; `p` preview (props for the node);
  `Space` selects candidates; `Enter` drills in; `c` opens Cleanup review
  with the selection.
- Esc: clear filter → collapse to root → pop.
- 80×24: meta all-or-none; kind column drops before size.

### 4.6 Cleanup review (`… › Cleanup`) — letter-hot
- Communicates: multi-select List of candidates grouped by project/family,
  `[✓]` default-selected only for rebuildable and old, size, mode (Trash /
  permanent / tool-managed), reclaimable total, dry-run result identical to
  execution rules.
- Stops: list, `Cancel`, `Review deletion…` (danger, ends in `…`).
- `Enter` on `Review deletion…` pushes Gate 1. Esc pops.
- 80×24: single column, buttons below the list.

### 4.7 Arguments (`… › Arguments`) — form
- Communicates: fields with label, expected value, default, validation, `•`
  masked for secrets, and a card with the live command.
- Stops: fields in reading order → `Cancel` → `Run` (primary). `Ctrl+S` runs.
- Esc: revert field → pop.

### 4.8 Gate 1 · review (`… › Review`) — page
- Communicates: plain-language action; absolute path/host/cwd; resource
  classes with counts and sizes; hidden/nested/symlink/mount inclusion;
  full command sequence (viewport); recoverable vs permanent; uncertainty,
  privilege, provenance, trust.
- Stops: facts viewport → `Cancel` (focused first) → `Continue…` (danger).
  The Enter that selected the action never reaches this page.
- Esc: pop (cancel is the default outcome).
- 80×24: facts scroll; action row stays pinned above the blank row.

### 4.9 Gate 2 · typed phrase — Dialog::facts (66 wide)
- Communicates: the same facts condensed, six command lines + `… N more`, the
  exact phrase to type, the field, `Cancel` + danger button disabled until
  the phrase matches exactly (`I UNDERSTAND:` prefix for host-wide).
- Stops: field (initial) → Cancel → confirm. `Enter` in the field only moves
  focus (DESIGN).
- Esc: cancel the dialog → Gate 1 page.
- Re-resolution before execution; a changed target closes the dialog with a
  status `Plan changed · review again` and re-renders Gate 1.
- 80×24: dialog fits (66 < 80); code preview capped at 4 lines.

### 4.10 Plan review (`Here › Upgrade everything`) — letter-hot
- Communicates: intent + host; lane outline (stage order, `needs`, lane
  letter, `join`); required vs optional; `[✓]` inclusion; privilege; the
  consequence sentence after any exclusion; step Props on the right.
- Stops: outline (Space toggles; required rows refuse with a status) →
  preview → `Cancel` → `Confirm plan` (primary; becomes `Continue…` when any
  step is destructive, which routes through Gate 1/2).
- `u` undoes the last exclusion; `-`/`*` collapse or expand lanes (§18 Q23).
- Esc: pop.
- 80×24: step Props become a drawer (`p`); lane column kept, `needs` dropped.

### 4.11 Plan tab (executing) — letter-hot
- Communicates: StepRail on the left with states (`·` queued, `⠋` running,
  `✓` done, `–` skipped/excluded, `✗` failed bold, `·` faint blocked), lane
  letter, elapsed; right = TextViewport of the selected step's retained
  output; StatusBar centre = `⠋ 3 of 8 · 2 running · 1 blocked · next 08
  waits for 04`.
- Stops: rail → viewport. `Enter` maximises the viewport (`z` too); `f`
  follow; `y` copy; `r` retry the failed step or its lane; `Ctrl+C` cancels
  remaining work (dialog: cancel remaining / stop running / keep).
- Esc: un-maximise → rail → tab strip.
- Final state: summary rows appended to the rail (`succeeded 6 · failed 1 ·
  excluded 2 · never started 1`); the tab's state slot shows `✓` or `!`.
- 80×24: rail full width; viewport behind `Enter` as a maximised view.

### 4.12 Activity tab — observe / attached
- Communicates: name, originating action, scope + cwd + host (StatusBar
  centre), state and elapsed, exit status, retained output, insights strip
  when the program exposes them (port, URL, error count).
- Observe mode keys: `↑↓ PgUp PgDn g G` scroll, `f` follow, `y` copy, `x`
  close (dialog if running), `r` restart, `Enter`/`i` attach.
- Attached: every key goes to the program; StatusBar right shows the
  `attached` chip; `Ctrl+]` detaches. Ctrl+C is the program's.
- Esc: clear selection → observe (if attached, Esc is the program's) → tab
  strip.
- 80×24: identical; the insights strip drops first.

### 4.13 Activities picker (`Ctrl+G`) — Picker
- Rows: `⠋/✓/! name · scope · cwd · elapsed`; `Enter` switches, `Delete`
  stops, `Alt+Enter` restarts. Also reachable as the `Running` section in
  Here.

### 4.14 Trust review (`… › Trust mise.toml`) — page
- Exact file, commands, env effect, propagation scope; `Cancel` (focused) /
  `Trust this file`. Re-shown when the definition changes.

### 4.15 Quit with running activities — Dialog::destructive
- `Keep running in background` (primary if the shell integration supports
  detach) / `Stop all` (danger) / `Cancel`.

### 4.16 Too small (<72×20)
- DESIGN four-line notice with the lockup.

---

## 5. Sketches at 100×30

Legend for every sketch: the only green cells are `▎`, the `›` on the focused
row, the `━` under the active tab, spinners `⠋` on running tabs/steps, a
primary button, and the lockup fill. `[ ]`/`[✓]` marks are green only when
checked. Red only on `!`, danger button labels and failed steps.

### 5.1 Here · empty state

```
 holla❯  File  Go  Help                                 api · ~/work/monorepo/apps/api      devbox
                                                                                                   
  Here    frontend dev ⠋   api tests ✓                                                             
 ━━━━━━──────────────────────────────────────────────────────────────────────────────────────────
 ▎Search actions and resources…                                                    scope ‹ here ›
                                                                                                   
  Suggested here                                     │ ▎Run tests                       task · here
 ▎› Run tests                 task     used 6× here  │
    Review 4 modified files   git      worktree dirty│  Does     run the project's test task
    Pull                      git      1 behind      │  Runs     cargo nextest run
    Start API dev             task     child apps/api│  In       ~/work/monorepo/apps/api
    Start ecosystem           task     parent monorepo│  Host     devbox · local
                                                     │  Scope    here · mise.toml [test]
  Recent here                                        │  Why      used 6× here · src changed 4 min
    Cargo check               task     2 h ago       │  Changes  none · read-only
    Follow api logs           docker   yesterday     │  Data     live
                                                     │  Gate     none · runs directly
  Running                                            │
    frontend dev              ⠋ 12 min child apps/web│  Alt+Enter  insert · copy · arguments… · pin
    api tests                 ✓ exit 0  4 min ago    │
                                                     │
  Explore   Tasks  Git  Files  Disk  Services  System│
                                                     │
                                                     │
                                                     │
                                                                                                   
 api  main · 4 changed · 1 behind      ⠋ discovering children 3/7      disk 82% ━━━━━━━━──  load 3.1
      Type to search  ↑↓ Move  Enter Run  Alt+Enter More  Tab Preview  Ctrl+↑↓ Scope  Ctrl+G Tabs
```

Notes: `Running` rows switch to the tab on `Enter`. `Explore` is one row of
choosable cells (`←→` inside it) so the empty state stays under 20 rows. The
`│` is a Splitter seam; the preview is a card on the surface plane (fill not
drawable in ASCII). The StatusBar sits on the elevated plane with three-cell
group gaps, no separators.

### 5.2 Here · query `logs`, scope `all`

```
 holla❯  File  Go  Help                                 api · ~/work/monorepo/apps/api      devbox
                                                                                                   
  Here    frontend dev ⠋   api tests ✓                                                             
 ━━━━━━──────────────────────────────────────────────────────────────────────────────────────────
 ▎logs▁                                                                             scope ‹ all ›
                                                                                                   
  Results · 6                                        │ ▎Follow worker logs          docker · child
 ▎› Follow api logs           docker   here          │
    Follow worker logs        docker   child  worker │  Does     stream logs of service worker
    api.log                   file     here   2.1 MB │  Runs     docker compose logs -f worker
    postgres journal          service  host   devbox │  In       ~/work/monorepo/services/worker
    Follow api + worker logs  docker   here   2 svc  │  Host     devbox · local
    Logs task                 task     parent  mise  │  Scope    child · compose project monorepo
                                                     │  Why      matches "logs" · service unhealthy
                                                     │  Changes  none · read-only stream
                                                     │  Data     live · compose ps 2 s ago
                                                     │  Gate     none · opens as an activity
                                                     │
                                                     │  Alt+Enter  insert · copy · since… · pin
                                                     │
                                                     │
                                                     │
                                                     │
                                                     │
                                                                                                   
 api  main · 4 changed · 1 behind      scope all · 4 sources                disk 82% ━━━━━━━━──
      Type to search  ↑↓ Move  Enter Open  Alt+Enter More  Tab Preview  Esc Clear  Ctrl+↑↓ Scope
```

Columns are fixed over all rows (`label · type · scope · detail`), so
streaming rows never shift alignment. `here` scope is muted; `child`,
`parent`, `host` are text-secondary. The cursor sits on row 2 to show that
the preview follows the cursor, not the first result.

### 5.3 Preview of a broad destructive action · query `docker clean`, scope `system`

```
 holla❯  File  Go  Help                                 api · ~/work/monorepo/apps/api      devbox
                                                                                                   
  Here    frontend dev ⠋   api tests ✓                                                             
 ━━━━━━──────────────────────────────────────────────────────────────────────────────────────────
 ▎docker clean▁                                                                  scope ‹ system ›
                                                                                                   
  Results · 5                                        │ ▎Clean Docker completely      docker · host
 ▎› Clean Docker completely   docker   host  alias dc│
    Stop all containers       docker   host  12 up   │  Does     stop and remove all containers,
    Prune builder cache       docker   host  6.1 GB  │           remove all images, prune networks,
    Prune unused volumes      docker   host  7 vol   │           volumes, system data, build cache
    Remove all images         docker   host  41 img  │  Runs     6 commands · see review
                                                     │  In       — (host-wide)
                                                     │  Host     devbox · local
                                                     │  Scope    host · every project on devbox
                                                     │  Why      alias dc · used 9× on devbox
                                                     │  Changes  12 containers, 41 images, 7 volumes
                                                     │           frees ~18.4 GB · volumes permanent
                                                     │  Data     live · docker system df 3 s ago
                                                     │  Gate     two gates · review, then type
                                                     │           REMOVE ALL DOCKER DATA ON devbox
                                                     │
                                                     │  Alt+Enter  stop only · copy · why here?
                                                     │
                                                     │
                                                                                                   
 api  main · 4 changed · 1 behind      scope host · devbox                    docker 18.4 GB
      Type to search  ↑↓ Move  Enter Review…  Alt+Enter More  Tab Preview  Esc Clear
```

The hint bar relabels `Enter Run` to `Enter Review…` on a destructive row: the
primary interaction opens Gate 1, never executes. Ranking is untouched by
risk (alias tier keeps it first); only the `Gate` line and the hint change.

### 5.4 Plan review · `Here › Upgrade everything`

```
 holla❯  File  Go  Help                                 api · ~/work/monorepo/apps/api      devbox
                                                                                                   
  Here › Upgrade everything    frontend dev ⠋   api tests ✓                                        
 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━──────────────────────────────────────────────────────────────────────
  Upgrade everything on devbox                        8 steps · 6 included · 2 excluded · ~6 min
                                                                                                   
  Steps                          needs   lane        │ ▎Apply Debian upgrades              step 04
 ▎[✓] 01 Preflight                 –       ·  required│
   [✓] 02 Refresh Debian metadata  01      a          │  Runs     sudo apt-get upgrade -y
   [✓] 03 Review Debian upgrades   02      a  14 pkgs │  In       / · devbox
   [✓] 04 Apply Debian upgrades    03      a  sudo    │  Needs    03 Review Debian upgrades
   [✓] 05 Inspect global mise      01      b          │  Unlocks  07 cleanup (excluded) · 08 verify
   [ ] 06 Upgrade mise tools       05      b  excluded│  Lane     a · runs beside lane b after 01
   [ ] 07 Package cleanup          04      a  optional│  Privilege sudo · asked once at 01
   [✓] 08 Final verification       04 06   join       │  Risk     mutating · bounded
                                                     │  Effect   14 upgraded · 0 removed · 0 held
  06 excluded → 08 verifies lane a only               │  Data     live · apt list 40 s ago
  01 is required by every step and cannot be excluded │  Gate     one confirmation · this page
                                                     │
                                                     │
                                                     │
                                                     │
                                                     │
                                                                          Cancel   ▎Confirm plan 
                                                                                                   
 api  main · 4 changed · 1 behind      plan · 2 lanes after 01               disk 82% ━━━━━━━━──
      ↑↓ Move  Space Include/exclude  p Step  u Undo exclusion  - Collapse lane  Enter Confirm
```

The outline is stage order with a lane letter, not a drawn graph: readers
scan `needs` to see edges, `lane` to see parallelism, `join` to see
convergence. Excluding a prerequisite recomputes in place and writes the
consequence sentence under the list; dependents flip to `[ ] … blocked by 05`
in faint and cannot be re-included until the prerequisite is. `Confirm plan`
is the page's one primary button; the plan becomes a tab on confirm.

### 5.5 Plan tab executing (activity multiplexer)

```
 holla❯  File  Go  Help                                 api · ~/work/monorepo/apps/api      devbox
                                                                                                   
  Here    frontend dev ⠋   api tests ✓   upgrade ⠋                                                 
 ───────────────────────────────────────━━━━━━━━━━━────────────────────────────────────────────────
  Upgrade everything on devbox         3 of 6 done · 2 running · 08 waits for 04     ⠋ 2 min 14 s
                                                                                                   
  Steps                                       lane    │ 04 Apply Debian upgrades           ▲ 12 
  01 ✓ Preflight                        4 s    ·      │ Preparing to unpack libssl3_3.0.13…
  02 ✓ Refresh Debian metadata          11 s   a      │ Unpacking libssl3:amd64 (3.0.13-1) …
  03 ✓ Review Debian upgrades           2 s    a      │ Setting up libssl3:amd64 (3.0.13-1) …
 ▎04 ⠋ Apply Debian upgrades            1:41   a      │ Preparing to unpack openssl_3.0.13…
  05 ⠋ Inspect global mise tools        1:58   b      │ Unpacking openssl (3.0.13-1) over …
  06 – Upgrade mise tools               excluded b    │ Setting up openssl (3.0.13-1) …
  07 – Package cleanup                  excluded a    │ Processing triggers for man-db …
  08 · Final verification               waits 04 join │ Preparing to unpack curl_8.5.0-2…
                                                      │ Unpacking curl (8.5.0-2) over (8.5.0-1) …
                                                      │ Setting up curl (8.5.0-2) …
                                                      │ ▁
                                                      │
                                                      │
                                                      │
                                                      │
                                                      │
                                                                                                   
 api  main · 4 changed · 1 behind      upgrade · lane a 04 · lane b 05          sudo ok  disk 82%
      ↑↓ Step  Enter Maximise  f Follow  y Copy  r Retry  Ctrl+C Cancel remaining  Ctrl+G Tabs
```

The tab strip is the multiplexer: `Alt+3` returns here from anywhere, `Alt+0`
to Here, `Ctrl+G` lists all. Two spinners on the rail and one on the tab are
the only green besides the rail's `▎` — all "live activity" by DESIGN. The
`▲ 12` cue in the viewport title says 12 lines of scrollback are above the
tail (follow paused). A failed 04 would show `✗` bold red, 07/08 would turn
faint `blocked by 04`, and lane b would keep running.

### 5.6 Gate 2 · typed phrase over the Gate 1 page (dimmed)

```
 holla❯  File  Go  Help                                 api · ~/work/monorepo/apps/api      devbox
                                                                                                   
  Here › Clean Docker completely › Review    frontend dev ⠋   api tests ✓                          
 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━────────────────────────────────────────────────────────
  Clean Docker completely on devbox                                          host-wide · permanent
                                                                                                   
  Action     stop and remove all containers, remove all images, prune networks, volumes, …
  Host             ╭ Remove all Docker data on devbox ─────────────────────────────╮
  Containers       │                                                               │
  Images           │   Action      stop + remove 12 containers, remove 41 images,  │
  Volumes          │               prune networks, 7 volumes, system data, cache   │
  Networks         │   Host        devbox · local                                  │
  Build cache      │   Scope       host-wide · every project on this machine       │
  Frees            │   Frees       ~18.4 GB                                        │
  Reversible       │   Reversible  no · volumes and images are permanent           │
  Sequence         │                                                               │
    docker stop    │   docker stop $(docker ps -q)                                 │
    docker rm -f   │   docker rm -f $(docker ps -aq)                               │
    docker rmi -f  │   docker rmi -f $(docker images -q)                           │
    docker network │   … 3 more                                                    │
    docker volume  │                                                               │
    docker builder │   Type REMOVE ALL DOCKER DATA ON devbox to confirm            │
                   │  ▎REMOVE ALL DOCKER DATA ON dev▁                              │
                   │                                                               │
                   │                        Cancel   Remove all Docker data        │
                   ╰───────────────────────────────────────────────────────────────╯
                                                                                                   
 api  main · 4 changed · 1 behind      review · gate 2 of 2                       docker 18.4 GB
 EDIT   Type the phrase  Tab Next  Esc Cancel
```

Focus starts in the field (typed-acknowledgement rule). `Remove all Docker
data` is a danger button, faint (disabled, out of the ring) until the phrase
matches exactly, then error-toned; `Cancel` is secondary. `Enter` in the field
moves focus, never confirms. The Gate 1 page behind is walked two ladder steps
down; its facts stay legible so the modal is not the only source of truth.
If re-resolution before execution finds a changed set, the dialog closes with
status `Plan changed · review again` and Gate 1 re-renders.

---

## 6. 80×24 summary

| screen | change |
|---|---|
| Here / domain pages | preview → drawer (`Tab`), one-line summary under the list; Explore as one row; Running ≤2 rows; reason column drops, type + scope never |
| Disk usage / Cleanup | meta all-or-none; buttons below the list |
| Plan review | step Props → drawer (`p`); `needs` column drops, lane kept |
| Plan tab | rail full width; output via `Enter` maximise |
| Activity tab | unchanged; insights strip drops |
| Gate 1 | facts scroll; action row pinned |
| Gate 2 | 66-wide dialog fits; code capped at 4 lines |
| Tab strip | `‹ ›` overflow after ~4 tabs; breadcrumb truncate_middle keeps the tail |
| StatusBar | centre leaves first, then right; left strong item truncates |
| Hint bar | drops from the right; `Type to search` / first verb always survives |

160×50: no new panes. The finder list gains the reason column at full length,
the preview card gets wrapped `Does`/`Changes`, the plan tab shows the rail
and the viewport side by side with more scrollback. Width is spent on
legibility, not on a third column — §16 "do not compensate with categories".

---

## 7. Known weaknesses of the chosen direction

- `Ctrl+↑/↓` for scope is invisible until read in the hint bar; the `@scope`
  tokens and the clickable readout are the discoverable routes. Watch this in
  user testing.
- The Here tab breadcrumb can reach `Here › Disk › Usage › Cleanup › Review`;
  truncation keeps the tail, but the head is what says "you are inside Here".
  Mitigation: the breadcrumb collapses to `Here › … › Review` at <100 cols.
- Two spinners (rail + tab) plus the row bar are three greens on the plan tab.
  DESIGN allows each; the sketch stays under its "one accent rule" because
  only the tab has a `━`. Still the densest green screen in the product.
- Attach/detach (`Ctrl+]`) adds a mode. The `attached` chip in the StatusBar
  and the hint-bar layer swap are the only signals; a monochrome terminal
  must rely on the chip text.
- One-shot commands become tabs that need closing. Mitigation: read-only
  one-shots with short output auto-close on `Esc` from the tab and show their
  outcome in `Recent here` (`✓ 2 s ago`).
