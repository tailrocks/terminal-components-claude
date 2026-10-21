# Tree composition adapter

Source base: `48c5fc33b81626925d67facb278c7656e625b887`.

This increment adds a borrowed public composition leaf for the existing
`termrock_navigation::Tree`. It delegates measurement to `Tree::measure` and
drawing to `Tree::draw`, carrying the caller's `TreeState` and item slice by
reference. It creates no input policy, state owner, row renderer, hit map, or
cursor path. The documented Manager contract at
`/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/manager-shared-contract-671923ea2-20261021/contract.json`
remains the API source; this change does not migrate Manager or remove its
private painter.

The facade-only integration test compares direct and composed complete
terminal buffers, registered regions, focus, cursor position, expansion and
activation actions across truecolor, 256-color, 16-color, and monochrome
profiles. It also checks Unicode labels, reordered sibling keys, narrow,
zero-sized, and offset regions. Measurement is checked for unchanged tree
state. The consumer imports only the public `termrock` facade.

Focused check: `cargo nextest run -p termrock --test tree_composition_consumer --features testing` — 2 passed, 0 skipped.

Product acceptance remains HOLD. The Manager composition, frozen application
flows, resize/input ownership across the runtime, full workspace gates, and
paired application comparisons remain unverified by this adapter test.
