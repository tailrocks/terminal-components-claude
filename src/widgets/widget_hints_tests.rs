use crate::core::hints::{HasHints, HintCx};
use crate::core::id::WidgetId;
use crate::core::list::NavProfile;
use crate::widgets::button::Button;
use crate::widgets::choice_dialog::ChoiceDialog;
use crate::widgets::form_dialog::{FormDialog, FormField};
use crate::widgets::help::HelpOverlay;
use crate::widgets::info_dialog::InfoDialog;
use crate::widgets::input::TextInput;
use crate::widgets::list::{ListBox, ListItem, SelectMode};
use crate::widgets::menu::{ContextMenu, MenuBar, MenuItem};
use crate::widgets::option_list::Item;
use crate::widgets::picker::Picker;
use crate::widgets::props::Prop;
use crate::widgets::select::Select;
use crate::widgets::tree::{TreeNode, TreeView};

fn line(h: &dyn HasHints) -> String {
    h.hints(&HintCx::default())
        .iter()
        .map(|h| format!("{} {}", h.key, h.action))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn under<W: HasHints>(profile: NavProfile, build: impl FnOnce() -> W) -> String {
    profile.install();
    let w = build();
    NavProfile::Desktop.install();
    line(&w)
}

fn both<W: HasHints>(build: impl Fn() -> W) -> (String, String) {
    (
        under(NavProfile::Desktop, &build),
        under(NavProfile::Vim, &build),
    )
}

const ID: WidgetId = WidgetId::of("w");

#[test]
fn list_hints_under_desktop_and_vim() {
    let single = || {
        ListBox::new(
            ID,
            vec![ListItem::new("Alpha"), ListItem::new("Beta")],
            SelectMode::Single,
        )
    };
    assert_eq!(
        both(single),
        ("↑↓ Move · Type to jump".into(), "↑↓ Move".into())
    );
    let multi = || {
        ListBox::new(
            ID,
            vec![ListItem::new("Alpha"), ListItem::new("Beta")],
            SelectMode::Multi,
        )
    };
    assert_eq!(
        both(multi).0,
        "↑↓ Move · Type to jump · Space Toggle · Ctrl+A Select all"
    );
}

#[test]
fn tree_hints_under_desktop_and_vim() {
    let tree = || TreeView::new(ID, vec![TreeNode::leaf("a"), TreeNode::leaf("b")]);
    assert_eq!(
        both(tree),
        (
            "↑↓ Move · Type to jump · ←→ Fold".into(),
            "↑↓ Move · ←→ Fold".into()
        )
    );
}

#[test]
fn select_hints_open_and_closed() {
    let mut s = Select::new(ID, "Kind", &["a", "b"], 0);
    assert_eq!(line(&s), "Enter Open");
    s.open = true;
    assert_eq!(
        line(&s),
        "↑↓ Move · Type to jump · Enter Choose · Esc Close"
    );
}

#[test]
fn menu_hints_under_desktop_and_vim() {
    let menu = || ContextMenu::new(ID, vec![MenuItem::new("Copy"), MenuItem::new("Paste")]);
    assert_eq!(
        both(menu),
        (
            "↑↓ Move · Type to jump · Enter Choose · Esc Close".into(),
            "↑↓ Move · Enter Choose · Esc Close".into()
        )
    );
    let bar = MenuBar::new(ID, vec![("File", vec![MenuItem::new("Quit")])]);
    assert_eq!(line(&bar), "", "a closed bar names nothing");
}

#[test]
fn picker_hints_follow_search_and_query() {
    let mut p = Picker::new(ID, "Open");
    p.items = vec![Item::new("a")];
    assert_eq!(
        line(&p),
        "Type Filter · ↑↓ Move · Enter Choose · Esc Cancel"
    );
    p.set_query("a");
    assert_eq!(line(&p), "Type Filter · ↑↓ Move · Enter Choose · Esc Clear");
    let fixed = Picker::new(ID, "Mode")
        .choose_hint("Switch")
        .dismiss(crate::ui::popup::Dismiss::ESCAPE);
    let mut fixed = fixed;
    fixed.set_searchable(false);
    assert_eq!(line(&fixed), "↑↓ Move · Enter Switch · Esc Cancel");
}

#[test]
fn modal_hints_name_their_own_buttons() {
    let choice = ChoiceDialog::new(ID, "Pick", "Which", &["One", "Two"], 0);
    assert_eq!(
        line(&choice),
        "↑↓ Choose · Enter Next · Tab Next · Esc Cancel"
    );

    let form = FormDialog::new(
        ID,
        "Add",
        vec![FormField::input(
            "name",
            TextInput::new(WidgetId::of("n"), "Name"),
        )],
    );
    let h = line(&form);
    assert!(h.starts_with("Tab Next field · "), "{h}");
    assert!(h.ends_with("Esc Cancel"), "{h}");

    let info = InfoDialog::new(ID, "About", vec![Prop::new("Version", "1.0").copyable()]);
    assert_eq!(line(&info), "↑↓ Move · y Copy · Esc Close");
    let plain = InfoDialog::new(ID, "About", vec![Prop::new("Version", "1.0")]);
    assert_eq!(line(&plain), "↑↓ Move · Esc Close", "nothing to copy, no y");

    let help = HelpOverlay::new(ID, "", vec![]);
    assert_eq!(line(&help), "↑↓ Scroll · Esc Close");
}

#[test]
fn a_busy_button_says_it_is_working_instead_of_naming_enter() {
    let mut b = Button::primary(ID, "Save");
    assert_eq!(line(&b), "Enter Save");
    b.busy = true;
    assert_eq!(line(&b), " Working…", "K35: Enter does nothing while busy");
}

#[test]
fn a_text_field_waiting_for_enter_hints_edit() {
    let i = TextInput::new(ID, "Name");
    let h = line(&i);
    assert!(h == "Enter Edit" || h.starts_with("Enter "), "{h}");
}

#[test]
fn completion_hints_read_its_own_keys() {
    use crate::widgets::completion::Completion;
    use crate::widgets::option_list::Item;
    let open = || {
        let mut c = Completion::new(ID);
        let item = Item::new("SELECT").glyph("K");
        c.open(vec![item], ratatui::layout::Rect::new(0, 0, 10, 1), 0);
        c
    };
    assert_eq!(
        both(open),
        (
            "↑↓ Move · Enter Accept · Esc Close".into(),
            "↑↓ Move · Enter Accept · Esc Close".into()
        )
    );
    assert_eq!(line(&Completion::new(ID)), "");
}
