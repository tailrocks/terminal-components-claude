# Jackin Reference Research Goal

You are a principal product researcher, terminal-UX architect, information architect, interaction-system analyst, and senior Rust/Ratatui engineer.

Your mission: perform a complete, source-backed analysis of the currently implemented Jackin terminal experience and produce exactly one canonical Markdown file:

`JACKIN_REFERENCE.md`

The file must describe every operator-visible Jackin surface, screen, tab, pane, overlay, dialog, picker, popup, workflow, state, action, datum, and transition that can be established from the approved remote sources below.

This is product archaeology and reference writing. Do not design, implement, recreate, or modify Jackin or the Junie design system.

## 1. Source boundary: remote locations only

Use only these four approved remote source locations:

1. Jackin repository, `main` branch:
   `https://github.com/jackin-project/jackin`
2. `terminal-components-claude`, `jackin` branch:
   `https://github.com/donbeave/terminal-components-claude/tree/jackin`
3. Junie prompt 1:
   `https://raw.githubusercontent.com/donbeave/terminal-components-claude/refs/heads/jackin/JUNIE_PROMPT1.md`
4. Junie prompt 2:
   `https://raw.githubusercontent.com/donbeave/terminal-components-claude/refs/heads/jackin/JUNIE_PROMPT2.md`

The numbering above is intentional: there are four sources, not three.

Interpret source scope as follows:

- Source 1 authorizes the contents of the Jackin repository at its current remote `main` revision.
- Source 2 authorizes the contents of the named `terminal-components-claude` remote `jackin` branch.
- Sources 3 and 4 authorize the contents of those exact raw Markdown documents.
- Files, paths, symbols, tests, fixtures, documentation, source links, and history reachable inside the two approved repository roots may be examined only as part of those roots.

Hard boundary:

- Do not use a local Jackin checkout.
- Do not use a local `terminal-components-claude` checkout as evidence.
- Do not clone, fetch, pull, checkout, switch branches, reset, clean, delete, or repair any repository.
- Do not run local cleanup or local source discovery.
- Do not inspect local files, local tests, local snapshots, local baselines, local documentation, or local build output as research evidence.
- Do not run Jackin or any other local application for behavioral verification.
- Do not use GitHub API URLs, search-engine results, package registries, other websites, other repositories, linked pages outside the four approved roots, or remembered product behavior.
- Do not follow a citation to a remote location outside the four approved roots.
- Do not invent source files, symbols, line ranges, labels, states, providers, or capabilities.

This restriction must be stated prominently in the resulting reference. The reference must clearly say that all research evidence came from the approved remote locations and that no local cleanup, checkout, clone, or local repository inspection was used.

If remote access is unavailable, do not substitute local files or outside sources. Record the missing evidence as `UNKNOWN`, state which approved source was unavailable, and continue with the other approved sources.

## 2. Deliverable boundary

Create and return exactly:

`JACKIN_REFERENCE.md`

The document title must be:

`# Jackin Current Product, Interface, and Workflow Reference`

The final working-tree change for this task must contain only `JACKIN_REFERENCE.md`. Do not create notes, scripts, screenshots, fixture files, rendered artifacts, source copies, or a reference application. If temporary in-memory notes are used, do not persist them.

Do not modify either remote repository. Do not modify any local repository content other than the required output file.

The output is documentation only. It must not contain Rust implementation, copied Jackin rendering code, a Jackin replica, a preview application, or a proposed future layout.

## 3. Research contract

The reference is the canonical scope map for a later agent that may build a completely redesigned Jackin TUI using the approved Junie-inspired design system described by Sources 2–4.

The document must answer:

> What can an operator currently see and do across the complete Jackin terminal experience, how do the interfaces and workflows relate, what data is presented in each state, and where is each behavior implemented in the approved remote source?

Use evidence labels:

- `SOURCE_VERIFIED` — directly established by current source.
- `TEST_VERIFIED` — established by an approved repository test or fixture.
- `DOC_VERIFIED` — established by approved repository documentation or Junie prompt.
- `RUNTIME_VERIFIED` — use only if the approved remote source itself contains an explicit runtime capture or transcript. Do not run local software.
- `PARTIALLY_IMPLEMENTED` — some visible or behavioral path exists, but not the full capability.
- `PLANNED_ONLY` — a plan or roadmap says it should exist, but current implementation does not establish it.
- `RESEARCH_ONLY` — research material, not shipped behavior.
- `INFERRED` — reasoned from source but not directly stated; explain the inference.
- `UNKNOWN` — evidence is insufficient.

When sources disagree:

1. Prefer implemented Jackin source and tests over documentation intent.
2. Describe the disagreement.
3. State the currently evidenced behavior.
4. Never silently turn a plan into a shipped feature.

Separate product semantics from presentation. Current layouts, colors, borders, glyphs, panel arrangement, shortcut placement, and navigation structure are evidence of the current product, not requirements for the future design.

## 4. Snapshot and citation rules

At the beginning of the document record:

- approved source locations used
- repository and branch/ref for each source
- remote revision or commit identifier if exposed by the approved repository view
- source observation date: `2026-09-02` unless the actual research date differs
- Jackin version, if discoverable inside Source 1
- source limitations and unavailable approved locations
- explicit statement: `Remote-only research; no local cleanup, checkout, clone, or local repository inspection was used.`

Do not claim an exact SHA unless it was actually visible in an approved remote location. Do not fetch an API endpoint merely to obtain a SHA. If Source 1 is mutable `main`, record that fact and the observed remote revision information available in the repository interface.

Every substantive claim must cite one or more of the four approved remote roots. Use this form:

```markdown
Source:

- [R1] `crates/.../path.rs`, symbol `Type::method`, lines `120–188` if visible in the approved remote source — why it matters.
- [R2] `DESIGN.md`, section `...` — why it matters.
- [R3] `JUNIE_PROMPT1.md`, section `...` — why it matters.
```

Rules:

- Use remote links rooted in the four approved locations only.
- Use repository-relative paths, symbols, headings, and exact remote line ranges when available.
- Never cite a local filesystem path.
- Never cite a mutable or external source outside the approved roots.
- Never invent line ranges. If the remote interface does not expose stable line numbers, use the exact path, symbol, heading, or code block anchor and say that line numbering is unavailable.
- Do not paste large source blocks. Summarize behavior and cite the source.

## 5. Investigation method

Perform read-only remote investigation. Parallelize independent remote reading when possible. Each investigation stream must return structured findings with:

- finding
- evidence class
- approved source ID
- repository-relative path or document heading
- symbol or section
- exact line range when available
- implications for the operator-visible experience
- unresolved uncertainty

Build the inventory from the union of evidence found in:

- application route, stage, screen, tab, modal, dialog, picker, overlay, launch-stage, and visible-state enums
- keymap/action enums and input-dispatch branches
- render-dispatch branches and render/view functions
- modal constructors and `open_*` call sites
- message, effect, subscription, and background-event handlers
- focus targets, hover targets, mouse hit regions, and scroll registries
- tests, snapshots, fixture variants, and visual-baseline registries
- command entry points and approved repository documentation
- Sources 3 and 4, only for the future implementation context and methodology

Search concepts such as `screen`, `surface`, `stage`, `route`, `view`, `render`, `tab`, `pane`, `modal`, `dialog`, `popup`, `overlay`, `picker`, `prompt`, `menu`, `footer`, `status`, `progress`, `error`, `help`, `usage`, `launch`, `console`, `capsule`, `hardline`, `eject`, and `exile`.

Do not stop at obvious screens. Every discovered visual variant must be documented, explicitly classified as non-operator-visible, or listed as an unresolved discrepancy.

Use these repository-relative paths as remote starting points, not as an exhaustive list. Resolve them only inside Source 1’s approved Jackin repository root and follow imports/call chains there:

