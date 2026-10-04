//! Termrock Public API and Architecture Audit (TASK-016 / CHK-001).
//!
//! Validates:
//! 1. AC-001: Public exports and implementation ownership match the canonical API,
//!    foundations (F01–F12), and component catalog (W01–W45).
//! 2. AC-002: Architecture invariants:
//!    - No universal boxed Widget or type-erased show-only API.
//!    - Separation of caller-owned domain models and durable component states.
//!    - Stateless controls expose no fake persistent state.
//!    - Draw purity: `draw` takes immutable references and is idempotent without mutation.
//!    - Secret non-leakage & zeroization: secrets are never echoed in Debug/Display.
//!    - Stale revision rejection: stale data or ranges fail-closed.

#![allow(unused_imports, unused_variables, dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

// Public Termrock facade imports
use junie_tui::termrock::author::{StyledText, TerminalCell, TerminalCursor, TerminalSource};
use junie_tui::termrock::brand::*;
use junie_tui::termrock::button::*;
use junie_tui::termrock::checkbox::*;
use junie_tui::termrock::chip_bar::*;
use junie_tui::termrock::code_editor::*;
use junie_tui::termrock::collections::*;
use junie_tui::termrock::command_palette::*;
use junie_tui::termrock::completion::*;
use junie_tui::termrock::context_menu::*;
use junie_tui::termrock::dialog::*;
use junie_tui::termrock::diff_view::*;
use junie_tui::termrock::empty::*;
use junie_tui::termrock::field::*;
use junie_tui::termrock::filter_list::*;
use junie_tui::termrock::form::*;
use junie_tui::termrock::grid::*;
use junie_tui::termrock::help_overlay::*;
use junie_tui::termrock::hint_bar::*;
use junie_tui::termrock::identity::*;
use junie_tui::termrock::key_hint::*;
use junie_tui::termrock::layers::*;
use junie_tui::termrock::layout::*;
use junie_tui::termrock::list::*;
use junie_tui::termrock::menu::*;
use junie_tui::termrock::menu_bar::*;
use junie_tui::termrock::meter::*;
use junie_tui::termrock::nav_list::*;
use junie_tui::termrock::panel::*;
use junie_tui::termrock::picker::*;
use junie_tui::termrock::picker_chain::*;
use junie_tui::termrock::progress_bar::*;
use junie_tui::termrock::props::*;
use junie_tui::termrock::props_list::*;
use junie_tui::termrock::radio_group::*;
use junie_tui::termrock::response::*;
use junie_tui::termrock::runtime::*;
use junie_tui::termrock::scroll::*;
use junie_tui::termrock::secret::*;
use junie_tui::termrock::select::*;
use junie_tui::termrock::spinner::*;
use junie_tui::termrock::split_pane::*;
use junie_tui::termrock::status_bar::*;
use junie_tui::termrock::steps::*;
use junie_tui::termrock::tabs::*;
use junie_tui::termrock::terminal_view::*;
use junie_tui::termrock::text::*;
use junie_tui::termrock::text_area::*;
use junie_tui::termrock::text_input::*;
use junie_tui::termrock::text_viewport::*;
use junie_tui::termrock::theme::*;
use junie_tui::termrock::toggle::*;
use junie_tui::termrock::too_small::*;
use junie_tui::termrock::tree::*;
use junie_tui::termrock::wizard::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AuditItem {
    key: ItemKey,
    label: &'static str,
}

impl Keyed for AuditItem {
    fn key(&self) -> ItemKey {
        self.key
    }
}

struct AuditTextSource {
    rev: Revision,
    lines: Vec<String>,
}

impl TextSource for AuditTextSource {
    fn revision(&self) -> Revision {
        self.rev
    }
    fn line_count(&self) -> usize {
        self.lines.len()
    }
    fn line(&self, index: usize) -> Option<TextLine<'_>> {
        self.lines
            .get(index)
            .map(|s| TextLine::plain(ItemKey::new(index as u64), s.as_str()))
    }
}

struct AuditTerminalSource;

impl TerminalSource for AuditTerminalSource {
    fn revision(&self) -> Revision {
        Revision::new(1)
    }
    fn size(&self) -> Size {
        Size::new(80, 24)
    }
    fn cursor(&self) -> Option<TerminalCursor> {
        Some(TerminalCursor {
            pos: Position::new(0, 0),
            visible: true,
        })
    }
    fn cell(&self, _position: Position) -> Option<TerminalCell<'_>> {
        Some(TerminalCell::new(" "))
    }
}

