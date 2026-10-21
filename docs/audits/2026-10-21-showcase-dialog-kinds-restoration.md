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