- Host console: `crates/jackin/src/console/adapter/run.rs`, `crates/jackin-console/src/tui.rs`, `crates/jackin-console/src/tui/model/`, `state/`, `update/`, `input/`, `keymap.rs`, `focus.rs`, `layout.rs`, `view/`, and `components/`.
- Console screens: `crates/jackin-console/src/tui/screens/workspaces/`, `editor/`, `settings/`, `usage.rs`, `usage/`, and `edit_save/`.
- Console modal system: `crates/jackin-console/src/tui/model/modal.rs`, `model/modal/`, and modal-related components/call sites.
- Launch cockpit: `crates/jackin-launch/src/tui/`, `components/`, launch model/state types, orchestration, tests, and baselines.
- Capsule: `crates/jackin-capsule/src/tui/`, `components/`, `daemon/`, `input/`, `keymap/`, `layout/`, session/control-plane types, tests, and baselines.
- Usage: `crates/jackin-usage/src/`, provider modules, `host/`, `crates/jackin-protocol/src/usage_broker.rs`, usage control types, console projection, capsule status/dialog integration, tests, and end-to-end tests.
- Product/UI documentation: `docs/content/reference/tui/`, `docs/content/reference/capsule/`, and relevant command, Workspace, Role, Auth, Usage, launch, Hardline, Eject, and Exile documentation.

At minimum, divide the remote investigation into these independent workstreams when parallel research is available: product model/vocabulary; host-console topology; Workspace manager; Workspace editor; Settings; dialogs/pickers; Usage/providers; launch; capsule; interaction; visual semantics; workflow synthesis; and completeness audit. The final synthesis must reconcile all streams.

## 6. Required reference structure

Use a linked table of contents and stable IDs. Keep the following major sections, in this order.

### Document Contract

Explain:

- purpose and audience
- exact remote-only source boundary
- explicit no-local-cleanup/no-checkout/no-clone rule
- approved source IDs R1–R4
- what the document covers and excludes
- snapshot/revision limits
- evidence classes
- how a future implementation agent should use the reference
- that the future design may change presentation while preserving semantics and capability

Include this exact statement:

> This document describes current product semantics and capabilities. It does not require the future design to preserve current layouts, styling, panels, colors, or navigation structure.

### Snapshot and Coverage Summary

Record snapshot metadata and derived counts. Include, when discoverable:

- top-level terminal surfaces
- routes and screens
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
- visual-baseline states

Do not invent counts. Explain how each count was derived and mark unavailable counts `UNKNOWN`.

### One-Page Product Explanation

Explain Jackin from the operator’s perspective:

- problem and promise
- primary operator
- host, Construct, Workspace, Role, Agent/runtime, instance, and session relationship
- isolation model
- primary lifecycle
- where configuration occurs
- where work occurs
- where usage comes from
- how an operator returns to running work

Avoid implementation detail unless it explains visible behavior.

### Canonical Vocabulary

Create:

| Term | Exact current meaning | Scope | Where visible | Evidence | Potential confusion |
|---|---|---|---|---|---|

Investigate at least Operator, Construct, Workspace, Role, Agent, Runtime, Session, Instance, jacking in, Hardline, Eject, Exile, Mount, Environment, Secret, Auth, Trust, Provider, Account, Usage, and Capsule. Discover additional terms. Preserve exact spelling and capitalization. Distinguish domain terms, command names, UI labels, metaphors, and internal names.

### Domain Model and Scope

For each UI-relevant entity document identity, visible fields, relationships, cardinality, ownership, scope, lifecycle, persistence, valid states, actions, errors, source types, and source references.

Include a Mermaid or compact ASCII relationship diagram plus readable prose. Explicitly answer:

- what is global
- what is Workspace-specific
- what is Role-specific
- what is Agent-specific
- what is instance/session-specific
- what belongs to a provider account
- what survives exit
- what exists only for a running Construct
- what can have multiple simultaneous instances

### Complete Application and Surface Topology

Create a graph of the real current terminal journey. Distinguish full-screen routes, embedded panes, tabs, inline pickers, global overlays, modal dialogs, temporary status surfaces, progress surfaces, in-Construct surfaces, CLI-to-TUI transitions, TUI-to-capsule transitions, reconnect/return paths, and lifecycle exits.

