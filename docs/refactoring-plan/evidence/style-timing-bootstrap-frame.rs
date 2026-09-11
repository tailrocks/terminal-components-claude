//! Real Showcase Lists full-frame workload; all observation formatting is untimed.
use std::fmt::Write as _;
use std::io::Write as _;
use std::time::Instant;
use junie_tui::{App as AppTrait, Cx, Response, Ui, Theme};
use junie_tui_testing::{Harness, perf};
use showcase_app::{App, PageId};

#[global_allocator]
static GLOBAL: perf::Counting = perf::Counting;

/* PROBE_IMPORT_AND_SUBJECT */

trait Observed: AppTrait { fn semantic(&self) -> String; }
impl Observed for App { fn semantic(&self)->String { self.style_timing_bootstrap_state() } }

fn cells<A: Observed>(h: &Harness<A>) -> String {
    let mut output=String::new();
    for y in 0..40 { for x in 0..120 {
        let c=h.cell(x,y);
        let bytes=c.symbol().as_bytes();
        write!(&mut output,"{x},{y},{}:",bytes.len()).unwrap();
        for b in bytes { write!(&mut output,"{b:02x}").unwrap(); }
        write!(&mut output,"/{:?}/{:?}/{:?}/{:?};",c.fg,c.bg,c.modifier,c.underline_color).unwrap();
    }}
    write!(&mut output,"cursor={:?};focus={:?};hover={:?};app={};clock={:?};effects={:?};capture={:?};layer={:?};quit={}",h.cursor(),h.focus(),h.hover(),h.app().semantic(),h.runtime().now(),h.runtime().records(),h.runtime().capture_owner(),h.runtime().top_layer(),h.runtime().quit_requested()).unwrap();
    output
}

#[test]
fn style_timing_bootstrap() {
    let _serial=perf::lock();
    /* HARNESS_INIT */
    for _ in 0..12 { h.draw(); }
    println!("STYLE_READY");
    std::io::stdout().flush().unwrap();
    let mut expected: Option<String>=None;
    for batch in 0..9 {
        for mode in 0..3 {
            let mut command=String::new();
            assert!(std::io::stdin().read_line(&mut command).unwrap()>0,"protected interleaving command missing");
            assert_eq!(command.trim(),"draw","wrong protected frame command");
            /* MODE_RESET */
            let (hits0,misses0)=h.runtime().style_cache_stats();
            let a0=perf::allocs(); let b0=perf::bytes();
            let start=Instant::now();
            h.draw();
            let denominator=start.elapsed().as_nanos();
            let allocations=perf::allocs()-a0; let bytes=perf::bytes()-b0;
            let (hits1,misses1)=h.runtime().style_cache_stats();
            let cache_hits=hits1-hits0; let cache_misses=misses1-misses0;
            println!("STYLE_FRAME_TIME|{batch}|{mode}|{denominator}");
            /* OBSERVATION */
            /* EFFECTIVE_CLOCK */
            println!("STYLE_CACHE|{batch}|{mode}|{}",h.runtime().style_timing_bootstrap_cache());
            let cell_bytes=cells(&h);
            if let Some(previous)=&expected { assert!(&cell_bytes==previous,"three-mode full cells/cursor/state changed"); }
            else { println!("STYLE_FRAME|{}",cell_bytes); expected=Some(cell_bytes); }
            println!("STYLE_ROW|{batch}|{mode}|{denominator}|{allocations}|{bytes}|{cache_hits}|{cache_misses}|{reported}|{reported_mode}|{cal_source}|{policy}|{witness:?}|{intervals:?}");
            std::io::stdout().flush().unwrap();
        }
    }
    /* EXTRA_BRANCH_PROOFS */
}
