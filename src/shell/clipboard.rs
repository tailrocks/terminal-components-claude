//! The system clipboard (plan §1.8, `Clipboard`): one copy path for every
//! app, with a truthful result. A terminal app reaches the system
//! clipboard only through OSC 52, which a terminal may refuse or cap, so
//! a copy reports what really happened ("Copied 12 characters", "Not
//! copied · … exceed the terminal's limit") instead of always claiming
//! success.
//!
//! [`Clipboard::copy`] keeps the text on the field clipboard (so in-app
//! Paste works whatever the terminal does), and queues the OSC 52 write;
//! the runtime writes it after the frame
//! ([`Application::take_output`](crate::runtime::Application::take_output)).
//! [`Clipboard::install`] hands every field Copy and Cut to the same
//! writer.

use std::cell::RefCell;

use crate::core::edit::clipboard;
use crate::theme::Tone;
use crate::ui::text::{plural, truncate};

thread_local! {
    /// Escape sequences waiting for the next frame's write.
    static OUT: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// The result of a copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Copied {
    /// On the system clipboard (`chars` characters).
    Copied { chars: usize },
    /// Kept in the app only: the terminal does not take OSC 52.
    Unsupported,
    /// Kept in the app only: the encoded text is larger than the
    /// terminal accepts.
    TooLarge { encoded: usize, limit: usize },
}

impl Copied {
    /// It reached the system clipboard.
    pub fn ok(&self) -> bool {
        matches!(self, Self::Copied { .. })
    }
}

/// How the app reaches the system clipboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clipboard {
    /// The terminal takes OSC 52 writes. Default `true` (most modern
    /// terminals; tmux needs `set-clipboard on`).
    pub osc52: bool,
    /// The largest base64 payload the terminal takes, in bytes. Default
    /// [`Self::DEFAULT_LIMIT`].
    pub limit: usize,
    /// Write the escape sequence to the terminal. Default `true`; a
    /// simulated terminal (a preview's world) turns it off and reads the
    /// result only.
    pub write: bool,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    /// xterm's default cap on a selection it accepts (and a safe floor
    /// for tmux and most emulators).
    pub const DEFAULT_LIMIT: usize = 100_000;

    pub const fn new() -> Self {
        Self {
            osc52: true,
            limit: Self::DEFAULT_LIMIT,
            write: true,
        }
    }

    /// See [`Self::osc52`].
    pub const fn osc52(mut self, on: bool) -> Self {
        self.osc52 = on;
        self
    }

    /// See [`Self::limit`].
    pub const fn limit(mut self, bytes: usize) -> Self {
        self.limit = bytes;
        self
    }

    /// See [`Self::write`].
    pub const fn write(mut self, on: bool) -> Self {
        self.write = on;
        self
    }

    /// Copy `text`: onto the field clipboard always, and onto the system
    /// clipboard when the terminal takes it.
    pub fn copy(&self, text: &str) -> Copied {
        clipboard::copy(text);
        self.system(text)
    }

    /// The system half of [`Self::copy`] (the field clipboard already
    /// holds `text`).
    fn system(&self, text: &str) -> Copied {
        if !self.osc52 {
            return Copied::Unsupported;
        }
        let encoded = text.len().div_ceil(3) * 4;
        if encoded > self.limit {
            return Copied::TooLarge {
                encoded,
                limit: self.limit,
            };
        }
        if self.write {
            OUT.with(|o| o.borrow_mut().push(osc52(text)));
        }
        Copied::Copied {
            chars: text.chars().count(),
        }
    }

    /// Hand every field Copy and Cut to the system clipboard too.
    pub fn install(self) {
        clipboard::set_hook(move |t| {
            self.system(t);
        });
    }

    /// The footer status of `result` for `text`: "Copied 12 characters ·
    /// text…" (secondary) or why it is not on the system clipboard
    /// (error).
    pub fn report(result: &Copied, text: &str) -> (String, Tone) {
        match result {
            Copied::Copied { chars } => {
                let shown = truncate(&text.replace('\n', " · "), 40);
                (
                    format!(
                        "Copied {} · {shown}",
                        plural(*chars, "character", "characters")
                    ),
                    Tone::Secondary,
                )
            }
            Copied::Unsupported => (
                "Not copied · this terminal does not accept OSC 52".to_owned(),
                Tone::Error,
            ),
            Copied::TooLarge { encoded, limit } => (
                format!(
                    "Not copied · {encoded} encoded bytes exceed the terminal's {limit} byte OSC 52 limit"
                ),
                Tone::Error,
            ),
        }
    }
}

/// The escape sequences queued since the last call, for the terminal.
pub fn take_output() -> Vec<String> {
    OUT.with(|o| std::mem::take(&mut *o.borrow_mut()))
}

/// The OSC 52 sequence that puts `text` on the system clipboard.
pub fn osc52(text: &str) -> String {
    format!("\x1b]52;c;{}\x07", base64(text.as_bytes()))
}

fn base64(bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ABC[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc_4648() {
        for (plain, enc) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(plain.as_bytes()), enc);
        }
    }

    #[test]
    fn a_copy_reports_what_reached_the_system_clipboard() {
        take_output();
        let c = Clipboard::new();
        assert_eq!(c.copy("héllo"), Copied::Copied { chars: 5 });
        assert_eq!(take_output(), [osc52("héllo")]);
        assert_eq!(clipboard::text().as_deref(), Some("héllo"));
        let (msg, tone) = Clipboard::report(&Copied::Copied { chars: 5 }, "héllo");
        assert_eq!(
            (msg.as_str(), tone),
            ("Copied 5 characters · héllo", Tone::Secondary)
        );

        // refused: the field clipboard still has it, nothing is written
        let c = Clipboard::new().limit(8);
        let r = c.copy("0123456789");
        assert_eq!(
            r,
            Copied::TooLarge {
                encoded: 16,
                limit: 8
            }
        );
        assert!(take_output().is_empty());
        assert_eq!(clipboard::text().as_deref(), Some("0123456789"));
        assert_eq!(Clipboard::report(&r, "").1, Tone::Error);
        assert_eq!(Clipboard::new().osc52(false).copy("x"), Copied::Unsupported);
        assert!(take_output().is_empty());

        // a simulated terminal reports without writing
        assert!(Clipboard::new().write(false).copy("x").ok());
        assert!(take_output().is_empty());
    }

    #[test]
    fn an_installed_clipboard_takes_field_copies() {
        take_output();
        Clipboard::new().install();
        clipboard::copy("cut");
        assert_eq!(take_output(), [osc52("cut")]);
        clipboard::clear_hook();
    }
}
