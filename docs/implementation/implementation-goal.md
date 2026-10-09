# Preserved implementation mission (inactive historical prompt)

> This file is an inactive historical prompt. It is not a live specification and it is not policy.
> Do not follow it. None of it is followed, including the clause that would supersede source-path protections and including its model, provider, and reasoning-level lines.
> Do not restore this file into `GOAL.md` or `AGENTS.md`.
> Root `GOAL.md` is the active task. This file does not supersede it.

---

/goal
# Finish the Termrock migration: real crate boundaries, zero Junie implementation, exact visual-baseline parity

## 1. Fixed mission and authority

Execute to completion in:

```text
Repository: https://github.com/tailrocks/terminal-components-claude
Working branch: termrock-implementation
Coordinator and ALL agents: Gemini 3.8 Flash
Reasoning level: high
Fallback: prohibited
Visual oracle: refs/tags/visual-baseline
Oracle commit: 4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
Annotated tag object: 1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5
```

Finish the actual refactoring, not another preparation report. The final repository must contain one real Termrock library implementation distributed across functionality-based crates, and four genuinely multi-crate applications using it exclusively. Every required visual/interaction case must pass against the immutable tag. All mandatory checklist items must be verified with current evidence.

This goal supersedes previous one-library-crate limits, preparation-only stop lines and source-path protections that prohibit the requested migration. It does not supersede exact visual behavior, immutable references, model policy or safety boundaries.

Keep `termrock-implementation` for the whole job. Do not recreate/reset it, make another product branch/repository, merge/rebase from main, force-push, rename the GitHub repository, or publish a release. Preserve unrelated changes; use a safe worktree without taking over someone else's checkout.

The inspected implementation was `99554cfa49a8c951fb0aebdde61f7fc857f73f9a`; inspected main was `84482d066c5f0bc531f875f7f9d7716929c1b9c6`. Fetch current refs and record fixed analysis SHAs. These observations are not instructions to discard later work.

Use **main only for structure/API/ownership comparison**. Never use main's images, runtime appearance, fixtures or changed behavior as expected output. Do not copy its renderer wholesale. Visual truth is only `visual-baseline`; preparation corpora require independently proved lineage to that tag.

Do not move, delete, recreate or retarget the tag/release. Keep tag-side execution outside the candidate workspace. Candidate artifacts must not depend on or secretly run old Junie code.

## 2. Execution discipline and completion ledger

Verify the exact Gemini 3.8 Flash/high configuration through actual runtime controls before work and at subagent creation. This replaces earlier model instructions for this repository. Do not invent a provider ID or silently substitute another model. Report a genuine unavailable/unverifiable model as a blocker.

Use subagents aggressively with independent source/architecture, library mechanisms, per-app migration, reference/coverage, CI, and fresh-review workstreams. Give workers bounded writable paths and shared API owners. The coordinator integrates changes and resolves disagreements from source/tests. Review scripts or unit tests named “independent review” are not independent agents.

Read current AGENTS/GOAL, canonical docs, all relevant task packages, component/foundation inventories, capture plans, drivers and CI. Integrate this specification into those canonical locations instead of leaving competing plans.

Adopt the accompanying `SPECIFICATION.md`, `CRATES.md`, `CHECKLIST.md`, `checklist.json` and ownership map into the canonical implementation plan. The initial checklist contains 160 mandatory rows: 71 cross-cutting closure requirements, 44 proposed package requirements and 45 component requirements. Source discovery may add rows; these counts are not test counts or a ceiling.

Use one authoritative machine-readable ledger with a derived Markdown view. Each row records required outcome, dependencies, owner, status, code/input fingerprint, concrete tests/checkpoints, evidence artifact hashes, reviewer and findings. States: pending, in_progress, blocked, verified. No unchecked, stale or blocked mandatory row is success.

A row becomes verified only after real implementation, execution evidence and the required review. Do not tick a box because a symbol exists, a file was moved, a test was named appropriately, a receipt says “passed,” or another agent claims completion. Reopen rows when relevant code/contracts/dependencies change. Final acceptance revalidates the final subject tree and complete expected case set.

Never delete/downgrade requirements or redefine the denominator to finish. A justified non-applicable state axis is not permission to waive a required component or app migration. Use the existing task format where applicable, not a new agent/workflow platform.

