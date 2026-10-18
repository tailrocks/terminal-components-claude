# Rust comment-removal verification

## Status

The comment cleanup is not accepted. Rust comments were removed across the eight requested source branches, but required strict Clippy gates fail. No lint-policy changes were applied. Main workspace checks and tests pass; this does not waive the lint failures or establish completion.

## Authority

The requested source constraint was to remove Rust comments without changing product behavior. The active product comparison is `design/interaction-polish`; `main` remains the structural authority. This report records evidence only and does not authorize behavior changes, lint-policy changes, or baseline updates.

## Current evidence

Initial comment-removal commits preserved non-comment token streams. Final source commits then restored CLI help metadata as `#[doc = "…"]` attributes and applied reviewed formatting. These metadata attributes are source tokens, not comments. Therefore final source tokens are not claimed to equal the starting source. Formatting also changes line positions; no line-position preservation is claimed.

| Branch | Starting revision | Initial comment-removal commit | Final reviewed source commit | Rust files | Comments removed in initial pass |
|---|---|---|---|---:|---:|
| `main` | `d1122f69e6b9c2364aa6ede8174e7d35b8cfff39` | `b03f80527b73456fbad7d24ecbc54222a3f8157f` | `bcc0f9f9bd87c84fffafcab2c04f63bd5928dded` | 508 | 27,358 |
| `design/interaction-polish` | `d9e569bf6fd33d90266db597c458442c729340ad` | `55ae07eb2ee6ea0d5c5891889b45a2079d06e050` | `7b7cfa04dffdd46b57fa85fff4677dba75928107` | 418 | 25,373 |
| `wip/reach-pages-a-2` | `267a8d08575765f4e77a732eaa87aeec27986719` | `a820c80074c3dcfc6c571375b6ed7f02f0e6eff0` | `a76de88250eb3d40a4e88f97ec4f799882312a19` | 418 | 25,431 |
| `wip/rapps-3-7-14` | `81be99a79c5158a27713d4f6f0ed06f6f4e1faf0` | `79b2d5c10fa3926d3a579f65c5bd3cf42d54cc67` | `73c93bc7f86bf4348bc2fe94ce24b154a091e785` | 418 | 25,368 |
| `wip/reach-pages-a` | `f5b538137bc06624762a5901ecf0a16f63b931a9` | `0363f2f590b6368df46da5707fd3ffe63e264298` | `9f67330508a65391891a555725a515fdbe92fae5` | 418 | 25,426 |
| `wip/plan-s16` | `b249a81be807879212bd4bf50a9246aff5c18d60` | `a3569aca74f841d627ef26c908a346e0ba05bbf0` | `a9a18e7bf0211986131b8258cf94e86f90901368` | 410 | 23,541 |
| `wip/s14-followups` | `95f62ca33eaf7b02fc5a2551572fa3258b81a880` | `29a0ffd5d6513853966c88849cc4bfbd2dcbe65f` | `f548c7f74961341fedd8e6d871fd978cc4d3c012` | 414 | 23,849 |
| `wip/v1-showcase` | `c1a49af0f6ade36838c6330ce8a034e8d589ea79` | `92cd52ab010299a18ff5a2f04e942a4764a75527` | `8f58710690bb69f1ee0fbf53c99649425cb6414b` | 383 | 20,630 |

The seven final source commits after the main row are from `/tmp/termrock-doc-cleanup/final-source-commits.json`; each records CLI metadata preservation. Main’s final reviewed source includes the formatter commit and three comment-derived blank-line removals after attributes. The latter change alters whitespace only.

The final main formatting review found whitespace-only changes in 453 files and formatter syntax changes in 19 files: 18 files changed optional enum-field trailing commas; `crates/holla-sim/src/catalogue.rs` lost redundant closure braces while retaining the same expression. The exception set is recorded below. No blanket final token-equality claim applies.

| File | Reviewed formatter change |
|---|---|
| `crates/holla-sim/src/catalogue.rs` | Removed redundant closure braces; expression remains `workdir.rsplit('/').next().map(str::to_owned)`. |
| `crates/jackin-preview-capsule-ui/src/screens/op_flow.rs` | Optional enum-field trailing commas only. |
| `crates/jackin-preview-domain/src/account.rs` | Optional enum-field trailing commas only. |
| `crates/jackin-preview-domain/src/workspace.rs` | Optional enum-field trailing commas only. |
| `crates/jackin-preview-sim/src/arbiter.rs` | Optional enum-field trailing commas only. |
| `crates/jackin-preview-sim/src/changes.rs` | Optional enum-field trailing commas only. |
| `crates/jackin-preview-sim/src/launch.rs` | Optional enum-field trailing commas only. |
| `crates/jackin-preview-sim/src/onepassword.rs` | Optional enum-field trailing commas only. |
| `crates/jackin-preview-sim/src/world.rs` | Optional enum-field trailing commas only. |
| `crates/tablepro-domain/src/model.rs` | Optional enum-field trailing commas only. |
| `crates/tablepro-sql/src/sql.rs` | Optional enum-field trailing commas only. |
| `crates/tablepro-ui/src/app.rs` | Optional enum-field trailing commas only. |
| `crates/termrock-conformance/tests/visual_baseline/support/typed_input.rs` | Optional enum-field trailing commas only. |
| `crates/termrock-editors/src/code.rs` | Optional enum-field trailing commas only. |
| `crates/termrock-editors/src/diff.rs` | Optional enum-field trailing commas only. |
| `crates/termrock-grid/src/grid.rs` | Optional enum-field trailing commas only. |
| `crates/termrock-runtime/src/keymap.rs` | Optional enum-field trailing commas only. |
| `crates/termrock-runtime/src/layer.rs` | Optional enum-field trailing commas only. |
| `crates/termrock-terminal/src/terminal_view.rs` | Optional enum-field trailing commas only. |

## Remaining work

- Resolve the required strict Clippy failures without changing lint policy. Main reports missing crate documentation at `crates/tablepro-demo/src/lib.rs` and `clippy::collapsible_if` in `crates/termrock-overlays/src/dialog.rs`; design reports two `clippy::collapsible_if` findings in `src/widgets/field_common.rs` and `src/widgets/menu.rs`. The nested-if diagnostics concern unchanged logic exposed by the strict gate after comment removal.
- Rerun required checks on every final source commit and record exact results. Main’s workspace check and test runs pass; they do not replace Clippy.
- Keep the cleanup unaccepted until all required gates pass. Do not change lint configuration to suppress the failures.

## Verification

Main `cargo check --workspace --offline`, `cargo test --workspace --offline`, and formatting checks pass in the recorded runs. Required strict Clippy does not pass for the findings above; the repository lint configuration was not changed.

The seven legacy visual-baseline checks produced five passes and two failures. The failures are `wip/plan-s16` and `wip/reach-pages-a`; both match their original branch outputs byte-for-byte, as recorded by `/tmp/termrock-doc-cleanup/visual-baseline-drift-proof.json` (`unchanged_render_output: true`). No baseline was updated. The other five branches passed their selected visual-baseline test.

Initial lexical verification used `rustc_lexer 0.1.0`; its raw-C literal limitation remains. Independent review found no raw-C literals in the inspected corpus. Temporary source lists, logs, and proof files are under `/tmp/termrock-comment-cleanup` and `/tmp/termrock-doc-cleanup`; they are evidence locations, not repository dependencies.
