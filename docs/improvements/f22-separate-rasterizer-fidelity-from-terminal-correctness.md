# F22 — Separate rasterizer fidelity from terminal correctness

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete capture-tool grapheme/cell fidelity, clipping, shaping/font coverage and evidence labeling needed for Holla visual review. Preserve complex-text fixtures and compare claimed terminal behavior against actual terminal evidence.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

No unrelated emulator certification or terminal protocol expansion. An unavailable required font/platform check remains pending, never passed.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · confirmed verification-tool limitation.** `tools/ansi2png.py` iterates
code points with a heuristic width: e+acute totals 2 rather than application 1;
woman-technologist totals 5 rather than 2. This host's selected font renders
東/京/☕ as the same missing-glyph mask. Terminal and verification agents
independently reproduced width errors; this is not evidence that the Rust buffer
or a particular terminal fails identically.

Two more independently checked capture defects share this geometry boundary:
`tools/ansi2html.py:141/147` pads 東京 with two spaces at four columns because it
counts scalars; `ansi2png.py:78` does not clip to `cols`, so rendering ABCD at two
columns paints C and part of D into the right margin. A correct width table alone
would not fix allocation clipping or the separate HTML policy.

Keep ANSI/text/cursor and buffer/copy tests authoritative for their respective
layers; label current PNGs approximate for complex text. For normative PNG
review, make PNG and HTML consume one reference grapheme/cell representation;
use shaping and tested font fallback with
explicit font/version metadata, then compare against real terminal captures.
Do not replace Unicode fixtures to conceal failures. Risk: medium tooling and
platform reproducibility. Acceptance: split-style/combining/ZWJ/variation-selector/
CJK fixtures agree on occupied cells, cursor and selected content in both formats;
no glyph/background paints into padding. Font coverage is asserted, missing glyphs
reported. Actual emulator inspection remains required
for a claimed emulator compatibility result.

## Evidence

**Slice status:** current shared slice complete.

**One reference:** `examples/cells.rs` and `tools/cells.py` compute occupied cells with the library's unicode-width and segmentation; `tools/ansi2html.py` and `tools/ansi2png.py` consume that layout, clip to `cols`, never paint into the padding, shape with raqm when available, assert font coverage and report missing or unshaped glyphs in `<name>.png.fidelity.json` (exact or approximate, and why). `tools/fidelity_check.py` runs eight fixtures (split-style, combining, ZWJ, variation selector, CJK, clip, clip-wide, reverse selection) and passes on this host with JetBrainsMono NFM.

**Evidence labelling:** `tools/capture.sh` writes `<name>.manifest.json` beside every frame (source revision and dirty flag, binary sha256, arguments per capture session, geometry, colour environment, tmux/python/Pillow versions, fonts with digests, PNG fidelity summary, review verdict). Text and ANSI captures stay authoritative; PNGs are review aids at the stated fidelity.

**Inspected:** the fidelity fixture PNGs (all exact on this host), `shots/h_hp03_find_unicode` (`café`, ☕), `shots/h_hp04_preview_control`.

**Limit:** actual emulator inspection is still required for an emulator-compatibility claim; none is made.

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
