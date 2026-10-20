# Picker dependency integration review

Status: review-only dependency integration. Not product acceptance and not ready for PR #38.

## Base and source

The branch is based on exact reviewed commit `4a0ab4983b84b4909d0182a323eb6e1714651691`; `a5766d24783c2b268e727497fe6861b6479c71cc` is its ancestor. Both ancestry checks pass. This retains the reviewed shared-style commits `f3dc9c506` and `708420d47` already in `4a0ab4983`.

Runtime dependencies, cherry-picked in order:

1. `2fe34623115a3efc0a864c0de4f6d3df6bbf948c` shared clipboard and pointer editing support.
2. `7747eb6a0d730ac0decf5140a6144d53dd9105b9` captured click sequences.
3. `d134e241eae592646772f514d32a6e53974618e1` release-time double-click expiry.
4. `fbd7cd7a3f7cd8ef914ad409ed39152ddca81f5e` baseline click-release bounds.
5. `f3e06913376d3820acf5a9597597aec91f2a576b` viewport-margin layer constraints.
6. `b090f17585f1be2cd231d11acceff1549ab1b80c` reviewed session caret capability and cursor projection.
7. `286ee126067296e2e2bea53830c5a21261bc894e` prepared settled frames.
8. `29d07015ae9a19bb282e30f48c1876ff15a8fc9e` settle prepared frames before terminal output.
9. `9461aca2f1a5d9b8d34b0c33540d12ab6e157366` validate focus-continuation receipts.
10. `6be6d55ee57b8e6eba1ebb80690c9908b38dcecc` gate layer completion on restored focus.

The session source blob at `b090f17585f1be2cd231d11acceff1549ab1b80c` equals the reviewed session projection blob at `94f937deb192d1678476b5b1d4c2e4511b75627e` (`05077060bef336a7a8f923f6d5eb9e38b3303d04`). The session branch merge contains unrelated runtime and text changes, so it was not merged.

Field dependencies were cherry-picked directly, without duplicating their runtime ancestry:

1. `35d65dd2be6c8e3560a129c776cbd8ceb8c200c3` shared contextual edit commands.
2. `c6dedf174e0a8e9531523222fd1e948f109f0daf` preserve drafts during exact focus continuation.
3. `de7cc53dbe8483f7e44837e469efbc9577f1455b` record the continuation runtime blocker.

All cherry-picks applied without conflicts. Relative to `4a0ab4983`, the selected source changes are limited to `crates/termrock-core/src/diagnostics.rs`, `crates/termrock-fields/{src/edit.rs,src/input.rs,src/lib.rs,src/textarea.rs}`, `crates/termrock-runtime/{src/author.rs,src/layer.rs,src/runtime.rs,src/runtime/clipboard.rs,src/ui/cx.rs,tests/monotonic.rs}`, `crates/termrock-session/src/session.rs`, `crates/termrock-test-support/src/harness.rs`, `crates/termrock-text/{src/buffer.rs,src/editor.rs}`, `crates/termrock/src/lib.rs`, and the source audit notes `docs/audits/2026-10-20-picker-family-contracts.md` and `docs/audits/2026-10-20-runtime-prepared-frame.md`. The only local test changes adapt the two field `publish()` helpers to use `Runtime::prepare`, `draw_prepared` through Ratatui `TestBackend`, and `acknowledge_presented`; production source is not changed locally.

## Verification

The initial scoped `cargo nextest` run at the incomplete `a576`-only base is superseded. At exact reviewed base `4a0ab4983b84b4909d0182a323eb6e1714651691`, focused run ID `17b4ffd2-af02-4a55-9449-264daac5d579` ran 10 pass, 1 fail. Only `termrock-fields::note::tests::note_wrap_and_clip` failed there, with actual muted color `Rgb(138, 138, 138)` versus expected `Rgb(128, 128, 128)`.

At this integrated source, focused run ID `b7c24a2e-44db-455e-a50c-4ce583f2c3df` ran 2 pass and 9 fail. `note_wrap_and_clip` remained red. Eight allocation tests that passed at exact base failed here:

- `termrock-navigation::item_row_columns::warmed_columns_paint_allocates_nothing`
- `termrock-navigation::list_row_renderer::full_row_renderer_allocates_nothing_after_warmup`
- `termrock-navigation::list_row_extent::warmed_multiline_publication_has_no_allocations`
- `termrock-navigation::nav_list_scrolling::sections_disabled_rows_and_warmed_redraw_use_authoritative_geometry`
- `termrock-navigation::tree_row_renderer::full_row_renderer_allocates_nothing_after_warmup`
- `termrock-navigation::tree_state_replacement::warmed_toggles_and_publication_allocate_nothing`
- `termrock-runtime::paint_matched::warmed_matched_paint_allocates_nothing`
- `termrock-runtime::paint_middle::warmed_borrowed_paint_allocates_nothing`

The full scoped run ID `a0d46de1-84be-4a12-953c-75e9a3b9f850` on the first stack is superseded by the runtime follow-up below. All three field continuation tests pass through the prepared-output test helper, including `redirected_edit_menu_closure_blurs_the_held_draft_once`.

`cargo clippy -p termrock-runtime -p termrock-session -p termrock-text -p termrock-fields -p termrock-navigation -p termrock-overlays --all-targets -- -D warnings` passed. `cargo fmt --all -- --check` exited 1 only for pre-existing formatting in `crates/showcase-demos/src/pages/chrome.rs`; `git diff 4a0ab4983..HEAD -- crates/showcase-demos/src/pages/chrome.rs` is empty. That unrelated file was not edited.

The initial runtime integration findings are superseded by the exact two-commit follow-up below. No application parity, external consumer, or architecture acceptance is claimed. The earlier a576-only comparison is superseded by the exact-base comparison above.

## Runtime follow-up

Parent runtime branch contributed exactly two commits on top of `6be6d55ee57b8e6eba1ebb80690c9908b38dcecc`:

1. `d18a4c9c13f2d7214ed96ff76b161d2a00546ab4` defers continuation receipts until focus settles. Independent review cleared its three findings.
2. `d11713bc68642aaf7854393806ba789fa80dbb9c` reuses prepared-frame buffers. Independent review remains active.

The first commit changes `crates/termrock-runtime/src/runtime.rs`, `crates/termrock-runtime/src/ui/cx.rs`, and `crates/termrock-test-support/src/harness.rs`; the second changes only `crates/termrock-runtime/src/runtime.rs` and the prepared-frame audit note.

Focused run ID `4f4b2efa-ae7b-4e0a-820a-3feb469816f9` exercised the eight prior allocation regressions plus the baseline note-color test: all eight allocation cases passed; only `note_wrap_and_clip` failed with the same RGB138 versus RGB128 assertion already present at exact base `4a0ab4983`.

Full scoped run ID `2d49f351-86fb-40cf-9200-b363666458f6` ran 697 tests: 696 passed, 1 failed, 4 skipped. The sole failure was `termrock-fields::note::tests::note_wrap_and_clip`, which also fails at exact base. Scoped clippy passed after the follow-up. Formatting remains red only in the untouched `crates/showcase-demos/src/pages/chrome.rs` baseline file.

The eight allocation regressions from the first stack are resolved by this two-commit follow-up. The earlier 9-failure results are retained above as superseded evidence; no failure was omitted or reclassified as baseline without comparison. `d11713bc68642aaf7854393806ba789fa80dbb9c` review, the baseline note-color assertion, and inherited formatting failure remain open.

## Reviewed stack integrated on the current shared-style base

This integration branch starts at `e61c16a06d28e6f68e37055923b605de6cd44f16`. That source already contains the reviewed shared-style fixes from `1a44bd04a7c90aceb76453b1b9411159d75cb706`, `8e9a7f59ee1072405aaee18a89f4d433485888ec`, and `941061f45cd2f043e0ef9cf4bdd928f796c17b15`; it also includes unrelated TablePro, Velnor, and audit changes made after `4a0ab4983`. The worktree was clean at the recorded base before cherry-picks. It contains the complete ordered runtime and fields chain from `a764ecea76c7309a60da1ef0c41ffe3a9973552f` through `081aececdba21c09e47c03472354c050eef90002`, including continuation follow-up `a64451325e33d9ec4cbfef9ec046830e63b68baf` and allocation fix `081aececdba21c09e47c03472354c050eef90002`, followed by evidence commit `801e6c94892703bde5309e3daa31f37c3a727910`. Session cursor-shape fix `5ca3c4f205aed4719d6558f2bbe9829a6321bdae`, note-color reference correction `f773fb8b230f165b21432093408a7998e9122b19`, and audit correction `c0c0f148b54c3459ee157de19dd49ca84bb3b447` were applied afterward. All cherry-picks applied without conflicts. No edits were made to `runtime/support.rs` or `ui/paint.rs`; their current shared-style/dialog state was retained.