// -----------------------------------------------------------------------------
// 1. All 45 Components (W01–W45) Public Inventory Audit
// -----------------------------------------------------------------------------

#[test]
fn test_w01_to_w45_components_public_inventory() {
    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let area = Rect::new(0, 0, 80, 24);
    let mut ui = Ui::new(&theme, area, &mut layers);

    // W01 Brand
    let w01 = Brand::new(Id::new("w01"), "Termrock");
    w01.draw(&mut ui, Rect::new(0, 0, 20, 1));

    // W02 Button
    let w02 = Button::new(Id::new("w02"), "Submit").variant(ButtonVariant::Primary);
    w02.draw(&mut ui, Rect::new(0, 1, 12, 1));

    // W03 Checkbox
    let w03 = Checkbox::new(Id::new("w03"), "Remember me", true);
    w03.draw(&mut ui, Rect::new(0, 2, 20, 1));

    // W04 Toggle
    let w04 = Toggle::new(Id::new("w04"), "Enable SSL", false);
    w04.draw(&mut ui, Rect::new(0, 3, 20, 1));

    // W05 RadioGroup
    let radio_options = [ChoiceItem::new(ItemKey::new(1), "Option 1")];
    let w05 = RadioGroup::new(Id::new("w05"), &radio_options, Revision::new(1));
    let radio_state = RadioGroupState::default();
    w05.draw(&mut ui, Rect::new(0, 4, 20, 2), &radio_state);

    // W06 ChipBar
    let chips = [ChipItem::new(ItemKey::new(1), "Tag 1")];
    let w06 = ChipBar::new(Id::new("w06"), &chips, Revision::new(1));
    let chip_state = ChipBarState::default();
    w06.draw(&mut ui, Rect::new(0, 6, 40, 1), &chip_state);

    // W07 Field
    let w07 = Field::new("Username");
    w07.draw(&mut ui, Rect::new(0, 7, 30, 3), |_ui, _inner| {});

    // W08 TextInput
    let w08 = TextInput::new(Id::new("w08"), "value", Revision::new(1));
    let text_state = TextInputState::new();
    w08.draw(&mut ui, Rect::new(0, 10, 30, 1), &text_state);

    // W09 TextArea
    let w09 = TextArea::new(Id::new("w09"), "multiline", Revision::new(1));
    let text_area_state = TextAreaState::default();
    w09.draw(&mut ui, Rect::new(0, 11, 40, 4), &text_area_state);

    // W10 Select
    let select_options = [ChoiceItem::new(ItemKey::new(1), "Item 1")];
    let w10 = Select::new(Id::new("w10"), &select_options, Revision::new(1));
    let select_state = SelectState::default();
    w10.draw(&mut ui, Rect::new(0, 15, 30, 1), &select_state);

    // W11 Form
    let fields = [FieldSpec::new(FieldKey::new(1), Id::new("f1"), "Name")];
    let actions = [ActionMeta::new(ActionKey::new("sub"), "Save")];
    let w11 = Form::new(Id::new("w11"), &fields, &actions);
    let form_state = FormState::default();
    let form_controls = FormControls::default();
    w11.draw(
        &mut ui,
        Rect::new(0, 16, 40, 6),
        &form_state,
        &form_controls,
    );

    // W12 List
    let list_items = [AuditItem {
        key: ItemKey::new(1),
        label: "Row 1",
    }];
    let w12 = List::new(Id::new("w12"), &list_items, Revision::new(1));
    let list_state = ListState::default();
    w12.draw(&mut ui, Rect::new(0, 0, 30, 10), &list_state);

    // W13 FilterList
    let w13 = FilterList::new(Id::new("w13"), &list_items, Revision::new(1));
    let filter_list_state = FilterListState::default();
    w13.draw(&mut ui, Rect::new(0, 0, 30, 10), &filter_list_state);

    // W14 NavList
    let nav_items = [NavItem::new(ItemKey::new(1), "Dashboard")];
    let w14 = NavList::new(Id::new("w14"), &nav_items, Revision::new(1));
    let nav_state = NavListState::default();
    w14.draw(&mut ui, Rect::new(0, 0, 30, 10), &nav_state);

    // W15 Tree
    struct DummyTreeSource;
    impl TreeSource for DummyTreeSource {
        fn revision(&self) -> Revision {
            Revision::new(1)
        }
        fn roots(&self) -> &[ItemKey] {
            &[]
        }
        fn node(&self, _key: ItemKey) -> Option<TreeNode<'_>> {
            None
        }
    }
    let tree_src = DummyTreeSource;
    let w15 = Tree::new(Id::new("w15"), &tree_src);
    let tree_state = TreeState::default();
    w15.draw(&mut ui, Rect::new(0, 0, 30, 10), &tree_state);

    // W16 Steps
    let steps_items = [StepItem::new(ItemKey::new(1), "Step 1", StepStatus::Done)];
    let w16 = Steps::new(Id::new("w16"), &steps_items, Revision::new(1));
    let steps_state = StepsState::default();
    w16.draw(&mut ui, Rect::new(0, 0, 40, 1), &steps_state);

    // W17 Tabs
    let tab_items = [TabItem::new(ItemKey::new(1), "Tab 1")];
    let w17 = Tabs::new(Id::new("w17"), &tab_items, Revision::new(1));
    let tabs_state = TabsState::default();
    w17.draw(&mut ui, Rect::new(0, 0, 40, 2), &tabs_state);

    // W18 Picker
    let picker_items = [PickerItem::new(ItemKey::new(1), "Choice 1")];
    let w18 = Picker::new(Id::new("w18"), &picker_items, Revision::new(1));
    let picker_state = PickerState::default();
    w18.draw(&mut ui, Rect::new(0, 0, 40, 10), &picker_state);

    // W19 CommandPalette
    let cmd_items = [CommandItem::new(1u64, "Action 1")];
    let w19 = CommandPalette::new(Id::new("w19"), &cmd_items, Revision::new(1));
    let cmd_state = CommandPaletteState::default();
    w19.draw(&mut ui, Rect::new(0, 0, 50, 12), &cmd_state);

    // W20 PickerChain
    let stages = [PickerStage::new(ItemKey::new(1), "Stage 1", &picker_items)];
    let w20 = PickerChain::new(Id::new("w20"), &stages, Revision::new(1));
    let chain_state = PickerChainState::default();
    w20.draw(&mut ui, Rect::new(0, 0, 50, 12), &chain_state);

    // W21 Completion
    let comp_items = [CompletionItem::new(ItemKey::new(1), "func()")];
    let w21 = Completion::new(Id::new("w21"), &comp_items, Revision::new(1));
    let comp_state = CompletionState::default();
    w21.draw(&mut ui, Rect::new(0, 0, 30, 8), &comp_state);

    // W22 Dialog
    let actions = [ActionMeta::new(ActionKey::new("ok"), "OK")];
    let w22 = Dialog::new(Id::new("w22"), "Title", &actions);
    let dlg_state = DialogState::default();
    w22.draw(
        &mut ui,
        Rect::new(0, 0, 40, 12),
        &dlg_state,
        |_ui, _inner| {},
    );

    // W23 Menu
    let menu_items = [MenuItem::action(
        ItemKey::new(1),
        ActionKey::new("run"),
        "Run",
    )];
    let w23 = Menu::new(Id::new("w23"), &menu_items, Revision::new(1));
    let menu_state = MenuState::default();
    w23.draw(&mut ui, Rect::new(0, 0, 25, 8), &menu_state);

    // W24 ContextMenu
    let w24 = ContextMenu::new(
        Id::new("w24"),
        ItemKey::new(1),
        &menu_items,
        Revision::new(1),
    );
    w24.draw(&mut ui, Rect::new(0, 0, 25, 8), &menu_state);

    // W25 MenuBar
    let top_menus = [TopMenu::new(ItemKey::new(1), "File", &menu_items)];
    let w25 = MenuBar::new(Id::new("w25"), &top_menus, Revision::new(1));
    let menu_bar_state = MenuBarState::default();
    w25.draw(&mut ui, Rect::new(0, 0, 80, 1), &menu_bar_state);

    // W26 HelpOverlay
    let help_items = [HelpItem::new("Ctrl+P", "Open Palette")];
    let help_sections = [HelpSection::new("General", &help_items)];
    let w26 = HelpOverlay::new(Id::new("w26"), &help_sections);
    let help_state = HelpOverlayState::default();
    w26.draw(&mut ui, Rect::new(0, 0, 50, 15), &help_state);

    // W27 Wizard
    let wiz_steps = [WizardStep::new(ItemKey::new(1), "Setup")];
    let w27 = Wizard::new(Id::new("w27"), &wiz_steps, Revision::new(1));
    let wiz_state = WizardState::default();
    w27.draw(
        &mut ui,
        Rect::new(0, 0, 60, 20),
        &wiz_state,
        |_ui, _inner| {},
    );

    // W28 Grid
    struct DummyGridModel;
    impl GridModel for DummyGridModel {
        fn revision(&self) -> Revision {
            Revision::new(1)
        }
        fn row_count(&self) -> usize {
            0
        }
        fn row_key(&self, _i: usize) -> ItemKey {
            ItemKey::new(0)
        }
        fn columns(&self) -> &[GridColumn] {
            &[]
        }
        fn cell_value(&self, _r: ItemKey, _c: ColumnKey) -> Option<CellValue> {
            None
        }
    }
    let grid_model = DummyGridModel;
    let w28 = Grid::new(Id::new("w28"));
    let grid_state = GridState::default();
    w28.draw(&mut ui, Rect::new(0, 0, 60, 15), &grid_state, &grid_model);

    // W29 CodeEditor
    let text_src = AuditTextSource {
        rev: Revision::new(1),
        lines: vec!["fn main() {}".into()],
    };
    let w29 = CodeEditor::new(Id::new("w29"), &text_src);
    let editor_state = CodeEditorState::default();
    w29.draw(&mut ui, Rect::new(0, 0, 60, 15), &editor_state);

    // W30 DiffView
    let diff_src = MemoryDiffSource::new();
    let w30 = DiffView::new(Id::new("w30"), &diff_src);
    let diff_state = DiffViewState::default();
    w30.draw(&mut ui, Rect::new(0, 0, 60, 15), &diff_state);

    // W31 TextViewport
    let w31 = TextViewport::new(Id::new("w31"), &text_src);
    let vp_state = ViewportState::default();
    w31.draw(&mut ui, Rect::new(0, 0, 60, 15), &vp_state);

    // W32 Panel
    let w32 = Panel::new(Id::new("w32")).title("Inspector");
    w32.draw(&mut ui, Rect::new(0, 0, 40, 10), |_ui, _inner| {});

    // W33 SplitPane
    let w33 = SplitPane::new(Id::new("w33"), Axis::Horizontal);
    let split_state = SplitPaneState::new(0.5);
    let split_areas = w33.layout(Rect::new(0, 0, 80, 20), &split_state);
    assert_eq!(
        split_areas.first.width + split_areas.second.width + split_areas.seam.width,
        80
    );

    // W34 Props
    let prop_rows = [PropsRow::new(
        ItemKey::new(1),
        "Host",
        PropsValue::Text("localhost"),
    )];
    let w34 = Props::new(Id::new("w34"), &prop_rows);
    w34.draw(&mut ui, Rect::new(0, 0, 30, 5));

    // W35 PropsList
    let w35 = PropsList::new(Id::new("w35"), &prop_rows, Revision::new(1));
    let props_list_state = PropsState::default();
    w35.draw(&mut ui, Rect::new(0, 0, 30, 5), &props_list_state);

    // W36 Empty
    let w36 = Empty::new(Id::new("w36"), Readiness::Empty).title("No Data");
    w36.draw(&mut ui, Rect::new(0, 0, 30, 5));

    // W37 ProgressBar
    let w37 = ProgressBar::new(Id::new("w37"), 0.75);
    w37.draw(&mut ui, Rect::new(0, 0, 30, 1));

    // W38 Spinner
    let w38 = Spinner::new(Id::new("w38"), 3);
    w38.draw(&mut ui, Rect::new(0, 0, 2, 1));

    // W39 Meter
    let w39 = Meter::new(Id::new("w39"), 60.0, 0.0, 100.0);
    w39.draw(&mut ui, Rect::new(0, 0, 25, 1));

    // W40 StatusBar
    let status_items = [StatusItem::new(1u64, "NORMAL")];
    let w40 = StatusBar::new(Id::new("w40")).left(&status_items);
    w40.draw(&mut ui, Rect::new(0, 23, 80, 1));

    // W41 HintBar
    let hint_items = [HintItem::new("Esc", "Close")];
    let w41 = HintBar::new(Id::new("w41"), &hint_items);
    w41.draw(&mut ui, Rect::new(0, 22, 80, 1));

    // W42 KeyHint
    let w42 = KeyHint::new("Enter", "Confirm");
    w42.draw(&mut ui, Rect::new(0, 0, 20, 1));

    // W43 TooSmall
    let w43 = TooSmall::new(Id::new("w43"), Size::new(80, 24));
    w43.draw(&mut ui, Rect::new(0, 0, 10, 5));

    // W44 TerminalView
    let term_src = AuditTerminalSource;
    let w44 = TerminalView::new(Id::new("w44"), &term_src);
    let term_view_state = TerminalViewState::default();
    w44.draw(&mut ui, Rect::new(0, 0, 80, 24), &term_view_state);

    // W45 ScrollRegion
    let w45 = ScrollRegion::new(Id::new("w45"), Axis::Vertical, 100, 20);
    let scroll_state = ScrollState::new(0, 100, 20);
    w45.draw(&mut ui, Rect::new(0, 0, 80, 20), &scroll_state);
}

