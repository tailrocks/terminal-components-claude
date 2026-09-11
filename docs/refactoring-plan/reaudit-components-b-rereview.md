# Independent rereview: component-B repairs

Date: 2026-09-11. Scope: TASK-021–031/073 repair contracts and shared ADJ-04/09, authored by another agent. TASK-067 consumer changes are excluded because this reviewer authored them. Planning-only review under PLANNING_GOAL.md and verify-and-stop; no production, baseline, task package, shared validator or Git ref was changed by this rereview.

## Result and boundary

No new blocking semantic counterexample found in the reviewed repair set. The owner confirmed source-qualification edits complete; all 85 package file line counts and SHA-256 values independently match the author's final local inventory below. That inventory is explicitly **before coordinator history-payload synchronization**. Final whole-plan acceptance still requires the coordinator's canonical projection, regenerated payloads, proof-producer qualification and independent integrated rereview. This report does not claim those unfinished operations passed, nor that future Rust mutants or full oracle parity have executed.

Authority: main `7b27732a8c3c131760ec3438f641cb3c11343a42`; oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Reviewed every new finite witness row, including the final audited-source column, and its README/trusted-obligation bindings. Inspected repaired scope/dependency metadata, fixed decisions and relevant verification consumers. Original unchanged historical appendix coverage remains in the owner's initial audit, not an invented second independent exhaustive history audit here.

## Source-backed adversarial findings disposition

| Repair | Independent source/counterexample check | Result |
| --- | --- | --- |
| TASK-027 shared readiness owner | Main `collection/empty.rs:126–220` actually paints inherited TITLE/HELP/ICON under source guards; `components/empty.rs` alone cannot repair it. A standalone substitute loses the composed owner even if text matches. TASK-027 `verify.toml:3` now includes the actual shared file, with TASK-015/020/022/024 hard ancestry; 020 owns List/Tree, 022 Grid, 024 FilterList/Picker. W-027-04 tests all five real forwarding owners and all four readiness states. | Original root cause—proof required without responsible writable source—removed. Final author clarification independently verified against :153–213: HELP requires Some(detail); ICON never paints in Empty, requires nonempty spinner frames in Loading/Partial, and noncleared glyph in Error. 027/031 witnesses forbid query-only credit and preserve truthful inherited owner ID plus composed EMPTY family. Forwarding defects return to family owner; 031 closes complete registry. |
| TASK-028 binding ownership | Main `keymap.rs:628–755` keys focused hints by real focus/flags/layer/binding-table/revision and derives effective visible chords; `ui/cx.rs:313` delegates effective lookup. Fixing only decorative HintBar paint cannot invalidate a stale shared snapshot. `verify.toml:3` now owns keymap, Ui and Cx, with 010/017/073 before 028 and 028 before 030. | Responsible shared boundary and serialization exist. W-028-04/05 test remap/removal/hidden versus routing/owner/revision and preserve 073 probe plus layer invariants. |
| TASK-022 gesture and F04 | Oracle `widgets/grid.rs:1322–1345` checks already-current unranged cell: editable begins editing; readonly activates; another cell moves. Universal single-Moved/double-Activated contradicts that branch. Oracle F04 document explicitly says deferred API defect and no established app transition. Main Grid readonly entry-point safeguards need not reproduce that unreachable old mutation defect. | README:141 D-GESTURE-AUTHORITY and W-022-01/07 preserve actual gesture and modern guards, distinguish historical proposal from oracle evidence. Remaining canonical suffix/payload synchronization is coordinator work, not permission to retain conflicting semantics. |
| TASK-025 configured paste and ADJ-09 | Oracle TablePro `app.rs:236–254` handles Dialog/Filter paste but lets Picker fall through. Holla `app.rs:480–498` checks modal stack, not open F10 menu. F01 explicitly records deferred routing fix. Main `code.rs:1021–1114` has disabled/editing/readonly checks, TextEditorCore paste, after-edit handling and scroll tail. However oracle `widgets/code.rs:609–631` gives active find-query paste priority even when the document is readonly or not editing; a document-only helper would be wrong. | Author caught and repaired that final target-selection ambiguity before final acceptance. Independently verified revised README:141 and W-025-05/06: disabled rejects; active find query uses shared single-line normalization; otherwise document eligibility applies. Runtime and public helper share target selection/mutation, with no global layer bypass, app-selected internal target or second editor. Exact oracle three-document-mode find-paste test passed. This reviewer verified source branches, not independently replayed every app route. |
| TASK-026 Diff copy | Oracle `widgets/diff.rs:736–776` explicitly expects four spaces before `let x=1` from a tab-indented diff selection. Returning a literal tab is therefore wrong here; globally expanding ordinary viewport copy is also wrong. | W-026-02 and fixed decision distinguish Diff source projection from TASK-021 raw source copying. Exact oracle regression executed successfully, below. |
| TASK-031/073 and ADJ-04 | Main architecture:6799–6815/6997–7003 explicitly accepts covered TextArea FIELD via actual changed selected resolution, unchanged sibling/theme and oracle-correct full frame. Architecture:6963 defines owned PARTS resolution union; :7532 separately requires documented SlotFn painted-cell equality. A query-only fake sentinel cannot satisfy the latter. | ADJ-04:45 preserves only the exact covered-style exception, never SlotFn waiver. 031 exact parsed declarations, finite source guards/conjunctions, same-ID children and caller-row provenance are distinct from 073 small foundation qualification. W-031-06/073-05 reject measurement-only slot credit; bypass-attribution with unchanged cells is an explicit real-source mutant. |
| TASK-073 timing producer | Main Ui:465–554 contains cache/binding paths, :605 uncached resolve, :634 actual surface color binding. Cache hit/miss counts or only `Ui::style` would miss real work. `style-timing-contract.md:7–19` includes all these boundaries plus uncovered direct calls, forbids wrapper double counting and detached cost extrapolation, compares actual disabled/calibration/measured production frames and rejects forged denominator/calibration/negative result/uncertainty. | Single testing-only caller-owned seam has a source-owning scope including testing export. Actual expensive-resolution and paint-only counter-controls, branch execution, no-allocation/cache contamination and compile/profile/clock freeze are required. These are future qualification obligations, not executed timing results. Foundation owner is coordinating the actual 070/072 observer qualification; no five-percent claim made here. |