At minimum connect host console, Workspace manager, Workspace editor, Settings, Usage, launch cockpit, capsule multiplexer, help/debug overlays, and lifecycle exits. Represent transitions, not disconnected screenshots.

### Surface Inventory

Assign a stable ID to every operator-visible surface. Use IDs consistently in all later sections and workflows. Suggested naming only:

- `CONSOLE-WORKSPACES`
- `EDITOR-GENERAL`
- `SETTINGS-TRUST`
- `MODAL-FILE-BROWSER`
- `LAUNCH-PROGRESS`
- `CAPSULE-USAGE`

Use:

| ID | Surface | Type | Entry | Primary purpose | Main data | Opens | Evidence/source |
|---|---|---|---|---|---|---|---|

Include screens, panes, tabs, overlays, dialogs, pickers, popups, menus, status surfaces, and progress surfaces. Do not omit a variant because it shares a renderer.

## 7. Per-surface specification

For every full screen, pane, tab, overlay, dialog, picker, popup, status surface, and progress surface, create a section:

`<ID> — <Surface name>`

Include all of the following.

### Classification

- type and product area
- evidence class
- current/shipped/partial/planned status
- operator-visible versus internal-only determination

### Operator purpose

What is the operator trying to understand or accomplish?

### Entry, exit, and destinations

Document opening action, prerequisites, source state, command/key/action, every normal exit, cancellation, confirmation, error exit, and next surface.

### Layout and complete visible content

Describe each region and its purpose: header, identity/context, navigation, sidebar, panes, tabs, lists/tables, content blocks, footer, status bar, overlays, and responsive changes. Inventory exact visible titles, labels, rows, fields, values, badges, glyphs, hints, buttons/actions, empty messages, errors, and progress labels. For dynamic text, document template and data source.

### Data contract

For every rendered datum document Rust/source type, field, subsystem, transformation, grouping, ordering, formatting, fallback, conditional visibility, empty behavior, stale behavior, and error behavior. Explain the deterministic fixture shape a future preview would need, without creating fixtures.

### State and identity

Distinguish only applicable states among focused, hovered, selected, active, expanded, current, running, edited, dirty, blocked, errored, loading, refreshing, stale, disabled, unavailable, saving, succeeded, and failed.

### Keyboard, mouse, focus, cursor, and scrolling

List every active key with context, guard, action, state transition, visible result, and source symbol. Document hover/click targets, focus transfer, pointer shape, wheel routing, scrollbar/drag behavior, double-click behavior, modal ownership, initial focus, traversal, composite-widget navigation, modal trapping, restoration, cursor visibility, axes, clamping, truncation, long-name handling, and resize behavior.

Never write “standard keyboard navigation” without the actual bindings.

### States, variants, opened surfaces, and transitions

Cover default, populated, empty, loading, refreshing, stale, error, validation error, disabled, unavailable, selected, hovered, focused, dirty, saving, success, failure, blocked, confirmation, and reduced-size states where applicable. List every opened surface by stable ID.

Use:

| Current state | Event | Guard | Next state | Visible result | Side effect | Evidence/source |
|---|---|---|---|---|---|---|

### Long-running work and visual semantics

Document operation, visible owner, intermediate state, completion, failure, cancellation, retry, and interactivity. Explain how current presentation communicates hierarchy, focus, hover, selection, warning, error, progress, context, and risk. Do not propose replacement styling.

### Implementation map

Use precise approved-remote citations:

| Responsibility | Approved source | Remote path/heading | Symbol | Lines/anchor | Why it matters |
|---|---|---|---|---|---|
| View/rendering | | | | | |
| Model/state | | | | | |
| Update | | | | | |
| Messages/effects | | | | | |
| Input/keymap | | | | | |
| Mouse/hit testing | | | | | |
| Focus/scroll | | | | | |
| Data projection | | | | | |
| Tests/baselines | | | | | |
| Documentation | | | | | |

### Future redesign obligation

State only what information must remain understandable, what action must remain possible, what state distinctions must survive, what workflow continuity must be preserved, and what backend-heavy behavior may be represented with deterministic simulation. Do not propose layout or visual design.

