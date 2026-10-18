# Architecture

## Status

Target architecture. Current implementation is incomplete and not ready.

## Authority

`main` is the structural reference. Keep reusable behavior in focused Rust crates with explicit public APIs; keep application-specific composition and domain behavior in application crates. The branch `design/interaction-polish` is a product reference, not an architectural template. It may contain better API ideas; adapt those ideas to the split crate structure.

## Current evidence

The root workspace manifest splits the shared Termrock library into foundation, layout, theme, rendering, runtime, controls, fields, navigation, viewport, overlays, grid, editors, forms, terminal, and session crates. Showcase, Jackin Preview, Holla, and TablePro each have app-specific crates for domain and presentation concerns. The manifests prove these boundaries are declared; they do not prove the refactor works correctly or that public APIs are complete.

## Remaining work

- Complete and repair the in-progress crate migration. Treat it as unfinished and not ready.
- Make reusable APIs cohesive, documented, and usable by independent application crates through declared public crate surfaces.
- Keep shared UI behavior, state ownership, identity, theming, and interaction reusable; keep app workflows and domain vocabulary in application crates.
- Support composition and local customization where required by the accepted design/API reference.
- Preserve explicit dependency direction. Retire legacy paths only through an intentional migration.
- Review design-branch API ideas individually, then implement accepted ideas within the main branch's crate structure.
- Preserve all useful planned work while reconciling plans with the current source and visual reference.

### Carried-forward contracts from the former architecture record

The former `COMPONENT_ARCHITECTURE.md` at immutable commit `d1122f69` records decisions and evidence through §75. Its history is not current authority. The table disposes every late section, retaining a section-specific source link for contracts that need current verification.

| Source section(s) | Subject | Disposition |
|---|---|---|
| §§21–24 | Slice 2 review; modern API/dependency policy; Form and Grid bounds; re-exports and field vocabulary | Candidate API contracts. Verify against current public crate surfaces and consumer needs before adoption. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §25 | Slice 3 foundations review | Historical decisions; verify each surviving behavior against current implementation and design reference. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §§26–29 | Layer sizing, Measure access, foundations followups, prototypes, residuals | Candidate layout and API requirements; current APIs and acceptance remain unverified. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §§30–35 | ChipBar checked state, mono fallback, style parts, capability detection, scrolling | Candidate rendering and API contracts; no old package paths or review claims establish current readiness. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §§36–40 | Digest accounting, gates, readiness evidence, runtime forcing, historical corrections | Evidence and gate history only. Re-run relevant checks against current source before readiness claims. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §§41–45 | Example/API gaps, Slice 8 scope, test disposition, override surface | Candidate API/conformance requirements; verify current examples, consumer paths, and declared checks. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §§46–49 | Slice 5 migration risks, gates, identity accounting, baseline policy | Migration history only. The current crate split exists, but its implementation remains incomplete and not ready; use current plan/status evidence. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §50 | Choice identity and semantic state | Candidate behavior contract; verify against current controls and the accepted visual/interaction reference. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §51 | Keyed StatusBar hover | Candidate state/identity contract; verify hover ownership and stable keys in current feedback crate. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §52 | Grid actions, state, sorting ownership | Candidate API contract. Current Grid emits keyed `Sort(ColumnKey, SortDir)`; adapter ordering and exact reader surface need current API review. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §53 | Tree keyed incremental index | Candidate performance/state contract; validate cache invalidation and key reconciliation in current navigation crate. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §54 | Jackin status, update causes, deadlines, dimming, status projection | Candidate runtime/product contract. Current runtime exposes `UpdateCause`, bootstrap, repaint deadlines, and semantic dimming; Jackin behavior and full acceptance still require verification. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §§55–56 | Closure traversal; SplitPane ownership | Candidate container contracts; verify exactly-once traversal and two-slot geometry in current crates. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §§57–59 | NavList choice/entry; Steps lifecycle, identity, cached frontier | Candidate interaction and performance contracts; verify against current navigation implementations. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §§60–61 | Tree chosen marker; Grid schema and ragged cells | Candidate rendering/data contracts; verify current semantic identity and structural cell behavior. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §§62–63 | TextViewport mono selection; TooSmall hierarchy | Candidate visual/layout contracts; compare current implementation with design reference at supported color levels and sizes. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §§64–66 | Conformance identity/gesture separation; local Grid readiness; isolated performance subjects | Candidate verification and performance contracts; no historical pass is current proof. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §67 | Slice 4F data, pointer, and form bridges | Migration-era API requirements; verify current crate bridges and remove obsolete implementation steps. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §§68–71 | Binding identity, focused hints, collection reveal/pressed ownership, mono resolution, Select glyph | Candidate API/rendering contracts; retain only after source and design verification. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §§72–73 | Exact-target reference rendering; disabled pointer, Props, inert registration | Candidate capture, state, and safety contracts; verify current capture tooling and registration behavior. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §74 | Slice 5 capture and baseline decisions | Historical migration procedure. Keep capture and reviewed-baseline requirements only where current plan requires them; no old slice status transfers. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |
| §75 | Recovery of absent historical renderings | Historical evidence-recovery record only; it does not define current product appearance or authorize baseline changes. [Immutable source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/COMPONENT_ARCHITECTURE.md). |

The detailed interaction, visual, and color acceptance ledger is maintained from the design branch in [the planned-work index](plans/design-work-index.md). Current branch comparisons and source-revision evidence are tracked in [branch authority comparison](verification/branch-authority-comparison.md).

### Former refactoring goal

`REFACTORING_GOAL.md` is historical input. Its actionable product journeys and quality gates are candidates only; validate them against current source and the design reference before treating them as required. Its old package names, staffing, sequencing, and readiness statements have no authority. This document does not claim full preservation of that file's contents.

## Verification

Verify crate boundaries from manifests and imports. Verify APIs with independent consumer builds and focused tests. Verify user-visible behavior against the accepted design reference. Passing compilation or tests alone does not establish completion.
