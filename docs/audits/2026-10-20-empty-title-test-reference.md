# Empty title test reference corrections

## Fixed reference and diagnosis

This test-only correction keeps the frozen product source at
`4169783ae883788186d161af09ba9c481bd84e5c` and the tag object at
`2718a6b8127a522a3d351d0b4bdb7542fc8da542`. The isolated test branch began at
`95cdc7bdb111dc1b683a638b421dae718bb29643`, which includes the shared palette
corrections.

On that base, `cargo nextest run -p termrock-theme --test
empty_style_inheritance` produced five owner-test failures and one separate
aggregate fingerprint failure. Each owner failure reported actual `Rgb(138,
138, 138)` against the test's expected `Rgb(128, 128, 128)` at the “other
instance” assertion in `crates/termrock-theme/tests/empty_style_inheritance.rs`.

The expected `Rgb(128, 128, 128)` was stale. Frozen `src/widgets/empty.rs`
renders an empty title with `t.muted()` (line 175). Frozen `src/theme.rs`
defines `MUTED = rgb(0x8a8a8a)` (line 528), assigns it to `text_muted` (line
753), and `Theme::muted()` returns that token (lines 1191–1192). An independent
frozen-widget probe rendered `EmptyState::new("Q")` and observed TrueColor
`Rgb(138, 138, 138)`, Ansi256 `Indexed(245)`, and Mono `Gray` for the title.
Probe source SHA-256:
`f6a606e3217eafac5e988597e97932d06d867f63e8f2e76db7e2685425fce774`; nextest
log SHA-256:
`d358d3dcbdb15920af73cb0870e07521229ace6f0516f66159e42d591a590bc5`.
Evidence is retained at `/tmp/termrock-empty-frozen-proof/`.

The owner-test repair changed only the stale TrueColor expectation to
`Rgb(138, 138, 138)`. Existing owner foreground/background/modifier
assertions, child override precedence, instance-patch isolation, and sentinel
isolation remain unchanged.

## Independent aggregate reference decision

The aggregate test hashes `Debug` bytes for all 32 complete cells frames; it
does not include cursor state. The test was introduced at
`ae317b61552f7d3a2919d4faacda682be5976932`. Its executed pre-inheritance
parent is `a4a702c6d12cff7d776285e87b32d09caa5ba33f`, where the independently
reproduced original hash is `10_278_960_817_768_212_803`. The later test-only
expectation `385_672_878_558_749_177` was added at
`2f1e4088033474e0daba5d2776568f04f75b0547` and conflicts with palette
semantics in the immutable frozen source. The historical `db53` artifact was
unavailable; it is not claimed as recovered.

Independent review projected only the frozen palette rules from source
`4169783ae883788186d161af09ba9c481bd84e5c` onto the exact 32 frames generated
by `2f1e4088033474e0daba5d2776568f04f75b0547`. The projection changed exactly
32 foreground values across 5,376 cells: muted/faint entries in TrueColor,
Ansi256, indexed 16, and Mono. It changed no symbols, positions, backgrounds,
attributes, underline data, or skipped comparisons. All projected frames
matched the current source output at `95cdc7bdb111dc1b683a638b421dae718bb29643`
cell-for-cell. The independent projection, not candidate output, yields the
correct aggregate fingerprint `2_278_838_586_319_248_431`; the test's
full-cell comparisons remain enabled. Frozen `Empty` itself has only Empty and
Error states and a different Error layout, so this decision is solely palette
authority, not a claim that these 32 states are frozen-baseline UX.

Preserved evidence at
`/private/tmp/termrock-empty-fingerprint-probe-20261020/`:

- `final-attribution.json` SHA-256:
  `abaccd91419e6d4e5282c368c24488beae8b46e204070cfb440c45cf2616b6d1`
- `projection-decisions.json` SHA-256:
  `2335ea426e9f77e1d4150a59f5514fc0a467776ebd8b415b23fab499f6422e46`
- `evidence-manifest.json` SHA-256:
  `edba9d25598f00a55dd0db9540470bf7cb65b5dc9ee34aa335a42d96b5a7b22a`
- Probe source (`src/lib.rs`) SHA-256:
  `d80c08f9b043b00587a21edd45cead5716d37506e3497f77e2e219a7bf87358b`

The old expectation and original source hash remain recorded above so the
reference correction is auditable; no frame or full-cell comparison was
removed or weakened.

## Verification and limits

- Before correction, full `empty_style_inheritance` nextest: 2 passed, 6 failed. Failures were the five owner tests plus `standalone_empty_cells_match_pre_inheritance_reference`; the latter failed at the separate aggregate fingerprint assertion (actual `2_278_838_586_319_248_431`, expected `385_672_878_558_749_177`).
- After correction, focused five owner tests: 5 passed, 0 failed.
- After the independent aggregate reference correction, `cargo nextest run -p termrock-theme --test empty_style_inheritance`: 8 passed, 0 skipped.
- `cargo clippy -p termrock-theme --all-targets -- -D warnings`, `cargo fmt --check --package termrock-theme`, and `git diff --check`: passed.
- Frozen source, tag, and widget behavior remain unchanged. The aggregate expected value now reflects the independently reviewed frozen palette projection; the test states remain broader than the frozen Empty widget and are not represented as frozen UX coverage.
