# Text, editor, grid, Diff, and viewport verification plan

Date: 2026-09-10. Scope: current `holla-fable` worktree only. No other branch/history inspected. This follow-up is planning only: no widget/source changes made. Existing primitives are sufficient foundations; proposed work below strengthens their ownership and geometry contracts, not a new widget inventory.

## Evidence and limits

`rtk cargo test --lib` passed all **132** current library tests. `rtk cargo build --lib` and `rtk cargo build --release --lib` passed. Public-API probes were compiled against these current artifacts. They rendered Ratatui buffers, inspected selected/copied text and pending grid changes, and measured a limited release workload. No external database or terminal mutation was used for the probes.

Temporary probe sources remain at `/tmp/tui-text-plan.KRJCvQ/`: `probe.rs` (viewport/text), `grid.rs` (paste/read-only), `modifiers.rs`, `perf.rs`, and `modal.rs` (actual TablePro modules plus TestBackend). These are **not retained repository regressions**. Exact flows and observed values are recorded below so the evidence does not depend on those temporary paths surviving. Future implementation must turn each accepted defect into a retained regression before fixing it.

Cross-review: the interaction auditor independently read, recompiled, and reran `probe.rs` and `grid.rs`, confirming TXT-01 through TXT-06 and the passing resize case. I independently rebuilt a separate TablePro harness and confirmed that auditor's modal-paste finding; its owner is [plan-interaction-verification.md](plan-interaction-verification.md), INT-01. This document does not duplicate the modal-routing work package.

No exhaustive claim is made about all modifiers, all Unicode scripts, all terminal implementations, or all public-state combinations. Current tests and temporary probes establish only their stated cases. A full live-stream performance budget and canonical-normalization/case-fold policy remain design decisions.

## Revalidation of completed fixes

These current retained tests passed in the 132-test run. Do not reopen them as missing capabilities; extend their coverage where the findings below identify a distinct boundary.

| Existing claim | Current retained evidence | What this proves / does not prove |
| --- | --- | --- |
| Ctrl+Enter no longer panics in CodeEditor | `src/widgets/code.rs:1041`, `modified_enter_commits_without_changing_document` | Ctrl+Enter commits, leaves editing, preserves document. Retained test does not enumerate every modifier |
| Find maps expanded lowercase back to source graphemes | `code.rs:1056`; `src/ui/text.rs:375`, `:383`, `:389`, `:399` | `İé` find `é`, combining ranges, Greek contextual lowercase, scalar rather than UTF-8-byte fuzzy matching, original offsets, and ASCII ranking pass. Full Unicode case folding/canonical equivalence is not promised |
| Find owns paste and grapheme Backspace | `code.rs:1065` | Editing, navigation, and read-only document modes preserve document while updating query |
| Replacing a document refreshes matches | `code.rs:1080` | Existing query no longer retains matches from the replaced document |
| Manual scrolling survives redraw; movement/resize reveal cursor | `code.rs:1089`, `:1105`, `:1148`; `src/widgets/textarea.rs:463`, `:485` | Tested vertical/horizontal cases and resize pass; offscreen CodeEditor cursor anchor returns none |
| CodeEditor read-only transitions block edits and keep navigation focus | `code.rs:1115`, `:1136` | Monochrome gutter survives; setting read-only during editing blocks Tab/BackTab/character mutation. This fix does not cover DataGrid's separate edit state, TXT-06 |
| Fields, editor, textarea and grid preserve tested wide-grapheme cursor/mouse geometry | `src/widgets/input.rs:517`, `:532`, `:547`, `:576`; `textarea.rs:438`, `:499`; `src/widgets/grid.rs:2122`; `code.rs:1160` | Masked/scrolled fields, clipped wide graphemes and nonzero-origin containment pass. Literal-tab semantics are distinct, TXT-02 |
| TextBuffer entry points preserve line modes and grapheme boundaries | `src/core/text.rs:608`–`:692` test symbols | Arbitrary byte boundaries, joined clusters, word movement, CRLF/CR normalization and single-line newline removal pass. Other controls remain stored; see TXT-02 |
| Diff review grapheme emphasis and column budget are corrected | `src/widgets/diff.rs:674`, `:694`, `:732` | ZWJ/combining highlights, scrollbar-dependent fallback, resize recovery, four-space tabs, aligned paired/unpaired rows, copied displayed tab text pass. Arbitrary caller-created viewport spans remain unsafe, TXT-01 |

