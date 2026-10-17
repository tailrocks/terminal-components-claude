# Termrock finalization specification

**Working branch:** `termrock-implementation`  
**Execution model for the future executor:** Gemini 3.8 Flash · high; exact match, no fallback.  
**Status:** specification and source review, not an implemented migration or a parity certificate.  
**Supersession:** the user's new many-crate requirement replaces the previous one-public-library/one-test-crate limit. Previous preparation-only stop lines no longer define completion.

## 0. Canonical contract owners

This document summarizes the target. It does not duplicate full rules.
Agent entry rules live in [AGENTS.md](AGENTS.md).
The active task lives in [GOAL.md](GOAL.md).
Component-only composition lives in [docs/architecture/component-composition.md](docs/architecture/component-composition.md).
Public signatures live in [docs/api/public-api.md](docs/api/public-api.md) and [types.md](docs/api/types.md).
Consumer examples live in [docs/api/consumer-recipes.md](docs/api/consumer-recipes.md).
Review procedure lives in [docs/process/per-commit-review.md](docs/process/per-commit-review.md).
Ownership and parity proof lives in [docs/verification/ownership-and-parity.md](docs/verification/ownership-and-parity.md).
Component details live in [docs/components/](docs/components/README.md).
Implementation status lives in [checklist.json](checklist.json).

## 1. Completion means all three outcomes together

The result is one Termrock implementation, a genuinely decomposed Cargo workspace, and all four migrated applications demonstrably preserving the immutable `visual-baseline` behavior. Neither new type declarations, namespace replacements, package counts, passing old applications, nor a green ordinary test run establish this result separately.

The existing repository remains the project. All product work stays on `termrock-implementation`. Do not recreate the branch, fork a new project, merge main, rename the GitHub repository, or publish a release as a side effect. Public Rust identity becomes `termrock`; binaries remain `showcase`, `jackin-preview`, `holla`, and `tablepro`.

The applications retain existing simulated operations and domain semantics. No real Jackin account/provider/container services, Holla process execution, or TablePro production database are introduced. Presentation adoption and functional crate extraction are authorized; product redesign is not.

## 2. Authority and reference matrix

| Concern | Authority | Observed/pinned identity |
|---|---|---|
| Working implementation | Current `termrock-implementation` | Inspected `99554cfa49a8c951fb0aebdde61f7fc857f73f9a`; fetch actual head, do not reset it |
| Visual output and existing observable behavior | `refs/tags/visual-baseline` only | Peeled `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; annotated object `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5` |
| Architecture/API comparison | `main`, read-only | Inspected `84482d066c5f0bc531f875f7f9d7716929c1b9c6`; record an actual fixed comparison SHA |
| Detailed intended API | Canonical `docs/architecture`, `docs/api`, `docs/components`, `docs/foundations` | Audit current files; this specification controls latest scope/structure/completion changes |
| Additional generic requirements | Existing family/capture/task inventories | Reconcile all entries; metadata is not execution proof |
| Tuiscotti | Qualified exact source/profile version | Resolve latest main at audit start; any upgrade requires tag-side qualification and explicit generation |
| Velnor Actions | Verified supported generator binary and configuration | Record release asset/source/checksum separately from inspected main |

`main` is never a visual oracle, fixture-value authority, or source of expected images. Use it to study public facades, state/props ownership, module boundaries, shared mechanisms and package organization. Keep an explicit table: main path/pattern → useful contract → keep/adapt/reject → destination → baseline tests protecting the adaptation.

Preparatory commits and imported snapshot corpora are usable evidence only to the extent their relevant source/render/fixture lineage is proved against the tag. They do not become competing visual authorities. Keep a read-only tag checkout outside the candidate workspace. Record any reference-only test instrumentation patch; it may expose state/time/constructors, not alter paint, layout, handlers or fixture results.

## 3. Physical workspace and dependencies

Use the functionality map in [CRATES.md](CRATES.md) and [crate-map.json](crate-map.json), and the 45 public-surface owners in [component-ownership.json](component-ownership.json). The proposed layout contains 44 packages: 19 library/facade packages, four Showcase, seven Jackin Preview, six Holla, five TablePro and three verification/tool packages.

This is not a minimum-number game. Each named responsibility must have meaningful owned implementation, a typed boundary, direct tests and a real consumer. Facades and binary entry points are intentionally thin; other empty forwarding packages fail review. Refine boundaries during G00 using source and dependency evidence, but preserve many library crates and real multiple functionality crates per app. Freeze the accepted inventory before migration; update per-package requirements on additions. Do not collapse the user-requested decomposition to one widgets crate and four monolithic app crates.

Root `Cargo.toml` is a virtual workspace with an explicit resolver appropriate to the pinned toolchain/edition, shared package/dependency/lint metadata, one lockfile and deliberate profiles. No root `[package]`, `[lib]`, `[[bin]]`, `src/`, `apps/`, Rust `tests/`, `build.rs` or implementation tool tree remains. Tests/examples/build scripts live with their owner under `crates/<package>/`. Documents, immutable baseline data and repository config may remain outside `crates/`.

Every normal, build and dev target is accounted for by Cargo metadata, not merely a directory count. No nested unlisted workspace, `exclude` escape, hidden feature, build-script generated legacy source, copied submodule or git/path dependency brings Junie back. An external immutable oracle checkout is not a candidate dependency.

Enforce a dependency DAG for relevant feature/target configurations, including default and minimal builds. Production never depends on conformance/tool crates. Domain packages do not import Ratatui/Termrock. Application UI packages use public `termrock` exports and app domain contracts. Library leaf crates use narrow lower-level contracts; none depends back on the facade.

Cross-crate `#[path]`, `include!`, symlinks or source copying are not module interfaces. Existing tests that directly compile `src/bin/tablepro/{db,sql,model}.rs` must become normal consumers of extracted public app packages. Internal same-crate test modules are fine. Public visibility must expose a deliberate API, not every internal field just to make extraction compile.

