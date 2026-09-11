# Showcase adversarial re-audit — tasks 032–039

Audit date: 2026-09-11. Planning only. Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural main `7b27732a8c3c131760ec3438f641cb3c11343a42`. Production source unchanged. Findings below describe the pre-repair transcripts, not new oracle behavior. Author repairs require independent re-review; this report does not close its own findings.

## Method and coverage

Read PLANNING_GOAL.md; canonical task/v5 README, verify/v2, task-meta/v1 implementation and AGENTS instructions; every task032–039 file in full. Verified source with immutable `git show`, not current candidate behavior. Read all23 oracle page implementations and shell/main/page dispatch; investigated shared input/dialog/table/scrollbar handlers behind each disputed transition. Compared task scopes/dependencies with main's controlled-component migration and remaining app-local compatibility paths. The verification skill keeps work planning-only; Rust testing policy applies only to disposable oracle diagnostics.

Default coverage remains all23 routes × four sizes × four palettes =368 complete initial frames, plus every stateful checkpoint and crosscut expansion. Initial frame matching alone cannot establish edit commits, valid retries, selected identity, overlay ownership, or absent geometry.

## Findings and original transcript provenance

### S01 — contradictory shell-only scope

`refactoring-tasks/terminal-components/completion/032/README.md:35,48` allows initial public page compositions although Goal:12 and trusted obligations:11 prohibit non-shell page repair. Original sentence: “Initial public page compositions required by owned shell frames are in scope, but later page-specific actions retain their named owners.” Tasks033–038 are the page owners; task032 contribution frames deliberately isolate shell except the complete undersize frame. Loophole: repair page-specific rendering early or import historical compositions under shell authorization. Root: scope text predates staged contribution split. Repair: state shell-only/minimal public Diff route consistently; retain downstream full-page owners.

### S02 — invalid commit ends editing; retries type into navigation mode

`docs/refactoring-plan/showcase-scenarios.tsv:38,51,74` (SC-INPUT-EMAIL-MASK, SC-DIALOG-PROMPT, SC-SETTINGS-ENV). Original subsequences respectively `Ctrl+U;Enter;type(mira@example.test);Enter`, `Ctrl+U;Enter;type(x repeated41);Enter;Ctrl+U;type(Review schema);Enter`, `type(lower-case);Enter;Ctrl+U;type(API_BASE_URL);Enter`. Oracle `src/widgets/input.rs:169–172,198–205`: commit sets editing=false before validation, printable keys outside editing are ignored; Enter restarts. Oracle `src/widgets/dialog.rs` keeps invalid modal open without restarting input editing. Thus email never valid, rename never tests max40, env modal never appends. Static scenario ownership/ID equality still passes. Root: intended outcome substituted for real input state machine. Repair: explicit Enter re-entry after each invalid commit; assert error and editing transition before valid retry. Preserve Required, invalid, valid, max40 and cancel branches, not simply remove invalid input.

### S03 — target controls do not exist at required narrow geometry

`showcase-scenarios.tsv:39,48` SC-TEXTAREA-EDIT and SC-PANEL-SCROLL originally require Commit message/nested list focus at80x24. Oracle `app.rs:841–879` page height18; `pages/textareas.rs:62–79` fixed rows13+1 leaves4 for second card; `src/widgets/textarea.rs:240–250` yields zero text rows and no registration. `pages/panels.rs:71–115` leaves nested card3 rows and no inner focus target. Loophole: materializer either aborts on immutable oracle or silently invents targeting/reflow. Root: uniform size product ignores source layout reachability. Repair: keep80x24 clipping/no-hit/no-focus checkpoints, and execute full omitted interaction at source-viable100x30/120x40/160x50. No app-local reflow or new scroll affordance authorized.

### S04 — wrong default grid modal action

`showcase-scenarios.tsv:65` SC-GRID-QUEUE original `p;Enter` claims Copy SQL status. Oracle `pages/grid.rs:225–244` opens facts without acknowledgment; `src/widgets/dialog.rs:130–134` initially focuses Cancel. Enter cancels. Root: generic primary-action assumption ignores dialog kind. Repair: retain Cancel branch, reopen pending SQL, Right;Enter for Copy SQL; assert status payload and pending data unchanged.

### S05 — intended Task edit actually selects Owner and fails

