# Improvements plan

Status: **research and planning complete; independent final review approved.**
Implementation is **not** part of this document's completion claim.
Scope: current `holla-fable` worktree, inspected 2026-09-10. Existing uncommitted
audit fixes are the baseline; no other branch or history was consulted.

## Decision

Repair ownership boundaries first: modal input, destructive action eligibility,
collection identity, read-only transitions, and retained text. Then make existing
capabilities reachable and keyboard-complete. Preserve Junie's geometry-first
focus, restrained planes, sparse framing, contextual hints and semantic color.
No new widget, framework, dependency or palette is approved by this plan.

The initial audit's completed fixes remain protected regressions, not work to
repeat. Fresh tests found additional defects; passing the existing suite does
not make those defects acceptable. Priorities express correctness and impact,
not effort or commercial value:

- **P1:** panic, leaked terminal state, unintended destructive action, lost/wrong-source copied text,
  or mutation violating the active interaction owner/read-only contract.
- **P2:** incorrect identity, text, interaction, layout, performance, reusable
  contract or required evidence; keyboard and documented-state coverage gaps.
- **P3:** optional integration/abstraction without a demonstrated current need.

Every item below distinguishes an executable reproduction, source-confirmed
absence, architectural explanation, and proposed design. A proposed fix is not
itself verified until its acceptance tests pass.

## Verified baseline and evidence boundaries

The [compiler-derived inventory](docs/tui-audit-inventory.md) covers the existing
31 widget modules, primitives, interaction states, public API and four consumers:
Showcase, TablePro, Jackin Preview and Holla. Final inventory: **1,573 source-backed
exported entries**, including all 37 production `TextBuffer` methods; **572
derived contracts** counted separately. The earlier regex count of 849 was
incomplete and must not be reused. The final catalogue has **23 showcase pages**,
including the already-existing `DiffView`; no new widget was added.

Fresh planning-pass verification reran formatting, Clippy, tests and Rustdoc:
**328 tests pass across eight suites**. The 460 showcase baseline cases cover
23 pages × five sizes × four palette levels. Disk evidence contains 250 representative
capture sets and 51 interaction sets, each with ANSI/text/cursor/HTML/PNG files.
Those counts establish artifact presence, not exhaustive interaction or visual
correctness. Existing final captures were sampled again; selected failing flows
were independently recaptured in isolated terminals.

The previous audit's 100,000-operation Unicode stress run, before/after timing
and controlling-PTY restoration checks are historical diagnostic evidence, not
retained automated tests. Retained Enter coverage tests Ctrl+Enter, not every
modifier combination. Facts acknowledgement coverage types individual keys,
not a paste event. Editor wheel tests do not establish editor scrollbar-drag
coverage. The plan adds these missing proofs rather than repeating broader
claims. Unicode search uses Rust lowercase mapping, not full Unicode case folding
or canonical normalization.

PNG rasterization is an independent implementation with known grapheme-width
and font limitations. Buffer/cursor/copy assertions prove the application model;
emitted ANSI proves backend semantics; real terminal inspection proves the
terminal/font path. None substitutes for all three. Actual `NO_COLOR=1` remains
separate from selecting the library's monochrome palette.

## Previous findings: disposition map

The [original audit](docs/tui-audit.md), [interaction report](docs/tui-audit-interactions.md),
[editor report](docs/tui-audit-editor.md), [inventory](docs/tui-audit-inventory.md)
and [initial ecosystem research](docs/tui-audit-research.md) are historical
records. Their original defect descriptions do not imply those fixes are still
missing. The new prior-fix verification ledger gives current symbols, retained
regressions and corrected claim scope for each row.

| Previous finding family | Current disposition / continuing obligation |
| --- | --- |
| TextBuffer UTF-8 selection panic, public offsets, grapheme word movement, atomic replacement, joined clusters, line-ending normalization | Fixed; preserve shared invariants across every editor. Retain the previously temporary stress proof as an automated deterministic test. |
| CodeEditor modified Enter, transformed search offsets, fuzzy Unicode matching, find paste/backspace, stale matches after `set_text` | Fixed; retain exact original-source grapheme mapping and add the missing input-modifier/real paste matrix. |
| CodeEditor tiny gutter/find footer, readonly focus, mid-edit read-only indentation | Fixed in CodeEditor; do not assume other editors inherit its guards. New grid transition finding is separate. |
| DataTable replacement edit state and source-row edit identity; invalid cell/header transitions in table/grid | Fixed for recorded flows (inventory API-1). Collection cursor/anchor issues below are distinct. |
| Input validation recovery, masked clicks, tiny allocations; TextArea horizontal editing | Fixed; retain keyboard, paste, Unicode, cursor and containment assertions. |
| TextArea/CodeEditor manual scroll preservation | Fixed for retained wheel tests; add explicit scrollbar-drag proof. |
| Terminal partial-startup restoration and queued Changed-event freshness | Fixed for demonstrated setup/dispatch paths; lifecycle coverage must state which exits/signals were actually exercised. |
| Forms minimum-size layout and sidebar reachability | Fixed; other showcase sections still fail reachability, described below. |
| Semantic focus gutters, monochrome reverse selection/disabled DIM, actual NO_COLOR backend resets | Fixed; retain palette and fresh-process emitted-ANSI tests. |
| Choice containment/empty geometry, footer status reservation, duplicate Tab hints | Fixed; add executable state/containment assertions before any new baseline approval. |
| First/second click, exact drag press anchor, Facts acknowledgement click routing | Fixed for named flows; app-level paste ownership is not covered by these fixes. |
| TablePro Ctrl+D collision | Fixed with Alt+D duplication alias; keep Data/Structure chord. Broader binding metadata remains a bounded architecture question. |
| Idle TextViewport cache thrash | Fixed; dirty append/replacement cost and public mutation invalidation are different remaining problems. |
| Diff showcase/catalogue contradiction, mode cache, narrow review fallback, Old/New headings | Fixed (inventory API-3/API-4); no second diff component. |
| Table/grid Unicode edit windows and click offsets; Diff grapheme emphasis/tab/scrollbar alignment; one-cell wrap overflow | Fixed for named primitives. Generic styled viewport span boundaries remain a separate defect. |
| Inventory API-2: collection invariants split between caller and widget | Promoted from architectural risk to reproduced supported-flow list panic and identity findings below. |
| Inventory API-5: duplicated shell pointer/focus ownership | Keep typed events and existing barriers. New modal routing failures justify owner-boundary work, not a universal Widget/Container rewrite. |
| Keyboard creation of viewport selections | Still absent; extend existing TextViewport and inherit through DiffView. |
| Real async workers/cancellation, Widget/Theme traits, configurable glyphs | Still conditional opportunities. Concrete consumer proof required before implementation. |