## 4. One implementation and one owner per mechanism

The current `src/termrock` code may contain useful behavior, but it cannot remain a module beside `core/runtime/theme/ui/widgets` under `junie_tui`. For every overlapping symbol choose the correct implementation by canonical API and tag parity; do not blindly prefer the newer file. Port proven old behavior into its new owner, migrate callers, and delete the replaced code.

There is one identity/event/response vocabulary, one runtime focus/hit/capture/layer/timing mechanism, one semantic theme resolver, one text engine, one collection reconciliation/scroll engine, one Grid for table and grid recipes, one menu engine, and one picker family. Generic behaviors do not live in app-local replacement widgets.

Preserve caller-owned durable state, short-lived borrowed props and controlled values. Updates produce typed actions; draw and measure cannot execute IO, commit edits, dispatch effects or mutate semantic state. Region publication and cursor requests are explicit frame facts; input must not use stale geometry. Distinguish focus, hover, navigation cursor, selection, press, activation and editing.

Preserve generic customization and exact parts. It is insufficient to call a shared widget and repaint over it. Slot replacements must stay constrained, inherit theme/surface policies and demonstrably affect intended cells. Tests must cover custom rows/cells and every documented part channel.

Read-only models and editable capabilities remain separate. Text retains Unicode/source mapping, bounded history, revision invalidation and exact copy semantics. Single-line versus multiline policies follow the tag rather than a newly uniform convention. Secret drafts cannot follow a blanket Clone/Debug/Serialize rule; test leak paths and keep real secrets out of fixtures.

TerminalView borrows prepared cells. It does not become an emulator, escape parser, PTY or daemon. Session/backend IO is optional and separate. The new crate graph must not pull Tuiscotti fonts/PNG/HTML rendering into the production library.

## 5. App boundaries and migration rules

**Showcase:** separate demo records, demonstration pages, shell/inspector/navigation and thin binary. Keep all 23 observed page identities and inspect source for additional routes. A page that still renders old widgets does not count as migrated because the shell imports Termrock.

**Jackin Preview:** separate domain values, deterministic simulation, shared presentation/action contracts, host screens, Capsule/inspect screens, coordinating application and thin CLI. Avoid cycles between host/capsule and the coordinator by putting shared actions/view contracts below them. Preserve intro/handoff/outro art and source time policy; product art is not a generic Termrock widget.

**Holla:** separate domain, catalog/ranking, plans/activities, simulated world, UI and thin composition-root binary. Preserve aliases, ranking, scope, target-bound gates, dependency outcomes and activity lifetime. Domain plan logic is distinct from the library's generic input runtime.

**TablePro:** separate plain models, SQL analysis/preview, demo database, UI and binary. Remove domain imports of legacy Grid `CellValue`/`CellKind`; map plain domain values to Termrock model views at the UI boundary. Preserve original update/insert/delete order, quoting, original-key predicates, null/default behavior and safety-mode rules through exact vectors.

