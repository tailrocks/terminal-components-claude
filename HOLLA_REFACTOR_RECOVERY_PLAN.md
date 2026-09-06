# Holla-first refactoring recovery plan

**Repository:** `donbeave/terminal-components-claude`  
**Audit date:** 6 September 2026  
**Implementation base:** `holla`  
**Deliverable status:** Source-backed diagnosis and implementation plan; no repository changes or local Rust test execution were performed in this audit.

## 1. Decision and scope

**Continue from the working `holla` application implementations. Extract reusable components incrementally. Treat `main` as a source of candidate APIs, infrastructure, security fixes, and lessons—not as the product implementation to copy wholesale.**

Success is the conjunction of three things:

1. The applications retain the accepted appearance, interactions, timing, navigation, command-line behavior, and safety semantics of the pinned `holla` revision.
2. Their controls become reusable through a coherent, composable Rust API, with clear state ownership and application-independent components.
3. Independent evidence proves both properties on the exact commit that will be merged into `main`.

A successful API migration with a broken application is a failed refactoring. A faithful screenshot with incorrect hitboxes is also a failed refactoring. Conversely, keeping all implementation duplication forever is not completion either.

### 1.1 Frozen references

| Role | Commit | Meaning |
|---|---|---|
| Working product reference | `4a46bcae03ac14d2cf6c7d55657babf6b584996e` | `holla` HEAD examined in this audit |
| Refactoring reference | `c12cad8728755cd2d03eefdd8e02891143fca86d` | `main` HEAD examined in this audit |
| Shared ancestor | `cc14dd6beae526884aabdf897e309be837b4f504` | Merge base returned by the branch comparison |

The branches have diverged: the comparison reports 441 commits on the `main` side and 10 on the `holla` side. Do not confuse the common ancestor with the current working branch. The `holla`-only work must survive recovery. These counts are commit-graph observations, not counts of independent features. [S23]

The application inventory is **four binaries**, not three: `showcase`, `tablepro`, `jackin-preview`, and `holla`. The working manifest also declares `default-run = "showcase"`. The inspected main workspace contains the first three applications but not the `holla` application. [S01] [S02]

### 1.2 Evidence classification and limitations

This report distinguishes:

- **Observed remote result:** GitHub Actions execution results and the inspected compiler log.
- **Verified source difference:** behavior follows directly from inspected code on the frozen revisions.
- **Historical record:** a document or commit message records a past decision or failure; that is not automatically proof of the present implementation.
- **Hypothesis requiring execution:** a suspected regression that still needs a reproducer.
- **Proposed implementation:** a design, file, command, or test that does not necessarily exist yet.

No interactive session, build, test suite, screenshot rendering, benchmark, or local bisect was executed in this audit. The user's report identifies `holla` as the accepted experience; this audit does not independently certify every behavior on that branch. GitHub source and CI were inspected instead. No usable subagent execution facility was available, so this was not a multi-subagent execution; Section 13 provides the actual parallel work allocation for implementation.

The path history for both requested documents was queried. All five identified `REFACTORING_GOAL.md` changes were traced by subject and chronology, and the four subsequent change patches were inspected. For `COMPONENT_ARCHITECTURE.md`, the available history index and substantial decision history were inspected, together with current contract sections and selected patches. **Every historical architecture revision was not reconstructed and diff-reviewed end to end.** Task H-02 makes that remaining review an explicit pre-refactoring gate and specifies its output. This document must not be presented as an exhaustive, executed audit of every component or every historical revision.

### 1.3 Reading order

Read Section 2 for concrete failures; Section 3 for how the goals and architecture evolved; Sections 4–5 for corrected contracts; Sections 6–10 for implementation and acceptance tests; and Sections 11–14 for CI, merge safety, parallel ownership, and completion.

---

## 2. Findings: what is actually wrong

### F-01 — P0: the inspected `main` revision does not compile in CI

**Classification:** observed remote result, corroborated by source/module inspection.

GitHub Actions CI run `34002472849` failed for both `gates (1.88.0)` and `gates (stable)` at the step named “MSRV compile check.” In the inspected Rust 1.88 job, command `cargo check --workspace --all-targets --all-features` failed with:

```text
error[E0432]: unresolved import `super::overlay_chrome`
  --> crates/tui/src/components/dialog.rs:12:29
no `overlay_chrome` in `components`
```

The module list in `crates/tui/src/components/mod.rs` does not declare that module, and a fetch of `crates/tui/src/components/overlay_chrome.rs` at the frozen revision returned not found. All later test, render-matrix, boundary, and documentation steps were skipped in the inspected CI jobs. The separate Perf workflow succeeded; that is not a successful correctness build. [S10] [S11]

**Required action:** investigate the complete overlay extraction/import change before borrowing main's dialog implementation. Recover the intended helper and its tests, or remove the incomplete dependency by using a correct existing implementation. Merely adding a `mod` declaration to a nonexistent file is not a fix. Do not stub out drawing to obtain a green build.

**Acceptance:** clean-checkout compilation on both toolchains, then dialog keyboard, mouse, layering, clipping, focus-restoration, and visual parity tests. Record subsequent failures rather than treating the first compile repair as completion.

**Important qualification:** the current observed CI failure is this compile error. Missing parity evidence described below is a separate later blocker inferred from the source; it did not cause this observed early CI failure.

### F-02 — P0: compact navigation paints one layout but registers a different layout

**Classification:** verified source defect; the coordinate reproduction below is source-derived, not interactively executed.

Relevant code:

- Main `apps/showcase/src/app.rs`: `App::draw`, `paint_sidebar`, `paint_sidebar_row`.
- Main `crates/tui/src/components/nav_list.rs`: `NavList::draw`.
- Working `src/bin/showcase/app.rs`: `draw_sidebar`.

Main first calls `nav().draw(...)`, which paints the generic navigation list and registers row hitboxes. It then paints a second sidebar over that output. The generic list includes section headings and inter-section gaps; the second pass removes them when the sidebar is compact. Registration is not updated to match the replacement picture. [S03] [S04]

At a normal 80×24 terminal, the shell's sidebar starts at zero-based row 2 and has height 20. There are 22 navigation entries, so the application's compact threshold of 27 sidebar rows is active:

| Zero-based terminal row | Compact picture painted by the application | Generic navigation registration underneath |
|---|---|---|
| 2 | Overview | Foundations heading, not the Overview row |
| 3 | Buttons | Overview hitbox |
| 4 | Inputs | Inter-section gap |
| 5 | Text areas | Components heading |
| 6 | Forms | Buttons hitbox |

Consequently, a mouse click around zero-based `(4, 3)`, on the visible **Buttons** label, resolves to **Overview**. Hover and pressed styling are susceptible to the same mismatch. At 100×30, sidebar height is 26 and the compact mismatch still applies. Include terminal heights 30 and 31 in tests because the layout policy changes at this boundary. [S03] [S04]

The working implementation calculates, paints, and registers each row in the same loop; its hitboxes follow its actual compact policy. [S05]

**Required fix:** introduce a single navigation layout computation that supplies row/heading/gap rectangles to both painting and registration. Make section visibility and gap policy explicit component configuration. Use a supported row painter or part slot for the legacy appearance. Remove the second independent navigation painting/layout pass.

**Required tests:** actual coordinate clicks on every visible row, all section boundaries, compact/expanded thresholds, hover positions, blank gaps, clipping, resize between press and release, keyboard cursor versus current destination, and the last visible row. Use expected coordinates from the frozen product contract—not coordinates read back from the candidate's own potentially wrong hitmap.

### F-03 — P1: elapsed-time behavior became event-count behavior

**Classification:** verified source difference.

Working Buttons uses an `Instant` deadline 2,200 ms after starting the long job. Tick handling checks that deadline and reports completion through the application's status channel. Main uses `busy_frames = 28` and decrements it on every page update without checking elapsed time or limiting it to timer ticks. Unrelated input/update activity can therefore change the completion time. [S06] [S07]

Main also changes the feedback route: it updates the page-local `last` string, whereas the working implementation sends transient status messages. The completion branch does not itself set the response to `Changed`; whether a repaint occurs for that update must be verified against runtime scheduling. This is an additional test target, not an independently executed repaint failure. [S06] [S07]

**Required fix:** preserve the duration and status semantics while injecting a monotonic clock into update logic. Use a deadline or duration-based progress model; do not use arbitrary event counts as elapsed time. Request repaint when the visible state changes and schedule the next required wakeup.

**Acceptance:** at 2,199 ms the job is still busy; at the first update at or after 2,200 ms it completes once. Sending 1,000 mouse/key events without advancing the test clock cannot complete it. Advancing time while idle does complete it. Repainting cannot advance the job. Preserve activation count, toggle values, text, disabled behavior, and global feedback.

### F-04 — P1: shell diagnostics and contextual footer were reduced

**Classification:** verified source difference.

Main's recovered footer provides either navigation hints or generic page hints. The working footer includes context-sensitive page/edit/modal hints and transient status behavior. Main's inspector reports a small set of coarse values; the working inspector includes mode, focus/hover/pressed rectangles, hover suppression, mouse position, last key, focus-stop count, hit-region count, tick, and color capability. [S03] [S05]

This is more than a typography discrepancy. The footer teaches the available interactions; the inspector is a diagnostic tool needed to understand runtime failures. Replacing a status-capable shell with a static approximation loses useful behavior even when its overall silhouette looks similar.

**Required fix:** model shell state explicitly and restore the original contextual information. Expose read-only runtime diagnostics needed by the inspector without letting applications mutate internal routing state. Derive hints from the active binding/context model, with explicitly owned application hints where necessary. Preserve original ordering, wording, spacing, priority, and collision handling for the Junie fixture.

**Acceptance:** captures and interaction assertions for navigation, focused page control, editing, active modal, busy/status feedback, inspector open/closed, narrow width, and status/hints competing for space. No hint may advertise a nonfunctional action.

### F-05 — P1: Jackin's command-line contract changed

**Classification:** verified source difference, independent of screenshot interpretation.

| Contract | Working branch | Inspected main |
|---|---|---|
| No-argument scenario | `FirstUse` | `Returning` |
| `-s`, `-m`, `-f`, `-c` | Recognized aliases | Not handled by the shown parser |
| `-h`, `--help` | Print usage and exit successfully | Not handled; execution proceeds toward application startup |
| Invalid scenario/motion/frame/color value | Diagnostic and exit status 2 | Generally ignored or falls back silently |
| `JACKIN_NO_MOTION=0` or empty | Does not enable reduced motion | Presence alone enables it |

These differences are visible in the two entry-point implementations. The new `--theme` option can be retained as an additive capability, but not at the expense of existing startup and error behavior. The working branch's entry point also drains stale input before entering the application; inspect the new runtime wrapper before claiming that behavior is absent everywhere. [S08] [S09]

**Required fix:** preserve the old defaults, aliases, error semantics, environment interpretation, explicit-option precedence, and help behavior. A parser migration must be separate from UI changes and tested as a process contract.

**Acceptance:** process tests for no arguments, every alias, help without a terminal, missing/invalid values, frame parsing, each color alias, absent/empty/`0`/`1` environment values, and explicit motion overriding the environment. Assert stdout, stderr, exit code, and that help/errors do not enter raw mode or start an interactive session.

### F-06 — P0: the recovery scope on main predates the fourth application

**Classification:** verified manifest/document difference.

The main workspace, existing recovery goal, and the inspected parity recipe dispatch target three applications. The current working branch additionally contains `holla`, with eleven documented scenarios and its own verification matrix. A successful replay of the old three-app suite cannot certify preservation of that fourth application. [S01] [S02] [S14] [S15] [S19]

**Required fix:** make all four binaries and their CLI/scenario inventories first-class migration obligations. Preserve the current Holla implementation and its in-memory safety boundary. Do not replace its branch-only work with the historical common ancestor.

### F-07 — P0 verification: a historical parity checker is not the same as current parity evidence

**Classification:** source-backed verification gap; final clean-checkout behavior still requires execution.

The inspected `xtask/src/parity.rs` expects `parity/recipes.tsv`, `parity/evidence.tsv`, and `parity/visual_review.tsv`, validates historical-manifest mappings and source/revision provenance, and fixes the historical recipe count at 499. Its binary dispatch supports the three older applications. The tracked `parity/` tree at the frozen main revision contains only `recipes.tsv`. [S15] [S16]

The parity contract is registered among boundary checks in `xtask/src/main.rs`; CI invokes boundary checks but the inspected workflow does not first run the parity replay or retrieve its evidence. This needs a clean-checkout reproduction after compilation is repaired. Do not manufacture empty evidence or disable the check. [S17] [S24]

**Required fix:** separate the historical 499-case suite from the new pinned-Holla product contract; retain provenance for both. Generate required candidate evidence in CI or download immutable, hash-verified artifacts from an explicitly identified trusted build of the same source fingerprint. Validate exact recipe-key sets, not an arbitrary fixed count for all future coverage.

**Acceptance:** missing/stale/mismatched/dirty-source evidence fails with an actionable error. Every required recipe has a result. Candidate captures cannot be substituted for baseline captures. The fourth app is included. The merge candidate's evidence is generated or validated against that exact candidate.

### F-08 — P1 verification: default-frame goldens cannot prove interaction parity

**Classification:** verified limitation of the inspected test, not a claim that no other tests exist.

`apps/showcase/tests/visual.rs` renders the default state of every `PageId` across sizes, themes, and color levels and checks the new implementation's own baseline file. This is useful regression coverage after a baseline has been independently accepted, but it does not drive clicks, typing, modal transitions, drag, timing, or page-specific workflows, and it does not itself compare to the working branch. [S18]

Similarly, TablePro's `set_surface` constructs named capture states. That is a legitimate view-fixture mechanism, but it is not evidence that a user can reach the same state through the real input route. For example, grid-editing and pending-change fixtures share direct data mutation during setup; verify the actual editing flow separately rather than inferring it from a surface label. [S20]

**Required fix:** keep pure model-to-view fixtures, and add an independent suite that reaches important states by actual input and validates domain outcomes. Both are required; neither replaces the other.

