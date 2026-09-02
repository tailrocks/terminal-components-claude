/goal Continue directly from the approved Junie-inspired Ratatui component library.

Your mission is to design and implement a complete, runnable, interactive terminal redesign of Jackin on top of this approved component library.

The target is:

> If Jackin had originally been designed as a premium modern terminal application using our approved Junie-inspired design system, what would the complete experience look and feel like?

Build a real interactive product preview whose screens, transitions, controls, and simulated workflows can be experienced directly.

Add a runnable application binary named `jackin` in this repository.

The complete current Jackin experience must be represented:

1. Construct entry and exit rituals
2. Host Workspace Manager
3. Create Workspace prelude
4. Workspace Editor
5. Global Settings
6. Host Usage
7. Host dialogs, pickers, help, status, and errors
8. Launch cockpit
9. Build log, failure, and runtime diagnostics
10. Capsule terminal and multiplexer
11. Capsule dialogs, context, and Usage
12. Detach, reconnect, takeover, dirty exit, and final Construct exit

Add one deliberate product extension:

13. A unified Account & Usage Center for registering multiple local AI coding-agent accounts and API-key profiles, then observing usage across them.

Compose everything above into one continuous Jackin application.

---

# 1. AUTHORITATIVE REFERENCES AND BOUNDARIES

Use these local artifacts as the authority:

- `JUNIE_PROMPT1.md` — original visual, component, interaction, and quality principles
- `JACKIN_REFERENCE.md` — canonical current-product semantics, surfaces, workflows, states, terminology, and redesign obligations
- `DESIGN.md`, `README.md`, `src/theme.rs`, `src/widgets/`, and the shared runtime — approved implementation foundations

`JACKIN_REFERENCE.md` defines the complete current Jackin product scope. Implement its operator-visible capabilities and interaction semantics through the approved Junie design language.

Implement the preview in this component-library repository, primarily under:

- `src/bin/jackin/`
- generic library files only when a real reusable component gap exists
- `Cargo.toml`, `README.md`, `DESIGN.md` when reusable conventions change, and Jackin capture artifacts

Extend current Jackin with the Account & Usage Center defined below. Use deterministic fixtures for external systems while making every operator interaction real.

---

# 2. USE THE APPROVED JUNIE DESIGN

Build Jackin with the approved component system and its established visual language:

- semantic colors and surfaces
- near-black hierarchy
- restrained green accent
- white and gray text hierarchy
- spacing and padding rhythm
- borders and grouping
- hover treatment
- focus treatment
- selection treatment
- editing treatment
- disabled and error states
- scrollbars
- keyboard behavior
- mouse behavior
- density and restraint

Jackin may require new composition and new generic primitives. Any new generic component must:

- use existing semantic tokens
- match existing visual language
- support keyboard input
- support mouse input where meaningful
- expose focus, hover, active, selected, disabled, error, loading, and editing states where meaningful
- live in the reusable library rather than contain Jackin domain knowledge

Translate current Jackin product meaning and workflows into this visual system with terminal-native composition.

---

# 3. PRODUCT MODEL MUST REMAIN CORRECT

Keep these concepts distinct in state, copy, and navigation:

- Operator
- Construct
- Workspace
- Role
- Mount
- Environment / secret
- Auth configuration
- Agent runtime
- Provider
- Account
- Usage surface
- Instance
- Session
- Capsule
- tab
- pane
- launch run

Represent provider, agent, account, auth mode, and Usage as separate linked concepts.

Include the current runtime Agent choices: Claude Code, Codex, Amp, Kimi Code, OpenCode, and Grok Build. Manual account registration covers the provider set defined in the Account & Usage Center section.

The core lifecycle is:

```text
Host configuration
    ↓
Workspace + Role + Auth + Agent selection
    ↓
Construct entry
    ↓
Launch pipeline
    ↓
Durable instance
    ↓
Capsule sessions, tabs, panes, and PTYs
    ↓
Detach / reconnect / exit
    ↓
Construct exit when the final instance closes
```

The UI must always make current scope clear: global, Workspace, Role, provider account, instance, session, tab, or pane.

---

# 4. MAKE THE APPLICATION FEEL LIKE ENTERING A WORLD

Jackin has a special boundary experience. Preserve it as a first-class product feature.

The operator should feel that they cross from the ordinary host into a focused digital society where agents, Workspaces, instances, sessions, and provider capacity are managed.

The feeling comes from sequence, depth, motion, soundless pacing, language, controlled green signal, clear state changes, and restrained terminal-native detail.

Keep the core application restrained and Junie-like. Concentrate spectacle at meaningful boundaries:

- Construct entry
- launch transition
- Capsule handoff
- final Construct exit

The boundary rituals and launch atmosphere may use refined digital rain, signal trails, glitch resolution, spatial warp, or another terminal-native effect derived from the current Jackin reference. They must still use the approved palette and typography hierarchy.

---

# 5. CONSTRUCT INTRO

Build a real, animated entry ritual.

Trigger it when the operator starts Jackin while there are zero running Jackin instances. Zero running instances define the empty-Construct boundary.

