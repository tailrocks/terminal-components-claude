# TextArea captured vertical drag repair

Status: **The reviewed captured-drag viewport regression is repaired in the Fields TextArea adapter. No Form or application acceptance is claimed.**

The independent review of `25724c71d1c6062db09f9d44f6407c232b8a4e8e` found that `captured_document_point` capped a captured pointer at the parent's last logical row. This changed established document-selection behavior: with a 5-row TextArea holding ten lines (`a` through `j`), dragging from `(4,2)` to `(4,11)` and typing `X` produced `Xe\nf\ng\nh\ni\nj`, where the original behavior replaces the selection through the document edge and commits `Xj`.

The captured projection now preserves downward physical-row extrapolation after applying the parent's source-row origin. Existing selection code remains responsible for clamping to document bounds. Initial pointer presses remain restricted to visible `TEXT`; only an active captured drag uses extrapolation. The focused regression uses the public `Harness`, `TextArea::draw_window`, normal click/Ctrl+Home/drag/key/Esc input and asserts the committed document value.

Independent evidence: `/Users/donbeave/Projects/termrock-cli-evidence-20261021/textarea-fields-review-25724/review.json` records the exact two boundary probes: both pass on the pre-regression comparison commit `f2d66896f3341b115a217a1bdad6379badebccbb`; the reviewed candidate passed the top-edge case and failed the bottom-edge case. The committed test was run RED with the old logical-row clamp (`Xe\nf\ng\nh\ni\nj`, expected `Xj`) and GREEN after this change. The previously reviewed top-edge behavior remains covered by that unchanged independent probe; this increment does not claim broader parent-clipped top/bottom coverage.

## Verification

- `cargo nextest run -p termrock-fields --test textarea_window captured_vertical_drag_outside_parent_window_reaches_document_edge --status-level fail`: 1 passed, 7 skipped. The same test failed before the code fix with the incorrect truncated selection.
- `cargo nextest run -p termrock-fields --all-targets --no-fail-fast --status-level fail`: 78 passed, 0 skipped (run `56a66815-15db-49fd-a827-c3d201a06acc`).
- `cargo clippy -p termrock-fields --all-targets -- -D warnings`: passed.
- `cargo fmt -p termrock-fields --check`: passed.
- `git diff --check`: passed.

The earlier three Fields repairs and their passing 77-test/Clippy evidence remain in [the stage 3 repair record](2026-10-21-textarea-parent-window-stage3-repairs.md). Resize-before-input, partial scrollbar-thumb drag, clipped caret menu anchoring, Form adoption, and application-level paired parity remain unresolved.
