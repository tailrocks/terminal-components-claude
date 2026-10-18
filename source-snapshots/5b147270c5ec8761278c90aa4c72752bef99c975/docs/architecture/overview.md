# Termrock architecture overview

This is the target architecture for the in-place refactor of this repository.
It describes the future Termrock library; it does not claim the current Rust
tree already implements these boundaries.

## Project and authority

Execution follows a canonical two-stage model:

1. **Stage A (Preparation on `termrock-refactor`):** starts from the immutable
   `visual-baseline` commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Focuses
   on snapshot, interaction, verification, and CI preparation without refactoring
   production behavior (`src/**` strictly read-only).
2. **Stage B (Implementation on `termrock-implementation`):** created only from
   the accepted Stage A commit. The Rust Termrock library refactor is executed
   exclusively on this branch. This repository remains the project; no separate
   repository is created. The library/crate identity becomes Termrock during
   implementation.

`showcase`, `tablepro`, `jackin-preview`, and `holla` remain reference
consumers and conformance fixtures. Their source, scenarios, observable
interaction, and approved snapshots are the frozen visual target. Future
library changes must preserve their output 1:1 unless a separate product
change is approved. Rather than deferring application adoption to the end,
candidate components are integrated incrementally through bounded consumer-adoption
checkpoints at each phase.

Authority is divided by contract: this page owns the high-level architecture
map; [API](../api/README.md) owns public signatures and caller rules;
[foundations](../foundations/README.md) own shared mechanisms;
[components](../components/README.md) own component behavior;
[design contracts](../design/visual-contract.md) own the visual and interaction
rules; [verification](../verification/README.md) owns oracle and parity
procedure; [implementation](../implementation/plan.md) owns phase order.
Tasks link to those owners and do not redefine them.

## Target shape

Termrock is a reusable Rust TUI component library. Applications own domain
models and durable component state, provide short-lived borrowed props, and
handle typed actions. Termrock supplies shared component behavior, measurement
and rendering APIs, semantic styling, and runtime interaction machinery.

Stateful components use distinct `update(...)`, `measure(...)`, and
`draw(...)` phases. Update consumes normalized input and changes caller-owned
semantic state; measure reports size from explicit constraints; draw reads
state and borrowed props and publishes paint and geometry. Draw does not
commit values, execute commands, or mutate semantic state. Stateless
components do not need artificial empty state types. Exact signatures and
exceptions belong to the [public API contract](../api/public-api.md).

The caller owns committed values, persistence, eligibility, and external
services. Component state holds only durable interaction data such as a draft,
cursor, selection anchor, or scroll position. The runtime owns live focus,
hover, hit regions, pointer capture, layers, geometry publication, and time.
Stable semantic IDs, item keys, column keys, and source revisions preserve
meaning when data reorders or changes. Shared ownership details are in the
[foundation contracts](../foundations/README.md).

## Package and dependency boundary

The workspace uses the multi-crate layout in [CRATES.md](../../CRATES.md):
Termrock library crates under `crates/termrock-*` with the `termrock` facade,
several functionality crates per preview application, and verification crates.
This supersedes the earlier one-public-library-crate limit.
The existing four applications remain consumers in this same project.
Keep reusable production code in the library crates.
Crate counts never prove reusable component adoption; the
[component-only contract](component-composition.md) owns that rule.

Keep the production dependency graph small: Ratatui core types, Unicode width
and grapheme handling, a reviewed zeroization primitive for secret buffers,
and a small flags helper only when a real contract requires it. CLI and other
application-only dependencies do not enter the public library graph. A
terminal session/backend adapter is optional. PTY, `tui-snap`, raster, font,
report, and conformance-manifest tooling stay outside the production library
dependency graph. The frozen baseline harness may keep its existing test-only
dependencies while it remains the trusted oracle. Exact package wiring is a
future implementation decision within this repository; it does not create a
second repository or project.

The current baseline still contains names such as `junie_tui`, `WidgetId`,
`Outcome`, and `RenderCtx`. These are current or legacy implementation names,
not future Termrock API or compatibility requirements.

## Shared mechanism ownership

Each shared behavior has one contract owner. Components use that mechanism
and retain only their own visual and interaction policy.

