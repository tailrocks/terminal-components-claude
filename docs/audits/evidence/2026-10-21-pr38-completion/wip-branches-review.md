# WIP branch review against PR #38

Pinned PR #38 tip: `7649ee7ed8e8645e49f859a171ea0193fccc1e82`
(`origin/parity/termrock-visual-baseline`, also current HEAD at review start).

Method: `git cherry <pinned> <branch>` (`-` = patch already present in pinned tip,
`+` = patch unique), `merge-base --is-ancestor`, `git show --stat`, then source
inspection at the pinned tip. Read-only; nothing merged or checked out.

Branches reviewed: 54 (`origin/wip/*`).

## Table

| branch | tip | unique commits | evidence in PR38 | recommendation | reason |
|---|---|---|---|---|---|
| baseline-cohort-admission-20261021 | 310a56de6 | 0 | tip is ancestor of `7649ee7` | already-integrated | fully contained |
| ci-doc-inventory-20261021 | 482cdaf48 | 0 | tip is ancestor | already-integrated | fully contained |
| ci-upgrade-handoff-20261020 | 706c13833 | 2 | docs only (`docs/handoff/README.md`, migration matrix); no code | exclude | stale handoff notes, 996 commits behind; nothing executable |
| context-menu-live-contract-20261021 | 1dee00913 | 0 | tip is ancestor | already-integrated | fully contained |
| controls-adopt-rate-limit-20261021 | d90357899 | 0 | equivalent `50b17ce07` in PR38 | already-integrated | cherry `-` |
| domain-seeded-lint-20261021 | c9357a9ef | 0 | tip is ancestor | already-integrated | fully contained |
| grid-style-inheritance-c29 | 4f9cbb0df | 0 | tip is ancestor | already-integrated | fully contained |
| holla-rate-limit-20261021 | 098fcfcda | 7 | `5b1b3bf9b` (band views/tag frames); `catalogue.rs` and `home.rs` byte-identical to tip | already-integrated | outcome landed, reworked/squashed |
| itemrow-metadata-contract-20261021 | 32628b360 | 3 | absent: tip `crates/termrock-navigation/src/item.rs:250` has only `ItemRow`; no `SemanticItemRow`, no `RowUi::meta_with_gap` | integrate | additive public row API + gap guard + tests; branch only 4 behind |
| jackin-live-capsule-menu-20261021 | de819fb1c | 1 | absent: tip `crates/jackin-preview-app/src/app.rs:8407` still calls `capsule_menu_view::paint` | needs-review | 168 behind, 779-line app.rs refactor; may conflict with `300a97bf8` |
| jackin-rate-limit-20261021 | 01dcfddde | 1 | absent: tip `app.rs:11157` still page-paints info card; `dialog.rs:440` has `head_in_border` but app not adopted | needs-review | 342 behind, in-flight WIP, needs rebase and visual proof |
| list-multiline-viewport-20261021-v2 | 7126662c9 | 0 | equivalent `4e6e0c123` | already-integrated | cherry `-` |
| manager-composition-interop-20261021 | 621997e7d | 3 | `d4b6021f2`; `composition.rs`, `master_detail.rs`, test and example hashes identical to tip | already-integrated | landed verbatim |
| master-detail-block-stage1-20261021 | e00262786 | 0 | tip is ancestor | already-integrated | fully contained |
| master-detail-pure-layout-20261021 | 912681032 | 0 | tip is ancestor | already-integrated | fully contained |
| menu-cursor-theme-contract-20261021 | 02dde9678 | 2 | absent: `crates/termrock-overlays/src/menu.rs:1030-1033` still excludes `StateFlags::ACTIVE`; no `menu_cursor_modifiers` in theme | needs-review | real frozen-baseline cursor fix + 4-level tests; 56 behind, theme conflicts likely |
| menu-disabled-reason-20261021-a7f2 | 38148abaf | 0 | tip is ancestor | already-integrated | fully contained |
| menu-underlay-fill-20261021 | 098aacd50 | 6 | absent: `crates/termrock-runtime/src/ui/layer_buf.rs:58-95` has no `UnderlayOp`/underlay style replay; no `tests/underlay_stack.rs` | needs-review | large runtime contract change (fade ink/order), 111 behind; must be re-proven |
| nextest-doc-inventory-guard-20261021 | fcb0d8d2f | 0 | tip is ancestor | already-integrated | fully contained |
| owned-fresh-plan-lifecycle-20261021 | 65796fa5a | 14 | absent: `tools/frozen_baseline.py:3465 create_fresh_plan` has no deadline/root-binding/owned receipt; no `_create_fresh_run_owned` | needs-review | verification hardening only, but 2271-line drift in `frozen_baseline.py`; rebase + full tool tests |
| palette-grid-repair-20261021 | b26f54872 | 2 | `7cc6835ff` and `dfff9d5e6` (same subjects, matching stats); grid fade at `termrock-grid/src/grid.rs:3719`, seam tests `termrock-theme/src/lib.rs:792` | already-integrated | landed reworked |
| palette-rate-limit-20261021 | 73fc468fc | 0 | equivalent `b3e228210` | already-integrated | cherry `-` |
| palette-test-integrity-20261021 | c60830f8a | 0 | tip is ancestor | already-integrated | fully contained |
| parent-fields-runtime-blocker-20261021 | 379465e9e | 0 | tip is ancestor | already-integrated | fully contained |
| plan-s16 | 70e04be37 | 879 | old `src/` + `xtask/` layout; no `crates/termrock-*` | exclude | pre-rewrite lineage (diverged 2026-09-03) |
| pr38-palette-style-integration-20261020 | b0a3bea5a | 1 | docs conflict record only; no code | exclude | documentation-only, 995 behind |
| public-consumer-contracts-20261021 | fc861e15a | 1 | absent: `crates/termrock/tests/public_component_contracts.rs` does not exist; APIs it uses do exist (`button.rs:156`, `filter_list.rs:56/364`) | needs-review | test-only and additive, but 270 behind; adapt names then gate |
| radio-navigation-contract-20261021 | 1b677b30c | 1 | `804ed9ae1` (radio policy) + `f0760cc20` (blank draft, ancestor); the one `+` is a duplicate blank-draft patch | already-integrated | cherry `-` for the rest |
| rapps-3-7-14 | b76517cec | 1054 | old `src/` layout | exclude | pre-rewrite lineage |
| reach-pages-a | 3cb2198dd | 1060 | old `src/` layout | exclude | pre-rewrite lineage |
| reach-pages-a-2 | 37e21b8a4 | 1064 | old `src/` layout | exclude | pre-rewrite lineage |
| recolor-provenance-20261021 | dfb54cfc5 | 0 | tip is ancestor | already-integrated | fully contained |
| s14-followups | 19fcb24b6 | 882 | old `src/` layout | exclude | pre-rewrite lineage |
| shared-design-working-tree-20261020 | 1d7fdb224 | 1052 | old `src/` + `xtask/` layout; snapshot deletes 12k lines | exclude | pre-rewrite snapshot |
| shell-reach-rustdoc-20261021 | 92bd50821 | 0 | tip is ancestor | already-integrated | fully contained |
| split-block-contract-20261021 | 759d0d411 | 0 | tip is ancestor | already-integrated | fully contained |
| style-conformance-20261020 | c2396b456 | 2 | picker title behavior at `termrock-theme/src/builtin/mod.rs:657-670` + test `:1037`; second commit docs only | already-integrated | code reworked into tip, remainder handoff note |
| table-consumer-baseline | fa02fa54a | 1 | `18b141dfa`; horizontal scroll present at `termrock-grid/src/table.rs:71/344` | already-integrated | landed reworked (14 other patches cherry `-`) |
| tablepro-blank-database-validation-20261021 | 8992262f4 | 1 | `e70f362db`; blank-draft behavior covered in `tablepro-ui/src/app/connection_form_focus.rs:9` | already-integrated | landed |
| tablepro-blank-domain-20261021 | f0760cc20 | 0 | tip is ancestor | already-integrated | fully contained |
| tablepro-name-prefix-20261021 | c8f92ea0c | 2 | `e70f362db` (blank draft) + `6b639c4dd` (name prefix) | already-integrated | both behaviors in tip |
| tablepro-picker-scope-20261021 | d40f8e5fb | 3 | `3c215786c` (picker query retention); remaining 2 commits docs handoff only | already-integrated | code landed; docs-only remainder |
| tablepro-query-focus-contract-20261021 | 3bab23ecf | 3 | absent: `tablepro-ui/src/app.rs:666` still `focus_new_query`, `:10150` still 120x40 hardcode; no `CodeEditor::commit_on_blur` | integrate | parity fix + tests; only 2 behind, clean rebase expected |
| tablepro-strict-clippy-20261021 | 8ec5d042e | 0 | equivalent `391d42bfa` | already-integrated | cherry `-` |
| terminal-view-input-owner-20261021 | eb068f3a5 | 0 | tip is ancestor | already-integrated | fully contained |
| termrock-extract-shell-20261021 | 9892d22a0 | 0 | tip is ancestor | already-integrated | fully contained |
| termrock-reviewed-contract-integration-20261020 | 8462bf28a | 4 | `f726ee673`, `4352776ff`, `53889e82a`, `398d53f5d` (same subjects); busy/focus button code at `termrock-controls/src/button.rs:87` | already-integrated | landed reworked |
| theme-field-hover-20261021 | 4507e7de2 | 0 | tip is ancestor | already-integrated | fully contained |
| tree-composition-adapter-20261021 | 92c4ac9b7 | 0 | tip is ancestor | already-integrated | fully contained |
| typed-dialog-props-slots-20261021 | 6cc98ab0f | 0 | tip is ancestor | already-integrated | fully contained |
| v1-showcase | 68651fba7 | 783 | old `src/` layout | exclude | pre-rewrite lineage |
| velnor-actions-0135-20261021 | dc5ade35e | 2 | no 0.1.35 adoption in PR38 (still 0.1.32, `mise.toml:9`, `ci.yml:1`) | exclude | superseded by 0.1.36 branch |
| velnor-actions-0136-20261021 | 00034e59d | 1 | no 0.1.36 adoption in PR38; tip still 0.1.32 | integrate | required latest pin; regenerate, run CI config gate |
| viewport-contract-repair-20261021 | 8a8172008 | 0 | tip is ancestor | already-integrated | fully contained |

