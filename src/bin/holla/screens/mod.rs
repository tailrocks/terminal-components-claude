//! Surfaces and the contract they share with the shell. A tab is something
//! that runs (an activity, an executing plan); a page is something you are
//! deciding (the finder, a domain page, Disk, a plan review, a gate). The
//! Here tab holds a stack of pages; every other tab holds one screen.

pub mod activity;
pub mod cleanup;
pub mod disk;
pub mod files;
pub mod finder;
pub mod modals;
pub mod plan;
pub mod review;
pub mod snapshot;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::pointer::PointerEvent;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::Tone;
use junie_tui::ui::context::ContextAnchor;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layer::{LayerEntry, LayerTag, ShellLayers};
use junie_tui::ui::layers::LayerResult;
use junie_tui::widgets::keyhint::Hint;
use junie_tui::widgets::menu::{ContextMenu, MenuAction, MenuItem};
use junie_tui::widgets::picker::PickerAction;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::statusbar::StatusItem;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::domain::context::Scope;
use crate::sim::world::{Msg, World};

/// The shell's layer stack: every dialog, picker, menu and text modal,
/// owned by the tab that opened it.
pub type Layers = ShellLayers<ModalResult, (), usize>;

/// One open layer of [`Layers`].
pub type LayerSlot = LayerEntry<ModalResult, (), usize>;

/// What a layer reports to its owner (the shell or a screen), by
/// [`LayerTag`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalResult {
    /// The pressed button index (None = cancelled) and the typed text.
    Dialog {
        action: Option<usize>,
        text: Option<String>,
    },
    Picked(usize),
    Cancelled,
    MenuChosen(usize),
    Closed,
    /// A picker event that leaves it open (the query changed, Submit, ...).
    PickerEvent(PickerAction),
    /// A library modal holla does not open (forms, choices, info).
    Other(LayerResult),
}

impl From<LayerResult> for ModalResult {
    fn from(r: LayerResult) -> Self {
        match r {
            LayerResult::Dialog { action, text } => Self::Dialog { action, text },
            LayerResult::Picker(PickerAction::Chosen(i) | PickerAction::ChosenAlt(i)) => {
                Self::Picked(i)
            }
            LayerResult::Picker(PickerAction::Cancelled) => Self::Cancelled,
            LayerResult::Picker(ev) => Self::PickerEvent(ev),
            LayerResult::Menu(MenuAction::Chosen(i)) => Self::MenuChosen(i),
            LayerResult::Menu(MenuAction::Dismissed) => Self::Cancelled,
            LayerResult::Help => Self::Closed,
            other => Self::Other(other),
        }
    }
}

/// What Enter should do with an item once the shell has resolved trust,
/// arguments and gates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Go {
    /// Run an item by id (the shell resolves trust, arguments and gates).
    Run {
        item: String,
        args: Vec<(String, String)>,
    },
    /// Show the alternatives menu for an item at an anchor.
    Alternatives { item: String, anchor: Rect },
    /// Push a page onto the Here tab.
    Push(Page),
    /// Pop the top page of the Here tab.
    Pop,
    /// Activate the Here tab (keeping its stack).
    Here,
    /// Switch the finder scope.
    Scope(Scope),
    /// Confirm a reviewed plan: gate it or start it.
    ConfirmPlan(String),
    /// A gate-1 page was accepted for a plan or an item.
    Gate1Accepted(GateTarget),
    /// A trust page accepted a configuration.
    Trusted { config: String, then: String },
    /// Build the cleanup plan for the selected candidates and review it.
    CleanupPlan(Vec<String>),
    /// Restart an activity in place (a failed launch's retry).
    Restart(String),
    /// Close the active tab (asks when something runs).
    CloseTab,
    /// Re-read the project configuration (an edit was simulated).
    Reload,
    /// Run an ad-hoc typed command as an activity (an OS opener, a reveal).
    Exec {
        label: String,
        argv: Vec<crate::domain::exec::Command>,
    },
    /// Copy through the terminal's OSC 52 channel with a truthful result.
    Osc52(String),
    /// Analyze a path in the Disk flow.
    Analyze(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateTarget {
    Plan(String),
    Item {
        item: String,
        args: Vec<(String, String)>,
    },
    /// An authorized deletion specification (HP21).
    Cleanup(crate::domain::cleanup::DeletePlan),
}

/// A page the Here tab can hold; built lazily by the shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Page {
    Finder {
        group: Option<&'static str>,
    },
    Disk {
        path: String,
    },
    PlanReview {
        plan: String,
    },
    Review(GateTarget),
    Trust {
        config: String,
        then: String,
    },
    Args {
        item: String,
    },
    Snapshot {
        kind: String,
    },
    /// The folder browser (HP04) or a file preview at `path`.
    Files {
        path: String,
        query: Option<String>,
    },
    /// The home file search (HP03).
    Find,
    /// Home folders and insight roots with cached sizes (HP19).
    DiskOverview,
    /// macOS Spotlight top files (HP19).
    TopFiles,
    /// The insight review, optionally one category (HP20).
    Cleanup {
        category: Option<String>,
    },
    /// Gate 1 for a deletion plan (HP21).
    CleanupGate {
        plan: crate::domain::cleanup::DeletePlan,
    },
    /// A cleanup report (HP22).
    Report {
        index: usize,
    },
    /// Custom-action configuration diagnostics (HP17).
    Config {
        path: String,
    },
}