`showcase-scenarios.tsv:46` SC-EDITABLE-CELL original `focus(Tasks);Right;Enter;End;type( ok);Enter`. Oracle `pages/editable.rs:62` initializes cursor_col1=Task; Right selects Owner, whose validator rejects spaces at:17. No successful commit; later cancel/error journey begins from wrong state. Root: assumed initial cursor. Repair: explicit Task column initial-state assertion, no initial Right for task success; retain separate Owner invalid-space, numeric, branch and cancel cases.

### S06 — sidebar click changes focus; prescribed Expand target disappears

`showcase-scenarios.tsv:49` SC-SIDEBAR-MODES original `click Collapse;Down;Enter;click Expand`. Oracle `app.rs` mouse-Up fallback focuses clicked button; `pages/sidebars.rs:453` does not override it. Down ignored; Enter re-expands. Next Expand target absent. Collapsed visual button label is `›`, not literal “Expand”. Root: semantic helper hides missing focus transition. Repair: explicit focus(demo navigation) after collapse, bind expand action to frozen oracle control identity and actual glyph.

### S07 — required scrollbar absent when contents fit

`showcase-scenarios.tsv:28,49` SC-SHELL-HITS / SC-SIDEBAR-MODES blanket thumb drags include nonoverflow frames. Oracle `pages/sidebars.rs:173,372–407`: demo13 rows; viewport14 at80x24 and16 at120x40; no scrollbar. Oracle `app.rs:958–1056`: shell sidebar fits at120x40/160x50. Root: control inventory does not distinguish source-present/absent targets per geometry. Repair: source-derived explicit no-scrollbar/no-hit branch at fit sizes; retain actual72x20 demo overflow and shell80x24 overflow drags; compact100x30/31 and expanded100x32 fit exactly or with spare rows. Registration evidence, not glyph guessing, chooses branch.

### S08 — route persistence transcript quits instead of testing editing shield

`showcase-scenarios.tsv:33` SC-SHELL-PERSIST original `...Enter;];[;Enter;type(q[]?i0);Esc`. Oracle `app.rs:314–328` clears content focus on route change; render repairs it to NAV; Enter remains navigation, q quits. Root: assumed focus persistence conflated with data persistence. Repair: assert committed value survives, assert navigation focus after roundtrip, explicitly focus Project name and Enter before typing reserved globals; independently test Ctrl+C in editing.

### S09 — wall-clock expiry asserted without the event that clears status

`showcase-scenarios.tsv:32` SC-SHELL-STATUS original only `time4000ms;time4000ms+1ns`. Oracle `app.rs:390–411` clears stored status only inside on_tick and only elapsed>4s; `draw_footer` uses stored status. Clock-only redraw must retain status; flash effective styling has its own <140ms check. Root: conflation of clock advancement, rendering, and Tick. Repair: clock-only unchanged checkpoint followed by explicit Tick; separately exact-deadline Tick alive and deadline+1ns Tick expired, replacement reset. Exact nanosecond boundary requires future clock injection; disposable harness can independently prove the event-ownership class without sleeps.

### S10 — mouse dialog launch restores prior focus, not necessarily launcher

`showcase-scenarios.tsv:50` SC-DIALOG-CONFIRM says close restores trigger focus for click launch. Oracle `app.rs` opens dialog/saves focus during page action before mouse fallback can focus launcher. Fresh mouse launch from NAV restores NAV; keyboard launch from focused trigger restores trigger. Root: universal restoration wording collapses two legitimate source paths. Repair: retain both fresh mouse and keyboard launch, assert distinct saved-focus ownership and unchanged background.

### S11 — Members proof claims mutations absent from transcript

`showcase-scenarios.tsv:73` SC-SETTINGS-MEMBERS required proof includes name edits and sorted identity, but original sequence only edits Role and never sorts before remove. Oracle `pages/settings.rs:28,276–355,555–566` includes name-nonempty validation and selected-member resolution through displayed order. Loophole: name validator or sorted removal can be broken while existing sequence passes. Root: required-observable prose not expanded to executable mutation transitions. Repair: explicit name invalid/retry+commit, explicit sort with retained selected identity, then cancel/confirm removal and empty boundary.

## Structural repair contract

### S12 — page drag target absent after folding or at tall sizes

`showcase-scenarios.tsv:44,47,77`: original SC-TREE-FOLD performs `*;-;End;Home;click disclosure;click row;wheel;drag`, leaving only five roots plus one expanded root (fits15 rows); original SC-EDITABLE-MOUSE requests a scrollbar at120x40 although14rows fit15; original SC-TASK-SCROLL requests a tree scrollbar at80/120 although the target tree fits. Oracle `pages/trees.rs:34`, `pages/editable.rs:75`, `pages/taskrunner.rs:215` and real hit registrations reproduce these absences. Root and loophole match S07. Repairs preserve absent-fit assertions, explicitly expand all before tree drag, resize editable to80x24 and task-tree to72x20, then restore original geometry. Production Harness confirms old absent/new present with completed drags.

