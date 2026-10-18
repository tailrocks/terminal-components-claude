# Independent closure, graph and execution-protocol reaudit

Reviewed 2026-09-11. Planning only. This report covers TASK-065–069 and their complete delivered file inventory, global dependency/integration contracts and current standalone task-format semantics. It is not whole-goal acceptance and does not claim that future proof executables, sealed oracle bundles or implementation gates already exist.

## Findings

### CLOSURE-01 — Binding TASK-065 source clauses contradict the accepted source-scan boundary

Status: confirmed; coordinator owns canonical ledger repair.

Locations: `refactoring-tasks/terminal-components/completion/065/trusted/source-obligations.tsv:6` (HIST:A47), `:21` (EARLY-AMEND-042), `:32` (HL-73-PROPS), and README R-001 at `:46`.

A47 requires “Enforce helper ownership and cross-file reach; no silent exemptions”. The two later exact clauses explicitly retain intra-module scope and reject a cross-file proof claim. README makes every mapped exact clause binding. Pinned architectural source `7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md:2193` explicitly says this is an intra-module/source-scan contract, not a cross-file call-graph proof; `:8577` repeats the limit. Thus the generated task simultaneously requires and excludes cross-file proof. A real executor must either exceed the accepted architecture or silently ignore one mandatory clause.

Root condition: a broad historical summary was promoted into the canonical remaining-work field without applying the narrower accepted amendment. Fix the canonical A47 row and mechanically regenerated task copies; add a bounded fixed decision to TASK-065. Require receiver rejection, configured same-ID sharing, explicit exemptions and fail-closed source/parse checks within the accepted intra-module boundary. Do not introduce whole-program call-graph analysis or pretend cross-file completeness.

### CLOSURE-02 — Actual style-frame attribution has no assigned production measurement seam

Status: confirmed; coordinator authorized one measurement-seam owner, TASK-073, coordinated with TASK-011.

Locations: `refactoring-tasks/terminal-components/completion/067/trusted/obligations.md:13`, `067/verify.toml:3`, and `067/README.md:46`.

TASK-067 must measure actual style-resolution time divided by the real restored Showcase Lists frame time, at most 5% under the strict profile. Its writable scope contains only six performance test files. Pinned main `apps/showcase/tests/perf.rs:27` measures rendering/digests, while `:129` proves only repeated digest equality despite its allocation-oriented test name. `crates/tui/tests/perf.rs:229` explicitly describes the synthetic foundation A/B stand-in. `crates/tui/src/ui/mod.rs:589` exposes style-cache hits/misses, not a duration. The existing TASK-011 and TASK-073 contracts at this review snapshot supply correctness/attribution observations but do not assign actual style-time measurement.

The old synthetic differential, query-count extrapolation and total-frame elapsed time do not independently measure the required numerator. An executor confined to the listed test files otherwise has to invent additional measurement architecture or violate scope.

Root condition: the closure task names a missing measurement without assigning the source-side capability that makes the assertion observable. Assign one narrow testing-only production seam to TASK-073, inherited unchanged by TASK-011; keep TASK-067 as its real-app consumer. Freeze exact timed region, complete or explicitly qualified sampling membership, clock/profile/optimization policy, raw timer overhead and calibration treatment, and authentic production-frame denominator. Require equal actual frames/semantic state with observation enabled/disabled and no allocation or cache-key contamination. Independent proof must reject a constant/foreign numerator, skipped resolution calls and a forged or enlarged denominator. Do not create a synthetic state model or revive the expired stand-in as the final proof.

## Reverified contracts and evidence

