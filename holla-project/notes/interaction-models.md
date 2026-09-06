# Holla — interaction-model brainstorm and decision

Status: decision recorded. Date: 2026-09-06.
Inputs: `holla-project/CONCEPT.md`, `holla-project/references/*.md`, `DESIGN.md`,
`GOAL.md` §3 (interaction-model brainstorm gate).

Nothing below is a wireframe. CONCEPT.md examples were treated as semantic
requirements; each model was derived from the needs in §2–§8 and then checked
against DESIGN.md's grammar. The ASCII in §6 is coarse information placement
for the chosen model, not pixel art.

---

## 0. What any model must carry

Distilled from CONCEPT.md; every candidate is graded against these.

1. **One root, useful before typing** — one entry point; empty state shows
   Suggested here (with reasons), Recent here, Explore entries, discovery
   status (§6.1–6.2). Never a category chooser first.
2. **cwd is primary, scopes are explorable** — Current default; Parent,
   Children, System reachable without relaunch; every result carries
   discovered-where, executes-where, scope class, and why-relevant for
   nonlocal rows (§5.1–5.4, contract §5.5 items 1–5, 11–12).
3. **One search across domains** — actions + resources + flows + handoffs in
   one result set; type and scope visible per row; exact aliases beat learned
   ranking (§6.3–6.6, §9).
4. **Preview answers six questions** (§7): what happens, target, why
   recommended, what changes, data freshness, confirmation/trust needed.
5. **Risk changes treatment, never availability** (§2.5, §10). Broad
   destructive = two gates: gate 1 review of the resolved plan (target, host,
   counts, sizes, inclusions, sequence, recovery), gate 2 a typed target-bound
   phrase (`REMOVE ALL DOCKER DATA ON devbox`, `I UNDERSTAND:` prefix for the
   broadest). Selection never counts as a gate. Revalidation on plan change.
6. **Compound intent = DAG plan, not a script** (§8.15–8.16): optional-step
   exclusion with downstream consequences explained; parallel branches only
   when independence proven; convergence steps; failure blocks dependents, not
   unrelated branches; per-step output retained as activities.
7. **Activities are persistent named contexts** (§8.14): running / waiting /
   succeeded / failed / stopped / detached; retained output, cwd, host, scope;
   fast switching; return to discovery without losing state. "Tab-capable,
   multiplexer-like"; representation open.
8. **Completion loops back** (§6.8): outcome, duration, scope, failures,
   follow-up actions.
9. **Local/remote consistent but unmistakable** (§8.12, §16); sensitive hosts
   declare role and get stronger confirmation.
10. **DESIGN.md shell and grammar**: menu bar + lockup, blank, body, blank,
    hint bar with layer precedence; focus = `▎` + bold; one accent; state
    grammar for empty/loading/partial/error/destructive/busy/success; 72×20
    minimum with prioritised dropping; facts dialog + typed acknowledgement is
    the default two-gate mapping (GOAL §2).

---

## 1. Model L — "Lens"

### Metaphor

Holla is a lens you hold up to the present moment. You summon it over your
shell, look through it at *here*, act, and put it away. It is not a place you
live in; it is a question you ask your context. Long-running work becomes
"photographs the lens took" — results that persist after the lens is closed
and can be re-summoned.

### Surfaces

- **Lens** — a centred picker in the upper third over a dimmed near-empty
  backdrop page (DESIGN "command palette" composition): query field, grouped
  result rows, scope readout in the title (`~/work/monorepo · devbox`),
  faint hint row, discovery spinner in the title meta.
- **Scope cycling** — `Tab` rotates a scope chip in the title row
  (All → Current → Parent → Children → System); per-row scope tag
  (`here · parent · frontend · host`) on every result.
- **Preview** — the focused row expands in place with a props block
  (what / target / why / changes / freshness / gate), or a right card appears
  inside a widened lens at ≥110 cols.
- **Zoom pages** — choosing a flow (Disk, Plan, Activity) "zooms" the lens to
  full screen. Esc zooms back to the lens; Esc again puts the lens away
  (process exits to shell).
- **Tray** — while activities exist, a one-line tray on the backdrop lists
  them (`⠋ frontend dev · ⠋ api logs`); a switcher lens mode
  (`Ctrl+G`-style) lists them as rows.
- **Gates** — facts dialogs over the dimmed lens; gate 2 carries the typed
  phrase field.

### Walkthrough

