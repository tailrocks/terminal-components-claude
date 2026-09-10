//! Reference grapheme-to-cell segmentation for the capture tools (F22).
//!
//! The PNG and HTML rasterizers must agree with the application about how
//! text occupies terminal cells. This example is that single reference: it
//! uses the same `unicode-segmentation` and `unicode-width` versions the
//! library renders with, so the tools never re-implement width heuristics.
//!
//! Usage: `cargo build --example cells`, then feed plain text lines on stdin;
//! one JSON array per input line is written, each element `[grapheme, width]`.
//! Control characters are reported with width 0 so callers can flag them.

use std::io::{BufRead, Write};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

fn escape(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        let mut json = String::from("[");
        for (i, g) in line.graphemes(true).enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str("[\"");
            escape(g, &mut json);
            json.push_str("\",");
            json.push_str(&UnicodeWidthStr::width(g).to_string());
            json.push(']');
        }
        json.push_str("]\n");
        out.write_all(json.as_bytes()).unwrap();
    }
}