Migrate vertical slices as foundations become ready; do not leave every app on Junie until the final task. Each slice must show new public components in an actual production screen and retain its tagged frame/behavior. Temporary bridges may exist only during migration with owner, affected callers and removal gate. None survives final acceptance.

The final checks examine imports and dynamic rendering ownership, not just namespaces. Renaming old Junie files wholesale to Termrock without implementing the agreed state/props/runtime contracts is also a failure.

Historical licenses/provenance and literal baseline-visible wording may retain the word Junie. Exempt exact data/text locations, not executable namespaces. Do not change a title rendered by the tag merely to make a text search return zero.

## 6. Baseline and coverage

Source inventories, prior 222 case descriptions, 302 roots and 7,550 size/color captures are starting requirements and evidence, not an exhaustive denominator. Reconcile source routes, states, actions, parts and source-family mappings. New discovered current behavior expands coverage; it does not disappear through a revised count constant.

Classify each expectation by acquisition: original approved artifact, newly captured unchanged tag behavior, or genuinely new extension. Imported/replayed ANSI preserves only what it encoded. Replay cursor/default state and generated approval strings are not proof of original interaction. Capture required missing information from the real tag implementation.

Maintain independent tag-side and Termrock-side adapters using the same stable semantic case contract. Direct component tests establish precise states, actual app fixture tests establish compositions, and PTY execution establishes the real binary/input/session path. No copied tag painter may run in the candidate adapter. A synthetic buffer compared to its clone remains only a comparator unit test.

Every execution receipt identifies case, source/fixture/program digests, dimensions, color/motion, ordered events, observed checkpoints, assertions and real binary identity. Counters are measured events/checkpoints, not hardcoded `passed` strings or promised frame counts. Preserve nonvisual assertions and compare stable observable projections rather than incidental struct layouts.

Required axes include normal/focus/hover/focus+hover/suppressed-hover; down/held/up/cancel/activation; checked/selected/current; disabled/read-only; edit/selection/validation/revision conflict; empty/loading/partial/error/stale; scroll/fade/retention; layers; resize; source reorder/removal; Unicode and masking; and motion boundaries. Define meaningful combinations and finite equivalence classes. Decorative elements have documented non-applicability; a missing test is not non-applicability.

Use established five application sizes and five color modes, component-local degenerate areas and real breakpoint neighbors. Test complete finite animation phase cycles at declared widths and before/at/after deadlines. Use observed readiness and controllable reference/candidate time, not arbitrary sleeps that skip transient states. Keep reduced motion separate from freezing simulated data.

All four applications must have source-derived scenario and journey coverage: Showcase every page; Jackin all host/Capsule/ritual/overlay flows; Holla finder/plan/activity/resources and gates; TablePro connection/workbench/query/table/history/structure/safety. Preserve original baseline coordinate programs; candidate-resolved locators are a separate functional layer and cannot conceal moved hitboxes.

## 7. Exact bundle verifier and approval boundary

Keep the user-requested screen/substep hierarchy, plus a standalone `components` namespace. Every declared visual checkpoint produces `.ansi`, `.html`, `.png`, `.ascii`, `.txt`, `.frame.json` from one observation, plus loss/fidelity/semantic companions and a complete manifest. ASCII is lossy diagnostic information, never the visual equality authority.

Use Tuiscotti native export/comparison APIs at the pinned version. Validate every bundle member, required fields, byte hashes, profile/font/source identities, and cross-artifact generation. Write into run/attempt/platform/shard-isolated actual directories; propagate all write errors and seal the manifest last. Verify intentional empty output using its schema, not a universal nonempty-file rule.

The admitted corpus index enumerates exact concrete captures and hashes their manifests. The admission receipt binds the index, required-case set, tag source, tool/profile and real review evidence. Candidate-controlled JSON flags cannot redefine what counts as approved. Use trusted-base comparisons or pinned external evidence to prevent changing the denominator and approving one's own output in the same change.

Normal verification reads expected PNG bytes and structurally compares canonical cells, dimensions, wide continuations, cursor, supported modifiers and semantic observations. Exact means zero decoded-pixel changes under the admitted profile, not a high perceptual score or equality of a short digest. Report the first actual cell/state differences.

Raw ANSI/TXT comparisons preserve whitespace. Tool-normalization exceptions must be explicitly qualified on unchanged tag inputs, not silently normalized on candidate output. HTML must represent the same frame/generation. Supplemental tests cover properties that the toolkit normalizes/drops; do not falsely claim their PNG proves them.

Expected writes are restricted to separate explicit tag-derived capture/admission operations. Ordinary unit/Nextest/CI execution cannot regenerate or approve the real store. Disposable temporary-store acceptance tests may test the approval API without becoming an admission path for product data.

