# API verification for the improvements plan

Scope: current `holla-fable` worktree only, September 10, 2026. This is a planning report. No product source was edited. Earlier findings were rechecked against current source, existing tests and temporary standalone probes.

## Inventory verification

The final [inventory](tui-audit-inventory.md) remains current. The SHA-256 over sorted library paths, NUL, file bytes and NUL is:

`40e412fd6bb7ff7700b90db054b0a4e5a5d535863e4eea4e55594f6bfb26202e`

It matches the compiler-derived inventory exactly: 47 library source files, 46 public modules, 31 widget modules, 23 showcase pages, 1,573 source-backed exported entries and 572 separately listed derived trait contracts. The exported-entry total comprises 668 functions/methods, 511 fields, 204 enum variants, 77 structs, 51 enums, 46 modules, 5 aliases, 1 trait, 6 free constants, 2 associated constants, 1 re-export and 1 explicit trait implementation. Individual test-only helpers, private palette contents and private runtime helpers are excluded. The earlier “849” scanner result is rejected as an API count; its extraction defects are documented in the inventory.

Focused revalidation passes:

- `rtk cargo test --lib widgets::table::tests`: 12 passed.
- `rtk cargo test --lib widgets::diff::tests`: 9 passed.

These focused checks prove their specific regression contracts, not the absence of unrelated remaining defects.

## Disposition of every previous API finding

| Previous claim | Current disposition | Current evidence and scope |
|---|---|---|
| API-1: DataTable replacement retains old edit/selection | Fixed | `src/widgets/table.rs:209` clears edit and selection before replacing rows. Tests at L935 and L956 cover empty/shorter/same-size replacement, sort and cursor. Additional source-identity tests at L986/L1003 cover sorting during editing; invalid-click preservation at L1022 passes. |
| API-2: ListBox exposes collection state requiring manual synchronization | Open; upgraded to a reproduced current-app defect | `src/widgets/list.rs:48-55` exposes items/checks/cursor but hides the range anchor. `src/bin/showcase/pages/settings.rs:579` manually replaces related state without resetting that anchor. Actual Settings flow now proves a panic; see PLAN-API-01. |
| API-3: DiffView missing from showcase | Fixed | `src/bin/showcase/pages/diff.rs` is registered through `pages/mod.rs` and `app.rs`. Existing DiffView, not a new widget, covers unified/review/empty, selection/copy and narrow fallback. `src/widgets/diff.rs:250` owns effective layout mode. Nine library diff tests pass. |
| API-4: DESIGN incorrectly denies existing context-menu/diff capability | Fixed | `DESIGN.md:860` documents DiffView; L1285 now excludes only toast/generic badge. README links the audit and inventory and labels the reusable-library ideas as hypotheses. |
| API-5: common mouse/focus bookkeeping copied across shells | Partly fixed; broader abstraction remains a hypothesis | Shared `PageEvent::Press` now fixes exact drag origin for Showcase Diff/Terminal. First-click focus handling and modal tests also improved. Four application shells still own bookkeeping. Duplication alone does not justify replacing all event routing or erasing typed events. A future extraction needs reproduced common behavior and parity tests across all four apps. |

### README hypotheses

`README.md:344-361` explicitly marks these as hypotheses.

| Hypothesis | Disposition | Necessary evidence before implementation |
|---|---|---|
| One dynamic Widget trait over render/on_* | Unproven; reject as an automatic task | Current typed events differ deliberately: GridEvent, PickerEvent, TreeEvent, InputEvent. Showcase already has the composition-level Page trait. Require at least two real compositions that need heterogeneous storage and a design retaining typed results, focus ordering and event consumption. |
| Theme trait or second theme | Optional investigation, not a correctness requirement | Theme already exposes semantic tokens and resolvers; no current consumer needs a different implementation. A second concrete theme can test hard-coded assumptions without introducing a public trait. Preserve Junie as the product contract. |
| Configurable gutter/marker glyphs | Unproven; reject arbitrary customization as a required change | Existing glyphs encode focus/selection semantics. `Theme::gutter_symbol` at `src/theme.rs:391` centralizes the focus glyph decision. A real alternate glyph family must demonstrate display-width, monochrome and accessibility behavior first. |
| Shared Container routing helper | Conditional opportunity | Preserve owner-specific outcomes, modal barriers and render-order focus. Prototype only after a shared defect has a test that the extraction would prevent. The narrow Press event is evidence that shared primitives can solve a specific class without a generic container framework. |

