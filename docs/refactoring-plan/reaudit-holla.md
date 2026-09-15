# Holla adversarial re-audit — 2026-09-11

## Scope and evidence

Reviewed TASK-040 through TASK-050: 114 package files, 9,702 lines. The appendix records the exact pre-repair bytes. Repeated canonical protocol/check material and copied scenario fields were read with content deduplication, then checked against their complete source records. No production refactoring or commits were performed.

Verified immutable oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c` and architectural main `7b27732a8c3c131760ec3438f641cb3c11343a42` using `git rev-parse`; inspected source with `git show`. All 986 native-site records matched their pinned source SHA-256 and exact source line. Distribution: app_tests.rs 27, app_tests_flows.rs 242, app_tests_parity.rs 538, app_tests_rows.rs 107, app_tests_proofs.rs 72. This verifies site provenance, not execution or exhaustive Rust branch coverage.

All 135 Holla scenario records were joined to their complete owners, the 23 stage contributions and package copies. Every copied obligations.md scenario field was byte-equal to the register; contribution subsets were exact. All eleven packages passed `taskfmt lint` using `/Users/donbeave/Projects/donbeave/task-format/experiment.toml`. Structural validity does not prove the routes implement their stated behavior.

## Semantic coverage

- TASK-040: permanent Here/tab/page ownership, menu/help/quit/close shortcuts, three semantic shell contributions, full undersized frame contribution, preview CLI and terminal profiles. Seven checks and frozen-host/progress binding reviewed.
- TASK-041: discovery/catalog/ranking/history, query editing, narrow drawer versus split, five source contributions, fifteen complete primary rows. Config parsing belongs here; later Activity/Config/Docker products remain separately owned.
- TASK-042: filesystem indexing/browser/preview/jump, stable paths, safe text and resource actions, three contributions, seven complete primary rows. Preview selection route is defective as specified below.
- TASK-043: Args/trust/review/gates, five contributions, five complete primary rows. Both Args and overlay-paste specifications contain source contradictions.
- TASK-044: Activity/streams/stdin/cancellation/World ownership, six contributions, twenty complete primary rows. Seeded ActivitiesMulti follow-up remains047; complete HP14 Git-batch constructor remains047.
- TASK-045: Disk/scan/tree/Spotlight/insights/platform facts, four contributions, fifteen complete primary rows. Read-only cleanup validation reuse and046 effect ownership checked.
- TASK-046: cleanup modes/gates/commit validation/Trash/no-fallback/report/quit and fifteen complete primary rows; zero new partial contributions.
- TASK-047: Git/native/Cargo/Gradle/IDEA and seventeen complete primary rows; includes complete HP14/output/Activities follow-ups.
- TASK-048: Docker/Compose/Brew/PG/SSH/system/port and seventeen complete primary rows; includes corrected late Args ownership but its asserted successful value is wrong.
- TASK-049: config/custom/upgrade/all remaining snapshot routes and fourteen complete primary rows. Enumeration uses oracle source identities, not candidate discovery.
- TASK-050: validation-only union of all135 rows, shell contributions and23 stage contributions, complete native/resized mapping. No production repair scope; actual source/route contradictions must be corrected before dispatch.

Across the packages, reviewed goals, contexts, preconditions, writable/forbidden scope, every R/AC/check mapping, fixed decisions, checklist, source-ledger roles and copied obligations. Shared UI mechanics remain owned by prerequisite components; no application-local widget duplication is proposed by this review.

## Findings requiring repair

### H-01 — Preview copy transcript exits the process before its find journey

Location: `docs/refactoring-plan/holla-scenarios.tsv:113`; copied by `completion/042/trusted/obligations.md:93` and TASK-050. `holla-stage-audit.tsv:113` claims the complete listed route was reviewed.

Violated source: oracle `src/bin/holla/app.rs:575–578` handles Ctrl+C globally by setting quit=true before FilesPage gets input. Files is not an attached Activity. `src/widgets/viewport.rs:1386–1389` implements selected-text copy with plain `y`; FilesPage forwards that Copy event at `screens/files.rs:1231`.

Counterexample: open a real text file preview, establish selection, issue Ctrl+C. The application sets quit=true. A real runtime exits; the subsequent `/`, `TYPE(site)`, Enter, Up, Escape cannot constitute a live executable journey. The H test helper can still call App after quit, so model continuation could falsely certify this sequence. Moreover, Shift+Right/Shift+Down from the default end caret can select nothing; copy must require an actual nonempty selection.

Root condition: an intended operation name was translated into an assumed conventional shortcut without checking application dispatch precedence or terminal state. The scenario asserted successful outcomes without a live-process predicate.

Structural repair: retain the original trace as rejected planning evidence. Add separate named source-correct copy/find and Ctrl+C-exit branches inside the same primary row. The live branch uses source Ctrl+Shift+Home, proves nonempty selection, uses `y`, requires clipboard payload/status and reaches find checkpoints with quit=false. A pointer click only creates a drag anchor and clears the keyboard browse caret (viewport.rs:1092); it cannot establish the assumed keyboard caret. The exit branch ends at process termination and terminal restoration; post-exit input never certifies product behavior. Empty previews retain explicit no-selection/no-copy outcomes.

### H-02 — Port field insertion is masked by snapshot fallback

Location: `docs/refactoring-plan/holla-scenarios.tsv:114`, `holla-stage-audit.tsv:114`, `holla-stage-contributions.tsv:11` (HO-043-F02), TASK-043/TASK-048 copied obligations.

Violated source: catalog.rs:2301 initializes the required port field to5173. input.rs:256–274 clicks at a caret and clears selection; input.rs:237–243 inserts paste. review.rs:999–1013 checks required nonemptiness only. app.rs:2600–2606 preserves raw argument text in the snapshot kind. snapshot.rs:337–339 parses u16 and silently falls back to5173.

Counterexample: click the first text cell of the existing5173 field and paste5173. Actual draft becomes51735173; submit produces raw `port:51735173`, yet rendered title remains `Port 5173` because of fallback. An arbitrary nonempty `abc` also submits and displays fallback5173; it is not a validation failure. Ctrl+A is Home in this oracle, not select-all; field_common.rs uses Ctrl+L for select-all.

Root condition: rendered title was treated as proof of exact accepted argument identity; the transcript never replaced the initial value and its undefined invalid-text branch assumes absent numeric validation.

Structural repair: preserve and name the original insertion/fallback branch. Add explicit Ctrl+L replacement with5173, empty-required refusal, and nonnumeric fallback branches. Freeze raw field value, emitted snapshot kind, displayed port and action argv separately. Keep early frame contributions before each successful submit and through the empty-required rejection. Full Snapshot descendants remain048. Do not add numeric validation to production; that would alter oracle UX.

### H-03 — Menu-paste isolation requirement contradicts oracle passthrough

Location: `docs/refactoring-plan/holla-scenarios.tsv:134`, TASK-043 `trusted/obligations.md:71`; TASK-040 R-002/D-001 use a broad paste-owner formulation.

Violated source: app.rs:480–498 routes paste to a top modal if present, otherwise straight to the page. It never checks `menu.is_open()`. F10 opens MenuBar in app.rs:553; ArgsPage::on_paste forwards to its focused field. Consequently F1/Ctrl+G/Ctrl+Q modal branches isolate the underlying field, whereas F10 intentionally receives page passthrough at this immutable revision.

Counterexample: start Args editing, press F10, paste café, press Escape to close the menu. The underlying Args draft now contains café. The row's universal no-hidden-page-mutation assertion is false on the oracle itself.

Root condition: the plan generalized keyboard menu capture to paste without examining their separate dispatch functions.

Structural repair: distinct named true-modal isolation and MenuBar passthrough branches with source-qualified owner observations. Preserve the menu quirk and its exact draft/cursor state. Update shared policy applicability rather than changing oracle behavior. Add independent qualification that rejects universal menu-capture normalization.

## Validation limits

This review found material failures, so it does not certify exhaustive source-to-runtime closure. The986-site check proves bytes/locations, and taskfmt proves schema. Full production capture adapters, all135 expanded PTY journeys and all native tests remain later accepted-baseline work, not results fabricated here.

## Executed counterexamples and repairs

All four source mechanisms were replayed with actual pinned App/Screen/editor handlers: Ctrl+C exit, end-caret selection no-op, port insertion/fallback, and F10 paste passthrough. The disposable driver completed with exit0 at80×24,100×30,120×40,160×50. At each size it rejected the old false outcome and passed corrected predicates:51735173 versus explicit5173/empty/abc input, true-modal isolation versus F10 passthrough, unchanged clipboard after end-caret/no-copy or Ctrl+C, exact nonempty copied payload, hello positive match and zero-match next/previous handling, and quit=false throughout the live copy/find branch.

The copied payload is exactly `    1 <html>\n    2 <body>hello</body>\n    3 </html>`. It includes oracle line-number prefixes. Clipboard generation is1. The draft-insertion case renders `Port 5173` despite51735173 raw input. Source inspection additionally proves the raw emitted Snapshot kinds; the disposable driver did not add a private-kind extraction seam, so those exact-kind assertions remain a required production-adapter qualification rather than a claimed measured observer result.

Command: `cargo run --offline --manifest-path /tmp/holla-reaudit.5pIf5v/Cargo.toml --target-dir /private/tmp/tui-snap-audit.656lHG/oracle-clock/target`. Driver SHA-256: `f85fd68a02c8811d6cbc411e65364f98f53b572373ed45dfe8ccc42b02dca059`; manifest: `a56a4f33b1ba2825fc47059833a8ae2609fbd36418f74dbcd91eb4a031d17550`; driver executable: `8c74b0d3ea1bb6695e23c4ced924c006b2677506cf6291f883e1aabbfa1a4db7`. All88 included oracle Holla/library source files were compared byte-for-byte with git-show output before replay. No oracle source was patched.

A separate owned120×40 PTY launched the actual oracle Holla executable (`b1388f872b8ad9ab3a24da042653f94a4443ca512f0462a8ef67e58578b4b564`), reached the Files route, sent Ctrl+C and observed exit0, exact restored termios and alternate-screen leave. No input was submitted after exit. This proves the real lifecycle branch in addition to the direct preview-specific dispatcher observation; it does not claim complete per-size PTY replay of the new copy/find branches. An initial PTY marker assumption failed, and its exact disposable child was terminated before the corrected readiness marker succeeded.

Repairs are normative in `holla-trace-corrections.md`, the three scenario rows, their source-stage audit entries and HO-043-F02. All135 primary IDs and all23 stage-contribution IDs remain present. Original native routes are untouched. The shared editor owns the F10 compatibility operation; no UI redesign or app-local insertion is introduced. TASK040–050 copies and explicit read-before-edit requirements carry the contract; coordinator owns shared baseline/global matrix/traceability synchronization.

### Preserved rejected transcript evidence

- HO-PREVIEW-SELECT originally: `OPEN(Browse ~/work/site); select each actual text preview; focus preview; SWEEP; Shift+Right; Shift+Down; Ctrl+C; /; TYPE(site); Enter; Up; Esc`. Rejected claims: copied selection and live find after Ctrl+C.
- HO-ARGS originally: `NEW(rust-dirty,Paused,0); TYPE(port); Enter; click oracle args.field child(0) cell; PASTE(5173); Ctrl+S; fresh route invalid text; submit; Esc`. Rejected claims: accepted raw5173 and arbitrary invalid-text rejection. The insertion behavior is retained explicitly as args-insert-fallback.
- HO-OVERLAY-PASTE originally: `Fresh route per F1/Ctrl+G/Ctrl+Q/F10: NEW(rust-dirty,Paused,0); TYPE(port); Enter; start args edit; invoke that exact shell chord; PASTE(café); Esc; compare underlying draft`. Rejected claim: no underlying field mutation across every branch. All four source branches remain present, with separate true-modal and MenuBar expectations.

## Pre-repair file ledger

Path | Lines | SHA-256
--- | ---: | ---
refactoring-tasks/terminal-components/completion/040/AGENTS.md | 83 | 7f2b166c6de1561028ddcabaf896a990185fb78eb9d4b8f133155dda3994eef7
refactoring-tasks/terminal-components/completion/040/README.md | 192 | 83d1ddc1b0f0ec3181f9e6a66d2db7044fa71e7311cdcc3fc192777df792c901
refactoring-tasks/terminal-components/completion/040/task.toml | 3 | 68afebc919fe847c5860b2ce5e1daf3e9bbc07a3bc37d3a9e05b5219fec36677
refactoring-tasks/terminal-components/completion/040/trusted/contributions.tsv | 4 | 68773b007f7809b358894794f8078b616d75d9431ce0682f20b5f7fb948eb56c
refactoring-tasks/terminal-components/completion/040/trusted/frame-contributions.tsv | 2 | 5e8ead4db45618df90bfb0a72fcc35c7bb736da8f5f07cb09d72d236372b9128
refactoring-tasks/terminal-components/completion/040/trusted/holla-route-expansion.md | 55 | aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550
refactoring-tasks/terminal-components/completion/040/trusted/holla-stage-contract.md | 31 | 2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1
refactoring-tasks/terminal-components/completion/040/trusted/obligations.md | 79 | b449b232c5cf33bdf61058f125cef6953054adbdec8e4252562be7c6b0588dde
refactoring-tasks/terminal-components/completion/040/trusted/scenario-ids.tsv | 2 | 5e63bf249f308074ad748211334d876ea581fa02ba8e129bc5a32f98c6f7a6da
refactoring-tasks/terminal-components/completion/040/trusted/shell-contribution-contract.md | 38 | f6e61281935fbaa5123cc66136861bd301f5954f695c058c2857d4545f3dccc5
refactoring-tasks/terminal-components/completion/040/trusted/source-obligations.tsv | 5 | 28e724380c9d6ab9560f8633c0b124f41e52b1e1569633de2dc659017d5a55b1
refactoring-tasks/terminal-components/completion/040/verify.toml | 60 | 81de8f80eb292ebc8c6dbe32ac28dd2461b937770b2042e85280dc07d96f4574
refactoring-tasks/terminal-components/completion/041/AGENTS.md | 83 | 7cb8a2db530f445bba0ff30c2754e704324783e43c41dcc7083f1795c5e8bbc5
refactoring-tasks/terminal-components/completion/041/README.md | 197 | 9548c27eb99de9fe2e727b076c0670c73bd1658f63e4f03b16cfa39ad53501c4
refactoring-tasks/terminal-components/completion/041/task.toml | 3 | 3ff0ec42d82d20775eb0afc7e9a596935d0b95cbc686ab65a3c468f4a56269f8
refactoring-tasks/terminal-components/completion/041/trusted/holla-route-expansion.md | 55 | aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550
refactoring-tasks/terminal-components/completion/041/trusted/holla-stage-contract.md | 31 | 2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1
refactoring-tasks/terminal-components/completion/041/trusted/obligations.md | 242 | f890130a3c8a5c1669cedecdab3cc4cf2d0fac87870bea2d6de97cbfa79a3b01
refactoring-tasks/terminal-components/completion/041/trusted/scenario-ids.tsv | 16 | 29eb4075ba974bea457cd87715bd6b170fb650384bfe066e80c6c2812d6ff352
refactoring-tasks/terminal-components/completion/041/trusted/source-obligations.tsv | 6 | fa97b1eb78c0e065d7ab97749c880c059150cc6240da35c2dde57afefbff69b4
refactoring-tasks/terminal-components/completion/041/trusted/stage-contributions.tsv | 6 | 53229d28687eb9a558971cc53129621dc9eb630954583dd46cee69c9a2ffc0ef
refactoring-tasks/terminal-components/completion/041/verify.toml | 60 | be199800076ba3d5c358c1e5f00ea4b6f1f782a62f7b09ea4c81ad9c6653ba1f
refactoring-tasks/terminal-components/completion/042/AGENTS.md | 83 | 8ce032ff54d36072739a96ebf557ac9359c9b6a1f867dfdbb0a91709676064b7
refactoring-tasks/terminal-components/completion/042/README.md | 197 | 9e0932a826471f2f48785e610966515453d4aaf43e61b764374011f032ed9fd9
refactoring-tasks/terminal-components/completion/042/task.toml | 3 | ab5a202fc6caa28062a16ef7037d665702a373ed98bfd3848a23b0c305181c4f
refactoring-tasks/terminal-components/completion/042/trusted/holla-route-expansion.md | 55 | aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550
refactoring-tasks/terminal-components/completion/042/trusted/holla-stage-contract.md | 31 | 2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1
refactoring-tasks/terminal-components/completion/042/trusted/obligations.md | 143 | a42ef826b2d942f0863a477702ea836ab493981e138a76e920c278229731818f
refactoring-tasks/terminal-components/completion/042/trusted/scenario-ids.tsv | 8 | 44f5d8517ea1bc3926a7bcdcdb6af23392cdd5b7928e5a5c748db02de0d85de6
refactoring-tasks/terminal-components/completion/042/trusted/source-obligations.tsv | 6 | f52f211d385af2980410e970a737b76e1d88357338ed0cd74ca1d20b75d25bbc
refactoring-tasks/terminal-components/completion/042/trusted/stage-contributions.tsv | 4 | 7ad13991d6033f4cd68dc2feb2b8c538a793a76095a9fa62bb101eea1e21dcfd
refactoring-tasks/terminal-components/completion/042/verify.toml | 60 | c1fe7753ce5d3c35fc8918b141cbb3959915925a536b552c17d293c941281a87
refactoring-tasks/terminal-components/completion/043/AGENTS.md | 83 | 49253a75384c8812650e1affe8c4391010c79a2a2636f9245f3490d401f63dc3
refactoring-tasks/terminal-components/completion/043/README.md | 197 | b08fd58a10d5eed44a5597f2cd4bd36b3e9963e3f49ddc24142d5f4875ea7cf8
refactoring-tasks/terminal-components/completion/043/task.toml | 3 | 353e1956e111ca3f230c454528839ad78d5a29c723d1de10b19aa63eeda54d32
refactoring-tasks/terminal-components/completion/043/trusted/holla-route-expansion.md | 55 | aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550
refactoring-tasks/terminal-components/completion/043/trusted/holla-stage-contract.md | 31 | 2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1
refactoring-tasks/terminal-components/completion/043/trusted/obligations.md | 132 | 89bb0b1e053a8626e919846638d88ea500b107670de4ed1a0ea2fb1ea788c620
refactoring-tasks/terminal-components/completion/043/trusted/scenario-ids.tsv | 6 | 3a5681b93b4b73fc58582d59016ca16f96a719fc35c01e2e7bd8aa3053a4695b
refactoring-tasks/terminal-components/completion/043/trusted/source-obligations.tsv | 7 | bb1751f37a8e812e43cf9507a130bc5564f156ebc2d171f84982c47d24ad6db1
refactoring-tasks/terminal-components/completion/043/trusted/stage-contributions.tsv | 6 | 535a5b6f04b3261286b2b5ee6f6a784283a87710def91b9cfc2c30ab3b376531
refactoring-tasks/terminal-components/completion/043/verify.toml | 60 | f7abef912d70dee536386e5e79133bbcadaa758979eddc55b268e2dd9ddc151c
refactoring-tasks/terminal-components/completion/044/AGENTS.md | 83 | 42a16d8790878afe9ea45520140b25fe87b974f9470b11806184205ccea2afd0
refactoring-tasks/terminal-components/completion/044/README.md | 199 | 38f0d7f89bb780571f0b5d9341eafa4c42082a476929523dd9ea050c4425ee96
refactoring-tasks/terminal-components/completion/044/task.toml | 3 | f704541d7998e261e2db3bb401169b9a160053fca46846a9209ad45a741bebc6
refactoring-tasks/terminal-components/completion/044/trusted/holla-route-expansion.md | 55 | aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550
refactoring-tasks/terminal-components/completion/044/trusted/holla-stage-contract.md | 31 | 2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1
refactoring-tasks/terminal-components/completion/044/trusted/obligations.md | 308 | de242570850797fd35f3da1cfe048e7a60e2378e3fd1da16b82bf870394e6d64
refactoring-tasks/terminal-components/completion/044/trusted/scenario-ids.tsv | 21 | 65ac842212620ee45e230e01ad98d1ebb448085f6fadccde96e653ce303d6da8
refactoring-tasks/terminal-components/completion/044/trusted/source-obligations.tsv | 10 | 257bf9b55c2f8506621a12124c086f15f43bf73ddde3a148f6dc6f6966599430
refactoring-tasks/terminal-components/completion/044/trusted/stage-contributions.tsv | 7 | cbe385ed18d5460e4bdc114dbcbbd885d0b00431dbcb8a0fe8e175eea48d5cc5
refactoring-tasks/terminal-components/completion/044/verify.toml | 60 | 8a8a4a30fa3f54c695b44a919031b8227952c70596573dfb1f44ec9c581c22c4
refactoring-tasks/terminal-components/completion/045/AGENTS.md | 83 | 74ba89066f093a578d959bf316735beb0a73ec999b8e94f97466a73fd8b678a0
refactoring-tasks/terminal-components/completion/045/README.md | 199 | 54f4a9217d42d607b2e0090b2c88b3ca4926c65093702f8cd4346329969bbd93
refactoring-tasks/terminal-components/completion/045/task.toml | 3 | 03fda1099815b2422abdd38a79ffc5d82a91766e3dc52b991ea6f6711f0b3982
refactoring-tasks/terminal-components/completion/045/trusted/holla-route-expansion.md | 55 | aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550
refactoring-tasks/terminal-components/completion/045/trusted/holla-stage-contract.md | 31 | 2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1
refactoring-tasks/terminal-components/completion/045/trusted/obligations.md | 231 | db6caaa2f392e8408449e588075064df9dc20d8288f6ef5282f640d8b99da897
refactoring-tasks/terminal-components/completion/045/trusted/scenario-ids.tsv | 16 | e79acbaea4a5d8dfdce129963e1ff21cbbc651cf525820523d1c430cf76ffc4f
refactoring-tasks/terminal-components/completion/045/trusted/source-obligations.tsv | 7 | 94ff58ac3a90606c4920c4e89a98adec457b46aedf20b14c618a21fb37ca8c93
refactoring-tasks/terminal-components/completion/045/trusted/stage-contributions.tsv | 5 | 9a70562b154a0057ea91996ed31e5aff6fb20fc888e4011a953119bc387cef0d
refactoring-tasks/terminal-components/completion/045/verify.toml | 60 | b0334f215911f946778177f034623f9cbc9d051ee16fdd3995027032b10cd5c3
refactoring-tasks/terminal-components/completion/046/AGENTS.md | 83 | 4deead5f59f09ab8f26cf480f4ef4006c8cdabd049217306b03e3f86ca25601c
refactoring-tasks/terminal-components/completion/046/README.md | 182 | a7fa977298277111fb27d276c4ca228a2ee598750817d3842d3bf1f2ace5eca2
refactoring-tasks/terminal-components/completion/046/task.toml | 3 | 4990793e210c3319a31967fb5d65cf5d8266adb75aa10351ed3386acddcbae54
refactoring-tasks/terminal-components/completion/046/trusted/holla-route-expansion.md | 55 | aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550
refactoring-tasks/terminal-components/completion/046/trusted/holla-stage-contract.md | 31 | 2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1
refactoring-tasks/terminal-components/completion/046/trusted/obligations.md | 200 | ca73d2c4cbc9453c105ae2786d556bc1a51ac4cd01fafec0f044bd98dbf1d9c8
refactoring-tasks/terminal-components/completion/046/trusted/scenario-ids.tsv | 16 | c33f2fd03e35589a415c5ad9b54f97997ce8ee182292276c654c7220e5d835a6
refactoring-tasks/terminal-components/completion/046/trusted/source-obligations.tsv | 10 | b2adebf2d1b73a4609bfbb32a9fb2ea178aaf12223606a5635e0e1d2016f4e7a
refactoring-tasks/terminal-components/completion/046/trusted/stage-contributions.tsv | 1 | 16bceed2c40e7d076a20b41e995037215e062dc8864b5764c4315d7d12b4e140
refactoring-tasks/terminal-components/completion/046/verify.toml | 60 | d7dc711fba94fad53b7853274b679ece4b3369320c5e4b23edc0e3b712580c87
refactoring-tasks/terminal-components/completion/047/AGENTS.md | 83 | beb7c1e3dd898b5ba63704f457dcda014db0db6d3eafd0874adc16d521f33b49
refactoring-tasks/terminal-components/completion/047/README.md | 184 | 97ed0b557cc509902c20980a8877b69be5c7e5e0406b6c23efbe254457b18c31
refactoring-tasks/terminal-components/completion/047/task.toml | 3 | 3e1c536fc52baccdd959be66af0eddd7e95a1b60c976c434d8187f2986db64eb
refactoring-tasks/terminal-components/completion/047/trusted/holla-route-expansion.md | 55 | aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550
refactoring-tasks/terminal-components/completion/047/trusted/holla-stage-contract.md | 31 | 2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1
refactoring-tasks/terminal-components/completion/047/trusted/obligations.md | 222 | 83293ff5faf74d5058c95dc161d99b084e82e54869693a3d9797d25ef9c351d7
refactoring-tasks/terminal-components/completion/047/trusted/scenario-ids.tsv | 18 | 657338f1abcc6a897dc9036bdea0cb7269559b43ebb7be84d49fb00f9c357091
refactoring-tasks/terminal-components/completion/047/trusted/source-obligations.tsv | 14 | 2c02650d1724ce337dff43d9516a795a8ea4d7cf365ec9a0f5bd54b62e4551b6
refactoring-tasks/terminal-components/completion/047/trusted/stage-contributions.tsv | 1 | 16bceed2c40e7d076a20b41e995037215e062dc8864b5764c4315d7d12b4e140
refactoring-tasks/terminal-components/completion/047/verify.toml | 60 | 0a401afb9194b89c1464ec4071db1f05b24352a0e7b445fa9568105facc7beb9
refactoring-tasks/terminal-components/completion/048/AGENTS.md | 83 | c1d31efcc066da8009d45631ad9ac3f384a3e6ac4ec99031765872c55f53889c
refactoring-tasks/terminal-components/completion/048/README.md | 182 | b3163ab9d21212eedbbd7c360c872a0b2ebc2668ed2d0a76551d5e1f69c504a5
refactoring-tasks/terminal-components/completion/048/task.toml | 3 | 8746428956c9ba752b004490dfa222aaf9de05a8e4f2df5351fef7634cfd5fa7
refactoring-tasks/terminal-components/completion/048/trusted/holla-route-expansion.md | 55 | aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550
refactoring-tasks/terminal-components/completion/048/trusted/holla-stage-contract.md | 31 | 2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1
refactoring-tasks/terminal-components/completion/048/trusted/obligations.md | 222 | 579c1d2910ac688d442c2e81fb2671726af0af6161f57fe47414f3a0cdbe579b
refactoring-tasks/terminal-components/completion/048/trusted/scenario-ids.tsv | 18 | 71f4b08577c67f4c6f9f112d34fdec0b2cfb3c545c407996a544772e4f17502c
refactoring-tasks/terminal-components/completion/048/trusted/source-obligations.tsv | 9 | 71cd6ee9a9938e0796b439b01afc668b797f4c04869b82f450eba0754b842e67
refactoring-tasks/terminal-components/completion/048/trusted/stage-contributions.tsv | 1 | 16bceed2c40e7d076a20b41e995037215e062dc8864b5764c4315d7d12b4e140
refactoring-tasks/terminal-components/completion/048/verify.toml | 60 | 998d005eac36a736e943a590c087d46cf738681c73ab709b9313bf2922e8cff4
refactoring-tasks/terminal-components/completion/049/AGENTS.md | 83 | 6eec59d70ee7d1e4929d3f284474d368ac78daf7ebcbb26fb8cc63ba7ceb0c73
refactoring-tasks/terminal-components/completion/049/README.md | 182 | a185698fd9581b3b78d559e7730af925b2cd1a1f21d06cafdbbbc44e60d213b4
refactoring-tasks/terminal-components/completion/049/task.toml | 3 | 21de367a4812ae29490d15f84e87ed2214dd7c72f1109b2afee4c68a08ad6b3e
refactoring-tasks/terminal-components/completion/049/trusted/holla-route-expansion.md | 55 | aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550
refactoring-tasks/terminal-components/completion/049/trusted/holla-stage-contract.md | 31 | 2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1
refactoring-tasks/terminal-components/completion/049/trusted/obligations.md | 189 | 36ec81ef54c97017d0f5e615231d2ab9039cb83a0d7c809a808131a93999b659
refactoring-tasks/terminal-components/completion/049/trusted/scenario-ids.tsv | 15 | c7b9f61731cf189ea0cd56f50f0398079a60bb3ee6264696b74158f5e871fa12
refactoring-tasks/terminal-components/completion/049/trusted/source-obligations.tsv | 8 | 9b57305897d9170723b428175623967190f98d1e2d78543347ffb179ab297c5b
refactoring-tasks/terminal-components/completion/049/trusted/stage-contributions.tsv | 1 | 16bceed2c40e7d076a20b41e995037215e062dc8864b5764c4315d7d12b4e140
refactoring-tasks/terminal-components/completion/049/verify.toml | 60 | 9f1300fc7fad5780cb3245066c27d1ec3b3bc58a11cddccdaf111309cf45c06b
refactoring-tasks/terminal-components/completion/050/AGENTS.md | 83 | 49eb6d8978e2e4c774b52fe6863865cccd70d42885da60a28ad4ab9ff8f3eee5
refactoring-tasks/terminal-components/completion/050/README.md | 177 | 5e20cf02fb5f988f7beb3c45796bcad651e2df2ff33d08c3383101965ab43203
refactoring-tasks/terminal-components/completion/050/task.toml | 3 | 8c77fbdff7aa0c10112d4ebedf3828c4419410052eec12c634ac5e89c1f41811
refactoring-tasks/terminal-components/completion/050/trusted/holla-route-expansion.md | 55 | aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550
refactoring-tasks/terminal-components/completion/050/trusted/holla-route-sites.tsv | 987 | eb3c6d66bffd441fd61964b41e6c08f0c2db2ba33ec02f0184bde43834aba045
refactoring-tasks/terminal-components/completion/050/trusted/holla-stage-audit.tsv | 136 | f5c33a9579d6a4b3062c98cd47a45f1ceb63008f6b79cb1370e3bafb0980cf5e
refactoring-tasks/terminal-components/completion/050/trusted/holla-stage-contract.md | 31 | 2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1
refactoring-tasks/terminal-components/completion/050/trusted/obligations.md | 1512 | 05a5cadc20b235c91677811a5a7961866af5673c6a0d96b6883036895b75f6dc
refactoring-tasks/terminal-components/completion/050/trusted/scenario-ids.tsv | 136 | 3545e1e6a5c4a7a163016dc58882cf0348e5876e14890b273a946b926c15f2e7
refactoring-tasks/terminal-components/completion/050/trusted/source-obligations.tsv | 5 | b3afc3e274b0cd8cc6f4373609e38f9c06e064a1edbc7091a7d76c08942e6d7c
refactoring-tasks/terminal-components/completion/050/trusted/stage-contributions.tsv | 24 | 700107ac6f3548c9c5765350f500241d4b6b128c483ac8421a1aff9f803b5a80
refactoring-tasks/terminal-components/completion/050/verify.toml | 60 | c1ca8d9bd145e8c92f64702d6f71524a4806e7201ba80d36bad3b5a36136081e

## Reproduction driver

The following exact temporary sources reproduce the measured direct checks. Resolve the immutable oracle into a disposable checkout and substitute its location for the absolute oracle-clock prefix when recreating this diagnostic elsewhere. Do not alter any oracle source. This is a real-App driver; it supplies inputs and asserts resulting state, and does not replace production handlers or rendering.

Cargo.toml:

```toml
[package]
name = "holla-reaudit"
version = "0.0.0"
edition = "2024"

