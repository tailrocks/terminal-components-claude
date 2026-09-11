# TASK-009 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

Normal exit, error, panic, partial startup and suspend/resume restore terminal modes in reverse order, preserve the prior panic hook and leave idempotently. Normalize every mouse button/modifier and horizontal wheel; drop key releases. Exactly one clock-free Bootstrap precedes input/draw; timer delivery issues Tick once then Settle. Flooded input never postpones the earliest deadline or advances elapsed time.

Cursor restoration belongs to the same shared cleanup operation on every path, including the panic hook before delegating to the previous hook. Oracle runtime.rs:119–122/241–247 executes Show during restore_terminal before previous(info); main session.rs:52–67 omits Show and relies on later leave().show_cursor(), which is too late for the previous hook. Record actual terminal commands, not only a generic restore callback label: after an actual hidden cursor, Show and mode restoration must be attempted before the previous hook observes terminal state. A failed Show/output still attempts every remaining cleanup operation, preserves the relevant I/O error and retry semantics, and never claims successful cleanup/publication. Reject the leave-only Show mutant; no second panic-only restoration engine.

Keep optional backend normalization at the terminal boundary; pure Theme/Runtime/Scene constructors do not read environment. Preserve Moment, monotonic advance_to, independent feedback versus simulation and public ColorLevel::narrow_to.

Boundary probes include 2199/2200 ms and feedback 139/140 ms, equal/backward time, no idle unsolicited ticks, contradictory TERM=dumb/FORCE_COLOR/NO_COLOR/redirected output and every 4×4 detected/requested level. Reject output failure claiming successful publication or capability widening.

## Fixed re-audit contract — safe terminal lifecycle and deadline owner

ADJ-13 is normative under R-002/AC-002/CHK-006. Preserve TerminalSession::enter/run public signatures. The sole permitted mutable core global is private SIGNAL_BROKER: OnceLock<Mutex<SignalBroker>> in runtime/session.rs, effectively cfg(all(unix, feature="crossterm")). SignalBroker contains only inactive/pending Arc<AtomicBool>, the serialized lease bit and optional two SigIds/install phase. Stable get_or_init initializes storage; locked fallible registration retains the same flags and successful first registration when the second fails, retrying only the missing registration. Poison returns I/O error. No active lease/terminal mutation before both registrations; no lock in handlers, terminal I/O, waiting or stop. No Runtime/Theme/model/frame/cache/callback data or second global is allowed. The caller-owned disposable prototype does not prove this singleton or retry/concurrency behavior.

Own the bounded xtask/src/main.rs rule18 integration, consuming TASK-072's independently qualified AST exception and mutants. Parse effective cfg/module/type/visibility and aliases, admit only the exact broker, retain immutable constants, reject second/global Mutex/atomic/thread-local/alias-wrapped state and malformed/unresolved source. No file-wide allowlist or spelling-based evasion. Backend-free graphs retain no broker or signal-hook dependency. Production tests cover both registration-failure points, same-flags retry/no duplicate handlers, poisoned initialization, concurrent entry and repeated/post-session cleanup; existing PTY/deadline/fairness obligations below remain separate and mandatory.

TASK-009 owns the bounded manifest/lock changes needed for safe Unix job control, plus runtime.rs for exactly-once clock-free Bootstrap and earliest-deadline scheduling. Keep workspace unsafe denial and backend-free purity. Add workspace signal-hook pinned to =0.3.18 with default-features=false (already present transitively in the pinned lock), make it an optional cfg(unix) tui dependency activated only by the crossterm backend feature, and update only the corresponding lock dependency edge. No direct libc calls, unsafe blocks, platform FFI test escape or dependency in backend-free builds.

Use safe flag registration and a terminal-boundary lifecycle service wait capped at 100 ms; crossterm retries EINTR, so a flag combined with an unbounded read is not sufficient. The wait bound is min(time remaining to the earliest real deadline, lifecycle quantum), with overdue deadlines handled immediately and rechecked after admitted input. A lifecycle-only wake is not Input::Tick, Settle, Bootstrap, simulation advancement, feedback expiry, repaint or application callback. Main architecture:658 prohibits simulation feedback from creating an idle polling loop; this separate backend service must not leak into feedback or deterministic runtime scheduling.

