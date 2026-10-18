# Jackin/TablePro repair re-review

Scope: bounded independent review of reaudit-jackin-tablepro.md and its repaired TASK-051–064 contracts. Skill verify-and-stop used for focused verification; no production or canonical AGENTS edits. Oracle tag and harness HEAD independently resolve 02f5294bfdbf38004cc49130d0aff1d01f31434c; main authority remains7b27732a8c3c131760ec3438f641cb3c11343a42.

## Material findings

### JTR-01 — reveal-and-click macro changes the state assumed by its suffix

At pre-correction tablepro.md:79, cell(row,column) sends Ctrl+Home/Down/Right to select its target and then performs a completed click. tablepro-scenarios.tsv:32–38,64,66,78 follows cell with Enter or another nonediting action. Oracle src/widgets/grid.rs:1334–1341 activates an already-current unranged cell: currency enters editing, JSON opens its viewer. The following Enter therefore commits currency before paste or closes the viewer before cp(json_viewer). A registered hitbox-only probe does not exercise this composition.

Independent exact-source tests in /tmp/jt-rereview.lBNmr0 pass for currency reveal+click+Enter then Paste=Ignored with no pending mutation, and JSON reveal+click+Enter closing the viewer. Root condition: the selector was treated as geometry-only while silently delivering a state-changing click. Structural repair: make cell explicitly keyboard selection only; separate click-cell with no hidden reveal/navigation and source-qualified actual click outcomes. Preserve completed-click coverage as explicitly named trajectories, never normalize source click semantics.

### JTR-02 — notes is neither a reliable inline editor target nor uniformly routable after registration

The first orders.notes value at the pinned deterministic source is a long text. At120x40 after keyboard reveal it has registered Rect{x:95,y:9,width:17,height:1}, and the topmost hit equals its intended cell ID, yet DataGrid::locate returns None because the partly visible trailing column is outside hscroll.offset..offset+viewport_len (grid.rs:1225–1235). The click is inert. Keyboard Enter then reaches the long-text OpenViewer branch (grid.rs:570–578), not the inline editor. Thus TP-031/034/035/036/063/065/077 notes-based inline/pending assertions cannot be obtained by adding reveal keys.

The independent notes probe proves hit present + locate absent + click no edit/modal, followed by Enter opening the viewer and paste leaving pending empty. The initial tentative hypothesis that the notes click itself opened a viewer was rejected by an intentionally failing diagnostic before inspecting the locate interval; it is not retained as evidence. Root condition: expected business-state names were not checked against source value/type/width and actual routing guards. Structural repair: use source-proven short currency text for editable literal/save trajectories (the original pending_edits_preview_and_save test also edits currency), while preserving the original notes inputs as explicit long-text-viewer and partial-hit nonactivation variants. Existence of a hit rectangle is not activation proof.

## Narrow proofs rerun

- Author harness /tmp/jt-reaudit.4mpJ2V: --lib audit:: passes5 tests; --bin jackin-audit audit_ passes2 tests. This independently reproduces lazy-node nonloading/loading timing, fifth-tick commit, picker paste fallthrough, offscreen notes hit registration, Jackin config re-entry, and Inspect re-entry.
- New exact-source harness /tmp/jt-rereview.lBNmr0: --lib audit:: passes4 tests, comprising the three counterexamples above plus positive select-only currency edit/preview/save through four-pending/fifth-saved ticks at80x24,100x30,120x40,160x50. The positive checks actual currency bytes, pending state, dialog opening, Saving/Saved status and appended history count; it does not qualify exact complete frames or PTY traces.
- Commands use cargo test with --manifest-path for the named harness and --target-dir /private/tmp/tui-snap-audit.656lHG/oracle-clock/target. Tests invoke actual App::handle and App::render. No model-only replacement handler is used.
- Harness dependencies point at the oracle worktree. Its tracked changes are confined to Showcase binaries/pages and TablePro app/main clock experiments, which are not imported by these harnesses; their TablePro App is copied from git show. Copied app.rs SHA256809f136ec6eef63066ea58a081a54d396fbbb8345f601311fdc55bb1ea00f5f1 equals the pinned git blob bytes. Imported Jackin source and shared library remain unchanged. These disposable harnesses are diagnostics, not protected baseline adapters.

