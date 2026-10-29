# Requirements

This is the conceptual acceptance surface shared by all eight branches. Implementation details and evidence remain branch-specific. These requirements are not a checklist of completed behavior: a compile, unit test, or single capture demonstrates only the code path, configuration, state, and dimensions it actually covers.

## Shared library, components, and runtime

### State, identity, and collections

Validate fallible mutations before committing them. Invalid input must leave the old state intact and expose a useful error. Batch updates should be atomic where a partial update would violate a component invariant. Audit every public mutation path, including setters and collection access; add transactional setters only when a real consumer needs them and the existing API cannot preserve invariants.

Selection, cursor, focus, and expansion should follow stable item identity when data is sorted, filtered, resized, or replaced. If an item disappears, use a documented fallback rather than retaining an invalid index. Cursor movement must not silently replace the selected set. Shift range extension takes precedence over platform toggle modifiers; Select All filters out nonselectable rows. Context-selecting an already selected row preserves the set, while selecting an unselected row changes it deliberately.

Metadata, status, checked state, and markers must remain visible without displacing essential labels or changing row hit targets. Collection and table layouts must account for hidden/dropped columns, preserve access to their meaning, and reveal useful information predictably on hover or focus. Schema ingestion validates input and exposes errors without partially corrupting rows, column identities, or selection.

### Input, focus, and semantics

Disabled components ignore keyboard, pointer, hover, and activation input. Read-only components remain navigable and copyable where appropriate but reject mutations; their visual state must remain distinguishable from disabled and editable states. Modal input takes precedence over background components, and closing a modal restores focus to a sensible owner. Modal focus barriers restrict reachable controls to the active modal slice, and Tab/BackTab wrap within it. Popup dismissal routes to the configured cancel action. Outside-click handling belongs to the popup owner through a backdrop hit target, not duplicated page logic.

Forward unhandled keys and modifiers so parent routing remains possible. Character keys, chords, and modifier combinations must match what supported terminals can deliver. Keyboard range selection must support creation, extension, contraction, and anchor behavior in text viewports and collection controls. Mouse hover, drag, and activation must use correct cell ownership, including seams, borders, scrollbars, and overlays. Test modified chords and modal lifecycle in real applications before introducing shared abstractions that could alter routing.

Editing and selection operate on grapheme boundaries. Text areas and inputs must own focus and hit regions consistently; form navigation must not activate a neighboring field. Copy and paste routes are consistent across editor, text viewport, and dialogs; modal fields receive paste before the page beneath them. Backspace and paste update find/query fields using the same editing contract as ordinary input. Query parsing must distinguish scalar replacement from append behavior according to the owning control. Clipboard success, failure, unsupported terminal capability, and cancellation are reported truthfully.

