# Main vs Holla: Refactor Breakage Analysis and Verification-First Recovery

**Date:** 2026-09-06
**Branches:** `holla` @ `4a46bca` (working) · `main` @ `c12cad8` (broken) · merge base `cc14dd6`
**Method:** 8 parallel read-only analysis agents over both branches, full document histories, build/test runs in detached worktrees (`/tmp/tc-main`, `/tmp/tc-main-pre`).

---

## 1. Executive summary

The `main` branch refactor pursued a sound target architecture but failed operationally: the visual oracle was deleted before a replacement harness existed, gates were non-binding or vacuous, and the tree has not compiled for its last 12 commits. The `holla` branch is behaviorally correct (269 tests green) but retains the monolithic structure the refactor was meant to fix.

Recovery strategy: **keep holla's behavior as the oracle, port main's proven mechanisms incrementally, and install a mutation-proven gate suite before any structural move.** Do not merge main's code; use it as reference text. Final merge to main replaces the tree (holla wins), preserving only main's frozen baseline archive, parity concepts, and document history.

---

## 2. How main broke — root causes (all verified)

### 2.1 Main HEAD does not compile

`8831a62` "feat(tui): expose dialog validation errors" rewrote `Dialog::draw` to delegate chrome painting to `overlay_chrome`, imported at `crates/tui/src/components/dialog.rs:12` and called at `:744`. **The helper was never committed — it exists in no commit on any branch** (`git log --all -S "fn overlay_chrome"` is empty). Every commit since (`8831a62..c12cad8`, 12 commits) fails `cargo check` with E0432. The entire parity/capture layer (`e51e29b`…`c12cad8`) was developed against a tree where `junie-tui` never compiled; its xtask-side tests pass standalone because xtask does not depend on junie-tui — a local "tests pass" illusion.

Control: worktree at `8831a62^` (= `25cf4e8`) builds clean in 11.68s.

### 2.2 Even the compiling ancestor is red

`cargo test --workspace` at `8831a62^`: **31 suites ok, 8 failed, 15 failing tests:**

| Suite | Failure | Cause |
|---|---|---|
| jackin `visual.rs` | `jackin_visual_baseline` | Baseline file in new digest schema; test writes old schema. **Can never pass.** |
| tui `architecture.rs` | `no_deprecated_or_legacy_api_usage`, `no_generic_component_copies_in_applications` | Apps bypass facade with raw `Style::add_modifier` (tablepro `app.rs:2103`, jackin `app.rs:218-222`) |
| tui `render_components.rs` | 6 grid digest failures | Grid rendering drifted from blessed digests, never re-blessed |
| showcase `app_tests.rs` | `complete_navigation_visits_every_page_and_every_state` | Panic at `app_tests.rs:811` — Pickers navigation state machine broken |
| tablepro `app_tests.rs` | `visual_surface_fixture_materializes_the_real_route` | Workbench fixture never reaches its named renderer |
| tablepro `perf.rs` | `frame_tablepro_grid_500x12_120x40` | Grid does not render its "Results" region at all (functional, not timing) |
| tablepro `visual.rs` | `tablepro_visual_baseline` | Digest drift `connections c1fdb4fc→4eee1e04` |
| xtask (at `25cf4e8`) | 3 capture tests | Ready-marker contract; fixed later, but at HEAD `parity::tests::frozen_manifest_and_mapping_cover_all_recipes` fails |

### 2.3 The oracle was deleted before the harness existed

`REFACTORING_GOAL.md` §3 made pre-refactor rendered output the regression evidence (authority rank 3), and §27/WP-0 made baseline capture a blocking prerequisite. Yet `7784719` (09-05) deleted the legacy root renderer and widgets, and the apps were rewritten (`4e07ea1`, `5042a40`, `444a8f4`) **before** the executable comparator existed (`e51e29b`, 09-06). Digest-only `Scene::assert_against` self-baselines stood in as oracle — the refactor blessed its own output. Main's own `GOAL.md` (rewritten 09-05 in `efada04`) now states this root cause verbatim.

### 2.4 Parity contract structurally unsatisfiable

`xtask/src/parity.rs:25` requires 5 artifacts per recipe (`txt, ansi, cursor, html, png`); `.gitignore:2-3` excludes `baseline/**/*.png` and `*.html` — those artifacts were never committed (`git log --all -- "baseline/before/*.png"` empty). `cargo run -p xtask -- parity --dry-run` fails immediately on any clean clone: `frozen artifact inventory differs`. No `parity/evidence.tsv`, `visual_review.tsv`, or `replays/` exist — nothing was ever replayed or approved.

