# DiffView

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-editors.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W30 · Phase: P5 · Legacy family: C38.

## Purpose and exclusions

DiffView consumes already computed hunks, line kinds, original source ranges, file status, and path metadata through `DiffSource`. It renders a unified listing or a side-by-side review presentation, with shared TextViewport selection, scrolling, copy, and Unicode projection.

The component must not own Git/patch generation, repository access, or file mutation; a second text, scroll, selection, or grapheme projection engine; silent replacement of requested Review mode with a different durable mode; a fabricated success screen for an empty diff; or a universal widget trait or legacy `WidgetId`/`Outcome`/`RenderCtx` API. Computing Git diffs, reading repositories, accepting patches, mutating files, or deciding review policy are caller responsibilities.

## Public API

The signatures below are the proposed target, not a claim about current Rust exports. No signature is source-checked.

```rust
DiffView::new(id: Id, source: &'a dyn DiffSource) -> DiffView<'a>

update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut DiffViewState,
) -> Response<DiffAction>

draw(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &DiffViewState,
) -> Rect

measure(
    &self,
    cx: &MeasureCx<'_>,
    constraints: Constraints,
) -> Size
```

Builders:

```rust
mode(DiffMode::Unified | DiffMode::Review)
wrap(WrapMode)
patch(StylePatch)
```

The source model supplies stable hunk/line keys, original text/ranges, line kind (`Context`, `Add`, `Remove`), file status, and source revision. The target actions are:

```rust
DiffAction::SelectionChanged
DiffAction::CopyRequested(TextSelection)
DiffAction::HunkActivated { key: ItemKey }
```

Copy resolves original source content according to the selected side and mode; it never returns decorated gutters, diff markers, separator glyphs, or tab expansion.

`DiffViewState` owns requested `DiffMode`, shared `ViewportState`, keyed hunk and line navigation, selection, and any source revision projection marker. Effective narrow fallback is derived from available width and scrollbar budget; it does not overwrite requested Review mode. No `DiffFile`, source rows, or repository state is retained as mutable domain data.

## Ordinary use

Consumer recipe: [EX-12 — Code and diff views](../api/consumer-recipes.md) (`proposed_target`).

Real preview consumers are the Showcase audit/diff flows and the Jackin Preview inspect views. An independent synthetic fixture supplies hunks with stable keys, both line kinds, and file status metadata.

The caller supplies computed diff records with raw content and source keys; DiffView owns presentation, selection, scrolling, and copy mapping. The caller handles typed selection, copy, and hunk activation requests and performs any review operation itself. Narrow fallback is a documented component policy, not a captured-width answer table.

## Ownership

- The caller owns diff computation, repository access, patch acceptance, file mutation, and review policy.
- DiffView owns requested mode, shared viewport state, keyed hunk/line navigation, selection, and typed requests.
- `update` routes mode toggle, selection, copy, hunk activation, scrolling, keyboard, pointer drag, focus, and capture. Shared TextViewport mechanics own selection/copy/scroll behavior. Source revision changes invalidate generated display lines and stale selections by stable key.
- `draw` is immutable. It computes current available width, reserves scrollbar space before choosing Review, projects the source into styled lines, then delegates visible text, selection, cursor (if configured), fade, and scrollbar painting to the shared viewport. It must not toggle mode or alter offset as a render side effect.
- `measure` computes the file header, hunk/line projection, gutters, separator, and width threshold. The Review breakpoint includes both side gutters, minimum content cells, separator, and scrollbar width. A resize from wide to narrow draws the unified fallback; widening restores requested Review automatically.
- Focus and hover belong to the runtime/viewport owner; keyboard hover suppression and pointer restoration follow the shared runtime contract.

Shared dependencies: [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`text`](../foundations/text.md), [`collections`](../foundations/collections.md), and [`conformance`](../foundations/conformance.md).

## Customization

Advertised parts are `container`, `header`, `old-gutter`, `new-gutter`, `context`, `addition`, `deletion`, `emphasis`, `selection`, `scrollbar`, `fade`, and `empty`.

Ordinary customization patches declared addition, deletion, or emphasis tones. Part style patches alter only declared rows/cells. Geometry, source mapping, focus/capture, and whole-surface ownership are not slots. Preserve baseline header text, line-number spacing, `+`/`-` markers, context tones, added/deleted surfaces, intraline bolding, separator, empty message, scrollbars and fades. Default semantic colors resolve through the shared truecolor/256/16/no-color policy. Fades protect selected or otherwise emphasised cells and cursor rows.

Advanced customization keeps the mode recipes: unified mode renders file/status header, hunk header, old/new line-number gutter, context lines, additions, and deletions with baseline markers and tones; an empty file uses the shared Empty/readiness presentation and `(no textual changes)` semantics from the oracle. Review mode pairs old and new columns, leaves a blank side for unpaired lines, keeps a visible separator, and bolds changed grapheme runs. Tabs expand through the shared display mapping before width and emphasis calculations. Wide graphemes and combining marks remain complete on both sides. The requested mode survives all resizes. Widths around the exact fallback threshold are tested with and without the scrollbar. A narrow terminal never produces wrapped or negative pane geometry.

## Behavior

Keyboard and pointer selection, wheel, page movement, Home/End, drag selection, scrollbar capture, copy, and Escape selection clearing follow [`TextViewport`](./text-viewport.md). Hunk activation is keyed and typed. Click and keyboard selection produce the same source range. Pointer release outside ends capture safely.

