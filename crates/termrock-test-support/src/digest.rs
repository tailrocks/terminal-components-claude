use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock::{App, ColorLevel, Cx, FocusRing, Registry, Response, Runtime, Theme, Ui};

#[derive(Debug, Default, Clone, Copy)]
pub struct NoApp;

impl App for NoApp {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, _ui: &mut Ui<'_>) {}
}

pub struct Scene {
    name: &'static str,
    theme_name: &'static str,
    color: ColorLevel,
    area: Rect,
    buf: Buffer,
    rt: Option<Runtime<NoApp>>,
    snapshot: termrock::RenderSnapshot,
}

pub struct SceneProjection<'a, M: ?Sized, F> {
    scene: &'a mut Scene,
    binding: termrock::RenderModel<'a, M, F>,
}

impl<M: ?Sized, F> core::fmt::Debug for SceneProjection<'_, M, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SceneProjection").finish_non_exhaustive()
    }
}

impl<M: ?Sized, F: Fn(&M, &mut Ui<'_>, Rect)> SceneProjection<'_, M, F> {
    pub fn draw(&mut self) {
        let Some(rt) = self.scene.rt.as_mut() else {
            return;
        };
        self.scene.buf.reset();
        rt.draw_bound_projection(self.scene.area, &mut self.scene.buf, &self.binding)
            .commit_inspected();
    }

    pub const fn scene(&self) -> &Scene {
        self.scene
    }
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
            snapshot: termrock::RenderSnapshot::default(),
        }
    }

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
            snapshot: termrock::RenderSnapshot::default(),
        }
    }

    #[must_use]
    pub const fn named(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub fn set_snapshot(&mut self, snapshot: termrock::RenderSnapshot) {
        self.snapshot = snapshot;
    }

    pub fn bind_model<'a, M: ?Sized, F>(
        &'a mut self,
        model: &'a M,
        paint: F,
    ) -> SceneProjection<'a, M, F>
    where
        F: Fn(&M, &mut Ui<'_>, Rect),
    {
        let binding = self.snapshot.bind_model(model, paint);
        SceneProjection {
            scene: self,
            binding,
        }
    }

    pub fn bind_app<'a, A: App>(
        &'a mut self,
        app: &'a A,
    ) -> SceneProjection<'a, A, impl Fn(&A, &mut Ui<'_>, Rect)> {
        self.bind_model(app, |app, ui, _| app.draw(ui))
    }

    pub fn draw_app<A: App>(&mut self, app: &A) {
        self.draw(|ui, _| app.draw(ui));
    }

    pub fn draw(&mut self, f: impl FnOnce(&mut Ui<'_>, Rect)) {
        let area = self.area;
        let Some(rt) = self.rt.as_mut() else {
            return;
        };
        self.buf.reset();
        rt.draw_projection(area, &mut self.buf, &self.snapshot, f)
            .commit_inspected();
    }

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
        rt.draw_projection(area, &mut self.buf, &self.snapshot, f)
            .commit_inspected();
    }

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

    pub const fn buffer(&self) -> &Buffer {
        &self.buf
    }

    pub const fn area(&self) -> Rect {
        self.area
    }

    pub const fn runtime(&self) -> Option<&Runtime<NoApp>> {
        self.rt.as_ref()
    }

    pub const fn runtime_mut(&mut self) -> Option<&mut Runtime<NoApp>> {
        self.rt.as_mut()
    }

    pub fn registry(&self) -> Option<&Registry> {
        self.rt
            .as_ref()
            .map(|rt| rt.projection_registry().unwrap_or_else(|| rt.registry()))
    }

    pub fn ring(&self) -> Option<&FocusRing> {
        self.rt
            .as_ref()
            .map(|rt| rt.projection_ring().unwrap_or_else(|| rt.ring()))
    }

    pub fn cursor_position(&self) -> Option<ratatui_core::layout::Position> {
        self.rt.as_ref().and_then(Runtime::projection_cursor)
    }

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

    pub fn assert_against(&self, baseline: &Baseline) {
        let key = self.key();
        let digest = self.digest();
        if bless_enabled() {
            baseline.bless(&key, &format!("{digest:016x}"));
            return;
        }

        let why = match baseline.lookup(&key, digest) {
            Lookup::Match => return,
            Lookup::Mismatch(expected) => format!(
                "digest of `{key}` changed: baseline {expected}, got {digest:016x}; \
                 classify the change against §20.10, capture it, then BLESS=1"
            ),
            Lookup::Missing => format!(
                "no baseline for `{key}` in {}; run with BLESS=1 to record it",
                baseline.path
            ),
        };
        panic!("{why}\n{}", self.text());
    }
}

fn bless_enabled() -> bool {
    let enabled = |v: std::ffi::OsString| !v.is_empty() && v != "0";
    match std::env::var_os("BLESS") {
        Some(value) => enabled(value),
        None => std::env::var_os("UPDATE_BASELINE").is_some_and(enabled),
    }
}

enum Lookup {
    Match,
    Mismatch(String),
    Missing,
}

const HEADER: &str =
    "# digest baseline: name w h theme color hash — regenerate with BLESS=1, review like source\n";

