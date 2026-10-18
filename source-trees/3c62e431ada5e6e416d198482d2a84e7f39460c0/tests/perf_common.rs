//! Shared performance-measurement harness (`docs/audit/performance-audit.md`
//! §7.1). This file is *included* (via `#[path]` / `mod perf_common;`) into
//! each perf test binary because `#[global_allocator]` is per binary: every
//! binary that includes it must declare
//!
//! ```ignore
//! #[global_allocator]
//! static GLOBAL: perf_common::Counting = perf_common::Counting;
//! ```
//!
//! Run in release with a single thread for meaningful timings:
//!
//! ```text
//! cargo test --release --test perf -- --test-threads=1 --nocapture
//! cargo test --release --bin showcase perf_tests -- --test-threads=1 --nocapture
//! cargo test --release --bin tablepro perf_tests -- --test-threads=1 --nocapture
//! cargo test --release --bin jackin-preview perf_tests -- --test-threads=1 --nocapture
//! ```
//!
//! Allocation counters are per thread, so counts stay exact with any
//! `--test-threads` even while unrelated tests in the same binary run;
//! benchmarks inside one process are additionally serialised by [`lock`].
//! Only wall time benefits from `--test-threads=1`.
//!
//! Environment knobs:
//! - `PERF_BLESS=1`   rewrite `tests/perf_baseline.txt` with this run's numbers.
//! - `PERF_STRICT=1`  also assert wall time against `baseline × 1.2`.
//! - `PERF_TARGET=1`  enable the post-refactor acceptance assertions (target
//!   allocation counts and "within N×" ratios); some are expected to fail
//!   on the pre-refactor tree.
//! - `PERF_ITERS=n`   cap every benchmark's iteration count at `n`.
//! - `PERF_FULL=1`    use full data sizes even in debug builds.
//!
//! Allocation and byte counts are hard-asserted against the baseline in
//! release builds (any increase fails). Debug builds always report and print
//! `PERF-DEBUG-MISMATCH` when they differ from the release baseline, but do
//! not fail: the baseline is recorded in release.
#![allow(dead_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

thread_local! {
    static T_ALLOCS: Cell<usize> = const { Cell::new(0) };
    static T_BYTES: Cell<usize> = const { Cell::new(0) };
}

/// Allocations made by the current thread since it started.
pub fn allocs() -> usize {
    T_ALLOCS.try_with(Cell::get).unwrap_or(0)
}

/// Bytes requested by the current thread since it started.
pub fn bytes() -> usize {
    T_BYTES.try_with(Cell::get).unwrap_or(0)
}

#[inline]
fn count(bytes: usize) {
    // `const`-initialised `Cell` thread-locals need no lazy init and no
    // destructor, so reading them inside the allocator cannot allocate;
    // `try_with` covers thread teardown.
    let _ = T_ALLOCS.try_with(|c| c.set(c.get() + 1));
    let _ = T_BYTES.try_with(|c| c.set(c.get() + bytes));
}

/// Counting shim: every allocation and reallocation is one tick on the
/// calling thread's counter; bytes accumulate requested sizes (reallocation
/// adds the growth only). Counting per thread keeps the numbers exact even
/// when other tests in the same binary run concurrently.
pub struct Counting;

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

/// Hold this for the whole body of a perf test so benchmarks in one process
/// never time each other; allocation counts are per thread and need no lock.
pub fn lock() -> MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    /// Median wall time per iteration.
    pub ns: u128,
    /// Allocations per iteration (median across repetitions).
    pub allocs: usize,
    /// Requested bytes per iteration (median across repetitions).
    pub bytes: usize,
    /// `HitRegistry::len()` after the last frame, for screen benchmarks.
    pub hits: Option<usize>,
    /// `FocusRing::reachable().len()` after the last frame.
    pub ring: Option<usize>,
}

impl Stats {
    pub fn with_regions(mut self, hits: usize, ring: usize) -> Self {
        self.hits = Some(hits);
        self.ring = Some(ring);
        self
    }
}

pub fn env_flag(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|v| !v.is_empty() && v != "0")
}

/// Full data sizes in release (or with `PERF_FULL=1`); one tenth in debug so
/// the whole suite stays within the debug test budget.
pub fn big(n: usize) -> usize {
    if cfg!(debug_assertions) && !env_flag("PERF_FULL") {
        (n / 10).max(1)
    } else {
        n
    }
}

