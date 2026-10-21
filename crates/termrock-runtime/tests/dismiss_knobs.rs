//! Port of the PR #26 reference popup `Dismiss` knob tests
//! (`src/ui/popup.rs` at f4867e9b5f8a08a3bb61ce80d644198a29f51984) onto the
//! parity layer `Dismiss` type. The parity type keeps its own constant names
//! (`NONE`/`ESC`/`ESC_AND_OUTSIDE`/`ALL`); the knobs (`pass_through`,
//! `close_button`, `refuse_feedback`, `allows`, `shows_close`) are the
//! reference vocabulary with defaults that preserve current behavior.

use termrock_runtime::layer::{Dismiss, DismissReason, RefuseFeedback};

#[test]
fn the_default_dismiss_refuses_everything_and_stays_silent_about_it() {
    let d = Dismiss::default();
    assert!(!d.esc);
    assert!(!d.outside_click);
    assert!(!d.focus_out);
    assert!(!d.pass_through, "the runtime keeps trapping input today");
    assert!(!d.close_button);
    assert_eq!(d.refuse_feedback, RefuseFeedback::Flash);
}

#[test]
fn allows_maps_each_reason_to_its_knob() {
    for (dismiss, esc, outside, focus_out) in [
        (Dismiss::NONE, false, false, false),
        (Dismiss::ESC, true, false, false),
        (Dismiss::ESC_AND_OUTSIDE, true, true, false),
        (Dismiss::ALL, true, true, true),
    ] {
        assert_eq!(dismiss.allows(DismissReason::Esc), esc, "{dismiss:?}");
        assert_eq!(
            dismiss.allows(DismissReason::OutsideClick),
            outside,
            "{dismiss:?}"
        );
        assert_eq!(
            dismiss.allows(DismissReason::FocusOut),
            focus_out,
            "{dismiss:?}"
        );
        assert!(
            dismiss.allows(DismissReason::Programmatic),
            "{dismiss:?}: an owner always closes its own layer"
        );
    }
}

#[test]
fn a_close_affair_only_shows_when_escape_really_dismisses() {
    for (dismiss, shows) in [
        (Dismiss::ESC, true),
        (Dismiss::ESC.close_button(true), true),
        (Dismiss::NONE.close_button(true), false),
        (Dismiss::ESC.close_button(false), false),
    ] {
        assert_eq!(
            dismiss.shows_close(),
            shows,
            "{dismiss:?}: a close affordance without Esc is a lie"
        );
    }
}

#[test]
fn the_knobs_chain_const_and_preserve_the_dismiss_paths() {
    for pass_through in [true, false] {
        let custom = Dismiss::ESC_AND_OUTSIDE
            .pass_through(pass_through)
            .close_button(false)
            .refuse_feedback(RefuseFeedback::None);
        assert_eq!(custom.pass_through, pass_through);
        assert!(!custom.close_button);
        assert_eq!(custom.refuse_feedback, RefuseFeedback::None);
        assert!(custom.allows(DismissReason::Esc));
        assert!(custom.allows(DismissReason::OutsideClick));
        assert!(!custom.allows(DismissReason::FocusOut));
    }
}
