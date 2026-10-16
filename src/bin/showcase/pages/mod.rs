//! Showcase pages. Each page owns its widgets and routes events to them.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusChange, FocusRing};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::dialog::{Dialog, DialogResult};

pub mod accounts_usage;
pub mod activity;
pub mod breadcrumbs_rails;
pub mod buttons;
pub mod celleditors;
pub mod checks_radios;
pub mod chips;
pub mod chrome;
pub mod columns;
pub mod connections;
pub mod dialog_kinds;
pub mod dialogs;
pub mod diff;
pub mod disk_cleanup;
pub mod editable;
pub mod editor;
pub mod empty_error;
pub mod file_browser;
pub mod finder;
pub mod focus_navigation;
pub mod forms;
pub mod grid;
pub mod help_keys;
pub mod inputs;
pub mod launch_cockpit;
pub mod lists;
pub mod master_detail;
pub mod menus;
pub mod meters;
pub mod notices;
pub mod overview;
pub mod pane_tree;
pub mod panels;
pub mod pickers;
pub mod popups_tooltips;
pub mod progress;
pub mod query_workbench;
pub mod quit_leave;
pub mod readonly_report;
pub mod row_menu;
pub mod scrolling;
pub mod selectable;
pub mod settings;
pub mod sidebars;
pub mod splits_stacks;
pub mod status_hints;
pub mod stub;
pub mod tables;
pub mod tabs;
pub mod taskrunner;
pub mod template;
pub mod terminal;
pub mod text_utilities;
pub mod textareas;
pub mod toolbars;
pub mod trees;
pub mod workspaces;

/// Event delivered to a page after the app has resolved hit-testing.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)] // DialogClosed carries the prompt text; events are short-lived
pub enum PageEvent {
    Key(Key),
    /// Text pasted while a control is editing.
    Paste(String),
    /// Pointer down at its original position, before any drag motion.
    Press {
        id: WidgetId,
        pos: Position,
    },
    /// Completed click (down and up on the same id).
    Click {
        id: WidgetId,
        pos: Position,
    },
    /// Secondary (right) button on `id`: a context action.
    Secondary {
        id: WidgetId,
        pos: Position,
    },
    /// Pointer moved while the button is held; `pressed` is the id where
    /// the press started.
    Drag {
        pressed: WidgetId,
        pos: Position,
    },
    /// The button went up away from `pressed`, where the press started
    /// (no click: the press and the release landed on different targets).
    /// A select chooses the option a press-drag ended on.
    Release {
        pressed: WidgetId,
        pos: Position,
    },
    /// Wheel over `id` (a scroll container).
    Wheel {
        id: WidgetId,
        delta: i32,
    },
    /// Horizontal wheel over `id`: columns of a table or grid, the text of
    /// an editor.
    WheelH {
        id: WidgetId,
        delta: i32,
    },
    Tick,
    /// A control lost focus (or a click on dead space ended its edit):
    /// the page that owns `from` ends the edit through the control's
    /// `on_blur`. Delivered to every page, so a page left while one of its
    /// fields was editing still commits it.
    Blur(FocusChange),
    DialogClosed {
        id: WidgetId,
        result: DialogResult,
        /// Text entered in a prompt dialog, if any.
        value: Option<String>,
    },
}

/// Things a page may ask the app to do.
#[derive(Debug)]
pub enum Request {
    OpenDialog(Box<Dialog>),
    Status(String),
}

pub struct PageCtx<'a> {
    pub focus: &'a mut Focus,
    pub ring: &'a FocusRing,
    pub requests: Vec<Request>,
    /// Modifier keys held with the pointer event being delivered (Shift
    /// extends a selection, Ctrl/Cmd toggles); empty for keys and ticks.
    pub mods: ratatui::crossterm::event::KeyModifiers,
    /// Multi-click count of the click being delivered (1 single, 2 double,
    /// 3 triple; [`junie_tui::core::pointer::PointerState::count_click`]), or
    /// of the click a press starts (`press_count`); 0 for every other event.
    pub clicks: u8,
}

impl PageCtx<'_> {
    pub fn focus_next(&mut self) {
        self.focus.next(self.ring);
    }
    pub fn focus_prev(&mut self) {
        self.focus.prev(self.ring);
    }
    pub fn status(&mut self, s: impl Into<String>) {
        self.requests.push(Request::Status(s.into()));
    }
    pub fn open(&mut self, d: Dialog) {
        self.requests.push(Request::OpenDialog(Box::new(d)));
    }
}

pub type Hint = (&'static str, &'static str);

