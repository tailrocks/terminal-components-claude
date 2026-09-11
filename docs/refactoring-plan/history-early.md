# Early architecture history: direct-source ledger

This ledger covers the initial `COMPONENT_ARCHITECTURE.md` and decisions through §29 (Adjudication Q), plus selected later amendments to early contracts. It is planning evidence, not a claim that the implementation satisfies those contracts. Main is pinned to `7b27732a8c3c131760ec3438f641cb3c11343a42`; the controlling Holla oracle is `02f5294bfdbf38004cc49130d0aff1d01f31434c`.

## Authority and coverage

Original sources were read with `git show <commit>:COMPONENT_ARCHITECTURE.md` and parent-relative `git diff`. Large original additions were read in overlapping chunks; adjudication sections were read at the revision introducing them. An authoritative decision section is evidence for its decisions, but does not prove every simultaneous inline signature/example edit was inspected. Such revisions remain partial below.

The initial 1–15 document and the 16–20/appendix addition are directly inspected originals. J–Q decision sections are directly inspected. The complete parent-relative inline deltas at 69fcdcad, 587c53bd, 4aabceb7, dc3e0fa1, 70dacec1 and 3ed377e3 are now directly inspected too; the main historian owns the remaining J/K/L/M inline reconciliation at 95ab6529, 27bd918e and 87ab93d4. Later amendments listed as direct are inspected original deltas, not deductions from the latest cumulative document. The early-amendments reader owns remaining later §§1–29 deltas. Reviews from other agents supply navigation only until their independently documented evidence is joined. REFACTORING_GOAL versions belong to the main historian; initial linked audits/reviews and early REFACTORING_STATE belong to the history-inputs reader.

A changed blob is not evidence of a changed requirement. The complete revision inventory contains repeated WIP blobs, reverts, cherry-picks and merges. Identical after-blobs allow content reuse only after the relevant content is directly read; different parent edges still need their delta classified. No pending edge below is promoted to complete merely because its blob appears elsewhere.

PLANNING_GOAL's immutable Holla parity requirement controls over old §20.10 permissions for intentional differences. Historical approval to change RadioGroup keystrokes, Select Tab behavior, dimming, status priority, hints, surface alias behavior or mono output does not authorize a changed observable today. Preserve the architectural ownership boundary while proving the exact pinned app behavior. The early architecture expressly protected rain timing, scenario behavior, minimum-size wording and grapheme/ellipsis output even under its looser old policy.

## Accepted foundation and rejected alternatives

The accepted design makes durable state caller-owned, properties configuration-only, and update/draw separate. Update may mutate state and emit typed actions but has no Buffer. Draw receives shared self/state/model and only paints, registers candidate geometry or fills derived runtime caches. J fixes per-phase item borrowing so captured render/config closures do not retain the data slice across mutable updates. Three concrete generic builder stages avoid changing all generic parameters at once. No universal Widget trait, runtime-owned component tree, fused show/on phase, Box<dyn Any> state, per-widget theme generic, retained semantic geometry, compatibility facade, or library-owned async/product model is accepted.

Identity uses kind-tagged, separator-delimited stable Id and ItemKey derivation. Public geometry/cache fields are rejected. Keyed focus, selection, range anchors and strip windows must survive insertion, removal and reorder; positional rendering cannot masquerade as keyed state. Registry, hit/ring/capture/layer/cursor facts are runtime-owned. Empty/tiny drawing publishes no stale registration. Layers own traps before paint, open-time IDs, lifecycle and pointer barriers; composition copies written cells only, independent of component draw order.

The author boundary includes only curated foundational types and shared paint/registration services. L/M choose ratatui-core vocabulary and ratatui-crossterm reexports rather than duplicate foreign geometry/paint types; app code uses the facade. Size remains the intrinsic min/preferred type, and role-aware Span remains separate from foreign raw text. Ui does not construct components: later §35 explicitly strikes Ui::scroll_region as impossible and layering-inverting, rather than deferring it. Owners call ScrollRegion's update/draw phases directly.

## Corrections builders must not lose

J distinguishes action, consumption, change and repaint. Only Response<()> supports bitwise combination. Frozen owner-indexed intent iteration cannot borrow mutable Cx for its whole loop; separate services/paste arena preserve borrowing. Empty queues do zero probes; focus reruns do not redeliver an intent and terminate at a bounded pass count. Pending focus survives resize and the fifth request until another handle.

