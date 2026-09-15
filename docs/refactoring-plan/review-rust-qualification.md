# Independent Rust architecture qualification review

Reviewer: `/root/review_rust_qualification`. Review date: 2026-09-11.

## Current verdict

Accepted for planning preparation and TASK072 qualification dispatch: the standalone Rust corpus, actual terminal-components corpus, protected observer adapter, sibling asset layout and AC-009/CHK-011 wiring pass independent review and execution. No material preparation defect remains open in this review. This result does not certify the production `tc-proof architecture` parser or terminal-components architecture; that implementation and its independent acceptance remain future TASK072 work.

No terminal-components production implementation was changed during this review. Tests compile disposable source copies. TASK072 and TASK073 remain future implementation work.

Independent Git checks resolve the immutable UI oracle to `02f5294bfdbf38004cc49130d0aff1d01f31434c` and the actual-source witness to `7b27732a8c3c131760ec3438f641cb3c11343a42`. `git archive` of the actual-source witness has SHA-256 `732e40d9cf82e98e3d6375aaeecee349e8cb774abc5094dbbff145555201633a`, matching the actual-source driver's declared archive pin.

## Independently executed standalone evidence

Command: `rtk proxy python3 -B docs/refactoring-plan/evidence/architecture-bootstrap-driver.py --self-test`.

Result: exit 0; 41 cases; 38 compiler successes; three compiler rejections; 16 runtime mutants; 15 source-only mutants; forged success without an observer launch rejected; each mutation restored to exact frozen source bytes. The result explicitly reports `ready_for_072_dispatch: false`.

The tested bytes have these SHA-256 digests:

| Evidence file | SHA-256 |
| --- | --- |
| `evidence/architecture-bootstrap-driver.py` | `a9a352710790f436687e6d65877de3ea298cd58a37799b176c46a13d9004323b` |
| `evidence/architecture-bootstrap-vectors.json` | `08483b2d157a64fbab1caf4874d5ae6f57e7c0ae6910363dac8502de1c38ced6` |
| `evidence/architecture-bootstrap-fixture/app.rs` | `ab68e3911d94f6c8b852adee385b882c44648ca56213c7282ba9dbf1ed4fe9fb` |
| `evidence/architecture-bootstrap-fixture/library.rs` | `c1dd8b7abd6dd36fb7fa9f6710e435cb2232c201a358f15dfadb20c1b65b58d4` |
| `evidence/architecture-bootstrap-fixture/main.rs` | `397a8f5cd59c96acee4c35fc63dbb4707c44e61dac058e42eea62b37a0593da9` |
| `evidence/runner-bootstrap-driver.py` | `5048853686c820bedf98a17a1b9f741e7b0f9de0448090a0d9ba5fca04308737` |
| `evidence/host-bootstrap-observer.py` | `0e95a95c17163d269533a67a062fe3aa900dc75b9e27dae19f1ca28bd5c9e5d1` |

The driver independently compiles private Rust sources, records source and executable digests, and executes the resulting binary through the protected observer. The candidate cannot supply an executable, command line, expected result, or observer event. The inherited launcher checks request identity, source tree, nonce, single execution, exact event digests, result schema, protected input stability, and bounded output authority.

The fixture executes update and draw with 24 combinations of disabled, busy, poison, and slot state. Checks compare every integer cell, every paint write and owner, retained value, actual phase counts, and exact resolution provenance. Poisoned library output distinguishes copied, dead, discarded-buffer, and overpainted calls. Runtime mutants also test false part ownership, missed restoration, invented owner IDs, registry omission, and unused slots.

The 15 source-only negatives intentionally retain valid runtime behavior. Receiver policy, duplicated props, unreferenced syntax errors, and absent or empty roots therefore still require the future production parser. The self-test validates their compiler/runtime premises; it does not demonstrate a successful parser rejection. The private associated/free helpers, explicit `this: &Self` data argument, trait delegation, legitimate art, dynamic IDs, unconfigured constructions, and test-only construction are positive controls.

A second independent execution used the proposed sibling directory layout in `/tmp/tc-rust-review-layout.8rfw7D`: `architecture-bootstrap/` contains the Rust driver, vectors and fixture, while `runner-bootstrap/` contains the inherited driver, observer, application and worker. All 41 cases passed again. This run used the updated runner driver SHA-256 `cb2e11775b155858c386e08c234de4daa66e3800803534c6f9fb86f7d29d18de`; its launch authority remained unchanged on inspection. Transitive files `runner-bootstrap-app.py` (`682d8e14c480b85175f78748ca25dd163f501a63b9751257007147b679e2fd2c`) and `runner-bootstrap-worker.py` (`e5bd488a947bd7fa43d46b9f3658016c3d6a586e60ccfba093aff24d2678198a`) are required because inherited fixture construction reads them before installing Rust sources. Omitting them fails closed with a missing-file error. This verifies the standalone relative import layout; final assembly verification is recorded below.

