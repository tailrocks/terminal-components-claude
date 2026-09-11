# Fresh adversarial Jackin/TablePro task audit

Audit authority: PLANNING_GOAL.md read completely. Immutable oracle tag independently resolves `02f5294bfdbf38004cc49130d0aff1d01f31434c`; architectural comparison is `7b27732a8c3c131760ec3438f641cb3c11343a42`. Skill: verify-and-stop, used for bounded validation without product edits. No task execution, product edit, canonical AGENTS edit or commit performed. Disposable Rust diagnostic sources are under `/tmp/jt-reaudit.4mpJ2V`.

## Findings

1. **JT-01 — TP-034 omitted the entire asynchronous commit interval.** Original `tablepro-scenarios.tsv:35` confirms Save, immediately captures `cp(saved)`, then opens History and captures `cp(history_entry)`, while asserting cleared pending changes and RowEdits history. Oracle `src/bin/tablepro/app.rs:1828–1834` sets `committing=Some(5)` and status Saving; `:339–344` calls finish_commit only on fifth Tick; `:1113–1147` then clears the active table's pending changes and records history. Source test `app_tests.rs:638` waits eight ticks. Counterexample: immediate capture still has pending edits; switching to History before countdown completion causes finish_commit's active-table match to return. Root condition: prose checkpoint name and eventual assertion were accepted without proving the preceding action sequence reaches that state. Repair: retain confirmation as `cp(saving)`, capture four-tick pending state, tick once and capture saved, then open History. Add explicit commit timing to action grammar and derive stage mirrors from the repaired source row.

2. **JT-02 — JA-024 appends config scrolling after replay has left config.** Original `jackin-scenarios.tsv:25` replays environments_stay_readable_with_a_hundred_roles then End/PageUp/Home and wheel(config). Oracle `src/bin/jackin_preview/app_tests.rs:1192–1197` ends by switching to Roles, entering its body and selecting +Load role. No Environments config surface remains painted. Root condition: replay treated as a named state setup without composing its actual final route/tab/focus. Repair: preserve complete source replay and its Roles assertion, explicitly Esc/4/Enter back into Environments before config scrolling. No private focus mutation or replay truncation.

3. **JT-03 — JA-060 appends Inspect inputs after replay closes Inspect.** Original `jackin-scenarios.tsv:61` replays inspect_changes_opens_from_the_view_menu_in_both_modes and immediately sends compact input. Oracle `src/bin/jackin_preview/app_tests_chrome.rs:153–157` sends d/Esc/Esc and asserts no Inspect plus Route::Capsule. Following Down/Enter goes to Capsule. Root condition matches JT-02. Repair: preserve replay, reopen Inspect through F10/Right/Right/End/Enter, then execute explicit compact/advanced continuation.

4. **JT-04 — TP-018 claimed lazy loading without selecting a lazy node.** Original `tablepro-scenarios.tsv:19` starts W, Right, then loading checkpoints. Oracle `src/bin/tablepro/app.rs:180–184`, `workbench.rs:150–158` starts Explorer at already-expanded public; `src/widgets/tree.rs:396–402` Right only moves to Tables. No pending lazy job exists. Repair: retain original expanded-node navigation as its own checkpoint, End to audit then Right to request actual lazy children; preserve one-tick loading and three-tick completion. This also repairs TASK-059's independently seeded empty-Workbench suffix.

5. **JT-05 — universal modal-first paste contradicts a reachable oracle route.** Oracle `src/bin/tablepro/app.rs:236–254` intercepts Dialog and Filter, but Picker falls through to Workbench; `workbench.rs:1194–1201` forwards to active editor; `tabs.rs:1543–1545` pastes while editing. `app.rs:638–645` opens Switcher/TabList even during editing; opening them at `:1232–1234` / `:1337–1338` leaves editing active. Counterexample: W, Ctrl+T, i, Ctrl+O (or Ctrl+G), paste(PARITY) mutates underlying editor and leaves picker query unchanged. Root condition: universal architecture amendment chosen without a frozen app-level oracle conflict disposition. Preserve exact observable behavior using an explicit reusable routing policy; separately retain ordinary Dialog capture and any non-observable stale-target safeguards. Coordinate TASK-017/023 owner with TASK-063; never silently bless new modal behavior.

## Independent proof and scope assessment

6. **JT-06 — offscreen cell selectors lacked their required reveal actions.** `tablepro.md` said cell(row,column) used visible cells and to scroll with specified keys, while TP-031/034/035/etc immediately target notes after T. Source `db.rs:489–503` makes notes column8; `src/widgets/grid.rs:1550` registers only current column rectangles. Disposable real-handler test confirms cell_id(0,8) has no hitbox at initial120×40, and Ctrl+Home plus eight Right inputs creates it. The action grammar now explicitly resolves source column ordinal/display row, focuses Grid through real traversal, sends the bounded reveal keys, then freezes and performs its ordinary cell click. No candidate geometry search or private cursor injection.

The additional `audit::cell_notes_needs_keyboard_reveal` test passes; TablePro diagnostic total is five, Jackin total two. JT-05 implementation policy was refined with component owners: app-specific consumed Paste dispatch invokes the narrow TASK-025 CodeEditor helper shared with normal Intent::Paste. Default layer capture remains; no hidden runtime intent, fake focus or duplicated insertion algorithm.

