# Coverage measurement

This file reports honest test coverage numbers.
It uses simple sentences.
Each sentence holds one fact.

## This copy

- Branch: `termrock-implementation`.
- Tip: `0a528b521cd40412a0ec668c67ef47d4a03218eb`.
- The same report lives on `visual-baseline`.
- Only this section differs between the two copies.

## Words

- **Reference**: the code before the refactor.
  It lives on branch `visual-baseline`.
- **Candidate**: the code after the refactor.
  It lives on branch `termrock-implementation`.
- **Coverage row**: one test case in the scenario registry.
  It has a stable ID.
  It names the app, the component, and the part.
  It lists input steps and checkpoints.
  It lists visual, state, action, and negative checks.
- **Checkpoint**: one named point in a Coverage row.
  The test captures the screen at the Checkpoint.
  The test compares the capture with the snapshot.
- **PTY**: a pseudo-terminal.
  It runs the app as a real terminal runs it.
  Tests that need a PTY use the `pty-capture` adapter.
  Tests that draw to a buffer use the `headless-tick` adapter.
- **Parameter case**: one Checkpoint capture with fixed viewport, color, and motion.
  All 299 Checkpoints set `capture: true`.
- **Legacy root**: one old snapshot path.
  The registry maps old paths to new row IDs.
  Pending roots have no row yet.

## Required coverage rows

- `registry.json` holds 171 scenarios.
- The 7 group slices hold 138 of the 171 scenarios.
  The 9 S1 rows plus the 10 T1 rows plus the 14 J1 rows in controls, fields, and navigation await slice-file homes.
- The 7 slices hold 147 coverage row refs across 46 components.
- Controls lists 24 row refs for 15 scenarios.
  Some rows sit under two components.
- Every row has at least one input step.
- Every row has at least one capture Checkpoint.
- Every row has at least one check in each of the 4 classes.

| Slice | Scenarios | Components | Row refs | Legacy entries |
|---|---|---|---|---|
| controls | 15 | 10 | 24 | 23 |
| fields | 16 | 3 | 16 | 11 |
| navigation | 16 | 8 | 16 | 32 |
| overlays | 27 | 10 | 27 | 48 |
| forms | 19 | 2 | 19 | 21 |
| feedback | 13 | 6 | 13 | 11 |
| data-views | 32 | 7 | 32 | 48 |
| total | 138 | 46 | 147 | 194 |

## Required parameter cases

- Checkpoints total 299.
  All 299 set `capture: true`.
- Rows by Checkpoint count: 87 rows have 1, 45 rows have 2, 34 rows have 3, 5 rows have 4.
- Viewports: 114 rows at 80x24, 50 at 120x40, 4 at 72x20, 1 at 100x30, 1 at 60x10, 1 at 12x6.
- Color: all 171 rows use truecolor.
- Motion: 161 rows paused, 10 rows playing.
- Adapters: 169 rows allow both adapters, 1 row PTY only, 1 row headless only.
- Input steps total 627.
- Checks total 1347: visual 352, state 335, action 328, negative 332.
- Legacy matrix: 302 roots x 5 sizes x 5 colors = 7550 captures.

## Implemented tests

- Registry: 171 of 171 rows are complete.
  Complete means inputs plus Checkpoints plus all 4 check classes.
- Reference Rust suite: `tests/harness/tests/visual_baseline/`.
  It holds matrix tests plus `negative_controls` (18 tests), `control_states` (14 tests),
  `button_busy_frames` (3 tests), `button_busy_live` (2 tests), and support unit tests.
- Candidate Rust suite: `crates/termrock-conformance/tests/`.
  It holds matrix tests plus `control_states` (109 tests), `button_busy_live` (2 tests),
  `button_busy_frames` (2 tests), `cursor_provenance` (6 tests), `parity_burndown` (3 tests),
  and support unit tests.
- The candidate has no `negative_controls` module.
- Validators print PASS: 7 of 7 on the reference, 4 of 4 on the candidate.
  The candidate has only 4 slice files.

## Executed cases

- Reference full run: 347 run, 347 passed, 0 failed, 1 skipped.
  The skip is the approval writer, excluded by repo config.
  Run SHA: `6007e9fdcbf353fef59177ed8b35d5801f1b0fe2`.
  Evidence: `/tmp/phase3z2-worker.md`, `/tmp/phase3z2-nextest.log`, `/tmp/phase3z2-review.md`.
- The reference tip adds 2 test-only commits after the run.
  No product code changed.
  No full rerun exists at the tip.
