# Test-integrity review — showcase / jackin / theme / holla parity stream

Reviewer: independent reviewer (read-only). No product code was changed by this
review.

| Item | Value |
| --- | --- |
| Reviewed sha | `bea5350ec99b0652335714275fd3f0d078399bca` (`feat(holla): width-band layout policy and a real history setting`) |
| Review worktree | `/Users/donbeave/Projects/review-tests-20261022` (detached, private `CARGO_TARGET_DIR`) |
| Parent of the first increment | `5dfff7f20f8adfeb9c0438040ee0b3db58bcc0b1` |
| Parent of the reviewed sha | `a972caaf1d888db8f654f59e545aa0ca7c47e26f` |
| Frozen oracle | tag `visual-baseline`, commit `4169783ae883788186d161af09ba9c481bd84e5c`, tree `ad0f7cf8899df8c5fea3ced3f60adf0ac9a2d297`, tuiscotti `a1ea27f757d75d07b05499276cd932e9f8e067ad` |
| Oracle frames | `target/frozen-baseline/source/snapshots/` after `python3 tools/frozen_baseline.py --repo . --root target/frozen-baseline prepare` (re-run here into `frozen-baseline/`, `EXIT=0`, `snapshots_sha256 6b547179584d2fb708993fb5ccd415a89423547263836271105b52be0294e1a8`) |
| Increments in scope | `ef35a7f6b`, `493f9054f`, `150320ab3`, `fc2c3bd4e`, `a972caaf1d`, `bea5350ec9`, plus the theme oracle commit `ec783f6c50` and the conformance commit `37f55475f7` inside the same range |

## Findings

`crates/jackin-preview-app/tests/baselines/jackin.txt:2`: 🛑 blocker: the 8 re-blessed
`accounts-mixed` digests cover the accounts **overview/registry** state, for which the
frozen oracle has **no approved frame** (the oracle approves only
`accounts/{detail,drawer,filter,add_form,add_form_required}`), so the re-bless carries no
oracle evidence. Bind the digests to an oracle-approved state, or record a reference
decision for the uncovered state.

`crates/jackin-preview-app/src/app/accounts_page.rs:1051`: 🛑 blocker: the new live
accounts page no longer reproduces the frozen approved frame. At the increment parent
`bdf62e3bea` the app rendered the approved `accounts/detail/120x40/truecolor` frame
**digest-identically** to the oracle (`2171bf1497db0aeb`, decoded from
`frozen-baseline/source/snapshots/jackin/accounts/detail/120x40/truecolor.ansi`); at
`bea5350ec9` the same state renders `6bf272a51b6de15c` — 22 of 40 rows and 805 of 4800
symbols, 736 foregrounds and 136 backgrounds differ (the credential chain
`chainargos.1password.com ·` / `1password`, the quota reset times, the `1,420 / 5,000`
figure, the `· quota readable` clause, the `Status` row and the
`Refresh Validate Edit… Set default Disable Remove…` footer are gone). Restore the
approved inspector content.

`crates/holla-ui/tests/visual.rs:25`: 🛑 blocker: the holla digest harness was moved to
`App::for_scenario_without_history` but `crates/holla-ui/tests/baselines/app.txt` was not
re-blessed, so the baseline is red
(`digest of 'first-use 80 24 junie truecolor' changed: baseline e30768d9339a0fb9, got
83a32521d5113b5d`). The baseline now records the `HOLLA_NO_HISTORY=1` projection while
the shipped default — history on, "the way the frozen reference runs it" — is no longer
covered by any digest. Re-bless the shipped configuration and cover the no-history
projection separately.

`crates/holla-sim/src/catalogue.rs:894`: 🛑 blocker:
`python3 tools/acceptance_architecture.py` reports `architecture_checks_passed: false`
with a `frame-marker` finding on this comment (the second finding is
`crates/holla-ui/src/app.rs:3908`); the parent `a972caaf1d` reports
`architecture_checks_passed: true`, `findings: []`. Reword the two comments so the
fail-closed architecture gate is green again.

`crates/termrock-conformance/tests/conformance.rs:47`: 🔶 major: `legacy_key_chords` is
an allowlist that exempts a consumed chord from "consumed must be declared in the binding
table" (`crates/termrock-test-support/src/conformance/driver.rs:1511`), and the range
grew it from 1 chord to 29 (`PICKER_LEGACY_CHORDS`, line 114), 28
(`QUERY_LEGACY_CHORDS`, line 83), 28 (`CODE_EDITOR_LEGACY_CHORDS`, line 52) and 2
(`CONTEXT_MENU_LEGACY_CHORDS`) for `TextInputCase`, `FieldCase`, `TextAreaCase`,
`FormCase`, `CodeEditorCase`, `FilterListCase` and `PickerCase`. Declare the chords in
`bindings()` instead of exempting them.

