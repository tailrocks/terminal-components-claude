# Termrock migration and acceptance matrix

Updated: 2026-10-20. Status: **in progress; all four applications incomplete**. This is the current migration record. Older audit reports remain evidence, not a complete inventory or acceptance declaration.

## TablePro data 120×40 color levels — 2026-10-20

`tablepro/table/data/120x40` truecolor, 256, and nocolor match the frozen
frames. 256 borders use the baseline palette indexes (subtle 238, strong 242,
seam and scroll track 239, faint 242, thumb 245, accent background 22,
on-accent 232), not nearest-gray of the truecolor RGB. Outer grid edges dim
when the cells are not truecolor RGB. The focused primary cell reverses in
mono. `connections/default` 120×40 truecolor, 256, and nocolor match. The
Connect label is bold reverse with no explicit ink in mono.
Full visual runs and `origin/main` are not done.

## Palette and independently diagnosed reference corrections — 2026-10-20

This increment follows style delivery `e597618d6576a725538a70d71250fd81c6599c28`.
The only production changes are the three Junie palette entries independently
reviewed at `95cdc7bdb111dc1b683a638b421dae718bb29643`: ANSI-256 muted/faint
245/242 and Mono faint `Gray`, all explicitly assigned by frozen `4169783`.
No application painter, captured frame, baseline tag, or test-tool pin changed.

Reference corrections were independently diagnosed and reviewed before merging:

- Empty title isolation: frozen widget execution proves muted RGB 138 rather
  than 128. All ownership and override assertions remain.
- Empty aggregate: immutable historical executions reproduced both old
  fingerprints. Projecting their 32 cases/5,376 cells through frozen palette
  assignments changes only 32 foreground channels; every other cell field is
  identical. Historical values and artifact hashes remain in the audit. This
  proves palette correction plus pre-inheritance compatibility, not that new
  Loading/Partial APIs reproduce frozen application screens or cursor state.
- Nested modal cells: frozen backdrop semantics retain ordinary text and dim
  its style. Full X/R/P/U cell and cursor assertions preserve both paint paths
  and draw orders. Independent execution against the genuine old compositor
  fails the corrected test, as recorded in the audit.
- Item-row fixture: its original hash remains asserted under an explicit
  historical palette. The independently established 174 muted foreground
  corrections produce the expected complete buffer; a wrong-color negative
  control fails. The candidate hash was not copied into the fixture.

Reviewed worker sources are `45b4ca679272b4b3c8948139cd87c1473d652bbf`,
`d3e5647c87aeb33cab03b378fe0f7111718ff3a0`, and
`23a3fac940f24a499b532b0ef580f19355715043`. Their detailed decisions remain
in the corresponding Empty, nested-layer, item-row and palette audits.
The integrated repeat runs 675 tests: 669 pass, the same six pre-existing
Dialog layout tests fail, and three remain skipped. Those failures are not
reported as green. Logs are retained under
`/Users/donbeave/Projects/termrock-cli-evidence-20261020/pr38-palette-reviewed/`.
Full library, application, format, CI and paired acceptance gates remain open.

## Reviewed shared-style delivery — 2026-10-20

The source series reviewed at `2863bf6e7f011c321088263cce12d423b16d73d3`
starts from PR #38 commit `1d7b8eb291e37cb4806ce1369a8a0abab08a81ac`.
Independent review verified all ten changed Rust files are byte-identical
to previously cleared `e72dbb15369d541ee86095f6caaee3f7e4680828`; current
TablePro fixes, Dialog backdrop handling, and focus-bar suppression remain
intact. Explicit theme, overlay and instance style channels reach the shared
components without application repainting. No application, CI, captured frame,
expected hash, or frozen reference is changed by this series.

The same four-package nextest command runs 675 cases after the change versus
662 on the parent: all 13 new regressions pass, two parent failures are fixed,
and the same 14 existing failures remain. The full gate is **not green**.
Those failures comprise six Dialog layout cases and eight theme capability,
empty-style, or nested-backdrop cases. The palette corrections and reference
repairs are separate review candidates, not silently included here. Strict
package Clippy and formatting pass. Detailed source mapping and original logs
are in [the style audit](../audits/2026-10-20-shared-style-regressions.md);
independent evidence is at `/tmp/termrock-style-review-2863`, and the parent
repeat is retained under
`/Users/donbeave/Projects/termrock-cli-evidence-20261020/pr38-style-delivery/`.

Fresh real-CLI evidence on unchanged parent `1d7b8eb` captures and replays all
four selected TablePro checkpoints. Picker-open and tab-list-open match the
candidate exactly, but both Esc-dismissed flows differ in 82 footer cells;
cursor state matches. This limited sample remains incomplete and does not
approve the runtime/Picker integration. See the preserved
`tablepro-picker-1d7/paired-tablepro-picker-result.json` evidence index beneath
the same external evidence root. All four applications remain incomplete.

## Fixed sources and starting tips

| Source | Identity recorded before this work | Use |
| --- | --- | --- |
| `parity/termrock-visual-baseline` | `0b8f4e32b24a137987307600b10d892bec0619c3` (remote observation 2026-10-20 10:52:50 UTC) | PR #38 candidate branch |
| PR #26 fixed source | `f4867e9b5f8a08a3bb61ce80d644198a29f51984` | Product/contract reference; do not substitute a moving branch |
| `visual-baseline` tag object | `2718a6b8127a522a3d351d0b4bdb7542fc8da542` | Frozen oracle; never move |
| Frozen source commit | `4169783ae883788186d161af09ba9c481bd84e5c`, tree `ad0f7cf8899df8c5fea3ced3f60adf0ac9a2d297` | Isolated baseline archive and CLI binaries |
| Tuiscotti | `a1ea27f757d75d07b05499276cd932e9f8e067ad` | Fixed PTY and frame-comparison tool |

