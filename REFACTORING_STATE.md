# Refactoring State

## Status

- Overall: Slice 2/3 — architecture accepted (Adjudications A–M); foundations crate under construction
- Slice: 3 — foundations (crates/tui as tui-next), prototype components next

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
  6. TablePro reads `Instant::now()` for status (5 s) and press flash (140 ms) — wall-clock in app state; digests stable in practice but the refactor injects a clock (runtime-owned motion tokens, §8.5).
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
- User directive: commit and push frequently — coordinator pushes `origin/main` after every recorded result.
- User directives (2026-09-04): use latest dependency versions and latest APIs/modern practices (Opus modern-api audit running; direct deps already latest: ratatui 0.30.2, crossterm 0.29.0, unicode-width 0.2.2, unicode-segmentation 1.13.3); ALL audits and implementations in subagents — coordinator only spawns, records, commits.
- Adjudication L (modern API, docs/audit/modern-api-audit.md) ACCEPTED: library depends on `ratatui-core` 0.1.2 (features std, underline-color) + `ratatui-crossterm` 0.1.2 behind default `crossterm` feature; never `ratatui`/`ratatui-widgets`/`ratatui-macros`; apps depend on `junie-tui` only; crossterm reached only via `ratatui_crossterm::crossterm`; one width fn via `CellWidth::cell_width`; painters = `Buffer::set_stringn`/`set_line`; `Stylize` banned in lib/apps; `Masked` forbidden; no keyboard-enhancement flags; `BorderSet` = alias of `symbols::border::Set`; bitflags 2.13 adopted; smallvec REJECTED (Vec; sorted-Vec KeySet); MSRV stays 1.88 with `cargo +1.88.0 check` gate; 20 binding rules + `architecture::no_deprecated_or_legacy_api_usage` (26 patterns) + `dependency_graph_is_exactly_the_declared_set`; `[workspace.lints]` with clippy::all deny, pedantic warn, panic/indexing_slicing/unwrap/expect/todo/unimplemented deny.
- Adjudication K ACCEPTED: K1 `Form` is a library component (4F): `FieldSpec`/`FieldKind`/`FormData{value,value_mut,visible,disabled,error,validate,validate_all}`/`FieldRef`/`FieldMut`/`Form::{update,draw}` with data per phase, `FormAction{Changed,Committed,Chose,Action,Invalid}` carrying no values (no `values()`; secrets never leave the caller), invariants F1–F13. K2: `Grid::update<M: GridModel>(…, &M)` + `Grid::update_editable<M: GridEditor>(…, &mut M)` + `draw<M: GridModel>`; `read_only_reason` and `actions` move onto `GridModel`; `GridCellActions` and `Grid::editable(bool)` deleted.
- Slice 2 prototype scope (review §9c): Button, Field+TextInput, List, Tabs, Dialog-as-layer, ScrollRegion, minimal runtime/theme; examples 01,05,06,07,08,09,10,11,12 verbatim; conformance 20 cases × 7 components; render digests junie/paper × truecolor/mono; overrides tests; overlay tests; migrated Buttons page in new crate with 3 retained test intents; xtask doc-check; legacy tree green; then fresh Opus API review of actual code.

## File ownership

