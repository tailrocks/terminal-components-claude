# Visible terminal caret retention contract (2026-10-21)

This audit records a bounded component contract added on the PR #38 work line.
It does not claim application or frozen-baseline parity.

## Public surface and ownership

- `TextViewport::caret_when_unfocused(bool)` opts a read-only viewport into
  publishing its existing state caret while another control owns keyboard
  focus. It defaults to `false`; follow mode, selection suppression, clipping,
  and viewport projection still apply.
- `ContextMenu::retain_cursor(Id)` opts a menu into retaining an already
  registered parent control's cursor. It defaults to `None` and does not
  register a control or create a cursor position.
- `Ui::retain_visible_owner_cursor(Id)` is the component-author adapter for
  that visible-state case. Existing strict `retain_owner_cursor` semantics
  still require keyboard focus.

The runtime accepts only a unique same-frame cursor request belonging to the
immediate parent of a drawn, topmost, nonmodal, noninert popover. It rejects
missing or stale requests, popup-owned registrations, empty/out-of-bounds
areas, and ambiguous owners. Retention moves the existing cursor request into
the popover layer. A real typing owner in that layer suppresses the retained
cursor even when its own caret is hidden; an ordinary visible top-layer cursor
request takes precedence. Closing the layer leaves no retained request.

## Evidence

The compiled integration consumer in
`crates/termrock-overlays/tests/visible_terminal_cursor.rs` uses only public
Termrock crates with log-console data (including wide and combining Unicode),
real `TextViewport`, `ContextMenu`, `Button`, `TextInput`, and the test harness.
It covers focus transfer, keyboard navigation, activation, dismiss/reopen,
caret projection/suppression/clipping, opt-in defaults, frame-cell equality,
and an editable overlay cursor taking precedence. Runtime unit tests cover
hidden editable caret suppression, top-layer visible cursor precedence,
strict-mode preservation, rejected owners, and conflicts.

Command evidence for the current worktree:

```text
cargo nextest run -p termrock-runtime -p termrock-viewport -p termrock-overlays --no-fail-fast
563 passed, 3 skipped
cargo clippy -p termrock-runtime -p termrock-viewport -p termrock-overlays --all-targets --no-deps
passed without warnings
cargo fmt --check
failed on an unchanged Jackin app.rs:261 blank line; scoped package formatting passed
```

The hidden-caret regression was checked before and after the runtime selection
change. With the legacy `typing_resolved.cursor.or(retained_cursor_owner)`
selection restored temporarily, `real_editable_overlay_typing_suppresses_retained_owner_when_caret_is_absent`
failed because it selected the retained terminal owner. With the implementation
restored, it passes. The test builds the live popover, parent cursor, editor
registration, and editable fallback typing declaration through `Ui`; it does
not fabricate a cursor stack or control in production.

The skipped tests are existing skips and were not counted as passing coverage.
This is a widget/runtime-level public consumer, not a real CLI run. No paired
baseline capture, four-application flow comparison, Showcase coverage result,
or visual-baseline acceptance claim is made here. PR #38 has no Showcase
catalog framework in this branch; the three additive APIs are declared above
instead of inventing one.

## Independent review correction

The original worktree did not pass repository-wide formatting; the command evidence above is corrected to match the independent before/after observation. Exact source `8be128f073c19076a2977e349e8732cd74f08821` received bounded independent runtime/API clearance with 563 native passes, three existing skips, nine external consumer passes and 72 unchanged default frame/focus/cursor/shape checkpoints. This does not approve the remaining application cursor workarounds or count as Showcase adoption.
