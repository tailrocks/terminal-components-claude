# Modal backdrop: reference DIM branch contradicts the frozen baseline

- Date: 2026-10-21
- Workstream: theme/visual-state extraction (PR #38), branch
  `parity/termrock-visual-baseline`
- Oracle: tag `visual-baseline` (4169783ae), snapshots
  `showcase/flows/dialogs/open/*` and every other approved dialog-open
  capture

## Finding

The PR #26 reference theme at f4867e9b5f8a08a3bb61ce80d644198a29f51984
(`Theme::backdrop`, `src/theme.rs:1243`) ends with:

```rust
if !blank && (fg == bg || matches!(self.level, ColorLevel::Ansi16 | ColorLevel::Mono)) {
    ... Style::new().fg(ink).bg(bg).add_modifier(Modifier::DIM)
}
```

The parity port (`Theme::backdrop_style`,
`crates/termrock-theme/src/lib.rs`) carried the same branch. SGR histograms
of the frozen captures prove the approved baseline never used it:

| tier | frozen snapshot `ESC[2m` runs | parity actual before fix |
|------|-------------------------------|--------------------------|
| 256  | 0                             | 12 (`2;38;5;245;48;5;235` ...) |
| 16   | 0                             | 85 (`2;38;5;7;48;5;0` ...) |
| none | 0                             | 50 |
| truecolor | 0                        | 0 |

(80x24 `showcase/flows/dialogs/open`; the other dialog-open captures show
the same signature.) At the 256 tier the frozen reference maps the underlay
to distinct inks instead (`38;5;238;48;5;233` — ghost on overlay) with no
modifier; the parity collision came from its then-current 256 palette mapping
`text_ghost` onto `surface_overlay`, and the DIM branch masked the palette
drift.

## Decision

The frozen output wins over the reference tip's code (reference defect:
the DIM branch is either newer than the approved captures or dead in the
capture flows). `backdrop_style` now never adds DIM; a mapped collision
keeps the collision-ink rescue so text stays readable, and blank cells keep
their `fg == bg` plane. The change is truecolor-neutral (the branch never
fired there except on collisions, which keep their ink rescue).

## Evidence after the fix

- `cargo nextest run -p termrock-theme -p termrock-runtime`: 531/532 green;
  the single failure
  (`termrock-theme::capability_palettes all_73_junie_slots_match_reference_capability_output`)
  is pre-existing on the clean tip and belongs to the concurrent palette
  workstream.
- Fresh PTY captures of `showcase/flows/dialogs/open` at 16/256/none after
  the fix: `ESC[2m` runs = 0 in all three actuals (was 12–85).
- `cargo nextest run -p termrock-conformance`: failure set identical to the
  clean tip (293 names, compared by name), i.e. no new conformance
  regressions.
- Updated to the frozen behavior: `modal_backdrop.rs`
  (`reduced_modal_cells_use_muted_ink_and_replace_modifiers`,
  `modal_mapping_collision_retains_readable_ink_without_dim`) and the
  in-crate `modal_backdrop_uses_theme_mapping_and_clears_only_focus_glyph`.

Remaining lower-tier differences in the dialog captures are palette-value
drift (e.g. muted 245 vs frozen 244 at 256) and app shell drift owned by
the concurrent style-chain and shell workstreams, not by the backdrop
mapping.