Field is idless chrome; the child control retains identity, input and ring registration. Form is layer-agnostic library composition, not a second focus runtime. K defines ordered controlled fields, hidden-draft preservation, pure measured height, local and cross validation, first-error focus/reveal, submit only after blur/commit, declared-action order, and ID/action-only results. No cloned values() result bundle is accepted. M chooses closed heterogeneous FieldKind and borrowed per-phase FormData options, with value_and_options solving the simultaneous mutable-value/shared-options borrow. Richer domain pickers use Chooser/app composition. The early architecture still calls the standalone FieldControl item channel open, but direct inspection of REFACTORING_STATE at ecc13378, lines 839–843, records the disposition: item-bearing controls use direct per-phase paths and Form drives its three choice controls directly; future trait widening remains open, not required implementation. Direct §67 inspection at that revision confirms configured-control private Form bridges and inherited-disabled OR configured-disabled without changing public standalone APIs.

Grid has shared GridModel update and a separate mutable GridEditor update_editable path. Read-only reasons and cell actions remain available to a GridModel-only consumer. SQL values, validation, mutation logs, undo and save/discard/preview stay in the TablePro adapter; all 22 DOM capabilities must be mapped, not discarded during generic extraction. Later gutters/header-prefix additions are actual optional paint surfaces, not authority to restore whole-grid Status.

N replaces the false min-size sentinel with LayerSize::Fill/Fixed. Fixed zero is genuinely empty; the runtime alone resolves anchored geometry. Reanchor/resize happens on update and must agree with same-frame draw. Dialog measures from props/design tokens (including prompt field rows), never reconstructs or recenters the layer independently. Ui::resolve and Theme::metrics are sizing paths without painting-record/cache side effects; Ui::style/with_part are painting paths. Surface inheritance is the left operand of the final Style patch.

## Styling, glyphs and conformance

Roles bind after the full precedence chain. Family and variant state rules interleave by specificity and stable declaration order; applying variant base after family state is wrong. Fixtures must use distinct bound roles so precedence cannot pass vacuously. Glyph Slot::Inherit, Set and Clear remain distinct: Clear reserves a blank cell, not zero width. PARTS and slot documentation require actual component-owned resolution/paint effects, not enums, arbitrary allowances or comments alone.

J's CIE76 nearest-16 proposal was rejected by §25 in favor of the exact categorical legacy mapping. O corrects an additional false estimate: border_subtle #262626 maps to DarkGray, not Black. Generic downgrade still matters for custom/Paper themes; later authored Junie tables add a separate explicit semantic path. The latest provenance amendment compares a slot to its actual last projection, including intermediate levels without authored tables. A slot changed by the author permanently loses eligibility until a new carrier is explicitly installed. Clone isolation, table-content equality/fingerprint and runtime cache invalidation are obligations.

P adds readable disabled DIM rules, union-based state coverage, and real readiness fixtures; Q's reason-string containment is insufficient to prove honest narrowing. Later §38 introduces REPORTS_STATUS and later §72 supersedes component-local forced-state mechanisms with inert reference projection. These later decision sections are owned by the middle/late readers; their pointers do not count as full direct delta coverage here. Never implement the earlier component state_override/inherit_forced design as current authority.

O fixes the style cache at two ways ×128 sets, generation-sensitive, one allocation, with a generation-wrap stale-entry test. Hit rate is the named workload's requirement, not a promise for arbitrary adversarial keys. Full ASCII requires every typed glyph and scrollbar/rule field, not just absence of box-drawing characters; no-box-drawing accepts non-ASCII checkmarks and is insufficient. §35 moves remaining value decisions to a serial theme-owned package, rejects Unicode autodetection, and preserves last-write-wins builder order.

Q keeps Select a nontrapping popover. Runtime FocusOut dismisses it, without restoring the opener and swallowing Tab. TRAPS_FOCUS is separate from OVERLAY; generic trap tests must actually have a trap/nonempty ring and test wrapping even with one stop. This is a historical architecture contract requiring explicit reconciliation with the pinned app's observable key semantics, not automatic permission to change them.

## Later direct amendments to early ownership

The publication contract supersedes immediate opener restoration. Close lifecycle and closing-owner FocusOut are immediate; opener FocusIn/input requires a successfully published new ring proving the opener enabled, present and admissible in current layers. Invalid openers use normal survivor reconciliation. Aborted/dropped paint or Scene projection cannot acknowledge restoration. The original next key is retained until publication and settlement; this preserves the obligation behind the renamed test rather than deleting it.

TypingPolicy::Fallback explicitly permits text/paste/editing ownership without navigation-focus movement. Complete candidate geometry selects a unique enabled, editing target on the top admissible layer; ambiguity diagnoses and chooses none. An idle/read-only primary editor still blocks another fallback. Contextual bind_before_typing is an owner-scoped product exception. Cursor offers require matching owner/layer declarations; unselected valid offers are silent, undeclared offers diagnose, raw set_cursor retains its rejection rules. Scene inspection uses the same explicit snapshot/model without live publication or updates.

