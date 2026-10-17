> **Status 2026-10-18: R-apps-7 and R-apps-14 landed; R-apps-3 open** on `origin/wip/rapps-3-7-14` at `81be99a79` (not gated). The scripts named below are in `docs/handoff/scripts/`. Next steps: [../README.md](../README.md).

# R-apps-3 / 7 / 14 handoff (agent report, 2026-10-18)

Done and pushed (design/interaction-polish ed65b5b31..5629775d9):
- R-apps-14: a2f397f1b fix (Dialog code preview joins body scroll, CODE_LINES/MIN_FACTS gone), 3144afba9 snapshots (400 captures attributed). Trade-off: on short screens a write gate's SQL is below the facts, needs a scroll.
- R-apps-7: 6e19572c0 fix (button::row_layout / row_layout_right wrap; new button::row_lines), 5629775d9 snapshots (45). Decision recorded in responsive.md: wrap, not a `…` menu (WAI-ARIA toolbar order, macOS dialogs grow). Open: tablepro editor toolbar (tabs.rs) should move to widgets::toolbar::Toolbar; holla activity and plan "Next" follow-up rows duplicate a pattern and should become one library row. holla activity-failed 72x20/80x24 "Node here project" re-filed as R-apps-4.
- Note: 3-line edit in src/bin/showcase/pages/terminal.rs (Fail button reserved line).

R-apps-3: WIP commit 81be99a79 on origin wip/rapps-3-7-14, NOT gated.
Contains: Tree/List meta folds after label (muted, cut with `…`, revealed) in src/widgets/tree.rs, list.rs; new ui::columns::dropped_text + reveal_dropped (catalog, coverage, showcase Columns page); jackin config.rs mounts `…` (replaces `2›` header tag, visible change); holla files.rs reveal::fit + reveal_dropped; holla plan.rs "step" heading in narrow layout; reach_allow R-apps-3 lines removed (tablepro result-plan lines re-filed under R-apps-2); showcase_reach_allow `QueryWorkbench | Playground | 120x40` deleted (may conflict with R42); showcase_baseline regenerated but NOT attributed.
Left:
1. Fix tablepro app_tests::explorer_opens_table_and_grid_navigates (`h.text().contains("organizations")` fails; likely tree label_cell width from `avail` with folded meta in narrow sidebar).
2. Verify regenerated tests/showcase_baseline.txt rows are intended.
3. Full gate + visual check-touched, attribute cells, snapshot-only re-approval commit.
4. Mark R-apps-3 done in docs/remaining-work.md and responsive.md.
5. Record decision: hand-rolled tables use shared reveal_dropped, not widgets::table (jackin group headers/change marks, holla row look don't map onto DataTable).
Scripts: scratchpad reach-apps/ gate.sh, accept.sh, cells.py, classify.py (copy into repo with the handoff).