All 150 existing parent rows were parsed and their declared application producers checked against actual task.toml transitive dependencies: every producer is an ancestor of its complete primary or the primary itself. This verifies graph consistency, not source completeness of the declared producer set. Jackin producers form 051→052→053→054→055→056→057; TablePro forms 058→059→060→061→062→063→064, with shared component prerequisites preserved.

Each of tasks051–064 has seven typed checks; preflight, direct capture, PTY capture, compare, account-tests, architecture and close bind immutable per-check contexts. Tasks051–056 and058–063 allow only their app src/tests. Tasks057/064 are evidence-only closures. Shared-library fixes therefore belong to prerequisite component owners. Candidate observation seams remain untrusted extraction, cannot replace real dispatch/rendering, and cannot change protected oracle mappings. Canonical AGENTS text is retained with campaign protocol precedence, not silently edited.

The 84 derived identities (42 complete-frame plus42 semantic contributions) are separately keyed; seed declarations are direct-only, whole-frame, source-qualified, independently replay-equivalent and cannot close the parent or claim CLI/PTy reachability. TASK-004/005 baseline producers must retain original full traces and source-native assertions. Empty-Workbench seeds correctly derive TP-026 after close, not Workbench::new masquerading as W. Source App::connect creates Query1 and source state traversal includes all App/Workbench fields, time, query_counter, focus, ring/hits, pending jobs and nested state. Seed constructor qualification is still future baseline work, not evidence already captured.

Source inventory examined against scenario groups: Jackin all eleven routes, eight worlds, CLI/motion/color/seed/time, Manager/Prelude, five Editor and Settings tabs, shared config, Accounts/Usage/op steps, eleven Cockpit stages and failure/cancel/handoff, Capsule prefix/topology/input/selection/menus/overlays/takeover/lifecycle, Inspect and dirty-exit, minimum-size and transient interaction states. TablePro CLI/Connections/basic+advanced forms/errors, Explorer/focus/drawer/tabs/empty, Grid editing/selection/sort/filter/structure/viewer/FK, Query/editor/completion/results/explain/cancel, safety/save/dirty flows, History/Switcher/help/layer/resize/color/runtime. Read source with git show at both pinned commits; main Jackin historical painters at app.rs:4037/5019/5189 and main TablePro shallow form/action/render ownership remain explicitly forbidden task endpoints.

Task semantic review:051 rituals/manager/prelude plus clock/status/inactive fanout;052 Editor/Settings/config plus secret lifetime;053 Accounts/Usage/op plus stale/unknown meter distinction;054 launch/effective credentials/cancellation;055 retained Capsule viewport/topology and no-clone bounds;056 Inspect/dirty exit and F09 copy requirement;057 full Jackin closure;058 Connections/form/simulation;059 Explorer/tab identity and F05;060 all22 Grid capabilities, adapter boundaries,500×12 and500×14 distinct performance obligations, F04/F06/F07/F08a;061 editor/query/result;062 safety/commit/dirty ownership;063 History/Switcher/help;064 full TablePro closure. Source-obligation IDs, mapping columns, common requirement/check bodies, per-task changes and all trusted table memberships inspected. F04/F08a/F09 historical clauses require coordinated exact-oracle adjudication where source differs; their accepted status alone does not authorize redesigned UI.

## Pre-repair package inventory

Final focused verification: TASK-063 canonical `taskfmt --config /Users/donbeave/Projects/donbeave/task-format/experiment.toml lint /Users/donbeave/Projects/terminal-components-claude/refactoring-tasks/terminal-components/completion/063` returns LINT PASS, zero errors/warnings. All150 parent ancestry, all84 contribution-parent joins, all14×7 protected argv/acceptance links and duplicate contribution identities pass mechanical checks. Scoped git diff --check passes. Current checker contexts/observer field schemas/complete expanded source traces remain future TASK-004/005/070/071/072 output and were not executed as if already present; their nonempty/source-binding/negative-vector contracts were reviewed, but no claim of full150-parent executable success is made. Further source-level history conflict adjudication is coordinated with component owners. Global baseline copies and hash/source-obligation regeneration are coordinator-owned.

Post-repair package hashes superseding corresponding pre-repair rows (unchanged package rows retain their recorded hashes):

