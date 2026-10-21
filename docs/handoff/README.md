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


## Parent Picker exact-head gate preservation (2026-10-21)

Preservation branch `wip/parent-picker-gate-20261021` was pushed and remotely verified at `9f6659a51b793fe60f9ffee86e986afc1d4824ba`. The integration worktree is `/Users/donbeave/Projects/termrock-worktrees/pr38-style-delivery-20261020`; its 1301 pre-existing tracked target-artifact deletions remain unstaged and outside this work. Picker/keymap source has independent bounded clearance, but the parent combined gate has three mouse/scroll failures in `termrock-navigation::list_row_extent`, with 900 passes and three skips. Do not treat the WIP as accepted or merge it without diagnosis.

Next: finish the independent unchanged-test comparison against PR38 `5f656a1833e543fb39723498513533e5753b257e`; classify and repair any introduced failures, preserve pre-existing failures explicitly, then run focused exact-head tests, strict scoped checks, synchronize under the shared push lock, push through PR38 and verify the remote SHA. Do not skip the three cases or update their expected values. Detailed failure names and the evidence log are in the current migration matrix. Main, design/PR26 and frozen references remain read-only.

## Shell geometry contract (2026-10-21)

- Tip: local commits begin at `6b73822c361e3a253709ad71fe6c885693fd1135`, followed by geometry measurement correction `e6a4829a5dcd0f0a1bad03467713c3f5a4858aca`; the public return-type fix and owned-leaf runtime adapter are the current uncommitted follow-up. Branch `wip/shell-geometry-contract-20261021` has not been published and needs rebase onto current PR #38.
- Item: unify `Chrome`/`AppShell` ownership in `termrock-shell`, share resolved header/body/footer geometry, preserve both public `CompositionSlots<'a>` return types, and use one existing composition renderer with an owned leaf adapter.
- State: independent geometry review is green. The next API review found source/lifetime compatibility failures in direct slots methods and those exact unchanged probes are now included as `api_legacy_compat.rs`. Scoped shell tests 55/55, runtime composition consumer 7/7, facade slots 10/10, full runtime package 382 pass/3 skipped; scoped Clippy exits 0 with existing `hit.rs:236` pedantic warning, fmt and diff-check pass. Evidence packets: [geometry/API review](../audits/evidence/2026-10-21-shell-geometry/shell-geometry-api-review-6b73822.json), [return/lifetime compatibility review](../audits/evidence/2026-10-21-shell-geometry/shell-geometry-api-review-e6a4829.json), plus the [current audit](../audits/2026-10-21-shell-geometry-contract.md). Independent review of the owned-leaf correction is still required. No application parity claim.
- Next: commit the current correction, then use the shared push lock only when available, fetch/rebase on `origin/parity/termrock-visual-baseline`, rerun focused gates on the exact rebased head and publish only WIP for independent review. No PR #38 integration until independent review passes. Resize invalidation/input dispatch remains unverified and out of scope.


## Parent blank New-connection validation hold (2026-10-21)

Temporary preservation branch: `wip/parent-tablepro-blank-validation-20261021`. The commit adding this entry preserves author `f0760cc204afc533e202604d0437f913b72bfd10` and parent integration `e404a63a62af0f4a9fa985ddc668c924296f173a`. Working copy: `/Users/donbeave/Projects/termrock-worktrees/pr38-style-delivery-20261020`. Native Domain/UI has 205 passes and three failures; source-qualified independent diagnosis distinguishes unsupported Database-required validation and stale Name/prefill assumptions. This is WIP protection, not PR38 acceptance.

Next: land the bounded domain validation fix after independent review, keeping Name/Port validation and footer Unicode assertions; enter Name through UI in the reconnect fixture. Review and land `c8f92ea0c0df2bc13a507e97b14ef6f65a90ea48` helper removal, then the separately reviewed shared radio contract. Rerun the complete affected native inventory, retain any unresolved reference/behavior failures, synchronize under the push lock, push only PR38 and verify the remote SHA. Remove this task-owned preservation ref only after integration/proof; leave fixed design/main/tag untouched.

## Parent preservation refs retired (2026-10-21)

The parent Picker and blank-draft WIP refs listed above are fully integrated and were retired with independently reviewed exact-tip atomic leases. Their source and records remain in PR38; do not recreate these completed preservation refs. See the current migration matrix and committed cleanup result. Remaining active component WIPs and the user working-tree backup remain preserved.
## Visible terminal caret retention (2026-10-21)

- WIP branch: `wip/visible-terminal-cursor-20261021`; implementation commit `df88318015e580cae57130693c651b01b7bedc22`, rebased onto PR #38 source `4f40bb294da89bbfcb2a6716c88a3158427aa0b2` (original isolated base was `5b9cffdb05fb257f7416dc872e24e75865ad98c2`).
- Scope: additive `TextViewport::caret_when_unfocused`, `ContextMenu::retain_cursor`, and `Ui::retain_visible_owner_cursor`; strict existing retention contract unchanged. Includes a real public-API log-console integration flow, runtime owner/layer/typing tests, and [audit](../audits/visible-terminal-cursor-retention-20261021.md).
- Verification: scoped nextest 563/563 passed with 3 skipped; scoped Clippy, format check, and diff check passed. A deliberate temporary restoration of legacy cursor-owner selection made `real_editable_overlay_typing_suppresses_retained_owner_when_caret_is_absent` fail by selecting the retained terminal cursor; implementation restoration passed. The paired baseline/CLI capture gate was not run.
- State: commit is local only because the shared push lock is currently owned by `wip/menu-danger-profile-20261021` at `2026-10-21T02:42:11Z`. No lock was removed or modified.
- Repository-wide `cargo fmt --check` after the rebase reports one unrelated formatting-only blank line in `crates/jackin-preview-app/src/app.rs:261`; the scoped format check for the three owned packages passes. This WIP did not change that application file.
- Next: wait for lock release, fetch/rebase normally if the remote WIP exists, rerun focused gates on the exact head, push and verify remote SHA. Obtain independent review before integrating into PR #38. This work does not claim Showcase coverage or four-application parity.

## Visible-caret current integration status (2026-10-21)

The author checkpoint above is historical. Source `8be128f073c19076a2977e349e8732cd74f08821` is already published on its WIP and independently cleared for the additive runtime contract. Parent integration now awaits exact-head checks and PR38 publication. Do not repeat the old lock wait or claim application adoption; the original workspace formatting claim is corrected in the audit.

## Seven integrated component WIPs retired (2026-10-21)

The canonical prepare stages one/two, Editor live API, Menu danger profile, Panel composition, visible cursor and master-detail policy remote WIPs are fully integrated and independently retired with exact-tip atomic leases. Preserve their local worktrees and evidence, but do not recreate their obsolete remote refs. See the current migration matrix and seven-ref retirement result for exact identities. Active BackTab, Button profile, Showcase Editor and other unreviewed work remain protected.

## Showcase Editor broad-gate hold (2026-10-21)

Preserve reviewed live Editor source and current records on `wip/parent-showcase-editor-validation-20261021`. The combined all-target suite ran203cases:200passed,threeheading/visualfailures,zeroskips. NoPR38push occurred. Independently compare exact161284db9before versus9c99356abafter in distinct targets, classify allthree and any masked Editor differences, then resolve regressions or record provenexistingfailures before publication. Do not bless candidate digests. See currentmatrix/log.
