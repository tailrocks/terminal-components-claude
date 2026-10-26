# Component caller contracts after T37

## Status and evidence

This guide preserves selected caller requirements from Rustdoc removed during T37. It records source-backed constraints, not behavior proof.

The removed attachments come from source `c817b99e9e43abbbf3519d56b2f5ee86924243d8`, product source `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`. The target source is `cfe0587ad034d9f0fd18694934f071f6f50c7dc5`.

The raw archive has 722 zero-based groups. The [transfer ledger](../implementation/pr38-final/results/t37-component-api-transfer/transfer.json) records each selected group, exact attachment, hashes, and disposition.

A Sol high review, `T37-COMPONENT-PROSE-01`, audited all 722 groups and approved this grouped scope. Its review decision identifies existing contract mappings and unresolved limits.

No behavior cases were selected. Behavior remains `NOT VERIFIED`. No Cargo command, build, or test ran for this transfer.

The source locations below identify removed caller prose. The ledger records retained source file hashes at the target revision and exact maintained excerpts.

## Focus registration and input ownership

### Button focus policy {#button-focus-policy}

- **C01 (G5):** Keep `ClickOnly` pointer hit-testable and pointer-activatable, but exclude it from keyboard traversal and activation. `Disabled` suppresses both.

### Form-clipped native controls {#form-clipped-native-controls}

- **C02 (G6, G20, G25, G27, G325, G332, G416, G549):** When Form clips a native body, use its keyboard-only registration helper. Preserve native disabled and read-only guards, including inherited Form disabling. Keep empty radio groups inert.

### Text input focus and command ownership {#text-input-focus-and-command-ownership}

- **C03 (G309, G313):** Pointer focus leaves initial placement to the pointer gesture. A retained draft keeps its caret and selection. Preserve a selection when a context-menu target lies inside it; otherwise place the caret at the target.
- **C04 (G320):** Exclude owner-routed editing commands from the field binding table. Keep field rendering and state ownership with the editor.
- **C05 (G302):** Update field state before acknowledging the exact layer opening that moves focus away. Process the matching continuation result before acknowledging that outcome.

### TextArea coordinates and drag {#textarea-coordinates-and-drag}

- **C06 (G329, G330, G334, G336):** Use shared body-window and text-part geometry for pointer mapping. A captured drag clamps above the visible window and extrapolates below it. Report caret rows relative to the logical visible body after document scrolling. Preserve full logical viewport and document row numbering when drawing through a parent window.

## Split controls and forms

### Split sizing and resize policy {#split-sizing-and-resize-policy}

- **C07 (G43–G50, G68, G71):** The last sizing-mode builder wins. Fixed detail widths floor; fixed master widths round to the nearest cell; both clamp to configured bounds. Resizable cell widths seed state until manual resizing. Normalize a zero resize step to one. An empty resize binding table disables keyboard resizing but keeps seam dragging. Preserve the configured drag-grab offset only when enabled. The default reset policy is balanced.

### Responsive MasterDetail geometry {#responsive-masterdetail-geometry}

- **C08 (G53):** Split mode returns empty pane rectangles from responsive summary resolution. Call `layout` when callers need actual split regions.
- **C09 (G601, G602):** The facade's resizable master share seeds only uninitialized split state. Preserve caller state during resize and redraw. A retained fixed-detail percentage does not report the resizable initial share.
- **C10 (G609):** Close a narrow drawer only for the scoped bubble binding routed after focused detail declines Escape.

### Form measurement and lanes {#form-measurement-and-lanes}

- **C11 (G344):** Treat legacy `measure` as an estimate. It ignores data-dependent visibility, radio-option counts, paired `columns(2)` rows, and explicit lanes. Use `measure_with_data` for live data or intrinsic sizing.
- **C12 (G368, G369):** Give each stable lane ID its own scroll state. Use one vertical lane or two lanes with one horizontal split; Form preflight rejects additional lanes.
- **C13 (G357, G360):** The default Enter policy submits when a field is not editing. Use `Never` when a caller submits through a separate scoped command. Run `update` first on the same frame. Call scoped submit after its command bubbles past the focused child. Then commit the active draft, validate through Form, and apply dialog submit policy.

### Pointer-held Form reveal {#pointer-held-form-reveal}

- **C14 (G343):** Hold automatic reveal while a child pointer gesture is live, so its hit target stays fixed between press and release. Start on press; end on release, drag-end, or cancellation. Use `PRESSED` only to clear a stale hold without a terminal intent.

### Isolated List focus loss {#isolated-list-focus-loss}

- **C15 (G429, G433):** Use the helper only for isolated `Settle` focus loss. After `None`, run ordinary update on the same component, state, and pass. Inspecting the bucket does not claim unclaimed intents. Empty buckets return an ignored response. List clears its click cursor; NavList has no focus-loss state.

## Measurement, windows, and geometry

