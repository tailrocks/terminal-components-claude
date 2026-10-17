# Main Branch Structural & Architectural Comparison

**Evaluation Date:** 2026-10-04  
**Evaluator:** Coordinator (Gemini 3.8 Flash · high)  
**Main Reference SHA:** `84482d066c5f0bc531f875f7f9d7716929c1b9c6`  
**Working Branch:** `termrock-implementation` (`99554cfa49a8c951fb0aebdde61f7fc857f73f9a`)  
**Visual Oracle:** `refs/tags/visual-baseline` (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`)

---

## 1. Executive Summary & Purpose

The `main` branch at `84482d066c5f0bc531f875f7f9d7716929c1b9c6` is inspected exclusively as a reference for **structure, API ergonomics, ownership boundaries, and linting policies**. Under the canonical migration rules:
- **Never use `main` as a visual oracle**: Expected appearance, frames, PNGs, and baseline interactions are exclusively defined by the immutable tag `visual-baseline`.
- **Never copy `main` renderer wholesale**: `main` retains legacy Junie rendering structures and an earlier 6-package architecture (`crates/tui`, `crates/tui-testing`, `apps/showcase`, `apps/tablepro`, `apps/jackin-preview`, `apps/holla`).
- **Target Architecture**: We establish a virtual workspace with 44 functionality-based packages physically located under `crates/`, eliminating all root `src/` and `apps/` trees and removing all legacy Junie code.

---

## 2. Workspace & Root Configuration Comparison

| Aspect | `main` (`84482d06`) | Target Architecture (`termrock-implementation`) | Decision & Rationale |
| :--- | :--- | :--- | :--- |
| **Workspace Type** | Root virtual workspace (`[workspace]`) | Root virtual workspace (`[workspace]`) | **Keep**: Virtual workspace pattern eliminates root package confusion and ensures uniform tooling. |
| **Resolver** | `resolver = "3"` | `resolver = "3"` | **Keep**: Modern Cargo edition 2024 resolver ensures exact and deterministic crate graph resolution. |
| **Rust Version** | `1.88` / `edition = "2024"` | `1.88` / `edition = "2024"` | **Keep**: Preserves toolchain consistency across development and CI environments. |
| **Workspace Members** | `crates/tui`, `crates/tui-testing`, `xtask`, `apps/*` (6 packages) | 44 packages under `crates/*` | **Adapt & Expand**: Move all application packages under `crates/` (`crates/showcase-*`, `crates/jackin-preview-*`, etc.) and decompose `crates/tui` into 19 library crates. |
| **Root Source Trees** | `apps/` tree at root | Zero code at root; only `crates/` contains first-party Rust | **Reject**: Root `apps/` violates the invariant that all first-party Rust packages live inside `crates/`. |
| **Lints & Warnings** | Strict `[workspace.lints]` with denied unsafe, panics, unwrap, indexing | Adopt comprehensive lint policies matching architectural requirements | **Keep & Adapt**: Enforce strict linting across all workspace crates with explicit exemptions only where justified. |

---

## 3. Library Architecture Comparison

### `main` Approach: Monolithic `crates/tui`
In `main`, all foundations, core types, theme definitions, runtime mechanisms, and all 45 components reside in a single crate (`crates/tui`).
- **Defects of Monolithic Approach:**
  - Changes to basic primitives (e.g., `Id`, `Style`) force recompilation of complex components (`Grid`, `CodeEditor`, `Dialog`).
  - No physical compiler-enforced boundary between pure layout/text math and stateful widgets.
  - Risk of unintentional circular coupling between components (e.g., `Field` depending on `Select` popup, or `Menu` depending on `Form`).

### Target Approach: 19 Functionality-Based Library Crates
We decompose the library into 18 focused functionality crates plus 1 ergonomic facade (`crates/termrock`):
1. `termrock-core`: Identity (`Id`, `ItemKey`), `Revision`, normalized input events, `Response`, `Moment`. (Allowed internal deps: none).
2. `termrock-layout`: Pure geometry, constraints, measurement math, clipping. (Allowed internal deps: `termrock-core`).
3. `termrock-theme`: Tokens, palettes, recipes, style patches, glyph sets. (Allowed internal deps: `termrock-core`).
4. `termrock-text`: Grapheme clusters, unicode width, secret protection, edit history. (Allowed internal deps: `termrock-core`, `termrock-theme`).
5. `termrock-render`: Cell painting primitives, layer buffer abstraction. (Allowed internal deps: `termrock-core`, `termrock-layout`, `termrock-theme`, `termrock-text`).
6. `termrock-collections`: Keyed model reconciliation, selection models, `ScrollState`. (Allowed internal deps: `termrock-core`, `termrock-layout`, `termrock-text`).
7. `termrock-runtime`: Frame publication, `Cx`/`Ui`, focus management, hit testing, pointer capture, layers, cursor arbitration. (Allowed internal deps: `termrock-core`, `termrock-layout`, `termrock-theme`, `termrock-render`, `termrock-collections`).
8. `termrock-controls`: `Brand`, `Button`, `Checkbox`, `Toggle`, `RadioGroup`, `Panel`, `SplitPane`, `Props`, `Empty`, `TooSmall`.
9. `termrock-fields`: `Field`, `TextInput`, `TextArea`.
10. `termrock-feedback`: `Spinner`, `ProgressBar`, `Meter`, `StatusBar`, `HintBar`, `KeyHint`.
11. `termrock-navigation`: `List`, `FilterList`, `NavList`, `Tree`, `Steps`, `Tabs`, `ChipBar`, `PropsList`.
12. `termrock-viewport`: `TextViewport`, `ScrollRegion`.
13. `termrock-overlays`: `Dialog`, `Menu`, `ContextMenu`, `MenuBar`, `Select`, `Picker`, `CommandPalette`, `PickerChain`, `Completion`, `HelpOverlay`.
14. `termrock-grid`: Tabular and cell-grid engine (`Grid`).
15. `termrock-editors`: `CodeEditor`, `DiffView`.
16. `termrock-forms`: `Form`, `Wizard`.
17. `termrock-terminal`: `TerminalView`.
18. `termrock-session`: Platform terminal backend entry/exit and suspend/resume.
19. `termrock`: Curated public facade re-exporting all functionality crates.

**Decision:** **Reject monolithic `crates/tui`**; **Adopt 19-crate functionality decomposition**.

---

## 4. Application Architecture Comparison

### A. Showcase (`showcase`)
- **`main` Structure:** Single package `apps/showcase` with internal submodules (`data.rs`, `app.rs`, `pages/*`).
- **Target Decomposition:**
  - `crates/showcase-data`: Pure deterministic showcase records without UI dependency.
  - `crates/showcase-demos`: Page implementations demonstrating Termrock controls.
  - `crates/showcase-ui`: App shell, navigation sidebar, inspector, help overlay.
  - `crates/showcase`: Thin CLI binary parsing flags and bootstrapping session.
- **Decision:** **Keep** page implementations and data logic; **Adapt** into 4 crates; **Reject** monolithic `apps/showcase`.

### B. Jackin Preview (`jackin-preview`)
- **`main` Structure:** Single package `apps/jackin-preview` with internal `domain/`, `sim/`, `screens/`.
- **Target Decomposition:**
  - `crates/jackin-preview-domain`: Pure workspace, account, and instance records.
  - `crates/jackin-preview-sim`: Simulation engines, virtual clock, fixture providers.
  - `crates/jackin-preview-presentation`: Shared view projections and animated ASCII art (`rain.rs`).
  - `crates/jackin-preview-host-ui`: Host screens (manager, prelude, editor, settings, accounts).
  - `crates/jackin-preview-capsule-ui`: Capsule screens (tabs, terminal pane, inspect overlays).
  - `crates/jackin-preview-app`: Top-level router and arbiter.
  - `crates/jackin-preview`: Thin binary CLI.
- **Decision:** **Keep** domain rules and simulation logic; **Adapt** into 7 crates; **Reject** screen-to-app circular dependencies.

### C. Holla (`holla`)
- **`main` Structure:** Single package `apps/holla` with `domain/`, `sim/`, `screens/`.
- **Target Decomposition:**
  - `crates/holla-domain`: Domain actions, activities, context, and effect types.
  - `crates/holla-catalog`: Catalog indexing, ranking, and manifest parsing.
  - `crates/holla-plan`: Plan graph validation and dependency resolution.
  - `crates/holla-sim`: Virtual filesystem and deterministic simulation world.
  - `crates/holla-ui`: Finder, plan tree, resource views, and navigation.
  - `crates/holla`: Thin binary entry point.
- **Decision:** **Keep** catalog ranking and plan engine; **Adapt** into 6 crates; **Reject** mixing domain types with Ratatui widgets.

### D. TablePro (`tablepro`)
- **`main` Structure:** Single package `apps/tablepro` where `model.rs` and `tabs.rs` contain domain logic directly importing widget cell values.
- **Target Decomposition:**
  - `crates/tablepro-domain`: Pure table schema, rows, and pending change models (zero UI dependency).
  - `crates/tablepro-sql`: Pure SQL parsing, completion analysis, and SQL generation.
  - `crates/tablepro-demo`: In-memory SQLite/catalog simulation.
  - `crates/tablepro-ui`: Workbench, grid adapters, and editor dialogs using `termrock-grid`.
  - `crates/tablepro`: Thin binary CLI.
- **Decision:** **Keep** SQL generation and safety vectors; **Adapt** into 5 crates; **Reject** coupling domain models to `Grid` cell types.

---

## 5. Verification & Tooling Infrastructure Comparison

| Component | `main` Reference | Target Architecture | Decision |
| :--- | :--- | :--- | :--- |
| **Test Support** | `crates/tui-testing` | `crates/termrock-test-support` | **Adapt**: Provide synthetic fixture helpers, event recorders, and checkpoint definitions. |
| **Conformance & Visual** | `apps/*/tests/visual.rs` | `crates/termrock-conformance` | **Adapt & Expand**: Unify component and full-app conformance and visual regression tests under an isolated crate. |
| **Repository Automation** | `xtask` | `crates/termrock-xtask` | **Adapt**: Maintain explicit checklist validators, baseline comparison utilities, and CI generation helpers. |

---

## 6. Synthesis: Keep / Adapt / Reject Matrix

| Area | Keep from `main` | Adapt from `main` | Reject from `main` |
| :--- | :--- | :--- | :--- |
| **Workspace** | `resolver = "3"`, `edition = "2024"`, lockfile discipline | Profile optimizations, shared dependency versions | Root `apps/` folder, monolithic root packages |
| **Library** | Public API ergonomics, method naming conventions | Component signatures to use borrowed models & typed responses | Single monolithic `crates/tui` crate, legacy Junie types |
| **Applications** | Scenario definitions, deterministic fixtures, CLI arguments | App internal module organization into multi-crate hierarchy | Mixing UI widgets into domain/sim modules |
| **Visuals** | None (Visual oracle is strictly `refs/tags/visual-baseline`) | None | Any rendering or frame changes introduced in `main` |

---
*Document authored as Gate G00-03 deliverable for Termrock Multi-Crate Migration.*
