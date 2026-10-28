# Remote Branch Cleanup Inventory and Audit

Audit date: **2026-10-22**.
Target baseline: `origin/parity/termrock-visual-baseline` (`106f71c6917461a06e6d74a97ea1ca95eb21ee7b`).
Reference polish: `origin/design/interaction-polish` (`f2f6eb716c4a512be3dc9423742fa06e86879b32`).
Repository trunk: `origin/main` (`a97e351df736b576904bc450619ca347ea64d65b`).
Frozen oracle tag: `visual-baseline` (`2718a6b8127a522a3d351d0b4bdb7542fc8da542`).

Total remote branches analyzed: **165**.

## Executive Summary

- **Protected branches (NEVER DELETE)**: **9 branches + 1 git tag** (Trunk `main`, design reference `design/interaction-polish`, baseline `parity/termrock-visual-baseline`, tag `visual-baseline`, and 6 forensic `preservation/*` branches).
- **Fully merged / subsumed branches (Safe to delete immediately)**: **65 branches**. All commits are already reachable ancestors or have 100% equivalent patch-ids in `parity/termrock-visual-baseline`.
- **Obsolete past-sprint task branches (Safe to delete / archive)**: **51 branches**. Task branches, exploratory fixes, and pre-consolidation spikes from prior sprints (dated 2026-10-20 or earlier).
- **Active in-flight WIP branches (Current sprint: 2026-10-21 / 2026-10-22)**: **40 branches**. In-progress worktrees, app-layer migrations, and consolidation candidates currently active.

## 1. Protected Branches (NEVER DELETE)

These branches and tags must **NEVER** be deleted, force-pushed, or retargeted under project hard rules:

| Branch / Ref | SHA | Date | Role / Invariant |
| --- | --- | --- | --- |
| `visual-baseline` (Tag) | `2718a6b812` | 2026-10-18 | Frozen pre-refactor visual oracle; referenced in AGENTS.md |
| `main` | `a97e351df7` | 2026-10-18 | Repository trunk (upstream default) |
| `design/interaction-polish` | `f2f6eb716c` | 2026-10-20 | Active design branch; reference interactions and modern contracts |
| `parity/termrock-visual-baseline` | `106f71c691` | 2026-10-22 | Reference PR #38 candidate baseline branch |
| `preservation/local-work-2026-10-17` | `0ab6bf38eb` | 2026-10-17 | Historical snapshot preservation |
| `preservation/terminal-components-rescue-20261019` | `cf61d53348` | 2026-10-19 | Historical snapshot preservation |
| `preservation/termrock-20261016` | `4169783ae8` | 2026-10-16 | Historical snapshot preservation |
| `preservation/termrock-combined-20261019` | `18ce1c9e83` | 2026-10-19 | Historical snapshot preservation |
| `preservation/termrock-design-interaction-polish-20261016` | `c379ea5e2b` | 2026-10-16 | Historical snapshot preservation |
| `preservation/termrock-sanitized-visibility-queue-20261019` | `0d89e5a1b3` | 2026-10-17 | Historical snapshot preservation |

## 2. Fully Merged or Subsumed Branches (65 Branches)

All commits on these branches are already present in `origin/parity/termrock-visual-baseline` (either as direct git ancestors `ahead=0`, or with 100% patch-equivalent cherry-picked commits). Deleting these branches loses zero commit history and zero code.

