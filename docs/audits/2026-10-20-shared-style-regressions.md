# Shared style and painting regression audit

## Scope and fixed evidence

This audit isolates shared rendering regressions found while reviewing
TablePro's live picker consumer. The frozen product reference remains
`visual-baseline` source `4169783ae883788186d161af09ba9c481bd84e5c` (tag object
`2718a6b8127a522a3d351d0b4bdb7542fc8da542`); Tuiscotti remains pinned to
`a1ea27f757d75d07b05499276cd932e9f8e067ad`.

Reviewed source points:

- `a848219fd858ed6352fcb403b2cc5d5953083e6f`: TablePro picker migration and
  the introduced shared-style changes.
- `1a947a251ab9b045610b47729bf481419b9fca8d`: shared paired-runner base used
  for integration.
- `99b719d85a0714c88484240dbc9184f110b518d`: initial isolated fix branch base.
- `94a1215e27e67bfad76f9edb8ca5b00fa9810c80`: first committed repair, later
  reviewed and found incomplete for family-level theme and overlay precedence.
- Read-only suite records: `/tmp/termrock-current-source-review-a848-nextest.log`
  and `/tmp/termrock-current-source-review-1a-nextest.log`.

The review records the source suite at `a848` as 538 passing, 2 failing, and 3
skipped; the two allocation/style regressions below were red there. At `1a`,
the same two regression tests passed. New focused tests below exercise the
remaining explicit-style and opaque-surface contracts.

## Findings and disposition

| Finding | Evidence before fix | Disposition | Evidence after fix |
|---|---|---|---|
| `ScrollRegion` removed explicit scrollbar `BOLD` together with its default bold. | `cargo nextest run -p termrock-navigation every_override_channel_is_forwarded_to_scrollbar` failed at `steps.rs:1402`: track cell lacked `BOLD`. | Follow-up fix checks the fully resolved track/thumb style. It clears underlay-only bold while retaining bold added by a family override, overlay, or instance patch. | Existing Steps regression plus a new `Theme::override_family` track test pass. |
| Modal backdrop allocated a `String` for every cell, and cloned the theme while projecting each modal. | `cargo nextest run -p termrock-runtime warmed_modal_snapshot_projection_performs_zero_allocations` failed at `a848` with 548 allocations. | Fixed in `ui/paint.rs`: compare the borrowed cell symbol and borrow the UI theme/page fields directly. | The warmed modal projection test passes with zero allocations. |
| `FilterList::draw` skipped its container fill whenever incoming cells happened to have non-reset backgrounds, even when `preserve_surface` was false. It also could not honor theme/overlay container styles on preserved opaque surfaces. | Source branch `99b` checked opacity without consulting preserve policy or explicit `CONTAINER` channels. Instance-only follow-up missed `Theme::override_family` and overlays. | Added allocation-free `ResolvedStyle` explicit-channel metadata without changing public `Resolved`; component-author `patch_style` applies only explicit channels to preserved cells. FilterList obtains resolved theme/overlay/instance provenance and patches opaque surfaces without repainting symbols or inherited channels. | Tests cover opaque non-preserved fill, instance danger background, and themed danger background plus underline modifier on an opaque surface. |
| Focused `RowUi` selection fill replaced owner family overrides with selection defaults. | Source branch `99b`, `resolve_row_part`, forced selection after resolving owner style; initial `94a` repair preserved only instance patches. | Put selection style in `StyleDefaults`; family/overlay rules resolve above it and instance patches remain topmost. | New focused-fill test verifies themed built-in `Role::Danger` and underline modifier survive. |
| Picker overlay chrome discarded explicit `CONTAINER` foreground, underline color, and modifiers while backdrop-mapping inherited foreground. | Source branch `99b`, `overlay_chrome`, always used mapped backdrop foreground and cleared all modifiers. Initial `94a` repair preserved instance foreground/add only. | Resolve full explicit provenance and preserve inherited backdrop fg when no explicit fg is set; apply theme, overlay, and instance fg/bg/underline/modifier channels in precedence order. | Picker-chrome tests verify instance fg/modifier and themed fg/underline-color/modifier. Author patch test verifies clear/reset, removals, inherited channels, and symbols. |

No app-level paint workaround or test-specific visual path was added. Picker
title/query geometry and TablePro composition remain outside this change.

## Verification and review status

Before-fix red checks:

- Scrollbar: `cargo nextest run -p termrock-navigation every_override_channel_is_forwarded_to_scrollbar` failed on the missing track `BOLD` modifier.
- Modal projection: `cargo nextest run -p termrock-runtime warmed_modal_snapshot_projection_performs_zero_allocations` failed with 548 allocations on `a848`.
- Opaque FilterList surface: with the new tests temporarily run against the old opacity-only condition, `cargo nextest run -p termrock-navigation -E 'test(explicit_container_patch_reaches_an_opaque_preserved_surface) | test(container_fill_is_not_skipped_for_an_opaque_unpreserved_area)'` failed both checks (danger patch ignored; magenta backing left unchanged).
- Focused RowUi owner patch: with the new test temporarily run against the old selection-default replacement, `cargo nextest run -p termrock-runtime focused_fill_keeps_explicit_builtin_role_and_modifier` failed because highlight blue replaced danger red.
- Picker chrome owner patch: with the new test temporarily run against the old backdrop-only foreground and empty-modifier behavior, `cargo nextest run -p termrock-runtime picker_chrome_keeps_explicit_container_foreground_and_modifier` failed because the explicit danger foreground was replaced by the background sample.