- Candidate full run: 483 run, 318 passed, 164 failed, 1 timed out, 1 skipped.
  The skip is the approval writer, excluded by repo config.
  Run SHA: `114cb9061d8d397cf9ad234015befdd9d61e0576`.
  Evidence: `/tmp/phase4a-worker.md`, `/tmp/phase4a-full2.log`, `/tmp/phase4a-review.md`.
- The candidate tip adds 9 commits after the run.
  Six change product code: Q53, FIX-002, Q54, Q55, FIX-002E, Q56.
  The candidate numbers are stale for those 6 deltas.
- Registry-driven execution is zero on both branches except slices 8A-S1, 8A-T1, 8A-J1, 8A-S2, and 8A-S3.
  On the reference branch, phase-8c executed slice 8A-S1 with 21 passes,
  phase-8e executed slice 8A-T1 with 14 passes, phase-8g executed
  slice 8A-J1 with 30 passes, phase-8v executed slice 8A-S2 with
  20 passes, and slice 8A-S3 added 19 checks (b712422f).
  On the candidate branch, phase-8d executed the ported S1 slice: 3 rows
  report `results.status = pass`, 18 report `fail`.
  Phase-8f executed the ported T1 slice: 5 rows report `pass`, 9 report
  `fail`.
  Phase-8h3 executed the ported J1 slice: 5 rows report `pass`, 25 report
  `fail`.
  Phase-8y executed the ported S2 slice: 2 rows report `pass`, 18 report
  `fail`.
  Phase-8aa executed the ported S3 slice: 0 rows report `pass`, 19 report
  `fail`, and 67 rows remain `unrun`.
  Evidence: `/tmp/phase8d-focused.log` (3 pass / 18 fail / 379 skip),
  `/tmp/phase8f-focused.log` (5 pass / 9 fail / 527 skip),
  `/tmp/phase8h3-focused.log` (5 pass / 25 fail / 534 skip),
  `/tmp/s2-run2.log` (2 pass / 18 fail / 686 skip),
  `/tmp/s3-run3.log` (0 pass / 19 fail / 706 skip).

## Reference passes

- All 347 executed reference tests passed.
- This includes the 18 negative-control tests.
- This includes the 2 live Button tests.
- Zero harness failures occurred.

## Candidate passes, regressions, blocked

- 318 tests passed, including the store-integrity gate.
- 165 tests did not pass: 164 failed plus 1 timed out.
- Zero harness failures occurred.
  All 165 are genuine product failures.

| Group | N | Meaning |
|---|---|---|
| G-control-states | 24 | paint and behavior checks failed, plus 2 API gaps |
| G-button-busy-live | 2 | frozen spinner, wrong marker tone |
| G-cells-differ | 90 | rendered frames differ from approved frames |
| G-wait-needle-target | 48 | expected text or target never appeared |
| G-timeout | 1 | frozen step rail exhausted the 600s budget |

- Blocked: the approval writer stays skipped by config on both branches.
- Blocked: registry rows cannot execute until a runner exists.
- Caveat: wait-group verdicts rest on single-run evidence.
  Per-test load flake is not excluded.

## Known candidate regressions

- Busy spinner is frozen: 1 distinct frame in a 28-tick window.
- Busy plus checked marker uses white, not accent `#48e054`.
- W01: press paints the same as hover.
- W02: disabled keeps the focus gutter.
- W03: checkbox hover and compact markers are wrong.
- W04: toggle compact markers are wrong.
- W05: no horizontal radio layout, no per-option disabled.
- W06: chipbar keyboard toggle and reorder keys are dead.
- W08: input navigation, paste, masking, and external-change rules fail (4 tests).
- W09: text area navigation, scroll, and wheel rules fail (3 tests).
- W10: select popup bottom row does not fade.
- W11: Enter while busy submits the form.
- W12: patched label does not read bold.
- W13: filter backspace, reseed, and wide-trail rules fail (3 tests).
- W16: running step shows 1 spinner phase, not 10.
- W17: focused tab does not read bold.
- 90 matrix captures differ cell by cell across jackin, showcase, tablepro, and pointer suites.
- 48 interaction flows never reach the expected state.
- One showcase terminal page never leaves `0 of 7`.
- The predicted button width divergence did NOT reproduce.
  Widths, rects, and glyphs pass.
  The live regression is marker tone, not geometry.

## Testability gaps

- Windows ConPTY compiles only.
  It never executed.
  Never claim Windows coverage.