Original reports sometimes assigned different priorities to the same defect.
This plan's definitions and current consequences govern execution; duplicates
are one work item, not independent defect counts.

## Execution backlog: ownership and identity

### F01 — Modal-first paste routing

**P1 · confirmed defect.** `src/bin/tablepro/app.rs:236`, `App::handle`,
routes paste to Dialog and Filter only; other modals fall through to the screen.
Actual App/TestBackend reproduction: connect Production, focus Query, enter
editing, Ctrl+O, paste SQL. Picker stays empty while the hidden query changes.
Interaction and editor agents independently reproduced the same owner violation.

Root: modal hit/focus barriers do not own all input classes, and render-time
focus repair is too late to prevent a hidden editor receiving an event.
Make the active modal the exhaustive first recipient of paste. A modal either
dispatches to its active child or consumes unsupported paste; never fall through.
Reconcile edit/focus transitions at opening/closing ownership boundaries, not
only when an old control happens to render. Reuse current modal variants and
typed outcomes. Do not replace the complete event system.

Risk: medium routing/focus risk. Acceptance: open every TablePro modal over an
editing query/grid, send paste before and after its first render, and assert
only the active eligible child changes. Test queued and delayed input, close/
cancel focus restoration, hidden/removed opener, keyboard/mouse opening, and
ordinary document paste without a modal. Preserve drafts unless the existing
explicit commit/cancel contract says otherwise.

### F02 — Picker action eligibility and destructive target identity

**P1 · confirmed defect.** `Picker::on_key`, `src/widgets/picker.rs:153/195`,
guards Enter but emits `Secondary(cursor)` for Delete on empty, disabled and
loading results. `src/bin/tablepro/app.rs:1502` uses display detail as a tab index
and falls back with `unwrap_or(i)`. Actual flow: connected TablePro → Ctrl+G →
unmatched query → Delete closes Query1 despite “No matches”. API probes and an
independent real-terminal design replay both confirm it.

Root: action eligibility differs by input path, and absent identity becomes an
unrelated valid target. Share an eligible-item resolver across primary,
secondary and pointer actions. Keep tab identity in an explicit owner mapping,
not display text. Missing/stale mappings must reject action. Fix both boundaries;
a widget-only guard leaves destructive owner fallback unsafe.

Risk: medium because tab close can discard state. Acceptance: empty, loading,
error and disabled results emit no item action; valid filtered Delete closes
exactly the selected tab. Insertion/removal while the picker remains open must
not retarget a result. Query reset still deliberately selects the first eligible
result; do not impose blanket cursor preservation on filtering.

### F03 — Atomic ListBox mutation

**P1 · confirmed defect.** `ListBox::move_to`, `src/widgets/list.rs:90`, uses a
private range anchor. Settings removal at `src/bin/showcase/pages/settings.rs:579`
replaces public items/checks/cursor but cannot repair that anchor. Reproduction:
Environment list, select zero-based rows 3–4, Remove selected, return to list,
Shift+Up. Current app panics at an out-of-bounds checked index. API App probe and
independent real-terminal replay agree.

Root: no owner can atomically maintain all collection invariants. Introduce
replacement/removal operations owning items, checks, cursor, chosen item,
selection anchor and scroll. Migrate Settings add/remove callers. Choose reset
versus semantic preservation explicitly; bounds clamping alone can still select
the wrong surviving item.

Risk: medium compatibility risk. Recommended end state: invariant-bearing
storage cannot be independently mutated. Stage accessors/checked operations and
migrate consumers before a deliberate visibility change. If public unchecked
writes remain for compatibility, record that residual API weakness; do not call
the bug class eliminated. Acceptance: actual flow regression plus empty/growing/
shrinking collections, removal around both range endpoints, disabled rows,
mouse removal, chosen/check-count consistency and scroll clamping.

### F04 — Reconcile DataGrid read-only transitions before mutation

**P1 · confirmed API defect; current app transition not established.**
`DataGrid::begin_edit` checks `editable`, but `commit_edit` and an existing edit's
paste path do not recheck it (`src/widgets/grid.rs:539/590`). Public sequence:
begin edit → `editable = false` → paste → commit still emits `CellChanged` and
writes pending data. Text and interaction agents independently reproduced it.
CodeEditor's prior fix does not protect this separate implementation.

Root: permission is checked only at transaction entry. Centralize current
mutation eligibility and reconcile active edit state whenever grid/column
read-only state changes and at every mutation boundary. Define whether the
draft is cancelled or retained read-only; never silently commit forbidden data.
Reuse current edit/pending structures and explicit setter/accessor migration.

Risk: medium transaction/API risk. Acceptance: flip grid and column permissions
mid-edit, then exercise paste, characters, Tab/BackTab, Enter, blur, click, direct
commit and render. Data/pending state must remain unchanged and no write event
may escape. Existing read-only navigation, selection/copy and valid edits remain.
This is UI mutation correctness, not proof of database authorization security.

### F05 — Preserve TreeView target across lazy insertion

**P2 · confirmed defect.** `TreeView::set_children` calls `flatten`, which keeps
only a clamped numeric cursor (`src/widgets/tree.rs:145/203/227`). Inserting
children above a focused sibling moves the cursor to a new child although the
old node survives. TablePro's `Workbench::tick_explorer` is a present delayed-load
consumer. API and independent design probes agree.

Root: flattened display position is treated as node identity. Capture the
focused path before rebuild, resolve it afterward, and define fallback for a
removed/hidden target (visible ancestor, then surviving neighbor). Keep cursor
and selected path distinct; do not steal focus when delivery occurs elsewhere.
Risk: medium collapse/filter behavior risk. Acceptance: delayed nested loads,
focus on later siblings, collapse, filters, empty children and keyboard/mouse
toggle paths preserve or deliberately relocate the target and reveal it.
Positional paths do not solve arbitrary sibling reordering; that remains a
separate owner-identity contract, not justification for a global key framework.

### F06 — Preserve DataGrid cursor identity through local sort

**P2 · design inconsistency with confirmed target change.**
`DataGrid::request_sort`, `src/widgets/grid.rs:1176`, changes `order` without
remapping cursor/range anchor. Supported `s` on rows beta/alpha changes the
focused value beta → alpha without navigation. TablePro query results enable
local sorting; DataTable already preserves source identity. API and verification
agents independently reproduced the mismatch, including pending-value sorting.

