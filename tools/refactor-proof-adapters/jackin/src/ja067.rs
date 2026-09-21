//! JA-067: the five color identities and the detect table.
//!
//! Each [`CaptureColor`] identity paints the capsule world at both
//! sizes; the mono downgrade paints no `Rgb` cells while truecolor does,
//! and `--color none` and `NO_COLOR` share the `Mono` level with
//! identical paint. The pure [`ColorLevel::from_env`] decision table —
//! dumb-term facts, `CLICOLOR_FORCE`, `NO_COLOR`, TTY status,
//! `COLORTERM`, then `TERM` — is captured row by row without touching
//! the environment.

use std::ffi::OsStr;

use jackin_app::{Motion, Scenario};
use junie_tui::ColorLevel;

use crate::ObservedFrame;
use crate::observe::{CaptureColor, DirectSession, Viewport};

/// Scenario id.
pub const JA067_ID: &str = "JA-067";
/// JA-067 sizes.
pub const JA067_SIZES: [Viewport; 2] = [Viewport::new(80, 24), Viewport::new(120, 40)];

/// One size of JA-067.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja067Capture {
    /// All five identity labels are distinct.
    pub labels_distinct: bool,
    /// The identity-to-level mapping matches the reference.
    pub levels_ok: bool,
    /// `Rgb` cell counts per identity, in [`CaptureColor::all`] order.
    pub rgb_cells: Vec<usize>,
    /// The mono downgrade paints different cells than truecolor.
    pub mono_differs: bool,
    /// `none` and `nocolor` paint identical frames.
    pub none_equals_nocolor: bool,
    /// Every [`ColorLevel::from_env`] row matches the reference.
    pub detect_table: bool,
    /// One frame per identity, in [`CaptureColor::all`] order.
    pub frames: Vec<ObservedFrame>,
}

fn rgb_cells(frame: &ObservedFrame) -> usize {
    frame
        .cells
        .iter()
        .filter(|cell| cell.fg.starts_with("Rgb"))
        .count()
}

fn levels_ok() -> bool {
    use CaptureColor::{Ansi16, Ansi256, NoColor, None, TrueColor};
    TrueColor.level() == ColorLevel::TrueColor
        && Ansi256.level() == ColorLevel::Ansi256
        && Ansi16.level() == ColorLevel::Ansi16
        && None.level() == ColorLevel::Mono
        && NoColor.level() == ColorLevel::Mono
}

/// One [`ColorLevel::from_env`] row: force, no-color, term, colorterm, tty, expected.
type DetectRow<'a> = (
    Option<&'a OsStr>,
    Option<&'a OsStr>,
    Option<&'a str>,
    Option<&'a str>,
    bool,
    ColorLevel,
);

fn detect_table_ok() -> bool {
    use ColorLevel::{Ansi16, Ansi256, Mono, TrueColor};
    let tty = true;
    let rows: [DetectRow<'_>; 14] = [
        // dumb wins over everything, forced colour included
        (
            Some(OsStr::new("1")),
            Some(OsStr::new("1")),
            Some("dumb"),
            Some("truecolor"),
            tty,
            Mono,
        ),
        // forced colour overrides NO_COLOR and a missing TTY
        (
            Some(OsStr::new("1")),
            Some(OsStr::new("1")),
            Some("xterm"),
            None,
            false,
            Ansi16,
        ),
        // empty and "0" do not force
        (
            Some(OsStr::new("")),
            None,
            Some("xterm"),
            Some("truecolor"),
            tty,
            TrueColor,
        ),
        (
            Some(OsStr::new("0")),
            None,
            Some("xterm"),
            Some("truecolor"),
            tty,
            TrueColor,
        ),
        // NO_COLOR: any non-empty value disables, empty does not
        (
            None,
            Some(OsStr::new("1")),
            Some("xterm"),
            Some("truecolor"),
            tty,
            Mono,
        ),
        (
            None,
            Some(OsStr::new("0")),
            Some("xterm"),
            Some("truecolor"),
            tty,
            Mono,
        ),
        (
            None,
            Some(OsStr::new("")),
            Some("xterm"),
            Some("truecolor"),
            tty,
            TrueColor,
        ),
        // piped output carries no escape sequences
        (None, None, Some("xterm"), Some("truecolor"), false, Mono),
        // COLORTERM first, then TERM signatures, then the 16-colour floor
        (None, None, Some("xterm"), Some("truecolor"), tty, TrueColor),
        (None, None, Some("xterm"), Some("24bit"), tty, TrueColor),
        (None, None, Some("xterm-256color"), None, tty, Ansi256),
        (None, None, Some("ghostty"), None, tty, Ansi256),
        (None, None, Some("xterm"), None, tty, Ansi16),
        (None, None, None, None, tty, Ansi16),
    ];
    rows.iter()
        .all(|(force, no_color, term, colorterm, tty, expected)| {
            ColorLevel::from_env(*force, *no_color, *term, *colorterm, *tty) == *expected
        })
}

fn capture_size(viewport: Viewport) -> Ja067Capture {
    let frames: Vec<ObservedFrame> = CaptureColor::all()
        .iter()
        .map(|color| {
            let session = DirectSession::fresh(
                JA067_ID,
                Scenario::CapsuleMulti,
                Motion::Full,
                0,
                viewport,
                *color,
            );
            session.observe("color")
        })
        .collect();
    let labels: Vec<&str> = CaptureColor::all()
        .iter()
        .map(|color| color.label())
        .collect();
    let mut distinct = labels.clone();
    distinct.sort_unstable();
    distinct.dedup();
    let rgb: Vec<usize> = frames.iter().map(rgb_cells).collect();
    let (truecolor, mono_none, mono_nocolor) = (
        frames.first().map(|frame| frame.digest).unwrap_or_default(),
        frames.get(3).map(|frame| frame.digest).unwrap_or_default(),
        frames.get(4).map(|frame| frame.digest).unwrap_or_default(),
    );
    Ja067Capture {
        labels_distinct: distinct.len() == labels.len(),
        levels_ok: levels_ok(),
        rgb_cells: rgb,
        mono_differs: truecolor != mono_none,
        none_equals_nocolor: mono_none == mono_nocolor,
        detect_table: detect_table_ok(),
        frames,
    }
}

/// Capture JA-067 at both listed sizes.
#[must_use]
pub fn ja067_color_identities() -> Vec<Ja067Capture> {
    JA067_SIZES.iter().map(|size| capture_size(*size)).collect()
}
