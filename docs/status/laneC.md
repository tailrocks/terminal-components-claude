# Lane C status

## Current state

The fresh Jackin review is recorded in
`docs/reviews/laneC-app-tick.md`. It proposes decisions for Slice 7 questions
Q1, Q2, and Q4 and records the next domain-first `RunId` package. No production
or test source was changed by Lane C.

Fresh source measurements found 67 legacy Jackin test functions, 36 visual
digest cases, and 23 methods on the legacy `Screen` trait. The test inventory is
28 application/chrome tests, 35 in-module tests, three performance tests, and
one visual-baseline test. All remain migration obligations.

## Proposed decisions

Q1 replaces `Screen::strip_right` with a pure draw-time projection into
`StatusItem` values. It does not use a `Jx` cache, does not widen `HintLayer`,
and does not add a seventh `Screen` method. Temporary formatted text remains
alive through the `StatusBar::draw` call; priorities, semantic tones, and stable
action keys are preserved.

Q2 adds a runtime-owned update cause with distinct Bootstrap, Event, Tick, and
Settle states. Bootstrap runs exactly once before the first draw or event and is
shared by terminal and headless execution. Timed repaint requests become real,
earliest-wins deadlines that survive unrelated input. Tick is visible only on
the first update pass for one timer delivery. Jackin, not the runtime, converts
that cause into a domain delta through `Route::tick_ms`; a runtime/theme or wall
clock must never replace it.

Q4 makes `dim_layer(area, 0)` an identity operation and defines semantic
Success/Warning/Danger/Info fading as Muted, Faint, Ghost, then erased. The
current implementation maps all non-ladder roles to Muted even at zero and
never erases, so a library correction and direct tests are required before the
rain migration.

The review lists eleven exact acceptance tests across runtime, dimming, and
Jackin. Existing deterministic frame, rain, tick-count, timestamp, outro, and
all 36 digest assertions remain required.

## Next package

After Slice 5 moves the application, Lane C starts with the `RunId` package in
`apps/jackin-preview`. The package replaces the free-form run-id strings in
both `Instance` and `LaunchRun`, routes both producers through one constructor,
adds a total fixed-width token and `Instance::container_uid`, and removes all
three unsafe derivations:

- Capsule's `replace('-', "")[..8]`
- Cockpit's timestamp `[..12]`
- Manager's `&i.run_id[4..8]`

Four exact tests cover total token generation, every fixture/scenario,
Cockpit-generated ids, Capsule container information, and Manager inspection.
The package is not complete until a source check finds no direct byte slice of
`run_id` in the moved application.

## Blockers

1. Lane A must accept and record Q1, Q2, and Q4 in the architecture and ledger.
2. Lane A must implement and verify the runtime cause/deadline/bootstrap work
   and the `dim_layer` semantic correction under `crates/**`.
3. Slice 4 must finish. Slice 5 must then perform the crate rename and create
   `apps/jackin-preview`; Lane C does not own the current legacy path and will
   not pre-empt the ordered move.

Until those blockers clear, Slice 7 application work has not started. The next
authorized Lane C action is the post-Slice-5 `RunId` package, followed by the
shell migration using the accepted tick and status contracts.
