use std::cell::RefCell;

use crate::core::edit::clipboard;
use crate::theme::Tone;
use crate::ui::text::{plural, truncate};

thread_local! {

    static OUT: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Copied {
    Copied { chars: usize },

    Unsupported,

    TooLarge { encoded: usize, limit: usize },
}

impl Copied {
    pub fn ok(&self) -> bool {
        matches!(self, Self::Copied { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clipboard {
    pub osc52: bool,

    pub limit: usize,

    pub write: bool,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    pub const DEFAULT_LIMIT: usize = 100_000;

    pub const fn new() -> Self {
        Self {
            osc52: true,
            limit: Self::DEFAULT_LIMIT,
            write: true,
        }
    }

    pub const fn osc52(mut self, on: bool) -> Self {
        self.osc52 = on;
        self
    }

    pub const fn limit(mut self, bytes: usize) -> Self {
        self.limit = bytes;
        self
    }

    pub const fn write(mut self, on: bool) -> Self {
        self.write = on;
        self
    }

    pub fn copy(&self, text: &str) -> Copied {
        clipboard::copy(text);
        self.system(text)
    }

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

    pub fn install(self) {
        clipboard::set_hook(move |t| {
            self.system(t);
        });
    }

    pub fn report(result: &Copied, text: &str, what: Option<&str>) -> (String, Tone) {
        match result {
            Copied::Copied { .. } if let Some(what) = what => {
                (format!("Copied {what}"), Tone::Secondary)
            }
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

pub fn take_output() -> Vec<String> {
    OUT.with(|o| std::mem::take(&mut *o.borrow_mut()))
}

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
        let (msg, tone) = Clipboard::report(&Copied::Copied { chars: 5 }, "héllo", None);
        assert_eq!(
            (msg.as_str(), tone),
            ("Copied 5 characters · héllo", Tone::Secondary)
        );

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
        assert_eq!(Clipboard::report(&r, "", Some("Host")).1, Tone::Error);
        assert_eq!(
            Clipboard::report(&Copied::Copied { chars: 1 }, "x", Some("Host")).0,
            "Copied Host"
        );
        assert_eq!(Clipboard::new().osc52(false).copy("x"), Copied::Unsupported);
        assert!(take_output().is_empty());

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