| Branch | Tip SHA | Date | Ahead/Behind | Subsumption Status | Last Subject |
| --- | --- | --- | --- | --- | --- |
| `parity/binding-runtime-20261020` | `98900f3e11` | 2026-10-20 | +15 / -1287 | 100% Cherry-Clean (`15` commits) | fix(runtime): preserve binding table API variants |
| `parity/ci-planning-fix-20261020` | `8beaf2865b` | 2026-10-20 | +2 / -1294 | 100% Cherry-Clean (`2` commits) | ci: generate GitHub Claude instructions as symlink |
| `parity/code-completion-composite-20261020` | `12e760e81d` | 2026-10-20 | +6 / -1333 | 100% Cherry-Clean (`6` commits) | fix(editors): scope manual completion to composite |
| `parity/completion-primitives-20261020` | `cf1ff91536` | 2026-10-20 | +1 / -1340 | 100% Cherry-Clean (`1` commits) | feat(editors): support completion splicing |
| `parity/composition-leaves-20261020` | `69605184c9` | 2026-10-20 | +1 / -1252 | 100% Cherry-Clean (`1` commits) | feat(termrock): add typed button and tabs composition f |
| `parity/field-continuation-20261020` | `de7cc53dbe` | 2026-10-20 | +12 / -1287 | 100% Cherry-Clean (`12` commits) | docs(fields): record continuation runtime blocker |
| `parity/frame-semantics-20261020` | `12ab1458fa` | 2026-10-20 | +4 / -1325 | 100% Cherry-Clean (`4` commits) | fix(tuiscotti): normalize semantic frame provenance |
| `parity/layout-plan-20261020` | `eba2423164` | 2026-10-20 | +3 / -1278 | 100% Cherry-Clean (`3` commits) | docs(layout): record independent regression results |
| `parity/menubar-hover-20261020` | `c02ea12093` | 2026-10-20 | +7 / -1315 | 100% Cherry-Clean (`7` commits) | docs(audit): record default menu parity pilot |
| `parity/migration-matrix-20261020` | `ea9f6f4400` | 2026-10-20 | +1 / -1295 | 100% Cherry-Clean (`1` commits) | docs: track termrock migration acceptance |
| `parity/nested-composition-consumer-20261020` | `ed1998f5ab` | 2026-10-21 | +0 / -1070 | Direct Ancestor (`ahead=0`) | fix: measure split groups at allocated widths |
| `parity/nested-modal-reference-20261020` | `d3e5647c87` | 2026-10-20 | +2 / -1270 | 100% Cherry-Clean (`2` commits) | docs(audit): distinguish nested layer regression receip |
| `parity/note-reference-color-20261020` | `c0c0f148b5` | 2026-10-20 | +2 / -1258 | 100% Cherry-Clean (`2` commits) | docs(audit): clarify note baseline evidence |
| `parity/picker-query-component-20261020` | `fe158fb56b` | 2026-10-20 | +7 / -1261 | 100% Cherry-Clean (`7` commits) | fix(fields): preserve hosted owner bounds |
| `parity/picker-stack-integration-20261020` | `801e6c9489` | 2026-10-20 | +0 / -1258 | Direct Ancestor (`ahead=0`) | docs: refresh picker runtime integration evidence |
| `parity/runtime-fields-binding-query-current-20261020` | `8635f4bf60` | 2026-10-20 | +34 / -1263 | 100% Cherry-Clean (`34` commits) | docs: correct runtime picker integration checkpoint |
| `parity/runtime-layer-contracts-20261020` | `0cf26d54ec` | 2026-10-20 | +13 / -1287 | 100% Cherry-Clean (`13` commits) | feat(runtime): compose same-owner bindings explicitly |
| `parity/session-cursor-parity-20261020` | `94f937deb1` | 2026-10-20 | +4 / -1287 | 100% Cherry-Clean (`4` commits) | merge: retain pushed session cursor fix |
| `parity/session-cursor-prepared-port-20261020` | `5ca3c4f205` | 2026-10-20 | +1 / -1258 | 100% Cherry-Clean (`1` commits) | fix(session): track cursor shape on prepared session |
| `parity/session-startup-20261020` | `db31adf1e7` | 2026-10-20 | +1 / -1341 | 100% Cherry-Clean (`1` commits) | fix(session): discard input queued before startup |
| `parity/shared-style-regressions-20261020` | `94a1215e27` | 2026-10-20 | +1 / -1279 | 100% Cherry-Clean (`1` commits) | fix: preserve explicit shared component styles |
| `parity/table-consumer-20261020` | `e3cccd8a27` | 2026-10-20 | +4 / -1338 | 100% Cherry-Clean (`4` commits) | docs: record table API stage and consumer state |
| `parity/text-runtime-editing-20261020` | `a099f3e242` | 2026-10-20 | +4 / -1294 | 100% Cherry-Clean (`4` commits) | fix(runtime): match baseline click release bounds |
| `parity/velnor-parity-20261020` | `e7961cee0f` | 2026-10-20 | +1 / -1286 | 100% Cherry-Clean (`1` commits) | ci: upgrade parity workflows to velnor 0.1.29 |
| `wip/action-variant-builder-20261021` | `160b1c328f` | 2026-10-21 | +1 / -131 | 100% Cherry-Clean (`1` commits) | feat(core): allow explicit action variants |
| `wip/baseline-cohort-admission-20261021` | `310a56de62` | 2026-10-21 | +0 / -977 | Direct Ancestor (`ahead=0`) | docs(audit): record fresh capture smoke receipt |
| `wip/ci-doc-inventory-20261021` | `482cdaf48c` | 2026-10-21 | +0 / -792 | Direct Ancestor (`ahead=0`) | Merge remote-tracking branch 'origin/parity/termrock-vi |
| `wip/code-editor-meta-precedence-20261021` | `671443b891` | 2026-10-21 | +0 / -185 | Direct Ancestor (`ahead=0`) | fix(editors): preserve CodeEditor footer styles |
| `wip/consolidation-refresh-20261021` | `e5077810f6` | 2026-10-22 | +0 / -34 | Direct Ancestor (`ahead=0`) | docs: adjudicate the jackin digest danger-ink drift |
| `wip/context-menu-live-contract-20261021` | `1dee009130` | 2026-10-21 | +0 / -443 | Direct Ancestor (`ahead=0`) | fix(overlays): size executable menu shortcuts correctly |
| `wip/controls-adopt-rate-limit-20261021` | `d90357899a` | 2026-10-21 | +1 / -592 | 100% Cherry-Clean (`1` commits) | wip: protect in-flight controls-adopt work after rate-l |
| `wip/domain-seeded-lint-20261021` | `c9357a9efb` | 2026-10-21 | +0 / -979 | Direct Ancestor (`ahead=0`) | refactor(tablepro-domain): extract seeded history data |
| `wip/editor-meta-merged-review-20261021` | `86f3ae3d6c` | 2026-10-21 | +0 / -169 | Direct Ancestor (`ahead=0`) | docs: preserve independently reproduced read-only caret |
| `wip/form-dialog-20261021` | `7c46c63f0c` | 2026-10-21 | +0 / -185 | Direct Ancestor (`ahead=0`) | Merge remote-tracking branch 'origin/parity/termrock-vi |
| `wip/form-field-layout-20261021` | `2f6193e6e2` | 2026-10-21 | +0 / -209 | Direct Ancestor (`ahead=0`) | feat(forms): size content fields from live data |
| `wip/grid-style-inheritance-c29` | `4f9cbb0df7` | 2026-10-21 | +0 / -1053 | Direct Ancestor (`ahead=0`) | fix(grid): preserve semantic row underline inheritance |
| `wip/holla-clock-gates-20261021` | `d4133a38a0` | 2026-10-22 | +1 / -36 | 100% Cherry-Clean (`1` commits) | feat(holla): width-band layout policy and a real histor |
| `wip/itemrow-metadata-contract-20261021` | `32628b360f` | 2026-10-21 | +0 / -273 | Direct Ancestor (`ahead=0`) | fix(runtime): reject overflowing row metadata gaps |
| `wip/list-multiline-viewport-20261021-v2` | `7126662c97` | 2026-10-21 | +1 / -539 | 100% Cherry-Clean (`1` commits) | fix(navigation): preserve multiline scroll viewport |
| `wip/master-detail-block-stage1-20261021` | `e002627864` | 2026-10-21 | +0 / -976 | Direct Ancestor (`ahead=0`) | test(termrock): use public cursor position accessor |
| `wip/master-detail-pure-layout-20261021` | `912681032b` | 2026-10-21 | +0 / -753 | Direct Ancestor (`ahead=0`) | Merge remote-tracking branch 'origin/parity/termrock-vi |
| `wip/menu-disabled-reason-20261021-a7f2` | `38148abafc` | 2026-10-21 | +0 / -374 | Direct Ancestor (`ahead=0`) | fix(overlays): keep blocked menu navigation hidden |
| `wip/nextest-doc-inventory-guard-20261021` | `fcb0d8d2fb` | 2026-10-21 | +0 / -900 | Direct Ancestor (`ahead=0`) | test(xtask): scan inner Rustdoc block comments |
| `wip/palette-rate-limit-20261021` | `73fc468fc7` | 2026-10-21 | +1 / -601 | 100% Cherry-Clean (`1` commits) | wip: protect in-flight palette work after rate-limit st |
| `wip/palette-test-integrity-20261021` | `c60830f8a3` | 2026-10-21 | +0 / -788 | Direct Ancestor (`ahead=0`) | docs(audit): record palette recovery provenance |
| `wip/parent-fields-runtime-blocker-20261021` | `379465e9e2` | 2026-10-21 | +0 / -799 | Direct Ancestor (`ahead=0`) | docs(migration): record independent clearances and moda |
| `wip/picker-row-contrast-20261021` | `c932adb5ee` | 2026-10-21 | +0 / -246 | Direct Ancestor (`ahead=0`) | fix(picker): scope contrast fallback to authored backgr |
| `wip/picker-row-default-precedence-20261021` | `f46caa2ce6` | 2026-10-21 | +0 / -248 | Direct Ancestor (`ahead=0`) | fix(picker): resolve component ink below owner styles |
| `wip/picker-short-footer-regression-20261021` | `cf854b9e5a` | 2026-10-21 | +0 / -246 | Direct Ancestor (`ahead=0`) | test(overlays): cover short picker footers |
| `wip/readonly-caret-runtime-20261021` | `cb2e2a41bb` | 2026-10-21 | +0 / -164 | Direct Ancestor (`ahead=0`) | docs: bind caret proof to final runtime test |
| `wip/recolor-provenance-20261021` | `dfb54cfc5d` | 2026-10-21 | +0 / -807 | Direct Ancestor (`ahead=0`) | test(runtime): cover partial recolor provenance |
| `wip/review-ui-ux-20261021` | `3226b9b409` | 2026-10-22 | +1 / -60 | 100% Cherry-Clean (`1` commits) | docs(audit): UI/UX and interaction review of the showca |
| `wip/rowui-container-width-20261021` | `e1d81b106c` | 2026-10-21 | +0 / -271 | Direct Ancestor (`ahead=0`) | test(navigation): cover row width across list variants |
| `wip/session-typed-input-20261021` | `26255e048c` | 2026-10-21 | +0 / -143 | Direct Ancestor (`ahead=0`) | Merge remote-tracking branch 'origin/parity/termrock-vi |
| `wip/shell-reach-rustdoc-20261021` | `92bd508210` | 2026-10-21 | +0 / -574 | Direct Ancestor (`ahead=0`) | docs(shell): fix testing-only reach module link |
| `wip/showcase-false-parity-20261021` | `095cafc1e4` | 2026-10-22 | +4 / -72 | 100% Cherry-Clean (`4` commits) | merge: keep the live chrome page over the pushed base |
| `wip/split-block-contract-20261021` | `759d0d411e` | 2026-10-21 | +0 / -1026 | Direct Ancestor (`ahead=0`) | fix(runtime): keep row label styles canonical |
| `wip/tablepro-blank-domain-20261021` | `f0760cc204` | 2026-10-21 | +0 / -599 | Direct Ancestor (`ahead=0`) | fix(tablepro): start new connection drafts blank |
| `wip/tablepro-query-focus-contract-20261021` | `3bab23ecfc` | 2026-10-21 | +5 / -277 | 100% Cherry-Clean (`5` commits) | fix(editors): preserve edit sessions on configured blur |
| `wip/tablepro-strict-clippy-20261021` | `8ec5d042eb` | 2026-10-21 | +2 / -499 | 100% Cherry-Clean (`2` commits) | Merge remote-tracking branch 'origin/parity/termrock-vi |
| `wip/termrock-extract-shell-20261021` | `9892d22a06` | 2026-10-21 | +0 / -641 | Direct Ancestor (`ahead=0`) | docs: hand off the shell extraction branch pending push |
| `wip/termrock-theme-typed-slots-20261021` | `8e7480415a` | 2026-10-21 | +0 / -172 | Direct Ancestor (`ahead=0`) | docs: record inherited theme profile failure after sync |
| `wip/theme-declaration-provenance-stage2a-20261021` | `a178975dbd` | 2026-10-21 | +0 / -86 | Direct Ancestor (`ahead=0`) | docs(theme): record normalization review fix evidence |
| `wip/theme-field-hover-20261021` | `4507e7de2b` | 2026-10-21 | +0 / -833 | Direct Ancestor (`ahead=0`) | docs(audit): record field hover capability gates |
| `wip/typed-dialog-props-slots-20261021` | `6cc98ab0fb` | 2026-10-21 | +0 / -690 | Direct Ancestor (`ahead=0`) | Merge remote-tracking branch 'origin/parity/termrock-vi |

