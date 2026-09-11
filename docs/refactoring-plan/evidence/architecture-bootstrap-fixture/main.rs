//! Private, planner-owned execution harness. Never executable candidate input.
mod library;
mod app;
use app::App;
use library::{SlotFn, Ui};

fn sentinel(ui: &mut Ui, index: usize) { ui.paint(index, 126); }

fn main() {
    let seed: u32 = std::env::args().nth(1).expect("seed").parse().expect("integer seed");
    print!("{{\"parts\":{:?},\"slots\":{:?},\"registry\":{:?},\"scenes\":[",
        library::PARTS, library::DOCUMENTED_SLOTS, library::REGISTRY);
    let mut first = true;
    for disabled in [false, true] {
        for busy in [false, true] {
            for poison in [false, true] {
                for slot_id in [0, 1, 2] {
                    let mut app = App { value: seed, disabled };
                    let mut ui = Ui::new(poison);
                    app.update(&mut ui);
                    let slot = if slot_id == 0 { None } else { Some((slot_id, sentinel as SlotFn)) };
                    app.draw(busy, slot, &mut ui);
                    if !first { print!(","); }
                    first = false;
                    print!("{{\"disabled\":{},\"busy\":{},\"poison\":{},\"slot\":{},\"value\":{},\"cells\":{:?},\"writes\":{:?},\"resolutions\":{:?},\"draws\":{},\"updates\":{}}}",
                        disabled, busy, poison, slot_id, app.value, ui.cells, ui.writes,
                        ui.resolutions, ui.draws, ui.updates);
                }
            }
        }
    }
    println!("]}}");
}
