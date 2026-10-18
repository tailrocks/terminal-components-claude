# Holla forward-port disposition audit

Pinned MAIN_BASE c12cad8728755cd2d03eefdd8e02891143fca86d; HOLLA_REFERENCE 794b095c196562d38f1b6f7ce379c128af2a023d. Read-only source audit; external evidence only. No source changes, tests, captures, or visual approvals claimed.

## Authority correction

SUPERSEDED: holla-project/notes/main-vs-holla-refactor-analysis.md:13 says “Do not merge main’s code; use it as reference text” and final merge replaces the tree. HOLLA_REFACTOR_RECOVERY_PLAN.md:1181 starts implementation from accepted Holla; its holla-recovery-line instructions and main-salvage direction are likewise superseded. Main is implementation base, Holla product reference. Preserve main workspace/public API and valid fixes; forward-port Holla into apps/holla. Historical absolute paths, three-app scope, and Claude/Fable/Opus/Kimi routing are obsolete. Findings remain investigation leads, never proof of current test results.

## Exact inventory artifacts

`disposition.tsv`: every net branch-only path, source blob, SHA256, destination and disposition. `all-touched-paths.txt`: union of individual commit paths, including transient paths. `commit-index.tsv`: 12 immutable commits and patch hashes. Each commit has a .patch export excluding binary/capture bulk; assets are separately hashed in disposition.tsv. `inventory.json`: actual asset/test counts and missing artifact types. `tests.tsv`: every source-declared Holla test identity; declaration inventory is not executed-test evidence. `controls.tsv`: named legacy control namespaces. `legacy-imports.tsv`: every explicit old-library use statement. Both DESIGN snapshots and exact divergence exported.

## Patch semantics

49961e0 adds Holla goal/prompt and agent files; deletes FEEDBACK, GOAL and JACKIN_REFERENCE. Preserve original obligations through main/history; do not propagate those deletions as product authority. 93d840f and 26ed5d8 adjust historical agent models/endpoints: archive, no execution constraint. 0f8e8fc adds complete Holla app/domain/sim/screens/tests and root binary target: forward-port content, replace old UI/runtime ownership. 58fea3e adds bold warning ▲ alongside error ! to shared status renderer; adapt shared component, strengthen actual four-color and modifier tests (old new test uses Junie default despite mono-oriented name). 94a3830 adds README and DESIGN product conventions. 11f0599 adds interaction alternatives, decisions, design note: retain traceability. 5512ad6 adds configurable capture matrix; retain namespace and repair fail-open behavior. 0f974bf adds actual capture sets: immutable historical evidence, not automatically newly pinned acceptance. 4a46bca adds scenario verification walkthrough and README link. 03a2bb5 adds historical breakage/recovery reports: keep findings, supersede implementation direction. 794b095 untracks DS_Store and ignores it: retain hygiene.

## Eleven scenario contracts

| Scenario | Required production state/journey |
|---|---|
| first-use | Empty local directory; progressive discovery; always-armed query; first-frame hints |
| rust-dirty | Dirty git worktree, behind branch; ranked suggestions/reasons; top-match simulated run |
| monorepo-root | Bounded children; update-all DAG; typed target phrase; billing divergence vs legacy fast-forward |
| monorepo-child | cwd primary; parent tasks visible with workspace scope tags |
| docker-cleanup | Blocked rows explain why; cleanup review/exclusion/dependency changes; effects match successful steps |
| disk-cleanup | Freshness labels; active today policy protected; only successful reclaim candidates removed |
| upgrade-plan | Debian remote host + global mise; parallel upgrade branches; held/reboot truth |
| activities-multi | Running/waiting/succeeded/failed/detached glyph+tone strip; global Ctrl+A; numbered activity routes |
| remote-host | Resolved SSH chain/identity filename; production host context on previews and quit |
| launch-failure | Seeded failed activity visible immediately; failure never success |
| hard-cases | Detached HEAD; docker discovery failure; missing SSH identity; long labels/breadcrumb; narrow layouts |

All scenarios: Home/Plan/Activity routes; header/menu/brand/breadcrumb/host/strip/footer; minimum 72x20; 80x24,100x30,120x40,160x50; truecolor/256/16/mono; full/reduced/paused clocks. Historical P6 is 11x4x3=132 excluding ANSI16 and interaction states.

## Control and behavior inventory

Always-armed query: type/paste/backspace, arrows, Enter top match, Esc clear/widen/return ladder; q asks quit only at empty query, typed q filters. Ctrl+S cycles Here/Project/Workspace/Host/Personal, Ctrl+P preview, Ctrl+O alternatives. Pin here, alias prompt/expansion, hide here, exact-command resurfacing, reset ranking. Priority sections Suggested here/top four, Recent here, Explore retain reasons and scope tags. Actions distinguish Ready, NeedsTrust(path), Blocked(reason), and ReadOnly/Bounded/Broad risk.

