# Migration handoff

Start with the current [Termrock migration and acceptance matrix](../architecture/termrock-migration-matrix.md). It records fixed source identities, application status, paired CLI coverage, architecture findings, unresolved differences, and next evidence. Treat every partial or missing result as incomplete work.

## Palette and shared-grid handoff (2026-10-21)

Preservation branch: `wip/palette-grid-repair-20261021`, tip `148c32c1bd7b646b032b3eaa739d1ee1338ecf2e` before this handoff update. The work is based on PR #38 commit `5c0075a09eec51b805f15d79ad945d4fb51fb5bf` and consists of:

- `7d5246b1b487dccfca76648bf13be13e2577026d`: remove exact-RGB global conversion exceptions; keep the frozen Junie ANSI-256 semantic values for `on_accent=232`, `border_strong=242`, and `disabled_fg=239`.
- `3a8b18362954da068e1c3418817fcb9d25c0ab37`: apply existing `Ui::scroll_edges` behavior in shared Grid and remove TablePro's private bottom-edge repaint.
- `148c32c1bd7b646b032b3eaa739d1ee1338ecf2e`: state the unresolved app-level repaint limitation accurately in the audit.

Focused checks: `cargo nextest run --locked -p termrock-grid` passed 125/125. The filtered theme check passed 2 tests, with 121 filtered. `cargo nextest run --locked -p tablepro-ui` passed 167/180 and failed 13; failures were not reproduced against the pre-change source, so their attribution remains unresolved. This is not application parity evidence.

Important limitation: TablePro's `draw_painted_grid` / `paint_scrolled_cells` runs after the shared Grid renderer and can overwrite the new shared fade. Thus direct Grid behavior is verified, but the application-visible final cells are not proven fixed. Remove that repaint only after mapping its row-number, cell-style, hit/focus and scrolling contracts to shared components, then compare the real CLI against the frozen baseline. An attempted merge into the existing WIP and theme-palette branches conflicts; integrate these commits through the PR #38 source branch after review and revalidation.

Next: independently review the exact commits, integrate onto current PR #38, repair the actual TablePro post-render repaint in its owning component migration, and only then continue the compatible per-color-level Split/Seam recipe work. Do not claim visual parity or treat the 13 failures as pre-existing.
