//! The "Inputs & forms" widget pages through the real app (plan §7, SC7
//! Buttons, SC8 Inputs and Text areas, SC9 Checks & radios, SC10 Chips &
//! selects): options set through `--opt` change the playground, the States
//! rows render, and the key interactions work.

use junie_tui::runtime::Application;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use crate::app::{App, PageId};
use crate::options;
use crate::pages::template::Section;
use junie_tui::core::event::{Input, Key, Mouse, MouseKind};
use junie_tui::theme::Theme;

struct Harness {
    app: App,
    term: Terminal<TestBackend>,
}

#[allow(dead_code)] // each page's tests use what they need
impl Harness {
    fn new(w: u16, h: u16, page: PageId) -> Self {
        let mut app = App::new(Theme::junie());
        app.goto(page);
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut h = Self { app, term };
        h.draw();
        h
    }

    fn page(&self) -> PageId {
        self.app.page
    }

    fn opt(&mut self, arg: &str) {
        self.app
            .apply_opt(options::parse_opt(arg).unwrap_or_else(|e| panic!("{arg}: {e}")));
        self.draw();
    }

    fn section(&mut self, s: Section) {
        let p = self.page();
        self.app.pages[p.index()]
            .template_mut()
            .expect("templated page")
            .set_section(s);
        self.draw();
    }

    fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }

    fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) {
        self.app.handle(Input::Key(Key { code, mods }));
        self.draw();
    }

    fn key(&mut self, code: KeyCode) {
        self.key_mod(code, KeyModifiers::NONE);
    }

    fn type_str(&mut self, s: &str) {
        for c in s.chars() {
            self.key(KeyCode::Char(c));
        }
    }

    fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) {
        self.app
            .handle(Input::Mouse(Mouse::new(kind, Position::new(x, y))));
        self.draw();
    }

    fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
    }

    fn row(&self, y: u16) -> String {
        let buf = self.term.backend().buffer();
        (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
    }

    fn text(&self) -> String {
        let buf = self.term.backend().buffer();
        (0..buf.area.height)
            .map(|y| self.row(y))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Column and row of `needle`'s first cell.
    fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let buf = self.term.backend().buffer();
        let want: Vec<&str> =
            unicode_segmentation::UnicodeSegmentation::graphemes(needle, true).collect();
        for y in 0..buf.area.height {
            let cells: Vec<&str> = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
            for x in 0..cells.len().saturating_sub(want.len() - 1) {
                if cells[x..x + want.len()] == want[..] {
                    return Some((x as u16, y));
                }
            }
        }
        None
    }

    fn at(&self, needle: &str) -> (u16, u16) {
        self.find(needle)
            .unwrap_or_else(|| panic!("{needle:?} not on screen:\n{}", self.text()))
    }

    fn status(&self) -> String {
        self.app.shell.status.text().unwrap_or_default().to_owned()
    }
}

/// Print every section of the pages in `DUMP_PAGES` at 120×40, after the
/// `--opt` values in `DUMP_OPTS` (`--run-ignored only --no-capture`).
#[test]
#[ignore]
fn dump_pages() {
    let pages = std::env::var("DUMP_PAGES").unwrap_or_else(|_| "buttons".into());
    for name in pages.split(',') {
        let page = PageId::from_name(name).unwrap();
        let mut h = Harness::new(120, 40, page);
        if let Ok(opts) = std::env::var("DUMP_OPTS") {
            for o in opts.split(',').filter(|o| !o.is_empty()) {
                h.opt(o);
            }
        }
        for s in Section::ALL {
            h.section(s);
            println!("==== {name} · {}\n{}", s.label(), h.text());
        }
    }
}

// ------------------------------------------------------------------ SC7

fn tool_id(i: usize) -> junie_tui::core::id::WidgetId {
    junie_tui::core::id::WidgetId::of("buttons")
        .sub("tools")
        .child(i)
}

/// The focus bar (`▎` in the focus ink) on row `y`, if any.
fn focus_bar_on(h: &Harness, y: u16) -> bool {
    let buf = h.term.backend().buffer();
    (0..buf.area.width).any(|x| {
        let c = &buf[(x, y)];
        c.symbol() == "▎" && c.fg == Theme::junie().focus
    })
}