Record exact tested source commits in every run. Product refactoring remains scoped to PR #38; PR #26 at the fixed revision above and the frozen baseline remain product references. A later explicit instruction authorized generated workflow upgrades on `main`, `parity/termrock-visual-baseline`, and `design/interaction-polish`; main delivery must use CI-only PR #39, and design changes are limited to generated CI configuration and migration evidence. No application/test-tool refactor or baseline change is authorized there. The current PR #38 runner can test separate builds of each fixed reference without modifying those references. Do not update frozen snapshots to make a candidate green.

## Application migration matrix

| Application | Baseline screen and flow families to preserve | Shared owner required | Current evidence / missing work | Product | Library |
| --- | --- | --- | --- | --- | --- |
| Showcase | Shell/chrome and page navigation; component catalog pages for buttons, inputs/forms, lists, tables, trees, tabs, menus, dialogs/popups, feedback, text/code, layout; each demonstrated state/configuration and keyboard/pointer route | `termrock` widgets, overlays, runtime, text/layout/theme; Showcase supplies demo data and actions | Native root gate at recorded checkpoint: 1,479 passed, 615 skipped. Current external component checkpoint `e7c4be2e5` reports 1,479 root tests, 48 code-editor checks, clippy/fmt; not full workspace or frozen CLI acceptance. Live table/list state and footer ownership remain open. | INCOMPLETE | INCOMPLETE |
| Jackin (`jackin-preview`) | Intro/outro and first-use; menu and capsule; manager/cockpit; workspace, split panes, editor and completion; picker, drawer, settings, accounts, usage, launch, help, save/cancel and resize flows | `termrock` menu, picker, editor/completion, drawer, master-detail/layout, shell and shared input/focus; Jackin retains simulation/domain actions and branded intro/outro/fade | Frozen mono test coordinate correction independently verified; no full candidate journeys. Manager/picker consumer still has full-frame failure; generic shell disappears under open picker unless shared owner layering is fixed. | INCOMPLETE | INCOMPLETE |
| TablePro | Connection/catalogue, create/edit/reconnect; multiple query tabs and tab-list/picker; explorer/workbench; SQL editing/completion; results, error, explain/analyze, history; filter editor, dialogs, help, scroll and resize | `termrock` Picker and tab list, table/grid, editor/completion, dialogs, shell/layout; TablePro owns SQL, connection and query effects | Live Picker/tab-list implementation is missing; a prior SQL editor Tab mismatch remains. Eighteen historical ANSI replay modules remain. The shared sorting component work currently targets Showcase Tasks, not an accepted TablePro conversion. | INCOMPLETE | INCOMPLETE |
| Holla | Home/catalogue and Finder; first-use, monorepo, activities, Docker/disk cleanup, launch failure; parity discovery/history/files/browser/Git/Cargo/task sources; menus, prompts, help, resize and terminal modes | `termrock` shell, menu, picker/search, list, dialog, status, layout/scroll; Holla owns simulation/domain actions and data | Single Holla menu-hover live case passed three phases. Existing Holla native digest and broad CLI captures remain red. `*_plate` model dispatch and application paint helpers are production render paths pending replacement and proof. | INCOMPLETE | INCOMPLETE |

A component-level buffer test, migrated digest, or same-source control does not satisfy product acceptance. A screen that paints through a large private UI crate does not satisfy the library gate.

## Paired test system and coverage

The shared runner is `tools/frozen_baseline.py`; its harness patch and copied helper remain test-only. The test suite drives real frozen and candidate CLIs through the pinned Tuiscotti PTY, captures baseline frames, replays the frozen binary, then compares the candidate. Canonical decoded frames include every cell/style attribute and cursor state. Original expected data is immutable. Each successful case must produce ordered checkpoint receipts in every phase.

- Original frozen inventory: 28,895 ignored scenarios plus 20 ordinary checks. The original stored corpus has 7,675 approvals and 21,220 pending rows. `--all` fails closed on the missing references; a selected case is partial.
- Ordinary checks: 11 source-only, 9 frozen-CLI, 9 candidate-CLI. `isolation::env_isolation` belongs in both real-binary groups. The same guard suite and runner files must exist byte-for-byte on both comparison branches.
- Registered extension inventory: 174 cases. The current pilot exercised 36 cases and emitted 174 ordered receipts in each reported phase. Extension coverage supplements the original inventory; it does not replace or rename original scenarios.
- Latest recorded candidate full-corpus attempt: 240 scenarios red. It is not acceptance. Most recent candidate reports also include a first Holla menu-hover cell mismatch before the shared title-hover correction; the one-case Holla rerun then passed. Re-run exact current branch heads.
- Independent diagnosis confirmed a stale grown-resize approval. The exact frozen tuple and immutable old/new hashes are recorded in `tools/frozen_cli_references.json`; only baseline capture can apply the documented decision. Frozen replay and candidate comparison remain strict. The old approval and original red reports remain preserved. Shrunk resize has no reference exception.
- Required next expansion: meaningful intermediate checkpoints after input, resize and controlled time; adjacent/breakpoint sizes, changed/reordered data, empty/large data, repeated open/close, pointer/pressed states, text edits, supported color modes and wide/combining text. Existing registered checkpoints are not proof of unregistered flows.

Evidence details and limits: [frozen CLI parity](../audits/2026-10-20-frozen-cli-parity.md), [full CLI runner](../audits/2026-10-20-full-cli-runner.md), [live frame support](../audits/2026-10-20-live-frame-support.md). Audit files are superseded only where this matrix says so; underlying red evidence remains active.

## Production replay-path inventory

Audit searched production crates for `historical_*`, `*_plate`, `RUNS`, ANSI includes, frame decoding, raw-buffer writes and direct paint. Findings below are confirmed replay or fixture-selected render families, not just name matches. Ordinary semantic demo data, icons, terminal viewers, and genuine layout breakpoints require individual review and must not be removed by naming heuristic.

