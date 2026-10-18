# Findings record: documentation-repair research

Scope: research record only. It changes no source, test, or checklist row.
Method: verify each claim with `git` or file reads before writing it.
Unverifiable items carry the label `unresolved`.
Branch: `termrock-implementation`. Task start commit equals HEAD at research time.

## 1. Research identities

| Reference | Resolved value | Verification command |
| --- | --- | --- |
| `visual-baseline` tag object | `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5` | `git rev-parse refs/tags/visual-baseline` |
| Tag target commit | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` | `git cat-file -p refs/tags/visual-baseline` |
| `origin/visual-baseline` tip | `adfe39f305469120f0fe8c5d71fe662347127536` | `git rev-parse origin/visual-baseline` |
| `termrock-refactor` tip | `15c1a0a73911584cf2339eaad2c0212b2661fbd8` | `git rev-parse termrock-refactor` |
| `termrock-implementation` HEAD at research time | `658e983a6112f2d515e5a44591f084190822d209` | `git rev-parse HEAD` |
| Task start commit | `658e983a6112f2d515e5a44591f084190822d209` | `git rev-parse HEAD` |

Note: `visual-baseline` is ambiguous as a bare refname. Commands above use full refs.
Local `refs/heads/visual-baseline` resolves to `adfe39f305469120f0fe8c5d71fe662347127536`. It matches the remote tip.

Two documentation-only commits landed after the research HEAD and before this
record was committed: `183900cfd` (component contract alignment) and
`8f4929b97` (the `ownership-and-parity.md` commit of this task). Both touch
`docs/` only.
Neither changes the audited code paths, so the traces below stay valid.

Pack-observed tips come from `termrock-doc-guardrails/sources.lock.json`:

| Pack field | Value | Live comparison |
| --- | --- | --- |
| `termrock-implementation` | `4d117b48092c5210c80070ac9bbeb6560973b2b0` | Stale. Research-time HEAD is `658e983a`. |
| `termrock-refactor` | `15c1a0a73911584cf2339eaad2c0212b2661fbd8` | Matches live tip. |
| `visual-baseline-branch` | `209c88e7494cad301771bed55916316d4e890ec7` | Stale. Tip is `adfe39f3`. |
| `visual-baseline-tag-commit` | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` | Matches frozen tag. |
| `baseline_src_tree` | `a372ce43f3ac9e010d1c1a44ba381a7181a6ab23` | Matches live check below. |

## 2. Branch-vs-tag comparison

Run these commands to reproduce the comparison:

```text
git rev-parse refs/tags/visual-baseline:src origin/visual-baseline:src
git diff --stat refs/tags/visual-baseline origin/visual-baseline
git diff --quiet refs/tags/visual-baseline origin/visual-baseline -- Cargo.toml Cargo.lock baselines
```

Observed results:

- Both `:src` trees resolve to `a372ce43f3ac9e010d1c1a44ba381a7181a6ab23`.
- `Cargo.toml` is identical across tag and branch.
- `Cargo.lock` is identical across tag and branch.
- `baselines/` is identical across tag and branch.
- The tag-to-branch diff touches 13 files with 12,558 added lines and zero deletions.
- All additions live in `docs/testing/`, `scripts/test-scope-check.sh`, `tests/scenario-registry/`, and `tests/visual_baseline/`.
- `snapshots/` exists at the tag and is identical across tag and branch.
- `snapshots/` and root `tests/` do not exist on `termrock-implementation` HEAD.
  That branch migrated captures to `baselines/tuiscotti-v1/` and per-crate tests.
  This is a branch-layout difference, not a tag-branch difference.

Limits:

- Equal `src` trees prove source-tree equality only.
- They do not prove equal dependencies, toolchains, fixtures, or captures.
- Check every other visual input before relying on branch equivalence.
- Keep the frozen tag immutable.

## 3. Findings F-C1..F-C5

All five commits are ancestors of HEAD. Verified with `git merge-base --is-ancestor`.
Commit order for the prelude pair: `9f3e04fe` (C1) precedes `6fc94a4d1` (revert),
which precedes `fbca27983` (C2). Verified with `git log --ancestry-path`.
Rule IDs come from `docs/architecture/component-composition.md` (ARC)
and `docs/verification/ownership-and-parity.md` (VER).
FIX owners come from `docs/implementation/code-remediation-backlog.md`.
DOC rows name the documentation-repair checklist IDs from the input pack.
Those rows land in `checklist.json` gate `DOC` in a later commit of this task.

