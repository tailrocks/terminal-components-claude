# PR38 completion evidence — remote branches outside wip/*, parity/*, main, design/interaction-polish

Read-only git archaeology. No branch, ref, worktree or file in the repository was modified.

- Repository: `/Users/donbeave/Projects/termrock-main-parity` (worktree of `tailrocks/terminal-components-claude`)
- Pinned PR38 tip: `7649ee7ed8e8645e49f859a171ea0193fccc1e82` (`origin/parity/termrock-visual-baseline`), re-checked after `git fetch --all --prune`, unchanged.
- Reviewed: 18 remote branches (`audit/*` 1, `chore/*` 1, `ci/*` 2, `codex/*` 5, `docs/*` 2, `fix/*` 1, `preservation/*` 6).
- Excluded by instruction: `origin/main`, `origin/design/interaction-polish`, `origin/parity/*`, `origin/wip/*`, and `origin` (symbolic HEAD).
- No `ledger*`, `holla*`, `work/*`, `task/*`, `integrate/*` refs exist on origin at review time.
- Method: `git rev-parse`, `git log -1`, `git merge-base --is-ancestor`, `git rev-list --count`, `git cherry -v <pin> <branch>`, then content checks with `git show <pin>:<path>`, `git grep <pin>`, and `git show <sha>`.

## Table