| Production path | Observed mechanism and purpose | Replacement owner / status |
| --- | --- | --- |
| `crates/jackin-preview-app/src/app/historical_*.rs` (41 replay files: `historical_accounts_add_form`, `historical_accounts_add_form_required`, `historical_accounts_detail`, `historical_accounts_detail_80`, `historical_accounts_drawer`, `historical_accounts_filter`, `historical_accounts_filter_80`, `historical_audit_accounts_80`, `historical_capsule_menu`, `historical_capsule_menu_80`, `historical_capsule_new_tab`, `historical_capsule_palette`, `historical_capsule_split_vertical`, `historical_capsule_zoom`, `historical_cockpit_cancel`, `historical_cockpit_info`, `historical_editor`, `historical_editor_auth`, `historical_editor_env`, `historical_editor_mounts`, `historical_editor_mounts_dirty`, `historical_editor_roles`, `historical_editor_save`, `historical_hard_cases_80`, `historical_hard_launch_locked`, `historical_hard_launch_picker`, `historical_help_overlay`, `historical_inspect`, `historical_intro_f300`, `historical_launch_picker`, `historical_manager_menu`, `historical_settings_agents`, `historical_settings_env`, `historical_settings_mounts`, `historical_settings_mounts_80`, `historical_settings_route`, `historical_settings_route_80`, `historical_settings_save`, `historical_settings_trust`, `historical_usage_detail`, `historical_usage_overview`) | `RUNS` arrays encode positioned text and colors; paint those cells after scenario/state/viewport selection. These are production captured-frame replays. `historical_paint.rs` is a separate historical style-recipe helper, not a `RUNS` frame; review it as theme ownership, not as proven replay. Likewise Showcase `historical_palette` functions return semantic theme styles; the name alone does not establish frame replay. | Move behavior into shared widgets/blocks and feed Jackin domain data/actions. All 41 replay files remain unresolved unless removed with a paired journey. |
| `crates/jackin-preview-presentation/src/{capsule.rs,launch_running.rs}` | Separate presentation-level positioned `RUNS` renderers; proves replay is not limited to app crate. | Shared component composition; unresolved. |
| `crates/tablepro-ui/src/app.rs` and 8 `crates/tablepro-ui/src/app/historical_*.rs` modules (`historical_form_advanced`, `historical_form_new`, `historical_history_tab`, `historical_query_completion`, `historical_query_results`, `historical_table_scrolled_right`, `historical_workbench_explorer_hidden`, `historical_workbench_maximized`) | Production predicates select fixed scenario/size/state frames; modules parse `RUNS` from `include_str!` ANSI or hold fixed runs; app repaints after normal UI. | Quick-open, tab list, and Safe Mode use `Picker`. Help uses `HelpOverlay`. Delete connection and close-tab confirmation use `Dialog`. The filter editor uses shared overlay chrome plus `TextInput`. A failed query paints the editor, result tab, and error card from the statement error. Explain and explain-analyze paint the editor and the plan strip from the plan tree. Those replay modules are removed. The 8 modules above are still unresolved. `historical_empty_query` is also a production selection predicate and needs source-level replacement review. |
| `crates/showcase-demos/src/pages/chrome.rs` | At source `0b8f4e32`, `MEDIUM_100X30` is explicitly copied from an approved snapshot; `paint_medium_100x30` writes its positioned cell tuples and dispatch selects exact 79×26 page dimensions. Additional narrow/wide render paths need the same purpose audit. | Real shared chrome/menu/workspace components and responsive composition; unresolved. |
| `crates/showcase-demos/src/pages/selectable/{frames.rs,delete_confirm.ansi,bulk_mixed.ansi}` | Imports two approved ANSI frames, decodes cells, resets and writes raw buffer for selectable demo cases. | Real shared selectable/table/list components with inert reference-state data; unresolved. |
| `crates/holla-sim/src/catalogue.rs` plus `crates/holla-ui/src/screens/home.rs` | Scenario and exact viewport/time `*_plate` dispatch selects named fixture view; home helpers paint dedicated fixed families, including parity files/browser and tag lists. Some catalogue functions build semantic domain view data, so review each function before removing it; exact dimension/time selection plus direct plate painters are not accepted as final architecture. | Shared Holla screens/components driven by simulation state and domain view data; unresolved. |
| `crates/termrock-e2e/src/lib.rs` | Test/evidence frame conversion and fixture-generation helpers. Search hit is test support, not by itself production replay; verify dependency/build reachability before any removal. | Test-only harness ownership, never app render path. Audit reachability remains open. |
| `tools/parity_replay.py`, `tools/parity_approve.py`, `tools/test-inventory/historical.json`, `crates/termrock-e2e/**`, `crates/visual-harness/**`, `crates/termrock-conformance/tests/visual_baseline/**`, `crates/showcase/tests/tuiscotti_qualification.rs` | Reference frame management, source inventories, and real-CLI/test harness checks. These are verification paths, not app render paths, and reference data belongs here. Verify dependency edges so no app binary links test-only capture code. | Keep reference artifacts and capture in test system; no production dependency permitted. Dependency/build graph proof pending. |

This is a scoped source inventory from the recorded branch tree, not a proof that *all* helper, presentation, build-script or generated-code paths were exhausted. Inspection of `crates/termrock-e2e/build.rs` found suite-file hashing and `rerun-if-changed` generation only; it does not generate application render code. The earlier 35-site ownership review and 85/88 audit finding counts are findings, not path counts or a complete coverage denominator. Continue repository-wide dependency/build/generated-source tracing. Do not claim replay-free architecture until each production path above has a replacement and the full audit records zero remaining paths.

## Architecture and independent validation

| Gate | Evidence | Status |
| --- | --- | --- |
| Independent production crates | Prior exact checkpoint: 19 production packages, 1,313 passed, 4 skipped; package clippy green | Partial; rerun at current integrated commit |
| Root default members | 1,479 passed, 615 skipped at prior checkpoint | Partial; six members only |
| Full workspace | 3,795 tests ran, 3,760 passed, 35 failed; workspace clippy failed at `crates/tablepro-ui/src/app.rs:9284` (`expect_used`) | RED; failures recorded in [independent validation](../audits/2026-10-20-independent-workspace-validation.md) |
| Code editor completion | 48 focused tests green at `e7c4be2e5` per independent checkpoint | Component only; consumer and outside-click ownership remain open |
| Master-detail | Shared geometry/state API and 374 tests, 3 skipped at recorded checkpoint; does not include live Manager composition | Component only; app use and full frame evidence absent |
| External public consumer | Not yet demonstrated for proposed composition API with unrelated domain data and no private app helpers | RED / missing |
| Ownership/replay audit | Existing exception ratchet and 35-site report; 85/88 prior audit findings are not a complete path inventory | RED / partial |