Play it immediately before the Host Workspace Manager, matching current empty-console startup and the user's beginning-of-day experience. An idle quit releases simulated pending entry state and returns directly to the host.

Preserve these semantics:

- play once when entering an empty Construct
- reconnecting, opening another session, or starting another instance joins the active Construct without replay
- prevent duplicate intro playback when concurrent entry attempts exist in simulated state
- enter the Host Workspace Manager after the ritual
- keep the terminal under one continuous application-owned experience

Use the current ritual as experiential reference:

- sparse opening phrases
- Jackin identity mark
- controlled glitch resolution
- accelerating spatial transition into the Construct

The current phrases may be retained or refined only if the result preserves their meaning: the operator is leaving the host boundary and entering the Construct.

Requirements:

- animation must be tick-driven, nonblocking, deterministic, and resize-aware
- Enter and Esc during the phrase sequence skip the remaining phrases and continue into the entry transition
- Enter and Esc during the entry transition finish that transition immediately and enter the manager
- drain stale triggering input before the ritual begins
- hide and restore cursor/wrap state correctly through the shared runtime
- provide a reduced-motion or instant path that preserves the entry meaning
- let Enter and Esc always advance the operator
- render coherently at all supported terminal sizes

Capture at least:

- first entry with zero instances
- entry when an instance already exists, proving no replay
- skip during phrase phase
- skip during transition phase
- resize during animation
- reduced-motion entry

Apply `JACKIN_NO_MOTION` across intro, cockpit, and outro. Explicit CLI motion mode takes precedence; otherwise `JACKIN_NO_MOTION` selects the static meaningful boundary state and immediate transition.

---

# 6. CONSTRUCT OUTRO

Build a matching exit ritual.

The rich outro belongs to the final Construct boundary:

- leaving one foreground session while other instances remain shows compact still-inside feedback
- closing the final running instance may play it once
- concurrent simulated exits produce one outro winner
- failure to determine whether instances remain must fail closed rather than falsely claim the Construct ended
- a missing elapsed-time value omits the duration line while retaining the exit transition
- a fresh-Construct launch that fails with zero surviving instances still reaches failure acknowledgement and then the single final-boundary outro
- a failed launch while another instance remains returns to the active Construct

Use the current experience as reference:

- decelerating spatial transition out of the Construct
- a quiet final caption such as `You were in the Construct for …`
- elapsed duration from Construct entry to final exit
- clear restoration to the host terminal

When other instances remain, provide compact feedback that the operator is still inside the Construct, with remaining instance count, safe identity summaries, and masked private paths.

Quitting an idle Host Workspace Manager releases pending entry state and returns directly to the host.

When remaining-instance discovery fails, show concise non-secret diagnostic/status feedback, reserve the rich outro, and restore or return safely.

Requirements:

- Enter and Esc during the exit transition advance to the closing caption; Enter and Esc during the caption finish the ritual
- reduced-motion path exists
- resize remains coherent
- cursor, wrap, raw mode, alternate screen, and mouse capture restore correctly
- continuous presentation between Capsule exit, outro, and terminal restoration
- rich outro reserved for the final running Jackin instance

---

# 7. OVERALL INFORMATION ARCHITECTURE

Build one continuous application around these primary regions:

```text
ENTRY RITUAL
    ↓
HOST CONTROL PLANE
    ├── Workspaces and instances
    ├── Accounts and Usage
    └── Global Settings
    ↓
CREATE / CONFIGURE / SELECT
    ↓
LAUNCH COCKPIT
    ↓
CAPSULE
    ↓
DETACH / RECONNECT / FINAL EXIT
    ↓
OUTRO
```

Determine final terminal-native layouts through rendering and interaction.

An operator must always understand:

- whether they are outside, entering, or inside the Construct
- how many instances are active
- which Workspace is selected
- which Role and Agent are selected
- which instance or session is current
- which account provides credentials
- which provider/account Usage is being shown
- which pane owns keyboard input
- where mouse hover is
- whether state is current, stale, loading, failed, unsupported, dirty, or unavailable
- whether an action affects global, Workspace, Role, instance, session, tab, or pane scope

Use progressive disclosure so each surface emphasizes its current task and context.

---

# 8. HOST WORKSPACE MANAGER

Create a polished Host Workspace Manager representing the current List surface.

It must support:

- current directory
- saved Workspaces
- expanded/collapsed instance children
- `+ New workspace`
- selected-row detail
- live instance summaries
- status and lifecycle labels
- empty, unavailable, loading, error, running, clean-exited, crashed, preserved-dirty, preserved-unpushed, restore-available, and failed-setup meanings where established by the reference

Actions must include:

- launch current directory
- launch saved Workspace
- expand/collapse
- edit Workspace
- delete Workspace with confirmation
- prewarm
- reconnect/restore instance
- start a new session
- open shell
- inspect instance
- stop instance
- purge instance with confirmation
- open Accounts & Usage
- open Settings
- open help
- quit

Keep selection identity stable while background fixture updates occur. Distinguish persisted Workspace data, persisted instance records, and live daemon/session snapshots.