#[test]
fn buttons_toolbar_is_off_the_tab_order_until_the_option_puts_it_in() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    h.at("▶ Run");
    assert!(
        !h.app.shell.ring.contains(tool_id(0)),
        "toolbar buttons take clicks only"
    );
    assert!(h.text().contains("not in the Tab order"));
    h.opt("buttons.toolbar_focus=on");
    assert!(h.app.shell.ring.contains(tool_id(0)));
    assert!(!h.text().contains("not in the Tab order"));
}

#[test]
fn buttons_toolbar_runs_and_cancels_by_shortcut_and_click() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    // Cancel is disabled until a query runs: a click does nothing
    let (ex, y) = h.at("Explain");
    let cancel = ex + "Explain".len() as u16 + 4;
    h.click(cancel, y);
    assert!(!h.status().contains("cancelled"), "{}", h.status());
    h.key(KeyCode::F(5));
    assert!(h.status().contains("Running"), "{}", h.status());
    h.click(cancel, y);
    assert_eq!(h.status(), "Query cancelled");
    h.key(KeyCode::F(5));
    h.key_mod(KeyCode::F(5), KeyModifiers::SHIFT);
    assert_eq!(h.status(), "Query cancelled");
    let (rx, ry) = h.at("▶ Run");
    h.click(rx + 1, ry);
    assert!(h.status().contains("Running"), "{}", h.status());
}

#[test]
fn buttons_focus_visible_keyboard_only_hides_the_ring_after_a_click() {
    for (value, ring) in [("always", true), ("keyboard-only", false)] {
        let mut h = Harness::new(120, 40, PageId::Buttons);
        h.opt(&format!("buttons.focus_visible={value}"));
        let (x, y) = h.at("Preview");
        h.click(x, y);
        assert!(h.status().contains("Preview"), "{}", h.status());
        assert_eq!(focus_bar_on(&h, y), ring, "{value}: {}", h.row(y));
    }
}

#[test]
fn buttons_row_folds_wraps_and_aligns_through_the_options() {
    let mut h = Harness::new(120, 60, PageId::Buttons);
    assert!(h.text().contains("every button fits"));
    h.opt("buttons.row_width=30");
    assert!(
        h.text().contains("behind …: Validate, Duplicate, Edit…"),
        "{}",
        h.text()
    );
    // the `…` button names what folded
    let (x, y) = h.at("…    Remove…");
    h.click(x, y);
    assert_eq!(h.status(), "3 more: Validate, Duplicate, Edit…");
    // wrapping keeps every label on screen
    h.opt("buttons.row_overflow=wrap");
    for label in ["Refresh", "Validate", "Duplicate", "Edit…", "Remove…"] {
        assert!(h.find(label).is_some(), "{label}");
    }
    let (_, refresh) = h.at("Refresh");
    let (_, remove) = h.at("Remove…");
    assert!(remove > refresh, "the row wraps onto a second line");
    // End packs the row against its right edge
    h.opt("buttons.row_overflow=more");
    let start = h.at("Refresh").0;
    h.opt("buttons.row_align=end");
    assert!(h.at("Refresh").0 > start);
}

#[test]
fn buttons_options_section_drives_the_playground() {
    let mut h = Harness::new(120, 40, PageId::Buttons);
    let start = h.at("Refresh").0;
    h.section(Section::Options);
    let (x, y) = h.at("start");
    h.click(x, y);
    // the list is open; End is its second option
    let (ex, ey) = h.at("end ");
    h.click(ex, ey);
    assert_eq!(
        h.app.pages[PageId::Buttons.index()].option_value("buttons.row_align"),
        Some(1)
    );
    h.section(Section::Playground);
    assert!(h.at("Refresh").0 > start);
}

#[test]
fn buttons_states_grid_follows_its_options_and_shows_the_icon_button() {
    let mut h = Harness::new(160, 50, PageId::Buttons);
    h.section(Section::States);
    assert!(h.text().contains("Icon ×"));
    let (x15, _) = h.at("Secondary");
    h.opt("buttons.label_width=20");
    assert_eq!(h.at("Secondary").0, x15 + 5);
    let marks = |h: &Harness| h.text().matches('▏').count();
    assert!(marks(&h) > 0);
    h.opt("buttons.focus_mark=colon");
    assert_eq!(marks(&h), 0);
}

