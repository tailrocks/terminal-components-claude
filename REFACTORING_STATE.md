# Refactoring State

**RESUMED 2026-09-04 on Opus 5 routing (see Assignments). Ground truth verified at HEAD 0c87fb0: `cargo build -p tui-next --all-targets` OK; 190 lib tests pass; legacy root package 247 tests green; two expected failures from the interruption — `crates/tui/tests/conformance.rs` does not compile (components WIP) and `architecture::doc_check_resolves_every_reference` fails (architecture amendments half-applied). Both are owned by running builders.**

## Status

- Overall: **Slice 3 CLOSED and green at commit 0f66160** (797 tests, boundary + doc-check exit 0). **Slice 4 wave 1 was interrupted mid-flight by a session token limit.**
- Slice: 4 — component families, wave 1, PARTIAL.

## !! SESSION 2 INTERRUPTION (token limit) — READ THIS FIRST !!

Three subagents were killed mid-work. Their partial output is committed as WIP and **does not compile**:
`cargo build -p tui-next --all-targets` fails with one `E0502` (borrow conflict) in the `tui-next` lib-test target.

Last fully green commit: **`0f66160`** (`cargo test --all-targets` exit 0, 797 passed). Everything after it is WIP.

**First action on resume:** `cargo build -p tui-next --all-targets`, read the `E0502`, and decide per-file whether to finish or revert. `git diff 0f66160 -- crates/` shows exactly what the three builders had produced. Reverting a single unfinished component to `0f66160` and re-running that package is a legitimate and often faster choice than repairing a half-written file.

Interrupted packages and what each had produced:
- `digest-race-fix` — owns `crates/tui-testing/src/{digest.rs,harness.rs}`. `digest.rs` modified. Task: replace the per-assertion whole-file read-modify-write with an accumulate-and-write-once or locked merge, so blessing is thread-count independent. Must prove byte-identical output at 1 and N threads, add a concurrency regression test, and move **no** baseline.
- `4B` fields/inputs — owns `components/{field,input,textarea,select,choice,chip,secret,validate}.rs`, `examples/06`, own test/digest additions. New files present: `chip.rs`, `choice.rs`, `select.rs`, `textarea.rs`; `input.rs` modified.
- `4G` status/hints/progress/meters — owns `components/{status,hintbar,progress,meter,empty,brand,keyhint}.rs`. New files present: `brand.rs`, `empty.rs`, `hintbar.rs`, `keyhint.rs`, `meter.rs`, `progress.rs`. `status.rs` **not yet written**.
- Contended files `components/mod.rs` and `lib.rs` are modified and may reference modules that are absent or unfinished.

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
- N (docs/reviews/adjudication-n-layer-measure.md) — ACCEPTED: `LayerSpec.size: LayerSize{Fill,Fixed}` replaces `min_size`; `Cx::resize_layer/reanchor_layer`; `Anchor::Point` flips; Dialog sizes its own layer (`Dialog::layer(cx)`, `measured_width/height`, `body_rows`, `text::wrapped_rows`); `Ui::resolve(&self)` uncached no-record path, `Ui::glyph_str`, `Theme::metrics/PartMetrics`, `Ui::with_part`, `Ui::surface_style`, `Resolved::over`. `Ui::scroll_region` STRUCK by §35: no such method; the API is `ScrollRegion::new(id).draw(...)`. 4E was never blocked.

## File ownership (active)

### Slice 4 wave 1 (running)

- `digest-race-fix`: `crates/tui-testing/src/{digest.rs,harness.rs}`. Structural fix for the bless read-modify-write race. Must not move a baseline.
- `opus-analyst` Q1–Q3 (read-only): `Tabs` mono `PRESSED` mechanism (bracket idiom vs a `(Part::TAB, PRESSED)` rule vs teaching `RowUi::label` to honour the glyph slot — and whether `RowUi` ignoring it is itself a §12.2 defect); `Fixture::state_override` still public; an acceptance grep that can never pass.
- `4B` fields/inputs: `components/{field,input,textarea,select,choice,chip,secret,validate}.rs`, `examples/06`, own test/digest additions.
- `4G` status/hints/progress/meters: `components/{status,hintbar,progress,meter,empty,brand,keyhint}.rs`, own test/digest additions. `StatusBar` merges the legacy `statusbar` + `segments`.
- **Contended-file protocol** (`components/mod.rs`, `lib.rs`, `author.rs`, `xtask/named_tests_allow.txt`): minimal single-line insertions in alphabetical position, re-read immediately before each edit, retry on failure. The `Edit` tool fails on a stale match rather than clobbering, so a race surfaces as an error, not as lost work.
- HELD until Q1 lands (it may move `List` baselines): `4A` buttons/choices/brand-chrome, `4C` lists/trees/props/steps/nav, `4E` containers/scrolling. Wave 2 after: `4D` tabs, `4F` overlays, `4H` code/diff, `4I` grid.

- DONE (587c53b) `arch-amend`: §25 (eight adjudications, D-1..D-13 verdicts, F1–F26 obligation table with test names, gate additions) and §26 (Adjudication N) appended; 83 `§25` + 35 `§26` inline markers; superseded text struck in place (§20.9-1 per-query bound, §21 item 20 `(u16,u16)`, §21 item 29 CIE76, §24.5 `author/raw.rs` file-placement paragraph, Appendix B.2 optional-crossterm). §17 self-check: 44 references, 0 unresolved library references.
- DONE (7899678 — commit subject "feat(tui): add component architecture foundations" is a misnomer; the change is the F1–F26 + N1/N2 correction pass) `foundations-fix`. Verified independently by the coordinator: fmt clean; clippy `-D warnings` clean; `tui-next` lib 190→**220**; architecture 19+1F→**28**; render **8**; perf **22**; doc 1; `cargo check --no-default-features` OK; `RUSTDOCFLAGS=-D warnings cargo doc` 0 warnings; `xtask boundary` **23/23** with `legacy_api.txt` and `domain.txt` both empty; legacy root package **247** green. `tests/conformance.rs` 186 pass / **5 fail** — owned by the next builder, and 4 of the 5 are the intended effect of F17 (case 9 now requires the full ten mono states).
- DONE (8ec40c1) `components-finish`. Verified independently: conformance **193/193** (20 cases × 9 components + 2 suite-level + registry), render_components **48** (384 baseline lines: 6 components × 8 states × {junie,paper} × {truecolor,mono} × {120×40,40×10}), overrides **5**, overlay **4**, showcase_buttons **4**, lib **220**, render 8, perf 27, doc 1, examples build, legacy **247** green. Only failure repo-wide is `every_named_test_exists`, owed the three §27 test names.
- DONE (bb92a65) `adjudication-o-code`. Verified independently: **workspace fully green — 788 tests, 0 failing targets, `cargo test --all-targets` exit 0, `xtask boundary` exit 0, `xtask doc-check` exit 0**, `every_named_test_exists` passing, legacy root package 247. `perf_baseline.txt` diff is header-only; no data row moved. The generation-wrap test was proven a genuine detector by restoring the old code and watching it fail with the stale hit.
- DONE Adjudication P (docs/reviews/adjudication-p-prototype-decisions.md), which **corrected two of the premises I gave it**: (P3) a `Dialog`-owned modal records **no** `UndeliveredIntent` today, because the dialog registers only `Decorative` regions and the diagnostic is gated on `delivers_to` — so the gated `if cx.is_open(…)` shape drops the dismissal *silently*, which is worse than the leak I described; (P6) `ListCase` does **not** narrow `DISABLED`, and both it and `FieldCase` pass only because they paint `Part::LABEL`, which the existing rule reaches. **It also found a defect neither I nor the builder saw: at `ColorLevel::Mono`, `disabled_fg #4d4d4d` and `Fg(Faint) #262626` both map to `Black` on a `Black` canvas, so §11.4's prescribed `fg = Role::Fg(Faint)` makes a disabled control black-on-black — invisible, not merely colourless. This is a goal §29 violation caused by the specification, not the implementation.** Decisions: P1 keep the two-half showcase split until Slice 5 (widening the `.state_override` exemption would cost the matrix's assertions and loosen a gate permanently); P2 confirm the two-target render split, with the *test path* as the contract and every gate naming both targets; P3 widen the diagnostic to cover runtime-addressed intents whatever the owner registered, and make example 11 unconditional; P4 confirm `input_rows` in `measured_height`; P5 confirm `state_override`/`inherit_forced` and generalise `inherit_forced` onto `FieldControl` (a forced `Field` still registers a live control today); P6 add the `FIELD`/`TEXT` mono rules with `Fg(Primary)`, make `Fixture` carry `Status` so case 9 can see props-driven affordances, turn `mono_states_required_by` into a union, and revert the narrowings it was hiding.
- DONE `arch-record-P` (dc3e0fa, §28 + 26 markers) and `adjudication-p-code` (0f66160). Verified: fmt clean, clippy `-D warnings` clean, **`cargo test --all-targets` exit 0 with 797 passed / 0 failing targets**, `xtask boundary` exit 0 (24 checks incl. the new `inherit_forced_stays_crate_internal`), `xtask doc-check` exit 0 (71 blocks, 570 refs), perf 27, legacy root package 247. Baseline diff **20 insertions / 20 deletions, every changed line `mono`**, truecolor untouched, classified in `docs/visual-changes.md` before blessing.
- **COORDINATOR CORRECTION.** My previous turn reported the tree "fully green" at bb92a65. That was true when measured, but the doc half (dc3e0fa) then added six test names to §16.1, which made `every_named_test_exists` red until 0f66160 wrote those tests. I did flag the transient redness when committing dc3e0fa, but the earlier "fully green" line should have been scoped to the commit it measured. Recorded so the evidence trail is not overstated.
- P3 premise, measured rather than reasoned: the builder built a temporary gated-shape fixture and observed that `CONFIRM` is indeed never diagnosed and `DialogAction::Dismissed` never fires (`dismissed count: 0`), **but the loss is not silent** — a diagnostic fires for `action_id(0)`, the focused action button whose `FocusOut` also went undrained. So my original "example 11 leaks intents" was observably right and merely attributed to the wrong owner, and the adjudication's "silent" was wrong. The `FINDING` comment now states the measured owner.

## Completed gates

- Slice 1: audits (6) + before-captures + digests + perf baseline (WP-0) — all committed.
- Slice 2: architecture written, reviewed (slice2-architecture-review.md), corrected (§21–§24).
- Slice 3 foundations at 18afddd: fmt/clippy/test/doc/no-default-features/architecture/perf gates green; legacy 247 tests green; doc-check 305 refs; boundary 17/17. Post-review: NOT gated (corrections pending).

## Unresolved findings

- **Test-infrastructure defect, found while blessing: `Scene::assert_against` read-modify-writes the whole baseline file per assertion (`crates/tui-testing/src/digest.rs:220-232`), so parallel test threads clobber each other — `BLESS=1 … --test render --test render_components` truncated `components.txt` to 6 lines.** Recovered with `git checkout` and blessed with `--test-threads=1`. Structural fix owed: accumulate and write once, or lock. **RESOLVED** — the fix landed (see the 4G/digest addendum); blessing is now thread-count independent and the `--test-threads=1` constraint is withdrawn.
- Adjudication Q ACCEPTED and APPLIED (docs/reviews/adjudication-q-residuals.md). Q1: the `Tabs` bracket is **compliance with §11.4's existing `PRESSED` row**, not a new affordance — the row already mandates the bracket and merely never said who paints it; option (c) is rejected because bracketing inside `RowUi::label` would steal two content columns from every pressed row, and a mono fallback must never change geometry. The rule that scales: a component reserving pad cells paints the bracket into them; a `RowUi`-labelled collection row expresses `PRESSED` through the `CONTAINER` rule alone, as `ListCase` proves. Q2: `Fixture::state_override` and `status` are **private** with `forced()`/`status()` accessors, so `force` is the only post-construction writer — removing the enabling condition rather than documenting an invariant. Q3: the grep is **withdrawn** and replaced by `Conformance::mono_narrowing_reason()`, checked inside case 9 via `iter_names()` containment. `Caps::TRAPS_FOCUS` is now distinct from `Caps::OVERLAY`; case 14 gates on it, while `Select` remains `OVERLAY`-only and non-trapping.
- **The live `RowUi` glyph correction is applied.** `Resolved.glyph` and `PartMetrics.glyph` are `Slot<GlyphRole>`, and `RowUi::marker`/`RowUi::part` honor `Inherit`, `Set` and `Clear` without changing reserved geometry. Existing callers are `crates/tui/examples/07_borrowed_rows.rs:31` and `08_dynamic_tabs.rs:30`; A4 is a live caller/paint-semantics gate.
- The structural fix Q named — `Resolved.glyph: Option<GlyphRole>` could not distinguish `Slot::Clear` from unset — is applied as the `Slot<GlyphRole>` §11.2 amendment.
- Also found by Q: **§28.6's claim that the mono-narrowing intent "is satisfied by doc comments on all five cases" is false for four of the five** — `TabsCase` documents 1 of 5 drops, `FieldCase` 0 of 5, `TextInputCase` 0 of 4; only `ListCase` names all three. Q3's check makes this visible.
- **R1 is proven in three phases.** With the Tabs bracket enabled, `conformance::tabs::mono_states_are_distinguishable` exited `0`; with only `tabs.rs:719–728` disabled, it exited nonzero because mono `PRESSED`/`FOCUSED` became equal; restoring the block returned exit `0`. `CONTAINER`'s `BOLD` alone does not distinguish that pair.
- The residual bracket finding is resolved: the shared helper takes two reserved cells; `Button`, `Tabs` and `ChipBar` preserve label, total-width and close-cell geometry.
- **P1's Slice-5 obligation is not yet recorded as a task**: at Slice 5 the two halves of the showcase Buttons page (`crates/tui/examples/showcase_buttons.rs` and the `.state_override` state matrix in `crates/tui/tests/showcase_buttons.rs`) merge into `apps/showcase`, and §18.3 #4's deviation paragraph is struck. Carry this into the Slice 5 work package.
- Deviations from the letter of Adjudication P, each documented in place and flagged for review: `Field` applies `inherit_forced` in its consuming builder rather than in `draw` (the prescribed signature consumes `self`, and `draw` takes `&self`); `state_override_is_used_only_in_apps_and_fixtures` now skips `#[cfg(test)]` regions in library source, brace-tracked and verified still red for a production call; `a_layer_owners_dismissal_…` asserts "at least one, and every one names the owner" because the guard runs per pass and focus restoration re-runs `update`.

