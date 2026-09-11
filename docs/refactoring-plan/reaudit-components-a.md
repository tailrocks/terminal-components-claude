# Component planning re-audit A: TASK-009–020

Status: **planning repairs applied; independent acceptance pending**. This is an adversarial planning review, not an execution receipt or a whole-plan clean certificate. The initial audit below preceded repairs. The parent subsequently authorized changes to TASK-009–020 package documents and metadata, excluding AGENTS; the repair checkpoint below records those changes. No production source, canonical AGENTS protocol, baseline, commit or branch was changed by this reviewer.

## Authority and review coverage

The immutable tag was resolved afresh to `02f5294bfdbf38004cc49130d0aff1d01f31434c`; local main resolves to `7b27732a8c3c131760ec3438f641cb3c11343a42`. PLANNING_GOAL.md was read completely. Architecture-adjudication.md and component-parity.tsv were inspected against the assigned task contracts. Caveman communication instructions were read; persisted findings use normal prose.

The inventory below covers all 72 assigned package files, totaling 8,745 lines. README requirements, acceptance blocks, fixed decisions, scopes and checklists were reviewed, with common boilerplate inspected once and task-number-normalized differences inspected for every other package. AGENTS and verify.toml were checked the same way: every AGENTS copy is identical after task-number normalization; every verify difference is its writable-path line. Every task.toml dependency list and every source-obligations requirement/acceptance/check mapping was inspected. Trusted obligation bodies were read in source order and compared with their parent requirements; source TSV metadata was additionally parsed and inspected for joins and disposition drift.

Semantic source review was deeper for the actual counterexamples below than for unchanged component families. Full original source was read for main terminal session, oracle runtime, main shared painter, oracle fade, and oracle Choice. Relevant original main RowUi, Choice, viewport and original historical TextArea adjudication sections were read. This review does **not** claim line-by-line semantic validation of every historical source revision or every production line behind all 400-plus repeated source clauses. Those untouched source expansions remain an explicit limitation; the presence of file hashes or passing join scripts cannot erase it.

## Findings

### CA-01 — TASK-009 cannot implement supported job control inside its declared dependency/scope contract

- Contract: `refactoring-tasks/terminal-components/completion/009/README.md:49` requires suspend/resume restoration; `009/verify.toml:3` permits only session/time/feedback/event and its completion test. `009/trusted/obligations.md:136` assigns F21 to this task.
- Source: pinned main `crates/tui/src/runtime/session.rs` has no signal handling or suspension/re-entry state. Oracle `src/runtime.rs:145` implements suspend, `:192` owns job_control, and `:280` handles pending suspension. Its signal/raise bridge requires unsafe calls.
- Enabling condition: main `crates/tui/src/lib.rs` forbids unsafe; workspace Cargo.toml denies unsafe. Main `crates/tui/Cargo.toml:20` gates terminal code through optional ratatui-crossterm and declares no safe signal dependency. A transitive signal-hook in a lockfile would not create a directly usable Rust dependency.
- Counterexample: send SIGTSTP to a live main application while raw mode is active. The current process stops without the oracle's event-loop restoration and full re-entry path. Copying the oracle bridge violates the accepted unsafe boundary; using a safe dependency requires manifests/lock outside TASK-009 scope.
- Structural repair: decide the exact supported Unix signal API, safe optional dependency/version/features, terminal-only dependency boundary, signal handler lifecycle and EINTR/wakeup behavior now. Assign the required Cargo.toml/Cargo.lock edits to an earlier producer or TASK-009 and serialize acceptance. Require actual owned PTY stop/continue, repeated suspension, resize while stopped, partial re-entry error, idle wait and flooded-input cases. Preserve no-default-feature isolation.
- Related scope inversion: TASK-009 also owns headless Bootstrap/deadline assertions implemented in runtime.rs, which belongs to downstream TASK-010. Explicitly distinguish preservation-only checks from repair ownership; a discovered runtime defect must not require a downstream repair to unblock its prerequisite.

### CA-02 — TASK-013 excludes the structural owner of its known cross-span grapheme defect