## 8. Required product-area coverage

### Host console

Trace startup, route/stage model, render dispatch, update dispatch, adapter wiring, initial selection, Workspace grouping, expanded/collapsed rows, running-instance children, details, launch, new session, reconnect, create/edit/delete, Settings, Usage, refresh, help, debug/container info, quit, empty/loading/error states, long names, many items, scrolling, mouse behavior, and focus transfer. Verify exact source terminology.

### Workspace manager

Document list, grouping, instance children, selection, detail pane, actions, status, scrolling, overlays, empty/error states, create/edit/delete, launch, additional session, and reconnect.

### Workspace editor

Discover exact current tabs from Source 1. For every tab document rows, sections, derived previews, editable/action rows, defaults, inheritance, overrides, add/remove, validation, dirty state, saving, discard/cancel, picker/text-entry flows, keyboard, mouse, focus, scrolling, empty/loading/error states.

Investigate and verify any source equivalents of General, Mounts, Roles, Environments/Secrets, and Auth. Specifically trace mount source/destination, read-only/writable/isolation, GitHub origin, file browser, role loading/overrides, environment scopes and masking, source-backed values such as 1Password, auth modes, provider/agent relationship, role-specific overrides, credential selection/generation, and save errors.

For every generic component, enumerate each concrete use.

### Global Settings

Discover exact tabs and document global defaults, Workspace interaction, inheritance, preview rows, grouped edit dialogs, Trust decisions, allowed/blocked states, mount/environment/auth defaults, dirty/save/discard/validation/loading/token-generation flows, and every modal opened by Settings. Clearly distinguish Settings from Workspace Editor behavior.

### Modal, popup, overlay, and picker catalog

Enumerate every operator-visible family and every concrete invocation. Investigate text input, file browser, mount destination, workdir, confirmation, save/discard/cancel, GitHub, error, container/debug, status, 1Password, Role, Role override, auth Role, source, auth source, scope, and auth-form flows only when established by Source 1.

For each family find every target, constructor, opener, input branch, result branch, title, message, button set, validation rule, cancellation behavior, nested/stacked behavior, and downstream transition. A generic `Confirm` entry is insufficient: deleting a Workspace and deleting a Secret are distinct workflows.

Create:

| Modal ID | Family | Opened from | Trigger | Data shown | Default focus | Actions | Result | Evidence/source |
|---|---|---|---|---|---|---|---|---|

Also catalog help, inline pickers, loading/status overlays, launch overlays, capsule menus/dialogs, backdrop behavior, and focus ownership.

### Usage system

Start from the current provider registry or equivalent Source 1 source of truth. Do not hardcode providers from memory. Enumerate every provider, account/surface, supported capability, and unsupported fallback.

For each provider document internal ID, display label, Agent/runtime relation, account discovery, identity, username/email, plan, credential origin, data source, refresh/timeout/error behavior, quota windows, buckets, status-bar slots, percentages, used/remaining semantics, spend, reset, stale/fresh state, confidence/source labels, login/secret requirements, unsupported/unavailable states, provider-specific errors, multiple accounts, switching, ordering, and normalization.

Trace:

`provider input → provider adapter → normalized usage data → broker/projection → console Usage → capsule status bar → capsule Usage dialog`

Document exact normalized structures and fields, including provider, account, lifecycle, membership, freshness, issues, windows, labels, values, reset labels, remaining percentage, money, severity, status slots, and unresolved capabilities.

For the host Usage route document entry action, header, layout/split behavior, account list, Overview, grouping, account rows, secondary status, meters, selection, detail pane, windows, reset text, notices, refresh, detail toggle, scrolling, empty/stale/error states, and exit. Explain exactly what each region displays.

For capsule Usage document headline, width-priority information loss, provider tabs, focused account, dialog, meters, severity, source/confidence, updated/stale labels, switching, refresh/loading/failure, and relationship to focused Agent/session.

Create:

| Provider | Account identity | Data source | Windows/buckets | Spend | Reset | Special states | Console | Capsule | Evidence/source |
|---|---|---|---|---|---|---|---|---|---|

