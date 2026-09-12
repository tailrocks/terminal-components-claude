# Component Architecture — Current Holla Tree vs. Target

**Provenance.** Compiled 2026-09-12 on branch `holla` from three sources:
(1) `COMPONENT_ARCHITECTURE.md` at `main` (the adjudicated target, accepted
through §74), (2) `DESIGN.md` in this working tree (the approved
visual/behavioral contract), (3) the `holla` branch source tree (`src/`) and
the `main` branch tree (`crates/`, `apps/`, `xtask/`) read via `git ls-tree` /
`git show`. Section references `§n` are to `COMPONENT_ARCHITECTURE.md`.

---

## 1. Current holla architecture (this branch)

A single Cargo package `junie-tui` (lib `junie_tui`) with `default-run =
"showcase"` and four binaries under `src/bin/`: `showcase`, `tablepro`,
`jackin-preview`, `holla` (root `Cargo.toml`). Edition 2024, `rust-version =
"1.88"`. `src/lib.rs` is a flat `pub mod` list — `core`, `runtime`, `theme`,
`ui`, `widgets` — with no facade, so every `pub` item in the library is also
reachable from the in-package binaries.

### 1.1 `src/core/` — framework primitives

- `event.rs` — crossterm → `Input` normalization and the `Outcome` reply
  enum; widgets reply `Outcome` so a router can decide propagation/redraw.
- `id.rs` — `WidgetId(u64)`: an FNV-1a hash of a path string, extended with
  positional `child(index)` for repeated rows; `Debug` prints only the hex
  hash. The target doc proves `Id::of(a).sub(b) == Id::of(ab)` exactly and
  keys children on *display* index (§1.2 defect 1).
- `focus.rs` — `FocusRing` rebuilt every frame in render order (Tab order =
  reading order), with a modal barrier index.
- `hit.rs` — `HitRegistry` rebuilt per frame; later registrations win, so
  overlays shadow what is beneath them.
- `scroll.rs` — `ScrollState`: a pure, rendering-free scroll model.
- `text.rs` — grapheme-aware `TextBuffer` shared by single- and multi-line
  editors; byte-offset addressing, grapheme-correct motion.

### 1.2 `src/ui/` — rendering support

- `ctx.rs` — `RenderCtx`: the per-frame snapshot (focus/hover/press) handed
  to widgets, through which they register hit regions and ring entries for
  the *next* event cycle.
- `layout.rs` — split panes and layout helpers (`Split::vertical` /
  `horizontal` collapse asymmetrically — a recorded latent defect, §1.3).
- `popup.rs` — anchored non-modal popups drawn last with a hit barrier; all
  popups share one `WidgetId::of("popup.surface")` (§1.3).
- `fade.rs` — scroll-edge fade (foreground ramp, emphasis-aware).
- `text.rs` — truncation/ellipsis/width helpers and `fuzzy` matching.

### 1.3 `src/widgets/` — 31 component modules

`brand, button, chips, choice, code, completion, dialog, diff, empty,
field_common, grid, hintbar, input, keyhint, list, menu, panel, picker,
progress, props, scrollbar, segments, select, splitter, statusbar, steps,
table, tabs, textarea, tree, viewport` (`src/widgets/mod.rs`). Every widget
is one plain struct mixing props, durable state and last-frame geometry
(often `pub area: Rect`), with:

- `render(&mut self, area, buf, ctx, bg: Color)` — draws *and* registers
  hit/focus regions; `&mut self` makes render-time semantic mutation legal
  (nine confirmed violations, §1.2 defect 5: commit-on-focus-loss in
  `input.rs`, overlay close in `select.rs`, action arming in `dialog.rs`,
  a staged DB mutation in `grid.rs`).
- small `on_*` handlers returning `Outcome` — one of **nine incompatible
  reply shapes** across the tree (§1.2 defect 2).
- collections own `String` items rebuilt per frame with no row-renderer hook
  (`ListBox { items: Vec<ListItem> }`, `row_id(i) = id.child(i)`,
  `src/widgets/list.rs`).
- the surface colour is threaded by hand as a trailing `bg: Color` through
  24 render signatures (§1.2 defect 7).

### 1.4 `src/theme.rs` — one 723-line file

