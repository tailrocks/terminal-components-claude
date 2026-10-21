# Column solver extraction

Source: PR #38 candidate `6825b07e86bab75872866c6ed49e6877084db9bb`.

`termrock-navigation::ItemColumns` previously owned a private four-column
solver in `crates/termrock-navigation/src/item.rs`. It measured picker rows
using minimum, ideal, maximum, flex, and drop behavior, while the layout crate
had no reusable column specification. This increment moves those mechanics to
`termrock-layout::columns` and adapts `ItemColumns` to the shared solver. Picker
measurements retain the existing 45% label cap, constraints, gap, and drop
ordering.

The shared API resolves any input slice without truncating it. Hidden columns
have no offset; visible zero-width columns retain an offset, so callers can
preserve gaps between visible columns without gaps for dropped columns. The
first-column helper uses stack workspaces for the existing four-column picker
measurement, retaining its no-allocation draw contract.

Verification used separate target directories for the frozen solver test
before and after the change:

- Before: `CARGO_TARGET_DIR=/private/tmp/shared-columns-before-target-20261021 cargo nextest run -p termrock-navigation --test item_row_columns` — 14 passed, 0 skipped.
- After: `CARGO_TARGET_DIR=/private/tmp/shared-columns-after-target-20261021 cargo nextest run -p termrock-layout --lib` — 32 passed, 0 skipped. Differential test compares the old four-column solver for every available width 0–160, gaps 0–3, and 781 constraint profiles.
- After: `CARGO_TARGET_DIR=/private/tmp/shared-columns-after-target-20261021 cargo nextest run -p termrock-navigation` — 208 passed, 0 skipped, including all four color profiles, frozen full-cell fingerprints, and the warmed zero-allocation renderer test.
- `CARGO_TARGET_DIR=/private/tmp/shared-columns-after-target-20261021 cargo clippy -p termrock-layout -p termrock-navigation --all-targets -- -D warnings` — passed.
- `cargo fmt --all -- --check` and `git diff --check` — passed.

An initial implementation allocated a temporary workspace during picker draw;
the existing allocation regression test caught it. `solve_columns_first`
shares the same solver with a bounded stack workspace for up to four columns
and falls back to the unbounded API for larger slices. No Holla or Showcase
consumer migration is included in this increment. Product parity and the
remaining navigation layout contracts remain unverified.
