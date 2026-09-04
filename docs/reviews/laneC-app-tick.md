# Lane C adjudication proposal: Jackin status, ticks, and dimming

**Status:** proposed for Q1, Q2, and Q4. Lane A must record the accepted
decisions in `COMPONENT_ARCHITECTURE.md` and `REFACTORING_STATE.md` before any
library implementation. This review is read-only with respect to `crates/**` and
the legacy Jackin tree.

## Scope and fresh evidence

The review compared `docs/plans/slice7-jackin.md` with the current runtime,
`StatusBar`, Jackin host chrome, six `strip_right` implementations, the rain
cross-fade, and all three unsafe `run_id` derivations. The current inventory is
67 legacy Jackin tests, 36 visual digest cases, and 23 methods on the legacy
`Screen` trait. The 67 tests break down as 28 application/chrome tests, 35
in-module tests, three performance tests, and one visual-baseline test. These
are preservation counts, not permission to collapse coverage during the move.

The current generic runtime exposes no update cause. `Input::Tick` increments
the runtime clock by the theme's uniform `design.motion.tick_ms`, then invokes
the same `App::update` path used by every other event. Its session loop treats
`repaint_after` only as a boolean reason to poll at the theme tick rate; it does
not honor the requested duration as the next deadline. It also draws before it
ever calls `App::update`. Those three facts make the plan's Q2 recommendation
too narrow: an elapsed-time accessor or a tick hook alone would not establish
the first deadline, preserve it across unrelated input, or cover headless use.

## Q1: replace `strip_right` with a pure status projection

Do not cache strip items through `Jx` or populate them as an update side effect.
The right-hand strip is a projection of current screen and world state, and the
existing `strip_right(&self, &World)` contract is pure. Turning it into
`jx.strip_item(...)` would make a repaint display the last update's snapshot and
would require every state mutation, including cross-route message delivery, to
remember to invalidate the cache.

Keep the six-method `Screen` trait unchanged. Each applicable screen instead
gets an inherent, pure draw-time projection that produces `StatusItem` values;
the Jackin shell dispatches to it by route while composing the host
`StatusBar`. Dynamic formatted strings must remain alive until
`StatusBar::draw` returns. An app-local projection object or a closure-scoped
backing store is acceptable; leaking strings, caching in `Jx`, or widening
`StatusItem` to own application text is not.

The projection preserves every current fact, priority, tone, and action key:
Manager busy work; Settings and Editor saving/change/status facts; Cockpit stage
count; Accounts refresh count; Capsule instance count; and the host Usage,
Accounts, Settings, and Help actions. Tone maps to semantic `Role`, priority is
copied exactly, and clickable strip ids become stable `ItemKey`s. `HintLayer`
is not widened: a hint is an available action, while these items are product
status and navigation chrome.

This removes `Screen::strip_right` without adding a seventh trait method and
without introducing stale derived state. It also preserves pure draw: the
projection may allocate temporary formatting storage but must not mutate the
screen, world, runtime, or component state.

## Q2: one update cause, exact deadlines, and deterministic bootstrap

Add a runtime-owned update cause to `Cx`, with at least `Bootstrap`, `Event`,
`Tick`, and `Settle`. `Cx::update_cause()` is an update-phase read; it does not
belong on `FrameRead`, because `Ui` has no update cause and exposing it there
would weaken the phase boundary. A single delivered timer event presents
`Tick` only on the first `App::update` pass. Any focus-settling rerun presents
`Settle`, so one physical tick can never advance Jackin twice.

The runtime performs exactly one `Bootstrap` update before the first draw or
the first externally handled event, whichever comes first. Bootstrap does not
advance either clock and carries no input intent. This is required because an
application cannot request its first repaint deadline from pure `draw`.
Bootstrap must be a `Runtime` guarantee, not a special call in the terminal
session, so `Harness`, `draw_buffer`, and other headless paths observe the same
lifecycle.

`Cx::request_repaint_after(d)` establishes the earliest outstanding deadline.
An unrelated key, mouse, paste, or resize event must not erase or postpone it.
When the deadline expires, the session delivers one `Input::Tick`; only that
delivery clears the expired deadline. The session waits for the minimum of the
outstanding deadline and its input poll requirement rather than substituting
`design.motion.tick_ms`. No unsolicited idle tick is generated when no deadline
exists. Headless tests continue to drive the identical path explicitly with
`Input::Tick`; they never consult wall time.

Jackin owns the meaning of a tick. On `UpdateCause::Tick`, and only once for that
cause, it computes `let interval = route.tick_ms(true) as i64`, calls
`world.tick(interval)`, advances rain, handoff, launch, jobs, modal loading, and
status expiry, then requests the next deadline derived from the active
producers. The runtime deadline schedules this work but never supplies Jackin's
domain delta. `Route::tick_ms` remains authoritative: 33 ms for
Intro/Outro/Handoff/Cockpit, 80 ms for Capsule, and 80/200 ms elsewhere under
the existing animation rule. `--motion paused` continues to freeze the product
clock. Neither runtime `clock_ms`, wall-clock elapsed time, nor
`design.motion.tick_ms` may be passed to `World::tick`.

