# Holla shell construction and clock contract

## Fixed authority and ownership

Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c` owns CLI, scenario names and observable time. Main `7b27732a8c3c131760ec3438f641cb3c11343a42` supplies typed App/Cx/Ui, pure parsing, feedback-clock synchronization and checked arithmetic. This planning contract assigns B-HO-01/04 without making040 depend on later domains or pretending their worlds already work.

TASK-040 owns canonical34-name parsing/help/error grammar, the shared constructor/result boundary, source epoch, start-before-frame-advance and admitted simulation tick mechanics. Existing full HO-BASE owners retain each actual world, route and domain transition. TASK-041 still depends on040;040 keeps its current shared prerequisites. TASK-050 requires every canonical34 production constructor and full trajectory to succeed, with no ScenarioUnavailable result or fallback world remaining.

## One canonical registry and typed construction

Use one canonical `Scenario` enum and ALL/CONCEPT/PARITY/name/from_name registry with all34 oracle identities and ordering. There must not be a second eleven-name CLI authority. Define a typed `ScenarioUnavailable { scenario: Scenario }` construction error in scenario.rs, expose the error alongside the public app constructor, and use `world_for(scenario: Scenario, motion: Motion) -> Result<World, ScenarioUnavailable>` from the fixture factory and `Result<App, ScenarioUnavailable>` from `App::for_scenario`. Motion is passed into fixture construction, not applied only afterward: source ActivitiesMulti setup already calls World ticks before App construction, and paused setup must remain paused. Legacy direct World test helpers receive an explicit Motion::Full to preserve their formerly running initial clock unless their frozen original setup says otherwise. Actual registered constructors return their real World; a not-yet-restored constructor may return the typed error only as an explicitly frozen future-owned stage failure.

The existing eleven constructors must remain real and successful while their responsible tasks restore source behavior. No FirstUse fallback, empty synthetic world, accepted placeholder, panic/unwrap in production, skipped constructor, executor-selected available-set or candidate receipt can satisfy a missing source world. Later domain tasks replace only their named unavailable factory arms with real constructors; final050 requires zero such arms/results. The error is an intermediate integration condition, not final oracle-visible behavior.

`cli::parse` still parses arguments without constructing an App or acquiring a terminal. Every valid source name parses successfully even when its later fixture factory is not yet restored. `lib::run` handles the construction Result before `run_with_feedback_clock`; unavailable construction emits a typed integration failure and exits nonzero without terminal acquisition, raw-mode changes or simulated provider effects. That outcome remains a named future-owned failure, never a passed valid-name oracle process case. Invalid option parsing remains a distinct typed CLI error with exact source stderr/exit grammar. Oracle errors include Debug-formatted rejected values at main.rs51–74; the old main non-echo assertion requires an explicit008 disposition, using synthetic nonsecret proof inputs.

Main evidence: scenario.rs34–70; fixtures.rs31–40,52–97; app.rs217–220; lib.rs36–68. Oracle evidence: scenario.rs ALL/CONCEPT/PARITY and name/from_name; main.rs37–120; app.rs133–175.

## Actual start and one simulation advance seam

The production constructor first builds its real World and App, then performs the source start route. Save the World clock running flag, temporarily enable advancement, invoke one real App-owned simulation advancement seam exactly N times, then restore the saved source motion policy. This seam also handles every admitted runtime simulation tick; it must not require a fabricated Cx or duplicate a second constructor-only domain timeline.

Oracle epoch is1789004460, displayed as 2026-09-10 08:41 UTC+7. Each delivered simulation tick advances80ms, independently of delivery cadence: paused requests500ms callbacks without simulation advancement; animating requests80ms and otherwise200ms, but a delivered nonpaused tick still advances80ms. Main's current cadence-as-delta and frame-as-milliseconds paths are wrong. Retain checked arithmetic, monotonic feedback synchronization before domain mutation, early/unrelated-input nonadvancement and accepted runtime admission semantics. Do not advance once per wall-clock millisecond or catch up invented domain events.

At040, the seam processes existing real shell/discovery state and expiry, without copying an obsolete runtime. Later044 adds Activity/plan event progression and ActivitiesMulti's48 source constructor tick calls,045 adds scans,046 per-item cleanup/report/quit settlement,047–049 concrete domain outcomes. Those callbacks must join the same production seam in source order; they cannot invent an independent clock. Complete frame/state assertions for later domains stay with their full parents. A time-only equality never certifies a missing domain transition.

An optimization may skip only an interval proved to have no distinct observable transition for the current real state. Keep bounded arithmetic and relevant existing overflow tests; do not reinstate unchecked arithmetic, claim u64::MAX was executed on the oracle, or use the old discovery-only fast-forward premise after restoring live state machines.

Oracle evidence: app.rs133–175,242–246,312 onward; world.rs482–552; clock.rs6–28. Main evidence: app.rs912–938 and world.rs174–202.

## Scope boundaries and mandatory setup adaptation

In addition to the original040 source scope, the responsible production layers are exactly:

- `apps/holla/src/clock.rs`: epoch and checked fixed-step clock behavior.
- `apps/holla/src/scenario.rs`: canonical34 identities, names, order and typed unavailable error.
- `apps/holla/src/sim/world.rs`: factory-error propagation if needed and the shared clock/frame advancement seam; no late domain model implementation.
- `apps/holla/src/domain/fixtures.rs`: canonical factory dispatch/result propagation and its existing constructor setup only; do not implement future FS/config/Activity/plan seeds here. Preserve current eleven real constructor bodies until their existing correction owners change them.
- Already-owned app.rs/lib.rs/cli.rs: start ordering, shared advancement, Result propagation and preterminal error handling.

The following additional files are writable only for host-approved constructor/motion/result setup adaptation under TASK-008's frozen mechanical patch records. They gain no production behavior or assertion-edit permission: `src/app/historical_tests.rs:13`; `src/dispatch.rs:214`; `src/screens/activity.rs:234`; `src/screens/home.rs:439,450`; `src/screens/plan.rs:326`; `src/screens/plan_gate.rs:225`; `src/sim/catalogue.rs:821`; `src/sim/pg.rs:157`; `src/sim/plans.rs:1003`; `tests/perf.rs:24,127`; `tests/visual.rs:31,45`, all relative to apps/holla at pinned main. Existing040 app.rs, cli.rs, screens/dialogs.rs and the newly assigned fixtures.rs contain further setup callers already inside source scope; their test edits obey the same restriction.

TASK-008 must record each original call site, test identity, fixture tuple and exact setup-only API adaptation before040 starts. Preserve every original eleven-fixture invocation, all compatible assertion bodies and original archive bytes. A loop originally covering eleven fixtures retains those exact eleven fixture tuples while the canonical registry grows; it cannot filter unavailable constructors, use filter_map, if-let skipping, weaken expected values or drop test iterations. Separate new proof identities cover all34 parser results and later all34 real constructors. Previously approved oracle-conflicting epoch/frame-ms/CLI/world-count expectations are handled only by008's independent disposition, not by this mechanical API permission.

CHK-006 rejects changes outside those allowed setup spans for the extra files, unapproved setup adaptation and any test/inventory skip. CHK-005 compiles and runs the complete original compatible eleven-fixture inventory on the integrated tree, including all caller modules, and reports every later unavailable case under its exact frozen owner. Missing008 records stop040; an executor does not invent a migration manifest.

## Named proof branches and stage contributions

All branches are source-inspected planning obligations, not executed qualification.003 freezes their actual source-derived numeric inputs, observation schema and full artifacts before execution.

- **cli-parse-all34** under HO-CLI-PREVIEW/HO-SHELL-C05: invoke the real parser for every34 canonical name with long and short scenario flags; require exact typed scenario/motion/frame/color results and source ordering. The oracle observation adapter runs actual main.rs parse_args in an isolated source-bound process and observes its return before App construction; it does not replace parsing with a hand-coded verdict. Help/errors still use genuine process stdout/stderr/exit. This parser contribution does not claim34 application launches. Candidate parser must remain the same code called by lib::run.
- **clock-firstuse-frame-0-1-40** under HO-BASE-01/HO-SHELL-C03: three fresh real FirstUse/Paused constructors at frame0/1/40; require source start identity and now_ms0/80/3200, the source epoch and paused running policy after construction. Ten subsequent source-equivalent paused ticks do not advance simulation. Compare every required shell observation; full Finder frames remain041.
- **clock-cadence-not-delta** under HO-BASE-01/HO-SHELL-C03: fresh real FirstUse/Reduced frame40 after source discovery settlement; prove one admitted200ms-cadence runtime tick advances simulation by80ms, while early/unrelated input does not advance. Use the actual runtime admission/feedback path; no synthetic Cx, direct clock assignment or changing event counts to force expected time.

HO-CLI-PREVIEW retains its complete owned help/error/stdout/stderr/exit grammar on actually constructed first-use process cases. Add each valid-name full application launch to its existing HO-BASE parent at its actual full owner. Whole34 constructor success and full transition trajectories are mandatory050, not040. Every source frame remains captured and compared by its real owner; parser-only/state-only contributions never count as a whole process/frame pass.

R-001 binds direct capture CHK-002, applicable executable capture CHK-003, exact owned full-frame/process comparison CHK-004 and source-qualified shell contribution CHK-006. R-002/AC-002/CHK-006 additionally proves one registry, actual constructor/error/clock paths, preterminal refusal and setup-only scope. R-003/CHK-005 preserves complete existing tests and source-owner accounting. No unavailable constructor may be hidden, blessed or reclassified as not applicable.
