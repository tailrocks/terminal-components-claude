# Public command-scope adapter

Plan: `command-scope-public-api-plan-7a104258b.json`, SHA-256 `9965d08f122c610e14b8901303fc4c26de5c6e44c3e352d6378608d1d9a9eaf6`.

This increment adds `Ui::command_scope(owner, table, render)` as a thin wrapper over `focus_scope(owner, ScopeMode::Normal)` and the existing scoped Bubble publication API. It runs the supplied renderer once, then publishes the static typed table with empty state flags while still inside that scope. The wrapper creates no focus control, hit region, pixels, cursor request, geometry, global key capture or domain effect. An empty table explicitly disables scope commands for that configuration. Existing component keymap remapping and removal remain in force.

The public-consumer tests use a real Button in a live command scope and a real Dialog with a focused PropsList. They exercise: background `i`/`x` fallback after the modal closes; suppression of those background bindings while the Dialog owns input; three close/reopen cycles and restoration of the actual opener Button; pointer reopen plus keyboard reopen; resize from 60×20 to 24×8 and back; `remap_component` and `remove_component`; empty binding tables; and a 0×0 view with no focus target. They compare all rendered cells, all registered regions, focus and cursor with the existing author primitives across TrueColor, Ansi256, Ansi16 and Mono profiles at 60×20 and 24×8.

Relevant existing runtime decline/staleness tests also passed: a declined inner binding continues to the outer scope and then global Escape, while opening a modal or moving focus during the same physical event suppresses stale outer/global dispatch. Independent research showed that global Capture and Bubble bindings bypass focused Dialog+PropsList and fire background `i`/`x`; this increment does not change those global policies or migrate Jackin's route-local global keys. Jackin adoption requires a separate shared focusability decision for the rendered Steps rail; adding an invisible control or a fallback when no real child is focused is explicitly excluded.

Focused verification:

- `cargo nextest run -p termrock --test slots --features termrock/testing --offline`: 10 passed, 0 skipped.
- `cargo nextest run -p termrock-runtime --offline -E 'test(declining_scoped_bubble_falls_through_to_application_escape) | test(declining_inner_scope_continues_to_outer_scoped_bubble) | test(opening_modal_from_inner_scope_skips_stale_outer_binding) | test(opening_modal_from_leaf_skips_stale_global_escape_fallback) | test(moving_focus_from_leaf_skips_stale_scope_and_global_fallbacks)'`: 5 passed, 375 filtered.
- `cargo clippy -p termrock-runtime -p termrock-overlays -p termrock --all-targets --features termrock/testing --offline -- -D warnings`: passed.
- `rustfmt --edition 2024 --check` on `ui/command_scope.rs` and `tests/slots.rs`, plus `git diff --check`: passed.

This API-level increment does not modify an application, remove global keymap entries, or claim frozen baseline parity. The worktree's 1,301 pre-existing tracked deletions under `crates/termrock-conformance/target/tuiscotti-*` are untouched and must not be staged.
