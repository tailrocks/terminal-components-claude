# T01 component and facade inventory

## Artifact status

`fragment.json` is the component and facade census for T01 (SHA-256 `5c66d4b053b502824d246c61e29c950ded8e782b44ea1e8c77dfe38fb5c2b119`; correction of artifact commit `36d7bc8b9c4235cdc42df5e7ab6e656e2a6a0e4f`), using source commit `542349e2f70ec9883d743594f906ac6c4e96ba5a` (tree `66507c4aae0555d65904b268d6c95e7f3e3d17d1`). The checkout is `/Users/donbeave/Downloads/termrock-t01-components`, branch `pr38/T01-components`. This directory is the only owned output. The fragment preserves W01–W45 and appends W46–W63; it does not edit the ownership matrix, source, configuration, tests, or other T01 fragments.

The fragment is pending independent Sol high review. It is a source inventory and test proposal only: 82 proposed case records, zero selected behavior cases, and product behavior `NOT_VERIFIED`. It does not approve T38 or later test plans. T38 has not started.

## Inventory scope and counts

The fragment inventories public declarations, public `pub use` and `pub mod` statements, source-level expanded paths, facade family globs, aliases/adapters, component APIs, defaults, generic trait implementation contracts, extension points, and current consumers for the controls, fields, feedback, navigation, viewport, overlays, grid, editors, forms, and `termrock` facade families. It carries source-bound `ScrollRegion`, `TextReport`, and `AppShell` component definitions and separately references the foundation index. The `a893` foundation artifact was rejected by independent Sol high review; a correction is active, so that cross-fragment index is not accepted. Support APIs receive qualified API IDs; WIDs identify reusable component concepts only.

The source ledger records 186 source records, including 169 current source paths. The joined item census has 2,433 declarations in its indexed package scope; it does not yet include W44’s 37 terminal-view declarations or its derive-generated `TerminalViewState: Default` contract, which remain explicit blockers. It also records 267 `pub use` statements and 118 public module declarations (385 source export rows), plus 1,468 expanded source-level records. That record count contains 1,436 named symbols and 32 wildcard edge placeholders; 1,370 named paths plus 21 wildcard labels yield 1,391 unique path labels. Externally reachable paths include 1,461 expanded entries / 1,384 unique paths. The separate component crosswalk contains 260 reexport rows and 321 canonical-plus-reexport path rows for W01–W63 after adding source-confirmed TerminalView crate/facade paths and the runtime-root ScrollRegion reexport. There are 1,094 component API IDs, 239 public trait implementation contracts, 30 extension points, 19 facade glob edges, and 4,394 facade-family API path candidates. All 4,394 family paths now have a canonical symbol or namespace owner. Methods stay attached to qualified receivers or traits; they are not counted as module exports.

The corrected census supersedes two intermediate measurements: 2,157 path-like rows from an earlier scanner pass and 382 export rows before three inline public module declarations were restored. The author facade has 173 direct glob symbols; its 178 expansion records also preserve five trait-implementation observations. These units remain separate in `fragment.json`.

Each component records its canonical definition owner and line/hash, named task, API IDs where joined, state/action/builder/default references, public reexport chains, public paths, required case IDs, and `NOT_VERIFIED` status. W44 now records the terminal crate-root and facade-module paths. Its 37 public declaration locators and one derive-generated `Default` contract are not yet joined to `items[]`/trait contracts, so `T01-OPEN-TERMINALVIEW-API-JOIN` blocks closure; no API IDs are fabricated. Each facade glob carries the exact edge and source manifest SHA/features. Private source modules remain source records but do not become external public paths. The nested core keyboard/session paths resolve through `termrock_core::keys`, including the `keys::*` alias and `termrock::core::*` facade path.

## New WID allocation

W46–W63 are append-only after W45. Their order is fixed in `fragment.json` with a per-ID reason. Similar names receive separate WIDs when their canonical definitions differ. Existing WIDs are unchanged.