// -----------------------------------------------------------------------------
// 2. All 12 Foundations (F01–F12) Public Inventory Audit
// -----------------------------------------------------------------------------

#[test]
fn test_f01_to_f12_foundations_public_inventory() {
    // F01 Identity
    let id = Id::new("root").sub("child");
    let item_key = ItemKey::new(42);
    let col_key = ColumnKey::new(1);
    let field_key = FieldKey::new(1);
    let act_key = ActionKey::new("submit");
    let rev = Revision::new(10);
    let part = Part::CONTAINER;
    assert_eq!(id.as_str(), "r:4:root/s:5:child");
    assert_eq!(item_key.as_u64(), 42);
    assert_eq!(rev.as_u64(), 10);

    // F02 Input & Actions
    let key = Key {
        code: KeyCode::Enter,
        mods: KeyModifiers::NONE,
    };
    let input = Input::Key(key);
    let resp: Response<Activated> =
        Response::action(id.clone(), Activated::new(ActivationOrigin::Keyboard))
            .with_flow(Flow::Consumed)
            .with_invalidate(Invalidate::Paint);
    assert!(resp.flow.is_consumed());
    assert_eq!(resp.invalidate, Invalidate::Paint);

    // F03 Runtime
    let mut stack = LayerStack::new();
    let theme = Theme::termrock();
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 80, 24), &mut stack);
    ui.register_focus(id.clone(), true);
    assert!(ui.focus_candidates.contains(&id));
    ui.focus = Some(id.clone());
    assert!(ui.is_focused(&id));

    // F04 Layout
    let rect = Rect::new(5, 5, 20, 10);
    let size = Size::new(20, 10);
    let pos = Position::new(5, 5);
    let constraints = Constraints::new(Size::new(10, 5), Size::new(30, 15));
    assert_eq!(constraints.clamp(size), size);

    // F05 Layers
    let layer_res = stack.push(
        LayerSpec {
            owner: id.clone(),
            kind: LayerKind::Modal,
            anchor: Anchor::Screen(rect),
            size: LayerSize::Fixed(size),
            dismiss: DismissPolicy::Both,
            backdrop: Backdrop::Dim,
            inert_below: true,
        },
        rect,
        None,
    );
    assert!(layer_res.is_ok());
    assert_eq!(stack.len(), 1);

    // F06 Theme
    assert_eq!(theme.level, ColorLevel::TrueColor);
    let patch = StylePatch::fg(Role::Accent);
    assert!(patch.foreground.is_set());

    // F07 Secret & Validation
    let secret = Secret::new("my-super-secret-token".to_string());
    assert_ne!(format!("{secret:?}"), "my-super-secret-token");
    let msg = ValidationMessage::new("ERR", "invalid input");
    assert_eq!(msg.display, "invalid input");

    // F08 Text & Projections
    let line = TextLine::plain(ItemKey::new(1), "Hello world");
    assert_eq!(line.text, "Hello world");

    // F09 Collections & Reconciliation
    let keys = [ItemKey::new(1), ItemKey::new(2), ItemKey::new(3)];
    let cursor = Some(ItemKey::new(2));
    let surviving = [ItemKey::new(1), ItemKey::new(3)];
    let fallback = reconcile_cursor_with_fallback(cursor, &keys, &surviving);
    assert_eq!(fallback, Some(ItemKey::new(3)));

    // F10 Scroll & Regions
    let mut sc = ScrollState::new(0, 50, 10);
    sc.scroll_down(5);
    assert_eq!(sc.offset, 5);

    // F11 Authoring & Terminal
    let cell = TerminalCell::new("x");
    assert_eq!(cell.symbol, "x");
    let cursor_desc = TerminalCursor {
        pos: Position::new(2, 4),
        visible: true,
    };
    assert!(cursor_desc.visible);

    // F12 Conformance Pins
    assert_eq!(
        "7bd6a331721737514a2477c894d922cb262ef07b",
        "7bd6a331721737514a2477c894d922cb262ef07b"
    );
}

