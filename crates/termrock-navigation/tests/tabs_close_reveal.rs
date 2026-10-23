use ratatui_core::style::Modifier;
use termrock_core::{Id, ItemKey, KeyCode, MouseKind, Part, PartRef, Response};
use termrock_navigation::{TabCloseReveal, Tabs, TabsAction, TabsState};
use termrock_runtime::rowui::RowUi;
use termrock_runtime::{App, Cx, Rect, ReferenceState, ReferenceTarget, Ui};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, GlyphRole, Role, Slot, StylePatch, Theme};

const TABS: Id = Id::root("tabs.close-reveal");

#[derive(Clone, Copy)]
struct Tab {
    id: u64,
    label: &'static str,
}

impl core::fmt::Display for Tab {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.label)
    }
}

fn tab_key(tab: &Tab) -> ItemKey {
    ItemKey::num(tab.id)
}

fn tab_row(tab: &Tab, row: &mut RowUi<'_>) {
    row.label(tab.label);
}

#[derive(Clone, Copy)]
enum ClosePaint {
    Default,
    CustomGlyph,
    Clear,
}

struct Page {
    state: TabsState,
    items: Vec<Tab>,
    close_keys: Vec<ItemKey>,
    reveal: Option<TabCloseReveal>,
    close_paint: ClosePaint,
    reference_hover: Option<ItemKey>,
    planned: bool,
    actions: Vec<TabsAction>,
}

impl Page {
    fn new(reveal: Option<TabCloseReveal>) -> Self {
        let mut state = TabsState::default();
        state.set_active(1, ItemKey::num(2));
        Self {
            state,
            items: vec![
                Tab {
                    id: 1,
                    label: "Pinned",
                },
                Tab {
                    id: 2,
                    label: "Open",
                },
                Tab {
                    id: 3,
                    label: "Idle",
                },
            ],
            close_keys: vec![ItemKey::num(2), ItemKey::num(3)],
            reveal,
            close_paint: ClosePaint::Default,
            reference_hover: None,
            planned: false,
            actions: Vec::new(),
        }
    }