### 2.5 Process failures

- **Red CI was non-binding.** CI failed on every recent push; merges continued. No branch protection.
- **pipefail-by-comment.** The blocking perf job piped cargo through `tee` under `bash -e` *without* `pipefail`; every step "succeeded" with E0432 inside. The workflow comment asserted the opposite of the actual shell semantics.
- **Decorative gates.** The architecture doc itself records ≥9 vacuous gates (§37.1, §43.1, §44.4…). `xtask doc-check` certified sections it never scanned (fixed upper section bound left §27–§72 invisible). `app_baselines_exist` checked file existence while the reader test could never pass. Root `tests/perf.rs` is dead (virtual workspace never compiles it).
- **Revert/restore thrash.** 28 commits: secret handling reverted/restored 3+ times; Props contract promoted/reverted/re-added/restabilized; "restore historical …" fixes across showcase, tablepro, jackin, runtime repaint cadence.
- **Goal drift.** Product goal → 1683-line architectural rewrite goal (§2: "no parallel legacy API", "backward compatibility not required") → agent-routing overlay → **180° reversal** in `efada04`: "restore the historical UI exactly." The architecture doc's own §46 proves the prescribed slice plan non-executable (scripted rename produces an unresolvable workspace; no compiling intermediate state).
- **Spec-caused defect.** Architecture §11.4's prescribed mono fallback made disabled text black-on-black ("a goal violation caused by the specification, not the implementation" — Adjudication P).

---

## 3. Document history findings

### REFACTORING_GOAL.md (5 revisions, 09-03 → 09-05)

1. `e48137f` — original: 30-section goal, 8 vertical slices, shadcn/ui principles, "no parallel legacy API", Junie default + one non-Junie theme required.
2. `dce91d5` — removed dead doc references from authority order.
3. `2d81eec` — added §0 model-routing mandate (Fable coordinators, Opus analysts, substitution = blocker). Later violated: Fable credits exhausted (HTTP 429), user-authorized deviation to Opus-5 recorded in state doc.
4. `25ea92b` — softened mandatory external research to analyst's choice.
5. `efada04` — demoted the entire doc to "historical architecture contract"; active goal became `GOAL.md` (parity restoration).

### COMPONENT_ARCHITECTURE.md (60 commits, 0 → 8,660 lines)

Phase 1: authored contract (§1–20: component model, identity, theme, testing, slice plan). Phase 2: adjudication ledger — every review appended numbered sections (§21–§73) striking earlier claims inline; includes §46 "Slice 5 cannot run as the appendix specifies" and §47 deferring the rename. Phase 3: capture hardening (added, twice reverted as unsafe, re-landed). Phase 4: doc corrected *to match code* — normative direction reversed. Net: design contract → self-contradicting ledger → descriptive mirror.

### REFACTORING_STATE.md (79 commits, append-only)

Last checkpoints: gates pass, conformance/lib tests green (934/735), **but** capture provenance stale, last independent visual audit **FAILING** (TablePro digest drift, Jackin 15 stale journey failures), "visual blessing remains unauthorized." Contains a coordinator correction that a reported "fully green" tree went red one commit later.

---

## 4. Divergence map

| | `main` | `holla` |
|---|---|---|
| Commits past base | 441 | 10 |
| Files changed | 2,603 (+237K/−68K) | 1,010 (+32K/−5K) |
| Structure | workspace: `crates/tui`, `crates/tui-testing`, `apps/{showcase,tablepro,jackin-preview}`, `xtask`, `parity/`, `baseline/` | monolith: `src/` lib + `src/bin/{showcase,tablepro,jackin_preview,holla}` |

Path mapping (base → main): `src/widgets/*` → `crates/tui/src/components/*` (semantic renames: `chips→chip`, `statusbar→status`, `splitter→split`, `table` absorbed into `grid`, `scrollbar` absorbed into `scroll_region`); `src/bin/*` → `apps/*`; new: `tui-testing`, `form/help/wizard/meter/nav_list/filter_list/picker_chain/too_small/scroll_region` components.

**Holla-only work (zero cherry-picks onto main, patch-id verified):**

| Commit | Content | Merge action |
|---|---|---|
| `0f8e8fc` | holla app (~6,300 lines: domain/sim/screens, 11 scenarios) | port as `apps/holla` |
| `58fea3e` | warning glyph fix: `▲` in keyhint.rs — **main still lacks this** | reapply to `components/keyhint.rs` |
| `5512ad6`,`0f974bf` | p6_matrix capture script + 132-PNG matrix | adapt paths |
| `94a3830`,`49961e0`,`11f0599`,`4a46bca` | DESIGN.md sections, holla-project docs | keep |
| `93d840f`,`26ed5d8` | agent config | trivial |

