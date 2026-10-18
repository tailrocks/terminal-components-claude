




use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::pages::dialog_kinds::DialogKindsPage;
use crate::pages::template::Section;
use junie_tui::ui::specimen::SpecimenState;
use junie_tui::widgets::form_dialog::FormDialog;
use junie_tui::widgets::picker::Picker;

fn page(h: &Harness) -> &DialogKindsPage {
    h.app.pages[PageId::DialogKinds.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<DialogKindsPage>())
        .expect("the Dialog kinds page")
}

fn harness() -> Harness {
    Harness::on(140, 50, PageId::DialogKinds)
}

fn open(h: &mut Harness, label: &str) {
    let (x, y) = h.find(label).unwrap_or_else(|| panic!("{label} button"));
    h.click(x + 1, y);
    assert!(is_open(h), "{label} opens a layer");
}

fn form(h: &Harness) -> &FormDialog {
    h.app
        .layers
        .top_as::<FormDialog>()
        .expect("the form is on top")
}


fn is_open(h: &Harness) -> bool {
    h.app
        .layers
        .top()
        .is_some_and(|e| e.owner == Some(PageId::DialogKinds.index()))
}

fn last(h: &Harness) -> String {
    page(h).log.last().cloned().unwrap_or_default()
}

#[test]
fn the_form_previews_discloses_tests_and_saves() {
    let mut h = harness();
    open(&mut h, "Add connection…");
    assert_eq!(h.bars(), 1, "one ▎, inside the form");
    assert!(
        h.text()
            .contains("postgres://db.eu-west-1.internal/orders-prod")
    );

    assert!(h.text().contains("7731"), "{}", h.text());
    assert!(!h.text().contains("hunter2"));


    for c in "billing".chars() {
        h.key(KeyCode::Char(c));
    }
    h.key(KeyCode::Tab);
    assert!(
        h.text()
            .contains("postgres://db.eu-west-1.internal/billing"),
        "the preview follows the field\n{}",
        h.text()
    );

    assert!(!form(&h).field("key").unwrap().visible);
    let (x, y) = h.find("Connect through an SSH tunnel").unwrap();
    h.click(x, y);
    assert!(form(&h).field("key").unwrap().visible, "{}", last(&h));

    let (x, y) = h.find("Test connection").unwrap();
    h.click(x + 1, y);
    assert!(h.text().contains("Connected in 42 ms"), "{}", h.text());
    assert!(last(&h).contains("Action(\"test\")"));

    let (x, y) = h.find("Save").unwrap();
    h.click(x + 1, y);
    assert!(!is_open(&h));
    assert!(last(&h).contains("Save · billing"), "{}", last(&h));
    assert_eq!(h.app.shell.focus.current(), Some(page(&h).buttons[0].id));
}

#[test]
fn the_chooser_stacks_a_picker_over_the_form_and_returns_to_it() {
    let mut h = harness();
    open(&mut h, "Add connection…");
    let (x, y) = h.find("Connect through an SSH tunnel").unwrap();
    h.click(x, y);
    let (x, y) = h.find("Choose…").expect("the chooser shows");
    h.click(x + 1, y);
    assert_eq!(h.app.layers.len(), 2, "the picker is stacked");
    assert!(h.app.layers.top_as::<Picker>().is_some());
    assert_eq!(
        h.bars(),
        0,
        "the form under the picker shows no ▎; the picker marks its row with a fill"
    );
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.layers.len(), 1, "back on the form");
    assert!(h.text().contains("~/.ssh/prod_rsa"), "{}", h.text());

    let (x, y) = h.find("Choose…").unwrap();
    h.click(x + 1, y);
    h.key(KeyCode::Esc);
    assert_eq!(h.app.layers.len(), 1);
    assert!(h.app.layers.top_as::<FormDialog>().is_some());


    let (x, y) = h.find("Choose…").unwrap();
    h.click(x + 1, y);
    assert_eq!(h.app.layers.len(), 2);
    h.click(2, 20);
    assert_eq!(h.app.layers.len(), 1, "{}", last(&h));
    assert!(last(&h).contains("Cancelled"), "{}", last(&h));
}

#[test]
fn keep_open_on_save_leaves_closing_to_the_page() {
    let mut h = harness();
    h.opt("dialogkinds.keep_open=on");
    open(&mut h, "Add connection…");
    assert!(!form(&h).pop_on_save);
    let (x, y) = h.find("Save").unwrap();
    h.click(x + 1, y);
    assert!(!is_open(&h), "{}", last(&h));
    assert!(last(&h).contains("closed by the page"), "{}", last(&h));
}

#[test]
fn the_info_dialog_copies_a_row_and_stays_open() {
    let mut h = harness();
    open(&mut h, "Server info");
    assert!(h.text().contains("read-only"), "the meta");
    assert!(h.text().contains("wal_level = logical"), "the detail block");
    h.key(KeyCode::Char('y'));
    assert!(is_open(&h), "copy keeps it open");
    assert!(
        last(&h).contains("Copy(\"db.eu-west-1.internal\")"),
        "{}",
        last(&h)
    );
    h.key(KeyCode::Esc);
    assert!(!is_open(&h));
    assert!(last(&h).contains("Info · Closed"), "{}", last(&h));

    h.opt("dialogkinds.info_keymap=ctrl");
    open(&mut h, "Server info");
    h.key(KeyCode::Tab);
    h.key_mod(
        KeyCode::Char('y'),
        ratatui::crossterm::event::KeyModifiers::CONTROL,
    );
    assert!(last(&h).contains("Copy"), "{}", last(&h));
}





