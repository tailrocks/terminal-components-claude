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
) {
    let case = Case::new(name, bin, args, 80, 24, Color::Truecolor, needle);
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
    );
}

pub fn tablepro_boot_sets_terminal_modes() {
    assert_boot_modes(
        "modes/tablepro/boot/80x24/truecolor",
        Bin::Tablepro.path(),
        &[],
        "Production",
    );
}

pub fn holla_boot_sets_terminal_modes() {
    assert_boot_modes(
        "modes/holla/boot/80x24/truecolor",
        Bin::Holla.path(),
        &[],
        "holla❯",
    );
}

pub fn jackin_boot_sets_terminal_modes() {
    assert_boot_modes(
        "modes/jackin/boot/80x24/truecolor",
        Bin::Jackin.path(),
        &[],
        "jackin❯",
    );
}
