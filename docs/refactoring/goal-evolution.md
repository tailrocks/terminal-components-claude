# Evolution of the Refactoring Goal

Compiled 2026-09-12 from repository history (read-only; nothing here reflects working-tree edits).

Sources:

- `REFACTORING_GOAL.md` at commits `e48137f1`, `dce91d51`, `2d81eec4`, `25ea92b0`, `efada044`, `d4715f8e`
  (chronology established via `git log --all --oneline -- REFACTORING_GOAL.md` and commit dates), plus `main:REFACTORING_GOAL.md`
  (`main` tip at time of writing: `7b27732a`, 2026-09-10, "Merge pull request #1 from donbeave/codex/main-holla-integration").
- `main:GOAL.md` (the successor execution goal referenced by `efada044`).
- `REFACTORING_STATE.md` history on `main`: 87 commits (`git log --follow --oneline main -- REFACTORING_STATE.md`),
  sampled at 13 points: `2d81eec4`, `d5e7075f`, `596a4706`, `8ed714ba`, `ad94d12a`, `14bca4a3`, `a1759b2a`, `07bb7193`,
  `2a0cf299`, `761f6d55`, `58a25d7e`, `61b6f8f9` (plus size sweep across all 87).
- Branch pointers: `holla` tip `b6b0d3c2` (2026-09-11); working tree on branch `holla`.

---

## 1. Timeline of REFACTORING_GOAL.md

### v1 — `e48137f1` (2026-09-03) "docs: add repository refactoring goal"

1,683 lines, 30 numbered sections. The original goal: a ground-up, in-place refactor.

- Mission: "Refactor this repository in place from an excellent but prototype-shaped Junie-inspired Ratatui design
  laboratory into a genuinely reusable, composable, flexible, themeable, and professionally engineered Rust TUI
  component system", with "the strongest architectural qualities of shadcn/ui, translated properly into Rust and
  terminal UI constraints".
- Framed as an execution contract, not a proposal: "This is an implementation goal. Do not stop after analysis,
  recommendations, an architecture document, or a migration plan."
