# Overlay dialog test lifecycle repair

At source revision `05a85a0065005eddb9979b852ff15f9036c8099c`, strict Clippy reported a duplicate `#[test]` on `border_head_paints_the_padded_title_in_the_top_border_row`, a missing `#[test]` on `body_is_total_and_clipped_when_the_dialog_cannot_draw`, and four unused `PaintedFrame` results in the direct-buffer dialog tests.

The test module now has one annotation per test. The clipped-body test is discovered by nextest. The four tests that inspect painted buffer cells explicitly drop the returned `PaintedFrame`; they do not commit presentation state because those assertions exercise the render buffer directly. No production component behavior or expected output changed.

Validation on the edited source passed:

- `rustup run 1.99.1 cargo nextest run --locked -p termrock-overlays --all-targets --no-fail-fast --status-level fail --final-status-level fail`: 156 passed, 0 skipped.
- `rustup run 1.99.1 cargo clippy --locked -p termrock-overlays --all-targets --no-deps -- -D warnings`: passed.
- `rustup run 1.99.1 cargo fmt --all -- --check` and `git diff --check`: passed.