The new `MasterDetail` owns layout geometry/state. Frozen narrow detail replaces body content; it is not a floating modal. Live Manager composition and actual dialog/picker layering remain separate work. The completion adapter owns editor/popup lifecycle; outside clicks currently dismiss without forwarding to owner. Picker policy/title-style and MenuBar changes with native fixture conflicts are deferred WIP at the cited checkpoints; do not infer parity from tests that only assert a proposed style.

## Decisions and next evidence

1. Keep frozen source/tag/Tuiscotti IDs fixed. Record commit, tree, binary hashes, environment, size, mode, scenario inventory, receipts and result for every candidate run.
2. Use the same PR #38 runner, test helper and manifests when testing each fixed reference and candidate. Earlier byte-identical dual-branch publication is historical; the latest instruction prohibits further PR #26 changes. A test inventory mismatch still blocks comparison.
3. Keep each app INCOMPLETE until original plus extension inventory and all required ordinary checks pass on the exact same candidate commit, with no skipped or missing case and a reviewed reference-data decision.
4. Keep library gate separate: remove replay from production, prove public shared APIs own mechanics, compile an external consumer, and complete source/dependency/build/generated-code review.
5. Resolve red behavior in shared components or domain model; retain baseline flows. Never select a renderer by test label, exact viewport/clock, terminal detector or captured accent. Never repaint a widget in app code or register absent controls to move focus/cursor.
6. Continue in order: TablePro Picker/tab-list real consumer and replay removal; Showcase real button/catalogue paths; Jackin semantic shell/workspace/picker/editor; Holla semantic catalogue/layout. Expand shared blocks only for demonstrated consumers.

No application has full product or library acceptance. This matrix becomes current at its merge commit; append exact commits and evidence without relabeling partial checks green.

## Latest bounded evidence

- Exact PR checkpoint `e7c4be2e52bc9877ba5edc9930235cceab19a336`: independently built candidate; Holla File-menu hover and four stored application controls passed all three phases with zero checkpoint differences. Evidence: `/Users/donbeave/Projects/termrock-current-cli-proof-20261020/live-paired/1792493484804932000-1666732d0b3c/report.json` and `/Users/donbeave/Projects/termrock-current-cli-proof-20261020/live-paired/1792493572222308000-16335f5b43ea/report.json`. Five selected checkpoints are not full parity.
- Showcase Tasks WIP `fa02fa54a6151c43cccd56d61e096f4a87808f13`: seven focused consumer checks passed. Fresh real CLI comparison of six sorting workflows at 120×40 truecolor failed at every initial checkpoint, with 1,952 differing cells each, including cursor shape. Frozen capture and replay matched 24 receipts each. Evidence: `/Users/donbeave/Projects/termrock-live-cli-table-consumer-20261020/live-paired/1792493587115197000-c06708be5d1e/report.json`. Earlier consumer evidence is superseded for this WIP. It is not accepted.
- Shared extension manifest hash `b995c00f0dc14d4e0a81d1d56140e1c10ba7936a043e0135196f8fe8421cabb8`: all 174 registered sequences are checked exactly. Thirty incorrect 100×30 checkpoint lists were repaired before integration; the omitted-middle guard is mandatory. Twenty-three guards pass; frozen capture/replay for 36 cases matched 174 receipts each. Remaining registered scenarios and required workflow obligations remain unverified.
- Whole-repository formatting at source `0b8f4e32` is red in the pre-existing Chrome fixed-cell constants. The new test module passes its own formatting check; root nextest and clippy passed. No formatting or architecture failure is relabeled successful.
- CI planning was independently observed failing before Rust jobs ran because standalone helper discovery failed. Skipped Rust jobs are blocked evidence, not passing validation. The reviewed discovery correction and generated symlink correction were pushed to PR commit `2de0d01255390d2a02b7775ee452268782802a38`, with remote SHA verified. Remote run `39415797723` passed Actionlint but failed Check generated files in Plan; downstream Rust jobs were skipped and provide no passing validation. DCO remained action-required. The local preview success does not supersede this remote failure; diagnosis is active.

## Current integration obligations

The staged contract work above is not all present in the candidate branch. Source identities in each evidence entry remain authoritative.

