# Refactoring history and decision authority

Investigation date: 2026-09-11. This is planning evidence, not implementation or parity certification. Commit-qualified paths below mean `git show <commit>:<path>`; historical pass counts are not current gate results.

## Immutable identities and topology

| Identity | Commit |
| --- | --- |
| UI/UX oracle, `holla-fable-2026-09-10` | `02f5294bfdbf38004cc49130d0aff1d01f31434c` |
| Architectural source, `main` / `origin/main` | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| Planning checkout, `holla` / `origin/holla` | `2e240139` |
| Merge base of main and holla | `cc14dd6beae526884aabdf897e309be837b4f504` |
| Previous Holla oracle used by PR #1 | `794b095c196562d38f1b6f7ce379c128af2a023d` |

The coordinator independently verified the annotated oracle tag locally and remotely before analysis. Local graph checks show `main...holla` contains **774 main-only and 38 holla-only commits**. These counts include merges and preserved historical branches; they are not numbers of independent changes.

`3dbc73e253a1c4aebbe9cccbfc6d5c5e3ff8d73e` has parents `02f5294b` and `794b095c`: the Holla-Fable implementation supersedes the earlier Holla branch. `git diff 02f5294b holla` contains exactly one changed file, `PLANNING_GOAL.md` (1,069 added lines). Thus current Holla and the oracle currently have identical tracked product code. This equality is observed, not permission to follow future Holla changes.

`7b27732a` merges parents `ef405e22` and `321f202e`. Its integration lineage includes `c3b51b96`, a semantic-union merge with parents `0191acbf` and `af1e752e`. Main already contains a substantial previous main/Holla integration. Repeating its cherry-picks would duplicate or overwrite later repairs.

From the merge base, main touches 4,085 paths, Holla 3,862 paths; exact-path overlap is 76. These totals include thousands of evidence artifacts. Semantic overlap is much larger than filename overlap because main deleted/moved the old `src` implementation. Main changes 494 source/package paths under `src`, `crates`, `apps`, and `Cargo.toml`; Holla changes 114. Holla's source delta adds 52,426 lines and removes 1,165, dominated by the expanded Holla simulation. Main's source delta adds 182,327 lines and removes 61,543. Counts characterize topology, not correctness.

## Timeline