/// The States legend explains the mark the grid draws: `:` under ASCII
/// glyphs or a `focus_mark` override, never a `▏` the grid no longer shows.
#[test]
fn states_legend_names_the_mark_the_grid_draws() {
    for opt in ["shell.glyphs=ascii", "buttons.focus_mark=colon"] {
        let mut h = Harness::new(160, 50, PageId::Buttons);
        h.section(Section::States);
        h.opt(opt);
        let text = h.text();
        assert!(text.contains(": depicts a focused sample"), "{opt}: {text}");
        assert!(!text.contains('▏'), "{opt}: {text}");
    }
}

#[test]
fn buttons_icon_pad_sets_the_plate() {
    let mut h = Harness::new(120, 60, PageId::Buttons);
    let gap = |h: &Harness| {
        let (_, y) = h.at("Danger hover");
        let row = h.row(y);
        let i = row.find("Danger hover").unwrap() + "Danger hover".len();
        row[i..].chars().take_while(|c| *c != '×').count()
    };
    let one = gap(&h);
    h.opt("buttons.icon_pad=2");
    assert_eq!(gap(&h), one + 1);
}

#[test]
fn buttons_row_arrows_stop_at_the_ends_unless_wrap_is_on() {
    let row = junie_tui::core::id::WidgetId::of("buttons").sub("row");
    for (wrap, after) in [("off", 4), ("on", 0)] {
        let mut h = Harness::new(120, 40, PageId::Buttons);
        h.opt(&format!("buttons.row_wrap={wrap}"));
        let (x, y) = h.at("Remove…");
        h.click(x, y);
        assert_eq!(h.app.shell.focus.current(), Some(row.child(4)));
        h.key(KeyCode::Right);
        assert_eq!(
            h.app.shell.focus.current(),
            Some(row.child(after)),
            "{wrap}"
        );
        h.key(KeyCode::Left);
        h.key(KeyCode::Left);
        let back = if wrap == "on" { 3 } else { 2 };
        assert_eq!(h.app.shell.focus.current(), Some(row.child(back)), "{wrap}");
    }
}

// ------------------------------------------------------------------ SC8

fn input_field(h: &Harness, i: usize) -> &junie_tui::widgets::input::TextInput {
    let page = h.app.pages[PageId::Inputs.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<crate::pages::inputs::InputsPage>())
        .expect("inputs page");
    page.field(i)
}

fn input_text(h: &Harness, i: usize) -> String {
    input_field(h, i).field.buffer.text().to_owned()
}

/// Focus Project name from the keyboard (strip, then Tab).
fn tab_to_project(h: &mut Harness) {
    let (x, y) = h.at("Playground    Options");
    h.click(x + 1, y);
    h.key(KeyCode::Tab);
    assert_eq!(
        h.app.shell.focus.current(),
        Some(input_field(h, 0).id),
        "{}",
        h.text()
    );
}

#[test]
fn inputs_edit_entry_and_focus_select_shape_how_focus_starts_an_edit() {
    // the default: focus edits with the whole value selected
    let mut h = Harness::new(120, 40, PageId::Inputs);
    tab_to_project(&mut h);
    assert!(input_field(&h, 0).editing);
    h.type_str("x");
    assert_eq!(input_text(&h, 0), "x");
    // caret at the end: typing continues the value
    let mut h = Harness::new(120, 40, PageId::Inputs);
    h.opt("inputs.focus_select=caret-end");
    tab_to_project(&mut h);
    h.type_str("x");
    assert_eq!(input_text(&h, 0), "payments-gatewayx");
    let mut h = Harness::new(120, 40, PageId::Inputs);
    h.opt("inputs.focus_select=caret-start");
    tab_to_project(&mut h);
    h.type_str("x");
    assert_eq!(input_text(&h, 0), "xpayments-gateway");
    // explicit: focus waits for Enter
    let mut h = Harness::new(120, 40, PageId::Inputs);
    h.opt("inputs.edit_entry=explicit");
    tab_to_project(&mut h);
    assert!(!input_field(&h, 0).editing);
    h.key(KeyCode::Enter);
    assert!(input_field(&h, 0).editing);
}

