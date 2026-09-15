# TASK-067 protected closure obligations

These clauses are planner-owned acceptance inputs. Freeze this file and the exact mapped historical rows before dispatch. Candidate output cannot amend them.

## Source authority

ARCH:A26; ARCH:A27; HIST:A116–A126; main COMPONENT_ARCHITECTURE.md §16 performance table and §§20.9,25,27,37,53,66,69,70; main apps/showcase/tests/perf.rs:129 currently compares digests, not allocation counts.

The full historical clauses, remaining work and proof requirements are included in source-obligations.tsv, with their original ledger locations. Architecture and decision IDs are namespaced; HIST:A23 is not ARCH:A23.

## Exact closure clauses

1. Implement style_resolve_share_of_frame_showcase_lists_120x40 against the actual restored production Lists frame: style-resolution time / total frame time <= 5% under PERF_STRICT=1. Keep zero-allocation and key-correctness/cache tests. Demote only the expired stand-in extrapolation from asserted to reported.

   Consume the single testing-only production probe produced and independently qualified by TASK-073, preserved by TASK-011. Freeze the complete reachable resolution-boundary census for the actual Lists draw, including any reached direct binding path; a wrapper cannot double-count its leaf and an uncovered path cannot be omitted. Run fixed interleaved batches of the identical warmed production frame in three modes: disabled hooks measure the entire real draw denominator; adjacent empty clock start/stop at each selected boundary measures calibration while real resolution executes outside that interval; measured hooks surround the actual resolution work. Use safe stable-Rust timing and caller-owned counters, all-call membership and identical call order across modes. No sampled extrapolation, synthetic frame, sleep budget or detached resolver microbenchmark is the numerator or denominator.

   Preserve exact complete frame/cursor and semantic-state equality across modes, and unchanged allocation/cache-key behavior. Report raw duration, matched calibration, raw-minus-calibration correction and paired uninstrumented frame time separately. The host fixes compiler optimization, clock/environment, warmup, batch count/order and measurement-error calculation before candidate dispatch. Negative correction, missing or unequal call census, nonpositive denominator or uncertainty that does not prove the bound fails; never clamp, discard unfavorable batches, retune tolerance or substitute an arbitrary larger frame budget. The protected judge independently computes the corrected numerator's upper bound divided by the paired frame denominator's lower bound and requires it to be at most 0.05.

   Independent qualification must reject constant/foreign numerator evidence, omitted resolution calls, duplicate wrapper accounting, an enlarged or unrelated denominator, swapped calibration/census, candidate-selected samples and an interval straddling the threshold. Its positive real-frame evidence remains valid after each rejection. TASK-067's tests consume this accepted protocol; candidate test assertions alone cannot qualify their own observation seam or judge.

2. Warm production allocation ceilings: Showcase Lists <20; TablePro 500x14 grid <100; Jackin Manager 100 rows <60; Jackin four-pane Capsule <200; TablePro 2k-line query editor <40; declared form <40 allocations/frame. Grid 500x12 load <8000 and render <100.

3. Require actual equal allocation counts for render_twice_allocates_the_same; digest equality remains a separate visual assertion. Capture before/after counters around the real calls, including negative allocator-count perturbation proving the assertion fails.

4. Preserve zero-allocation keyboard cursor/navigation, mouse/wheel/focus/style/Unicode paths; local Grid sort <=1 allocation/comparison; 10k part cache hit floor >=90% plus deterministic key-generation tests; strict per-query bound <=16ns and accepted overlay/backdrop ratios.

5. Preserve 100k List <500 allocations/frame and <=1.5x strict 1k rendering; Tree counts/subtree work and isolated perf_collections target; borrowed Picker 19 visible rows/38 calls/zero bytes and <=1.5x; unchanged warm viewport zero allocations and prefix visits, equal visible visits at 1k/100k, <=1.5x strict.

6. Measure cold, reflow and invalidation separately without imposing a false document-independent threshold. Append work equals changed/appended content, not retained document length; retained selection/marks cannot restore whole-document work.

7. Row ellipsis inline-symbol corpus has zero allocations. ZWJ corpus is separate: equal 10k/100k counts for 80 columns and <=80 allocations. Primed 100k list/viewport bytes per frame <64KiB. Debug/release allocation counts differ by at most one.

8. Empty intent queues at 20/500 controls have zero probes/allocations; nonempty single-pass differential is exactly480 probes, not an arbitrary multiple. Preserve normalized strict ratio <=1.25x; raw O(n) stub-loop timing is report-only.

9. Form real Runtime Tick+draw has zero allocations/bytes; one-shot theme downgrade allocation ceiling1079 and no downgrade inside frame profiles. Preserve hit registry bounds, minimum ring ownership and all other exact accepted rows listed in the sealed performance inventory.

10. PERF_BLESS is never available. The retired capsule_pane_clone_4x2000 remains absent with replacement no-clone proof; do not reintroduce cloning to satisfy an obsolete name.

## Check selection and evidence

The host binds the complete all-app and component required set to this task context; no candidate subset selector is permitted. Direct and PTY captures use their separately sealed oracle lanes. CHK-004 compares every required completed product checkpoint. CHK-006 additionally enforces the primary closure outcome; CHK-006 runs the exact applicable accepted architecture/API/performance checks, CHK-005 executes and reconciles the full test inventory, and CHK-007 requires all independently checked products and post-run integrity. TASK-070/071/072 implement and independently qualify those operations before dispatch.

The accepted task-stage command inventory records exact cargo targets, profiles and immutable relocation rules. TASK-066 may produce the explicitly scoped render-target relocation only after independent old/new identity comparison; downstream contexts use the host-accepted relocation receipt. A candidate edit to tools/test-inventory does not itself redefine that authority.

This is proof completion, not authority to repair arbitrary production performance defects in test scope. A failed bound returns to its component/application owner. Frozen before rows retain their original SHA and units; candidate performance results cannot overwrite them.
