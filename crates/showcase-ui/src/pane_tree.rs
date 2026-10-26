use showcase_demos::pages::{Page, PageStatus, PageUpdate};
use termrock::author::{Focusability, FrameRead, LayoutFacts, with_part_hit_precedence};
use termrock::{
    ActionKey, Binding, Button, Chord, Constraints, Cx, Id, Intent, ItemKey, KeyCode, KeyModifiers,
    Maximized, Panel, PanelKind, Part, PartRef, Rect, Response, ScopeId, SplitAxis, SplitPane,
    SplitPaneState, StateFlags, Ui, Variant,
};

const ID: Id = Id::root("showcase_ui::pane_tree");
const COMMAND_OWNER: Id = ID.sub("commands");
const LEAF_OWNERS: Id = ID.sub("panes");
const SPLIT_OWNERS: Id = ID.sub("splits");
const GAP: u16 = 1;
const MIN_PANE: (u16, u16) = (12, 3);
const RESIZE_STEP: i16 = 2;
const BUTTON_COUNT: usize = 4;

const NEXT: ActionKey = ActionKey::custom("showcase.pane_tree.next");
const PREVIOUS: ActionKey = ActionKey::custom("showcase.pane_tree.previous");
const FOCUS_LEFT: ActionKey = ActionKey::custom("showcase.pane_tree.focus_left");
const FOCUS_RIGHT: ActionKey = ActionKey::custom("showcase.pane_tree.focus_right");
const FOCUS_UP: ActionKey = ActionKey::custom("showcase.pane_tree.focus_up");
const FOCUS_DOWN: ActionKey = ActionKey::custom("showcase.pane_tree.focus_down");
const RESIZE_LEFT: ActionKey = ActionKey::custom("showcase.pane_tree.resize_left");
const RESIZE_RIGHT: ActionKey = ActionKey::custom("showcase.pane_tree.resize_right");
const RESIZE_UP: ActionKey = ActionKey::custom("showcase.pane_tree.resize_up");
const RESIZE_DOWN: ActionKey = ActionKey::custom("showcase.pane_tree.resize_down");
const ZOOM_ALT: ActionKey = ActionKey::custom("showcase.pane_tree.zoom_alt");
const ZOOM_JETBRAINS: ActionKey = ActionKey::custom("showcase.pane_tree.zoom_jetbrains");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaneCommand {
    Next,
    Previous,
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    ResizeLeft,
    ResizeRight,
    ResizeUp,
    ResizeDown,
    Zoom,
}

const fn binding(
    action: ActionKey,
    chord: Chord,
    cmd: PaneCommand,
    label: &'static str,
) -> Binding<PaneCommand> {
    Binding {
        action,
        chord: Some(chord),
        cmd,
        label,
        priority: 50,
        visible: true,
    }
}

const COMMANDS: &[Binding<PaneCommand>] = &[
    binding(
        NEXT,
        Chord::key(KeyCode::F(6)),
        PaneCommand::Next,
        "Next pane",
    ),
    binding(
        PREVIOUS,
        Chord::with(KeyCode::F(6), KeyModifiers::SHIFT),
        PaneCommand::Previous,
        "Previous pane",
    ),
    binding(
        FOCUS_LEFT,
        Chord::with(KeyCode::Left, KeyModifiers::ALT),
        PaneCommand::FocusLeft,
        "Pane to the left",
    ),
    binding(
        FOCUS_RIGHT,
        Chord::with(KeyCode::Right, KeyModifiers::ALT),
        PaneCommand::FocusRight,
        "Pane to the right",
    ),
    binding(
        FOCUS_UP,
        Chord::with(KeyCode::Up, KeyModifiers::ALT),
        PaneCommand::FocusUp,
        "Pane above",
    ),
    binding(
        FOCUS_DOWN,
        Chord::with(KeyCode::Down, KeyModifiers::ALT),
        PaneCommand::FocusDown,
        "Pane below",
    ),
    binding(
        RESIZE_LEFT,
        Chord::with(KeyCode::Left, KeyModifiers::ALT.union(KeyModifiers::SHIFT)),
        PaneCommand::ResizeLeft,
        "Narrower",
    ),
    binding(
        RESIZE_RIGHT,
        Chord::with(KeyCode::Right, KeyModifiers::ALT.union(KeyModifiers::SHIFT)),
        PaneCommand::ResizeRight,
        "Wider",
    ),
    binding(
        RESIZE_UP,
        Chord::with(KeyCode::Up, KeyModifiers::ALT.union(KeyModifiers::SHIFT)),
        PaneCommand::ResizeUp,
        "Shorter",
    ),
    binding(
        RESIZE_DOWN,
        Chord::with(KeyCode::Down, KeyModifiers::ALT.union(KeyModifiers::SHIFT)),
        PaneCommand::ResizeDown,
        "Taller",
    ),
    binding(
        ZOOM_ALT,
        Chord::with(KeyCode::Char('z'), KeyModifiers::ALT),
        PaneCommand::Zoom,
        "Zoom pane",
    ),
    binding(
        ZOOM_JETBRAINS,
        Chord::with(
            KeyCode::F(12),
            KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
        ),
        PaneCommand::Zoom,
        "Zoom pane",
    ),
];

