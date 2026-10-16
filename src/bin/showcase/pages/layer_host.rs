//! The page-level layer stack the Overlays pages share: library layers
//! (`Picker`, `FormDialog`, `ChoiceDialog`, `InfoDialog`, `HelpOverlay`,
//! `Dialog`) opened on one [`ShellLayers`], drawn over the page and given
//! the page's events while one is open.
//!
//! This is the routing every app does for its `ShellLayers`, kept once for
//! the showcase pages (Pickers, Dialog kinds) rather than per page. It is
//! wiring only: what a key, click or dismissal does is the layer's own.
//! Plan S6/S7 move it into the library (`PointerRouter` and
//! `Request::Open`), and the pages then hand their layers to the shell.
//!
//! Every click a page receives while a layer is open belongs to that
//! layer: the layer's modal barrier keeps the page below out of the hit
//! map, so a click there never reaches the page (and so never dismisses
//! the layer: plan S5 deviation 5, for S6).

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Position;

use super::{PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::focus::Focus;
use junie_tui::core::id::WidgetId;
use junie_tui::core::intent::DismissReason;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::layer::{Dismissed, Layer, LayerCx, LayerEntry, LayerTag, ShellLayers, Taken};
use junie_tui::ui::layers::LayerResult;
use junie_tui::widgets::keyhint::Hint as KeyHint;

/// What an open layer reported to the page.
pub struct Report {
    /// The tag it was opened with.
    pub tag: LayerTag,
    pub result: LayerResult,
    /// The closed entry when the result closed it (read a form's values
    /// or a picker's rows from it); `None` for an event.
    pub closed: Option<LayerEntry<LayerResult>>,
}

#[derive(Default)]
pub struct LayerHost {
    pub layers: ShellLayers<LayerResult>,
}

impl LayerHost {
    /// Open `layer` on top; focus moves into it and comes back when it
    /// closes.
    pub fn open(&mut self, layer: Box<dyn Layer<LayerResult>>, tag: LayerTag, focus: &mut Focus) {
        self.layers.open(layer, tag, (), focus);
    }

    pub fn is_open(&self) -> bool {
        !self.layers.is_empty()
    }

    /// The top layer's tag.
    pub fn top_tag(&self) -> Option<&LayerTag> {
        self.layers.top().map(|e| &e.tag)
    }

    /// Whether the page under the stack shows no focus indicator.
    pub fn hides_page_focus(&self) -> bool {
        self.layers.hides_page_focus()
    }

    /// Draw every open layer over the page, bottom first.
    pub fn render(&mut self, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let screen = *buf.area();
        for e in self.layers.iter_mut() {
            e.layer.render(screen, buf, ctx, &());
        }
    }

    /// The top layer's footer hints, built from its state; `None` when
    /// it has none (the page's own hints then show).
    pub fn hints(&self, focus: Option<WidgetId>) -> Option<Vec<KeyHint>> {
        let hints = self.layers.top()?.layer.hints(focus);
        (!hints.is_empty()).then_some(hints)
    }

    /// Route `ev` to the top layer. `None` when no layer is open (the page
    /// handles the event); otherwise the outcome and what the layers
    /// reported, in order.
    pub fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Option<(Outcome, Vec<Report>)> {
        let top = self.layers.top_mut()?;
        let mut lcx = LayerCx {
            focus: cx.focus,
            ring: cx.ring,
            app: &(),
        };
        let mut reports = Vec::new();
        let out = match ev {
            PageEvent::Key(key) => {
                let out = top.layer.on_key(key, &mut lcx);
                // a layer that has no use for Esc leaves it to its policy
                if out == Outcome::Ignored && key.code == KeyCode::Esc && key.mods.is_empty() {
                    self.dismiss(DismissReason::Esc, cx.focus, &mut reports)
                } else {
                    out
                }
            }
            PageEvent::Paste(text) => top.layer.on_paste(text),
            PageEvent::Press { id, pos } => top.layer.on_press(*id, *pos, cx.clicks),
            PageEvent::Click { id, pos } => {
                top.layer
                    .on_click_count(*id, *pos, cx.clicks.max(1), &mut lcx)
            }
            PageEvent::Drag { pressed, pos } => top.layer.on_drag(*pressed, *pos),
            PageEvent::Release { pressed, pos } => top.layer.on_release(*pressed, *pos),
            PageEvent::Wheel { delta, .. } => top.layer.on_wheel(*delta, Position::default()),
            PageEvent::WheelH { delta, .. } => top.layer.on_wheel_h(*delta, Position::default()),
            PageEvent::Blur(change) => top.layer.on_blur(*change),
            PageEvent::Tick => top.layer.on_tick(&()),
            PageEvent::Middle { .. } | PageEvent::Secondary { .. } => Outcome::Consumed,
            PageEvent::DialogClosed { .. } => return None,
        };
        self.drain(cx.focus, &mut reports);
        Some((out, reports))
    }

    /// Dismiss the top layer for `reason` under its own policy.
    pub fn dismiss(
        &mut self,
        reason: DismissReason,
        focus: &mut Focus,
        reports: &mut Vec<Report>,
    ) -> Outcome {
        match self.layers.dismiss(reason, focus) {
            Dismissed::None => Outcome::Ignored,
            Dismissed::Refused => Outcome::Consumed,
            Dismissed::Closed(entry, result) => {
                if let Some(result) = result {
                    reports.push(Report {
                        tag: entry.tag.clone(),
                        result,
                        closed: Some(entry),
                    });
                }
                Outcome::Changed
            }
        }
    }

    /// Take everything the open layers reported.
    pub fn drain(&mut self, focus: &mut Focus, reports: &mut Vec<Report>) {
        loop {
            let Some(tag) = self.top_tag().cloned() else {
                break;
            };
            match self.layers.take(focus) {
                Some(Taken::Event(result)) => reports.push(Report {
                    tag,
                    result,
                    closed: None,
                }),
                Some(Taken::Closed(entry, result)) => reports.push(Report {
                    tag: entry.tag.clone(),
                    result,
                    closed: Some(entry),
                }),
                None => break,
            }
        }
    }
}
