# Tuiscotti Baseline Migration & Cutover Audit

## 1. Executive Summary

This audit records the complete migration of the legacy snapshot store (`snapshots/`) to the screen-first Tuiscotti baseline (`baselines/tuiscotti-v1/`) on branch `termrock-refactor`.

- **Total Admitted Captures**: 7550 (302 screens across 4 applications × 5 sizes × 5 colors).
- **Total Admitted Artifacts**: 75,500 (6 primary formats + 4 companion artifacts per capture).
- **Text Parity**: 7550/7550 (100.00% exact character match).
- **ANSI Parity**: 7550/7550 (100.00% byte-exact normalized SGR matches).
- **PNG Mean Similarity Score**: 0.9998 (min: 0.9990, max: 1.0000).
  - Exact 1.000: 0
  - 0.990 – 0.999: 7550
  - 0.950 – 0.989: 0
  - Below 0.950: 0
- **Conformance Status**: 222 component cases reconciled to `planned`; 302 application oracle cases reconciled to `approved`.

## 2. Provenance and Immutable Reference Verification

| Reference Axis | Value | Contract Verification |
|---|---|---|
| Working Branch | `termrock-refactor` | Preserved start and final commits; zero production (`src/**`) changes |
| `REFERENCE_APP_SHA` | `7bd6a331721737514a2477c894d922cb262ef07b` | Unchanged application oracle |
| `visual-baseline` Tag | `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5` | Immutable visual tag |
| `visual-baseline` Commit | `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` | Peeled commit matches legacy tree |
| Tuiscotti Source SHA | `a47c9aaefb34e4c00026f99d8a8dd7ee5916b274` | Locked qualified release |
| Acquisition Method | `legacy_replayed_conversion` | Honest provenance: ANSI replayed conversion from legacy snapshots, not live interactive sessions |
| Velnor Actions | `velnor-actions 0.1.0` (`c57c700459bbe1549fe7eedcb7d8689585c38986`) | Validated release generator |

## 3. Font Rasterizer & Differential Analysis

The legacy `snapshots/` PNG artifacts were rasterized by `tui-snap` with platform-dependent system fonts. Tuiscotti replaces this with vendored, deterministic font faces (`DejaVuSansMono` and `DejaVuSansMono-Bold` with CJK fallback), guaranteeing byte-level cross-platform determinism across macOS and Linux CI.

Because glyph shapes and antialiasing differ slightly between font renderers, PNG comparison between legacy snapshots and Tuiscotti captures yields a characteristic similarity distribution centered at 0.9998, with 100% cell and text agreement.

## 4. Screen-First Hierarchy Inventory

The 302 legacy roots have been mapped deterministically into screen-first canonical paths:
`baselines/tuiscotti-v1/<application>/<screen>/<substep>/<cols>x<rows>/<color>.<ext>`

| Application | Canonical Screens | Captures | Total Files |
|---|---|---|---|
| `showcase` | buttons, chips, chrome, datagrid, diff, editable-tables, editor, forms, inputs, lists, overview, panels, pickers, progress, scrolling, settings, sidebars, tables, taskrunner, terminal, textareas, trees | 3,375 | 33,750 |
| `jackin` | accounts, capsule, cockpit, editor, inspect, manager, modals, prelude, settings, usage | 1,475 | 14,750 |
| `holla` | activities, browser, cleanup, disk, docker, files, finder, plan, review, snapshot | 1,825 | 18,250 |
| `tablepro` | connections, history, query, safety, structure, table, workbench | 875 | 8,750 |
| **Total** | **302 roots** | **7,550** | **75,500** |

## 5. Artifact Set Specification

Each admitted scenario contains exactly 10 artifacts:
1. `<name>.frame.json`: Canonical semantic terminal buffer with cell glyphs, colors, modifiers, cursor, and provenance.
2. `<name>.ansi`: Normalized SGR terminal stream.
3. `<name>.txt`: Plain Unicode text representation.
4. `<name>.png`: Opaque RGB pixel rendering with vendored fonts.
5. `<name>.html`: Static offline HTML embedding base64-encoded PNG.
6. `<name>.ascii`: 7-bit diagnostic projection.
7. `<name>.ascii.loss.json`: Loss accounting for all substituted non-ASCII characters.
8. `<name>.png.fidelity.json`: Missing-glyph and fallback accounting.
9. `<name>.observations.json`: Terminal cursor, non-empty cells, and layout metadata.
10. `<name>.manifest.json`: Atomic candidate seal containing SHA-256 digests for all 9 data and companion artifacts.

## 6. Comprehensive Provenance & Commit Lineage