#[test]
fn inputs_field_keys_and_a_custom_text_keymap_select_all() {
    for (opt, key, mods) in [
        ("inputs.field_keys=emacs", 'l', KeyModifiers::CONTROL),
        ("inputs.field_keys=desktop", 'a', KeyModifiers::CONTROL),
        ("inputs.text_keymap=custom", 'a', KeyModifiers::ALT),
    ] {
        let mut h = Harness::new(120, 40, PageId::Inputs);
        h.opt("inputs.focus_select=caret-end");
        h.opt(opt);
        tab_to_project(&mut h);
        h.key_mod(KeyCode::Char(key), mods);
        h.type_str("y");
        assert_eq!(input_text(&h, 0), "y", "{opt}");
    }
}

#[test]
fn inputs_text_commands_list_both_presets_and_the_custom_binding() {
    let mut h = Harness::new(120, 60, PageId::Inputs);
    let text = h.text();
    assert!(text.contains("Desktop keys") && text.contains("Emacs keys"));
    assert!(!text.contains("Alt+A"));
    h.opt("inputs.text_keymap=custom");
    let text = h.text();
    assert!(text.contains("Alt+A · Ctrl+A"), "{text}");
    assert!(text.contains("Alt+A · Ctrl+L"), "{text}");
}

#[test]
fn inputs_blur_action_decides_what_leaving_an_edit_does() {
    for (value, kept) in [("commit", true), ("cancel", false), ("keep", true)] {
        let mut h = Harness::new(120, 40, PageId::Inputs);
        h.opt("inputs.focus_select=caret-end");
        h.opt(&format!("inputs.blur_action={value}"));
        tab_to_project(&mut h);
        h.type_str("-v2");
        let (x, y) = h.at("mira@example");
        h.click(x, y);
        let text = input_text(&h, 0);
        assert_eq!(text == "payments-gateway-v2", kept, "{value}: {text}");
        assert_eq!(input_field(&h, 0).editing, value == "keep", "{value}");
    }
}

#[test]
fn inputs_label_options_wrap_hide_optional_and_take_clicks() {
    let mut h = Harness::new(120, 40, PageId::Inputs);
    assert!(h.text().contains("once the task is do…"));
    h.opt("inputs.wrap_label=on");
    let text = h.text();
    assert!(!text.contains("once the task is do…"), "{text}");
    assert!(text.contains("done"), "{text}");
    assert!(text.contains("Search files  optional"));
    h.opt("inputs.plain_label=on");
    assert!(!h.text().contains("optional"));
    // a label click focuses its field, unless label_hit is off
    for (value, focused) in [("on", true), ("off", false)] {
        let mut h = Harness::new(120, 40, PageId::Inputs);
        h.opt(&format!("inputs.label_hit={value}"));
        let (x, y) = h.at("Search files");
        h.click(x + 2, y);
        assert_eq!(
            h.app.shell.focus.current() == Some(input_field(&h, 4).id),
            focused,
            "label_hit={value}"
        );
    }
}

#[test]
fn inputs_required_message_overrides_the_theme_text() {
    let mut h = Harness::new(120, 40, PageId::Inputs);
    h.opt("inputs.required_message=custom");
    tab_to_project(&mut h);
    h.key(KeyCode::Backspace);
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Name the project to continue"),
        "{}",
        h.text()
    );
}

#[test]
fn inputs_pointer_select_off_keeps_a_double_click_from_selecting() {
    for (value, selects) in [("default", true), ("off", false)] {
        let mut h = Harness::new(120, 40, PageId::Inputs);
        h.opt(&format!("inputs.pointer_select={value}"));
        let (x, y) = h.at("payments-gateway");
        h.click(x + 2, y);
        h.click(x + 2, y);
        let sel = input_field(&h, 0).field.buffer.selected_text();
        assert_eq!(sel.is_some(), selects, "{value}: {sel:?}");
    }
}

fn area_of(h: &Harness, i: usize) -> &junie_tui::widgets::textarea::TextArea {
    h.app.pages[PageId::TextAreas.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<crate::pages::textareas::TextAreasPage>())
        .expect("text areas page")
        .area(i)
}