### F-09 — P1 architectural: satisfying static API rules became separable from useful behavior

**Classification:** verified source pattern plus historical evidence; causal interpretation is an inference.

Examples in the inspected main code include constructing stateless shell props during update expressly to satisfy a props guard, and then using manual compatibility painting after components draw. Buttons also contains disabled API demo update calls separate from the actual drawn playground. These patterns are not all runtime bugs by themselves, but they show that a checker can reward syntactic compliance without establishing semantic equivalence. [S03] [S06]

The architecture history records missing named tests behind green checks, ineffective assertions, incomplete slot implementations, and impossible migration/package ordering. These admissions reinforce the need to test gate effectiveness rather than merely enumerate gates. Section 3 distinguishes those historical records from current verified defects.

**Required fix:** enforce consistent IDs, props, binding availability, hit geometry, state ownership, and effects through executable contracts. Do not require meaningless calls or duplicate property construction merely because a source scanner expects them.

### F-10 — Important unresolved hypotheses, not established defects

The following deserve targeted tests, but must not be described as proven failures based on this audit alone:

- Whether `NavListAction::EnterContent` actually transfers focus to content in every shell path. The inspected app handles it like `Chose` by changing the page; check runtime focus behavior and the working keyboard contract.
- Whether timer completion with an ignored response fails to repaint under the current scheduler.
- Whether all later parity/approval checks are reachable and fail closed after the compile blocker is fixed.
- Whether TablePro's dual visual/live connection tree state stays synchronized during every interaction.
- Whether each historical secret-handling and capture-hardening fix remains complete in the current implementation.
- Whether historical visual-change exceptions are still appropriately scoped after later grid and scrolling changes.

Create one issue and one executable reproducer for each. Resolve them as confirmed/fixed, not reproducible with evidence, or explicitly approved behavior change. Do not silently discard them.

---

## 3. What the document history says about the failure

### 3.1 `REFACTORING_GOAL.md`: all five identified path changes

Dates below use the UTC dates from the retrieved history.

| Commit | Date | Change and implication |
|---|---|---|
| `e48137f1852ea8f1b2885478a3f3312749443831` | 3 Sep 2026 | Introduced the repository refactoring goal. Good objectives included reusable APIs and preserving Junie behavior. The original creation patch was not independently reconstructed here; the resulting goal and subsequent changes were inspected. |
| `dce91d51c848cf07eaad4058296775e644940bd5` | 3 Sep 2026 | Removed stale `JACKIN_REFERENCE`, earlier goal, and feedback references from authority/read-first lists, shifting reliance toward current source and tests. This did not authorize dropping Jackin behavior; it made a complete living behavior inventory more important. |
| `2d81eec4358e88ef3a37c52466d612ba9e493238` | 3 Sep 2026 | Added model-specific analyst/builder/coordinator roles and execution constraints. This governed how work was done, not what product behavior could be removed. |
| `25ea92b08f41838896af58ab062834cab606ba9b` | 3 Sep 2026 | Relaxed research-tool/source prescriptions. It did not waive parity, safety, or evidence requirements. |
| `efada04419f72ecada75cfffb4b1bc6014a05ba2` | 5 Sep 2026 | Marked the old refactoring goal historical and replaced active `GOAL.md` with a parity-first recovery mission. This was an explicit recognition that architecture completion was not product completion. |

Sources: the frozen goal, the active recovery goal, and the linked commit patches. [S13] [S14] [G1] [G2] [G3] [G4] [G5]

**What was right:** preserving the design system, extracting generic controls, separating domain models from rendering, centralizing routine interaction mechanics, improving identity and state ownership, and requiring baseline evidence before deleting old behavior.

**What was unsafe:** the authority hierarchy put goal/design prose above rendered evidence and existing application behavior; the API contract became too easy to treat as an end in itself; operational model-routing rules occupied the product contract; and broad migration slices made application replacement and old-code deletion possible before end-to-end equivalence was demonstrated.

The original goal did contain important anti-regression requirements. Therefore the evidence does **not** support the simplistic explanation “the original goal never asked for parity.” A more accurate explanation is that parity requirements were insufficiently executable and insufficiently coupled to deletion and merge gates, while architecture conformance was more concretely rewarded.

### 3.2 `COMPONENT_ARCHITECTURE.md`: decision evolution and warnings

The current architecture document is roughly one megabyte and contains accumulated adjudications through Section 74. It still contains strong authority language and an exact-contract mindset. That accumulated history is valuable, but a new implementer needs a concise current contract plus a traceable archive—not several generations of rules interpreted by guessing which later section supersedes which earlier statement. [S12]

The following are particularly relevant historical records. These rows report what the commit records say; they are not blanket assertions that each old issue remains unfixed today.

| Historical change | Recorded problem | Rule to retain in recovery |
|---|---|---|
| `2c848f9e10d050c3099a826067aecca04d027157` | Hand-maintained conformance inventories could diverge from the component registry. | Derive required coverage from a single inventory and verify exact set equality. |
| `2399c1adeef55c6b69a0faa1875bdce888ec69d2` | Public-surface claims and actual module/export paths differed. | Compile external-consumer probes and inspect exported APIs instead of trusting prose counts. |
| `1b580d749ba761809bf6d52cf94139c67518e904` | Claimed example inventory included an absent example; a nonzero check was inadequate. | Verify every named example exists, compiles, and has an intended purpose. |
| `e6eaca4896755d23ec290deaf97508114bc06964` | Legacy test retirement/migration needed explicit accounting, including stronger secret assertions. | Map each old test's behavioral obligation to retained or stronger coverage before deletion. |
| `de817f2627b12739c658a08c0a2ce456e2da016a` | Declared component parts/slots and actual resolver use differed. | Give each advertised override a sentinel-style or sentinel-painter test. |
| `3dc44c48625c1cd03e67e685d41111d55919bfde` | Planned rename/deletion order could destroy still-unmigrated application builds. | Keep compiling intermediate states; packaging cleanup follows consumer migration. |
| `364689766c64985aced3b4bb1bdb79a5ee15be54` | Some style assertions were ineffective because compared patches touched disjoint slots. | Mutate the property under test and prove the assertion fails. |
| `b60c8280d1e117dfb2f095a59c1fb0c783850ebd` | Package naming and migration scheduling were inconsistent. | Use a temporary internal package identity; perform the final rename only after old consumers are gone. |
| `444ae3d9dbd449115c06e3695c326b76c2b43c1e` | Named tests were absent despite earlier green gates; an allow-list mechanism was not actually consumed. | Reject missing tests and unconsumed exemptions; test the checker. |
| `4d2621102a75343cb000bda483c2235a76ce2627` | Disabled visual fixtures and blessed digests did not establish the claimed disabled appearance. | Validate the fixture state independently and require visible/semantic evidence. |
| `0defae2283b4a2599d3a469e247ebed31283412d`, then `067869c7bebc633225fc0dd1c1c8e2d1228cbc90` | A capture-hardening claim was subsequently reverted as unsafe/incomplete. | Review the final implementation and adversarial tests, not a reassuring commit title. |
| `8514210cf8592bc9ed1a9f9693847077d08c904b` | Added classification for shared scrolling/grid geometry movement. | A changed digest needs a bounded, explained product decision; it is not automatically acceptable because a shared component moved. |

Commit links are indexed in Appendix A. The latest geometry-exception patch was inspected directly. The broader table is based primarily on retrieved history messages and contract records; reconstructing all parent-relative patches is still required by H-02.

### 3.3 Existing recovery on main must be recognized, not ignored

Main's active `GOAL.md` already calls for parity restoration. It describes the old three-application reference and a 499-capture historical suite, and identifies early application migrations and removal of the old root implementation as important divergence points. Current main also contains real shell restoration code and provenance-aware parity machinery. [S14] [S15] [S03]

Thus “main is just the original failed rewrite” would be inaccurate. It is a failed/refactored implementation **plus subsequent recovery work**, with some concrete restoration already present. The compact-sidebar defect demonstrates why even recovery can repair the picture while retaining broken interaction geometry.

The recovery goal names commits such as `7784719`, `4e07ea1`, `5042a40`, `444a8f4`, and `1378c31` as relevant migration/deletion points. Treat those as leads from the document until their actual diffs and behavior have been inspected. Do not attribute every regression to a single one of those commits without a targeted bisect or patch-level analysis. [S14]

### 3.4 Causal explanation

The strongest supported causal chain is:

**Broad replacement and rigid architecture acceptance → incomplete characterization and weak/uncoupled gates → old implementations removed before equivalent behavior was demonstrated → new self-baselines normalized changed output → visual compatibility painting layered over different runtime geometry → continued behavior drift despite recovery documentation.**

This is an inference from the source differences and history, not a claim about any contributor's intention. It explains why more documentation, more named tests, and more reusable-looking APIs did not themselves ensure a working experience.

The remedy is not abandoning reusability. It is changing the migration unit from “rewrite an application to fit the API” to **“extract one proven behavior into a reusable component, then replace one call site with independent equivalence evidence.”**

---

## 4. Replacement goal: normative text to adopt on `holla`

The following is proposed replacement content, not a statement that these files have already been changed.

### 4.1 Mission

Refactor the pinned working applications into reusable Rust components without unapproved changes to their accepted appearance, interaction behavior, timing, command-line contract, or safety boundary. The implementation starts from `holla`; `main` supplies candidate designs and verified improvements. All four existing binaries remain buildable and usable throughout the migration.

### 4.2 Authority and conflict resolution

Use this order:

1. Explicit current product decisions and safety requirements, including this recovery mission.
2. Approved behavior/visual/CLI contracts captured from the frozen `holla` reference.
3. Executable acceptance tests and independently reviewed reference artifacts that implement those contracts.
4. The current, consolidated component architecture.
5. Historical documents, old migration plans, and existing implementation details.

Tests can be incomplete or wrong; they do not outrank an explicit accepted product requirement. When evidence conflicts, record the smallest reproducible discrepancy and resolve it explicitly. Do not change the expected output just because the candidate already produces something else.

A security defect is not made mandatory by being present in the reference. Handle such differences through an approved, tightly scoped exception with negative tests. Never silently restore credential disclosure, real destructive side effects, or unsafe shell execution for visual fidelity.

### 4.3 Required outcomes

- Preserve all existing application entry points, scenarios, navigation routes, meaningful states, CLI defaults/aliases, and keyboard/mouse workflows.
- Preserve Junie's existing geometry, typography, glyphs, spacing, state styling, contextual hints, and feedback for the accepted baseline environment.
- Provide consistent borrowed props and explicit durable state where useful; keep business/domain data owned by applications.
- Provide stable identity for controls and collection items, correct focus/modal/capture behavior, and one source of truth for interactive geometry.
- Support composable content and documented customization of appropriate parts without requiring applications to repaint an incompatible component underneath.
- Expose generic list/tree/grid/form/overlay contracts, not TablePro SQL types or Holla/Jackin-specific domain state in the core library.
- Support pure model-to-view verification and real-input end-to-end verification.
- Preserve security improvements from main when they are verified and applicable.
- Prove reusability in external-consumer-style examples and in at least two genuinely different application contexts for shared components.
- Finish with one coherent implementation per migrated behavior and one public API—not permanent old/new public systems.

### 4.4 Explicit non-goals

This recovery is not a redesign, a feature reduction, a rewrite to match the exact shape of current main, a new retained-mode UI framework, a universal plugin system, or a change from deterministic previews into real infrastructure execution.

Do not introduce a second independent catalog/lookbook shell. Preserve the current Showcase while improving its reuse. Any later information-architecture change, such as reorganizing application previews into new navigation sections, is a separate product change with its own baseline approval.

Breaking Rust API changes are permitted where they improve the final API. Breaking user-visible behavior is not implicitly permitted by that permission. Temporary private migration adapters are permitted; permanent duplicate public APIs are not.

Performance targets must be measured. Do not sacrifice correctness for theoretical zero allocation, alter visible timing to improve a benchmark, or declare “fastest” without an appropriate benchmark design.

### 4.5 Deletion and completion rule

A legacy implementation may be deleted only after its behavior inventory, corresponding candidate tests, independent visual comparison, real-input flows, and public-consumer proof pass on the same candidate revision. The deletion itself must then pass the same gates.

An agent may report a task as `implemented` after code changes, but only an independent verifier may report it as `verified`. The merge controller reports completion only after testing the exact merged tree.

### 4.6 Document structure

Create a short authoritative `GOAL.md` and a consolidated `COMPONENT_ARCHITECTURE.md`. Keep historical documents in a clearly marked archive and retain their Git history. Make `REFACTORING_GOAL.md` a concise pointer to the current mission rather than a second competing normative document.

Create an obligation map from every surviving historical requirement to its current contract/test or explicit supersession. Remove model names and transient agent/tool availability from the product architecture; keep execution-role instructions in contributor/agent guidance.

---

## 5. Corrected component architecture

### 5.1 Ownership boundaries

| Concern | Owner | Must not happen |
|---|---|---|
| Application/domain data, simulated effects, command results | Application model/services | Core components importing SQL, Docker, provider, capsule, host, or account models |
| Durable UI state: cursor, selection, scroll, edit draft, expansion | Explicit component state held by application/composite owner | Hidden state that disappears when props are reconstructed |
| Ephemeral presentation/configuration | Borrowed props for the current update/draw | Retaining a borrow across frames or cloning entire domain models for routine rendering |
| Focus, hover, pointer capture, modal stack, committed hit geometry | Runtime | Each page independently implementing a competing global router |
| Layout and registered interactive parts | Component/composite layout result | App painting a different geometry over the registered component |
| Colors, glyph roles, component-state styles | Theme/design system, with bounded public overrides | Inferring semantic roles by comparing raw colors |
| Business side effects and validation decisions | Application/domain layer, invoked from update | Mutating domain state or executing effects during drawing |
| Test fixtures and expected artifacts | Test/verification layer | Shipping a separate fake renderer to satisfy snapshots |

### 5.2 Keep a two-phase model, but define the frame lifecycle precisely

The main proposal of explicit update and draw phases is useful. Its correctness depends on lifecycle rules, not just method names:

1. Establish initial application state and layout/registration before pointer routing.
2. Resolve input against the most recently committed, presented geometry and current modal/focus context.
3. Apply component and application updates; record actions and effects explicitly.
4. Reconcile changed identity/collection state and compute the next layout.
5. Draw and register parts from that same layout; commit the frame atomically.
6. Reconcile focus/capture for removed or disabled controls according to a documented policy, with a bounded redraw if necessary.
7. Process subsequent geometry-sensitive input against the new committed frame—not a stale pre-update batch.

Draw may populate a runtime-owned frame-registration builder, but must not mutate business data, commit an edit, advance timers, execute commands, or consume user input. Repeated drawing of the same view model, component state, and render context must be observationally equivalent.

Do not impose an elaborate retained tree unless a concrete requirement justifies it. Reuse the smallest runtime model that passes the existing interaction contracts.

### 5.3 One layout result for painting and interaction

Introduce private layout structures where necessary; public geometry customization should be intentional rather than accidental. A navigation layout, for example, should contain visible row keys/rectangles, heading rectangles, gap rectangles, clip region, and any scroll/compact policy.

Both the painter and hit registration consume those rectangles. A part slot receives its already-resolved rectangle and style/state context. It may replace painting within that region, but it does not secretly move the interactive target. A customization that changes geometry must participate in layout/measurement through a documented API.

Every registered target must be clipped to its visible interactive region. Clipped-away rows cannot remain clickable. Decorative headings and gaps cannot activate neighboring items. Overlay order must govern both painting and hit resolution.

### 5.4 Identity and reconciliation

Use stable control IDs and stable item keys. Index keys are acceptable only for fixed, immutable fixture order, not filtered/reordered domain collections. Preserve selection and editing by logical key across filtering, sorting, insertion, and deletion.

Test removed selected items, removed focused controls, duplicate keys, empty collections, all-disabled collections, and IDs constructed through different parent/child segment boundaries. Define collision and duplicate-registration diagnostics. Do not assume a custom hash is collision-free merely because it is deterministic.

### 5.5 Consistent responses without discarding component-specific semantics

Use a coherent response envelope for consumed/changed/repaint/action/effect information. Keep typed actions for meaningful component outcomes: activation, chosen item, edit intent, committed/cancelled draft, selection change, requested sort, and requested navigation.

Do not conflate cursor movement with value selection, navigation with entering the content focus scope, or rendering a busy state with scheduling its completion. Do not require every passive component to implement a meaningless update call.

Props relevant to interaction must agree between update and draw. Test this through the published interaction contract and, where useful, debug metadata—not through a blanket rule that the same constructor text must appear in both methods.

### 5.6 Timing and effects

Inject a clock into updates. Represent real behavior durations using deadlines or elapsed duration. Represent deterministic animation fixtures using an explicit animation phase/tick supplied to the view. Keep the distinction visible in types or APIs.

A repaint must not advance simulation time. Paused motion must not freeze unrelated input processing. Reduced motion must preserve application transitions while reducing or eliminating animation. Idle runtime scheduling must wake for due timers without a busy loop.

Simulated domain effects remain simulated. Production-like preview actions must not acquire real filesystem, Docker, SSH, provider, database, or credential access as a side effect of component extraction.

### 5.7 Theme and rendering contract

Extract the working Junie tokens and state behavior before generalizing them. Preserve resolved foreground/background colors, modifiers, glyph placement, gutter widths, disabled/selected/hover/pressed/focused combinations, borders, cursor behavior, and clipping.

Distinguish style replacement from style patching. A patch that omits a modifier is not necessarily a request to remove that modifier; tests must expose inheritance mistakes such as unintended bold text. Give each advertised part override a visibly unique sentinel test.

State-reference matrices may use an explicitly inert reference-state scope. Such a scope must not register focus, bind input, or fake application navigation. It is for demonstrating real component rendering, not a substitute implementation.

### 5.8 Generic collection and editing APIs

Lists and trees borrow application data through keys, labels, row painters, child relationships, and disabled-state accessors. Grids borrow columns/rows/cells through generic adapters. Application code owns SQL validation, dirty-row tracking, destructive-query classification, connection safety, and persistence.

Editing is a transaction with a draft, selection/caret, validation result, and explicit commit/cancel outcome. A view snapshot labelled “editing” does not prove this transaction exists. Ensure blur, Escape, Enter, Tab, mouse selection, and external model updates follow the accepted policy.

### 5.9 Public customization proof

For each component, prove: default use; common variant; custom data model; part-style override; content/row customization; disabled/empty state; interaction; and use from a crate that cannot access private internals.

Do not export internal runtime mutation merely to make an example compile. Conversely, do not forbid legitimate low-level application composition so aggressively that consumers must reconstruct the component's painted geometry. The API boundary should prevent duplicated mechanics, not all application-specific presentation.

### 5.10 Package layout: destination, not the first migration step

A reasonable destination is:

```text
crates/tui/                 reusable components and runtime
crates/tui-testing/         public deterministic test support
apps/showcase/              existing catalog and demos
apps/tablepro/              existing TablePro application/domain adapters
apps/jackin-preview/        existing deterministic Jackin preview
apps/holla/                 existing deterministic Holla preview
xtask/                     verification and development commands
```

This is a proposal, not a requirement to rearrange all files immediately. Keep the working package and binaries intact while extracting small pieces. If two implementations temporarily coexist, give the candidate a distinct internal package name. Delay the public package rename, facade-only enforcement, and old-root deletion until all consumers and tests have migrated.

Preserve ordinary launch commands. Test how `cargo run` resolves after any virtual-workspace change; do not accidentally turn a no-argument launch into an ambiguous-package error.

---

## 6. Ordered execution plan

All paths introduced in this section are proposed deliverables. Adapt their location to the final workspace, but keep task IDs and acceptance obligations stable. Do not mark a task complete from a document edit alone.

### Phase 0 — Freeze, reconstruct, and measure the starting point

#### H-01 — Establish isolated source and evidence workspaces

**Owner:** integrator. **Depends on:** nothing.

1. Inspect the existing checkout, branch, worktree list, staged changes, unstaged changes, untracked files, and current remote refs. Do not reset, stash, delete, or overwrite another worker's files automatically.
2. Record the frozen commits from Section 1, current remote heads, and their differences. If a branch has advanced, keep this audit's references and create an explicit delta-review task.
3. Maintain a detached, unmodified `holla` reference worktree; a detached `main` reference worktree; and a writable recovery worktree based on the pinned `holla` commit. The recovery branch is the candidate for updating `holla`, not a replacement base taken from main.
4. Use separate build output directories for reference and candidate. Record executable hashes so a stale binary cannot masquerade as a different source revision.
5. Put generated artifacts outside the read-only reference worktree. Record the compiler, lockfile, target triple, terminal parser, locale, theme/color mode, font identity for images, dimensions, and scenario arguments.
6. Create `docs/refactor/revisions.json` and an initial `docs/refactor/status.md` containing observed—not inferred—build/test status.

**Exit gate:** every artifact and process can be attributed to an exact source, build configuration, and binary; the known-good reference tree remains clean.

#### H-02 — Finish the exhaustive document-history audit

**Owner:** history/architecture analyst; independently checked by verifier. **Depends on:** H-01.

1. Export full reachable history on frozen main for each requested path, including merge commits. Follow renames separately for each path; do not assume a path-filtered first-parent log captures every change.
2. For each relevant commit, store its parents, date, subject, full message, document blob hash, document snapshot or deletion marker, and diff against each parent. For the creation commit, compare against the empty tree.
3. Distinguish requirements added, changed, weakened, removed, superseded, or merely moved. Mark pure formatting changes explicitly rather than ignoring their commits.
4. Review every baseline-change exception, public API rule, migration-order rule, test retirement, fixture substitution, secret-handling rule, and claim of completed verification.
5. Map requirements to the implementation commits that attempted them. Inspect deletion and application-rewrite commits identified by the existing recovery goal; do not rely solely on their subjects.
6. Produce `docs/refactor/history/index.tsv`, `goal-obligations.tsv`, `architecture-obligations.tsv`, and `supersession-map.md`.
7. Each obligation row must contain: stable ID; first introducing commit; later changing commits; original intent; current rule; implementation location; evidence/test; decision (`retain`, `correct`, `supersede`, `reject`, or `unresolved`); rationale; and reviewer.
8. Compare the export's commit set with the API history already inspected. Explain discrepancies caused by merges/renames rather than assuming one listing is complete.
9. Reconcile apparently contradictory sections into a single current contract. No architectural rewrite may begin while a relevant parity, ownership, safety, or deletion obligation remains ambiguous.

**Exit gate:** every reachable document-changing commit is indexed and reviewed; all retained obligations have a current home. An automated index is not sufficient evidence of semantic review.

**Do not:** erase the history, paraphrase only the latest document, treat an archived checkpoint as an accepted merge, or treat a later “verified” title as proof that the exact current tree was verified.

#### H-03 — Build and discover actual tests on both frozen revisions

**Owner:** verifier. **Depends on:** H-01.

1. Build all four working binaries with their recorded lockfile/toolchain. Run and retain the complete test list and complete results, including ignored tests, feature-gated tests, integration tests, and binary-local tests.
2. Record any working-branch failures separately from new regressions. A failing old test must be understood; do not silently declare the entire baseline green.
3. Reproduce main's compile failure when a local toolchain is available. For diagnostic execution of main only, use a separate, explicitly recorded minimal repair patch; never label that patched tree as the original frozen main.
4. After any diagnostic main repair, continue through all checks to expose downstream problems. Keep source fingerprints and evidence separate for original and patched main.
5. Test actual launch commands, help, malformed arguments, and terminal cleanup. Record current package/binary names using Cargo metadata rather than guessing from directory names.
6. Write `docs/refactor/baseline-health.md` with commands, exit statuses, log paths, environment, and exclusions.

**Exit gate:** known baseline health; no test-count claim based only on README wording; missing or undiscovered tests are visible.

#### H-04 — Create a complete product and dependency inventory

**Owner:** application analysts. **Depends on:** H-01; can run alongside H-02/H-03.

1. Enumerate the four binaries, all CLI flags/aliases/defaults/environment controls, startup routes, minimum sizes, application states, modal/picker states, and deterministic scenarios.
2. Enumerate all 22 Showcase page modules and each interactive control on each page. Include shell controls and diagnostics; do not inventory only the central content pane.
3. Enumerate every working TablePro tab/view, menu, editor state, query result, safety flow, and connection operation; compare these to main's 21 named surfaces.
4. Enumerate Jackin's eight documented scenarios and Holla's eleven scenarios, plus intermediate states reachable within each scenario.
5. Map each old component/module and app-local reusable control to its consumers. Include helpers that own interaction or geometry even if they are not named “widget.”
6. Map old tests to these behaviors. Record uncovered routes, not merely a total test count.
7. Classify main-only changes as candidate reuse, security fix, additive capability, incompatible implementation, or unexplained behavior change.

**Deliverables:** `product-contract.toml`, `component-map.tsv`, `test-obligations.tsv`, `main-salvage.tsv`.

**Exit gate:** exact coverage sets can be computed; every existing route/control has an owner and a preservation decision.

### Phase 1 — Build the independent acceptance system before refactoring

#### V-01 — Capture the working product as an immutable oracle

**Owner:** verification engineer. **Depends on:** H-03, H-04.

1. Reuse working capture/scenario tools where trustworthy. Audit their process invocation, scenario selection, capture timing, and exit/error handling before trusting their artifacts.
2. Preserve the historical 499-case suite in its original namespace. Capture a separate suite tied to current `holla` HEAD; do not overwrite the historical suite or assume it covers branch-only work.
3. Capture full-screen terminal cell data, including text, foreground/background, modifiers, cursor state, dimensions, and meaningful interaction metadata where available. Also retain raw terminal output or a replayable recording for end-to-end cases.
4. Render those cell artifacts to images using a pinned renderer/font environment for human visual review. Keep the cell artifact authoritative for exact terminal semantics; PNG appearance supplements it.
5. Preserve full shell areas: title/header, sidebar, separators, page content, footer, status, inspector, and overlays. A cropped page-only snapshot is not full application parity.
6. Use deterministic fixture clocks/randomness/environment. If reference instrumentation is necessary, maintain it as a separate, reviewed capture patch with its own source hash and prove it does not change the observed product contract.
7. Capture startup and the important intermediate states reached by input traces, not just the final screenshot.
8. Store the approved manifest with source revision, executable hash, recipe version, terminal environment, and hashes of all artifacts.

**Exit gate:** the baseline can be regenerated from its own recorded source/environment, and candidate rendering code is not involved in creating expected output.

#### V-02 — Introduce pure view fixtures

**Owner:** testing-library engineer. **Depends on:** V-01 and the state inventory.

1. Provide a way to construct component state and an application view model without executing business effects.
2. Render through the real production component/view function into an off-screen terminal buffer. Do not implement a separate snapshot renderer that approximates the application.
3. Dump stable cell artifacts to files and provide a renderer for review images.
4. Make focus/hover/pressed/disabled/reference-state input explicit and deterministic in inert fixture contexts.
5. Verify that drawing the same fixture twice produces identical output and does not change its domain state, edit draft, clock, or action count.
6. Capture combinations that matter: disabled+selected, focused+editing, hover+pressed, modal+background focus, empty/error/loading, and long/clipped text.

**Exit gate:** a view regression is caught without running business logic, and a business-state mutation during draw is caught independently.

#### V-03 — Introduce real-input application traces

**Owner:** runtime verifier. **Depends on:** V-01; may proceed alongside V-02.

1. Define scenario traces using actual key, paste, pointer move/down/up/wheel, resize, and time-advance operations.
2. Drive the same logical trace against reference and candidate, using separate adapters only where the source APIs differ. The adapters may translate transport; they must not manufacture domain outcomes or bypass routing.
3. Capture frames and observable outcomes after each meaningful step. Track focused logical control, selected logical item, active screen/modal, edit state, status, and simulated effects.
4. Keep a required coordinate-driven suite. Semantic “activate Buttons” helpers alone would miss F-02 because they bypass the wrong hitbox.
5. Add real PTY smoke/acceptance tests for startup, raw-mode cleanup, alternate screen, resizing, paste, mouse protocol, and representative complete flows.
6. Normalize internal IDs when comparing different implementations, using stable product-level names. Do not require old private hashes or allocation order to be identical.

