# Component caller contracts after T37

## Status and evidence

This guide preserves selected caller requirements from Rustdoc removed during T37. It records source-backed constraints, not behavior proof.

The removed attachments come from source `c817b99e9e43abbbf3519d56b2f5ee86924243d8`, product source `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`. The target source is `cfe0587ad034d9f0fd18694934f071f6f50c7dc5`.

The raw archive has 722 zero-based groups. The [transfer ledger](../implementation/pr38-final/results/t37-component-api-transfer/transfer.json) records each selected group, exact attachment, hashes, and disposition.

A Sol high review, `T37-COMPONENT-PROSE-01`, audited all 722 groups and approved this grouped scope. Its review decision identifies existing contract mappings and unresolved limits.

No behavior cases were selected. Behavior remains `NOT VERIFIED`. No Cargo command, build, or test ran for this transfer.

The source locations below identify removed caller prose. The ledger records retained source file hashes at the target revision and exact maintained excerpts.

## Focus registration and input ownership

### Button focus policy

- **C01 (G5):** Keep `ClickOnly` pointer hit-testable and pointer-activatable, but exclude it from keyboard traversal and activation. `Disabled` suppresses both.

### Form-clipped native controls

- **C02 (G6, G20, G25, G27, G325, G332, G416, G549):** When Form clips a native body, use its keyboard-only registration helper. Preserve native disabled and read-only guards, including inherited Form disabling. Keep empty radio groups inert.

### Text input focus and command ownership

- **C03 (G309, G313):** Pointer focus leaves initial placement to the pointer gesture. A retained draft keeps its caret and selection. Preserve a selection when a context-menu target lies inside it; otherwise place the caret at the target.
- **C04 (G320):** Exclude owner-routed editing commands from the field binding table. Keep field rendering and state ownership with the editor.
- **C05 (G302):** Update field state before acknowledging the exact layer opening that moves focus away. Process the matching continuation result before acknowledging that outcome.

### TextArea coordinates and drag

- **C06 (G329, G330, G334, G336):** Use shared body-window and text-part geometry for pointer mapping. A captured drag clamps above the visible window and extrapolates below it. Report caret rows relative to the logical visible body after document scrolling. Preserve full logical viewport and document row numbering when drawing through a parent window.

## Split controls and forms

### Split sizing and resize policy

- **C07 (G43–G50, G68, G71):** The last sizing-mode builder wins. Fixed detail widths floor; fixed master widths round to the nearest cell; both clamp to configured bounds. With no active initial-cell policy, `.resizable()` restores use of the stored interactive ratio without rewriting it. `initial_master_cells` selects resizable mode and controls the initial master width until manual resize; configured minima can clamp it. `ResetPolicy::Initial` restores that cell width. A later `master_share` call clears the cell setting and selects its percentage. The stored basis-point ratio stays precise after manual resizing; query resolved geometry for the initial cell extent. Other builders and the legacy default retain the existing percentage. Normalize a zero resize step to one cell. An empty resize table disables keyboard resizing but keeps seam dragging. By default, narrow seam drags use raw pointer coordinates; enable drag-offset preservation to keep the grab offset. The default reset policy is `Balanced`.

### Responsive MasterDetail geometry

- **C08 (G53):** Split mode returns empty pane rectangles from responsive summary resolution. Call `layout` when callers need actual split regions.
- **C09 (G601, G602):** The facade's resizable master share seeds only uninitialized split state. Preserve caller state during resize and redraw. A retained fixed-detail percentage does not report the resizable initial share.
- **C10 (G609):** Close a narrow drawer only for the scoped bubble binding routed after focused detail declines Escape.

### Form measurement and lanes

- **C11 (G344):** Treat legacy `measure` as an estimate. It ignores data-dependent visibility, radio-option counts, paired `columns(2)` rows, and explicit lanes. Use `measure_with_data` for live data or intrinsic sizing.
- **C12 (G368, G369):** Give each stable lane ID its own scroll state. Use one vertical lane or two lanes with one horizontal split; Form preflight rejects additional lanes.
- **C13 (G357, G360):** The default Enter policy submits when a field is not editing. Use `Never` when a caller submits through a separate scoped command. Run `update` first on the same frame. Call `FormDialog::submit_current` only after the scoped command bubbles past the focused child. It commits the active field draft, validates through Form, and applies dialog submit policy. Do not repeat those internal steps in the caller.

