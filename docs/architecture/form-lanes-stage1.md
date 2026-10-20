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
field drawing, clipping, and control registration derive their geometry from
the same lane plan. Measurement and drawing do not validate or mutate domain
data. The pre-existing `measure` API remains the documented data-independent
estimate; the default form planner is unchanged.

This stage does not add sections, Tabs, section lifecycle handling, or
cross-section validation. The TablePro application still has not migrated to
this public Form layout. Exact application parity therefore remains unproven.

Verification: `cargo nextest run -p termrock-forms --all-targets` passes 57/57;
`cargo clippy -p termrock-forms --all-targets -- -D warnings`,
`cargo check -p termrock-forms --example form_lanes`, package-scoped formatting,
and `git diff --check` pass. A deliberate fixed-radio-height mutation caused
`live_radio_options_match_measurement_to_rendered_lane_extent` to fail
(measured extent 6 versus the actual rendered extent 8); restoring the
data-aware planner made that test pass again.

Invalid layout aborts draw/update without partial registration and returns the
empty constrained measure fallback; callers can inspect the typed error
through `validate_layout`. `FormLayoutError::field_id` supplies the associated
field for field-specific errors. Runtime diagnostic emission is a separate
prerequisite owned by the runtime author; this packet does not modify runtime
files. Independent review and delivery remain pending.
