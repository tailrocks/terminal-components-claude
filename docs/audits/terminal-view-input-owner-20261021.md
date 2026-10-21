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
the terminal; it passed after the edit. The current source passed all 8
`termrock-terminal` library tests with
`cargo nextest run -p termrock-terminal --lib --features termrock-runtime/testing`.
The unchanged facade/owner type identity check passed in
`termrock-conformance::control_states::w01_w45_resolve_through_owner_crate_modules`.
An external consumer using only the public `termrock` API also compiled and
passed its Nextest check. Strict package Clippy and workspace formatting passed;
full application parity remains incomplete.
