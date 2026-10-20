# Branch consolidation audit (in progress)

Inventory snapshot: **96 remote branches**, observed 2026-10-20 16:07:32 UTC. The JSON companion records each exact starting SHA and audit assignment. A–E reports now cover all 96 starting branches exactly once. A fresh remote inventory found one additional branch, `wip/pr38-palette-reviewed-20261020` at `636028fa130b2a0f841badb3ba93aa2330b907d6`, protected after the original inventory; its source review and base attribution are complete; PR #38 integration remains pending. PR #38 advanced from `6bcdc421` to `abfff94e`; run 39455259301 was in progress when recorded.

The external companion at `E-ci-reference-catchall.json` contains all 97 live branch rows, the exact 96-branch starting inventory, and the A–E reports. It preserves each reviewer’s branch-level status and evidence. Reviewed sources can still be blocked, red, superseded, or pending integration; review does not equal merge or acceptance.

This per-branch ledger supersedes coarse branch summaries in the migration matrix. It preserves bounded component work for selective integration even when an entire branch or an application parity gate remains unaccepted. The shared-style 94a/8e9/e72 work is represented by already-integrated 2863/e597 equivalents. Initial Button commit 7dab is superseded by the reviewed 673936/b0f8/2d stack. The full findings and source hashes remain in the branch reports.

## E findings

PR #38 already pins Velnor Actions **0.1.29** at source commit `e7b2f088715379c22be473fab896a43d50cc16fc`, and `zizmor` 1.31.1. Its config sets `claude_md = "symlink"`, `crate_timeout_minutes = 15`, and excludes `tools/frozen-cli-frame-support/**`; generated `.github/AGENTS.md` identifies 0.1.29, and `.github/CLAUDE.md` is a symlink. This is candidate-specific generated output. Do not replace it with workflows generated for another branch's crate graph.

| Branch | Starting SHA | E disposition | Evidence and reason |
|---|---|---|---|
| `chore/rust-comment-cleanup` | `0b1142a2` | Exclude | Three commits touch 492 Rust files to remove or reflow comments on the legacy tree. No unique component or parity behavior; broad documentation loss and source churn. |
| `ci/design-rust-timeout-20261020` | `f2f6eb71` | Reference duplicate | Exact design-reference head. Its 0.1.29 workflow generation and 15-minute crate timeout are not a PR #38 delta. |
| `ci/velnor-0.1.29-main-20261020` | `a6929aeb` | Separate delivery, blocked | Main-only upgrade belongs to PR #39. Latest run 39436987238 passed Actionlint, then Plan exceeded 5m0s, Rust jobs skipped, and Required failed. PR #38 already has 0.1.29. |
| `ci/zizmor-design-20261020` | `4860d035` | Superseded reference duplicate | Design head already contains the zizmor 1.31.1 pin. |
| `design/interaction-polish` | `f2f6eb71` | Protected reference | PR #26 is read-only; never merge, modify, close, or delete. |
| `main` | `a97e351d` | Protected base | No direct integration. Main's workflow upgrade is tracked separately in PR #39. |
| `parity/ci-planning-fix-20261020` | `8beaf286` | Duplicate, generated output obsolete | Symlink configuration and frozen CLI helper exclusion already exist in PR #38. Its generated output targets Velnor 0.1.16 and the older tree. |
| `parity/runtime-crate-timeout-15-20261020` | `4d0df770` | Already integrated | Branch tip is an ancestor of the PR #38 inventory head; 15-minute crate timeout is present. |
| `parity/termrock-visual-baseline` | `6bcdc421` | Integration target | Target itself; advanced through historical intermediate `0cd5edb8` to `abfff94e` during the audit. |
| `parity/velnor-design-upgrade-20261020` | `85333445` | Reference duplicate | Generated for the design branch's tree. PR #38 has its own 0.1.29 output and crate graph. |
| `parity/velnor-parity-20261020` | `e7961cee` | Superseded duplicate | Earlier 0.1.29 generation. Its older config would drop PR #38's current 15-minute timeout and discovery exclusions. |
| `wip/shared-design-working-tree-20261020` | `1d7fdb22` | Exclude and preserve | Snapshot based on design reference; 163 changed files and 12,624 deletions against design head. Not safe to merge. Keep branch pending its own disposition. |

## D findings

D reviewed 15 branches. No wholesale merge was recommended. Architecture checker revisions are superseded by exact final files already in PR #38; frozen-table workflow revisions are stale and their candidate comparisons remain red; docs and working-tree snapshots are excluded or require manual reconciliation. `parity/buttons-reference-decision-20261020` and `parity/ordinary-cli-partition-20261020` are selective tool-port candidates only. `wip/style-conformance-20261020` requires component/runtime review. Preserve all red candidate evidence; no D finding establishes app acceptance. Branch-by-branch hashes, ancestry counts, findings, and scopes are in the JSON companion.

## A, B, and C findings

A reviewed 23 branches. Four are bounded integration candidates, four are already integrated, 14 are superseded, and the dialog dismissal stack is blocked by a reproduced P1: a second close of a reopened layer generation loses its `CANCEL` action. Keep the dismissal integration blocked until the generation-specific fix passes independent probes.

B reviewed 23 branches. It staged selective widget/runtime integrations and recorded architecture gaps. The Button source has known paired visual differences; composition and master-detail contracts still need proof in real application flows. The post-inventory palette source was protected as `636028fa`; parent follow-up commit `082ad5d3ab3b4b29e790f7b3e4ea21205a5719a0` reports four unchanged source/test blobs independently clear after attributing the full 19-crate failure set. Focused strict Clippy and formatting passed. Source review is clear; PR #38 integration remains pending.

C reviewed 23 branches. TablePro picker and live-contract work, Showcase table work, and selected shared component migrations are staged behind dependencies and paired app proof. Archived monoliths, stale copies, and test-tool-only changes are excluded from app integration. `wip/termrock-reviewed-contract-integration-20261020` is routed to B for selective Button source review; its useful work is not blanket-rejected, and its whole branch is not approved.

## Verification limits

E heads were checked directly with `git ls-remote`; all initial rows matched the inventory capture. The refreshed inventory records one added palette WIP and the moved PR #38 head. PR #39's Plan timeout and Required failure were checked through GitHub run 39436987238 and check-run annotations. A–D performed read-only reviews and focused probes as recorded in their source reports; those reports do not constitute full application acceptance. No generated files were hand-edited. PR #38's latest CI run was still running at observation time, so no result is claimed. Product parity and the reusable-library gate remain incomplete.
