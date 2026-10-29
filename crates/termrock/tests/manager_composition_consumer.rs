use termrock::author::{CompositionLeaf, Measure};
use termrock::blocks::{AppShell, MasterDetail, MasterDetailPolicy, MasterDetailState};
use termrock::composition::{self, CompositionError, Group};
use termrock::controls::Button;
use termrock::navigation::{NavList, NavListAction, NavListState};
use termrock::{
    App, Buffer, ColorLevel, Cx, Id, Input, ItemKey, Key, KeyCode, KeyModifiers, Rect, Response,
    Runtime, Theme, Ui,
};

const WORKSPACE: Id = Id::root("public.manager-composition.workspace");
const MASTER: Id = Id::root("public.manager-composition.master");
const DETAIL: Id = Id::root("public.manager-composition.detail");
const SUMMARY: Id = Id::root("public.manager-composition.summary");
const ENTRIES: [Entry; 4] = [
    Entry {
        key: 10,
        label: "Boreal workspace",
    },
    Entry {
        key: 11,
        label: "Pine — 東京",
    },
    Entry {
        key: 12,
        label: "Café · e\u{301} notes",
    },
    Entry {
        key: 20,
        label: "New workspace",
    },
];

#[derive(Clone)]
struct Entry {
    key: u64,
    label: &'static str,
}

fn item_key(item: &Entry) -> ItemKey {
    ItemKey::num(item.key)
}

fn row(item: &Entry, ui: &mut termrock::RowUi<'_>) {
    ui.label(item.label);
}

type EntryList =
    NavList<'static, Entry, fn(&Entry) -> ItemKey, fn(&Entry, &mut termrock::RowUi<'_>)>;

fn nav_list() -> EntryList {
    NavList::new(MASTER)
        .key(item_key as fn(&Entry) -> ItemKey)
        .row(row as fn(&Entry, &mut termrock::RowUi<'_>))
        .scrollable(true)
}

fn policy() -> MasterDetailPolicy {
    MasterDetailPolicy::default()
        .collapse_below(108)
        .summary_rows(2)
}

fn manager_block() -> MasterDetail<'static> {
    MasterDetail::new(WORKSPACE, MASTER, DETAIL).policy(policy())
}

#[derive(Clone)]
struct ManagerConsumer {
    nav_state: NavListState,
    block_state: MasterDetailState,
    entries: Vec<Entry>,
    last_action: Option<NavListAction>,
    composed: bool,
}

impl ManagerConsumer {
    fn new(composed: bool) -> Self {
        Self {
            nav_state: NavListState::new(),
            block_state: MasterDetailState::default(),
            entries: ENTRIES.to_vec(),
            last_action: None,
            composed,
        }
    }

    fn with_content<R>(&self, f: impl FnOnce(&MasterDetail<'_>) -> R) -> R {
        let nav = nav_list();
        let master = Group::column([composition::nav_list(&nav, &self.nav_state, &self.entries)]);
        let detail_button = Button::new(DETAIL, "Inspect workspace");
        let detail = Group::column([composition::button(&detail_button)]);
        let summary_button = Button::new(SUMMARY, "Open workspace details");
        let summary = Group::row([composition::button(&summary_button)]);
        let block = manager_block()
            .master(&master)
            .detail(&detail)
            .summary(&summary);
        f(&block)
    }
}

impl App for ManagerConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let _ = manager_block().update(cx, &mut self.block_state);
        let response = nav_list().update(cx, &mut self.nav_state, &self.entries);
        self.last_action = response.action_ref().copied();
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.with_content(|block| {
            if self.composed {
                let leaf = composition::master_detail(block, &self.block_state)
                    .expect("configured master/detail content validates");
                let body = Group::column([leaf]);
                AppShell::new()
                    .body_slot(&body)
                    .show(ui)
                    .expect("shell resolves nested master/detail composition");
            } else {
                let body = DirectMasterDetail {
                    block,
                    state: &self.block_state,
                };
                AppShell::new()
                    .body_slot(&body)
                    .show(ui)
                    .expect("shell resolves direct master/detail content");
            }
        });
    }
}

