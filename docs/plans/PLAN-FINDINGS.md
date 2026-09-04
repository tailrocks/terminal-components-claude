# Cross-cutting findings from the Slice 6 and Slice 7 plans

Both plans were produced read-only against the tree. Each item below is a **measured discrepancy** between a recorded number or claim and what the source actually says. They are listed separately from the plans because several affect work already in flight.

## Counts in `COMPONENT_ARCHITECTURE.md` §16.4 that are wrong

| Recorded | Actual | Evidence |
|---|---|---|
| TablePro "21 tests" | **23** | `src/bin/tablepro/app_tests.rs` — the audit's own table lists 23 rows; the omitted two are `acceptance_flow_keyboard_only:680` and `acceptance_flow_mouse:782` |
| Jackin "22 (17 + 5 chrome)" | **28 (22 + 6 chrome)**, plus 10 in-module units = 38 | `app_tests_chrome.rs` has six `#[test]` fns |
| Jackin `Screen` trait "20 methods" (app-audit) | **23** | `screens/mod.rs:231-328` |
| §16.6 capsule frame "≈480 000 allocs" | **1 080 602** | `tests/perf_baseline.txt:6` — the real figure is 5.4× the documented estimate, so the `< 200` Slice 7 target is a 5 400× reduction, not a 2 400× one |

`architecture::every_named_test_exists` compares the documented inventory against the compiled list, so each of these fails the gate until corrected.

## §29 is cited eight times and does not exist

`COMPONENT_ARCHITECTURE.md:9, 45, 1734, 1815, 1869, 3943, 6211, 6222` all reference "§29 / Adjudication Q"; the document ends at §28.8. `REFACTORING_STATE.md` lists "Record as §29" as an open task. Slice 6 quotes §29's `Slot<GlyphRole>` contract for `RowDecor`/`CellDecor` markers, so the plan depends on a section that is not written. `xtask doc-check`'s §21–§26 range and the inline markers will disagree until it lands.

## Three live colour collisions in `Theme::junie()` that `rain::dim_buffer` reverse-maps

`dim_buffer` (`rain.rs:102-172`) identifies a cell's ladder position by comparing its rendered colour against palette fields. In Junie today:

- `accent == focus == success == GREEN` (`theme.rs:167, 172, 177`) — the three-way test at `rain.rs:128` is degenerate.
- `border_subtle == text_ghost == WHITE_15` (`theme.rs:159, 165`) — **every border cell already reverse-maps to ladder step 0 and is erased at `steps >= 1`**.
- `disabled == text_faint == WHITE_30` (`theme.rs:164, 173`) — every disabled cell maps to step 1.

The last two are silent defects in today's handoff cross-fade, visible only in `HandoffStage::CockpitDim`/`CapsuleDim` frames, which no test pins. They must be classified before any Jackin baseline is re-blessed.

## The `capsule.rs:1183` panic has a deeper root cause than a missing bounds check

`run_id` is a bare `String` with two producers that agree on nothing: `fixtures.rs:469` yields `"run-7f3a"` (7 bytes after `replace`), `cockpit.rs:84-90` yields 21 chars. The display site invents a derived token by byte-slicing `[..8]`. Every fixture instance panics; `cockpit.rs:88`'s `[..12]` is the same defect one line over. The structural fix is a `RunId` newtype with one constructor and only total accessors, plus moving the formatting into `Instance::container_uid()`. The workspace already denies `clippy::indexing_slicing` and `clippy::panic`, so with the newtype in place the lint has nothing left to fight.

## Open questions that block work beyond their own slice

- **Slice 6 Q1/Q2/Q3 block Slice 4 wave 2**, not merely Slice 6: whether the library `Grid` owns a sort permutation and on what comparison (it sees only `CellRef` and cannot reproduce NULLs-last ordering); what `GridState` exposes (eleven TablePro chords and three migrated tests need the cursor); and whether a control may register a zero-area focus entry (the explorer drawer's focus stop depends on it, and rule R5 forbids registering from a component that cannot draw).
- **Slice 7 Q2 blocks Jackin entirely**: the accepted `App` trait has no tick hook and no accessor for elapsed virtual time, yet Jackin's clock, both rain state machines, the launch stage machine, `world.jobs` and the status timeout are all tick-driven.
- **Slice 7 Q1**: `Screen::strip_right` is listed as removed with no named replacement, and `HintLayer.status` cannot carry its priorities and tones.

## The highest-risk single item in either slice

Jackin's virtual clock advances by the **route's** nominal interval (`Route::tick_ms`: 33 ms for intro/outro/handoff/cockpit, 80 ms for Capsule), not by real time or by a uniform token. If the migration re-bases it on `design.motion.tick_ms` or on a wall-clock delta, every `h.ticks(n)` count (~40 sites), `FAILURE_TICKS = 77`, `RUNNING_FRAME = 20`, `OUTRO_FRAME = 150`, every fixture timestamp and the outro elapsed caption all break at once — and `rain::TICK_MS = 33` with `HANDOFF_LEN = 12` means a uniform 80 ms would run the intro at 2.4× the wrong speed.
