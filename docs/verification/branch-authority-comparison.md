# Branch authority comparison

## Status

Documentation and comment cleanup in progress. This comparison records the starting revisions, not completed product acceptance.

## Authority

`main` supplies the split-crate structural direction. Its refactor remains broken, incomplete, and not ready as a product. `design/interaction-polish` supplies the accepted visual appearance, UI/UX, colors, and behavior. Its monolithic structure must be replaced by the split-crate architecture. Better API ideas from that branch remain eligible for reuse within those boundaries.

## Current evidence

Starting revisions inspected on 2026-10-18. Counts cover tracked paths, including independent tooling and excluded packages; they are not workspace-member counts.

| Branch | Starting revision | Cargo manifests | Markdown files | Rust files |
| --- | --- | ---: | ---: | ---: |
| `main` | `d1122f69e6b9c2364aa6ede8174e7d35b8cfff39` | 50 | 116 | 508 |
| `design/interaction-polish` | `d9e569bf6fd33d90266db597c458442c729340ad` | 4 | 737 | 418 |
| `wip/reach-pages-a-2` | `267a8d08575765f4e77a732eaa87aeec27986719` | 4 | 727 | 418 |
| `wip/rapps-3-7-14` | `81be99a79c5158a27713d4f6f0ed06f6f4e1faf0` | 4 | 727 | 418 |
| `wip/reach-pages-a` | `f5b538137bc06624762a5901ecf0a16f63b931a9` | 4 | 727 | 418 |
| `wip/plan-s16` | `b249a81be807879212bd4bf50a9246aff5c18d60` | 4 | 724 | 410 |
| `wip/s14-followups` | `95f62ca33eaf7b02fc5a2551572fa3258b81a880` | 4 | 724 | 414 |
| `wip/v1-showcase` | `c1a49af0f6ade36838c6330ce8a034e8d589ea79` | 4 | 724 | 383 |

The modular branch declares shared-library and application crate families. Design and WIP trees retain four package manifests. Manifest counts establish structural differences, not functional correctness.

WIP documentation differs primarily in backlog, audits, CI guidance, and handoff state. Missing backlog entries do not prove completed work. Preserve showcase reachability, responsive-layout work, API and shared-component plans, visual-test plans, and application follow-ups until their acceptance criteria are verified.

## Remaining work

Finish documentation reconciliation and lexical comment removal across every listed branch. Preserve branch-specific planned requirements. Record independent review and current checks before claiming completion.

## Verification

Inventory command: `git ls-tree -r --name-only <starting-revision>`. Compare files by blob identity before transferring shared edits; reconcile changed files separately. Compare Rust non-comment token kinds and text before and after comment removal, then check compilation and relevant tests. Compilation alone does not prove visual or behavioral parity.
