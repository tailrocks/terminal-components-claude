//! Pointer and caret cursor shapes (ported from PR #26 `src/core/shape.rs`,
//! commit f4867e9b5).
//!
//! Pure vocabulary and sync machinery only. Nothing in the workspace emits
//! a shape sequence yet: an app opts in by driving [`ShapeSync`], so frozen
//! apps keep sending no cursor-shape escapes.

use std::io::Write;

/// The mouse pointer shape a control asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum PointerShape {
    /// The arrow.
    #[default]
    Default,
    /// The I-beam over text.
    Text,
    /// The pointing hand over a link-like control.
    Pointer,
    /// The column resize arrows over a vertical seam.
    ColResize,
    /// The row resize arrows over a horizontal seam.
    RowResize,
}

impl PointerShape {
    /// The CSS cursor name, which OSC 22 uses.
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Text => "text",
            Self::Pointer => "pointer",
            Self::ColResize => "col-resize",
            Self::RowResize => "row-resize",
        }
    }

    /// The OSC 22 sequence that sets this pointer shape.
    pub fn osc22(self) -> String {
        format!("\x1b]22;{}\x1b\\", self.css_name())
    }
}

/// The text caret shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum CaretShape {
    /// The blinking bar.
    #[default]
    Bar,
    /// The block.
    Block,
    /// The underline.
    Underline,
    /// Leave the caret to the terminal.
    Terminal,
}

impl CaretShape {
    /// The DECSCUSR sequence for the shape, if it has one.
    pub const fn decscusr(self) -> Option<&'static str> {
        match self {
            Self::Block => Some("\x1b[2 q"),
            Self::Underline => Some("\x1b[4 q"),
            Self::Bar => Some("\x1b[6 q"),
            Self::Terminal => None,
        }
    }
}

/// The DECSCUSR reset sequence.
pub const CARET_RESET: &str = "\x1b[0 q";

/// Which shape sequences the terminal supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShapeSupport {
    /// OSC 22 pointer shapes.
    pub pointer: bool,
    /// DECSCUSR caret shapes.
    pub caret: bool,
}

impl ShapeSupport {
    /// No shape support at all.
    pub const NONE: Self = Self {
        pointer: false,
        caret: false,
    };

    /// Detect support from the provided environment lookup.
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

    /// Detect once per process from the real environment.
    pub fn current() -> Self {
        static CURRENT: std::sync::OnceLock<ShapeSupport> = std::sync::OnceLock::new();
        *CURRENT.get_or_init(|| Self::from_env(|k| std::env::var(k).ok()))
    }
}

/// The shapes a frame wants in effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Shapes {
    /// The pointer shape, or `None` to leave it alone.
    pub pointer: Option<PointerShape>,
    /// The caret shape, or `None` to leave it alone.
    pub caret: Option<CaretShape>,
}

static SENT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
const SENT_POINTER: u8 = 1;
const SENT_CARET: u8 = 2;

/// Applies wanted shapes to an output, writing a sequence only when the
/// shape changes and only when the terminal supports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShapeSync {
    support: ShapeSupport,
    pointer: Option<PointerShape>,
    caret: Option<CaretShape>,
}

impl ShapeSync {
    /// A sync for a terminal with the given support.
    pub fn new(support: ShapeSupport) -> Self {
        Self {
            support,
            pointer: None,
            caret: None,
        }
    }

    /// Write the sequences for `want`, returning whether anything was written.
    pub fn apply(&mut self, want: Shapes, out: &mut impl Write) -> std::io::Result<bool> {
        use std::sync::atomic::Ordering;
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

    /// Forget what was written so the next apply resends.
    pub fn forget(&mut self) {
        self.pointer = None;
        self.caret = None;
    }
}

/// The reset sequences for every shape this process has sent, once.
pub fn reset_sequence() -> String {
    use std::sync::atomic::Ordering;
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

        let s = ShapeSupport::from_env(env(&[("TERM", "tmux-256color"), ("KITTY_WINDOW_ID", "1")]));
        assert!(!s.pointer && s.caret);

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

        out.clear();
        sync.forget();
        sync.apply(want, &mut out).unwrap();
        assert!(!out.is_empty());
        reset_sequence();

        let mut sync = ShapeSync::new(ShapeSupport::NONE);
        out.clear();
        assert!(!sync.apply(want, &mut out).unwrap());
        assert!(out.is_empty());
        assert_eq!(reset_sequence(), "");

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
}
