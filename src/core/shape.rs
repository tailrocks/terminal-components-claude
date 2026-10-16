//! Pointer and caret shapes.
//!
//! On the desktop the mouse pointer says what a press will do: an I-beam
//! over text, a resize arrow over a split seam. Terminals that implement
//! the OSC 22 pointer-shape protocol (kitty, WezTerm, ghostty, foot) let an
//! application set it with CSS cursor names; [`PointerShape`] is that
//! vocabulary, chosen per hit region ([`crate::core::hit::HitRegistry::set_shape`])
//! and emitted by the runtime only when it changes. Text fields keep the
//! macOS bar caret (DECSCUSR, [`CaretShape`]) instead of the terminal's
//! block.
//!
//! Both are detection-gated ([`ShapeSupport::from_env`]): nothing is sent
//! to a terminal not known to understand it, and both are reset on exit,
//! on suspend and after a panic. Both are overridable:
//! [`crate::core::pointer::PointerConfig::pointer_shapes`] and
//! [`crate::core::pointer::PointerConfig::caret_shape`].

use std::io::Write;
use std::sync::atomic::{AtomicU8, Ordering};

use super::hit::HitRegistry;
use super::pointer::PointerState;

/// A pointer shape, by its CSS cursor name (OSC 22).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum PointerShape {
    /// The arrow.
    #[default]
    Default,
    /// The I-beam: text that a press places the caret in or selects.
    Text,
    /// The pointing hand: a link.
    Pointer,
    /// A horizontal resize: a vertical seam between columns or panes.
    ColResize,
    /// A vertical resize: a horizontal seam between rows or panes.
    RowResize,
}

impl PointerShape {
    /// The CSS cursor name OSC 22 takes.
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Text => "text",
            Self::Pointer => "pointer",
            Self::ColResize => "col-resize",
            Self::RowResize => "row-resize",
        }
    }

    /// The OSC 22 sequence that sets this shape.
    pub fn osc22(self) -> String {
        format!("\x1b]22;{}\x1b\\", self.css_name())
    }
}

/// The text caret's shape (DECSCUSR). Default [`CaretShape::Bar`], the
/// macOS and Windows text caret.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum CaretShape {
    /// A steady vertical bar between cells.
    #[default]
    Bar,
    /// A steady block over the cell.
    Block,
    /// A steady underline under the cell.
    Underline,
    /// Whatever the terminal's user configured: nothing is sent.
    Terminal,
}

impl CaretShape {
    /// The DECSCUSR sequence for this shape (`None` for
    /// [`CaretShape::Terminal`]).
    pub const fn decscusr(self) -> Option<&'static str> {
        match self {
            Self::Block => Some("\x1b[2 q"),
            Self::Underline => Some("\x1b[4 q"),
            Self::Bar => Some("\x1b[6 q"),
            Self::Terminal => None,
        }
    }
}

/// DECSCUSR 0: the terminal's own caret shape again.
pub const CARET_RESET: &str = "\x1b[0 q";

/// What the terminal is known to understand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShapeSupport {
    /// OSC 22 pointer shapes.
    pub pointer: bool,
    /// DECSCUSR caret shapes.
    pub caret: bool,
}

impl ShapeSupport {
    /// Nothing is sent.
    pub const NONE: Self = Self {
        pointer: false,
        caret: false,
    };

    /// Detect support from the environment (`TERM`, `TERM_PROGRAM`). OSC 22
    /// is sent only to kitty, WezTerm, ghostty and foot, never through tmux
    /// or screen (they do not pass it on). DECSCUSR is sent to the xterm
    /// family and the terminals above; not to the Linux console, `dumb` or
    /// an unknown `TERM`.
    pub fn from_env(get: impl Fn(&str) -> Option<String>) -> Self {
        let term = get("TERM").unwrap_or_default().to_ascii_lowercase();
        let program = get("TERM_PROGRAM").unwrap_or_default().to_ascii_lowercase();
        let multiplexed = term.starts_with("tmux")
            || term.starts_with("screen")
            || program == "tmux"
            || get("TMUX").is_some();
        let modern = term.contains("kitty")
            || term.contains("ghostty")
            || term.starts_with("foot")
            || term.starts_with("wezterm")
            || program == "wezterm"
            || program == "ghostty"
            || get("KITTY_WINDOW_ID").is_some();
        let pointer = modern && !multiplexed;
        let caret = modern
            || term.starts_with("xterm")
            || term.starts_with("tmux")
            || term.starts_with("screen")
            || term.starts_with("alacritty")
            || term.starts_with("rxvt-unicode")
            || term.starts_with("vte");
        Self { pointer, caret }
    }