The author7 passing probes remain useful but did not test the full cell-macro+suffix composition. Jackin replay continuations and the repaired lazy-load/commit countdown boundaries are source-consistent within the inspected scope. No broad claim of all150 parents, all84 contributions, full application parity or future oracle qualification is made.

## Repair authority and independence

After these findings the coordinator explicitly authorized this reviewer to repair the canonical TablePro scenario/grammar, canonical flow semantics, exact donor clones and the bounded TASK-022 partial-hit contract; remaining source-field mirror synchronization belongs to the closure coordinator. Such edits are author work, not independently accepted by this same report. A separate reviewer must recheck the finalized repaired bytes and actual-source probes.

## Authorized correction and handoff

The findings were repaired as author work after explicit coordinator authorization. `tablepro.md` now separates keyboard-only cell selection from explicit source-bound completed clicks. Eight canonical TablePro rows (TP-031/034/035/036/037/063/065/077) were corrected. Inline/save/dirty paths use short currency and explicit Ctrl+L; TP-031 separately retains current/different/repeated mouse behavior, and TP-037 retains the long notes click/Enter/paste source branch. No historical source-native test was removed. TASK-022 owns the source-qualified partial-hit routing contract, not an application shim.

Canonical stage-audit8 rows and semantic/frame contribution7 rows were updated. TP-031's renamed previous_source_focus and three added click checkpoints, and TP-037's four notes checkpoints, are explicitly in the contribution sets; accepted post-preset-T seed is restarted for fresh variants. Exact donor rows were synchronized into005/058–064 stage payloads and005/060/062/064 contribution payloads. The005 stage donor also received the already-authoritative JA-024/JA-060/TP-018/TP-071 rows previously awaiting coordinator synchronization. Obligations source-field mirror synchronization is delegated to the closure coordinator; no independent success is claimed before its final check.

Two additional exact-source probes now pass: explicit current/different/repeated currency clicks at all four ALL sizes, and retained notes actual click/Enter/paste at all four sizes. All four tested notes geometries produce a registered but source-unroutable cell, inert completed click, then keyboard-opened viewer and unchanged pending state. Total new harness:6 passing tests (alongside author7 rerun). Only the120x40 rectangle coordinate is asserted numerically in this report. All future color profiles/full-cell equality/PTY/compiler-qualified observer paths remain unexecuted here.

TASK-005/022/060/062/063/064 taskfmt lint passes with zero errors/warnings using the canonical experiment config. The independent JT author has been assigned fresh review of these newly authored corrections and the six probes; this report does not self-approve them.

### Changed-file handoff inventory

Hashes cover stable authored canonical/donor files at handoff, before any subsequent independently requested repair or coordinator-wide reseal. They do not cover unchanged files as if newly audited.

