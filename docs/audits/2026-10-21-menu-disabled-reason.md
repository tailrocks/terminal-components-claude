# Shared ContextMenu disabled reasons (WIP)

This audit records a bounded shared-widget implementation on PR #38's WIP
branch, based on read-only contract
`menu-disabled-reason-contract-20261021/contract.json` (SHA-256
`95a36ea84f66c7fa5bb5f7c0fb082f8259e10604475e3a4d0b01844d84fd660f`). The
frozen source is `visual-baseline` commit
`4169783ae883788186d161af09ba9c481bd84e5c`; no frozen artifact was modified.

`MenuItem::reason` borrows explanatory text, marks the item disabled, and is
independent of its executable chord and display hint. A later
`disabled(false)` re-enables activation and suppresses the reason. The shared
ContextMenu renders the selected disabled reason as a noninteractive `HELP`
decoration in its bottom border. It does not add a row or affect measurement.
The only theme change adds the muted semantic `MENU/HELP` recipe. Opt-in
`skip_disabled(false)` and `clear_on_leave(true)` model the frozen menu's
disabled-row keyboard and hover lifecycle while existing menus retain their
defaults.

Focused widget tests cover the borrowed reason/chord contract, unchanged
natural width, disabled pointer and keyboard activation, bottom-border
placement, all four color profiles, pointer-to-keyboard priority, clear-on-leave,
and `HELP` style/slot overrides without a hit target. Before the implementation,
the focused public API probes failed to compile because the reason API and
status decoration did not exist; after implementation the focused tests passed.
The pre-existing natural-width assertion remains exact (`16` cells), and passed
against the base and after the change.

**Not accepted as application parity:** Jackin/Capsule adoption, end-to-end CLI
comparison, full frozen flow inventory, resize/reanchor behavior, all-disabled
and zero/narrow Unicode matrix, independent review, full workspace gates, and
PR #38 integration are still outstanding. This WIP is only a shared component
increment. The branch does not claim the full product or library acceptance
gates.