Preserve the cursor's source row through ascending/descending/unsorted order.
Explicitly clear or reconcile rectangular display selections; retain source-keyed
checks and pending edits. Risk: medium selection semantics. Acceptance: duplicates,
all sort directions, keyboard/mouse selection, invalid drafts and pending changes;
the same record remains focused and no range silently acquires unrelated rows.
Do not promise globally sorted unloaded data: local-sort and fetch-more contracts
must remain distinct.

A related API probe changes beta to pending aardvark, then sorts ascending;
display reads alpha, aardvark because sorting compares stored rows rather than
effective displayed values. Resolve this policy explicitly: sort effective
values, or label a deliberately committed-data order. Acceptance must include
pending edits and null/type ordering; do not accidentally clear pending changes.

### F07 — Checked row/schema ingestion

**P2 · architecture/API weakness; malformed-input panic reproduced, no current
app producer proven.** `DataTable::new/set_rows` accept ragged vectors while
sorting indexes `rows[a][col]` (`src/widgets/table.rs:136/209/225`). Two columns
with one-cell rows panic when sorting column 1. DataGrid missing-cell reads use
Null, but local sorting indexes directly (`src/widgets/grid.rs:428/502`).

Root: the accepted data shape is neither enforced nor consistently interpreted.
Define rectangularity and index validity at ingestion; prefer checked rejection
that leaves the previous dataset/edit transaction untouched. Do not silently
equate absent cells with SQL Null. Coordinate schema replacement with active
edits and pending/source identities. Risk: medium public API compatibility.
Acceptance: zero columns, empty/short/extra rows, schema changes during editing,
and invalid source references produce documented errors or deliberate normalization;
well-formed behavior is unchanged. Additive checked APIs are a migration step,
not a complete root fix while unchecked storage remains writable.

Evidence and detailed probes for F02/F03/F05–F07:
[API verification](docs/plan-api-verification.md), with independent user-flow
checks in [design verification](docs/plan-design-verification.md).

### F08 — Complete existing editing and binding contracts

Four related but independently testable items; none needs a new input widget.
Detailed evidence: [interaction verification](docs/plan-interaction-verification.md).

| Item | Classification / evidence / impact | Root fix, risk and acceptance |
| --- | --- | --- |
| F08a · Between upper-value paste | P2 confirmed defect. TablePro `App::handle` at `app.rs:240` checks only `FilterEditor.value`, unlike key/click paths iterating `value` and `value2`. Actual rendered filter probe consumes paste while upper value is editing and leaves it empty. | One active-input resolver over the same field set for keys/paste/clicks; reuse F01 modal routing. Low risk. Focus each bound by keyboard and mouse, paste distinct values, apply and assert both; hidden upper field and action focus receive no edit. |
| F08b · Picker grapheme editing/query paste | P2 confirmed deletion defect + coverage gap. `picker.rs:199` uses `String::pop`: Backspace on 👩‍💻 leaves woman+ZWJ. No query-paste entry point; owners consume paste or leak it through F01. | Reuse TextBuffer/grapheme operations and add query paste emitting existing `QueryChanged` exactly once. Medium owner/API risk. Typed/pasted Unicode queries produce identical rows; complete-cluster Backspace, clear, disabled/search-disabled/loading/error and escape behaviors are tested in widget and real owner. Coordinate F02, but keep destructive eligibility separate. |
| F08c · Ctrl+Shift+Home/End | P2 confirmed defect. `field_common.rs:78/79` calls document movement with `false` rather than Shift; TextArea probe moves to byte 0 with no selection. Shared by code/text/cell editing. | Propagate Shift at shared action boundary. Low risk. New/existing/reversed selections extend through Unicode document endpoints; unshifted navigation, empty text, line Home/End and all supported owners remain correct. |
| F08d · Unassigned modifiers perform plain actions | P2 architecture/API weakness. DataTable Ctrl+S sorts at `table.rs:438`; several navigation controls match code without modifier ownership. TablePro intercepts Save, so this is not a proven broken TablePro Save flow. | Define a chord/precedence matrix before changes; explicit bindings act, unassigned nonmodal chords return Ignored, top modal may consume without unrelated action. Medium compatibility risk. Preserve Shift ranges and Picker Ctrl+J/K/N/P; test Save/Run/Find/Close/detach through real owners. `Key::plain` intentionally permits Shift; adding it everywhere is not the complete policy. |

Key hints and matching remain separate data (`widgets/keyhint.rs::Hint`). Once
the matrix exposes duplicated actual bindings, a small shared descriptor may
drive both label and match. Preserve `HintBar` layers, typed events and owner
precedence; no universal command bus. Test advertised action reachability in the
state displaying its hint, not merely absence of duplicate labels.

### F09 — Deliver Inspect copy without closing the modal

**P2 · confirmed defect by complete source trace, not full-app replay.**
`InspectChanges` advertises `y Copy` but discards `DiffView::on_key` events at
`src/bin/jackin_preview/screens/inspect.rs:250/343`. `CustomModal` lacks a
nonclosing effect route; `Request::Copy` already owns preview clipboard updates
at `src/bin/jackin_preview/app.rs:1964`. Interaction agent and main independently
traced the entire missing path.

Add a narrow typed nonclosing effect/request path through the existing custom
modal interface. Forward copy to the existing owner, preserve selection/focus
and keep Inspect open. Risk: medium internal interface risk; inspect every
CustomModal implementation and preserve close/done semantics. Acceptance:
compact/open-file and advanced/diff mouse selection → y delivers exact text,
increments preview clipboard generation once and shows truthful feedback without
closing. Empty selection does not fabricate a copy. Later keyboard selection
uses the same path. This must not introduce OS clipboard writes.

## Execution backlog: text ownership and live output

### F10 — Segment logical text before applying styles

**P1 · confirmed text-loss defect.** `TextViewport::ensure_layout`,
`src/widgets/viewport.rs:305`, segments each span separately and drops zero-width
pieces. One span `👩‍💻X` puts X at column 2; split styled spans put X at 4.
Separate spans `a`, combining acute, `X` copy `aX`, losing the stored accent.
Text and interaction agents independently rendered and copied both cases.

Root: style boundaries incorrectly define grapheme boundaries. Segment a whole
logical line, map source byte ranges to complete graphemes, then resolve styles
with a documented deterministic precedence for a cluster crossing runs. Use the
same map for rendering, width, selection, caret and copy. Reuse current Line/Span
concepts; the previous Diff-specific span fix is not the generic root fix.

Risk: medium style/API compatibility. Acceptance: split/unsplit combining,
ZWJ, variation-selector and flag sequences retain identical text, widths and
copied ranges; only declared style precedence may differ. Test narrow clipping,
nonzero origin, mixed tones and existing Diff emphasis/geometry. Cache logical
graphemes with document revisions so correctness does not add idle reparsing.