FeedbackClock::Simulation is distinct from elapsed time and from event count. It begins at the sought fixture epoch; only admitted coalesced domain-time steps synchronize it. Equal is idempotent, backwards/wrong-policy reject atomically, paused simulation has no wall expiry/deadline spin. Future owner/part activation feedback grants no focus/input authority and preserves source-visible cadence after disappearance.

Dynamic form security tightens the public boundary: FormState::error returns generic Invalid value for every slot, because it cannot know a dynamic owner's current sensitivity between updates. Only Form's private rendering path can expose current plain-field detail. Hidden/inactive fields reconcile in update. Direct secret-String lifecycle must use sensitive() before begin copies bytes; ordinary control update establishes sensitivity first. Secret::expose is crate-private, masks take explicit SecretPolicy, and zeroization is best-effort under safe-Rust MA-13, not guaranteed erasure. The enclosing owner must zeroize on cancellation/dismissal; Form does not receive or own layer lifecycle events.

Meter's original semantic-enum RaisedSurface amendment was itself superseded by ReferenceLift: compare the resolved current color to Canvas first, Surface/Elevated next, Field next, otherwise Popover. Ordering under capability aliases is normative source behavior. This is an explicit authored color policy, not permission to infer painted roles from arbitrary RGB. Typed role/source-surface provenance remains necessary for delayed dimming. No amendment blesses baselines.

## Planning deltas and remaining proof

The companion obligations TSV extracts historical requirements and supersessions for the parent plan. Every row still needs an implementation location, deterministic acceptance proof, actual code status and work-package assignment from the present-tree audit. A source statement or historical green test is not current proof.

Highest-risk unresolved requirements:

- FieldControl's stale open wording must be reconciled with the directly inspected STATE disposition and §67 bridges. Verify direct per-phase composition and disabled propagation; do not invent a task to widen the scalar trait.
- Choice/Brand pressed-bracket applicability was not adjudicated by Q. Read subsequent exact decisions and current shared reference capability rather than extrapolating Button's reserved padding.
- Exact API exports, inline examples, Rustdoc/checker coverage and original R1–R20/API and DOM acceptance maps require source-to-implementation joins; token presence is not signature or compilation proof.
- Old tests can be vacuous: wrong baseline file, one-digit rows that never overflow, shared buggy family enumeration, a Clear branch conflated with Inherit, one-stop trap skipping, or fabricated mono reasons.
- App screenshot/perf baselines, capture matrices, frozen-evidence rules and serialized ownership were repeatedly corrected after Q. Their original estimates and early sequencing dates must not become current acceptance.
- The full indexed parent-delta review remains open wherever marked below. Unread WIP/merge parents are not discarded as duplication without content and lineage proof. The original linked audit/review reader's ledger must be joined separately.

## Per-revision coverage

All rows use path `COMPONENT_ARCHITECTURE.md`. Commit + parent + path is the exact join key into `history-revision-index.tsv`; after-blob identifies the source content. Full semantic parent-delta means the changed text and its removed counterpart were directly read (repeated long unchanged test lists may be read once through additions plus the original). Partial means only the stated ranges are credited. Pending means no direct credit from this reader, even if another reader inspected a later adjudication section.