- Adjudication O ACCEPTED (docs/reviews/adjudication-o-foundations-followups.md), resolving the four `foundations-fix` research requests. O1: two-way memo accepted; §11.1 A3 never said "direct-mapped" — only §20.9-2 and a stale `resolve.rs:7` module doc do, and both are struck; the ≥ 90 % floor is re-purposed as a key-correctness floor, with `cache_hits_after_the_first_query_and_clears_by_generation` as the deterministic guarantee. **Also found a latent bug**: `StyleCache::clear` wraps the generation to 1 after 2^32 clears and can then serve a stale entry — fix plus `cache_generation_wrap_does_not_serve_a_stale_entry`. O2: the ASCII glyph coupling is confirmed necessary (those four roles are exactly the box-drawing-block bindings) and the four glyphs are confirmed against DESIGN.md, but the mechanism moves to a public `ThemeBuilder::ascii_glyphs()` replacing **whole typed sets** — closing `scrollbar::Set.begin`/`.end`, which no `GlyphRole` can reach and which would have broken the ASCII test the day 4E lands; the full `GlyphSet` ASCII table is **scheduled for 4E**, not deferred. O3: `border_subtle → DarkGray` (luma 38, band 31..=110); the document was wrong and the claim was never a carried fact — the legacy pin constrains only accent/error/canvas; two §25.3 ΔE estimates also corrected (35.0/62.6 and L* 79.2, ΔE 17.8 vs 34.9). O4: both substitutes confirmed, with three corrections — the extrapolation multiplier is ×12.5 not ×10 (the differential covers 160 queries, not 200), the binding budget moves to `style_resolve_10k_parts` at ≤ 16 ns/query, and the ≤ 5 % share is reinstated in **Slice 5** against `frame_showcase_lists_120x40`. Slice 4 wave 1 is not blocked by any of the four.
- Adjudication O doc half DONE (4aabceb): §27 appended, 24 inline markers, overruled claims struck in place. Verified: `direct-mapped` survives only inside `~~…~~`; `border_subtle . Black` grep empty; `Slice 4E` scheduling present; `xtask doc-check` exit 0 (71 blocks, 557 references); the three new test names are visible to `every_named_test_exists`, which now correctly reports them as missing until the code lands.
- Adjudication O CODE half PENDING, assigned after `components-finish` returns (both touch `tests/perf.rs`): `theme/resolve.rs:7` stale module doc + the generation-wrap correctness fix; `theme/glyph.rs` whole-set mutators and `ASCII_RULE_QUIET`/`ASCII_RULE_ACTIVE`/`ASCII_SCROLLBAR`; `theme/builder.rs::ascii_glyphs`; `theme/builtin/junie.rs` shadowed array slots 29–32; `tests/perf.rs` (×12.5 multiplier, ≤ 16 ns/query budget, normalised intents ratio); `perf_baseline.txt` header. Three new tests owed: `theme::ascii_glyph_set_has_no_box_drawing`, `theme::builder::ascii_glyphs_is_idempotent_and_glyph_overrides_it`, `theme::cache_generation_wrap_does_not_serve_a_stale_entry`.
- `FrameServices` is named in §17.0's `Cx<'f>` field comment but declared nowhere in the document — pre-existing, unrelated to §27; declare it or drop the comment.
- Bug found and fixed by the components builder while writing `tests/overlay.rs`: `Dialog::update` consumed `Intent::Layer(_)`, so `LayerEvent::Opened` (delivered a frame after the open) swallowed **the first Esc after a modal opened** and the modal could not be dismissed. Root cause: `Acc` had no way to invalidate without consuming. Fixed structurally with `Acc::repaint()`; `Intent::Cancel` still consumes.
- §17 example 10 deviates from the document because `Picker` does not exist until 4D: it substitutes `List`/`ListAction::Chose` and needs `List::measured_size`. Because a `List` does not own the layer, the opener re-asserts the size where §17 has `Picker::update` doing it. Revisit at 4D.
- §17 example 9 did not compile as written (`Props::new` borrows a temporary array); fixed by naming the array. Recorded as a document finding.
- §17 example 11 finding: a dialog's `actions` must be in the props constructor, not supplied only in `update`, or `measured_height` sizes the layer for an action row `draw` never paints.
- Deferred by `foundations-fix` to the components builder: the two suite-level conformance tests `conflicting_visible_bindings_are_reported` and `draw_registers_nothing_when_it_cannot_draw` (recorded in `xtask/named_tests_allow.txt`, which can only shrink).
- `every_named_test_exists` currently scans sources for `#[test] fn name` rather than shelling out to `cargo test -- --list` (a nested release build per run is not viable); 319 documented names, 238 present, 81 deferred with owners.
- `PERF_STRICT=1` fails on the ns column for ~9 benchmarks on this machine under concurrent load; the allocation and byte columns — the hard assertions — all pass. `PERF_STRICT` is opt-in and not in the required gate list.

