# Coverage measurement

This file reports honest test coverage numbers.
It uses simple sentences.
Each sentence holds one fact.

## This copy

- Branch: `visual-baseline`.
- Tip: `82fb265b2667a0064cd2444dc7870f86fae195eb`.
- The same report lives on `termrock-implementation`.
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
  All 372 Checkpoints set `capture: true`.
- **Legacy root**: one old snapshot path.
  The registry maps old paths to new row IDs.
  Pending roots have no row yet.

## Required coverage rows

- `registry.json` holds 244 scenarios.
- The 7 group slices hold the same 244 scenarios.
- The 7 slices hold 253 coverage row refs across 47 components.
- Controls lists 94 row refs for 85 scenarios.
  Some rows sit under two components.
- Every row has at least one input step.
- Every row has at least one capture Checkpoint.
- Every row has at least one check in each of the 4 classes.

| Slice | Scenarios | Components | Row refs | Legacy entries |
|---|---|---|---|---|
| controls | 85 | 11 | 94 | 91 |
| fields | 17 | 3 | 17 | 11 |
| navigation | 30 | 8 | 30 | 45 |
| overlays | 48 | 10 | 48 | 69 |
| forms | 19 | 2 | 19 | 21 |
| feedback | 13 | 6 | 13 | 11 |
| data-views | 32 | 7 | 32 | 48 |
| total | 244 | 47 | 253 | 296 |

## Required parameter cases

- Checkpoints total 372.
  All 372 set `capture: true`.
- Rows by Checkpoint count: 160 rows have 1, 45 rows have 2, 34 rows have 3, 5 rows have 4.
- Viewports: 114 rows at 80x24, 123 at 120x40, 4 at 72x20, 1 at 100x30, 1 at 60x10, 1 at 12x6.
- Color: all 244 rows use truecolor.
- Motion: 234 rows paused, 10 rows playing.
- Adapters: 242 rows allow both adapters, 1 row PTY only, 1 row headless only.
- Input steps total 976.
- Checks total 1931: visual 498, state 481, action 474, negative 478.
- Legacy matrix: 302 roots x 5 sizes x 5 colors = 7550 captures.

## Implemented tests

- Registry: 244 of 244 rows are complete.
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
- Registry-driven execution is zero on the candidate branch.
  On the reference branch, phase-8c executed slice 8A-S1: 21 rows
  report `results.status = pass`; phase-8e executed slice 8A-T1:
  14 more rows report `pass`; phase-8g executed slice 8A-J1:
  30 more rows report `pass`; phase-8i executed slice 8A-H1:
  29 more rows report `pass`; phase-8k executed slice 8A-H2:
  22 more rows report `pass`; phase-8n executed slice 8A-H3
  (partial: 22 of 25 roots, the upgrade triplet deferred):
  22 more rows report `pass`, so 138 pass and 106 remain `unrun`.
  Evidence: `/tmp/phase8c2-focused.log` (21 pass / 0 fail / 370 skip),
  `/tmp/phase8e-focused.log` (14 pass / 0 fail / 391 skip),
  `/tmp/phase8g-focused.log` (30 pass / 0 fail / 405 skip),
  `/tmp/phase8i-focused.log` (29 pass / 0 fail / 435 skip),
  `/tmp/phase8k-focused.log` (22 pass / 0 fail / 464 skip),
  `/tmp/phase8n-focused.log` (22 pass / 0 fail / 486 skip).

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
  That is 56 missing coverage row refs and 32 roots triaged nowhere on the candidate.
  On the reference, 26 of those 32 have a mapped row and 6 are pending-only.
- All 244 `cand_symbols` mirror `ref_symbols` exactly.
  They are placeholders, not derived candidate facts.
- All 244 `snapshots.cand` lists are empty.
- The union `legacy_roots` list still marks all 302 roots pending.
  It was never reconciled with the slice mappings.

## Negative-control status

- Reference: 18 negative-control tests, all pass.
- Reference: argv0 mask holds fail-closed with negative controls.
- Candidate: html mask tests 6 of 6 pass, including negative controls.
- Candidate: mask forgiveness fired 0 times in the full run.
- Registry: 478 negative checks exist across 244 rows, all unrun.
- Candidate: no dedicated negative-controls module exists.

## Uncovered inventory

- App journeys: showcase 23 pages, holla 34 scenarios and 15 routes,
  jackin 8 scenarios and 12 routes, tablepro 2 screens and 21 surfaces.
- Registry rows by app: showcase 94, jackin-preview 47, tablepro 28, holla 75.
- Holla is the largest gap: 75 rows against 34 scenarios.
- Legacy roots: 258 distinct roots triaged in slices.
  253 have a mapped row, 5 are pending-only.
  49 roots still pending.
- Pending roots by app: holla 49, jackin 0, showcase 0, tablepro 0.
- 16 slice legacy entries are marked pending, not mapped.
- 32 roots appear in more than one slice.
  24 roots are mapped by more than one slice.
- `TOOSMALL-NOTICE-001` has no reference snapshots.
  It is a behavioral probe.
- No full-suite rerun exists at either current tip.
- No registry-driven runner exists yet; the 21 S1 rows plus the 14 T1
  rows plus the 30 J1 rows plus the 29 H1 rows plus the 22 H2 rows
  plus the 22 H3 rows were executed as live-PTY tests
  (`showcase_pending_s1.rs`, phase-8c; `tablepro_pending_t1.rs`,
  phase-8e; `jackin_pending_j1.rs`, phase-8g; `holla_pending_h1.rs`,
  phase-8i; `holla_pending_h2.rs`, phase-8k; `holla_pending_h3.rs`,
  phase-8n), so 106 rows remain unrun as cases.

## Evidence paths

- Tips: `git rev-parse origin/visual-baseline origin/termrock-implementation`.
- Registry: `tests/scenario-registry/v1/registry.json` plus the slice files.
- Reference run: `/tmp/phase3z2-worker.md`, `/tmp/phase3z2-review.md`, `/tmp/phase3z2-nextest.log`.
- Candidate run: `/tmp/phase4a-worker.md`, `/tmp/phase4a-review.md`, `/tmp/phase4a-full2.log`,
  `/tmp/phase4a-triage.tsv`.
- Journeys: `/tmp/phase2-journeys.md`.
- Machine-readable twin of this file: `docs/testing/coverage-measurement.json`.
