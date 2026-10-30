# Form lanes, stage 1

This increment adds borrowed field-lane composition to `termrock-forms`. The
consumer provides stable field and lane IDs plus one shared split policy;
`Form` retains the field state, controls, scroll behavior, and real action
buttons. The existing linear form layout remains the default.

The public notification-preferences consumer in
`crates/termrock-forms/tests/form_lanes_consumer.rs` exercises two lanes,
70/30 paired fields, live radio option counts, conditional visibility,
field-backed fill, narrow/zero-width bounds, and malformed-layout rejection.
The runnable consumer example is `crates/termrock-forms/examples/form_lanes.rs`.
The fallible `try_layout` builder validates static references and rejects
unknown/repeated fields, duplicate lanes, unsupported lane counts, invalid
split axes/sizing, and malformed fill rows. `validate_layout(data)` then checks
the final selected group and currently visible fields. Update, draw, and
data-aware measurement revalidate before changing/registering anything, so a
group or visibility change cannot silently activate an omitted field.

The implementation uses `SplitModel::layout` for both sibling lanes and
paired rows. `measure_with_data`, update-time focus reveal, lane scrolling,
field drawing and control registration derive their geometry from the same
lane plan. Measurement and drawing do not validate or mutate domain data. The
pre-existing `measure` API remains the documented data-independent estimate;
the default form planner is unchanged. Form lanes record their real owner
rectangle as a decorative region and logical layout area so the next update
uses the last composed bounds, including after a resize.

This stage does not add sections, Tabs, section lifecycle handling, or
cross-section validation. The TablePro application still has not migrated to
this public Form layout. Exact application parity therefore remains unproven.

Verification before the latest lane corrections: `cargo nextest run
-p termrock-forms --all-targets` passed 57/57;
`cargo clippy -p termrock-forms --all-targets -- -D warnings`,
`cargo check -p termrock-forms --example form_lanes`, package-scoped formatting,
and `git diff --check` pass. A deliberate fixed-radio-height mutation caused
`live_radio_options_match_measurement_to_rendered_lane_extent` to fail
(measured extent 6 versus the actual rendered extent 8); restoring the
data-aware planner made that test pass again.

Invalid layout aborts draw/update without partial registration and returns the
empty constrained measure fallback; callers can inspect the typed error
through `validate_layout`. Draw and update append the bounded
`InvalidComponentLayout` diagnostic, with `FormLayoutError::field_id`
identifying fields when applicable. Measurement emits no diagnostic. Tests
assert the invalid update leaves existing FormState untouched and invokes no
domain validation, while invalid drawing registers no included control.

Independent review of `451789d` requested changes. Two findings are resolved
in the current working increment: Fill now reserves fixed trailing spacer rows
before taking remaining height, and the lane Form owner registers its actual
drawn area for the next update. Public regressions for both were verified red
under deliberate removal of their fixes, then green when restored. The Form
area is exposed only through a decorative CONTAINER region and the test checks
that no focusable Form control is registered.

The reviewed RadioGroup-owned window renderer is now implemented. It accepts
the complete item slice and a first visible row, paints and registers only the
visible rows, and keeps original item indexes as keys. Form clips field chrome
and Radio body rows using the original field source positions in both linear
and lane layouts. A real buffer/registry regression checks a ten-option field
at scroll offset 18: options 8 and 9 occupy rows 0 and 1, help occupies row 2,
and the next field begins at row 3. A separate public Controls consumer proves
only original keys 8 and 9 are registered and pointer activation chooses item
8. Focused radio updates now reveal the current original cursor row.

TextArea's partial viewport and cursor mapping is still unresolved; this Radio
contract does not imply all field kinds are correct when partially clipped.
The legacy linear Form had the same saturation issue and now uses this
Radio-specific source-window path. The default layout and fully visible
RadioGroup draw behavior retain their existing geometry. Exact application
parity remains unproven.