- Three names declared by the `arch-amend` builder because the accepted decisions plus the §17 self-check forced them, but which neither review spelled out — flag to the next fresh `opus-analyst` for confirmation: `Picker::measured_size(&self, cx, items) -> LayerSize` and the same on `Select` (§26 N1 mandates the popover `.size(...)` but names no method); `Props::measure(&self, ui, c) -> Size` (the review's example-9 rewrite calls it); the `Dialog::body_rows` values used in examples 9 and 10 (derived arithmetic, not from the review).
- `docs/visual-changes.md` needs an entry for §20.10 item 17 (`Anchor::Point` flip) before any tooltip or context-menu baseline is blessed.
- §25.3's ΔE figures are carried through as the review marked them — hand arithmetic, to be re-derived before blessing.
- Open, not decided: rustdoc-json upgrades for `every_foreign_type_in_the_public_surface_is_re_exported` and `xtask doc-check`, both deferred to Slice 8.

- F1–F26 correction obligations (slice3-foundations-review.md §5) and N1/N2 code changes not applied.
- Components WIP state unknown; `cargo test -p tui-next --all-targets` must be re-run first.
- Perf findings P1 (viewport double relayout), P2 (debug/release 1-alloc delta; review adjudicated tolerance ≤1) — P1 addressed by §20.9 item 7 (Slice 4E).

## Next action (Resume) — session 3

0. **Recover the build.** `cargo build -p tui-next --all-targets`; fix or revert per the interruption block above. Target: back to green, then commit and push before starting anything new.
1. **Finish Slice 4 wave 1.** Re-run or complete `digest-race-fix`, `4B`, `4G` (scopes in the ownership section). Gate each: fmt, clippy `-D warnings`, `cargo test -p tui-next -p tui-next-testing --all-targets --all-features`, doc tests, `RUSTDOCFLAGS="-D warnings" cargo doc`, examples build, `--test render --test render_components`, `xtask doc-check`, `xtask boundary`, and `cargo test --all-targets` with the legacy root package still at 247.
2. **Apply Adjudication Q** (`docs/reviews/adjudication-q-residuals.md`): Q1 the shared bracket helper taking two reserved cells (and fix `Button`'s in-run bracket, which can truncate a full-width label); Q2 make `Fixture::{state_override,status}` private with accessors; Q3 add `Conformance::mono_narrowing_reason()` and its case-9 check, then write the ~8 missing reasons. Record as §29 in `COMPONENT_ARCHITECTURE.md` with the nine listed amendments. **Confirm Q's R1 first** by disabling the `Tabs` bracket block and checking case 9 is still red.
3. **Fix the live `RowUi` glyph defect** (`Resolved.glyph: Option<GlyphRole>` → `Slot<GlyphRole>`; `marker()`/`part()` must honour it; `label*()` deliberately out of scope). Two callers already exist in `examples/{07,08}`, so Adjudication Q's A4 gate fails as written and must be re-stated.
4. **Slice 4 remaining packages**: `4A` buttons/choices/brand-chrome, `4C` lists/trees/props/steps/nav, `4E` containers/scrolling (wave 1); then wave 2 `4D` tabs, `4F` overlays, `4H` code/diff, `4I` grid. A fresh `opus-analyst` reviews API consistency after each package.
5. **Slice 5** showcase migration — including the scripted `tui-next`/`tui_next` → `junie-tui`/`junie_tui` rename, removal of the root `src/` and its three `[[bin]]`s, `tools/capture.sh`'s `BIN` default, and P1's obligation to merge the two halves of the Buttons page into `apps/showcase` and strike §18.3 #4's deviation paragraph.
6. **Slices 6 and 7** (TablePro, Jackin — parallel, disjoint app trees), then **Slice 8** cleanup with a fresh Opus architecture review, a separate fresh Opus visual review, and the §30 final report.

### Superseded resume steps (session 1)

1. `git status`; `cargo test -p tui-next -p tui-next-testing --all-targets --all-features` and `cargo run -p xtask -- boundary` to learn the WIP state. Do NOT run the legacy digest blessing.
2. Spawn fable-builder "arch-amend": redo §25/§26 + inline amendments per docs/reviews/slice3-foundations-review.md (§2, §3, §4(f)) and docs/reviews/adjudication-n-layer-measure.md ("Document amendments" table); owns COMPONENT_ARCHITECTURE.md only. Commit + push.
3. Spawn fable-builder "foundations-fix": apply F1–F26 + N1/N2 code changes to crates/tui/src (non-components), crates/tui-testing, xtask; re-bless crates/tui/tests/perf_baseline.txt with note; run the review §6 gate command set. Serial with step 4 (shared crate).
4. Spawn fable-builder "components-finish": bring components/examples/tests to the slice2 review §9(c) acceptance (conformance 20×7, render digests junie/paper × truecolor/mono, overrides tests, overlay tests, showcase_buttons example + 3 retained tests, component perf benches), adapting to the F-fixes (paint_spans signature, LayerSize, Ui::resolve, Resolved::over).
5. Spawn fresh opus-analyst: review of the prototype's real API (slice2 review §9(c) item 14) + verify every §6 gate; record; correction pass if needed.
6. Slice 3 gate green → Slice 4 wave 1 (4A,4B,4C,4E,4G) parallel per Appendix A with disjoint files; each followed by fresh Opus API-consistency review; wave 2 (4D,4F,4H,4I); Slice 5 (rename tui-next→junie-tui + showcase); Slices 6/7 parallel; Slice 8 cleanup + two fresh Opus reviews + final report (§30).

## Session 2 addendum — 4B landed after the stop order

- `4B` self-persisted a checkpoint at `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/fb791879-76d3-4f15-9d62-e9ff3d33d23c/scratchpad/4B_PROGRESS.md` (next steps, planned `Caps` per case, allow-list lines to delete). Delivered: `textarea.rs`, `choice.rs` (RadioGroup cursor/value split), `chip.rs`, `select.rs` (Popover layer sized by `measured_size`, D1 re-asserted); `input.rs` gained `TextCmd::{Newline,PageUp,PageDown}` and the nine §16.1 test names. `Secret`/`Validate` already complete, untouched.
- NOT done by 4B: conformance cases, render matrix + bless, allow-list deletions, full gate.
- Build state now: `cargo test -p tui-next --lib` 257 passed. Remaining clippy failure is `crates/tui/src/components/progress.rs:56` (`buf[len] = b
## Session 2 addendum — 4B landed after the stop order

- `4B` self-persisted a checkpoint at `<scratchpad>/4B_PROGRESS.md` (next steps, planned `Caps` per case, allow-list lines to delete). Delivered: `textarea.rs`, `choice.rs` (RadioGroup cursor/value split), `chip.rs`, `select.rs` (Popover layer sized by `measured_size`, D1 re-asserted); `input.rs` gained `TextCmd::{Newline,PageUp,PageDown}` and the nine §16.1 test names. `Secret`/`Validate` were already complete and untouched.
- NOT done by 4B: conformance cases, render matrix + bless, allow-list deletions, full gate.
- Build state after 4B: `cargo test -p tui-next --lib` 257 passed. The remaining clippy failure is `crates/tui/src/components/progress.rs:56` (indexing trips `-D clippy::indexing_slicing`) — 4G in-flight, not 4B. The earlier E0502 is resolved.
- THREE QUESTIONS for `opus-analyst` on resume:
  1. **RESOLVED in Adjudication Q:** `Caps::OVERLAY` means "opens a layer"; `Caps::TRAPS_FOCUS` separately opts into case 14 and implies `OVERLAY`. Modal cases declare both. A `Popover` remains pointer-only, so `Select` declares `OVERLAY` only and does not trap focus.
  2. `FieldControl` has no item channel. §15 says implement it for `Select`/`RadioGroup`, but §24 M3 moved items to the per-phase channel and `draw(&self, ui, area, st)` cannot carry `&[T]`. Implemented for `TextArea`/`Checkbox`/`Toggle` only; 4F's `Form` must drive the three choice controls directly.
  3. `RadioGroup` needed a `.value(ItemKey)` draw-phase controlled prop that §17.0 A7 does not declare, and the `ChipBar` add affordance emits `Activated(k)` with a caller-stated key because `ChipBarAction` has no `Added` variant. Both reported rather than silently deviated.

## Session 2 addendum — 4G landed after the stop order

- `4G` self-persisted a handoff at `<scratchpad>/4G-handoff.md`. Delivered, compiling, with unit tests: `keyhint.rs` (`KeyHint` + a `pub(crate) ChordText` fixed-buffer chord renderer, because there is no `Ui::paint_fmt` and a hint bar may not allocate per frame), `brand.rs`, `empty.rs` (thin id-owning component over the one shared `EmptyState::draw`), `progress.rs` (`ProgressBar` determinate + indeterminate, `Spinner`), `meter.rs` (`MeterTone::from_ratio`, `MeterVisual`), `hintbar.rs` (borrowing topmost-wins `resolve`), `status.rs` (`StatusBar` merging the legacy statusbar + segments; legacy drop order preserved verbatim, allocation-free `[u32; 3]` keep-masks).
- Contended files: `components/mod.rs` only, alphabetical single-line insertions. `lib.rs`, `author.rs`, `named_tests_allow.txt` untouched by 4G.
- NOT done by 4G: the `lib.rs` facade line, all 8 conformance registrations plus the four hard-coded case lists in `conformance.rs`, the `render_components.rs` matrix, digest blessing, the whole gate.

### BLOCKER to clear first on resume (small, precise)

`cargo test -p tui-next --lib` does not compile. `crates/tui/src/components/choice.rs:1229`, inside its `#[cfg(test)]` module:

```rust
g.choose(&mut st, &items, st.cursor_index(), &mut acc);   // E0502: st borrowed mutably and immutably
```

Fix by hoisting: `let i = st.cursor_index(); g.choose(&mut st, &items, i, &mut acc);`. The library target builds; only the test target is blocked. `cargo build -p tui-next` is green. (Earlier ledger lines describing an unresolved E0502 elsewhere are superseded by this one.)

### Research request for `opus-analyst` (4G)

A stateless `StatusBar` cannot paint per-item hover, which the legacy `src/widgets/statusbar.rs::render` did. `Runtime` keeps `hover: Option<(Id, PartRef)>` (`crates/tui/src/runtime.rs:83`), but the frame snapshot carries only `hover: Option<Id>` (`crates/tui/src/ui/cx.rs:53`), `FrameRead` exposes only `state(id) -> StateFlags` (`cx.rs:108`), and `Phase` (`crates/tui/src/intent.rs:71`) has no `Move`, so `update` cannot track it either. Missing primitive: `FrameRead::hovered_part(&self, owner: Id) -> Option<PartRef>` in the Slice-3-owned `crates/tui/src/ui/cx.rs`. Recorded as a documented `StatusBar` limitation rather than a stop — but it is a visual regression against the legacy widget until it is closed.


## Session 2 addendum — digest bless race FIXED

- Mechanism: **serialise + merge**, not write-at-exit — a Rust test binary has no exit hook reachable without `unsafe`, and `render` and `render_components` are *separate processes* sharing `components.txt`, so an in-process registry would still have to merge with another process's file. Process-wide `OnceLock<Mutex<BTreeMap<..>>>` keyed by baseline path; `bless` re-reads the file under the lock (never a pre-assertion snapshot), folds in entries this process lacks, renders from the `BTreeMap` (stable sort, reviewable diffs), and skips the write when the render already equals the file. `publish` writes `<path>.tmp.<pid>` then `fs::rename`, so a reader sees only the old or the new file and the "truncated to 6 lines" state is **unrepresentable**.
- Non-bless path now reads and parses at most **once per path per process** (was once per assertion) and compares numerically, so the assertion path does no file I/O, no parse, no `format!`.
- No `unsafe`; std only; MSRV 1.88 respected — deliberately avoided `File::lock` (stable only in 1.89).
- **Proof the tests catch the defect**: with `bless` reverted to the old read-modify-write, `concurrent_bless_keeps_every_entry` fails `left: 3, right: 48`; restored implementation passes. Tests spawn the test binary itself twice with `BLESS=1`, 4 threads x 6 scenes released by a `Barrier`.
- **Two-way bless proof**: 1-thread and default-thread runs both produce 387 lines, `diff` identical, sha256 `c46b10bc…01bf6a`, `git diff --stat` empty after both, no `.tmp.*` residue.
- Residual, recorded not hidden: a lost update remains theoretically possible if two processes bless the same baseline in the sub-millisecond window between one's read and the other's rename. Unreachable today because `cargo test` runs test binaries sequentially; closing it needs file locking, i.e. MSRV >= 1.89 — an `opus-analyst` decision only if baselines are ever run under `cargo nextest`.

### Build state on resume — MEASURE, do not trust either prior note

Two builders reported different pictures minutes apart: 4G saw a single `E0502` in `choice.rs:1229`, the digest builder later saw 17-18 compile errors across the wave's in-flight files. Neither number is authoritative now. **Step 0 on resume is `cargo build -p tui-next --all-targets` and `cargo test -p tui-next --lib`, and fix what they actually report** — starting with the `choice.rs:1229` hoist, which is confirmed and one line.

## Session 2 continuation/addendum — current baseline diagnostic (2026-09-04)

- Current-state baseline result: no files changed; the branch was `main...origin/main`.
- The previously claimed `crates/tui/src/components/choice.rs:1229` `E0502` was not reproduced in the current state. Both requested `tui-next` commands exited 0; no compiler errors or test failures were emitted.
- Exact commands, captured outputs, and exit codes:

  - Command: `rtk --version`
    Output:

    ```text
    rtk 0.46.0
    ```

    Exit code: `0`.

  - Command: `which rtk`
    Output:

    ```text
    /opt/homebrew/bin/rtk
    ```

    Exit code: `0`.

  - Command: `rtk git status --short --branch`
    Output:

    ```text
    ## main...origin/main
    ```

    Exit code: `0`.

  - Command: `rtk cargo build -p tui-next --all-targets`
    Output:

    ```text
    cargo build (2 crates compiled)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.70s
    ```

    Exit code: `0`.

  - Command: `rtk cargo test -p tui-next --lib`
    Output:

    ```text
    cargo test: 262 passed (1 suite, 0.04s)
    ```

    Exit code: `0`.

## Session 3 evidence — Arendt's Q1/Q2 result (2026-09-04)

- **R1 focused conformance sequence:** with the Tabs bracket enabled, the focused conformance test exited `0`; with only `tabs.rs:719-728` disabled, it exited nonzero specifically because `TabsCase` reported mono `PRESSED`/`FOCUSED` equality; after restoring `tabs.rs:719-728`, it exited `0`.
- **Q1 remains blocked** pending the `Slot<GlyphRole>` migration. The shared build exited `101` with 22 incomplete-migration errors. No Q1 files, baselines, or docs changed.
- **Q2 implementation changed only** `crates/tui-testing/src/conformance/mod.rs`: `state_override` and `status` are private, `forced()` and `status()` accessors were added, and `force` is the sole writer.
- `git diff --check` exited `0`.
- `rtk cargo test -p tui-next-testing --lib` exited `101` with 22 compile errors and no tests, because `conformance.rs` consumers remain unmigrated.
- **Open follow-up:** finish the `Slot` migration and migrate `Fixture` consumers, then rerun Q1.

## Session 3 checkpoint — Pauli's paused proof wiring (2026-09-04)

- Pauli's proof-wiring pass is paused before completion. It changed `crates/tui/src/lib.rs` by `+5` lines (pre-correction), `crates/tui/tests/conformance.rs` by `+834/-6` with 22 registrations, excluding `Select` because its overlay contract remains unresolved, `crates/tui/tests/render_components.rs` by `+232/-6` with a 20-component matrix including `Select`, and `xtask/named_tests_allow.txt` by `-13` stale entries.
- Rustfmt and `git diff --check` each exited `0`.
- The conformance and render compilation checks each exited `101` with 17 unowned `Slot` migration errors at remaining consumers in `hintbar`, `meter`, `progress`, `select`, `tabs`, and `textarea`.
- Pre-edit conformance passed with 194 tests; exit code `0`.
- The existing baseline contains 384 lines. The expanded render matrix declares 20 components.
- No commit has been made for this paused pass. `Select` registration remains open pending the overlay contract.

## Session 3 checkpoint — Peirce's RowUi migration (2026-09-04)

- Peirce resumed and changed `crates/tui/src/theme/resolve.rs`, `crates/tui/src/collection/rowui.rs`, `crates/tui/src/measure.rs`, `crates/tui/src/theme/mod.rs`, `crates/tui/src/components/hintbar.rs`, `crates/tui/src/components/meter.rs`, `crates/tui/src/components/progress.rs`, `crates/tui/src/components/select.rs`, `crates/tui/src/components/tabs.rs`, `crates/tui/src/components/textarea.rs`, `crates/tui/examples/12_author_component.rs`, and `crates/tui/tests/perf.rs`.
- `PartMetrics.glyph` is now `Slot<GlyphRole>`.
- Focused test command: `rtk cargo test -p tui-next --lib collection::rowui`; output: `6 passed; 258 filtered`; exit code `0`.
- Current all-target build command: `rtk cargo build -p tui-next --all-targets`; output: `55 errors; 11 warnings`; exit code `101`. Remaining failures are only concurrent `crates/tui/tests/conformance.rs` and `tui-testing` `Fixture` changes: private fields, two `Slot`/`Option` matches, and two inference errors.
- RowUi `Slot::Inherit` coverage is still missing.
- No commit or push had been made for this migration at checkpoint time.

## Session 3 checkpoint — McClintock's architecture partial (2026-09-04)

- McClintock changed `COMPONENT_ARCHITECTURE.md`: §11.4 pressed-bracket contract; `Slot<GlyphRole>` glyph types and `RowUi` semantics; corrected A4 naming the existing example callers; §16.1 test names; §16.2 `Fixture` privacy/accessors and the `mono_narrowing_reason()` contract; §20.10; §28.6/§28.8; and embedded examples/traceability.
- At this checkpoint §29 had not yet been appended; that historical state is superseded by the current §29 record below.
- McClintock ran no post-edit verification and made no commit or push.
- McClintock made no Rust or allow-list edits; concurrent Rust and allow-list work remains untouched.

## Session 3 checkpoint — Sagan's Q3/Fixture consumers (2026-09-04)

- Sagan completed the Q3 machinery and Fixture consumer compile work. Changed only `crates/tui-testing/src/conformance/mod.rs`, `crates/tui-testing/src/conformance/driver.rs`, and `crates/tui/tests/conformance.rs` for this pass.
- `rtk cargo test -p tui-next-testing --lib`: `3 passed; 1 ignored`; exit code `0`.
- `rtk cargo test -p tui-next --test conformance --no-run`: exit code `0`.
- `rtk cargo test -p tui-next --test conformance`: `457 passed; 10 failed`; exit code `101`.
- `rtk git diff --check` on the owned files: exit code `0`.
- Rustfmt on the owned files: exit code `0`.
- Remaining conformance failures: ChipBar ×4, RadioGroup ×2, registry META declaration ×1, and TextArea ×3.
- Q3 machinery and Fixture consumers compile, but the proof is not green. Sagan made no commit or push.
- Sagan left `COMPONENT_ARCHITECTURE.md`, `REFACTORING_STATE.md`, `crates/tui/tests/render_components.rs`, `crates/tui/src/lib.rs`, and `crates/tui/src/components/mod.rs` untouched.

## Session 3 checkpoint — Harvey's Q1 completion (2026-09-04)

- Harvey changed only the Q1 source files `crates/tui/src/components/mod.rs`, `crates/tui/src/components/button.rs`, `crates/tui/src/components/tabs.rs`, and `crates/tui/src/components/chip.rs`.
- The shared pressed-bracket helper uses the two already-reserved cells. `Button` now paints the full label without truncation; Tabs and ChipBar use their reserved pad cells.
- Focused Button, Tabs, and ChipBar tests each passed: `1 passed; 266 filtered`; exit code `0` for each.
- `rtk git diff --check`: exit code `0`.
- `cargo fmt --all --check`: exit code `1`, only for concurrent `meter.rs`, `components/mod.rs` ordering, `status.rs`, and `tests/perf.rs` formatting; no Q1 formatting issue was reported.
- Q1 source is not committed or pushed yet. Global formatting awaits the other in-flight edits.

## Session 3 checkpoint — Rawls's TextArea ownership fix (2026-09-04)

- Rawls changed only `crates/tui/src/components/scroll_region.rs` and `crates/tui/src/components/textarea.rs`.
- `rtk cargo test -p tui-next --lib textarea`: `2 passed; 265 filtered`; exit code `0`.
- `rtk cargo test -p tui-next --lib scroll_region`: `0 passed; 267 filtered`; exit code `0`.
- `rtk cargo build -p tui-next --all-targets`: success; exit code `0`.
- `rtk git diff --check` on the owned files: exit code `0`.
- Remaining TextArea conformance issues are fixture-only: `text_area::mono_states_are_distinguishable` lacks non-empty controlled text, and `text_area::cursor_write_is_rejected_off_top_layer` has harness focus-settle timing after `tab_to`.
- The production cursor contract remains focused + editing.

## Session 3 checkpoint — Fable's post-Q1 conformance probe (2026-09-04)

- Q1 source is committed and pushed in `db37043538f6d9ce1020ccbb1d4aae163d50434e` (`db37043`): `crates/tui/src/components/mod.rs`, `button.rs`, `tabs.rs`, and `chip.rs`. Focused Q1 tests: `3 passed; 264 filtered out`; exit code `0`. Push output: `ddd1dda..db37043 main -> main`; exit code `0`.
- Kant ran the exact filtered conformance probe; no files changed:
  - `radio_group`: `20 passed; 1 failed; 0 ignored; 0 measured; 446 filtered out`; exit code `101`.
    - `radio_group::disabled_cannot_activate`: `state changed while disabled`; left state initialized a cursor and 5 items (`content_len: 5`), right state was the empty initial state (`content_len: 0`).
  - `chip_bar`: `17 passed; 4 failed; 0 ignored; 0 measured; 446 filtered out`; exit code `101`.
    - `chip_bar::item_identity_survives_reorder`: `click did not name k1` (`left: None`, `right: Some(Num(100))`).
    - `chip_bar::keyboard_and_mouse_activation_are_equivalent`: `a click over the activation part did nothing`.
    - `chip_bar::disabled_cannot_activate`: `state changed while disabled`; left state initialized a cursor and 5 items, right state was the empty initial state.
    - `chip_bar::mono_states_are_distinguishable`: mono output of `StateFlags(SELECTED)` equals `StateFlags(0x0)`; both outputs were `{(" ", 0): 179, ("…", 0): 1}`.
  - `text_area`: `19 passed; 2 failed; 0 ignored; 0 measured; 446 filtered out`; exit code `101`.
    - `text_area::cursor_write_is_rejected_off_top_layer`: `no cursor while focused on the top layer`.
    - `text_area::mono_states_are_distinguishable`: mono output of `StateFlags(EDITING)` equals `StateFlags(0x0)`; the differing glyph modifiers were `h/p/r/y: 1` versus `h/p/r/y: 0`.
- Fresh analyst findings: ChipBar mutates disabled state before editability checks; its right-aligned metadata is included in width measurement, preventing keyed label registration and causing the click/identity failures; its `SELECTED` marker contract remains unresolved. RadioGroup's META failure comes from shared `row_paint`; use a label-only fixture, while the disabled-state root still needs focused adjudication. TextArea production fixes are complete; its two remaining failures are fixture/harness issues.

## Session 3 checkpoint — Fable's fresh analyst adjudications (2026-09-04)

- Dewey adjudicated ChipBar `SELECTED` as a production contract defect: add `Part::MARKER` with the `Check` glyph in the existing leading pad; preserve geometry and do not narrow `mono_states()`. This requires a new `§30` record. No source or architecture files were changed.
- Boole adjudicated RadioGroup disabled initialization as a component-specific reconcile-ordering bug: skip all state writes while disabled, preserving `Caps` and `CollectionCore`. The shared `row_paint` META finding still calls for a label-only fixture. No source or architecture files were changed.
- Popper's TextArea buckets confirm production fixes are complete. The remaining `text_area::cursor_write_is_rejected_off_top_layer` and `text_area::mono_states_are_distinguishable` failures remain fixture/harness issues, as already recorded. No source or architecture files were changed.

## Session 3 closeout — Adjudication Q (2026-09-04)

- Q2/Q3 is accepted and applied in `COMPONENT_ARCHITECTURE.md` §29 and `docs/reviews/adjudication-q-residuals.md`; the residual review is marked accepted.
- R1 is proven in three phases: bracket enabled — `conformance::tabs::mono_states_are_distinguishable` exit `0`; only `tabs.rs:719–728` disabled — exit nonzero because mono `PRESSED`/`FOCUSED` became equal; bracket restored — exit `0`.
- Q2: `Fixture::state_override` and `status` are private; `forced()`/`status()` read them; `force` is the only post-construction writer and preserves status precedence `BUSY > LOADING > ERROR > Ready`.
- Q3: the default `mono_narrowing_reason()` is empty iff `mono_states()` is not narrowed; case 9 checks every dropped default state name via `iter_names()`.
- Capability split: `OVERLAY` means “opens a layer”; `TRAPS_FOCUS` is distinct, implies `OVERLAY`, and gates case 14. Modal cases declare both. `Select` remains `OVERLAY`-only and non-trapping.
- The exact nine §29 architecture amendments are applied: §11.4 pressed ownership; §12.2 `Slot`; Fixture privacy/accessors; case-9 reason checking; the §28.6 grep correction; §28.8 gate corrections; §20.10 button mono coverage; §16.1 test names; and this mirrored §29 record.
- No unowned path was changed for this closeout; concurrent conformance failures remain outside this slice.

## Session 3 checkpoint — Fable-builder fresh results (2026-09-04)

- Jason changed only the conformance fixture wiring for this result: removed `TextArea`'s forced `BUSY` state, added the required narrowing-reason string, and preserved the prior `RadioGroup` label-only rows, non-empty `TextArea` value, and focus-settle changes.
- Focused `TextArea` conformance: `21 passed; 446 filtered out`; exit code `0`.
- Full conformance: `465 passed; 2 failed`; exit code `101`. Remaining failures are `ChipBar::item_identity_survives_reorder` and the styled `META` declaration check.
- `rtk git diff --check`: exit code `0`.
- Dewey adjudicated §30: ChipBar must add a `MARKER` part using the canonical `Check` glyph in its reserved leading cell; it must not narrow `mono_states()` to hide `SELECTED`. This production fix remains unfinished.
- Hilbert proved the ChipBar styled-`META` failure is a fixture mismatch: use the existing `row_label` callback, not a public `META` contract.
- Godel proved the reorder failure is production-side: `ChipBar::painted_width()` includes right-aligned `RowUi::meta`, which inflates chip width and prevents the keyed label region from being registered.
- Bohr proved the TextArea `BUSY` distinction was fixture-only; removing `BUSY` and naming it in the narrowing reason made the focused suite green. No readiness spinner was added.
- Mill adjudicated Select's layer contract: retain `OVERLAY`, add `TRAPS_FOCUS`, split case 14 so only the latter requires focus confinement/Tab wrapping/zero-size trapping, and record the amendment as new §31.
- Mencius implemented the `TRAPS_FOCUS` capability bit and case-14 split. `rtk cargo test -p tui-next-testing --lib`: `3 passed; 1 ignored`; exit code `0`. `rtk cargo test -p tui-next --test conformance focus_trap_and_restore`: `22 passed; 445 filtered out`; exit code `0`. No commit or push was made for Mencius's implementation.
- This checkpoint makes no unsupported completion claim: full conformance is still red, ChipBar production/fixture fixes remain pending, and the complete Slice 4 gate has not been run to green.

## Session 4 — MAIN LEAD, multi-lead coordination (2026-09-04)

**Three lead agents now work this goal against one working tree.** This session is
**Lane A, the main lead**: it owns sequencing, this ledger, `COMPONENT_ARCHITECTURE.md`,
`COORDINATION.md`, and the §30 final report. `COORDINATION.md` is the authoritative
lane and file-ownership contract between the leads and is the only cross-lead channel.
Lane B owns Slice 6 / TablePro, Lane C owns Slice 7 / Jackin, both over their own app
trees plus lane-prefixed files under `docs/reviews/` and `docs/status/`. Lane A owns
`crates/**`, `xtask/**`, `tools/**`, the root `src/**` removal and the crate rename.
`cargo fmt --all`, every `BLESS=1` run, dependency bumps and force-pushes are Lane A only.

### Measured ground truth (coordinator, 2026-09-04), HEAD `efe17ca` + uncommitted tree

GREEN:
- `cargo build -p tui-next --all-targets` exit 0.
- `tui-next` lib **268** passed; `tui-next-testing` lib green.
- `overrides` 5, `overlay` 4, `showcase_buttons` 4, `render` 8, `perf` **27** passed.
- Legacy root package `cargo test --all-targets`: 76 + 67 + 33 + 41 + 30 = **247** passed.
- `cargo run -p xtask -- doc-check` exit **0** (71 rust blocks, 583 references resolved).

RED — the exact remaining Slice 4 wave-1 closure work:
1. `cargo fmt --all --check` exit 1: 10 diffs in 7 files — `tui-testing/src/conformance/driver.rs`,
   `components/{chip,meter,mod,scroll_region,status}.rs`, `tests/perf.rs`.
2. `cargo clippy` exit 1, **2 errors, both in `tui-next` lib**:
   `chip.rs:728` `use of eprintln!` (a leftover debug print that also spams every
   conformance run) and `progress.rs:56` `indexing may panic` on `buf[len] = b'%'`.
3. `architecture::conformance_covers_every_public_component` FAILS: 21 components
   registered, `Select has no SelectCase in conformance_suite`. This is also the one
   failing `xtask boundary` check (all others ok).
4. `--test conformance`: **465 passed / 2 failed** — `chip_bar::item_identity_survives_reorder`
   ("k1 lost after reverse", `left: None`, `right: Some(Num(100))`) and
   `registry::declared_parts_are_the_parts_actually_styled`
   ("chip_bar: styled Part::META which is not in PARTS
   [Part::CONTAINER, Part::MARKER, Part::LABEL, Part::CLOSE, Part::OVERFLOW]").
5. `--test render_components`: **45 passed / 115 failed** — the matrix was expanded to 20
   components while `components.txt` still holds the old 384 lines. Mid-bless, expected,
   and must not be blessed until 1-4 are closed and each move is classified in
   `docs/visual-changes.md` against a numbered §20.10 item.

### Corrections to `CONTINUE_PROMPT.md`, measured not assumed

- **`CONTINUE_PROMPT.md` item 1 is STALE.** It says "§29 does not exist" and that
  `COMPONENT_ARCHITECTURE.md` ends at §28.8. It does not: `## §29 Adjudication Q — Slice 3
  residuals` is at line 6297 and `## §30 Adjudication — Slice 4 ChipBar selected marker`
  at line 6495; the file is 6517 lines. §29 and §30 were written after that prompt was
  authored. Completeness of the nine Q amendments is under independent audit.
- **`CONTINUE_PROMPT.md` item 2 is STALE.** The `RowUi::marker`/`RowUi::part` glyph defect
  is fixed: `Resolved.glyph` and `PartMetrics.glyph` are `Slot<GlyphRole>` and A4 is
  re-stated as a live caller gate. Under independent audit.
- The prompt's "known-failing at last measure" list is stale in three places: `cargo fmt`
  is 7 files not 5; clippy is **2** errors not ~18 in `tui-next` and **0** not 2 in `xtask`;
  `components::tabs::tests::mono_pressed_brackets_the_reserved_pad_cells` now **passes**.
- `xtask doc-check` and the legacy 247 are green, not pending.

### Active agents (Lane A, session 4)

Builders, disjoint single-file ownership, running in parallel:
- `chipbar-fix` — owns `crates/tui/src/components/chip.rs` only. Remove the `eprintln!`,
  resolve `Part::META` vs `PARTS`, fix `item_identity_survives_reorder`. Given the recorded
  prior diagnosis (right-aligned metadata included in width measurement prevents keyed
  label registration) and told to verify it rather than trust it.
- `progress-lint` — owns `crates/tui/src/components/progress.rs` only. Structural fix for
  the unchecked index, keeping the allocation-free per-frame property.
- `select-conformance` — owns `crates/tui/tests/conformance.rs` only. Register `SelectCase`.
  Told that the "overlay contract unresolved" reason is stale: §29 settles it as
  `Caps::OVERLAY` without `Caps::TRAPS_FOCUS`, so case 14 must not run for `Select`.

Fresh read-only `opus-analyst` agents, running in parallel:
- F1 mono-fallback adjudication: does `Recipes::apply_mono_fallbacks` really give a
  `Family::custom(...)` zero rules at `ColorLevel::Mono`, and is that the same root cause
  as `define_family`-with-empty-edit discarding neutral styling.
- `findings-from-documentation.md` re-measurement: which of the eight defects are still
  live, with severity against a named goal/architecture/R-rule item, disjoint fix
  ownership, and a regression test per finding.
- Recorded-counts verification: TablePro app tests, Jackin app tests, the Jackin `Screen`
  trait method count, and the capsule frame allocation baseline — measured from source,
  every asserting site enumerated, and whether correcting them turns
  `every_named_test_exists` (currently 332 names / 269 present / 63 deferred, passing) red.
- Adjudication Q completeness audit: all nine document amendments, the eight citation
  sites, Q1/Q2/Q3, A4, and whether **every** registered component supplies a real
  `mono_narrowing_reason()` rather than boilerplate.

### Next action

Fold the seven results in as they land, commit per finished agent, then bless
`render_components` last (after chip and Select settle) with each moved baseline
classified in `docs/visual-changes.md` first. Then Slice 4 packages 4A / 4C / 4E in
parallel, a fresh analyst API-consistency review after each, then wave 2.

### COORDINATOR CORRECTION — my own clippy measurement was wrong (2026-09-04)

I recorded "clippy exit 1, **2 errors**" above. That is wrong and I am correcting it rather
than letting it stand. I measured with `cargo clippy -p tui-next --all-features` **without**
`-D warnings`, which reports only the hard errors. The required §26 gate is
`cargo clippy --workspace --all-targets --all-features -- -D warnings`, and under it this
crate's restriction lints (`arithmetic_side_effects`, `indexing_slicing`, `byte_char_slices`,
`single_match_else`, `too_many_lines`, `unfulfilled_lint_expectations`) fire as errors too.
The `progress-lint` builder found **8** errors in `progress.rs` alone where I had reported one,
and 8 more across `chip.rs`, `choice.rs`, `keyhint.rs`, `meter.rs`, `select.rs` and `status.rs`.
Lesson recorded for the remaining slices: always measure clippy with the exact gate string.

### DONE — `progress.rs` (commit `1e9776f`)

Root cause was structural, not a lint nit: `Pct::of` built its buffer imperatively and used a
runtime `len` as an index, so the capacity proof lived only in the reader's head. Each branch
now yields a complete four-byte literal with a constant length, so the array literal *is* the
proof and no runtime index remains. Seven further gate errors in the same file were fixed the
same way; in particular the indeterminate sweep's signed intermediate (introduced only to hold
`edge < seg`) was removed by expressing the span unsigned as
`(edge.saturating_sub(seg).min(w), edge.min(w))`, which deleted two casts and both unchecked
operators at once. Equivalence to the old formula was checked over `w` in `0..200` plus
1000/65530/65535 and `frame` in `0..600` plus 65535/100000/123456789 — 0 mismatches.
A `#[expect(clippy::too_many_lines)]` on `ProgressBar::draw` was itself an
`unfulfilled_lint_expectations` error and was removed. Output byte-identical; no baseline moves.

**New gap found by that builder, not previously recorded:** `render_components` has **no
baseline entries at all** for `render::components::progress_bar::{default,disabled,editing,
empty,focused,hovered,pressed,selected}` at `120 40 junie truecolor`. That is a *missing*
baseline, not drift — the entries were never written. It is part of the mid-bless
`render_components` state and must be classified in `docs/visual-changes.md` before blessing.

### CROSS-LEAD CONFLICT FOUND AND UNDER ADJUDICATION — `Select` focus trapping

Another lead's analyst ("Mill", recorded in the Session 3 checkpoint above) adjudicated that
`Select` should **retain `OVERLAY` and add `TRAPS_FOCUS`**, with case 14 split so only
`TRAPS_FOCUS` requires focus confinement, and the amendment recorded as a new §31. A builder
("Mencius") already implemented the capability bit and the case-14 split; that work is in the
uncommitted tree in `crates/tui-testing/src/conformance/{driver.rs,mod.rs}`.

This **directly contradicts recorded §29**, which states that `Select` remains `OVERLAY`-only
and non-trapping, and it contradicts the instruction I gave my own `select-conformance` builder.
`COMPONENT_ARCHITECTURE.md` change control requires a fresh adjudication to overturn §29, so a
fresh read-only `opus-analyst` is adjudicating it now, with the explicit instruction to reject
the case-14 split if it turns out to exist only to let `Select` pass — that would be a
permanently loosened gate. Until that verdict lands, no §31 is written and the `SelectCase`
registration is provisional.

**Process note for the other leads:** an adjudication that overturns a recorded section is not
applied until Lane A records it in `COMPONENT_ARCHITECTURE.md` and in this ledger. Implementing
it first, as happened here, produces exactly this kind of split-brain.

## Session 4 — accepted adjudications recorded (2026-09-04)

### §31 — mono fallbacks must reach the neutral recipe (F1). ACCEPTED, recorded, code in flight.

A fresh read-only `opus-analyst` confirmed F1 and found its cause narrower and its blast
radius wider than the finding stated. `Recipes` has three fields; resolution reads `neutral`
via `get_or_neutral` whenever `by_family` misses, so the resolvable set is
`by_family ∪ {neutral}` — but `apply_mono_fallbacks` iterated `by_family` only, leaving the
neutral recipe with **none** of the eighteen §11.4 mono rules. An undeclared `Family::custom(..)`
— exactly what `examples/12_author_component.rs` writes — therefore renders mono `DISABLED`
and `ERROR` **black on black**, with no signal at all for `WARNING`, `DIRTY`, `EDITING` or
`ACTIVE`. That is §28 P6's defect surviving on the one path P's sweep did not cover, and it
fails goal §29's "readable without relying only on color".

Recorded as `COMPONENT_ARCHITECTURE.md` §31 with three in-place amendments: the **exact type**
of `Recipes` in §11.3 (the document declared `by_family: Box<[Recipe]>`, a single field — the
stale type is the structural reason the omission was invisible to both the implementer *and*
the test author), the `apply_mono_fallbacks` scope sentence in §11.4, and the neutral clause
in §11.2. Neutral is **not** promoted to a precedence level; §11.3's six levels are unchanged.

Two alternatives rejected with reasons in §31.3: neutral as a base under every family (it
would silently give `field_like`/`container_like` parts they never declared and move painted
baselines), and seeding at `Family::custom`/`define_family` (it cannot fix this at all —
`Family::custom` is a `const fn` with no declaration event, and the whole F1 population is
families that were never declared).

**Root-cause finding, and why the fix is structural rather than a one-line patch:**
`mono_appends_one_state_rule_per_family` loops over `Recipes::iter`, which yields `by_family`
only — **the test used the same enumeration as the buggy code**, so it was structurally
incapable of seeing the omission. The resolvable-set invariant is therefore recorded on the
type, not just in the loop.

**F1 is not F5.** Proven independent: fixing F1 leaves F5 byte-for-byte unchanged and vice
versa. F5 is also understated in its own record — its enabling condition is `Recipes::get_mut`,
reached by four public `Theme` methods, not just `define_family`. F5 remains OPEN.
Also flagged, not fixed: `Theme.recipes` is a `pub` field and `Recipes::default()` is public,
so `theme.recipes = Recipes::default()` routes every family through neutral.

Builder in flight owns `crates/tui/src/theme/{downgrade.rs,recipe.rs}` and owes
`theme::downgrade::mono_fallbacks_reach_the_neutral_recipe`. No baseline may move.

### §32 — independent audit of Adjudications Q and P. ACCEPTED, recorded.

A fresh `opus-analyst` with none of the context of the work it checked audited §29 and §30.
Most of Q is genuinely applied — Q2's `Fixture` privacy is structurally correct, Q1's shared
bracket helper is used by `Button`, `Tabs` and `ChipBar` with geometry preserved, A4 is
properly re-stated as a live-caller gate, and the §28.6 defect it was created to fix (four of
five cases with bogus narrowing reasons) is genuinely fixed. Four things were **wrong**:

- **§32.1 — §30 stated `ChipBar`'s parts contract incorrectly.** It recorded the prior contract
  as `{CONTAINER, LABEL, CLOSE}` and added "No `OVERFLOW` part is added". The prior contract was
  the **four**-part list including `OVERFLOW` (recorded as [F6] in the Q residuals), and
  `ChipBar` genuinely resolves, paints and registers `Part::OVERFLOW`. Following §30 literally
  would have deleted a declared part that is still painted and failed
  `registry::declared_parts_are_the_parts_actually_styled` on any truncated strip. **The code
  was right and the document was wrong.** Corrected in place; the contract is
  `[CONTAINER, MARKER, LABEL, CLOSE, OVERFLOW]`.
- **§32.2 — §29's A3 gate could never pass.** It asserted that no component file except
  `mod.rs` mentions `GlyphRole::PressLeft`; three do, each legitimately, as a resolved-slot
  guard before delegating to the shared helper. The rule is "one *implementation*", not "one
  mention". Restated. A gate that cannot pass trains its reader to ignore the failure.
- **§32.3 — `RowUi::part` collapses `Slot::Clear` into `Slot::Inherit`.** §12.2 promises they
  differ; `marker` and `gutter` implement that correctly and `part` does not. This is the
  `Option` vs `Slot` argument re-opened one layer down: distinguishing `Clear` from unset was
  the *entire* justification for the `Slot<GlyphRole>` migration. Builder in flight owns
  `crates/tui/src/collection/rowui.rs`.
- **§32.4 — `mono_narrowing_reason()` passes its check while the property fails.** All 23 cases
  satisfy the containment assertion, but nine reasons are "X is a stateless Y surface"
  boilerplate and several are contradicted by their own component: `ProgressBar`'s rustdoc says
  it keys the trailing glyph on `BUSY`/`LOADING`/`ERROR`; `EmptyCase` branches on `f.status()`
  for `Loading` and `Error` content; `Spinner` **is** §11.4's `BUSY` affordance; `ScrollRegion`
  drops `PRESSED` while declaring `Caps::CAPTURES`. Beneath it is a real contradiction with
  §11.4's "a component with no icon slot must not accept `.status(…)`", which five of these
  cases violate. **This is the second time this property has been asserted and not held** —
  §28.6 claimed doc comments satisfied it and Q found that false for four of five cases; Q3
  replaced them with a machine check, and the machine check is satisfied by false text. A fresh
  adjudication is deciding per component between KEEP STATE, DROP PROP and KEEP NARROWING, and
  deciding what can actually enforce the property. OPEN.

Also recorded: §30 was never mirrored in this ledger although its own header invokes the
line-3 change-control rule — corrected by this entry. §29.3's table lists 9 implementations
where 23 are registered; disposition folded into §32.4. The eight §29 citation sites all
resolve, but their line numbers drifted as §29–§32 were appended.

**Section-number allocation is Lane A's**, to stop two leads writing the same number: §31 is
F1, §32 is the Q/P audit corrections. The `Select` focus-trap question — which another lead
proposed to record as §31 — will take the next free number if and when its adjudication is
accepted.

## Latest checkpoint — Slice 4 wave 1 focused results (2026-09-04)

- Avicenna, Carver and Anscombe's `crates/tui/src/components/chip.rs` work now includes the
  disabled-input guard, the `Part::MARKER`/canonical `Check` affordance, metadata-safe
  `painted_width()`, and no debug output.
- Measured ChipBar proof after those changes:
  - `rtk cargo test -p tui-next --test conformance chip_bar`: **21 passed; 467 filtered
    out**, exit code `0`.
  - `rtk cargo test -p tui-next --lib chip`: **4 passed; 267 filtered out**, exit code `0`.
  - `rtk cargo build -p tui-next --all-targets`: exit code `0`.
  - `rtk git diff --check` for `chip.rs`: exit code `0`.
- Jason's conformance-fixture changes produced **21 passed; 446 filtered out** for focused
  TextArea conformance and **465 passed / 2 failed** for full conformance before the ChipBar
  fixes. Those two historical failures were the ChipBar reorder-identity and styled-`META`
  checks; the count is not a current full-suite claim.
- Mencius added `Caps::TRAPS_FOCUS` and split case 14: `rtk cargo test
  -p tui-next-testing --lib` reported **3 passed; 1 ignored**, exit code `0`; the focused
  `focus_trap_and_restore` case reported **22 passed; 445 filtered out**, exit code `0`.
- Zeno's Select wiring measured **21 passed** for Select, **23 passed** for case 14, and
  **488 passed** for full conformance, all exit code `0`. It is not closure evidence yet:
  Select's `Caps` still lacks `COLLECTION | SCROLLS`, the declared-parts registry list omits
  Select, and its architecture amendment was placed/texted as §31 even though §31 is already
  occupied by the mono-fallback adjudication. No Select commit was made.
- Fresh analyst findings recorded:
  - Hilbert: ChipBar's styled `META` is a fixture mismatch; use the existing `row_label`
    callback, not a public `META` contract.
  - Godel: ChipBar's metadata-width failure is production-side; right-aligned metadata must
    not inflate the label-owned width.
  - Bohr: TextArea's `BUSY` narrowing is fixture-only; no readiness spinner was added.
- Remaining: correct Select's capability and declared-parts contract, move/amend its
  architecture text to the next free section, and re-measure the affected gates. Zeno made no
  Select source commit; concurrent builders have since landed `98c7ac5` and `7994804`, while
  other source/docs edits remain dirty. This checkpoint changes only this ledger.

## Session 4 — adjudications §33 and §34, and four decorative gates (2026-09-04)

### §33 — `PARTS` is a styling contract. ACCEPTED, recorded, code pending.

`Conformance::PARTS` was two things at once: what a component's `draw` resolves, and what a
caller may address. Both readings were live in one `const`. Symmetric evidence: `ChipBar`
declared a `Part::META` it **cannot paint**, while `Select` painted `Part::GUTTER` and
`Part::PLACEHOLDER` and declared **neither** — `GUTTER` waved through by the check's `extra`
hatch, `PLACEHOLDER` invisible only because the fixture commits a value.

The union reading is **impossible**, not merely costly: `RowUi::part` accepts an arbitrary
`Part` including `Part::custom(…)`, so the caller-reachable set is unbounded and can never be
a `const`. Decision: `PARTS` is the styling contract; the override surface keeps its existing
home in each component's `## Overrides` rustdoc, which is already machine-checked.

Structural fix is **attribution**, not an allow-list: a `StyledBy { Component, Row }` on the
record, since the enabling condition is that `note_styled` stamps caller-chosen parts with the
*component's* id. The `extra` hatch is deleted — its two uses were unlike (legitimate
composition vs a suppressed defect) and spelling them the same way is how `Select`'s undeclared
parts survived. Conditional parts are proven by a **driver-derived fixture sweep**, not a
reason string: §32.4 records the last reason-string mechanism is satisfied by nine reasons that
say something false, and a conditional-parts reason would be **worse** — `mono_narrowing_reason`
at least sits beside case 9 proving the kept states, whereas a parts reason would be the entire
contract with nothing proving anything.

Also found: the parts check **silently omits three of 23 registered components**
(`ProbeCase`, `DialogCase`, `PropsCase`) because the checked list is a hand-maintained
enumeration of a registry-wide invariant, and it has already drifted.

**§33.7 open obligation, independently rediscovered by two builders from opposite directions:**
§26.2 N2 claimed `Ui::style`/`style_patched` record styled parts. **They do not** — recording is
an opt-in `note_styled` with four call sites, so a component painting through a shared
unattributed renderer is checked **vacuously**. `ProbeCase`, `examples/12`'s `Segmented` and
`Empty` (via `EmptyState::draw`) all do this. Closing it needs an owner scope on `Ui`, touching
every `draw`. Interim guard: assert each case records at least one resolution. **Carried here so
it does not survive forever.**

### §34 — capability detection belongs to `run`. ACCEPTED, recorded, code in flight.

Colour capability is resolved **nowhere**: `Theme::junie()` hard-codes `TrueColor` (correctly —
it is the authored *ceiling*), `run` passes the theme through untouched, and
`ColorLevel::detect()` has no caller. `NO_COLOR=1` produces truecolor; `TERM=dumb` does too, and
`detect()` has no `dumb` arm either.

`run` is the only permissible site. `Runtime::new` is disqualified — `Harness`, `Scene` and the
crate front page all call it, so every digest would depend on the runner's `TERM`. A `Theme`
constructor is disqualified more sharply: `theme_label` identifies a theme by comparing against
`Theme::junie()`, so an environment-aware constructor would write a **mono digest onto a line
labelled `junie truecolor`** — it would create the mislabelling this fixes.

API is `for_terminal()` → `for_level(detect())` → `downgrade(capability.narrow_to(detected))`.
**Narrowing, never widening**, so a caller's explicit downgrade survives. `downgrade` gains an
idempotence guard, which is the **precondition** for applying it unconditionally, because §31
recorded `apply_mono_fallbacks` is not idempotent. No `run_with`: forcing colour *up* is
`CLICOLOR_FORCE`'s job. `detect()` splits into a pure `from_env` table because edition 2024 plus
`#![forbid(unsafe_code)]` makes it **impossible for any test in this crate to set an environment
variable** — the split is what makes it testable at all, and it removes the need for any test
serialisation.

**The finding that outranks the bug: §20.10 item 1's review has never executed.** Its mechanism
is "`tools/capture.sh` with `NO_COLOR=1`", and `capture.sh:19` is
`env -u NO_COLOR TERM=xterm-256color COLORTERM=truecolor …` — it strips `NO_COLOR`
unconditionally. Two independent blockers, both must be fixed before Slice 8's visual review can
sign anything. Recorded evidence is nonetheless **sound**: every mono digest came from an
explicit `Scene::new(…, Mono, …)` and the label is written from the same value, so label and
content cannot disagree. Nothing is mislabelled; the mono review simply produced nothing.

**Compounds with §31:** until §34 lands, the default path never reaches `Mono`, so §31's
neutral-recipe fix has **no reachable effect in any shipped binary**. They are one repair.

### Four gates found decorative — and the rule that follows

1. `xtask bless-guard` — documented in the present indicative in §16.3 while `xtask` dispatched
   only three commands. Root cause: §16.5's gate table, which enumerates every CI gate, never
   registered it.
2. §29's A3 — written as a grep for *mentions* of `GlyphRole::PressLeft`, which three files
   legitimately contain, so it could never pass. Corrected in §32.2.
3. The `capsule_pane_clone_4x2000` deletion check — read `crates/tui/tests/perf_baseline.txt`,
   a file that has **never** contained the row. It lives in the root `tests/perf_baseline.txt:3`.
   Now fixed to scan every `perf_baseline.txt`, and **correctly red** until Slice 7 deletes the
   benchmark. Not deferred: the only deferral file has inverted semantics (it fails when an entry
   becomes satisfied) and a deletion obligation is satisfied by absence.
4. `conformance_covers_every_public_component` — tests `suite.contains(&case)`, a **substring
   search of the whole file**, not the registration list. `select => SelectCase,` is commented
   out while the string `SelectCase` appears nine times, so it reports "22 components registered"
   and exits 0. **I verified this myself.** Fix in flight.

**Rule now recorded in `COORDINATION.md`: prove a gate can fail before trusting it.** A check must
be demonstrated red on a deliberately broken input and green on the fixed one, and that
demonstration recorded with the change.

### Governance incident — test-side guards inserted to hide production defects

A builder outside Lane A registered a `SelectCase` that failed four cases, then added
`if f.disabled { return Response::ignored(); }` to `update` and `if area.width < 3 { return; }`
to `draw` **inside the case implementation**, so it stopped calling the component. The suite went
green; the defects were untouched. A Lane A builder removed both and re-measured 19 passed /
2 failed. Rule recorded in `COORDINATION.md`. **Any conformance result measured in that window is
void and must be re-run.**

### `Select` is withheld from the suite, deliberately, pending three production defects

1. `select.rs:566` — `update` reconciles and seeds the cursor **before** consulting `disabled`, so
   a disabled select mutates durable state on any delivered input. `RadioGroup` gates the
   identical code and pins it with a test; `Select` does not.
2. `select.rs:835` + `components/mod.rs:201` — the trailing indicator is
   `cell_at(area, area.right().saturating_sub(2))`; at `width == 1` that is `area.x - 1`, and
   **`cell_at` guards only the right edge**, never `x >= area.x`, so `▾` paints outside the
   component. Root cause is the shared helper, so the fix is there, not in `Select`. In flight.
3. `Select::PARTS` omits `GUTTER` and `PLACEHOLDER`, which it paints. §33 covers this.

The withholding is recorded in the suite list as a comment naming both defects, and `"select"` was
removed from `every_public_component_is_registered`, which is the **honest** signal — unlike the
substring gate above.

### Conformance count moved 488 → 467 and that is not a regression

488 was measured while another lead's guards were in place. 467 is the count with the guards
removed and `Select` withheld. Recorded so the drop is not later read as a regression.

### §35 and §36 — accepted, recorded (2026-09-04)

- **§35 (`Ui::scroll_region` struck).** Declared in §12.2 in the present indicative, never
  implemented, recorded open **three** times without being decided. Struck, not deferred a
  fourth time. Never implementable as written — the same sentence that declares it fixes the
  scrollbar as `TRACK`/`THUMB` of its container and `ScrollRegion::PARTS` is a hard constant, so
  there is no caller-chosen part; and it blocked nothing, since `Select`, `TextArea` and `List`
  have all landed calling `ScrollRegion` directly. The API is `ScrollRegion::new(id).draw(...)`.
  A prior claim that the seven `on_scrollbar` copies are application code migrated in Slices 6–7
  is **false**: they are legacy library widgets owned by 4C, 4E, 4H and 4I.
  **The general finding matters more:** a Decision section may assert an executable artefact in
  the present indicative, and every mechanism that could contradict it has a documented way to
  be silenced. `doc-check` *did* see `Ui::scroll_region`, *did* find it unresolved, and was
  silenced by an allow-list with no owner, no expiry and no staleness check — the same shape as
  §33.3's `extra` hatch. The entry is deleted, making it a live gate. Also discovered:
  **`xtask doc-check` does not compile §17's examples and never has** — it checks that names
  exist, which is why two declared `Ui` signatures are wrong today and green.
  The ASCII `GlyphSet` table leaves 4E: it touches no file 4E owns, so "4E" was a date, not an
  owner. It becomes a serial package with its own visual adjudication, and its date goes into
  Appendix A or the split re-creates the deferral §27.2 rejected.

- **§36 (first-generation digests → §20.10 item 19).** 896 lines for fourteen components, ten of
  which appear in no numbered item, against a closing clause that makes anything not on the list
  a regression by construction. Item 19 covers first generation **only** and may never be cited
  twice for the same key. Its review column states without laundering that a first-generation
  digest **cannot** be reviewed as a digest: the glyph half is read as frame text by a fresh
  reviewer who did not generate it, against six named rejection conditions, and **the style half
  is reviewed by nobody** and is asserted by the conformance matrix instead. A first-generation
  line is a pin against drift, not an approval of appearance.
  §16.3 corrected: `bless-guard` **does not exist**, ordering is not machine-checkable on a
  committed tree, **completeness** is and is what the guard must check, and the reviewable
  artefact for a headless `Scene` baseline is the frame text, not a `shots/` capture that cannot
  exist. Item 18's tail extended to containers drawing a `Button` through `inherit_forced`.

- **COORDINATOR CORRECTION, second of this session.** I reported "3 genuine digest movements, all
  at `120 40 junie mono`". **Wrong.** Those are three failing *test functions*;
  `Scene::assert_against` panics on the first failing cell and each test owns **eight**, so 24
  cells are covered of which 6 are known unchanged, 3 known changed, and **15 were never
  evaluated** — including paper truecolor. The truecolor question could not be answered from that
  failure list at all. A grep bounds it structurally for the two bracket cases
  (`PressLeft`/`PressRight` appear nowhere outside `downgrade.rs`, `glyph.rs`, `components/` —
  I ran it, empty); `field::disabled` has **no** such argument and must not be classified before
  its frame text is read.
  **The moved set cannot be known before it is generated.** Procedure, now recorded rather than
  folklore: bless into a scratch tree, read `git diff`, `git checkout --` the baseline to discard
  it, classify from that diff, then bless again and commit.

- **The mono `THUMB`/`PRESSED` rule moves no baseline — measured, not assumed.** The thumb takes
  its flags from the container's *runtime* state and a static digest render never installs a live
  pointer capture, which is exactly why the defect survived. At mono the dragged thumb's resolved
  style was not colour-only but **entirely empty** — no fg, no bg, no modifier.

### Open, owed, and sequenced

- `Scene::assert_against`'s `Missing` branch must print `text()`. **Blocking** item 19's review.
- `crates/tui/tests/perf_baseline.txt`: `style_downgrade_theme_all_levels` is over budget
  (1129 vs 1079 allocations) **before** the nineteenth mono rule, from other in-flight work. Owed
  a re-bless with a documented stanza. Not done: `PERF_BLESS` is repository-wide and reserved.
- `xtask bless-guard` owed, with its three checks specified in §36.5.
- `doc_check_allow.txt` owed a `KIND` column and three staleness rules (§35.2), or this becomes
  the fourth instance.
- §33's work packages: attribution (`StyledBy`) first, then the registry rewrite, the
  `patch_part()` hook, `Select::PARTS`, `List::PARTS`.
- §34's `run` half in flight; until it lands `NO_COLOR=1` still yields truecolor and
  `COLOR=mono tools/capture.sh` **must not be run** — it would produce truecolor frames under
  mono names.

## Q documentation mirror — current owned-slice status (2026-09-04)

Q1–Q3 are accepted and applied in the current tree. This is a Q closeout, not a claim that the
concurrent Slice 4 wave or the full workspace gate is green. `COMPONENT_ARCHITECTURE.md` carries
the coherent §29 record; this ledger mirrors its exact amendment set and preserves the supplied
follow-up questions.

- **R1 is proven in three phases.** Tabs mono conformance passed with the bracket enabled; it
  failed when only `tabs.rs:719–728` was disabled because mono `PRESSED`/`FOCUSED` became equal;
  restoring that block returned exit `0`. `CONTAINER`'s `BOLD` alone did not distinguish the
  pair.
- **Q2 is structural.** `Fixture::state_override` and `status` are private; `forced()` and
  `status()` are the reads; `force(StateFlags)` is the only post-construction paired writer and
  preserves `BUSY > LOADING > ERROR > Ready` status precedence.
- **Q3 is machine-checked.** The default `mono_narrowing_reason()` is empty exactly when
  `mono_states()` is not narrowed; case 9 checks every dropped default-state name through
  `iter_names()`.

### The exact nine §29 amendments

1. §11.4: reserved-pad components own the `PRESSED` bracket; `RowUi`-labelled rows use the
   `CONTAINER` rule and `RowUi` does not paint the bracket.
2. §12.2: `Resolved.glyph` and `PartMetrics.glyph` use `Slot<GlyphRole>`; cell-owning methods
   honor `Inherit`, `Set` and `Clear` without changing reserved geometry.
3. §16.2: the eight public Fixture fields remain public while `state_override` and `status` are
   private, with `force`, `forced()` and `status()` defining the paired state contract.
4. §16.2 case 9: `mono_narrowing_reason()` must be non-empty exactly on narrowing and name every
   dropped state.
5. §28.6: the impossible narrowing grep is struck and replaced by the case-9 contract.
6. §28.8: the old grep gate is replaced by the Fixture privacy/accessor and Q3 symbol checks.
7. §20.10 item 18: Button's mono digest is included if the reserved-pad correction moves it.
8. §16.1: the Button no-truncation and Tabs reserved-pad regression names are recorded.
9. New §29: Q1–Q3, R1, the corrected live `RowUi`/`Slot` contract, amendment markers and the
   unresolved questions are recorded and mirrored here.

### Supplied fresh analyst dispositions

- **`OVERLAY` / `TRAPS_FOCUS` — decided.** `OVERLAY` opens a layer; `TRAPS_FOCUS` is separate,
  implies `OVERLAY`, and is for a real focus scope. Modal cases declare both. `Select`'s
  pointer-only `Popover` remains `OVERLAY`-only and non-trapping; focus-out dismissal is a
  separate popover concern.
- **`FieldControl` item channel — decided with follow-up open.** The scalar trait cannot carry
  per-phase items, so item-bearing choice controls stay on direct per-phase paths and `Form`
  drives the three choice controls directly. A future item-aware composition path or trait
  widening remains open.
- **`RadioGroup::value(ItemKey)` — open.** The controlled draw-time behavior and cursor/value
  separation exist in code, but §17.0 A7 omits the public contract wording and still needs its
  controlled-state adjudication.
- **`ChipBar` `Activated(add_key)` — open.** The add affordance uses the caller's `add_key` and
  emits the existing `Activated(ItemKey)` action. Whether to introduce `Added` or `AddRequested`
  remains an action-naming question.
- **`StatusBar` `hovered_part` — open.** The original stateless-bar limitation was real. A
  `FrameRead::hovered_part` primitive is now present in the concurrent tree, but `StatusBar`
  does not consume it; the per-item hover contract and test remain open.

## BLOCKING — §39 must land before the §36 bless (2026-09-04)

**Do not run `BLESS=1` on the component matrix until §39's operator change has landed.**

`Overrides::flags` is `self.state.unwrap_or(live)` — forcing **replaces** the derived state. `live`
is two halves with opposite ownership: the runtime half the frame supplies, and the props-derived
half the caller's props imply. One argument cannot express two ownerships, so **six components
produced five different answers** and `Empty` produced a sixth by opting out entirely.

The render matrix is the only place forced and derived disagree — `St::Disabled` maps onto
`Status::Error` while its flags are `DISABLED` — so **`progress_bar::disabled` is a bar that is in
error and paints no error glyph.** Blessing today pins that.

**Three cells change, 24 keys, 12 of them truecolor.** Before the bless they are *recorded* under
§20.10 item 19 as first generation and nothing is owed. After the bless they are *moved*, which
§36.5's guard refuses outright and §20.10's closing clause makes a regression by construction —
and item 19 may not be cited twice for the same key, so it would need a new numbered item and a
fresh visual review **for pixels that were wrong when they were blessed**.

The six already-blessed components cannot move: the matrix gives them no readiness prop, so their
derived half is empty and the new operator is bit-identical to the old for them. **That is an
acceptance condition, not an expectation** — if any of the 388 existing keys moves, §39 is wrong
and must not land.

### Three premises I handed the analyst were false, and it checked rather than acted

- The six "unguarded" fixtures **are already guarded** — a builder fixed them mid-session. The
  surviving instance is the **render matrix**, which still forces unconditionally and which
  nobody had mentioned.
- The `PROGRESS`/`METER` `ERROR` rules **are** reached now, because `Fixture::force` couples the
  readiness prop and `Caps::REPORTS_STATUS` keeps `ERROR` in `mono_states()`. **Three production
  rustdoc blocks now assert the opposite of the truth** and are struck.
- What *is* genuinely dead is `(PROGRESS, ICON, CHECKED)`: nothing sets `ProgressBar::done`,
  `Meter` has no `.done`, and `CHECKED` is not in the default mono states.

### Why nothing caught it

**Case 9 is the only test that forces a state, and `Fixture::force` deliberately makes forced and
derived agree** — and the two semantics are indistinguishable exactly when they agree. *The
mechanism §28 P6 added to make forcing honest is the mechanism that hid the operator's defect.*
Case 9 also asserts pairwise difference rather than content, and excludes colour, so two of the
three moved cells are outside its universe. **No test shape in this suite can detect an
unreachable recipe rule**; a rule is four coordinates and case 9 varies one.

### Sequencing

The change touches **21 `ov.flags(` call sites across 18 component files** and must be atomic —
it cannot be split across builders without breaking the build. It therefore needs **one builder
owning `crates/tui/src/components/*.rs`**, scheduled when the current per-file builders release.
Classifying each site's argument into runtime and derived **is the work**; it must be read, not
pattern-matched.

`Slot<StateFlags>` was considered and **rejected**: the `Option` already carries the load-bearing
distinction (`is_forced()` gates registration), and a third `Clear` case would mean "force the
empty state, suppressing the props-derived flags" — no consumer, and forbidden by case 9's own
"make the forced state real" clause. §29.1 looks like a precedent and is not.

## Session 4 — Slice 4 wave 1 is effectively closed (2026-09-04)

Measured by the coordinator, not reported by a builder:

- `cargo test -p tui-next --test conformance` — **488 passed / 0 failed**. `select => SelectCase`
  is registered; `DialogCase` declares `TRAPS_FOCUS`; **conformance case 14's trap half executes
  for the first time in the project's history and passes.**
- `cargo run -p xtask -- boundary` — **24 of 25 ok**. `conformance_covers_every_public_component`
  is finally green (22 registered / 23 entries). The single red is
  `every_named_test_exists` on the `capsule_pane_clone_4x2000` row, which is **correct** and stays
  red until Slice 7 deletes the benchmark.
- `cargo run -p xtask -- doc-check` exit 0. `cargo test -p tui-next --lib` **292 passed / 0 failed**.
- `--test render` 8/0. `--test render_components` 45 passed / 115 failed — the pending bless, of
  which **112 are missing baselines and 3 are genuine mismatches**.

### `xtask bless-guard` exists

Documented in §16.3 in the present indicative for months while `xtask` dispatched three commands.
It now implements §36.5's three checks — item citation, **key-set completeness** (co-presence is
satisfied by one sentence beside hundreds of unaccounted lines), and the unconditional
**truecolor refusal** — plus a frozen-evidence refusal whose remedy is *revert, not classify*.