#[test]
fn textareas_esc_reverts_and_focus_select_through_the_options() {
    for (value, reverted) in [("off", false), ("on", true)] {
        let mut h = Harness::new(120, 40, PageId::TextAreas);
        h.opt(&format!("textareas.esc_reverts={value}"));
        let (x, y) = h.at("Anything the agent");
        h.click(x, y);
        h.type_str("hello");
        h.key(KeyCode::Esc);
        assert_eq!(
            area_of(&h, 1).field.buffer.text().is_empty(),
            reverted,
            "esc_reverts={value}"
        );
    }
    let mut h = Harness::new(120, 40, PageId::TextAreas);
    h.opt("textareas.focus_select=select-all");
    h.opt("textareas.edit_entry=immediate");
    let (x, y) = h.at("Playground    Options");
    h.click(x + 1, y);
    h.key(KeyCode::Tab);
    h.type_str("z");
    assert_eq!(area_of(&h, 0).field.buffer.text(), "z");
}

#[test]
fn textareas_states_rows_render() {
    let mut h = Harness::new(160, 60, PageId::TextAreas);
    h.section(Section::States);
    let text = h.text();
    for row in [
        "rest", "hover", "focused", "editing", "error", "disabled", "empty",
    ] {
        assert!(text.contains(row), "{row}: {text}");
    }
    assert!(text.contains("Explain why, in the imperative"));
}

// ------------------------------------------------------------------ SC9

#[test]
fn checks_hit_decides_whether_blank_row_space_toggles() {
    for (value, toggles) in [("label", false), ("row", true)] {
        let mut h = Harness::new(120, 40, PageId::ChecksRadios);
        h.opt(&format!("checksradios.hit={value}"));
        let (x, y) = h.at("[ ] Include archived projects");
        // blank space right of the label, inside the control's row
        h.click(x + 34, y);
        let row = h.row(y);
        assert_eq!(
            row.contains("[✓] Include archived"),
            toggles,
            "{value}: {row}"
        );
        h.click(x + 6, y);
        assert_eq!(
            h.row(y).contains("[✓] Include archived"),
            !toggles,
            "{value}"
        );
    }
}

#[test]
fn checks_mixed_box_checks_when_toggled() {
    let mut h = Harness::new(120, 40, PageId::ChecksRadios);
    let (x, y) = h.at("[–] Select all rows");
    h.click(x + 5, y);
    assert!(h.row(y).contains("[✓] Select all rows"), "{}", h.row(y));
    assert!(h.text().contains("last: Select all rows checked"));
    h.opt("checksradios.mixed=off");
    assert!(h.text().contains("[ ] Select all rows"));
}

#[test]
fn checks_radio_keys_follow_the_nav_option() {
    for (value, j_moves, end_moves) in [
        ("default", false, true),
        ("vim", true, true),
        ("no-home-end", false, false),
    ] {
        let mut h = Harness::new(120, 40, PageId::ChecksRadios);
        h.opt(&format!("checksradios.radio_nav={value}"));
        let (x, y) = h.at("Read committed");
        h.click(x, y);
        h.key(KeyCode::Char('j'));
        assert_eq!(
            h.text().contains("(●) Repeatable read"),
            j_moves,
            "{value}: j"
        );
        let (x, y) = h.at("Read committed");
        h.click(x, y);
        h.key(KeyCode::End);
        assert_eq!(
            h.text().contains("(●) Serializable"),
            end_moves,
            "{value}: End"
        );
    }
}

#[test]
fn checks_disabled_option_stops_every_control() {
    let mut h = Harness::new(120, 40, PageId::ChecksRadios);
    h.opt("checksradios.disabled=on");
    let (x, y) = h.at("Auto-save drafts");
    h.click(x, y);
    assert!(h.text().contains("Auto-save drafts on"));
    let (x, y) = h.at("Serializable");
    h.click(x, y);
    assert!(h.text().contains("(●) Read committed"));
    h.opt("checksradios.disabled=off");
    let (x, y) = h.at("Auto-save drafts");
    h.click(x, y);
    assert!(h.text().contains("Auto-save drafts off"));
}

