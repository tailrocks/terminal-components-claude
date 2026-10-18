#![doc = r" Shared performance-measurement harness (`docs/audit/performance-audit.md`"]
#![doc = r" §7.1, moved here from `tests/perf_common.rs` by Slice 3 with the same"]
#![doc = r" semantics). `#[global_allocator]` is per binary: every perf test binary"]
#![doc = r" declares"]
#![doc = r""]
#![doc = r" ```ignore"]
#![doc = r" #[global_allocator]"]
#![doc = r" static GLOBAL: termrock_test_support::perf::Counting = termrock_test_support::perf::Counting;"]
#![doc = r" ```"]
#![doc = r""]
#![doc = r" Allocation counters are per thread, so counts stay exact with any"]
#![doc = r" `--test-threads`; benchmarks inside one process are serialised by"]
#![doc = r" [`lock`]. Only wall time benefits from `--test-threads=1`."]
#![doc = r""]
#![doc = r" Environment knobs:"]
#![doc = r" - `PERF_BLESS=1`    rewrite the baseline with this run's numbers."]
#![doc = r" - `PERF_STRICT=1`   also assert wall time against `baseline × 1.2`, and every"]
#![doc = r#"   "within N×" ratio (`PERF_TARGET` is folded into it — §16.6 declares exactly"#]
#![doc = r"   two knobs, MI-14)."]
#![doc = r" - `PERF_ITERS=n`    cap every benchmark's iteration count at `n`."]
#![doc = r" - `PERF_FULL=1`     use full data sizes even in debug builds."]
#![doc = r" - `PERF_BASELINE`   the baseline file (default: `crates/tui/tests/perf_baseline.txt`)."]
#![doc = r""]
#![doc = r" Allocation and byte counts are hard-asserted against the baseline in"]
#![doc = r" release builds (any increase fails). Debug builds always report and print"]
#![doc = r" `PERF-DEBUG-MISMATCH` when they differ, but do not fail."]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

thread_local! {
    static T_ALLOCS: Cell<usize> = const { Cell::new(0) };
    static T_BYTES: Cell<usize> = const { Cell::new(0) };
}

#[doc = r" Allocations made by the current thread since it started."]
pub fn allocs() -> usize {
    T_ALLOCS.try_with(Cell::get).unwrap_or(0)
}

#[doc = r" Bytes requested by the current thread since it started."]
pub fn bytes() -> usize {
    T_BYTES.try_with(Cell::get).unwrap_or(0)
}

#[inline]
fn count(bytes: usize) {
    let _ = T_ALLOCS.try_with(|c| c.set(c.get().saturating_add(1)));
    let _ = T_BYTES.try_with(|c| c.set(c.get().saturating_add(bytes)));
}

#[doc = r" Counting shim: every allocation and reallocation is one tick on the"]
#[doc = r" calling thread's counter; bytes accumulate requested sizes (reallocation"]
#[doc = r" adds the growth only)."]
#[derive(Debug, Default, Clone, Copy)]
pub struct Counting;

#[expect(unsafe_code, reason = "counting allocator; see SAFETY")]
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        count(l.size());

        unsafe { System.alloc(l) }
    }

    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }

    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        count(l.size());

        unsafe { System.alloc_zeroed(l) }
    }

    unsafe fn realloc(&self, p: *mut u8, l: Layout, new_size: usize) -> *mut u8 {
        count(new_size.saturating_sub(l.size()));

        unsafe { System.realloc(p, l, new_size) }
    }
}

static LOCK: Mutex<()> = Mutex::new(());

#[doc = r" Hold this for the whole body of a perf test so benchmarks in one process"]
#[doc = r" never time each other; allocation counts are per thread and need no lock."]
pub fn lock() -> MutexGuard<'static, ()> {
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[doc = r" One benchmark's medians."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    #[doc = r" Median wall time per iteration."]
    pub ns: u128,
    #[doc = r" Allocations per iteration (median across repetitions)."]
    pub allocs: usize,
    #[doc = r" Requested bytes per iteration (median across repetitions)."]
    pub bytes: usize,
    #[doc = r" Registry length after the last frame, for screen benchmarks."]
    pub hits: Option<usize>,
    #[doc = r" Reachable ring length after the last frame."]
    pub ring: Option<usize>,
}

impl Stats {
    #[doc = r" Attach the region counts."]
    #[must_use]
    pub const fn with_regions(mut self, hits: usize, ring: usize) -> Self {
        self.hits = Some(hits);
        self.ring = Some(ring);
        self
    }
}

#[doc = r" Whether an environment flag is set to something other than empty or `0`."]
pub fn env_flag(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|v| !v.is_empty() && v != "0")
}