    fn draw_tabs(&self, ui: &mut Ui<'_>) {
        let patches = [(
            Part::CLOSE,
            match self.close_paint {
                ClosePaint::Default => StylePatch::new(),
                ClosePaint::CustomGlyph => StylePatch::new()
                    .set_glyph(GlyphRole::Error)
                    .set_fg(Role::Danger)
                    .add(Modifier::UNDERLINED),
                ClosePaint::Clear => {
                    let mut patch = StylePatch::new()
                        .set_fg(Role::Warning)
                        .add(Modifier::UNDERLINED);
                    patch.glyph = Slot::Clear;
                    patch
                }
            },
        )];
        let area = ui.full();
        if self.planned {
            let base = Tabs::new(TABS)
                .closable(false)
                .closable_keys(&self.close_keys);
            let mut tabs = match self.reveal {
                Some(reveal) => base.close_reveal(reveal),
                None => base,
            }
            .key(tab_key);
            if !matches!(self.close_paint, ClosePaint::Default) {
                tabs = tabs.patch_part(&patches);
            }
            let layout = tabs.layout_default(area, area, &self.state, &self.items, None);
            tabs.draw_layout(ui, &layout)
                .expect("matching test layout draws");
        } else {
            let base = Tabs::new(TABS)
                .closable(false)
                .closable_keys(&self.close_keys);
            let mut tabs = match self.reveal {
                Some(reveal) => base.close_reveal(reveal),
                None => base,
            }
            .key(tab_key)
            .row(tab_row);
            if !matches!(self.close_paint, ClosePaint::Default) {
                tabs = tabs.patch_part(&patches);
            }
            tabs.draw(ui, area, &self.state, &self.items);
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let base = Tabs::new(TABS)
            .closable(false)
            .closable_keys(&self.close_keys);
        let tabs = match self.reveal {
            Some(reveal) => base.close_reveal(reveal),
            None => base,
        }
        .key(tab_key)
        .row(tab_row);
        let mut response = tabs.update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if let Some(key) = self.reference_hover {
            let target = ReferenceTarget::new(TABS, ReferenceState::HOVERED)
                .part(PartRef::item(Part::CLOSE, key));
            ui.reference(Some(target), |ui| self.draw_tabs(ui));
        } else {
            self.draw_tabs(ui);
        }
    }
}

fn page(reveal: Option<TabCloseReveal>) -> Page {
    Page::new(reveal)
}

fn harness(reveal: Option<TabCloseReveal>) -> Harness<Page> {
    Harness::new(page(reveal), Theme::junie(), 36, 3)
}

fn part_area(h: &Harness<Page>, part: Part, key: ItemKey) -> Option<Rect> {
    h.area_of_part(TABS, PartRef::item(part, key))
}

fn tab_area(h: &Harness<Page>, key: ItemKey) -> Rect {
    part_area(h, Part::TAB, key).expect("tab target is registered")
}

fn close_area(h: &Harness<Page>, key: ItemKey) -> Option<Rect> {
    part_area(h, Part::CLOSE, key)
}

fn close_position(tab: Rect) -> (u16, u16) {
    (tab.right().saturating_sub(2), tab.y)
}

#[test]
fn default_reveal_and_explicit_always_preserve_complete_render_and_actions() {
    let mut default = harness(None);
    let mut always = harness(Some(TabCloseReveal::Always));

    assert_eq!(default.buffer(), always.buffer());
    assert_eq!(
        default.runtime().registry().regions(),
        always.runtime().registry().regions()
    );
    assert_eq!(default.cursor(), always.cursor());
    assert_eq!(
        close_area(&default, ItemKey::num(2)),
        close_area(&always, ItemKey::num(2))
    );
    assert_eq!(
        close_area(&default, ItemKey::num(3)),
        close_area(&always, ItemKey::num(3))
    );

    let _ = default.key(KeyCode::Right);
    let _ = always.key(KeyCode::Right);
    assert_eq!(default.app().actions, always.app().actions);
    assert_eq!(
        default.app().actions,
        [TabsAction::Activated(ItemKey::num(3))]
    );

    let mut default_click = harness(None);
    let mut always_click = harness(Some(TabCloseReveal::Always));
    let idle = ItemKey::num(3);
    let _ = default_click.click_part(TABS, PartRef::item(Part::TAB, idle));
    let _ = always_click.click_part(TABS, PartRef::item(Part::TAB, idle));
    assert_eq!(default_click.app().actions, [TabsAction::Activated(idle)]);
    assert_eq!(always_click.app().actions, [TabsAction::Activated(idle)]);
    assert_eq!(default_click.app().state.active(), Some(idle));
    assert_eq!(always_click.app().state.active(), Some(idle));

    default.app_mut().actions.clear();
    always.app_mut().actions.clear();
    let _ = default.click_part(TABS, PartRef::item(Part::CLOSE, ItemKey::num(3)));
    let _ = always.click_part(TABS, PartRef::item(Part::CLOSE, ItemKey::num(3)));
    assert_eq!(default.app().actions, always.app().actions);
    assert_eq!(default.app().actions, [TabsAction::Close(ItemKey::num(3))]);
}

#[test]
fn hover_reveals_active_and_tab_hovered_closes_without_changing_allocations() {
    let mut h = harness(Some(TabCloseReveal::Hover));
    let always = harness(Some(TabCloseReveal::Always));
    let pinned = ItemKey::num(1);
    let active = ItemKey::num(2);
    let idle = ItemKey::num(3);
    let pinned_tab = tab_area(&h, pinned);
    let active_tab = tab_area(&h, active);
    let idle_tab = tab_area(&h, idle);
    let active_close = close_area(&h, active).expect("active eligible close remains visible");
    let idle_close_cell = close_position(idle_tab);
    let before = (
        h.app().state.active(),
        h.app().state.cursor(),
        h.app().state.first(),
    );

    assert_eq!(
        close_area(&h, pinned),
        None,
        "pinned item has no close allocation"
    );
    assert_eq!(
        close_area(&h, idle),
        None,
        "inactive close is not a live hit target"
    );
    assert_eq!(tab_area(&always, pinned), pinned_tab);
    assert_eq!(tab_area(&always, active), active_tab);
    assert_eq!(tab_area(&always, idle), idle_tab);
    assert_eq!(
        close_area(&always, idle),
        Some(Rect::new(idle_close_cell.0, idle_close_cell.1, 1, 1))
    );
    assert_eq!(h.cell(idle_close_cell.0, idle_close_cell.1).symbol(), " ");
    assert_eq!(
        h.cell(idle_close_cell.0, idle_close_cell.1).style(),
        h.cell(idle_tab.right().saturating_sub(1), idle_tab.y)
            .style(),
        "hidden close cell keeps the tab plane style"
    );

    let _ = h.mouse(MouseKind::Move, idle_tab.x.saturating_add(1), idle_tab.y);
    assert_eq!(
        close_area(&h, idle),
        Some(Rect::new(idle_close_cell.0, idle_close_cell.1, 1, 1))
    );
    assert_ne!(h.cell(idle_close_cell.0, idle_close_cell.1).symbol(), " ");
    assert_eq!(
        tab_area(&h, idle),
        idle_tab,
        "reveal keeps reserved geometry stable"
    );
    assert_eq!(close_area(&h, active), Some(active_close));
    assert_eq!(
        (
            h.app().state.active(),
            h.app().state.cursor(),
            h.app().state.first()
        ),
        before
    );

    let _ = h.mouse(MouseKind::Move, 35, 2);
    assert_eq!(
        close_area(&h, idle),
        None,
        "moving away hides only the close hit"
    );
    assert_eq!(h.cell(idle_close_cell.0, idle_close_cell.1).symbol(), " ");
    assert_eq!(
        h.cell(idle_close_cell.0, idle_close_cell.1).style(),
        h.cell(idle_tab.right().saturating_sub(1), idle_tab.y)
            .style(),
        "hidden cell does not retain close hover style"
    );
    assert_eq!(tab_area(&h, idle), idle_tab);
    assert_eq!(
        (
            h.app().state.active(),
            h.app().state.cursor(),
            h.app().state.first()
        ),
        before
    );
}

#[test]
fn hidden_close_cell_keeps_tab_activation_and_allows_immediate_close() {
    let mut h = harness(Some(TabCloseReveal::Hover));
    let idle = ItemKey::num(3);
    let tab = tab_area(&h, idle);
    let (x, y) = close_position(tab);

    assert_eq!(close_area(&h, idle), None);
    let _ = h.mouse(MouseKind::Down, x, y);
    assert_eq!(h.app().state.active(), Some(ItemKey::num(2)));
    assert_eq!(
        close_area(&h, idle),
        None,
        "newly focused close cannot replace the pressed tab target"
    );
    let _ = h.mouse(MouseKind::Up, x, y);
    assert_eq!(h.app().state.cursor(), Some(idle));
    assert_eq!(h.app().state.active(), Some(idle));
    assert_eq!(h.app().actions, [TabsAction::Activated(idle)]);
    assert_eq!(h.app().items.len(), 3, "tab activation never removes items");
    assert_eq!(h.focus(), Some(TABS));
    assert!(h.diagnostics().is_empty());
    assert!(
        close_area(&h, idle).is_some(),
        "activation reveals close immediately after release"
    );

    h.app_mut().actions.clear();
    let _ = h.click_part(TABS, PartRef::item(Part::CLOSE, idle));
    assert_eq!(h.app().actions, [TabsAction::Close(idle)]);
}

#[test]
fn hover_close_paints_while_tab_press_defers_its_hit_in_legacy_and_planned_draws() {
    let idle = ItemKey::num(3);
    let glyph = Theme::junie().design.glyphs.get(GlyphRole::Error);

    for planned in [false, true] {
        let mut h = harness(Some(TabCloseReveal::Hover));
        h.app_mut().close_paint = ClosePaint::CustomGlyph;
        let tab = tab_area(&h, idle);
        let (x, y) = close_position(tab);

        assert_eq!(close_area(&h, idle), None);
        let _ = h.mouse(MouseKind::Down, x, y);
        assert_eq!(h.app().state.cursor(), Some(idle));
        assert_eq!(h.app().state.active(), Some(ItemKey::num(2)));
        h.app_mut().planned = planned;
        h.draw();

        assert_eq!(
            h.cell(x, y).symbol(),
            glyph,
            "hover reveal paints the close"
        );
        assert!(
            h.cell(x, y).modifier.contains(Modifier::UNDERLINED),
            "the close uses its configured local style during TAB capture"
        );
        assert_eq!(close_area(&h, idle), None, "TAB retains the live target");

        let _ = h.mouse(MouseKind::Up, x, y);
        assert_eq!(
            h.app().actions,
            [TabsAction::Activated(idle)],
            "planned={planned}"
        );
        assert_eq!(close_area(&h, idle), Some(Rect::new(x, y, 1, 1)));
        h.app_mut().actions.clear();
        let _ = h.click_part(TABS, PartRef::item(Part::CLOSE, idle));
        assert_eq!(h.app().actions, [TabsAction::Close(idle)]);
    }

    let mut cleared = harness(Some(TabCloseReveal::Hover));
    cleared.app_mut().close_paint = ClosePaint::Clear;
    let tab = tab_area(&cleared, idle);
    let (x, y) = close_position(tab);
    let _ = cleared.mouse(MouseKind::Down, x, y);
    cleared.app_mut().planned = true;
    cleared.draw();
    assert_eq!(cleared.cell(x, y).symbol(), " ");
    assert!(cleared.cell(x, y).modifier.contains(Modifier::UNDERLINED));
    assert_eq!(close_area(&cleared, idle), None);

    let _ = cleared.mouse(MouseKind::Up, x, y);
    assert_eq!(cleared.app().actions, [TabsAction::Activated(idle)]);
    assert_eq!(close_area(&cleared, idle), Some(Rect::new(x, y, 1, 1)));
}

#[test]
fn inert_reference_keeps_hover_close_paint_during_a_real_tab_press() {
    let idle = ItemKey::num(3);
    let glyph = Theme::junie().design.glyphs.get(GlyphRole::Error);
    let mut h = harness(Some(TabCloseReveal::Hover));
    h.app_mut().close_paint = ClosePaint::CustomGlyph;
    let tab = tab_area(&h, idle);
    let (x, y) = close_position(tab);

    let _ = h.mouse(MouseKind::Down, x, y);
    h.app_mut().planned = true;
    h.app_mut().reference_hover = Some(idle);
    h.draw();
    assert_eq!(h.cell(x, y).symbol(), glyph);
    assert!(h.cell(x, y).modifier.contains(Modifier::UNDERLINED));
    assert_eq!(
        close_area(&h, idle),
        None,
        "reference adds no live close hit"
    );
    h.app_mut().reference_hover = None;
    h.draw();
    let _ = h.mouse(MouseKind::Up, 35, 2);
    assert!(
        h.app().actions.is_empty(),
        "outside release never activates"
    );
}

#[test]
fn always_draw_ignores_hover_press_deferral_before_update() {
    let idle = ItemKey::num(3);
    let glyph = Theme::junie().design.glyphs.get(GlyphRole::Error);

    for planned in [false, true] {
        for reveal in [None, Some(TabCloseReveal::Always)] {
            let mut h = harness(Some(TabCloseReveal::Hover));
            h.app_mut().close_paint = ClosePaint::CustomGlyph;
            let tab = tab_area(&h, idle);
            let (x, y) = close_position(tab);

            let _ = h.mouse(MouseKind::Down, x, y);
            assert_eq!(close_area(&h, idle), None);

            h.app_mut().reveal = reveal;
            h.app_mut().planned = planned;
            h.draw();

            let mut control = harness(Some(TabCloseReveal::Hover));
            control.app_mut().close_paint = ClosePaint::CustomGlyph;
            let control_tab = tab_area(&control, idle);
            assert_eq!(control_tab, tab);
            let _ = control.mouse(MouseKind::Down, x, y);
            assert_eq!(close_area(&control, idle), None);
            control.app_mut().reveal = reveal;
            control.app_mut().planned = planned;
            let _ = control.tick();
            let control_close = close_area(&control, idle).expect("clean Always close is live");
            assert_eq!(tab_area(&h, idle), control_tab);
            assert_eq!(close_area(&h, idle), Some(control_close));
            assert_eq!(h.cell(x, y).symbol(), glyph);
            assert_eq!(h.cell(x, y).symbol(), control.cell(x, y).symbol());
            assert_eq!(h.cell(x, y).style(), control.cell(x, y).style());

            let _ = h.mouse(MouseKind::Up, x, y);
            let _ = control.mouse(MouseKind::Up, x, y);
            assert!(
                h.app().actions.is_empty(),
                "a configuration change during a TAB press never closes the tab"
            );
            assert!(control.app().actions.is_empty());
            assert_eq!(h.app().state.active(), Some(ItemKey::num(2)));
            assert_eq!(h.app().state.cursor(), Some(idle));
            assert_eq!(close_area(&h, idle), Some(control_close));

            let _ = h.click_part(TABS, PartRef::item(Part::CLOSE, idle));
            assert_eq!(h.app().actions, [TabsAction::Close(idle)]);
        }
    }
}

#[test]
fn pressed_close_alone_keeps_capture_visible_until_release() {
    let mut h = harness(Some(TabCloseReveal::Hover));
    let active = ItemKey::num(2);
    let idle = ItemKey::num(3);
    let idle_tab = tab_area(&h, idle);
    let (x, y) = close_position(idle_tab);

    assert_eq!(h.app().state.active(), Some(active));
    assert_eq!(h.app().state.cursor(), Some(active));
    assert_eq!(close_area(&h, idle), None);

    let _ = h.mouse(MouseKind::Move, x, y);
    assert!(close_area(&h, idle).is_some(), "hover reveals the close");
    assert_eq!(h.app().state.active(), Some(active));
    assert_eq!(h.app().state.cursor(), Some(active));

    let _ = h.mouse(MouseKind::Down, x, y);
    let _ = h.mouse(MouseKind::Drag, 35, 2);
    assert_eq!(h.app().state.active(), Some(active));
    assert_eq!(h.app().state.cursor(), Some(active));
    assert!(
        close_area(&h, idle).is_some(),
        "physical close press alone retains the captured affordance"
    );
    assert_ne!(h.cell(x, y).symbol(), " ");

    let _ = h.mouse(MouseKind::Up, 35, 2);
    assert!(h.app().actions.is_empty(), "release outside never closes");
    assert_eq!(close_area(&h, idle), None);
    assert_eq!(h.cell(x, y).symbol(), " ");

    let _ = h.mouse(MouseKind::Move, x, y);
    let _ = h.mouse(MouseKind::Down, x, y);
    let _ = h.mouse(MouseKind::Up, x, y);
    assert_eq!(h.app().actions, [TabsAction::Close(idle)]);
}

#[test]
fn tab_press_does_not_suppress_a_close_that_was_already_live() {
    let mut h = harness(Some(TabCloseReveal::Hover));
    let active = ItemKey::num(2);
    let tab = tab_area(&h, active);
    let close = close_area(&h, active).expect("active close is already live");
    let body_x = tab.x.saturating_add(1);

    assert_ne!(body_x, close.x);
    let _ = h.mouse(MouseKind::Down, body_x, tab.y);
    assert_eq!(close_area(&h, active), Some(close));
    assert_ne!(h.cell(close.x, close.y).symbol(), " ");
    let _ = h.mouse(MouseKind::Up, body_x, tab.y);
    assert_eq!(h.app().state.active(), Some(active));
}

#[test]
fn focused_inactive_cursor_and_held_close_remain_revealed_during_capture() {
    let mut h = harness(Some(TabCloseReveal::Hover));
    let idle = ItemKey::num(3);
    let tab = tab_area(&h, idle);
    assert_eq!(h.focus(), Some(TABS));

    let _ = h.mouse(MouseKind::Down, tab.x.saturating_add(1), tab.y);
    let _ = h.mouse(MouseKind::Drag, 35, 2);
    let _ = h.mouse(MouseKind::Up, 35, 2);
    assert_eq!(h.app().state.active(), Some(ItemKey::num(2)));
    assert_eq!(h.app().state.cursor(), Some(idle));
    assert!(
        close_area(&h, idle).is_some(),
        "focused cursor row reveals while inactive"
    );
    assert!(
        h.app().actions.is_empty(),
        "dragging off the row does not activate it"
    );

    let close = close_area(&h, idle).expect("focused row close is live");
    let (x, y) = (close.x, close.y);
    let _ = h.mouse(MouseKind::Down, x, y);
    let _ = h.mouse(MouseKind::Drag, 35, 2);
    assert!(
        close_area(&h, idle).is_some(),
        "captured close remains revealed outside"
    );
    assert!(h.cell(x, y).symbol() != " ");
    let _ = h.mouse(MouseKind::Up, 35, 2);
    assert!(
        !h.app()
            .actions
            .iter()
            .any(|a| matches!(a, TabsAction::Close(_)))
    );
}

#[test]
fn stale_ineligible_close_release_is_revalidated_and_removed_from_registry() {
    let mut h = harness(Some(TabCloseReveal::Hover));
    let idle = ItemKey::num(3);
    let tab = tab_area(&h, idle);
    let _ = h.mouse(MouseKind::Move, tab.x.saturating_add(1), tab.y);
    let close = close_area(&h, idle).expect("hovered close is registered");
    let (x, y) = (close.x, close.y);
    let _ = h.mouse(MouseKind::Down, x, y);

    h.app_mut().close_keys.retain(|key| *key != idle);
    let _ = h.mouse(MouseKind::Up, x, y);
    assert!(
        !h.app()
            .actions
            .iter()
            .any(|a| matches!(a, TabsAction::Close(_)))
    );
    assert_eq!(
        close_area(&h, idle),
        None,
        "newly ineligible item has no stale target"
    );
}

#[test]
fn custom_glyph_and_clear_styles_work_in_live_and_inert_hover_frames() {
    for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
        let theme = Theme::junie().downgrade(level);
        let error_glyph = theme.design.glyphs.get(GlyphRole::Error);
        let mut custom = Page::new(Some(TabCloseReveal::Hover));
        custom.close_paint = ClosePaint::CustomGlyph;
        let mut h = Harness::new(custom, theme, 36, 3);
        let active_close = close_area(&h, ItemKey::num(2)).expect("active close is visible");
        let custom_cell = h.cell(active_close.x, active_close.y);
        assert_eq!(custom_cell.symbol(), error_glyph);
        assert!(custom_cell.modifier.contains(Modifier::UNDERLINED));

        let idle_tab = tab_area(&h, ItemKey::num(3));
        let idle_close = close_position(idle_tab);
        h.app_mut().reference_hover = Some(ItemKey::num(3));
        let _ = h.tick();
        let (x, y) = idle_close;
        assert_eq!(h.cell(x, y).symbol(), error_glyph);
        assert_eq!(
            close_area(&h, ItemKey::num(3)),
            None,
            "inert reference draw registers no hit"
        );
    }

    let mut cleared = Page::new(Some(TabCloseReveal::Hover));
    cleared.close_paint = ClosePaint::Clear;
    let h = Harness::new(cleared, Theme::junie().downgrade(ColorLevel::Mono), 36, 3);
    let active_close = close_area(&h, ItemKey::num(2)).expect("clear-styled close remains live");
    let cell = h.cell(active_close.x, active_close.y);
    assert_eq!(cell.symbol(), " ");
    assert!(cell.modifier.contains(Modifier::UNDERLINED));
}
