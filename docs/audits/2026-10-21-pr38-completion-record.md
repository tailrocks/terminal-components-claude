# PR #38 completion record — 2026-10-21

This is the current running record for the Termrock completion work delivered
through PR #38 (`parity/termrock-visual-baseline`). It records fixed reference
identities, what was reviewed, what was changed, what was integrated or
excluded, the evidence, and what is still incomplete. Acceptance is not
claimed; the status table at the end is the authority.

## Fixed references

Recorded before any change and not modified during this work:

| Purpose | Revision |
| --- | --- |
| PR #26 product reference (`design/interaction-polish`) | `f4867e9b5f8a08a3bb61ce80d644198a29f51984` |
| Frozen baseline source | `4169783ae883788186d161af09ba9c481bd84e5c` |
| Frozen baseline tag object | `2718a6b8127a522a3d351d0b4bdb7542fc8da542` (peels to `4169783ae`) |
| Tuiscotti pin | `a1ea27f757d75d07b05499276cd932e9f8e067ad` (`Cargo.toml`, `termrock-e2e`, `visual-harness`, `showcase`) |

PR #38 tip recorded at the start of this session:
`7649ee7ed8e8645e49f859a171ea0193fccc1e82`. The branch advanced during the
work (`e9375eb4f`, `40d1d89b7`, …) because other workstreams are landing; the
review evidence below is pinned to `7649ee7ed`, and each landed change was
rebased and re-verified on the then-current tip before publication.

No reference revision was changed. No reference decision was required for the
work in this record.

## Root-cause fix: unstable `Theme::fingerprint`

`Theme::fingerprint` hashed the `Debug` representation. `Recipes` carries a
per-construction `revision` counter for cache invalidation, so two independent
`Theme::junie()` values printed differently and fingerprinted differently even
though `PartialEq` deliberately ignores `revision`.

Effect on the gate: every conformance check that compares a theme fingerprint
before and after a local style override failed order-dependently
(`local_override_does_not_mutate_the_theme` for every component that has one)
plus the runtime/overlay checks that compare against it. The `termrock-conformance`
package had 602 failing cases on the pinned tip.

Fix: `Recipes` has a manual `Debug` that omits `revision`; the revision stays
private bookkeeping and semantic equality is unchanged. Regression test
`fingerprint_ignores_per_construction_revision` asserts two `Theme::junie()`
values are equal and fingerprint equally.

Evidence: `cargo nextest run -p termrock-conformance` fell from 602 failures to
19 failures on the same source, with no expectation updated and no test
weakened. The remaining 19 are separate component-contract failures listed in
"Open verification".

## Picker contract gaps closed

The branch audit found PR #38 documents contracts that the code did not
implement. Two were implemented additively; the frozen defaults of existing
consumers do not change except where the reference geometry required it.

- `FilterPolicy::Substring`: case-insensitive contiguous matching, alongside
  the existing `Label` subsequence and `Caller` policies. Test:
  `substring_policy_requires_a_contiguous_match`.
- `Picker::max_rows(rows)`: explicit row cap with the existing theme
  `popup_max_rows` as the default. Zero is treated as one. Test:
  `max_rows_caps_the_requested_layer_height`.

Picker geometry from `parity/picker-geometry-20261020` (`28cc24880`,
`6b72d77a6`) was evaluated: seven chrome rows for a searchable card, height
clamping, and `GeometricCenter`. Adopting it changes every picker frame
(TablePro `quick-switcher 120 40` `b206bc55…` → `b90250e6…`). The source
contract is plausible but the change has not been proven against the frozen
tuiscotti corpus, and the current application digests were re-approved from a
candidate. It was therefore **not** adopted in this increment; the pending
decision needs a paired reference capture. No digest was blessed.

Source: `crates/termrock-navigation/src/filter_list.rs`,
`crates/termrock-overlays/src/picker.rs`,
`crates/termrock-overlays/tests/picker_projection.rs`.

## Branch consolidation ledger

Full tables: [parity branches](evidence/2026-10-21-pr38-completion/parity-branches-review.md),
[WIP branches](evidence/2026-10-21-pr38-completion/wip-branches-review.md),
[other branches](evidence/2026-10-21-pr38-completion/other-branches-review.md).

Reviewed 51 `parity/*`, 54 `wip/*` and 18 other remote branches against the
pinned tip using patch-id equivalence plus source verification.

- Already integrated (rebased, reworked, or literal ancestors): 47 `parity/*`,
  21 ancestor + 4 patch-equivalent + 9 reworked `wip/*`, and 12 other branches.
