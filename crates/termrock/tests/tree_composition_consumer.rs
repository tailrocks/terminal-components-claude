use termrock::composition::{self, Group};
use termrock::navigation::{
    Tree, TreeAction, TreeBranchActivation, TreeBranchClick, TreeNode, TreeState,
};
use termrock::{
    App, Buffer, ColorLevel, Cx, Id, Input, ItemKey, Key, KeyCode, KeyModifiers, Rect, Response,
    Runtime, Theme, Ui,
};

const TREE_ID: Id = Id::root("public.tree-composition.library-only");
const ENTRIES: [Entry; 4] = [
    Entry {
        id: 10,
        depth: 0,
        parent: true,
        label: "Boreal",
    },
    Entry {
        id: 11,
        depth: 1,
        parent: false,
        label: "Pine — 東京",
    },
    Entry {
        id: 12,
        depth: 1,
        parent: false,
        label: "Café · e\u{301}",
    },
    Entry {
        id: 20,
        depth: 0,
        parent: false,
        label: "Empty leaf",
    },
];

#[derive(Clone)]
struct Entry {
    id: u64,
    depth: u16,
    parent: bool,
    label: &'static str,
}

fn item_key(item: &Entry) -> ItemKey {
    ItemKey::num(item.id)
}

fn node(item: &Entry) -> TreeNode {
    if item.parent {
        TreeNode::parent(item.depth)
    } else {
        TreeNode::leaf(item.depth)
    }
}

fn row(item: &Entry, ui: &mut termrock::RowUi<'_>) {
    ui.label(item.label);
}

type EntryTree = Tree<'static, Entry, fn(&Entry) -> ItemKey, fn(&Entry, &mut termrock::RowUi<'_>)>;

fn tree() -> EntryTree {
    Tree::new(TREE_ID)
        .key(item_key as fn(&Entry) -> ItemKey)
        .row(row as fn(&Entry, &mut termrock::RowUi<'_>))
        .node(&node)
        .branch_activation(TreeBranchActivation::Activate)
        .branch_click(TreeBranchClick::Choose)
        .activate_selected_on_click(true)
}

struct TreeConsumer {
    state: TreeState,
    last_action: Option<TreeAction>,
    area: Rect,
    composed: bool,
    items: Vec<Entry>,
}

impl TreeConsumer {
    fn new(composed: bool, area: Rect) -> Self {
        Self {
            state: TreeState::default(),
            last_action: None,
            area,
            composed,
            items: ENTRIES.to_vec(),
        }
    }
}

impl App for TreeConsumer {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = tree().update(cx, &mut self.state, &self.items);
        self.last_action = response.action_ref().copied();
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let component = tree();
        if self.composed {
            let child = composition::tree(&component, &self.state, &self.items);
            let group = Group::column([child]);
            let before = self.state.clone();
            let _ = group
                .measure(
                    ui,
                    termrock::Constraints::loose(self.area.width, self.area.height),
                )
                .expect("tree leaf composition measures");
            assert_eq!(self.state, before, "measurement does not update tree state");
            group.show(ui).expect("tree leaf composition draws");
        } else {
            component.draw(ui, self.area, &self.state, &self.items);
        }
    }
}

fn publish(runtime: &mut Runtime<TreeConsumer>, area: Rect, buffer: &mut Buffer) {
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
    panic!("tree composition consumer did not settle");
}

fn key_input(runtime: &mut Runtime<TreeConsumer>, area: Rect, code: KeyCode) -> Buffer {
    let mut buffer = Buffer::empty(area);
    publish(runtime, area, &mut buffer);
    let _ = runtime
        .handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }))
        .expect("tree key input dispatches");
    publish(runtime, area, &mut buffer);
    buffer
}

fn rendered_buffer(composed: bool, area: Rect, level: ColorLevel) -> Buffer {
    let mut runtime = Runtime::new(
        TreeConsumer::new(composed, area),
        Theme::junie().for_level(level),
    );
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(area);
    publish(&mut runtime, area, &mut buffer);
    buffer
}

