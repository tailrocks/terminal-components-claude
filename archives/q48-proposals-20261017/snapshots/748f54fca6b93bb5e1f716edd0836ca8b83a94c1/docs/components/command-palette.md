# CommandPalette

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-overlays.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W19 · Group: Overlays · Phase: P4.

## Purpose and exclusions

Canonical Termrock contract for W19. `CommandPalette` is a thin command-oriented configuration of [`Picker`](./picker.md), not a command executor.

The caller supplies action metadata from the same binding catalog used by menus/help, including stable `ActionKey`, effective shortcut, label, detail and eligibility. `CommandPalette` filters and presents these actions, then returns a typed `ActionKey`. It never spawns, executes or interprets a command and never owns application state.

## Public API

```rust
CommandPalette::new(
    id: Id,
    commands: &'a [CommandItem<'a>],
    revision: Revision,
) -> CommandPalette<'a>;

CommandPalette::query(&'a str)
    .readiness(Readiness<'a>)
    .patch(StylePatch);

CommandPalette::update(&self, cx: &mut Cx<'_>, state: &mut PickerState)
    -> Response<CommandPaletteAction>;
CommandPalette::draw(&self, ui: &mut Ui<'_>, area: Rect, state: &PickerState) -> Rect;
CommandPalette::measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size;
```

Reuse `PickerState` directly; do not create `CommandPaletteState`. `CommandPaletteAction` is `Execute { action: ActionKey, origin }` or `Dismissed`. The name `Execute` means “return the selected action key to the caller”; actual execution belongs to the application. Expose only read-only current key/query/scroll observations.

## Ordinary use

Consumer recipe: [EX-09 — Menus, context menus, and a command palette](../api/consumer-recipes.md) (`proposed_target`).

Showcase picker scenarios and the Jackin Preview capsule command palette are the baseline consumers.

The caller supplies borrowed commands, a source revision, and readiness, then dispatches the returned `ActionKey` itself.

## Ownership

- Binding catalog, effective shortcut, eligibility and product dispatch are caller-owned borrowed props.
- The visible shortcut must come from the effective binding after remapping. A disabled command remains nonactivating even when it exactly matches the query.
- Stable `ActionKey`s survive catalog revision/reorder. A removed action cannot be rebound to a new row at the old index.
- Query editing, filtering, selection, focus, scroll, modal layer and Escape behavior are the Picker contract. This wrapper contributes metadata and typed `ActionKey` mapping only.
- `update` returns typed actions and changes shared PickerState; `draw` is semantically immutable; `measure` is pure. Runtime owns focus/capture/layer/placement/time.

Shared dependencies: [`picker`](./picker.md), [`input-actions`](../foundations/input-actions.md), [`identity`](../foundations/identity.md), [`runtime`](../foundations/runtime.md), [`layers`](../foundations/layers.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`author`](../foundations/author.md), and [`conformance`](../foundations/conformance.md).

## Customization

Styled parts are `container`, `title`, `query`, `row`, `label`, `shortcut`, `detail`, `status` and `empty`. Resolve styles through [`theme`](../foundations/theme.md) and Picker precedence. Preserve the baseline command modal/popup surface, query field, row marker/gutter, shortcut alignment, detail hierarchy, disabled treatment, empty/status states, clipping and backdrop. Remapped shortcuts must be visibly correct; this is a semantic/visual contract.

## Behavior

- Open and close use Picker's runtime-owned layer and focus restoration. Query typing, grapheme deletion, paste, navigation, scrolling, outside click and Escape follow [`picker.md`](./picker.md).
- Enter on an eligible row returns one `ActionKey`; it does not dispatch it. If alternate activation is part of the shared Picker policy, preserve its origin in the typed action.
- Keyboard shortcuts shown in rows are metadata, not an alternate direct dispatch path. The palette must not intercept unrelated application bindings while it owns the modal focus.
- Disabled commands cannot be activated by keyboard, pointer or a stale captured row. Hover and keyboard-suppressed hover follow the shared runtime policy.
- Where command activation has baseline feedback, sample 140 ms and expiry explicitly; redraw frequency cannot advance motion.

## Visual matrix

| Axis | Required cases |
|---|---|
| Geometry | normal, zero/tiny area, non-zero origin, exact fit, one-cell short, long Unicode labels/details, narrow popup |
| Focus/hover | unfocused, focused query/row, hovered, focus+hover, keyboard-suppressed hover, pointer restoration |
| Activation | pointer down/held/release inside/outside, removed/disabled action, keyboard activation and applicable feedback |
| Source/readiness | empty, ready, duplicate labels with distinct keys, disabled action, catalog reorder/insert/remove/filter, stale target, loading/partial/error/retry when supplied |
| Scroll/layer | no overflow/start/middle/end, wheel boundary, thumb drag, resize/reanchor, edge fade/protected row; closed/open/nested/Escape/outside/owner removed/focus restore |

Only applicable readiness and layer states are required. Shared dimensions/capabilities are governed by [`../verification/visual-parity.md`](../verification/visual-parity.md).

## Verification

The immutable oracle is commit [`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/commit/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b). The frozen visual evidence (legacy family C28, consolidated with Picker) is the picker/command-palette usage in [`src/widgets/picker.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/picker.rs), Showcase picker scenarios and Jackin Preview capsule command palette in [`src/bin/jackin_preview/screens/capsule.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/capsule.rs). Current `PickerEvent`, `WidgetId`, `Outcome` and `RenderCtx` identifiers are legacy implementation names.

Use [`../reference/capture-plans/command-palette.json`](../reference/capture-plans/command-palette.json). Compare exact symbols/continuation cells/styles/modifiers supported by the tool, cursor, focus/capture/layer owner, query draft/committed state, selected `ActionKey` and typed action count/target. Bind expected output first with [`../verification/oracle-and-provenance.md`](../verification/oracle-and-provenance.md); candidate code cannot bless its own output.

| Case | Required observation |
|---|---|
| W19-01 | A remapped command shortcut is shown and returned consistently. |
| W19-02 | A disabled command remains nonactivating despite an exact query match. |
| W19-03 | Empty results and duplicate labels with distinct `ActionKey`s remain distinguishable. |
| W19-04 | Escape restores owner focus and leaves product state untouched. |

Negative tests must prove: no command execution or process spawning; one shared PickerState/engine is used; displayed shortcuts reflect remapping; disabled actions cannot activate through any path; duplicate labels do not collapse keys; catalog revisions cannot retarget a held action; draw cannot dispatch or mutate product state; Escape restores focus; and patches affect only declared palette parts.

## Rejected use

Forbidden: painting command rows with shortcuts and dispatching in preview instead of using CommandPalette.

```rust
// Forbidden: preview paints its own command rows and dispatches.
ui.paint_str(row, 0, "Save  Ctrl+S");
if clicked { save_now(); }
```

Rule: [ARC-012](../architecture/component-composition.md) prohibits direct `ui.paint_str` rendering of controls and padded strings that simulate menus. Use `CommandPalette::new` with stable `ActionKey`s and dispatch the returned key in the caller.

## Known gaps

- Capture plan W19 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-004 historical Capsule screens](../implementation/code-remediation-backlog.md): pending; the Jackin Preview capsule command-palette consumer still uses its historical path.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