### F11 — One retention-aware mutation boundary

**P2 · confirmed bounded-line contract defect.** `max_lines(3)` followed by
`set_lines(eight)` or applied after `with_lines(eight)` retains eight. Only push
enforces the limit (`viewport.rs:155/166/171/189`); empty replace-last also needs
zero-limit semantics. Holla Activity configures 4,000 and Plan 2,000 but populates
through `set_lines`. Supported API probes and present consumer traces agree.

Enforce the declared cap at construction, replacement, append, tail replacement
and limit changes, through one transaction returning the removed-prefix delta.
Pair with F12 so trimming cannot corrupt identity. Risk: medium visible-tail/API
change. Acceptance: limits 0/1/N, oversized batches, shrinking cap and Holla
integration beyond configured limits retain the correct tail. Define zero and
logical-line versus byte limits. No OOM or byte bound was demonstrated; this
plan uses P2 rather than the text report's P1, while keeping the violated cap
mandatory to fix. A line cap does not bound one arbitrarily long line.

### F12 — Rebase retained identity, never silently retarget selection

**P1 · confirmed wrong-source copy; P2 reading-position defect.**
`TextViewport::push`, `viewport.rs:171`, saturates selected line indices after
eviction without resolving removed identities; it omits drag and visible anchors.
Cap 3, A/B/C: select A, append D → copied selection becomes B. Press B, append D,
drag → wrong newline range. With follow off, visible B jumps to C although B
survives. Text and interaction agents independently reproduce all three.

Apply one edit delta/stable-line reconciliation to selection head/anchor, active
drag anchor, producer caret and first visible logical position. Clear entirely
evicted selections; define clipping of partially surviving ranges. Preserve
retained reading position while follow is off, then map back into wrapped rows.
Dataset replacement needs an explicit reset/identity contract, not only index
clamping. Risk: medium/high selection semantics. Acceptance: all three repros,
partial/multiline eviction, active drag, wrapped rows, resize, follow-on, empty
and zero retention. Existing passing logical selection across resize must remain.

### F13 — Make cache validity part of content ownership

**P2 · confirmed API/cache weakness, public-field mutation case.**
`TextViewport.lines` is writable but cache invalidation is private. Render OLD,
assign its span text NEW, render at same size → model NEW, buffer OLD
(`viewport.rs:111/245/305`). Text and interaction probes agree; this is distinct
from F11's supported setter failure.

Prefer read-only content access plus revisioned mutation methods/guards that
also own F11/F12. Migrate direct readers/writers from the inventory before
deliberate field-visibility changes. A mandatory caller revision token only
works if the API enforces or reliably detects its update; a private dirty flag
cannot be the caller contract. Risk: medium public compatibility. Acceptance:
every supported mutation updates text/copy/cursor next frame, including same-byte-
length edits; unchanged frames reuse cache. Do not rehash the whole history on
each frame as an unmeasured workaround or claim completion while silent direct
mutation remains possible.

### F14 — Explicit tab/control geometry and copy policy

**P2 · design inconsistency / architecture weakness.** Literal `A\tB` gives
different geometry in TextInput/TextArea/CodeEditor versus the viewport's existing
four-space expansion. `TextBuffer` normalizes CR/LF but stores other controls;
separate renderers have no shared declared display/copy rule. Text probes and
interaction peer verification agree. Ratatui filters control graphemes here;
these observations do **not** prove terminal escape injection.

Define source storage separately from display: tab expansion, visible treatment
of controls, source-to-cell mapping and copy-as-source versus copy-as-displayed.
Reuse the logical grapheme mapping from F10 and existing four-space viewport
policy as the default candidate. Literal document tabs are not the same as the
Tab key's configurable code indentation. Preserve data; no silent control-byte
deletion. Risk: medium/high public cursor-column semantics. Acceptance: the same
tab/ESC/BEL/CR/LF/CRLF/combining fixture across input, textarea, code, cell editors,
viewport and both Diff modes proves declared storage, cursor, click, clipping
and copy behavior. Use additive policy/mapping APIs during compatibility migration.

### F15 — Incremental live-output work, measured separately from idle redraw

**P2 · architecture/API weakness with measured performance impact.** `replace_last` dirties the
whole document; `ensure_layout` recreates all cells, potentially twice for final
scrollbar width (`viewport.rs:196/245/305`). A release probe's 20 updates of
80-character rows at 80×20 averaged approximately 4.47/34.41/172.49 ms per update
at 1k/10k/50k lines. Only the first replacement changes the contents; the remaining
calls supply the same line again, yet all dirty the cache. An independent repeat
observed the same scaling. These are single shared-host samples, not frame latency
or a promised budget. Holla also reconstructs complete filtered vectors on output
changes. The prior idle-cache fix remains valid and independently retested.

After F10–F13, separate logical-line parsing from width-dependent visual rows;
invalidate changed lines only, apply append/tail/batch deltas at producers, and
avoid reparsing source merely to resolve scrollbar width. Risk: medium/high stale
cache risk. Acceptance: fresh-layout equality and instrumented zero unchanged-line
reparses for a tail update; bounded retained state and correct anchors under churn.
Retain workloads for idle, append-at-cap, tail replacement, batch, resize, wrap,
selection and search at agreed sizes. Record allocations/reflows plus median/tail
timings, establish a representative latency target, then prove it; no arbitrary
threshold inferred from one sample. Preserve existing idle zero-reflow gate.

Full text evidence, temporary probe results and exact limits:
[text verification](docs/plan-text-verification.md). Temporary probes are current
evidence, not repository regression tests; implementation must retain them.

## Execution backlog: reachable, reusable interactions

### F16 — Reach every showcase section at supported sizes

**P2 · confirmed inaccessible-control defect plus coverage gap.** At 72×20,
TextAreas' fixed upper-card reservation leaves the enabled error field without
usable geometry; three Tabs visit only Task description/Notes then navigation.
At 100×30, Inputs exposes only three of eight state references. Buttons' matrix
is vertically/horizontally truncated without a reveal route. Locations:
`pages/textareas.rs:64`, `inputs.rs:123`, `buttons.rs:72/150`, under
`src/bin/showcase`. Design captures/replay and ecosystem source cross-check agree.

Root: layout truncation has no interaction owner for omitted sections. Use existing
Tabs/Select for compact section switching, retaining stacked/wide composition
when it fits. Preserve drafts and restore focus per section on resize. A shared
app helper is justified after two pages exercise identical semantics. Existing
ScrollPanel/TextViewport display text; neither is an arbitrary-child scrolling
container. Risk: medium focus/draft/resize behavior. Acceptance: every enabled
control and documented reference state reachable by keyboard/mouse at all five
sizes without resizing; hidden sections keep drafts but no stale hits/cursors.
Test lower-field editing through shrink/grow and independent wheel ownership.

