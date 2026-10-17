> **Status 2026-10-18: open.** The D10 holla fix is on `origin/wip/v1-showcase` at `c1a49af0f`; V1 itself is 0 of 66 roots. The V1 scripts are now in `docs/handoff/scripts/v1/` (set `V1_DIR` to a scratch dir, run from the worktree root; `base.sh` extracts the tag read-only). Next steps: [../README.md](../README.md).

I stopped as asked. No V1 group is finished, so nothing was pushed. One holla fix is committed locally on `v1-showcase` and is not pushed.

**V1 (showcase legacy re-approval): not started on any group. 0 of 66 roots accepted.**
- Inventory: 1,650 legacy captures across 66 roots in 26 groups (`flows/*` 50 roots, `fade` 10, `audit` 5, `hover` 3, `resize` 2). All 1,650 legacy `.ansi` files are byte-identical to the `visual-baseline` tag. I only read the tag, through a scratch `git archive`.
- I stopped the full determinism capture at about 475 of 1,710 scenarios (the filter also picks up some pending rows). Nothing was accepted and no snapshot files changed.
- Comparing against the baseline row by row is useless for the showcase. The pages were restructured (tree sidebar, `Playground / Options / States` tabs, new page content), so a plain diff is nearly the whole screen. The method I set up instead compares what each interaction changes: baseline page → baseline flow, against the approved `snapshots/showcase/pages/<page>` → actual flow. `audit/*` has no page root of its own, because `audit/buttons`, `diff`, `forms`, `inputs` and `textareas` are those pages, so it needs direct review.
- One open item from the first `audit` look: 9 of 25 captures per root have text that varies by palette, beyond the palette label. I haven't looked into why yet.
- **To resume:** run `det.sh` per group filter from `groups/*.filter`, then `report.py <group>` and `effect.py <root>`. Read the effect at 120x40, run the automated checks at the other sizes, then `tuiscotti accept` per group and add the "Showcase legacy (V1)" section to `visual-reapproval.md`. The scripts are in `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/v1-showcase/`: `env.sh`, `mkfilter.py`, `det.sh`, `ansi.py`, `report.py`, `effect.py`, plus the extracted baseline under `base/`.

**Holla flake (`holla/flows/task/input_cancelled`): cause found and fixed in commit `c1a49af0f`, unpushed.**
- **Cause:** holla's fixture clock advances once per runtime tick, and ticks arrive on wall time. The task's shown duration counted the ticks a prompt spent waiting for the operator, so it depended on how fast the harness typed. The script part is 7 ticks (560 ms), and the wait time pushed the total either side of 1 s. That is the `1 s` → `0 s` digit at (32,4) and in the footer.
- **Fix** (`src/bin/holla/domain/activity.rs`): the shown duration now counts only the script's own time, leaving out the time a prompt waited. If a task is cancelled while a prompt is waiting, that wait is closed and excluded too.
- **Second defect fixed in the same commit:** when an answer ends the program, the lines after the prompt were still printed. That is why `rolling out` appears after `rollout cancelled by operator`; the baseline has it too. Those lines are now dropped.
- **Test:** `domain::activity::tests::duration_does_not_depend_on_how_long_a_prompt_waited` fails on the parent (4 vs 3) and passes now. All 227 holla tests pass and fmt is clean.
- **Visible change:** the captures will now show `0 s` instead of `1 s`, and the `rolling out` line disappears. Holla durations now mean script time rather than wall time, which is a product decision for the owner to confirm.

**Still open before `c1a49af0f` can be pushed:**
- Re-approve `holla/flows/task/input_cancelled`, and possibly `input_killed`, in a snapshot-only commit that names the fix.
- The 20× determinism proof was not run.
- The full gate and clippy were not run, and I haven't added the entry in `docs/audits` yet.

**Remaining risk (not fixed):** `input_killed` shows `1 s` = (key `s` − start) + 12 ticks. The gap before `s` is wall-paced while the task runs, so on a heavily loaded machine it could become `2 s`. The prompt fix doesn't help here, so this should be recorded in `docs/audits`.