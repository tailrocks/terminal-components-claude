# Goal — restore the historical UI/TUI while finishing the refactor

This is the canonical continuation goal. Read this file before acting. Treat
older execution prompts as historical unless they agree with this contract.

## Mission

Keep the current refactor's architecture, public `junie-tui` API, package
boundaries, security fixes, and product improvements. Restore the user-visible
Showcase, TablePro, and Jackin Preview experience to the known-good
pre-refactor behavior.

The refactor concept is correct. The rendering migration was not. The old
look, geometry, copy, glyphs, spacing, colors, focus treatment, cursor
placement, keyboard behavior, mouse behavior, scrolling, resizing, overlays,
and state transitions are the contract. Do not redesign the products.

Small approved additions are allowed: extra sections, extra state visibility,
and minor product polish may remain when they do not displace or restyle the
historical experience. Record every such addition. Everything else that
differs from the historical evidence is a regression until proven otherwise.

## Current facts

The current repository already has the intended architectural direction:

- Workspace packages: `crates/tui`, `crates/tui-testing`, `apps/showcase`,
  `apps/tablepro`, `apps/jackin-preview`, and `xtask`.
- Public library: `junie-tui` / `junie_tui` under `crates/tui`.
- Current application code: `apps/*/src`; current tests: `apps/*/tests`.
- Shared rendering and interaction code: `crates/tui/src/theme/`,
  `crates/tui/src/components/`, `crates/tui/src/ui/`, `crates/tui/src/runtime.rs`,
  `crates/tui/src/layout.rs`, `crates/tui/src/focus.rs`, and
  `crates/tui/src/hit.rs`.
- The current implementation and tests are not proof of historical parity.
  They mostly exercise the post-migration implementation against baselines
  generated from that same implementation.

Known current evidence:

- Historical evidence contains 499 real-terminal captures in
  `baseline/before/`, with `.ansi`, `.txt`, `.cursor`, `.html`, and `.png`
  artifacts plus exact key/mouse recipes in `baseline/before/MANIFEST.md`.
- The historical source copy is
  `/Users/donbeave/Projects/donbeave/terminal-components-claude` at `d5e7075`.
  The requested `cc14dd6beae526884aabdf897e309be837b4f504` state is the same
  known-good UI/source state for the relevant files.
- Current `shots/capture-matrix.tsv` covers only 96 app matrix cells and its
  provenance is stale relative to current `HEAD`.
- `apps/showcase/tests/baselines/showcase.txt`,
  `apps/tablepro/tests/baselines/tablepro.txt`, and
  `apps/jackin-preview/tests/baselines/jackin.txt` are refactor-era
  self-baselines. They are not historical approval.
- Current app visual tests do not read `baseline/before/**`.
- `Scene::assert_against` compares a digest, not a full artifact contract.
  `.txt`, `.ansi`, `.cursor`, and human-reviewed `.png`/`.html` comparisons
  are required for parity.
- Existing state docs contain many superseded checkpoint claims. Source,
  fresh commands, fresh captures, and this goal outrank stale prose.

## Root cause to fix

The migration treated product rendering as replaceable implementation instead
of preserving it behind the new architecture. This removed the executable
visual oracle before a parity harness existed.

Evidence:

- `18afddd` added the new foundations. This was the correct architectural
  direction.
- `7784719` removed the historical root renderer and widgets before parity was
  proven. This removed the strongest regression oracle.
- Showcase was rewritten during migration (`4e07ea1`; current shell at
  `apps/showcase/src/app.rs:627`). The historical shell owns distinct header,
  sidebar, inspector, main, footer, compact-layout, and hit/focus behavior at
  the old `src/bin/showcase/app.rs` shell/render functions.
- TablePro was rewritten around the new facade (`5042a40`; current update/draw
  at `apps/tablepro/src/app.rs:1016` and `:1190`). The historical connection
  surface, workbench split, explorer, result grid, tabs, dialogs, and editor
  geometry must be restored.
