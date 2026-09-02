You are a principal product researcher, terminal UX architect, information architect, interaction-system analyst, and senior Rust/Ratatui engineer.

Your mission is to perform a COMPLETE, SOURCE-BACKED ANALYSIS of the current jackin❯ product and produce ONE canonical Markdown reference describing every operator-visible terminal interface, screen, tab, pane, overlay, dialog, picker, popup, workflow, state, action, data presentation, and transition currently implemented by Jackin.

The result will be used by a later `/goal` to build a completely redesigned Jackin TUI inside the approved Junie-inspired Ratatui design-system repository.

This phase produces REFERENCE DOCUMENTATION ONLY.

DO NOT implement the redesigned Jackin application.

DO NOT recreate the current Jackin UI.

DO NOT copy Jackin rendering code.

DO NOT extract Jackin widgets into another repository.

DO NOT build a current-Jackin reference application.

DO NOT create screenshots or a visual-reference project.

DO NOT modify Jackin source code.

DO NOT modify the existing Junie design system, showcase, TablePro application, `DESIGN.md`, themes, components, visual baselines, or runtime.

Analyze the REAL Jackin source and point the future implementation agent directly to that source.

---

# 1. INPUTS

## Current Jackin product

Repository:

`/Users/donbeave/junie-style-2/jackin`

Use the local checkout's current `main` branch, but pin the analysis to the exact commit SHA present when this task starts.

Treat the local Jackin checkout and its checked-in source, tests, fixtures, snapshots, and documentation as the complete reference. Do not use websites or ask the user to open or use one.

Do not alter the Jackin checkout.

## Future implementation repository

The later redesign will be implemented in:

`/Users/donbeave/junie-style-2/terminal-components-claude`

This repository already contains:

- the approved Junie-inspired TUI design system
- its reusable Rust/Ratatui components
- its canonical `DESIGN.md`
- its showcase application
- the TablePro application
- the interaction system
- focus management
- keyboard and mouse handling
- hover
- selection
- editing
- scrolling
- overlays
- dialogs
- tables
- grids
- pickers
- responsive layouts
- visual-review infrastructure

Do not change any of those during this phase.

## Methodology references

The operator has supplied two Junie prompts:

- `JUNIE_PROMPT1.md`: creation of the approved design-system laboratory
- `JUNIE_PROMPT2.md`: creation of TablePro as a real application using that design system

Use them only to understand what information the future Jackin implementation `/goal` will require.

In particular, understand the established process:

1. research the real product deeply
2. understand its workflows and data
3. preserve the approved design system
4. translate product semantics rather than copying old layouts
5. build one coherent interactive application
6. use deterministic fixture data where backend implementation is unnecessary
7. run, interact with, capture, inspect, critique, and correct the final design

DO NOT execute that implementation process now.

This phase prepares its product reference.

---

# 2. SINGLE REQUIRED OUTPUT

Create exactly one canonical file:

`/Users/donbeave/junie-style-2/terminal-components-claude/JACKIN_REFERENCE.md`

This file must be titled:

# Jackin Current Product, Interface, and Workflow Reference

This document will be the canonical source of truth for:

- the current Jackin product model
- current operator-visible capabilities
- current interface inventory
- current workflow inventory
- redesign scope
- the mapping from each interface to its real Jackin implementation

The pinned Jackin source remains the authoritative evidence for implementation details.

`JACKIN_REFERENCE.md` is the authoritative navigation, experience, and redesign-scope map that tells the future agent:

- what exists
- what it means
- how it behaves
- what data it presents
- how users reach it
- what other surfaces it can open
- what states it supports
- which exact source files and symbols explain it

The future agent should not need to rediscover the repository structure.

It should be able to read `JACKIN_REFERENCE.md`, select a feature or screen, and open only the specifically cited source ranges.

Do not create multiple reference files.

Temporary notes or scripts may be used during analysis, but remove them before completion.

The final Git diff in `terminal-components-claude` must contain only:

`JACKIN_REFERENCE.md`

---

# 3. ABSOLUTE BOUNDARIES

This is PRODUCT ARCHAEOLOGY, not implementation.

Do not:

- create a Rust binary
- add a Cargo target
- add fixtures to the Junie project
- recreate current Jackin screens
- reproduce Jackin widgets
- copy rendering functions
- copy theme constants
- copy state types
- create PNG, ANSI, HTML, TXT, or cursor artifacts
- redesign navigation
- propose new screen layouts
- create a Junie mapping
- implement a Jackin preview
- modify the approved design system
- change TablePro
- change Jackin
- treat roadmap concepts as shipped features
- infer functionality merely because it would make sense

You may run the current Jackin application to verify behavior.

You may run existing tests and existing visual baselines.

You may inspect existing screenshots or rendered fixtures.

Those are evidence only.

The only deliverable is the Markdown reference.

---

# 4. PRIMARY RESEARCH QUESTION

The document must answer:

> What can an operator currently see and do across the complete Jackin terminal experience, how do those interfaces and workflows relate, what data is presented in each state, and where is every behavior implemented in the real source?

The analysis must cover the complete terminal journey, including where applicable:

- command entry
- host console
- workspace selection and management
- workspace details
- running instances
- workspace creation
- workspace editing
- global settings
- role configuration
- environment and secret configuration
- authentication configuration
- trust configuration
- mounts
- source and scope selection
- provider/account usage
- launch preparation
- launch progress
- launch errors
- transition into the construct
- capsule multiplexer
- agent sessions
- tabs
- panes
- splits
- zoom
- scrollback
- agent state
- usage inside the capsule
- branch and container context
- hardline/reconnection
- eject
- exile
- debug information
- confirmation
- cancellation
- help
- quit and shutdown behavior

Do not assume this list is exhaustive.

Discover the complete current set from source.

---

# 5. ANALYSIS SNAPSHOT

At the beginning, record:

- source repository
- source branch
- exact commit SHA
- source commit date
- Jackin version if discoverable
- analysis date
- local documentation revision where discoverable
- local checkout path used
- future implementation repository path

All source references in the final document must be pinned to this SHA.

Never link only to mutable `main` when an immutable commit link can be produced.

If source changes during analysis, continue against the original pinned SHA unless a serious source error requires restarting the snapshot.

---

# 6. SOURCE OF TRUTH AND EVIDENCE PRIORITY

