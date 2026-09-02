//! Route screens and the contract they share with the shell: input,
//! requests back to the app, modal ownership, rendering and hints.

pub mod accounts;
pub mod capsule;
pub mod cockpit;
pub mod config;
pub mod editor;
pub mod manager;
pub mod modals;
pub mod prelude;
pub mod settings;
pub mod usage;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::keyhint::Hint;
use junie_tui::widgets::picker::{Picker, PickerItem};
use junie_tui::widgets::segments::Segment;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::domain::account::AccountId;
use crate::domain::agent::Agent;
use crate::domain::instance::InstanceId;
use crate::domain::onepassword::OpReference;
use crate::domain::workspace::{Workspace, WorkspaceId};
use crate::sim::launch::LaunchPlan;
use crate::sim::world::{Msg, World};
use modals::{
    BrowserResult, ChoiceDialog, FileBrowser, FormDialog, FormValues, HelpOverlay, InfoDialog,
    InfoResult, OpFlow,
};

/// Identifies a modal's purpose and target for its owning screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalTag {
    pub kind: &'static str,
    pub key: String,
    pub n: usize,
}

impl ModalTag {
    pub fn new(kind: &'static str) -> Self {
        Self {
            kind,
            key: String::new(),
            n: 0,
        }
    }
    pub fn key(mut self, k: impl Into<String>) -> Self {
        self.key = k.into();
        self
    }
    pub fn n(mut self, n: usize) -> Self {
        self.n = n;
        self
    }
}

/// A screen-specific modal that still lives on the shared modal stack.
pub trait CustomModal {
    fn on_key(&mut self, key: &Key, focus: &mut Focus, ring: &FocusRing, w: &World) -> Outcome;
    fn on_click(
        &mut self,
        _id: WidgetId,
        _pos: Position,
        _focus: &mut Focus,
        _w: &World,
    ) -> Outcome {
        Outcome::Consumed
    }
    fn on_wheel(&mut self, _delta: i32) -> Outcome {
        Outcome::Consumed
    }
    fn on_tick(&mut self, _w: &World) -> Outcome {
        Outcome::Ignored
    }
    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World);
    fn done(&mut self) -> Option<ModalResult>;
    fn initial_focus(&self) -> WidgetId;
    fn hints(&self) -> Vec<Hint>;
    fn is_editing(&self) -> bool {
        false
    }
    fn cancel_on_outside_click(&self) -> bool {
        true
    }
}

#[allow(clippy::large_enum_variant)]
pub enum Modal {
    Dialog(Dialog),
    Picker(Picker),
    Browser(FileBrowser),
    Choice(ChoiceDialog),
    Form(FormDialog),
    Op(OpFlow),
    Info(InfoDialog),
    Help(HelpOverlay),
    Custom(Box<dyn CustomModal>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalResult {
    /// `action` is the pressed button index (None = cancelled); `text` the
    /// prompt's value.
    Dialog {
        action: Option<usize>,
        text: Option<String>,
    },
    Picked(usize),
    PickedAlt(usize),
    /// Picker Tab: cycle scope (owner re-supplies rows).
    Scope,
    Cancelled,
    Browser(BrowserResult),
    Choice(Option<usize>),
    Form(Option<FormValues>),
    /// Named button on a form other than Save/Cancel (e.g. Validate).
    FormAction(String, FormValues),
    Op(Option<OpReference>),
    Info(InfoResult),
    Custom(String),
}

/// Navigation the app performs on a screen's behalf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Go {
    Manager,
    Settings,
    Accounts {
        select: Option<AccountId>,
    },
    Usage {
        select: Option<AccountId>,
    },
    /// Edit an existing Workspace, or create from a pending one.
    Editor {
        workspace: Option<WorkspaceId>,
        pending: Option<Box<Workspace>>,
    },
    Prelude,
    Launch {
        workspace: Option<WorkspaceId>,
        role: String,
        agent: Agent,
        account: Option<AccountId>,
        plan: LaunchPlan,
    },
    Attach {
        instance: InstanceId,
        pane: Option<u64>,
    },
    NewSession {
        instance: InstanceId,
        agent: Option<Agent>,
        account: Option<AccountId>,
    },
    /// The client leaves; the instance keeps running.
    Detach,
    /// The foreground instance ended (last session closed / exit chosen).
    InstanceEnded {
        instance: InstanceId,
        purge: bool,
    },
    /// A failed fresh launch was acknowledged.
    LaunchFailedAck {
        instance: Option<InstanceId>,
    },
    Quit,
}

pub enum Request {
    Status(String),
    Error(String),
    Open(Box<Modal>, ModalTag),
    /// Replace the top modal (nested flows keep one visible owner).
    Replace(Box<Modal>, ModalTag),
    Close,
    Go(Go),
    Copy(String),
    Help,
    /// Refresh the top picker's items (owner changed its data).
    RefreshPicker,
    /// Mutate the form that is the top modal (after a nested picker).
    WithForm(Box<dyn FnOnce(&mut FormDialog)>),
}

pub struct Cx<'a> {
    pub focus: &'a mut Focus,
    pub ring: &'a FocusRing,
    pub requests: Vec<Request>,
}