- fable-builder "baseline-capture": baseline/before/**, tools/** (capture additions only). No src/ edits.
- (done 95ab652) arch-edits builder: §21 Adjudication J applied (34 items, 44 inline markers), docs/visual-changes.md created.
- (done) wp0-digests builder: tablepro 42 + jackin 36 pre-refactor cell-exact digests in tests/baselines/, CI workflows .github/workflows/{ci,perf}.yml; tag perf/baseline moved to this commit.
- (done) opus: docs/audit/modern-api-audit.md; docs/reviews/adjudication-k-form-grid.md.
- (done 27bd918) arch-fold builder: §22 Adjudication L, §23 Adjudication K, §15.1 Form API, §17 example 13, 46 inline markers.
- (done) Adjudication M recorded at docs/reviews/adjudication-m-small-items.md (ACCEPTED: M1 no rename — ratatui `Size`/`Line`/`Span`/`Text` not re-exported at root, `author::raw::{Line,Span,Text}` only, `Frame` root-only, `Ui::paint_spans`; M2 `theme::border::ASCII` plain const, Junie=ROUNDED, Paper=PLAIN, no auto selection; M3 `FieldKind` closed over `LabelSelect/LabelRadio/LabelChips` aliases, `FormData::{options,value_and_options}`).
- (done) arch-record-M: §24 appended, 32 markers; self-declared (not adjudicated) names recorded in §24.4: `SelectAction{Chose,Opened,Closed}`, `RadioGroupAction{Chose}`, `ChipBarAction{Toggled,Closed,Activated}`, `SelectState`/`RadioGroupState`/`ChipBarState`; `author::raw` lives in crates/tui/src/author/raw.rs (§24.5) — to be confirmed by the post-prototype Opus API review.
- (done 18afddd) foundations builder: workspace root; crates/tui (`tui-next`, TEMPORARY name), crates/tui-testing, xtask; 75 files, 22.8K lines; gates green (187 lib + 20 arch + 21 conformance + 18 perf + 5 render tests; legacy 247 tests green; doc-check 305 refs resolved; boundary 17/17). Twelve deviations + eight research requests recorded in its report → adjudication running.
- (done) docs/reviews/slice3-foundations-review.md: verdict "components may build on this surface: NO as it stands; YES after 7 blockers + 8 adjudications". BLOCKERS: BL-1 precedence applies variant after family state rules (test vacuous: focus==accent in Junie); BL-2 spin-loop "unreachable" arms; BL-3 Ui::raw marks whole clip + clobbers roles (CellUi::drop); BL-4 paint_spans allocates per row; BL-5 Ansi16 CIE76 contradicts DESIGN.md:320 → restore legacy categorical metric; BL-6 set_cursor keeps first writer not focused; BL-7 Harness::resolved hardcodes Family::BUTTON. 14 MAJOR (hit ordering by layer, xtask cfg(test) scan bug, fit-differential test weakened, crossterm feature gates nothing, capture origin, custom family resolves empty, resize drops focus intents, conformance cases 9/12 weak, every_named_test_exists missing, foreign-type check is a grep, trybuild absent, zeroize elidable, doc-check misses §24), 16 MINOR. Adjudications: 1 crossterm normal dep CONFIRMED (+rule 27, two-file check); 2 Id structural derive CONFIRMED (test rewritten); 3 CIE76 REJECTED; 4 fit split inline(0)/wide(bounded); 5 §22.7(2) split 2a–2d; 6 probe-count assertion; 7 Track::Auto ACCEPTED + example 9 uses rows_measured; 8 style bound = per-frame ≤5% + cache hit ≥90%. Deviations D-1..D-13 accepted except D-3 (rejected), D-10 (broad regex + path allow), D-11 (`GlyphRole::SecretMask`). Fix list F1–F26 pending (correction pass after components builder finishes).
- Adjudication N ACCEPTED (docs/reviews/adjudication-n-layer-measure.md): N1 `LayerSpec.size: LayerSize{Fill,Fixed(w,h)}` replaces `min_size (0,0)` sentinel; runtime stays the one resolver (clamps, never grows); `Cx::resize_layer`/`reanchor_layer` are the only mutable spec fields; `Anchor::Point` flips; Dialog sizes its layer from props+DesignTokens (`body_rows`, `measured_width/height`, `Dialog::layer(cx)`, re-asserted every update); new `text::wrapped_rows`. N2 `Measure::measure(&self, &Ui, c)` unchanged; new `Ui::resolve(&self,…)` uncached/no-record path, `Ui::glyph_str`, `Theme::metrics -> PartMetrics`; `Ui::style` stays `&mut` and is the only recording query; `Ui::with_part` and `Ui::surface_style` confirmed; `Resolved::over(inherited)` added. `Ui::scroll_region` left as open item for 4E.
- fable-builder "components": crates/tui/src/components/**, examples 01,05–11 + showcase_buttons, tests conformance/render/overrides/overlay/showcase_buttons + baselines + perf additions, append-only lib.rs/author.rs re-exports. Foundation files frozen for it.
- (done) perf-baseline builder: tests/perf*.rs, src/bin/*/perf_tests.rs.
- (done) baseline-capture builder: baseline/before/**, tools/baseline_capture.sh.

## Completed gates

- Slice 1 baseline commands: done (see Baseline).
- WP-0 perf baseline: done (07cb2c9); digests + CI done (next commit; tag perf/baseline). Full §26-shaped gate run green on single package: fmt, clippy -D warnings, test (lib 76, jackin 67, showcase 33, tablepro 41, perf 30), doc tests, doc -D warnings, build --all-targets.

## Unresolved findings

- Library domain leaks (grep sweep, coordinator): src/widgets/grid.rs (105 SQL/NULL/PK/FK hits), src/widgets/dialog.rs:21 ("(SQL)" doc), src/widgets/statusbar.rs:335 (jackin label in test). Apps: 82 `.owns(`/`.locate` call sites.
- Research conflict to adjudicate (Opus synthesis): interaction-audit B2/B3 proposes retained handle/render split with runtime-delivered `Event` + `Response{flow,invalidate,action}`; architecture-research §2 proposes immediate-mode `show(ui, area)` with intent queue drained during show. Must weigh Scenario A ergonomics vs migration of ~55K lines of app code and deterministic test harnesses.
- app-audit agent reported no grep/glob; its counts are lower bounds over files read in full.
- Perf finding P1 (new cost centre): TextViewport::render calls ensure_layout twice per frame (width, then width-1 on overflow) so the whole buffer re-lays out every frame even with no push — 15.2M allocs/frame at 100k lines. Must be covered by §20.9 item 7 (windowed incremental layout); acceptance `viewport_100k_lines_render` allocs independent of buffer size.
- Perf finding P2: debug vs release differ by exactly one 3-byte alloc (optimizer-elided String). Decision needed for `debug_and_release_alloc_counts_match` tolerance (defer to Slice 3 gate Opus review; harness currently reports).
- Perf harness deviations: thread-local counters (not global atomics) for exactness under concurrent tests; realloc counted; `hits` ±10% enforced inside report(); PERF_TARGET=1 gates post-refactor assertions.

## Next action

- MSRV re-examined during refactor and deliberately held at 1.88 (Adjudication L §22.5); CI gate `cargo +1.88.0 check --workspace --all-targets --all-features` makes it a fact.
- After components builder + foundations review: correction builder applies review fixes; fresh Opus review of the prototype's real API (slice2 review §9c item 14); then Slice 3 gate → Slice 4 waves. (Previously: spawn components builder (Button, Field+TextInput+Secret, List, Tabs, Dialog-as-layer, ScrollRegion; examples 01,05–12; conformance/render/override/overlay tests; migrated Buttons page in-crate) → fresh Opus review of real API → Slice 3 gate → Slice 4 waves (workspace root; crates/tui as tui-next on ratatui-core; scope per review §9c + Adjudication L rules) (workspace root + crates/tui as tui-next, prototype scope above). Then fresh Opus review of actual API; then Slice 3 foundations completes the crate.