| branch | tip | unique commits | evidence in PR38 | recommendation | reason |
|---|---|---|---|---|---|
| `audit/replay-origin-current-source-20261020` | `55824ec3ba5d` 2026-10-20 | 2 (`c2463c933625` docs: audit replay-origin production paths; `55824ec3ba5d` docs: correct replay source inventory counts) | `docs/audits/2026-10-20-replay-origin-current-source.md` in PR38 is byte-identical (`diff` empty) | already-integrated | both commits landed; nothing left |
| `chore/rust-comment-cleanup` | `0b1142a2fb24` 2026-10-18 | 3 (`c7b7db59c926` chore: remove Rust comments while preserving documentation metadata; `9ee542a366b5` style: remove spacing left by safety comment cleanup; `0b1142a2fb24` style: preserve strict Clippy after comment removal) | PR38 keeps Rust comments (e.g. `crates/termrock-overlays/src/picker.rs:106` rustdoc "Blank rows kept under the list") and `AGENTS.md` says comments are kept unless asked | exclude | superseded, policy-rejected sweep over the pre-crate-split tree (492 files) |
| `ci/design-rust-timeout-20261020` | `f2f6eb716c4a` 2026-10-20 | 1051 (1050 = pre-split holla-fable lineage; 1 = `f2f6eb716c4a` ci: raise design Rust job timeout to 15 minutes) | `.velnor/config.toml:9` `crate_timeout_minutes = 15` in PR38 | already-integrated | config value present; tip SHA is also the tip of the protected `origin/design/interaction-polish`, so the commit survives panel deletion |
| `ci/zizmor-design-20261020` | `4860d0350d5f` 2026-10-20 | 1050 (1049 lineage; 1 = `4860d0350d5f` ci: update design zizmor pin to 1.31.1) | `mise.toml:7` `zizmor = "1.31.1"`, `mise.lock:251-270` 1.31.1 URLs | already-integrated | pin present; later upgrades (`velnor 0.1.32`) supersede the branch docs |
| `codex/rescue-capture-command-prod-gate-20261017` | `77fc91a82aeb` 2026-10-17 | 1 (`77fc91a82aeb` fix(visibility): enable capture runner in Unix builds) | `crates/termrock-visibility-tests/src/bin/capture-command.rs:52` `#[cfg(unix)] fn main` → `:100` `execute_request_with_signal(...)`; the old `#[cfg(all(unix, test))]` gate now sits only on the test-only wrapper at `:253` | already-integrated | the production gate is open on Unix builds |
| `codex/rescue-pr17-e44-20261017` | `e44d5504e91d` 2026-10-09 | 434 (all = pre-split lineage; tip = `e44d5504e91d` fix(visibility): label recorded handoff actions and publish accepted claims) | `tools/visibility/queue.py:1083,1158` "Recorded next action at handoff"; `crates/termrock-visibility-tests/tests/queue_v2.rs:2466` `handoff_next_action_is_labeled_as_recorded_in_v1_and_v2`; tip is an ancestor of `preservation/termrock-20261016` and `preservation/termrock-combined-20261019` | already-integrated | both the code and the regression test landed, plus the branch is redundant with two preservation refs |
| `codex/rescue-status-proposal-check-20261018` | `2a5b0b5f40c5` 2026-10-18 | 2 (`77fc91a82aeb` capture gate; `2a5b0b5f40c5` chore: archive status proposal preservation snapshot) | capture gate integrated as above; snapshot re-points status fixtures at `docs/implementation/visibility/evidence/reports/status-source-archive-20261009/`, and PR38 holds that archive (71 files) with the path referenced at `crates/termrock-visibility-tests/tests/status.rs` (3 hits) and `tools/visibility/status.py` (1 hit) | exclude | preservation snapshot; the archive it names already ships in PR38, so no work is missing |
| `codex/termrock-local-proposals-20261018` | `824876cb044c` 2026-10-18 | 2 (`77fc91a82aeb`; `824876cb044c` archive: preserve local termrock proposals) | PR38 has no `archive/` directory; queued proposal source copies (`queue_v2.rs` 4777 lines, `status.rs` 4562 lines, `status.py` 5615 lines) exist only on the branch | needs-review | pure preservation snapshot, no new work; decide keep vs drop |
| `codex/termrock-q48-source-only-20261019` | `086443450833` 2026-10-19 | 3 (`77fc91a82aeb`; `824876cb044c`; `086443450833` archive: preserve reviewed Q48 queue v2 source without private snapshots) | same as above plus `archive/proposals/q48-queue-v2-20261017/{README.md,SOURCE-MAP.tsv,queue_v2.rs}`; PR38 has no `archive/` tree | needs-review | preservation snapshot of the reviewed Q48 queue v2 source; no product/CI delta |
| `docs/branch-authority-cleanup` | `c4b62deccf0f` 2026-10-18 | 12 (`b03f80527b73` chore: remove Rust comments without changing code tokens; `dc9b1e42c19f` docs: reconcile architecture and product authority…; `039daab014ae` chore: format comment-free Rust and retire obsolete guides; `bcc0f9f9bd87` style: remove comment-derived attribute spacing; `1e04593ab368` docs: reconcile remaining guides…; `b9b9bb0f2f35` docs: include final attribute-spacing source revision; `f510169bee51` docs: rebuild project documentation and retire obsolete reports; `eec887bcbdd8` docs: make safety guidance describe ongoing maintenance; `94b1711cdf0e` docs: map current package and dependency ownership; `fa60869245e9` docs: use the current facade package in pull request checks; `ba9a9fd2667f` fix: preserve the frozen visibility proof source fixture; `c4b62deccf0f` docs: define source documentation policy and lint levels) | the branch's policy "Rust source files … contain no comments or prose documentation" (`docs/development.md` diff) contradicts current `AGENTS.md`; PR38 ships its own docs tree (2026-10-21 audits, `docs/handoff`) and keeps comments | exclude | superseded docs/comment policy; not contained in any preservation ref, so deletion loses only obsolete commits |
| `docs/interaction-reference-cleanup` | `1d35481674bd` 2026-10-18 | 1045 (1044 lineage; 1 = `1d35481674bd` docs: reconcile branch authority and preserve planned work) | rewrites `GOAL.md`, `README.md`, `docs/GOAL.md`, adds `SAFETY.md` and comment-free policy; PR38 has its own `docs/GOAL.md`, `docs/architecture/*`, `docs/handoff`, and a different AGENTS contract | exclude | superseded design-lineage docs rewrite; unique tip commit is in no preservation ref |
| `fix/picker-focus-and-rows-20261020` | `56a8547a4864` 2026-10-20 | 4 (`d4832fe73fd0` feat(picker): reuse shared query editing contracts; `c1335e19cb5e` fix(filter-list): propagate query editor repaint; `f8e08fc29edc` fix(picker): preserve query theme and editor repaint; `56a8547a4864` fix(picker): inherit row limit and avoid duplicate focus) | present: `filter_list.rs:721` `acc.repaint()`, `:864` `acc.fold(&input.erase())`, `:233` `query_edit_menu`, picker query theming (`docs/audits/2026-10-20-picker-family-contracts.md` "editor response is folded into the list response"). **Missing**: `FilterPolicy::Substring` (PR38 enum is only `Label`/`Caller`, `filter_list.rs:221`; `Label` is a subsequence match at `:641`), `Picker::max_rows` (PR38 caps at `picker.rs:363` `d.size.popup_max_rows`), searchable-picker `CursorStyle::Fill` default (PR38 `picker.rs:103,146,338` forces `Bar`), plus `crates/termrock/tests/picker_contract.rs` external-consumer test | integrate | the branch fulfils claims PR38's own audit doc already makes ("substring filtering", "`max_rows`, default 12") and the AGENTS fill-for-owner-list rule; port only those hunks, not the whole branch |
| `preservation/local-work-2026-10-17` | `7982d64c6c0d` 2026-10-17 | 3 (`17c543693bf2` feat(visibility): add queue evidence and reviewer handoff; `b404ee06ae59` feat(visibility): report status execution attempts and phases; `7982d64c6c0d` fix(visibility): reconcile incomplete attempt counts) | execution-attempt-history tests survive in PR38 only as retired records: `crates/termrock-visibility-tests/tests/status.rs:5228` is `#[ignore = "… kept as a historical record outside active verification (recovery ledger R-41)"]`, and checked-in facts assert `execution_attempt_history` / `product_phase_observations` are absent (`status.rs:2717` area); the status.py support functions (`validate_execution_attempt_history`, `render_execution_attempt_history`, `validate_product_phase_observations`, `render_required_command_rows`) exist only on this branch | exclude | deliberately retired feature (R-41), not a PR38 gap |
| `preservation/terminal-components-rescue-20261019` | `c8c114cf223c` 2026-10-19 | 687 (pre-split lineage incl. `49961e03b500`… `215990ef2d68` audit fixes, `5cdbe63962d5`/`df006ceb0b01` branch-audit workflows; tip `c8c114cf223c` archive: preserve all salvage tips and source snapshots, 3-parent merge) | product content carried into the crate-split tree (e.g. warning glyph `▲` at `crates/termrock-feedback/src/hintbar.rs:799`; `.gitignore:9` `.DS_Store`; holla action launcher as `crates/holla-ui/src/screens/actions.rs`); hand-added `holla-fable-audit*.yml` workflows are deliberately absent (PR38 has only the velnor-generated `.github/workflows/ci.yml`) | exclude | preservation of the superseded pre-split lineage; no unported library/parity/CI delta |
| `preservation/termrock-20261016` | `023fdeb41fd7` 2026-10-16 | 663 (pre-split lineage; tip `023fdeb41fd7` preserve: retain Termrock local history, 6-parent merge) | same supersession evidence as above; ancestry contains `codex/rescue-pr17-e44` tip | exclude | preservation merge snapshot, no new work |
| `preservation/termrock-combined-20261019` | `f34cb4cf1fbe` 2026-10-19 | 577 (pre-split lineage + visibility prototypes; tip `f34cb4cf1fbe` Preserve combined Termrock local refs and source objects, 7-parent merge) | same supersession evidence; contains `2a5b0b5f40c5`, `77fc91a82aeb`, `824876cb044c`, `e44d5504e91d` tips | exclude | preservation merge snapshot, no new work |
| `preservation/termrock-design-interaction-polish-20261016` | `e172d3b2f946` 2026-10-16 | 465 (design-lineage history; tip `e172d3b2f946` test(showcase): table tests follow the Basic table template) | tip is an ancestor of the protected `origin/design/interaction-polish` (verified with `git merge-base --is-ancestor`) | exclude | redundant with the protected reference; retention is an owner decision |
| `preservation/termrock-sanitized-visibility-queue-20261019` | `17c543693bf2` 2026-10-17 | 1 (`17c543693bf2` feat(visibility): add queue evidence and reviewer handoff) | PR38 `tools/visibility/queue.py` implements the same policy in evolved form: `transition_record` appends evidence, `reassign_record:955` performs the reviewer/owner handoff with claim history; four old tests (`test_append_evidence_*`, `test_reviewer_handoff_*` in `tools/visibility/tests/test_queue.py`) were dropped with the old API, while PR38's file has 35 other queue tests | exclude | superseded prototype of the queue evidence/handoff policy |

