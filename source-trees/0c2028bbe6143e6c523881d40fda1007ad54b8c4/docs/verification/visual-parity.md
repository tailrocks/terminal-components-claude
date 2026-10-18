# Visual parity

Visual parity means exact equality with the trusted frame for the same sealed
fixture, event program, time sample, geometry, capability mode, and renderer
profile. The target is the output of the existing repository at
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, not a redesigned default theme.

The future `Termrock` theme and reusable components may have new internal
names. They must reproduce the baseline applications' rendered output and
observable visual states 1:1.

## Frame comparison

The comparator gates, in order, dimensions and then every row-major cell:

1. grapheme/symbol and display width;
2. wide-cell continuation semantics, including the continuation cell's
   placement and preserved style;
3. foreground and background colors;
4. every modifier preserved by the pinned tool;
5. cursor coordinates, visibility, and canonical cursor properties supported by
   the capture path.

The semantic observation at the same checkpoint additionally gates focus owner,
capture owner, active layer path, selected/current stable key, navigation key,
draft and committed values, typed action count, and typed action targets. A
frame digest is diagnostic only; it never replaces field-by-field comparison.

The approved grouped store contains `.ansi`, `.txt`, `.png`, and `.html` for
each scenario. Cell and semantic equality are authoritative. Pixel equality
means equality of decoded pixels under the same qualified raster profile; PNG
compression bytes are not themselves a UX comparison. An HTML or PNG report is
review evidence, not an approval source.

## Renderer and capability pins

The baseline toolkit revision is `tuiscotti` commit
`a47c9aaefb34e4c00026f99d8a8dd7ee5916b274` (`TUISCOTTI_SHA`), replacing legacy `tui-snap`
`2d43458ad2bc37d76653c22d56e61ee74512d893`. All approved visual checkpoints export
six native formats (`.ansi`, `.html`, `.png`, `.ascii`, `.txt`, `.frame.json`) plus
companion fidelity and loss sidecars from a single observation. Qualify old-tool/new-tool output
on an unchanged frame set and classify renderer/parser-only changes separately. Do not combine
a toolkit upgrade with a product approval.


Each applicable component and composed fixture exercises:

- truecolor;
- 256-color;
- 16-color;
- explicit `none`; and
- an actual `NO_COLOR` environment path (`nocolor`).

Record terminal profile, renderer/tool revision, font/raster profile, locale,
timezone, environment, fixture digest, and every artifact hash. A canonical
PNG is not a promise of pixel identity on an arbitrary terminal or font
installation.

## Geometry

Baseline composed scenes use `72x20`, `80x24`, `100x30`, `120x40`, and
`160x50`. Baseline-like minimum-size scenes also test `71/72/73` columns and
`19/20/21` rows, plus zero/tiny allocations, nonzero origins, exact fit,
one-cell-short allocation, long Unicode content, and narrow-to-wide recovery.
The `72x20` rule belongs to a composed baseline fixture; it is not a global
minimum for every component. Each component's plan adds local allocation
boundaries.

## Applicable visual states

The source-pack plans state applicability per component. Capture only states a
component can actually own, and record an explicit not-applicable reason for
the others. Never invent focus or pressed screenshots for decorative
components such as `Spinner` or `KeyHint`.

| Axis | Required applicable observations |
| --- | --- |
| Focus and hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer-motion restoration |
| Activation | pointer down, held press, release inside, release outside, removed/disabled target, keyboard activation, feedback and expiry |
| Controlled choice | chosen/checked versus keyboard cursor versus hover; caller accepts or rejects a proposed value |
| Editing | navigation, editing, selection, valid/invalid, read-only, commit/cancel/blur/tab, external revision conflict |
| Data | empty, loading, partial, ready, error, retry, stale source, insert/remove/reorder/filter |
| Scroll | no overflow, top/middle/bottom, horizontal/vertical, thumb drag, reading/tail, protected-row fade |
| Layers | closed/open, nested, outside click, Escape ladder, reanchor, owner removal, focus restoration |
| Motion | every unique phase and wrap, before/at/after deadlines, paused/reduced, completion/failure |

Mandatory combinations are included when meaningful: focus+hover;
editing+invalid; selected+disabled; nested modal+paste; drag+resize; and
source change while a press is held. Composite activation is attributed to its
actual child control; a dialog does not receive a fabricated pressed state.

## Baseline visual language

Verification preserves the baseline's existing semantic surfaces, restrained
accent use, text hierarchy, focus gutter, selected/current markers, hover
lifting, cursor rules, modal backdrop, focus trapping/restoration, scrollbar
and fade behavior, and narrow-width/capability behavior. It also preserves the
distinction between editing and navigation, disabled and read-only, and
empty/loading/partial/error/success states. The characterized activation
feedback boundary is 140 ms where the component advertises it. Keyboard input
suppresses stale hover until pointer motion resumes.

No verification case authorizes restyling or a product change. A theme or
component API rename is acceptable only when the rendered baseline remains
equal.

## Timing and motion cases

Time is supplied by a deterministic owner and recorded with the checkpoint.
The source plans require boundary samples rather than sleeps:

- Button-like activation feedback: `0`, `139`, `140`, and `141` ms. Keep the
  pointer-held frame separate from post-release feedback; release outside never
  activates.
- Spinner: all ten glyph phases, including `9 -> 0` wrap. The phase source's
  cadence is recorded by its owner; it is not a universal milliseconds-per-
  phase rule.
- Determinate progress: `0`, `1`, `50`, `99`, and `100`, exact rounding
  boundaries, semantic statuses, track lengths 5 and 6, and label-fit
  thresholds. Indeterminate progress enumerates every unique phase per track
  width.
- Meter: `0`, `59`, `60`, `84`, `85`, `100`, unknown, all semantic tones, and
  Line/Block modes. Unknown is not zero. Refreshing may share Spinner visuals
  while still accepting supplied data updates.

Full-motion, reduced-motion, and paused behavior remain distinct observations.
Do not normalize animation away to make a frame match.

## Component plan binding

Every one of the 45 source-pack plans is bound to a component contract under
[`../components/`](../components/), a source/oracle reference, an applicable
state matrix, and an exact comparison case. The 45 JSON plans use schema
`termrock-spec/capture-plan-v1`, pin the same baseline commit, require the five
dimensions and five capabilities above, and have no expected artifacts yet.
Their 222 case descriptions are the minimum required set; a newly discovered
applicable case increases the set. A missing case or artifact is a failure,
not an invitation to reduce the denominator.

Mutation gates and artifact authority are defined in
[`oracle-and-provenance.md`](oracle-and-provenance.md) and
[`conformance.md`](conformance.md).