- CI discovery correction `445c7e9af00bdbc5f9a2de86172b3e91b31eab74` received independent generated-tree and plan review. It discovers 47 packages and 44 crate jobs and adds five missing visual-harness tasks. It does not establish CI acceptance: 39 generated doctest commands conflict with nextest-only policy; no generated task enables the `testing` feature; standalone helper tests and paired-runner tests lack a normal generated execution path. The generated `.github/CLAUDE.md` symlink is corrected through `[agents] claude_md = "symlink"` in source configuration and regeneration (`8beaf2865b922fbea0ae082f914367d1fed64609`). The remaining execution gaps need supported generator corrections, not hand edits. See [CI planning audit](../audits/2026-10-20-ci-planning.md).
- Independent reference review confirmed the grown resize approval is stale: two real frozen captures and overview controls have identical 4,800 cells and cursor, ANSI SHA `f2e07ed284673b1d543441767460193e987034e163ca7ea7d3d4496473a0d442`, full frame JSON SHA `01e9f7d349fc8075df7b64188db97a05705792aac8c781184db14a6d1ecc253a`. Legacy grown/shrunk SHA `7769a6686dfc5cb5c69facb03f54a0fa57e963b35fcf315b1c4625745e2b7708` predates the October Components card. Preserve both artifacts. The documented decision authorizes only the independently reviewed grown tuple in paired capture, with exact hashes and an explicit legacy-red verdict; the exact capture-only decision is implemented and the two-case frozen capture/replay pilot passes, and the deliberate hidden-cursor-only negative control has been verified. Shrunk120×40 separately fails its no-change wait before any checkpoint and has no approved reference exception. Evidence: `/Users/donbeave/Projects/termrock-reference-review-20261020`.
- Test integrity: canonical ANSI does not retain hidden cursor position, shape, or blink. The reviewed helper adds full semantic frame equality, mandatory semantic receipts, and preserved actual-frame diagnostics. A fresh two-chunk pilot matched frozen capture/replay and rejected the candidate overview for hidden cursor `Block` versus frozen `Bar`; its second candidate case was not run. The suite was published identically on both comparison branches, and a deliberately mutated hidden cursor was rejected with zero cell differences. Earlier ANSI-only pilots cannot establish complete cursor-state parity.
- TablePro priority: replace both tab-list and quick-switcher replay paths, phantom editor/cursor registrations and fixture predicates with the real Picker. Shared field editing, presentation preservation, content slots, menu/clipboard/selection contracts and unrelated-data consumer proof precede consumer acceptance. Do not restore a captured background when live workbench defects become visible.

- Shared contract review remains active: field command execution, selection invalidation, scrolled drag coordinates, secret clipboard temporaries, and duplicated command mechanics require fixes before the field API can be accepted. The strict architecture scanner is also unaccepted: referenced modules behind Cargo-target aliases and missing live includes exposed false-green source accounting. These findings do not authorize production replay paths.

- Same-source paired-runner positive control: a clean, separately built frozen-source candidate passed Holla Files and Showcase Overview at 120×40 truecolor, with chunk size one. All three phases emitted both ordered checkpoints and matched complete semantic frames. Evidence: `/Users/donbeave/Projects/termrock-live-self-control-20261020/live-paired/1792496449799955000-02f47d78355a`. This validates the bounded harness path, not the refactored applications.

- Shared test-system integration checks: 28 Python guards and 25 standalone semantic-helper nextest tests pass, with helper clippy and formatting and extension formatting clean. The optional underline-color and `Default`/`Indexed`/`Rgb` variants match the pinned serialization schema. Exact per-phase inventory unions remain mandatory across multiple chunks. These checks do not accept application parity; deliberate candidate-defect verification is recorded below.

- Deliberate candidate defect: isolated frozen source changed only the default caret from `Bar` to `Block`; the runner recorded its dirty source fingerprint and separate candidate binary hashes. Frozen capture/replay passed and candidate comparison failed solely on hidden cursor shape, with zero differing cells. Evidence: `/Users/donbeave/Projects/termrock-live-hidden-cursor-proof-20261020/live-paired/1792496903298882000-d4f64b7e5c12`. The immutable baseline and approvals were unchanged.
- Shared publication checkpoint: PR `0168ccc15246343e6a1f846a30bf7964991b439c` and design `46ff0ebbf457ed46da301706229d941ee985a102` were pushed and remote IDs verified. Thirteen shared test/helper/manifest/documentation files were byte-identical; the identity manifest is `/Users/donbeave/Projects/termrock-cli-evidence-20261020/shared-semantic-branch-identity.json`. These remain partial test-system checks, not application acceptance.

## Latest instruction checkpoint and delivery boundary

Fixed product references remain PR #26 source `f4867e9b5f8a08a3bb61ce80d644198a29f51984`, baseline source `4169783ae883788186d161af09ba9c481bd84e5c`, tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`, and Tuiscotti `a1ea27f757d75d07b05499276cd932e9f8e067ad`. No ref changed.

Before the workflow-only upgrade, recorded tips were `main` `a97e351df736b576904bc450619ca347ea64d65b`, remote PR #38 `1a947a251ab9b045610b47729bf481419b9fca8d`, and observed design branch `8681124d2cbbc75ef8ed998b5918883039a022c5`. The shared PR #38 remote later advanced to `a848219fd858ed6352fcb403b2cc5d5953083e6f`. Worker integration commit `90cd26402ffb421fd13db4418a8f9201c3a31d5e` was local only and is not a recorded remote PR #38 tip. The design comparison branch at `8681124d2cbbc75ef8ed998b5918883039a022c5` is an observed branch tip, not the fixed PR #26 reference; fixed PR #26 source remains `f4867e9b5f8a08a3bb61ce80d644198a29f51984`. Design workflow author tip `8533344520cfdf12734c1c149fc89ec946b3ea46` is based on `8681124d2cbbc75ef8ed998b5918883039a022c5`.

Latest workflow instruction explicitly authorizes updating all three branches with Velnor Actions generated files. Main upgrade is isolated in CI-only PR #39; do not push directly to `main`. Parity generated-file regeneration and audit are worker changes for integration into PR #38. Design upgrade is limited to workflow source/config, generated files, and migration evidence. Do not add application or test-tool changes to design, retarget or rewrite PR #26, or change frozen references. This scoped workflow authorization supersedes the earlier blanket ban on additional design/main workflow updates only.

The parity and design upgrades use stable Velnor Actions `0.1.29` at source commit `e7b2f088715379c22be473fab896a43d50cc16fc`. Independent complete-input parity generation found the previously committed manifest and CI workflow used nextest profile `default` in 88 commands while tracked `.config/nextest.toml` configures `ci`. Regeneration changed only `.github/velnor/task-execution-manifest-v1.json` and `.github/workflows/ci.yml`; a fresh preview matched all 21 `.github` entries by path, kind, bytes, mode, and symlink target. Plan: 47 workspace crates, 44 Rust execution jobs, 259 task IDs. Design verification at author tip `8533344520cfdf12734c1c149fc89ec946b3ea46` independently confirmed all 22 `.github` entries against a fresh full-input preview; its plan selected the configured `ci` profile and found two workspaces and two jobs. Both results establish generator consistency, not successful remote CI. Existing blockers remain: generated doctest tasks invoke `cargo test --doc` contrary to the nextest-only instruction; DCO is action-required in observed remote runs; full product parity and full coverage remain incomplete. Details: [Velnor parity regeneration](../audits/2026-10-20-velnor-0.1.29-parity.md), [CI planning audit](../audits/2026-10-20-ci-planning.md).