## Needs action

### Integrate (3)

1. `origin/wip/itemrow-metadata-contract-20261021`
   - `d378405cf43936b8e057e524f4ed69a6e0c46858` — add `SemanticItemRow`, `ItemLabelOverflow`, `RowUi::meta_with_gap`
   - `e8cc0197e54836bbadfb2df1e200bf79cd7912c4` — `Tree::row_adapter`, keep closure `row`
   - `32628b360f6015fe4ef49c805c0e45e8264a874d` — checked metadata-gap budget
   - Next: rebase 4 commits onto `7649ee7`, run `cargo nextest run -p termrock-navigation -p termrock-runtime`, clippy/fmt, then push.

2. `origin/wip/tablepro-query-focus-contract-20261021`
   - `a6cb8a4ff73ca2c7b244a12ba1efd8adee2f86d4` — focus new query editor at every viewport
   - `b8a33979e66370736664e82f404834d73cdd0e28` — deliver focus-out to inactive editors
   - `3bab23ecfcc9cedbdea22700328737282dfa06ae` — `CodeEditor::commit_on_blur`
   - Next: rebase 2 commits behind, run tablepro-ui + termrock-editors nextest and the 120x40 parity capture.

3. `origin/wip/velnor-actions-0136-20261021`
   - `bfd5c74b390df83792c21aeb6d59ddd1beae379e` — generated workflows 0.1.36
   - Next: rebase 4 behind, re-run `velnor-actions generate` from `.velnor/config.toml`, confirm `mise.toml` pin and `.velnor/release-manifest.json` agree, run CI config/actionlint gate.