**Exit gate:** a candidate that paints correctly but ignores clicks, routes to the wrong row, or changes time semantics fails.

#### V-04 — Test the verification system itself

**Owner:** independent verifier. **Depends on:** V-02, V-03.

Deliberately introduce isolated mutations in throwaway test worktrees: move a hitbox one row without moving its label; drop a click; change the button deadline; remove a scenario; remove a named test; alter one color/modifier; reuse stale candidate evidence; replace baseline with candidate output; and disable modal input isolation.

For each mutation, record the exact failing test/check. Restore the tree after each experiment. A mutation that passes requires a stronger gate before refactoring proceeds. The suite should also reject duplicate recipe keys, missing files, unexpected files where relevant, parser errors, and zero-test filters.

**Exit gate:** known representative failures are actually detected, not merely mentioned in documentation.

### Phase 2 — Consolidate contracts and prove a small extraction

#### A-01 — Replace competing goals with one current contract

**Owner:** architect and integrator. **Depends on:** H-02, H-04, V-01.

Adopt Section 4's mission and Section 5's architecture, adjusted only by evidence from the completed history review. Archive rather than erase the old documents. Link every retained requirement to a test and responsible implementation area. Explicitly name all four applications and forbid candidate self-blessing.

**Exit gate:** no surviving “architecture above product parity” rule, no contradictory deletion order, and no model-specific execution policy masquerading as a product requirement.

#### A-02 — Pilot a button extraction without replacing the Buttons page

**Owner:** component engineer. **Depends on:** V-02, V-03, A-01.

1. Retain the working page's layout, labels, model, status requests, timing, and activation semantics.
2. Extract only button rendering and routine activation handling behind the candidate reusable API.
3. Preserve public configuration for variant, disabled/busy state, icon/label, and part styling that the real page needs.
4. Replace one call site; compare all relevant default/hover/focus/press/disabled/busy fixtures and actual inputs.
5. Replace the remaining button call sites only after the first one passes.
6. Add an unrelated external consumer using the same component with custom content or styling.

**Exit gate:** no full-page rewrite, no changed snapshots, correct time/status behavior, and demonstrable reuse. This pilot must pass before copying a whole family from main.

#### A-03 — Pilot navigation extraction with shared geometry

**Owner:** runtime/component engineer. **Depends on:** A-02 and coordinate tests.

1. Extract the working sidebar's compact/expanded policy into the reusable navigation layout.
2. Preserve separate keyboard cursor and active destination state.
3. Preserve the accepted keyboard commands, click semantics, section headers/gaps, current marker, focus gutter, hover behavior, and clipping.
4. Expose row painting through the component's resolved rectangle; eliminate application-side geometry reconstruction.
5. Test 80×24, 100×30, heights around the compact threshold, and widths around the sidebar-width threshold.
6. Confirm content-entry actions transfer focus according to the working contract.

**Exit gate:** F-02's coordinate trace passes; all visible rows and blank gaps behave correctly; there is exactly one geometry source for painting and registration.

### Phase 3 — Harden shared foundations incrementally

#### R-01 — Identity and collection state

**Owner:** runtime engineer. **Depends on:** successful pilots.

Implement or selectively reuse stable IDs/keys, reconciliation, duplicate diagnostics, and disabled/empty collection behavior. Migrate one collection consumer at a time. Include reorder/filter/remove/add tests and selection/edit preservation. Keep item-key policy visible at the application adapter boundary.

**Exit gate:** no focus or selection jumps caused by incidental indices; logical state survives valid model changes.

#### R-02 — Input routing, focus, modal stack, and capture

**Owner:** runtime engineer. **Depends on:** R-01.

Define key-phase precedence, editing interception, global actions, nested modal handling, pointer capture, release-outside behavior, wheel ownership, and hover suppression. Preserve the working semantics in each migrated consumer. Add frame-generation tracking or equivalent protection against stale geometry where required.

**Exit gate:** one event has the intended consumer/effect; hidden/disabled/background controls cannot receive forbidden input; resize and removal invalidate stale targets safely.

#### R-03 — Theme, parts, and text geometry

**Owner:** rendering engineer. **Depends on:** pilots; can parallel R-01 with disjoint ownership.

Extract working theme tokens and glyph behavior, then provide semantic variants/parts. Use sentinel tests for each override. Audit width/truncation/wrapping/ellipsis, combining characters, double-width characters, and clipping at one-cell/zero-cell bounds. Preserve working colors rather than “approximating” them with new semantic tokens.

**Exit gate:** independent cell parity for existing Junie states, no stray modifiers, no out-of-bounds paints, and documented alternate-theme behavior.

#### R-04 — Clock, repaint, diagnostics, and terminal lifecycle

**Owner:** runtime engineer. **Depends on:** R-02.

Add injected clock/update cause and due-wakeup scheduling without changing accepted durations. Restore read-only inspector diagnostics and shell status support. Verify panic/error/quit cleanup and idle behavior. Do not couple reduced motion to disabling real actions or timers.

**Exit gate:** timing is input-rate independent; cleanup is reliable; diagnostics correspond to actual routing state.

### Phase 4 — Migrate component families, preserving callers

**Owner:** component engineers. **Depends on:** relevant Phase 3 foundations.

For every row in Section 7, use this exact sub-sequence:

1. Identify the working implementation and all callers.
2. Add missing characterization fixtures and traces before editing it.
3. Design the smallest API needed by those callers plus a genuinely different external consumer.
4. Compare candidate main code against that contract; copy only the useful parts with their tests and dependencies.
5. Preserve or mechanically relocate working rendering first; generalize after parity is established.
6. Migrate one call site and run its targeted suite.
7. Run every other consumer affected by shared runtime/theme/layout changes.
8. Validate external-consumer compilation and customization.
9. Remove now-unused local mechanics only after equivalence is proven.
10. Run the full affected-family suite after deletion, and record evidence hashes.

Do not replace multiple families, their callers, package layout, and golden files in a single change. A family may contain several small commits; every integration point must compile and preserve the experience.

### Phase 5 — Complete application migrations without application rewrites

#### APP-01 — Showcase shell and all 22 pages

**Owner:** Showcase engineer. **Depends on:** migrated families used by each page.

Preserve the shell and existing page states. Replace event/render call sites incrementally. Restore contextual hints, status, inspector, modal behavior, and timer semantics as first-class contracts. Remove guard-appeasing dummy calls. Keep inert visual reference states separate from live controls. Follow the page matrix in Section 8.1.

**Exit gate:** every navigation route is reachable by keyboard and mouse; every page's existing behavior is accounted for; full-screen parity and timed/interactive traces pass.

#### APP-02 — TablePro

**Owner:** TablePro engineer. **Depends on:** forms, tree, tabs, grid/editor, overlays, split/scroll.

Keep working domain models, query simulation, safety policy, result editing, and connection semantics. Introduce generic adapters rather than relocating domain types into the core. Preserve old workflows first; use main's named surfaces to identify additional fixture obligations, not to replace reachable state transitions. Consolidate duplicated visual/live state only when a test proves equivalence.

**Exit gate:** the workflows in Section 8.2 pass through actual input, including dirty edits and safety denial/cancel paths. Rendering cannot commit a value or execute a query.

#### APP-03 — Jackin Preview

**Owner:** Jackin engineer. **Depends on:** forms, collections, overlays, terminal/log viewport, status/animation.

First lock down the CLI contract in F-05. Preserve first-use, intro/cockpit/outro, account/provider/capsule/settings workflows, failure recovery, and motion policy. Keep all external systems simulated. Confirm that secret values remain absent from rendering/logging/diagnostics as required. Inspect and preserve startup input draining at the actual runtime boundary.

**Exit gate:** every scenario and important intermediate state passes both view and real-input tests; help/errors never start a TUI.

#### APP-04 — Holla

**Owner:** Holla engineer. **Depends on:** shared families required by Holla; inventory/testing starts in Phase 0.

Retain all branch-only implementation and tests. Migrate controls within the existing application rather than replacing it with a generic picker demo. Preserve command ranking, scope, risk preview/review, typed host acknowledgement, activity lifecycle, hidden-command recovery, and in-memory execution boundaries. Use Section 8.4 as a minimum checklist, then fill every additional route discovered in H-04.

**Exit gate:** all eleven scenarios and the branch's own documented verification matrix remain valid, extended with real-input tests and missing color/motion coverage where necessary.

**Scheduling note:** these application tasks can proceed in parallel only after their shared APIs stabilize. Their relative completion order is not a reason to delay characterization or safety fixes in another application.

### Phase 6 — Verify reusability, then restructure packages

#### PKG-01 — Public-consumer and API usability review

**Owner:** independent API reviewer. **Depends on:** migrated components and applications.

Build external examples that use only documented exports. Review boilerplate, borrow ergonomics, state ownership, trait complexity, customization, and domain leakage. Confirm application-specific styling does not require private internals or duplicate hit registration. Remove advertised but nonfunctional builders/parts or implement them properly.

**Exit gate:** reusable APIs work outside this repository's privileged call sites. Examples are all discovered and executed/compiled as appropriate.

#### PKG-02 — Move packages and perform the final rename

**Owner:** integrator. **Depends on:** all application migrations and PKG-01.

1. Move one application/package at a time without changing behavior.
2. Update Cargo metadata, lockfile, feature relationships, launch commands, test discovery, examples, docs, capture commands, workflow paths, and artifact lookup.
3. Ensure binary-local tests were not lost when code moved to library/application crates.
4. Resolve the temporary candidate package identity only after the old root package is no longer needed.
5. Preserve `cargo run` behavior or provide an explicitly approved equivalent; test all four `--bin` launches.
6. Run default-feature, no-default-feature core, and all-feature checks without accidental backend feature unification hiding a dependency error.

**Exit gate:** clean workspace build, preserved developer/product CLI, correct feature boundaries, and no missing test targets.

#### PKG-03 — Delete migration adapters and obsolete code

**Owner:** integrator, approved by verifier. **Depends on:** PKG-02 and full parity.

For each deletion, attach the behavior/test mapping and surviving implementation. Delete only migration-specific duplication; preserve the frozen reference artifact/archive required for independent verification. Run the suite after deletion, not just before it.

**Exit gate:** one final implementation and public API, no app-specific control copies, no fake registration wrappers, and no permanent alternate renderer used only for tests.

### Phase 7 — Final verification and merge

#### FINAL-01 — Full acceptance and independent review

**Owner:** independent verifier. **Depends on:** all prior phases.

Run the complete product, component, CLI, safety, public-consumer, package, and mutation-gate suites. Review full-screen reference/candidate/difference images for all changed families and affected application scenarios. Inspect exact cell differences, not only PNG similarity. Confirm no missing evidence and no unapproved baseline changes.

**Exit gate:** every requirement is `verified` or has an explicit product-approved exception. Unknown is not pass.

#### FINAL-02 — Rehearse and verify the actual merge

**Owner:** integrator. **Depends on:** FINAL-01.

Follow Section 12. Create a merge candidate against the current main head, resolve differences by the accepted product contract, and rerun the same complete suite against the resulting merge tree. Preserve applicable main-only security fixes and all Holla-only product work. Do not use an `ours` merge to hide discarded changes.

**Exit gate:** the exact merge result—not just the pre-merge Holla branch—passes every required gate.

---

## 7. Component-by-component migration checklist

The old path families below come from the working component architecture and caller inventory; H-04 must confirm the exact full set. Names in the target column are conceptual API families, not a requirement to copy main's precise type names. Include app-local controls such as sidebar navigation even where no old library widget existed.

| Working area / family | Reusable destination and required behavior | Minimum specific proof |
|---|---|---|
| `widgets/button`, choice controls | Button, checkbox, toggle, radio/segmented choice with explicit state/value ownership | Activate once; disabled never activates; busy deadline; cursor/value separation; press-release-outside |
| `widgets/input`, `field_common`, `editable` callers | Text input/edit transaction and field composition | Enter/escape/blur policies; paste; caret/selection; Unicode editing; validation not triggered by draw |
| `widgets/textarea`, `code`, editor callers | Text area/editor with generic highlighting/diagnostics | Multiline navigation; scroll-to-caret; selection; insert/delete/newline/tab; diagnostic styling; no SQL types in core |
| `widgets/completion`, `select` | Completion/select overlay using borrowed items and stable keys | Query changes; no results; selection; commit/cancel; outside click; focus restoration; popup clipping |
| `widgets/list`, app-local sidebar | Generic list/NavList with shared row layout | Compact headings; exact row hits; cursor versus current; disabled skipping; filtering/reorder; long labels |
| `widgets/tree` | Tree with application-provided hierarchy/keys | Expand/collapse; ancestor fallback; removed children; focus; horizontal clipping; scroll and pointer alignment |
| `widgets/grid`, `table` | Generic table/grid model and editor adapter | Cell/row selection; header sorting; scroll; inline draft commit/cancel; ragged/empty data; stable cell identity |
| `widgets/tabs`, `segments` | Tabs/choice strip with stable identities | Overflow; active versus keyboard focus; close/reorder where supported; disabled items; wheel/click target |
| `widgets/picker`, `menu` | Picker, command palette, menu, nested menu composition | Search; empty match; cancel; chain/back; secondary click; submenu clipping; modal isolation |
| `widgets/dialog` | Reusable dialog shell with typed outcomes and optional form content | Initial focus; typed acknowledgement; nested overlays; default action; escape/outside policy; no click-through |
| Forms page and app forms | Form composition over field adapters, explicit draft/validation | Dynamic field removal; error rendering; submit/cancel; masked values; server/domain error mapping |
| `widgets/panel`, `empty` | Panel/empty-state presentation with bounded content slots | Border/title/meta geometry; small rectangles; custom gutter/title; empty action reachability |
| `widgets/splitter`, `scrollbar`, `viewport` | Split/scroll regions and text viewport | Drag capture; constraints; wheel ownership; thumb/track geometry; no stale hit targets; long content |
| `widgets/progress`, `steps` | Progress/spinner/stepper with explicit time/value input | Paused/reduced/full motion; monotonic timing; disabled state; exact width/clipping; no busy loop |
| `widgets/statusbar`, `hintbar`, `keyhint` | Status and contextual command presentation | Binding/hint agreement; priority/truncation; live status; edit/modal mode; footer collisions |
| `widgets/chips`, `props` | Generic chips/property list with controlled values | Selection/removal where supported; keyed state; long values; secret masking; real custom slots |
| `widgets/diff` | Generic diff source/presentation | Added/removed/context styling; line-number widths; long lines; scroll; alternate modes if present |
| `widgets/brand`, shell chrome | Brand/title/gutter/separator composition | Exact glyphs/modifiers; capability labels; header action hits; narrow width; no accidental bold inheritance |
| App-local activity/capsule/connection cards | Compose generic primitives; extract only shared mechanics | Different domain models use the same primitives without domain imports or forced visual homogenization |