[dependencies]
junie_tui = { package = "junie-tui", path = "/private/tmp/tui-snap-audit.656lHG/oracle-clock" }
ratatui = { version = "0.30", features = ["crossterm_0_29"] }
unicode-width = "0.2"
unicode-segmentation = "1"
```

src/main.rs:

```rust
#![allow(dead_code, unused_imports)]
#[path = "/private/tmp/tui-snap-audit.656lHG/oracle-clock/src/bin/holla/app.rs"] mod app;
#[path = "/private/tmp/tui-snap-audit.656lHG/oracle-clock/src/bin/holla/app_tests.rs"] mod app_tests;
#[path = "/private/tmp/tui-snap-audit.656lHG/oracle-clock/src/bin/holla/clock.rs"] mod clock;
#[path = "/private/tmp/tui-snap-audit.656lHG/oracle-clock/src/bin/holla/domain/mod.rs"] mod domain;
#[path = "/private/tmp/tui-snap-audit.656lHG/oracle-clock/src/bin/holla/scenario.rs"] mod scenario;
#[path = "/private/tmp/tui-snap-audit.656lHG/oracle-clock/src/bin/holla/screens/mod.rs"] mod screens;
#[path = "/private/tmp/tui-snap-audit.656lHG/oracle-clock/src/bin/holla/sim/mod.rs"] mod sim;
use app_tests::H;
use scenario::{Motion,Scenario};
use ratatui::crossterm::event::{KeyCode,KeyModifiers};
use junie_tui::core::event::{Input,Key};

