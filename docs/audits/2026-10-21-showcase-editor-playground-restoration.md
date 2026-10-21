# Showcase Editor Playground restoration

Status: implementation in progress; page parity remains incomplete. Frozen
Options and States contracts have now been mapped; their restoration is blocked
on shared composition/specimen APIs recorded below.

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

## Frozen Options and States contract map

The frozen section order is `Playground`, `Options`, `States`. The tab strip
supports pointer activation, Left/Right movement, and direct `1–3` selection;
the section caption is respectively “the live widget”, “one control per
option”, or “every state, drawn by the widget”. The section container owns a
scrolling body. Options are live controls grouped under “Page options”; changing
one updates `PageValues` and rebuilds the actual `CodeEditor` and completion
configuration while preserving text, cursor, and diagnostics.

The frozen editor Options matrix contains 22 controls, in this order:

| Option | Values | Default |
| --- | --- | --- |
| Read-only | off, on | off |
| Edit entry | theme, immediate, type-to-edit, explicit | theme |
| Tab leaves the editor | off, on | off |
| Esc reverts | off, on | off |
| Field keys | theme, emacs, desktop | theme |
| Text key map | preset, custom | preset |
| Reading keys | default, vim, no-page | default |
| Command keys | default, custom | default |
| Indent | 4, 2 | 4 |
| Syntax tones | off, on | on |
| Blocks | off, on | on |
| Document | sample, empty | sample |
| Horizontal scroll | theme, on, off | theme |
| Scrollbar gap | 1, 0, 2 | 1 |
| Track click | page, jump | page |
| Completion rows | 8, 4 | 8 |
| Completion width | default, wide | default |
| Completion placement | below, above | below |
| Completion fit | default, flush | default |
| Completion cursor | fill, bar | fill |
| Completion dismiss | light, escape | light |
| Completion keys | default, vim | default |

The frozen States matrix is an inert `StateGrid` with rows `Rest`, `Focused`,
`Editing`, `Disabled`, `Busy`, `Error`, `Empty`, `Overflow`; columns `Tones`,
`Plain`, `Warning`; the warning specimen owns a diagnostic. A “Compare levels”
toggle switches to a color-level comparison. The focus legend describes the
specimen focus mark and does not register any specimen as a live control.

The current public composition API can place Tabs, panels, props lists, and
code editors, but has no Select leaf adapter. Recreating the full Options
matrix with direct per-row Rect layout would violate the consumer composition
boundary. The old StateGrid/SpecimenState renderer also has no public Termrock
equivalent. Repainting a state matrix in Showcase or registering invisible
controls would violate the widget-ownership boundary. Concrete shared API
proposals for the required next owner are (1) `composition::select(component,
state, items)` delegating measure/draw to `Select`, and (2) an inert shared
specimen grid whose widget adapter delegates the same widget renderer with an
explicit visual `StateFlags`/specimen state and performs no input/focus
registration. The parent coordinator has been notified; no shared crate was
edited by this worktree.

The frozen Find and Run buttons use plain labels. Their `.shortcut` values are
tooltip metadata whose empty text suppresses a tooltip; they do not display an
inline shortcut suffix. The page uses the shared button labels without a
page-local shortcut painter. Keyboard hints remain in the page footer.

The first real Showcase route test exposed an application publication bug:
after the editor and completion overlay published their active bindings, the
page called `Ui::publish_bindings` for Ctrl+R and replaced those tables. This
increment uses `Ui::publish_additional_bindings`, preserving the component's
Tab, Enter, completion, and focus bindings while adding Ctrl+R. No runtime or
shared keymap semantics changed. The earlier audit overstated the original Tab
probe: its focus assertion passed, but its literal `col 5` assertion was stale.
From cursor column 2, four-space indentation correctly reaches column 6. The
independent corrected semantic probe expects column 6 and passes; the unchanged
original probe does not pass as written. Completion Enter did fail before the
binding fix and passes after it. The editor still has a separate BackTab
dedent gap, tracked by its shared editor owner.

Independent review of the running route found three more application-owned
state defects, fixed here without changing the shared editor: pointer and
other non-tick updates consumed run countdown steps; finishing a run diagnosed
the block under the cursor at finish time instead of the block submitted at
start; and an empty manual completion list used source order rather than the
frozen fuzzy-score/label ordering. Countdown advances only on `UpdateCause::Tick`,
the submitted range is retained until the run completes, and all completion
results sort by fuzzy penalty then label while retaining stable `ItemKey`s.
The actual-route tests cover pointer movement during a run, moving from the
submitted second block to the first before completion, and accepting the first
alphabetical empty-query result. Provider unit tests also cover stable keys and
label ordering for tied non-empty fuzzy scores.

The independent follow-up review packet
`/private/tmp/showcase-binding-independent-f4a4-20261021/review.json` (SHA-256
`e4a263e6a336f6ee21c5d422f7c8c7430a6d070cb9a02b24c976af8ffe792044`)
reproduced two additional existing frozen-contract differences. Re-running the
same unchanged block now appends its warning again, and text edits continue to
clear the accumulated diagnostics. The state block readout checks exact cursor
containment so the blank separator says “between blocks”. The shared
`CodeEditor::current_block` fallback to the preceding block remains unchanged
for execution; a first review probe that assumed that behavior was a defect
failed its blank-separator setup and was rejected. The separate readout probe
isolates and verifies the actual difference.

The existing syntax highlighter and block segmenter stay demo data supplied to
the shared editor's callback contracts. The editor itself owns text editing,
cursor movement, block selection, completion UI, focus, and rendering.

## Verification

The audit packet `/private/tmp/editor-shell-routing-review-3807-20261021/review.json`
(SHA-256 `4863bf49cf97da624bc9f98eb660f829eba14fccf2d4bd313f4dbd10000c08f6`)
contains the real-app probes and recorded pre-fix failures. Validation for this
increment: `cargo nextest run -p showcase-ui --test app_tests --no-capture`
passed 44/44; `cargo nextest run -p showcase-demos --lib --no-capture` passed
50/50; `cargo nextest run -p termrock-editors --test completing_code_editor
--no-capture` passed 17/17; `cargo fmt --check` passed; and scoped `cargo clippy
-p showcase-demos -p showcase-ui --all-targets -- -D warnings` passed. The
Validation for this follow-up: `cargo nextest run -p showcase-ui --test
app_tests --no-capture` passed 46/46; `cargo nextest run -p showcase-demos
--lib --no-capture` passed 50/50; `cargo nextest run -p termrock-editors --test
completing_code_editor --no-capture` passed 17/17; `cargo fmt --check` passed;
and scoped `cargo clippy -p showcase-demos -p showcase-ui --all-targets --
-D warnings` passed. The Playground route also covers live find/Escape,
second-block Ctrl+R diagnostics, and Run button/countdown. No shared editor or
runtime code changed. Options/States and BackTab dedent remain incomplete.
Candidate digest files were not edited or blessed.
