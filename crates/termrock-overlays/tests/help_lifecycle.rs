use ratatui_core::layout::Rect;
use termrock_controls::Button;
use termrock_core::id::Id;
use termrock_overlays::{HelpAction, HelpOverlay, HelpOverlayState, HelpSection};
use termrock_runtime::layer::DismissReason;
use termrock_runtime::{App, Cx, KeyCode, Response, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const HELP: Id = Id::root("help.lifecycle");
const OPEN: Id = Id::root("help.lifecycle.open");
const UNDERLAY: Id = Id::root("help.lifecycle.underlay");
const DONE: Id = HELP.part(termrock_core::id::Part::ACTIONS);
const ROWS: &[(&str, &str)] = &[("Esc", "Close")];
const SECTIONS: &[HelpSection<'static>] = &[HelpSection::plain("Work", ROWS)];

fn help() -> HelpOverlay<'static> {
    HelpOverlay::new(HELP, "test", SECTIONS)
}

#[derive(Default)]
struct Page {
    state: HelpOverlayState,
    closed: Vec<DismissReason>,
    underlay_activations: usize,
    draw_closed_help: bool,
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Button::new(OPEN, "Open help")
            .update(cx)
            .on_activated(|| cx.open_layer(HELP, help().layer(cx)));
        response |= Button::new(UNDERLAY, "Underlay action")
            .update(cx)
            .on_activated(|| self.underlay_activations += 1);
        response |= help()
            .update(cx, &mut self.state)
            .on_action(|action| match action {
                HelpAction::Closed(reason) => self.closed.push(reason),
            });
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(OPEN, "Open help").draw(ui, Rect::new(1, 1, 14, 1));
        Button::new(UNDERLAY, "Underlay action").draw(ui, Rect::new(20, 1, 18, 1));
        let drawn = ui.layer(HELP, |ui, area| {
            help().draw(ui, area, &self.state);
        });
        if drawn.is_none() && self.draw_closed_help {
            help().draw(ui, Rect::new(1, 3, 36, 14), &self.state);
        }
    }
}

fn page() -> Harness<Page> {
    Harness::new(Page::default(), Theme::junie(), 60, 20)
}

fn assert_clean(harness: &mut Harness<Page>) {
    let diagnostics = harness.diagnostics().to_vec();
    assert!(
        diagnostics.is_empty(),
        "undelivered Help intents: {diagnostics:?}"
    );
    harness.draw();
    assert_eq!(harness.diagnostics(), diagnostics);
}

#[test]
fn escape_and_outside_dismissal_drain_done_focus_and_restore_the_opener() {
    for outside in [false, true] {
        let mut harness = page();
        assert_eq!(harness.focus(), Some(OPEN));
        let _ = harness.click_id(OPEN);
        assert!(harness.is_open(HELP));
        assert_eq!(harness.focus(), Some(DONE));
        assert_clean(&mut harness);

        if outside {
            let _ = harness.click(0, 0);
        } else {
            let _ = harness.key(KeyCode::Esc);
        }

        assert!(!harness.is_open(HELP));
        assert_eq!(harness.app().closed.len(), 1);
        assert_eq!(harness.focus(), Some(OPEN));
        assert_eq!(harness.app().underlay_activations, 0);
        assert_clean(&mut harness);
        let _ = harness.click_id(OPEN);
        assert!(harness.is_open(HELP));
        assert_eq!(harness.focus(), Some(DONE));
        let _ = harness.key(KeyCode::Esc);
        assert!(!harness.is_open(HELP));
        assert_eq!(harness.app().closed.len(), 2);
        assert_eq!(harness.focus(), Some(OPEN));
        assert_clean(&mut harness);
    }
}

#[test]
fn done_enter_space_and_click_close_once_without_activating_the_underlay() {
    for activation in [KeyCode::Enter, KeyCode::Char(' ')] {
        let mut harness = page();
        let _ = harness.click_id(OPEN);
        assert_eq!(harness.focus(), Some(DONE));
        let _ = harness.key(activation);
        assert!(!harness.is_open(HELP));
        assert_eq!(
            harness.app().closed.len(),
            1,
            "{activation:?} produced {:?}",
            harness.app().closed
        );
        assert_eq!(harness.focus(), Some(OPEN));
        assert_eq!(harness.app().underlay_activations, 0);
        assert_clean(&mut harness);
        let _ = harness.click_id(OPEN);
        assert!(harness.is_open(HELP));
        assert_eq!(harness.focus(), Some(DONE));
        let _ = harness.key(KeyCode::Esc);
        assert!(!harness.is_open(HELP));
        assert_eq!(harness.app().closed.len(), 2);
        assert_eq!(harness.focus(), Some(OPEN));
        assert_clean(&mut harness);
    }

    let mut harness = page();
    let _ = harness.click_id(OPEN);
    assert_eq!(harness.focus(), Some(DONE));
    assert!(harness.area_of(DONE).is_some());
    let _ = harness.click_id(DONE);
    assert!(!harness.is_open(HELP));
    assert_eq!(harness.app().closed.len(), 1);
    assert_eq!(harness.focus(), Some(OPEN));
    assert_eq!(harness.app().underlay_activations, 0);
    assert_clean(&mut harness);
    let _ = harness.click_id(OPEN);
    assert!(harness.is_open(HELP));
    assert_eq!(harness.focus(), Some(DONE));
    let _ = harness.key(KeyCode::Esc);
    assert!(!harness.is_open(HELP));
    assert_eq!(harness.app().closed.len(), 2);
    assert_eq!(harness.focus(), Some(OPEN));
    assert_clean(&mut harness);
}

#[test]
fn question_binding_closes_open_help_when_its_real_binding_owner_has_focus() {
    let mut harness = page();
    assert_eq!(harness.focus(), Some(OPEN));
    let _ = harness.click_id(OPEN);
    assert!(harness.is_open(HELP));
    assert_eq!(harness.focus(), Some(DONE));

    assert!(harness.tab_to(HELP));
    assert_eq!(harness.focus(), Some(HELP));
    let response = harness.key(KeyCode::Char('?'));

    assert!(response.is_consumed());
    assert!(!harness.is_open(HELP));
    assert_eq!(harness.app().closed.len(), 1);
    assert_eq!(
        harness.app().closed.last(),
        Some(&DismissReason::Programmatic)
    );
    assert_eq!(harness.focus(), Some(OPEN));
    assert_eq!(harness.app().underlay_activations, 0);
    assert_clean(&mut harness);
}

#[test]
fn closed_help_draw_binding_is_published_but_does_not_emit_closed() {
    let app = Page {
        draw_closed_help: true,
        ..Page::default()
    };
    let mut harness = Harness::new(app, Theme::junie(), 60, 20);
    assert!(!harness.is_open(HELP));
    assert!(harness.tab_to(HELP));
    assert_eq!(harness.focus(), Some(HELP));

    let response = harness.key(KeyCode::Char('?'));

    assert!(!response.is_consumed());
    assert!(!harness.is_open(HELP));
    assert!(harness.app().closed.is_empty());
    assert_eq!(harness.focus(), Some(HELP));
    assert_eq!(harness.app().underlay_activations, 0);
    assert_clean(&mut harness);
}