#[test]
fn public_tree_composition_matches_direct_renderer_and_preserves_typed_actions() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for area in [Rect::new(0, 0, 72, 10), Rect::new(0, 0, 24, 4)] {
            assert_eq!(
                rendered_buffer(true, area, level),
                rendered_buffer(false, area, level),
                "direct and composed cells differ at {level:?}, {area:?}"
            );
        }

        let area = Rect::new(0, 0, 72, 10);
        let mut direct = Runtime::new(
            TreeConsumer::new(false, area),
            Theme::junie().for_level(level),
        );
        let mut composed = Runtime::new(
            TreeConsumer::new(true, area),
            Theme::junie().for_level(level),
        );
        let _ = direct.initialize();
        let _ = composed.initialize();
        let mut direct_buffer = Buffer::empty(area);
        let mut composed_buffer = Buffer::empty(area);
        publish(&mut direct, area, &mut direct_buffer);
        publish(&mut composed, area, &mut composed_buffer);
        assert_eq!(direct.registry().regions(), composed.registry().regions());
        assert_eq!(direct.focus(), composed.focus());
        assert_eq!(direct.cursor_position(), composed.cursor_position());
        assert_eq!(direct_buffer, composed_buffer);

        let direct_frame = key_input(&mut direct, area, KeyCode::Right);
        let composed_frame = key_input(&mut composed, area, KeyCode::Right);
        assert_eq!(direct_frame, composed_frame);
        assert_eq!(direct.app().state, composed.app().state);
        assert_eq!(direct.app().last_action, composed.app().last_action);
        assert_eq!(
            direct.app().last_action,
            Some(TreeAction::Expanded(ItemKey::num(10)))
        );

        let direct_frame = key_input(&mut direct, area, KeyCode::Down);
        let composed_frame = key_input(&mut composed, area, KeyCode::Down);
        assert_eq!(direct_frame, composed_frame);
        let direct_frame = key_input(&mut direct, area, KeyCode::Enter);
        let composed_frame = key_input(&mut composed, area, KeyCode::Enter);
        assert_eq!(direct_frame, composed_frame);
        assert_eq!(direct.app().state, composed.app().state);
        assert_eq!(direct.app().last_action, composed.app().last_action);
        assert_eq!(
            direct.app().last_action,
            Some(TreeAction::Activated(ItemKey::num(11)))
        );

        direct.app_mut().items.swap(1, 2);
        composed.app_mut().items.swap(1, 2);
        let mut direct_buffer = Buffer::empty(area);
        let mut composed_buffer = Buffer::empty(area);
        publish(&mut direct, area, &mut direct_buffer);
        publish(&mut composed, area, &mut composed_buffer);
        let resized = Rect::new(0, 0, 35, 6);
        let _ = direct
            .handle(Input::Resize(resized.width, resized.height))
            .expect("direct resize dispatches");
        let _ = composed
            .handle(Input::Resize(resized.width, resized.height))
            .expect("composed resize dispatches");
        direct.app_mut().area = resized;
        composed.app_mut().area = resized;
        let mut direct_buffer = Buffer::empty(resized);
        let mut composed_buffer = Buffer::empty(resized);
        publish(&mut direct, resized, &mut direct_buffer);
        publish(&mut composed, resized, &mut composed_buffer);
        assert_eq!(direct_buffer, composed_buffer);
        assert_eq!(direct.registry().regions(), composed.registry().regions());
        assert_eq!(direct.app().state.cursor(), Some(ItemKey::num(11)));
        assert_eq!(direct.app().state, composed.app().state);
    }
}

#[test]
fn public_tree_composition_handles_zero_and_offset_roots() {
    for area in [
        Rect::new(0, 0, 0, 0),
        Rect::new(7, 3, 1, 1),
        Rect::new(3, 2, 24, 0),
    ] {
        for composed in [false, true] {
            let mut runtime = Runtime::new(
                TreeConsumer::new(composed, area),
                Theme::junie().for_level(ColorLevel::Mono),
            );
            let _ = runtime.initialize();
            let mut buffer = Buffer::empty(area);
            publish(&mut runtime, area, &mut buffer);
            assert_eq!(
                runtime.app().state.cursor(),
                Some(ItemKey::num(10)),
                "resize and drawing leave the component's reconciled stable key intact"
            );
        }
    }
}
