//! WI-HOLLA-FOCUS-01: Holla hit-testing runs on the shared Registry.
//!
//! Contract `docs/design/interaction-contract.md` (Scroll section): the wheel
//! scrolls the topmost container under the pointer, and a modal layer takes
//! hits when the pointer is over it. Visual authority is the frozen
//! `visual-baseline` tag (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
//! The journeys below drive real Holla widgets through their real render +
//! hit paths; region order, barrier shadowing, and wheel routing must come
//! from the shared `termrock::Registry`, not an app-local copy.

use holla_ui::tui::core::focus::FocusRing;
use holla_ui::tui::core::hit::HitRegistry;
use holla_ui::tui::core::id::WidgetId;
use holla_ui::tui::theme::{ColorLevel, Theme};
use holla_ui::tui::ui::ctx::{Interaction, RenderCtx};
use holla_ui::tui::widgets::button::Button;
use holla_ui::tui::widgets::tree::{TreeNode, TreeView};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

#[test]
fn holla_click_hits_row_and_wheel_hits_container_through_shared_registry() {
    let theme = Theme::for_level(ColorLevel::TrueColor);
    let id = WidgetId::of("files");
    let mut tree = TreeView::new(
        id,
        vec![
            TreeNode::leaf("alpha"),
            TreeNode::leaf("beta"),
            TreeNode::leaf("gamma"),
        ],
    );
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut buf = Buffer::empty(Rect::new(0, 0, 40, 8));
    {
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        tree.render(Rect::new(0, 0, 40, 8), &mut buf, &mut ctx, theme.canvas);
    }
    // The second visible row is a clickable child of the container.
    let row_pos = Position::new(4, 1);
    assert_eq!(hits.hit(row_pos), Some(tree.row_id(1)));
    // The wheel targets the container beneath the row (contract), not the row.
    assert_eq!(hits.hit_scroll(row_pos, termrock::Axis::V), Some(id));
    assert_eq!(hits.hit_scroll(row_pos, termrock::Axis::H), Some(id));
    assert_eq!(hits.area_of(tree.row_id(1)).map(|r| r.y), Some(1));
}

#[test]
fn holla_modal_barrier_shadows_page_hits_and_wheel() {
    let theme = Theme::for_level(ColorLevel::TrueColor);
    let page = WidgetId::of("page.list");
    let modal_btn = WidgetId::of("quit").sub("ok");
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut buf = Buffer::empty(Rect::new(0, 0, 40, 12));
    {
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        // Page content registers first, like App::draw_frame does.
        ctx.control(page, Rect::new(0, 0, 40, 10), false);
        ctx.scrollable(page, Rect::new(0, 0, 40, 10));
        // Then the modal claims its barrier and draws its own controls.
        ctx.begin_modal();
        let mut ok = Button::primary(modal_btn, "Quit");
        ok.render(Rect::new(14, 5, 12, 1), &mut buf, &mut ctx, theme.canvas);
    }
    // Page hits and wheel are unreachable behind the barrier ...
    assert_eq!(hits.hit(Position::new(2, 2)), None);
    assert_eq!(
        hits.hit_scroll(Position::new(2, 2), termrock::Axis::V),
        None
    );
    // ... while the modal control answers on its own layer.
    let area = hits.area_of(modal_btn).expect("modal button registered");
    assert_eq!(hits.hit(Position::new(area.x + 1, area.y)), Some(modal_btn));
}

#[test]
fn holla_widget_id_bridge_is_stable_and_distinct() {
    assert_eq!(
        WidgetId::of("a.b").runtime_id(),
        WidgetId::of("a.b").runtime_id()
    );
    assert_ne!(
        WidgetId::of("a.b").runtime_id(),
        WidgetId::of("a.c").runtime_id()
    );
    assert_ne!(
        WidgetId::of("a").runtime_id(),
        WidgetId::of("a").child(0).runtime_id()
    );
}
