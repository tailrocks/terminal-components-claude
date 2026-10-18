# Rust comment-removal verification

## Status

Cleanup remains incomplete. Initial removal commits exist on all eight worktrees, but design verification found changed Clap help text because doc comments are macro inputs. Main workspace compilation and tests pass; those results did not catch this behavior dependency. Resolve the consumed metadata before acceptance.

## Authority

Source code changes in this cleanup remove comments only. Public code tokens, visuals, and functionality must remain unchanged. Documentation describes branch authority separately in the [comparison](branch-authority-comparison.md).

## Current evidence

Verification date: 2026-10-18. The temporary verifier uses `rustc_lexer 0.1.0`, compares each non-comment/non-whitespace token kind and its exact text against the starting revision, and rejects remaining lexical comments. File lists come from `git ls-files '*.rs'`, covering crate sources, tests, examples, build scripts, and Rust tooling without traversing other worktrees.

| Branch | Starting revision | Comment-removal commit | Tracked Rust files | Removed comments |
| --- | --- | --- | ---: | ---: |
| `main` | `d1122f69e6b9c2364aa6ede8174e7d35b8cfff39` | `b03f80527b73456fbad7d24ecbc54222a3f8157f` | 508 | 27358 |
| `design/interaction-polish` | `d9e569bf6fd33d90266db597c458442c729340ad` | `55ae07eb2ee6ea0d5c5891889b45a2079d06e050` | 418 | 25373 |
| `wip/reach-pages-a-2` | `267a8d08575765f4e77a732eaa87aeec27986719` | `a820c80074c3dcfc6c571375b6ed7f02f0e6eff0` | 418 | 25431 |
| `wip/rapps-3-7-14` | `81be99a79c5158a27713d4f6f0ed06f6f4e1faf0` | `79b2d5c10fa3926d3a579f65c5bd3cf42d54cc67` | 418 | 25368 |
| `wip/reach-pages-a` | `f5b538137bc06624762a5901ecf0a16f63b931a9` | `0363f2f590b6368df46da5707fd3ffe63e264298` | 418 | 25426 |
| `wip/plan-s16` | `b249a81be807879212bd4bf50a9246aff5c18d60` | `a3569aca74f841d627ef26c908a346e0ba05bbf0` | 410 | 23541 |
| `wip/s14-followups` | `95f62ca33eaf7b02fc5a2551572fa3258b81a880` | `29a0ffd5d6513853966c88849cc4bfbd2dcbe65f` | 414 | 23849 |
| `wip/v1-showcase` | `c1a49af0f6ade36838c6330ce8a034e8d589ea79` | `92cd52ab010299a18ff5a2f04e942a4764a75527` | 383 | 20630 |

Initial removal preserved non-comment token streams. Required Rust formatting subsequently changed only whitespace in 453 main files and redundant punctuation in 19 main files, as reviewed independently below. Independent review scanned all 508 main files and 2,879 design/WIP files: zero comments, unchanged non-comment tokens and file sets. Whitespace checks pass for Rust diffs. Fixtures cover nested block comments, doc comments, raw strings, byte/raw-byte strings, ordinary C strings, lifetimes, raw identifiers, Unicode, CRLF, and shebangs. `rustc_lexer 0.1.0` does not safely handle every raw-C literal; independent review found none in the inspected source corpus. This verifier must not be generalized to source containing raw-C literals without upgrading its lexer. Existing code line positions remain preserved; trailing comment-only EOF lines are removed. No `line!()` or `column!()` consumers were found in tracked crate code. Formatting changes source line positions; source-identity receipts necessarily change.

Removing doc comments intentionally removes generated Rust API prose and doctest examples. Existing lint settings remain unchanged; `missing_docs` warnings are expected. The visibility audit embeds and hashes its own Rust source, so its source-identity receipt changes. This is not evidence of a visual or interaction change. Historical receipts retain their original scope.

### Formatting review

`cargo fmt --all` removes comment-derived blank lines. Independent review compared all 472 affected main files with `dc9b1e42c`: 453 retained identical non-whitespace tokens. Eighteen files only changed optional trailing enum-field commas (40 removed, seven added). One expression closure lost redundant block braces around its unchanged return expression. These are formatter changes with no new executable logic; the following list bounds every punctuation exception.

| File | Reviewed formatting exception |
| --- | --- |
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

Finish design checks and complete documentation review before declaring the whole task complete. Product refactor readiness remains incomplete regardless of cleanup checks.

## Verification

Main: `cargo check --workspace --offline` passed before and after comment removal. `cargo test --workspace --offline` passed after removal. Design full nextest run: 3,281 passed, one `app_tests::showcase_visual_baseline` failure. The unchanged original design revision passes that test when run alone. Eight snapshot rows differ on “Status & hints”; Clap consumes removed doc comments as visible help text. Token equality after dropping doc comments is insufficient proof of semantic equality. No baseline was updated. Tests on one branch do not establish acceptance on another.

Session proof artifacts are under `/tmp/termrock-comment-cleanup` (tracked file lists, apply reports, token comparisons and whitespace checks), with build/test logs under `/tmp/termrock-doc-cleanup`. Immutable commit pairs above permit reproducing the source comparison independently; temporary log paths are not permanent project dependencies.