Exercise the [product interaction policies](product.md#interaction) through real consumers: page-first key routing, copying an existing selection before quit handling, clean and dirty leaves, running-task guards, and configurable dismissal. Verify both consumed and unhandled keys so text focus alone does not accidentally swallow configured quit behavior.

### Rendering, accessibility, and APIs

Render empty, loading, unavailable, error, focused, selected, hovered, pressed, disabled, and read-only states deterministically. Ensure focus is visible, keyboard navigation has no trap, status is understandable without color alone, and hints do not promise unavailable keys. Meet the product's contrast, marker, overlay, and color-level requirements. Scrollbars, borders, and markers must not consume neighboring cells or create inconsistent hit regions.

Keep component APIs composable across controls, fields, feedback, navigation, lists, trees, tables, grids, editors, forms, and overlays. Expose override points for behavior consumers reasonably vary, with sensible application/profile defaults and per-component configuration beneath them. User remapping is a later input-policy layer. Prefer a concrete consumer and behavior test over speculative traits, glyph abstractions, or generic callback surfaces. Preserve compatible public APIs unless an explicit migration includes downstream updates, rationale, and evidence.

Required behavior includes identity-safe tab close/switch and documented fallback: closing a tab must not redirect to a different tab merely because its numeric index was reused. DataGrid must enter read-only state or validate before mutation can escape; a ListBox batch update must preserve model and selection invariants atomically. Show checked rows, schema metadata, and validation state without losing keyboard or pointer ownership. New configurable close-target policies or generalized atomic-mutation APIs are optional API expansions, deferred until a concrete consumer demonstrates the need; they do not defer these baseline invariants.

## Applications and journeys

### Showcase

Showcase is the executable catalog of component behavior. Make each component, meaningful variant, state, and supported configuration reachable through real interactions. Organize demonstrations into shell, inputs/forms, tables, navigation, overlays, feedback, layout, text/code, and application-screen areas. A page shows components in realistic context and exposes relevant options and states, rather than implementing its own rendering or interaction logic.

Every route must be reachable by keyboard and pointer. Handle direct navigation, back/forward movement, resize, narrow terminals, stale events, empty data, and failed actions. Cover shell and navigation behavior as well as individual components. Demonstrate changed library capability in the same integration slice that adds it. Track any capability that cannot yet be represented with a concrete explanation rather than silently omitting it.

Verify deterministic colors and output under full, reduced, and monochrome themes. Capture representative resting, focused, selected, hovered, pressed, disabled, modal, error, and narrow-layout states. Measure startup and interactive workloads, including idle and dirty/repaint-heavy cases, with reproducible fixtures and provenance. Include focused regressions for editor, list, tree, table, grid, popup, and selection behavior.

### Holla

Current Holla applications are deterministic simulations and do not perform real host side effects. These journey requirements describe what the interface and simulation must demonstrate; they do not assert a production filesystem or process adapter.

Cover project, file, repository, and tool discovery; filtering and search; selection and detail inspection; Git/repository status and operation previews; build and task selection; service/container and cleanup screens; configuration and trust state; learning/usage views; logs, output, errors, and result navigation. File and folder flows include preview, open, reveal, and safe navigation. Repository flows include status, worktrees, mirrors, and guarded actions. Task flows cover projects, builds, services, containers, IDEs, and package managers as simulated states.

Model jobs as stateful workflows: queued/running/paused/stopped/completed/failed/cancelled states; progress and status messages; streamed output and stdin where relevant; inspection; retry or follow-up navigation; and cleanup on cancellation. Terminal ownership must be explicit across foreground jobs, suspend/resume, resize, and exit, with restoration on every exit path. Disk views include scan progress, tree/top-file summaries, and insights. Cleanup/deletion shows dry-run or preview, confirmation, recovery/error behavior, and log navigation. Configuration and trust flows explain scope and effects.

The CLI parses supported scenario/navigation/configuration options, rejects invalid combinations with actionable errors, and preserves deterministic behavior. Capability boundaries make unsupported platform operations clear rather than implying success. Paths, workspace boundaries, secrets, and user data require explicit privacy and safety treatment in any future real adapter.

Any future real filesystem, repository, process, Cargo/build, package-manager, service, container, cache, logging, or configuration adapter is a separate scope. It needs explicit user consent where appropriate, capability checks, narrow path/process scope, cancellation, cleanup, privacy review, failure semantics, and isolated host integration tests before it can be described as supported.

### Jackin Preview

Exercise account and identity states; provider configuration and validation; usage and activity; workspace and instance state; cockpit controls; capsule selection, launch, and lifecycle; and transitions between cockpit and capsule views. Its account/editor journey covers role assignment, allow/default policy, environment-authentication masking and scope, credential re-selection, isolated/read-only mode, add/remove/undo behavior, save preview then save, and a dirty-state leave prompt. Cover empty accounts, incomplete configuration, provider errors, quota/usage states, launch failure, cancellation, and recovery.

Exercise adaptive resource discovery with stable action identity; search, edit, and learn flows; file/folder preview, open, and reveal; Git status, worktrees, and mirrors; project/build/service/container/IDE/package-manager task journeys; output streaming, stdin, cancellation, and terminal ownership; disk scan/tree/top-file/insight views; and guarded cleanup with dry-run, recovery, and log inspection. Preserve brand-specific transitions while keeping focus, routing, dialogs, status, and rendering shared. Simulation must show state, progress, output, and outcomes without claiming a real provider, secret store, or host service was contacted. Do not expose credentials in captures or diagnostics.

### TablePro

Exercise connection selection and configuration, connection success/failure, schema and table discovery, table and row inspection, query editing, query validation/execution, results, filtering, sorting, selection, and detail navigation. The query surface includes cancel, run, run all, explain, analyze, create query, close/switch query, tab listing, history, safe mode, commit, find, and maximize. SQL classification ranks read/write decisions; safe mode denies writes and other writes require explicit confirmation.

Browse, filter, and inspect schema/row grids while preserving stable row and column identity; reveal dropped-column metadata predictably. Use shared Grid, Tabs, Select, Form, ChipBar, and Code-viewer behavior for interaction, while app-owned SQL/catalog/filter semantics stay in the application domain. Editing preserves schema/type constraints and provides clear validation. Cover empty schemas/results, large result sets, stale selection after filtering/sorting, query errors, cancellation, and keyboard/mouse traversal.

Demonstrate export and transaction state as deterministic simulated behavior unless a separately reviewed database adapter exists. Distinguish draft, pending, committed, rolled-back, and failed states. Never imply an external database side effect when none occurred. Keep SQL/domain state separate from reusable table, editor, form, and navigation behavior.

### Cross-application integration

Treat all four applications as consumers of shared components. Test app-level key overrides and shell behavior where each app differs; do not assume the same quit key, chord, or page order everywhere. Verify real journey sequences so focus, modal routing, selection identity, clipboard, status, resize, and terminal lifecycle work together. A page-specific workaround is evidence of a shared component or routing gap, not an accepted second implementation.

## Verification and evidence

For a behavior change, combine focused unit tests with a real consumer regression. Cover pointer and keyboard paths, modifiers, first-click focus versus activation, modal open/close, cancellation, stale results, errors, and resize as relevant. Test viewport mutation/cache behavior when APIs can invalidate layout or identity. Async cancellation and stale-result tests are required where a real async consumer exists; do not create speculative traits for hypothetical consumers.

Visual evidence names application, route, state, dimensions, color level, focus/hover/pressed/disabled condition, and source revision. Complete frames include cursor placement and all rendered cells; tests should inspect semantic state/effects as well as pixels/ANSI output. Direct and PTY journeys cover actual input, not just function calls. Capture-to-ANSI round trips preserve colors, cursor placement, and screen contents. A golden image or approved baseline applies only to its named scenario and dimensions.

Operational report publication must preserve source and receipt identity while avoiding a second maintained documentation tree. Migrate historical publisher cursor and output-path contracts deliberately before integrating them with the replacement documentation system; local report rendering alone does not migrate a remote publishing protocol.

Performance evidence uses production-representative workloads and records allocation/byte/work bounds alongside controlled timing uncertainty, environment, and baseline. CLI verification covers supported flags, defaults, invalid arguments, deterministic output, and relevant exit status. Portability work names supported OS/terminal capabilities and tests fallbacks. Evidence is reproducible and traceable; historical passes do not verify current source. API changes require source compatibility review and compilation of real consumers.

## Deferred proposals and external dependencies

Possible additions such as safer table close/target-identity policies, additional atomic ListBox/DataGrid APIs, checked-row interaction variants, alternative paste routing, expanded copy semantics, additional read-only policies, broader Showcase states, cross-app resize/render/performance proof, and new component abstractions remain unselected until a concrete consumer and acceptance contract justify them.

Possible host adapters for filesystem, Git, Cargo, Docker, Homebrew, Gradle, IDE integrations, package managers, PTYs/processes, configuration, trust, caches, Trash, logging, and CLI workflows are future operational capabilities. They require isolated host probes and explicit safety/privacy boundaries; current deterministic applications do not prove them.

Upstream Velnor workflow generation and Tuiscotti terminal testing are external dependencies, not current guarantees. Desired workflow properties include deterministic named profiles and real partitions, correct triggers and artifact upload conditions, timeouts and resource hints, required-job membership, strict key validation, byte-compatible defaults, stable toolchain discovery, and one publish gate. Desired terminal-test support includes key location/wait/mismatch diagnostics, compatibility with current terminal libraries, ANSI cursor round trips, and avoiding absolute paths in gated artifacts. These become project claims only after upstream release, any required pin update, integration, and verification here.