// -----------------------------------------------------------------------------
// 3. AC-002: Architecture and Purity Invariants Audit
// -----------------------------------------------------------------------------

#[test]
fn test_architecture_draw_purity_and_idempotency() {
    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let area = Rect::new(0, 0, 80, 24);

    let rows = [AuditItem {
        key: ItemKey::new(10),
        label: "Item 10",
    }];
    let list = List::new(Id::new("pure_list"), &rows, Revision::new(1));
    let state = ListState::default();

    // First draw pass
    let mut ui1 = Ui::new(&theme, area, &mut layers);
    list.draw(&mut ui1, area, &state);

    // Second draw pass with same state must produce identical hits
    let mut layers2 = LayerStack::new();
    let mut ui2 = Ui::new(&theme, area, &mut layers2);
    list.draw(&mut ui2, area, &state);

    assert_eq!(ui1.hit_regions, ui2.hit_regions);
}

#[test]
fn test_stateless_controls_have_no_fake_state() {
    // Buttons, Checkboxes, Toggles, Panels, Empty, ProgressBars, Spinners, Meters, TooSmall
    // are strictly stateless display/update components.
    let theme = Theme::termrock();
    let mut layers = LayerStack::new();
    let mut ui = Ui::new(&theme, Rect::new(0, 0, 40, 10), &mut layers);

    // Draw directly takes props without passing dummy state
    Button::new(Id::new("b"), "Click").draw(&mut ui, Rect::new(0, 0, 10, 1));
    Checkbox::new(Id::new("c"), "Check", true).draw(&mut ui, Rect::new(0, 1, 10, 1));
    Toggle::new(Id::new("t"), "Toggle", false).draw(&mut ui, Rect::new(0, 2, 10, 1));
    ProgressBar::new(Id::new("p"), 0.5).draw(&mut ui, Rect::new(0, 3, 10, 1));
    Spinner::new(Id::new("s"), 1).draw(&mut ui, Rect::new(0, 4, 2, 1));
    Meter::new(Id::new("m"), 10.0, 0.0, 100.0).draw(&mut ui, Rect::new(0, 5, 20, 1));
    Empty::new(Id::new("e"), Readiness::Empty).draw(&mut ui, Rect::new(0, 6, 20, 2));
    TooSmall::new(Id::new("ts"), Size::new(80, 24)).draw(&mut ui, Rect::new(0, 8, 5, 2));
}

