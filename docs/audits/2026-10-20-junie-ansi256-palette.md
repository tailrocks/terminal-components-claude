# Junie ANSI-256 semantic palette correction

## Scope and fixed source

This increment changes only the Junie ANSI-256 foreground entries for the
muted and faint semantic slots. Frozen source remains
`4169783ae883788186d161af09ba9c481bd84e5c`; the `visual-baseline` tag object
remains `2718a6b8127a522a3d351d0b4bdb7542fc8da542`. Work began from shared-style
source `e72dbb15369d541ee86095f6caaee3f7e4680828` on isolated branch
`parity/theme-palette-baseline-20261020`.

Frozen `src/theme.rs` defines the 256-color muted and faint values as indices
245 and 242 (`Theme::apply_256`, lines 921–941). The existing unchanged
`crates/termrock-theme/tests/capability_palettes.rs::all_73_junie_slots_match_reference_capability_output`
computes expected ANSI-256 values from the truecolor semantic tokens. On the
starting source, it failed at `Ansi256`: `fg[2]` and `fg[3]` were 244 and 238,
while expected values were 245 and 242. `junie_256()` now uses those two
baseline indices. No other palette entries or test expectations changed.

## Verification and remaining failure

- Before correction: `cargo nextest run -p termrock-theme --test capability_palettes all_73_junie_slots_match_reference_capability_output` — reproduced the ANSI-256 mismatch at source `e72dbb15369d541ee86095f6caaee3f7e4680828` (exit 100).
- After correction: `cargo nextest run -p termrock-theme --test capability_palettes -E 'not test(all_73_junie_slots_match_reference_capability_output)'` — 8 passed, 1 excluded.
- Full unchanged suite after correction: `cargo nextest run -p termrock-theme --test capability_palettes` — 8 passed, 1 failed. The same all-73 test now passes its ANSI-256 comparison and exposes a separate Mono mismatch: explicit `junie_mono()` has `fg[3] = DarkGray`, while expected Mono projection is `Gray`. This palette entry is outside this ANSI-256-only increment and remains unresolved. The existing all-73 check therefore remains red; no full palette pass is claimed.
- `cargo clippy -p termrock-theme --all-targets -- -D warnings` — passed.
- `cargo fmt --check --package termrock-theme` and `git diff --check` — passed.
- The unrelated pre-existing theme suite failures for empty-owner inheritance, nested layer projection, and aggregate style hashing remain unmodified and unapproved. No stored hashes or expected frames were changed.

The frozen tag, baseline artifacts, ANSI-16 and Mono palette definitions, custom
palette behavior, and truecolor values remain unchanged. Full visual/product
parity and independent review remain pending.