- Contract: `013/README.md:45` requires complete logical-line segmentation before style ranges; `:47` requires split-span and wide-shadow red cases. `013/verify.toml:3` permits only `crates/tui/src/text` and its completion test.
- Source: main `crates/tui/src/ui/paint.rs:140` implements paint_spans by independently calling paint_str for each `sp.text`; `:76` owns the actual grapheme writer and continuation reset. Main `components/viewport.rs:472` separately walks each run with independent segmentation.
- Counterexample: two styled spans containing `"e"` and `"\u{301}"`. paint_spans writes the base letter, then the isolated combining grapheme has zero width and is skipped. The complete logical grapheme must retain the mark. Editing text/** alone cannot make this painter see both spans or repair its continuation writes.
- Enabling condition: the plan maps a cross-span rendering contract to text algorithms while leaving the per-span iteration at another writable owner. TASK-011 owns paint.rs in a parallel branch; TASK-021 owns the viewport consumer downstream.
- Structural repair: give TASK-013 the shared paint_spans/writer path, define the borrowed logical-text/style-range handoff, and serialize it after TASK-011 (and before any later shared-painter owner). Assign actual consumer migration explicitly to TASK-015/TASK-021. Require direct public Ui/RowUi span fixtures with split combining marks, split ZWJ sequences, style-boundary selection/copy, nonzero clipping and exact shadow cells. A new helper tested without replacing the production per-span loop is not completion.

### CA-03 — TASK-016 simultaneously requires visible override changes and accepts a resolution-only exception

- Contract: `016/trusted/obligations.md:15` says each declared override level and replacement slot must change intended painted cells. The same file `:176` (EARLY-AMEND-044) accepts TextArea FIELD proof by its recorded resolution when composed painting leaves the final digest unchanged.
- Original authority: `15371443:COMPONENT_ARCHITECTURE.md:6723` says this resolution assertion closes the covered FIELD false negative; `:6737` explicitly rejects silently broadening TextArea's override API. The later duplicate discussion at `:6923` confirms the same contract.
- Counterexample: apply an instance patch to the default TextArea's covered FIELD surface. Its resolution changes, then composed content covers it. The accepted historical test passes; universal CP-COMMON painted-cell wording fails.
- Enabling condition: shared blanket acceptance prose collapses distinct resolution coverage, public painted customization and SlotFn substitution contracts.
- Structural repair: explicitly adjudicate the FIELD case and encode its exact proof disposition. Preserve the accepted resolution-only exception where the public contract promises only resolution; continue requiring actual painted-cell proof for public slot substitution and exposed paint overrides. If visible FIELD behavior is to change, that requires source-qualified acceptance and an oracle-compatible design, not an executor improvisation. Update CP-COMMON and copies consistently.

### CA-04 — TASK-017's universal modal-first paste rule contradicts a reachable immutable TablePro trajectory

- Contract: `017/README.md:49` says modal-first paste never leaks; `017/trusted/obligations.md:108` assigns F01 and notes that an exact oracle conflict must be documented. No fixed decision resolves the actual conflict.
- Source: oracle `src/bin/tablepro/app.rs:236–254` consumes paste for Dialog and Filter only. Picker falls through to the active underlying screen. Independent application reviewer confirmed the continuation through `workbench.rs:1194` and `tabs.rs:1543`.
- Counterexample: `W; Ctrl+T; i; type(SELECT ); Ctrl+O; paste(PARITY)`, or use Ctrl+G for TabList. The picker is open, its query remains unchanged, and the underlying editing query changes. Switcher/TabList open paths retain editing.
- Enabling condition: an uncompleted historical improvement was promoted to universal current parity behavior without reconciling the higher-priority immutable oracle. Layer architecture and app paste ownership are different contracts.
- Structural repair: freeze this exact trajectory and disposition before dispatch. Retain the ordinary reusable layer barrier; define the explicit source-qualified application compatibility policy needed to reproduce the oracle trajectory through supported production APIs. If that cannot coexist with an accepted architecture rule, record the precise conflict and its conservative resolution in the plan rather than silently changing the oracle. Update TASK-017, TablePro owners, application scenarios and source mappings together. Do not bless a repaired application frame.

### CA-05 — TASK-018 still delegates the Choice geometry/API reconciliation that planning was supposed to settle

- Contract: `018/README.md:46` requires completing `mono_pressed_choice_keeps_the_label_geometry`; `018/trusted/obligations.md:165` retains the explicit unresolved geometry item. Fixed decisions do not contain an oracle-backed resolution.
- Source: oracle `src/widgets/choice.rs` paints compact marks for widths below four and labels at x+5; it never inserts pressed brackets inside the label. Main `crates/tui/src/components/choice.rs:181–190` prepends PressLeft and appends PressRight inside the label run, consuming content width.
- Counterexample: a mono pressed checkbox with a label occupying the available label area loses or shifts content when brackets are inserted. A named test title without fixed expected positions lets the executor choose among incompatible repairs.
- Additional concrete interaction discrepancy: oracle Choice `:151–165` changes both RadioGroup cursor and selected value on Up/Down/j/k. Main Choice `:1138–1142` changes only cursor and emits no action; `:1188–1200` chooses only on Space/Enter. Controlled ownership alone does not reproduce the oracle's arrow-commit UX.
- Structural repair: add a fixed source-qualified Choice adjudication now. Define marker/gutter/label/trailing positions at widths 0,1,2,3,4,5 and full-width overflow; retain oracle mono state output without new in-run label columns. Define exactly how oracle app compositions commit stable radio cursor changes while preserving caller-owned value and generic navigation/value separation. Freeze separate arrow, activation, external-value change and removal/reorder traces. Consumer ownership must be explicit.

### CA-06 — The trusted A12 disposition retains a stale unresolved status

- Contract location: `010/trusted/source-obligations.tsv:13` has a corrected implementation sentence followed by `Source disposition: unresolved`.
- Conflicting authority: `010/trusted/obligations.md:200–204` and ADJ-01 recognize the accepted publication-time implementation from `715ee0777e20a09e0f373b07024076bdc742ef1d`.
- Counterexample: a consumer using the normative TSV disposition sees an unresolved design while another consumer using the appendix sees a preservation obligation. A semantic join can remain syntactically complete in both cases.
- Structural repair: rebuild every source-qualified status/disposition from one adjudicated row; reject embedded obsolete status phrases in generated copies. Preserve the historical unresolved status only as historical evidence with its superseding edge, never as the current task disposition.

## Checks and residual review limits

The task metadata consistently binds seven host-controlled operations and the same acceptance/check links. No candidate-local command substitutes for the host tool. TASK-073 precedes every owned task; task-specific source writes are separated except the explicit serialized runtime/painter owners described above. Canonical AGENTS remains unchanged and the README campaign adaptation explicitly supersedes local executor authority.

The oracle fade constants, height thresholds, RGB arithmetic, background-majority selection, row exemptions and non-RGB DIM branch were read directly and agree with TASK-014's stated arithmetic. This does not prove downstream call-site completeness. F1–F13 are individually present in TASK-019, including the private Form bridge decision; that does not prove the future generic CHK-006 implementation tests each semantic predicate.

All findings above are source/contract counterexamples, not claims that future tc-proof operations have executed. No source-string scan, SHA inventory, test count or scenario self-join is used as evidence that all reachable states are covered. New capture tasks may build trusted baselines, but they still need predetermined, source-derived scenario membership and explicit policy decisions.

## File inventory

SHA-256 values below identify bytes at this audit's inventory checkpoint. They are not a semantic PASS.

| Path | Lines | SHA-256 |
| --- | ---: | --- |
| `refactoring-tasks/terminal-components/completion/009/AGENTS.md` | 83 | `0260a43a13384226baf2bdf479776ada61c46f4e899007e47400cfb931d69706` |
| `refactoring-tasks/terminal-components/completion/009/README.md` | 171 | `57b7c0bd14f9a968946f204f488c76c662ab6d093356f79aabce7fe4228af32a` |
| `refactoring-tasks/terminal-components/completion/009/task.toml` | 3 | `b372dc7e92a11276638daf268bf752716857f34bedb19f70a940587e1be58a56` |
| `refactoring-tasks/terminal-components/completion/009/trusted/obligations.md` | 313 | `df218a155e2cfe9826a950b42a758d857aedaec949e983a912e8109821eca572` |
| `refactoring-tasks/terminal-components/completion/009/trusted/source-obligations.tsv` | 26 | `1bab831c56f863e1d7aa3672544162a092106937d2a97cf90ab9bd26906d0b53` |
| `refactoring-tasks/terminal-components/completion/009/verify.toml` | 60 | `260881a85e8b791321d8a0064886cd671bc198a0a96836dbeea486349055eb70` |
| `refactoring-tasks/terminal-components/completion/010/AGENTS.md` | 83 | `c77cf1534cb327240a294a9c3c8a6708a2b56474fca04307ef7ec9c07d93c102` |
| `refactoring-tasks/terminal-components/completion/010/README.md` | 172 | `bebebd2da45e158ac6c199979b39464157f3f375a5ec33d8ccea8d1867a99d11` |
| `refactoring-tasks/terminal-components/completion/010/task.toml` | 3 | `6d2315891237c41818a11094e19ee960fa474059187584d660c5f80b790a0740` |
| `refactoring-tasks/terminal-components/completion/010/trusted/obligations.md` | 826 | `0a2e5c291d06167fdd5398f6f6e05d152e3f544e39e6a844292088eefe4c9a17` |
| `refactoring-tasks/terminal-components/completion/010/trusted/source-obligations.tsv` | 80 | `9f2d41c0206a0b934f5c5ec24836d064aaf8d6864069ffe65b020e4c8104f625` |
| `refactoring-tasks/terminal-components/completion/010/verify.toml` | 60 | `f69bba5dbfd8d21076d2ae3fd1332a6be93ba70e378fd75455aa00df4d980d51` |
| `refactoring-tasks/terminal-components/completion/011/AGENTS.md` | 83 | `46713a1dda4a827090c6d01e8b8a6efd3c8873f4f57770452f09024bc37552aa` |
| `refactoring-tasks/terminal-components/completion/011/README.md` | 169 | `106ffabba0eaa2d291ff268320b18cf36fd933b9d5bd918afdb7f9f89b2b61ea` |
| `refactoring-tasks/terminal-components/completion/011/task.toml` | 3 | `b372dc7e92a11276638daf268bf752716857f34bedb19f70a940587e1be58a56` |
| `refactoring-tasks/terminal-components/completion/011/trusted/obligations.md` | 617 | `d1af212b7eb06f24924a68328f256d560907e7d8e6d332f46e66afbf33911f5e` |
| `refactoring-tasks/terminal-components/completion/011/trusted/source-obligations.tsv` | 64 | `3ba092cb1adf94d54cef0e434066e2866c32266ffedee45f0bca7e6fb4d5c718` |
| `refactoring-tasks/terminal-components/completion/011/verify.toml` | 60 | `9fbbe0afc81893cd7427197169e17e9ae47a4d013736286cdba9e77d3715c821` |
| `refactoring-tasks/terminal-components/completion/012/AGENTS.md` | 83 | `a836350399d2bdf8368c739b1bd785a955a917094b9c72e3e5feb68898d3bfcb` |
| `refactoring-tasks/terminal-components/completion/012/README.md` | 170 | `07ccc181b53e83701ffda10a95820b125d35cf38e5bde04047dd4ab75b241abd` |
| `refactoring-tasks/terminal-components/completion/012/task.toml` | 3 | `b372dc7e92a11276638daf268bf752716857f34bedb19f70a940587e1be58a56` |
| `refactoring-tasks/terminal-components/completion/012/trusted/obligations.md` | 260 | `1b1027f738e24ebf3b34411e15ee4ad5ba81e770d6b17c14b72cdae7930edc6c` |
| `refactoring-tasks/terminal-components/completion/012/trusted/source-obligations.tsv` | 23 | `acdcb581c9acacbd6fd0b69b0ebaed98c705dc95bc4e8be4f97626111ed94db7` |
| `refactoring-tasks/terminal-components/completion/012/verify.toml` | 60 | `91f6d69e8f152d64a1f026fc12e9043af17643750969310b13f6cd0dd6c638dd` |
| `refactoring-tasks/terminal-components/completion/013/AGENTS.md` | 83 | `bd2278cdae126f3cf5b3b8a6b151d47134d198262d1f8878021315a66513bd55` |
| `refactoring-tasks/terminal-components/completion/013/README.md` | 168 | `9cd7c227bbfd0c1e5ea0072a2dd186b99d704e9e542640d048b2c40874b9ab26` |
| `refactoring-tasks/terminal-components/completion/013/task.toml` | 3 | `b372dc7e92a11276638daf268bf752716857f34bedb19f70a940587e1be58a56` |
| `refactoring-tasks/terminal-components/completion/013/trusted/obligations.md` | 162 | `3a7ecf6eab0262d7a387602058022a8edc81f5e61be362f3da8991eb9aa01343` |
| `refactoring-tasks/terminal-components/completion/013/trusted/source-obligations.tsv` | 12 | `4cb418248b0e6211e52d37e215e7edb15df21472c34d29c9f6c3c6daaa1e99b2` |
| `refactoring-tasks/terminal-components/completion/013/verify.toml` | 60 | `b24c11a2aae80bc90c53ec6a2eaabeedcfedf7f3dcc0abe95431d5bff30177b1` |
| `refactoring-tasks/terminal-components/completion/014/AGENTS.md` | 83 | `091a0ee40e9ea43163c9dfe19ddde0d45a47c5a2b6bcb42869fad2cebb3e7841` |
| `refactoring-tasks/terminal-components/completion/014/README.md` | 171 | `440b9066ab45111ded77f7ae97a3d45639cc087f4240d4a980866a03bebac5ef` |
| `refactoring-tasks/terminal-components/completion/014/task.toml` | 3 | `f7fd5652a7c153a0fd386769cde279b59ac1f1ba2bcb82dc8650a194caf36225` |
| `refactoring-tasks/terminal-components/completion/014/trusted/obligations.md` | 176 | `1a9d38fdc0a607bc321ddd277f9e76b7c9fa205da948efc946e3803e0cacbf3d` |
| `refactoring-tasks/terminal-components/completion/014/trusted/source-obligations.tsv` | 15 | `6d16c5bcc79f859b5da92b259f9fafaf27cabc23128a3d0f6ea27b9857776e7f` |
| `refactoring-tasks/terminal-components/completion/014/verify.toml` | 60 | `13c7e931025ac5f19b6bb68001e6e9f229a7d17d22e8112e7123720fa2df7bcc` |
| `refactoring-tasks/terminal-components/completion/015/AGENTS.md` | 83 | `ab57edeb75d7638274746b2153e65c47ecfa5a5dc7f1ad3073a259d31a96690e` |
| `refactoring-tasks/terminal-components/completion/015/README.md` | 168 | `1ab2b3524129b785f6a22e3d7d9ed4db011c8cc9791a4c2116355f4ce09d11c7` |
| `refactoring-tasks/terminal-components/completion/015/task.toml` | 3 | `1a03ee7d14b3d32f83a629256f0c79f51c6f9d0a000eba43c82f8e7a6218739b` |
| `refactoring-tasks/terminal-components/completion/015/trusted/obligations.md` | 322 | `fcebed2480114224738c058017879a0b3a25b2b6306be8a38fb99d01a013a727` |
| `refactoring-tasks/terminal-components/completion/015/trusted/source-obligations.tsv` | 27 | `0e3d0df21c160e3f8074d3fe53201392bdda9855bdadc15293f551d64c52c19b` |
| `refactoring-tasks/terminal-components/completion/015/verify.toml` | 60 | `c91eb1e3da134a483ff6eb12d568edae3de26026e1446d7aad8ad7338ec9b785` |
| `refactoring-tasks/terminal-components/completion/016/AGENTS.md` | 83 | `3240e629c47b42706f69ae2bf1ee57cd1d7da2dd7f3c80d1525cb96e33dd37b1` |
| `refactoring-tasks/terminal-components/completion/016/README.md` | 173 | `ed8e5744d5164a47b00b5acf65a4b2936d4b4f20f28dc80635f91d84534ccb25` |
| `refactoring-tasks/terminal-components/completion/016/task.toml` | 3 | `fa9829d29758ee58e34caf9b22bb30b45a5a530f25a9c1b9c81e2984e0b3fa1f` |
| `refactoring-tasks/terminal-components/completion/016/trusted/obligations.md` | 311 | `dc905ab5cc0fcd79c10db3b43a2c966f0d1f1ad8b78caf415554bb7377a6938d` |
| `refactoring-tasks/terminal-components/completion/016/trusted/source-obligations.tsv` | 26 | `ec091af921faf0c6187e1c43c4201673b3693800f783db092bf22988f0ecd596` |
| `refactoring-tasks/terminal-components/completion/016/verify.toml` | 60 | `82a76a0aa4b4360be7a72028b2f087ef87751b8018d86e2f5158f207af39a185` |
| `refactoring-tasks/terminal-components/completion/017/AGENTS.md` | 83 | `ecc0e46f081916357208badcc6260bb9a1cbde00a20266bf13546e46c441c272` |
| `refactoring-tasks/terminal-components/completion/017/README.md` | 170 | `9c016e3d4a33e55642ce132620711d0d700aa46ab6bfaeee8b93c0076ed17afe` |
| `refactoring-tasks/terminal-components/completion/017/task.toml` | 3 | `bbba3b256cfa2a8eed433bae8e847c46bde34e658724f0f5be5b80b9ce784866` |
| `refactoring-tasks/terminal-components/completion/017/trusted/obligations.md` | 275 | `33b333ad8643232715938b91e2897594937e0682050ad06a322a956acfa7ee53` |
| `refactoring-tasks/terminal-components/completion/017/trusted/source-obligations.tsv` | 26 | `643999f9dc5b666036bcdc73a16ce204af09af7742ed497a5ab7b02ec2a66380` |
| `refactoring-tasks/terminal-components/completion/017/verify.toml` | 60 | `89c388ad58f6fa58a7d1abe955df1b7ec165b9467904668882cfd67904ad76ba` |
| `refactoring-tasks/terminal-components/completion/018/AGENTS.md` | 83 | `f6997af2f49e5990625d990324bdb60de845f17df58f477287890a8efad8b403` |
| `refactoring-tasks/terminal-components/completion/018/README.md` | 169 | `e01ab0b8839be1f6b3c78822ee3603a79323a5b27565c80ab2f883d4d9d64964` |
| `refactoring-tasks/terminal-components/completion/018/task.toml` | 3 | `e3f25f7a69f425001f33053aa93462ef85409f83f52eaff200b38100f27d6613` |
| `refactoring-tasks/terminal-components/completion/018/trusted/obligations.md` | 317 | `4b8612bae5e8a9030de5fc6f7f7a20474db38aadf1893c6c0130afeba9cef535` |
| `refactoring-tasks/terminal-components/completion/018/trusted/source-obligations.tsv` | 28 | `ecf7db557f48e267c8e8f9f2b703acb384b51f1ad88a3b4142b8cb8f9376f74b` |
| `refactoring-tasks/terminal-components/completion/018/verify.toml` | 60 | `f8be1f0dbb9554d24f51e77b0e61fc1ba92bf2fb29cc34081a6870c7aa6dfce9` |
| `refactoring-tasks/terminal-components/completion/019/AGENTS.md` | 83 | `acd691430481d4346ac4135b7c62b3e9e7bf6a343bf6588a4162ccc5631d98cf` |
| `refactoring-tasks/terminal-components/completion/019/README.md` | 168 | `1567e2c173169f9805b52453e6ee0fd24bf8b3ccd076d519bb878fd75e62f84c` |
| `refactoring-tasks/terminal-components/completion/019/task.toml` | 3 | `09c79e04e788510b37f5567c26d8cdae0206ac41dfdcdcb52210c335cf8e9bc4` |
| `refactoring-tasks/terminal-components/completion/019/trusted/obligations.md` | 428 | `ccdbb4013d66cfd2bee4774fece098d8bcf0f8778046e8a5e2cb4ce05ab0c0e1` |
| `refactoring-tasks/terminal-components/completion/019/trusted/source-obligations.tsv` | 43 | `44ac9c69cb4d4b83a73d0cbbcb3e02c389923dfdbcc122c5bb1715d263d192ad` |
| `refactoring-tasks/terminal-components/completion/019/verify.toml` | 60 | `cc9f8c2fea0dd9436e7fb1ce0027d0e3d68da6905e4e2a5b7e8add3da61dd290` |
| `refactoring-tasks/terminal-components/completion/020/AGENTS.md` | 83 | `5777c79e5835551b36a816c95aa14e198566ca01a9ef7bc163c632db47b7d6be` |
| `refactoring-tasks/terminal-components/completion/020/README.md` | 173 | `09d9ccdabbdee7008e03e5d6c81e5c52db60be0f91d84d66f6aa3b0996f4c4ce` |
| `refactoring-tasks/terminal-components/completion/020/task.toml` | 3 | `feb44e1058abdf6a6031569531926139c970d29656092cc37c864944bcf235cf` |
| `refactoring-tasks/terminal-components/completion/020/trusted/obligations.md` | 528 | `3490894708657b3acbea005515722535fbcb50a97cfcc5ef8cc9e8aecdb78147` |
| `refactoring-tasks/terminal-components/completion/020/trusted/source-obligations.tsv` | 46 | `bf2c7fb82f57d095902f0505ab6f1708e1ea07646393aa6867990aa6f6ae8d75` |
| `refactoring-tasks/terminal-components/completion/020/verify.toml` | 60 | `00e48a74f3e3b79af1fdb513c4bca24b6c4dae4ea38fbba2b539b18dcfd8d4cd` |


## Authorized repair checkpoint

The parent authorized bounded package repairs after receiving the findings above. All edits stayed in this report and TASK-009–020 package files; the twelve AGENTS SHA-256 values remain identical to the initial inventory. Shared adjudication, source ledgers, root task graph and application-owner contracts remain the parent/assigned reviewers' responsibility.

- **CA-01:** TASK-009 now owns root/tui manifests, lock dependency edge and runtime.rs as well as terminal session code. The future safe Unix backend dependency is signal-hook =0.3.18 with defaults off. Its process-lifetime broker explicitly assumes initial default SIGTSTP and no competing registration, serializes sessions, retains default stopping outside a session and never relies on unregister restoring a handler. The 100-ms backend service quantum has due/overdue/flood fake-clock witnesses; it cannot synthesize logical Tick/Settle/draw or simulation expiry. Owned PTY proof retains the oracle's 10-second safety bound, exact termios, repeated sessions/suspensions, failed setup/re-entry and post-session stopping. This is a future implementation contract, not a claim that a broker prototype ran.
- **CA-02:** TASK-013 owns ui/paint.rs and follows TASK-011; TASK-014 follows TASK-013. One borrowed logical-grapheme source/painter contract replaces per-span segmentation. TASK-015 and TASK-021 consume the boundary/style contract under their own later receipts, not as impossible TASK-013 prerequisites.
- **CA-03:** Owned CP-COMMON copies now distinguish visible override/slot cell proof from the accepted covered TextArea FIELD recorded-Resolved exception.
- **CA-04:** TASK-017 keeps default runtime capture. Holla menu and TablePro picker paste are explicit source-qualified app commands through shared configured component operations, not hidden runtime intents or a new LayerSpec bypass. TASK-016 fixes the exact public TextInput::paste signature; TASK-025 owns CodeEditor. Subsequent source review corrected idle TextInput paste to begin editing after sensitivity setup, while idle TextArea/document CodeEditor differ. Empty paste still performs source validation/repaint. First inert frame uses the existing pure current-width projection rather than waiting for reveal.
- **CA-05:** TASK-018 fixes exact Choice marker/label/trailing geometry and adds typed Navigated(ItemKey), including clamped arrows but excluding reconciliation and pointer Press. Generic controlled values remain separate. TASK-019 exposes Form::radio_navigation_commits(bool), default false, via a private shared bridge. Narrow choice.rs/form.rs/conformance compatibility scopes and predecessor edges are explicit.
- **CA-06:** TASK-010's embedded A12 current disposition no longer says unresolved after reporting accepted implementation.

### Additional source findings repaired during independent recheck

**CA-07 — ChipBar's missing shared API and compile ownership.** Oracle chips.rs provides Lead/ClearAll, source-specific +/X/Backspace behavior, per-item checked/removable/error state and clipped nonreserved geometry. Real Showcase pages/chips.rs:47–113 and TablePro tabs.rs:406/528, workbench.rs:747–751/1075 consume these paths. Main chip.rs exposed none of the lead/clear providers, and its painter reserves Add and scrolls; an application could not reproduce the source without copying generic chrome or inventing keys. The actual source also returns on overflow before group focus registration while retaining earlier visible hits. TASK-020 now owns exact borrowed lead/checked/removable/error builders, generic scrollable/reserve_add policies, keyless LeadRequested/ClearRequested, exact modifier guards and source preset geometry/focus. Generic defaults remain supported. Additive actions require conformance.rs:1678 and apps/showcase/src/pages/chips.rs:269–274 exhaustive adaptation; both are in the narrow producer scope, with TASK-018 preceding the shared conformance edit and TASK-037 owning actual Showcase domain migration. The parent recorded ADJ-11 and app consumers. Tiny containment remains an architecture lane, not a fabricated tiny oracle baseline.

**CA-08 — Input validation was incorrectly promoted to a commit veto.** The original TASK-016 R-003 and trusted paragraph promised “validation denial blocking commit/navigation.” Oracle input.rs:169–172/203–215 ends editing, validates and still emits Committed/CommittedTab; main input.rs:745–753 writes controlled value before recording validation. The enabling condition was a blanket editor/form “validation” label that merged distinct transition contracts. TASK-016 now requires the actual invalid commit, error, end-edit and committed-tab semantics; only the later Form submission gate blocks submission. W-016-04 and the fixed validation disposition explicitly reject the false veto and preserve read-only/disabled safety.

### Finite witness and mechanical verification checkpoint

Twelve new trusted/source-witnesses.md files define **106 unique case IDs** across all owned task families. Each row binds a concrete source, authority lane, finite state/action set, exact observations and a real isolated production mutation. Shared expansion fixes theme/capability, allocation/origin and clipping sets, per-action first/repeated-draw checkpoints, source ownership and explicit component-versus-later-app phase ownership. These manifests supplement every historical obligation and public part/slot census; they do not replace it with a representative count. TASK-006 must freeze the exact expanded inputs and independent expectations before implementation. No executor chooses applicability or invents source output.

Mechanical checks performed: all 24 task/verify TOML documents parse with Python's tomllib; all 106 case IDs are unique; every README/trusted obligations document binds its witness file; every new table row has five columns; all twelve original AGENTS hashes match. These checks prove document integrity only, not future production parity or complete semantic source coverage. Independent reviewer /root/reaudit_jackin_tablepro inspected the actual repairs and separately executed disposable source witnesses for split-cluster loss, inert TextInput projection and clipped ChipBar focus behavior; their report owns those execution claims. No future tc-proof gate was run by this reviewer.

The original audit inventory remains above as its pre-repair checkpoint. The current package checkpoint is **84 files / 9,200 lines**:

| Path | Lines | SHA-256 |
| --- | ---: | --- |
| `refactoring-tasks/terminal-components/completion/009/AGENTS.md` | 83 | `0260a43a13384226baf2bdf479776ada61c46f4e899007e47400cfb931d69706` |
| `refactoring-tasks/terminal-components/completion/009/README.md` | 182 | `ae6084c411481a10f8bd1fb8dd35958f680df5c1f0c7b601b3d8e5670076d040` |
| `refactoring-tasks/terminal-components/completion/009/task.toml` | 3 | `b372dc7e92a11276638daf268bf752716857f34bedb19f70a940587e1be58a56` |
| `refactoring-tasks/terminal-components/completion/009/trusted/obligations.md` | 327 | `3927b1ec2d4166cbbb7b4e573a174a4380d149b5a4e0e7a3d97f43976383134b` |
| `refactoring-tasks/terminal-components/completion/009/trusted/source-obligations.tsv` | 26 | `1bab831c56f863e1d7aa3672544162a092106937d2a97cf90ab9bd26906d0b53` |
| `refactoring-tasks/terminal-components/completion/009/trusted/source-witnesses.md` | 23 | `7115206a46b6f435e6f5bfe7aa33edc271152ac4500826da572c9a7f1f272674` |
| `refactoring-tasks/terminal-components/completion/009/verify.toml` | 60 | `e5de5be67f86350dac4daea6367e11b54f324a6def6fb01d70b996c2d47df953` |
| `refactoring-tasks/terminal-components/completion/010/AGENTS.md` | 83 | `c77cf1534cb327240a294a9c3c8a6708a2b56474fca04307ef7ec9c07d93c102` |
| `refactoring-tasks/terminal-components/completion/010/README.md` | 173 | `a92c31d05b03c256c30ea0ff294cebd0672984b7d195b47383874aae65a3ff72` |
| `refactoring-tasks/terminal-components/completion/010/task.toml` | 3 | `6d2315891237c41818a11094e19ee960fa474059187584d660c5f80b790a0740` |
| `refactoring-tasks/terminal-components/completion/010/trusted/obligations.md` | 830 | `8f366fbc296e459d34ff2b0987f55282f2b62626eee88d9eec89ea2f5cf30538` |
| `refactoring-tasks/terminal-components/completion/010/trusted/source-obligations.tsv` | 80 | `d6ad5604bb510fa5d24afb8c7c46dd940a71d88f1ed617c2a821e4e2fd60b4f1` |
| `refactoring-tasks/terminal-components/completion/010/trusted/source-witnesses.md` | 23 | `659f407cd90cbaa766d55c186c2e24bf702ac381a763fb0dadbb8ff587afaa0c` |
| `refactoring-tasks/terminal-components/completion/010/verify.toml` | 60 | `f69bba5dbfd8d21076d2ae3fd1332a6be93ba70e378fd75455aa00df4d980d51` |
| `refactoring-tasks/terminal-components/completion/011/AGENTS.md` | 83 | `46713a1dda4a827090c6d01e8b8a6efd3c8873f4f57770452f09024bc37552aa` |
| `refactoring-tasks/terminal-components/completion/011/README.md` | 172 | `c259fdece783ba4c1277cb157f5f5d539cc0215137e4ea9e039db789fc1a3ee4` |
| `refactoring-tasks/terminal-components/completion/011/task.toml` | 3 | `b372dc7e92a11276638daf268bf752716857f34bedb19f70a940587e1be58a56` |
| `refactoring-tasks/terminal-components/completion/011/trusted/obligations.md` | 625 | `062394d393178cce9868e0921ab13e6df02112261b7b547d6dde6ff2150cad54` |
| `refactoring-tasks/terminal-components/completion/011/trusted/source-obligations.tsv` | 64 | `3ba092cb1adf94d54cef0e434066e2866c32266ffedee45f0bca7e6fb4d5c718` |
| `refactoring-tasks/terminal-components/completion/011/trusted/source-witnesses.md` | 22 | `580559cbe7c220feaefd68e42118eb808bbf94feed97576d398a0e057d232b27` |
| `refactoring-tasks/terminal-components/completion/011/verify.toml` | 60 | `9fbbe0afc81893cd7427197169e17e9ae47a4d013736286cdba9e77d3715c821` |
| `refactoring-tasks/terminal-components/completion/012/AGENTS.md` | 83 | `a836350399d2bdf8368c739b1bd785a955a917094b9c72e3e5feb68898d3bfcb` |
| `refactoring-tasks/terminal-components/completion/012/README.md` | 171 | `8cb94c71ed345997be892a08835d8bebbf3aca85de8ceb24186a2e443f1e3fec` |
| `refactoring-tasks/terminal-components/completion/012/task.toml` | 3 | `b372dc7e92a11276638daf268bf752716857f34bedb19f70a940587e1be58a56` |
| `refactoring-tasks/terminal-components/completion/012/trusted/obligations.md` | 264 | `3ba4442405971a79b8d5ef13a491e0d7d2eaf44eea94ed972706c90dfd813b6d` |
| `refactoring-tasks/terminal-components/completion/012/trusted/source-obligations.tsv` | 23 | `acdcb581c9acacbd6fd0b69b0ebaed98c705dc95bc4e8be4f97626111ed94db7` |
| `refactoring-tasks/terminal-components/completion/012/trusted/source-witnesses.md` | 22 | `fe5a7dc4b4d897d7e2f4afb2b868868347db350520621dc0de8f1089452c0d7e` |
| `refactoring-tasks/terminal-components/completion/012/verify.toml` | 60 | `91f6d69e8f152d64a1f026fc12e9043af17643750969310b13f6cd0dd6c638dd` |
| `refactoring-tasks/terminal-components/completion/013/AGENTS.md` | 83 | `bd2278cdae126f3cf5b3b8a6b151d47134d198262d1f8878021315a66513bd55` |
| `refactoring-tasks/terminal-components/completion/013/README.md` | 173 | `068f704e02a8179986a95b1ec8be85107d745ce15d8504642a76b190f3079c10` |
| `refactoring-tasks/terminal-components/completion/013/task.toml` | 3 | `38e66c957c340f66d716ab9dd24870a70b15b09ff7673b2615b6584c8a3b8afe` |
| `refactoring-tasks/terminal-components/completion/013/trusted/obligations.md` | 174 | `d61628a07e2a74917c54481a8dde1a6a300d1ad5c92beb4916d8280fd9d8e100` |
| `refactoring-tasks/terminal-components/completion/013/trusted/source-obligations.tsv` | 12 | `4cb418248b0e6211e52d37e215e7edb15df21472c34d29c9f6c3c6daaa1e99b2` |
| `refactoring-tasks/terminal-components/completion/013/trusted/source-witnesses.md` | 22 | `0f6c754e0c869280624f9d6e31e22d718e982b3ba42a26169283f9c33922a2d8` |
| `refactoring-tasks/terminal-components/completion/013/verify.toml` | 60 | `51f1e740cc11ccfe7ad57991e20a9ba91cf46da90690f676ebfeb677663b5a4b` |
| `refactoring-tasks/terminal-components/completion/014/AGENTS.md` | 83 | `091a0ee40e9ea43163c9dfe19ddde0d45a47c5a2b6bcb42869fad2cebb3e7841` |
| `refactoring-tasks/terminal-components/completion/014/README.md` | 172 | `26e40acfb6bc51434d54bdc139d043fd986b04d6a6c70bd3ecd37bade665d058` |
| `refactoring-tasks/terminal-components/completion/014/task.toml` | 3 | `1ef5fe1b675207e208bf7a33a9c54b6f0bf052fddb60e26b788a883dfbcbef18` |
| `refactoring-tasks/terminal-components/completion/014/trusted/obligations.md` | 180 | `18727afd2bef1d78dd3aac33cfc5a0b58266e848147e1a640ec7c79d5b3f4caf` |
| `refactoring-tasks/terminal-components/completion/014/trusted/source-obligations.tsv` | 15 | `6d16c5bcc79f859b5da92b259f9fafaf27cabc23128a3d0f6ea27b9857776e7f` |
| `refactoring-tasks/terminal-components/completion/014/trusted/source-witnesses.md` | 22 | `7cc3e1ef73327d79330a3db5061d6a8ad6ad35265699a2011acd4132bf3363d4` |
| `refactoring-tasks/terminal-components/completion/014/verify.toml` | 60 | `13c7e931025ac5f19b6bb68001e6e9f229a7d17d22e8112e7123720fa2df7bcc` |
| `refactoring-tasks/terminal-components/completion/015/AGENTS.md` | 83 | `ab57edeb75d7638274746b2153e65c47ecfa5a5dc7f1ad3073a259d31a96690e` |
| `refactoring-tasks/terminal-components/completion/015/README.md` | 171 | `39a7a794912f08eec1e84e2c43bd9997799984ca5e5b687bde957a2e059910e9` |
| `refactoring-tasks/terminal-components/completion/015/task.toml` | 3 | `1a03ee7d14b3d32f83a629256f0c79f51c6f9d0a000eba43c82f8e7a6218739b` |
| `refactoring-tasks/terminal-components/completion/015/trusted/obligations.md` | 330 | `f78b6061b5d9a1f014ac63395f3ed2ab46e8277e49f579761c0c30d9a4d10019` |
| `refactoring-tasks/terminal-components/completion/015/trusted/source-obligations.tsv` | 27 | `0e3d0df21c160e3f8074d3fe53201392bdda9855bdadc15293f551d64c52c19b` |
| `refactoring-tasks/terminal-components/completion/015/trusted/source-witnesses.md` | 21 | `3f766f555efb75a5df8b3b475418ccad495d3d4569ce928fcf783a7e42281606` |
| `refactoring-tasks/terminal-components/completion/015/verify.toml` | 60 | `c91eb1e3da134a483ff6eb12d568edae3de26026e1446d7aad8ad7338ec9b785` |
| `refactoring-tasks/terminal-components/completion/016/AGENTS.md` | 83 | `3240e629c47b42706f69ae2bf1ee57cd1d7da2dd7f3c80d1525cb96e33dd37b1` |
| `refactoring-tasks/terminal-components/completion/016/README.md` | 176 | `3ac4a06057ad7e8938ff93905d802d474692d5fdf625f5cc3001494474c82394` |
| `refactoring-tasks/terminal-components/completion/016/task.toml` | 3 | `fa9829d29758ee58e34caf9b22bb30b45a5a530f25a9c1b9c81e2984e0b3fa1f` |
| `refactoring-tasks/terminal-components/completion/016/trusted/obligations.md` | 325 | `7ccaa511148efa0cc7a80d0aaf3eb07b72e9e08f28cb06c8b47b2eeea11b8d8a` |
| `refactoring-tasks/terminal-components/completion/016/trusted/source-obligations.tsv` | 26 | `ec091af921faf0c6187e1c43c4201673b3693800f783db092bf22988f0ecd596` |
| `refactoring-tasks/terminal-components/completion/016/trusted/source-witnesses.md` | 24 | `c47542dec24eda63c2ec65623f87429cbabd3a47825596250c9a925c4d689b61` |
| `refactoring-tasks/terminal-components/completion/016/verify.toml` | 60 | `82a76a0aa4b4360be7a72028b2f087ef87751b8018d86e2f5158f207af39a185` |
| `refactoring-tasks/terminal-components/completion/017/AGENTS.md` | 83 | `ecc0e46f081916357208badcc6260bb9a1cbde00a20266bf13546e46c441c272` |
| `refactoring-tasks/terminal-components/completion/017/README.md` | 173 | `08c8179683612feae9fdb750c2bd3619eee660a518fbe36382de2d40e87ee38d` |
| `refactoring-tasks/terminal-components/completion/017/task.toml` | 3 | `bbba3b256cfa2a8eed433bae8e847c46bde34e658724f0f5be5b80b9ce784866` |
| `refactoring-tasks/terminal-components/completion/017/trusted/obligations.md` | 285 | `30d264b308532af43a3f94a69684d536941322741f58333a640619b869cc251b` |
| `refactoring-tasks/terminal-components/completion/017/trusted/source-obligations.tsv` | 26 | `643999f9dc5b666036bcdc73a16ce204af09af7742ed497a5ab7b02ec2a66380` |
| `refactoring-tasks/terminal-components/completion/017/trusted/source-witnesses.md` | 23 | `999ac74615016ee7b72b92c63bf285b22a28b88908b5ac2793e92073e4f282b4` |
| `refactoring-tasks/terminal-components/completion/017/verify.toml` | 60 | `89c388ad58f6fa58a7d1abe955df1b7ec165b9467904668882cfd67904ad76ba` |
| `refactoring-tasks/terminal-components/completion/018/AGENTS.md` | 83 | `f6997af2f49e5990625d990324bdb60de845f17df58f477287890a8efad8b403` |
| `refactoring-tasks/terminal-components/completion/018/README.md` | 175 | `3a252ee1bb91c32406bc4f20ce641997a2aa4c6be02523ef8e79d3c5335fdb3d` |
| `refactoring-tasks/terminal-components/completion/018/task.toml` | 3 | `e3f25f7a69f425001f33053aa93462ef85409f83f52eaff200b38100f27d6613` |
| `refactoring-tasks/terminal-components/completion/018/trusted/obligations.md` | 333 | `ee2d435d4f4090fa2bc7b870577d877481e6ae537caaf713bb2cf7a5ede77efa` |
| `refactoring-tasks/terminal-components/completion/018/trusted/source-obligations.tsv` | 28 | `ecf7db557f48e267c8e8f9f2b703acb384b51f1ad88a3b4142b8cb8f9376f74b` |
| `refactoring-tasks/terminal-components/completion/018/trusted/source-witnesses.md` | 22 | `d7386df84757703b40c98af97cb21d3e892f94b0f1f4797601f20cbe606fbaba` |
| `refactoring-tasks/terminal-components/completion/018/verify.toml` | 60 | `7c58a5ae4b9bf5d454b147325918f807394951f68740397aa8106c835ee68040` |
| `refactoring-tasks/terminal-components/completion/019/AGENTS.md` | 83 | `acd691430481d4346ac4135b7c62b3e9e7bf6a343bf6588a4162ccc5631d98cf` |
| `refactoring-tasks/terminal-components/completion/019/README.md` | 173 | `02ad16aae278189891854ed2f864c75653762b70ac44d2b1c6ac3dbc0c7bc887` |
| `refactoring-tasks/terminal-components/completion/019/task.toml` | 3 | `09c79e04e788510b37f5567c26d8cdae0206ac41dfdcdcb52210c335cf8e9bc4` |
| `refactoring-tasks/terminal-components/completion/019/trusted/obligations.md` | 438 | `03e8d20700432390146cca0d7b36997df3c224acf7e484e46ae273b3e03b9d69` |
| `refactoring-tasks/terminal-components/completion/019/trusted/source-obligations.tsv` | 43 | `44ac9c69cb4d4b83a73d0cbbcb3e02c389923dfdbcc122c5bb1715d263d192ad` |
| `refactoring-tasks/terminal-components/completion/019/trusted/source-witnesses.md` | 29 | `ad333b83834ffc7351642403fa82cf52b043d95cf6b9f0f0bc4aa28683bbbe4a` |
| `refactoring-tasks/terminal-components/completion/019/verify.toml` | 60 | `68fc154d721ccb9372053346e6ad06fc3e12a821b8412e8a4599e47edcb1f4b6` |
| `refactoring-tasks/terminal-components/completion/020/AGENTS.md` | 83 | `5777c79e5835551b36a816c95aa14e198566ca01a9ef7bc163c632db47b7d6be` |
| `refactoring-tasks/terminal-components/completion/020/README.md` | 180 | `62139a973dd57295c1c01f4ca987b793ddc9a35226af1ca1b942f61a7bb00953` |
| `refactoring-tasks/terminal-components/completion/020/task.toml` | 3 | `e45bd4f468ec28d614654c1b9ee494a43202dc3ba89e95d43b6144508c55a38f` |
| `refactoring-tasks/terminal-components/completion/020/trusted/obligations.md` | 544 | `d57a0b1ef92f19185e24edfd2433c5907db1b76086af9dbceb2d132e83571538` |
| `refactoring-tasks/terminal-components/completion/020/trusted/source-obligations.tsv` | 46 | `bf2c7fb82f57d095902f0505ab6f1708e1ea07646393aa6867990aa6f6ae8d75` |
| `refactoring-tasks/terminal-components/completion/020/trusted/source-witnesses.md` | 33 | `6ef824d14f41309c7c08bd9f759b3fbf34ab193df4504ecdbbfe0cc3b484c065` |
| `refactoring-tasks/terminal-components/completion/020/verify.toml` | 60 | `063c44ccae73f24decd97ee42b3af8b6b2f218ff725e0a6e063f0ff9d5a521be` |

## Subsequent independent corrections and feasibility

The preceding inventory is a checkpoint, not an assertion that later reviewed bytes are identical. ADJ-12 subsequently corrected TASK-019's validation contract. An initial attempted witness refinement treated main's early return as accepted behavior; the original F10 requirement and oracle TablePro's complete name/port validation refute that interpretation. The final task validates every visible field once in declaration order, retains every sensitivity-safe error, then focuses/reveals the first invalid field; cross-validation runs only after all fields pass. W-019-10 now distinguishes first/middle/last and simultaneous failures with later visible validators. No production behavior was changed.

Latest affected paths: `019/README.md` 173 lines, SHA-256 `46f1e440b51f9a5c05ebec99a66d80b2fca853df67e5661d10184090011ed252`; `019/trusted/obligations.md` 442 lines, `7421b92ed1a9cdc81b72e830c663f7eaaff7dd9386022d2658f81e262147f87f`; `019/trusted/source-witnesses.md` 29 lines, `925bbdf69a4c0b68705d88bfbf0844e0ee4436d6d33965548189388630a3b50f`. Prefixes are `refactoring-tasks/terminal-components/completion/`. The full 84-file inventory now totals 9,204 lines; other entries above remain identical at this checkpoint, including all canonical AGENTS files.

The separate planning-only `evidence/session-broker-bootstrap-20260911/README.md` records an executed safe Rust broker prototype and owned-PTY judge. Ordinary and optimized Python each preserve all required observations and reject four actual runtime variants. The coordinator independently rebuilt stable and Rust 1.88.0/MSRV executables and reran the documented PTY matrix; see `reaudit-session-broker-rereview.md`. This establishes bounded macOS dependency/protocol feasibility, not TASK-009 completion: real Runtime scheduling/Bootstrap integration, Linux, backend-free feature isolation, panic chaining, actual I/O cleanup retry/re-entry failure and all signal interleavings remain explicit production witnesses. The prototype's consumed restoration mask is not approved production cleanup-retry code. No baseline, production file, commit or merge was created.