| Commit | Parent | After blob | Direct coverage |
|---|---|---|---|
| 2e4530236995bb375fca49bb3582acf5904ae7a8 | cefc4b8af27226d6b6494b49dd5224ba2bd48f6f | c241a6578c85a28f83ff5f6e330fcb77bffd6356 | Direct original/full semantic parent delta |
| e8d053c9bd47837986fcc87ff4ebaba80aeb5fd5 | 2e4530236995bb375fca49bb3582acf5904ae7a8 | 910280b3eb64574b8dfd1eade2d0147c524deca8 | Direct original/full semantic parent delta |
| 95ab652983bf6c8e77c727bd31eeeaf93b0e7635 | e2be0ced5cf12e78aab12f88f03ed7435c5a36b3 | d4d8ceafcf8d09393ec94049947cdf51a3a69717 | Partial: §21 complete; initial inline delta excerpt; remaining inline signature/example delta not yet independently reconciled |
| 27bd918e3a8a0a7fdba14fb10643139340d6281f | f2d30b654e5c6bea154392a8be6e9f4b7d4dbcb1 | 41ffb0bbda75082b73c608a360a3e71c11f6b347 | Partial: §§22–23 and new §15.1 Form complete; remaining inline signature/example delta pending |
| 87ab93d4d21b75ce975c3a5cfecd987a0b935d2f | d57ed9ff43a8f6845eaa298969826c9e14b8a122 | 05eaf05038737b596f14961de90358421b05ee56 | Partial: §24 complete; inline parent delta pending |
| 69fcdcad413375a181c72a144b776e08a7b2b5fd | fa87a59c6d1028bc08515132e1f7719624bf9a08 | 1ae808efa30e6d2c5b46ba779fadf36af043cba8 | Direct original/full semantic parent delta
| 587c53bdfaa6f391b686f2383b5675194d9ba9f8 | 1129ab1c0225548f3a2bd20cae5e33aa87fbc5a5 | 64c0f6911f2f7b89166ab62cd9c632b7203812bf | Direct original/full semantic parent delta
| 4aabceb774f4d35e6cecfd326e5dc8bc1ee40461 | eed8187221c1aa4b4d92f4294bb8c09a7a29ade7 | 26987b4ad93c489dcae164e03d5d631f05e795ef | Direct original/full semantic parent delta
| dc3e0fa1be9e4896b246a477958895deed072ff0 | 1653fb133e979aea6f733f6e0ca10bbb3a1256a2 | 0c17e2d06d1285ef23cf56a689dd7cb907e03300 | Direct original/full semantic parent delta
| 70dacec1dd527218bdba394675938a9e9c4de285 | ad94d12ad582ca92d0866d01c727804bf42fdbb3 | 2d161ea3bf2c3e75f248f04824664607641fd7ce | Direct original/full semantic parent delta
| 3ed377e3046bff295fec662ddeca350acf971215 | 98c7ac515ec2166ec908ccdd9f410f7005b3cc07 | 5eb5176abbf41976b0dbe2efae9a4eca014172a8 | Direct original/full semantic parent delta
| 9486b654ed006bb9aa916c2d2ca9087a18ec42d8 | 95f486f0b51148d378a2cf5f22ac5a8c67129bb5 | 19f6ff13c6ab278cb2cfd9248bddb85e369dfb84 | Pending direct early-section delta inspection |
| afc60678dc088387a1b49cc6e0c79780e6d22f1b | 36bfad8c3a7a85474c65d3fb3a17c0a105cb5e99 | 76d0f64a58729ef40fde22dd12101e419780738b | Direct original/full semantic parent delta |
| 838f25e0e4a00193d163f8715fc04a11aad628c9 | e1883489d21a8228c86cfee0bfb7dcdaca6b53df | 6c853aac44fd8510a4b67dfe234e4a72c8d79ccc | Pending direct early-section delta inspection |
| 8ee997712616cc5317633d974e53495bed714151 | 5da6aeb58c3da9609fbade0a48ca9aef7d9f58f7 | 9f0362d21da2f26143be14471a8ae6bb2eac787b | Pending direct early-section delta inspection |
| 6ec29171cbc645d403dee47a8c6348261cca19ba | 22cf4b985057f2424dead6240a9cdd33897746bf | 2166eda6f3206b7cac1cdf1f1ad34b7dd206fc73 | Direct original/full semantic parent delta |
| 739754c762cd65159d1f621886913e1a49827be4 | 6ec29171cbc645d403dee47a8c6348261cca19ba | 05cfca259c2bc967a7f67557d4458ee7cd88e7d0 | Pending direct early-section delta inspection |
| 7f7cc6aca8021169ecfdb53e878654bac40c9afd | 739754c762cd65159d1f621886913e1a49827be4 | a29dfdea5d92c0d2d09715a9a22a27cbfd4009be | Pending direct early-section delta inspection |
| a3fe79b1ee61117e8a044e05d68294546d5daf0d | 319d8aa9a645b7e67268bfeccceb22d22ef5fb81 | d6f60764bf593c4620e8393455bfd7d2caf1d4fb | Pending direct early-section delta inspection |
| 2c848f9e10d050c3099a826067aecca04d027157 | a3fe79b1ee61117e8a044e05d68294546d5daf0d | 6ed690e33a812f6ac7a75b52595bc378a4c1a80e | Pending direct early-section delta inspection |
| 2399c1adeef55c6b69a0faa1875bdce888ec69d2 | 0673591036fe53149b71a7a46f4e4e6e5ea417b4 | 9df29306287943025d52705399dc06127422c2d0 | Pending direct early-section delta inspection |
| 1b580d749ba761809bf6d52cf94139c67518e904 | 087b6b810f23d248fb2eb88cb1271ea2c0116099 | 5fd42c9d9b55721593fec0784506d3e990ba92ba | Pending direct early-section delta inspection |
| e6eaca4896755d23ec290deaf97508114bc06964 | 3fa382cb7c5769262702093cbdb001336ffb2c95 | d08be5afe63a1447ed0b975b2ee120989eb95dd6 | Pending direct early-section delta inspection |
| de817f2627b12739c658a08c0a2ce456e2da016a | f165ac447eb500b71d189070ea736101088540af | f730219c58a7abc35d4fa3e1e42fc0c4f96b0d9e | Pending direct early-section delta inspection |
| 3dc44c48625c1cd03e67e685d41111d55919bfde | de817f2627b12739c658a08c0a2ce456e2da016a | e7d72bc0a5e8c9f51a7fc50f8974105c452e9862 | Pending direct early-section delta inspection |
| 364689766c64985aced3b4bb1bdb79a5ee15be54 | d62785d31aa9272baba76509f2fce96e89b087cc | aa15f681432b0d8f629d0ec4a2469a2e28954c9f | Pending direct early-section delta inspection |
| b60c8280d1e117dfb2f095a59c1fb0c783850ebd | 364689766c64985aced3b4bb1bdb79a5ee15be54 | 60bd93be945e632f09ffd37d1aaa237d64acf060 | Partial: partial parent delta; rename/baseline/guard obligations read; missing middle pending |
| 444ae3d9dbd449115c06e3695c326b76c2b43c1e | 7433a2c0237190af207898b8dd5a9fca342033c9 | c0f44dc76a61fdc4a968e52c247e5c891808c60e | Pending direct early-section delta inspection |
| 4d2621102a75343cb000bda483c2235a76ce2627 | e6ecd1d80ad0b362abc707e5d3870a86de2246f3 | cfd1961a3fc9728b173b1b33e7a939e5851ec8b2 | Pending direct early-section delta inspection |
| 3adb6efec560ba14a1e49faaa44057ba0ee824ad | aed6f41dfe605074b3b32c9343e7f063dbf3d3e2 | dc03a38408ccd7d47eb0d3dab6113d6eb2074c52 | Pending direct early-section delta inspection |
| 14bca4a3246777bb5109d901f9259823749a34b3 | bfcf5e44204d8a7ba1d84b6ec63e7b3af76ad0e1 | 8ee6f561ebd026143130dfcec52b12a04ad00b03 | Pending direct early-section delta inspection |
| a1759b2a21adeac08222f2438243033fabd77512 | f28a81e3fc748330fdb73ed9fe91ab368af0a7ef | d9ab7d46c7490f3c028d844da3b9148d40fd43cc | Pending direct early-section delta inspection |
| ecc13378f4ff1cd5902ffd96bcddc580ba894f43 | 26913cc14b6304419c9626a019f1af2d0b8a1ce7 | 34602bce819eb2f88f9661b1ff3403fee8f9d3d3 | Pending direct early-section delta inspection |
| cae32f882697cf92f7bdbe18e8292e7d1ff47a60 | e9545703d9f2c6a0a5e790911b80120777adda35 | c0f55ebf1f345933f4e9d877d6f7b32af1d8d283 | Pending direct early-section delta inspection |
| 0c6c45df865de00dc1edf26a782792abafc2af1c | 23eae339424bfd8b719e0c7a9aa449b8cb155fdd | d15ad05d0b51c6d4fb2483db013db2bc9ed22986 | Pending direct early-section delta inspection |
| 3296f55ee8f2d1e1b243c501660171b0cab298bd | bdee6917ee218a5bedad4c45a48b9713601256be | d15ad05d0b51c6d4fb2483db013db2bc9ed22986 | Pending direct early-section delta inspection |
| a573eba86d938099416d302ca9788bf1ba9ed8e7 | b0c020825b76e8ef06c65059e1986a6afc56b0c0 | d15ad05d0b51c6d4fb2483db013db2bc9ed22986 | Pending direct early-section delta inspection |
| 08cabdf0d4c6b15e4637196ad0660b6887ba5aec | b0c020825b76e8ef06c65059e1986a6afc56b0c0 | d15ad05d0b51c6d4fb2483db013db2bc9ed22986 | Pending direct early-section delta inspection |
| 612ff7af6b80fc5c2da24a0bf8791136ec9d0a8a | 08cabdf0d4c6b15e4637196ad0660b6887ba5aec | aeee7aef75988ab05d311158080b53148821cd89 | Pending direct early-section delta inspection |
| 37f9031ef59a7b6fb7711a157b95355d0b63d525 | d252c1ea6d31838e34fe80df534378538f7a5235 | aeee7aef75988ab05d311158080b53148821cd89 | Pending direct early-section delta inspection |
| 246f7e40d4da181d6549db2f2d6e63bb4d5c5436 | 7784719628b39f346a62e6c6f9def2a5c0dba58c | 06254f3e91e3b9ad9079bf60cc702d9bfcb33454 | Pending direct early-section delta inspection |
| 2f20bea26da20307ee75038f1475f83364b460ae | 638135defc61b1462a766f5e32fa76859cda9612 | 84673d7952007475398287694982e79d7f6fa05b | Pending direct early-section delta inspection |
| df83c5ed5d0f5b131908105cca8cafc18d2ad364 | a40daa59106265f46695c9acca34d52699c16655 | aeee7aef75988ab05d311158080b53148821cd89 | Pending direct early-section delta inspection |
| 9b311666e983c739699d158bae5b64d96cab44ad | 246f7e40d4da181d6549db2f2d6e63bb4d5c5436 | 543ed3f3957276613686b478c06b5da0e2281f23 | Pending direct early-section delta inspection |
| 0defae2283b4a2599d3a469e247ebed31283412d | 668f46da9003f860a498217d76728a13995d0ae7 | 84673d7952007475398287694982e79d7f6fa05b | Pending direct early-section delta inspection |
| 130b85a41bdd36a40b33b4004ed8e8ba81c21a85 | 6a89884bd4a0eba8e884daebbed0877635da7cf9 | aeee7aef75988ab05d311158080b53148821cd89 | Pending direct early-section delta inspection |
| 067869c7bebc633225fc0dd1c1c8e2d1228cbc90 | f83eda976b1f05e9447795b86816171d0ade026f | aeee7aef75988ab05d311158080b53148821cd89 | Pending direct early-section delta inspection |
| ef15e28b9c5dd9938b83a9d79f69f61396392a19 | 067869c7bebc633225fc0dd1c1c8e2d1228cbc90 | 84673d7952007475398287694982e79d7f6fa05b | Pending direct early-section delta inspection |
| e45fde1f2d1ad304dd0fb8ebe7ea7baf26ae9e9a | 76f44830a4329fe3a98487cd339ef8a91b580225 | b6ddc1dd296e056792cb2c916adc8a91ee30cddf | Pending direct early-section delta inspection |
| 737ed7dec9920eca54c900fe389f738a03dfa4d7 | bdfda5dfacb9751f59f1e2781ce35cdc6ddbeee7 | 84673d7952007475398287694982e79d7f6fa05b | Pending direct early-section delta inspection |
| 557331b616ad51592097c04ee8fd18a92e79f99a | 737ed7dec9920eca54c900fe389f738a03dfa4d7 | de5a88af4eafc88b42685f67cf7f52fd600bd200 | Pending direct early-section delta inspection |
| 383af57c991b5431880b75be1f9ac198ce2249fe | 696bbefe82079862fe15c56b5612a72444b3ad1e | 64ca6531ed72c45b42d0f26eb1ec0281771bab26 | Pending direct early-section delta inspection |
| 8cfb810a9d6645d738e93c6877f658581bf2266f | 87bfbfdaa35dd3eb3d3ec49d3f66bda5883397ec | 64ca6531ed72c45b42d0f26eb1ec0281771bab26 | Pending direct early-section delta inspection |
| 1f01b83fb506a68cb595e1f9beb27e674eb52b40 | 139ad23e0288ffd715b4480c21f7a3c68fdba64d | 7196e1dd828853762c8f93b355c225f953154490 | Pending direct early-section delta inspection |
| 87dae98dc4aa4ed37f03be8bd2fff66fb87d70df | 34cf02ac96b8461fbbeb09d046d7e2b3e5ecc8c2 | 7196e1dd828853762c8f93b355c225f953154490 | Pending direct early-section delta inspection |
| ec3d760de9d178a91c153a12885f2652f0e7fa33 | 34cf02ac96b8461fbbeb09d046d7e2b3e5ecc8c2 | de5a88af4eafc88b42685f67cf7f52fd600bd200 | Pending direct early-section delta inspection |
| e524ceaa2344c0a334f3fd1130055eb1ee08890f | ec3d760de9d178a91c153a12885f2652f0e7fa33 | aa986bd47b9ead3b5b67b2db413429d1fcc87fc7 | Pending direct early-section delta inspection |
| e524ceaa2344c0a334f3fd1130055eb1ee08890f | e6ca522c0c21008e27d6f40d2c2ddcb7e778a927 | aa986bd47b9ead3b5b67b2db413429d1fcc87fc7 | Pending direct early-section delta inspection |
| 1192419969e285d36b918d6fe575909b5fe79022 | 5d17cc0e22913fe21c45354e121af414bd1a299b | aa8c57b62c7e750344ac445fd01967482dea7770 | Pending direct early-section delta inspection |
| aabe0b5ffe2fd036ecfd664db55d80e5d8c4940b | 5d17cc0e22913fe21c45354e121af414bd1a299b | abccf925cc581d848a0a2b43386c012a7e3ddb89 | Pending direct early-section delta inspection |
| aabe0b5ffe2fd036ecfd664db55d80e5d8c4940b | 06e190c48fbd9078322aa19222f608d48c3a3f17 | abccf925cc581d848a0a2b43386c012a7e3ddb89 | Pending direct early-section delta inspection |
| c493a2cdd814a4c5f53c5d660ebeaa28abdbdcd7 | d0083af1b7c4cfcab2d13c1127c89cfe08bb34b6 | aa8c57b62c7e750344ac445fd01967482dea7770 | Pending direct early-section delta inspection |
| 15ecde1e8b36a009ea13bce827d63ab7ecd94a34 | 26bcc9290842541ce6c66f0222d6ced1d4a3a55e | 5ba93b29f2ff2fffa26a0b70a823ecdd283a4442 | Pending direct early-section delta inspection |
| 0f018368e801440b159f8e5e7d77bb7cd2638876 | 776a2ce097c8c9cee5096d2dac075c8ffb0a47f2 | 2c33c67b4dc231d5841e35b9ed28f6f27dcecd60 | Pending direct early-section delta inspection |
| 82158df5ca4ff301e34fce0bf5bc018d767facd2 | 0f018368e801440b159f8e5e7d77bb7cd2638876 | bb5a839c84f05066998d40a627b87cb92e710ac6 | Pending direct early-section delta inspection |
| 33eeb99e7964fbb68fda38115f2d8b57ba7a66ed | a7e64c6b60979bea14cf74d3efc2c917e94e34c4 | ef5304a3c9fdbb678d0f6803257e359627cb542d | Pending direct early-section delta inspection |
| e49de3f8cf36e54e01efe4a55503da0642c4dea3 | b8552ffebdd27beb42076f60b6367a7a9a50c320 | 81a347142a6347ec5f6f832eaf3c7760197cb69e | Direct original/full semantic parent delta |
| 03697429564e20290a49278c3129ef5a123b4bc6 | e49de3f8cf36e54e01efe4a55503da0642c4dea3 | 181ebd1d0cb379a24111d0e2e61210de1c49612a | Pending direct early-section delta inspection |
| c936d51fb4ee7c374cf1f7a789645caabfec78c8 | 5dc310a530723fa2072734d12d9becd26d40d3cf | 6b33cf1d2d039aeaf5ac4aafe301b2eb3da20f85 | Direct original/full semantic parent delta |
| 91f0296a72b64268cab37dfc7dcf8b4c6844bbbe | b23df21c93a4694a4e71c4e76029bea14e275759 | b1a245c18cdbc80bbd4c67e556172b96661609c2 | Pending direct early-section delta inspection |
| 3f26ab1455e4d73ffff5de874e24a481a68658d5 | 7944fa17e2698d84fb95f39579dddef87fcc3ae0 | 8bdeebd36bace141da234ef9b1d4030a23b99022 | Direct original/full semantic parent delta |
| cd70747d3f40799026654ead9f5ccf93802fd862 | a5a8b952875ad833ee363311da066cd91c8cac06 | 4d8025e0090723f34a4dcb92230928f0e9444369 | Pending direct early-section delta inspection |
| 43a147f037ebba15e4acff0cfcf61e3641d10a23 | a473ed302aa26c3a1c99bb75f65ccf4ab779be53 | 7ae08eb589a81c780f37abe1f020698a77ab6185 | Pending direct early-section delta inspection |
| 15371443467d1f8a069067d0cae106bd8636bfec | f713ccb2a34eecbc9e43fa7121b5ee7f5aa1b86a | d330fa2952adc754008c452cb95c21ef5aeedfd7 | Pending direct early-section delta inspection |
| 2a0cf299d02fae2adff2020f068ac8d5a00757e8 | af5e63a249d85a08b95b00b4f29fd17973989a68 | cafb31a2229b86ea4e862217772b6799a77f43c3 | Pending direct early-section delta inspection |
| f01703a6a20e53423cfa136ebba5067f8a3da197 | 443c91a33146b3056950c1956cbe5d058ccfd13a | 359d4e9cbf574d15daa259226b2c41c1318265e1 | Pending direct early-section delta inspection |
| 8514210cf8592bc9ed1a9f9693847077d08c904b | 4c0939405acf905dcf279f480eb92c9eb7fc839d | 460792d0cb7dc407b5d11415949c474538d4dec0 | Pending direct early-section delta inspection |
| 2ebe74c54a4ee02efcc3f4adc81bc9f7450b0448 | 588423d2b7beb19806d7f24d4d33d3a5c810bda0 | ce5d17948fb1e54b6e05360dbdf8c4fff6b5240c | Pending direct early-section delta inspection |
| 2caff4552ae845bb7b81f19a8f87c15c16a925ce | 54391b4b799dc84fe6d94212f96efd5a5a31541c | 8e5ee3b33fc42978c9150c525c3675b42424c400 | Direct original/full semantic parent delta |
| 4afca16fe004764fa95c6c2aeea59fe85329d350 | 708a0722653b2185108a43e746f73f3887017f63 | 687c405fde12358dbc27dbdd24659de3765ef99b | Direct original/full semantic parent delta |
| 4a9dcab115db911e22fb114efa3527f96eff4509 | 4afca16fe004764fa95c6c2aeea59fe85329d350 | 103a31a11defa1c68467aa87273cb2fbe7cd91df | Direct original/full semantic parent delta |
| b1c8c4476136c14f6a1dbc0b9fb3340bad7318a6 | b44de49b5fe8d5242d2cc0b15d657da82fbaf199 | d825a172a4216e9009e20224310f6defb2f563a6 | Direct original/full semantic parent delta |
| 934c92dc2b5c466489088dc7011814f6eed648c0 | ee175d5d87fc18ed7fd4cc0aa78bf65d60537090 | 36343e8069f0ed4ac2bf118cfab8633d4c72ea40 | Direct original/full semantic parent delta |
| 2f139259ad4320b42b02aa70356e622338319380 | d3a6da03fa5276a553dc4694a7f0cf5ca890c03e | 03c4f9caccc0c4e42b970c2190d3c897c8f657f3 | Direct original/full semantic parent delta |
| d52ee38b35270396fe8cdb9fa37073f0bc794bc6 | 976cb45cd714c138017587de101009dcfaea8ef5 | bcd85a487b0b0273c07338295a3a76a48192179f | Direct original/full semantic parent delta |
| 29ed8e5e7f69b24dd8ca8351e91333d613d7ae99 | d52ee38b35270396fe8cdb9fa37073f0bc794bc6 | ebcca7205c673b53312d77f9904314dbb8f14bd2 | Direct original/full semantic parent delta |
| 577bf53311cf264917e4c2c57b2e97dfcce5bac2 | 2317e34d275a5c576222b9cac33f532c3fffbf03 | ca29d577423544a62c777b54de8c343b6dc4ce0c | Direct original/full semantic parent delta |
| aacb7cb3f54ac9d01a9ae9235b36ecce749528f1 | d2df556b2dfa23b6fa62935d1ce403f634dda2a4 | 11b6b3157784387e7862d64029e3da2c68588a05 | Direct original/full semantic parent delta |
| 309241051cce3af5802c2587e591c1dcd0e757ba | 7991eb0a709c25810a26001dc95f4512f33023fa | 853f2ccae06c5de064654ef6ca97760a51cc28bf | Direct original/full semantic parent delta |
| d4715f8e453fa8078a7a51dc73c63931bd53af03 | f98c375d6e5c891a97132dc8659722a324f32b1f | b2da2ea84bd94458ebf7ea3a1dee04532a092ed1 | Pending direct early-section delta inspection |
| da4b136493f1a94bf0f377530d1085d69c29b6ca | f9ba0c3a8166449dc3763d41ab4419b2251a87b5 | de3a8b215c05d4a2e4dbe6a3a6a7c5825f301881 | Direct original/full semantic parent delta |
| c62ec8d334d6ff89bc73ea1aee9f76d4a124f3aa | f394c6a794efc8ea93695d36e99f5dc919461342 | c57cb71fb1f41b54046004fe39618434d826f72e | Direct original/full semantic parent delta |
| e94c144364003c6b527a12109cb43015416bf6b5 | c62ec8d334d6ff89bc73ea1aee9f76d4a124f3aa | 0628139ff3fb7cb92bfdea4f972a6dc1f2fa4679 | Pending direct early-section delta inspection |
| c3b51b96ba1b7b096bb310252399e7b38856f54c | 0191acbfb034621c02142fc6e322bdf345cc0a24 | 3b7885e63c7a61abf94b52590529868571fd4fa8 | Pending direct early-section delta inspection |
| c3b51b96ba1b7b096bb310252399e7b38856f54c | af1e752e145e15ef7a91821c9adc6ef9d965551a | 3b7885e63c7a61abf94b52590529868571fd4fa8 | Pending direct early-section delta inspection |
| 7b27732a8c3c131760ec3438f641cb3c11343a42 | ef405e22db9eba06ec3a423321d4115b5956f42e | 3b7885e63c7a61abf94b52590529868571fd4fa8 | Pending direct early-section delta inspection |