Use this evidence priority; do not consult websites or remote documentation:

1. current source at the pinned SHA
2. current tests, snapshots, baseline registries, and fixtures
3. current runnable behavior
4. current repository documentation
5. plans, roadmap, or research documents

Plans, roadmap files, and research documents must never override shipped code.

For every important capability, classify the evidence as one or more of:

- `SOURCE_VERIFIED`
- `TEST_VERIFIED`
- `RUNTIME_VERIFIED`
- `DOC_VERIFIED`
- `PARTIALLY_IMPLEMENTED`
- `PLANNED_ONLY`
- `RESEARCH_ONLY`
- `INFERRED`
- `UNKNOWN`

When sources disagree:

- describe the disagreement
- identify which behavior is currently implemented
- do not silently reconcile it
- do not rewrite the product according to documentation intent

---

# 7. SYSTEMATIC COMPLETENESS METHOD

Do not rely on documentation navigation or filenames alone.

Build the surface inventory from the UNION of:

- top-level application stage and route enums
- screen enums
- tab enums
- modal enums
- dialog enums
- picker enums
- overlay enums
- launch-stage enums
- visible-state enums
- keymap action enums
- input-dispatch branches
- render-dispatch branches
- `render_*` functions
- `view` modules
- modal constructors and `open_*` call sites
- message enums
- effect enums
- subscription and background-event handlers
- focus targets
- hover targets
- mouse hit regions
- scroll registries
- tests
- snapshot tests
- PNG baseline registries
- documentation sections
- public CLI command entry points

Search for concepts and symbols containing terms such as:

- screen
- surface
- stage
- route
- view
- render
- tab
- pane
- modal
- dialog
- popup
- overlay
- picker
- prompt
- menu
- footer
- status
- progress
- error
- help
- usage
- launch
- console
- capsule

Do not stop after finding the obvious screens.

At the end, every discovered visual variant must be:

- documented
- explicitly classified as not operator-visible
- or listed as an unresolved discrepancy

No visual enum variant may remain unaccounted for.

---

# 8. STARTING SOURCE MAP

Use these paths as starting points, not as an exhaustive list.

Follow imports and call chains wherever necessary.

## Host console

- `crates/jackin/src/console/adapter/run.rs`
- `crates/jackin-console/src/tui.rs`
- `crates/jackin-console/src/tui/`
- `crates/jackin-console/src/tui/model.rs`
- `crates/jackin-console/src/tui/model/`
- `crates/jackin-console/src/tui/state.rs`
- `crates/jackin-console/src/tui/state/`
- `crates/jackin-console/src/tui/update.rs`
- `crates/jackin-console/src/tui/input/`
- `crates/jackin-console/src/tui/keymap.rs`
- `crates/jackin-console/src/tui/focus.rs`
- `crates/jackin-console/src/tui/layout.rs`
- `crates/jackin-console/src/tui/view/`
- `crates/jackin-console/src/tui/components/`

## Console screens

- `crates/jackin-console/src/tui/screens/workspaces/`
- `crates/jackin-console/src/tui/screens/editor/`
- `crates/jackin-console/src/tui/screens/settings/`
- `crates/jackin-console/src/tui/screens/usage.rs`
- `crates/jackin-console/src/tui/screens/usage/`
- `crates/jackin-console/src/tui/screens/edit_save/`

## Console modal system

- `crates/jackin-console/src/tui/model/modal.rs`
- `crates/jackin-console/src/tui/model/modal/`
- modal-related components under `crates/jackin-console/src/tui/components/`
- all modal constructor and `open_*` call sites
- all concrete confirmation targets
- all concrete text-input targets
- all picker targets and stages

## Launch cockpit

- `crates/jackin-launch/src/tui/`
- `crates/jackin-launch/src/tui/components/`
- launch model/state types re-exported from `jackin-core`
- launch progress and orchestration call sites
- launch tests and baselines

## Capsule / in-construct multiplexer

- `crates/jackin-capsule/src/tui/`
- `crates/jackin-capsule/src/tui/components/`
- `crates/jackin-capsule/src/tui/daemon/`
- `crates/jackin-capsule/src/tui/input/`
- `crates/jackin-capsule/src/tui/keymap/`
- `crates/jackin-capsule/src/tui/layout/`
- session and control-plane protocol types
- capsule tests and baselines

## Usage system

- `crates/jackin-console/src/tui/screens/usage.rs`
- `crates/jackin-usage/src/`
- `crates/jackin-usage/src/usage.rs`
- every provider module under `crates/jackin-usage/src/usage/`
- `crates/jackin-usage/src/host/`
- `crates/jackin-protocol/src/usage_broker.rs`
- usage-related types in `crates/jackin-protocol/src/control.rs`
- capsule usage dialog and status-bar integration
- console adapter projection wiring
- usage tests and end-to-end tests

## Product and UI documentation

- `docs/content/reference/tui/`
- `docs/content/reference/capsule/`
- relevant command documentation
- relevant concept documentation
- relevant workspace, role, auth, usage, launch, hardline, eject, and exile documentation

Follow all additional relevant paths discovered during research.

---

# 9. USE SUBAGENTS AGGRESSIVELY

Parallelize read-only investigation.

Each subagent must return structured findings with exact source paths, symbols, and line ranges.

At minimum delegate:

## A. Product model and vocabulary

Determine:

- what Jackin is
- who the Operator is
- what a Construct is
- what a Workspace is
- what a Role is
- what an Agent/runtime is
- what an instance/session is
- what mounts, environments, auth, trust, and providers mean
- how concepts relate
- what persists
- what is global
- what is workspace-specific
- what is role-specific
- what is agent-specific
- what is session-specific

## B. Host-console topology

Map all host-console routes, stages, screens, persistent regions, and transitions.

## C. Workspace manager

Map:

- workspace list
- grouping
- instance children
- selection
- detail pane
- actions
- create/edit/delete
- launch
- new-session behavior
- status
- scrolling
- overlays
- empty and error states

## D. Workspace editor

Map every current editor tab and every row, action, state, picker, and dialog.

Discover the exact current tabs from source.

Do not assume documentation is complete.

## E. Global settings

Map every settings tab, field, row, grouped editor, action, state, and dialog.

Discover exact tabs and behavior from source.