| Date / commits | Event | Current disposition |
| --- | --- | --- |
| 2026-09-02 `4b857a07`, `65cb78d3` | Original laboratory and TablePro goals. | Historical product context; superseded execution prompts. Preserve behavior through the current oracle. |
| 2026-09-03 `86f25734`, `80331448`, `1e6b2031`, `5384c217`, `90c991aa` | Jackin prompt, preview isolation, rename `JUNIE_PROMPT3.md` to `GOAL.md`, 1Password priority and design authority. | Product behavior context; real provider integration remains outside this planning goal. |
| `74545e0f`, `7fbef957` | Earlier GOAL/GOAL2 generations deleted. | Deletion does not prove product capabilities were intentionally removed. |
| `e48137f1`–`25ea92b0` | Refactoring goal created, authority cleaned up, routing/state ledger added, research tooling relaxed. | Architectural capabilities survive; old model routing and implementation mandate do not govern this planning-only task. |
| 2026-09-04 `cefc4b8a`, `07cb2c9f` | Before-refactor captures and performance harness. | Historical evidence remains immutable; it is older than the current oracle. |
| `2e453023`, `e8d053c9`, `95ab6529` | Architecture A–I, testing/migration appendices, J review corrections. | Two-phase borrowed component model accepted, amended by later sections. |
| `27bd918e`, `87ab93d4` | L/K/M: modern dependency/API policy, Form, read-only/editable Grid split, facade and collection borrowing corrections. | Accepted with later backend/input and API amendments. |
| `18afddd2`, `7899678d` | Foundations, testing crate and xtask; correction pass. | Preserve structural architecture, not temporary `tui-next` naming. |
| `69fcdcad` | Interrupted partial components and amendments snapshot. | Historical interruption, not current work ownership. |
| `587c53bd`, `4aabceb7`, `dc3e0fa1` | Foundations F1–F26, N/O/P adjudications. | Refined layer sizing, measure/style access, cache/performance, mono and testing contracts. |
| `d7950b93`, `8ed714ba`, `cc1c877e`, `8b9d4d6d` | Second interruption; partial 4B/4G and wave-1 handoff. | Partial commits were preservation, not completed verification. |
| `70dacec1`–`4d262110` | Q and §§30–49: mono, parts, readiness, gate defects, first-generation blessing and migration ordering. | Later §72 replaces local forced-state APIs; fail-closed proof obligations survive. |
| `3adb6efe`, `14bca4a3`, `a1759b2a`, `ecc13378` | Remaining component contracts, collection scaling, component integration and closure decisions. | Component code present does not establish UI parity; audit explicitly remained red. |
| 2026-09-05 `4e07ea1`, `5042a40`, `444a8f4`, `1378c31b`, `77847196` | Showcase/TablePro/Jackin facade migrations and deletion of the old root package. | Architectural move survives. Historical renderer deletion before parity proof enabled the regression class. |
| `c9b765b8`–`55648350`, `54a7aa17` | Capsule chrome attempts preserved through user stops; tests moved from 0/6 to 2/6 before untested final edits. | Handoff restart instructions and pass counts are superseded; specific chrome obligations remain. |
| `efada044` | GOAL replaced with UI parity restoration; REFACTORING_GOAL gains historical-contract header. | Architecture retained; rendering redesign rejected. Its old oracle and approved-polish allowance are superseded by PLANNING_GOAL. |
| `f98c375d`, `c12cad87` | Historical component/application rendering and capture repairs. | Starting point of previous main-based integration, not parity completion. |
| 2026-09-06–08 `0f8e8fc9`–`794b095c` | First Holla simulation, scenarios, reference documents and old branch comparison. | Earlier Holla oracle, superseded by the September 10 tag. |
| 2026-09-08 `4e84b744`, `8f7e1c92`, `2caff455`, `b1c8c447`, `2f139259`, `934c92dc`, `577bf533` | Backend-free input, monotonic scheduling, successful-output publication, typing/cursor ownership, feedback timing and restored focus validation. | Accepted runtime work to preserve and verify against the new oracle. |
| `4afca16f`, `4a9dcab1`, `d52ee38b`, `29ed8e5e`, `aacb7cb3`, `30924105`, `da4b1364` | Authored palettes/provenance, symbolic Meter rest, detailed Grid gutters/header prefix and optional scrollbars. | Later accepted API amendments; historical generic color-equality ban has one explicitly authored Meter policy exception. |
| `675fd58`, `5eb277c`, `0d676b1`, `42d3a68`, `2b4a4ff`, `f2e06ac`, `b30c5ad`, `f8282d7`, `7bd8831`, `da92e48` | Ten adjudicated ports: viewport facts, focus repaint, Tree disclosure, badge recipe, layer parent, picker APIs, gutter role, Showcase Escape, replay and scanner. | Already integrated; do not re-port entire stale branches. |
| 2026-09-09 `c3b51b96`, `0191acbf`, `da38e52e`, `569167ee`, `378d51c9`, `61b6f8f9` | Semantic union, digest refresh, worktree consolidation and cleanup. | Consolidation completed historically; admitted rendering is not the present oracle. Remaining defects are explicitly documented. |
| 2026-09-10 `aebe6e5e`, `321f202e`, `7b27732a` | CI repair, capture regeneration and merged PR #1. | Current main tip; merge does not certify full parity. |
| `43dfdd71`, `f85954f8`, `215990ef` | Holla-Fable launcher/world and shared/app audit fixes. | New product baseline lineage, not old main architecture. |
| `abb480ac`, `c9661f6c`, `64add0ea` | Improvements plan extended for legacy capability parity, focused, split into detailed contracts. | Concrete F/HP obligations survive by scope; broad new frameworks retired. |
| `c9710d81`, `b5f237d4`, `03df2681`, `8f7ed0e1`, `1615aa56` | Viewport/Picker/Tree identity, execution and cleanup ownership, job control, rasterizer provenance and proof coverage. | Oracle behavior and invariant evidence requiring architectural migration. |
| `11ae5078`, `4399e9dd`, `1aaa9b01`, `812bad84` | HP01–15 and HP17–23 simulated journeys and row proofs; Current phase checked complete. | Simulation-only product acceptance; production providers/CLI remain deferred. |
| `53b8212f`, `e4866ce4`, `92d91629`, `02f5294b` | One-click focus/edit, scroll-edge fades and truthful shared scroll model. | Exact current UI contract; absent from the older integration oracle. |

