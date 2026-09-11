# Independent assembly and Choice caller rereview

## Result and boundary

PASS for the two-line dependency projection repair and the corrected ADJ-10 Jackin caller census. No production or global plan file was edited by this reviewer. Checks ran in `/tmp/assembly-rereview.ySTjup`; this report is the only repository write for this subtask. Verify-and-stop bounded the work. This is not global plan approval or a claim that the whole assembler is an independent acceptance judge.

## Actual assembly checks

The current `evidence/assemble-plan.py:77–78` reads each task package's `task.toml` and projects its ordered canonical dependencies into the index. It does not infer dependencies from titles, prose, task numbering or the stale index. Existing writable-path projection remains sourced from verify.toml.

Two disposable trees copied the actual 73 packages' task/verify metadata plus the eleven authored/input/output TSVs: 157 files each. The authoring script ran with `python3 -B` and `python3 -B -O`, never against the live repository with `--write`.

- Initial normalization changed only traceability.tsv and task-index.tsv, matching the copied checkpoint's outstanding joins. All 73 dependency fields then exactly equaled canonical metadata, including empty and multiple-dependency lists. Metadata order was preserved.
- Every index field other than dependencies/writable_paths was compared before and after, including titles, keys, purpose prose and wave. All remained equal. All non-output files retained their hashes. The two modes produced identical complete file maps.
- Repeated dry and write runs reported `changed: []` in both modes.
- A real negative fixture replaced TASK-020's index dependencies with `TASK-999;TASK-999`, leaving task.toml untouched. Both modes detected task-index.tsv as the only drift. Write repaired it from metadata; a repeated check reported no changes.
- An old-mechanism mutant removed only the two new lines from a disposable script copy. On the same drift it reported no changes and retained the incorrect dependencies. An independent metadata/index comparison rejected precisely TASK-020. The repaired normal and optimized runs had zero mismatches. This proves the new projection removes the old stale-index mechanism rather than merely changing a source string.

Representative commands were `rtk proxy python3 -B [-O] docs/refactoring-plan/evidence/assemble-plan.py --root /tmp/assembly-rereview.ySTjup/{normal,optimized} [--write]`. The independent comparison used explicit exceptions, not optimization-stripped assertions.

Important interface limit: dry-run exits zero even when `changed` is nonempty. This is a mechanical join/report utility, not a fail-closed no-drift gate. A caller claiming no drift must inspect `changed == []` or use the separate plan validator; exit zero alone is insufficient. The root coordinator was notified.

## ADJ-10 exact caller census

Pinned oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c` was searched with `git grep`, then each actual constructor branch and the shared dialog were read through `git show`.

| Actual oracle path and line | Branch | Current owner |
| --- | --- | --- |
| src/bin/jackin_preview/screens/prelude.rs:164 | Destination step chooses default/same path versus edited destination | TASK-051 |
| src/bin/jackin_preview/screens/config.rs:1568 | Mount/environment scope choice | TASK-052 |
| src/bin/jackin_preview/screens/capsule.rs:1093 | Dirty exit chooses new agent, inspect, keep or discard | TASK-056 |

These are the only actual ChoiceDialog constructor call sites in the oracle Rust source. `screens/modals.rs:590` is the shared RadioGroup constructor inside ChoiceDialog, not an additional screen. Its key dispatch calls RadioGroup::on_key and `fire` consumes radio.selected. Direct Form RadioGroup constructors also occur in Accounts (`accounts.rs:537`) and Config (`config.rs:1652,1785`), matching TASK-053/052. The correct task caller list adds TASK-051 and retains TASK-056.

Root subsequently caught a material ownership error in this review: it had assigned capsule.rs:1093 to TASK-055 merely from its file location. The actual branch is dirty `request_exit`, not Capsule split. Independent reread of `app-flow-stage-audit.tsv:60` confirms JA-059 has TASK-056 as both previous and complete primary and owns every dirty-exit variant; TASK-055 is only an earlier producer. Current ADJ-10:148 now explicitly binds this call to TASK-056. Source path census was correct; the original owner conclusion was false and is withdrawn. Structural lesson: derive application ownership from the complete source trajectory and stage contract, never filename affinity.

## Exact checkpoint

| File | Lines | SHA-256 |
| --- | ---: | --- |
| docs/refactoring-plan/evidence/assemble-plan.py | 93 | e9d1a953bec7ee17cdeb5867388e1588eec2a067e818258ec3f9d5b172ef63fb |
| docs/refactoring-plan/architecture-adjudication.md | 160 | f183d42669696b4a7c184838e26537ef855d594746d19f98fe54baec1a6775b2 |

The normalized disposable task-index digest is `8c3a1f2c469816efcb32736edc6bb1642b9fc5f41b5897edca355fc322251473`. This identifies the tested copy, not a promise that the concurrently edited live plan has that digest. No input capture, golden blessing, branch change, commit or publication occurred.
