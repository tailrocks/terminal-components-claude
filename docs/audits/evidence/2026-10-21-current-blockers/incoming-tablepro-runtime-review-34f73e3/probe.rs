use termrock::{App, Cx, Response, Ui, Rect, Color, Theme, ScrollState};
use termrock::author::PaintStyle;
use termrock_test_support::Harness;

struct ScrollPage;
impl App for ScrollPage {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> { Response::ignored() }
    fn draw(&self, ui: &mut Ui<'_>) {
        for y in 0..4u16 { for x in 0..10u16 {
            let i = y * 10 + x;
            let bg = if i < 8 { Color::Rgb(i as u8 + 1, 0, 0) } else { Color::Rgb(0, 0, 0) };
            ui.paint_str(Rect::new(x, y, 1, 1), "x", PaintStyle::new().fg(Color::Rgb(255,255,255)).bg(bg));
        }}
        let mut state = ScrollState::default();
        state.set_content(10);
        state.set_viewport(4);
        ui.scroll_edges(ui.full(), &state);
    }
}

#[test]
fn ninth_background_majority_receives_scroll_edge_fade() {
    let h = Harness::new(ScrollPage, Theme::junie(), 10, 4);
    assert_ne!(h.cell(5, 3).fg, Color::Rgb(255,255,255), "the 32-cell black majority must receive overflow fading even after eight earlier colors");
}

#[test]
fn new_connection_starts_with_blank_domain_fields() {
    let mut app = tablepro_ui::TableProApp::default();
    app.begin_connection_form();
    let h = Harness::new(app, Theme::junie(), 120, 40);
    let draft = h.app().connection_draft().expect("new form");
    assert_eq!(draft.name, "", "new connection must not clone the active connection name");
    assert_eq!(draft.database, "");
    assert_eq!(draft.user, "");
}