At a pending stop, perform no mode-changing work in the signal handler: the runner restores terminal modes, invokes the safe signal-hook default-handler emulation, then re-enters modes after continuation, re-reads geometry and publishes a full redraw. Safe emulation may stop through SIGSTOP internally; the contract preserves observable external SIGTSTP/job-control lifecycle, not equality of the kernel-reported stopping signal. Use one process-lifetime safe backend signal broker with explicit active/inactive session state, not per-session register/unregister (signal-hook-registry:696–699 does not restore default handling after the last unregister). In the supported owned-process environment, initial SIGTSTP disposition is default and no competing SIGTSTP registrations exist; arbitrary third-party sigaction restoration is not promised. Register conditional default emulation for inactive sessions and a pending flag for active sessions, with a serialized session lease and pending-bit clearing at activation/deactivation. All partial startup, failed re-entry and exit paths make the broker inactive only after mode cleanup, so post-session SIGTSTP retains normal stop behavior. Reject overlapping sessions before terminal mutation. Preserve the prior panic hook; do not claim exact previous OS signal handler restoration. The broker is backend-only, never a second runtime.

Positive/negative owned-PTY probes on macOS/Linux require an otherwise idle externally signalled session, two suspend/fg cycles, exact launch termios while stopped and on final exit, reverse mode cleanup, resized resumed geometry, panic/startup-failure restoration, post-session SIGTSTP default stopping and repeated sessions without duplicate handlers. Deterministic broker probes cover stop arriving before activation, during active wait, during cleanup and after deactivation; repeated flags coalesce without repeated stop after resume. The owned PTY includes partial-startup and re-entry failure followed by a successful new session and an external stop. Oracle tests/terminal_suspend.rs uses a 10-second safety timeout and 20-ms observation sampling; it establishes no 100-ms historical latency guarantee. Keep the oracle's 10-second hard PTY safety bound and report measured stop latency without inventing an OS scheduling parity tolerance. Separately prove the 100-ms service cap under the protected fake clock, and reject a mutant requiring real input before stop. Distinct witnesses reject idle logical Tick/Settle/draw activity, simulation expiry from service wakes, and a near/overdue real deadline delayed until the service quantum or starved by input flood. Preserve exactly one Bootstrap before first draw/input; the lifecycle service cannot synthesize another.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each advertised visible override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. The accepted covered-surface TextArea FIELD exception (main COMPONENT_ARCHITECTURE.md:6799–6815,6997–7003; HIST:EARLY-AMEND-044) instead requires exact recorded selected-part Resolved values while the composed painter's final digest stays unchanged; this is not an exemption for a visible override or replacement slot. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Finite source-qualified witnesses

`trusted/source-witnesses.md` is normative under R-001/R-002/R-003. TASK-006 freezes every expanded case ID, exact source-qualified input/checkpoint and independent expected observation before dispatch; CHK-002/004 compare parity lanes, CHK-005 executes real positive/negative cases and CHK-006 proves architectural ownership. The finite rows supplement all assigned historical clauses and public part/slot census; they do not replace them with a representative subset. Missing source coverage is a failed precondition, not executor-selected non-applicability.

## Specific regression target

