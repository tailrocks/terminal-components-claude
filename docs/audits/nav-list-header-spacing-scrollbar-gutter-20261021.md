# NavList header, spacing, and stable gutter contract

This increment adds three optional NavList presentation settings without
changing existing defaults:

- `header_text_patch` styles the header label cells independently from the
  header padding. The existing whole-header slot still replaces the complete
  header renderer.
- `marker_gap` reserves cells after the one-cell marker and before the icon;
  the icon and label move together, and the reserved cells are clipped as part
  of the row. Its default is zero.
- `ScrollbarGutter::{Auto, Stable}` controls the shared ScrollRegion gutter.
  `Auto` keeps the previous width until content overflows. `Stable` reserves
  exactly one cell from the start; an overflowing region paints the scrollbar
  in that same reserved cell. `scrollbar_visible(false)` still disables both
  the scrollbar and its gutter.

The default rendering path continues to use the existing header style,
zero marker gap, and automatic scrollbar reservation. Existing NavList and
ScrollRegion rendering tests remain the compatibility oracle; this change does
not introduce application-specific variants, labels, or drawing paths.

Focused proof on the implementation revision:

- `cargo nextest run -p termrock-navigation nav_list::tests`: passed.
- `cargo nextest run -p termrock-runtime scroll_region::tests`: passed.
- `cargo nextest run -p termrock-navigation -p termrock-runtime`: 610 passed;
  three ignored tests were not run by the default profile.
- `cargo clippy -p termrock-navigation -p termrock-runtime -p termrock
  --all-targets -- -D warnings`, `cargo fmt --all -- --check`, and
  `git diff --check`: passed.
- Regression checks cover header text versus padding, header-slot precedence,
  marker and icon/label displacement, and stable gutter widths with and
  without overflow.

An exploratory `--run-ignored all` run exposed two terminal-driver fixtures
that require a configured terminal device in this environment:
`termrock-runtime::monotonic::idle_terminal_driver_fixture` and
`termrock-runtime::monotonic::immediate_rearm_terminal_input_fairness_fixture`
failed with `Device not configured`. They are intentionally ignored by the
default suite and are unrelated to this NavList/ScrollRegion change.

No application consumer migration is included in this contract increment.
