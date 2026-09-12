# Consolidated refactoring goal — terminal-components-claude

**Provenance:** Consolidated on 2026-09-12 from `PLANNING_GOAL.md`, `REFACTORING_COMPLETION_PLAN.md`, `IMPROVEMENTS_PLAN.md`, `DESIGN.md`, `docs/improvements-plan-reference.md` and all of `docs/refactoring-plan/` (holla working tree); `main:GOAL.md`, `main:GOAL2.md`, `main:REFACTORING_GOAL.md` (read via `git show`); and product intent from `holla-project/GOAL.md`, `holla-project/CONCEPT.md`, `holla-project/PROMPT.md`. Each section names its sources. Where sources contradict, both statements are kept and flagged in §9.

## 1. The goal (single coherent statement)

Finish the interrupted architecture/API refactoring of this repository: complete main's accepted reusable component architecture and public `junie-tui` API, while reproducing — exactly, and with fail-closed proof — the user-visible behavior of the immutable oracle tag `holla-fable-2026-09-10` (commit `02f5294bfdbf38004cc49130d0aff1d01f31434c`) across all four applications: **Showcase, Holla, Jackin and TablePro**.

Architecture changes; the product experience does not. The strict parity rule is 1:1 equivalence from the user's perspective — "similar", "close enough", "improved" or "modernized" are not acceptable, and nothing may be redesigned because the refactored implementation could be cleaner. Parity must be achieved *through* the refactored reusable component system, not through application-local patches.

Provenance: `PLANNING_GOAL.md` mission and §1 strict parity rule; `REFACTORING_COMPLETION_PLAN.md` §1 executive goal; `main:GOAL.md` mission ("keep the current refactor's architecture, public `junie-tui` API, package boundaries, security fixes … restore the user-visible experience"); `main:REFACTORING_GOAL.md` §1–2 architectural target.

## 2. Situation and root cause

- `main` was mid-refactoring when interrupted: parts implemented, parts unfinished, some components/applications broken, some visual behavior regressed, and historical documents holding decisions that must be reconstructed rather than ignored (`PLANNING_GOAL.md` mission).
- Root cause: the migration treated product rendering as replaceable implementation instead of preserving it behind the new architecture, removing the executable visual oracle before a parity harness existed — `18afddd` added the correct new foundations, `7784719` removed the historical renderer/widgets before parity was proven, and facade-only enforcement (`1378c31`) cemented the divergence (`main:GOAL.md`).
- The dominant regression mechanism on main is independent ownership of visible content and interactive state: main sometimes calls a reusable component and then paints another model over its output, or displays controls without live state/handlers (`REFACTORING_COMPLETION_PLAN.md` §1).
- The `holla` branch is the known-good pre-refactor application experience. Its architecture is old and must **not** be treated as the desired architecture, but its previews, flows, visual behavior and overall UX are substantially correct (`PLANNING_GOAL.md`).
- Why a fourth app exists: the Holla app was built on the holla line as a new product — a context-adaptive action launcher ("this folder, this host, right now"), fully simulated with deterministic in-memory fixtures, in the Junie TUI design system (`holla-project/GOAL.md`, `holla-project/CONCEPT.md`, `holla-project/PROMPT.md`). Main-era goals therefore name three apps while the current goal names four.

## 3. Sources of truth (authority hierarchy)

### User-visible behavior

1. The immutable tag `holla-fable-2026-09-10` → `02f5294bfdbf38004cc49130d0aff1d01f31434c` (annotated tag object `a643909d9a782adaf0aa1e3357710a5ed3f24443`).
2. Deterministic evidence captured from that tag.
3. The moving `holla` branch as supporting implementation context only — it may not silently redefine the visual contract.

Provenance: `PLANNING_GOAL.md` §1, §3; `REFACTORING_COMPLETION_PLAN.md` §2–3. This supersedes `main:GOAL.md`'s older oracle (historical source `d5e7075`/`cc14dd6` plus 499 `baseline/before/` captures) — see §9.1.

The oracle is the acceptance oracle for every observable characteristic: layout, spacing, dimensions, alignment, borders, separators, glyphs/icons, text, colors, themes, focus/hover/pressed/selected/disabled/active states, mouse and keyboard interaction, focus traversal, scrolling and scroll boundaries, scroll-edge fades, cursor behavior, text editing, selection, menus, dialogs, overlays, popovers, tables/grids, truncation, wrapping, status bars, empty/error states, navigation, resizing, terminal-size-dependent behavior, application flows — across Showcase, Holla, Jackin and TablePro (`PLANNING_GOAL.md` §1).