| File | Lines | SHA256 |
| --- | ---: | --- |
| `refactoring-tasks/terminal-components/completion/051/trusted/app-flow-stage-audit.tsv` | 71 | `e91045afc28e2394ec69b9744bba295a610c1d4c5462fb3bb576dfdccc5f8c23` |
| `refactoring-tasks/terminal-components/completion/052/trusted/app-flow-stage-audit.tsv` | 71 | `e91045afc28e2394ec69b9744bba295a610c1d4c5462fb3bb576dfdccc5f8c23` |
| `refactoring-tasks/terminal-components/completion/052/trusted/obligations.md` | 186 | `758da2b4032dc0ceceeaba5bb644a27b26bc3538b595fbb63ae08afd0d0332e7` |
| `refactoring-tasks/terminal-components/completion/053/trusted/app-flow-stage-audit.tsv` | 71 | `e91045afc28e2394ec69b9744bba295a610c1d4c5462fb3bb576dfdccc5f8c23` |
| `refactoring-tasks/terminal-components/completion/054/trusted/app-flow-stage-audit.tsv` | 71 | `e91045afc28e2394ec69b9744bba295a610c1d4c5462fb3bb576dfdccc5f8c23` |
| `refactoring-tasks/terminal-components/completion/055/trusted/app-flow-stage-audit.tsv` | 71 | `e91045afc28e2394ec69b9744bba295a610c1d4c5462fb3bb576dfdccc5f8c23` |
| `refactoring-tasks/terminal-components/completion/056/trusted/app-flow-stage-audit.tsv` | 71 | `e91045afc28e2394ec69b9744bba295a610c1d4c5462fb3bb576dfdccc5f8c23` |
| `refactoring-tasks/terminal-components/completion/056/trusted/obligations.md` | 87 | `6b5d66031633a2607914282c11f3ec2130666610f9d654e741f8f7e46d9b5369` |
| `refactoring-tasks/terminal-components/completion/057/trusted/app-flow-stage-audit.tsv` | 71 | `e91045afc28e2394ec69b9744bba295a610c1d4c5462fb3bb576dfdccc5f8c23` |
| `refactoring-tasks/terminal-components/completion/057/trusted/obligations.md` | 663 | `5421f6c2dd7f12ec63e19192d0cd17e08d7426279cdfdfc19d7dd2e564729e28` |
| `refactoring-tasks/terminal-components/completion/058/trusted/app-flow-stage-audit.tsv` | 81 | `6a7c242fde7672184b5632ce4f8b7b191ea025cf0acc2e22f94509724595233d` |
| `refactoring-tasks/terminal-components/completion/059/trusted/app-flow-stage-audit.tsv` | 81 | `6a7c242fde7672184b5632ce4f8b7b191ea025cf0acc2e22f94509724595233d` |
| `refactoring-tasks/terminal-components/completion/059/trusted/obligations.md` | 73 | `77261cc4a2948391dc4f409a85d1442271b2d0cb6d61fa5f40c77f8eca66b6ce` |
| `refactoring-tasks/terminal-components/completion/060/trusted/app-flow-stage-audit.tsv` | 81 | `6a7c242fde7672184b5632ce4f8b7b191ea025cf0acc2e22f94509724595233d` |
| `refactoring-tasks/terminal-components/completion/061/trusted/app-flow-stage-audit.tsv` | 81 | `6a7c242fde7672184b5632ce4f8b7b191ea025cf0acc2e22f94509724595233d` |
| `refactoring-tasks/terminal-components/completion/061/trusted/obligations.md` | 273 | `f6546e3936554c9492c0572c7a10199934e7e6f50e36a42e0232abcc9abd1635` |
| `refactoring-tasks/terminal-components/completion/062/trusted/app-flow-stage-audit.tsv` | 81 | `6a7c242fde7672184b5632ce4f8b7b191ea025cf0acc2e22f94509724595233d` |
| `refactoring-tasks/terminal-components/completion/063/README.md` | 176 | `31531c2b5616b42597791e4b3ecfb9943152ffd326ab112f5a91376c3a4813cd` |
| `refactoring-tasks/terminal-components/completion/063/trusted/app-flow-stage-audit.tsv` | 81 | `6a7c242fde7672184b5632ce4f8b7b191ea025cf0acc2e22f94509724595233d` |
| `refactoring-tasks/terminal-components/completion/063/trusted/obligations.md` | 383 | `f22a6ecd1e45b42fc4860f80f651d2264598ab44c5ed99c14b6f212dea0ee280` |
| `refactoring-tasks/terminal-components/completion/064/trusted/app-flow-stage-audit.tsv` | 81 | `6a7c242fde7672184b5632ce4f8b7b191ea025cf0acc2e22f94509724595233d` |
| `refactoring-tasks/terminal-components/completion/064/trusted/obligations.md` | 833 | `3a2b3b8f8d6fa584cdc1563ea3630d2868ebee3c802ca56da5129a727447724f` |

Repair status: JT-01–04 source scenarios and task051–064 obligation/stage mirrors repaired; JT-05 receives both explicit real-input variants in TP-071 and TASK-063 fixed decision. Shared TASK-017/023 adjudication and global baseline004/005 synchronization belong to coordinator/component owners.

Executed proof: `cargo test --manifest-path /tmp/jt-reaudit.4mpJ2V/Cargo.toml --target-dir /private/tmp/tui-snap-audit.656lHG/oracle-clock/target --lib audit:: -- --nocapture` passes four tests: old TP018 produces no busy node; corrected audit-node path has busy rows after one tick and clears after three; both picker chords paste into hidden editor; commit stays pending through four ticks and finishes on fifth. `--bin jackin-audit audit_` passes two exact-source replay/continuation tests for hundred-role and Inspect boundary classes. All source modules/library used from oracle worktree were independently `git diff --name-only` clean; the TablePro App was copied directly with `git show` because that worktree's App already had unrelated clock instrumentation. No canonical source changed. These six focused proofs establish the repaired defect classes; they do not claim that the future full150-parent expanded campaign has already run.

Line count and SHA256 bind every audited package file below. Repeated content was compared/deduplicated by hash; common AGENTS/verify/README structure additionally compared by task-ID-normalized differences. Inventory is pre-repair evidence, not a claim that future edited bytes have the same hash.

