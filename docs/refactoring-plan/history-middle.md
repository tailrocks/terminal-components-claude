# Direct history review: architecture §§30–55

Planning-only evidence, 2026-09-11. Architecture source is `7b27732a8c3c131760ec3438f641cb3c11343a42`; UI/UX oracle is `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Historical green counts below are records, not current test execution.

## Direct-read coverage and method

Directly read `ecc13378:COMPONENT_ARCHITECTURE.md` lines 6706–7995 (§§30–55, including both §47 records and the §54 addendum). Then inspected all parent-relative architecture edges in `history-revision-index.tsv`, selecting every hunk intersecting §§30–55. Read all superseded/deleted historical lines plus every later unique hunk. Thus original inserted text still present in the cumulative source was read there; overwritten original text was read from its actual deletion delta. This avoids treating a later summary as a substitute for the source. Exact identical-delta skips and commit/blob identities appear below. Changes outside these sections belong to early/late historians; an edge listed here is not whole-document certification.

Directly read `ecc13378:REFACTORING_STATE.md` lines 346–1669. All fifteen state deltas from `70dacec1^..ecc13378` are additive only: the removed-line sweep returned no deleted content. The cumulative reading therefore includes the actual content of each earlier version, including contradictory measurements and later supersessions. Later state edges belong to the late historian.

Directly read all of `main:docs/reviews/findings-from-documentation.md`, `laneB-grid-contract.md`, `laneC-app-tick.md`, and `docs/audit/legacy-test-disposition.md`. Read the latter's historical removed lines at `568e1ef6` and `9df371d5`: explicit env reveal became always-masked edit/save; account masks may retain the final four characters, and the full secret must not render. The three review documents have one substantive creation each (`de47f879`, `3adb6efe`, `3adb6efe`). The legacy disposition was created at `3fa382cb`; its two later amendments are preserved explicitly. Reading this report does not certify its inferred application assertions or name-only perf equivalence: those require actual source/workload comparison.

## Decision reconstruction

`70dacec1` (§§31–32) corrected the architectural type that concealed neutral-recipe omission and distinguished the resolvable set from stored families. The test had enumerated precisely the same incomplete set as the bug. `9486b654` (§§33–34) separated component-owned styled parts from arbitrary caller row parts, required observable conditional fixture coverage, and put capability detection at the terminal boundary. `afc60678` rejected `Ui::scroll_region` and unspecific suppressions. These are structural requirements; later implementation-status prose does not revoke them.

`739754c7` (§38) replaced readiness prose with capabilities that imply paint obligations. Spinner itself can legitimately narrow readiness; an accepting status component cannot excuse missing paint with a reason string. `7f7cc6ac` (§39) separated runtime state from props-derived state; local forced-state APIs were subsequently superseded by §72's scoped inert reference mechanism. The ownership rule survives. `de817f26` (§45) separately required override promises to equal actual painted substitution behavior: query existence and heading presence are insufficient.

§§36–49 document why green tests and first-generation hashes did not establish visual correctness. The first bless occurred before §39, then §49 corrected **22** measured keys, not the predicted 24. `bfcf5e4` is the state ledger's implementation closeout, reporting 12 truecolor and 10 mono movements. Keys already generated cannot become first-generation again through changing a comparison base or reverting a bless. Truecolor changes require their specific authorized scope; declared blockers and a real comparison base must fail closed. Current PLANNING_GOAL imposes a stronger immutable-oracle rule: historical movement classifications never authorize changing the new reference.

§§46–48 rejected moving unmigrated sources into `apps/` and deferred the crate rename until the last consumer moved. Preserve the completed result and its single-library/bin ownership guards. Do not reconstruct the temporary staging plan. Later bulk name replacement produced the nonsensical historical phrase `junie-tui → junie-tui`; this is stale documentation, not a new decision. `08cabdf0` hardened due-app presence, and `612ff7af`/`37f9031e` required each Showcase page's own production update/draw roots rather than one shared implementation satisfying all pages.

`3adb6efe` (§§50–55) resolved ChipBar `CHECKED`, payloadless add requests, caller-controlled RadioGroup value, StatusBar keyed hover, adapter-owned Grid sorting and explicit focus without geometry, cached incremental Tree projection, Jackin runtime/product-time ownership, and total container closures. Tree query projection includes only matches and ancestors, ignores but preserves saved expansion, and cannot collapse away forced ancestors. Node-access counters—not allocation counts alone—prove warm scaling.

The state mirror at `ecc13378:REFACTORING_STATE.md:832` resolves the scalar FieldControl limitation more precisely than §30's remaining “unresolved” shorthand: item-bearing choice controls use direct per-phase paths and Form drives them directly; future item-aware trait widening remains open. Later §67's configured bridges preserve this direction. Do not add a trait-widening task solely from the stale shorthand.

`ecc13378`'s §54 addendum strengthens the proposal in `laneC-app-tick.md`: exactly four update causes, not “at least” four; status formatting uses fixed app-owned buffers, not the proposal's permitted temporary allocations. It explicitly retains inactive Manager/Accounts message delivery through inherent methods. The review's proposed total RunId package also covers Manager's byte slice, not only Capsule/Cockpit. Reconcile its full-ID-derived token with immutable-oracle visible formatting instead of silently changing text.

The same checkpoint reverses the preceding visual PASS to FAIL while retaining both records. It explicitly says no Slice-4 closure and identifies missing itemized visual/API/§73 findings. A later merged commit cannot retroactively convert the old PASS into sufficient evidence.

## Current source findings

Direct source inspection at pinned main confirms concrete remaining gate defects:

- `crates/tui/tests/conformance.rs:4396` still accepts `extra`, checks only observed parts ⊆ declared parts ∪ extras, and `:4461` manually lists cases. Probe, Dialog, Props and PropsList are registered but omitted from this parts check. This violates §§33/41's ownership-aware exact equality and registry-derived completeness.
- `crates/tui/src/ui/mod.rs:96` defines `StyledQuery` as `(Id, Family, Variant, Part, Resolved)`. `note_styled` at `:560` is opt-in; there is no component/row provenance. `0f018368` correctly labels the attribution contract a target and its implementation absent. This correction admits missing work; it does not reject the contract.
- `xtask/src/main.rs:7318` still scans only §§3–17 and §§21 onward. The migration map, rejected alternatives, and §20 requirements remain outside doc-check. Later parser-tail repairs did not close that omission.
- `15371443` narrowly makes `Conformance::patch_part()` a future ordering-hardening option after `ebfa8b8`'s recorded-style/sibling proof. Requiring the hook itself is unsupported; validating actual advertised visible overrides remains required.
- `theme/glyph.rs:349` asserts absence of U+2500–U+257F, not that the complete glyph set is single-byte ASCII of width one. `theme/builder.rs:237` changes only quiet/active rule and scrollbar sets. §35's broader table and assertion remain distinct obligations.
- Seed setters in `theme/builder.rs:105–135` retain the limited explicitly documented tint/on-color cascade. §43's wider syntax/meter cascade is an unresolved proposal; do not implement it silently during parity work.
- Named §54 application tests such as `status_projection_allocates_zero_per_frame` and `cross_route_msg_fanout_reaches_an_inactive_screen` were not found under `apps/jackin-preview`. This is a traceability/proof gap, not proof the behavior fails; map equivalent current tests or provide explicit verification work.

The ledger is granular in `history-middle-obligations.tsv`. The historian's completed canonical join now assigns every HM row to actual task IDs and exact requirement/acceptance/check edges in `traceability-history.tsv`; the former UNASSIGNED staging state is retired. Full clauses and proofs are installed in the corresponding protected task source assets. Current source locations establish presence/absence only where stated. Historical implementation claims have not been rerun here.

## Source coverage index

The tables below qualify the corresponding global history-index rows for this section scope. `cumulative+removed` means full retained section source plus all overwritten text directly read; `direct delta` means the later amendment itself was read; `identical delta` identifies an exact comparison already read.


| Commit / parent | After blob | Read disposition |
| --- | --- | --- |
| `70dacec1 / ad94d12a` | `2d161ea3bf2c3e75f248f04824664607641fd7ce` | cumulative+removed |
| `9486b654 / 95f486f0` | `19f6ff13c6ab278cb2cfd9248bddb85e369dfb84` | cumulative+removed |
| `afc60678 / 36bfad8c` | `76d0f64a58729ef40fde22dd12101e419780738b` | cumulative+removed |
| `838f25e0 / e1883489` | `6c853aac44fd8510a4b67dfe234e4a72c8d79ccc` | cumulative+removed |
| `8ee99771 / 5da6aeb5` | `9f0362d21da2f26143be14471a8ae6bb2eac787b` | cumulative+removed |
| `739754c7 / 6ec29171` | `05cfca259c2bc967a7f67557d4458ee7cd88e7d0` | cumulative+removed |
| `7f7cc6ac / 739754c7` | `a29dfdea5d92c0d2d09715a9a22a27cbfd4009be` | cumulative+removed |
| `a3fe79b1 / 319d8aa9` | `d6f60764bf593c4620e8393455bfd7d2caf1d4fb` | cumulative+removed |
| `2c848f9e / a3fe79b1` | `6ed690e33a812f6ac7a75b52595bc378a4c1a80e` | cumulative+removed |
| `2399c1ad / 06735910` | `9df29306287943025d52705399dc06127422c2d0` | cumulative+removed |
| `1b580d74 / 087b6b81` | `5fd42c9d9b55721593fec0784506d3e990ba92ba` | cumulative+removed |
| `e6eaca48 / 3fa382cb` | `d08be5afe63a1447ed0b975b2ee120989eb95dd6` | cumulative+removed |
| `de817f26 / f165ac44` | `f730219c58a7abc35d4fa3e1e42fc0c4f96b0d9e` | cumulative+removed |
| `3dc44c48 / de817f26` | `e7d72bc0a5e8c9f51a7fc50f8974105c452e9862` | cumulative+removed |
| `36468976 / d62785d3` | `aa15f681432b0d8f629d0ec4a2469a2e28954c9f` | cumulative+removed |
| `b60c8280 / 36468976` | `60bd93be945e632f09ffd37d1aaa237d64acf060` | cumulative+removed |
| `444ae3d9 / 7433a2c0` | `c0f44dc76a61fdc4a968e52c247e5c891808c60e` | cumulative+removed |
| `4d262110 / e6ecd1d8` | `cfd1961a3fc9728b173b1b33e7a939e5851ec8b2` | cumulative+removed |
| `3adb6efe / aed6f41d` | `dc03a38408ccd7d47eb0d3dab6113d6eb2074c52` | cumulative+removed |
| `a1759b2a / f28a81e3` | `d9ab7d46c7490f3c028d844da3b9148d40fd43cc` | cumulative+removed |
| `ecc13378 / 26913cc1` | `34602bce819eb2f88f9661b1ff3403fee8f9d3d3` | cumulative+removed |
| `0c6c45df / 23eae339` | `d15ad05d0b51c6d4fb2483db013db2bc9ed22986` | direct delta |
| `3296f55e / bdee6917` | `d15ad05d0b51c6d4fb2483db013db2bc9ed22986` | identical delta 0c6c45df |
| `a573eba8 / b0c02082` | `d15ad05d0b51c6d4fb2483db013db2bc9ed22986` | identical delta 0c6c45df |
| `08cabdf0 / b0c02082` | `d15ad05d0b51c6d4fb2483db013db2bc9ed22986` | identical delta 0c6c45df |
| `612ff7af / 08cabdf0` | `aeee7aef75988ab05d311158080b53148821cd89` | direct delta |
| `37f9031e / d252c1ea` | `aeee7aef75988ab05d311158080b53148821cd89` | identical delta 612ff7af |
| `246f7e40 / 77847196` | `06254f3e91e3b9ad9079bf60cc702d9bfcb33454` | direct delta |
| `df83c5ed / a40daa59` | `aeee7aef75988ab05d311158080b53148821cd89` | direct delta |
| `8cfb810a / 87bfbfda` | `64ca6531ed72c45b42d0f26eb1ec0281771bab26` | identical delta 246f7e40 |
| `1f01b83f / 139ad23e` | `7196e1dd828853762c8f93b355c225f953154490` | direct delta |
| `87dae98d / 34cf02ac` | `7196e1dd828853762c8f93b355c225f953154490` | direct delta |
| `e524ceaa / ec3d760d` | `aa986bd47b9ead3b5b67b2db413429d1fcc87fc7` | identical delta 87dae98d |
| `aabe0b5f / 5d17cc0e` | `abccf925cc581d848a0a2b43386c012a7e3ddb89` | direct delta |
| `aabe0b5f / 06e190c4` | `abccf925cc581d848a0a2b43386c012a7e3ddb89` | identical delta aabe0b5f |
| `15ecde1e / 26bcc929` | `5ba93b29f2ff2fffa26a0b70a823ecdd283a4442` | direct delta |
| `0f018368 / 776a2ce0` | `2c33c67b4dc231d5841e35b9ed28f6f27dcecd60` | direct delta |
| `15371443 / f713ccb2` | `d330fa2952adc754008c452cb95c21ef5aeedfd7` | direct delta |
| `d4715f8e / f98c375d` | `b2da2ea84bd94458ebf7ea3a1dee04532a092ed1` | direct delta |
| `c3b51b96 / 0191acbf` | `3b7885e63c7a61abf94b52590529868571fd4fa8` | identical delta d4715f8e |


### State and review source versions

| Commit | Document | Blob | Coverage |
| --- | --- | --- | --- |
| `70dacec1` | `REFACTORING_STATE.md` | `8d1a9bfc6f749f9c9668d63af275824b01fbb1a3` | additive-only cumulative source |
| `3de3e8e2` | `REFACTORING_STATE.md` | `416b87985adc246586696a99933731b333ad4799` | additive-only cumulative source |
| `c6176164` | `REFACTORING_STATE.md` | `508a30788dab7447a7436157248d9c78d4bdf5b9` | additive-only cumulative source |
| `afc60678` | `REFACTORING_STATE.md` | `a7853cdcadce7d757a3f073913116456c35afe69` | additive-only cumulative source |
| `0223f276` | `REFACTORING_STATE.md` | `c964f12c98ee311dd3e7a802247a5f5ffdfa829a` | additive-only cumulative source |
| `7f7cc6ac` | `REFACTORING_STATE.md` | `7737f48263488422c06412f67996f98b44849d29` | additive-only cumulative source |
| `7e49bbd1` | `REFACTORING_STATE.md` | `66272b2c3bbd5c73be50c43c0ad5fdf03fced89b` | additive-only cumulative source |
| `463efcad` | `REFACTORING_STATE.md` | `07d920d8ebcf9e04d4e6ad7418e45f240e48111b` | additive-only cumulative source |
| `f165ac44` | `REFACTORING_STATE.md` | `3f6bf9eacd092ce967e5b61fef4561edcfbe9808` | additive-only cumulative source |
| `43d8db05` | `REFACTORING_STATE.md` | `1eb3536a3571562bd15c090dc395c47499b617b2` | additive-only cumulative source |
| `3adb6efe` | `REFACTORING_STATE.md` | `20f064978d7be79f88156f4312216271f12c061c` | additive-only cumulative source |
| `14bca4a3` | `REFACTORING_STATE.md` | `5de5987de1c5b86216576941bcf379a1b79f2569` | additive-only cumulative source |
| `a1759b2a` | `REFACTORING_STATE.md` | `07870186b0a45f68d2d173f55cb0ccca43413a31` | additive-only cumulative source |
| `5715d2df` | `REFACTORING_STATE.md` | `1cf0e0cb3b86e91bdaee1b51dac89db465d0bee0` | additive-only cumulative source |
| `ecc13378` | `REFACTORING_STATE.md` | `8c78d0b50313f05889283d7fa916dc548738374c` | additive-only cumulative source |
| `de47f879` | `docs/reviews/findings-from-documentation.md` | `e05247bc1860f03c5bfca707cc60c45b8cb0f866` | direct source + historical removed lines |
| `3adb6efe` | `docs/reviews/laneB-grid-contract.md` | `3c0770a44f14cf26a803bdee79cdc653da335a7d` | direct source + historical removed lines |
| `3adb6efe` | `docs/reviews/laneC-app-tick.md` | `582a84d547cc56fc5837aff284b6d79ccfb6df5a` | direct source + historical removed lines |
| `9df371d5` | `docs/audit/legacy-test-disposition.md` | `e61a91daf98d4fecbab63eab7900ec40c1283ddd` | direct source + historical removed lines |
| `568e1ef6` | `docs/audit/legacy-test-disposition.md` | `f5ed1a4ef6ca64d984e564fabf576afef814b444` | direct source + historical removed lines |
| `3fa382cb` | `docs/audit/legacy-test-disposition.md` | `acfefe422366c4644b237fe61ef69e7e883983e2` | direct source + historical removed lines |