| WID | Canonical definition | Owner task | Allocation reason |
|---|---|---:|---|
| W46 | `termrock_controls::master_detail::MasterDetail` — `crates/termrock-controls/src/master_detail.rs:129` | T12 | Controls component, separate from facade block adapter |
| W47 | `termrock_controls::radio_field::RadioField` — `crates/termrock-controls/src/radio_field.rs:13` | T12 | Reusable field adapter required by the inventory |
| W48 | `termrock::blocks::split::SplitBlock` — `crates/termrock/src/blocks/split.rs:20` | T10 | Reusable facade composition block |
| W49 | `termrock::blocks::master_detail::MasterDetail` — `crates/termrock/src/blocks/master_detail.rs:169` | T10 | Separate canonical facade definition from W46 |
| W50 | `termrock_grid::table::Table` — `crates/termrock-grid/src/table.rs:122` | T16 | Reusable grid component |
| W51 | `termrock_grid::grid_viewport::GridViewport` — `crates/termrock-grid/src/grid_viewport.rs:22` | T16 | Reusable grid viewport component |
| W52 | `termrock_forms::form_dialog::FormDialog` — `crates/termrock-forms/src/form_dialog.rs:91` | T18 | Reusable form adapter required by the inventory |
| W53 | `termrock_forms::form_layout::FormLayout` — `crates/termrock-forms/src/form_layout.rs:68` | T18 | Reusable form layout component |
| W54 | `termrock_overlays::select_field::SelectField` — `crates/termrock-overlays/src/select_field.rs:13` | T15 | Reusable overlay field adapter |
| W55 | `termrock_overlays::notice_reveal::NoticeReveal` — `crates/termrock-overlays/src/notice_reveal.rs:29` | T15 | Reusable overlay reveal component |
| W56 | `termrock_editors::completion::CompletingCodeEditor` — `crates/termrock-editors/src/completion.rs:216` | T17 | Reusable editor completion adapter |
| W57 | `termrock_fields::note::Note` — `crates/termrock-fields/src/note.rs:13` | T13 | Reusable field primitive; separate from consumer-example `Note` |
| W58 | `termrock_feedback::notice::Notice` — `crates/termrock-feedback/src/notice.rs:120` | T12 | Reusable feedback component |
| W59 | `termrock_feedback::section::TextSection` — `crates/termrock-feedback/src/section.rs:9` | T12 | Reusable feedback content primitive |
| W60 | `termrock_feedback::hintbar::DerivedHintBar` — `crates/termrock-feedback/src/hintbar.rs:680` | T12 | Reusable derived hint component |
| W61 | `termrock_shell::report::TextReport` — `crates/termrock-shell/src/report.rs:68` | T19 | Reusable report component; support types do not receive WIDs |
| W62 | `termrock_shell::shell::AppShell` — `crates/termrock-shell/src/shell.rs:26` | T10 | Reusable shell definition; facade names are aliases. T10 owns `shell.rs`; T19 owns shell policy integration. |
| W63 | `termrock_fields::field_error::FieldErrorChrome` — `crates/termrock-fields/src/field_error.rs:10` | T13 | Additional reusable public field component found in current source |

## Cross-fragment inputs and remaining blockers

`provisional_foundation_index` points to `/Users/donbeave/Downloads/termrock-t01-foundation/docs/implementation/pr38-final/results/t01-foundation-api/foundation-api.fragment.json`, commit `a893ad5a46934ed090220b04403dd529d59f64e5`, captured SHA-256 `3ba2da120c3053527e76f606e13da134f5f9b04f929702a9b4675e70a8c1138e`. Independent Sol high review `/root/t01_inventory_review` (gpt-6.1-sol, high) returned `CORRECTION`; no formal review ID was assigned. A new foundation correction is active. The captured 19-package crosswalk remains rejected provenance, not accepted joins.

`external_app_route_fragment` pins the app-route artifact at `/Users/donbeave/Downloads/termrock-t01-apps/docs/implementation/pr38-final/results/t01-app-routes/fragment.json`, commit `7f6fa8e5e2643f6c9de0b0a6ec9752f800cbcfc7`, SHA-256 `c63bc59a4d2312aceec26483a49eecbd854635fe559f5dc811cdfa1672b9ad0c`. Its independent route review accepted the scoped inventory at commit `7f6fa8e5e2643f6c9de0b0a6ec9752f800cbcfc7`; behavior remains `NOT VERIFIED`. The component fragment crosswalk records 304 route-case/component links, 199 deduplicated route/WID mappings across 54 routes (W62 AppShell appears on all 54), and 25 app/component links across four applications for 14 new WIDs. It groups 211 route source rows into 199 route/WID mappings and retains all 12 duplicate rows as evidence. The current joins report zero definition path/line/hash, definition-owner-task, or API-case-ID mismatches. Fifty-one current application consumer occurrences are bound to source file hashes and lines. These are source and proposal joins; route and app behavior remains `NOT VERIFIED`.