This decision keeps `App` at its accepted method set; no separate `App::tick`
hook is added. It also makes the source of mutation explicit without sending a
runtime duration across the product boundary.

## Q4: semantic dimming must be monotonic and total

Adopt the old rain rule for `Success`, `Warning`, `Danger`, and `Info`, but with
the missing zero-step rule made explicit. `dim_layer(area, 0)` is a byte-for-byte
no-op. For those four non-ladder foreground roles, one step becomes
`Fg(Muted)`, two becomes `Fg(Faint)`, three becomes `Fg(Ghost)`, and four or
more erases the glyph into the resolved backdrop background. The rule belongs
in `Ui::dim_layer`, never in Jackin.

The current implementation does not satisfy that contract: every non-ladder
role becomes `Fg(Muted)` for every step, including zero, and it never erases.
The plan's phrase “adopt `rain.rs:133-139`” is therefore insufficient unless
the zero-step identity and erase behavior are both tested. Ladder roles retain
their existing saturating semantic step-down. Background handling remains
role-based. No color-equality reverse lookup is allowed, because Junie already
has role collisions and reduced color levels create more.

The change intentionally corrects the legacy cross-fade where
`border_subtle == text_ghost` and `disabled == text_faint` are misclassified by
color identity. Any moved handoff digest is a defect-fix visual change and must
be captured and classified before blessing.

## Exact acceptance tests

Lane A should bind the public/runtime half with these exact tests:

- `runtime::bootstrap_runs_once_before_first_draw_without_a_tick`
- `runtime::tick_cause_is_delivered_once_when_focus_settles`
- `runtime::repaint_deadline_survives_unrelated_input_and_keeps_the_earliest`
- `runtime::headless_tick_uses_the_same_update_cause_without_wall_clock`
- `ui::dim_layer_zero_steps_is_byte_identical`
- `ui::dim_layer_semantic_roles_step_monotonically_and_erase`

Lane C should bind the application half with these exact tests after the crate
move:

- `jackin::status_projection_is_pure_and_preserves_priority_tone_and_key`
- `jackin::pure_repaint_never_reuses_stale_status_items`
- `jackin::route_tick_ms_is_the_only_world_tick_delta`
- `jackin::one_runtime_tick_advances_the_route_once`
- `jackin::handoff_fade_is_monotonic_under_every_colour_level`

The existing
`reduced_motion_and_paused_frames_are_deterministic`, the frame-282 byte
identity assertion, rain timeline tests, approximately forty `ticks(n)` call
sites, fixture timestamps, Cockpit failure/running frames, and the outro caption
remain mandatory regression evidence. Q1 also retains the existing 36 digest
keys at their three responsive widths. Q4 adds handoff frames under Junie and
Paper at truecolor and mono; every difference is classified before blessing.

## Next Lane C package: total `RunId`

After Slice 5 creates `apps/jackin-preview`, the first Lane C mutation is a
single domain-first package over the moved equivalents of:

- `domain/instance.rs`
- `domain/fixtures.rs`
- `sim/launch.rs`
- `screens/cockpit.rs`
- `screens/capsule.rs`
- `screens/manager.rs`
- the focused unit and application test files

`RunId` replaces both `Instance.run_id: String` and `LaunchRun.run_id: String`.
One constructor normalizes stamp and suffix input; `as_str`, `Display`, and a
fixed-width ASCII `RunToken([u8; 8])` derived from the whole id are total. Both
the fixture and Cockpit producers use that constructor. `Instance::container_uid`
owns the Capsule formatting. Capsule's
`replace('-', "")[..8]`, Cockpit's timestamp `[..12]`, and Manager's separate
`&i.run_id[4..8]` image-tag slice all disappear in this package. The Manager
site is part of the same bug class and must not be deferred merely because its
current fixtures happen to be long enough.

The exact package tests are:

- `instance::run_id_token_is_total_ascii_and_fixed_width`
- `instance::container_uid_is_total_for_every_fixture_run_id`
- `jackin::manager_inspect_is_total_for_every_fixture_run_id`
- `jackin::container_info_opens_in_every_scenario`

The tests cover `Scenario::ALL`, a Cockpit-generated run, empty/non-ASCII and
one-character constructor inputs, and the Manager inspect path. They also grep
the moved Jackin tree for direct byte slicing of `run_id`. Existing invocation
and log paths continue to use `Display`; no raw inner `String` becomes public.

## Integration blockers

Lane C cannot start the app migration while Slice 4 is incomplete and Slice 5
has not created `apps/jackin-preview` with the final `junie-tui` crate name.
Q2 and Q4 additionally require Lane A to record and implement their library
halves. Q1 is application-local after its architecture wording is accepted.
None of these blockers permits changing the 67-test or 36-digest inventory.