`crates/termrock-conformance/tests/conformance.rs:2599`: 🔶 major: the `CodeEditorCase`
audit target was swapped. The parent audited the live **editing** table
(`binding_states() = [FOCUSED|EDITING]`, per the comment removed by this range); the
reviewed sha audits the default `[empty, FOCUSED]` nav table and exempts the editing
chords through `legacy_key_chords`, so the editing table is no longer cross-checked at
all. Restore `binding_states()` for the code editor.

`crates/termrock-conformance/tests/baselines/components.txt:1`: 🔶 major: 1060 of 2562
rows were re-blessed in one commit whose message is a single line
(`fix(conformance): achieve 100% control-state and baseline parity`) with no oracle
evidence; the pre-bless baseline was already red for 232 rows across many commits
(`docs/audits/2026-10-22-conformance-red-triage.md`), so the re-bless closes a drift
rather than restores oracle-verified values. The re-bless is also stale at the reviewed
sha: `render::components::picker_chain::{disabled,pressed} 120 40 junie truecolor` fail.
Record per-row oracle evidence, or restore the last oracle-verified values and fix the
source instead.

`crates/jackin-preview-app/tests/app_tests.rs:1780`: 🔶 major: the drawer case lost its
approved-cell assertion. It used to assert that the crumb cell `(43,0)` is `"A"` with
foreground `0xb3_b3_b3`, background `0x000000` and no `BOLD`; it now asserts only that
the crumb text is absent, so no cell-level check remains for the drawer header row.
Keep a positive approved-cell assertion for the drawer.

`crates/jackin-preview-app/tests/app_tests.rs:2049`: 🟡 minor: the inspector check
`contains("Inspector ·")` was replaced by `contains("Tab Actions")`; `Tab Actions` is
the shared drawer footer, present in the frame for other reasons, so the assertion no
longer proves that the inspector opened. Assert the inspector's own label.

`crates/jackin-preview-app/tests/preview.rs:185`: 🟡 minor: the accounts-label
assertion `contains("Engineering")` was replaced by `contains("1Password available")`
(`crates/jackin-preview-app/src/app/accounts_page.rs:1369`), so the rendered account
label is no longer covered. Keep both labels.

`docs/architecture/termrock-migration-record.md:93`: 🟡 minor: the record states
`architecture_checks_passed | true` for the reviewed tree, but the gate is false at
`bea5350ec9` (see the finding above). Update the record.

`crates/termrock-theme/src/downgrade.rs:701`: 🟡 minor: the six per-family mono rule
readers added by `a972caaf1d` return the built-in tables they are meant to cover
(`VIEWPORT_MONO_RULES.to_vec()` and friends), so the "reserved mono rule coverage check"
now compares each table with itself and can no longer detect a wrong rule set. Pin the
expected rule sets instead of re-reading them.

No `#[ignore]` was added or removed anywhere in the range, no timeout was raised in any
test, no retry was added, and no test file was deleted
(`git diff --name-status 5dfff7f20f bea5350ec9` shows only 4 deletions, all
`crates/jackin-preview-app/src/app/accounts_*.rs` production views).

## Defect-detection experiment

Both new regression test files were applied to their pre-increment parent, alone, and
run.

1. `crates/jackin-preview-app/tests/accounts_page.rs` at parent `bdf62e3bea`
   (`git checkout --detach bdf62e3bea` in a scratch worktree, copy the file only):

   ```
   cargo nextest run -p jackin-preview-app --test accounts_page
   Summary [ 0.205s] 4 tests run: 1 passed, 3 failed
   FAIL the_accounts_page_has_no_size_keyed_screen
   FAIL the_accounts_page_renders_the_live_registry_tree
   FAIL the_accounts_page_is_the_same_page_at_every_size
   EXIT=100
   ```

   The new test file detects the frozen accounts frames. The one test that passes at
   the parent (`the_accounts_page_follows_the_live_selection`) is the weak one: the old
   page also carried the crumb and the credential.