### Architecture/API/refactoring intent

1. Accepted, non-superseded architectural decisions reconstructed from repository history (the canonical 620-clause ledger across nine ledgers; adjudications ADJ-01–14).
2. Latest authoritative refactoring state/checkpoints and their supporting commits (architectural main `7b27732a8c3c131760ec3438f641cb3c11343a42`).
3. Actual implemented code on `main` — evidence of implementation, never proof that an architectural exception was accepted.
4. Earlier plans and historical documents as evidence, with each statement classified as proposed / accepted / rejected / superseded / amended / partially or completely implemented / subsequently broken / deferred / abandoned.

Provenance: `PLANNING_GOAL.md` §2–3, §6; `REFACTORING_COMPLETION_PLAN.md` §2, §5–6; `main:REFACTORING_GOAL.md` §3 authority order; `main:GOAL.md` authority order.

### Visual language

`DESIGN.md` is the authority for the approved default Junie TUI visual language; its tokens are exact. Recognizing principles: one green hue only where attention belongs (red/amber only as safety tones); state is geometry before color (every state survives monochrome); whitespace groups while borders bound; dense rows with quiet chrome; keyboard first with mouse equal; contextual discoverability via the hint footer; safety proportional to risk (two-gate typed confirmation for broad destructive actions); quiet timed feedback; 72×20 minimum with prioritized dropping when narrow (`DESIGN.md` overview; `main:REFACTORING_GOAL.md` §3 item 2; `holla-project/GOAL.md` §2).

### Task specification

The current canonical task-format — revision `52d9f1eb7721f409bc47beb9fced7997b5c13ede`, schemas `task/v5`, `verify/v2`, `task-meta/v1` — with no invented custom schema (`PLANNING_GOAL.md` §9; `refactoring-tasks/README.md`; `docs/refactoring-plan/task-format.md`).

### Verification foundation

`tui-snap` at the independently reviewed PR #1 head `883d03f19d890bbbf27468798db78b04e85297ac` (open, unmerged; a hard prerequisite of reference qualification), plus ordinary deterministic Rust tests and accepted architecture/API/performance gates. Verification is fail-closed: golden evidence comes only from the immutable oracle; a mismatch fails; no candidate can bless, regenerate or redefine expected output. Tool gaps are fixed only as generally reusable primitives via a separate PR to `donbeave/tui-snap`, never auto-merged (`PLANNING_GOAL.md` §8, §14; `REFACTORING_COMPLETION_PLAN.md` §11–12; `docs/refactoring-plan/proof-contract.md`).

## 4. Accepted architectural direction and rejected paths

Fixed direction: caller-owned durable state; short-lived borrowed props; separate mutable update and shared-reference draw; typed responses/actions; stable identity; runtime-owned input/focus/layers/time; semantic paint provenance; one reusable implementation per component family; backend-free core consumers; curated application/author facades; the virtual workspace of `junie-tui`, `junie-tui-testing`, `xtask` and four application packages (`REFACTORING_COMPLETION_PLAN.md` §4, §6).

Rejected — must not be resurrected: the old widget facade; an owned runtime component tree; an untyped action bus; a universal Widget/Theme abstraction; an application-owned generic overlay stack; Grid SQL semantics; generic RGB role inference; superficial builder-wrapping of existing structs; any parallel legacy component API or compatibility shim that should have disappeared (`REFACTORING_COMPLETION_PLAN.md` §6; `main:REFACTORING_GOAL.md` §2; `PLANNING_GOAL.md` §12).

Deferred: operational Holla providers and HP16 headless/list/run/doctor expansion; the "Later" sections of `IMPROVEMENTS_PLAN.md` (remaining F-task portions, Holla production/non-UI integration, conditional O04/O05/O07). Retired framework proposals O01/O02/O03/O06 are removed, not pending (`REFACTORING_COMPLETION_PLAN.md` §6; `IMPROVEMENTS_PLAN.md`; `docs/improvements-plan-reference.md`).

## 5. Strict invariants

