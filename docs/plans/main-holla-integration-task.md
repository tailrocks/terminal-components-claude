
Finish the architectural refactoring of https://github.com/donbeave/terminal-components-claude, repair the broken main-branch implementation, restore the complete accepted TUI experience from holla, migrate the Holla application to the refactored public API, and integrate the verified result into main.

This is an implementation-and-verification goal, not an analysis-only assignment. Continue through investigation, plan, implementation, application migration, independent review, corrections, and verified integration. A buildable library with broken applications is failure. Matching screenshots with broken interactions is failure. Restoring the old monolith instead of completing the architecture is also failure.

## 1. Direction and authority

MAIN IS THE IMPLEMENTATION BASE. Preserve and finish its reusable architecture and workspace. HOLLA IS THE PRODUCT REFERENCE, not a replacement implementation tree. Work on an integration branch descended from main; do not reset main to holla, overwrite main with holla's tree, or reinstate a permanent legacy component API.

Inspected references, to revalidate at execution start:
- main: c12cad8728755cd2d03eefdd8e02891143fca86d
- holla: 794b095c196562d38f1b6f7ce379c128af2a023d

Fetch current refs and record immutable MAIN_BASE and HOLLA_REFERENCE commits. Inspect any changes since these inspected revisions before adopting newer tips. Never silently compare against a moving branch.

The required result is the conjunction of two contracts:
1. Product: preserve holla's accepted appearance, content, geometry, keyboard/mouse behavior, timing, CLI behavior, workflows, and simulation/safety boundaries, interpreted with DESIGN.md.
2. Architecture: complete the original reusable-component goals and their valid subsequent corrections, including state ownership, public API, composition, theming, runtime ownership, domain boundaries, and testing.

Where an architectural detail cannot represent the product contract, fix the abstraction with an evidence-backed decision; do not simplify the product to fit an inadequate API. Breaking the experimental Rust API is permitted where necessary. Unexplained user-visible changes are not.

Read GOAL.md, REFACTORING_GOAL.md, COMPONENT_ARCHITECTURE.md, REFACTORING_STATE.md, COORDINATION.md, both branches' DESIGN.md, relevant audits/reviews, and holla's recovery reports. Explicitly supersede the holla-first/tree-replacement instructions in HOLLA_REFACTOR_RECOVERY_PLAN.md and holla-project/notes/main-vs-holla-refactor-analysis.md. Their findings are investigation leads, not authority to abandon main. Historical three-application scope and developer-machine absolute paths are also outdated.

This prompt supersedes historical Claude/Fable/Opus-specific execution restrictions. Use actual available Codex capabilities and configured models. Preserve independent architectural judgment and review without blocking on unavailable historical model names or inventing agents/tools.

Preserve valid main-only security fixes and additive capabilities. A reference defect does not authorize credential disclosure, unsafe effects, or inaccessible controls. Any necessary behavioral exception must be narrow, reproduced, tested, documented, and independently reviewed; it cannot excuse unrelated visual drift.

## 2. Safe execution and parallel ownership

Inspect repository instructions, status, worktrees, remotes, branches, toolchains, and uncommitted changes before writing. Preserve unrelated work, existing target directories, and other agents' changes. Never use destructive reset/clean, force-push, blanket conflict resolution, or unrelated branch deletion.

Create isolated reference and candidate worktrees with separate build, capture, and artifact directories. Keep the pinned holla source and original reference artifacts immutable. Do not confuse binaries sharing the same name across worktrees.

Use subagents aggressively for independent bounded work: history/obligation audit; runtime and identity; theme/layout/components; verification/CI; Showcase; TablePro; Jackin; Holla; independent review. Give each writing task one branch/worktree, explicit owned paths, base SHA, dependencies, contracts, and acceptance tests. Shared runtime files, manifests, public exports, and baseline approvals have a single writer at a time. One integrator owns integration.

Maintain a resumable execution plan with dependency-ordered tasks and a compact evidence ledger. Each task records requirement IDs, source references, ownership, reproducer, tests, commands/results, artifacts, reviewer findings, and candidate SHA. Distinguish implemented, reviewed, verified, and integrated. Do not turn the plan into another unbounded contradictory adjudication log. Follow the repository's ExecPlan convention if present.