DiffView does not invent a generic activation animation. It is read-only, so editing states do not apply.

Identity and reconciliation rules:

- Hunk and line identities are stable semantic keys from `DiffSource`.
- Selection and current line reconcile by key across source refreshes and mode projection; removed keys clear instead of clamping to another row.
- Mode changes may rebuild display lines but preserve requested mode, valid selection, and scroll intent where source keys still exist.
- A changed source revision invalidates stale byte ranges, display caches, and copy mappings before update or draw.
- Intraline emphasis maps original grapheme ranges to display cells; it cannot change source text or selection boundaries.

## Visual matrix

| Axis | Required states | Applies when |
|---|---|---|
| Geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | Always |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover | Interactive selection surface |
| Activation | No generic whole-surface activation; hunk activation and copy/selection are typed | Diff recipe |
| Scroll | no overflow, start, middle, end, wheel boundary, thumb drag, resize while scrolled, edge fade, protected row | Overflow exists |
| Output | source replaced, source appended, selection, source copy, cursor clipped | Source/selection configured |
| Review mode | requested Unified, requested Review, effective unified fallback, restored Review | Review is requested |
| Empty/readiness | empty/no-change, loading/error when supplied | Source supplies state |

Editing, disabled, and generic whole-surface activation states are not applicable and must not be fabricated.

## Verification

All required parity is pinned to `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Source evidence is `src/widgets/diff.rs` (blob `e0b5bdefa3b7fa514295a245de80ebf0f3bcf438`) and the Jackin Preview inspect composition in `src/bin/jackin_preview/screens/inspect.rs`. Inline tests cover unified/review lines, hunk/header counts, narrow fallback, resize recovery, scrollbar-aware width, Unicode intraline emphasis, tabs, selection, and raw-source copy. Visual consumers are Showcase audit/diff flows and Jackin Preview inspect views; discovery roots are `snapshots/showcase/audit` and `snapshots/jackin` with interaction traces in `tests/visual_baseline`. See [`visual-parity`](../verification/visual-parity.md), [`oracle-and-provenance`](../verification/oracle-and-provenance.md), and [`public-api`](../api/public-api.md).

Use [`../reference/capture-plans/diff-view.json`](../reference/capture-plans/diff-view.json). It is `planned_not_captured` with no approved expected artifacts. Baseline output is bound as `ExistingOracle`/`ExtractedOracle`; new API/safety guarantees are `Extension`.

Run applicable cases at `72×20`, `80×24`, `100×30`, `120×40`, and `160×50`, with truecolor, 256-color, 16-color, `none`, and `nocolor` capabilities.

| Case | Required observation |
|---|---|
| W30-01 | Unified/review with additions, deletions, context, and intraline emphasis. |
| W30-02 | Wide → narrow → wide restores requested Review. |
| W30-03 | Exact width around the split/scrollbar threshold. |
| W30-04 | Selection crosses tabs, wide graphemes, and hunk boundaries. |
| W30-05 | Empty/no-change and source revision replacement. |
| W30-06 | Click/keyboard selection equivalence and raw-source copy. |

Record exact symbols, wide-cell continuation, colors/modifiers, dimensions, cursor/focus/capture/layer owners, selected stable key, source revision, requested/effective mode, offset, action count and typed action target. Invalid oracle setup blocks capture.

Required negative tests:

- `draw` cannot change requested/effective mode, scroll offset, selection, or source revision.
- Review fallback cannot overwrite durable requested Review mode.
- Scrollbar width is included in breakpoint calculation; no negative or wrapped side geometry is possible.
- Copy cannot return gutter markers, separators, expanded tabs, or unrelated source text.
- Source refresh cannot apply stale selection/ranges to a different hunk/line.
- Unicode emphasis and selection cannot split graphemes or continuation cells.
- Empty diff cannot fabricate a success state.
- Part overrides cannot replace source mapping, scrollbar, focus, or capture.

Acceptance requires a compiling external consumer, source-state tests, exact applicable snapshots/traces, and independent review.

## Rejected use

Forbidden: painting old/new columns with padded strings and tracking a selected hunk index instead of using DiffView.

```rust
// Forbidden: preview owns diff rendering and hunk tracking.
ui.paint_str(row, 0, "- old line      | + new line    ");
if clicked { hunk = hunk_index; }
```

Rule: [ARC-012](../architecture/component-composition.md). Preview code must not recreate component rendering or generic interaction. The following patterns are prohibited in preview drawing paths:

```text
historical full-screen painters
snapshot files or exported frame data used as application output
hardcoded answer tables for 72x20, 80x24, 100x30, 120x40, or 160x50
fixture/scenario IDs that select a different painter
paused mode that bypasses component updates or layers
raw buffer access through termrock::author or an alias
direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls
padded strings that simulate columns, selection, fields, menus, or buttons
manual borders, focus gutters, scrollbars, or widget cursors
normal component drawing followed by a historical overlay
app-specific screens moved into a termrock crate under a generic name
fake state flags that display success without the normal component action
```

Use `DiffView::new` with a keyed `DiffSource` and handle the typed `DiffAction`.

## Known gaps

- Capture plan W30 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-003 historical editor and cockpit paths](../implementation/code-remediation-backlog.md): pending.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
