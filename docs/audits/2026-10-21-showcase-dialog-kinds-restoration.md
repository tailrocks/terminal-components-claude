# Showcase DialogKinds restoration: incomplete chooser checkpoint

Status: **INCOMPLETE**. This WIP restores the frozen DialogKinds route, its nine
live form fields, conditional SSH chooser, nested picker, actions, and status
path. It is not an acceptance or parity claim.

References remain fixed at PR #26 `f4867e9b5f8a08a3bb61ce80d644198a29f51984`,
baseline commit `4169783ae883788186d161af09ba9c481bd84e5c`, and the release tag
object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`.

The route's Name field now uses public `TextInput::focus_select(SelectAll)`;
the default caret policy remains unchanged. “Test connection” and “Save” use
public `Action::with_variant(Secondary/Primary)`. The application no longer
selects text by calling FormState directly.

## Exact live-app failure

Command:

```text
cargo nextest run -p showcase-ui --test app_tests -E 'test(dialog_kinds_add_connection_uses_live_form_and_stacked_picker)' --no-fail-fast
```

Result: **0 passed, 1 failed**. The live Showcase app regression records the
actual chooser geometry and input sequence. At 120×40, the frozen scenario's
pointer point is `(33, 27)`. `area_of(KEY)` is `Rect { x: 32, y: 27, width: 9,
height: 1 }` before Down; after Down it is `Rect { x: 32, y: 26, width: 9,
height: 1 }` and remains there through Tick and Up. Focus is the chooser. The
Down/Up responses are owned by the shell status control, and the picker does
not open. This is a live-app RED. No Enter substitution, input replay,
application geometry adjustment, invisible control, or painting workaround
was added.

Focused DialogKinds suite before this diagnostic checkpoint: 1 passed / 2
failed (Save action passed; both chooser flows failed). The shared Form
pointer-reveal fix b40 was independently reviewed and integrated, with
ordinary and lane FormDialog tests passing; this app-level case remains
unresolved and needs a bounded shared-layout/input investigation.

## Resume

Investigate why the real Showcase composition changes chooser hit geometry on
Down even though the reviewed Form pointer-hold tests keep it stable. Preserve
this exact trace as the regression. Only change a shared component after the
root cause and ownership contract are established. Then rerun all three
DialogKinds tests, the Showcase coverage gate, and relevant formatting and
Clippy checks before the next WIP checkpoint. Do not integrate this app WIP
into PR #38 while the chooser regression is red.

## Layer and form-state trace (diagnostic only)

The failure was repeated with `DIALOG_KINDS_TRACE=1` in a temporary,
environment-gated diagnostic in the Showcase page. That instrumentation was
removed after capture. The full unmodified test log is preserved outside the
repository at
`/Users/donbeave/Projects/termrock-cli-evidence-20261021/showcase-dialog-kinds-trace-260a.log`
(SHA-256 `fde6f703902285bedabee687d0c19abaf23fa2c584ba7d8f41db76c5bdabbd72`).

The trace distinguishes the outer layer from the inner scroll hold:

- Before chooser Down, the layer is `Rect { x: 29, y: 2, width: 62,
  height: 33 }`. Its content has grown from 26 to 29 rows after toggling the
  tunnel; the scroll offset remains 0.
- On Down, `FormState.pointer_hold` becomes the chooser field ID. In the
  immediately following app update, the modal remains `Rect { x: 32, y: 6,
  width: 56, height: 27 }` and the chooser remains at y=27. The form has
  content length 29 and viewport length 26; pending form reveal is `None`.
- After that update is published, before Tick, the layer is now `Rect { x: 32,
  y: 5, width: 56, height: 30 }`, body viewport/content are both 29 rows, and
  the chooser is y=26. Pointer hold remains active and inner scroll offset is
  still 0. Tick and Up keep this expanded geometry; Up correctly does not emit
  `Click` because it is no longer over the pressed target.

This evidence points to outer FormDialog layer remeasurement/reanchoring when
the previously hidden conditional field becomes visible, rather than the
inner Form scroll reveal consuming a pending request. That is a source-based
hypothesis, not a confirmed fix. The independently reviewed b40 native test
does not invalidate its Down frame before measuring; it starts with the
conditional field already visible and therefore does not cover this dynamic
layer-size transition. No shared source was changed based on this hypothesis.