## 3. Obsolete Task Branches From Past Sprints (51 Branches)

These branches were created for specific tasks, audits, or fixes in past sprints (dated on or before 2026-10-20). Their useful contracts were superseded, partially integrated, or abandoned in favor of consolidated branches. They are safe to delete or archive as git tags.

| Branch | Tip SHA | Date | Ahead/Behind | Unique Commits | Last Subject | Recommended Disposition |
| --- | --- | --- | --- | --- | --- | --- |
| `audit/replay-origin-current-source-20261020` | `55824ec3ba` | 2026-10-20 | +2 / -1281 | 2 | docs: correct replay source inventory counts | Safe to delete / archive |
| `chore/rust-comment-cleanup` | `0b1142a2fb` | 2026-10-18 | +3 / -1545 | 3 | style: preserve strict Clippy after comment remova | Safe to delete / archive |
| `ci/design-rust-timeout-20261020` | `f2f6eb716c` | 2026-10-20 | +1064 / -2382 | 1051 | ci: raise design Rust job timeout to 15 minutes | Safe to delete / archive |
| `ci/zizmor-design-20261020` | `4860d0350d` | 2026-10-20 | +1063 / -2382 | 1050 | ci: update design zizmor pin to 1.31.1 | Safe to delete / archive |
| `codex/rescue-capture-command-prod-gate-20261017` | `77fc91a82a` | 2026-10-17 | +1 / -1550 | 1 | fix(visibility): enable capture runner in Unix bui | Safe to delete / archive |
| `codex/rescue-pr17-e44-20261017` | `e44d5504e9` | 2026-10-09 | +472 / -2382 | 434 | fix(visibility): label recorded handoff actions an | Safe to delete / archive |
| `codex/rescue-status-proposal-check-20261018` | `2a5b0b5f40` | 2026-10-18 | +2 / -1550 | 2 | chore: archive status proposal preservation snapsh | Safe to delete / archive |
| `codex/termrock-local-proposals-20261018` | `824876cb04` | 2026-10-18 | +2 / -1550 | 2 | archive: preserve local termrock proposals | Safe to delete / archive |
| `codex/termrock-q48-source-only-20261019` | `0864434508` | 2026-10-19 | +3 / -1550 | 3 | archive: preserve reviewed Q48 queue v2 source wit | Safe to delete / archive |
| `docs/branch-authority-cleanup` | `c4b62deccf` | 2026-10-18 | +12 / -1548 | 12 | docs: define source documentation policy and lint  | Safe to delete / archive |
| `docs/interaction-reference-cleanup` | `1d35481674` | 2026-10-18 | +1046 / -2382 | 1045 | docs: reconcile branch authority and preserve plan | Safe to delete / archive |
| `fix/picker-focus-and-rows-20261020` | `56a8547a48` | 2026-10-20 | +4 / -1294 | 4 | fix(picker): inherit row limit and avoid duplicate | Safe to delete / archive |
| `parity/acceptance-architecture-20261020` | `c6f1ee6246` | 2026-10-20 | +3 / -1305 | 3 | docs: refresh architecture gate evidence | Safe to delete / archive |
| `parity/acceptance-architecture-module-path-20261020` | `461c7d1e1a` | 2026-10-20 | +15 / -1294 | 15 | docs: bind module scan reports to source revision | Safe to delete / archive |
| `parity/acceptance-architecture-p1-20261020` | `73e8dea226` | 2026-10-20 | +7 / -1295 | 7 | docs: record exact checked architecture revision | Safe to delete / archive |
| `parity/acceptance-architecture-symlink-20261020` | `a48e94fae4` | 2026-10-20 | +11 / -1294 | 11 | docs: bind source report revision | Safe to delete / archive |
| `parity/button-contract-20261020` | `7dab012ee6` | 2026-10-20 | +1 / -1281 | 1 | fix(controls): share button state and busy renderi | Safe to delete / archive |
| `parity/button-contract-review-20261020` | `2d65740b71` | 2026-10-20 | +3 / -1279 | 3 | fix(theme): match pressed button variant ink | Safe to delete / archive |
| `parity/buttons-reference-decision-20261020` | `2289a4f476` | 2026-10-20 | +4 / -1278 | 3 | docs: record TablePro delete dialog parity proof | Safe to delete / archive |
| `parity/completion-manual-fixes-20261020` | `1a150420b0` | 2026-10-20 | +7 / -1333 | 1 | fix(editors): retain manual completion binding | Safe to delete / archive |
| `parity/composition-slots-20261020` | `be674ef879` | 2026-10-20 | +1 / -1294 | 1 | feat(termrock): add typed picker layer slot | Safe to delete / archive |
| `parity/edit-menu-overlay-20261020` | `12a24761ff` | 2026-10-20 | +12 / -1294 | 8 | fix(overlays): match edit menu capability rows | Safe to delete / archive |
| `parity/empty-style-reference-fix-20261020` | `45b4ca6792` | 2026-10-20 | +11 / -1273 | 3 | docs(audit): correct empty test evidence labels | Safe to delete / archive |
| `parity/frozen-table-workflows-20261020` | `b6d75a22d7` | 2026-10-20 | +1 / -1315 | 1 | test: add frozen CLI table sorting workflows | Safe to delete / archive |
| `parity/item-row-palette-reference-20261020` | `23a3fac940` | 2026-10-20 | +9 / -1270 | 1 | test: preserve legacy item row palette fixture | Safe to delete / archive |
| `parity/master-detail-20261020` | `1ca91e1421` | 2026-10-20 | +7 / -1341 | 2 | fix(runtime): shadow hidden chords in focused hint | Safe to delete / archive |
| `parity/picker-family-contracts-20261020` | `0416a30146` | 2026-10-20 | +10 / -1294 | 6 | fix(fields): prepare contextual edits from pointer | Safe to delete / archive |
| `parity/picker-geometry-20261020` | `6b72d77a6e` | 2026-10-20 | +7 / -1294 | 7 | fix(overlays): match picker geometry and hints | Safe to delete / archive |
| `parity/runtime-fields-integration-20261020` | `ffc3b25efb` | 2026-10-20 | +28 / -1273 | 2 | docs: record repeated runtime gate result | Safe to delete / archive |
| `parity/session-cursor-policy-20261020` | `5b6a231219` | 2026-10-20 | +2 / -1273 | 2 | fix(session): track cursor shape before flushing | Safe to delete / archive |
| `parity/shared-style-integration-20261020` | `e72dbb1536` | 2026-10-20 | +6 / -1273 | 3 | fix: preserve explicit dialog chrome styles | Safe to delete / archive |
| `parity/showcase-button-consumer-20261020` | `60138e17c6` | 2026-10-20 | +4 / -1279 | 4 | refactor(showcase): use shared button rendering | Safe to delete / archive |
| `parity/strict-accounting-fix-20261020` | `70b7f9e6c1` | 2026-10-20 | +34 / -1305 | 23 | fix(architecture): account for Rust path attribute | Safe to delete / archive |
| `parity/table-component-20261020` | `f0899c71b1` | 2026-10-20 | +8 / -1336 | 1 | fix(grid): restore table baseline sort and tone st | Safe to delete / archive |
| `parity/tablepro-footer-focus-20261020` | `1ac6dfced9` | 2026-10-20 | +1 / -1263 | 1 | fix(tablepro): scope footer hints to focused query | Safe to delete / archive |
| `parity/tablepro-live-contracts-20261020` | `4ab81ae474` | 2026-10-20 | +3 / -1279 | 3 | fix(tablepro): restore explorer command and footer | Safe to delete / archive |
| `parity/tablepro-picker-live-20261020` | `366dc67c95` | 2026-10-20 | +11 / -1289 | 10 | refactor(tablepro): use shared picker contracts | Safe to delete / archive |
| `parity/theme-palette-baseline-20261020` | `95cdc7bdb1` | 2026-10-20 | +8 / -1273 | 3 | fix(theme): match Junie mono faint palette | Safe to delete / archive |
| `parity/velnor-design-upgrade-20261020` | `8533344520` | 2026-10-20 | +1062 / -2382 | 1049 | docs: clarify Velnor preview verification gap | Safe to delete / archive |
| `wip/ci-upgrade-handoff-20261020` | `706c138339` | 2026-10-20 | +2 / -1271 | 2 | docs: correct latest CI handoff evidence | Safe to delete / archive |
| `wip/plan-s16` | `70e04be37a` | 2026-10-18 | +880 / -2382 | 879 | docs: reconcile validation contracts across branch | Safe to delete / archive |
| `wip/pr38-palette-style-integration-20261020` | `b0a3bea5a4` | 2026-10-20 | +8 / -1270 | 1 | docs: record palette integration reference conflic | Safe to delete / archive |
| `wip/rapps-3-7-14` | `b76517cec3` | 2026-10-18 | +1055 / -2382 | 1054 | docs: reconcile validation contracts across branch | Safe to delete / archive |
| `wip/reach-pages-a` | `3cb2198dd6` | 2026-10-18 | +1061 / -2382 | 1060 | docs: reconcile validation contracts across branch | Safe to delete / archive |
| `wip/reach-pages-a-2` | `37e21b8a47` | 2026-10-18 | +1065 / -2382 | 1064 | docs: reconcile validation contracts across branch | Safe to delete / archive |
| `wip/s14-followups` | `19fcb24b68` | 2026-10-18 | +883 / -2382 | 882 | docs: reconcile validation contracts across branch | Safe to delete / archive |
| `wip/shared-design-working-tree-20261020` | `1d7fdb224a` | 2026-10-20 | +1065 / -2382 | 1052 | wip: preserve design working tree snapshot | Safe to delete / archive |
| `wip/style-conformance-20261020` | `c2396b4562` | 2026-10-20 | +13 / -1336 | 2 | docs: preserve deferred style parity handoff | Safe to delete / archive |
| `wip/table-consumer-baseline` | `fa02fa54a6` | 2026-10-20 | +15 / -1338 | 1 | fix(showcase): preserve baseline task status tone | Safe to delete / archive |
| `wip/termrock-reviewed-contract-integration-20261020` | `8462bf28aa` | 2026-10-20 | +4 / -1278 | 4 | refactor(showcase): use shared button rendering | Safe to delete / archive |
| `wip/v1-showcase` | `68651fba71` | 2026-10-18 | +784 / -2382 | 783 | docs: reconcile validation contracts across branch | Safe to delete / archive |