### Pointer-held Form reveal

- **C14 (G343):** Suppress automatic Form reveal only while a child pointer gesture is live. Start the hold on press; clear it on release, drag-end, or cancellation. Clear it when the field is missing, hidden, or disabled. Otherwise, without a fresh press, clear `pointer_hold` when the held field state lacks `PRESSED`.

### Isolated List focus loss

- **C15 (G429, G433):** Use the helper only for isolated `Settle` focus loss. After `None`, run ordinary update on the same component, state, and pass. Inspecting the bucket does not claim unclaimed intents. Empty buckets return an ignored response. List clears its click cursor; NavList has no focus-loss state.

## Measurement, windows, and geometry

### Measurement inputs

- **C16 (G63):** Measure PropsList from live rows and keyed state. Use its shared update/draw geometry for wrapping and copy-hint reservation.
- **C17 (G441, G442):** For custom rows, project the label's actual rendered cell width with a pure measurement callback. Otherwise use intrinsic width when available, then the 12-cell minimum. Do not invoke row painters or state projections during measurement.
- **C18 (G395):** Pass the grid body's resolved chrome to its scroll bars. Do not recompute bar alignment from column widths.

### Window preflight and caller-owned chrome

- **C19 (G60):** Validate Panel window geometry before drawing. Reject a partial window with a legacy slot before paint, decoration, or callback. Pass the callback visible inner-body rows rebased to the original inner-body source rows. Full windows retain `draw` behavior.
- **C20 (G127–G130):** Keep Accounts header and footer outside pane geometry. Preserve physical master rows and the detail logical origin, scroll viewport, and post-scrollbar physical clip separately. Keep title and action rows outside that clip. The explicit shell painter leaves detail windows, scrollbars, and input to its caller; the legacy whole-page path stays 39 columns. Treat `first_source_row` as the source row at `content_clip` top. Adjust returned source rows for the current `Ui` clip. A valid empty clip returns an empty range. Invalid geometry or an offset beyond `lines.len()` returns `None` before paint. Window painters keep logical columns and omit cut graphemes whole; callers own row backgrounds. Retain typed `DrawerLine` values when rebuilding width projections. Row projection returns full logical text and styles or a typed error.
- **C21 (G137–G139):** Supply Inspector `text_width`, `prewrap_width`, and scroll state. Widths must be positive and fit the logical viewport. Window offsets count visual rows, not source lines; use the measured count for caller scroll state. Keep text anchored at the fixed logical left edge; clipping may reduce its right edge. The caller owns outer chrome, backgrounds, scrollbars, fades, input, and scroll state. Empty or invalid requested rectangles, bad widths, uncontained clips, and out-of-range offsets return `None` before paint. Valid geometry hidden by an ancestor returns an empty range.

## Feedback and fixed Settings painters

### Notice row limits

- **C22 (G176, G177):** Normalize a zero Notice message-row cap to one. A zero detail-row cap suppresses detail.

### Safe values and fixed body limits

- **C23 (G210):** Pass only a star count for a plain secret value. Treat provider paths and environment names as public display references, not resolved secrets.
- **C24 (G228, G276–G278, G280–G284):** The live Editor role view carries at most 29 visible rows and the full source count. Only `paint_editor_environment_body` and `paint_editor_roles_body` require 120 columns and 40 rows. Env text clips by terminal-cell width. The current Roles gate accepts printable ASCII or `·`; each accepted character occupies one terminal cell (`width(text) == text.chars().count()`) and fits its column. Visible role rows cannot exceed the source count.
- **C25 (G285–G287):** Whole-page Env painting requires empty sections, mounts, and add-mount content; validates all rows before paint; and owns page chrome. The fixed Editor Env body reserves rows 6–36 and leaves row 37 for its footer; it clips text by terminal-cell width. The fixed Editor Roles body supports at most 29 rows and accepts only supported fixed-column text. These fixed bodies validate before painting and do not scroll, drop rows, paint controls, or own input.
- **C26 (G288, G289, G291, G293):** Agents bodies support six rows and two notes; Trust bodies support four rows and two notes. Validate capacities before painting. Body-only painters clip page-relative bands, paint no chrome, and impose no minimum page height. Whole-page Agents and Trust painters require empty sections, mounts, and add-mount content; they validate the full layout and own page chrome. Agents bodies do not handle input or focus; Trust bodies also leave counts and save previews to callers.
- **C27 (G292):** The Mounts painter borrows rows, clips to the body band, and owns no page chrome, actions, hints, input, or focus.
- **C28 (G265–G267):** Provide one fixed offset per hint. Choose one hint-tail owner. Supply a running-context cluster when a breadcrumb note uses its fixed anchor.