### 7.1 Every component needs an explicit contract record

For each exported component, record its identity scheme; borrowed props; durable state owner; value owner; actions/effects; focusability; disabled behavior; keybindings; pointer/wheel/capture behavior; measurement; clipping; parts/slots; theme resolution; test fixtures; and real application consumers.

Passive components may legitimately have no update phase. Stateful controls must not hide durable state in an ephemeral props value. Custom renderers must receive enough context to implement the documented customization without duplicating internal routing.

### 7.2 Reuse must not erase distinct application behavior

A generic `Grid` can serve a TablePro data table and a component demonstration without dictating SQL behavior. A generic `Picker` can serve Holla ranking and a simple selection demo without owning ranking policy. A shared overlay can preserve different cancellation rules through explicit configuration rather than silently imposing one rule everywhere.

Do not generalize a component by deleting a feature that one consumer needs. Conversely, do not put every consumer's business rules into the generic component. Make the boundary explicit and test it with dissimilar consumers.

---

## 8. Application acceptance matrices

These are minimum required suites, not permission to omit additional behaviors found in H-04. For every row, add both a pure view fixture and an input trace wherever the state is interactive. Existing keys, copy, dimensions, and cancellation policies must be read from the working implementation rather than invented while writing tests.

### 8.1 Showcase: all 22 page modules plus the shell

The working page registry contains the following 22 modules. Its `Page` contract explicitly includes contextual hints, editing state, animation needs, status requests, and dialog requests; these are part of the behavior being migrated, not optional presentation details. [S25]

| Page module | Required view states | Required interaction/behavior checks |
|---|---|---|
| `overview` | Entire default composition, narrow layout, displayed component states | Any live overview controls remain reachable; decorative references remain inert |
| `buttons` | Variants, disabled, focused, hovered, pressed, busy, completion/status | Every live button once; toggles; disabled suppression; 2,200-ms deadline; unrelated input cannot accelerate it |
| `inputs` | Empty/value/placeholder, focus, selection, editing, invalid/disabled, clipped text | Begin editing; insert/delete; paste; caret motion; commit/cancel/blur; exact hints; no accidental global shortcut while typing |
| `textareas` | Multiline, wrapped/clipped, caret at edges, selection, scroll | Newline, text navigation, paste, edit transaction, viewport following, Escape/Tab policy |
| `forms` | Initial draft, field errors, grouped fields, disabled, submitted/cancelled | Focus order; edit each field kind; validation; submit/cancel; dynamic fields if supported; no submission from draw |
| `lists` | Cursor/current/selected, empty, disabled item, scrolled, long labels | Keyboard/wheel/click agreement; stable selection; filtering if supported; no hit target on clipped rows |
| `pickers` | Open/closed, query/no-match, selected result, nested or chained choices where supported | Open; type; navigate; choose; cancel/back; outside click; restore previous focus |
| `tables` | Header/body, selected rows, empty/ragged/long data, horizontal/vertical overflow | Column/header actions; row selection; wheel/keyboard; correct row after scroll; no division/underflow edge failure |
| `grid` | Focused cell, editor, pending changes, validation failure, clipped columns | Enter/edit/commit/cancel; cell traversal; scroll-to-cell; model mutation only after accepted commit |
| `trees` | Expanded/collapsed, nested, current/selected, empty, long branch labels | Expand/collapse; parent/child navigation; mouse; removal reconciliation; visibility/hitbox agreement |
| `dialogs` | Information/confirmation/prompt variants present in the working page, typed text, nested state | Open; initial focus; confirm/cancel; Escape/outside policy; closed result/value; background isolation |
| `chips` | All chip variants, selected/disabled/removable states present in source | Selection/removal/toggle behavior; stable keys; keyboard and pointer activation agreement |
| `progress` | Initial, intermediate, complete, busy/spinner, paused/reduced motion | Clock-driven advance; no advance on draw; completion once; no unnecessary continuous wakeups |
| `panels` | All panel kinds, title/meta/gutter/body, empty/narrow rectangles | Any live actions retain geometry; custom part painting stays within resolved rectangles |
| `scrolling` | Top/middle/bottom, empty/short/long content, thumb/track extremes | Wheel; page/line keys; thumb drag; resize; nested scroll ownership; no stale captures |
| `sidebars` | Sectioned/compact, icons/badges, selected/current/focus, long labels | Exact row clicks; section gaps inert; compact threshold; content focus entry; scroll if supported |
| `chrome` | Menu/header/footer/status/key-hint combinations, inspector/chrome examples | Header/menu actions; hint consistency; no active hit targets behind overlays |
| `editable` | Display/edit/draft/invalid/committed/cancelled | Explicit transaction boundaries; blur policy; changed value and cancel restoration; status feedback |
| `editor` | Editor text, caret/selection, diagnostics/completion where present | Working editing keys; selection; paste; completion acceptance; viewport; no text loss |
| `settings` | Initial setting values, active/disabled controls, changed values | Every setting control; apply/cancel/reset where implemented; nested focus; retain intended values across navigation |
| `taskrunner` | Initial, queued/running, progress/log, completed/failed states present in source | Start; advance simulated work; inspect results; cancel/retry where supported; status and clock behavior |
| `terminal` | Initial output, styled output, long lines, scrollback, selection if supported | Input routing; scroll and follow-tail; copy/selection semantics if present; no real process execution introduced |

**Shell suite, separate from page tests:** all 22 destinations by keyboard and actual coordinate click; header Help/Inspector clicks; sidebar focus/content focus; page state after leaving and returning; `q`/other global actions versus text entry; modal focus/close; inspector diagnostics; status expiry; no-size/too-small handling; compact-to-expanded resize; width transition; and no stale press/capture after navigation.

Required source-derived regression cases:

- `showcase.sidebar.buttons_click_80x24`: click the visible Buttons row using the reference coordinate; assert active page Buttons, not Overview.
- `showcase.sidebar.compact_100x30`: repeat at 100×30, including section-boundary rows.
- `showcase.sidebar.height_30_to_31`: resize across the compact policy boundary and repeat hit tests.
- `showcase.buttons.deadline_not_event_count`: hold the clock fixed while processing many unrelated updates.
- `showcase.footer.edit_modal_status`: verify hints and status in editing, modal, navigation, and normal page states.
- `showcase.inspector.matches_runtime`: compare displayed diagnostic values with read-only runtime observations.

### 8.2 TablePro: preserve actual workflows, not just surface names

The inspected main exposes 21 capture surfaces. Use them as a cross-check against the working application's complete state inventory; neither list alone proves all workflows are covered. [S20]

| Surface/group | Required transition and assertion |
|---|---|
| `connections` | Start in the accepted screen; navigate groups/connections; selected details and actual connection target agree |
| `connections-failed` | Attempt the fixture's failing connection through input; show the correct error without connecting to another item or losing recoverability |
| `workbench-default` | Connect successfully; assert intended database/schema, active tab, initial query/content, and focus |
| `explorer-focused` | Enter explorer focus; expand/collapse hierarchy; choose the actual keyed object and open it |
| `table-grid` | Open the intended table from explorer; assert columns, row values, selected cell, and scroll position |
| `grid-cell-editing` | Begin a real edit; assert draft/caret/editor and unchanged committed model before commit |
| `pending-change-bar` | Commit a valid draft into pending changes; assert dirty indicators and pending actions; cancel/revert paths do not accidentally persist |
| `structure-view` | Toggle the table structure view through the accepted binding/action; preserve object identity and return behavior |
| `query-editing` | Enter query edit mode; type/paste/change SQL; keep edit keys from triggering unrelated app shortcuts |
| `completion-popup` | Trigger completion from the real editor position; choose/cancel; assert inserted text and restored editor focus |
| `results-grid` | Run a successful fixture query; verify result data, row count/status, column geometry, and navigation |
| `error-result` | Run an invalid fixture query; verify error result/status and ability to edit/retry |
| `explain-plan` | Request explain; verify actual explain model/content rather than changing only a surface enum |
| `history-tab` | Run queries, open history, select/reopen as supported; verify entries and active tab |
| `quick-switcher` | Open/filter/select/cancel; verify chosen object/tab and focus restoration |
| `tab-list-picker` | Create enough real tabs for overflow; choose/close/switch with stable identity and correct active content |
| `safe-mode-picker` | Change the policy through UI; verify displayed and executed safety policy are the same |
| `filter-editor` | Apply/clear/cancel a filter; verify data and selected-row identity, not just filter text |
| `safety-dialog-typed-ack` | Run a destructive fixture query; require correct acknowledgement; wrong/empty/cancelled acknowledgement cannot execute |
| `help-dialog` | Open/close by existing routes; visible keys agree with behavior; background controls do not respond |
| `maximised-tab` | Toggle maximization; preserve active tab/data/focus; restore split layout and interaction geometry |

Additional obligations: connection form create/edit/cancel; splitter drag; tab close with dirty state; pending-change apply/revert; safe-mode behavior across connection/tab changes; invalid/ragged grid data; long identifiers and values; and context-menu actions present in the working implementation.

For every destructive or mutating operation, assert both the visible state and the underlying simulated domain effect count. A rejected operation must have zero effect even if the dialog looks correct. Use a generic grid adapter so SQL and pending-change policy remain in the application.

### 8.3 Jackin Preview: eight scenarios plus shared transitions

The working CLI names these scenarios: `first-use`, `returning`, `accounts-mixed`, `launch-running`, `launch-failure`, `capsule-multi`, `outro-last`, and `hard-cases`. It explicitly states that Docker, daemons, PTYs, providers, and 1Password are deterministic simulations. [S08]

| Scenario | Minimum acceptance obligation |
|---|---|
| `first-use` | Preserve no-argument startup, first-use content and intro progression, initial focus, and navigation into usable controls |
| `returning` | Preserve returning-user fixture state, selected environment/provider/account as seeded, and launch/navigation actions |
| `accounts-mixed` | Inventory every seeded account/usage state; preserve availability/status distinctions, selection, account details, and return route |
| `launch-running` | Preserve progress/log/status transitions, interactive navigation during work, and accepted motion behavior |
| `launch-failure` | Reach and display the actual simulated failure; preserve recovery/retry/back behavior and diagnostic information |
| `capsule-multi` | Preserve multiple capsule identities, switching, active-versus-focused distinction, nested overlays, and persistence across routes |
| `outro-last` | Preserve exit/outro conditions, final capsule/session semantics, animation timing, and terminal cleanup |
| `hard-cases` | Enumerate every seeded edge case from `scenario`/`sim`; preserve long labels, unavailable resources, empty/error states, and narrow layout behavior |

Shared required traces: launch; account/usage navigation; settings; help; nested picker/dialog; terminal/log viewport; keyboard focus order; pointer targets; failure exit/recovery; and every route exposed by the working application. Capture intro/cockpit/outro at explicit fixture phases under full/reduced/paused motion where applicable.

Do not infer detailed expected account or quota values from scenario names. Read the fixture model and freeze those expected values before writing tests. Preserve the old CLI table in F-05, including a successful noninteractive `--help` path.

### 8.4 Holla: eleven scenarios and branch-only behavior

The working Holla README describes eleven scenarios and a 132-image matrix from eleven scenarios × four sizes × three color modes. That is a documentation claim about the provided verification workflow; H-03 must inspect/run the actual script and confirm output completeness. Preserve that matrix and add missing coverage instead of assuming it is a complete interaction suite. [S19]

| Scenario | Required preserved behavior and negative cases |
|---|---|
| `first-use` | Empty/new-user guidance, always-armed query, Suggested/Recent/Explore organization, useful first action, help/menu access |
| `rust-dirty` | Correct project/repository context; dirty/diverged state affects offered actions and review rather than disappearing behind a generic list |
| `monorepo-root` | Workspace-level scope and project choices; correct breadcrumb/context; actions apply to the intended root |
| `monorepo-child` | Child-project scope versus workspace scope; preserve the documented differing repository outcomes such as diverged versus fast-forwardable targets |
| `docker-cleanup` | Preview/review of cleanup candidates; exclusions and dependency consequences; no actual Docker execution |
| `disk-cleanup` | Correct cleanup eligibility and blocked reasons; the documented active “Today” item remains protected; no actual file deletion |
| `upgrade-plan` | Dependency-aware plan review; excluding an item updates dependent decisions correctly; risk classification is not silently weakened |
| `activities-multi` | Multiple activity identities/statuses, navigation while work runs, selection by documented shortcuts, merged log identity, persistence across screens |
| `remote-host` | Host-bound context, remote review/typed acknowledgement, visible host identity, cancel/quit confirmation behavior, no real SSH connection |
| `launch-failure` | Visible simulated failure, correct activity state/logs, recovery path, no false success or loss of context |
| `hard-cases` | Detached HEAD, detection failures, missing SSH identity, long breadcrumb and other seeded hard cases remain intelligible and safe |

Shared contracts from the working walkthrough include scope progression, preview/review for different risk levels, pin/alias/hide/reset behavior, exact-command recovery of hidden commands, and persistent activity states. Preserve the documented keyboard routes (`Ctrl+P`, `Ctrl+O`, `Ctrl+S`, `Ctrl+A`, digit navigation, help/menu and quit routes) according to the actual active context. [S19]