The selected Workspace or instance exposes a focused detail projection from the same underlying state.

Large trees, long paths, long labels, missing daemon data, missing manifest data, and many instances must remain usable.

Purged and superseded instance records remain hidden from the normal tree, matching current Jackin. Their meaning may appear only in the action result, confirmation, or a clearly separate historical fixture.

---

# 9. CREATE WORKSPACE PRELUDE

Represent the current five-part create chain:

1. choose a directory or source
2. choose mount destination behavior
3. optionally edit destination
4. choose working directory
5. name the Workspace

The flow must support:

- file-browser interaction using deterministic fixture paths
- Git/source choices where current Jackin exposes them
- first-mount readonly choice
- source path as the default mount destination
- same-path fast path
- destination basename as the default Workspace name
- forward progression
- Esc/back rewind
- cancellation to the Workspace Manager
- validation
- exact parent restoration
- handoff into the Workspace Editor with pending state

Keep the sequence staged and focused.

If a reusable stepper or wizard primitive is required, implement only the generic mechanics in the library. Keep Workspace rules inside the Jackin application.

---

# 10. WORKSPACE EDITOR

Build all five current Workspace Editor tabs:

1. General
2. Mounts
3. Roles
4. Environments
5. Auth

The Editor must maintain original and pending state separately.

Support:

- General controls for Name, Working directory, Keep awake where platform-relevant, and Git pull behavior
- tab navigation
- row focus and selection
- add/edit/remove flows
- enabled/disabled and allowed/default Role states
- `+ Load role`, trust, unavailable, and load-error chains
- source and destination paths
- readonly semantics and Shared / Worktree / Clone isolation cycling
- source-drift and running-isolated cleanup blockers before save
- environment scope and source
- masked secrets
- plain, 1Password-style, and other currently represented source choices using synthetic fixture data
- Auth mode/source/folder selection
- validation errors
- dirty counts/state
- effective save preview
- asynchronous simulated save
- save success/failure
- Save / Discard / Cancel on dirty exit
- correct focus restoration after every child modal

Render masked synthetic secret metadata throughout the flow.

---

# 11. GLOBAL SETTINGS

Build all five current Settings tabs:

1. General
2. Mounts
3. Environments
4. Auth
5. Trust

Preserve the distinction between global Settings and Workspace-specific Editor state.

Represent current behaviors including:

- coauthor and DCO preferences
- global mounts and scope
- global environments and secret sources
- global/Role Auth configuration
- Role source trust
- add/edit/delete/reset flows
- child pickers and forms
- validation
- dirty state
- effective save preview
- Save / Discard / Cancel
- error recovery

The new account registry integrates with Auth controls while retaining global, Workspace, and Role scope semantics.

---

# 12. UNIFIED ACCOUNT & USAGE CENTER

This is the major deliberate extension.

Build one singular interface where an operator can register, inspect, edit, disable, remove, validate, and select multiple AI coding-agent accounts.

Manual account registration is limited to these current Jackin provider families:

- Claude Code
- Codex
- Grok Build
- OpenCode

Treat those labels as an operator-facing linked mapping across separate dimensions:

| Agent runtime | Provider adapter identity | Usage surface | Allowed manual credential sources | Endpoint rule |
|---|---|---|---|---|
| Claude Code | Anthropic / Claude | Claude | local Claude profile/home folder; direct Anthropic API key | no invented endpoint field |
| Codex | OpenAI | Codex | local `CODEX_HOME`-style folder; direct OpenAI API key | no invented endpoint field |
| Grok Build | xAI / Grok | Grok | local Grok profile folder; direct xAI/deployment API key | endpoint/deployment field only in a source-backed Grok fixture |
| OpenCode | OpenCode | OpenCode | local OpenCode profile folder; direct OpenCode API key | no arbitrary custom endpoint |

Store and display these dimensions separately even when one row links them.

Credential modes:

- local agent home/config folder
- direct API key for a selected supported provider

`Direct API Key` is a credential source for one of the supported provider families.

One operator may have multiple accounts for the same provider. Examples:

- Claude Code · Personal
- Claude Code · Work
- Codex · Primary
- Codex · Experiments
- Grok Build · Team
- OpenCode · Go subscription

For a folder-backed account, support realistic fields and states such as:

- display name
- provider family
- source type
- local folder path
- detected credential/profile type
- stable non-secret account identity when available
- stable subject/handle when available
- discovery provenance and confidence
- account lifecycle
- environment or purpose label
- enabled/disabled
- default for provider
- validation state
- last refresh
- freshness
- recoverable issue

For an API-key-backed account, support:

- display name
- provider family
- masked key entry
- optional endpoint only where current provider semantics support one
- validation state
- enabled/disabled
- default for provider
- last refresh/freshness

Actions must include:

- add account
- choose provider
- choose folder or API key
- browse/type folder path
- validate source
- save
- edit
- duplicate protection
- enable/disable
- set provider default
- remove with confirmation
- refresh one
- refresh provider
- refresh all
- search/filter
- inspect details without revealing secrets