Work autonomously without routine questions. Resolve ambiguity by source inspection and independent review. Continue useful authorized work around external blockers, but report them accurately; never fabricate evidence or bypass a gate.

## 3. Audit unfinished work before migrating further — G00

Recheck and fix the observed issues rather than treating P7-named commits as completion:

- Root Cargo still defines `junie-tui` / `junie_tui` with all four binaries; there is no finished Termrock workspace.
- `src/lib.rs` exports legacy core/runtime/theme/ui/widgets beside `termrock`.
- Actual application rendering, including the inspected Jackin manager and Showcase paths, still imports legacy Junie widgets.
- New Termrock code has dependencies on legacy core types; a facade rename will not remove dual ownership.
- Inspected conformance drivers exercise Junie. Synthetic buffer-clone parity tests qualify comparators, not tag-versus-Termrock rendering.
- Tests directly include old app source and require `src/bin` to exist. Replace obsolete layout assertions without deleting behavioral obligations.
- The visual gate reads/comparisons only part of the bundle, uses digest/score shortcuts and emits incomplete difference information. Authenticate the entire approval chain and all required observations.
- CI still selects the monolithic package and does not explicitly run the ignored visual matrix. The inspected latest run failed Velnor publication with `publish_refused:unprotected_ref`, after Required passed.

Inventory every first-party Rust file/target, duplicate type/mechanism, consumer import, feature/build path and test population. Produce a migration map: source module → canonical behavior/API → owning destination crate → consumers/tests → removal condition. Inspect all source rather than sampling only facades.

Compare main's workspace, app library entry points, private implementation modules, public exports, state/props boundaries and test package. Record keep/adapt/reject decisions and protecting baseline cases. Main's six-package structure is useful evidence, not the final many-crate layout requested here.

## 4. Target workspace specification — G02

All first-party Rust implementation, application code, tests, examples, build scripts and repository Rust tools must live inside `crates/<package>/`. Root Cargo is a virtual workspace with explicit resolver, shared dependency/lint/package configuration and one lockfile. No root package, root src/apps/Rust-tests tree, detached nested workspace or excluded legacy target survives.

Implement this functionality-based default decomposition; detailed responsibilities and allowed edge directions are in `CRATES.md`:

```text
Library and facade (19):
  termrock-core          termrock-layout        termrock-theme
  termrock-text          termrock-render        termrock-collections
  termrock-runtime       termrock-controls      termrock-fields
  termrock-feedback      termrock-navigation    termrock-viewport
  termrock-overlays      termrock-grid          termrock-editors
  termrock-forms         termrock-terminal      termrock-session
  termrock

Showcase (4):
  showcase-data          showcase-demos         showcase-ui
  showcase

Jackin Preview (7):
  jackin-preview-domain  jackin-preview-sim     jackin-preview-presentation
  jackin-preview-host-ui jackin-preview-capsule-ui
  jackin-preview-app     jackin-preview

Holla (6):
  holla-domain           holla-catalog          holla-plan
  holla-sim              holla-ui               holla

TablePro (5):
  tablepro-domain        tablepro-sql           tablepro-demo
  tablepro-ui            tablepro

Verification/tooling (3):
  termrock-test-support  termrock-conformance   termrock-xtask
```

This is not a quota satisfied by empty wrappers. Freeze the source-reconciled map before moving code. Any refinement needs explicit ownership/dependency evidence, checklist updates and independent review; it cannot collapse the requested library/app decomposition. Only the facade and thin binary entry points are intentionally forwarding packages.

Domain/simulation packages contain plain data/rules and no UI dependency. UI adapters map those data to generic Termrock models. In particular, TablePro SQL/domain must not depend on Grid cell types. Preserve exact SQL-preview and safety vectors.

Library dependencies flow downward; leaf crates never import the facade. Runtime cannot depend on component crates. Fields do not depend on overlays; Select is an overlay composition. Dialog/menu slots prevent reverse imports from Grid/editors/forms. Shared app presentation/action contracts sit below host/capsule views and coordinating apps.

No `#[path]`, `include!`, alias, symlink, generated source or copied old painter may bypass a crate boundary. Same-crate test modules are fine. No production package depends on conformance/xtask, Tuiscotti or raster/fonts. Keep terminal session/backend dependencies optional.

Use Cargo metadata and all relevant feature/target graphs to verify real package membership and dependencies. Do not rely on folder names or default-members. Every nontrivial crate needs local tests and actual downstream consumers.