- Some terminal modes cannot be tested: DEC modes 4, 6, 20, OSC 4, OSC 10, OSC 11, CSI 14, 16, 18t.
  The backend absorbs or drops them.
- Cursor shape and blink need live PTY captures.
  Raw-ANSI replay cannot observe the cursor.
- Emoji and other uncovered codepoints render tofu.
  Tests must assert tofu plus advance plus the fidelity record, not emoji pixels.
- Hidden text is concealment, not redaction.
  Tests must use synthetic secrets only.
- Unknown escape sequences are absorbed silently.
  No reporting surface exists.
- `press_chord` drops unknown `+` modifiers on f-chords silently.
  No test sends such chords.
- The engine docs claim portable-pty plus alacritty.
  The manifest pins termpane.
  A lockfile check must resolve this before citing engine provenance.
- The candidate lacks the `negative_controls` module port.
- The candidate lacks 3 slice files: controls, fields, navigation.
  That is 56 missing coverage row refs and 55 roots triaged nowhere on the candidate.
  On the reference, 50 of those 55 have a mapped row and 5 are pending-only.
- All 171 `cand_symbols` mirror `ref_symbols` exactly.
  They are placeholders, not derived candidate facts.
- All 171 `snapshots.cand` lists are empty.
- The union `legacy_roots` list still marks all 302 roots pending.
  It was never reconciled with the slice mappings.

## Negative-control status

- Reference: 18 negative-control tests, all pass.
- Reference: argv0 mask holds fail-closed with negative controls.
- Candidate: html mask tests 6 of 6 pass, including negative controls.
- Candidate: mask forgiveness fired 0 times in the full run.
- Registry: 332 negative checks exist across 171 rows.
  The 42 in S1 rows were exercised live (3 rows pass, 18 fail).
  The 28 in T1 rows were exercised live (5 rows pass, 9 fail).
  The 60 in J1 rows were exercised live (5 rows pass, 25 fail).
  The 30 in S2 rows were exercised live (2 rows pass, 18 fail).
  The 38 in S3 rows were exercised live (0 rows pass, 19 fail).
  The other 134 remain unrun.
- Candidate: no dedicated negative-controls module exists.

## Uncovered inventory

- App journeys: showcase 23 pages, holla 34 scenarios and 15 routes,
  jackin 8 scenarios and 12 routes, tablepro 2 screens and 21 surfaces.
- Registry rows by app: showcase 94, jackin-preview 47, tablepro 28, holla 2.
- Holla is the largest gap: 2 rows against 34 scenarios.
- Legacy roots: 156 distinct roots triaged in slices.
  147 have a mapped row, 9 are pending-only.
  155 roots still pending.
- Pending roots by app: holla 122, jackin 14, showcase 9, tablepro 10.
- 20 slice legacy entries are marked pending, not mapped.
- 32 roots appear in more than one slice.
  24 roots are mapped by more than one slice.
- `TOOSMALL-NOTICE-001` has no reference snapshots.
  It is a behavioral probe.
- No full-suite rerun exists at either current tip.
- No registry-driven runner exists yet; the 21 S1 rows plus the 14 T1
  rows plus the 30 J1 rows plus the 20 S2 rows plus the 19 S3 rows were
  executed as tests (`showcase_pending_s1.rs`, phase-8d port of 76537c27;
  `tablepro_pending_t1.rs`, phase-8f port of 07a56acf;
  `jackin_pending_j1.rs`, phase-8h3 port of 3f167a6e;
  `showcase_pending_s2.rs`, phase-8y port of 91aba510: 18 both-adapter,
  1 headless-only, 1 PTY-only;
  `showcase_pending_s3.rs`, phase-8aa port of b712422f: 17 both-adapter,
  2 PTY-only), so 67 rows remain unrun as cases.

## Evidence paths

- Tips: `git rev-parse origin/visual-baseline origin/termrock-implementation`.
- Registry: `tests/scenario-registry/v1/registry.json` plus the slice files.
- Reference run: `/tmp/phase3z2-worker.md`, `/tmp/phase3z2-review.md`, `/tmp/phase3z2-nextest.log`.
- Candidate run: `/tmp/phase4a-worker.md`, `/tmp/phase4a-review.md`, `/tmp/phase4a-full2.log`,
  `/tmp/phase4a-triage.tsv`.
- Journeys: `/tmp/phase2-journeys.md`.
- Machine-readable twin of this file: `docs/testing/coverage-measurement.json`.
