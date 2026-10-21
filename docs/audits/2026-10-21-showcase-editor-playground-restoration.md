# Showcase Editor Playground restoration

Status: implementation in progress; page parity remains incomplete.

Review basis: frozen `visual-baseline` source commit
`4169783ae883788186d161af09ba9c481bd84e5c` and design reference
`f4867e9b5f8a08a3bb61ce80d644198a29f51984`. The source review packet is
`/private/tmp/termrock-audit-packets/showcase-editor-replacement-review-20261021.json`
(SHA-256 `0700787bda138011faf40c631d42c77fd6ed8c84ebb0dea1394dafd258e29f44`).

The replay renderer and fixed row include were already removed from the
candidate. This increment restores the frozen Playground's semantic data and
interactions through the shared `CodeEditor`, `CompletingCodeEditor`, `Button`,
`PropsList`, `Panel`, and measured composition APIs. It restores the `gzip(true)`
sample, four-space Tab indentation, the actual completion layer, live find
query/match data, running block state, completion count, cursor/block details,
run count/time, and diagnostics produced by the selected block. The page uses
the public `Group` and component composition adapters for its layout; no
application repaint or captured cell table is part of this implementation.

The frozen `Options` and `States` sections remain missing. Their editor option
matrix and inert state specimens remain required follow-up work. This
increment does not claim either section, full Showcase capability coverage, or
1:1 visual and functional parity.

The current shared `Button` API has no shortcut-label builder or separate
shortcut slot. The Playground can activate Find and Run through the shared
button and editor actions, but the frozen inline shortcut presentation needs a
separate shared Button contract review before it can be called identical. Keep
this gap visible; do not draw a page-local shortcut or use a captured frame.

At the real Showcase page route, current shell Tab routing advances focus out
of the editor before its configured `TabBehavior::Indent` runs, and the open
completion overlay's Enter is currently delivered as an editor newline. The
shared editor-completion integration test proves the standalone composite
contract, but does not prove the page-in-shell path. Those two host integration
defects need a shared runtime/focus owner and an additional cross-layer fix;
this page increment does not bypass them or claim the flows complete.

The existing syntax highlighter and block segmenter stay demo data supplied to
the shared editor's callback contracts. The editor itself owns text editing,
cursor movement, block selection, completion UI, focus, and rendering.

## Verification

Passed `cargo fmt --check` and focused real-app nextest:
`cargo nextest run -p showcase-ui --test app_tests
editor_playground_keeps_live_find_completion_and_run_contracts --no-capture`
(one test passed). The new app test exercises the real Editor route, actual
find query/matches, visible live completion popup and Escape dismissal, shortcut
run of the second code block with diagnostics, and Run button/countdown. Existing
`termrock-editors` completion integration tests cover standalone Tab acceptance
and the following configured Tab indentation. This focused app test does not
resolve the host Tab and Enter findings above. Scoped Clippy remains pending.
Candidate digest files were not edited or blessed.
