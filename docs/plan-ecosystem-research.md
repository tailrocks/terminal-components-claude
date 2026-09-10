# Ecosystem evidence for the improvement plan

Research date: 2026-09-10. Scope: the current `holla-fable` worktree only.
No other project branch or history was inspected. This extends
[the first research report](tui-audit-research.md), reconciles its findings with
[the completed audit](tui-audit.md), and uses the final coverage/API sections of
[the inventory](tui-audit-inventory.md). Earlier reports contain historical
snapshots; the local symbols named below were inspected again.

Primary documentation was fetched live. Rat-focus reported 2.1.1, rat-widget
3.2.1 and rat-ftable 2.2.0; other links are their projects' current documentation.
The observations below concern documented interaction/API contracts, not an
unsupported ranking of project quality. Terminal protocol, backend color output
and rasterizer limitations belong to the separate terminal/capture investigation.

## Source-to-capability map

### R1. Explicit keyboard selection in existing viewports

[Helix's keymap](https://docs.helix-editor.com/keymap.html#select--extend-mode)
defines a selection mode in which movement extends a range. Its selection
commands distinguish the cursor from the anchor and retain keyboard operations
for extending or collapsing the range. This is a useful accessibility pattern;
Helix's entire modal editing vocabulary is not a proposed replacement for Junie.

**Local evidence:** `TextViewport::on_key` in `src/widgets/viewport.rs:563`
supports scrolling, follow, copy and clearing selection. Range creation remains
in `on_click`, `on_drag` and `select_word_at`; there is no keyboard range-creation
path. `DiffView::on_key` delegates to it. Holla's `ActivityTab` uses the viewport
for output; Jackin's Capsule reads its selection for Copy selection and export
selected path. Existing editable controls already have keyboard selection, so a
second text editor is unnecessary.

**Classification:** confirmed coverage gap, P2. **Consumers:** Holla activity
logs, Jackin terminal panes and Inspect diffs, Showcase Terminal and Diff.
**Candidate:** extend the existing viewport with a keyboard browse caret and
selection operations, using its current logical-line/display-cell mapping and
copy event. This is a generic capability with multiple present consumers.
Keep terminal input forwarding an app decision; entering selection must be an
explicit action in a live pane. Do not turn navigation into implicit selection.

Acceptance must prove the same text is copied by keyboard and mouse, including
CJK, combining marks, emoji, tabs and wrapped lines. Define caret/anchor behavior
on resize, content replacement, append and scrollback eviction. Selection must
pause follow; leaving selection must not silently send its keys to the simulated
shell. Include empty and one-line output, narrow/wide panes, offscreen autoscroll,
monochrome selection, clear/Escape behavior, discoverable contextual hints and
Showcase interaction scenarios. No mouse-only command may be the sole entry.

### R2. Find within read-only output, without a second viewer

[Zellij's possible actions](https://zellij.dev/documentation/keybindings-possible-actions.html)
separate search input, next/previous occurrence and search options from ordinary
scrolling. They also expose copying a selection and opening scrollback in an
editor. These are separate operations, not evidence that every application needs
shell integration or external-editor launching.

**Local evidence:** `TextViewport` and `DiffView` expose no find query, match list
or next/previous-match action. Holla Activity filters service streams, which is
different from searching the visible text. Jackin Capsule's palette filters
commands; it does not search terminal output. `CodeEditor::FindState`, `refind`
and the shared `ui::text::find_ranges` already solve query-to-grapheme mapping for
editable/read-only code, but the find controller is owned by CodeEditor.

**Classification:** coverage gap, P2; the absence is confirmed, exact interaction
design remains proposed. **Consumers:** Holla activity/plan output and Jackin
Capsule/Inspect; Showcase can demonstrate the shared behavior.
**Candidate:** a bounded extraction of query/match navigation that can compose
with existing viewports and reuse the corrected source-grapheme mapping. Start
after R1's caret/follow contract is settled. Reuse TextInput and HintBar for the
query and match count; no new search-box widget is needed.

Acceptance: no-match/empty/query-edit states, forward/backward wrapping, match
visibility, query paste and Escape, content revision invalidation, Unicode
mapping, resize, and follow restoration. A real consumer must demonstrate two
uses before extracting a public controller. Regex engines, replacement, multiple
cursors, file search and shell-command boundaries are deferred: current needs do
not establish those broader semantics.

### R3. Binding ownership and discoverability

[prompt_toolkit's key-binding guide](https://python-prompt-toolkit.readthedocs.io/en/master/pages/advanced_topics/key_bindings.html#attaching-a-filter-condition)
documents predicates for individual bindings and conditional binding groups,
plus composition of independently defined binding sets.
[Yazi's keymap](https://yazi-rs.github.io/docs/configuration/keymap/)
documents separate manager, tasks, picker, input, confirm, completion and help
layers, with explicit first-match precedence. [Zellij's keybindings](https://zellij.dev/documentation/keybindings)
likewise partition actions by modes, including a locked mode.

**Local evidence:** `Hint` in `widgets/keyhint.rs:10` contains only display
strings. Four app shells independently decide active mode, modifier ownership,
popup/modal precedence and hints. The prior Ctrl+D collision is fixed by Alt+D
for duplication; that fix does not make binding declarations authoritative.
`HintBar::resolve` already selects the topmost hint layer. Typed widget outcomes
and events already express consumption and actions.

**Classification:** architecture/API weakness, P2. **Consumers:** all four
shells, especially TablePro editor/grid and Jackin pane/prefix contexts.
Use the interaction audit's reproduced collisions to drive a table of active
chords and exact precedence. Verify every advertised action is reachable in the
state displaying it and every inactive action propagates or rejects consistently.
A narrow shared binding descriptor becomes justified when it removes duplicated
actual matching/hint data. A universal command bus, wholesale keymap replacement
or dynamic widget trait is deferred: those change more than the proven owner
inconsistency and risk erasing existing typed events.

### R4. Focus state, event freshness and modal restoration

[rat-focus 2.1.1](https://docs.rs/rat-focus/2.1.1/rat_focus/)
collects an ordered focus set, supports next/previous/point focus and documents
rebuilding that set for each event. Its gained/lost-focus hooks make transitions
explicit. [Urwid's container API](https://urwid.org/manual/widgets.html#container-widgets)
offers a saved focus path that can be restored through nested containers.

**Local evidence:** `FocusRing` and `HitRegistry` already rebuild in render order
and have modal barriers. The runtime now renders after Changed events, correcting
stale hit/focus registration between queued keys. `Focus` owns only the current
WidgetId. Dialog focus save/restore remains app-owned. TextInput and TextArea
commit on losing focus inside `render`, so transition side effects depend on
whether the old control is rendered; a hidden-control failure requires the
interaction agent's reproduction before being called a defect.

**Classification:** existing core coverage plus architecture/API weakness, P2;
unproven lifecycle failures are hypotheses. **Consumers:** all shells and nested
editor/dialog flows. Preserve the existing ordering/barrier model. If a failure
is proven, move that transition to its responsible focus/owner boundary and
test hidden, removed, disabled and restored controls. Replacing Junie's focus
system with rat-focus, or adding a modal stack solely because another framework
has one, is rejected without a demonstrated missing contract.

### R5. Stable positions when collections change

[Urwid's dynamic ListBox documentation](https://urwid.org/manual/widgets.html#dynamic-listbox-with-listwalker)
explicitly warns that insertion above an integer focus position shifts its
meaning unless the owner updates it. Its custom walker examples use opaque
positions, including path-based identities, and resolve neighbors through the
data owner. This supports treating selection identity and display index as
different concepts.

**Local evidence:** ListBox exposes `items`, `checked` and cursor while keeping
its range anchor private. Showcase Settings mutates those public vectors.
`TreeView::set_children` rebuilds flattened paths after lazy expansion.
DataGrid local sorting updates its display order. The API investigation owns
executable proofs of stale anchors/cursor identity in these present flows;
those results, not this analogy, decide defect priority.

**Classification:** architecture/API weakness, P2; reproduced consequences are
tracked individually by the API investigation. **Consumers:** Showcase Settings,
TablePro explorer/results and any caller using lazy TreeView data.
Prefer replacement/removal/sort APIs with explicit preserve/reset identity
contracts and atomic repair of dependent state. Do not impose one generic row
identifier or collection framework across lists, trees, grids and styled text.
Acceptance includes removal above/at selection, lazy insertion, sorting, empty
replacement, disabled rows and stale hit IDs, with the current item preserved or
reset according to the documented operation.

### R6. Distinguish visible-row drawing from bounded data work

[rat-widget 3.2.1](https://docs.rs/rat-widget/3.2.1/rat_widget/)
keeps state/event handling and scrolling alongside its widgets; its rendering
discussion emphasizes avoiding unnecessary copying.
[rat-ftable 2.2.0](https://docs.rs/rat-ftable/2.2.0/rat_ftable/)
uses data adapters rather than eagerly constructing cell objects for all rows.
It explicitly cautions that an iterator without efficient skipping or row counts
may still traverse all data. Urwid's custom walkers similarly demonstrate lazy
loading rather than making laziness an automatic property of a list widget.

**Local evidence:** ListBox, TreeView, DataTable and DataGrid already draw visible
rows; TablePro already pages loaded rows. The previous viewport idle-cache defect
is fixed and measured. New content still rebuilds viewport grapheme/visual rows.
`ActivityTab::sync` in `holla/screens/activity.rs:74` reconstructs all filtered
output and calls `set_lines` on each length change. `max_lines(4000)` is configured
there, while viewport capacity enforcement currently occurs in `push`, not
`set_lines`; Plan output follows the same pattern with 2000. The text/API audit
owns the capacity and mutation probes.

**Classification:** confirmed defects/architecture weakness, P2. The text audit's
TXT-03 probe proves `max_lines(3); set_lines(8 rows)` and
`with_lines(8 rows).max_lines(3)` both retain eight rows. Its TXT-07 optimized
probe measured 20 replace-last/layout operations on 80-character rows at 80×20:
89,308 µs for 1,000 rows, 688,107 µs for 10,000, and 3,449,885 µs for 50,000.
These are single samples of a specific workload, not a general FPS claim.
Unchanged layout calls were below that probe's microsecond timer resolution.

**Consumers:** Holla activity/plan logs and Jackin pane output. Prove caps for
every content-entry operation first. Measure append, wrap resize and active
search/selection workloads separately; the idle-redraw benchmark cannot establish
those costs. Reuse append/replace-last paths or incremental caches where measured
work scales with retained content unnecessarily. Defer a generic virtual-data-source
trait until a present consumer cannot use the current paging/window APIs.

### R7. Durable operation state already exists

[Yazi's task commands](https://yazi-rs.github.io/docs/configuration/keymap/#tasks.inspect)
let users inspect failed-operation details and live stdout/stderr, and cancel a
task. The interaction keeps operation status and its evidence available together.

**Local evidence:** Holla already has named activities, service filters, output,
stop/restart and follow-up actions; plans have step output and failure propagation.
Jackin has launch/capsule activity; TablePro has running/cancelled/error result
states. StepRail, Progress, TextViewport, EmptyState and contextual hints already
support these compositions.

**Classification:** existing coverage. **Consumers:** Holla, Jackin, TablePro.
Verify terminal states remain inspectable, cancelled tasks stop progressing and
inactive actions cannot trigger. No new task-manager, toast, retry or error-card
widget is justified merely by these external examples. Real async cancellation
and stale response ownership remain the earlier deferred integration opportunity;
the deterministic fixture contract does not require a production worker system.

### R8. Copy behavior must name its destination

[Helix's clipboard commands](https://docs.helix-editor.com/keymap.html#space-mode)
distinguish editor yanking from system clipboard operations. Zellij's Copy action
also operates on a current selection. These illustrate why selection availability
and copy destination are separate responsibilities.

**Local evidence:** widgets already return payload-bearing Copy events; Holla and
Jackin own a simulated clipboard. Jackin app-level requests explicitly report
the preview clipboard, while Capsule's menu reports Selection copied after
writing the same in-memory world field. The interaction audit traced Inspect's
compact/advanced key paths (`inspect.rs:250/343`): both discard DiffView's Copy
event, neither has an explicit `y` handler, yet both hint lists advertise Copy
(`inspect.rs:749/772`). `CustomModal` exposes only an outcome and terminal
`done()` result; it lacks the nonclosing request path that app-level Copy needs.

**Classification:** confirmed copy-delivery defect, P2; the copy-label comparison
remains a separate consistency check. **Consumers:** Jackin Inspect directly;
nonclosing modal effects can also serve existing Info/Picker compositions when
needed. Propagate the existing `Request::Copy` payload through the responsible
custom-modal owner rather than creating a new clipboard subsystem. Test exact
payload and destination, unchanged modal visibility/focus, unavailable/empty
selection, and keyboard/mouse parity.
A real OS clipboard provider or terminal clipboard escape is deferred because
the current apps intentionally simulate side effects. Preserve that boundary
and avoid claiming a system copy occurred when only a preview buffer changed.

## Recommendation and exclusions

The strongest generic addition is keyboard selection in **the existing
TextViewport**, followed by a shared find-navigation capability only after two
current consumers prove its composition boundary. Neither requires a new viewer.
The most concrete structural work is transactional collection/content mutation,
followed by event/binding ownership changes supported by reproduced app failures.

No source justifies adding a new command palette, file picker, badge family,
notification system, universal widget trait, alternative theme or production
async framework. Existing components and typed events cover those compositions,
or the proposed system requires a real consumer absent from the fixture apps.
FTXUI was not added as another catalogue comparison: rat-widget, prompt_toolkit
and Urwid already supply primary evidence for the specific state, event and data
boundaries under consideration.

This document records research and plan candidates only. It makes no claim that
the proposed additions or newly identified backlog defects have been implemented.