#[doc = r" Full data sizes in release (or with `PERF_FULL=1`); one tenth in debug."]
pub fn big(n: usize) -> usize {
    if cfg!(debug_assertions) && !env_flag("PERF_FULL") {
        (n / 10).max(1)
    } else {
        n
    }
}

#[doc = r" Iteration count: the release default, reduced in debug, capped by `PERF_ITERS`."]
pub fn iters(release_default: usize) -> usize {
    let mut n = if cfg!(debug_assertions) {
        (release_default / 10).max(1)
    } else {
        release_default
    };
    if let Some(cap) = std::env::var("PERF_ITERS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        n = n.min(cap.max(1));
    }
    n
}

const REPS: usize = if cfg!(debug_assertions) { 3 } else { 9 };

#[doc = r" Warm up `warm` iterations, then run `REPS` repetitions of `iters`"]
#[doc = r" iterations each and return the per-iteration medians."]
pub fn bench(warm: usize, iters: usize, f: &mut dyn FnMut()) -> Stats {
    let iters = iters.max(1);
    for _ in 0..warm {
        f();
    }
    let mut ns_v = Vec::with_capacity(REPS);
    let mut allocs_v = Vec::with_capacity(REPS);
    let mut bytes_v = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        let a0 = allocs();
        let b0 = bytes();
        let t0 = Instant::now();
        for _ in 0..iters {
            f();
        }
        let dt = t0.elapsed().as_nanos();
        let a = allocs() - a0;
        let b = bytes() - b0;
        ns_v.push(dt / iters as u128);
        allocs_v.push(a.div_ceil(iters));
        bytes_v.push(b.div_ceil(iters));
    }
    ns_v.sort_unstable();
    allocs_v.sort_unstable();
    bytes_v.sort_unstable();
    Stats {
        ns: ns_v[REPS / 2],
        allocs: allocs_v[REPS / 2],
        bytes: bytes_v[REPS / 2],
        hits: None,
        ring: None,
    }
}

#[doc = r" Accumulates only explicitly measured sections of one benchmark iteration."]
#[doc = r" Setup between sections is excluded. Clock-read overhead is retained; no"]
#[doc = r" calibration subtraction can make an operation appear faster than measured."]
#[derive(Debug, Default)]
pub struct Sample {
    ns: u128,
    allocs: usize,
    bytes: usize,
}

impl Sample {
    #[doc = r" Measure one section, including its allocations, and return its result."]
    pub fn measure<T>(&mut self, f: impl FnOnce() -> T) -> T {
        let a0 = allocs();
        let b0 = bytes();
        let t0 = Instant::now();
        let result = f();
        self.ns += t0.elapsed().as_nanos();
        self.allocs += allocs() - a0;
        self.bytes += bytes() - b0;
        result
    }
}

#[doc = r" Like [`bench()`], but each iteration explicitly marks its measured sections."]
#[doc = r" Use when a public lifecycle requires untimed setup between operations."]
#[doc = r" Always retain a separate [`bench()`] measurement of the complete lifecycle."]
pub fn bench_sampled(warm: usize, iters: usize, f: &mut dyn FnMut(&mut Sample)) -> Stats {
    let iters = iters.max(1);
    for _ in 0..warm {
        f(&mut Sample::default());
    }
    let mut ns_v = Vec::with_capacity(REPS);
    let mut allocs_v = Vec::with_capacity(REPS);
    let mut bytes_v = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        let mut sample = Sample::default();
        for _ in 0..iters {
            f(&mut sample);
        }
        ns_v.push(sample.ns / iters as u128);
        allocs_v.push(sample.allocs.div_ceil(iters));
        bytes_v.push(sample.bytes.div_ceil(iters));
    }
    ns_v.sort_unstable();
    allocs_v.sort_unstable();
    bytes_v.sort_unstable();
    Stats {
        ns: ns_v[REPS / 2],
        allocs: allocs_v[REPS / 2],
        bytes: bytes_v[REPS / 2],
        hits: None,
        ring: None,
    }
}

#[doc = r" Measure exactly one execution of `f` (no repetitions, no median)."]
pub fn measure_once(f: &mut dyn FnMut()) -> Stats {
    let a0 = allocs();
    let b0 = bytes();
    let t0 = Instant::now();
    f();
    let ns = t0.elapsed().as_nanos();
    Stats {
        ns,
        allocs: allocs() - a0,
        bytes: bytes() - b0,
        hits: None,
        ring: None,
    }
}

#[derive(Debug, Clone, Copy)]
struct Entry {
    ns: u128,
    allocs: usize,
    bytes: usize,
    hits: Option<usize>,
    ring: Option<usize>,
}

