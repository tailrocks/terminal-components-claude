//! WI-JACKIN-PRELUDE-BRAND: prelude brand lockup through stock `Brand`.
//!
//! The prelude branch of `draw_host_menu` renders its dim identity lockup
//! through the stock `Brand` over `"jackin❯"` with a one-part dim patch —
//! the same component the manager arm draws. This test pins that dispatch:
//! row-0 x=1–9 carries ` jackin❯ ` in the frozen dim tone exactly (symbol,
//! fg, bg, non-bold), and the journey reaches the prelude needle (the
//! `Route::Prelude` arm is live, not a stored painter).
use jackin_preview_app::{App, Motion, Scenario};
use termrock::KeyCode;
use termrock_test_support::Harness;

#[test]
fn paused_prelude_brand_renders_through_stock_brand() {
    use termrock::Color;

    const DIM: Color = Color::Rgb(77, 77, 77);
    const OVERLAY: Color = Color::Rgb(39, 39, 42);

    let app = App::for_scenario_at(Scenario::FirstUse, Motion::Paused, 400);
    let mut h = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let _ = h.key(KeyCode::Char('n'));
    for _ in 0..60 {
        let _ = h.tick();
        if h.text().contains("step 1 of 5") {
            break;
        }
    }
    assert!(
        h.text().contains("step 1 of 5"),
        "journey must reach the prelude needle"
    );

    for (i, want) in " jackin❯ ".chars().enumerate() {
        let x = 1 + i as u16;
        let c = h.cell(x, 0);
        assert_eq!(c.symbol(), want.to_string(), "brand symbol x={x}");
        assert_eq!(c.fg, DIM, "brand fg x={x}");
        assert_eq!(c.bg, OVERLAY, "brand bg x={x}");
        assert!(
            !c.modifier.contains(termrock::Modifier::BOLD),
            "brand must be non-bold x={x}"
        );
    }
}
