# Design and interaction verification for the improvement plan

Scope: the current `holla-fable` worktree only, inspected on 2026-09-10.
This is a planning document. No application code, tests, or baselines were
changed during this verification pass. Existing final captures were inspected,
and the Text areas minimum-size flow was recaptured in an isolated temporary
tmux server. No other branch or history was consulted.

## Evidence reviewed

The current design contract remains `DESIGN.md`: focus is visible geometry;
hover does not take focus; green is reserved for focus, primary actions,
selection and activity; disabled and read-only are distinct; a supported
terminal size must retain usable interactions.

The following final images were inspected, rather than relying on the earlier
before-fix screenshots:

- `shots/audit/showcase-buttons_72x20_truecolor.png` and
  `showcase-buttons_120x40_truecolor.png`.
- `shots/audit/showcase-inputs_100x30_truecolor.png` and
  `showcase-inputs_160x50_none.png`.
- `shots/audit/showcase-textareas_120x40_truecolor.png`.
- `shots/audit/showcase-forms_72x20_no_color.png`.
- `shots/audit-flows/forms_invalid_80x24_no_color.png` and
  `inputs_selected_120x40_no_color.png`.
- `shots/audit-flows/diff_review_120x40_none.png`.
- `shots/audit/tablepro-production_72x20_no_color.png`,
  `jackin-accounts_120x40_256.png`, and
  `holla-upgrade_80x24_no_color.png`.

Fresh temporary evidence:

- `/tmp/plan-design.7CXPiQ/textareas_72x20.png`.
- `/tmp/plan-design.7CXPiQ/textareas_72x20_three_tabs.png`.
- Matching ANSI, text and cursor files accompany these images. Start
  `target/debug/showcase --page textareas --color truecolor` at 72×20;
  send `Tab Tab Tab`. Focus returns to navigation after visiting only Task
  description and Notes. Commit message, the editable error example, never
  becomes a focus stop at this size.

The renderer's known CJK/emoji font limitation remains a tool limitation.
Missing-glyph boxes in a PNG are not evidence of a library Unicode failure.

## Prior design findings: current dispositions

| Prior finding | Current disposition and evidence |
| --- | --- |
| Forms fields overwrite the footer at 72×20 | Closed for the reproduced case. `src/bin/showcase/pages/forms.rs:136` reserves actions and uses compact field geometry. The final minimum-size actual-NO_COLOR image shows name, description, reviewer, options and actions within the page. `form_keeps_all_controls_inside_small_normal_and_wide_page` passes again. |
| Long toggle labels escape their allocation | Closed for the reproduced case. `src/widgets/choice.rs` now bounds the switch, label and state; final Forms captures show ellipses rather than overlap. Current public containment tests cover additional small/empty geometry. |
| Later sidebar pages disappear at minimum size | Closed for the reproduced case. `src/bin/showcase/app.rs:934` uses `ScrollState`; keyboard reveal and independent wheel scrolling are tested. `minimum_sidebar_reveals_last_page_and_wheel_does_not_move_focus` passes again. |
| The first click starts editing | Closed for the reproduced Inputs, TextAreas, Forms and Code editor cases. Completed-click dispatch in `src/bin/showcase/app.rs:693` preserves the prior focus. The four-page regression passes again. This is not a claim that every application's pointer routing is identical. |
| Long footer status disappears | Closed. `src/widgets/keyhint.rs:92` truncates to remaining display width. Showcase uses the shared renderer. All three keyhint tests pass again, including narrow rectangles and long Unicode error status beside EDIT. |
| Multiline field advertises Enter Commit / Esc Cancel | Closed for Forms. `src/bin/showcase/pages/forms.rs:376` emits Esc Finish while its textarea edits. Other multiline help should remain in the state-reachability audit below. |
| Actual NO_COLOR exposes hidden focus bars and loses selection | Closed for inspected final flows. Final actual-NO_COLOR input selection visibly retains reversal, underline and the single live focus marker; disabled API token remains dim. `Theme::gutter_symbol`, disabled styles and final runtime normalization provide distinct layers of the fix. |
| DiffView has no showcase coverage | Closed. The existing DiffView now has the Diff page, Review/Empty controls and Old/New review headings. The final monochrome review capture shows those distinctions. No new diff component is needed. |
| Baseline omits minimum size and monochrome | Partly closed. `src/bin/showcase/app_tests.rs:642` now covers five sizes and four palette levels. It still excludes the sidebar and proves a frame digest, not that hidden sections or every state can be reached. D4 below addresses the remaining verification weakness. |

