# Main-based Holla integration execution plan

## Authority and completion

The complete [user task](main-holla-integration-task.md) is the acceptance
checklist. Main supplies the reusable architecture; pinned Holla supplies the
product contract interpreted with DESIGN.md. Completion requires all four apps,
all surviving architecture obligations, all three verification layers, the full
gate matrix, independent review, and verification of the exact merged commit.
A scoped pass never establishes full application or goal acceptance.

This supersedes Holla-first/tree replacement instructions in
HOLLA_REFACTOR_RECOVERY_PLAN.md and holla-project/notes/main-vs-holla-refactor-analysis.md,
historical three-app scope, machine-specific defaults, and unavailable-model
restrictions. Historical findings remain investigation leads. Safety exceptions
require reproduced evidence, narrow documentation, and independent review.

## Immutable inputs and ownership

- MAIN_BASE: c12cad8728755cd2d03eefdd8e02891143fca86d.
- HOLLA_REFERENCE: 794b095c196562d38f1b6f7ce379c128af2a023d.
- Integration: codex/main-holla-integration, [PR #1](https://github.com/donbeave/terminal-components-claude/pull/1).
- Original dirty checkout, reference tree, original artifacts, and existing build
  directories remain preserved. No reset/clean, force-push, tree replacement,
  fake ancestry merge, or unreviewed baseline approval.
- External evidence root E: ../terminal-components-integration-evidence.
  This is an execution location, never a portable application default.
- Root alone writes integration and this plan. Agents use isolated branches,
  pinned dependencies, explicit file ownership, signed commits and Codex trailers.
  All integration is committed/pushed to PR #1; merge follows full acceptance.

Dependency order: characterize → reproduce → shared fix → caller migration →
independent view/behavior proof → review → integration → exact final verification.

## Requirement ledger

| Task | User sections | Current state | Remaining proof/work |
|---|---|---|---|
| T01 Sources/work preservation | 1–2 | Pinned, isolated, original dirty tree preserved | Revalidate concurrent main before merge |
| T02 History/obligations | 3 | Complete reachable history and parent-relative patches reviewed; H01–H20/1483 leads and68 edges indexed | Consolidate normative GOAL/architecture/coordination docs; close every mapped obligation |
| T03 Baselines/tools | 4,8–9 | Historical499 recipes:1497 original text/ANSI/cursor artifacts preserved;998 missing HTML/PNG reproducibly regenerated from exact historical sources | Current pinned-Holla full reference/candidate image review; keep historical and Holla namespaces distinct |
| T04 Compiler/chrome | 5 | Real shared chrome repair integrated and scoped verified | Include in final full matrix |
| T05 Runtime/time/identity | 5 | Publication, immutable Scene binding, typing, feedback clock, activation origin, focus restoration, hover/capture and terminal cleanup repairs integrated | Full product consumers/live timing; strict style measurement; final adversarial matrix |
| T06 Theme/components | 6 | Rich Props, keyed Tree lineage, Grid previews/gutters, logical Split, matched paint and List activation policies integrated | Columns clipping repair review; multiline List; Grid fetch; every family/customization layer; seven Meter digest differences unapproved |
| T07 Showcase | 7 | Shell/sidebar/namespace, elapsed Buttons and Progress, allocation repairs integrated | Actual Grid adapter, all22 pages, responsive geometry, independent images/live flows; no full-app acceptance |
| T08 TablePro | 7 | Stable tabs/rows, guarded destructive routes, SQL Unicode/completion model, row actions and real Ctrl+O typed destinations integrated | Switcher Columns visual migration, completion popup reachability, explorer cadence, remaining editor/history/filter/structure/save routes and exact presentation |
| T09 Jackin | 7 | CLI/lifecycle/tick admission, exact cwd association and eight-scenario fixture graph integrated | Real Manager replacing historical paint; captured actions/repository/editor saves; multiline roster; all other source routes/scenarios/CLI/visual proof |
| T10 Holla | 7 | Real App/binary,11 scenarios, query/feedback, exact Plan binding, source footer/menus and historical journeys through28ebbbe integrated | Final Git/disk identity review; full52 semantic mapping;132-case/color/Paper/image/live acceptance |
| T11 Verification layers | 8 | Actual production pure-view, runtime and PTY slices exist | Complete four-app reference/candidate/cell/cursor/image and real-input matrices; no fixture-only reachability claims |
| T12 Gates/inventory/performance | 9 | Pipefail, four-binary inventory, actual Cargo-context and test tools integrated; scoped MSRV/Clippy/perf runs pass | Required3211 historical obligations/700 semantic deltas and final required manifest; exact full matrix; strict style/Picker gates; capture tool integration/Linux proof |
| T13 Independent review | 2,8,10 | Continuous committed slice review; original reds retained | Review every remaining candidate and all changed surfaces; no blanket acceptance |
| T14 Merge/exact proof | 10 | PR #1 open, main unmerged | All acceptance gates on resolved candidate, main revalidation, protected merge, exact merged-source verification |

## Integrated evidence index

Entries are scoped results on named commits, not proof for later changed source.
Full commands, findings, original failures, artifact hashes and prior counts remain
in the immutable checkpoint archives and E reports.

| Area | Integrated commits | Evidence and limits |
|---|---|---|
| Foundations and earlier app repairs | Through d36a75b and subsequent runtime chain | Prior checkpoint retains full slice ledger, including historical artifacts, theme/Mono, chrome, identity, viewport and publication proofs |
| Pointer/focus/terminal ownership | cdbcb05/715ee07/f5c3763,577bf53,8569cf7 | E/removed-opener-independent and E/terminal-cleanup/root-8569cf7; successful publication validates removed openers; actual PTYs restore input on errors; unwritable output cannot prove escape delivery |
| Typed completion and switcher | c23d2fb/8949e68/c44c0f3,141fe72/2a41e3e | E/completion-independent/REPAIR_REVIEW.md:589 source ASCII cases exact; E/tablepro-quick-switcher/INDEPENDENT-ROUTES.md:340/356 frames unchanged,16 changed;11 TablePro release gates pass; full UI still open |
| Tree/Props/List | 87c1664,a88c81e/2317e34,043c117/cb59b80 | E/props-rich-independent,tree-lineage-independent,jackin-manager-production/INDEPENDENT-LIST-REVIEW.md; root759 library+46 Props/Tree and22 List tests pass; replacement/ABA and executed mutation proofs |
| Grid preview/keyboard editor | adeecef/02ef49f/e164c60/66011f8/dbf9f94 | E/grid-column-geometry/INDEPENDENT-TRAIN-REVIEW.md; root759 library+50 public tests and two unchanged release collection gates |
| Detailed Grid gutter | aacb7cb/57e9545/e9fb460/d86fd35 | E/grid-gutter-independent/FINAL_REVIEW.md SHAe573569d…93d16:829 independent tests and rebuilt128 cases/83,968 compact cells; root759+50 pass; repeat-click/hover original reds retained |
| Grid header prefixes | 3092410 | E/grid-column-geometry/INDEPENDENT-PREFIX-REVIEW.md SHAafb616fa…7221ec;20 independent MSRV tests, root33 Grid tests; keyed two-cell reservation and explicit ICON override precedence; PrimaryKey additive, PrimaryMark preserved; workspace MSRV Clippy and stable fmt pass |
| Logical Split geometry | c562ee5/ec2ef74/03e7cf9 | E/split-axis-independent/REVIEW.md SHA6e461305…cfbc46:783 independent tests; root14 Split+10 publication; original clipped-axis32→52; logical math never grants unclipped input |
| Borrowed text painters | 55bcc19,34007dd | E/middle-paint-independent and E/paint-matched/INDEPENDENT-REVIEW.md; root8 matched/middle tests pass; Unicode/provenance/zero allocation; retained4160 plain-paint cases exact |
| Jackin source fixtures | 2bb6d1f/7695e7d | E/jackin-fixture-independent/REVIEW.md SHA3d1594dd…49a8d75; root110 tests:67 lib,27 actual app,6 identity,3 cwd,7 fixtures; not full Manager or editor acceptance |
| Holla Plan and source journeys | Through3d3063f,8246d9e,28ebbbe | E/plan-binding-independent, holla-journeys-independent, holla-a613-independent; exact immutable review binding fixes original replacement bypass; root41 historical tests at28ebbbe include6 latest originals |
| Cross-app smoke | 03e7cf9 | Root263 app library tests:158 Holla,67 Jackin,5 Showcase,33 TablePro; workspace all-target/all-feature MSRV Clippy and stable fmt pass; later28ebbbe test-only addition independently/root verified |

## Current dependency queue

1. **Shared presentation:** tablepro_cli_review owns Columns ca20fdb and clipping
   repair e3c18a1; clipping is independently repaired, but root holds integration
   after nine independently executed theme/scope/instance color and modifier
   override failures caused by post-resolution defaults. Owner repairs at the
   shared style boundary; original failures stay retained. Grid header prefixes, neutral ICON recipe and additive PrimaryKey are
   integrated3092410. Showcase owner continues borrowed fetch/Spinner presentation.
   Runtime owner implements List row_height/row_gap from exact Manager roster.
2. **Actual apps:** baseline_gates owns Jackin app.rs and Manager composition,
   consuming reviewed shared primitives. holla_disposition owns captured Manager
   actions/repository and editor-save payload/token repair; app wiring stays with
   baseline. TablePro owner migrates the actual switcher after Columns acceptance.
   Showcase owner replaces actual Grid page after shared presentation proof.
3. **Holla historical closure:** PG initial focus is integrated2cc4c4c after independent167-test
   review including real-UI stale-target probes; root166 Holla tests pass.
   E/holla-pg-focus-independent/REVIEW.md SHAcfd53c72…320ee7 binds proof;
   scoped MSRV Clippy and workspace stable formatting pass.
   history_audit reviews d80e215 Git/disk assertions. Owner map claims all52 identities, including
   frame_seek outside historical module; identity mapping alone is not semantic
   acceptance. Git policy-skip differs explicitly from unsafe reference behavior.
4. **Native capture:** unintegrated chain47ac27d→7e8a36c→12fb562→55c7dc0→fd62e3a
   →cf080ff→4aa6766 awaits root full review. Earlier ignored-input/build-script
   provenance defects and vendor-cache deadlock remain retained failures.
   Confinement/vendor isolation/recursive duplicate-JSON repairs have scoped
   evidence; actual Cargo tests passed112 at final repair. Author-generated288
   captures (startup112,Holla132,ANSI16-44) at immutable cb7a78a are captured,
   not visually approved. E/app-inventory-phase2/NATIVE-INTEGRATION.md binds chain.
   Linux isolated VM provisioning failed; no Linux adapter execution claimed,
   VM preserved, no permanent impossibility inferred.
5. **Performance:** strict style and Picker gates remain unresolved. Contaminated
   parent/candidate ABBA and fixed-address probes do not establish a causal layout
   effect. E/feedback-clock/style-path/PLACEMENT-CHECKPOINT.md retains raw results.
   Coordinate a bounded quiet measurement using already-built immutable binaries;
   no threshold waiver, cache reset or code change justified by noise.
6. **Final contract:** consolidate normative documents and semantic mappings,
   remove legacy paths only after replacements pass, execute every declared
   toolchain/feature/doc/boundary/parity/capture/performance/mutation gate, inspect
   source-bound images/live journeys independently, then merge and verify exact
   merged source. Full task remains the requirement-by-requirement checklist.

## Checkpoint archive and update rule

The complete423-line plan at d7b772c is preserved byte-for-byte in
[execution-checkpoints-through-d7b772c.md](../audit/main-holla/execution-checkpoints-through-d7b772c.md),
SHA256 ac2f2af7feb504a1484adb022c8c29139335d26cd18b702d03f529101325fa05.
It includes superseded observations, not current approval. Its original relative
links retain the former plan directory; use this plan for current navigation. The earlier352-line
[archive](../audit/main-holla/execution-checkpoints-through-825bbb6.md) retains
SHA256 b99d5700f77b77ae77eeaf774f0766b01ec6eb46623d48cdece588dbf1801855.

Update the relevant task/index/queue entry in place. Keep detailed chronological
evidence outside this compact plan. Preserve original failures and source-bound
reports. Never erase obligations, approve snapshots, or infer completion from
counts. Main remains unmerged; the goal remains active.
