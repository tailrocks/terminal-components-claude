# Termrock library specification

**Library-only revision · 27 September 2026**  
**Destination:** proposed repository `tailrocks/termrock-new`; public crate name `termrock`.  
**Status:** proposed contracts and reference pack, not an implementation or parity certificate.

## 1. Goal and hard scope boundary

Build a small, reusable Rust TUI component library that can reproduce the appearance and observable interaction of the pinned visual baseline while eventually supplying every generic visual building block needed by Jackin.

**Do not build Jackin, jackin-new, a new Jackin preview, a migration adapter to live services, or a replacement backend in this milestone.** Jackin provides requirements and test fixtures only. A test consumer may compose the library into a recognizable baseline frame, but it must contain inert data and generic event scripts, not workspace/auth/launch/daemon business logic.

Deliver the library, a tiny generic component laboratory, headless interaction tests, a `tui-snap` conformance harness, and public API examples. No web catalog, Studio, plugin registry, component installer, custom rasterizer, CLI product, terminal emulator, SQL workbench or general application framework.

Minimality means one implementation per behavior, not removing visual states or interactions. A widget is incomplete if it paints correctly but does not focus, scroll, edit, activate, disable, reconcile, or handle its applicable pointer/keyboard states correctly.

This document **supersedes the earlier pack's real-Jackin integration stages for the present milestone**. Preserve those as future product work; they are not prerequisites for accepting this library.

## 2. Sources and precedence

| Responsibility | Immutable reference |
|---|---|
| Visual output and baseline preview interactions | `donbeave/terminal-components-claude@4a79c0a2d40fca46fc406b77157ce3b3f12ec16b` |
| API/ownership concepts | Refactoring documents at that same revision; inspected main facade at `f758dc3f88196c3b93d70994dc7095edbfce1c93` |
| Generic capability requirements | Previously inspected `jackin-project/jackin@07e290f894eccfab785c698aafbaeeaa6e1c092d`, not a product implementation target |
| Original baseline renderer | `tui-snap@2d43458ad2bc37d76653c22d56e61ee74512d893` |
| Separately qualified toolkit candidate | `tailrocks/tui-snap@9dc86daff1dcbf20805b145916e8f04e9515f929` |

The annotated visual-baseline tag object is `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`. Pin the commit, never a moving branch or an old SHA embedded inside historical audit prose. The previous Termrock implementation is optional source-reading material, not the new project's architecture authority.

The approved source output wins over aesthetic simplifications. Refactoring concepts control ownership, not permission to change the appearance. Where source behavior, tests and prose disagree, record the exact event path and decide the narrow discrepancy before approving that case; do not silently select whichever result the candidate already produces.

Three evidence lanes must remain distinct:

1. **Existing oracle:** an approved artifact from the pinned source tree.
2. **Extracted oracle:** a newly isolated component/state captured from unchanged pinned rendering and event code by a trusted adapter.
3. **Extension:** new robustness, adapter or theme behavior with no old equivalent. It is specified and reviewed, not attributed to the old tag.

New public type names and signatures in this pack are proposals. Reference paths point to existing code; a new type need not exist there under the same name. See [the source manifest](reference/sources.lock.json).

## 3. Exact deliverable inventory

The [component index](COMPONENTS.md) contains **45 separately documented component/API surfaces**. Thin wrappers deliberately share implementations: CommandPalette uses Picker; ContextMenu/MenuBar use Menu; PropsList uses Props and shared collection behavior. TerminalView is an explicitly new borrowed-cell adapter boundary. This is not a claim that the baseline had 45 independent widget modules.

The [foundation index](FOUNDATIONS.md) contains **12 shared contracts**, covering identities, input/responses/keymaps, runtime/focus/capture/time, layers, layout, themes, text, collections, secrets/validation, component authorship, conformance and optional session integration.

The prior 54-family ledger remains fully traceable in [family-disposition.json](reference/family-disposition.json). Infrastructure rows are not converted into fake screenshot widgets; consolidation rows do not become duplicate components. The old `op_flow.rs` reference is corrected to the baseline's actual `screens/modals.rs`, and Meter's primary baseline implementation is `src/widgets/progress.rs`.

## 4. Public API architecture

### 4.1 State, props and phases

Use ephemeral `X<'a>` props that borrow data, and caller-owned `XState` only when durable interaction state exists. Stateless controls do not gain an empty state type merely for symmetry. States contain no terminal rectangles, colors, callback closures, domain objects, live service clients or borrowed model data.

Normal state types should support Debug/Default and useful equality/clone where safe. Secret-bearing state is intentionally not Clone, ordinary Debug, or serializable. A universal derive rule must not leak credentials.

For a stateful component, use:

```rust
update(&self, cx: &mut Cx<'_>, state: &mut XState) -> Response<XAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &XState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

These are **signature declarations**, not an implemented trait. Do not create a universal boxed Widget trait, a render-and-update `show()` API or an untyped event bus. Read-only model inputs and state-dependent measurement are explicit extra parameters where necessary; Grid's read-only/editable entries are documented separately.

`draw` may write pixels/cells, current geometry, cursor requests and layout facts through Ui. It must not commit fields, validate by mutating domain data, select an item, close a popup, execute a command, read a clock/environment variable or do IO. Update owns semantic changes. Repeated draw must be observably idempotent.

Immutable signatures are necessary but not sufficient: side effects can hide behind interior mutability or callbacks. Enforce purity with constrained author APIs, code review, state-before/after tests and no hidden RefCell/Cell/Mutex escape in component draw state.

### 4.2 One authority for each concern

| Concern | Owner |
|---|---|
| Domain rows, committed values, eligibility | Caller |
| Drafts, navigation cursor, reading anchor | Caller-owned component state |
| Focus, hover, capture, press and feedback | Runtime |
| Geometry, hit regions, layers and cursor arbitration | Runtime/Ui frame publication |
| Color roles, glyphs, dimensions, recipe defaults | Theme |
| Semantic action result | `Response<A>` |
| Filesystem, providers, PTYs, clipboard emission, persistence | Future application/edge adapter |

The runtime uses last published valid geometry for events. A resize or topology change invalidates geometry and requires republication before coordinate-sensitive dispatch. Do not inspect a separate application tree or duplicate `owns()/locate()` chains in every screen.

### 4.3 Controlled values and reconciliation

Checkbox/radio/select values and collection selection stay caller-controlled. Widget state holds a cursor/anchor, not a hidden replacement copy of the selected domain entity. Source changes reconcile by stable key, never by row label or current display index.

Each dynamic source has a revision. Surviving identity survives reorder/filter/refresh. A removed cursor can choose a documented fallback for navigation, but a captured action, acknowledgement target or draft never silently retargets to that fallback.

Text fields own temporary drafts. Commit transfers the new value to the caller; cancel restores the starting value according to that component's baseline policy. External changes during editing require a revision-aware conflict policy: preserve active draft and report conflict until caller chooses accept/reload, rather than silently overwriting either version.

### 4.4 Public facade and custom components

Expose standard components and core vocabulary from `termrock`; expose constrained advanced composition from `termrock::author`, theme types from `termrock::theme`, and pure helpers from `termrock::layout`. Keep raw focus/hit registries and layer internals private. No legacy `junie_tui`, WidgetId/Outcome/RenderCtx aliases are required in a fresh crate.

Component construction uses builders and borrowed model/painter inputs. Shared `.patch`, `.patch_part` and named-part slots replace app-local repainting. Slot callbacks draw only within their assigned rectangle and inherit surface/override rules. Do not call a standard widget then paint over it to imitate the old output.

## 5. Composition and consolidation rules

One Grid supports table-row and data-grid presentation. One Menu engine serves Menu, ContextMenu and MenuBar. One Picker engine serves Picker/CommandPalette and chained journeys. One TextEditorCore backs TextInput/TextArea/CodeEditor. One ScrollRegion provides scroll bounds, thumb capture and edge fades. Panel + TextViewport replaces ScrollPanel; StatusBar absorbs legacy segments. DerivedHintBar is a metadata adapter, not a second painter.

Preserve distinct visual recipes even when mechanisms merge. Consolidation is not permission to make all rows look the same, remove cell selection, remove command metadata or flatten a tab into a generic button.

Product-specific composites are not public widgets named AccountCard, WorkspacePicker, LaunchCockpit, Capsule, OnePasswordPicker or JackinShell. They are generic combinations with typed caller data. Their future business logic is explicitly outside this repository.

TerminalView borrows prepared terminal cells and returns typed input/copy/link requests. It is not a TextViewport pretending to be a terminal and not a new terminal emulator. A future termpane/Jackin adapter is deferred; conformance uses synthetic borrowed cells now.

## 6. Baseline interaction grammar

Focus, current/selected value, hover, held press, completed activation feedback and editing are separate axes. Standard focused controls show the baseline gutter; hover lifts the correct surface; disabled controls do not hover/activate; selection markers do not collide with the gutter. Preserve full focus+hover combinations.

A pointer gesture is down -> possible drag -> up -> conditional activation. Down alone is not click. Release outside, deleted target, ineligible target and click-through overlays must not produce a business action. Keyboard input suppresses stale hover until real pointer motion.

The baseline TextInput and TextArea differ: single-line Escape rolls back, multiline Escape commits. Single-line paste can enter editing; the tagged TextArea handler requires editing already active. Shared text infrastructure must not erase those policies. Form submit validation remains separate from field navigation/commit.

The precise keys and source-specific modifier exceptions live in per-component references. Binding remaps must propagate to hints, menus and help. No generic y/n confirmation handler may run before an editing field or higher modal consumes text.

## 7. Rendering, capabilities and motion

Default Theme::termrock preserves the baseline token values, glyph vocabulary, spacing, border hierarchy and narrow-width decisions. Never restyle from screenshots by eye when source recipes and cell snapshots exist.

Test truecolor, 256,16, explicit none and environment NO_COLOR separately. Paper/custom themes prove theme independence but are extension cases. Font/profile policy belongs to the conformance toolkit; the shipped library is not a font renderer.

Animations accept supplied phase/time and motion policy. Spinner has ten baseline glyphs; its pure tick-index function does not imply one global wall-clock cadence. Progress and meters have different semantic color roles. Determinate progress rounding, percentage/suffix columns, six-cell track fallback and Meter's 59/84 consumption thresholds have explicit boundary cases.

Paused/reduced visual motion must not freeze externally supplied data updates. A loading indicator may stop animating while completed data still changes the scene. Test full unique animation cycles plus transition boundaries, not only initial/final pictures.

## 8. Snapshot and interaction proof

[VISUAL-VERIFICATION.md](VISUAL-VERIFICATION.md) defines the required procedure. Use both pure production-view and actual executable/PTY paths in tui-snap. The executable here is the generic component laboratory, not Jackin.

Compare exact terminal dimensions, symbols/continuation cells, supported colors/modifiers, cursor state, and semantic observations (focus, capture, keys, action count, committed/draft values). With an identical qualified renderer/profile, require zero decoded-pixel differences. Also compare the grouped ANSI/TXT/HTML representations supported by the pinned tool.

Do not claim tui-snap preserves every terminal attribute. Its inspected schema normalizes some attributes; supplement uncovered behavior with direct cell/model/protocol assertions and disclose those limits. A style-normalized screenshot cannot prove a dropped attribute.

Original approved snapshots remain immutable. Tool/parser/font updates are qualified independently before comparing candidate widgets. New isolated states are captured from the unchanged source by a trusted oracle adapter; never approved from the candidate. Missing expected output fails closed.

Each component has a capture-plan file. These are requirements, not captured frames. No `.png`, `.ansi`, `.frame.json` or acceptance result has been fabricated in this deliverable.

## 9. Repository shape and implementation quality

```text
termrock-new/
  Cargo.toml                  # virtual workspace; default member = library
  Cargo.lock
  mise.toml                   # one local/CI entry point
  AGENTS.md                   # concise rules; no repetitive agent diaries
  crates/
    termrock/
      src/lib.rs
      src/{identity,event,response,runtime,layout,theme,text,collection}/
      src/components/<component>.rs
      src/components/<component>/tests.rs
      src/author.rs
      src/session/            # optional crossterm feature
      tests/public_api/
    termrock-conformance/     # publish=false
      src/fixtures/
      src/bin/lab.rs          # tiny generic consumer
      tests/
  reference/sources.lock.json
  reference/components.json
  reference/capture-plans/
  snapshots/approved/         # trusted imported/extracted oracle artifacts