### F-C1 — Mixed-subject commit with superseded prelude painter

- Source commit: `9f3e04fe667aa704a03e7b70d6a1f4bae579165a`.
- Source file: `crates/jackin-preview-app/src/app.rs`.
- Symbols: `draw_historical_prelude_120_40`, `draw_historical_prelude_80_24`.
- Observed mechanism: the commit adds palette-based full-screen prelude painters.
- The painter builds a `put` closure over `ui.paint_str` and `ui.fill`.
- It emits a hardcoded 120x40 frame: menu bar, dialog borders, file rows, checkboxes, footer hints.
- Dispatch gates on `self.route == Route::Prelude` plus exact size match (`120x40`, `80x24`).
- No scenario gate and no motion gate appear in the C1 dispatch hunk.
- Affected component: Prelude wizard (Picker, fields, choices, Dialog).
- Affected preview: `jackin-preview-app`.
- Violated requirement: ARC-006 (drawing permission), ARC-009 (one path),
  ARC-012 (historical painters, hardcoded answer tables, direct `ui.paint_str`/`ui.fill`).
- Violated requirement: VER-004 (ownership proof), VER-005 (generalization checks).
- Required documentation change: require review of the complete diff and all changed
  consumers. State that a commit title is not a scope inventory.
- Future implementation owner: FIX-001 (historical Prelude replacement).
- Rejecting test: change prelude source records and step state at a non-listed size.
  Expect component-owned output. Reject any size-gated historical frame.
- Trace to HEAD: historical, not active. `6fc94a4d1` reverted the C1 prelude code.
  `fbca27983` then added the RGB variant that remains at HEAD (`app.rs:5033`).
- Conforming part: the same commit fixes `tablepro` `table_data` conformance inside
  owning library crates (`termrock-grid/src/grid.rs`, `termrock-navigation/src/tabs.rs`,
  `termrock-runtime/src/rowui.rs`, `termrock-theme/src/builtin/mod.rs`).
  The grid fix adjusts generic column-width logic (`prefix_glyph`, `header_prefix`,
  `GridColumnFit`). It names no app route or fixture ID. This part is a remediation
  template, not a violation.

### F-C2 — RGB historical prelude painter, active at HEAD

- Source commit: `fbca27983a4f6ef980a2e24ec068ab57c33d6177`.
- Source file: `crates/jackin-preview-app/src/app.rs`.
- Symbols: `historical_span_style`, `draw_historical_prelude_120_40`,
  `draw_historical_prelude_80_24`.
- Observed mechanism: the commit adds `historical_span_style(fg, bg, bold)`.
- The helper builds `termrock::author::{Color, Modifier, Style}` values from RGB tuples.
- `draw_historical_prelude_120_40` paints the frame cell by cell with `ui.paint_str`.
- Output includes fixed field text (`~/src/payments-platform`), selected rows,
  checkbox glyphs (`[ ]`), and button labels.
- Dispatch gates on `Route::Prelude` plus exact size (`120x40`, `80x24`).
- The commit also binds `KeyCode::Char('n')` to `CMD_NEW_WORKSPACE` in two phases.
- Rendering and input change in one commit.
- Affected component: Prelude wizard and workspace keymap.
- Affected preview: `jackin-preview-app`.
- Violated requirement: ARC-006, ARC-009, ARC-012 (raw `termrock::author` access,
  direct control rendering, padded strings, manual borders).
- Violated requirement: VER-004, VER-006 (forbidden-pattern detection).
- Required documentation change: require resolving visual mismatch in the owning
  component, semantic theme, or documented layout recipe. Prohibit reproducing
  expected frames in application code.
- Future implementation owner: FIX-001.
- Rejecting test: mutate the library Picker/field painter. Expect the real prelude
  to change. Reject output that stays fixed.
- Trace to HEAD: active. Definitions live at `app.rs:4976` (`historical_span_style`)
  and `app.rs:5033`/`app.rs:5177` (prelude painters). Dispatch lives at `app.rs:11662`.

### F-C3 — Historical manager painters behind scenario and motion gates

