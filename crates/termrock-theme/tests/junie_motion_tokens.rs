use termrock_theme::Theme;

#[test]
fn junie_default_spinner_frames_match_reference_cycle() {
    let expected = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    assert_eq!(Theme::junie().design.motion.spinner_frames, &expected);
}
