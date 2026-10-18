//! Planner-owned external probe of the actual production Showcase application.
use junie_tui::{Id, KeyCode, Theme};
use junie_tui_testing::Harness;
use showcase_app::{App, PageId};

fn hexadecimal(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn capture(harness: &Harness<App>, name: &str) {
    let area = *harness.buffer().area();
    println!("ARCHFRAME|{name}|{}|{}|{:?}|{:?}", area.width, area.height,
        harness.focus(), harness.area_of(Id::root("showcase_app::pages::grid::grid.metrics")));
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = harness.cell(x, y);
            println!("ARCHCELL|{name}|{x}|{y}|{}|{:?}|{:?}|{:?}|{}",
                hexadecimal(cell.symbol().as_bytes()), cell.fg, cell.bg, cell.underline_color,
                cell.modifier.bits());
        }
    }
    println!("ARCHTEXT|{name}|{}", hexadecimal(harness.text().as_bytes()));
}

#[test]
fn architecture_bootstrap_production_grid_and_chrome() {
    let mut harness = Harness::new(App::with_page(PageId::Grid), Theme::junie(), 120, 40);
    capture(&harness, "initial");
    let _ = harness.key(KeyCode::Tab);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    capture(&harness, "activated");
}