## Finite witness semantic coverage

TASK-021: six cases cover borrowed revisions including equal-length/saturation, keyboard/mouse selection, split-run graphemes, eviction identity, follow policy and raw copying. TASK-022: seven cases cover current-cell gestures, edit modes/validation, ragged cells, reorders, actual cell paint, all 22 generic capabilities and deferred F04. TASK-023: five cases cover body callbacks, nested modal/focus lifecycle, Move-only menu hover, effective actions and resize/reopen. TASK-024: six cases cover semantic versus rendered rows, grapheme query edits, both readiness owners, palette target identity, staged query retention and completion routing. TASK-025: six cases cover readonly pointer, edit/focus distinction, diagnostics revision, find/completion targets, configured document paste and active find-query paste across document modes. TASK-026: four cases cover mode geometry, projected copy, row/scroll identity and same-ID paint.

TASK-027: six cases cover activation kind, panel callbacks, secret/public properties, five readiness owners, standalone/shared identity and too-small boundaries. TASK-028: five cases cover status priority/drop geometry, precise hover, hint fragments, effective binding state and shared-owner serialization. TASK-029: four cases cover elapsed-time progress/spinner states, tiny meter modes, exact thresholds and semantic reference precedence. TASK-030: four cases cover shared help actions/geometry and keyed wizard identity/lifecycle. TASK-031/073 each have nine finite closure/foundation cases. Every package keeps complete source obligations and inherited theme/capability/override/degenerate axes; rows cannot replace the full inventory or use an executor-selected applicability exemption. Public slot expectations come from independent parsed source promises and real cell sinks, not candidate enums or a manually duplicated expected list.

## Executed independent checks