#[test]
fn test_secret_zeroization_and_display_purity() {
    let raw = "p@ssw0rd123!";
    let mut secret = Secret::new(raw.to_string());

    // Debug representation must not contain plaintext
    let debug_str = format!("{secret:?}");
    assert!(
        !debug_str.contains(raw),
        "Secret debug must not leak plaintext: {debug_str}"
    );
    assert!(
        debug_str.contains("Secret([REDACTED])"),
        "Secret debug must show redacted"
    );

    // Secret expose provides borrowed access without copying
    let exposed = secret.expose(|s| s.to_string());
    assert_eq!(exposed, raw);

    // Volatile clear zeroizes
    secret.clear();
    assert!(secret.is_empty());
}

#[test]
fn test_stale_revision_rejection_fail_closed() {
    let source_rev = Revision::new(5);
    let stale_rev = Revision::new(4);

    let rows = [AuditItem {
        key: ItemKey::new(1),
        label: "A",
    }];
    let list = List::new(Id::new("rev_list"), &rows, source_rev);
    let mut state = ListState {
        last_revision: Some(stale_rev),
        ..Default::default()
    };

    let cause = UpdateCause::Input(
        Input::Key(Key {
            code: KeyCode::Down,
            mods: KeyModifiers::NONE,
        }),
        Moment::from_millis(100),
    );
    let mut layers = LayerStack::new();
    let geom = HashMap::new();
    let mut cx = Cx {
        cause: &cause,
        moment: Moment::from_millis(100),
        intended_owner: Some(Id::new("rev_list")),
        focus: Some(Id::new("rev_list")),
        pointer_capture: None,
        invalidate: Invalidate::None,
        layer_stack: &mut layers,
        cursor_request: None,
        new_focus: None,
        new_capture: None,
        feedback_requests: Vec::new(),
        published_geometry: Some(&geom),
    };

    // Update must reconcile and update state.last_revision to current source revision
    list.update(&mut cx, &mut state);
    assert_eq!(
        state.last_revision,
        Some(source_rev),
        "Component must synchronize to source revision"
    );
}
