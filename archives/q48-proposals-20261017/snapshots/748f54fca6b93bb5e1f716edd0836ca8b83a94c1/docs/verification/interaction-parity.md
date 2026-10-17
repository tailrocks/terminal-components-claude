# Interaction parity

Interaction parity proves the event-to-state-to-frame contract at sealed
checkpoints. It covers the same existing application behavior as visual parity
and keeps semantic differences visible when two frames happen to look alike.

## Checkpoint model

Each case has a stable scenario ID, fixture revision, initial dimensions and
capability mode, ordered input program, deterministic time samples, and named
checkpoints. At every checkpoint record:

- the exact input bytes and decoded logical event;
- update result and typed action sequence (count, type, target identity, and
  order);
- focus owner, hover owner, active layer path, and pointer-capture owner;
- selected/current stable key, navigation key, scroll offsets and boundaries;
- controlled value, draft, committed value, revision/conflict result, and
  external model mutation;
- cursor position/visibility and the exact frame compared by
  [`visual-parity.md`](visual-parity.md).

The runtime sequence is explicit: publish current geometry, dispatch one event,
update caller-owned state, resolve focus/hit/capture/layers, draw from
immutable draw state, then observe and compare. A test must include unchanged
frame checkpoints for ignored actions where the contract says no state change.

## Input and pointer traces

Keyboard traces use fixed decoded keys and preserve component-specific policy.
Pointer traces use zero-based coordinates resolved against the trusted oracle's
published geometry. Seal the numeric coordinates and event bytes before the
candidate runs. A candidate may not calculate its own hitboxes, choose a new
coordinate, and call the resulting gesture equivalent; semantic-target tests
are a separate lane.

For actionable controls, cover pointer down, held press, release inside,
release outside, target removal, target disabling, keyboard activation, and
feedback expiry. Release outside has no activation. A disabled control blocks
the underlying target and cannot click through. A removed or reordered item
cannot receive a stale activation.

Hover is observable only when the component owns a hit target. Keyboard input
suppresses stale hover; a later pointer-motion event restores it. Resize during
drag, capture-owner removal, nested overlays, and focus restoration after close
are separate checkpoints where applicable.

## Focus, layers and reconciliation

Focus, hover, press, pointer capture, geometry, layer order, and cursor
arbitration belong to shared runtime mechanisms. Components publish identity and
intent; they do not invent local hit-testing or independent focus systems.

Layer traces cover open/close, nested layers, outside dismissal, Escape
precedence, reanchoring after resize, removed owners, click-through prevention,
and restoration of the exact surviving focus owner. A single outside dismissal
gesture cannot also activate a newly exposed underlying control.

Collection traces cover source revision, filtering, insertion, removal and
reorder. Compare stable item/column keys rather than display indices. A
surviving current or selected key remains the same after reorder; a removed key
follows the documented nearest surviving fallback. A stale held press or stale
completion result cannot act on the new occupant of an old index.

## Editing and controlled values

Caller-owned controlled values are compared separately from temporary drafts.
An accepted commit updates the caller value; a rejected change leaves the
controlled value unchanged and the component cannot drift into an uncontrolled
value. External revision conflicts are reported and tested.

Text editing uses the shared grapheme/Unicode model, but component policy stays
intentional. `TextInput` and `TextArea` may have different Escape, paste,
selection, and commit behavior when the frozen baseline requires it. Do not
flatten those differences into a universal shortcut. Cover combining marks,
CJK, wide cells, multiline paste, deletion, navigation, read-only navigation,
selection direction, valid/invalid feedback, blur/tab commit, and cancel.

## PTY and pure lanes

Pure captures prove component state transitions, exact frames, semantic traces,
and deterministic time. The PTY laboratory separately proves terminal key
decoding, mouse down/up/drag, bracketed paste, resize, event ordering,
application routing, and terminal cleanup. Both use deterministic fixtures and
isolated data; components never spawn processes.

`TerminalView` consumes caller-provided prepared cells, continuation-cell
metadata, cursor/selection state, and emits typed copy/link/interaction
requests. The PTY lane may exercise a generic adapter, but Termrock remains a
presentation library. It does not become a terminal emulator, PTY runtime,
shell, daemon, session manager, or escape parser.

## Applicable interaction axes

The capture plans declare applicability per component. The required axes are:

- normal, zero/tiny allocation, nonzero origin, exact fit, one-cell-short and
  narrow-to-wide recovery;
- unfocused/focused/hovered/focus+hover and keyboard-suppressed/restored hover
  for interactive owners;
- down/held/release inside/release outside, removed or disabled target, and
  activation feedback for actionable parts;
- chosen versus cursor versus hover for controlled choices;
- navigation/editing/selection/valid-invalid/read-only/commit-cancel/conflict
  for editors;
- empty/loading/partial/ready/error/retry/stale and dynamic source change;
- top/middle/bottom scroll, wheel, thumb capture, protected-row edge fade,
  follow/tail, resize anchoring;
- nested overlays, outside dismissal, Escape ladder, reanchor, owner removal,
  and focus restoration;
- every unique motion phase, phase wrap, timing boundary, completion/failure,
  paused and reduced motion.

Decorative components explicitly record non-applicability. `Spinner` has phase
coverage but no invented focus/click case. `KeyHint` has rendering and
geometry coverage but no editing state. Composite components attribute action
traces to their real child controls.

## Interaction failures

The verifier fails on a wrong focus/capture owner, selected or navigation key,
draft/commit observation, action count or action target even when cells match.
It also fails on duplicate activation, click-through, stale-key activation,
incorrect release-outside behavior, cursor drift, wrong resize anchoring, or a
semantic result hidden by a normalized screenshot attribute. Negative mutation
requirements live in [`conformance.md`](conformance.md).