- Source commit: `d6637639107d2046247d0a537946520960a6c719`.
- Source file: `crates/jackin-preview-app/src/app.rs`.
- Symbols: `draw_historical_manager_tree_expanded_120_40`,
  `draw_historical_manager_detail_drawer_120_40`,
  `draw_historical_manager_help_overlay_120_40`,
  `draw_historical_manager_launch_picker_120_40`,
  `draw_historical_manager_hard_launch_picker_120_40`,
  `draw_historical_manager_menu_open_120_40`,
  `draw_historical_manager_inspect_120_40`,
  `draw_historical_manager_quit_confirm_120_40`.
- Observed mechanism: the commit adds eight 120x40 manager painters (3,178 added lines).
- Each painter emits full-screen output with `ui.paint_str` and `historical_span_style`.
- Dispatch requires `Route::Manager`, size `(120, 40)`, scenario in
  `{Returning, FirstUse, HardCases}`, and `Motion::Paused`.
- Inner selection uses state flags: `help_open`, `manager_quit_confirm`,
  `manager_menu_open`, `manager_inspect_open`, agent options, `detail_open()`,
  tree expansion.
- The commit narrows an older size ladder by removing `(120, 40)` from the
  `(72, 20) | (80, 24) | (100, 30) | (160, 50)` match arm.
- The commit adds Escape handling that clears the new inspect/menu/quit flags.
- Affected component: Manager List/Tree, Menu, Dialog, HelpOverlay, launch picker.
- Affected preview: `jackin-preview-app`.
- Violated requirement: ARC-009, ARC-012 (scenario-selected painters, paused bypass,
  historical overlay after normal drawing: `draw_footer` and `draw_layers` run first).
- Violated requirement: VER-004, VER-005, VER-007 (mutation matrix).
- Required documentation change: state that an omitted or unread patch is not an empty
  change. Require source comparison. Prohibit per-size answer selection.
- Future implementation owner: FIX-002.
- Rejecting test: vary workspace data and dimensions under each scenario.
  Drive help, menu, inspect, and quit through real events. Reject flag-selected frames.
- Trace to HEAD: active. Definitions live at `app.rs:7481`–`app.rs:10196`.
  Dispatch lives at `app.rs:11814` with identical gates.

### F-C4 — Historical editor/cockpit module with layer bypass

- Source commit: `087a1af433c0e99a32fd537d19d8418ebaac9c2a`.
- Source files: `crates/jackin-preview-app/src/app.rs`,
  `crates/jackin-preview-app/src/app/historical_editor_cockpit.rs`.
- Symbols: `draw_historical_editor_mounts_120_40`,
  `draw_historical_editor_mounts_dirty_120_40`,
  `draw_historical_editor_roles_120_40`, `draw_historical_editor_env_120_40`,
  `draw_historical_editor_auth_120_40`,
  `draw_historical_editor_save_preview_120_40`,
  `draw_historical_cockpit_info_120_40`,
  `draw_historical_cockpit_cancel_confirm_120_40`,
  `draw_historical_cockpit_debug_120_40`.
- Observed mechanism: the commit adds a 2,054-line historical module.
- It widens `historical_span_style` to `pub(super)` for module use.
- It adds an early return from normal layer drawing for 120x40 paused scenarios
  (`Returning`, `FirstUse`, `HardCases`).
- Editor dispatch selects painters from `EditorTab` plus `dirty` and `preview_open`.
- Cockpit dispatch selects painters from cancel-confirm, info, and debug flags.
- The commit also changes event bindings and state handling.
- Affected component: Editor tabs/fields, cockpit overlays, layer stack.
- Affected preview: `jackin-preview-app`.
- Violated requirement: ARC-009, ARC-012 (paused mode bypassing layers,
  state-flag painter selection).
- Violated requirement: VER-004, VER-006.
- Required documentation change: require reviewing rendering and input together.
  State that paused capture must not remove the normal layer mechanism.
- Future implementation owner: FIX-003.
- Rejecting test: exercise all tabs, dirty/preview states, and real overlays
  under paused and running motion. Expect identical component paths.
- Trace to HEAD: active. Module is declared at `app.rs:245`.
  Call sites live at `app.rs:11889`–`app.rs:11933`.

### F-C5 — Historical capsule painters with palette/menu bypass

