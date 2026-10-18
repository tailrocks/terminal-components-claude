use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::shell::Page as ShellPage;
use junie_tui::ui::leave::Stakes;
use junie_tui::widgets::keyhint::Hint;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::{Cx, Holla};
use crate::sim::world::World;

struct Dirty;

const FIELD: WidgetId = WidgetId::of("test.dirty");

impl ShellPage<Holla> for Dirty {
    fn on_key(&mut self, _key: &Key, _w: &mut World, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }

    fn render(
        &mut self,
        _area: Rect,
        _buf: &mut Buffer,
        ctx: &mut junie_tui::ui::ctx::RenderCtx,
        _w: &World,
    ) {
        ctx.ring.register(FIELD);
    }

    fn hints(&self, _focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        vec![]
    }

    fn crumb(&self, _w: &World) -> String {
        "Draft".into()
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(FIELD)
    }

    fn stakes(&mut self, _w: &World) -> Stakes {
        Stakes::Unsaved { changes: 1 }
    }
}

impl crate::screens::Screen for Dirty {}

fn with_dirty_page() -> H {
    let mut h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    h.app.router.push(Box::new(Dirty));
    h.app.shell.focus.set(Some(FIELD));
    h.draw();
    assert_eq!(h.app.router.depth(), 2);
    h
}

fn click_text(h: &mut H, label: &str) {
    let (x, y) = h
        .find(label)
        .unwrap_or_else(|| panic!("{label}\n{}", h.text()));
    h.click(x + 1, y);
}

#[test]
fn going_back_from_a_page_with_unsaved_edits_asks_first() {
    let mut h = with_dirty_page();
    h.key(KeyCode::Esc);
    assert_eq!(h.app.router.depth(), 2, "nothing dropped yet\n{}", h.text());
    assert!(h.text().contains("Unsaved changes"), "{}", h.text());

    h.key(KeyCode::Esc);
    assert!(h.app.modals.is_empty(), "{}", h.text());
    assert_eq!(h.app.router.depth(), 2, "{}", h.text());

    let root = h.app.crumbs.areas[0];
    h.click(root.x, root.y);
    assert!(h.text().contains("Unsaved changes"), "{}", h.text());
    click_text(&mut h, "Discard");
    assert!(h.app.modals.is_empty(), "{}", h.text());
    assert_eq!(h.app.router.depth(), 1, "{}", h.text());
}

#[test]
fn quitting_with_unsaved_page_edits_asks_first() {
    let mut h = with_dirty_page();
    h.ctrl(KeyCode::Char('q'));
    assert!(
        !h.app.shell.quit,
        "a page would lose its edit\n{}",
        h.text()
    );
    assert!(h.text().contains("Unsaved changes"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(!h.app.shell.quit, "Cancel stays\n{}", h.text());

    h.ctrl(KeyCode::Char('c'));
    assert!(!h.app.shell.quit, "{}", h.text());
    assert!(h.text().contains("Unsaved changes"), "{}", h.text());
    click_text(&mut h, "Discard");
    assert!(h.app.shell.quit, "Discard quits\n{}", h.text());
}

#[test]
fn a_clean_quit_leaves_at_once() {
    let mut h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    h.ctrl(KeyCode::Char('q'));
    assert!(h.app.shell.quit, "{}", h.text());
}
