//! Independently frozen synthetic production workload and measurement instrument.
use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

struct Instrument;
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static STYLE_CALLS: AtomicU64 = AtomicU64::new(0);
static DRAW_CALLS: AtomicU64 = AtomicU64::new(0);
static WORK: AtomicU64 = AtomicU64::new(0);
// SAFETY: This wrapper preserves System's pointer/layout contract and performs
// only allocation-free atomic increments before forwarding each allocation.
unsafe impl GlobalAlloc for Instrument {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: GlobalAlloc supplies a valid layout; System owns allocation.
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: The caller provides the original System pointer and layout.
        unsafe { System.dealloc(pointer, layout) }
    }
}
#[global_allocator]
static INSTRUMENT: Instrument = Instrument;

fn work(rounds: u64) {
    let mut value = black_box(19_u64);
    for index in 0..rounds {
        value = black_box(value.wrapping_mul(1664525).wrapping_add(index));
    }
    WORK.fetch_add(rounds, Ordering::Relaxed);
    black_box(value);
}

fn resolve_style(rounds: u64) {
    STYLE_CALLS.fetch_add(1, Ordering::Relaxed);
    work(rounds);
}

fn draw_frame(heavy_style: bool, extra_allocation: bool) -> u128 {
    DRAW_CALLS.fetch_add(1, Ordering::Relaxed);
    let allocation = vec![black_box(7_u8); black_box(128)];
    black_box(&allocation);
    if extra_allocation {
        black_box(vec![black_box(9_u8); black_box(256)]);
    }
    let style_start = Instant::now();
    resolve_style(if heavy_style { 2_000_000 } else { 200 });
    let style_ns = style_start.elapsed().as_nanos();
    work(if heavy_style { 200 } else { 2_000_000 });
    style_ns
}

fn main() {
    let mode = std::env::args().nth(1).expect("fixed workload mode");
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    let start = Instant::now();
    let mut style_ns = 0_u128;
    if mode != "smoke" && mode != "noop" {
        for _ in 0..8 {
            style_ns += draw_frame(mode == "heavy-style", mode == "allocation");
        }
    } else if mode == "smoke" {
        black_box(0x51a7_u64);
    }
    let total_ns = start.elapsed().as_nanos();
    let allocations = ALLOCATIONS.load(Ordering::Relaxed) - before;
    let draws = DRAW_CALLS.load(Ordering::Relaxed);
    let calls = STYLE_CALLS.load(Ordering::Relaxed);
    let work = WORK.load(Ordering::Relaxed);
    println!(
        "{{\"allocations\":{allocations},\"draws\":{draws},\"style_calls\":{calls},\"work\":{work},\"style_ns\":{style_ns},\"frame_ns\":{total_ns}}}"
    );
}