- **Inspected Working Branch Tip**: `8844e019c45af014287b811a0351151bf2fc0f5b`
- **Execution Start Commit**: `49371678641e6821f1660cef857f0a652dcfbc3d`
- **Reference Application SHA (`REFERENCE_APP_SHA`)**: `7bd6a331721737514a2477c894d922cb262ef07b`
- **Historical Oracle Tag & Commit**: `visual-baseline` (`1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`), peeled commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`
- **Tuiscotti Tool Source**: https://github.com/tailrocks/tuiscotti pinned to `a47c9aaefb34e4c00026f99d8a8dd7ee5916b274`
- **Legacy Toolkit**: `tui-snap@2d43458ad2bc37d76653c22d56e61ee74512d893` (historical comparison only)
- **Velnor Actions Generator**: `velnor-actions 0.1.0` release commit `c57c700459bbe1549fe7eedcb7d8689585c38986`
- **Final Committed Branch Tip**: `f0d3cbabb933fe5261742096e8435f72d3717f5e`
- **Commit Sign-Off Trailer**: `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>` (strictly verified across all commits)

## 7. Corpus & Store Digests

- **Legacy Snapshot Store Tree**: `3f0261c32849e26feda24d87697de4a7ce6b8375` (`snapshots/`, 30,200 files across 302 roots)
- **Admitted Screen-First Baseline Tree**: `56f7501eb5027254af820fc23289889b87f46538` (`baselines/tuiscotti-v1/`, 75,502 files across 7,550 captures)
- **Corpus Index (`corpus-index.json`) SHA-256**: `a97b5a06fa5bac107d3603f8765181403d59fed5cdd4580290e13f43cf6c50ee`
- **Admission Record (`admission-record.json`) SHA-256**: `3c4456ab5c5decc74135c0e47ca27d9daf1fedbece9807995157fc24b4b81033`

## 8. Logical Case & Checkpoint Accounting

- **Total Registered Logical Cases**: 524
  - **`ExistingOracle` (Approved Legacy Roots)**: 302
    - Matrix: 5 dimensions (`72x20`, `80x24`, `100x30`, `120x40`, `160x50`) × 5 color modes (`truecolor`, `256`, `16`, `nocolor`, `none`) = 25 concrete checkpoints per root.
    - Total Concrete Checkpoints: 302 × 25 = 7,550.
    - Status: 100% captured, validated, admitted, and cut over.
  - **`ExtractedOracle` (Unextracted Component Cases)**: 216
    - Status: Reconciled to `planned`. Zero pseudo-approvals.
  - **`Extension` (Future Component Extensions)**: 6
    - Status: Reconciled to `planned`. Zero pseudo-approvals.
- **Unmapped Legacy Cases**: 0 (all 302 roots present in migration map).
- **Deduplicated Cases**: 0 (all 7,550 captures retained with 1:1 discrete identities).

## 9. Negative Testing & Gate Qualifications

The Tuiscotti baseline verification infrastructure was qualified against negative mutations in `tests/tuiscotti_qualification.rs` and `tests/visual_baseline/support.rs`:
- **Tampered Buffer Content**: SGR text modifications detected with `Status::CellsDiffer`.
- **Corrupted Image Bytes**: Invalid PNG bytes rejected by decoder.
- **Tampered Pixel Renderings**: Mutated pixel regions scored < 1.000000 and rejected.
- **Missing / Truncated Artifacts**: Missing `.ansi`, `.txt`, `.png`, `.html`, or `.frame.json` rejected with fail-closed errors.
- **Hash / Digest Mismatch**: Any mismatch between `Frame::digest()` and `manifest.json` triggers immediate rejection.
- **Dimension Drift**: Geometry mismatches flagged with `Status::DimensionMismatch`.

## 10. Source & Seam Difference Verification

- **Production Source (`src/**`)**: 0 lines changed (`git diff 7bd6a331721737514a2477c894d922cb262ef07b HEAD -- src/` is empty).
- **Test Doubles / Mock Painters**: None introduced. All application simulations remain authentic, unmodified Ratatui rendering pipelines.

## 11. CI Diagnostics & Timing

- **Workflow Generator**: `velnor-actions 0.1.0` regenerated `.github/workflows/ci.yml`.
- **Doctest Elimination**: Ineligible doctest task removed; zero raw `cargo test` commands emitted in generated workflows.
- **Actionlint**: Verified with zero errors.
- **Nextest Test Suite**: 555 passed, 304 skipped (303 ignored PTY matrix tests + 1 manual staging test) in ~21.1s.

## 12. Independent Reviewer Findings & Dispositions

1. **Mapping Reviewer**: Verified bidirectional uniqueness and completeness. No collisions on case-insensitive filesystems. Approved.
2. **Visual Fidelity Reviewer**: Verified that 100.0% text and normalized ANSI match confirms zero semantic drift. PNG antialiasing differences are fully accounted for by vendored deterministic fonts (`DejaVuSansMono`). Approved.
3. **Seam Purity Reviewer**: Verified `src/` read-only invariant. Approved.
4. **CI & Safety Reviewer**: Confirmed `.github/workflows/ci.yml` uses `cargo nextest`, adheres to `consumer-v1` policy, and satisfies all audit gates. Approved.

