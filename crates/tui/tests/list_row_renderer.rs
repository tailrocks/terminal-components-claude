//! External non-Display consumer of List's full-row renderer.
use junie_tui::{
    App, Cx, Family, Id, ItemKey, KeyCode, List, ListState, Part, Rect, Response, Role, StateFlags,
    StylePatch, Ui, Variant,
};
use junie_tui_testing::Harness;
use ratatui_core::style::Color;
use std::cell::RefCell;

#[global_allocator]
static GLOBAL: junie_tui_testing::perf::Counting = junie_tui_testing::perf::Counting;
const ID: Id = Id::root("custom.list");
struct Step {
    key: u64,
    title: &'static str,
}
fn key(step: &Step) -> ItemKey {
    ItemKey::num(step.key)
}
fn paint(ui: &mut Ui<'_>, row: Rect, flags: StateFlags, _: ItemKey, step: &Step) {
    let style = ui
        .style(Family::LIST, Variant::DEFAULT, Part::CONTAINER, flags)
        .style;
    ui.fill(row, style);
    ui.paint_str(Rect::new(row.x, row.y, 1, 1), "|", style);
    ui.paint_str(Rect::new(row.x.saturating_add(2), row.y, 1, 1), "!", style);
    ui.paint_str(
        Rect::new(
            row.x.saturating_add(4),
            row.y,
            row.width.saturating_sub(4),
            1,
        ),
        step.title,
        style,
    );
    // A maliciously oversized request remains clipped to the shared row.
    ui.paint_str(Rect::new(40, row.y, 2, 1), "XX", style);
}
struct Page {
    state: ListState,
    items: Vec<Step>,
    seen: RefCell<Vec<(ItemKey, StateFlags)>>,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            state: ListState::default(),
            items: (0..20)
                .map(|key| Step {
                    key,
                    title: "title",
                })
                .collect(),
            seen: RefCell::new(Vec::with_capacity(20)),
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        List::new(ID)
            .render_row(&paint)
            .key(key)
            .update(cx, &mut self.state, &self.items)
            .erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        self.seen.borrow_mut().clear();
        let renderer = |ui: &mut Ui<'_>, row, flags, key, item: &Step| {
            self.seen.borrow_mut().push((key, flags));
            paint(ui, row, flags, key, item);
        };
        // Type-changing builders must preserve the installed full-row painter.
        List::new(ID)
            .render_row(&renderer)
            .key(key)
            .row(|_: &Step, out: &mut junie_tui::RowUi<'_>| out.label("UNREACHABLE"))
            .draw(ui, Rect::new(1, 1, 20, 4), &self.state, &self.items);
        let raw = ui.paint_patch(&StylePatch::new().set_fg(Role::Custom(Color::Yellow)));
        ui.paint_str(Rect::new(40, 1, 1, 1), "S", raw);
    }
}

#[test]
fn non_display_rows_use_exact_geometry_once_and_retain_shared_navigation() {
    let mut h = Harness::new(Page::default(), junie_tui::Theme::junie(), 44, 8);
    assert_eq!(h.app().seen.borrow().len(), 4);
    for y in 1..5 {
        assert_eq!(h.cell(1, y).symbol(), "|");
        assert_eq!(h.cell(2, y).symbol(), " ");
        assert_eq!(h.cell(3, y).symbol(), "!");
        assert_eq!(h.cell(5, y).symbol(), "t");
        assert_eq!(h.cell(41, y).symbol(), " ");
    }
    assert_eq!(h.cell(40, 1).symbol(), "S");
    assert!(!h.text().contains("UNREACHABLE"));
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    let chosen = h.app().state.chosen();
    h.app_mut().items.swap(0, 1);
    let _ = h.key(KeyCode::Null);
    assert_eq!(h.app().state.chosen(), chosen);
    assert!(
        h.app()
            .seen
            .borrow()
            .iter()
            .any(|(key, flags)| Some(*key) == chosen && flags.contains(StateFlags::SELECTED))
    );
    let _ = h.key(KeyCode::End);
    assert_eq!(h.app().seen.borrow().len(), 4);
    assert!(
        h.app()
            .seen
            .borrow()
            .iter()
            .any(|(key, _)| *key == ItemKey::num(19))
    );
}

#[test]
fn full_row_renderer_allocates_nothing_after_warmup() {
    let mut h = Harness::new(Page::default(), junie_tui::Theme::junie(), 44, 8);
    h.draw();
    let before = junie_tui_testing::perf::allocs();
    h.draw();
    assert_eq!(junie_tui_testing::perf::allocs() - before, 0);
}

#[test]
fn ancestor_clip_preserves_logical_alignment_and_skips_invisible_callbacks() {
    let page = Page::default();
    let seen = RefCell::new(Vec::new());
    let mut scene = junie_tui_testing::Scene::new(
        "list_clip",
        junie_tui::Theme::junie(),
        junie_tui::ColorLevel::TrueColor,
        44,
        8,
    );
    scene.draw(|ui, _| {
        ui.with_area(Rect::new(3, 2, 5, 1), |ui| {
            let renderer = |ui: &mut Ui<'_>, row, flags, key, item: &Step| {
                seen.borrow_mut().push((row, ui.full()));
                paint(ui, row, flags, key, item);
            };
            List::new(ID).render_row(&renderer).key(key).draw(
                ui,
                Rect::new(1, 1, 20, 4),
                &page.state,
                &page.items,
            );
        });
    });
    assert_eq!(
        seen.borrow().as_slice(),
        &[(Rect::new(1, 2, 19, 1), Rect::new(3, 2, 5, 1))]
    );
    assert_eq!(scene.buffer().cell((1, 2)).map(junie_tui::Cell::symbol), Some(" "));
    assert_eq!(scene.buffer().cell((3, 2)).map(junie_tui::Cell::symbol), Some("!"));
    assert_eq!(scene.buffer().cell((5, 2)).map(junie_tui::Cell::symbol), Some("t"));
    assert_eq!(scene.buffer().cell((8, 2)).map(junie_tui::Cell::symbol), Some(" "));
}
