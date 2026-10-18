# Independent review of parity-plan repairs

Review date: 2026-09-11. Verdict: **PARITY-FINAL-01–04 are resolved in the reviewed planning contracts. No remaining parity-plan finding was identified in this review.** This report reviews the repairs to `review-parity-final.md`. It is not an application parity receipt, an oracle-capture receipt, or approval of a candidate implementation. No production source or authored task contract was changed by this reviewer.

## Authority and review boundary

Independently resolved Git objects:

- Oracle: `02f5294bfdbf38004cc49130d0aff1d01f31434c`.
- Architectural main: `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- Planning checkout HEAD: `2e2401393c47360741ebd321679de08982dca50a`.

Read all 150 Jackin/TablePro parent scenario rows, all 84 separate frame/semantic contribution rows, their stage and seed contracts, actual TASK-004/005 and TASK-051–064 packages, and the application trace fragment. Read the Holla stage and native/resized contracts, all 135 Holla ownership rows and their contribution rows, and relevant TASK-003/040–050 packages. Direct `git show`/`git grep` checks at the oracle corroborated the cross-surface transitions and source-native geometry described below. Source and contract inspection is distinct from executing the later repaired applications.

## Findings and disposition

### PARITY-FINAL-01 — Holla cross-surface ownership

Resolved in the reviewed planning contracts. The repair retains complete parents under concrete downstream producers. `HO-HP02` and `HO-ROUTE-05` close at TASK-044; `HO-ARGS` and `HO-GATES` at TASK-048; `HO-TRUST` at TASK-049. The earlier Finder/arguments/gate tasks have explicit nonempty complete-frame contributions and separate actual-state observations. None closes its parent from a state-only result or an empty preservation intersection.

The audit also follows less obvious routes: HP01 opens Config; file actions create an Activity or open Disk; HP14 constructs a second Git-batch world; HP19 opens Trash review; LaunchFailure follows into the Port5173 snapshot. Relevant direct checks include oracle `app_tests_flows.rs:24,310,576,601`, `app_tests_parity.rs:1731,2528`, `screens/plan.rs:646,674`, and `screens/disk.rs:1006,1183,1230`. The allowed Disk SWEEP inputs do not include the deletion chord; plan-review confirmation opens its gate/dialog and does not itself accept execution.

Two early-proof hazards were corrected during review. TASK-045 now explicitly owns the existing read-only path validator/canonicalizer used by HP23 platform assertions; TASK-046 reuses that validator for accepted cleanup and owns effects/reports. The later HP17 failed-save branch follows an already-created Activity tab, so its frames were removed from the early TASK-043 contribution while its semantic assertion and complete parent frames remain required. The initial changed-byte-refusal prefix remains nonempty.

The final author follow-up also found that `HO-ACTIVITIES` sweeps the `Review 4 modified files` button. Independently confirmed oracle `domain/fixtures.rs:2438` and `screens/activity.rs:292–296`: activation opens `git.review`. Complete ownership now belongs to TASK-047. `HO-044-F05` retains the full original native route and complete Activity SWEEP checkpoints before that foreign-route activation. The plain native `HO-ROUTE-04` remains TASK-044 because it never activates that button. The final audit contains 135 parents, 25 ownership changes and 23 nonempty contributions; the package/trace joins match those identities.

### PARITY-FINAL-02 — Jackin launch and nested configuration flows

Resolved in the reviewed planning contracts. JA-022 and JA-028 close after the picker/Accounts producer TASK-053. JA-011 closes at TASK-054. JA-040 and JA-043–045 close after Capsule producer TASK-055; JA-056 and JA-058 close after dirty-exit/Inspect producer TASK-056. The 9 earlier Jackin contributions each have a separate complete-frame and semantic identity.

Oracle `app_tests.rs:179` enters Capsule and checks pane echo; `:1201` checks effective accounts before finishing in Capsule. The TASK-054 frame selectors stop before Capsule. TASK-055's JA-058 selector stops before the first Ctrl-Q, preserving the original dirty-exit/outro assertions for TASK-056. Manager/Prelude/intro/outro composition remains explicitly upstream in TASK-051; its limited launch-picker behavior does not imply completion of later account eligibility or launch execution.

### PARITY-FINAL-03 — TablePro query, switcher and History dependencies

Resolved in the reviewed planning contracts. Oracle `app.rs:176–185` constructs Workbench and immediately calls `new_query("")`; connection completion is not an empty Workbench. Accordingly TP-005/016/017–021/026 complete at TASK-061. TP-046/050 and all full parents requiring the T quick-switcher prefix complete at TASK-063. All original prefix, query, History and final checkpoints remain required.

TASK-059's empty-Workbench fixtures derive from intact TP-026 `cp(empty)`, then freeze their own source focus cycle and coordinates. The T-based Grid/safety contributions are independently qualified direct state fixtures. The contract requires exact full frame, cursor and complete continuation-state equality with the intact oracle prefix, then identical frozen suffix execution. It retains App and Workbench state, Query1, query counter, focus, modal state, time and nested concrete state; it forbids surrogate rendering or copying expected cells. These fixtures receive no PTY or CLI reachability claim. TP-037, TP-077 and TP-079 name their additional source-derived seed boundaries explicitly. Full TASK-063 parents must still execute their actual switcher/help inputs.

The two stale semantic sentences assigning TP-005 `cp(connected)` and TP-016 `cp(workbench)` to TASK-059 were reported and corrected to TASK-061. Actual source checks corroborated `App::connect`, connection tick dispatch at `app.rs:335`, Workbench construction and query/History ownership. No cropped frame or relaxed parent was introduced by these repairs.

### PARITY-FINAL-04 — Native assertions versus resized capture

Resolved in the reviewed planning contracts. Product A executes unchanged source-native tests with their original constructors, coordinates, explicit resizes, mutations and assertions. Product B separately materializes oracle-derived per-size action/frame programs. A protected bidirectional source-site map distinguishes unchanged domain assertions, source-relative geometry, source-guarded layout applicability and exact pointer-owner remapping. The candidate cannot write the map, choose coordinates or drop a failed assertion.

The concrete contract retains Holla's native row38/row28/row22 assertions and native `(3,38)` status-path click. It preserves explicit resize events and freezes the same semantic owner for resized captures, including field-relative argument clicks. All per-size captures remain complete and unmasked.

Independently checked all 986 inventoried source sites against the pinned source: 986 distinct IDs across 5 files, with zero source SHA-256 or source-line mismatches. The inventory contains 746 assertion-only sites, 19 assertion/input sites, 139 spatial/input sites and 82 constructor sites. This verifies the supplied inventory's identity; the contract still requires qualified source traversal to detect additional sites and materialize the complete accepted map.

`runner-bootstrap-extensions-protocol.md:73–83` and `runner-bootstrap-native.py` explicitly distinguish the tiny native/resized qualification fixture from actual Holla execution. The fixture's pointer `(2,height-2)` is not claimed to be the real Holla status-path pointer. This review does not claim that the fixture executed all 986 Holla sites, that all native tests were rerun here, or that real Holla golden frames already exist.

## Independent mechanical checks

- All 150 Jackin/TablePro parent IDs are unique and match their complete-primary obligation lists. Every listed application producer is either that owner or a transitive ancestor in actual `task.toml` dependencies.
- All 42 frame and 42 semantic contribution rows contain every required field, require closure, and precede their complete parent owner through the actual dependency graph. All contribution source references resolve to 16 real pinned source objects.
- All 125 concrete TablePro contribution checkpoint-label occurrences resolve to their original parent labels or the explicitly named TP-026 empty-state fixture.
- All 84 APP-FLOW identities have exactly four trace roles: primary, baseline, application closure and final integration. All 336 exact source/requirement/acceptance/check tuples occur in the corresponding protected source-obligation payloads.
- TASK-004/005 and TASK-051–064 copies match global table rows exactly: 1,350 copied stage rows, 168 frame rows and 168 semantic rows. TASK-069's host-bound complete campaign catalog supplies final union membership; its integration tuples are present in protected source obligations.
- The source-object audit identified two citation-normalization defects, now corrected: JA-047 names actual `screens/capsule.rs:1836` instead of a nonexistent `on_tick` file; JA-067 labels `shots/audit/jackin-*` as an artifact pattern rather than a Git source object. Neither correction changes parent actions or acceptance.

The final catalog join, historical-test relocation, bootstrap sealing and runtime qualification are separate acceptance work. This report does not replace those checks or convert diagnostic failures into application passes.

## Reviewed snapshot hashes

These SHA-256 values were recorded after the final author handoff and source-citation corrections. They identify observed planning files, not an atomic Git tree or execution receipt. Paths are relative to `docs/refactoring-plan/`.

| File | SHA-256 |
| --- | --- |
| `app-flow-stage-audit.tsv` | `e0fdeb97a4e8a393aa68393ddb3b647af372e8d976f0f395c8c2a64372af929e` |
| `app-flow-contributions.tsv` | `b21362fcc94ae5f4277fa7e4197ddab223ba068f4581526e1f2fb97cbdbcfa2f` |
| `app-flow-frame-contributions.tsv` | `5e29488474c8e3752968a130e25a4f6c5a116a6d859983ed8bfd4dae0ba933c1` |
| `app-flow-contribution-contract.md` | `3e2ecb1b2bce8b2a6fc159283e3771b5f198812e97d4f6555958d857fd86deaa` |
| `holla-stage-audit.tsv` | `f5c33a9579d6a4b3062c98cd47a45f1ceb63008f6b79cb1370e3bafb0980cf5e` |
| `holla-stage-contributions.tsv` | `700107ac6f3548c9c5765350f500241d4b6b128c483ac8421a1aff9f803b5a80` |
| `holla-stage-contract.md` | `2d8dd4c203d8b49ffec5b744e1aa8783fa841a2c6a83e08aec30183b57596cb1` |
| `holla-route-expansion.md` | `aab96c612abe05388b08eb1275dc9324cf768dabb69ec67bc14c42b246176550` |
| `holla-route-sites.tsv` | `eb3c6d66bffd441fd61964b41e6c08f0c2db2ba33ec02f0184bde43834aba045` |
| `traceability-jackin-tablepro.tsv` | `8aa13ec847d731407f4884cb69fe90a472c6d8751f89dea8751527eede29006a` |
| `traceability-showcase-holla.tsv` | `24d66d7d8c4df9eccb433e79a72f70bca8f66b7cee4d66f4a7cc49b1e4e7ed74` |
