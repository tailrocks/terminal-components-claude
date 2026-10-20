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

All cherry-picks applied without conflicts. Relative to `4a0ab4983`, the selected source changes are limited to `crates/termrock-core/src/diagnostics.rs`, `crates/termrock-fields/{src/edit.rs,src/input.rs,src/lib.rs,src/textarea.rs}`, `crates/termrock-runtime/{src/author.rs,src/layer.rs,src/runtime.rs,src/runtime/clipboard.rs,src/ui/cx.rs,tests/monotonic.rs}`, `crates/termrock-session/src/session.rs`, `crates/termrock-test-support/src/harness.rs`, `crates/termrock-text/{src/buffer.rs,src/editor.rs}`, and `crates/termrock/src/lib.rs`. The only local test changes adapt the two field `publish()` helpers to use `Runtime::prepare`, `draw_prepared` through Ratatui `TestBackend`, and `acknowledge_presented`; production source is not changed locally.

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

The full scoped run ID `a0d46de1-84be-4a12-953c-75e9a3b9f850` ran 696 tests: 687 passed, 9 failed, 4 skipped. Its failures are the same nine cases above. All three field continuation tests pass through the prepared-output test helper, including `redirected_edit_menu_closure_blurs_the_held_draft_once`. That test-only result does not clear the independent runtime review, which still requires acknowledgement, redirected-focus, and field-retention fixes.

`cargo clippy -p termrock-runtime -p termrock-session -p termrock-text -p termrock-fields -p termrock-navigation -p termrock-overlays --all-targets -- -D warnings` passed. `cargo fmt --all -- --check` exited 1 only for pre-existing formatting in `crates/showcase-demos/src/pages/chrome.rs`; `git diff 4a0ab4983..HEAD -- crates/showcase-demos/src/pages/chrome.rs` is empty. That unrelated file was not edited.

The eight allocation regressions and baseline note-color assertion remain open. Runtime-stage review for `6be6d55ee57b8e6eba1ebb80690c9908b38dcecc` is pending. No application parity, external consumer, or architecture acceptance is claimed. The earlier a576-only comparison is superseded by this exact-base comparison.