- Jackin's journeys/screens were rewritten (`444a8f4`; current route/update
  and draw paths at `apps/jackin-preview/src/app.rs:2544` and `:3257`). The
  historical manager, editor, accounts, usage, cockpit, Capsule, menus,
  dialogs, and responsive states must be restored.
- Facade-only enforcement (`1378c31`) cemented the divergence by making the
  new output the only tested output without first proving visual equivalence.

The structural fix is a parity-first rendering contract: preserve historical
composition and interaction semantics, then implement them through the new
runtime/components. Do not paper over digest mismatches by blessing them.

## Authority and non-goals

Authority, highest first:

1. This goal and explicit user-approved additions.
2. Historical source at `d5e7075` / `cc14dd6`.
3. `baseline/before/MANIFEST.md` and its captured artifacts.
4. Historical interaction tests and source behavior.
5. Current product semantics and current security fixes.
6. Current refactor docs, current self-baselines, and current rendered output.

Do not:

- redesign Showcase, TablePro, Jackin, or the Junie/Paper visual language;
- replace historical output with a cleaner-looking or more generic shell;
- delete product screens, fields, menus, states, or interactions to make a
  digest pass;
- bless a post-refactor baseline merely because a test is green;
- reset, clean, checkout, overwrite, or discard unrelated worktree changes;
- reopen the accepted component architecture without concrete parity evidence;
- move SQL, Jackin, or other product/domain logic into `junie-tui`;
- weaken tests, broaden allowlists, or hide failures behind ignored cases;
- claim parity from a compile-only check or a text-only digest.

The old baseline includes a few known defects documented in
`baseline/before/NOTES.md`. Fixing a defect is allowed and desirable, but it
must be isolated, tested, documented as a deliberate behavior fix, and never
used to excuse unrelated visual drift.

## Required execution model

Use multiple subagents. Start with independent read-only scouts, then use
builders with disjoint write ownership, then use an independent reviewer.

Required roles:

1. Baseline mapper: map all 499 historical recipes to current constructors,
   routes, sizes, themes, color modes, and input sequences.
2. Shared-renderer investigator: identify token, recipe, glyph, layout, paint
   order, focus, hit, cursor, scroll, and layer differences.
3. Showcase investigator/builder.
4. TablePro investigator/builder.
5. Jackin investigator/builder.
6. Parity-harness builder: make old-vs-current comparisons executable and
   fail closed.
7. Independent visual/interaction reviewer.
8. Final gate reviewer.

No two builders may edit the same file at once. Use separate worktrees or an
explicit ownership table. One integrator merges/reconciles changes. Every
builder reports changed files, preserved behavior, commands, failures, and
remaining work. Do not redo a completed scout's investigation without new
evidence.

## Phase 0 — safe preflight

1. Record `git status --short --branch`, `git log --oneline -10`, current
   `HEAD`, and toolchain. Preserve all existing `.codex-target-*` directories
   and unrelated changes.
2. Inspect `baseline/before/MANIFEST.md`, `baseline/before/NOTES.md`, the old
   local copy, current `README.md`, current source, current tests, and current
   state docs.
3. Build old and current code with separate target directories. Never mix old
   and current binaries or captures.
4. Run focused current tests before editing. Record failures as baseline
   evidence, not completion claims.
5. Freeze the historical artifacts. No command may update
   `baseline/before/**`.

## Phase 1 — build the parity oracle first

Create a machine-readable mapping from each historical capture to:

- owning app and route/page/surface;
- viewport and color level;
- theme;
- initial state;
- exact key/mouse/resize/tick recipe;
- expected historical `.txt`, `.ansi`, `.cursor`, and optional `.png`/`.html`;
- current replay command and current artifact paths.

Run both implementations for identical recipes. Compare:

- `.txt`: exact dimensions, glyphs, copy, row/column geometry, clipping,
  wrapping, blank space, and paint order;
