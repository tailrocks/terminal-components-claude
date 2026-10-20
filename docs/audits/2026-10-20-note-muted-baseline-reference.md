# Note muted-color test reference correction

The `note_wrap_and_clip` test expected RGB `(128, 128, 128)` for the muted
note tone. A repeated capture from the frozen `visual-baseline` application
shows RGB `(138, 138, 138)`. This was a stale test expectation, not a product
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
  its recorded change identifier is
  `d777292f23832052994c69b21e98e35ec5536bbbf276be300227dcbb071c2db8`.

No baseline source, frozen artifact, or production code was changed. The
reference decision was independently reviewed and approved before this test
expectation was corrected.
