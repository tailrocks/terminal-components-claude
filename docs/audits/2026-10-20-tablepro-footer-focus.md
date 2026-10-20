# TablePro footer focus projection

This change narrows the editor-specific footer hints to the state in which the
query editor actually owns keyboard focus. It addresses the observed
QuickOpen/tab-list Escape dismissal mismatch where the editor footer remained
visible after focus returned to Explorer. The component and navigation state
paths are unchanged.

## Source and change

- Frozen reference: `visual-baseline` source
  `4169783ae883788186d161af09ba9c481bd84e5c`, tag object
  `2718a6b8127a522a3d351d0b4bdb7542fc8da542`.
- Candidate base: `72c2d3658f3d89f58b3f4690f9dbdce728be8cef`.
- Candidate edit: `crates/tablepro-ui/src/app.rs`, `draw_footer` requires
  `query_id()` to be focused before choosing the editor footer projection.
- Source diagnosis: `historical_empty_query` only checked query contents, so an
  empty query selected editor hints after the picker closed even when Explorer
  owned focus. This is application footer selection, not runtime focus
  restoration.

## Verification status

- `cargo check --locked -p tablepro-ui --lib`: passed.
- `git diff --check`: passed.
- `cargo fmt --all -- --check`: fails on pre-existing formatting differences
  across unrelated workspace files; the focused changed hunk is rustfmt
  formatted. No workspace-wide formatting changes were applied.
- Paired CLI parity is **blocked, not passed**. The frozen runner rejected the
  rebuilt baseline before capture because the executable hashes differ from
  the immutable reference decision. Building from the frozen source at the
  evidence root produced `showcase=9dca33802b77a081d2775e0a339539e05eba52c918d12d317c2e9a3f9ff2fd24`,
  `tablepro=e6de446bd497073eff1777280a64294f44a9e4d1629646fc9c984039b539c888`,
  `jackin-preview=b1a4ee3297691a286aec7122e1a2ba4fec3392a06fb18db490aba35f9cd95546`,
  and `holla=192e36373947acccb0c70ea6bd66cf64a4eabb09f23889d1964b1b0e0b2595b2`.
  Rebuilding manually from the preserved original source path into the original
  evidence `cargo-target` path also did not reproduce the approved hashes.
  The runner reported `reference decision baseline binaries are not the
  reviewed frozen application build`; no expected snapshot or reference
  decision was changed. The paired five-case command therefore has no capture,
  replay, or candidate-comparison result.

Evidence root: `/Users/donbeave/Projects/termrock-cli-evidence-20261020/tablepro-picker-footer-72c2`.
The pre-fix paired evidence remains under
`/Users/donbeave/Projects/termrock-cli-evidence-20261020/tablepro-picker-1d7`.
Exact paired verification must be rerun after recovering the approved frozen
binary build; this source change is not parity-accepted until that succeeds.