All product changes remain on `parity/termrock-visual-baseline`; workflow-only PR #39 is not a replacement for PR #38. PR #38 must reach acceptance and required repository review before merge. Full application comparison must pass twice on the same integrated commit; no complete passing run has been established.

The newly requested replay-origin audit starts from `6923311119e060176ee3cf07c585416556416024`, `f067d3f088621ec2a0fe4e7eaf7ae2c3a7c5957c`, `fd3991049709f08c8434d567c45e1b55583a5a4f`, and `51046a780fafe61bf10ed2dc9d2df4c1eeffa734`. Focused independent source tracing is assigned; presence of these commits is verified, but that audit is not yet complete. Any conflict between fixed reference behavior requires recorded evidence and independent review rather than an implicit reference substitution.

Runtime viewport-margin implementation `f3e06913376d3820acf5a9597597aec91f2a576b` passed 162 runtime tests and independent exact-source review. Picker integration remains pending. Edit-menu focus continuation is still a design gate: opening tokens must prevent stale delivery, and focus restoration/command execution must settle before output without an extra visible frame or handlers during pure rendering. No stage2 implementation or full edit-menu parity is accepted.

The bounded architecture diagnostic from independently reviewed source `70b7f9e6c196b57d1b13aee56ac6d6a17e6db388` is integrated without changing its four files. Its 39 regression guards pass. This supersedes earlier scanner-review findings only for those repaired lexical accounting defects; semantic ownership, production resources/generated inputs, external-consumer proof, and full product checks remain incomplete. The tool always reports `acceptance_complete = false`; a clean lexical scan cannot accept the migration. See [architecture diagnostic and limits](../audits/2026-10-20-acceptance-architecture.md).

Integration check against product source at `99b719d85` reports 84 lexical findings and five unresolved items, with zero reported read errors or unaccounted inputs. The diagnostic exits red and keeps overall acceptance false. The report is `/Users/donbeave/Projects/termrock-cli-evidence-20261020/architecture-diagnostic-integration/source-report.json`; 39 diagnostic guards and 28 paired-runner guards pass. This does not resolve any production finding.

The independently corrected [replay-origin audit](../audits/2026-10-20-replay-origin-current-source.md) covers exact product source `a848219fd858ed6352fcb403b2cc5d5953083e6f`; later Safe Mode change `c0e88b7ad` is outside that audit. It confirms remaining replay mechanisms in all four application trees. Subsequent independent real-CLI comparisons of `a848219` confirm exact Quick Open-open and tab-list-open checkpoints at 120×40, but Escape dismissal, tab activation, and all five tested 100×30 cases fail. WIP `366dc67` also fails both sizes. Complete source/binary identities and preserved receipts are in [the paired-run audit](../audits/2026-10-20-full-cli-runner.md). No full application is accepted.

Workflow delivery checkpoint: parity remote `99b719d85a0714c88484240dbc9184f110b518d4` and design remote `9b1c77b89e46b89f56632c841f1ade40371f03dc` were verified after normal fetch/rebase, complete-input generated-file comparison, plan, and actionlint checks. Main delivery remains CI-only PR #39 at `d1ecbfdfbe117ffad061f6db896c0a3c98b3c11d`; DCO remains blocked and required checks on that head are not established. These workflow checks do not attest Rust or product parity. The parity checkout still fails full formatting on the pre-existing Showcase Chrome cell table; no formatting or baseline artifacts were changed to hide it.

## Current CI timeout checkpoint (supersedes the workflow delivery status above)

Parity source `a5766d24783c2b268e727497fe6861b6479c71cc` uses Velnor Actions 0.1.29 (`e7b2f088715379c22be473fab896a43d50cc16fc`). Its latest observed run `39429669130` had Actionlint pass, Plan canceled after 5m09s, and all Rust jobs skipped. The run's `cancelled` conclusion does not prove GitHub's job timeout caused cancellation; downstream missing artifact/Required failures are not source failures. A distinct earlier Plan failure at `a848219fd858ed6352fcb403b2cc5d5953083e6f` was the 0.1.16 generated-file mismatch recorded in [the timeout audit](../audits/2026-10-20-velnor-timeout-parity.md).

Parity now explicitly configures `workflow.crate_timeout_minutes = 15`, matching the reviewed main-delivery upgrade config and raising all 44 generated Rust-job limits from five to fifteen minutes. Velnor 0.1.29 source confirms that field affects Rust capacity-phase and OpenTofu root jobs only; Plan remains a built-in five-minute control job with no override. This mitigates Rust-job budget risk but does not fix or explain the observed Plan cancellations. Fresh generation preview, plan (47 packages, 44 Rust jobs, 259 task IDs), and the CI-equivalent Actionlint command pass. Remote Rust validation remains pending; no skipped job is counted as a pass. DCO is not a protected required check. See [the timeout audit](../audits/2026-10-20-velnor-timeout-parity.md).


CI-only delivery review: source `4d0df7704dcb284d0c9726e78ea68bcca30838fc` independently cleared with full generated-tree comparison, unchanged task manifest, plan, and Actionlint. Main PR #39 remains blocked after its single rerun confirmed the fixed five-minute Plan timeout; all 43 Rust jobs were skipped. Design `59af1c4f760387be455bc79d44be75f4416a63f3` passed Plan/Actionlint but has a pre-existing Rust formatting failure. These statuses supersede earlier pending-status notes and any suggestion that optional DCO was the protected-check blocker. Details and exact job identities: [parity timeout audit](../audits/2026-10-20-velnor-timeout-parity.md). All four application acceptance statuses remain incomplete.

## Latest remote CI status (2026-10-20; supersedes the preceding status paragraph)

