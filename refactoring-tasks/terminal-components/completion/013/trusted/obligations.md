# TASK-013 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

Segment each complete logical line before applying style ranges so split combining marks, ZWJ emoji and CJK clusters remain indivisible. Tabs/control display coordinates and copied original bytes remain consistent at clipping edges; clear wide continuation shadows. Cover Shift navigation, Ctrl/Alt word and document grammar, paste, selection direction and fuzzy indexes in original graphemes.

Preserve borrowed TextEditorCore/TextBuffer/CellWidth ownership and allocation-free clipping; no per-widget alternative Unicode algorithm or preformatted owned span copies.

Red cases split a grapheme across differently styled spans, clip half a wide cell/tab, reorder byte versus grapheme fuzzy offsets, and use platform modifier variants. Existing keyboard_editor/fuzzy/paint checks remain active; 100k-grapheme clipping to eighty columns must retain its work/allocation contract.

## Fixed re-audit contract — cross-fragment grapheme source and painter

Repair the enabling split in main ui/paint.rs:140–168 as well as text infrastructure: styled fragments are a single logical line before Unicode grapheme segmentation. A grapheme spanning fragments takes its style/provenance from the fragment containing its first byte, matching oracle viewport.rs:19–21; preserve original source byte offsets and copy text. Empty fragments, CRLF, combining marks, emoji ZWJ and regional-indicator pairs must not create artificial boundaries. Preserve the shared ratatui CellWidth measure, clipping, wide-cell shadow reset and tab-stop semantics.

Provide one borrowed cross-fragment logical-grapheme projection consumed by paint_spans; its exact iterator/scratch representation stays private, with no full-document materialization, source cloning or per-fragment segmentation fallback. Scratch needed for a cross-fragment cluster is proportional to that cluster, not a fictitious fixed Unicode maximum, and is counted in allocation/work witnesses. TASK-015 RowUi/CellUi styled spans and TASK-021 LineRef/viewport runs consume this same boundary/provenance contract rather than introducing their own splitting engine. TASK-011 precedes this shared painter repair; TASK-014 then preserves it while adding fades.

Exact direct cases split e + combining acute, an emoji ZWJ sequence and an RI pair at every UTF-8 fragment boundary, including empty fragments and distinct styles. Compare unsplit/split source offsets, cells, cursor/hits, selection/copy and clipping at both edges; assign cluster style to its first byte, not the final fragment. Reject a per-span iterator mutant, dropped zero-width suffix, incorrect style ownership, a stale wide shadow and a whole-document clone. Record visible callback/byte work rather than claiming PASS from source search.

## Fixed branch re-audit contract — single-pass formatted text and private scratch

BA-FMT in branch-diff-components-a.md demonstrates that RowUi::label_fmt/main collection/rowui.rs:256 loses a combining suffix when one Display invocation writes a cluster in separate fragments. A borrowed-span fix alone does not cover fmt::Write. TASK-013 supplies one crate-private streaming logical-grapheme projection and Ui painter, shared with the existing borrowed span boundary/provenance engine; TASK-015 alone migrates RowUi/CellUi/default Display consumers. Add no public scratch argument, second segmentation engine, whole formatted-line String/Vec, repeated Display invocation or fixed Unicode-cluster truncation.

The concrete storage owner is one private text scratch field in ui::FrameState, already owned by one Runtime and reborrowed through Ui. It contains a 64-byte inline pending buffer and reusable overflow capacity for the single unfinished cluster plus at most one scalar lookahead, never the full row/document. Keep valid overflow text as String so as_str is O(1); do not revalidate or resegment the growing prefix per scalar. A scoped guard borrows the actual Ui, temporarily takes its scratch, and restores cleared capacity on normal completion, returned fmt::Error and panic unwind. Best-effort safe wiping follows the existing Secret contract; no guaranteed-erasure claim, plaintext Debug/Clone, published FrameOut/state/effect field, global/static/thread-local cache or extra dependency. Secret controls still project masks before this path. A new Runtime is cold; the same Runtime may retain capacity, but different runtimes share nothing.

The shared painter invokes the original Display exactly once per actual paint callback, keeps formatter write effects even after clipping, and stops further segmentation/scratch growth once output cannot admit another cluster. Successfully written prefix bytes are flushed even if Display returns fmt::Error; panic only clears/restores storage and never emits a new partial cluster during unwinding. Cluster style belongs to its first source byte. Preserve the existing clipping, tab/control policy, width measure, original-byte provenance and wide-cell shadow reset, without per-fragment style resolution or geometry drift. Internal checked offsets and end-of-format flush must be explicit; the disposable open-ended GraphemeCursor prototype is feasibility evidence, not an unchecked EOF/API assumption to copy.

