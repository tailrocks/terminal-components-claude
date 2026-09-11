# Fresh adversarial audit: component tasks 021–031 and 073

Audit scope: 72 task-package files, 6,848 lines at the hashes below. Read the complete PLANNING_GOAL.md and architecture-adjudication.md, component inventory and assigned trusted clauses. Task README/verify boilerplate was reviewed once and every per-task difference reviewed; canonical AGENTS files were compared after exact task-ID normalization. All 243 source-obligation rows were parsed, their requirement text and source-ledger reference compared against their full trusted Markdown clauses, and their typed requirement/acceptance/check bindings inspected. This is a semantic planning review, not execution of the future refactor or a claim that product parity passes.

Pins independently resolved/read: main `7b27732a8c3c131760ec3438f641cb3c11343a42`; oracle tag dereference `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Original source was inspected using git show/git grep at these pins. Targeted historical authorities include architecture §§33, 45, 64 and the pinned F01/F04 improvement documents; source history includes Grid/empty/diff revisions c62ec8d3, f9ba0c3a and 30924105. The report does not claim every historical source file was independently reread in full. Original component sources were inspected at their responsible API, rendering, dispatch, fixture and test sites; the package line audit is exhaustive, the repository source audit is targeted.

## Findings requiring repair

### B-01 — The Grid gesture disposition contradicts the oracle and TASK-022

Locations: `completion/022/trusted/obligations.md:325`, `completion/022/trusted/source-obligations.tsv:30`, `completion/031/trusted/obligations.md:421`, `completion/031/trusted/source-obligations.tsv:42`. Paths in this section are relative to `refactoring-tasks/terminal-components/`. Also amend EARLY-AMEND-029 at `completion/031/trusted/obligations.md:205`.

Violation: HL-64-GESTURE remains unqualified “accepted” and requires single-click Moved / double-click Activated. TASK-022 README:45 correctly requires an already-current cell without a range to edit or activate on a completed single click.

Counterexample: oracle `src/widgets/grid.rs:1322–1345` computes `same` before movement, then calls begin_edit for editable cells or emits Activated for read-only cells. The same candidate cannot preserve that behavior and satisfy an unconditional single-click-Moved witness.

Root condition: global A58 carries accepted_with_oracle_update, but supplemental source-qualified aliases retain obsolete product authority. A successful traceability join does not reconcile semantic contradictions.

Structural repair: retain historical text as historical evidence; supersede its product gesture with an explicit oracle-current disposition in every alias. Preserve the reusable PointerGesture mechanism, real semantic setup and keyed target proof. Require separate trajectories for initially current/different/ranged/read-only/editable cells, release outside and reorder between publication/action.

### B-02 — Shared EmptyState implementation is outside its repair owner's scope

Locations: `completion/027/verify.toml:3`, `completion/027/README.md:34`, `completion/027/trusted/obligations.md:75`.

Violation: TASK-027 owns collection EmptyState inheritance, geometry and all five collection-owner readiness proofs, but permits only the standalone `components/empty.rs`, omitting `collection/empty.rs`, which its own source contract names.

Concrete boundary: main `components/list.rs:1186–1192` invokes `EmptyState::draw_inherited`; its title/help/icon resolution and shared centered geometry live in `collection/empty.rs:138–220`. Changing the standalone Empty widget cannot repair that production path. TASK-015's broad collection path is constrained to row-author overrides, and TASK-008 can change inline test disposition only. A discovered wrong inherited blank/title/help/icon cell has no authorized responsible repair in TASK-027.

Root condition: task scope follows visible component filenames rather than all implementation files responsible for the contracted family.

Structural repair: explicitly assign `crates/tui/src/collection/empty.rs` to TASK-027 and serialize after TASK-015. Keep component forwarding repairs with their collection owners. Test the shared painter directly using nonzero origin, clipped/empty rectangles, all Empty/Loading/Partial/Error shapes, owner EMPTY sentinel, explicit child overrides/Clear and blank-cell inheritance; compare five production collection paths separately.

### B-03 — Binding/cache obligations lack a production repair owner

Locations: `completion/028/verify.toml:3`, `completion/028/README.md:48`, `completion/030/verify.toml:3`, `completion/030/trusted/obligations.md:211`.

Violation: TASK-028 owns A48/A50/HL-68-HINTS while TASK-030 owns HL-68-MAP/PUBLISH, but neither can repair the actual shared binding/cache/publication implementation. The complete task index names `keymap.rs` only under TASK-008's inline-test exception.

Counterexample boundary: main `keymap.rs:638` owns FocusedHints; `ui/mod.rs` owns the UiCore keymap/revision/descriptors; `ui/cx.rs:313` resolves component chords through that shared keymap. A stale structural-map replacement or descriptor-publication failure cannot be repaired in HintBar, HelpOverlay or Wizard without introducing the component-local cache/resolver the tasks forbid. This is a proven scope gap, not a claim that the existing cache is defective.

Root condition: preservation/proof clauses are assigned to consumer tasks without an authorized owner for a failed lower-layer invariant.

Structural repair: give one task narrowly bounded production ownership of `keymap.rs`, `ui/mod.rs` and the necessary Cx binding accessors; serialize it after TASK-073 and all other writers of those paths. Dependent help/menu/hint proofs must use the shared resolver. Freeze positive and negative witnesses for same-focus structural remap/removal, unchanged-map revision/allocation stability, dynamic descriptor changes, hidden/latent commands, duplicate diagnostics and owner-local isolation.

### B-04 — Deferred Improvements behavior is routed into an unconditional oracle comparator

Locations: `completion/022/trusted/source-obligations.tsv:17` (F04), `completion/023/trusted/source-obligations.tsv:7` (F01), `docs/refactoring-plan/history-ledger-reconciliation.md:14`.

Violation: the source rows map later_open behavior to R-001/AC-001/CHK-004 exact oracle parity. The oracle's own F01 and F04 files explicitly say Deferred and describe behavior opposite the proposed invariant. F01's clause says the oracle conflict “must be documented”; the task graph is not execution-ready while the disposition is left to future execution.

Counterexamples: oracle `docs/improvements/f04-reconcile-datagrid-read-only-transitions-before-mutation.md:20–32` records begin edit, disable editing, paste, commit still writing pending data. Oracle F01 records Ctrl+O picker over an editing query retaining an empty picker while hidden query receives pasted SQL. The independent TablePro auditor also traced the reachable Ctrl+O/Ctrl+G route. Preserving that exact app trajectory and enforcing universal modal-first paste cannot share one expected value.

Root condition: an improvement tracker entry is treated as accepted implementation work merely because it maps to a task; “later_open” is not an acceptance disposition.

Structural repair: the coordinator must record an explicit source-qualified authority decision. Separate accepted reusable architecture invariants from immutable app trajectories. Do not fabricate oracle equality for a changed interaction or let the executor choose an exception. Preserve deferred source observations and freeze exact applicable source classes and observations.

## Clarification worth fixing

TASK-026 README:45 says “source-preserving selection/copy.” For Diff, the relevant source is the oracle's projected diff text. Oracle `src/widgets/diff.rs:770–772` explicitly expects `"    let x=1"` after selecting a tab-indented hunk; raw hunk text begins with a tab. TASK-021's ordinary viewport raw-source policy must not silently change Diff to raw-hunk copy. TASK-026 already names this exact oracle test, so this is ambiguous prose rather than absent regression coverage. Add a fixed decision and paired raw-viewport/projected-Diff copy vectors.

## Semantic coverage and surviving contracts

- 021: revisioned borrowed output; retained reading/caret/selection/marks; append/replace/front eviction; Shift selection; raw-source copy; non-tailing End; warm/cold/append/reflow work. Source owns a real missing behavior class; no proof of current parity inferred.
- 022: GridModel/GridEditor split, ragged holes, source keys, optional gutter/header APIs, validation, fetch row and oracle current-cell click. B-01 and B-04 prevent acceptance.
- 023: bare-R Dialog callback, once-at-empty body, runtime layer placement, enabled Move menu handling, typed actions, dismissal/restoration. B-04 needs a binding disposition.
- 024: AsItem independent from RowFn, borrowed sources, original grapheme indexes, completion insert fallback and editor-owned bindings, stage/back/error/retry. The family owns its production files; shared empty/resolver concerns remain cross-task.
- 025: oracle readonly caret versus main's readonly pointer guard; completed-click editing, shared text grammar, revision caches, diagnostics/find/completion. Main code.rs:1079 guards the pointer branch with !read_only; oracle code.rs:535 explicitly moves readonly caret. The task addresses this actual difference.
- 026: borrowed DiffSource, update-owned projection, requested Review retained through effective Unified fallback, scrollbar budget and complete grapheme emphasis. Oracle thresholds 42/43 and actual scrollbar allocations 43/44 are present in the named test. Copy clarification above applies.
- 027: standalone chrome, borrowed Props copy/secret containment, actual EmptyState owner inheritance, TooSmall four text rows with blank separation. B-02 identifies its missing implementation owner.
- 028: exact status group padding/drop/truncation, keyed hover, hints/chords and fresh fixed-buffer projection. B-03 identifies missing shared ownership.
- 029: deterministic progress/spinner phase and exact ASCII sequence, Meter line/block/no-ratio/Series, all threshold boundaries, authored alias policy and explicit rest colors. Historical changed digests are explicitly not accepted oracle evidence.
- 030: Help effective metadata and keyed Wizard retained step state. Generic jobs/domains remain outside the library. B-03 applies to its shared resolver clauses.
- 031: complete generated subject enumeration, owned-PARTS equality, real-state fixture sweep, exact public documented slot promises versus actual sentinel-painted cells, independent negative mutations, no extra set or fake IDs. Correctly distinguishes covered resolution from visible slot paint; retained TextArea FIELD query proof does not excuse ignored slots. B-01 applies to inherited gesture aliases.
- 073: foundation is independently qualified observation/registry, not premature all-family equality. Positive same-ID composition and caller custom parts; truthful original identity; nested callback restoration; measure-versus-paint; registry omission and ignored-slot negatives. Final closure remains TASK-031. This split is coherent.

The typed checks use exact host contexts. Direct capture, PTY diagnostic capture, comparison, regression accounting, architecture and final gate remain distinct. No task can silently claim unrepaired full-app PTY parity. The canonical AGENTS differences are task identifiers only; campaign binding explicitly overrides the legacy executor-authoritative invocation. No AGENTS file was changed.

## Per-file evidence inventory

Hashes describe the reviewed pre-repair files, not a future corrected result.

```tsv
path	lines	sha256
refactoring-tasks/terminal-components/completion/021/AGENTS.md	83	f26e1457d29f4c497f48e382288ff61f9092cf112910d4f1ac737b6da7305643
refactoring-tasks/terminal-components/completion/021/README.md	168	7a400b68922e40db9c52eb5883b02570bce3d3211af491ce8fd18dca091add43
refactoring-tasks/terminal-components/completion/021/task.toml	3	90d64dc3c0329056ee942aa83546534fe51cc057a057cde2b5eb6e0383ad033e
refactoring-tasks/terminal-components/completion/021/trusted/obligations.md	238	9ac36e3a8df0a2bae7a4f7c6c1cecb0e7dbd6546f7bab43954752b65c080a070
refactoring-tasks/terminal-components/completion/021/trusted/source-obligations.tsv	22	476e221f56fe27154aed49b21cd972ae1415d7b3628fc09cb649c4efcec7fe3b
refactoring-tasks/terminal-components/completion/021/verify.toml	60	6cd3d6d7cc33d38beca66a3a6669170560581092f52a8a0d01167f5800d2cb19
refactoring-tasks/terminal-components/completion/022/AGENTS.md	83	85225a21cdf421926f1f49cbff07108c0d71d0edb28fb77a5f4facb96fa92c75
refactoring-tasks/terminal-components/completion/022/README.md	168	f89cdfcfbed887840a73bf2d31e0e254856b3296d7397eea402c17382d8833fe
refactoring-tasks/terminal-components/completion/022/task.toml	3	26f6902a1f53d09c8ba4842805c8ececc1f0ec2cad24f3f57a6b3f14c6bfc340
refactoring-tasks/terminal-components/completion/022/trusted/obligations.md	431	188156dc6e22677876dff3074927416383c437cc6b47480535d4a3b7801f6979
refactoring-tasks/terminal-components/completion/022/trusted/source-obligations.tsv	42	aaff9f5916705741758604c3acaecb6321b1afbea430742f02beb6d7170479a5
refactoring-tasks/terminal-components/completion/022/verify.toml	60	a46dd2993ad4e722c5c780b6006cd7fa547de87e6f4d67a7fb66581e57463e3d
refactoring-tasks/terminal-components/completion/023/AGENTS.md	83	2b1d071d970891b9fbb6b5d8690bb7905d29529252a3931f626fa6831fa9cc6f
refactoring-tasks/terminal-components/completion/023/README.md	169	204f85447d775033e0ae0063900fbd72eaf548b7724c3eaf66626ab00a997743
refactoring-tasks/terminal-components/completion/023/task.toml	3	89c001740aafa1be5573101a3bd2c129298f9cbcaba1f89908a88bb3ebcba421
refactoring-tasks/terminal-components/completion/023/trusted/obligations.md	179	391448e97c01a18d0fbe82209e7473e1ccfeb4f4e55fddd0cef11af25aa96341
refactoring-tasks/terminal-components/completion/023/trusted/source-obligations.tsv	14	372f41d026a1900f2c31feddd0adc9e91139c309e0e96a9d77bee41d31232320
refactoring-tasks/terminal-components/completion/023/verify.toml	60	ec27c3a33acc52965ae34277788de2d70c160a75bf9c2953e75406fd10f72a16
refactoring-tasks/terminal-components/completion/024/AGENTS.md	83	09bb8a0a4a307f3d26c986082f009e4974305333b87d3078ef0e38385d61753e
refactoring-tasks/terminal-components/completion/024/README.md	171	6bc7253a6dce70bdd5d0e9be0305259e7c805af44e9854e3141f2001a442fe4a
refactoring-tasks/terminal-components/completion/024/task.toml	3	6447dffa9384add36e6157d923b63ba88606063f76da51b8fa29dececd0626bb
refactoring-tasks/terminal-components/completion/024/trusted/obligations.md	190	9b275a2da77dfff572e39a2dca5482bc713539967056b0441c10f0aaada0a4a7
refactoring-tasks/terminal-components/completion/024/trusted/source-obligations.tsv	14	de3ef6cfc79d9a3c4444010006daa3d1aac551cf142cc0c36fbf0b8e367318bb
refactoring-tasks/terminal-components/completion/024/verify.toml	60	82701ba5668be1ea4462e4cd39444cf5f62280096d1e91eccca7664f93f7c9a4
refactoring-tasks/terminal-components/completion/025/AGENTS.md	83	32821cb4d985c0217cceebad730c04fffe747597b3da30f850573ed3c78b543d
refactoring-tasks/terminal-components/completion/025/README.md	168	226d5d4b02d08bcdfb63a5a256c3fe16f7b5fac77cb94023aa900471f93f0733
refactoring-tasks/terminal-components/completion/025/task.toml	3	499a3dbb50b175902c1913d34866c9f1b8ee8a88105ff075b0591bc497f56c47
refactoring-tasks/terminal-components/completion/025/trusted/obligations.md	95	8e39af55acccd6904678dcebe38ba9de202ff9f99dba45dd692c545d401c7455
refactoring-tasks/terminal-components/completion/025/trusted/source-obligations.tsv	6	34fc1635416bb0c1437ee3b46fe808086ea1c9e8ee67b792c3cbcb30c71c1ecc
refactoring-tasks/terminal-components/completion/025/verify.toml	60	9f31d8b238b2aad4ec37e129a2645bf257955d6493280fdc39c76c7ec02e53d0
refactoring-tasks/terminal-components/completion/026/AGENTS.md	83	e3e2cafff7f76adc9c0a391420dc14c0fe38bac1bdd22996d8eb078e6b64b36e
refactoring-tasks/terminal-components/completion/026/README.md	168	3cbc012361d8dbbf5b10bef68fb9ddeb4741a1291aa87d6ef4667bcde5afd532
refactoring-tasks/terminal-components/completion/026/task.toml	3	9e03a8f34b2e3361821e5da2289f034cf5a726ccb634452780c98aecca7d27d5
refactoring-tasks/terminal-components/completion/026/trusted/obligations.md	73	1a50c348280309a053f66148498fc5d9142a3cc1914ae30c68bd1d82300ff3ce
refactoring-tasks/terminal-components/completion/026/trusted/source-obligations.tsv	5	75db4352f65caa5a5e555ebc08ac91c23eeb0e9d421302d7b675acfc8cc4475f
refactoring-tasks/terminal-components/completion/026/verify.toml	60	a6eadc02bce74a37185c881c2152bc9f1ba097bbb8cb797b450036582e257b40
refactoring-tasks/terminal-components/completion/027/AGENTS.md	83	2354787905d89c348df40a29f5eb2a1a26ae1e93daf642a2a717dd769be1ef41
refactoring-tasks/terminal-components/completion/027/README.md	173	ee08a480d86e0e682bc9dfcb485544accd534a7624e56cb8c40988dbdb38951a
refactoring-tasks/terminal-components/completion/027/task.toml	3	67d382a230de4541557894b1705ef00548e4021d7fcc7a49154a62757ddad3eb
refactoring-tasks/terminal-components/completion/027/trusted/obligations.md	196	bb71cd02e5c8358bea1d381c19d51005aac97d662dd9d6d643a8f280df160eee
refactoring-tasks/terminal-components/completion/027/trusted/source-obligations.tsv	12	619f722a5ab33e9b72c5b65ef02d5cae2274d9527a00674027916f47198ea3e3
refactoring-tasks/terminal-components/completion/027/verify.toml	60	0bcda6497b89e7ccf92fe98589a87cfcf46cf5743db71400b2d745d20c786eb6
refactoring-tasks/terminal-components/completion/028/AGENTS.md	83	7d86f5f34ab6652b610a43edecdb796f2ec9fb8a2fb94b437fd85ce799b1d510
refactoring-tasks/terminal-components/completion/028/README.md	170	76105a3714a4ce5819dc003ca6c83f5039b236f84ef674a1a8447fd1996757d5
refactoring-tasks/terminal-components/completion/028/task.toml	3	34d9220e917994fdd02944d01aac036d42ec450ca35335d9b93f0444032cf858
refactoring-tasks/terminal-components/completion/028/trusted/obligations.md	272	14ecfc6ccc06120ebbcac46aadf77b47f29d6fe129b66f789172a97a2af695d0
refactoring-tasks/terminal-components/completion/028/trusted/source-obligations.tsv	23	bd3b1b161b1411e885646e43a4396e636825637f87dfce9929aaff6dbd6b0e5c
refactoring-tasks/terminal-components/completion/028/verify.toml	60	e4c30c0987539eec7d4dae2121a1d671c697d8a9f8b8e415f75dd9c6a9be4079
refactoring-tasks/terminal-components/completion/029/AGENTS.md	83	d5b6e0846d627c559e60b87bba04a5ba3555b361dcde6da3a11419675ef4b711
refactoring-tasks/terminal-components/completion/029/README.md	169	70d13126c6ef30f348397b9020a108adb170a6905e035a7498b033c203199cdb
refactoring-tasks/terminal-components/completion/029/task.toml	3	36c5997f4738ffad52453efe53ee5563e80b7e14fc4d90058b09973cbc6fdf42
refactoring-tasks/terminal-components/completion/029/trusted/obligations.md	155	c16a27ce4df63e870b9e8ed6a5420b5cd214ea226ffc45cb74215ec5ccdd3cc6
refactoring-tasks/terminal-components/completion/029/trusted/source-obligations.tsv	10	9133a07942eb82e60592f4e248685281523f9c0d83cb1d61e93fb539f4fd92d6
refactoring-tasks/terminal-components/completion/029/verify.toml	60	e7db585a334738e617a33332af5e7b5170ac29dbe421ec1347bd8b302688e82d
refactoring-tasks/terminal-components/completion/030/AGENTS.md	83	def099e341c0730eb7af9d4b7c90ca99bef46525b9acbca5adb3ca94829ae2c9
refactoring-tasks/terminal-components/completion/030/README.md	169	f85f12f981ea845ed785e64104d6573624ef46306c5d946ffc29fb65797dfe24
refactoring-tasks/terminal-components/completion/030/task.toml	3	7b4099532460f23556fc97651fb555a4b4b4e779bbd4b0fcd79abd3bef2827a3
refactoring-tasks/terminal-components/completion/030/trusted/obligations.md	233	da33d5730f5423857a7ba323f33acfb5321eb8f8de4078e4a0fb01efff55a742
refactoring-tasks/terminal-components/completion/030/trusted/source-obligations.tsv	20	df4c430687538953916799951936d99d66ee9d8c156479022bad9e931d8398dd
refactoring-tasks/terminal-components/completion/030/verify.toml	60	4a4d878b71daa209b80ecb552209723a0e456ad9849c0c86c4712e72fe4987c4
refactoring-tasks/terminal-components/completion/031/AGENTS.md	83	138c4f699df48f8e53efc82b669bbbae2ef0b94007bc146940ab910c8dadc169
refactoring-tasks/terminal-components/completion/031/README.md	137	a90afa6f23907456ab6973280c88756fe721bbaf18fd00f9247959c20e2006c8
refactoring-tasks/terminal-components/completion/031/task.toml	3	0b33ef6d9e149dd72d977984f5e23fe5668eb4c8036fa08290784448bad18770
refactoring-tasks/terminal-components/completion/031/trusted/obligations.md	644	bc0417d95b90b78b7405114ced90a192d940c0b7da13d3f2a7156a55dd15d7c3
refactoring-tasks/terminal-components/completion/031/trusted/source-obligations.tsv	67	9d87b33a95910de901088382171e865dccaadfe8311afe828f92245973f55e78
refactoring-tasks/terminal-components/completion/031/verify.toml	36	9a48e243b5cec786228ec1f209eb046d025249d2caadf1c75f6b46f669ffe5b4
refactoring-tasks/terminal-components/completion/073/AGENTS.md	83	c8b04038872db37bc0a675f634d769a5762e7bf5d97c10fed48949373396384d
refactoring-tasks/terminal-components/completion/073/README.md	143	b78c851af21464633a608296e1c3ce733f6cc64ca0e6de8dc2db2a3e1da13d41
refactoring-tasks/terminal-components/completion/073/task.toml	3	6c6e7f73d960f060d906eadb67ee5f3f0835477e707f5cb5617fe29e584d2fa9
refactoring-tasks/terminal-components/completion/073/trusted/obligations.md	210	16ab5ff34da2c5041db924a15b622a05397305eb995b1e0fc56a6a606b458513
refactoring-tasks/terminal-components/completion/073/trusted/source-obligations.tsv	20	c86ead2b7a6dd8df3f711719ed29fa4a95548e97a6fbb333441970c7d4e75b8a
refactoring-tasks/terminal-components/completion/073/verify.toml	36	b83b92a941290c02b9c2b10cb0ca7472681665c3deb5f4a208380f38fca837fc
```

## Authorized planning repairs and recheck

Repairs completed within TASK-021–031/073 packages only; canonical AGENTS and production source remain unchanged. Added concrete finite source/state/observable/real-mutant companions for all twelve tasks, source anchors for021–030, and independently sealed guard/sink census contracts for031/073. These strengthen the future executable obligations; they are not a claim that future Rust witnesses or all semantic source paths have already passed.

- B02:027 now owns collection/empty.rs narrowly after015 plus repaired five-owner forwarding predecessors020/022/024. Standalone Empty and shared readiness painter remain distinct owners with one shared layout policy.
- B03:028 now owns keymap.rs, ui/mod.rs and ui/cx.rs narrowly for effective binding metadata/cache publication;017 is an explicit predecessor.030 consumes this boundary.073 attribution/timing must survive the later writes.
- B01/B04:022 fixed decision records parent/historian's oracle-current gesture supersession and deferred F04 disposition while preserving accepted main readonly guards. Shared ledgers and derived source-obligation payload synchronization are coordinator-owned; old hashes below must be refreshed by the final coordinator audit after that synchronization. No executor is delegated an authority choice.
-021 binds013 whole-logical-line grapheme painter/first-byte source style.025 supplies configured target-aware paste through the shared editor path, not a modal passthrough: final source recheck of oracle code.rs:609–631/1127–1140 requires active find query priority even with readonly/nonediting document.026 fixes projected Diff copy versus raw viewport copy with exact oracle tab-vector and width/scrollbar thresholds.
-073 owns the one stable-Rust caller-scoped actual style timing seam. Its separate trusted timing contract covers three real modes, complete all-call resolver census, calibrated/error-bounded numerator/denominator and independently executed source/arithmetic mutants.067 consumes this seam, not query-count extrapolation.

Checks: all twelve package taskfmt lint runs passed with zero errors/warnings using /Users/donbeave/Projects/donbeave/task-format/experiment.toml. The read-only diagnostic re-parsed every assigned TOML/source row, checked README scope/check bindings and normalized unchanged AGENTS. Parent's independent closure reviewer reports full73-node DAG acyclic and no unordered scope overlaps. No future production implementation or campaign acceptance run is claimed. Earlier exploratory shell failures (missing explicit taskfmt config, unsupported rg label option) were diagnosed and replaced; successful checks above used supported commands.

### Post-repair inventory before coordinator payload synchronization

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
