# TerminalView

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-terminal.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W44 · Group: Adapter boundary · Phase: P6.

## Purpose and exclusions

TerminalView is a generic prepared-cell terminal presentation component. It borrows a caller-provided terminal-cell snapshot, supported mode/selection metadata and stable history coordinates. It preserves cell styles, wide-cell continuations, cursor state and selection while providing typed interaction requests.

This is a new reusable adapter boundary extracted from the Jackin preview's app-local terminal pane. It has no standalone legacy widget family. Baseline application regions and new adapter behavior are verified in separate lanes.

Termrock does not become:

- a terminal emulator;
- a PTY runtime;
- a shell;
- a daemon;
- an agent-session manager;
- a terminal escape parser.

Those concerns remain external. A future application adapter supplies terminal cells and retains raw input/session/process state.

## Public API

Signatures are the target shape; builders may use the repository's consuming-builder convention.

```rust
TerminalView::new(
    id: Id,
    source: &'a dyn TerminalSource,
) -> TerminalView<'a>

TerminalView::update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut TerminalViewState,
) -> Response<TerminalAction>
TerminalView::draw(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &TerminalViewState,
) -> Rect
TerminalView::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size

TerminalView::interaction(TerminalInteraction)
    .selection(Option<TerminalSelection>)
    .dimmed(bool)
    .patch(StylePatch);   // chrome only; child cell colors remain intact

TerminalAction::Forward { token: InputToken }
TerminalAction::CopyRequested(TerminalSelection)
TerminalAction::OpenLink { key: LinkKey }
```

TerminalViewState contains a view offset and selection anchor in stable line/cell coordinates. The committed selection is caller-controlled. Safe read-only observations expose source-keyed selection/caret and reading offset; the component does not expose a global Buffer or source collection.

Forwarding returns the host-retained input token. The view cannot reconstruct bytes that the host already discarded.

## Ordinary use

Consumer recipe: [EX-15 — Terminal cells are content inside a component](../api/consumer-recipes.md) (`proposed_target`).

Existing consumers include frozen Jackin capsule views; their baseline output remains protected. The caller supplies the borrowed `TerminalSource`, owns the committed selection policy and clipboard/export, and retains raw input/session/process state. update handles scrolling, selection gestures, link/copy requests and input forwarding after the outer modal/prefix layer gives permission. draw blits the borrowed cells and reports inner viewport/cursor facts without performing I/O.

## Ownership

| Concern | Owner |
| --- | --- |
| Prepared cells, styles, continuation metadata and source history | Caller-owned TerminalSource |
| View offset and selection anchor | TerminalViewState |
| Committed selection policy and clipboard/export | Caller/application |
| Focus, hit testing and pointer capture | Runtime and layer owner |
| Terminal mode authority, parser, PTY and process lifetime | External adapter |
| Resize I/O | Host adapter after draw reports facts |

Outer modal or prefix ownership wins before forwarding. Normal terminal mode emits Forward; selection mode emits source-coordinate CopyRequested, subject to stale-history rejection. A link action carries a stable LinkKey and leaves safety policy to the host. measure and draw agree on viewport geometry and wide-cell handling.