After-fix checks on the isolated branch:

- Focused set: `cargo nextest run -p termrock-runtime -p termrock-navigation -p termrock-overlays -E 'test(every_override_channel_is_forwarded_to_scrollbar) | test(explicit_container_patch_reaches_an_opaque_preserved_surface) | test(container_fill_is_not_skipped_for_an_opaque_unpreserved_area) | test(focused_fill_keeps_explicit_builtin_role_and_modifier) | test(picker_chrome_keeps_explicit_container_foreground_and_modifier) | test(warmed_modal_snapshot_projection_performs_zero_allocations)'` — 6 passed, 0 failed.
- Original package suites (first commit): `cargo nextest run -p termrock-runtime -p termrock-navigation -p termrock-overlays --status-level fail` — 544 passed, 3 skipped.
- Follow-up focused theme/style suite: 11 passed, 0 failed.
- Follow-up package suites: `cargo nextest run -p termrock-runtime -p termrock-navigation -p termrock-overlays --status-level fail` — 549 passed, 3 skipped.
- `cargo nextest run -p termrock-theme --status-level fail` on the follow-up
  reports 8 failures (664 passed, 8 failed, 3 skipped). The same eight failed
  on pre-follow-up commit `94a1215e27e67bfad76f9edb8ca5b00fa9810c80`
  (114 passed, 8 failed, 0 skipped): `all_73_junie_slots_match_reference_capability_output`,
  six `*_empty_inherits_owner` checks, `nested_public_layers_dim_the_composited_origin_and_preserve_unwritten_cells`,
  and `standalone_empty_cells_match_pre_inheritance_reference`. They are
  confirmed pre-existing at that source commit and remain unresolved here.
- Clippy: `cargo clippy -p termrock-theme -p termrock-runtime -p termrock-navigation -p termrock-overlays --all-targets -- -D warnings` — passed after the final ScrollRegion metadata edit.
- Formatting: `rustfmt --check --edition 2024` on the nine changed Rust files — passed. Workspace `cargo fmt --all --check` remains blocked by existing formatting differences in untouched `crates/showcase-demos/src/pages/chrome.rs`; that file was left unchanged.
- `git diff --check` — passed.
- Independent code review: pending exact-commit review.
- Full application parity and library acceptance: pending; this audit is not an acceptance claim.

## Post-rebase lint repair

The source branch advanced to `a5766d24783c2b268e727497fe6861b6479c71cc`
while this worker series was rebased. Strict clippy then reported three
redundant `u16` conversions in `crates/termrock-overlays/src/help.rs` at the
scroll label width, first column key width, and help key width. These were
lint-only changes: `termrock_text::width` already returns `u16`. Removed the
identity conversions without changing layout values. The strict package clippy
gate was rerun after this repair and passed. The three source lines were the
only extension beyond the original shared-style file ownership.

## Reapplication onto current parity source

This integration starts from remote `parity/termrock-visual-baseline` commit
`1d7b8eb291e37cb4806ce1369a8a0abab08a81ac`. It reapplies only the reviewed
shared-style chain, preserving the intervening TablePro fixes:

| Original commit | Integrated commit | Change |
|---|---|---|
| `f3dc9c506c1f5f3aa41c1985c468546d60452e07` | `ea4a92429` | Preserve explicit component style channels. |
| `708420d470cc333867efaf0738faf21a3cd15a79` | `3ba054515` | Preserve theme/overlay/instance precedence. |
| `4a0ab4983b84b4909d0182a323eb6e1714651691` | `600113898` | Remove redundant Help width conversions. |
| `e72dbb15369d541ee86095f6caaee3f7e4680828` | `35aa04f24` | Apply explicit style channels to Dialog backdrop chrome too. |

The second cherry-pick conflicted in `support.rs` because the current parity
source had extended backdrop chrome from Picker to Dialog. The final resolution
uses one explicit-channel path for both Picker and Dialog, preserving the
current Dialog backdrop behavior. `ui/paint.rs` retains the current borrowed
backdrop projection and focus-bar foreground suppression. No app, CLI, baseline,
test-tool, or CI files were changed by this series.

Exact four-package comparison:

- Parent `1d7b8eb`: `cargo nextest run --locked -p termrock-runtime -p
  termrock-navigation -p termrock-overlays -p termrock-theme` — 662 run, 646
  passed, 16 failed, 3 skipped.
- Integrated candidate `35aa04f24`: the same command — 675 run, 661 passed,
  14 failed, 3 skipped. The candidate adds 13 passing regression tests and
  fixes the two parent failures `steps::tests::every_override_channel_is_forwarded_to_scrollbar`
  and `warmed_modal_snapshot_projection_performs_zero_allocations`.
