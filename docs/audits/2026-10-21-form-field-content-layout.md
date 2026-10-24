# Form content sizing policy

## Decision

`Form` now has an opt-in `FieldSizingPolicy::Intrinsic` through
`Form::field_sizing`. The default remains `FieldSizingPolicy::Fixed`, preserving
existing callers and `FieldSpec` struct-literal compatibility.

In intrinsic mode, `Note` placement height comes from the number of live note
rows. Drawing keeps each row's supplied semantic `Role`, and clipped rendering
starts at the matching source row. `Chooser` height follows its displayed value
and optional detail. The value occupies the row before the trailing shared
`Button`; only the button's resolved area registers the chooser hit target.

The same policy feeds ordinary placement, explicit lane placement, measurement,
update, and drawing. Other field types retain their current sizing rules. SQL,
validation, and domain actions remain application-owned.

## Frozen source comparison

The frozen source `src/widgets/form_dialog.rs` at
`4169783ae883788186d161af09ba9c481bd84e5c` renders chooser value before the
button (lines 873–915) and uses each note row's tone. The fixed design source at
`f4867e9b5f8a08a3bb61ce80d644198a29f51984` was reviewed as byte-identical by the
FormDialog architecture audit. This shared policy implements those field
contracts without restoring the application form implementation.

## Evidence

Focused actual-widget checks cover zero, one, and five note rows, following-field
placement, per-row semantic tones, partial note scrolling, chooser value/button
ordering, Unicode detail, narrow widths, and button-only activation. The public
`form_intrinsic_content` example compiles through the public crate API.

This increment does not migrate an application FormDialog and does not prove
application parity. App adoption remains pending the reusable FormDialog
composition work and paired reference checks.