2. `crates/showcase-demos/tests/no_false_parity.rs` at parent `ef35a7f6b`:

   ```
   cargo nextest run -p showcase-demos --test no_false_parity
   Summary [ 0.118s] 7 tests run: 4 passed, 3 failed
   FAIL no_page_replays_a_historical_frame
   FAIL no_page_switches_on_an_exact_page_size
   FAIL no_page_holds_a_captured_cell_table
   EXIT=100
   ```

   The new test file detects the captured cell tables and the exact-page-size branches
   in the task-runner and terminal pages. The four rendering assertions pass at the
   parent because `ef35a7f6b` had already made the chrome page live; they are
   containment checks, not defect detectors, for that page.

## Verdict on the 8 re-blessed `accounts-mixed` digests

**Not justified by the frozen oracle.**

* Self-consistency: the app at `bea5350ec9` renders
  `accounts-mixed 120 40 junie truecolor = 84a815016d3be882`,
  `100 30 junie truecolor = be7cc5dcb1dd422b`,
  `120 40 junie mono = 054bbebc2d8558c1`,
  `100 30 junie mono = 24de4845abdc063a` — byte-identical to the re-blessed file
  (verified by rendering the app through `termrock_test_support::Harness` and
  recomputing `Scene::digest()`). The re-bless records the current output, not a
  verified value.
* Oracle coverage: the frozen oracle approves five accounts screens
  (`detail`, `drawer`, `filter`, `add_form`, `add_form_required`) at 5 sizes x 5 colour
  modes. **None of them is the overview/registry state the 8 digests cover.** The
  digests therefore have no oracle counterpart at all.
* Oracle regression: for the state the oracle does approve, the increment regressed.
  The oracle frame `accounts/detail/120x40/truecolor.ansi` decodes to
  `2171bf1497db0aeb`; the parent `bdf62e3bea` renders exactly that digest when the
  `Work` account is selected (4 x `Down`), while `bea5350ec9` renders
  `6bf272a51b6de15c` for the same state — 22 of 40 rows differ at 120x40 and 28 of 30 at
  100x30 against **every** approved accounts frame. The new page matches no oracle
  accounts frame at all.
* The pre-increment baseline was itself already stale for `accounts-mixed 100 30 junie
  truecolor` (baseline `1078e1535637988f`, app at `bdf62e3bea` renders
  `4427ad1b1eabafb3`), so part of the re-bless silently absorbed an older drift.

The theme increment `ec783f6c50`, by contrast, **is** oracle-backed: the approved
`showcase/pages/codeeditor/120x40/256.ansi` frame paints `Indexed(111)` x52,
`Indexed(116)` x18, `Indexed(183)` x14, `Indexed(245)` x306 and `Indexed(242)` x149, and
`truecolor.ansi` paints `Rgb(121, 184, 255)` x52 (0x79b8ff keyword),
`Rgb(139, 213, 202)` x18 (0x8bd5ca string), `Rgb(138, 138, 138)` x251 (0x8a8a8a muted)
and `Rgb(115, 115, 115)` x148 (0x737373 faint) — exactly the values the commit and the
updated `crates/termrock-theme/tests/capability_palettes.rs` assert.

## Expected-inventory deltas

* Reference inventory untouched: `baselines/tuiscotti-v1/` has **no** change in the
  range. `baselines/tuiscotti-v1/corpus-index.json` still records 302 screens / 7550
  captures / 75500 artefacts (holla 125 screens x 25, showcase 84 x 25, jackin 49 x 25,
  tablepro 44 x 25 = 302 screens, 5 sizes x 5 colour modes). The frozen source still
  carries the 28 895 registered scenario cases (`EXPECTED_SCENARIOS = 28_895` in
  `tools/frozen_baseline.py`, unchanged in the range) plus the 20 ordinary checks.
* Conformance pending inventory unchanged: 351 `#[ignore]` cases in
  `crates/termrock-conformance/tests/visual_baseline/` at the parent and 351 at the
  reviewed sha.
* `crates/termrock-conformance/tests/baselines/components.txt`: the key set is
  identical (2562 rows before and after, `diff` of the sorted
  `name w h theme colour` keys is empty); only 1060 hashes changed. No coverage shrink,
  but the hashes are unverified (see the finding).