Required negative tests:

- Read-only, bounded, and broad actions follow their respective confirmation/review rules.
- A wrong typed phrase or wrong host cannot authorize the reviewed remote action.
- Excluding a prerequisite updates dependent plan decisions; stale approved state cannot leak into the changed plan.
- Hiding a command does not destroy its metadata or make reset impossible.
- Scope changes do not run a command against the previous or wrong project/host.
- Activity navigation does not reset or silently complete running work.
- Masking and diagnostics do not reveal credential values or private key content.
- Every simulated action has zero real external-system effects.
- Full/reduced/paused motion preserve interactive correctness and selected logical state.

### 8.5 Cross-application terminal matrix

Reuse every existing approved matrix dimension. Add targeted dimensions that expose boundary behavior rather than multiplying all possible combinations indiscriminately.

Required axes:

- Existing canonical sizes, plus 80×24 and 100×30 for the confirmed compact-sidebar defect.
- Each application's minimum width/height minus one, exactly minimum, and plus one; obtain its minimum from the working source.
- Zero/one-cell component rectangles for library robustness; these are component tests, not a requirement that complete applications be usable at 1×1.
- Showcase height 30/31 and widths around its 110-column sidebar policy; verify the exact threshold from the pinned source.
- Truecolor, ANSI-256, ANSI-16, and monochrome. Preserve current Junie appearance; alternate themes are additive tests unless already part of the accepted working contract.
- Long ASCII labels, combining characters, CJK/double-width text, emoji sequences, empty strings, and terminal-width clipping.
- Initial, focus, hover, press, disabled, editing, busy, modal, error, and completion states as relevant.
- Resize during editing, dragging, modal display, animation, and an input burst.

Do not use a passing count as a coverage definition. The manifest must state the exact required case keys and states, and the test run must match that set.

---

## 9. Verification design and artifacts

### 9.1 Keep three distinct layers

**Layer A — Pure view regression:** application view model or component props/state → production view code → cell dump → review image. This isolates the view and makes a large number of cheap state/size/style tests practical.

**Layer B — Interaction and state-machine regression:** deterministic input/time trace → real runtime/component/application updates → state/actions plus per-step frames. This proves focus, clicks, editing, modal behavior, timing, and domain outcomes.

**Layer C — Real terminal regression:** launch the actual binary under a PTY, send real terminal input, resize it, collect output, and verify lifecycle and representative complete flows. This catches parser/runtime/terminal integration errors that direct method calls can miss.

Keep component unit tests, domain unit tests, compile tests, documentation examples, and performance tests as additional layers. None substitutes for the three product layers above.

### 9.2 Artifact contract

An example proposed organization:

```text
verification/
  contracts/
    product.toml
    cases.toml
    exceptions.toml
  baselines/
    historical-499/manifest.tsv
    holla-4a46bcae/manifest.json
    holla-4a46bcae/<case-id>/<step>.cells.json
    holla-4a46bcae/<case-id>/<step>.png
  traces/
    showcase/...
    tablepro/...
    jackin/...
    holla/...
```

Candidate artifacts should normally be written to a separate output directory/CI artifact, not committed back into the source merely to make a checker find them:

```text
<verification-output>/<source-fingerprint>/
  manifest.json
  build-metadata.json
  results.json
  <case-id>/<step>.cells.json
  <case-id>/<step>.png
  <case-id>/<step>.diff.json
  <case-id>/<step>.diff.png
  <case-id>/events.jsonl
  <case-id>/observations.jsonl
  <case-id>/terminal-recording
  <case-id>/stderr.txt
```

Extend existing tooling rather than introducing a second incompatible artifact system without need. In particular, main's hard-coded evidence paths may need an explicit output/evidence-directory option so generated artifacts do not contaminate source-cleanliness checks.

### 9.3 Manifest requirements

Record source commit and tree/source fingerprint; executable hash; Cargo.lock hash; compiler/target/features/profile; fixture and trace versions; test-harness/terminal-parser/render-tool version; dimensions; color/theme; locale; environment allow-list; time/randomness configuration; baseline namespace and hash; expected case set; actual case set; per-step artifacts and hashes; result; and review/exception IDs.

Include every input that can affect rendering or behavior in the fingerprint, including fixture data, assets, build scripts, runtime code, and capture pipeline. Do not hash only `.rs` files or trust the branch name. Keep generated outputs outside the hashed input set to avoid circular provenance.

A correct source fingerprint can support reuse of artifacts only under an explicitly designed policy. Do not casually accept old evidence because “only documentation changed” without proving that no build, fixture, contract, or capture input changed.

### 9.4 Comparison rules

Compare cell position, grapheme/symbol, foreground, background, modifiers, cursor position/visibility, and relevant terminal modes. Normalize only explicitly nonsemantic data. Do not normalize away all whitespace, colors, timestamps, IDs, or dynamic output in ways that hide a real defect.

For cross-implementation interaction comparisons, use stable product-level identities. Preserve exact visible text and state where part of the contract, but do not require private type names, numeric hashes, or memory-layout artifacts to match.

On mismatch, report the first differing step and cell, changed region, expected/actual values, input history, logical state, and hit/focus diagnostic information. Large screenshots without a machine-readable difference are inadequate for reliable debugging.

### 9.5 Proposed trace example for the confirmed sidebar defect

The following is an illustrative schema to implement, not an existing command or test API:

```toml
id = "showcase.sidebar.buttons_click_80x24"
application = "showcase"
width = 80
height = 24
baseline = "holla-4a46bcae"
coordinate_system = "zero-based-terminal-cells"

[[steps]]
kind = "checkpoint"
name = "initial"
expected_page = "overview"

[[steps]]
kind = "pointer_move"
x = 4
y = 3

[[steps]]
kind = "pointer_down"
button = "left"
x = 4
y = 3

[[steps]]
kind = "pointer_up"
button = "left"
x = 4
y = 3

[[steps]]
kind = "checkpoint"
name = "buttons-open"
expected_page = "buttons"
compare_full_frame = true
```

Confirm these reference coordinates during V-01. The transport adapter must convert to the terminal protocol's coordinate convention exactly once. The expected row must not be obtained from the candidate hitmap, which would make the test circular.

### 9.6 Approval and baseline changes

During behavior-preserving extraction, expected product artifacts remain unchanged. A baseline update requires an explicit product/security exception stating the requirement, reason, affected case/step/region, before/after evidence, and approving identity.

The implementation agent must not approve its own unexplained differences. An approval token or file is not proof of visual review. Review records must identify actual artifacts and their hashes. A general “geometry changed because of ScrollRegion” explanation is insufficient to approve unrelated application or color-mode differences.

Changes that are truly internal—private IDs, package paths, memory layout—normally require no visual exception because they should not change product output.

---

## 10. Mandatory adversarial and mutation tests

### 10.1 Geometry and input

- Move a registered row one cell while preserving its painted label; coordinate-driven tests must fail.
- Paint a compact list with expanded hitboxes; F-02's reproducer must fail.
- Register a clipped row or an invisible overlay background; clicks must not reach it.
- Press on one control and release on another or outside; no unintended activation.
- Resize or switch pages between press and release; no stale activation/capture.
- Open a modal while background input is queued; background actions remain blocked according to the accepted policy.
- Remove or disable the focused control; focus reconciles safely and predictably.
- Send wheel events over a nested scroll region; only the intended owner scrolls.
- Send paste containing newline, escape-like text, Unicode, and application shortcut letters while editing; preserve text/edit safety and do not trigger unrelated global commands.

### 10.2 Timing, state, and effects

- Replace a deadline with an update counter; timing tests must fail.
- Advance the clock without user input; due visible transitions occur.
- Render repeatedly without update; state/effect counters remain unchanged.
- Commit an edit during draw; model-mutation tests must fail.
- Return success from a cancelled/wrong-acknowledgement destructive operation; domain-effect assertions must fail.
- Reorder/remove a selected item; identity/reconciliation tests detect incorrect logical selection.
- Remove status publication while preserving the page picture; shell/status assertions must fail.

### 10.3 Themes and customization

- Remove a claimed part-style resolver call; sentinel override tests must fail.
- Add inherited bold/reverse/underline unexpectedly; exact modifier comparison must fail.
- Replace disabled style with enabled style; fixture validity and output assertions must fail.
- Override a slot but leave hidden internal paint visible; the sentinel rendering test must fail.
- Use a custom row renderer with wide characters; measurement, clipping, and hit geometry remain coherent.

### 10.4 Gate and provenance integrity

- Delete a required test function or test target; the inventory/discovery gate must fail.
- Run a misspelled filter that matches zero tests; the wrapper must fail rather than report success.
- Delete a recipe or replace one recipe with a duplicate; exact-set coverage must fail.
- Substitute a candidate capture for baseline; provenance/baseline protection must fail.
- Use an artifact from a different commit, fixture, executable, size, or color; validation must fail.
- Mark an unreviewed mismatch approved with only a generated token; approval validation must reject the incomplete record.
- Break an artifact write halfway through; no partial file is published as complete evidence.
- Make a capture command fail but leave an older output file present; the old file must not be mistaken for a new success.

### 10.5 Capture and secret safety

Review process spawning, argument separation, temporary directories, symlink handling, cleanup, artifact publication, and error paths in any main-derived capture tools. Do not pass scenario text, titles, paths, or captured data through a shell as executable syntax. Retain the valid intent of the capture-hardening history without assuming every attempted hardening patch was correct.

Test representative secrets as sentinels: their raw values must not appear in screen buffers, diagnostics, debug output, error messages, event logs, screenshots, snapshots, or serialized evidence where masking is required. Preserve the previews' no-real-external-effects boundary.

---

## 11. CI and acceptance gates

### 11.1 Required evidence at each integration boundary

Use an explicit gate manifest rather than a convention that “the checks were green.” Each gate record must identify its required case/target set, execution command, source commit, configuration, result, artifacts, and owner.

| Gate | What must actually run | Failure conditions |
|---|---|---|
| Source integrity | Verify reference/candidate commits, source fingerprints, lockfile, executable hashes, and clean reference tree. | Mixed revisions, unknown dirty patches, stale binary, or missing provenance. |
| Build compatibility | Compile the complete intended workspace on the declared minimum Rust version and the primary toolchain. | Any failed member/target; accidentally excluding an application; missing extracted helper. |
| Test discovery | Enumerate required test targets and test cases, including binary-local and feature-gated tests. | Missing target/case, empty filtered run, unexpected ignored requirement, or duplicate replacing a missing case. |
| Component semantics | State/action, focus, pointer, disabled, edit, timing, and reconciliation tests for affected families. | Wrong output/action/effect, not just a panic. |
| Pure views | Render fixed models with explicit visual state, size, theme, and clock; compare full cells. | Unapproved symbol, style, width, geometry, cursor, clipping, or blank-cell differences. |
| Application traces | Replay input against the real application update/render path and verify intermediate/final state. | Inaccessible state, wrong hit target, wrong focus, missing status, or incorrect domain effect. |
| Terminal-process tests | Launch the actual binaries in a controlled terminal for selected end-to-end scenarios and lifecycle checks. | CLI drift, parser/coordinate mismatch, missing input, terminal mode leak, crash, or wrong exit status. |
| Historical parity | Reproduce the retained historical recipe namespace and separately the current Holla acceptance namespace. | Missing/extra/unaccounted cases, mismatched provenance, or an unapproved difference. |
| Public API | Compile external-consumer examples and test every advertised customization mechanism. | Private API dependence, inert builders/slots, or duplicated control mechanics needed by consumers. |
| Dependency boundaries | Check intended backend-free/core and feature combinations without workspace feature-unification masking. | Forbidden dependency, accidental backend requirement, or unsupported advertised feature combination. |
| Gate effectiveness | Run the relevant mutation/adversarial tests against the verification tools. | A deliberately introduced regression produces a pass. |
| Integration result | Run all required gates on the final merge commit or the exact committed merge candidate. | Evidence exists only for a predecessor, a separate worktree, or a different merge resolution. |

A gate may have legitimately different coverage at an early extraction step and at final acceptance. Make that coverage explicit. Do not present an early component-only pass as whole-application approval.

### 11.2 Cargo command contract

The following are **commands to execute during implementation**, not commands run by this audit. Confirm target names and supported features with Cargo metadata before using them. Rust 1.88 is the minimum version declared in the inspected working manifest; any deliberate change to this promise needs its own decision and consumer impact review. [S01]

```bash
# Source/build inventory; save the complete output as evidence.
cargo metadata --locked --format-version 1
rustc --version --verbose
cargo --version

# Whole-workspace compilation: do not infer this from a perf-only workflow.
cargo +1.88.0 check --locked --workspace --all-targets --all-features
cargo +stable check --locked --workspace --all-targets --all-features
cargo +stable build --locked --workspace --all-targets --all-features

# Hygiene and documentation; record the exact compiler versions resolved.
cargo +stable fmt --all -- --check
cargo +stable clippy --locked --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo +stable doc --locked --workspace --all-features --no-deps

# Discovery and execution are separate obligations.
cargo +stable test --locked --workspace --all-targets --all-features -- --list
cargo +stable test --locked --workspace --all-targets --all-features
cargo +stable test --locked --workspace --doc --all-features
cargo +1.88.0 test --locked --workspace --all-targets --all-features
```

Also run default-feature tests and each advertised feature combination. An all-feature run is not a substitute for these. Once the extracted package actually provides a backend-free core, check it in isolation using its actual package name and relevant target; do not add a premature no-default-feature gate that cannot yet apply to the legacy package. The intended final check can resemble:

```bash
# Final extracted package only; validate that this name matches Cargo metadata.
cargo +stable check --locked -p junie-tui --lib --no-default-features
```

Keep functional tests that need the `testing` feature distinct from API probes for ordinary consumers. Compile-fail tests may be compiler-output-sensitive: pin their primary compiler or normalize diagnostics carefully without deleting the semantic failure check.

The `+stable` commands above are convenient selectors. Evidence must record the exact resolved version; reproducible baseline capture must use a pinned version. Do not compare a moving compiler/tooling environment against an old artifact without recording and reviewing that difference.

