# Holla working notes — decisions and open questions

Running log. Newest at bottom per section. Status: [x] accepted, [ ] open.

## Decisions

- [x] App shell mirrors jackin_preview: `main.rs` (arg parse: `--color`, `--scenario`,
  `--motion full|reduced|paused`, `--frame N`), `scenario.rs` (`Scenario` enum +
  `Motion`), `app.rs` (`App` owning route/modal-stack/focus/ring/hits/status),
  `screens/` behind a `Screen` trait with `Cx`/`Request`/`Go`, `domain/` typed
  model + `fixtures.rs`, `sim/` for live-feeling behavior. Env fallback
  `HOLLA_NO_MOTION` (JACKIN_NO_MOTION spirit).
- [x] Brand mark: ` holla❯ ` lockup (DESIGN.md: brand lockup anatomy).
- [x] Scenario names per GOAL.md: `first-use`, `rust-dirty`, `monorepo-root`,
  `monorepo-child`, `docker-cleanup`, `disk-cleanup`, `upgrade-plan`,
  `activities-multi`, `remote-host`, `launch-failure`, `hard-cases`.
- [x] Capture prefix `h_*`; tmux key name `Escape`.

## Interaction model

- [x] **Winner: Console** (brainstorm notes `interaction-models.md` §5). One home
  surface (menu bar + always-ready query + ranked results + inspector), flow
  pages, plan page, activity tab strip, review (gate 1 full-body reading
  surface) + gate (gate 2 facts dialog with typed phrase). Rejected: Lens
  (transient overlay fights persistent activities + gates), Atlas (miller
  columns collapse at 72–80 cols, no DAG convergence, gates need 66 cols).
- [x] Two-gate mapping: gate 1 = full-body `review` surface (reading posture,
  width for real facts); gate 2 = `Dialog::facts` with typed acknowledgement
  (GOAL §2 default). Bounded mutation = one `confirm` dialog. Read-only runs
  direct.
- [x] Activities: docked tab strip under menu bar (catalogue tabs widget,
  `tabs-height` 2) visible whenever ≥1 activity exists; home is the base
  surface, not a tab; `0` returns home; digits jump on activity pages.
- [x] Chrome: menu bar row 0 (lockup ` holla❯ ` + menus + crumb `path · host ·
  role` right), activity strip rows 2–3 when activities exist, body, blank,
  hint bar (layer precedence: modal › menu › screen › fallback, centered).
  No global StatusBar; activity pages own a status strip in-body.
- [x] Module layout mirrors jackin: `main.rs scenario.rs clock.rs app.rs
  domain/{mod,fixtures,...}.rs sim/{mod,world,...}.rs screens/{mod,home,
  review,plan,activity,flows/*,modals}.rs app_tests.rs app_tests_chrome.rs`.

## P0 session decisions (2026-09-06)

- [x] Home key model: query always armed (`is_editing() = true`, EDIT badge
  always on). Plain chars filter; bare `?`/`q` on an EMPTY query are chrome
  (help / quit-confirm); Enter runs focused row (or top match when query
  focused); ↑/↓ move focus query↔rows; Esc clears query then is consumed —
  home is base, Esc never leaves; F1/F10/Ctrl+Q chrome work while editing.
- [x] EDIT badge speaks for the topmost context only: when a modal is open,
  only the modal's editing state counts — home's always-on editing must not
  leak into the modal hint layer.
- [x] Info dialogs (help/about): single Close button (pop auto-added Cancel),
  `cancel_index = Some(0)` so Esc closes.
- [x] Quit dialog: `Dialog::confirm` with `initial_focus` forced to Cancel
  (cancellation is the default for a destructive-ish chrome action).
- [x] `World::seek` forces `clock.running = true` during fast-forward and
  restores after — else `advance` no-ops and seek loops forever.
- [x] Capture harness: `export PY=/tmp/holla-venv/bin/python` (fresh venv,
  Pillow; tools/env.sh old path dead). `cargo build --bin holla` before
  captures — `cargo test` does not relink the binary the tmux session runs.
  tmux `send-keys C-q` swallowed by flow control; quit dialog captured via
  bare `q`, Ctrl+Q path covered by unit test.
- [x] Borrow splitting: no `self.screen()` accessor; screens are direct
  fields (`self.home.on_key(...)`) so field borrows split (jackin pattern).

## P1 session decisions (2026-09-06)

- [x] Domain model: `domain/{mise,git,docker,disk,pg,ssh,github,debian,
  ranking,activity}.rs` — plain data, no process spawning. Fixture contract
  lands whole per file; module-level `#[allow(dead_code)]` until P2+ renders.
- [x] Fixture home pinned: `HOME = "/home/dev"`; display paths keep `~`,
  typed confirmation phrases (P3) expand via `expand_home`.