* New coverage added by the range:
  - `crates/showcase-demos/tests/no_false_parity.rs`: 7 new tests (3 source guards over
    chrome/taskrunner/terminal + 4 render checks at 2-4 sizes).
  - `crates/jackin-preview-app/tests/accounts_page.rs`: 4 new tests (live registry tree
    and counts, same page at two sizes, live selection through the real key path, and a
    source guard against size-keyed screens).
  - `crates/termrock-conformance/tests/conformance.rs`: a new
    `activation_probe_guards` module with 5 tests (1 positive, 3 `#[should_panic]`
    faults, 1 theme fingerprint test) plus the `Conformance::fixture()` /
    `needs_layer_opened` migration of the shared driver.
  - `crates/showcase-ui/tests/shell_chrome.rs` (+40), `crates/tablepro-ui/tests/tab_list_owner_contract.rs`
    (+74), `crates/termrock-consumer-example/tests/consumer.rs` (+350),
    `crates/termrock-fields/tests/keyboard_editor.rs` (+259),
    `crates/termrock-grid/tests/grid_viewport_bounds.rs` (+39),
    `crates/termrock-theme/tests/picker_title.rs` (+103),
    `crates/termrock/tests/picker_contract.rs` (+39),
    `crates/termrock/tests/public_component_contracts.rs` (+296).
  - Test-count deltas (parent `5dfff7f20f` -> reviewed `bea5350ec9`):
    jackin-preview-app 184 -> 188, showcase-demos 52 -> 53, showcase-ui 83 -> 87,
    holla-ui 144 -> 147, holla-sim 81 -> 81, termrock-conformance 579 -> 584.
* Frozen-artefact isolation holds: `tools/acceptance_policy.json` is unchanged and
  still lists `termrock-test-support`, `termrock-conformance`, `visual-harness`,
  `termrock-xtask` and `termrock-visibility-publisher` as forbidden production
  dependencies; the architecture checker reports `dependency_violations: []`,
  `application_shared_closure_missing: []`, `errors: []`, `unaccounted_inputs: []`.
  No production crate reads `snapshots/`, `baselines/`, `shots/` or `parity/` at runtime
  (the only `snapshots/`/`baselines/` string literals in library code are inside the
  `#[cfg(test)]` module of the non-published `termrock-e2e` crate,
  `crates/termrock-e2e/src/lib.rs:12227`, `:12446`, `:12491`).

## Negative controls

The conformance suite keeps its negative controls and they still fail when the behaviour
is wrong. 14 negative-control tests exist and all pass at the reviewed sha
(`cargo nextest run -p termrock-conformance --all-targets`): 6 in
`crates/termrock-conformance/tests/visual_baseline/negative_controls_phase6a.rs` and 8 in
`negative_controls_phase8ap.rs` (the same count as at the parent). Each injects a defect
— mutated foreground, dropped cursor, shrunk dimensions, truncated ANSI, mixed
`.txt`, un-reported drag, boundary clocks — and asserts the checker rejects it; e.g.
`negative_phase8ap_fg_mutation_fails_ansi_keeps_txt` and
`negative_phase8ap_single_cursor_visibility_and_y_detected` PASS in the run recorded
below. The single edit in that range
(`negative_controls_phase8ap.rs:459`, adding `ReferenceState::FOCUS_VISIBLE` to the
focused state) makes the control stricter, not weaker.

## Commands run and results

Private `CARGO_TARGET_DIR` per worktree; `cargo nextest` throughout.