/// What a holla page asks of the shell (the library request).
pub type Request = junie_tui::shell::Request<ModalResult, (), Go>;

/// The event context a holla page handles input in (the library one).
pub type Cx<'a> = junie_tui::shell::Cx<'a, ModalResult, (), Go>;

/// Status-bar contribution of a screen: the centre item and extra right
/// items.
#[derive(Default)]
pub struct StatusBits {
    pub center: Option<StatusItem>,
    pub right: Vec<StatusItem>,
}

/// Everything a page or tab screen can do. Default bodies keep small
/// screens small.
pub trait Screen {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome;
    /// One pointer event the router resolved for this screen
    /// ([`junie_tui::core::pointer::PointerRouter`]): a press (with the
    /// click count it starts), a completed click (its count is
    /// `ev.count()`), a drag of the held press, a release away from it,
    /// a secondary or middle click, or a wheel notch on either axis
    /// (offered to each owner under the pointer, innermost first, until
    /// one takes it). Hand it to the owning widget
    /// ([`junie_tui::core::pointer::MouseTarget::route`]).
    fn pointer(&mut self, _ev: &PointerEvent, _w: &mut World, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }
    /// The context key ([`Cx::context`]: Shift+F10 and the Menu key by
    /// default) on the focused control: open its menu below the cursor row.
    fn on_context_key(&mut self, _w: &mut World, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }
    fn on_paste(&mut self, _text: &str, _w: &mut World, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }
    /// A control lost focus (or a click on dead space ended its edit):
    /// the screen that owns `ch.from` ends the edit through the control's
    /// `on_blur`.
    fn on_blur(
        &mut self,
        _ch: junie_tui::core::focus::FocusChange,
        _w: &mut World,
        _cx: &mut Cx,
    ) -> Outcome {
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
        _tag: &LayerTag,
        _result: ModalResult,
        _w: &mut World,
        _cx: &mut Cx,
    ) -> Outcome {
        Outcome::Ignored
    }
    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World);
    fn hints(&self, focus: Option<WidgetId>, w: &World) -> Vec<Hint>;
    /// Breadcrumb segment shown in the Here tab label (`Disk › Usage`).
    fn crumb(&self, w: &World) -> String;
    fn status(&self, _w: &World) -> StatusBits {
        StatusBits::default()
    }
    /// Letters type into a query on this screen.
    fn typing_hot(&self) -> bool {
        false
    }
    fn is_editing(&self) -> bool {
        false
    }
    fn animating(&self, _w: &World) -> bool {
        false
    }
    fn enter(&mut self, _w: &mut World, _cx: &mut Cx) {}
    fn primary_focus(&self) -> Option<WidgetId>;
    /// Esc at the top of this screen's ladder.
    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        cx.go(Go::Pop);
        Outcome::Changed
    }
    /// The finder behind this screen, when it is one.
    fn as_finder(&mut self) -> Option<&mut finder::FinderPage> {
        None
    }
    /// The files page behind this screen, when it is one.
    fn as_files(&mut self) -> Option<&mut files::FilesPage> {
        None
    }
    /// Proof accessors: the tests read work counters through the page.
    #[cfg(test)]
    fn as_activity(&mut self) -> Option<&mut activity::ActivityTab> {
        None
    }
    #[cfg(test)]
    fn as_disk(&mut self) -> Option<&mut disk::DiskPage> {
        None
    }
}

/// Where the context menu of hand-drawn list row `i` opens for the
/// keyboard: below the row as drawn this frame (`list.child(i)`), else
/// below its slot in `area` scrolled by `offset`.
pub fn row_anchor(list: WidgetId, i: usize, area: Rect, offset: usize) -> ContextAnchor {
    let dy = i
        .saturating_sub(offset)
        .min(area.height.saturating_sub(1) as usize) as u16;
    ContextAnchor::Control {
        id: list.child(i),
        fallback: Rect::new(area.x, area.y + dy, area.width.max(1), 1),
    }
}