- Pinned-main read-only worktree: `cargo test --locked -p junie-tui --test empty_style_inheritance --test independent_empty_carrier --test derived_hintbar_metadata --test grid_disabled --no-fail-fast`: **18 passed, four suites**. Existing tests bracket the repaired ownership decisions; they are not new future-task mutant qualification.
- Clock-adapted oracle worktree using the pinned source's unchanged Diff test: `cargo test --locked -p junie-tui widgets::diff::tests::tab_indented_review_aligns_context_changes_and_copied_text --lib`: **1 passed**, 173 filtered. This is the exact projected-copy case, not the entire oracle suite.
- Same oracle worktree: `cargo test --locked -p junie-tui widgets::code::tests::paste_and_backspace_edit_find_query_in_each_document_mode --lib`: **1 passed**, 173 filtered, after independently checking the source branch and revised target-aware TASK-025 helper. TASK-025 lint reran successfully after that final author change.
- Pinned taskfmt 0.2.0, explicit `/Users/donbeave/Projects/donbeave/task-format/experiment.toml`: full catalog lint **73 packages, zero errors/warnings**.
- Independent TOML dependency/scope traversal: **73-node acyclic DAG**, all repaired 027/028/030/031 predecessor joins present, all five newly required source paths present, **zero unordered overlapping writable paths**. No tree-integration receipt or future acceptance inferred from dependency metadata.
- Final local package inventory: **85/85** line counts and SHA-256 values match author-confirmed snapshot. No hash substituted for semantic review.

## Reviewed package snapshot

The 85-file inventory includes unchanged package support files for precise identity, not a claim that hashing them proves their semantics. Canonical AGENTS remain unchanged. Coordinator synchronization may legitimately change protected source-obligation payloads; rereview those deltas before final acceptance.

