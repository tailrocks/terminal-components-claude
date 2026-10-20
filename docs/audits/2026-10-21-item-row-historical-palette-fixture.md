# Item-row historical palette fixture repair

The failing check is a historical library-constructor fixture, not the frozen application visual oracle. Its expected ANSI-256 full-cell fingerprint remains `12184706452759668872`. Production Junie palette values and application parity expectations remain unchanged.

At PR #38 source `efda9593a905e3d462a62852748cf43254276dca`, the fixture reconstructed `fg[2] = Indexed(244)` and `fg[3] = Indexed(238)`, but omitted the historical `disabled_fg = Indexed(238)`. Its third item is disabled. The focused nextest run failed with fingerprint `17020103994864682368` before this repair. Adding the missing fixture-only token makes the existing fingerprint pass.

The test captures the omitted-token cell diff: 56 foreground-only changes, all on the disabled row at terminal row `y = 3`, columns `x = 1`, `x = 3..=56`, and `x = 64`; each changes `Indexed(239)` to `Indexed(238)`. Symbols, backgrounds, modifiers, cursor, and all registered hit regions remain equal. After reconstructing the historical palette, converting 174 `Indexed(244)` muted cells to frozen `Indexed(245)` and the 56 disabled-row cells to frozen `Indexed(239)` produces a full buffer equal to the current frozen-token rendering. The historical fingerprint is unchanged.

The recorded passing ancestor `80b50e4c5fb993ad29b9edf5868391d1499e1764` uses the pre-correction palette and is the source of the retained checksum; current focused tests verify that checksum and compare every buffer cell, cursor, and region for the reconstructed and frozen renderings. This run did not execute the whole test binary at `80b50e4`, so cross-commit registry equality remains unclaimed. The exact cell-level comparison and 80b50e4 evidence are recorded in the diagnosis packet at `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/itemrow-fingerprint-diagnosis-20261021.json` (SHA-256 `c9858e1ad1db08711cf0a627b0fa6f4a3a12ab4af23efcb005bf68f6fe3193b8`).

Validation on this change:

- `rustup run 1.99.1 cargo nextest run -p termrock-navigation --test item_row_columns constructor_defaults_match_the_pre_policy_full_cell_fixture --features termrock-runtime/testing --no-capture` — 1 passed.
- Full `item_row_columns` target, strict Clippy, and formatting checks remain to run before publication.
