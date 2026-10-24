use std::time::Duration;

use crate::scenarios::Bin;
use crate::support::{self, Case, Color};

fn typed_before_start_is_discarded(
    name: &'static str,
    bin: &'static str,
    needle: &'static str,
    query_duration_ms: Option<u64>,
) {
    let mut case = Case::new(name, bin, &[], 100, 30, Color::Truecolor, needle);
    if let Some(duration_ms) = query_duration_ms {
        case = case.with_query_duration_ms(duration_ms);
    }
    let mut s = support::spawn_with_env(&case, &[("JUNIE_TUI_START_DELAY_MS", "800")]);

    s.type_text("q").expect("type");
    support::boot(&mut s, case.needle);

    std::thread::sleep(Duration::from_millis(600));
    assert!(
        s.find(needle).is_ok(),
        "`{name}`: a key typed before startup reached the app (it quit)"
    );
}

pub fn showcase_discards_keys_typed_before_start() {
    typed_before_start_is_discarded(
        "startup_input/showcase/q/100x30/truecolor",
        Bin::Showcase.path(),
        "Junie Design system",
        None,
    );
}

pub fn tablepro_discards_keys_typed_before_start() {
    typed_before_start_is_discarded(
        "startup_input/tablepro/q/100x30/truecolor",
        Bin::Tablepro.path(),
        "Production",
        Some(crate::scenarios::TABLEPRO_QUERY_DURATION_MS),
    );
}