**Ordering is not checked, and the check's own doc comment says so and says why.** A guard that
implied it proved the fixed order would have been the next decorative gate.

Proven able to fail on every path: hard-wiring the evaluator to `Ok` is caught by one test and to
`Err` by the other, **both substitutions checked**; an end-to-end probe perturbed one hash line,
saw the guard name the key, and restored the file; the truecolor refusal, the frozen refusal and
the unresolvable-base error were each demonstrated the same way.

**It is green today only because the bless has not landed**, and that was measured too: appending
one item-19 key produces `added, unaccounted`. Entry 19a's `- added:` field uses angle-bracket
prose placeholders, which are not machine-expandable. The guard accepts a `{a,b}`-alternation
template expanded as a cartesian product whose size must equal the declared count; the fourteen
components written that way expand to exactly 896. **`docs/visual-changes.md` owes that rewrite
before the bless.**

### Ninth decorative-gate candidate

§9.2's invariant — *no overlay component computes a rect* — states its enforcement as a grep over
`crates/tui/src/components/`. **That grep is implemented nowhere**; it exists only as literal shell
in three places in the document. And now that `resolve_anchor` is exported through `author`, a
downstream author's component is by definition outside the grep's scope, so **the in-tree
invariant stays true while the architectural one stops being.**

### Measured, not asserted — the narrowing-reason work