### Needs review before integration (6)

1. `menu-cursor-theme-contract`: `4d84ff9a2082075510708aedec1be76758001465`, `02dde9678d3d2d3a6940659025a6551735bdb8d7`.
   Add `ACTIVE` to the context-menu cursor set and per-level cursor modifiers. Rebase 56 behind; verify frozen menu captures at truecolor/256/16/mono, not just the new unit test.
2. `menu-underlay-fill`: `860529a316`, `bb012956690`, `28a5d2323f`, `cfd2304fb1`, `5ac1ae24b9`, `098aacd503`.
   Runtime underlay ink/order replay. Rebase 111 behind; re-run `underlay_stack` plus every visual site that fades under an overlay.
3. `owned-fresh-plan-lifecycle`: `4afcc6e63d`, `0775a750ac`, `48a4b21a2f`, `17abfae6e3`, `3f8ca9b9e7`, `b30b7cc77f`, `8c0ae36c3f`, `7328b530d0`, `14bfb3891b`, `014d371620`, `1dad60ad0e`, `489beb371a`, `7a4b433587`, `65796fa5a0`.
   Rebase the 14 commits onto the 317 KB `frozen_baseline.py`; run `pytest tools/test_frozen_baseline.py` and a fresh plan create/verify cycle on the rebased tree.
4. `jackin-live-capsule-menu`: `de819fb1ca4e30790e869b1b9eed004109cab3aa`.
   Delete `capsule_menu_view.rs` and drive capsule context menus from live widgets. 168 behind; re-run jackin visual groups at all frozen widths.
5. `jackin-rate-limit`: `01dcfddde88fac6cb854fcc1dc0f2c9b6d7fdf8d`.
   Route cockpit info card through the live dialog + `info_card::paint_facts`. 342 behind; needs red-green visual proof at 256/16/mono.
6. `public-consumer-contracts`: `fc861e15af48089fc606e83d3b7bf4bf28b0aacf`.
   Add `crates/termrock/tests/public_component_contracts.rs` + Cargo target. 270 behind; rebase, adapt to current `termrock` re-exports, run with `--features testing`.

### No action

- 21 branches are direct ancestors of `7649ee7`: fully integrated.
- 4 branches have only patch-equivalent commits: `controls-adopt`, `list-multiline-viewport-v2`, `palette-rate-limit`, `tablepro-strict-clippy`.
- 7 branches belong to the pre-rewrite `src/` lineage: `plan-s16`, `rapps-3-7-14`, `reach-pages-a`, `reach-pages-a-2`, `s14-followups`, `shared-design-working-tree-20261020`, `v1-showcase`.
- Doc/handoff-only branches: `ci-upgrade-handoff-20261020`, `pr38-palette-style-integration-20261020`; `style-conformance-20261020` and `tablepro-picker-scope-20261021` have only documentation left after their code landed.
- `velnor-actions-0135-20261021` is superseded by the 0.1.36 branch.