/// Iteration count for a benchmark: the release default, reduced in debug
/// and capped by `PERF_ITERS`.
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

/// Warm up `warm` iterations, then run `REPS` repetitions of `iters`
/// iterations each and return the per-iteration medians.
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

/// Measure exactly one execution of `f` (no repetitions, no median).
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

// ---------------------------------------------------------------- baseline

#[derive(Debug, Clone, Copy)]
struct Entry {
    ns: u128,
    allocs: usize,
    bytes: usize,
    hits: Option<usize>,
    ring: Option<usize>,
}

fn baseline_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/perf_baseline.txt")
}

fn read_baseline() -> BTreeMap<String, Entry> {
    let mut out = BTreeMap::new();
    let Ok(text) = std::fs::read_to_string(baseline_path()) else {
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

fn write_baseline(map: &BTreeMap<String, Entry>) {
    let mut s = String::from(
        "# perf baseline: name ns allocs bytes [hits ring]\n\
         # regenerate with PERF_BLESS=1 in a release build; review like a snapshot\n",
    );
    for (name, e) in map {
        s.push_str(&format!("{name} {} {} {}", e.ns, e.allocs, e.bytes));
        if let (Some(h), Some(r)) = (e.hits, e.ring) {
            s.push_str(&format!(" {h} {r}"));
        }
        s.push('\n');
    }
    std::fs::write(baseline_path(), s).expect("write perf baseline");
}

/// One machine-readable line per benchmark, then the baseline policy:
/// bless, or compare (allocations/bytes/hits hard in release, time only
/// under `PERF_STRICT`).
pub fn report(name: &str, s: &Stats) {
    let mut line = format!(
        "PERF {name} ns={} allocs={} bytes={}",
        s.ns, s.allocs, s.bytes
    );
    if let (Some(h), Some(r)) = (s.hits, s.ring) {
        line.push_str(&format!(" hits={h} ring={r}"));
    }
    println!("{line}");

    if env_flag("PERF_BLESS") {
        let mut map = read_baseline();
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
        write_baseline(&map);
        return;
    }
    let map = read_baseline();
    let Some(base) = map.get(name) else {
        println!("PERF-NOBASE {name} (run with PERF_BLESS=1 to record)");
        return;
    };
    let release = !cfg!(debug_assertions);
    let counts_match = s.allocs <= base.allocs && s.bytes <= base.bytes;
    if !counts_match {
        if release {
            panic!(
                "{name}: allocation regression: allocs {} > {} or bytes {} > {}",
                s.allocs, base.allocs, s.bytes, base.bytes
            );
        }
        println!(
            "PERF-DEBUG-MISMATCH {name} allocs={} base={} bytes={} base={}",
            s.allocs, base.allocs, s.bytes, base.bytes
        );
    }
    if let (Some(h), Some(bh)) = (s.hits, base.hits) {
        // §7.2 F `hit_registry_size_is_bounded`: ±10 % of the recorded size
        let lo = bh - bh / 10;
        let hi = bh + bh / 10;
        if h < lo || h > hi {
            let msg = format!("{name}: hit registry size {h} outside baseline {bh} ±10 %");
            if release {
                panic!("{msg}");
            }
            println!("PERF-DEBUG-MISMATCH {msg}");
        }
    }
    if s.ns > base.ns + base.ns / 5 {
        if env_flag("PERF_STRICT") && release {
            panic!(
                "{name}: time regression: ns {} > baseline {} × 1.2",
                s.ns, base.ns
            );
        }
        println!("REGRESSION? {name} ns={} baseline={}", s.ns, base.ns);
    }
}

/// Ratio helper for the "within N×" acceptance thresholds: prints the ratio
/// and asserts it only when `strict` is set.
pub fn check_ratio(name: &str, a: u128, b: u128, max: f64, strict: bool) {
    let ratio = a as f64 / b.max(1) as f64;
    println!("PERF-RATIO {name} ratio={ratio:.2} max={max}");
    if strict {
        assert!(ratio <= max, "{name}: ratio {ratio:.2} exceeds {max}");
    }
}

// ------------------------------------------------------------ fixtures

/// One line of `n` graphemes mixing ASCII, CJK (width 2), combining marks and
/// an emoji ZWJ sequence (§7.2 E).
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