const HINTS: &[(&str, &str)] = &[
    ("F6 / Shift+F6", "Cycle panes"),
    ("Alt+arrows", "Move focus"),
    ("Alt+Shift+arrows", "Resize by two cells"),
    ("Alt+Z", "Zoom / restore"),
];

const PANES: &[(&str, &[&str])] = &[
    (
        "shell",
        &[
            "~/src/termrock $ cargo nextest run",
            "  Starting 1,842 tests across 8 crates",
            "  Summary: all checks passed",
            "~/src/termrock $ _",
        ],
    ),
    (
        "build",
        &[
            "  Compiling termrock v0.1.0",
            "  Finished dev profile in 2.14s",
            "  Waiting for file changes",
        ],
    ),
    (
        "logs",
        &[
            "16:42:01 INFO  router: page pane-tree",
            "16:42:03 INFO  split: resize by two cells",
            "16:42:07 WARN  seam at minimum",
        ],
    ),
    (
        "editor",
        &[
            "fn toggle_zoom(&mut self) {",
            "    restore_saved_ratios();",
            "}",
        ],
    ),
    ("monitor", &["CPU [|||||     23%]", "MEM [||||||||  61%]"]),
    (
        "notes",
        &["- F6 cycles panes", "- Alt+Z restores the live layout"],
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Refusal {
    ZoomedFocus,
    ZoomedResize,
    NoNeighbour,
    NothingToResize,
    AtMinimum,
    OnePane,
}

impl Refusal {
    const fn message(self) -> &'static str {
        match self {
            Self::ZoomedFocus => "Unzoom to move focus",
            Self::ZoomedResize => "Unzoom to resize",
            Self::NoNeighbour => "No pane that way",
            Self::NothingToResize => "Nothing to resize in that direction",
            Self::AtMinimum => "Cannot resize: pane at minimum size",
            Self::OnePane => "Nothing to zoom: one pane",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SplitStep {
    key: u64,
    axis: SplitAxis,
    second: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    Leaf(u64),
    Split {
        key: u64,
        axis: SplitAxis,
        state: SplitPaneState,
        first: Box<Node>,
        second: Box<Node>,
    },
}

impl Node {
    fn leaves(&self, out: &mut Vec<u64>) {
        match self {
            Self::Leaf(key) => out.push(*key),
            Self::Split { first, second, .. } => {
                first.leaves(out);
                second.leaves(out);
            }
        }
    }

    fn leaf_keys(&self) -> Vec<u64> {
        let mut keys = Vec::new();
        self.leaves(&mut keys);
        keys
    }

    fn leaf_count(&self) -> usize {
        match self {
            Self::Leaf(_) => 1,
            Self::Split { first, second, .. } => first.leaf_count() + second.leaf_count(),
        }
    }

    fn first_leaf(&self) -> u64 {
        match self {
            Self::Leaf(key) => *key,
            Self::Split { first, .. } => first.first_leaf(),
        }
    }

    fn minimum(&self) -> (u16, u16) {
        match self {
            Self::Leaf(_) => MIN_PANE,
            Self::Split {
                axis,
                first,
                second,
                ..
            } => {
                let first = first.minimum();
                let second = second.minimum();
                match axis {
                    SplitAxis::Horizontal => (
                        first.0.saturating_add(GAP).saturating_add(second.0),
                        first.1.max(second.1),
                    ),
                    SplitAxis::Vertical => (
                        first.0.max(second.0),
                        first.1.saturating_add(GAP).saturating_add(second.1),
                    ),
                }
            }
        }
    }

    fn path_to(&self, target: u64, path: &mut Vec<SplitStep>) -> bool {
        match self {
            Self::Leaf(key) => *key == target,
            Self::Split {
                key,
                axis,
                first,
                second,
                ..
            } => {
                path.push(SplitStep {
                    key: *key,
                    axis: *axis,
                    second: false,
                });
                if first.path_to(target, path) {
                    return true;
                }
                path.pop();
                path.push(SplitStep {
                    key: *key,
                    axis: *axis,
                    second: true,
                });
                if second.path_to(target, path) {
                    return true;
                }
                path.pop();
                false
            }
        }
    }

    fn split_leaf(&mut self, target: u64, fresh: u64, key: u64, axis: SplitAxis) -> bool {
        match self {
            Self::Leaf(current) if *current == target => {
                let old = Self::Leaf(*current);
                *self = Self::Split {
                    key,
                    axis,
                    state: SplitPaneState::new(50),
                    first: Box::new(old),
                    second: Box::new(Self::Leaf(fresh)),
                };
                true
            }
            Self::Leaf(_) => false,
            Self::Split { first, second, .. } => {
                first.split_leaf(target, fresh, key, axis)
                    || second.split_leaf(target, fresh, key, axis)
            }
        }
    }

    fn remove_leaf(&mut self, target: u64) -> bool {
        if let Self::Split { first, second, .. } = self {
            if matches!(&**first, Self::Leaf(key) if *key == target) {
                let sibling = std::mem::replace(&mut **second, Self::Leaf(0));
                *self = sibling;
                return true;
            }
            if matches!(&**second, Self::Leaf(key) if *key == target) {
                let sibling = std::mem::replace(&mut **first, Self::Leaf(0));
                *self = sibling;
                return true;
            }
            return first.remove_leaf(target) || second.remove_leaf(target);
        }
        false
    }

    fn split_mut(&mut self, target: u64) -> Option<&mut Self> {
        match self {
            Self::Split { key, .. } if *key == target => Some(self),
            Self::Split { first, second, .. } => {
                if let Some(found) = first.split_mut(target) {
                    return Some(found);
                }
                second.split_mut(target)
            }
            Self::Leaf(_) => None,
        }
    }

    fn collect_split_states(&self, out: &mut Vec<(u64, SplitPaneState)>) {
        if let Self::Split {
            key,
            state,
            first,
            second,
            ..
        } = self
        {
            out.push((*key, *state));
            first.collect_split_states(out);
            second.collect_split_states(out);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ZoomSnapshot {
    target: u64,
    states: Vec<(u64, SplitPaneState)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TreeModel {
    root: Node,
    selected: u64,
    next_pane: u64,
    next_split: u64,
    zoom: Option<ZoomSnapshot>,
}

impl TreeModel {
    fn new() -> Self {
        let root = Node::Split {
            key: 1,
            axis: SplitAxis::Horizontal,
            state: SplitPaneState::new(50),
            first: Box::new(Node::Leaf(1)),
            second: Box::new(Node::Split {
                key: 2,
                axis: SplitAxis::Vertical,
                state: SplitPaneState::new(50),
                first: Box::new(Node::Leaf(2)),
                second: Box::new(Node::Leaf(3)),
            }),
        };
        Self {
            root,
            selected: 1,
            next_pane: 4,
            next_split: 3,
            zoom: None,
        }
    }

    fn leaves(&self) -> Vec<u64> {
        self.root.leaf_keys()
    }

    fn split_selected(&mut self, axis: SplitAxis) -> Option<u64> {
        let pane = self.next_pane;
        let next_pane = pane.checked_add(1)?;
        let split = self.next_split;
        let next_split = split.checked_add(1)?;
        self.unzoom();
        if !self.root.split_leaf(self.selected, pane, split, axis) {
            return None;
        }
        self.next_pane = next_pane;
        self.next_split = next_split;
        self.selected = pane;
        Some(pane)
    }

    fn close(&mut self, target: u64) -> Result<(), Refusal> {
        if self.root.leaf_count() <= 1 {
            return Err(Refusal::OnePane);
        }
        if self.zoom.as_ref().is_some_and(|zoom| zoom.target == target) {
            self.unzoom();
        }
        if !self.root.remove_leaf(target) {
            return Err(Refusal::NoNeighbour);
        }
        if self.selected == target {
            self.selected = self.root.first_leaf();
        }
        Ok(())
    }

    fn toggle_zoom(&mut self) -> Result<(), Refusal> {
        if self.zoom.is_some() {
            self.unzoom();
            return Ok(());
        }
        if self.root.leaf_count() <= 1 {
            return Err(Refusal::OnePane);
        }
        let mut path = Vec::new();
        if !self.root.path_to(self.selected, &mut path) || path.is_empty() {
            return Err(Refusal::NoNeighbour);
        }
        let mut states = Vec::new();
        self.root.collect_split_states(&mut states);
        for step in &path {
            if let Some(Node::Split { state, .. }) = self.root.split_mut(step.key) {
                state.toggle_max(if step.second {
                    Maximized::Second
                } else {
                    Maximized::First
                });
            }
        }
        self.zoom = Some(ZoomSnapshot {
            target: self.selected,
            states,
        });
        Ok(())
    }

    fn unzoom(&mut self) {
        let Some(snapshot) = self.zoom.take() else {
            return;
        };
        for (key, saved) in snapshot.states {
            if let Some(Node::Split { state, .. }) = self.root.split_mut(key) {
                *state = saved;
            }
        }
    }
}

fn leaf_owner(key: u64) -> Id {
    LEAF_OWNERS.item(ItemKey::num(key))
}

fn split_owner(key: u64) -> Id {
    SPLIT_OWNERS.item(ItemKey::num(key))
}

fn content_owner(key: u64) -> Id {
    leaf_owner(key).sub("content")
}

fn focused_leaf_key(tree: &TreeModel, focused: Id) -> Option<u64> {
    tree.leaves()
        .into_iter()
        .find(|key| leaf_owner(*key) == focused)
}

fn report_layout(ui: &mut Ui<'_>, owner: Id, area: Rect) {
    ui.report_layout(
        owner,
        LayoutFacts::new(0, 0, area.height, area.width).with_logical_area(area),
    );
}

fn split_control(key: u64, axis: SplitAxis, first: &Node, second: &Node) -> SplitPane<'static> {
    let first_min = first.minimum();
    let second_min = second.minimum();
    let (min_first, min_second) = match axis {
        SplitAxis::Horizontal => (first_min.0, second_min.0),
        SplitAxis::Vertical => (first_min.1, second_min.1),
    };
    SplitPane::new(split_owner(key), axis)
        .gap(GAP)
        .min_first(min_first)
        .min_second(min_second)
        .resizable(true)
        .focusability(Focusability::ClickOnly)
        .keyboard_navigation(false)
        .hit_padding(1)
        .preserve_drag_offset(true)
}

fn update_splits(node: &mut Node, cx: &mut Cx<'_>, response: &mut Response<()>) {
    if let Node::Split {
        key,
        axis,
        state,
        first,
        second,
    } = node
    {
        let control = split_control(*key, *axis, first, second);
        *response |= control.update(cx, state).erase();
        update_splits(first, cx, response);
        update_splits(second, cx, response);
    }
}

fn clear_split_drag_state(
    node: &mut Node,
    key: u64,
    cx: &mut Cx<'_>,
    response: &mut Response<()>,
) -> bool {
    match node {
        Node::Leaf(_) => false,
        Node::Split {
            key: current,
            axis,
            state,
            first,
            second,
        } if *current == key => {
            let control = split_control(*current, *axis, first, second);
            *response |= control.clear_drag_state(state);
            if cx.capture_owner() == Some(split_owner(*current)) {
                cx.release_capture();
                *response |= Response::changed();
            }
            true
        }
        Node::Split { first, second, .. } => {
            clear_split_drag_state(first, key, cx, response)
                || clear_split_drag_state(second, key, cx, response)
        }
    }
}

fn clear_all_split_drag_states(node: &mut Node, cx: &mut Cx<'_>, response: &mut Response<()>) {
    if let Node::Split {
        key,
        axis,
        state,
        first,
        second,
    } = node
    {
        let control = split_control(*key, *axis, first, second);
        *response |= control.clear_drag_state(state);
        if cx.capture_owner() == Some(split_owner(*key)) {
            cx.release_capture();
            *response |= Response::changed();
        }
        clear_all_split_drag_states(first, cx, response);
        clear_all_split_drag_states(second, cx, response);
    }
}

fn prepare_hidden_split_drag_states(
    node: &mut Node,
    constraints: Constraints,
    cx: &mut Cx<'_>,
    response: &mut Response<()>,
) {
    if let Node::Split {
        key,
        axis,
        state,
        first,
        second,
    } = node
    {
        let control = split_control(*key, *axis, first, second);
        let (first_constraints, second_constraints) = control.pane_constraints(state, constraints);
        // The draw path omits a split seam when either resolved child is empty.
        let first_empty = first_constraints.max.0 == 0 || first_constraints.max.1 == 0;
        let second_empty = second_constraints.max.0 == 0 || second_constraints.max.1 == 0;
        if first_empty || second_empty || cx.capture_owner() != Some(split_owner(*key)) {
            *response |= control.clear_drag_state(state);
            if cx.capture_owner() == Some(split_owner(*key)) {
                cx.release_capture();
                *response |= Response::changed();
            }
        }
        prepare_hidden_split_drag_states(first, first_constraints, cx, response);
        prepare_hidden_split_drag_states(second, second_constraints, cx, response);
    }
}

fn page_tree_area(area: Rect) -> Option<Rect> {
    if area.is_empty() || area.height <= 2 {
        return None;
    }
    Some(Rect {
        y: area.y.saturating_add(2),
        height: area.height.saturating_sub(2),
        ..area
    })
}

fn nearest_pane(panes: &[(u64, Rect)], from: u64, direction: Direction) -> Option<u64> {
    let (_, source) = panes.iter().find(|(key, _)| *key == from)?;
    let (center_x, center_y) = (
        i32::from(source.x) + i32::from(source.width) / 2,
        i32::from(source.y) + i32::from(source.height) / 2,
    );
    panes
        .iter()
        .filter(|(key, area)| {
            *key != from
                && match direction {
                    Direction::Left => area.right() <= source.x,
                    Direction::Right => area.x >= source.right(),
                    Direction::Up => area.bottom() <= source.y,
                    Direction::Down => area.y >= source.bottom(),
                }
        })
        .min_by_key(|(_, area)| {
            let x = i32::from(area.x) + i32::from(area.width) / 2;
            let y = i32::from(area.y) + i32::from(area.height) / 2;
            (x - center_x).abs() + (y - center_y).abs()
        })
        .map(|(key, _)| *key)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

fn direction_layout(cx: &Cx<'_>, tree: &TreeModel) -> Vec<(u64, Rect)> {
    tree.leaves()
        .into_iter()
        .filter_map(|key| {
            cx.layout(leaf_owner(key))
                .and_then(|facts| facts.logical_area)
                .filter(|area| !area.is_empty())
                .map(|area| (key, area))
        })
        .collect()
}

fn button_id(index: usize) -> Id {
    ID.sub("buttons").index(index)
}

fn toolbar_button(index: usize, zoomed: bool) -> Button<'static> {
    let label = match index {
        0 => "Split right",
        1 => "Split below",
        2 => "Close pane",
        _ if zoomed => "Restore",
        _ => "Zoom",
    };
    Button::new(button_id(index), label).variant(Variant::SECONDARY)
}

fn paint_leaf(ui: &mut Ui<'_>, key: u64, area: Rect, selected: bool, framed: bool, zoomed: bool) {
    if area.is_empty() {
        return;
    }
    let owner = leaf_owner(key);
    report_layout(ui, owner, area);
    let focused = selected && ui.state(owner).contains(StateFlags::FOCUSED);
    let name = pane_content(key).0;
    let title = format!("{} {name}", if selected { "● selected" } else { "○ pane" });
    let meta = if zoomed { Some("zoomed") } else { None };
    if framed {
        let mut panel = Panel::new(owner)
            .kind(PanelKind::Framed)
            .title(&title)
            .focused(focused);
        if let Some(meta) = meta {
            panel = panel.meta(meta);
        }
        panel.draw(ui, area, |ui, inner| {
            report_layout(ui, content_owner(key), inner);
            paint_lines(ui, inner, pane_content(key).1);
        });
    } else {
        ui.paint_str(Rect { height: 1, ..area }, &title, ui.surface_style());
        let content = Rect {
            y: area.y.saturating_add(1),
            height: area.height.saturating_sub(1),
            ..area
        };
        report_layout(ui, content_owner(key), content);
        paint_lines(ui, content, pane_content(key).1);
    }
    ui.register_control(owner, area, Focusability::Focusable);
}

fn pane_content(key: u64) -> (&'static str, &'static [&'static str]) {
    let index = (key.saturating_sub(1) % PANES.len() as u64) as usize;
    PANES[index]
}

fn paint_lines(ui: &mut Ui<'_>, area: Rect, lines: &[&str]) {
    for (index, line) in lines.iter().enumerate() {
        let Ok(row) = u16::try_from(index) else {
            break;
        };
        if row >= area.height {
            break;
        }
        ui.paint_str(
            Rect {
                y: area.y.saturating_add(row),
                height: 1,
                ..area
            },
            line,
            ui.surface_style(),
        );
    }
}

fn draw_node(ui: &mut Ui<'_>, node: &Node, area: Rect, selected: u64, framed: bool, zoomed: bool) {
    match node {
        Node::Leaf(key) => paint_leaf(ui, *key, area, *key == selected, framed, zoomed),
        Node::Split {
            key,
            axis,
            state,
            first,
            second,
        } => {
            if area.is_empty() {
                return;
            }
            let control = split_control(*key, *axis, first, second);
            control.draw(ui, area, state, |ui, first_area, second_area| {
                draw_node(ui, first, first_area, selected, framed, zoomed);
                draw_node(ui, second, second_area, selected, framed, zoomed);
            });
        }
    }
}

#[derive(Debug)]
pub(crate) struct PaneTreePage {
    tree: TreeModel,
}

impl PaneTreePage {
    pub(crate) fn new() -> Self {
        Self {
            tree: TreeModel::new(),
        }
    }

    fn set_refusal(status: &mut Option<PageStatus>, refusal: Refusal) {
        *status = Some(PageStatus(refusal.message().to_owned()));
    }

    pub(crate) fn prepare_for_route_leave(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        clear_all_split_drag_states(&mut self.tree.root, cx, &mut response);
        response
    }

    pub(crate) fn prepare_for_presentation(
        &mut self,
        cx: &mut Cx<'_>,
        page_area: Option<Rect>,
    ) -> Response<()> {
        let mut response = Response::ignored();
        // `None` means the page or its pane tree will not be eligible next frame.
        if let Some(tree_area) = page_area.and_then(page_tree_area) {
            prepare_hidden_split_drag_states(
                &mut self.tree.root,
                Constraints::loose(tree_area.width, tree_area.height),
                cx,
                &mut response,
            );
        } else {
            clear_all_split_drag_states(&mut self.tree.root, cx, &mut response);
        }
        response
    }

    fn prepare_for_zoom(&mut self, cx: &mut Cx<'_>, response: &mut Response<()>) {
        if self.tree.zoom.is_some() || self.tree.root.leaf_count() <= 1 {
            return;
        }
        let mut path = Vec::new();
        if self.tree.root.path_to(self.tree.selected, &mut path) && !path.is_empty() {
            clear_all_split_drag_states(&mut self.tree.root, cx, response);
        }
    }

    fn toggle_zoom(&mut self, cx: &mut Cx<'_>, response: &mut Response<()>) -> Result<(), Refusal> {
        self.prepare_for_zoom(cx, response);
        self.tree.toggle_zoom()
    }

    fn prepare_for_close(&mut self, cx: &mut Cx<'_>, target: u64, response: &mut Response<()>) {
        if self.tree.root.leaf_count() <= 1 {
            return;
        }
        let mut path = Vec::new();
        if self.tree.root.path_to(target, &mut path)
            && let Some(parent) = path.last()
        {
            clear_split_drag_state(&mut self.tree.root, parent.key, cx, response);
        }
    }

    fn focus_pane(&mut self, cx: &mut Cx<'_>, key: u64, response: &mut Response<()>) {
        self.tree.selected = key;
        cx.focus(leaf_owner(key));
        *response |= Response::changed();
    }

    fn activate_button(
        &mut self,
        cx: &mut Cx<'_>,
        index: usize,
        response: &mut Response<()>,
        status: &mut Option<PageStatus>,
    ) {
        match index {
            0 | 1 => {
                let axis = if index == 0 {
                    SplitAxis::Horizontal
                } else {
                    SplitAxis::Vertical
                };
                if let Some(key) = self.tree.split_selected(axis) {
                    cx.focus(leaf_owner(key));
                    *status = Some(PageStatus(format!("Opened {}", pane_content(key).0)));
                    *response |= Response::changed();
                }
            }
            2 => {
                let selected = self.tree.selected;
                self.prepare_for_close(cx, selected, response);
                match self.tree.close(selected) {
                    Ok(()) => {
                        cx.focus(leaf_owner(self.tree.selected));
                        *response |= Response::changed();
                    }
                    Err(refusal) => Self::set_refusal(status, refusal),
                }
            }
            _ => match self.toggle_zoom(cx, response) {
                Ok(()) => {
                    cx.focus(leaf_owner(self.tree.selected));
                    *response |= Response::changed();
                }
                Err(refusal) => Self::set_refusal(status, refusal),
            },
        }
    }

    fn focus_direction(
        &mut self,
        cx: &mut Cx<'_>,
        direction: Direction,
        status: &mut Option<PageStatus>,
        response: &mut Response<()>,
    ) {
        if self.tree.zoom.is_some() {
            Self::set_refusal(status, Refusal::ZoomedFocus);
            return;
        }
        let panes = direction_layout(cx, &self.tree);
        if let Some(key) = nearest_pane(&panes, self.tree.selected, direction) {
            self.focus_pane(cx, key, response);
        } else {
            Self::set_refusal(status, Refusal::NoNeighbour);
        }
    }

    fn resize_direction(
        &mut self,
        cx: &mut Cx<'_>,
        axis: SplitAxis,
        delta: i16,
        status: &mut Option<PageStatus>,
    ) -> Response<()> {
        if self.tree.zoom.is_some() {
            Self::set_refusal(status, Refusal::ZoomedResize);
            return Response::ignored();
        }
        let mut path = Vec::new();
        if !self.tree.root.path_to(self.tree.selected, &mut path) {
            Self::set_refusal(status, Refusal::NothingToResize);
            return Response::ignored();
        }
        let Some(step) = path.iter().rev().find(|step| step.axis == axis) else {
            Self::set_refusal(status, Refusal::NothingToResize);
            return Response::ignored();
        };
        let owner = split_owner(step.key);
        let Some(area) = cx
            .layout(owner)
            .and_then(|facts| facts.logical_area)
            .filter(|area| !area.is_empty())
        else {
            Self::set_refusal(status, Refusal::NothingToResize);
            return Response::ignored();
        };
        let Some(Node::Split {
            key,
            axis,
            state,
            first,
            second,
        }) = self.tree.root.split_mut(step.key)
        else {
            Self::set_refusal(status, Refusal::NothingToResize);
            return Response::ignored();
        };
        let control = split_control(*key, *axis, first, second);
        let before_state = *state;
        let constraints = Constraints::loose(area.width, area.height);
        let (before_first, _) = control.pane_constraints(&before_state, constraints);
        let before_extent = match axis {
            SplitAxis::Horizontal => before_first.max.0,
            SplitAxis::Vertical => before_first.max.1,
        };
        let result = control.resize_by_cells(state, area, delta);
        let (after_first, _) = control.pane_constraints(state, constraints);
        let after_extent = match axis {
            SplitAxis::Horizontal => after_first.max.0,
            SplitAxis::Vertical => after_first.max.1,
        };
        let movement = i32::from(after_extent) - i32::from(before_extent);
        let moved_cells = movement.abs();
        let direction_matches = if delta > 0 {
            movement > 0
        } else {
            movement < 0
        };
        if moved_cells < i32::from(RESIZE_STEP) || !direction_matches {
            *state = before_state;
            Self::set_refusal(status, Refusal::AtMinimum);
            Response::ignored()
        } else {
            result.erase()
        }
    }

    fn run_command(
        &mut self,
        cx: &mut Cx<'_>,
        command: PaneCommand,
        response: &mut Response<()>,
        status: &mut Option<PageStatus>,
    ) {
        match command {
            PaneCommand::Next | PaneCommand::Previous => {
                if self.tree.zoom.is_some() {
                    Self::set_refusal(status, Refusal::ZoomedFocus);
                    return;
                }
                let leaves = self.tree.leaves();
                if leaves.len() <= 1 {
                    Self::set_refusal(status, Refusal::NoNeighbour);
                    return;
                }
                let current = leaves
                    .iter()
                    .position(|key| *key == self.tree.selected)
                    .unwrap_or(0);
                let next = if command == PaneCommand::Previous {
                    (current + leaves.len() - 1) % leaves.len()
                } else {
                    (current + 1) % leaves.len()
                };
                self.focus_pane(cx, leaves[next], response);
            }
            PaneCommand::FocusLeft => self.focus_direction(cx, Direction::Left, status, response),
            PaneCommand::FocusRight => self.focus_direction(cx, Direction::Right, status, response),
            PaneCommand::FocusUp => self.focus_direction(cx, Direction::Up, status, response),
            PaneCommand::FocusDown => self.focus_direction(cx, Direction::Down, status, response),
            PaneCommand::ResizeLeft => {
                *response |= self.resize_direction(cx, SplitAxis::Horizontal, -RESIZE_STEP, status)
            }
            PaneCommand::ResizeRight => {
                *response |= self.resize_direction(cx, SplitAxis::Horizontal, RESIZE_STEP, status)
            }
            PaneCommand::ResizeUp => {
                *response |= self.resize_direction(cx, SplitAxis::Vertical, -RESIZE_STEP, status)
            }
            PaneCommand::ResizeDown => {
                *response |= self.resize_direction(cx, SplitAxis::Vertical, RESIZE_STEP, status)
            }
            PaneCommand::Zoom => match self.toggle_zoom(cx, response) {
                Ok(()) => {
                    cx.focus(leaf_owner(self.tree.selected));
                    *response |= Response::changed();
                }
                Err(refusal) => Self::set_refusal(status, refusal),
            },
        }
    }

    fn sync_selected_leaf_from_focus(&mut self, cx: &Cx<'_>) -> bool {
        let focused = cx.focused_in_scope(ScopeId::new(COMMAND_OWNER));
        let Some(key) = focused.and_then(|owner| focused_leaf_key(&self.tree, owner)) else {
            return false;
        };
        if self.tree.selected == key {
            return false;
        }
        self.tree.selected = key;
        true
    }

    fn draw_toolbar(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let zoomed = self.tree.zoom.is_some();
        let gap_count = (BUTTON_COUNT - 1) as u16;
        let width = area.width.saturating_sub(gap_count);
        let base = width / BUTTON_COUNT as u16;
        let remainder = width % BUTTON_COUNT as u16;
        let mut x = area.x;
        for index in 0..BUTTON_COUNT {
            let slot_width = if (index as u16) < remainder {
                base.saturating_add(1)
            } else {
                base
            };
            toolbar_button(index, zoomed).draw(
                ui,
                Rect {
                    x,
                    width: slot_width,
                    height: 1,
                    ..area
                },
            );
            x = x.saturating_add(slot_width).saturating_add(1);
        }
    }
}

impl Page for PaneTreePage {
    fn title(&self) -> &'static str {
        "Pane tree"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let mut status = None;
        if self.sync_selected_leaf_from_focus(cx) {
            response |= Response::changed();
        }
        update_splits(&mut self.tree.root, cx, &mut response);

        for index in 0..BUTTON_COUNT {
            let button = toolbar_button(index, self.tree.zoom.is_some());
            let button_response = button.update(cx);
            let activated = button_response.activated();
            response |= button_response.erase();
            if activated {
                self.activate_button(cx, index, &mut response, &mut status);
            }
        }

        for key in self.tree.leaves() {
            let owner = leaf_owner(key);
            let clicked = cx.intents(owner).any(|intent| {
                matches!(
                    intent,
                    Intent::Pointer {
                        phase: termrock::Phase::Click,
                        ..
                    }
                )
            });
            if clicked {
                self.focus_pane(cx, key, &mut response);
            }
        }

        let command = cx.intents(COMMAND_OWNER).find_map(|intent| match intent {
            Intent::Binding(action) => Binding::command(COMMANDS, action),
            _ => None,
        });
        if let Some(command) = command {
            self.run_command(cx, command, &mut response, &mut status);
        }

        PageUpdate { response, status }
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(area, ui.surface_style());
        ui.command_scope(COMMAND_OWNER, COMMANDS, |ui| {
            if area.is_empty() {
                return;
            }
            let toolbar = Rect { height: 1, ..area };
            self.draw_toolbar(ui, toolbar);
            let hints = Rect {
                y: area.y.saturating_add(1),
                height: u16::from(area.height > 1),
                ..area
            };
            ui.paint_str(
                hints,
                "F6 cycle · Alt+arrows focus · Alt+Shift+arrows resize by 2 · Alt+Z zoom",
                ui.surface_style(),
            );
            let Some(tree_area) = page_tree_area(area) else {
                return;
            };
            let framed = self.tree.root.leaf_count() > 1 || self.tree.zoom.is_some();
            with_part_hit_precedence(ui, PartRef::of(Part::SEAM), |ui| {
                draw_node(
                    ui,
                    &self.tree.root,
                    tree_area,
                    self.tree.selected,
                    framed,
                    self.tree.zoom.is_some(),
                );
            });
        });
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        HINTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split_states(tree: &TreeModel) -> Vec<(u64, SplitPaneState)> {
        let mut states = Vec::new();
        tree.root.collect_split_states(&mut states);
        states
    }

    #[test]
    fn focus_sync_maps_only_live_leaf_owners() {
        let mut tree = TreeModel::new();
        assert_eq!(focused_leaf_key(&tree, leaf_owner(2)), Some(2));
        assert_eq!(focused_leaf_key(&tree, button_id(0)), None);
        assert_eq!(focused_leaf_key(&tree, split_owner(1)), None);
        assert!(tree.close(3).is_ok());
        assert_eq!(focused_leaf_key(&tree, leaf_owner(3)), None);
    }

    #[test]
    fn initial_tree_keeps_shell_left_of_build_and_logs() {
        let tree = TreeModel::new();
        assert_eq!(tree.leaves(), [1, 2, 3]);
        assert_eq!(tree.selected, 1);
        assert!(matches!(
            &tree.root,
            Node::Split {
                key: 1,
                axis: SplitAxis::Horizontal,
                ..
            }
        ));
        if let Node::Split { first, second, .. } = &tree.root {
            assert_eq!(**first, Node::Leaf(1));
            assert!(matches!(
                &**second,
                Node::Split {
                    key: 2,
                    axis: SplitAxis::Vertical,
                    ..
                }
            ));
        }
    }

    #[test]
    fn closing_focused_nested_leaf_selects_first_leaf_in_whole_tree() {
        let mut tree = TreeModel::new();
        tree.selected = 3;
        assert!(tree.close(3).is_ok());
        assert_eq!(tree.leaves(), [1, 2]);
        assert_eq!(tree.selected, 1);
    }

    #[test]
    fn new_leaf_and_split_keys_increase_after_close() {
        let mut tree = TreeModel::new();
        assert_eq!(tree.split_selected(SplitAxis::Horizontal), Some(4));
        assert_eq!(tree.next_pane, 5);
        assert_eq!(tree.next_split, 4);
        assert!(tree.close(4).is_ok());
        assert_eq!(tree.split_selected(SplitAxis::Vertical), Some(5));
        assert_eq!(tree.next_pane, 6);
        assert_eq!(tree.next_split, 5);
        let ids: Vec<_> = split_states(&tree)
            .into_iter()
            .map(|(key, _)| key)
            .collect();
        assert!(ids.contains(&4));
        assert!(!ids.contains(&3));
    }

    #[test]
    fn closing_a_nonselected_leaf_preserves_selection() {
        let mut tree = TreeModel::new();
        tree.selected = 3;
        assert!(tree.close(1).is_ok());
        assert_eq!(tree.leaves(), [2, 3]);
        assert_eq!(tree.selected, 3);
    }

    #[test]
    fn one_leaf_keeps_the_shared_close_and_zoom_refusal() {
        let mut tree = TreeModel {
            root: Node::Leaf(1),
            selected: 1,
            next_pane: 2,
            next_split: 1,
            zoom: None,
        };
        assert_eq!(tree.close(1), Err(Refusal::OnePane));
        assert_eq!(tree.toggle_zoom(), Err(Refusal::OnePane));
        assert_eq!(Refusal::OnePane.message(), "Nothing to zoom: one pane");
    }

    #[test]
    fn zoom_restores_every_saved_split_state() {
        let mut tree = TreeModel::new();
        if let Some(Node::Split { state, .. }) = tree.root.split_mut(1) {
            state.set_percent(37);
        }
        if let Some(Node::Split { state, .. }) = tree.root.split_mut(2) {
            state.set_percent(63);
        }
        tree.selected = 3;
        let before = split_states(&tree);
        assert!(tree.toggle_zoom().is_ok());
        assert_eq!(tree.zoom.as_ref().map(|zoom| zoom.target), Some(3));
        assert!(matches!(
            tree.root.split_mut(1),
            Some(Node::Split { state, .. }) if state.maximized() == Maximized::Second
        ));
        assert!(matches!(
            tree.root.split_mut(2),
            Some(Node::Split { state, .. }) if state.maximized() == Maximized::Second
        ));
        assert!(tree.toggle_zoom().is_ok());
        assert_eq!(split_states(&tree), before);
    }

    #[test]
    fn split_while_zoomed_restores_ratios_before_insert() {
        let mut tree = TreeModel::new();
        if let Some(Node::Split { state, .. }) = tree.root.split_mut(1) {
            state.set_percent(41);
        }
        tree.selected = 3;
        let before = split_states(&tree);
        assert!(tree.toggle_zoom().is_ok());
        assert_eq!(tree.split_selected(SplitAxis::Horizontal), Some(4));
        assert!(tree.zoom.is_none());
        let after = split_states(&tree);
        assert_eq!(
            after.iter().find(|(key, _)| *key == 1).map(|(_, s)| *s),
            before.iter().find(|(key, _)| *key == 1).map(|(_, s)| *s)
        );
        assert_eq!(tree.leaves(), [1, 2, 3, 4]);
    }

    #[test]
    fn close_zoomed_leaf_restores_surviving_ancestor_state_first() {
        let mut tree = TreeModel::new();
        if let Some(Node::Split { state, .. }) = tree.root.split_mut(1) {
            state.set_percent(44);
        }
        let before = tree.root.split_mut(1).and_then(|node| match node {
            Node::Split { state, .. } => Some(*state),
            Node::Leaf(_) => None,
        });
        tree.selected = 3;
        assert!(tree.toggle_zoom().is_ok());
        assert!(tree.close(3).is_ok());
        assert!(tree.zoom.is_none());
        assert_eq!(tree.selected, 1);
        assert_eq!(tree.leaves(), [1, 2]);
        let after = tree.root.split_mut(1).and_then(|node| match node {
            Node::Split { state, .. } => Some(*state),
            Node::Leaf(_) => None,
        });
        assert_eq!(after, before);
    }

    #[test]
    fn directional_focus_uses_wholly_sideways_candidates_and_stable_ties() {
        let panes = [
            (1, Rect::new(0, 0, 10, 10)),
            (2, Rect::new(11, 0, 10, 5)),
            (3, Rect::new(11, 6, 10, 4)),
            (4, Rect::new(5, 5, 5, 5)),
        ];
        assert_eq!(nearest_pane(&panes, 1, Direction::Right), Some(2));
        assert_eq!(nearest_pane(&panes, 2, Direction::Left), Some(1));
        assert_eq!(nearest_pane(&panes, 1, Direction::Up), None);
    }
}