- The same 14 failures remain on both revisions: six Dialog layout/size tests
  and eight Theme palette, empty-style inheritance, or nested-backdrop tests.
  They are recorded as unresolved, not passed or suppressed.
- Strict package clippy (`cargo clippy --locked -p termrock-runtime -p
  termrock-navigation -p termrock-overlays -p termrock-theme --all-targets --
  -D warnings`) passed. `rustfmt --edition 2024 --check` on every changed Rust
  file and `git diff --check` passed.
- Full application parity and library acceptance remain incomplete; this
  focused integration is not a completion claim.
## Integration onto current PR #38 source

Applied the independently reviewed shared-style commits, in order, onto
`09df933dc31f8c92528e57ae6810c0377c83cfb6`:

- `f3dc9c506` — explicit component style channels.
- `708420d47` — theme and overlay precedence.
- `4a0ab4983` — redundant Help width conversions.

The cherry-picks produced `1a44bd04a`, `8e9a7f59e`, and `941061f45`. Resolved
the `support.rs` collision by retaining the current PR source's `DIALOG`
backdrop path and applying the new explicit-channel path only to `PICKER`.
The `ui/paint.rs` merge retains current `clear_bar` foreground suppression and
the borrowed, allocation-free backdrop path. No later PR source changes were
overwritten; each incoming patch was scoped to the owned files.

Validation on merged source `941061f45`:

- `cargo nextest run -p termrock-runtime -p termrock-navigation -p termrock-overlays -p termrock-theme` — 658 passed, 14 failed, 3 skipped. All 14 failures reproduce on the exact parent `09df933`: the same six Dialog tests and eight Theme tests. The parent has two additional failures (`steps::tests::every_override_channel_is_forwarded_to_scrollbar` and `warmed_modal_snapshot_projection_performs_zero_allocations`) fixed by this change. The merged source adds 10 focused tests; all pass.
- The same package command on exact parent `09df933` — 646 passed, 16 failed, 3 skipped. Eight Theme failures: `all_73_junie_slots_match_reference_capability_output`, `filter_empty_inherits_owner`, `select_empty_inherits_owner`, `list_empty_inherits_owner`, `grid_empty_inherits_owner`, `tree_empty_inherits_owner`, `nested_public_layers_dim_the_composited_origin_and_preserve_unwritten_cells`, and `standalone_empty_cells_match_pre_inheritance_reference`. Six Dialog failures are unchanged at the merged source.
- The parent and merged worktrees used separate `CARGO_TARGET_DIR` values for the final repeat runs, preventing Cargo artifacts from crossing source revisions.
- Strict package clippy, `cargo clippy -p termrock-runtime -p termrock-navigation -p termrock-overlays -p termrock-theme --all-targets -- -D warnings` — passed.
- `rustfmt --edition 2024 --check` on all 10 changed Rust files and `git diff --check` — passed.
- Workspace `cargo fmt --all --check` — blocked by existing formatting differences in untouched `crates/showcase-demos/src/pages/chrome.rs`.
- Independent review of the exact integrated collision resolution remains pending. This increment does not establish application parity or full library acceptance.

## Dialog container channel follow-up

Independent external-consumer review of `e61c16a06d28e6f68e37055923b605de6cd44f16`
found that the preserved Dialog backdrop branch still replaced explicit
container foreground with sampled backdrop ink and cleared container
modifiers. The external `tuiscotti` consumer probes at
`/tmp/shared-style-external-e61/src/lib.rs` reproduced both defects: 1/3 pass,
with the theme and instance Dialog-channel cases failing. The frozen-background
and focus suppression cases passed.

Changed `overlay_chrome` to resolve Picker and Dialog backdrop containers
through one `ResolvedStyle` path. Both retain sampled backdrop ink for inherited
foreground, card background and neutralized underlay modifiers; explicit
theme, overlay, and instance channels then apply with shared precedence. Added
unit coverage for instance fg/bg/underline/modifier channels, theme-overlay-
instance precedence including `Inherit`, and `Clear` resets. No cursor, focus,
or application-specific branch was added.

Post-fix evidence:

- Runtime support tests `dialog_chrome_*` — 3 passed, 0 failed. Coverage checks
  instance fg/bg/underline/modifiers; theme, overlay, and instance precedence;
  and explicit clear behavior. On `e61c16a06`, the external actual-Dialog probes
  failed for instance and theme channels before this fix.
- External consumer rebuilt against this source — 4 passed, 0 skipped: actual
  Dialog instance channels, actual Dialog theme channels, actual Picker
  channels, and unchanged modal focus-color suppression with ordinary content
  retained.
- Combined focused runtime set, including Picker and
  `warmed_modal_snapshot_projection_performs_zero_allocations` — 7 passed,
  0 failed; modal projection remains allocation-free.
- Strict `cargo clippy -p termrock-runtime --all-targets -- -D warnings`,
  `rustfmt --edition 2024 --check crates/termrock-runtime/src/support.rs`, and
  `git diff --check` — passed.
- PR #38 source integration and full application parity remain pending.
