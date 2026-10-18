# Showcase

Showcase composes reusable Termrock components under the [component-only contract](../architecture/component-composition.md). Its ordinary consumer shape is [EX-19](../api/consumer-recipes.md#ex-19--showcase-proves-ordinary-consumer-use).

## Role

Showcase is the reference laboratory for Termrock components. It exposes
individual components, composed screens, and their meaningful focus, hover,
editing, selection, overlay, scrolling, resize, color, and motion states. It
is a conformance consumer, not a product roadmap or a redesign target.

The frozen oracle is commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Preserve every rendered page,
fixture, keyboard route, mouse target, timing boundary, and approved artifact
under `snapshots/showcase/`.

## Current implementation

The current binary is `showcase` (`src/bin/showcase/main.rs`) and the current
package/library identifiers remain `junie-tui` and `junie_tui`. The current
CLI title and boot needle still use the legacy Junie wording. Future Termrock
renaming must preserve those baseline strings when they are part of an
approved frame.

The navigation contains 23 pages:

```text
overview          buttons          inputs             textareas
forms             lists             trees              tables
editabletables    panels            sidebars           dialogs
progress          scrolling        terminal           codeeditor
diff              datagrid         chipsselects       pickers
chrome            settings         taskrunner
```

These page names are current CLI values. Their component ownership is
specified in [component contracts](../components/README.md); shared focus,
hit testing, layers, layout, theme, text, and scrolling belong to the
[foundation contracts](../foundations/README.md).

## Preserved shell composition

The shell is one header row, a blank row, the body, a blank row, and a one-row
footer. The page body uses the full width. Its navigation sidebar is 19 cells
wide, or 24 cells once the terminal is at least 110 columns wide, with a
two-cell gap before the page. A 30-cell inspector appears at 100 columns and
wider. Below 25 rows, the sidebar drops section headings and blank rows and
becomes one contiguous list. Preserve these breakpoints and page geometry;
they are part of the frozen laboratory output.

## Run and inspect

```sh
# Overview, the default page.
cargo run --release

# A named page, with an explicit color path.
cargo run --release -- --page datagrid --color truecolor
cargo run --release -- --page diff --color 256

# Freeze tick-derived content at a reproducible frame when the page supports it.
cargo run --release -- --page progress --motion paused --frame 80
cargo run --release -- --page scrolling --motion paused --frame 1600

# CLI options and current page names.
cargo run --release -- --help
```

`--color` accepts `truecolor`, `256`, `16`, and `none` (with `24bit` and
`mono` aliases). `NO_COLOR=1` exercises the environment-selected no-color
path. `--motion paused --frame N` is the deterministic capture path for
tick-derived fixtures; do not infer a new timing contract from a live run.

## Baseline evidence

Source and behavior evidence is kept at the frozen commit:

- `src/bin/showcase/app.rs` — shell routing, focus, hit ownership, resize and
  the page inventory.
- `src/bin/showcase/pages/` — the page fixtures and their compositions.
- `tests/visual_baseline/showcase.rs` — approved page and interaction capture
  invocations, including focus, editing, dialogs, pickers, grid, editor,
  scroll, terminal and resize cases.
- `snapshots/showcase/` — approved grouped `.ansi`, `.txt`, `.png`, and
  `.html` output for the five standard sizes and declared color paths.

Run the Showcase lane with:

```sh
cargo nextest run --run-ignored only \
  -E 'binary(visual_baseline) & test(showcase_)'
```

The future Termrock implementation must prove the same semantic transitions
as the frames: keyboard navigation and editing, completed click versus
release-outside cancellation, hover suppression/restoration, disabled and
read-only controls, modal focus trapping/restoration, stable selection keys,
scroll and resize retention, and every applicable animation phase. A passing
static screenshot without the matching interaction trace is insufficient.

## Boundary

Only reusable library work belongs in the future migration. Keep the 23-page
Showcase fixture world and its output intact while replacing legacy library
internals with Termrock public APIs. Do not add application features, remove
pages, simplify fixtures, or restyle the laboratory as part of the refactor.