/// Open a row's context menu through the shell's layer stack: `items`
/// titled `title` at `anchor`, answered to the owner as
/// [`ModalResult::MenuChosen`] under `tag`.
pub fn open_context(
    cx: &mut Cx,
    id: WidgetId,
    anchor: ContextAnchor,
    title: impl Into<String>,
    items: Vec<MenuItem>,
    tag: LayerTag,
) -> Outcome {
    let menu = anchor.apply(ContextMenu::new(id, items)).title(title);
    cx.open(menu, tag);
    Outcome::Changed
}

/// The key a context-menu item replays, from its character in the menu's
/// tag: `'\n'` is Enter, `'←'`/`'→'` the arrows, `'\u{1b}'` Esc, any other
/// character that plain key.
pub fn replay_key(c: char) -> Key {
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};
    let code = match c {
        '\n' => KeyCode::Enter,
        '←' => KeyCode::Left,
        '→' => KeyCode::Right,
        '\u{1b}' => KeyCode::Esc,
        c => KeyCode::Char(c),
    };
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

/// A menu's chosen item `k` under `tag` of `kind` whose key is
/// `row:keys`: the row it was opened for and the key item `k` replays.
pub fn row_choice(
    tag: &LayerTag,
    kind: &str,
    result: &ModalResult,
) -> Option<(usize, Option<Key>, usize)> {
    let ModalResult::MenuChosen(k) = *result else {
        return None;
    };
    if tag.kind != kind {
        return None;
    }
    let (row, keys) = tag.key.split_once(':')?;
    let row = row.parse::<usize>().ok()?;
    Some((row, keys.chars().nth(k).map(replay_key), k))
}

/// A tick's rebuild of a screen's view from the world: `Changed` only when
/// what `view` reads differs afterwards. The simulated world advances every
/// tick (reduced motion included); a screen whose rows did not change asks
/// for no redraw, so an idle screen draws nothing.
pub(crate) fn rebuild_on_tick<S, V: PartialEq>(
    screen: &mut S,
    view: impl Fn(&S) -> V,
    rebuild: impl FnOnce(&mut S),
) -> Outcome {
    let before = view(screen);
    rebuild(screen);
    if view(screen) == before {
        Outcome::Ignored
    } else {
        Outcome::Changed
    }
}

/// `2 min 14 s` from ticks.
pub fn ticks_label(ticks: u64) -> String {
    junie_tui::ui::text::duration(
        std::time::Duration::from_millis(ticks * crate::sim::world::TICK_MS as u64),
        junie_tui::ui::text::DurationStyle::Spoken,
    )
}

/// Tone for a risk word in rows and previews (D-7).
pub fn risk_tone(risk: crate::domain::action::Risk) -> Tone {
    use crate::domain::action::Risk;
    match risk {
        Risk::ReadOnly => Tone::Faint,
        Risk::Mutating => Tone::Muted,
        Risk::Destructive => Tone::Error,
        Risk::Privileged => Tone::Warning,
    }
}

/// A one-row section heading: text-muted (faint is 4.4:1 on the canvas),
/// sentence case (DESIGN hierarchy).
pub fn heading(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    w: u16,
    text: &str,
    t: &junie_tui::theme::Theme,
    bg: ratatui::style::Color,
) {
    if w == 0 {
        return;
    }
    buf.set_string(
        x,
        y,
        junie_tui::ui::text::truncate(text, w as usize),
        ratatui::style::Style::new()
            .fg(t.tone_on(junie_tui::theme::Tone::Muted, bg))
            .bg(bg),
    );
}

/// Route a pointer press to a raw `ScrollState` scrollbar. Widget-owned
/// scroll models expose this directly; Holla's simple screens keep the model
/// inline, so they share the exact same track mapping here.
pub(crate) fn scroll_press(area: Rect, pos: Position, scroll: &mut ScrollState) -> Outcome {
    let track = Rect::new(area.right().saturating_sub(1), area.y, 1, area.height);
    if scrollbar::press(track, pos, scroll) {
        Outcome::Changed
    } else {
        Outcome::Consumed
    }
}

/// Route a pointer drag to a raw `ScrollState` scrollbar.
pub(crate) fn scroll_drag(area: Rect, pos: Position, scroll: &mut ScrollState) -> Outcome {
    let track = Rect::new(area.right().saturating_sub(1), area.y, 1, area.height);
    if scrollbar::drag(track, pos, scroll) {
        Outcome::Changed
    } else {
        Outcome::Consumed
    }
}