When missing coverage is discovered mid-refactor, derive an additive expectation from the unchanged tag, review it independently, and retain prior requirements. A product defect in the tag cannot be silently fixed under a parity waiver. New API semantics are separately accepted extensions, and all mandatory extensions must be implemented/tested at final closure.

## 8. Build and CI gates after splitting packages

Use explicit workspace operations; `default-members` convenience must not hide packages from CI. Discover actual target/package identities through Cargo metadata and machine-readable build output. Cross-package end-to-end tests must receive paths/digests of freshly built candidate binaries, not accidentally execute a reference or old `target/debug` binary.

Run normal tests with Nextest, preserved ignored visual tests explicitly, minimal library/backend feature checks, all-target compilation/lints, public API examples and compile-fail/negative cases. Do not select baseline staging/admission writers with a blanket `--run-ignored all`. List and reconcile the intended executable set before sharding.

Velnor generates workflows from supported configuration; never hand-edit generated YAML or fake a generator identity. Regenerate after each workspace-shape change and inspect actual package/task selection, generated-file reproducibility and final run artifacts.

The inspected Velnor publication error `publish_refused:unprotected_ref` belongs to reusable CI-result publication, not visual admission. Respect the guard: select supported eligible publication behavior or fix the generator generically. Do not weaken protection, forge event identity, change branch protection without authorization, or suppress required visual failures. Noneligible optional cache publication must not be mistaken for a completed visual gate.

Cache actual Mise/Cargo homes and compatible builds. Initial qualification executes real captures. Include checkout/corpus download/materialization costs in end-to-end timing; 75,000 artifact files can dominate transport. Optimize artifact materialization and storage with content hashes and explicit integrity without hiding required data behind unverifiable caches. Never drop formats or cases for speed.

Measure clean/warm full builds and representative leaf/app changes to show the crate split improves incremental scope. The two-minute CI target is a measured objective, not a justification for false success. Use bounded PTYs and compatible prebuilt test binaries. Execute native Linux and macOS where required; report actual platform coverage.

Final Required validation reconciles all concrete case/checkpoint IDs, shard results and artifact hashes. Missing/cancelled/stale/required-skipped results fail even if other tests passed. Upload current-run failure evidence and verify final pushed-commit CI, not an earlier green revision.

## 9. Checklist state and evidence contract

Use [CHECKLIST.md](CHECKLIST.md) and [checklist.json](checklist.json): 160 initially unchecked mandatory rows, including one per proposed crate and one per component. These are completion obligations, not a fixed test count. Integrate them into the repository's existing task format and canonical docs rather than creating a competing agent platform.

Allowed states are pending, in_progress, blocked and verified. A row becomes verified only when code, actual tests, artifacts and required review satisfy its acceptance. Required top-level rows have no automatic waived/not-applicable success state. Non-applicable sub-axis or boundary refinements require evidence but cannot remove a user outcome.

Receipts identify subject commit or source-tree/input fingerprint, command/tool/runner, expected/executed case set, result, artifact digests, reviewer and resolved findings. Reopen stale evidence when code/contracts/dependencies change. Final all-green status is recomputed for the final subject tree, not inferred by adding historical passes.

Avoid circular certificate commits: a final external CI artifact binds the tested final commit; a checklist view can reference the stable code-tree fingerprint and earlier evidence while final acceptance is established externally. Any subsequent relevant source/config change invalidates the final receipt and requires re-verification.

A small `termrock-xtask` gate command may validate these contracts, but must execute real checks or validate signed/pinned real receipts; it cannot emit success from literals or create its own independent review. Exit nonzero and enumerate outstanding IDs whenever any mandatory row is unchecked, failed, blocked, stale or missing proof.

## 10. Final stop line

Do not stop at crate scaffolding, a green synthetic parity test, P7-named commits, or a narrative review packet. Continue through implementation, real consumer replacement, deletion of all legacy paths and final exact conformance. Use fresh real Gemini 3.8 Flash/high reviewers for API/crate, application visual/coverage and verifier/CI security.

Completion requires a clean candidate workspace with no Junie implementation or imports; all functionality packages under crates; all 45 documented surfaces and all required foundations implemented; all four production apps using them; all required tag parity and separately specified extensions checked; zero open mandatory checklist items; generated CI verified at final head; and factual evidence. External tool/model/permission blockers remain blockers, not fabricated completion. No unauthorized merge, release or GitHub rename is part of this goal.
