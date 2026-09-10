# Rich-TUI design-system audit

Scope: `holla-fable` in `donbeave/terminal-components-claude`, starting at
`7962b7b9913cf8bb736f309815e20a5d9036e0d9` with a clean worktree on
2026-09-10. Only this branch's current files and initial HEAD were examined.
No other branch or history informed the audit. Applications remain deterministic
simulations; no external service, database, container, or command was operated.

## Coverage and design decision

The [inventory](tui-audit-inventory.md) records the initial 31 widget modules,
22 showcase pages, primitive APIs, interaction states, and direct usage in
Showcase, TablePro, Jackin Preview, and Holla. Its API appendix enumerates
declarations, callables and state fields with source locations. The final catalogue adds a
showcase page for the already-public `DiffView`; it adds no widget or dependency.
The compiler-derived final inventory contains 1,573 source-backed exported
entries, including all 37 production `TextBuffer` methods; 572 derived trait
contracts are recorded separately. The initial regex count was corrected rather
than treated as proof of API completeness.

The existing library already covers inputs, multiline editing, code, completion,
lists, trees, editable tables, grids, filtering, pickers, menus, modal decisions,
tabs, terminal/log viewports, diffs, splitters, progress, steps, quotas, properties,
status and contextual help. Composition is the appropriate extension seam.
The README's proposed widget/theme traits and configurable glyphs remain
hypotheses: four consumers already use public tokens and typed component events.

[Primary-source ecosystem research](tui-audit-research.md) compared Bubbles,
Textual, fzf, and Lazygit. It supported improving command reachability, modal
ownership, selection accessibility, and viewport behavior. It did not justify a
new command palette, file picker, notification widget, or asynchronous framework.

## Findings and implemented response

P1 means panic, data corruption, or an inaccessible required interaction. P2
means incorrect interaction, rendering, performance, or reusable API behavior.
Evidence below names stable symbols; linked detailed reports retain the initial
reproduction and distinguish source observations from executable proof.

