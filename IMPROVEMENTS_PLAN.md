# Improvements plan

Status: **planning complete; implementation pending.** Historical audit results
are recorded in the [audit reference](docs/improvements-plan-reference.md).
Scope: the `holla-fable` worktree and `tailrocks/holla` default-branch snapshots
inspected on 2026-09-10, pinned in that reference. Earlier findings remain valid
within their recorded scope.

## Decision

Preserve every useful existing-Holla outcome within the new Here/context,
resource/action, preview, scope, activity and plan model. The external baseline
defines capability requirements; `holla-project/CONCEPT.md` defines the new
product model; `DESIGN.md` defines the Junie interaction and visual system.
This pass specifies deterministic product representations and future execution
contracts. It does not enable real commands in the preview.

Repair ownership boundaries first: modal input, destructive action eligibility,
collection identity, read-only transitions, and retained text. Then make existing
capabilities reachable and keyboard-complete. Preserve Junie's geometry-first
focus, restrained planes, sparse framing, contextual hints and semantic color.
No new widget, framework, dependency or palette is approved by this plan.

The initial audit's completed fixes remain protected regressions, not work to
repeat. The earlier library planning pass found additional defects; passing the existing suite does
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

## Evidence boundaries and protected regressions

Audit counts, prior-finding dispositions, source snapshots and verification
history are in the [audit reference](docs/improvements-plan-reference.md).
Preserve the [prior-fix ledger](docs/plan-prior-fixes-verification.md) and
[disposition-map regression obligations](docs/improvements-plan-reference.md#previous-findings-disposition-map);
completed fixes are not work to repeat. F23 retains the missing proof requirements.
This plan's priorities govern duplicate findings, which count as one work item.

Historical passing tests and capture counts do not establish current correctness.
Buffer/cursor/copy assertions, emitted ANSI and actual terminal inspection prove
different layers. PNGs remain approximate for complex text until F22 passes;
actual NO_COLOR is separate from the monochrome palette. Unicode search uses
Rust lowercase mapping, not full case folding or canonical normalization.

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

## Conditional opportunities and explicit exclusions

Generic widget/theme/glyph, keyed-collection, container/modal and worker
framework proposals are removed from this plan. Former O01/O02/O03/O06 entries
remain only in the [audit reference](docs/improvements-plan-reference.md#retired-framework-proposals).
Concrete fixes F01–F09 remain required. HP14/HP15 retain task ownership,
cancellation, generation/revision matching, stale completion rejection, failure
inspection and shutdown. HP18 retains scanner ownership; HP22 retains cleanup
and report ownership. HP02/HP17/HP18 retain persisted-state contracts.

O04, O05 and O07 below remain conditional. Their absence does not excuse a known
defect or defer a concrete HP requirement. Reuse existing components and keep
application policy with its owner; no generic framework is an implementation task.

| ID / classification / priority | Evidence, possible direction, risk and admission test |
| --- | --- |
| O04 · optional opportunity · P3: matrix paste | Grid exports TSV-like ranges but pastes into one single-line cell; `A\tB\r\nC\tD` becomes `A\tBC\tD` under existing newline normalization. No matrix-import promise exists. Document current scope; only add import after explicit product choice with parser/type/bounds/readonly validation, atomic pending updates, undo and overwrite confirmation. High data-loss risk; not a routine Unicode patch. |
| O05 · optional opportunity · P3: terminal extensions/OS clipboard | Explicit-width protocols, synchronized output, keyboard enhancement and real clipboard transport are not automatically required. First reproduce a named terminal/backend problem or present consumer need, verify capability/fallback and cleanup, and preserve preview side-effect boundaries. Medium/high compatibility/privacy risk. F21's proven suspend defect is not deferred under this item. |
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

## Existing Holla functional-parity audit

### Authority, scope and counting

The authoritative comparison is [the capability matrix](docs/holla-parity-matrix.md).
It is backed by [source inventories](docs/holla-parity-evidence.md),
[the preview inventory](docs/holla-preview-inventory.md), and
[independent verification](docs/holla-parity-verification.md).
Only matrix rows count as capabilities; evidence-report subrows, action variants,
tests and plan items are not extra counts. Each matrix row has exactly one
disposition and one owning HP item. Shared dependencies do not double-count it.

The comparison is pinned to the 2026-09-10 audit snapshots in the
[audit reference](docs/improvements-plan-reference.md#holla-audit-snapshots-and-counts).
The matrix and linked source inventories retain exact revisions and evidence.

“Covered” proves the complete stated interaction and outcome in the current
deterministic preview. It does not claim live external execution. A label,
command string, generic success animation, fixture resource, or conceptual
promise alone is insufficient. Partial rows name the missing outcome. Missing
means no usable representation of the stated capability. Internal safeguards
with user-visible consequences remain requirements; release plumbing and
documentation-only promises receive explicit exclusions.

### Three distinct work categories

- **A — Existing-Holla parity:** HP01–HP23 below. These preserve baseline user
  capabilities, including obscure CLI, filesystem and safety workflows. Each
  mapped Partial/Missing row must meet its item's acceptance contract.
- **B — New-Holla functionality:** existing context rings, explanatory relevance,
  exact aliases/pins/hide controls, PostgreSQL insights, SSH/remote identity,
  GitHub cloning, specialist handoffs, dependency-aware editable plans and
  persistent-in-session activity tabs originate in the new concept. Their
  current state is recorded in the preview inventory. They are not evidence
  of old-provider parity. Native apt/dnf/systemd-user and broader cross-session
  activity restoration are expansion where the baseline lacks them.
- **C — Design-system/library work:** retained F01–F23 and conditional O04/O05/O07.
  Existing widgets, focus/hit ownership, selection, retained text, lifecycle
  and capture contracts support A. No new generic widget, framework, palette
  or dependency is authorized by a parity row.

### Common implementation and evidence contract

Every HP item includes its mapped matrix rows' exact old behavior, source
locations, preview evidence and semantic differences by reference. Those rows
are mandatory acceptance cases, not illustrative examples. Implement all named
action variants and boundary cases before closing an item.

The implementation route has two explicit gates. **Representation gate:**
model discovery, inputs, outputs, errors and effects in deterministic fixtures;
make the full interaction reachable in the preview. **Operational gate:** in a
separately authorized production integration, connect typed adapters and prove
the same contract against isolated processes/filesystems on supported platforms.
The current preview retains its no-real-command/no-real-filesystem-mutation
guarantee. Finishing this planning task does not pass either future gate.

Architecture: use stable typed resource/action identities and explicit provenance,
effective cwd, host, argv, execution kind, risk, freshness and outcome. Resolve
these facts once for review and revalidate before mutation. Do not make command
display strings executable or infer success effects by an incomplete ID switch.
Discovery, simulation and later production adapters must produce the same
domain events. Keep app-specific policy in Holla; extract shared components only
when an actual second consumer needs the same contract.

Interaction/state acceptance for every UI item: keyboard-first reachability;
pointer selection/activation and wheel ownership through the hit registry;
one active focus owner; query input never interpreted as page hotkeys; modal
paste/keys/wheel cannot reach the background; cancel restores a still-valid
opener; resize/provider arrival cannot retarget selected or confirmed resources.
Use Here pages for decisions, previews and alternatives for resource operations,
activities for running work and plans for compound intent. Preserve DESIGN.md's
focus gutter, muted metadata, semantic danger, Cancel-default destructive gates,
contextual hints and monochrome legibility. Do not restore the old launcher layout.

Automated acceptance for every item: add the named deterministic scenario to
the existing Holla harness; assert discovery, selection, exact target/cwd/argv,
gate ownership, state transitions, failure and final world effects. Assert
absence of side effects after cancel, stale confirmation or invalid input.
Existing 53 passing Holla tests are a baseline, not new parity proof. Run
`rtk cargo test --bin holla`, then relevant library/CLI tests and the existing
per-change gates when implementation touches their owners. Adapter tests must
use controlled executables/temp trees and never live user cleanup targets.

Capture/visual acceptance for every UI item: extend existing `tools/holla_shots.sh`
and `tools/holla_flows.sh` composition, using an isolated terminal and output
directory. Capture decisive entry, review, running, failure and result states
at 80×24 and 120×40 in TrueColor and Mono; add 72×20, 100×30, 160×50, ANSI-16,
ANSI-256 and actual NO_COLOR for changed geometry/state grammar. Include a
keyboard journey and pointer journey for each new decision surface. Inspect
PNG/terminal rendering and retained text/cursor evidence; record provenance.
For CLI-only rows retain stdout/stderr/exit transcripts instead of inventing a
visual screen. Capture instructions below identify the additional decisive frame.

Safety defaults are shared requirements, not optional enhancements: no shell
interpolation of discovered text; explicit shell interpreter actions remain
possible after truthful review; no secrets in preview/history/output copies;
target-bound authorization; Trash first, permanent deletion explicit; no target mutation
on dry run (explicit audit-log writes remain allowed); no success claim without observed/simulated outcomes. Named legacy
defects in the matrix are not behaviors to reproduce.

### Conflict resolutions

| Legacy behavior / conflict | Preserved value and new equivalent | Required proof |
| --- | --- | --- |
| Executing a launcher row exits the launcher; generic confirmation precedes destructive actions. | Keep fast invocation and informed consent; return to Here while a named activity retains output. Use risk-appropriate Junie facts/typed gates. | HP14/HP15 preserve identity, cwd, output, prompt access, cancellation and final result through navigation. HP21 proves no gate bypass. |
| Scattered providers, callbacks and shell snippets permit preview/execution drift or swallowed failures. | Preserve every actual command outcome in typed action definitions and explicit plan steps; command preview derives from the executable specification. | HP05–HP17 compare displayed and executed argv and effects; dependent stages do not run after failed prerequisites. |
| Old group navigation/key hints differ from current Junie; browser recommendations are display-only. | Here mixed results retain discoverability; Files is a resource page, with Junie alternatives/focus grammar. Recommendation facts remain inspectable; execution from browser is new functionality. | HP01–HP04 prove each resource/action accessible without reproducing the old layout or treating labels as execution. |
| Global frecency and learned query choice differ from exact aliases/context ranking. | Preserve retained usage, query-choice learning and privacy controls; exact user aliases win, learned choices are contextual and explainable. | HP02 demonstrates restart, opt-out and reset; learned choice cannot defeat an exact alias or safety gate. |
| `run --yes` both confirms actions and persistently trusts project content. | Preserve unattended invocation, but make durable trust an explicit operation distinct from one-run authorization. | HP16/HP17 fixtures prove noninteractive execution, stable rejection exits and no silent persistent trust; document the deliberate CLI semantic change. |
| Some legacy deletions bypass descendant protections/process guards; dry run can stop Gradle and reports deletion wording. | Preserve selectable cleanup and previews, strengthen one shared authorization boundary, carry typed dry-run outcome through result/log layers. | HP20–HP22 prove protected descendants, process uncertainty, no dry-run daemon stop, and truthful estimates/results. |
| Legacy deletion quit screen can drop the worker's result receiver; legacy task output is an unbounded in-memory vector. | Retain cleanup ownership through quit decisions; distinguish task cancellation from an irreversible operation already committed. Apply the new library's bounded-retention contract without silently losing output identity. | HP15/HP22 prove no detached unknown mutation, no lost report, visible retained/truncated output and explicit final state. |
| New concept promises broad developer tooling while legacy macOS capabilities are concrete. | Keep Homebrew, services, Amp, Oh My Zsh and macOS artifact/platform behavior; Linux upgrade examples do not supersede them. | HP10/HP13/HP20/HP23 provide macOS and Linux fixtures with correct availability and equivalent outcomes. |

These are planned equivalences. None earns “Covered — redesigned/superseded”
until the current preview actually demonstrates the row's full value.

### A — Parity implementation items

<a id="hp01"></a>

#### HP01 — Adaptive discovery and stable action identity

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve available actions, instant discovery, stable identity, grouping and keyboard invocation. The preview has timed sources and a finder, but lacks the complete legacy probe/registry contract.

**Source evidence and mandatory scope:** [matrix HP01](docs/holla-parity-matrix.md#hp01--adaptive-discovery-and-stable-action-identity) — `L01`, `L07`, `L02`, `L03`, `L04`, `L05`, `L14`, `L06`, `L16`, `L17`, `L18`, `L19`, `L25`, `L26`, `L27`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Model executable missing/present, local markers, daemon failure and unknown state separately. Keep Current/Project/Host provenance visible; unavailable tools cannot become runnable through aliases. Retain built-in Find/Disk/Cleanup routes. Merge provider contributions deterministically by canonical action ID; report collisions and malformed-source warnings without retargeting existing selection. First render must precede slow discovery/history work.

**Architecture / reusable components:** Give each source a generation and each row an identity independent of index, label or arrival order. Reuse Picker, Input, Panel, EmptyState, StatusBar and FocusRing; do not add a provider widget.

**Required deterministic fixture:** `parity-discovery` — empty PATH; each legacy provider individually available; marker-only/tool-only; .git file; unavailable daemon; out-of-order/failed sources; conflicting IDs; query typed before completion.

**Acceptance / automated verification:** Assert registry and visible action sets, source provenance, deterministic order and warning recovery. Assert first paint before a blocked source completes, preserved selection through inserts/removals, query reset semantics, Enter/separator eligibility, Home/End/PageUp/PageDown and preview focus/long-command scrolling. Measure first-paint latency with a stated environment; do not invent a universal 100 ms guarantee. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture discovering, no tools, one failed source, collision detail, long preview and narrow return-focus states.

**Dependencies:** F02/F03/F05/F08/F23; HP07/HP10/HP13/HP17 supply provider-specific fixtures.

<a id="hp02"></a>

#### HP02 — Search, query editing and durable usage learning

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve fuzzy matching, query editing, recents, learned query choices and resilient persisted usage. Current usage is in memory; stored last-used timestamps do not establish decay.

**Source evidence and mandatory scope:** [matrix HP02](docs/holla-parity-matrix.md#hp02--search-query-editing-and-durable-usage-learning) — `L08`, `L10`, `L15`, `L30`, `L09`, `I-H01`, `I-H02`, `I-H03`, `I-H04`, `I-H05`, `I-H06`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Keep exact user aliases strongest. Preserve matching across label/group/description/keywords and grapheme-safe emphasis; learned choices must still match and must not authorize execution. Explain the difference between an explicit pin, a remembered query and contextual frequency. Retain five-item recent projection and truthful invocation history, including failed invocations; headless runs remain outside implicit learning. Implement visible query selection/undo/redo/word deletion.

**Architecture / reusable components:** Use a versioned usage-store adapter with deterministic clock, atomic merge and opt-out. Preserve v1 legacy reads or explicitly migrate that store into the new model without losing choices; writes retain concurrent updates. Reuse Input/Picker and existing ranking explanations. Do not confuse activity logs with usage.

**Required deterministic fixture:** `parity-history` — threshold/ties, exact alias versus learned query, whitespace/case normalization, Unicode, 20-use bound, 10-day decay, 90-day expiry, clock skew, corrupt/versioned store, two writers, save error, HOLLA_NO_HISTORY=1.

**Acceptance / automated verification:** Assert no unmatched learned results; bounded automatic frecency, deterministic ties and positive-only recents. Assert restart continuity, stale query pruning, opt-out performs neither reads nor writes, and save errors do not change action outcome. Exercise query Ctrl-A, Ctrl-Z, redo, word-delete and selected replacement. Never claim ignored clipboard requests as supported copy. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture Recent here, Why this result, changed ranking after restart, disabled-history status and Unicode match emphasis.

**Dependencies:** HP01/HP17; F08/F10/F23. Explicit pins remain new-product behavior, separate from automatic learning.

<a id="hp03"></a>

#### HP03 — Find files and perform resource actions

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve home file/folder discovery and Open, Reveal, Copy path, Analyze actions. Current file rows are fixture resources; Reveal incorrectly copies cwd.

**Source evidence and mandatory scope:** [matrix HP03](docs/holla-parity-matrix.md#hp03--find-files-and-perform-resource-actions) — `I-F01`, `I-F02`, `I-F03`, `I-F04`, `I-F05`, `I-F06`, `I-F07`, `I-F08`, `I-F09`, `I-F10`, `I-F11`, `I-F12`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Provide a Files scope in Here with mixed files/directories, indexed/partial counts and exact resource identity. Rank exact filename/stem, filename substring and fuzzy matches with deterministic path ties; expose bounded results (legacy 100) honestly. Keep an explicit blank-query state. Alternatives must operate on the selected path, not current cwd. Analyze directories directly and files via parent; preview resulting scope.

**Architecture / reusable components:** Use a cancellable file-index adapter; retain ignore/hidden policy, no symlink following and cloud/root exclusions. OS handoff and clipboard adapters receive validated path identity and typed intent. Reuse Picker, menu, Props and existing disk page routing.

**Required deterministic fixture:** `parity-files` — mixed home results; indexing in progress; .ignore/hidden/iCloud exclusions; same basename; Unicode byte-to-grapheme match; open failure; reveal on macOS/Linux; clipboard size/write failure; file versus folder Analyze.

**Acceptance / automated verification:** Assert bounded ranking, exact selected path through refresh/alternatives and cancellation joins pending index work. Assert macOS open/open -R and Linux xdg-open/parent fallback argv, failed handoff visibility, OSC52 payload/cap/flush behavior, and simulated Analyze root. Clipboard result must name actual destination without unsupported success claims. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture progressive results, resource alternatives, failed opener/clipboard and Analyze handoff at narrow width.

**Dependencies:** HP01/HP02/HP18/HP19/HP23; F09/F10/F23.

<a id="hp04"></a>

#### HP04 — Browse folders and safely preview files

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve the real browser and safe file preview, including obscure path navigation and asynchronous race protection. No equivalent general browser exists in the preview.

**Source evidence and mandatory scope:** [matrix HP04](docs/holla-parity-matrix.md#hp04--browse-folders-and-safely-preview-files) — `I-B01`, `I-B03`, `I-B04`, `I-B05`, `I-B07`, `I-B08`, `I-B10`, `I-B12`, `I-B14`, `I-B02`, `I-B06`, `I-B09`, `I-B11`, `I-B13`, `I-P01`, `I-P02`, `I-P03`, `I-P04`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Create a Files page inside Here: real directory semantics in simulated fixtures, directories first, type/size/mtime/hidden metadata, parent/child navigation and highlighted return target. Exact-path picker accepts relative/home/absolute paths and prioritizes exact existing paths over fuzzy suggestions. Preserve page/query/scroll state when returning to Here; make any context rebind explicit. File Enter previews; OS open remains an alternative. Directory command recommendations identify preview-only versus executable actions.

**Architecture / reusable components:** Use typed native path identity rather than lossy display text. Generation-tag listing/jump/preview results; discard stale successes and errors. Reuse Picker, TreeView where needed, Input, Splitter and TextViewport. Bounded preview adapter sanitizes content/title/path/links and validates the opened descriptor, not only pre-open metadata.

**Required deterministic fixture:** `parity-browser` — file/dir starting path, invalid path, hidden toggle, exact jump, missing/broken symlink, lossy-name collision, delayed old response, capped directory count, empty/large/binary/invalid UTF-8/control-rich file, FIFO/device replacement.

**Acceptance / automated verification:** Assert parent+highlight behavior, 50 jump suggestions and source labels, 2048-entry/40 ms lower-bound directory counts, 256 KiB/2000-line/4096-character preview bounds and safe UTF-8 cap. Assert no read of special opened descriptor, nonblocking race handling, sanitized terminal bytes, inline failures, focus/scroll retention and visible replacement for blind Ctrl-L editing. File reading never executes content. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture listing/preview focus, hidden entries, path error, partial count, binary/error/long-text preview and stale-result rejection journey.

**Dependencies:** HP03/HP15/HP23; F01/F05/F10–F14/F19/F20/F23.

<a id="hp05"></a>

#### HP05 — Current-repository Git operations

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve current Git pull, push and full status. Preview pull is ff-only, status is a snapshot and push has no modeled effect.

**Source evidence and mandatory scope:** [matrix HP05](docs/holla-parity-matrix.md#hp05--current-repository-git-operations) — `OP01`, `OP02`, `OP03`, `OP04`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** A Git resource exposes full status and synchronization alternatives with effective repository/cwd. Retain fast-forward default while offering the ordinary configured pull behavior after explicit conflict/rebase/merge review; do not silently remove configurations that old git pull supports. Push must change modeled remote state and report rejection/auth/upstream failures. Status retains the detail users obtain from full git status.

**Architecture / reusable components:** Separate repository identity, operation argv and result/effect; remove success-only generic fallbacks for these actions. Reuse snapshot Props/TreeView and activity TextViewport; no Git-specific shared widget.

**Required deterministic fixture:** `parity-git-current` — .git file/dir, subdirectory context distinction, clean/dirty/behind/diverged/detached/rebase, no upstream, rejected push, merge-configured pull and command failure.

**Acceptance / automated verification:** Assert correct repository/cwd for discovery and invocation, preview/argv equality, no push effect on failure and observed state after success. Assert full status detail and truthful unavailable-state recovery. New scope expansion may extend local marker discovery without dropping the old current-folder route. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture full status, pull strategy review, push rejection and successful local/remote before-after state.

**Dependencies:** HP01/HP14/HP15/HP17; F23.

<a id="hp06"></a>

#### HP06 — Repository batches, mirrors and hygiene

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve sibling-repository pull/push/status, origin+GitLab mirroring and Git hygiene. Existing child plans cover only part of these outcomes.

**Source evidence and mandatory scope:** [matrix HP06](docs/holla-parity-matrix.md#hp06--repository-batches-mirrors-and-hygiene) — `OP05`, `OP06`, `OP08`, `OP10`, `OP11`, `OP07`, `OP09`, `OP12`, `OP13`, `OP14`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Show immediate discovered repositories as resources, including legacy multi-repo threshold behavior as a fixture; new scope controls may expose one repository too. Build plans for parallel pull/push, sequential status --short and origin plus optional GitLab pushes. Label mirror scope exactly; do not call arbitrary remotes included. Offer fetch --prune and gc independently of merged-branch availability. Resolve origin/HEAD then main/master fallback; explain unavailable default.

**Architecture / reusable components:** Use canonical repository/worktree identity, per-repo remote identities and explicit concurrency/failure policies. Merged cleanup reviews sorted unique candidates, current/default exclusions and visible 30-of-N bound; execute git branch -d --, never implicit force. Revalidate branch/worktree/merge state before execution. Reuse Picker, Plan/StepRail and existing facts dialogs.

**Required deterministic fixture:** `parity-git-batch` — zero/one/many immediate repositories, colliding leaf names, failed repo, absent/present GitLab, custom default, no default, no merged branches, >30 branches, branch occupied by another worktree, stale merge eligibility.

**Acceptance / automated verification:** Assert every intended repo/remote operation and cwd, independence versus ordered status, retained per-repo failure output, no hard-coded default, exact delete argv and protected current/default branches. A failed prerequisite blocks only dependents; independent batch peers still run. Test stale confirmation revocation and Git refusal without -D fallback. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture batch selection, mirror preview, mixed results, capped branch review and stale/worktree refusal.

**Dependencies:** HP01/HP05/HP14/HP15/HP21; F02/F05/F23.

<a id="hp07"></a>

#### HP07 — Native project-task adapters

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve package.json, Just, Make, Taskfile and mise tasks. Current pnpm-looking tasks are mise fixtures, not native adapters.

**Source evidence and mandatory scope:** [matrix HP07](docs/holla-parity-matrix.md#hp07--native-project-task-adapters) — `OP20`, `OP21`, `OP22`, `OP23`, `OP24`, `OP25`, `OP26`, `OP27`, `OP28`, `OP29`, `OP30`, `OP58`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Each task resource states defining file, runner, effective cwd, description, provenance and exact argv. Preserve package lock precedence pnpm/yarn/bun/npm, sorted script names and string-only values; Just summary discovery; conservative Make declaration order; Taskfile JSON discovery; uncapped ordered mise descriptions. Preserve visible first-30 limits where legacy has them, or provide an explicit more-results control. Missing runners/discovery errors must be truthful rather than invented task success.

**Architecture / reusable components:** Create bounded typed source adapters with no implicit shell expansion. Task execution still delegates recipe interpretation to its tool; trusted custom config does not automatically trust a project task file. Review newly introduced trust separately while preserving intentional task execution. Reuse existing task picker, argument form, trust page and activities.

**Required deterministic fixture:** `parity-task-sources` — each filename variant/parser, invalid/missing manifests, runner absent, duplicate tasks, 31 entries, package lock conflicts, whitespace/Unicode/metacharacters, description-only mise output, failed discovery, child versus ancestor cwd.

**Acceptance / automated verification:** Assert exact IDs, order/cap/source diagnostics and argv for every OP20–OP30 variant. Assert Make never executes during discovery and rejects unsupported target syntax without claiming a full Make parser. Assert failed mise discovery cannot consume misleading stdout as authoritative. Test original cwd and explicit trust cancellation; launch-time args are not runtime stdin. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture one native adapter per source, capped results, parse/unavailable state, provenance review and running task in child cwd.

**Dependencies:** HP01/HP14/HP15/HP17; F01/F08/F23.

<a id="hp08"></a>

#### HP08 — Cargo build, test, lint and clean

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve cargo build, test, clippy --all-targets --all-features and cargo clean. Existing generic output and target-path assumption are insufficient.

**Source evidence and mandatory scope:** [matrix HP08](docs/holla-parity-matrix.md#hp08--cargo-build-test-lint-and-clean) — `OP15`, `OP16`, `OP17`, `OP18`, `OP19`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Expose all four legacy choices as Cargo alternatives with exact flags and cwd. Keep richer check/fmt/run/nextest choices as category B. Provide distinct build/test/lint success/failure output; clean must state tool-native permanent effects rather than Trash. Resolve actual target ownership so the modeled effect matches the command; shared/custom targets must not produce a false deletion claim.

**Architecture / reusable components:** Use Cargo action specification and semantic task effects; cleanup target resolution is shared with artifact observations, not id-prefix inference. Reuse activity output and safety review; no new component.

**Required deterministic fixture:** `parity-cargo` — cwd manifest/tool gates; clean and failed builds; passing/failing tests; all-targets/all-features clippy; target missing/custom/shared; clean refused/cancelled/succeeded.

**Acceptance / automated verification:** Assert all four argv vectors, distinct diagnostic/result states, correct target effect and no mutation before confirmation. Compare expected target ownership with post-run rescan; do not claim freed bytes from command success alone. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture Cargo alternatives, compile/test/lint failure and clean review/result with tool-native recovery wording.

**Dependencies:** HP01/HP07/HP14/HP18/HP21/HP22; F10–F15/F23.

<a id="hp09"></a>

#### HP09 — Docker and Compose outcomes

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve Compose up/down/finite logs, container stop-and-remove, full cleanup and builder prune. Full cleanup graph alone does not cover standalone variants.

**Source evidence and mandatory scope:** [matrix HP09](docs/holla-parity-matrix.md#hp09--docker-and-compose-outcomes) — `OP31`, `OP32`, `OP33`, `OP34`, `OP35`, `OP36`, `OP37`, `OP38`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Separate Compose project from host container resources. Include all four manifest names, docker executable/daemon/plugin states and finite logs --tail 200 independently of follow logs. Legacy docker.stop-all stops and removes captured running/stopped IDs: offer an explicit Stop and remove all plan retaining that outcome, alongside new stop-only/remove-only alternatives. Full cleanup includes all images, network/system/volume prune; builder prune remains independently callable. Explain named-volume and buildx expansion separately.

**Architecture / reusable components:** Resolve immutable Docker identities into typed argv and dependency barriers. Remove shell substitution, swallowed image errors and preview/execution drift; model effects for every standalone variant. Do not run remove after failed stop or use force implicitly. Use existing resource snapshots, plans, facts and activities.

**Required deterministic fixture:** `parity-docker` — empty daemon, stopped+running containers, daemon/query failure, IDs changed after review, failed stop/remove/image stage, finite versus follow logs, Compose down without volumes, standalone builder prune, complete cleanup.

**Acceptance / automated verification:** Assert exact target classes/argv and scoped counts, successful no-op versus discovery failure, per-stage effects and mixed results. Compare every standalone action with its own world mutation. Require reauthorization after target drift, truthful permanent deletion, failure visibility and dependency blocking while independent prune branches remain independent. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture each standalone variant review/result, finite log completion, daemon failure, stop failure barrier and full cleanup final inventory.

**Dependencies:** HP01/HP14/HP15/HP17/HP21/HP22/HP23; F23.

<a id="hp10"></a>

#### HP10 — Homebrew service lifecycle

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve Homebrew service discovery, cache and start/stop/restart. Existing remote systemd resources are a different provider.

**Source evidence and mandatory scope:** [matrix HP10](docs/holla-parity-matrix.md#hp10--homebrew-service-lifecycle) — `OP39`, `OP40`, `OP41`, `OP42`, `OP43`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Expose service resources on hosts with brew, including Linuxbrew. Support both array and services-array JSON forms, valid sorted unique names and all three verbs regardless initial status. Preserve 30-action/10-service bound visibly or allow explicit further discovery. Show cache age; refresh stale names before target-changing operations.

**Architecture / reusable components:** Versioned service-cache adapter preserves v1, 300-second TTL, tolerant corruption and atomic replacement. Bind action identity to service/host/verb and re-observe status after mutation. Reuse resource Picker/Props, alternatives and Activity.

**Required deterministic fixture:** `parity-brew-services` — both schemas, malformed rows, no brew, empty/failing command, fresh/stale/wrong-version cache, 11 services, started/stopped/error state and each verb.

**Acceptance / automated verification:** Assert cache hit avoids probe, expiry boundary/corruption refresh, cache write failure remains nonfatal, deterministic IDs/order/count, exact brew services argv and observed lifecycle outcome. Missing service after review must not retarget another row. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture macOS/Linuxbrew service alternatives, stale-cache refresh, failed stop and successful restart.

**Dependencies:** HP01/HP14/HP17/HP23; F02/F23.

<a id="hp11"></a>

#### HP11 — Gradle tasks, daemon and recursive cleanup

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve installed-Gradle clean/build/test and recursive .gradle/build cleanup with daemon stop. One wrapper-clean fixture does not cover these.

**Source evidence and mandatory scope:** [matrix HP11](docs/holla-parity-matrix.md#hp11--gradle-tasks-daemon-and-recursive-cleanup) — `OP44`, `OP45`, `OP46`, `OP47`, `OP48`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Keep Gradle command actions for build.gradle/build.gradle.kts with installed gradle. Wrapper preference may extend the new model but must retain legacy routes. Separate ordinary tool clean from depth-five Trash cleanup and global cache insights. Resolve exact candidates, review rebuild cost and show daemon stop as an explicit prerequisite; stop failure/unknown state must prevent unsafe cache deletion.

**Architecture / reusable components:** Share bounded candidate traversal with HP12 and deletion policy with HP20/HP21. Activity owns daemon state and cleanup report. Reuse tasks, candidates, Plan and facts; avoid standalone daemon widget.

**Required deterministic fixture:** `parity-gradle` — installed/wrapper-only/missing tool, both build files, clean/build/test success/failure, mixed build/.gradle candidates depth 5/6, symlink/node_modules exclusion, daemon active/stop failed, no candidates.

**Acceptance / automated verification:** Assert exact commands/cwd, selected directories only, no traversal through links, ignored node_modules, safe empty result, mixed failure report. Dry-run must not stop a daemon or mutate cleanup targets; explicit audit logging remains allowed. Distinguish tool-native clean from Trash and prerequisite failure from completed cleanup. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture three task choices, daemon-stop barrier, Trash candidate plan, dry-run and failed prerequisite.

**Dependencies:** HP07/HP12/HP14/HP18/HP20/HP21/HP22; F23.

<a id="hp12"></a>

#### HP12 — IntelliJ metadata cleanup

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve IntelliJ .idea/.iml cleanup and shared bounded filesystem discovery. The preview has no corresponding provider.

**Source evidence and mandatory scope:** [matrix HP12](docs/holla-parity-matrix.md#hp12--intellij-metadata-cleanup) — `OP49`, `OP50`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Discover when .idea exists or idea is installed, then review exact .idea directories and lowercase .iml files under cwd to depth five. Keep no-match success visible. Do not imply IDE shutdown exists in the baseline; new process-aware guard policy must explain any additional block. Combine overlapping candidates once and preserve per-path errors.

**Architecture / reusable components:** One candidate walker serves IDEA and recursive Gradle: skips symlinks/node_modules, stops descending selected directories, stable sorted paths. One validated Trash executor handles mutation and report/log health. Reuse Disk candidate rows, Props, facts and Plan.

**Required deterministic fixture:** `parity-idea` — .idea marker versus executable, .iml-only applicability, depth-five boundary, nested .idea, mixed files, node_modules/symlink decoys, missing/unreadable entries, failed/skipped/log-failed deletion.

**Acceptance / automated verification:** Assert exact detection/enumeration, no duplicate/nested deletion, no-match no-op and up-to-five error summaries with full report access. Both callers must use the same safety boundary; do not silently discard logging failure or treat skipped targets as full success. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture IDE cleanup discovery, exact paths/Trash facts and partial-failure activity with log-health detail.

**Dependencies:** HP01/HP18/HP20/HP21/HP22; F02/F05/F23. HP11 shares this traversal contract.

<a id="hp13"></a>

#### HP13 — All legacy upgrade managers

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve Brew packages/casks, mise, Amp, Oh My Zsh and upgrade-all. Current apt+mise graph misses macOS workflows; standalone mise upgrade loops.

**Source evidence and mandatory scope:** [matrix HP13](docs/holla-parity-matrix.md#hp13--all-legacy-upgrade-managers) — `OP51`, `OP52`, `OP55`, `OP53`, `OP54`, `OP56`, `OP57`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Build host-specific plan from detected managers, and keep each manager directly searchable. Preserve Brew update, greedy --yes upgrade, cleanup, autoremove, doctor; cask variant only macOS. Preserve amp update, mise upgrade and sh <resolved ZSH>/tools/upgrade.sh. Show $ZSH override exactly. Aggregate independently runnable managers; review before apply and keep per-stage output.

**Architecture / reusable components:** Use shared typed stage specifications for standalone and aggregate variants; remove shell-chain duplication. Explicitly block dependent Brew stages after prerequisite failure while retaining independent managers. Refresh availability before execution and invalidate changed reviewed plans. Reuse existing upgrade Plan/Activity.

**Required deterministic fixture:** `parity-upgrade-managers` — every manager alone, all managers, none, Linuxbrew versus macOS casks, ZSH override/fallback, disappeared executable, failed update/doctor, independent successful manager, standalone mise follow-up.

**Acceptance / automated verification:** Assert every command/flag/order/cwd and exact preview, correct availability, parallel manager branches and blocked dependent stages. Standalone mise Upgrade must reach execution and change versions. Report partial upgrade and verification failure without generic success. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture macOS aggregate plan, cask-only route, custom ZSH path, failed Brew prerequisite, independent completion and mise upgrade result.

**Dependencies:** HP01/HP14/HP15/HP17/HP23; F23.

<a id="hp14"></a>

#### HP14 — Task execution, output and results

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve real task specifications, sequential/parallel scheduling, live output, focus and summaries in activities. Current fixtures cover selected scripts, not all execution semantics.

**Source evidence and mandatory scope:** [matrix HP14](docs/holla-parity-matrix.md#hp14--task-execution-output-and-results) — `L23`, `E25`, `E26`, `E27`, `E28`, `E30`, `E31`, `E32`, `E36`, `E41`, `E42`, `E44`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Model queued/running/cancelling/succeeded/failed/cancelled with exact task identity, cwd/host and result. Independent batch jobs continue after peer failure; dependent plan steps block. Preserve per-task retained output, manual reading position and follow-tail, post-completion inspection, empty batch and final shell summary. Every mutating action needs an explicit effect or failure; unknown scripts must not silently succeed.

**Architecture / reusable components:** Replace generic-success fallback and incomplete effect-by-ID switch with typed execution events and action-owned outcomes. Separate runner completion from UI dismissal. Reuse Activities, tabs, TextViewport, StatusBar and Plan; adopt F10–F15 retention contracts.

**Required deterministic fixture:** `parity-executor` — sequential and parallel jobs, first failure, queued cancellation, missing executable/cwd, CRLF/ANSI/invalid UTF-8, no-final-newline, fast exit, stream interleave, burst retention, scrolled output, empty jobs and mixed results.

**Acceptance / automated verification:** Assert final buffered bytes arrive before terminal state, per-task ordered output, no cross-task retargeting, tail preservation, return-to-Here continuity and task/page focus bindings. Assert aggregate CLI failure and truthful UI result even after dismissing output. Test independent versus dependent failure policies separately. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture pending/running/mixed failures, retained scrolled output, final unterminated output, empty batch and completed task after returning from Here.

**Dependencies:** HP15/HP16/HP17; F10–F15/F19/F20/F21/F23. Task ownership, cancellation, stale-result rejection and shutdown are required; a generic worker framework is outside this plan.

<a id="hp15"></a>

#### HP15 — Runtime input, cancellation and terminal ownership

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve prompt input, password attention, cancellation/reaping and terminal restoration. Attached monitor fixtures currently consume ordinary keys without a response model.

**Source evidence and mandatory scope:** [matrix HP15](docs/holla-parity-matrix.md#hp15--runtime-input-cancellation-and-terminal-ownership) — `E29`, `E33`, `E34`, `E35`, `E38`, `E39`, `E40`, `E43`, `E37`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Add explicit task input mode separate from launch arguments and screen navigation. Show background password/confirmation attention, select the owning task without silently sending keys, and retain task/cwd/host identity. Forward Unicode, CR/DEL/Tab and supported control bytes; Escape returns to controls. Protect secret input from echo/history/inspection/copy. Stop request enters Cancelling; completion waits for owned process cleanup.

**Architecture / reusable components:** One session supervisor owns task PTY, stdin arbitration, process group and output drain across UI lifetime. Future Unix adapter uses controlling terminal and group cancellation: TERM, 750 ms escalation, KILL/reap while ownership remains valid; Linux subreaper versus macOS group extinction are explicit. Scope excludes deliberately escaped sessions. Reuse input/edit affordances, masked Input, Activity and Cancel-default dialog; no claim of full terminal emulation.

**Required deterministic fixture:** `parity-task-input` — partial no-newline Password:, repeated/background prompt, q/h/j/i payload, Ctrl-C/D/Tab/Unicode, task finishes during input, stdin EOF, two prompting tasks, resistant child, spawn/cancel race, terminal/render failure.

**Acceptance / automated verification:** Assert exact input bytes and single target, no navigation leakage, no input after completion, redaction and no stale PID signaling. Assert queued tasks never spawn after cancel, no owned descendants remain before Cancelled, no output/report loss and restored terminal after failure. Run isolated PTY tests on macOS and Linux at operational gate. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture attention/input badge, masked prompt, keep-running/stop dialog, Cancelling and acknowledged completion; retain PTY restoration transcripts.

**Dependencies:** HP14/HP17/HP23; F01/F08/F10–F15/F21/F23.

<a id="hp16"></a>

#### HP16 — CLI list, run and doctor contract

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve bare launcher, browse, list, list --json, exact-ID run, doctor, help/version and exits. Preview parser currently ignores unknown arguments.

**Source evidence and mandatory scope:** [matrix HP16](docs/holla-parity-matrix.md#hp16--cli-list-run-and-doctor-contract) — `E01`, `E03`, `E04`, `E06`, `E07`, `E08`, `E09`, `E10`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Specify one CLI over the shared registry, with deterministic simulated adapters while in preview. Preserve JSON v1 fields id/label/group/danger and text listing, warnings on stderr, exits 0/1/2/3/4, global warning refusal for run, exact-ID lookup and confirmation requirements. Doctor exposes detected groups/counts, scan timing and config paths/health. Bare invocation remains Here; browse retains file/directory start semantics.

**Architecture / reusable components:** Use a typed parser and execution kind so GUI-required actions are explicitly identified; do not claim all registry actions are noninteractive. Preserve actual headless task routes and stdin. Separate explicit durable trust approval from one-run --yes, document changed behavior, and provide an explicit trust operation for unattended setups without changing existing list v1 fields.

**Required deterministic fixture:** `parity-cli` — help/version/invalid args, empty/mixed registry, exact/unknown ID, all risk-confirm-trust combinations, valid+invalid config, warning-before-lookup, known task output, stdin prompt/EOF and GUI-required action.

**Acceptance / automated verification:** Assert valid JSON-only stdout, exact schema/danger strings, diagnostics stderr and correct exit for every branch. No execution on refusal or parse error. Verify list and run resolve same ID/cwd/argv. Retain old commands; new explicit trust command and simulation selectors must have their own strict parser tests. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** CLI transcript captures: JSON/text listing, all exits, doctor healthy/warnings and stdin prompt. UI captures only for browse/Here handoff; no invented CLI screen.

**Dependencies:** HP01/HP03/HP04/HP14/HP15/HP17/HP23; F23.

<a id="hp17"></a>

#### HP17 — Custom actions, configuration and trust

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve global/project custom actions, declared risk/confirm, exact argv, scoped execution and durable content trust. Session path trust is insufficient.

**Source evidence and mandatory scope:** [matrix HP17](docs/holla-parity-matrix.md#hp17--custom-actions-configuration-and-trust) — `E11`, `E12`, `E13`, `E15`, `E16`, `E17`, `E18`, `E19`, `E14`, `E20`, `E21`, `E22`, `E23`, `E24`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Read XDG_CONFIG_HOME/holla/actions.toml or ~/.config/holla/actions.toml plus cwd .holla.toml; retain every required/optional [[action]] field and default. Present configuration resources, diagnostics with source/index, valid siblings, grouping and keywords. Reserve IDs against actual registry rather than stale hardcoded names. Show program/each argv/cwd/source/trust/risk before execution; explicit sh -c remains possible and clearly identified.

**Architecture / reusable components:** One validated action specification drives preview and execution; never interpolate argument strings. Bind trust to reviewed whole-file digest, origin/path and effective cwd/argv; re-read on execution so edits revoke review. Legacy digest-only trusted.json may be read as migration evidence, but broader path binding requires renewed review. Persist sorted/versioned approvals atomically; corruption untrusted, save failure no launch. Global config remains user-owned trusted input; inherited environment has no invented TOML overrides.

**Required deterministic fixture:** `parity-custom-actions` — minimal/full schema, invalid ID/type/blank field, duplicate global/project/builtin ID, malformed sibling/file, XDG/fallback, spaces/newlines/metacharacters, comment edit, identical bytes moved, config edited during review, corrupt store/write failure.

**Acceptance / automated verification:** Assert accepted argv preserved exactly, safe/mutating/destructive plus confirm combinations, precise cwd and no implicit shell. Review displays whole trust scope; cancellation changes nothing; unchanged authorized content survives restart, changed content/path re-prompts. --yes authorizes one run only; explicit durable approval supports automation and revocation without alias bypass. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture custom provenance, exact argv facts, source-index diagnostics, Cancel-default trust review and changed-definition rejection.

**Dependencies:** HP01/HP14/HP16/HP21; F01/F08/F09/F23.

<a id="hp18"></a>

#### HP18 — Disk measurement, progress and cache

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve trustworthy disk bytes, progressive scans, cancellation, error states and durable cached hints. Timed fixture GB counts lack scanner semantics.

**Source evidence and mandatory scope:** [matrix HP18](docs/holla-parity-matrix.md#hp18--disk-measurement-progress-and-cache) — `LD004`, `LD005`, `LD006`, `LD010`, `LD011`, `LD007`, `LD008`, `LD009`, `LD017`, `LD018`, `LD019`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Model file/directory roots, allocated versus apparent bytes, hardlink identity, symlink leaves, hidden entries, skip subtrees and inaccessible/dataless outcomes. Stream bounded updates after first paint without moving active targets. Cache age is a hint: root/depth-two v3 sizes, seven-day/mtime validation, pre-scan snapshot, changed-path omission, merge/atomic save; partial and deep-unverified data must remain labeled.

**Architecture / reusable components:** Introduce a filesystem observation adapter and immutable identity-bearing events used by simulation and later live scanner. Bound/coalesce producer queues and cancel/join workers on owner removal/rescan. Separate measured bytes, apparent size, estimate, observation freshness and physical free space. Reuse TreeView/progress/Props/StatusBar.

**Required deterministic fixture:** `parity-disk-scan` — sparse files, hardlinks, symlink dirs, hidden/skip roots, permission/missing/I/O/dataless errors, rapidly changing trees, bounded burst, cancel/rescan, old/new/missing cache nodes, stale TTL/mtime, two writers and corrupt cache.

**Acceptance / automated verification:** Assert recursive totals/counts and dedup saturating arithmetic, no symlink traversal, live partial/error counts, bounded queue, cancellation acknowledgment, no stale generation replacement. Assert exact cache schema/TTL behavior, no disk/history I/O before first paint, changed-path exclusion and valid concurrent merge. Do not label incomplete or depth-limited freshness exact. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture cached-first/live/partial/error/cancelled states, allocated/apparent differences and cache replacement preserving selection.

**Dependencies:** HP19/HP20/HP22/HP23; F05/F11–F15/F23.

<a id="hp19"></a>

#### HP19 — Disk tree, top files and selection

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve root overview/custom path, expanding size tree, noise folding, arbitrary selection and Spotlight top files. Current disk page is flat and ignores requested path.

**Source evidence and mandatory scope:** [matrix HP19](docs/holla-parity-matrix.md#hp19--disk-tree-top-files-and-selection) — `LD001`, `LD002`, `LD003`, `LD012`, `LD013`, `LD014`, `LD015`, `LD020`, `LD021`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Make Here/selected/custom root explicit and validate the path with recoverable inline errors. Offer actual hierarchical allocated-largest-first tree, apparent-sort alternative, parent percentages and presentation-only noise folding. Space selects files/directories; parent dominates descendants without losing stricter safety policies. Keep paths/focus/checks stable across sorting/live updates and rescan after cleanup. Add macOS global Top files as a distinct scope with tree-scan fallback on Linux.

**Architecture / reusable components:** Use observed tree identities and selection closure, not indices. Top-files adapter models >=100 MiB query, top 50, exact paths, NUL-safe records, 16-way stat and five-second timeout; unavailable differs from empty. Reuse TreeView, Picker/Input, Props, progress and existing Disk composition.

**Required deterministic fixture:** `parity-disk-navigation` — home roots/cached sizes, custom file/dir/missing/relative input, nested large tree, all 14 folded names, re-sort, overlapping selection/disappearance, delete refresh; Spotlight duplicates/stat error/timeout/empty/Linux.

**Acceptance / automated verification:** Assert root-filtered observations, expand/collapse, size sort and honest displayed units, fold toggles preserve totals, arbitrary multi-selection counts/estimates and no target drift. Assert cleanup route/reset/rescan and independent global top-files selection through same safety gate. Exercise arrows/Enter/Space/sort/fold/rescan and pointer equivalents. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture custom-path error, expanded/folded tree, overlapping selection, post-cleanup rescan and Top files unavailable/empty/list states.

**Dependencies:** HP03/HP18/HP20/HP21/HP22/HP23; F02/F05/F23.

<a id="hp20"></a>

#### HP20 — Complete cleanup insight taxonomy and guards

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve all 18 reachable cleanup categories, project artifact classifier, streaming sizing, age policy and process guards. Family enums and a few candidates are not full coverage.

**Source evidence and mandatory scope:** [matrix HP20](docs/holla-parity-matrix.md#hp20--complete-cleanup-insight-taxonomy-and-guards) — `LD022`, `LD023`, `LD024`, `LD025`, `LD026`, `LD027`, `LD029`, `LD030`, `LD033`, `LD034`, `LD035`, `LD037`, `LD039`, `LD028`, `LD031`, `LD032`, `LD038`, `LD040`, `LD041`, `LD042`, `LD043`, `LD044`, `LD045`, `LD046`, `LD047`, `LD036`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Implement every LD022–LD047 root/tool/platform/age contract. Category resources lead to candidate detail or one cleanup plan. Preserve rebuildable/old-only/review-first distinctions; review-first starts unchecked; too-recent/unknown-age gated items remain visible but ineligible. Keep Xcode/Simulator guards, restricted pnpm root resolution and explicit Gradle stop prerequisite. Global review and category-specific entry remain searchable.

**Architecture / reusable components:** Centralize policy with category/path identity, inherited strictest descendant restrictions, freshness and process observation; the same policy applies from tree, insight or custom cleanup entry. Failed process probes mean unknown, not not-running. Share scanner sizing with bounded concurrency three; propagate partial/inaccessible diagnostics. Reuse grouped rows, TreeView, Props, selection and plans.

**Required deterministic fixture:** `parity-insights` — one positive/negative fixture per 18 category; macOS/Linux; 7/30/90-day boundaries, future/unknown age, no home, rejected pnpm store, all 12 artifact names × indicators/decoys/depth-six/link/nested case, active/unknown process, partial sizing.

**Acceptance / automated verification:** Assert exact roots/detection and category actions, hidden mac-only categories on Linux, largest-first candidates, age/default/manual eligibility and skip recheck before execution. Recent ineligible items cannot become selected via Space/select-all or a different entry point. Assert safe empty versus unavailable/partial and no direct Docker disk-image candidate. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture category overview/detail, review-first warning, age-disabled row, unavailable process guard, partial scan and all category families across platform captures.

**Dependencies:** HP11/HP12/HP18/HP19/HP21/HP22/HP23; F02/F05/F23.

<a id="hp21"></a>

#### HP21 — Deletion authorization, dry run and recovery mode

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve deletion review, Trash default, explicit permanent mode, dry-run and path protections. Current gates protect fixture labels, not filesystem identity.

**Source evidence and mandatory scope:** [matrix HP21](docs/holla-parity-matrix.md#hp21--deletion-authorization-dry-run-and-recovery-mode) — `L20`, `L21`, `LD048`, `LD049`, `LD051`, `LD055`, `LD057`, `LD058`, `LD050`, `LD052`, `LD053`, `LD054`, `LD056`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Resolve paths, inherited policies, recovery mode and estimated effects before review. Keep Cancel default; permanent selection is explicit and resets authorization; broad operation adds typed target-bound phrase. Dry-run uses identical discovery/validation and emits Would remove, with no daemon stop, cleanup-target mutation or false Removed result; its audit-log write is explicit. Application-owned filesystem mutation has one shared boundary; external Cargo/Gradle/Docker effects disclose tool-native behavior.

**Architecture / reusable components:** Create immutable authorized cleanup specification carrying canonical parent/leaf identity, host/root/selection/mode/revision, category/process/age policy and typed dry-run outcome. Validate absolute lexical components and every LD053/LD054 deny rule, including ancestor containment of protected descendants. Reject symlink ancestors except exact macOS aliases; preserve selected-link-only semantics. Revalidate after sizing, never silently fallback from Trash to permanent. State the unprivileged-user threat model and remaining pathname race limit.

**Required deterministic fixture:** `parity-delete-safety` — every allowed/denied root, home/container/browser/cloud data, ancestor containing protected child, duplicate parent/child, Unicode/newline names, replaced symlink ancestor/leaf, missing/unreadable target, drift during sizing, Trash collision/exhaustion, mode change and dry-run.

**Acceptance / automated verification:** Assert no denied target or protected descendant mutates through any entry, canonical checks rerun at commit, leaf symlink target survives, retry only transient AlreadyExists (max three), no permanent fallback. Assert dry-run validates/logs exact would-results but triggers no process or cleanup-target effects; explicit audit logging is allowed. Gate invalidates on any material target/mode/policy change; alias/history/CLI cannot bypass authorization. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture selected path/mode facts, permanent toggle, typed gate, invalid/changed target, Trash backend failure and truthful dry-run result.

**Dependencies:** HP17/HP18/HP19/HP20/HP22/HP23; F01/F02/F05/F23. Safety boundary precedes wiring any new destructive action.

<a id="hp22"></a>

#### HP22 — Cleanup ownership, outcomes and operation log

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve cleanup progress, mixed outcomes, estimates, per-item operation logs and ownership. Current history records only simulated successful removal and falsely frees Trash space immediately.

**Source evidence and mandatory scope:** [matrix HP22](docs/holla-parity-matrix.md#hp22--cleanup-ownership-outcomes-and-operation-log) — `LD016`, `LD059`, `LD060`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Keep cleanup in an activity independent of modal lifetime. Before irreversible commit allow cancellation; after commit keep execution/report ownership until settled and make quit wait explicit. Report removed/trashed/would-remove/failed/skipped separately, with counts, path/reason detail and log health. Same-volume Trash preserves filesystem used bytes until emptied; estimates and measured capacity change remain separate.

**Architecture / reusable components:** Typed result carries mode, dry-run, selected/executed identities, estimated size and observed outcome through worker, UI and log. Preserve JSONL v1 per requested item (including duplicates/protected/process-skipped) at XDG cache or ~/.cache/holla/ops.log: timestamp_ms/mode/path/size/outcome/error. Log failure is visible and does not invent rollback of completed deletion. Keep worker handle/report outside confirmation enum.

**Required deterministic fixture:** `parity-cleanup-results` — mixed success/fail/skip, duplicate, permission change, vanished target, process skip, dry-run, log write failure, receiver failure, result arrives during quit dialog, Stay/Leave and Trash versus permanent capacity.

**Acceptance / automated verification:** Assert every requested path gets one correct audit outcome; no success-only record after failure, correct first-six/overflow/full-log summary, persistent restart readability and JSON escaping. Assert no lost worker/report on modal replacement; quit completion is acknowledged. Apply only successful effects and rescan; reflect APFS clone/purgeable/hardlink estimation limits honestly. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture running/quit-wait/finished, mixed report with log failure, dry-run history and Trash estimated-versus-physical capacity.

**Dependencies:** HP14/HP15/HP18/HP20/HP21/HP23; F10–F15/F21/F23.

<a id="hp23"></a>

#### HP23 — Platform and terminal capability contract

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve implemented macOS/Linux and terminal behavior while keeping explicit exclusions separate. Current platform fixtures do not cover legacy fallbacks.

**Source evidence and mandatory scope:** [matrix HP23](docs/holla-parity-matrix.md#hp23--platform-and-terminal-capability-contract) — `X01`, `X02`, `X03`, `X04`, `X05`, `X06`, `X07`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Provide host-specific fixtures for every platform-sensitive matrix row: macOS casks/insights/dataless scanning/Spotlight/native Trash/open/reveal; Linux portable tools, hidden mac-only actions, FreeDesktop Trash, xdg-open fallback and process subreaper. Do not add apt/dnf/systemd-user as an old requirement. Do not call Windows, plugins, release infrastructure or nonexistent CLI strings parity gaps.

**Architecture / reusable components:** Platform capability adapter supplies explicit available/unavailable/error and native-path/store resolution. Preserve distinct XDG paths for frecency/sizes/logs versus dirs-based service/trust caches. Keep all effects simulated until production integration; no generic OS framework extraction without reuse. Apply Junie glyph/focus/no-color grammar to every added surface.

**Required deterministic fixture:** `parity-platforms` — macOS personal/development, Linux without desktop/trash mount helper, Linuxbrew, missing opener/OSC52, dataless policy failure, protected platform aliases, unavailable Spotlight and TERM/backend error.

**Acceptance / automated verification:** Assert all platform gates/fallbacks, no macOS command spawned on Linux, no destructive fallback, honest clipboard result and correct store location. At operational gate run controlled filesystem/PTY/Trash/open probes on both OSes; unavailable OS evidence remains explicitly pending, never passed by a macOS fixture. Run all relevant F21/F22/F23 terminal and provenance gates. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture platform availability/unsupported explanations in TrueColor/Mono/NO_COLOR; retain CLI/PTY/platform transcripts. No capture requirement for excluded CI or documentation-only features.

**Dependencies:** All HP01–HP22 platform cases; F21/F22/F23. This is a cross-cutting acceptance gate, not a new operating-system integration project.

### Delivery order and closure

1. Freeze row contracts and deterministic fixture inputs. Resolve typed action,
   filesystem identity, trust and execution ownership first (HP01/HP14–HP18/HP21/HP22).
2. Complete Files and disk resource navigation (HP03/HP04/HP19), then full insight
   policy (HP20). Add operational providers HP05–HP13 against shared contracts;
   independent adapters can progress alongside resource work.
3. Complete durable ranking/custom configuration and CLI contracts
   (HP02/HP16/HP17), without postponing their required safety boundaries.
4. Run per-row semantic, state and platform gates (HP23 plus F23). Review current
   screenshots and actual terminal evidence before approving changed baselines.

Dependencies above express shared contracts and integration gates, not a mandate
for a single giant patch. HP11/HP12 share traversal; HP14/HP15 share task ownership;
HP16/HP17 share trust/CLI semantics; HP18–HP22 share immutable cleanup identity.
Define these interfaces jointly, then implement reviewable vertical slices.

To close a Partial/Missing row, record its fixture/test and inspected capture
or CLI transcript, exact target/effect assertions, failure/cancel cases and
platform scope. A successful old test or similarly named new action is not
closure. To close a provider item, every mapped action variant must pass.
No real execution is enabled by this plan or by finishing this audit.

The independent verification report records the final source-to-row sweep,
Covered-row challenges, corrections and residual limits. Future baseline
updates must pin both revisions, repeat that sweep and classify new capabilities;
otherwise the current parity claim does not automatically extend to newer code.

Historical verification results are retained in the
[audit reference](docs/improvements-plan-reference.md#holla-audit-completion-evidence).

## Stop conditions

Planning history and its completion criteria are retained in the
[audit reference](docs/improvements-plan-reference.md#historical-planning-completion-criteria).
This plan and its linked references retain finding dispositions, evidence,
uncertainty, dependencies, compatibility decisions and acceptance criteria.
All local evidence links must resolve.

Eventual implementation is complete only when every mapped HP capability passes
its representation gate (and operational gate for a production release), every
accepted F-item is fixed or
its stated coverage proof is satisfied, all relevant gates pass, no known
introduced regression remains, and conditional O04/O05/O07 retain explicit unclaimed
status. Retired framework proposals are not completion requirements. A proven
tool/platform limit must state missing evidence and the next required external
check; it is not a passing result. Do not equate completing
this plan, passing the old suite, or updating hashes with implementing the work.
