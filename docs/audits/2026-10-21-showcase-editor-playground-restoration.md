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

The frozen Find and Run buttons use plain labels. Their `.shortcut` values are
tooltip metadata whose empty text suppresses a tooltip; they do not display an
inline shortcut suffix. The page uses the shared button labels without a
page-local shortcut painter. Keyboard hints remain in the page footer.

The first real Showcase route test exposed an application publication bug:
after the editor and completion overlay published their active bindings, the
page called `Ui::publish_bindings` for Ctrl+R and replaced those tables. This
increment uses `Ui::publish_additional_bindings`, preserving the component's
Tab, Enter, completion, and focus bindings while adding Ctrl+R. No runtime or
shared keymap semantics changed. Before the fix, the two real-app Tab and
completion Enter probes both failed; after the fix both pass. The editor still
has a separate BackTab dedent gap, tracked by its shared editor owner.

The existing syntax highlighter and block segmenter stay demo data supplied to
the shared editor's callback contracts. The editor itself owns text editing,
cursor movement, block selection, completion UI, focus, and rendering.

## Verification

The audit packet `/private/tmp/editor-shell-routing-review-3807-20261021/review.json`
(SHA-256 `4863bf49cf97da624bc9f98eb660f829eba14fccf2d4bd313f4dbd10000c08f6`)
contains the unchanged real-app probes failing before this fix. After the fix,
all 41 `showcase-ui` app tests passed, including the real-route checks for
configured four-space Tab retaining editor focus, completion accepting both
Enter and Tab, the following Tab indenting, navigation-mode Tab traversal to
Find, and a Help overlay blocking page Ctrl+R. The Playground route still
verifies live find/Escape, second-block Ctrl+R diagnostics, and Run button/
countdown. All 17 `termrock-editors` completing-editor tests passed. `cargo fmt
--check` and scoped Clippy passed. This increment changed no shared editor or
runtime code. Options/States and BackTab dedent remain incomplete. Candidate
digest files were not edited or blessed.
