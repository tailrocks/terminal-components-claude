//! Scroll performance of the TablePro table surfaces, driven through the
//! real `App` on a `TestBackend` (see docs/audits/design-review-2026-10/
//! performance.md).
//!
//! `measure_table_surfaces` is an ignored measurement: it prints per-event
//! handling and per-frame render cost for each scenario. Run it in the
//! release profile for representative numbers:
//!
//! `cargo nextest run --cargo-profile release --bin tablepro --run-ignored only -E 'test(measure_table_surfaces)' --no-capture`

use std::time::{Duration, Instant};

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};

use junie_tui::core::event::{Input, Key, Mouse, MouseKind};

use crate::app_tests::H;
use crate::workbench::WorkTab;

/// Open the customers table (48,912 rows, 500 loaded, 11 columns) and
/// focus its grid.
pub fn customers(w: u16, h: u16) -> H {
    let mut h = H::connected(w, h);
    h.ctrl('o');
    h.type_str("customers");
    h.key(KeyCode::Enter);
    let id = grid_of(&h).0;
    h.app.focus.set(Some(id));
    h.draw();
    h
}

/// A query tab whose result grid holds `SELECT * FROM customers`.
pub fn query_results(w: u16, h: u16) -> H {
    let mut h = H::connected(w, h);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Char('i'));
    h.type_str("SELECT * FROM customers");
    h.key(KeyCode::Esc);
    h.ctrl('r');
    h.ticks(12);
    let id = grid_of(&h).0;
    h.app.focus.set(Some(id));
    h.draw();
    h
}

/// The active surface's grid id and area as last drawn.
pub fn grid_of(h: &H) -> (junie_tui::core::id::WidgetId, Rect) {
    match h.wb().active_tab() {
        Some(WorkTab::Table(t)) => (t.grid.id, t.grid.area),
        Some(WorkTab::Query(q)) => {
            let g = q.active_grid().expect("query has a result grid");
            (g.id, g.area)
        }
        _ => panic!("no grid surface"),
    }
}

fn wheel(kind: MouseKind, pos: Position) -> Input {
    Input::Mouse(Mouse::new(kind, pos))
}

fn key(code: KeyCode) -> Input {
    Input::Key(Key {
        code,
        mods: KeyModifiers::NONE,
    })
}

/// Handle and draw cost of a scenario, per event.
struct Cost {
    events: usize,
    frames: usize,
    handle: Duration,
    draw: Duration,
}

impl Cost {
    fn per_event_us(&self) -> f64 {
        self.handle.as_secs_f64() * 1e6 / self.events.max(1) as f64
    }
    fn per_frame_us(&self) -> f64 {
        self.draw.as_secs_f64() * 1e6 / self.frames.max(1) as f64
    }
    fn total_ms(&self) -> f64 {
        (self.handle + self.draw).as_secs_f64() * 1e3
    }
}

/// Feed `inputs` the way the runtime does: every input is handled, and a
/// frame is drawn after each one (what the event loop did before input
/// coalescing) or once per `batch` inputs.
fn run(h: &mut H, inputs: &[Input], batch: usize) -> Cost {
    let mut cost = Cost {
        events: 0,
        frames: 0,
        handle: Duration::ZERO,
        draw: Duration::ZERO,
    };
    for chunk in inputs.chunks(batch.max(1)) {
        for input in chunk {
            let t = Instant::now();
            h.app.handle(input.clone());
            cost.handle += t.elapsed();
            cost.events += 1;
        }
        let t = Instant::now();
        h.draw();
        cost.draw += t.elapsed();
        cost.frames += 1;
    }
    cost
}

fn scenarios(h: &H) -> Vec<(&'static str, Vec<Input>)> {
    let (_, area) = grid_of(h);
    let mid = Position::new(area.x + area.width / 2, area.y + area.height / 2);
    let mut out = vec![
        (
            "wheel burst ×60",
            vec![wheel(MouseKind::WheelDown, mid); 60],
        ),
        ("PageDown ×20", vec![key(KeyCode::PageDown); 20]),
        ("End/Home ×10", {
            (0..10)
                .map(|i| {
                    key(if i % 2 == 0 {
                        KeyCode::End
                    } else {
                        KeyCode::Home
                    })
                })
                .collect()
        }),
        ("key repeat Down ×120", vec![key(KeyCode::Down); 120]),
        ("hscroll wheel ×20", {
            (0..20)
                .map(|i| {
                    wheel(
                        if i < 10 {
                            MouseKind::WheelRight
                        } else {
                            MouseKind::WheelLeft
                        },
                        mid,
                    )
                })
                .collect()
        }),
    ];
    let hover: Vec<Input> = (0..area.height.saturating_sub(2))
        .map(|dy| {
            wheel(
                MouseKind::Move,
                Position::new(area.x + area.width / 3, area.y + 1 + dy),
            )
        })
        .collect();
    out.push(("hover moves over rows", hover));
    out
}