### Launch cockpit

Trace public entry through transition into the running Construct. Document identity, target, Workspace/Role/Agent, initial state, every stage/enum variant, labels, current/completed/failed state, progress, status, animation/no-motion, build-log capture/button/overlay/tail-follow/scroll/drag, failure popup/diagnostics/next steps/copy/reveal/open actions, acknowledgement, container/debug info, footer/context identity, debug differences, quit confirmation, hard cancellation, completion, alternate-screen continuity, capsule transition, and return behavior.

Use:

| Stage | Trigger | Visible label | Intermediate state | Completion | Failure | Evidence/source |
|---|---|---|---|---|---|---|

State which backend actions may be simulated later and what visible transitions must be represented. Do not implement them now.

### Capsule / in-Construct multiplexer

Map initial tab, pane tree, focus, active/custom/automatic labels, shell versus Agent pane, provider label, Agent states, status glyphs, borders, split directions, resizing, drag, zoom, pane selection, text selection, scrollback, hardware cursor, cursor visibility, prefix/normal/dialog/drag/selection modes, menu, new Agent/session, close/confirmation, status/context bars, branch/container/debug identity, Usage, provider switching, copyable values, hover/pointer/link/resize/text behavior, tab switching, Hardline/reconnection, detach/eject/exile where implemented, exit, and shutdown.

Trace visible TUI state and only those daemon/control-plane actions that change visible state. Create:

| Context | Input | Guard | Visible action | Control-plane effect | Next state | Evidence/source |
|---|---|---|---|---|---|---|

## 9. Shared interaction reference

Create one canonical cross-surface map for:

- global, screen, modal, tab, list/tree, form, picker, launch, and capsule-prefix keys
- Tab, BackTab, arrows, `j/k/h/l`, Enter, Space, Esc, PageUp/PageDown, Home/End, character/control shortcuts, prefix sequences, cancellation, destructive actions, help, and quit where used
- action versus toggle versus navigation versus editing semantics
- motion tracking, hover, pointer changes, click-to-focus/select/activate, wheel routing, scrollbar and split dragging, text selection, modal ownership, and background-hit suppression
- focus graph, composite widgets, tab bar/content, initial focus, transfer, restoration, resize behavior, and focus indication
- distinction among hover, focus, selection, current, active, and editing
- navigation cursor, hardware cursor, editing cursor, cursor visibility, scrollback suppression, and modal suppression
- vertical/horizontal/list/detail/dialog/file/build-log/capsule scrolling, follow-tail, scrollbar, clamping, and resize behavior

Use:

| Context | Key/input | Action | Conditions | Visible result | Evidence/source |
|---|---|---|---|---|---|

No hidden active shortcut may be omitted.

## 10. Current visual language

Describe only current verified presentation: brand line/pill, palette, semantic colors, terminal background assumptions, surfaces, panels, borders, focus borders, tabs, selected/hovered rows, cursor gutter, action/preview/disabled rows, warnings, errors, loading, progress, status/context/hint bars, separators, dialogs/backdrops, links/copy affordances, pointer changes, motion, responsive behavior, density, and terminal text modifiers. Cite every rule.

End with:

### Product meaning carried by current presentation

List meanings such as active interaction owner, selected Workspace, active tab, running state, blocked Agent, destructive action, current context, and provider status only when evidenced.

### Current presentation choices that are not future design invariants

List verified replaceable choices such as palette, border color, box composition, status-bar treatment, footer order, cursor glyph, panel layout, tab paint, modal geometry, and animation. Do not propose replacements.

## 11. Complete workflow catalog

For every meaningful workflow include workflow ID, goal, prerequisites, initial surface, ordered steps, decisions, dialogs, visible data, alternate paths, cancellation, errors, recovery, final state, persistent effects, and exact approved-remote citations.

Use:

| Step | Surface | Operator action | Visible response | State/effect | Next surface | Evidence/source |
|---|---|---|---|---|---|---|

Investigate, verifying exact implementation before inclusion:

- console startup, Workspace inspection, detail navigation, expansion/collapse, launch, extra session, reconnect, create/edit/save/discard/delete, refresh, Settings, Usage, help, debug/container info, and exit
- General, mounts, source browsing, destination choice, GitHub source, isolation/read-only changes, Roles, environments/secrets, scopes, source selection, 1Password, Auth, role-specific overrides, credential generation/selection, and validation/save errors
- global Settings tabs, save/discard, Trust, and errors
- Usage entry, Overview, provider/account selection, windows, detail toggle, refresh, provider switching, refreshing/stale/login/secret/unsupported/unavailable/error states, and exit
- launch initiation, every stage, build log, failure, diagnostics, copy/reveal/open, acknowledgement, debug info, cancellation, quit confirmation, completion, and capsule entry
- capsule pane interaction, tab/menu, Agent/shell creation, split/resize/focus/zoom, scrollback/select/copy, Agent state, Usage/provider switching, branch/container/debug context, close, Hardline/reconnect, Eject, Exile, and return

If a listed flow is absent, mark it `NOT_CURRENTLY_ESTABLISHED` or `UNKNOWN`; never present it as shipped.

## 12. Data-presentation map

Map Workspace identity, repository, branch, workdir, Role, Agent/provider, runtime, mounts, environment values, Auth mode/source, Trust, instance/container identity, invocation/debug identity, Agent state, launch stage, Usage, errors, diagnostics, and versions.

Use:

| Data concept | Source type | Source subsystem | Surfaces | Formatting | Empty/fallback | Evidence/source |
|---|---|---|---|---|---|---|

This must make future deterministic fixture design possible without inventing backend behavior.

## 13. Operator-visible copy and terminology

Inventory stable titles, tabs, panel/row labels, action/button labels, statuses, progress stages, empty states, validation/error headings, confirmation copy, footer hints, help text, and lifecycle vocabulary. For each phrase record location, condition, dynamic substitutions, and approved-remote source. Do not dump arbitrary backend diagnostics.

## 14. Responsive and hard-case inventory

Document verified behavior, minimums, and breakpoints where discoverable for approximately 80×24, 100×30, 120×40, and 160×50. Cover narrow/wide console, long names and paths, many mounts/env values/providers/accounts/windows/tabs/panes, long diagnostics, large logs, nested pickers, empty/loading/stale/unavailable/error/blocked states, and reduced-size behavior.

Do not redesign. State what information and actions must remain representable later.

## 15. Future preview scenario matrix

Prepare scenarios for the later Junie-based preview without creating fixtures or proposing layouts:

| Scenario ID | Product area | Required initial data shape | State | Interaction | Visible outcome | Evidence/source |
|---|---|---|---|---|---|---|

Include populated/empty Workspace manager, multiple instances, create/edit, every editor and Settings tab, picker and confirmation families, multiple Usage providers/accounts, fresh/stale/refreshing/error Usage, launch progress/failure/build log, one/multiple-Agent capsule, Idle/Working/Done/Blocked states, splits, Usage dialog, debug/container info, help, and responsive layouts.

## 16. Redesign coverage contract

Use stable IDs and state exactly what the future preview must cover:

### Must represent

Information and identity that must remain understandable.

### Must support interactively

Workflows that must be operable in the preview.

### May use deterministic simulation

Backend-heavy operations whose visible states and transitions may be simulated, such as Docker, Git, provider calls, credentials, Usage refresh, image building, and Agent startup, only where Source 1 confirms the visible contract.

### Must preserve semantically

State distinctions and product concepts that cannot be lost.

### May be completely redesigned

Current layout, chrome, styling, panel arrangement, visual hierarchy, and shortcut presentation that are not product invariants.

### Out of scope unless required

Backend details that do not affect the visible experience.

Do not propose future visual design.

## 17. Targeted source-reading index

End with an index answering:

> I am implementing or redesigning `<surface or workflow>`. Which exact approved remote paths and symbols should I open?

Organize by product model, console routing, Workspace manager/editor, Settings, modal system, file browser, Role/source/scope pickers, Auth, Usage projection and each provider, launch, failure, build log, capsule tabs/panes/dialogs/Usage, focus, mouse, keymaps, scrolling, visual tokens, tests, baselines, and documentation.

