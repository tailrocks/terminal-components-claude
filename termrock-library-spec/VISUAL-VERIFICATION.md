# Exact visual and interaction verification

## Authority first

Pin the visual source to `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Preserve its existing `snapshots/` files and original tui-snap pin `2d43458ad2bc37d76653c22d56e61ee74512d893` as an immutable historical corpus. Do not run a candidate-driven accept/bless command against it.

The newer toolkit pin `9dc86daff1dcbf20805b145916e8f04e9515f929` is a qualification candidate, not a reason to overwrite old output. Compare old-tool/new-tool output on a fixed unchanged frame set and classify renderer/parser-only differences separately. Product changes and tool changes never share one bulk approval.

## Capture layers

**Component fixture:** instantiate the actual baseline component or its unchanged app-local composition through a trusted adapter; use neutral data or copied synthetic oracle labels. Capture all applicable states with a deterministic event/time program.

**Generic composed scene:** a test-only consumer uses only new Termrock public APIs to compose list/detail, editor/inspector, nested forms/pickers, progress/output and tabbed pane layouts. Existing Jackin source is a visual reference; no new Jackin route reducers, services, lifecycle simulation or launch engine is implemented.

**PTY laboratory:** run a tiny generic component lab to exercise key decoding, mouse down/up/drag, bracketed paste, resize and terminal cleanup. The conformance crate owns the PTY/toolkit dependencies. Components do not spawn processes.

Pure captures prove exact component frames and state transitions. PTY captures prove the executable adapter sees and routes actual terminal input. Neither a PNG-only test nor a headless-only test replaces the other.

## Oracle adapters and provenance

Keep original source files immutable. Prefer public constructors and pure state setup. When app-local source is not externally accessible, use an isolated trusted capture workspace with an explicitly reviewed adapter patch that exposes fixture/state/clock entry points only. Record the patch hash and diff; no paint, layout, event-handler or fixture-result changes are permitted. This is oracle instrumentation, not a new product implementation.

Record repository/commit, source paths and available Git blob IDs, adapter digest, toolchain/lockfile digests, input program, fixture revision, time samples, dimensions, capability/motion environment, renderer/profile/font digests, output artifact hashes and approval receipt. This pack distributes no font files and no new captures.

Resolve pointer target descriptions into absolute numeric events **against the trusted oracle** and then seal them. The candidate cannot resolve its own hitboxes and call that the same gesture: that would hide layout drift. Semantic-target tests are valuable as a separate interaction lane, not a substitute for sealed coordinate parity.

## Exact comparator

Gate dimensions, cell symbols, wide-cell continuation semantics, foreground/background, preserved modifiers, cursor coordinates/visibility and canonical supported cursor properties. Also compare focus owner, captured owner, active layer path, selected/cursor stable keys, draft/commit observations and typed action count/targets at declared checkpoints.

Use the pinned grouped store for `.ansi`, `.txt`, `.png` and `.html`. Preserve canonical cursor metadata in a separate test observation where a grouped artifact alone is insufficient. Exact pixel comparison means equality of decoded pixels with an identical qualified raster profile, not approximate SSIM or a broad mismatch threshold. Do not conflate PNG compression bytes with pixel identity.

The inspected tui-snap schema freezes/normalizes some terminal attributes, including blink and some underline/overline data. Record exactly what its version preserves. Test dropped attributes through direct model/protocol assertions; never report them as screenshot-proven. Canonical PNG rendering is not a guarantee of pixel identity with every user's terminal/font system.

## Mandatory state model

| Axis | Cases |
|---|---|
| Common geometry | Normal, zero/tiny allocation, nonzero origin, exact fit, one-cell-short, long Unicode text, narrow-to-wide recovery |
| Focus/hover, where interactive | Unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, restored hover |
| Gesture, where activating | Down, held press, release-inside, release-outside, removed/disabled target, activation feedback and expiry |
| Controlled choice | Chosen/checked versus cursor versus hover; caller accepts or rejects change |
| Editing, where supported | Navigation, editing, selection, valid/invalid, read-only, commit/cancel/blur/tab, external revision conflict |
| Data | Empty, loading, partial, ready, error, retry, stale source, dynamic insert/remove/reorder/filter |
| Scroll | No overflow, top/middle/bottom, horizontal/vertical, thumb drag, protected-row edge fade, reading/tail |
| Layers | Closed/open, nested, outside, Escape ladder, reanchor, owner removed, focus restoration |
| Motion | Every unique phase, phase wrap, before/at/after deadlines, paused/reduced, state completion/failure |

Do not assign hover/click/pressed to a decorative Spinner or invent an editing state for KeyHint. Non-applicability is explicit with a reason. Composite activation styles are attributed to their real child controls, not a fake pressed style across the entire dialog.

For each advertised visual variant, every individually applicable state must be covered. Record mandatory combinations such as focus+hover, editing+invalid, selected+disabled, nested-modal+paste, drag+resize and source-change+held-press. A coverage reducer may avoid logically impossible combinations, but may not omit a state, variant or real interaction edge without an explicit reviewed disposition.

## Dimensions and capability

Use baseline composed sizes 72x20,80x24,100x30,120x40,160x50, plus local component-size boundary fixtures. At minimum test 71/72/73 columns and19/20/21 rows for the baseline-like minimum-size scene. Individual widgets have no global72x20 minimum.

Exercise truecolor,256,16, explicit none and NO_COLOR environment paths. Use fixed locales, input fixtures and time; do not read home-directory accounts or live provider/network data. Full-motion/reduced/paused differences are recorded rather than silently normalized away.

## Animation examples

Spinner uses all10 glyph phases and wrap9->0. A tick is an explicit phase sample; its source function does not establish a universal milliseconds-per-phase. Capture the supplying owner cadence separately.

Button activation feedback tests include0,139,140,141ms under the characterized feedback clock. Capture pointer-held state separately from post-release feedback. A release-outside must have no activation even if held press was visibly drawn.

Determinate progress tests include0,1,50,99,100 percent, exact rounding boundaries and all semantic statuses. Track lengths5 and6 and label-fit thresholds must be separate cases. Indeterminate sequences enumerate every unique phase for each tested track width; include status change mid-cycle.

Meter covers0,59,60,84,85,100 and unknown, all semantic tones and Line/Block modes. Unknown is not0. Refreshing uses the shared Spinner but still permits new supplied data to update.

## Required negative tests

A one-cell glyph/color/modifier change fails. A hidden/moved cursor fails its observation gate. An incorrect action target or duplicate activation fails even when pictures match. A deleted required case and missing expected artifact fail. A wrong oracle SHA or renderer digest fails. A standard component replaced with a dead call plus custom paint fails mutation/ownership tests.

Disabled controls must block the wrong underlying target, not allow click-through. A stale key after reorder must not activate the new occupant. A secret marker must not appear anywhere in text artifacts, debug traces or a report. Draw-state mutation must fail repeatability tests.

## Capture-plan status

The45 files under `capture-plans/` are plans with explicit source roots and case requirements. They contain no expected hashes, approvals or captured image data. Before execution, turn each required baseline case into an ExistingOracle or ExtractedOracle entry and bind it to reviewed immutable artifacts. Robustness/adapter/theme extensions are recorded separately. No plan counts as a passing test.