struct DirectMasterDetail<'a> {
    block: &'a MasterDetail<'a>,
    state: &'a MasterDetailState,
}

impl Measure for DirectMasterDetail<'_> {
    fn measure(&self, ui: &Ui<'_>, constraints: termrock::Constraints) -> termrock::Size {
        self.block
            .measure(ui, self.state, constraints)
            .unwrap_or_default()
    }
}

impl CompositionLeaf for DirectMasterDetail<'_> {
    fn draw(&self, ui: &mut Ui<'_>, logical: Rect) {
        ui.with_area(logical, |ui| {
            let _ = self.block.show(ui, self.state);
        });
    }
}

fn publish<T: App>(runtime: &mut Runtime<T>, area: Rect, buffer: &mut Buffer) {
    for _ in 0..16 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        if runtime.needs_present() {
            runtime.draw_buffer(area, buffer).commit_presented();
        }
        if !runtime.needs_settle() && !runtime.needs_present() {
            return;
        }
    }
    panic!("nested shell and master/detail composition did not settle");
}

fn render(
    composed: bool,
    area: Rect,
    level: ColorLevel,
) -> (ManagerConsumer, Buffer, Vec<termrock::Region>) {
    let mut runtime = Runtime::new(
        ManagerConsumer::new(composed),
        Theme::junie().for_level(level),
    );
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(area);
    publish(&mut runtime, area, &mut buffer);
    (
        runtime.app().clone(),
        buffer,
        runtime.registry().regions().to_vec(),
    )
}

struct DepthConsumer {
    wrappers: usize,
    accepted: bool,
}

impl App for DepthConsumer {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let master_button = Button::new(MASTER, "Different project domain");
        let detail_button = Button::new(DETAIL, "Inspect project");
        let master = nested_buttons(&master_button, 126);
        let detail = nested_buttons(&detail_button, 126);
        let block = manager_block().master(&master).detail(&detail);
        let state = MasterDetailState::default();
        let block = composition::master_detail(&block, &state)
            .expect("master/detail's own nested groups remain below the limit");
        let mut group = Group::column([block]);
        for _ in 0..self.wrappers {
            group = Group::column([group.into_child()]);
        }

        let result = group.measure(ui, termrock::Constraints::loose(120, 40));
        if self.accepted {
            result.expect("depth 128 is accepted");
            group
                .show(ui)
                .expect("depth 128 is drawable through the same composition resolver");
        } else {
            assert!(matches!(
                result,
                Err(CompositionError::DepthExceeded {
                    depth,
                    maximum: 128
                }) if depth == 129 + self.wrappers - 1
            ));
            assert!(matches!(
                group.show(ui),
                Err(CompositionError::DepthExceeded {
                    depth,
                    maximum: 128
                }) if depth == 129 + self.wrappers - 1
            ));
        }
    }
}

fn nested_buttons<'a>(button: &'a Button<'a>, depth: usize) -> Group<'a> {
    assert!(depth > 0);
    let mut group = Group::column([composition::button(button)]);
    for _ in 1..depth {
        group = Group::column([group.into_child()]);
    }
    group
}

fn key_input(runtime: &mut Runtime<ManagerConsumer>, area: Rect, code: KeyCode) -> Buffer {
    let mut buffer = Buffer::empty(area);
    publish(runtime, area, &mut buffer);
    let _ = runtime
        .handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }))
        .expect("consumer key input dispatches after presentation");
    publish(runtime, area, &mut buffer);
    buffer
}