| Command | Result |
| --- | --- |
| `cargo nextest run -p holla-ui --all-targets` @ `bea5350ec9` | 147 run, 145 passed, **2 failed**: `holla-ui::visual holla_visual_baseline`, `holla-ui app::tests::menu_bar_pointer_hover_changes_only_the_file_title_and_clears_on_exit` (EXIT=100) |
| `cargo nextest run -p holla-ui --all-targets -E 'test(~menu_bar_pointer_hover) or test(~holla_visual_baseline)'` @ parent `a972caaf1d` | 2 run, 0 passed, **2 failed** with the identical messages (EXIT=100) — both failures pre-date `bea5350ec9` |
| `cargo nextest run -p showcase-demos -p showcase-ui -p jackin-preview-app --all-targets` @ `bea5350ec9` | 327 run, 320 passed, **7 failed**: `jackin-preview-app::visual jackin_visual_baseline` (`returning 120 40 junie mono`), `showcase-ui::visual showcase_visual_baseline` (`Overview 80 24 junie 256`), `showcase-ui::app_tests editor_playground_keeps_live_find_completion_and_run_contracts`, `editor_shell_completion_accepts_enter_then_tab_indents`, `editor_shell_completion_accepts_tab_then_next_tab_indents`, `showcase-ui::page_headings all_page_headings_match_the_reference_capture_matrix`, `all_shell_headers_match_reference_captures_and_visibility_thresholds` (EXIT=100) |
| `cargo nextest run -p jackin-preview-app -p showcase-ui --test visual` @ parent `a972caaf1d` | 2 run, 0 passed, **2 failed** with the identical digests (`Overview 80 24 junie 256`, `returning 120 40 junie mono`) — the two visual-baseline failures pre-date `bea5350ec9` |
| `cargo nextest run -p termrock-conformance --all-targets` @ `bea5350ec9` | 1467 run, 1465 passed, **2 failed**: `render::components::picker_chain::disabled` and `::pressed` (`120 40 junie truecolor`) (EXIT=100) |
| `cargo nextest run -p showcase -p holla-sim --all-targets` @ `bea5350ec9` | 83 run, 83 passed, 0 failed (EXIT=0) |
| `python3 tools/acceptance_architecture.py` @ `bea5350ec9` | `architecture_checks_passed: false`, 2 `frame-marker` findings, `dependency_violations: []` (EXIT=1) |
| `python3 tools/acceptance_architecture.py` @ parent `a972caaf1d` | `architecture_checks_passed: true`, `findings: []` (EXIT=0) |
| `python3 tools/frozen_baseline.py --repo . --root frozen-baseline prepare` @ `bea5350ec9` | EXIT=0, oracle prepared from `4169783ae883788186d161af09ba9c481bd84e5c` |
| `cargo nextest run -p jackin-preview-app --test accounts_page` @ `bdf62e3bea` + new file only | 4 run, 1 passed, **3 failed** (EXIT=100) |
| `cargo nextest run -p showcase-demos --test no_false_parity` @ `ef35a7f6b` + new file only | 7 run, 4 passed, **3 failed** (EXIT=100) |

Not run here: the full workspace gate (`cargo nextest run --all-targets` across all
crates), `cargo clippy --all-targets` and `cargo fmt --check`. The focused suites
listed above were run instead; no claim is made about the packages not listed.

## Reproducing the oracle comparison

`ansi_digest.py` in this directory decodes a tuiscotti `.ansi` oracle frame into
`{symbol, fg, bg, modifier}` cells and computes the same FNV-1a digest as
`crates/termrock-test-support/src/digest.rs` (`Scene::digest`). The decoder is
self-validating: applied to
`frozen-baseline/source/snapshots/jackin/accounts/detail/120x40/truecolor.ansi` it
yields `2171bf1497db0aeb`, and the pre-increment app (`bdf62e3bea`) renders exactly that
digest for the same state, while the reviewed sha renders `6bf272a51b6de15c`.

```
python3 ansi_digest.py \
  frozen-baseline/source/snapshots/jackin/accounts/detail/120x40/truecolor.ansi 120 40
```

## Verdict

**HELD.** Bounded scope: the stream `ef35a7f6b` .. `bea5350ec9` (plus `ec783f6c50` and
`37f55475f7` inside the same range) is not cleared.

Blocking items, all inside the reviewed sha:

1. The holla digest baseline is red and the shipped default configuration is no longer
   covered (`crates/holla-ui/tests/visual.rs:25`).
2. The fail-closed architecture gate is red at the reviewed sha
   (`crates/holla-sim/src/catalogue.rs:894`, `crates/holla-ui/src/app.rs:3908`).
3. The 8 re-blessed `accounts-mixed` digests have no frozen-oracle counterpart, and the
   new live accounts page regressed against the one approved accounts frame the oracle
   does hold (digest-identical before the increment, divergent after it).

Major items: the `legacy_key_chords` allowlist expansion and the `CodeEditorCase`
audit-target swap in the conformance suite, the unevidenced 1060-row
`components.txt` re-bless (which is itself stale for two rows), and the inverted
approved-cell assertion in the jackin drawer case.

Cleared items: the showcase guard test and the accounts page test both fail on their
pre-increment parents; the frozen-artefact isolation holds; the 302-screen / 7550-capture
reference inventory and the 351-case conformance pending inventory were not shrunk; the
negative controls are intact and still reject broken input; the theme palette increment
`ec783f6c50` is corroborated by the approved oracle frames; `-p showcase` and
`-p holla-sim` are green (83/83).

Note on the showcase coverage gate: `AGENTS.md` names
`src/bin/showcase/app_tests_catalog.rs` and `src/catalog.rs`, but no such files exist in
this layout at the reviewed sha (the showcase code lives in `crates/showcase*`), so the
gate named by `AGENTS.md` cannot be run as written. The showcase package's own tests
(`-p showcase`) pass.
