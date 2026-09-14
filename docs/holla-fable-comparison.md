# visual-baseline vs holla-fable comparison ledger

Branch refs (verified 2026-09-14):

| Ref | SHA |
| --- | --- |
| `visual-baseline` | `772b7a225baf56ab130c857cb7a49dddc7bae34a` |
| `holla-fable` | `3dbc73e253a1c4aebbe9cccbfc6d5c5e3ff8d73e` |
| merge-base | `3dbc73e2` (= `holla-fable`; `visual-baseline` is 38 commits ahead) |

Relationship: **not a merge task**. `holla-fable` is the historical reference; `visual-baseline` already contains every commit from `holla-fable` plus additive work. Comparison is semantic regression audit + selective restoration.

## Summary

| Layer | Verdict |
| --- | --- |
| `src/lib.rs`, `src/core/`, `src/ui/`, `src/widgets/` | **Identical** between branches |
| Application binaries | **visual-baseline strictly better** on integration (scroll, determinism, responsive UX, coverage tests) except **CLI help richness** (ported in this pass) |
| Test/snapshot infrastructure | **visual-baseline strictly better** (tuisnap harness, grouped snapshots, `app_tests_coverage`, nextest) |
| Documentation | **Reorganized on visual-baseline**; knowledge preserved under `docs/`; capture workflow refs updated to tuisnap paths |
| Legacy `shots/` corpus | **Partially superseded** by `snapshots/`; some frozen frames documented in `docs/baseline/tuisnap-coverage.md` |

## Decision ledger

| Area | holla-fable | visual-baseline before | Decision | Final state | Evidence |
| --- | --- | --- | --- | --- | --- |
| Component library | Baseline widgets/runtime | Same code | **HOLLA_BETTER (tie)** | Unchanged | `git diff holla-fable..visual-baseline -- src/lib.rs src/widgets src/core` empty |
| Holla scrollbar mouse | Render-only scrollbars | Full press/drag/click routing | **HOLLA_BETTER** | Kept | `src/bin/holla/screens/mod.rs` `scroll_press`/`scroll_drag`; modal routing in `app.rs` |
| Holla keyboard scroll sync | `ensure_visible` in render | `ensure_visible` on nav keys | **HOLLA_BETTER** | Kept | `disk.rs`, `files.rs`, `plan.rs` diffs |
| Showcase busy states | Wall-clock `Instant` | Tick counters + `Motion::Paused` fast-forward | **HOLLA_BETTER** | Kept | `pages/buttons.rs`, `pages/forms.rs`, `app.rs` |
| Showcase responsive dialogs | No keyboard fallback | `'d'` opens destructive dialog when actions clipped | **HOLLA_BETTER** | Kept | `pages/dialogs.rs` |
| Jackin capsule focus | No-op `focus_pane_by_id` stub | Real focus on pane open | **HOLLA_BETTER** | Kept | `screens/capsule.rs` |
| Jackin editor paste | Paste without sync | `sync_pending()` after paste | **HOLLA_BETTER** | Kept | `screens/editor.rs` |
| TablePro password field | Plain text input | `.masked()` + responsive E/D shortcuts | **HOLLA_BETTER** | Kept | `connections.rs` |
| CLI parsing | Manual argv loops | `clap` derive + validation | **HOLLA_BETTER** | Kept | All four `main.rs` |
| CLI `--help` content | Rich scenario/key reference | Auto-generated clap help only | **HOLLA_FABLE_BETTER** | **Ported** | Restored via `after_help()` on all four binaries; `Scenario::CONCEPT`/`PARITY` public again |
| `Scenario::CONCEPT` export | Public const | Test-only `PARITY` | **HOLLA_FABLE_BETTER** | **Ported** | `src/bin/holla/scenario.rs` |
| `examples/cells.rs` | Grapheme JSON reference | Removed | **OBSOLETE** | Intentionally removed | Superseded by tuisnap; no restore |
| `shots/` visual baseline | 4,666 PNG files + bash harness | `snapshots/` + Rust `tests/visual_baseline/` | **HOLLA_BETTER** | Kept | `store_integrity` passes; 303 PTY captures |
| App behavioral coverage | None | 68+ `app_tests_coverage` tests | **HOLLA_BETTER** | Kept | `docs/baseline/app-test-coverage-report.md` |
| Docs organization | Root-level scatter | Structured `docs/` tree | **HOLLA_BETTER** | Kept | `docs/traceability.md` maps retired paths |
| HP01–HP23 product parity | Documented gaps | Same simulation scope | **NEEDS_EMPIRICAL (unchanged)** | Deferred product work | `docs/parity/holla-parity-matrix.md` — not in scope of branch tip diff |
| Frozen legacy frames (Alt+Enter, wall-clock HP flows, TablePro Advanced tab) | Present in `shots/` | Partially migrated | **COMPLEMENTARY** | Documented frozen evidence | `docs/baseline/tuisnap-coverage.md` supersession table |
| CI workflow | `cargo test` on `holla-fable` push | Same + `snapshots/` inventory | **COMPLEMENTARY** | Kept | `.github/workflows/holla-fable-audit.yml` |

## 38-commit themes (holla-fable..visual-baseline)

1. Visual snapshot batches (holla, jackin, showcase, tablepro)
2. tuisnap baseline harness (`tests/visual_baseline/`)
3. Docs consolidation into `docs/`
4. `app_tests_coverage` modules (+3,333 LOC)
5. clap CLI migration
6. Responsive/scrolling baseline fixes
7. Legacy bash capture tooling removal

## Verification (final pass)

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| `cargo check --all-targets --all-features` | pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | pass |
| `cargo nextest run` | 545 passed, 303 skipped (ignored PTY captures) |
| `store_integrity` | pass |
| All four binaries build | pass |

## Residual holla-fable advantages

After this pass, **no material implementation advantage remains in holla-fable** at the branch-tip diff layer:

- Shared library: identical
- App integration: visual-baseline equal or better
- CLI help: restored on visual-baseline
- Visual baseline: visual-baseline superset with documented frozen legacy frames

Outstanding **product parity** gaps (HP01–HP23 vs legacy real Holla) are documented in parity matrices and are **future product work**, not regressions introduced between `holla-fable` and `visual-baseline`.