## 4. Active In-Flight WIP Branches (40 Branches)

These branches represent ongoing work from the current sprint (dated 2026-10-21 and 2026-10-22). They contain unmerged unique commits and active worktrees. They should **NOT** be deleted until their respective topics are merged or explicitly handed off.

| Branch | Tip SHA | Date | Ahead/Behind | Unique Commits | Last Subject | Current Status |
| --- | --- | --- | --- | --- | --- | --- |
| `wip/ansi16-shell-chrome-20261021` | `cf48be987e` | 2026-10-22 | +75 / -154 | 60 | docs(audit): record the 16-colour Overview frame r | Active In-Flight WIP |
| `wip/branch-consolidation-20261021` | `37f704c9df` | 2026-10-21 | +7 / -221 | 5 | merge: fold the current parity tip into the consol | Active In-Flight WIP |
| `wip/conformance-semantic-fixes-20261021b` | `160b582355` | 2026-10-22 | +69 / -154 | 56 | fix(theme): let the mono state rules carry the sta | Active In-Flight WIP |
| `wip/diagnose-component-digest-drift-20261021` | `fa5f100a46` | 2026-10-22 | +3 / -55 | 1 | merge: keep the diagnosis branch on the parity tip | Active In-Flight WIP |
| `wip/external-public-consumer-20261021` | `fd1e65243d` | 2026-10-21 | +2 / -276 | 2 | test(consumer): land public consumer contracts and | Active In-Flight WIP |
| `wip/holla-geometry-blocks-20261021` | `63a0f9ccc6` | 2026-10-21 | +5 / -276 | 5 | fix(shell): drop the expect calls from the router  | Active In-Flight WIP |
| `wip/holla-home-migration-20261021` | `51a17fa1dd` | 2026-10-21 | +51 / -210 | 43 | test(termrock-shell): the band policy and the scre | Active In-Flight WIP |
| `wip/holla-home-row-composition-20261021` | `06b9fe0b88` | 2026-10-21 | +3 / -205 | 2 | docs(holla): record frozen home row comparison | Active In-Flight WIP |
| `wip/holla-menu-patch-precedence-20261021` | `378b798779` | 2026-10-21 | +4 / -215 | 4 | docs: record raw recipe provenance API limit | Active In-Flight WIP |
| `wip/holla-rate-limit-20261021` | `098fcfcdab` | 2026-10-21 | +7 / -842 | 7 | wip: protect in-flight holla work after rate-limit | Active In-Flight WIP |
| `wip/jackin-app-layer-20261021` | `4246661e5c` | 2026-10-21 | +5 / -76 | 3 | chore(xtask): drop the stale Jackin ownership exce | Active In-Flight WIP |
| `wip/jackin-false-parity-20261021` | `773a1ca1d1` | 2026-10-22 | +5 / -57 | 2 | merge: keep the pushed jackin accounts branch in t | Active In-Flight WIP |
| `wip/jackin-live-capsule-menu-20261021` | `de819fb1ca` | 2026-10-21 | +1 / -443 | 1 | refactor(jackin): use live capsule context menus | Active In-Flight WIP |
| `wip/jackin-rate-limit-20261021` | `01dcfddde8` | 2026-10-21 | +1 / -617 | 1 | wip: protect in-flight jackin work after rate-limi | Active In-Flight WIP |
| `wip/manager-composition-interop-20261021` | `621997e7dd` | 2026-10-21 | +6 / -335 | 3 | Merge remote-tracking branch 'origin/parity/termro | Active In-Flight WIP |
| `wip/owned-fresh-plan-lifecycle-20261021` | `1f0d0ce4f7` | 2026-10-21 | +24 / -339 | 18 | docs(audit): record rooted worker review evidence | Active In-Flight WIP |
| `wip/owned-runner-ordinary-phase-20261021` | `139bf1cd77` | 2026-10-21 | +25 / -339 | 19 | feat: add ordinary checks to fresh baseline runs | Active In-Flight WIP |
| `wip/palette-grid-repair-20261021` | `b26f548722` | 2026-10-21 | +8 / -1148 | 2 | fix(theme): carry profile provenance through cache | Active In-Flight WIP |
| `wip/pr38-completion-verify-20261021` | `8e976098d1` | 2026-10-21 | +3 / -227 | 3 | docs: record conformance driver handoff | Active In-Flight WIP |
| `wip/public-consumer-contracts-20261021` | `fc861e15af` | 2026-10-21 | +1 / -545 | 1 | test: add public component consumer contracts | Active In-Flight WIP |
| `wip/radio-navigation-contract-20261021` | `1b677b30c7` | 2026-10-21 | +5 / -508 | 1 | Merge remote-tracking branch 'origin/parity/termro | Active In-Flight WIP |
| `wip/session-consolidation-20261021` | `382b3193d7` | 2026-10-22 | +96 / -56 | 73 | docs(matrix): record the measured allocation attri | Active In-Flight WIP |
| `wip/showcase-app-layer-cleanup-20261021` | `d996860142` | 2026-10-21 | +3 / -276 | 3 | test(showcase): pin the shell chrome hit regions | Active In-Flight WIP |
| `wip/showcase-baseline-rebless-20261021` | `f4518ac3a5` | 2026-10-22 | +96 / -56 | 73 | docs(audit): keep the re-bless evidence out of the | Active In-Flight WIP |
| `wip/showcase-dialog-kinds-restoration-20261021` | `260a10e361` | 2026-10-21 | +14 / -125 | 9 | wip: diagnose Showcase chooser hit shift | Active In-Flight WIP |
| `wip/showcase-heading-fixtures-20261021b` | `30001296a0` | 2026-10-21 | +65 / -154 | 52 | test(showcase): refresh the shell header and page  | Active In-Flight WIP |
| `wip/showcase-pages-migration-20261021` | `8997fd295f` | 2026-10-21 | +50 / -210 | 42 | refactor(showcase): draw the card chrome with the  | Active In-Flight WIP |
| `wip/tablepro-app-layer-20261021` | `6849f9a601` | 2026-10-21 | +50 / -210 | 42 | refactor(tablepro): compose the grid chrome from t | Active In-Flight WIP |
| `wip/tablepro-blank-database-validation-20261021` | `8992262f4c` | 2026-10-21 | +2 / -512 | 1 | fix(tablepro): allow blank connection databases | Active In-Flight WIP |
| `wip/tablepro-geometry-blocks-20261021` | `3cdbc322f4` | 2026-10-21 | +4 / -276 | 1 | refactor(tablepro): let the shared viewport own th | Active In-Flight WIP |
| `wip/tablepro-grid-perf-20261021` | `136a567e89` | 2026-10-22 | +83 / -76 | 64 | perf(tablepro): keep the workbench frame under the | Active In-Flight WIP |
| `wip/tablepro-name-prefix-20261021` | `c8f92ea0c0` | 2026-10-21 | +2 / -530 | 2 | fix(tablepro): preserve typed connection name pref | Active In-Flight WIP |
| `wip/tablepro-picker-scope-20261021` | `d40f8e5fb7` | 2026-10-21 | +6 / -655 | 3 | docs: hand off keymap identity fix | Active In-Flight WIP |
| `wip/text-input-focus-select-20261021` | `27f65eaef5` | 2026-10-21 | +2 / -153 | 1 | Merge remote-tracking branch 'origin/parity/termro | Active In-Flight WIP |
| `wip/theme-palette-adjudication-20261021` | `b16bc773de` | 2026-10-22 | +74 / -154 | 59 | docs(matrix): record the workspace-wide before and | Active In-Flight WIP |
| `wip/velnor-actions-0135-20261021` | `dc5ade35e5` | 2026-10-21 | +3 / -340 | 2 | docs(ci): record workflow audit review boundary | Active In-Flight WIP |
| `wip/velnor-actions-0136-20261021` | `507c232073` | 2026-10-21 | +4 / -275 | 2 | ci: refresh Velnor verification on current PR head | Active In-Flight WIP |
| `wip/w1-jackin-cursor` | `9efc4a88fa` | 2026-10-22 | +1 / -34 | 1 | refactor(jackin-preview): route zoom caret and dia | Active In-Flight WIP |
| `wip/w1-lib-lineblock` | `2829063eef` | 2026-10-22 | +1 / -34 | 1 | Add termrock blocks paint_row helper; default empt | Active In-Flight WIP |
| `wip/w1-tablepro-owner` | `5a322bb5f0` | 2026-10-22 | +1 / -34 | 1 | tablepro: drop set_surface seeding, add safe-mode  | Active In-Flight WIP |

