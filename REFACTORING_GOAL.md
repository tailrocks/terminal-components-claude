You are a principal Rust framework engineer, senior Ratatui engineer, terminal interaction architect, component-library architect, public-API designer, and professional product designer specializing in premium keyboard-first terminal experiences.

This is an implementation goal. Do not stop after analysis, recommendations, an architecture document, or a migration plan. Research the problem, make the architectural decisions, implement the complete refactor, migrate every application, run and interact with the result, inspect rendered output, correct problems, and deliver a finished repository.

Use only the configured agent types for this goal:

- `refactor-coordinator` — `claude-fable-5-1`, effort `high`; owns sequencing, repository state, integration, durable records, completion evidence, and the final report.
- `opus-analyst` — `claude-opus-5`, effort `high`, read-only; owns all research and judgment work.
- `fable-builder` — `claude-fable-5-1`, effort `high`; owns implementation and execution work within an assigned scope.

Use subagents aggressively where supported. Parallelize independent Opus research/review and disjoint Fable implementation work. The primary Fable coordinator owns execution and integrated repository state. Opus owns architectural synthesis and adjudication of conflicting research. The Fable coordinator resolves the recorded result against the authority order in this document and implements it.

Resolve ordinary decisions autonomously within these model boundaries. Opus compares and selects architectural and public-API alternatives. Fable selects local implementation details that do not change accepted architecture or public invariants. Do not ask the user to choose between routine Rust API alternatives.

# 0. MANDATORY MODEL ROUTING

Run the primary coordinator and every implementation worker with `claude-fable-5-1`, effort `high`.

Use fresh `opus-analyst` agents for every exploratory repository audit, external or current-documentation research, architecture decision, alternative comparison, public-API critique, test-design review, domain-boundary decision, security or threat analysis, performance interpretation, visual judgment, architectural root-cause diagnosis, and independent review. "Research" includes investigation of repository source, tests, captures, and rendered behavior, not only web research. Opus agents are read-only and return evidence-backed findings.

Fable may perform targeted reads needed to execute an accepted design. Fable owns all worktree mutations, production code, migrations, test and capture execution, benchmark collection, documentation updates, cleanup, corrections, integration, and final reporting. Fable records accepted Opus findings in `COMPONENT_ARCHITECTURE.md`, `REFACTORING_STATE.md`, and other repository documents.

Fable must not silently change an accepted architecture or public invariant. If implementation exposes such a need, pause that slice, obtain fresh Opus adjudication, record the decision, then continue with Fable.

Never use generic, inheriting, built-in Explore, or built-in Plan agents for this goal. Never use Opus for repository mutation. Never pass per-invocation model or effort overrides; configured agent definitions own routing. Treat an unavailable required model, model substitution, or model/effort mismatch as a blocker.

Maintain `REFACTORING_STATE.md` throughout the run so work survives compaction and resume. Record baseline revision/status, active slice, agent and model assignment, accepted decisions, explicit file ownership, completed gates, pre-existing failures, unresolved findings, and next action. Fable alone edits this ledger.

---

# 1. MISSION

Refactor this repository in place from an excellent but prototype-shaped Junie-inspired Ratatui design laboratory into a genuinely reusable, composable, flexible, themeable, and professionally engineered Rust TUI component system.

The target experience is analogous to the strongest architectural qualities of shadcn/ui, translated properly into Rust and terminal UI constraints:

- beautiful, polished defaults
- open and understandable implementation
- predictable and consistent APIs
- strong composition
- semantic theming
- easy local customization
- reusable behavior
- escape hatches for advanced use
- components that work naturally together
- APIs that are understandable to both engineers and coding agents
- application code that expresses product intent rather than focus, hit-test, cursor, and styling plumbing

Do not imitate React, JSX, Tailwind, CSS variables, DOM event bubbling, `asChild`, or virtual-DOM mechanics literally. Research why those mechanisms work in shadcn/ui and translate the underlying principles into an idiomatic Rust/Ratatui architecture.

The approved existing Junie design must remain the default visual and interaction experience.

A user who does nothing should receive the current high-quality Junie appearance and behavior.

A user who wants customization must be able to:

- supply a completely custom theme using their own colors
- override only a few semantic tokens and inherit the rest
- customize one component family globally
- customize a component within a subtree or local scope
- customize one specific component instance
- customize meaningful logical parts and interaction states
- create custom variants without replacing component behavior
- combine components into new higher-level components
- implement a custom component using the same interaction and theme infrastructure

The exact Rust types, traits, generics, state model, theme representation, and composition mechanism are architectural decisions for you to research and make. The requirements in this goal describe required capabilities and invariants, not a prescribed implementation.

---

# 2. NON-NEGOTIABLE RESULT

At completion:

1. The reusable component system has one coherent public API and interaction model.
2. `showcase`, `tablepro`, and `jackin-preview` use the refactored public component API.
3. There is no parallel legacy component API.
4. Backward compatibility with the current experimental API is not required.
5. Old APIs, compatibility wrappers, deprecated aliases, temporary adapters, and duplicate implementations are removed once migration is complete.
6. The existing Junie visual language remains available as the polished default theme.
7. At least one substantially different non-Junie theme proves that the theme system is real rather than a renamed Junie palette.
8. Per-component and per-instance customization is demonstrated in running code.
9. Normal application code no longer manually reimplements focus registration, hover, press/release, hit testing, child ownership lookup, cursor placement, modal barriers, or routine component event routing.
10. Complex product behavior remains in the applications, while generic interaction and presentation behavior lives in the reusable system.
11. Every existing component is intentionally migrated, decomposed, replaced, or removed with its disposition documented.
12. The repository is built, tested, run, navigated, clicked, scrolled, resized, visually captured, independently reviewed, and corrected.

The refactor must be architectural. Do not satisfy this goal by adding builders around the existing structs, making fields private, introducing a superficial `Widget` trait, or renaming `Theme`.

---

# 3. AUTHORITY ORDER

Use the following authority order when requirements conflict:

1. This goal — highest authority for architecture, API, migration, and completion.
2. `DESIGN.md` — authority for the approved default visual language, terminal interaction principles, state treatment, spacing, glyph semantics, responsive behavior, and design quality.
3. Existing rendered output, capture artifacts, visual baselines, and interaction tests — regression evidence for the default theme and current product behavior.
4. Current source under `src/core/`, `src/ui/`, `src/widgets/`, and `src/runtime.rs` — implementation evidence, not an architecture that must be preserved.
5. Current `showcase`, `tablepro`, and `jackin-preview` behavior and tests — application semantics that must continue to work unless an existing defect is deliberately corrected.
6. shadcn/ui and external references — conceptual and architectural references, not implementation templates.

Preserve the exact default Junie token values and established visual behavior unless a change is necessary to correct a demonstrated inconsistency or defect. Document every intentional visual change and review it through actual captures.

When implementation establishes a reusable visual or interaction rule, update `DESIGN.md` so documentation and rendered behavior remain aligned.

---

# 4. SCOPE

Audit and refactor all relevant layers, including:

- `src/core/event.rs`
- `src/core/focus.rs`
- `src/core/hit.rs`
- `src/core/id.rs`
- `src/core/scroll.rs`
- `src/core/text.rs`
- `src/runtime.rs`
- `src/theme.rs`
- `src/ui/ctx.rs`
- `src/ui/layout.rs`
- `src/ui/popup.rs`
- `src/ui/text.rs`
- every module under `src/widgets/`
- application-specific reusable-looking controls under `src/bin/showcase/`
- application-specific reusable-looking controls under `src/bin/tablepro/`
- application-specific reusable-looking controls under `src/bin/jackin_preview/`
- tests, fixtures, captures, documentation, and tooling
- Cargo package or workspace organization
- CI or other deterministic quality gates

The existing widget modules include:

- brand
- button
- chips
- choice
- code
- completion
- dialog
- diff
- empty
- field_common
- grid
- hintbar
- input
- keyhint
- list
- menu
- panel
- picker
- progress
- props
- scrollbar
- segments
- select
- splitter
- statusbar
- steps
- table
- tabs
- textarea
- tree
- viewport

No component may be silently skipped because it is difficult or application-specific today.

---

# 5. IMPORTANT NON-GOALS

Do not turn this project into a clone of a web framework.

Do not introduce a virtual DOM, CSS parser, Tailwind-like class system, stringly typed style language, plugin ABI, or declarative macro DSL unless rigorous research proves that it is necessary and materially better than simpler Rust APIs.

Do not implement the shadcn registry, installer CLI, or code-distribution service as part of this goal. Preserve architectural room for future distribution, but first finish the component library itself.

Do not redesign TablePro or Jackin from scratch.

Do not integrate real databases, provider APIs, terminals, or external Jackin processes. Preserve their current deterministic preview and simulation boundaries.

Do not retain bad APIs to minimize the diff.

Do not create a giant universal trait containing rendering, events, layout, focus, theme, data access, and application effects merely to claim that every component implements one trait.

Do not hide behavior behind excessive procedural macros or code generation. The implementation must remain open, readable, debuggable, and easy for engineers and coding agents to understand.

Do not optimize for backward compatibility. This is an experimental research repository whose purpose is to discover the strongest possible API.

---

# 6. BASELINE THE REPOSITORY BEFORE REFACTORING

Before changing architecture:

1. Inspect `git status` and preserve unrelated local changes.
2. Read `README.md`, `DESIGN.md`, and the relevant current source and test files.
3. Read every core, UI, theme, runtime, and widget module.
4. Inspect every place where the three applications construct, mutate, render, and route events to components.
5. Run the current formatting, linting, tests, and builds.
6. Run each application.
7. Exercise representative keyboard and mouse flows.
8. Capture representative output at the supported viewport sizes.
9. Record pre-existing failures separately from regressions.
10. Preserve a before-refactor evidence set for visual comparison.

At minimum run the repository’s current equivalents of:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --bins
````

Use the existing `tools/capture.sh`, Ratatui `TestBackend`, deterministic scenarios, and visual baseline facilities.

Capture at least:

* showcase at `80×24`, `100×30`, `120×40`, and `160×50`
* representative default, focused, hovered, pressed, disabled, selected, error, busy, editing, overflow, and empty states
* TablePro connection, editor, grid, tabs, dialog, menu, picker, and results surfaces
* Jackin host, settings, account/usage, launch, Capsule, menu, modal, tab, status-bar, and responsive surfaces
* truecolor, 256-color, 16-color, and no-color or monochrome output where supported

Do not regenerate a visual baseline merely because a test fails. Inspect and classify the difference first.

---

# 7. COMPLETE CURRENT-STATE AUDIT

Create a complete component and interaction audit before finalizing the architecture.

For every component and reusable-looking application control, record:

* current responsibility
* public constructors and methods
* public mutable fields
* configuration or props
* owned application data
* owned interaction state
* frame-local geometry and caches
* render signature and render-time mutation
* keyboard handlers
* mouse handlers
* paste, wheel, drag, and tick behavior
* event or result type
* focus registration
* child identity
* hit-test registration
* ownership and `locate` helpers
* scroll ownership
* overlay behavior
* theme dependencies
* raw background or color parameters
* hard-coded glyphs, dimensions, and spacing
* domain-specific assumptions
* application-specific copies or variants
* existing tests
* target disposition: retain, refactor, decompose, compose, move, or remove

Build an API-inconsistency matrix. Explicitly identify differences such as:

* `Outcome` versus `(Outcome, Option<Event>)` versus booleans
* `new`, `with_items`, specialized constructors, direct public-field mutation, and builders
* internal versus caller-owned selection
* internal versus caller-owned data
* component-specific `owns`, `locate`, `row_id`, `close_id`, and scrollbar handling
* render methods that require raw background colors
* render methods that perform semantic state transitions
* components that expose internal rectangles for later routing
* components that hard-code keyboard bindings
* components with closed body/content enums
* components that combine generic presentation with application-domain behavior
* validators restricted to function pointers or other unnecessarily narrow extension mechanisms
* masked or secret-bearing controls whose `Debug`, cloning, snapshots, logs, or tests could expose raw values