```tsv
path	lines	sha256
refactoring-tasks/terminal-components/completion/021/AGENTS.md	83	f26e1457d29f4c497f48e382288ff61f9092cf112910d4f1ac737b6da7305643
refactoring-tasks/terminal-components/completion/021/README.md	170	aecff730cc9a5819f392db9121d8294d8ea84ab41c02a5bb6250d305ef5d8ed4
refactoring-tasks/terminal-components/completion/021/task.toml	3	90d64dc3c0329056ee942aa83546534fe51cc057a057cde2b5eb6e0383ad033e
refactoring-tasks/terminal-components/completion/021/trusted/obligations.md	240	2c13a6b7a6aea12f2c638675f07f766c74ca0e1f836e1436c18aac777cd1918f
refactoring-tasks/terminal-components/completion/021/trusted/source-obligations.tsv	22	476e221f56fe27154aed49b21cd972ae1415d7b3628fc09cb649c4efcec7fe3b
refactoring-tasks/terminal-components/completion/021/trusted/source-witnesses.md	18	a399adf07d15ce0127cda533c3ca5779ada39f8ae9ab43f7027cac7d81df99d6
refactoring-tasks/terminal-components/completion/021/verify.toml	60	6cd3d6d7cc33d38beca66a3a6669170560581092f52a8a0d01167f5800d2cb19
refactoring-tasks/terminal-components/completion/022/AGENTS.md	83	85225a21cdf421926f1f49cbff07108c0d71d0edb28fb77a5f4facb96fa92c75
refactoring-tasks/terminal-components/completion/022/README.md	170	cf2bedd5bc2f8c9c22608022770becf9d2159dfd7dc3472b810d7cfd19308f66
refactoring-tasks/terminal-components/completion/022/task.toml	3	26f6902a1f53d09c8ba4842805c8ececc1f0ec2cad24f3f57a6b3f14c6bfc340
refactoring-tasks/terminal-components/completion/022/trusted/obligations.md	433	4a2db752ee85d528510adc2c1c50e25529ed1581b805ef6483aae1afed69aac3
refactoring-tasks/terminal-components/completion/022/trusted/source-obligations.tsv	42	aaff9f5916705741758604c3acaecb6321b1afbea430742f02beb6d7170479a5
refactoring-tasks/terminal-components/completion/022/trusted/source-witnesses.md	19	61bd746d762c33736825706ca2ab46ac8ca62f6e02c72c93b3635c239809fe3d
refactoring-tasks/terminal-components/completion/022/verify.toml	60	a46dd2993ad4e722c5c780b6006cd7fa547de87e6f4d67a7fb66581e57463e3d
refactoring-tasks/terminal-components/completion/023/AGENTS.md	83	2b1d071d970891b9fbb6b5d8690bb7905d29529252a3931f626fa6831fa9cc6f
refactoring-tasks/terminal-components/completion/023/README.md	170	fb16e54c7096f08e9604ab8e8bfdb58fe5621a1152e851c99dbed756becb292e
refactoring-tasks/terminal-components/completion/023/task.toml	3	89c001740aafa1be5573101a3bd2c129298f9cbcaba1f89908a88bb3ebcba421
refactoring-tasks/terminal-components/completion/023/trusted/obligations.md	181	144f09f8d58770835bf2d5e7f590df7659702b58fc65a04ceb57606f028b0666
refactoring-tasks/terminal-components/completion/023/trusted/source-obligations.tsv	14	372f41d026a1900f2c31feddd0adc9e91139c309e0e96a9d77bee41d31232320
refactoring-tasks/terminal-components/completion/023/trusted/source-witnesses.md	17	05774c9970bd8e960cd4d74bfd8719e3e97cfdd6a59838900c7d135f99d54fd1
refactoring-tasks/terminal-components/completion/023/verify.toml	60	ec27c3a33acc52965ae34277788de2d70c160a75bf9c2953e75406fd10f72a16
refactoring-tasks/terminal-components/completion/024/AGENTS.md	83	09bb8a0a4a307f3d26c986082f009e4974305333b87d3078ef0e38385d61753e
refactoring-tasks/terminal-components/completion/024/README.md	172	6dcc448135c6e5eb537679d9b862130bf3ad95e51b88b22268df4f00e8584434
refactoring-tasks/terminal-components/completion/024/task.toml	3	6447dffa9384add36e6157d923b63ba88606063f76da51b8fa29dececd0626bb
refactoring-tasks/terminal-components/completion/024/trusted/obligations.md	192	8117816cc84965961c453dc26f10a8f906452f8a51f3abaa9dc9857108d2c238
refactoring-tasks/terminal-components/completion/024/trusted/source-obligations.tsv	14	de3ef6cfc79d9a3c4444010006daa3d1aac551cf142cc0c36fbf0b8e367318bb
refactoring-tasks/terminal-components/completion/024/trusted/source-witnesses.md	18	45acf1b598110c0222fb64545cad67455745a44f549355e920f39f428cb6e467
refactoring-tasks/terminal-components/completion/024/verify.toml	60	82701ba5668be1ea4462e4cd39444cf5f62280096d1e91eccca7664f93f7c9a4
refactoring-tasks/terminal-components/completion/025/AGENTS.md	83	32821cb4d985c0217cceebad730c04fffe747597b3da30f850573ed3c78b543d
refactoring-tasks/terminal-components/completion/025/README.md	170	652b3e5eae5e09867fe9ce42d84cf0ab1ebcb5ec3f480ddd3c4c06bf84fddfef
refactoring-tasks/terminal-components/completion/025/task.toml	3	499a3dbb50b175902c1913d34866c9f1b8ee8a88105ff075b0591bc497f56c47
refactoring-tasks/terminal-components/completion/025/trusted/obligations.md	97	de094d70543393c42c68fccf07e2d51ee39c9ece658021fb5387121a356c4349
refactoring-tasks/terminal-components/completion/025/trusted/source-obligations.tsv	6	34fc1635416bb0c1437ee3b46fe808086ea1c9e8ee67b792c3cbcb30c71c1ecc
refactoring-tasks/terminal-components/completion/025/trusted/source-witnesses.md	18	19fdbc3fa4a534996bd62a3d9937cc14f4b11e758770a5cac43348345ccf0246
refactoring-tasks/terminal-components/completion/025/verify.toml	60	9f31d8b238b2aad4ec37e129a2645bf257955d6493280fdc39c76c7ec02e53d0
refactoring-tasks/terminal-components/completion/026/AGENTS.md	83	e3e2cafff7f76adc9c0a391420dc14c0fe38bac1bdd22996d8eb078e6b64b36e
refactoring-tasks/terminal-components/completion/026/README.md	171	bd02ee72a20710b88bdcbf5ce89516b017c71fb1bf711a1ef7b0917c077eef22
refactoring-tasks/terminal-components/completion/026/task.toml	3	9e03a8f34b2e3361821e5da2289f034cf5a726ccb634452780c98aecca7d27d5
refactoring-tasks/terminal-components/completion/026/trusted/obligations.md	75	fd363829ca065b046a6e25870ecc58023a3e13a6cc9398be17e87d0ac646d649
refactoring-tasks/terminal-components/completion/026/trusted/source-obligations.tsv	5	75db4352f65caa5a5e555ebc08ac91c23eeb0e9d421302d7b675acfc8cc4475f
refactoring-tasks/terminal-components/completion/026/trusted/source-witnesses.md	16	8ddbaa0bb7ccabd5b5fb3066b3d0fd85d232e2218f074374fc1913ea2a652366
refactoring-tasks/terminal-components/completion/026/verify.toml	60	a6eadc02bce74a37185c881c2152bc9f1ba097bbb8cb797b450036582e257b40
refactoring-tasks/terminal-components/completion/027/AGENTS.md	83	2354787905d89c348df40a29f5eb2a1a26ae1e93daf642a2a717dd769be1ef41
refactoring-tasks/terminal-components/completion/027/README.md	177	b68504398bf356e9c1e45655b2dc786b63443e9e31cdaaab0ca3ea9bfa7706b2
refactoring-tasks/terminal-components/completion/027/task.toml	3	4ea7538a6a2850e1d49f0a0e6c4b08333a527feeb856e911ecb812df93699195
refactoring-tasks/terminal-components/completion/027/trusted/obligations.md	198	d2a14ba3da3cd6de1205941b320d210ad6a8c098cc0852eaa86e92b227c9e394
refactoring-tasks/terminal-components/completion/027/trusted/source-obligations.tsv	12	619f722a5ab33e9b72c5b65ef02d5cae2274d9527a00674027916f47198ea3e3
refactoring-tasks/terminal-components/completion/027/trusted/source-witnesses.md	18	1b383a072dbbcfeb8f94554c68616ec67f7a5a8c072a568aa214326ea873bb6a
refactoring-tasks/terminal-components/completion/027/verify.toml	60	f5a0ef23707346961c03c06d61e00a7cf65b8c2454243d8588e8cce7e9771a6d
refactoring-tasks/terminal-components/completion/028/AGENTS.md	83	7d86f5f34ab6652b610a43edecdb796f2ec9fb8a2fb94b437fd85ce799b1d510
refactoring-tasks/terminal-components/completion/028/README.md	176	858fc0e292517f1514ee2e616502d18c4707e332fa964e32e31754a69776833f
refactoring-tasks/terminal-components/completion/028/task.toml	3	f51053b532ccc921020746545b4480f7e8b394b305ea236b82686647ac8651b4
refactoring-tasks/terminal-components/completion/028/trusted/obligations.md	274	77fdce380f747d928f53c4a6f2baf2f48c90dc5121a5db12e3a721ac635fcbbd
refactoring-tasks/terminal-components/completion/028/trusted/source-obligations.tsv	23	bd3b1b161b1411e885646e43a4396e636825637f87dfce9929aaff6dbd6b0e5c
refactoring-tasks/terminal-components/completion/028/trusted/source-witnesses.md	17	b263dc922298457176f50fc200f4102d42e92d59ae981044ad135a8db6084eb2
refactoring-tasks/terminal-components/completion/028/verify.toml	60	faa8347e98b36b938edad733a6b7a597ee324ea394a82598e3f2aebff15438b8
refactoring-tasks/terminal-components/completion/029/AGENTS.md	83	d5b6e0846d627c559e60b87bba04a5ba3555b361dcde6da3a11419675ef4b711
refactoring-tasks/terminal-components/completion/029/README.md	170	a5193b81b2818b09f9659a23daf19bf36a559b5e9909639e2b6e59ae9bcf6b15
refactoring-tasks/terminal-components/completion/029/task.toml	3	36c5997f4738ffad52453efe53ee5563e80b7e14fc4d90058b09973cbc6fdf42
refactoring-tasks/terminal-components/completion/029/trusted/obligations.md	157	c814a90c13ae341934bd2cef4928bf2fac54833b5f859ed837096dd897da71b1
refactoring-tasks/terminal-components/completion/029/trusted/source-obligations.tsv	10	9133a07942eb82e60592f4e248685281523f9c0d83cb1d61e93fb539f4fd92d6
refactoring-tasks/terminal-components/completion/029/trusted/source-witnesses.md	16	69a0a8d2b2b72ecc4dd67ef097a49f182de394547b4dd04b74bdbb8bafa26826
refactoring-tasks/terminal-components/completion/029/verify.toml	60	e7db585a334738e617a33332af5e7b5170ac29dbe421ec1347bd8b302688e82d
refactoring-tasks/terminal-components/completion/030/AGENTS.md	83	def099e341c0730eb7af9d4b7c90ca99bef46525b9acbca5adb3ca94829ae2c9
refactoring-tasks/terminal-components/completion/030/README.md	170	03506b26038059e1c90d9c3c98a43a73454ab9b8ef03de5569def8ebf013fcfc
refactoring-tasks/terminal-components/completion/030/task.toml	3	7b4099532460f23556fc97651fb555a4b4b4e779bbd4b0fcd79abd3bef2827a3
refactoring-tasks/terminal-components/completion/030/trusted/obligations.md	235	35325b06fb5895b6ceeefe08d6e26941ff83c327fa2a8d9d92e951a443fe726d
refactoring-tasks/terminal-components/completion/030/trusted/source-obligations.tsv	20	df4c430687538953916799951936d99d66ee9d8c156479022bad9e931d8398dd
refactoring-tasks/terminal-components/completion/030/trusted/source-witnesses.md	16	c853648276ceb30f3e636428870f130524b6e6b6f700c9e1356368435b8b096a
refactoring-tasks/terminal-components/completion/030/verify.toml	60	4a4d878b71daa209b80ecb552209723a0e456ad9849c0c86c4712e72fe4987c4
refactoring-tasks/terminal-components/completion/031/AGENTS.md	83	138c4f699df48f8e53efc82b669bbbae2ef0b94007bc146940ab910c8dadc169
refactoring-tasks/terminal-components/completion/031/README.md	138	8fde9d51f2bded827e01b325b24b5860e080d6112d3cd4b5ae9a0ea6b50eb1e9
refactoring-tasks/terminal-components/completion/031/task.toml	3	0b33ef6d9e149dd72d977984f5e23fe5668eb4c8036fa08290784448bad18770
refactoring-tasks/terminal-components/completion/031/trusted/obligations.md	646	4c19f1ff585fb8cd5ea43d4050d2de155a1f307f59972593f4bdb74c5c467c2d
refactoring-tasks/terminal-components/completion/031/trusted/source-obligations.tsv	67	9d87b33a95910de901088382171e865dccaadfe8311afe828f92245973f55e78
refactoring-tasks/terminal-components/completion/031/trusted/source-witnesses.md	21	bcead449b5de1d98f6df6b32c551cced674a7b1474a30361f80b32d81a12cdc8
refactoring-tasks/terminal-components/completion/031/verify.toml	36	9a48e243b5cec786228ec1f209eb046d025249d2caadf1c75f6b46f669ffe5b4
refactoring-tasks/terminal-components/completion/073/AGENTS.md	83	c8b04038872db37bc0a675f634d769a5762e7bf5d97c10fed48949373396384d
refactoring-tasks/terminal-components/completion/073/README.md	148	bf47581e66fafbd77be6dc7b7b91feb874a8777c6d3c023d5c2d3c91545ed7d8
refactoring-tasks/terminal-components/completion/073/task.toml	3	6c6e7f73d960f060d906eadb67ee5f3f0835477e707f5cb5617fe29e584d2fa9
refactoring-tasks/terminal-components/completion/073/trusted/obligations.md	212	7836106b7fc921ad8afff55cc98409d33f73ae2db1a4335919e9173a0160d121
refactoring-tasks/terminal-components/completion/073/trusted/source-obligations.tsv	20	c86ead2b7a6dd8df3f711719ed29fa4a95548e97a6fbb333441970c7d4e75b8a
refactoring-tasks/terminal-components/completion/073/trusted/source-witnesses.md	22	221d847c867ce3f1ba5966c1ffd52d902a272f52daf560e1da9a3728d932f666
refactoring-tasks/terminal-components/completion/073/trusted/style-timing-contract.md	19	c252b247bf1d6f58c4ef1aa729705b3f61b4c792bd93da13268a0f7e3c44c6d1
refactoring-tasks/terminal-components/completion/073/verify.toml	36	0615eba75e3605b517ed0270404c0b5292a05aa1c666400af6dcad4fbcb67dc1
```
