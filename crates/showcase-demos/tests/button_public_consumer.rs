use termrock::{BusyMarkerPolicy, Button, Id, Variant};

#[test]
fn consumer_can_configure_busy_marker_and_inspect_public_button_parts() {
    let button = Button::new(Id::root("external.consumer.releaser"), "Release")
        .variant(Variant::SECONDARY)
        .checked(true)
        .busy_marker_policy(BusyMarkerPolicy::Replace);

    assert_eq!(button.id(), Id::root("external.consumer.releaser"));
    assert!(Button::PARTS.contains(&termrock::Part::GUTTER));
    assert!(Button::PARTS.contains(&termrock::Part::MARKER));
}