---

## 5. Holla branch state (the oracle)

**269 tests green** (8.5s): lib units + showcase 41 (+visual digest) + tablepro 38 + jackin 34+8 + holla 59.

**Pinned invariants:** byte-identical determinism (scenario+motion+frame+size ⇒ same picture); `▲`/`!` status glyphs at every color level; content-driven dialog width (`Dialog.width` pub); tonal activity strip (5 states, tone+glyph); two-gate destructive flows with live dependent recalc; ranking semantics (pin>alias+urgency; exact-query resurface); plans as DAGs (per-child primary branch, policy-skip non-excludable); simulation guarantee (no real process ever spawns).

**Reusability gaps (the legitimate reason for the refactor):** no uniform component signature convention; concrete Junie-only theme with no override mechanism; app shell (`Cx`/`Request`/`Go`/`Modal`) duplicated 3–4×; fixture clock/scenario machinery duplicated; test harness `H` duplicated; `grid.rs` 2,191-line mega-widget; table+grid duplication.

---

## 6. Holla's verification blind spots (audit findings)

All of these can change today with 269 tests green:

1. **All color/tone in tablepro/jackin/holla** — their tests assert buffer text only. Tonal strip, danger rows, `◆` production tone could swap silently.
2. **Theme downgrade paths** — mono has zero tests; Ansi16 one smoke test; 256 one accent assert; tablepro/jackin no downgrade coverage at all. **Live defect:** holla's `disabled` (WHITE_30) and `text_muted` (WHITE_50) both map to DarkGray at mono — indistinguishable.
3. **runtime.rs completely unpinned** — repaint cadence, tick scheduling, input coalescing. This is the exact class main regressed (`7020afc`).
4. **Focus-ring order** — derives from render-registration order; a refactor reordering rendering permutes Tab order with all tests green.
5. **Hover/pressed rendering** — digest is static; pressed flash unpinned everywhere.
6. **App layout positions** — `find()` self-locates; rows can move.
7. **All ~400 shots + 132-PNG matrix are eyeball-only** — nothing in cargo test reads them.

---

## 7. What main did well (port list, trust-verified)

Main's core mechanisms were green at the last compiling commit (lib 741, conformance 934, tui-testing 16). Only grid *visuals* and app-shell baselines were red.

**Component architecture (M1–M10):**
- M1 caller-owned `XState` + borrowed props + `update/draw` two-phase frame (render purity as compile error)
- M2 `Cx`/`Ui` split (update vs draw contexts)
- M3 `Response<A>` with typed actions + invalidate levels
- M4 theme tokens/recipes/StylePatch 6-level precedence — **highest port risk** (every resolved color moves)
- M5 capability downgrade incl. **mono state manifest: DISABLED → primary fg + DIM** (fixes black-on-black; test-pinned `mono_disabled_is_dim_and_readable`)
- M6 no Widget trait — main explicitly rejected trait objects; **validates holla's design**
- M7 collection vocabulary (`CollectionCore`/`KeySet`/`RowUi`) shared by list/tree/grid
- M8 runtime-owned layer stack replacing per-app Modal enums
- M9 `App` trait + `Runtime` + virtual clock (`clock_ms` advanced only by ticks)
- M10 tui-testing: `Harness` (id-addressed clicks, resolved-style assertions), `Scene`/`Baseline` (merge-under-lock, atomic rename, `BLESS=0` fails closed), conformance matrix (20 generated cases/component)

**Verification machinery:**
- Frozen immutable oracle + explicit recipe mapping, symlink/missing rejection (`parity.rs:543-627`)
- Provenance-bound evidence: sha256 tree fingerprint, dirty-flag binding (`parity.rs:1759-1811`)
- Fail-closed comparator with printable first-difference cell coords
- Bless-guard with fail-closed base ("HEAD fallback passes vacuously" — `main.rs:8569-8582`) + classified-change ledger
- Boundary-check registry (~40 named rules) + `every_named_test_exists` anti-blind-spot check
- Open-ended doc-check ranges (fixed bounds are silent expiry dates)
- Counting-allocator perf harness