## Tabs and navigation

### Tabs configuration and plan lifecycle

- **C29 (G460, G476):** Validate effective action identities and chord conflicts across the selected navigation profile and replacement groups. A profile changes only Prev/Next descriptors. Keep the same descriptors for publication and dispatch. Validate New tables only for New and Close tables only for Close.
- **C30 (G474, G495, G496):** Each fresh nonempty default-row plan formats every item once and invokes each configured key, mark, and affix mapper once, including off-window items. Capture keys, marks, close eligibility, Display segments, and owned affix text for geometry, rendering, and the hidden-tabs menu. Empty-area plans skip the affix mapper and preserve legacy callback/key lookup behavior. Rebuild after item content or mapper changes; no same-key text cache persists. `previous` is the caller-owned receipt from the last committed draw. Planning does not paint or read a buffer. Drawing uses captured values without rerunning key, mark, or affix mappers or `Display`.
- **C31 (G497):** Rebuild a captured plan after every state or data change. `draw_layout` checks component identity and rendering configuration; it does not check caller data freshness. Rebuild on either mismatch before drawing.
- **C32 (G470, G471, G475):** Treat allocated and clipped part bounds as geometry, not live hit permission. The committed hit registry determines current interaction visibility.

### Tabs bindings and presentation

- **C33 (G480, G481, G492, G493):** With `None`, keep the built-in New or Close binding in the base `Bindings::bindings` table. `Some(&[])` removes only that command's keyboard entries; pointer eligibility and Middle behavior remain governed by their settings. New replacement tables contain only `TabsCmd::New`; Close tables contain only `TabsCmd::Close`. Retain each binding's action, chord, label, priority, and visibility metadata. Validate action uniqueness and effective-chord conflicts across the potential table, omitting a replaced default group. Validate even when New is disabled, no item is closable, or Tabs is passive. Accessors return empty supplemental slices when gated or passive. Manual publishers include `Bindings::bindings` plus `new_key_table` or `close_key_table` supplemental entries.
- **C34 (G488–G490):** Coverage overrides change active-rule presentation only. Passive Tabs leaves automatic focus and keyboard mapping, but visible parts remain pointer-interactive. A passive focused-row setting changes presentation only.
- **C35 (G484, G485, G491):** Hover reveal reserves centered close geometry but registers a hit only under its active, focused, hovered, or pressed conditions. Close targets default to one cell; even widths round down to odd and zero becomes one. Targets stay inside each tab and may overlap label or mark cells. They do not change tab sizing or reserved close-label space. Nonempty marks reserve two cells in fresh plans.

### Tree trailing marks

- **C36 (G499):** Include a trailing mark and following fill in the right-side cell budget. Normalize budgets below two to two. This narrows only the default `RowUi` lane; custom full-row renderers keep the full row.

## Editors and overlays

### Editor completion, blur, and find

- **C37 (G85, G91, G92, G95, G98, G99):** Use `replace_len` bytes before the cursor through the editor's grapheme-safe edit path. Blur commits by default. Disabling blur commit supports temporary editor removal, but explicit Cancel and focus-leaving commands still commit. Resolve selected completion by stable key only while its popup is open. Focus loss dismisses completion popups. Disabled `open_find` calls return ignored without opening find or dismissing completion. Enabled direct calls open find and focus the editor. Enabled `CompletingCodeEditor` calls dismiss completion and clear items before delegation; read-only editors may open find.

