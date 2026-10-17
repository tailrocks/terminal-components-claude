//! Showcase pages. Each page owns its widgets and routes events to them.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::FocusChange;
use junie_tui::core::id::WidgetId;
use junie_tui::core::pointer::PointerEvent;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layer::{Layer, LayerTag};
use junie_tui::ui::layers::LayerResult;
use junie_tui::widgets::dialog::{Dialog, DialogResult};
use junie_tui::widgets::form_dialog::{FieldValue, FormValues};
use junie_tui::widgets::help::HelpOverlay;

pub mod accounts_usage;
pub mod activity;
pub mod adapter;
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
    /// A pointer event the shell's router resolved to its owner
    /// ([`junie_tui::core::pointer::PointerRouter`]): a press, click,
    /// drag, release, middle or secondary click, or a wheel notch on
    /// either axis. Pages hand it to their widgets (`widget.route(ev)`).
    Pointer(PointerEvent),
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

/// What a layer the shell holds reports to the page that opened it
/// ([`junie_tui::shell::Page::on_layer_result`]).
#[derive(Debug, Clone)]
pub enum LayerRes {
    /// A library layer's own result (a keyboard reference closed).
    Lib(LayerResult),
    /// A dialog the page opened closed: its id, how it closed and a
    /// prompt's text. The shell reads them off the closed dialog.
    Dialog {
        id: WidgetId,
        result: DialogResult,
        value: Option<String>,
    },
    /// Any other layer a page opened (a picker, a form) reported to it.
    Page(Report),
}

/// What a layer a page opened reported: its result, what the page reads
/// back from the layer, and whether it closed. The shell reads them off
/// the layer, so the page never reaches into the layer stack.
#[derive(Debug, Clone)]
pub struct Report {
    /// The result; `None` when the layer was dismissed without one.
    pub result: Option<LayerResult>,
    /// A form's values as they were when it reported.
    pub values: Option<FormValues>,
    /// The closed layer when the report closed it (a picker's rows);
    /// `None` for an event (the layer stays open).
    pub closed: Option<Closed>,
}

impl Report {
    /// The text of form field `name` (empty when there is none).
    pub fn text(&self, name: &str) -> String {
        self.values
            .iter()
            .flatten()
            .find_map(|(n, v)| match v {
                FieldValue::Text(t) if n == name => Some(t.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }
}

/// A layer the shell closed, handed to the page that opened it.
#[derive(Clone)]
pub struct Closed(pub std::rc::Rc<dyn Layer<LayerRes>>);

impl Closed {
    /// The closed layer as its concrete type, if it is an `L`.
    pub fn get<L: Layer<LayerRes>>(&self) -> Option<&L> {
        let any: &dyn std::any::Any = &*self.0;
        any.downcast_ref()
    }
}

impl std::fmt::Debug for Closed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Closed")
    }
}

impl From<LayerResult> for LayerRes {
    fn from(r: LayerResult) -> Self {
        Self::Lib(r)
    }
}

/// The showcase's own requests: the `Go` payload of the library
/// [`Request`].
#[derive(Debug)]
pub enum Go {
    /// Set a live option ([`CxExt::set_option`]).
    SetOption(crate::options::OptSet),
}

/// Things a page may ask the shell to do (the library's
/// [`junie_tui::shell::Request`]).
pub type Request = junie_tui::shell::Request<LayerRes, (), Go>;

/// The context a page handles an event in (the library's
/// [`junie_tui::shell::Cx`]).
pub type Cx<'a> = junie_tui::shell::Cx<'a, LayerRes, (), Go>;

/// The tag of a dialog a page opens on the shell's layer stack.
pub const DIALOG_TAG: &str = "dialog";
/// The tag of a keyboard reference on the shell's layer stack.
pub const HELP_TAG: &str = "help";

/// The showcase's requests, spelled as [`Cx`] methods.
pub trait CxExt {
    /// Open `d` as the shell's modal dialog; its result comes back as
    /// [`PageEvent::DialogClosed`].
    fn open_dialog(&mut self, d: Dialog);
    /// Open `help` as the shell's modal reference. The shell refuses it
    /// over another modal and gives focus back when it closes.
    fn open_help(&mut self, help: HelpOverlay);
    /// Set a live option (a page's Options section): the shell applies it
    /// through `App::apply_opt`, the path `--opt` and the Inspector take.
    fn set_option(&mut self, set: crate::options::OptSet);
}

impl CxExt for Cx<'_> {
    fn open_dialog(&mut self, d: Dialog) {
        self.open(d, LayerTag::new(DIALOG_TAG));
    }
    fn open_help(&mut self, help: HelpOverlay) {
        self.open(help, LayerTag::new(HELP_TAG));
    }
    fn set_option(&mut self, set: crate::options::OptSet) {
        self.go(Go::SetOption(set));
    }
}

pub type Hint = (&'static str, &'static str);

pub trait Page {
    fn title(&self) -> &'static str;
    fn blurb(&self) -> &'static str;
    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx);
    /// The natural size of the Playground when offered `avail` (plan §1.4
    /// `Measure`): the template lays the Playground out at it and scrolls
    /// on the axes where it is larger than the page, so nothing is cut
    /// off. Default `avail`: the Playground lays itself out in whatever
    /// room it gets.
    fn measure(&self, avail: ratatui::layout::Size, _ctx: &RenderCtx) -> ratatui::layout::Size {
        avail
    }
    fn handle(&mut self, ev: &PageEvent, cx: &mut Cx) -> Outcome;
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
        self.template().is_some_and(|t| t.popup_open())
    }
    /// A key for the top layer this page opened, before the layer sees
    /// it: the page's own key over its layer (a picker's retry). Default:
    /// none, the layer takes every key.
    fn on_layer_key(&mut self, _key: &Key, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }
    /// A layer this page opened (not a dialog) reported `report` under
    /// `tag` ([`LayerRes::Page`]).
    fn on_layer(&mut self, _tag: &LayerTag, _report: Report, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }
    /// The value index of the page option `key` (one of
    /// `crate::options::page_options` for this page), `None` for a key the
    /// page does not have.
    fn option_value(&self, _key: &str) -> Option<usize> {
        None
    }
    /// Set the page option `key` to value index `value`, rebuilding the
    /// widget through its builder (the Inspector's Options tab and `--opt`).
    fn set_option(&mut self, _key: &str, _value: usize) {}
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
            match ev {
                super::PageEvent::Pointer(p) => self.state.pointer(self.id, p),
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