| Priority / classification | Evidence and impact | Structural fix | Risk and verification |
| --- | --- | --- | --- |
| P1 confirmed defect | `TextBuffer::selection_lines` sliced one byte before a selection endpoint; selecting `日` panicked. Public offsets and word movement could split UTF-8 or combining clusters. | Clamp public offsets to grapheme boundaries; count selected lines without slicing arbitrary bytes; maintain cursor/anchor invariants after mutation. Replace selections atomically so neighboring clusters cannot move the insertion point. Normalize line endings at every input boundary. | Medium: all editors share this model. Unit regressions cover emoji, combining marks, CJK, arbitrary offsets, CRLF, word operations, and joining regional indicators. Independent 100,000-operation deterministic Unicode probe passed. |
| P1 confirmed defect | `CodeEditor::on_key` panicked on modified Enter; `refind` panicked searching `é` in `İé`. | Handle the full shared edit-action contract. Map transformed search text back to original grapheme ranges; use the same mapping for fuzzy matches. | Medium: search and editing. Focused regressions verify expanding lowercase mappings, Greek final sigma, grapheme matches, and every Enter modifier. |
| P1 confirmed defect | `DataTable::set_rows` retained an edit from the previous dataset; committing after replacement could panic or mutate unrelated data. | Cancel dataset-bound editing and selection at replacement, retaining sorting and navigation semantics. | Low. Empty, shorter, and same-sized replacement regressions; sort behavior retained. |
| P1 confirmed defect | Sorting during an invalid DataTable edit changed the meaning of its display-row index; correcting the draft then wrote into another source row. Invalid cell clicks in both table implementations also moved navigation and could replace the invalid draft. | Bind edits to stable source-row identity, matching DataGrid; failed validation prevents cell-navigation transitions. | Medium: public `EditState.row` is now explicitly a source index. Sorting, invalid-edit, keyboard correction and click-preservation regressions. No repository consumer depended on the old display-index meaning. |
| P1 confirmed defect | `TerminalSession::enter` enabled raw mode before any restoration owner existed; later setup failure left terminal state changed. | Construct a restoration guard before subsequent fallible initialization. Normal exit and partial initialization use the same idempotent rollback. | Medium: terminal lifecycle. Failure/panic/idempotence tests and isolated controlling-PTY checks verify original termios after normal quit and broken-output startup. |
| P1 confirmed defect / design inconsistency | Forms at 72×20 drew fields into the footer and options over actions. Sidebar painted only the first visible entries and consumed wheel without scrolling, hiding later pages. | Reserve actions before laying out form content; compact existing fields and textarea viewport. Give navigation its own existing `ScrollState` and scrollbar. | Medium: responsive composition and focus. Rendered before/after captures, minimum/normal/wide/mono containment and sidebar navigation tests. |
| P2 confirmed defect | `TextInput::validate` retained `Required` after correction; masked click used raw character widths; narrow rendering could underflow or spill. | Derive validation completely. Use the displayed grapheme geometry for cursor, scrolling, and hit mapping; bound every text region. | Medium: clipped and masked editing. Required-field keyboard/paste recovery, masked Unicode clicks, and exhaustive small-rectangle tests. |
| P2 confirmed defect | `CodeEditor::on_paste` edited the document while its find field was active; long find queries and tiny editor gutters drew outside their allocation. | Route to the active nested editor first; bound find query, match count, body, gutter, footer, and hardware cursor. | Low/medium. Document unchanged during find paste; nonzero-origin sentinel tests across widths and monochrome. |
| P2 confirmed defect | TextArea edited invisible long-line text; TextArea and CodeEditor immediately undid wheel/scrollbar movement while editing. | Horizontal text-area viewport shares cursor/click geometry. Follow the cursor after edits, movement, or resize; preserve manual scrolling and hide offscreen cursors. | Medium: scroll ownership. Unicode long-line, resize, paste, click, wheel and scrollbar regressions; existing Textareas showcase fixture extended. |
| P2 confirmed defect | `Theme::gutter` callers always emitted `▎`, hiding it with equal colors. Actual `NO_COLOR` exposed every hidden focus marker; color-only selection disappeared. | Unfocused/disabled gutters emit spaces; monochrome text selection uses reverse video. Read-only CodeEditor retains its navigable focus marker. | System-wide symbol change, intentional. Four-palette state tests and actual `NO_COLOR=1` terminal captures; focused/read-only/disabled states checked separately. |
| P2 confirmed defect | Choice controls wrote full labels/markers beyond narrow allocations; empty RadioGroup retained previous hit geometry. | Bound each row segment; clear stale geometry before early return; preserve compact selection symbols and avoid long-label integer overflow. | Low. Nonzero-origin allocation tests, tiny widths, long labels, hidden state, and keyboard/mouse selection tests. |
| P2 confirmed defect | Footer status disappeared when longer than the available row; Showcase duplicated the faulty reservation logic. | Shared key-hint renderer reserves a truncated status and edit badge; Showcase uses it. | Low: long-status presentation. Unicode, short-width, EDIT and status-priority tests; normal layout retained. |
| P2 confirmed defect | Showcase focused inputs on MouseDown, so MouseUp treated the first click as an edit request. Viewport drag selection started at first motion, losing the actual press origin. | Completed-click dispatch preserves previous focus. An explicit page Press event supplies the exact selection anchor to Terminal and Diff pages. | Medium: shell dispatch. First/second-click tests and real App/TestBackend drag tests with distinct press and first-motion coordinates. |
| P2 confirmed defect | Facts acknowledgement inputs rendered hit regions, but `Dialog::on_click` only dispatched plain prompt inputs. | Both dialog variants route through existing `input_mut`. | Low. TablePro acknowledgement click, paste, token validation, and explicit confirmation regression. |
| P2 architecture/API weakness, verified behavior fixed | TablePro owned Ctrl+D for Data/Structure, hiding the grid's duplicate-row action. | Add portable Alt+D duplicate alias and matching contextual hints; retain standalone Ctrl+D and TablePro's established chord. | Low. Plain Ctrl+D, Alt+D duplication, editable/read-only state, editing/modal suppression, undo/default-value tests. |
| P2 confirmed defect | Overflowing `TextViewport::render` repeatedly alternated full and scrollbar-reduced width, invalidating its single cache and rebuilding every grapheme twice per draw. | Cache the complete width/height/wrap layout constraints; invalidate on content or geometry changes. | Medium: layout caching. Fresh-layout equality, content/resize/wrap tests, and measured unchanged-frame reflow counts. |
| P2 coverage gap / design inconsistency | `DiffView` existed in Jackin Inspect but had no showcase page; DESIGN incorrectly denied both diff and context-menu components. Public `mode` assignment could bypass layout invalidation. | Demonstrate existing unified/review/empty states; keep narrow fallback in `DiffView`, expose effective layout mode, and include public mode in cache validity. Review columns say Old/New. Correct catalogue. | Low/medium. Both modes and palettes, small/wide/resize, direct public mutation, wheel, exact drag/copy, keyboard/mouse controls, and empty-state tests. |

