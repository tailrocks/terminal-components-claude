# Terminal-tool qualification — incomplete fidelity, explicit limits

Pinned sources: tui-snap `5036cf87e621e6beb66deffe3224abdbefc955cb`, tui-test `2d5f8020ba6eb77b9c4da38892c2f6f200311b90`. All changes are external harness/evidence only. tui-snap source status contains pre-existing untracked `target` symlink; no tracked source changes. Binary and evidence hashes: provenance.json.

Reproduce: `rtk proxy python3 /Users/donbeave/Projects/terminal-components-integration-evidence/tool-qualification/run_all.py`. It launches each full tui-test session inside one Python process, unique name, finally closes. `rtk proxy python3 .../qualify.py --strict` intentionally exits 1: neither engine is fully qualified. No snapshot approval is created. Fixture oracle is hand-authored from explicit ANSI commands; expected-cells.json never copies a capture.

## Measured capabilities

Both PTY engines preserve isolated bold, DIM, reverse, RGB foreground/background, indexed 256 and ANSI16 colors, monochrome bold/underline, combining graphemes, wide lead glyph, zero-based cursor position, and strikethrough. Initial full 32x12 grid is compared including blank cells. Both click(4,3) emit SGR coordinates 5,4; drag endpoints 2,2→5,4 map to 3,3→6,5. Resize reaches fixture as 36x14. Style-only settling sees red→green→blue same-glyph updates; tuisnap wait_stable(300ms) returned final blue after 513ms. tui-test wait idle also captured final blue. Ten injected mutations (glyph, bold, DIM, reverse, one-cell coordinate across both outputs) were detected. A live tui-test missing-text assertion exited 1 as required.

No NO_COLOR is inherited by fixtures; TERM=xterm-256color, COLORTERM=truecolor recorded from children. Fixture contains explicit RGB/256/16 and monochrome regions; this qualifies parsing those encodings, not an application's capability selection.

Both fixture processes exited 0 and restored ECHO/ICANON. Exact termios equality differed by macOS PENDIN (536870912); original and resulting full values retained, not silently normalized. tui-test direct-run `wait exit` succeeds and state.exited=0; `expect exit-code 0` fails because it queries tracked shell-command status, absent for direct program run. Use state.exited for this workflow.

## Blocking fidelity defects / unsupported observations

- tui-snap PTY SGR 1;2;7 loses bold while retaining DIM/reverse. Full oracle exposes this at (6,1).
- tui-snap wide continuation (1,5) loses fg/bg (#010203/#040506 → default), although glyph widths/continuation metadata survive.
- Autowrap disabled, writing ABC at final column: tui-snap retains A; tui-test retains C (expected terminal overwrite). This divergence is not accepted as equivalent clipping.
- tui-snap DIM raster color calculation narrows before dividing: white on black resolves (25,25,25), expected (153,153,153) under its documented 60% rule. `snap-dim-resolution.txt` reproduces through public API. PNG manually inspected: DIM white F nearly disappears, consistent with bad value. PNG cannot be accepted as faithful until corrected.
- tui-snap canonical schema omits hidden/blink. HIDDEN H renders visibly in inspected PNG; unsuitable as a secret-masking proof.
- tui-test Alacritty SGR5 reports blink=false despite emitted blink. Width/continuation metadata absent in CLI cells; empty follower glyph and positioning survive but do not replace explicit width proof.
- tui-test CLI cursor reports x/y only; shape/visibility/blinking unavailable in captured state. tui-snap PTY reports correct visible, steady Bar cursor from CSI6q. This does not qualify tuisnap's separate raw-ANSI parser path.
- tuisnap paste converts LF to CR. Exact requested and received bytes retained; do not claim literal-byte paste preservation. tui-test raw `write` preserves explicit bracketed paste LF; that qualifies transport injection, not a dedicated high-level paste method (CLI has none).
- tuisnap exposes click/drag but no dedicated wheel method in public Session; tui-test scroll down amount2 emitted two button65 SGR events. No unsupported method invented.

Initial probes are preserved under initial-probes: first fixture assumed each read was one input, causing a tuisnap timeout when drag/paste/s coalesced. Corrected fixture accepts coalesced trailing control trigger. Historical NO_COLOR-contaminated captures outside this directory remain untouched.

## Scope limits

This is tooling proof, not app parity, approval, or terminal pixel equivalence. SVG from tui-test emitted, not independently visually inspected. Pure model-to-view tuisnap adapter and separate raw ANSI parser need their own qualification; fixes belong upstream or a explicitly reviewed pinned patch. No tool defect authorizes losing an application requirement. Full source/toolchain build evidence lives in parent's tool-build ledgers; this harness uses stable 1.98.1 isolated external target and locked Cargo graph after initial resolution.
