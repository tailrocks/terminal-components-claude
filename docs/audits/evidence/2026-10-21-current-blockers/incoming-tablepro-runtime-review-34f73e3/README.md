# Incoming TablePro/runtime review

Reviewed source commits d0e7ec9bfb8f77861cedbfa35bd1ec678f195a64 and 05d63fe9c8a0e9a327489ae657a4ff6dc6571324, integrated in 34f73e3066da7a77eac2d4719a697dc085906dda. Comparison parent is 5f82c4d35d12e0fb77b0f580a8ef200240661602. Frozen source is 4169783ae883788186d161af09ba9c481bd84e5c; fixed design reference is f4867e9b5f8a08a3bb61ce80d644198a29f51984.

Source-only stage; compilation and probes have NOT run because the coordinator reserved a frozen-capture quiet window. The prepared probe uses real public Ui::scroll_edges and TableProApp/Harness. Resume it unchanged against both source revisions after release. This is not product parity evidence.

Findings:

1. Runtime paint.rs:241–264 drops all background colors first encountered after the first eight distinct colors. A ninth background covering 32 of 40 cells cannot become the majority; scroll_edges therefore fails to fade those cells. The old HashMap counted every color. Keep exact majority semantics (a small-table fallback is acceptable); add a public render regression, not a private helper-only assertion.
2. TablePro app.rs:1251 clones the active connection into the New connection draft. Frozen connections.rs:138–184 and 523–529 use ConnForm::new(None,None) with blank Name, Database, and Username. Previous candidate also initialized these blank. app.rs:9776–9782 changes the empty_new_connection_draft test helper to accept this behavioral change. Restore creation defaults independently of editing/duplication and test the real form.
3. TablePro app.rs:10219–10231 replaces the fixed New connection card with a full-content-width Connect to database panel. Removing the overwritten/invisible form and exact 120x40/truecolor/focus painter is correct, but the replacement does not preserve the frozen New connection title, Ctrl+S Save metadata, sidebar plus detail composition, or detail width min(84). Frozen connections.rs:1385–1405 defines those contracts. Fix shared form/panel/layout contracts instead of restoring the old painter.
4. TablePro app.rs:3556–3557 adds an exact SQL-string presentation branch for SELECT * FROM orders. Frozen tabs.rs:1715–1722 emits Planning … · r Raw for every non-analyzed plan. This adds unsupported visible text for one query and differs for equivalent SQL. Use actual plan/domain state and preserve frozen text.
5. Native test baselines/tablepro.txt replaces 157 of 168 digest entries (314 diff lines), with no added/removed scenarios. The commit contains no pinned frozen capture, immutable checkpoint receipts, reference-data defect diagnosis, or documented reference decision. These new values cannot establish parity. Preserve both Git blobs, diagnose against the real frozen binary, and do not bless candidate output.

Valid cleanup: tests/tab_ownership.rs:148 removes only a redundant Some(Tab::Query(_)) => None match arm followed by _ => None. No assertion or test was removed. The Grid::move_cursor_to added deferred ensure-visible call has no source-proven correctness defect here, but resize/keyed-state runtime verification remains outstanding.

Remaining architectural violations include app.rs:10238–10262 drawing public Tabs and then paint_legacy_tabs over it; extensive manual Grid/connection/header painters, app geometry, terminal stdout cursor manipulation in prepare_grid_slots, and pin_tab_window component mechanics. These predate this increment; the commit does not establish the library gate.

No Rust checks, real CLI comparisons, broad product gates, or approval of refreshed expected hashes are claimed at this stage.
