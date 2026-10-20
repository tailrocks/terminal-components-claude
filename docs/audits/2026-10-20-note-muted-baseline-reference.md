# Note muted-color test reference correction

The `note_wrap_and_clip` test expected RGB `(128, 128, 128)` for the muted
note tone. The frozen baseline source and its stored TablePro capture agree
on RGB `(138, 138, 138)`. This was a stale test expectation, not a product
rendering defect. The test now expects the frozen reference value; production
rendering and all existing wrap, clip, and geometry assertions are unchanged.

Evidence reviewed independently:

- Frozen baseline source commit: `4169783ae883788186d161af09ba9c481bd84e5c`.
- Baseline `src/theme.rs` defines `MUTED` as `rgb(0x8a8a8a)` and uses it for
  muted text styling (source locations around lines 528, 753, and 1192).
- Frozen TablePro `form_new/120x40/truecolor.ansi` capture contains
  `38;2;138;138;138`; reported artifact digest:
  `b767c9a2a384d0d699f8c7d2a2adbe74096447ccc1af699b83030a060feb9094`.
- The previous test expectation (`128`) is preserved as historical evidence;
  the prior `note.rs` source SHA256 is
  `d777292f23832052994c69b21e98e35ec5536bbbf276be300227dcbb071c2db8`.

Validation:

- Before correction, `cargo nextest run -p termrock-fields -E
  'test(note_wrap_and_clip)' --no-fail-fast` failed because actual muted tone
  was `(138, 138, 138)` while expected was `(128, 128, 128)`.
- After correction, the same focused command passed (1 passed, 66 skipped).
- `cargo nextest run -p termrock-fields`: 67 passed, 0 skipped.
- `cargo clippy -p termrock-fields --all-targets -- -D warnings` passed.
- `rustfmt --check --edition 2024 crates/termrock-fields/src/note.rs` and
  `git diff --check` passed. Workspace `cargo fmt --check` still reports
  unrelated existing diffs in `crates/showcase-demos/src/pages/chrome.rs`.

No baseline source, frozen artifact, or production code was changed. The
reference decision was independently reviewed and approved before this test
expectation was corrected.