#[doc = r" The baseline file: `PERF_BASELINE` or the library's checked-in file."]
pub fn baseline_path() -> String {
    std::env::var("PERF_BASELINE").unwrap_or_else(|_| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tui/tests/perf_baseline.txt"
        )
        .to_owned()
    })
}

fn read_baseline(path: &str) -> BTreeMap<String, Entry> {
    let mut out = BTreeMap::new();
    let Ok(text) = std::fs::read_to_string(path) else {
        return out;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 4 {
            continue;
        }
        let (Ok(ns), Ok(allocs), Ok(bytes)) = (f[1].parse(), f[2].parse(), f[3].parse()) else {
            continue;
        };
        let hits = f.get(4).and_then(|v| v.parse().ok());
        let ring = f.get(5).and_then(|v| v.parse().ok());
        out.insert(
            f[0].to_owned(),
            Entry {
                ns,
                allocs,
                bytes,
                hits,
                ring,
            },
        );
    }
    out
}

#[doc = r" Rewrite the baseline, **preserving the existing `#` header comments**: a"]
#[doc = r" re-blessed baseline records why a number moved, and blessing must not"]
#[doc = r" silently delete that reason."]
fn write_baseline(path: &str, map: &BTreeMap<String, Entry>) {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut header = String::new();
    for l in existing
        .lines()
        .take_while(|l| l.trim_start().starts_with('#') || l.trim().is_empty())
    {
        header.push_str(l);
        header.push('\n');
    }
    let mut s = if header.trim().is_empty() {
        String::from(
            "# perf baseline: name ns allocs bytes [hits ring]\n\
             # regenerate with PERF_BLESS=1 in a release build; review like a snapshot\n",
        )
    } else {
        header
    };
    for (name, e) in map {
        s.push_str(&format!("{name} {} {} {}", e.ns, e.allocs, e.bytes));
        if let (Some(h), Some(r)) = (e.hits, e.ring) {
            s.push_str(&format!(" {h} {r}"));
        }
        s.push('\n');
    }
    std::fs::write(path, s).expect("write perf baseline");
}

fn region_mismatch(name: &str, message: &str, release: bool) {
    assert!(!release, "{name}: {message}");
    println!("PERF-DEBUG-MISMATCH {name}: {message}");
}

fn check_regions(name: &str, s: &Stats, base: &Entry, release: bool) {
    match (s.hits, base.hits) {
        (Some(h), Some(bh)) => {
            let lo = bh - bh / 10;
            let hi = bh + bh / 10;
            if h < lo || h > hi {
                region_mismatch(
                    name,
                    &format!("hit registry size {h} outside baseline {bh} ±10 %"),
                    release,
                );
            }
        }
        (None, None) => {}
        (actual, expected) => region_mismatch(
            name,
            &format!("hit registry metric presence {actual:?} differs from baseline {expected:?}"),
            release,
        ),
    }
    match (s.ring, base.ring) {
        (Some(ring), Some(expected)) if ring != expected => region_mismatch(
            name,
            &format!("reachable ring length {ring} differs from baseline {expected}"),
            release,
        ),
        (Some(_), Some(_)) | (None, None) => {}
        (actual, expected) => region_mismatch(
            name,
            &format!(
                "reachable ring metric presence {actual:?} differs from baseline {expected:?}"
            ),
            release,
        ),
    }
}

#[doc = r" One machine-readable line per benchmark, then the baseline policy against"]
#[doc = r" the default baseline file."]
pub fn report(name: &str, s: &Stats) {
    report_to(&baseline_path(), name, s);
}

#[doc = r" [`report`] against an explicit baseline file."]
pub fn report_to(path: &str, name: &str, s: &Stats) {
    let mut line = format!(
        "PERF {name} ns={} allocs={} bytes={}",
        s.ns, s.allocs, s.bytes
    );
    if let (Some(h), Some(r)) = (s.hits, s.ring) {
        line.push_str(&format!(" hits={h} ring={r}"));
    }
    println!("{line}");

    if env_flag("PERF_BLESS") {
        let mut map = read_baseline(path);
        map.insert(
            name.to_owned(),
            Entry {
                ns: s.ns,
                allocs: s.allocs,
                bytes: s.bytes,
                hits: s.hits,
                ring: s.ring,
            },
        );
        write_baseline(path, &map);
        return;
    }
    let map = read_baseline(path);
    let Some(base) = map.get(name) else {
        println!("PERF-NOBASE {name} (run with PERF_BLESS=1 to record)");
        return;
    };
    let release = !cfg!(debug_assertions);
    let counts_match = s.allocs <= base.allocs && s.bytes <= base.bytes;
    if !counts_match {
        assert!(
            !release,
            "{name}: allocation regression: allocs {} > {} or bytes {} > {}",
            s.allocs, base.allocs, s.bytes, base.bytes
        );
        println!(
            "PERF-DEBUG-MISMATCH {name} allocs={} base={} bytes={} base={}",
            s.allocs, base.allocs, s.bytes, base.bytes
        );
    }
    check_regions(name, s, base, release);
    if s.ns > base.ns + base.ns / 5 {
        assert!(
            !(env_flag("PERF_STRICT") && release),
            "{name}: time regression: ns {} > baseline {} × 1.2",
            s.ns,
            base.ns
        );
        println!("REGRESSION? {name} ns={} baseline={}", s.ns, base.ns);
    }
}