## New verified findings and bounded plan items

### PLAN-API-01 — Preserve ListBox invariants during collection replacement

Classification: confirmed defect. Priority: P1. Consumer: Showcase Settings Environment list.

Evidence:

- `src/widgets/list.rs:90` keeps a private range anchor and indexes every position between anchor and destination.
- `src/bin/showcase/pages/settings.rs:579` drains selected items, replaces `items`, replaces `checked` and clamps `cursor`; it cannot reset the private anchor.
- Real App probe: Environment's five initial variables; navigate to row 3 (zero-based), Shift+Down selects rows 3–4, activate Remove selected, return to list, Shift+Up. The application panics.
- This is not arbitrary invalid direct mutation: the probe executes the actual existing Settings handler and its supported controls.

Root cause: collection mutation is split across app-owned public vectors and a library-owned private selection endpoint. No replacement operation owns the complete invariant.

Proposed bounded work:

1. Add an explicit replacement/retention operation that updates items, checks, cursor, chosen item, scroll and range anchor together.
2. Migrate Settings add/remove paths to those operations.
3. Decide and document the compatibility boundary for direct public field mutation. Privatizing invariant-bearing fields would prevent the class fully, but is a public API change. If deferred, record legacy unchecked mutation as a remaining API weakness; do not claim it has been eliminated.

Risk/dependencies: medium API-compatibility risk, low visual risk. No new widget is needed. Coordinate collection accessors with all current readers before changing visibility.

Acceptance: actual App regression for the sequence above; replacement after range selection at beginning/middle/end; empty and shorter lists; disabled rows; checked-count and scroll consistency; keyboard and mouse removal; source API compatibility decision documented. A length clamp alone is insufficient because a stale valid index can still refer to a different item.

### PLAN-API-02 — Validate Picker secondary actions and tab identity

Classification: confirmed defect. Priority: P1. Consumer: TablePro Open tabs picker; reusable Picker contract affects all consumers.

Evidence:

- `src/widgets/picker.rs:153` checks Ready/nonempty/enabled before Enter activation.
- `src/widgets/picker.rs:195` emits `Secondary(cursor)` on Delete unconditionally.
- Standalone probes emit `Some(Secondary(0))` for empty, disabled and Loading pickers; disabled Enter returns None.
- `src/bin/tablepro/app.rs:1502-1510` handles Secondary by reading a row's display detail and falls back with `unwrap_or(i)` before `close_tab`.
- Actual App probe: connect Production, Ctrl+G, type `no-such-tab`, Delete. Open tab count changes from 1 to 0 despite no matching row.
- `open_tab_list` at `src/bin/tablepro/app.rs:1282` initially uses descriptive detail text; its query-refresh branch at L1462 repurposes detail as a decimal source index. This display-to-identity coupling enables the fallback.

Root cause: primary and secondary action eligibility diverge; the owner treats absent mapping as a valid target and stores identity in presentation text.

Proposed bounded work:

1. Centralize Picker action eligibility for Enter, alternate activation, click and Delete. Empty, loading, error or disabled entries must never produce an item action.
2. Give TablePro's tab picker an explicit owner-held mapping from visible result to stable tab identity. Reject missing mappings; never fall back to an unrelated index.
3. Keep query changes and active-row reset semantics explicit. Do not make every Picker automatically preserve cursor identity, because query changes intentionally select the first eligible result.

Risk/dependencies: low widget risk, medium owner-mapping risk. Tab close may discard editor state, so the owner rejection is required in addition to the widget guard.

Acceptance: actual TablePro empty-filter Delete keeps all tabs; disabled/loading/error Delete produces no Secondary; valid filtered Delete closes exactly the selected tab; clearing/retyping a query preserves correct mapping; a tab inserted/removed while a picker is open cannot retarget an existing result; mouse/Enter parity remains unchanged.

### PLAN-API-03 — Preserve TreeView cursor target after lazy insertion