Focused commands rerun successfully during this pass:

```text
rtk cargo test --bin showcase click_regressions   2 passed
rtk cargo test --bin showcase form_keeps          1 passed
rtk cargo test --lib keyhint                      3 passed
```

The previous full gates are recorded in `tui-audit-verification.md`; they were
not rerun as a substitute for the targeted evidence in this planning pass.

## Verified plan candidates

### D1 — Make every showcase section reachable at supported sizes

Classification: **confirmed defect** for the missing interactive error field;
**coverage gap** for inaccessible static state references. Priority: **P2**.

Evidence:

- `src/bin/showcase/pages/textareas.rs:64` reserves 13 rows, then a gap,
  leaving the disabled/error card no usable body at 72×20. The error field is
  rendered only in the lower card at lines 74–79. The fresh three-Tab replay
  confirms it is not reachable, rather than merely clipped in a screenshot.
- `src/bin/showcase/pages/inputs.rs:123` reserves 17 rows for the playground;
  its eight-state reference loop stops at the available bottom. At 100×30,
  the final capture contains only default, placeholder and hover. Focus,
  editing, error, error+focus and disabled reference rows cannot be revealed.
- `src/bin/showcase/pages/buttons.rs:72` reserves the playground first. Its
  reference matrix is truncated vertically at small sizes and horizontally
  by the fixed 15-cell column loop at lines 150–160. There is no input route
  that changes the state reference viewport; the handler routes only buttons.
- `src/bin/showcase/pages/mod.rs:139` is a geometry helper, not an overflow
  owner. Returning a shorter rectangle does not make omitted content reachable.

Impact: the library's advertised minimum size can demonstrate only a subset
of its states, and an enabled editor disappears from keyboard navigation.
The showcase cannot reliably serve as the reusable design contract at those
sizes.

Recommended plan: give multi-section showcase pages an explicit compact
section-switching composition using existing `Tabs`/`Select` controls. Retain
side-by-side or stacked sections where they fit. For the button matrix, allow
the existing button kind/state selection to reveal complete examples rather
than requiring all four columns simultaneously. Preserve user-entered drafts
and the selected section across resize; restore focus when switching back.

Reuse: `Tabs`, `Select`, `ScrollState`, scrollbar and focus registration are
already available. `ScrollPanel` and `TextViewport` render read-only text;
they are not existing arbitrary-child widget containers. Do not describe
wrapping a form in either as a solved composition. A shared application-level
section helper is justified only after two pages use the same interaction.

Risk: **medium**, because hidden controls, drafts, focus restoration and
resize interact. No public widget API break is required.

Acceptance:

1. At 72×20, 80×24, 100×30, 120×40 and 160×50, keyboard and mouse can reveal
   every documented section and every enabled control without resizing.
2. Each state reference is reachable in all four palette levels and actual
   NO_COLOR; static examples are clearly labelled as references.
3. No enabled field disappears from the workflow. A hidden section retains
   its draft and does not leave stale hit regions.
4. Resizing while a lower field is editing retains text and exposes a valid
   focus destination. Wheel ownership does not pull focus into another pane.
5. Normal and wide compositions retain existing spacing and restrained planes.

### D2 — Prevent zero-height sections from painting outside their allocation

Classification: **confirmed defect**. Priority: **P2**.

Evidence: `src/bin/showcase/pages/buttons.rs:151` draws matrix headers without
checking that the matrix's inner area has positive height. In the final
72×20 Buttons capture, `Primary` and `Secondary` appear in the blank row below
the page, even though the State matrix card has no allocated height. The row
loop guards vertical bounds, but the separate header loop does not.

Impact: detached labels violate the spacing contract and make the screen
appear incomplete. This also demonstrates why frame snapshots cannot alone
prove containment.