### S13 — blocked Tick ownership asserted without blocked Tick input

`showcase-scenarios.tsv:29,36,76`: shell Help, busy-button modal/hidden, and pipeline-cancel transcripts previously supplied no Tick during the interval whose suspension they claimed. Merely opening and closing an overlay can pass when domain dispatch incorrectly leaks through overlays. Oracle `app.rs:390–412` calls page Tick only without a global dialog. Repair adds real blocked-interval Tick inputs before close/return, plus unchanged domain-state assertions and the first eligible Tick afterward. Real Harness confirms1000 modal ticks leave a running pipeline uncompleted while1000 visible ticks complete it. No wall-clock sleeps or changed oracle domain code.


Scenario expansion must validate each action against immutable oracle focus/hit geometry and assert semantic checkpoints before recording expected frames. Helper lookup failure is an error, never candidate retargeting or silent omission. Source-absent controls get explicit negative checkpoints plus source-present variants, preserving coverage. Rejected pre-repair transcripts remain here as counterexamples. Every corrected scenario must synchronize source TSV, owning task trusted obligations, task039 aggregate, baseline002 frozen source and global parity copies/expanded case keys. Shell crosscut repair ownership remains task032; all23 focus/resize/routing validation must name the staged page/component repair owners rather than granting task039 production scope.

## Validation status

Source-confirmed findings above are not yet independently closed. Disposable oracle harness regressions and exact per-file digest coverage follow below. Full expected-frame generation belongs future baseline task002; no baseline blessing or production repair performed by this audit.

## Full-file inspection ledger (pre-repair)

### Oracle-to-plan semantic coverage

| Oracle page source (all under `src/bin/showcase/pages`) | Source states/transitions checked | Plan owner / scenario coverage |
|---|---|---|
| overview.rs | Token palettes, text/typography, badge/spacing reference; no live page focus | 037 / BASE + all23 focus/resize/shell |
| buttons.rs | Four actions, two toggles, disabled vs inert matrix, busy/nonactivatable,2200ms eligible-page completion | 033 / BUTTON-ACTIONS, DISABLED, TIME |
| inputs.rs | Six fields/five enabled, required/email, blur/commit/revert, key shielding, masks/Unicode/paste | 033 / INPUT-EDIT, EMAIL-MASK;032 route persistence |
| textareas.rs | Long/empty/disabled/error, edit newline/selection/commit, cursor reveal/scroll, narrow clipping | 033 / TEXTAREA-EDIT, SCROLL; S03 |
| forms.rs | Compact/tall controls, name/reviewer validation, radio/checkbox/toggle, busy1800ms strict, reset | 033 / FORM-VALIDATE, SUBMIT |
| lists.rs | Single choice vs cursor, multi/range/all/none, disabled rows, empty boundary | 037 / LIST-SELECT + focus/hits |
| trees.rs | Hierarchy/disclosure/row activation, ancestor/fold-all/expand-all, source selection, fit/overflow | 037 / TREE-FOLD; S12 |
| tables.rs | Tasks/Checks columns/tones, three-state sort, row choice, wheelH/V and clamp | 034 / TABLE-SORT + focus/hits |
| editable.rs | Task/Owner/Branch/Changes rules, readonly columns, edit/cancel/Tab, source identity/sort, clipping | 034 / EDITABLE-CELL, MOUSE; S05/S12 |
| panels.rs | Cards/framed/nested, independent prose/log/list owners, follow and geometry | 036 / PANEL-SCROLL; S03 |
| sidebars.rs | Eight items/three sections/Billing disabled, current vs cursor, collapse glyph/focus/scroll | 037 / SIDEBAR-MODES; S06/S07 |
| dialogs.rs | Confirm/default, destructive cancel, custom three actions, prompt empty/max/valid, cancel/history | 038 / DIALOG-CONFIRM, PROMPT, CHOICES; S02/S10 |
| progress.rs | Build, pause/resume/restart, static progress/spinner/meter variants, domain ticks | 036 / PROGRESS-RUN + BASE/focus |
| scrolling.rs | Wrapped prose,120-row list,400-line follow log, append, thumb/track, wheel boundaries/hover, resize labels | 036 / SCROLL-OWNERS, THUMB |
| terminal.rs | Seven stages/cached/skipped/failure/blocked, terminal follow/selection/copy, step rail/split | 036 / TERMINAL-RUN, SELECT |
| editor.rs | Sample/syntax/blocks, editing/Unicode/selection, completion/query/filter/accept/dismiss, find, run/diagnostics | 036 / EDITOR-EDIT, COMPLETE, RUN |
| diff.rs | Unified/review narrow fallback, empty toggle/focus, five hunks/Unicode/copy, original press anchor | 036 / DIFF-MODES, SELECT;032 minimal public route |
| grid.rs |40/80/96 live customers/eight typed columns, ranges/sort/copy, pending insert/duplicate/delete/undo, SQL,4tick commit and500-seat error, toolbar/filter/viewer | 034 / GRID-NAV, EDIT, QUEUE, ACTIONS; S04 |
| chips.rs | Enable/remove/edit request/add candidate cycle/clear/all-any, overflow, select popup/current/cursor/cancel, disabled engine | 037 / CHIP-FILTERS, SELECTS |
| pickers.rs | Grouped search/scopes/empty, alternate choose, tab secondary close boundary, six level rows, captured vs outside input | 038 / PICKER-QUICK, TABS-LEVEL |
| chrome.rs | MenuBar/context ownership, session list, disabled/options/outside, brand and usage/PR actions, footer priority | 038 / CHROME-MENUS |
| settings.rs | General dirty/confirm/direct-save, Members name+role/sort/remove/invite/empty, Environment multi/prompt/validation/add/remove | 035 / SETTINGS-GENERAL, MEMBERS, ENV; S02/S11 |
| taskrunner.rs | Target tree, max2 pipeline state machine, integration failure, rerun, cancel preserving done results, log/follow and hidden modal tick suspension | 036 / TASK-RUN, SCROLL; S12/S13 |

