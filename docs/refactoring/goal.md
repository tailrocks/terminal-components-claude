# Consolidated refactoring goal — terminal-components-claude


> Historical evidence; claims below apply to the revisions and scope identified in the report.

Prove architectural readiness from current crate boundaries and reusable API ownership; prove design parity from current interaction and visual evidence. Historical completion labels or old passing checks do not prove the current goal is achieved.

**Provenance:** Consolidated on 2026-09-12 from `docs/sources/PLANNING_GOAL.md`, `docs/sources/REFACTORING_COMPLETION_PLAN.md`, `docs/improvements/PLAN.md`, `docs/design/DESIGN.md`, `docs/improvements/plan-reference-audit.md` and all of `docs/refactoring-plan/` (holla working tree, now `visual-baseline`); [GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/GOAL.md), [GOAL2.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/GOAL2.md), [REFACTORING_GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/REFACTORING_GOAL.md) (read via `git show`); and product intent from `docs/product/GOAL.md` and `docs/product/CONCEPT.md`. Each section names its sources. Where sources contradict, both statements are kept and flagged in §9.

## 1. The goal (single coherent statement)

Complete the started but unfinished refactor into split crates with reusable public APIs, following `main`'s architectural direction. Preserve the exact intended user experience from `design/interaction-polish`: visuals, colors, layouts, interactions, behavior, and all planned work across **Showcase, Holla, Jackin, and TablePro**. The `main` refactor remains incomplete, broken, and not ready. Use current code and visual evidence to verify each migrated capability; historical visual-baseline pins are provenance only.

Architecture changes; the product experience does not. The strict parity rule is 1:1 equivalence from the user's perspective — "similar", "close enough", "improved" or "modernized" are not acceptable, and nothing may be redesigned because the refactored implementation could be cleaner. Parity must be achieved *through* the refactored reusable component system, not through application-local patches.

Provenance: `docs/sources/PLANNING_GOAL.md` mission and §1 strict parity rule; `docs/sources/REFACTORING_COMPLETION_PLAN.md` §1 executive goal; [GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/GOAL.md) mission ("keep the current refactor's architecture, public `junie-tui` API, package boundaries, security fixes … restore the user-visible experience"); [REFACTORING_GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/REFACTORING_GOAL.md) §1–2 architectural target.

## 2. Situation and root cause

- `main` was mid-refactoring when interrupted: parts implemented, parts unfinished, some components/applications broken, some visual behavior regressed, and historical documents holding decisions that must be reconstructed rather than ignored (`docs/sources/PLANNING_GOAL.md` mission).
- Root cause: the migration treated product rendering as replaceable implementation instead of preserving it behind the new architecture, removing the executable visual oracle before a parity harness existed — `18afddd` added the correct new foundations, `7784719` removed the historical renderer/widgets before parity was proven, and facade-only enforcement (`1378c31`) cemented the divergence ([GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/GOAL.md)).
- The dominant regression mechanism on main is independent ownership of visible content and interactive state: main sometimes calls a reusable component and then paints another model over its output, or displays controls without live state/handlers (`docs/sources/REFACTORING_COMPLETION_PLAN.md` §1).
- `design/interaction-polish` is the intended application experience and the authority for previews, flows, visuals, colors, interactions, and behavior. Its monolithic architecture is not the target. `main` is the split-crate architectural direction, but its refactor remains incomplete, broken, and not ready.
- Holla is a context-adaptive action launcher ("this folder, this host, right now"), fully simulated with deterministic in-memory fixtures, in the Junie TUI design system (`docs/product/GOAL.md` and `docs/product/CONCEPT.md`). Main-era goals therefore name three apps while the current project contains four.

## 3. Sources of truth (authority hierarchy)

### User-visible behavior

1. Current design authority: the specifications and planned work on `design/interaction-polish`.
2. Current source and current deterministic visual/interaction evidence establish implementation state for the exact revision tested.
3. Historical snapshots, the `visual-baseline` tag, and earlier branch captures are scoped evidence only; they do not supersede the design branch or prove current parity.

The intended contract covers layout, spacing, dimensions, alignment, borders, separators, glyphs/icons, text, colors, themes, focus/hover/pressed/selected/disabled/active states, mouse and keyboard interaction, focus traversal, scrolling and scroll boundaries, scroll-edge fades, cursor behavior, text editing, selection, menus, dialogs, overlays, popovers, tables/grids, truncation, wrapping, status bars, empty/error states, navigation, resizing, terminal-size-dependent behavior, and application flows across Showcase, Holla, Jackin, and TablePro. See `docs/design/DESIGN.md` and the detailed behavior and task records; historical source docs are provenance, not current authority.

### Architecture/API/refactoring intent

1. Accepted, non-superseded architectural decisions reconstructed from repository history (the canonical 620-clause ledger across nine ledgers; adjudications ADJ-01–14).
2. Latest authoritative refactoring state/checkpoints and their supporting commits (architectural main `7b27732a8c3c131760ec3438f641cb3c11343a42`).
3. Actual implemented code on `main` — evidence of implementation, never proof that an architectural exception was accepted.
4. Earlier plans and historical documents as evidence, with each statement classified as proposed / accepted / rejected / superseded / amended / partially or completely implemented / subsequently broken / deferred / abandoned.

