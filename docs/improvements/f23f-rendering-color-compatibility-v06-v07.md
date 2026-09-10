# F23f — rendering/color compatibility (V06/V07)

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Apply F22 checks and fresh-process palette/NO_COLOR/force-color/TERM assertions to Holla and changed shared primitives. Record actual terminal/font/tmux versions and scope.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

Unrelated application visual matrices and unavailable external-platform evidence remain deferred or explicitly pending.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

Family policy: [F23 shared context](shared-contracts.md#f23-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** Four palette buffers and one fresh-process NO_COLOR backend path; real font/terminal breadth unproven.

**Root fix / retained acceptance:** F22 fidelity checks plus subprocess matrix for explicit palette, NO_COLOR empty/nonempty, force-color and relevant TERM policy. Assert color SGR separately from reverse/DIM/bold/underline. Record real emulator/font/tmux versions.

## Evidence

**Slice status:** current Holla/shared slice complete · unrelated application matrices stay Later.

**Fresh-process matrix:** `tests/holla_pty.rs: explicit_palettes_emit_exactly_their_colour_class_and_keep_attributes` (truecolor frames carry 24-bit SGR and no palette colour; 256 and 16 frames carry only their class; attributes survive), `no_color_policy_is_the_backend_rule_in_a_fresh_process` (a non-empty `NO_COLOR` suppresses colour even against an explicit `--color truecolor`, reverse and bold survive; an empty `NO_COLOR` does not suppress). `runtime.rs: no_color_backend_preserves_selection_attributes`; `theme.rs: accent_survives_downgrade`, `hover_and_focus_are_distinct_styles`; `tests/focus_gutter.rs` (no meaning carried by colour alone).

**Recorded environment:** tmux 3.7c, xterm-256color with `COLORTERM=truecolor` inside tmux, Python 3.14.7, Pillow 12.3.0, JetBrainsMono NFM (digests in every manifest), Darwin 25.6.0.

**Captures inspected:** `shots/h_parity_executor_mono`, `shots/h_<scenario>_mono` spot checks; the Mono palette is four greys and is separate from the `NO_COLOR` proof.

**Deferred remainder:** external emulator/font breadth (Later checkbox).

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