- Source commit: `34b28f6ebe63a0c968367d7bd822a6245c97647e`.
- Source files: `crates/jackin-preview-app/src/app.rs`,
  `crates/jackin-preview-app/src/app/historical_capsule.rs`.
- Symbols: `draw_historical_capsule_menu_120_40`,
  `draw_historical_capsule_new_tab_120_40`,
  `draw_historical_capsule_split_vertical_120_40`,
  `draw_historical_capsule_zoom_120_40`,
  `draw_historical_capsule_palette_120_40`.
- Observed mechanism: the commit adds a 1,615-line historical module.
- Dispatch requires 120x40, `Motion::Paused`, and `Scenario::CapsuleMulti`.
- Inner selection uses `capsule_tab_menu_open`, `capsule_new_tab_open`,
  `capsule_split_vertical_open`, `capsule_palette_open`, and `capsule.zoomed`.
- The size-specific path returns before the ordinary `draw_layers` call.
- Guard hunks suppress normal menu and palette work under the same gate.
- A screenshot can match while the normal component path never runs.
- Affected component: Capsule Tabs, SplitPane, TerminalView, Menu/Picker.
- Affected preview: `jackin-preview-app`.
- Violated requirement: ARC-009, ARC-012 (paused bypass, overlay before/after
  component drawing).
- Violated requirement: VER-004, VER-005.
- Required documentation change: require the same component and input paths for
  fixture, paused, and normal execution. Allow only supplied data and declared
  time/motion policy to differ.
- Future implementation owner: FIX-004.
- Rejecting test: run capsule menu, tabs, splits, palette, and zoom under every
  motion policy with normal input. Expect normal layer behavior in each run.
- Trace to HEAD: active. Module is declared at `app.rs:246`.
  Dispatch lives at `app.rs:11676`–`app.rs:11700`.

## 4. Cross-preview pattern sweep

Verified by `grep` at HEAD. Each row names file and symbol.

| Preview | File | Symbol / mechanism |
| --- | --- | --- |
| Holla | `crates/holla-ui/src/tui/runtime.rs` | `TerminalSession`, `render_frame`, `Application::render(&mut Frame)`; app renders through `ratatui::Frame`, not Termrock components. |
| Holla | `crates/holla-ui/src/tui/widgets/code.rs:654` | `buf.set_stringn` paints editor text into a foreign `Buffer`. |
| Holla | `crates/holla-ui/src/tui/widgets/segments.rs:116` | `buf.set_string` paints segments into a foreign `Buffer`. |
| Holla | `crates/holla-ui/src/tui/widgets/choice.rs:20` | `buf.set_stringn` paints choices into a foreign `Buffer`. |
| TablePro | `crates/tablepro-ui/src/safety_dialog.rs:239` | `SafetyDialog::draw` uses `ui.fill` and `ui.paint_str` for borders and rows. |
| TablePro | `crates/tablepro-ui/src/filter_editor.rs` | Direct `paint_str`/`fill` usage (file-level match). |
| TablePro | `crates/tablepro-ui/src/app.rs` | Direct `paint_str`/`fill` usage (file-level match). |
| Showcase | `crates/showcase-demos/src/pages/mod.rs:100` | `paint_clipped_meta` helper uses `ui.paint_str`. |
| Showcase | `crates/showcase-demos/src/pages/terminal.rs:122` | `paint_narrow_rail` helper uses `ui.paint_str`. |
| Showcase | `crates/showcase-demos/src/pages/terminal.rs:419` | `paint_narrow_adornments` helper uses `ui.paint_str`. |

Note: `runtime.rs:525` also calls `Buffer::set_string`, but that call sits in a
test module. The production foreign-runtime evidence is the `render_frame` /
`Application::render` path plus the widget `set_string` calls above.
Future implementation owner for all rows: FIX-005.

## 5. Checklist evidence audit

Do not edit `checklist.json`. Another workstream owns it. This section records
observations only.

### 5.1 Missing evidence objects

Three cited SHAs do not exist in this repository:

| Cited SHA | `git cat-file -t` result |
| --- | --- |
| `2e93120438ec908c69bf901b09b93be66ecacddc` | `fatal: bad object` |
| `64dec781f9a2ba14a8409393a52eec89e90099ee` | `fatal: bad object` |
| `ad73ae9f0500ddd02d64aad142bbecb2122c0617` | `fatal: bad object` |

