# Runtime underlay fill repair

Status: runtime implementation and bounded runtime gates pass on `wip/menu-underlay-fill-20261021`. Menu integration and all application acceptance gates remain incomplete.

## Superseded finding

The initial e4 repair sampled the base page during layer painting. Its underline-color fix was valid for a single overlay, but its claim that nested overlays already preserved the nearest lower-layer ink was incorrect. The original nested test ignored a failed `Runtime::handle(Tick)` and never opened its upper layer. Independent review corrected the fixture with `stub::deliver` plus assertions for upper-layer admission, area and black fill; the corrected fixture failed Red-versus-Blue on both the pre-repair revision and commit `860529a316dd1e45e6e2f9f3921d93e964fa0e66`. The prior nested-overlay PASS and the claim that layer fill could sample `self.page` are superseded. Preserve the old fixture and logs in the external review packet; do not cite them as proof.

The same independent review found that page fill could retain `ForcedWidth(4)` when replacing a wide symbol with a space. That defect also reproduced on both revisions.

## Current implementation

`LayerDraw` now carries a private per-cell style intent for underlay fills. The existing ascending layer compositor resolves that intent against the live destination after lower overlays and the layer backdrop have been applied. It inherits unmodified foreground, underline color and modifiers, plus foreground/background semantic roles. Tracked paints and explicit style patches accumulate channel and modifier operations; raw full-cell writes cancel the intent. `move_cell` transfers it with cell data and roles. Ordinary cells without intent keep the prior complete-cell copy path.

Page fill is immediate: it snapshots foreground/underline/modifiers, resets symbol and transport metadata, emits an ordinary space, and replaces the background. Layer fill emits a reset ordinary space and records only its explicit background before deferred composition. Both paths reset wide/skip transport metadata and respect the active clip. No public API, application, menu, theme, or input-dispatch code changed.

The invalid old unit expectation that a layer closure can observe inherited page ink was updated: it now asserts placeholder state before composition and inherited state after composing against the actual page.

## Evidence and checks

- Corrected public Runtime regressions: `cargo nextest run -p termrock-runtime --test underlay_stack` — 3 passed. They assert a real upper layer was admitted via `stub::deliver`, then check nearest-lower Blue ink despite reverse closure order, underline/modifier retention, upper black background, explicit post-fill glyph paint, raw-write intent cancellation, and page-fill width reset.
- Corrected internal compositor regressions — deferred inheritance/roles/`patch_style`: 1 passed; `move_cell_transfers_underlay_intent_with_the_moved_cell`: 1 passed.
- Before/after defect proof: copied the unchanged public `underlay_stack.rs` test into a detached worktree at `860529a316dd1e45e6e2f9f3921d93e964fa0e66`, then ran `CARGO_TARGET_DIR=/private/tmp/termrock-menu-underlay-prefx-target-20261021 cargo nextest run -p termrock-runtime --test underlay_stack`. It failed the admitted nested overlay (Red vs expected lower Blue) and ordinary-space metadata (`ForcedWidth(4)` vs `None`); raw-write cancellation passed. The same command on this WIP passes all 3 tests.
- `CARGO_TARGET_DIR=/private/tmp/termrock-menu-underlay-target-e4eae58e cargo nextest run -p termrock-runtime --all-targets` — 403 passed, 3 skipped (Nextest reports those existing ignored cases; no new test was skipped).
- `CARGO_TARGET_DIR=/private/tmp/termrock-menu-underlay-target-e4eae58e cargo clippy -p termrock-runtime --all-targets -- -D warnings` — passed.
- `cargo fmt --check` and `git diff --check` — passed on the final edited sources.

Remaining runtime evidence: independent review of the final commit and deeper backdrop/profile behavior. Full paired Tuiscotti application parity, menu unification, and all four app acceptance gates are still unverified and incomplete.
