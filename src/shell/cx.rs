//! What a page may ask of the shell (plan §1.8, `Cx` / `Request`): one
//! event context for every page of every app, and one request type the
//! shell applies after the page returns.

use std::any::Any;

use crate::core::focus::{Focus, FocusRing};
use crate::theme::Tone;
use crate::ui::context::ContextConfig;
use crate::ui::layer::{Layer, LayerTag};
use crate::ui::popup::Dismiss;

/// A deferred action on the top layer ([`Request::Top`]).
pub type TopFn<R, W> = Box<dyn FnOnce(&mut dyn Layer<R, W>)>;

/// Something a page asks the shell to do. `R` is the app's layer result,
/// `W` the app state its layers read, `G` the app's navigation.
pub enum Request<R, W: ?Sized = (), G = ()> {
    /// A footer status in `Tone` (information expires; an error stays
    /// until the next action, [`super::StatusLine`]).
    Status(String, Tone),
    /// Open a layer on top, owned by the page that asked
    /// ([`crate::ui::layer::ShellLayers::open`]).
    Open(Box<dyn Layer<R, W>>, LayerTag),
    /// Close the page's own top layer without a result.
    Close,
    /// Act on the top layer (fill a form field from a chooser).
    Top(TopFn<R, W>),
    /// Navigate.
    Go(G),
    /// Copy text to the clipboard; the shell reports the result.
    Copy(String),
    /// Open the keyboard reference.
    Help,
    /// A layer of the page refused an outside click under its
    /// [`Dismiss`] policy: the shell flashes it
    /// ([`crate::ui::modal::refuse_outside_click`]).
    RefusedOutsideClick(Dismiss),
}

impl<R, W: ?Sized, G> Request<R, W, G> {
    /// The background-request policy: a page that is mounted but not
    /// shown (a background tab, another route) may only report a status
    /// or an error. Anything that acts on the screen (a layer,
    /// navigation, the clipboard) is dropped, so the page in front never
    /// changes under the user because of a page behind it.
    pub fn allowed_in_background(&self) -> bool {
        matches!(self, Self::Status(..))
    }
}

/// The background-request policy over a batch: what a page that is not
/// shown may still report ([`Request::allowed_in_background`]).
pub fn background<R, W: ?Sized, G>(
    requests: Vec<Request<R, W, G>>,
) -> impl Iterator<Item = Request<R, W, G>> {
    requests.into_iter().filter(Request::allowed_in_background)
}

/// The context a page handles an event in: focus and the ring of the last
/// frame, the app's context-menu keys, and the requests it makes.
pub struct Cx<'a, R, W: ?Sized = (), G = ()> {
    pub focus: &'a mut Focus,
    pub ring: &'a FocusRing,
    /// Which keys open a context menu and whether a secondary click moves
    /// the cursor ([`ContextConfig::DEFAULT`] unless the app sets one).
    pub context: ContextConfig,
    pub requests: Vec<Request<R, W, G>>,
}

impl<'a, R, W: ?Sized, G> Cx<'a, R, W, G> {
    pub fn new(focus: &'a mut Focus, ring: &'a FocusRing) -> Self {
        Self {
            focus,
            ring,
            context: ContextConfig::DEFAULT,
            requests: Vec::new(),
        }
    }

    /// With the app's context-menu configuration.
    pub fn context(mut self, context: ContextConfig) -> Self {
        self.context = context;
        self
    }

    /// The requests made so far, taken.
    pub fn take(&mut self) -> Vec<Request<R, W, G>> {
        std::mem::take(&mut self.requests)
    }

    pub fn focus_next(&mut self) {
        self.focus.next(self.ring);
    }

    pub fn focus_prev(&mut self) {
        self.focus.prev(self.ring);
    }

    /// A neutral footer status ("Copied 12 chars").
    pub fn status(&mut self, s: impl Into<String>) {
        self.status_in(s, Tone::Secondary);
    }

    /// A footer status in `tone`.
    pub fn status_in(&mut self, s: impl Into<String>, tone: Tone) {
        self.requests.push(Request::Status(s.into(), tone));
    }

    /// An error status: it stays until the next action.
    pub fn error(&mut self, s: impl Into<String>) {
        self.status_in(s, Tone::Error);
    }

    pub fn open(&mut self, layer: impl Layer<R, W>, tag: LayerTag) {
        self.requests.push(Request::Open(Box::new(layer), tag));
    }

    pub fn close(&mut self) {
        self.requests.push(Request::Close);
    }

    pub fn go(&mut self, g: G) {
        self.requests.push(Request::Go(g));
    }

    pub fn copy(&mut self, s: impl Into<String>) {
        self.requests.push(Request::Copy(s.into()));
    }

    pub fn help(&mut self) {
        self.requests.push(Request::Help);
    }

    pub fn refuse_outside_click(&mut self, dismiss: Dismiss) {
        self.requests.push(Request::RefusedOutsideClick(dismiss));
    }
}

impl<'a, R: 'static, W: ?Sized + 'static, G> Cx<'a, R, W, G> {
    /// Act on the top layer when it is an `L` (fill a form field from a
    /// chooser the form opened).
    pub fn with_top<L: Layer<R, W>>(&mut self, f: impl FnOnce(&mut L) + 'static) {
        self.requests
            .push(Request::Top(Box::new(move |layer: &mut dyn Layer<R, W>| {
                let any: &mut dyn Any = layer;
                if let Some(l) = any.downcast_mut::<L>() {
                    f(l);
                }
            })));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_background_page_may_only_report_a_status() {
        let mut focus = Focus::default();
        let ring = FocusRing::default();
        let mut cx: Cx<'_, ()> = Cx::new(&mut focus, &ring);
        cx.status("saved");
        cx.error("lost");
        cx.go(());
        cx.copy("x");
        cx.help();
        cx.close();
        let kept: Vec<_> = background(cx.take()).collect();
        assert_eq!(kept.len(), 2);
        assert!(matches!(kept[0], Request::Status(_, Tone::Secondary)));
        assert!(matches!(kept[1], Request::Status(_, Tone::Error)));
    }
}
