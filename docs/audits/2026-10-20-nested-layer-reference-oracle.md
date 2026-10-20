# Nested layer reference oracle correction

Reference: frozen source commit `4169783ae883788186d161af09ba9c481bd84e5c`.

`nested_public_layers_dim_the_composited_origin_and_preserve_unwritten_cells`
previously asserted that ordinary `X` text became a space and that raw-painted
`R` received a black background. The test itself did not establish either
expectation from the frozen reference.

The frozen `Theme::backdrop` implementation at `src/theme.rs:1243-1310` keeps
glyphs and maps a non-canvas background to `surface_overlay`. Frozen Junie
tokens at `src/theme.rs:734-755` define Ghost as `#262626`, Overlay as
`#27272a`, muted as `#8a8a8a`, and primary as white. Applying that code to the
scenario's explicit styles yields these full cells:

| Cell | Symbol | Foreground | Background | Underline | Modifier |
|---|---|---|---|---|---|
| lower ordinary paint | `X` | `#262626` | `#27272a` | default | none |
| lower raw paint | `R` | `#262626` | `#27272a` | default | none |
| dimmed unwritten page | `P` | `#8a8a8a` | `#000000` | default | none |
| top modal paint | `U` | `#ffffff` | terminal default | default | none |

The regression asserts those cell properties for both layer orders and both
paint paths, plus absence of a cursor. Evidence separates two different
results:

- Before the oracle correction, the current candidate failed the old assertion:
  actual `"X"`, expected the obsolete `" "`. This showed the assertion was
  stale; by itself, it did not demonstrate the old compositor's behavior.
- Reviewer replayed the new assertions against the genuine old `dim_layer`
  compositor at commit `1a947a251ab9b045610b47729bf481419b9fca8d`. It failed
  with actual `(" ", Rgb(0, 0, 0), Rgb(39, 39, 42), Reset, NONE)` and expected
  `("X", Rgb(38, 38, 38), Rgb(39, 39, 42), Reset, NONE)`. This is the negative
  regression proof. Receipt: `/private/tmp/termrock-nested-modal-probes-before-1a947/before.log`.
- On corrected commit `27de89d3af887b200dc813514fc1680d68aa78f1`, reviewer ran
  the same external assertions successfully. Native focused nextest also
  passed 1/1, and the complete `theme_holla_states` target passed 16/16.

No production code, snapshot, stored approval, or aggregate fingerprint changed.

Independent corroboration: reviewer `section_review` diagnosed this stale
expectation from the frozen `ctx.rs:658-688` and `theme.rs:1243-1302` behavior.
