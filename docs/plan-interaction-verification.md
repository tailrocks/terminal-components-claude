# Interaction verification for the improvements plan

Verified September 10, 2026 against the current `holla-fable` worktree. This is a planning report: no production source was changed during this verification. No other branch or history was inspected. Findings below distinguish executable reproductions, complete source traces, and coverage decisions.

## Verification performed

- Rebuilt the current library and ran all 132 library tests: passed.
- Ran all 34 showcase tests: passed. These include modal trapping/restoration, hover/focus distinction, disabled activation, editing validation, drag scrolling, representative sizes, and palette/baseline checks.
- Ran TablePro's `safety_gate_acknowledgement_supports_mouse_focus_edit_and_confirmation`: passed.
- Compiled temporary executables against the current library and, where necessary, the current application modules. Their source was supplied through Rust stdin; binaries live under `/tmp/junie-plan-*`. No repository test or application source was edited.
- Independently rebuilt and ran the text reviewer's retained viewport/grid probes. Results are recorded under cross-verification below.

Passing existing tests is evidence for their named scenarios. It does not establish complete keyboard/mouse parity or cover the new reproductions below.

## Existing findings: current disposition

The earlier `tui-audit-interactions.md` mixes findings recorded before implementation with later fixed labels. Use this table for planning; do not schedule already-fixed defects again.

| Earlier finding | Current disposition | Current evidence |
| --- | --- | --- |
| Unicode `selection_lines` panic and unsafe byte/grapheme offsets | Fixed | `src/core/text.rs:95`; tests at `608`, `619`, `643`, `658`, `677`, `692`. Fresh standalone `multi("日"); select_all(); selection_lines()` returns `(0, 0)`. |
| Modified Enter panics in CodeEditor | Fixed for the tested bindings | `src/widgets/code.rs:1041` tests modified Enter committing without document changes. Shared binding producer remains `src/widgets/field_common.rs:24`. Do not infer that every possible terminal modifier combination was tested. |
| Case-fold search reuses transformed byte offsets | Fixed | `src/widgets/code.rs:256`, test `1056`; fresh search for `é` in `İé` yields original-byte match `[2..4]` without panic. |
| Find-bar paste changes the CodeEditor document | Fixed in CodeEditor | `src/widgets/code.rs:584`, test `1065`; fresh probe preserves document `hello` and sets needle `needle`. Separate application modal paste leakage remains INT01 below. |
| DataTable sorting redirects active edits to another source row | Fixed | `src/widgets/table.rs:86` documents source identity; tests `986`, `1003`; all library tests pass. |
| Invalid table/grid cell clicks move away and replace the draft | Fixed for the cell-click paths | `src/widgets/table.rs:1022`, `src/widgets/grid.rs:2093`. Other collection mutation/selection invariants belong to the API verification report. |
| Invalid DataGrid edit emits header sort/reload request | Fixed | `src/widgets/grid.rs:2109` proves no sort event/state change while the invalid draft remains. |
| Table/grid edit windows mix scalar and display-cell offsets | Fixed for the covered cell windows | `src/ui/text.rs` public `slice_cells`; rendered tests `src/widgets/table.rs:1036`, `src/widgets/grid.rs:2122` cover CJK, emoji, combining marks, clicks, adjacent cells, and monochrome. Cross-style viewport segmentation remains a separate text finding. |
| Required TextInput error never clears | Fixed | `src/widgets/input.rs:176`, regression `500`. |
| Masked input mouse positions use raw widths | Fixed | `src/widgets/input.rs:251`, regressions `517`, `532`. |
| Narrow TextInput panic and cursor outside allocation | Fixed for tested widths/states | `src/widgets/input.rs:547`, `576`; no writes outside the tested allocated rectangles. |
| Showcase first click edits instead of focusing | Fixed | `src/bin/showcase/app.rs:631`, `696`: press no longer pre-focuses editor controls; completed click sees prior focus. Fresh real-App/TestBackend probe: click 1 `focused=true, editing=false`; click 2 `focused=true, editing=true`. |
| Facts acknowledgement ignores mouse | Fixed | `src/widgets/dialog.rs:330` uses `input_mut`; TablePro regression `src/bin/tablepro/app_tests.rs:459` passes mouse focus/edit, wrong-value refusal, correction, and confirmation. |
| TextArea long edits are invisible | Fixed for tested lines/layouts | `src/widgets/textarea.rs:265`, tests `438`, `485`, `499`; existing Textareas showcase includes long Unicode text without changing the 28-line scenario contract. |
| Editing render undoes manual wheel/scrollbar movement | Fixed in TextArea and CodeEditor | Tests `src/widgets/textarea.rs:463`, `src/widgets/code.rs:1089`, `1105`, `1148` cover preserved manual viewports, hidden offscreen cursor, and resumed cursor follow. |
| Inconsistent modifier matching | Still open; reproduced architecture/API weakness | INT05 below. Existing application shortcut interception hides some collisions but does not repair reusable widget contracts. |