### Completion and prompt ownership

- **C38 (G508):** The caller owns the editor splice. The completion controller owns dismissal, including when the splice callback returns false.
- **C39 (G514):** Set the prompt's primary Enter key only to an action present in the action list.

### Menu and NoticeReveal layers

- **C40 (G526, G532):** Use `try_layer` for a menu anchored to another component's part. Missing owners or parts return `None`; fixed anchors retain `layer` behavior.
- **C41 (G537, G538):** Call NoticeReveal update from app updates, including ticks and settles. Require the full 300 ms hover rest; update schedules repaint at its deadline. It stores a stable part identity, revision, and time; draw resolves the borrowed Notice each frame. Reset it when its page or Notice becomes inactive. A scheduled repaint after reset is harmless; start a new delay only from a current source row. Current code tracks PAGE hover only when there is no active capture. All-layer hover remains open under T15.

### Picker builder order

- **C42 (G541, G543):** Apply `Picker::layout` before `max_rows`; `max_rows` then overrides the policy row limit. A later layout replaces earlier picker-level row-limit settings. A zero-row footer hides hint painting but keeps the hint string.

## Viewport and event routing

### TextViewport paint and navigation

- **C43 (G550):** Store supplied run styles. A grapheme crossing runs uses the first scalar's run. Selection foreground and background remain authoritative while producer modifiers and underline compose. Rebuild styled projections when theme or paint context changes. Do not use source revision as a theme-style cache key.
- **C44 (G582, G583, G586, G588):** Horizontal pan is off by default and applies only without wrapping. Plain arrows belong to panning only when no producer caret owns them. Remap semantic pan actions through public action keys, not raw-key fallbacks. Use `TextViewport::layout_revision` when borrowed content changes without a state-generation or dimension change. A saturated `layout_revision` disables layout reuse. Keep this value separate from routed `source_revision` identity. Unfocused carets remain subject to follow, selection, viewport, and clipping rules. They do not make the viewport editable or change focus styling. One click selects a word only when enabled; double click selects a word and three or more clicks select a logical line.

### Existing maintained mappings

The following archived groups already map to maintained clauses. The ledger binds each mapping to its exact source attachment and destination excerpt.

- **C45 (G26, G186, G326, G432, G467, G535, G539, G589, G593):** Retain the matching clauses in [authoring: event routing and committed layout](https://github.com/tailrocks/terminal-components-claude/blob/fe032a24154819a579c591e1cf87230558d286ee/docs/contracts/authoring.md#event-routing-and-committed-layout).
- **C46 (G516):** Retain the matching rule in [authoring: total clipped composition](authoring.md#total-clipped-composition).
- **C47 (G341, G342, G345, G346):** Retain the matching rules in [Form lanes, stage 1](../architecture/form-lanes-stage1.md#form-lanes-stage-1). G344's legacy-estimate exclusions are recorded in C11.
- **C48 (G591):** Retain the stale-revision rule in [foundation runtime contract](https://github.com/tailrocks/terminal-components-claude/blob/71c13bdaf1ce4f6fe33717a16ff3dc757c2e37a3/docs/contracts/foundation-runtime.md#wheel-routing-and-scroll-windows). Check the live revision before layout preparation or scroll-state mutation. Leave missing or stale attempts unacknowledged and unchanged.

### Approved retirement

- **C49 (G581):** Do not document `TextViewport::selection_binding_chords` as a current API. Its testing-only accessor was retired under `T37-SELECTION-BINDING-01`.

## Open requirements and proof limits

CodeEditor Cancel continues to commit when `commit_on_blur(false)` is set. Any intended rollback promise remains an open T17 requirement.

The current fixed Roles gate accepts printable ASCII or `·` and checks terminal-cell width. Broader Unicode support remains open under T07/T28. This transfer does not change source.

NoticeReveal currently observes PAGE hover only without active capture. Any broader layer-observation promise remains an open T15 requirement.

These statements describe inspected source; they do not verify runtime outcomes. Use an approved test plan before adding behavior evidence.