#[test]
fn checks_states_grid_shows_all_three_widgets() {
    let mut h = Harness::new(120, 40, PageId::ChecksRadios);
    h.section(Section::States);
    let text = h.text();
    for col in [
        "Check off",
        "Check on",
        "Mixed",
        "Radio group",
        "Toggle off",
        "Toggle on",
    ] {
        assert!(text.contains(col), "{col}: {text}");
    }
    let rows = h.app.pages[PageId::ChecksRadios.index()]
        .template()
        .unwrap()
        .state_rows()
        .to_vec();
    assert!(rows.contains(&junie_tui::ui::specimen::SpecimenState::Disabled));
}

// ----------------------------------------------------------------- SC10

fn sort_select(h: &Harness) -> &junie_tui::widgets::select::Select {
    h.app.pages[PageId::Chips.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<crate::pages::chips::ChipsPage>())
        .expect("chips page")
        .sort()
}

fn focus(h: &mut Harness, id: junie_tui::core::id::WidgetId) {
    h.app.shell.focus.focus(id);
    h.draw();
}

const CHIPS: junie_tui::core::id::WidgetId = junie_tui::core::id::WidgetId::of("chips");

#[test]
fn chips_digits_switch_streams_only_when_the_bar_asks() {
    for (value, switched) in [("on", true), ("off", false)] {
        let mut h = Harness::new(120, 40, PageId::Chips);
        h.opt(&format!("chipsselects.digit_toggles={value}"));
        focus(&mut h, CHIPS.sub("streams"));
        h.key(KeyCode::Char('2'));
        assert_eq!(
            h.text().contains("last action: stream worker off"),
            switched,
            "{value}"
        );
    }
    // Space switches the cursor stream either way
    let mut h = Harness::new(120, 40, PageId::Chips);
    focus(&mut h, CHIPS.sub("streams"));
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("last action: stream api off"));
}

#[test]
fn chips_overflow_marker_and_menu_follow_the_options() {
    let mut h = Harness::new(120, 40, PageId::Chips);
    h.opt("chipsselects.bar_width=40");
    let (x, y) = h.at("+3");
    h.click(x, y);
    // the menu lists the hidden chips; choosing one activates it
    let (mx, my) = h.at("country in (DE, FR)");
    h.click(mx, my);
    assert_eq!(h.status(), "Would open the editor for country in (DE, FR)");
    h.opt("chipsselects.overflow_menu=off");
    let (x, y) = h.at("+3");
    h.click(x, y);
    assert!(h.find("total > 100").is_none(), "no menu: {}", h.text());
    h.opt("chipsselects.overflow_marker=off");
    assert!(h.find("+3").is_none());
}

#[test]
fn chips_custom_keymap_rebinds_remove_add_and_clear() {
    let mut h = Harness::new(120, 40, PageId::Chips);
    h.opt("chipsselects.chip_keymap=custom");
    focus(&mut h, CHIPS.sub("filters"));
    h.key(KeyCode::Char('x'));
    assert!(h.text().contains("last action: nothing yet"));
    h.key(KeyCode::Delete);
    assert!(h.text().contains("last action: removed status = 'pending'"));
    h.key(KeyCode::Char('n'));
    assert!(h.text().contains("last action: added created_at"));
    h.key_mod(KeyCode::Delete, KeyModifiers::SHIFT);
    assert!(h.text().contains("last action: cleared all filters"));
}

#[test]
fn chips_select_arrows_step_or_open() {
    // SI13: by default ↓ opens the list and never changes the value
    let mut h = Harness::new(120, 40, PageId::Chips);
    focus(&mut h, CHIPS.sub("sort"));
    h.key(KeyCode::Down);
    assert!(sort_select(&h).open);
    assert_eq!(
        sort_select(&h).selected,
        0,
        "opening never changes the value"
    );
    h.key(KeyCode::Esc);
    h.opt("chipsselects.arrows=step");
    focus(&mut h, CHIPS.sub("sort"));
    h.key(KeyCode::Down);
    assert!(!sort_select(&h).open);
    assert_eq!(sort_select(&h).selected, 1);
}