## F. Dialogs, overlays, and pickers

Enumerate:

- every generic modal variant
- every concrete target or invocation of each variant
- every trigger
- every stage
- every result
- every cancellation path
- every validation path
- every nested or stacked flow

A generic `Confirm` entry is not enough.

Document every distinct user-facing use of that confirmation component.

## G. Usage and provider data

Trace every supported provider, account shape, quota/window representation, status, freshness state, error, and display surface.

## H. Launch cockpit

Map identity, stages, transitions, status, progress, build log, failure, confirmation, container/debug information, cancellation, and success transition.

## I. Capsule multiplexer

Map tabs, panes, splits, agents, shell sessions, zoom, focus, scrollback, menus, status, usage, context bars, dialogs, pointer states, reconnection, and lifecycle actions.

## J. Interaction model

Map keyboard, mouse, focus, hover, selection, editing, scrolling, pointer shape, modal priority, cursor visibility, and help.

## K. Current visual semantics

Map colors, hierarchy, surfaces, borders, tabs, chrome, status, progress, modal treatment, focus and hover semantics.

Do not redesign them.

## L. Workflow synthesis

Build all meaningful end-to-end operator journeys, including alternate, cancellation, failure, empty, stale, and recovery paths.

## M. Completeness audit

Independently compare the finished document against:

- every discovered tab variant
- every modal variant
- every picker variant
- every launch stage
- every keymap context
- every visual baseline
- every usage provider
- every capsule dialog/action
- every documented command flow

The primary agent owns final synthesis and the single output file.

---

# 10. REQUIRED STRUCTURE OF `JACKIN_REFERENCE.md`

The document must contain the following major sections.

Use a linked table of contents and stable section IDs.

---

## 1. Document Contract

Explain:

- what the document covers
- what it does not cover
- the pinned source revision
- evidence classifications
- how the future redesign agent should use it
- that the document defines redesign scope
- that pinned source links provide implementation evidence

State explicitly:

> This document describes current product semantics and capabilities. It does not require the future design to preserve current layouts, styling, panels, colors, or navigation structure.

---

## 2. Snapshot and Coverage Summary

Record the snapshot metadata.

Include discovered counts such as:

- top-level terminal surfaces
- screen routes
- editor tabs
- settings tabs
- modal variants
- concrete modal flows
- picker types
- launch stages
- capsule dialog types
- keymap contexts
- usage providers
- end-to-end workflows
- visual baseline states

Do not invent counts.

Derive them from source and explain the counting method.

---

## 3. One-Page Product Explanation

Explain Jackin from the operator’s perspective.

Cover:

- core problem
- primary operator
- product promise
- isolation model
- relationship between host, Construct, Workspace, Role, Agent, and session
- primary lifecycle
- where configuration happens
- where work happens
- where usage information comes from
- how an operator returns to running work

Avoid implementation details unless needed to explain the product.

---

## 4. Canonical Vocabulary

Create a table:

| Term | Exact current meaning | Scope | Where visible | Source | Potential confusion |
|---|---|---|---|---|---|

Cover all product-specific terminology.

Examples to investigate include:

- Operator
- Construct
- Workspace
- Role
- Agent
- Runtime
- Session
- Instance
- Jacking in
- Hardline
- Eject
- Exile
- Mount
- Environment
- Auth
- Trust
- Provider
- Account
- Usage
- Capsule

Do not assume these are the complete terms.

Preserve exact current spelling and capitalization.

Distinguish:

- domain terms
- command names
- UI labels
- metaphors
- internal implementation names

---

## 5. Domain Model and Scope

Describe each UI-relevant entity.

For each entity record:

- identity
- fields visible to the operator
- relationships
- cardinality
- ownership
- scope
- lifecycle
- persistence
- valid states
- available actions
- error conditions
- source types
- code references

Include a relationship diagram.

Use Mermaid or compact ASCII, but keep a readable textual explanation as well.

Explicitly answer:

- What belongs to global settings?
- What belongs to a Workspace?
- What belongs to a Role?
- What belongs to an Agent?
- What belongs to an instance/session?
- What belongs to a provider account?
- What survives exit?
- What is created only for a running Construct?
- What can have multiple simultaneous instances?

---

## 6. Complete Application and Surface Topology

Create a top-level graph showing all current terminal surfaces and transitions.

Distinguish:

- full-screen routes
- embedded panes
- tabs
- inline pickers
- global overlays
- modal dialogs
- temporary status surfaces
- progress surfaces
- in-construct surfaces
- CLI-to-TUI transitions
- TUI-to-capsule transitions
- return/reconnect transitions

The graph must show the relationship between at least:

- host console
- workspace manager
- workspace editor
- settings
- Usage
- launch cockpit
- capsule multiplexer
- help/debug overlays
- lifecycle exits

Do not represent disconnected screen screenshots.

Represent the real application graph.

---

## 7. Surface Inventory

Create a compact summary table for every operator-visible surface.

Use stable IDs, for example:

- `CONSOLE-WORKSPACES`
- `EDITOR-GENERAL`
- `SETTINGS-TRUST`
- `MODAL-FILE-BROWSER`
- `LAUNCH-PROGRESS`
- `CAPSULE-USAGE`

Choose accurate names based on source.

Columns should include:

| ID | Surface | Type | Entry | Primary purpose | Main data | Opens | Source |
|---|---|---|---|---|---|---|---|

Every detailed section and workflow must use these IDs consistently.

---

# 11. REQUIRED PER-SURFACE SPECIFICATION

For every full screen, pane, tab, overlay, dialog, picker, popup, status surface, and progress surface, provide a detailed specification with this structure.

## `<ID> — <Surface name>`

### Classification

- type
- product area
- evidence class
- current/shipped status

### Operator purpose

What is the operator trying to understand or accomplish?

### Entry conditions

- how the surface is opened
- prerequisites
- source state
- commands or actions that reach it

### Exit and destination

- every normal exit
- cancellation
- confirmation
- error exit
- next surfaces

### Layout and composition

Describe the current structure:

- header
- identity/context
- navigation
- sidebar
- panes
- tabs
- content blocks
- tables/lists
- footer
- status bars
- overlays
- responsive changes

Do not merely write “two-pane layout.”

Explain what each region contains and why.

### Complete visible content

List all operator-visible:

- titles
- tabs
- sections
- labels
- rows
- fields
- values
- badges
- icons/glyphs
- help text
- counts
- status text
- footer hints
- buttons/actions
- empty messages
- error messages
- progress labels

For dynamic text, document its template and data source.

Preserve exact labels where practical.

### Data contract

For every rendered datum explain:

- Rust type
- field
- source subsystem
- transformation
- grouping
- ordering
- formatting
- fallback
- conditional visibility
- empty behavior
- stale behavior
- error behavior

The future implementation agent must understand what realistic deterministic fixture data is needed.

### Selection and identity

Explain the difference between:

- focused
- hovered
- selected
- active
- expanded
- current
- running
- edited
- dirty
- blocked
- errored

Only include states that apply.

### Keyboard interaction

List every active key by context.

For each key record:

- key
- guard/condition
- action
- state transition
- visible result
- source keymap/action symbol

No hidden active shortcut may be omitted.

### Mouse interaction

Document:

- hover targets
- click targets
- focus transfer
- selection
- pointer shape
- wheel behavior
- scrollbar behavior
- drag behavior
- double-click or second-click behavior where present
- modal input ownership

### Focus model

Explain:

- initial focus
- focus owners
- traversal
- internal composite-widget navigation
- modal trapping
- focus restoration
- visual focus indication

### Scrolling and overflow

Explain:

- axes
- keyboard bindings
- mouse-wheel behavior
- scroll focus
- scrollbar
- clamping
- truncation
- long-name handling
- resize behavior

### States and variants

Cover all meaningful states:

- default
- populated
- empty
- loading
- refreshing
- stale
- error
- validation error
- disabled
- unavailable
- selected
- hovered
- focused
- dirty
- saving
- success
- failure
- blocked
- confirmation
- reduced-size

### Opened surfaces

List every dialog, popup, picker, overlay, or route that this surface can open.

Link to its stable ID.

### State transitions

Use a compact table:

| Current state | Event | Guard | Next state | Visible result | Side effect | Source |
|---|---|---|---|---|---|---|

### Long-running work

Describe:

- operation
- visible owner
- intermediate state
- completion
- failure
- cancellation
- retry
- whether the UI remains interactive

### Current visual semantics

Describe how the current renderer communicates:

- hierarchy
- focus
- hover
- selection
- warning
- error
- progress
- context
- risk

Separate meaning from styling.

### Implementation map

Include a table such as:

| Responsibility | Repository path | Symbol | Exact lines | Why this matters |
|---|---|---|---|---|
| View/rendering | ... | ... | ... | ... |
| Model/state | ... | ... | ... | ... |
| Update | ... | ... | ... | ... |
| Messages/effects | ... | ... | ... | ... |
| Input/keymap | ... | ... | ... | ... |
| Mouse/hit testing | ... | ... | ... | ... |
| Focus/scroll | ... | ... | ... | ... |
| Data projection | ... | ... | ... | ... |
| Tests/baselines | ... | ... | ... | ... |
| Documentation | ... | ... | ... | ... |

Every path must use the local source checkout rooted at `/Users/donbeave/junie-style-2/jackin`, with the analyzed SHA recorded alongside it.

### Future redesign obligation

Do not propose a layout.

State only:

- what information must remain understandable
- what action must remain possible
- what state distinctions must survive
- what workflow continuity must be preserved
- what may be represented using deterministic simulation in the future preview

---

# 12. HOST CONSOLE COVERAGE

Document the complete host-console experience.

At minimum determine:

- startup route
- initial selection
- workspace list structure
- workspace grouping
- expanded/collapsed rows
- running-instance children
- detail presentation
- launch behavior
- new-session behavior
- reconnect behavior
- workspace creation
- workspace editing
- workspace deletion
- Settings entry
- Usage entry
- refresh
- help
- debug/container information
- quit
- empty state
- loading state
- errors
- long names
- many workspaces
- many instances
- horizontal and vertical scrolling
- mouse behavior
- focus transfer

Do not assume these names or actions are exact.

Verify every item and use source terminology.

Trace the top-level route/stage model, render dispatch, update dispatch, and adapter wiring.

---

# 13. WORKSPACE EDITOR COVERAGE

Discover every current workspace-editor tab from source.

For every tab, document:

- rows
- sections
- derived preview rows
- editable rows
- action rows
- default values
- inheritance
- override behavior
- add/remove actions
- validation
- saving
- dirty state
- save/discard/cancel flow
- picker entry
- text-entry flow
- keyboard
- mouse
- focus
- scrolling
- empty/error/loading states

Known starting concepts to verify include:

- General
- Mounts
- Roles
- Environments or secrets
- Auth

Do not rely on the labels above if source differs.

Specifically investigate:

## General

- all fields
- toggles
- workspace identity
- repository/workdir behavior
- derived values
- validation

## Mounts

- inherited/global mounts
- workspace mounts
- source
- destination
- read-only/writable
- isolation
- GitHub origin
- file browser
- mount-destination choice
- add/remove
- horizontal overflow

## Roles

- available roles
- allowed/default state
- role loading
- role input
- overrides
- expand/collapse
- unavailable/error states

## Environments and secrets

- scopes
- keys
- literal values
- source-backed values
- masking
- 1Password or other source flows
- add/edit/remove
- picker stages
- validation

## Auth

- modes
- role-specific overrides
- provider/agent relationship
- source selection
- folder/source preview
- auth form
- token generation
- save behavior
- errors

For every generic component, enumerate its concrete use.

---

# 14. GLOBAL SETTINGS COVERAGE

Discover every Settings tab from source.

For every tab, document all content and flows.

Known starting concepts to verify include:

- General
- Mounts
- Environments
- Auth
- Trust

Specifically document:

- global defaults
- workspace interaction
- inheritance
- preview-only rows
- grouped edit dialogs
- trust decisions
- allowed/blocked states
- mount defaults
- environment defaults
- authentication defaults
- dirty state
- save flow
- discard flow
- validation
- loading and token generation
- every modal opened by Settings

Clearly distinguish global Settings behavior from Workspace Editor behavior where both expose similar concepts.

---

# 15. COMPLETE MODAL, POPUP, OVERLAY, AND PICKER CATALOG

Enumerate every operator-visible modal family and every concrete flow that uses it.