pub trait Page {
    fn title(&self) -> &'static str;
    fn blurb(&self) -> &'static str;
    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx);
    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome;
    /// Contextual key hints for the footer given the focused widget.
    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint>;
    /// True while a control on this page is in edit mode.
    fn editing(&self) -> bool {
        false
    }
    /// What quitting would lose on this page (keyboard audit #21): the
    /// shell's quit keys ask first when any page is not clean.
    fn leave_stakes(&self) -> junie_tui::ui::leave::Stakes {
        junie_tui::ui::leave::Stakes::Clean
    }
    /// True when the page needs periodic ticks (spinners, progress).
    fn animating(&self) -> bool {
        false
    }
    /// The page as `Any`, so tests can reach its widgets by type.
    #[allow(dead_code)]
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
    /// True while the page draws a layer above itself (a modal picker, an
    /// open menu that owns the keyboard): the shell then renders the page
    /// with no focus indicator ([`RenderCtx::page_under_layer`]).
    fn layer_open(&self) -> bool {
        false
    }
    /// The page's Playground | Options | States template, for pages built
    /// on it ([`template::Template`]).
    #[allow(dead_code)] // tests and the coverage gate read it
    fn template(&self) -> Option<&template::Template> {
        None
    }
    #[allow(dead_code)] // tests and the coverage gate drive it
    fn template_mut(&mut self) -> Option<&mut template::Template> {
        None
    }
}

/// Shared helpers for laying out demo sections.
pub mod layout {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    use junie_tui::theme::Theme;

    /// Small muted label above a control.
    pub fn caption(x: u16, y: u16, buf: &mut Buffer, t: &Theme, text: &str, bg: Color) {
        buf.set_string(x, y, text, t.muted().bg(bg));
    }

    /// Split vertically into rows with fixed heights; the last takes the rest.
    pub fn rows(area: Rect, heights: &[u16]) -> Vec<Rect> {
        let mut y = area.y;
        let mut out = Vec::new();
        for (i, &h) in heights.iter().enumerate() {
            let last = i == heights.len() - 1;
            let h = if last {
                area.bottom().saturating_sub(y)
            } else {
                h.min(area.bottom().saturating_sub(y))
            };
            out.push(Rect::new(area.x, y, area.width, h));
            y = y.saturating_add(h);
        }
        out
    }

    /// A page's sections as a measured [`Stack`]: on a short terminal the
    /// page scrolls by whole sections (wheel, scrollbar, focus) instead of
    /// squeezing every section to a row or two.
    ///
    /// [`Stack`]: junie_tui::ui::layout::Stack
    pub struct Sections {
        pub id: junie_tui::core::id::WidgetId,
        pub stack: junie_tui::ui::layout::Stack,
        pub state: junie_tui::ui::layout::StackScroll,
    }

    impl Sections {
        /// One blank row between sections, like the fixed layouts had.
        pub fn new(id: junie_tui::core::id::WidgetId) -> Self {
            Self {
                id,
                stack: junie_tui::ui::layout::Stack::default().gap(1),
                state: Default::default(),
            }
        }

        /// Lay the sections out and draw the page scrollbar. `focus` is
        /// the section holding keyboard focus.
        pub fn layout(
            &mut self,
            area: Rect,
            sections: &[junie_tui::ui::layout::StackChild],
            focus: Option<usize>,
            buf: &mut Buffer,
            ctx: &mut junie_tui::ui::ctx::RenderCtx,
        ) -> junie_tui::ui::layout::StackLayout {
            let l = self.stack.layout(area, sections, &mut self.state, focus);
            junie_tui::ui::layout::Stack::chrome(&l, area, buf, ctx, self.id, &self.state);
            l
        }

        /// The page scroll's own events: the wheel over the page and the
        /// scrollbar. `None` for anything else.
        pub fn handle(&mut self, ev: &super::PageEvent) -> Option<junie_tui::core::event::Outcome> {
            use junie_tui::core::event::Outcome;
            use junie_tui::widgets::scrollbar::id_for;
            let changed = |moved: bool| {
                if moved {
                    Outcome::Changed
                } else {
                    Outcome::Consumed
                }
            };
            match ev {
                super::PageEvent::Wheel { id, delta } if *id == self.id => {
                    Some(changed(self.state.on_wheel(*delta)))
                }
                super::PageEvent::Press { id, pos } | super::PageEvent::Click { id, pos }
                    if *id == id_for(self.id) =>
                {
                    Some(changed(self.state.on_scrollbar(*pos)))
                }
                super::PageEvent::Drag { pressed, pos } if *pressed == id_for(self.id) => {
                    Some(changed(self.state.on_scrollbar_drag(*pos)))
                }
                _ => None,
            }
        }
    }

    /// Two columns with a gap; if too narrow, stack vertically.
    pub fn columns(area: Rect, left_w: u16, gap: u16) -> (Rect, Rect) {
        if area.width < left_w + gap + 20 {
            let h = area.height / 2;
            return (
                Rect::new(area.x, area.y, area.width, h),
                Rect::new(area.x, area.y + h, area.width, area.height - h),
            );
        }
        (
            Rect::new(area.x, area.y, left_w, area.height),
            Rect::new(
                area.x + left_w + gap,
                area.y,
                area.width - left_w - gap,
                area.height,
            ),
        )
    }
}