Temporary modifier observation, independently separate from the retained Ctrl+Enter test: Enter and Shift+Enter inserted `\n` and kept editing; Ctrl+Enter, Alt+Enter, and Ctrl+Shift+Enter committed unchanged text. Thus the existing broad wording “modified Enter” is imprecise for Shift alone. Agree explicit modifier policy in the interaction plan before broadening tests/docs; do not infer all modified Enter combinations behave identically.

Passing negative check: a wrapped viewport containing `abcdefghij`, width 6/height 3, mouse-selected from `(1,0)` to `(3,1)`, copied `bcdefghi`; resizing to width 4/height 4 preserved the same selection. Logical selection already survives this visual reflow. There is no evidence here for replacing the logical-coordinate model.

## Verified open findings

### TXT-01 — styled span boundaries can corrupt a grapheme

**Classification:** confirmed defect; architecture/API weakness. **Priority:** P1, because copied text can lose a stored combining mark.

**Current evidence:** `src/widgets/viewport.rs:305` (`ensure_layout`), especially the per-span segmentation at `:320` and zero-width discard at `:343`; `selected_text` at `:435` copies the resulting cells. `Line` and `Span` accept independent strings without any grapheme-boundary contract.

**Reproduction:** render one logical line as either one span `👩‍💻X` or three spans `👩‍`, bold `💻`, `X`. The joined form places `X` at column 2; split form places it at column 4. For `a` + bold combining acute `\u{301}` + `X`, rendering and copying produce `aX`; one span `a\u{301}X` copies the accent correctly.

**Root cause:** style boundaries incorrectly act as segmentation boundaries. A standalone combining suffix has zero width and is discarded; a split ZWJ sequence gets measured as separate visible cells. Fixing Diff's own span construction did not remove this enabling condition from the generic viewport.

**Proposed primitive:** a logical-line grapheme iterator with source byte ranges and style resolution across run boundaries. Segment concatenated logical text, then map each complete grapheme to one terminal cell/style. Specify a deterministic style precedence for a cluster crossing spans, such as the style owning its first base scalar; do not manufacture partial terminal graphemes. Reuse the resulting mapping for render, selection, caret, copy, and width accounting.

**Acceptance:** retained tests for split/unsplit combining, ZWJ emoji, variation selectors and regional-indicator pairs yield identical text/width/selection; only the documented cluster style precedence may differ. Include a width-one clipping case and nonzero pane origin. Existing Diff geometry and copy tests stay green.

**Risk:** style precedence at a cluster boundary is a real compatibility decision. Selection/copy should retain the complete original grapheme, not silently replace it. Avoid rebuilding joined text on every idle frame; integrate with TXT-05/TXT-07.

### TXT-02 — tab and control display policy differs across text primitives

**Classification:** design inconsistency; architecture/API weakness. **Priority:** P2.

**Current evidence:** `src/core/text.rs:12` normalizes only CR/LF; `pos_of`/`offset_at` at `:486`/`:498` use `UnicodeWidthStr`; TextInput display runs at `src/widgets/input.rs:127`; editor/textarea draw graphemes directly. `src/ui/text.rs:9` defines four-space tabs used by TextViewport and Diff review only.

**Reproduction:** paste `A\tB` into editing TextInput, TextArea, and CodeEditor. All store the literal tab and report logical column 3 at the end; `B` is two cells after `A`, leaving a one-cell gap. TextViewport renders `A\tB` with `B` at column 5. Diff now follows the viewport's four-space policy.

`TextBuffer::single("A\tB\u{1b}[31mC\u{7}D\r\nE")` stores `A\tB\u{1b}[31mC\u{7}DE`; multi-line stores the same controls with `\nE`. CR/LF behavior agrees with current docs. Escape/BEL and literal tabs have no explicit product display/copy policy. Locally installed Ratatui `Buffer::set_stringn` filters graphemes containing control characters before writing cells, so this evidence **does not establish terminal escape injection**; invisible stored controls and divergent width behavior are the demonstrated concern.

