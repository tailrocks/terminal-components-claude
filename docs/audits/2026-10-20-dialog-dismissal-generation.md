# Dialog dismissal receipts are scoped to one opening and update callback

The previous `DialogState::action_close_pending: bool` suppressed every queued
programmatic dismissal after an action. A real `Harness` reproduction showed
that Confirm could close a dialog, reopen the same ID, and independently close
the new opening before the next update; the expected Cancel action was lost.
The regression failed on `1be6567aeaa7bfc80c81ee1f2f10eb39f66248d9` and passes
after this change.

The runtime now attaches each layer event to its exact `LayerOpening`. A
programmatic close also carries an opaque `UpdateToken` identifying the
runtime and one `Cx` application callback. Dialog suppresses only the close
receipt whose opening and callback token match the action it just emitted.
Esc/outside dismissals remain runtime-originated and carry no callback token.
Existing `LayerEvent`, `Intent::Layer`, and `Cx::layer_event` remain supported
as event projections.

Independent review found a second edge in the first receipt draft: comparing
`None == None` could suppress an ancestor-cascade dismissal when a dialog had
no action receipt. Suppression now requires a positive exact `(opening, token)`
pair. The real Harness case closes an ancestor around an acknowledgement
dialog and checks that the child resolves Cancel and clears its secret. Runtime
tests also assert callback-token uniqueness across update, Esc callback, and
separate runtime instances, plus receipt identity across reopened generations.
The ancestor-cascade assertion was deliberately run with the weaker `None ==
None` predicate and failed as intended (`left: []`, expected one Cancel action),
then passed after suppression required a positive exact action receipt.

Focused Harness coverage exercises a normal action close, close/reopen without
closing the new opening, close/reopen/close in the same callback, and a later
independent programmatic close of the same opening. The focused regression
passed on the changed source; the runtime library suite passed 197 tests.
Broader application validation is outside this focused fix. Independent
external review of the final published source is pending. This audit does not
claim application visual parity or full repository acceptance.
