# Refactoring State

## Status

- Overall: Slice 2 — COMPONENT_ARCHITECTURE.md complete (20 sections + slice plan + package layout), under fresh Opus critique
- Slice: 2 — architecture review and representative prototype

## Baseline

- Revision: d5e7075 (main). Worktree clean except untracked-noise `.DS_Store`.
- Toolchain: cargo/rustc 1.98.0; package `junie-tui` edition 2024, MSRV 1.88; ratatui 0.30 (crossterm_0_29), unicode-width 0.2, unicode-segmentation 1.
- Gates before refactor (logs in target/baseline/*.log):
  - `cargo fmt --check` exit 0
  - `cargo clippy --all-targets -- -D warnings` exit 0, 0 warnings
  - `cargo test --all-targets` exit 0 — 76 (lib) + 63 (jackin) + 26 (tablepro) + 33 (showcase) = 198 passed, 0 failed
  - `cargo build --bins` exit 0
- Pre-existing failures (from captures, see baseline/before/NOTES.md):
  1. PANIC jackin-preview View→Container info: src/bin/jackin_preview/screens/capsule.rs:1183 `&i.run_id.replace('-', "")[..8]` on 7-byte id (unguarded byte slice). Fix in Slice 7 (structural: width-safe truncation helper).
  2. jackin `Ctrl+B i` listed in View menu but prefix handler rejects it (menu/binding drift — the B9 binding-table design removes the class).
  3. `Ctrl+\` palette chord undeliverable via tmux legacy encoding (harness limitation, not app bug).
  4. jackin F10 reopens last-used menu, not File (semantics decision deferred to Slice 7 Opus review).
  5. TablePro has no menu bar (by design today).
- Perf baseline (WP-0): commit 07cb2c9, tag perf/baseline; tests/perf_baseline.txt blessed in release. Key before-numbers: showcase lists frame 213 allocs; tablepro grid frame 1,030 allocs; jackin capsule 4-pane frame 1,080,602 allocs / 74 MB; viewport 100k-line render 15.2M allocs / 1 GiB per frame; grid load 61,005 allocs; tree 100k toggle 596,840 allocs.
- Source size: ~72K lines; lib ~16K (core/ui/theme/runtime/widgets), showcase ~8K, tablepro ~12K, jackin ~35K.
- Before-refactor captures: baseline/before/ — 499 captures (208 showcase, 108 tablepro, 183 jackin) as .ansi/.txt/.cursor (committed) + .html/.png (gitignored, regenerable via `tools/baseline_capture.sh all`); MANIFEST.md has exact key/mouse recipes; NOTES.md findings.

## Assignments

- Coordinator: `refactor-coordinator` (`claude-fable-5-1`, high) — this session.
- Implementation: `fable-builder` (`claude-fable-5-1`, high).
- Research/review: `opus-analyst` (`claude-opus-5`, high, read-only).
- Slice 1 Opus audits: docs/audit/app-audit.md, domain-boundary-audit.md, interaction-audit.md, architecture-research.md (done, committed); api-audit.md, performance-audit.md (all six done, committed).

## Accepted decisions

- COMPONENT_ARCHITECTURE.md (commit after cefc4b8) is the accepted architecture, pending Slice 2 critique edits. Adjudications recorded there: A retained state + props + explicit update/draw phases with pre-resolved intents (immediate-mode `show` rejected); B unified Id (FNV with separator+kind, ItemKey, typed Part, PartRef in registry); C `Response<A>{flow,invalidate,state,action}`; D tokens+recipes+StylePatch+overlay scopes, precedence 1–6, mono fallback rule; E layer stack/focus scopes/capture/wheel/cursor/invalidation; F workspace, crate name stays `junie-tui`, `author` module; G keyed collection vocabulary, DataTable removed, Grid split via GridModel/GridEditor/GridCellActions; H Field/TextEditorCore/BlurPolicy/Validate/Secret; I dispositions + J1–J13.
- §20.9 folds performance-audit R1–R7 and §6.3 as binding amendments (16 items) with named acceptance tests.
- §20.10 lists 14 intentional visual changes; each requires a docs/visual-changes.md entry before any baseline regeneration.
- Appendix A amends goal §27 Slice 4: family builders do not touch apps; showcase migration is entirely Slice 5 (disjoint ownership).
- Slice 2 review (docs/reviews/slice2-architecture-review.md): adjudications A–I upheld; 16 BLOCKER / 31 MAJOR / 24 MINOR surface corrections accepted in full; recorded as §21 Adjudication J. Three amendments: data passed to update/draw phase calls (B3/B15); `Ui::cache<T>(id)` for derived per-frame caches + rule R8 (B5); Esc dismissal after app.update (B6).
- Staging (Opus §7, replaces coordinator proposal): root package stays `junie-tui` and becomes workspace root; new library at crates/tui named `tui-next`/`tui_next` during Slices 3–4 (TEMPORARY, deliberate — do not "fix"); single scripted rename to `junie-tui`/`junie_tui` at start of Slice 5 when root src/ and root bins are removed. Mapping: `junie_tui::author` ⇔ `tui_next::author` until then. `junie-tui-legacy` rename REJECTED (doc-target collision, duplicate bin names, default-run).
- User directives (2026-09-04): use latest dependency versions and latest APIs/modern practices (Opus modern-api audit running; direct deps already latest: ratatui 0.30.2, crossterm 0.29.0, unicode-width 0.2.2, unicode-segmentation 1.13.3); ALL audits and implementations in subagents — coordinator only spawns, records, commits.
- Open architecture decisions (Opus K1/K2 running): `Form` API sketch; `Grid::update` model bound.
- Slice 2 prototype scope (review §9c): Button, Field+TextInput, List, Tabs, Dialog-as-layer, ScrollRegion, minimal runtime/theme; examples 01,05,06,07,08,09,10,11,12 verbatim; conformance 20 cases × 7 components; render digests junie/paper × truecolor/mono; overrides tests; overlay tests; migrated Buttons page in new crate with 3 retained test intents; xtask doc-check; legacy tree green; then fresh Opus API review of actual code.

## File ownership

- fable-builder "baseline-capture": baseline/before/**, tools/** (capture additions only). No src/ edits.
- (done 95ab652) arch-edits builder: §21 Adjudication J applied (34 items, 44 inline markers), docs/visual-changes.md created.
- fable-builder "wp0-digests": src/bin/{tablepro,jackin_preview}/visual_tests.rs (+ mod line in main.rs), tests/baselines/{tablepro,jackin}.txt, .github/workflows/{ci,perf}.yml.
- opus-analyst (running): docs/audit/modern-api-audit.md; docs/reviews/adjudication-k-form-grid.md (K1 Form API, K2 Grid::update bound).
- (done) perf-baseline builder: tests/perf*.rs, src/bin/*/perf_tests.rs.
- (done) baseline-capture builder: baseline/before/**, tools/baseline_capture.sh.

## Completed gates

- Slice 1 baseline commands: done (see Baseline).
- WP-0 perf baseline: done (07cb2c9); fmt/clippy/test green after harness added.

## Unresolved findings

- Library domain leaks (grep sweep, coordinator): src/widgets/grid.rs (105 SQL/NULL/PK/FK hits), src/widgets/dialog.rs:21 ("(SQL)" doc), src/widgets/statusbar.rs:335 (jackin label in test). Apps: 82 `.owns(`/`.locate` call sites.
- Research conflict to adjudicate (Opus synthesis): interaction-audit B2/B3 proposes retained handle/render split with runtime-delivered `Event` + `Response{flow,invalidate,action}`; architecture-research §2 proposes immediate-mode `show(ui, area)` with intent queue drained during show. Must weigh Scenario A ergonomics vs migration of ~55K lines of app code and deterministic test harnesses.
- app-audit agent reported no grep/glob; its counts are lower bounds over files read in full.
- Perf finding P1 (new cost centre): TextViewport::render calls ensure_layout twice per frame (width, then width-1 on overflow) so the whole buffer re-lays out every frame even with no push — 15.2M allocs/frame at 100k lines. Must be covered by §20.9 item 7 (windowed incremental layout); acceptance `viewport_100k_lines_render` allocs independent of buffer size.
- Perf finding P2: debug vs release differ by exactly one 3-byte alloc (optimizer-elided String). Decision needed for `debug_and_release_alloc_counts_match` tolerance (defer to Slice 3 gate Opus review; harness currently reports).
- Perf harness deviations: thread-local counters (not global atomics) for exactness under concurrent tests; realloc counted; `hits` ±10% enforced inside report(); PERF_TARGET=1 gates post-refactor assertions.

## Next action

- When modern-api audit + K1/K2 land: builder records them as §22/§21 additions; then spawn Slice 2 prototype builder (workspace root + crates/tui as tui-next, prototype scope above). Then fresh Opus review of actual API; then Slice 3 foundations completes the crate.