The current generic console modal system contains multiple families. Use its actual current enum as a starting point and verify all variants.

Investigate concepts including:

- text input
- file browser
- mount destination choice
- workdir selection
- confirmation
- save/discard/cancel
- GitHub picker
- confirm save
- error popup
- container/debug information
- status popup
- 1Password item picker
- role picker
- role override picker
- auth role picker
- source picker
- auth source picker
- scope picker
- auth form

Do not stop at the generic modal names.

For each generic family:

1. find every concrete target variant
2. find every constructor
3. find every `open_*` call site
4. find every input branch
5. find every result branch
6. find every distinct title
7. find every distinct message
8. find every button set
9. find every validation rule
10. find every cancellation behavior
11. find every downstream transition

For example, a generic confirmation used for deleting a workspace and a generic confirmation used for deleting a secret are two distinct user workflows and must be documented separately.

Also catalog:

- keyboard help
- inline pickers
- loading/status overlays
- launch overlays
- capsule menus and dialogs
- nested/stacked modal behavior
- backdrop and focus behavior

Create a modal invocation matrix:

| Modal ID | Family | Opened from | Trigger | Data shown | Default focus | Actions | Result | Source |
|---|---|---|---|---|---|---|---|---|

---

# 16. USAGE SYSTEM — DEEP PROVIDER ANALYSIS

This section must be exceptionally detailed.

The future redesigned Jackin application must demonstrate realistic provider usage rather than generic progress bars.

Start from the current provider registry or equivalent source-of-truth list.

Do not hardcode providers from memory.

Enumerate every current provider/surface and every unsupported fallback.

For each provider document:

- internal provider ID
- display label
- related agent/runtime names
- account-discovery mechanism
- account identity
- optional username/email
- plan label
- credential origin
- data source
- whether data comes from CLI, local files, keychain, OAuth, RPC, REST, or another mechanism
- refresh behavior
- timeout/error behavior
- quota or usage windows
- bucket labels
- status-bar slots
- percentages
- used/remaining semantics
- monetary spend where supported
- reset time
- stale/fresh behavior
- confidence/source labels
- needs-login state
- needs-secret state
- unsupported state
- unavailable state
- provider-specific errors
- multiple-account behavior
- provider switching
- ordering
- normalization into the shared protocol

Trace the complete pipeline:

PROVIDER-SPECIFIC INPUT
→ PROVIDER ADAPTER
→ NORMALIZED USAGE DATA
→ BROKER/PROJECTION
→ CONSOLE USAGE SCREEN
→ CAPSULE STATUS BAR
→ CAPSULE USAGE DIALOG

Use exact source types and fields.

Document the normalized structures, including where relevant:

- provider
- account
- lifecycle
- membership
- freshness
- issues
- limit windows
- labels
- values
- reset labels
- remaining percentage
- money
- severity
- status slots
- unresolved capabilities

## Host-console Usage route

Document:

- entry shortcut/action
- top-level header
- split ratio or layout behavior
- account list
- Overview row
- provider grouping
- account rows
- secondary status lines
- meters
- selection
- detail pane
- account detail
- window rows
- reset text
- notices
- refresh behavior
- detail toggle
- scrolling
- empty state
- stale state
- errors
- exit

Explain exactly what is displayed on the left and right.

## Capsule usage presentation

Document:

- status-bar usage headline
- information-drop priorities under width pressure
- provider tabs
- focused account
- usage dialog
- quota meters
- severity
- source/confidence
- updated/stale labels
- provider switching
- refresh
- loading
- failure
- relationship to the focused agent/session

## Provider matrix

Create a normalized table:

| Provider | Account identity | Data source | Windows/buckets | Spend | Reset | Special states | Console | Capsule | Source |
|---|---|---|---|---|---|---|---|---|---|

Do not hide provider differences behind a generic summary.

---

# 17. LAUNCH COCKPIT COVERAGE

Map the complete launch experience.

Trace the launch UI from its public entry through transition into the running construct.

Document:

- launch identity
- target type
- Workspace/Role/Agent identity
- initial state
- all launch stages
- stage labels
- current/completed/failed status
- label transitions
- progress
- status text
- motion/animation
- no-motion mode
- build-log capture
- build-log button
- build-log overlay
- tail following
- scrolling
- scrollbar dragging
- failure popup
- diagnostics
- next steps
- copyable fields
- reveal/open actions
- failure acknowledgement
- container/debug information
- status footer
- context identity
- debug-mode differences
- quit confirmation
- hard cancellation
- launch completion
- alternate-screen continuity
- transition into capsule
- return behavior

Enumerate every launch-stage enum variant and every failure presentation variant.

For every stage record:

| Stage | Trigger | Visible label | Intermediate state | Completion | Failure | Source |
|---|---|---|---|---|---|---|

Document which stages can be simulated in the future design preview and what visible transition must be represented.

Do not implement that simulation now.

---

# 18. CAPSULE / IN-CONSTRUCT EXPERIENCE

Map the complete in-construct multiplexer.

Document:

- initial tab
- pane tree
- focused pane
- active tab
- custom and automatic tab labels
- shell vs agent pane
- visible provider label
- agent state
- Idle
- Working
- Done
- Blocked
- Unknown
- tab status glyphs
- pane borders
- splits
- split direction
- resizing
- drag state
- zoom
- pane selection
- text selection
- scrollback
- hardware cursor
- cursor visibility
- prefix mode
- normal mode
- dialog mode
- drag mode
- selection mode
- menu
- new agent/session flows
- close flows
- confirmation
- status bar
- branch context
- container identity
- debug identity
- Usage headline
- Usage dialog
- provider switching
- debug/container information
- copyable values
- mouse hover
- pointer shape
- link targets
- resize pointer
- text pointer
- tab switching
- hardline/reconnection
- detach/eject/exile behavior where connected
- exit and shutdown

Trace both:

- visible TUI state
- daemon/control-plane actions that change visible state

Do not document low-level daemon architecture unless it changes the operator experience.

Create a capsule action matrix:

| Context | Input | Guard | Visible action | Control-plane effect | Next state | Source |
|---|---|---|---|---|---|---|

---

# 19. SHARED INTERACTION SYSTEM

Create one canonical interaction reference covering all surfaces.

## Keyboard

Map:

- global keys
- screen-specific keys
- modal keys
- tab-list keys
- content keys
- list/tree keys
- form keys
- picker keys
- launch keys
- capsule-prefix keys
- cancellation
- destructive actions
- help
- quit

For each keymap context include:

| Context | Key | Action | Conditions | Visible result | Source symbol |
|---|---|---|---|---|---|

Explicitly document:

- Tab
- BackTab
- arrows
- j/k/h/l aliases
- Enter
- Space
- Esc
- PageUp/PageDown
- Home/End where used
- character shortcuts
- control shortcuts
- prefix sequences

Distinguish:

- action semantics
- toggle semantics
- navigation semantics
- editing semantics

## Mouse

Document:

- motion tracking
- hover
- pointer changes
- click
- click-to-focus
- click-to-select
- click-to-activate
- scroll-wheel routing
- scrollbar dragging
- split dragging
- text selection
- modal ownership
- background hit suppression

## Focus

Document:

- focus graph
- focus owner
- composite widgets
- tab bar vs tab content
- initial focus
- transfer
- restoration after modal
- scroll-induced focus where applicable
- focus sustainability across resize
- visual focus indication

## Hover

Explain how hover differs from:

- focus
- selection
- current item
- active item
- editing

## Cursor

Document:

- navigation cursor glyphs
- hardware cursor
- cursor visibility
- editing cursor
- scrollback suppression
- modal suppression

## Scrolling

Document:

- vertical
- horizontal
- list
- detail pane
- dialog body
- file browser
- build log
- capsule scrollback
- scrollbars
- follow-tail
- clamping
- resize behavior

---

# 20. CURRENT VISUAL LANGUAGE

Describe the current Jackin visual presentation so the future designer understands what information it currently communicates.

Cover:

- product brand line/pill
- palette
- semantic colors
- terminal-default background assumptions
- surfaces
- panels
- borders
- border focus
- tab styling
- active tab
- focused tab
- selected row
- hovered row
- cursor gutter
- action rows
- preview rows
- disabled rows
- warnings
- errors
- loading
- progress
- status bars
- branch/context bars
- hint bars
- blank separator rows
- dialogs
- backdrops
- links
- copy affordances
- pointer changes
- digital rain or other motion
- responsive behavior
- density
- typography through terminal modifiers

Use source paths for every rule.

End this section with two explicit subsections:

## Product meaning carried by the current presentation

For example:

- active interaction owner
- selected workspace
- active tab
- running state
- blocked agent
- destructive action
- current context
- provider status

## Current presentation choices that are not future design invariants

Examples may include, only if verified:

- PHOSPHOR palette
- green focus borders
- current box composition
- current white status bar
- exact footer ordering
- current cursor glyph
- exact panel layout
- current tab paint
- current modal geometry
- current animations

The future Junie implementation must preserve meaning and capability, not current paint.

Do not propose replacement designs here.

---

# 21. COMPLETE WORKFLOW CATALOG

Document every meaningful operator workflow.

For each workflow include:

- workflow ID
- goal
- prerequisites
- initial surface
- steps
- decisions
- opened dialogs
- visible data
- alternate paths
- cancellation
- errors
- recovery
- final state
- persistent effects
- exact source references

Use a step table:

| Step | Surface | Operator action | Visible response | State/effect | Next surface | Source |
|---|---|---|---|---|---|---|

At minimum investigate:

## Console and Workspace

- open console
- inspect workspaces
- navigate workspace details
- expand/collapse a Workspace
- inspect running instances
- start a Workspace
- start an additional session
- reconnect to a running instance
- create a Workspace
- edit a Workspace
- save changes
- discard changes
- delete a Workspace
- refresh state
- open Settings
- open Usage
- open help
- open debug/container information
- exit console

## Workspace configuration

- edit General
- add/remove/edit mounts
- browse for a source
- choose mount destination
- open GitHub source
- change isolation/read-only behavior
- add/remove/allow/default Roles
- add/edit/delete environment values
- choose environment scope
- select source
- use 1Password picker
- configure Auth
- add role-specific Auth override
- generate or select credentials
- resolve validation/save errors

## Settings

- modify global General settings
- modify global mounts
- modify global environments
- modify global Auth
- modify Trust
- save
- discard
- resolve errors

## Usage

- enter Usage
- inspect Overview
- select provider/account
- inspect all windows
- toggle detail
- refresh
- switch provider in capsule
- handle refreshing
- handle stale data
- handle needs login
- handle needs secret
- handle unsupported
- handle unavailable
- handle error
- exit Usage

## Launch

- initiate launch
- observe each stage
- inspect build log
- scroll/follow build log
- encounter launch failure
- copy diagnostics
- reveal/open relevant files where supported
- dismiss failure
- inspect container/debug information
- cancel
- confirm quit
- complete launch
- enter capsule

## Capsule

- interact with current pane
- switch tabs
- open menu
- create an agent/shell
- split pane
- resize pane
- focus another pane
- zoom
- enter scrollback
- select/copy text
- inspect agent state
- inspect Usage
- switch Usage provider
- inspect branch/container/debug context
- close pane/session/tab
- reconnect/hardline
- eject
- exile
- return to host

Verify exact current capabilities before including them as shipped.

If a listed flow is not implemented, classify it accurately rather than pretending it exists.

---

# 22. DATA-PRESENTATION MAP

Create a cross-cutting section explaining what data appears where.

Include domains such as:

- Workspace identity
- repository
- branch
- workdir
- Role
- Agent/provider
- runtime
- mounts
- environment values
- auth mode/source
- trust
- instance/container identity
- invocation/debug identity
- agent state
- launch stage
- Usage
- errors
- diagnostics
- versions

Use a table:

| Data concept | Source type | Source subsystem | Surfaces | Formatting | Empty/fallback | Source |
|---|---|---|---|---|---|---|

This section is essential for constructing realistic deterministic fixtures in the future design preview.

---

# 23. OPERATOR-VISIBLE COPY AND TERMINOLOGY

Inventory important current copy.

Include:

- screen titles
- tab labels
- panel labels
- row labels
- action labels
- button labels
- status messages
- progress stages
- empty-state messages
- validation messages
- error headings
- confirmation copy
- footer hints
- help text
- lifecycle vocabulary

Do not dump every arbitrary backend diagnostic.

Document stable UI copy and dynamic templates.