## Document lineage and review evidence

The seven required document histories were searched across all local refs, including `--follow` where applicable. `GOAL.md` has multiple unrelated generations and a detected `JUNIE_PROMPT3.md` rename; `GOAL2.md` has an earlier TablePro generation, deletion, then consolidation recreation. `HANDOFF_SLICE4_WAVE1.md` is append-and-supersede history, not current execution order. No other required-document rename was reported by these follow searches.

`main:docs/audit/main-holla/history/INDEX.md` records an earlier reconstruction of 484 reachable commits, 68 changed parent-relative edges, and 64 unique blobs for REFACTORING_GOAL and COMPONENT_ARCHITECTURE through `c12cad87`. Its reports retain substantive original and amended contracts: `early/REPORT.md`, `api/REPORT.md`, `middle/early-large-report.md`, `middle/report-11-21.md`, `integrator-semantic-review.md`, `late/AUDIT.md`, `CURRENT-CONTRACT.md`, and `current-source-gaps.md`. They are historical independent review evidence, not fresh execution results. Some INDEX links name external patch files that are not tracked; recover exact content with Git instead of assuming those links exist.

`historical-named-obligation-inventory.json` in that directory indexes 1,483 literal API/test/path/command leads. It must remain a coverage input, not an acceptance checklist whose old presence flags prove completion. The current matrices must also cover later amendments and Holla-only improvements.

Fresh direct-source coverage is indexed in `history-revision-index.tsv` by changed parent edge and before/after Git blob. `history-other-docs.md` covers every generation and material amendment of GOAL, GOAL2, REFACTORING_GOAL, handoff and Improvements, including renamed/deleted Jackin prompts; `history-other-obligations.tsv` adds 56 individually identified refactor capability/invariant rows. `history-early.md`, `history-middle.md`, `history-late.md` and the historical-input review cover disjoint architecture and ledger ranges, including old removed text and exact-identical source variants. Review statuses specify scope; a partial section read is never labeled whole-blob certification. Earlier archive reports locate decisions but do not certify these fresh reads.

Indexed13-path universe: **281 exact parent/path edges; zero missing/extra/duplicate keys in independent Git re-enumeration**. This comprises96 architecture edges,104 STATE edges and81 other/renamed/linked document edges. The continuation adds the three JACKIN_GOAL precursor edges and30 JUNIE_PROMPT1/2, JACKIN_REFERENCE and DESIGN edges beyond the original248. `history-source-continuation.md` records full read identities and clause dispositions, not merely filenames. This finite path universe is not a claim that every historical requirement is correctly mapped by counting edges. `history-inline-api.md` closes the full J/K/L/M inline deltas; `history-early-amendments-coverage.tsv` closes85 later older-prefix edges. Architecture completion is the union of disjoint directly read sections, not a claim every reader read every blob. `history-inputs.md` and `history-inputs-api-app-research.md` record the earlier source review of15 linked audit/review documents, including the821-line Slice2 review and its exact nomenclature delta; those reports specify per-version full-read versus delta-read coverage. The current continuation does not claim to have independently reread those15 documents merely because their report rows join. Their HI/HAR rows preserve audit facts versus rejected proposals. `history-ledger-reconciliation.md` specifies namespace, duplicate-equivalent, refinement and supersession joins; `history-inline-obligations.tsv` separately preserves all13 Form invariants. Source-reading completion does not certify present implementation or waive any remaining proof/task mapping.

