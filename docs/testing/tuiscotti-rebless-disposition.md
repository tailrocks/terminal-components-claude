# Tuiscotti re-bless disposition (phase3f)

Worktree `tc-tuiscotti-visbase` at `d63f7f61`, tuiscotti `a47c9aa`.
Blessed 7400/7550 combos (`.ansi/.html/.png/.txt`); 150 combos in 6 roots
left on stale approvals (harness input errors, no verdicts possible).

## Cohorts (per phase3d review section 5)

- Render-only 6801: pixel/encoder-only drift, cells identical. Bless.
- S-584 (S-575 renderer-format + T1-small 8 + T2 residual): actual-only SGR
  sequences all carry `;58` (underline color) and strip to approved members;
  zero glyph change (txt_match=true). Bless as encoder-fidelity upgrade.
- T1 `tablepro/connections/form_advanced` 25 combos (15 wide txt-differ +
  8 narrow ansi-only + 2 matching no-ops): stale approvals. Cause:
  product change `892186260` (sidebar now always painted); harness ruled
  out (no scenario/driver changes in range). Documented re-approval. Bless.
- T2 `tablepro/query/error/160x50/16`: content already approved (2/2
  normal-mode re-captures byte-identical txt); only S-class ansi bytes
  blessed with S. No separate content approval.

## Not blessed (stale, 150 combos)

`holla/fade/cleanup_list_wheel`, `holla/flows/disk/tree_selected`,
`holla/flows/disk/tree_unfolded`, `tablepro/table/filter_editor`,
`tablepro/table/filtered`, `tablepro/table/sorted-filtered` — each test
errors in the harness input layer before capture (`support.rs:859` bad
chord / `:914`), producing zero verdicts. Evidence: gap-rerun log.
Unblocks on the harness typed-input repairs (impl side); then capture
and bless these roots in a follow-up.

## Green definition (post-bless full ignored run)

`cd tests/harness && cargo nextest run --run-ignored only
-E 'binary(visual_baseline)'`: every snapshot test PASSES (fresh capture
matches blessed bytes) EXCEPT the 6 listed roots' tests, which fail with
pre-capture harness input errors and produce no verdicts. Zero
pixels-differ/cells-differ failures; zero txt/ansi mismatches.
Non-snapshot `control_states::toosmall_notice_all_apps` is excluded: it
fails on a settle-wait timeout independent of snapshots (pre-existing).
Known flake: `tablepro_query_results` intermittently fails 2-4 combos per
run on the pre-comparison scenario wait `` `wait:Ctrl+X Explain` timed
out`` (15s); the failing subset moves between runs and every one of the
25 combos matched its blessed bytes on retry (23/25, 22/25, 21/25 matched
across three post-bless runs; zero mismatches). Retry the test until
green; a green run must show zero `pixels-differ`/`cells-differ` and zero
txt/ansi mismatches — wait-timeout-only failures are the flake, any
mismatch is a bless defect.
