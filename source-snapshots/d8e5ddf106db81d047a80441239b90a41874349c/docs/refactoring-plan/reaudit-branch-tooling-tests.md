# Whole-branch tooling/tests reread — work in progress

Not a completed partition or whole-plan acceptance. Exact endpoints: holla `2e2401393c47360741ebd321679de08982dca50a` and main `7b27732a8c3c131760ec3438f641cb3c11343a42`; UI oracle remains separately `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Inventory has244paths and138274 changed text lines. Mode/blob membership comes from the coordinator's no-rename inventory. The table below records byte bindings, not semantic acceptance. Numeric prefix is physical line count; suffix is SHA-256. Binary physical-line counts have no semantic significance.

Current evidence: full executable source read for CI/perf workflow, test-inventory implementation/tests/profiles, package manifests and deleted historical capture-flow scripts; complete lockfile package/edge records parsed on both endpoints. Main lock120packages/223resolvededges versus holla182/352; every dependency reference resolves uniquely. All3211historical catalog IDs recompute from origin and canonical `[target,identity]`; counts2470initial-repaired-main/269Holla-reference/472publication-parent and exact catalog SHA35484514ad8241a4ce693b5c2498fea0102384502c2d44dfbec259ed86574e0e match pending required.json. All3211mappings remain unresolved, zero accepted targets, matching explicit pending authority. Original source-artifact reconciliation remains separate, not proved by catalog consistency.

Fresh pinned-main inventory tests:13passed normal and optimized, including real Cargo library/bin/integration/example/rustdoc, features, ignored status, dedicated logfile and package runtime cwd. Perf workflow3tests passed, including all4actual pipeline strings and removal-of-pipefail negative control. These preserve existing work; they do not qualify the new proof runner or complete TASK006/066.

BT01 historical nonacceptance: deleted `.github/workflows/holla-fable-audit-publish.yml:234` decodes malformed embedded base64 (31953data characters,1mod4). Direct stdlib decode raises before its declared payload SHA. Raw source blob93256f1db0d57685fbdac097d0a90724da711783/SHA47f685c3e24e55e51d12c99b883e5833539c9fe75f66edf20978385a108ed889 verified. Therefore its encoded patch cannot be credited as reproducible accepted publication evidence. Preserve archival bytes; do not restore/run this retired branch-specific publisher. No network request, patch application, credential read or publication was performed. Its surrounding code hashes source then uploads immutable blobs/trees, but uses assert-based checks and is not a current protected host/CAS authority.

BT02 surviving exporter geometry regressed: main `tools/ansi2html.py:147`/154 counts Unicode scalars, never clips; direct execution from the exact Git blob produced `界 ` for two columns, `é` without its required second-column pad, and `abc` for a two-column row. Main `tools/ansi2png.py:37`/86 uses scalar wcwidth and paints without per-grapheme clipping, unlike Holla's shared cells.layout, continuation cells and explicit raster fidelity report. Complete old/new exporter, shared cell module and eight-case fidelity checker source read. CW9/CH20/PAD12 dimensions alone do not prove faithful glyph occupancy. Old fidelity checker itself allowed missing Pillow to skip PNG checks, and old layout can place a later narrow glyph after a clipped wide glyph; neither historical implementation is blindly promoted to acceptance. Main capture.sh:868–895 still invokes these scalar exporters. Root condition: two independent approximate raster/text representations remain reachable from capture while F22 requires canonical geometry and distinct evidence lanes. Structural disposition needed: explicitly retire these paths from acceptance and label legacy diagnostic outputs, or route a retained exporter through the qualified canonical cell representation with width/combining/ZWJ/clipping/reverse/cursor and missing-font adversaries. Current F22 owner lanes001R003AC003CHK005,070R001AC001CHK004 and069R004AC004CHK001 already require renderer separation, but surviving-tool disposition is not yet explicitly accounted here. No exporter or production source was changed.

Full main capture.sh, capture_exec.sh and capture_provenance.py read. Preserve its useful exact argv staging, ancestor no-follow descriptor walks, nonblocking special-file rejection, per-run ownership and transactional five-artifact publication/rollback as implementation evidence; do not treat its self-recorded metadata as an immutable oracle receipt. Its default tmux server/global settings differ from Holla's dedicated socket, readiness means any nonblank text plus fixed waits, JSON accepts duplicate keys, and a mode change owned by the same UID does not establish hostile-worker isolation. These are legacy diagnostic boundaries, not substitutes for TASK001/070 protected host and pinned tui-snap execution. Tooling partition currently34paths read/semantically inspected; remaining rows are intentionally not credited.

Historical-render exact semantic binding completed: all499 unique recipe IDs/viewports and recipe-file SHA match the archive; all1497 input hashes match archive bytes and unchanged main bytes; all998 output hashes match installed main files; both renderer hashes match pinned source. Pins SHA e2dcdb2952e5cbd283b841ce055962c4d961dfa967d43c879bdbc49a5abdb6cb. This independently executes `/tmp/historical-pin-audit.1x2kEM/check.py`, using exact Git blobs, and credits source/archive relationships rather than model-reading repeated generated JSON fields. No image generation or font download occurred. Regenerator source enforces pinned fonts/Pillow/renderer/recipe set and absent output; its eight negative tests use assert for partial-output/sentinel checks, so their historical optimized-mode claims would need stronger guards. Preserve this archival generation boundary, never promote its scalar renderer or499 historical captures to immutable-oracle acceptance.

Qualified-capture source acquisition, fixture, harness, oracle, smoke, pins and integrity tests read. Fresh normal/optimized integrity tests both passed2tests (four missing/tampered repair mutations before any acquisition directory), validating all pinned payload hashes. The384 expected cells were reconstructed from the independent oracle's complete authored constants and matched the generated expected-cells JSON exactly, including every row-major coordinate/color/modifier/width/continuation. This does not rerun the live tool matrix or bless24old approvals. The old acquisition script pins upstream/final trees/locks and records build hashes, but leaves ambient RUSTC/Cargo/native configuration influence and does not replace the repaired actual-compiler host boundary. Preserve tool repairs/source history; current TASK001/070 uses its separately pinned, freshly qualified engine. Full patch/bundle and historical proof-payload content review remains uncredited until its rows below are read.

Current source/semantic-data review count51/244. Earlier34 count describes the preceding capture/exporter batch, not final coverage.

BT03 inherited digest writer's cross-process claim is false: `crates/tui-testing/src/digest.rs:1–12`/455–479 claims serialization across processes, but store() supplies only a process-local Mutex and the read/merge/rename transaction has no interprocess lock. The existing concurrent_bless_keeps_every_entry test runs its two subprocesses sequentially; only each process's threads overlap. Fresh unchanged pinned-main testing-crate unit tests passed17 with one intentionally ignored worker. Then `/tmp/digest-concurrency-audit.X1z3gP/probe.py` launched that exact compiled ignored worker concurrently for halves0/1, with its destination restricted to a new temporary diagnostic baseline. First attempt: both exit0, expected48 entries, actual46, lost2. No tracked baseline or source changed. Root condition is transaction ownership wider than the lock. Structural disposition must explicitly remove the unsupported guarantee and keep legacy diagnostic blessing outside acceptance, or serialize an authorized host-only publication transaction and prove two simultaneous processes. Existing blanket prohibition of candidate oracle blessing remains mandatory and is not relaxed by this finding.

All six tui-testing source files read completely. Preserve Runtime publication/settlement and explicit elapsed-time harness behavior, immutable Scene model binding, component-owned resolved style observation, capability-union requirements, exact root/control/activation/scroll/opener identities and modal trap checks. Existing Scene digest omits cursor and is FNV diagnostic evidence, never canonical parity. Existing conformance outside-area tests inspect symbols but not style-only writes; collection retention partly uses a detached CollectionCore rather than each actual component's state; empty binding tables return early. Therefore these inherited cases are not substitutes for the separately frozen component whole-cell/sentinel, actual stable-key lifecycle and031 source/slot/registry witnesses. Existing perf helper reports missing/malformed baseline as PERF-NOBASE and succeeds; hard host-required accounting and067 exact accepted workload/metric membership must reject that omission. No legacy debug-only or optional strict timing policy can override the fixed067 acceptance predicates.

These new source paths map to031 conformance closure and073 observation/timing substrate;008 owns historical test extraction only, not arbitrary helper repair;067 consumes the inherited perf substrate and its immutable required records.57/244 paths now have complete source or explicit generated-data semantic review. Component-b independently owns the still-unread changed crates/tui/tests and crates/tui/examples complement; no unreceived review is credited.

BT03 follow-up: a stronger parent-controlled two-pipe readiness barrier released both real Rust workers only after both were ready. On a fresh temporary destination both exited0 but only36/48 entries survived (12lost). The coordinator authorized a planning repair: TASK031 now binds R006/AC006/CHK005 plus D007 and W03110/11 to an actual interprocess locked fresh-disk transaction, same-key/cache preservation, exact48-key overlapping-process union and fail-closed lock/publication failures. Public API retained; std-only atomic sibling-directory lock keeps MSRV1.88 and dependency scope unchanged. Crash-left locks are not stolen automatically. No repaired helper implementation or repaired positive concurrency result is claimed yet; this is a mandatory future implementation/proof obligation, independently reviewed by the coordinator. The earlier wording-only retirement option is withdrawn: it cannot repair still-reachable unsafe public callers.

All eight deleted Holla goal/prompt/decision/concept/interaction/recipe/design/critique documents read in full. Preserve their historical semantic provenance: simulated provider-only world, virtual clock, scope/target/ranking identity, activities surviving navigation, two-gate target revalidation, full dependency-graph state and mono/geometry evidence. They are not new executable instructions or authority to restore removed APIs, provider execution, raw-key routing or visual redesign. The brainstorm contains intentionally rejected models and pre-fix sketches; e.g. shell-substitution Docker sketch, old glyphs,110-column preview amendment and gate-focus prose differ from later accepted decisions/source. The recipe note expressly documents obsolete Theme/RenderCtx, per-frame hit/focus repair, string-label menu dispatch and scalar/FNV/capture approximations. Current040–050 app ownership,003 immutable-oracle baseline,009 runtime,028 action-key routing and068 documentation must preserve actual pinned-source behavior through their fixed replacements, not mechanically resurrect those historical recipes. Critique B1–B4/S1–S16 plus response and D20–D24 remain source history, not unverified fresh visual acceptance; future picture equality is independently pinned to02f5294b.

Historical qualified-capture report/provenance/review programs now read: archival assertions, `+stable`, external sibling tool paths and self-reported clean/acquisition/test results are retained history, not the current fixed executable-identity boundary. Exact-provenance lists six commands, not all fresh gates. No archive program was executed to regenerate approved outputs. Full parity_approve.py/parity_replay.py read: retain fixed499-recipe grammar, argv/artifact/source relationship and review records as archive; their FNV/raw comparison, string reviewer identity, mutable working-tree allowance, per-row retries and three separate tmux snapshot queries cannot replace002–006 immutable oracle or070 protected capture. No recursive replay cleanup, tracked approval write or remote action was run.

BA-ART01 integration: the existing72 comparator vectors check cells/cursor/checkpoint syntax but do not establish one atomic producer generation for ANSI/text/cursor. Individually valid hashes can bind mixed frames. Required structural capture proof belongs070R001AC001CHK004 (regression070R003AC003CHK005), not comparatorCHK008 alone: one observed screen generation must supply cells/cursor and derived text with identical fixed action/tick/semantic checkpoint; actual alternating A/B producer and correctly hashed mixed-generation bundle must be rejected, coherent positives restored. Coordinator owns the shared repair; no new accepted result claimed here.

Current explicit own read count76/244, including one xtask Cargo.toml overlap. Disjoint final review ownership is114 retained paths here,112 tui tests/examples paths by component-b and18 xtask paths by historian. The final union must consume their explicit completed rows; no ongoing reviewer totals count as source acceptance.

| Path | Holla lines/SHA-256 | Main lines/SHA-256 | Read status |
|---|---|---|---|
| .DS_Store | 6/78f826e5a04536dff20ab9d2622f0f962172590372a8c76e73b3308b5ae0d010 | absent | not yet read |
| .gitattributes | 4/6cb0b7f0583ba696df0706ed13dbb91234e2f07cdc10a87fe96232e0d1221114 | absent | read; disposition/owner review ongoing |
| .github/workflows/ci.yml | absent | 187/892aebc346a1b267805f52843de84098317f559163477f591b6712abbb82dd54 | read; disposition/owner review ongoing |
| .github/workflows/holla-fable-audit-bootstrap.yml | 41/58bd58459a51ac9dc40aebc4614e06035923c5efba13d7e3f9b5c715894c188f | absent | read; disposition/owner review ongoing |
| .github/workflows/holla-fable-audit-publish.yml | 306/47f685c3e24e55e51d12c99b883e5833539c9fe75f66edf20978385a108ed889 | absent | read; disposition/owner review ongoing |
| .github/workflows/holla-fable-audit.yml | 97/5ed64c2713ba8e429d6bb9eae214e92a3749c67c81f3913f13c9ea787d5dc205 | absent | read; disposition/owner review ongoing |
| .github/workflows/perf.yml | absent | 109/53410a742dbc6b90a82074053d048cc2b551280366bad6834227bb360aa46163 | read; disposition/owner review ongoing |
| .gitignore | 4/87a6785529d0831c2c871bbd7c2e37b5cee8abc11169b9654afe68b6ad7f89d5 | 19/d9cdd403fe32b1f5f9e084b1093cce64efc60c9123324b73416234ff772be5d0 | read; disposition/owner review ongoing |
| Cargo.lock | 1673/6ae45e696498725ad5904bf52d83414787d0a2cdd7d1242bd93f14c0aa8ecb6e | 1094/8690159422308c08961890608c5746e5ed13da4ff9cb2fd038c2dc253c0100cd | read; disposition/owner review ongoing |
| Cargo.toml | 38/b74bafc1483375b964a724759e25cdf1d3b0b380aa949863ccc6fa8f5596d7fc | 78/f604c67fe0d693d8ea0a02b762c9970de518582cd640567918f1d2d49bc431b2 | read; disposition/owner review ongoing |
| crates/tui-testing/Cargo.toml | absent | 21/4baea2c3cdca49cb82326d27e276b2848c6af780444908e512e5d128ea15a451 | read; disposition/owner review ongoing |
| crates/tui-testing/src/conformance/driver.rs | absent | 1480/efbb58ec51972506be79645abf8c5b3d9124845639a77baedff03eecb08ce92b | read; disposition/owner review ongoing |
| crates/tui-testing/src/conformance/mod.rs | absent | 555/940e439ecf155e67952adf8dc757a824883da506adb0bceaf5ef1528bc59ac60 | read; disposition/owner review ongoing |
| crates/tui-testing/src/digest.rs | absent | 814/0df92bf5fba8eb7f2b8a59a0f94aa7a32cbcc12b9ed9dea90774dd24498718a5 | read; disposition/owner review ongoing |
| crates/tui-testing/src/harness.rs | absent | 623/c49299ff146976b899c1af1e7a51f38f6dc9819c0eb7128fcb0b7b0959fd2900 | read; disposition/owner review ongoing |
| crates/tui-testing/src/lib.rs | absent | 71/74aa895dea8c5d0aba76a6c4d7d932e53301ac2b4a852d3704293be0f4fecb4a | read; disposition/owner review ongoing |
| crates/tui-testing/src/perf.rs | absent | 568/08691cfa113f91319c3f1770efb92f9204cf8b3d30f7db0160b85943b30755a2 | read; disposition/owner review ongoing |
| crates/tui/Cargo.toml | absent | 64/d952d892b0bec0c4e5d4eee432194111ed40b82fae0a55235001441b75d18299 | read; disposition/owner review ongoing |
| crates/tui/README.md | absent | 57/29b6786ef6920f422067a141f4178bfab303378865250a404f50a54938a32f32 | read; disposition/owner review ongoing |
| crates/tui/examples/01_button.rs | absent | 38/c10ce0f0757d5e4e91b7460a71f3233ea80705b3137b4fb175d3e8dbbaf90fc7 | not yet read |
| crates/tui/examples/02_custom_theme.rs | absent | 240/ecc28b9eedd19bac10bb010a064bb525416697947ab2e37b028d5f7bc24b613b | not yet read |
| crates/tui/examples/03_partial_theme.rs | absent | 161/9e16210777fd18ce0741ae50fb14eb17df77565f726a98d099313e4460a1f47a | not yet read |
| crates/tui/examples/04_family_recipe.rs | absent | 209/6e3f1b07d3eb9aa80ac7ee7afe83c437c5c3b2da66a62e080a950e7f4f3438fd | not yet read |
| crates/tui/examples/05_instance_patch.rs | absent | 34/9987de169bbd8348d36894c1e21dafa387aeaf9f3119289508582da58804cd43 | not yet read |
| crates/tui/examples/06_validated_field.rs | absent | 68/9889001cb9ee370b81fb2a79d141e3450e3b27f8868ac14dc92cdb8de11391bb | not yet read |
| crates/tui/examples/07_borrowed_rows.rs | absent | 79/39fbc34a533169434771bd847644c431bccacb771ae41df1672c3efc14fe72dc | not yet read |
| crates/tui/examples/08_dynamic_tabs.rs | absent | 70/8250448318e5de1919bf483a83de87bd824e88e0edc57c19876efb8184cddbf1 | not yet read |
| crates/tui/examples/09_composed_dialog.rs | absent | 117/fc60c8f95e929603bfcff85bea3173a2ec0480d8ed771bcc41a7e4c553dc8d2f | not yet read |
| crates/tui/examples/10_nested_overlay.rs | absent | 126/0cdbc3bec9eb5fbb6f707f5f22184e550aef485abf60ea6eefadf93534001aeb | not yet read |
| crates/tui/examples/11_small_app.rs | absent | 134/a64c8f6adf13d1882d84dbbef71f9d4e006cc03bde1989115124ace6279c4db7 | not yet read |
| crates/tui/examples/12_author_component.rs | absent | 396/582a418292e458a45e3f222099e05ee85e211c0b2724b5eb72d74e6d2a4b7ad7 | not yet read |
| crates/tui/examples/13_connection_form.rs | absent | 298/8b0c6f3ea48127633c714a3d70cddb69c8e4cfa13f05032de456ae7d3e9468db | not yet read |
| crates/tui/examples/showcase_buttons.rs | absent | 216/44fe5bb3dda0e9899dfc067e848e9de0a741b960c3c0c5ad4a85746cc595518f | not yet read |
| crates/tui/tests/activation_origin.rs | absent | 203/415e66d3cf3b19bc909c6ef07c3098c3530172e8f811303dd98430be719134bf | not yet read |
| crates/tui/tests/allow/domain.txt | absent | 0/e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 | not yet read |
| crates/tui/tests/allow/legacy_api.txt | absent | 0/e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 | not yet read |
| crates/tui/tests/architecture.rs | absent | 354/a656eff88879fcf2f142b99ddd81aea779ab8298e803c8df6898ce1251fdafd4 | not yet read |
| crates/tui/tests/author_style_defaults.rs | absent | 490/d5525aa7357707feefd5121295154710f29b02e796e3df3ddd1d534872ab3807 | not yet read |
| crates/tui/tests/baselines/components.txt | absent | 2563/942219d9ad335ac892ad9658a85a57972fa6cf18afa33220af3b0705cdee474f | not yet read |
| crates/tui/tests/capability_palettes.rs | absent | 346/c7db3cb0de9d2e9bcc8224c5cffe4d76262042949b3a1701b2cad9fd84a2f38e | not yet read |
| crates/tui/tests/conformance.rs | absent | 4504/5c966c4068ebf0f8f3ab7aa4655fa1d1fb4ba736f3dd594c44fff49192789735 | not yet read |
| crates/tui/tests/derived_hintbar_metadata.rs | absent | 224/f3b4cf8c626ba26f72b71e56b53806dcfca9d99719a7439bb6d49f88be34ca21 | not yet read |
| crates/tui/tests/descriptive_hints.rs | absent | 240/3ec41874d45cef0d8c68d4e26f76734fec6c0aca33ca0bc54d3874a7f7092f11 | not yet read |
| crates/tui/tests/dialog_ack_keyboard.rs | absent | 132/066b94c470f14f1029a0668a2a0dc1ed5d54f22766612fc46e1c61ef0c1787f9 | not yet read |
| crates/tui/tests/dialog_acknowledgement.rs | absent | 129/c77b9d408ee358e028bcc6ace7248310d78cc8aa1035a64a5c71823f1991fd23 | not yet read |
| crates/tui/tests/dialog_navigation.rs | absent | 61/e3c1f9afe2733b47723d0b4b4f415b084c427fd66768e458a9953da91380a8f4 | not yet read |
| crates/tui/tests/empty_style_inheritance.rs | absent | 432/08e52ba2f3f3d80a2a3cf3c41c3bc9023aa83fc5964d8b26b70000095c218d37 | not yet read |
| crates/tui/tests/feedback_clock.rs | absent | 580/5eda3b542e17a9bdce806f650ab8814b783b67a5a50aa945fa33205702db34cc | not yet read |
| crates/tui/tests/fixtures/grid_model.rs | absent | 49/ee146cdaca093610eed65c51ce028a8a8910b6aaaf34951e4f21485193411621 | not yet read |
| crates/tui/tests/fixtures/text.rs | absent | 71/b219aa4cebe4a48073434d3fef383feb726bd15c8bde58f2c156809c978f9c7f | not yet read |
| crates/tui/tests/focus_restoration.rs | absent | 363/40bb3b6a05404ac82d037169fbe7a5b4e38784429a9bab00b39771320d04bd4d | not yet read |
| crates/tui/tests/focus_traversal.rs | absent | 212/760720dc6e5f83c318313497b9fcf5d8df3d2abc309f6d3d66cc17a1cfe2fbcb | not yet read |
| crates/tui/tests/fuzzy_boundary.rs | absent | 74/03067b283843d8a987fa92bae8007d2e5946451c6deaa70c1886f58e98c16247 | not yet read |
| crates/tui/tests/grid_blur_policy.rs | absent | 155/08880f861d185db1d755b63db1b601745e290b2c1e0402bbd976874d4c8b9219 | not yet read |
| crates/tui/tests/grid_cell_renderer.rs | absent | 304/eaf1e3ac63db0b773999f31eb7ff018d0d504f5ef3b7b12b29fa97a9760881a5 | not yet read |
| crates/tui/tests/grid_column_fit.rs | absent | 294/4d0cb723484d1c2fdeb95cf6ce7ffaeb378093a370492ecab94ecb7bcf07084a | not yet read |
| crates/tui/tests/grid_detailed_gutter.rs | absent | 446/456509e7011a1e835b33cf0c2408ce257a94b698cd89a2a778957b0d7741b5aa | not yet read |
| crates/tui/tests/grid_disabled.rs | absent | 167/bb9d150f3c3be9a2b530a85c12a7676014061a114cc63f9b0cac2cff656bea31 | not yet read |
| crates/tui/tests/grid_fetch_presentation.rs | absent | 177/a2afd7a2567e2d7607ff6cd77c5da6575d5c5e6c6d1dd309fb323b1b58575886 | not yet read |
| crates/tui/tests/grid_fetch_sentinel.rs | absent | 353/ad13e62ade1bfb904c9e6ef0ebc0f5f9bd54525396856b613e9ae93db3c21a0d | not yet read |
| crates/tui/tests/grid_header_geometry.rs | absent | 123/6169d8e5af177623a52a9528a2f22ca477914557b51665227b3a7f734944697c | not yet read |
| crates/tui/tests/grid_header_prefix.rs | absent | 231/74f8d3b2a01fc03fad61882c1071c11b628452ab688fe5efb7c37d6864a7b64d | not yet read |
| crates/tui/tests/grid_keyed_cursor.rs | absent | 241/dee983eaa7fd24158221b9c65912abf9c0637465ac2df0f0bee29e5f2b6e0ae2 | not yet read |
| crates/tui/tests/grid_model_contract.rs | absent | 28/5ca4e28eb4ff994d44bdbc4e83b06cd4f0fed00bfbbe9996eaabd00286555e33 | not yet read |
| crates/tui/tests/grid_sample_live_actions.rs | absent | 108/e052535dc01d3f0bb74f17ab6efc1102e8ba81b4a12d113484a4e21db2ea4a00 | not yet read |
| crates/tui/tests/grid_width_sample.rs | absent | 208/b48585494f71a8a405c0df84c6682e5492eec74ffa6809bfa0ddfa0090213494 | not yet read |
| crates/tui/tests/independent_empty_carrier.rs | absent | 46/449593f6474bac275a8af878b7e67a62ac8de563f1dda914b7b6aada9ceecbb9 | not yet read |
| crates/tui/tests/independent_paint_carrier.rs | absent | 94/2e4ad41f4f0c38c2f19abb96961454d4d618fdef9e79380c21ec0dea12a8c668 | not yet read |
| crates/tui/tests/initialization.rs | absent | 146/4660f4c32389447662d7b96ada90bbde81effaa7c325c7f4acd15ab1175019eb | not yet read |
| crates/tui/tests/input_placeholder.rs | absent | 92/f42c9bb4f5a5c9a401af4708850dd9f5e629f0699a52c488372c233124e2daa5 | not yet read |
| crates/tui/tests/item_row_columns.rs | absent | 473/1f939b63e2f2e0f9bfb80a9b043d99a967379efdcd81bde67e1a82f159415379 | not yet read |
| crates/tui/tests/keyboard_editor.rs | absent | 316/0a4e57558df6987c0f75ff79f74eeb8630c8bcb199d7632a65fcda97073d2b56 | not yet read |
| crates/tui/tests/layer_focus_reparent.rs | absent | 192/b5ab5728312b12f1be11edb7612c42bc3cf9e73506880b7580388813a85c2736 | not yet read |
| crates/tui/tests/list_boundary.rs | absent | 157/b831485d2abe656a7a1dfba4c6a4f596510c5639da44ab9b95830a13b01629aa | not yet read |
| crates/tui/tests/list_focused_click.rs | absent | 204/d72741c0228a14cae76df3fa1e621d7803891e1bab0774c5ccfbba477b226e30 | not yet read |
| crates/tui/tests/list_pointer_item.rs | absent | 235/220f7410d0ae29205954f3e811a3406fa08dfc7b769eacf81304febe2c8c4966 | not yet read |
| crates/tui/tests/list_row_extent.rs | absent | 253/5b946699384739fe5f202b9a62a902613baad5d168637f221a4f126e9994425b | not yet read |
| crates/tui/tests/list_row_renderer.rs | absent | 175/96ff1f123eec307efa225811ec29308b95a2ba9ebc0a075f3f2e6f7e1a90c02f | not yet read |
| crates/tui/tests/menu_chord_case.rs | absent | 68/d7f38a26a2aba3937120aa74c9ff8e25a6725075457c2ab61f0bd27e56ad086d | not yet read |
| crates/tui/tests/meter_defaults.rs | absent | 223/c098e338eb9e0ee8cb284365231a4c914a74a9005b72b90dce3e3f35f42b13cb | not yet read |
| crates/tui/tests/meter_readout.rs | absent | 226/d890523c2f5db17c88c1a3e5915ac23cd0314131a2ddbefd93a94d8edee9016c | not yet read |
| crates/tui/tests/meter_semantic_tokens.rs | absent | 399/f195df47394091998338843b2d09049fbbbc14dd8422ba4311e489128211e16f | not yet read |
| crates/tui/tests/monotonic.rs | absent | 410/66eec1e016e48937454141f3aba8d732d868cc01b4dc1c08ab7dd0dc4fece470 | not yet read |
| crates/tui/tests/nav_compact.rs | absent | 133/792dace224a1f17ad76bd961fb5f2cd67dcd058cca976e6b30d67af3319a6b7c | not yet read |
| crates/tui/tests/nav_list_boundary.rs | absent | 120/107ae70b0ad13575d18529b74dcf418375778569838e0c8d5a652f0e9936cca7 | not yet read |
| crates/tui/tests/nav_list_scrollbar_visibility.rs | absent | 204/ef6b01e26ddbd00876a909dbb87036a64998c561eb586edaa63ab4bc9b5b5b4a | not yet read |
| crates/tui/tests/nav_list_scrolling.rs | absent | 184/fca74e75cd841cd08a18dcb24c1f1c76fa8989fb27ba25aae8839ac516ae872a | not yet read |
| crates/tui/tests/overlay.rs | absent | 353/9116adb06e146ac75dcf23592bfef096ad974e9baf41293c93ba40975f266332 | not yet read |
| crates/tui/tests/overrides.rs | absent | 287/b8c1fe68b5e319f3dd8abd6815578b3bef15b3f46d893123f5dbcd272c3e21e5 | not yet read |
| crates/tui/tests/paint_matched.rs | absent | 168/39eb67f9f3b63097a323762f6024eac9e4d7a3a44061c684de1f7daf249ef6d5 | not yet read |
| crates/tui/tests/paint_middle.rs | absent | 136/9143ce066d31b3c212832a77bcece58c24a96c002e55a8e067b4d59316d882e8 | not yet read |
| crates/tui/tests/panel_badge.rs | absent | 146/46cdb6d228392e27535891982f7dc62f808c4caa6d06496858146e6f607d13bc | not yet read |
| crates/tui/tests/perf.rs | absent | 1714/3eecdfb5e5f9a0cb9d60a6acd9289c7a171689da1b559fdbb5332a5e738c0867 | not yet read |
| crates/tui/tests/perf_baseline.txt | absent | 113/e701660042110270f94ea21c78fb9114084afcd1cc57d60320f8e97dfc11d5c4 | not yet read |
| crates/tui/tests/perf_collections.rs | absent | 809/c2acd0143f93b9ac2a0a73a5d805ce2ee5966fe5572e8279c1a98f7a9ae41b30 | not yet read |
| crates/tui/tests/picker_nonsearchable.rs | absent | 256/177467a10e455a41ca8abd535c580e5620d55aa53bbde39d403120fa893a9b75 | not yet read |
| crates/tui/tests/picker_projection.rs | absent | 136/b56b601ea1d5bfda63aa7cb74e98aab622b9bd8b14b51473fab29249dec4acbc | not yet read |
| crates/tui/tests/picker_width.rs | absent | 90/95b2864a1fee8e1f74abd2c1354957de0c3c6d1896f8f52c76163d90ffd32cb2 | not yet read |
| crates/tui/tests/pointer_capture_eligibility.rs | absent | 350/a7846d822448deb263f5079306bd87d90a35c3b9e298cbb7effdf2cfb54fa35a | not yet read |
| crates/tui/tests/pointer_publication.rs | absent | 317/6398c51fc1c6c6730778b0be6bcc5447567c7734abab4504f4872630c4caa4c7 | not yet read |
| crates/tui/tests/progress_paused_glyph.rs | absent | 54/2d1f37e57cc74079f1a31b4821342dc8417600d4b5e255856d5934fdeb04c575 | not yet read |
| crates/tui/tests/props_rich.rs | absent | 451/ee3781fb338f672d664282e99952df88857d790f8489c26bef59730ba6347d50 | not yet read |
| crates/tui/tests/props_rich_perf.rs | absent | 52/029448ae8757343e0b569e90b5fd209dc450950a8cf8e8d1ac30f394e6f18ad5 | not yet read |
| crates/tui/tests/publication.rs | absent | 378/94dbf72c817d7c7ed7d380da700ff3582299b034a8106d32a378426f59454f24 | not yet read |
| crates/tui/tests/render.rs | absent | 542/374da1ef70f4995b378fe3e52b195ac17f52f8ccdaf1105dcef81263ace525c3 | not yet read |
| crates/tui/tests/render_components.rs | absent | 2081/a234e2659a633d38713f3cf0f22a53583d298a0f67e2b119c0f40a9eb1184b2d | not yet read |
| crates/tui/tests/rowui_glyph_contract.rs | absent | 131/2ab4d79a7242197578a2a7ab2e06d1f976884332df6389073581674d49da3c32 | not yet read |
| crates/tui/tests/scene_snapshot.rs | absent | 393/b944e35c98da653014b1e27e2b49069589b543b8e2c1f8487588e093e9fe1667 | not yet read |
| crates/tui/tests/scrollbar_full_track.rs | absent | 128/5f017c9ac01431231c3dabbc4e8704f852312ad5278bcf218551e8bc5fd16d4f | not yet read |
| crates/tui/tests/showcase_buttons.rs | absent | 643/c3d3d000e4f8aeae446399559cb323e50214af04ef792b724664c096c4893c37 | not yet read |
| crates/tui/tests/spinner_gap.rs | absent | 64/188b89dd3617a4bd00a624afecb7cae9e1040b6b8d39534ddf98095e0618a028 | not yet read |
| crates/tui/tests/split_seam_geometry.rs | absent | 512/b4ff447e3028eea8ca61e881e31a52a99c4abde2ac825cdedd29ae98bd780958 | not yet read |
| crates/tui/tests/status_bar_hover.rs | absent | 195/e3e7b208f8681c4080af8243d40dadb811611db8dcc7ed7042481cf23c1f4153 | not yet read |
| crates/tui/tests/status_spinner.rs | absent | 150/9205d70a75619cb58d4a6c903c1d7db2fd1b2e89d27d0ee8a3701f9238a823b8 | not yet read |
| crates/tui/tests/terminal_cleanup.rs | absent | 41/6d601f7000f44a1dc7ea7f605e8b1bb5f39e1232c881d7e559b4b12a176bbcde | not yet read |
| crates/tui/tests/theme_holla_states.rs | absent | 817/87ec0024131dd38c239ab6e12b938b46182cb0d60927bf77c3897b8a1294fc46 | not yet read |
| crates/tui/tests/tree_branch_policy.rs | absent | 126/800adaf7e87191873be6136f2d584fb659db02207ef5ed43b4f075efb0eafe22 | not yet read |
| crates/tui/tests/tree_presentation.rs | absent | 329/df635eb9c2f511719142065e82d33dd557e9683a6335d7037d857ecc3ca5eb08 | not yet read |
| crates/tui/tests/tree_row_renderer.rs | absent | 255/4cab25dabac971814870d32e2cae36591ed73b35429d44c7788b8b4ebda8f627 | not yet read |
| crates/tui/tests/tree_selected_click.rs | absent | 214/83f74b7e6b60ad52d6deeaeb136ade6151d278474037382219c892c99d3f6992 | not yet read |
| crates/tui/tests/tree_state_replacement.rs | absent | 143/dc4ac854ff7e371de687f7bc4a0d67698ef2da025027bce27dbf2423fc483fae | not yet read |
| crates/tui/tests/typing_owner.rs | absent | 751/666e5df508d23f224b406bcc73f8c063022ceee3fb817b431b41ecfb71e63a94 | not yet read |
| crates/tui/tests/ui/bitor_is_defined_only_for_unit.rs | absent | 11/1c70a7a18bf30b452d06c6f422f102631e5362521de006f88c077524aa712c0e | not yet read |
| crates/tui/tests/ui/bitor_is_defined_only_for_unit.stderr | absent | 13/436cc3cacf1e785368659f159dbdf9e0b0853acc61a127eac549c19338856450 | not yet read |
| crates/tui/tests/ui/grid_editable_update_rejects_shared_model.rs | absent | 50/78c48e66fb26c63cf21c3ca0ddd37234a482fd10b1461ad877648939eb4b7cff | not yet read |
| crates/tui/tests/ui/grid_editable_update_rejects_shared_model.stderr | absent | 15/c63729632cc7257e7d5f77e060cf67cc963796fb9ed146dc45a75f4ba9080718 | not yet read |
| crates/tui/tests/ui/is_not_clone_not_eq.rs | absent | 13/6149aa188300d6da3a1b2caeeee6ff58d50db092ef734a9b9e1aa93743b7d1c5 | not yet read |
| crates/tui/tests/ui/is_not_clone_not_eq.stderr | absent | 30/334f528369d29184a3465efa7f457c07eb5f5bcf4500b0d44805711e20332faa | not yet read |
| crates/tui/tests/ui/must_use_is_enforced.rs | absent | 14/1bce16799a00d40483eaeb0f5303da1122cc804572b4de8ba133b9b98d94153e | not yet read |
| crates/tui/tests/ui/must_use_is_enforced.stderr | absent | 16/5528b171bd172d7efab27d6a3daa625d68dc8c89c18e33fe34b48367cd9328c5 | not yet read |
| crates/tui/tests/viewport.rs | absent | 64/d8258e69ee5e525b6123cc6c4d03bac66a9bcaa827242c874975d445154259a5 | not yet read |
| examples/cells.rs | 50/c9e45ac54cca4d7ce00f61de177203a2d7948895a536d165c5f6f4667e25fcf4 | absent | read; disposition/owner review ongoing |
| holla-project/GOAL.md | 280/d2c8b4400476178eeb9e4cc79878ae36c682350ff7520f2c6aba21ff50eed77c | absent | read; disposition/owner review ongoing |
| holla-project/PROMPT.md | 18/c5e1d40a9490fa919d764768389fce3d6e2c9791643d6a45ce0b5e5f7b599261 | absent | read; disposition/owner review ongoing |
| holla-project/notes/00-decisions.md | 145/565504bf4e9ac030ecc489919b903348284727fa4693ed7b04e31acb292b7ddd | absent | read; disposition/owner review ongoing |
| holla-project/notes/01-concept-constraints.md | 266/c51f88ffc83259e23e922974c7ee0ff36b87c3ce311011a22bad2bc76b957a7f | absent | read; disposition/owner review ongoing |
| holla-project/notes/02-interaction-models.md | 657/ea04e73bd03788999b21fb3bbd4174e346d437f07d30cad904511156825055db | absent | read; disposition/owner review ongoing |
| holla-project/notes/03-recipe-audit.md | 480/cac6143afbf1bbcdf2faf2f934211c5f55abeb91298e7da54620a8b90a585825 | absent | read; disposition/owner review ongoing |
| holla-project/notes/04-design-note.md | 163/70d95cc96549a16efee5512a3eb7f52319d786d8a10d30a2ef92a3b8f781c489 | absent | read; disposition/owner review ongoing |
| holla-project/notes/05-visual-critique.md | 360/4ace18165e95d158b4953f34581c0d83b3ecdfc95556e00cbe5c79ec1fbadcd0 | absent | read; disposition/owner review ongoing |
| tests/baselines/jackin.txt | absent | 36/2a48d1928875b896355b512f2fb941c028e6578a4622586e693d16e488aec07c | not yet read |
| tests/baselines/tablepro.txt | absent | 42/c2bc99617bb7176b8a91bf4da2f2fb11640818de626f8fe418f8945e2f7f63aa | not yet read |
| tests/choice_containment.rs | 180/753f21830764db5a9378b14101a1c05da065a9d44ccd344c134d01ab1f608743 | absent | not yet read |
| tests/focus_gutter.rs | 185/e6301390fcc9fa99920b78567bc91842f513568142d36f9233f532ba4c5d430d | absent | not yet read |
| tests/holla_pty.rs | 404/9e77dd3125c36bf33e92dcb2a1aaa165311a327d5b29ccaba9519c4b48088327 | absent | not yet read |
| tests/perf.rs | absent | 849/82b23d8faf123fd3dd4673a79be3a2ee4a3236f8de5975e6621d844d771fc2ac | not yet read |
| tests/perf_baseline.txt | absent | 43/9e776b072330c143c2ce318183f3fb95986faf85413153a0be3a7c027f7dcd3b | not yet read |
| tests/perf_common.rs | absent | 375/43885c6f63e012712b457733b86ab714b5af3309588dd71c35b142764d65a477 | not yet read |
| tests/showcase_baseline.txt | 460/df3bf55c6451de45bd50e5174082288a7d98323b0429aed5742d2c86a820e61d | 44/2dfa4ac87ab0778313403c3ad57da22445f59f77893df48c3c49dfd806464d91 | not yet read |
| tests/terminal_suspend.rs | 472/f4873b4743c880b2219484caf65b2021259eba6e66e914802ca10c9f3548234e | absent | not yet read |
| tools/__pycache__/ansi2html.cpython-314.pyc | 132/8c9acf3ed1125ec2161de0def003416368ca41f9d497c476ef1222e5e86db6a2 | 132/0cd16be6f36b5b68788a158cb66210a38c7c59dc247305d3b81ce5abc9c2f02c | not yet read |
| tools/ansi2html.py | 48/c5b636b6f946d50db771462692c1cb0e62e03bf3b8587e048af1f34061468813 | 169/b8defedcf1703a410be28dc3761377f55bab258239cc5e6fa6b2cddca27a04b5 | read; disposition/owner review ongoing |
| tools/ansi2png.py | 227/96be2a3d80c3e309f1da80d227a752b6479d02dda8c0c35499c7a5c1a1c9f631 | 106/f29d92633e44337e2aba6fe0a960053014234ef5811ba18ee4527f336bb9eef2 | read; disposition/owner review ongoing |
| tools/app-inventory.json | absent | 143/974ab37fd0220e75a1a57f4df8f14b880dc5149242c77fd73e8135632dd25ed4 | read; disposition/owner review ongoing |
| tools/app-inventory.md | absent | 34/86b9b0044970454984dbb141122b5bad7152ba2084cfe4b3fa16e569495f1313 | read; disposition/owner review ongoing |
| tools/audit_flows.sh | 142/ad3aac8329ec8d6644043bb03da3015d1f18023a86fe3d6595dc295a52b2381d | absent | read; disposition/owner review ongoing |
| tools/audit_shots.sh | 68/814d3cfc96758208fc1daab80817602c140e4df115f5a814fb099172e1af3e5d | absent | read; disposition/owner review ongoing |
| tools/baseline_capture.sh | absent | 673/b52fcd00677716bf48205fa3868da57a0e45271f8b5c210a2b8d42ebcabcc19a | not yet read |
| tools/capture.sh | 166/0d4d10e62025fc8d905cd2ca7f2630b09d442f49ce61d64d264109111aa75502 | 1043/84d510b5ee084a21846ca5a47a0fad04a0d495ee7b5c8715490597232e2ba672 | read; disposition/owner review ongoing |
| tools/capture_exec.sh | absent | 47/4ac33d52557b01c6942e980b401099d1ef85389a9d3a109ece794bf5a7ffaa0d | read; disposition/owner review ongoing |
| tools/capture_provenance.py | absent | 1142/2601d6dbaccc9a346914e2b21d0114d66a5d5c7dcd660d0c118688a8e65b72f5 | read; disposition/owner review ongoing |
| tools/cells.py | 269/0ee86a3c2808aa143f4c2b56b5e335e1cfa50497e78c1dcd79934e286a56d835 | absent | read; disposition/owner review ongoing |
| tools/env.sh | 5/9b852825e19b318e347529eb4ee4c104fd95eae7a6e949b6d3739371d11b9757 | 3/ebadb31631ed9297b1d6d9104e1703a4f34f0b47ff39dcde06d3a2bcd035ef27 | read; disposition/owner review ongoing |
| tools/fidelity_check.py | 163/91302034a504e629382ee9efe5f0a91730934afa9a10f805b589656556a7edc8 | absent | read; disposition/owner review ongoing |
| tools/historical-render/README.md | absent | 54/4464aaae58d83bf064e94f8929a82251b83d2a8fbcc095aaec923187ef236c8e | read; disposition/owner review ongoing |
| tools/historical-render/pins.json | absent | 15513/e2dcdb2952e5cbd283b841ce055962c4d961dfa967d43c879bdbc49a5abdb6cb | read; disposition/owner review ongoing |
| tools/historical-render/regenerate.py | absent | 128/f1928d0f8e37d2f7ffea9d7432499f4b809adeb3bce1e9ebc8828757105567ab | read; disposition/owner review ongoing |
| tools/historical-render/requirements.txt | absent | 1/570c9387510a35edf1e287600993944ebc8ee5af602c809cf00cd57e2b723779 | read; disposition/owner review ongoing |
| tools/historical-render/test_failures.py | absent | 55/609f79ed826467a47351cea3593de8cd0065037a315d00b63d9d0b12628f3934 | read; disposition/owner review ongoing |
| tools/holla_flows.sh | 90/aed16d4b4cabb5ac6ca04ea003c34cc53116527d10e572044b1e596edb593b4f | absent | read; disposition/owner review ongoing |
| tools/holla_parity_flows.sh | 173/901f9d3352ad80dcbe7a6e85d4c5b30427425a11681053689b0451a26f43b408 | absent | read; disposition/owner review ongoing |
| tools/holla_shots.sh | 46/d396b090cff0d5ebbfa2964556c3d1b1a32b4142b7557cafc7e28d801f72f8c3 | absent | read; disposition/owner review ongoing |
| tools/parity_approve.py | absent | 503/a6a12644028208ed2fb52c9b61f59f7375c8e2af34a0f5f76ccb8627d2a4f16c | read; disposition/owner review ongoing |
| tools/parity_replay.py | absent | 607/995db0c27ee8ebf3ed42e04d4501b822ba3aada1c0b77670a16e61bb4483cfba | read; disposition/owner review ongoing |
| tools/qualified-capture/README.md | absent | 101/b6103126b99e0b7b88f6502dadf80556c842ea8078899e76c3da005b8bae17ce | read; disposition/owner review ongoing |
| tools/qualified-capture/acquire.py | absent | 154/267c4e7fa912e7564d01a21636286b76f0fc826afcfecc0450cb9774033736b8 | read; disposition/owner review ongoing |
| tools/qualified-capture/expected-cells.json | absent | 6530/c3a07e85392d78f4d661292d8fbf860a33371ecfe1b7af6408afec907fec9ea8 | read; disposition/owner review ongoing |
| tools/qualified-capture/fixture.py | absent | 40/7d8c4da061f79c78b77ef380045c476cc89ecb1c0539a44a5fed4d8cb62b2935 | read; disposition/owner review ongoing |
| tools/qualified-capture/harness/Cargo.lock | absent | 2287/a33dda53c5ea575b550f8998fa40f9646c35c33c456719b9ad900e333ddf0c58 | not yet read |
| tools/qualified-capture/harness/main.rs | absent | 17/a68c2340069549f5439998d6cb6e75606135d2b92a749dd2ca9a3f1f8040613a | read; disposition/owner review ongoing |
| tools/qualified-capture/licenses/tui-snap-LICENSE | absent | 200/b62eb1da60929bc24f7f8f1fe18c8f228fd39e4d096bf04d9102a038d1cb27ac | not yet read |
| tools/qualified-capture/licenses/tui-test-LICENSE | absent | 21/c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383 | not yet read |
| tools/qualified-capture/licenses/vt100-LICENSE | absent | 21/53a04b0a11073f3d0183c31a4b7bdf9a3f68d453adeec136e59e5a9d9a554370 | not yet read |
| tools/qualified-capture/oracle.py | absent | 101/ab41d25c5ac603ab656f92fcffba430c36ed815a72dd3e234cb167df31afb300 | read; disposition/owner review ongoing |
| tools/qualified-capture/pins.json | absent | 89/2e8245bb0cd3ae48c2b4a31f2203c7cc974d1de6ff15aa2bcae0456623cc1d60 | read; disposition/owner review ongoing |
| tools/qualified-capture/proof/REPORT.md | absent | 44/8aab52a0cc8abfd75879177d20291b66ec31e35387cb2c4cd3022e3b4591759b | read; disposition/owner review ongoing |
| tools/qualified-capture/proof/applied-review-verification.json | absent | 102/129668763dd11c059e12fb0afbfcd9a1f71fffe0497534f41fdc21eeaeceb509 | read; disposition/owner review ongoing |
| tools/qualified-capture/proof/clean-acquisition.json | absent | 119/135d63e8cd4d9b50b65f1b866d9b6aac787d1f6b017b86f3617e5fdd85cf5fd5 | read; disposition/owner review ongoing |
| tools/qualified-capture/proof/clean-metadata.json | absent | 1/3b05e3de7995c36695c2b395e3ba3c487e5498ffbb9209c436e02d8faead5d7c | not yet read |
| tools/qualified-capture/proof/cursor-independent-review/REVIEW.md | absent | 11/4192f7fbbcaf014f785615c3171c3edae8a35f768e2ee90bcc062ddf855e7b95 | read; disposition/owner review ongoing |
| tools/qualified-capture/proof/cursor-independent-review/SHA256SUMS | absent | 7/51e3022a83886cd8deac0e55f8a86c09d553504a7fd820ad73e9e2cdd6d773d2 | not yet read |
| tools/qualified-capture/proof/cursor-review.md | absent | 11/4192f7fbbcaf014f785615c3171c3edae8a35f768e2ee90bcc062ddf855e7b95 | not yet read |
| tools/qualified-capture/proof/cursor-review.sha256 | absent | 7/51e3022a83886cd8deac0e55f8a86c09d553504a7fd820ad73e9e2cdd6d773d2 | not yet read |
| tools/qualified-capture/proof/exact-provenance.json | absent | 122/0e8d9e2b73f916120b4dc1410e932acac30f266d5fa4a755e0ce5bccd7cd73e8 | read; disposition/owner review ongoing |
| tools/qualified-capture/proof/final-test-identities.json | absent | 198/d94e6c350fc5c4211bc090dc2d866594644b5241121b8804c2d057267d4baf13 | not yet read |
| tools/qualified-capture/proof/final-tool-gates.json | absent | 132/0ea1628659794a561fa14234dd489bb11fcf72bc36c78aac1531d40297dfee6c | not yet read |
| tools/qualified-capture/proof/full-0.log | absent | 107/5c27a0cf9b68e5180aa9936c7d700a8e647c8d199649b265dbad0dddd06cc8ab | not yet read |
| tools/qualified-capture/proof/full-1.log | absent | 93/ea682a11de6d7146d6a141c40dd62446bc43ca8fac99e3d05bf37f6265de5ddd | not yet read |
| tools/qualified-capture/proof/full-2.log | absent | 10/fa5b6e6312af1af4796301ac5847fd3ed76cfc7385f4dd0d51d86bed82a6168c | not yet read |
| tools/qualified-capture/proof/full-3.log | absent | 3/cc357669b0e6ab6d5e44af2553b0e97a595971cac14736bbc0b025ff2e00303a | not yet read |
| tools/qualified-capture/proof/full-4.log | absent | 0/e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 | not yet read |
| tools/qualified-capture/proof/full-5.log | absent | 6/20d02763237c0c63749011d0b82afb2c1d42c5e48b884b0074e733e2f183cae5 | not yet read |
| tools/qualified-capture/proof/independent-review/REVIEW.md | absent | 13/c3612b4e11a49227e1180637d8a80f9f2a5842beb997f2fb5bbbc62575702d4e | read; disposition/owner review ongoing |
| tools/qualified-capture/proof/independent-review/verified.json | absent | 153/01efdd9159aef7e2289a695f96c4447db067fa199783072a7b8b37830d2035d6 | not yet read |
| tools/qualified-capture/proof/matrix-status.json | absent | 25/702380245b12a3e9e670614fa23a0f06f8c56f67df290d7f28afb29a98e93b85 | not yet read |
| tools/qualified-capture/proof/migration-review.md | absent | 13/c3612b4e11a49227e1180637d8a80f9f2a5842beb997f2fb5bbbc62575702d4e | not yet read |
| tools/qualified-capture/proof/migration-reviewed-hashes.json | absent | 153/01efdd9159aef7e2289a695f96c4447db067fa199783072a7b8b37830d2035d6 | not yet read |
| tools/qualified-capture/proof/recorded-environment.json | absent | 41/cb8ff299bf57c805b927d576f6fd3dd7634968230481fd9820a1badb587704b3 | not yet read |
| tools/qualified-capture/proof/resolved-tools.json | absent | 41/cb8ff299bf57c805b927d576f6fd3dd7634968230481fd9820a1badb587704b3 | not yet read |
| tools/qualified-capture/proof/run_final_gates.py | absent | 19/816b5b6037a659333f1b798a04c870a6479b2baf0a09fef54d4fe876f8b7a68f | read; disposition/owner review ongoing |
| tools/qualified-capture/proof/run_repaired.py | absent | 30/9138faac95b5157794bf092cb3ae11ddaa89a2065752963a76fd8b08b01b6d4e | read; disposition/owner review ongoing |
| tools/qualified-capture/proof/source-artifacts.json | absent | 11/bfb8bb649215e715ccb48b0ecc8fc855850ff895ab8492b4bc7d4bcb5440e57f | not yet read |
| tools/qualified-capture/proof/test-identities.json | absent | 198/d94e6c350fc5c4211bc090dc2d866594644b5241121b8804c2d057267d4baf13 | not yet read |
| tools/qualified-capture/repairs/tuisnap-code.patch | absent | 5839/50844650914c58bf4b7d056a58adfed9d9078e02c69f9b84c310fa15eedf7da5 | not yet read |
| tools/qualified-capture/repairs/tuisnap.bundle | absent | 1958/58bc361167557f7835f329bbc9e8b73e44fc1c45e351869b04148e51c7fbb750 | not yet read |
| tools/qualified-capture/repairs/vt100-upstream.json | absent | 25/71f26a2ed3eda44859e0f311ff9de8897dcded18318b12f9d2cf167c036043f0 | read; disposition/owner review ongoing |
| tools/qualified-capture/tests/test_integrity.py | absent | 41/c360bb4471d11cc70923adac571b5ee9ed4f38d5e629b872412982224cbc0465 | read; disposition/owner review ongoing |
| tools/qualified-capture/tuitest_smoke.py | absent | 39/151815c99e7373e8a8cffb771a8cbc6e44a76dc17f1ac8a9b7ed3ff7ae163e5d | read; disposition/owner review ongoing |
| tools/test-inventory/.gitignore | absent | 1/32dae3052f331ee34d628ef535709b301259a45df7c7522c4d35dcf49873f00b | read; disposition/owner review ongoing |
| tools/test-inventory/README.md | absent | 126/675ad26c4e808b283cc8e818799b3348ad69bb0997d71182010df4d40a19f76c | read; disposition/owner review ongoing |
| tools/test-inventory/historical.json | absent | 19295/cd9c5c4908caf3150cd962e289f7310908c3a5b2285cd26cae624d520068af65 | read; disposition/owner review ongoing |
| tools/test-inventory/inventory.py | absent | 390/271d660ca69efdc6edddf92c75e39b4b95c8ffd5e838ba539d168f3682d242c0 | read; disposition/owner review ongoing |
| tools/test-inventory/profiles.json | absent | 60/5e8c81779b129888e27075ea456e408e0b0237a992791521f4e8b0283b7e378f | read; disposition/owner review ongoing |
| tools/test-inventory/required.json | absent | 16123/1ab6cd9cd95a0ace872085d763547707cb5c5948b727d8f90c2636fc4e8838b2 | read; disposition/owner review ongoing |
| tools/test-inventory/test_inventory.py | absent | 264/8756f66419a2ce958f7f322e439c9896e6a2973705a051c9fb231088b70121b5 | read; disposition/owner review ongoing |
| tools/test_perf_workflow.py | absent | 84/da138ca0bf1c6eb456387e25bb771d4b873ff244cdb98f73372c655d662edae8 | read; disposition/owner review ongoing |
| xtask/Cargo.toml | absent | 25/b947b70436b9234a1f09e53d051ceb51941066613cf17b009d9ed79b6bd0aad7 | read; disposition/owner review ongoing |
| xtask/doc_check_allow.txt | absent | 43/c66f8c9f61f176642d7504e42f732fbb4981ed9642c9916221617037e64ab471 | not yet read |
| xtask/fixtures/backend-free-core/Cargo.lock | absent | 396/02528f9f2b8831180f0720eaf6f563a77b25ee47d7b19c8a583a5070c2c5df40 | not yet read |
| xtask/fixtures/backend-free-core/Cargo.toml | absent | 11/2113ccd29c241dd8f484d84d0c5cad6cfa1a59031e6d8b1656fb5fafd07d520d | not yet read |
| xtask/fixtures/backend-free-core/src/main.rs | absent | 30/bfd7f5a303b9e3a63229e1453ad38dd4b858728cf68920e7e0ada1861abe9087 | not yet read |
| xtask/fixtures/backend-free-testing/Cargo.lock | absent | 406/2e4042dd4796cd46ff7112d3625acea4b5252eef3e129308f7ba3872f1adc222 | not yet read |
| xtask/fixtures/backend-free-testing/Cargo.toml | absent | 12/ce0a5bc9c18d9c75fed02049869725ef9b2afa4241494e9e8679d0eac756716e | not yet read |
| xtask/fixtures/backend-free-testing/src/main.rs | absent | 31/1dd5a512fe224da50965e560f5ded9bc8d5fb4026d3ba72aa3064ca257d66baa | not yet read |
| xtask/named_tests_allow.txt | absent | 22/375f27376e34d9255871b09a34389e999ecb1ee4a5974fa1a168ec2ebd80e493 | not yet read |
| xtask/src/app_inventory.rs | absent | 424/7299a0e24ff4cdeb8e8e43b3d70684421da871b961225bf895bfee13343e8fd8 | not yet read |
| xtask/src/backend_free.rs | absent | 164/45110cd218f86688cb89ff20d5cca6b1795354ddfdfdb5526d0d0ea84c9fa025 | not yet read |
| xtask/src/capture_build.rs | absent | 758/939b10389d564055314c767ee3d5417a234b25e05751b4e3a661738dd724e793 | not yet read |
| xtask/src/capture_confinement.rs | absent | 372/eeee04840bf786b3c15995c03f0bdbcf0e769f75937182b315abf485bd8ccde4 | not yet read |
| xtask/src/capture_contract.rs | absent | 608/cc84815abbae5eea199c32b2081c3361c7a203e73559e492a01c39aeb2047834 | not yet read |
| xtask/src/capture_wiring.rs | absent | 263/9a3bb8979cf7e623abc2500bca544e34250eef8ce498744bb41145776a4a2c67 | not yet read |
| xtask/src/historical_additions.rs | absent | 254/4c5f7db4f65d1f6881d9b94c245c6647f0dfa79396915e2f15e71a8dc8cf8183 | not yet read |
| xtask/src/main.rs | absent | 11816/28e36c1c752e456c074ebab5e8cc6fb7bb17a392d8416da46aa72b66f0df5982 | not yet read |
| xtask/src/parity.rs | absent | 2072/60b2dc9a6063bb49f90efefc7548ae34ff9fc772913d3a42fd231f0a1aaedc5a | not yet read |