fn report(label: &str, h: &mut H) {
    let (w, ht) = {
        let a = h.term.backend().buffer().area;
        (a.width, a.height)
    };
    for (name, inputs) in scenarios(h) {
        let per_event = run(h, &inputs, 1);
        // the runtime's own pump with its default policy
        let t = Instant::now();
        let frames = pump(h, junie_tui::runtime::Coalesce::default(), &inputs);
        let runtime = t.elapsed();
        println!(
            "{label:<14} {w}x{ht:<3} {name:<22} handle {:>8.1} µs/event  draw {:>8.1} µs/frame  per-event loop {:>7.2} ms ({} frames)  runtime pump {:>6.2} ms ({frames} frames)",
            per_event.per_event_us(),
            per_event.per_frame_us(),
            per_event.total_ms(),
            per_event.frames,
            runtime.as_secs_f64() * 1e3,
        );
    }
    // resize between the two measured sizes and back
    let t = Instant::now();
    let n = 10;
    for i in 0..n {
        if i % 2 == 0 {
            h.resize(w + 13, ht + 7);
        } else {
            h.resize(w, ht);
        }
    }
    println!(
        "{label:<14} {w}x{ht:<3} {:<22} {:>8.1} µs/resize (handle + draw)",
        "resize ×10",
        t.elapsed().as_secs_f64() * 1e6 / n as f64
    );
}

#[test]
#[ignore = "measurement: prints timings, asserts nothing"]
fn measure_table_surfaces() {
    for (w, ht) in [(120, 40), (200, 55)] {
        let mut h = customers(w, ht);
        report("table view", &mut h);
        let mut h = query_results(w, ht);
        report("query results", &mut h);
    }
}

/// Pump `inputs` through the runtime's own `InputPump`, drawing whenever
/// the event loop would. Returns the number of frames drawn.
fn pump(h: &mut H, policy: junie_tui::runtime::Coalesce, inputs: &[Input]) -> usize {
    let mut queue: std::collections::VecDeque<Input> = inputs.iter().cloned().collect();
    let mut pump = junie_tui::runtime::InputPump::new(policy);
    let mut frames = 0;
    while pump.has_pending() || !queue.is_empty() {
        if pump.drain(&mut h.app, || Ok(queue.pop_front())).unwrap() {
            h.draw();
            frames += 1;
            if pump.settle(&mut h.app) {
                h.draw();
                frames += 1;
            }
        }
    }
    frames
}

/// Guard (P1): a burst of wheel notches, key repeats and pointer moves over
/// the customers table draws one frame per burst, and the screen it ends on
/// is cell-for-cell the one per-event drawing ends on.
#[test]
fn input_bursts_over_the_customers_table_draw_once_and_land_identically() {
    use junie_tui::runtime::Coalesce;
    for (w, ht) in [(120, 40), (200, 55)] {
        let mut coalesced = customers(w, ht);
        let mut per_event = customers(w, ht);
        let (_, area) = grid_of(&coalesced);
        let mid = Position::new(area.x + area.width / 2, area.y + area.height / 2);
        let mut inputs = vec![wheel(MouseKind::WheelDown, mid); 60];
        inputs.extend(vec![key(KeyCode::Down); 30]);
        inputs.extend(vec![wheel(MouseKind::WheelRight, mid); 4]);
        inputs.extend(
            (0..area.height - 2)
                .map(|dy| wheel(MouseKind::Move, Position::new(area.x + 20, area.y + 1 + dy))),
        );
        let policy = Coalesce {
            max_latency: Duration::MAX,
            ..Coalesce::default()
        };
        let frames = pump(&mut coalesced, policy, &inputs);
        let baseline = pump(&mut per_event, Coalesce::OFF, &inputs);
        // wheel (one batch of 60 ≤ 64), Down ×30, wheel-right ×4, sweep
        assert_eq!(frames, 4, "{w}x{ht}: one frame per burst");
        assert!(baseline > 90, "{w}x{ht}: per-event drawing drew {baseline}");
        if coalesced.text() != per_event.text() {
            for (y, (a, b)) in coalesced
                .text()
                .lines()
                .zip(per_event.text().lines())
                .enumerate()
            {
                if a != b {
                    eprintln!("{y}\n C {a}\n P {b}");
                }
            }
        }
        assert_eq!(
            coalesced.term.backend().buffer(),
            per_event.term.backend().buffer(),
            "{w}x{ht}: coalescing must not change what is drawn"
        );
    }
}
