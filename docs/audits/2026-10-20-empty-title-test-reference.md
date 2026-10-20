# Empty title owner test reference repair

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

Changed only the stale TrueColor expectation to `Rgb(138, 138, 138)`. Existing
owner foreground/background/modifier assertions, child override precedence,
instance-patch isolation, and sentinel isolation remain unchanged. The
aggregate fingerprint `385_672_878_558_749_177` was not changed.

## Verification and limits

- Before correction, full `empty_style_inheritance` nextest: 2 passed, 6 failed. Failures were the five owner tests plus `standalone_empty_cells_match_pre_inheritance_reference`; the latter failed at the separate aggregate fingerprint assertion (actual `2_278_838_586_319_248_431`, expected `385_672_878_558_749_177`).
- After correction, focused five owner tests: 5 passed, 0 failed.
- After correction, full `empty_style_inheritance` nextest: 7 passed, 1 failed. Only the unchanged aggregate fingerprint test remains red.
- Frozen source, tag, widget behavior, and aggregate expected value remain unchanged. The remaining fingerprint conflict needs independent diagnosis; this test-reference correction does not resolve it or claim the full test file passes.
