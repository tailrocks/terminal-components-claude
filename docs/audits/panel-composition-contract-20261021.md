# Panel composition contract — 2026-10-21

## Decision

Add `composition::panel(&Panel, Child) -> Child` as the first reusable Panel
adapter. `Panel` continues to own chrome, insets, and clipping. The existing
child component continues to own rendering, focus, hit regions, cursor, and
input behavior. The facade delegates the body to the existing composition
resolver and does not paint or register substitute controls.

`Panel::measure_with_body` measures the supplied body under constraints
reduced by the same insets used by `Panel::inner`. It adds body size and panel
chrome requirements before fitting the outer constraints. Existing
`Panel::measure` behavior remains unchanged; a one-cell body is covered as a
compatibility assertion.

The runtime exposes component-author `CompositionSlots::draw_at` for a
logical root that can be larger than the current visible `Ui` clip. The shared
resolver measures and reflows against logical bounds, then clips rendering,
registration, and interaction bounds to the inherited visible region.

## Coverage

- Body-aware measurement: Card and Framed panels, title/meta/badge width,
  Unicode cell width, outer tight constraints, zero constraints, and default
  measurement compatibility.
- Runtime logical-root clipping: width-dependent layout uses the full logical
  bounds while the leaf receives the intersection as its visible clip.
- Public facade consumer: live Button and CodeEditor, pointer activation,
  keyboard activation and traversal, Unicode editing before and after resize,
  and rejection of an 8,000-deep descendant before child draw. All behavior
  uses existing child component APIs.
- Focused combined `cargo nextest` run passed 366/366 tests across runtime,
  the Panel badge integration test, and public facade Panel consumer; zero
  skips.
- Strict affected-package Clippy passed for `termrock-controls`,
  `termrock-runtime`, and `termrock` across all targets. Scoped rustfmt and
  `git diff --check` passed.

## Limits

This increment adds no named Panel header/footer slots and no application
adoption. Accounts still needs measured footer composition and mixed-content
scrolling. It does not prove application parity or remove any replay path.