| File | Lines | SHA256 |
| --- | ---: | --- |
| `refactoring-tasks/terminal-components/completion/051/AGENTS.md` | 83 | `989b40d040c9676cbe086d318ee6e21f009ce404bd9b58d7a24fccd652954a77` |
| `refactoring-tasks/terminal-components/completion/051/README.md` | 174 | `21dd1c94e839a5b912bd29a66a71efdeabb27569316e1578d3fdfb57d91b8cbd` |
| `refactoring-tasks/terminal-components/completion/051/task.toml` | 3 | `47e201ba032928fd8badef8bcec2f089e0cfe3a25be1d827b7be5a5bd8e5fad9` |
| `refactoring-tasks/terminal-components/completion/051/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/051/trusted/app-flow-contributions.tsv` | 1 | `b77b050764bf72cfd2b0ce39cf1bafc7fda54fb258f818a87609d5c2f010a8c1` |
| `refactoring-tasks/terminal-components/completion/051/trusted/app-flow-frame-contributions.tsv` | 1 | `be74679510f54598eaff62ebf8271cae441184df9931eaa204a6796b7fab5abd` |
| `refactoring-tasks/terminal-components/completion/051/trusted/app-flow-stage-audit.tsv` | 71 | `c9ac26ce21d7aedad0bfd5a7f43455fe7293896d72e5f7c3af5f1c35806de2f0` |
| `refactoring-tasks/terminal-components/completion/051/trusted/obligations.md` | 114 | `2719a1187da45a4957b83b06f70a2cb50bfbd24f0cfd21b5c32adadd232a8a1e` |
| `refactoring-tasks/terminal-components/completion/051/trusted/source-obligations.tsv` | 20 | `4825b56613dfe8cd74626def2cbeae8bf6243edc2a9c8b85b56b3fd69a6b1766` |
| `refactoring-tasks/terminal-components/completion/051/verify.toml` | 60 | `10f86d4b767563a340b4271ee037686697e099845229ce76ba1e87e2d43f178f` |
| `refactoring-tasks/terminal-components/completion/052/AGENTS.md` | 83 | `057890969169024ee6fd3444793a2cba949612afc18ebebe43b480150cb565c7` |
| `refactoring-tasks/terminal-components/completion/052/README.md` | 174 | `2aaa99850121ade477343f4945e51db9f8a2d20ce0b12e81f3d9b6fc6f7ac072` |
| `refactoring-tasks/terminal-components/completion/052/task.toml` | 3 | `c4c35aadff11ede19d2f23fd3f47abbb11c8bd9fda7843f4afd12780b090a460` |
| `refactoring-tasks/terminal-components/completion/052/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/052/trusted/app-flow-contributions.tsv` | 3 | `161e6e5e7fd8e1c4dbf560ce1e141ecca4a0792a6156aa766b5994045a9a1c2e` |
| `refactoring-tasks/terminal-components/completion/052/trusted/app-flow-frame-contributions.tsv` | 3 | `13b7cc8660ad9b401b1a3c16cc5e5a7380c216cf64694f34c977a9c8c3b49b35` |
| `refactoring-tasks/terminal-components/completion/052/trusted/app-flow-stage-audit.tsv` | 71 | `c9ac26ce21d7aedad0bfd5a7f43455fe7293896d72e5f7c3af5f1c35806de2f0` |
| `refactoring-tasks/terminal-components/completion/052/trusted/obligations.md` | 186 | `4ecb0449d3b300c9caab5dba60fbf5981c2ba00492229f726f2f0272243b45aa` |
| `refactoring-tasks/terminal-components/completion/052/trusted/source-obligations.tsv` | 9 | `7781ce6ce47ce7efe97a7d0dd06749f398f994c5198ffc6ee64d21587539c4f4` |
| `refactoring-tasks/terminal-components/completion/052/verify.toml` | 60 | `cf3e88f64132e0be14315c0f20c8c8ad53a5eeeb445525959edb84c04280980a` |
| `refactoring-tasks/terminal-components/completion/053/AGENTS.md` | 83 | `b25ea6b83429f1cf234b6db6b9988959d7101fc27647fc88b2db0785ee09d2b4` |
| `refactoring-tasks/terminal-components/completion/053/README.md` | 174 | `d1d7ff0cc9e8f2c4613e1349e33dc773b3a86ed04da45142201d9ef906e4300d` |
| `refactoring-tasks/terminal-components/completion/053/task.toml` | 3 | `988597aae0e62eed034585719d85084152da93310c2e01febad67d09ee4e2c57` |
| `refactoring-tasks/terminal-components/completion/053/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/053/trusted/app-flow-contributions.tsv` | 2 | `bc51877ecc165c388dfc44d9c57757257eb3d589646d015b9042bd31e3fbdb49` |
| `refactoring-tasks/terminal-components/completion/053/trusted/app-flow-frame-contributions.tsv` | 2 | `d07958b8fffe41ef92ae450d75cfb6f62fcf9a222d1d2f5f9af3d314be395751` |
| `refactoring-tasks/terminal-components/completion/053/trusted/app-flow-stage-audit.tsv` | 71 | `c9ac26ce21d7aedad0bfd5a7f43455fe7293896d72e5f7c3af5f1c35806de2f0` |
| `refactoring-tasks/terminal-components/completion/053/trusted/obligations.md` | 159 | `358b61adc00618ee94a4e0e7d9d529d2bcc7fc0652994c789bcc6f8c1eec2c66` |
| `refactoring-tasks/terminal-components/completion/053/trusted/source-obligations.tsv` | 8 | `90e9f462033a0519a29a8ae6bdf9cbb463c464071b55fbed429006ea0b519ace` |
| `refactoring-tasks/terminal-components/completion/053/verify.toml` | 60 | `1ef457231e0a94db789633fc2919fcaabde6f47d62bbcdeaad9bcfdea2e8c8da` |
| `refactoring-tasks/terminal-components/completion/054/AGENTS.md` | 83 | `4dd267c9c21429b0a26a2f1751d24c526937f3e80f3c521856173f293a0b5cff` |
| `refactoring-tasks/terminal-components/completion/054/README.md` | 174 | `ba6f76aeafce5d94c81235b9eb7e4f132fcb4860226ba382f062e0e4c777d822` |
| `refactoring-tasks/terminal-components/completion/054/task.toml` | 3 | `5f6e6472a0945ee770351f4196a5349e183d9b45c5c56eca4dd336470d8bb337` |
| `refactoring-tasks/terminal-components/completion/054/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/054/trusted/app-flow-contributions.tsv` | 5 | `764c0ea760793d5d0762eba33469bded220985d873f6c4f54b222d9b99c8e1d2` |
| `refactoring-tasks/terminal-components/completion/054/trusted/app-flow-frame-contributions.tsv` | 5 | `69da3f337bffd45ea1a25a65e4578eb4db5adbed3481719e2cce641d3d9c6754` |
| `refactoring-tasks/terminal-components/completion/054/trusted/app-flow-stage-audit.tsv` | 71 | `c9ac26ce21d7aedad0bfd5a7f43455fe7293896d72e5f7c3af5f1c35806de2f0` |
| `refactoring-tasks/terminal-components/completion/054/trusted/obligations.md` | 96 | `9fc0f1aa01bac7323ddbc1d0f621ba41bf5a19d1856520be9373ac54c29adb3d` |
| `refactoring-tasks/terminal-components/completion/054/trusted/source-obligations.tsv` | 14 | `1bbad63a00c4dcbf5b73f6e7d23c87b66f5595f83e2dcb45519d5f26e4a45d98` |
| `refactoring-tasks/terminal-components/completion/054/verify.toml` | 60 | `26eacbf1167c981f8821da0b62cbc6351e3b7a7b3cb238c433ca0c7ad42635de` |
| `refactoring-tasks/terminal-components/completion/055/AGENTS.md` | 83 | `5d08fd537d79d249c28e95156a8a903fb227698bbfd67f142b15029848558038` |
| `refactoring-tasks/terminal-components/completion/055/README.md` | 174 | `d872c31f6e7f75522a8287f62b405b660e8f7cda39b3d5869f6c5568eb3dbbdc` |
| `refactoring-tasks/terminal-components/completion/055/task.toml` | 3 | `cf3568ff93adad2062b0871911e30335ec418442ed676261ce4be95e73cac71f` |
| `refactoring-tasks/terminal-components/completion/055/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/055/trusted/app-flow-contributions.tsv` | 3 | `333e1474fa148899250a5bbb7d514bf125b6d79768b487410234f9c1a3ef87d0` |
| `refactoring-tasks/terminal-components/completion/055/trusted/app-flow-frame-contributions.tsv` | 3 | `a7315529c4f31d37d55de448934ae916e21ff7da7fae19ba15af76bdd1635efa` |
| `refactoring-tasks/terminal-components/completion/055/trusted/app-flow-stage-audit.tsv` | 71 | `c9ac26ce21d7aedad0bfd5a7f43455fe7293896d72e5f7c3af5f1c35806de2f0` |
| `refactoring-tasks/terminal-components/completion/055/trusted/obligations.md` | 195 | `15936b4a7ca6dd81b5cceaddf30b7b8051fc13976e132456aca62e02935c03f4` |
| `refactoring-tasks/terminal-components/completion/055/trusted/source-obligations.tsv` | 16 | `b2387c0b83eb4df29919eb5a6923ba7a22f8ad560e42620e71afa1feb4b6c5b4` |
| `refactoring-tasks/terminal-components/completion/055/verify.toml` | 60 | `8c89f6602b74f761b8153e50caf80f3e5c47975d3df82b386fb0aef577a9864b` |
| `refactoring-tasks/terminal-components/completion/056/AGENTS.md` | 83 | `fecceb33d9beea7584a13ad6f772ff02d64e5ad5c1388babf4e61ec047f176db` |
| `refactoring-tasks/terminal-components/completion/056/README.md` | 174 | `45bf69629dc0c592459e0bd04fdf0069b2e1dfe26fb3e0be63db9878c1347e8a` |
| `refactoring-tasks/terminal-components/completion/056/task.toml` | 3 | `71e123fead04a15628de1f88f8424750683658ea7d15cf9bd0db5d4ebee6894f` |
| `refactoring-tasks/terminal-components/completion/056/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/056/trusted/app-flow-contributions.tsv` | 1 | `b77b050764bf72cfd2b0ce39cf1bafc7fda54fb258f818a87609d5c2f010a8c1` |
| `refactoring-tasks/terminal-components/completion/056/trusted/app-flow-frame-contributions.tsv` | 1 | `be74679510f54598eaff62ebf8271cae441184df9931eaa204a6796b7fab5abd` |
| `refactoring-tasks/terminal-components/completion/056/trusted/app-flow-stage-audit.tsv` | 71 | `c9ac26ce21d7aedad0bfd5a7f43455fe7293896d72e5f7c3af5f1c35806de2f0` |
| `refactoring-tasks/terminal-components/completion/056/trusted/obligations.md` | 87 | `da852c47a2dbc5bf7b7c006df22f3b30436ad2aed54da9351f36707667511a72` |
| `refactoring-tasks/terminal-components/completion/056/trusted/source-obligations.tsv` | 2 | `1c35daf2283cb318ab46fa9fa1d05f832c8b2098946d449bb0457f31df470853` |
| `refactoring-tasks/terminal-components/completion/056/verify.toml` | 60 | `01f8965354fc48e2c18cfd2ed6c068fc3f2d1bb06243108ee054746a6cc2517b` |
| `refactoring-tasks/terminal-components/completion/057/AGENTS.md` | 83 | `50a54e68e5e498fc27157d1e4b6402ebc83b0d0a5712cede86be2eb06ba1dde5` |
| `refactoring-tasks/terminal-components/completion/057/README.md` | 170 | `653f771011e67e63e1c2a19a352f05d08799dcdc5e883e5849fe1d77c8687e4a` |
| `refactoring-tasks/terminal-components/completion/057/task.toml` | 3 | `e48ea7941886ac5d087a9de1493a695195758561a03b591f7d12bfd963d225cb` |
| `refactoring-tasks/terminal-components/completion/057/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/057/trusted/app-flow-contributions.tsv` | 10 | `b69464241ccfa252034d082cffa3168b347ed6814f656dd79cca5800b0393af5` |
| `refactoring-tasks/terminal-components/completion/057/trusted/app-flow-frame-contributions.tsv` | 10 | `0bfda39160f7bab083c8fe9da2bb24e6328667dc36e56975333d08a95cae7b4a` |
| `refactoring-tasks/terminal-components/completion/057/trusted/app-flow-stage-audit.tsv` | 71 | `c9ac26ce21d7aedad0bfd5a7f43455fe7293896d72e5f7c3af5f1c35806de2f0` |
| `refactoring-tasks/terminal-components/completion/057/trusted/obligations.md` | 663 | `f4b3aa312e8cd347f78c1a893167c4064b455375864d878f76a971aa742853cb` |
| `refactoring-tasks/terminal-components/completion/057/trusted/source-obligations.tsv` | 22 | `1a14c66e36172b300811209f6a60662d6f04429717b8d26b9aa0a156587e1408` |
| `refactoring-tasks/terminal-components/completion/057/verify.toml` | 60 | `a8e839508e3d76848fc25ad4c82e34d10c40c9de3fc27d9d02423c08eebb0d6f` |
| `refactoring-tasks/terminal-components/completion/058/AGENTS.md` | 83 | `285748a673cb7e3ef5b3e3799d90b8c449fe8a0b765d28e07a9c8b6d1b521be2` |
| `refactoring-tasks/terminal-components/completion/058/README.md` | 174 | `2b5de788e77bdb5429684a4def7ed20dd3a532bc97d4a5213f699edbd99ec842` |
| `refactoring-tasks/terminal-components/completion/058/task.toml` | 3 | `e8d7d4fbc329e8c552f4f41c142e6c220753fde9f1122c416391f569f04eafb8` |
| `refactoring-tasks/terminal-components/completion/058/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/058/trusted/app-flow-contributions.tsv` | 3 | `4475727e12a602b6f84694e18bea0ccd405a91f5181780a6719e7e9e6b107a39` |
| `refactoring-tasks/terminal-components/completion/058/trusted/app-flow-frame-contributions.tsv` | 3 | `57aaa4e9ae8c655059c259c85341e5c4918cafc5382b7bec480f73efb861231b` |
| `refactoring-tasks/terminal-components/completion/058/trusted/app-flow-stage-audit.tsv` | 81 | `7fc84c4bea19c71772faca0b840c9cff6a4570cb7cca8853c47bc59d28a0e1db` |
| `refactoring-tasks/terminal-components/completion/058/trusted/obligations.md` | 193 | `41c438382ec62316722517deb549d35f44f5f843e4d8c92c2ce7abd58d1f9af8` |
| `refactoring-tasks/terminal-components/completion/058/trusted/source-obligations.tsv` | 8 | `bcce8ff49c96083b8ad2da5db3e220deec9b93e1b5ccf25a864682f84cb457d1` |
| `refactoring-tasks/terminal-components/completion/058/verify.toml` | 60 | `531d22c5f31d082aa40f5a486cbaaf994a54079db49e58cbbafad14fa47207d8` |
| `refactoring-tasks/terminal-components/completion/059/AGENTS.md` | 83 | `c748cd90f199617c93fd26d4026d3ed17a49ebb1e4e94f9bbd01209faa55dc4f` |
| `refactoring-tasks/terminal-components/completion/059/README.md` | 174 | `8c078d46c307fc2f59401d64b801a9e0304b78e28d26e10312eaffdfc878b85f` |
| `refactoring-tasks/terminal-components/completion/059/task.toml` | 3 | `1e8803bc7bcd1675843bbf2e2cee9f349aa1ee16141d19ba9421330a0fa15da9` |
| `refactoring-tasks/terminal-components/completion/059/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/059/trusted/app-flow-contributions.tsv` | 5 | `df790a3ff62dcffa5d655a78550b8fe7f47235a28bf86fe66cff01256761255e` |
| `refactoring-tasks/terminal-components/completion/059/trusted/app-flow-frame-contributions.tsv` | 5 | `0543340fd396b3569ff52b34b0b19eaae6818622263c15f5bbdf86fbeed29e28` |
| `refactoring-tasks/terminal-components/completion/059/trusted/app-flow-stage-audit.tsv` | 81 | `7fc84c4bea19c71772faca0b840c9cff6a4570cb7cca8853c47bc59d28a0e1db` |
| `refactoring-tasks/terminal-components/completion/059/trusted/obligations.md` | 73 | `c42d313d246cf7b7ba0e4c20366e118fe9dee96449e9e22435a2e08a2052caf7` |
| `refactoring-tasks/terminal-components/completion/059/trusted/source-obligations.tsv` | 10 | `70e6f4e2579665e57f407b4c1f36d2f3be7d49d1bd4bce8fe889060baa54eabf` |
| `refactoring-tasks/terminal-components/completion/059/verify.toml` | 60 | `15d6e8055d463082514a9106d04445d66a283792cecf1307709a10b4278301eb` |
| `refactoring-tasks/terminal-components/completion/060/AGENTS.md` | 83 | `1292099e7ab88ba292f2b2e0e2def52e615d478b7d51175037647f8dbb5433c4` |
| `refactoring-tasks/terminal-components/completion/060/README.md` | 174 | `4338fc3aa2d36c2eb003133fd9bab9224b8ed34070afbdfdc5ac1c63df98206d` |
| `refactoring-tasks/terminal-components/completion/060/task.toml` | 3 | `a5d2b66ee586a026d8d4d6245d1f2debb09fdc903432c726970f6364940d9054` |
| `refactoring-tasks/terminal-components/completion/060/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/060/trusted/app-flow-contributions.tsv` | 20 | `16e85fd6fd5bb37de0b74726e6f1bfb16427272d88375276eff0d7257b42b20f` |
| `refactoring-tasks/terminal-components/completion/060/trusted/app-flow-frame-contributions.tsv` | 20 | `1d8e518ac34132aecb43901c694f99659ef7037041a401fdce89ea7f9a1ae576` |
| `refactoring-tasks/terminal-components/completion/060/trusted/app-flow-stage-audit.tsv` | 81 | `7fc84c4bea19c71772faca0b840c9cff6a4570cb7cca8853c47bc59d28a0e1db` |
| `refactoring-tasks/terminal-components/completion/060/trusted/obligations.md` | 223 | `534a4b2e014789038616f43afd787810c0e2be29b8b52082450f7d985d565f77` |
| `refactoring-tasks/terminal-components/completion/060/trusted/source-obligations.tsv` | 55 | `41985919ca164bee593bc5e69bb0090a8e197db0e9055b6bd65144d43125a5e9` |
| `refactoring-tasks/terminal-components/completion/060/verify.toml` | 60 | `c289ec6ca0c3f840d950a456aefb78adf458f1020f3c68af8f47ed6e2bfce96b` |
| `refactoring-tasks/terminal-components/completion/061/AGENTS.md` | 83 | `dab6a4a619e1ccc91bea7e1adc80bdcb849df66bf6a4d532fb6c953ce779a8b6` |
| `refactoring-tasks/terminal-components/completion/061/README.md` | 174 | `f5c4594c27f8d09811eef4358cf5514868ddd2a06f9ce52348a3aa3414748dac` |
| `refactoring-tasks/terminal-components/completion/061/task.toml` | 3 | `67ec9997ea32b36ffa5de9a350c43a944a789de540f6a777b2e0db624eb43c28` |
| `refactoring-tasks/terminal-components/completion/061/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/061/trusted/app-flow-contributions.tsv` | 3 | `3fb0699e73bc6fa42e5dbb4d29e50a62c9e2c930a778c008c0dfdca005424d10` |
| `refactoring-tasks/terminal-components/completion/061/trusted/app-flow-frame-contributions.tsv` | 3 | `864f164c94ade79605ebb8e197ed0566d51d40db9a143a23b3d719facafc386b` |
| `refactoring-tasks/terminal-components/completion/061/trusted/app-flow-stage-audit.tsv` | 81 | `7fc84c4bea19c71772faca0b840c9cff6a4570cb7cca8853c47bc59d28a0e1db` |
| `refactoring-tasks/terminal-components/completion/061/trusted/obligations.md` | 273 | `04d4e6ac36b7aa1609704add6dc26f71eaaf6b3f7b9dfa5cf6d751abdf59090c` |
| `refactoring-tasks/terminal-components/completion/061/trusted/source-obligations.tsv` | 5 | `6558229163c8cd174662f83d8d3d7b747a357862c3335ef7cac8ef91a71e8012` |
| `refactoring-tasks/terminal-components/completion/061/verify.toml` | 60 | `432890ea5d444efb62c186f31724f9dd1a7a59e081e164f16618acdfbe249036` |
| `refactoring-tasks/terminal-components/completion/062/AGENTS.md` | 83 | `4d8cde594f58e6e97bcf961faab251a52affaeb80875797a099ffabb7831c7de` |
| `refactoring-tasks/terminal-components/completion/062/README.md` | 174 | `573759d6829235cc20425b993b330bce4f8b831947ae182d2e2feb78c0588ff4` |
| `refactoring-tasks/terminal-components/completion/062/task.toml` | 3 | `b7c16a0b1626dec5fa01c6c8642063b3c614838206bc5902473a92259f808c61` |
| `refactoring-tasks/terminal-components/completion/062/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/062/trusted/app-flow-contributions.tsv` | 7 | `ef5079d7b5f58be59e01e3392cbfe2f8c2dad7e1eb2dcb12abe7791c480cfc7f` |
| `refactoring-tasks/terminal-components/completion/062/trusted/app-flow-frame-contributions.tsv` | 7 | `f3124f7ffa248b0bc8a4e93de88ff65f9d38b8b5dda8efaaf11240291b1a07c3` |
| `refactoring-tasks/terminal-components/completion/062/trusted/app-flow-stage-audit.tsv` | 81 | `7fc84c4bea19c71772faca0b840c9cff6a4570cb7cca8853c47bc59d28a0e1db` |
| `refactoring-tasks/terminal-components/completion/062/trusted/obligations.md` | 153 | `6dd160be7c5ab030837a753328d13889acfc0a14497a4dbbe0fc352f3d688a6a` |
| `refactoring-tasks/terminal-components/completion/062/trusted/source-obligations.tsv` | 13 | `56713cabfaf0b95ecdfadcc57c3b5144aaeea890863c8dad5e1c6713582e9109` |
| `refactoring-tasks/terminal-components/completion/062/verify.toml` | 60 | `f887ebd29ef63a662aae0186e60c0ba094226093ceb1732c89a438fdf5cefa2b` |
| `refactoring-tasks/terminal-components/completion/063/AGENTS.md` | 83 | `dcb0ceb0ee833bdb7ee48725a41fab55376d63dcd400745e72a6fc422e7d3317` |
| `refactoring-tasks/terminal-components/completion/063/README.md` | 174 | `46b7c1d6971999131c7d9287b6e8e76fd81e1da56224a6aaef5c0ce905fbfd38` |
| `refactoring-tasks/terminal-components/completion/063/task.toml` | 3 | `05c3063e6dd9687c7e8bace587326b90f0c744d8918737d53b3cb9c7ea6cd3b3` |
| `refactoring-tasks/terminal-components/completion/063/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/063/trusted/app-flow-contributions.tsv` | 1 | `b77b050764bf72cfd2b0ce39cf1bafc7fda54fb258f818a87609d5c2f010a8c1` |
| `refactoring-tasks/terminal-components/completion/063/trusted/app-flow-frame-contributions.tsv` | 1 | `be74679510f54598eaff62ebf8271cae441184df9931eaa204a6796b7fab5abd` |
| `refactoring-tasks/terminal-components/completion/063/trusted/app-flow-stage-audit.tsv` | 81 | `7fc84c4bea19c71772faca0b840c9cff6a4570cb7cca8853c47bc59d28a0e1db` |
| `refactoring-tasks/terminal-components/completion/063/trusted/obligations.md` | 383 | `cade65fff56388ea41a227312edd4d2050743657eaf8c0d9004e36f7ea10f67c` |
| `refactoring-tasks/terminal-components/completion/063/trusted/source-obligations.tsv` | 1 | `3c79871023bc92c8cffd6e06a375b7a7deae7fa16e333e43a73b7b48f9554a6f` |
| `refactoring-tasks/terminal-components/completion/063/verify.toml` | 60 | `c63a990dcc2947a326cb9a22712e15a9ae95f7ed9441e5eb9ecdb99e0c7f1595` |
| `refactoring-tasks/terminal-components/completion/064/AGENTS.md` | 83 | `e4a1caf2e67dd8f69f0d29c4801a1e24ba2c29778b7567005c6a7487023a6548` |
| `refactoring-tasks/terminal-components/completion/064/README.md` | 170 | `5dea30af463a78e92c040adabd5154b714125b88f9a8057db239bfa32760f8b1` |
| `refactoring-tasks/terminal-components/completion/064/task.toml` | 3 | `432afbe67b847ff308c60e8705af49eedfde0ead82de5eb5665e5bb739d6aeb3` |
| `refactoring-tasks/terminal-components/completion/064/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/064/trusted/app-flow-contributions.tsv` | 34 | `2960a85d76b58b8b2e0b942ca083780d04dbe3044a3dde665b6463d98c04221c` |
| `refactoring-tasks/terminal-components/completion/064/trusted/app-flow-frame-contributions.tsv` | 34 | `eaa5c2336bde4cf711c2e4126860136b17dd714a3f21e93275f73d587012c707` |
| `refactoring-tasks/terminal-components/completion/064/trusted/app-flow-stage-audit.tsv` | 81 | `7fc84c4bea19c71772faca0b840c9cff6a4570cb7cca8853c47bc59d28a0e1db` |
| `refactoring-tasks/terminal-components/completion/064/trusted/obligations.md` | 833 | `b5374f6549c83ecee182792ce57eabb27703191ff753d24eb55fcaaab8fe04b7` |
| `refactoring-tasks/terminal-components/completion/064/trusted/source-obligations.tsv` | 70 | `b94d7e1c7d5ba10d72cca72d506be3229697a33645d05928c73be325ecb7abdd` |
| `refactoring-tasks/terminal-components/completion/064/verify.toml` | 60 | `a3f9254c9d04809af7d97c230ae0d3154e026c18c5651c49b6635cc0d5963d97` |