Shared dependencies: [`identity`](../foundations/identity.md), [`input/actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md), [`layout`](../foundations/layout.md), [`text`](../foundations/text.md), [`authoring`](../foundations/author.md), [`session`](../foundations/session.md) and [`conformance`](../foundations/conformance.md). See also the [ScrollRegion contract](./scroll-region.md).

## Customization

Parts are container, terminal-cell, cursor, selection and link. Original cell foreground, background, modifiers, wide-cell continuation and combining symbols are preserved. The view does not recolor child terminal output to the Termrock theme. A dimmed presentation is applied only when the baseline overlay policy asks for it.

Unicode width and continuation cells come from the prepared source; Termrock must not reinterpret or split them. Theme patches apply to view chrome only.

Ordinary example: `TerminalView::new(id, &source).interaction(TerminalInteraction::Normal)`.

## Behavior

- Cursor visibility and position, selection highlight, clipping, scrollbars, edge fades and focus/inactive pane treatment are exact proof fields.
- update handles scrolling, selection gestures, link/copy requests and input forwarding after the outer modal/prefix layer gives permission. Modal/prefix ownership cannot be bypassed by forwarding.
- No generic whole-surface activation; scroll/selection gestures and typed requests apply.
- Selection/caret state cannot escape stable source coordinates. A stale history coordinate cannot copy a replacement line; source replacement cannot silently retarget a selection or cursor.
- draw reports viewport/cursor facts; it never resizes a PTY as a paint side effect. draw cannot perform resize, clipboard, link, daemon or process I/O.
- TerminalView cannot parse escape sequences or spawn/own a PTY.

## Visual matrix

| Axis | Required states |
| --- | --- |
| Geometry | normal, zero/tiny area, nonzero origin, exact fit, 1-cell short, long/combining Unicode, narrow then wide |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover |
| Activation | No generic whole-surface activation; scroll/selection gestures and typed requests apply |
| Scroll | no overflow, start/middle/end, wheel boundary, thumb drag, resize while scrolled, edge fade |
| Output | source replaced, source appended, selected cells, source copy, clipped cursor |
| Retention | history evicted, selection evicted, retained reading anchor |
| Modal/input | modal intercept, prefix intercept, normal forwarding |
| Resize/I/O | reports viewport/cursor facts; never resizes a PTY as a paint side effect |
| Color/protocol | original cell colors, continuation metadata and explicitly classified new attributes |

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen sources are frozen-at-4a79c0a2 ([capsule.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/capsule.rs) and [pty.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/sim/pty.rs)). The capsule and PTY files are preserved integration evidence. They do not make PTY/session code part of the Termrock library. Use [`../reference/capture-plans/terminal-view.json`](../reference/capture-plans/terminal-view.json), the [ScrollRegion contract](./scroll-region.md), [runtime](../foundations/runtime.md), [text](../foundations/text.md) and [conformance](../foundations/conformance.md); candidate code cannot create expected output.

| Case | Authority and observation |
| --- | --- |
| W44-01 | Extension/oracle subregion: borrowed cells with wide cells, combining symbols and cursor visibility |
| W44-02 | Extension/oracle subregion: focused, inactive and dimmed terminal regions |
| W44-03 | Extension/oracle subregion: modal intercepts; normal mode emits forwarding token |
| W44-04 | Extension/oracle subregion: source-coordinate copy and stale-history rejection |
| W44-05 | Extension/oracle subregion: viewport resize reports facts and performs no I/O |
| W44-06 | Extension disposition: new protocol attributes are explicitly outside baseline proof |

Record exact dimensions, symbols, continuation metadata, foreground/background, modifiers supported by the comparator, cursor position/visibility, focus and capture owners, selection coordinates, offset, source revision, action count, action target and forwarded token identity. No candidate code may create or approve its own expected artifacts.

Required negative tests:

- TerminalView cannot parse escape sequences or spawn/own a PTY;
- draw cannot perform resize, clipboard, link, daemon or process I/O;
- modal/prefix ownership cannot be bypassed by forwarding;
- a stale history coordinate cannot copy a replacement line;
- wide-cell continuation and combining symbols cannot be lost or recolored;
- selection/caret state cannot escape stable source coordinates;
- a candidate cannot generate or bless its own expected snapshot;
- source replacement cannot silently retarget a selection or cursor.

## Rejected use

Forbidden: replaying a stored frame file as terminal output instead of presenting borrowed source cells through TerminalView.

```rust
// Forbidden: preview replays stored frame data as output.
let frame = load_frame("capsule-01.snap");
blit_frame(row, &frame.cells);
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits "snapshot files or exported frame data used as application output". Use `TerminalView::new` with a caller-owned `TerminalSource` and handle the typed `TerminalAction`.

## Known gaps

- Capture plan W44 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