### F17 — Empty allocations emit nothing

**P2 · confirmed containment defect.** Buttons draws State matrix headers without
checking positive inner height (`pages/buttons.rs:151`). At 72×20 Primary/Secondary
appear outside the zero-height allocation in the blank row below the page.
The row loop is bounded, separate metadata/header writes are not. Design image
review and ecosystem source verification agree.

Bound every composed section, including headers, metadata, hit regions and
cursor, at one allocation boundary; combine with F16 for reveal rather than
painting elsewhere. Risk: low for clipping, medium if combined with navigation.
Acceptance: nonzero-origin sentinel tests for zero/one/small dimensions leave
all outside cells untouched; blank row remains blank in the reproduced frame.
The shared wrap helper's `w.max(1)` behavior does not itself guarantee zero-area
containment—callers must enforce their allocation.

### F18 — Distinguish read-only from disabled

**P2 · design inconsistency.** `pages/textareas.rs:37` labels `.disabled(true)`
as “Read-only transcript”; disabled TextArea refuses focus/keys, unlike
`DESIGN.md`'s readable/navigable read-only contract. Design and ecosystem audits
independently verify it.

Label the existing disabled fixture accurately. Demonstrate a read-only transcript
with existing TextViewport and a stated reason; use read-only CodeEditor for code.
Risk: low fixture correction; keyboard-copy completion depends on F19. Acceptance:
disabled examples remain dim/inert/unfocusable; read-only examples remain readable,
navigable/selectable, expose copy and reject mutation. Add read-only field APIs
only if a real field-shaped consumer cannot compose the existing primitives.

### F19 — Keyboard selection in the existing TextViewport

**P2 · confirmed coverage gap.** `viewport.rs:563` scrolls/follows/copies/clears;
Shift+Right does nothing and Shift+Down scrolls without selection. Range creation
is mouse-only. Diff delegates to the same handler. Design, interaction, API and
research agents independently checked the absence and present consumers: Holla
output, Jackin panes/Inspect, Showcase Terminal/Diff.

Add an explicit keyboard browse/selection caret, distinct from producer-owned
terminal caret. Reuse logical CellPos, selection/copy events and visual mapping.
Specify entry/exit, anchor extension, word/line/document movement, follow pause,
offscreen autoscroll and hints before implementation. Follow F10–F14 ownership
work; do not introduce another viewer or duplicate Diff selection state.

Risk: medium/high interaction semantics. Acceptance: keyboard and mouse copy
identical partial/word/multiline/wrapped Unicode ranges; preserve source identity
under resize, append and retention. Escape clears selection before exiting the
owner; no browse keys leak into attached terminal input. Cover empty/single-line,
no-selection copy, monochrome reversal and hardware-cursor policy. Plain scrolling
keeps existing behavior outside explicit selection mode.

### F20 — Find in read-only output through existing composition

**P2 · coverage gap: confirmed capability absence; proposed interaction/API design.**
TextViewport/Diff expose no query or next/previous match action. Holla service
filters and Jackin command filtering do not search displayed output. CodeEditor's
FindState and shared `ui::text::find_ranges` already provide a source-grapheme
mapping foundation. Ecosystem and API agents independently verified both absence
and reuse. This is not a defect in a promised current search feature.

After F19, compose a query input/match navigation into two present consumers
(Holla output and Jackin panes/Inspect), then extract only the demonstrated common
controller. Reuse TextInput/HintBar and source mapping, not a new search widget.
Risk: medium focus/follow/content invalidation. Acceptance: pasted query never
edits output, empty/no-match/next/previous/wraparound states, Unicode ranges,
content revision/eviction, match reveal, resize, Escape and explicit follow
restoration. Defer regex, replacement, file search and multiple cursors; current
evidence supports text finding, not those broader semantics.

Design evidence and peer dispositions:
[design verification](docs/plan-design-verification.md).

## Execution backlog: terminal lifecycle and trustworthy proof

### F21 — Restore terminal state for supported job-control suspension

**P1 · confirmed lifecycle defect in an isolated external-SIGTSTP flow.**
Current runtime has no suspend/continue ownership. Terminal auditor created an
owned controlling PTY with a non-orphan foreground app process group, ran
Showcase, sent SIGTSTP and returned foreground to its shell process. The stopped
application left ICANON/ECHO disabled and termios different from the saved state.
SIGCONT followed by normal quit restored the original termios. This proves the
specific suspension gap, not failure of normal teardown or every signal route.

Root: restoration belongs to startup/exit only, not job-control lifecycle.
Define suspend/re-enter as explicit TerminalSession transitions: restore owned
modes before supported suspension, re-acquire on continuation, rebuild geometry
and fully redraw. Integrate signals through a safe event/notification boundary,
not arbitrary allocation/rendering in a signal handler. Keep repeated transitions
idempotent and panic-hook ownership explicit. Raw-mode Ctrl+Z is a key byte, not
equivalent proof of external SIGTSTP handling.

Risk: medium/high platform/terminal lifecycle. The successful temporary probe
has known startup/deadline/reap cleanup weaknesses; do not retain it unchanged.
Track only live owned children, guard startup, bound all waits and reap on failure.
Acceptance: retained owned-PTY
normal, partial-startup, post-init draw/read error, panic, repeated enter/leave
and suspend/continue cases; shell canonical/echo state while stopped, resize
while suspended, resumed focus/full redraw and final exact termios. Explicitly
document unsupported platforms and unrecoverable cases: SIGKILL cannot unwind,
and a lost output device cannot receive restoration escapes. Do not promise
“every exit path”. Never run these probes against the user's live terminal.

### F22 — Separate rasterizer fidelity from terminal correctness

**P2 · confirmed verification-tool limitation.** `tools/ansi2png.py` iterates
code points with a heuristic width: e+acute totals 2 rather than application 1;
woman-technologist totals 5 rather than 2. This host's selected font renders
東/京/☕ as the same missing-glyph mask. Terminal and verification agents
independently reproduced width errors; this is not evidence that the Rust buffer
or a particular terminal fails identically.

Two more independently checked capture defects share this geometry boundary:
`tools/ansi2html.py:141/147` pads 東京 with two spaces at four columns because it
counts scalars; `ansi2png.py:78` does not clip to `cols`, so rendering ABCD at two
columns paints C and part of D into the right margin. A correct width table alone
would not fix allocation clipping or the separate HTML policy.