Any checklist row citing these objects as evidence must reopen. Missing objects
cannot support a `verified` status.

Two cited SHAs share a 9-character prefix with real commits but diverge after
it. The real commits are `2e93120434854665ba10d15aae5dd3c172380bf3`
(`fix(forms): enter editing on text field focus`) and
`64dec781f237ac53efbea948dea543120e528a8b`
(`ci: regenerate workflows with job-level env hoisting`). The cited
`...38ec908c...` and `...f9a2ba1...` suffixes resolve to nothing. Treat the
cited values as wrong, not as shorthand. The third value, `ad73ae9f0`, names a
`velnor-actions` generator revision from another repository (see commit
`f93a0ccac`), not an object in this repository.

### 5.2 Path-scoped staleness

Both SHAs below exist as commits. Later work touched their scopes.

- `80d6a19a543c4ac1ec6b995b973427240319803e`
  (`fix(holla): gate CleanupPage tick updates on running cleanup job`).
  `git log <sha>..HEAD -- crates/holla-ui crates/holla-domain crates/holla`
  returns one commit: `389f3d644`. Evidence scoped to Holla crates is stale.
- `a12dd2fc51cf152929536c626020ba3dd1f88f75`
  (`fix(theme): restore mono rule for error field underline to preserve rule counts`).
  `git log <sha>..HEAD -- crates/termrock-theme` returns two commits:
  `9f3e04fe6`, `d0b047769`. Evidence scoped to `termrock-theme` is stale.

Rows citing these SHAs as current-state evidence must reopen or re-verify
against HEAD.

### 5.3 Named row defects

- G00-01 tip staleness: the row records tip `99554cfa49a8c951fb0aebdde61f7fc857f73f9a`.
  HEAD is `658e983a6112f2d515e5a44591f084190822d209`. The recorded tip is an
  ancestor of HEAD with 97 commits between them. The row must reopen.
- G01-09 dimension mismatch: evidence cites `80x24, 100x30, 120x36, 140x42, 160x48`.
  The established matrix in code and ARC-012 is
  `72x20, 80x24, 100x30, 120x40, 160x50`.
  (Evidence: `draw_historical_manager_{72_20,80_24,100_30,120_40,160_50}` at HEAD
  and the C3 size-ladder hunk.) Three of five cited sizes match no matrix entry.
  The row must reopen.
- G01-10 store-integrity-as-parity: evidence cites two `store_integrity` runs over
  75,500 files with zero drift. Integrity checks stored data. It does not execute
  application interactions. VER-001 states this separation explicitly.
  The row must reopen as an oracle-qualification claim.
- Self-reviewed G00/G01 rows: all G00-01..G00-07 and G01-01..G01-10 rows carry
  status `verified` with the same coordinator identity as author and reviewer.
  No independent reviewer approved them. The rows predate the exact-subject
  review procedure. Each row needs independent re-review at implementation
  acceptance (FIX-018). Rows with additionally stale or unrelated evidence
  reopen now: G00-01, G01-09, G01-10. The rest keep their status until that
  re-review. Do not reset them without review.

## 6. Finding-to-work linkage

| Finding | Source commit | DOC row | FIX owner | Status at HEAD |
| --- | --- | --- | --- | --- |
| F-C1 | `9f3e04fe` | DOC-003, DOC-004 | FIX-001 | Historical (reverted by `6fc94a4d1`); tablepro library fix active and conforming. |
| F-C2 | `fbca27983` | DOC-003, DOC-004 | FIX-001 | Active (`app.rs:4976`, `app.rs:5033`, `app.rs:11662`). |
| F-C3 | `d66376391` | DOC-003, DOC-004 | FIX-002 | Active (`app.rs:7481`+, `app.rs:11814`). |
| F-C4 | `087a1af43` | DOC-003, DOC-004 | FIX-003 | Active (`app/historical_editor_cockpit.rs`, `app.rs:11889`+). |
| F-C5 | `34b28f6eb` | DOC-003, DOC-004 | FIX-004 | Active (`app/historical_capsule.rs`, `app.rs:11676`). |
| Cross-preview sweep | n/a (HEAD grep) | DOC-005 | FIX-005 | Active. |
| Checklist audit | n/a (HEAD refs) | DOC-042 | n/a (checklist owner) | Open defects recorded. |