## 5. Repair and expand the tag-derived verification gate first — G01

Before changing an unprotected behavior, make its reference contract executable. Do not implement the whole library on a weak harness and hope to compare at the end.

Use Tuiscotti's latest main resolved once at audit start, then pin its exact SHA and full renderer/font/profile identity. Qualify upgrades separately on unchanged tag inputs. Do not silently advance pins during a generation or use main as a visual reference.

Classify imported ANSI reconstruction as conversion evidence, not a live scenario or original cursor/focus capture. Reacquire missing information from real tag widgets/apps through audited behavior-neutral fixture/time adapters. The old source stays in an external read-only checkout, never a candidate package dependency.

Bind every requirement to a real driver, event/time program, machine-checkable semantic assertions and concrete checkpoints. Capture the same semantic scenario independently through the tag adapter and the candidate public API/real application. Hardcoded event/frame counters and boot-only programs are not coverage of richer interactions.

Preserve all existing correctness assertions while migrating them. Current 45 surfaces, 12 foundations, 54 families, previous 222 case descriptions/302 roots and 7,550 size/color captures are floors to reconcile—not proof or hard ceilings.

Cover all 45 required component/API surfaces:

```text
Brand, Button, Checkbox, Toggle, RadioGroup, ChipBar, Field,
TextInput, TextArea, Select, Form, List, FilterList, NavList, Tree,
Steps, Tabs, Picker, CommandPalette, PickerChain, Completion,
Dialog, Menu, ContextMenu, MenuBar, HelpOverlay, Wizard, Grid,
CodeEditor, DiffView, TextViewport, Panel, SplitPane, Props,
PropsList, Empty, ProgressBar, Spinner, Meter, StatusBar,
HintBar, KeyHint, TooSmall, TerminalView, ScrollRegion.
```

For proposed wrappers, capture the current equivalent composition. Genuine new-only API behavior has a separate Extension contract; it must be implemented/tested by final acceptance rather than left planned. Do not label missing current behavior as an extension to evade parity.

Cover applicable variants and focus/hover/suppressed-hover, gesture down/held/up/cancel, current/selected/checked, disabled/read-only, editing/selection/validation, revision conflict, empty/loading/partial/error/stale, scrolling/fades, layers, resizing, source changes and motion.

Test combinations that expose bugs: disabled/removed/reordered target between down and up, drag+resize, nested modal+paste, focus loss while editing, scroll/selection after append/eviction and below-minimum resize/recovery. Assert exactly which actions and targets occur and which must not occur.

Retain five established application sizes and five color paths; add component-local tiny/zero/nonzero-origin areas and real breakpoint neighbors. Preserve Unicode graphemes/continuations/tabs, raw-source copy, protected inputs and the distinct single-line/multiline Escape/paste behavior.

Capture finite animation phase cycles and wraparound, status changes, pause/reduction and before/at/after deadlines, including applicable 0/139/140/141-ms feedback. Do not invent a universal spinner cadence. Controlled time and observable readiness replace sleeps as correctness arguments. Reduced motion must not freeze simulated data completion.

Define finite equivalence classes and reproducible property sequences for unbounded input spaces. No false claim of literally testing every possible input; no omission of a documented state/variant/transition.

## 6. Migrate real consumers and remove Junie — G03/G04

Implement the accepted API: caller-owned state, borrowed props, controlled values, explicit reconciliation, typed actions, stable IDs and separate update/draw/measure. Preserve observable tagged behavior while removing old render-time semantic mutation in the new architecture.

One runtime owns generic interaction. One shared text engine, collection/scroll engine, theme resolver and layering mechanism serve all components. Grid has row/cell modes instead of parallel table engines; menu/picker wrappers share actual engines; StatusBar absorbs segments and Panel+TextViewport absorbs ScrollPanel.

No public Junie compatibility aliases, alternate old/new feature path, copy of the whole old library, duplicated event types or placebo facade survives. Port proven behavior into the correct owner; names alone do not decide which existing implementation is correct.

Migrate coherent vertical slices with actual consumers as dependencies become ready:

1. Foundations/runtime/public API and externally compiled usage probes.
2. Controls, chrome, collections and scrolling in real pages.
3. Fields/forms/choices/secrets and real editing flows.
4. Menus/dialogs/pickers/completion and nested app overlays.
5. Grid/viewport/code/diff and progress/status/motion.
6. Prepared-cell TerminalView/session edge and remaining application consumers.
7. Remove bridges/legacy paths; close every public API, app and performance gate.

Each slice includes new owning code, source references, preserved semantic tests, tag-versus-candidate captures, an actual migrated app path and a targeted paint/action mutation demonstrating ownership. Passing an unchanged old app or an unused standalone new widget is insufficient.

Keep `showcase`, `jackin-preview`, `holla` and `tablepro` binaries and current CLI/scenario behavior. Split their implementation by the map, but do not introduce real services or change fixture/domain behavior. Preserve all paths, failures and recovery interactions. Product art/visible titles—including literal Junie wording present in the tag—remain exact; narrow data/provenance/license exemptions are not permission for Junie code namespaces.

A temporary bridge requires explicit owner, affected callers and removal gate. Remove all bridges before completion. All app presentation paths must use public Termrock components, not dead calls followed by custom repainting.

Move cross-app integration tests to actual crate dependencies. Build fresh real binary targets and obtain their paths/digests from Cargo metadata/build output; do not assume `CARGO_BIN_EXE_*` spans packages or run stale/reference binaries by mistake. Update path/CWD/asset assumptions without changing rendered output.

## 7. Artifact, semantic and anti-cheating specification — G05

Preserve the screen/substep hierarchy with standalone component coverage. Every visual checkpoint produces ANSI, HTML, PNG, ASCII, TXT and canonical frame JSON from the same observation, plus loss/fidelity/semantic metadata and complete manifests.

Use native Tuiscotti exporters/comparators, not a second rasterizer or parser. ASCII is lossy diagnostic output, never a Unicode/style equality oracle. Every required member is hash-validated; intentionally empty outputs are schema-validated rather than blindly rejected.

Compare complete cell structure, dimensions, continuations, supported colors/modifiers, cursor and stable semantic observations. Use exact decoded pixels against stored expected PNGs. Never regenerate expected pixels, trim content, hide regions, use a perceptual threshold or accept a short digest as the only parity proof. Report actual first differing cells/state, not empty diagnostics.

Validate the entire chain: required case set → execution checkpoints → bundle members → leaf manifests → corpus index → independent admission/CI receipt. Scope actuals by commit/run/attempt/platform/shard. Missing, stale, incomplete, cancelled, duplicated or required-skipped results fail.

Separate explicit capture/review/admission from normal verification. Tests/CI cannot approve the real store or repair missing expected files. Candidate changes cannot rewrite expected cases or comparator policy and then certify themselves. Freeze the tag-side contract under a reviewed reference; new discovered cases are additive tag-derived admissions, never candidate-generated expectations.

Run negative mutations through the production verifier: one glyph, color-only, modifier, wide continuation, cursor-only, one decoded pixel/channel, wrong/duplicate action, missing/corrupt every artifact, mixed generation, false hashes/profile/source, write failure, missing checkpoint/case/shard and attempted approval write. Synthetic comparator tests remain useful but are not substituted for real application evidence.

Audit all four app inventories directly from source: every Showcase page; Jackin host/Capsule/ritual/overlay world; Holla scenario/finder/plan/activity/resource/gate; TablePro connection/workbench/query/table/history/structure/safety. Preserve all original obligations and add identified gaps.

Use isolated synthetic HOME/config/runtime environments and never real credentials. Execute direct component tests, real in-process app tests, and Tuiscotti PTY tests. Verify live cursor/input/lifecycle rather than inferring them from old ANSI imports.

## 8. Whole-workspace CI and performance — G06

Regenerate workflows with verified Velnor Actions and supported config after workspace changes. Do not hand-edit generated YAML, spoof a release checksum or falsify branch/default-branch identity. Inspect the generated graph and actual execution for `termrock-implementation`.

Respect Velnor's `publish_refused:unprotected_ref` safeguard. Reusable CI-result publication is distinct from visual admission. Configure supported eligible publication behavior or make a minimal reviewed upstream generator fix; do not weaken protection, change branch protection without authorization, or hide required visual failures.

Use `cargo nextest` for all Rust tests, including real documentation-example checks in compatible targets. Do not introduce `cargo test` through generated doctest steps or wrappers. Explicitly select the intended ignored visual population; never select staging/admission writers through an indiscriminate ignored-test command.

