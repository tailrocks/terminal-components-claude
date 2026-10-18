# Audit of Unfinished Work & Existing Completion Claims (Gate G00-05)

**Evaluation Date:** 2026-10-04  
**Evaluator:** Coordinator (Gemini 3.8 Flash · high)  
**Branch:** `termrock-implementation` (`99554cfa49a8c951fb0aebdde61f7fc857f73f9a`)  
**Scope:** Re-evaluating claims from P1–P7 commits, conformance driver dispatch, synthetic versus real parity, and CI execution.

---

## 1. Executive Summary

While commits up through `99554cfa` successfully implemented the 45 Termrock components and established passing test suites (841 passing tests in `cargo nextest`), treating the repository as refactoring-complete was premature. A rigorous audit reveals critical architectural gaps between the current state and the mandatory completion criteria:

1. **Monolithic Crate Remains Active:** Root `Cargo.toml` still defines `junie-tui` as a single monolithic package enclosing both the legacy code, the new Termrock modules, and all four application binaries.
2. **Dual Ownership in `src/lib.rs`:** `src/lib.rs` still exports legacy Junie modules (`core`, `runtime`, `theme`, `ui`, `widgets`) alongside `termrock`.
3. **Legacy Imports in Applications:** Inspected application rendering paths (`src/bin/jackin_preview/screens/manager.rs`, `src/bin/showcase/pages/buttons.rs`, etc.) continue to import legacy Junie widgets rather than consuming Termrock public APIs exclusively.
4. **Synthetic vs. Tag Parity Disconnect:** Conformance tests often compare candidate rendering against cloned buffer states rather than asserting true parity against the frozen `visual-baseline` tag oracle.
5. **Coupling Between Termrock and Legacy Core:** Certain `src/termrock` modules still reference legacy core types (e.g. legacy `Id` or `Event` conversions), preventing clean physical separation.
6. **CI Packaging and Publication Bottlenecks:** CI previously targeted the monolithic package, ignored visual test executions by default, and failed Velnor publication with `publish_refused:unprotected_ref`.

---

## 2. Detailed Audit Findings

### A. Root Workspace & Crate Structure
- **Finding:** The repository has not yet transitioned to a virtual workspace.
- **Evidence:** `Cargo.toml` at repository root contains `[package] name = "junie-tui"`, `[lib] path = "src/lib.rs"`, and four `[[bin]]` sections.
- **Required Action (G02):** Convert root `Cargo.toml` to a virtual workspace with `resolver = "3"` and no root targets. Establish 44 functionality crates under `crates/`.

### B. Dual Ownership & Legacy Pollution
- **Finding:** Legacy Junie code remains fully active and compiled into the library.
- **Evidence:** `src/lib.rs` contains:
  ```rust
  pub mod core;
  pub mod runtime;
  pub mod theme;
  pub mod ui;
  pub mod widgets;
  pub mod termrock;
  ```
- **Required Action (G03/G04):** Eliminate `src/lib.rs`, `src/core`, `src/ui`, `src/widgets`, `src/theme.rs`, and `src/runtime.rs`. Ensure zero legacy code or imports survive.

### C. Application Consumer Migration
- **Finding:** Applications in `src/bin/` have only partially adopted Termrock.
- **Evidence:** Showcase pages and Jackin screens still import `junie_tui::widgets::*` in multiple interactive and layout paths.
- **Required Action (G04):** Decompose each application into its target crates and rewrite all rendering code to consume public Termrock APIs exclusively.

### D. Conformance Drivers & Visual Gate Lineage
- **Finding:** Many parity checks qualifying components do so by cloning in-memory buffers or replaying converted legacy ANSI strings, rather than executing independent candidate drivers against tag-oracle expected captures.
- **Evidence:** Conformance tests qualify the comparator logic rather than proving exact visual match against `refs/tags/visual-baseline`.
- **Required Action (G01/G05):** Authenticate the entire 6-format bundle (ANSI, HTML, PNG, ASCII, TXT, frame JSON) against the frozen oracle commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.

### E. CI and Publication Failures
- **Finding:** The latest CI run failed during Velnor publication due to `publish_refused:unprotected_ref`, and the default nextest run skipped 304 visual baseline tests.
- **Evidence:** `.config/nextest.toml` sets `default-filter = "not test(rebuild_review_html)"` and visual tests are ignored unless explicitly passed `--run-ignored only`.
- **Required Action (G06):** Regenerate Velnor workflows for the virtual workspace, configure supported publication eligibility, and run both default and visual test suites explicitly.

---

## 3. Conclusion & Gate Decision

The 841 passing tests provide a functional foundation for the Termrock components, but Gate G00 establishes that substantial physical restructuring, consumer rewrites, legacy deletion, and visual gate expansion remain mandatory. Proceeding to Gate G01 and G02 is justified to resolve these findings systematically.

---
*Authored as Gate G00-05 deliverable for Termrock Multi-Crate Migration.*
