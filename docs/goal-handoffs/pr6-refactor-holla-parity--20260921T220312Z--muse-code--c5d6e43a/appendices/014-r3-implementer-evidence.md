# TASK-014 r3 implementer evidence (2026-09-21)

Worktree: /tmp/tc-014-r3/wt @ 80399a1e (scope-base 227f21b6, clean)
Run dirs: /tmp/tc-014-r3/run (manual 7 checks), /tmp/tc-014-r3/run-verify (taskfmt verify + validate)
Commit: 80399a1ebf7020bcdb76ff3d48f89cd7e2e71c8b (ONE commit on 227f21b6, -s as Alexey Zhokhov, Co-authored-by Codex)

## r1 -> r3 delta (repair of REJECTED 01849854)

r1 failed ONLY rule-22 + palette-literals gates (Color::Rgb at paint.rs:549-550).
r3 cherry-picked r1 onto 227f21b6 (zero conflicts) then reworked exactly that
hunk onto the 079-owned `theme::builder::fade_mix` path (theme/** untouched):

- paint.rs: +`use crate::theme::builder::{FadeOutcome, fade_mix};`
- `fade_edge_row` matches `fade_mix(cell.fg, container, keep)`:
  Blended(mixed)->repaint fg; ApplyDim->`|=` DIM keeping fg; Unchanged->as-is.
  Guard (`bg == container`, non-reversed) and glyph/geometry behavior unchanged.
- Removed local `fn fade_mix(u8,u8,f32)->u8` (superseded; name clash resolved).
- Call domain is exactly {0.55, 0.80}, so helper `amount == 0.55` DIM routing
  is identical to r1's `keep <= 0.55`; helper f32 transcription reproduces all
  oracle pins (110/160, OUTER/INNER, other vectors) — proven by 15/15 + 7/7.

## Grep proof (owned writable paths)

- Production src: ZERO `Color::Rgb(`/`Color::from_u32(` hits (scroll.rs,
  scroll_region.rs, author.rs, paint.rs non-test lines).
- Remaining 11 hits all inside paint.rs `#[cfg(test)] mod tests` (line 685-EOF)
  + completion_014.rs: gate-excluded regions that pin oracle values (110/160).
- Changed vs base: exactly scroll_region.rs, paint.rs, completion_014.rs.

## 079 dead_code warnings: GONE

- Pristine 227f21b6: `FadeOutcome`/`fade_mix` never-used warnings PRESENT.
- r3 tree: 6 warnings, all pre-existing streaming-scratch dead code
  (StreamGuard/StreamWriter/paint_display/text_scratch/clusters len-peak-cap);
  zero fade-related warnings. Consumption cleared them.

## Dependency receipts (accepted+integrated, /tmp/tc-goal-b518/receipts/)

TASK-013/010/011/012/008/073 shas re-verified byte-identical to r1 bindings.
Oracle peel read-only: refs/tags/visual-baseline = 4a79c0a2 / tree 0b1f1343.

## taskfmt (re-qualified before use)

0.2.0 / afd3b575dbcc7044620bec4b9493a74eca3e5ef2 / f9781ef8...664de.
lint 014: exit 0, errors=0 warnings=0.

## Contracted checks (native prepare + launcher, scope-base 227f21b6)

- campaign-build-proof.sh: exit 0 both run dirs (build.log, build-verify.log)
- prepare: exit 0 both (prepare.out, prepare-verify.out; tree 90bfb46d)
- manual CHK-001..007 in run/: ALL exit 0 (manual-CHK-*.out/.err)
- taskfmt verify in run-verify/: exit 0, SUMMARY pass=11 fail=0, DONE
- validate run-verify: exit 0 (transcript stored as taskfmt-logs/taskfmt-verify.log)

## Focused nextest (cargo nextest only)

- completion_014: 15/15 exit 0
- lib fade filter (7 paint scroll_edges + 4 builder fade_mix): 11/11 exit 0
- lib scroll filter (scroll_region + scroll): 19/19 exit 0
- architecture palette_literals + no_deprecated_or_legacy_api_usage: 2/2 PASS
- full -p junie-tui: 2805 run, 2800 pass, 5 fail, 7 skipped, exit 100 —
  EXACTLY the knowns: 4 viewport TASK-021 + registry every_named_test_exists
  TASK-068 (missing split_first_pane...), all reproduced without the r3 delta.
  (First full run also flaked no_unsafe/no_unreachable_spin_loops/
  no_todo_or_unimplemented under load; green in isolation + rerun.)

## Static gates

- rustfmt --check on the 3 owned files: exit 0 (workspace-wide 029 diff pre-existing)
- clippy -p junie-tui --lib: exit 0, 8 warnings all pre-existing (clusters + streaming)
- clippy --test completion_014: 0 warnings from this task