Task traceability is now explicit: `historical-obligations-canonical.tsv` preserves620 source identities, `history-task-map.tsv` gives their actual task-ID sets, and `traceability-history.tsv` gives1084 requirement/acceptance/check edges. Full canonical source clauses and proof obligations are installed in all73 tasks' protected `trusted/source-obligations.tsv`; the historian's exact payload check found1084 matching edges and zero missing sources, task-set mismatches or field mismatches. Whole-catalog schema and binding checks remain the plan validator's responsibility. The dynamic-disabled capture assessment was corrected against actual implementation `715ee077`, and the old nonoptional backend vocabulary rule is explicitly superseded by `4e84b744`'s owned keys and optional terminal adapter. These are retention obligations, not fresh implementation mandates.

Historical test runs and literal source presence are still not current requirement discharge. Current code, tests and proof mappings belong in the architecture/component/application matrices and final task audit. Preserve negative findings and corrected counts when reconciling an old completion paragraph.

## Supersession rules and unresolved historical ideas

1. **Current PLANNING_GOAL wins.** Earlier implementation prompts, old model restrictions, stop/restart orders, worktree deletion mandates, branch names, allowed product polish, baseline exceptions and old oracle pins do not authorize present implementation or product changes.
2. **Keep accepted architecture.** Borrowed short-lived props, caller-owned durable state, shared-reference draw, typed response/actions, runtime-owned interaction/layers/time, semantic theme recipes, keyed collections, open containers and workspace/API boundaries remain accepted. Replacing main with the old API is rejected.
3. **Later amendments win narrowly.** §72 replaces component-local `state_override`/`inherit_forced` with inert exact-target `Ui::reference`. §70 replaces materialized mono state rules with static resolver fallbacks. K corrects Grid update bounds; §52 preserves domain sorting in adapters. §67 uses semantic `AsItem`, not `Display`/paint output. §50 separates checked state, caller radio value and payloadless AddRequested. §53 rejects full-tree scans; §66 isolates unrelated performance binaries. §74 makes color flags a ceiling. Later backend-free input supersedes unconditional backend vocabulary coupling.
4. **Do not resurrect rejected frameworks.** No universal Widget/Theme trait, owned runtime component tree, boxed untyped action bus, permanent legacy facade, app-owned overlay stack, Grid SQL semantics, duplicate DataTable/ScrollPanel, source-location item identity, or global color-equality reverse lookup. Improvements proposals O01 (generic worker extraction), O02 (Widget/Theme/glyph), O03 (generic keyed collections) and O06 (Container/modal manager) are retired, not unchecked implementation goals. Concrete task ownership remains mandatory where current flows need it.
5. **Rejected patch is not rejected requirement.** Consolidation rejected the item-row override test because production forwarding was missing; the underlying override obligation survives. Showcase Grid/progress WIP was rejected as unreviewed and stale, not as permission to omit the current oracle's composition. Manager rewrite rejection similarly leaves behavior to reimplement through accepted architecture.
6. **Do not mistake old “closed” for current proof.** Historical declarations that Slice 4, porting, or GOAL2 closed describe their own scope. The old 499-recipe archive was 60 exact / 379 mismatches / 60 run failures at the consolidation probe; TablePro startup focus prevented 60 recipes reaching anchors. `REFACTORING_STATE.md:2230` records this explicitly. Re-pointing or blessing failed evidence was refused. The new oracle requires an additional pinned contract and explicit disposition of obsolete comparisons, never silent weakening.
7. **Unresolved proof/design leads:** Choice mono bracket geometry; captured pointer when owner becomes disabled; visible-cell override isolation and RowUi/ColumnsUi forwarding; PARTS query attribution; seed-token cascade; complete ASCII glyph conversion versus border-only ASCII; Props scanner's non-method/cross-module reach; doc-check omission of §§18–20; callback/interior-mutation draw purity; old text-field FIELD query proof versus visible customization. These must be mapped to accepted implementation evidence, a bounded task, or an explicit current-scope disposition.
8. **Security and exact parity need explicit reconciliation.** Historical main fixes must not be discarded reflexively. If a main safety change alters a user-visible oracle flow, create a row naming both behaviors and the precise conflict; do not silently waive the oracle or reintroduce real effects. Holla remains a deterministic simulation. Operational HP01–23 providers, persistence, filesystem deletion and full HP16 CLI are deferred product development, not hidden refactoring deliverables.

