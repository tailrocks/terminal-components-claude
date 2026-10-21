# Form pointer focus reveal during a held gesture

A chooser nested in a scrolling form could move after pointer-down when focus auto-reveal ran on an intervening update. The terminal runtime correctly dispatches a click only when the release still hits the pressed owner and part; the Form-owned layout change caused the release to miss.

`FormState` now tracks the child whose pointer press began, and defers Form's automatic focus, radio-cursor, and queued reveal scrolling until release, drag-end, cancellation, or the held field becoming hidden/disabled. This is shared by both ordinary forms and lane forms. A live `PRESSED` state only keeps an already observed gesture alive when no terminal intent is present; it does not independently start suppression. Explicit reveal requests remain queued during the hold. No button, runtime, or application behavior was changed.

Regression coverage drives a public FormDialog through pointer Down, Tick, Up at the original chooser coordinates, asserts stable hit geometry and exactly one Chose action, and checks release outside cancels. A second test exercises the same contract through a lane-based Form. Existing form tests continue to cover keyboard focus, validation and explicit reveal behavior.

## Evidence

- Exact pre-fix source: `5478ecfc93e294ccd9ebd75058de7be09e2a389f`.
- Test-only diagnostic source SHA-256: `63f405802c3c631dbcb333d58a09495f05c0718fffa6f5cc19557371d32f2b6c`.
- Exact-parent command: `CARGO_TARGET_DIR=/Users/donbeave/Projects/termrock-worktrees/showcase-form-pointer-reveal-20261021/target cargo nextest run -p termrock-forms --lib -E 'test(form_dialog_keeps_clipped_chooser_hit_area_stable_through_pointer_release)' --no-fail-fast`.
- Exact-parent result: expected failure at the intervening Tick; chooser moved from `Rect { x: 32, y: 29, width: 9, height: 1 }` to `Rect { x: 32, y: 28, width: 9, height: 1 }`.
- The test-only diagnostic worktree is detached and not part of product changes.

## Independent clearance and integration

The bounded change received an independent CLEAR review at candidate `b40dd49bd3e2fbc57a4c647206351f920ca8ea9f`; packet SHA-256 is `825ac638edc3476ff06a45f59d14b4d209cec5180af4455c84fe1927a2105eb8`. The verdict is limited to Form pointer-hold/deferred reveal behavior, not application parity. Reviewers exercised both ordinary and lane paths, outside release, invalidation/resize, explicit reveal, drag-end, and modal interruption; detailed logs and limits remain in the packet.

This source was merged as a separate integration commit into the current PR #38 head. The seven protected expected/assertion blobs remain pinned by the existing held-expectations manifest and are checked again on the merged tree. This integration adds no expected-output changes.
