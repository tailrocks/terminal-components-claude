//! Terminal modes are not part of any snapshot format (`Frame` has no mode
//! state), so each app gets one boot check on the live emulator state: the
//! shared runtime (`src/runtime.rs`) must enter the alternate screen (1049),
//! enable any-motion SGR mouse reporting (1003 + 1006) and bracketed paste
//! (2004). A regression here would otherwise surface only indirectly, as
//! changed hover/click captures or refused pointer input.

use crate::support::{self, Case, Color, HOLLA, JACKIN, SHOWCASE, TABLEPRO};

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

#[test]
fn showcase_boot_sets_terminal_modes() {
    assert_boot_modes(
        "modes/showcase/boot/80x24/truecolor",
        SHOWCASE,
        &[],
        "Junie Design system",
    );
}

#[test]
fn tablepro_boot_sets_terminal_modes() {
    assert_boot_modes(
        "modes/tablepro/boot/80x24/truecolor",
        TABLEPRO,
        &[],
        "Production",
    );
}

#[test]
fn holla_boot_sets_terminal_modes() {
    assert_boot_modes("modes/holla/boot/80x24/truecolor", HOLLA, &[], "holla❯");
}

#[test]
fn jackin_boot_sets_terminal_modes() {
    assert_boot_modes("modes/jackin/boot/80x24/truecolor", JACKIN, &[], "jackin❯");
}