Keep ANSI/text/cursor and buffer/copy tests authoritative for their respective
layers; label current PNGs approximate for complex text. For normative PNG
review, make PNG and HTML consume one reference grapheme/cell representation;
use shaping and tested font fallback with
explicit font/version metadata, then compare against real terminal captures.
Do not replace Unicode fixtures to conceal failures. Risk: medium tooling and
platform reproducibility. Acceptance: split-style/combining/ZWJ/variation-selector/
CJK fixtures agree on occupied cells, cursor and selected content in both formats;
no glyph/background paints into padding. Font coverage is asserted, missing glyphs
reported. Actual emulator inspection remains required
for a claimed emulator compatibility result.

### F23 — Retain contract proofs, not only frame hashes

**P2 · coverage gaps / verification architecture weaknesses.** Existing hashes
pass with F16/F17 present. They cover one focused frame per page, exclude sidebar,
and omit hardware cursor/underline-color metadata. Capture counts have no automatic
source/binary provenance. The [prior-fix ledger](docs/plan-prior-fixes-verification.md)
and independent design review establish these limits.

| Proof item | Existing evidence / uncertainty | Required retained acceptance |
| --- | --- | --- |
| F23a · state reachability (design D4) | Static hashes do not traverse enabled controls or distinguish references from live states. | Executable page/state/input-route inventory with expected focus owner, visible target and semantic marker; keyboard/mouse, min/normal/wide, all palettes and actual NO_COLOR. Disabled controls excluded from traversal; read-only controls included. |
| F23b · lifecycle (V01/V02) | Guard/panic/idempotence units and historical PTY checks; fresh F21 suspension failure. | F21 owned-PTY matrix, exact mode/termios assertions and scoped best-effort documentation. Counter callbacks are not actual terminal proof. |
| F23c · event freshness/resize (V03) | Changed-event minimal pump passes; historical real burst replay not retained. | Real-App/PTY batched-versus-separated page/modal/resize/activation/paste/mouse sequences; below-minimum→normal→wide→minimum; focus, hits, drafts and cursor agree. No timing-based correctness assertions. |
| F23d · input-flood fairness (V04) | Ignored/consumed events drain until empty before tick checks. Source risk only; no sustained starvation measured. | Finite injected queue and bounded owned-PTY flood measure dispatch/tick/quit latency. Establish fairness contract, then implement bounded draining if it fails. Do not label absent sustained-flood proof a confirmed freeze. |
| F23e · dirty/idle performance (V05) | Fixed idle cache; F15 replacement scaling measured, no whole-app performance bound. | Retained F15 workloads plus large list/tree/table/grid rendering and producer synchronization; work counts, allocations, median/tail timings, fresh-layout equivalence. |
| F23f · rendering/color compatibility (V06/V07) | Four palette buffers and one fresh-process NO_COLOR backend path; real font/terminal breadth unproven. | F22 fidelity checks plus subprocess matrix for explicit palette, NO_COLOR empty/nonempty, force-color and relevant TERM policy. Assert color SGR separately from reverse/DIM/bold/underline. Record real emulator/font/tmux versions. |
| F23g · evidence provenance (V08) | Historical probes partly in /tmp; 301 complete sets prove presence, not generating source. | Retain small deterministic Unicode/PTY/burst harnesses; manifest source and binary digests, scenario/state/size/color environment, capture tools/fonts and review result. Hashes/manifests never substitute for semantic assertions or visual inspection. |
| F23h · precise subsidiary regressions (V09) | Current tests narrower than some names/report claims. | Add Ctrl/Alt/Shift Enter semantic combinations, actual Facts Input::Paste, editor scrollbar press/drag/redraw and nonzero-find-index reset to existing owning tests. Preserve known-good behavior; no framework needed. |

Risk: low/medium test maintenance; lifecycle harnesses need strict process/PTY
ownership and cleanup. Write contract assertions, not copies of implementation
logic. Keep platform-unavailable checks explicit with required external evidence;
do not silently treat skipped coverage as passed. Only update baselines after
contract tests and human/agent visual review approve the intentional change.

## Extended primary-source research

Fetched live on 2026-09-10 through Firecrawl. This extends, rather than replaces,
the earlier Bubbles/Textual/fzf/Lazygit comparison. Local reproductions decide
defect status; another project's feature list is not proof of a missing widget.
Full source-to-consumer reasoning: [ecosystem report](docs/plan-ecosystem-research.md).

