# Migration handoff

Start with the current [Termrock migration and acceptance matrix](../architecture/termrock-migration-matrix.md). It records fixed source identities, application status, paired CLI coverage, architecture findings, unresolved differences, and next evidence. Treat every partial or missing result as incomplete work.

## Palette and shared-grid handoff (2026-10-21)

Preservation branch: `wip/palette-grid-repair-20261021`, tip `148c32c1bd7b646b032b3eaa739d1ee1338ecf2e` before this handoff update. The work is based on PR #38 commit `5c0075a09eec51b805f15d79ad945d4fb51fb5bf` and consists of:

- `7d5246b1b487dccfca76648bf13be13e2577026d`: remove exact-RGB global conversion exceptions; keep the frozen Junie ANSI-256 semantic values for `on_accent=232`, `border_strong=242`, and `disabled_fg=239`.
- `3a8b18362954da068e1c3418817fcb9d25c0ab37`: apply existing `Ui::scroll_edges` behavior in shared Grid and remove TablePro's private bottom-edge repaint.
- `148c32c1bd7b646b032b3eaa739d1ee1338ecf2e`: state the unresolved app-level repaint limitation accurately in the audit.

Focused checks: `cargo nextest run --locked -p termrock-grid` passed 125/125. The filtered theme check passed 2 tests, with 121 filtered. `cargo nextest run --locked -p tablepro-ui` passed 167/180 and failed 13; failures were not reproduced against the pre-change source, so their attribution remains unresolved. This is not application parity evidence.

Important limitation: TablePro's `draw_painted_grid` / `paint_scrolled_cells` runs after the shared Grid renderer and can overwrite the new shared fade. Thus direct Grid behavior is verified, but the application-visible final cells are not proven fixed. Remove that repaint only after mapping its row-number, cell-style, hit/focus and scrolling contracts to shared components, then compare the real CLI against the frozen baseline. An attempted merge into the existing WIP and theme-palette branches conflicts; integrate these commits through the PR #38 source branch after review and revalidation.

Next: independently review the exact commits, integrate onto current PR #38, repair the actual TablePro post-render repaint in its owning component migration, and only then continue the compatible per-color-level Split/Seam recipe work. Do not claim visual parity or treat the 13 failures as pre-existing.

## Fields integration blocked by incoming runtime change (2026-10-21)

Preservation branch: `wip/parent-fields-runtime-blocker-20261021`. The exact Fields source `332f40d07e38512f91cfb038eb9174c413755686` and its independent review are committed. Before synchronizing incoming PR #38 changes, scoped Fields/Forms/Runtime verification passed 504 tests with three existing skips, strict Clippy and scoped formatting. After normally merging shared PR #38 through `adf334b21745b9f713c26275987d09bf22eeda16`, the same suite has 503 passes and one allocation-test failure. Publication to PR #38 stopped.

The unchanged isolated allocation test fails with inherited `NO_COLOR` present (one allocation versus zero expected), and passes when that variable is absent. Incoming `modal_backdrop` now reads `std::env::var_os("NO_COLOR")` on each paint. Independent diagnosis and a shared-layer repair are required; removing the variable is diagnostic evidence, not a gate workaround. Preserve incoming history and all reference artifacts. See [exact blocker evidence](../audits/2026-10-21-fields-publication-incoming-runtime-blocker.json).

Next: independently review the incoming runtime change, fix the per-paint environment dependency without changing the test or hiding failures, rerun the complete scoped gate under the original environment, normally synchronize/push PR #38, and verify its remote SHA. All four applications remain incomplete.
## Showcase Selection bypass removal (2026-10-21)

