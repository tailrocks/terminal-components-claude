# Rust Source, Target, and Consumer Census

**Evaluation Date:** 2026-10-04  
**Evaluator:** Coordinator (Gemini 3.8 Flash · high)  
**Branch:** `termrock-implementation` (`99554cfa49a8c951fb0aebdde61f7fc857f73f9a`)  
**Scope:** Complete inventory of first-party Rust files across `src/` and `tests/`, targets, namespaces, rendering paths, and migration classification.

---

## 1. Target and Package Structure Overview

### Current State (Unmigrated Monolithic Package)
- Root `Cargo.toml`: Package `junie-tui` (v0.1.0)
- Library: `src/lib.rs` (compiles `junie_tui`)
- Binaries:
  - `src/bin/showcase/main.rs` (`showcase`)
  - `src/bin/tablepro/main.rs` (`tablepro`)
  - `src/bin/jackin_preview/main.rs` (`jackin-preview`)
  - `src/bin/holla/main.rs` (`holla`)
- Tests: 28 test suites under `tests/` compiling against `junie_tui`.

### Target Architecture (Virtual Workspace with 44 Crates)
- Root `Cargo.toml`: Virtual workspace (`resolver = "3"`), zero root packages.
- 44 Crates under `crates/`:
  - 19 Library/Facade crates (`termrock-core` through `termrock`)
  - 4 Showcase crates (`showcase-data`, `showcase-demos`, `showcase-ui`, `showcase`)
  - 7 Jackin Preview crates (`jackin-preview-domain`, `jackin-preview-sim`, `jackin-preview-presentation`, `jackin-preview-host-ui`, `jackin-preview-capsule-ui`, `jackin-preview-app`, `jackin-preview`)
  - 6 Holla crates (`holla-domain`, `holla-catalog`, `holla-plan`, `holla-sim`, `holla-ui`, `holla`)
  - 5 TablePro crates (`tablepro-domain`, `tablepro-sql`, `tablepro-demo`, `tablepro-ui`, `tablepro`)
  - 3 Tooling/Verification crates (`termrock-test-support`, `termrock-conformance`, `termrock-xtask`)

---

## 2. Source Classification Taxonomy

- **Legacy (`LEGACY`):** Old Junie modules located in `src/core/`, `src/ui/`, `src/widgets/`, `src/theme.rs`, `src/runtime.rs`. To be eliminated completely once Termrock crates and consumer apps are migrated.
- **New Termrock (`NEW`):** Termrock library modules located in `src/termrock/` developed during P1–P7. To be extracted into the 19 dedicated `termrock-*` crates under `crates/`.
- **Application (`APP`):** Application code located in `src/bin/*`. Currently importing mixed legacy widgets and Termrock APIs. To be decomposed into multi-crate application packages under `crates/`.
- **Verification (`TEST`):** Integration and regression tests located in `tests/`. To be migrated into `crates/termrock-conformance/`, `crates/termrock-test-support/`, and per-crate unit tests.

---

## 3. Inventory by Directory & Module

### A. Legacy Core, Theme, UI & Widgets (`src/`)

