# Master/detail responsive contract consolidation

## Scope

Addressed the duplicated responsive state and mode/summary decision reported as MD-2 in the independent review of `efda9593a905e3d462a62852748cf43254276dca`. The low-level control owns the shared `MasterDetailState` now re-exported by the composition facade. This preserves the facade's public state name, `Copy`, `Default`, and `drawer_open()` contract while using one drawer and focus-return state value. Its existing split state remains available to the resizable controls adapter.

Both adapters now call `termrock_controls::MasterDetail::resolve_responsive_layout` for mode selection and narrow summary geometry. The composition facade uses that result in measure, show, and scoped Escape handling; its Groups still own content composition. The controls adapter adds its configured `SplitPane` regions to that responsive result. SplitPane remains the shared owner for actual split geometry and seam interaction.

The adapters retain distinct defaults and input contracts:

| Adapter | Split policy | Collapse | Summary | Input |
|---|---|---:|---:|---|
| Controls | 32% master; minima 28/40; gap 2; user-resizable seam | 100 columns | 6 rows | master Tab opens; detail Escape/Left/BackTab close; remappable |
| Composition facade | 34% detail bounded 34–48; gap 2; fixed pane | 108 columns | 1 row | typed open/close actions; scoped Escape after detail declines |

The public method `resolve_responsive_layout` returns empty split rectangles in `Split` mode by design. The configured `SplitPane` owns those regions; the control's `layout` method returns its complete split regions. The facade uses only the shared mode and summary result before delegating split drawing to its configured pane.

## Regression coverage

`facade_and_control_share_resolver_but_keep_default_breakpoints_and_rows` checks the 104-column adapter boundary and the default one-row versus six-row summaries at 99 columns. The public file-catalogue consumer still exercises measured/drawn responsive content, repeated drawer open/close, nested clipped width, and scoped keyboard behavior.

Checks on the final source before commit:

- `cargo nextest run -p termrock-controls -p termrock --all-targets --features termrock/testing` — 165 passed, 0 skipped.
- `cargo nextest run -p termrock --test master_detail_consumer --features testing` — 7 passed, 0 skipped.
- `cargo clippy -p termrock-controls -p termrock --all-targets --features termrock/testing -- -D warnings` — passed.
- `cargo fmt --package termrock --package termrock-controls --check` — passed.

## Remaining scope

This increment does not remove the old facade composition block, merge the two adapters' different keyboard policies, or prove application parity. The file-catalogue sample exercises the facade, but a broader external consumer review must verify public state compatibility, update-before-layout ordering, and exact pane geometry. Product acceptance remains incomplete.