```

Start with one public library crate and one nonpublished conformance crate. Split production crates only for measured compile/dependency benefit; never one crate per widget. Do not import an entire prior workspace.

Use a verified stable Rust toolchain pinned through Mise. Normal core dependencies should be limited to Ratatui core types, Unicode measurement/segmentation, a reviewed zeroization primitive for secrets, and a small flags helper only where justified. Crossterm/session integration is optional. tui-snap/PTY/raster/font/report dependencies belong solely to conformance.

Tests live in separate files. For the new implementation propose a 400-line maximum per production Rust file, excluding generated source; split on cohesive responsibilities, not one-method traits. Reuse existing repository governance rather than adding a custom agent platform. Missing docs/broken links, unsafe production code, TODO production paths, broad lint exemptions and unchecked value/geometry arithmetic are blocking findings.

Proposed review-lane CI target: two minutes, measured on a declared runner; this is a performance goal, not a demonstrated property. Publish cold/warm compile and shard durations. Shard/prebuild/reuse immutable artifacts rather than skip required states or relax gates. This is a target, not a measured result of this specification task.

## 10. Acceptance and stop line

The milestone ends when every included component and shared helper has an implemented public API, passing behavior tests, exact applicable baseline snapshots, validated extensions, external-consumer examples and independent review. Generic composed fixtures must use those same public components without private imports or replacement painters.

No real account login, workspace save, Docker launch, terminal daemon or Jackin CLI parity is required to finish this milestone. Those are future product integration work. Conversely, a static component gallery does not finish the library: all applicable state transitions, pointer/keyboard paths and source-change/resize behavior must pass.

See [IMPLEMENTATION-PLAN.md](IMPLEMENTATION-PLAN.md) and [REVIEW.md](REVIEW.md). The current package provides the specification and reference inventory only. API compilation, oracle capture, Rust tests, runtime parity and independent-agent review remain unexecuted.