Use deterministic simulated detection and provider responses. Interaction must be real even when backend data is simulated.

Validation distinguishes discovered material, authenticated identity, and usable quota access.

Handle:

- valid source
- missing folder
- unreadable folder
- wrong-provider folder
- missing credential file
- malformed credential metadata
- duplicate folder/account
- empty API key
- invalid API key
- unauthorized
- rate limited
- provider unavailable
- stale last-good data
- unsupported quota visibility
- account with no stable public identity

Secrets remain masked. Diagnostics expose origin type and safe path labels.

---

# 13. ACCOUNT INTEGRATION

The Account & Usage Center must connect to the rest of Jackin.

Allow current account identity to appear where relevant:

- Workspace Editor Auth
- Global Settings Auth
- Role-specific Auth choices
- launch Agent/provider/account selection
- new Capsule session selection
- Host Usage
- Capsule Usage

Support a clear precedence model in fixture state:

```text
session choice
    overrides Workspace / Role choice
    overrides provider default account
    falls back to discovered/current source
```

Show the active scope and why each account was selected.

Changing an account selection mutates only its explicit session, Workspace, Role, or global scope.

---

# 14. USAGE OBSERVABILITY

Create both:

- a concise overall Usage overview
- detailed provider and account views

Represent:

- provider
- account identity
- credential origin without secret material
- plan or account type when known
- quota windows
- used/remaining values
- reset times
- spend/credits where supplied
- Current / Stale / Refreshing / Failed freshness
- Available / Not started / Warning / Exhausted / Unsupported / Unavailable / Error quota status
- last-good data retained under stale/error state
- unresolved provider/account identity

Show overall observability across all registered accounts and providers. Overall must be honest.

Use an overall health/capacity summary, account counts, warnings, exhausted windows, stale sources, and provider-level rollups for genuinely comparable units and windows.

The current Usage registry includes Claude, Codex, Amp, Grok, Z.AI, Kimi, MiniMax, OpenCode, and Unsupported states. Preserve current read-only projection semantics for these reference-backed surfaces. New manual registration remains limited to Claude Code, Codex, Grok Build, OpenCode, and their direct API-key modes.

Provide at least one complete deterministic account/detail fixture, or the explicit Unsupported sentinel state, for every current Usage surface, including Amp, Z.AI, Kimi, and MiniMax. Those non-registerable provider fixtures remain discovered/read-only and expose no account-mutation action.

Host Usage and Capsule Usage are read-only projections. Account mutation belongs to the Host Account & Usage Center.

---

# 15. HOST MODALS, PICKERS, HELP, STATUS, AND ERRORS

Cover the current modal families through connected workflows:

- text input
- file browser
- mount destination choice
- working-directory picker
- confirmation
- Save / Discard / Cancel
- save preview
- GitHub/source picker
- error popup
- status popup
- container info
- 1Password-style picker
- Role picker
- provider/Agent picker
- source picker
- Auth source picker
- scope picker
- Auth form
- account picker

The coverage ledger remains authoritative over this family summary. Exercise every Editor, Settings, host modal, Usage, launch-stage, Capsule-mode, and Capsule-dialog child ID, including Role override/Auth Role, mount scope/Role/confirmation/preview, environment source/1Password/Role/scope/confirmation, and nested Auth source-folder flows.

Rules:

- one visible modal owner at a time
- modal focus barrier blocks parent input
- Enter commits only when valid
- Esc cancels or rewinds safely
- nested flows restore the exact parent and target focus
- help has explicit top-level priority
- destructive actions use explicit labels and safe default focus
- errors remain explicit and recoverable
- bodies scroll and shrink for small terminals
- modal barriers capture background input

---

# 16. LAUNCH COCKPIT

Build a premium launch transition preserving the exact ordered 11-stage model:

1. Identity
2. Role
3. Credentials
4. Construct
5. Agent Binaries
6. Derived Image
7. Workspace
8. Network
9. Sidecar
10. Capsule
11. Hardline

Represent stage states:

- Queued
- Running
- Done
- Skipped
- Failed
- Blocked

`Blocked` is a modeled-only fixture because current Jackin renders the state without an audited runtime producer.

The cockpit must show:

- current target identity
- Workspace
- Role
- Agent/provider/account selection
- current stage/frontier
- activity text
- completed/skipped stage count and ordered frontier
- container identity when available
- debug/run identity when appropriate
- cancel/quit controls
- digital-rain or signal atmosphere integrated with the approved design

Motion communicates activity and transition while stage status stays primary.

The cockpit must continue animating while simulated launch work advances. Use deterministic tick-driven state.

Support a continuous handoff from cockpit to Capsule.

---

# 17. BUILD LOG AND LAUNCH DIAGNOSTICS

Build the current launch overlays:

## Build log

- active/complete title state
- bounded retained lines
- ANSI-like fixture text rendered safely
- wrapped continuations
- tail follow
- manual scroll
- PageUp/PageDown
- wheel
- scrollbar drag
- return to active cockpit

## Failure

- failed stage
- safe summary
- next step
- run identity
- only the run ID as the failure overlay's copyable identifier
- long detail scrolling
- acknowledgement
- no implicit retry or false success
- frozen failure motion