Additional reproduced defects closed during verification:

- Queued page-change keys were processed before rendering rebuilt the focus
  ring, so a fast `] Tab Enter Ctrl+L` sequence behaved differently from delayed
  keys (P1 confirmed defect). The runtime now renders after a Changed outcome
  before dispatching the next event; ignored and consumed events still batch.
  A real focus-ring regression and byte-identical burst/delayed terminal replays
  prove the correction. This changes event scheduling without a new public API.
- Page hints and the Showcase shell both emitted Tab, duplicating the same key
  in the new Diff footer and several existing pages (P2 design inconsistency).
  The shell adds the fallback only when the page has no contextual Tab hint.
  A regression checks every page and refreshed interaction captures show one hint.
- Actual `NO_COLOR` made the pinned Crossterm backend emit an empty color reset
  after modifiers, erasing reverse, bold and underline (P2 confirmed defect).
  The shared runtime removes color metadata after semantic rendering when the
  backend disables colors. Monochrome disabled controls use DIM. A subprocess
  verifies emitted ANSI; live captures verify selection and disabled states.
- Table/grid cell editors mixed display-cell scrolling with scalar slicing;
  clicks omitted the horizontal offset (P2 confirmed defect). Shared
  `slice_cells` preserves graphemes and occupied-cell coordinates. Regression
  tests cover Unicode clicks and neighboring-cell containment in both palettes.
- Diff review emphasis split ZWJ and combining graphemes across spans; tabs used
  different widths during measurement and rendering, and scrollbars could cover
  the final review cell (P2 confirmed defects). Whole-grapheme emphasis, shared
  four-space tab expansion, and scrollbar-aware layout fix the common geometry
  cause. Regressions verify column alignment, changed-run emphasis and copied text.
- `ui::text::wrap` returned an empty line followed by an oversized grapheme for
  one-cell allocations (P2 confirmed defect). A glyph wider than the entire line
  becomes `…`; fitting graphemes stay intact. Tests cover widths 0–7, CJK, emoji
  and combining marks. Risk is limited to allocations narrower than one glyph.

Detailed evidence: [interactions](tui-audit-interactions.md),
[editor/search](tui-audit-editor.md), [inventory/API](tui-audit-inventory.md),
[research](tui-audit-research.md), and [verification](tui-audit-verification.md).

## Visual and behavior verification

The showcase baseline now covers all pages at 72×20, 80×24, 100×30, 120×40,
and 160×50 in truecolor, ANSI-256, ANSI-16, and monochrome. Baseline changes
must be reviewed against rendered output: hidden focus glyphs becoming spaces
affect many hashes without changing truecolor appearance. Form geometry, sidebar
visibility, long-line text, bounded controls, footer priority, and the Diff page
are intentional visible changes.

`tools/audit_shots.sh` reproduces representative real applications and component
pages in those sizes and palettes, with a separate actual `NO_COLOR=1` case.
The capture harness uses an isolated tmux socket and configurable output folder;
it reports rasterization failures instead of silently omitting PNGs.

