# Branch consolidation audit (in progress)

Inventory snapshot: **96 remote branches**, observed 2026-10-20 16:07:32 UTC. The JSON companion records each exact starting SHA and audit assignment. Validation found 96 rows, 96 unique branch names, and 12 E-assigned branches. Fresh `git ls-remote` checks at 16:10 UTC matched all 12 E heads. The remote PR #38 head advanced during review from inventory snapshot `6bcdc42173232b53ed48b221ae8d099791283e39` to `0cd5edb8856396bde1bbd2f8a5377940a3d50447`; latest run 39455126168 was still in progress when recorded.

E (CI, reference, and catchall) and D (tests and proof) reports are recorded. A–C rows remain explicitly pending; this record makes no approval claims for them. The external copy at `E-ci-reference-catchall.json` includes all 96 branch rows and both completed reports. Detailed D evidence remains linked from that JSON to the captured evidence directory.

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
| `parity/termrock-visual-baseline` | `6bcdc421` | Integration target | Target itself; advanced to `0cd5edb8` during the audit. |
| `parity/velnor-design-upgrade-20261020` | `85333445` | Reference duplicate | Generated for the design branch's tree. PR #38 has its own 0.1.29 output and crate graph. |
| `parity/velnor-parity-20261020` | `e7961cee` | Superseded duplicate | Earlier 0.1.29 generation. Its older config would drop PR #38's current 15-minute timeout and discovery exclusions. |
| `wip/shared-design-working-tree-20261020` | `1d7fdb22` | Exclude and preserve | Snapshot based on design reference; 163 changed files and 12,624 deletions against design head. Not safe to merge. Keep branch pending its own disposition. |

## D findings

D reviewed 15 branches. No wholesale merge was recommended. Architecture checker revisions are superseded by exact final files already in PR #38; frozen-table workflow revisions are stale and their candidate comparisons remain red; docs and working-tree snapshots are excluded or require manual reconciliation. `parity/buttons-reference-decision-20261020` and `parity/ordinary-cli-partition-20261020` are selective tool-port candidates only. `wip/style-conformance-20261020` requires component/runtime review. Preserve all red candidate evidence; no D finding establishes app acceptance. Branch-by-branch hashes, ancestry counts, findings, and scopes are in the JSON companion.

## Verification limits

E branch heads were checked directly with `git ls-remote`. PR #39's Plan timeout and Required failure were checked through GitHub run 39436987238 and check-run annotations. D performed a separate read-only history/source review; it did not run full checked-out builds. No generated files were hand-edited. PR #38's latest CI run was still running at observation time, so no result is claimed. This audit does not establish product parity, the reusable-library gate, or A–C branch disposition.
