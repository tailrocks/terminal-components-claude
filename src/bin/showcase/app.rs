//! Application shell: layout, event routing, focus/hover bookkeeping,
//! navigation, modal stack, footer hints.

use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;

use crate::pages::{Page, PageCtx, PageEvent, Request};
use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::core::pointer::{FocusOn, PointerConfig, PointerState, Release, SecondaryRoute};
use junie_tui::theme::{BadgeKind, Theme};
use junie_tui::ui::ctx::{Interaction, RenderCtx, fill};
use junie_tui::ui::leave::{KeyContext, LeaveGuard, LeaveKey, QuitKeys, Resolved, Stakes};
use junie_tui::widgets::dialog::{Dialog, DialogBody, DialogResult};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::tree::{TreeAction, TreeNode, TreeView};

pub const MIN_WIDTH: u16 = 72;
pub const MIN_HEIGHT: u16 = 20;

const NAV: WidgetId = WidgetId::of("app.nav");
const HEADER_HELP: WidgetId = WidgetId::of("app.header.help");
const HEADER_INSPECT: WidgetId = WidgetId::of("app.header.inspect");
const HELP_DIALOG: WidgetId = WidgetId::of("app.help");
/// The brand lockup in the header: the app menu's Quit (mouse #40).
pub const HEADER_BRAND: WidgetId = WidgetId::of("app.header.brand");
/// The quit confirmation the brand opens.
pub const QUIT_DIALOG: WidgetId = WidgetId::of("app.quit");

/// Tick delivery mode: `Paused` pins every tick-derived glyph at
/// `--frame N` (the pages are fast-forwarded synchronously at startup) so
/// animated surfaces are capturable; the default is unchanged live ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Full,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    Overview,
    Buttons,
    Forms,
    Inputs,
    TextAreas,
    Panels,
    Sidebars,
    Dialogs,
    Tables,
    Editable,
    Lists,
    Trees,
    Progress,
    Scrolling,
    Terminal,
    Editor,
    Diff,
    Grid,
    Chips,
    Chrome,
    Pickers,
    Settings,
    TaskRunner,
    Selectable,
    CellEditors,
}

pub struct NavEntry {
    pub id: PageId,
    pub label: &'static str,
    /// The sidebar group the page sits under ([`NAV_GROUPS`]); empty for a
    /// top-level page.
    pub group: &'static str,
}

/// Sidebar groups in tree order. A group never holds a page with its own
/// name, and the tree is at most two levels deep.
pub const NAV_GROUPS: &[&str] = &[
    "App shell",
    "Inputs & forms",
    "Tables",
    "Navigation",
    "Overlays",
    "Feedback",
    "Layout",
    "Text & code",
    "Screens",
];

/// Every page in tree order (group by group): the sidebar, `[`/`]` and
/// the page vector all follow it.
pub const NAV_ENTRIES: &[NavEntry] = &[
    NavEntry {
        id: PageId::Overview,
        label: "Overview",
        group: "",
    },
    NavEntry {
        id: PageId::Chrome,
        label: "Chrome",
        group: "App shell",
    },
    NavEntry {
        id: PageId::Buttons,
        label: "Buttons",
        group: "Inputs & forms",
    },
    NavEntry {
        id: PageId::Inputs,
        label: "Inputs",
        group: "Inputs & forms",
    },
    NavEntry {
        id: PageId::TextAreas,
        label: "Text areas",
        group: "Inputs & forms",
    },
    NavEntry {
        id: PageId::Chips,
        label: "Chips & selects",
        group: "Inputs & forms",
    },
    NavEntry {
        id: PageId::Forms,
        label: "Forms",
        group: "Inputs & forms",
    },
    NavEntry {
        id: PageId::Tables,
        label: "Basic table",
        group: "Tables",
    },
    NavEntry {
        id: PageId::Editable,
        label: "Editable",
        group: "Tables",
    },
    NavEntry {
        id: PageId::Grid,
        label: "Data grid",
        group: "Tables",
    },
    NavEntry {
        id: PageId::Selectable,
        label: "Selection",
        group: "Tables",
    },
    NavEntry {
        id: PageId::CellEditors,
        label: "Cell editors",
        group: "Tables",
    },
    NavEntry {
        id: PageId::Lists,
        label: "Lists",
        group: "Navigation",
    },
    NavEntry {
        id: PageId::Trees,
        label: "Trees",
        group: "Navigation",
    },
    NavEntry {
        id: PageId::Sidebars,
        label: "Sidebars",
        group: "Navigation",
    },
    NavEntry {
        id: PageId::Dialogs,
        label: "Dialogs",
        group: "Overlays",
    },
    NavEntry {
        id: PageId::Pickers,
        label: "Pickers",
        group: "Overlays",
    },
    NavEntry {
        id: PageId::Progress,
        label: "Progress",
        group: "Feedback",
    },
    NavEntry {
        id: PageId::Panels,
        label: "Panels",
        group: "Layout",
    },
    NavEntry {
        id: PageId::Scrolling,
        label: "Scrolling",
        group: "Layout",
    },
    NavEntry {
        id: PageId::Editor,
        label: "Code editor",
        group: "Text & code",
    },
    NavEntry {
        id: PageId::Diff,
        label: "Diff",
        group: "Text & code",
    },
    NavEntry {
        id: PageId::Terminal,
        label: "Terminal",
        group: "Text & code",
    },
    NavEntry {
        id: PageId::Settings,
        label: "Settings",
        group: "Screens",
    },
    NavEntry {
        id: PageId::TaskRunner,
        label: "Task runner",
        group: "Screens",
    },
];

/// Extra `--page` names for pages whose label is long or reads several
/// ways (compared like labels: letters and digits only).
const PAGE_ALIASES: &[(&str, PageId)] = &[
    // labels before the sidebar became a tree (a group never repeats a
    // page's name): their `--page` names keep working
    ("tables", PageId::Tables),
    ("editabletables", PageId::Editable),
    ("selectabletable", PageId::Selectable),
    ("optioneditors", PageId::CellEditors),
    ("options", PageId::CellEditors),
    ("enums", PageId::CellEditors),
    ("choices", PageId::CellEditors),
    ("selection", PageId::Selectable),
    ("bulkedit", PageId::Selectable),
    ("selectable", PageId::Selectable),
];

/// Which sidebar groups are open: the navigation tree's expansion policy.
/// Whatever the policy, navigating to a page opens its group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavExpand {
    /// Only the current page's group is open at start. A group opened only
    /// because navigation passed through it closes again when navigation
    /// moves on; groups the user opens or closes stay as they are left.
    #[default]
    CurrentGroup,
    /// Only the current page's group is open after every navigation.
    Accordion,
    /// Every group starts open.
    All,
}

impl PageId {
    pub fn index(self) -> usize {
        NAV_ENTRIES.iter().position(|e| e.id == self).unwrap_or(0)
    }

    /// The page's path in the sidebar tree: `[root]` for a top-level page,
    /// `[group, position in group]` otherwise.
    pub fn nav_path(self) -> Vec<usize> {
        let e = &NAV_ENTRIES[self.index()];
        let roots = nav_roots();
        if e.group.is_empty() {
            return vec![
                roots
                    .iter()
                    .position(|r| *r == NavRoot::Page(self))
                    .unwrap_or(0),
            ];
        }
        let g = roots
            .iter()
            .position(|r| *r == NavRoot::Group(e.group))
            .unwrap_or(0);
        let k = NAV_ENTRIES
            .iter()
            .filter(|x| x.group == e.group)
            .position(|x| x.id == self)
            .unwrap_or(0);
        vec![g, k]
    }

