use core::cell::Cell;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::layout::{Position, Rect};
use termrock::blocks::{MasterDetail, MasterDetailPolicy, MasterDetailState};
use termrock::composition::{self, Group};
use termrock::controls::Button;
use termrock::layout::Constraints;
use termrock::navigation::{NavList, NavListAction, NavListState};
use termrock::{
    App, ColorLevel, Cx, Id, Input, ItemKey, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Part,
    PartRef, Response, Runtime, Theme, Ui,
};

const WORKSPACE: Id = Id::root("public.nav-list.workspace");
const NAV_ID: Id = Id::root("public.nav-list.catalogue");
const DETAIL_ID: Id = Id::root("public.nav-list.detail");
const SUMMARY_ID: Id = Id::root("public.nav-list.summary");

struct CatalogueEntry {
    id: u64,
    section: &'static str,
    label: &'static str,
    draws: Cell<usize>,
}

fn key(entry: &CatalogueEntry) -> ItemKey {
    ItemKey::num(entry.id)
}

fn section(entry: &CatalogueEntry) -> &str {
    entry.section
}

fn disabled(entry: &CatalogueEntry) -> bool {
    entry.id == 12
}

fn row(entry: &CatalogueEntry, ui: &mut termrock::RowUi<'_>) {
    entry.draws.set(entry.draws.get().saturating_add(1));
    ui.label(entry.label);
}

type CatalogueList = NavList<
    'static,
    CatalogueEntry,
    fn(&CatalogueEntry) -> ItemKey,
    fn(&CatalogueEntry, &mut termrock::RowUi<'_>),
>;

fn catalogue_list() -> CatalogueList {
    let key_fn: fn(&CatalogueEntry) -> ItemKey = key;
    let row_fn: fn(&CatalogueEntry, &mut termrock::RowUi<'_>) = row;
    NavList::new(NAV_ID)
        .key(key_fn)
        .row(row_fn)
        .section(&section)
        .disabled_item(&disabled)
        .scrollable(true)
        .leave_at_boundary(true)
}

fn entry(id: u64, section: &'static str, label: &'static str) -> CatalogueEntry {
    CatalogueEntry {
        id,
        section,
        label,
        draws: Cell::new(0),
    }
}

fn initial_entries() -> Vec<CatalogueEntry> {
    vec![
        entry(11, "Packages", "Archive historical package objects"),
        entry(12, "Packages", "Inspect signed manifests"),
        entry(21, "Workspace", "Unicode notes — édit"),
        entry(22, "Workspace", "Build and publish"),
    ]
}

struct CatalogueConsumer {
    state: NavListState,
    entries: Vec<CatalogueEntry>,
    last_action: Option<NavListAction>,
    block_state: MasterDetailState,
}

impl CatalogueConsumer {
    fn new() -> Self {
        Self {
            state: NavListState::new(),
            entries: initial_entries(),
            last_action: None,
            block_state: MasterDetailState::default(),
        }
    }

    fn block(&self) -> MasterDetail<'static> {
        MasterDetail::new(WORKSPACE, NAV_ID, DETAIL_ID).policy(MasterDetailPolicy::default())
    }
}

impl App for CatalogueConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let block = self.block();
        let dismissed = block.update(cx, &mut self.block_state);
        if dismissed.is_consumed() {
            return dismissed.erase();
        }
        let response = catalogue_list().update(cx, &mut self.state, &self.entries);
        self.last_action = response.action_ref().copied();
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let nav = catalogue_list();
        let detail_button = Button::new(DETAIL_ID, "Package details");
        let summary_button = Button::new(SUMMARY_ID, "Choose an action to inspect");
        let master =
            Group::column([composition::nav_list(&nav, &self.state, &self.entries).flex(1)]);
        let detail = Group::column([composition::button(&detail_button)]);
        let summary = Group::row([composition::button(&summary_button)]);
        let block = self
            .block()
            .master(&master)
            .detail(&detail)
            .summary(&summary);

        let state_before_measure = self.state.clone();
        let draws_before_measure: Vec<_> =
            self.entries.iter().map(|item| item.draws.get()).collect();
        let _ = block
            .measure(
                ui,
                &self.block_state,
                Constraints::loose(ui.full().width, ui.full().height),
            )
            .expect("public NavList master-detail measurement fits");
        debug_assert_eq!(self.state, state_before_measure);
        debug_assert_eq!(
            self.entries
                .iter()
                .map(|item| item.draws.get())
                .collect::<Vec<_>>(),
            draws_before_measure,
            "measurement must not execute row painters"
        );
        block
            .show(ui, &self.block_state)
            .expect("public NavList master-detail composition fits");
    }
}

fn present(runtime: &mut Runtime<CatalogueConsumer>, area: Rect) {
    let prepared = runtime.prepare(area).expect("prepare consumer frame");
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    terminal
        .draw(|frame| {
            runtime
                .draw_prepared(prepared, frame)
                .expect("draw consumer frame");
        })
        .expect("present consumer frame");
    runtime
        .acknowledge_presented(prepared)
        .expect("acknowledge consumer frame");
}

fn flush(runtime: &mut Runtime<CatalogueConsumer>, area: Rect) {
    for _ in 0..16 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        if runtime.needs_present() {
            present(runtime, area);
        }
        if !runtime.needs_settle() && !runtime.needs_present() {
            return;
        }
    }
    panic!("consumer runtime did not settle");
}

fn key_input(runtime: &mut Runtime<CatalogueConsumer>, area: Rect, code: KeyCode) {
    flush(runtime, area);
    let _ = runtime
        .handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }))
        .expect("dispatch consumer key");
    present(runtime, area);
}