- **No redesign.** Do not alter colors, spacing, interaction patterns, layouts, flows, hover/focus behavior, scrolling, component appearance or application composition during the refactor (`PLANNING_GOAL.md` §1, §21).
- **Fail-closed parity evidence.** No visual-regression task may "fix" the baseline: no overwriting snapshots, blessing changed output, weakening comparison, deleting scenarios/fixtures, suppressing gates, or changing the reference commit. Expected evidence is sealed outside every executor's writable scope (`PLANNING_GOAL.md` §14; `REFACTORING_COMPLETION_PLAN.md` §11).
- **Reusable architecture.** All four applications consume the refactored public APIs; no duplicated styling/interaction logic, no app-local versions of reusable components, no old/new architectures coexisting at the end (`PLANNING_GOAL.md` §12; `main:REFACTORING_GOAL.md` §2).
- **Architecture is verified, not only visuals.** Deterministic gates for forbidden legacy imports, dependency boundaries, public API surface, registry completeness, module ownership, duplication, deprecated-path removal, compile/MSRV/clippy/fmt/doc, and performance where historically required — only checks supported by reconstructed accepted requirements (`PLANNING_GOAL.md` §13).
- **Execution-ready tasks only.** Every task is a genuine task-format package with explicit scope, MUST/MUST-NOT/non-regression requirements, typed acceptance criteria, deterministic `verify.toml`, fixed decisions and explicit DAG dependencies; no vague "fix components" tasks (`PLANNING_GOAL.md` §9–11).
- **Bidirectional traceability.** Requirement→task, task→requirement, component→task, application-state→verification, verification→requirement, historical-decision→implementation; no orphan requirements, no orphan tasks, no important golden state without proof (`PLANNING_GOAL.md` §18).
- **Mandatory independent review.** Fresh-context reviewers attack the plan and the repaired plan; findings are accepted-and-repaired or rejected with evidence (`PLANNING_GOAL.md` §17; `REFACTORING_COMPLETION_PLAN.md` §24).

## 6. How current intent differs per branch (explicit)

### `holla` (this branch)

Finish the refactoring **on top of the holla line's preserved UI/UX** — porting main's completed architectural work without breaking UI/UX or visual output. Concretely, this branch hosts: the behavior-oracle lineage (the `holla-fable-2026-09-10` tag; the completed simulated-Holla product work tracked in `IMPROVEMENTS_PLAN.md` — H00, F02–F23h, HP01–HP23 simulated slices all checked), and the entire planning deliverable (`PLANNING_GOAL.md`, `REFACTORING_COMPLETION_PLAN.md`, `docs/refactoring-plan/`, `refactoring-tasks/`). Holla's own old architecture is context only, never the target.

Per the completion plan's chosen integration strategy (§13): at the start of future execution, create an isolated integration branch **from pinned main `7b27732a`**, preserve main's workspace/reusable API/runtime/accepted protections, and adapt the oracle's app composition, deterministic domain fixtures and behavior through those APIs. Do **not** merge the two full branches, recreate the legacy API, or cherry-pick stale ports wholesale. No branch, production port, commit, merge or publication is created during the planning goal. So "on top of holla" describes the planning home and the behavioral contract; the execution branch itself starts from main's architecture and converges toward replacing `main`.

### `main`

The workspace/crates refactoring approach that was partially done: a virtual workspace (`crates/tui` = `junie-tui`, `crates/tui-testing`, `xtask`, four app packages), with a substantial prior integration merged as PR #1 at `7b27732a` (produced by `main:GOAL2.md`'s worktree-consolidation campaign on `codex/main-holla-integration`). It is architecturally real but incomplete and visually regressed against the oracle: Showcase 22 vs 23 pages (Diff missing), Holla 11 vs 34 worlds, simplified Jackin Settings/Usage, simplified TablePro query editor/plan/history (`REFACTORING_COMPLETION_PLAN.md` §7).

Main's own goal documents retain historical roles: `main:REFACTORING_GOAL.md` is the historical architecture contract (its slice order and model-routing rules are superseded; its accepted architecture is preserved); `main:GOAL.md` is the parity-first restoration goal on that workspace (three apps, older oracle — superseded pins, see §9); `main:GOAL2.md` is the completed worktree-consolidation goal. Current intent for main: its completed work is preserved and ported into the integration branch, its unfinished owners continue, and its obsolete self-baselines receive explicit disposition rather than silent deletion.

