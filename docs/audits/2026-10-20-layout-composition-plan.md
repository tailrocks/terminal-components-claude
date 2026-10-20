# Termrock layout composition plan

Status: bounded geometry primitive implemented; framework adapters and live application consumers remain open.

## Source and scope

- Source branch at start: `parity/termrock-visual-baseline`, `a5766d24783c2b268e727497fe6861b6479c71cc`.
- Product reference remains baseline source `4169783ae883788186d161af09ba9c481bd84e5c`; this change does not alter reference artifacts or application rendering.
- Added pure immutable row, column, and wrapping plans in `crates/termrock-layout/src/composition_plan.rs`.
- Existing `layout.rs` free functions, `measure.rs` types, and runtime APIs remain unchanged.
- No widget, input, application, or domain dependency is introduced.

## Contract delivered

`ChildPolicy` supplies intrinsic size, minimum and optional maximum dimensions, and flex weight. A pure `Fn(&T, Constraints) -> Size` callback measures each child once. `Composition` supplies padding, gap, main and cross alignment, and overflow behavior. The result retains source indices and exposes logical bounds plus bounds clipped to the padded content region. `Clip` compresses to the available region; `PreserveMinimum` retains minimum extents while reporting the visible intersection.

Tests cover row and column measurement, wrapping order and line heights, weighted flex caps and reallocation, deterministic remainder placement, padding and gaps, shifted origins, zero-sized regions, conflicting bounds, clipping, and large child inputs. An integration test compiles the public API with separate session-summary data.

## Independent review follow-up

Review of `8f7fc45a0ddd448036bacc324a215ce74a9814c8` found three defects, now fixed in the follow-up commit:

- Minimum size inside `ChildPolicy::intrinsic` now participates in normalization. Regression uses two intrinsic `Size::exact(8, 1)` children in an eight-cell row and preserves the first child's minimum.
- Main-axis totals, wrap-line widths, offsets, and cursor accumulators now use saturating `u64` arithmetic. A 65,538-child `PreserveMinimum` case exercises totals above `u32::MAX`.
- Wrapping computes gap positions once per line, removing repeated per-child vector construction. An 8,000-child, single-line regression counts exactly one gap-position resolution; no timing threshold is used.

## Verification and remaining work

- `mise exec -- cargo nextest run -p termrock-layout --all-targets`: 22 passed, 0 skipped after review fixes.
- `mise exec -- cargo clippy -p termrock-layout --all-targets -- -D warnings`: passed.
- `cargo fmt -p termrock-layout --check`: passed.
- `git diff --check`: passed.

The plan is a geometry primitive only. It is not yet wired to runtime draw/update, clipping, hit registration, focus, or cursor placement. No application uses this API yet, so it does not establish library-gate completion or product parity. Grid plans, reusable block composition, resize invalidation, adapters, and real Showcase/Jackin/Holla consumers remain pending.
