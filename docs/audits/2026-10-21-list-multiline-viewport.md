# List multiline viewport correction

Status: WIP candidate; independent review and PR #38 integration pending.

`List::update` computed the logical PageUp/PageDown distance in a local named `viewport`. That shadowed the cell-height viewport used by the final `scroll_view` call, so the final layout reset multi-line row scrolling using a row count. Existing regression checks failed: `three_lines_share_key_and_gap_has_no_item_hit`, `reveal_wheel_and_resize_preserve_stride_and_reach_last_key`, and `scrollbar_drag_and_multi_step_wheel_share_row_aligned_extent`.

Renamed the logical page distance to `page_rows`, preserving PageUp/PageDown behavior while the final scroll publication retains the cell viewport. Added a real mouse-down checkpoint before release at viewport height 8, a wheel offset check through tick/redraw, and a stride-1 control.

Evidence on the pre-fix source: new pointer checkpoint failed with viewport `2`, expected `8`; existing suite had 3 failures and 7 passes. Evidence after fix: `cargo nextest run -p termrock-navigation --test list_row_extent` passed 11/11; `cargo nextest run -p termrock-navigation` passed 207/207; `cargo clippy -p termrock-navigation --all-targets -- -D warnings` passed; `cargo fmt --check --package termrock-navigation` and `git diff --check` passed.

No baseline artifact or application comparison was run; this is a component-level regression correction, not product parity evidence. Default one-line scrolling remains covered in the new test.