Shell/main reviewed separately: all23 exact normalized CLI names and route cycle; invalid parser2/help/quit; minimum72x20; compact31/32; sidebar109/110; inspector99/100; fixed header/footer/body layout; key-before-global, pointer down/up/capture, modal prior-focus barrier, clock-vs-Tick; four oracle color modes. `data.rs` source read fully for task rows, project tree, languages, log/prose; constructors of Grid/Terminal/Editor/Task runner provide their own domain fixtures. No invented reference-column grid or disabled Picker row was added.

### Stage and proof feasibility

Tasks032–038 have explicit production write slices, ordered downstream; all shared dependencies reach TASK-031's component/runtime foundation.032 is shell-only with a minimal real Diff route,033 inputs/forms/buttons,034 grid/tables/editable,035 settings,036 viewport/editor/diff/task-runner pages,037 selection/navigation pages,038 overlays/chrome.039 remains artifact-only closure, not an escape for late production fixes. CHK001 host preflight→CHK002 direct capture→CHK003 PTY capture→CHK004 full comparison→CHK005 inventory accounting→CHK006 architecture→CHK007 close use independently produced `/proof/bin/tc-proof` with per-check contexts. The protected task payloads name the producer tasks001/070/071/072 and source baselines002–006; no future command was described as already runnable. Canonical task/v5 typed acceptance and verify/v2 mappings are consistent within this slice; task-meta/v1 dependencies use canonical project/group/number IDs.

Shared root synchronization still required: copied Showcase TSV in baseline002/host payloads, application-parity fields and expanded inventories/hashes, shell-contributions C02/C06/C07 global rows, and branch/checkpoint materialization grammar. Crosscut FOCUS/RESIZE repair-owner routing must be reviewed against all page/component owners before closure; artifact-only039 cannot perform such repairs.

### Actual validation, not completion certification

Disposable extraction: `/tmp/showcase-reaudit.AWN3SD`, populated with `git archive` of exact O. Only its `src/bin/showcase/app_tests.rs` received temporary tests via apply_patch. SHA-256 of diagnostic test file: `979a9698e66681a234be644243e41a2f60941cc84faca45598269762b09a37da`. Existing production App/Harness event routing/render/hit registration remain unchanged. Command `rtk proxy cargo test --locked --bin showcase reaudit_` in that directory: **14 passed,0 failed,41 filtered**,0.17s. Tests exercise the runtime failure classes above, plus six-page/seven-size scrollbar registration inventory and domain Tick suspension; S01 is planning scope contradiction, not a runtime bug. Initial diagnostic iterations exposed a mistyped PageId, text-lookup hitting explanatory Copy/Collapse text, and wrong guessed Copy status; corrected diagnostic fixtures use oracle control IDs and actual `Copied N statements`. Those failures are not represented as oracle regressions.

