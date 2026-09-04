//! Headless digest scenes (`COMPONENT_ARCHITECTURE.md` §16.3, §21 item 28).
//!
//! A `Scene` owns a headless runtime (registry + ring + layers + style
//! stack) built from a theme, draws a closure into a buffer and produces an
//! FNV-1a digest of `(symbol, fg, bg, modifier)` per cell. Baselines are
//! one `name w h theme color hash` line each, sorted; `BLESS=1` merges.
//!
//! Blessing is safe at any `--test-threads` and across the several test
//! binaries that share one baseline file: every write is serialised by a
//! process-wide lock, merged into the file's *current* content (never a
//! snapshot read before the assertion) and published by an atomic rename,
//! so no reader ever observes a partial file and no writer drops another
//! writer's entries. A non-bless run reads each baseline once per process
//! and never touches the file.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

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

    /// Compare against (or, under `BLESS=1`, merge into) a baseline file.
    ///
    /// Thread- and process-safe in both directions: blessing merges one
    /// entry under a lock, comparing reads the file at most once per
    /// process and writes nothing.
    pub fn assert_against(&self, baseline: &Baseline) {
        let key = self.key();
        let digest = self.digest();
        if bless_enabled() {
            baseline.bless(&key, &format!("{digest:016x}"));
            return;
        }
        match baseline.lookup(&key, digest) {
            Lookup::Match => {}
            Lookup::Mismatch(expected) => panic!(
                "digest of `{key}` changed: baseline {expected}, got {digest:016x}; \
                 classify the change against §20.10, capture it, then BLESS=1\n{}",
                self.text()
            ),
            Lookup::Missing => panic!(
                "no baseline for `{key}` in {}; run with BLESS=1 to record it",
                baseline.path
            ),
        }
    }
}

/// Whether this process was asked to rewrite baselines.
fn bless_enabled() -> bool {
    std::env::var_os("BLESS").is_some_and(|v| !v.is_empty() && v != "0")
}

/// The outcome of one baseline lookup; the expected text is cloned only
/// when the assertion is about to fail.
enum Lookup {
    Match,
    Mismatch(String),
    Missing,
}

const HEADER: &str =
    "# digest baseline: name w h theme color hash — regenerate with BLESS=1, review like source\n";

/// Every baseline this process has touched, keyed by path: the file's
/// entries as first read, plus everything blessed since. One lock for all
/// paths — blessing is rare and never on a hot path.
fn store() -> MutexGuard<'static, BTreeMap<&'static str, BTreeMap<String, String>>> {
    static STORE: OnceLock<Mutex<BTreeMap<&'static str, BTreeMap<String, String>>>> =
        OnceLock::new();
    STORE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        // the map is consistent at every point a panic could unwind
        // through, so poisoning must never cascade a single failed scene
        // into every other scene in the same binary
        .unwrap_or_else(PoisonError::into_inner)
}