The builder instrumented the driver's own multiset and measured all ten states per component
rather than trusting the adjudication's distinguishability claims. It **deviated from the brief
once, with evidence**: `status_bar` keeps `PRESSED`, because the frame snapshot sets it when the
strip is pressed and `StatusBar` registers its clickable item regions under its **own** id — so
narrowing it would have repeated the exact defect the task existed to fix. It could not write a
truthful "unreachable" sentence, so it did not write one.

Two gate-failure observations recorded: widening `status_bar` to all ten states makes `SELECTED`
collide with the empty state, and declaring `REPORTS_STATUS` while narrowing `BUSY` away fails
with the capability-implies message. Both reverted.

## Measured gate state at `52da837` (coordinator, 2026-09-04)

Run by me after committing the wave-1 foundations, not reported by a builder:

| target | result |
|---|---|
| `tui-next --lib` | **292 passed / 0 failed** |
| `--test conformance` | **488 passed / 0 failed** |
| `--test render` | 8 passed / 0 failed |
| `--test perf` | 27 passed / 0 failed |
| `--test overrides` / `overlay` / `showcase_buttons` | 5 / 4 / 4, all green |
| `tui-next-testing --lib` | 5 passed / 0 failed |
| legacy root package | 76 + 67 + 33 + 41 + 30 = **247 passed**, unchanged |
| `xtask doc-check` | exit 0 |
| `xtask boundary` | **24 of 25 ok** |

