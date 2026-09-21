# TASK-079 fade-mix helper obligations

Each row maps one TASK-014 fade demand to the named theme binding this
repair must provide. All in-tree demand sources are read-only inputs:
TASK-014's package (R-001, W-014-01..W-014-04), `components.md` CP-01,
and the palette rule (`COMPONENT_ARCHITECTURE.md` §22.7 R-10/D-10,
`xtask/src/main.rs` `palette_literals_are_confined_to_theme_builtins`).
No oracle source beyond those in-tree descriptions is consulted; the
pinned transcription below is the exact authority the exhaustive sweep
proves the implementation against. Transcription-vs-oracle truth is
TASK-014 r3's oracle-lane proof, not this task's.

## O-001 — Oracle-exact fade-mix helper (CHK-002, R-001, AC-001)

Demand: TASK-014 R-001 requires oracle `src/ui/fade.rs` arithmetic —
55% outer / 80% inner retained contrast, componentwise rounding,
majority-background ties, non-RGB outer DIM — but the palette rule
forbids `Color::Rgb(`/`from_u32`/hex construction anywhere outside
`theme/builtin/junie.rs`, `theme/builtin/paper.rs`,
`theme/builder.rs`, and `theme/downgrade.rs`, and no callable theme
API blends today (`blend`, `shift_l`, `lab_to_rgb`, `nearest_256` are
all private; `rgb_of` cannot discriminate `(Rgb, Rgb)`;
`downgrade_color` collapses; `derive_unset` has fixed alphas). TASK-014
scope excludes `theme/`, so the named path must exist first.

Obligation: add exactly one `pub(crate)` helper plus its directly
necessary outcome type in `crates/tui/src/theme/builder.rs`, reachable
as `crate::theme::builder::fade_mix` with no `mod.rs` change
(`pub(crate) mod builder` already exposes the module crate-wide):

```rust
pub(crate) enum FadeOutcome { Blended(Color), ApplyDim, Unchanged }
pub(crate) fn fade_mix(fg: Color, bg: Color, amount: f32) -> FadeOutcome
```

`Blended(c)` paints `c`. `ApplyDim` keeps `fg` and the caller adds `DIM`.
`Unchanged` keeps `fg` as-is. The branch table, with the two legal
amounts `0.55` (outer row) and `0.80` (inner row) per TASK-014 R-001:

| `fg` | `bg` | `amount` | outcome |
| --- | --- | --- | --- |
| `Rgb` | `Rgb` | caller amount | `Blended` of the pinned mix below |
| any non-`Rgb` on either side | | `0.55` | `ApplyDim` |
| any non-`Rgb` on either side | | `0.80` | `Unchanged` |
| any non-`Rgb` on either side | | any other | `Unchanged` (fail closed) |

Row is encoded by the amount: outer holds iff `amount == 0.55f32`
exactly (same-literal comparison is exact; no epsilon). Named,
`Indexed`, and `Reset` are all non-`Rgb`. Majority-background
selection, ties, reversed/different-background exclusion, and
protected rows stay TASK-014's selection logic; this helper owns only
the per-cell color branch. Pinned channel transcription (`f32` ops in
this order, round half away from zero, clamp, cast):

```text
mix(f, b, a) = ((f as f32) * a + (b as f32) * (1.0 - a)).round().clamp(0.0, 255.0) as u8
```

Pinned vectors from the W-014-01 witness inputs `fg (120,160,200)`,
`bg (20,40,60)`: outer `0.55` yields `(75,106,137)`; inner `0.80`
yields `(100,136,172)`. The existing private `blend` uses `f64` and is
not known bit-exact to this `f32` transcription at `.5` boundaries;
reusing it is at the implementer's risk and the sweep decides. No
other `Color::Rgb`/`from_u32`/hex construction is added anywhere.

## O-002 — Exhaustive parity proof (CHK-002, R-002, AC-001)

Demand: the helper must equal the O-001 transcription on its full
input space, not on a sample. A representative subset cannot carry a
rounding-boundary proof.

Obligation: colocate exactly these four `#[test]`s in
`crates/tui/src/theme/builder.rs` `mod tests` (fixed names; the
CHK-002 filter `fade_mix_oracle_` matches exactly this set):

- `fade_mix_oracle_outer_matches_pinned_vector`: W-014-01 inputs at
  `0.55` yield exactly `Blended(Color::Rgb(75,106,137))`.
- `fade_mix_oracle_inner_matches_pinned_vector`: W-014-01 inputs at
  `0.80` yield exactly `Blended(Color::Rgb(100,136,172))`.
- `fade_mix_oracle_sweep_matches_transcription`: for every channel
  pair `(f, b)` in `[0,255]^2`, independently per channel, at both
  `0.55` and `0.80`, the helper's channel equals the pinned `mix`
  transcription. No sampling, no boundary-only subset.
- `fade_mix_oracle_non_rgb_branch_matrix`: `fg` by `bg` over all
  sixteen named colors, all 256 indexed values, `Reset`, and `Rgb`
  samples, at both amounts: `(Rgb,Rgb)` cells match the transcription;
  every other pair yields `ApplyDim` at `0.55` and `Unchanged` at `0.80`,
  keeping the input `fg`; illegal amounts (`0.0`, `0.7`, `1.0`) with a
  non-`Rgb` pair yield `Unchanged`.

CHK-002 runs `cargo nextest run --locked -p junie-tui --lib
fade_mix_oracle_` and all four tests pass. This proves
implementation == transcription; transcription == oracle is proven by
TASK-014 r3's oracle lanes.

## O-003 — No other theme change, genuine accounting (CHK-003, CHK-005, R-003, AC-002, AC-003)

Demand: the repair must change no other theme behavior, and its
executions must account through the genuine production path.

Obligation: CHK-003 runs `cargo nextest run --locked -p junie-tui
--lib theme::`; every pre-existing colocated theme unit test
(builder, downgrade, and sibling theme modules) passes unchanged.
TASK-011's integration targets stay TASK-011's own gate, not this
task's. CHK-005 judges the prepared production account-tests context
with the trusted `acct-presence` driver: the template declarations
lift exactly, both requires flags are true, and `accepted_inventory`
plus `accepted_disposition` are present as well-formed provenance
objects. Projection correctness is TASK-078's ownership, consumed via
the base tree; this driver asserts envelope plus presence only.

## O-004 — Strict scope, no TASK-014 work (CHK-001, CHK-006, gate CHK-007, R-004, AC-004, AC-005, AC-006)

The change is confined to `crates/tui/src/theme/builder.rs`: the O-001
helper plus its outcome type and the O-002 colocated tests only.
Forbidden: every theme sibling (`mod.rs`, `border.rs`, `builtin/`,
`downgrade.rs`, `glyph.rs`, `palettes.rs`, `patch.rs`, `recipe.rs`,
`resolve.rs`, `role.rs`, `tokens.rs`), every TASK-014 scope file
(`scroll.rs`, `components/scroll_region.rs`, `ui/paint.rs`,
`author.rs`, `tests/completion_014.rs`) — TASK-079 must not do
TASK-014's work — any new `crates/tui/tests/completion_079.rs`, any
other product file, any `refactoring-tasks/**` or proof-tool change,
and any further `Color::Rgb`/`from_u32`/hex construction site. The
palette rule stays green via CHK-006. Taskfmt scope plus
forbidden-path gates enforce this; the reviewer confirms it.