Recommended plan: enforce the allocation boundary for each composed section,
including headers and metadata, and use D1's section navigation to expose
omitted content. An empty section must emit no text, hit region or cursor.
Reuse bounded drawing and the existing layout helpers; no component addition.

Risk: **low** for the boundary fix, **medium** if folded into D1's navigation.

Acceptance: render each multi-section page into nonzero-origin rectangles
surrounded by sentinel cells. Zero-height/width sections and supported-size
compositions must leave every sentinel untouched. Reproduce the current
72×20 Buttons flow and verify the blank row remains blank.

### D3 — Add keyboard creation of text selections to TextViewport

Classification: **coverage gap**. Priority: **P2**.

Evidence: `src/widgets/viewport.rs:563` handles scrolling, follow, copy and
selection clearing, but has no keyboard path that creates a range. Range
creation occurs in `select_word_at` and `on_drag` at lines 497 and 535.
`Shift+Down` satisfies the current `Key::plain()` predicate and scrolls;
it does not select. `y` can copy only a selection already created by a mouse.
`src/widgets/diff.rs:286` delegates directly to this handler, so the gap
affects both plain terminal/log text and diffs.

Impact: keyboard-only users cannot perform the showcased selection/copy
workflow. Mouse parity is incomplete despite a working keyboard copy key.

Recommended plan: add a navigation caret and explicit keyboard range
extension to the existing viewport. Separate the selectable navigation caret
from the optional producer-owned terminal caret. Define entry/exit keys,
Shift extension, document/line/word boundaries, copy and clear before coding.
Specify how selection pauses follow, how wrapping maps to logical positions,
and what bounded retention does when an endpoint is removed.

Reuse: `CellPos`, the existing selection storage, visual-row mapping,
grapheme geometry, `ViewportEvent::Copy` and `ScrollState`. DiffView should
inherit the behavior. Do not introduce another text viewer or duplicate diff
selection state.

Risk: **medium/high** interaction risk, especially wrapped rows, follow mode
and producer updates. Prefer additive API and preserve unmodified scrolling.

Acceptance: keyboard and mouse select and copy identical Unicode ranges in
plain, wrapped and diff views; reverse video survives actual NO_COLOR;
selection remains meaningful across resize and bounded retention; follow
state and the hardware cursor are explicit; footer hints expose the available
entry/extend/copy/exit actions. Cover no-selection copy as a deliberate state.

### D4 — Verify state reachability separately from image baselines

Classification: **architecture/API weakness in verification**, not a new
runtime component defect. Priority: **P2**.

Evidence: `src/bin/showcase/app_tests.rs:642` hashes one focused frame per page,
size and palette. Lines 657–666 exclude navigation. Those baselines currently
accept D1's hidden state references and D2's escaped matrix header. A hash can
identify later change but does not establish that the captured state is valid.

Recommended plan: extend the inventory with an executable reachability list:
page, state, input sequence, expected focus owner, expected visible control,
and meaningful state marker. Reuse the existing App/TestBackend harness and
isolated capture tooling. Start with the documented states and D1/D2 cases;
do not infer exhaustive coverage from a larger screenshot count.

Risk: **low/medium** test-maintenance risk. Assertions should express contract
invariants, not mirror layout implementation or bless every current pixel.

Acceptance: each documented state has a reproducible keyboard route where
appropriate, mouse parity evidence, min/normal/wide checks and monochrome
meaning assertions. Disabled controls are excluded; read-only controls retain
navigation. Baselines are updated only after these assertions and visual
review pass. Keep static visual examples separate from live interaction proof.

### D5 — Separate disabled demonstrations from read-only content

Classification: **design inconsistency**. Priority: **P2** for the misleading
contract example; broader read-only field APIs remain an unproven opportunity.

Evidence: `src/bin/showcase/pages/textareas.rs:37–39` labels a textarea
`Read-only transcript` and immediately applies `.disabled(true)`.
`src/widgets/textarea.rs:100` refuses keyboard events when disabled, and render
excludes disabled fields from the focus ring. The final 120×40 capture displays
this content at disabled intensity. `DESIGN.md:700` instead says read-only
content remains fully readable, with mutation absent and a reason shown.
`src/widgets/code.rs:632` already preserves navigation/find for read-only code.