The direct clock test backdates only a test-owned status timestamp to prove dispatch ownership; it does not claim nanosecond boundary replay. Exact clock seams, all palettes, complete per-checkpoint serialized cells, PTY navigation/lifecycle, production candidate equality and every full proposed transcript remain future baseline/implementation gates, not accomplished by these focused tests.

`rtk proxy python3 -B docs/refactoring-plan/evidence/validate-plan.py --summary` ran during concurrent parent/team edits: nonzero,114 cross-artifact errors (shared index/frozen-copy drift and other slices included). No global pass claimed. Bare `python` shim had no selected version; `python3 -B` worked. Independent re-review must validate final shared synchronization and author repairs.

Every listed file read in full; SHA-256 captures the inspected planning revision before author repairs. These are source-review attestations, not execution-pass assertions.

| Path | Lines | SHA-256 | Semantic coverage |
|---|---:|---|---|
| `refactoring-tasks/terminal-components/completion/032/AGENTS.md` | 83 | `ebad0815479d1b162a586e92d57599530c765831009fae75dd14c12e86c02116` | Canonical safety/baseline/verification execution instructions; no mutation. |
| `refactoring-tasks/terminal-components/completion/032/README.md` | 193 | `25d8ece6094d2ac62f8bf57f19102c3747cff85b6be48478b54ad33b2e43eedd` | Goal/context/preconditions/scope, every requirement, typed AC→CHK link, decisions and checklist; staged ownership reviewed. |
| `refactoring-tasks/terminal-components/completion/032/task.toml` | 3 | `efd0d631f04bc1b1f7b5ef8961c4037c44e2d03384153809973979208af3f63b` | task-meta/v1 status and canonical dependency IDs; graph path checked. |
| `refactoring-tasks/terminal-components/completion/032/trusted/contributions.tsv` | 8 | `a250b3f6ab00cb15dd7f391c7a935b7958ec5b834b58b421cd4c6c9ba2afb638` | Seven shell semantic slices and consuming architecture checks; no whole-page claim. |
| `refactoring-tasks/terminal-components/completion/032/trusted/frame-contributions.tsv` | 2 | `7cc56edbdbeb1143a6f08d6c7da5ca7da4ac651084c58450009814b7b3c60637` | One complete undersize-frame contribution and compare ownership. |
| `refactoring-tasks/terminal-components/completion/032/trusted/obligations.md` | 98 | `c24a641b45dc9cb8e6c0c706606b4f339a09972a121af4109d97c7704045ffe3` | Every source obligation and scenario checkpoint/size/proof/citation; crosschecked actual oracle event state. |
| `refactoring-tasks/terminal-components/completion/032/trusted/scenario-ids.tsv` | 1 | `a7cc4d2c3e553648eb07193bfc1f38b12a2df70225e40154f081885666fda10b` | Complete owned scenario ID enumeration and aggregate inclusion. |
| `refactoring-tasks/terminal-components/completion/032/trusted/shell-contribution-contract.md` | 38 | `f6e61281935fbaa5123cc66136861bd301f5954f695c058c2857d4545f3dccc5` | Shell-only allowed source and cut boundaries, contributor/aggregate contract. |
| `refactoring-tasks/terminal-components/completion/032/trusted/source-obligations.tsv` | 10 | `15de5162876cbe3b2c586fdff8c93f7abcd1e6223bc1f9c166a404efe4105834` | Every historical evidence ID, resolved owner and architecture/behavior gate. |
| `refactoring-tasks/terminal-components/completion/032/verify.toml` | 60 | `e18fbffe42057c14d437eeab8c138704d31d303653114a09299b9412bc32191c` | verify/v2 command arrays, deadlines/scopes/output artifacts; future producer prerequisites and proof links checked. |
| `refactoring-tasks/terminal-components/completion/033/AGENTS.md` | 83 | `7f2adc2b88d2f40a455fb1f876ea5496c23765fbb0cfbf30509042ea2caa5007` | Canonical safety/baseline/verification execution instructions; no mutation. |
| `refactoring-tasks/terminal-components/completion/033/README.md` | 178 | `b2cb3a290327f23a5d376429d5f27fa2c016f61f7d231ccab6fbcc8751fc7d86` | Goal/context/preconditions/scope, every requirement, typed AC→CHK link, decisions and checklist; staged ownership reviewed. |
| `refactoring-tasks/terminal-components/completion/033/task.toml` | 3 | `5b43c3784db0e6335ffa8b4b4386a1b75dab6d08f66d1cbeea88d719a8d44172` | task-meta/v1 status and canonical dependency IDs; graph path checked. |
| `refactoring-tasks/terminal-components/completion/033/trusted/obligations.md` | 144 | `eb582738fd365f350984e7f4c6eb019bd2eb001f5e672f1cb76ffd2a783e65ac` | Every source obligation and scenario checkpoint/size/proof/citation; crosschecked actual oracle event state. |
| `refactoring-tasks/terminal-components/completion/033/trusted/scenario-ids.tsv` | 14 | `8d80a78faf91be30c23fc8271aad72c25f3bc45c373633dab5006fec978dfffe` | Complete owned scenario ID enumeration and aggregate inclusion. |
| `refactoring-tasks/terminal-components/completion/033/trusted/source-obligations.tsv` | 3 | `b3fd9ddfa812b3d4286cbf0d204fdb231405108da4dd3b43a5e5b62b98c5fcd2` | Every historical evidence ID, resolved owner and architecture/behavior gate. |
| `refactoring-tasks/terminal-components/completion/033/verify.toml` | 60 | `3967e3283ba991e027595e3b1d8530de9a78f445638e20cc41cfcfd1b9453981` | verify/v2 command arrays, deadlines/scopes/output artifacts; future producer prerequisites and proof links checked. |
| `refactoring-tasks/terminal-components/completion/034/AGENTS.md` | 83 | `f4e500ec7282b48831758feaed5761a911bd6b0ae231579177cd891fb96c8977` | Canonical safety/baseline/verification execution instructions; no mutation. |
| `refactoring-tasks/terminal-components/completion/034/README.md` | 178 | `58a026ca6850782f7314901f0e27b0b4fe8e98dcb2bd38e8adfc12d54906a2f5` | Goal/context/preconditions/scope, every requirement, typed AC→CHK link, decisions and checklist; staged ownership reviewed. |
| `refactoring-tasks/terminal-components/completion/034/task.toml` | 3 | `d2c05a78afd7bf98450a6f8e355ada3a7e25ae2cf7243b2fa21b6864c74b7c0c` | task-meta/v1 status and canonical dependency IDs; graph path checked. |
| `refactoring-tasks/terminal-components/completion/034/trusted/obligations.md` | 117 | `034f54f0a5273e74a9a8562053c2e7398cb2cb92ba5bdb77881ffd38ba566a03` | Every source obligation and scenario checkpoint/size/proof/citation; crosschecked actual oracle event state. |
| `refactoring-tasks/terminal-components/completion/034/trusted/scenario-ids.tsv` | 11 | `c559ba03e92264940ad50fe2042a75dff6b1f8219b2efd099c2303d8ca80528f` | Complete owned scenario ID enumeration and aggregate inclusion. |
| `refactoring-tasks/terminal-components/completion/034/trusted/source-obligations.tsv` | 2 | `bd32ea76aee8cfe2cef4685d8e213eaf08149abc0b2f4cde9843cde87b2ab8f1` | Every historical evidence ID, resolved owner and architecture/behavior gate. |
| `refactoring-tasks/terminal-components/completion/034/verify.toml` | 60 | `544dd5bbdf927a7824ad7d1874effe22560ff980128c24f55c2bf39b99e3c002` | verify/v2 command arrays, deadlines/scopes/output artifacts; future producer prerequisites and proof links checked. |
| `refactoring-tasks/terminal-components/completion/035/AGENTS.md` | 83 | `52e1b8016a6062bdb2c984ac006860ab3b53e1f09bace02bf3f74506184f12cd` | Canonical safety/baseline/verification execution instructions; no mutation. |
| `refactoring-tasks/terminal-components/completion/035/README.md` | 178 | `a544e5c117d01e2277c66a5ff15922af60afd3e19e44c006825aa9703cc81331` | Goal/context/preconditions/scope, every requirement, typed AC→CHK link, decisions and checklist; staged ownership reviewed. |
| `refactoring-tasks/terminal-components/completion/035/task.toml` | 3 | `ba64b39c7515a6e43db26bf699d2a7e82cb2f3cc86a9b356df3f9dad8d4153b1` | task-meta/v1 status and canonical dependency IDs; graph path checked. |
| `refactoring-tasks/terminal-components/completion/035/trusted/obligations.md` | 63 | `8c9cb36dd6ac1d6cc9ae2c84d7b52bf34c8019ad16d7fe532051465e44a7dd38` | Every source obligation and scenario checkpoint/size/proof/citation; crosschecked actual oracle event state. |
| `refactoring-tasks/terminal-components/completion/035/trusted/scenario-ids.tsv` | 5 | `88c9bd644b4434ba4072fc5690334be5349dd447bff74b3420def9f4a3f9b5d6` | Complete owned scenario ID enumeration and aggregate inclusion. |
| `refactoring-tasks/terminal-components/completion/035/trusted/source-obligations.tsv` | 1 | `3c79871023bc92c8cffd6e06a375b7a7deae7fa16e333e43a73b7b48f9554a6f` | Every historical evidence ID, resolved owner and architecture/behavior gate. |
| `refactoring-tasks/terminal-components/completion/035/verify.toml` | 60 | `a2d871bbc8ab840f87c5c087cb248e0d0e59b809c772c1175dd974cc5591e856` | verify/v2 command arrays, deadlines/scopes/output artifacts; future producer prerequisites and proof links checked. |
| `refactoring-tasks/terminal-components/completion/036/AGENTS.md` | 83 | `3813c3b3d16939ba3db5d095bdad17740d0be61cb64a37e6fa09619fbe6c16ee` | Canonical safety/baseline/verification execution instructions; no mutation. |
| `refactoring-tasks/terminal-components/completion/036/README.md` | 178 | `d89aca1c0ea9b950dbba82b4f632b72917adb6a9b1a01c44847aa3f9079906b8` | Goal/context/preconditions/scope, every requirement, typed AC→CHK link, decisions and checklist; staged ownership reviewed. |
| `refactoring-tasks/terminal-components/completion/036/task.toml` | 3 | `99377ad813366b89103835f2250a021f9f420f122fa3dc970facc2ab17730c17` | task-meta/v1 status and canonical dependency IDs; graph path checked. |
| `refactoring-tasks/terminal-components/completion/036/trusted/obligations.md` | 207 | `42e2b60eb5f14246598659e62360744d293505cd94dbd7060499f872baf47328` | Every source obligation and scenario checkpoint/size/proof/citation; crosschecked actual oracle event state. |
| `refactoring-tasks/terminal-components/completion/036/trusted/scenario-ids.tsv` | 21 | `c1cb00e05ecf4763c55e9cae6b71841bca272f90b1a5eabc7eea86579ce490b8` | Complete owned scenario ID enumeration and aggregate inclusion. |
| `refactoring-tasks/terminal-components/completion/036/trusted/source-obligations.tsv` | 1 | `3c79871023bc92c8cffd6e06a375b7a7deae7fa16e333e43a73b7b48f9554a6f` | Every historical evidence ID, resolved owner and architecture/behavior gate. |
| `refactoring-tasks/terminal-components/completion/036/verify.toml` | 60 | `f60fc5f4fc05b6732726afa386e22c7e4ead086652da3a4dca653f18a3c40df5` | verify/v2 command arrays, deadlines/scopes/output artifacts; future producer prerequisites and proof links checked. |
| `refactoring-tasks/terminal-components/completion/037/AGENTS.md` | 83 | `9a040fd66683bcc69283437f07b8c8748172c20a9e70f4c228e23f7cbac05ebd` | Canonical safety/baseline/verification execution instructions; no mutation. |
| `refactoring-tasks/terminal-components/completion/037/README.md` | 178 | `dec6cbad2a1fe161ef5667ce16af8d5363520e4f5dbff2d45c7236098835a7c0` | Goal/context/preconditions/scope, every requirement, typed AC→CHK link, decisions and checklist; staged ownership reviewed. |
| `refactoring-tasks/terminal-components/completion/037/task.toml` | 3 | `d50bf6b883c8beb2addd1d9023d8b99a32b71059733195fcc366fba348d50076` | task-meta/v1 status and canonical dependency IDs; graph path checked. |
| `refactoring-tasks/terminal-components/completion/037/trusted/obligations.md` | 117 | `a46793e9078436058cf8e78cc811b10a4777ff20a76113f9a9f56a162734176b` | Every source obligation and scenario checkpoint/size/proof/citation; crosschecked actual oracle event state. |
| `refactoring-tasks/terminal-components/completion/037/trusted/scenario-ids.tsv` | 11 | `f7cdbda0c018ef630e5463388e9eb899ceda1afd50ae9b2f4a9c320f77b80f19` | Complete owned scenario ID enumeration and aggregate inclusion. |
| `refactoring-tasks/terminal-components/completion/037/trusted/source-obligations.tsv` | 1 | `3c79871023bc92c8cffd6e06a375b7a7deae7fa16e333e43a73b7b48f9554a6f` | Every historical evidence ID, resolved owner and architecture/behavior gate. |
| `refactoring-tasks/terminal-components/completion/037/verify.toml` | 60 | `3182e6205559993822631c32f0af7cb6067811d6b7eaa4073111b91eb7479abc` | verify/v2 command arrays, deadlines/scopes/output artifacts; future producer prerequisites and proof links checked. |
| `refactoring-tasks/terminal-components/completion/038/AGENTS.md` | 83 | `2803b88cb10a3f35cb531f3b24ca06de6d5406bfcc451ffbc42aeff0416efb98` | Canonical safety/baseline/verification execution instructions; no mutation. |
| `refactoring-tasks/terminal-components/completion/038/README.md` | 178 | `987feb3dbf511cc2fb49863029aa20d3efd05b836b4306241dc366eaf4d6eac6` | Goal/context/preconditions/scope, every requirement, typed AC→CHK link, decisions and checklist; staged ownership reviewed. |
| `refactoring-tasks/terminal-components/completion/038/task.toml` | 3 | `36efa922871ee5f6aafdb8c59d80ace61bb89021c7781af0c34c391dd4711354` | task-meta/v1 status and canonical dependency IDs; graph path checked. |
| `refactoring-tasks/terminal-components/completion/038/trusted/obligations.md` | 108 | `be8a35bee29a07b9d621dc66589864adcb5d63990aee1165470b9e773692eb06` | Every source obligation and scenario checkpoint/size/proof/citation; crosschecked actual oracle event state. |
| `refactoring-tasks/terminal-components/completion/038/trusted/scenario-ids.tsv` | 10 | `4b337b67ff011dce8539d08deac8fbdd404d0ccb2e38acd2b318d6ac1bafca95` | Complete owned scenario ID enumeration and aggregate inclusion. |
| `refactoring-tasks/terminal-components/completion/038/trusted/source-obligations.tsv` | 1 | `3c79871023bc92c8cffd6e06a375b7a7deae7fa16e333e43a73b7b48f9554a6f` | Every historical evidence ID, resolved owner and architecture/behavior gate. |
| `refactoring-tasks/terminal-components/completion/038/verify.toml` | 60 | `f08f6a1bdff73bc3e78a86a167f17a1022bd1f6e027c35cfc1a881aa0eb1a933` | verify/v2 command arrays, deadlines/scopes/output artifacts; future producer prerequisites and proof links checked. |
| `refactoring-tasks/terminal-components/completion/039/AGENTS.md` | 83 | `5c233e83a879526321c2572b8a67893eae90d2429671e8c91a2367000d99914e` | Canonical safety/baseline/verification execution instructions; no mutation. |
| `refactoring-tasks/terminal-components/completion/039/README.md` | 174 | `6a7dcbaa653f6ffb5c876f141c6080af9988166dd6f2972fd4282a377920b4cf` | Goal/context/preconditions/scope, every requirement, typed AC→CHK link, decisions and checklist; staged ownership reviewed. |
| `refactoring-tasks/terminal-components/completion/039/task.toml` | 3 | `e96efdb5dedaac64e0a88e66fbff96d1ac273ca8a6c6c1e7f1da37b2c65d9117` | task-meta/v1 status and canonical dependency IDs; graph path checked. |
| `refactoring-tasks/terminal-components/completion/039/trusted/obligations.md` | 711 | `d6f982569ec8caf4d87d7df1c9c3f59c8a56ad5f88ce66ea16adbae0b1fd5c50` | Every source obligation and scenario checkpoint/size/proof/citation; crosschecked actual oracle event state. |
| `refactoring-tasks/terminal-components/completion/039/trusted/scenario-ids.tsv` | 77 | `08a60997d3fca3c0b8de7276f45488c196dcb0a01b820a5c166f0d0e2fbc45e1` | Complete owned scenario ID enumeration and aggregate inclusion. |
| `refactoring-tasks/terminal-components/completion/039/trusted/source-obligations.tsv` | 5 | `d25ac2c6e7c86faab7d8aa73ff824b30993cfc2afedd90ac80e8a3577857c6e2` | Every historical evidence ID, resolved owner and architecture/behavior gate. |
| `refactoring-tasks/terminal-components/completion/039/verify.toml` | 60 | `71d93fa002faf58155389fe403f5c39cf5736b06fdbed5b879a1f43de50cc7d5` | verify/v2 command arrays, deadlines/scopes/output artifacts; future producer prerequisites and proof links checked. |
