# Master/detail resizable policy

Contract: `master-detail-policy-contract-4db067a8-20261021/contract.json`, SHA-256 `9dab3c3dd67d5d6a16fdfe8445d086e6aded91118f7d79aa123d042dc6771061`.

The facade now offers an opt-in `MasterDetailPolicy::resizable_master(initial_percent, min_master, min_detail)` policy. The existing fixed-detail defaults and fixed drawing/update path remain intact. The controls `MasterDetail` now exposes its configured `split_pane()` for adapters that need to measure the same component policy. The facade delegates update, split state, responsive regions, focus routing, and seam behavior to that existing controls engine.

`MasterDetailAction::Resized(u8)` is a public enum addition so callers can observe split changes without losing `Response` flow or invalidation metadata. Downstream exhaustive matches may need a new arm. `detail_share_percent()` continues to report the fixed-detail setting and is documented not to report the opt-in master share. Split state initializes during update only; measure and draw use a state snapshot.

The file catalogue example accepts borrowed document data and demonstrates a separate 45% master policy. Its default remains the existing fixed-detail policy. Consumer coverage checks that the default and explicitly restated fixed policy produce equal complete buffers, cursor/focus visibility and control regions at the prior breakpoint matrix; opt-in widths 99–102; tiny and short viewports; measurement/draw state purity; drag persistence through collapse/re-expansion; immediate resize input deferral/replay; and reordered stable IDs with empty, long and Unicode labels.

Focused Rust verification uses `cargo nextest`; no frozen reference binary or paired application capture ran in this increment. This does not establish Accounts migration, complete master/detail interaction parity, whole-application visual parity, or library architecture acceptance. The resize-before-presentation behavior is covered here using the runtime's pending-input contract; runtime-wide operational preservation remains a separate check.
