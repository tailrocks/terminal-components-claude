//! Showcase pages. Each page is a real consumer of the `tui-next` facade.

use std::time::Duration;

use tui_next::{
    Button, Cx, Id, ItemKey, List, ListState, Panel, PanelKind, ProgressBar, Rect, Response, RowUi,
    Select, SelectState, TextArea, TextAreaState, TextInput, TextInputState, Tree, TreeNode,
    TreeState, Ui, Variant, id, layout,
};

use crate::data::{CODE, LANGUAGES, TASKS, TREE, TaskRow};

const PANEL: Id = id!("panel");
const PRIMARY: Id = id!("primary");
const SECONDARY: Id = id!("secondary");
const DISABLED: Id = id!("disabled");
const CHECKBOX: Id = id!("checkbox");
const TOGGLE: Id = id!("toggle");
const INPUT: Id = id!("input");
const TEXTAREA: Id = id!("textarea");
const LIST: Id = id!("list");
const TASK_LIST: Id = id!("task-list");
const TREE_ID: Id = id!("tree");
const SELECT: Id = id!("select");
const PROGRESS: Id = id!("progress");

/// A page owns its durable component state while the runtime owns interaction state.
pub(crate) trait Page: Send {
    /// Stable display title.
    fn title(&self) -> &'static str;
    /// One-line explanation shown above the controls.
    fn blurb(&self) -> &'static str;
    /// Drive component updates for this page.
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()>;
    /// Paint the page into the supplied content rectangle.
    fn draw(&self, ui: &mut Ui<'_>, area: Rect);
}

/// Page families represented by the original showcase navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DemoKind {
    Overview,
    Buttons,
    Inputs,
    TextAreas,
    Forms,
    Lists,
    Trees,
    Tables,
    Editable,
    Panels,
    Sidebars,
    Dialogs,
    Progress,
    Scrolling,
    Terminal,
    Editor,
    Grid,
    Chips,
    Pickers,
    Chrome,
    Settings,
    TaskRunner,
}

impl DemoKind {
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Buttons => "Buttons",
            Self::Inputs => "Inputs",
            Self::TextAreas => "Text areas",
            Self::Forms => "Forms",
            Self::Lists => "Lists",
            Self::Trees => "Trees",
            Self::Tables => "Tables",
            Self::Editable => "Editable tables",
            Self::Panels => "Panels",
            Self::Sidebars => "Sidebars",
            Self::Dialogs => "Dialogs",
            Self::Progress => "Progress",
            Self::Scrolling => "Scrolling",
            Self::Terminal => "Terminal",
            Self::Editor => "Editor",
            Self::Grid => "Data grid",
            Self::Chips => "Chips & selects",
            Self::Pickers => "Pickers",
            Self::Chrome => "Chrome",
            Self::Settings => "Settings",
            Self::TaskRunner => "Task runner",
        }
    }

    pub(crate) const fn blurb(self) -> &'static str {
        match self {
            Self::Overview => {
                "A small, deterministic tour of the public tui-next application facade."
            }
            Self::Buttons => "Actions carry focus, hover, disabled and readiness state.",
            Self::Inputs => "Controlled text editing keeps drafts in application state.",
            Self::TextAreas => "Multiline editing and scrolling use the same phase contract.",
            Self::Forms => "Form-like composition built from public controls.",
            Self::Lists => "Keyed rows reconcile without positional application state.",
            Self::Trees => "Pre-order keyed nodes expand and collapse in place.",
            Self::Tables => "Task rows demonstrate stable identity and metadata columns.",
            Self::Editable => "Editable records keep domain values separate from focus state.",
            Self::Panels => "Cards and frames supply surfaces and clipped body areas.",
            Self::Sidebars => "Navigation is a collection with an independent cursor.",
            Self::Dialogs => "Modal ownership belongs to the runtime layer stack.",
            Self::Progress => "Determinate and animated progress survive colour downgrade.",
            Self::Scrolling => "Long content stays clipped to its viewport.",
            Self::Terminal => "Terminal output is painted through the public Ui writer.",
            Self::Editor => "Code content remains a borrowed, read-only preview here.",
            Self::Grid => "Rows and columns can be composed without leaking internals.",
            Self::Chips => "Collection controls share stable ItemKey identity.",
            Self::Pickers => "Select controls own their popup geometry and dismissal.",
            Self::Chrome => "Brand and status surfaces provide application chrome.",
            Self::Settings => "Settings are ordinary controlled choices.",
            Self::TaskRunner => "A running task advances on runtime repaint deadlines.",
        }
    }
}

/// Shared implementation used by the twenty-two named page modules.
#[derive(Debug)]
pub(crate) struct DemoPage {
    kind: DemoKind,
    input: String,
    input_state: TextInputState,
    text: String,
    text_state: TextAreaState,
    list_state: ListState,
    task_state: ListState,
    tree_state: TreeState,
    select_state: SelectState,
    checked: bool,
    toggled: bool,
    clicks: u32,
    frame: usize,
}

