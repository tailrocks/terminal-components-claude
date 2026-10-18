# F23g — evidence provenance (V08)

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Retain small deterministic Unicode/PTY/burst harnesses and provenance manifests for changed Holla flows and shared components. Record source/binary digests, scenario, size, color, tools/fonts and inspected result.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

Rebuilding the entire historical multi-application evidence archive is deferred.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

Family policy: [F23 shared context](shared-contracts.md#f23-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** Historical probes partly in /tmp; 301 complete sets prove presence, not generating source.

**Root fix / retained acceptance:** Retain small deterministic Unicode/PTY/burst harnesses; manifest source and binary digests, scenario/state/size/color environment, capture tools/fonts and review result. Hashes/manifests never substitute for semantic assertions or visual inspection.

## Evidence

**Slice status:** current Holla/shared slice complete · rebuilding the historical archive stays Later.

**Harnesses retained:** `tools/fidelity_check.py` (Unicode cell fixtures), `tests/holla_pty.rs` (PTY palette, `NO_COLOR`, flood), `tests/terminal_suspend.rs` (job control), `app_tests_proofs.rs` (burst, idle, inventory). **Manifests:** `tools/capture.sh` writes `<name>.manifest.json` per frame with source revision and dirty flag, binary sha256, per-session arguments (keyed by capture session so concurrent runs never mix), geometry, colour environment, tool and font versions with digests, the PNG fidelity summary and a `review` field that `tools/capture.sh review <name> "<verdict>"` fills after inspection. Scripts: `tools/holla_shots.sh`, `tools/holla_flows.sh`, `tools/holla_parity_flows.sh`.

**Rule kept:** hashes and manifests never substitute for the semantic tests or the visual inspection recorded in the HP and H00 evidence.

**Deferred remainder:** the historical multi-application archive (Later checkbox).

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