## GitHub evidence

Read-only `gh pr list --state all` found one PR; `gh issue list --state all` found no issues. [PR #1](https://github.com/donbeave/terminal-components-claude/pull/1) is merged at `7b27732a`, merge timestamp `2026-09-10T13:21:40Z`. Its body still says draft, acceptance incomplete and PR unmerged. Those sentences are stale. The body remains useful evidence of explicitly outstanding application fidelity, simulation timing, destructive guards, historical mappings, provenance, full gate matrix and independent review. No formal submitted PR reviews were returned; one automated summary comment records completed review at `321f202`. A review-summary badge does not discharge the body’s omissions.

## Recommended eventual integration

At the start of future execution, create an isolated integration branch **from pinned main `7b27732a`**. Integrate independently accepted proof/baseline preparation commits on that same ancestry chain. Freeze executable reference evidence from `02f5294b` and seal exact test dispositions before any production refactoring begins. Preserve main as the architectural starting point. Port/reimplement oracle composition, domain simulations, data and interaction semantics through its public API in bounded dependency order. Use Holla source as behavior evidence and selective source material, not a merge parent to resolve mechanically. This planning goal creates no integration branch.

This choice preserves the already-landed foundations, backend boundary, input/time/publication fixes, caller-state API, generic Grid/forms, architectural gates and substantial prior app migration. Starting from Holla would require rebuilding those accepted invariants while maintaining a second API. A normal merge would combine old `src` and new `crates/apps` trees, revive deleted APIs, and leave extensive rename/delete conflicts where exact-path overlap understates semantic overlap.

Before each transplant, compare against both the main target and oracle source, identifying already-integrated equivalents. Holla domain-only algorithms/data may be copied after dependency/effect audit; rendering, event routing and widget internals must be adapted to the accepted owner boundaries. Carry source/test dispositions together. Keep the reference checkout and target directory separate. Shared runtime/theme/geometry work precedes dependent apps; every app can then preserve its own product composition without duplicating the runtime.

Do not merge candidate into main until full oracle equality, architecture/API gates, test inventories, feature/toolchain/performance gates and fresh independent reviews pass on one exact candidate. Verify the eventual merged commit again. Planning itself does not create that branch, move refs, cherry-pick production commits, bless artifacts or publish a PR.

## Reproducible archaeology commands

```sh
rtk proxy git rev-parse 'holla-fable-2026-09-10^{commit}'
rtk proxy git merge-base main holla
rtk proxy git rev-list --left-right --count main...holla
rtk proxy git log --all --follow --name-status -- GOAL.md
rtk proxy git log --all --follow --name-status -- GOAL2.md
rtk proxy git log --all --follow --name-status -- HANDOFF_SLICE4_WAVE1.md
rtk proxy git log --all -p -- COMPONENT_ARCHITECTURE.md
rtk proxy git log --all -p -- REFACTORING_GOAL.md
rtk proxy git log --all -p -- REFACTORING_STATE.md
rtk proxy git log --all -p -- IMPROVEMENTS_PLAN.md
rtk proxy git diff --find-renames=40% cc14dd6 main -- src crates apps Cargo.toml
rtk proxy git diff --find-renames=40% cc14dd6 holla -- src crates apps Cargo.toml
rtk proxy gh pr view 1 --json title,body,comments,reviews,mergedAt,url
```