Classification: confirmed behavior defect in a supported library operation, with an existing asynchronous consumer. Priority: P2.

Evidence:

- `TreeView::flatten`, `src/widgets/tree.rs:145`, rebuilds rows and merely clamps the numeric cursor at L203.
- `set_children`, L227, is the documented lazy-load delivery API and calls flatten.
- `src/bin/tablepro/workbench.rs:332` completes delayed explorer loads and calls `set_children` at L353; the user can move to another row before completion.
- Probe: tree contains lazy “Loading” followed by “Keep focus here”; move to sibling, then deliver child to earlier node. Cursor label changes from “Keep focus here” to “New child” without navigation.

Root cause: the cursor identifies a flattened row position rather than the node path already carried by FlatRow. Inserting visible descendants before that position changes its target.

Proposed bounded work: capture the cursor's path before flattening, then resolve it in the new flattened rows. If its node disappears or becomes hidden, apply a documented fallback, such as nearest visible ancestor then a neighboring surviving row. Preserve scroll visibility and separate explicit selected path from navigation cursor.

Risk/dependencies: medium interaction risk because collapse/filter intentionally changes visible rows. No universal stable-ID system is needed for child insertion under a stable parent path.

Acceptance: delayed child delivery while cursor rests on a later sibling keeps the same node; load completion while focus is elsewhere does not steal focus; collapse of an ancestor has predictable fallback; filtered and nested loads preserve visible target when possible; keyboard and mouse toggling behave consistently.

Limit: positional paths cannot preserve semantic identity if the owner reorders/replaces siblings. That is a separate replacement-identity contract, not proof that arbitrary public `nodes` mutation is currently supported.

### PLAN-API-04 — Preserve DataGrid cursor identity across local sorting

Classification: design inconsistency / confirmed target change. Priority: P2. Consumer: TablePro query-result grids.

Evidence:

- `src/widgets/grid.rs:1176`, `request_sort`, permutes display order without remapping cursor or display-range anchor.
- `src/bin/tablepro/tabs.rs:2075` enables local sorting on query results.
- Probe: fully loaded rows beta, alpha; cursor on beta; invoke the supported `s` key. Current value becomes alpha because cursor remains at display index zero.
- By contrast, `DataTable::sort_by` at `src/widgets/table.rs:242` explicitly preserves the source row.
- Grid pending changes and selected rows are keyed by source row (`src/widgets/grid.rs:184`, L341), so this finding is about cursor/range identity, not a claim that those stores are corrupted by sorting.
- Related comparator evidence: `apply_local_sort` at L502 reads stored row cells, while rendering uses effective pending values through `value` at L428. The probe records beta → aardvark, then cycles to ascending; displayed values are `["alpha", "aardvark"]`. This proves the distinction; the intended ordering policy for unsaved edits needs an explicit decision.

Root cause: display index is retained across a permutation, although source index already identifies the same record within the loaded dataset.

Proposed bounded work: preserve source cursor through ascending/descending/unsorted transitions; define whether a rectangular display selection follows its rows or clears on reordering. Preserve its source-row checks and pending edits. Decide whether sorting uses visible effective values or committed values; the UI must not imply one while silently sorting the other.

Risk/dependencies: medium interaction risk; depends on explicit range-selection policy. Shared row-state abstraction is not required.

Acceptance: cursor keeps the same source row through all sort directions, including duplicate values; selected source rows and pending edits stay attached to originals; range-selection behavior is documented and tested; sort after keyboard/mouse selection remains predictable; pending text/number/Null values follow the declared comparator policy before and after commit/discard.

## API boundary weaknesses, not proven current-app failures

### PLAN-API-05 — Define row/schema mutation contracts before hiding state

Classification: architecture/API weakness. Priority: P2 after reproduced failures.

`DataTable::new` at `src/widgets/table.rs:136` and `set_rows` at L209 accept arbitrary nested vectors. Sorting indexes `rows[a][col]` at L225. A probe supplies two columns but one cell per row; sorting column 1 panics. DataGrid's `value` at `src/widgets/grid.rs:428` substitutes Null for missing cells, while local sort at L502 directly indexes row cells. The library currently has inconsistent handling of the same malformed shape.