Provenance: `docs/sources/PLANNING_GOAL.md` §2–3, §6; `docs/sources/REFACTORING_COMPLETION_PLAN.md` §2, §5–6; [REFACTORING_GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/REFACTORING_GOAL.md) §3 authority order; [GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/GOAL.md) authority order.

### Visual language

`docs/design/DESIGN.md` and its linked branch plans specify the intended Junie TUI visual language and exact tokens. Preserve all documented details, including the attention colors, monochrome-readable state grammar, spacing, focus, hints, safety, feedback, and narrow-terminal behavior. Its full contract is retained in `design/interaction-polish`; historical product notes provide context only.

### Task specification

The current canonical task-format — revision `52d9f1eb7721f409bc47beb9fced7997b5c13ede`, schemas `task/v5`, `verify/v2`, `task-meta/v1` — with no invented custom schema (`docs/sources/PLANNING_GOAL.md` §9; `refactoring-tasks/README.md`; `docs/refactoring-plan/task-format.md`).

### Verification foundation

`tui-snap` at the independently reviewed PR #1 head `883d03f19d890bbbf27468798db78b04e85297ac` (open, unmerged; a hard prerequisite of reference qualification), plus ordinary deterministic Rust tests and accepted architecture/API/performance gates. Verification is fail-closed: golden evidence comes only from the immutable oracle; a mismatch fails; no candidate can bless, regenerate or redefine expected output. Tool gaps are fixed only as generally reusable primitives via a separate PR to `donbeave/tui-snap`, never auto-merged (`docs/sources/PLANNING_GOAL.md` §8, §14; `docs/sources/REFACTORING_COMPLETION_PLAN.md` §11–12; `docs/refactoring-plan/proof-contract.md`).

## 4. Accepted architectural direction and rejected paths

Fixed direction: caller-owned durable state; short-lived borrowed props; separate mutable update and shared-reference draw; typed responses/actions; stable identity; runtime-owned input/focus/layers/time; semantic paint provenance; one reusable implementation per component family; backend-free core consumers; curated application/author facades; the virtual workspace of `junie-tui`, `junie-tui-testing`, `xtask` and four application packages (`docs/sources/REFACTORING_COMPLETION_PLAN.md` §4, §6).

Rejected — must not be resurrected: the old widget facade; an owned runtime component tree; an untyped action bus; a universal Widget/Theme abstraction; an application-owned generic overlay stack; Grid SQL semantics; generic RGB role inference; superficial builder-wrapping of existing structs; any parallel legacy component API or compatibility shim that should have disappeared (`docs/sources/REFACTORING_COMPLETION_PLAN.md` §6; [REFACTORING_GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/REFACTORING_GOAL.md) §2; `docs/sources/PLANNING_GOAL.md` §12).

Deferred: operational Holla providers and HP16 headless/list/run/doctor expansion; the "Later" sections of `docs/improvements/PLAN.md` (remaining F-task portions, Holla production/non-UI integration, conditional O04/O05/O07). Retired framework proposals O01/O02/O03/O06 are removed, not pending (`docs/sources/REFACTORING_COMPLETION_PLAN.md` §6; `docs/improvements/PLAN.md`; `docs/improvements/plan-reference-audit.md`).

## 5. Strict invariants

- **No redesign.** Do not alter colors, spacing, interaction patterns, layouts, flows, hover/focus behavior, scrolling, component appearance or application composition during the refactor (`docs/sources/PLANNING_GOAL.md` §1, §21).
- **Fail-closed parity evidence.** No visual-regression task may "fix" the baseline: no overwriting snapshots, blessing changed output, weakening comparison, deleting scenarios/fixtures, suppressing gates, or changing the reference commit. Expected evidence is sealed outside every executor's writable scope (`docs/sources/PLANNING_GOAL.md` §14; `docs/sources/REFACTORING_COMPLETION_PLAN.md` §11).
- **Reusable architecture.** All four applications consume the refactored public APIs; no duplicated styling/interaction logic, no app-local versions of reusable components, no old/new architectures coexisting at the end (`docs/sources/PLANNING_GOAL.md` §12; [REFACTORING_GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/REFACTORING_GOAL.md) §2).
- **Architecture is verified, not only visuals.** Deterministic gates for forbidden legacy imports, dependency boundaries, public API surface, registry completeness, module ownership, duplication, deprecated-path removal, compile/MSRV/clippy/fmt/doc, and performance where historically required — only checks supported by reconstructed accepted requirements (`docs/sources/PLANNING_GOAL.md` §13).
- **Execution-ready tasks only.** Every task is a genuine task-format package with explicit scope, MUST/MUST-NOT/non-regression requirements, typed acceptance criteria, deterministic `verify.toml`, fixed decisions and explicit DAG dependencies; no vague "fix components" tasks (`docs/sources/PLANNING_GOAL.md` §9–11).
- **Bidirectional traceability.** Requirement→task, task→requirement, component→task, application-state→verification, verification→requirement, historical-decision→implementation; no orphan requirements, no orphan tasks, no important golden state without proof (`docs/sources/PLANNING_GOAL.md` §18).
- **Mandatory independent review.** Fresh-context reviewers attack the plan and the repaired plan; findings are accepted-and-repaired or rejected with evidence (`docs/sources/PLANNING_GOAL.md` §17; `docs/sources/REFACTORING_COMPLETION_PLAN.md` §24).

