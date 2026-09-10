# F23a — state reachability (design D4)

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Build the executable Holla page/state/input-route inventory. Verify expected owner, visible target, semantic marker, keyboard/mouse traversal, supported sizes, palettes and actual NO_COLOR for current surfaces.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

Full showcase state traversal and unrelated application inventories remain deferred.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

Family policy: [F23 shared context](shared-contracts.md#f23-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** Static hashes do not traverse enabled controls or distinguish references from live states.

**Root fix / retained acceptance:** Executable page/state/input-route inventory with expected focus owner, visible target and semantic marker; keyboard/mouse, min/normal/wide, all palettes and actual NO_COLOR. Disabled controls excluded from traversal; read-only controls included.

## Evidence

**Slice status:** current Holla/shared slice complete · full showcase traversal stays Later.

**Executable inventory:** `app_tests_proofs.rs: inventory` lists 19 Holla states (finder, finder preview, help modal, activities picker, alternatives, the first and second gates, arguments, trust, activity, plan review, files browse and find, disk overview, tree, top files, facts drawer, cleanup insight, cleanup gate) with their scenario, route, expected focus owner, visible marker and modal flag; `inventory_reaches_every_state_with_its_focus_owner_at_every_size_and_palette` drives each route at 72x20, 100x30 and 160x50 (the capture matrix adds 80x24 and 120x40) in TrueColor, ANSI-256, ANSI-16 and Mono, asserts the marker, the focus owner, that Tab and Shift+Tab traverse every reachable stop and return, and that disabled controls are not in the ring. Actual `NO_COLOR` is proven separately in `tests/holla_pty.rs: no_color_policy_is_the_backend_rule_in_a_fresh_process`.

**Captures inspected:** the base matrix `shots/h_<scenario>_{80x24,100x30,120x40,160x50,mono}` for the concept and parity scenarios (spot-checked at 80x24 and mono: `h_parity_discovery_80x24`, `h_parity_executor_mono`) and the flow frames cited by the HP tasks.

**Deferred remainder:** showcase-wide state traversal (Later checkbox).

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
