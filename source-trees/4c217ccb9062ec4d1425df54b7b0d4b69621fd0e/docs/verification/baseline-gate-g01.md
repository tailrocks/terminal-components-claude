# Gate G01 Verification: Baseline Lineage, Tool Qualification & Bundle Sealing

**Evaluation Date:** 2026-10-04  
**Evaluator:** Coordinator (Gemini 3.8 Flash · high)  
**Branch:** `termrock-implementation` (`a672100cd`)  
**Visual Oracle:** `refs/tags/visual-baseline` (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, annotated tag `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`)  
**Tuiscotti Pin:** `a47c9aaefb34e4c00026f99d8a8dd7ee5916b274`

---

## 1. Executive Summary

Gate G01 establishes the verification and baseline oracle contracts that will qualify the multi-crate Termrock implementation without circular self-certification. All 10 requirements (G01-01 through G01-10) are verified with executable evidence:

1. **Tag Oracle Lineage:** The immutable tag `visual-baseline` is frozen at peeled commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.
2. **Tuiscotti Qualification:** Verified with 4 passing tests in `tests/tuiscotti_qualification.rs` covering pure in-memory rendering across all 6 formats, PTY sessions, GroupedStore management, and negative mutation detection.
3. **Cryptographic Bundle Authentication:** All 75,500 artifacts across 7,550 captures are authenticated with exact byte counts and SHA-256 hashes against `baselines/tuiscotti-v1/corpus-index.json` (SHA-256: `a97b5a06fa5bac107d3603f8765181403d59fed5cdd4580290e13f43cf6c50ee`) via `store_integrity`.
4. **Honest Lineage:** Provenance is explicitly recorded as `legacy_replayed_conversion` in `admission-record.json`.
5. **Decoupled Admission:** Staging and baseline capture tools (`tests/stage_corpus.rs`) are physically separate from verification gates, preventing candidate self-approval.

---

## 2. Requirement Verification Matrix

| Gate ID | Requirement | Verification Evidence | Status |
| :--- | :--- | :--- | :--- |
| **G01-01** | Recover immutable tag oracle outside candidate | Tag `visual-baseline` (object `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`, commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`) verified read-only and immutable. External checkout preserved. | **VERIFIED** |
| **G01-02** | Qualify Tuiscotti and render profile | `tests/tuiscotti_qualification.rs` executed: 4/4 tests passed (0.63s). 6-format rendering, companion validators, and negative mutation detection confirmed. | **VERIFIED** |
| **G01-03** | Audit current baseline lineage | `docs/verification/tuiscotti-migration-audit.md` and `admission-record.json` record honest `acquisition_method: "legacy_replayed_conversion"`. | **VERIFIED** |
| **G01-04** | Reconcile all declared requirements | `tests/conformance/required_cases.json` reconciles 45 components, 12 foundations, 54 legacy families, 222 component cases, 302 application roots (524 total cases). | **VERIFIED** |
| **G01-05** | Bind real executable case drivers | Drivers implemented in `tests/conformance/drivers.rs` with typed inputs, event scripts, and semantic checkpoint bindings. | **VERIFIED** |
| **G01-06** | Capture missing original behavior | All 4 applications (`showcase`, `tablepro`, `jackin-preview`, `holla`) captured across 5 sizes and 5 colors (7,550 captures total). | **VERIFIED** |
| **G01-07** | Seal six-format and semantic bundles | Each scenario manifest seals 10 artifacts (`.ansi`, `.txt`, `.png`, `.html`, `.ascii`, `.ascii.loss.json`, `.png.fidelity.json`, `.observations.json`, `.frame.json`, `.manifest.json`). Total 75,500 files. | **VERIFIED** |
| **G01-08** | Separate capture from admission | Admission record `admission-record.json` contains cryptographic hash of `corpus-index.json`. Verification checks hashes read-only and cannot mutate admission. | **VERIFIED** |
| **G01-09** | Validate all states and transitions | Size ladder (80x24, 100x30, 120x36, 140x42, 160x48) and 5 color modes (TrueColor, ANSI256, ANSI16, Monochromatic, Grayscale) covered across all interactive states. | **VERIFIED** |
| **G01-10** | Run oracle requalification twice | Ran `store_integrity` twice independently; all 75,500 artifact hashes verified with zero drift. | **VERIFIED** |

---

## 3. Cryptographic Hashes and Signatures

- `corpus-index.json` SHA-256: `a97b5a06fa5bac107d3603f8765181403d59fed5cdd4580290e13f43cf6c50ee`
- `admission-record.json` Status: `admitted` (sealed 2026-10-03T22:09:44Z)
- Total Admitted Captures: 7,550
- Total Admitted Artifacts: 75,500
- Mean PNG Similarity: 0.9998
- Text Parity: 100.0%
- ANSI Normalized SGR Parity: 100.0%

---
*Gate G01 closed and verified.*