## Container/runtime info

- target
- Workspace/Role/Agent
- account/provider where appropriate
- container/run identity
- optional debug fields
- explicit copy/dismiss
- absent fields are omitted

Ctrl-C hard abort, Ctrl-Q confirmation, typed cancellation, failure, and normal handoff must remain distinct outcomes.

---

# 18. CAPSULE EXPERIENCE

Build the in-Construct Capsule as a real interactive terminal-workspace simulation.

Terminal content remains primary. Surround it with restrained context and control.

Support:

- multiple tabs
- multiple PTY-like panes
- horizontal and vertical splits
- current pane focus
- nearest-pane navigation
- split resizing
- zoom/unzoom
- tab switching and indexed jumps
- tab overflow
- status/attention glyphs
- Workspace/branch/PR context
- Usage/account context
- container/debug context
- hardware cursor simulation
- live view versus scrollback
- typing snaps back to live
- mouse wheel scrollback
- mouse drag selection
- word selection where practical
- copy action
- clear pane
- rename tab
- detach
- close pane/tab
- exit

Use believable deterministic terminal transcripts for Claude Code, Codex, Amp, Kimi Code, Grok Build, OpenCode, shells, and mixed sessions.

Deterministic services simulate PTYs, containers, daemons, and agent processes while preserving terminal interaction semantics.

Input priority routes dialog, palette, selection drag, scrollback, and prefix modes before simulated pane input.

---

# 19. CAPSULE MODES AND DIALOGS

Represent the exact current Capsule mode priority:

- normal
- prefix awaiting
- dialog
- drag/resize
- text selection

Treat Command Palette as a dialog variant and live-versus-scrollback as pane state. Preserve this separation in input routing.

Build connected flows for:

- Command Palette
- Agent Picker
- Provider/account Picker
- Rename Tab
- Export File
- Container Info
- GitHub Context
- Usage
- Spawn Failure
- Split Direction Picker
- Close Target Picker
- Confirm Action
- Exec Picker
- Exit Dirty
- Exit Inspect

Support core actions:

- new tab/session
- next/previous/indexed tab
- pane focus movement
- horizontal/vertical split
- split resize
- zoom
- close pane/tab
- clear
- detach
- Usage
- palette
- redraw
- rename
- export/copy/open simulated host target with explicit gates
- dirty exit choices

Dialog and palette focus must be contained. Close must restore the prior mode and pane focus.

---

# 20. DETACH, RECONNECT, TAKEOVER, AND DIRTY EXIT

Preserve lifecycle meaning:

- Detach leaves the client while the instance, Capsule, sessions, tabs, and panes remain.
- Reconnect restores the durable in-Construct state.
- A new attach may take over the single active client; the old attach is displaced while daemon state remains.
- Exit may shut down the final session/instance only through explicit policy.
- Dirty exit offers explicit branches such as start another Agent, inspect, keep, discard, or cancel where current semantics require them.

Make durable versus transient state visible.

Present separate actions and consequences for:

- closing a pane
- closing a tab
- ending a session
- detaching a client
- stopping an instance
- purging an instance
- exiting the final Construct

These actions have different scope and consequence.

---

# 21. CONTINUOUS END-TO-END WORKFLOWS

The screens must connect into real journeys.

At minimum, support these classes of workflow:

## First use

Empty Construct → intro → empty Workspace Manager → register accounts → create Workspace → configure → launch → cockpit → Capsule.

## Returning operator

Existing instance → no repeated intro within same Construct → inspect Workspace/instance → reconnect → restore Capsule tabs/panes.

## Multiple accounts

Register two accounts for one provider → choose provider default → set Workspace override → launch with visible resolved account → inspect per-account Usage.

## Configuration

Edit mounts/Roles/environments/Auth → encounter validation → inspect save preview → save → return with selection restored.

## Launch failure

Launch → inspect build log → fail one stage → inspect safe failure detail → acknowledge → return with explicit failed state.

## Multi-session Capsule

Spawn session → split pane → change focus → resize → scroll/select/copy → open palette → inspect Usage → detach → reconnect.

## Construct exit

Exit one instance while another remains → see still-inside feedback and no rich outro → exit final instance → outro with elapsed duration → terminal restoration.

---

# 22. CRITICAL STATE RULE

Across the entire application:

```text
HOVER
!= FOCUS
!= CURRENT
!= SELECTED
!= ACTIVE
!= EDITING
!= DIRTY
!= RUNNING
!= STALE
!= ERROR
!= DISABLED
```

Model these as semantic application states shared by rendering and interaction.

Pair important state with copy, glyph, shape, position, or another non-color signal.

Use restrained visual treatment with immediate state clarity.

---

# 23. KEYBOARD-FIRST

The complete required journey works with keyboard alone.

Use current Jackin shortcuts where they remain terminal-appropriate, including familiar semantics for:

- tree navigation
- tab/pane movement
- launch and reconnect actions
- edit/save/cancel
- Usage refresh
- prefix commands
- palette
- split/zoom/close
- detach/exit

