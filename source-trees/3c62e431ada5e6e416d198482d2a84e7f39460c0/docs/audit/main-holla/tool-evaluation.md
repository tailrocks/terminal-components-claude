# Terminal tool evaluation (initial)

Current source revisions are pinned in tool-pins.json; inspected upstream
README, manifests and adapter sources. Stable rustc 1.98.1 builds both tools
outside the application workspace. Neither is an MSRV dependency.

- tui-snap all-target tests initially failed because its PTY test hardcodes
  source/target/debug/examples/fixture_app. Building examples in its separate
  target and adding a target symlink in the external tool checkout resolved
  that test without modifying tracked tool source. The repeated all-target
  run passed, including its existing visual fixtures (135.71 seconds).
- tui-test CLI build initially failed because Zig had no selected version.
  Explicitly adding installed Zig 0.16.0 to the build PATH succeeded. A mise
  exec attempt alone failed to execute the child build; log retained.
- Separate CLI calls lost the tui-test daemon when the tool invocation ended.
  A single Python orchestration process owning run/assert/input/resize/close
  succeeded for all ten operations. Session names are isolated and cleanup
  results are saved; this is an environment lifecycle constraint.
- Actual pinned Holla first-use 80x24 paused frame 0 captured to canonical JSON,
  PNG and text, with binary/tool/source hashes. Image inspected. First attempt
  inherited NO_COLOR=1 and contained only default colors despite --color
  truecolor; retained as contaminated evidence, never an expectation. A new
  separate capture with NO_COLOR unset preserved RGB fields and was inspected.
- Independent tui-test/alacritty run reached Holla, opened F1 help, dismissed
  it, and resized 80x24 to 100x30. Screens and cursor were inspected. This smoke
  does not prove all color/style, coordinate, effect, shutdown or timing cases.

## Qualification remaining

Explicitly test DIM/bold/reverse, color modes, Unicode wide/combining, cursor,
mouse coordinates, clipping and style-aware settling with known sentinels.
tui-snap raw ANSI adapter drops strikethrough and cursor shape; pure-buffer
adapter drops blink/hidden and clamps a right-edge wide glyph to width 1.
These paths cannot silently normalize meaningful differences. Preserve full
native cell/style data alongside visualization, and reject unsupported states
or supply a reviewed exact adapter before treating parity results as proof.
Rendered PNG uses pinned font/profile approximations, not emulator pixel identity.

External execution logs are initial/tuisnap-*.log and initial/tuitest-*.log;
reference capture/provenance is holla-reference-captures/. Original source and
reference artifacts are unchanged. Full clean-checkout acquisition/capture
scripts, coverage expansion and independent artifact review remain required.