- [x] Discovery is per-domain and progressive: mise 600, git 900, ssh 1200,
  github 1600, docker 2400, pg 2800, disk 3200 virtual ms. Failures settle
  (known, not pending) — `discovering()` ends; hard-cases fails docker only.
- [x] `World::tick` and `World::seek` share `step()`; seek forces the clock
  running during fast-forward.
- [x] rust-dirty is pure-behind (ahead 0, behind 3) so the vocabulary string
  `branch is 3 commits behind` survives; diverged/detached covered by git
  unit tests instead.
- [x] `Domain::label` reserved for per-domain scanning notes in P2.

## P2 session decisions (2026-09-06)

- [x] Action model: `Action{id,title,kind,scope,scope_label,reason,risk,
  availability,target,command,workdir,long_running,keywords}` with
  `Scope{Here,Project,Workspace,Host,Personal}` weights 100/80/60/40/20,
  `Risk{ReadOnly,Bounded,Broad}` (confirmation text per risk),
  `Availability{Ready,NeedsTrust(path),Blocked(reason)}`,
  `ActionKind{Task,Git,Plan,Connect,Flow,Clone,System}`.
- [x] Ranking formula: scope weight + pin(+100) + alias(+90) +
  min(usage,10)×2 + urgency(+30); total deterministic sort (score desc, kind
  asc, title asc). Sections: Suggested here (top 4), Recent here
  (`recent:`-ids), Explore (rest). Urgency keywords: unhealthy/modified/
  behind/untrusted/% full/detached — can elevate nonlocal into Suggested with
  explicit scope tag, never above pin/alias (resolves open question #6).
- [x] Scope switching key: **Ctrl+S chord** cycles None→Here→Project→
  Workspace→Host→Personal→None (resolves open question #1; chord, not
  focus-stop chip). Esc clears scope filter before clearing query; header
  segment `scope: X` priority 6 + in-field label.
- [x] Scope-tag vocabulary (resolves open question #2): ring labels
  `here / project / workspace / host / personal`; place label is the relative
  dir (`apps/frontend`) or host name; pins show `pin`; ssh rows `ssh config`.
- [x] Search haystack includes `keywords` (task's real command line, e.g.
  `vite --port 5199`) — never displayed, only matched. Alias expansion: query
  `gs` matches the row whose command expands from the alias.
- [x] Row layout: reason owns the right edge; scope tag gets leftover budget
  (≥10 cols → `truncate_middle`, else dropped); `▲` Warning after title for
  NeedsTrust/Blocked. Crumb drops first at narrow widths (header priority).
- [x] Preview = `Dialog::facts` answering §7: Will happen / Target / Why
  recommended / Will change / Freshness / Confirmation + command code line;
  Close + Run (simulated) via `with_actions`; Blocked rows get disabled
  secondary.
- [x] Clone flow = §6.7 structured-arguments instance: searchable
  `PickerModal` (repos) → facts review (Account/Owner/Protocol/Destination/
  Primary branch/Fork) → Clone (simulated).
- [x] `long_running` ReadOnly runs start a named Activity (scope = workdir);
  ReadOnly one-shots report `Would run: {cmd} · simulated, nothing executed`;
  launch-failure scenario + Task kind → Failed activity with honest output.
- [x] PickerModal payloads travel via `ModalResult::Custom(payload)`; the
  screen mutates the world in `on_modal` (CustomModal world is read-only).

## P3 session decisions (2026-09-06)

- [x] Plan domain: `Plan{action_id,title,host,phrase,will_change,steps,
  effect,ran}`, `PlanStep{id,title,command,branch,deps,optional,fails,
  ok_lines,fail_lines,state,lines}`, `StepState{Pending,Excluded,
  PolicySkipped,Succeeded,Failed,Skipped}`. Deps are always earlier indexes;
  `run()` is deterministic declaration-order execution; failure/skip
  propagates to dependents as `Skipped("needs X")`.
- [x] Exclusion: Space toggles optional steps; `blocked_by_exclusion` is
  computed (never stored) so restore is free; required steps refuse with a
  status; policy-skipped steps (disk: active-today artifacts) are shown,
  never removable.
- [x] Gate 2 = `Dialog::facts` with `Some(token)` — confirm disabled until
  `input.text().trim() == token`. Ack input armed via `begin_edit()` at
  construction **holla-side** (lib default unchanged: tablepro's token gates
  keep navigation-mode start; arming on open is holla's convention because
  the phrase is the gate's whole point).
- [x] Phrases bound to target: `REMOVE ALL DOCKER DATA ON devbox`,
  `DELETE GENERATED ARTIFACTS ON devbox`, `RESTART PAYMENTS ON prod-eu-1`;
  broadest (upgrade-everything) takes the `I UNDERSTAND:` prefix.
- [x] Production raises risk: docker restart on `Environment::Production`
  becomes Broad (plan + two gates); elsewhere Bounded (one confirm).
- [x] Honest world effects after a run (`apply_effect`): restarted container
  heals (the urgency row vanishes from home); cleanup removes what it
  claimed (partial on failure: web/cron gone, payments-old kept); Debian
  pending drops to held, reboot stays required; mise tools flip to Active.
- [x] Docker cleanup failure is fixture truth: `payments-old` has a bind
  mount still registered → removal fails → images/volumes skip; builder
  cache (parallel branch) succeeds.
- [x] Plan glyphs: `·` pending, `−` excluded/skipped, `▲` blocked-by-
  exclusion, `✓` succeeded, `✗` failed (✗ added to the vocabulary — failure
  must be unmistakable; DESIGN.md update lands in P6).
- [x] Plan screen: title + posture line (drops when it would overprint),
  4 facts, DAG rows (glyph, title, `· branch`, right state note), output
  pane for the focused step post-run, scroll keeps the focused row visible.

## P4 session decisions (2026-09-06)

- [x] Strip is rendered manually (not the Tabs widget): per-state tones and
  glyphs (`● ✓ ✗ … −`), `{glyph} {n} {name}` tabs, active tab inverted in
  its state tone, `0 home` marker at the right end. Tabs widget is built
  for document tabs, not a status strip.
- [x] Strip docks at row 1 (the old blank separator row) whenever ≥1
  activity exists — no layout shift for the body; clickable from every
  route via `strip.act` child ids.
- [x] Keyboard reachability: **Ctrl+A** is chrome (like F10) and cycles the
  strip from any route — first press enters the first activity, next
  presses advance, wrap-around. Digits `1–9` jump only on activity pages;
  `0` returns home. Home digits still filter the query (P0 model holds).
- [x] Viewing state (`ActivityScreen.current`, per-activity scroll map)
  lives in the screen, not the World — the world stays fixture truth.
- [x] Merged logs (§12): `docker.logs` starts a `container logs` activity
  and navigates straight to its page; lines are `{service:<8} | …`,
  interleaved round-robin, per-service tone by deterministic name hash;
  only Running containers speak. Page detects merged lines by ` | ` and
  colors the prefix.
- [x] Capture race: after Enter-driven navigation the shot must wait
  ~0.3–0.5s or the pane still shows the previous route (seen twice;
  behavior verified correct via app tests + settled recaptures).

## Open questions

- [ ] Whether `switcher` picker needed or strip suffices at fixture scale — P4.
- [ ] Review-surface content cap for very large target sets: scrollable list
  vs truthful summary + count — decide in P3.
- [ ] Follow-ups section lifetime on home (session-only vs persisted) — P2/P6.
- [ ] §9 rank 3 live-urgency vs §5.5.3 local-over-global tension (synthesis
  found contradiction): resolve as — urgency can elevate a nonlocal item into
  Suggested, but scope tag + reason must be explicit; never above an exact
  alias or pin.

## Fixture vocabulary (semantic facts required by concept)

From CONCEPT.md + references (each must appear as a reason/fact somewhere):

- `branch is 3 commits behind`, `4 modified files`, `used 6 times in this
  project`, `defined by project task runner`, `service is unhealthy`,
  `12 GB generated artifacts`, `available on this host` (§9)
- Typed phrases: `DELETE EVERYTHING IN /work/scratch`,
  `REMOVE ALL DOCKER DATA ON devbox`, `RESTART PAYMENTS ON prod-eu-1`,
  prefix `I UNDERSTAND:` for broadest actions (§10)
- Docker classes: containers, images, networks, volumes, system data, builder
  cache; disk accounting via `docker system df` (tech-stack §3)
- Cleanup families: Cargo target dirs, Gradle builds + `.gradle`, Node
  `node_modules`/dist, package caches, logs, temp data (mole §families)
- Freshness facts: `inactive 31 days`, `active today`, `activity unknown`;
  sizes `8.4 GB`, `12.7 GB`, `3.1 GB`, `900 MB` (mole example)
- mise: tools active/missing/outdated, tasks incl. namespaced
  `//projects/frontend:build`, trust state per exact file (tech-stack §1)
- Git: dirty/behind/diverged/detached, worktrees, submodules vs children,
  primary branch resolution, never hard-coded `main` (tech-stack §2)
- pg: blocker tree, cancel before terminate, revalidate PID+query (§8.7)
- SSH: literal aliases from `~/.ssh/config`, jump chain, identity filenames,
  host-key policy, multiplexing state (§8.10)
- Upgrade plan: preflight → Debian metadata → review → apply; mise inspect →
  upgrade; optional cleanup; final verification (§8.16)
- Activities: named, scoped, states running/waiting/succeeded/failed/detached,
  retained output, multi-service log merging (§8.14)

## P5 session decisions (2026-09-06)

- **Pin outranks alias + urgency**: pin weight 150 so a pin always beats
  alias (90) + live urgency (30) combined — pins are the strongest explicit
  user intent; urgency may elevate but never above a pin.
- **Hidden rows resurface on exact-command query**: hide removes a row from
  passive ranking, but typing the row's full command exactly brings it back —
  that is the only path to Unhide / Reset ranking. Documented on the menu
  item (`gone from this folder's list · reset restores`).
- **`reset_at` clears pin + alias + hide, not usage**: usage is history
  ("happened" truth), not a preference; resetting ranking never rewrites the
  past.
- **Dialog width is content-driven, set per dialog** (`Dialog.width` is
  pub): pg tree 88, monitor 84, ssh 76 — the policy lines are the point of
  those dialogs, truncation would hide the decision-relevant fact.
- **pg dialog initial focus = Cancel blocker**: `Dialog::facts` focuses its
  confirm button; on the lock tree that is the policy-recommended action
  (cancel before terminate). Terminate is one Right away and revalidates.
- **git.sync policy-skipped steps read `required`**: `policy_skipped()` sets
  `optional = false` so the reviewer cannot exclude them; the row shows
  `required · detached at a1b2c3d · skipped`. Accepted pairing: "required"
  here means "not excludable", and the skip reason sits in the same row.
- **Quit confirm on sensitive hosts names the identity**: title
  `Quit holla on prod-eu-1?`, body leads with `◆ prod-eu-1 · ssh ·
  production` — leaving a production box must say which box.
- **Host metrics are integers** (`load_x100`, MB, days): keeps `Host`
  `Eq` for fixture determinism; rendering formats `3.10` from 310.
- **Enter on `flow.monitor`/`pg.locks` opens their surfaces directly** —
  dedicated flows bypass the generic preview (§8.6/§8.7 are the preview).

## P6 session decisions (2026-09-06)

Full-pass fixes driven by the independent visual critique of the 132-PNG
matrix (two critic agents, 4 sizes × 3 color levels × 11 scenarios):

- **Row reason needs a real gap or yields**: a row's reason (right edge)
  renders only when two empty cells exist after the title
  (`row.width > rw + 24 && row.right() > x + 2 + rw`); otherwise the reason
  is dropped for that row and the title ellipsizes with `…`. A flush
  title/reason collision mid-word reads as a rendering bug; a missing
  reason reads as a narrow terminal.
- **The breadcrumb truncates before chrome does**: the header pre-computes
  `crumb_budget = rest − (identity + level + "? help" + 8)` and
  middle-truncates the crumb when over budget, instead of letting
  segment-priority shedding drop color·size and ?help while a long crumb
  survives. Identity and chrome are the constant frame; the path is the
  variable part.
- **Launch-failure seeds its failure**: the scenario is indistinguishable
  from rust-dirty until you act, which made the matrix shot a false
  positive for "healthy repo". The fixture now seeds a failed
  `seed db` activity (`psql: connection … failed · exit code 1`), so the
  activity strip shows `✗ 1 seed db` from the first frame — the scenario's
  truth is visible without interaction.
- **Warning statuses carry `▲` like errors carry `!`**:
  `keyhint::render_aligned` prefixes a Warning status with a bold `▲` in
  every color level, mirroring Error's bold `!`. In mono the tone vanishes;
  the glyph is what keeps a failure/warning status from reading as plain
  text dimmer than the rows above it. Recorded in DESIGN.md's glyph table
  (status row), not as a holla-local special case.
- **Critique misread dismissed**: "docker-cleanup hides its blocked row"
  — discovery succeeds in that scenario by design (the hard-cases scenario
  owns the discovery failure); the blocked *action* row the critic wanted
  is the discovery-failure row, which lives in hard-cases.
- **Cursor over placeholder glyph accepted**: the text cursor sitting on
  the input placeholder's first cell is standard terminal cursor
  behavior, not a defect.
- **Header budget counts the strip's real overhead**: `segments::render`
  charges 2 cells per segment plus a flat 2, so the crumb reserve is
  `identity + level + "? help" + 10` for four right segments — the first
  pass under-reserved by 2 and the color·size chip still dropped at 120
  columns. Verified visually at all four sizes.
- **In-body discovery failure carries `▲` too**: the home body's
  `docker: discovery failed` line is Warning-toned; in mono the tone
  vanishes, so the line now leads with `▲`, matching the footer-status
  convention.