The fixture uses a miniature `Ui`, four integer cells, and a constant registry. It cannot establish actual junie `Ui`, `RowUi`, `ColumnsUi`, canonical styled cells, geometry, focus, hit regions, production registry enumeration, or all production states. Its protocol preserves this distinction explicitly. No future TASK073 API is imported or required.

## Independently executed actual terminal-components evidence

The reviewer copied the stable probes and their preparation driver to `/tmp/tc-rust-review-layout.8rfw7D/actual-probes/` before execution, isolating this review from later adapter edits. Command: `rtk proxy python3 -B /tmp/tc-rust-review-layout.8rfw7D/actual-probes/architecture-bootstrap-main-driver.py --repository /Users/donbeave/Projects/terminal-components-claude`. The entire command exited 0.

| Actual-source evidence file | SHA-256 |
| --- | --- |
| `architecture-bootstrap-main-driver.py` (reviewed preparation version) | `5c1ff3562fc11b051fe44f270c4924f8f58bb6947760924787b6727841fe2c1c` |
| `architecture-bootstrap-main-probe.rs` | `235ebd62add080c5ea6b554c493fa1456049cd3c1c3b973b4943d1e41f73d1c6` |
| `architecture-bootstrap-conformance-probe.rs` | `4ab0ebae375b17c3dd30787ab567fec6218cc787eb3f08e57faf698ac9d03f50` |
| `architecture-bootstrap-nested-row-probe.rs` | `ae1082058743a4b50b30034578ba9b07390793fbd93eb96bb61f71fcf08109ed` |
| `architecture-bootstrap-rain-probe.rs` | `71dcd5d13d240dad8c33a3eeb7e8e73862180c4a2bd504d4a20195d7fbe9dc2f` |

All 11 Showcase variants compiled through real offline, locked Cargo builds and executed the compiler-selected test executable under the protected sandbox. Each checkpoint captured all 4,800 cells, including UTF-8 symbol bytes, foreground, background, underline color and modifiers, plus focus, registered Grid area and rendered text. The recorded executable digest is checked again after execution. The driver validates the whole archive and selected original Git blobs before applying private fixture mutations, and verifies the exact complete source-file inventory after each replacement.

The original Grid picture is unchanged when its real metric model is perturbed, but removing the compatibility paint exposes metric cells that respond to that perturbation. Dead and reference-only Grid draws retain the original picture while changing the production keyboard outcome. Removing the actual Brand or StatusBar call also leaves the original chrome picture unchanged; removing chrome compatibility painters exposes real component output, including perturbed Brand text. These are measured negative witnesses from actual application paths, not integer-model claims or successful old boundary scans.

The actual conformance macro wrapper executes all 44 existing registry entries, enumerating each case through the original `Conformance` API rather than a second manual sweep. It observed 14,448 style resolutions. Its full-registry output retains missing and extra PARTS as unresolved coverage/architecture evidence; this review does not transform those differences into a passing full-registry architecture verdict. A source-sliced original Button case supplies a positive control with exact PARTS equality and 21 state/geometry combinations. Removing the Probe registration produces exactly 43 cases. An extra owned part and an unreachable declared part each produce exactly one expected Button difference. Ignored ICON slots, a measurement-only style query, slot documentation drift, omitted capability-required state and a fabricated RowUi owner all produced the expected actual Rust failure, exit 101.

The slot probes execute 12 paint cases and 12 hit/focus/activation cases across Ready, Busy and Loading. They compare the actual public rustdoc slot declaration with parts that change real buffer cells, preserve returned geometry and neighboring cells, compare control and part hit areas and focus-ring IDs, and check activation behavior. The actual row and column probes retain real owner IDs and use existing Rust caller locations to distinguish client row callbacks from default library rows. A separate private unit probe verifies the existing outer RowUi owner's value and before/inner/after resolutions; deliberately corrupting that existing field after the nested return fails with exit 101. It also verifies real A/B/C/D cells. No new public ownership API is introduced.

Both legitimate extension controls passed: Jackin's actual rain painter changes cells outside a real Button while preserving the Button's cells, registration, focus and two activations; the actual external-author example runs nonempty Rust tests successfully.

The author repaired three review findings before this run: a focused actual-component positive now avoids treating an already negative whole-main subject as mutant discrimination; actual nested owner restoration and state-omission witnesses now exist; exact source replacement now removes stale injected files and verifies the complete inventory. The final adapter review below closes the remaining preparation transport checks.

## Final protected adapter and layout acceptance

The final independent run copied all 14 architecture assets and the four required transitive runner assets into `/tmp/tc-rust-review-layout.8rfw7D/final-architecture/{architecture-bootstrap,runner-bootstrap}/`. Both commands exited 0 from this relocated sibling layout:

```sh
python3 -B /tmp/tc-rust-review-layout.8rfw7D/final-architecture/architecture-bootstrap/architecture-bootstrap-driver.py --self-test
python3 -B /tmp/tc-rust-review-layout.8rfw7D/final-architecture/architecture-bootstrap/architecture-bootstrap-actual-driver.py --self-test
```

The standalone run again passed all 41 premises. The actual run compiled and froze all 25 actual-source subjects, replayed both passing and failing binaries after relocation, passed two real external FD positive/recovery executions, rejected an always-pass process on a genuine compiled extra-owned-part subject, and rejected both forged event content and success without observation. Each attack must fail for its exact independently specified reason and event count; an unrelated exception cannot satisfy it. The transport-only executable used by this rehearsal is explicitly not a production checker.

The adapter freezes source bytes, compiler artifacts and expectations before launching any candidate. Its observer executes only its own immutable artifact with fixed arguments. It binds actual Git tree, original source archive, full source hashes, compiler/Cargo identity, lockfile, executable and raw event hashes. Raw stdout/stderr remain available as base64, including actual cells and retained state. A measured failing-binary replay difference contained only a runtime thread ID in the Rust panic header. Semantic replay excludes only that anchored numeric diagnostic field and the anchored libtest final wall duration; original raw bytes/hashes, test names, source locations, assertions and all cell/state observations remain intact.

The final execution-bearing asset digests are:

| Asset | SHA-256 |
| --- | --- |
| `architecture-bootstrap-driver.py` | `03a21847f508e24bdbf3f8f20552e810507179ef3aa024a28e9d26c6f473d915` |
| `architecture-bootstrap-actual-driver.py` | `6126ddefffb24c4fe459fb6cf753a5580774682068c3f8ba40a64f35ae08548a` |
| `architecture-bootstrap-main-driver.py` | `4a08097beacac8008c8c3a654a92620ee2613dcc52b2589ce3adb25c014aa412` |
| `architecture-bootstrap-protocol.md` | `54616b03f78de9ae73f4d318070950fffbe921a6744a7d81dd09fa514b601dc3` |
| `architecture-bootstrap-state-vectors.json` | `03742cb09ce0c9605b1db758e4fb35aefc3e1d16ff207168dafc85fbc0e6fcb8` |
| `architecture-bootstrap-vectors.json` (approved readiness metadata) | `72c20cadbfe2bbb9dea09ee48e00f857961ce025dab8d78694b0b891d4f85380` |
| `main-source.tar.gz` | `ab9568812e86e3c8e18c1d945886d4b2387b1e175ad61ba4d60750b5d6f98119` |

The four Rust probe digests, three miniature Rust source digests and four transitive runner digests remain those recorded above, using runner driver `cb2e11775b155858c386e08c234de4daa66e3800803534c6f9fb86f7d29d18de`. The tested vectors had digest `c6c6cdfc1fb5730b0e7d1961a7b66270e50642b1a896c6fbe2f3583e34849947`; after the successful independent run, the reviewed metadata-only change enables readiness and clears preparation blockers. All 41 cases and execution-bearing files remain unchanged. Decompressing the frozen archive independently produced the expected tar SHA, 5,029 source files and all seven expected original blobs.

CHK-011 uses `python3 /task/trusted/architecture-bootstrap/architecture-bootstrap-driver.py --runner /work/tools/refactor-proof/bin/tc-proof`, maps R-001 to AC-009, and requires exit 0. The outer driver runs the standalone matrix and then the actual matrix. README R-001 and protected obligations explicitly require both, including positive/mutant discrimination and external protected observer transport. The task's only dependency remains TASK071; the fixture APIs are existing pinned APIs and private observations, so no TASK072 dependency on TASK073 is introduced.

After the coordinator froze TASK072's assets, the reviewer independently compared all 14 files under `refactoring-tasks/terminal-components/completion/072/trusted/architecture-bootstrap/` with the final evidence sources, plus the four required sibling runner files with the executed relocated copies. Every byte comparison passed. The final campaign receipt must retain these identities. Later TASK072 acceptance must rebuild the actual submitted dispatcher and prove that profile inputs reach its production parser/checker, without a fixture-specific verdict branch. No checker implementation or checker acceptance has been performed during this planning review, and no terminal-components production repair or parity acceptance is implied.

Reviewed TASK072 contract digests: README `8bd3304c3c9cd735100a8abed3b6b5a06b7d451e7895f7a2382444ddcc007eb6`; task metadata `72d3244c9370ad607b88aae871ab3d80c29d96f99617c73bcd445e4a4fb7f98e`; verification manifest `03d4d19c18b879f82f59879a6dad239b9ae51c7bfdd2a4e80cb3de5e29135be5`; protected obligations `e5c4383cf0a40934c116fba1b54c948ddbb887224db77fb5b511c241d02e26ae`. Canonical task-format validation is owned by the coordinator; this review independently checked the concrete command, typed references, scope, and prerequisite relationship.