#[test]
fn public_nav_list_composition_keeps_live_keys_input_and_bounds() {
    let area = Rect::new(0, 0, 120, 10);
    let mut runtime = Runtime::new(CatalogueConsumer::new(), Theme::junie());
    present(&mut runtime, area);

    let nav_area = runtime
        .area_of(NAV_ID)
        .expect("composed NavList owns bounds");
    assert!(nav_area.width >= 12);
    assert!(runtime.area_of(DETAIL_ID).is_some());
    let first = runtime
        .area_of_part(NAV_ID, PartRef::item(Part::ROW, ItemKey::num(11)))
        .expect("live keyed row has a hit region");
    assert!(first.width > 0);
    assert_eq!(runtime.app().last_action, None);

    runtime.set_focus(Some(NAV_ID));
    present(&mut runtime, area);

    key_input(&mut runtime, area, KeyCode::Down);
    assert_eq!(runtime.app().state.cursor(), Some(ItemKey::num(21)));
    key_input(&mut runtime, area, KeyCode::Enter);
    assert_eq!(
        runtime.app().last_action,
        Some(NavListAction::Chose(ItemKey::num(21)))
    );

    let target = runtime
        .area_of_part(NAV_ID, PartRef::item(Part::ROW, ItemKey::num(22)))
        .expect("second section row keeps its stable key region");
    for kind in [MouseKind::Down, MouseKind::Up] {
        flush(&mut runtime, area);
        let _ = runtime
            .handle(Input::Mouse(Mouse {
                kind,
                pos: Position::new(target.x, target.y),
                mods: KeyModifiers::NONE,
            }))
            .expect("dispatch pointer row activation");
        present(&mut runtime, area);
    }
    assert_eq!(
        runtime.app().last_action,
        Some(NavListAction::Chose(ItemKey::num(22)))
    );
    assert_eq!(runtime.focus(), Some(NAV_ID));

    runtime.app_mut().entries.swap(0, 2);
    flush(&mut runtime, area);
    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Down,
            mods: KeyModifiers::NONE,
        }))
        .expect("update after stable-key reorder");
    present(&mut runtime, area);
    assert!(
        runtime
            .area_of_part(NAV_ID, PartRef::item(Part::ROW, ItemKey::num(21)))
            .is_some()
    );
    assert!(runtime.area_of(NAV_ID).is_some());

    let narrow_area = Rect::new(0, 0, 107, 10);
    present(&mut runtime, narrow_area);
    assert!(runtime.area_of(NAV_ID).is_some());
    assert!(runtime.area_of(SUMMARY_ID).is_some());
    for width in [108, 109] {
        let resized = Rect::new(0, 0, width, 10);
        present(&mut runtime, resized);
        assert!(runtime.area_of(NAV_ID).is_some());
        assert!(runtime.area_of(DETAIL_ID).is_some());
        assert!(runtime.area_of(NAV_ID).unwrap().right() <= width);
    }

    runtime.app_mut().entries.clear();
    present(&mut runtime, Rect::new(0, 0, 40, 4));
    assert!(
        runtime.area_of(NAV_ID).is_some(),
        "empty data keeps its control area"
    );
}

#[test]
fn public_nav_list_scrolls_live_unicode_rows_and_preserves_geometry() {
    let area = Rect::new(0, 0, 80, 5);
    let mut app = CatalogueConsumer::new();
    app.entries = (0..40)
        .map(|index| {
            let id = 100 + index;
            if index % 2 == 0 {
                entry(
                    id,
                    "Long catalogue",
                    "Archive historical package objects — édit",
                )
            } else {
                entry(id, "Long catalogue", "Inspect manifests")
            }
        })
        .collect();
    let mut runtime = Runtime::new(app, Theme::junie());
    present(&mut runtime, area);
    let nav = runtime.area_of(NAV_ID).expect("composed scroll area");
    assert!(nav.right() <= area.right());

    runtime.set_focus(Some(NAV_ID));
    present(&mut runtime, area);
    key_input(&mut runtime, area, KeyCode::End);
    assert_eq!(runtime.app().state.cursor(), Some(ItemKey::num(139)));
    let last_row = runtime
        .area_of_part(NAV_ID, PartRef::item(Part::ROW, ItemKey::num(139)))
        .expect("End scrolls the last stable row into the rendered viewport");
    assert!(last_row.y >= nav.y && last_row.bottom() <= nav.bottom());

    let resized = Rect::new(0, 0, 107, 4);
    flush(&mut runtime, area);
    let _ = runtime
        .handle(Input::Resize(resized.width, resized.height))
        .expect("resize reaches framework");
    present(&mut runtime, resized);
    assert!(runtime.area_of(NAV_ID).is_some());
    assert!(
        runtime
            .area_of_part(NAV_ID, PartRef::item(Part::ROW, ItemKey::num(139)))
            .is_some()
    );
}

#[test]
fn public_nav_list_draws_inside_master_detail_breakpoint_policy() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for (width, detail_visible) in [(107, false), (108, true), (109, true)] {
            let area = Rect::new(0, 0, width, 10);
            let mut runtime =
                Runtime::new(CatalogueConsumer::new(), Theme::junie().for_level(level));
            present(&mut runtime, area);
            assert_eq!(
                runtime.area_of(DETAIL_ID).is_some(),
                detail_visible,
                "level={level:?} width={width}"
            );
            assert!(runtime.area_of(NAV_ID).is_some());
            assert!(runtime.area_of(NAV_ID).unwrap().right() <= width);
        }
    }
}