    /// The page at a sidebar tree path; `None` for a group row.
    pub fn from_nav_path(path: &[usize]) -> Option<Self> {
        let roots = nav_roots();
        match (path, path.first().and_then(|&r| roots.get(r))) {
            ([_], Some(NavRoot::Page(id))) => Some(*id),
            ([_, k], Some(NavRoot::Group(g))) => NAV_ENTRIES
                .iter()
                .filter(|e| e.group == *g)
                .nth(*k)
                .map(|e| e.id),
            _ => None,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        // compare on letters and digits only, so "chips & selects",
        // "chips-selects" and "chipsselects" all resolve
        let norm = |s: &str| -> String {
            s.chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .map(|c| c.to_ascii_lowercase())
                .collect()
        };
        let n = norm(name);
        NAV_ENTRIES
            .iter()
            .find(|e| norm(e.label) == n)
            .map(|e| e.id)
            .or_else(|| {
                PAGE_ALIASES
                    .iter()
                    .find(|(alias, _)| *alias == n)
                    .map(|&(_, id)| id)
            })
    }
}

/// A root row of the sidebar tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavRoot {
    /// A top-level page (Overview).
    Page(PageId),
    /// A group of pages.
    Group(&'static str),
}

/// The sidebar's root rows in order: top-level pages and groups as they
/// first appear in [`NAV_ENTRIES`].
pub fn nav_roots() -> Vec<NavRoot> {
    let mut out = Vec::new();
    for e in NAV_ENTRIES {
        let root = if e.group.is_empty() {
            NavRoot::Page(e.id)
        } else {
            NavRoot::Group(e.group)
        };
        if !out.contains(&root) {
            out.push(root);
        }
    }
    out
}

/// The sidebar tree: top-level pages, then [`NAV_GROUPS`] as branches with
/// their pages as leaves. Type-ahead on, `›` on the current page; the
/// card it sits in is its edge, so it keeps no divider column.
fn nav_tree() -> TreeView {
    let nodes = nav_roots()
        .into_iter()
        .map(|root| match root {
            NavRoot::Page(id) => TreeNode::leaf(NAV_ENTRIES[id.index()].label),
            NavRoot::Group(g) => TreeNode::dir(
                g,
                NAV_ENTRIES
                    .iter()
                    .filter(|e| e.group == g)
                    .map(|e| TreeNode::leaf(e.label))
                    .collect(),
            ),
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(
        nodes.iter().filter(|n| !n.children.is_empty()).count(),
        NAV_GROUPS.len()
    );
    TreeView::new(NAV, nodes)
        .type_ahead(true)
        .selected_marker(Some("›"))
}

pub struct App {
    pub theme: Theme,
    pub pages: Vec<Box<dyn Page>>,
    pub page: PageId,
    /// The navigation sidebar: every page as a leaf under its group.
    pub nav: TreeView,
    /// Which sidebar groups are open ([`NavExpand::CurrentGroup`]).
    pub nav_expand: NavExpand,
    /// A group navigation opened on its own (not the user): it closes
    /// again when navigation moves to another group
    /// ([`NavExpand::CurrentGroup`]).
    nav_auto: Option<usize>,
    pub focus: Focus,
    pub ring: FocusRing,
    pub hits: HitRegistry,
    /// Hover, press, flash and click-count state.
    pub pointer: PointerState,
    /// Origin of the pointer clock.
    epoch: Instant,
    pub dialog: Option<Dialog>,
    pub inspector: bool,
    pub size: (u16, u16),
    pub tick: u64,
    pub last_key: Option<String>,
    pub status: Option<(String, Instant)>,
    pub quit: bool,
    /// Which keys ask to quit. Default [`QuitKeys::DEFAULT`]: Ctrl+C
    /// (after the page), Ctrl+Q and `q` (never over a field or unsaved
    /// edits); Esc never quits.
    pub quit_keys: QuitKeys,
    /// The quit guard: unsaved page edits ask Cancel / Discard (pages
    /// save on their own, so the shell offers no Save).
    leave: LeaveGuard<()>,
    pub motion: Motion,
    nav_reveal: bool,
    /// Type-to-filter in the sidebar (`/`): the query while filtering.
    pub nav_filter: Option<String>,
    layout: ShellLayout,
    /// Focus to restore when the help dialog closes.
    saved_focus: Option<WidgetId>,
    /// Modifiers held with the pointer event being handled; pages read
    /// them from [`PageCtx::mods`].
    pointer_mods: KeyModifiers,
    /// Multi-click count of the click being delivered ([`PageCtx::clicks`]).
    pointer_clicks: u8,
}

#[derive(Debug, Default, Clone, Copy)]
struct ShellLayout {
    header: Rect,
    sidebar: Rect,
    main: Rect,
    inspector: Rect,
    footer: Rect,
    too_small: bool,
}

impl App {
    /// Where the navigation sidebar was drawn in the last frame (the visual
    /// baseline test hashes everything but it).
    #[allow(dead_code)]
    pub fn sidebar_area(&self) -> Rect {
        self.layout.sidebar
    }

    /// Default live-tick construction (`with_motion` with `Full`, frame 0);
    /// the binary goes through `with_motion`, the in-process tests use this.
    #[allow(dead_code)]
    pub fn new(theme: Theme) -> Self {
        Self::with_motion(theme, Motion::Full, 0)
    }

    pub fn with_motion(theme: Theme, motion: Motion, frame: u64) -> Self {
        use crate::pages::*;
        let pages: Vec<Box<dyn Page>> = NAV_ENTRIES
            .iter()
            .map(|e| -> Box<dyn Page> {
                match e.id {
                    PageId::Overview => Box::new(overview::OverviewPage::new()),
                    PageId::Buttons => Box::new(buttons::ButtonsPage::new()),
                    PageId::Forms => Box::new(forms::FormsPage::new()),
                    PageId::Inputs => Box::new(inputs::InputsPage::new()),
                    PageId::TextAreas => Box::new(textareas::TextAreasPage::new()),
                    PageId::Panels => Box::new(panels::PanelsPage::new()),
                    PageId::Sidebars => Box::new(sidebars::SidebarsPage::new()),
                    PageId::Dialogs => Box::new(dialogs::DialogsPage::new()),
                    PageId::Tables => Box::new(tables::TablesPage::new()),
                    PageId::Editable => Box::new(editable::EditablePage::new()),
                    PageId::Lists => Box::new(lists::ListsPage::new()),
                    PageId::Trees => Box::new(trees::TreesPage::new()),
                    PageId::Progress => Box::new(progress::ProgressPage::new()),
                    PageId::Scrolling => Box::new(scrolling::ScrollingPage::new()),
                    PageId::Terminal => Box::new(terminal::TerminalPage::new()),
                    PageId::Editor => Box::new(editor::EditorPage::new()),
                    PageId::Diff => Box::new(diff::DiffPage::new()),
                    PageId::Grid => Box::new(grid::GridPage::new()),
                    PageId::Chips => Box::new(chips::ChipsPage::new()),
                    PageId::Pickers => Box::new(pickers::PickersPage::new()),
                    PageId::Chrome => Box::new(chrome::ChromePage::new()),
                    PageId::Settings => Box::new(settings::SettingsPage::new()),
                    PageId::TaskRunner => Box::new(taskrunner::TaskRunnerPage::new()),
                    PageId::Selectable => Box::new(selectable::SelectablePage::new()),
                    PageId::CellEditors => Box::new(celleditors::CellEditorsPage::new()),
                }
            })
            .collect();
        let mut focus = Focus::default();
        focus.focus(NAV);
        let mut app = Self {
            theme,
            pages,
            page: PageId::Overview,
            nav: nav_tree(),
            nav_expand: NavExpand::default(),
            nav_auto: None,
            focus,
            ring: FocusRing::default(),
            hits: HitRegistry::default(),
            pointer: PointerState::new(PointerConfig::DEFAULT.focus_on(FocusOn::Release)),
            epoch: Instant::now(),
            dialog: None,
            inspector: false,
            size: (0, 0),
            tick: frame,
            last_key: None,
            status: None,
            quit: false,
            quit_keys: QuitKeys::DEFAULT,
            leave: LeaveGuard::new(WidgetId::of("showcase.leave")).save(false),
            motion,
            nav_reveal: true,
            nav_filter: None,
            layout: ShellLayout::default(),
            saved_focus: None,
            pointer_mods: KeyModifiers::NONE,
            pointer_clicks: 0,
        };
        app.sync_nav(None);
        // Paused fast-forward: page Tick handlers are pure functions of the
        // tick count, so N synchronous dispatches reproduce the exact
        // post-N-ticks state. Every page is advanced (not only the current
        // one) because `goto` runs after construction; unvisited pages stay
        // idle anyway, and `on_tick` never fires again while paused.
        // Requests are dropped: a Status raised by a construction-time tick
        // (the progress bar crossing 100 %) is a wall-clock artifact that
        // paused motion can never age out, so it would leak a stale footer
        // message into unrelated pages' pinned frames.
        if motion == Motion::Paused {
            for _ in 0..frame {
                for i in 0..app.pages.len() {
                    let mut cx = PageCtx {
                        focus: &mut app.focus,
                        ring: &app.ring,
                        requests: Vec::new(),
                        mods: KeyModifiers::NONE,
                        clicks: 0,
                    };
                    app.pages[i].handle(&PageEvent::Tick, &mut cx);
                }
            }
        }
        app
    }

    /// Apply [`Self::nav_expand`] after the current page changed from
    /// `from` (`None` at start) and put the tree cursor and the current
    /// marker on it.
    fn sync_nav(&mut self, from: Option<PageId>) {
        let path = self.page.nav_path();
        // a top-level page has no group to open
        let group = (path.len() > 1).then_some(path[0]);
        let prev_auto = self.nav_auto.take();
        match (self.nav_expand, from) {
            (NavExpand::All, None) => self.nav.expand_all(),
            (NavExpand::All, Some(_)) => {}
            (NavExpand::CurrentGroup, None) | (NavExpand::Accordion, _) => {
                self.nav.expanded.clear();
            }
            (NavExpand::CurrentGroup, Some(_)) => {
                if let Some(g) = prev_auto
                    && Some(g) != group
                {
                    self.nav.expanded.remove(&vec![g]);
                }
            }
        }
        // the group is navigation's own when navigation opened it now, or
        // it already was and navigation stays inside it
        if let Some(group) = group {
            let opened_now = !self.nav.expanded.contains(&vec![group]);
            if self.nav_expand == NavExpand::CurrentGroup
                && (opened_now || prev_auto == Some(group))
            {
                self.nav_auto = Some(group);
            }
            self.nav.expanded.insert(vec![group]);
        }
        // the tree is rebuilt from the expansion set
        self.nav.flatten();
        self.nav.selected = Some(path.clone());
        self.nav.reveal(&path);
        self.nav_reveal = true;
    }

    /// Change the sidebar's expansion policy and apply it from scratch.
    pub fn set_nav_expand(&mut self, policy: NavExpand) {
        self.nav_expand = policy;
        self.nav_auto = None;
        self.sync_nav(None);
    }

    pub fn goto(&mut self, page: PageId) {
        if self.page != page {
            let from = self.page;
            self.page = page;
            self.sync_nav(Some(from));
            // focus stays on nav if it was there; otherwise first widget on page
            if !self.focus.is(NAV) {
                self.focus.set(None);
            }
        }
    }

    pub fn animating(&self) -> bool {
        if self.motion == Motion::Paused {
            return false;
        }
        let page = &self.pages[self.page.index()];
        page.animating()
            || self.pointer.has_flash()
            || self
                .dialog
                .as_ref()
                .is_some_and(|d| d.actions.iter().any(|b| b.busy))
    }

    pub fn tick_interval(&self) -> Duration {
        // a held drag past a selecting body's edge autoscrolls on the tick
        if self.pointer.autoscrolling() {
            return Duration::from_millis(self.pointer.config.autoscroll_ms);
        }
        // a press held on a scrollbar track pages on the tick
        if self.pointer.track_repeating() {
            return Duration::from_millis(self.pointer.config.track_repeat_ms);
        }
        if self.animating() {
            Duration::from_millis(80)
        } else {
            Duration::from_millis(400)
        }
    }

    fn interaction(&self) -> Interaction {
        let flash = self.pointer.flashing(self.now_ms());
        Interaction {
            focus: self.focus.current(),
            hover: self.pointer.hover,
            pressed: self.pointer.pressed,
            flash,
            focus_hidden: false,
            hover_suppressed: self.pointer.hover_suppressed,
            pointer_focus: self.pointer.pointer_focus,
            motion_unreported: !self.pointer.reports_motion(),
            tick: self.tick,
            // a paused capture has no clock: hover content shows at once
            hover_ms: (self.motion != Motion::Paused).then(|| self.pointer.hover_ms(self.now_ms())),
        }
    }

    fn set_status(&mut self, s: String) {
        self.status = Some((s, Instant::now()));
    }

    // ------------------------------------------------------------------ input

    pub fn handle(&mut self, input: Input) -> Outcome {
        let out = self.handle_input(input);
        out.or(self.forward_blur())
    }

    /// Hand every focus change the input caused to the dialog and the
    /// pages, so fields end their edits on the event path.
    fn forward_blur(&mut self) -> Outcome {
        let mut out = Outcome::Ignored;
        for ch in self.focus.settle(&self.ring) {
            if let Some(d) = self.dialog.as_mut() {
                out = out.or(d.on_blur(ch));
            }
            for i in 0..self.pages.len() {
                out = out.or(self.dispatch_to(i, PageEvent::Blur(ch)));
            }
        }
        out
    }

    fn handle_input(&mut self, input: Input) -> Outcome {
        match input {
            Input::Resize(w, h) => {
                self.size = (w, h);
                Outcome::Changed
            }
            Input::Tick => self.on_tick(),
            Input::Paste(text) => {
                if let Some(d) = self.dialog.as_mut() {
                    return d.on_paste(&text);
                }
                self.dispatch(PageEvent::Paste(text))
            }
            Input::Key(key) => {
                self.last_key = Some(describe_key(&key));
                self.pointer.suppress_hover();
                self.on_key(key)
            }
            Input::Mouse(m) => self.on_mouse(m),
        }
    }

    /// A due edge-autoscroll step replays the held drag at the pointer
    /// ([`PointerState::autoscroll_due`](junie_tui::core::pointer::PointerState::autoscroll_due)).
    /// A due track repeat replays the press held on a scrollbar track, so
    /// it keeps paging toward the pointer
    /// ([`PointerState::track_repeat_due`](junie_tui::core::pointer::PointerState::track_repeat_due)).
    fn autoscroll(&mut self) -> Outcome {
        let out = match self.pointer.autoscroll_due(self.now_ms()) {
            Some((_, pos)) => self.on_mouse(Mouse::new(MouseKind::Drag, pos)),
            None => Outcome::Ignored,
        };
        match self.pointer.track_repeat_due(self.now_ms()) {
            Some(pos) => out.or(self.on_mouse(Mouse::new(MouseKind::Down, pos))),
            None => out,
        }
    }

    fn on_tick(&mut self) -> Outcome {
        // edge autoscroll is the user's drag, not motion: it runs paused too
        let auto = self.autoscroll();
        if self.motion == Motion::Paused {
            return auto;
        }
        let mut out = auto;
        if self.animating() {
            self.tick = self.tick.wrapping_add(1);
            out = Outcome::Changed;
        }
        if self.pointer.expire_flash(self.now_ms()) {
            out = Outcome::Changed;
        }
        if let Some((_, at)) = &self.status
            && at.elapsed() > Duration::from_secs(4)
        {
            self.status = None;
            out = Outcome::Changed;
        }
        if self.dialog.is_none() {
            let page_out = self.dispatch(PageEvent::Tick);
            out = out.or(page_out);
        }
        out
    }

    fn dispatch(&mut self, ev: PageEvent) -> Outcome {
        self.dispatch_to(self.page.index(), ev)
    }

    fn dispatch_to(&mut self, i: usize, ev: PageEvent) -> Outcome {
        let mut cx = PageCtx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: Vec::new(),
            mods: self.pointer_mods,
            clicks: self.pointer_clicks,
        };
        let out = self.pages[i].handle(&ev, &mut cx);
        let requests = std::mem::take(&mut cx.requests);
        for r in requests {
            match r {
                Request::OpenDialog(d) => self.open_dialog(*d),
                Request::Status(s) => self.set_status(s),
            }
        }
        out
    }

    /// What quitting would lose across every page.
    pub fn quit_stakes(&self) -> Stakes {
        self.pages
            .iter()
            .fold(Stakes::Clean, |acc, p| acc.max(p.leave_stakes()))
    }

    /// Every quit goes through here: a clean showcase quits at once,
    /// unsaved page edits ask Cancel / Discard first.
    fn request_quit(&mut self) -> Outcome {
        let stakes = self.quit_stakes();
        match self.leave.request_quit(&stakes, &self.theme.strings) {
            Ok(_) => {
                self.quit = true;
                Outcome::Consumed
            }
            Err(d) => {
                self.open_dialog(*d);
                Outcome::Changed
            }
        }
    }

    fn open_dialog(&mut self, d: Dialog) {
        self.saved_focus = self.focus.current();
        self.focus.set(Some(d.initial_focus));
        self.dialog = Some(d);
        self.pointer.hover = None;
        self.pointer.pressed = None;
    }

    fn close_dialog(&mut self) -> Outcome {
        let Some(d) = self.dialog.take() else {
            return Outcome::Ignored;
        };
        self.focus.set(self.saved_focus.take());
        if self.leave.owns(d.id) {
            if let Some(result) = d.result
                && let Resolved::Discard(_) = self.leave.resolve(result)
            {
                self.quit = true;
            }
            return Outcome::Changed;
        }
        // the brand's quit confirmation: Quit goes through the same guard
        // as the q key
        if d.id == QUIT_DIALOG {
            if d.result == Some(DialogResult::Action(1)) {
                return self.request_quit();
            }
            return Outcome::Changed;
        }
        if d.id != HELP_DIALOG
            && let Some(result) = d.result
        {
            let id = d.id;
            let value = match &d.body {
                DialogBody::Input(i) => Some(i.text().to_owned()),
                _ => None,
            };
            return self
                .dispatch(PageEvent::DialogClosed { id, result, value })
                .or(Outcome::Changed);
        }
        Outcome::Changed
    }

    fn on_key(&mut self, key: Key) -> Outcome {
        if self.layout.too_small {
            if key.is_char('q') || key.ctrl_char('c') {
                return self.request_quit();
            }
            return Outcome::Consumed;
        }
        if let Some(d) = self.dialog.as_mut() {
            let out = d.on_key(&key, &mut self.focus, &self.ring);
            if d.result.is_some() {
                return self.close_dialog().or(out);
            }
            return out.or(Outcome::Consumed);
        }
        // the sidebar tree owns its navigation keys while focused; a typed
        // letter is type-ahead only when neither the page's shortcuts nor
        // the shell's own letters claim it (below)
        let mut type_ahead = false;
        if self.focus.is(NAV) && self.nav_filter.is_some() {
            return self.on_filter_key(&key);
        }
        if self.focus.is(NAV) && key.is_char('/') {
            self.nav_filter = Some(String::new());
            return Outcome::Changed;
        }
        if self.focus.is(NAV) {
            self.nav_reveal = true;
            type_ahead = matches!(key.code, KeyCode::Char(c) if !c.is_whitespace())
                && !key.chord()
                && self.nav.nav.action(&key).is_none()
                && junie_tui::core::keymap::lookup(&self.nav.keymap, &key).is_none();
            let leaf = self.nav.cursor_path().and_then(PageId::from_nav_path);
            if let Some(target) = leaf
                && matches!(key.code, KeyCode::Right | KeyCode::Char('l'))
                && key.plain()
            {
                // → on a page opens it and steps into it
                self.goto(target);
                self.focus.set(None);
                self.focus.next(&self.ring);
                if self.focus.is(NAV) {
                    self.focus.next(&self.ring);
                }
                return Outcome::Changed;
            }
            if !type_ahead {
                let out = self.nav_event(|nav| nav.on_key(&key));
                if out.consumed() {
                    return out;
                }
            }
        }
        // page first (editing widgets capture everything)
        let activating = matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) && key.plain();
        let was_editing = self.pages[self.page.index()].editing();
        let out = self.dispatch(PageEvent::Key(key));
        if out.consumed() {
            if activating
                && !was_editing
                && out == Outcome::Changed
                && let Some(f) = self.focus.current()
            {
                self.flash(f);
            }
            return out;
        }
        // the page had the first chance (Ctrl+C copies a selection there);
        // the quit keys go through the guard
        let kc = KeyContext {
            text_focus: was_editing,
            dirty: !self.quit_stakes().is_clean(),
            selection: false,
        };
        if self.quit_keys.classify(&key, kc) == Some(LeaveKey::Quit) {
            return self.request_quit();
        }
        if key.is_char('q') && kc.dirty {
            self.set_status("Unsaved changes · Ctrl+Q asks before quitting".into());
            return Outcome::Changed;
        }
        let shell_letter = matches!(key.code, KeyCode::Char('q' | 'i' | '?' | '[' | ']' | '0'));
        if type_ahead && !shell_letter {
            let out = self.nav_event(|nav| nav.on_key(&key));
            if out.consumed() {
                return out;
            }
        }
        // global keys
        match key.code {
            KeyCode::Tab => {
                self.focus.next(&self.ring);
                Outcome::Changed
            }
            KeyCode::BackTab => {
                self.focus.prev(&self.ring);
                Outcome::Changed
            }
            KeyCode::Char('?') => {
                self.open_help();
                Outcome::Changed
            }
            KeyCode::Char('i') if key.plain() => {
                self.inspector = !self.inspector;
                Outcome::Changed
            }
            // tree order: NAV_ENTRIES lists the pages group by group
            KeyCode::Char(']') => {
                let next = (self.page.index() + 1) % NAV_ENTRIES.len();
                self.goto(NAV_ENTRIES[next].id);
                Outcome::Changed
            }
            KeyCode::Char('[') => {
                let n = NAV_ENTRIES.len();
                let prev = (self.page.index() + n - 1) % n;
                self.goto(NAV_ENTRIES[prev].id);
                Outcome::Changed
            }
            KeyCode::Char('0') if key.plain() => {
                self.focus.focus(NAV);
                Outcome::Changed
            }
            KeyCode::Esc => {
                // Esc at top level: return focus to navigation
                if !self.focus.is(NAV) {
                    self.focus.focus(NAV);
                    Outcome::Changed
                } else {
                    Outcome::Consumed
                }
            }
            _ => Outcome::Ignored,
        }
    }

    /// The pointer's way to quit: the brand opens a confirmation.
    fn open_quit(&mut self) {
        let d = Dialog::confirm(
            QUIT_DIALOG,
            "Quit the showcase?",
            "Pages keep no state between runs.",
            "Quit",
        );
        self.open_dialog(d);
    }

    fn open_help(&mut self) {
        let text = "Tab / Shift+Tab   move keyboard focus\n\
                    ↑↓←→              move inside the focused control\n\
                    Enter / Space     activate · start editing\n\
                    Esc               cancel editing · back to navigation\n\
                    [ ]               previous / next page\n\
                    ← →               close / open a sidebar group\n\
                    a–z               jump to a sidebar row by name\n\
                    /                 filter the sidebar (Esc clears)\n\
                    0                 jump to navigation\n\
                    i                 toggle state inspector\n\
                    q                 quit\n\n\
                    Mouse: hover to preview, click to focus and activate, wheel to scroll, drag the scrollbar thumb.";
        let mut d = Dialog::confirm(HELP_DIALOG, "Keyboard & mouse", text, "Close");
        d.actions.remove(0);
        d.actions[0].kind = junie_tui::theme::ButtonKind::Secondary;
        d.cancel_index = Some(0);
        d.initial_focus = d.actions[0].id;
        d.width = 70;
        self.open_dialog(d);
    }

    /// A button release at `m.pos` over `hit`.
    fn on_release(&mut self, m: Mouse, hit: Option<WidgetId>) -> Outcome {
        let pressed = self.pointer.pressed;
        let id = match self.pointer.release(hit) {
            Release::Click(id) => id,
            Release::Outside => {
                if let Some(d) = self.dialog.as_mut() {
                    let out = d.on_click_outside();
                    if d.result.is_some() {
                        return self.close_dialog().or(out);
                    }
                    let dismiss = d.dismiss;
                    let now = self.now_ms();
                    junie_tui::ui::modal::refuse_outside_click(&mut self.pointer, dismiss, now);
                }
                return Outcome::Changed;
            }
            Release::Cancelled => {
                // the press-drag of a select ends on an option
                return match pressed {
                    Some(pressed) if self.dialog.is_none() => self
                        .dispatch(PageEvent::Release {
                            pressed,
                            pos: m.pos,
                        })
                        .or(Outcome::Changed),
                    _ => Outcome::Changed,
                };
            }
        };
        if let Some(d) = self.dialog.as_mut() {
            let out = d.on_click(id, m.pos, &mut self.focus);
            if d.result.is_some() {
                return self.close_dialog().or(out);
            }
            return out.or(Outcome::Changed);
        }
        if id == HEADER_HELP {
            self.open_help();
            return Outcome::Changed;
        }
        if id == HEADER_BRAND {
            self.open_quit();
            return Outcome::Changed;
        }
        if id == scrollbar::id_for(NAV) {
            // the press already moved the thumb
            return Outcome::Consumed;
        }
        if id == HEADER_INSPECT {
            self.inspector = !self.inspector;
            return Outcome::Changed;
        }
        if let Some((i, toggle)) = self.nav.locate(id) {
            self.focus.focus(NAV);
            self.nav_event(|nav| {
                if toggle {
                    nav.on_click_toggle(i)
                } else {
                    nav.on_click_row(i)
                }
            });
            return Outcome::Changed;
        }
        if id == NAV {
            // the sidebar below its last row
            self.focus.focus(NAV);
            return Outcome::Changed;
        }
        self.flash(id);
        // The page owns the click; a control the page did not
        // refocus is focused here so keyboard and pointer agree.
        let before = self.focus.current();
        self.pointer_clicks = self.pointer.count_click(id, m.pos, self.now_ms());
        let out = self.dispatch(PageEvent::Click { id, pos: m.pos });
        self.pointer_clicks = 0;
        if self.dialog.is_none()
            && self.pointer.config.focus_on == FocusOn::Release
            && self.focus.current() == before
            && self.ring.contains(id)
        {
            self.focus.focus(id);
        }
        out.or(Outcome::Changed)
    }

    fn on_mouse(&mut self, m: Mouse) -> Outcome {
        let before = self.focus.current();
        self.pointer_mods = m.mods;
        let out = self.route_mouse(m);
        self.pointer_mods = KeyModifiers::NONE;
        // focus a click moved shows only on controls that want a ring
        // after a click (FocusVisible)
        let pressed = m.kind == MouseKind::Down && self.pointer.pressed.is_some();
        if pressed || (m.kind == MouseKind::Up && self.focus.current() != before) {
            self.pointer.focused_by_click();
        }
        out
    }

    fn route_mouse(&mut self, m: Mouse) -> Outcome {
        self.pointer.pos = Some(m.pos);
        match m.kind {
            MouseKind::Move => self.pointer.on_move(&self.hits, m.pos),
            MouseKind::Drag => {
                if let Some(pressed) = self.pointer.on_drag(&self.hits, m.pos) {
                    if let Some(d) = self.dialog.as_mut() {
                        return d.on_drag(pressed, m.pos);
                    }
                    if pressed == scrollbar::id_for(NAV) {
                        return self.nav.on_scrollbar_drag(m.pos);
                    }
                    return self.dispatch(PageEvent::Drag {
                        pressed,
                        pos: m.pos,
                    });
                }
                Outcome::Ignored
            }
            MouseKind::Down => {
                let Some(id) = self.pointer.on_down(&self.hits, m.pos) else {
                    if self.dialog.is_some() {
                        return Outcome::Consumed;
                    }
                    self.pointer.dead_space(&mut self.focus);
                    return Outcome::Ignored;
                };
                // a press on a modal's own body is dead space inside it:
                // it ends the focused field's edit in place
                if self.dialog.is_some() {
                    self.pointer.surface_press(&self.hits, id, &mut self.focus);
                }
                if self.dialog.is_some() {
                    return Outcome::Changed;
                }
                if let Some((i, _)) = self.nav.locate(id) {
                    self.focus.focus(NAV);
                    self.nav.cursor = i;
                    return Outcome::Changed;
                }
                if id == scrollbar::id_for(NAV) {
                    return self.nav.on_scrollbar(m.pos).or(Outcome::Changed);
                }
                if self.pointer.config.focus_on == FocusOn::Press && self.ring.contains(id) {
                    self.focus.focus(id);
                }
                // the click count this press starts: the second press of a
                // double click selects a word in a field
                self.pointer_clicks = self.pointer.press_count(id, m.pos, self.now_ms());
                let out = self.dispatch(PageEvent::Press { id, pos: m.pos });
                self.pointer_clicks = 0;
                out.or(Outcome::Changed)
            }
            MouseKind::Up => {
                // a pass-through backdrop: the click closes the popup and
                // then lands on what is beneath it, as a click of its own
                let (hit, beneath) = self.pointer.on_up(&self.hits, m.pos);
                let out = self.on_release(m, hit);
                match beneath {
                    Some(below) => {
                        self.pointer.press(below);
                        out.or(self.on_release(m, Some(below)))
                    }
                    None => out,
                }
            }
            MouseKind::SecondaryDrag => self.pointer.on_secondary_drag(&self.hits, m.pos),
            // press, drag onto a context-menu row, release: the release
            // chooses it, delivered as a click on the row
            MouseKind::SecondaryUp => match self.pointer.on_secondary_up(&self.hits, m.pos) {
                Some(id) => {
                    self.pointer.press(id);
                    self.on_release(m, Some(id))
                }
                None => Outcome::Ignored,
            },
            MouseKind::Middle => Outcome::Ignored,
            MouseKind::Secondary => {
                let route = self.pointer.route_secondary(&self.hits, m.pos);
                // a dialog registers no light-dismiss backdrop: a secondary
                // click inside or outside it stays swallowed
                if self.dialog.is_some() {
                    return Outcome::Consumed;
                }
                match route {
                    SecondaryRoute::Nothing => Outcome::Ignored,
                    SecondaryRoute::Swallowed => Outcome::Consumed,
                    SecondaryRoute::Target(id) => {
                        self.dispatch(PageEvent::Secondary { id, pos: m.pos })
                    }
                    SecondaryRoute::Outside { backdrop, reopen } => {
                        // the open popup closes exactly as a primary click
                        // on its backdrop closes it ...
                        self.pointer.press(backdrop);
                        let out = self.on_release(m, Some(backdrop)).or(Outcome::Changed);
                        // ... then the control beneath gets the secondary
                        // click, so its menu opens at the new spot
                        match reopen {
                            Some(id) if self.dialog.is_none() => {
                                out.or(self.dispatch(PageEvent::Secondary { id, pos: m.pos }))
                            }
                            _ => out,
                        }
                    }
                }
            }
            MouseKind::WheelLeft | MouseKind::WheelRight => {
                let delta = if m.kind == MouseKind::WheelLeft {
                    -1
                } else {
                    1
                };
                if let Some(d) = self.dialog.as_mut() {
                    return d.on_wheel_h(delta);
                }
                let Some(id) = self.hits.hit_scroll(m.pos) else {
                    return Outcome::Ignored;
                };
                let out = self.dispatch(PageEvent::WheelH { id, delta });
                self.pointer.scrolled(m.pos, out);
                out
            }
            MouseKind::WheelUp | MouseKind::WheelDown => {
                let delta = self.pointer.wheel_delta(m.kind);
                if let Some(d) = self.dialog.as_mut() {
                    return d.on_wheel(delta);
                }
                // the notch goes to the innermost owner that takes it: a
                // field (or its label) inside a scrolling stack passes it on
                let mut out = Outcome::Ignored;
                for id in self.hits.scroll_stack(m.pos) {
                    out = if self.nav.owns(id) {
                        self.nav.on_wheel(delta)
                    } else {
                        self.dispatch(PageEvent::Wheel { id, delta })
                    };
                    if out != Outcome::Ignored {
                        break;
                    }
                }
                // the rows under the pointer change with the scroll: hover
                // is re-resolved once the new frame has registered its hits
                self.pointer.scrolled(m.pos, out);
                out
            }
        }
    }

    /// A key while the sidebar filters (`/`): letters narrow the tree to
    /// matching pages (their groups stay as context), ↑↓ move, Enter opens
    /// the page under the cursor (or the first match), Esc ends the filter.
    fn on_filter_key(&mut self, key: &Key) -> Outcome {
        let query = self.nav_filter.get_or_insert_with(String::new);
        match key.code {
            KeyCode::Esc => {
                self.end_filter();
                return Outcome::Changed;
            }
            KeyCode::Backspace => {
                query.pop();
            }
            KeyCode::Char(c) if !key.chord() => query.push(c),
            KeyCode::Enter => {
                let rows = self.nav.rows();
                let leaf = |i: usize| rows.get(i).and_then(|r| PageId::from_nav_path(&r.path));
                let target = leaf(self.nav.cursor).or_else(|| (0..rows.len()).find_map(leaf));
                if let Some(page) = target {
                    self.end_filter();
                    self.goto(page);
                }
                return Outcome::Changed;
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Home | KeyCode::End => {
                return self.nav_event(|nav| nav.on_key(key));
            }
            KeyCode::Tab | KeyCode::BackTab => {
                self.end_filter();
                return Outcome::Ignored;
            }
            _ => return Outcome::Consumed,
        }
        let q = query.clone();
        self.nav.set_filter(Some(&q));
        // the cursor lands on the first matching page, not its group
        if let Some(i) = (0..self.nav.rows().len())
            .find(|&i| PageId::from_nav_path(&self.nav.rows()[i].path).is_some())
        {
            self.nav.cursor = i;
        }
        Outcome::Changed
    }

    /// Leave type-to-filter: the full tree with the current page in view.
    fn end_filter(&mut self) {
        self.nav_filter = None;
        self.nav.set_filter(None);
        let path = self.page.nav_path();
        self.nav.reveal(&path);
        self.nav_reveal = true;
    }

    /// Run a sidebar tree operation: a page it activates becomes the
    /// current page, and once the user opens or closes the group navigation
    /// opened, it is theirs (it no longer closes on its own,
    /// [`NavExpand::CurrentGroup`]).
    fn nav_event(
        &mut self,
        f: impl FnOnce(&mut TreeView) -> (Outcome, Option<TreeAction>),
    ) -> Outcome {
        let auto_open =
            |nav: &TreeView, g: Option<usize>| g.map(|g| nav.expanded.contains(&vec![g]));
        let before = auto_open(&self.nav, self.nav_auto);
        let (out, ev) = f(&mut self.nav);
        if auto_open(&self.nav, self.nav_auto) != before {
            self.nav_auto = None;
        }
        if let Some(TreeAction::Activate(path)) = ev
            && let Some(page) = PageId::from_nav_path(&path)
        {
            self.goto(page);
            return Outcome::Changed;
        }
        out
    }

    /// Flash a widget as pressed (keyboard activation feedback).
    pub fn flash(&mut self, id: WidgetId) {
        let now = self.now_ms();
        self.pointer.flash(id, now);
    }

    /// Milliseconds on the pointer clock.
    fn now_ms(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }

    // ----------------------------------------------------------------- render

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        self.size = (area.width, area.height);
        let theme = self.theme;
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let interaction = self.interaction();
        let mut cursor = None;
        {
            let buf = frame.buffer_mut();
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            ctx.page_under_layer(
                self.dialog.is_some() || self.pages[self.page.index()].layer_open(),
            );
            self.draw(area, buf, &mut ctx);
            ctx.flush_overlays(buf);
            cursor = cursor.or(ctx.cursor);
        }
        self.hits = hits;
        self.ring = ring;
        self.pointer.refresh_hover(&self.hits);
        // keep focus valid against the freshly built ring
        if self.dialog.is_none() {
            if !self.layout.too_small
                && !self.focus.current().is_some_and(|c| self.ring.contains(c))
            {
                self.focus.set(self.ring.first());
            }
        } else {
            self.focus.ensure_valid(&self.ring);
        }
        if let Some(pos) = cursor {
            frame.set_cursor_position(pos);
        }
    }

    fn compute_layout(&self, area: Rect) -> ShellLayout {
        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            return ShellLayout {
                too_small: true,
                ..Default::default()
            };
        }
        let header = Rect::new(area.x, area.y, area.width, 1);
        let footer = Rect::new(area.x, area.bottom() - 1, area.width, 1);
        let body = Rect::new(
            area.x,
            area.y + 2,
            area.width,
            area.height.saturating_sub(4),
        );
        let wide = area.width >= 110;
        let sidebar_w = if wide { 24 } else { 19 };
        let inspector_w = if self.inspector && area.width >= 100 {
            30
        } else {
            0
        };
        // the sidebar card takes one column of the two-column gutter: its
        // own blank padding keeps the second, so its rows and the page
        // still sit two columns apart
        let sidebar = Rect::new(body.x, body.y, sidebar_w + 1, body.height);
        let main_x = body.x + sidebar_w + 2;
        let main_w = body
            .width
            .saturating_sub(sidebar_w + 2 + inspector_w + if inspector_w > 0 { 2 } else { 0 });
        let main = Rect::new(main_x, body.y, main_w, body.height);
        let inspector = if inspector_w > 0 {
            Rect::new(main.right() + 2, body.y, inspector_w, body.height)
        } else {
            Rect::ZERO
        };
        ShellLayout {
            header,
            sidebar,
            main,
            inspector,
            footer,
            too_small: false,
        }
    }

    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        fill(buf, area, t.base());
        let layout = self.compute_layout(area);
        self.layout = layout;
        if layout.too_small {
            self.draw_too_small(area, buf);
            return;
        }
        self.draw_header(layout.header, buf, ctx);
        self.draw_sidebar(layout.sidebar, buf, ctx);
        self.draw_main(layout.main, buf, ctx);
        if !layout.inspector.is_empty() {
            self.draw_inspector(layout.inspector, buf, ctx);
        }
        self.draw_footer(layout.footer, buf, ctx);
        if let Some(d) = self.dialog.as_mut() {
            d.render_in(
                area,
                junie_tui::widgets::dialog::body_bounds(area, 1, 1),
                buf,
                ctx,
            );
        }
    }

    fn draw_too_small(&self, area: Rect, buf: &mut Buffer) {
        let t = self.theme;
        let lines = [
            ("Junie Design system", t.title()),
            ("Terminal too small", t.secondary()),
            (
                &*format!(
                    "Need {MIN_WIDTH}×{MIN_HEIGHT}, have {}×{}",
                    area.width, area.height
                ),
                t.muted(),
            ),
            ("q Quit", t.faint()),
        ];
        let y0 = area.y + area.height.saturating_sub(5) / 2;
        for (i, (text, style)) in lines.iter().enumerate() {
            let w = junie_tui::ui::text::width(text) as u16;
            let x = area.x + area.width.saturating_sub(w) / 2;
            let y = y0 + i as u16 + if i == 3 { 1 } else { 0 };
            if y < area.bottom() {
                buf.set_string(x, y, text, style.bg(t.canvas));
            }
        }
    }

    fn draw_header(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        let mut x = area.x + 1;
        // the brand is the app menu's way to quit: it lifts on hover and
        // press, and keeps its look at rest
        let brand = Rect::new(x, area.y, 7, 1);
        let lifted = ctx.interaction.hovered(HEADER_BRAND) || ctx.interaction.pressed(HEADER_BRAND);
        let ladder = ctx.title_style(HEADER_BRAND, t.canvas);
        let (mark, name) = if lifted {
            (ladder.fg(t.accent), ladder)
        } else {
            (t.accent_fg(), t.title())
        };
        buf.set_string(x, area.y, "▪", mark);
        if lifted {
            buf.set_string(x + 1, area.y, " ", ladder);
        }
        x += 2;
        buf.set_string(x, area.y, "Junie", name);
        ctx.clickable(HEADER_BRAND, brand);
        x += 6;
        buf.set_string(x, area.y, "Design system", t.secondary());
        x += 14;
        let entry = &NAV_ENTRIES[self.page.index()];
        let crumb = if entry.group.is_empty() {
            format!("/ {}", entry.label)
        } else {
            format!("/ {} / {}", entry.group, entry.label)
        };
        buf.set_string(x, area.y, &crumb, t.muted());
        let crumb_w = junie_tui::ui::text::width(&crumb) as u16;
        // right side: capability + actions
        let cap = format!("{} · {}×{}", t.level.label(), self.size.0, self.size.1);
        let mut rx = area.right().saturating_sub(1);
        let help = " ? Help ";
        let insp = if self.inspector {
            " i Inspector · on "
        } else {
            " i Inspector "
        };
        for (label, id) in [(help, HEADER_HELP), (insp, HEADER_INSPECT)] {
            let w = junie_tui::ui::text::width(label) as u16;
            rx = rx.saturating_sub(w);
            // a header link: rest, hover and press from the shared ladder
            let style = ctx.title_style(id, t.canvas);
            buf.set_string(rx, area.y, label, style);
            ctx.clickable(id, Rect::new(rx, area.y, w, 1));
            rx = rx.saturating_sub(1);
        }
        let cw = junie_tui::ui::text::width(&cap) as u16;
        if rx > x + crumb_w + cw + 2 {
            buf.set_string(rx.saturating_sub(cw + 1), area.y, &cap, t.faint());
        }
    }

    fn draw_sidebar(&mut self, card: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        // the tree sits in a card like every other pane: its title carries
        // focus-within (the only `▎` stays on the cursor row), one column
        // of padding keeps the labels readable at the narrow width and
        // keeps the scrollbar off the card's edge
        let count = NAV_ENTRIES.len().to_string();
        let panel = junie_tui::widgets::panel::Panel::card(Some("Components"))
            .pad_x(1)
            .focused(ctx.interaction.focused(NAV))
            .meta(&count);
        let bg = panel.bg(&t);
        // the card's padding and title belong to the tree: a click there
        // focuses it and the wheel scrolls it (registered beneath the rows)
        ctx.clickable_under(NAV, card);
        ctx.scrollable(NAV, card);
        let Some(area) = panel.try_render(card, buf, &t) else {
            return;
        };
        // type-to-filter: the query takes the first row
        let area = match &self.nav_filter {
            Some(q) if area.height > 2 => {
                let row = Rect::new(area.x, area.y, area.width, 1);
                let fs = t.field_style(junie_tui::ui::ctx::VisualState {
                    editing: true,
                    ..Default::default()
                });
                fill(buf, row, fs);
                buf.set_string(row.x + 1, row.y, "/", fs.fg(t.text_muted));
                let shown = junie_tui::ui::text::truncate(q, row.width.saturating_sub(4) as usize);
                buf.set_string(row.x + 3, row.y, &shown, fs);
                if ctx.interaction.focused(NAV) {
                    ctx.set_cursor(ratatui::layout::Position::new(
                        row.x + 3 + junie_tui::ui::text::width(&shown) as u16,
                        row.y,
                    ));
                }
                Rect::new(area.x, area.y + 1, area.width, area.height - 1)
            }
            _ => area,
        };
        // the cursor row comes into view after navigation and on resize
        // (the wheel scrolls freely in between)
        let resized = self.nav.scroll.viewport_len != area.height as usize;
        self.nav.scroll.set_viewport(area.height as usize);
        if self.nav_reveal || resized {
            self.nav.scroll.ensure_visible(self.nav.cursor);
            self.nav_reveal = false;
        }
        // one focus stop; the scrollbar takes the last content column when
        // the tree overflows (layout audit #19), clear of the card's edge
        self.nav.render(area, buf, ctx, bg);
    }

    fn draw_main(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        let i = self.page.index();
        let page = self.pages[i].as_mut();
        // page title row
        buf.set_string(area.x, area.y, page.title(), t.title());
        let tw = junie_tui::ui::text::width(page.title()) as u16;
        if area.width > tw + 4 {
            let blurb = junie_tui::ui::text::truncate(page.blurb(), (area.width - tw - 3) as usize);
            buf.set_string(area.x + tw + 2, area.y, &blurb, t.muted());
        }
        let body = Rect::new(
            area.x,
            area.y + 2,
            area.width,
            area.height.saturating_sub(2),
        );
        page.render(body, buf, ctx);
    }

    fn draw_inspector(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        let panel = junie_tui::widgets::panel::Panel::card(Some("State"));
        let inner = panel.render(area, buf, &t);
        let bg = t.surface;
        let page = &self.pages[self.page.index()];
        let mode = if self.dialog.is_some() {
            "MODAL"
        } else if page.editing() {
            "EDIT"
        } else {
            "NAV"
        };
        let fmt_rect = |r: Option<Rect>| match r {
            Some(r) => format!("{}·{} {}×{}", r.x, r.y, r.width, r.height),
            None => "—".to_owned(),
        };
        let focus_area = self.focus.current().and_then(|f| self.hits.area_of(f));
        let hover_area = self.pointer.hover.and_then(|h| self.hits.area_of(h));
        let rows: Vec<(&str, String)> = vec![
            ("mode", mode.to_owned()),
            ("focus", fmt_rect(focus_area)),
            (
                "hover",
                if self.pointer.hover_suppressed {
                    "suppressed".into()
                } else {
                    fmt_rect(hover_area)
                },
            ),
            (
                "pressed",
                fmt_rect(self.pointer.pressed.and_then(|p| self.hits.area_of(p))),
            ),
            (
                "mouse",
                self.pointer
                    .pos
                    .map(|p| format!("{}·{}", p.x, p.y))
                    .unwrap_or("—".into()),
            ),
            ("last key", self.last_key.clone().unwrap_or("—".into())),
            (
                "focus ring",
                format!("{} stops", self.ring.reachable().len()),
            ),
            ("hit regions", format!("{}", self.hits_len())),
            ("tick", format!("{}", self.tick)),
            ("colors", t.level.label().to_owned()),
        ];
        for (i, (k, v)) in rows.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            buf.set_string(inner.x, y, format!("{k:<11}"), t.muted().bg(bg));
            let vs = t.primary().bg(bg);
            buf.set_string(
                inner.x + 12,
                y,
                junie_tui::ui::text::truncate(v, inner.width.saturating_sub(12) as usize),
                vs,
            );
        }
        let _ = ctx;
    }

    fn hits_len(&self) -> usize {
        self.hits.len()
    }

    fn draw_footer(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        let page = &self.pages[self.page.index()];
        let mut hints: Vec<(&'static str, &'static str)> = Vec::new();
        let mut dialog_hints = None;
        if let Some(d) = self.dialog.as_ref() {
            dialog_hints = Some(d.hints(self.focus.current()));
        } else if self.focus.is(NAV) {
            if self.nav_filter.is_some() {
                hints.push(("type", "Filter"));
                hints.push(("Enter", "Open"));
                hints.push(("Esc", "Clear"));
            }
            hints.push(("↑↓", "Move"));
            let on_group = self
                .nav
                .cursor_path()
                .is_some_and(|p| PageId::from_nav_path(p).is_none());
            if on_group {
                hints.push(("←→", "Fold"));
                hints.push(("Enter", "Toggle"));
            } else if self.nav_filter.is_none() {
                hints.push(("Enter", "Open"));
            }
            if self.nav_filter.is_none() {
                hints.push(("/", "Filter"));
            }
            hints.push(("Tab", "Into page"));
            hints.push(("q", "Quit"));
        } else {
            for (k, v) in page.hints(self.focus.current()) {
                hints.push((k, v));
            }
            // the focused cursor row is cut: name the reveal key
            junie_tui::ui::reveal::advertise(&mut hints, ctx.reveal_hint, |h| h.0, |k, a| (k, a));
            if !page.editing() && !hints.iter().any(|(key, _)| *key == "Tab") {
                hints.push(("Tab", "Next"));
            }
        }
        let hints: Vec<_> = dialog_hints.unwrap_or_else(|| {
            hints
                .into_iter()
                .map(|(key, action)| junie_tui::widgets::keyhint::hint(key, action))
                .collect()
        });
        let badge = (page.editing() && self.dialog.is_none()).then_some(("EDIT", BadgeKind::Edit));
        junie_tui::widgets::keyhint::render(
            area,
            buf,
            &t,
            &hints,
            badge,
            self.status.as_ref().map(|(message, _)| message.as_str()),
        );
        let _ = ctx;
    }
}