fn parse(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
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

fn render(entries: &BTreeMap<String, String>) -> String {
    let mut s = String::from(HEADER);
    for (k, v) in entries {
        s.push_str(k);
        s.push(' ');
        s.push_str(v);
        s.push('\n');
    }
    s
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

    /// Compare `digest` with the recorded entry for `key`. No file access
    /// after the first lookup of this path, so the assertion path of a
    /// non-bless run neither reads nor writes the baseline.
    fn lookup(&self, key: &str, digest: u64) -> Lookup {
        let mut store = store();
        let entries = store
            .entry(self.path)
            .or_insert_with(|| parse(&std::fs::read_to_string(self.path).unwrap_or_default()));
        match entries.get(key) {
            None => Lookup::Missing,
            Some(expected) => {
                if u64::from_str_radix(expected, 16).is_ok_and(|e| e == digest) {
                    Lookup::Match
                } else {
                    Lookup::Mismatch(expected.clone())
                }
            }
        }
    }

    /// Merge one entry into the file.
    ///
    /// Held under the process-wide lock: re-read the file, fold in whatever
    /// another process wrote since, apply this entry, render and publish by
    /// atomic rename. Nothing here rewrites the file from a snapshot taken
    /// before the assertion, so concurrent blessing cannot drop entries and
    /// no `--test-threads=1` is required.
    fn bless(&self, key: &str, digest: &str) {
        let mut store = store();
        let entries = store.entry(self.path).or_default();
        let on_disk = std::fs::read_to_string(self.path).ok();
        if let Some(text) = on_disk.as_deref() {
            for (k, v) in parse(text) {
                entries.entry(k).or_insert(v);
            }
        }
        entries.insert(key.to_owned(), digest.to_owned());
        let text = render(entries);
        if on_disk.as_deref() == Some(text.as_str()) {
            return; // already recorded: leave the file, and its mtime, alone
        }
        self.publish(&text);
    }

    /// Write `text` where a reader can only ever see the old or the new file.
    fn publish(&self, text: &str) {
        if let Some(dir) = Path::new(self.path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        // one writer per process (the store lock) and a pid-tagged name, so
        // two processes blessing the same baseline never share a temporary
        let tmp = format!("{}.tmp.{}", self.path, std::process::id());
        std::fs::write(&tmp, text).expect("write baseline");
        if let Err(e) = std::fs::rename(&tmp, self.path) {
            let _ = std::fs::remove_file(&tmp);
            panic!("publish baseline {}: {e}", self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::Barrier;

    use tui_next::{Family, Part, StateFlags, Variant};

    use super::*;

    /// Where a spawned worker writes; set by the driver tests below.
    const PATH_ENV: &str = "TUI_NEXT_DIGEST_TEST_BASELINE";
    /// Which half of [`NAMES`] a worker records.
    const SLICE_ENV: &str = "TUI_NEXT_DIGEST_TEST_SLICE";
    const WORKER: &str = "digest::tests::bless_worker";
    const PER_THREAD: usize = 6;

    static NAMES: [&str; 48] = [
        "bless.00", "bless.01", "bless.02", "bless.03", "bless.04", "bless.05", "bless.06",
        "bless.07", "bless.08", "bless.09", "bless.10", "bless.11", "bless.12", "bless.13",
        "bless.14", "bless.15", "bless.16", "bless.17", "bless.18", "bless.19", "bless.20",
        "bless.21", "bless.22", "bless.23", "bless.24", "bless.25", "bless.26", "bless.27",
        "bless.28", "bless.29", "bless.30", "bless.31", "bless.32", "bless.33", "bless.34",
        "bless.35", "bless.36", "bless.37", "bless.38", "bless.39", "bless.40", "bless.41",
        "bless.42", "bless.43", "bless.44", "bless.45", "bless.46", "bless.47",
    ];

    /// The regression worker, driven as a child process so `BLESS` can be
    /// set without mutating this process's environment. Every thread builds
    /// its scenes, waits on a barrier so the writes collide as hard as the
    /// scheduler allows, then asserts (or, under `BLESS=1`, records) them.
    #[test]
    #[ignore = "spawned by the concurrent-bless driver tests"]
    fn bless_worker() {
        let path = std::env::var(PATH_ENV).expect("baseline path");
        let baseline = Baseline::new(Box::leak(path.into_boxed_str()));
        let slice: usize = std::env::var(SLICE_ENV)
            .expect("slice")
            .parse()
            .expect("slice index");
        let half = NAMES.len() / 2;
        let names = &NAMES[slice * half..(slice + 1) * half];
        let barrier = Barrier::new(names.chunks(PER_THREAD).count());
        std::thread::scope(|s| {
            for chunk in names.chunks(PER_THREAD) {
                let barrier = &barrier;
                s.spawn(move || {
                    let scenes: Vec<Scene> = chunk.iter().copied().map(worker_scene).collect();
                    barrier.wait();
                    for scene in &scenes {
                        scene.assert_against(&baseline);
                    }
                });
            }
        });
    }

    fn worker_scene(name: &'static str) -> Scene {
        let mut scene = Scene::new(name, Theme::junie(), ColorLevel::TrueColor, 10, 3);
        scene.draw(|ui, area| {
            let r = ui.style(
                Family::BUTTON,
                Variant::PRIMARY,
                Part::CONTAINER,
                StateFlags::empty(),
            );
            ui.fill(area, r.style);
        });
        scene
    }

    fn temp_baseline(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!(
            "tui-next-digest-{tag}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir.join("baseline.txt")
    }

    /// Run the worker over `path` for `slice`, with or without `BLESS`.
    fn run_worker(path: &Path, slice: usize, bless: bool) -> std::process::Output {
        let exe = std::env::current_exe().expect("test binary");
        let mut cmd = Command::new(exe);
        cmd.args(["--exact", WORKER, "--ignored", "--nocapture"])
            .env(PATH_ENV, path)
            .env(SLICE_ENV, slice.to_string())
            .env_remove("BLESS");
        if bless {
            cmd.env("BLESS", "1");
        }
        cmd.output().expect("run worker")
    }

    fn entry_lines(text: &str) -> Vec<&str> {
        text.lines().filter(|l| !l.starts_with('#')).collect()
    }

    fn output_text(out: &std::process::Output) -> String {
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    }

    /// The truncation regression: blessing used to read the whole baseline,
    /// insert one entry and rewrite the file, so parallel threads (and the
    /// second test binary sharing one baseline) overwrote each other — a
    /// 387-line `components.txt` came back with 6 lines. Every entry of both
    /// worker processes must survive, sorted, with the header intact.
    #[test]
    fn concurrent_bless_keeps_every_entry() {
        let path = temp_baseline("concurrent");
        for slice in 0..2 {
            let out = run_worker(&path, slice, true);
            assert!(
                out.status.success(),
                "bless worker {slice} failed:\n{}",
                output_text(&out)
            );
        }
        let text = std::fs::read_to_string(&path).expect("baseline written");
        assert!(text.starts_with(HEADER), "header lost:\n{text}");
        let lines = entry_lines(&text);
        assert_eq!(
            lines.len(),
            NAMES.len(),
            "entries clobbered each other:\n{text}"
        );
        for name in NAMES {
            assert!(
                lines.iter().any(|l| l.starts_with(&format!("{name} "))),
                "`{name}` missing:\n{text}"
            );
        }
        let mut sorted = lines.clone();
        sorted.sort_unstable();
        assert_eq!(lines, sorted, "baseline order is not deterministic");
        // idempotent: re-blessing recorded scenes changes nothing at all
        let out = run_worker(&path, 0, true);
        assert!(out.status.success(), "{}", output_text(&out));
        assert_eq!(
            std::fs::read_to_string(&path).expect("baseline"),
            text,
            "re-blessing rewrote the file"
        );
        let _ = std::fs::remove_dir_all(path.parent().expect("temp dir"));
    }

    /// A non-bless run compares and never writes: it passes against a
    /// matching baseline, fails loudly against a corrupted one, and leaves
    /// the file byte-identical either way.
    #[test]
    fn non_bless_run_fails_loudly_and_never_writes() {
        let path = temp_baseline("assert");
        assert!(run_worker(&path, 0, true).status.success());
        let blessed = std::fs::read_to_string(&path).expect("baseline");

        let out = run_worker(&path, 0, false);
        assert!(
            out.status.success(),
            "matching baseline must pass:\n{}",
            output_text(&out)
        );
        assert_eq!(
            std::fs::read_to_string(&path).expect("baseline"),
            blessed,
            "a passing non-bless run wrote the baseline"
        );

        let corrupt = blessed
            .lines()
            .map(|l| {
                if l.starts_with("bless.00 ") {
                    "bless.00 10 3 junie truecolor 0000000000000000".to_owned()
                } else {
                    l.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        std::fs::write(&path, &corrupt).expect("corrupt baseline");
        let out = run_worker(&path, 0, false);
        assert!(!out.status.success(), "a mismatch must fail the run");
        let log = output_text(&out);
        assert!(
            log.contains("digest of `bless.00 10 3 junie truecolor` changed"),
            "the failure must name the scene:\n{log}"
        );
        assert_eq!(
            std::fs::read_to_string(&path).expect("baseline"),
            corrupt,
            "a failing non-bless run wrote the baseline"
        );
        let _ = std::fs::remove_dir_all(path.parent().expect("temp dir"));
    }
}