## 5. Recommended Cleanup Actions

### Phase 1: Prune 65 Fully Subsumed Branches
The 65 fully subsumed branches can be pruned immediately from remote with no loss of work:
```bash
# Delete fully subsumed branches on origin
git push origin --delete parity/binding-runtime-20261020
git push origin --delete parity/ci-planning-fix-20261020
git push origin --delete parity/code-completion-composite-20261020
git push origin --delete parity/completion-primitives-20261020
git push origin --delete parity/composition-leaves-20261020
git push origin --delete parity/field-continuation-20261020
git push origin --delete parity/frame-semantics-20261020
git push origin --delete parity/layout-plan-20261020
git push origin --delete parity/menubar-hover-20261020
git push origin --delete parity/migration-matrix-20261020
git push origin --delete parity/nested-composition-consumer-20261020
git push origin --delete parity/nested-modal-reference-20261020
git push origin --delete parity/note-reference-color-20261020
git push origin --delete parity/picker-query-component-20261020
git push origin --delete parity/picker-stack-integration-20261020
git push origin --delete parity/runtime-fields-binding-query-current-20261020
git push origin --delete parity/runtime-layer-contracts-20261020
git push origin --delete parity/session-cursor-parity-20261020
git push origin --delete parity/session-cursor-prepared-port-20261020
git push origin --delete parity/session-startup-20261020
git push origin --delete parity/shared-style-regressions-20261020
git push origin --delete parity/table-consumer-20261020
git push origin --delete parity/text-runtime-editing-20261020
git push origin --delete parity/velnor-parity-20261020
git push origin --delete wip/action-variant-builder-20261021
git push origin --delete wip/baseline-cohort-admission-20261021
git push origin --delete wip/ci-doc-inventory-20261021
git push origin --delete wip/code-editor-meta-precedence-20261021
git push origin --delete wip/consolidation-refresh-20261021
git push origin --delete wip/context-menu-live-contract-20261021
git push origin --delete wip/controls-adopt-rate-limit-20261021
git push origin --delete wip/domain-seeded-lint-20261021
git push origin --delete wip/editor-meta-merged-review-20261021
git push origin --delete wip/form-dialog-20261021
git push origin --delete wip/form-field-layout-20261021
git push origin --delete wip/grid-style-inheritance-c29
git push origin --delete wip/holla-clock-gates-20261021
git push origin --delete wip/itemrow-metadata-contract-20261021
git push origin --delete wip/list-multiline-viewport-20261021-v2
git push origin --delete wip/master-detail-block-stage1-20261021
git push origin --delete wip/master-detail-pure-layout-20261021
git push origin --delete wip/menu-disabled-reason-20261021-a7f2
git push origin --delete wip/nextest-doc-inventory-guard-20261021
git push origin --delete wip/palette-rate-limit-20261021
git push origin --delete wip/palette-test-integrity-20261021
git push origin --delete wip/parent-fields-runtime-blocker-20261021
git push origin --delete wip/picker-row-contrast-20261021
git push origin --delete wip/picker-row-default-precedence-20261021
git push origin --delete wip/picker-short-footer-regression-20261021
git push origin --delete wip/readonly-caret-runtime-20261021
git push origin --delete wip/recolor-provenance-20261021
git push origin --delete wip/review-ui-ux-20261021
git push origin --delete wip/rowui-container-width-20261021
git push origin --delete wip/session-typed-input-20261021
git push origin --delete wip/shell-reach-rustdoc-20261021
git push origin --delete wip/showcase-false-parity-20261021
git push origin --delete wip/split-block-contract-20261021
git push origin --delete wip/tablepro-blank-domain-20261021
git push origin --delete wip/tablepro-query-focus-contract-20261021
git push origin --delete wip/tablepro-strict-clippy-20261021
git push origin --delete wip/termrock-extract-shell-20261021
git push origin --delete wip/termrock-theme-typed-slots-20261021
git push origin --delete wip/theme-declaration-provenance-stage2a-20261021
git push origin --delete wip/theme-field-hover-20261021
git push origin --delete wip/typed-dialog-props-slots-20261021
```

### Phase 2: Archive or Delete Obsolete Past-Sprint Branches
For the 51 obsolete past-sprint branches, create archive tags if audit retention is desired before deleting:
```bash
# Example archival pattern before deletion:
# git tag archive/<branch-name> origin/<branch-name>
# git push origin --delete <branch-name>
```
