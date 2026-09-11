# Independent review of the cell-selector repair

## Scope and result

The newly B-authored JTR-01/JTR-02 repair is source-consistent in this bounded review. This reviewer authored earlier JT planning changes, but did not author B's select-only/click-cell repair or its six counterexample/positive probes. Verify-and-stop limited this review to those new mechanisms, eight canonical TablePro scenarios, their affected flow contributions and TASK-022's shared routing responsibility. This is not approval of all 150 application parents, all 84 contributions, complete application frames or PTY parity.

## Independent execution and source binding

Independently read all six tests in `/tmp/jt-rereview.lBNmr0/lib.rs`, then ran:

`rtk proxy cargo test --offline --manifest-path /tmp/jt-rereview.lBNmr0/Cargo.toml --target-dir /private/tmp/tui-snap-audit.656lHG/oracle-clock/target --lib audit:: -- --nocapture`

Result: six passed, zero failed, ten unrelated tests filtered. Tests execute actual TablePro App::handle/App::render, not a replacement business-state model. The harness directly calls App::connect to establish its initial connected fixture; it does not prove the complete CLI/connection-navigation prefix. All four prescribed sizes are exercised by the repaired positive tests.

The imported App copy `/tmp/jt-reaudit.4mpJ2V/app.rs` has SHA-256 `809f136ec6eef63066ea58a081a54d396fbbb8345f601311fdc55bb1ea00f5f1`, independently matching `git show 02f5294bfdbf38004cc49130d0aff1d01f31434c:src/bin/tablepro/app.rs`. The six-test harness digest is `7882b228d5d2bfb4f7bfb59ac8136bd77e2e0341c429629d1b7d805e933fd06e`. Its imported oracle library and connections/db/model/sql/tabs/workbench files have no tracked difference from that pin. Other disposable worktree experiments are outside these imports. This is a diagnostic source replay, not a protected baseline receipt.

## Counterexamples and structural correction

Oracle `src/widgets/grid.rs:1310–1345` first resolves actual click applicability, then distinguishes current unranged versus different cells. A current editable cell begins editing; a different cell moves without beginning editing. The old cell macro had already selected the destination before clicking it. The extra Enter suffix therefore committed inline currency before paste, or closed the JSON viewer before its expected checkpoint. Both old-mechanism counterexamples pass as explicit negative demonstrations: currency receives no pending paste mutation; JSON viewer is closed.

Current `tablepro.md:79` makes cell selection keyboard-only, with real focus, Ctrl+Home and row/column navigation. `tablepro.md:81` makes completed click-cell an explicit separate input with no hidden reveal. It binds the actual topmost hit, Grid::locate result, clipping and a non-reference-arrow interior coordinate. It does not normalize current-cell activation or equate a registered rectangle with a routable target. The four-size currency test independently proves current click edits, different click only moves, and repeated click edits.

Oracle `grid.rs:1213–1235` limits locate to the source-visible column interval. First-row orders.notes is partially registered but outside that routable interval at all four tested sizes. At120x40 its registered rectangle is `(95,9,17,1)`. All four completed notes clicks remain inert. Keyboard Enter then reaches `grid.rs:570–578`'s long-text OpenViewer branch; subsequent paste creates no pending edit. The source notes value is not a short inline field. This independently confirms the corrected source interpretation, including the author's rejected earlier hypothesis that the notes click itself opened the viewer.

Inline/pending scenarios now use source-proven short currency and Ctrl+L, matching the original `app_tests.rs` pending-edit/save setup. The positive save test proves actual replacement bytes, nonempty pending state, preview, acknowledgement/token gate, Saving status, four still-pending ticks and the fifth saved tick with appended history. Notes behavior remains explicitly covered in TP-037 instead of being silently deleted or redesigned.

## Exact planning checks

Read all changed canonical rows TP-031/034/035/036/037/063/065/077 and their eight stage-audit rows, seven semantic contributions and seven whole-frame contributions. New TP-031 click checkpoints and TP-037 notes checkpoints occur in their contribution selections. Each separate variant restarts its accepted seed. TP-077 help-paste remains semantic-only at the early owner and full-frame at its complete parent; the repair does not manufacture an early Help dependency.

Compared all affected donor rows in TASK-005 and TASK-058–064 against the canonical stage/semantic/frame tables: 106 exact matches, zero drift. TASK-060 depends on TASK-022, so the shared routing guard precedes application use. TASK-022 lint passed with zero errors/warnings. This mechanical comparison does not stand in for the coordinator's source-obligation/history synchronization.

One clarification was requested from B and is now resolved: W-022-08 distinguishes the generic Grid producer's geometry/routing/external-edit action from the actual TablePro viewer modal and pending-state application proof. Independently reread the repaired row: TASK-022 proves the source-ineligible click, unchanged edit state, keyed `GridAction::EditRequested` from `EditIntent::External`, and no Grid mutation on paste. The actual viewer, pending model and whole application frames remain TASK-060's contribution and TASK-063's parent. Main `grid.rs:229–239,408–409,2905,2921` independently confirms those existing API names and dispatch, so this clarification introduces no invented OpenViewer API or future Dialog/application prerequisite. Repeated TASK-022 lint passes. The bounded correction is accepted with no remaining material finding in this rereview scope.

## Checked checkpoint inventory

These digests identify reviewed bytes, not concurrent future edits or an entire-plan seal.

| File | Lines | SHA-256 |
| --- | ---: | --- |
| docs/refactoring-plan/tablepro.md | 103 | aa30f9763c17cfab9f7151d85514d212de909cc34b371ae2d932f892943a7062 |
| docs/refactoring-plan/tablepro-scenarios.tsv | 81 | 83bc5cc6c78fc4cbc578af84d609c66283069e9b43d00264896f486997a31374 |
| docs/refactoring-plan/app-flow-stage-audit.tsv | 151 | fdfa824b490e9dfc8fc42063e7d4ce30c470430bd84443b63cb2a82c23f69edc |
| docs/refactoring-plan/app-flow-contributions.tsv | 43 | 19b07398437bdc4373aa45cc27d57937c103190a8f2ae248f92e672d0216d8ec |
| docs/refactoring-plan/app-flow-frame-contributions.tsv | 43 | ed65acdaca68aeef061b38ddd33f2a098a919c0feef65ac2d2344d681be70cb4 |
| refactoring-tasks/terminal-components/completion/022/trusted/obligations.md | 435 | 62550801cc0aafe1a128d7884ab40e06ce567ab41b467478e0f836e7a57ae938 |
| refactoring-tasks/terminal-components/completion/022/trusted/source-witnesses.md | 20 | 564bd6dd4874cad610fccdbd8ca48c626906b18d9c6a779a8b9b33b5f4ed3a71 |

Only this review report was written for the JT rereview. No canonical scenario, task, product source, golden artifact, AGENTS file or commit was changed by this reviewer.
