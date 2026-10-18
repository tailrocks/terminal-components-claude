//! Rich decorative rows keep the Props allocation-free paint contract.
use junie_tui::{ColorLevel, ItemKey, Props, PropsRow, Role, Theme};
use junie_tui_testing::{
    Scene,
    perf::{Counting, lock, measure_once},
};
#[global_allocator]
static GLOBAL: Counting = Counting;

#[test]
fn wrapped_and_toned_rows_allocate_nothing_after_warmup() {
    let _guard = lock();
    let rows: Vec<_> = (0..24)
        .map(|key| {
            PropsRow::new(
                ItemKey::num(key),
                "Accounts",
                "Claude · Personal ★ · Claude · Work · Codex · Primary ★\nnext line",
            )
            .tone(if key % 2 == 0 {
                Role::Danger
            } else {
                Role::Success
            })
            .wrap()
        })
        .collect();
    let props = Props::rich(&rows);
    let mut scene = Scene::new(
        "props_rich_perf",
        Theme::junie(),
        ColorLevel::TrueColor,
        60,
        80,
    );
    let mut bound = scene.bind_model(&props, |props, ui, area| {
        props.draw(ui, area);
    });
    for _ in 0..4 {
        bound.draw();
    }
    let stats = measure_once(&mut || {
        for _ in 0..200 {
            bound.draw();
        }
    });
    assert_eq!(
        stats.allocs, 0,
        "full repeated rich Props captures allocate"
    );
    assert_eq!(stats.bytes, 0);
}