fn store() -> MutexGuard<'static, BTreeMap<&'static str, BTreeMap<String, String>>> {
    static STORE: OnceLock<Mutex<BTreeMap<&'static str, BTreeMap<String, String>>>> =
        OnceLock::new();
    STORE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
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

#[derive(Debug, Clone, Copy)]
pub struct Baseline {
    path: &'static str,
}

impl Baseline {
    pub const fn new(path: &'static str) -> Self {
        Baseline { path }
    }

    pub const fn path(&self) -> &'static str {
        self.path
    }

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

    fn bless(&self, key: &str, digest: &str) {
        let lock_path = format!("{}.lock", self.path);
        let start = std::time::Instant::now();
        let _lock = loop {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(file) => break file,
                Err(_) => {
                    if start.elapsed() > std::time::Duration::from_secs(10) {
                        let _ = std::fs::remove_file(&lock_path);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
        };
        struct LockGuard<'a>(&'a str);
        impl Drop for LockGuard<'_> {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(self.0);
            }
        }
        let _guard = LockGuard(&lock_path);

        let mut entries = std::fs::read_to_string(self.path)
            .map(|t| parse(&t))
            .unwrap_or_default();
        entries.insert(key.to_owned(), digest.to_owned());
        let text = render(&entries);
        self.publish(&text);
    }

    fn publish(&self, text: &str) {
        if let Some(dir) = Path::new(self.path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }

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

    use termrock::{Family, Part, StateFlags, Variant};

    use super::*;

    const PATH_ENV: &str = "TUI_NEXT_DIGEST_TEST_BASELINE";
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
            "termrock-digest-{tag}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir.join("baseline.txt")
    }

    fn run_worker(path: &Path, slice: usize, bless: bool) -> std::process::Output {
        run_worker_with_flags(path, slice, bless.then_some("1"), None)
    }

    fn run_worker_with_flags(
        path: &Path,
        slice: usize,
        bless: Option<&str>,
        update_baseline: Option<&str>,
    ) -> std::process::Output {
        let exe = std::env::current_exe().expect("test binary");
        let mut cmd = Command::new(exe);
        cmd.args(["--exact", WORKER, "--ignored", "--nocapture"])
            .env(PATH_ENV, path)
            .env(SLICE_ENV, slice.to_string())
            .env_remove("BLESS")
            .env_remove("UPDATE_BASELINE");
        if let Some(value) = bless {
            cmd.env("BLESS", value);
        }
        if let Some(value) = update_baseline {
            cmd.env("UPDATE_BASELINE", value);
        }
        cmd.output().expect("run worker")
    }

    #[test]
    fn update_baseline_is_an_alias_with_bless_precedence() {
        let cases = [
            ("alias", None, Some("1"), true),
            ("canonical", Some("1"), Some("0"), true),
            ("canonical_false", Some("0"), Some("1"), false),
        ];
        for (tag, bless, update_baseline, writes) in cases {
            let path = temp_baseline(tag);
            let out = run_worker_with_flags(&path, 0, bless, update_baseline);
            assert_eq!(
                out.status.success(),
                writes,
                "unexpected baseline mode for {tag}: {}",
                output_text(&out)
            );
            assert_eq!(path.exists(), writes, "baseline write decision for {tag}");
            if writes {
                let text = std::fs::read_to_string(&path).expect("baseline written");
                assert!(text.starts_with(HEADER), "header lost for {tag}: {text}");
            }
            let _ = std::fs::remove_dir_all(path.parent().expect("temp dir"));
        }
    }

    #[test]
    fn baseline_false_values_do_not_write_or_leak_environment_values() {
        let cases = [
            ("bless-empty", Some(""), None),
            ("bless-zero", Some("0"), None),
            ("alias-empty", None, Some("")),
            ("alias-zero", None, Some("0")),
        ];
        for (tag, bless, update_baseline) in cases {
            let path = temp_baseline(tag);
            let out = run_worker_with_flags(&path, 0, bless, update_baseline);
            assert!(!out.status.success(), "false value blessed for {tag}");
            assert!(!path.exists(), "false value wrote a baseline for {tag}");
            let _ = std::fs::remove_dir_all(path.parent().expect("temp dir"));
        }

        let secret = "baseline-secret-must-not-escape";
        let path = temp_baseline("secret-value");
        let out = run_worker_with_flags(&path, 0, None, Some(secret));
        assert!(out.status.success(), "a truthy alias must bless");
        let text = std::fs::read_to_string(&path).expect("baseline written");
        assert!(
            !text.contains(secret),
            "baseline contents leaked the flag value"
        );
        assert!(
            !output_text(&out).contains(secret),
            "baseline flag value leaked into worker output"
        );
        let _ = std::fs::remove_dir_all(path.parent().expect("temp dir"));

        let path = temp_baseline("ignored-secret");
        let out = run_worker_with_flags(&path, 0, Some("0"), Some(secret));
        assert!(!out.status.success(), "BLESS=0 must remain fail-closed");
        assert!(!path.exists(), "ignored alias must not write a baseline");
        assert!(
            !output_text(&out).contains(secret),
            "baseline flag value leaked into worker output"
        );
        let _ = std::fs::remove_dir_all(path.parent().expect("temp dir"));
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

        let out = run_worker(&path, 0, true);
        assert!(out.status.success(), "{}", output_text(&out));
        assert_eq!(
            std::fs::read_to_string(&path).expect("baseline"),
            text,
            "re-blessing rewrote the file"
        );
        let _ = std::fs::remove_dir_all(path.parent().expect("temp dir"));
    }

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

    #[test]
    fn a_missing_baseline_prints_the_frame_it_would_record() {
        let path = temp_baseline("missing");
        let out = run_worker(&path, 0, false);
        assert!(
            !out.status.success(),
            "a missing baseline must fail the run"
        );
        let log = output_text(&out);
        let frame = worker_scene("bless.00").text();
        assert!(
            log.contains(&format!(
                "no baseline for `bless.00 10 3 junie truecolor` in {}; \
                 run with BLESS=1 to record it\n{frame}",
                path.display()
            )),
            "the failure must name the scene and print its frame:\n{log}"
        );
        assert!(!path.exists(), "a failing non-bless run wrote the baseline");
        let _ = std::fs::remove_dir_all(path.parent().expect("temp dir"));
    }
}
