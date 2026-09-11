# Independent rereview: component-a repairs

## Scope and authority

This is an independent rereview of the current TASK-009–020 repair contracts, not an implementation or a claim of complete component/application parity. The reviewer did not author these component repairs. PLANNING_GOAL.md was read in full. Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c` and main `7b27732a8c3c131760ec3438f641cb3c11343a42` remain distinct authorities. `git show` and detached source reads were used instead of trusting the original audit's conclusions.

Only this report is writable in the repository during this rereview. Disposable Rust diagnostics live under `/tmp`; no product source, canonical AGENTS, expected frames, golden baseline, task package or architecture adjudication was edited by this reviewer. The verify-and-stop skill bounded verification; automatically selected TUI policy supplied no authority to bless or mutate a frame.

Final status: the inspected repair contracts pass this scoped independent rereview at the bytes inventoried below. Every concrete finding below was repaired by its owner and the changed text was re-read. This is not approval of all component parity, every historical clause's implementation, the global plan, or the still-future verification tools. Root owns global history/matrix synchronization.

## Independent findings and dispositions

### RRA-01 — Form radio policy lacked the writable private bridge

- Source: main `crates/tui/src/components/choice.rs:980–997` writes a Form value only for `RadioGroupAction::Chose`; `components/form.rs:1124–1126` consumes that private bridge and exhaustively maps its action. Original rereview TASK-019 `verify.toml:3` allowed only form.rs and its completion test.
- Counterexample: add Navigated in TASK-018, then attempt TASK-019's promised policy through the existing private bridge. The value mutation branch is outside TASK-019's allowed source scope. Implementing app-side cursor lookup instead would duplicate the shared engine.
- Root condition: additive API planning accounted for the public action consumer, not the private mutation owner.
- Repair re-read: TASK-019 now owns choice.rs narrowly and freezes `Form::radio_navigation_commits(self, enabled: bool) -> Self`, default false. It passes the policy through the private bridge; no FieldControl widening. Accepted commit operations include equal-value repeats, not only inequality. TASK-018 remains its prerequisite.

### RRA-02 — TextInput paste eligibility incorrectly inherited the main regression

- Source: oracle `src/widgets/input.rs:237–249` unconditionally calls begin_edit and live_validate for an enabled paste, then returns Changed. `begin_edit:158–165` snapshots only on idle entry. Main `components/input.rs:1214–1221` instead requires an already-editing state and validates only when the edit core reports changed bytes.
- Counterexample: idle enabled input, including an empty paste, must enter edit; an already-present validation error is checked even if paste changes no bytes. The old planned helper's `editing` guard/no-begin instruction denied the oracle event. Root independently found the idle case; this rereviewer identified and executed the empty-payload/validation consequence.
- Root condition: a source-qualified application escape route was specified using current-main insertion guards rather than the source widget's complete admission/lifecycle algorithm.
- Repair re-read: TASK-016 begins idle TextInput from the caller-controlled value, establishes sensitivity before copying, rejects readonly/disabled/inherited-disabled, preserves an existing edit/selection, validates and requests repaint on empty admitted paste, and does not synthesize a commit. TextAction::Changed remains a typed draft mutation, while invalidation represents edit-entry/no-byte-change work. Idle TextArea/CodeEditor document paste remains a separate, guarded case.
- Required negative proof: idle empty/nonempty, active replacement, empty paste with prior error and callback count, disabled rejection, exact first snapshot restored by cancel, changing sensitivity before entry, no secret exposure, runtime/programmatic single delivery, and true-modal isolation.

### RRA-03 — No-Cx paste equivalence must distinguish model state from displayed viewport

- Source: main `components/input.rs:1243–1248` reconciles stored hscroll from `Cx::area`; `ui/cx.rs:625–627` obtains that area from the last hit registry. `ui/mod.rs:460–462,733–745` suppresses inert/reference registration. Therefore a hidden component cannot rely on an invented area appearing during ordinary Settle.
- Counterexample: paste a line longer than the underlying field while the menu remains open, then resize before closing. Waiting until reveal would allow an incorrect first visible background frame; an immediate helper-only stored-hscroll equality promise would require geometry the frozen no-Cx API does not possess.
- Existing structural solution verified: `components/input.rs:1420–1436` already derives the effective painted hscroll from the current draw allocation and cursor without mutating durable state. The shared component must retain that pure current-allocation viewport projection. No public fake/stale geometry, synthetic focus, hidden runtime intent or app scrolling engine is required.
- Executed witness: actual main TextInput::draw under Ui::reference(None), a long draft, no update calls, nonzero origin, widths 40→10→30→6. First frames expose the current tail and left ellipsis; edit/draft remain unchanged. This proves the mechanism exists, not that Holla's entire frame already matches. Full source-qualified app capture must compare the first menu-open/picker-open frame and resize while still open, not just a later reveal.

### RRA-04 — New ChipBar actions require the actual exhaustive app caller

- Source: main `apps/showcase/src/pages/chips.rs:269–274` exhaustively matches only Toggled/Activated/Closed/AddRequested. `crates/tui/tests/conformance.rs:1676–1683` is the other exhaustive non-widget consumer.
- Counterexample: TASK-020 adds LeadRequested/ClearRequested and repairs only conformance. Workspace compilation fails in the existing Showcase page before its TASK-037 migration. A future app task cannot retroactively make this component task buildable.
- Root condition: enum-extension scope was derived from library consumers only.
- Required structural repair sent to author: narrow compile-only action adaptation authority for the existing Showcase chips file in TASK-020, with source callback/domain migration still owned by TASK-037 and DAG serialization retained. No original match behavior or test identity may be deleted.
- Final repair re-read: TASK-020 README scope and verify.toml now include only the needed Showcase match adaptation; its fixed contract preserves all four existing mappings and records new placeholder outcomes as diagnostic until TASK-037. TASK-020 is an actual ancestor of TASK-037.

### RRA-05 — Chip overflow focus is an observable early-return effect

- Source: oracle `src/widgets/chips.rs:211–215` returns on first nonfitting chip; the one group focus registration occurs only at the end of render. Earlier visible chip/lead hits can exist while the group is absent from the next focus ring.
- Counterexample: a narrow strip fits the first chip but overflows the second. An unconditional modern one-focus-stop registration changes traversal despite matching the visible ellipsis.
- Required repair sent to author: specify source clipped-strip focus eligibility and execute the partial-hit/no-ring trajectory; do not promote the introductory “one focus stop” description into a universal assertion. The lead's unbounded old Buffer write also needs an honest source-applicable width/clipping disposition, not a silently redesigned full-cell oracle.
- Final repair re-read: TASK-020 and ADJ-11 explicitly retain visible hits but omit group registration at early overflow, then restore it on a fitting frame. Tiny containment is an architectural lane, not a fabricated oracle frame for the raw lead overflow. The ordinary generic scrolling policy retains its separately tested focus behavior. A real oracle width-10/40 diagnostic confirms the exact partial-hit/no-ring versus fitting-ring result.

### RRA-06 — Invalid input commit was conflated with blocked Form submission

- Source: oracle `src/widgets/input.rs:169–172` ends editing and validates; its Enter branch returns InputEvent::Committed even when validation fails. Main `components/input.rs:745–753` writes the caller-controlled value before validation. TASK-016's introductory contract and new W-016-04 instead said validation denial blocks commit/navigation.
- Executed counterexample: required empty input, begin edit, Enter. Actual oracle returns Changed plus Committed, ends editing, and records `Required`. A planned component that refuses the commit changes source behavior. Invalid Form/domain submission is a separate operation with a separately owned gate.
- Root condition: validation result, edit commit and domain submission were collapsed into one ambiguous “commit” condition.
- Required structural repair sent to author/root: preserve each input/textarea commit's source action/value/edit/error sequence; bind denied Form submission and its focus/reveal behavior to the actual Form/domain owner instead. Freeze invalid Enter and Tab paths independently; do not infer a validation veto where the source only records an error.
- Final repair re-read: TASK-016's fixed validation disposition and W-016-04 preserve invalid TextInput commit, error recording and committed-tab semantics. Oracle CommittedTab is explicitly not a new modern enum variant. Main's MoveNext/MovePrev are reserved (`input.rs:885`); normal runtime traversal and configured FocusOut commit provide the modern Tab path. TASK-019 separately owns failed Form submission.

### RRA-07 — Shared painter witness must not require future consumer receipts

- Counterexample: the first W-013-07 placed Ui, RowUi and viewport migration plus TASK-015/021 receipts in TASK-013's mandatory witness set. Those future tasks cannot become an implicit prerequisite of their own producer.
- Final repair re-read: W-013-07 now closes the current shared projection/public Ui painter path only. RowUi and LineRef remain mandatory TASK-015/021 cases joined later by TASK-031/069. The twelve witness headers distinguish producer-owned direct gates from complete application diagnostic frames and forbid future consumer receipts becoming an earlier gate. No case or later closure requirement was deleted.

## Stable repaired contracts rechecked

| Owner | Semantic rereview and enabling condition removed |
| --- | --- |
| 009 | Direct optional cfg(unix) signal-hook `=0.3.18`, default-features=false, backend-only feature edge, manifest/lock authority and runtime.rs scope. Unbounded read plus a flag is explicitly rejected; service timeout is min(100 ms, earliest real deadline), with overdue/input-flood cases and no fabricated logical Tick/Settle/draw/feedback work. Broker is process-lifetime, not unregister-as-restoration; supported default/no-competing-handler environment is explicit. Partial startup/re-entry/cleanup, inactive post-session default stop, repeated sessions and pending-signal phases require real PTY plus deterministic probes. |
| 010 | Publication-owned capture cancellation and twelve existing eligibility tests are preserved. CP-COMMON's narrow covered-FIELD amendment does not weaken focus, pointer or inert ownership. No fresh complete capture-parity claim. |
| 011 | TASK-073 alone owns real style timing; 011 preserves the seam and no second clock/numerator/denominator/Theme timing state. Painter overlap is serialized before 013. |
| 012 | The CP-COMMON amendment is confined to actual covered FIELD style resolution; pure measure/empty geometry, component clipping and current hit allocation remain separate requirements. |
| 013 | Actual main paint_spans per-fragment segmentation owner is writable, with 011→013→014 order. Styled fragments form one logical line before segmentation; first-byte style matches oracle viewport.rs:19–23. Shared borrowed projection, original offsets/copy and counted cluster scratch reject full-document clones or a helper that leaves production paint_spans unchanged. |
| 014 | Depends on 013 and 011 before editing shared paint.rs; fade geometry/writer obligations cannot race the cross-span repair. |
| 015 | RowUi/CellUi consumers explicitly use 013's logical run and provenance; their collection-owned scope is sufficient for the consumer repair. No per-component suffix patch. |
| 016 | Exact public paste signature, sealed generic target, one EditAction::Paste engine, sensitivity-before-copy, controlled-value commit lifecycle, source idle TextInput admission and separate viewport projection. FIELD resolution exception is not an ignored slot exemption. Invalid input commit is separate from Form submission. See RRA-02/03/06. |
| 017 | Default runtime modal/inert capture remains intact. Source-qualified app dispatch is an explicit model command, not LayerSpec passthrough or hidden input delivery. TablePro helper owner is 025, Holla TextInput helper owner 016. Recipient/nonrecipient, focus, menu/query, validation and first-frame equality are separate source obligations. |
| 018 | Oracle fixed marker/label geometry replaces the unresolved bracket test-name promise. Navigated is additive, includes clamped admitted arrows, and is emitted at binding branches/origin-aware helper—not blindly from move_cursor also used by Pointer::Press. Conformance and Form exhaustive adaptation are writable; uncommitted Navigated is erased using existing Response::take_action preserving flow/invalidation/id/state. |
| 019 | Explicit opt-in Form policy consumes shared typed events in its private bridge; defaults remain activation-only. Repeated same-key operation can commit/report, while seed/reconcile/remove/Press cannot. RRA-01 scope repair re-read. |
| 020 | Borrowed per-item checked/removable/error data, lead and nonreserved/non-scrolling composition use one existing ChipBar. Keyless Lead/Clear/Add have no sentinel item key. Modified character versus Enter/Delete source guards are distinguished; providers cannot be cached as durable domain values. Pure property callbacks are expressly allowed; external provider I/O is not. Compile/focus repairs are verified in RRA-04/05. |

ADJ-04 was checked against main COMPONENT_ARCHITECTURE.md:6799–6815,6997–7003: selected component-owned FIELD Resolved may differ with unchanged composed digest, preserving the accepted ebfa8b8 case. This does not authorize an ignored replacement slot, invented measurement query, missing PARTS entry, public patch_part conformance hook, or fake owner ID. ADJ-09 preserves deferred F01/F04 as source-qualified compatibility, not an inferred product fix; its corrected lifecycle distinguishes idle TextInput, idle TextArea/document CodeEditor and CodeEditor find-query priority. Final ADJ-10/11 were read completely and match the explicit RadioGroup/Form policy and borrowed ChipBar geometry/focus contracts above. The Radio group title can remain shared Form chrome (main form.rs:1605–1611), not a second app-local painter or an invented required generic RadioGroup title API.

## Executed evidence

1. Canonical `rtk proxy taskfmt --config /Users/donbeave/Projects/donbeave/task-format/experiment.toml lint <absolute completion/009…020 path>`: twelve packages PASS, zero errors/warnings, re-executed after the final repairs.
2. `/tmp/ca-rereview.vWUTyh`: actual pinned-main public Scene/Ui APIs, backend-free dependencies. `cargo test --offline --manifest-path .../Cargo.toml --target-dir /tmp/tc-adjudication-target -- --nocapture`: two tests PASS. Split `e`/combining-mark spans lose the mark in old paint_spans while unsplit text retains it; this is a negative witness for the old mechanism, not implemented repair proof. The inert current-width TextInput projection test is described in RRA-03.
3. `/tmp/ca-oracle-rereview.2t1QAH`: actual oracle library widgets; the referenced widget/core/library files have an empty git diff against the oracle pin. Four tests PASS: all four clamped Up/Down/j/k operations, same-key repeated Changed and disabled rejection; idle empty paste entry/validation, active replacement and original snapshot restoration; invalid TextInput commit; and chip overflow retaining earlier hits but losing the group focus entry. The surrounding oracle diagnostic checkout has unrelated app clock instrumentation, not modifications to these exercised widgets.
4. Parsed actual task metadata: 73-task graph is acyclic; 011 precedes 013, 013 precedes 014/015, 018 precedes 019/020, and 020 precedes 037. Writable scopes include every newly discovered private bridge or exhaustive consumer. This checks the new repair ordering, not the whole global execution protocol.

These temporary crates use their resolved diagnostic dependency locks, not a claim of full pinned-workspace/MSRV gate execution. No generated frame was made a golden. Safe signal lifecycle, future helpers, new enum implementations, full source app trajectories, full conformance sweep and protected tc-proof gates remain future execution obligations. Existing source behavior and a bounded counterexample are not substitutes for them.

## Final checkpoint and limits

All 106 finite source-witness rows across TASK-009–020 were inspected, including their shared expansion/checkpoint rules and new API/geometry/action cases. Counts identify coverage, not execution proof. Existing historical source ledgers remain additional requirements, not replaced by the new rows. No further material contradiction was found in the repaired contracts inspected here.

The signal broker remains a future TASK-009 implementation: neither author nor rereviewer executed a disposable broker, and no live signal/termios success is claimed. The fixed safe API/environment/lifecycle constraints, explicit due/overdue deadlines and pending-signal checkpoints are preparation contracts requiring later real PTY and mutation proof. Full component parity, all application frames, masked Unicode projection, full conformance/slot census, baseline sealing and protected tc-proof execution were not executed by this rereview. Root's final global synchronization and independent campaign acceptance remain separate.

The inventory below identifies 84 task-package files plus the adjudication file at the final rereview checkpoint. SHA equality is byte identity only. README/verify/task metadata were checked for contract/scope/DAG linkage; obligations and the 106 new finite witness rows were checked semantically as described above. Canonical AGENTS and retained source ledgers were not rewritten by this reviewer.

| Path | Lines | SHA-256 |
| --- | ---: | --- |
| refactoring-tasks/terminal-components/completion/009/AGENTS.md | 83 | 0260a43a13384226baf2bdf479776ada61c46f4e899007e47400cfb931d69706 |
| refactoring-tasks/terminal-components/completion/009/README.md | 182 | ae6084c411481a10f8bd1fb8dd35958f680df5c1f0c7b601b3d8e5670076d040 |
| refactoring-tasks/terminal-components/completion/009/task.toml | 3 | b372dc7e92a11276638daf268bf752716857f34bedb19f70a940587e1be58a56 |
| refactoring-tasks/terminal-components/completion/009/trusted/obligations.md | 327 | 3927b1ec2d4166cbbb7b4e573a174a4380d149b5a4e0e7a3d97f43976383134b |
| refactoring-tasks/terminal-components/completion/009/trusted/source-obligations.tsv | 26 | 1bab831c56f863e1d7aa3672544162a092106937d2a97cf90ab9bd26906d0b53 |
| refactoring-tasks/terminal-components/completion/009/trusted/source-witnesses.md | 23 | 7115206a46b6f435e6f5bfe7aa33edc271152ac4500826da572c9a7f1f272674 |
| refactoring-tasks/terminal-components/completion/009/verify.toml | 60 | e5de5be67f86350dac4daea6367e11b54f324a6def6fb01d70b996c2d47df953 |
| refactoring-tasks/terminal-components/completion/010/AGENTS.md | 83 | c77cf1534cb327240a294a9c3c8a6708a2b56474fca04307ef7ec9c07d93c102 |
| refactoring-tasks/terminal-components/completion/010/README.md | 173 | a92c31d05b03c256c30ea0ff294cebd0672984b7d195b47383874aae65a3ff72 |
| refactoring-tasks/terminal-components/completion/010/task.toml | 3 | 6d2315891237c41818a11094e19ee960fa474059187584d660c5f80b790a0740 |
| refactoring-tasks/terminal-components/completion/010/trusted/obligations.md | 830 | 8f366fbc296e459d34ff2b0987f55282f2b62626eee88d9eec89ea2f5cf30538 |
| refactoring-tasks/terminal-components/completion/010/trusted/source-obligations.tsv | 80 | d6ad5604bb510fa5d24afb8c7c46dd940a71d88f1ed617c2a821e4e2fd60b4f1 |
| refactoring-tasks/terminal-components/completion/010/trusted/source-witnesses.md | 23 | 659f407cd90cbaa766d55c186c2e24bf702ac381a763fb0dadbb8ff587afaa0c |
| refactoring-tasks/terminal-components/completion/010/verify.toml | 60 | f69bba5dbfd8d21076d2ae3fd1332a6be93ba70e378fd75455aa00df4d980d51 |
| refactoring-tasks/terminal-components/completion/011/AGENTS.md | 83 | 46713a1dda4a827090c6d01e8b8a6efd3c8873f4f57770452f09024bc37552aa |
| refactoring-tasks/terminal-components/completion/011/README.md | 172 | c259fdece783ba4c1277cb157f5f5d539cc0215137e4ea9e039db789fc1a3ee4 |
| refactoring-tasks/terminal-components/completion/011/task.toml | 3 | b372dc7e92a11276638daf268bf752716857f34bedb19f70a940587e1be58a56 |
| refactoring-tasks/terminal-components/completion/011/trusted/obligations.md | 625 | 062394d393178cce9868e0921ab13e6df02112261b7b547d6dde6ff2150cad54 |
| refactoring-tasks/terminal-components/completion/011/trusted/source-obligations.tsv | 64 | 3ba092cb1adf94d54cef0e434066e2866c32266ffedee45f0bca7e6fb4d5c718 |
| refactoring-tasks/terminal-components/completion/011/trusted/source-witnesses.md | 22 | 580559cbe7c220feaefd68e42118eb808bbf94feed97576d398a0e057d232b27 |
| refactoring-tasks/terminal-components/completion/011/verify.toml | 60 | 9fbbe0afc81893cd7427197169e17e9ae47a4d013736286cdba9e77d3715c821 |
| refactoring-tasks/terminal-components/completion/012/AGENTS.md | 83 | a836350399d2bdf8368c739b1bd785a955a917094b9c72e3e5feb68898d3bfcb |
| refactoring-tasks/terminal-components/completion/012/README.md | 171 | 8cb94c71ed345997be892a08835d8bebbf3aca85de8ceb24186a2e443f1e3fec |
| refactoring-tasks/terminal-components/completion/012/task.toml | 3 | b372dc7e92a11276638daf268bf752716857f34bedb19f70a940587e1be58a56 |
| refactoring-tasks/terminal-components/completion/012/trusted/obligations.md | 264 | 3ba4442405971a79b8d5ef13a491e0d7d2eaf44eea94ed972706c90dfd813b6d |
| refactoring-tasks/terminal-components/completion/012/trusted/source-obligations.tsv | 23 | acdcb581c9acacbd6fd0b69b0ebaed98c705dc95bc4e8be4f97626111ed94db7 |
| refactoring-tasks/terminal-components/completion/012/trusted/source-witnesses.md | 22 | fe5a7dc4b4d897d7e2f4afb2b868868347db350520621dc0de8f1089452c0d7e |
| refactoring-tasks/terminal-components/completion/012/verify.toml | 60 | 91f6d69e8f152d64a1f026fc12e9043af17643750969310b13f6cd0dd6c638dd |
| refactoring-tasks/terminal-components/completion/013/AGENTS.md | 83 | bd2278cdae126f3cf5b3b8a6b151d47134d198262d1f8878021315a66513bd55 |
| refactoring-tasks/terminal-components/completion/013/README.md | 173 | 068f704e02a8179986a95b1ec8be85107d745ce15d8504642a76b190f3079c10 |
| refactoring-tasks/terminal-components/completion/013/task.toml | 3 | 38e66c957c340f66d716ab9dd24870a70b15b09ff7673b2615b6584c8a3b8afe |
| refactoring-tasks/terminal-components/completion/013/trusted/obligations.md | 174 | d61628a07e2a74917c54481a8dde1a6a300d1ad5c92beb4916d8280fd9d8e100 |
| refactoring-tasks/terminal-components/completion/013/trusted/source-obligations.tsv | 12 | 4cb418248b0e6211e52d37e215e7edb15df21472c34d29c9f6c3c6daaa1e99b2 |
| refactoring-tasks/terminal-components/completion/013/trusted/source-witnesses.md | 22 | 0f6c754e0c869280624f9d6e31e22d718e982b3ba42a26169283f9c33922a2d8 |
| refactoring-tasks/terminal-components/completion/013/verify.toml | 60 | 51f1e740cc11ccfe7ad57991e20a9ba91cf46da90690f676ebfeb677663b5a4b |
| refactoring-tasks/terminal-components/completion/014/AGENTS.md | 83 | 091a0ee40e9ea43163c9dfe19ddde0d45a47c5a2b6bcb42869fad2cebb3e7841 |
| refactoring-tasks/terminal-components/completion/014/README.md | 172 | 26e40acfb6bc51434d54bdc139d043fd986b04d6a6c70bd3ecd37bade665d058 |
| refactoring-tasks/terminal-components/completion/014/task.toml | 3 | 1ef5fe1b675207e208bf7a33a9c54b6f0bf052fddb60e26b788a883dfbcbef18 |
| refactoring-tasks/terminal-components/completion/014/trusted/obligations.md | 180 | 18727afd2bef1d78dd3aac33cfc5a0b58266e848147e1a640ec7c79d5b3f4caf |
| refactoring-tasks/terminal-components/completion/014/trusted/source-obligations.tsv | 15 | 6d16c5bcc79f859b5da92b259f9fafaf27cabc23128a3d0f6ea27b9857776e7f |
| refactoring-tasks/terminal-components/completion/014/trusted/source-witnesses.md | 22 | 7cc3e1ef73327d79330a3db5061d6a8ad6ad35265699a2011acd4132bf3363d4 |
| refactoring-tasks/terminal-components/completion/014/verify.toml | 60 | 13c7e931025ac5f19b6bb68001e6e9f229a7d17d22e8112e7123720fa2df7bcc |
| refactoring-tasks/terminal-components/completion/015/AGENTS.md | 83 | ab57edeb75d7638274746b2153e65c47ecfa5a5dc7f1ad3073a259d31a96690e |
| refactoring-tasks/terminal-components/completion/015/README.md | 171 | 39a7a794912f08eec1e84e2c43bd9997799984ca5e5b687bde957a2e059910e9 |
| refactoring-tasks/terminal-components/completion/015/task.toml | 3 | 1a03ee7d14b3d32f83a629256f0c79f51c6f9d0a000eba43c82f8e7a6218739b |
| refactoring-tasks/terminal-components/completion/015/trusted/obligations.md | 330 | f78b6061b5d9a1f014ac63395f3ed2ab46e8277e49f579761c0c30d9a4d10019 |
| refactoring-tasks/terminal-components/completion/015/trusted/source-obligations.tsv | 27 | 0e3d0df21c160e3f8074d3fe53201392bdda9855bdadc15293f551d64c52c19b |
| refactoring-tasks/terminal-components/completion/015/trusted/source-witnesses.md | 21 | 3f766f555efb75a5df8b3b475418ccad495d3d4569ce928fcf783a7e42281606 |
| refactoring-tasks/terminal-components/completion/015/verify.toml | 60 | c91eb1e3da134a483ff6eb12d568edae3de26026e1446d7aad8ad7338ec9b785 |
| refactoring-tasks/terminal-components/completion/016/AGENTS.md | 83 | 3240e629c47b42706f69ae2bf1ee57cd1d7da2dd7f3c80d1525cb96e33dd37b1 |
| refactoring-tasks/terminal-components/completion/016/README.md | 176 | 3ac4a06057ad7e8938ff93905d802d474692d5fdf625f5cc3001494474c82394 |
| refactoring-tasks/terminal-components/completion/016/task.toml | 3 | fa9829d29758ee58e34caf9b22bb30b45a5a530f25a9c1b9c81e2984e0b3fa1f |
| refactoring-tasks/terminal-components/completion/016/trusted/obligations.md | 325 | 7ccaa511148efa0cc7a80d0aaf3eb07b72e9e08f28cb06c8b47b2eeea11b8d8a |
| refactoring-tasks/terminal-components/completion/016/trusted/source-obligations.tsv | 26 | ec091af921faf0c6187e1c43c4201673b3693800f783db092bf22988f0ecd596 |
| refactoring-tasks/terminal-components/completion/016/trusted/source-witnesses.md | 24 | c47542dec24eda63c2ec65623f87429cbabd3a47825596250c9a925c4d689b61 |
| refactoring-tasks/terminal-components/completion/016/verify.toml | 60 | 82a76a0aa4b4360be7a72028b2f087ef87751b8018d86e2f5158f207af39a185 |
| refactoring-tasks/terminal-components/completion/017/AGENTS.md | 83 | ecc0e46f081916357208badcc6260bb9a1cbde00a20266bf13546e46c441c272 |
| refactoring-tasks/terminal-components/completion/017/README.md | 173 | 08c8179683612feae9fdb750c2bd3619eee660a518fbe36382de2d40e87ee38d |
| refactoring-tasks/terminal-components/completion/017/task.toml | 3 | bbba3b256cfa2a8eed433bae8e847c46bde34e658724f0f5be5b80b9ce784866 |
| refactoring-tasks/terminal-components/completion/017/trusted/obligations.md | 285 | 30d264b308532af43a3f94a69684d536941322741f58333a640619b869cc251b |
| refactoring-tasks/terminal-components/completion/017/trusted/source-obligations.tsv | 26 | 643999f9dc5b666036bcdc73a16ce204af09af7742ed497a5ab7b02ec2a66380 |
| refactoring-tasks/terminal-components/completion/017/trusted/source-witnesses.md | 23 | 999ac74615016ee7b72b92c63bf285b22a28b88908b5ac2793e92073e4f282b4 |
| refactoring-tasks/terminal-components/completion/017/verify.toml | 60 | 89c388ad58f6fa58a7d1abe955df1b7ec165b9467904668882cfd67904ad76ba |
| refactoring-tasks/terminal-components/completion/018/AGENTS.md | 83 | f6997af2f49e5990625d990324bdb60de845f17df58f477287890a8efad8b403 |
| refactoring-tasks/terminal-components/completion/018/README.md | 175 | 3a252ee1bb91c32406bc4f20ce641997a2aa4c6be02523ef8e79d3c5335fdb3d |
| refactoring-tasks/terminal-components/completion/018/task.toml | 3 | e3f25f7a69f425001f33053aa93462ef85409f83f52eaff200b38100f27d6613 |
| refactoring-tasks/terminal-components/completion/018/trusted/obligations.md | 333 | ee2d435d4f4090fa2bc7b870577d877481e6ae537caaf713bb2cf7a5ede77efa |
| refactoring-tasks/terminal-components/completion/018/trusted/source-obligations.tsv | 28 | ecf7db557f48e267c8e8f9f2b703acb384b51f1ad88a3b4142b8cb8f9376f74b |
| refactoring-tasks/terminal-components/completion/018/trusted/source-witnesses.md | 22 | d7386df84757703b40c98af97cb21d3e892f94b0f1f4797601f20cbe606fbaba |
| refactoring-tasks/terminal-components/completion/018/verify.toml | 60 | 7c58a5ae4b9bf5d454b147325918f807394951f68740397aa8106c835ee68040 |
| refactoring-tasks/terminal-components/completion/019/AGENTS.md | 83 | acd691430481d4346ac4135b7c62b3e9e7bf6a343bf6588a4162ccc5631d98cf |
| refactoring-tasks/terminal-components/completion/019/README.md | 173 | 02ad16aae278189891854ed2f864c75653762b70ac44d2b1c6ac3dbc0c7bc887 |
| refactoring-tasks/terminal-components/completion/019/task.toml | 3 | 09c79e04e788510b37f5567c26d8cdae0206ac41dfdcdcb52210c335cf8e9bc4 |
| refactoring-tasks/terminal-components/completion/019/trusted/obligations.md | 438 | 03e8d20700432390146cca0d7b36997df3c224acf7e484e46ae273b3e03b9d69 |
| refactoring-tasks/terminal-components/completion/019/trusted/source-obligations.tsv | 43 | 44ac9c69cb4d4b83a73d0cbbcb3e02c389923dfdbcc122c5bb1715d263d192ad |
| refactoring-tasks/terminal-components/completion/019/trusted/source-witnesses.md | 29 | ad333b83834ffc7351642403fa82cf52b043d95cf6b9f0f0bc4aa28683bbbe4a |
| refactoring-tasks/terminal-components/completion/019/verify.toml | 60 | 68fc154d721ccb9372053346e6ad06fc3e12a821b8412e8a4599e47edcb1f4b6 |
| refactoring-tasks/terminal-components/completion/020/AGENTS.md | 83 | 5777c79e5835551b36a816c95aa14e198566ca01a9ef7bc163c632db47b7d6be |
| refactoring-tasks/terminal-components/completion/020/README.md | 180 | 62139a973dd57295c1c01f4ca987b793ddc9a35226af1ca1b942f61a7bb00953 |
| refactoring-tasks/terminal-components/completion/020/task.toml | 3 | e45bd4f468ec28d614654c1b9ee494a43202dc3ba89e95d43b6144508c55a38f |
| refactoring-tasks/terminal-components/completion/020/trusted/obligations.md | 544 | d57a0b1ef92f19185e24edfd2433c5907db1b76086af9dbceb2d132e83571538 |
| refactoring-tasks/terminal-components/completion/020/trusted/source-obligations.tsv | 46 | bf2c7fb82f57d095902f0505ab6f1708e1ea07646393aa6867990aa6f6ae8d75 |
| refactoring-tasks/terminal-components/completion/020/trusted/source-witnesses.md | 33 | 6ef824d14f41309c7c08bd9f759b3fbf34ab193df4504ecdbbfe0cc3b484c065 |
| refactoring-tasks/terminal-components/completion/020/verify.toml | 60 | 063c44ccae73f24decd97ee42b3af8b6b2f218ff725e0a6e063f0ff9d5a521be |
| docs/refactoring-plan/architecture-adjudication.md | 160 | 0fea2ca854cf890591993ace26bfd0dba423b993ceb1e0cc49a16f8ec90ddc67 |
