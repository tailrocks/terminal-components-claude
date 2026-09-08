//! Post-resolution footer metadata keeps the focused control's real hints.
use junie_tui::{
    App, Button, Chord, Cx, Family, Hint, HintBar, HintLayer, Id, KeyCode, Part, Rect, Response,
    Role, StylePatch, Theme, Ui,
};
use junie_tui_testing::{Harness, Scene};
use ratatui_core::style::Color;

#[global_allocator]
static GLOBAL: junie_tui_testing::perf::Counting = junie_tui_testing::perf::Counting;
const BUTTON: Id = Id::root("footer.button");
const FOOTER: Id = Id::root("footer.derived");
const AREA: Rect = Rect::new(0, 2, 60, 1);
fn row(scene: &Scene, y: u16) -> String {
    (0..scene.buffer().area.width)
        .filter_map(|x| scene.buffer().cell((x, y)))
        .map(junie_tui::Cell::symbol)
        .collect()
}
fn layer(label: &'static str) -> HintLayer {
    HintLayer {
        hints: vec![Hint {
            chord: Chord::key(KeyCode::Esc),
            label,
            priority: 50,
        }],
        badge: Some("BASE"),
        status: Some("inherited".into()),
        centered: false,
    }
}
struct Page {
    screen: HintLayer,
    status: String,
    badge: String,
    centered: Option<bool>,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            screen: layer("SCREEN"),
            status: "saved".into(),
            badge: "EDIT".into(),
            centered: None,
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Button::new(BUTTON, "Run").update(cx).erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(BUTTON, "Run").draw(ui, Rect::new(0, 0, 10, 1));
        let bar = HintBar::derived(FOOTER)
            .screen(&self.screen)
            .status_text(Some(&self.status))
            .badge(Some(&self.badge));
        let bar = self.centered.map_or(bar, |centered| bar.centered(centered));
        bar.draw(ui, AREA);
        HintBar::derived(Id::root("footer.unmodified"))
            .screen(&self.screen)
            .draw(ui, Rect::new(0, 3, 60, 1));
    }
}

#[test]
fn focused_hints_keep_priority_while_borrowed_footer_metadata_remains_visible() {
    let theme = Theme::junie().override_family(Family::HINTBAR, |r| {
        r.part(Part::LABEL).base(
            StylePatch::new()
                .set_fg(Role::Custom(Color::Red))
                .set_bg(Role::Custom(Color::Blue)),
        );
        r.part(Part::BADGE)
            .base(StylePatch::new().set_fg(Role::Custom(Color::Cyan)));
    });
    let mut h = Harness::new(Page::default(), theme, 60, 5);
    assert!(h.row(2).contains("Activate"));
    assert!(!h.row(2).contains("SCREEN"));
    assert!(h.row(2).contains("EDIT"));
    assert!(h.row(2).contains("saved"));
    assert!(h.row(3).contains("Activate"));
    assert!(!h.row(3).contains("saved"));
    assert!(!h.row(3).contains("EDIT"));
    assert_eq!(h.cell(54, 2).symbol(), "s");
    assert_eq!(
        (h.cell(54, 2).fg, h.cell(54, 2).bg),
        (Color::Red, Color::Blue)
    );
    assert_eq!(h.cell(2, 2).fg, Color::Cyan);
    let baseline = h.row(3);
    h.app_mut().status = "done".into();
    h.draw();
    assert!(h.row(2).contains("done"));
    assert_eq!(
        h.row(3),
        baseline,
        "cached focused hint context was not mutated"
    );
}

