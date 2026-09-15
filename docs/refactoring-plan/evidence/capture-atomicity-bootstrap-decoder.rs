//! Reads actual PTY bytes; all exports borrow one decoded immutable Frame.
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    let mut raw = Vec::new();
    io::stdin().read_to_end(&mut raw)?;
    anyhow::ensure!(!raw.is_empty() && raw.len() <= 65536, "invalid raw capture size");
    let source = std::env::args().nth(1).ok_or_else(|| anyhow::anyhow!("missing source identity"))?;
    let frame = tuisnap::ansi::replay_raw(
        &raw, 24, 4, 0, tuisnap::Provenance::now("atomicity-fixture", &source, Vec::new()),
    )?;
    frame.validate()?;
    let text = frame.text();
    // This minimal fixture HTML is a serialization-origin witness only. It
    // does not claim production HTML/PNG renderer or font-fidelity coverage.
    let escaped = text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    println!("{}", serde_json::json!({
        "schema": "tc-atomic-decoded/v1",
        "frame": frame,
        "text": text,
        "ansi": tuisnap::render::ansi_dump(&frame),
        "html": format!("<!doctype html><meta charset=\"utf-8\"><pre>{escaped}</pre>"),
    }));
    Ok(())
}
