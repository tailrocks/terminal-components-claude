# Slice A — explicit bootstrap, no update from painting

Base89090d6. Candidate branch codex/runtime-initialize, isolated worktree terminal-components-initialize. Scope: runtime initialization/live session, testing Harness/Scene documentation, initialization integration suite, and69 explicit initialization setup migrations in existing behavioral tests/helpers.203 Runtime constructor leads classified in caller-migration.json; pure Stub/slot/reference projections deliberately remain uninitialized. No blanket constructor rewrite. No manifests, exports, theme, application production, baseline or xtask edits.

## Implemented guarantee

Runtime::initialize()->Response<()> performs one Bootstrap update, retains returned invalidation/deadlines, advances no time, and is idempotent. Live run and behavioral Harness call it before painting. draw_with_buffer no longer invokes app initialization; bare render and closure Scene cannot trigger App::update implicitly. New tests check first render plus1000 repeated draws leave app value/commits/update/effect counters untouched; explicitly initialized runtime receives one bootstrap/effect; closure production projection never updates supplied app; Harness initializes once before painting.

Runtime::handle retains lazy initialization as an explicit temporary UPDATE-path bridge. No new panic or silently ignored event path introduced. It still routes against last-drawn geometry: first input before geometry is NOT proven delivered. SliceB must add owned PendingInput/publication guard/explicit settling. No time semantics change; Input::Tick remains old cadence until SliceC. Existing runtime focus reconciliation can affect repeated frame cells. We DO NOT claim generic cell purity from effect counters. A proposed Scene.draw_app convenience was removed before commit because full explicit interaction/layer snapshot belongs to next pure-production capture slice.

## Verification

Stable and Rust1.88:742 library unit +935 conformance +7focus traversal +5initialization pass. tui-testing16 pass plus1 intentionally ignored subprocess helper (spawned by concurrent-bless driver). All three app behavioral suites pass: Showcase29,TablePro35,Jackin27+6chrome+10preview; app libraries Jackin63,TablePro17. Full all-target application runs were executed on both toolchains and failed six targets; independently repeated on pinned89090d6 base with same six failing cases. See app-comparison.json and assertion-comparison.txt; no baseline edited to pass.

Pre-existing failed cases: jackin_visual_baseline; showcase_visual_baseline; tablepro_visual_baseline; wheel_showcase_lists; frame_tablepro_grid_500x12_120x40; reference_hover_targets_only_the_visible_row. These are remaining product/performance obligations, not excused failures or completion evidence.

Strict all-target Clippy for both library/testing packages passes. Library doctests3 pass; testing perf doctest intentionally ignored. Changed-file rustfmt and git diff --check pass. Full workspace cargo fmt --check FAILS unchanged11 paths in Showcase,TablePro,xtask, independently reproduced at base; external base-fmt.log lists exact files. Root owns those corrections. Required identities were enumerated in stable-list.log before execution; final initialization log records renamed final case identities.

Mutation: temporarily reinstating draw-time initialize runs exactly1 selected first-render test and fails with value fixture initialized vs fixture(exit101); original source restored, final tests green. Logs and SHA256 manifest accompany this report. No full-goal, terminal publication, visual parity or independent reviewer approval claimed.

## Next prerequisite outside this slice

Explicit session capability choice needs shared run_with_options(app,theme,SessionOptions{color_policy}) with Detect preserving run default and a reviewed Explicit policy for caller CLI. Must adjudicate historical§74 color-ceiling versus pinned reference explicit-option precedence before caller wiring. No environment mutation or app-local terminal loop workaround. Separate from lifecycle/time ownership.

Library performance target:33 passed,0 failed; no allocation/byte thresholds changed.

Candidate: d7bcb16da916e4bfd6cb3d033b6a6c68b8bd99d5.