## New or remaining implementation candidates

### INT01 — P1 confirmed defect: paste reaches the document behind a picker

**Executable evidence.** A fresh TablePro application with `TestBackend(120, 40)` connects to Production. Send Tab, `i`, and Ctrl+O, drawing after each event. The query editor is editing and the picker is visibly open. Send `Input::Paste("SELECT secret")`. The underlying query becomes `SELECT secret`; the picker query remains empty. The probe inspected the real `App`, active `WorkTab::Query`, and `Modal::Picker` states, without setting private fields. The text reviewer independently reproduced the same route with another pasted value.

**Current locations.** `src/bin/tablepro/app.rs:236` routes paste specially only for Dialog and Filter, then falls through to the screen. `src/bin/tablepro/workbench.rs:1196` forwards paste to the active editor/grid. `src/bin/tablepro/tabs.rs:1544` forwards it to the editing query. `src/bin/tablepro/app.rs:1230` opens the picker without immediately changing focus; `2091` repairs focus only after drawing. Thus the first modal frame can still leave the background editor's editing flag true. Holla (`src/bin/holla/app.rs:415`) and Jackin (`src/bin/jackin_preview/app.rs:489`) already consume unsupported top-modal paste instead of falling through.

**Root cause and plan.** Event routing is split by input kind, and paste lacks the exhaustive modal ownership rule already applied to keys. Route paste through the top overlay first; unsupported overlays consume it, searchable pickers receive query paste through INT03. Treat focus transfer as one overlay transition, so correctness does not depend on a second render. Reuse existing `Input`, `Outcome`, modal variants, and picker; no new component is needed.

**Risk/dependencies.** Medium interaction risk: picker opening while editing, invalid grid drafts, and focus restoration must retain intentional commit/cancel semantics. Depends on a narrow picker paste API if pasted search is included. This is hidden document mutation, not evidence of query execution or network access.

**Acceptance.** Reproduce query editing → Ctrl+O → paste before and after additional redraws; only picker query changes. Repeat with a grid draft, an invalid draft, picker error/loading states, non-searchable picker, and nested/topmost modal routes. Closing the picker restores the original focus and leaves the background value unchanged. Add application tests, not only a Picker unit test.

### INT02 — P2 confirmed defect: BETWEEN upper-bound paste is ignored

**Executable evidence.** Connect TablePro to Production, open a table from the explorer, and open the filter editor. Select the existing Between operator, focus the visible `value2` input, and enter editing through its public API. After drawing, `value2.editing` is true. `App.handle(Input::Paste("50"))` returns `Consumed`; both value strings remain empty. This probe uses a real rendered filter and real application paste dispatch; the operator/focus setup uses the same public controls as keyboard handling.

**Current locations.** `src/bin/tablepro/app.rs:240` checks only `f.value.editing`. Keyboard dispatch (`1692`) and click dispatch (`2073`) iterate both `value` and `value2`. The second input is rendered for Between at `2405`.

**Root cause and plan.** Different event paths enumerate different form controls. Give FilterEditor one focused-input routing method used by keys, paste, and clicks, or at minimum make paste select the active input from the same control set. Do not infer the target from the first field's editing flag alone.

**Risk/dependencies.** Low; share the modal-first routing work from INT01. Preserve single-value and value-free operators.