ALL existing borrowed inline ASCII/CJK/combining zero-allocation, 500x3 span, 10k ellipsis and long-ZWJ visible-column allocation gates remain byte-for-byte applicable. Narrow label_fmt's universal “0 allocations” rustdoc to the proved inline domain; do not relax A120 or quietly subtract painted-cell storage. New streaming witnesses report actual scratch, painted-cell/style and total allocation scopes, cold first owner, same-owner warm reuse, fresh owner and resize separately. For the frozen fixtures e+acute/CJK/ASCII/family-ZWJ scratch stays zero cold/warm; a+256 acute+z and a+4096 acute+z have cold scratch ceilings 5 and 9 allocation/reallocation events respectively, warm scratch zero when capacity is sufficient, live pending bytes <= the current unfinished cluster bytes+4 and retained capacity <2*(H+4), where H is the largest cluster encountered over this same Runtime scratch owner's lifetime, not merely the current draw. Long→short→long and allocation resize preserve empty logical scratch plus high-water capacity reuse; a new Runtime resets that lifetime and its counts remain cold. Output long-cell allocation is independently retained in total counts, not hidden as warm zero. Future hosts freeze exact allocator/toolchain observations before dispatch; lower counts can satisfy the ceiling only with identical bytes/work and no preallocation moved outside the measured cold scope.

The planning qualifier evidence/streaming-grapheme-bootstrap-20260911 demonstrates seven actual prototype tests, exact public List cells, single original formatting invocation and a compiled quadratic UTF-8 revalidation mutant rejection. Product proof still executes current production paths: add named ui::paint::tests::completion_013_streaming_projection, completion_013_streaming_storage and completion_013_streaming_cleanup unit cases for this crate-private seam, plus public borrowed-painter checks in completion_013.rs. TASK-007 freezes their exact package/target/test identities. TASK-013 must not await TASK-015 or TASK-021 receipts; those later consumers and TASK-031/069 close their own source-to-painter joins.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each advertised visible override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. The accepted covered-surface TextArea FIELD exception (main COMPONENT_ARCHITECTURE.md:6799–6815,6997–7003; HIST:EARLY-AMEND-044) instead requires exact recorded selected-part Resolved values while the composed painter's final digest stays unchanged; this is not an exemption for a visible override or replacement slot. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Finite source-qualified witnesses

`trusted/source-witnesses.md` is normative under R-001/R-002/R-003. TASK-006 freezes every expanded case ID, exact source-qualified input/checkpoint and independent expected observation before dispatch; CHK-002/004 compare parity lanes, CHK-005 executes real positive/negative cases and CHK-006 proves architectural ownership. The finite rows supplement all assigned historical clauses and public part/slot census; they do not replace them with a representative subset. Missing source coverage is a failed precondition, not executor-selected non-applicability.

## Specific regression target

