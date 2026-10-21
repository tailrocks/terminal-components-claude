# Modal backdrop DIM: the frozen baseline conflicts with itself

- Date: 2026-10-21
- Workstream: theme/visual-state extraction (PR #38), branch
  `parity/termrock-visual-baseline`
- Oracle: tag `visual-baseline` (4169783ae) snapshots
- Status: no change landed; owner decision required

## Finding

The frozen baseline disagrees with itself about the DIM modifier under a
modal backdrop. SGR-2 run counts in the approved captures:

| frozen capture (120x40 unless noted) | tier | `ESC[2m` runs |
|---|---|---|
| `showcase/flows/dialogs/open/80x24` | 256  | 0 |
| `showcase/flows/dialogs/open/80x24` | 16   | 0 |
| `showcase/flows/dialogs/open/80x24` | none | 0 |
| `tablepro/overlays/picker_open`     | 256  | 5 |
| `tablepro/overlays/picker_open`     | 16   | 150 |
| `tablepro/overlays/picker_open`     | none | 151 |

The showcase dialog underlays dim through ink choice alone; the tablepro
picker underlays are DIM-modded. The PR #26 reference tip
(`Theme::backdrop`, `src/theme.rs:1243`, DIM at Ansi16/Mono and on
collisions) matches the tablepro side and contradicts the showcase dialog
side; the parity port (`Theme::backdrop_style`) inherited that behavior, and
the currently-green `tablepro_ui::visual tablepro_visual_baseline` mono
digest pins it.

## What was tried

Removing the DIM branch from `backdrop_style` (commit `ee57fe5e3` before
rewrite) made fresh dialog-open captures emit zero DIM runs at 16/256/none —
toward the showcase frozen output — but flipped the tablepro mono digest
(`quick-switcher 120 40 junie mono`, baseline `af463bd8d299cf6b`) red. One
shared mapping cannot serve both frozen behaviors.

## Decision

Reverted. `backdrop_style` keeps the DIM behavior the currently-green
tablepro frozen output pins; the dialog-side no-DIM divergence stays
recorded here instead of being changed silently. Resolving it needs an owner
decision on which frozen mechanism is authoritative (per-component backdrop
kind, or re-approval of one side), plus a shell/dialog owner to move the
dialog underlay onto the winning path.

Related drift that keeps the dialog-open captures red at every tier:
palette downgrade values (e.g. muted 245 vs frozen 244 at 256, ghost 235 vs
238) owned by the concurrent palette workstream, and app shell content drift
(`docs/audits/2026-10-21-selectable-bypass-removal.md`).