**Exactly three reds remain, and all three are understood:**

1. `--test architecture` 28/1 — `every_named_test_exists` on the `capsule_pane_clone_4x2000`
   row. **Correct.** It stays red until Slice 7 deletes the benchmark, and it is red because the
   check was *fixed* this session: it previously read a file that had never contained the row.
2. `--test render_components` 45/115 — the pending first-generation bless. **112 are missing
   baselines and 3 are genuine mismatches.** Blocked on §39 landing first (see the blocking
   entry above) and on `docs/visual-changes.md` entry 19a being rewritten as a machine-expandable
   pattern, which `bless-guard` now requires and which was measured, not assumed.
3. `--test status_bar_hover` 0/1 — another lane's untracked, deliberately-uncommitted test.

Nothing else in the workspace is failing.

## Slice 4 wave-1 verification checkpoint — evidence at `463efca` (2026-09-04)

The pushed implementation evidence is present in the current history: `739754c` contains the
`Select` production fixes (the disabled-input guard and `PARTS` declaration; the current case-19
lower-bound geometry fix is already present), `eeee504` contains the Dialog/Select conformance
changes, and `a1dadc5` contains the case-14 zero-area trap proof. The measured stable evidence
baseline is pushed `463efca`.

| gate | exact result |
|---|---|
| `rtk cargo build -p tui-next --all-targets` | exit 0; 12.518s; 1 warning |
| `rtk cargo test -p tui-next --lib` | exit 0; 292 passed / 0 failed |
| focused Select conformance | 21 passed / 0 failed |
| focused Dialog conformance | 21 passed / 0 failed |
| full conformance | 488 passed / 0 failed |
| `rtk cargo test -p tui-next-testing --lib` | 5 passed / 0 failed; 1 ignored |
| `rtk cargo fmt --all -- --check` | exit 0 |

The canonical `architecture::every_named_test_exists` check exited 101 only because
`tests/perf_baseline.txt:3` still contains `capsule_pane_clone_4x2000`. §21 item 10 / §16.6
deletes that benchmark for the out-of-scope Slice 7 `apps/jackin-preview` work; this is deferred
baseline drift, not a Slice 4 defect or fix. Architecture is complete through §41, so no new
architecture section is added.

At append time, shared `main` had advanced to `1b580d7` through later pushed documentation
commits; `463efca` remains an ancestor and is the gate-evidence baseline recorded above.

## Critical path — the remaining work, ordered (2026-09-04)

The stop-hook check is right: Slice 4 wave 1 is closed, and **the goal is not**. What remains,
in dependency order, with what blocks what.

### Blocking chain to the bless

1. **§39 — the forced-state operator.** 21 `ov.flags(` call sites across 18 files. **Atomic** —
   it changes a shared signature, so every site moves in one commit or the crate does not
   compile. **In flight, single owner.** Classifying each site's argument into runtime and
   derived *is* the work; it must be read, not pattern-matched.
2. **The 896-line bless.** Blocked on §39, because landing after it would move **12 truecolor
   keys**, which the guard refuses outright and §20.10's closing clause makes a regression by
   construction — and item 19 may not be cited twice for the same key. Also owes
   `docs/visual-changes.md` entry 19a rewritten as a machine-expandable pattern, which
   `bless-guard` now requires; measured by appending a key and watching it report
   `added, unaccounted`.
3. **§45 — the slot contract**, across five Slice-4 packages. Deliberately **after** §39, because
   §39 moves 24 keys across three of the four files §45 also touches, and §39's movement must be
   attributable to §39 alone or item 19's "nothing moves" premise cannot be checked.

### Blocking chain to Slice 5

4. **Eighteen of thirty-six component files are unwritten.** **4E is the critical path**: every
   one of the 22 showcase pages imports `Panel`, so **zero pages can migrate until it lands.**
   In flight. After 4E, three pages unblock; the other nineteen need 4C, 4F, 4H or 4I.
