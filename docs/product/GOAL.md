# GOAL: Design and build Holla — a context-adaptive action launcher for the terminal

Holla is a NEW application in this repository, built exactly the way `tablepro`
and `jackin-preview` were built: a binary in `src/bin/`, on the Junie TUI design
system (`DESIGN.md`), with deterministic fixtures, capture-harness evidence, and
an iterative visual review loop.

Everything about what Holla IS lives in `docs/product/CONCEPT.md` and
`docs/product/references/`. Everything about how it must LOOK and FEEL lives in
`DESIGN.md`. Everything about HOW to run this work lives in this file and in the
original design-system goal this repo descends from
(https://raw.githubusercontent.com/donbeave/terminal-components-claude/e43cf670d6cb793e5761819e8778600797bbf1aa/GOAL.md).

---

## 0. FIRST: RESEARCH THE REFERENCE

Do not write code before the reading and research gates below pass.

Authoritative, in priority order:

1. `docs/product/CONCEPT.md` — the product concept. The single source of truth
   for what Holla is: vision (§1), problem (§2), product model (§4), context
   model and rings (§5), root experience (§6), experience states (§7), core
   domains (§8), personalization/ranking (§9), safety and two-gate confirmation
   (§10), plan-based workflows (§8.15), activity multiplexing (§8.14),
   boundaries (§14), open questions (§18).
2. `docs/product/references/` — pattern notes supporting the concept:
   - `universal-launcher-patterns.md` — one root surface, useful empty state,
     mixed result types, primary action + alternatives, fuzzy + deterministic
     aliases, user control over ranking, Holla expands local → global.
   - `terminal-workflow-patterns.md` — search channels, contextual history,
     progressive disk analysis, destructive maintenance as first-class,
     hierarchical project discovery, activity multiplexing, dependency-aware
     plans, system-upgrade orchestration.
   - `context-adaptive-product-principles.md` — context before catalog,
     recommendations before categories, intent before syntax, resources plus
     actions, here-first-then-expand, explain relevance, separate
     relevance/confidence/risk, bind confirmation to target, stream without
     disruption, preserve active work, represent compound intent as a plan.
   - `technology-stack-workflows.md` — the priority stack: mise, Git/GitHub,
     Docker, `btm`, disk, PostgreSQL/`pg_activity`, Rust/nextest, SSH, artifact
     cleanup (Rust, Gradle, Node). Realistic command examples and required
     understanding per domain.
   - `mole-disk-cleanup-patterns.md` — progressive analysis, cleanup families,
     candidate facts, freshness-aware defaults, dry-run parity, recovery
     modes, audit history, editable cleanup plan.
3. `DESIGN.md` — the Junie TUI design system. Tokens, text ladder, glyph table,
   interaction grammar, focus model, state grammar, component catalogue,
   composed patterns, agent guardrails. This constrains every pixel.
4. Recipe precedents in code:
   - `src/bin/jackin_preview/` — the deterministic-preview recipe: `Scenario`
     enum + `Motion full|reduced|paused` + `--frame N` (scenario.rs), an
     `arbiter` for cross-cutting rules, `sim/` (in-memory world: pty, provider,
     onepassword, launch, changes), `domain/` (typed model + fixtures),
     `screens/` per surface, chrome composed from the shared `junie_tui`
     widgets (`widgets::brand` lockup, `widgets::menu` menu bar + context
     menus, `widgets::statusbar`, `widgets::hintbar` hint layers),
     `app_tests.rs` + `app_tests_chrome.rs`.
   - `src/bin/tablepro/` — the workbench recipe: `Application` trait wiring in
     main.rs, `Screen`/`Modal`/`Request`/`Cx` shell in app.rs, `WidgetId::of`
     constants, focus ring + hit registry (core/focus, core/hit), hint bar
     (`widgets::keyhint`), destructive facts dialog with typed acknowledgement
     (`Dialog::facts(…, token, confirm)` in tablepro app.rs), responsive
     identity strip (prioritised segments).

The concept documents deliberately do NOT prescribe visuals. The design system
deliberately does NOT dictate product structure. The design work is the
original synthesis between them. Consequences for you:

- CONCEPT.md and `references/` supply semantic requirements only. Their
  examples, names (Root, Suggested here, Preview, Action Panel, Gate), and
  notations are product ideas and decisions, never wireframes, layouts,
  components, or display order (CONCEPT.md "How to interpret", §19 handoff,
  final interpretation rule).
- DESIGN.md supplies the visual language only. It does not tell you what
  screens Holla has or how the launcher is arranged.
- Nothing in this goal file is a wireframe either. Phase names describe what
  must be TRUE at each gate, not what must be drawn. Brainstorm at least three
  distinct interaction models (§3), pick one with written justification, and
  derive every screen from that model. Impress with a better interface, not
  with compliance to an implied sketch.

## 1. OBJECTIVE

Design and implement `holla`: a runnable terminal application in this repo
(`src/bin/holla/`, `[[bin]] name = "holla"` in Cargo.toml) that demonstrates
the complete Holla product concept — context, recommendations, search across
domains, scope navigation, previews, safety gates, plans, and activities — as a
deterministic, fully simulated preview, in the Junie design system.

Like `jackin-preview`: every external system (mise, git, gh, docker, btm,
pg_activity, ssh, the filesystem) is a deterministic in-memory simulation.
Nothing real is ever executed. No real command in the stack is ever spawned by
the holla binary. Simulation is the product contract of this repository.

Product truth to preserve while simulating (from CONCEPT.md):

- The current working directory is the primary contextual object; context rings
  Here → Project → Workspace → Host → Personal expand outward only as needed
  (CONCEPT.md §5).
- Capability ≠ action ≠ recommendation (§2.4). Every recommendation carries a
  reason ("branch is 3 commits behind", "12 GB generated artifacts").
- Search is one interaction across actions AND resources, with type and scope
  visible per result (§6.3).
- Exact aliases beat learned ranking; ranking explains itself and is
  user-correctable (§6.6, §9).
- Destructive actions stay discoverable and recommendable; risk changes
  treatment, never availability (§2.5, §10). Two-gate confirmation with a
  target-bound typed phrase (`DELETE EVERYTHING IN /work/scratch`,
  `REMOVE ALL DOCKER DATA ON devbox`).
- Compound intents become reviewable dependency-graph plans with optional
  branches, parallelism, failure propagation, per-step output (§8.15).
- Long-lived work becomes named activities with retained output and fast
  switching — a tab-capable multiplexer-like model (§8.14).
- The priority stack (mise, Git/GitHub, Docker, btm, disk, PostgreSQL,
  Rust/nextest, SSH, stack cleanup) dominates discovery and journeys (§8.0,
  technology-stack-workflows.md §12).

## 2. DESIGN SYSTEM NON-NEGOTIABLES (from DESIGN.md)

- All color through `junie_tui::theme` state resolvers. Never an RGB literal in
  app code. Green only: focus `▎`, primary action, chosen marker on the focused
  row, active document tab, EDIT badge, live activity. Highlight blue only for
  anchored-menu cursor rows. Red/amber only as safety tones.
- State is geometry before color: `▎` + bold focus, one-plane hover lift,
  marker glyphs, underline + hardware cursor + badge while editing. Every
  state must survive monochrome.
- Reuse the glyph table and contextual letters (`s` sort, `f` filter, `p`
  preview, `x` close, `u` undo, `y` copy). No new glyph meanings.
- State grammar for every screen: empty (EmptyState), loading (spinner),
  loaded, partial (`↓ N loaded · Enter fetches more`), no matches, recoverable
  error (`!` + message in place), failed operation, read-only, disabled,
  destructive (Dialog::destructive on Cancel), busy, success (footer status
  4–5 s).
- Shell: one-row header/menu bar with the brand lockup ` holla❯ `, blank row,
  body, blank row, hint bar. HintBar layer precedence; modals contribute
  layers, never draw their own. StatusBar for surface state. Minimum 72×20
  with the too-small notice; responsive = prioritised dropping, never scaling.
- Two-gate destructive flow maps onto the existing facts dialog + typed
  acknowledgement by default (design system native; TablePro precedent) —
  or onto an equally deliberate composition of existing catalogue widgets if
  the chosen interaction model makes a stronger one; justify the choice in
  the design note.
- Progress bars, step rails, quota meters, pickers, context menus, tabs — use
  the catalogue before inventing anything. A new GENERIC widget needs a
  showcase page in the same change.
- Monospace grid; sentence case; `·` joins clauses, ` › ` hierarchy, `…`
  truncation, en dash ranges, past-tense status sentences.

## 3. RESEARCH AND BRAINSTORM PHASE (subagents, parallel)

Use subagents aggressively before converging. Delegate with explicit,
disjoint scopes; the primary agent owns synthesis and final consistency:

1. CONCEPT SYNTHESIS — read CONCEPT.md + all references/ files; produce the
   constraint list any Holla design must satisfy (scope model, ranking,
   safety, plans, activities), each with a citation.
2. INTERACTION-MODEL BRAINSTORM — produce at least three distinct
   terminal-native representations of the root experience (e.g. command-palette
   root + drawer flows; split root list/preview; full-screen context board with
   palette overlay), each evaluated against: empty-state usefulness, scope
   visibility, progressive disclosure into Disk/Cleanup/Plan/Activities,
   two-gate safety ergonomics, activity switching, narrow-terminal behaviour.
   Compare honestly against CONCEPT.md §15 exploration questions and pick ONE
   coherent direction with written justification.
3. RECIPE AUDIT — extract the concrete app-assembly recipe from
   jackin_preview and tablepro (module layout, App/Cx/Request wiring, chrome,
   scenario/motion/frame determinism, test + capture patterns) into a
   checklist the build phases must follow.
4. VISUAL CRITIQUE (after the first coherent screens exist) — review rendered
   captures independently; judge hierarchy, density, one-hue discipline, state
   legibility, and "does this feel like Holla per CONCEPT.md §16 quality bar".

Record accepted decisions and open questions as you go (state file or goal
notes at the executor's discretion). Do not blindly merge conflicting
recommendations.

## 4. BUILD PHASES (each ends in a gate; captures are the evidence)

- P0 Recipes and skeleton — Cargo bin `holla`, App/runtime wiring per the
  recipe audit, chrome shell (menu bar + lockup, hint bar, status bar,
  too-small notice), `--color`, `--scenario`, `--motion`, `--frame` flags
  (with a `HOLLA_NO_MOTION` env fallback in the `JACKIN_NO_MOTION` spirit).
  Gate: skeleton captures at 80×24/100×30/120×40/160×50 look like the family.
- P1 Fixture world — `domain/` typed model + `domain/fixtures.rs`-style
  fixtures for the priority stack: a Rust workspace monorepo with mise tasks
  and child projects, git states (dirty, behind, diverged, detached, worktrees,
  submodules), docker/compose state, disk candidates with candidate facts,
  pg activity, ssh config, GitHub account/org/repo state for the cloning
  journey, ranking memory (pins, aliases, per-path usage), activities. `sim/`
  only if a live-feeling behavior needs it. Gate: every scenario reproducible
  byte-identical.
- P2 Root experience — empty state that communicates the §6.2 concepts
  (Suggested here, Recent here, Explore, discovery status) in whatever
  grouping and form the chosen model serves best, cross-domain search with
  mixed result types, preview
  answering CONCEPT.md §7 questions, primary action + alternatives, scope
  navigation Current/Parent/Children/System with explicit scope on every
  nonlocal result, structured argument collection (§6.7). Gate: journeys from
  CONCEPT.md §12 (Rust project, monorepo
  children, nested child with parent ecosystem, mise trust and child-task
  execution) walk end-to-end.
- P3 Safety and plans — preview/trust, one-gate bounded-mutation confirmation,
  two-gate broad-destructive flow (facts + typed target-bound phrase, with the
  `I UNDERSTAND:` prefix for the broadest actions per CONCEPT.md §10),
  revalidation on plan change and immediately before execution, and at least
  two plan graphs: complete Docker
  cleanup (technology-stack-workflows.md §3) and "upgrade everything on a
  Debian host" (CONCEPT.md §8.16) with parallel branches, optional exclusion,
  failure propagation, per-step output. Gate: two-gate captures; invalid
  phrase cannot proceed; exclusion recalculates dependents.
- P4 Activities — named activities with retained output, state
  (running/waiting/succeeded/failed/detached), fast switching, return to root
  without losing state; multi-service log merging (CONCEPT.md §12 follow-logs
  journey). Gate: switching away and back preserves output and scope.
- P5 Depth + hard cases — disk progressive analysis with freshness-aware
  selection and dry-run parity; git-across-children bulk plan with per-project
  results and primary-branch resolution (never hard-coded `main`); GitHub
  clone review flow (account, org, protocol, destination, primary branch);
  pg blocking
  tree with cancel-before-terminate revalidation; SSH resolution preview;
  system snapshot with the `btm` handoff recommendation (§8.6); remote-host
  identity and stronger confirmation; ranking controls
  (pin/alias/hide/reset + reasons); hard-cases scenario (long labels, missing
  data, discovery failure, narrow width). Gate: hard-cases captures legible.
- P6 Full pass — all scenarios × 4 sizes × color levels (truecolor/256/mono),
  independent visual critique, fixes, README + design notes documenting the
  chosen interaction model and any new reusable patterns added to DESIGN.md.

Scenarios at minimum (naming convention like jackin): `first-use` (empty dir),
`rust-dirty`, `monorepo-root`, `monorepo-child`, `docker-cleanup`,
`disk-cleanup`, `upgrade-plan`, `activities-multi`, `remote-host`,
`launch-failure`, `hard-cases`. Each scenario must demonstrate empty, loading,
partial, success, failure, and dangerous states somewhere in its flows.

## 5. QUALITY BAR

Reject the result if it is a generic fuzzy-finder with a green theme. It must
demonstrate: opening without typing already suggests a plausible next action
with a visible reason; scope is unmistakable on every result; a destructive
action is reachable by search and still deliberate to execute; plans read as
graphs, not scripts; activities survive navigation. Same family feel as
tablepro and jackin-preview: a reader finds the keyboard destination in under
a second, the only green is where DESIGN.md allows, nothing framed that a
blank row could separate.

## 6. VERIFICATION

- `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings`;
  `cargo test` (app tests + scenario/motion determinism tests + plan-graph
  logic tests).
- tuisnap baseline frames for every scenario at 80×24, 100×30, 120×40,
  160×50, plus mono (`tools/tuisnap_baseline.sh` → `shots/tuisnap/`;
  `holla_<scenario>_default_<size>_<color>` captures). Review
  `shots/tuisnap/report.html`, then `tuisnap accept --store shots/tuisnap
  --all`. A change is not done until the frame has been looked at.
- Journeys demonstrable: CONCEPT.md §12 Rust project, monorepo root/child,
  mise trust, git across children, docker cleanup, disk cleanup, upgrade
  plan, activities, remote host.
- No panic paths; Esc ladder defined for every surface; focus ring
  deterministic; modal containment verified.

## 7. BOUNDARIES

CONCEPT.md §14 holds. Holla in this repo is NOT: a shell replacement, an
autonomous cleanup daemon, a command encyclopedia, a real executor of any
stack command, a macOS-only mental model, or a product needing configuration
before usefulness. AI assistance only with provenance/preview/scope intact;
deterministic local behavior is the default. Do not research or imitate a
named launcher product; derive the design from CONCEPT.md and DESIGN.md only.

## 8. DELIVERABLE

A runnable `holla` binary indistinguishable in craft from tablepro and
jackin-preview; deterministic scenarios with tests and captures; DESIGN.md
updated for any convention added on purpose; a short design note recording the
chosen interaction model, rejected alternatives, and how each CONCEPT.md
product principle is satisfied.