impl DemoPage {
    pub(crate) fn new(kind: DemoKind) -> Self {
        Self {
            kind,
            input: String::from("operator"),
            input_state: TextInputState::default(),
            text: String::from(CODE),
            text_state: TextAreaState::default(),
            list_state: ListState::default(),
            task_state: ListState::default(),
            tree_state: TreeState::new(),
            select_state: SelectState::default(),
            checked: false,
            toggled: true,
            clicks: 0,
            frame: 0,
        }
    }

    fn button(id: Id, label: &'static str, variant: Variant, disabled: bool) -> Button<'static> {
        Button::new(id, label).variant(variant).disabled(disabled)
    }

    fn language_list() -> List<
        'static,
        &'static str,
        impl Fn(&&'static str) -> ItemKey,
        impl Fn(&&'static str, &mut RowUi<'_>),
    > {
        List::new(LIST)
            .key(|value: &&'static str| ItemKey::text(value))
            .row(|value: &&'static str, row: &mut RowUi<'_>| row.label(value))
    }

    fn task_list()
    -> List<'static, TaskRow, impl Fn(&TaskRow) -> ItemKey, impl Fn(&TaskRow, &mut RowUi<'_>)> {
        List::new(TASK_LIST)
            .key(|task: &TaskRow| ItemKey::num(u64::from(task.id)))
            .row(|task: &TaskRow, row: &mut RowUi<'_>| {
                row.label(task.name);
                row.meta(task.owner);
            })
    }

    fn tree()
    -> Tree<'static, TreeNode, impl Fn(&TreeNode) -> ItemKey, impl Fn(&TreeNode, &mut RowUi<'_>)>
    {
        Tree::new(TREE_ID)
            .key(|node: &TreeNode| ItemKey::Num(u64::from(node.depth())))
            .node(&Self::tree_node)
            .row(|node: &TreeNode, row: &mut RowUi<'_>| {
                let name = if node.depth() == 0 {
                    "src / Cargo.toml"
                } else {
                    "module"
                };
                row.label(name);
            })
    }

    fn tree_node(node: &TreeNode) -> TreeNode {
        *node
    }

    fn select() -> Select<
        'static,
        &'static str,
        impl Fn(&&'static str) -> ItemKey,
        impl Fn(&&'static str, &mut RowUi<'_>),
    > {
        Select::new(SELECT)
            .key(|value: &&'static str| ItemKey::text(value))
            .row(|value: &&'static str, row: &mut RowUi<'_>| row.label(value))
            .placeholder("Choose a language")
    }

    fn update_controls(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        let button = Self::button(PRIMARY, "Primary action", Variant::PRIMARY, false).update(cx);
        if button.activated() {
            self.clicks = self.clicks.saturating_add(1);
        }
        response |= button.erase();
        response |= Self::button(SECONDARY, "Secondary action", Variant::SECONDARY, false)
            .update(cx)
            .erase();
        response |= Self::button(DISABLED, "Disabled action", Variant::SECONDARY, true)
            .update(cx)
            .erase();
        response |= tui_next::Checkbox::new(CHECKBOX, "Keep me signed in")
            .update(cx, &mut self.checked)
            .erase();
        response |= tui_next::Toggle::new(TOGGLE, "Live updates")
            .update(cx, &mut self.toggled)
            .erase();
        response |= TextInput::new(INPUT)
            .placeholder("Type a name")
            .update(cx, &mut self.input_state, &mut self.input)
            .erase();
        response |= TextArea::new(TEXTAREA, 4)
            .update(cx, &mut self.text_state, &mut self.text)
            .erase();
        response |= Self::language_list()
            .update(cx, &mut self.list_state, LANGUAGES)
            .erase();
        response |= Self::task_list()
            .update(cx, &mut self.task_state, TASKS)
            .erase();
        response |= Self::tree().update(cx, &mut self.tree_state, TREE).erase();
        response |= Self::select()
            .update(cx, &mut self.select_state, LANGUAGES)
            .erase();
        if matches!(self.kind, DemoKind::Progress | DemoKind::TaskRunner) {
            self.frame = self.frame.wrapping_add(1);
            cx.request_repaint_after(Duration::from_millis(120));
        }
        response
    }

    fn paint_text(ui: &mut Ui<'_>, area: Rect, text: &str) {
        let _ = ui.paint_str(area, text, ui.surface_style());
    }

    fn draw_content(&self, ui: &mut Ui<'_>, area: Rect) {
        let (copy, controls) = layout::split_v(area, 2);
        Self::paint_text(ui, copy, self.blurb());
        match self.kind {
            DemoKind::Overview | DemoKind::Chrome => {
                tui_next::Brand::new(PRIMARY, "SHOWCASE")
                    .tagline("tui-next public facade")
                    .draw(ui, controls);
            }
            DemoKind::Buttons
            | DemoKind::Forms
            | DemoKind::Dialogs
            | DemoKind::Editable
            | DemoKind::Grid
            | DemoKind::Settings => {
                let (first, rest) = layout::split_v(controls, 1);
                let (second, rest) = layout::split_v(rest, 1);
                let (third, fourth) = layout::split_v(rest, 1);
                Self::button(PRIMARY, "Primary action", Variant::PRIMARY, false).draw(ui, first);
                Self::button(SECONDARY, "Secondary action", Variant::SECONDARY, false)
                    .draw(ui, second);
                Self::button(DISABLED, "Disabled action", Variant::SECONDARY, true).draw(ui, third);
                tui_next::Checkbox::new(CHECKBOX, "Keep me signed in")
                    .checked(self.checked)
                    .draw(ui, fourth);
            }
            DemoKind::Inputs => {
                TextInput::new(INPUT)
                    .value(&self.input)
                    .placeholder("Type a name")
                    .draw(ui, controls, &self.input_state);
            }
            DemoKind::TextAreas | DemoKind::Editor => {
                TextArea::new(TEXTAREA, 4)
                    .value(&self.text)
                    .draw(ui, controls, &self.text_state);
            }
            DemoKind::Lists | DemoKind::Chips | DemoKind::Pickers => {
                let (list_area, select_area) =
                    layout::split_v(controls, controls.height.saturating_sub(1));
                Self::language_list().draw(ui, list_area, &self.list_state, LANGUAGES);
                Self::select().draw(ui, select_area, &self.select_state, LANGUAGES);
            }
            DemoKind::Trees => {
                Self::tree().draw(ui, controls, &self.tree_state, TREE);
            }
            DemoKind::Tables | DemoKind::TaskRunner => {
                let (tasks, progress) =
                    layout::split_v(controls, controls.height.saturating_sub(1));
                Self::task_list().draw(ui, tasks, &self.task_state, TASKS);
                ProgressBar::new(PROGRESS)
                    .label("Completion")
                    .ratio(0.35 + f64::from((self.frame % 60) as u8) / 200.0)
                    .frame(self.frame)
                    .draw(ui, progress);
            }
            DemoKind::Progress => {
                let (progress, spinner) = layout::split_v(controls, 1);
                ProgressBar::new(PROGRESS)
                    .label("Build")
                    .ratio(0.72)
                    .frame(self.frame)
                    .draw(ui, progress);
                tui_next::Spinner::new(SECONDARY)
                    .label("working")
                    .frame(self.frame)
                    .draw(ui, spinner);
            }
            DemoKind::Panels | DemoKind::Sidebars | DemoKind::Scrolling | DemoKind::Terminal => {
                Self::paint_text(ui, controls, CODE);
            }
        }
    }
}

impl Page for DemoPage {
    fn title(&self) -> &'static str {
        self.kind.title()
    }

    fn blurb(&self) -> &'static str {
        self.kind.blurb()
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.update_controls(cx)
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        Panel::new(PANEL)
            .kind(PanelKind::Framed)
            .title(self.title())
            .meta("public API")
            .draw(ui, area, |ui, body| self.draw_content(ui, body));
    }
}

