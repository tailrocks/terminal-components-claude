# F08d — Unassigned modifiers perform plain actions

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Specify and test Holla chords, editor/modal precedence, advertised action reachability and shared Picker/navigation bindings used by Holla. Unassigned modified chords must not perform unrelated plain actions.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

The full TablePro/DataTable and other-application chord audit remains deferred. No universal command bus or broad binding framework.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

Family policy: [F08 shared context](shared-contracts.md#f08-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** P2 architecture/API weakness. DataTable Ctrl+S sorts at `table.rs:438`; several navigation controls match code without modifier ownership. TablePro intercepts Save, so this is not a proven broken TablePro Save flow.

**Root fix / retained acceptance:** Define a chord/precedence matrix before changes; explicit bindings act, unassigned nonmodal chords return Ignored, top modal may consume without unrelated action. Medium compatibility risk. Preserve Shift ranges and Picker Ctrl+J/K/N/P; test Save/Run/Find/Close/detach through real owners. `Key::plain` intentionally permits Shift; adding it everywhere is not the complete policy.

## Evidence

**Slice status:** current Holla/shared slice complete · the TablePro/DataTable chord audit stays Later.

**Chord matrix (Holla):** shell chords `Ctrl+Q`, `Ctrl+C`, `Ctrl+G`, `Ctrl+W`, `Alt+0` to `Alt+9`, `F1`, `F10` are handled before the page; an editing owner that returns Ignored lets a shell chord through; unassigned modified chords are Ignored by every owner and perform no plain action (the F23d flood uses an unbound `Ctrl+B`); Enter with Shift chooses like Enter, Alt+Enter is the alternate action, Ctrl+Enter is not a chord. Query editing owns `Ctrl+A/Z/Y/U` and `Ctrl+Backspace`/`Alt+Backspace`; `Ctrl+W` stays the shell's close-tab. Activity input mode forwards every plain key to stdin and still yields Alt chords to the shell. Tests: `app_tests_proofs.rs: enter_with_modifiers_maps_to_exact_picker_and_finder_semantics`, `a_finite_flood_of_ignored_input_never_starves_the_tick_check`; `app_tests_parity.rs: hp02_…` (query chords, Alt+0 while editing), `hp15_…` (input mode versus Alt chords); `app_tests_flows.rs: esc_ladder_and_quit_rules`, `menu_bar_help_and_about`.

**Hints:** every advertised chord is reachable (`F1` reference, footer hints per page); the F23a inventory checks the hint row per state.

**Captures inspected:** `shots/h_flow_help` (key reference), `shots/h_hp15_input_mode` (input-mode hints).

**Deferred remainder:** TablePro/DataTable chord audit (Later checkbox); no universal command bus.

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