### Measurement inputs {#measurement-inputs}

- **C16 (G63):** Measure PropsList from live rows and keyed state. Use its shared update/draw geometry for wrapping and copy-hint reservation.
- **C17 (G441, G442):** Measure Steps with a pure label-width projection when a custom row needs intrinsic width. Otherwise use intrinsic width when available, then the 12-cell minimum. Do not invoke row painters or state projections during measurement.
- **C18 (G395):** Pass the grid body's resolved chrome to its scroll bars. Do not recompute bar alignment from column widths.

### Window preflight and caller-owned chrome {#window-preflight-and-caller-owned-chrome}

- **C19 (G60):** Validate Panel window geometry before drawing. A partial window with a legacy slot fails before paint, decoration, or callback. Full windows retain `draw` behavior.
- **C20 (G127–G130):** Keep Accounts header and footer outside pane geometry. Use physical master rows and preserve the detail logical origin, scroll viewport, and physical clip separately. Keep title and action rows outside that clip. The explicit shell painter leaves detail windows, scrollbars, and input to its caller; the legacy whole-page path stays 39 columns. Window painters keep logical anchors and omit cut graphemes whole. Return the visible original source-row range. Leave row backgrounds to the caller. Invalid geometry returns `None` before paint. Row projection returns full text and styles or a typed error.
- **C21 (G137–G139):** Supply Inspector wrapping widths and scroll state. Window offsets count visual rows, not source lines. Use the measured row count for caller scroll state. Keep text anchored at the logical left edge; clipping may reduce its right edge. The caller owns outer chrome, backgrounds, scrollbars, fades, input, and scroll state. Invalid geometry returns `None` before paint; valid ancestor-hidden geometry returns an empty range.

## Feedback and fixed Settings painters

### Notice row limits {#notice-row-limits}

- **C22 (G176, G177):** Normalize a zero Notice message-row cap to one. A zero detail-row cap suppresses detail.

### Safe values and fixed body limits {#safe-values-and-fixed-body-limits}

- **C23 (G210):** Pass only a star count for a plain secret value. Treat provider paths and environment names as public display references, not resolved secrets.
- **C24 (G228, G276–G278, G280–G284):** The live Editor role view carries at most 29 visible rows and the full source count. Its fixed key/value/source or role columns require 120 columns; its frame and footer require 40 rows. Require printable single-cell display strings. Visible role rows cannot exceed the source count.
- **C25 (G285–G287):** Validate every fixed body row and environment Add action before painting. Require empty sections, mounts, and add-mount content for whole-page Env, Agents, and Trust painters. The full-screen environment body reserves rows 6–36; the footer starts at row 37. The role body supports at most 29 rows. Every environment row must fit before the footer. These entry points do not scroll, drop rows, paint controls, or own input. Fixed Env columns clip to terminal-cell width.
- **C26 (G288, G289, G291, G293):** Agents bodies support six rows and two notes. Trust bodies support four rows and two notes. Validate capacities before painting. Fixed content-band painters clip to page-relative bands and do not paint chrome. Agents painters do not handle input or focus. Trust painters do not handle counts, input, focus, or caller save previews.
- **C27 (G292):** The Mounts painter borrows rows, clips to the body band, and owns no page chrome, actions, hints, input, or focus.
- **C28 (G265–G267):** Provide one fixed offset per hint. Choose one hint-tail owner. Supply a running-context cluster when a breadcrumb note uses its fixed anchor.

## Tabs and navigation

### Tabs configuration and plan lifecycle {#tabs-configuration-and-plan-lifecycle}

- **C29 (G460, G476):** Validate effective action identities and chord conflicts across the selected navigation profile and replacement groups. A profile changes only Prev/Next descriptors. Keep the same descriptors for publication and dispatch.
- **C30 (G474, G495, G496):** A fresh nonempty default-row plan captures formatted item segments, marks, eligibility, and mapped affixes, including off-window items. Rebuild after item data or mapper changes. `previous` is the caller-owned receipt from the last committed draw. Drawing the plan does not call item formatting or mark callbacks.
- **C31 (G497):** Rebuild a captured plan after every state or data change. `draw_layout` checks component identity and rendering configuration; it does not check caller data freshness. Rebuild on either mismatch before drawing.
- **C32 (G470, G471, G475):** Treat allocated and clipped part bounds as geometry, not live hit permission. The committed hit registry determines current interaction visibility.

### Tabs bindings and presentation {#tabs-bindings-and-presentation}