## 7. Non-goals

No UI redesign; no UX polish or new product features; no approximate parity; no operational Holla provider wiring or deferred CLI expansion; no merge into `main` during planning; no automatic tui-snap PR merge; no legacy API revival; no opportunistic terminal-components bugfix; no production refactoring, task execution, integration branch, commit or publication under the current (reopened) planning goal (`REFACTORING_COMPLETION_PLAN.md` §22; `PLANNING_GOAL.md` §19, §21; `docs/refactoring-plan/reaudit-plan.md`).

## 8. Definition of done (end-to-end completion gate)

The final integration gate must prove: build correctness (all authoritative build/test/lint/doc/architecture gates); architectural completion (no unfinished refactoring path, no obsolete old/new duplication); API completion (all reusable components exposed and consumed as intended); application completeness (all four apps); visual parity for every covered golden state (`final output == holla-fable-2026-09-10 reference` under the selected deterministic representation); interaction parity (keyboard, mouse, focus, editing, scrolling, overlays, resize, navigation, important flows); no hidden baseline drift (expected artifacts provably not regenerated from candidate code); and merge readiness (the integration branch is safe to merge into/replace `main`, with the resulting merge tree verified as well) (`PLANNING_GOAL.md` §16; `REFACTORING_COMPLETION_PLAN.md` §20; `main:GOAL.md` completion conditions).

## 9. Contradictions between sources (both statements preserved)

1. **Oracle identity.** `main:GOAL.md` pins the historical source `d5e7075`/`cc14dd6` plus 499 `baseline/before/` captures covering three apps (no Holla app). The holla planning set pins immutable tag `holla-fable-2026-09-10` = `02f5294` covering four apps. `REFACTORING_COMPLETION_PLAN.md` §2 explicitly states the current planning goal supersedes older visual oracle pins; the `baseline/before` archive remains historical evidence and must never be relabeled as a September 10 oracle capture.
2. **Baseline mutability.** `main:GOAL.md` permits baseline updates through explicit review records and allows recorded "approved additions" and isolated reviewed bug fixes. The holla contract is stricter: expected evidence is oracle-only, sealed outside writable scope, and no candidate-side update is possible at all. Current authority is the stricter holla contract; deliberate defect fixes documented in `baseline/before/NOTES.md` remain allowed only as isolated, reviewed, oracle-qualified corrections.
3. **Application count.** Main-era goals name three apps (showcase, tablepro, jackin-preview); the current goal names four, adding Holla per `holla-project/` product intent. Main's migrated Holla package (11 worlds) is incomplete against the 34-world oracle.
4. **Execution base.** The branch-level framing is "finish the refactoring on top of holla, porting main's completed work"; the plan's chosen strategy (§13) instead branches from pinned main `7b27732a` and ports the oracle's behavior through main's APIs. Both are preserved in §6: holla = planning home + behavioral source of truth; main = architectural base for the execution branch; end state replaces `main`.
5. **Non-Junie theme.** `main:REFACTORING_GOAL.md` §2 requires at least one substantially different non-Junie theme proving the theme system is real. The current completion plan's parity oracle is Junie-only, and theme customization remains accepted architecture (semantic tokens, overridable themes). The non-Junie-theme demonstration is not restated as an explicit obligation in the completion plan — flagged as an unresolved historical requirement that traceability must not silently drop.
6. **Operational rules.** Model routing (`claude-fable-5-1`/`claude-opus-5` agent rules), `rtk` command prefixes, `BLESS_GUARD_BASE=d5e7075`, and old slice ordering in `main:REFACTORING_GOAL.md`/`main:GOAL.md` are historical. `REFACTORING_COMPLETION_PLAN.md` §2 states the current planning goal supersedes historical model-routing instructions, stale stop/restart orders, baseline blessing permissions and implementation mandates; `BLESS_GUARD_BASE` is redefined as the recorded real parent/base.
7. **Catalog size.** Companion text describes "74 sealed task-format v5 packages"; the catalog contains **73** task packages (`TASK-001`–`TASK-073`) plus the group `README.md` — 74 entries in the directory. `task-index.tsv` has 73 data rows and `REFACTORING_COMPLETION_PLAN.md` §14/§16 says 73. See `docs/refactoring/plan.md` §6.