Whole-workspace operations must explicitly select the workspace, not just default members or the removed junie package. Relevant native commands include:

```sh
mise exec -- cargo metadata --locked --format-version 1 --no-deps
mise exec -- cargo fmt --all --check
mise exec -- cargo clippy --workspace --all-targets --locked -- -D warnings
mise exec -- cargo check -p termrock --no-default-features --locked
mise exec -- cargo nextest run --workspace --locked --no-tests fail
```

Add actual supported feature/platform lanes, public API/compile-fail cases, the specifically selected visual/PTY suite, bundle integrity, mutations and concrete-result aggregation. Verify command flags against pinned tools; ordinary test success does not imply ignored cases ran.

Implement a small completion check in existing tooling or `termrock-xtask` that validates real evidence and returns nonzero for any open/stale mandatory item. This is an implementation requirement, not a claim that a command already exists. Do not build another orchestration platform.

Measure clean/warm builds and representative leaf-crate changes. Verify actual cache homes match configured Cargo/Mise paths. Use compatible prebuilt binaries and bounded PTYs. Initial acceptance executes captures rather than replaying cached pass receipts.

Include corpus checkout/download/materialization, rendering and upload in timing. Optimize toward the two-minute CI objective with measured sharding/cache/storage behavior, not fewer snapshots/formats, weakened comparisons or unreported costs. Run native Linux and macOS for the declared supported application/PTY contract and two independent complete fresh verification passes.

Required aggregation fails on any missing/failed/cancelled/required-skipped shard or artifact. Publish current-run diff/observation/results even on failure. Inspect all final-commit jobs and downloadable evidence; an earlier green commit or a passing Required job before a mandatory later failure is not final success.

## 9. Master gates and final review — G07

Maintain the full per-row checklist and these mandatory gate conditions:

- [ ] **G00:** source/main-structure audit, exact authorities, migration map, crate DAG and evidence ledger accepted.
- [ ] **G01:** independent tag-side captures, executable coverage and protected complete-artifact verifier qualified.
- [ ] **G02:** virtual workspace; every meaningful library/app/test/tool crate physically under crates; no source-inclusion shortcuts.
- [ ] **G03:** shared ownership and public APIs implemented and externally tested, including minimal/backend dependency lanes.
- [ ] **G04:** all four real apps migrated; every Junie implementation/import/alias/bridge and obsolete root code tree removed.
- [ ] **G05:** every required component/app/transition and extension verified; exact cells/pixels/semantics and negative gates pass.
- [ ] **G06:** generated whole-workspace CI, complete visual execution, proper cache/publication policy and measured performance verified.
- [ ] **G07:** fresh independent reviews, final clean-checkout runs and final pushed-commit acceptance pass; all 160+ mandatory rows have valid current evidence.

Do not mark an aggregate gate complete while one child item remains planned or blocked. Test functions called review/release are automated tests, not independent reviewer approval. Use fresh Gemini 3.8 Flash/high reviewers who did not author the audited areas. Review actual images/traces, graph/import evidence and failure probes.

Use frequent coherent commits with:

```text
Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>
```

Preserve `CLAUDE.md` symlinks, global Git settings and unrelated work. Keep one product branch. No unauthorized release, repository rename, main merge or destructive cleanup of other people's work.

Avoid a self-referential certificate: final external CI evidence binds the tested final commit and corpus/required-case digests. Later relevant changes invalidate it. Keep the working tree clean and push completed work before final verification.

The final report must list exact branch/head/oracle/main-comparison/tool identities; actual crates and dependency graph; legacy removals; per-app and per-component consumer coverage; expected/executed checkpoint sets; format/hash/semantic/pixel results; mutations; feature/native-platform/repeat runs; cold/warm timings; actual final CI links; reviewer findings/resolutions; and the verified checklist.

Do not stop after scaffolding, renaming, a new facade, a synthetic screenshot, or a default Nextest pass. Continue until every mandatory outcome is implemented and verified. If an essential external model/tool/permission/platform remains unavailable, identify the failing operation and leave the gate incomplete rather than inventing completion.

**Final definition:** only Termrock implementation remains; all Rust code is in meaningful crates, including multiple crates per application; all four actual applications consume it; visuals and existing interactions match visual-baseline; every required checklist item is verified. Nothing less is refactoring completion.