**Proposed primitive:** an explicit text-display policy feeding the shared grapheme/source-offset map: tab expansion width, treatment of other controls, and copy-as-source versus copy-as-displayed semantics. Keep storage normalization separate from display. Existing four-space viewport behavior supplies a default candidate, but CodeEditor's configurable indentation remains a separate operation; do not conflate a tab key inserting indentation with a literal tab in a document.

**Acceptance:** one literal-tab/control fixture across input, textarea, code, grid editor, viewport, unified/review Diff proves consistent declared geometry, click-to-offset, cursor, clipping and copy. Tests distinguish stored source from rendered symbols. Cover CRLF, lone CR, LF, tab, ESC, BEL and combining text following them. Decide control visibility before implementation; do not silently delete document data.

**Risk:** changing text-buffer column semantics affects public cursor APIs. Add policy/mapping APIs with compatibility defaults before migrating consumers; avoid a widget-specific replacement that leaves other geometry paths inconsistent.

### TXT-03 — bounded retention is bypassed by supported population paths

**Classification:** confirmed defect; architecture/API weakness. **Priority:** P1 for the claimed bounded-output contract.

**Current evidence:** `src/widgets/viewport.rs:166` sets the limit; only `push` at `:171` applies it. `with_lines` at `:155` and `set_lines` at `:189` can retain more than the limit. `replace_last` at `:196` can append when empty without checking a zero limit.

**Reproduction:** `TextViewport::new(id).max_lines(3); set_lines(eight_lines)` has length 8. `TextViewport::with_lines(id, eight_lines).max_lines(3)` also has length 8. These are supported public methods, not arbitrary memory corruption.

**Real-app reachability:** `src/bin/holla/screens/activity.rs:45` configures 4,000 retained lines but `:104` feeds `set_lines`; `src/bin/holla/screens/plan.rs:954` configures 2,000 but `:1022` also feeds `set_lines`. Actual current demo scripts need not reach those limits for the bypass to be real; the configured invariant is ineffective on their chosen population path.

**Proposed primitive:** one retention-aware document mutation boundary, used by construction, replacement, append, live-tail replacement and limit changes. Decide whether retention is logical lines, bytes, or both. Apply trimming once and return the removed-prefix extent so selection/caret/scroll rebasing is atomic (TXT-04).

**Acceptance:** every supported text-population path and changing the limit after population preserves the cap; verify limits 0/1/N, oversized batch replacement and append. Holla activity/plan integration tests exceed their configured limits and retain the correct tail. Document whether a zero cap is allowed and whether the cap bounds bytes as well as line count.

**Risk:** dropping lines on `set_lines` changes visible content and line identities; pair with TXT-04 and expose enough outcome information to owners. A line-count cap alone cannot bound a single arbitrarily long line; do not claim a byte bound without implementing one.

### TXT-04 — retention eviction retargets selection, drag anchor and scroll context

**Classification:** confirmed defect. **Priority:** P1 for wrong-source copy; P2 for viewport movement.

**Current evidence:** `TextViewport::push`, `src/widgets/viewport.rs:171`, saturates selection/caret indices after prefix removal, does not rebase `drag_anchor`, and leaves the manual scroll offset unchanged. `clamp_positions` at `:221` only clamps some line indices. Drag uses its stored logical anchor at `:516`.

**Reproduction with cap 3 and lines A/B/C:**

- Select A, append D and redraw: `selected_text` changes from `Some("A")` to `Some("B")`, although B was never selected.
- Press at the start of B, append D, redraw, then drag to column 1 of the now-first B row: copied selection becomes `Some("\n")`; the retained B anchor was not rebased.
- With follow off and a one-row viewport manually showing B, append D: the top becomes C. The retained reading position B still exists, but the visual offset did not follow its logical identity.

**Proposed primitive:** a document-edit delta or stable logical-line identity applied atomically to selection endpoints, active drag anchor, caret and visible anchor. Clear a selection whose entire source was evicted; clamp a partially surviving selection according to an explicit policy. Preserve the first visible retained logical position while follow is off. Reflow the rebased position into visual rows after wrapping/resize.

