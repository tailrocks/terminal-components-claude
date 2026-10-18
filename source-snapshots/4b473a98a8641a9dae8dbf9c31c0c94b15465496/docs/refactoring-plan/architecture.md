# Architecture and API audit

This audit compares immutable UI oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c` with main `7b27732a8c3c131760ec3438f641cb3c11343a42`. It does not authorize implementation or baseline changes. Source references use `commit:path:line`; the short commits below identify those pinned objects. Component-specific findings and application flows are maintained in the companion inventories. `architecture-matrix.tsv` covers the cross-cutting contracts that apply to those inventories.

## Architectural state

Main has completed the physical workspace cut: virtual root, `junie-tui`, `junie-tui-testing`, `xtask`, and four application packages. The oracle has one package with public `core`, `ui`, `widgets`, `theme`, and `runtime` modules plus four binaries. Main no longer carries that old root library. Recreating it or placing old source under `apps/` would violate the accepted architecture. Evidence: `02f5294:Cargo.toml:1`, `02f5294:src/lib.rs:1`, `7b27732:Cargo.toml:1`, `7b27732:crates/tui/src/lib.rs:20`.

The accepted component model is retained caller-owned state, borrowed props, explicit `update(&mut self/state, &mut Cx)` and read-only `draw(&self/state, &mut Ui)`. Runtime owns interaction dispatch, geometry publication, focus, layers, pointer capture, feedback timing, and cursor ownership. Components return typed `Response<A>`; identity is `Id`, stable `ItemKey`, and `PartRef`. Applications retain domain data and decisions. Evidence: `7b27732:COMPONENT_ARCHITECTURE.md:118`, `7b27732:COMPONENT_ARCHITECTURE.md:332`, `7b27732:COMPONENT_ARCHITECTURE.md:379`, `7b27732:crates/tui/src/runtime.rs:44`.

Most structural migrations exist, but physical relocation and current green source checks do not establish completion. Main still contains explicitly labelled compatibility painting and an application page that draws a live generic Grid before overwriting its pixels with a fixed historical-looking fixture. The interaction owner and visible data are different. This is a concrete example of the prohibited end state: generic component reuse in syntax, duplicated rendering in behavior. Evidence: `7b27732:apps/showcase/src/pages/grid.rs:278`, `7b27732:apps/showcase/src/app.rs:667`, `7b27732:apps/showcase/src/app.rs:1258`.

## Binding target and superseded language

1. Keep the curated application facade at the crate root and the separate component-author facade. Internal `components`, `text`, `ui`, `runtime`, and interaction modules remain private. `author::raw` is the qualified ratatui escape hatch; it does not authorize applications to reconstruct reusable widgets. Evidence: `7b27732:crates/tui/src/lib.rs:20`, `7b27732:crates/tui/src/author.rs:1`.
2. Keep one generic implementation per component family. `DataTable` is absorbed by `Grid`; `ScrollPanel` by `TextViewport`; `statusbar` and `segments` by `StatusBar`; field editing by `TextEditorCore`/`FieldControl`; scrollbar mechanics by `ScrollRegion`; popup placement and backdrop composition by runtime layers. Evidence: `7b27732:COMPONENT_ARCHITECTURE.md:3891`.
3. Follow implemented accepted names, not early illustrative names. The public main facade exposes `Brand`, `ChipBar`, and `ScrollRegion`; do not resurrect `Lockup`, standalone old `Chip`, or the struck `Ui::scroll_region`. `GridCellActions`, `Grid::editable(bool)`, old `WidgetId`, `Outcome`, and `RenderCtx` remain rejected. Evidence: `7b27732:crates/tui/src/lib.rs:121`, `7b27732:COMPONENT_ARCHITECTURE.md:1314`, `7b27732:COMPONENT_ARCHITECTURE.md:3886`.
4. Keep the stronger backend-free implementation. `ratatui-crossterm` is optional behind `crossterm`; core keyboard vocabulary is library-owned, and normalization is feature-gated. Earlier prose claiming a normal non-optional dependency and exactly two references is stale description of an earlier boundary, not permission to reintroduce that dependency. Evidence: implementation commit `4e84b744`, `7b27732:crates/tui/Cargo.toml:17`, `7b27732:crates/tui/src/event.rs:1`, `7b27732:xtask/fixtures/backend-free-core/Cargo.toml:1`, `7b27732:xtask/fixtures/backend-free-testing/Cargo.toml:1`.
5. Keep semantic paint provenance. `PaintStyle`, authored palettes, `StyleDefaults`, and symbolic Meter rest policy preserve semantic roles through capability conversion and layer dimming. Do not infer general paint roles from RGB equality. Meter's narrowly specified ordered color comparison is an accepted explicit policy and is not the generic `Theme::raise` algorithm. Evidence: `7b27732:COMPONENT_ARCHITECTURE.md:8758`, `7b27732:crates/tui/src/theme/mod.rs:188`.
6. Keep runtime time and presentation contracts: explicit `Moment`, monotonic `advance_to`, driver-owned feedback/simulation clock, `PaintedFrame` presentation acknowledgment, pending-input barriers, and immutable render snapshots. `Input::Tick` is an update trigger and does not advance elapsed time. Evidence: commits `8f7e1c92`, `b1c8c447`; `7b27732:crates/tui/src/event.rs:24`, `7b27732:crates/tui/src/runtime/time.rs:1`, `7b27732:crates/tui/src/runtime.rs:1816`.
7. Preserve current public override contracts: theme, subtree, instance, part patches and slots; row/column/custom-cell extension points must honor them while preserving geometry and interaction. Merely exporting a method is insufficient. Evidence: `7b27732:crates/tui/src/author.rs:16`, `7b27732:COMPONENT_ARCHITECTURE.md:964`, `7b27732:COMPONENT_ARCHITECTURE.md:1179`.
8. Current UI parity authority supersedes all earlier permission to change user-visible behavior. Historical visual ledger entries explain main's divergence but cannot exempt final output from the pinned oracle. Preserve the architecture mechanism and reproduce oracle presentation through it.

The remaining fixed decisions are adjudicated in [architecture-adjudication.md](architecture-adjudication.md), ADJ-01 through ADJ-08. In particular, §73's “dynamic disabling unspecified” prose is stale: commit `715ee0777e20a09e0f373b07024076bdc742ef1d` implemented cancellation of ineligible held targets at successful publication. Main `runtime.rs:1725` calls the shared actual-part eligibility authority in `capture.rs:30`; the focused `pointer_capture_eligibility` target was independently rerun on pinned main and passed all twelve tests. Preserve that implementation and extend true overlapping-owner proof; do not infer a new runtime defect merely because `pointer_captured` does not repeat the publication check.

## Remaining obligations demonstrated by current source

### Application duplication and cosmetic reuse

Showcase Grid registers and updates `MetricModel`, then paints fixed customer rows. Its text includes `rows 1–0 of 40 loaded`, while actions produce a selected metric. Replace that split ownership with a real domain model whose live `Grid` paints the oracle data and behavior. Do not delete live Grid calls while retaining the fixed picture. Main source: `apps/showcase/src/pages/grid.rs:263`, `:278`, `:288`.

Showcase header/footer draws generic `Brand`/`StatusBar`, then paints compatibility chrome over the same rectangles. Reusable portions must become real generic component configuration or supported reusable extension points; application-owned content remains application-owned. `HistoricalPalette` in Jackin is not by itself proof of duplication: it uses custom semantic families and authored style pairs. Audit each consumer; retain legitimate product art (especially the explicitly accepted rain renderer) and remove generic widget replications. Main source: `apps/showcase/src/app.rs:1258`, `apps/jackin-preview/src/app/historical_paint.rs:1`, `apps/jackin-preview/src/rain.rs:48`.

The current anti-copy gate only recognizes four spelling patterns: `fn render`, `Style::new()`, `Block::default()`, and `buf/buffer.set_string`. It does not inspect `Ui::paint_str` composition or detect a component's output overwritten by another renderer. The root cause is an insufficient executable definition of reusable-component ownership. Strengthen the gate with explicit component/part ownership and production reachability checks, validated using a renamed helper, a dead component call, an inert reference component followed by custom painting, and a live component followed by paint-over. Keep the legitimate external-author example and app-owned rain green. Do not substitute a blanket prohibition on all `Ui` painting: the author API intentionally allows new components and application art. Evidence: `7b27732:xtask/src/main.rs:4464`, `7b27732:xtask/src/main.rs:4497`.

### Row and column override propagation

The consolidation ledger rejected an unimplemented test requiring instance part patches to reach `RowUi`, `ColumnsUi`, and `part()` painters. That was a rejected patch, not a rejected override requirement. Current `RowUi` has only a label patch; `part()` calls plain `ui.style`, and `ColumnsUi` retains only the label patch. Complete the existing borrowed override contract across all row branches and apply it through List, Tree, Picker, NavList, Tabs, Steps, and other row consumers. Independent tests must distinguish theme, subtree, instance, part, and custom renderer contributions and exercise clipping/mono/surface inheritance. Evidence: `7b27732:docs/audit/consolidation/disposition-ledger.md:468`, `7b27732:crates/tui/src/collection/rowui.rs:29`, `7b27732:crates/tui/src/collection/rowui.rs:314`, `7b27732:crates/tui/src/collection/rowui.rs:666`.

### Unclosed named obligations and documentation

`xtask/named_tests_allow.txt` contains two unresolved names. `mono_pressed_choice_keeps_the_label_geometry` is a real unclosed Choice contract. `capsule_pane_clone_4x2000` is intentionally absent because viewport cloning was deleted; its absence is tested in Jackin. Do not add a clone benchmark to satisfy its name. Close that historical obligation through the no-clone replacement proof, reconcile its documentation, and remove only a reviewed obsolete deferral. The `doc-check` allowlist contains cross-crate references as well as historical references; a source scan's inability to resolve a valid cross-crate API is different from unfinished production code. Require exact current-target resolution or explicit, evidence-backed archival disposition. Evidence: `7b27732:xtask/named_tests_allow.txt:19`, `7b27732:apps/jackin-preview/tests/perf.rs:126`, `7b27732:xtask/doc_check_allow.txt:1`.

The packaging exception for the split Showcase Buttons example/test still says it expires at Slice 5, and the render matrix prose still promises the two targets merge at Slice 5. Both split structures survive. The fixed execution decision is to close both expired promises: put the Buttons demonstration matrix and its production-app assertions under Showcase, and consolidate the two library render targets into one while preserving every qualified scenario identity and baseline key through an exact relocation map. Keep the external Buttons example only as a separate public-consumer example; it must not substitute for the application matrix. Update every CI/architecture invocation in the same consolidation task, after an exact before/after test inventory proves no case was lost. Independent review must validate that equivalence; preserving the stale packaging indefinitely is not the chosen plan. Evidence: `7b27732:COMPONENT_ARCHITECTURE.md:2054`, `7b27732:COMPONENT_ARCHITECTURE.md:3934`, `7b27732:crates/tui/examples/showcase_buttons.rs:1`, `7b27732:crates/tui/tests/showcase_buttons.rs:1`.

### Performance and test preservation

The accepted post-Slice-5 style-resolution share benchmark remains missing. `style_resolve_share_of_frame_showcase_lists_120x40` is required at at most 5% under strict measurement; main still runs the foundation stand-in and says the real frame does not exist, although the application benchmark exists. Add the real measured attribution check, keep the zero-allocation and cache-correctness checks, and demote only the superseded stand-in extrapolation as specified. Evidence: `7b27732:COMPONENT_ARCHITECTURE.md:2246`, `7b27732:crates/tui/tests/perf.rs:229`, `7b27732:apps/showcase/tests/perf.rs:27`.

Some application tests are performance smoke tests only. For example `render_twice_allocates_the_same` compares digests without counting allocations. Do not use its name to claim allocation proof. Run and preserve the real `perf::bench`/`report_to` counts and add missing measurements against original rows. The deterministic allocation/byte gates block; wall-clock thresholds remain the separately labelled advisory strict job on shared CI, with controlled local strict evidence for final performance assessment. Evidence: `7b27732:apps/showcase/tests/perf.rs:129`, `7b27732:apps/showcase/tests/perf.rs:142`, `7b27732:.github/workflows/perf.yml:1`.

Exact historical test accounting exists but is unfinished: `tools/test-inventory/required.json` remains pending, with historical obligations unresolved and no approved target matrix. Its 3,211 source-qualified obligations predate the newly pinned oracle and must be reconciled with that oracle's tests rather than mistaken for complete present coverage. Map exact test identities and reviewed relocations, execute all required profiles, and wire verification into the final gate. Counts alone, source names alone, listing-only runs, ignored or filtered tests cannot close it. Evidence: `7b27732:tools/test-inventory/README.md:69`, `7b27732:tools/test-inventory/required.json:1`.

## Gate contract

These commands are mandatory future implementation gates at the final candidate, with no baseline blessing variables. `BLESS_GUARD_BASE` must resolve the actual integration base, not the candidate itself. Use a dedicated target directory and capture compiler versions, commands, exit codes, and source/lock fingerprints.

```sh
rtk cargo +1.88.0 check --locked --workspace --all-targets --all-features
rtk cargo +stable check --locked --workspace --all-targets --all-features
rtk cargo fmt --all --check
rtk cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
rtk cargo build --locked --workspace --all-targets --all-features
rtk cargo build --locked -p junie-tui --examples
rtk cargo check --locked -p junie-tui --no-default-features
rtk cargo test --locked --workspace --all-targets --all-features
rtk cargo test --locked -p junie-tui --test render --test render_components
rtk cargo test --locked --workspace --doc --all-features
rtk cargo doc --locked --workspace --all-features --no-deps
rtk cargo run --locked -p xtask -- app-inventory --json
rtk cargo run --locked -p xtask -- boundary
rtk cargo run --locked -p xtask -- bless-guard
rtk cargo run --locked -p xtask -- doc-check
rtk cargo test --locked -p junie-tui --test perf --test perf_collections --release -- --test-threads=1 --nocapture
rtk cargo run --locked -p xtask -- app-perf
```

Use `RUSTFLAGS=-D warnings` and `RUSTDOCFLAGS=-D warnings` for the CI-equivalent source/doc sweep. Stable owns trybuild stderr fixtures; the exact MSRV test command may skip only `architecture::compile_fail_cases_hold`, as current CI does, while compiling every target and running every behavioral test. Both isolated backend-free consumer fixtures are necessary: a workspace feature union can otherwise conceal an unwanted backend dependency. Architecture's public-item source pin does not replace rustdoc compilation or the stronger rustdoc-JSON foreign-type gate. Evidence: `7b27732:.github/workflows/ci.yml:66`, `7b27732:crates/tui/tests/architecture.rs:339`.

Preserve allocation/byte thresholds, wide/grapheme safety, 100k List/Tree/Picker/Viewport/Steps scaling, O(visible) render/dispatch probes, and bounded cache work. Strict wall-time assertions use the accepted 1.2x baseline and named special ratios; do not silently change denominators or benchmark scenes. The new production-view/canonical-cell and PTY oracle gates supplement these commands, because main's existing digests and `parity_contract` do not establish parity with the new immutable oracle.

Semver checks are historically deferred during this intentional public-API rewrite. Finalize the API inventory and release readiness, then establish the reviewed baseline for future `v0.1.1` compatibility; do not compare the new architecture against the old Holla API or publish/tag during this planning goal. Evidence: `7b27732:COMPONENT_ARCHITECTURE.md:2209`.

## Current measurements

On 2026-09-11, an isolated detached worktree at the pinned main commit was created at `/tmp/tc-architecture.L0vAdH/main`. `CARGO_TARGET_DIR=/tmp/tc-architecture.L0vAdH/target-msrv RUSTFLAGS=-D warnings rtk cargo +1.88.0 check --locked --workspace --all-targets --all-features` exited **0**, checking all four apps and library/testing/xtask packages. This replaces the stale historical claim that current main necessarily fails its MSRV build. It proves compile correctness only, not runtime or parity correctness.

The separate `CARGO_TARGET_DIR=/tmp/tc-architecture.L0vAdH/target-doc rtk cargo run --locked -p xtask -- doc-check` run exited **0**: 76 Rust blocks, 880 resolved references, and 35 explicitly allowlisted references. This proves the scoped resolver works; the allowlisted references remain subject to the closure audit above.

`CARGO_TARGET_DIR=/tmp/tc-architecture.L0vAdH/target BLESS_GUARD_BASE=7b27732a8c3c131760ec3438f641cb3c11343a42 rtk cargo run --locked -p xtask -- boundary` exited **1**, with exactly two failing checks:

- `capture_matrix_contract`: checked-in records name artifact paths that do not equal the paths expected for this checkout; capture-state stderr files are absent. Example: `showcase_junie_truecolor_80x24: ansi artifact path is not shots/showcase_junie_truecolor_80x24/ansi`, followed by a missing `shots/.capture-state/.../stderr.log`. This is a clean-checkout evidence portability/completeness failure, not a measured UI mismatch.
- `parity_contract`: `cannot inspect parity replay evidence parity/evidence.tsv: No such file or directory (os error 2)`. No current replay equality was established by this gate.

All other registered boundary checks passed, including isolated backend-free core/testing consumers. The run reported 391 documented names, 389 present and two deferred; 43 conformance components in 44 registrations; 14 external examples; 121 application source files across four apps; 208 configured constructions; and 43 components reachable across 22 Showcase pages. The anti-copy gate passed despite the source-proven paint-over above, directly demonstrating that its present scope cannot prove architectural completion.

The baseline guard's zero-change result used the same pinned main as its comparison base solely to isolate source-gate status. It is **not** evidence that an integration diff preserves baselines. The final candidate must compare against its actual pinned integration base and separately verify trusted oracle artifacts. No source fix or baseline edit was performed; these measurements do not claim full tests, lint, performance, or parity pass.