Support:

- Tab / Shift+Tab focus traversal
- arrows and `h/j/k/l` where semantically appropriate
- Enter/Space activation
- Esc cancellation/back
- contextual action keys
- deterministic focus order
- modal focus containment

Provide concise contextual hints relevant to current focus and mode.

---

# 24. MOUSE-EXCELLENT

Mouse-capable terminals must provide coherent:

- hover
- press and release activation
- row selection
- disclosure toggles
- tabs
- pane focus
- split dragging
- wheel scrolling
- scrollbar interaction
- text selection
- dialog buttons
- picker rows
- account actions
- Usage detail selection

Hover remains distinct from keyboard focus. Switching between input methods retains selected/current identity.

Modal and overlay hit regions must block background actions.

---

# 25. RESPONSIVE TERMINAL DESIGN

Design for:

- 80×24
- 100×30
- 120×40
- 160×50

The optimal experience may favor modern medium and large terminals, but 80×24 must remain coherent.

At smaller sizes:

- collapse or hide secondary detail
- use one primary pane plus overlays/drawers
- reduce low-priority metadata
- keep active identity, focus, state, and critical actions visible
- present one readable primary pane with secondary content in overlays or drawers

At larger sizes:

- expose useful context
- use width for detail, live instance/session topology, Usage, and terminal panes
- use additional width for meaningful context

Define a minimum supported size and render a polished too-small state below it.

Intro, outro, cockpit, dialogs, file browser, Account & Usage Center, and Capsule must all respond to resize.

---

# 26. DETERMINISTIC FIXTURE WORLD

Use realistic deterministic in-memory state.

Include:

- no-instance first startup
- current directory
- several saved Workspaces
- multiple Roles
- realistic mounts and paths
- masked environment/secret metadata
- running, stopped, crashed, preserved, and reconnectable instances
- multiple sessions/tabs/panes
- branch/PR context
- launch stages and logs
- provider accounts with mixed health
- current/stale/refreshing/error Usage
- long labels and paths
- enough terminal output for scrollback and selection

Use believable names and values, such as:

- `payments-platform`
- `infra-control-plane`
- `release-automation`
- `customer-portal`
- Personal / Work / Team provider accounts
- realistic repository paths, Role names, branches, run IDs, and timestamps

Fixtures use synthetic paths and credentials plus simulated Docker, provider, daemon, and PTY state.

---

# 27. IMPLEMENTATION ARCHITECTURE

Follow the established application pattern.

Add:

- a `jackin` binary in `Cargo.toml`
- `src/bin/jackin/main.rs`
- an application shell and route model
- deterministic domain fixtures
- focused screen modules
- README run/help documentation
- capture support through the existing tools

Provide a deterministic scenario contract:

- an `App::for_scenario(...)` constructor or equivalent pure fixture entry point
- `--scenario` choices covering at least `first-use`, `returning`, `accounts-mixed`, `launch-running`, `launch-failure`, `capsule-multi`, `outro-last`, and `hard-cases`
- `--motion full|reduced|paused`
- `--frame <N>` as the exact fixture tick selector for paused capture of intro, cockpit, and outro phases
- a fixture clock that drives deterministic elapsed-time captions

Each scenario represents one coherent world with internally consistent startup state.

Use one terminal-session guard that owns raw mode, alternate screen, mouse capture, bracketed paste, cursor visibility, and line-wrap state, restoring them on every exit path. Keep intro, manager, cockpit, Capsule, and outro as application route states inside one runtime session. Check quit state after tick-driven transitions as well as key/mouse input. Set final quit after the outro or its reduced-motion equivalent completes.

Model concurrent boundary behavior with a deterministic in-memory arbiter:

- pending entry claims suppress duplicate intro
- remaining-instance discovery returns typed success/failure
- one-consumer exit token permits one outro
- repeated messages are idempotent

This models current cross-process semantics inside the deterministic preview.

Keep clear boundaries between:

## Reusable design system

- theme tokens
- focus ring
- hit testing
- interaction state
- scrolling
- overlays/dialogs
- generic lists/trees/tabs/forms/pickers
- generic reusable animation mechanics only if reuse is proven

## Jackin application

- Construct lifecycle
- Workspaces/Roles/mounts/environments/Auth
- account registry and resolution
- Usage projection
- instances/sessions
- launch stages
- Capsule state
- simulated provider/runtime effects
- intro/outro choreography

Render functions are pure over current state. Provider, filesystem, Docker, daemon, and PTY simulations update through deterministic messages and ticks.

Start with app-specific composition, extracting only mechanics that serve multiple Jackin surfaces or reusable library needs.

Known component gaps require explicit decisions:

- use a reusable masked `SecretInput` or equivalent for API keys
- add reusable split-handle geometry and drag mechanics if both Host and Capsule need them
- consider a reusable selectable read-only viewport only if it serves more than the simulated terminal
- keep CreatePrelude, file-browser composition, nested Capsule pane tree, and ritual choreography application-specific unless reuse is proven