**Acceptance.** Keyboard-focus and mouse-focus each Between value; paste different values and apply. The resulting filter contains both values. Paste into the lower field still works; hidden `value2` never receives paste; Apply/Cancel focus consumes paste without changing either value.

### INT03 — P2 confirmed defect plus coverage gap: Picker query editing bypasses grapheme/paste primitives

**Executable evidence.** Type the scalars of `👩‍💻` through `Picker::on_key`, then Backspace. Current query is `"👩\u{200d}"`, an incomplete displayed grapheme. `src/widgets/picker.rs:199` uses `String::pop`, unlike the shared grapheme-aware `TextBuffer`. The picker has no `on_paste` entry point. Holla and Jackin consume picker-modal paste (`app.rs:415` and `489` respectively), while TablePro has INT01. Therefore pasted searchable queries are absent across the current owners, not merely missing from a fixture.

**Plan.** Reuse `TextBuffer` editing semantics or its grapheme-boundary logic inside Picker query edits. Add a query-paste method that returns the existing `QueryChanged` event; route it through each actual owner so filtering/ranking is refreshed exactly once. Preserve the public query compatibility contract unless the API plan explicitly approves a migration.

**Risk/dependencies.** Low to medium: query normalization and owner refresh behavior, not a new search engine or component. Coordinate with the API report's Picker status/secondary-action guards; those are a separate data-integrity concern.

**Acceptance.** Backspace removes one full combining or ZWJ grapheme. Typed and pasted identical queries produce identical query text, match rows, selection, and clear/close behavior. Non-searchable pickers consume paste. Loading/error behavior is defined and tested. Cover showcase plus one real application owner.

### INT04 — P2 confirmed defect: Ctrl+Shift+Home/End clears selection

**Executable evidence.** TextArea contains `first\nsecond`, enters editing, and moves to document end. Ctrl+Shift+Home produces cursor byte 0 with `selection=None`. `src/widgets/field_common.rs:78` and `79` call `move_doc_start(false)` / `move_doc_end(false)` before the Shift-aware Home/End cases. Both TextArea and CodeEditor use this shared dispatcher.

**Plan.** Preserve the Shift flag for document-boundary movement at the shared binding layer. Keep unshifted Ctrl+Home/End as movement without selection. This is an existing edit action, not a new mode or key family.

**Risk/dependencies.** Low; grouped with INT05's binding conformance table, but does not require a broad routing rewrite.

**Acceptance.** Ctrl+Shift+Home/End extends an existing selection anchor and creates a selection when none exists, including backward ranges, Unicode endpoints, empty documents, and multiline text. Plain/Ctrl Home/End retain existing semantics in TextInput, TextArea, CodeEditor, and cell editors where supported.

### INT05 — P2 architecture/API weakness: unbound modifiers trigger plain navigation/actions

**Executable evidence.** A two-row DataTable receives `Ctrl+S`; `on_key` returns `Changed` and changes sort to ascending. `src/widgets/table.rs:398`–`442` matches many key codes without modifier checks, including sorting at `438`. Similar unguarded paths exist in `src/widgets/grid.rs:992`, `src/widgets/textarea.rs:110`, `src/widgets/select.rs:78`, and `src/widgets/menu.rs:190`. `Key::plain` intentionally tolerates Shift (`src/core/event.rs:64`), so adding that check everywhere would not itself define all selection behavior.

**Qualification.** TablePro currently intercepts Ctrl+S at the application layer before DataTable/DataGrid. The probe proves a reusable API collision; it does not prove TablePro's Save command is broken. Picker deliberately supports Ctrl+J/K/N/P and those bindings must remain.

**Plan.** Establish a binding matrix for plain, Shift selection, explicit Control/Alt chords, and unassigned modifiers. Shared edit actions own text bindings; each navigation control owns only its documented chords. Nonmodal unassigned keys propagate as `Ignored`; modal layers may consume them without performing an unrelated action. Test owner precedence where application chords coexist with controls.

**Risk/dependencies.** Medium compatibility risk. Inventory actual application shortcuts first; avoid redefining intentional Emacs/Vim conventions or terminal key normalization based only on familiar desktop behavior.