Parallelize characterization and disjoint implementation, but settle shared interfaces before dependent workers invent incompatible designs. Commit coherent changes frequently. Reviewers verify committed candidates independently. On a blocked task, continue independent work while resolving its dependency; never manufacture a pass.

## 3. Reconstruct the original plan and every relevant historical change

Review the complete reachable history of COMPONENT_ARCHITECTURE.md and REFACTORING_GOAL.md, including creation, renames, deletions, merge-parent differences, reversals, and later supersessions. Do not substitute current prose, commit titles, or existing audit summaries for patch review.

Use full-history Git inspection, for example:

git log --follow --name-status "$MAIN_BASE" -- COMPONENT_ARCHITECTURE.md
git log --follow --name-status "$MAIN_BASE" -- REFACTORING_GOAL.md
git log --root --full-history -m -p --find-renames "$MAIN_BASE" -- COMPONENT_ARCHITECTURE.md REFACTORING_GOAL.md

Cross-check discovered changes against reachable commit/parent trees so history simplification does not hide obligations. Index every relevant revision and parent-relative patch; review unique snapshots and semantic deltas. Record formatting-only changes and path absence explicitly. Review DESIGN.md divergence and all holla-only changes as well.

Produce an obligation map: original requirement -> amendments/supersessions -> current implementation -> gap -> concrete acceptance test. Record where failures were introduced, separating observed reproduction, source-proven defect, historical report, and unverified hypothesis. Use targeted bisects when feasible; isolate unrelated compiler failures rather than misclassifying them as the behavioral regression.

Investigate the initial architecture, application migrations, legacy deletion, facade enforcement, and later parity recovery. Known history leads include 2e45302, 18afddd, 7784719, 4e07ea1, 5042a40, 444a8f4, 1378c31, 444ae3d, and 8831a62; they are leads, not substitutes for complete history.

Consolidate the current normative architecture and execution goal without losing obligations. Archive superseded prose with traceability. Never weaken a requirement merely because current code or a snapshot already violates it. Complete the relevant history/contract review before architectural replacement or legacy deletion; read-only baseline work and a minimal compiler repair may proceed concurrently.

## 4. Establish honest baseline health and independent reference evidence

Build and test the pinned reference and initial main separately. Save actual commands, toolchain versions, discovered/executed test counts, failures, and logs. A report saying tests once passed is not current evidence.

Create exact inventories of every component, all application routes/scenarios, CLI options, meaningful interaction states, and branch-only changes. Map each to its migration destination and required proof. The application inventory is four binaries: showcase, tablepro, jackin-preview, and holla. Preserve their existing command-line entry points.

Inventory baseline/before, its historical 499 recipes, parity/recipes.tsv, screenshots, capture scripts, and required artifact files. Verify actual availability and hashes rather than trusting prose. Keep the historical recipe namespace separate from a new pinned-holla acceptance namespace. Historical captures from another revision cannot silently become current-holla expectations.

Missing historical artifacts must be recovered or explicitly regenerated from their exact pinned historical source with honest provenance. Do not fabricate original captures, copy candidate output into expected files, or reduce coverage to make an incomplete archive pass. Make artifact acquisition/generation reproducible in a clean checkout.

Capture holla's actual production views and real-input journeys before changing expectations. Pin viewport, theme/color capability, fixture data, clock/motion/frame, environment, terminal engine, rendering profile/font, executable hash, and source revision. Keep reference and candidate outputs distinct. Add observation/clock adapters only when necessary, isolated and reviewed so they do not change reference behavior.

## 5. Repair the build, then shared causes—not cosmetic symptoms

Investigate the missing overlay_chrome import/call introduced in 8831a62. Recover the intended implementation from main's prior inline dialog chrome or implement a correct shared helper. Preserve validation errors, surfaces, borders, clipping, registration, and body/action layout. Do not add a no-op stub or remove functionality just to compile. Rerun the complete checks to expose subsequent failures.