**Acceptance:** the three repros above; partial/multiline selected eviction; active drag during append; wrapped lines; follow-on behavior; empty/zero retention. Selection must never silently retarget unrelated text. Retained reading position stays stable unless it was evicted. The already-passing `bcdefghi` resize selection case remains intact.

**Risk:** display-row offsets alone cannot express stable anchors under wrap. Preserve the existing logical `CellPos` concept; add mutation identity/deltas rather than replacing it with visual coordinates.

### TXT-05 — public content mutation bypasses the viewport cache

**Classification:** confirmed defect; architecture/API weakness. **Priority:** P2.

**Current evidence:** `TextViewport.lines` is public at `src/widgets/viewport.rs:111`; `set_area` at `:245` only rebuilds for private `dirty`/layout keys.

**Reproduction:** render a one-line viewport containing `OLD`, assign `view.lines[0][0].text = "NEW"`, then render at the same size. The public model reads `NEW`, buffer still displays `OLD`. Source and render disagree without any unsupported operation documented on the public field.

**Proposed primitive:** explicit content ownership and revisioned mutation. Prefer read-only line access plus mutation methods/guards that invalidate affected lines and apply retention/deltas; plan a compatibility migration for public fields. If fields remain mutable, every cache key must include a reliable content revision the caller is required and able to update, or mutation must be detected. Rehashing all content each frame is not a measured solution to TXT-07.

**Acceptance:** every public mutation route updates render, selection/copy and caret on the next frame; unchanged renders preserve cache; tests cover changing text without changing line/byte count. Inventory must identify callers before tightening visibility.

**Risk:** making fields private is an API break. Coordinate with the API inventory rather than patching individual callers around stale caches.

### TXT-06 — DataGrid can commit after editability is revoked

**Classification:** confirmed defect. **Priority:** P1.

**Current evidence:** `DataGrid::begin_edit`, `src/widgets/grid.rs:539`, checks `editable`; `on_key` at `:924` dispatches an existing editor first; `on_paste` at `:1390` and `commit_edit` at `:590` do not recheck it. The analogous CodeEditor state transition is already fixed and tested.

**Reproduction:** one text cell `old`; `begin_edit`; select all draft text; set `grid.editable = false`; paste replacement; `commit_edit`. Result remains `Some(CellChanged { row: 0, col: 0 })`, and `pending.value(0,0)` contains the replacement despite editability being revoked. The standalone probe used `A\tB\r\nC\tD` and produced `Text("A\tBC\tD")`; the same bypass does not require tabs.

**Proposed primitive:** centralize the edit-session validity transition before all document mutations, including explicit commit calls, keyboard, paste, click-induced commit and data/schema changes. Define whether revocation cancels the draft or preserves it without permitting commit; do not quietly commit it as part of revocation. Extend the existing edit-state machinery rather than adding an alternate editor.

**Acceptance:** revoke grid editability and individual-column editability during editing; keyboard/paste/Enter/Tab/direct commit cannot create new pending changes. Existing pending changes remain intact. Test sort, focus changes and invalid-draft behavior around revocation after choosing draft preservation semantics.

**Risk:** transaction-like state changes can affect unsaved draft ownership. Explicit policy required, but the current ability to create a write while disabled is not an acceptable default.

### TXT-07 — one live-line update rebuilds the retained history

**Classification:** measured architecture/performance weakness. **Priority:** P2; no target frame budget was specified.

**Current evidence:** `replace_last` marks the whole viewport dirty (`src/widgets/viewport.rs:196`); `ensure_layout` at `:305` recreates every cell and visual row. A dirty overflowing viewport first computes full width then reduced scrollbar width (`set_area`, `:245`), so the whole history may be rebuilt twice. Existing idle-cache tests prove idle behavior only.

**Release probe:** macOS arm64, Rust 1.98.0, current release library, standalone `rustc -O -C lto=thin`; 80 ASCII characters per logical row, unwrapped viewport 80×20. Each sample performs 20 `replace_last(80 chars)` + `set_area` operations after warm layout. Measured elapsed totals:

| Retained rows | 20 replacement requests | Approximate per request |
| --- | ---: | ---: |
| 1,000 | 89,308 µs | 4.47 ms |
| 10,000 | 688,107 µs | 34.41 ms |
| 50,000 | 3,449,885 µs | 172.49 ms |

Twenty unchanged `set_area` calls reported less than 1 µs at the timer's output resolution. Do not derive a speedup ratio from that value. These are single samples on a shared development machine, not terminal frame times, a benchmark baseline, or before/after claims. Source inspection explains the observed full-history scaling.

Peer review clarified the workload: only the first replacement changes `x` to `y`; the remaining requests resubmit identical `y` text and still invalidate the cache. This is not twenty distinct content changes. The interaction auditor independently repeated the workload and measured 84,464 / 666,123 / 3,423,090 µs for twenty requests at 1k / 10k / 50k rows. The repeat confirms the scaling, not a target frame budget; see [interaction verification](plan-interaction-verification.md).

Holla's activity (`activity.rs:75`–`:104`) and plan (`plan.rs:1000`–`:1022`) also rebuild/clone output vectors on each changed output length. TXT-03 means their nominal retention caps currently do not constrain this population path.

**Proposed primitive:** separate logical grapheme-cell caching from width-dependent visual rows; version changed logical lines, update only affected cells, and reuse unchanged line layouts. Add append/replace-tail/batch-delta APIs for live producers so apps need not replace the entire document. Choose the final scrollbar width without reparsing source text twice. Reuse the existing TextViewport and preserve logical anchors.

**Acceptance:** retained benchmarks for idle frame, one append at cap, last-line replacement, batched append, resize, wrap toggle and selection during churn at agreed 1k/10k/50k sizes. Instrument changed-line parsing to prove unchanged lines are not reparsed for one-line updates; verify bounded retention and copy semantics concurrently. Choose a target latency budget after representative workload measurements; do not invent a pass threshold from this single sample.

**Risk:** incremental invalidation can make styles, selection and width caches stale; land ownership/retention invariants and parity tests first. Cached visual rows still need reflow on actual width changes.

### TXT-08 — grid clipboard import scope is undefined beyond one cell

**Classification:** verified current behavior; potential coverage gap, not a confirmed contract violation. **Priority:** P3 / policy decision.

**Evidence/reproduction:** the grid exports ranges with tabs/newlines in `src/widgets/grid.rs:879` onward, but editing paste delegates to one single-line TextBuffer at `:1390`. Pasting `A\tB\r\nC\tD` into one cell stores `A\tBC\tD`. This follows the documented single-line normalization policy; no current contract promises TSV matrix import.

**Proposed decision:** either explicitly document and preview one-cell paste handling, or authorize a reusable matrix-paste/import primitive with parsing, type validation, all-or-nothing pending changes and clear error reporting. Do not infer permission to add matrix paste from this observation alone.

**Acceptance if authorized:** round-trip the grid's own exported ranges, quoted tabs/newlines, nullable/typed/read-only columns, bounds, invalid cells, undo and confirmation when replacing multiple cells. Keep simple text-input paste semantics separate.

**Risk:** importing clipboard matrices is new behavior with data-loss potential, not a routine Unicode fix. The planning choice should precede implementation.

## Dependency order and stop conditions

1. Decide text display/copy and edit-revocation policies (TXT-02, TXT-06); retain reproductions for every accepted defect.
2. Establish shared logical-line/grapheme ownership and revisioned mutation (TXT-01, TXT-05). Keep compatibility migration explicit.
3. Apply retention and edit deltas atomically (TXT-03, TXT-04), then adapt Holla's actual population paths.
4. Implement narrow DataGrid revocation enforcement with draft/pending-state tests (TXT-06); it can proceed independently after its policy is decided.
5. Add incremental live-output updates and measure representative workloads (TXT-07). Existing idle performance is a regression gate, not proof of live behavior.
6. Treat TXT-08 as an explicit product decision; do not add a new component automatically.

Completion proof for the eventual implementation: retained regressions for the accepted cases; relevant monochrome/nonzero-origin render inspection; source/copy/cursor invariants; representative Holla live-output and TablePro edit flows; then repository fmt, strict all-target Clippy and full tests. No source changes were made by this planning audit.