Search the application directories for:

* direct `Focus` and `FocusRing` manipulation
* direct `HitRegistry` use
* `.owns(...)` and `.locate(...)`
* manually derived child IDs
* manual pressed/hover routing
* raw Ratatui styles and color decisions
* hand-built dialogs, menus, tabs, forms, sidebars, status bars, and scrollbars
* custom key handling that duplicates component behavior
* reusable interaction logic embedded in screens

The audit is not the final deliverable. Use it to make decisions and then continue immediately into implementation.

---

# 8. EXTERNAL RESEARCH

Research current primary sources before choosing the new architecture.

At minimum inspect:

* the current shadcn/ui source and documentation:
  [https://github.com/shadcn-ui/ui](https://github.com/shadcn-ui/ui)
* Rust API Guidelines:
  [https://rust-lang.github.io/api-guidelines/](https://rust-lang.github.io/api-guidelines/)
* the Ratatui version used by this repository and current Ratatui source/documentation:
  [https://github.com/ratatui/ratatui](https://github.com/ratatui/ratatui)
* current Crossterm input and terminal semantics:
  [https://github.com/crossterm-rs/crossterm](https://github.com/crossterm-rs/crossterm)

You may inspect other maintained Rust UI or TUI libraries when they provide useful evidence about:

* retained versus immediate component models
* stateless view plus external state
* stateful widgets
* event propagation
* command/message architectures
* focus scopes
* overlays
* component identity
* declarative composition
* style recipes
* custom item renderers
* testability

Use primary documentation and source rather than blog summaries.

Do not copy another framework wholesale. Record:

* which problem a reference solves
* whether that problem exists here
* how Rust ownership and terminal rendering change the trade-off
* what was adopted
* what was rejected
* why the chosen design is appropriate for this repository

Record versions or commits for important external references so the research remains reproducible.

---

# 9. ARCHITECTURE DECISION PHASE

Create `COMPONENT_ARCHITECTURE.md` or an equivalently focused architecture document.

The document must contain:

1. Current-state diagnosis.
2. Design goals and non-goals.
3. The component model.
4. State ownership rules.
5. Rendering rules.
6. Event and semantic-action model.
7. Component identity and stable-key model.
8. Focus and interaction model.
9. Overlay and modal model.
10. Layout, measurement, and surface inheritance.
11. Theme and customization model.
12. Composition and component-part model.
13. Public API conventions.
14. Generic versus domain-specific boundaries.
15. Package or workspace boundary.
16. Testing strategy.
17. Representative proposed usage examples.
18. Migration map for every current component.
19. Alternatives considered and rejected.
20. Known trade-offs.

Do not pause for user approval after writing this document. Treat it as an internal design checkpoint, independently review it, improve it, and continue to implementation.

Do not assume that the README’s suggestion of one `Widget` trait and one `Theme` trait is automatically correct.

Explicitly compare alternatives for the following questions.

## 9.1 Component model

Evaluate options such as:

* retained component objects
* stateless views with explicit external state
* stateful widgets
* separate behavior controllers and renderers
* generic composition
* trait-object composition
* enum-based composition
* render closures or delegates
* a deliberate hybrid

Judge them against:

* normal-use ergonomics
* advanced customization
* type safety
* borrowed-data support
* lifetime complexity
* heterogeneous composition
* testability
* code readability
* dynamic collections
* performance
* application migration
* component-author experience

Choose the smallest coherent model that supports all required use cases.

## 9.2 Theme representation

Do not assume a trait is necessary.

Compare:

* a concrete data-driven theme
* semantic token structures
* component recipes
* typed style patches
* traits
* generic theme parameters
* trait objects
* scoped overlays
* a deliberate combination

The result must make custom themes and local overrides easy without spreading generics throughout every application or forcing dynamic dispatch everywhere.

## 9.3 Event model

Compare ways to represent:

* consumed versus ignored input
* redraw or invalidation
* semantic component actions
* application-domain actions
* child-part identity
* keyboard and mouse equivalence
* event capture and propagation
* bubbling or delegation
* overlay precedence
* pointer capture
* focus transitions

The result must eliminate ad hoc combinations of booleans, tuples, and unrelated component-specific conventions.

## 9.4 Composition model

Determine how a Rust caller can:

* place arbitrary content in panels and dialogs
* compose headers, bodies, footers, actions, descriptions, and overlays
* render custom rows or cells
* wrap or decorate components
* override logical parts
* build higher-level components from lower-level primitives
* reuse behavior with different presentation
* provide borrowed content without unnecessary allocation

Translate shadcn/ui’s compound-component and open-code ideas into idiomatic Rust rather than reproducing its web syntax.

## 9.5 Package boundary

Evaluate whether the repository should become a Cargo workspace with one or more library crates and separate application packages.

A workspace is a likely solution, but do not choose it merely because it is conventional. The non-negotiable requirement is a mechanically enforceable boundary proving that applications consume only supported public APIs.

If the repository remains one package, provide another convincing mechanism and document why it is stronger than a workspace split.

---

# 10. PUBLIC API QUALITY

Create one predictable vocabulary across the component system.

Every public component must have documented and consistent answers to:

* How is it constructed?
* Which data does it borrow or own?
* Which state does the caller own?
* Which state does the component system own?
* How is it configured?
* How are variants and sizes selected?
* How is it disabled or made read-only?
* How are loading, busy, error, selected, and editing states represented?
* How are events delivered?
* How is focus handled?
* How is it rendered?
* How is it measured?
* Which logical parts are styleable?
* How is a local style override applied?
* How is a custom item renderer supplied?
* How is identity kept stable?
* How is it tested?

Follow Rust API design principles:

* clear ownership
* minimal unnecessary cloning
* borrowed inputs where they improve ergonomics
* no unnecessary `'static` restrictions
* no boolean parameter soup
* typed variants for semantically different modes
* private implementation details
* small focused traits
* useful concrete defaults
* meaningful error types
* no panics during normal interaction
* complete rustdoc
* predictable names
* no gratuitous generic parameters
* no stringly typed component internals unless strongly justified
* no public mutable layout rectangles or caches
* no accidental exposure of secret data through `Debug` or logs

Do not make every type generic merely to appear flexible.

Do not make every component a trait object merely to support heterogeneous containers.

Optimize for the common path while retaining deliberate lower-level extension points for component authors.

A normal application author and a custom component author may need different API layers. Design those layers explicitly rather than exposing all internals to everyone.

---

# 11. STATE OWNERSHIP AND RENDERING

Separate these concerns clearly:

1. Application-domain data and effects.
2. Component configuration or props.
3. Durable component interaction state.
4. Controlled values and selections.
5. Frame-local layout and geometry.
6. Theme and resolved visual state.
7. Semantic actions emitted to the application.

Rendering must be deterministic.

Rendering may:

* write to the Ratatui buffer
* register frame-local interaction metadata
* report cursor placement
* calculate and cache frame-local geometry where justified
* update non-semantic viewport metadata required by the current frame

Rendering must not silently:

* commit an edit
* cancel an edit
* run validation because focus changed
* mutate application-domain data
* activate an action
* change a selected value
* reorder data
* delete or insert records
* emit domain effects
* close an overlay
* alter focus as a hidden consequence of drawing

Focus-loss, validation, commit, cancellation, synchronization, and activation must happen through explicit interaction or lifecycle transitions that can be tested independently of rendering.

Document controlled and uncontrolled behavior clearly. Do not copy React terminology unless it improves the Rust API, but provide equivalent capabilities where they are useful.

When externally supplied data changes:

* selection must be reconciled predictably
* cursor and scroll state must remain valid
* dynamic item identity must not depend only on a shifting numeric index
* disappearing focused elements must produce a defined focus transition
* edits must not silently target a different item after reorder

---

# 12. COMPONENT IDENTITY AND DISPATCH

Design a robust identity and dispatch model.

It must support:

* readable debugging
* stable identity across frames
* nested components
* repeated components
* dynamic collection reorder
* component parts such as tab close buttons and scrollbar thumbs
* event source identification
* focus restoration
* overlay ownership
* collision detection or convincing collision safety
* tests that can address logical controls
* local IDs without requiring callers to manually invent names for every internal child

Applications should not need long chains such as:

* check `owns(id)`
* call `locate(id)`
* manually translate a child ID to an index
* manually focus the parent
* call the child-specific click handler
* separately apply the emitted event

Replace this with a coherent reusable dispatch or composition mechanism.

The exact implementation may use paths, scoped IDs, nodes, registrations, typed keys, metadata, delegates, or another researched approach. Prove it through the migrated applications.

Any remaining manual dispatch in application code must be genuinely domain-specific and explicitly justified.

---

# 13. INTERACTION SYSTEM

The component system must provide a coherent interaction foundation so application authors do not debug the same focus, cursor, hover, and selection problems repeatedly.

Preserve the approved DESIGN.md semantics while generalizing their implementation.

Support, where meaningful:

* deterministic Tab and Shift+Tab navigation
* disabled controls skipped by keyboard focus
* roving focus or internal cursors for composite widgets
* nested focus scopes
* modal focus trapping
* restoration of prior focus
* focus reconciliation when controls disappear
* focus-visible treatment
* keyboard navigation that does not conflict with editing
* hardware cursor placement
* hover without stealing keyboard focus
* stale-hover suppression after keyboard interaction
* mouse down, mouse up, and activation only on a valid completed click
* pressed feedback
* pointer capture for drag operations
* text or range selection
* wheel routing to the topmost scrollable target
* nested scrolling
* scrollbar click and drag
* context or secondary actions
* overlay precedence
* inert background content
* click-outside dismissal
* resize
* paste
* ticks and animation
* no-color and monochrome operation

Keyboard and mouse activation of the same control should produce the same semantic component action whenever their meaning is equivalent.

Generic components may provide good terminal-native default bindings, but application-domain chords must remain outside generic component behavior.

Research whether component bindings should be configurable through commands, key maps, action descriptors, or another mechanism. Ensure the result does not hard-code one application’s shortcuts into generic components.

Integrate contextual action or hint metadata where useful so the shared HintBar can describe the active component without every screen manually recreating the same key descriptions.

---

# 14. OVERLAYS, DIALOGS, MENUS, AND POPUPS

Create one coherent overlay model for:

* dialogs
* confirmation flows
* prompts
* pickers
* command palettes
* menus
* context menus
* selects
* completion popups
* viewers
* anchored popovers

It must define:

* stacking
* z-order
* nested overlays
* modal versus non-modal behavior
* focus trapping
* focus restoration
* pointer barriers
* inert backgrounds
* click-outside behavior
* Esc behavior
* anchor geometry
* flipping and clamping
* clipping
* small-terminal behavior
* cursor ownership
* contextual hints
* lifecycle events

Do not preserve a closed dialog architecture in which the reusable dialog understands only a hard-coded list of body types.

The system must permit arbitrary composed dialog content while still offering ergonomic convenience constructors for common confirmations, destructive confirmations, prompts, typed acknowledgements, and fact summaries.

Convenience APIs must be implemented on top of the same primitives rather than through a separate rendering path.

---

# 15. THEMING AND CUSTOMIZATION

The theme system is a primary deliverable.

Preserve the current Junie design as a built-in default.

Research and implement a theme architecture with semantic roles rather than components depending directly on Junie-specific palette assumptions.

At minimum cover:

* canvas and surfaces
* foreground hierarchy
* borders
* accent and on-accent
* focus
* selection
* hover and elevation
* input surfaces
* overlays
* destructive, error, warning, success, and information roles
* disabled treatment
* syntax roles
* chart or meter roles where appropriate
* terminal color capability downgrade
* monochrome or no-color operation

Also determine which of these belong in color tokens, layout/design tokens, component recipes, or separate configuration:

* spacing
* padding
* dimensions
* glyphs
* border glyph sets
* focus indicators
* selection indicators
* scrollbar symbols
* animation cadence
* component density
* variant defaults

The minimum programmatic customization scenarios are:

1. Construct the default Junie theme with no configuration.
2. Create a complete custom theme by supplying custom colors.
3. Change only a small set of semantic roles and inherit or derive the rest safely.
4. Override one component family globally.
5. Override one component variant globally.
6. Override a component within a local scope or subtree.
7. Override one component instance.
8. Override a meaningful logical part such as container, border, gutter, marker, label, metadata, icon, body, action row, track, thumb, or selected row.
9. Override styles for meaningful states such as focused, hovered, pressed, selected, disabled, error, busy, and editing.
10. Build an application-specific custom component that consumes the same semantic theme.
11. Apply truecolor, 256-color, 16-color, and no-color conversion to both built-in and user-provided themes.
12. Preserve state meaning when color is unavailable.

Design and document deterministic override and style-merging semantics.

Unspecified values must inherit predictably. There must be an explicit way to replace or clear a default when needed. State and part precedence must be documented and covered by tests.

Avoid forcing callers to pass raw background colors through every component render call. Background and surface inheritance should be contextual or otherwise represented semantically. Any remaining public raw-color parameter requires explicit architectural justification.

After migration:

* reusable components must not contain literal palette colors
* applications must not recreate component colors
* default component rendering must resolve through the theme or recipe system
* user overrides must not require editing component internals
* the custom theme must visibly affect every public component
* focus, selection, editing, disabled, and error states must remain understandable without relying only on hue

Add at least one highly distinct theme to the showcase. It must not be a minor accent-color variation. Use it to expose hidden Junie assumptions and correct them.

Provide a runtime or deterministic showcase mechanism for switching themes and inspecting local overrides.

---

# 16. VARIANTS, SIZES, PARTS, AND EXTENSION

Establish a consistent approach to:

* semantic variants
* size or density variants
* component parts
* default variants
* custom variants
* style recipes
* state-specific resolution
* per-instance patches
* custom content
* icons or glyphs
* prefixes and suffixes

Do not preserve unrelated component-specific configuration conventions merely because they already exist.

Do not force every possible custom design into a closed enum.

Do not abandon type safety for arbitrary string maps without strong justification.

Document each public component’s logical parts. Those part identities should be useful for:

* theming
* local overrides
* event source metadata
* testing
* accessibility or semantic descriptions
* component composition

The exact mechanism is your decision. It may involve typed part enums, structured recipes, style callbacks, patch objects, or another researched design.

---

# 17. LAYOUT, MEASUREMENT, AND SURFACE COMPOSITION

Create consistent layout conventions for components.

Address:

* minimum size
* preferred size
* fixed versus content-derived size
* width and height measurement
* padding and insets
* clipping
* truncation
* wrapping
* horizontal overflow
* vertical overflow
* responsive collapse
* empty areas
* parent surface inheritance
* component composition inside panels and overlays
* terminal resize
* Unicode grapheme and display width

Remove component-specific layout utilities when they represent a generally reusable layout concept. For example, action rows, right-aligned button groups, framed surfaces, form fields, and split regions should not each invent unrelated layout behavior.

Do not build a large general-purpose layout engine unless the current applications demonstrate the need. Prefer a small, consistent set of composable terminal layout primitives.

Components must behave safely when given empty or very small rectangles. They must not panic, underflow, write outside their area, or leave stale hit regions.

---

# 18. GENERIC COLLECTIONS AND DATA-HEAVY COMPONENTS

Lists, trees, tables, tabs, grids, property views, steps, pickers, completions, and similar components must share coherent collection concepts where appropriate.

Research and decide how to support:

* borrowed data
* owned data
* stable item keys
* custom row or cell rendering
* item metadata
* disabled items
* empty, loading, partial, and error states
* cursor versus selected value
* single and multiple selection
* range selection
* virtualization
* large data sets
* asynchronous or lazy data
* scrolling
* sorting requests
* filtering requests
* activation
* child actions
* reordering
* focus reconciliation

Do not force every collection into one giant abstraction when their semantics differ. Reuse common behavior and vocabulary without erasing meaningful differences.

## DataGrid boundary

Audit the current `DataGrid` especially carefully.

Its current implementation combines generic grid mechanics with database concepts such as:

* database cell types
* nullable columns
* primary keys
* references
* pending SQL changes
* SQL preview
* database filtering
* engine-oriented validation
* following foreign-key references

Separate general-purpose grid behavior from TablePro-specific database behavior.

The reusable layer may include generic:

* columns
* rows
* cells
* viewport
* cursor
* selection
* editing hooks
* validation hooks
* sorting and filtering requests
* custom cell renderers
* row state decoration
* action surfaces

Database schemas, SQL generation, foreign keys, nullable semantics, commit queues, and TablePro workflows belong in TablePro adapters, models, or composed domain components unless a concept is truly generic.

Prove this boundary by having TablePro retain all of its current capabilities while the reusable grid no longer requires database-domain knowledge.

---

# 19. FORMS AND TEXT EDITING

Create coherent APIs across:

* text input
* masked input
* textarea
* code editor
* editable cells
* selects
* choices
* chips
* validation
* help and error messages
* field labels

Evaluate whether a field wrapper or field composition should own common label, required, optional, help, error, and layout behavior rather than embedding it independently inside each control.

Support:

* navigation mode versus editing mode
* explicit begin, commit, cancel, and focus-loss transitions
* external value synchronization
* controlled values where required
* selection and cursor behavior
* paste
* Unicode graphemes
* horizontal scrolling
* validation hooks
* application-domain validation
* read-only and disabled distinctions
* secure masking
* no secret exposure through rendering, captures, logs, cloning, or `Debug`

Do not restrict extensibility to bare function pointers when closures, delegates, external validation, or another design would provide a substantially better API.

Do not let a render call commit or validate a value as an incidental side effect.

---

# 20. COMPLEX AND COMPOSED COMPONENTS

Audit components such as:

* dialogs
* menus
* pickers
* command palettes
* code editor plus completion
* data grid plus pending-actions surface
* diff viewer
* panels
* status bars
* hint bars
* tab workspaces
* split panes

Determine whether each should be:

* a primitive
* a headless behavior/state machine
* a styled component
* a composition of smaller components
* an application-specific composition
* multiple layers with convenience APIs

Avoid monolithic components that own unrelated data, layout, focus, domain behavior, and effects.

Avoid forcing applications to manually coordinate multiple low-level pieces when the coordination is generic and repeatedly required.

The showcase must demonstrate both:

* high-level ergonomic use
* lower-level composition and customization

---

# 21. PACKAGE AND MODULE ARCHITECTURE

Create a clear boundary between reusable code and applications.

The final organization must ensure:

* applications consume supported public APIs
* private internals cannot be used accidentally
* reusable code contains no TablePro or Jackin domain terminology
* application models and effects remain outside the generic library
* generic components can be tested independently
* examples compile as external-style consumers
* public modules are understandable and deliberately curated
* internal helpers are not exported merely because applications need them today

A Cargo workspace with separate application packages is acceptable and likely useful, but the final decision must follow the architecture research.

Preserve the runnable binary names:

* `showcase`
* `tablepro`
* `jackin-preview`

Preserve the current Rust edition and MSRV unless a change is strongly justified, documented, and verified.

Verify current dependency versions against their primary sources. Upgrade only when an upgrade materially supports the architecture or fixes a relevant issue. Do not combine the refactor with unrelated dependency churn.

Keep dependencies focused. Avoid adding a framework-sized dependency merely to replace a small amount of well-understood code.

---

# 22. APPLICATION MIGRATION

Migrate all three applications completely.

## 22.1 Showcase

Turn the showcase into the canonical component conformance laboratory and API documentation application.

For every public component, demonstrate where meaningful:

* default
* focused
* hovered
* focus plus hover
* pressed
* selected
* disabled
* read-only
* busy
* loading
* error
* warning
* editing
* empty
* overflow
* narrow terminal
* custom content
* custom variant
* global theme override
* local theme override
* per-instance part override
* truecolor
* 256 color
* 16 color
* no color

Add deterministic navigation to these states so captures and tests can reproduce them.

The showcase itself must use the same public API expected of downstream applications. It must not receive privileged access to library internals.

## 22.2 TablePro

Preserve TablePro’s current product capabilities and visual quality.

Move database-specific behavior out of generic components where necessary.

TablePro should own:

* database schema semantics
* SQL behavior
* query safety
* pending database changes
* connection models
* foreign-key behavior
* database validation
* TablePro-specific commands and workflows

The reusable system should own:

* generic controls
* interaction state machines
* focus and pointer mechanics
* overlays
* generic lists, trees, tables, grids, tabs, fields, editors, menus, and status surfaces
* theme and customization
* common layout and scrolling behavior

TablePro screens should become clearer examples of composition, with less low-level event and focus plumbing.

## 22.3 Jackin Preview

Preserve the deterministic Jackin product experience and the semantics established by the current source and interaction tests.

Migrate:

* host chrome
* menus
* tabs
* forms
* trees
* account and usage surfaces
* status bars
* hint bars
* pickers
* dialogs
* launch progress
* diff and inspection surfaces
* Capsule components
* scrollable and split surfaces

When Jackin requires a genuinely reusable primitive, implement it in the library.

When behavior is genuinely Jackin-specific, keep it in the application and compose it from public primitives.

Visual effects such as Jackin’s lifecycle rain or transition rendering may remain application-specific, but they must consume semantic theme APIs rather than assume Junie palette fields directly.

---

# 23. ERGONOMIC LITMUS TESTS

Use these scenarios to judge the architecture. These are acceptance tests, not prescribed syntax.

## Scenario A — Simple interactive screen

A new application author can build a screen containing:

* a text field
* a button
* a list
* a dialog

The author should not need to manually:

* create child hit regions
* inspect mouse coordinates
* maintain hover state
* maintain pressed state
* derive internal IDs
* implement Tab traversal
* implement modal focus trapping
* restore focus after dialog close
* place the text cursor
* determine which list row was clicked

The application should primarily handle semantic actions and domain state.

## Scenario B — Custom theme

A user creates a distinctly different theme by providing their own semantic colors and a small number of design choices.

All components render coherently without editing component source.

Color capability downgrade works for the custom theme.

## Scenario C — Local override

Two buttons using the same global theme appear in one screen.

One uses the default component recipe.

The other overrides a meaningful part or state locally without requiring a new global theme or a copied renderer.

## Scenario D — Custom collection content

An application supplies borrowed domain objects and a custom row or cell renderer to a list, table, picker, or grid.

The application does not need to convert the whole collection into duplicated owned strings on every frame.

Selection, focus, scrolling, and activation continue to work.

## Scenario E — Dynamic identity

Tabs or rows are inserted, removed, and reordered.

Focus, active selection, close actions, and pending edits remain associated with the logical item rather than a previous numeric position.

## Scenario F — Nested overlays

A dialog opens a picker or menu.

Only the topmost interaction is reachable.

Click, Esc, focus restoration, scrolling, contextual hints, and cursor behavior remain correct without application-specific barrier manipulation.

## Scenario G — Custom component authoring

A downstream author creates a new component using the public author-level primitives.

It participates in:

* theme resolution
* focus
* hover
* pressed state
* event dispatch
* hit testing
* cursor output
* overlays or scrolling where appropriate
* testing and visual capture

The author does not need private library access.

## Scenario H — TablePro adapter

TablePro composes a reusable grid with database-specific models and actions.

The generic grid source contains no requirement to understand SQL, primary keys, foreign keys, or database commits.

## Scenario I — Visual preservation

Switching the showcase, TablePro, and Jackin to the built-in Junie theme produces the approved current visual language and state semantics.

---

# 24. DOCUMENTATION

Update or create:

* `README.md`
* `DESIGN.md`
* `COMPONENT_ARCHITECTURE.md`
* public Rust documentation
* component-author documentation
* downstream-user quick start
* theme customization guide
* component override guide
* migration mapping from the old experimental API
* standalone examples or an external-style consumer example

For every public component, document:

* purpose
* normal construction
* data and state ownership
* variants
* emitted actions
* focus behavior
* keyboard behavior
* mouse behavior
* layout behavior
* theme parts
* customization examples
* important invariants

Include concise, compiling examples for at least:

1. A default button.
2. A complete custom theme.
3. A partial theme override.
4. A globally customized component recipe.
5. A per-instance part override.
6. A text field with external validation.
7. A list or table with custom domain rows.
8. Dynamic tabs with stable identity.
9. A composed dialog.
10. A nested menu or picker.
11. A small application that relies on shared focus and event dispatch.
12. A custom downstream component.

Use doctests or compile-checked examples where practical.

Documentation must describe the implementation that actually exists. Do not leave aspirational architecture in the docs.

---

# 25. TESTING STRATEGY

Add tests at multiple levels.

## 25.1 Unit tests

Cover:

* component state transitions
* edit begin, commit, cancel, and focus loss
* event consumption and invalidation
* stable identity
* focus traversal
* nested focus scopes
* focus restoration
* disabled and read-only behavior
* hit ordering
* pointer press and release
* drag capture
* nested scrolling
* overlay stacking
* theme inheritance
* theme override precedence
* style patch semantics
* color downgrade
* Unicode graphemes and display widths
* small and empty rectangles
* dynamic data reconciliation
* secret redaction

## 25.2 Shared conformance tests

Create reusable contract tests for interactive component families where appropriate.

Examples include:

* disabled controls cannot activate
* keyboard and mouse activation emit equivalent semantic actions
* focused controls participate in the correct traversal order
* hover does not steal focus
* rendering twice with unchanged inputs is semantically stable
* rendering does not commit or cancel edits
* component areas remain clipped
* no-color output retains state indicators
* local overrides do not mutate the global theme

## 25.3 Rendering tests

Use Ratatui `TestBackend`, snapshots, or the repository’s established baseline approach.

Test:

* representative component states
* multiple viewport sizes
* default Junie theme
* the custom non-Junie theme
* local overrides
* truecolor, 256, 16, and no-color modes
* narrow and overflow behavior
* modal and nested-overlay composition

Review snapshot changes rather than blindly accepting them.

## 25.4 Application integration tests

Retain and migrate all current application tests.

Add coverage for:

* complete showcase navigation
* TablePro editing, results, tabs, grids, dialogs, and menus
* Jackin host-to-Capsule journeys
* keyboard-only flows
* mouse flows
* resizing
* focus restoration
* custom theme injection
* no regressions in deterministic scenarios

## 25.5 Architecture checks

Add deterministic checks that prevent regression into the old architecture.

Where appropriate, verify that:

* application packages do not import private implementation modules
* application directories do not contain generic component copies
* generic library code does not depend on TablePro or Jackin domain modules
* literal palette values remain confined to built-in theme definitions or explicit theme fixtures
* semantic render operations do not require caller-supplied raw background colors without justification
* legacy `owns`/`locate` routing patterns are not scattered through applications
* public components have documentation
* all examples compile

Prefer architectural boundaries enforced by Rust packages and visibility over fragile text-based checks. Use text checks only for invariants the compiler cannot express.

## 25.6 Performance checks

Measure representative render and interaction paths before and after the refactor.

Pay attention to:

* large lists, trees, tables, and grids
* per-frame allocations
* cloning of rows or strings
* ID lookup complexity
* overlay dispatch
* style resolution
* Unicode processing

Do not prematurely optimize every path, but do not accept an architectural design that requires copying entire application data sets or performing avoidable full-tree scans on every event.

---

# 26. QUALITY GATES

After the final package or workspace structure is established, run the strongest applicable equivalents of:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo build --workspace --all-targets --all-features
```

Also:

* build all three binaries
* run all three binaries
* exercise keyboard navigation
* exercise mouse click, hover, wheel, and drag
* resize through supported viewport sizes
* run deterministic capture scenarios
* inspect actual rendered output
* compare the default Junie output with the baseline
* inspect the non-Junie theme across the full catalog
* inspect local overrides
* run any repository-specific stronger checks

Add or update CI so the final deterministic formatting, lint, test, documentation, and build gates run automatically, unless repository policy explicitly forbids CI changes.

Do not suppress warnings broadly.

Do not add unexplained lint exceptions.

Avoid `unsafe`. Any unavoidable `unsafe` requires a written safety rationale and focused tests.

---

# 27. IMPLEMENTATION STRATEGY

Use vertical slices rather than a single uncontrolled big-bang rewrite.

## Slice 1 — Baseline and audit

* capture current behavior
* complete the component inventory
* map application usage
* identify API inconsistencies
* identify domain leakage
* identify reusable controls hidden in applications
* complete external research

## Slice 2 — Architecture and representative prototype

Design the proposed architecture and test it on a representative set:

* Button
* a field and TextInput
* List or Tree
* Tabs
* Dialog or Menu
* one scrollable component
* theme customization
* application event dispatch

Migrate a small showcase surface to the proposed API.

Write representative downstream usage examples.

Have a fresh, read-only `opus-analyst` API reviewer critique:

* ceremony
* naming
* ownership
* lifetime complexity
* customization
* testability
* event flow
* component-author experience

Revise the architecture before expanding it.

Do not continue with an awkward API merely because code has already been written.

## Slice 3 — Foundations

Implement and stabilize:

* identity
* events and semantic actions
* interaction registration and dispatch
* focus scopes
* pointer handling
* scrolling
* overlays
* cursor output
* surface inheritance
* theme resolution
* style overrides
* layout primitives
* component-author API

Keep the repository compiling and tested throughout.

## Slice 4 — Component families

Migrate coherent families, continuously updating showcase pages and tests:

* buttons and choices
* fields, inputs, textarea, select, and chips
* lists, trees, tables, props, and steps
* tabs and navigation
* panels, splitters, scrollbars, and viewports
* dialogs, menus, pickers, completion, and popups
* progress, empty states, hints, segments, brand, and status bars
* code editor and diff viewer
* generic grid and TablePro database adapter

After each family, have a fresh, read-only `opus-analyst` review API consistency. Fable applies verified corrections. Consolidate repeated concepts rather than reproducing inconsistencies in a new namespace.

## Slice 5 — Showcase migration

* migrate every page
* add custom theme coverage
* add local override coverage
* add author-level custom component example
* complete conformance captures
* remove privileged access to internals

## Slice 6 — TablePro migration

* migrate all screens
* move database semantics to application-level adapters
* remove manual generic interaction routing
* preserve workflows and visual behavior
* run full TablePro interaction tests

## Slice 7 — Jackin migration

* migrate all reusable surfaces
* preserve product semantics and deterministic scenarios
* remove generic interaction duplication
* run complete keyboard and mouse journeys
* inspect responsive and overlay behavior

## Slice 8 — Cleanup and independent verification

* remove all legacy APIs
* remove dead code
* remove compatibility shims
* tighten visibility
* update documentation
* regenerate only reviewed baselines
* run full quality gates
* run a fresh, read-only `opus-analyst` architecture review
* run a separate fresh, read-only `opus-analyst` visual review
* use Fable to correct every material issue found

At the end of every slice:

1. format
2. compile
3. lint affected targets
4. run affected tests
5. run representative application flows
6. inspect rendered output
7. correct regressions before continuing

---

# 28. SUBAGENT RESPONSIBILITIES

Every responsibility in this section is mandatory `opus-analyst` work using `claude-opus-5` at effort `high`. Spawn fresh agents for independent reviews. Opus agents remain read-only; Fable records their findings and performs every repository mutation.

At minimum delegate independent work for:

## Current API audit

Inventory every component API, state model, event shape, render signature, extension point, and application usage.

## Rust API research

Compare idiomatic Rust component, ownership, generic, trait, and public-API alternatives using current primary sources.

## Theme architecture

Design and critique semantic tokens, recipes, variants, parts, style patches, inheritance, capability downgrade, and user customization.

## Interaction architecture

Audit and design identity, event dispatch, focus scopes, hit testing, hover, pointer capture, scrolling, cursor output, keymaps, and invalidation.

## Overlay architecture

Audit dialogs, menus, selects, pickers, completion, context menus, focus trapping, stacking, anchoring, and dismissal.

## Complex-component and domain-boundary audit

Analyze grid, table, code editor, diff, status bars, and application-specific components. Identify what is generic, composed, or domain-specific.

## Application migration audit

Map showcase, TablePro, and Jackin use of components, direct drawing, manual routing, and duplicate abstractions.

## Performance and ownership review

Inspect allocations, cloning, large collection behavior, IDs, lookup complexity, and borrowing ergonomics.

## Visual QA

Compare default-theme output before and after, inspect the custom theme, and review focus, hover, selection, editing, spacing, hierarchy, overflow, and responsive layouts.

## Independent verifier — fresh Opus agent

Review the final repository without assuming the primary implementation is correct. Attempt to find:

* inconsistent APIs
* hidden legacy paths
* missing component migration
* theme leaks
* domain leaks
* render-time semantic mutation
* focus or overlay bugs
* awkward downstream usage
* undocumented public APIs
* visual regressions
* tests that pass without proving the requirement

The primary agent must synthesize the findings and fix verified issues.

---

# 29. DEFINITION OF DONE

This goal is complete only when all of the following are true:

* the complete current component system has been audited
* the architecture is documented
* the architecture was independently reviewed
* the implementation matches the documented architecture
* every current component was migrated, decomposed, replaced, or deliberately removed
* no legacy component API remains
* no compatibility facade remains
* the default Junie theme preserves the approved visual language
* a clearly different custom theme works across the complete catalog
* partial theme overrides work
* global component customization works
* scoped customization works
* per-instance and logical-part overrides work
* truecolor, 256-color, 16-color, and no-color handling work
* component state remains readable without relying only on color
* component APIs are consistent and documented
* semantic component actions use one coherent model
* render operations do not perform hidden semantic transitions
* dynamic item identity remains stable
* application code no longer manually reproduces routine focus, hover, pressed, hit-test, cursor, and child-routing mechanics
* nested overlays, focus trapping, and focus restoration work
* generic collection APIs support custom domain data and rendering
* the generic grid no longer requires database semantics
* TablePro retains its database capabilities through application-level composition or adapters
* Jackin retains its complete deterministic product behavior
* all three applications consume supported public APIs
* the reusable library contains no TablePro or Jackin domain dependencies
* the showcase demonstrates every public component and customization layer
* standalone or external-style examples compile
* all public APIs have rustdoc
* all application and component tests pass
* formatting and clippy pass with warnings denied
* documentation builds with warnings denied
* all binaries build and run
* keyboard-only workflows work
* mouse workflows work
* scrolling and dragging work
* supported resize states work
* visual captures have been reviewed
* intentional baseline differences are documented
* secret-bearing controls do not expose raw values through debug output, captures, or logs
* no material TODO, stub, placeholder, dead legacy module, or unimplemented normal path remains

The quality target is:

> A Rust developer can build a premium terminal interface by composing these components without repeatedly debugging focus, hover, cursor, selection, scrolling, overlays, and styling—and can still replace the complete visual identity or precisely customize one component without fighting the library.

---

# 30. FINAL REPORT

When implementation is complete, provide a concrete final report containing:

1. The final package and module architecture.
2. The central architectural decisions and their rationale.
3. Important alternatives rejected.
4. The new state, render, event, identity, focus, overlay, and theme models.
5. A concise old-to-new API mapping.
6. Representative downstream usage examples.
7. Proof of full component coverage.
8. Proof that showcase, TablePro, and Jackin use the public API.
9. Proof of the custom theme and local overrides.
10. Intentional visual changes.
11. Tests and exact commands run.
12. Captures or scenarios reviewed.
13. Performance findings.
14. Independent-review findings and corrections.
15. Any remaining trade-offs that are inherent rather than unfinished work.

Do not finish with only an architecture proposal.

Research it.

Design it.

Prototype it.

Challenge it.

Refactor it completely.

Migrate every consumer.

Run it.

Interact with it.

Theme it.

Capture it.

Review it independently.

Remove the old architecture.

Deliver the finished reusable Rust TUI component system.