`ColorLevel` detection (`NO_COLOR`/`COLORTERM`/`TERM`), a private `mod
palette` holding every RGB literal (the one place colours are spelled), and
a flat `Theme: Copy` struct of ~30 semantic colour fields. Downgrade
(`for_level`) is Junie-only and re-lists all 30 fields in a macro, so a new
token is silently skipped; `lift()`/`backdrop()` dispatch on colour
*equality* against Junie tokens (§1.2 defect 6).

### 1.5 `src/runtime.rs` — terminal session + event loop

`TerminalSession` owns raw mode, alternate screen, mouse capture, bracketed
paste, cursor and line-wrap state with restoration on every exit path
(including panics); the event loop drains unchanged input and renders state
changes; `SIGTSTP`/`SIGCONT` job control is handled via an atomic-flag
handler. Applications implement `Application { handle(Input) -> Outcome,
render(&mut Frame), should_quit, tick_interval }`.

### 1.6 `src/bin/*` — the four applications

Each app re-implements the interaction machine by hand: hover and hover
suppression, press/release/activation with the 140 ms flash, focus
save/restore, wheel target lookup, click-outside, double-click, cursor
plumbing, and `owns()`/`locate()` dispatch chains (≥ 66 chained call sites
in showcase, §1.2 defects 3–4). Because the binaries live in the same
package, they can and do reach `pub(crate)` machinery (`HitRegistry`,
`FocusRing`) — the boundary a workspace makes compiler-enforced
(Appendix B.1).

### 1.7 `DESIGN.md` — the behavioral contract this tree implements

`DESIGN.md` (1405 lines) fixes the approved design language: colour planes
and the text/border/accent ladders, typography, layout rhythm, the
interaction grammar (Tab order = reading order, the `Esc` ladder, 140 ms
press flash, hover suppression after any key press, click-outside
dismissal), the focus model (frame-rebuilt ring, modal barriers, focus
save/restore), the state grammar (empty/loading/partial/error/disabled/
read-only/pending/success treatments) and a per-component catalogue. The
target preserves these verbatim: the Junie token values become
`Theme::junie()` (§1.1), and the grammar entries reappear as runtime-owned
behaviour and the mono fallback manifest (§8.6, §11.4).

---

## 2. Target component architecture (`COMPONENT_ARCHITECTURE.md` at `main`)

The document is the single source of truth for the refactor, accepted
through §74, with authority order `REFACTORING_GOAL.md` › `DESIGN.md` ›
rendered output/tests › current source. Its §1 diagnoses the tree described
in §1 above (nine structural defects + latent defects); the rest is the
binding design.

### 2.1 The component model (§3) — three layers, two phases

1. **Durable interaction state** — caller-owned `XState` structs: no
   lifetimes, no `Rect`/`Color`/`Style`, no borrowed data; `Debug + Clone +
   Default + PartialEq` (invariant S2). Stateless components have none.
2. **Per-frame props** — short-lived `X<'a>` structs borrowing application
   data, built with consuming builders; **props never borrow state**.
3. **Two phases with pre-resolved intents** —
   `update(&self, cx: &mut Cx<'_>, st: &mut XState) -> Response<XAction>`
   (no `Buffer` in scope, headless-testable, the *only* place semantics
   change) and `draw(&self, ui: &mut Ui<'_>, area: Rect, st: &XState) ->
   Rect` (paints, registers regions, reports layout facts). Render purity is
   a **compile error**, not a review rule — the nine §1.2(5) violations
   become structurally impossible. No `show()`, no universal `Widget` trait;
   uniformity comes from naming/signature conventions (§13) enforced by an
   architecture check.

The runtime frame sequence (§3.3): `Runtime<A>::handle(Input)` normalizes
input, matches the app Capture keymap, resolves pointer/wheel/key against
**last frame's** `Registry`/`FocusRing` (never a fresh app-tree scan), runs
runtime-owned interaction bookkeeping (hover, suppression, press, 140 ms
flash, double-click, capture), applies focus policy, delivers `Intent`s to
the focused/hit owner, then the app bubble phase; draw runs as a separate
pass with deferred layer compositing. Everything §1.6 lists as
app-implemented moves into this one machine.

### 2.2 State ownership, rendering, events (§4–§6)