**Failure-mode prevention rules (each kills a verified main failure):**
1. Green gates must be binding (pre-commit hook; `git rebase --exec`); "compiles" is gate zero.
2. Never trust shell semantics asserted in prose — explicit `set -o pipefail`, assert on artifacts.
3. One baseline file, one hasher, one regen var; boundary grep forbids ad-hoc baseline writers.
4. Gates assert the property the failure violates, never file existence.
5. A blocking gate lands green in the same change, or advisory-first.
6. Every self-referential baseline paired with one externally-generated oracle; blesses loud.
7. Open-ended ranges over append-only docs.
8. `.gitignore` cross-checked against required-artifact constants in doctor gate.

---

## 8. Recovery plan

### Phase 0 — Goal correction (docs only)

Write corrected `GOAL.md` on holla: refactor for reusable component API with **zero behavior change**. Authority order: holla's green tests + visual digests + shots (the working oracle) > architecture docs. Holla's output is the truth; main's output is wrong-by-evidence. Non-goals: big-bang workspace cutover, deleting working code before its replacement is proven, parallel legacy API as end-state.

### Phase 1 — Oracle hardening (before any structural move)

Cheapest-first; each item independently landable:

1. `tools/gate.sh` + `.githooks/pre-commit` (`git config core.hooksPath .githooks`) + `doctor` mode (hooksPath set, regen var unset, `git check-ignore` clean over shots/tests/tools, test-count floor ≥ 269).
2. Fail-closed showcase baseline: panic on verify-with-var-set; schema header + entry count.
3. Extract `src/core/digest.rs` (FNV cell loop from showcase `app_tests.rs:635-653`); unify to `tests/visual_baseline.txt` with scoped lines + exact set-equality per scope.
4. Per-bin visual digests: holla (11 scenarios × sizes × 3 color levels — the p6 matrix as text hashes), jackin, tablepro. Kills blind spots 1, 2 (partial), 6.
5. Showcase digest at all 4 color levels (Ansi256/Ansi16/Mono) — pins every downgrade mapping as-rendered.
6. Theme downgrade unit table (exact mapped color per token per level) + adopt main's mono DISABLED→DIM rule (fixes holla's disabled/muted mono collision).
7. Runtime cadence as pure function — extract tick/poll/coalescing decisions from `runtime.rs`, pin them. Only item touching src/ in this phase.
8. Full-ring Tab tests per bin; chrome row-position asserts.
9. Unit tests for the 15 untested widgets (template: `brand.rs:97`); `diff` showcase page.
10. API surface snapshot (`tests/api_surface.txt`, ~606 pub items, anti-vacuity floors).
11. Interaction replay scripts as checked-in data (`tests/replay/*.txt`) — oracles survive test-code deletion.
12. `tests/shots_manifest.rs` + `shots/MANIFEST.txt` — shots become asserted.
13. `tools/mutation-check.sh` — applies the mutation table (flip `▲` at keyhint.rs:76; pin weight 150→140; `Dialog.width` private; Enter/Esc arm swap…), requires the mapped gate to go red, reverts. **Go/no-go for Phase 2.**

### Phase 2 — Incremental refactor (compiling + green at every commit)

Trust-ordered adoption of main's mechanisms; behavior from holla, structure from main:

1. tui-testing extraction: `Scene`/`Baseline` machinery (keep holla's sidebar-mask as exclusion rects), `Harness` with id-addressed clicks. Pure addition.
2. Virtual clock into runtime; dedupe holla/jackin clock+scenario machinery (time source = library service; scenario content stays app-side).
3. Mono DIM rules into theme (with Phase-1 digests guarding).
4. Collection vocabulary (cursor/selection/scroll reconciliation shared by list/tree/grid).
5. Per-widget M1 ports (state/props split), one widget family per slice; `Response<A>` at boundaries.
6. App shell extraction: single `App` trait + `Runtime` + layer stack; dedupe the 3–4 `Cx`/`Request`/`Go`/`Modal` copies.
7. Theme recipes/precedence pipeline — last, highest risk, only after color digests pin every level.
8. Crate cutover (`crates/tui`, `crates/tui-testing`, `apps/*`) only after the facade is stable — the sequencing main skipped.

### Phase 3 — Merge to main

Main is not mergeable conventionally (441 commits, broken tip). When holla is complete and green: merge with holla's tree winning; preserve from main only `baseline/before/` + `parity/recipes.tsv` + parity tooling concepts + docs (archived under `docs/refactor-attempt-1/`). Reapply-forward holla-only work is identity (it *is* holla). Cheap main triage if main must stay alive meanwhile: define `overlay_chrome` or revert the dialog call site; unify baseline schema; fix `.gitignore` vs artifact contradiction.

### Validation contract (every step)

`cargo test` green (269+ growing); visual digest diff empty or in a commit naming the intentional change; p6 matrix unchanged; no gate lands without a proven mutation failure; a replacement oracle goes green in slice N before the old one is deleted in slice N+1.

---

## 9. Companion document

`HOLLA_REFACTOR_RECOVERY_PLAN.md` (external analysis, repo root) — independently derived, converges on the same strategy (preserve working apps, extract incrementally, adopt main's APIs selectively with proof obligations). Its novel claims were verified by subagents; results below.

## 10. External plan verification results (2026-09-06, 4 agents)

**Every load-bearing claim checked: CONFIRMED.**

- **F-01 compile blocker** — confirmed locally (dialog.rs:12,744; no `overlay_chrome` module in any commit). Caveat: the plan says "recover the intended helper" — there is nothing to recover, it was never committed; honest options are reimplement-or-remove.
- **F-02 sidebar hitbox overlap** — confirmed with exact math, and **worse than claimed**. Main paints the sidebar twice: `NavList::draw` registers hitboxes with section headings/gaps (`apps/showcase/src/app.rs:1228`), then `paint_sidebar` overpaints compact without registering (`:1238`). At 80×24 every visible row is shifted up 3: visible **Buttons** clicks `overview`; visible **Overview** has no hitbox at all; hover highlight lands on the correct row while the clickable rect is 3 rows away — looks right, clicks wrong. Holla is immune by construction (one loop paints and registers the same `Rect`, `src/bin/showcase/app.rs:894-918`). Fix: one layout drives both; move compact policy into `NavList`.
- **F-03 timing regression** — confirmed on all sub-claims. Holla: 2,200 ms wall-clock deadline, completion only on Tick (`src/bin/showcase/pages/buttons.rs:53,190-194`). Main: `busy_frames = 28` decremented unconditionally per `update()` call — any key/mouse/resize advances completion, one event can decrement multiple times via settle passes; the virtual clock does not mitigate (counter never consults it). Main also dropped app-level status publication entirely. Fix: gate decrement on `UpdateCause::Tick`, or deadline in `clock_ms` exposed via `Cx`.
- **F-05 jackin CLI regression** — confirmed on all 9 sub-claims (FirstUse→Returning default, aliases dropped, help dropped, invalid values silently ignored, `JACKIN_NO_MOTION` presence-vs-value flip). Same drift class in showcase (worse) and tablepro (mixed — main is stricter on unknown flags, holla friendlier on help). Neither side has argv-parsing tests; main's tablepro `parse_args<I>(args: I)` is the right shape to standardize on. Holla app exists only on holla, as expected.
- **F-04 inspector reduction, F-06 four binaries, F-07 parity machinery unsatisfiable, F-08 self-baselined default-state-only visual tests, F-09 guard-appeasing shell props** — all confirmed.

**External plan gaps (things our analysis has that it lacks):** the 15-failure/8-suite inventory at `8831a62^`; the `.gitignore` png/html unsatisfiability mechanism; the BLESS vs UPDATE_BASELINE schema fork; holla's unpinned `runtime.rs`; the exact breaking-commit span (`8831a62`, 12 commits).

**External plan strengths (adopt into our design):** evidence-classification taxonomy (observed/verified-source/historical/hypothesis/proposed); gate-integrity mutations beyond ours (candidate-as-baseline substitution, cross-commit artifact reuse, zero-test filter, tee-exit-code, half-written artifacts); anti-circularity rules (expected coordinates never read from candidate hitmap); full-input fingerprint manifests (fixtures, assets, toolchain, locale, font); 7-step frame lifecycle with atomic commit; `main-salvage.tsv` disposition ledger ("unexplained difference blocks integration"); merge-rehearsal protocol (full gates rerun on the exact committed merge tree from a clean worktree, no `-s ours`, evidence regenerated post-merge); `implemented ≠ verified ≠ integrated` status machine; CI aggregator treating skipped/neutral/zero-case as failure, plus a deliberately-failing PR to prove checks block.

**Combined verdict:** both analyses agree on strategy. The external plan's verification design is stronger on gate integrity and merge rehearsal; this analysis is stronger on main's actual failure inventory, holla's blind spots, and port-risk ranking. Use the external plan's task structure (H/V/A/R/APP/PKG/FINAL) as the execution skeleton, amended with: the `8831a62^` failure inventory as acceptance targets, the `.gitignore`/schema-fork fixes, `runtime.rs` cadence pinning, and reimplement-or-remove (not "recover") for `overlay_chrome`.