1. **Empty state** — summon: query empty, rows = Suggested here (reason in
   the detail column), Recent here, Explore; `⠋ discovering…` in the title.
2. **Query** — `test` filters actions+resources across domains; matched
   characters bold; scope tag right-aligned per row.
3. **Selecting** — arrows move a stable selection; `Enter` = primary action;
   `Alt+Enter` = alternates menu (inspect, change scope, copy, insert, pin).
4. **Preview** — focused row expands with the §7 answers; moving on collapses
   it.
5. **Destructive** — `docker clean` → Clean Docker completely ranked top
   (exact intent + frequent on this host, `host · destructive` tag). `Enter`
   never runs: gate-1 facts dialog (Docker accounting, classes, sequence,
   recovery), then gate-2 phrase dialog `REMOVE ALL DOCKER DATA ON devbox`.
   Click-outside dismissal must be *disabled* for gates — an exception to
   picker/dialog mouse rules.
6. **Plan** — "upgrade everything" zooms to a full-screen plan page (grouped
   steps, parallel groups, step card). At this point the "lens" is a
   full-screen app; the metaphor is stretched.
7. **Activities** — Esc to shell does not kill work; tray persists on next
   summon. Switching between `frontend dev` and `api logs`: summon →
   switcher → choose → zoom. Minimum three keystrokes plus re-entry, every
   time.

### Evaluation

- **Empty state**: good content but cramped — the picker lives in the upper
  third; Suggested + Recent + Explore + status fits, tightly.
- **Scope visibility**: strong per-row tags and title scope readout; scope
  cycling via `Tab` is one keystroke. All-scope search with grouped headers
  works.
- **Progressive disclosure**: weak-to-mixed. Every deep flow (Disk, Cleanup,
  Plan, Activities) must become a full-screen zoom, which is the metaphor
  breaking; the product ends up with two navigation grammars (picker rules /
  page rules).
- **Two-gate ergonomics**: mixed. Facts dialogs are DESIGN-native, but the
  lens's transient semantics (click-outside dismiss, Esc = put away) are
  actively dangerous around destructive review and need special-casing;
  "summon-and-dismiss" trains exactly the haste the gates exist to interrupt.
- **Activity switching**: weakest of the three. Persistent named activities
  with fast switching (§8.14) fight a transient overlay; switching costs a
  re-summon each time and the switcher is one more layer.
- **Narrow terminals**: excellent — the picker is compact by design; facts
  dialog (66) fits 72.
- **DESIGN.md fit**: the picker is a catalogue composition, but DESIGN
  defines pickers/modals as *transient layers over a page with its own
  focus ring*; making the transient layer the whole product inverts the
  elevation model and the modal-barrier focus model.
- **§15 questions**: strong on "distinguish recommendations/search/navigation"
  and keyboard discovery; weak on "one mental model across representations"
  (zoom pages are a second grammar), "several running activities remain
  accessible", "tab-like vs multiplexer-like" (Lens answers: neither —
  a re-summoned switcher), and "aggregate progress vs per-step output"
  (two different zoom pages).

---

## 2. Model A — "Atlas"

### Metaphor

Context is geography and you walk it. Here is a place; parents are up,
children are down, the host is the weather over all of it. The screen is a
strip of adjacent columns, left to right = orientation to depth: a compass
column, an items column, a detail column. Moving right goes deeper into a
thing; moving left comes home. Depth is spatial, not hierarchical.

### Surfaces

- **Compass column** (leftmost, fixed): `Current` / `Parent: monorepo` /
  `Children (3)` / `System` / `Activities (2)` — the §5.3 scope directions as
  a literal place you can point at. `←/→` or `h/l` between columns; the
  focused compass entry decides what the items column lists.
- **Items column** — the chosen scope's actions + resources, ranked; group
  headers in All-scope search; per-row scope marker inherited from the
  compass plus an explicit tag on cross-scope rows.
- **Detail column** — preview props + alternates as rows (the §7 questions
  and the §6.5 alternatives are the same column: facts above, action rows
  below).
- **Query** — one query row above the columns filters the items column;
  typing always targets it.
- **Plan** — items column becomes the step tree (groups = branches, sibling
  groups under a `Parallel` header, convergence row with `waits for` meta);
  detail column = step card.
- **Activities** — compass `Activities` entry; items = activities with state
  glyphs; detail column = live output viewport. Switching = moving the cursor
  — output is always on the right.
