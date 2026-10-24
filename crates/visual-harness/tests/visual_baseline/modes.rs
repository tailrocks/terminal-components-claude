use crate::scenarios::Bin;
use crate::support::{self, Case, Color};

const REQUIRED: [(u16, &str); 4] = [
    (1049, "alternate screen"),
    (1003, "any-motion mouse reporting"),
    (1006, "SGR mouse encoding"),
    (2004, "bracketed paste"),
];

fn assert_boot_modes(
    name: &'static str,
    bin: &'static str,
    args: &'static [&'static str],
    needle: &'static str,
    query_duration_ms: Option<u64>,
) {
    let mut case = Case::new(name, bin, args, 80, 24, Color::Truecolor, needle);
    if let Some(duration_ms) = query_duration_ms {
        case = case.with_query_duration_ms(duration_ms);
    }
    let session = support::spawn_boot(&case);
    let modes = session.modes();
    let missing: Vec<String> = REQUIRED
        .iter()
        .filter(|(mode, _)| !modes.contains(mode))
        .map(|(mode, what)| format!("{mode} ({what})"))
        .collect();
    assert!(
        missing.is_empty(),
        "`{name}` booted without {} (live modes: {modes:?})",
        missing.join(", ")
    );
}

pub fn showcase_boot_sets_terminal_modes() {
    assert_boot_modes(
        "modes/showcase/boot/80x24/truecolor",
        Bin::Showcase.path(),
        &[],
        "Junie Design system",
        None,
    );
}

pub fn tablepro_boot_sets_terminal_modes() {
    assert_boot_modes(
        "modes/tablepro/boot/80x24/truecolor",
        Bin::Tablepro.path(),
        &[],
        "Production",
        Some(crate::scenarios::TABLEPRO_QUERY_DURATION_MS),
    );
}

pub fn holla_boot_sets_terminal_modes() {
    assert_boot_modes(
        "modes/holla/boot/80x24/truecolor",
        Bin::Holla.path(),
        &[],
        "holla❯",
        None,
    );
}

pub fn jackin_boot_sets_terminal_modes() {
    assert_boot_modes(
        "modes/jackin/boot/80x24/truecolor",
        Bin::Jackin.path(),
        &[],
        "jackin❯",
        None,
    );
}
