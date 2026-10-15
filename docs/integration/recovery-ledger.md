# Termrock recovery ledger

Branch: `integrate/unique-0722f4f9`. Plan and record in ASD-STE100 Simplified Technical English.
Commands, identifiers, and paths stay unchanged. Evidence = commands in this repository at the
recorded heads. "Main" = 0722f4f91, "target" = 2e07d6f7b, "PR #16" = c54419a41,
"PR #17" = c4f47c954, "EF" = f8d35d46a, "RC" = 4eafb5eda, "target-initial" = aac478a09.

## 1. Source record (re-verified 2026-10-15)

| Source | Head | Tree ID | Merge base with main | State |
|---|---|---|---|---|
| main | 0722f4f91ec9a20241a56ccb1f74177be09b9f7c | c1cb1a56cf91730e | — | frozen, matches record |
| target-initial | aac478a09efc1a145779a71a65c87f7a2baa5d31 | 61b4ed54948a574f | 0722f4f91 (main) | first target commit |
| target | 2e07d6f7bb72e59ff8e5f876565deec3af02272d | 416ad5fde533d539 | 0722f4f91 (main) | frozen, matches record |
| PR #16 (termrock-refactor) | c54419a41693e9eadf68e128c50f07e445ee6707 | 4ee8706bdc78ed03 | cc14dd6beae526884 | branch restored + pushed; PR closed unmerged |
| PR #17 (termrock-implementation) | c4f47c954f88166ec25ddd78115119529bdae76b | f46edd020862c95f | cc14dd6beae526884 | PR open; head == branch head |
| integrate/goal-ef2fa1ea4 | f8d35d46a6fc650c464aec77ca4ccdcf818e891a | 537d40d0deb9e224 | c3108b5797498522 | frozen, matches record |
| integrate/goal-reconcile-41ccfd59 | 4eafb5eda0ea09cdcc16eb1d56a1e03ab5b827f0 | 42ce9db967d36882 | 41ccfd59cf8b8d54 | frozen, matches record |

Fetch: `git fetch origin '+refs/heads/*:refs/remotes/origin/*' '+refs/pull/16/head:refs/remotes/origin/pr/16' '+refs/pull/17/head:refs/remotes/origin/pr/17' --no-prune --no-tags`.
PR #16 head is an ancestor of PR #17 head (`git merge-base --is-ancestor` = true). PR #17 head ==
`origin/termrock-implementation`. One source counted once.
Recovery refs: `refs/recovery/{main,target-initial,target,pr16,pr17,goal-ef2fa1ea4,goal-reconcile-41ccfd59}`.
Bundle: `/Users/donbeave/Projects/termrock-recovery/termrock-recovery-20261015.bundle` (outside the
tree; `git bundle verify` reports a complete history, sha1). No reset, no force-push, no deletion of
main, target, or any source.

## 2. Baseline on the target (before crate changes)

Worktree `/Users/donbeave/Projects/termrock-integration` at 2e07d6f7b, log
`/Users/donbeave/Projects/termrock-recovery/baseline-target.log`:

| Check | Command | Result |
|---|---|---|
| metadata | `cargo metadata --locked --format-version 1` | PASS (exit 0) |
| format | `cargo fmt --all -- --check` | PASS (exit 0) |
| lint | `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` | PASS (exit 0) |
| tests | `cargo test --locked --workspace --all-targets --all-features` | PASS (exit 0; last suite 122 passed, 0 failed) |

PR #17 build health at c4f47c954 (`/Users/donbeave/Projects/termrock-recovery/notes/pr17-check.log`):
`cargo check --locked --workspace --all-targets --all-features` FAILS in `termrock-runtime`
(5 errors: `ratatui_crossterm` unlinked; feature `crossterm = []` empty; no optional dependency).
PR #17 also re-exports `termrock_runtime::{Acc, cell_at, ...}` which that tree does not define.
The integration fixes both; see rows R-03 and R-10.

## 3. Comparison method (section 4 of the goal)

For every source S: merge base B, then
`git log --left-right --cherry-mark M...S`, `git diff --find-renames B S`,
`git diff --find-renames M S`, `git diff --find-renames T S`. Key facts:

- main is 830 commits ahead of cc14dd6be; `git diff --stat cc14dd6be main` = 4167 files,
  +477146/−66979. The monolith moved from root `src/` to `crates/tui` (package `junie-tui`)
  and absorbed a reviewed port of Termrock behavior (main #19).
- PR #17 branched at cc14dd6be and rebuilt the tree as 44 workspace members + 2 nested
  workspaces under `crates/`. Its library content is an independent evolution of the same
  monolith: 89 of 91 mapped module files differ from the target (`diff` census,
  `/Users/donbeave/Projects/termrock-recovery/notes/` and codemod census output).
- EF and RC are cherry-pick stacks of the target's own warning-gap/RowChrome/RowUi work;
  their entire `crates/tui` delta is byte-identical to the target
  (`git diff f8d35d46a 2e07d6f7b -- crates/tui/src/collection/rowui.rs crates/tui/src/components/hintbar.rs crates/tui/src/components/list.rs | wc -l` = 0;
  `git diff 4eafb5eda 2e07d6f7b -- crates/tui | wc -l` = 0).
- PR #16's Tuiscotti baseline survives into PR #17 byte-identical for
  `tests/visual_baseline/{showcase,tablepro,jackin}.rs` → `crates/termrock-conformance/tests/visual_baseline/`,
  and `tests/holla_pty.rs` → `crates/holla/tests/holla_pty.rs` (blob 4ff4baea unchanged).

Decision rule: target content wins for behavior; PR #17 supplies the crate structure and its
unique features, ported on top; nothing is rejected for layout alone.

## 4. Decision rows

Legend: main/target state = PRESENT/ABSENT/PARTIAL at the frozen heads. Destination = path or
crate in the integration branch. Test = the check that proves the row.

### From main and target (keep; nothing to port)

| ID | Source | Paths | Benefit | Main | Target | Destination | Decision | Test |
|---|---|---|---|---|---|---|---|---|
| R-01 | main | apps/tablepro/tests/form_radio.rs (451 lines, blob 6832fc214) | One painted row per option; after Space the Staging marker is present; off fields stay Tab-reachable | PRESENT | PRESENT | crates/tablepro-ui/tests/form_radio.rs | KEEP | `cargo test -p tablepro-ui` (form_radio green) |
| R-02 | main | crates/tui/src/components/choice.rs (RadioGroupState cursor/value separation :1545, :1578, :1620, :1694) | Radio cursor independent of value; caller-controlled commit | PRESENT | PRESENT | crates/termrock-controls/src/choice.rs | KEEP | cargo test -p termrock-controls (radio_group_separates_cursor_from_value) |
| R-03 | target aac478a09 | crates/tui/src/components/hintbar.rs | Warning-gap paint in the label tone (warning_status_gap_uses_the_label_tone) | ABSENT | PRESENT | crates/termrock-feedback/src/hintbar.rs | KEEP | cargo test -p termrock-feedback |
| R-04 | target aac478a09 | crates/tui/src/components/list.rs (RowChrome enum) | Fourth bool on List removed; gutter chrome typed | ABSENT | PRESENT | crates/termrock-navigation/src/list.rs | KEEP | cargo test -p termrock-navigation |
| R-05 | target aac478a09 | crates/tui/src/collection/rowui.rs | Cell::symbol flush-meta assertions (row_ui_meta_is_dropped_all_or_none) | ABSENT | PRESENT | crates/termrock-runtime/src/rowui.rs | KEEP | cargo test -p termrock-runtime |
| R-06 | target 2e07d6f7b | docs/implementation/visibility/recovered-nextest.toml | Recovered nextest config out of Velnor generation path | ABSENT | PRESENT | same path | KEEP | file present; no .config/nextest.toml at root |
| R-07 | main | crates/termrock-e2e/** | Parity harness, admission schemas, tree-ID vs SHA-256 source_snapshot_tree validation | PRESENT | PRESENT | crates/termrock-e2e (unchanged; own workspace + lockfile) | KEEP | `diff` vs 2e07d6f7b empty |
| R-08 | main | tools/visibility/**, tools/refactor-proof | Visibility queue/status tools; refactor-proof harness | PRESENT | PRESENT | unchanged (target bytes restored after import) | KEEP | `git diff 2e07d6f7b -- tools/ mise.toml` empty |
| R-09 | main | crates/tui/src/components/form.rs (EnterPolicy :300), components/chip.rs (LabelChips :160) | Tested public entries retained | PRESENT | PRESENT | crates/termrock-forms/src/form.rs, crates/termrock-navigation/src/chip.rs | KEEP | cargo test -p termrock-forms -p termrock-navigation |
| R-10 | target | crates/tui/src/runtime.rs `#[cfg(feature = "crossterm")]` gating; crates/tui Cargo.toml `crossterm = ["dep:ratatui-crossterm"]` | Backend-free consumers compile; optional terminal I/O | PRESENT | PRESENT | crates/termrock-runtime Cargo.toml + cfg gates | KEEP/RESTORE | `cargo check -p termrock-runtime --no-default-features` |
| R-11 | target | .github/workflows/ci.yml, .velnor/config.toml | Current generated CI + discovery excludes | PRESENT | PRESENT | unchanged; regenerated for the new workspace (row R-24) | KEEP | velnor-actions plan/generate diff check |
| R-12 | EF/RC | crates/tui/src/{components/hintbar.rs, components/list.rs, collection/rowui.rs} | Same bytes as target (cherry-picks of the target work) | ABSENT | PRESENT | see R-03..R-05 | PRESENT (do not re-apply) | `git diff f8d35d46a 2e07d6f7b -- <files>` = empty; `git diff 4eafb5eda 2e07d6f7b -- crates/tui` = empty |
| R-13 | EF | apps/tablepro/tests/form_radio.rs (422-line version) | Older assertions demand Staging absent after selection | superseded | PRESENT (451-line corrected) | not applied | REJECT (superseded by PR #22 corrected behavior; goal forbids hidden-option assertions) | R-01 test |
| R-14 | RC | apps/tablepro/tests/form_radio.rs | Cherry-pick was already empty against 41ccfd59 | PRESENT | PRESENT | none needed | PRESENT | documented in RC completion report; patch emptiness re-verified by R-12 diffs |

### From PR #16 (via PR #17 history; counted once)

| ID | Source | Paths | Benefit | Main | Target | Destination | Decision | Test |
|---|---|---|---|---|---|---|---|---|
| R-15 | PR16→PR17 | tests/visual_baseline/{showcase,tablepro,jackin}.rs | Typed visual baseline journeys per app | ABSENT | ABSENT | crates/termrock-conformance/tests/visual_baseline/ | PORT (byte-identical through PR #17) | file list + explicit ignored run (section 12) |
| R-16 | PR16→PR17 | tests/holla_pty.rs | Holla PTY interaction test | ABSENT | ABSENT | crates/holla/tests/holla_pty.rs | PORT | file present; explicit run |
| R-17 | PR16→PR17 | tests/conformance/registry.rs + required_cases.json | Typed case registry and reference data | ABSENT | ABSENT | crates/termrock-test-support (registry) + conformance data | PORT | registry compiles; store-integrity checks pass |
| R-18 | PR16 | tests/tuiscotti_qualification.rs | API qualification coverage | ABSENT | ABSENT | crates/termrock-conformance/tests/ | PORT (recovered from PR #16 history; PR #17 dropped the file) | explicit run |
| R-19 | PR16 | tests/choice_containment.rs, tests/focus_gutter.rs, tests/terminal_suspend.rs, tests/conformance.rs | Choice containment, focus gutter, terminal suspension coverage | ABSENT | ABSENT | crates/termrock-conformance/tests/ + owner crates | PORT or REPLACE (folded checks verified against control_states.rs coverage) | per-test runs |
| R-20 | PR16 | .config/nextest.toml | pty group filters for visual/PTY binaries | ABSENT | ABSENT (moved to docs/…/recovered-nextest.toml) | keep out of discovery until the harness exists | REPLACE (target disposition kept; see R-06) | no .config/nextest.toml at root |

### From PR #17 (structure + unique features)

| ID | Source | Paths | Benefit | Main | Target | Destination | Decision | Test |
|---|---|---|---|---|---|---|---|---|
| R-21 | PR17 | crates/* (44 members), root Cargo.toml, rustfmt.toml | The required functional crate split | ABSENT | ABSENT | crates/termrock-* (19 libs), 4 app groups, 4 tools | PORT (structure) | cargo metadata --locked; workspace check |
| R-22 | PR17 | crates/termrock-terminal | TerminalView, TerminalSource, TerminalCell, TerminalCursor, TerminalViewState/Selection/Interaction/Action, InputToken, LinkKey | ABSENT | ABSENT | crates/termrock-terminal | PORT | cargo check -p termrock-terminal; facade exports |
| R-23 | PR17 | crates/termrock-session | TerminalSession, DefaultTerminal, chain_panic_hook, run, run_with_feedback_clock | ABSENT | ABSENT | crates/termrock-session | PORT | cargo check -p termrock-session |
| R-24 | PR17 | crates/termrock-render | PartStyle primitives + TerminalSource surface | ABSENT | ABSENT | crates/termrock-render | PORT | cargo check -p termrock-render |
| R-25 | PR17 | navigation/item.rs | Shared item vocabulary (Item, AsItem, ItemRow, ItemRowLayout, ItemColumns) | ABSENT | ABSENT | crates/termrock-navigation/src/item.rs | PORT (content from target picker; breaks navigation↔overlays cycle) | cargo check -p termrock-navigation -p termrock-overlays |
| R-26 | PR17 | grid.rs GridOverflowIndicator (Count/Ellipsis) | Header overflow policy choice | ABSENT | ABSENT | crates/termrock-grid/src/grid.rs (enum + builder + draw branches) | PORT | cargo test -p termrock-grid |
| R-27 | PR17 | text/secret.rs CellWriter abstraction | Secrets paint through a trait; text crate stays below runtime | ABSENT | ABSENT (target write_mask takes CellUi directly) | crates/termrock-text/src/secret.rs + `impl CellWriter for CellUi` | REPLACE (retains target masking behavior; removes text→runtime cycle) | cargo test -p termrock-text (zeroize/mask tests) |
| R-28 | PR17 | core/variant.rs raw()+Debug, ALL; core/layer_types.rs; core/keys.rs | Recipe Variant/layer/phase vocabulary lives below theme/runtime | PARTIAL (types in theme/keymap/layer) | PARTIAL | crates/termrock-core/src/{variant,layer_types,keys}.rs; theme/keymap/layer re-export | REPLACE (one canonical home; old paths kept by re-exports) | workspace check + re-export paths compile |
| R-29 | PR17 | collections key.rs key_at/index_of family; reconcile.rs StepDir/first_enabled/seek; scroll.rs grab/press_track/drag_track | Shared cursor stepping, thumb-grab drag, reverse lookup | ABSENT | ABSENT | crates/termrock-collections/src/{collection/key.rs, collection/reconcile.rs, scroll.rs} | PORT (three-way merge with target fixes) | cargo test -p termrock-collections (track/drag tests) |
| R-30 | PR17 | crates/termrock-test-support, -conformance, -xtask, -visibility-publisher | Test scope, conformance harness, ownership gate, visibility publisher | ABSENT (tui-testing only) | ABSENT | crates/termrock-{test-support,conformance,xtask,visibility-publisher} | PORT | per-crate checks |
| R-31 | PR17 | crates/termrock-visibility-tests (nested workspace) | Pinned visibility tool tests | ABSENT | ABSENT | unchanged nested workspace | PORT | per-workspace check |
| R-32 | PR17 | holla-ui/src/tui/** (vendored monolith copy) | none — duplicates production code | ABSENT | ABSENT | deleted | REJECT (second production implementation; goal §7 forbids) | absence in tree; holla uses termrock facade |
| R-33 | PR17 | controls lib.rs re-export of `termrock_runtime::InheritedFormState` | Form/choice state sharing without a controls→forms cycle | PRESENT (monolith) | PRESENT | canonical `termrock_controls::choice::InheritedFormState`; forms re-exports | REPLACE (type home moved down; behavior unchanged) | cargo check -p termrock-forms -p termrock-controls |
| R-34 | PR17 | `#[non_exhaustive]` on Status/Phase shared enums | Split forward-compat | PRESENT (attrs) | PRESENT | kept; downstream matches gained documented `_` arms | PORT | workspace check (button, split, viewport, filter_list, list, tabs, picker_chain) |
| R-35 | PR17 | facade `termrock` module-per-crate re-exports + curated `termrock::author` | Public API entry for apps | ABSENT | ABSENT | crates/termrock/src/lib.rs (extended with target root exports) | PORT | cargo check -p termrock; api-map |

### Integration-specific fixes (this branch)

| ID | Paths | Benefit | Decision | Test |
|---|---|---|---|---|
| R-36 | crates/termrock-runtime/src/support.rs | Shared component helpers (Acc, SlotFn, overlay_chrome, cell_at, first_row, shift, paint_pressed_bracket) as one crate-owned copy | PORT from target components/mod.rs | workspace check; all component crates compile |
| R-37 | crates/termrock-core/src/id.rs Revision | TerminalSource/selection staleness contract needs the type; PR #17 referenced it but never defined it | PORT (implemented; zero/next/raw) | cargo check -p termrock-terminal -p termrock-render |
| R-38 | crates/termrock-runtime Cargo.toml + cfg gates | Restores backend-free compile lost in PR #17 | RESTORE of target contract | `cargo check -p termrock-runtime --no-default-features` |
| R-39 | docs/integration/api-map.md | Every public item has a destination | NEW (required by goal §7) | this file's table |
| R-40 | crates/termrock-e2e untouched | Isolated harness keeps own workspace/lockfile | KEEP | diff vs target empty; per-workspace checks |

## 5. Removals (monolith retirement)

| Path | Reason | Decision | Authorized by |
|---|---|---|---|
| crates/tui (junie-tui) | Content moved into the split; goal §7 forbids a second production implementation in the old monolithic package | REMOVE after move | termrock-recovery-goal.md §1, §7 |
| crates/tui-testing (junie-tui-testing) | Merged into crates/termrock-test-support | REMOVE after merge | §7 |
| apps/* | Moved into crates/<app-group> crates | REMOVE after move | §7 |
| xtask/ (root) | Replaced by crates/termrock-xtask | REMOVE after parity check | §7 |
| holla-project/ | Not present on target; no action | — | — |

## 6. Validation record

Filled in section 12 runs after the move; every entry carries command, result, evidence.