Menus: brand About/Quit, File/Go/Help host menus, F10, pointer labels/rows/outside dismissal. F1/question-mark help. Dialogs: preview facts, mise trust, clone picker and argument/review, alias prompt, PG blocker tree/cancel/terminate with revalidation, monitor snapshot/btm handoff, SSH resolution chain, quit naming production identity. Preserve nested modal focus restoration, background barriers, drag/wheel ownership.

Plan: review DAG, optional exclusion via Space, required/policy-skipped refusal, live dependent explanation, parallel branches, typed phrase second gate, run/back buttons, truthful per-step output/result, failed step skips dependents, cancellation has no effect. Activity: named identity retained across route changes, Ctrl+A globally, 1–9 on activity route, 0 home, per-activity scroll, merged logs retain service identity.

## Domain/effect boundaries

Keep app-side: action/scope/risk/availability/ranking; activity; host and production identity; mise tools/tasks/trust; git repositories/primary-branch resolution/children; GitHub repositories/clone arguments; SSH resolution policy/filename only; Docker containers/cache/reclaim; disk candidates/freshness; Debian pending/security/held/reboot; PG sessions/blockers; plan DAG and outcomes; fixture world/discovery.

Explicit PlanEffect variants: RestartContainer(name), DockerCleanup, DiskReclaim, DebianUpgraded, GitSync. sim/plans.rs apply_effect uses actual succeeded steps to update world. Also characterize one-shot simulated runs, activity starts, trust, memory pin/alias/hide/reset, PG cancel/terminate, clipboard request. Never implement shell git/docker/ssh/mise/btm/pg_activity/fs mutations. Add instrumented effect sink/spawn guard and adversarial secrets tests; source absence of process calls alone is weaker evidence.

## Public API dependencies / required sequence

1. Shared runtime owns focus/ring, hit geometry, hover suppression, pressed/flash/capture, modal layers, cursor, input routing, monotonic clock/scheduler. Old app.rs owns all of these; do not transplant them as a second runtime.
2. Caller-owned short-lived props and typed Response actions; public custom author boundary supports exact query row, sectioned result anatomy, state-toned activity strip, plan rows, log output. Product drawing remains app-side; no legacy RenderCtx or widget imports.
3. Shared input/button/dialog/picker/menu/props/segments/hintbar/scroll primitives expose required parts, states, widths, geometry and renderer hooks. Map HintBar precedence and right status; global warning glyph belongs shared keyhint. Dialog content-driven widths and typed-ack validation cannot regress.
4. Domain plan.rs currently imports legacy widgets::props::Prop and builds visual facts. Move visual fact adaptation to screens; keep plan domain plain data.
5. Replace positional ROWS.child(n) identity in home.rs (focused_action lines 78–83, render around 1080) with stable Action.id. Sorting/filtering/discovery/ranking currently can change the entity behind position. Retain logical subject through modal return with stable keys, not row_of indexes. This is source-proven reference architecture defect; narrow identity correction needs regression tests and review.
6. Old Clock advances tick interval (80/200ms) per Input::Tick. Preserve paused --frame fixture semantics, but adapt live scheduling to actual monotonic elapsed time. Rendering must not advance clock or perform effect. Test discovery thresholds 600/900/1200/1600/2400/2800/3200ms and status/flash deadlines explicitly.
7. Register apps/holla exactly once; retain CLI -c/-s/-m/-f aliases and color aliases 24bit/mono, FirstUse default, help exit 0 before raw mode, malformed known-option exit 2, explicit motion precedence. HOLLA_NO_MOTION absent/empty/0 => Full, any other nonempty => Reduced. Unknown arguments currently ignored: characterize and preserve unless documented narrow correction.
8. Migrate all named tests to actual Cargo binary targets, then production canonical-cell/cursor views, in-process real runtime traces, terminal-process tests. Old text-only H harness and find() coordinates do not prove colors/geometry. Do not bless candidate as expected.

## Capture defects requiring repair

p6_matrix.sh sets -uo pipefail without -e, checks only shot failures, continues after start/resize failures, does not return fail variable as exit code, hardcodes target/debug/holla and /usr/bin/python3, and counts existing glob outputs (stale output can look successful). New runner must isolate sessions/output/binary, propagate every failure, enumerate exact 132 identities, record source/toolchain/binary/environment/clock/terminal engine/capability/font/hash, verify all 5 artifacts, and add ANSI16 plus behavior-state coverage. Original shots stay immutable.

## Evidence boundary

Full path/hash and commit inventories are exhaustive. Semantic review covers public contracts, source-proven migration dependencies, all small non-asset patch intents and selected application/control/domain implementations. The 8,600-line initial Holla implementation, deleted historical prose, and complete 1,685-line recovery prose have NOT received exhaustive line-by-line semantic review in this bounded pass; patch exports preserve the remaining review input. No image inspection, live reference run, independent behavior comparison, or safe-effect proof occurred here. Do not label those requirements complete from this report.