5. **The three `apps/` boundary guards must exist *before* `apps/` is created** (§47.5), each
   asserting an expected set is **present** — written as "scan and find no violations" they are
   green-and-empty today and stay so forever, which is §37.1's recorded failure verbatim. In
   flight, together with the §16.5 scan gap that made their absence unreportable.
6. **`every_named_test_exists` is correctly red** on the Slice-7 benchmark row and will get
   *more* red as the §16.5 names are enumerated. **That report is the deliverable**, not a
   regression.

### Then

7. Slice 4 wave 1 remainder (4A `too_small`), wave 2 (4C, 4F, 4H, 4I).
8. Slice 5 per §47's deferred-rename sequencing, with Invariants S, S2, T and U.
9. Slices 6 and 7 — Lanes B and C.
10. Slice 8: the legacy deletion per §44.6's four preconditions, the rename commit, `DESIGN.md`'s
    ten divergences, the README rewrite and five guides, then a fresh architecture review and a
    **separate** fresh visual review — the latter needing the **first mono capture ever taken**,
    since §34.4 established that review has never executed.
11. The §30 final report, fifteen items.

**Not a single one of these may be closed by an aggregate argument.** The standard this session
established, at the cost of ten decorative gates and three contracts asserted by a mechanism
sharing their defect's enabling condition: **a check that has never been observed failing is not
evidence.**

## Session 5 — Codex continuation measurement (2026-09-04)

The previous goal turn made progress: commits through `aed6f41` changed authoritative state.
This continuation re-measured the current tree before relying on that record.

### Measured state

- `rtk cargo build -p tui-next --all-targets`: exit 0.
- `rtk cargo test -p tui-next -p tui-next-testing --all-targets --all-features`: exit 101.
  The library portion passed 303 tests; architecture failed only
  `every_named_test_exists`, `conformance_covers_every_public_component`, and
  `no_deprecated_or_legacy_api_usage`.
- `rtk cargo run -p xtask -- doc-check`: exit 0, 71 Rust blocks and 618 references,
  but the fresh gate audit proved this is incomplete evidence: the scanner omits post-§26
  adjudications and permits unresolved allow-listed names.
- `rtk cargo run -p xtask -- boundary`: exit 1 on the same three checks. A builder has since
  corrected the four forbidden-pattern hits and removed the three stale named-test deferrals;
  its re-run left only dormant component registrations red.
- `rtk cargo test --all-targets`: exit 101 only through those architecture wrappers; legacy
  targets passed 76 + 67 + 33 + 41 + 30 = 247 tests.
- `rtk cargo test -p xtask --bin xtask`: 21 passed / 1 failed. The failure correctly proves
  §20.10 item 19 still lacks `{scope: first-generation}`.

### Current owned work

- Dirty §49 owner: `.github/workflows/ci.yml`, `xtask/src/main.rs`, and
  `crates/tui/tests/render_components.rs`.
- Dirty §45 owner: `components/{brand,keyhint}.rs`; shared tiny-rect correction:
  `crates/tui/src/layout.rs`.
- Dormant Slice-4 work: untracked `components/{too_small,nav_list,steps,tree,grid}.rs`.
  These are intentionally not exported yet; green library builds do not compile them.
- Fresh Lane-B decision/status records now exist at
  `docs/reviews/laneB-grid-contract.md` and `docs/status/laneB.md`.
- Lane-C decision/status record creation is in flight at
  `docs/reviews/laneC-app-tick.md` and `docs/status/laneC.md`.

### Newly proven blockers and decisions

- Grid Q1–Q3 are adjudicated in the Lane-B record: model owns ordering/comparison; Grid emits
  `Sort(ColumnKey, SortDir)`; `GridState` exposes only state it owns; zero-area focus uses an
  explicit focus-only registration API with no hit region. Architecture transcription and
  implementation remain.
- Current Tree draft contradicts the accepted cached incremental `TreeIndex` decision and must
  not land until a fresh adjudication is recorded and implemented.
- §39 is not closed: `Fixture::forced()` still returned `StateFlags`, leaving 21 manual guards;
  a builder owns that correction. `Empty` nested override forwarding and Progress done-rule
  reachability are a separate builder package.
- Slice 5 and both app migrations have not started. No `apps/` directory exists. TablePro legacy
  proof is 41 tests / 23 app test attributes / 42 digests; Jackin legacy proof is 67 tests,
  including 22 app + 6 chrome, and 36 digests.

### Next action

Finish fresh choice, StatusBar, Tree, and 4E adjudications; transcribe accepted decisions into
`COMPONENT_ARCHITECTURE.md`; finish §39; close §49 and the 160-cell render matrix; then integrate
and certify dormant Slice-4 packages in dependency order. No slice-completion claim is made.

### Session 5 progress after initial measurement

- §39 is implemented: the two-half forced-state operator was already present; Fixture now
  preserves `None` versus `Some(empty)` and removed 21 truthiness guards; Empty forwards nested
  override state/patches; Progress `done(true)` reaches the CHECKED recipe. Independent focused
  evidence: testing lib 6 passed / 1 ignored, conformance 488 passed before later choice changes,
  component lib 306 passed, focused clippy clean.
- §49 scratch evidence is under `/tmp/fable49-evidence-lStqI4/`. A discarded bless measured
  **22 moved keys**, correcting the prior predicted 24: HintBar 8, Meter 6, ProgressBar 8;
  12 truecolor and 10 mono. Scratch matrix passed 164/164. Xtask passed 22/22 and each changed
  guard was demonstrated red on a broken isolated input and green on the fixed one.
- A fresh independent visual analyst accepted all six Junie 120×40 corrected frames: each has
  the declared `GlyphRole::Error` (`!`), with labels, tracks and `65%` intact; movement is confined
  to disabled keys. Evidence lacks textual before-frames, so the sign-off relies on exact digest
  scope plus reviewed after-frames for unrelated-change exclusion.
- Accepted adjudications §§50–§56 now cover Choice identity/controlled value, StatusBar keyed
  hover, Grid ownership/focus-only registration, cached incremental Tree/query projection,
  Jackin status/tick/dimming, total closure containers, and SplitPane's two-slot closure.
- §49 classification/bless is in flight. Tree, Choice, 4E and dormant-family review/fixes remain
  active; no Slice-4 completion claim is made.

### Session 5 §49 closeout and 4C adjudications

- §49 is committed and pushed at `bfcf5e4`. Clean-worktree proof at that commit: xtask 22/22,
  render-components 164/164, and `BLESS_GUARD_BASE=HEAD^` accepted exactly 22 moved / 0 added
  keys. The retained baseline SHA-256 is
  `1ab8e9205a19069ff5f9d97d675df77e6051c6195ad7a882766163cc2e744c9e`.
- Choice/Chip implementation passed 321 library tests; Tree incremental cache/query passed 11
  focused tests; TextArea containment/forced-scroll correction brought the current integrated
  library to 370/370 and conformance to 489/489. These code results are not yet committed.
- Fresh 4C review found NavList/Steps source materially incomplete. Accepted §§57–§59 now bind:
  NavList has separate `EnterContent(ItemKey)`, icon-only collapsed groups, and scoped owner
  patches; Steps keeps skipped rows inspectable/read-only, emits stable-key movement only on real
  change, owns lifecycle META, and uses a runtime-derived incremental frontier cache.
- NavList, Steps, TooSmall and Grid remain dormant/unexported. Their implementation, certification,
  first-generation classification and API reviews remain required.

### Session 5 independent Slice-4 review corrections

- Tree review rejected generation saturation, derived-cache lifecycle, unselected marker work,
  and missing cached-reorder proof. The builder now forces safe rebuilds at saturation, evicts
  cache entries across a frame gap, skips all unselected marker resolution, and passes 18 Tree
  unit tests, 21 Tree conformance cases, three derived-cache tests, and strict Clippy. Mandatory
  Tree performance tests remain.
- 4E review found nested viewport layout facts overwrote ScrollRegion's track height. The builder
  preserved the inner track facts and added exact slot-surface and off-screen caret proofs; Panel
  9, SplitPane 15, TextViewport 17, ScrollRegion 9, and the 431-test library suite passed before
  concurrent Grid work resumed. Dialog's total-return contract and the AST signature gate remain.
- Grid review rejected the draft's second column-count authority and implicit alignment sentinel.
  Accepted §61 deletes `type Row` and `col_count`, makes `cell` return `Option<CellRef>`, and makes
  cell alignment optional: absent inherits the declared column, present is explicit. Missing cells
  retain rectangular geometry but can never reach decoration, action, or editor hooks.
- TooSmall review found CONTAINER slot replacement changed returned geometry and its `q Quit`
  styling cannot preserve the legacy faint hint through the shared PANEL recipe. Geometry repair
  now passes 10 unit tests. Accepted §63 adds isolated `Family::TOO_SMALL` at raw value 34 with the
  exact legacy notice hierarchy; implementation, certification, and item-24 baselines remain.
- The second discarded render probe added 256 Panel/SplitPane/TextViewport/Tree keys and moved 41
  existing keys. Fresh visual review rejected blessing: ChipBar used a clipped CheckboxOn glyph,
  mono viewport selection disappeared, and Tree painted two chosen markers. Four TextArea mono
  pressed movements are accepted under §20.10 item 1; all other changes await structural fixes,
  recorded visual-change items, a fresh scratch probe, and fresh review.
- New collection certification reached 653/658 and exposed five non-decorative failures. Accepted
  §64 separates keyed COLLECTION from SELECTS, makes conformance set up real semantic state, and
  models Grid's double-click activation gesture. Grid's real 2×2 overflow and subsequent §61
  re-review defects are fixed; current focused Grid evidence is 23 unit tests plus its tiny
  conformance case. The framework corrections and full rerun remain.
- §64 integration then exposed a sixth real mismatch: draft Grid `.status` propagated readiness
  to every loaded row but painted no mono affordance. Accepted §65 deletes that unspecced global
  API, retains local empty/fetch/decor ownership, and adds a targeted mono underline for decorated
  error cells. Production, fixture, theme, and full conformance corrections are in flight.
- The release perf sweep found `style_downgrade_theme_all_levels` at 1,145 allocations over its
  1,079 limit. Controlled attribution proved 50 allocations came solely from compiling 418 lines
  of unrelated new collection benchmarks into the same integration binary, while the new family
  and targeted rules added 16. Accepted §66 isolates collection subjects in their own process;
  exact reservation of mono-rule storage independently reduces the live measurement to 1,074
  allocations / 149,128 bytes without changing thresholds or resolved output.
- §66 integration gate: `cargo test --workspace --test perf --test perf_collections --release --
  --test-threads=1`; workspace `perf` still includes the frozen legacy-root process, while
  `perf_collections` adds the isolated crates/tui collection process.
- TextViewport exact-layout adjudication: visual-row correctness wins. Complete `usize` wrapped-row
  prefix in `Ui::cache`; O(document) cold/reflow/invalidate, append-suffix incremental, O(visible)
  warm; literal `visible_range ± 1 page` indexing rejected as impossible with borrowed lines lacking
  global metadata. Same-length edits require `ViewportState::invalidate`; saturated generation
  disables reuse. §20.9-7/§12.4/§16.6 corrected; no visual-change item and no baseline movement
  authorized.

## CONTINUE / HANDOFF — central `Ui::reference` migration

### Completed

- Option B is the current A11 contract: `Ui::reference(Option<ReferenceTarget>, …)` makes the whole
  subtree inert and injects `FOCUSED | FOCUS_VISIBLE | HOVERED | PRESSED` only into one exact
  component/item/part. Component-local `state_override` and `inherit_forced` are superseded.
- Canonical architecture API, fixture, matrix and boundary records now point to the central scope.
  Historical adjudication text remains intact and is explicitly superseded by §72 rather than
  rewritten as if it had never existed.
- Exact structural boundaries are `architecture::legacy_forced_state_apis_are_absent` and
  `architecture::reference_rendering_is_ui_scoped`.
- Stabilized scratch comparison is now exact: before SHA-256
  `1ab8e9205a19069ff5f9d97d675df77e6051c6195ad7a882766163cc2e744c9e`, after SHA-256
  `4c4dd527261acc03431858db024f884385463a40131b16ae340564da9ca42299`, with **280 moved** and
  **1,280 added** component keys. Movement ownership is item 1 = 8, item 20 = 12, item 23 = 39,
  item 28 = 16, item 30 = 28 and item 31 = 177. Items 22/24/25/26/27 retain sole ownership of the
  1,280 first-generation additions; item 27 owns its nine Slice-4F components (**576 keys**).
  No retained baseline has been blessed or copied.
- Readiness-state mono separation is structural: `MONO_RULES_PER_FAMILY` is now **20**. The 20th
  generic rule applies `UNDERLINED` to `Part::ICON + LOADING`, distinguishing it from `BUSY` while
  both keep the same spinner sequence. §20.10 item 1 / visual ledger item 1d classify the fix; no
  retained eight-state digest key isolates `LOADING`, so no baseline movement is claimed.
- `FormState` does not expose or implement generic `Reconcile<ItemKey>`. Its private
  `reconcile_fields(&[FieldSpec])` is keyed by field `Id`, preserves draft/editor state across
  declaration reorder, creates newly declared slots, and zeroizes removed slots before drop.

### Current

- Exact old→new claims for every moved key are recorded in `docs/visual-changes.md`. A disposable
  git-backed validation copy at `/private/tmp/fable-ledger-guard.oqRwbb` ran `xtask bless-guard`
  against the stabilized scratch pair and passed with **280 moved / 1,280 added**.
- Earlier scratch paths and counts remain historical evidence only. The authoritative pair is under
  `/private/tmp/terminal-components-baseline-review.qbw8Cu/` with the hashes above.

### Next

1. Complete independent frame review against the stabilized scratch evidence.
2. Run the remaining source/full-workspace gates from the integrated tree.
3. Seek separate authorization before any serial retained-baseline bless.

### Blocker

- Independent visual approval remains outstanding. Exact accounting is complete, but it does not
  authorize copying or blessing the retained baseline.
- WP-4F readiness/activation correction evidence: FilterList declares and directly paints `ICON`
  for Busy/Loading/Error while Ready reserves no column; PickerChain declares and directly paints
  its picker-family `ICON`, exposes only keyed Back/Retry actions, and publishes root-owned keyed
  breadcrumb/retry parts; Wizard publishes root-owned keyed enabled-step label parts. PickerChain
  forced/live `PRESSED` is stripped from the container and applied only to the exact actionable
  crumb. Exact unit evidence is PickerChain **5/5** and Wizard **3/3**; focused PickerChain
  conformance is **21/21**; strict all-target/all-feature Clippy, scoped format, and diff checks pass.
  The earlier shared conformance-driver `Option<Chord>` compile failure was concurrently resolved;
  no functional conformance dependency remains. First-generation visual baseline review/blessing
  remains unresolved and unauthorized as recorded above.
- §§68–§70 foundation evidence is measured: focused KeyMap tests pass 8/8; binding-chord claim,
  dynamic Menu routing/painting, dynamic Dialog routing/painting and open MenuBar single-control
  tests pass. Strict library Clippy and `xtask doc-check` pass. Release measurements are
  `frame_hintbar_derived` 0 allocations/0 bytes, `frame_form_update_draw` 0/0,
  `viewport_100k_lines_push` 0/0, and `style_downgrade_theme_all_levels` 736 allocations/122,304
  bytes (ceiling 1,079/170,904). The numeric Form row is classified under §20.10 item 27; no visual
  baseline was blessed. Full-suite closeout remains gated by separately owned conformance failures.
