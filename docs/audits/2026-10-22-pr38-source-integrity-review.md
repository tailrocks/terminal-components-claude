# PR #38 source and test-integrity review

## Scope and limits

Reviewed source at PR #38 commit `45b8a8020663df109d71d84f39f553836d1a0b82`.
This is a source and evidence-integrity review only. No Rust tests, builds, visual
replays, acceptance gates, or remote-census checks were run for this report. The
3,000,000,000,000-byte free-space gate remains unmet; the coordinator's latest
reported measurement was approximately 2.893 TB. The detailed operator evidence
paths cited by earlier migration entries are external to this sparse worktree and
were not independently re-opened here. Their prior findings must not be described
as newly revalidated or as currently recoverable.

The frozen references remain the recorded PR #26 commit
`f4867e9b5f8a08a3bb61ce80d644198a29f51984`, baseline commit
`4169783ae883788186d161af09ba9c481bd84e5c`, and baseline tag object
`2718a6b8127a522a3d351d0b4bdb7542fc8da542`. No reference revision change is
proposed.

## TablePro and Jackin source findings

The TablePro captured-frame bypasses introduced by commits
`6923311119e060176ee3cf07c585416556416024` and
`51046a780fafe61bf10ed2dc9d2df4c1eeffa734` are absent from production source at
the reviewed head. Those commits' `historical_picker_open.rs` and
`historical_tablist_open.rs` parsed `snapshots/tablepro/.../*.ansi` into fixed
`RUNS` and painted those cells. The current source has no `historical_*` modules,
captured snapshot includes, or fixed `RUNS` tables under `crates/tablepro-ui/src`.
Retain the real shared-picker adapters and typed state:

- `crates/tablepro-ui/src/quick_switcher.rs` builds a public `Picker` from live
  workbench search results and stable typed targets.
- `crates/tablepro-ui/src/tab_list.rs` uses stable tab keys and current tab
  records for labels, detail, and selected state.
- `crates/tablepro-ui/src/safe_mode.rs` builds picker rows from the typed
  `SafeMode` values.
- Keep the associated picker contract tests, including changed and multi-entity
  data cases. Add any missing empty, long-content, adjacent-size, and resize cases
  before removing or weakening a behavior check.

Jackin still has production false-parity paths. `crates/jackin-preview-app/src/app.rs`
selects a paused cockpit using `Scenario::LaunchRunning` and exact frame tick 40
(around `launch_cockpit_scene`, lines 7260-7275), then invokes
`app/launch_cockpit.rs` from the draw path (around lines 10668-10675). That module
describes itself as the non-120x40 alternative to a presentation blit and embeds
fixed workspace, agent, account, credential, and spinner values before painting a
buffer. Keep simulator fixtures as demo data; replace the presentation path with
shared components consuming typed live view data, and let responsive layout—not a
captured tick—choose the arrangement.

`app/launch_dialog_plate.rs` calls Termrock's info and cancel cards and then
recolors/reshapes their rendered cells. `app.rs` invokes it for reduced-color
states (around lines 10773-10810). This is application-owned correction painting.
Move the required color-pair semantics into shared card/theme APIs, add capability
tests for truecolor, 256, 16, mono, and `NO_COLOR`, then remove the plate.

Other render selectors still combine exact sizes, theme/accent identity, scenario
labels, and transient state. Examples include `launch_failure_frame`
(`app.rs`, around lines 7246-7258), manager picker selectors (around lines
9952-10008), and capsule zoom/menu/palette frame selectors (around lines
8213-8324). Some size-based breakpoints are legitimate, but these compound
conditions produce scenario-specific reference frames rather than one component
contract. Convert each flow to shared blocks/overlays with stable IDs and the same
update/draw configuration. Preserve the user-visible flow and use scenario values
only as input data. Cover each flow at adjacent sizes and with changed data/themes.

`manager_locked_view.rs` gates a hard-cases paused manager at one fixed workspace
row and paints fixed session-chrome strings; `manager_frame_view.rs` also gates on
fixture and paused state, though it derives some leaf status from current world
data. These should become shared manager/shell compositions receiving live typed
view models. Keep its height-based row capacity as layout policy. The main app
draw path still manually carves header/content/footer regions and paints status
and hint bars (`app.rs`, around lines 10624-10667), so the application composition
boundary remains incomplete.

The named historical Jackin hard-launch commit
`f067d3f088621ec2a0fe4e7eaf7ae2c3a7c5957c` added an 899-line captured hard-launch
path. Its old historical module is no longer present, and current picker selection
uses live Termrock Picker behavior. Tests that open the real picker should remain;
cell assertions alone do not prove the application is a thin library consumer.

## Test and evidence integrity

The current TablePro baseline artifact blob is reported as
`3f72cdfb...`, while the migration record previously named `e4c7211e...`.
Candidate-side blessing commits `4fb24d8eae` and `000a510a73` followed restoration
`ef23c3303c`; therefore earlier restoration claims are superseded at this head.
Do not treat the present candidate artifact as a newly verified expected output
or as a reference. Rebind the artifact inventory to the immutable baseline and
rerun the pinned paired oracle after the resource gate is met.

For Holla, expected artifact `75b70...` is reported to be replaced by candidate
commit `7f240b2730` from `a638...`. This remains a provenance/review issue until
immutable reference bytes and a paired run are verified. The palette index 238
matches the frozen reference and must not be reverted merely to match a changed
candidate expectation. The paused tick-80 frame selectors are separately
scenario/tick-conditioned false-parity paths. A failing time-4000 checkpoint is
reported to take the ordinary runtime path; do not infer a defect or pass without
the underlying receipt and exact state comparison.

Showcase source review reports that `LEGACY_NAV` statically paints navigation
while input is also owned elsewhere, creating duplicate rendering/input ownership.
The Buttons scenario's fixture expects Overview while the reference navigation
route is different. The exact visual root cause is not established here. Resolve
the ownership and route mismatch at the shared component/shell layer; retain the
failing evidence and do not update expected output from the candidate.

The oracle remains pending: the full runner rejects `--all`, and the legacy
corpus remains diagnostic-only by default. The branch ledger lists 165 branches,
while local origin refs total 188; neither is a complete current remote-branch
census. No claim is made that all relevant branches were reviewed or consolidated.

## Required next work

After the free-space gate is satisfied and the shared API contract is agreed:

1. Replace Jackin launch, manager, capsule, and reduced-color plate paths by
   shared components in small interaction-contract increments.
2. Keep TablePro's live Picker adapters; verify them across changed/empty data,
   resizing, selection, and close/reopen flows.
3. Reconcile candidate artifact provenance with the fixed baseline before any
   visual comparison. Preserve candidate outputs separately from expected data.
4. Repair Showcase navigation ownership and route expectations from source-level
   interaction evidence, then verify the whole affected flow.
5. Repair the oracle invocation and perform two identical complete paired runs
   against separately built pinned reference and candidate binaries.
6. Reconcile remote branch inventory and review exact final commits with
   independent UI, runtime/API, and test-integrity reviewers.

Until that work and review are complete, exact product match and reusable-library
acceptance remain incomplete.