Final combined-worktree gates pass:

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`: 328 passed across eight suites, including the 460-case showcase
  baseline and the public cell-slicing doctest; no skipped tests
- `cargo doc --no-deps` and `git diff --check`

The final capture matrix contains 250 complete sets across ten representative
fixtures, five sizes and five color modes. It was regenerated after the runtime
fix. A further [51 interaction capture sets](../shots/audit-flows/README.md)
exercise selection/editing, invalid forms, Diff review/drag/copy/empty states,
and TablePro's acknowledgement gate through real terminal events. The final
interaction rerun includes the contextual-hint correction.

Representative inspected evidence:

- [Minimum-size form under actual NO_COLOR](../shots/audit/showcase-forms_72x20_no_color.png)
- [Selected input with reverse video](../shots/audit-flows/inputs_selected_120x40_no_color.png)
- [Small invalid form](../shots/audit-flows/forms_invalid_80x24_none.png)
- [Diff review with Old/New columns](../shots/audit-flows/diff_review_120x40_none.png)
- [Wide mouse-selected diff](../shots/audit-flows/diff_drag_160x50_no_color.png)
- [Acknowledgement armed by typing and navigation](../shots/audit-flows/tablepro_ack_armed_120x40_truecolor.png)
- [TablePro minimum-size drawer](../shots/audit/tablepro-production_72x20_no_color.png)
- [Jackin account error/disabled/busy states](../shots/audit/jackin-accounts_120x40_256.png)
- [Holla plan at small size](../shots/audit/holla-upgrade_80x24_no_color.png)

The PNG renderer has a confirmed font/grapheme limitation: some CJK and emoji
appear as missing-glyph boxes, and its code-point width heuristic cannot prove
complex grapheme geometry. ANSI/text artifacts preserve the content; Ratatui
buffer, cursor, copied-text and column-alignment tests supply that proof. This
tool limitation is not presented as a library rendering failure or hidden by
changing fixture data.

For the measured 4,000-line viewport case, ten unchanged redraws fell from
93.290 ms and 40 redundant reflows to 386.667 µs and zero redundant reflows in
the optimized diagnostic run. This is a bounded measurement, not a whole-app
performance guarantee. Terminal startup failure and normal quit also restored
the complete original termios state in isolated controlling-PTY checks.

## Prioritized remaining backlog

| Priority / classification | Evidence versus hypothesis | Next step, risk, and acceptance |
| --- | --- | --- |
| P2 coverage gap | `TextViewport` supports keyboard scrolling/copy/clear and mouse selection; keyboard creation of a text range remains absent. This is an accessibility gap in an existing primitive. | Design a keyboard caret/selection contract with follow, wrapping, and resize semantics; reuse the viewport rather than adding another viewer. Medium interaction risk. Keyboard and mouse must copy identical Unicode ranges. |
| P2 architecture/API weakness | Public mutable collection/index fields in list, tree, picker and grid APIs place invariant repair on consumers. Concrete `DataTable::set_rows` defects were fixed; arbitrary external mutation is not proof that every component crashes. | Audit supported setters and document mutation contracts, then add bounded transactional setters where real consumers demonstrate need. Medium compatibility risk; preserve stable identity and stale-hit rejection. |
| P2 architecture/API weakness | Key hints and event dispatch remain separate data; modifier checks and modal lifecycle are implemented by several owners. The duplicate-row collision and acknowledgement routing were fixed. Further leakage is a hypothesis until reproduced. | Build focused real-app chord/overlay tests before introducing shared binding or modal abstractions. Medium routing risk; editing, popup and modal ownership must win consistently. |
| P3 optional opportunity | Production async workers, cancellation and stale results are not exercised by deterministic fixture apps. | Validate with a real asynchronous consumer before adding a runtime framework. No current component defect is claimed. |
| P3 optional opportunity | Widget/theme traits and configurable state glyphs may help a future consumer, but the audit found no current use case blocked by their absence. | Require a concrete composition or theming case; retain the current grammar and typed events. No speculative API break. |

The remaining backlog distinguishes absent coverage and architectural risks
from verified defects. It does not excuse a failed test or an introduced
regression in the delivered changes.