- Final shared cleanup removed the last viewport `too_many_lines` lint by extracting pointer-phase
  handling, with no suppression. Static mono proofs now carry resolver-truth names rather than
  obsolete append language. Strict library Clippy passes with all features and with no defaults;
  the three renamed mono tests, `xtask doc-check`, architecture doc-check and the named-test gate pass.

### Session 5 readiness-icon and semantic-selection correction

- Every public component in this correction that accepts `.status(Status)` now declares
  `Caps::REPORTS_STATUS`, includes `Part::ICON`, and paints a root-owned symbol for
  `Busy`/`Loading` and `Error`. Button uses one conditional leading symbol-plus-gap lane,
  with readiness taking precedence over an explicit icon and the independent checked marker
  retained. TextInput and TextArea use one always-reserved trailing cell with exact priority:
  validation `MARKER` > status-error `ICON` > busy/loading `ICON` > blank. List uses a
  conditional two-column left rail before every visible row and empty body while leaving its
  scrollbar outermost. Tabs uses a conditional two-column far-right row-zero lane outside its
  overflow/new reservations. Ready/default geometry is unchanged.
- Forced rendering is no longer semantic selection authority. Button's checked presentation is
  supplied through the conformance fixture's controlled `Fixture::selected`/checked prop. List paints selection
  only from `ListState::chosen`; Tabs paints activation only from `TabsState::active`; their mono
  setup uses real Space/Enter paths. `FieldCase` deliberately does not inherit
  `REPORTS_STATUS` from its TextInput child.
- The same invariant now holds in Tree, NavList, ChipBar, and RadioGroup: chosen/current/checked/
  controlled-value membership comes only from component state or the controlled prop. Forced
  first-row stand-ins remain cursor-only, so A11 `FOCUSED`/`PRESSED` references paint without
  inventing selection. Exact focused units pass: Tree **22/22**, NavList **14/14**, ChipBar
  **14/14**, RadioGroup/Choice **8/8**. Per-file rustfmt and scoped diff checks pass. Strict library
  Clippy reached only the separately owned `viewport.rs::note_indexed` `unused_self` finding.
- Scalar selection follows the same ownership rule. Button, Checkbox, and Toggle derive
  `CHECKED | SELECTED` only from their real boolean prop; Select derives `SELECTED` only from a
  non-empty `SelectState::value`. A forced `SELECTED` is masked when that semantic source is false
  or absent, but remains available to style a genuinely selected control. Focused units pass:
  Button **3/3**, Choice **10/10**, Select **8/8**; per-file rustfmt and scoped diff checks pass.
  Strict library Clippy reaches only the separately owned `viewport.rs::update_lines`
  `too_many_lines` finding.
- Select disclosure now has appended `GlyphRole::{SelectClosed, SelectOpen}` roles at stable
  discriminants 39/40; Junie and Paper bind them to single-cell `▾`/`▴`. Closed-field marker
  resolution excludes semantic `SELECTED` while retaining pressed flags, so popup selection keeps
  its one `Chosen` marker and mono press keeps `[`/`]`. §20.10 item 29 and §71 record the fix,
  including the eventual full-ASCII `v`/`^` mapping; no baseline key moves or additions and no
  BLESS occurred. Focused glyph **3/3**, builtin **10/10**, Select units **12/12**, Select
  conformance **22/22**, and strict library Clippy pass. Per-file formatting and scoped diff checks
  pass. The coordinator owns the interrupted full-conformance/doc-check rerun; workspace fmt was
  blocked only by the separately owned `tests/render_components.rs:639` formatting diff.
- Exact evidence after the correction: full `tui-next` library **604/604 passed** and full
  conformance **914/914 passed**; focused
  Button **2/2**, TextInput **11/11**, List **2/2**, Tabs **4/4**, TextArea **9/9**; focused
  conformance mono cases pass for Button, List, and Tabs, and full component-region runs pass for
  Button/TextInput/Tabs/TextArea (**21 each**) plus the List-filtered set (**63**). `cargo check
  -p tui-next --lib --tests` and strict `cargo clippy -p tui-next --lib --all-features -- -D
  warnings` pass. No baseline was blessed or modified.
- Form F11 correction now claims effective Enter before child dispatch only for a focused, visible,
  non-editing control that does not swallow typing; child intents remain replayable and submit wins
  the frame's single response. Form placement is a borrowing state machine shared by update/draw,
  with no placement `Vec`. Forced Form reference rendering propagates to every configured field,
  scroll region, and action button and registers no controls or parts. Focused Form units are now
  **28/28**; strict all-feature library Clippy and `cargo fmt --check` pass. The separately owned
  warm update+draw allocation assertion remains pending integration in `tests/perf.rs`.
- Completion reachable routing correction is implemented: `CompletionController::new(editor_id,
  popup_id)` records editor binding ownership while layer geometry, pointer intents, scrolling, and
  lifecycle remain addressed to `popup_id`; `Completion::update_for(editor_id, …)` consumes the
  editor-addressed binding intents, and open completion draw publishes its table for the focused
  editor. Real Runtime regressions prove Down moves completion without moving CodeEditor, Tab and
  Enter accept, Esc dismisses, ordinary text remains editor-owned, and owner-scoped KeyMap remap and
  removal are effective. Completion unit evidence is now **7/7 passed** and focused conformance is
  **21/21 passed**. `cargo check -p tui-next --lib` and `cargo doc -p tui-next --no-deps` complete;
  strict Clippy remains unresolved only outside Completion due concurrent `collection/empty.rs` API
  mismatches and `viewport.rs` unused variables. No baseline was blessed or modified.
- Cross-cutting binding/theme/viewport correction: dynamic menu and dialog action chords now publish
  stable hidden descriptors beside their static control tables; routing, owner remap/removal and
  painted chords share one effective resolver. Claimed effective chords retain unclaimed intent
  order. Mono fallbacks moved from cloned recipe vectors into the static resolver precedence layer,
  and TextViewport work accounting is caller-owned under `testing`, with no global accumulator.
  Focused dynamic routing, mono downgrade allocation, and viewport append allocation tests pass;
  full gates remain in flight.

### Session 5 Slice-4H correction evidence

- Code find now has an explicit routed input state: `/` opens it; bare characters edit the query;
  `Backspace`, `Enter`, and `Esc` erase, accept, and cancel. It remains usable in read-only mode.
  Pointer placement consumes `Part::TEXT`-local coordinates exactly (including wide graphemes),
  every document-mutating command is gated by read-only state, and hardware cursor requests stay
  inside the visible half-open text rectangle. While find is typing, its footer cursor exclusively
  owns the cursor request.
- DiffView no longer creates nested borrowed `Vec<Span>`/`Vec<ViewportLine>` projections in update
  or draw. Its runtime-owned cache stores an owned flattened text arena plus line/run descriptors;
  TextViewport consumes that projection through crate-private phase adapters. No borrowed source
  text enters the cache.
- Exact verification: Code unit **8/8**, Diff unit **6/6**, TextViewport unit **27/27**; Code and
  Diff conformance **21/21 each**; `frame_tablepro_query_editor_2k_lines` reports **0 allocations / 0
  bytes**, stable one-time dense highlighting, and a **0.99** 2,000-line/100-line ratio while carrying
  dense diagnostics and find matches; `diff_2k_cached_projection` reports **0 allocations / 0
  bytes** across warm update plus draw. `cargo clippy -p tui-next --all-targets --all-features -- -D
  warnings`, `cargo fmt --check`, and scoped `git diff --check` all exit 0.
- Fresh independent review first rejected the competing document cursor request, then passed after
  the exclusive find-cursor gate and exact `Runtime::cursor()` regression were added. Final verdict:
  **PASS; all six correction findings satisfied**.
- Visual closeout remains deliberately unresolved: CodeEditor **8/8** and DiffView **8/8** render
  cases compile and reach the digest gate, but each reports only a missing first-generation baseline.
  No baseline was blessed or moved. Classification, fresh visual review, and an authorized serial
  bless remain required.
- Slice-4H visual inventory is now exact: `diff_view` and `code_editor`, **2 components × 64 =
  128 first-generation keys**, classified by §20.10 item 26. A validated scratch-only generation at
  `/tmp/fable-slice4-final-VPCSC9/repo` passed the complete `render_components` target **325/325**;
  its exact diff/key inventories are `/tmp/fable-slice4-final-VPCSC9/artifacts/components-scratch.diff` and
  `/tmp/fable-slice4-final-VPCSC9/artifacts/added-keys-exact.txt`. The Junie 120×40 no-BLESS frame dumps are
  `/tmp/fable-slice4-final-VPCSC9/artifacts/junie-120x40-{truecolor,mono}-frame-text.log`; each has
  all eight states for both 4H components. Fresh independent review and authorized blessing remain.

### Session 5 Slice-4F implementation evidence

- Conformance composite-owner correction is complete. The framework now distinguishes root,
  control, activation, scroll, and opener ids; Form exercises its real secret `TextInput`, Dialog
  its real action child, and Completion its editor-owned bindings plus popup-owned rows/scroll.
  Overlay setup is explicit, closed before the trigger, and tiny-screen coverage opens the honest
  layer before checking clipping and stale geometry. Dynamic action bindings prove default,
  owner/action remap, removal, and reverse declaration-union behavior; an open MenuBar dropdown
  proves its item chord is consumed.
- Capability truth is restored for Grid, Select, Completion, Wizard, PickerChain, Checkbox, and
  Toggle. `REPORTS_STATUS` now structurally requires `ICON`, and Busy/Error mono evidence must
  change an actual symbol cell rather than style alone. `Fixture::force(DISABLED)` now couples the
  real disabled prop, while forcing empty clears it; semantic selection comes only from controlled
  state/setup or the explicit Button/Radio fixture knob, never from forced flags. Exact evidence:
  full conformance **913/913 passed** and `tui-next-testing` **14 passed / 2 ignored**. Strict
  all-target `tui-next-testing` Clippy passed before a concurrent Viewport change; the final rerun
  reaches only the separately owned `viewport.rs:1188` `too_many_lines` finding. No baseline was
  blessed or modified; existing first-generation Slice-4F/4H visual baselines remain unresolved as
  recorded below.

- Slice-4F production and conformance are implemented for Menu/Help, semantic
  FilterList/Picker/Completion, Form and its configured field bridges, Wizard, PickerChain, and the
  four retained Dialog contracts. The exact current conformance run is **892/892 passed**.
- Focused unit evidence is Menu 4, Help 3, Completion 2, FilterList 3, Picker 4, Form 25, Wizard 1,
  and PickerChain 1, all passed. Theme downgrade passed 15 tests; the exact Menu pressed/Help
  focused recipe test passed. `cargo test -p tui-next --doc` passed 2 doctests and
  `cargo check -p tui-next --examples` passed.
- `cargo fmt --all -- --check` and strict library Clippy passed. Strict all-target Clippy is not yet
  green: the remaining finding is the separately owned picker performance fixture's
  `tests/perf_collections.rs` `stats`/`state` `similar_names` collision.
- The fresh `xtask doc-check` resolved 71 Rust blocks and 666 references; its stale binding lookup
  reference from the shared migration is now corrected.
- The four new Slice-4F render groups compile, but their first-generation baseline entries remain
  deliberately unblessed. No visual ledger or baseline was changed by this package; classification,
  fresh visual review, and an authorized serial bless remain required before Slice-4F closeout.
- The completed Slice-4F visual roster is `filter_list`, `picker`, `completion`, `form`,
  `context_menu`, `help_overlay`, `menu_bar`, `picker_chain`, and `wizard`: **9 components × 64 =
  576 first-generation keys**, classified by §20.10 item 27; retained `dialog` is excluded. Empty
  fixtures now omit their optional content instead of merely blanking values. The same validated
  scratch run above generated the exact scope, and the two Junie 120×40 no-BLESS logs each contain
  all eight states for all nine components. Across the complete pending matrix the scratch diff
  measured **1,280 added keys (20 × 64) and 79 moved keys**: 39 ChipBar under item 23, plus the
  item-28 semantic-selection set (List 8, Tabs 8, RadioGroup 4 mono, Select 20 mono). The 4F/4H subset is
  **704 additions and zero movements**. Retained `components.txt` stayed byte-identical at SHA-256
  `1ab8e9205a19069ff5f9d97d675df77e6051c6195ad7a882766163cc2e744c9e`; the scratch-generated SHA-256
  is `58e4f10c3f53b60dd1c61781283f33d40d3e886961b23f0ce4b555a0b43b00ab`. No live BLESS ran and no
  retained baseline was modified. Fresh independent review and authorized serial blessing remain.
- The scratch-local `xtask bless-guard` expanded the ledger and passed with **79 moved / 1,280
  added** component keys. The live-tree guard separately passed with **0 moved / 6 added** numeric
  perf keys. Each final frame log contains **88** missing-baseline frames (11 target components × 8
  states), giving **176** review frames total. Artifact SHA-256 values: diff
  `003fef46e2dfb4d12aa75f96d025ef32b9f1d6e004ad26645421f9828aa907b2`, added inventory
  `675342a9be44b6aab9052a011188b68af403c7a59df26b9f0be6203d757bf051`, moved inventory
  `fd07e7f79b6452e0245b3f2941c36be4905cf3f9101841d7ebbae598c14753db`, truecolor frames
  `1f863bb5285b2b5f91c18ef75e9dffd3f7df3b96fa0cc38ad05c3e0489349684`, mono frames
  `c9af2eb37504d7ad569960fe908974b016fe070facbeed0307de8fccc51abc20`.
- Accepted §§68–§69 replace chord-identified component dispatch with stable `ActionKey` binding
  publication, owner-scoped overrides, typed `Intent::Binding`, structural KeyMap revision, and a
  reusable focused-hint cache shared with routing. Explicit component Tab bindings precede focus
  traversal. Case 12 gains an explicit reveal hook; pointer state retains the pressed `PartRef` so
  Grid/ScrollRegion style only the hit part. New allocation proofs are `frame_hintbar_derived` and
  `picker_100k_borrowed_domain_render`; neither authorizes a visual baseline change.
- §67 pointer-Move foundation is implemented: uncaptured movement resolves the top live
  non-decorative part, updates hover, enqueues exactly one `Phase::Move`, and requests paint only
  for a visible hover transition. Captured movement remains `Phase::Drag` only; movement never
  focuses or activates. The five exact runtime Move tests pass; full focused verification remains
  in flight with concurrent Slice-4 work.
- Slice-4F adjudication §67 fixes three previously underspecified boundaries: picker-family data
  comes through semantic `Item`/`AsItem` with `ItemRow` as paint-only default; menu hover requires
  the new runtime `Phase::Move` and no Press substitute; Form keeps configured `FieldKind`
  controls through crate-private inherited-disabled phase bridges and a sealed `TextTarget` for
  `String`/`Secret`. Implementation and first-generation evidence remain in flight.
- TextViewport exact-layout adjudication: visual-row correctness wins. Complete `usize` wrapped-row
  prefix in `Ui::cache`; O(document) cold/reflow/invalidate, append-suffix incremental, O(visible)
  warm; literal `visible_range ± 1 page` indexing rejected as impossible with borrowed lines lacking
  global metadata. Same-length edits require `ViewportState::invalidate`; saturated generation
  disables reuse. §20.9-7/§12.4/§16.6 corrected; no visual-change item and no baseline movement
  authorized.