Recommended plan: label the current fixture as disabled if that is the state
being demonstrated, and demonstrate genuine read-only transcript behavior with
the existing `TextViewport` composition. Show an explicit read-only reason.
Use the existing read-only CodeEditor for code samples. D3 supplies keyboard
selection parity for the viewport demonstration.

Do not add `read_only` to every text widget merely for naming consistency.
If a real consumer needs a field-shaped selectable noneditable value, first
record why existing viewport/code composition is inadequate; then design an
additive read-only field contract with selection and mutation tests.

Risk: **low** for correcting the showcase semantics; **medium** for any later
field API addition. Acceptance: disabled examples remain unfocusable and dim;
read-only examples retain readable content, keyboard navigation and copy, show
their reason, and refuse mutation through every input path.

## Visual conclusions and limits

The inspected final product screens retain consistent dark planes, quiet
metadata, sparse framing, contextual footer hints and meaningful monochrome
markers. Holla's plan keeps selected steps, dependency columns and the
confirmation action legible at 80×24. TablePro's minimum-size explorer drawer
retains a visible focus row and scope identity. Jackin's account warnings use
both safety glyphs and wording. No palette replacement or additional decorative
component is justified by this evidence.

The static reference matrices intentionally show multiple focus/pressed states
at once. Their additional green is not evidence that normal application
screens violate the accent budget. Their problem is reachability at constrained
sizes, addressed by D1.

This pass verifies the named prior fixes and candidates above. It does not
claim every existing screen or overlay has been exercised. D4 defines the
additional evidence needed before claiming complete state coverage.

## Independent cross-verification of API-agent findings

The API agent supplied source locations and temporary probes. This design
verification independently read the production handlers and reproduced the
two application failures through real terminal input, rather than rerunning
the agent's App harness. A separately written public-API probe verified the
tree focus issue. No source changes were made.

### CV1 — Range selection after removing environment variables panics

Disposition: **confirmed defect, P1**. This replaces the earlier general
mutable-list risk with a concrete application crash.

Reproduction at 120×40:

1. Start `target/debug/showcase --page settings --color truecolor`.
2. Send `Tab`, `3`, `Tab`, `Down` three times, `Shift+Down`.
3. Send `Tab`, `Tab`, `Enter` to activate Remove selected. LOG_LEVEL and
   FEATURE_FLAGS disappear; three variables remain. Focus returns to navigation.
4. Send `Tab`, `Tab`, `Shift+Up`.

Observed terminal stderr:

```text
thread 'main' panicked at src/widgets/list.rs:96:31:
index out of bounds: the len is 3 but the index is 3
```

Evidence: `/tmp/plan-design.7CXPiQ/settings_range_before_remove.png`,
`settings_range_after_remove.png`, and `stderr.log`; the independent replay
script is `/tmp/plan-design.7CXPiQ/crossverify.sh` (`list` case).

Cause: `src/bin/showcase/pages/settings.rs:579` replaces public `items`,
resets `checked`, and clamps `cursor`, but cannot repair the private range
anchor. `src/widgets/list.rs:93–97` reuses that stale anchor as an unchecked
index into the shortened collection.

Plan: provide a transactional ListBox collection replacement/removal API
that reconciles cursor, chosen item, checked rows, range anchor and scrolling;
use it in Settings. Public mutation safety remains a broader contract decision,
but this supported application flow must not depend on inaccessible private
state. Defensively bound range operations as well.

Risk: medium, because selection identity must be explicit. Acceptance: replay
the exact flow without panic, ensure removed rows cannot remain selected, retain
the intended surviving cursor, and test empty/shorter/reordered replacement plus
keyboard and mouse range selection. Do not accept a Settings-only cursor clamp.

### CV2 — Delete on an empty tab picker closes an unrelated tab

Disposition: **confirmed defect, P1**. The unrelated closure is proved; loss
of an unsaved draft is a further risk requiring a dedicated regression.

Independent real-terminal reproduction at 120×40:

1. Start `target/debug/tablepro --connect Production --color truecolor`.
2. Send `Ctrl+G`, type `nosuchtab`. The picker says No matches; Query 1 remains
   visible behind the modal.