fn paste(h:&mut H,s:&str) { h.app.handle(Input::Paste(s.into())); h.draw(); }
fn port(w:u16,h:u16)->H {
 let mut h=H::new(Scenario::RustDirty,Motion::Paused,0,w,h);
 h.type_str("port"); h.key(KeyCode::Enter);
 let a=h.app.hits.area_of(screens::review::ARG.child(0)).expect("port field");
 h.click(a.x+2,a.y); assert!(h.text().contains("EDIT")); h
}
fn browser(w:u16,h:u16,select:bool)->H {
 let mut h=H::new(Scenario::ParityBrowser,Motion::Reduced,0,w,h); h.ticks(4);
 h.type_str("Browse ~/work/site");h.key(KeyCode::Enter);
 // Follow the real list to the source-provided index.html entry.
 for _ in 0..20 {
  if h.text().contains("index.html") {
   if let Some((_,y))=h.find("index.html") {let a=h.app.hits.area_of(screens::files::LIST).unwrap();h.click(a.x+2,y);h.ticks(1);break;}
  }
  h.key(KeyCode::Down);
 }
 h.key(KeyCode::Right);
 if select {h.app.handle(Input::Key(Key{code:KeyCode::Home,mods:KeyModifiers::CONTROL|KeyModifiers::SHIFT}));h.draw();}h
}
fn main() {
 for (w,hh) in [(80,24),(100,30),(120,40),(160,50)] {
  let mut h=port(w,hh);paste(&mut h,"5173");
  assert!(h.text().contains("51735173"),"{}",h.text());
  h.ctrl(KeyCode::Char('s'));assert!(h.text().contains("Port 5173"));
  println!("{w}x{hh} old port: draft=51735173, visible=Port5173; valid-value claim REJECTED");
  let mut h=port(w,hh);h.ctrl(KeyCode::Char('l'));paste(&mut h,"5173");
  assert!(!h.text().contains("51735173"));h.ctrl(KeyCode::Char('s'));assert!(h.text().contains("Port 5173"));
  let mut h=port(w,hh);h.ctrl(KeyCode::Char('l'));h.key(KeyCode::Backspace);h.ctrl(KeyCode::Char('s'));
  assert!(h.text().contains("Required"));assert!(h.text().contains("arguments"));
  let mut h=port(w,hh);h.ctrl(KeyCode::Char('l'));paste(&mut h,"abc");h.ctrl(KeyCode::Char('s'));assert!(h.text().contains("Port 5173"));
  println!("{w}x{hh} new port: explicit replacement, empty-required rejection, nonnumeric fallback PASS");
  for (key,ctrl,leaks) in [(KeyCode::F(1),false,false),(KeyCode::Char('g'),true,false),(KeyCode::Char('q'),true,false),(KeyCode::F(10),false,true)] {
   let mut h=port(w,hh);if ctrl {h.ctrl(key);}else{h.key(key);}paste(&mut h,"café");h.key(KeyCode::Esc);
   assert_eq!(h.text().contains("café"),leaks,"key={key:?} {}",h.text());
  }
  println!("{w}x{hh} old universal paste-block claim REJECTED; new modal isolation/menu passthrough PASS");
  let mut h=browser(w,hh,false);h.ctrl(KeyCode::Char('c'));assert!(h.app.quit);assert!(h.app.world.clipboard.is_none());
  println!("{w}x{hh} old preview Ctrl+C: quit=true; continued-process claim REJECTED");
  let mut h=browser(w,hh,false);h.app.handle(Input::Key(Key{code:KeyCode::Right,mods:KeyModifiers::SHIFT}));h.draw();h.app.handle(Input::Key(Key{code:KeyCode::Down,mods:KeyModifiers::SHIFT}));h.draw();h.key(KeyCode::Char('y'));assert!(h.app.world.clipboard.is_none());
  println!("{w}x{hh} old end-caret selection claim REJECTED; actual no-copy PASS");
  let mut h=browser(w,hh,true);h.key(KeyCode::Char('y'));assert!(!h.app.quit);assert_eq!(h.app.world.clipboard.as_deref(),Some("    1 <html>\n    2 <body>hello</body>\n    3 </html>"));assert_eq!(h.app.clipboard_gen,1);
  h.key(KeyCode::Char('/'));h.type_str("hello");assert!(h.text().contains("1 of 1"));h.key(KeyCode::Enter);h.key(KeyCode::Down);h.key(KeyCode::Up);
  for _ in 0..5 {h.key(KeyCode::Backspace);}h.type_str("zzzz-no-preview-match");assert!(h.text().contains("0 of 0"));h.key(KeyCode::Enter);h.key(KeyCode::Down);h.key(KeyCode::Up);h.key(KeyCode::Esc);assert!(!h.app.quit);
  println!("{w}x{hh} new preview y copy/find: process remains alive PASS");
 }
}
```
