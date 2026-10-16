# Showcase coverage: library capabilities

Status: inventory, 2026-10-16, a read-only pass over `design/interaction-polish` at `8f12ad218`. It lists every library item, variant, state and knob bundle, and whether the showcase demonstrates it (✅ fully, ⚠️ partly, ❌ not at all). Every ⚠️ and ❌ row is mapped to a step of the showcase coverage track (SC) in [`shared-components-plan.md` §7](../../architecture/shared-components-plan.md#7-showcase-coverage-track-sc). Line numbers drift as fixes land. Paths under `/private/tmp` refer to the auditor's scratch files, which are not kept.

## 1. Coverage summary

The showcase is far from complete. It does show every major widget family, but most configuration knobs are invisible: about 40 of the matrix rows are knob bundles (one row covers several knobs), and nearly all of those are ❌. Several whole widgets are never shown at all.

I counted one row per item, variant group or knob bundle (243 rows; the 3 Strategy-B rows are core rows added after the original count):

| module | rows | ✅ fully | ⚠️ partly | ❌ not shown |
|---|---|---|---|---|
| core | 47 | 9 | 13 | 25 |
| ui | 40 | 10 | 20 | 10 |
| widgets | 123 | 54 | 22 | 47 |
| theme | 15 | 3 | 2 | 10 |
| runtime | 6 | 2 | 3 | 1 |
| shell and cross-cutting states | 13 | 4 | 9 | 0 |
| **TOTAL** | **244** | **82 (34%)** | **69 (28%)** | **93 (38%)** |

Main findings:
- **Whole widgets never shown (10):** `Breadcrumb`, `SidebarRail`, `ButtonRow`, `ChoiceDialog`, `FormDialog`, `InfoDialog`, `HelpOverlay`, `KeyReference`, `PropsList`, and `ModalStack` (stacked modals).
- **Hand-drawn instead of the library widget:** the header breadcrumb and the `?` help (a `Dialog::confirm`, not `HelpOverlay`). The Sidebars page uses its own `NavList` (`sidebars.rs:28`), not `SidebarRail`. The Basic table header menu is built by hand (`tables.rs:313`), not `ui::columns::header_menu`.
- **Builders/policies never called:** none of these is called anywhere in the showcase: `nav()`, `keymap()` overrides, `track_click()`, `cursor_style()`, `placement()`, `blur_action()`, `focus_select()`, `field_keys()`, `pointer_select()`, `reveal()`, `Theme::with_*`, `LevelOverrides`, `Coalesce`. All `PointerConfig` fields except `focus_on(Release)` stay at their defaults.
- **No live knob switching anywhere.** The Inspector (`i`, `app.rs:1595`) only reads state: mode, focus, hover, pressed, mouse, last key, focus ring, hit regions, tick, colours. It is hidden below 100 columns (`app.rs:1386`).
  - `--color` sets the colour level at launch only; nothing switches it in-app.
  - `--motion reduced` behaves exactly like `full` (`main.rs:124`). That looks like a bug.
  - `--nav` and `--page` work as described.

## 2. Capability matrix

### core

| item | kind | shown | where (page / reach) | missing variants/options |
|---|---|---|---|---|
| `edit::EditEntry` Immediate/TypeToEdit/Explicit | knob | ⚠️ | inputs.rs:49 TypeToEdit ("Search files", Inputs page); other fields use Immediate | Explicit; `Theme::with_edit_entry`; TextArea/CodeEditor/Dialog `edit_entry` |
| `edit::FocusSelect` SelectAll/CaretEnd/CaretStart | knob | ❌ | default only | CaretEnd, CaretStart |
| `edit::FieldKeys` Emacs/Desktop, `for_platform` | knob | ❌ | platform default | flip it; `field_keys()`; `Theme::with_field_keys` |
| `edit::TextCmd`, `CMD_TEXT_KEYS`, `text_keymap` | knob | ❌ | default bindings | custom keymap; list of bound commands |
| `edit::UndoStack`/`EditKind` | helper | ⚠️ | Ctrl+Z works in fields (implicit) | undo coalescing explainer |
| `edit::clipboard` | helper | ⚠️ | copy in Terminal, Diff, Data grid (grid.rs:249) | clipboard fallback/status |
| `event::Outcome/Input/Key/Mouse/MouseKind` | helper | ✅ | shell; Inspector "last key"/"mouse" | event-log view |
| `event::InputConfig` (shift_wheel_horizontal, ctrl_click_secondary, legacy_ctrl_punctuation) | knob | ❌ | — | all 3 |
| `event::ClientPlatform` MacOs/Other | knob | ⚠️ | selectable.rs:293 (toggle-mods hint) | live platform switch |
| `focus::Focus/FocusRing/FocusChange/forward_blur` | helper | ✅ | shell; Inspector "focus ring" | — |
| `focus::FocusFallback` Nearest/First | knob | ❌ | — | both |
| `focus::BlurAction` Commit/Cancel/Keep | knob | ❌ | defaults | `blur_action` on Input/TextArea/Table/Grid, `EditorConfig.blur` |
| `focus::request_focus_next/cancel_focus_request` | helper | ❌ | — | — |
| `hit::*` (HitRegistry, HitRegion, Bar, Axis, ReleaseFrom) | helper | ✅ | shell; Inspector "hit regions" | hit-region overlay |
| `id::WidgetId/ItemKey/Part/PartRef` | helper | ✅ | everywhere | — |
| `intent::Intent/Phase/FocusVia/ActionKey/DismissReason/LayerEvent` | helper | ❌ | — | intent log |
| `keyboard::KeyboardProtocol` Auto/On/Off, kitty flags, `super_delivered` | knob | ❌ | env var only | show the active protocol |
| `keymap::Nav`/`NavKeys` (arrows/vim/page/home_end) | knob | ❌ | `.nav()` never called | vim, page/home_end off |
| `keymap::KeyBinding/KeyMap/Actions/lookup/label_for/hint_key/select_all_bindings` | knob | ⚠️ | app.rs:858 `lookup` | any `.keymap()` override |
| `nav::step_index` | helper | ✅ | implicit | — |
| `PointerConfig.focus_on` Press/Release | knob | ⚠️ | app.rs:496 fixed to Release | Press |
| `PointerConfig.double_click_ms/slop` | knob | ❌ | — | — |
| `PointerConfig.wheel_lines` | knob | ❌ | — | — |
| `PointerConfig.hover_refresh_after_scroll` | knob | ❌ | — | — |
| `PointerConfig.flash_ms` | knob | ❌ | — | — |
| `PointerConfig.secondary_outside` Reopen/Close/Ignore | knob | ❌ | — | all |
| `PointerConfig.end_edit_on_dead_space` | knob | ❌ | — | — |
| `PointerConfig.autoscroll/_ms` | knob | ⚠️ | app.rs:616, on | off, speed |
| `PointerConfig.track_repeat/_delay_ms/_ms` | knob | ⚠️ | app.rs:620, on | off, timing |
| `PointerConfig.pointer_shapes/caret_shape` | knob | ⚠️ | main.rs:157 `shapes_for` | off; caret variants |
| `PointerConfig.click_skips_animation` | knob | ❌ | — | — |
| `pointer::Release/SecondaryRoute/PointerState` | helper | ✅ | app.rs:999, :1185 | — |
| `response::Response/Flow/Invalidate/StateFlags` | helper | ❌ | — | — |
| `scroll::ScrollState` | helper | ✅ | all scrollers | — |
| `scroll::FollowCursor` OnMove/Always | knob | ❌ | — | Always |
| `scroll::ScrollbarVisibility` Auto/Always/Never | knob | ❌ | Auto | Always, Never |
| `scroll::TrackClick` Jump/Page | knob | ❌ | Page (app-wide default) | Jump, app-wide and per widget |
| `scroll::HScroll` (enabled/bar/step/track_click, `off`) | knob | ⚠️ | implicit on Text areas and Basic table overflow | off, no bar, step, `horizontal()` |
| `selection::SelectionMode` None/Single/Multi | knob | ⚠️ | Single tables.rs:170; Multi selectable.rs:304 | None |
| `selection::TriState`/`RowSelection` | state | ✅ | selectable.rs:365 (mixed header), :557 | — |
| `shape::PointerShape` (5 shapes) | state | ⚠️ | implicit; needs a terminal that supports it | indicator |
| `shape::CaretShape` (4), `ShapeSupport`, `ShapeSync` | knob | ❌ | — | all |
| `text::TextBuffer/CursorPos` | helper | ✅ | every field | — |
| `typeahead::TypeAhead` (timeout) | knob | ⚠️ | sidebar app.rs:374 | timeout |
| Strategy B — `nav.rs` modules (`FocusPolicy`, `AdjacentMove`, `AdjacentKeys`, `Scope`, `Bounds`) | knob | ❌ | — | no page applies a nav policy |
| Strategy B — nav policy fields on Tabs/Tree/List/Select/Picker/Menu/Dialog/Toolbar | knob | ❌ | — | none set |
| Strategy B — Toolbar widget | widget | ❌ | — | not in showcase |

### ui

| item | kind | shown | where | missing |
|---|---|---|---|---|
| `columns::Col` (content/fixed/flex/max/droppable) + `solve` | helper | ❌ | — | droppable columns |
| `columns::ColumnSizing` (resizable/seam_w/min_width) | knob | ⚠️ | Data grid seams grid.rs:389 | Basic table seams, resizable=false |
| `columns::header_menu`/`HeaderAction` (5)/`HeaderLabels` | widget | ❌ | tables.rs:313 builds its own | library header menu, Autofit, ResetWidths |
| `columns::render_seam` | helper | ⚠️ | via grid | — |
| `context::ContextConfig` (keys, moves_cursor) | knob | ⚠️ | row_menu.rs:27, DEFAULT | moves_cursor=false |
| `context::ContextKeys` (shift_f10/menu_key/extra) | knob | ❌ | — | extra key |
| `context::ContextAnchor` Pointer/Control | state | ✅ | editor.rs:242/250 | — |
| `context::ContextRequest/ContextSource` | helper | ✅ | lists.rs:111, trees.rs | — |
| `ctx::Interaction` | helper | ✅ | app.rs:630 | — |
| `ctx::FocusVisible` Always/KeyboardOnly | knob | ⚠️ | implicit (app.rs:1087) | toggle; `Button::focus_visible` |
| `ctx::VisualState` (8 flags) | state | ⚠️ | buttons.rs:102 matrix; inputs.rs:145 reference | selected and busy rows; matrices for other widgets |
| `RenderCtx.reveal/tooltips/hscroll/track_click` (app-wide) | knob | ❌ | defaults (app.rs:1343) | app-wide switcher |
| `RenderCtx.inert`/`page_under_layer` | state | ✅ | app.rs:1344 | — |
| `ctx::Backdrop` (keep_bottom/keep_top) | knob | ❌ | — | — |
| `fade::scroll_edges(_except)` | state | ✅ | sidebars.rs:269, Scrolling page | — |
| `layout::Split` (percent/min_*/maximized) | widget | ⚠️ | terminal.rs:104 | Maximized First/Second |
| `layout::SplitDir` | knob | ⚠️ | Horizontal only (terminal.rs:105) | Vertical |
| `layout::Stack/StackChild/StackScroll/StackLayout` | widget | ✅ | pages/mod.rs:215, panels.rs:87, taskrunner.rs:230 | — |
| `layout::Overflow` Scroll/Clip, `min_child_rows`/`scrollbar_gap`/`track_click` | knob | ❌ | Scroll | Clip and the rest |
| `leave::Stakes` Clean/Unsaved/Running | state | ⚠️ | grid.rs:308, selectable.rs:1129 | Running |
| `leave::LeaveGuard/Resolved` | widget | ✅ | app.rs:506, :799 (quit with unsaved edits) | — |
| `leave::QuitKeys` (ctrl_c/ctrl_q/letter/esc_quits) | knob | ⚠️ | app.rs:505, DEFAULT | letter off, esc_quits |
| `leave::KeyContext/LeaveKey` Copy/Quit | state | ⚠️ | app.rs:896 | the Ctrl+C=Copy case is not surfaced |
| `leave::unsaved_dialog/discard_dialog/running_dialog` | widget | ⚠️ | discard only (selectable.rs:42) | unsaved, running |
| `modal::Modal/ModalStack/Dismissal/ensure_focus` | widget | ❌ | shell holds a single `Option<Dialog>` | stacked modals |
| `modal::Frame` (placement/anchor/fit/margin/min_size/backdrop/dim) | knob | ⚠️ | `chrome` (pages/mod.rs:231) | Upper/Middle, anchor, dim |
| `modal::refuse_outside_click/close_button/border_style` | state | ⚠️ | app.rs:1008 | close button × |
| `popup::Dismiss` (escape/outside_click/pass_through/close_button/refuse_feedback) | knob | ⚠️ | pass_through chips.rs:67 | close_button, escape off |
| `popup::RefuseFeedback` Flash/None | knob | ⚠️ | Flash (implicit) | None |
| `popup::Placement` (5) | knob | ⚠️ | Below + auto-flip | Above/Right/Left/Center |
| `popup::Overflow` Shrink/Slide, `VAlign`, `Place`/`place_with` | knob | ⚠️ | `place` celleditors.rs:307 | Slide, Third |
| `popup::backdrop/surface(_for)` | helper | ✅ | app.rs:1185, panels.rs:97 | — |
| `reveal::Reveal` (hover/keyboard/max_rows/scroll/wheel_lines/delay_ms/wheel_passes_at_ends) | knob | ⚠️ | default; footer advertises key (app.rs:1700) | every knob; per-widget `reveal()` |
| `reveal::RevealMode` Tooltip/InlineWrap | knob | ⚠️ | Tooltip | InlineWrap |
| `reveal::RevealState/RevealLine/truncated/queue_cut` | helper | ✅ | implicit | — |
| `text::width/truncate/fit/wrap/fuzzy` | helper | ✅ | chips.rs:136, pickers.rs:145 | — |
| `text::truncate_middle/thousands/fit_right/slice_cells/wider_than/width_capped/find_ranges` | helper | ❌ | — | — |
| `text::WrapOpts`/`wrap_with` | knob | ❌ | — | keep_indent, hang_at_gap |
| `tooltip::Tip` + `show` | widget | ⚠️ | buttons.rs:26-30 (hover a button) | `shortcut` in tip |
| `tooltip::Tooltips` (enabled/delay_ms/max_width) | knob | ❌ | — | all |

### widgets

| item | kind | shown | where | missing |
|---|---|---|---|---|
| `Lockup` (text, compact) | widget | ⚠️ | chrome.rs:65 (menu-bar brand) | compact; header lockup is hand-drawn |
| `Breadcrumb` (segments, clickable) | widget | ❌ | header crumb hand-drawn (app.rs ~1480) | whole widget |
| `Button` primary/secondary/subtle/danger/toggle | widget | ✅ | buttons.rs:26-32 | — |
| Button rest/hover/focus/focus+hover/pressed/disabled | state | ✅ | buttons.rs:102 static matrix; :77 live | — |
| Button busy | state | ✅ | buttons.rs:54 ("Start long job") | busy row in the matrix |
| Button tooltip/shortcut | knob | ⚠️ | tooltip only | shortcut |
| Button focusable/focus_visible/mnemonic | knob | ❌ | — | all |
| `ButtonRow` (align, gap, overflow More/Wrap), `row_layout_right`, `RowLayout.more/hidden` | widget | ❌ | pages use `row_layout` only | End align, More, Wrap |
| `ChipBar`/`Chip` | widget | ⚠️ | chips.rs:44-52 (lead, add, disabled chip) | Chip.error, removable=false |
| ChipBar overflow_marker/overflow_menu/close_target_w | knob | ⚠️ | default overflow | toggles |
| `ChipEvent` (6) | state | ✅ | chips.rs:84-117 | — |
| `ChipCmd`, ChipBar `nav`/`keymap` | knob | ❌ | — | — |
| `Checkbox` (checked/disabled/mixed/hit) | widget | ⚠️ | forms.rs:71; mixed selectable.rs:379 | disabled; CheckHit Label vs Row |
| `RadioGroup` | widget | ⚠️ | forms.rs:70, settings.rs:122 | disabled; `nav` |
| `Toggle` on/off/disabled | widget | ✅ | forms.rs:73-74 | — |
| `ChoiceDialog` (radio/stepper/option_tones/line/buttons) | widget | ❌ | — | whole widget |
| `CodeEditor` highlighter/segmenter/placeholder/diagnostics/running | widget | ✅ | editor.rs:195-198, :275, :283 | — |
| CodeEditor read_only/tab_leaves/esc_reverts/edit_entry/field_keys/text_keymap/nav/keymap/scrollbar_gap/track_click/horizontal/indent | knob | ❌ | — | all |
| CodeEditor find (`FindState`) | state | ⚠️ | default keys only | discoverable find demo |
| `Severity`/`Diagnostic` | state | ✅ | editor.rs:285-292 | — |
| `CodeCmd` (7) | knob | ⚠️ | default keys | override |
| `Completion`/`CompletionItem` | widget | ✅ | editor.rs:199, :223 | — |
| Completion max_rows/popup_width/fit/placement/cursor_style/nav | knob | ❌ | — | all |
| `Dialog::confirm` | widget | ✅ | dialogs.rs:51 (Dialogs page → "Confirm run") | — |
| `Dialog::prompt` | widget | ✅ | dialogs.rs:63, settings.rs:589 | — |
| `Dialog::destructive` | widget | ✅ | dialogs.rs:84, selectable.rs:524 | — |
| `Dialog::facts` | widget | ⚠️ | grid.rs:233 (Preview SQL; token=None) | typed-token ack (`AckInput`, `AckEnter` Submit/NextField) |
| Dialog custom actions (3 buttons), cancel_label | knob | ✅ | dialogs.rs:72, taskrunner.rs:345 | — |
| Dialog accelerators/mnemonics/guard_dirty_prompt/confirm_index/button_row/body_scroll/max_height/outer_margin/backdrop/placement/anchor/nav/keymap/edit_entry/width | knob | ❌ | — | all |
| `DialogBody`/`DialogResult` | state | ✅ | dialogs.rs:133 | — |
| Dialog body overflow | state | ❌ | — | long body that scrolls |
| `DiffView` Unified/Review, `DiffLine` | widget | ✅ | diff.rs:52, :63 ("Review" toggle) | — |
| `DiffStatus` | state | ⚠️ | Modified only (diff.rs:30) | Added/Deleted/Renamed |
| DiffView empty/no-file/narrow | state | ⚠️ | diff.rs:68 ("Empty" toggle) | no file; explicit narrow |
| `unified_lines`/`review_lines` (`_with`), `DiffLineKind` | helper | ⚠️ | implicit | — |
| `EmptyState` (title, hint) | widget | ✅ | chips.rs:393, editor.rs:397, selectable.rs:731 | — |
| EmptyState `error()` | state | ❌ | — | — |
| EmptyState `actions()`/`EmptyAction`/`render_ctx` | widget | ❌ | — | clickable actions |
| `PointerSelect` (drag/word/line/drag_by_word) | knob | ❌ | — | each flag |
| `field_common` edit plumbing | helper | ✅ | implicit | — |
| `FormDialog` (+ FormField kinds incl. chooser/note/hidden, FieldValue, FormEvent, keep_open_on_save) | widget | ❌ | — | whole widget |
| `DataGrid` CellKind ×7 / CellValue incl. Null/Default | widget | ✅ | grid.rs:117-140 | — |
| `ColumnSpec` primary/read_only/nullable/type_label/enum_values/editor | knob | ✅ | grid.rs:117-138 | references (FK), min/max_width |
| Grid pending changes, commit/discard/Preview SQL, undo | state | ✅ | grid.rs:233 | — |
| Grid paging, `RowTotal` | state | ⚠️ | grid.rs:142, :210 | Unknown total |
| Grid loading | state | ✅ | grid.rs:221 (refresh) | — |
| Grid `RowState`, cell/row errors | state | ⚠️ | edit flow | Error row |
| Grid read-only (`GridEvent::ReadOnly`) | state | ⚠️ | grid.rs:268 | whole-grid read-only, bar_labels |
| Grid editor_config/type_ahead_column/overflow_badges/bar_labels/duplicate_keys/width_sample_rows/resizable_columns/seam_hit_w/track_click/nav/keymap/row_numbers/header_labels/filtered_cols/blur_action | knob | ❌ | — | all |
| `GridCmd` (18) | knob | ⚠️ | default keys | key reference/override |
| `HelpOverlay` | widget | ❌ | `?` uses Dialog::confirm (app.rs:986) | whole widget |
| `HintBar`/`HintLayer` (resolve, badge, status) | widget | ✅ | chrome.rs:137-168 | — |
| `HintLayer.align`/`HintAlign` | knob | ❌ | — | Center |
| `InfoDialog` | widget | ❌ | — | whole widget |
| `TextInput` value/placeholder/help/required/disabled/validator | widget | ✅ | inputs.rs:29-55 | — |
| TextInput masked/reveal_tail | knob | ✅ | inputs.rs:51-53 ("API key") | — |
| TextInput plain_label | knob | ✅ | selectable.rs:323 | — |
| TextInput 8 states | state | ✅ | inputs.rs:145 (State reference) | — |
| TextInput required_message/wrap_label/blur_action/focus_select/field_keys/text_keymap/pointer_select/label_hit | knob | ❌ | — | all |
| `keyhint` render/hint/Hint/constants | widget | ⚠️ | app footer; selectable.rs:1159 | render_toned/aligned/styled; `HintPriority` |
| `KeyReference` | widget | ❌ | — | whole widget |
| `ListBox` Single/Multi, disabled, meta, empty_text | widget | ✅ | lists.rs:33/53/56 | — |
| ListBox ClickAction/skip_disabled/wrap/type_ahead/reveal/empty()/scrollbar_gap/track_click/nav/keymap, `ListCmd` | knob | ❌ | — | all |
| `ContextMenu`/`MenuItem` shortcut/disabled/reason/danger/separator, title, at | widget | ✅ | lists.rs:79-104, tables.rs:313, chrome.rs:126 (right-click / Shift+F10) | — |
| MenuItem danger_if/separator_if | helper | ❌ | — | — |
| ContextMenu placement/anchor(_control)/fit/min_width/max_rows/edge_margin/release_selects/hide_owner_focus/nav/wrap/type_ahead | knob | ❌ | — | all |
| `MenuBar` (labels, menus, brand) | widget | ✅ | chrome.rs:41-65 | — |
| MenuBar OpenOn, Overflow Hide/Chevron, hover_switch, press_drag, hide_owner_focus, nav | knob | ❌ | — | all; narrow overflow |
| `MenuBarEvent::Closed` | state | ❌ | — | — |
| `ColumnEditor` Text/Select/Combo/Radio/Checkbox/Segmented/Tags | widget | ✅ | celleditors.rs:147-154 (`for_options` ladder); editable.rs:34 Text | — |
| `OptionItem`/`Options` | knob | ✅ | celleditors.rs:35-129 | — |
| out-of-set values flagged | state | ✅ | celleditors.rs:339 | — |
| `EditorConfig` (10 fields) | knob | ❌ | — | all |
| `Affordance` Focused/Always/Never | knob | ❌ | Focused | Always, Never |
| `ChoiceEditor`/`ChoiceResult` standalone | widget | ⚠️ | only inside tables | standalone |
| `Panel` card/framed/untitled, meta/focused/pad_x/title_min | widget | ✅ | panels.rs:97/108/243, scrolling.rs:79 | — |
| Panel PanelFocus Title/Bar, focus_mark, meta_short, bg_override, mark_clipped | knob | ⚠️ | mark_clipped sidebars.rs:462 | Title vs Bar, meta_short |
| `ScrollPanel` wrap/tail/follow | widget | ✅ | panels.rs:39, scrolling.rs:37, taskrunner.rs:100 | — |
| ScrollPanel hscroll/horizontal/track_click/nav/keymap, `PanelCmd` | knob | ❌ | — | all |
| `Picker` search/scope/groups/tags/alt action/delete/non-searchable/width | widget | ✅ | pickers.rs:108-120, :162 (Pickers page buttons) | — |
| `PickerStatus` Ready/Loading/Error | state | ❌ | Ready only | Loading, Error |
| Picker max_rows/fit/placement/anchor/backdrop/cursor_style/delete_is_secondary/nav/empty() | knob | ❌ | — | all |
| PickerItem disabled/key | state | ❌ | all rows enabled (pickers.rs:155) | disabled row |
| `render_bar` + `ProgressStatus` ×4 | widget | ✅ | progress.rs:106-115 | — |
| `BarLayout`/`render_bar_with` | knob | ❌ | — | — |
| `render_indeterminate(_with)`, `render_spinner`, `spinner_frame` | widget | ✅ | progress.rs, taskrunner.rs | — |
| `Meter` + `MeterTone` ×9 + `MeterVisual` Line/Block | widget | ✅ | progress.rs:142-173 | `MeterTone::Level` |
| `MeterLevel` thresholds | state | ✅ | progress.rs:143-145 | — |
| `Prop` (tone, wrap) + `props::render` | widget | ✅ | chips.rs:370-384 | copyable |
| `props::Sheet`, `measure(_with)`, `render_with` | knob | ❌ | — | — |
| `PropsList` (+ PropsEvent/PropsCmd) | widget | ❌ | — | whole widget |
| `SidebarRail` (collapsed_rail, RailEvent) | widget | ❌ | Sidebars page uses its own NavList | whole widget |
| `scrollbar` (render_*, Bars, VSplit, position_label, press/drag) | widget | ✅ | Scrolling page | — |
| `segments::render`/`Segment` | widget | ✅ | chips.rs:326-352 (wide and narrow) | — |
| Segment clickable | knob | ❌ | — | — |
| `Select` help/disabled/dismiss/mixed | widget | ✅ | chips.rs:57-70; mixed via selectable.rs:365 | — |
| Select SelectArrows/WheelMode/max_rows/placement/popup_width/popup_fit/cursor_style/close_on_blur/hover_moves_cursor/press_drag/label_hit/track_click/wrap/type_ahead/nav | knob | ❌ | — | all |
| `Splitter` | widget | ✅ | terminal.rs:105 (drag the seam) | — |
| `SeamDoubleClick` Reset/Ignore, hit_pad | knob | ⚠️ | Reset | Ignore, hit_pad |
| `StatusBar`/`StatusItem` strong/chip/priority/clickable | widget | ✅ | chrome.rs:86-111 | — |
| StatusItem busy/meter/tooltip/Emphasis::Plain; centre group | knob | ❌ | — | all |
| `StepRail` + `StepState` ×6 | widget | ✅ | terminal.rs:97, :127-190 | — |
| StepRail selectable/numbered/track_click/nav | knob | ❌ | — | all |
| `DataTable` sort, numeric, empty_text, right-aligned | widget | ✅ | tables.rs:24-30, :170-179 | — |
| DataTable editable/cell_nav/validator | widget | ✅ | editable.rs:34-65 | — |
| DataTable multi-select, selection_column, toggle_mods, disabled_rows | widget | ✅ | selectable.rs:299-309 | — |
| `Align`, `SortDir` | knob | ✅ | tables.rs:29, :111 | — |
| DataTable esc_clears/overflow_marks/type_ahead_column/editor_config/resizable_columns/seam_hit_w/track_click/nav/keymap/empty()/header_labels/blur_action | knob | ❌ | — | all |
| `TableCmd`/`TableEvent` | state | ✅ | events handled on table pages | keymap override |
| `Tabs` | widget | ⚠️ | settings.rs:116 (3 plain tabs, digit jump) | — |
| `TabItem` dirty/busy/error/prefix/suffix/closable | state | ❌ | — | all |
| `TabRule` Auto/Accent/Neutral/FocusOnly | knob | ⚠️ | Auto | Accent, Neutral, FocusOnly |
| Tabs allow_new/new_key/close_keys/middle_click_closes/overflow_menu/close_target_w/wrap/nav/keymap; TabEvent Close/New; TabCmd | knob | ❌ | — | all |
| `TextArea` value/placeholder/help/disabled/error | widget | ✅ | textareas.rs:31-42 | — |
| TextArea focus_select/esc_reverts/field_keys/text_keymap/nav/pointer_select/blur_action/label_hit/horizontal | knob | ❌ | — | all |
| `TreeView`/`TreeNode` dir/leaf/leaf_meta, menu, expand_all | widget | ✅ | trees.rs:37, taskrunner.rs:88 | — |
| Tree filter/type_ahead/selected_marker | knob | ✅ | sidebar (app.rs:374, :1277; type `/`) | — |
| TreeNode lazy/glyph/note/busy | state | ❌ | — | all |
| Tree BranchAction/row_click_toggles/indent/divider/alt_click_recursive/reserve_right/show_meta/track_click/nav/wrap, `TreeCmd` | knob | ❌ | — | all |
| `TreeEvent::Expand` | state | ❌ | — | lazy expand |
| `TextViewport` scrollback/follow/selection/copy/max_lines + `Span` | widget | ✅ | terminal.rs:92-182 | — |
| Span italic/underline/reversed | knob | ❌ | — | — |
| TextViewport caret/wrap/hscroll/scrollbar_gap/track_click/nav/keymap/with_lines, ViewportCmd, `SelectionChanged`, WorkCounters | knob | ❌ | — | all |

### theme

| item | kind | shown | where | missing |
|---|---|---|---|---|
| `Theme::junie/for_level` + tokens | widget | ✅ | overview.rs:81 (Tokens panel) | many tokens (field_hover, highlight*, selection_bg, success, …) |
| `ColorLevel` ×4, detect, label | knob | ⚠️ | `--color` (main.rs:40); level shown in header and Inspector | in-app switch; comparison |
| `LevelOverrides`, `for_level_with` | knob | ❌ | — | all |
| `Plane`/`Kind`/`Ladder(s)`/`with_ladder` | knob | ❌ | implicit | ladder explorer |
| `StateModifiers` | knob | ❌ | — | — |
| `CursorStyle` Fill/Bar, `Interactive`, `CursorRow` | knob | ❌ | Fill | Bar |
| `Tone`, `SyntaxTone` | state | ✅ | tables, editor | — |
| `ButtonKind` ×5 | state | ✅ | buttons.rs:146 | Toggle row in the matrix |
| `BadgeKind` | state | ⚠️ | chrome.rs:168 (Edit) | other kinds |
| `HintAlign`/`HintBarStyle` | knob | ❌ | — | all |
| `Glyphs` + ascii_glyphs/ascii/with_glyphs | knob | ❌ | Unicode only | ASCII set |
| `Strings` + with_strings/fill/follow | knob | ❌ | — | override demo |
| `Spacing`/`Insets` | knob | ❌ | — | — |
| Theme edit_entry/field_keys defaults | knob | ❌ | — | — |
| `contrast_ratio`/`legible` | helper | ❌ | — | contrast table |

### runtime

| item | kind | shown | where | missing |
|---|---|---|---|---|
| `Application`/`run` | helper | ✅ | main.rs:141-160 | — |
| tick_interval / motion | knob | ⚠️ | `--motion`, `--frame` | `reduced` is treated as Full (main.rs:124) |
| `Coalesce` (6 fields, OFF) | knob | ❌ | — | all |
| `Application::shapes` | knob | ✅ | main.rs:157 | — |
| `TerminalSession`, job_control | helper | ⚠️ | implicit | — |
| `SizeWatch`/`InputPump`/drain* | helper | ⚠️ | implicit | — |

### Cross-cutting visual states and shell

| item | kind | shown | where | missing |
|---|---|---|---|---|
| hover/pressed/focused | state | ✅ | buttons.rs:102, inputs.rs:145; live everywhere | matrices for other widgets |
| selected | state | ⚠️ | live in lists/tables/trees | static reference |
| disabled | state | ✅ | buttons, inputs, text areas, list items, toggle, select | radio, checkbox, menu-bar item, tab, picker row |
| busy | state | ⚠️ | Button, Task runner, editor running | StatusItem, TreeNode, TabItem |
| error | state | ⚠️ | inputs, text areas, editable, progress/meter | Chip, EmptyState, Picker, Tab |
| empty | state | ⚠️ | lists.rs:56, tables.rs:171, Settings env list, Diff | the `empty(EmptyState)` widgets |
| loading | state | ⚠️ | grid.rs:221 | Picker Loading, lazy tree |
| overflow | state | ⚠️ | Scrolling page, segments, chips | ButtonRow, Tabs, MenuBar, Dialog body |
| narrow / too small | state | ⚠️ | too-small screen (app.rs:1438), chips.rs:352 | per-widget narrow presets without resizing the terminal |
| colour levels | state | ⚠️ | `--color` only | in-app switch |
| Inspector `i` | knob | ⚠️ | app.rs:1595 (read-only) | knob flipping; hidden below 100 columns |
| `--nav current/accordion/all` | knob | ✅ | main.rs:60 | — |
| `--page NAME` + aliases | knob | ✅ | main.rs:101, PAGE_ALIASES | — |

## 3. Proposed showcase structure

**Rules kept:** the same 9 groups and Overview at the top. No group holds a page with the group's own name, and the tree stays at most 2 levels deep. Existing page ids and `--page` aliases keep working.

### A. The Inspector becomes "State | Options"

`i` opens a right pane with two tabs, built from the library itself (`Tabs` + `PropsList`/`ListBox` rows), so it also demonstrates those widgets:
- **State tab:** today's diagnostics, plus a `Response`/`Intent` event log and the active `KeyboardProtocol`/`PointerShape`.
- **Options tab:** the knobs of the page's focused widget. Each page exposes a list of options, and each option is one of:
  - `Toggle` (bool)
  - `Select` (enum)
  - `Segmented` (2–5 values)
  - numeric stepper

  Flipping an option rebuilds the widget with that builder call.
- **Shell-wide section, always present:** colour level (TrueColor/256/16/Mono), `LevelOverrides`, `Glyphs` Unicode/ASCII, `Strings` default/override, `FocusVisible`, `TrackClick` Page/Jump, `HScroll` on/off, `Reveal` mode, `Tooltips` delay, `PointerConfig` (focus_on, secondary_outside, wheel_lines, autoscroll, track_repeat, double-click), `NavKeys` vim, `FieldKeys` Emacs/Desktop, `ClientPlatform`, `EditEntry`/`FocusSelect`/`BlurAction`, `QuitKeys`, `Coalesce`, motion Full/Reduced/Paused.
- **Below 100 columns** the pane becomes an overlay (Frame placement Right) instead of disappearing.
- **Same knobs from the CLI:** `--opt key=value` sets them at launch for capture.

### B. A "States" pattern per widget

Each widget page gets a third section, after "Playground" and "Options". It is a static grid, rows = states, columns = variants, like buttons.rs:102 today. States: rest, hover, pressed, focused, focused+hover, selected, disabled, busy, error, empty, loading, overflow, narrow (rendered into a 24-column slot).

It is drawn with `VisualState`, so it stays exact. Separately, a "Compare levels" switch renders the grid at all 4 colour levels side by side.

### C. Pages to add or extend, by group

| Group | Pages (existing ➕ new) | Adds |
|---|---|---|
| — | Overview ➕ token tables | all `Theme` tokens; `Plane`×`Kind` ladder explorer; `contrast_ratio` table per level; Spacing/Insets |
| App shell | Chrome; ➕ **Breadcrumbs & rails**; ➕ **Help & keys**; ➕ **Quit & leave** | MenuBar OpenOn/Overflow/hover_switch/press_drag; StatusItem busy/meter/tooltip/centre group; Lockup compact; Breadcrumb clickable/static; SidebarRail expanded/collapsed; HelpOverlay; KeyReference (sections, key_max_share); HintLayer align; `keyhint::render_*`, HintPriority; QuitKeys, LeaveGuard with Clean/Unsaved/Running, unsaved/discard/running dialogs |
| Inputs & forms | Buttons, Inputs, Text areas, Chips & selects, Forms; ➕ **Choices** (checkbox, radio, toggle) | ButtonRow align/overflow More/Wrap; Button shortcut/focusable/mnemonic; TextInput required_message/wrap_label/label_hit/pointer_select/focus_select/blur_action/text_keymap; TextArea esc_reverts; Checkbox disabled/mixed/CheckHit; RadioGroup disabled; Chip error/removable, overflow menu; Select arrows/wheel/placement/cursor_style/close_on_blur/press_drag |
| Tables | Basic table, Editable, Data grid, Selection, Cell editors; ➕ **Columns** | `ui::columns` Col/solve with droppable columns, ColumnSizing, the library `header_menu` (Autofit/ResetWidths), seams on DataTable; SelectionMode None; EditorConfig + Affordance; Grid RowTotal Unknown, references/FK follow, read-only grid, row numbers, overflow badges, duplicate_keys; `empty(EmptyState)` on table and grid |
| Navigation | Lists, Trees, Sidebars; ➕ **Tabs**; ➕ **Menus** | Tabs: closable/new/dirty/busy/error/prefix/suffix, all 4 TabRules, overflow menu, middle-click. Trees: lazy/busy/glyph/note nodes, BranchAction, indent/divider. List: ClickAction, skip_disabled. ContextMenu placement/anchor_control/release_selects/max_rows. Sidebars switches to the library `SidebarRail` |
| Overlays | Dialogs, Pickers; ➕ **Dialog kinds**; ➕ **Popups & tooltips** | Dialog kinds: FormDialog (every field kind), InfoDialog, ChoiceDialog (radio/stepper/tones), facts with typed token + AckEnter, long scrolling body, mnemonics/accelerators, guard_dirty_prompt, stacked modals via ModalStack. Popups & tooltips: Placement ×5, Overflow, VAlign, Dismiss variants incl. close button, Backdrop, Frame placement, Tooltips knobs, Reveal Tooltip vs InlineWrap. Pickers: PickerStatus Loading/Error, disabled rows |
| Feedback | Progress; ➕ **Empty & error** | BarLayout, MeterTone::Level; EmptyState error and actions; status toasts |
| Layout | Panels, Scrolling; ➕ **Splits & stacks** | Split Vertical/Maximized, Splitter double-click Ignore/hit_pad, Stack Overflow Clip/min_child_rows; Panel PanelFocus Title/Bar, meta_short; ScrollbarVisibility, FollowCursor, TrackClick per widget |
| Text & code | Code editor, Diff, Terminal; ➕ **Text utilities** | CodeEditor read_only/find/tab_leaves; Diff Added/Deleted/Renamed/no-file; TextViewport caret/wrap/hscroll; Span italic/underline/reversed; truncate_middle, thousands, wrap_with/WrapOpts, find_ranges |
| Screens | Settings, Task runner | Task runner leave_stakes Running; Settings tabs with dirty markers |

**Check against the naming rule:** none of the new leaf names repeats its group's name. The tree stays 2 levels.

### D. Keeping it complete

Add a coverage test next to `app_tests_coverage.rs`. It lists each public builder and enum variant (a list like the one I generated) and fails unless at least one showcase option or state-grid row references it. That stops new knobs from shipping without a showcase entry.

The generated inventory files are in `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/inv-scripts/`: `grouped.txt` lists per type which builders are and are not called, and `matrix.md` is the table above. I made no edits, commits or pushes, and did not touch the `visual-baseline` tag.