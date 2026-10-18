<<<<<<< 81a8bf15cd3042f80649e2b48fed479829518dbd
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

Use multiple agents. Start with independent read-only scouts, then use
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
=======
/goal
# Finish Termrock through reusable components and exact visual-baseline parity

## Active visibility prerequisite

At the start of every task, read [WORK_QUEUE.md](WORK_QUEUE.md) and the accepted task record in [tasks.json](docs/implementation/visibility/tasks.json). The VIS-01 owner publishes the first [STATUS.md](STATUS.md). Until that file exists, use the queue and accepted record for the initial visibility work. After its publication, read the status report at the start of every task. Compare each observed source commit with the current candidate sources, and review changes to measured sources since then. The commit that publishes a status report may differ from the observed source commit; keep those identities separate. Ask the integrator to reconcile stale source facts before relying on the report. Confirm that the claim token and allowed paths match the accepted task record. Confirm that the task remains open at its assigned priority. Ask the integrator to reconcile any mismatch before implementation. P0 visibility work takes priority; broad product refactoring waits until this prerequisite is closed.

These instructions and records are not enforced automatically. `STATUS.md` reports current state. The accepted task records define scope. `WORK_QUEUE.md` lists task priorities. The public API, component ownership, checklist, review, and verification requirements below remain in force. Never move, delete, recreate, or retarget the frozen `visual-baseline` tag or release. Do not reset or rebase `termrock-implementation`, merge `main` into it, force-push it, or rewrite its history. Product-source changes belong only on `termrock-implementation`. The only branch exception is test, report, and CI updates on `origin/visual-baseline`; do not change product source or the frozen tag or release there. Use `origin/visual-baseline` as a test and scenario reference only. Do not use captures from that branch as expected visual output. `main` is structural evidence only, never expected visual output. Run Rust tests with `cargo nextest`; do not use `cargo test`.

## 1. Active task

Finish the real Rust refactor across all 44 workspace crates, all four preview applications
(`showcase`, `jackin-preview`, `holla`, and `tablepro`), the required test coverage, and reproducible CI.
Make all product-source changes exclusively on `termrock-implementation`. The only branch exception is test, report, and CI updates on `origin/visual-baseline`.

Acceptance requires satisfying every condition conjunctively:

```text
exact default visual parity
AND observable interaction parity
AND reusable component ownership
AND simple, coherent public APIs
AND supported customization
AND one implementation of each shared mechanism
AND independently reviewed, executable evidence
```

Preserve the 44-member functionality-based workspace under `crates/`.
Preserve preview application flows, state transitions, CLI contracts, and visible content
using fixture data and simulated operations. Do not connect real external providers, credentials,
databases, Docker, or services.

## 2. Authorities

Make product-source changes only on the existing `termrock-implementation` branch. The only branch exception is test, report, and CI updates on `origin/visual-baseline`.
Do not reset or rebase `termrock-implementation`, merge `main` into it, force-push it, or rewrite its history.
Do not move, delete, recreate, or retarget the frozen `visual-baseline` tag or release.

| Reference | Purpose |
| --- | --- |
| Frozen `visual-baseline` tag (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, object `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`) | Immutable visual and observable interaction authority. |
| `origin/visual-baseline` branch | Test, report, and CI updates only; test/scenario reference after proving tag relation, never expected visual output. |
| `termrock-refactor` | Planning and preparation history. |
| `termrock-implementation` | Active candidate implementation branch. |
| `main` | Structural and API comparison only. Never visual authority. |

Research identities and branch-versus-tag comparison live in
[docs/implementation/findings-record.md](docs/implementation/findings-record.md).

## 3. Write scope

Authorized write scope:
- Rust source (`crates/**/src/**`, `Cargo.toml`, `Cargo.lock`)
- Integration, characterization, and conformance tests (`crates/**/tests/**`)
- Repository tools and xtasks (`crates/termrock-xtask/**`, `.velnor/**`)
- Generated CI configurations and workflows (`.github/**`)
- Documentation, specifications, recipes, and checklists (`docs/**`, `SPECIFICATION.md`, `CRATES.md`, `CHECKLIST.md`, `checklist.json`, `component-ownership.json`, `crate-map.json`, `GOAL.md`, `AGENTS.md`)
- Review receipts (`docs/implementation/review-receipts/**`)

Do not modify frozen baseline artifacts (`baselines/**`) or retarget the `visual-baseline` tag.
Do not stage or commit the `termrock-doc-guardrails/` input pack.

## 4. Canonical owners

| Contract | Owner |
| --- | --- |
| Agent entry rules | Root `AGENTS.md`. |
| Component-only composition | [docs/architecture/component-composition.md](docs/architecture/component-composition.md). |
| Public signatures and ownership | [docs/api/public-api.md](docs/api/public-api.md) and [types.md](docs/api/types.md). |
| Consumer examples | [docs/api/consumer-recipes.md](docs/api/consumer-recipes.md) and [example-catalog.json](docs/api/example-catalog.json). |
| Per-work and per-commit review | [docs/process/per-commit-review.md](docs/process/per-commit-review.md). |
| Visual and ownership verification | [docs/verification/ownership-and-parity.md](docs/verification/ownership-and-parity.md). |
| Individual component contracts | [docs/components/](docs/components/README.md) files. |
| Research record and findings | [docs/implementation/findings-record.md](docs/implementation/findings-record.md). |
| Remediation backlog | [docs/implementation/code-remediation-backlog.md](docs/implementation/code-remediation-backlog.md). |
| Current implementation status | `checklist.json` and derived `CHECKLIST.md`. |

## 5. Central rule: Component ownership

A preview screen is a composition of reusable Termrock components.
The component owns its pixels, geometry, interaction, measurement, and state rules.
The preview supplies domain data, layout choices, durable component state, and typed application actions.

Low-level painting belongs strictly inside reusable library components.
Preview code must not use raw cell writes, full-screen historical painters, scenario/size bypasses,
or painted-over components.

## 6. Execution discipline and per-commit review

Follow [docs/process/per-commit-review.md](docs/process/per-commit-review.md) for every work item and every proposed commit:

```text
approved small work plan
→ implement one coherent change
→ verify the exact candidate
→ independent review
→ commit immediately
→ push promptly
→ continue
```

Freeze the parent SHA, proposed Git tree, and SHA-256 of the full staged binary diff before review.
The reviewer must not author the submitted change.
Commits must strictly use sign-off: `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>`.
All Rust test runs must use `cargo nextest`, never `cargo test`.
>>>>>>> 0b7545ff7e3fff40254fd7a84a80c2c8a7872f3f