- **C33 (G480, G481, G492, G493):** `None` retains base New or Close bindings. `Some(&[])` removes that command's keyboard bindings without changing eligible pointer actions or Middle behavior. Validate the full potential table even when gated or passive. Accessors return empty supplemental tables when commands are gated or Tabs is passive. Publish only supplemental tables when replacing base entries. Use `new_key_table` or `close_key_table` for manual publication.
- **C34 (G488–G490):** Coverage overrides change active-rule presentation only. Passive Tabs leaves automatic focus and keyboard mapping, but visible parts remain pointer-interactive. A passive focused-row setting changes presentation only.
- **C35 (G484, G485, G491):** Hover reveal reserves centered close geometry but registers a hit only under its active, focused, hovered, or pressed conditions. Close targets default to one cell; even widths round down to odd, zero becomes one, and targets stay inside each tab. Nonempty marks reserve two cells in fresh plans.

### Tree trailing marks {#tree-trailing-marks}

- **C36 (G499):** Include a trailing mark and following fill in the right-side cell budget. Normalize budgets below two to two. This narrows only the default `RowUi` lane; custom full-row renderers keep the full row.

## Editors and overlays

### Editor completion, blur, and find {#editor-completion-blur-and-find}

- **C37 (G85, G91, G92, G95, G98, G99):** Use `replace_len` bytes before the cursor through the editor's grapheme-safe edit path. Blur commits by default. Disabling blur commit supports temporary editor removal, but explicit Cancel and focus-leaving commands still commit. Resolve selected completion by stable key only while its popup is open. Focus loss dismisses completion popups. Opening find dismisses the popup first and focuses the editor. Read-only editors may open find; disabled editors ignore it.

### Completion and prompt ownership {#completion-and-prompt-ownership}

- **C38 (G508):** The caller owns the editor splice. The completion controller owns dismissal, including when the splice callback returns false.
- **C39 (G514):** Set the prompt's primary Enter key only to an action present in the action list.

### Menu and NoticeReveal layers {#menu-and-noticereveal-layers}

- **C40 (G526, G532):** Use `try_layer` for a menu anchored to another component's part. Missing owners or parts return `None`; fixed anchors retain `layer` behavior.
- **C41 (G537, G538):** Call NoticeReveal update from app updates, including ticks and settles. It stores a stable part identity, revision, and time; draw resolves the borrowed Notice each frame. Reset it when its page or Notice becomes inactive. A scheduled repaint after reset is harmless; start a new delay only from a current source row. Current code tracks PAGE hover only when there is no active capture. All-layer hover remains open under T15.

### Picker builder order {#picker-builder-order}

- **C42 (G541, G543):** Apply `Picker::layout` before `max_rows`; `max_rows` then overrides the policy row limit. A later layout replaces earlier picker-level row-limit settings. A zero-row footer hides hint painting but keeps the hint string.

## Viewport and event routing

### TextViewport paint and navigation {#textviewport-paint-and-navigation}

- **C43 (G550):** Store supplied run styles. A grapheme crossing runs uses the first scalar's run. Selection foreground and background remain authoritative while producer modifiers and underline compose. Rebuild styled projections when theme or paint context changes.
- **C44 (G582, G583, G586, G588):** Horizontal pan is off by default and applies only without wrapping. Plain arrows belong to panning only when no producer caret owns them. Remap semantic pan actions through public action keys, not raw-key fallbacks. Advance source revision for changed borrowed content at unchanged state generation and dimensions; saturation disables layout reuse. Unfocused carets remain subject to follow, selection, viewport, and clipping rules. They do not make the viewport editable or change focus styling. One click selects a word only when enabled; double click selects a word and three or more clicks select a logical line.

### Existing maintained mappings {#existing-maintained-mappings}

The following archived groups already map to maintained clauses. The ledger binds each mapping to its exact source attachment and destination excerpt.

- **C45 (G26, G186, G326, G432, G467, G535, G539, G589, G593):** Retain the matching clauses in [authoring: event routing and committed layout](authoring.md#event-routing-and-committed-layout).
- **C46 (G516):** Retain the matching rule in [authoring: total clipped composition](authoring.md#total-clipped-composition).
- **C47 (G341, G342, G345, G346):** Retain the matching rules in [Form lanes, stage 1](../architecture/form-lanes-stage1.md). G344's legacy-estimate exclusions are recorded in C11.
- **C48 (G591):** Retain the stale-revision rule in [foundation runtime: wheel routing and scroll windows](foundation-runtime.md#wheel-routing-and-scroll-windows). Check the live revision before layout preparation or scroll-state mutation. Leave missing or stale attempts unacknowledged and unchanged.

### Approved retirement {#approved-retirement}

- **C49 (G581):** Do not document `TextViewport::selection_binding_chords` as a current API. Its testing-only accessor was retired under `T37-SELECTION-BINDING-01`.

## Open requirements and proof limits

CodeEditor Cancel continues to commit when `commit_on_blur(false)` is set. Any intended rollback promise remains an open T17 requirement.

NoticeReveal currently observes PAGE hover only without active capture. Any broader layer-observation promise remains an open T15 requirement.

These statements describe inspected source; they do not verify runtime outcomes. Use an approved test plan before adding behavior evidence.