For every important phrase identify:

- where it appears
- when it appears
- dynamic substitutions
- source path

This lets the future designer preserve product meaning even when rewriting presentation.

---

# 24. RESPONSIVE AND HARD-CASE INVENTORY

Document current behavior and required future scenario coverage for:

- approximately 80x24
- approximately 100x30
- approximately 120x40
- approximately 160x50

Determine actual minimums and breakpoints from source.

Cover:

- narrow console
- wide console
- long workspace names
- long Role names
- many mounts
- many environment values
- many providers/accounts
- many quota windows
- many tabs
- many panes
- long diagnostics
- long file paths
- large build logs
- deeply nested pickers
- empty screens
- loading
- stale data
- unavailable providers
- launch failure
- blocked agents

Do not redesign responsive behavior.

Describe current behavior and what information/actions must remain representable later.

---

# 25. FUTURE PREVIEW SCENARIO MATRIX

The next `/goal` will build a runnable Jackin design preview with deterministic data, similar in methodology to TablePro.

Prepare a scenario matrix without designing the interface.

For every important product area list the scenarios required to demonstrate its complete behavior.

Use:

| Scenario ID | Product area | Initial data | State to demonstrate | Interaction required | Visible outcome | Source |
|---|---|---|---|---|---|---|

Include sufficient scenarios for:

- populated Workspace manager
- empty Workspace manager
- multiple running instances
- create/edit Workspace
- each editor tab
- dirty/save/discard
- each Settings tab
- every important picker family
- every important confirmation family
- multiple usage providers
- multiple accounts
- fresh/stale/refreshing/error Usage
- launch progress
- launch failure
- build log
- capsule with one agent
- capsule with multiple agents
- Working/Idle/Done/Blocked states
- split panes
- Usage dialog
- debug/container info
- help
- responsive layouts

Do not create fixture data now.

Describe the required data shape and semantic state.

---

# 26. REDESIGN COVERAGE CONTRACT

End the product analysis with a complete checklist for the later implementation `/goal`.

This section defines what the future Junie-based Jackin preview must cover.

Organize it into:

## Must represent

Information and identity that must always be understandable.

## Must support interactively

Workflows that must be operable in the runnable preview.

## May use deterministic simulation

Backend-heavy operations whose visible states and transitions should be real but whose infrastructure may be simulated.

Examples may include:

- Docker work
- Git operations
- provider network calls
- credential operations
- usage refresh
- image building
- agent startup

## Must preserve semantically

State distinctions and product concepts that cannot be lost.

## May be completely redesigned

Current layouts, chrome, styling, panel arrangement, visual hierarchy, shortcut presentation, and other presentation choices that are not product invariants.

## Out of scope unless required

Backend implementation details that do not affect the visible experience.

Use the stable surface and workflow IDs throughout this checklist.

Do not propose the future visual design.

---

# 27. TARGETED SOURCE-READING INDEX

Create a final index optimized for the future implementation agent.

The index must answer:

> I am implementing or redesigning `<surface or workflow>`. Which exact files and symbols should I open?

Organize by:

- product model
- console routing
- Workspace manager
- Workspace editor
- Settings
- modal system
- file browser
- Role pickers
- source/scope pickers
- auth form
- Usage projection
- each Usage provider
- launch cockpit
- launch failure
- build log
- capsule tabs
- capsule panes
- capsule dialogs
- capsule Usage
- focus
- mouse
- keymaps
- scrolling
- visual tokens
- tests
- baselines
- documentation

Each entry must provide:

- local source path rooted at `/Users/donbeave/junie-style-2/jackin`, pinned to the analyzed SHA
- repository-relative path
- exact line range
- symbol
- concise reason to inspect it

This source-reading index is one of the most important parts of the document.

---

# 28. SOURCE-CITATION FORMAT

Every substantive implementation claim must cite real source.

Use a consistent format such as:

```markdown
Source:

- `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L120-L188` (commit `<PINNED_SHA>`) — renders the workspace list and detail split.
- `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/update.rs:L310-L390` (commit `<PINNED_SHA>`) — handles selection and activation.
```

Generate line numbers against the pinned checkout using a reliable method such as `nl -ba`.

Verify every source path and line range against the pinned local checkout before finishing.

Do not cite an entire large file when a precise range is available.

Do not paste large code blocks.

Use:

- path
- line range
- symbol
- explanation

rather than copying implementation.

Short operator-visible labels or enum names may be quoted where necessary.

---

# 29. CURRENT VS PLANNED

Create a capability-status table:

| Capability | Current | Partial | Planned | Research only | Evidence |
|---|---:|---:|---:|---:|---|

Inspect plans and research only to prevent future concepts from being mistaken for shipped behavior.

The redesign reference should center on current implemented capabilities.

A planned capability may be included only in a clearly separated appendix if understanding it prevents ambiguity.

Never include a planned screen in the main surface inventory as if it currently exists.

---

# 30. DOCUMENT QUALITY RULES

Completeness is more important than brevity.

There is no arbitrary token or page cap.

However:

- do not paste source files
- do not repeat identical explanations
- do not include generic TUI theory
- do not include generic design advice
- do not explain Rust concepts unrelated to visible behavior
- do not narrate every internal helper
- do not copy documentation wholesale
- do not use vague claims such as “supports settings”
- do not say “standard keyboard navigation”
- do not say “opens a popup” without naming the popup, trigger, data, actions, and result

Prefer:

- normalized tables
- stable IDs
- state-transition matrices
- diagrams
- source maps
- exact copy
- exact key bindings
- explicit conditions
- exact data fields
- direct cross-references

A future agent must be able to convert the document directly into implementation scope and acceptance criteria.

---

# 31. COMPLETENESS AUDITS

Before finishing, perform independent audits.

## Surface audit

Compare the document against every:

- route
- screen
- stage
- tab
- pane
- overlay
- modal
- dialog
- popup
- picker
- menu
- status surface
- progress surface

## Modal-call-site audit

For each modal family, verify all concrete call sites and target variants are documented.

## Keymap audit

For every keymap context:

- every active key appears in the reference
- every key is attached to the right scope
- context-dependent behavior is documented

## Usage-provider audit

For every current provider:

- provider module documented
- account identity documented
- fields/windows documented
- status behavior documented
- console representation documented
- capsule representation documented
- source links present