**Acceptance.** Table Ctrl+S does not sort unless explicitly documented as a table binding. Explicit sort keys still work. Shift selection still works in list/grid/text controls. Picker Ctrl+J/K/N/P still moves. Application Save, Run, Find, Close, and detach chords reach their intended owner with a widget focused and with text editing active as applicable.

### INT06 — P2 coverage gap: TextViewport selection cannot be created by keyboard

**Executable evidence.** A populated TextViewport receives Shift+Right: `(Ignored, None)`. Shift+Down returns `(Changed, None)` but creates no selection. `selection()` remains `None`. `src/widgets/viewport.rs:563` supports scrolling, follow, copying an existing selection, and clearing it. Selection creation occurs through mouse paths at `497` and `535`; no keyboard selection-creation path exists. `caret` at `116` is terminal/application state, not a keyboard selection cursor. DiffView composes this viewport, so it inherits the gap.

**Plan.** Extend the existing viewport with an explicit selection-navigation mode or equivalent opt-in keyboard anchor/head operations. Reuse its logical selection and copy result, with the same Unicode/wrapped-line mapping as mouse selection. Keep normal scrolling and attached-terminal input ownership intact. Update contextual hints in viewport, diff, logs, and detached terminal owners.

**Risk/dependencies.** Medium: coordinate with the text report's eviction/anchor identity and styled-grapheme fixes before adding another selection producer. Selection-mode activation and cancellation are product decisions; the absence is confirmed, while a particular key such as `v` is only a proposed choice.

**Acceptance.** A keyboard-only user selects/copies a word, partial line, multiline range, and wrapped Unicode range without a mouse. Selection behaves consistently across resize and scrollback eviction. Esc cancels selection before leaving its owner. Normal navigation and attached terminal input retain their current behavior. Monochrome selections remain legible.

### INT07 — P2 confirmed defect: Inspect advertises Copy but discards its event

**Complete source evidence.** `src/bin/jackin_preview/screens/inspect.rs:250` and `343` discard the event returned by `DiffView::on_key`. Both reading modes advertise `y Copy` at `749` and `772`. There is no alternate `y`, `Copy`, or clipboard handler in this module. `CustomModal` (`src/bin/jackin_preview/screens/mod.rs:66`) exposes `Outcome`, read-only World access, and terminal `done()` results; it has no nonclosing request channel. The existing application `Request::Copy` handler (`src/bin/jackin_preview/app.rs:1964`) updates the preview clipboard and status, but the discarded diff event never reaches it. The parent independently reviewed this complete path. This is source-confirmed; this review did not inject a selected Inspect range into a full running Jackin scenario.

**Plan.** Preserve the typed copy event through a narrow nonclosing modal-effect/request channel and route it into the existing preview `Request::Copy` handler. Leave Inspect open, retain selection/focus, and show existing feedback. Do not change this preview application's clipboard target to the operating-system clipboard.

**Risk/dependencies.** Medium interface risk within the app's custom-modal transport. A default empty effect-drain method can preserve other implementations; choose the final shape after inspecting all CustomModal consumers. Depends on the text report's accurate selected-text behavior, not on adding a new diff component.

**Acceptance.** In compact/open-file and advanced/diff modes, drag-select text then press `y`; exact selected text reaches `World.clipboard`, clipboard generation increments once, feedback appears, and the modal stays open with focus/selection retained. No selection does not fabricate a copy. Unrelated custom-modal actions and close results remain unchanged.

## Focus, layering, and state conclusions

Existing primitives are sufficient for the identified work: `FocusRing`, `HitRegistry`, `RenderCtx::begin_modal`, `Input`, `Outcome`, `TextBuffer`, TextInput, Picker, TextViewport, and DiffView. Modal barriers already restrict hit/focus reachability; Holla and Jackin already save/restore focus per stack entry (`src/bin/holla/app.rs:922`, `940`; `src/bin/jackin_preview/app.rs:1229`, `1255`). Showcase's restoration and backdrop-click regression passes. These facts do not justify replacing the focus system.

The independently observed layering defect is inconsistent **paste ownership**, not absence of a modal stack. A shared overlay-routing contract may reduce recurrence; a universal overlay manager remains a hypothesis until its proposed API demonstrably removes duplicated owner behavior without obscuring application policy.