fn describe_key(k: &Key) -> String {
    let mut s = String::new();
    if k.ctrl() {
        s.push_str("Ctrl+");
    }
    if k.alt() {
        s.push_str("Alt+");
    }
    if k.shift() && !matches!(k.code, KeyCode::Char(_)) {
        s.push_str("Shift+");
    }
    match k.code {
        KeyCode::Char(' ') => s.push_str("Space"),
        KeyCode::Char(c) => s.push(c),
        KeyCode::BackTab => s.push_str("Shift+Tab"),
        other => s.push_str(&format!("{other:?}")),
    }
    s
}

#[cfg(test)]
mod click_regressions {
    use super::*;
    use junie_tui::core::event::Mouse;
    use ratatui::layout::Position;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn minimum_sidebar_reveals_last_page_and_wheel_does_not_move_focus() {
        let mut app = App::new(Theme::junie());
        // every group open: the tree overflows the minimum sidebar
        app.set_nav_expand(NavExpand::All);
        app.goto(PageId::TaskRunner);
        let mut term = Terminal::new(TestBackend::new(72, 20)).unwrap();
        term.draw(|frame| app.render(frame)).unwrap();
        let last = app.nav.rows().len() - 1;
        assert_eq!(app.nav.cursor, last, "the current page is the last row");
        assert!(app.hits.area_of(app.nav.row_id(last)).is_some());
        assert!(app.nav.scroll.offset > 0);
        let focus = app.focus.current();
        let cursor = app.nav.cursor;
        app.handle(Input::Mouse(Mouse::new(
            MouseKind::WheelUp,
            Position::new(2, 4),
        )));
        let offset = app.nav.scroll.offset;
        term.draw(|frame| app.render(frame)).unwrap();
        assert_eq!(
            app.nav.scroll.offset, offset,
            "render must not undo wheel scrolling"
        );
        assert_eq!(app.focus.current(), focus);
        assert_eq!(app.nav.cursor, cursor);
        app.handle(Input::Key(Key {
            code: KeyCode::End,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        }));
        term.draw(|frame| app.render(frame)).unwrap();
        assert!(app.hits.area_of(app.nav.row_id(last)).is_some());
        let track = app.hits.area_of(scrollbar::id_for(NAV)).unwrap();
        let before = app.nav.scroll.offset;
        for kind in [MouseKind::Down, MouseKind::Up] {
            app.handle(Input::Mouse(Mouse::new(
                kind,
                Position::new(track.x, track.y),
            )));
            term.draw(|frame| app.render(frame)).unwrap();
        }
        assert!(
            app.nav.scroll.offset < before,
            "a track click pages toward the pointer"
        );
        assert_eq!(app.focus.current(), focus);
    }