No current TablePro/showcase data producer was shown to emit ragged rows. This is an adapter-boundary weakness, not a claim that normal tables presently crash. Likewise, arbitrary writes to public `cursor`, `columns`, `rows`, `edit` or selection collections can violate invariants; synthetic corruption alone is not evidence for a user-facing failure.

Plan: state the accepted rectangularity/index contracts, then choose a checked ingestion API or a documented normalization policy. Rejecting malformed rows must preserve the previous dataset and its edit state. Additive checked APIs can precede a visibility migration; keep public-API changes deliberate and migrate actual consumers. Do not silently pad with Null unless missing and Null have the same intended meaning.

Acceptance: table/grid adapters receive zero columns, empty rows, missing/extra cells, stale edit columns and invalid source indices through supported boundaries; results are documented errors or intentional normalization, never an unexplained panic. Existing well-formed rendering/sorting/editing remains unchanged.

### PLAN-API-06 — Keep replacement identity scoped to actual consumers

Classification: conditional architecture opportunity. Priority: P3 investigation.

Picker `set_items` at `src/widgets/picker.rs:103` intentionally resets to the first enabled item; query refresh in TablePro and Jackin relies on it. Holla activity refresh at `src/bin/holla/app.rs:331-333` restores the previous numeric cursor after replacement. Preserving that number would not preserve the selected activity if an earlier row is inserted, but no current in-modal scenario producing that insertion was proven here. Record this as a hypothesis requiring an actual asynchronous flow, not an established Holla defect.

Tree path identity, table source-row identity and WidgetId serve different purposes. WidgetId owns focus/hit regions, not domain identity. A single new global key type is not justified. PLAN-API-02 and PLAN-API-03 can use owner-held identities/path resolution first; consider a reusable keyed-collection helper only if two consumers need the same reconciliation policy.

Rejected blanket claim: every append after local sort is inherently broken. `DataGrid::local_sort` at L335 explicitly describes fully loaded data; `append_rows` at L462 has a different fetch-more contract. TablePro enables local sorting for capped results, which needs contract clarification before extending its pagination behavior. Showcase owns its existing fetch-more path. Do not silently redefine sorting as global order over rows that have not been fetched.

## Cross-audit coordination

The interaction audit owns query editing/paste, modifier leakage, keyboard selection and modal routing. This report owns Picker action eligibility and owner identity, avoiding duplication of its query-editor work. The text audit owns TextViewport public-content cache invalidation and the `max_lines` replacement bypass; those findings belong together in [text verification](plan-text-verification.md). The ecosystem research independently identified the real Holla bounded-log consumers. A broad focus-lifecycle abstraction remains a hypothesis until an actual hidden/unmounted editor flow proves the needed contract.

## Reproduction evidence

Temporary probes are outside the repository at `/tmp/tui-plan-api.3YUJ47`. They import current product source or the current library build; they do not patch product files.

- `probe.rs`: library contracts for ListBox, TreeView, DataGrid, malformed DataTable and Picker secondary actions.
- `showcase_probe.rs`: actual Showcase App/pages and Settings removal handler; result `actual_settings_removal_then_shift_panics=true`.
- `tablepro_probe.rs`: actual TablePro App/modules and TestBackend; result `tablepro_tabs_before=1 after=0`.

Compile pattern used:

```sh
rtk proxy rustc --edition 2024 /tmp/tui-plan-api.3YUJ47/probe.rs \
  --extern junie_tui=target/debug/deps/libjunie_tui-ffeb255397b0ab5a.rlib \
  --extern ratatui=target/debug/deps/libratatui-8fa0541cd99c980f.rlib \
  -L dependency=target/debug/deps -o /tmp/tui-plan-api.3YUJ47/probe
rtk proxy /tmp/tui-plan-api.3YUJ47/probe
```

The library artifact used is the current September 10 build, not the older September 8 artifact also present under target. Independent [design cross-verification](plan-design-verification.md) reproduces the Settings crash and unrelated TablePro tab closure through real terminal flows and captures (CV1/CV2), and reproduces TreeView focus drift with a separate public-API executable (CV3). Its tree proof additionally verifies that the previously focused node still exists after insertion.