    /// [`Self::from_env`] on the process environment, detected once.
    pub fn current() -> Self {
        static CURRENT: std::sync::OnceLock<ShapeSupport> = std::sync::OnceLock::new();
        *CURRENT.get_or_init(|| Self::from_env(|k| std::env::var(k).ok()))
    }
}

/// The pointer shape for what is under (or held by) the pointer: the
/// pressed control's shape during a drag (a seam keeps its resize arrow
/// while the pointer runs ahead), else the hovered control's.
pub fn under(pointer: &PointerState, hits: &HitRegistry) -> PointerShape {
    pointer
        .pressed
        .or(pointer.hover)
        .and_then(|id| hits.shape_of(id))
        .unwrap_or_default()
}

/// The shapes an app shows under its [`PointerState::config`]: the shape
/// [`under`] the pointer when
/// [`crate::core::pointer::PointerConfig::pointer_shapes`] is on, and the
/// configured caret.
pub fn shapes_for(pointer: &PointerState, hits: &HitRegistry) -> Shapes {
    Shapes {
        pointer: pointer.config.pointer_shapes.then(|| under(pointer, hits)),
        caret: Some(pointer.config.caret_shape),
    }
}

/// What an application asks the runtime to show, per frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Shapes {
    /// The pointer shape; `None`: the application does not set pointer
    /// shapes.
    pub pointer: Option<PointerShape>,
    /// The caret shape; `None`: left to the terminal.
    pub caret: Option<CaretShape>,
}

/// What the runtime last sent, so a shape is written only when it changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShapeSync {
    support: ShapeSupport,
    pointer: Option<PointerShape>,
    caret: Option<CaretShape>,
}

/// Bits of what has been sent this process: a reset is written only for
/// what was set (`restore` runs from a panic hook and has no state).
static SENT: AtomicU8 = AtomicU8::new(0);
const SENT_POINTER: u8 = 1;
const SENT_CARET: u8 = 2;

impl ShapeSync {
    pub fn new(support: ShapeSupport) -> Self {
        Self {
            support,
            pointer: None,
            caret: None,
        }
    }

    /// Write what changed in `want` to `out` (nothing the terminal is not
    /// known to support). Returns whether anything was written.
    pub fn apply(&mut self, want: Shapes, out: &mut impl Write) -> std::io::Result<bool> {
        let mut wrote = false;
        if self.support.pointer
            && let Some(p) = want.pointer
            && self.pointer != Some(p)
        {
            out.write_all(p.osc22().as_bytes())?;
            self.pointer = Some(p);
            SENT.fetch_or(SENT_POINTER, Ordering::Relaxed);
            wrote = true;
        }
        if self.support.caret
            && let Some(c) = want.caret
            && self.caret != Some(c)
            && let Some(seq) = c.decscusr()
        {
            out.write_all(seq.as_bytes())?;
            self.caret = Some(c);
            SENT.fetch_or(SENT_CARET, Ordering::Relaxed);
            wrote = true;
        }
        Ok(wrote)
    }

    /// Forget what was sent (the terminal was handed back and re-entered):
    /// the next [`Self::apply`] writes the shapes again.
    pub fn forget(&mut self) {
        self.pointer = None;
        self.caret = None;
    }
}