#[test]
fn external_shell_master_detail_group_consumer_matches_direct_component_geometry() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for width in [120, 108, 107, 100, 80, 1, 0] {
            let area = Rect::new(5, 3, width, 12);
            let (direct_app, direct_buffer, direct_regions) = render(false, area, level);
            let (composed_app, composed_buffer, composed_regions) = render(true, area, level);
            assert_eq!(direct_buffer, composed_buffer, "{level:?}, {area:?}");
            assert_eq!(direct_regions, composed_regions, "{level:?}, {area:?}");
            assert_eq!(direct_app.nav_state, composed_app.nav_state);
            assert_eq!(direct_app.block_state, composed_app.block_state);
            assert_eq!(direct_app.last_action, composed_app.last_action);
        }

        let area = Rect::new(5, 3, 120, 12);
        let mut direct = Runtime::new(ManagerConsumer::new(false), Theme::junie().for_level(level));
        let mut composed =
            Runtime::new(ManagerConsumer::new(true), Theme::junie().for_level(level));
        let _ = direct.initialize();
        let _ = composed.initialize();
        let mut direct_buffer = Buffer::empty(area);
        let mut composed_buffer = Buffer::empty(area);
        publish(&mut direct, area, &mut direct_buffer);
        publish(&mut composed, area, &mut composed_buffer);
        assert_eq!(direct.registry().regions(), composed.registry().regions());
        assert_eq!(direct.focus(), composed.focus());
        assert_eq!(direct.cursor_position(), composed.cursor_position());
        direct.set_focus(Some(MASTER));
        composed.set_focus(Some(MASTER));

        let direct_frame = key_input(&mut direct, area, KeyCode::Down);
        let composed_frame = key_input(&mut composed, area, KeyCode::Down);
        assert_eq!(direct_frame, composed_frame);
        assert_eq!(
            composed.app().last_action,
            Some(NavListAction::Moved(ItemKey::num(11)))
        );
        let direct_frame = key_input(&mut direct, area, KeyCode::Enter);
        let composed_frame = key_input(&mut composed, area, KeyCode::Enter);
        assert_eq!(direct_frame, composed_frame);
        assert_eq!(direct.app().last_action, composed.app().last_action);
        assert_eq!(direct.app().nav_state, composed.app().nav_state);
        assert_eq!(
            composed.app().last_action,
            Some(NavListAction::Chose(ItemKey::num(11))),
            "keyboard activation returns the selected stable key"
        );

        let resized = Rect::new(5, 3, 80, 8);
        let _ = direct
            .handle(Input::Resize(resized.width, resized.height))
            .expect("direct shell accepts resize");
        let _ = composed
            .handle(Input::Resize(resized.width, resized.height))
            .expect("composed shell accepts resize");
        let mut direct_buffer = Buffer::empty(resized);
        let mut composed_buffer = Buffer::empty(resized);
        publish(&mut direct, resized, &mut direct_buffer);
        publish(&mut composed, resized, &mut composed_buffer);
        assert_eq!(direct_buffer, composed_buffer);
        assert_eq!(direct.registry().regions(), composed.registry().regions());
        assert_eq!(direct.focus(), composed.focus());
        assert_eq!(direct.cursor_position(), composed.cursor_position());
        assert_eq!(direct.app().block_state, composed.app().block_state);
    }
}

#[test]
fn mixed_group_and_master_detail_depth_counts_block_and_internal_groups() {
    for (wrappers, accepted) in [(0, true), (1, false), (2, false)] {
        let mut runtime = Runtime::new(DepthConsumer { wrappers, accepted }, Theme::junie());
        let _ = runtime.initialize();
        let area = Rect::new(0, 0, 120, 40);
        let mut buffer = Buffer::empty(area);
        publish(&mut runtime, area, &mut buffer);
    }
}

#[test]
fn master_detail_adapter_validates_required_content_slots() {
    let state = MasterDetailState::default();
    let no_content = manager_block();
    assert_eq!(
        composition::master_detail(&no_content, &state).err(),
        Some(termrock::blocks::MasterDetailError::MissingMaster)
    );
}