#[doc = r#" Ratio helper for the "within N×" acceptance thresholds: prints the ratio"#]
#[doc = r" and asserts it only when `strict` is set."]
pub fn check_ratio(name: &str, a: u128, b: u128, max: f64, strict: bool) {
    let ratio = a as f64 / b.max(1) as f64;
    println!("PERF-RATIO {name} ratio={ratio:.2} max={max}");
    if strict {
        assert!(ratio <= max, "{name}: ratio {ratio:.2} exceeds {max}");
    }
}

#[doc = r" One line of `n` graphemes whose symbols all fit ratatui `Cell`'s inline"]
#[doc = r" `CompactString` storage: ASCII, CJK (width 2) and combining marks, and no"]
#[doc = r" ZWJ sequence. A painter over this corpus must record **0** allocations —"]
#[doc = r" any count above zero is the painter's, not the buffer's (adjudication 4)."]
pub fn unicode_line_inline(n: usize) -> String {
    const PARTS: [&str; 6] = ["a", "b", "漢", "字", "e\u{301}", "\u{00E9}"];
    let mut s = String::with_capacity(n * 4);
    for i in 0..n {
        s.push_str(PARTS[i % PARTS.len()]);
    }
    s
}

#[doc = r" One line of `n` graphemes mixing ASCII, CJK (width 2), combining marks and"]
#[doc = r" an emoji ZWJ sequence. The ZWJ symbol exceeds `Cell`'s inline storage, so"]
#[doc = r" each such **cell** heap-allocates — a property of the buffer, not of the"]
#[doc = r" painter."]
pub fn unicode_line(n: usize) -> String {
    const PARTS: [&str; 8] = [
        "a",
        "b",
        "漢",
        "字",
        "e\u{301}",
        "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
        "c",
        "\u{00E9}",
    ];
    let mut s = String::with_capacity(n * 4);
    for i in 0..n {
        s.push_str(PARTS[i % PARTS.len()]);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::{Entry, Stats, bench_sampled, check_regions, count};

    const BASE: Entry = Entry {
        ns: 1,
        allocs: 0,
        bytes: 0,
        hits: Some(100),
        ring: Some(4),
    };

    const fn stats(hits: Option<usize>, ring: Option<usize>) -> Stats {
        Stats {
            ns: 1,
            allocs: 0,
            bytes: 0,
            hits,
            ring,
        }
    }

    #[test]
    fn sampled_sections_exclude_setup_and_include_each_measured_allocation() {
        let stats = bench_sampled(2, 3, &mut |sample| {
            count(17);
            assert_eq!(
                sample.measure(|| {
                    count(31);
                    7
                }),
                7
            );
            count(19);
            sample.measure(|| count(37));
        });
        assert_eq!(stats.allocs, 2);
        assert_eq!(stats.bytes, 68);
    }

    #[test]
    fn region_metrics_accept_hit_tolerance_and_exact_ring() {
        check_regions("screen", &stats(Some(90), Some(4)), &BASE, true);
        check_regions("screen", &stats(Some(110), Some(4)), &BASE, true);
    }

    #[test]
    #[should_panic(expected = "reachable ring length 3 differs from baseline 4")]
    fn reachable_ring_mismatch_fails_in_release() {
        check_regions("screen", &stats(Some(100), Some(3)), &BASE, true);
    }

    #[test]
    #[should_panic(expected = "hit registry size 111 outside baseline 100 ±10 %")]
    fn hit_registry_mismatch_still_fails_in_release() {
        check_regions("screen", &stats(Some(111), Some(4)), &BASE, true);
    }

    #[test]
    #[should_panic(expected = "reachable ring metric presence None differs from baseline Some(4)")]
    fn missing_region_metric_fails_in_release() {
        check_regions("screen", &stats(Some(100), None), &BASE, true);
    }
}