- Parity PR #38 remote head is `39aaf8d5e1dab5cf1c613c0bb7e76638d89d57af` and contains timeout source `4d0df7704dcb284d0c9726e78ea68bcca30838fc` (`c52aafc2f266842c1743151dd379c3a38ca990bc` is an ancestor). It uses Velnor 0.1.29 and a 15-minute Rust job timeout. Latest run [39438100451](https://github.com/tailrocks/terminal-components-claude/actions/runs/39438100451) was still in progress at observation; no required green result is established.
- Design remote head is `f2f6eb716c4a512be3dc9423742fa06e86879b32`; its CI-only changes use Velnor 0.1.29, 15-minute Rust jobs, and zizmor 1.31.1. Latest run [39438273472](https://github.com/tailrocks/terminal-components-claude/actions/runs/39438273472) was still in progress (Actionlint passed; Plan remained in progress). The preceding run [39435382085](https://github.com/tailrocks/terminal-components-claude/actions/runs/39435382085) passed Plan and Actionlint but failed Rust formatting in `Rust / xtask`; `Rust / junie-tui` was cancelled. Therefore design has no complete green required run.
- Main remains at `a97e351df736b576904bc450619ca347ea64d65b`; CI upgrade PR #39 is still separate at `a6929aeba7479d0b3a746f69d412a1ab701b5242` and has not been bypassed or merged. Latest run [39436987238](https://github.com/tailrocks/terminal-components-claude/actions/runs/39436987238) had Plan cancelled, Actionlint successful, all 43 Rust jobs skipped, and the required verdict failed. The same five-minute Plan cancellation is confirmed by three runs; no supported config override for Plan is recorded. These are workflow blockers, not successful Rust validation.
- The fixed baseline remains tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`, source `4169783ae883788186d161af09ba9c481bd84e5c`; tuiscotti remains pinned to `a1ea27f757d75d07b05499276cd932e9f8e067ad`. No baseline or test-tool reference moved. The design-range diff is CI configuration/generated workflow and audit documentation only; it contains no product, test, or reference-data edits.
- The earlier optional-DCO note is superseded: DCO is not the protected required blocker on these runs. The former “pending” language is superseded by the concrete run states above. Showcase, Jackin, TablePro, and Holla remain incomplete for product parity and library architecture acceptance; these CI results do not establish either gate.

## Branch consolidation (2026-10-20)

Delivery stays on `parity/termrock-visual-baseline` (PR #38). `design/interaction-polish` (PR #26) is reference-only: not merged, not modified, not closed. Its pinned review commit remains `f4867e9b5f8a08a3bb61ce80d644198a29f51984`. The remote branch tip is `f2f6eb716c4a512be3dc9423742fa06e86879b32` from other CI work; this record does not move it. `origin/main` remains `a97e351df736b576904bc450619ca347ea64d65b`. Tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542` and source `4169783ae883788186d161af09ba9c481bd84e5c` are unchanged.

Integrated: `be674ef8799c55bc3700cdd36f519cafebc95bed` from `parity/composition-slots-20261020` as `0cd5edb8856396bde1bbd2f8a5377940a3d50447`. It adds the typed picker layer slot. `termrock::slots` passed. Docs conflict kept the frozen-cli note and the new slot section.

Also integrated after cell checks: layout plans `8f7fc45a0` and `a39869753` (`termrock-layout` consumer test passed); session cursor `5e4eb27ec` and `f0360a467` (TablePro explain and explain-analyze 120×40 truecolor stayed at 0 cells); Junie ANSI-256 palette `737ecc836` and `95cdc7bdb` (79 `termrock-theme` lib tests passed); legacy item-row fixture `23a3fac94` (13 `item_row_columns` tests passed). Remote tip after that series is recorded in git, not here, until the push. Full four-app `VISUAL_FULL` runs are still not done. Do not merge to `main` before those two identical runs.

Already patch-equivalent on PR #38 (`git cherry` minus): `db31adf1e` session startup, `cf1ff9153` completion splicing.

Excluded, not merged:

- Design-history branches (`wip/shared-design-working-tree-20261020`, `wip/reach-pages-a`, `wip/reach-pages-a-2`, `wip/rapps-3-7-14`, `wip/s14-followups`, `wip/plan-s16`, `wip/v1-showcase`, `docs/interaction-reference-cleanup`, `ci/design-rust-timeout-20261020`, `ci/zizmor-design-20261020`, `parity/velnor-design-upgrade-20261020`). Each is 700+ commits ahead of a pre-termrock base. A merge would restore the monolith.
- `wip/termrock-reviewed-contract-integration-20261020` and `chore/rust-comment-cleanup`. Open parity or a 492-file comment strip, not a behavior fix.
- Preservation and rescue archives (`preservation/*`, `codex/rescue-*`, `codex/termrock-*`). They keep old snapshots and local history. They are not the delivery tree.

Not yet integrated. `git cherry` marks these `+` against `6bcdc4217`. They stay on their branches until a focused visual check shows they do not move baseline cells:

- `1ac6dfced` tablepro footer focus
- `7dab012ee` shared button state
- `94a1215e2` shared style regressions
- `d4832fe73`, `c1335e19c`, `f8e08fc29`, `56a8547a4` picker query editing
- `7e38a850a`, `bcb51e8c4`, `4ab81ae47` tablepro live picker contracts
- `17ad7f673`, `5b6a23121` session cursor policy
- dialog dismissal series `23f911902` through `1be6567ae`
- `8e9a7f59e`, `737ecc836`, `95cdc7bdb` theme palette
- `0638339b7`, `28cc24880`, `6b72d77a6` picker geometry

No other branch was deleted or closed. Product parity for showcase, jackin, tablepro, and holla remains incomplete.
## Latest theme reference-test diagnosis (2026-10-20)

- The isolated shared-theme palette branch `parity/theme-palette-baseline-20261020` is remotely verified at `95cdc7bdb111dc1b683a638b421dae718bb29643`. Its capability palette suite passes 9/9 after Junie ANSI-256 and Mono semantic corrections; see [palette audit](../audits/2026-10-20-junie-ansi256-palette.md).
- A separate test-reference correction branch updates the stale empty-title owner expectation from `Rgb(128, 128, 128)` to frozen `Rgb(138, 138, 138)`, then updates the aggregate hash only after independent projection of frozen palette rules across all 32 complete frames (5,376 cells). The old expected hash, original source hash, independent source revisions, projection artifacts, and limits are preserved in the [empty title reference audit](../audits/2026-10-20-empty-title-test-reference.md). Full-cell comparisons remain intact. This is a test-reference repair, not frozen Empty UX parity proof.
- The test-reference branch is not integrated into PR #38. This focused file result does not establish full application acceptance; remaining architecture, behavior, and paired application verification are unresolved.


Expanded independent-library check for palette cohort `636028fa130b2a0f841badb3ba93aa2330b907d6`: both nextest hash partitions across 19 Termrock crates ran 1,361 tests, with 1,352 passing and nine failing. Six failures are the existing Dialog layout assertions; the meter semantic-token, Note wrapping, and full-track scrollbar failures were independently reproduced on base `e597618d6576a725538a70d71250fd81c6599c28`. Partition exclusions are not additional skipped cases. Logs are under `/Users/donbeave/Projects/termrock-cli-evidence-20261020/pr38-palette-reviewed/library-partition-{1,2}.log`; base attribution is in `branch-consolidation/palette-failure-base-e597.log`. This is not a passing full-library gate or application parity proof.

## Runtime/query branch integration checkpoint

Reviewed runtime/query source `7c1a092ab7cfa37002f06e505cd4577f133750f4` was integrated with PR #38 source `072fc24d6dd4f53dbd544485174e39aff2c92f43` on the existing branch `parity/runtime-picker-allocation-20261020`. Merge base was `6bcdc42173232b53ed48b221ae8d099791283e39`. The PR source brings the reviewed composition plan, session caret-capability policy, branch ledger, and latest palette cohort. The `crates/termrock-session/src/session.rs` conflict was resolved to retain the runtime branch's prepared-frame prepare/draw/acknowledge loop, resize retry, and success-aware cursor-shape cleanup, together with frozen capability selection and hidden-cursor shape restoration. Its tests cover shape-write failure and flush failure. No application source, test approval, binary, tag, or tuiscotti pin changed in this runtime stack.

Exact merged-candidate checks: affected library nextest ran 434 tests, 428 passed, six existing Dialog layout assertions failed, and zero were skipped (`/Users/donbeave/Projects/termrock-cli-evidence-20261020/branch-consolidation/A-runtime-merged-native.log`). The independent public Picker query probe passed 16/16 (`A-runtime-merged-query.log`). The warmed 100k borrowed Picker rendered 38 visible items with zero allocations and bytes; `PERF_STRICT=1` passed the 1.5 ratio limit at 1.13 (`A-runtime-merged-perf.log`). Strict Clippy, scoped formatting, and `git diff --check` passed (`A-runtime-merged-clippy.log`, `A-runtime-merged-fmt.log`, `A-runtime-merged-diffcheck.log`). These results do not include real-CLI paired checks. Dialog dismissal/reopen remains blocked by the independent P1 generation-receipt finding; no dismissal-contract commits are included. Product parity and library acceptance remain incomplete.


## Reviewed runtime and terminal-profile consolidation (2026-10-20)

Runtime/query source `ce81e0adee987812788effd39d70acd242b0e95b` preserves the prepared Session loop, successful terminal-write cleanup, public query editing, stable row identity, and typed layer slots. Independent merge review passed 17 Session tests and 16 public query probes. Parent checks across 19 independent Termrock crates ran 1,453 tests: 1,445 passed and eight failed. The remaining failures are the six known Dialog layout assertions, meter semantic tokens, and full-track scrollbar expectation; the previously failing Note expectation is corrected. Four ignored subprocess-driver fixtures are listed separately in the test inventory; partition exclusions are not passing cases. Strict affected-package Clippy passed. The 100k Picker allocates zero bytes, but strict timing exceeded its 1.5 threshold under concurrent load and remains unproven. Logs: `/Users/donbeave/Projects/termrock-cli-evidence-20261020/branch-consolidation/parent-runtime-library-partition-{1,2}.log`, `parent-runtime-clippy.log`, and `parent-runtime-ignored-tests.json`.

Terminal-profile source `8207fbd009a3ced805e07ac734572cbd65078de7` was independently reviewed with 31 passing Python guard tests and a successful patch dry run against the frozen source. It propagates the same explicit TERM setting, including absence, through the actual Tuiscotti child environment for both phases. Frozen identities, immutable expected-data checks, ordered receipts, complete case accounting, and candidate isolation remain enforced. Live profile and application comparisons remain blocked by the unavailable approved baseline binaries; no replacement build identity is admitted. These integrations do not complete any application or library acceptance gate.


## Baseline rebuild diagnostic: not an admitted reference build (2026-10-20)

An independent diagnostic compared rebuilt, unadmitted frozen-source binaries with sealed historical frame copies through pinned Tuiscotti `baseline-check`. Four Showcase and ten TablePro checkpoints matched (14/14); the indexed eligible contexts contain no Jackin or Holla frames. Historical receipts preserve frame digests and approved-build metadata but do not bind the per-run baseline executable path and hash. Therefore executable identity remains unverified and the rebuilt binaries remain unadmitted; this result does not clear the live product gate. No source, frozen pin, approval, expected frame, or identity guard was changed. Evidence: `/Users/donbeave/Projects/termrock-cli-evidence-20261020/baseline-rebuild-diagnostic-expanded-20261020/diagnostic-report.json` (SHA-256 `14c38a03f58eb3a9694cd6e23c7830bc9eb45f6fc119c6d16268d06572ba109d`) and `receipt-audit.json` (SHA-256 `a8e16150643b8121ce75a126f3fdf8f4e478853b3b1e14249994a15cbeb8a648`).