## Deletion candidates

Safe now (fully integrated, no unique surviving content):

1. `audit/replay-origin-current-source-20261020` — doc byte-identical in PR38.
2. `ci/design-rust-timeout-20261020` — timeout in `.velnor/config.toml:9`; tip SHA also held by protected `origin/design/interaction-polish`.
3. `ci/zizmor-design-20261020` — zizmor 1.31.1 in `mise.toml` / `mise.lock`.
4. `codex/rescue-capture-command-prod-gate-20261017` — capture runner enabled at `capture-command.rs:52,100`.
5. `codex/rescue-pr17-e44-20261017` — code + test landed (`queue.py:1083`, `queue_v2.rs:2466`); tip also an ancestor of two preservation refs.

Safe after owner acknowledgement (superseded; unique commits live nowhere else, content obsolete):

6. `chore/rust-comment-cleanup` — comment-removal sweep contradicting current `AGENTS.md`.
7. `docs/branch-authority-cleanup` — comment-free documentation policy and retired guides, contradicted by current docs tree.
8. `docs/interaction-reference-cleanup` — design-lineage docs authority rewrite, superseded by PR38 docs.
9. `codex/rescue-status-proposal-check-20261018` — snapshot of old status sources whose referenced archive already ships in PR38.

Redundant preservation ref (delete only on explicit owner decision):

9. `preservation/termrock-design-interaction-polish-20261016` — every commit is contained in the protected `origin/design/interaction-polish`.

## Must keep

Protected, never delete:

- `origin/main`
- `origin/design/interaction-polish`