| File Path | Lines | Classification | Current Imports / Exports | Destination Crate | Removal Condition |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `src/core/event.rs` | 243 | `LEGACY` | Junie event types, key modifiers | `termrock-core` (reconciled) | Delete when `termrock-core` input events replace all uses |
| `src/core/focus.rs` | 187 | `LEGACY` | Focus ring, tab traversal | `termrock-runtime` | Delete when `termrock-runtime` focus ring replaces all uses |
| `src/core/hit.rs` | 142 | `LEGACY` | Hit detection rects | `termrock-runtime` | Delete when `termrock-runtime` hit testing replaces all uses |
| `src/core/id.rs` | 115 | `LEGACY` | Control `Id` | `termrock-core` | Delete when `termrock-core::Id` replaces all uses |
| `src/core/scroll.rs` | 178 | `LEGACY` | Old scroll state | `termrock-collections` | Delete when `termrock-collections::ScrollState` replaces all uses |
| `src/core/text.rs` | 312 | `LEGACY` | Text measurement | `termrock-text` | Delete when `termrock-text` replaces all uses |
| `src/core/mod.rs` | 45 | `LEGACY` | Module exports | N/A | Delete with `src/core/` |
| `src/runtime.rs` | 512 | `LEGACY` | Legacy generic loop | `termrock-runtime` / `termrock-session` | Delete when `termrock-runtime` and `termrock-session` replace all uses |
| `src/theme.rs` | 480 | `LEGACY` | Legacy theme definitions | `termrock-theme` | Delete when `termrock-theme` replaces all uses |
| `src/ui/ctx.rs` | 290 | `LEGACY` | Legacy paint context | `termrock-render` | Delete when `termrock-render` replaces all uses |
| `src/ui/fade.rs` | 85 | `LEGACY` | Scroll edge fade | `termrock-viewport` | Delete when `termrock-viewport` replaces all uses |
| `src/ui/layout.rs` | 210 | `LEGACY` | Constraints math | `termrock-layout` | Delete when `termrock-layout` replaces all uses |
| `src/ui/popup.rs` | 140 | `LEGACY` | Legacy popup placement | `termrock-overlays` | Delete when `termrock-overlays` replaces all uses |
| `src/ui/text.rs` | 195 | `LEGACY` | Grapheme cluster utilities | `termrock-text` | Delete when `termrock-text` replaces all uses |
| `src/ui/mod.rs` | 35 | `LEGACY` | UI re-exports | N/A | Delete with `src/ui/` |
| `src/widgets/*.rs` (32 files) | ~8,400 | `LEGACY` | All 32 legacy Junie widgets | Various `termrock-*` crates | Delete when all apps and tests migrate to `termrock-*` |

### B. New Termrock Implementation (`src/termrock/`)

