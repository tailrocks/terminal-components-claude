# Review findings: hit registry, capture, table grab (PR #38 workstream)

Date: 2026-10-21. Findings from the independent review of
`design/interaction-polish` at pinned commit `f4867e9b5` (review refs
`a6a0dd37d` / `50c8d9b22`), applied to `parity/termrock-visual-baseline`.
Ownership: `crates/termrock-runtime/src/hit.rs`,
`crates/termrock-runtime/src/capture.rs`,
`crates/termrock-grid/src/table.rs` and their tests. Frozen four-app output
is untouched: none of these APIs had external callers besides the ported
tests, so no app-visible behaviour changed.

## 1. MAJOR — `register_thumb` dropped PR #26's id/owner split

Finding: the port discarded `owner` (`let _ = owner;`) and registered the
thumb under the bar id, so thumb hits resolved to `owner = bar_id` instead
of the scrolled widget. PR #26 (`src/core/hit.rs` ~:308,
`register_part_as(bar_id, owner, THUMB, thumb)`) resolves thumb hits to the
owner while keeping the bar id as the region's control id.

Fix: `Region` gained the PR #26 `id` field (`Region::id`), and
`push_kind` takes `(id, owner)`; every existing registration passes
`id = owner`, so only the split registrations differ. `register_thumb` now
registers `(id: bar_id, owner)` with part `THUMB`: hits resolve to the
scrolled widget, the region keeps the bar's id.

Tests (`crates/termrock-runtime/src/hit.rs`):
`a_bar_resolves_to_its_owners_track_and_the_thumb_is_its_own_part`
(corrected to PR #26's expectation — thumb hit resolves to
`(owner, THUMB)`, and the region keeps `(id: bar_id, owner)`).

Fail-before: reverting only `register_thumb` to the ported body makes the
test fail at "the thumb resolves to the bar's owner, its own part" (got
`owner = bar_id`).

## 2. MINOR — `push` writing scroll onto the previous region (no defect)

Finding: with an empty area, `push_kind` early-returns while `push`
unconditionally writes `scroll` onto `self.regions.last_mut()`, allegedly
corrupting the previous region.

Investigation: `push` already guards `if area.is_empty() { return; }`
before calling `push_kind`, and `push_kind`'s only early return is the
identical emptiness check — the corrupting path cannot trigger. Verified
identical at the review commits `a6a0dd37d` and `50c8d9b22` and at the
worktree base. Recorded as no-defect; the regression test locks the
behaviour in.

Test: `pushing_an_empty_area_never_touches_the_previous_regions_scroll`
(scrollable region, then empty-area scroll and part registrations; the
first region's owner and scroll state are unchanged). Cannot fail before a
fix because there is no defect to fix — the guard predates this workstream.

## 3. MINOR — `register_backdrop_of` discarded the backdrop id

Finding: `_id` was discarded while the doc claimed the id is queryable;
PR #26 stores `HitRegion::plain(id, area)`, enabling
`part_of(id)` / `has_owner(backdrop_id)`.

Fix: `register_backdrop_of` stores `id` as the region's id (via the new
`Region::id` field), and `has_owner` matches a region's owner or its
control id — a strict extension, since `id == owner` everywhere else.

Tests: `a_backdrop_catches_clicks_for_its_owner` (hit.rs) asserts
`has_owner` on both the backdrop id and the owner and that the region
keeps `(id: backdrop, owner)`; the ported
`a_secondary_outside_a_backdrop_reopens_the_layer_beneath` (capture.rs)
now passes the real backdrop id (`owner.part(Part::BACKDROP)`) and asserts
`has_owner(back)` after registration.

Fail-before: reverting `register_backdrop_of` / `has_owner` (keeping the
`id` field) fails both tests (`has_owner(back)` false; region id observed
as the owner).

## 4. MINOR — table thumb grab armed on failed capture

Finding: `table.rs` ignored the `cx.capture(bar_id, THUMB)` result but
armed the grab via `press_track`; release checks
`cx.capture_owner() == Some(bar_id)`, so a failed capture could leave a
grab the release path never clears. `scroll_region.rs:250` already branches
on the capture result.

Fix: the thumb-press arm follows the `scroll_region.rs` precedent — the
grab (`press_track`) is armed only when the capture succeeds.

Test (`crates/termrock-grid/src/table.rs`):
`a_failed_thumb_capture_does_not_reanchor_or_stick_the_grab` — driven
through the real runtime: first thumb press captures; a second press while
the capture is live (delivered to the bar with a failed capture) must not
jump the scroll or re-anchor the grab; the release clears the capture and
leaves the offset.

Fail-before: reverting the guard fails at "a failed capture must not jump
the scroll" (offset jumped from 2).

## 5. MINOR — secondary-release acceptance keyed by owner, not control id

Finding: `pointer_tests` accept a release on the owner where PR #26
accepts on the part id (`accepts_release` exact-Id).

Resolution: parity's `Hit` carries the owner and has no separate control
id in its hit type, so acceptance is keyed by `hit.owner` — the exact id a
widget passes to `Registry::accept_release`, not a coarser class.
Documented rather than re-keyed: docs on `Registry::accepts_release` and
`SecondaryTracker::on_secondary_up` state the adaptation, and the test is
renamed to `a_secondary_release_on_an_accepting_owner_is_a_click` with a
port note.

## Gate

`cargo nextest run -p termrock-runtime -p termrock-grid --all-targets`,
`cargo clippy --all-targets -p termrock-runtime -p termrock-grid`,
`cargo fmt --check` — green on the pushed commits. `cargo nextest run -p
termrock-conformance` was run on the fix HEAD and on the base `6abc95d37`:
both report the same 586 pre-existing failures and
`diff` of the sorted failure sets is empty, so the conformance failure set
is unchanged by these fixes.