- Integrated later / still queued: picker geometry and the picker contract
  hunks above; `edit-menu-overlay-20261020` (overlay adapter still absent) and
  `fix/picker-focus-and-rows-20261020` (only the missing hunks; the branch is
  far behind and merging it wholesale would revert current APIs).
- Needs independent review, not merged: `buttons-reference-decision` pins,
  `codex/termrock-q48-source-only`, `codex/termrock-local-proposals`,
  `owned-fresh-plan`, `menu-cursor`, `menu-underlay`, `jackin-live-capsule`,
  `jackin-rate-limit`, `public-consumer-contracts`.
- Excluded: `design/interaction-polish` (read-only product reference),
  `velnor-design-upgrade` (pre-crate-split monolith), the seven pre-rewrite
  `src/` WIP branches, `velnor-actions-0135` (superseded by 0136), and
  documentation-only/obsolete task branches.
- Deletion candidates are listed in the other-branches review with the
  protection rule applied; nothing was deleted in this increment.

## False parity inventory

Full inventory with file:line and mechanism:
[false-parity inventory](evidence/2026-10-21-pr38-completion/false-parity-inventory.md).

Confirmed still present on the pinned tip:

- **Jackin**: 26 `historical_*` functions and the 22-call dispatch tail in
  `crates/jackin-preview-app/src/app.rs`; fixed cell tables `const ENTRIES` in
  `accounts_add_form_view.rs`, `usage_detail_view.rs`,
  `usage_overview_view.rs`; frame-identity and exact-size predicates; 5 forced
  cursor registrations.
- **Holla**: 19 `*_plate` predicates and 17 `now_ms() == 80` tag frames in
  `holla-ui/src/app.rs` and `holla-sim/src/catalogue.rs`; the earlier
  handoff claim that production has no `*_plate` dispatch is false — the
  predicates were renamed from plates to width-band tag frames.
- **Showcase**: fixed cell tables in `showcase-demos` `taskrunner.rs` and
  `terminal.rs` (about 1 900 runs) plus a tick-settle gate.
- **TablePro**: `paint_legacy_tabs`, explorer micro-repaints, draw-order
  `thread_local` channels, and the 120×40 query-focus hardcode.

Order of removal (from the inventory): TablePro pickers → Showcase button
repairs → Jackin cell tables → Holla presentation selectors → residual
repaint/forced-cursor paths. None of these bypasses may be renamed or moved
into Termrock; each is replaced by the owning component and only then deleted.

## PR #26 behavior gaps

Full table: [PR26 component gap](evidence/2026-10-21-pr38-completion/pr26-component-gap.md).

Highest-priority reusable gaps still absent from Termrock: refused dismissal
with feedback (`Dismiss::allows` has no caller), popover fit/slide policy,
table column seams and drag autofit, `Dismiss::pass_through` wiring, and drag
autoscroll. Items that must **not** be ported (frozen-capture conflicts or
superseded owner decisions): modal backdrop DIM, neutral inactive selection,
`ScrollState` context default 1, compact check glyph.

## Open verification

Known failures on the pinned tip after the fingerprint fix that are **not**
caused by this increment and are not accepted as green:

- `termrock-conformance`: 19 cases — `context_menu`/`grid` mono distinction,
  `dialog`/`filter_list`/`code_editor`/`help_overlay`/`picker` binding
  agreement, `filter_list`/`picker` hover restoration, and the `help_overlay`
  focus/traversal/tiny-rect/wheel group. Investigation started; root causes
  are not yet recorded.
- `termrock-navigation::item_row_columns`: two pre-existing failures
  (`constructor_defaults_match_the_pre_policy_full_cell_fixture`,
  `owner_part_styles_win_over_painter_defaults_in_columns_mode`).
- Application paired comparison (tuiscotti) was not run in this increment;
  no application parity claim is made.

Full workspace nextest, clippy, and formatting were run on this source before
publication; exact results and the pushed commit are recorded in the handoff
entry for the increment.

## Status

| Requirement | Status |
| --- | --- |
| Exact product match (Showcase, TablePro, Jackin, Holla) | Incomplete — false-parity inventory above; no paired comparison in this increment |
| Reusable Termrock library; thin applications | Incomplete — picker contracts and theme fix landed; Jackin/Holla/Showcase/TablePro bypasses remain |
| Branch consolidation into PR #38 | In progress — ledger complete; two bounded integrations identified, six branches held for review |
| Reference integrity | Frozen tag, PR #26 and Tuiscotti pin unchanged |