Complete the architectural foundations:
- Caller-owned durable component state, short-lived borrowed props, typed semantic actions, and Response semantics that distinguish consumption, repaint, and layout invalidation.
- Separate update and draw. Drawing must not commit edits, validate/stage domain operations, navigate, advance clocks, open/close overlays, or execute effects. Enforce API boundaries and test repeated-draw state/effect invariance; shared references alone are not sufficient proof.
- One runtime owns global focus, hover suppression, press/release/flash, capture, routing, cursor, modal layers, and scheduling. Application code owns product intent, composition, and domain state.
- Stable namespaced control/item identity across insertion, deletion, sorting, filtering, and reordering. No durable logical selection keyed only by display position.
- One authoritative layout supplies painting, clipping, focus facts, hit regions, scrolling, and cursor geometry. Commit presented geometry consistently. Handle first input, resize, route changes, removed controls, and queued input without routing against stale incompatible geometry.
- Correct typing/editing versus global shortcuts, paste handling, wheel ownership, pointer capture, disabled controls, focus restoration, and nested-modal barriers. No background activation through overlays.
- Explicit monotonic time and wakeup/repaint scheduling. Rendering and arbitrary event counts must not advance elapsed-time behavior. Preserve reduced/paused-motion contracts and actual live timing.

Fix shared component/runtime causes before scattering app-local patches. Product-specific drawing through the public author API is legitimate; repainting a second incompatible generic control over registered geometry is not.

## 6. Restore the full design system and finish every component family

Derive the visual/state contract from pinned holla source and DESIGN.md: exact Junie tokens, semantic color roles, state precedence, surfaces, backdrop dimming, modifiers, glyphs, spacing, borders, row anatomy, copy, wrapping, truncation, responsiveness, and cursor treatment.

Preserve focused versus selected/current versus hovered versus pressed behavior, including their combinations; disabled, busy, editing, error, and inactive states must remain meaningful. Preserve keyboard hover suppression and activation feedback timing. Test truecolor, ANSI256, ANSI16, and monochrome, including semantic glyph/modifier distinctions where hue disappears. Retain and verify the non-Junie theme independently; do not pretend it has a pre-refactor Holla baseline.

Prove all advertised customization layers: partial semantic tokens; family/variant; subtree scope; instance; logical parts; interaction states; custom content/row/cell renderers. Verify precedence, surface inheritance, and that overrides actually affect intended cells without changing unrelated instances. Use sentinel overrides and externally compiled examples, not builder calls with no effect.

Cover every existing family and reusable app-side control through the obligation map: buttons/choices; inputs/textareas/editor/forms; lists/navigation/trees/tabs/chips; tables/grid; viewport/diff/scroll/split; menus/pickers/completion; panels/dialogs/help/wizards; status/hints/progress and remaining families. Preserve rich capabilities rather than replacing them with demonstration-level approximations.

Keep collections borrowed and efficient, with stable keys and real renderer hooks. Retain the backend-free public core and reusable testing boundary. Keep SQL, connection models, providers, capsules, host/action ranking, and other product vocabulary/behavior in application adapters. Demonstrate custom component authoring with public API only.

Remove compatibility overpainting, meaningless calls added only to satisfy source scanners, and duplicate generic interaction logic once their real replacements pass. Fix inadequate static checks rather than contorting production code around them. Do not remove useful safety or boundary enforcement.

## 7. Restore and migrate all four applications

Showcase: cover all 22 pages and the complete shell, including navigation, header, inspector, contextual footer, transient status, help, editing, animation, and responsive layouts.

A concrete first regression: NavList registers sectioned geometry, then paint_sidebar paints compact geometry over it. At 80x24, source inspection predicts a click at zero-based (4,3), visually on Buttons, resolves to Overview. Confirm against both binaries and fix with one shared layout/configurable compact policy. Test every visible row, gaps, clipping, hover, focus, and resizing, including terminal heights 30 and 31. Expected coordinates must come from the reference contract, not the candidate hitmap.