| Module Path | Lines | Classification | Components / Responsibilities | Owning Destination Crate |
| :--- | :--- | :--- | :--- | :--- |
| `src/termrock/identity.rs` | 340 | `NEW` | `Id`, `ItemKey`, `Revision`, `Moment` | `crates/termrock-core` |
| `src/termrock/response.rs` | 420 | `NEW` | `Response<T>`, `Activated`, `ActivationOrigin` | `crates/termrock-core` |
| `src/termrock/layout.rs` | 380 | `NEW` | `Constraints`, `Size`, pure tracks/layout | `crates/termrock-layout` |
| `src/termrock/theme.rs` | 510 | `NEW` | `Theme`, tokens, `StylePatch`, recipes | `crates/termrock-theme` |
| `src/termrock/text.rs` | 550 | `NEW` | `TextSource`, grapheme measurement, clusters | `crates/termrock-text` |
| `src/termrock/secret.rs` | 180 | `NEW` | `Secret`, `SecretText`, volatile zeroization | `crates/termrock-text` |
| `src/termrock/author.rs` | 140 | `NEW` | Low-level cell drawing facts | `crates/termrock-render` |
| `src/termrock/collections.rs` | 260 | `NEW` | Reconciliation, `SelectionMode`, selection | `crates/termrock-collections` |
| `src/termrock/scroll.rs` | 330 | `NEW` | `ScrollState` math and `ScrollRegion` widget | Math -> `termrock-collections`; Widget -> `termrock-viewport` |
| `src/termrock/runtime.rs` | 580 | `NEW` | `Cx`, `Ui`, focus ring, hit regions, pointer | `crates/termrock-runtime` |
| `src/termrock/layers.rs` | 310 | `NEW` | Overlay layer stack and modal trapping | `crates/termrock-runtime` |
| `src/termrock/button.rs` | 430 | `NEW` | W02 `Button` | `crates/termrock-controls` |
| `src/termrock/brand.rs` | 210 | `NEW` | W01 `Brand` | `crates/termrock-controls` |
| `src/termrock/checkbox.rs` | 290 | `NEW` | W03 `Checkbox` | `crates/termrock-controls` |
| `src/termrock/toggle.rs` | 310 | `NEW` | W04 `Toggle` | `crates/termrock-controls` |
| `src/termrock/radio_group.rs` | 520 | `NEW` | W05 `RadioGroup` | `crates/termrock-controls` |
| `src/termrock/panel.rs` | 280 | `NEW` | W32 `Panel` | `crates/termrock-controls` |
| `src/termrock/split_pane.rs` | 360 | `NEW` | W33 `SplitPane` | `crates/termrock-controls` |
| `src/termrock/props.rs` | 160 | `NEW` | W34 `Props` | `crates/termrock-controls` |
| `src/termrock/empty.rs` | 220 | `NEW` | W36 `Empty` | `crates/termrock-controls` |
| `src/termrock/too_small.rs` | 95 | `NEW` | W43 `TooSmall` | `crates/termrock-controls` |
| `src/termrock/field.rs` | 120 | `NEW` | W07 `Field` | `crates/termrock-fields` |
| `src/termrock/text_input.rs` | 680 | `NEW` | W08 `TextInput` | `crates/termrock-fields` |
| `src/termrock/text_area.rs` | 780 | `NEW` | W09 `TextArea` | `crates/termrock-fields` |
| `src/termrock/spinner.rs` | 90 | `NEW` | W38 `Spinner` | `crates/termrock-feedback` |
| `src/termrock/progress_bar.rs` | 180 | `NEW` | W37 `ProgressBar` | `crates/termrock-feedback` |
| `src/termrock/meter.rs` | 270 | `NEW` | W39 `Meter` | `crates/termrock-feedback` |
| `src/termrock/status_bar.rs` | 330 | `NEW` | W40 `StatusBar` | `crates/termrock-feedback` |
| `src/termrock/hint_bar.rs` | 140 | `NEW` | W41 `HintBar` | `crates/termrock-feedback` |
| `src/termrock/key_hint.rs` | 55 | `NEW` | W42 `KeyHint` | `crates/termrock-feedback` |
| `src/termrock/list.rs` | 390 | `NEW` | W12 `List` | `crates/termrock-navigation` |
| `src/termrock/filter_list.rs` | 210 | `NEW` | W13 `FilterList` | `crates/termrock-navigation` |
| `src/termrock/nav_list.rs` | 280 | `NEW` | W14 `NavList` | `crates/termrock-navigation` |
| `src/termrock/tree.rs` | 410 | `NEW` | W15 `Tree` | `crates/termrock-navigation` |
| `src/termrock/steps.rs` | 270 | `NEW` | W16 `Steps` | `crates/termrock-navigation` |
| `src/termrock/tabs.rs` | 360 | `NEW` | W17 `Tabs` | `crates/termrock-navigation` |
| `src/termrock/chip_bar.rs` | 490 | `NEW` | W06 `ChipBar` | `crates/termrock-navigation` |
| `src/termrock/props_list.rs` | 350 | `NEW` | W35 `PropsList` | `crates/termrock-navigation` |
| `src/termrock/text_viewport.rs` | 530 | `NEW` | W31 `TextViewport` | `crates/termrock-viewport` |
| `src/termrock/select.rs` | 820 | `NEW` | W10 `Select` | `crates/termrock-overlays` |
| `src/termrock/picker.rs` | 640 | `NEW` | W18 `Picker` | `crates/termrock-overlays` |
| `src/termrock/command_palette.rs` | 570 | `NEW` | W19 `CommandPalette` | `crates/termrock-overlays` |
| `src/termrock/picker_chain.rs` | 190 | `NEW` | W20 `PickerChain` | `crates/termrock-overlays` |
| `src/termrock/completion.rs` | 360 | `NEW` | W21 `Completion` | `crates/termrock-overlays` |
| `src/termrock/dialog.rs` | 410 | `NEW` | W22 `Dialog` | `crates/termrock-overlays` |
| `src/termrock/menu.rs` | 610 | `NEW` | W23 `Menu` | `crates/termrock-overlays` |
| `src/termrock/context_menu.rs` | 85 | `NEW` | W24 `ContextMenu` | `crates/termrock-overlays` |
| `src/termrock/menu_bar.rs` | 390 | `NEW` | W25 `MenuBar` | `crates/termrock-overlays` |
| `src/termrock/help_overlay.rs` | 270 | `NEW` | W26 `HelpOverlay` | `crates/termrock-overlays` |
| `src/termrock/grid.rs` | 980 | `NEW` | W28 `Grid` | `crates/termrock-grid` |
| `src/termrock/code_editor.rs` | 580 | `NEW` | W29 `CodeEditor` | `crates/termrock-editors` |
| `src/termrock/diff_view.rs` | 610 | `NEW` | W30 `DiffView` | `crates/termrock-editors` |
| `src/termrock/form.rs` | 310 | `NEW` | W11 `Form` | `crates/termrock-forms` |
| `src/termrock/wizard.rs` | 460 | `NEW` | W27 `Wizard` | `crates/termrock-forms` |
| `src/termrock/terminal_view.rs` | 510 | `NEW` | W44 `TerminalView` | `crates/termrock-terminal` |
| `src/termrock/mod.rs` | 75 | `NEW` | Public re-exports | `crates/termrock` (facade) |

