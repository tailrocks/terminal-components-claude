# Jackin Current Product, Interface, and Workflow Reference

<a id="document-top"></a>

Current-source archaeology. No redesign spec.

**Pinned source:** `/Users/donbeave/junie-style-2/jackin` at
`8d161b3b41c64da0de3ab5f4aef1969316c193d1` (`main`, `0.6.4`,
2026-09-01 05:16:40 +07:00).

### Contents

- [Document Contract](#s1-inputs)
- [Snapshot and Coverage Summary](#s5-snapshot)
- [One-Page Product Explanation](#product-explanation)
- [Canonical Vocabulary](#canonical-vocabulary)
- [Domain Model and Scope](#domain-model-and-scope)
- [Complete Application and Surface Topology](#surface-topology)
- [Surface Inventory](#surface-inventory)
- [Per-Surface Specifications](#s11-surfaces)
- [Shared Interaction Reference](#s19-interaction)
- [Current Visual Language](#s20-visual)
- [Complete Workflow Catalog](#s21-workflows)
- [Data-Presentation Map](#s22-data-map)
- [Operator-Visible Copy and Terminology](#s23-copy)
- [Responsive and Hard-Case Inventory](#s24-hard-cases)
- [Future Preview Scenario Matrix](#s25-preview)
- [Redesign Coverage Contract](#s26-redesign-contract)
- [Targeted Source-Reading Index](#s27-index)
- [Current Versus Planned](#s29-current-planned)
- [Completeness Audits](#s31-audits)
- [Final Verification Record](#final-verification-record)

<a id="s1-inputs"></a>

## Document Contract

This file is the sole merged reference artifact. Its sole product-evidence root
is `/Users/donbeave/junie-style-2/jackin`, pinned at commit
`8d161b3b41c64da0de3ab5f4aef1969316c193d1`. Every product claim must resolve
to source, tests, baselines, or documentation inside that local clone. The
supplied briefs define coverage and method only; they are not product evidence.

No remote repository, website, package registry, search result, memory, or
unrelated local checkout may fill an evidence gap. Record such a gap as
`UNKNOWN` with the inspected local paths and missing proof.

> This document describes current product semantics and capabilities. It does
> not require the future design to preserve current layouts, styling, panels,
> colors, or navigation structure.

<a id="s2-output"></a>

### Scope and single required output

This file is the only requested artifact: `JACKIN_REFERENCE.md`.

It documents the current host console, workspace editor, Settings, launch
cockpit, Capsule TUI, Usage surfaces, shared interaction rules, workflows,
visual grammar, evidence index, and current/planned boundary at the pinned SHA.

<a id="s3-boundaries"></a>

### Absolute boundaries

- Current Jackin only. No redesigned app, mock screen, fixture, screenshot,
  generated image, or implementation patch.
- Planned or research material is labeled and never used as shipped inventory.
- “Unknown” means source evidence was insufficient. It does not mean “probably
  exists.”
- Docs are contract evidence; executable source and tests decide shipped
  behavior when they disagree.
- Every implementation claim below points to local source, exact lines, symbol,
  and the pinned SHA. See [§28](#s28-citations).

<a id="s4-question"></a>

### Primary research question

What does an operator actually see and do in every current Jackin TUI surface,
how does state move between surfaces, what data is exposed or withheld, and
which behaviors are current, partial, planned, or research-only?

<a id="s5-snapshot"></a>

## Snapshot and Coverage Summary

| Fact | Pinned result |
|---|---|
| Repository | `/Users/donbeave/junie-style-2/jackin` |
| Commit | `8d161b3b41c64da0de3ab5f4aef1969316c193d1` |
| Branch/status | `main`, clean at audit time |
| Product version | `0.6.4` |
| Host-console source files | 301 under `crates/jackin-console/src` |
| Launch source files | 50 under `crates/jackin-launch/src` |
| Capsule source files | 153 under `crates/jackin-capsule/src` |
| Usage source files | 56 under `crates/jackin-usage/src` |
| Rust files discovered | 1,235 under the pinned tree |
| Docs files discovered | 599 under the pinned tree |
| Console PNG baselines | 53 under `crates/jackin-console/src/tui/view/baselines/png` |
| Rust snapshot files | 18 matching `*.snap` / `*.snapshot` in the pinned tree |
| Test/fixture files | 438 broad `*test*`/snapshot matches; not all are TUI tests |

The 53 PNG names cover brand and non-brand states: workspace list, editor tabs,
Settings tabs, create-prelude steps, generic modal families, confirmations,
help, status, and container-info surfaces. They are evidence of current visual
states, not a new artifact produced by this task.

Source anchors: `/Users/donbeave/junie-style-2/jackin/Cargo.toml:L41-L46`
(`workspace version`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L576-L749`
(`render`); baseline registry at
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view/baselines/png`
(`PNG baseline files`).

<a id="s6-evidence"></a>

### Source of truth and evidence taxonomy

1. Executable Rust state, update, input, render, adapter, and runtime code.
2. Tests and checked-in visual baselines.
3. Local reference docs and public command docs.
4. Roadmap/research pages, labeled as future or rationale.

When a docs page says “planned” but code ships the behavior, the inventory says
current and records the docs boundary. When code and docs conflict, both are
named; neither is silently reconciled.

Evidence labels used throughout:

- `SOURCE_VERIFIED`: executable local source directly proves behavior.
- `TEST_VERIFIED`: local test or checked-in baseline proves observable behavior.
- `DOC_VERIFIED`: local documentation proves contract or intent, not shipment
  when executable source disagrees.
- `INFERRED`: conclusion follows from multiple cited local sources; inference is
  named and its premises remain visible.
- `PARTIAL`: current code implements only the stated subset; missing integration
  or behavior is named.
- `PLANNED` / `RESEARCH_ONLY`: local roadmap or research material, excluded from
  shipped inventory.
- `UNKNOWN`: documented local search found insufficient proof. Never silently
  promoted to current behavior.

<a id="s7-method"></a>

### Systematic completeness method

The audit walked four linked registries:

1. Route/state enums: console manager stages, editor/settings tabs, modal
   variants, launch stages, Capsule modes/dialogs, Usage providers.
2. Render dispatch: each enum arm, frame/layout helper, overlay precedence,
   footer/hint builder, and baseline name.
3. Input/update dispatch: keymap, mouse hit regions, focus order, scroll owner,
   async effect/result, and cancellation branch.
4. Evidence closure: tests, docs, source index, current/planned classification,
   hard-case matrix, and final diff check.

For every surface, the record answers: classification; operator purpose; entry;
exit; composition; visible content; data contract; identity/selection;
keyboard; mouse; focus; scrolling; states; opened surfaces; transitions;
long-running work; visual semantics; implementation map; redesign obligation.

<a id="s8-source-map"></a>

### Starting source map

| Domain | Canonical source areas | Responsibility |
|---|---|---|
| Config/domain | `crates/jackin-config/src`, `crates/jackin-core/src` | Workspace, mounts, roles, auth, env, agent, instance, launch facts |
| Host console | `crates/jackin-console/src/tui` | Routes, screens, state, input, update, effects, render |
| Host adapter | `crates/jackin/src/console`, `crates/jackin/src/app` | Terminal ownership, services, runtime effects, outcomes |
| Launch | `crates/jackin-launch/src/tui`, `crates/jackin-runtime/src/runtime/launch` | Progress cockpit, build log, failure, hardline handoff |
| Capsule | `crates/jackin-capsule/src/tui`, daemon/session/control modules | In-container mux, PTY sessions, status, dialogs, usage |
| Usage | `crates/jackin-usage/src`, `crates/jackin-protocol/src/usage_broker.rs` | Provider registry, accounts, projection, freshness, quota state |
| Docs contracts | `docs/content/reference/tui`, `docs/content/reference/developer-reference/specs`, public commands | Invariants, terminology, rationale, known future boundaries |

Top-level route evidence: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/stage.rs:L11-L36`
(`ConsoleManagerStage` / `ConsoleManagerStageRoute`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model.rs:L19-L52`
(`ConsoleAppStage`, `ConsoleApp`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens.rs:L4-L11`
(`screens` module registry).

<a id="s10-contract"></a>

## One-Page Product Explanation

<a id="product-explanation"></a>

### 10.1 One-page product explanation

Jackin is an operator tool for entering an isolated “Construct”: a role-backed
workspace becomes a durable runtime instance, then a Capsule process owns the
in-container PTY multiplexer and agent sessions. The host console manages
workspaces and instances; the launch cockpit exposes preparation and failure;
the Capsule is the attached terminal experience. Usage is a projection system,
not an agent identity.

The operator selects a current directory, saved workspace, or existing instance;
configures workdir/mounts/roles/env/auth; launches or reconnects; chooses an
agent/provider where offered; monitors status; and returns to the host console
when the attached Capsule exits. Durable state can outlive the terminal attach.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs:L260-L328`
(`WorkspaceConfig`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/instance.rs:L14-L133`
(`InstanceStatus`, `SessionStatus`, `SessionRecord`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/config.rs:L13-L81`
(`CapsuleConfig`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/daemon.rs:L182-L274`
(`SessionSupervisor`, `ClientRegistry`).

<a id="canonical-vocabulary"></a>

## Canonical Vocabulary

| Term | Current meaning | Not interchangeable with | Evidence |
|---|---|---|---|
| Operator | Human/controller represented by host config and runtime identity; no first-class `Operator` domain struct | Agent, provider | `crates/jackin-config/src/app_config.rs:L31-L90`; runtime identity `crates/jackin-runtime/src/runtime/identity.rs:L12-L38` |
| Construct | Shared image/runtime span from first launch through last exit | Workspace, container | `crates/jackin-runtime/src/runtime/universe.rs:L1-L8,L73-L171` |
| Role | Namespace/name-selected, trusted, validated role repository/manifest | Workspace | `crates/jackin-core/src/manifest.rs:L19-L70`; `crates/jackin-core/src/selector.rs:L16-L109` |
| Workspace | Persisted scope: workdir, mounts, roles, defaults, env, auth, runtime, dirty policy | Instance | `crates/jackin-config/src/schema.rs:L260-L328,L544-L584` |
| Mount | Host source to container destination with readonly/isolation semantics | Agent home, auth file | `crates/jackin-config/src/schema.rs:L73-L106`; `crates/jackin-runtime/src/runtime/launch/mounts.rs:L23-L185` |
| Agent | Closed runtime selection: Claude, Codex, Amp, Kimi, OpenCode, Grok | Usage surface/provider | `crates/jackin-core/src/agent.rs:L20-L185` |
| Provider | Launch/provider adapter identity, separate from agent and usage surface | Agent runtime | `crates/jackin-protocol/src/lib.rs:L242-L320`; `crates/jackin-protocol/src/provider_adapter.rs:L25-L303` |
| Instance | Durable host/container identity and lifecycle | Session | `crates/jackin-core/src/instance.rs:L14-L74,L112-L162` |
| Session | Agent runtime/terminal session record or live Capsule PTY session | Instance | `crates/jackin-core/src/instance.rs:L78-L110`; `crates/jackin-capsule/src/session.rs:L17-L35,L138-L218` |
| Capsule | In-container daemon and terminal multiplexer/control plane | Host console | `crates/jackin-capsule/src/config.rs:L13-L81`; `crates/jackin-capsule/src/daemon.rs:L182-L274` |
| Usage surface | Provider-specific quota/account projection: Claude, Codex, Amp, Grok, Z.AI, Kimi, MiniMax, OpenCode, Unsupported | Agent | `crates/jackin-usage/src/usage.rs:L200-L313` |
| Account | A provider credential/quota identity in the projection | Auth mode | `crates/jackin-protocol/src/control.rs:L519-L581` |
| Freshness | Current, stale, refreshing, failed projection quality | Quota status | `crates/jackin-protocol/src/usage_broker.rs:L201-L275` |

## Domain Model and Scope

```text
Operator host/config
        │ resolves defaults, auth, env, role sources
        ▼
Workspace ── selects/trusts ──> Role/Manifest ── selects ──> Agent runtime
   │                                  │                         │
   │ mounts/workdir/policy             │ image/construct         │ durable home/auth
   ▼                                  ▼                         ▼
Launch pipeline ───────────────> Instance/Container ───────> Capsule daemon
                                      │                         │
                                      │ one or more              │ one active attach
                                      ▼                         ▼
                                Session records          PTY panes/tabs/sessions
                                      │                         │
                                      └──────── Usage capability/projection ──> Host Usage
```

This diagram expresses ownership and data flow, not a proposed architecture.
The current model intentionally has partial multi-session representation:
Capsule can host multiple sessions while durable instance records retain
singular `agent_runtime` fields in key views.

<a id="surface-topology"></a>

## Complete Application and Surface Topology

```text
jackin CLI
├─ Console / bare interactive jackin
│  └─ ConsoleApp(Manager)
│     ├─ List: workspace tree + preview + list/status overlays
│     ├─ CreatePrelude: linear first-mount/workdir/name wizard
│     ├─ Editor: General | Mounts | Roles | Environments | Auth
│     ├─ Settings: General | Mounts | Environments | Auth | Trust
│     ├─ ConfirmDelete / ConfirmInstancePurge
│     └─ Usage overlay over List
├─ Launch / Load
│  └─ Launch cockpit: 11-stage rail + build log/failure/container/quit overlays
└─ Hardline / attached instance
   └─ Capsule TUI
      ├─ status bar + branch/context bar + pane tree
      ├─ normal/prefix/dialog/drag/select modes
      ├─ command palette, agent/provider/close/split/exec pickers
      ├─ usage dialog and read-only info dialogs
      └─ attached agent/session PTYs
```

CLI command evidence: `/Users/donbeave/junie-style-2/jackin/crates/jackin/src/cli.rs:L101-L168`
(`Command`); bare-console behavior: `/Users/donbeave/junie-style-2/jackin/crates/jackin/src/cli/role.rs:L132-L145`
(`ConsoleArgs`, interactive dispatch); console route/render evidence in
`crates/jackin-console/src/tui/model/stage.rs:L11-L36` and
`crates/jackin-console/src/tui/view.rs:L112-L169,L576-L749`.

## Surface Inventory

| ID | Surface | Classification | Entry | Exit/destination |
|---|---|---|---|---|
| C-LIST | Workspace manager list | Current | Console startup / `Esc` | Launch, edit, Settings, create, Usage, quit |
| C-USAGE | Host Usage overlay | Current | `u` / usage action; startup projection is broker-populated | `Esc`/`q` returns List |
| C-PRELUDE | Create workspace prelude | Current | `+ New workspace` / create action | Editor on complete; List on cancel |
| C-EDITOR | Workspace editor | Current | `e` on saved workspace; create completion | Save to List; cancel/back to List |
| C-SETTINGS | Global Settings | Current | `s` from List | Save/back to List |
| C-MODAL | Host modal/popup families | Current | Route actions | Result branch or parent surface |
| L-COCKPIT | Launch progress cockpit | Current | Load/launch outcome | Capsule hardline, failure exit, cancel/quit |
| K-CAPSULE | In-construct Capsule | Current | Hardline attach | Detach/exit to host; session actions stay inside |
| K-USAGE | Capsule Usage dialog | Current | Prefix `u` / palette Usage | Close returns Capsule |
| K-INFO | Capsule read-only info dialogs | Current | Container/GitHub context actions | Dismiss returns Capsule |

Host Usage startup and refresh wiring: `/Users/donbeave/junie-style-2/jackin/crates/jackin/src/console/adapter/run.rs:L49-L133,L859-L864,L1070-L1087`
(`load_console_usage_state`, input-loop invocation, startup projection load).

Implementation topology: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/stage.rs:L11-L62,L199-L243`
(`ManagerStage`, dispatch plans); `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/model.rs:L25-L84`
(`LaunchView`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/model.rs:L15-L61`
(`MuxMode`, `MuxModeState`).

### Inventory detail contract

The compact inventory above is supplemented here with the required type,
purpose, data, opened surfaces, and implementation evidence columns. IDs are
stable document IDs, not source enum discriminants.

| ID | Type | Operator purpose | Data contract | Opens | Source |
|---|---|---|---|---|---|
| L-BUILDLOG | Launch opaque overlay | Inspect Docker build output | Bounded ANSI lines, scroll/tail state | None; returns cockpit | `crates/jackin-launch/src/tui/components/build_log_dialog.rs:L46-L300` — overlay |
| L-FAILURE | Launch failure overlay | Understand safe launch failure | Summary/stage/run ID/detail/next-step | Copy run ID only | `crates/jackin-launch/src/tui/components/failure_dialog.rs:L25-L71,L250-L310` — failure |
| L-CONTAINERINFO | Launch read-only overlay | Inspect runtime identity/debug | Version/container/role/agent/target/run ID | None; copy/dismiss | `crates/jackin-launch/src/tui/components/container_info_dialog.rs:L21-L120` — info |
| K-DIALOG | Capsule modal | Execute/dismiss one capsule action | Dialog variant/action state | Spawn/split/close/exec/exit branches | `crates/jackin-capsule/src/tui/components/dialog.rs:L353-L440` — DialogAction |

<a id="atomic-child-inventory"></a>

### Atomic child inventory

Aggregate IDs remain navigation anchors. These 87 child IDs are canonical enum-
level identities; later sections supply their visible content, interactions,
states, and redesign obligations. Model names and visible labels remain separate
where they differ.

| Child ID | Parent | Model variant / visible label | Local source |
|---|---|---|---|
| EDITOR-GENERAL | [C-EDITOR](#c-editor) | `General` / General | `crates/jackin-console/src/tui/screens/editor/model.rs:L21-L48` |
| EDITOR-MOUNTS | [C-EDITOR](#c-editor) | `Mounts` / Mounts | `crates/jackin-console/src/tui/screens/editor/model.rs:L21-L48` |
| EDITOR-ROLES | [C-EDITOR](#c-editor) | `Roles` / Roles | `crates/jackin-console/src/tui/screens/editor/model.rs:L21-L48` |
| EDITOR-ENVIRONMENTS | [C-EDITOR](#c-editor) | `Secrets` / Environments | `crates/jackin-console/src/tui/screens/editor/model.rs:L21-L48` |
| EDITOR-AUTH | [C-EDITOR](#c-editor) | `Auth` / Auth | `crates/jackin-console/src/tui/screens/editor/model.rs:L21-L48` |
| SETTINGS-GENERAL | [C-SETTINGS](#c-settings) | `General` / General | `crates/jackin-console/src/tui/screens/settings/model.rs:L48-L88` |
| SETTINGS-MOUNTS | [C-SETTINGS](#c-settings) | `Mounts` / Mounts | `crates/jackin-console/src/tui/screens/settings/model.rs:L48-L88` |
| SETTINGS-ENVIRONMENTS | [C-SETTINGS](#c-settings) | `Environments` / Environments | `crates/jackin-console/src/tui/screens/settings/model.rs:L48-L88` |
| SETTINGS-AUTH | [C-SETTINGS](#c-settings) | `Auth` / Auth | `crates/jackin-console/src/tui/screens/settings/model.rs:L48-L88` |
| SETTINGS-TRUST | [C-SETTINGS](#c-settings) | `Trust` / Trust | `crates/jackin-console/src/tui/screens/settings/model.rs:L48-L88` |
| SETTINGS-MODAL-MOUNT-TEXT | [C-SETTINGS](#c-settings) | `MountText` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-MOUNT-FILE | [C-SETTINGS](#c-settings) | `MountFileBrowser` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-MOUNT-DST | [C-SETTINGS](#c-settings) | `MountDstChoice` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-MOUNT-SCOPE | [C-SETTINGS](#c-settings) | `MountScopePicker` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-MOUNT-ROLE | [C-SETTINGS](#c-settings) | `MountRolePicker` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-MOUNT-CONFIRM | [C-SETTINGS](#c-settings) | `MountConfirm` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-MOUNT-PREVIEW | [C-SETTINGS](#c-settings) | `MountPreviewSave` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-ENV-TEXT | [C-SETTINGS](#c-settings) | `EnvText` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-ENV-SOURCE | [C-SETTINGS](#c-settings) | `EnvSourcePicker` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-ENV-OP | [C-SETTINGS](#c-settings) | `EnvOpPicker` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-ENV-ROLE | [C-SETTINGS](#c-settings) | `EnvRolePicker` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-ENV-SCOPE | [C-SETTINGS](#c-settings) | `EnvScopePicker` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-ENV-CONFIRM | [C-SETTINGS](#c-settings) | `EnvConfirm` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-AUTH-TEXT | [C-SETTINGS](#c-settings) | `AuthTextInput` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-AUTH-SOURCE | [C-SETTINGS](#c-settings) | `AuthSourcePicker` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-AUTH-OP | [C-SETTINGS](#c-settings) | `AuthOpPicker` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-AUTH-FOLDER | [C-SETTINGS](#c-settings) | `AuthSourceFolderPicker` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| SETTINGS-MODAL-AUTH-FORM | [C-SETTINGS](#c-settings) | `AuthForm` | `crates/jackin-console/src/tui/screens/settings/model.rs:L618-L697` |
| MODAL-TEXT-INPUT | [C-MODAL](#c-modal) | `TextInput` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-FILE-BROWSER | [C-MODAL](#c-modal) | `FileBrowser` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-MOUNT-DST | [C-MODAL](#c-modal) | `MountDstChoice` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-WORKDIR | [C-MODAL](#c-modal) | `WorkdirPick` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-CONFIRM | [C-MODAL](#c-modal) | `Confirm` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-SAVE-DISCARD | [C-MODAL](#c-modal) | `SaveDiscardCancel` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-GITHUB | [C-MODAL](#c-modal) | `GithubPicker` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-CONFIRM-SAVE | [C-MODAL](#c-modal) | `ConfirmSave` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-ERROR | [C-MODAL](#c-modal) | `ErrorPopup` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-CONTAINER-INFO | [C-MODAL](#c-modal) | `ContainerInfo` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-STATUS | [C-MODAL](#c-modal) | `StatusPopup` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-OP | [C-MODAL](#c-modal) | `OpPicker` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-ROLE | [C-MODAL](#c-modal) | `RolePicker` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-ROLE-OVERRIDE | [C-MODAL](#c-modal) | `RoleOverridePicker` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-AUTH-ROLE | [C-MODAL](#c-modal) | `AuthRolePicker` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-SOURCE | [C-MODAL](#c-modal) | `SourcePicker` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-AUTH-SOURCE | [C-MODAL](#c-modal) | `AuthSourcePicker` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-SCOPE | [C-MODAL](#c-modal) | `ScopePicker` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| MODAL-AUTH-FORM | [C-MODAL](#c-modal) | `AuthForm` | `crates/jackin-console/src/tui/model/modal.rs:L47-L112` |
| USAGE-CLAUDE | [C-USAGE](#c-usage) / [K-USAGE](#k-usage) | `Claude` / Anthropic | `crates/jackin-usage/src/usage.rs:L226-L294` |
| USAGE-CODEX | [C-USAGE](#c-usage) / [K-USAGE](#k-usage) | `Codex` / OpenAI | `crates/jackin-usage/src/usage.rs:L226-L294` |
| USAGE-AMP | [C-USAGE](#c-usage) / [K-USAGE](#k-usage) | `Amp` / Amp | `crates/jackin-usage/src/usage.rs:L226-L294` |
| USAGE-GROK | [C-USAGE](#c-usage) / [K-USAGE](#k-usage) | `Grok` / xAI | `crates/jackin-usage/src/usage.rs:L226-L294` |
| USAGE-ZAI | [C-USAGE](#c-usage) / [K-USAGE](#k-usage) | `Zai` / Z.AI | `crates/jackin-usage/src/usage.rs:L226-L294` |
| USAGE-KIMI | [C-USAGE](#c-usage) / [K-USAGE](#k-usage) | `Kimi` / Kimi | `crates/jackin-usage/src/usage.rs:L226-L294` |
| USAGE-MINIMAX | [C-USAGE](#c-usage) / [K-USAGE](#k-usage) | `Minimax` / MiniMax | `crates/jackin-usage/src/usage.rs:L226-L294` |
| USAGE-OPENCODE | [C-USAGE](#c-usage) / [K-USAGE](#k-usage) | `OpenCode` / OpenCode | `crates/jackin-usage/src/usage.rs:L226-L294` |
| USAGE-UNSUPPORTED | [C-USAGE](#c-usage) / [K-USAGE](#k-usage) | `Unsupported` / Usage | `crates/jackin-usage/src/usage.rs:L226-L294` |
| LAUNCH-IDENTITY | [L-COCKPIT](#l-cockpit) | `Identity` / identity | `crates/jackin-core/src/launch_progress.rs:L14-L102` |
| LAUNCH-ROLE | [L-COCKPIT](#l-cockpit) | `Role` / role | `crates/jackin-core/src/launch_progress.rs:L14-L102` |
| LAUNCH-CREDENTIALS | [L-COCKPIT](#l-cockpit) | `Credentials` / credentials | `crates/jackin-core/src/launch_progress.rs:L14-L102` |
| LAUNCH-CONSTRUCT | [L-COCKPIT](#l-cockpit) | `Construct` / construct | `crates/jackin-core/src/launch_progress.rs:L14-L102` |
| LAUNCH-AGENT-BINARIES | [L-COCKPIT](#l-cockpit) | `AgentBinaries` / agent binaries | `crates/jackin-core/src/launch_progress.rs:L14-L102` |
| LAUNCH-DERIVED-IMAGE | [L-COCKPIT](#l-cockpit) | `DerivedImage` / derived image | `crates/jackin-core/src/launch_progress.rs:L14-L102` |
| LAUNCH-WORKSPACE | [L-COCKPIT](#l-cockpit) | `Workspace` / workspace | `crates/jackin-core/src/launch_progress.rs:L14-L102` |
| LAUNCH-NETWORK | [L-COCKPIT](#l-cockpit) | `Network` / network | `crates/jackin-core/src/launch_progress.rs:L14-L102` |
| LAUNCH-SIDECAR | [L-COCKPIT](#l-cockpit) | `Sidecar` / sidecar | `crates/jackin-core/src/launch_progress.rs:L14-L102` |
| LAUNCH-CAPSULE | [L-COCKPIT](#l-cockpit) | `Capsule` / capsule | `crates/jackin-core/src/launch_progress.rs:L14-L102` |
| LAUNCH-HARDLINE | [L-COCKPIT](#l-cockpit) | `Hardline` / hardline | `crates/jackin-core/src/launch_progress.rs:L14-L102` |
| CAPSULE-MODE-NORMAL | [K-CAPSULE](#k-capsule) | `Normal` | `crates/jackin-capsule/src/tui/model.rs:L15-L61` |
| CAPSULE-MODE-PREFIX | [K-CAPSULE](#k-capsule) | `PrefixAwait` | `crates/jackin-capsule/src/tui/model.rs:L15-L61` |
| CAPSULE-MODE-DIALOG | [K-CAPSULE](#k-capsule) | `Dialog` | `crates/jackin-capsule/src/tui/model.rs:L15-L61` |
| CAPSULE-MODE-DRAG | [K-CAPSULE](#k-capsule) | `Drag` | `crates/jackin-capsule/src/tui/model.rs:L15-L61` |
| CAPSULE-MODE-SELECT | [K-CAPSULE](#k-capsule) | `Select` | `crates/jackin-capsule/src/tui/model.rs:L15-L61` |
| CAPSULE-DIALOG-PALETTE | [K-DIALOG](#k-dialog) | `CommandPalette` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-AGENT | [K-DIALOG](#k-dialog) | `AgentPicker` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-RENAME | [K-DIALOG](#k-dialog) | `RenameTab` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-EXPORT | [K-DIALOG](#k-dialog) | `ExportFile` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-CONTAINER | [K-INFO](#k-info) | `ContainerInfo` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-GITHUB | [K-INFO](#k-info) | `GitHubContext` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-USAGE | [K-USAGE](#k-usage) | `Usage` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-SPAWN-FAILURE | [K-DIALOG](#k-dialog) | `SpawnFailure` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-SPLIT | [K-DIALOG](#k-dialog) | `SplitDirectionPicker` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-CLOSE-TARGET | [K-DIALOG](#k-dialog) | `CloseTargetPicker` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-CONFIRM | [K-DIALOG](#k-dialog) | `ConfirmAction` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-PROVIDER | [K-DIALOG](#k-dialog) | `ProviderPicker` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-EXEC | [K-DIALOG](#k-dialog) | `ExecPicker` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-EXIT-DIRTY | [K-DIALOG](#k-dialog) | `ExitDirty` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |
| CAPSULE-DIALOG-EXIT-INSPECT | [K-DIALOG](#k-dialog) | `ExitInspect` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L287` |


<a id="s11-surfaces"></a>

## Per-Surface Specifications

The records below are the current interaction contract. “Future redesign
obligation” states what a later implementation must preserve semantically; it
does not prescribe placement, colors, or component shape.

<a id="c-list"></a>

### 11.1 C-LIST — Workspace manager

| Field | Current contract |
|---|---|
| Classification | Current host-console stage; manager `List`. |
| Operator purpose | Select a directory/workspace/instance; launch, reconnect, edit, prewarm, inspect, stop, purge, delete, or open Settings. |
| Entry conditions | Console startup; return from Editor/Settings/create/confirm; `Esc` from Usage. Selection follows the current directory’s saved workspace when present. |
| Exit/destination | Launch/restore/new session/shell/inspect leave for host handling; `e` → Editor; `s` → Settings; `n` on sentinel → CreatePrelude; `u` → Usage; `q`/`Esc` → quit/back ladder. |
| Layout/composition | Header; split body with names/tree left and selected-row preview right; fixed workspace footer. Default split is 30%, clamped 20–80%, draggable seam. |
| Complete visible content | `Current directory`; saved workspace rows; instance children; `+ New workspace`; disclosure glyphs; selected `▸`; preview sections General, Mounts, Environments, Roles; instance live tab/pane tree or session rows; status/footer hints. |
| Data contract | Workspace summary: name, workdir, mount counts, allowed roles, default/last role. Instances: id, container base, workspace, workdir, role, agent, lifecycle status, timestamps. Preview may include daemon snapshot, sessions, mount info, env, roles. |
| Selection/identity | `ManagerListRow`: current dir, current-dir instance, saved workspace, workspace instance, new workspace. Purged/superseded instances are hidden. |
| Keyboard | `↑/↓`/`k/j`; `←/→` tree disclosure; `h/l` horizontal scroll; Enter context action; `E`, `N`, `D`, `W`, `O`, `S`; instance `R/A/X/I/T/P`; Tab preview; Esc/q; Ctrl-Q quit confirmation. |
| Mouse | Click row selects; click URL opens when allowed; wheel scrolls focused pane; drag seam resizes split; scrollbar/preview hit regions are routed before base rows. |
| Focus | Two keyboard nodes: names list then Preview. Tab enters preview; preview arrows move panes; Enter attaches selected pane; Esc/Left/BackTab returns. Footer and scroll blocks are not focus nodes. |
| Scrolling/overflow | Left tree and right preview own vertical scroll; list/detail also support horizontal offsets; focused block controls scroll. Long labels are clipped; session names truncate with ellipsis. |
| States/variants | Empty current directory; no saved workspaces; expanded/collapsed; active/running/clean-exited/crashed/preserved-dirty/preserved-unpushed/restore-available/superseded/purged/failed-setup; daemon unavailable; sessions unavailable; no sessions; no daemon tabs; status/error popup. Loading is implicit pending async state, with no explicit loading renderer found. |
| Opened surfaces | Inline agent/provider/role/new-session pickers; list modal (role, container info, status/error, GitHub); CreatePrelude; Editor; Settings; delete/purge confirms; Usage; Keyboard help. |
| Long-running work | Throttled instance refresh, daemon/exec session inventory, mount-info refresh, role/picker/file-browser loads, prewarm, GitHub URL resolution, and config operations are async effects. |
| Current visual semantics | Workspace and instance tones differ; current selection uses `▸`; live/status labels are compact; right pane is a detail projection, not a second source of truth. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/model.rs:L10-L83` — `ManagerListRow`, `WorkspaceSummary`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L52-L95,L205-L342,L1284-L1614` — row, preview, instance renderers; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/update.rs:L25-L121,L222-L267` — action/focus plans; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/keymap.rs:L986-L1258` — keymap. |
| Future redesign obligation | Preserve row identity, instance lifecycle meaning, launch/reconnect ladder, preview-vs-list focus, explicit destructive confirmations, and the distinction between manifest sessions and live daemon snapshots. |

Sources are all at the pinned SHA. The workspace frame/render contract is
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L576-L684`
(`render`); list topology is
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view/list.rs:L31-L94`
(`render_list_body`).

#### C-LIST transitions

| From | Event | Result |
|---|---|---|
| List row | Enter current directory | Launch current directory |
| List row | Enter saved workspace | Launch named workspace |
| Instance row | Enter/R | Reconnect/restore path, or error if unavailable |
| Instance row | A | New session; agent/provider selection may open |
| Instance row | X/I/T/P | Shell/inspect/stop/purge path |
| Saved workspace | E | Editor with original and pending config snapshots |
| Saved workspace | D | ConfirmDelete stage |
| New workspace | Enter/N | CreatePrelude |
| Any List | S | Settings |
| Any List | U/u | Usage overlay |
| Any List | ? | Keyboard help |

Evidence: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/list.rs:L45-L197,L198-L325`
(`handle_list_key`, action outcomes); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/state/update.rs:L91-L145,L180-L300`
(`update_manager`).

<a id="c-usage"></a>

### 11.2 C-USAGE — Host Usage overlay

Atomic registry children: [USAGE-CLAUDE](#atomic-child-inventory), [USAGE-CODEX](#atomic-child-inventory), [USAGE-AMP](#atomic-child-inventory), [USAGE-GROK](#atomic-child-inventory), [USAGE-ZAI](#atomic-child-inventory), [USAGE-KIMI](#atomic-child-inventory), [USAGE-MINIMAX](#atomic-child-inventory), [USAGE-OPENCODE](#atomic-child-inventory), [USAGE-UNSUPPORTED](#atomic-child-inventory).

| Field | Current contract |
|---|---|
| Classification | Current broker-populated overlay over List; not a manager stage. |
| Operator purpose | Inspect projected provider accounts, lifecycle, quota windows, freshness, and errors. |
| Entry conditions | Console startup loads the broker's current projection into manager state; List `u` clones those accounts and notice into `UsageScreenState`. |
| Exit/destination | Esc/q returns to List; no launch-side effect. |
| Layout/composition | Reuses workspace header/footer; header title `usage`; 30/70 account-list/detail split. Overview is row zero. |
| Complete visible content | Overview; grouped provider/account rows; status; quota window label, meter/value, reset time; detail fields Provider, Account, Status, Limits; unresolved notice. |
| Data contract | `UsageAccount`: provider/account label/status/windows; projection freshness maps stale available to `stale`; unresolved provider notice is retained. |
| Selection/identity | Index 0 is Overview; account index is one-based offset. Detail toggles selected account. |
| Keyboard | Esc/q close; `↑/↓`/`j/k` select; Enter detail; PageUp/PageDown scroll; `r` asks the adapter to reload the broker projection and updates accounts/notice in place. |
| Mouse | No dedicated Usage click contract found beyond the console’s modal/overlay routing; wheel is consumed by the active surface. |
| Focus | Usage owns selection/detail state; it is opened as a List overlay and returns to the List selection. |
| Scrolling/overflow | Detail body scrolls; long account/window text is clipped by the screen area. |
| States/variants | available, stale, not started, needs login, needs secret, unsupported, unavailable, error; empty `No providers configured.` plus refresh hint; unresolved notice. |
| Opened surfaces | None; provider detail stays inside Usage. |
| Long-running work | Broker discovery/current reads occur in the host adapter at startup and on `r`; success replaces staged accounts/notice, failure preserves the route and shows `Usage unavailable: …`. |
| Current visual semantics | Severity meters use red/yellow/green; provider grouping separates identity from quota status. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/usage.rs:L20-L95,L102-L165` — `UsageScreenState`, projection/keys; `:L167-L362` — list/detail/meter render; `/Users/donbeave/junie-style-2/jackin/crates/jackin/src/console/adapter/run.rs:L49-L133,L859-L864,L1070-L1087` — startup/current projection, input-loop invocation, and `r` refresh; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L751-L767` — overlay frame. |
| Future redesign obligation | Preserve provider/account identity, freshness, lifecycle vs quota distinction, unresolved/error copy, and read-only projection semantics. |

Usage source contract: `/Users/donbeave/junie-style-2/jackin/crates/jackin-protocol/src/usage_broker.rs:L201-L275`
(`Freshness`, quota state, windows, issues); provider registry:
`/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage.rs:L200-L313`
(`UsageSurface`, IDs, labels, order).

<a id="c-prelude"></a>

### 11.3 C-PRELUDE — Create workspace prelude

| Field | Current contract |
|---|---|
| Classification | Current linear wizard stage retaining workspace chrome. |
| Operator purpose | Bootstrap the first mount, workdir, and name before entering the full Editor. |
| Entry conditions | List `+ New workspace` / create action. |
| Exit/destination | Completed five-step input enters Editor; Esc/cancel returns List; file-browser cancellation cancels prelude. |
| Layout/composition | Workspace frame with the active modal; no list body. Modal area is prepared from workspace content area and footer reservation. |
| Complete visible content | Steps: Mount source; Mount destination; optional Edit destination; Working directory; Workspace name. File browser rows, destination choices, workdir picker, text inputs, readonly choice, and context hints. |
| Data contract | Pending mount source/destination/readonly/workdir/name; first mount becomes a `MountConfig`; resulting `WorkspaceConfig` uses the chosen workdir and one mount, other fields default. |
| Selection/identity | Wizard step index; browser remembers last cwd; `used_edit_dst` controls rewind path. |
| Keyboard | Modal controls use shared text/file-browser/picker keymaps; Esc rewinds/cancels according to active step. |
| Mouse | File browser and modal hit regions; wheel/selection delegated to active modal. |
| Focus | Active modal control only; parent wizard is blocked. |
| Scrolling/overflow | File browser and workdir picker scroll; text fields horizontally scroll as needed. |
| States/variants | In progress; completed; cancelled; same-path fast path; edited destination; browser error/empty/error popup. No review phase; optional destination edit may be skipped. |
| Opened surfaces | FileBrowser; MountDstChoice; destination TextInput; WorkdirPick; name TextInput; URL/error branches. |
| Long-running work | File listing/navigation/commit validation and Git URL resolution are async effects. |
| Current visual semantics | Wizard labels communicate linear progress; default destination is source absolute path; default workspace name is destination basename. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/create_prelude.rs:L11-L33,L49-L64,L145-L268` — wizard/state/plans; `:L277-L378` — pending values/build; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/prelude.rs:L3-L173` — modal flow; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/file_browser.rs:L588-L626` — commit transition. |
| Future redesign obligation | Preserve step order, cancellation rewind, same-path/default behavior, readonly flag, and exact handoff into editable workspace config. |

#### C-PRELUDE transitions

| Step | Commit/result | Next |
|---|---|---|
| Mount source | Host path / Git URL resolved | Mount destination choice |
| Destination choice | Same path | Workdir picker |
| Destination choice | Edit | Destination text input |
| Destination choice | Cancel | Browser reopens at last cwd |
| Destination text | Commit | Workdir picker |
| Workdir picker | Commit | Name text input |
| Workdir picker | Cancel after edit | Destination text input |
| Workdir picker | Cancel otherwise | Destination choice |
| Name text | Commit | Editor |
| Any step | Esc/cancel | List, unless a parent rewind branch applies |

Evidence: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/create_prelude.rs:L219-L268`
(`CreatePrelude*Plan`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L120-L124,L654-L659`
(`CreatePrelude` frame/modal).

<a id="c-editor"></a>

### 11.4 C-EDITOR — Workspace editor

Atomic registry children: [EDITOR-GENERAL](#atomic-child-inventory), [EDITOR-MOUNTS](#atomic-child-inventory), [EDITOR-ROLES](#atomic-child-inventory), [EDITOR-ENVIRONMENTS](#atomic-child-inventory), [EDITOR-AUTH](#atomic-child-inventory).

| Field | Current contract |
|---|---|
| Classification | Current manager stage; create or edit mode. |
| Operator purpose | Edit workspace identity, workdir, mounts, allowed/default roles, environments, auth, keep-awake and Git-pull policy. |
| Entry conditions | List `e`; create-prelude completion. State holds original and pending config. |
| Exit/destination | Save flow returns List after async config write; clean Esc returns List; dirty Esc opens Save/Discard/Cancel; errors keep Editor/modal parent. |
| Layout/composition | Header, five-tab strip, active body, dynamic contextual footer; tab bar/content focus; modal backdrop preserves footer. |
| Complete visible content | General, Mounts, Roles, Environments (model `Secrets`), Auth; rows, sentinels, selection, dirty count, contextual hints. |
| Data contract | Pending/original `WorkspaceConfig`, pending name, modal chain, expanded/unmasked sets, scroll state, async operations, save planner/commit state. |
| Selection/identity | Tab `General → Mounts → Roles → Secrets/Environments → Auth`; row focus per tab; spacer/source rows are not focusable. |
| Keyboard | Global `s/S` save, Esc; tab Left/Right/Tab/BackTab; rows Up/Down/k/j; h/l horizontal where applicable; Enter immediate action. Per-tab keys are detailed in [§13](#s13-editor). |
| Mouse | Tab/mount/auth row selection; focus transfer; scrollbar drag/wheel; modal precedence blocks background. |
| Focus | TabBar or TabContent; tab-content row plans skip non-focusable spacers; modal focus blocks editor. |
| Scrolling/overflow | Content vertical scroll; mounts horizontal and vertical; secret/auth rows expand; long paths/source refs clip/scroll. |
| States/variants | Create/Edit; clean/dirty; role not in registry; mounted workspace running; isolated mount requiring cleanup; source drift; validation/save/op/token errors. |
| Opened surfaces | TextInput, FileBrowser, WorkdirPick, MountDstChoice, Confirm, SaveDiscardCancel, GithubPicker, ConfirmSave, ErrorPopup, OpPicker, RolePicker, RoleOverridePicker, SourcePicker, AuthSourcePicker, ScopePicker, AuthForm, AuthRolePicker. |
| Long-running work | Role load/registration/trust, file browser, GitHub resolution, 1Password validation/commit, Claude token generation, config save, isolation cleanup. |
| Current visual semantics | Checkboxes and `★` communicate role allow/default; masked secrets show deliberate concealment; save preview displays `+/-/~` field changes. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/model.rs:L21-L49,L249-L295,L356-L510` — tabs/state/row targets; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/frame.rs:L45-L118` — frame/tab dispatch; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/editor.rs:L127-L523,L538-L1104` — input/modal/save dispatch. |
| Future redesign obligation | Preserve pending-vs-original isolation, row identity, secret masking, validation-before-write, save preview, dirty exit, trust/auth scope, and async failure recovery. |

Editor model evidence: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/model.rs:L21-L49`
(`EditorTab`); visible row evidence in
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/general_tab.rs:L39-L70`,
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/mounts_tab.rs:L65-L154`,
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/roles_tab.rs:L19-L49`,
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/secrets_tab.rs:L23-L64,L93-L124,L191-L238`, and
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/auth_tab.rs:L23-L87`.

<a id="c-settings"></a>

### 11.5 C-SETTINGS — Global Settings

Atomic registry children: [SETTINGS-GENERAL](#atomic-child-inventory), [SETTINGS-MOUNTS](#atomic-child-inventory), [SETTINGS-ENVIRONMENTS](#atomic-child-inventory), [SETTINGS-AUTH](#atomic-child-inventory), [SETTINGS-TRUST](#atomic-child-inventory), [SETTINGS-MODAL-MOUNT-TEXT](#atomic-child-inventory), [SETTINGS-MODAL-MOUNT-FILE](#atomic-child-inventory), [SETTINGS-MODAL-MOUNT-DST](#atomic-child-inventory), [SETTINGS-MODAL-MOUNT-SCOPE](#atomic-child-inventory), [SETTINGS-MODAL-MOUNT-ROLE](#atomic-child-inventory), [SETTINGS-MODAL-MOUNT-CONFIRM](#atomic-child-inventory), [SETTINGS-MODAL-MOUNT-PREVIEW](#atomic-child-inventory), [SETTINGS-MODAL-ENV-TEXT](#atomic-child-inventory), [SETTINGS-MODAL-ENV-SOURCE](#atomic-child-inventory), [SETTINGS-MODAL-ENV-OP](#atomic-child-inventory), [SETTINGS-MODAL-ENV-ROLE](#atomic-child-inventory), [SETTINGS-MODAL-ENV-SCOPE](#atomic-child-inventory), [SETTINGS-MODAL-ENV-CONFIRM](#atomic-child-inventory), [SETTINGS-MODAL-AUTH-TEXT](#atomic-child-inventory), [SETTINGS-MODAL-AUTH-SOURCE](#atomic-child-inventory), [SETTINGS-MODAL-AUTH-OP](#atomic-child-inventory), [SETTINGS-MODAL-AUTH-FOLDER](#atomic-child-inventory), [SETTINGS-MODAL-AUTH-FORM](#atomic-child-inventory).

| Field | Current contract |
|---|---|
| Classification | Current manager stage for global settings; separate from workspace Editor. |
| Operator purpose | Edit global General policy, global Mounts, global Environments, global Auth, and Trust registry. |
| Entry conditions | List `s/S`; returns from save or cancel. |
| Exit/destination | Save writes global config asynchronously and returns List on success; dirty back opens discard; error popup stays in Settings. |
| Layout/composition | Header 3 rows, five tabs 2 rows, active body min 5 rows, dynamic contextual footer. Modal layer priority is error → mounts → env → auth. |
| Complete visible content | General: coauthor trailer, DCO; Mounts: global mount rows; Environments: global/role env rows; Auth: global/provider auth; Trust: role-source trust rows. |
| Data contract | `SettingsState` plus General/Mounts/Env/Auth/Trust pending/original state, selected auth kind, op/token request, modal chains, error popup, footer height. |
| Selection/identity | Tabs `General → Mounts → Environments → Auth → Trust`; content selection is tab-local; env rows carry global/role scope; auth rows target global/workspace-role scope. |
| Keyboard | Tab bar Left/Right/Tab/BackTab; General arrows/Space/S/Esc; Mounts arrows/h/l/S/R/A/D/O/N/1/2/3/Enter; Env arrows/A/S/D/M/P/Enter; Auth arrows/Enter/S/Esc; Trust arrows/h/l/Space/S/Esc. |
| Mouse | Tab selection, row/focus selection, wheel/scrollbar, modal controls; modal regions shadow settings body. |
| Focus | TabBar or active tab content; modal focus replaces content focus; Esc may clear selected auth kind before leaving. |
| Scrolling/overflow | Mounts and content scroll clamp to frame; long source/destination/role/env values clip or scroll. |
| States/variants | Clean/dirty; auth selected/unselected; env masked/unmasked/expanded; trusted/untrusted source; op unavailable; save/error; no rows. |
| Opened surfaces | Mount text/file-browser/destination/scope/role/confirm/preview; env text/source/op/role/scope/confirm; auth text/source/op/source-folder/form; error popup. |
| Long-running work | 1Password listing/validation/commit, Claude token mint, file-browser listing/validation, GitHub resolution, config save. |
| Current visual semantics | Settings global scope is distinct from Editor workspace scope; trust is a policy list; dirty footer shows contextual change count. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model.rs:L48-L107,L523-L697` — tabs/state/form/modal variants; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/view.rs:L101-L460` — frame/header/tab/body/footer; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/keymap.rs:L254-L889` — settings/global-mount keymaps; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model/general_impls.rs:L10-L67`, `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model/env_impls.rs:L10-L202`, `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model/auth_impls.rs:L12-L315`, `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model/trust_impls.rs:L10-L131` — state behavior. |
| Future redesign obligation | Preserve global-vs-workspace scope, auth/env/mount/trust semantics, dirty/save/discard safety, secret masking, and modal parent restoration. |

Settings/editor distinction is source-backed, not stylistic: Editor owns a
pending `WorkspaceConfig` while Settings owns global policy substate. Compare
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/model.rs:L249-L295`
(`EditorState`) with
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model.rs:L90-L107`
(`SettingsState`).

<a id="l-cockpit"></a>

### 11.6 L-COCKPIT — Launch progress

Atomic registry children: [LAUNCH-IDENTITY](#atomic-child-inventory), [LAUNCH-ROLE](#atomic-child-inventory), [LAUNCH-CREDENTIALS](#atomic-child-inventory), [LAUNCH-CONSTRUCT](#atomic-child-inventory), [LAUNCH-AGENT-BINARIES](#atomic-child-inventory), [LAUNCH-DERIVED-IMAGE](#atomic-child-inventory), [LAUNCH-WORKSPACE](#atomic-child-inventory), [LAUNCH-NETWORK](#atomic-child-inventory), [LAUNCH-SIDECAR](#atomic-child-inventory), [LAUNCH-CAPSULE](#atomic-child-inventory), [LAUNCH-HARDLINE](#atomic-child-inventory).

| Field | Current contract |
|---|---|
| Classification | Current transitional host TUI. |
| Operator purpose | Show launch identity, 11-stage progress, activity, build diagnostics, failure, and handoff readiness. |
| Entry conditions | Direct Load or console launch outcome; host terminal session enters launch view. |
| Exit/destination | Hardline attach to Capsule; noninteractive return; failure acknowledgement/exit; prompt cancellation; quit/hard abort. |
| Layout/composition | Header, body/rain, progress rail, hint row, status footer; opaque build-log/failure/container/quit overlays. |
| Complete visible content | Brand/header, `Preparing launch...` or `Loading <role> in <target>`, stage rail, active activity, container chip, optional debug run ID, hints, failure/build details. |
| Data contract | `LaunchView`: identity, ordered `StageView`s, status/activity, failure, build-log state, container info, quit confirm, frame/motion. |
| Selection/identity | Current stage frontier; failed stage wins; run ID/invocation and container identity are copyable/displayed according to mode. |
| Keyboard | Ctrl-C hard abort; Ctrl-Q quit confirmation; build log Esc/arrows/j/k/PageUp/PageDown; failure Enter/Esc; container info Enter copy/Esc close. |
| Mouse | Footer activity opens build log; container/debug chip opens info; failure copy target copies; outside failure acknowledges; scrollbar drag scrolls build log/detail. |
| Focus | Active overlay owns focus; failure/container/build body swallows unrelated clicks. |
| Scrolling/overflow | Build log tail-follow/scroll; failure and container info body scroll; wrapped long lines use `↳`. |
| States/variants | Queued/running/done/skipped/failed/blocked model; runtime does not emit Blocked in audited paths; motion/no-motion; debug/non-debug; interactive/noninteractive. |
| Opened surfaces | Build log, failure, container info, quit confirmation. |
| Long-running work | Pipeline stages, Docker image builds, role/workspace preparation, network/sidecar/capsule health, hardline attach. |
| Current visual semantics | Rail communicates stage frontier and completion; activity is compact; debug IDs are amber/diagnostic; failure freezes motion. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/launch_progress.rs:L14-L177` — stage/status/identity/failure facts; `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/model.rs:L25-L84` — view state; `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/progress_rail.rs:L15-L245`, `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/header.rs:L15-L118`, `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/footer.rs:L60-L181`, `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/build_log_dialog.rs:L46-L300`, `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/failure_dialog.rs:L25-L71`, `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/container_info_dialog.rs:L21-L120` — composition; `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/view.rs:L25-L178` — precedence/frame. |
| Future redesign obligation | Preserve stage order, failure acknowledgement/copy semantics, hard abort/quit distinction, durable container identity, and handoff ownership. |

<a id="k-capsule"></a>

### 11.7 K-CAPSULE — In-construct Capsule

Atomic registry children: [CAPSULE-MODE-NORMAL](#atomic-child-inventory), [CAPSULE-MODE-PREFIX](#atomic-child-inventory), [CAPSULE-MODE-DIALOG](#atomic-child-inventory), [CAPSULE-MODE-DRAG](#atomic-child-inventory), [CAPSULE-MODE-SELECT](#atomic-child-inventory).

| Field | Current contract |
|---|---|
| Classification | Current in-container attached TUI and multiplexer. |
| Operator purpose | Operate tabs/panes and agent PTYs while preserving terminal fidelity; inspect branch/context, usage, status, and session state. |
| Entry conditions | Launch reaches Capsule ready then Hardline; reconnect attaches to a live container. |
| Exit/destination | Detach/prefix detach, exit confirmation, session/tab/pane close; host receives terminal ownership after Capsule exits. |
| Layout/composition | Two-row status bar, optional branch/context bar, pane tree, scrollbars/selection/cursor, bottom chrome; modal backdrop when dialog is active. |
| Complete visible content | Tabs, focus/active glyph, agent labels/state, pane labels, branch/PR context, usage chips, container/debug chip, terminal output, menu/prefix hint, toasts/notices/tooltips. |
| Data contract | Daemon snapshots and events: tabs, focused pane, pane/session IDs, labels, agent, public state, output, usage/control views; one attach client and multiple PTY sessions. |
| Selection/identity | Active tab, focused pane, hovered tab/menu/branch/status/container/debug/copy target; agent cursor/output state gates hardware cursor. |
| Keyboard | Ctrl-Q global; configured prefix then c/n/x/hjkl/"/%/z/p/&/Ctrl-L/d/u/space/:/r; 0–9 jump tabs; Alt-Shift arrows resize; palette/dialog keymaps. |
| Mouse | Pane/status/menu/branch/dialog targets; click/drag/select; wheel/scrollback; dialog captures clicks and wheel; clipboard/host URL modifiers route outward. |
| Focus | Normal, PrefixAwait, Dialog, Drag, Select; dialog > drag > select > prefix > normal priority; cursor hidden in scrollback/dialog/no output/no pane. |
| Scrolling/overflow | Pane scrollback, dialog/detail scroll, selection drag; tabs overflow with `›`; labels truncate to custom-label max 16 where applicable. |
| States/variants | Agent Idle/Working/Done/Blocked/Unknown; shell vs agent vs mixed/Agents panes; tab/pane active; modal/prefix/drag/select; daemon/socket/session unavailable. |
| Opened surfaces | Command palette, agent/provider picker, rename, export, container info, GitHub context, Usage, spawn failure, split direction, close target, confirm action, exec picker, dirty exit/inspect. |
| Long-running work | PTY input/output, daemon status detection, usage refresh, clipboard/export, Git/PR resolution, agent spawn/exit, socket attach. |
| Current visual semantics | Status glyphs encode public agent state; branch/context bar carries source context; pane chrome is subordinate to terminal output; menus expose operator actions. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/model.rs:L15-L229` — modes/hover/cursor/status; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/view.rs:L173-L324` — frame; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/status_bar.rs:L50-L340`, `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/branch_context_bar.rs:L52-L205`, `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog.rs:L146-L440`, `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/palette.rs:L29-L148` — visible chrome/dialogs; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/input.rs:L197-L335,L643-L867` and `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/keymap.rs:L22-L467` — routing/keymap. |
| Future redesign obligation | Preserve byte/terminal fidelity, PTY routing, mode priority, attach/detach semantics, public state vocabulary, and safe host-action boundaries. |

<a id="c-modal"></a>

### 11.8 C-MODAL — Host concrete modal/picker surface

Atomic registry children: [MODAL-TEXT-INPUT](#atomic-child-inventory), [MODAL-FILE-BROWSER](#atomic-child-inventory), [MODAL-MOUNT-DST](#atomic-child-inventory), [MODAL-WORKDIR](#atomic-child-inventory), [MODAL-CONFIRM](#atomic-child-inventory), [MODAL-SAVE-DISCARD](#atomic-child-inventory), [MODAL-GITHUB](#atomic-child-inventory), [MODAL-CONFIRM-SAVE](#atomic-child-inventory), [MODAL-ERROR](#atomic-child-inventory), [MODAL-CONTAINER-INFO](#atomic-child-inventory), [MODAL-STATUS](#atomic-child-inventory), [MODAL-OP](#atomic-child-inventory), [MODAL-ROLE](#atomic-child-inventory), [MODAL-ROLE-OVERRIDE](#atomic-child-inventory), [MODAL-AUTH-ROLE](#atomic-child-inventory), [MODAL-SOURCE](#atomic-child-inventory), [MODAL-AUTH-SOURCE](#atomic-child-inventory), [MODAL-SCOPE](#atomic-child-inventory), [MODAL-AUTH-FORM](#atomic-child-inventory).

| Field | Current contract |
|---|---|
| Classification | Current child surface owned by a host stage; blocks parent input and returns a typed result. |
| Operator purpose | Complete one path, picker, credential, confirmation, diagnostic, or save-preview decision. |
| Entry conditions | List, CreatePrelude, Editor, or Settings action opens the variant. |
| Exit/destination | Enter commits; Esc cancels/rewinds; validation keeps it open; result restores parent or routes next stage. |
| Layout/composition | Centered bordered dialog over dimmed content; title, body/input/list, buttons/hints, optional scrollbar; footer remains reserved. |
| Complete visible content | Variant title, target/context, input or selectable rows, validation/error text, action row, hints, and any copy/status detail. |
| Data contract | Variant-specific path, mount, env, role/provider/source, auth, instance/container, status/error, or save-diff state; secrets masked. |
| Selection/identity | Active variant plus target key/path/row/provider/role/instance; parent identity remains retained. |
| Keyboard | Active child owns focus; Tab/BackTab/arrows move controls; Enter commits; Esc cancels; picker query/scroll/select are local. |
| Mouse | Dialog hit regions, list rows, scrollbar, copy, and external-link targets precede parent rows. |
| Focus | Modal focus barrier blocks parent stage and returns focus to the parent target after result. |
| Scrolling/overflow | File/picker/detail bodies scroll; long values wrap/clip; modal rectangle shrinks to viewport. |
| States/variants | Empty, loading/pending, validation error, unavailable, success, cancel, and variant-specific result branches where source-backed. |
| Opened surfaces | FileBrowser, MountDstChoice, WorkdirPick, GithubPicker, OpPicker, Role/Source/Scope pickers, AuthForm, Confirm, SaveDiscardCancel, ConfirmSave, ErrorPopup, StatusPopup, ContainerInfo. |
| Long-running work | Filesystem/GitHub/role/secret-provider reads, validation, token minting, cleanup, and save effects return through typed messages. |
| Current visual semantics | Backdrop and border establish modality; destructive actions use explicit labels; info/status dialogs are read-only and dismissible. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/modal.rs:L22-L119,L169-L253`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L423-L488,L634-L747`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/editor.rs:L538-L968`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/list.rs:L504-L693`. |
| Future redesign obligation | Preserve variant identity, typed result branches, focus trapping, parent restoration, validation, masking, cancellation, and reserved-footer geometry. |

<a id="l-buildlog"></a>

### 11.9 L-BUILDLOG — Launch build-log overlay

| Field | Current contract |
|---|---|
| Classification | Current launch diagnostic overlay; exposes bounded build output without leaving the launch run. |
| Operator purpose | Inspect build output while retaining launch stage identity and control. |
| Entry conditions | Footer/activity or build action opens it during an active launch. |
| Exit/destination | Esc closes to Launch Cockpit; failure may replace it; quit/hard abort remains separate. |
| Layout/composition | Centered dialog with wrapped log lines, tail-follow viewport, scrollbar, title, and hints. |
| Complete visible content | Build title, bounded ANSI/log lines, continuation wraps, scroll position, tail state, scrollbar, and hints. |
| Data contract | Current launch run plus retained diagnostic lines; no secret-bearing credential material. |
| Selection/identity | Run identity and current log cursor/offset; no cross-run log mixing. |
| Keyboard | Arrows/j/k/PageUp/PageDown scroll; Esc closes; launch quit/abort keys keep their typed paths. |
| Mouse | Wheel and scrollbar drag scroll; footer activity opens this overlay. Build-log content has no copy action. |
| Focus | Overlay owns focus; background launch controls are blocked except explicitly routed abort/quit. |
| Scrolling/overflow | Empty, accumulating, tail-follow, manually scrolled, truncated/retained; wrapped lines fit dialog. |
| States/variants | Open/closed, no output, active stream, tail-follow, manual scroll, retained/truncated, pipeline failure. |
| Opened surfaces | Returns Launch Cockpit; failure overlay may replace it when the pipeline fails. |
| Long-running work | Build log subscription/retention updates while pipeline continues. |
| Current visual semantics | Diagnostic text is secondary to the stage rail; wrapped continuation lines use the launch continuation glyph. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/build_log_dialog.rs:L46-L300`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/footer.rs:L60-L181`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/keymap.rs:L59-L141`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/subscriptions.rs:L239-L293,L405-L425,L810-L835`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-diagnostics/src/build_log.rs:L16-L69`. |
| Future redesign obligation | Preserve run association, bounded diagnostics, scroll/tail semantics, and non-destructive return to the active launch. |

<a id="l-failure"></a>

### 11.10 L-FAILURE — Launch failure overlay

| Field | Current contract |
|---|---|
| Classification | Current terminal launch failure surface; freezes progress and gives safe acknowledgement/copy/exit choices. |
| Operator purpose | Explain the failed stage and provide safe diagnostic acknowledgement. |
| Entry conditions | Pipeline failure produces a `LaunchFailure` for the active run. |
| Exit/destination | Enter/Esc acknowledge or exit per result; handler/retry outcome remains outside this overlay. |
| Layout/composition | Failure title, failed stage, summary/detail, wrapped cause, run/container identity, action row, scrollable body. |
| Complete visible content | Failed stage, safe summary, detail/next step, run ID, copy target, action labels, and scroll affordance. |
| Data contract | `LaunchFailure`, target, run ID, stage, detail; credentials excluded from copyable diagnostics. |
| Selection/identity | Failed stage and run/container identity remain stable while detail scrolls. |
| Keyboard | Enter/Esc acknowledge/exit; dialog actions and scrolling are local. |
| Mouse | Copy target, wheel, scrollbar, and outside acknowledgement are routed to dialog behavior. |
| Focus | Dialog owns focus; background launch controls are blocked. |
| Scrolling/overflow | Short/long detail, wrapped cause, copy result, acknowledged, exit/handoff. |
| States/variants | Failed/frozen, detail scrolled, copy success/failure, acknowledged, external handler result. |
| Opened surfaces | Dismiss to launch failure result; no false success or implicit retry. |
| Long-running work | None after failure except copy/acknowledgement and adapter cleanup. |
| Current visual semantics | Failure freezes motion and uses high-severity treatment; stage identity remains visible. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/failure_dialog.rs:L25-L71,L250-L310`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/update.rs:L16-L239`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/run.rs:L220-L235,L412-L472`. |
| Future redesign obligation | Preserve failed-stage identity, safe copy, acknowledgement semantics, and separation of quit, cancel, and retry outcomes. |

<a id="l-containerinfo"></a>

### 11.11 L-CONTAINERINFO — Launch container/debug info

| Field | Current contract |
|---|---|
| Classification | Current read-only launch identity/debug dialog. |
| Operator purpose | Inspect and copy runtime identity without mutating the launch. |
| Entry conditions | Container/debug chip or failure/info action opens it. |
| Exit/destination | Enter/Esc dismisses to Launch Cockpit or failure parent; copy is explicit. |
| Layout/composition | Bordered info body with target, role, agent, container/run/debug IDs, wrapped values, and hints. |
| Complete visible content | Target, role, agent, container/run/debug identity, absent-field state, copy target, and dismiss hint. |
| Data contract | `LaunchIdentity` and container/debug metadata; absent fields omitted or unavailable, never guessed. |
| Selection/identity | Current launch invocation/container/run identity; copy target identifies exact selected value. |
| Keyboard | Enter/Esc dismiss; copy and scrolling are local. |
| Mouse | Copy, wheel, scrollbar, and dismiss hit regions are active; background blocked. |
| Focus | Dialog captures focus and restores parent focus after close. |
| Scrolling/overflow | Compact/wrapped info; copy success/failure; absent debug identity. |
| States/variants | Runtime/debug field present/absent, wrapped, copy success/failure, parent cockpit/failure. |
| Opened surfaces | Returns to Launch Cockpit or failure overlay parent. |
| Long-running work | Clipboard operation only; launch pipeline continues behind the dialog. |
| Current visual semantics | Diagnostic identifiers are visibly secondary and copy-oriented. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/container_info_dialog.rs:L21-L120`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/launch_progress.rs:L117-L177`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/subscriptions.rs:L557-L639`. |
| Future redesign obligation | Preserve durable identity, omitted-field semantics, copy boundaries, and parent pipeline ownership. |

<a id="k-usage"></a>

### 11.12 K-USAGE — Capsule Usage dialog

Atomic registry children: [CAPSULE-DIALOG-USAGE](#atomic-child-inventory), [USAGE-CLAUDE](#atomic-child-inventory), [USAGE-CODEX](#atomic-child-inventory), [USAGE-AMP](#atomic-child-inventory), [USAGE-GROK](#atomic-child-inventory), [USAGE-ZAI](#atomic-child-inventory), [USAGE-KIMI](#atomic-child-inventory), [USAGE-MINIMAX](#atomic-child-inventory), [USAGE-OPENCODE](#atomic-child-inventory), [USAGE-UNSUPPORTED](#atomic-child-inventory).

| Field | Current contract |
|---|---|
| Classification | Current in-Capsule read-only quota/usage dialog. |
| Operator purpose | Inspect and refresh scoped provider/account quota without changing config or PTY state. |
| Entry conditions | Usage chip or dialog action opens it from attached Capsule. |
| Exit/destination | Escape closes to Capsule normal mode; `r` refreshes in place; no launch/config branch. |
| Layout/composition | Dialog with Overview/provider tabs, account identity, quota windows, reset/remaining values, severity meter, refresh hint, detail body. |
| Complete visible content | Tabs, provider/account identity, status, windows, values/reset, meter, unresolved/error copy, refresh and close hints. |
| Data contract | Capsule usage projection keyed by provider/account/window; stale/unavailable/error remain visible. |
| Selection/identity | Overview or provider/account tab; selected provider and focused account remain distinct. |
| Keyboard | Dialog owns arrows/tabs/Enter/actions, `r` refresh, and wheel/Page scrolling; pane input blocked. |
| Mouse | Dialog rows, tabs, wheel, and scrollbar are local; background PTY is blocked. |
| Focus | Dialog mode owns focus and restores prior Capsule mode on close. |
| Scrolling/overflow | Detail scrolls; narrow view switches to single-column meter layout; long labels clip/wrap. |
| States/variants | Overview/provider; available/stale/not-started/needs-login/needs-secret/unsupported/error; empty/unresolved/refresh-pending. |
| Opened surfaces | Returns Capsule normal mode; provider detail remains inside the dialog. |
| Long-running work | Usage relay refresh/cache request; stale data remains labeled until replacement. |
| Current visual semantics | Green/yellow/red meter severity; identity is distinct from quota status. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog/usage.rs:L8-L180`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog_widgets/usage.rs:L117-L186,L516-L593,L760-L824`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/input.rs:L643-L867`. |
| Future redesign obligation | Preserve read-only semantics, provider/account identity, refresh/freshness labels, meter thresholds, and dialog focus ownership. |

<a id="k-info"></a>

### 11.13 K-INFO — Capsule read-only info/context dialogs

Atomic registry children: [CAPSULE-DIALOG-CONTAINER](#atomic-child-inventory), [CAPSULE-DIALOG-GITHUB](#atomic-child-inventory).

| Field | Current contract |
|---|---|
| Classification | Current Capsule informational surfaces for container/debug identity, GitHub/branch context, export, and status details. |
| Operator purpose | Inspect public runtime/context facts and invoke explicit copy/open actions. |
| Entry conditions | Status, branch/context, container/debug, or action menu opens a typed dialog. |
| Exit/destination | Esc/close returns to prior Capsule mode; external URL/clipboard is an explicit boundary. |
| Layout/composition | Bordered modal with title, context rows or wrapped detail, optional copy/open controls, dismissal hints. |
| Complete visible content | Public tab/pane/container facts, branch/PR/remote, session/status, copy/open target, unavailable/error text. |
| Data contract | Current tab/pane/container, branch/PR/remote, session, public status; secrets/hidden credential values omitted. |
| Selection/identity | Dialog target identifies current tab/pane/container/context; copy target is explicit. |
| Keyboard | Dialog owns keys for scroll, dismiss, copy, and open; parent pane input is blocked. |
| Mouse | Dialog rows, wheel, scrollbar, clipboard, and URL modifier targets are local. |
| Focus | Dialog focus barrier restores prior Capsule mode on close. |
| Scrolling/overflow | Compact/wrapped/overflow; unavailable context, clipboard failure, no-pane/no-daemon states. |
| States/variants | Container/debug, GitHub/branch, export/status, unavailable, copy failure, external action. |
| Opened surfaces | Capsule normal/prefix mode; external URL/clipboard action may leave the TUI. |
| Long-running work | Git/PR resolution, daemon query, export, and clipboard effects may be pending. |
| Current visual semantics | Context and diagnostics remain subordinate to terminal content; copy/open targets are explicit. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog.rs:L146-L440`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/branch_context_bar.rs:L52-L205,L345-L395`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/input.rs:L350-L590`. |
| Future redesign obligation | Preserve public-only context, typed copy/open actions, focus barriers, and no-pane/daemon-unavailable honesty. |

<a id="k-dialog"></a>

### 11.14 K-DIALOG — Capsule action dialogs and palette

Atomic registry children: [CAPSULE-DIALOG-PALETTE](#atomic-child-inventory), [CAPSULE-DIALOG-AGENT](#atomic-child-inventory), [CAPSULE-DIALOG-RENAME](#atomic-child-inventory), [CAPSULE-DIALOG-EXPORT](#atomic-child-inventory), [CAPSULE-DIALOG-SPAWN-FAILURE](#atomic-child-inventory), [CAPSULE-DIALOG-SPLIT](#atomic-child-inventory), [CAPSULE-DIALOG-CLOSE-TARGET](#atomic-child-inventory), [CAPSULE-DIALOG-CONFIRM](#atomic-child-inventory), [CAPSULE-DIALOG-PROVIDER](#atomic-child-inventory), [CAPSULE-DIALOG-EXEC](#atomic-child-inventory), [CAPSULE-DIALOG-EXIT-DIRTY](#atomic-child-inventory), [CAPSULE-DIALOG-EXIT-INSPECT](#atomic-child-inventory). K-INFO and K-USAGE own the three specialized dialog IDs.

| Field | Current contract |
|---|---|
| Classification | Current action surfaces for palette, spawn/exec, rename, split, close, dirty exit, and confirmations. |
| Operator purpose | Execute one explicit Capsule action while preserving target/session identity and safety gates. |
| Entry conditions | Prefix/menu/action opens a modal or palette. |
| Exit/destination | Selection/Enter commits a typed command; Esc cancels; dirty exit requires an explicit branch. |
| Layout/composition | Palette query/list or action dialog with title, selectable rows, validation/error text, hints, optional scroll. |
| Complete visible content | Query/action rows, target tab/pane/session, command/choice, validation/error, confirmation, and hints. |
| Data contract | Target tab/pane/session, agent/provider, split direction, close target, command, and exit policy. |
| Selection/identity | Selected action and target pane/tab/session are stable across query/confirm steps. |
| Keyboard | Query/row navigation, prefix commands, Enter, Esc, and dialog-specific keys are local. |
| Mouse | Palette/dialog rows, wheel, scrollbar, and confirmation controls are local. |
| Focus | Dialog/palette captures focus; background PTY is blocked until result. |
| Scrolling/overflow | Query/list/detail scroll; long actions clip; no-match and empty states are explicit. |
| States/variants | Query empty/no match, unavailable agent/provider, spawn failure, split/close confirmation, dirty choices, success return. |
| Opened surfaces | New pane/tab, agent/session, split, close, detach/exit, or Capsule normal mode. |
| Long-running work | Agent spawn/exit, daemon command, PTY/session setup, pane layout, and detach/takeover effects. |
| Current visual semantics | Destructive actions are confirmed; palette/action rows are distinct from terminal output; failure is recoverable. |
| Implementation map | `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/palette.rs:L29-L148`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog.rs:L146-L440`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/input.rs:L197-L335,L350-L590`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/keymap.rs:L22-L467`. |
| Future redesign obligation | Preserve action target identity, explicit destructive/dirty branches, PTY isolation, recoverable errors, and daemon ownership. |

<a id="s12-console"></a>

### Host console coverage

#### 12.1 Startup and ownership

Interactive bare `jackin` and the Console command enter the same host console
experience. The host adapter acquires cwd, Docker/command/in-place handlers,
probes `op` availability, starts background work where applicable, and runs the
console inside a terminal session. Raw mode, alternate screen, mouse capture,
diagnostics buffering, cursor restoration, and teardown belong to the host
adapter/terminal guard.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin/src/app/load_cmd.rs:L173-L230`
(`handle_console` setup); `/Users/donbeave/junie-style-2/jackin/crates/jackin/src/console/adapter/run.rs:L1032-L1241`
(`run_console` event loop); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/terminal.rs:L77-L143`
(`TerminalSession` ownership).

#### 12.2 Startup selection and list/detail

Initial stage is List. Workspaces are projected from config; selection follows
the current directory’s saved workspace if available. The left pane is the
names/tree selection surface. The right pane is a selected-row projection:
general fields, mounts, environments, roles, or instance live/session state.
Tree disclosure is `None`, `Collapsed`, or `Expanded`, rendered as no glyph,
`▶`, or `▼`.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/state/manager.rs:L109-L179`
(`ManagerState::new`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L52-L95,L615-L742`
(`Disclosure`, list rendering); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view/list.rs:L31-L94`
(`render_list_body`).

#### 12.3 Launch/new session/reconnect

Current directory and saved workspace Enter actions create launch outcomes.
Live instances can reconnect, create a new session, open a shell, inspect,
stop, or purge. Provider/agent selection is inline for new-session and launch
paths. Reconnect/restore and host terminal handoff are adapter outcomes, not
render-only transitions.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/list.rs:L45-L197,L265-L325`
(`handle_list_key`, `instance_action_outcome`); `/Users/donbeave/junie-style-2/jackin/crates/jackin/src/app/restore.rs:L112-L180`
(`restore`/new session paths); `/Users/donbeave/junie-style-2/jackin/crates/jackin/src/console/adapter/run.rs:L647-L696,L801-L810`
(`ConsoleInPlaceHandler` outcomes).

#### 12.4 Create/edit/delete/Settings/Usage

- Create uses the five-step prelude in [§11.3](#s11-surfaces).
- Edit uses the five-tab Editor with pending/original snapshots.
- Delete workspace uses `ConfirmDelete`; instance purge uses
  `ConfirmInstancePurge`; both return to List after the typed effect.
- Settings is a full stage, not a modal over List.
- Usage is an overlay over List, with account selection/detail.

Route/render proof: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/stage.rs:L11-L36`
(`ConsoleManagerStage`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L112-L169,L641-L725`
(`route` and modal render plans); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/dispatch.rs:L346-L462`
(`create/delete/purge transitions`).

#### 12.5 Refresh/help/debug/quit

Refresh is async and throttled; refresh failure clears derived instance surfaces,
collapses preview, and opens a deduplicated error popup. `?` opens live merged
Keyboard shortcuts help. Debug mode exposes invocation/run correlation and
reserves a chip area. Ctrl-Q opens quit confirmation; Ctrl-C is the hard escape
path in the host loop; plain `q` follows the current stage/modal ladder.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/state/manager.rs:L767-L818`
(`instance refresh`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/components/keyboard_help.rs:L40-L88`
(`console_help_entries`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/debug.rs:L82-L170`
(`debug facts`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/run.rs:L31-L47,L192-L251,L225-L234`
(`quit/help dispatch`).

#### 12.6 Input/render adapter trace

```text
crossterm event
  → console input dispatcher / modal precedence
  → typed ManagerMessage or ManagerEffect
  → update_manager reducer
  → host adapter starts/polls service effect
  → result message updates state
  → prepare_for_render
  → route renderer + footer + one visible overlay
```

Dispatch priority is help, list modal, inline new-session/provider/agent/role
pickers, editor modal, Settings error/mount/env/auth dialogs, create-prelude
modal, then stage. The render layer makes the same route/overlay ownership
explicit; only one modal overlay is presented at a time even when parent state
is retained for restoration.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/stage.rs:L46-L62,L199-L243`
(`ConsoleInputDispatchPlan`, resolver); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/dispatch.rs:L47-L130`
(`dispatch`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L492-L574,L576-L749`
(`prepare_for_render`, `render`).

#### 12.7 Empty/loading/errors/hard cases

| Case | Current visible behavior |
|---|---|
| No saved workspaces | Current-directory row and `+ New workspace`; selected-row explainer/detail |
| No sessions | `No sessions recorded` |
| Manifest session read failure | `Sessions unavailable (manifest read error)` |
| Live daemon with no tabs | `Daemon reports no tabs` |
| Instance refresh failure | Derived instance surfaces clear; deduplicated error popup |
| Loading | No explicit loading copy found in manager/editor renderers; pending async state is implicit |
| Long names/paths | Row/detail clipping and horizontal scroll; session names truncate with ellipsis |
| Many rows | Focused list block scrolls; expanded children participate in visual index |
| Horizontal + vertical overflow | List/detail/editor mounts/settings mounts have independent offsets |
| Mouse/focus | Clickable rows and scrollbars are routed by hit regions; modal blocks background |

Evidence: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L1284-L1614`
(`instance/session rendering`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/update.rs:L655-L713,L835-L886,L1120-L1245`
(`visual rows`, scrolling, preview); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/mouse.rs:L90-L307`
(`mouse routing`).

<a id="s13-editor"></a>

### Workspace editor coverage

#### 13.1 Tab inventory

| Order | Model | Visible label | Content |
|---:|---|---|---|
| 1 | `General` | General | Name, Working dir, Keep awake, Git pull |
| 2 | `Mounts` | Mounts | Destination, mode, isolation, source/kind, add/remove |
| 3 | `Roles` | Roles | Allowed roles, default role, registry status, load |
| 4 | `Secrets` | Environments | Workspace and role env vars, mask/op/scope |
| 5 | `Auth` | Auth | Workspace and role auth mode/source/folder |

The visible “Environments” label is the deliberate UI name for model
`EditorTab::Secrets`.

Source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/model.rs:L21-L49`
(`EditorTab`, `ALL`, labels).

#### 13.2 General

Rows are `Name`, `Working dir`, `Keep awake`, and `Git pull`. Keep awake shows
enabled/disabled with `(macOS only)` when enabled; Git pull shows enabled or
disabled. Name edit uses a text input; workdir uses a picker; toggles mutate
pending state only.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/general_tab.rs:L39-L70`
(`general_form_section`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/model.rs:L160-L208`
(`EditorImmediateAction`).

#### 13.3 Mounts

Each row presents destination, readonly mode, isolation, mount kind, and an
optional host source continuation. The `+ Add mount` sentinel opens source
selection/file browser. Isolation cycles `Shared → Worktree → Clone → Shared`.
Mount delete, readonly toggle, horizontal/vertical scroll, GitHub open, and
destination editing are contextual actions. Source/destination validation is
performed before save.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/mounts_tab.rs:L65-L154`
(`mount_lines`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/keymap.rs:L140-L252`
(`EditorContentAction`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs:L73-L106,L592-L654`
(`MountConfig`, validation).

#### 13.4 Roles

Header reports `Allowed roles: all` or custom count. Rows use `[x]`/`[ ]`,
`★` for default, and `+ Load role` sentinel. Role loading resolves a source,
may require trust, updates registry state, and can open an error popup. A role
not in registry is visible as such; it is not invented as loaded.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/roles_tab.rs:L19-L49`
(`EditorRoleRow`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/editor.rs:L970-L1104`
(`role resolution`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs:L442-L456`
(`RoleSource`).

#### 13.5 Environments / secrets

Workspace key rows, role headers, role key rows, add sentinels, and spacers are
rendered. Values are masked by default and can be unmasked; op-backed values
carry an `[op]` marker. Role headers show `▼/▶ Role: <role> (<n> vars)` and may
show `(not in registry)`. Forbidden/reserved keys are rejected. Delete and
scope/source/op flows preserve the parent editor modal as needed.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/secrets_tab.rs:L23-L64,L93-L124,L191-L238`
(`secret rows`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/components/editor_rows.rs:L48-L176`
(`SecretLineRow`, masking/source); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/update.rs:L675-L687`
(`secret validation`).

#### 13.6 Auth

Auth rows represent workspace and role overrides. They show auth kind/mode,
source/source-folder, inherited values, an Add sentinel, and spacer rows.
Claude supports sync/API key/OAuth token/ignore modes; other agents expose sync,
API key, and ignore in the core registry. Auth form targets and selected kind
are state, not just text.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/auth_tab.rs:L23-L87`
(`auth rows`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/agent.rs:L93-L124`
(`auth modes`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/model.rs:L420-L510`
(`auth/confirm targets`).

#### 13.7 Save, dirty, validation, and modal parents

Editor starts with original and pending config snapshots. Dirty state includes
name and config differences; change count covers workdir, roles, mounts, env,
auth, and toggles. Save builds a planner preview, validates, shows effective
changes, then commits. Running isolated mounts and source drift can block or
require cleanup confirmation. Save failures stay visible in an error popup.

Leaving dirty Editor opens `Save changes before leaving?`; choices are Save,
Discard, Cancel. Modal parents are re-mounted after child picker/token/op
operations where needed. Successful save marks clean and returns to List.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/model/state_impl/workspace.rs:L52-L129`
(`is_dirty`, `change_count`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/save.rs:L123-L260`
(`begin_editor_save`, commit); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/components/save_discard.rs:L5-L6`
(`editor_exit_save_discard_state`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/components/confirm_save.rs:L43-L135`
(`ConfirmSave` controls).

#### 13.8 Editor transition table

| Current | Action | Next/result |
|---|---|---|
| Tab bar | Left/Right/Tab/BackTab | Previous/next tab or content focus |
| General | Name/workdir | TextInput/WorkdirPick; commit updates pending |
| Mounts | Add | FileBrowser → destination choice/edit → pending mount |
| Roles | Load | Role picker/load/trust/error or registry update |
| Environments | Add/edit/delete | scope/source/op/text/confirm child modal |
| Auth | Enter on row | auth kind/source/form child modal |
| Any clean | Esc | List |
| Any dirty | Esc | Save/Discard/Cancel modal |
| Any | S | planner/validation → save preview → commit or error |

#### 13.9 Editor mouse/focus/scroll contract

Tab and mount/auth rows are mouse-selectable. Click transfers focus; wheel and
scrollbar drag adjust the selected content scroll. Modal-open regions are
non-clickable in the underlying Editor. Tab-content focus skips spacer/source
rows that are display-only. Source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/mouse/selection.rs:L11-L24,L69-L135`
(`editor selection`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/mouse/scroll_pan.rs:L15-L123`
(`scroll focus`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/update.rs:L363-L407`
(`row bounds`).

<a id="s14-settings"></a>

### Global Settings coverage

#### 14.1 Tabs and scope

Settings tabs are `General`, `Mounts`, `Environments`, `Auth`, and `Trust`.
They edit global configuration and role-source policy. They are not the
workspace Editor: Editor edits one workspace’s pending config; Settings edits
global mounts/env/auth/trust/general state.

Source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model.rs:L48-L107`
(`SettingsTab`, `SettingsState`); config model:
`/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/app_config.rs:L31-L90`
(`AppConfig`) [path typo corrected below: `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/app_config.rs:L31-L90`].

#### 14.2 General

Two independent flags are visible: coauthor trailer and DCO. Up/Down selects,
Space toggles, S saves, and Esc/Q backs out. Dirty navigation uses a discard
confirmation. Source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model/general_impls.rs:L10-L67`
(`SettingsGeneralState`); keymap `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/keymap.rs:L347-L424`.

#### 14.3 Mounts

Global mounts show source/destination/readonly and support add, remove, readonly
toggle, source/destination/scope/role child pickers, preview/save, rename, and
GitHub URL actions. Global mount sources are modeled separately from scoped
mount entries; source/destination path validation is strict. Source:
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model.rs:L617-L697,L1114-L1200`
(`SettingsModal`, `GlobalMountsState`); config `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs:L458-L542`
(`GlobalMountConfig`, `MountEntry`).

#### 14.4 Environments

Rows distinguish global and role scopes. Add/edit/delete opens text/source/op,
role/scope, and confirm variants. Values can be masked/unmasked; op-backed
entries are explicitly marked. State tracks selected/pending/original rows,
expanded roles, modal, unmasked rows, and scroll. Source:
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model/env_impls.rs:L10-L202`
(`SettingsEnvState`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model.rs:L548-L615`
(`SettingsEnvRow`, targets).

#### 14.5 Auth

Auth rows are global/provider-oriented and can open source/op/source-folder
pickers or an AuthForm. AuthForm focus targets are Mode, SourceFolder,
CredentialSource, Save, Cancel, Reset. Claude token generation is an explicit
async path; source folders are validated before persistence. Source:
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model/auth_impls.rs:L12-L315`
(`SettingsAuthState`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model.rs:L523-L546`
(`AuthFormFocus`, target).

#### 14.6 Trust

Trust rows represent role source/git/trusted state and allow horizontal choice
and toggle. Save persists trust policy; errors stay in Settings. This is
separate from role selection in Editor and from per-load branch trust in the
launch pipeline. Source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model/trust_impls.rs:L10-L131`
(`SettingsTrustState`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs:L442-L456`
(`RoleSource`).

#### 14.7 Settings frame, modal priority, save

Frame geometry is header 3, tab strip 2, body minimum 5, footer dynamic.
Render/modal priority is ErrorPopup → Mounts → Environments → Auth → none.
Root effects start async config save; success updates `AppConfig`, marks
Settings clean, and returns List; failure stores an error popup.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/view.rs:L101-L147,L149-L372`
(`settings_frame_areas`, render priority/tab/body); `/Users/donbeave/junie-style-2/jackin/crates/jackin/src/console/effects.rs:L710-L756`
(`config save effect`).

<a id="s15-modals"></a>

### Modal, popup, overlay and picker catalog

#### 15.1 Host `ConsoleModal` families

The host console stores one visible modal per owning route, with parent state
retained for child flows. The generic catalog is exhaustive at the current
model level:

| Family / variant | Owner/target | Input/result branches |
|---|---|---|
| `TextInput` | Name, destination, env, auth credential | Commit, cancel, continue; invalid commit stays open |
| `FileBrowser` | Editor mount source, auth folder, Settings mount/auth, CreatePrelude source | Navigate, navigate up, Git URL resolve/open, commit path, cancel |
| `MountDstChoice` | CreatePrelude/Settings mount | Same path, edit destination, cancel/reopen browser |
| `WorkdirPick` | CreatePrelude/editor workdir | Commit, cancel/rewind |
| `Confirm` | Delete env, trust role, isolated cleanup, destructive action | Confirm/cancel |
| `SaveDiscardCancel` | Dirty Editor/Settings exit | Save, discard, cancel |
| `GithubPicker` | GitHub source/context | Select/cancel/open URL |
| `ConfirmSave` | Editor/Settings save preview | Save/cancel; preview scroll |
| `ErrorPopup` | Validation/service/save/op/token failures | Dismiss; parent may restore |
| `ContainerInfo` | List/debug container | Copy/dismiss/scroll |
| `StatusPopup` | In-place action status | Dismiss/timeout |
| `OpPicker` | Environment/auth 1Password source | Existing/new selection/cancel/parent restore |
| `RolePicker` | List/editor role load | Select/cancel |
| `RoleOverridePicker` | Editor role environment/auth override | Select/cancel |
| `AuthRolePicker` | Auth role target | Select/cancel |
| `SourcePicker` | Env/auth source | Plain/op source/cancel |
| `AuthSourcePicker` | Auth source kind | Select/cancel |
| `ScopePicker` | Env/mount scope | Global/workspace/role scope/cancel |
| `AuthForm` | Workspace/global/role auth target | Mode/source/folder/save/cancel/reset |

Source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/modal.rs:L22-L119,L169-L253`
(`ConsoleModal`, `SecretsPickerTarget`, modal helpers); renderer dispatch:
`/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L423-L488`
(`render_modal`).

#### 15.2 Generic modal behavior

Text input commits on Enter; validation failure leaves it open. Esc selects the
cancel action when available. Tab/BackTab cycles controls; arrows move enabled
buttons; `y/n` are quick answers in text dialogs. File/picker dialogs own
navigation, query, scroll, and result selection. Modal render installs hit and
focus barriers, dims the content region, and leaves the reserved footer visible.

Source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/modal.rs:L22-L113`
(`modal state`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L634-L639,L727-L747`
(`backdrop/overlay`); shared modal behavior is implemented by `termrock` and
consumed by the host components at `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/components`.

#### 15.3 Concrete invocation matrix

| Invocation | Title/copy | Buttons/validation | Downstream |
|---|---|---|---|
| Workspace delete | `Delete workspace?` plus workspace identity | Cancel/Delete | `ConfirmDelete` → RemoveWorkspace |
| Instance purge | Purge warning with instance/container label | Cancel/Purge | `ConfirmInstancePurge` → purge effect |
| Dirty editor/settings exit | `Save changes before leaving?` | Save/Discard/Cancel | Save, reload, or remain |
| Save preview | Effective config diff | Save/Cancel; scroll | Config commit or parent |
| Env deletion | Delete key/value/scope | Cancel/Delete | Remove pending row |
| Role trust | Trust role source | Cancel/Trust | Persist role trust/load |
| Isolated cleanup | Isolated mount cleanup warning | Cancel/continue/save | Cleanup then save or remain |
| Settings error | Service/validation title + detail | Dismiss | Restore parent where needed |
| Container info | Container, role, agent, target, run/debug id | Copy/dismiss | Clipboard/dismiss |
| Status | Action result | Dismiss | Return owning route |
| New session | Agent/provider/role selection | Select/cancel | Host launch outcome |
| Global/role auth | Auth form/source/credential | Save/cancel/reset | Pending auth config |
| Env source | Plain text or 1Password | Select/cancel | Pending env value |
| Mount add | File source → destination/scope/role | Select/commit/cancel | Pending mount |

Concrete title/message strings are collected in [§23](#s23-copy). Constructor
and open-site map: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/editor.rs:L538-L968`
(`handle_editor_modal`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/list.rs:L504-L693`
(`handle_list_modal`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/prelude.rs:L93-L173`
(`handle_prelude_modal`); Settings variants at `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model.rs:L617-L697`.

#### 15.4 Modal blockers and priority

Dispatch priority is explicit: help; list modal; inline new-session/provider;
launch provider; inline agent; inline role; Editor modal; Settings error;
Settings mounts; Settings env; Settings auth; CreatePrelude modal; active
stage. Render gives status overlay its own later pass; help draws its own
backdrop and cannot coexist with another modal. Parent chains are stateful but
only the current child is visible.

Source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/stage.rs:L46-L62,L199-L243`
(`ConsoleInputDispatchPlan`, resolver); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L641-L747`
(`ConsoleModalRenderPlan`, status/help precedence).

#### 15.5 CreatePrelude concrete chain

FileBrowser → MountDstChoice → optional destination TextInput → WorkdirPick →
name TextInput. Destination cancel reopens the browser at its last cwd; workdir
cancel rewinds to destination input when edit was used, otherwise to choice.
There is no review phase. Source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/create_prelude.rs:L19-L33,L219-L268`
(`wizard/plans`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/file_browser.rs:L588-L626`
(`prelude file-browser outcome`).

#### 15.6 Modal invocation, focus, validation, and result matrix

| ID | Constructor/open site | Exact copy or title | Default focus | Validation/cancel | Downstream result | Evidence |
|---|---|---|---|---|---|---|
| M-TEXT | `TextInput`; Editor/Prelude input handlers | Field label/context; name, path, env, or role text | Text field | Commit validates; Esc cancels/rewinds | Pending value or parent remains | `crates/jackin-console/src/tui/model/modal.rs:L47-L50`; `crates/jackin-console/src/tui/input/editor.rs:L538-L620`; `crates/jackin-console/src/tui/input/prelude.rs:L93-L173` |
| M-FILE | `FileBrowser`; mount/prelude open site | Path/browser title | Selected path row | Path/selection commit; Esc returns parent | Source/workdir/mount continuation | `crates/jackin-console/src/tui/model/modal.rs:L51-L54`; `crates/jackin-console/src/tui/file_browser.rs:L588-L626` |
| M-DST | `MountDstChoice`; mount/prelude | Destination choice | First enabled choice | Same path or edit; Esc rewinds | Destination or TextInput | `crates/jackin-console/src/tui/model/modal.rs:L55-L58`; `crates/jackin-console/src/tui/model/create_prelude.rs:L219-L268` |
| M-WORKDIR | `WorkdirPick`; Editor/Prelude | Working-directory picker | Current/default directory | Valid path required; Esc rewinds | Pending workdir | `crates/jackin-console/src/tui/model/modal.rs:L59-L61`; `crates/jackin-console/src/tui/input/prelude.rs:L93-L173` |
| M-CONFIRM | `Confirm`; row mutations | Action warning | Cancel | Typed destructive choice; Esc is safe cancel | Remove pending row or remain | `crates/jackin-console/src/tui/model/modal.rs:L62-L65`; `crates/jackin-console/src/tui/keymap.rs:L986-L1258` |
| M-DIRTY | `SaveDiscardCancel`; dirty parent exit | `Save changes before leaving?` | Cancel | Save, discard, or remain | Save flow, reload, or parent remains | `crates/jackin-console/src/tui/model/modal.rs:L66-L68`; `crates/jackin-console/src/tui/input/editor.rs:L986-L1104` |
| M-SAVEPREVIEW | `ConfirmSave`; save preview | Effective config diff | Save/confirm control | Diff scroll; Save or Cancel | Async config commit or parent | `crates/jackin-console/src/tui/model/modal.rs:L72-L74`; `crates/jackin-console/src/tui/components/save_preview.rs:L31-L124,L1278-L1291` |
| M-ERROR | `ErrorPopup`; any failed async action | Service/validation error title/detail | Dismiss | No mutation; Esc/Enter dismiss | Parent restored with error acknowledged | `crates/jackin-console/src/tui/model/modal.rs:L75-L77`; `crates/jackin-console/src/tui/view.rs:L423-L488` |
| M-INFO | `ContainerInfo`/`StatusPopup`; host List | Container identity or action result | Dismiss | Read-only; copy where offered | Return to List | `crates/jackin-console/src/tui/model/modal.rs:L78-L83`; `crates/jackin-console/src/tui/view.rs:L423-L488` |
| M-PICK | Role/Op/Source/Scope pickers; Editor/List | Provider, role, source, or scope label | First selectable row | Query/select; Esc cancels | Typed role/provider/source/scope result | `crates/jackin-console/src/tui/model/modal.rs:L84-L106`; `crates/jackin-console/src/tui/input/editor.rs:L620-L968` |
| M-AUTH | `AuthForm`; Editor auth | Auth source/mode form | Mode/source control | Field validation; Esc cancels | Pending auth config or error | `crates/jackin-console/src/tui/model/modal.rs:L107-L112`; `crates/jackin-console/src/tui/input/editor.rs:L620-L968` |
| M-GITHUB | `GithubPicker`; mount/context path | GitHub picker/search | Query or first result | Resolve/select; Esc cancels | URL/source result or error | `crates/jackin-console/src/tui/model/modal.rs:L69-L71`; `crates/jackin-console/src/tui/input/editor.rs:L538-L968` |

The table is a behavior contract, not a claim that every variant appears in
every parent. Parent-specific visibility is recorded in the surface inventory;
dispatch priority and backdrop ownership are in §15.4.

<a id="s16-usage"></a>

### Usage system

#### 16.1 Registry and normalized model

Usage surface order is Claude, Codex, Amp, Grok, Z.AI, Kimi, MiniMax,
OpenCode, Unsupported. Provider, agent runtime, usage surface, account, quota
window, lifecycle, freshness, and confidence are separate axes.

| Surface | Canonical id / label | Account and input roots | Current adapter output |
|---|---|---|---|
| Claude | `claude` / Anthropic | Keychain/config; OAuth identity; API/token env | Session, weekly/all-models, model scopes, spend; OAuth authoritative, CLI estimated |
| Codex | `codex` / OpenAI | `$CODEX_HOME/auth.json`, handoff, API key | 5-hour Session, 7-day Weekly, Spark/credits; RPC then REST; API-key quota unsupported |
| Amp | `amp` / Amp | Amp secrets/handoff/API key | Free daily percentage; credits/workspaces detail |
| Grok | `grok` / xAI | `~/.grok/auth.json`, XAI/deployment key | Billing/weekly-like window, credits, prepaid/on-demand |
| Z.AI | `zai` / Z.AI | API key and endpoint overrides | Token/credit Session or Weekly; MCP time detail |
| Kimi | `kimi` / Kimi | `~/.kimi*`, `KIMI_CODE_API_KEY` | Rate/session and Weekly; missing credential needs secret; presence-only possible |
| MiniMax | `minimax` / MiniMax | API/token and endpoint overrides | General Session/Weekly; detail model windows |
| OpenCode | `opencode` / OpenCode | `opencode-go` auth.json key | Rolling/Weekly/Monthly; rate-limit unavailable; 401 login; 403 unsupported |
| Unsupported | `—` (`None`) / Usage | No provider adapter | Explicit unsupported state |

Registry evidence: `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage.rs:L39-L48,L200-L265,L282-L313`
(`modules`, `UsageSurface`, `ALL`, labels); host IDs/order:
`/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/host.rs:L72-L116,L163-L231`
(`HostSurfaceId`, compact prefixes, URLs, agent labels).

Provider implementation evidence: `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/claude.rs:L16-L68,L101-L262,L630-L760`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/codex.rs:L12-L29,L90-L243,L410-L503,L674-L780`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/amp.rs:L15-L126,L165-L238`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/grok.rs:L12-L37,L39-L125,L253-L318`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/kimi.rs:L16-L86,L88-L159,L221-L283`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/minimax.rs:L16-L74,L76-L198,L239-L302`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/opencode.rs:L4-L9,L55-L78,L99-L205`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/zai.rs:L16-L81,L83-L161,L164-L295`.

#### 16.2 Projection/broker contract

`UsageProjectionSchemaV1` validates percentages to 0–100. Refresh phases and
coordination errors distinguish unavailable, unauthorized, owner lost, timeout,
corrupt state, provider timeout/unavailable, missing secret, protocol mismatch,
and rate limiting. Freshness is Current/Stale/Refreshing/Failed. Quota status is
Available/NotStarted/Warning/Exhausted/Unsupported/Unavailable/Error. Windows
retain category, values, reset, and semantic issue/recoverability metadata.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-protocol/src/usage_broker.rs:L10-L23,L25-L88,L90-L150,L201-L275`
(`protocol v1`, phases/errors/schema/freshness/quota); `/Users/donbeave/junie-style-2/jackin/crates/jackin-protocol/src/usage_broker.rs:L360-L503`
(`projection hierarchy`).

Host normalized view separates `TokenUsageSummary` (per-session token
telemetry) from subscription quota/spend. `AccountUsageSnapshotView` carries
provider/account/source/confidence/window/used/limit/reset/status/error.
`FocusedUsageView` adds focused agent/provider, buckets, status-bar label, tabs,
and error. Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-protocol/src/control.rs:L501-L581,L680-L843,L926-L974`
(`TokenUsageSummary`, account/focused/bucket/status views).

#### 16.3 End-to-end pipeline

```text
host credential/profile discovery
  → provider adapter (API / RPC / CLI / local evidence)
  → account identity + lifecycle catalog
  → bounded broker/coordinator refresh
  → UsageProjectionV1 (last-good + freshness/issue metadata)
  → host Console renderer and/or Capsule scoped relay
  → account/detail/status-bar presentation
```

Host discovery scans config/profile/env/handoff roots; canonical account
identity is surface plus stable provider subject/handle. Presence-only evidence
is not an authenticated account. The Capsule receives scoped opaque capabilities
through a per-container `0600` relay; it must not read canonical host
projection data directly.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/host/discovery.rs:L24-L95,L528-L598`
(`discovery`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/host/accounts.rs:L19-L109,L116-L285`
(`identity/lifecycle/catalog`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/host/projection.rs:L158-L272`
(`projection ordering`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-runtime/src/usage_relay.rs:L212-L300,L347-L393,L440-L594`
(`relay/capability boundary`).

#### 16.4 Host Console Usage reality

Host Usage is current and broker-populated. Console startup opens host usage
discovery, ensures the broker process, reads each capability's current state,
builds the canonical projection, and stages its accounts/notice in manager
state. Opening `u` clones that staged projection; pressing `r` repeats the
adapter load and updates the open screen. Opening alone performs no extra fetch.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/usage.rs:L20-L95,L102-L362`
(`UsageScreenState`, `from_projection`, render/keys); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/list.rs:L49-L62`
(`open usage`); host adapter startup/refresh:
`/Users/donbeave/junie-style-2/jackin/crates/jackin/src/console/adapter/run.rs:L49-L133,L859-L864,L1070-L1087`.

#### 16.5 Capsule Usage reality

Capsule has no separate Usage screen. It presents a bottom Usage chip and a
`Dialog::Usage` overlay. The dialog has Overview/provider tabs, refresh `r`,
provider switching, read-only detail scrolling, and red/yellow/green meter
severity. The bottom chip compacts focused quota/status according to available
space and priority.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog/usage.rs:L8-L180`
(`UsageDialogTab`, state/actions); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/branch_context_bar.rs:L75-L205`
(`usage chip`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog.rs:L146-L180,L353-L440`
(`Usage`/actions); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/daemon/multiplexer_utils.rs:L207-L381,L471-L539`
(`refresh/cache`).

#### 16.6 Ordering/gaps

OpenCode exists in the canonical Usage registry and host IDs, but the inspected
Capsule provider-switch order and desktop provider order exclude it. Record this
as a surface-ordering gap, not as provider absence. Host Console population is
current and broker-backed. No secret value is displayed in account headers.

#### 16.7 Provider/account evidence matrix

| ID/provider | Plan/input/credential origin | Windows, remaining, spend | Refresh/error lifecycle | Status slot | Console/Capsule mapping | Source |
|---|---|---|---|---|---|---|
| U-CLAUDE | Claude config/keychain, OAuth identity, API/token env; OAuth or CLI evidence is classified | Session, weekly/all-models, model scopes, spend/credits where supplied | Token/API/RPC discovery; missing auth, parse, network, and stale states remain typed | Account row plus quota window/status/error | Host registry and Console account; Capsule scoped usage when relay supports it | `crates/jackin-usage/src/usage/claude.rs:L16-L68,L101-L262,L630-L760` |
| U-CODEX | `CODEX_HOME/auth.json`, handoff, or API key; RPC path precedes REST | Five-hour Session, seven-day Weekly, Spark/credits; API-key quota can be unsupported | RPC/REST/auth failures map to unavailable/needs-login/unsupported; last-good projection is freshness-labeled | Provider/account row, window, reset, issue | Host row; Capsule relay projection | `crates/jackin-usage/src/usage/codex.rs:L12-L29,L90-L243,L410-L503,L674-L780` |
| U-AMP | Amp secret/handoff/API-key inputs | Free daily percentage plus credits/workspace detail | Discovery and provider response errors retain lifecycle/freshness | Daily meter and detail status | Host row; Capsule scoped provider view | `crates/jackin-usage/src/usage/amp.rs:L15-L126,L165-L238` |
| U-GROK | `~/.grok/auth.json`, XAI/deployment key | Billing/weekly-like window, credits, prepaid/on-demand | Auth/config/API errors and unsupported account modes stay explicit | Account/billing status and window | Host row; Capsule only through scoped relay | `crates/jackin-usage/src/usage/grok.rs:L12-L37,L39-L125,L253-L318` |
| U-ZAI | API key plus endpoint overrides | Token/credit Session or Weekly; MCP time detail | Endpoint/auth/response failures become typed issue/freshness states | Window meter and error/status | Host row; Capsule relay if capability is present | `crates/jackin-usage/src/usage/zai.rs:L16-L81,L83-L161,L164-L295` |
| U-KIMI | `~/.kimi*` and `KIMI_CODE_API_KEY`; presence-only is not authenticated | Rate/session and Weekly windows | Missing secret, parse, and API errors distinguish needs-secret/unavailable | Account state plus rate/session meter | Host row; Capsule relay mapping | `crates/jackin-usage/src/usage/kimi.rs:L16-L86,L88-L159,L221-L283` |
| U-MINIMAX | API/token and endpoint overrides | General Session/Weekly plus model windows | Auth/endpoint/provider errors retain recoverability metadata | Window/model detail and status | Host row; Capsule relay mapping | `crates/jackin-usage/src/usage/minimax.rs:L16-L74,L76-L198,L239-L302` |
| U-OPENCODE | `opencode-go` auth.json key | Rolling/Weekly/Monthly; rate-limit may be unavailable | 401 means login; 403 unsupported; stale/error remains visible | Provider row and unsupported/error detail | Host registry/host Console; inspected Capsule switch order excludes it | `crates/jackin-usage/src/usage/opencode.rs:L4-L9,L55-L78,L99-L205`; `crates/jackin-usage/src/usage.rs:L200-L265` |
| U-UNSUPPORTED | Registry sentinel; no credential origin or adapter | No quota window | Explicit unsupported, never synthesized as zero | Unsupported status | Host/Capsule may render sentinel only where catalog exposes it | `crates/jackin-usage/src/usage.rs:L282-L313`; `crates/jackin-protocol/src/usage_broker.rs:L201-L275` |

Account identity is separate from provider label: discovery, stable subject or
handle, lifecycle, source, confidence, and last-good projection are retained.
Source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/host/accounts.rs:L19-L109,L116-L285`; projection and relay boundaries: `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/host/projection.rs:L158-L272`, `/Users/donbeave/junie-style-2/jackin/crates/jackin-runtime/src/usage_relay.rs:L212-L300,L347-L393,L440-L594`.

<a id="s17-launch"></a>

### Launch cockpit coverage

#### 17.1 Stage registry

The launch rail is exactly 11 ordered stages:

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

`StageStatus` is Queued, Running, Done, Skipped, Failed, or Blocked. Audited
runtime paths emit the first five states; Blocked is modeled/rendered/tested
but no runtime emission was found. Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/launch_progress.rs:L14-L102`
(`LaunchStage`, `StageView`, `StageStatus`); runtime transitions in
`/Users/donbeave/junie-style-2/jackin/crates/jackin-runtime/src/runtime/launch/launch_pipeline.rs:L677-L712,L949-L1038,L1165-L1250,L1267-L1437`
and `crates/jackin-runtime/src/runtime/launch/launch_pipeline/launch_core/orchestrate.rs:L810-L956,L1437-L1527,L1790-L1819`.

#### 17.2 Operator-visible cockpit

Initial view shows all stages queued and `preparing launch`. Identity then
drives the header to `Loading <role> in <target>`; the progress rail advances by
done/skipped stages, pulses the running stage, colors failure red, and animates
the active label. Footer activity is formatted with an uppercased first word
and trailing ellipsis. Container chip appears after identity has a container;
debug run ID appears only in debug.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/update.rs:L16-L239`
(`initial_view`, update/frontier); `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/header.rs:L15-L118`
(`header`); `crates/jackin-launch/src/tui/components/progress_rail.rs:L15-L245` (`rail`);
`crates/jackin-launch/src/tui/components/footer.rs:L60-L181`
(`activity/footer`).

#### 17.3 Build-log overlay

Build log is a full opaque overlay. It is opened by clicking footer activity
when lines exist and closed with Esc. It tail-follows; arrows/j/k/PageUp/PageDown,
wheel, and scrollbar drag scroll. Long/ANSI lines wrap with `↳`; title is
`Docker build · building…` while active and `Docker build` otherwise. Buffer
capacity is 5,000 lines and old lines drop.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/build_log_dialog.rs:L46-L97,L180-L300`
(`build log render`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-diagnostics/src/build_log.rs:L16-L69`
(`bounded buffer`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/subscriptions.rs:L239-L293,L810-L835`
(`mouse/open/scroll`).

#### 17.4 Failure and container-info overlays

Failure popup shows summary, failed stage, run ID, optional backend query, and
next step. Full detail remains diagnostics-only. Only run ID is copyable;
reveal/open payload is absent in the model. Enter/Esc acknowledges; outside
click acknowledges; in-popup non-copy click is swallowed; body scrolls.

Container info shows version, container, role, agent, target, run ID and, in
debug, telemetry/invocation details. Enter copies the available copy target;
Esc/outside dismisses. Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/failure_dialog.rs:L25-L71,L250-L310`
(`failure`); `crates/jackin-launch/src/tui/components/container_info_dialog.rs:L21-L120` (`info`);
`crates/jackin-launch/src/tui/subscriptions.rs:L347-L404,L778-L808`
(`routing`).

#### 17.5 Quit/cancel/handoff

Ctrl-C is immediate hard abort with terminal restoration and no cleanup wait.
Ctrl-Q opens `Exit jackin❯?`; Yes maps to hard exit, No/Esc resumes. Pipeline
cancellation returns typed `LaunchCancelled`. On Capsule ready, the rich
cockpit stops before interactive `docker exec -it` so Capsule owns the PTY.
Noninteractive launch marks Hardline detached and returns. Success/failure
both pass through exit rendering; last-instance exit plays the Construct outro.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/subscriptions.rs:L42-L63,L557-L639`
(`hard/quit`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/run.rs:L220-L235,L296-L328`
(`cancel`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-runtime/src/runtime/launch/launch_runtime.rs:L1150-L1274`
(`hardline handoff`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-runtime/src/runtime/launch/launch_pipeline.rs:L1493-L1520`
(`failure/exit`).

<a id="s18-capsule"></a>

### Capsule / in-construct experience

#### 18.1 Frame and ownership

Capsule is PID 1 in the container and owns the PTY/session/multiplexer/control
plane. The host has one active attach client; Capsule can retain multiple
PTY-backed sessions. Its frame is top status bar, optional branch/context bar,
pane tree, scrollbars/selection/cursor, notices/toasts/tooltips, and bottom
chrome. Dialogs/backdrops sit above pane rendering.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/daemon.rs:L1-L27,L258-L274`
(`daemon ownership`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/view.rs:L173-L324`
(`frame`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/layout.rs:L109-L120`
(`reserved bottom rows`).

#### 18.2 Tabs, panes, zoom, split

Pane tree leaves, horizontal splits, and vertical splits are rendered. Split
ratios clamp to `.05–.95`, default `.5`; focused Alt-Shift arrows change the
focused split by `.05`. Zoom replaces the visible tree with one full pane.
Automatic labels are Shell, one agent label, Agents, or Mix, with count where
needed. Tabs overflow with `›`; active tab has an underline and status glyphs.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/layout.rs:L27-L40,L145-L184,L327-L423,L619-L680`
(`tree/split/zoom`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/model.rs:L231-L335`
(`visible panes/labels`); `crates/jackin-capsule/src/tui/components/status_bar.rs:L4-L67,L175-L341`
(`tab chrome`).

#### 18.3 Capsule action matrix

| Action | Key route | Result |
|---|---|---|
| New tab | Prefix `c` / palette | Spawn session/tab; selected tab changes |
| Next/previous tab | Prefix `n`/`p` | Active tab moves |
| Jump tab | Prefix `0–9` | Select indexed tab if present |
| Move focus | Prefix `h/j/k/l` | Nearest pane in direction |
| Split | Prefix `"` / `%` | Top-bottom / side-by-side PTY pane |
| Resize | Alt-Shift arrows | Adjust focused split ratio |
| Zoom | Prefix `z` / palette | Zoom/unzoom active pane |
| Close pane/tab | Prefix `x` / `&` / palette | Close target; confirm when policy requires |
| Clear | Prefix Ctrl-L / palette | Clear focused pane |
| Detach | Prefix `d` | Leave client; daemon/PTYs persist |
| Usage | Prefix `u` / palette/chip | Open Usage dialog |
| Palette | Prefix space/colon or configured palette key | Open command palette |
| Redraw | Prefix `r` | Request redraw |
| Rename | Palette | Set custom label, max 16 chars |
| Export/link/host path | Palette/dialog/click | Copy, open, reveal subject to safety gates |
| Exit | Ctrl-Q / palette | Dirty/exit confirmation, then shutdown path |

Static key source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/keymap.rs:L22-L262,L402-L467`
(`global`, prefix, resize); palette source:
`/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/palette.rs:L29-L148`
(`PaletteCommand`, visible labels).

#### 18.4 Input parser and terminal fidelity

The parser accepts data, key/paste, mouse press/release, prefix commands,
palette open, exit, resize, focus in/out. It handles incomplete Escape with a
short hold, CSI/kitty/CSI-u controls, SGR/X10 mouse, bracketed paste, modified
arrows, focus events, and terminal reports. Dialog/prefix/drag/select modes
block pane forwarding according to priority; ordinary agent bytes pass through.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/input.rs:L197-L335,L350-L590,L643-L867`
(`InputEvent`, parser, CSI/mouse); `/Users/donbeave/junie-style-2/jackin/docs/content/reference/capsule/multiplexer-design-rules.mdx:L5-L180`
(`terminal fidelity contract`).

#### 18.5 Status and context chrome

Status bar is two rows. Tab glyph priority is Blocked `●`, Done `○`, Working
`▶`, Idle `◆`, Unknown blank. The branch/context bar hides the default branch;
left content is PR number/title, resolving branch, or branch. Right slots carry
usage, container, and debug run ID, with compact-slot priorities. Menu is
`☰Menu`; prefix awaiting is `prefix…`.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/status_bar.rs:L50-L67,L203-L340`
(`status bar`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/branch_context_bar.rs:L52-L205`
(`branch/context/usage`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/model.rs:L193-L229`
(`VisibleAgentState`, labels).

#### 18.6 Cursor, selection, scrollback

Hardware cursor appears only when there is no dialog, a pane/session exists,
recent output was received, view is live rather than scrollback, and the agent
has not hidden its cursor. Scrollback is tail-relative; typing snaps to live.
Selection supports drag, autoscroll, word double-click, and OSC52 copy.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/model.rs:L168-L191`
(`cursor_visible`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/session.rs:L646-L762`
(`scrollback`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/daemon/mouse_input.rs:L329-L529`
(`selection`).

#### 18.7 Dialog catalog

| Dialog | Purpose | Result |
|---|---|---|
| CommandPalette | Search/execute Capsule actions | Command action/dismiss |
| AgentPicker / ProviderPicker | Spawn agent/provider | Spawn or cancel |
| RenameTab | Custom label | Rename/cancel |
| ExportFile | Save/reveal/open output | Export/host action |
| ContainerInfo | Read-only runtime/debug facts | Copy/dismiss |
| GitHubContext | Branch/PR/CI URL facts | Copy/open/dismiss |
| Usage | Overview/provider quota | Refresh/provider switch/close |
| SpawnFailure | Explain spawn failure | Dismiss |
| SplitDirectionPicker | Choose split direction | Split/dismiss |
| CloseTargetPicker | Pick pane/tab | Target/confirm |
| ConfirmAction | Close pane/tab/exit | Confirm/cancel |
| ExecPicker | Host/exec operation | Confirm/cancel |
| ExitDirty | Start, inspect, keep, discard | Exit action |
| ExitInspect | Repo/file inspect choice | Inspect/dismiss |

Source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog.rs:L146-L351,L353-L440`
(`Dialog`, `ExitDirtyRow`, `ConfirmKind`, `DialogAction`).

#### 18.8 Detach, exit, dirty state, takeover

Detach leaves the client while preserving daemon, PTYs, sessions, and panes.
Exit requests confirmation; dirty exit offers StartNewAgent, Inspect, Keep, or
Discard. A new attach takes over the single client; the old client receives a
shutdown/takeover while daemon state remains. Normal last-live-session exit
drains and shuts down. Hardline/eject/exile are host/runtime lifecycle terms,
not Capsule TUI commands.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog.rs:L289-L345`
(`exit/confirm`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/daemon.rs:L788-L823,L1335-L1494`
(`shutdown/takeover`); `/Users/donbeave/junie-style-2/jackin/docs/content/reference/capsule/session-lifecycle.mdx:L30-L76`
(`lifecycle contract`).

<a id="s19-interaction"></a>

## Shared Interaction Reference

### 19.1 Host console routing

Host crossterm normalization produces key, mouse, resize, paste, and tick
events. Mouse routing checks modal/copy/file-browser/picker regions, then tabs,
focus/scrollbar drag, wheel, row selection, URL, and list seam. A click focuses
the hit; a matching release activates it. Modal outside-click rules are
surface-specific; modal-open regions do not pass through to background.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/mouse.rs:L90-L307`
(`handle_mouse_with_config`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/mouse/hover.rs:L27-L34,L130-L230`
(`hover`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/mouse/scroll_pan.rs:L15-L123`
(`scroll focus`).

### 19.2 Modal/focus barriers

The console’s render/input plans expose one visible modal owner at a time.
Editor/Settings retain parent modal state for nested flows, but children own
the interaction barrier while visible. Workspace footer remains outside the
modal backdrop. Keyboard help is a top-priority overlay and cannot coexist with
another modal.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/stage.rs:L46-L62,L199-L243`
(`dispatch priority`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L634-L747`
(`backdrop/modal/help`); generic modal state `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/modal.rs:L22-L113`.

### 19.3 Keyboard semantics

Host Editor/Settings use tab bar Left/Right/Tab/BackTab, content Up/Down and
`j/k`, contextual horizontal `h/l`, Enter actions, S save, Esc/back. Workspace
List uses tree Left/Right, horizontal h/l, action letters, Tab preview. Launch
and Capsule use their own static maps. Key hints are generated from the same
registries where the source supports registration; unregistrable mouse/dynamic
keys are documented as exceptions.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/keymap.rs:L59-L252,L254-L889,L986-L1359`
(`editor/settings/list/global maps`); `/Users/donbeave/junie-style-2/jackin/docs/content/reference/tui/navigation.mdx:L222-L222`
(`hint registry rule`).

### 19.4 Scroll semantics

Console scroll ownership is local: list, detail, editor mounts/content, Settings
mounts/content, modal body, file browser, and preview each retain offsets and
clamp against current rectangles. Capsule scrollback is tail-relative; dialogs
have their own vertical/horizontal scroll. Wheel changes scroll, not focus, in
the host’s generic contract.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/update.rs:L835-L886,L1120-L1245`
(`list/preview scroll`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/frame.rs:L321-L498,L668-L710`
(`editor geometry`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/scroll_input.rs:L18-L62`
(`Capsule dialog scroll`).

### 19.5 Hover, cursor, pressed, focus

Host render rebuilds hit/focus regions every frame, validates focus after
render, and exposes hover/pressed state to components. Hardware cursor is set
by the last eligible cursor owner in the frame. Capsule suppresses hover while
dragging/selecting and computes pointer shape in priority order: grabbing,
text-selecting, dialog targets/default, split resize, link, selectable text,
default.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L492-L539`
(`prepare_for_render`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/model.rs:L63-L166`
(`PointerShapeState`, `HoverTarget`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/model.rs:L168-L191`
(`cursor`).

### 19.6 Async ownership

The host console remains nonblocking: service effects start/poll outside render,
then typed result messages update state. Launch races progress against
cancellation. Capsule daemon/control owns PTY/session operations and caches
usage/status. Render reads state; provider calls and filesystem/runtime work do
not happen inside view functions.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/effect.rs:L10-L66`
(`ManagerEffect`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/state/update.rs:L91-L145,L180-L300`
(`update_manager`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/daemon/multiplexer_utils.rs:L207-L381`
(`daemon refresh/cache`).

<a id="s20-visual"></a>

## Current Visual Language

### 20.1 Host console frame grammar

Host console uses a brand header, route-specific body, contextual footer, and
modal backdrop that excludes the footer. Workspace is a two-pane tree/detail
composition. Editor and Settings use a three-row header, two-row tab strip,
body, and dynamic footer. Dialogs use bounded modal rectangles that shrink for
narrow terminals. Selection, disclosure, status, masking, and compact labels
carry meaning; exact placement is current implementation.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L365-L410,L634-L747`
(`header/footer/backdrop/render`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/view.rs:L101-L212`
(`Settings frame`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/frame.rs:L32-L118`
(`Editor frame`).

### 20.2 Brand tokens

| Token | Value | Meaning in current UI |
|---|---|---|
| `PHOSPHOR_GREEN` | `0,255,65` | primary Jackin brand/highlight |
| `PHOSPHOR_GREEN_DIM` | `0,140,30` | dim brand text/secondary rail |
| `PHOSPHOR_GREEN_DARK` | `0,80,18` | dark brand field |
| `BLACK` / `WHITE` | `0,0,0` / `255,255,255` | canvas / high-contrast text |
| Rain colors | head/fresh/body/mid/dim/dark ladder | launch/brand motion and depth |
| `LINK_BLUE` | `0,80,180` | link/action affordance |
| `DEBUG_AMBER` | `204,92,0` | debug/run identity |
| `STATUS_BLOCKED_RED` | `255,60,60` | blocked/danger state |
| `MENU_IDLE_BG` / hover | `18,70,130` / `32,92,158` | menu/status chrome |
| Awaiting/hover | `96,180,255` / `132,202,255` | prefix/focus emphasis |
| `CYAN` / dim | `0,180,180` / `0,120,120` | instance/live secondary status |
| `ACTION_ACCENT` | `180,255,180` | action emphasis |
| `DISCLOSURE_ACCENT` | `255,208,102` | tree disclosure |

Source: `/Users/donbeave/junie-style-2/jackin/crates/jackin-brand/src/lib.rs:L34-L79`
(`brand tokens`); adaptation: `/Users/donbeave/junie-style-2/jackin/crates/jackin-tui/src/tokens.rs:L22-L59`
(`shared tokens`).

### 20.3 Host and launch semantic grammar

Current host presentation uses `▸` for selected rows, `▶/▼` for disclosure,
checkboxes for role allowance, `★` for default role, `[op]` for on-demand
secret references, compact status labels for instance state, and meters for
quota severity. Launch uses a progress rail, pulsing active stage, frozen
failure view, opaque build log, and copyable identity/debug chips.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L52-L95,L615-L742,L1148-L1272`
(`selection/disclosure/roles`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/progress_rail.rs:L101-L148,L216-L245`
(`stage status`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/footer.rs:L85-L181`
(`chips`).

### 20.4 Capsule semantic grammar

Capsule puts terminal output first, with status/context chrome around it. Tab
glyphs map to public agent states; branch/PR context sits apart from quota
status; usage/container/debug chips are ranked slots. Dialogs are bounded,
opaque/modal; pointer shape and cursor visibility reflect current interaction
mode and output state.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/status_bar.rs:L282-L340`
(`TabGlyph`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/branch_context_bar.rs:L75-L205,L345-L395`
(`context/slot priority`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/view.rs:L173-L324`
(`compositor`).

### 20.5 Motion and feedback

Launch render ticks at roughly 33 ms for stage label/rain animation, with
`JACKIN_NO_MOTION` disabling frame advancement. Console and launch use
short-lived activation/status updates; Capsule uses daemon/session events and
terminal output rather than decorative blocking progress. Long-running work is
always represented by state/activity or a dedicated log/detail surface.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/run.rs:L80-L188,L412-L472`
(`tick/motion`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/animation.rs:L305-L330`
(`intro/outro`); `/Users/donbeave/junie-style-2/jackin/crates/jackin/src/console/adapter/run.rs:L1063-L1222`
(`console event/redraw loop`).

### 20.6 Product meaning carried by current presentation

- Workspace/instance tone separates durable configuration from live lifecycle.
- Left/right split separates identity selection from inspectable detail.
- Tab strips separate domains without merging Editor and Settings scope.
- Masks and `[op]` mark secret handling without exposing values.
- Progress rail expresses ordered launch dependency and current frontier.
- Failure/build overlays preserve operator diagnosis without losing run identity.
- Capsule status glyphs make agent attention state scannable while keeping PTY
  output primary.
- Usage bars distinguish account/quota health from agent activity.

These meanings are backed by the render/data sources cited in §§11–18.

### 20.7 Current presentation choices that are not future design invariants

The following are implementation choices, not promises to copy into a future
surface: exact split percentages; exact row glyphs; current border/rain styling;
footer row counts; modal widths; tab underline shape; provider chip order;
animation timing; compact truncation characters; and whether a detail pane is
left or right. Preserve the semantics and transitions in [§26](#s26-redesign-contract),
not these coordinates or decorations.

<a id="s21-workflows"></a>

## Complete Workflow Catalog

“Current” means executable path at the pinned SHA. “Partial” means a visible
surface or one branch exists but the end-to-end producer/integration is absent.
“Planned” and “research-only” do not appear as current controls.

| Workflow | Classification | Current path and result |
|---|---|---|
| Bare interactive startup | Current | `jackin` on an interactive terminal enters the host console; terminal guard owns alt screen/raw mode and teardown. |
| Console startup error | Current | Docker/handler setup failure becomes startup error popup; no stage interaction until dismissed/returned. |
| Select current directory | Current | List row Enter launches current directory with default/resolved context. |
| Select saved workspace | Current | Enter launches named workspace; E opens Editor; D opens delete confirm; S opens Settings. |
| Expand/collapse workspace | Current | Left/Right or h/l changes tree disclosure; children become selectable visual rows. |
| Create workspace | Current | New-workspace sentinel → five-step CreatePrelude → Editor with pending workspace. |
| Edit workspace | Current | Editor snapshots original/pending config; tabs mutate pending only. |
| Save workspace | Current | Planner/validation → ConfirmSave preview → async config write → List or error. |
| Leave dirty Editor | Current | Save changes before leaving? → Save/Discard/Cancel. |
| Delete workspace | Current | ConfirmDelete → RemoveWorkspace effect → List/config reload. |
| Launch named/current | Current | Manager outcome → launch pipeline → cockpit → Capsule hardline or failure. |
| Provider/agent selection | Current | Inline launch/new-session pickers select compatible provider/agent; result leaves host List for launch. |
| Role load/trust | Current | Role picker/load resolves source; trust confirmation/persistence may precede registration. |
| Prewarm | Current | List W/prewarm effect prepares named runtime/image resources; status returns to List. |
| Reconnect live instance | Current | R/Enter on instance uses restore/hardline attach ladder; unavailable state opens error. |
| New session in instance | Current | A/N opens agent/provider selection, then host launches a new Capsule session. |
| Open shell | Current | X hands off shell action to host/container handler; remains an instance action, not a List-only render. |
| Inspect instance | Current | I opens/returns diagnostic/instance-info path; exact host presentation depends on handler outcome. |
| Stop instance | Current | T sends in-place stop action and status popup; List refresh follows. |
| Purge instance | Current | P → ConfirmInstancePurge → purge action; purged/superseded rows disappear from visible tree. |
| Settings General | Current | Toggle coauthor/DCO; save/discard global config. |
| Settings mounts | Current | Add source → destination → scope/role → preview/save; remove/readonly/rename/GitHub branches. |
| Settings environments | Current | Add/edit/delete/mask; choose scope/source/op/role; save global config. |
| Settings auth | Current | Choose auth kind → source/folder/op/auth form → save/reset; token mint can suspend terminal and remount form. |
| Settings trust | Current | Select role source, toggle trust, save; errors stay visible. |
| Usage host view | Current | Startup loads broker current state into the canonical projection; `u` opens it and `r` reloads accounts/notice. |
| Usage Capsule view | Current | Prefix u/palette/chip → Overview/provider dialog; r refreshes; Esc closes. |
| Add mount in Editor | Current | FileBrowser → destination choice/edit → pending `MountConfig`; validation before save. |
| Add environment in Editor | Current | Scope/source/op/text → pending env row; mask/delete/role expansion. |
| Configure auth in Editor | Current | Auth kind/source/folder/form → pending workspace/role auth override. |
| GitHub source/context | Current | GitHub picker/URL resolution or Capsule GitHub context dialog; open/copy branches are gated. |
| 1Password source | Current | `op` availability is probed; picker can select/create source where wired; unavailable becomes explicit error. |
| Save/discard child modal | Current | Modal outcomes restore parent or commit pending result. |
| Launch progress | Current | Ordered 11-stage rail with async updates and activity/footer. |
| Docker build diagnostics | Current | Footer activity opens bounded opaque build log; close, vertical scroll, page scroll, wheel, and scrollbar drag are supported. Copy belongs to failure/container-info overlays, not build log. |
| Launch failure | Current | Failure popup shows safe summary/stage/run ID/next step; acknowledgement returns error path. |
| Launch cancellation | Current | Typed cancellation tears down progress and returns through exit rendering. |
| Launch quit | Current | Ctrl-Q confirmation; Ctrl-C hard abort; terminal restoration contract. |
| Capsule attach | Current | Capsule owns PTY after hardline; host cockpit stops before exec attach. |
| Capsule new tab/split | Current | Prefix/palette spawns PTY-backed pane/tab, updates tree/focus. |
| Capsule focus/resize/zoom | Current | Prefix h/j/k/l, Alt-Shift arrows, z update focus/tree/ratios/visible PTY sizes. |
| Capsule close | Current | Close pane/tab target picker and confirm action as needed; session/tree cleanup. |
| Capsule scrollback/select/copy | Current | Scrollback state, mouse selection, autoscroll, OSC52 clipboard. |
| Capsule dirty exit | Current | ExitDirty offers start agent/inspect/keep/discard; daemon drains after result. |
| Capsule detach | Current | Prefix d exits attached client while daemon/session state persists. |
| Capsule takeover reconnect | Current | New attach shuts down old client with takeover; daemon state remains. |
| Capsule exec/host URL | Current with safety gates | Exec is approval/credential gated; links validate scheme and use host action. |
| Apple Container full parity | Planned/partial | Backend selection exists; finalize/hardware parity remains open. |
| Capsule host event stream/Desktop bridge | Planned | Roadmap Phase 4; no current TUI control. |
| Host daemon live reconciliation | Planned/partial | Lifecycle foundation exists; broader cross-container event aggregation remains open. |

Workflow evidence: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/list.rs:L45-L325`
(`List actions`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/dispatch.rs:L346-L462`
(`stage/effect transitions`); `/Users/donbeave/junie-style-2/jackin/crates/jackin/src/app/load_cmd.rs:L403-L486`
(`console launch outcomes`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/daemon/pane_layout.rs:L12-L195,L339-L421`
(`pane/split/focus effects`). Planned boundaries: `/Users/donbeave/junie-style-2/jackin/docs/content/roadmap/in-progress.mdx:L10-L49`
(`roadmap index`); Capsule roadmap at
`docs/content/roadmap/(reactive-daemon-program)/jackin-capsule.mdx:L7-L47`.

### 21.1 Workflow step sheets

| ID | Prerequisite/input | Ordered steps | Alternate/error/recovery | Persistent effect | Evidence |
|---|---|---|---|---|---|
| WF-START | Interactive TTY and load command | Guard terminal → load config/roles → initialize manager → render List | Guard/setup error returns typed startup error; no false List | Terminal ownership/restore state | `crates/jackin/src/console/adapter/run.rs:L50-L133`; `crates/jackin/src/cli.rs:L101-L168` |
| WF-LIST-LAUNCH | Current or saved workspace row | Select → resolve config/role/auth → launch pipeline → Capsule or handler | Missing config/auth/role opens error; failure returns failed stage | Instance/manifest/launch identity | `crates/jackin-console/src/tui/input/list.rs:L45-L197`; `crates/jackin/src/app/load_cmd.rs:L403-L486` |
| WF-CREATE | `+ New workspace` sentinel | FileBrowser → destination choice → optional edit → workdir → name → Editor | Esc rewinds by step; browser/path validation stays open | Pending WorkspaceConfig with first mount | `crates/jackin-console/src/tui/model/create_prelude.rs:L11-L33,L219-L378`; `crates/jackin-console/src/tui/input/prelude.rs:L3-L173` |
| WF-EDIT-SAVE | Saved workspace or prelude handoff | Snapshot original/pending → mutate tabs → save preview → validate/write | Validation/error keeps Editor; Cancel leaves pending intact | Workspace config changes only after commit | `crates/jackin-console/src/tui/input/save.rs:L123-L260`; `crates/jackin-console/src/tui/components/save_preview.rs:L31-L124,L1278-L1291` |
| WF-DIRTY-EXIT | Dirty Editor/Settings | Esc → SaveDiscardCancel → save, discard, or cancel | Cancel remains; save error restores parent modal | Pending changes either committed or dropped | `crates/jackin-console/src/tui/model/modal.rs:L76-L80`; `crates/jackin-console/src/tui/input/editor.rs:L986-L1104` |
| WF-DELETE-PURGE | Workspace/instance target | Confirm typed target → effect → reload visible rows | Esc/cancel is safe; unavailable target becomes error/status | Config deletion or instance purge/supersede | `crates/jackin-console/src/tui/model/stage.rs:L11-L36`; `crates/jackin-console/src/tui/input/list.rs:L198-L325` |
| WF-RECONNECT | Instance row and restore identity | Select instance → restore/attach ladder → Capsule or error | Missing daemon/session opens unavailable/error; retry remains outside view | Attach ownership may move to new client | `crates/jackin/src/app/restore.rs:L112-L180`; `crates/jackin-console/src/tui/screens/workspaces/update.rs:L337-L520` |
| WF-NEWSESSION | Live instance with compatible target | Select A/N → choose agent/provider/role → launch session → attach | Picker cancel returns List; spawn/launch failure is typed | New session/tab/pane and manifest update | `crates/jackin-console/src/tui/screens/workspaces/update.rs:L35-L121,L337-L520`; `crates/jackin-capsule/src/tui/input.rs:L197-L335` |
| WF-INSTANCE-ACTION | Instance row | X shell or I inspect or T stop or P purge → status/handler | Invalid lifecycle/status opens status/error; purge confirms | Shell/inspection/stop/purge effect | `crates/jackin-console/src/tui/screens/workspaces/update.rs:L25-L33,L147-L267`; `crates/jackin-console/src/tui/view.rs:L423-L488` |
| WF-SETTINGS | List and global config | Open tab → mutate global state → child modal flows → save/discard | Child cancel restores Settings; save error stays visible | Global config policy | `crates/jackin-console/src/tui/screens/settings/model.rs:L48-L107,L617-L697`; `crates/jackin-console/src/tui/screens/settings/view.rs:L127-L147` |
| WF-MOUNT | Add/edit mount target | Source browser → destination → scope/role → readonly/isolation → pending/save | Invalid path/scope/cleanup error stays in child; cancel drops branch | MountConfig after save | `crates/jackin-console/src/tui/screens/editor/view/mounts_tab.rs:L65-L154`; `crates/jackin-config/src/schema.rs:L73-L88,L592-L654` |
| WF-ENV | Env row target | Add/edit → scope/source/op/text → mask/expand/delete → save | Reserved/invalid values reject; missing op remains explicit | Env value/scope in pending/global config | `crates/jackin-console/src/tui/screens/editor/view/secrets_tab.rs:L23-L64,L93-L124,L191-L238`; `crates/jackin-core/src/env_model.rs:L76-L199` |
| WF-AUTH-TRUST | Auth or role-source target | Choose kind/source/folder/op → form/trust → validate → save | Missing credential/provider error; cancel preserves parent | Auth or trust policy after commit | `crates/jackin-console/src/tui/screens/settings/model.rs:L523-L697`; `crates/jackin-config/src/schema.rs:L370-L456` |
| WF-USAGE-HOST | List with projected account state | Open `u` → select Overview/account → `r` refresh if needed → scroll detail → Esc | Empty/unresolved/stale/provider error remains labeled; refresh failure updates notice | None; read-only view | `crates/jackin/src/console/adapter/run.rs:L49-L133,L859-L864,L1070-L1087`; `crates/jackin-console/src/tui/screens/usage.rs:L45-L95,L139-L362`; `crates/jackin-console/src/tui/view.rs:L751-L767` |
| WF-USAGE-CAPSULE | Capsule attach and relay capability | Open chip/dialog → Overview/provider → `r` refresh → inspect → Esc | Unsupported/stale/missing-secret/error is visible; last-good remains labeled | Relay refresh/cache only | `crates/jackin-capsule/src/tui/components/dialog/usage.rs:L8-L180`; `crates/jackin-runtime/src/usage_relay.rs:L212-L300,L347-L393` |
| WF-LAUNCH-FAIL | Pipeline failure at any stage | Rail update → freeze failed stage → failure dialog → copy/ack/exit | Long detail scrolls; retry/handler outcome stays external | Cleanup/failure result, no false success | `crates/jackin-core/src/launch_progress.rs:L198-L313`; `crates/jackin-launch/src/tui/components/failure_dialog.rs:L25-L71,L250-L310` |
| WF-LAUNCH-CANCEL | Active launch | Ctrl-C hard abort or Ctrl-Q quit confirm → teardown → restore terminal | Cancel/quit are distinct; terminal restoration is mandatory | No partial handoff presented as success | `crates/jackin-launch/src/tui/run.rs:L80-L188,L338-L375,L1192-L1220`; `crates/jackin-launch/src/tui/subscriptions.rs:L45-L85,L557-L610` |
| WF-CAPSULE-PANE | Active Capsule | Prefix/palette → new tab/split/close → choose target → update layout/focus → PTY resize | Spawn/close/layout error returns dialog/status; dirty close confirms | Daemon pane/tab/session state | `crates/jackin-capsule/src/tui/keymap.rs:L22-L467`; `crates/jackin-capsule/src/tui/daemon/pane_layout.rs:L12-L195,L339-L421` |
| WF-CAPSULE-EXIT | Active/dirty Capsule | Exit/detach → dirty choice if needed → drain/close or detach → host restore | Inspect/start/keep/discard branches are explicit; takeover shuts old client | Durable daemon session or clean close | `crates/jackin-capsule/src/tui/input.rs:L643-L867`; `crates/jackin-capsule/src/daemon.rs:L788-L823,L1335-L1494` |

<a id="s22-data-map"></a>

## Data-Presentation Map

| Source data | Host presentation | Capsule/launch presentation | Omitted/safety boundary |
|---|---|---|---|
| Workspace name/workdir | List row; General detail; Editor General | Launch target/header | No secret values implied by name/path |
| Mount source/destination/mode/isolation | Mount rows, save diff, settings rows | Launch identity/mount facts where exposed | Sensitive path confirmation; auth files separate |
| Allowed/default/last role | Roles rows, `★`, summary | Launch role/header; Capsule label | Untrusted role is not silently accepted |
| Env key/value/source/scope | Masked editor/settings rows, `[op]`, scope | Injected runtime env | Secret value hidden in UI/diagnostic summary |
| Auth mode/source/folder | Auth rows/forms | Credential resolution stage | Token/API secret not rendered |
| Agent runtime/provider | Picker rows; instance summary | Launch identity; Capsule pane/tab label | Provider ≠ agent ≠ usage surface |
| Instance status/id/container | Tree row/status, preview, info | Launch chip; Capsule container chip | Purged/superseded hidden from visible tree |
| Session record | Instance session table | Live PTY tab/pane/session | Persisted manifest and live daemon data are distinct |
| Launch stage/status/progress | — | Rail/activity/build/failure | Full failure detail diagnostic-only |
| Usage provider/account | Host Usage rows/detail | Capsule chip/dialog | Capability is opaque; no canonical projection direct read |
| Quota window/value/reset/severity | Meter/detail | Meter/chip/detail | Unsupported/unavailable states remain explicit |
| Git branch/PR/CI | GitHub picker/action | Branch/context bar/dialog | URL opening validates/gates host action |
| Invocation/run/container IDs | Debug/footer/container info | Debug chip/container info | Only allowed copy targets exposed |
| PTY bytes/output/cursor | Host hands off | Capsule pane buffer, selection, cursor | Modal/prefix/scrollback mode controls forwarding |

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs:L260-L328,L544-L584`
(`workspace/effective view`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/instance.rs:L14-L133`
(`instance/session`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-protocol/src/control.rs:L519-L581,L680-L843`
(`usage/control views`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-runtime/src/usage_relay.rs:L212-L300,L347-L393,L440-L594`
(`usage relay boundary`).

### 22.1 Data-map evidence

| ID | Source type/subsystem | Formatting/fallback | Rendered in | Evidence |
|---|---|---|---|---|
| D-WORKSPACE | Persisted/resolved config | Name/path are clipped or scrolled; missing optional fields use defaults | List, preview, Editor, Launch identity | `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs:L260-L328,L342-L368` |
| D-MOUNT | Config mount model and validation | Source/destination/readonly/isolation stay separate; invalid paths reject | Editor/Settings rows, save preview, launch facts | `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs:L73-L88,L592-L654` |
| D-ENV | Env model/value source | Values mask; `[op]` denotes external source; reserved/invalid keys reject | Editor/Settings Environments | `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/env_value.rs:L11-L157`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/env_model.rs:L76-L199` |
| D-AUTH | Auth config/role override | Mode/source/folder render; token/API material omitted | Editor/Settings Auth, launch Credentials | `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs:L110-L138,L370-L456`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/auth.rs:L12-L36` |
| D-ROLE | Role registry/trust/effective override | Unknown/untrusted role is labeled, not silently accepted | List preview, Editor/Settings Roles/Trust, launch Role | `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs:L110-L138,L442-L456`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L1148-L1272` |
| D-INSTANCE | Runtime instance index | Lifecycle labels map enum to short/status text; purged/superseded hidden | List tree/preview, Launch identity, info | `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/instance.rs:L14-L133`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L1284-L1398` |
| D-SESSION | Persisted manifest plus live daemon snapshot | Manifest read error is explicit; live tabs are not invented from records | Host preview, Capsule tabs/panes | `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/instance.rs:L78-L133`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L1284-L1398` |
| D-LAUNCH | Pipeline stage/status/identity | Ordered rail; failed/blocked/unavailable remain typed | Launch rail/activity/build/failure/info | `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/launch_progress.rs:L14-L177,L198-L313` |
| D-USAGE | Provider adapter → broker/projection | Last-good plus freshness/issue; unsupported is not zero | Host Usage, Capsule chip/dialog | `/Users/donbeave/junie-style-2/jackin/crates/jackin-protocol/src/usage_broker.rs:L10-L23,L90-L150,L201-L275`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/host/projection.rs:L158-L272` |
| D-GIT | Git/GitHub context | URL/open action is validated/gated; unresolved context stays labeled | Prelude picker, Editor mount, Capsule context | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/modal.rs:L82-L113`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/branch_context_bar.rs:L52-L205,L345-L395` |
| D-PTY | Daemon PTY bytes/cursor/layout | Modal/prefix/scrollback modes gate forwarding; resize recalculates PTY sizes | Capsule pane terminal | `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/input.rs:L197-L335,L643-L867`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/daemon/pane_layout.rs:L177-L195` |

<a id="s23-copy"></a>

## Operator-Visible Copy and Terminology

### 23.1 Host console copy

| Surface | Current copy |
|---|---|
| Workspace rows | `Current directory`; `+ New workspace`; instance `{id}  {role}` plus `[status]` where applicable |
| New workspace detail | `Create a workspace from this directory.` plus current workdir/mount guidance |
| Empty sessions | `No sessions recorded`; `Sessions unavailable (manifest read error)` |
| Empty live tabs | `Daemon reports no tabs` |
| Running panel | `Running`; `{count} instance(s) running`; `· ↓ navigate instances` or `· → expand` |
| Workspace detail | `Working dir {workdir}`; General/Mounts/Environments/Roles |
| Editor General | `Name`; `Working dir`; `Keep awake`; `enabled (macOS only)`; `Git pull` |
| Editor sentinels | `+ Add mount`; `+ Load role`; `+ Add environment variable`; `+ Add {role} environment variable` |
| Role warning | `(not in registry)`; `Allowed roles: all` or custom count |
| Settings | `General`; `Mounts`; `Environments`; `Auth`; `Trust` |
| Usage empty | `No providers configured.` / `Press R to refresh.` |
| Usage unavailable | `Usage unavailable: ...` or unresolved provider notice |
| Help | `Keyboard shortcuts` |
| Dirty save | `Save changes before leaving?` |
| Debug | location labels include `list`, editor/settings mode/tab/field/modal, `quit-confirm` |

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L221-L267,L366-L410,L787-L880,L1284-L1614`
(`labels/detail`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/general_tab.rs:L39-L70`
(`General copy`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/secrets_tab.rs:L191-L238`
(`environment copy`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/usage.rs:L167-L362`
(`Usage copy`).

### 23.2 Launch copy

`Preparing launch...`; `Loading <role> in <path>`; stage labels Identity, Role,
Credentials, Construct, Agent Binaries, Derived Image, Workspace, Network,
Sidecar, Capsule, Hardline; activity is title-cased first word plus `…`;
`Docker build · building…`; `Docker build`; `(waiting for docker build output…)`;
`↳` wrapped-line prefix; failure title/summary/stage/run id/next step; `Exit
jackin❯?`; and outro `You were in the Construct for …`.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/header.rs:L43-L118`
(`header copy`); `crates/jackin-launch/src/tui/components/footer.rs:L60-L76` (`activity`);
`crates/jackin-launch/src/tui/components/build_log_dialog.rs:L64-L97,L180-L268`
(`build-log visible strings/rendering`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/failure_dialog.rs:L25-L71`
(`failure copy`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/animation.rs:L305-L330`
(`outro`).

### 23.3 Capsule copy

Status/context: `☰Menu`, `prefix…`, `PR #<number> · <title>`, `Resolving PR ·
<branch>`, `Branch · <branch>`, `Selection copied`. Palette labels include
`New tab`, `Split pane`, `Zoom / unzoom pane`, `Export file`, `Open link under
cursor`, `Clear pane`, `Usage`, `Close`, and `Exit`. Pane labels include Shell,
agent label, Agents, and Mix. Usage and dirty-exit labels preserve their
explicit statuses/actions.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/status_bar.rs:L203-L207,L282-L340`
(`status copy/glyphs`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/branch_context_bar.rs:L75-L205`
(`context copy`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/palette.rs:L97-L148`
(`palette labels`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/model.rs:L212-L335`
(`pane labels`).

<a id="s24-hard-cases"></a>

## Responsive and Hard-Case Inventory

### 24.1 Actual terminal rules

| Surface | Actual minimum/breakpoint | Behavior |
|---|---|---|
| `jackin load` rich cockpit | Requires TTYs, non-`dumb` TERM, no CI, at least 80×24 | Otherwise returns the canonical rich-terminal error before launch UI. |
| Host workspace frame | No separate global minimum guard found | Header/body/footer heights use saturating subtraction; tiny test rectangles render/clamp rather than claim a new layout. |
| Host list/detail seam | Seam draggable from width 40; default split 30% | Below 40, seam dragging is unavailable; pane geometry remains derived. |
| Host Editor/Settings | Header 3 + tabs 2 + body/footer | Body/footer shrink; modal widths use stable preferred width and shrink to outer width minus 4. |
| Launch dialogs | Preferred width is stable from a 160-column reference | Width shrinks only when too narrow; height clamps to outer height. |
| Capsule usage dialog | Under 64 columns, narrow single-column dialog | Meters go full width; reset/detail labels remain visible where possible. |
| Capsule split tree | Ratios 5–95% | Split geometry and PTY sizes update after resize. |

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/terminal.rs:L15-L52`
(`require_rich_terminal`, `terminal_supports_rich_surface`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/layout.rs:L10-L19,L41-L69,L282-L314`
(`host constants/geometry/dialog shrink`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog_widgets/usage.rs:L117-L186,L516-L593,L760-L824`
(`narrow Usage`); Capsule split `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/layout.rs:L27-L40,L145-L184`.

### 24.2 Required viewport checks

| Viewport | Current evidence/expected handling |
|---|---|
| ~80×24 | Launch is the minimum accepted rich terminal. Host render/tests use 80×24; dialogs and list/detail clamp. Capsule has direct 80×24 tests. |
| ~100×30 | Launch terminal restoration tests use 100×30; host layouts have enough rows for header/body/footer but narrower modal/detail content. |
| ~120×40 | Host mouse/modal tests and most visual baselines use 120×40; file browser reference modal is 84×22 at this size. |
| ~160×50 | Launch dialog preferred-width reference is 160 columns; stable modal width stops scaling at the preferred width. |

Evidence: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view/tests.rs:L121-L216`
(`80×24 frame/modal facts`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/mouse/tests.rs:L520-L523`
(`120×40 modal reference`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/terminal/tests.rs:L1-L15`
(`100×30 terminal`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/socket_backend/tests.rs:L38-L53`
(`80×24→120×40 resize`); launch fixed-width geometry `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/layout.rs:L282-L314`.

### 24.3 Hard content cases

- Long paths: horizontal scroll/clip in list detail, Editor mounts, and Settings
  mounts; source and destination remain distinct.
- Long names: workspace rows use horizontal content offsets; Capsule custom
  labels cap at 16; session names truncate with ellipsis in host preview.
- Many rows: expanded children are part of visual row index; focused block
  scrolls and selection clamps after refresh.
- Many instances: running summary exposes count and an expand/navigate hint;
  purged/superseded entries are excluded.
- Missing daemon: live pane tree becomes explicit empty/unavailable state.
- Missing manifest/session data: `Sessions unavailable (manifest read error)`;
  no invented session rows.
- Async refresh error: derived cache clears and error popup opens; no stale
  success is silently presented as current.
- Secret values: masked by default; diagnostics and account headers omit secret
  material.
- Modal overflow: body scrolls; backdrop does not cover reserved footer.
- Terminal resize: scroll offsets and modal rectangles are recomputed/clamped
  during preparation; Capsule forwards new pane dimensions to PTYs.

Sources: `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L492-L576,L1284-L1614`
(`scroll/detail/empty`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs:L492-L574`
(`prepare/clamp`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/model.rs:L140-L191`
(`label/cursor limits`); `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/daemon/pane_layout.rs:L177-L195`
(`resize propagation`).

<a id="s25-preview"></a>

## Future Preview Scenario Matrix

This matrix defines future evidence scenarios only. It creates no fixtures and
does not turn planned behavior into inventory.

| Scenario | Must observe | Current status |
|---|---|---|
| Empty first startup | Current directory, New workspace, no saved rows, footer/help/quit | Current |
| Populated workspace tree | Saved rows, expanded/collapsed instances, selected detail, running summary | Current |
| Live instance | Daemon tabs/panes, agent/state labels, reconnect/new-session/shell/inspect/stop/purge | Current; live data may be unavailable |
| Crashed/preserved instance | Status label, restore/relaunch branch, explicit error/confirm | Current model; exact runtime recovery is path-dependent |
| Create prelude | Each five step/rewind/same-path branch | Current |
| Editor every tab | General, mounts, roles, environments, auth; dirty/save/error | Current |
| Settings every tab | Global scope, dirty/save/error, each child modal | Current |
| Usage current/stale/unavailable | Account identity, window/reset, freshness/status, no secrets | Host renderer partial; Capsule current |
| Launch clean | All 11 stages, build/activity/footer, hardline handoff | Current |
| Launch failure | Failed stage, safe detail, copy run id, acknowledge/exit | Current |
| Launch build log | Opaque log, wrap, tail, scroll, bounded retention | Current |
| Capsule multi-pane | Split/resize/zoom/focus, PTY output/cursor, overflow | Current |
| Capsule dirty exit | Explicit row choice, no accidental destructive exit | Current contract; docs/source drift exists |
| Capsule takeover | Old attach shutdown, daemon state retained, new attach active | Current |
| Narrow 80×24 | Rich launch accepted; host/capsule modal/list clipping remains coherent | Current tests |
| Wide 160×50 | Fixed preferred dialog widths; expanded detail; no proportional modal drift | Current geometry |
| OpenCode Usage | Registry presence vs Capsule switch/order behavior | Current provider; ordering gap |
| Apple Container | Launch/attach/finalize parity and lifecycle cleanup | Planned/partial; hardware gate |
| Host event stream | Cross-container session/status updates without polling | Planned |

### 25.1 Preview scenario contracts

| ID | Initial data | State | Interaction | Visible outcome | Source |
|---|---|---|---|---|---|
| P-EMPTY | No saved workspace/instances; current directory exists | List empty | Move/select New workspace, open help, quit | Current directory, `+ New workspace`, empty guidance, footer | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L828-L880`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view/list.rs:L31-L94` |
| P-TREE | Saved workspace with instance children and mixed statuses | Expanded/collapsed List | Arrows/disclosure, select row, Tab preview | Stable row identity, status tones, selected detail, running summary | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L52-L95,L787-L826,L1284-L1614` |
| P-LIVE | Running instance with daemon tabs/sessions | Live instance preview | R/A/X/I/T/P | Live tabs/panes or explicit unavailable; action status/handler path | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L1284-L1398`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/update.rs:L25-L33` |
| P-PRELUDE | Directory or Git URL | Five-step create state | Commit, skip/edit destination, Esc rewind | Browser/choice/workdir/name sequence and Editor handoff | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/create_prelude.rs:L11-L33,L219-L378` |
| P-EDITOR-DIRTY | Workspace config with pending mount/env/role edits | Editor dirty | Tab/row edits, save, Esc | Dirty count, save preview, validation/error or List after commit | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/model.rs:L211-L295`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/save.rs:L123-L260` |
| P-SETTINGS | Global config with auth/env/mount/trust rows | Settings dirty/modal | Switch tabs, child picker/form, save/discard | Global scope retained; parent restored after child result | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model.rs:L48-L107,L617-L697`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/view.rs:L127-L147` |
| P-USAGE | Projection with multiple accounts, stale and unresolved entries | Host Usage or Capsule Usage dialog | Select account/provider, scroll, refresh in Capsule | Identity, windows, reset, meter, freshness/error; no secrets | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/usage.rs:L167-L362`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog/usage.rs:L8-L180` |
| P-LAUNCH-BUILD | Launch identity and active build log | Running pipeline/build | Open log, tail/scroll, Esc | Stage rail, activity, bounded wrapped log, return to cockpit | `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/progress_rail.rs:L15-L245`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/build_log_dialog.rs:L46-L300` |
| P-LAUNCH-FAIL | Failed stage with run/container identity | Frozen failure | Scroll, copy, Enter/Esc | Failed stage, safe detail, copy result, typed acknowledgement/exit | `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/failure_dialog.rs:L25-L71,L250-L310` |
| P-CAPSULE-MULTI | Multiple tabs/panes with PTY output | Normal/focus/drag/select modes | Prefix split/focus/resize/zoom, mouse select/copy | Pane tree, focus, cursor/output, resized PTY dimensions | `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/model.rs:L15-L229`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/daemon/pane_layout.rs:L12-L195,L339-L421` |
| P-CAPSULE-DIRTY | Active Capsule with dirty sessions | ExitDirty dialog | Escape, choose inspect/start/keep/discard | Explicit branch; daemon drain/close or durable detach | `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog.rs:L353-L440`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/input.rs:L643-L867` |
| P-RESIZE | 80×24 then 120×40 terminal | Recomputed layout | Resize, scroll, open modal | Clamped dialogs, stable frame, updated pane/PTY sizes | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view/tests.rs:L121-L216`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/socket_backend/tests.rs:L38-L53` |

<a id="s26-redesign-contract"></a>

## Redesign Coverage Contract

### 26.1 Must represent

- Operator-visible host stages: List, CreatePrelude, Editor, Settings, confirms,
  Usage, help, status/error overlays.
- Current data identity: workspace, mount, role/trust, agent/provider, instance,
  session, Capsule pane/tab, Usage account/window/freshness.
- Launch’s 11-stage ordered dependency and failure/build/container diagnostics.
- Capsule terminal output/PTY fidelity, tab/pane tree, modes, status/context
  chrome, usage, dirty exit, detach/takeover.
- Current/planned/unknown distinction and all explicit safety boundaries.

### 26.2 Must support interactively

- List selection/tree disclosure/detail, launch/reconnect/new session, shell,
  inspect, stop, purge, delete, prewarm, Settings, Usage, help, quit.
- Editor and Settings tab navigation, row mutation, modal child flows,
  masking, validation, dirty save/discard, save preview, async error recovery.
- Launch overlay scroll/copy/acknowledge/quit/cancel/handoff.
- Capsule prefix/palette, pane/tab lifecycle, focus/split/zoom/resize, scrollback,
  selection/copy, usage refresh, context actions, dirty exit, detach/exit.

### 26.3 May simulate

For a future preview only: provider responses, Docker progress timing, daemon
event cadence, filesystem contents, role registry contents, exact account meter
values, and terminal output text. Simulation must be labeled and must not be
presented as current implementation evidence.

### 26.4 Must preserve semantically

- Modal precedence and parent restoration.
- Scope boundaries: global vs workspace vs role; provider vs agent vs Usage.
- Secret masking/no-secret diagnostics; typed confirmations and fail-closed host
  actions.
- Launch stage order, failure ack, run/container identity, terminal ownership.
- Capsule PTY bytes, input routing, cursor/scrollback/focus mode semantics,
  single attach client, durable detach/takeover lifecycle.
- Current/error/unsupported/stale/unknown states; no invented loading success.

### 26.5 May redesign

Coordinates, split direction, tab placement, border treatment, colors, glyphs,
modal width, footer composition, animation timing, and information grouping
may change if the semantic and interaction obligations remain true.

### 26.6 Out of scope

New Jackin features, Apple Container completion, host-daemon event stream,
Desktop Agent Hub, new provider support, new auth backends, screenshot/image
production, UI fixture creation, and source refactors.

<a id="s27-index"></a>

## Targeted Source-Reading Index

All entries below are rooted in the pinned repository and were read at the
listed lines. `Repo path` is included so the absolute path and repository
relative path remain unambiguous.

### 27.1 Product, config, and domain

| Absolute path | Repo path / lines | Symbol | Why read |
|---|---|---|---|
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/app_config.rs` | `crates/jackin-config/src/app_config.rs:L31-L90,L92-L191` | `AppConfig` | Global auth/env/roles/runtime/telemetry/workspaces/defaults |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs` | `crates/jackin-config/src/schema.rs:L40-L106` | `DirtyExitPolicy`, `MountConfig`, `KeepAwakeConfig` | Policy/mount semantics |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs` | `crates/jackin-config/src/schema.rs:L110-L138,L260-L368` | `WorkspaceRoleOverride`, `WorkspaceConfig` | Role/env/auth/workspace contract |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs` | `crates/jackin-config/src/schema.rs:L370-L456,L544-L706` | auth/resolve/edit types | Defaults, effective workspace, role source, edits |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/agent.rs` | `crates/jackin-core/src/agent.rs:L20-L185` | `Agent`, `ALL`, auth/runtime maps | Agent/provider labels and auth modes |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/instance.rs` | `crates/jackin-core/src/instance.rs:L14-L162` | `InstanceStatus`, `SessionStatus`, records | Lifecycle and identity labels |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/launch_progress.rs` | `crates/jackin-core/src/launch_progress.rs:L14-L313` | stages/status/identity/failure | Launch facts, errors, targets, outcomes |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/env_value.rs` | `crates/jackin-core/src/env_value.rs:L11-L157` | `EnvValue`, `OpRef` | Secret/env value forms |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/env_model.rs` | `crates/jackin-core/src/env_model.rs:L76-L199` | env model | Reserved/sensitive key validation |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/auth.rs` | `crates/jackin-core/src/auth.rs:L12-L36` | `AuthForwardMode` | Auth semantic modes |

### 27.2 Host console topology and frame

| Absolute path | Repo path / lines | Symbol | Why read |
|---|---|---|---|
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model.rs` | `crates/jackin-console/src/tui/model.rs:L19-L52,L65-L83` | `ConsoleApp`, `ConsoleAppStage` | Top-level state, quit |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/stage.rs` | `crates/jackin-console/src/tui/model/stage.rs:L11-L62,L73-L108,L199-L243` | manager routes/dispatch resolver | Complete route and blocker priority |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs` | `crates/jackin-console/src/tui/view.rs:L14-L169,L492-L749` | frame/render plans | Route render, modal/backdrop/help precedence |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/layout.rs` | `crates/jackin-console/src/tui/layout.rs:L10-L19,L41-L106,L282-L314` | layout constants/helpers | Frame geometry, seam, modal shrink |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/state/manager.rs` | `crates/jackin-console/src/tui/state/manager.rs:L109-L179,L767-L818` | manager init/refresh | Startup selection and refresh errors |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/dispatch.rs` | `crates/jackin-console/src/tui/input/dispatch.rs:L47-L130,L346-L462` | dispatcher/stage outcomes | Input routing and transitions |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/state/update.rs` | `crates/jackin-console/src/tui/state/update.rs:L91-L145,L180-L300` | `update_manager` | Message reducer/effect application |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/keymap.rs` | `crates/jackin-console/src/tui/keymap.rs:L59-L252,L254-L889,L986-L1359` | editor/settings/list maps | Keyboard behavior and hints |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/mouse.rs` | `crates/jackin-console/src/tui/input/mouse.rs:L90-L307` | `handle_mouse_with_config` | Mouse precedence, seam, row click |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/terminal.rs` | `crates/jackin-console/src/tui/terminal.rs:L77-L143` | `TerminalSession` | Host terminal ownership/teardown |

### 27.3 Workspace list and create

| Absolute path | Repo path / lines | Symbol | Why read |
|---|---|---|---|
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/model.rs` | `crates/jackin-console/src/tui/screens/workspaces/model.rs:L10-L83` | list rows/hover/summary | Tree identity and preview data |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs` | `crates/jackin-console/src/tui/screens/workspaces/view.rs:L52-L95,L205-L342` | disclosure/row render | Labels, tones, selection |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs` | `crates/jackin-console/src/tui/screens/workspaces/view.rs:L787-L880,L882-L1117` | running/new/mount/env panels | Preview visible copy/content |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs` | `crates/jackin-console/src/tui/screens/workspaces/view.rs:L1148-L1614` | roles/instance/session render | Full detail/live/session states |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view/list.rs` | `crates/jackin-console/src/tui/screens/workspaces/view/list.rs:L31-L94` | `render_list_body` | Split/list/sidebar composition |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/update.rs` | `crates/jackin-console/src/tui/screens/workspaces/update.rs:L25-L121,L193-L267,L337-L520` | action/focus/tree/new-session plans | List transitions and focus |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/create_prelude.rs` | `crates/jackin-console/src/tui/model/create_prelude.rs:L11-L33,L49-L64,L145-L268,L277-L378` | wizard/state/plans | Exact prelude steps/defaults/rewinds |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/prelude.rs` | `crates/jackin-console/src/tui/input/prelude.rs:L3-L173` | `handle_prelude_key` | Prelude modal dispatch |

### 27.4 Editor

| Absolute path | Repo path / lines | Symbol | Why read |
|---|---|---|---|
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/model.rs` | `crates/jackin-console/src/tui/screens/editor/model.rs:L21-L49,L249-L295` | `EditorTab`, `EditorState` | Tabs and pending state |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/model.rs` | `crates/jackin-console/src/tui/screens/editor/model.rs:L356-L510` | row/focus/modal targets | Secret/auth/confirm identity |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/general_tab.rs` | `crates/jackin-console/src/tui/screens/editor/view/general_tab.rs:L39-L70` | `general_form_section` | General fields/copy |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/mounts_tab.rs` | `crates/jackin-console/src/tui/screens/editor/view/mounts_tab.rs:L65-L154` | `mount_lines` | Mount fields/sentinel |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/roles_tab.rs` | `crates/jackin-console/src/tui/screens/editor/view/roles_tab.rs:L19-L49` | role rows | Allowed/default/load presentation |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/secrets_tab.rs` | `crates/jackin-console/src/tui/screens/editor/view/secrets_tab.rs:L23-L64,L93-L124,L191-L238` | secret rows | Mask/scope/role env content |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/view/auth_tab.rs` | `crates/jackin-console/src/tui/screens/editor/view/auth_tab.rs:L23-L87` | auth rows | Auth form/list presentation |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/editor.rs` | `crates/jackin-console/src/tui/input/editor.rs:L127-L523,L538-L1104` | `handle_editor_key/modal` | Editor action/modal/save dispatch |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/save.rs` | `crates/jackin-console/src/tui/input/save.rs:L123-L260` | save planner/commit | Validation, preview, write |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/components/save_preview.rs` | `crates/jackin-console/src/tui/components/save_preview.rs:L31-L124,L1278-L1291` | save preview | Visible diff semantics |

### 27.5 Settings and modals

| Absolute path | Repo path / lines | Symbol | Why read |
|---|---|---|---|
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model.rs` | `crates/jackin-console/src/tui/screens/settings/model.rs:L48-L107,L523-L697` | tabs/state/modal enum | Full Settings scope/modal catalog |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/view.rs` | `crates/jackin-console/src/tui/screens/settings/view.rs:L101-L460` | frame/render/footer | Settings geometry and copy |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model/general_impls.rs` | `crates/jackin-console/src/tui/screens/settings/model/general_impls.rs:L10-L67` | `SettingsGeneralState` | General flags/save |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model/env_impls.rs` | `crates/jackin-console/src/tui/screens/settings/model/env_impls.rs:L10-L202` | env state | Global/role env behavior |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model/auth_impls.rs` | `crates/jackin-console/src/tui/screens/settings/model/auth_impls.rs:L12-L315` | auth state | Auth child/modal chain/token path |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model/trust_impls.rs` | `crates/jackin-console/src/tui/screens/settings/model/trust_impls.rs:L10-L131` | trust state | Trust rows/toggle/save |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/model/modal.rs` | `crates/jackin-console/src/tui/model/modal.rs:L22-L119,L169-L253` | `ConsoleModal` | Generic modal variants and targets |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view.rs` | `crates/jackin-console/src/tui/view.rs:L423-L488,L641-L725` | modal renderer/plans | Variant render and priority |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/components/file_browser` | `crates/jackin-console/src/tui/components/file_browser` | file browser state/input | Browser selection/navigation contract |

### 27.6 Usage

| Absolute path | Repo path / lines | Symbol | Why read |
|---|---|---|---|
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage.rs` | `crates/jackin-usage/src/usage.rs:L39-L48,L200-L313` | modules/surfaces/order | Complete provider registry |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/host.rs` | `crates/jackin-usage/src/host.rs:L72-L231,L394-L523` | host IDs/glance/chips | Host order, labels, compact slots |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-protocol/src/usage_broker.rs` | `crates/jackin-protocol/src/usage_broker.rs:L10-L275,L360-L503` | protocol/projection | Freshness/status/error/window contract |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-protocol/src/control.rs` | `crates/jackin-protocol/src/control.rs:L501-L581,L680-L843,L926-L974` | usage views/buckets/status | Normalized account/focused presentation |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/usage.rs` | `crates/jackin-console/src/tui/screens/usage.rs:L20-L362` | host Usage screen | Render/keys/meter and integration gap |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-runtime/src/usage_relay.rs` | `crates/jackin-runtime/src/usage_relay.rs:L212-L300,L347-L393,L440-L594` | usage relay | Capsule capability boundary |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/claude.rs` | `crates/jackin-usage/src/usage/claude.rs:L16-L68,L101-L262,L630-L760` | Claude adapter | Auth/windows/fallback |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/codex.rs` | `crates/jackin-usage/src/usage/codex.rs:L12-L29,L90-L243,L410-L503,L674-L780` | Codex adapter | RPC/REST/quota/failure |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/amp.rs` | `crates/jackin-usage/src/usage/amp.rs:L15-L126,L165-L238` | Amp adapter | Secret/account/daily quota |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/grok.rs` | `crates/jackin-usage/src/usage/grok.rs:L12-L37,L39-L125,L253-L318` | Grok adapter | Auth/billing/error |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/kimi.rs` | `crates/jackin-usage/src/usage/kimi.rs:L16-L86,L88-L159,L221-L283` | Kimi adapter | Presence/secret/windows |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/minimax.rs` | `crates/jackin-usage/src/usage/minimax.rs:L16-L74,L76-L198,L239-L302` | MiniMax adapter | Endpoint/window parsing |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/opencode.rs` | `crates/jackin-usage/src/usage/opencode.rs:L4-L9,L55-L78,L99-L205` | OpenCode adapter | HTTP status mapping |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage/zai.rs` | `crates/jackin-usage/src/usage/zai.rs:L16-L81,L83-L161,L164-L295` | Z.AI adapter | Token/credit/window mapping |

### 27.7 Launch, Capsule, tests, and docs

| Absolute path | Repo path / lines | Symbol | Why read |
|---|---|---|---|
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/model.rs` | `crates/jackin-launch/src/tui/model.rs:L25-L84` | `LaunchView` | Cockpit state/overlays |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/update.rs` | `crates/jackin-launch/src/tui/update.rs:L16-L239` | view update/frontier | Stage progression and failure freeze |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/progress_rail.rs` | `crates/jackin-launch/src/tui/components/progress_rail.rs:L15-L245` | rail renderer | 11-stage visual semantics |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/build_log_dialog.rs` | `crates/jackin-launch/src/tui/components/build_log_dialog.rs:L46-L300` | build-log renderer | Opaque log/wrap/scroll |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/failure_dialog.rs` | `crates/jackin-launch/src/tui/components/failure_dialog.rs:L25-L71,L250-L310` | failure dialog | Safe failure copy/copy target |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/components/container_info_dialog.rs` | `crates/jackin-launch/src/tui/components/container_info_dialog.rs:L21-L120` | container info | Runtime/debug facts |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/subscriptions.rs` | `crates/jackin-launch/src/tui/subscriptions.rs:L42-L63,L239-L293,L347-L404,L557-L639,L778-L835` | mouse/quit/failure/log routing | Overlay actions and abort semantics |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/model.rs` | `crates/jackin-capsule/src/tui/model.rs:L15-L229,L231-L349` | mux/hover/cursor/state | Modes, status, pointer, pane labels |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/input.rs` | `crates/jackin-capsule/src/tui/input.rs:L197-L335,L350-L590,L643-L867` | parser/input events | Terminal input/fidelity |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/keymap.rs` | `crates/jackin-capsule/src/tui/keymap.rs:L22-L467` | keymaps | Prefix/dialog/resize actions |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog.rs` | `crates/jackin-capsule/src/tui/components/dialog.rs:L146-L440` | dialog/actions | Capsule overlay catalog |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/status_bar.rs` | `crates/jackin-capsule/src/tui/components/status_bar.rs:L50-L340` | status bar/glyphs | Tab/menu/status chrome |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/branch_context_bar.rs` | `crates/jackin-capsule/src/tui/components/branch_context_bar.rs:L52-L205,L345-L395` | context/chip slots | Branch/PR/Usage/container/debug |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/view.rs` | `crates/jackin-capsule/src/tui/view.rs:L173-L324` | compositor | Full frame/overlay/toast |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-brand/src/lib.rs` | `crates/jackin-brand/src/lib.rs:L34-L79` | brand constants | Exact visual tokens |
| `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/view/baselines/png` | directory, 53 PNGs | baseline registry | Existing visual evidence |
| `/Users/donbeave/junie-style-2/jackin/docs/content/reference/developer-reference/specs/operator-console.mdx` | `docs/content/reference/developer-reference/specs/operator-console.mdx:L5-L71` | operator-console spec | Host invariants |
| `/Users/donbeave/junie-style-2/jackin/docs/content/reference/capsule/multiplexer-design-rules.mdx` | `docs/content/reference/capsule/multiplexer-design-rules.mdx:L5-L180` | mux rules | Terminal fidelity invariant |
| `/Users/donbeave/junie-style-2/jackin/docs/content/roadmap/in-progress.mdx` | `docs/content/roadmap/in-progress.mdx:L10-L49` | roadmap registry | Planned/current boundary |

<a id="s28-citations"></a>

### Source-citation format

Each substantive implementation claim uses this form:

```text
Source: /Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/launch_progress.rs:L14-L74
(`LaunchStage`) — defines ordered launch-stage identity and visible labels;
commit 8d161b3b41c64da0de3ab5f4aef1969316c193d1.
```

Tables may put the explanation in the “why” column and repeat the SHA in the
section-level source line. Body citations use full absolute paths. Tests/docs citations identify whether
they prove behavior, layout, a contract, or only intent.

Citation resolution rule: every source citation in this file inherits the pinned
Jackin root and SHA above. A short path in a table resolves under
`/Users/donbeave/junie-style-2/jackin`; the exact-path source index in §27 is the
canonical expansion. Every evidence row must still provide a real path, symbol
or subsystem, and in-range line span. A citation is rejected if its path is
missing, its range exceeds the pinned file, or its content belongs to another
commit. Repeating the 40-character SHA on every table cell would add noise; the
single source lock is normative for all rows. A continuation range with no
repeated path attaches to the immediately preceding path in the same source
sentence.

<a id="s29-current-planned"></a>

## Current Versus Planned

| Classification | Included here |
|---|---|
| Current | Host List/Editor/Settings/modal system, launch cockpit, Capsule TUI, provider adapters, normalized Usage model, current tests/baselines |
| Partial | OpenCode Capsule/desktop ordering; some runtime recovery/Apple backend parity; docs/source mismatches named in this file |
| Planned | Capsule host event stream/Phase 4, broader host-daemon reconciliation, Apple finalize/hardware parity, future host bridge/Desktop Agent Hub, new auth/provider work |
| Research-only | Design-history pages, roadmap ideas, future layout choices, preview scenarios |

Roadmap pages describe unfinished intent only; shipped behavior remains in
source/public/reference docs. Source: `/Users/donbeave/junie-style-2/jackin/docs/content/roadmap/index.mdx:L8-L44`
(`roadmap boundary`); current Capsule boundary:
`docs/content/roadmap/(reactive-daemon-program)/jackin-capsule.mdx:L7-L47`.

Known current/source-doc tensions:

- Console docs describe polling/cache contracts while live instance preview
  can render daemon snapshots; persisted manifest session data and live data
  remain separate.
- Capsule dirty-exit source accepts some Escape/Keep paths that a design page
  describes more strictly; this is recorded as drift, not normalized away.
- Agent source includes Grok even where older docs omit it.
- Usage registry includes OpenCode while some Capsule/provider switch orders do
  not expose it.
- Apple backend selection is wired but finalization/hardware parity is not
  shipped.

### 29.1 Capability/evidence matrix

| Capability | Classification | Current evidence | Planned boundary | Source |
|---|---|---|---|---|
| Host workspace manager | Current | List tree, preview, instance actions, empty/error states, mouse seam | Coordinates may change; identity/actions must remain | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/view.rs:L52-L95,L1284-L1614`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/workspaces/update.rs:L25-L121` |
| Workspace Editor | Current | Five tabs, pending/original isolation, child modals, validation/save | No new fields implied by preview | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/editor/model.rs:L21-L49,L211-L295`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/input/editor.rs:L127-L523,L538-L1104` |
| Global Settings | Current | General/Mounts/Environments/Auth/Trust with global scope | Keep distinct from workspace Editor | `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/settings/model.rs:L48-L107,L617-L697` |
| Host Usage | Current | Broker-backed startup projection, `r` refresh, rows, detail, meter, freshness/error labels | Provider work stays broker-owned | `/Users/donbeave/junie-style-2/jackin/crates/jackin/src/console/adapter/run.rs:L49-L133,L859-L864,L1070-L1087`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-console/src/tui/screens/usage.rs:L45-L95,L167-L362` |
| Capsule Usage | Current | Chip/dialog, provider tabs, refresh, stale/error states | Relay capability remains opaque | `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/components/dialog/usage.rs:L8-L180`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-runtime/src/usage_relay.rs:L212-L300,L347-L393` |
| Launch cockpit | Current | 11-stage rail, activity, build/failure/info/cancel/quit surfaces | Handoff/Apple parity remains path-dependent | `/Users/donbeave/junie-style-2/jackin/crates/jackin-core/src/launch_progress.rs:L14-L177,L198-L313`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-launch/src/tui/run.rs:L80-L188,L412-L472` |
| Capsule terminal/multiplexer | Current | PTY output/input, panes/tabs, modes, focus/resize, dialogs, dirty exit | Host event stream is not current | `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/model.rs:L15-L229,L231-L349`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-capsule/src/tui/input.rs:L197-L335,L643-L867` |
| Provider/account adapters | Current/partial by provider | Registry, credential discovery, provider-specific windows/errors, normalized projection | New providers/auth backends out of scope; ordering gap recorded | `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/usage.rs:L200-L313`; `/Users/donbeave/junie-style-2/jackin/crates/jackin-usage/src/host/projection.rs:L158-L272` |
| Host event stream | Planned | No current Console control or aggregation path found | Roadmap Phase 4 / daemon reconciliation | `/Users/donbeave/junie-style-2/jackin/docs/content/roadmap/in-progress.mdx:L10-L49`; `/Users/donbeave/junie-style-2/jackin/docs/content/roadmap/(reactive-daemon-program)/jackin-capsule.mdx:L7-L47` |
| Desktop Agent Hub/bridge | Planned | No current TUI surface in audited tree | Future host bridge only | `/Users/donbeave/junie-style-2/jackin/docs/content/roadmap/index.mdx:L8-L44` |
| Apple Container backend | Partial/planned | Runtime backend selection exists; finalize/hardware parity unresolved | Do not present full parity as current | `/Users/donbeave/junie-style-2/jackin/crates/jackin-config/src/schema.rs:L176-L194,L330-L368`; `/Users/donbeave/junie-style-2/jackin/docs/content/roadmap/in-progress.mdx:L10-L49` |

Classification is evidence-scoped: “current” means a source-backed path exists;
“partial” means a renderer or adapter exists without a proven end-to-end path;
“planned” means roadmap/absence evidence only. No row grants future behavior to
the current inventory.

<a id="s30-quality"></a>

### Document quality rules

- No pasted source files. All prose is normalized synthesis.
- No generic TUI theory is used as evidence.
- No vague “supports X” claim without source, line range, symbol, and reason.
- “No evidence found” is scoped to the audited tree/search, never presented as
  proof of nonexistence outside it.
- Current visual baselines are named, not copied or regenerated.
- Planned behavior is never placed in the current surface inventory.
- Exact copy is quoted only when source-backed and useful to operators.
- The document describes layout and interaction; it does not propose a future
  layout.
- Secrets, token values, and credential material are not reproduced.

<a id="s31-audits"></a>

## Completeness Audits

Coverage registries, aggregate specifications, workflows, and source index are
present. Atomic registry rows cross-link to aggregate specifications and are not
claimed as full per-child specifications.

## Final Verification Record

- Pinned local Jackin HEAD: `8d161b3b41c64da0de3ab5f4aef1969316c193d1`.
- Atomic registry: 87 unique child IDs covering Editor tabs, Settings tabs and
  modal variants, host modal variants, Usage provider variants, launch stages,
  Capsule modes, and Capsule dialogs. These IDs cross-link to aggregate surface
  specifications; inventory rows are not substitutes for full per-child specs.
- Host Usage: current broker-backed startup population and `r` refresh verified
  at `crates/jackin/src/console/adapter/run.rs:L49-L133,L859-L864,L1070-L1087`.
- No remote URLs or non-local evidence sources are permitted.