#[test]
fn chips_select_list_rows_placement_and_tab() {
    let mut h = Harness::new(120, 40, PageId::Chips);
    h.opt("chipsselects.max_rows=4");
    focus(&mut h, CHIPS.sub("sort"));
    h.key(KeyCode::Enter);
    let text = h.text();
    assert!(text.contains("customer"), "{text}");
    assert!(!text.contains("currency"), "4 rows only: {text}");
    // Swallow keeps the list open on Tab
    h.opt("chipsselects.tab_in_open=swallow");
    focus(&mut h, CHIPS.sub("sort"));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Tab);
    assert!(sort_select(&h).open);
    // above the field
    let mut h = Harness::new(120, 40, PageId::Chips);
    let (_, field) = h.at("created_at  ");
    h.opt("chipsselects.placement=above");
    focus(&mut h, CHIPS.sub("sort"));
    h.key(KeyCode::Enter);
    let (_, first) = h.at("total");
    assert!(first < field, "the list opens above the field");
}

#[test]
fn chips_select_dismiss_and_mixed_follow_the_options() {
    // light: an outside click closes the list; escape-only (with the
    // list kept open on blur) leaves it open, and Esc closes it
    for (value, closes) in [("light", true), ("escape", false)] {
        let mut h = Harness::new(120, 40, PageId::Chips);
        h.opt(&format!("chipsselects.dismiss={value}"));
        h.opt("chipsselects.close_on_blur=off");
        focus(&mut h, CHIPS.sub("sort"));
        h.key(KeyCode::Enter);
        assert!(sort_select(&h).open);
        let (x, y) = h.at("Filters");
        h.click(x, y);
        assert_eq!(!sort_select(&h).open, closes, "{value}");
        focus(&mut h, CHIPS.sub("sort"));
        h.key(KeyCode::Esc);
        assert!(!sort_select(&h).open, "{value}: Esc closes");
    }
    let mut h = Harness::new(120, 40, PageId::Chips);
    h.opt("chipsselects.mixed=on");
    assert!(h.text().contains("Mixed"));
}

/// SC10 finding (interaction.md): under `Dismiss::ESCAPE` with the default
/// blur option, a click on empty page space still closed a select's list:
/// the refused click fell through to dead space, ended the field's edit,
/// and `close_on_blur` closed it behind the policy's back. The policy is
/// the one source: the refused click is consumed and `focus_out` decides
/// blur, so only Esc closes it.
#[test]
fn an_escape_only_select_stays_open_on_a_click_outside() {
    let mut h = Harness::new(120, 40, PageId::Chips);
    h.opt("chipsselects.dismiss=escape");
    focus(&mut h, CHIPS.sub("sort"));
    h.key(KeyCode::Enter);
    assert!(sort_select(&h).open);
    let (x, y) = h.at("Filters");
    h.click(x, y);
    assert!(sort_select(&h).open, "a refused click leaves it open");
    assert_eq!(h.app.shell.focus.current(), Some(CHIPS.sub("sort")));
    h.key(KeyCode::Esc);
    assert!(!sort_select(&h).open, "Esc closes it");
}

/// M3: a right click on a field the keyboard is not in focuses it and
/// opens the standard Edit menu above the page; the keys then drive the
/// menu (Enter chooses), and the page needs no code for it.
#[test]
fn a_right_click_on_a_field_focuses_it_and_opens_the_edit_menu() {
    let mut h = Harness::new(120, 40, PageId::Inputs);
    h.draw();
    let (x, y) = h.at("mira@example");
    let owner = input_field(&h, 2).id;
    assert_ne!(h.app.shell.focus.current(), Some(owner));
    h.mouse(MouseKind::Secondary, x + 2, y);
    h.draw();
    assert_eq!(
        h.app.shell.focus.current(),
        Some(owner),
        "the right click focused the field"
    );
    for label in ["Cut", "Copy", "Paste", "Select all"] {
        assert!(
            h.find(label).is_some(),
            "the menu shows {label}:\n{}",
            h.text()
        );
    }
    // nothing is selected and nothing copied: the cursor starts on the
    // one available command, Select all; Enter chooses it
    h.key(KeyCode::Enter);
    h.draw();
    assert!(
        h.find("Select all").is_none(),
        "choosing closed the menu:\n{}",
        h.text()
    );
    assert_eq!(
        input_field(&h, 2).field.buffer.selected_text(),
        Some("mira@example")
    );
}