Twelve explicit open requirements remain blockers. The current source lacks a public `InspectorBody` declaration/export, recorded as `T01-OPEN-APPAPI-INSPECTORBODY-EXPORT`; final inventory must wait for its migration row to be updated. `T01-OPEN-EXPORT-OWNER-JOIN` lists 83 exact export symbol paths whose importing statements are present but whose canonical project/external definition owner is not yet joined; final cross-fragment integration must resolve those rows against current declarations, the corrected foundation index, or declared external dependencies. Other open requirements identify concrete current-source gaps such as the W44 API-item join, Notice all-layer behavior, editor cancellation, shell key/footer policy, session feature mapping, application API ownership, Table sort owner, feature reconciliation, and unrecoverable T33 samples. The W44 terminal-view API join is a twelfth explicit blocker, with 37 source-located declarations awaiting item/API-ID/signature joins. The fragment reconstructs 41 T33 historical workloads; 40 report names and 200 sample rows remain missing. Historical source and removed test documentation are kept separate from current source.

Nine retained architecture/application migration items have explicit dispositions. `TextViewport::selection_binding_chords` carries the authorized T37 `RETIRE` decision (`T37-SELECTION-BINDING-01`); `Runtime::draw_scene` compatibility is `KEEP` under `T37-BUILD-COMPAT-01`. These decisions do not establish product behavior.

## Commands and results

Commands ran from `/Users/donbeave/Downloads/termrock-t01-components` against the fixed source checkout. No Cargo command, test, Nextest, old test/helper, or behavior case ran.

| Command | Result |
|---|---|
| `git rev-parse HEAD` | `36d7bc8b9c4235cdc42df5e7ab6e656e2a6a0e4f` before this correction; fixed product source remains `542349e2f70ec9883d743594f906ac6c4e96ba5a` |
| `git rev-parse HEAD^{tree}` | `edd275b38ca7a0ec98613a71c0b7530d99728c53` before this correction; fixed product input tree is `66507c4aae0555d65904b268d6c95e7f3e3d17d1` |
| `python3 -m json.tool docs/implementation/pr38-final/results/t01-components-api/fragment.json >/dev/null` | Passed JSON parse |
| `python3 /tmp/t01_validate_corrected.py` | Passed with 0 issues: 169 current source paths, 63 WIDs, 82 requirements/cases, 260 reexports, 321 public paths, 385 export rows, 83 explicit pending export-owner joins, 304 route case links, 199 route/WID mappings, and 25 app/component links. The validator is outside the repository and checks document/source structure only. |
| `python3 /tmp/t01_components_correction_check.py` | Passed with 0 focused correction issues; checked W44/W45, callback joins, 13 unexpanded globs, trait receivers, PropsList tasks, review states, and reconciled counts. |
| Package/facade manifest structural check (`python3 -`) | Passed: 10 package manifests and 19 facade glob source/manifest maps match hashes, package names/features, and source lines. |
| `git show a893ad5a46934ed090220b04403dd529d59f64e5:docs/implementation/pr38-final/results/t01-foundation-api/foundation-api.fragment.json | shasum -a 256` | `3ba2da120c3053527e76f606e13da134f5f9b04f929702a9b4675e70a8c1138e`; verifies the exact rejected foundation artifact as provenance only. |
| App-route cross-fragment check (Python structural check) | Passed: exact route fragment SHA/commit; 304 component-case links; 199 grouped route/WID joins; 54-route W62 coverage; app consumer SHA/line bindings; zero definition, owner, or API-case-ID mismatches. The route inventory has independent scoped acceptance; route behavior remains `NOT VERIFIED`. |
| `git diff --cached --check` | Passed; no whitespace errors in the two owned T01 documentation files. |
| Cargo/tests/Nextest | Not run by T01; product behavior remains `NOT_VERIFIED`. |

The structural checks are not product tests or public API compile checks. The source item inventory still excludes 37 public terminal-view declarations and the derived `TerminalViewState: Default` contract; their exact source locators are retained on W44 and block T01 closure until an API-ID/signature join is supplied. Source scanning does not replace rustc visibility resolution or all-feature builds. The app-route fragment is committed at the path, commit, and SHA above; its independent scoped review is accepted. Component inventory review remains pending. The implementation worker was `/root/t01_components_api`, assigned `gpt-6-luna` at max effort; no nested agents were used. Final T01 integration must retain all 4,511 historical coverage rows under the history worker’s classification and complete independent review before any test plan is approved.