- `.ansi`: foreground/background colors and modifiers;
- `.cursor`: position and visibility;
- `.png`/`.html`: independent human visual review;
- event result and state after every input step;
- focus order, hover/pressed/selected/disabled/editing/error/busy states;
- wheel, drag, mouse click, resize, overlay dismissal, and scroll ownership.

The comparator must fail closed on missing mappings, missing artifacts,
stale provenance, changed dimensions, or unexplained differences. It must
print the first differing artifact, recipe, coordinates, and source owner.

Do not make the current post-migration baselines the reference. Replace or
retain them only after current output is proven equal to the historical output
plus explicitly recorded additions.

## Phase 2 — restore shared visual contracts

Fix shared causes before scattering app-specific patches. Compare the old
`src/theme.rs` and `src/widgets/` behavior with the current equivalents.

Preserve exactly unless an approved bug fix requires otherwise:

- Junie and Paper token values, color downgrade behavior, and monochrome
  fallback;
- semantic role resolution and per-state style precedence;
- glyph mapping, width accounting, reserved pads, borders, underlines, and
  focus bars;
- `Panel` card/framed/overlay geometry, padding, title/meta placement, and
  surface fills;
- button/list/choice/select/input/textarea/form/table/grid/tree/tabs/menu/
  picker/dialog/status/hint/progress/viewport behavior;
- focus registration and order, hit regions, pointer capture, cursor, scroll,
  layer bounds, modal barrier, and resize recovery;
- wrapping, truncation, overflow rows, empty states, and narrow-terminal
  layout thresholds.

The new API is an implementation boundary, not permission to change the
contract. If the new API cannot express an old contract, add the narrowest
correct capability and test it. Do not redesign the component model.

## Phase 3 — restore each application

### Showcase

Restore the historical shell and all 22 pages through `apps/showcase`:

- header lockup/breadcrumb/capability/actions;
- section labels, navigation spacing, current marker, hover/focus behavior,
  compact sidebar rules, inspector, and footer hints;
- every page's original layout, cards, panels, copy, sample data, dimensions,
  colors, and state matrix;
- dialogs, menus, pickers, completion, editor, grid, scrolling, inspector,
  resize, and mouse flows.

Keep modern ownership and public API usage. Move only the rendering/interaction
contract back to the historical behavior.

### TablePro

Restore the historical product flow and frame contract:

- Connections screen and selection/focus/error/reconnect states;
- identity strip, explorer/workbench split, drawer threshold, tabs and tab
  overflow;
- SQL editor, completion, diagnostics, result tabs, grid, sorting, filtering,
  edits, pending changes, SQL preview/save, structure, history, quick switcher,
  EXPLAIN, Safe Mode, safety dialogs, help, scroll, and resize;
- old status/hint placement, button treatment, borders, table widths, row
  density, and modal geometry.

Keep query safety, pending-edit semantics, deterministic database fixtures, and
the generic-library/domain boundary. Do not fix a visual mismatch by removing
data or product behavior.

### Jackin Preview

Restore route by route:

- intro/outro ritual and timing/copy;
- manager, prelude, editor, settings, accounts, usage, cockpit, handoff, and
  Capsule surfaces;
- host/Capsule chrome, menus, tabs, pickers, dialogs, status and hint bars;
  file browser, terminal panes, scrollback, split/zoom, inspect/diff, and
  responsive behavior;
- scenario fixtures, key/mouse journeys, focus order, cursor, overlays,
  transitions, and virtual-clock behavior.

Keep the modern simulation/domain/security architecture. Fix known panics and
unreachable controls separately, including safe handling of short identifiers
and any advertised binding that can be made reachable without changing the
historical contract.

## Phase 4 — tests and evidence

Add parity tests at the narrowest useful layers:

- shared component contract tests against historical frames where applicable;
- app route/surface tests with exact geometry/text/style/cursor assertions;
- interaction replay tests that compare every transition, not only the final
  frame;
