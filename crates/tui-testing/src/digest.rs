//! Headless digest scenes (`COMPONENT_ARCHITECTURE.md` §16.3, §21 item 28).
//!
//! A `Scene` owns a headless runtime (registry + ring + layers + style
//! stack) built from a theme, draws a closure into a buffer and produces an
//! FNV-1a digest of `(symbol, fg, bg, modifier)` per cell. Baselines are
//! one `name w h theme color hash` line each, sorted; `BLESS=1` rewrites.

use std::collections::BTreeMap;
use std::path::Path;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use tui_next::{App, ColorLevel, Cx, FocusRing, Registry, Response, Runtime, Theme, Ui};

/// An application that does nothing; the scene draws through a closure.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoApp;

impl App for NoApp {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, _ui: &mut Ui<'_>) {}
}

/// A named headless frame.
pub struct Scene {
    name: &'static str,
    theme_name: &'static str,
    color: ColorLevel,
    area: Rect,
    buf: Buffer,
    rt: Option<Runtime<NoApp>>,
}

impl core::fmt::Debug for Scene {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Scene")
            .field("name", &self.name)
            .field("area", &self.area)
            .field("digest", &format_args!("{:016x}", self.digest()))
            .finish_non_exhaustive()
    }
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

fn fnv(mut h: u64, bytes: &[u8]) -> u64 {
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

fn color_level_label(c: ColorLevel) -> &'static str {
    match c {
        ColorLevel::TrueColor => "truecolor",
        ColorLevel::Ansi256 => "256",
        ColorLevel::Ansi16 => "16",
        ColorLevel::Mono => "mono",
        _ => "other",
    }
}

fn theme_label(theme: &Theme) -> &'static str {
    if *theme == Theme::junie() {
        "junie"
    } else if *theme == Theme::paper() {
        "paper"
    } else {
        "custom"
    }
}

impl Scene {
    /// A scene of `w × h` under `theme` downgraded to `color`.
    pub fn new(name: &'static str, theme: Theme, color: ColorLevel, w: u16, h: u16) -> Self {
        let theme_name = theme_label(&theme);
        let theme = if color == ColorLevel::TrueColor {
            theme
        } else {
            theme.downgrade(color)
        };
        let area = Rect::new(0, 0, w, h);
        Scene {
            name,
            theme_name,
            color,
            area,
            buf: Buffer::empty(area),
            rt: Some(Runtime::new(NoApp, theme)),
        }
    }

    /// A scene over an existing frame buffer (the harness snapshot).
    pub fn from_buffer(
        name: &'static str,
        theme_name: &'static str,
        color: ColorLevel,
        buf: Buffer,
    ) -> Self {
        let area = *buf.area();
        Scene {
            name,
            theme_name,
            color,
            area,
            buf,
            rt: None,
        }
    }

    /// Rename the scene (the baseline key).
    #[must_use]
    pub const fn named(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Run the whole draw phase with `f` as the page painter.
    pub fn draw(&mut self, f: impl FnOnce(&mut Ui<'_>, Rect)) {
        let area = self.area;
        let Some(rt) = self.rt.as_mut() else {
            return;
        };
        self.buf.reset();
        rt.draw_scene(area, &mut self.buf, f);
    }

    /// Draw over a pre-filled buffer (sentinel tests).
    pub fn draw_over(
        &mut self,
        prefill: impl FnOnce(&mut Buffer),
        f: impl FnOnce(&mut Ui<'_>, Rect),
    ) {
        let area = self.area;
        let Some(rt) = self.rt.as_mut() else {
            return;
        };
        self.buf.reset();
        prefill(&mut self.buf);
        rt.draw_scene(area, &mut self.buf, f);
    }

    /// FNV-1a over `(symbol, fg, bg, modifier)` per cell.
    pub fn digest(&self) -> u64 {
        let mut h = FNV_OFFSET;
        for pos in self.area.positions() {
            let Some(c) = self.buf.cell(pos) else {
                continue;
            };
            h = fnv(h, c.symbol().as_bytes());
            h = fnv(
                h,
                format!("{:?}|{:?}|{}", c.fg, c.bg, c.modifier.bits()).as_bytes(),
            );
        }
        h
    }

    /// The frame as text, rows joined with newlines.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for y in 0..self.area.height {
            if y > 0 {
                out.push('\n');
            }
            out.push_str(&crate::harness::row_text(&self.buf, y).0);
        }
        out
    }

    /// The buffer.
    pub const fn buffer(&self) -> &Buffer {
        &self.buf
    }

    /// The area.
    pub const fn area(&self) -> Rect {
        self.area
    }

    /// The headless runtime, when the scene owns one.
    pub const fn runtime(&self) -> Option<&Runtime<NoApp>> {
        self.rt.as_ref()
    }

    /// The headless runtime, mutably.
    pub const fn runtime_mut(&mut self) -> Option<&mut Runtime<NoApp>> {
        self.rt.as_mut()
    }

    /// Last frame's registry.
    pub fn registry(&self) -> Option<&Registry> {
        self.rt.as_ref().map(Runtime::registry)
    }

    /// Last frame's ring.
    pub fn ring(&self) -> Option<&FocusRing> {
        self.rt.as_ref().map(Runtime::ring)
    }

    /// The baseline line key: `name w h theme color`.
    pub fn key(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.name,
            self.area.width,
            self.area.height,
            self.theme_name,
            color_level_label(self.color)
        )
    }

    /// Compare against (or, under `BLESS=1`, write into) a baseline file.
    pub fn assert_against(&self, baseline: &Baseline) {
        let key = self.key();
        let digest = format!("{:016x}", self.digest());
        let mut lines = baseline.read();
        if std::env::var_os("BLESS").is_some_and(|v| !v.is_empty() && v != "0") {
            lines.insert(key, digest);
            baseline.write(&lines);
            return;
        }
        match lines.get(&key) {
            Some(expected) => assert_eq!(
                *expected,
                digest,
                "digest of `{key}` changed; classify the change against §20.10, capture it, then BLESS=1\n{}",
                self.text()
            ),
            None => panic!(
                "no baseline for `{key}` in {}; run with BLESS=1 to record it",
                baseline.path
            ),
        }
    }
}

/// A baseline file: one `name w h theme color hash` line each, sorted.
#[derive(Debug, Clone, Copy)]
pub struct Baseline {
    path: &'static str,
}

impl Baseline {
    /// A baseline at `path`.
    pub const fn new(path: &'static str) -> Self {
        Baseline { path }
    }

    /// The path.
    pub const fn path(&self) -> &'static str {
        self.path
    }

    fn read(&self) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        let Ok(text) = std::fs::read_to_string(self.path) else {
            return out;
        };
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, hash)) = line.rsplit_once(' ') {
                out.insert(key.to_owned(), hash.to_owned());
            }
        }
        out
    }

    fn write(&self, lines: &BTreeMap<String, String>) {
        let mut s = String::from(
            "# digest baseline: name w h theme color hash — regenerate with BLESS=1, review like source\n",
        );
        for (k, v) in lines {
            s.push_str(k);
            s.push(' ');
            s.push_str(v);
            s.push('\n');
        }
        if let Some(dir) = Path::new(self.path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        std::fs::write(self.path, s).expect("write baseline");
    }
}
