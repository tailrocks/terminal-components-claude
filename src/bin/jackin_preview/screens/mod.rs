//! Route screens and the contract they share with the shell: input,
//! requests back to the app, modal ownership, rendering and hints.

pub mod accounts;
pub mod capsule;
pub mod cockpit;
pub mod config;
pub mod editor;
pub mod inspect;
pub mod manager;
pub mod modals;
pub mod prelude;
pub mod rowmenu;
pub mod settings;
pub mod usage;

use junie_tui::core::event::Outcome;
use junie_tui::widgets::option_list::Item;
use junie_tui::widgets::segments::Segment;

use crate::domain::account::AccountId;
use crate::domain::agent::Agent;
use crate::domain::instance::InstanceId;
use crate::domain::onepassword::OpReference;
use crate::domain::workspace::{Workspace, WorkspaceId};
use crate::sim::launch::LaunchPlan;
use crate::sim::world::{Msg, World};
use junie_tui::ui::layer::Layer;
pub use junie_tui::ui::layer::LayerTag;
use junie_tui::ui::layers::LayerResult;
use junie_tui::widgets::menu::MenuAction;
use junie_tui::widgets::picker::PickerAction;
use modals::{BrowserResult, FormAction, FormDialog, FormValues, InfoResult};

/// What a layer reports to the screen that opened it. Library modals
/// report a [`LayerResult`], converted once here; the app's own layers
/// (the file browser, the 1Password chain, the Capsule's custom modals)
/// report a `ModalResult` directly.
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
    /// A picker event the shell handles itself (the query changed: the
    /// owner re-supplies rows) or that nobody needs (Back, Submit).
    PickerEvent(PickerAction),
    Cancelled,
    Browser(BrowserResult),
    Choice(Option<usize>),
    Form(Option<FormValues>),
    /// Named button on a form other than Save/Cancel (e.g. Validate).
    FormAction(String, FormValues),
    /// A form event before the shell attaches the form's values (Save,
    /// a chooser or action button) or answers it itself (a field
    /// changed). Screens never receive it.
    FormEvent(FormAction),
    Op(Option<OpReference>),
    Info(InfoResult),
    /// The help overlay closed; nobody is told.
    Help,
    Custom(String),
}

impl From<LayerResult> for ModalResult {
    fn from(r: LayerResult) -> Self {
        match r {
            LayerResult::Dialog { action, text } => ModalResult::Dialog { action, text },
            LayerResult::Picker(PickerAction::Chosen(i))
            | LayerResult::Menu(MenuAction::Chosen(i)) => ModalResult::Picked(i),
            LayerResult::Picker(PickerAction::ChosenAlt(i)) => ModalResult::PickedAlt(i),
            LayerResult::Picker(PickerAction::NextScope | PickerAction::PrevScope) => {
                ModalResult::Scope
            }
            LayerResult::Picker(PickerAction::Cancelled) | LayerResult::Menu(_) => {
                ModalResult::Cancelled
            }
            LayerResult::Picker(ev) => ModalResult::PickerEvent(ev),
            LayerResult::Form(FormAction::Cancel) => ModalResult::Form(None),
            LayerResult::Form(ev) => ModalResult::FormEvent(ev),
            LayerResult::Choice(c) => ModalResult::Choice(c),
            LayerResult::Info(r) => ModalResult::Info(r),
            LayerResult::Help => ModalResult::Help,
            LayerResult::File(r) => ModalResult::Browser(r),
        }
    }
}

/// Every layer on the app's stack: it reads the world while it draws and
/// handles input, and reports a [`ModalResult`] to its owner.
pub type AppLayer = dyn Layer<ModalResult, World>;

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

/// What a jackin screen asks of the shell (the library request).
pub type Request = junie_tui::shell::Request<ModalResult, World, Go>;

/// The event context a jackin screen handles input in (the library one).
pub type Cx<'a> = junie_tui::shell::Cx<'a, ModalResult, World, Go>;

/// The types a jackin screen works with ([`junie_tui::shell::Page`]).
pub struct Jackin;

impl junie_tui::shell::PageTypes for Jackin {
    type State = World;
    type Msg = Msg;
    type Result = ModalResult;
    type LayerState = World;
    type Go = Go;
}

/// What a jackin route screen adds to the library page
/// ([`junie_tui::shell::Page`], whose Esc at the top goes back: the
/// manager). Default bodies let small screens stay small; the shell calls
/// only what it needs.
pub trait Screen: junie_tui::shell::Page<Jackin> {
    /// Rows for a picker this screen opened; `None` keeps the current rows.
    fn picker_items(&mut self, _tag: &LayerTag, _query: &str, _w: &World) -> Option<Vec<Item>> {
        None
    }
    /// A form field changed: reveal/hide dependent fields.
    fn form_changed(&mut self, _tag: &LayerTag, _form: &mut FormDialog, _w: &World) {}
    /// Extra right-hand strip segments (dirty count, refreshing…).
    fn strip_right(&self, _w: &World) -> Vec<Segment> {
        vec![]
    }
    /// True while the screen draws a keyboard-owned layer above itself (an
    /// open menu): the shell then renders the page with no focus indicator
    /// ([`junie_tui::ui::ctx::RenderCtx::page_under_layer`]).
    fn layer_open(&self) -> bool {
        false
    }
    /// Leave through the screen's own unsaved-changes question (Save /
    /// Discard / Cancel); a clean screen goes straight to the manager.
    fn request_leave(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        cx.go(Go::Manager);
        Outcome::Changed
    }
}

/// The Health value as spans, shared by Usage and Accounts: the health
/// word (blocked and degraded in error bold, attention in warning, the
/// rest primary), then each issue in the tone of what it says
/// ([`OverallSummary::issue_parts`](crate::domain::usage::OverallSummary::issue_parts)).
pub fn health_spans(
    s: &crate::domain::usage::OverallSummary,
) -> Vec<junie_tui::widgets::viewport::Span> {
    use crate::domain::usage::HealthWord;
    use junie_tui::theme::Tone;
    use junie_tui::widgets::viewport::Span;
    let word = Span::new(
        s.health.label(),
        match s.health {
            HealthWord::Blocked | HealthWord::Degraded => Tone::Error,
            HealthWord::Attention => Tone::Warning,
            _ => Tone::Normal,
        },
    );
    let word = if matches!(s.health, HealthWord::Blocked | HealthWord::Degraded) {
        word.bold()
    } else {
        word
    };
    let mut v = vec![word];
    for (text, tone) in s.issue_parts() {
        v.push(Span::new(" · ", Tone::Muted));
        v.push(Span::new(text, tone));
    }
    v
}
