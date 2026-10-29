# Harness source audit paths

The determinism and overlay source audits now resolve the workspace through
`support::manifest_path("")`. The prior `CARGO_MANIFEST_DIR/src` and
`src/bin` roots are stale after the application split and scanned no source.
Both audits discover every crate directory under `crates/` that has a
`Cargo.toml` and `src/`, including app facades, domain/runtime dependencies,
reusable `termrock` crates, and Jackin presentation, host UI, and capsule UI
painters. This covers `crates/termrock/src` and every package dependency source
without maintaining a fragile app-crate allowlist.

The explicit exclusions are test or repository-tool packages: `visual-harness`,
`termrock-test-support`, `termrock-conformance`, `termrock-e2e`, and
`termrock-visibility-tests` contain test machinery; `termrock-xtask` and
`termrock-visibility-publisher` are developer/operator tools outside app
rendering. All other crate source roots are discovered automatically. A
nonempty root and source-file count guard fail closed if the scan path becomes
stale.

The current deterministic-source inventory contains one reviewed call:
`crates/termrock-session/src/session.rs::Instant::now` at the session loop
scheduling origin. It controls when another frame is drawn, not frame content.
The old monolith allowances (`src/runtime.rs`, `src/core/typeahead.rs`, and
`src/clock.rs`) have no matching current production source. The old
`target_os` allowances also have no matching current production source. They
were retired after scanning the current production crate tree; all current
`Instant::now` and `target_os` occurrences remain exact-count gated.

Overlay scanning uses the same discovered production roots. Exact identifier
boundaries prevent names such as `ProductDialog::new` from matching
`Dialog::new`. `HISTORICAL_SITES` retains
the old monolith owner-to-scenario inventory and continues checking every
historical scenario row. `SITES` inventories current constructors and maps
each one to current scenario rows or an explicit open gap.

Six current site mappings remain open because no matching scenario row exists:

- `crates/holla-ui/src/screens/actions.rs::picker`: `holla/parity/custom-actions`
  has no captured open-and-dismiss sequence for the actions picker.
- `crates/jackin-preview-app/src/app.rs::draw_layers` and
  `::open_capsule_help`: no capsule-help scenario row; the existing manager
  help scenario exercises different content.
- `crates/jackin-preview-app/src/app.rs::update_overlays`: same uncaptured
  capsule-help owner path.
- `crates/jackin-preview-app/src/app.rs::editor_exit_dialog`: no editor-exit
  confirmation scenario; `jackin/editor/quit_guard` exercises application
  quit guard.
- `crates/jackin-preview-app/src/app.rs::launch_dialog`: no current scenario
  row opens the launch confirmation.

The scanner regression checks create temporary package manifests and inject a
forbidden clock token or new overlay constructor, then exercise the actual
crate-root discovery. The overlay check also asserts that an unlisted site
fails coverage and a prefix lookalike does not enter the inventory.

## Verification

- `cargo fmt --check`: passed after formatting the 16 pre-existing drifted
  Rust files under `crates/visual-harness/tests/visual_baseline`.
- `cargo nextest run -p visual-harness --test visual_baseline -E
  'test(/determinism::/)|test(/overlay_sites/)' --no-fail-fast --no-capture`:
  four focused audit tests passed.
- `cargo clippy -p visual-harness --all-targets`: passed before the formatting-
  only cleanup.
- Overlay audit still reports 6 OPEN scenario mappings. These are recorded
  gaps, not evidence of complete baseline parity.