Add `crates/tui/tests/completion_009.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `terminal_cleanup`, `feedback_clock`, `monotonic`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:runtime-session-time

- family: runtime-session-time
- reference_implementation: O:src/runtime.rs
- main_implementation: M:crates/tui/src/runtime.rs;runtime/session.rs;runtime/time.rs;runtime/feedback.rs
- architectural_target: App update/draw runtime; separate simulation and feedback clock; optional backend
- visual_status: unverified
- interaction_status: unverified
- api_refactor_status: implemented; verify
- tests_available: M:terminal_cleanup.rs;feedback_clock.rs;monotonic.rs;terminal session units
- tests_missing: Oracle full PTY lifecycle suspend/resume and cancellation; idle and flood fairness

### COMP:event-response

- family: event-response
- reference_implementation: O:src/core/event.rs
- main_implementation: M:crates/tui/src/event.rs;intent.rs;response.rs;action.rs
- architectural_target: Input to typed Intent; Response flow/invalidation/actions
- visual_status: unverified
- interaction_status: boundary and activation source differences need proof
- api_refactor_status: implemented; verify
- tests_available: M:activation_origin.rs;publication.rs;typing_owner.rs;compile-fail response fixtures
- tests_missing: Oracle consume/change distinctions and action count per exact event sequence

### ARCH:A06

- id: A06
- area: backend boundary
- oracle_state: ratatui/crossterm coupled input
- main_state: own key vocabulary optional backend
- architectural_target: backend-free core and testing; backend only session/normalization
- status: implemented;earlier prose superseded
- remaining_obligation: reconcile stale two-files/normal dependency language
- available_gates: core_is_backend_free;isolated backend-free fixtures
- missing_proof: locked both feature closures at final candidate
- evidence: 7b27732:crates/tui/Cargo.toml:17;7b27732:crates/tui/src/event.rs:1

### ARCH:A12

- id: A12
- area: time and feedback
- oracle_state: Instant and app-specific tick clocks
- main_state: Moment monotonic advance and FeedbackClock
- architectural_target: driver time separated from simulation; precise presented feedback
- status: implemented;flow parity unproven
- remaining_obligation: carry same timing boundaries through all restored app scenarios
- available_gates: monotonic;feedback_clock;activation_origin;app timing tests
- missing_proof: seeded equal-time and threshold PTY/action schedules
- evidence: 7b27732:crates/tui/src/runtime/time.rs:1;7b27732:crates/tui/src/event.rs:24

### ARCH:A30

- id: A30
- area: MSRV/dependency policy
- oracle_state: edition2024 Rust1.88 ratatui
- main_state: edition2024 Rust1.88 ratatui-core split
- architectural_target: keep floor lint policy exact allowed graph; optional backend
- status: measured MSRV compile pass
- remaining_obligation: preserve latest accepted constraints and run full gates
- available_gates: cargo +1.88.0 check --locked --workspace --all-targets --all-features
- missing_proof: full stable/MSRV execution and lint/docs sweep at final candidate
- evidence: 7b27732:Cargo.toml:22;7b27732:.github/workflows/ci.yml:66

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A89

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §22,§25 later 4e84b744;amendment=4e84b744; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Backend-free input vocabulary and core/testing builds without terminal feature unification
- Disposition: accepted_later_supersession; current retain_and_verify
- Remaining proof: Isolated no-default-feature consumers exclude backend requirement
- Gates: backend-free fixtures
- Origin: docs/refactoring-plan/historical-obligations.tsv:90; global semantic anchor; supplemental clauses retained

### HIST:A95

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §22,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Terminal lifecycle restores modes in reverse with chained panic hook and idempotent leave
- Disposition: accepted; current retain_and_verify
- Remaining proof: Partial startup, normal exit, panic, error and suspension
- Gates: owned PTY lifecycle
- Origin: docs/refactoring-plan/historical-obligations.tsv:96; global semantic anchor; supplemental clauses retained

### HIST:A96

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §22; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Exhaustive mouse normalization/modifiers/horizontal wheel; drop key release and avoid keyboard enhancements
- Disposition: accepted; current retain_and_verify
- Remaining proof: Normalized input corpus and actual binary behavior
- Gates: event tests;PTY inputs
- Origin: docs/refactoring-plan/historical-obligations.tsv:97; global semantic anchor; supplemental clauses retained

### HIST:A97

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §54 later timing; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Exactly one Bootstrap; first timer pass Tick then Settle; monotonic deadlines persist and input does not advance elapsed time
- Disposition: accepted_amended; current retain_and_verify
- Remaining proof: 2199/2200 ms and input-flood invariance; no unsolicited ticks
- Gates: elapsed_contract;runtime scheduler
- Origin: docs/refactoring-plan/historical-obligations.tsv:98; global semantic anchor; supplemental clauses retained

### HIST:A104

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §74; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: --color ceiling applied once in runner; TERM=dumb/NO_COLOR/redirected output respected
- Disposition: accepted; current compare_new_oracle
- Remaining proof: Full 4x4 levels and fresh-process environment matrix
- Gates: color CLI tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:105; global semantic anchor; supplemental clauses retained

### HIST:F21

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f21-restore-terminal-state-for-supported-job-control-suspension.md
- Requirement: Runtime owns terminal restoration on supported suspension/resume
- Disposition: current_done; current current_main_mapping_required
- Remaining proof: Actual owned PTY signal/session proof, no model-only substitute
- Gates: SIGTSTP/SIGCONT lifecycle
- Origin: docs/refactoring-plan/historical-obligations.tsv:155; global semantic anchor; supplemental clauses retained

### HIST:F23b

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f23b-lifecycle-v01-v02.md
- Requirement: Terminal lifecycle proves declared exit/error/panic/startup paths
- Disposition: current_done; current current_main_mapping_required
- Remaining proof: Enumerate actual exercised exits and restoration
- Gates: owned PTY lifecycle
- Origin: docs/refactoring-plan/historical-obligations.tsv:158; global semantic anchor; supplemental clauses retained

### HIST:F23d

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f23d-input-flood-fairness-v04.md
- Requirement: Input floods cannot starve work or advance simulated time incorrectly
- Disposition: current_done; current current_main_mapping_required
- Remaining proof: Bounded fairness and deterministic clock assertions
- Gates: input flood
- Origin: docs/refactoring-plan/historical-obligations.tsv:160; global semantic anchor; supplemental clauses retained

### HIST:O05

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; docs/improvements-plan-reference.md
- Requirement: Terminal extensions/OS clipboard
- Disposition: conditional_not_selected; current current_main_mapping_required
- Remaining proof: Preserve existing interface without adding live integration
- Gates: scope disposition
- Origin: docs/refactoring-plan/historical-obligations.tsv:193; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-049

- Source: 934c92dc §8.6; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: FeedbackClock Simulation uses absolute domain-owned synchronized time and preserves source cadence without wall deadlines.
- Disposition: accepted extension; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Wrong-policy/backwards reject atomically; equal idempotent; tick/input/draw/wall count cannot age simulation; owner disappearance does not cancel cadence.
- Gates: Nonzero epoch;80/160/coalesced delayed steps;paused no spin/deadline;future stable row/owner no input authority;elapsed140 retained.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:50; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-042

- Source: 934c92dc §8 feedbackamendment; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Select elapsed or absolute simulation feedback policy before init; simulation advances only from admitted domain coalesced step, never draw/inputcount/walltime.
- Disposition: accepted_later_extension; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Default elapsed140 and source80/160 simulation are distinct policies; no blanket keyboard flash rule.
- Gates: Paused no spin/deadline; sought nonzeroepoch; delayed/coalesced full/reduced; wrongpolicy/backwards atomic rejection.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:43; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-045

- Source: 27bd918e §22;587c53bd §25;4e84b744f9b3a1e007aa9080e7ab924ed8ca7246; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Backend-free core owns KeyCode/modifier/input vocabulary; optional terminal adapter retains exhaustive conversion; no direct crossterm/ratatui umbrella.
- Disposition: accepted_later_supersession; current implemented by direct source inspection; isolated dependency fixture proof remains retention gate
- Remaining proof: Historical §25 nonoptional backend vocabulary dependency is superseded by4e84b744 own keys and optional adapter; do not reintroduce backend through feature unification.
- Gates: Retain isolated backend-free-core/backend-free-testing fixture graphs and no-default compilation; exhaustive event/modifier conversion and optional session feature.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:46; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-006

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:116-128
- Requirement: Runtime restoration, one-shot redraw and reusable session need explicit ownership
- Disposition: accepted direction; hook detail finding; current unverified
- Remaining proof: Check stacked panic hook and partial initialization/restore behavior
- Gates: Deterministic deadlines; normal/error/panic restoration; repeated session entry
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:7; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-022

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=e81ca17b; docs/audit/app-audit.md:633-671
- Requirement: Deterministic motion/effects preserved while semantic theme replaces palette reversal
- Disposition: accepted direction; current unverified
- Remaining proof: Keep pacing seed skip hold handoff and resize oracle states
- Gates: Full/Reduced/Paused frames; duplicate-color theme; ANSI/Mono; actual paint provenance
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:23; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-008

- Source: e81ca17b; docs/audit/interaction-audit.md B10
- Requirement: Out-of-band repaint deadlines alongside Response invalidation
- Disposition: accepted; Layout retained as Paint-equivalent initially; current not independently tested
- Remaining proof: One-shot repaint and deterministic clock; preserve oracle animation phases
- Gates: virtual tick deadline sequences; no input-dependent drift
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:9; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-025

- Source: 27bd918e;587c53bd;4e84b744f9b3a1e007aa9080e7ab924ed8ca7246; docs/audit/modern-api-audit.md1; foundations review2.1
- Requirement: Backend-free core owns input vocabulary; optional ratatui-crossterm terminal adapter; no umbrella/widgets/macros
- Disposition: accepted later supersession of historical nonoptional backend vocabulary; current implemented by direct source inspection; retain isolated graph and conversion proof
- Remaining proof: Preserve optional backend and core-owned keys; do not reconstruct obsolete mandatory backend dependency
- Gates: dependency/path checks and no-default-features compile; no direct app crossterm
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:26; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-029

- Source: ba858131; docs/audit/modern-api-audit.md R14-R20
- Requirement: Exhaustive mouse normalization; typed terminal mode commands; no keyboard enhancement; no Masked
- Disposition: accepted with later normalization refinements requiring join; current not independently tested
- Remaining proof: Preserve modifiers/right-middle button events and key press/repeat/release policy; terminal restore order
- Gates: synthetic normalization; mouse PTY; panic/partial-init restoration; exact backend boundary
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:30; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-54-CAUSE

- Source: ecc13378 COMPONENT_ARCHITECTURE §54.1–.3; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Cx-only Bootstrap/Event/Tick/Settle; one Bootstrap; one Tick per delivery; earliest persistent deadline
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Ui/FrameRead update cause; session-only bootstrap; unsolicited idle ticks
- Gates: Retain exact six library tests listed in §54.8; test terminal/headless lifecycle and settle reruns
- Origin: docs/refactoring-plan/history-late-obligations.tsv:2; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-74-COLOR

- Source: ecc13378 §74.1; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Runtime color request ceiling weaker(detected,requested), clamp silently; environment precedence intact
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Per-binary detect/replace; overrequest error; flag raises capability
- Gates: 4x4 levels plus NO_COLOR/dumb/redirect tests; app parsers pass request only
- Origin: docs/refactoring-plan/history-late-obligations.tsv:52; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-74-ORDER

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §74.1 open question; main tokens.rs525/644 (current cumulative source; originating edge in section coverage index); COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Public narrow_to exists; ColorLevel has no Ord
- Disposition: unresolved-prose-existing-solution; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Proposed plan rejects new public Ord; retains existing meet API
- Gates: Record fixed adjudication to close stale public-order/private-rank question; no builder discretion
- Origin: docs/refactoring-plan/history-late-obligations.tsv:53; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM10

- Source: 9486b654 §34;later §74; COMPONENT_ARCHITECTURE.md
- Requirement: Terminal boundary narrows authored/requested color ceiling; deterministic Theme Runtime Harness Scene constructors do not read environment
- Disposition: accepted_amended; current retain_verify
- Remaining proof: Preserve authored palettes and pure canonical labeling; reconcile flags with oracle explicitly
- Gates: TERM dumb; force/nonzero; NO_COLOR nonempty/empty; stdout tty; unset TERM; pre-downgraded theme
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:11; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM11

- Source: 9486b654 §34.3; COMPONENT_ARCHITECTURE.md
- Requirement: Capability detection has pure argument-based environment decision table; dumb outranks force; force outranks NO_COLOR/pipe; downgrade idempotent
- Disposition: accepted; current retain_verify
- Remaining proof: Test independent precedence intersections and repeat narrowing; no test env mutation required
- Gates: pure from_env table; equal-level idempotence; never widen
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:12; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM12

- Source: 9486b654 §34.4;later §40 correction; COMPONENT_ARCHITECTURE.md
- Requirement: Capture must actually route through capability-aware runtime and propagate requested theme/color; flags and labels alone prove nothing
- Disposition: accepted; current needs_oracle_capture_verification
- Remaining proof: Full executable mono capture with contradictory truecolor env proving effective downgrade; matching metadata
- Gates: PTY color matrix plus canonical frame metadata equality
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:13; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM62

- Source: 3adb6efe §54;ecc13378 §54.1–2; COMPONENT_ARCHITECTURE.md;docs/reviews/laneC-app-tick.md
- Requirement: Runtime exactly one intent-free clock-free bootstrap; Cx-only Bootstrap Event Tick Settle; one timer delivery causes Tick once through settling
- Disposition: accepted_strengthened; current retain_verify
- Remaining proof: Same lifecycle terminal and headless; no draw-time cause branch; event before first draw still bootstraps once
- Gates: bootstrap_runs_once; tick_cause_once_when_focus_settles; headless lifecycle
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:63; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM63

- Source: 3adb6efe §54;ecc13378 §54.3;later runtime amendments; COMPONENT_ARCHITECTURE.md
- Requirement: Repaint deadlines persistent earliest-wins; unrelated inputs cannot erase/postpone; expired delivery clears only its deadline; no unsolicited idle ticks
- Disposition: accepted_later_monotonic_amendments; current retain_verify
- Remaining proof: Retain later monotonic/publication contracts; compare exact oracle simulation feedback timing
- Gates: unrelated key/mouse/paste/resize; min poll/deadline; headless controlled ticks; no-wall-time equality
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:64; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
