# T01 application and route fragment

## Record

- Task: T01 application and route inventory.
- Fragment: app-routes in fragment.json.
- Schema: termrock.pr38.t01.fragment.v1.
- Fixed input: 542349e2f70ec9883d743594f906ac6c4e96ba5a.
- PR26 reference: 5b3dbd7ae799f325efbb4f4cc11b2f5518dec948.
- PR26 source: src/bin/showcase/app.rs, SHA-256 d4c95b87d76320a94bf8fef3d5810014d7027ce4b5649b9875a60be925926bb1.
- Worktree: /Users/donbeave/Downloads/termrock-t01-apps.
- Branch: pr38/T01-apps.
- Owner: /root/t01_app_routes.
- Required selection: gpt-6-luna, max effort.
- Review: independent Sol high review is pending.
- Fragment SHA-256: c63bc59a4d2312aceec26483a49eecbd854635fe559f5dc811cdfa1672b9ad0c.

## API inventory

The inventory covers 22 packages with unique manifest paths. It scans 183 production Rust files.

It records 4,317 declarations, methods, fields, and variants. Each row records its source owner, visibility, reachability, cfg attributes, line, and hash.

Reachable API groups contain 431 types, 156 constructors, 34 builders, 1,183 functions, 1,061 values, 1,056 fields, and 8 aliases. These groups can overlap.

The inventory records 258 public re-exports and 159 public module paths. It preserves nested export destinations and definition owners.

The census finds holla_catalog::ranking::Pin in crates/holla-catalog/src/ranking.rs. It finds holla_plan::plan::PlanStep in crates/holla-plan/src/plan.rs.

It resolves jackin_preview_app::screens::FileBrowserState to crates/jackin-preview-capsule-ui/src/screens/file_browser.rs. It records InputField::text and InputField::choice under holla_domain::args.

Package rows use manifest_path as their identity. Internal unresolved re-exports fail the structural inventory.

## Showcase routes

The fragment preserves all 54 PR26 route identities. Its key set matches the reference PageId enum, NAV_ENTRIES, and factory arms.

The enum spans lines 80–136. The navigation registry spans lines 157–428. The factory spans lines 676–876.

Current tags include 24 Live, one Selection, and 29 Placeholder entries. Five Placeholder entries select stored pages.

Those pages are Checks & radios, Empty & error, Notices, Master-detail, and Diff. Their indexes are 12, 31, 33, 37, and 40.

Each route row records its reference identity, aliases, current tag, target, page object, factory, dispatch paths, shared APIs, owner, task, source spans, and proposed cases.

The other 24 Placeholder entries paint their current navigation labels. They remain T22 registry work. Their planned content tasks remain separate.

Selection uses its own model and paint path. It has no PageId factory.

All 189 case records resolve. They include route behavior, route content, route components, and component API cases. Every case is proposed and NOT VERIFIED.

These cases are not approved T38 plans. T01 source inspection does not prove product behavior.

Current navigation keys derive from entry and group indexes. Alias search checks labels, live page titles or slugs, and aliases.

A later invalid --page keeps the previous valid choice. An invalid first value keeps Overview. App::for_page_name uses Overview for one invalid name.

Unknown CLI options currently fall through without an error. Invalid themes select Junie. Invalid color values have no effect.

The current motion options expose Full and Paused. They do not expose Reduced motion.

The route policy findings record proposed T22 changes for stable route keys, explicit CLI errors, help, and reduced motion.

Task Runner resolves to crates/showcase-demos/src/pages/taskrunner.rs. Its module declaration is in pages/mod.rs.

## Applications and boundaries

The fragment records 21 journeys across Showcase, Holla, Jackin Preview, and TablePro. All journeys remain NOT VERIFIED.

Showcase uses static demo data. Keep application state in its consumer crates.

Holla scans immutable snapshots. It does not read host files or launch processes. Port checks and task runs use simulated state.

Jackin uses a virtual world and scripted PTY state. Keep credentials masked. Repeated draws must not change product state.

TablePro uses in-memory data and simulated SQL. It does not connect to an external database.

Keep SQL policy in TablePro. Keep result grids read-only where the contract requires it.

Current Ctrl+R and F5 run the full editor buffer. They do not run only the selected statement.

T08 records current assembly, data, SQL, and simulation owners. It plans consumer migrations without moving source in T01.

T07 owns Jackin page assembly that currently sits in termrock-feedback.

W46–W63 component references appear where current app or route sources use them. They link to definition owners and named tasks.

W62 AppShell definition ownership belongs to T10. T19 owns shell policy integration.

W41 HintBar stays on StatusHints. W60 DerivedHintBar is a separate route-matrix requirement on StatusHints and ReadOnlyReport.

Jackin presentation prelude and rain globs resolve to their own modules.

## SQL and performance

The current TablePro comparator keeps NULL values last in ascending and descending order. It reverses only non-NULL comparisons.

This is a source fact. It does not prove sorting behavior.

The four historical performance files define Holla, Jackin, Showcase, and TablePro workloads. T01 read these files without running tests.

No measurements ran. measurement_value remains null. T33 owns the open measurements.

## Source audit and commands

The audit checked 288 immutable source blobs across the fixed source, PR26, and historical commits.

It checked 9,481 source references and 1,355 line-bound source locations. Hash, line, reference, and range errors were zero.

The generator read immutable blobs with git show. It did not run Cargo, tests, benchmarks, PTY probes, or T37 cleanup.

| Command or action | Result |
| --- | --- |
| git rev-parse HEAD | Fixed input was 542349e2f70ec9883d743594f906ac6c4e96ba5a. |
| git -C /Users/donbeave/Downloads/termrock-pr26-reference-20261026 rev-parse HEAD | Fixed reference was 5b3dbd7ae799f325efbb4f4cc11b2f5518dec948. |
| python3 /tmp/t01_app_inventory.py | Exit 0. It generated the corrected inventory, case registry, and source audit. |
| python3 -m json.tool docs/implementation/pr38-final/results/t01-app-routes/fragment.json >/dev/null | Exit 0. |
| Read-only Python structural validator | Exit 0. It checked package keys, exports, 54 route keys, app journeys, and 189 case records. |
| Read-only Git source audit | Exit 0. It checked 288 blobs, 9,481 references, and 1,355 source spans. |
| Read-only W62 owner crosswalk check | Exit 0. It assigned T10 to definition ownership and T19 to shell policy behavior. |
| Cargo, builds, tests, benchmarks, PTY probes, and cleanup | Not run. Behavior remains NOT VERIFIED. |

## Open work

APP-HISTORY-ROWS and APP-STALE-HISTORY-ROUTE-NULL remain open. /root/t01_history_requirements owns their reconciliation.

The stale NULL-sort claim must not override current SQL source. Historical rows must remain historical records.

The fragment SHA-256 is c63bc59a4d2312aceec26483a49eecbd854635fe559f5dc811cdfa1672b9ad0c. Independent Sol high review remains pending.