## 6. Current branch roles

### `design/interaction-polish`

This branch defines the required UI/UX: visuals, colors, layouts, interactions, and behavior. Keep its concrete design specifications and all planned design work. Its monolithic crate structure is not the target. Useful API ideas from this branch may be carried forward when they fit the split-crate architecture.

### `main`

This branch defines the architectural direction: split crates and reusable public APIs. The refactor has started but is incomplete, broken, and not ready. Do not describe it as completed, working, or ready; recorded slices, local test passes, or a successful build do not prove project-level readiness. Complete the architecture while preserving the UI/UX authority of `design/interaction-polish`.

### Historical records

The source goals, branch comparisons, task audits, and completion plans below record earlier identities, findings, and decisions. Keep their concrete requirements and planned work, but treat superseded branch strategies and completion claims as historical. Current source, manifests, task acceptance criteria, and verification determine present state.

## 7. Scope boundaries

Preserve the specified UI/UX; do not redesign it. Reorganize implementation into reusable crates and APIs following `main`'s direction. Retain planned product, parity, audit, and refactoring work unless current evidence proves it obsolete. Avoid unrelated behavior or feature changes while performing documentation cleanup and Rust comment removal; those requested edits must not change runtime behavior or visuals. Historical planning restrictions that prohibit implementation are not current project direction.

## 8. Definition of done (end-to-end completion gate)

The final integration gate must prove build correctness, architectural completion, reusable API completion, and application completeness across all four apps. It must verify every planned visual and interaction contract against `design/interaction-polish`, including keyboard, mouse, focus, editing, scrolling, overlays, resize, navigation, and important flows. Verify no planned work or reusable behavior was lost. Confirm the integrated crate structure and all current Rust, architecture, and visual gates pass. Historical evidence can support a finding but cannot substitute for current integrated-tree verification.

## 9. Contradictions between sources (both statements preserved)

1. **Historical oracle identities.** [GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/GOAL.md) and earlier planning records pin different historical sources and capture sets. Those identities explain old audits; none is the current UI/UX authority. `design/interaction-polish` is the current design reference. Re-check source and current visual evidence when asserting implementation parity.
2. **Historical baseline rules.** Older records disagree about baseline mutability and approval. Those are historical tool and verification decisions, not design authority. Follow `docs/visual-testing/README.md` for current visual procedures and preserve `design/interaction-polish` as the intended output.
3. **Application count.** Main-era goals name three apps (showcase, tablepro, jackin-preview); the current goal names four, adding Holla per `docs/product/` product intent. Main's migrated Holla package (11 worlds) is incomplete against the 34-world oracle.
4. **Execution base (historical conflict; current resolution).** Earlier planning alternated between continuing on visual-baseline and starting from main. Current authority resolves that conflict: main supplies the split-crate architectural direction but remains incomplete, broken, and not ready; design/interaction-polish defines the UI/UX, colors, interactions, and behavior. Implement the intended design in reusable crates and retain useful API ideas. Older strategy descriptions are historical, not current instructions.
5. **Non-Junie theme.** [REFACTORING_GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/REFACTORING_GOAL.md) §2 requires at least one substantially different non-Junie theme proving the theme system is real. The current completion plan's parity oracle is Junie-only, and theme customization remains accepted architecture (semantic tokens, overridable themes). The non-Junie-theme demonstration is not restated as an explicit obligation in the completion plan — flagged as an unresolved historical requirement that traceability must not silently drop.
6. **Operational rules.** Model routing (`claude-fable-5-1`/`claude-opus-5` agent rules), `rtk` command prefixes, `BLESS_GUARD_BASE=d5e7075`, and old slice ordering in [REFACTORING_GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/REFACTORING_GOAL.md)/[GOAL.md](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/sources/main/GOAL.md) are historical. `docs/sources/REFACTORING_COMPLETION_PLAN.md` §2 states the current planning goal supersedes historical model-routing instructions, stale stop/restart orders, baseline blessing permissions and implementation mandates; `BLESS_GUARD_BASE` is redefined as the recorded real parent/base.
7. **Catalog size.** Companion text describes "74 sealed task-format v5 packages"; the catalog contains **73** task packages (`TASK-001`–`TASK-073`) plus the group `README.md` — 74 entries in the directory. `task-index.tsv` has 73 data rows and `docs/sources/REFACTORING_COMPLETION_PLAN.md` §14/§16 says 73. See `docs/refactoring/plan.md` §6.