## Launch audit

Every launch stage, overlay, failure action, progress state, and terminal transition is accounted for.

## Capsule audit

Every visible agent state, mode, tab/pane action, menu/dialog action, usage interaction, and lifecycle path is accounted for.

## Test/baseline audit

Compare the reference against:

- snapshot tests
- visual tests
- PNG baseline registries
- fixture variants

Any baseline state not represented in the document must be explained.

## Docs-versus-source audit

Find all cases where documentation and code differ.

Record material differences.

## Redesign-readiness audit

Confirm every current capability has:

- semantic meaning
- visible data
- interaction
- state variants
- source map
- future coverage classification

---

# 32. ACCEPTANCE TEST FOR THE DOCUMENT

Imagine a future coding agent receives only:

- `/Users/donbeave/junie-style-2/terminal-components-claude`
- its existing `DESIGN.md`
- the approved Junie component showcase
- the existing TablePro implementation
- `JUNIE_PROMPT1.md`
- `JUNIE_PROMPT2.md`
- `JACKIN_REFERENCE.md`
- access to the pinned Jackin source links

That agent is then instructed:

> Build a completely redesigned, coherent, interactive Jackin TUI preview using the approved Junie design system.

Could the agent use `JACKIN_REFERENCE.md` to determine:

- every current operator-visible surface?
- every tab?
- every pane?
- every modal and concrete modal flow?
- every picker?
- every important row and field?
- what data appears on every screen?
- where that data originates?
- every provider-specific Usage capability?
- every keyboard action?
- every mouse action?
- every focus transition?
- every loading, stale, error, empty, and disabled state?
- every launch stage?
- every capsule state?
- every end-to-end workflow?
- what must survive semantically?
- what may be completely redesigned?
- which exact source ranges to inspect for each feature?
- which backend operations may be simulated in the design preview?
- what acceptance journeys the final preview must demonstrate?

Could the agent do this without broadly searching and rediscovering the Jackin repository?

If any answer is no, the reference is incomplete.

---

# 33. NO BLOCKING

Do not ask the operator to make ordinary research decisions.

If the application cannot be run:

- inspect source
- inspect tests
- inspect baselines
- inspect fixtures
- inspect docs
- record the evidence class

If Docker, credentials, keychain access, provider APIs, or network services are unavailable:

- do not block
- do not use real secrets
- do not mutate the operator’s real environment
- trace the current code path
- verify through tests where possible
- document runtime uncertainty precisely

If one research path is blocked, spawn additional subagents to find another evidence path.

If source and docs disagree, resolve current behavior from source/tests and record the conflict.

Precise `UNKNOWN` is better than invented certainty.

---

# 34. FINAL VERIFICATION

Before finishing:

1. Confirm the Jackin commit SHA is pinned.
2. Confirm all links use that SHA.
3. Confirm all cited line ranges exist.
4. Confirm every editor tab is documented.
5. Confirm every Settings tab is documented.
6. Confirm every console modal variant is documented.
7. Confirm every concrete modal use is documented.
8. Confirm every picker mode and target is documented.
9. Confirm every Usage provider is documented.
10. Confirm provider-specific data differences are documented.
11. Confirm host-console Usage and capsule Usage are both documented.
12. Confirm every launch stage is documented.
13. Confirm launch build-log and failure flows are documented.
14. Confirm every capsule visible state is documented.
15. Confirm keyboard, mouse, focus, hover, cursor, and scrolling are documented.
16. Confirm all important error, empty, stale, unavailable, loading, and disabled states are documented.
17. Confirm end-to-end workflows are connected.
18. Confirm current presentation is separated from product semantics.
19. Confirm planned functionality is separated from shipped functionality.
20. Confirm the targeted source-reading index is complete.
21. Confirm the future preview scenario matrix is complete.
22. Confirm no redesign has been proposed or implemented.
23. Confirm no Jackin code was copied.
24. Confirm no reference application was created.
25. Confirm no existing Junie source was changed.
26. Confirm the final Git diff contains only `JACKIN_REFERENCE.md`.

---

# 35. DEFINITION OF DONE

This phase is complete only when one comprehensive file exists:

`/Users/donbeave/junie-style-2/terminal-components-claude/JACKIN_REFERENCE.md`

and that file:

- analyzes Jackin at an exact pinned commit
- explains the product and mental model
- maps canonical vocabulary
- maps domain entities and scope
- maps the complete application topology
- catalogs every operator-visible screen
- catalogs every pane and tab
- catalogs every overlay
- catalogs every modal family
- catalogs every concrete modal flow
- catalogs every picker
- catalogs every status and progress surface
- documents every visible field, row, group, label, and action
- documents data sources and transformations
- documents every important state
- documents keyboard behavior
- documents mouse behavior
- documents focus behavior
- documents hover behavior
- documents cursor behavior
- documents scrolling and overflow
- documents responsive behavior
- documents the Workspace manager
- documents Workspace creation and editing
- documents every Workspace Editor tab
- documents every Settings tab
- documents roles, mounts, environments, Auth, and Trust
- deeply documents Usage
- documents every current Usage provider
- documents provider-specific accounts, windows, spend, reset, lifecycle, freshness, and errors
- documents host-console Usage
- documents capsule Usage
- documents launch stages
- documents build-log behavior
- documents launch failures
- documents transition into the construct
- documents the capsule multiplexer
- documents agents, shells, tabs, panes, splits, zoom, scrollback, and state
- documents hardline, reconnect, eject, exile, and shutdown where currently implemented
- documents help and debug information
- maps all important end-to-end workflows
- separates shipped, partial, planned, and research-only capabilities
- separates product semantics from replaceable current presentation
- contains a future preview scenario matrix
- contains a redesign coverage contract
- contains immutable source links with exact line ranges
- contains a targeted source-reading index
- has passed all completeness audits
- lets the future implementation agent avoid broad repository rediscovery

No Rust implementation is produced.

No current Jackin replica is produced.

No source code is extracted.

No design is proposed.

No Junie component is modified.

The work stops after the complete source-backed reference exists.

Analyze the real product.

Map every visible capability.

Map every workflow.

Map every datum.

Map every state.

Point directly to the real source.

Prepare the canonical reference.

STOP BEFORE DESIGNING OR IMPLEMENTING.