Each entry must include approved source ID, remote repository-relative path or raw-document heading, exact symbol, exact line range or stable anchor when available, and reason to inspect it. No local path may appear.

## 18. Current versus planned

Create:

| Capability | Current | Partial | Planned | Research only | Unknown | Evidence |
|---|---:|---:|---:|---:|---:|---|

Keep planned/research concepts out of the main current surface inventory unless clearly marked and needed to prevent ambiguity.

## 19. Completeness audits

Before finishing, independently audit the document against every discovered:

- route, screen, stage, tab, pane, overlay, modal, dialog, popup, picker, menu, status, and progress surface
- modal family, concrete call site, target variant, input/result/cancel path, validation rule, and nested flow
- keymap context and active key
- Usage provider, account, window, spend/reset/freshness/error capability, console projection, and capsule projection
- launch stage, overlay, failure action, progress state, and terminal transition
- capsule Agent state, mode, tab/pane action, menu/dialog action, Usage interaction, and lifecycle path
- snapshot, visual test, fixture, and baseline variant reachable inside Source 1
- material documentation-versus-source disagreement

Also audit that every capability has semantic meaning, visible data, interaction, state variants, source evidence, and future scenario coverage where applicable.

## 20. Document quality rules

Completeness beats brevity. Use normalized tables, stable IDs, graphs, state-transition matrices, exact copy, exact bindings, data fields, explicit conditions, and direct cross-references.

Do not:

- paste source files or large code blocks
- repeat identical explanations
- include generic TUI theory or generic design advice
- narrate irrelevant internal helpers
- copy approved documentation wholesale
- use vague claims such as “supports settings” or “standard keyboard navigation”
- say “opens a popup” without naming the popup, trigger, data, actions, and result
- propose a redesign

## 21. Acceptance test

Assume a future coding agent receives the approved Sources 2–4, an existing Junie design system, and `JACKIN_REFERENCE.md`, then is asked to build a completely redesigned interactive Jackin TUI preview.

The reference passes only if that agent can determine, without broad Jackin rediscovery:

- every operator-visible surface, tab, pane, modal, concrete modal flow, picker, row, field, and action
- data and data origin on every surface
- provider-specific Usage behavior in console and capsule
- keyboard, mouse, focus, hover, cursor, and scrolling behavior
- loading, stale, error, empty, disabled, unavailable, validation, blocked, and reduced-size states
- every launch stage, build-log path, failure path, and capsule transition
- every capsule tab/pane/Agent lifecycle capability
- every meaningful workflow and recovery path
- what must survive semantically and what may be redesigned
- what may be simulated deterministically
- which exact approved remote source paths and symbols explain each feature
- which scenarios the future preview must demonstrate

If any answer is no, improve the reference or mark the missing evidence precisely as `UNKNOWN`.

## 22. Final verification and stop condition

Before completing:

1. Confirm the four approved remote sources are the only evidence locations.
2. Confirm the document explicitly states that no local cleanup, checkout, clone, or local repository inspection was used.
3. Confirm revision/ref metadata is recorded without inventing a SHA.
4. Confirm all citations are remote-only and use approved source IDs.
5. Confirm all cited paths, symbols, and line ranges/anchors exist remotely where asserted.
6. Confirm every editor tab and Settings tab is documented.
7. Confirm every modal family, concrete modal use, picker, launch stage, Usage provider, capsule state, and keymap context is accounted for.
8. Confirm host Usage and capsule Usage are both documented.
9. Confirm current versus planned behavior is separated.
10. Confirm the scenario matrix, redesign contract, source-reading index, workflow catalog, and completeness audits exist.
11. Confirm no local source was used as evidence.
12. Confirm no Jackin code was copied and no application or artifact was created.
13. Confirm the only deliverable is `JACKIN_REFERENCE.md`.

Stop after the complete source-backed reference exists.

Do not implement.

Do not design.

Do not clone or checkout.

Use only the four approved remote locations.

Return `JACKIN_REFERENCE.md`.