At exact base `e61c16a06d28e6f68e37055923b605de6cd44f16`, isolated target `/private/tmp/runtime-fields-base-e61-target`, nextest run `f2e2dc36-6104-4384-aaad-6007da32e483` ran 772 tests: 764 passed, 8 failed, 5 skipped. Failures: `termrock-fields::note::tests::note_wrap_and_clip`; six `termrock-overlays::dialog::tests` (`a_prompt_dialog_sizes_its_own_field_row`, `a_growing_body_resizes_the_layer_on_the_next_frame`, `action_cap_is_shared_by_update_and_draw`, `draw_lays_out_against_the_height_it_asked_for`, `layer_size_is_a_pure_function_of_props_and_design_tokens`, `valid_dialog_preserves_chrome_and_clips_its_body`); and `termrock-test-support::perf_collections::picker_100k_borrowed_domain_render`.

At integrated source, isolated target `/private/tmp/runtime-fields-candidate-target`, nextest run `3db5061b-d94b-4ce3-88c4-36c25feba97a` ran 833 tests: 826 passed, 7 failed, 5 skipped. The same six Dialog and one large-picker allocation failures remain; `note_wrap_and_clip` now passes after the independently diagnosed RGB `(138,138,138)` frozen-reference correction. Thus no new failure appeared relative to the exact base; 61 additional tests were included. The skipped cases remain unverified. Dialog and large-picker failures stay recorded as blockers; this integration did not modify their owners.

After this audit and migration-matrix update were committed as `36a7ee5829d3d1c43e2732a088221d824cb154aa`, exact-HEAD nextest rerun `7b7109ab-ea88-4a62-a72d-43a9037f9349` repeated the same result: 833 tests, 826 passed, 7 failed, 5 skipped, with the identical six Dialog and one picker-allocation failures. This is the source branch's final pre-push test result.

After the follow-up evidence note was committed as `fa6a368b9b458966596ccac08befbe2de6bd5f6e`, exact-HEAD nextest run `617207bf-9177-451f-a298-d79a4af6bd03` again reported 833 tests, 826 passed, 7 failed, 5 skipped, with the same seven failures. Scoped strict Clippy, package formatting, and commit diff check passed at this source checkpoint.

On the integrated tree, strict package Clippy passed: `cargo clippy -p termrock-core -p termrock-runtime -p termrock-session -p termrock-test-support -p termrock -p termrock-fields -p termrock-text -p termrock-navigation -p termrock-overlays --all-targets -- -D warnings`. Scoped package formatting and `git diff --check e61c16a06d28e6f68e37055923b605de6cd44f16..HEAD` also passed. Full application/product acceptance, external-consumer acceptance, and all-skips-cleared acceptance remain open.

Focused comparison confirmed the inherited failures are exact: the six Dialog tests fail on both trees with matching assertion messages (`valid_dialog_preserves_chrome_and_clips_its_body`: `!inner.is_empty()`; `a_prompt_dialog_sizes_its_own_field_row`: actual `6`, expected `3`; other four match their baseline assertions). Base nextest run ID `ec4ad973-5d5a-4541-af38-29ac5799d3fa`; candidate run ID `ec49d092-7560-4282-9819-6f36242c5c38`. The borrowed-domain picker benchmark also fails identically on each tree: 188 allocations, 31,594 bytes. Base run ID `a83c80fc-9762-4075-8743-ecdd5d1bcbc8`; candidate run ID `5f712977-430c-45fa-ab1e-f07c99a3f398`. These exact-base comparisons verify inherited test failures only; they do not satisfy the remaining library gate.