- current-vs-historical artifact comparison for all mapped captures;
- tests proving approved additions are additive and do not move protected
  historical regions;
- tests for known bug fixes and secret redaction/security invariants.

Keep the historical archive immutable. Update current baselines only through an
explicit review record containing:

- old artifact hash and current artifact hash;
- exact diff classification;
- whether the difference is restoration, approved addition, or bug fix;
- reviewer and command/capture provenance.

No baseline update is valid if it only makes a test green.

## Phase 5 — required verification

Run after each app/shared-layer integration and again at the end:

```sh
rtk cargo fmt --all -- --check
rtk git diff --check
rtk cargo check --workspace --all-features
rtk cargo build --workspace --all-targets --all-features
rtk cargo test --workspace --all-targets --all-features
rtk cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" rtk cargo doc --workspace --all-features --no-deps
rtk cargo test --workspace --doc --all-features
rtk cargo run -p xtask -- doc-check
rtk cargo run -p xtask -- boundary
rtk cargo test -p junie-tui --test render --test render_components
rtk cargo test -p showcase --test visual
rtk cargo test -p tablepro --test visual
rtk cargo test -p jackin-preview --test visual
```

Run all three real binaries. Replay the full historical capture recipe and
run the parity comparator. Use a fresh provenance-backed capture matrix. Run
the complete workspace gates with a validated comparison base:

```sh
BLESS_GUARD_BASE=d5e7075f436f0e437c7d12cf3d1e638e763b26f6 \
  rtk cargo run -p xtask -- bless-guard
```

If a command fails, record the exact exit code/output and continue independent
work. Do not call a slice complete while a required gate or parity comparison
is unresolved.

## Completion conditions

Stop only when all are proven with fresh evidence:

1. All 499 historical recipes have a current replay or an explicit, reviewed
   reason they cannot run; no missing mapping is silently skipped.
2. Showcase, TablePro, and Jackin match historical geometry, copy, glyphs,
   colors, modifiers, cursor, focus, input behavior, overlays, scrolling,
   resizing, and state transitions, except recorded approved additions or
   isolated reviewed bug fixes.
3. Current visual tests fail when a protected historical artifact changes.
4. Current baseline files are derived from verified parity, not post-refactor
   self-output.
5. New component/runtime APIs remain in use; no return to duplicate legacy
   production paths is needed.
6. Product semantics, SQL safety, Jackin simulation, and secret redaction
   remain intact.
7. Fresh captures have current provenance; stale `shots` evidence is replaced
   or clearly marked historical.
8. Full build, test, clippy, doc, boundary, capture, comparator, and review
   gates pass.
9. `REFACTORING_STATE.md` gets one current appended checkpoint; do not rewrite
   historical entries. It lists remaining work as none only when the proof is
   real.
10. The final report names every changed file, every approved visual change,
    every command/capture/review result, and any intentionally retained known
    defect.

## Documentation cleanup

`REFACTORING_GOAL.md` and `COMPONENT_ARCHITECTURE.md` remain architecture
reference documents. They do not override this parity-first execution order.
`REFACTORING_STATE.md` is an evidence ledger, not proof by itself.

The old `docs/REFACTORING_AUDIT_REPORT.md`,
`docs/REFACTORING_EXECUTION_GOAL_PROMPT.md`, `CONTINUE_PROMPT.md`, and
`RESUME_PROMPT.md` contain superseded migration assumptions. Mark them as
historical or redirect them to this file. Do not leave contradictory active
instructions such as “apps are not migrated” or obsolete model-routing rules.

## Final handoff

Report tersely but with evidence:

- restored surfaces and exact approved additions;
- remaining defects/blockers, if any;
- current-vs-historical artifact counts and first/last mismatch if present;
- every required command and result;
- app runs, capture provenance, and independent visual review;
- current branch/commit and worktree status;
- exact files changed.

Never claim completion from percentages, documentation, or self-blessed
baselines. The output must be visibly the old product, implemented through the
new architecture.