    #[test]
    fn one_completed_click_focuses_and_edits_a_field() {
        for page in [
            PageId::Inputs,
            PageId::TextAreas,
            PageId::Forms,
            PageId::Editor,
        ] {
            let mut app = App::new(Theme::junie());
            app.goto(page);
            let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
            term.draw(|frame| app.render(frame)).unwrap();
            let id = app.ring.next(Some(NAV)).unwrap();
            let area = app.hits.area_of(id).unwrap();
            let pos = Position::new(area.x + 2, area.y + area.height / 2);
            // the press alone focuses nothing; the completed click focuses
            // and edits in one go, and a second click only keeps editing
            app.handle(Input::Mouse(Mouse::new(MouseKind::Down, pos)));
            term.draw(|frame| app.render(frame)).unwrap();
            assert!(!app.pages[page.index()].editing(), "{page:?}: press alone");
            for _click in 0..2 {
                app.handle(Input::Mouse(Mouse::new(MouseKind::Up, pos)));
                term.draw(|frame| app.render(frame)).unwrap();
                assert_eq!(app.focus.current(), Some(id), "{page:?}");
                assert!(
                    app.pages[page.index()].editing(),
                    "{page:?}: one click edits"
                );
                app.handle(Input::Mouse(Mouse::new(MouseKind::Down, pos)));
            }
        }
    }
}