- Non-negotiable result (#2): "one coherent public API and interaction model"; "`showcase`, `tablepro`, and
  `jackin-preview` use the refactored public component API"; "There is no parallel legacy component API"; "Backward
  compatibility with the current experimental API is not required"; the Junie visual language remains "the polished
  default theme"; "At least one substantially different non-Junie theme proves that the theme system is real".
- Authority order (#3) ranked this goal first, then `DESIGN.md`, rendered-output evidence, current source
  ("implementation evidence, not an architecture that must be preserved"), app behavior, `JACKIN_REFERENCE.md`,
  existing `GOAL.md`/`FEEDBACK.md`, and "shadcn/ui and external references — conceptual and architectural references,
  not implementation templates".
- Non-goals (#5): no "clone of a web framework", no virtual DOM / CSS parser / Tailwind-like class system, no
  shadcn registry or installer CLI, "Do not redesign TablePro or Jackin from scratch", "Do not optimize for backward
  compatibility."
- Sections 6–30 prescribe the full program: baseline, audit, external research, architecture decision phase, public
  API quality, state/render/event/focus/overlay/theme models, component families, package architecture, application
  migration, litmus tests, docs, testing, quality gates, implementation strategy, subagent responsibilities,
  definition of done, final report. Closes with "Deliver the finished reusable Rust TUI component system."

### v2 — `dce91d51` (2026-09-03) "docs: remove obsolete goal references"

Same-day trim after `7fbef957` "chore: remove obsolete project documents". −318 bytes.

- Removed: `JACKIN_REFERENCE.md` and existing `GOAL.md`/`FEEDBACK.md` from the authority order (items 6–7 gone;
  external references renumbered to item 6), from the mandatory reading list, and from the Jackin-preservation
  clause, which now reads "Preserve the deterministic Jackin product experience and the semantics established by
  the current source and interaction tests."
- Net effect: prior product-intent documents stop being authority; the mission itself is unchanged.

### v3 — `2d81eec4` (2026-09-03) "chore: configure refactor agent routing"

+22 lines net. Adds execution-governance machinery; the technical goal is untouched.

- Added: "# 0. MANDATORY MODEL ROUTING" — three configured agent types: "`refactor-coordinator` — `claude-fable-5-1`,
  effort `high`", "`opus-analyst` — `claude-opus-5`, effort `high`, read-only; owns all research and judgment work",
  "`fable-builder` — `claude-fable-5-1` … owns implementation and execution work". Rules include: "Never use Opus for
  repository mutation", "Treat an unavailable required model, model substitution, or model/effort mismatch as a
  blocker", and "Fable must not silently change an accepted architecture or public invariant."
- Added: the `REFACTORING_STATE.md` ledger requirement — "Maintain `REFACTORING_STATE.md` throughout the run so work
  survives compaction and resume… Fable alone edits this ledger."
- Re-scoped later sections: independent reviewers/verifiers are now specifically "fresh, read-only `opus-analyst`"
  agents; Fable applies corrections.

### v4 — `25ea92b0` (2026-09-03) "docs: keep research tooling flexible"

Research section de-prescripted.

- "# 8. EXTERNAL RESEARCH" → "# 8. ARCHITECTURE RESEARCH"; "Research current primary sources before choosing the new
  architecture" → "Investigate the design questions that materially affect the new architecture. The analyst chooses
  the evidence and read-only tools appropriate to each question."
- Removed: the mandatory inspection list with pinned URLs (shadcn/ui, Rust API Guidelines, Ratatui, Crossterm),
  "Use primary documentation and source rather than blog summaries", and "Record versions or commits for important
  external references so the research remains reproducible."
- Broadened: authority item "shadcn/ui and external references" → "Relevant prior art"; non-goal "a clone of a web
  framework" → "a clone of another UI framework".
- Net effect: evidence gathering delegated to analyst judgment rather than a fixed source list.

### v5 — `efada044` (2026-09-05) "docs: replace refactor goal with UI parity goal"

The pivotal re-scope — done by prepending five lines, not by editing the body.

- Added header (verbatim): "> Historical architecture contract. The active execution goal is `GOAL.md`.
  > This document remains useful for preserving the accepted `junie-tui` architecture, but its old slice order and
  > completion prompt do not override the parity-first UI/TUI restoration work."
- The entire v4 body (mission, 30 sections, "Deliver the finished reusable Rust TUI component system.") is retained
  unchanged underneath the demotion notice.
- In the same commit, `GOAL.md` gains its new header: "# Goal — restore the historical UI/TUI while finishing the
  refactor", declaring itself "the canonical continuation goal… Treat older execution prompts as historical unless
  they agree with this contract." Its mission: "Keep the current refactor's architecture, public `junie-tui` API,
  package boundaries, security fixes, and product improvements. Restore the user-visible Showcase, TablePro, and
  Jackin Preview experience to the known-good pre-refactor behavior." Its diagnosis: "The refactor concept is
  correct. The rendering migration was not." And its new rule: "Everything else that differs from the historical
  evidence is a regression until proven otherwise."
- Net effect: the goal flips from "finish the new component system (old visuals preserved as default theme)" to
  "keep the architecture, but make the products look and behave exactly as they did before the refactor".

### v6 — `d4715f8e` (2026-09-08) "chore: drop agent definitions and scrub agent mentions"

Terminology scrub only; +14 bytes.

- Renamed the three agent types to generic roles: "`refactor-coordinator`" → "`coordinator`",
  "`opus-analyst`" → "`read-only analyst`", "`fable-builder`" → "`builder`" (model/effort assignments unchanged).
- "Use subagents aggressively" → "Use agents aggressively"; "# 28. SUBAGENT RESPONSIBILITIES" →
  "# 28. AGENT RESPONSIBILITIES".
- Net effect: no goal change; the document is decoupled from the dropped `.claude/agents/` definitions.

### Current on `main` (tip `7b27732a`, 2026-09-10)

`main:REFACTORING_GOAL.md` is byte-identical to v6 (`d4715f8e`). It has not been edited since 2026-09-08 and still
carries the `efada044` demotion header, so on `main` it is a historical architecture contract; `GOAL.md`
(restore-parity) is the live execution goal.

---

## 2. REFACTORING_STATE.md evolution (87 commits on `main`, sampled)

The ledger was created by `2d81eec4` and grew monotonically (modulo one early rewrite) from ~1.9 KB to 189 KB. It is
append-only in practice: the frozen header still describes a 2026-09-04 interruption, while later commits add a
"READ FIRST" pointer declaring the *last* section authoritative.

| Sample | Date | Size | What was recorded done / in progress |
|---|---|---|---|
| `2d81eec4` (first) | 2026-09-03 | 1.9 KB | "Overall: not started"; "Active slice: Slice 1 — baseline and audit"; model routing recorded; everything "pending". |
| `d5e7075f` | 2026-09-03 | 0.6 KB | Compact rewrite of the ledger; still "not started", "Slice: 1 — baseline and audit". |
| `596a4706` | 2026-09-04 | 1.8 KB | "Slice 1 in progress"; baseline green (fmt/clippy/198 tests/build all exit 0, "Pre-existing failures: none"); five Opus audits running (api, app, domain-boundary, interaction, architecture-research). |
| `8ed714ba` | 2026-09-04 | 31 KB | "Slice 3 CLOSED and green at commit 0f66160 (797 tests…)"; "Slice 4 wave 1 was interrupted mid-flight by a session token limit" — three builders killed, tree does not compile (E0502). |
| `ad94d12a` | 2026-09-04 | 61 KB | Sessions 3–4: multi-lead coordination; cross-lead conflict over `Select` focus trapping under adjudication; process note: overturning a recorded section before it is recorded "produces exactly this kind of split-brain". |
| `14bca4a3` | 2026-09-04 | 107 KB | Session 5: §49 bless committed at `bfcf5e4` (xtask 22/22, render-components 164/164); library 370/370, conformance 489/489; adjudications §§50–§59; "NavList, Steps, TooSmall and Grid remain dormant/unexported… no Slice-4 completion claim is made." |
| `a1759b2a` | 2026-09-05 | 131 KB | `Ui::reference` migration handoff; §§67–§69 (ActionKey binding publication, runtime `Phase::Move`); visual PASS recorded (later superseded). |
| `07bb7193` | 2026-09-05 | 156 KB | "READ FIRST: the authoritative state is the last section of this file… where they conflict, the last checkpoint governs"; the `a1759b2` visual **PASS** is explicitly "superseded by the **FAIL** recorded in the last checkpoint"; package renamed `tui-next` → `junie-tui`; doc-check PASS (76 blocks, 859 refs); capture evidence stale; "No baseline edit or blessing is authorized". |
| `2a0cf299` | 2026-09-05 | 168 KB | 934 passing conformance tests at `f713ccb`; "No full workspace/Slice 4 completion claim follows." (Last 09-05 revision.) |
| `761f6d55` | 2026-09-08 | 172 KB | Porting-analysis session: local working-tree campaign vs `codex/main-holla-integration`; hunks classified ALREADY-PRESENT / SUPERSEDED / SKIP. |
| `58a25d7e` | 2026-09-08 | 177 KB | "Porting session CLOSED" — 10 commits landed on the integration branch; "migration complete; nothing further to port"; PR #1 verified open; deviation note: "executed on Opus 5 per recorded routing deviation (Fable 5.1 capacity exhausted)". |
| `61b6f8f9` (last on `main`) | 2026-09-09 | 189 KB | Final state — see below. |

### Final state on `main` (`61b6f8f9`, 2026-09-09)

- Holla-integration checkpoint (2026-09-08): execution authority is now `docs/plans/main-holla-integration-task.md`
  ("Earlier claims/routing/scope are historical"); pins `MAIN_BASE c12cad87…`, `HOLLA_REFERENCE 794b095c…`; candidate
  run: "2,451 passed, 18 failed, 1 ignored" of 2,470 tests; "No full goal or parity completion claim."
- Consolidation checkpoint (2026-09-09): full CI-gate sweep on the integration branch passes "except
  `parity_contract`". Parity triage against the frozen `baseline/before` captures: "60 byte-exact matches, 379
  mismatches, 60 run failures" of 499 recipes. `main` itself was red at the MSRV step — "316 of 316 runs" failing —
  which the consolidation repaired.
- Closeout: "The GOAL2 consolidation is complete up to the standing open items: the inherited red `parity_contract`
  (honest red, refusal paths documented above) and the successor work rejected with root causes in the ledger
  (manager rewrite, item-row part-patch threading, showcase WIP fixtures)." Cleanup deleted 231 scratch worktrees,
  111 local and 91 remote branches, leaving "three remote branches: `main` (protected), `holla` (the frozen
  pre-refactor product line), and `codex/main-holla-integration`". PR #1 merged into `main` the next day
  (`7b27732a`, 2026-09-10).

Phases of work as the ledger records them: (1) setup/baseline (09-03), (2) audits → architecture → slices 1–3 green
(09-04), (3) Slice 4 component families with a token-limit interruption, multi-lead adjudications §§29–§69, bless
governance (09-04 → 09-05), (4) authoritative-checkpoint bookkeeping with stale-capture and no-bless caveats (09-05),
(5) pivot to holla-parity porting onto `codex/main-holla-integration` (09-08), (6) gate sweep, parity triage, and
workspace cleanup (09-09).

---

## 3. Reading guide

- **Current intent on `main`**: `REFACTORING_GOAL.md` is *not* the active goal — since `efada044` it is a
  self-declared "Historical architecture contract", valuable for "preserving the accepted `junie-tui` architecture".
  The active execution goal on `main` is `GOAL.md` ("restore the historical UI/TUI while finishing the refactor"),
  with `docs/plans/main-holla-integration-task.md` cited by the ledger as the integration-phase execution authority.
- **On the `holla` branch** (current working tree): `REFACTORING_GOAL.md`, `REFACTORING_STATE.md`, and `GOAL.md` do
  not exist at all. The holla line's current plan lives in `REFACTORING_COMPLETION_PLAN.md` (repo root) and
  `docs/refactoring-plan/` — those documents, not anything above, govern holla work. (Pointers only; their content
  is out of scope here.)
- **Contradictions and tensions between versions**:
  1. v5's demotion header vs its own body: the file simultaneously says "This is an implementation goal… Deliver the
     finished reusable Rust TUI component system." (body) and "its old slice order and completion prompt do not
     override the parity-first UI/TUI restoration work" (header). Readers must apply the header's precedence rule.
  2. Goal inversion between v1–v4 and `GOAL.md`: the original goal made the refactored architecture the deliverable
     and the historical visuals merely "the polished default theme" with documented intentional changes allowed;
     `GOAL.md` makes the historical product behavior the contract — "Everything else that differs from the
     historical evidence is a regression until proven otherwise" — while keeping the new architecture.
  3. `REFACTORING_STATE.md` contradicts itself by design: the frozen header still reports "Slice 4 wave 1…
     interrupted… does not compile" (E0502), while the 2026-09-05 READ-FIRST bullet declares earlier content
     "historical evidence, retained unedited; where they conflict, the last checkpoint governs" — including an
     explicit PASS→FAIL supersession of the `a1759b2` visual audit.
  4. Outcome vs goal: the final ledger records `parity_contract` still red ("60 byte-exact matches, 379 mismatches,
     60 run failures" of 499 recipes), so the restore-parity goal in `GOAL.md` was *not* fully satisfied at the
     2026-09-09 closeout; the ledger calls this an "honest red" left as a standing open item.
  5. Minor: v3 mandated `claude-fable-5-1` execution and treated substitution as "a blocker"; the 2026-09-08 porting
     session was "executed on Opus 5 per recorded routing deviation (Fable 5.1 capacity exhausted)", recorded as a
     deviation rather than a blocker.
