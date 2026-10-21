# TerminalView input ownership increment

Base under test: `6825b07e86bab75872866c6ed49e6877084db9bb`.

`TerminalView::draw` now registers `Interactive` views as editing owners with
`StateFlags::EDITING`; `ReadOnly` views register read-only focus controls, and
`SelectionOnly` views remain focusable without becoming typing owners. Pointer
press begins selection capture on the registered terminal part, and release
ends it. The existing source cursor now requests a runtime cursor without
repainting its cell with reverse or underline modifiers.

The public `TerminalCursor` contract supplies position and visibility, but no
shape. This increment uses the runtime's existing block cursor request and
does not claim source-selected cursor shape. The cell renderer, selection and
scroll implementation, follow behavior, and viewport reuse remain unresolved.
No Jackin application was changed or accepted by this increment.

Real-runtime tests cover pointer focus/capture, forwarded key and paste input,
source cursor placement without synthetic cell modifiers, and the rule that
read-only and selection-only views cannot own paste typing. The focused test
`focused_interactive_view_forwards_typing_and_exposes_source_cursor` failed
against the unchanged source at `6825b07` because paste was not delivered to
the terminal; it passed after the edit. The first increment passed all 8 `termrock-terminal` library tests. After the
independent drag-review fix below, the current source passes all 9 with
`cargo nextest run -p termrock-terminal --lib --features termrock-runtime/testing`.
The unchanged facade/owner type identity check passed in
`termrock-conformance::control_states::w01_w45_resolve_through_owner_crate_modules`.
An external consumer using only the public `termrock` API also compiled and
passed its Nextest check. Strict package Clippy and workspace formatting passed;
full application parity remains incomplete.

## Independent drag-review follow-up

The independent review at `terminal-view-input-independent-review-177a3d44-20261021/review.json`
(SHA-256 `ff7bab2034ce6dbbda6257e88dc7af415246099ad94a3ab3930a399ed882f6db`)
found that capture causes the runtime to emit `Release`, `DragEnd`, then a
trailing `Click`; handling that click as a new press collapsed a dragged range.
The TerminalView now starts selection only on `Press`. A `Click` creates a
single-cell selection without leaving `selecting` set, and the trailing click
preserves an existing non-degenerate drag range. This uses existing public
state fields and adds no owner state.

The exact unchanged independent public consumer is at
`/private/tmp/terminal-view-regression-consumer-20261021/src/lib.rs`. The
reviewer's pre-fix run is preserved at
`/private/tmp/terminal177-review-20261021/after-consumer.log`; this follow-up
reproduced both failures against pre-fix commit `2f74af500e5ea906b024a1e01569d10022b64103`
in `/private/tmp/terminal-view-drag-before-20261021.log` (8 passed, 2 failed),
then passed all 10 at the candidate in
`/private/tmp/terminal-view-drag-after-20261021.log`. The consumer exercises
selection range, click completion, input forwarding, cursor/modal ownership,
read-only behavior, and exhaustive public-state construction. Visual
application parity remains outside this component increment.

## Captured-link boundary follow-up

A second independent review found that a captured drag's trailing `Click` was
still passed to `link_resolver` before the range guard. The same 13-case public
consumer now covers drag-to-link suppression, a genuine link click exactly
once, and a genuine link click after an existing text selection. A pre-fix run
against `cf9f5cac09d70a96221dc358f165450bd4d7d5fb` failed those three boundary
cases (10 passed, 3 failed); the candidate passes all 13. The exact fixture and
review are preserved under `/private/tmp/terminal-view-link-regression-20261021`
and `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/terminal-view-drag-independent-review-33e4eb75-20261021/review.json`
(SHA-256 `a01383af219c21506e9593493b90577beee6b06f7c09055e6734562fd462be2e`).

The update path now classifies and skips a completed drag before link
resolution. Link presses defer activation to the real Click phase, allowing
captured drags to cancel link activation while preserving one OpenLink action
for a real click. Existing view state stores the press point so a genuine link
click following a previous selection is not mistaken for a trailing drag
click. No application or runtime routing workaround was added.