/// Generate a named page wrapper while keeping the implementation shared.
macro_rules! define_page {
    ($name:ident, $kind:ident) => {
        #[derive(Debug)]
        pub(crate) struct $name {
            inner: $crate::pages::DemoPage,
        }

        impl $name {
            /// Construct the page with deterministic sample state.
            pub(crate) fn new() -> Self {
                Self {
                    inner: $crate::pages::DemoPage::new($crate::pages::DemoKind::$kind),
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::pages::Page for $name {
            fn title(&self) -> &'static str {
                self.inner.title()
            }

            fn blurb(&self) -> &'static str {
                self.inner.blurb()
            }

            fn update(&mut self, cx: &mut tui_next::Cx<'_>) -> tui_next::Response<()> {
                self.inner.update(cx)
            }

            fn draw(&self, ui: &mut tui_next::Ui<'_>, area: tui_next::Rect) {
                self.inner.draw(ui, area);
            }
        }
    };
}

pub(crate) use define_page;

pub(crate) mod buttons;
pub(crate) mod chips;
pub(crate) mod chrome;
pub(crate) mod dialogs;
pub(crate) mod editable;
pub(crate) mod editor;
pub(crate) mod forms;
pub(crate) mod grid;
pub(crate) mod inputs;
pub(crate) mod lists;
pub(crate) mod overview;
pub(crate) mod panels;
pub(crate) mod pickers;
pub(crate) mod progress;
pub(crate) mod scrolling;
pub(crate) mod settings;
pub(crate) mod sidebars;
pub(crate) mod tables;
pub(crate) mod taskrunner;
pub(crate) mod terminal;
pub(crate) mod textareas;
pub(crate) mod trees;
