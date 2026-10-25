# T00 source and tool record

## Input

The frozen product source is `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`. Its tree is `f75288f7fae1f12e549c0f4fbfc5bb23734cb154`. The integration commit is `381c88c2f1a26bb727ab9658e8b694bea5915aa6`. It contains the frozen source plus plan records.

PR38 is open at head `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`. PR26 is open at head `5b3dbd7ae799f325efbb4f4cc11b2f5518dec948`. The base commits are in `source-state.json`. The common ancestor is `cc14dd6beae526884aabdf897e309be837b4f504`. The historical target is `e08ac57d10ae54c87337eae0be08b1f5e8f4bda4`. The `visual-baseline` tag object peels to `4169783ae883788186d161af09ba9c481bd84e5c`. The independent check found the PR26 reference worktree clean.

## Source inventory

The current tree ledger has 94,896 paths and 16,734,723,891 blob bytes. The current test-related ledger has 93,917 paths and 16,711,376,143 blob bytes. Each ledger row stores the path, language, Git blob ID, size, and SHA-256. The source inventory has 6,204 explicit Rust test attributes, including one new test at the frozen head. It has 282 Python test functions, 426 test target candidates, 5 Rustdoc fences, and 18 Markdown Rust examples.

The test inventory covers all languages and records Cargo targets, dependencies, helper modules, include paths, features, fixtures, doctest options, macro candidates, harnesses, CI files, Nextest settings, generated files, directives, legal files, diagnostics, and CLI help. It separates dev dependencies from normal dependencies in test-only packages. It marks unresolved module and macro cases as candidates. Current Velnor output has 14 generated files. Three older generated workflows remain archived evidence. The legal inventory has three license files. The bounded source scan found no SPDX identifier lines. It scanned 3,025 directive lines, including both extensionless `tc-proof` shebangs.

The 1d6 source adds `narrow_buttons_route_keeps_tree_markers_thumb_and_body`. It also changes one expected heading row in a fixture. These are source facts only. The assertion did not run. Its behavior is `NOT VERIFIED`.

The source refresh from 3a to 31f removes two ownership exception rows: OWN-11 for `showcase-demos/src/pages/terminal.rs` and OWN-14 for `termrock-shell/src/nav.rs`. The target also changes the ownership scanner. It adds a narrow enum range proof and changes OWN-13 to ignore a baseline string only when its full span is certified test code. Both `termrock-xtask` jobs cancelled at 1d6. No new ownership finding count is available.

## CI record

Run `40233372267` is bound to source `3a2c977c881cd1745de16425c982791ee3ef2cc3`. It has four failures: Holla visual replay and baseline cases, Jackin default and testing baselines, and the Required aggregate. Its 66 cancelled jobs make the production-build inventory incomplete. Logs and the 100-job REST response are in this directory.

Run `40235807469` is bound to source `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`. The full paginated response contains 100 jobs: 95 cancelled, one failed Required aggregate, two skipped, and two successful. The Required step ran `false` and exited 1. No explicit production-build step appears. Cancelled jobs leave production-build results unknown. Successful individual steps inside cancelled jobs do not pass their jobs.

The current Holla fixture expectation is unaccepted parity evidence. The latest CI run does not prove it.

## Ownership and safety

`crates/termrock-runtime/src/capture.rs` has production pointer capture and secondary-button routing. Keep this product code. `tools/capture.sh`, `tools/capture_exec.sh`, `tools/baseline_capture.sh`, and `tools/qualified-capture` are capture or evidence tools. Keep their evidence contract under review.

The visibility publisher has normal production dependencies and is a reporting library. Do not classify it as test-only by name. Preserve publisher, observer, proof CLI, and capture diagnostics until their consumers and output contracts have a migration record. `shell::reach` is a testing-feature harness; removal needs a reviewed migration record.

The ungated `RenderSnapshot`, `RenderSnapshotError`, `Runtime::render_snapshot`, and clock-paste diagnostic API are production surfaces. Preserve them. The testing feature also exposes model, projected-frame, runtime, and viewport work probes. Review their public consumers and lifecycle before removal. `IntentQueue` insertion and dispatch methods remain production APIs.

Keep `SAFETY.md`, workspace `unsafe_code = deny`, workspace `undocumented_unsafe_blocks = deny`, and the existing scoped refactor-proof directive. Do not widen an allowance. This inventory is not an unsafe-code review.

## Tools

Official release pages showed Rust 1.99.1, mise 2026.10.19, mbx 1.27.0, and cargo-nextest 0.9.149 as current. The pins match these versions. `mise install` exited 1 because Java metadata for `temurin-27.0.0+35` was not available for `macos-x64`. The tracked pins did not change. `mise doctor` found one PATH-order warning. `mbx doctor` reported zero failures and warnings. The Rust build guard rejected Cargo version queries before Cargo ran.

## Limits

T00 ran no Cargo command and no legacy test code. It read the Nextest version only. T00 did not prove product behavior. The complete hash ledgers include immutable baseline, snapshot, audit, and visibility assets. Static scanner candidates do not authorize deletion.