- **Gates** — the column strip is replaced by a two-column gate view (facts
  left, sequence right) then a phrase field; at 120 the detail column is
  ~38 cols, too narrow for 66-wide facts, so gates break the column layout.

### Walkthrough

1. **Empty state** — compass on Current; items = Suggested / Recent /
   Explore; detail column = context card (path, host, project, git summary).
   The "where am I" answer is the leftmost column, permanently.
2. **Query** — `logs` filters items; in All scope, group headers per scope
   keep rows labelled.
3. **Selecting** — `↑↓` in items; `Enter` primary; `→` steps into the detail
   column's alternates.
4. **Preview** — detail column *is* the preview: target, cwd, reason, risk,
   command, freshness; follows the cursor.
5. **Destructive** — `Enter` on Clean Docker completely swaps the strip to
   the gate view (layout exception); gate 1 facts + sequence, `Continue…`,
   gate 2 phrase field; Esc unwinds to the strip.
6. **Plan** — step tree in the items column; `Parallel` group rows with
   branch labels; convergence row `Final verification · waits for: debian,
   mise`; excluding a step in the detail column marks dependents
   `blocked: prerequisite excluded`.
7. **Activities** — compass → Activities; cursor on `frontend dev`, live
   output right; `↓` to `api logs`, output swaps instantly. Genuinely
   one keystroke.

### Evaluation

- **Empty state**: good — the context card gives the empty items column a
  reason to exist.
- **Scope visibility**: the strongest of the three. Scope is *geometry*: the
  compass column is the §5.4 contract rendered permanently; parent/child
  movement is `←/→`; returning to Current is one `←`.
- **Progressive disclosure**: native — every flow is "drill right". Disk and
  Cleanup are natural column chains (scope → candidates → candidate facts).
- **Two-gate ergonomics**: weak. Facts need ~66 cols; columns can't deliver
  that without abandoning the strip, so the most safety-critical moment in
  the product uses a different layout than everything else.
- **Activity switching**: excellent in the Activities place (cursor move,
  live output alongside); but leaving to discovery and back means walking
  the compass, and output visibility depends on staying in that place.
- **Narrow terminals**: worst of the three. Three columns at 72 = ~22 cols
  each; the compass + items + detail cannot coexist; the model degenerates
  into one column with modes, losing its core idea exactly when space is
  scarce (remote sessions, §16/§18.16).