- §4 gives each of the seven state concerns exactly one home (app domain
  data → the app as `&'a [T]`; props → frame-local builders; durable state →
  caller's `XState`; controlled values → caller, drafted in `XState` until
  commit; geometry → the runtime's `Registry`; theme → `Ui`/`Cx`; actions →
  the `Response` return channel), with invariants S1–S6 (no public fields,
  geometry only via `cx.area(id)` = last frame's facts, controlled by
  default, explicit `reconcile`, draw-time facts flow *up* via
  `ui.report_layout`).
- §5 fixes the `draw` signatures (R1), forbids mutation in draw (R2),
  routes all painting through `Ui` so a per-layer written-cell bitset stays
  exact (R3), makes clipping automatic (R4), degenerate rects safe (R5),
  draw idempotent (R6), layer painting deferred-composited (R7).
- §6 replaces the nine reply shapes with one type: `Response<A> { id, flow,
  invalidate, state, action }` with orthogonal `Flow`/`Invalidate` (so
  "consumed at a scroll boundary without repaint" is expressible), 16-bit
  `StateFlags` (`bitflags`), and small typed per-component `XAction` enums
  in which every item-targeting action carries an `ItemKey`, never `usize`.
- §7 replaces the FNV hash + positional `child(index)` with `Id`/`ItemKey`
  stable keys (`id!` spec); `ByIndex` exists but is documented as unstable
  under reorder.

### 2.3 Focus, overlays, layout (§8–§10)

- §8: focus scopes and traps, runtime-owned pointer capture, wheel routing
  (innermost scrollable on the top layer), single cursor owner, runtime-
  owned hover/press/activation/click-outside, and an explicit
  `FeedbackClock` (wall `Elapsed` vs. `Simulation` for the Holla/Jackin
  simulation fidelity).
- §9: a **runtime-owned layer stack** — `Cx::open_layer(id, LayerSpec{
  kind, owner, anchor, dismiss, size, backdrop, inert_below })`,
  `Ui::layer(id, |ui, rect| …)`; content is drawn by the app into the layer
  (borrows freely, nothing boxed or `'static`). Modals trap focus and
  pointer; popovers trap pointer only. `DialogBody` is deleted — dialog
  content is an open slot; the component computes a *size*, the runtime
  computes the *rect*. `begin_modal` and the shared `"popup.surface"` id are
  deleted.
- §10: `Measure` trait + a small `layout` module (`rows`, `columns`,
  `action_row`, `responsive_columns`, `Track::{Fixed, Flex, Auto}`,
  one axis-parameterised `Split`), and **surface inheritance**: a
  `Surface` ladder (`Canvas, Surface, Elevated, Overlay, Popover` +
  `Field`/`FieldHover`) pushed by containers via `Ui::with_surface`, where
  `raise` is ladder-index arithmetic — deleting every `bg: Color`
  parameter and the equality-dispatch `lift()`.

### 2.4 Theme and customization (§11)

Concrete data, no theme trait: `ColorTokens` arrays + `DesignTokens`
(spacing, sizes, glyphs, borders, motion, density, meter thresholds), typed
`Recipes` keyed `(Family, Variant, Part)` with ordered `StateRule`s, and
`StylePatch` storing **`Role`, never `Color`** with `Slot<Inherit|Set|Clear>`
semantics. A six-level precedence chain (recipe base → variant delta →
state rules → mono fallback manifest → global override → scope overlay →
instance patch) resolves, memoised per frame; roles bind to colours last,
against `(theme, surface, capability)`. Capability downgrade is generic
(`downgrade_color`: nearest-256 / luma-ladder-16 / mono) with optional
authored semantic palettes; `Theme::paper()` is the required distinct
non-Junie (light) builtin. §11.5 maps each concern to its home (colour →
`ColorTokens`, spacing → `design.space`, glyphs → `design.glyphs`, …).

### 2.5 Composition, collections, domain boundary (§12, §14)

Composition via slot closures (`impl FnOnce(&mut Ui, Rect) -> R`), part
replacement via `.slot(Part, &'a dyn Fn …)`, and one collection vocabulary
for `List/Tree/Grid/Tabs/Picker/Completion/Props/Steps/Chips`: caller keys
(`KeyFn`), `Fn` row renderers painting through `RowUi`/`CellUi`,
`EmptyState`, owner-supplied `RowDecor`/`CellDecor`, the `Reconcile` trait
with one table-driven reconcile rule, and a shared `ScrollRegion`
component (deleting the seven `on_scrollbar` copies). `DataTable` is
deleted; `Grid` is the one tabular component behind `GridModel` /
`GridEditor`, with capability chosen by entry point (`update` takes `&M`,
`update_editable` takes `&mut M`). Everything database-shaped (pending
changes, undo, SQL preview, PK/nullability/FK) moves to
`apps/tablepro/src/grid_model.rs` — goal G8: no TablePro/Jackin vocabulary
in the library.