### 11.3 Test discovery must survive the package move

A root-level test directory in a virtual workspace does not, by itself, establish an executable test target. After PKG-02, verify where each old test is compiled. For every old binary-local module and integration test, record its new owning package and target. Preserve conditional compilation and fixture paths deliberately rather than assuming `cargo test --workspace` found everything.

Compare the expected inventory with the executed inventory by stable obligations and test targets. Test function counts alone are insufficient: replacing one meaningful test with a no-op keeps the count unchanged. Use the mutation tests and semantic assertions to establish actual coverage. Report which required ignored tests were explicitly executed and why other ignored tests are non-gating.

A test wrapper must fail when its requested filter matches zero tests. It must preserve the process's nonzero exit code, including when output is piped to `tee`; use shell pipeline error handling appropriately. Logs must include the test summary and exit status, not only selected passing lines.

### 11.4 Make parity executable from a clean checkout

Main already wires its parity contract into the boundary checker; recovery should inspect and reuse valid logic rather than creating a second contradictory checker. However, missing generated evidence must have a real producer in the pipeline, and the hard-coded three-binary/499-recipe assumptions must not be mistaken for coverage of current Holla. [S15] [S17] [S24]

Implement the following ordered stages:

1. Validate the read-only baseline manifest and its source provenance. Download or reproduce its artifacts using a versioned, integrity-checked mechanism.
2. Build the candidate from the exact commit under test and record its executable hashes.
3. Run candidate capture/replay against the complete declared case set. Write into a fresh output directory, not over an old successful result.
4. Compare candidate output to the appropriate baseline namespace. Produce machine-readable differences plus human-review images.
5. Validate any approved exception against its exact requirement, case, step, artifact hashes, and approver. CI must never approve an unexplained difference itself.
6. Run the boundary/parity contract against those just-produced artifacts. Do not silently fall back to stale local files.
7. Publish logs and differences even when comparison fails. Mark the required gate failed, not skipped or neutral.

Keep generated evidence outside the source tree fingerprint, or define a precise exclusion contract. Otherwise capturing evidence can dirty the source tree whose cleanliness the evidence requires, creating a circular process. Exclusions must be explicit and limited to generated outputs; never exclude application or fixture source to hide a mismatch.

Represent the old 499 recipes as a retained, versioned case set if they remain valid. Define the current four-application contract as its own exact set, with mapping where cases are genuinely equivalent. Do not enforce the literal number 499 as a permanent cap, and do not delete historical obligations merely because current case names differ.

### 11.5 CI job dependencies and final status

Separate build, discovery, component tests, application parity, terminal-process tests, API boundaries, and documentation jobs where practical. A documentation failure should not hide every independent behavioral result. A failed application build, however, cannot produce a legitimate pass for tests requiring that binary.

An always-running final aggregator must check every required job and configuration explicitly. `skipped`, `cancelled`, `neutral`, absent evidence, and a zero-case run are not equivalent to success. Print a concise summary of completed, failed, and unexecuted obligations. Preserve the actual early failure cause, as F-01 does, instead of attributing the run to later unexecuted gates.

Record artifact retention and a durable location for release/merge acceptance evidence. A temporary CI artifact link that expires cannot be the only long-term record of an accepted baseline change. Keep the repository's required checks aligned with the gate manifest and exercise a deliberately failing PR to prove they block integration.

---

## 12. Integrating the recovered `holla` branch into main

### 12.1 Branch policy

The writable recovery implementation starts from the accepted Holla revision. It must not begin by replacing its applications with main's versions, merging main wholesale into a dirty working checkout, or copying the entire main `crates/tui` tree before a preservation harness exists.

Use isolated local worktrees and task branches for concurrent implementation. Integrate verified slices into the recovery line for `holla`. Updating a remote branch is a separate repository action; this report itself has not created, pushed, reset, or merged any branch.

### 12.2 Review the actual three-way difference

Maintain three explicit references: the shared ancestor, the accepted Holla baseline, and the main revision being considered for integration. Add the recovery candidate as a fourth reference once implementation starts. The 10 Holla-side commits and 441 main-side commits identified here explain why selecting one side wholesale is not adequate. Recompute the graph when remote heads change. [S23]

For every changed source/configuration family, classify the intended disposition:

| Class | Required treatment |
|---|---|
| Working product behavior and branch-only Holla capability | Preserve unless an explicit, separately approved exception says otherwise. |
| Main API/component implementation | Reuse only after its semantics, dependencies, customization, and relevant application parity are proved. |
| Main security/capture correction | Inspect the complete dependency chain and adversarial tests; retain the actual correction, not merely a commit with a reassuring title. |
| Main additive feature | Preserve where compatible, or explicitly defer with a reason and owner; do not let it force a silent change to the accepted default experience. |
| Main self-baseline or broad visual exception | Treat as evidence to review, not as authoritative replacement for the working product baseline. |
| Obsolete rewrite scaffolding or duplicate control implementation | Remove only after equivalent surviving behavior and tests are identified. |
| Unexplained difference | Remain unresolved and block deletion/integration of the affected requirement. |

Keep this in `main-salvage.tsv` with source commits, paths, disposition, rationale, dependencies, tests, and reviewer. A file being “newer” is not a disposition rule.

### 12.3 Safe integration sequence

1. Finish and verify the recovery candidate against the frozen Holla product contract. Record its exact commit and all gate results.
2. Fetch the current main head and identify changes since this audit's `c12cad...` reference. Review that delta rather than silently assuming the audited main is still current.
3. Create a disposable integration branch/worktree from the main head to be merged into. Merge the verified Holla recovery line there using an ordinary merge workflow that exposes conflicts.
4. Resolve conflicts at the level of behavior and ownership. Use Holla's accepted UX as the product contract, the corrected architecture as the structural contract, and the salvage ledger for applicable main-only fixes. Do not choose `ours` or `theirs` for entire architectural trees without file-by-file justification.
5. Preserve all four binaries, their startup routes, flags/aliases, scenario data, assets, docs, test obligations, and launch commands. Check binary-local tests and feature relationships again after conflict resolution.
6. Create a real local commit for the merged candidate before generating final provenance-bound evidence. An uncommitted hand-edited tree is not the final tested commit.
7. Run the full gate manifest on that commit from a clean worktree. Rerun capture/replay; do not relabel predecessor artifacts with the merged commit hash.
8. Review every remaining difference and approved exception. Recheck the missing-module class of integration error by compiling all members/targets, not only recently edited applications.
9. Submit the tested result through the chosen protected integration process. When that process creates a different merge/squash commit, verify the resulting source tree and required checks; generate final evidence for the resulting commit rather than claiming identity with an earlier SHA.
10. If main changes again before integration, repeat the delta review and merged-result checks. Old green checks are not transferable to a changed merge result.
11. After integration, run the actual main binaries and the selected terminal-process smoke suite again. Keep a permanent acceptance record linking baseline, recovery commit, merge result, toolchain, artifacts, and exceptions.

Do not use a synthetic `ours` merge to make the graph look integrated while discarding main changes without review. Do not force-push/reset main to the Holla tree. Do not squash away the requirement/disposition records even when the repository chooses a squash-merge history.

### 12.4 Rollback and regression handling

If a slice introduces a regression, stop that slice's promotion. Retain its evidence and reproducer, restore the last verified implementation in the recovery line, and reduce the change size. Other independent work may continue in isolated branches, but it must not hide the regression in a mixed integration commit.

If a regression is discovered after merging, create a normal reviewed corrective or revert commit. For a merge revert, first inspect parent order and document the intended mainline; do not run a blind mainline-number command. Preserve unrelated subsequent work. Reverify the recovered product and state any main-only fixes that need reapplication.

The rollback target is a **verified coherent application/runtime state**, not a collection of individual files picked from different revisions. Keep the baseline artifacts and the negative regression test so the same failure cannot recur unnoticed.

---

## 13. Proposed multi-agent execution and ownership

**This section is a future execution plan. These subagents were not launched during this audit.** Use the actual models and execution facilities available in the implementation session; do not bake obsolete model identifiers into the product architecture.

### 13.1 Workstreams

| Role | Primary responsibility | Owned outputs / boundaries | Cannot self-approve |
|---|---|---|---|
| Integrator / lead | Dependency order, shared API decisions, slice integration, package moves, CI, final merge. | Cargo manifests/lockfile, public facade, shared contract changes, workflow files, integration status. | Its own final merged-result parity. |
| History / architecture analyst | Complete H-02; resolve old versus current obligations; assess main reuse candidates. | History exports, supersession map, architecture decisions and salvage recommendations. | A claim that implementation meets the contract without execution evidence. |
| Baseline / verification engineer | Capture immutable working behavior; independent traces; test/gate effectiveness. | Baseline manifests, fixture contract, comparator, reference artifacts, negative gate tests. | Unexplained product changes or candidate-generated replacement baselines. |
| Runtime / interaction engineer | Input normalization, focus, pointer capture, layers, identity, clocks, diagnostics. | Assigned runtime modules and their focused tests; shared-interface changes go through lead. | Whole-application parity based only on unit tests. |
| Rendering / component engineer | Shared layout, themes, text measurement, part/slot behavior, family extraction. | One assigned component family and tests at a time; not every app simultaneously. | Override/geometry conformance without external consumer and parity proof. |
| Showcase engineer | Preserve shell and all pages while switching verified components. | Showcase application files and application tests. | Baseline changes to make its migration pass. |
| TablePro engineer | Preserve database preview workflows, editor/grid behavior, safety policy, and domain adapters. | TablePro application/domain files and traces. | Destructive-operation safety without independent effect assertions. |
| Jackin engineer | Preserve scenarios, transition/timing semantics, CLI and simulated integrations. | Jackin application files and traces. | Claims that scenario screenshots prove real navigation. |
| Holla engineer | Preserve all branch-only flows, ranking/scope rules, previews, task DAGs, activity state, and safety. | Holla application files and traces. | Dropping a route because the old main plan did not list it. |
| Independent reviewer | Reproduce failures, inspect candidate patches, challenge assertions, approve gate completeness. | Review records and independent reproductions; does not rewrite the builder's evidence to conceal a failure. | Its own implementation changes; another reviewer checks those. |

This is a role inventory, not a requirement to run ten workers simultaneously. A small number of bounded workers with clear ownership is preferable to many agents editing the same core files. Separate workstreams can be time-multiplexed when execution capacity is limited, but independent approval still matters.

### 13.2 Concurrency rules

Use one worktree and branch per writing worker. Each task declares its base commit, file ownership, expected shared interfaces, dependencies, and forbidden changes. Shared core files, Cargo configuration, and the public facade have a single writer at a time. Application teams request interface changes through the lead instead of racing to edit shared components.

The baseline owner controls expected artifacts. Builders can produce candidate captures and propose scoped exceptions, but cannot silently bless their own output. Reviewers use a clean checkout of the committed candidate, not the builder's mutable worktree.

Integrate in the dependency order in Section 6. Application work can characterize behavior in parallel before runtime changes; it must not independently invent incompatible runtime contracts. Commit bounded changes frequently in task branches, with source and evidence tied to those commits. Frequent commits do not justify promoting unverified mixed work to the shared recovery branch.

### 13.3 Task handoff format

Every handoff must include:

```text
Task ID:
Base commit:
Candidate commit:
Owned paths:
Contract / behavior preserved:
Change summary:
Main-derived code and source commits:
Tests discovered:
Commands executed and exit statuses:
Reference / candidate / difference artifact paths and hashes:
Known failures and unexecuted checks:
Baseline exceptions requested (normally none):
Reviewer findings and resolution:
Next dependency unlocked:
```

Use distinct status values: `not-started`, `in-progress`, `implemented`, `reviewed`, `verified`, and `integrated`. “Implemented” is not “verified.” A reviewer must be able to reproduce the result without reconstructing hidden local modifications.

When blocked, document the precise dependency and work on another independent bounded task. Do not invent missing evidence, waive a critical gate, merge a broken slice, or create an unbounded chain of nearly identical “final” checkpoints. Changes to the plan require a decision record with affected obligations, not an unexplained status edit.

---

## 14. Completion criteria and first implementation sequence

### 14.1 Definition of done

The following are acceptance requirements, not boxes already checked by this report:

- [ ] Both requested document histories have a complete reachable-change index, parent-relative patches, semantic review, and current-obligation map.
- [ ] The current goal clearly prioritizes the accepted product contract and defines reusability, non-goals, exceptions, and deletion rules without contradictory competing authority.
- [ ] All four binaries build and launch; all required flags, aliases, defaults, environment controls, exit codes, and terminal lifecycle behavior are preserved or explicitly approved.
- [ ] All 22 Showcase pages and shell behaviors, all TablePro workflows, all Jackin scenarios/transitions, and all Holla scenarios/transitions are accounted for by exact coverage sets.
- [ ] F-02's compact-sidebar coordinate reproducer passes, with one authoritative layout for painting and hit registration across resize boundaries.
- [ ] Timing is based on an explicit clock; unrelated input and draw calls cannot advance application jobs or animations improperly.
- [ ] Contextual hints, transient status, focus behavior, modal behavior, and diagnostics match the accepted experience.
- [ ] View fixtures render independently from business effects, can be dumped to stable files and rendered for review, and compare full cell/style/cursor information.
- [ ] Real-input traces prove reachability and effects; selected actual-terminal process tests validate the end-to-end boundary rather than only in-process rendering.
- [ ] Every extracted component has meaningful state, interaction, styling/customization, Unicode, clipping, and external-consumer coverage.
- [ ] Each old implementation/test removal has a surviving behavior/evidence mapping; no obligation disappeared during package movement or test renaming.
- [ ] Historical and current baseline namespaces are immutable and independently attributable; every visual/behavioral exception is narrow, justified, and reviewed.
- [ ] The gate mutation suite proves that wrong hitboxes, missing tests, bad timing, incorrect styles, stale evidence, and wrong safety effects are detected.
- [ ] All required CI jobs actually executed on the exact candidate; skipped/missing/zero-case work is not counted as a pass.
- [ ] Applicable main-only security and infrastructure corrections and all Holla-side product work have explicit integration dispositions.
- [ ] The exact committed merge result passes the full acceptance manifest from a clean environment; there is a documented coherent rollback path.