Keep pending work or unreviewed preservation content:

- `fix/picker-focus-and-rows-20261020` — integration source for `FilterPolicy::Substring`, `Picker::max_rows`, searchable `CursorStyle::Fill`, and the external-consumer picker test.
- `codex/termrock-q48-source-only-20261019`, `codex/termrock-local-proposals-20261018` — proposal archives that exist nowhere else (PR38 has no `archive/` tree).
- `preservation/local-work-2026-10-17` — only copy of the attempt-history/phase-view status.py implementation; retirement under R-41 is recorded but the prototype source is unreviewed elsewhere.
- `preservation/terminal-components-rescue-20261019`, `preservation/termrock-20261016`, `preservation/termrock-combined-20261019`, `preservation/termrock-sanitized-visibility-queue-20261019` — pre-split lineage and queue-prototype preservation refs.

## Cross-branch notes

- Pre-split lineage group: `ci/design-rust-timeout-20261020`, `ci/zizmor-design-20261020`, `codex/rescue-pr17-e44-20261017`, `docs/interaction-reference-cleanup`, `preservation/terminal-components-rescue-20261019`, `preservation/termrock-design-interaction-polish-20261016` all descend from merge base `cc14dd6beae5` (2026-09-03, "chore: organize Holla concept references"). Their large unique counts are that shared lineage, not per-branch work. Full per-commit lists: `git cherry -v 7649ee7ed8e8 origin/<branch>`.
- Newest shared lineage commits, all superseded by the crate split and the tuiscotti visual gate: `49961e03b500` docs(holla): goal and `/goal` prompt; `93d840fecc30` chore(agents): kimi k3 subagents; `26ed5d8e37a2` docs(agents); `0f8e8fc97fc4` feat(holla): context-adaptive action launcher, fully simulated; `58fea3e1389e` fix(widgets): warning statuses carry `▲`; `94a383098cb5` docs: tonal entity strip, dialog width, status glyphs.
- Content from that lineage that has no PR38 equivalent is documentation and branch-local tooling only: `holla-project/notes/{decisions,design-note,interaction-models}.md`, `DESIGN.md`, `HOLLA_REFACTOR_RECOVERY_PLAN.md`, `tools/{holla_flows,holla_shots,p6_matrix,audit_flows,audit_shots}.sh`, `tests/focus_gutter.rs`, and the `holla-fable-audit*.yml` workflows (`git ls-tree -r 7649ee7ed8e8 | grep -E 'holla-project|HOLLA_REFACTOR'` is empty). Workflows are velnor-generated now, so hand-added workflows must not return.
- `chore/rust-comment-cleanup` and `docs/branch-authority-cleanup` encode a "no Rust comments" policy that current `AGENTS.md` rejects ("existing comments kept unless asked"); do not port.
- PR38 doc/code gap worth tracking independently of any branch: `docs/audits/2026-10-20-picker-family-contracts.md:3` claims contiguous substring filtering and `max_rows` default 12, while PR38 code has `FilterPolicy::{Label,Caller}` (`filter_list.rs:221`) and no `Picker::max_rows` (`picker.rs:363` uses `d.size.popup_max_rows` only). `fix/picker-focus-and-rows-20261020` supplies both.

## Appendix — unique commits of the small branches

- `audit/replay-origin-current-source-20261020`: `c2463c933625` docs: audit replay-origin production paths; `55824ec3ba5d` docs: correct replay source inventory counts.
- `chore/rust-comment-cleanup`: `c7b7db59c926`, `9ee542a366b5`, `0b1142a2fb24`.
- `codex/rescue-capture-command-prod-gate-20261017`: `77fc91a82aeb`.
- `codex/rescue-status-proposal-check-20261018`: `77fc91a82aeb`, `2a5b0b5f40c5`.
- `codex/termrock-local-proposals-20261018`: `77fc91a82aeb`, `824876cb044c`.
- `codex/termrock-q48-source-only-20261019`: `77fc91a82aeb`, `824876cb044c`, `086443450833`.
- `docs/branch-authority-cleanup`: `b03f80527b73`, `dc9b1e42c19f`, `039daab014ae`, `bcc0f9f9bd87`, `1e04593ab368`, `b9b9bb0f2f35`, `f510169bee51`, `eec887bcbdd8`, `94b1711cdf0e`, `fa60869245e9`, `ba9a9fd2667f`, `c4b62deccf0f`.
- `fix/picker-focus-and-rows-20261020`: `d4832fe73fd0`, `c1335e19cb5e`, `f8e08fc29edc`, `56a8547a4864`.
- `preservation/local-work-2026-10-17`: `17c543693bf2`, `b404ee06ae59`, `7982d64c6c0d`.
- `preservation/termrock-sanitized-visibility-queue-20261019`: `17c543693bf2`.