| File | Lines | SHA256 |
| --- | ---: | --- |
| `docs/refactoring-plan/tablepro.md` | 103 | `aa30f9763c17cfab9f7151d85514d212de909cc34b371ae2d932f892943a7062` |
| `docs/refactoring-plan/tablepro-scenarios.tsv` | 81 | `83bc5cc6c78fc4cbc578af84d609c66283069e9b43d00264896f486997a31374` |
| `docs/refactoring-plan/app-flow-stage-audit.tsv` | 151 | `fdfa824b490e9dfc8fc42063e7d4ce30c470430bd84443b63cb2a82c23f69edc` |
| `docs/refactoring-plan/app-flow-contributions.tsv` | 43 | `19b07398437bdc4373aa45cc27d57937c103190a8f2ae248f92e672d0216d8ec` |
| `docs/refactoring-plan/app-flow-frame-contributions.tsv` | 43 | `ed65acdaca68aeef061b38ddd33f2a098a919c0feef65ac2d2344d681be70cb4` |
| `refactoring-tasks/terminal-components/completion/022/trusted/source-witnesses.md` | 20 | `564bd6dd4874cad610fccdbd8ca48c626906b18d9c6a779a8b9b33b5f4ed3a71` |
| `refactoring-tasks/terminal-components/completion/022/trusted/obligations.md` | 435 | `62550801cc0aafe1a128d7884ab40e06ce567ab41b467478e0f836e7a57ae938` |
| `refactoring-tasks/terminal-components/completion/005/trusted/app-flow-stage-audit.tsv` | 151 | `fdfa824b490e9dfc8fc42063e7d4ce30c470430bd84443b63cb2a82c23f69edc` |
| `refactoring-tasks/terminal-components/completion/058/trusted/app-flow-stage-audit.tsv` | 81 | `3a3f3d5d670d8c9b47dbf79271e9e2f43a848389d1b30f99ad3f44d172e45a56` |
| `refactoring-tasks/terminal-components/completion/059/trusted/app-flow-stage-audit.tsv` | 81 | `3a3f3d5d670d8c9b47dbf79271e9e2f43a848389d1b30f99ad3f44d172e45a56` |
| `refactoring-tasks/terminal-components/completion/060/trusted/app-flow-stage-audit.tsv` | 81 | `3a3f3d5d670d8c9b47dbf79271e9e2f43a848389d1b30f99ad3f44d172e45a56` |
| `refactoring-tasks/terminal-components/completion/061/trusted/app-flow-stage-audit.tsv` | 81 | `3a3f3d5d670d8c9b47dbf79271e9e2f43a848389d1b30f99ad3f44d172e45a56` |
| `refactoring-tasks/terminal-components/completion/062/trusted/app-flow-stage-audit.tsv` | 81 | `3a3f3d5d670d8c9b47dbf79271e9e2f43a848389d1b30f99ad3f44d172e45a56` |
| `refactoring-tasks/terminal-components/completion/063/trusted/app-flow-stage-audit.tsv` | 81 | `3a3f3d5d670d8c9b47dbf79271e9e2f43a848389d1b30f99ad3f44d172e45a56` |
| `refactoring-tasks/terminal-components/completion/064/trusted/app-flow-stage-audit.tsv` | 81 | `3a3f3d5d670d8c9b47dbf79271e9e2f43a848389d1b30f99ad3f44d172e45a56` |
| `refactoring-tasks/terminal-components/completion/005/trusted/app-flow-contributions.tsv` | 43 | `19b07398437bdc4373aa45cc27d57937c103190a8f2ae248f92e672d0216d8ec` |
| `refactoring-tasks/terminal-components/completion/060/trusted/app-flow-contributions.tsv` | 20 | `dc2a37008904fa1375009a726d09a7d74b2a92ec1607b361a78313ad173d6dbd` |
| `refactoring-tasks/terminal-components/completion/062/trusted/app-flow-contributions.tsv` | 7 | `f139a63226aeb92817be45238a47cb8588af6e213fa5e87d701539eb4b4515de` |
| `refactoring-tasks/terminal-components/completion/064/trusted/app-flow-contributions.tsv` | 34 | `1d003c7076df0441411ff6f5bea3be8abb21ad8573fc1eb993617407a7013722` |
| `refactoring-tasks/terminal-components/completion/005/trusted/app-flow-frame-contributions.tsv` | 43 | `ed65acdaca68aeef061b38ddd33f2a098a919c0feef65ac2d2344d681be70cb4` |
| `refactoring-tasks/terminal-components/completion/060/trusted/app-flow-frame-contributions.tsv` | 20 | `73728bd5d6b8f5722b9f2a5dc614647f548a9432b0cd2301c36412833a54b920` |
| `refactoring-tasks/terminal-components/completion/062/trusted/app-flow-frame-contributions.tsv` | 7 | `487f396b272e8735dd156196a8152ccc7849db20e812fc39dcb05767520af3f1` |
| `refactoring-tasks/terminal-components/completion/064/trusted/app-flow-frame-contributions.tsv` | 34 | `fb01c672e755c6aa275b44e70dc3832a2f39b38e765a1fb3ede708d6789ee255` |
| `refactoring-tasks/terminal-components/completion/060/trusted/source-obligations.tsv` | 55 | `2f1245ade7ad9e0fed60b8714b4d5bd2248e21e77091a4cb0f60f0d4d9136299` |
| `refactoring-tasks/terminal-components/completion/062/trusted/source-obligations.tsv` | 13 | `def53c06979e4ecd4d637d1eb949a57d941a61993bf90ae7bdf0023ee2fa7b6e` |

The nine changed APP-FLOW decision_requirement exemplars in primary060/062 source-obligations.tsv now equal their canonical semantic/frame row text. The coordinator's subsequent sync-derived projection must use these corrected primary exemplars, not propagate stale checkpoint names or notes-based inline claims. Source-field clause mirrors remain a separate projection and do not substitute for this authority synchronization.