#[test]
fn a_click_outside_a_light_info_or_help_closes_it_and_a_form_stays() {
    let mut h = harness();
    for (label, closed) in [
        ("Server info", "Info · Closed"),
        ("Keyboard help", "Help · closed"),
    ] {
        open(&mut h, label);
        h.click(1, 2);
        assert!(!is_open(&h), "{label}\n{}", h.text());
        assert!(last(&h).contains(closed), "{label}: {}", last(&h));
    }
    open(&mut h, "Add connection…");
    h.click(1, 2);
    assert!(is_open(&h), "the form refuses an outside click");
}

#[test]
fn the_choice_returns_the_option_and_the_help_closes() {
    let mut h = harness();
    open(&mut h, "Release channel…");
    assert!(h.text().contains("Step 2 of 3"), "the stepper");
    h.key(KeyCode::Down);
    let (x, y) = h.find("Cancel   Next").expect("the buttons");
    h.click(x + 10, y);
    assert!(!is_open(&h), "{}", h.text());
    assert_eq!(last(&h), "Choice · Some(1)");
    open(&mut h, "Keyboard help");
    assert!(h.text().contains("Next field"));
    h.key(KeyCode::Esc);
    assert_eq!(last(&h), "Help · closed");
}

#[test]
fn the_danger_close_button_reaches_every_kind() {
    let mut h = harness();
    h.opt("dialogkinds.close_button=danger");
    for label in [
        "Add connection…",
        "Server info",
        "Release channel…",
        "Keyboard help",
    ] {
        open(&mut h, label);
        assert!(h.text().contains('✕'), "{label}: the custom ×");
        h.key(KeyCode::Esc);
        assert!(!is_open(&h), "{label}: Esc closes");
    }
}

#[test]
fn the_states_grid_draws_every_row() {
    let mut h = Harness::on(220, 80, PageId::DialogKinds);
    h.section(Section::States);
    let tpl = h.app.pages[PageId::DialogKinds.index()].template().unwrap();
    for s in [
        SpecimenState::Rest,
        SpecimenState::Hover,
        SpecimenState::Focused,
        SpecimenState::Selected,
        SpecimenState::Disabled,
    ] {
        assert!(tpl.state_rows().contains(&s), "{s:?}");
    }
    assert_eq!(h.bars(), 1, "the samples depict focus with ▏");
    assert!(h.text().contains("Release channel"));
}



struct TickingForm {
    form: FormDialog,
    ticks: std::rc::Rc<std::cell::Cell<u32>>,
}

impl junie_tui::ui::layer::Layer<crate::pages::LayerRes> for TickingForm {
    fn spec(&self) -> junie_tui::ui::layer::LayerSpec {
        junie_tui::ui::layer::Layer::<crate::pages::LayerRes>::spec(&self.form)
    }
    fn render(
        &mut self,
        screen: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        ctx: &mut junie_tui::ui::ctx::RenderCtx,
        _: &(),
    ) {
        self.form.render(screen, buf, ctx);
    }
    fn on_key(
        &mut self,
        _: &junie_tui::core::event::Key,
        _: &mut junie_tui::ui::layer::LayerCx<'_>,
    ) -> junie_tui::core::event::Outcome {
        junie_tui::core::event::Outcome::Ignored
    }
    fn on_tick(&mut self, _: &()) -> junie_tui::core::event::Outcome {
        self.ticks.set(self.ticks.get() + 1);
        junie_tui::core::event::Outcome::Changed
    }
    fn on_dismiss(&mut self, _: junie_tui::core::intent::DismissReason) {}
    fn take_result(&mut self) -> Option<junie_tui::ui::layer::LayerOut<crate::pages::LayerRes>> {
        None
    }
}




#[test]
fn a_page_hosted_form_ticks_and_esc_dismisses_through_the_shell() {
    use junie_tui::runtime::Application;
    let mut h = harness();
    let back = h.app.shell.focus.current();
    let ticks = std::rc::Rc::new(std::cell::Cell::new(0));
    let layer = TickingForm {
        form: page(&h).form(),
        ticks: ticks.clone(),
    };
    let owner = Some(PageId::DialogKinds.index());
    let app = &mut h.app;
    app.layers.open(
        Box::new(layer),
        junie_tui::ui::layer::LayerTag::new("probe"),
        owner,
        &mut app.shell.focus,
    );
    h.draw();
    h.app.handle(junie_tui::core::event::Input::Tick);
    h.app.handle(junie_tui::core::event::Input::Tick);
    assert_eq!(ticks.get(), 2, "the top layer receives the ticks");
    h.key(KeyCode::Esc);
    assert!(
        h.app.layers.is_empty(),
        "Esc dismisses it through the shell"
    );
    assert_eq!(h.app.shell.focus.current(), back, "focus comes back");
}
