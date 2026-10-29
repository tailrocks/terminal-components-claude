# Junie capability palette corrections

## Scope and fixed source

This increment corrects Junie's muted and faint foreground entries in the
ANSI-256 and Mono capability palettes. Frozen source remains
`4169783ae883788186d161af09ba9c481bd84e5c`; the `visual-baseline` tag object
remains `2718a6b8127a522a3d351d0b4bdb7542fc8da542`. Work began from shared-style
source `e72dbb15369d541ee86095f6caaee3f7e4680828` on isolated branch
`parity/theme-palette-baseline-20261020`.

Frozen `src/theme.rs` defines 256-color muted and faint values as indices 245
and 242 (`Theme::apply_256`, lines 921–941), and Mono muted and faint values as
`Gray` (`Theme::apply_mono`, lines 1053–1078). The truecolor `TEXT_FAINT` is
RGB `#737373`, whose shared downgrade rule maps to `Gray`. The existing unchanged
`crates/termrock-theme/tests/capability_palettes.rs::all_73_junie_slots_match_reference_capability_output`
computes expected ANSI-256 values from the truecolor semantic tokens. On the
starting source, it failed at `Ansi256`: `fg[2]` and `fg[3]` were 244 and 238,
while expected values were 245 and 242. `junie_256()` now uses those two
baseline indices.

That unchanged test then exposed Mono `fg[3]`: explicit `junie_mono()` used
`DarkGray`, while the frozen `text_faint` value and generic projection both
resolve to `Gray`. The actual frozen rules prove the palette defect; the Mono
`fg[3]` entry now uses `Gray`. No test expectation changed. Both corrections
stay in the shared theme palette; application-specific rendering is untouched.

## Verification and remaining gaps

- Before corrections: `cargo nextest run -p termrock-theme --test capability_palettes all_73_junie_slots_match_reference_capability_output` — reproduced the ANSI-256 mismatch at source `e72dbb15369d541ee86095f6caaee3f7e4680828` (exit 100).
- After ANSI-256 correction at `737ecc836fc40d77b51d797bda9c3927b12eeee7`: the unchanged test advanced to Mono and failed on `fg[3]` (`DarkGray` vs `Gray`); this intermediate red is superseded by the Mono correction below.
- After both corrections: `cargo nextest run -p termrock-theme --test capability_palettes all_73_junie_slots_match_reference_capability_output` — 1 passed, 8 skipped.
- Full unchanged palette suite: `cargo nextest run -p termrock-theme --test capability_palettes` — 9 passed, 0 failed, 0 skipped. This covers all four profiles, custom palette overrides, and intermediate-level mutations.
- `cargo clippy -p termrock-theme --all-targets -- -D warnings` — passed.
- `cargo fmt --check --package termrock-theme` and `git diff --check` — passed.
- The unrelated pre-existing theme suite failures for empty-owner inheritance, nested layer projection, and aggregate style hashing remain unmodified and unapproved. No stored hashes or expected frames were changed.

The frozen tag, baseline artifacts, ANSI-16 palette, custom palette behavior,
and truecolor values remain unchanged. Full visual/product parity and
independent review remain pending.