### 14.2 Start with these changes, in this order

**First change set: evidence and corrected scope only.** Freeze the revisions; finish the history review; inventory all four apps; record real baseline build/test health; capture the accepted reference; write the consolidated goal. No application rewrite, package rename, or old-renderer deletion belongs in this change set.

**Second change set: verification that fails for known wrong behavior.** Add a pure Buttons view fixture, a clock-controlled Buttons behavior trace, the compact-sidebar coordinate trace, and Jackin CLI contract tests. Demonstrate that candidate versions with the known regressions fail. Keep expected output tied to the working branch.

**Third change set: one complete extraction.** Extract a button using the existing working page and then navigation using a single shared layout. Prove appearance, input, focus, timing, status, and customization before moving to another family. This is the point where the proposed API earns acceptance through a real consumer.

**Subsequent change sets:** shared foundations and component families in Section 6, then application completion, public-consumer proof, package moves, deletion, and final merge verification. No later phase may retroactively excuse an unexplained regression introduced earlier.

### 14.3 Final recommendation

Do not repeat main's application rewrite from a different branch. Preserve the running Holla applications and make each reusable abstraction prove that it can represent their existing behavior without special hidden painting or duplicated event mechanics.

The essential migration rule is:

> Characterize the accepted behavior → add an independent regression test → extract the smallest reusable component → replace one caller → compare views and real interactions → verify the public API → delete the old code only after equivalence → test the actual merge result.

This repairs both failures: a broken product behind green-looking architecture checks, and a working product whose reusable API was never truly finished.

---

## Appendix A. Historical decision links

The links below identify the actual commits discussed in Section 3. They are audit leads and evidence records; a linked title is not itself proof of a successfully executed implementation.

| Document / topic | Commit | Evidence use |
|---|---|---|
| Goal: Refactoring-goal introduction | [`e48137f1852e`](https://github.com/donbeave/terminal-components-claude/commit/e48137f1852ea8f1b2885478a3f3312749443831) | See Section 3.1 and source [G1]. |
| Goal: Removal of stale authority references | [`dce91d51c848`](https://github.com/donbeave/terminal-components-claude/commit/dce91d51c848cf07eaad4058296775e644940bd5) | See Section 3.1 and source [G2]. |
| Goal: Execution-role prescriptions | [`2d81eec4358e`](https://github.com/donbeave/terminal-components-claude/commit/2d81eec4358e88ef3a37c52466d612ba9e493238) | See Section 3.1 and source [G3]. |
| Goal: Research prescription change | [`25ea92b08f41`](https://github.com/donbeave/terminal-components-claude/commit/25ea92b08f41838896af58ab062834cab606ba9b) | See Section 3.1 and source [G4]. |
| Goal: Parity-first active recovery goal | [`efada04419f7`](https://github.com/donbeave/terminal-components-claude/commit/efada04419f72ecada75cfffb4b1bc6014a05ba2) | See Section 3.1 and source [G5]. |
| Architecture: Conformance inventory versus registry | [`2c848f9e10d0`](https://github.com/donbeave/terminal-components-claude/commit/2c848f9e10d050c3099a826067aecca04d027157) | See Section 3.2; distinguish historical claim from current verification. |
| Architecture: Public surface and documentation consistency | [`2399c1adeef5`](https://github.com/donbeave/terminal-components-claude/commit/2399c1adeef55c6b69a0faa1875bdce888ec69d2) | See Section 3.2; distinguish historical claim from current verification. |
| Architecture: Missing example and inadequate count check | [`1b580d749ba7`](https://github.com/donbeave/terminal-components-claude/commit/1b580d749ba761809bf6d52cf94139c67518e904) | See Section 3.2; distinguish historical claim from current verification. |
| Architecture: Legacy-test obligation accounting | [`e6eaca489675`](https://github.com/donbeave/terminal-components-claude/commit/e6eaca4896755d23ec290deaf97508114bc06964) | See Section 3.2; distinguish historical claim from current verification. |
| Architecture: Advertised parts versus resolver behavior | [`de817f2627b1`](https://github.com/donbeave/terminal-components-claude/commit/de817f2627b12739c658a08c0a2ce456e2da016a) | See Section 3.2; distinguish historical claim from current verification. |
| Architecture: Unsafe migration/deletion ordering | [`3dc44c48625c`](https://github.com/donbeave/terminal-components-claude/commit/3dc44c48625c1cd03e67e685d41111d55919bfde) | See Section 3.2; distinguish historical claim from current verification. |
| Architecture: Ineffective style assertions | [`364689766c64`](https://github.com/donbeave/terminal-components-claude/commit/364689766c64985aced3b4bb1bdb79a5ee15be54) | See Section 3.2; distinguish historical claim from current verification. |
| Architecture: Package identity and migration timing | [`b60c8280d1e1`](https://github.com/donbeave/terminal-components-claude/commit/b60c8280d1e117dfb2f095a59c1fb0c783850ebd) | See Section 3.2; distinguish historical claim from current verification. |
| Architecture: Missing tests despite prior green checks | [`444ae3d9dbd4`](https://github.com/donbeave/terminal-components-claude/commit/444ae3d9dbd449115c06e3695c326b76c2b43c1e) | See Section 3.2; distinguish historical claim from current verification. |
| Architecture: Disabled fixture validity | [`4d2621102a75`](https://github.com/donbeave/terminal-components-claude/commit/4d2621102a75343cb000bda483c2235a76ce2627) | See Section 3.2; distinguish historical claim from current verification. |
| Architecture: Capture-hardening claim | [`0defae2283b4`](https://github.com/donbeave/terminal-components-claude/commit/0defae2283b4a2599d3a469e247ebed31283412d) | See Section 3.2; distinguish historical claim from current verification. |
| Architecture: Subsequent capture correction/revert | [`067869c7bebc`](https://github.com/donbeave/terminal-components-claude/commit/067869c7bebc633225fc0dd1c1c8e2d1228cbc90) | See Section 3.2; distinguish historical claim from current verification. |
| Architecture: Shared geometry difference classification | [`8514210cf859`](https://github.com/donbeave/terminal-components-claude/commit/8514210cf8592bc9ed1a9f9693847077d08c904b) | See Section 3.2; distinguish historical claim from current verification. |


## Appendix B. Read-only history collection procedure

These commands are a starting procedure for H-02. They were **not executed by this audit** and do not replace the semantic review. Run them in a complete local Git checkout whose object access has been verified. Store outputs outside the source worktree. Do not modify the accepted baseline to collect history.

```bash
set -euo pipefail
MAIN=c12cad8728755cd2d03eefdd8e02891143fca86d
HOLLA=4a46bcae03ac14d2cf6c7d55657babf6b584996e

# Verify source objects exist and inspect the current worktree without changing it.
git cat-file -e "${MAIN}^{commit}"
git cat-file -e "${HOLLA}^{commit}"
git status --short
git worktree list --porcelain
git merge-base "$MAIN" "$HOLLA"
git rev-list --left-right --count "${HOLLA}...${MAIN}"

# Parent-aware, full-reachable path history. Capture the complete output.
git log --full-history --topo-order --reverse \
  --format='%H%x09%P%x09%aI%x09%s' "$MAIN" -- \
  REFACTORING_GOAL.md COMPONENT_ARCHITECTURE.md

# Follow renames for each path independently. Review names before exporting snapshots.
git log --follow --name-status "$MAIN" -- REFACTORING_GOAL.md
git log --follow --name-status "$MAIN" -- COMPONENT_ARCHITECTURE.md

# Include merge-parent diffs and creation diffs, not just the first-parent story.
git log --root --full-history -m -p --find-renames "$MAIN" -- \
  REFACTORING_GOAL.md COMPONENT_ARCHITECTURE.md
```

A production exporter must additionally enumerate all reachable commits and compare the two document entries against **each parent tree**, following the rename paths discovered above. This cross-check is necessary because history simplification and rename heuristics can omit or alter how a path's changes are presented. For a root/creation comparison, use the empty tree. Explicitly record path absence and deletion; a failed `git show` must not be mistaken for an empty document.

For each resulting commit/path pair, export the full commit message, parents, tree ID, blob ID, source snapshot, and per-parent patch. Hash each output and produce a manifest. A merge can inherit a document unchanged from one parent while changing it relative to another; retain that distinction. Review both unique document snapshots and the changes that made them authoritative on main.

Reviewers should then fill the obligation and supersession tables in H-02, including formatting-only commits. Compare their completed review set to the export manifest. Report any object access failure or unresolved rename explicitly instead of marking the history complete.

## Appendix C. Source index

All source-file links below are pinned to the audited commits. The CI link identifies the inspected execution job. Source code establishes the code differences described here; only the cited CI execution establishes an observed remote execution result. Proposed tasks and tests elsewhere in the document are not claims of existing implementation.

| Source | Description |
|---|---|
| [S01] | Working Holla Cargo manifest: four binaries, default run target, declared Rust version. |
| [S02] | Main Cargo workspace manifest: refactored packages and three application members. |
| [S03] | Main Showcase shell, navigation composition, footer, and inspector. |
| [S04] | Main generic navigation list: section layout, registration, state, and actions. |
| [S05] | Working Showcase shell and actual compact row registration. |
| [S06] | Main Buttons page: update-count-based busy state and feedback handling. |
| [S07] | Working Buttons page: elapsed-time deadline and status publication. |
| [S08] | Working Jackin entry point: default scenario, flags, errors, help, and environment handling. |
| [S09] | Main Jackin entry point: changed CLI behavior. |
| [S10] | Failing Rust 1.88 CI job for main at the audited revision, including the unresolved import. |
| [S11] | Main component module declarations; the referenced overlay helper is not declared. |
| [S12] | Main accumulated component architecture contract. |
| [S13] | Main historical refactoring goal. |
| [S14] | Main active parity-first recovery goal. |
| [S15] | Main historical parity/evidence contract implementation. |
| [S16] | Main tracked parity directory, inspected for checked-in evidence. |
| [S17] | Main CI workflow and ordering of compile/test/boundary gates. |
| [S18] | Main Showcase default-view matrix test. |
| [S19] | Working Holla scenario and verification documentation. |
| [S20] | Main TablePro application, named surfaces, and fixture-state construction. |
| [S21] | Frozen main source tree. |
| [S22] | Frozen Holla source tree. |
| [S23] | Comparison of the two pinned commits and their divergence. |
| [S24] | Main verification-task implementation and boundary checker registration. |
| [S25] | Working Showcase page registry and event/hint/edit/animation contracts. |

[S01]: https://github.com/donbeave/terminal-components-claude/blob/4a46bcae03ac14d2cf6c7d55657babf6b584996e/Cargo.toml
[S02]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/Cargo.toml
[S03]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/apps/showcase/src/app.rs
[S04]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/crates/tui/src/components/nav_list.rs
[S05]: https://github.com/donbeave/terminal-components-claude/blob/4a46bcae03ac14d2cf6c7d55657babf6b584996e/src/bin/showcase/app.rs
[S06]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/apps/showcase/src/pages/buttons.rs
[S07]: https://github.com/donbeave/terminal-components-claude/blob/4a46bcae03ac14d2cf6c7d55657babf6b584996e/src/bin/showcase/pages/buttons.rs
[S08]: https://github.com/donbeave/terminal-components-claude/blob/4a46bcae03ac14d2cf6c7d55657babf6b584996e/src/bin/jackin_preview/main.rs
[S09]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/apps/jackin-preview/src/main.rs
[S10]: https://github.com/donbeave/terminal-components-claude/actions/runs/34002472849/job/101403684087
[S11]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/crates/tui/src/components/mod.rs
[S12]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/COMPONENT_ARCHITECTURE.md
[S13]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/REFACTORING_GOAL.md
[S14]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/GOAL.md
[S15]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/xtask/src/parity.rs
[S16]: https://github.com/donbeave/terminal-components-claude/tree/c12cad8728755cd2d03eefdd8e02891143fca86d/parity
[S17]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/.github/workflows/ci.yml
[S18]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/apps/showcase/tests/visual.rs
[S19]: https://github.com/donbeave/terminal-components-claude/blob/4a46bcae03ac14d2cf6c7d55657babf6b584996e/src/bin/holla/README.md
[S20]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/apps/tablepro/src/app.rs
[S21]: https://github.com/donbeave/terminal-components-claude/tree/c12cad8728755cd2d03eefdd8e02891143fca86d
[S22]: https://github.com/donbeave/terminal-components-claude/tree/4a46bcae03ac14d2cf6c7d55657babf6b584996e
[S23]: https://github.com/donbeave/terminal-components-claude/compare/4a46bcae03ac14d2cf6c7d55657babf6b584996e...c12cad8728755cd2d03eefdd8e02891143fca86d
[S24]: https://github.com/donbeave/terminal-components-claude/blob/c12cad8728755cd2d03eefdd8e02891143fca86d/xtask/src/main.rs
[S25]: https://github.com/donbeave/terminal-components-claude/blob/4a46bcae03ac14d2cf6c7d55657babf6b584996e/src/bin/showcase/pages/mod.rs
[G1]: https://github.com/donbeave/terminal-components-claude/commit/e48137f1852ea8f1b2885478a3f3312749443831
[G2]: https://github.com/donbeave/terminal-components-claude/commit/dce91d51c848cf07eaad4058296775e644940bd5
[G3]: https://github.com/donbeave/terminal-components-claude/commit/2d81eec4358e88ef3a37c52466d612ba9e493238
[G4]: https://github.com/donbeave/terminal-components-claude/commit/25ea92b08f41838896af58ab062834cab606ba9b
[G5]: https://github.com/donbeave/terminal-components-claude/commit/efada04419f72ecada75cfffb4b1bc6014a05ba2

---

**Audit boundary:** this artifact provides a source-backed recovery plan. It does not certify runtime parity, claim that subagents executed, or claim that every historical architecture revision was completely reviewed. Those remaining verification obligations are explicit gates, not implied successes.