| Concern | Owner |
| --- | --- |
| Public construction, props, state, `update`/`measure`/`draw`, typed actions | [Public API](../api/public-api.md) |
| IDs, stable keys, revisions, duplicate detection | [Identity](../foundations/identity.md) |
| Input normalization, bindings, event flow, responses | [Input and actions](../foundations/input-actions.md) |
| Focus, hit testing, hover, capture, feedback, time | [Runtime](../foundations/runtime.md) |
| Overlay order, anchoring, dismissal, focus trapping and restoration | [Layers](../foundations/layers.md) |
| Constraints, measurement, allocation, clipping, surfaces | [Layout](../foundations/layout.md) |
| Semantic roles, recipes, capability conversion, patches | [Theme](../foundations/theme.md) |
| Unicode editing, projection, selection, cursor mapping | [Text](../foundations/text.md) |
| Keyed models, presenters, ordering, reconciliation | [Collections](../foundations/collections.md) |
| Secret storage, masking, validation, redaction | [Secrets and validation](../foundations/secret-validation.md) |
| Scroll position, thumb behavior, shared fades | [ScrollRegion](../components/scroll-region.md) |
| Extension parts, slots, row/cell authorship | [Component authoring](../foundations/author.md) |
| Oracle lanes, exact comparison, negative gates | [Verification](../verification/README.md) |
| Optional terminal session setup and cleanup | [Session edge](../foundations/session.md) |

No component may create a parallel focus, capture, geometry, layer, text,
scroll, theme, identity, or terminal-session mechanism.

## Consolidated public surfaces

The architecture reduces engines while preserving distinct baseline behavior:

| Shared mechanism | Public surfaces |
| --- | --- |
| One Grid engine | table-row mode and cell/grid mode |
| One Menu engine | `Menu`, `ContextMenu`, `MenuBar` |
| One Picker mechanism | `Picker`, `CommandPalette`, chained selection |
| One text editing core | `TextInput`, `TextArea`, `CodeEditor` |
| One ScrollRegion mechanism | scrolling, thumb capture, shared fades |
| Panel plus TextViewport | framed viewport behavior without a separate ScrollPanel engine |
| StatusBar | segments-style presentation |
| DerivedHintBar | binding metadata adapter, not another painter |

Sharing an engine does not flatten component behavior. TextInput and TextArea
keep their different Escape and paste policies; table-row and cell/grid modes
keep their own markers and interactions; menu surfaces keep distinct
placement and key grammar. Component contracts preserve these differences.

## TerminalView boundary

`TerminalView` presents caller-provided prepared cells, including styles,
continuation cells, cursor, and selection. It emits typed interaction, copy,
or link requests. Termrock does not become a terminal emulator, PTY runtime,
shell, daemon, agent-session manager, or terminal escape parser. Such systems
remain external; the optional [session adapter](../foundations/session.md)
handles only terminal setup, input normalization, presentation, and cleanup.

## Compatibility and implementation

The future Termrock default theme must reproduce the frozen applications'
semantic surfaces, text hierarchy, accents, focus gutter, selected/current
markers, hover lift, disabled/read-only/error/loading/empty/success states,
modal backdrops, scrollbars, fades, cursor rules, narrow geometry, capability
modes, and motion policy. The visual details live in the
[visual contract](../design/visual-contract.md) and component-specific
contracts. Identity changes do not authorize restyling.

Implementation proceeds in the P0–P7 sequence in the
[implementation plan](../implementation/plan.md) exclusively on
`termrock-implementation`. Approved baseline output is immutable. Verification
keeps `ExistingOracle`, `ExtractedOracle`, and `Extension` lanes separate, and
candidate code cannot bless its own expected output. Consumer adoption occurs
incrementally at each phase (P2 basic controls/chrome, P3 fields/editing/forms,
P4 menus/dialogs/pickers, P5 grid/output, P6 terminal view, P7 full audit) into
the four applications (`showcase`, `tablepro`, `jackin-preview`, `holla`),
ensuring candidate components are validated in live compositions while keeping
scenarios and visual output invariant. Exact captures and applicable interaction
states are specified in [visual parity](../verification/visual-parity.md),
[interaction parity](../verification/interaction-parity.md), and
[conformance](../verification/conformance.md).