impl Cx<'_> {
    pub fn focus_next(&mut self) {
        self.focus.next(self.ring);
    }
    pub fn focus_prev(&mut self) {
        self.focus.prev(self.ring);
    }
    pub fn status(&mut self, s: impl Into<String>) {
        self.requests.push(Request::Status(s.into()));
    }
    pub fn error(&mut self, s: impl Into<String>) {
        self.requests.push(Request::Error(s.into()));
    }
    pub fn open(&mut self, modal: Modal, tag: ModalTag) {
        self.requests.push(Request::Open(Box::new(modal), tag));
    }
    pub fn replace(&mut self, modal: Modal, tag: ModalTag) {
        self.requests.push(Request::Replace(Box::new(modal), tag));
    }
    pub fn close(&mut self) {
        self.requests.push(Request::Close);
    }
    pub fn go(&mut self, g: Go) {
        self.requests.push(Request::Go(g));
    }
    pub fn copy(&mut self, s: impl Into<String>) {
        self.requests.push(Request::Copy(s.into()));
    }
    pub fn help(&mut self) {
        self.requests.push(Request::Help);
    }
    pub fn with_form(&mut self, f: impl FnOnce(&mut FormDialog) + 'static) {
        self.requests.push(Request::WithForm(Box::new(f)));
    }
}

/// Everything a route screen can do. Default bodies let small screens stay
/// small; the shell calls only what it needs.
pub trait Screen {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome;
    fn on_click(&mut self, _id: WidgetId, _pos: Position, _w: &mut World, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }
    fn on_double_click(
        &mut self,
        id: WidgetId,
        pos: Position,
        w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        let _ = (id, pos, w, cx);
        Outcome::Ignored
    }
    fn on_drag(&mut self, _pressed: WidgetId, _pos: Position, _w: &mut World) -> Outcome {
        Outcome::Ignored
    }
    fn on_release(
        &mut self,
        _pressed: WidgetId,
        _pos: Position,
        _w: &mut World,
        _cx: &mut Cx,
    ) -> Outcome {
        Outcome::Ignored
    }
    fn on_wheel(&mut self, _id: WidgetId, _delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        Outcome::Ignored
    }
    fn on_paste(&mut self, _text: &str, _w: &mut World) -> Outcome {
        Outcome::Ignored
    }
    fn on_tick(&mut self, _w: &mut World, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }
    fn on_msg(&mut self, _msg: &Msg, _w: &mut World, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }
    fn on_modal(
        &mut self,
        _tag: &ModalTag,
        _result: ModalResult,
        _w: &mut World,
        _cx: &mut Cx,
    ) -> Outcome {
        Outcome::Ignored
    }
    /// Rows for a picker this screen opened; `None` keeps the current rows.
    fn picker_items(
        &mut self,
        _tag: &ModalTag,
        _query: &str,
        _w: &World,
    ) -> Option<Vec<PickerItem>> {
        None
    }
    /// A form field changed: reveal/hide dependent fields.
    fn form_changed(&mut self, _tag: &ModalTag, _form: &mut FormDialog, _w: &World) {}
    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World);
    fn hints(&self, focus: Option<WidgetId>, w: &World) -> Vec<Hint>;
    /// Scope crumb after the Construct state: `Workspaces › api-server`.
    fn crumb(&self, w: &World) -> String;
    /// Extra right-hand strip segments (dirty count, refreshing…).
    fn strip_right(&self, _w: &World) -> Vec<Segment> {
        vec![]
    }
    fn is_editing(&self) -> bool {
        false
    }
    fn animating(&self, _w: &World) -> bool {
        false
    }
    /// The route became active.
    fn enter(&mut self, _w: &mut World, _cx: &mut Cx) {}
    /// Where keyboard focus lands when nothing better is known.
    fn primary_focus(&self) -> Option<WidgetId>;
    /// Esc at the top of this screen's ladder: leave it?
    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        cx.go(Go::Manager);
        Outcome::Changed
    }
}

/// Segment helper.
pub fn seg(text: impl Into<String>, tone: Tone) -> Segment {
    Segment::new(text, tone)
}

pub fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("{n} {one}")
    } else {
        format!("{n} {many}")
    }
}