/// The resets for whatever was set this process: the arrow pointer and
/// the terminal's own caret. Empty when nothing was sent.
pub fn reset_sequence() -> String {
    let sent = SENT.swap(0, Ordering::Relaxed);
    let mut s = String::new();
    if sent & SENT_POINTER != 0 {
        s.push_str(&PointerShape::Default.osc22());
    }
    if sent & SENT_CARET != 0 {
        s.push_str(CARET_RESET);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::id::WidgetId;
    use ratatui::layout::{Position, Rect};

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn the_sequences_are_byte_exact() {
        assert_eq!(PointerShape::Text.osc22(), "\x1b]22;text\x1b\\");
        assert_eq!(PointerShape::ColResize.osc22(), "\x1b]22;col-resize\x1b\\");
        assert_eq!(PointerShape::RowResize.osc22(), "\x1b]22;row-resize\x1b\\");
        assert_eq!(PointerShape::Default.osc22(), "\x1b]22;default\x1b\\");
        assert_eq!(CaretShape::Bar.decscusr(), Some("\x1b[6 q"));
        assert_eq!(CaretShape::Block.decscusr(), Some("\x1b[2 q"));
        assert_eq!(CaretShape::Underline.decscusr(), Some("\x1b[4 q"));
        assert_eq!(CaretShape::Terminal.decscusr(), None);
        assert_eq!(CARET_RESET, "\x1b[0 q");
    }

    #[test]
    fn support_is_detected_from_the_terminal() {
        let s = ShapeSupport::from_env(env(&[("TERM", "xterm-kitty")]));
        assert_eq!(
            s,
            ShapeSupport {
                pointer: true,
                caret: true
            }
        );
        let s = ShapeSupport::from_env(env(&[
            ("TERM", "xterm-256color"),
            ("TERM_PROGRAM", "WezTerm"),
        ]));
        assert!(s.pointer && s.caret);
        let s = ShapeSupport::from_env(env(&[("TERM", "xterm-ghostty")]));
        assert!(s.pointer);
        let s = ShapeSupport::from_env(env(&[("TERM", "foot")]));
        assert!(s.pointer);
        // iTerm2, Terminal.app: the caret only
        let s = ShapeSupport::from_env(env(&[
            ("TERM", "xterm-256color"),
            ("TERM_PROGRAM", "iTerm.app"),
        ]));
        assert_eq!(
            s,
            ShapeSupport {
                pointer: false,
                caret: true
            }
        );
        // tmux does not pass OSC 22 on
        let s = ShapeSupport::from_env(env(&[("TERM", "tmux-256color"), ("KITTY_WINDOW_ID", "1")]));
        assert!(!s.pointer && s.caret);
        // the console and unknown terminals get nothing
        assert_eq!(
            ShapeSupport::from_env(env(&[("TERM", "linux")])),
            ShapeSupport::NONE
        );
        assert_eq!(
            ShapeSupport::from_env(env(&[("TERM", "dumb")])),
            ShapeSupport::NONE
        );
        assert_eq!(ShapeSupport::from_env(env(&[])), ShapeSupport::NONE);
    }

    /// A shape is written once, when it changes, and only where supported;
    /// the reset covers exactly what was sent.
    #[test]
    fn shapes_are_written_on_change_only() {
        let full = ShapeSupport {
            pointer: true,
            caret: true,
        };
        let mut sync = ShapeSync::new(full);
        let mut out = Vec::new();
        let want = Shapes {
            pointer: Some(PointerShape::Text),
            caret: Some(CaretShape::Bar),
        };
        assert!(sync.apply(want, &mut out).unwrap());
        assert_eq!(String::from_utf8_lossy(&out), "\x1b]22;text\x1b\\\x1b[6 q");
        out.clear();
        assert!(!sync.apply(want, &mut out).unwrap(), "unchanged: nothing");
        assert!(out.is_empty());
        let want = Shapes {
            pointer: Some(PointerShape::ColResize),
            ..want
        };
        sync.apply(want, &mut out).unwrap();
        assert_eq!(String::from_utf8_lossy(&out), "\x1b]22;col-resize\x1b\\");
        assert_eq!(reset_sequence(), "\x1b]22;default\x1b\\\x1b[0 q");
        assert_eq!(reset_sequence(), "", "reset once");
        // after a suspend the shapes go out again
        out.clear();
        sync.forget();
        sync.apply(want, &mut out).unwrap();
        assert!(!out.is_empty());
        reset_sequence();
        // unsupported: nothing at all
        let mut sync = ShapeSync::new(ShapeSupport::NONE);
        out.clear();
        assert!(!sync.apply(want, &mut out).unwrap());
        assert!(out.is_empty());
        assert_eq!(reset_sequence(), "");
        // the terminal's own caret: nothing for the caret
        let mut sync = ShapeSync::new(full);
        sync.apply(
            Shapes {
                pointer: None,
                caret: Some(CaretShape::Terminal),
            },
            &mut out,
        )
        .unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn the_shape_follows_the_hovered_or_pressed_control() {
        let text = WidgetId::of("field");
        let seam = WidgetId::of("seam");
        let mut hits = HitRegistry::default();
        hits.register(text, Rect::new(0, 0, 10, 1));
        hits.set_shape(text, PointerShape::Text);
        hits.register(seam, Rect::new(10, 0, 1, 5));
        hits.set_shape(seam, PointerShape::ColResize);
        hits.register(WidgetId::of("button"), Rect::new(0, 2, 6, 1));
        let mut p = PointerState::default();
        p.on_move(&hits, Position::new(3, 0));
        assert_eq!(under(&p, &hits), PointerShape::Text);
        p.on_move(&hits, Position::new(2, 2));
        assert_eq!(
            under(&p, &hits),
            PointerShape::Default,
            "a button: the arrow"
        );
        p.on_down(&hits, Position::new(10, 1));
        p.on_drag(&hits, Position::new(20, 1));
        assert_eq!(
            under(&p, &hits),
            PointerShape::ColResize,
            "held while dragging"
        );
    }
}