- **DESIGN.md fit**: mixed. Lists/trees/viewport are catalogue widgets, but
  the column strip container is a new generic widget (needs a showcase page
  per GOAL §2), nested scroll regions multiply, and the plan DAG wants
  connector semantics the tree widget deliberately avoids ("Avoid: ASCII
  connector lines").
- **§15 questions**: superb on "how users move between current, parent,
  child, system scope" and "one mental model" (everything is a column of
  things); weak on "dependency graphs … understandable" (a tree column shows
  hierarchy, not convergence — the DAG's defining feature), on narrow-host
  presentation, and on gate distinctness.

---

## 3. Model C — "Console"

### Metaphor

An operator console for *here*. One home surface permanently answers "what
can I do here", and its primary instrument is a command line that is always
ready. Everything else — flows, plans, activities, gates — is a page the
console opens; Esc always walks back home, and the console never forgets
running work: activities are channels docked in a strip under the menu bar.

Launcher speed is preserved structurally: home *is* a query field with a
ranked list under it — the full window is the launcher, not an IDE around
one.

### Surfaces

- **Home** — menu bar (lockup `holla❯`, menus, path breadcrumb, host,
  discovery status); query row (focused at launch); results list (empty
  query → Suggested here / Recent here / Explore with faint section headers;
  query → mixed results with per-row scope and risk tags); after completions,
  a transient Follow-ups section (§6.8).
- **Inspector** — at ≥100 cols a preview card right of the list answers §7
  for the focused row (what, target, runs-in, why, changes, freshness, gate);
  below 100 cols `p` opens the same content as a full-body drawer that closes
  when focus leaves (DESIGN responsive rule: secondary panes become drawers).
- **Action menu** — `o` opens the §6.5 alternatives as an anchored context
  menu on the focused row (inspect, change scope, copy command, insert into
  shell, run now, pin/alias/hide/reset, choose specialist).
- **Args form** — structured arguments (§6.7) as a form page/section: label,
  value, default, validation, secret masking, live command preview.
- **Flow pages** — Disk, Cleanup, Tasks, Git, Files, Services, System:
  full-body pages with their own hint layers, entered from Explore, search,
  or suggestions.
- **Plan page** — left: the graph as a grouped step list (group headers =
  phases, `Parallel` groups = proven-independent branches, step rows use
  step-rail state glyphs, convergence rows carry `waits for:` meta); right:
  step card (exact command, scope, privilege, dependencies, expected effect,
  output tail). `Space` excludes optional steps; dependents flip to
  `blocked: <reason>` and the plan recomputes in place. During execution the
  page is the aggregate view; `Enter` on a step opens its activity tab.
- **Activity strip + activity page** — when ≥1 activity exists, a tab strip
  under the menu bar (`tabs-height` 2): `1 ⠋ frontend dev · 2 ⠋ api logs ·
  3 ✓ tests`. Activity page = status strip (state, scope, cwd, host,
  duration, exit), insights card (port, health, program insights), text
  viewport with follow-tail. Home is the base, not a tab: `0` or Esc returns.
- **Review (gate 1)** — a full-body reading surface for destructive and
  trust decisions: props (action, absolute target, host, env, cwd, counts,
  sizes, hidden/nested/symlink inclusion), resource-class list, full command
  sequence or truthful summary, recovery mode, exclusions, provenance/trust.
  Actions: `Cancel` (focused) and `Continue…`.
- **Gate (gate 2)** — DESIGN facts dialog with typed acknowledgement over
  the dimmed review surface: field requires the target-bound phrase
  (`REMOVE ALL DOCKER DATA ON devbox`; `I UNDERSTAND:` prefix for the
  broadest); Execute stays disabled until exact match; revalidation reruns
  on any material plan change. Bounded mutations use `review` + one `confirm`
  dialog; read-only skips both.

### Walkthrough

1. **Empty state** — home: menu bar shows `~/work/monorepo · devbox · local`
   and `⠋ discovering…`; query focused with a muted placeholder; Suggested
   rows carry reasons (`Pull · branch is 3 behind · here`); Recent here;
   Explore row. A plausible next action is visible in the first frame.
2. **Query** — `test` re-filters across domains; matched characters bold;
   each row: label · reason · scope tag (`here / parent / frontend / host`)
   · risk tag when relevant (`destructive` in the error tone, a sanctioned
   safety use). Selection is identity-keyed and survives streaming inserts.
3. **Selecting** — `↑↓`/`j k`; `Enter` = primary action (for destructive:
   opens review, never executes); `o` = action menu; exact aliases (`gp`,
   `du`) resolve deterministically regardless of ranking.
4. **Preview** — inspector follows focus and answers the §7 questions;
   `p` drawer below 100 cols; freshness stated (`live · cached 12 s`).
5. **Destructive** — query `docker clean` → Clean Docker completely on top
   (intent match + host frequency; risk tag does not demote). `Enter` →
   review page: Docker accounting, affected classes with counts and sizes,
   the six-step sequence, recovery (volumes/images permanent), host. Focus
   starts on `Cancel`. `Continue…` → gate dialog: type
   `REMOVE ALL DOCKER DATA ON devbox`; mismatch keeps Execute disabled;
   Esc cancels with status `Cancelled · nothing was executed`. Aliases never
   bypass either gate.
6. **Plan** — "upgrade everything" → plan page: Preflight → `Parallel`
   (refresh Debian metadata ∥ inspect mise tools) → Review → Apply branches
   → Optional cleanup → `Final verification · waits for: debian, mise`.
   `Space` excludes the cleanup step and one mise tool; dependents flip to
   blocked with reasons; the plan recomputes; `Run plan…` confirms and
   executes with branch parallelism; a failed branch blocks its dependents
   and leaves the other branch running; each step's output opens as an
   activity tab.
7. **Activities** — two tabs run: `1 ⠋ frontend dev`, `2 ⠋ api logs`.
   From an activity page, `1`/`2` jump directly (DESIGN tab grammar); from
   home, digits are query text (editing suppresses chords per DESIGN), so
   switching uses `Ctrl+1…9`, `[`/`]` when not editing, or the clickable
   strip; `0` returns home. Output, cwd, and scope are retained per tab.

### Evaluation

- **Empty state**: strongest — home is a real recommendation surface with
  room for Suggested + Recent + Explore + discovery status + follow-ups,
  not a crowded overlay.
- **Scope visibility**: menu-bar path/host is persistent chrome (contract
  §5.5.1–2); per-row scope + risk tags on every result; inspector states
  discovery scope *and* execution cwd; remote hosts get `◆` role identity in
  the bar. Slightly less spatial than Atlas, but always-on.
- **Progressive disclosure**: clean ladder — home → inspector → flow page →
  review → gate; each step adds exactly one surface and one Esc rung.
- **Two-gate ergonomics**: strongest — gate 1 is a *reading* surface (full
  width, scrollable, facts), gate 2 is a *typing* surface (modal barrier,
  one field). Two gates get two distinct geometries, so they can never blur
  into one click-through; the modal barrier guarantees containment; the
  dialog mapping is the GOAL §2 default, with the review page as the
  justified stronger gate-1 composition (reading needs width a 66-col dialog
  summary can't give; truthful summary stays available for narrow cases).
- **Activity switching**: fastest — one keystroke between activities on
  activity pages, strip always visible, state retained; multiplexer-like
  without embedding a multiplexer.
- **Narrow terminals**: inspector drops to a drawer, list metadata drops
  all-or-none, tab strip overflows with `‹ ›`, menu segments drop by
  priority (lockup, path, host last); plan page gives the whole body to the
  graph with the step card as a drawer; gate dialog 66 fits 72. All are
  DESIGN-sanctioned drops, no invention.
- **DESIGN.md fit**: native — the shell *is* DESIGN's shell; tabs, viewport,
  step-rail glyphs, quota meters (Disk), facts dialog, context menu, hint
  layers all come from the catalogue. The plan graph composes list + rail
  glyphs + group headers (screen-level composition, not a new generic
  widget). One accent discipline holds: focus bar, primary buttons, active
  tab rule, spinner, EDIT badge.
- **§15 questions**: context stays ambient without overwhelming (bar + tags,
  detail on demand); recommendations/search/navigation are visually distinct
  (sections vs filtered rows vs pages); live discovery streams under a
  frozen selection; one mental model (home + pages, Esc home) covers every
  representation; risk/confidence/trust/scope are separate row facts; local
  and remote share geometry and differ in identity + confirmation strength;
  DAG reads as groups + parallel blocks + convergence meta; exclusions
  recompute visibly; aggregate plan and per-step activity are one `Enter`
  apart.

---

## 4. Head-to-head

| Criterion | Lens | Atlas | Console |
|---|---|---|---|
| Empty state before typing | Mixed (cramped overlay) | Good (context card) | **Strong** (full surface) |
| Scope visibility per result | Good (tags + chip) | **Strongest** (spatial) | Strong (bar + tags + inspector) |
| Progressive disclosure into Disk/Cleanup/Plan/Activities | Weak (zoom = second grammar) | **Strong** (drill right) | Strong (page ladder) |
| Two-gate ergonomics | Mixed (dismiss semantics need exceptions) | Weak (layout exception at the worst moment) | **Strong** (read page + type dialog, two geometries) |
| Activity switching speed | Weak (re-summon) | Strong (cursor move, in-place) | **Strongest** (one key from anywhere, strip always on) |
| 80×24 / 72×20 | Strong | **Weak** (model collapses) | Strong (sanctioned drops) |
| DESIGN.md grammar fit | Mixed (inverts elevation model) | Mixed (new generic widget) | **Strong** (all catalogue) |
| §15 one-mental-model coherence | Weak | Strong | **Strong** |
| §15 DAG + plan editing | Weak | Mixed (no convergence) | **Strong** |
| §15 aggregate ⇄ per-step output | Weak | Mixed | **Strong** |

---

## 5. Decision: **Console**

Console is the only model in which every first-class pillar of CONCEPT.md
has a *native* home instead of an exception:

1. **Activities are a product pillar, not a feature** (§8.14). Persistent
   named activities with fast switching and retained scope are the hardest
   requirement in the brief. Console's docked tab strip is multiplexer-like
   continuity in DESIGN's own grammar; Lens must special-case persistence
   against its transience; Atlas can only show output while you stay in one
   place.
2. **Safety moments deserve the strongest geometry, not a layout exception.**
   Atlas breaks its columns exactly when facts need width; Lens must disable
   its dismiss instincts exactly when deliberation matters. Console gives
   gate 1 a reading surface and gate 2 a contained typing surface — two
   postures, two geometries, both catalogue-native.
3. **Plans are graphs, not scripts** (§8.15). Console's full-body plan page
   expresses branches, exclusion consequences, and convergence with rail
   glyphs and group structure; Lens abandons its metaphor to get there;
   Atlas's tree cannot show convergence.
4. **Launcher speed survives.** Home is query-first: the field is focused in
   the first frame, `Enter` always means the primary action, exact aliases
   are deterministic. Console is a launcher whose result list lives in a
   full window — the weight is in the chrome, not in the keystroke count.
5. **Nothing new must be invented.** Every Console surface is an existing
   DESIGN composition; the brainstorm budget goes into fixtures, ranking,
   and safety copy — where the product risk actually lives.

**Why not Lens** (rejected): its transient summon-and-dismiss metaphor is
genuinely excellent for the simple-action path — launch, glance, act, gone —
and it should inform how *light* home feels. But the product's two deepest
requirements (persistent activities, two-gate deliberation) both demand the
opposite posture, and every deep flow forces a metaphor-breaking zoom into
what then becomes Console with worse chrome. Chosen: its lightness; rejected:
its grammar.

**Why not Atlas** (rejected): the spatial scope compass is the single best
idea in this brainstorm — scope as *place* — and Console keeps its essence
as persistent scope tags, the breadcrumb, and explicit scope pages rather
than a column. Atlas loses on the two moments that matter most (gates,
plans), collapses exactly on the narrow remote terminals where Holla must
be most trustworthy (§8.12, §18.16), and requires a new generic widget
family before the first screen can render.

---

## 6. Console key surfaces at 120×40 (semantic sketches)

Legend: zones and information placement only. `▎` = focus, `›` = chosen,
`⠋` = live, `━` = active rule, `·` = clause join, `›` in paths = hierarchy.
Green appears only at: lockup, focus bar, primary action, active tab rule,
spinner, EDIT badge.

### 6.1 Root empty state (home)

```
row 1   holla❯  File  Go  Help              monorepo › apps › frontend · devbox · local · ⠋ discovering…
row 2   (blank)
body    ▎ Search actions, files, services…                     ← query field, focused at launch
        (blank)
        ┌ results list (~72 cols) ────────────────┐  ┌ inspector card · surface (~40 cols) ─┐
        Suggested here                            │  Review 4 modified files
        ▎› Review 4 modified files · here         │  What     open the Git changes flow
          Pull · branch is 3 behind · here        │  Target   apps/frontend · devbox
          Analyze disk usage · 12 GB artifacts    │  Runs in  ~/work/monorepo/apps/frontend
          Start frontend dev · child: frontend    │  Why      4 modified files in worktree
        (blank)                                   │  Changes  read-only · no confirmation
        Recent here                               │  Data     live
          Run tests · 2 h ago · here              │  Enter Open flow · o More actions
        (blank)                                   │
        Explore                                   │
          Tasks · Git · Files · Disk · Services · System      ← faint section, rows or segments
row 39  (blank)
row 40  Enter Run · o Actions · p Preview · ? Help · q Quit        4 suggestions · devbox
```

Notes: section headers faint; one blank row between sections; reasons and
scope tags are muted right-aligned meta; discovery spinner in the menu bar is
the one live-activity accent. Below 100 cols the inspector leaves and `p`
opens the same content as a full-body drawer.

### 6.2 Active search (`docker clean`)

```
row 1   holla❯  File  Go  Help              ~/work/scratch · devbox · local
body    ▎ docker clean▌                                 ← editing: accent underline + cursor
        (blank)
        ┌ results ────────────────────────────────────┐  ┌ inspector ───────────────────────┐
        ▎› Clean Docker completely   host · destructive│  Clean Docker completely
          Stop all containers        host · destructive│  What     stop + remove all
          Prune builder cache        host · 3.1 GB     │           containers, images,
          Compose logs · api         project           │           networks, volumes, cache
          Dockerfile                 here · file       │  Host     devbox · local
        (blank)                                        │  Why      exact intent · used 6×
        5 results · all scopes                         │  Gate     review + typed phrase
                                                        │  Enter begins review — never runs
row 40  Enter Review… · o Actions · Tab Scopes · Esc Clear
```

Notes: destructive rows stay first-class in ranking; the risk tag (error
tone) is a safety tone, not a demotion. Scope column present on every row.
Streaming inserts never move the selected identity.

### 6.3 Plan review ("Upgrade everything on this system")

```
row 1   holla❯  File  Go  Help              Plan · Upgrade everything · devbox
body    ┌ graph (~62 cols) ───────────────────────────┐  ┌ step card · surface (~50 cols) ──┐
        Preflight                                     │  03 Discover Debian upgrades
          01 ✓ Inspect host · 1.2 s                   │  Command   apt list --upgradable
        Parallel                                      │  Runs      host · no privilege
          02 ⠋ Refresh Debian metadata                │  Needs     01 Preflight
          03 ·  Inspect global mise tools             │  Unlocks   04 Review · 06 Apply
        Review                                        │  ─ output tail ─
          04 ·  Review Debian upgrades                │  12 upgradable packages …
          05 ·  Review mise updates                   │
        Apply · parallel                              │
          06 ·  Apply Debian upgrades · privileged    │
          07 ·  Upgrade selected mise tools           │
        Optional                                      │
          08 [✓] Package cleanup · Space excludes     │
        Converge                                      │
          09 ·  Final verification · waits for 06, 07 │
        (blank)                                       │
        [ Run plan… ]  [ Cancel ]    8 steps · 2 parallel branches · 1 optional
row 40  Space Exclude · ↑↓ Steps · Enter Step output · Esc Home
```

Notes: step rows use step-rail state glyphs (`·` queued, `⠋` running, `✓`
done, `–` skipped, `✗` failed, blocked = faint with a reason); group headers
carry phase and branch semantics; convergence is stated as `waits for:` meta,
never connector lines. Excluding 08 or a tool in 07 recomputes dependents in
place. `Enter` on a running step opens its activity tab — aggregate and
per-step output are one keystroke apart.

### 6.4 Activities view (`frontend dev` focused)

```
row 1   holla❯  File  Go  Help              monorepo › apps › frontend · devbox · local
row 3   1 ⠋ frontend dev   2 ⠋ api logs   3 ✓ tests · 19 s     ← tab strip, 2 rows
        ━━━━━━━━━━━━━━━━                                          accent rule under active tab
body    status strip · elevated:  frontend dev · running 4:12 · cwd apps/frontend · devbox
        insights card:            ready on :3000 · 0 errors · rebuilt 12 s ago
        ┌ viewport · frame ─────────────────────────────────────────────────────┐
          … live output, following tail; ▲ 41 scrollback cue when paused …
row 40  0 Home · 1–9 Switch · f Follow · y Copy · x Stop… · Esc Home
```

Notes: the strip appears whenever ≥1 activity exists, on every surface; tabs
carry state glyphs (`⠋` running, `✓` succeeded, `!` failed, `•` needs input).
Home is the base surface, not a tab. `x` Stop… is a confirm dialog; stopping
never loses retained output — the tab becomes `stopped` and stays until
closed.

### 6.5 Two-gate flow (review → gate)

Gate 1 — review surface (full body; reading posture):

```
row 1   holla❯  File  Go  Help              Review · Clean Docker completely
body    Action     Stop and remove all containers, images, networks, volumes, builder cache
        Host       devbox · local                     Environment  development
        Targets    14 containers (6 running) · 23 images · 5 networks · 6 volumes · cache 12.4 GB
        Includes   stopped containers · unnamed volumes · dangling images
        Excludes   — (none)
        Sequence   1 stop running → 2 remove containers → 3 images → 4 networks → 5 volumes → 6 builder
        Recovery   images and volumes permanent · no undo · containers irreversible
        Data       resolved 8 s ago · re-resolved before execution
        (blank)
        [ Cancel ]  [ Continue… ]          ← focus starts on Cancel
row 40  Enter Choose · ←→ Move · Esc Cancel
```

Gate 2 — facts dialog over the dimmed review (typing posture):

```
        (backdrop: review surface dimmed two steps; hint bar stays live)
                  ╭──────────── Prove intent ──────────────╮   elevated, centred
                  Type to confirm:
                  ▎REMOVE ALL DOCKER DATA ON devb▌          ← accent underline, hardware cursor
                  Required: REMOVE ALL DOCKER DATA ON devbox
                  (broadest actions prefix: I UNDERSTAND:)
                            [ Cancel ]   [ Execute ]        ← Execute disabled until exact match
row 40  (dialog hint layer)  Esc Cancel · Enter Choose
```

Notes: the interaction that selected the action never counts as gate 1
(`Enter` opened review; it is not a confirmation). Any material plan change
between gates or before execution re-resolves and invalidates. Cancellation
is always the default: footer status `Cancelled · nothing was executed`.

---

## 7. Surface names (module naming)

Screens / surfaces the winner implies:

- `home` — root surface: query row, results list, sections (Suggested,
  Recent, Explore, Follow-ups), discovery status.
- `inspector` — preview card (≥100) / preview drawer (<100); step-card
  variant reused by `plan`.
- `action_menu` — anchored alternates menu on a focused result.
- `args_form` — structured argument collection with validation, secret
  masking, live command preview.
- `flows/disk` — progressive disk analysis (quota meters, largest-first,
  drill-down).
- `flows/cleanup` — candidate review with freshness-aware selection,
  exclusions, dry-run parity.
- `flows/tasks`, `flows/git`, `flows/files`, `flows/services`, `flows/system`
  — domain flow pages from Explore/search.
- `plan` — graph review + aggregate execution (grouped step list, step card,
  exclusion recompute).
- `activity` — one activity: status strip, insights card, text viewport.
- `activity_strip` — the docked tab strip (catalogue tabs widget; screen
  chrome, not a page).
- `review` — gate-1 reading surface (destructive, trust, privileged modes).
- `gate` — gate-2 typed-phrase dialog (facts dialog variant).
- `confirm` — one-gate dialog for bounded mutations.
- `switcher` — optional activity list picker (`Ctrl+G`-style) for >9
  activities or mouse-free discovery.

## 8. Esc ladder per surface

| Surface | Esc step 1 | Esc step 2 | Esc step 3 |
|---|---|---|---|
| home (query editing) | clear query text | — (empty query) quit to shell* | — |
| home (list focus) | focus query field | quit to shell* | — |
| inspector drawer (<100) | close drawer → list | — | — |
| action_menu | close menu → owner row | — | — |
| args_form | revert field edit | cancel form → home | — |
| flows/* | exit inner mode (filter/selection) | leave flow → home | — |
| plan (review) | step card focus → graph | home (draft persists in session) | — |
| plan (executing) | step card → graph | home — plan keeps running as activities | — |
| activity | clear viewport selection | home (`0` equivalent) — activity keeps running | — |
| review (gate 1) | activate Cancel → back to origin surface | — | — |
| gate (gate 2) | revert phrase field edit | cancel dialog → dismiss review | — |
| confirm | cancel | — | — |

\* Quit with activities running raises a confirm dialog
(`2 activities running · Quit anyway?`). `Ctrl+C` always quits after that
confirmation; on executing plans it first offers cancellation. Every surface
contributes its hint layer; the hint bar never moves.

## 9. Three strongest risks of the winner, and mitigations

1. **Heaviness: full-screen console reads as an IDE, not a launcher.** If the
   first frame is not query-focused with plausible suggestions, the product
   fails its promise ("few keystrokes", §17). Mitigations: query field
   focused at launch in every scenario; Suggested section rendered in the
   first paint with progressive deepening (discovery status visible, never
   blocking); `Enter` is always the primary action; exact aliases are
   deterministic regardless of ranking; capture review at 80×24 explicitly
   judges "keyboard destination in under a second".
2. **Key-space collision: an always-editing query swallows global chords.**
   Digits, `[` `]`, `q`, `x` are text while editing, so tab-jump and quit
   grammar must not fight typing. Mitigations: DESIGN's own rule — chords
   ignored while a control edits; on home, activity switching uses
   `Ctrl+1…9` / clickable strip / `0` to leave an activity; digits jump only
   on non-editing surfaces (activity pages, strip focus); every collision
   documented in the affected surface's hint layer so discovery never
   requires memorisation.
3. **Streaming discovery vs stable selection (§5.5.9, open question §18.5).**
   Late-arriving results can reorder the list under the cursor and destroy
   trust. Mitigations: results are identity-keyed and selection follows
   identity, never index; order freezes after the first navigation keypress
   (new arrivals append below, badged); stale/cached rows marked per
   contract item 6; reorder policy covered by app tests with scripted
   streaming fixtures.

---

## 10. Open questions handed to the build phases

- Exact scope-tag vocabulary per ring (`here / parent: X / child: Y / host /
  personal`) — settle in P2 with fixtures.
- Whether `switcher` is needed in P4 or the strip suffices at fixture scale.
- Review-surface content cap for very large target sets (Docker 100+):
  scrollable list vs truthful summary + count — decide in P3 against the
  facts-dialog precedent.
- Follow-ups section lifetime on home (session-only vs persisted) — P2/P6.