Landed on `parity/termrock-visual-baseline`: the Selection page's ANSI
frame-replay bypass (`selectable/frames.rs` + two `.ansi` fixtures) is
replaced by the shared overlay `Dialog` and real selection rendering; the
shell no longer calls `selectable.stamp(ui)`. Evidence, the two
fixture-masked 120x40 captures that now fail on out-of-scope shell/nav/
footer drift, and the remaining lower-tier backdrop question are recorded
in `docs/audits/2026-10-21-selectable-bypass-removal.md`. Next steps:
have the shell owners reconcile nav/sidebar/footer painting with the
frozen captures (then re-approve on purpose), and hand the tier-dependent
modal-backdrop mapping to the runtime owners.
||||||| parent of 7201b3497 (docs: record PR #38 landing-queue integration state)

## Landing-queue integration state (2026-10-21)

Queue integration onto `parity/termrock-visual-baseline` is complete through step 7; pushed tip `bfbb72516` (sequence `8a8e28300` → `499cec504` → `efda9593a` → `2d66ea9be` → `bfbb72516`, all gated and pushed under the push lock). Full landing/skip/hold record: [migration matrix, PR #38 landing-queue section](../architecture/termrock-migration-matrix.md).

Resume points:

1. `parity/buttons-reference-decision-20261020` executable pins: re-advance the Buttons reference decision through the cohort-review machinery (six reviewed audit docs pin `frozen_cli_references.json` at `2a43fb11…`; branch pins `d5d8d648…`). Docs commits already landed.
2. `parity/tablepro-picker-live-20261020`: DONE 2026-10-21. Re-ported onto the landed `query_input` design; both frozen tab-picker geometry cases pass at both sizes via the real shared Picker path. Three live contracts landed in `quick_switcher_contract.rs`; the branch's dirty-delete confirm flow is superseded by the recorded landed discard decision and its chosen-tab-focus test was already covered by `tab_list_contract`. Record: [migration matrix, "tablepro-picker-live re-port"](../architecture/termrock-migration-matrix.md).
3. Still held per queue: `wip/textarea-fields-stage3-20261021`, `wip/steps-shared-contract-20261021`.

## wip/termrock-extract-shell-20261021 (shell-layer extraction, 2026-10-21)

LANDED 2026-10-21: merged as `70229e698` onto then-origin tip `01ed92ca9` and pushed to `parity/termrock-visual-baseline` as `fab2e44dd` (48/48 shell tests, clippy/fmt clean, `cargo check --workspace` exit 0 on the exact pushed HEAD). Nothing left to resume; record in the [migration matrix](../architecture/termrock-migration-matrix.md).

- Tip: see `git log origin/wip/termrock-extract-shell-20261021 -1` (13 commits on top of `origin/parity/termrock-visual-baseline`).
- Item: PR #26 shell layer (commit `f4867e9b5`) ported into the new `crates/termrock-shell` — nav/Router, chrome, status, quit guard, keyboard deliverability, key router, page contract, AppShell slots, footer compose rules, reach harness, esc vocabulary, `shell_consumer` example, additive `termrock::shell` facade re-export, matrix rows.
- State: complete and gated (nextest 48/48, clippy clean, fmt clean, workspace `cargo check` exit 0, rebased onto `origin/parity/termrock-visual-baseline` at rebase time); no app crate or shared-crate source touched.
||||||| parent of aa2b1b34b (docs(audit): modal backdrop DIM conflicts inside the frozen baseline)

||||||| parent of e5c615d2a (docs: record theme extraction handoff and adoption list)

## Theme/visual-state extraction handoff (2026-10-21)

Worktree `/private/tmp/termrock-extract-theme-20261021`, branch tip pushed to
`parity/termrock-visual-baseline` (commits `be80fc53c`, `d23bf3043`,
`458f4cc85`, `b00925cb4`). Landed:

- `Theme::gutter_symbol` / `Theme::gutter` / `Theme::chosen_in` /
  `Theme::inactive_look` + `InactiveSelection` (Accent default, owner
  decision 2026-10-17) + `Theme::legible_mark` + pub `contrast_ratio`
  in `crates/termrock-theme/src/visual.rs` (PR26 `src/theme.rs:1811/1822/586`).
- Popup `Dismiss` knobs on the runtime layer type
  (`crates/termrock-runtime/src/layer.rs`): `pass_through`, `close_button`,
  `RefuseFeedback`, `allows(DismissReason)`, `shows_close()` (PR26
  `src/ui/popup.rs:24-60,121`).
- Modal backdrop tier fix toward the frozen baseline: `backdrop_style` no
  longer emits DIM at any tier (frozen captures carry zero SGR-2 runs);
  reference defect recorded in
  `docs/audits/2026-10-21-modal-backdrop-dim-vs-frozen-baseline.md`.
- Test ports: `crates/termrock-theme/tests/{focus_gutter_api,backdrop_tiers,esc_vocabulary}.rs`,
  `crates/termrock-runtime/tests/dismiss_knobs.rs`.

Adoption needed (not this workstream's files):

1. Controls/navigation/apps: paint focus gutters and chosen inks through the
   new theme helpers (controls `button.rs`/`choice.rs` paint via
   `Part::GUTTER`; `holla-ui/src/screens/home.rs:760` hand-paints `▎`);
   renderers read `InactiveSelection` via `Theme::inactive_look(policy,
   under_modal)` — policy default `InactiveSelection::Accent`.
2. `termrock` facade `crates/termrock/src/lib.rs:122`: re-export
   `RefuseFeedback` next to `Dismiss`.
3. Runtime wiring for the new knobs: `runtime.rs:1177` (`outside_click`),
   `:1762` (`esc`) and `layer.rs` focus-out should route through
   `Dismiss::allows(reason)`; `pass_through` and `close_button` rendering are
   unwired by design (defaults preserve behavior).
4. `crates/termrock-controls/tests`: port PR26 `tests/choice_containment.rs`
   (needs Checkbox/RadioGroup/Toggle hit-region internals).
5. `tests/esc_vocabulary.rs` `every_esc_label_starts_with_a_verb` is
   `#[ignore]`d until jackin `launch_cockpit.rs` replaces "Keep launching"
   (x2) with a verb-initial Esc label.
6. tablepro-ui `app_tests.rs` `unfocused_grid_cursor_does_not_style_the_cursor_row`
   still passes; revisit when navigation adopts `InactiveSelection::Accent`
   so the owner decision and the frozen capture are reconciled on purpose.

Process incident: a shared-stash race during this session consumed one
foreign `git stash` entry (its only visible content, deletions under
`crates/termrock-conformance/target/`, was reverted immediately). If your
stash disappeared around 07:15+07, check `git fsck --unreachable | grep
commit`. Do not use the shared stash stack from worktrees.


## TablePro shared Picker scope handoff (2026-10-21)

Preservation branch: `wip/tablepro-picker-scope-20261021`, implementation source `d911e577a1342729719a746a1b314bea7880be46` based on candidate `5f82c4d35d12e0fb77b0f580a8ef200240661602`. This branch contains the shared FilterList/Picker backward-scope and explicit selection-policy contract, TablePro opt-in wiring, regression tests, and audit `../audits/2026-10-21-tablepro-picker-scope.md`. A follow-up optimization in `crates/termrock-navigation/src/filter_list.rs` avoids query cloning/storage for default `Preserve` consumers; its focused tests and strict shared Clippy pass. The exact final preservation tip is the commit that adds this entry.

Verification is scoped, not full parity. The package nextest run had 545/557 passing and 12 failing; the same-source pre-change run had the identical 12 failures, listed in the audit. Focused navigation, Picker projection and real TablePro tests pass, as do strict shared Clippy, formatting, and `git diff --check`. Two deliberate negative probes failed as intended: removing BackTab scope binding and removing the Picker's query-reset rearm. Logs are preserved under `/private/tmp/termrock-tablepro-picker-scope-logs/`; package-before, package-final, package-verified, negative-probe, and focused/Clippy SHA-256 values are recorded in the audit or the agent handoff. No pinned real-CLI paired comparison was run; application parity is incomplete.

Publication blocker: the repository push lock was owned by `termrock-fix-findings` (`.git/push.lock`, owner timestamp `2026-10-21T07:34:14Z`) when checked. This worktree did not acquire or alter it. Parent coordination owns publication and independent review. Next: review this exact source against current PR #38; assess incoming TablePro `app.rs`, `tab_ownership`, and native-baseline changes separately; resolve the lock; then normally synchronize, rerun the required gate on exact HEAD, publish this WIP and verify the remote SHA. Do not blindly rebase or treat fixtures as accepted output. The pinned paired app comparison and the remaining TablePro tab-strip repaint/geometry work are still required.