Raw synthetic API-key text lives only in transient edit state. Rendering, captures, logs, committed fixture records, and diagnostics use masks, safe fingerprints, or a short synthetic tail.

---

# 28. SIMULATED PRODUCT SERVICES

Use deterministic in-memory services for container state, PTYs, agent processes, provider responses, credential discovery, fixture files, 1Password-style sources, host actions, and daemon events.

Copy actions write to an in-memory preview clipboard and show status. Export/open/reveal actions expose simulated success, cancellation, and error states. Logs render from sanitized text or structured styled spans.

The operator can navigate, focus, hover, type, edit, save, cancel, select, scroll, resize, split, drag, refresh, detach, reconnect, trigger errors, and complete the full product flow.

---

# 29. BUILD IN VERTICAL SLICES

## Slice 1 — Complete Jackin spine

- application shell and terminal-session guard
- intro
- preconfigured Workspace Manager
- launch cockpit
- Capsule handoff and one interactive tab/pane
- detach/reconnect
- remaining-instance and final-exit behavior
- outro

Run it. Interact with it. Capture it. Critique it. Fix it.

## Slice 2 — Accounts and Usage

- Account & Usage Center
- account add/edit/remove/default flow
- account precedence and launch/session integration
- Host Usage
- Capsule Usage
- all current provider projection fixtures

Run it. Interact with it. Capture it. Critique it. Fix it.

## Slice 3 — Configuration and full Capsule

- Create Workspace prelude
- Workspace Editor
- Global Settings
- host modal families
- build/failure/info overlays
- complete tabs/panes/splits
- focus/resize/zoom
- terminal scrollback/selection
- palette/dialogs
- account/context integration
- detach/reconnect/takeover/dirty exit

Run it. Interact with it. Capture it. Critique it. Fix it.

## Slice 4 — Integration and hard cases

- every remaining-instance/final-outro/failure boundary variant
- complete responsive pass
- cross-screen state continuity
- hard cases
- full acceptance journey

Complete each slice's interactive and visual quality pass before starting the next slice.

---

# 30. SUBAGENTS

Use multiple subagents in parallel before and during implementation.

At minimum delegate independent work for:

### REFERENCE COVERAGE AUDIT

Map every must-represent and must-support item from `JACKIN_REFERENCE.md` to a planned preview surface and acceptance proof.

### CURRENT JACKIN SOURCE REVIEW

Use the cited pinned source to resolve ambiguous current semantics, shortcuts, state ownership, intro/outro triggers, and lifecycle boundaries.

### HOST INFORMATION ARCHITECTURE

Design the terminal-native relationship among Workspaces, instances, configuration, Accounts, Usage, and launch.

### ACCOUNT & USAGE SPECIALIST

Design multiple-account registration, account resolution, safe secret handling, freshness, honest aggregation, and provider/account error states.

### INTRO / OUTRO MOTION SPECIALIST

Design a deterministic, resize-aware, skippable, reduced-motion-compatible Construct boundary ritual using the approved visual system.

### CAPSULE TERMINAL UX SPECIALIST

Design panes, tabs, prefix/palette modes, scrollback, selection, cursor, detach, takeover, and dirty exit.

### COMPONENT GAP ANALYSIS

Identify reusable primitive gaps and keep Jackin domain state in the application.

### INTERACTION AND RESPONSIVE REVIEW

Audit keyboard, mouse, focus, modal barriers, scrolling, and all four viewport sizes.

### INDEPENDENT VISUAL REVIEW

Inspect actual rendered output and recommend precise improvements to hierarchy, restraint, consistency, and Jackin identity.

The primary agent owns synthesis and resolves conflicting recommendations against the approved design and product goal.

---

# 31. VISUAL REVIEW LOOP

For every major surface and transition:

1. implement
2. run
3. render/capture actual output
4. inspect hierarchy, spacing, alignment, density, copy, and state
5. navigate with keyboard
6. interact with mouse
7. resize
8. critique
9. fix
10. repeat

Inspect intro and outro as complete sequences.

Inspect the Account & Usage Center with multiple accounts, mixed health, long labels, empty state, validation, and errors.

Inspect Capsule with real-looking terminal content and several pane layouts.

Use deterministic scenario/frame controls with the existing capture harness. Example pattern:

```bash
rtk cargo build --bin jackin
BIN=target/debug/jackin ARGS='--scenario first-use --motion paused --frame 0' tools/capture.sh start 120 40
tools/capture.sh shot j_intro_phrase
tools/capture.sh stop
```

Capture named sequence checkpoints, including `j_intro_phrase`, `j_intro_warp_early`, `j_intro_warp_late`, `j_outro_warp`, and `j_outro_caption`, plus representative Host, Account/Usage, launch, Capsule, modal, error, 80-column, and 160-column states.

---

# 32. VISUAL TARGET

The result feels precise, calm, powerful, and unusually polished.

Core qualities:

- clear visual hierarchy with restrained borders and green accents
- unmistakable focus, hover, selection, editing, dirty, running, stale, and error states
- explicit Workspace, Role, Agent, provider, account, instance, session, tab, and pane scope
- honest provider/account Usage presentation
- masked credential data
- meaningful animation at lifecycle boundaries
- fully interactive Capsule panes
- complete keyboard and mouse operation
- coherent layouts at every supported size
- modeled loading, success, failure, and recovery states