Button activation uses the same `can_activate` guard for keyboard and mouse (`src/widgets/button.rs:67`, `74`, `87`), rejecting disabled and busy controls. Disabled/pressed visuals are explicitly suppressed at `119`/`123`; the showcase disabled/hover/focus tests pass. Error recovery and acknowledgement validation are covered by the tests listed above. No additional hover, pressed, busy, or focus-restoration defect was established by this review. No claim of exhaustive state coverage is made.

## Independent cross-verification of the text/API work

The text reviewer supplied temporary `probe.rs` and `grid.rs`. This review read them, inspected their responsible source paths, recompiled them against the current library, and reproduced:

| Peer finding | Independently observed current result | Scope qualification |
| --- | --- | --- |
| Styled grapheme split | Joined ZWJ puts following `X` at column 2; split styled spans put it at 4. Split combining accent copies `aX` instead of `áX`. | Supported styled viewport input; segmentation is per span at `src/widgets/viewport.rs:321`. |
| Retention limit bypass | `max_lines(3)` plus `set_lines(8)` and `with_lines(8).max_lines(3)` both retain 8 lines. | Supported public constructors/setter; cap enforced only by `push` at `171`. |
| Eviction identity | Selected `A` becomes selected/copied `B`; retained drag anchor on `B` copies only newline; manually visible top `B` becomes `C`. | Supported bounded-stream append; `push` saturating index subtraction at `178` retargets content and omits drag/view anchor repair. |
| Public mutation/cache bypass | Public model string becomes `NEW`; rendered cells still say `OLD`. | Public mutation is representable but bypasses `dirty` at `ensure_layout:305`. Classify API/invariant weakness separately from supported setter flows. |
| Grid editability changes during edit | `editable=false` after beginning an edit still accepts paste and commits `CellChanged` into pending data. | Public runtime flag; guards absent from `commit_edit` and `on_paste:1390`. Coordinate the API plan's mutation/ownership policy. |
| Tab normalization differences | Input/TextArea/CodeEditor render `A\tB` with one display cell for the tab; TextViewport places `B` after four spaces. | Confirmed inconsistent geometry; retained control bytes are not by themselves proof of terminal escape execution. |
| Wrapped selection on resize | Selected `bcdefghi` remains exactly `bcdefghi` after resize. | Passing counterexample; do not schedule a general resize-selection rewrite based on the eviction findings. |

The API reviewer owns stale ListBox selection anchors after removal, Tree replacement identity, grid sorting selection identity, and Picker secondary-action validity. In particular, its empty-tab-picker Delete finding is distinct from INT03 query editing; the final plan should keep its data-integrity priority and avoid merging it into a cosmetic search task.

Additional independent review of the text report's release probe confirms full-history update scaling: 20 replace-last/layout requests took 84,464 µs at 1,000 rows, 666,123 µs at 10,000 rows, and 3,423,090 µs at 50,000 rows. Each row contains 80 ASCII characters, viewport 80×20, current release library, `rustc -O -C lto=thin`. Idle calls rounded to 0 µs. The supplied probe changes `x` to `y` on its first request and resubmits identical `y` afterward; these figures are repeated replacement requests, not 20 distinct content changes or measured terminal frame times. The reviewer was asked to retain that distinction. No performance budget or speedup ratio follows from this sample.

The grid TSV observation is also reproduced: pasting `A\tB\r\nC\tD` into one edited cell stores `A\tBC\tD`. This is current single-cell normalization behavior; it does not prove a matrix-import contract violation. Keep any matrix-paste feature as a separate coverage/product decision.

## Suggested dependency order

1. Close hidden/incorrect mutation paths: INT01 and the API/text reports' invalid secondary action, readonly edit, and collection identity defects.
2. Repair shared event/editing ownership: INT02, INT03, INT04, INT07; preserve typed outcomes and existing preview clipboard semantics.
3. Establish modifier conformance (INT05) against actual application bindings.
4. Add keyboard viewport selection (INT06) after logical text/retention identity is correct.

Each implementation item requires a narrow failing regression first, relevant keyboard/mouse/paste parity, application integration tests, and rendered small/normal/wide plus monochrome evidence where its visuals change. No additional component is justified by this interaction inventory.
