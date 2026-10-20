# TablePro Picker consumer migration

Status: scoped implementation is pushed as WIP on `parity/tablepro-picker-live-20261020`; it is not ready for acceptance or PR integration. Exact baseline parity has not been established.

## Fixed references

- Frozen tag object: `2718a6b8127a522a3d351d0b4bdb7542fc8da542`.
- Frozen source commit: `4169783ae883788186d161af09ba9c481bd84e5c`.
- Pinned tuiscotti commit: `a1ea27f757d75d07b05499276cd932e9f8e067ad`.
- Consumer branch base: `2de0d01255390d2a02b7775ee452268782802a38`.

## Scoped implementation

TablePro's quick switcher and open-tabs overlay use the shared `Picker` and
`PickerSlotExt`, with app-owned `SwitchItem`/`TabSummary` data and stable
`ItemKey` identity. The Picker owns filtering, selection, dismissal, and
rendering. The consumer sets baseline-specific width, row cap, geometric
centering, cursor, and footer hints through public configuration. The shared
Picker layer now leaves intrinsic size resolution to the common layer runtime
and requests the same 4-column/2-row viewport margin as the frozen popup.

Choosing an open tab resolves its stable key against current data, activates
it, synchronizes the active model, and focuses the selected tab's primary live
control. Deleting a dirty query follows the application's existing domain
confirmation flow. Query row metadata is derived from the matching execution
history entry rather than the prior generic `complete`/raw-error text. Quick
Switcher scope is shown in the shared muted scope-label slot.

Removed the 16 production `historical_*` modules and the associated app-level
dispatch/ANSI frame painters, including their exact viewport, TTY, color,
fixture, and connection predicates and phantom cursor registration. The normal
screen composition is now used for these states. This removal exposes existing
live-renderer differences; no captured output was retained as a replacement.

## Verification

Commands used the worktree-local isolated target
`/tmp/tablepro-picker-live-target-aa7c-20261020`:

- `cargo nextest run -p termrock-overlays --lib picker --no-fail-fast`: 20
  passed, 48 filtered.
- `cargo nextest run -p tablepro-ui --test quick_switcher_contract --no-fail-fast`:
  13 passed, zero skipped.
- `cargo nextest run -p tablepro-ui --all-targets --no-fail-fast`: 168 passed,
  5 failed, zero skipped. Failures include a live `ORDER BY` completion
  assertion, `tablepro_visual_baseline`, and pre-existing grid allocation and
  row-action failures. The replay-dependent maximized screenshot assertions
  were removed; a domain-level maximization check remains. These failures mean
  product parity is not proven.
- `cargo clippy -p termrock-overlays --all-targets -- -D warnings`: pass.
- `cargo fmt --all --check`: fails on pre-existing formatting in
  `crates/showcase-demos/src/pages/chrome.rs`; both changed packages were
  formatted with `cargo fmt -p tablepro-ui -p termrock-overlays`.
- `git diff --check`: pass.
- `cargo clippy -p tablepro-ui --all-targets -- -D warnings`: pass.
- `cargo clippy -p termrock-overlays --all-targets -- -D warnings`: pass.

No real-binary paired capture/replay using the pinned tuiscotti tool has been
run for this WIP. No claim of 1:1 visual or functional parity is made.

## Remaining blockers and audit limits

- Run the frozen baseline and candidate binaries through the same registered
  tuiscotti workflows and compare all checkpoints, frames, cursor state, and
  interactions. The currently failing live screen tests must be diagnosed
  against the fixed baseline without restoring a replay path.
- Run `cargo clippy -p tablepro-ui --all-targets -- -D warnings` and full
  required repository gates on the integrated branch.
- Audit remaining TablePro presentation/application helper crates and all
  production dependency/resource/build paths for replay mechanisms; this
  change removes the 16 app-level historical screen modules only.
- Shared field editing and drag-selection contracts remain outside this
  scoped change; no app-local input workaround was added.
- External consumer and architecture-boundary checks remain outstanding.
