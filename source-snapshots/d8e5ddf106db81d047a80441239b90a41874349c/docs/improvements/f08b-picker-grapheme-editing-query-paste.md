# F08b — Picker grapheme editing/query paste

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Implement grapheme-safe query edits and one QueryChanged event per paste in the shared Picker; route through Holla modal ownership. Prove typed/pasted query equivalence and the full widget state matrix.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

TablePro picker-owner paste integration and F08a remain deferred; do not claim those application routes fixed.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

Family policy: [F08 shared context](shared-contracts.md#f08-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** P2 confirmed deletion defect + coverage gap. `picker.rs:199` uses `String::pop`: Backspace on 👩‍💻 leaves woman+ZWJ. No query-paste entry point; owners consume paste or leak it through F01.

**Root fix / retained acceptance:** Reuse TextBuffer/grapheme operations and add query paste emitting existing `QueryChanged` exactly once. Medium owner/API risk. Typed/pasted Unicode queries produce identical rows; complete-cluster Backspace, clear, disabled/search-disabled/loading/error and escape behaviors are tested in widget and real owner. Coordinate F02, but keep destructive eligibility separate.

## Evidence

**Slice status:** current shared/Holla slice complete · TablePro picker-owner paste and F08a stay Later.

**Shared widget:** the Picker query edits by grapheme (Backspace removes a whole 👩‍💻 cluster) and paste is one edit that emits `QueryChanged` exactly once. Test: `picker.rs: query_edits_are_grapheme_safe_and_paste_is_one_event`.

**Holla ownership:** the app routes `Input::Paste` to the topmost owner only (the modal picker, the finder query, the files jump picker, the facts page field). Tests: `app_tests_proofs.rs: facts_page_takes_an_actual_paste_into_the_editing_field_only`, `batched_and_separated_event_sequences_agree_on_focus_hits_drafts_and_cursor` (paste inside the sequence); `app_tests_parity.rs: hp02_recents_learned_queries_query_editing_and_persistence_merge` (typed and pasted queries rank the same rows; paste is one undo step), `hp03_find_files_indexes_home_truthfully_and_resource_actions_are_exact` (`café` query).

**Captures inspected:** `shots/h_hp02_query_selected`, `shots/h_hp03_find_unicode`.

**Deferred remainder:** TablePro picker-owner paste (Later checkbox).

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