3. Send Delete (tmux key `DC`), then Escape.
4. Query 1 is gone; the workbench says No open tabs.

Viewed evidence:

- `/tmp/plan-design.7CXPiQ/picker_empty_before_delete.png`.
- `/tmp/plan-design.7CXPiQ/picker_empty_after_delete.png`.
- `/tmp/plan-design.7CXPiQ/picker_underlying_after_delete.png`.

The replay is the `picker` case in the same temporary script. This operated
only the deterministic TablePro simulation; no database statement was run.

Cause: `src/widgets/picker.rs:195` emits `Secondary(cursor)` unconditionally,
unlike Enter's readiness/item/disabled checks. Empty `set_items` retains cursor
zero. `src/bin/tablepro/app.rs:1502–1511` resolves an absent item with
`unwrap_or(i)` and closes that index, converting the invalid result into tab 0.
The picker also continues advertising Delete Close tab when no row exists.

Plan: enforce valid, ready, enabled targets for secondary events in Picker;
in TablePro, resolve a concrete tab identity and fail closed if it is absent.
Reuse the existing guarded tab-close workflow for valid targets and ensure
dirty/running tabs retain their existing protections. Footer actions should
reflect whether a target exists.

Risk: medium, because valid close workflows and dirty-tab confirmation must
remain intact. Acceptance: empty, disabled, loading and stale-result pickers
cannot close anything; filtering to no matches preserves every tab and draft;
valid filtered deletion closes only the represented tab; protected close paths
still require their established confirmation. Test both primitive events and
the full TablePro flow.

### CV3 — Lazy tree children move focus to another surviving node

Disposition: **confirmed defect, P2**. This is a stable-focus failure, not
just a speculative identity API preference.

Independent executable: `/tmp/plan-design.7CXPiQ/tree_crossverify.rs` and
matching binary, compiled against the current library. Setup: roots are a lazy
folder, keep B and keep C; keyboard cursor is on keep C. Supplying two children
for the earlier folder produces:

```text
before=[2] keep C
after=[0, 1] loaded child B
previous_node_survives=keep C
```

Cause: `src/widgets/tree.rs:227` calls `flatten()` after inserting children.
At lines 203–205, flatten replaces visible rows and clamps the numeric cursor,
without restoring its original path. New rows above the cursor change the
identity of the destination even though the previously focused node survives.

Plan: preserve the current path across this supported structural update and
resolve it back into visible rows. If the node disappears, define a nearest
surviving ancestor/sibling fallback. Reuse the tree's existing paths and
scroll model; globally introducing opaque node IDs is not required to fix this
specific stable-path case.

Risk: medium for removal/filter interactions, low for insertion above an
unchanged path. Acceptance: asynchronous insertion above/below the cursor
preserves the same node; selection remains separate from focus; removal uses
the documented fallback; scrolling changes only enough to retain the focus
destination. Include actual lazy-load consumer tests as well as TreeView tests.

## Independent cross-verification of INT03, INT04 and INT05

The interaction report's three remaining claims were independently read and
reproduced with a separate temporary public-API program:
`/tmp/plan-design.7CXPiQ/interaction_crossverify.rs`. The current library was
rebuilt with `rtk cargo build --lib`; the executable links the resulting
`target/debug/libjunie_tui.rlib`. No product source or tests were changed.

