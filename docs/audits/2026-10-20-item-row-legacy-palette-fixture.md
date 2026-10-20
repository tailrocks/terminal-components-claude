# Item row fixture palette correction

## Reference decision

The full-cell hash `12184706452759668872` in
`crates/termrock-navigation/tests/item_row_columns.rs` records an earlier
fixture palette. It is not a frozen baseline frame. The fixture's earlier
muted and faint Junie ANSI-256 values were 244 and 238. Frozen source
`4169783ae883788186d161af09ba9c481bd84e5c` defines those values as 245 and
242. The frozen palette remains authoritative; the old hash remains useful
only as a checksum for the explicitly reconstructed historical fixture.

The test now renders both palettes. It first verifies the old buffer against
the historical hash, then changes only cells whose foreground is Indexed(244)
to Indexed(245). Exactly 174 cells change; symbols, backgrounds, modifiers,
geometry, and cursor remain equal to current output. A wrong muted value is
also checked to remain unequal. The existing default-versus-explicit row
policy buffer and cursor comparisons remain in place.

## Verification

- Pinned source inspected: `b0a3bea5a486c4b67a83a10ec5463c690c590208`.
- Palette diagnosis inspected: `/tmp/termrock-item-palette-diagnosis/`.
- Before repair, restored the source test from the pinned commit and ran
  `cargo nextest run -p termrock-navigation --test item_row_columns constructor_defaults_match_the_pre_policy_full_cell_fixture --no-fail-fast`:
  expected failure; current ANSI-256 fingerprint was
  `4308317598678580912`, historical fixture hash was
  `12184706452759668872`.
- After repair, the same focused command passed. The full
  `cargo nextest run -p termrock-navigation --test item_row_columns` passed
  all 13 tests with no skips.
- `cargo clippy -p termrock-navigation --test item_row_columns -- -D warnings`,
  `cargo fmt --check --package termrock-navigation`, and `git diff --check`
  passed.
- No baseline artifact, production palette, or frozen reference was changed.