Another concrete regression: Buttons replaced a 2,200 ms deadline with busy_frames decremented on every update. Restore elapsed-time and status semantics. Prove busy at 2,199 ms and one completion at/after 2,200 ms; 1,000 unrelated input events without clock advancement cannot finish it; idle time can; repeated draws cannot. Verify repaint and global feedback, not only page-local text.

TablePro: preserve connection management, explorer, document tabs, editor/results split, completion, data/structure views, sorting/filtering, scrolling/resizing, editing/validation, null/default distinctions, pending changes, undo/revert, SQL preview, history, and safety workflows present in the reference. Keep domain semantics in adapters over generic components. Prove named fixtures reach their actual renderers, and separately prove user-input reachability and effects.

Jackin: preserve every scenario and journey, manager/editor/accounts/usage/cockpit/Capsule surfaces, menus, overlays, timing, diagnostics, and responsive behavior. Restore the reference CLI defaults, aliases, help/error exit semantics, environment interpretation, and explicit-option precedence. Specifically recheck FirstUse versus Returning and JACKIN_NO_MOTION absent/empty/0/1. Help/errors must not enter raw mode.

Holla: forward-port src/bin/holla into apps/holla using the completed public API and shared runtime. Preserve all eleven scenarios: first-use, rust-dirty, monorepo-root, monorepo-child, docker-cleanup, disk-cleanup, upgrade-plan, activities-multi, remote-host, launch-failure, hard-cases.

Preserve always-armed query, context rings, explainable ranking, pin/alias/hide/reset and exact-query resurfacing, menus/previews, activity navigation and tones, merged logs, host identity, plan DAGs, dependency recalculation, typed confirmations, and truthful success/failure/cancellation. Preserve the existing 132-case capture matrix and extend missing color/state coverage rather than treating it as exhaustive.

Holla and Jackin remain deterministic simulations. Never turn simulated git/docker/ssh/mise/provider/filesystem operations into real external effects. Verify with effect/spawn guards and adversarial tests. Secrets must not leak through views, diagnostics, Debug, errors, snapshots, traces, or artifact metadata.

Account for every holla-only code/design/doc/script/asset change, including warning glyph behavior and capture tooling. Adapt useful changes to the new architecture. Preserve one implementation of each shared behavior and exactly one Cargo binary target per public executable.

## 8. Build three complementary verification layers

A. Pure model-to-view tests: render actual production views from explicit fixture models, UI state, viewport, theme, and clock without executing business effects. Save canonical cells and cursor data plus readable expected/actual/diff artifacts. Compare glyphs, widths, foreground/background, all relevant modifiers, blank cells, clipping, geometry, and cursor—not text-only or unexplained hashes.

B. In-process behavioral tests: drive the real runtime/update path; assert focus order, hit targets, actions, editing, selection, scroll ownership, timing, layer/capture state, domain outcomes, and effect counts after important steps. Fixed-model fixtures do not prove reachability; interaction tests do not replace pure-view coverage.

C. Terminal-process tests: launch the actual compiled binaries and verify CLI, keyboard, pointer coordinates, wheel/drag, paste, resize, startup/shutdown, and terminal restoration. Use controlled readiness and clocks where supported, bounded waits, explicit exit checks, isolated session names, and failure artifacts—not sleeps that hide races.

Evaluate and pin current donbeave/tui-snap for canonical frame/visual evidence and microsoft/tui-test for independent terminal-process interaction. Reuse existing Rust harnesses rather than rebuilding everything. Inspect actual supported versions/APIs first. Keep any newer-toolchain test tooling isolated from the library's Rust 1.88 promise; do not add an incompatible test dependency to its MSRV build.

Qualify the chosen tools against DIM/bold/reverse, color modes, wide/combining Unicode, cursor, mouse coordinates, clipping, and frame settling. Do not normalize away meaningful differences. Canonical cell/style data is authoritative for deterministic comparison; rendered images additionally require inspection. A deterministic offscreen renderer is not automatically pixel-identical to a terminal emulator.