The operator feels the special world most strongly at entry, launch, Capsule handoff, and exit. During work, clarity and control dominate spectacle.

---

# 33. PREVIEW STATE MATRIX

Implement and visually review:

## Boundary rituals

- zero-instance intro
- active-instance no-replay
- concurrent entry claim
- phrase skip
- transition skip
- resize
- reduced motion
- one-of-many instance exit
- final-instance outro
- concurrent final exits
- missing elapsed duration
- running-instance discovery failure
- fresh-Construct launch failure with zero surviving instances, followed by one outro
- launch failure while another instance remains, with no rich outro

## Workspace Manager

- no Workspaces
- current directory only
- many Workspaces
- many instance children
- collapsed/expanded
- long paths/names
- missing daemon data
- crashed/preserved/reconnectable instance
- async refresh failure

## Account & Usage Center

- no accounts
- several providers
- several accounts for one provider
- duplicate folder
- wrong-provider folder
- missing/unreadable folder
- valid/invalid API key
- disabled/default account
- current/stale/refreshing/error
- warning/exhausted/unsupported quota
- provider unavailable/rate limited
- unresolved account identity
- long account labels
- no secret leakage

## Create / Editor / Settings

- validation failure
- child modal cancellation
- dirty exit
- save preview overflow
- save failure
- long mounts
- masked secrets
- Role trust error

## Launch

- all queued
- active progress
- skipped stage
- blocked stage
- long build log
- manual log scroll and tail resume
- failure
- cancellation
- quit confirmation
- Capsule handoff

## Capsule

- no sessions
- one pane
- many tabs
- tab overflow
- nested splits
- resize limits
- zoom
- long output
- scrollback versus live
- selection/copy
- dialog focus barrier
- spawn failure
- dirty exit
- detach/reconnect
- takeover

## Responsive

- 80×24
- 100×30
- 120×40
- 160×50
- below-minimum state

---

# 34. COMPLETE JACKIN FLOW

Run this connected experience:

1. Launch `jackin` with deterministic state containing zero running instances.
2. See and complete the Construct intro.
3. Arrive at the Host Workspace Manager.
4. Open Account & Usage Center.
5. Add a Claude Code local-folder account.
6. Add a second Claude Code local-folder account.
7. Add a Codex account.
8. Add a Grok Build API-key profile using masked synthetic input.
9. Add an OpenCode local-folder account.
10. Validate accounts and set one provider default.
11. Inspect the honest overall Usage summary.
12. Inspect one provider and one account in detail.
13. Return to the Workspace Manager with focus restored.
14. Create a Workspace through the complete prelude.
15. Configure General, Mounts, Roles, Environments, and Auth.
16. Select a non-default account for the Workspace.
17. Inspect save preview and save.
18. Launch the Workspace.
19. Continue directly to launch because the operator is already inside the Construct.
20. Watch all 11 launch stages advance.
21. Open, scroll, and close the build log.
22. Complete the Capsule handoff.
23. Type into the active simulated terminal pane.
24. Open a second Agent session using a different account.
25. Split the pane.
26. Move focus and resize the split.
27. Zoom and unzoom.
28. Scroll terminal history, select text, copy, then return live.
29. Open the command palette.
30. Open Capsule Usage and inspect the active account.
31. Close the dialog and restore pane focus.
32. Detach while the instance remains active.
33. Reconnect and see the retained tabs/panes.
34. Start or expose a second instance.
35. Exit one foreground instance and remain inside the Construct while the second instance runs.
36. Observe compact feedback that another instance remains in the Construct.
37. Exit the final instance through the dirty-exit safety flow.
38. See the Construct outro and elapsed-duration caption.
39. Skip or complete the outro.
40. Return to the restored terminal.

This flow must work with keyboard only.

Mouse interaction must also work naturally throughout.

Also run the launch-failure paths: failure with another instance still running returns to the Construct; fresh-Construct failure with zero surviving instances acknowledges the failure, runs one final outro, and restores the terminal.

---

# 35. DEFINITION OF DONE

This phase is complete only when this repository contains a coherent, runnable, interactive Jackin terminal application built on the approved Junie component system.

It must cover the complete current Jackin product experience described by `JACKIN_REFERENCE.md`, plus the unified Account & Usage Center described here.

It must preserve the special feeling of entering and leaving the Construct through meaningful intro and outro rituals.

It must have been:

- built
- run
- navigated
- typed into
- clicked
- dragged
- scrolled
- resized
- visually captured
- independently critiqued
- corrected
- experienced through the full Jackin flow

The quality target is:

> This feels like entering a precise digital Construct built for orchestrating AI coding agents, their Workspaces, sessions, accounts, and capacity.

Build and run the `jackin` binary:

```bash
rtk cargo build --bin jackin
rtk cargo run --bin jackin
```

Build the world.

Enter it.

Work inside it.

Leave it cleanly.

Deliver it.
