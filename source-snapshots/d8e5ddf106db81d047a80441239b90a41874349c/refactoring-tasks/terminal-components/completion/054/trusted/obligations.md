# TASK-054 trusted obligations

## Fixed ADJ-16 Steps consumer

ADJ-16 is binding for oracle src/bin/jackin_preview/screens/cockpit.rs121: use shared Steps::passive(id).presentation(StepsPresentation::Rail) with the existing stable keys, borrowed lifecycle/row/meta data and source default numbering. Preserve custom meta Some(text)/Some(empty)/None, all lifecycle/spinner frames, exact ordinal/meta geometry, scroll/fade and narrow allocation. Passive retains wheel/scrollbar but no row focus/hit/hover/action; do not substitute disabled or existing ClickOnly new. No app-owned ordinal/meta/glyph rail painter or invented activation callback. Domain stage/timer/frontier data stays in this app. Bind R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005 to exact current task-owned component contributions and intact source scenarios under the existing ownership map; no future closure receipt is required. Baseline006 freezes these policy branches independently before candidate implementation.

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/jackin_preview/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-004/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Restore Cockpit stage frontier/status, cached/skipped states, build-log and information/credential overlays, effective-account resolution, recoverable/error branches, cancellation/detach and handoff into Capsule.

Use shared Steps, Progress/Spinner, TextViewport, picker/dialog and explicit runtime clocks. Keep launch plan/frontier, provider simulation and entry/exit arbitration application-owned; modal input cannot fall through to launch actions.

- All eleven stage changes and exact tick boundaries are observed; Input::Tick triggers update while accepted Moment owns elapsed time.
- CredentialsLocked and BlockedSidecar retain their honest fixture classifications. BlockedSidecar is modeled-only and must not be claimed as a CLI-reachable PTY state.
- Launch failure with another running instance and final-instance failure are separate lifecycle branches; late jobs cannot fabricate a successful launch after cancellation.

## Exact membership and staging

Primary complete scenario IDs: JA-011, JA-041, JA-042.

All primary scenarios require every checkpoint. Dependency or scenario routing cannot be relaxed by the executor. A closed checkpoint is monotonic even while another checkpoint in its scenario remains future-owned.

The baseline expansion algorithms and finite axes in jackin.md and jackin-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Resolve all eight worlds, Full/Reduced/Paused, fixed seed 0x4A41434B494E5E5E and epoch 1788401640; the ordinary CLI cannot be credited with modeled-only state observations. JA-065 uses last reachable tick before/at expiry plus separate precise interaction-clock proof. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires JA-040, JA-043, JA-044, JA-045 through the exact frame and semantic flow tables. Every selected checkpoint is a complete frame. Semantic assertions have separate fixed fields/types and never claim cell equality. Complete parent ownership is fixed in app-flow-stage-audit.tsv. Read app-flow-contribution-contract.md for direct seed derivation and mandatory independent equivalence qualification. App closure reruns every app contribution and every full original parent.

## Source scenario clauses

### JA-011

- world: returning
- sizes: 80x24,120x40
- actions: replay(manager_launch_picker_hides_agents_without_an_account); variant choose agent with one account,with multiple accounts,with no valid account; Escape each nested picker
- checkpoints: Unavailable agent filtering, account defaults/preferences, cancellation keeps selection and no launch effect
- components: picker,focus,overlay
- source: app_tests.rs:1110;domain/workspace.rs

### JA-040

- world: launch-running
- sizes: 80x24,100x30,120x40,160x50
- actions: fresh(launch-running,Full,0); T(1) repeated through deterministic launch completion; checkpoint each of eleven Stage changes and cached/skipped state; T(12) handoff
- checkpoints: All stage frontier/status/activity/account lines, build progress, handoff phase and Capsule arrival
- components: steps,progress,spinner,scheduler
- source: sim/launch.rs:10,241;screens/cockpit.rs;app_tests.rs:179

### JA-041

- world: launch-running
- sizes: 80x24,120x40
- actions: fresh(launch-running,Full,0); T(40); K(b,PageUp,Home,PageDown,End,Esc); click(build log); wheel(log,100); wheel(log,-100); K(i,Esc,c,Esc)
- checkpoints: Build log overlay full scroll range, at-tail behavior, info/credential views; modal input never launches/cancels underneath
- components: log viewport,info,scrollbar,overlay
- source: screens/cockpit.rs:575;app_tests.rs:646

### JA-042

- world: launch-failure
- sizes: 80x24,100x30,120x40,160x50
- actions: replay(launch_failure_returns_to_the_construct_when_another_instance_runs); separate variant no other instance remains
- checkpoints: Network failure details and acknowledge; nonfinal manager status vs final lifecycle; no successful launch effect
- components: error dialog,steps,arbiter
- source: app_tests.rs:203;sim/launch.rs:79

### JA-043

- world: launch-running with CredentialsLocked,BlockedSidecar
- sizes: 80x24,120x40
- actions: fixture LaunchPlan=CredentialsLocked; advance to block; resolve unlock/retry; advance completion; reset LaunchPlan=BlockedSidecar; advance block; K(Enter); reset K(Esc)
- checkpoints: Recoverable credentials/error frontier; retry clears hold; modeled-only blocked sidecar status and dismissal
- components: steps,picker,dialog
- source: sim/launch.rs:79,413;screens/cockpit.rs:641

### JA-044

- world: launch-running
- sizes: 80x24,120x40
- actions: variant K(Ctrl-C),K(Ctrl-Q,Esc),K(Ctrl-Q,Right,Enter),K(d); advance deterministic ticks
- checkpoints: Cancel/detach/quit confirmation; state/effects and entry ownership match oracle; build stops or persists exactly
- components: dialog,async ownership,arbiter
- source: screens/cockpit.rs:575

### JA-045

- world: returning
- sizes: 80x24,120x40
- actions: replay(cockpit_resolves_every_effective_account_for_the_container)
- checkpoints: All effective accounts resolved, preferred launches, runtime auth sources and per-account failures; no secret rendering
- components: credentials,steps,status
- source: app_tests.rs:1201

## Regression and trust closure

Run every required source-qualified Jackin and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; task-owned and previously closed identities must all pass. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