### 2.6 Package layout (Appendix B)

The repo becomes a **virtual Cargo workspace** (the `pub(crate)` boundary
is unenforceable while apps share the library's package):

- `crates/tui` — package `junie-tui`: `#![forbid(unsafe_code)]`,
  `#![deny(missing_docs)]`, a **curated `lib.rs` facade** (all modules
  `pub(crate)` except `theme`, `layout`, `author`; one reviewable
  re-export line per public item), two documented layers —
  `junie_tui::*` for application authors, `junie_tui::author::*` for
  component authors. Internal layout: root vocabulary modules (`id`,
  `event`, `intent`, `response`, `keymap`, `focus`, `hit`, `capture`,
  `scroll`, `cursor`, `layer`, `runtime` + `runtime/session.rs`,
  `diagnostics`, `secret`, `validate`, `field_control`, `layout`,
  `measure`), plus `ui/`, `text/`, `theme/` + `theme/builtin/{junie,
  paper}`, `collection/`, and `components/` (38 files listed in B.2).
- `crates/tui-testing` — `junie-tui-testing`, `publish = false`, harness /
  digest / perf / conformance driver on the library's `testing` feature.
- `apps/{showcase,tablepro,jackin-preview}` — each a `[lib]` + thin
  `[[bin]]` preserving the exact binary names; the app's only normal
  dependency is `junie-tui`.
- `xtask` — boundary checks, bless-guard, capture matrix driver.
- Dependency set fixed at `ratatui-core` + `ratatui-crossterm` +
  `unicode-width` + `unicode-segmentation` + `bitflags`; workspace lints
  deny `panic`, `indexing_slicing`, `unwrap_used`, `expect_used`,
  `todo`, `unimplemented`, `print_stdout`, … (B.2).

Testing (§16): unit tests on buffer-free `XState` machines, a shared
conformance suite, render/snapshot baselines, app integration tests,
executable **architecture checks** (e.g. `no_domain_vocabulary_in_the_
library`, `palette_literals_are_confined_to_theme_builtins`), and perf
gates.

---

## 3. Delta — current tree → target

### 3.1 What the refactoring must achieve structurally

| # | From (holla branch) | To (target) |
|---|---|---|
| 1 | One package: `junie-tui` lib + 4 in-package bins (`src/bin/*`), `default-run` | Virtual workspace: `crates/tui`, `crates/tui-testing`, `apps/*` (lib + thin bin), `xtask`; binary names preserved; `default-run` dropped (B.1) |
| 2 | Flat `pub mod` facade (`src/lib.rs`); apps reach `pub(crate)` internals | Curated facade, `pub(crate)` modules, two layers (`junie_tui::*`, `author::*`); cross-crate boundary makes internals unnameable (B.1, B.3) |
| 3 | `src/core/{event,id,focus,hit,scroll,text}` | Root vocabulary modules + `intent.rs`, `response.rs`, `keymap.rs`, `capture.rs`, `cursor.rs`, `layer.rs`, `diagnostics.rs`, `secret.rs`, `validate.rs`, `field_control.rs`; text moves to `text/{buffer,editor,measure,fuzzy,span}` |
| 4 | `src/ui/{ctx,layout,popup,fade,text}`; popups drawn last by hand | `ui/{mod,cx,paint,surface,layer_buf}` + `layer.rs` runtime-owned layer stack with deferred compositing; `layout.rs`/`measure.rs` primitives; `popup.rs` deleted |
| 5 | 31 `src/widgets/*` structs with `render(&mut self,…,bg: Color)` + `on_*` → `Outcome` | 38 `components/*` with `XState` + borrowed props + `update`/`draw` split; `Response<XAction>`; no public fields, no geometry in state, no `bg` parameter (Surface inheritance) |
| 6 | `WidgetId` FNV hash + positional `child(index)` | `Id`/`ItemKey` stable keys; every collection takes a `KeyFn`; actions carry `ItemKey` |
| 7 | `src/theme.rs`: one flat 30-field `Copy` struct, Junie-only macro downgrade, equality-dispatch `lift`/`backdrop` | `theme/{tokens,role,glyph,recipe,patch,resolve,downgrade,border}` + `builtin/{junie,paper}`; role-based `StylePatch`, six-level precedence, generic + authored downgrade |
| 8 | `src/runtime.rs`: session guard + loop; apps run the interaction machine | `runtime.rs` + `runtime/session.rs` owning hover/press/flash/double-click/capture/focus/wheel/cursor and the layer stack; apps express product intent only (G3) |
| 9 | `DataGrid` carries DB domain (pending changes, undo, SQL preview) | `GridModel`/`GridEditor` in the library; all DB logic in `apps/tablepro/src/grid_model.rs` (§12.3, G8) |
| 10 | Ad-hoc tests; review-enforced rules | Conformance suite, render baselines, executable architecture checks, perf gates, trybuild compile-fail cases (§16) |

### 3.2 What `main` has already realized

Reading the `main` tree against Appendix B, the structural target is
**landed, not pending**:

- The workspace exists exactly as specified: root `Cargo.toml` is virtual
  (`resolver = "3"`, `workspace.package` edition 2024 / rust 1.88, the
  fixed dependency set, the B.2 lint table); the root package and its
  `src/` are gone entirely (the staged "root keeps `junie-tui` during
  Slices 3–4" arrangement in B.1 has completed).
- `crates/tui/src` matches B.2 almost file-for-file: all root vocabulary
  modules, `ui/`, `text/`, `theme/` with `builtin/{junie,paper}.rs`,
  `collection/`, `runtime/session.rs`, `author.rs`, and `components/` with
  39 files — B.2's 38 plus `empty.rs`. Small additions beyond B.2's literal
  list: `action.rs`, `keys.rs`, `theme/builder.rs`, `theme/palettes.rs`.
- `crates/tui/src/lib.rs` is the curated facade as specified:
  `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`, `pub mod` only for
  `theme`, `layout`, `author`, one `pub use` line per item, feature gates
  `crossterm`/`testing` (the `Session`-only `crossterm` gate of §25 D-1).
- `crates/tui/examples/01..13` plus `showcase_buttons.rs` exist (the §17
  scenario proofs), as do `crates/tui/tests/{conformance,render,
  architecture,perf}.rs`, `perf_baseline.txt`, `tests/ui/` trybuild cases
  and `tests/allow/` lists, plus a large body of per-contract tests
  (grid_*, layer_*, nav_list_*, theme_holla_states, feedback_clock, …).
- `crates/tui-testing` exists; `apps/showcase`, `apps/tablepro`,
  `apps/jackin-preview` exist as packages with `src/` + `tests/`;
  `docs/guides/{quickstart,theming,overrides,authoring,migration}.md` and
  `docs/visual-changes.md` (the §20.10 ledger) exist; `xtask/` carries the
  check tooling.
- **Beyond the paper:** `main` adds `apps/holla` (package with
  `src/{app,cli,clock,dispatch,scenario}.rs` plus `app/`, `domain/`,
  `screens/`, `sim/` submodules) — Appendix B names only the three original
  apps, so the Holla application's workspace form exists only on `main`,
  not in the adjudicated layout.

### 3.3 What exists only on paper (relative to this branch)

From the `holla` branch's vantage point, *all* of §2 is unrealized: this
tree is exactly the pre-refactor monolith that §1 of the target document
diagnoses (same file paths and line-level defects), plus the fourth binary
`src/bin/holla`. Nothing of the workspace, the two-phase component model,
the layer stack, the recipe/theme system, the collection vocabulary, or the
testing machinery exists here. The gap is therefore not a set of patches to
this tree but a port: the branch's unique content (the `holla` application)
must land on top of `main`'s realized workspace as `apps/holla` — which
`main` has already done in its own line — rather than evolving `src/`
toward the target in place.

One caveat on "realized": §3.2 verifies *layout and surface shape* (files,
facade, manifests, test names) against the paper. Conformance of `main`'s
code to every adjudicated invariant (§4–§16, §21–§74) is enforced there by
the executable architecture/conformance suites and is not re-verified here.