Require independent before/after/diff review of changed surfaces and representative live flows across all applications. No fabricated human approval, automatic acceptance of candidate snapshots, or reviewer records unbound to actual artifact hashes.

## 9. Make the gates capable of detecting failure

Add adversarial/mutation tests demonstrating detection of: a one-cell hitbox offset; incorrect compact geometry; dropped focus/hover/disabled style; missing status; event-count timing; draw-time domain mutation; modal input leakage; wrong destructive effect; inert customization; missing required test; zero-match filter; duplicate/missing recipe; stale/wrong-binary capture; missing reference; and secret leakage.

Enumerate required test targets/cases before execution, including binary-local, feature-gated, doc, integration, and relocated legacy tests. Check expected identities, not just aggregate counts. A virtual-workspace root test file is not coverage unless Cargo actually runs it. Every removed test obligation needs a surviving mapping.

Repair CI and capture provenance. In particular, perf.yml pipes Cargo through tee without explicitly enabling pipefail. Specify the shell and failure handling; prove a deliberately failing piped command fails the job. Advisory wall-clock benchmarks may remain advisory, but compilation, correctness, and declared allocation/byte gates must be binding.

Generate or fetch immutable hash-verified evidence for the exact candidate before parity checks consume it. Missing evidence, stale fingerprints, skipped required jobs, empty test sets, failed capture commands with old output files, or unreviewed differences must fail. Add Holla to dispatch, coverage, binary-identity, capture, and CI inventories.

Run the declared toolchain/feature matrix, including at minimum:

cargo metadata --locked --format-version 1
cargo +1.88.0 check --locked --workspace --all-targets --all-features
cargo +stable check --locked --workspace --all-targets --all-features
cargo +stable build --locked --workspace --all-targets --all-features
cargo +stable fmt --all -- --check
cargo +stable clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo +stable test --locked --workspace --all-targets --all-features -- --list
cargo +stable test --locked --workspace --all-targets --all-features
cargo +1.88.0 test --locked --workspace --all-targets --all-features
cargo +stable test --locked --workspace --doc --all-features
RUSTDOCFLAGS='-D warnings' cargo +stable doc --locked --workspace --all-features --no-deps

Also run default-feature and supported isolated feature combinations, backend-free/public-consumer checks without workspace feature-unification masking, all actual xtask boundary/parity/capture gates, terminal tests, and existing performance gates. Verify command names against the real tool; implement missing required functionality rather than inventing successful commands. Record resolved toolchain versions and meaningful performance comparisons. Do not lower thresholds or bypass lint/security checks merely to pass.

## 10. Integrate and prove completion

Integrate in bounded dependency-ordered slices: characterize -> failing reproducer -> shared fix -> migrated caller -> independent view/behavior verification -> review -> integration. Require continuous cross-application smoke tests. Retire old paths only after their obligations are covered, then rerun after deletion.

Forward-port and reconcile Holla branch work through an explicit migration/disposition map and controlled merge. Preserve main's architecture and valid improvements while adapting Holla functionality. Do not use ours/theirs blanket strategies, fake ancestry-only merges, or old-tree replacement as a substitute for content integration. Preserve the reference branch/history.

Use a PR/integration branch, respect repository protections, and merge to main only when the complete acceptance set passes on the resolved candidate. Revalidate against any concurrent main changes. Run final verification on the exact committed merge result, not just predecessor worktrees. Preserve source/evidence identity without modifying the tested source merely to attach a report; attach external verification artifacts to that commit.

Completion requires all four applications genuinely usable with accepted appearance and interactions; all surviving architecture obligations implemented; one coherent public API; Holla fully migrated; no material stubs, duplicate legacy system, or unexplained regression; meaningful customization/public-consumer proof; safe simulations; and required gates actually executed and passing.

Deliver the history/root-cause and obligation maps, changed architecture decisions, application/component coverage, reproduction commands, test counts/results, reference/candidate/diff evidence, review resolutions, measured performance, and final commit/PR/main-merge status. Clearly distinguish any external permission/artifact/tool blocker from completion. Never claim a merge, visual inspection, subagent execution, test run, or parity result that did not happen.