| Claim | Independent result | Disposition |
| --- | --- | --- |
| INT03: scalar Picker Backspace splits a grapheme | Typing `👩‍💻` then Backspace leaves `"👩\u{200d}"`; typing `e\u{301}` then Backspace leaves `"e"`. `src/widgets/picker.rs:200` calls `String::pop`. | Confirmed P2 defect. Use shared grapheme editing semantics; preserve the query compatibility contract. |
| INT03: searchable Picker paste is absent | The complete Picker implementation has no paste entry point. Holla `app.rs:415` and Jackin `app.rs:489` consume unsupported picker-modal paste; TablePro `app.rs:236` has no Picker branch and falls through to the active screen. | Source-confirmed coverage gap. This pass independently confirms the missing API and owner routes; INT01's document mutation has separate executable proof in the interaction/text reports. |
| INT04: Ctrl+Shift+Home/End drops Shift selection | A TextArea containing `first\n日second` enters editing at the end. Ctrl+Shift+Home yields `cursor=0, selection=None`; Ctrl+Shift+End yields `cursor=15, selection=None`. | Confirmed P2 defect. `src/widgets/field_common.rs:78–79` hardcodes `false` for document-boundary extension. Repair the shared binding, not each editor separately. |
| INT05: Ctrl+S sorts a reusable DataTable | A fresh table with values `z`, `a` starts `sort=None`. Ctrl+S returns `(Changed, None)` and sets `sort=Some((0, Asc))`. | Confirmed P2 API/binding inconsistency. `src/widgets/table.rs:438` matches `Char('s')` without modifier ownership. This does not establish that TablePro Save fails; its application-level interception is distinct. |

The independently observed outputs match the interaction report. Preserve its
bounded acceptance criteria: keyboard grapheme deletion and query paste parity;
Shift-aware document selection across text owners; and a modifier-conformance
matrix that retains intentional application and Picker chords. None of these
findings justifies a new widget or a wholesale event-framework replacement.

## Independent cross-verification dispositions

A second agent independently inspected the current source on 2026-09-10.
This cross-check did not edit application code or regenerate any baseline.

| Finding | Disposition | Independent evidence and limits |
| --- | --- | --- |
| D1 | Confirmed from allocation and routing | `TextAreasPage::render` reserves 13 rows before the gap/lower section; the lower editable Commit message is rendered only there. `TextArea::render` returns before control registration when its allocation has no body rows. Inputs clips its reference loop at `inner.bottom`; Buttons has fixed matrix columns and its event handler routes only to actual playground buttons. None of these pages owns section navigation or a reference scroll offset. The first agent's terminal replay supplies the actual 72×20 key flow; this reviewer independently verified its enabling code path. |
| D2 | Confirmed from drawing contract | Buttons' header loop checks horizontal width but not `inner.height` or `inner.bottom`. `Panel::render` returns an empty allocation unchanged, while the header then calls `buf.set_string` at that empty allocation's y-coordinate in the larger screen buffer. The separate row loop's vertical guard does not protect the headers. This reviewer did not claim a second independent PNG capture. |
| D3 | Confirmed capability absence | The complete `TextViewport::on_key` match has no operation that creates or extends its selection, and DiffView delegates to it. Existing `CellPos`, click/drag selection and copy payloads justify extending this primitive. The independent ecosystem report R1 also maps this gap to present Holla and Jackin consumers. |
| D4 | Confirmed verification limitation | `showcase_visual_baseline` constructs each page, draws, sends one Tab, and hashes that frame across five sizes/four palettes. It explicitly skips cells in `sidebar_area`. It performs no traversal of reference sections, enabled controls, editing/error transitions, or hidden states. Passing hashes therefore cannot prove reachability; this is a coverage limitation, not a claim that snapshots provide no regression value. |
| D5 | Confirmed design inconsistency | TextAreas constructs `TextArea("Read-only transcript").disabled(true)`. Disabled TextArea rejects keys and calls `ctx.control(..., true)`, excluding focus; the state grammar describes readable, nonmutating read-only content separately. CodeEditor already has a distinct read-only navigation path. Correcting the fixture or composing TextViewport is justified; a new field API is not established by this example alone. |

The reuse boundary was checked independently: `ScrollPanel` is explicitly a
read-only text panel with `Vec<String>` and no focusable children; `TextViewport`
stores styled lines. Neither accepts arbitrary child widgets or coordinates
their nested focus/event routing. `pages::layout::rows` only divides rectangles.
Tabs and Select already expose selection events and keyboard/mouse behavior, so
an application-level section composition is feasible without claiming an
existing generic scrolling form container or adding one speculatively.

Independent focused check: `rtk cargo test --bin showcase showcase_visual_baseline`
passed (one test, 33 filtered out, 2.22 s). The passing current baseline alongside
the source-confirmed D1/D2 paths supports D4's distinction between stable images
and valid/reachable interaction states. `git diff --check` also passed.