| Primary evidence | Useful contract / local decision |
| --- | --- |
| [Helix selection keymap](https://docs.helix-editor.com/keymap.html#select--extend-mode) | Explicit keyboard range extension distinguishes caret and anchor. Supports F19; do not import Helix's whole modal vocabulary. Its separate system-clipboard commands also reinforce truthful copy destinations. |
| [Zellij actions](https://zellij.dev/documentation/keybindings-possible-actions.html) and [keybinding modes](https://zellij.dev/documentation/keybindings) | Scroll/search/copy are distinct actions with ownership. Supports F19/F20 and input-mode tests, not external-editor launch or terminal emulation additions. |
| [prompt_toolkit bindings](https://python-prompt-toolkit.readthedocs.io/en/master/pages/advanced_topics/key_bindings.html#attaching-a-filter-condition) | Conditional binding groups make active ownership explicit. Supports F08d conformance before any shared descriptor. |
| [Yazi keymap](https://yazi-rs.github.io/docs/configuration/keymap/) | Separate manager/input/picker/confirm/help layers make precedence inspectable. Its task inspection confirms existing operation/output composition; no new task widget follows. |
| [rat-focus 2.1.1](https://docs.rs/rat-focus/2.1.1/rat_focus/) | Focus-set freshness and transition hooks are explicit. Existing FocusRing/HitRegistry already supply much of the mechanism; F01/F23 test owner transitions instead of replacing it. |
| [Urwid containers and dynamic ListBox](https://urwid.org/manual/widgets.html#dynamic-listbox-with-listwalker) | Insertions change integer-position meaning; opaque positions/lazy walkers separate identity and presentation. Supports F03/F05/F06 without imposing one identity type across domains. |
| [rat-widget 3.2.1](https://docs.rs/rat-widget/3.2.1/rat_widget/) and [rat-ftable 2.2.0](https://docs.rs/rat-ftable/2.2.0/rat_ftable/) | Visible-row drawing does not automatically bound copying or iterator traversal. Supports F15 measurement and existing paged/adapted data reuse, not a mandatory virtual-data trait. |
| [Kitty text-sizing protocol](https://sw.kovidgoyal.net/kitty/text-sizing-protocol/#fixing-the-character-width-issue-for-the-terminal-ecosystem) | Terminal/application Unicode width agreement is a coordination problem; optional explicit widths have a capability-detection protocol. Inference: F22 needs declared emulator/font evidence, and unconditional protocol emission is not a safe universal renderer fix. No current Junie failure on a named emulator was established by this research. |
| [Kitty OSC5522 clipboard extension](https://sw.kovidgoyal.net/kitty/clipboard/) | This extension specifies permission/error outcomes and separate read/write operations; do not attribute its status protocol to legacy OSC52. Inference: an eventual OS provider belongs at the application boundary with explicit destination/failure behavior. Current preview Copy events must remain simulated; F09 fixes delivery only. |

Earlier evidence remains relevant: [Bubbles Key/Help](https://github.com/charmbracelet/bubbles#key)
and [Textual binding declarations](https://textual.textualize.io/guide/input/#bindings)
motivate hint/action consistency; [Textual modal screens](https://textual.textualize.io/guide/screens/#modal-screens)
motivate complete input ownership. [fzf preview](https://github.com/junegunn/fzf#preview-window)
is already covered by Picker/Splitter/TextViewport composition. These links were
reviewed in the initial same-day research; the extended report adds the primary
sources above rather than claiming every historical page was fetched twice.

## Conditional opportunities and explicit exclusions

These remain tracked, not silently discarded. None authorizes unrelated feature
work. Absence of a current consumer is different from an excuse to leave a
known defect unfixed.

| ID / classification / priority | Evidence, possible direction, risk and admission test |
| --- | --- |
| O01 · optional opportunity · P3: real async ownership | Existing apps simulate workers. No live-service stale-response defect proven. When integrating a real consumer, test cancellation on owner removal, generation/revision matching, stale completion rejection, failure inspection and shutdown. Then choose a shared worker boundary if two consumers need it. Medium/high lifecycle risk; no framework now. |
| O02 · optional opportunity · P3: Widget/Theme/glyph abstractions | README ideas are hypotheses; typed events and public semantic tokens already serve four apps. Require two blocked heterogeneous compositions for a Widget trait, a real second theme requirement, or tested alternate glyph-width/monochrome semantics. Medium/high public API risk. Existing page-level composition and theme resolvers remain default. |
| O03 · conditional architecture opportunity · P3: generic keyed collections | Holla restores numeric picker cursor after refresh, but a current in-modal insertion retargeting that selection was not reproduced. Reproduce an actual update flow first; F02 uses owner tab identity, F05 uses tree paths. Medium identity/compatibility risk. Only extract common reconciliation after two consumers share the same preserve/reset semantics. |
| O04 · optional opportunity · P3: matrix paste | Grid exports TSV-like ranges but pastes into one single-line cell; `A\tB\r\nC\tD` becomes `A\tBC\tD` under existing newline normalization. No matrix-import promise exists. Document current scope; only add import after explicit product choice with parser/type/bounds/readonly validation, atomic pending updates, undo and overwrite confirmation. High data-loss risk; not a routine Unicode patch. |
| O05 · optional opportunity · P3: terminal extensions/OS clipboard | Explicit-width protocols, synchronized output, keyboard enhancement and real clipboard transport are not automatically required. First reproduce a named terminal/backend problem or present consumer need, verify capability/fallback and cleanup, and preserve preview side-effect boundaries. Medium/high compatibility/privacy risk. F21's proven suspend defect is not deferred under this item. |
| O06 · conditional architecture opportunity · P3: generic Container/modal manager | Existing barriers, focus rings and typed outcomes already work. F01/F08/F09 establish narrow routing/effect gaps; fix them at those boundaries. Extract only policy shared by at least two real owners with parity tests. High routing risk if broadened; duplication alone is insufficient. |
| O07 · optional opportunity · P3: read-only field APIs | Correct F18 fixture now using existing viewport/editor. Require a field-shaped selectable noneditable consumer that cannot use them before extending all input APIs. Medium state/API risk; navigation/copy/mutation rejection must be specified together. |

No new command palette, file picker, preview viewer, Diff widget, notification/
toast family, task manager, retry card, paginator or generic badge is justified
by this inventory. Existing controls or composition already cover the relevant
flows, or the proposed capability lacks a demonstrated reusable contract. This
is a coverage decision, not a permanent prohibition against future evidence.

## Dependency order and bounded delivery units

Each row is an independently reviewable implementation unit or coordinated
invariant change, not one giant rewrite. Work may run in parallel when ownership
does not overlap; any shared-file changes need one integration owner.

| Stage | Work / dependencies | Exit evidence |
| --- | --- | --- |
| 0 · freeze contracts and retain failures | Snapshot current inventory/source digest; turn accepted temporary repros into owning tests. Record decisions for draft revocation, replacement identity, span style precedence, tab/copy semantics and public-field migration. | Tests fail for the intended reason; prior suite still establishes baseline. No unrelated baseline updates or source normalization. |
| 1A · owner safety | F01/F02/F03/F04; F08a/b can share touched routes without merging their distinct tests. | No hidden paste, unrelated tab close, stale-anchor panic or forbidden grid commit through any listed path. Real app tests as well as units. |
| 1B · text identity | F10/F11/F12/F13 as coordinated document-mutation boundaries; settle F14 mapping compatibility before exposing changed cursor semantics. | Split-style text and selected source preserved; every ingestion path capped; cache follows content; retained reading/drag identity correct. |
| 1C · lifecycle | F21 with F23b in isolated PTYs; independent of document/widget work. | Shell usable while stopped; resume/quit modes correct; unsupported/fatal cases documented precisely. |
| 2 · existing API consistency | F05/F06/F07, F08c/d, F09, remaining F14 policy migration; depend on their specific owner/transaction decisions, not an all-library rewrite. | Stable targets, checked shape errors, pending-sort policy, chord/selection parity and delivered preview copy; current well-formed behavior preserved. |
| 3 · reachable design contract | F16/F17/F18 and executable F23a; can proceed beside nonoverlapping stage 1/2 work. | Every enabled control/reference state reachable at five sizes, bounded cells/hits/cursors, readable readonly/disabled distinction. |
| 4 · keyboard/output capability | F19 after F10–F14; F20 after selection/follow contract and two consumer compositions. | Keyboard/mouse exact-copy parity, find ownership and content invalidation, reusable docs/showcase states. |
| 5 · measured live rendering | F15 after content ownership; F22 capture fidelity and remaining F23 proofs run throughout and finish here. | Incremental-work counters and representative timing budget pass; terminal/font evidence honest; state/provenance manifests complete. |

Do not postpone an independently fixable P1 merely to complete a broad API design.
If a compatibility decision requires explicit approval, isolate the narrow safe
change and state the residual root cause. Never mark the root fixed while its
enabling unchecked path remains. Visibility/signature changes require an API
migration record: callers, old/new behavior, deprecation boundary, source examples,
compatibility tests and a scoped rollback preserving user data/drafts. A rollback
must not silently re-enable a known destructive path; disable that action if needed.

## Per-change acceptance gate

For each changed primitive or composition, add only relevant cases from this
matrix, and explicitly mark nonapplicable states rather than implying coverage:

- Default, hover, pressed, focused, selected, editing, disabled, read-only,
  error/recovery, busy/loading, empty and unavailable states; meaningful hints.
- Keyboard, mouse, drag, wheel/scrollbar, paste and programmatic mutation;
  modifier propagation, topmost owner, focus order/trap/restore and stale-hit rejection.
- 72×20, 80×24, 100×30, 120×40, 160×50; below-minimum recovery where applicable;
  nonzero-origin and zero-size sentinels; resize during edit/drag/selection.
- TrueColor, ANSI-256, ANSI-16, Mono and real NO_COLOR backend subprocesses;
  no meaning carried solely by color, no hidden-by-color glyphs.
- Emoji/ZWJ/variation selectors, CJK, combining marks, tabs/control policy,
  source/display mapping, exact copied payload, cursor and clipping invariants.
- Public API docs and inventory update, existing showcase demonstration,
  actual consumer integration, deterministic regression and inspected capture
  for a visual change. New public extraction requires demonstrated reuse.

Repository gates after integration:

```sh
rtk cargo fmt --check
rtk cargo clippy --all-targets -- -D warnings
rtk cargo test
rtk cargo doc --no-deps
rtk git diff --check
```

Regenerate only deliberate affected baselines after inspecting output. Capture
scripts default to seven fixtures, not the ten-case 250-set audit; the exact
ten-case reproduction is in the [verification ledger](docs/plan-prior-fixes-verification.md).
Use an isolated socket and a new explicit output directory. Retain provenance
and failed cases. Larger screenshot counts cannot replace state assertions.

## Finding traceability and independent verification

All listed findings were checked by scoped subagents; cross-checks below name
which claims have a second executable reproduction versus independent source
review. The authoring agent's report remains the detailed evidence record.

| Source findings | Plan disposition | Independent check |
| --- | --- | --- |
| Prior main 17 rows, six additional groups, editor nine rows | Fixed or narrower proof scope; baseline table and F23 | Verification agent reran all 328 tests and made a per-symbol/test ledger; editor/interaction/design/API agents checked their respective prior families. |
| Inventory API-1/3/4 | Fixed; protect regressions/catalogue | API tests/inventory digest and prior-fix ledger. |
| Inventory API-2; PLAN-API-01/02/03 | F03/F02/F05 | API real-App/library probes; design independently replayed both app failures and tree insertion. |
| PLAN-API-04/05 | F06/F07 | Verification agent independently reproduced cursor drift, pending-value order and ragged Table/Grid panic. |
| PLAN-API-06; README hypotheses/API-5 | O02/O03/O06; narrow F01/F08/F09 | API and ecosystem source checks; no unproven Holla insertion bug promoted to fact. |
| INT01/02 | F01/F08a | Editor independent modal-paste App repro; main traced both-value omission against key/click paths. Upper-value test is rendered real dispatch with controlled setup, not a pure terminal replay. |
| INT03/04/05 | F08b/c/d | Design independently reproduced cluster Backspace, lost Shift range and Ctrl+S sorting; owner paste absence source-traced. |
| INT06; design D3; research R1 | F19 | Interaction key probes plus independent API/design/ecosystem source checks. |
| INT07; research R8 | F09 | Interaction and main complete copy-effect source trace; full selected Inspect App replay remains acceptance work. |
| TXT-01/03/04/05/06 | F10/F11/F12/F13/F04 | Interaction independently recompiled/reran text agent's public-API probes; main inspected ownership paths. |
| TXT-02/07/08 | F14/F15/O04 | Interaction independently checked tab geometry and repeated release workload; TSV behavior reproduced but matrix import remains optional. |
| Design D1/D2/D4/D5 | F16/F17/F23a/F18 | Design rendered/replayed; ecosystem independently inspected source and reran the still-passing baseline. |
| Research R2/R3/R4/R5/R6/R7 | F20/F08/F01/F03–F07/F11–F15; existing operation coverage | API cross-checked viewport find absence and identity recommendations; local probes independently ground architecture/performance claims. |
| Verification V01–V09 | F21–F23 and F15 | Fresh gate/artifact checks; terminal agent independently investigates lifecycle/rasterizer, with final review checking claim scope. |
| Terminal TV1/TV2/TV3/TV4/TV5 | F22/F21/F23f | Verification agent repeated width/font/HTML checks, inspected PNG overflow evidence and independently reviewed the isolated SIGTSTP probe topology/limitations. No universal terminal claim. |

Evidence documents:

- [API verification](docs/plan-api-verification.md)
- [Interaction verification](docs/plan-interaction-verification.md)
- [Text verification](docs/plan-text-verification.md)
- [Design verification](docs/plan-design-verification.md)
- [Extended ecosystem research](docs/plan-ecosystem-research.md)
- [Prior-fix ledger](docs/plan-prior-fixes-verification.md)
- [Terminal verification](docs/plan-terminal-verification.md)
- [Final cross-review](docs/plan-final-review.md)

Final planning gate: all four cross-review findings resolved; no omitted finding
or unresolved factual issue. Main checked 34 local links across this plan and
eight supporting reports; all resolve. `git diff --check` passes. The 47-file
library SHA-256 remains
`40e412fd6bb7ff7700b90db054b0a4e5a5d535863e4eea4e55594f6bfb26202e`,
matching the verified inventory. This pass changes documentation only: this
plan, supporting reports and a README link. Product sources, dependencies and
baselines were not changed; the already-dirty implementation remains baseline.

## Stop conditions

Planning is complete when this file covers every prior/new finding disposition,
source evidence and uncertainty, dependencies, compatibility decisions and
acceptance; subagent cross-review has no unresolved factual issue; all local
evidence links resolve; product sources/baselines remain unchanged by this pass.

Eventual implementation is complete only when every accepted F-item is fixed or
its stated coverage proof is satisfied, all relevant gates pass, no known
introduced regression remains, and conditional O-items retain explicit unclaimed
status. A proven tool/platform limit must state missing evidence and the next
required external check; it is not a passing result. Do not equate completing
this plan, passing the old suite, or updating hashes with implementing the work.
