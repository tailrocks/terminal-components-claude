# Test disposition (TASK-008 product `test-disposition`)

`disposition.py` dispositions the reconciled TASK-007 test inventory against
the frozen oracle (`visual-baseline` tag commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) and the TASK-006 component
coverage. Every verdict is hand-reviewed and evidenced; nothing is deleted
or ignored because it fails. A migrated test that fails after receiving
oracle assertions is a future-owner diagnostic: it stays visibly failed
until its stage-map owner closes it.

Requirements: Python 3.11+, Git. No third-party packages.

```sh
python3 tools/test-disposition/disposition.py decide \
  --root "$PWD" --verdicts tools/test-disposition/verdicts.json \
  --stage tools/test-disposition/stage-map.json \
  --manifest tools/test-disposition/patch-manifest.json \
  --conflicts tools/test-inventory/conflicts.json \
  --listing tools/test-inventory/listing.json \
  --output /external/evidence/dispositions.json
python3 tools/test-disposition/disposition.py apply \
  --root "$PWD" --manifest tools/test-disposition/patch-manifest.json
python3 tools/test-disposition/disposition.py verify \
  --root "$PWD" --manifest tools/test-disposition/patch-manifest.json \
  --archive archives/test-authority
cd tools/test-disposition && python3 -m unittest -v test_disposition.py
```

`decide` cross-checks the hand verdicts against the accepted inventory and
emits the reviewable record: every TASK-007 conflict has exactly one
decision; every non-preserve span is implemented by exactly one manifest
patch; every decision identity exists in the reconciled listing; every
rename records its relocation; every stage entry names an exact closing
task. `dispositions.json` in this directory is the committed output for the
recorded parent (5,925 listing identities: 15 hand-decided, the rest
bulk-preserved by rule, zero deleted).

`apply` reproduces the candidate from the recorded parent mechanically: each
old span must match byte-for-byte exactly once at its recorded offset and
SHA-256 before replacement. Ambiguity, drift, overlap, or an already-applied
tree is a hard refusal, never a fuzzy match.

`verify` re-derives each changed file from the immutable
`archives/test-authority` originals plus exactly the manifest spans and
fails on any outside-span byte difference, archive mutation, or file
changed outside the manifest plus this task's own product directories.

## Verdicts

| ID | Test | Verdict | Owner |
| --- | --- | --- | --- |
| V-001 | holla `home_hint_casing_preserves_lowercase_physical_shortcuts` | replace (oracle hints, scope pill, Ctrl+Q quit) | TASK-041 (+040) |
| V-002 | holla `home_escape_clears_scope_before_canonical_query` | relocate to `home_escape_clears_query_before_scope` | TASK-041 |
| V-003 | holla `scenario::tests::names_round_trip` | replace (world count 11 → 34) | TASK-049 (witness 050) |
| V-004–V-009 | tui `keyboard_editor` (6 tests) | preserve (ownership/safety; coarse flag reviewed) | — |
| V-010–V-011 | tui `viewport` (2 tests) | preserve (lifecycle; coarse flag reviewed) | — |
| V-012 | viewport `retention_fixes_up_selection_and_caret` | replace (evicted caret → None) | TASK-021 |
| V-013 | viewport `the_text_width_…` | relocate (scrollbar only on overflow) | TASK-021 |
| V-014 | viewport `the_focus_gutter_…` | relocate (no gutter; origin text) | TASK-021 |
| V-015 | viewport `tabs_expand_…` | relocate (copy source tab byte) | TASK-021 |

File-level `reviews[]` in `verdicts.json` preserve the 10 code.rs, 6 diff.rs
and 25 remaining viewport.rs inline tests with per-test rationale and
negative-identifier sweeps, and reconcile the R-002 named items (Holla
Ctrl+A/scope-first/frame-ms, old22-pageShowcase, TablePro
Surface/confirmation, Meter pins → TASK-029). The W-021-07, W-025-07/08 and
W-026-05 conflicting observations that have no current-tree test pin are
recorded as unborn `completion_021/025/026.rs` modules in `stage-map.json`,
owned by TASK-021/025/026.

## Failing after migration (expected)

Seven migrated tests fail until their owners restore oracle behavior:
the three holla tests above plus the four viewport tests. Each failure is
recorded in `stage-map.json` with its exact closing task and closing
condition. No other test may change outcome: the engine's `verify` plus
`cargo nextest` for the touched packages prove it.