#[test]
fn metadata_applies_after_layer_choice_and_can_explicitly_clear() {
    let top = layer("TOP");
    let mode = layer("MODE");
    let screen = layer("SCREEN");
    let global = layer("GLOBAL");
    for (which, label) in [(0, "TOP"), (1, "MODE"), (2, "SCREEN"), (3, "GLOBAL")] {
        let mut scene = Scene::new(
            "footer_layers",
            Theme::junie(),
            junie_tui::ColorLevel::TrueColor,
            60,
            2,
        );
        scene.draw(|ui, _| {
            let bar = HintBar::derived(FOOTER).global(&global);
            let bar = if which <= 2 { bar.screen(&screen) } else { bar };
            let bar = if which <= 1 { bar.mode(&mode) } else { bar };
            let bar = if which == 0 { bar.top(&top) } else { bar };
            bar.status_text(Some("saved"))
                .badge(Some("EDIT"))
                .draw(ui, Rect::new(0, 0, 60, 1));
            bar.status_text(None)
                .badge(None)
                .draw(ui, Rect::new(0, 1, 60, 1));
        });
        let first = row(&scene, 0);
        let second = row(&scene, 1);
        assert!(first.contains(label));
        assert!(first.contains("saved"));
        assert!(first.contains("EDIT"));
        assert!(!first.contains("inherited"));
        assert!(second.contains(label));
        assert!(!second.contains("BASE"));
        assert!(!second.contains("inherited"));
    }
    assert_eq!(top, layer("TOP"));
}

#[test]
fn no_override_cells_match_direct_bar_for_all_modes_and_both_themes() {
    let context = layer("Quit");
    for theme in [Theme::junie(), Theme::paper()] {
        for level in [
            junie_tui::ColorLevel::TrueColor,
            junie_tui::ColorLevel::Ansi256,
            junie_tui::ColorLevel::Ansi16,
            junie_tui::ColorLevel::Mono,
        ] {
            let mut direct = Scene::new("footer_direct", theme.clone(), level, 60, 1);
            direct.draw(|ui, area| {
                HintBar::new(FOOTER, &context).draw(ui, area);
            });
            let mut derived = Scene::new("footer_derived", theme.clone(), level, 60, 1);
            derived.draw(|ui, area| {
                HintBar::derived(FOOTER).screen(&context).draw(ui, area);
            });
            assert_eq!(direct.buffer(), derived.buffer());
        }
    }
}

#[test]
fn metadata_without_a_hint_context_still_renders_and_redraw_allocates_nothing() {
    let mut scene = Scene::new(
        "footer_status_only",
        Theme::junie(),
        junie_tui::ColorLevel::TrueColor,
        60,
        1,
    );
    scene.draw(|ui, area| {
        HintBar::derived(FOOTER)
            .status_text(Some("saved"))
            .badge(Some("EDIT"))
            .draw(ui, area);
    });
    let line = row(&scene, 0);
    assert!(line.contains("saved"));
    assert!(line.contains("EDIT"));
    let mut h = Harness::new(Page::default(), Theme::junie(), 60, 5);
    h.draw();
    let before = junie_tui_testing::perf::allocs();
    h.draw();
    assert_eq!(junie_tui_testing::perf::allocs() - before, 0);
}

#[test]
fn centered_override_follows_context_selection_without_mutating_cached_hints() {
    let mut h = Harness::new(Page::default(), Theme::junie(), 60, 5);
    let left = h.row(2);
    let inherited = h.row(3);
    let left_position = left.find("Activate");
    h.app_mut().centered = Some(true);
    h.draw();
    assert!(h.row(2).find("Activate") > left_position);
    assert_eq!(h.row(3), inherited);
    h.app_mut().centered = Some(false);
    h.draw();
    assert_eq!(h.row(2), left);
    assert_eq!(h.row(3), inherited);

    let mut top = layer("TOP");
    top.centered = true;
    let mut scene = Scene::new(
        "centering",
        Theme::junie(),
        junie_tui::ColorLevel::TrueColor,
        60,
        2,
    );
    scene.draw(|ui, _| {
        HintBar::derived(FOOTER)
            .top(&top)
            .draw(ui, Rect::new(0, 0, 60, 1));
        HintBar::derived(Id::root("left"))
            .top(&top)
            .centered(false)
            .draw(ui, Rect::new(0, 1, 60, 1));
    });
    assert!(row(&scene, 0).find("TOP") > row(&scene, 1).find("TOP"));
    assert!(top.centered);
}