Add `crates/tui/tests/completion_013.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `keyboard_editor`, `fuzzy_boundary`, `paint_middle`, `paint_matched`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:text-core

- family: text-core
- reference_implementation: O:src/core/text.rs;ui/text.rs;widgets/field_common.rs
- main_implementation: M:crates/tui/src/text/*
- architectural_target: Shared grapheme-safe editable text measurement fuzzy matching
- visual_status: split styled graphemes differ in viewport
- interaction_status: editing modifier/copy behavior unverified
- api_refactor_status: implemented; borrowed projection semantics incomplete
- tests_available: M:keyboard_editor.rs;fuzzy_boundary.rs;paint_middle.rs;paint_matched.rs
- tests_missing: CP-TEXT-OUTPUT cross-style graphemes tabs controls copy; field modifier matrix

### ARCH:A15

- id: A15
- area: text editor and secrets
- oracle_state: TextBuffer clones fn callbacks
- main_state: TextEditorCore TextBuffer Secret Validate FieldControl
- architectural_target: one editor action grammar; borrowed validation; secret containment
- status: implemented;behavior drift
- remaining_obligation: preserve exact legacy editing interactions through shared core
- available_gates: keyboard_editor;input_placeholder;compile_fail_cases_hold;secret unit tests
- missing_proof: oracle selection/edit/blur/unicode/readonly sequences
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:1450;7b27732:crates/tui/src/text/editor.rs:1

### ARCH:A16

- id: A16
- area: text measurement/paint
- oracle_state: fit allocated strings multiple width paths
- main_state: one CellWidth measure borrowed painters
- architectural_target: one width contract; zero allocation clipping; original grapheme fuzzy indices
- status: implemented
- remaining_obligation: preserve unicode and clipping under parity additions
- available_gates: fuzzy_boundary;paint_middle;paint_matched;fit_100k_grapheme_line_to_80_wide
- missing_proof: wide-cell/combining canonical parity frames
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:3882;7b27732:crates/tui/src/text/measure.rs:1

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A29

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §22,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: One Unicode width/grapheme writer matches terminal cell width and clears wide-cell shadows
- Disposition: accepted; current retain_and_verify
- Remaining proof: CJK/combining/ZWJ/halfwidth marks and no non-ASCII exclusions
- Gates: Unicode corpus;canonical cells
- Origin: docs/refactoring-plan/historical-obligations.tsv:30; global semantic anchor; supplemental clauses retained

### HIST:A30

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §22,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Span rendering inherits label base style without intermediate string/vector allocations
- Disposition: accepted; current retain_and_verify
- Remaining proof: Styled-span segmentation agrees with complete logical text
- Gates: span differential;500x3 zero alloc
- Origin: docs/refactoring-plan/historical-obligations.tsv:31; global semantic anchor; supplemental clauses retained

### HIST:A120

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §25,§27; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Row ellipsis 10k zero allocations; ZWJ storage separate bound <=80 for 80 columns
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: No arbitrary <=8 allowance or skipped Unicode workload
- Gates: grapheme perf
- Origin: docs/refactoring-plan/historical-obligations.tsv:121; global semantic anchor; supplemental clauses retained

### HIST:F08c

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f08c-ctrl-shift-home-end.md
- Requirement: Ctrl+Shift+Home/End extend selection according to editor contract
- Disposition: later_open; current current_main_mapping_required
- Remaining proof: Modifier matrix and selection anchor integrity
- Gates: Editor modified keys
- Origin: docs/refactoring-plan/historical-obligations.tsv:141; global semantic anchor; supplemental clauses retained

### HIST:F10

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f10-segment-logical-text-before-applying-styles.md
- Requirement: Segment complete logical graphemes before applying span styles
- Disposition: current_done; current current_main_mapping_required
- Remaining proof: Styled split clusters match unstyled geometry and copy
- Gates: Unicode style corpus
- Origin: docs/refactoring-plan/historical-obligations.tsv:144; global semantic anchor; supplemental clauses retained

### HIST:F14

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f14-explicit-tab-control-geometry-and-copy-policy.md
- Requirement: Tab/control display geometry and copied text policy explicit
- Disposition: current_done; current current_main_mapping_required
- Remaining proof: Paint/cursor/hit/search/copy agree on logical source
- Gates: Text control corpus
- Origin: docs/refactoring-plan/historical-obligations.tsv:148; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-007

- Source: 6ec29171 §§16.1,29.7; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Regression proof exercises Clear geometry, keyed strip rendering, both cell boundaries, and actual ASCII glyph sets.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Clear reserves a blank cell; same buggy enumeration or non-overflowing fixture is insufficient; no-box-drawing is weaker than all-ASCII.
- Gates: Clear/Inherit differential; overflowing reorder; left/right out-of-area rejection; full typed GlyphSet ASCII scan.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:8; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-023

- Source: e8d053c9 §20.9;587c53bd §25;4aabceb7 §27; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Text paint is single direct grapheme walk, no fit String/Vec path; inline Cell symbols allocate0; long ZWJ allocation bound depends on visible columns, not document length.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Reject magic<=8; retain honest CompactString allocation caveat.
- Gates: ASCII/CJK/combining zero; long ZWJ clip bound<=visible columns; exact legacy fit padding/ellipsis differential.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:24; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-044

- Source: 27bd918e §22;87ab93d4 §24; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: ratatui-core CellWidth is width source; segmentation in text; use set_stringn without pretruncate; reset wide shadow cells; preserve raw vocabulary through curated reexports.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Role-aware Span and intrinsic Size intentionally remain own types; foreign raw text qualified author::raw.
- Gates: Combining/CJK/halfwidthmark/ZWJ/control corpus; facade compile example; wide→narrow stalecell; nonalloc span painting.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:45; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-005

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:89-114;196-207
- Requirement: Scroll state invariants and Unicode text offsets need structural safe interfaces
- Disposition: audit finding; exact fix not accepted by this source; current unverified
- Remaining proof: Validate surviving issue class including casefold-to-original mapping
- Gates: Grapheme/cell corpus; Turkish dotted I; boundary wheel; no out-of-bounds
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:6; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-027

- Source: ba858131; docs/audit/modern-api-audit.md R1-R6; foundations F4/F10
- Requirement: One cell-width source and clipped writers; no render fit allocation; reset wide glyph continuation cells
- Disposition: accepted amended: set_span added beside set_line; current not independently tested
- Remaining proof: Complete Unicode corpus including halfwidth marks/ZWJ/CJK/control cases; preserve padding/ellipsis by oracle
- Gates: legacy differential with no trim/skip; tiny rects; shadow-cell reset; zero allocations inline corpus
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:28; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
