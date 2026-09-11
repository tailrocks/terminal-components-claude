# Whole-branch generated-artifact disposition

## Exact scope and method

The artifact partition contains 7,019 changed paths between Holla `2e2401393c47360741ebd321679de08982dca50a` and main `7b27732a8c3c131760ec3438f641cb3c11343a42`. All are endpoint additions/deletions. The census reads every Git blob, recomputes its object identity, records SHA-256, parses every text/JSON/TSV/HTML/cursor artifact and verifies/decodes every PNG. This covers 5,399 unique blobs and 218,438,713 endpoint bytes, with no decode errors.

There are 1,214 complete five-format frame sets: PNG, ANSI, text, HTML and cursor. Holla supplies 603; main supplies 611. Additional paths are three Markdown files, three TSVs, 219 logs, one lock, 112 guards, 605 JSON records/sidecars and six loose fade/scroll text files. Every path has exact side/blob/hash and format observations in `branch-artifact-records.jsonl`; aggregate counts are in `branch-artifact-census.json`.

This is full byte/format/relationship inspection, not a claim that a person visually reviewed every raster or manually read every repeated frame cell. No historical recipe was executed, no image was regenerated, no live VT replay occurred, and no frame received oracle acceptance. The three Markdown bodies were read; generated table/recipe fields were parsed and checked across every record rather than treating a truncated terminal listing as a full read.

## Complete relationship checks

`branch-artifact-relationships.json` records all checks and all differences, including harmless serialization differences. Exact results:

- All 1,214 frame sets have their four companion formats and valid cursor triples.
- All 302 Holla manifest ANSI hashes match their archived bytes. All 302 PNG fidelity dimensions match the recorded cell grid plus the renderer's explicit 12-pixel padding per edge. Sidecars report one approximate capture and one missing glyph, zero unshaped/clipped/control entries. These are historical sidecar claims, not fresh font coverage measurements.
- All 499 `baseline/before/manifest.tsv` IDs are unique and equal the complete `parity/recipes.tsv` ID set. Every recipe preserves its source viewport, historical command and ordered steps; all current argv values are nonempty string arrays; all 2,495 expected artifact paths exist at the pinned endpoint. This proves archival relationships, not that current argv successfully replays the old recipe.
- All 112 capture-matrix hashes/byte counts match PNGs. All 560 artifact hashes/byte counts in main's capture-provenance manifest match their archived files. Matrix and provenance PNG membership is identical.
- Exact SGR-stripped ANSI/text hashes differ in 617 sets; exact HTML-pre/text hashes differ in all 1,214. A separately labeled diagnostic removing line-ending and right-padding differences reduces both comparisons to the same 17 sets. Padding normalization is never authorized for actual cell-parity acceptance.

## BA-ART-01: one capture set can contain different time states

All 17 remaining ANSI/text differences were inspected through exact Git text and unified diffs in `branch-artifact-text-differences.json`. Ten are main's historical Showcase Progress/Scrolling/Terminal sets; seven are Holla's scrolling/discovery/log sets. Differences include spinner phases, progress glyph positions, elapsed time and an entire log window advancing by one record. For example, main `showcase_terminal_default_120x40` has `0.6 s` in ANSI versus `0.7 s` in text; Holla `h_flow_discovering` changes its discovering spinner. These are not merely whitespace or font fallback.

The enabling evidence gap is treating independently captured representations as one atomic frame because they share a basename and valid hashes. Each file can be intact while representing a different observable state. TASK-001/002/003/070 must retain a single host-observed frame identity for canonical cells/cursor and derived representations, with exact clock/step provenance. Never choose whichever archived member happens to match current output or suppress changing cells. The qualified capture/comparison path must reject mismatched frame identity and changed observations; existing archived evidence remains unchanged.

## BA-ART-02: historical provenance is not the immutable oracle

Main `baseline/before/MANIFEST.md` names `0cd1bf7` in its generated header but `d5e7075` in its method body. Its 499 recipes cover earlier three-app states, not the complete current four-app oracle. Main `shots/capture-provenance.json` binds all 112 entries to dirty revision `61b6f8f9f793d7a2f3cef22154342a8ef7cb25aa`, not either compared tip or immutable oracle. Holla's 302 capture manifests record dirty earlier revisions: 12 at `e4866ce41a485532e57bdc5761d90f0af541086c` and 290 at `92d91629804ffadb109ffbcd7a6765ac73afff7d`.

The old README explicitly warns that some PNG fonts lack CJK/emoji coverage. The old baseline NOTES record an earlier Container-info crash, undeliverable legacy Ctrl+backslash, unwired prefix and states captured only in progress. Those are useful historical limitations, not instructions to reintroduce old defects or infer completed later states. The newer immutable-oracle Settings crash is a separate reproduced finding and authority decision.

TASK-003 preserves historical files/provenance as archival evidence and independently generates source-qualified current oracle baselines. TASK-007/008 disposition original recipe/test identities; full app owners close their actual reached routes. TASK-069 must not accept the historical `required` recipe flag, dirty capture, matching PNG hash or prior review string as a fresh completion receipt.

## BA-ART-03: preserve canonical rendering improvements, not scalar exporters

The tooling partition independently confirms main's surviving ANSI exporters regress from Holla's shared cell-grid path: scalar `len`/width logic mismeasures combining/wide text and fails clipping. Historical HTML/PNG byte equality cannot validate those algorithms. Keep the accepted canonical cell/rendering direction and qualified renderer boundary; explicitly retire inaccurate legacy exporters from acceptance or route their live callers through the qualified grid. Do not copy old Holla scripts wholesale or claim all PNGs are faithful because decoding succeeded. Exact executable-source findings and responsible script scopes belong to the tooling report.

## Porting decision

Do not port generated images, guard/lock files, logs or dirty historical captures as new baselines. Preserve recoverable historical evidence and its exact source references; finish main's controlled capture and canonical comparison paths. Carry forward any source-qualified interaction requirement through existing task/scenario ownership, not through unexecuted archival recipes. This partition establishes artifact integrity, limitations and use disposition only; current product behavior and visual parity remain separately unproved.
