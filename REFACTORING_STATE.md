# Refactoring State

**RESUMED 2026-09-04 on Opus 5 routing (see Assignments). Ground truth verified at HEAD 0c87fb0: `cargo build -p tui-next --all-targets` OK; 190 lib tests pass; legacy root package 247 tests green; two expected failures from the interruption — `crates/tui/tests/conformance.rs` does not compile (components WIP) and `architecture::doc_check_resolves_every_reference` fails (architecture amendments half-applied). Both are owned by running builders.**

## Status

- Overall: Slice 3 (foundations) built and independently reviewed; corrections pending. Slice 2 prototype components partially built (WIP).
- Slice: 3 — foundations correction pass + prototype components, before Slice 4.

## Baseline

- Revision: d5e7075 (main) at start. Toolchain rustc 1.98.0; MSRV 1.88 held (Adjudication L §22.5); edition 2024.
- Deps (verified latest stable): ratatui-core 0.1.2, ratatui-crossterm 0.1.2 (crossterm 0.29), unicode-width 0.2.2, unicode-segmentation 1.13.3, bitflags 2.13. smallvec REJECTED. Legacy root package still on ratatui 0.30.2.
- Gates before refactor: fmt/clippy/test/build all green; 198 legacy tests (now 247 incl. perf/visual harness tests).
- Pre-existing failures (baseline/before/NOTES.md): (1) PANIC jackin View→Container info, src/bin/jackin_preview/screens/capsule.rs:1183 `[..8]` slice; (2) jackin `Ctrl+B i` listed in menu but rejected; (3) `Ctrl+\` undeliverable via tmux (harness); (4) jackin F10 reopens last menu; (5) TablePro has no menu bar; (6) TablePro wall-clock `Instant::now()` for status/flash.
- Before-refactor evidence: baseline/before/ (499 captures, MANIFEST.md); tests/baselines/{tablepro,jackin}.txt digests; tests/showcase_baseline.txt; tests/perf_baseline.txt (tag perf/baseline).

## Assignments (agent definitions in .claude/agents/)

- Coordinator: `refactor-coordinator` (**claude-opus-5**, high). Implementation: `fable-builder` (**claude-opus-5**, high). Research/review: `opus-analyst` (claude-opus-5, high, read-only).
- **MODEL DEVIATION, USER-AUTHORIZED 2026-09-04.** The goal mandates `claude-fable-5-1` for the coordinator and every implementation worker and says to treat model substitution as a blocker. Fable 5.1 credits were exhausted mid-session (HTTP 429 killed three builders). The user directed: "Continue using only Opus 5. We run out of tokens for Fable." `.claude/agents/{refactor-coordinator,fable-builder}.md` and `.claude/settings.json` were repointed to `claude-opus-5`, effort high. Agent definitions still own routing (no per-invocation overrides). The separation of duties is unchanged: `opus-analyst` stays read-only for all research and review, `fable-builder` remains the only implementer, the coordinator only spawns/records/commits. Revert the three files to `claude-fable-5-1` if Fable capacity returns and the mandate is to be honoured literally.
- USER DIRECTIVES: (a) all audits and implementations in subagents — coordinator only spawns, records, commits; (b) use latest dependency versions and latest APIs/modern practices (Adjudication L); (c) commit and push `origin/main` frequently — after every recorded result.
- `SendMessage` tool is unavailable in this environment: continuation of an agent = spawn a fresh one with the file paths as context.

## Accepted decisions (all recorded in COMPONENT_ARCHITECTURE.md; reviews under docs/reviews/, audits under docs/audit/)

- A–I: component model (retained XState + props + explicit update/draw; immediate-mode `show` rejected), Id/ItemKey/Part/PartRef, `Response<A>`, tokens+recipes+StylePatch+overlays precedence 1–6, layer stack/focus scopes/capture/wheel/cursor, workspace + crate name `junie-tui`, keyed collections + Grid split, Field/TextEditorCore/Secret, dispositions + J1–J13.
- J (§21): Slice 2 review corrections (34 items). Staging: root package stays `junie-tui`; new lib at crates/tui is TEMPORARILY `tui-next`/`tui_next` until Slice 5 (single scripted rename). `junie-tui-legacy` rename REJECTED.
- K (§23): Form API (library component, `FormData`, no `values()`); Grid `update<M: GridModel>(&M)` + `update_editable<M: GridEditor>(&mut M)`; `GridCellActions` deleted.
- L (§22): ratatui-core + ratatui-crossterm (normal dep; `crossterm` feature gates only the session), no ratatui/ratatui-widgets/macros; one width fn via CellWidth; set_stringn/set_line/set_span painters; Stylize banned; Masked forbidden; BorderSet = alias of symbols::border::Set; bitflags yes, smallvec no; 20 rules R-1..R-20 + 26 forbidden patterns; lints block; MSRV 1.88 with `cargo +1.88.0 check` gate.
- M (§24): no rename of our Size/Span; ratatui text types only in `author::raw`; `theme::border::ASCII` const; `FieldKind` closed over Label* aliases; `FormData::{options,value_and_options}`.
- Slice 3 foundations review (docs/reviews/slice3-foundations-review.md) — ACCEPTED in full: 7 BLOCKERS (BL-1 precedence variant-after-state-rules; BL-2 spin-loop "unreachable"; BL-3 Ui::raw marks clip/clobbers roles; BL-4 paint_spans allocates; BL-5 Ansi16 CIE76 → restore legacy categorical metric; BL-6 set_cursor first-writer; BL-7 Harness::resolved hardcodes BUTTON), 14 MAJOR, 16 MINOR; fix list F1–F26; 8 adjudications (crossterm normal dep confirmed; Id structural derive confirmed; CIE76 rejected; fit split inline/wide; §22.7(2) split 2a–2d; intents-drain probe counts; Track::Auto accepted + rows_measured; style bound per-frame ≤5% + cache hit ≥90%); deviations D-1..D-13 (D-3 rejected, D-10 broad regex + path allow, D-11 `GlyphRole::SecretMask`).
- N (docs/reviews/adjudication-n-layer-measure.md) — ACCEPTED: `LayerSpec.size: LayerSize{Fill,Fixed}` replaces `min_size`; `Cx::resize_layer/reanchor_layer`; `Anchor::Point` flips; Dialog sizes its own layer (`Dialog::layer(cx)`, `measured_width/height`, `body_rows`, `text::wrapped_rows`); `Ui::resolve(&self)` uncached no-record path, `Ui::glyph_str`, `Theme::metrics/PartMetrics`, `Ui::with_part`, `Ui::surface_style`, `Resolved::over`. `Ui::scroll_region` open for 4E.

## File ownership (active)

- DONE (587c53b) `arch-amend`: §25 (eight adjudications, D-1..D-13 verdicts, F1–F26 obligation table with test names, gate additions) and §26 (Adjudication N) appended; 83 `§25` + 35 `§26` inline markers; superseded text struck in place (§20.9-1 per-query bound, §21 item 20 `(u16,u16)`, §21 item 29 CIE76, §24.5 `author/raw.rs` file-placement paragraph, Appendix B.2 optional-crossterm). §17 self-check: 44 references, 0 unresolved library references.
- RUNNING `foundations-fix`: owns crates/tui/src/** except components/**, crates/tui-testing/src/**, xtask/**, crates/tui/tests/{architecture,perf,render,fixtures,allow,ui}, crates/tui/Cargo.toml, crates/tui/README.md, root Cargo.toml. Applies F1–F26 + N1/N2. Permitted mechanical call-site updates in components/** where a signature changes; must not add component features. `crates/tui/tests/conformance.rs` excluded from its gates.
- PENDING `components-finish` (serial, after foundations-fix): owns crates/tui/src/components/**, crates/tui/examples/**, crates/tui/tests/{conformance,overrides,overlay,showcase_buttons}.rs and baselines. 9 component files exist (button, dialog, field, input, list, props, scroll_region, tabs, mod); examples 01,05–12 exist; overrides.rs / overlay.rs / showcase_buttons NOT yet present.

## Completed gates

- Slice 1: audits (6) + before-captures + digests + perf baseline (WP-0) — all committed.
- Slice 2: architecture written, reviewed (slice2-architecture-review.md), corrected (§21–§24).
- Slice 3 foundations at 18afddd: fmt/clippy/test/doc/no-default-features/architecture/perf gates green; legacy 247 tests green; doc-check 305 refs; boundary 17/17. Post-review: NOT gated (corrections pending).

## Unresolved findings

- Three names declared by the `arch-amend` builder because the accepted decisions plus the §17 self-check forced them, but which neither review spelled out — flag to the next fresh `opus-analyst` for confirmation: `Picker::measured_size(&self, cx, items) -> LayerSize` and the same on `Select` (§26 N1 mandates the popover `.size(...)` but names no method); `Props::measure(&self, ui, c) -> Size` (the review's example-9 rewrite calls it); the `Dialog::body_rows` values used in examples 9 and 10 (derived arithmetic, not from the review).
- `docs/visual-changes.md` needs an entry for §20.10 item 17 (`Anchor::Point` flip) before any tooltip or context-menu baseline is blessed.
- §25.3's ΔE figures are carried through as the review marked them — hand arithmetic, to be re-derived before blessing.
- Open, not decided: `Ui::scroll_region(id, part, …)` (Slice-3-owned, blocks 4E); rustdoc-json upgrades for `every_foreign_type_in_the_public_surface_is_re_exported` and `xtask doc-check`, both deferred to Slice 8.

- F1–F26 correction obligations (slice3-foundations-review.md §5) and N1/N2 code changes not applied.
- Components WIP state unknown; `cargo test -p tui-next --all-targets` must be re-run first.
- Perf findings P1 (viewport double relayout), P2 (debug/release 1-alloc delta; review adjudicated tolerance ≤1) — P1 addressed by §20.9 item 7 (Slice 4E).
- Open for 4E: `Ui::scroll_region` convenience.

## Next action (Resume)

1. `git status`; `cargo test -p tui-next -p tui-next-testing --all-targets --all-features` and `cargo run -p xtask -- boundary` to learn the WIP state. Do NOT run the legacy digest blessing.
2. Spawn fable-builder "arch-amend": redo §25/§26 + inline amendments per docs/reviews/slice3-foundations-review.md (§2, §3, §4(f)) and docs/reviews/adjudication-n-layer-measure.md ("Document amendments" table); owns COMPONENT_ARCHITECTURE.md only. Commit + push.
3. Spawn fable-builder "foundations-fix": apply F1–F26 + N1/N2 code changes to crates/tui/src (non-components), crates/tui-testing, xtask; re-bless crates/tui/tests/perf_baseline.txt with note; run the review §6 gate command set. Serial with step 4 (shared crate).
4. Spawn fable-builder "components-finish": bring components/examples/tests to the slice2 review §9(c) acceptance (conformance 20×7, render digests junie/paper × truecolor/mono, overrides tests, overlay tests, showcase_buttons example + 3 retained tests, component perf benches), adapting to the F-fixes (paint_spans signature, LayerSize, Ui::resolve, Resolved::over).
5. Spawn fresh opus-analyst: review of the prototype's real API (slice2 review §9(c) item 14) + verify every §6 gate; record; correction pass if needed.
6. Slice 3 gate green → Slice 4 wave 1 (4A,4B,4C,4E,4G) parallel per Appendix A with disjoint files; each followed by fresh Opus API-consistency review; wave 2 (4D,4F,4H,4I); Slice 5 (rename tui-next→junie-tui + showcase); Slices 6/7 parallel; Slice 8 cleanup + two fresh Opus reviews + final report (§30).
