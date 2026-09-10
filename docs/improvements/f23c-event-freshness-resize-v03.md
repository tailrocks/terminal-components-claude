# F23c — event freshness/resize (V03)

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Retain real Holla App/owned-PTY batched-versus-separated page/modal/resize/activation/paste/mouse cases. Assert focus, hits, drafts and cursor after each transition.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

Broader TablePro/Jackin/showcase flow matrices remain deferred.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

Family policy: [F23 shared context](shared-contracts.md#f23-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** Changed-event minimal pump passes; historical real burst replay not retained.

**Root fix / retained acceptance:** Real-App/PTY batched-versus-separated page/modal/resize/activation/paste/mouse sequences; below-minimum→normal→wide→minimum; focus, hits, drafts and cursor agree. No timing-based correctness assertions.

## Evidence

**Slice status:** current Holla/shared slice complete · broader TablePro/Jackin matrices stay Later.

**Proof:** `app_tests_proofs.rs: batched_and_separated_event_sequences_agree_on_focus_hits_drafts_and_cursor` runs page, modal, resize, activation, paste and mouse on the real App both as one batch and as separately rendered events and asserts equal focus, hit regions, drafts and cursor; `resize_below_minimum_then_normal_then_wide_then_minimum_keeps_every_state_consistent` walks every inventory state through 40x10, 100x30, 200x60, 72x20, 40x10 and 120x40 and asserts the too-small notice, recovery, focus owner, markers and that a key after each resize lands on the current owner. No timing-based assertion is used.

**Captures inspected:** `shots/h_disk_cleanup_16` and `shots/h_upgrade_plan_16` (16-colour), the 80x24 base frames.

**Deferred remainder:** TablePro/Jackin/showcase flow matrices (Later checkbox).

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
