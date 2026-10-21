# Chooser activation independent review

Review scope: exact commit `9c07189c8f38a4e29bda4060f2dfd5efc8ceda73`, parent `8e7480415a065eac03c0fdfbdd0810edeb85b186`. Only the shared Form/FormDialog delta is reviewed. No actual Showcase flow or product parity approval is implied.

The two Form update paths exclude Chooser from the idle Enter submission shortcut. Button remains the input owner and maps activation to FormAction::Chose. FormDialog skips its chrome update only for that child action. Response::action is consumed; combining preserves the child's response. No rendering or measurement path dispatches input or effects. No new unsafe code, concurrency, ownership, or fallible-I/O path is introduced; public signatures are unchanged.

An external public-API consumer copies the new chooser regression test without changing its assertions and supplements it with disabled pointer, exactly-once activation across repeated draws, following Escape cancellation, and ordinary TextInput editing behavior. Parent and candidate use identical probe source. The new chooser Enter assertion fails against the exact parent because Enter submits the dialog; candidate passes. Existing native Form tests cover selection/dropdown, custom layout, validation, disabled controls, and editing Enter behavior.

Preliminary observer corrections are retained in logs: a missing Part import prevented the first consumer compile. A probe initially mistook an editing TextInput for idle and expected submit; both parent and candidate instead committed the text. The separately corrected observer asserts that existing committed behavior. Another preliminary assertion overlooked a normal text commit on focus transfer. No product test assertion or expected snapshot was changed.

There is no independent read-only flag in FormData or Button for a chooser. FieldMut::ReadOnly is intentionally the data contract for an enabled chooser, not a disabled state. Disabled pointer behavior is tested through FormData::disabled. This review does not certify an additional, nonexistent read-only chooser policy or a complete nested Picker/application flow.

The first unique /private/tmp build target lost files during a consumer link (ENOENT). Cause unknown; no cleanup performed by this reviewer. The final checks use a fresh unique target under the project evidence directory; prior failed logs are retained, not interpreted as product defects.
