# TextArea parent-window stage 3 regressions repaired

Status: **Three independently reproduced P2 regressions repaired in Fields; Form/application acceptance remains incomplete.**

## Reviewer findings and changes

The independent review of `61426b83dbf509b2e08a9e93458bc5f068b684d6` in `/Users/donbeave/Projects/termrock-cli-evidence-20261021/textarea-fields-review-61426/review.json` reproduced these failures against the public probes in `/private/tmp/termrock-textarea-61426-probes/tests/`:

1. Ordinary horizontal editing used `body.width - 2 - optional_scrollbar`; the established TextArea editing viewport is `body.width - 3`, regardless of the vertical scrollbar. Geometry now derives that preserved logical width from the same resolved window. A Harness regression verifies the first visible symbol `q` and cursor `(14,2)` with no vertical overflow, plus the same horizontal behavior when a vertical scrollbar is present.
2. The initial pointer press must hit actual `TEXT`, but an active captured drag must continue outside `TEXT`. Drag projection now maps through the parent source origin, clamps above to the first visible source row, and below to the last logical row while preserving horizontal extension. A consumer test captures a drag from a parent-clipped Unicode `三é` row to x=21, replaces the selected document row with `X`, and commits.
3. An empty placeholder belongs to logical source row zero. It is now painted only when row zero maps into the visible `TEXT` area. A consumer test covers source row zero visible and hidden.

The original external probe files and their reviewer logs remain unchanged. The reviewer reports the ordinary width/cursor and outside-drag probes passed on the comparison commit `ea553177c2c38995bfebd50085d41a2a963f826c` and failed on `61426b83...`; the placeholder source-row probe failed on the latter. Their hashes and detailed outputs are in the linked review packet.

## Verification on this repair

- `cargo nextest run -p termrock-fields --test textarea_window`: 7 passed, 0 skipped.
- `cargo nextest run -p termrock-fields --all-targets --no-fail-fast --status-level fail`: 77 passed, 0 skipped.
- `cargo clippy -p termrock-fields --all-targets -- -D warnings`: passed.
- `cargo fmt -p termrock-fields --check`: passed.
- `git diff --check`: passed.

During test development, the first new placeholder assertion used a consumer fixture without a configured placeholder; the fixture was corrected to include the same `type here` placeholder as the independent probe. The initial overflow width assertion inspected the first physical row after Ctrl+End had correctly scrolled the six-line document; it was corrected to assert the cursor's visible last row. The final tests assert the intended source cells and cursor coordinates.

## Remaining coverage limits

Resize-before-input, captured drag above and below the parent window, partial scrollbar-thumb drag, and full Form edit-menu/reveal behavior remain unverified. The initially clipped keyboard-menu anchor ordering remains unresolved. No application comparison or baseline acceptance is claimed.