### C. Application Binaries (`src/bin/`)

#### 1. Showcase (`src/bin/showcase/`)
- `data.rs` -> `crates/showcase-data` (pure deterministic demo records)
- `pages/*.rs` (21 files) -> `crates/showcase-demos` (demo page UI compositions)
- `app.rs` -> `crates/showcase-ui` (showcase shell and coordinator)
- `main.rs` -> `crates/showcase` (thin binary entry point)
- Tests (`app_tests.rs`, `app_tests_coverage.rs`) -> local integration tests in `crates/showcase-ui` and `crates/termrock-conformance`

#### 2. Jackin Preview (`src/bin/jackin_preview/`)
- `domain/*.rs` (7 files) -> `crates/jackin-preview-domain`
- `sim/*.rs` (7 files) + `clock.rs` -> `crates/jackin-preview-sim`
- `rain.rs` + shared presentation models -> `crates/jackin-preview-presentation`
- `screens/manager.rs`, `editor.rs`, `accounts.rs`, `cockpit.rs`, `settings.rs`, `prelude.rs`, `usage.rs`, `config.rs` -> `crates/jackin-preview-host-ui`
- `screens/capsule.rs`, `inspect.rs`, `modals.rs` -> `crates/jackin-preview-capsule-ui`
- `app.rs`, `arbiter.rs`, `scenario.rs` -> `crates/jackin-preview-app`
- `main.rs` -> `crates/jackin-preview` (thin binary entry point)

#### 3. Holla (`src/bin/holla/`)
- `domain/action.rs`, `activity.rs`, `cleanup.rs`, `context.rs`, `custom.rs`, `digest.rs`, `effect.rs`, `exec.rs`, `outcomes.rs`, `parity.rs`, `scripts.rs`, `stack.rs` -> `crates/holla-domain`
- `domain/catalog.rs`, `ranking.rs`, `manifest.rs`, `usage.rs` -> `crates/holla-catalog`
- `domain/plan.rs` -> `crates/holla-plan`
- `sim/*.rs` + `domain/fixtures.rs` -> `crates/holla-sim`
- `screens/*.rs`, `app.rs`, `scenario.rs` -> `crates/holla-ui`
- `main.rs` -> `crates/holla` (thin binary entry point)

#### 4. TablePro (`src/bin/tablepro/`)
- `connections.rs`, `model.rs` (records only) -> `crates/tablepro-domain`
- `sql.rs` -> `crates/tablepro-sql`
- `db.rs` -> `crates/tablepro-demo`
- `workbench.rs`, `tabs.rs`, `app.rs` -> `crates/tablepro-ui`
- `main.rs` -> `crates/tablepro` (thin binary entry point)

---

## 4. Test Suite Census (`tests/`)

| Test File | Description | Target Relocation |
| :--- | :--- | :--- |
| `tests/visual_baseline/*` (8 files) | Oracle store verification, full-app baseline checks | `crates/termrock-conformance` |
| `tests/conformance/*` (3 files) | Conformance test harness, required cases registry | `crates/termrock-test-support` & `crates/termrock-conformance` |
| `tests/termrock_*.rs` (17 files) | Unit and integration probe tests for Termrock components | Relocated to owning crate `tests/` and `crates/termrock-conformance` |
| `tests/external_consumer.rs` | External compilation verification | `crates/termrock-conformance` |
| `tests/doc_examples.rs` | Doc examples verification | `crates/termrock-conformance` |
| `tests/stage_corpus.rs` | Baseline capture staging utility | `crates/termrock-xtask` |

---
*Authored as Gate G00-04 deliverable for Termrock Multi-Crate Migration.*
