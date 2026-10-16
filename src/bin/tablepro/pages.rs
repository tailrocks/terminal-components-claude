//! tablepro's two pages on the library [`Page`] contract: Connections
//! (the saved connections and their form) and the Workbench of one open
//! connection. The app routes input, rendering and hints to the shown one
//! through [`Page`]; a connection that finishes connecting reports
//! [`Act::Connected`] instead of a private event.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::FocusChange;
use junie_tui::core::id::WidgetId;
use junie_tui::core::pointer::PointerEvent;
use junie_tui::shell::{Page, PageTypes};
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::leave::Stakes;
use junie_tui::widgets::keyhint::Hint;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::app::{Act, Cx, ModalResult};
use crate::connections::{ConnEvent, ConnectionsScreen};
use crate::model::History;
use crate::workbench::Workbench;

/// The types a tablepro page works with: the pages read and add to the
/// query history.
pub struct Tablepro;

impl PageTypes for Tablepro {
    type State = History;
    type Msg = ();
    type Result = ModalResult;
    type LayerState = ();
    type Go = Act;
}

impl Page<Tablepro> for ConnectionsScreen {
    fn on_key(&mut self, key: &Key, _h: &mut History, cx: &mut Cx) -> Outcome {
        ConnectionsScreen::on_key(self, key, cx).0
    }

    fn pointer(&mut self, ev: &PointerEvent, _h: &mut History, cx: &mut Cx) -> Outcome {
        ConnectionsScreen::pointer(self, ev, cx).0
    }

    fn on_paste(&mut self, text: &str, _h: &mut History, _cx: &mut Cx) -> Outcome {
        ConnectionsScreen::on_paste(self, text)
    }

    fn on_blur(&mut self, ch: FocusChange, _h: &mut History, _cx: &mut Cx) -> Outcome {
        ConnectionsScreen::on_blur(self, ch)
    }

    /// A connection that finished connecting opens its workbench.
    fn on_tick(&mut self, _h: &mut History, cx: &mut Cx) -> Outcome {
        match self.tick() {
            Some(ConnEvent::Connected(i)) => {
                cx.go(Act::Connected(i));
                Outcome::Changed
            }
            None => Outcome::Ignored,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _h: &History) {
        ConnectionsScreen::render(self, area, buf, ctx);
    }

    fn hints(&self, focus: Option<WidgetId>, _h: &History) -> Vec<Hint> {
        ConnectionsScreen::hints(self, focus)
    }

    fn crumb(&self, _h: &History) -> String {
        "Connections".into()
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        None
    }

    fn is_editing(&self) -> bool {
        ConnectionsScreen::is_editing(self)
    }

    fn animating(&self, _h: &History) -> bool {
        ConnectionsScreen::animating(self)
    }

    fn stakes(&mut self, _h: &History) -> Stakes {
        self.form_stakes()
    }
}

impl Page<Tablepro> for Workbench {
    fn on_key(&mut self, key: &Key, h: &mut History, cx: &mut Cx) -> Outcome {
        Workbench::on_key(self, key, cx, h)
    }

    fn pointer(&mut self, ev: &PointerEvent, h: &mut History, cx: &mut Cx) -> Outcome {
        Workbench::pointer(self, ev, cx, h)
    }

    fn on_paste(&mut self, text: &str, _h: &mut History, _cx: &mut Cx) -> Outcome {
        Workbench::on_paste(self, text)
    }

    fn on_blur(&mut self, ch: FocusChange, _h: &mut History, _cx: &mut Cx) -> Outcome {
        Workbench::on_blur(self, ch)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, h: &History) {
        Workbench::render(self, area, buf, ctx, h);
    }

    fn hints(&self, focus: Option<WidgetId>, _h: &History) -> Vec<Hint> {
        Workbench::hints(self, focus)
    }

    fn crumb(&self, _h: &History) -> String {
        self.connection.name.clone()
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Workbench::primary_focus(self)
    }

    fn is_editing(&self) -> bool {
        Workbench::is_editing(self)
    }

    fn animating(&self, _h: &History) -> bool {
        Workbench::animating(self)
    }
}