- Local immutable tag resolves to `02f5294bfdbf38004cc49130d0aff1d01f31434c`; local main resolves to `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- Installed taskfmt and its read-only source checkout both identify `52d9f1eb7721f409bc47beb9fced7997b5c13ede`. Relevant gate/config/verify/template files match their checkout HEAD. Full project lint exits zero: 73 packages, no package errors or warnings.
- Canonical metadata graph reproduces 73 tasks, depth 33, 24 deepest paths and zero unordered writable-scope overlaps. All 72 other tasks are ancestors of TASK-069; every production task TASK-009–069 has TASK-001–008 as ancestors. All 150 Jackin/TablePro stage-audit rows include every declared producer in the complete owner's ancestry.
- All five AGENTS files are byte-identical to the current template after TASK-000 substitution. The campaign adaptation explicitly supersedes the incompatible empty-progress/executor-authoritative sequence while preserving final canonical progress validation.
- Pinned taskfmt `harness/src/cmds/verify.rs` supplies full contract enforcement. `gate.rs:251–358` executes checks in declared order, independently checks scope/protected paths and checks progress after declared checks. `:361–384` resolves explicit base, environment base or configured base and rejects absent immutable base. `:445–461` confirms bare paths cover subtrees and wildcards cross slashes. `:808–839` requires parsed DONE progress. These semantics match the campaign's explicit-base, nonempty-progress host handoff.
- Integration begins from pinned main before proof preparation, preserves accepted preparation ancestry and defers production work until complete reference/disposition receipts. The host freezes actual filesystem content in a fresh object database, binds exact trees and progress, isolates candidate code from expected evidence and uses expected-parent local ref updates. Parallel joins require combined-tree checks. Nothing here authorizes main mutation or pushing.
- TASK-066 explicitly owns assertion-preserving render-target relocation and atomic CI/command migration. New command identities must come from independently accepted relocation evidence, not its writable inventory proposal. Archive bytes and oracle expectations remain protected.
- TASK-068 resolves public facade/documentation/35 reference dispositions after ownership, relocation and performance closure. It does not revive obsolete APIs or authorize crate publication.
- TASK-069 requires all original parents, derived contributions, native assertions and lane-specific exact frame/semantic products together; zero unresolved identities; every accepted build/API/performance gate; fresh independent reviews; authentic receipts; exact tested tree and remote-main drift check. Report-only writable scope cannot repair or waive a failing product.

## Per-file semantic coverage

Every delivered package file below was included in the review. README sections, requirement/acceptance/check/checklist links, exact scopes, dependency sidecars and canonical AGENTS were inspected. All historical source-obligation rows were parsed and their decision, authority, remaining-work and proof fields examined; the cross-file contradiction above came from comparing those fields rather than schema lint.

The 069 contribution contracts were read for parent/contribution separation, seed qualification, complete-frame selection, native/resized assertions, all-lane honesty and final closure. All copied contract/table bytes match their campaign originals. All 986 `holla-route-sites.tsv` identities are unique, and every row's source blob SHA-256 and exact stripped source line were independently compared to the immutable oracle: five source files, 139 input/spatial sites, 82 constructors, 19 combined assertion/input sites and 746 assertion sites. This validates every delivered source-site row; it does not claim every site has already executed in a future adapter.

The complete 150-row Jackin/TablePro and 135-row Holla stage tables were parsed for identities and final inclusion; all declared Jackin/TablePro producer ancestry was checked. Their detailed application-domain judgments remain subject to the separate application reviewers, not an invented second complete application audit. All 84 Jackin/TablePro contribution clauses in TASK-069's source obligations were reviewed for original-parent preservation, explicit seed/whole-frame boundaries, and typed final comparator versus semantic-architecture assignment. The shell and Holla contracts independently require their complete table unions at TASK-069 even when the historical source-obligation file is not a second copy of those rows.

TASK-065 coverage: ownership/props and exceptions, purity, production roots, dependency boundaries, old/new duplication and negative witnesses. TASK-066: exact inventories, assertion meanings, source profiles, preserved archive lineage, target relocation and no zero-match/listing-only proof. TASK-067: all absolute allocation/byte bounds, real workload dimensions, strict ratios, warm/cold/append distinctions, allocator isolation, counter honesty and expired benchmark treatment. TASK-068: actual signatures/visibility, external consumers, documented APIs, no-default-feature/MSRV boundaries, suppression expiry and semver deferral. TASK-069: full application/component union, exact oracle equality, full accounting, independent reviews and integration readiness.

## Snapshot fingerprints

Fingerprints below describe the reviewed pre-repair snapshot. Authorized repairs require new validation and updated fingerprints; these hashes do not certify later bytes.

| File | Lines | SHA-256 |
| --- | ---: | --- |
| `refactoring-tasks/terminal-components/completion/065/AGENTS.md` | 83 | `5fd87254cbaacce871f0f9113a83d09400e28bc12389c1089d0907f3c805f487` |
| `refactoring-tasks/terminal-components/completion/065/README.md` | 184 | `9e2d36d8fff8ff377087c925af5141d009109f6e93c6ba75c9cfabdd880644c0` |
| `refactoring-tasks/terminal-components/completion/065/task.toml` | 3 | `d34538f86d45f02d33fd8e610f6153b3f1cb7e56f9ef2b8ef5ca19c3232be6d0` |
| `refactoring-tasks/terminal-components/completion/065/trusted/obligations.md` | 29 | `9f331ddf30299dc46907a81a2559558ba3f73abe54ff41c20743dd00d5f13135` |
| `refactoring-tasks/terminal-components/completion/065/trusted/source-obligations.tsv` | 49 | `5fe63bc47637fdbf4b871445996eb34f32dd4910b6368c01cde67a872fd25c39` |
| `refactoring-tasks/terminal-components/completion/065/verify.toml` | 60 | `ebd82d5d7d8a3150be99dfd4e06a94a68c5dced13aaffd6d60fcf95a36aeb854` |
| `refactoring-tasks/terminal-components/completion/066/AGENTS.md` | 83 | `4ce0db89d92edfd4fe762fd13037b8683080158d9c727b400677fd66f2a280b3` |
| `refactoring-tasks/terminal-components/completion/066/README.md` | 184 | `ca3d71062b97ceaa85e799b4af818dbccd8db59da05202c02dcafd99b5538382` |
| `refactoring-tasks/terminal-components/completion/066/task.toml` | 3 | `42355404410195f0fd717835ee759aca6173a0cc94df170bbd0c85e709b85e19` |
| `refactoring-tasks/terminal-components/completion/066/trusted/obligations.md` | 31 | `c32dc73a7d3768004d5330e78cb206bfb53b378983b80c0435f77fc493c3f92f` |
| `refactoring-tasks/terminal-components/completion/066/trusted/source-obligations.tsv` | 29 | `a448b317517f3ca9f6516ce0b8c2e71e02468f8f9f131e3c85433d3dce5bce68` |
| `refactoring-tasks/terminal-components/completion/066/verify.toml` | 60 | `af7f1b3e9507ad512fc7d0f958099b1bdb73b00585374aaf2726744c73d79a4c` |
| `refactoring-tasks/terminal-components/completion/067/AGENTS.md` | 83 | `b738e00cf9a715ab9c89fa73ff34a38b18de9b623f758ce8ba4117f5715dac41` |
| `refactoring-tasks/terminal-components/completion/067/README.md` | 184 | `121585236e8d097820e43bc9461b932a19101b2b34e65344df8c0ce03abf889e` |
| `refactoring-tasks/terminal-components/completion/067/task.toml` | 3 | `e59f5f11a82c2938fa88c4d60ec3672abdad487ab614d88f4c57a12680cc6d91` |
| `refactoring-tasks/terminal-components/completion/067/trusted/obligations.md` | 39 | `5bb2fe62eeed9da583ee80cd0007d5ae03dd38c6aa09cf1ee389a9081333fbfc` |
| `refactoring-tasks/terminal-components/completion/067/trusted/source-obligations.tsv` | 54 | `fce6d8178c2687a9d0f762576dfcf9d42bd66885ef9c754b31f75490e5c4217c` |
| `refactoring-tasks/terminal-components/completion/067/verify.toml` | 60 | `9df98218f1790eaa25f28eb60aa547fbfbcbfd420ac4d030212ab00f31d1edd8` |
| `refactoring-tasks/terminal-components/completion/068/AGENTS.md` | 83 | `f959d9d2c5e0a0038b73632095d5779abd1c98d7b25d7c95cda34d8b6246d52f` |
| `refactoring-tasks/terminal-components/completion/068/README.md` | 184 | `e3e91f0c9bd512b0294289c1201a0b785ba25853e922a87fdeaf2da2eef9b3d1` |
| `refactoring-tasks/terminal-components/completion/068/task.toml` | 3 | `d4d2119375341d2e3e8aa61e977ca952b2f8331bf2f1604a9572bc51b6181cd5` |
| `refactoring-tasks/terminal-components/completion/068/trusted/obligations.md` | 35 | `7ed5eeff11664d8ba08d29a28457ebb683182daf4a845268aed2d7b79b418b1c` |
| `refactoring-tasks/terminal-components/completion/068/trusted/source-obligations.tsv` | 56 | `112824c361125fe2f33cfa272070cdf9bdc08b1223a5bec804bc7f78ff251488` |
| `refactoring-tasks/terminal-components/completion/068/verify.toml` | 60 | `07e01c20bfebbb7952037447e501389ea27e2bc3dc55a7311478ae0085a7abe0` |
| `refactoring-tasks/terminal-components/completion/069/AGENTS.md` | 83 | `a742721ad41eec30a3821eadd9d570e2b78e893baef29ac8edff972e78206245` |
| `refactoring-tasks/terminal-components/completion/069/README.md` | 170 | `d1a59ed483bb7f044d896bb720ac22cd1d3b548006f0cecf905a5314cad027e1` |
| `refactoring-tasks/terminal-components/completion/069/task.toml` | 3 | `51c56e998135953bd55189b0f5daf89848ea864b05fdcde237cf366ae03fc70a` |
| `refactoring-tasks/terminal-components/completion/069/trusted/app-flow-contribution-contract.md` | 39 | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `refactoring-tasks/terminal-components/completion/069/trusted/app-flow-contributions.tsv` | 43 | `b21362fcc94ae5f4277fa7e4197ddab223ba068f4581526e1f2fb97cbdbcfa2f` |
| `refactoring-tasks/terminal-components/completion/069/trusted/app-flow-frame-contributions.tsv` | 43 | `5e29488474c8e3752968a130e25a4f6c5a116a6d859983ed8bfd4dae0ba933c1` |
| `refactoring-tasks/terminal-components/completion/069/trusted/app-flow-stage-audit.tsv` | 151 | `e0fdeb97a4e8a393aa68393ddb3b647af372e8d976f0f395c8c2a64372af929e` |
| `refactoring-tasks/terminal-components/completion/069/trusted/holla-route-expansion.md` | 55 | `aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550` |
| `refactoring-tasks/terminal-components/completion/069/trusted/holla-route-sites.tsv` | 987 | `eb3c6d66bffd441fd61964b41e6c08f0c2db2ba33ec02f0184bde43834aba045` |
| `refactoring-tasks/terminal-components/completion/069/trusted/holla-stage-audit.tsv` | 136 | `f5c33a9579d6a4b3062c98cd47a45f1ceb63008f6b79cb1370e3bafb0980cf5e` |
| `refactoring-tasks/terminal-components/completion/069/trusted/holla-stage-contract.md` | 31 | `2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1` |
| `refactoring-tasks/terminal-components/completion/069/trusted/holla-stage-contributions.tsv` | 24 | `700107ac6f3548c9c5765350f500241d4b6b128c483ac8421a1aff9f803b5a80` |
| `refactoring-tasks/terminal-components/completion/069/trusted/obligations.md` | 39 | `166973af8bab9a364e09e4c8a401650a280d27bbc64aa0fb76d586714406cc8e` |
| `refactoring-tasks/terminal-components/completion/069/trusted/shell-contribution-contract.md` | 38 | `f6e61281935fbaa5123cc66136861bd301f5954f695c058c2857d4545f3dccc5` |
| `refactoring-tasks/terminal-components/completion/069/trusted/shell-contributions.tsv` | 11 | `8805a7d739e196a7ca801860047d0c11e353410845c0d6251fe855dce25da84e` |
| `refactoring-tasks/terminal-components/completion/069/trusted/shell-frame-contributions.tsv` | 3 | `ddf9d9cbcc872b97af43a01166dcbf86497b6f5dc86cfda9334c4b2210040f1e` |
| `refactoring-tasks/terminal-components/completion/069/trusted/source-obligations.tsv` | 125 | `ca494f894e944878bfe67d10d3e0517e8e01031be86b8b4de64580c8ddcd3408` |
| `refactoring-tasks/terminal-components/completion/069/verify.toml` | 60 | `758eae552c72404cd0f7da55dc3a2503fe3ea5ccd4acdb67a3bbc4bd02f63b94` |
| `docs/refactoring-plan/proof-contract.md` | 242 | `2a9599117aebbd560023b577dc11db350f710529c29b8a05bc8e4b812a3e69ce` |
| `docs/refactoring-plan/campaign-executor-protocol.md` | 19 | `a8cf605e3fd58fe152b1e1f6db6b363fdb52923538aae6bd0f9cb73a5e0adf61` |
| `docs/refactoring-plan/task-format.md` | 65 | `823909e2dfbbcad8998c60d999622d62d0e7ce5b8bb69e8385d06e42559cf1a5` |
| `docs/refactoring-plan/task-graph.md` | 59 | `4e5190ca02da03eda871f2dba6e5fe10c3d9faa44abf7e7c08f1dd2b66d70ce6` |
| `docs/refactoring-plan/task-graph.json` | 1693 | `631917da6b00a48f924d7e3a7df8821fb551ae4e17ccaa4023faf357cd4953c8` |
| `docs/refactoring-plan/task-index.tsv` | 74 | `1020d88667367bcc7b6100f2def51126971db576bf08ac15a309758b6ce86bb0` |

No terminal-components production file, ref, commit or task-format source was modified during this audit. The only audit artifact written was this report. The coordinator's subsequently authorized task-package repairs are separate from this snapshot.

## Authorized package repairs after the audit snapshot

The coordinator subsequently authorized narrow TASK-065–069 package repairs. TASK-065 now has D-007 and an exact closure clarification binding receiver rejection and approved exemptions to the accepted intra-module/source-scan boundary. Canonical A47 reconciliation and regeneration remain coordinator-owned; this clarification does not conceal the contradictory pre-repair source row.

TASK-067 now names TASK-073 as the sole production style-probe owner, requires its accepted ancestry and fixes the real-workload consumer contract. Its P-004, R-002, D-007 and first protected closure clause require three paired all-call modes, full frame/state/call-census preservation, independent numerator/calibration/denominator accounting and conservative threshold proof. TASK-011's owner confirmed it preserves that seam instead of adding another. TASK-073 implementation planning and protected qualification additions remain with their assigned owners. Neither finding is declared independently closed by these edits alone.

Both modified packages pass pinned taskfmt lint with zero errors and warnings. All five AGENTS files remain byte-canonical after task-ID substitution. No verify scope, dependency metadata, production source or baseline was changed by these repairs.

| Repaired file | Lines | SHA-256 |
| --- | ---: | --- |
| `refactoring-tasks/terminal-components/completion/065/README.md` | 185 | `e44bb6670740b787194e8361ceec29300b551709bd1d43f4c54f76904387c1ac` |
| `refactoring-tasks/terminal-components/completion/065/trusted/obligations.md` | 31 | `8318778a8229ee992c11813e27b2fedf33b0bfbc1dd5679f64cc294ca5d5e451` |
| `refactoring-tasks/terminal-components/completion/067/README.md` | 186 | `ca64ad9bd128303d0de08d24954c928e96f7934a72d173990b0436dbb3940195` |
| `refactoring-tasks/terminal-components/completion/067/trusted/obligations.md` | 45 | `3309eda71411a09fa384a8271ae284a5e2043504c10320d7846cd1bd0f8a7e63` |
