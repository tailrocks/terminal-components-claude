# Master/detail state purity repair

## Scope

Addressed MD-1 and MD-3 from the independent API boundary review of `efda9593a905e3d462a62852748cf43254276dca`. `MasterDetail::draw` now reads an already-resolved `MasterDetailState` and uses the same `layout` result exposed to callers. Focus reconciliation now runs from `update`, where it changes the caller-owned state before subsequent drawing and input routing. Drawer state no longer uses interior mutability.

The public constructors, builder methods, update, layout, and draw signatures remain unchanged. SplitPane still owns split sizing and seam interaction.

## Regression coverage

`drawing_real_children_does_not_reconcile_or_mutate_caller_state` uses two real Buttons with distinct domain labels. It focuses the detail child, draws twice, and verifies state, focus, and resolved geometry remain unchanged. A subsequent update reconciles focus into drawer state; narrow layout and drawing agree. `focused_detail_child_selects_drawer_after_narrow_resize` retains the resize path and verifies master focus closes the drawer during update, before the next layout.

Focused nextest command: `cargo nextest run -p termrock-controls --lib -E 'test(master_detail)'` — 15 passed, 73 filtered. Full controls command: `cargo nextest run -p termrock-controls --all-targets` — 139 passed, 0 skipped. Strict scoped lint: `cargo clippy -p termrock-controls --all-targets -- -D warnings` — passed. Formatting: `cargo fmt --package termrock-controls --check` — passed.

## Deferred findings and limits

MD-2 remains open: `crates/termrock/src/blocks/master_detail.rs` still owns a second responsive composition path. This increment does not change facade exports or app adoption. The independent review's broader modal, pointer, child interaction, and application parity checks remain required. No architecture or product acceptance is claimed.
