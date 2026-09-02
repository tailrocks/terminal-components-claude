You are a principal product designer, terminal UX architect, and senior Rust/Ratatui engineer.

You are improving the already-implemented `jackin-preview` after receiving a new round of customer feedback.

This is NOT a request to redesign the product from scratch.

The current Jackin Preview is already the approved baseline. Preserve everything that is working and everything that is not explicitly superseded by this feedback. Your job is to make the existing experience substantially more polished, coherent, scalable, and intuitive while keeping the existing Junie-inspired design language intact.

Do not stop at analysis or a plan. Inspect the implementation, make the changes, run the application, interact with it, capture it, test it, and iterate until the result is visually and behaviorally finished.

Use subagents aggressively where supported. Parallelize source inspection, interaction auditing, account-model work, reusable-component work, visual QA, and independent verification. The primary agent remains responsible for the final integrated quality.

# Authority order

When requirements conflict, use this precedence:

1. This customer-feedback goal — highest authority for changes requested here.
2. `DESIGN.md` — visual language, spacing, color, component and interaction principles.
3. `JACKIN_REFERENCE.md` — Jackin semantics, terminology, historic behavior, and especially the older Capsule chrome/status experience.
4. Existing `jackin-preview` implementation — baseline behavior that must not regress.
5. Existing `GOAL.md` — original preview intent and coverage.

Read all relevant implementation before changing it, especially:

* `src/bin/jackin_preview/`
* `src/bin/jackin_preview/app.rs`
* `src/bin/jackin_preview/screens/capsule.rs`
* `src/bin/jackin_preview/screens/accounts.rs`
* `src/bin/jackin_preview/screens/config.rs`
* `src/bin/jackin_preview/screens/editor.rs`
* `src/bin/jackin_preview/screens/settings.rs`
* `src/bin/jackin_preview/screens/usage.rs`
* `src/bin/jackin_preview/domain/account.rs`
* `src/bin/jackin_preview/domain/workspace.rs`
* `src/bin/jackin_preview/domain/instance.rs`
* `src/widgets/tabs.rs`
* `src/widgets/picker.rs`
* `src/widgets/progress.rs`
* `src/widgets/scrollbar.rs`
* `src/widgets/keyhint.rs`
* `src/widgets/tree.rs`
* `src/widgets/viewport.rs`
* `src/widgets/code.rs`
* `src/theme.rs`

Prefer fixing or extending reusable primitives instead of adding screen-specific rendering hacks.

When a new reusable interaction or visual rule is established, update `DESIGN.md` so the documentation describes the actual final system.

# Core quality bar

The result must feel like a premium, deliberate terminal application rather than a collection of individually styled Ratatui screens.

Maintain the existing restrained Junie/Jackin visual system:

* near-black surfaces and hierarchy
* carefully restrained phosphor/green accent
* strong white/gray information hierarchy
* background elevation instead of excessive borders
* compact but breathable spacing
* strong hover/focus/selection distinction
* terminal-native keyboard behavior
* first-class mouse and trackpad behavior
* consistent overlays and anchored menus
* no gratuitous decorative Unicode
* no generic ncurses look
* no dense dashboard clutter
* no duplicated UI patterns implemented independently

This feedback is about both DESIGN and UX. Do not satisfy it merely by changing colors.

# 1. Redesign the Capsule status bar

The Capsule status/context bar currently displays information such as:

* branch / GitHub context
* current session/context
* weekly or provider Usage
* container/runtime information
* debug/run information

Its POSITION is fundamentally fine.

Its DESIGN is not.

Create a dedicated reusable status-bar component and redesign this surface so it is visually distinct from ordinary content, tabs, hints, lists, or panels.

The older Jackin Capsule status/context chrome described in `JACKIN_REFERENCE.md` is an important reference. Reinterpret its visual prominence through the CURRENT Junie-inspired design system rather than literally restoring an old theme.

Requirements:

* visibly separate the status bar from ordinary content
* use a deliberate full-width surface/background treatment
* make left, center, and right information groups immediately readable
* use spacing and tonal grouping rather than lots of separators
* gracefully prioritize/hide low-priority information on narrow terminals
* preserve branch/PR context
* preserve session/runtime context
* preserve weekly/provider usage
* preserve container/debug information where applicable
* support clickable regions where they already make sense
* provide hover states
* work at minimum supported terminal width
* never become a noisy collection of unrelated chips

Do not hard-code Jackin domain semantics into the generic component. The reusable status-bar primitive should accept structured sections/items and priorities; the Capsule supplies Jackin data.

The final status bar should be one of the visually characteristic parts of Jackin.

# 2. One canonical Jackin logo everywhere

Audit every place where Jackin branding is rendered.

There must be ONE canonical Jackin logo/brand treatment everywhere.

Create or centralize a reusable Jackin brand component instead of manually constructing the logo in multiple screens.

The logo must use the phosphor/accent background treatment requested by the customer.

This brand lockup is allowed to use stronger phosphor emphasis than ordinary controls. Do NOT use that as justification for turning the whole application green.

Use the same:

* glyph/mark
* `Jackin` text treatment
* foreground/background relationship
* padding
* casing
* spacing

across every screen that shows the product identity.

Remove competing versions such as independent `▪ Jackin`, plain text Jackin headers, alternate logo glyphs, or differently styled variants unless a deliberately compact version of the SAME component is required.

# 3. Permanent context-aware bottom HintBar

Button/key hints must remain in ONE stable physical location: the bottom of the application window.

The footer must not jump vertically or disappear when navigating.

Create or improve a reusable `HintBar` abstraction owned by the application shell.

The bottom HintBar must adapt to the current interaction context:

normal screen:
hints for the active screen

dialog open:
dialog-specific hints

picker open:
picker-specific hints

context menu open:
menu navigation / choose / close hints

diff inspector:
diff navigation / mode switching hints

editing mode:
editing/save/cancel hints

Capsule prefix mode:
prefix-related hints

Do not render one unrelated global footer underneath a modal while separately rendering another hint row inside the modal.

There should be one canonical hint surface.

Overlay precedence must be approximately:

topmost modal/menu interaction
>
temporary mode
>
active screen
>
global fallback

The footer itself remains physically pinned to the bottom.

Hints should be concise enough that they do not become a sentence-heavy command reference.

# 4. Agent tab context menu

Capsule coding-agent tabs need a proper context menu.

The tab menu must include at minimum:

* Change title
* Close

The menu should be anchored visually to the selected tab and behave like a real context menu rather than a centered generic confirmation dialog.

Support:

* mouse secondary/context action where the runtime exposes it
* a keyboard-accessible equivalent
* keyboard navigation inside the menu
* mouse hover
* click selection
* Esc dismissal
* correct HintBar contents while open

`Change title` should provide a polished rename interaction.

`Close` must use the existing safe-close semantics where confirmation is necessary.

Do not implement this as a one-off drawing function if an anchored reusable ContextMenu/Menu primitive is the correct abstraction.

# 5. Redesign selected tabs globally

Audit ALL tab systems, including the reusable `Tabs` widget and the custom Capsule agent-tab strip.

Selected/active tabs must follow one consistent visual rule:

* selected tab has a distinct BACKGROUND
* selected tab has the green/phosphor UNDERLINE
* selected tab does NOT have the green left-side vertical gutter/border
* hover remains distinguishable from selected
* focus remains distinguishable from selected
* inactive tabs stay visually quiet

The existing left `▎` selection treatment must not be used for tabs.

Do not globally remove that treatment from list rows or other components where the gutter is still appropriate. This rule specifically changes tab selection.

The active background should come from the semantic surface system; do not make the entire tab phosphor green.

Use the shared tab implementation wherever practical so the rule cannot drift between screens.

# 6. Redesign progress and Usage meters

Progress/quota meters need a richer semantic system.

We need visual differentiation between:

* low
* medium
* high

For Usage/quota consumption, use the semantic meaning of the value:

low usage / healthy capacity
medium usage / attention
high usage / danger or exhaustion

Where the domain already exposes quota status, use that status rather than recomputing contradictory semantics.

Where no domain threshold exists, centralize sensible defaults rather than scattering numbers through screens. A reasonable default is:

* low: 0–59%
* medium: 60–84%
* high: 85–100%

Make these thresholds easy to adjust.

Most importantly, extend the progress/meter widget with configurable VISUAL MODES.

At minimum support:

1. Line/track mode

   * equivalent to the current compact `━━━━────` treatment

2. Block/background mode

   * the filled portion is represented by a strong background block
   * the whole used region feels visually filled, not like a tiny one-character line
   * text remains readable with appropriate contrast

The customer specifically wants the strong background-block presentation available again, similar in visual weight to older Jackin.

The choice must be a real widget option/type, not duplicated rendering implementations.

For example, conceptually:

`MeterVisual::Line`
`MeterVisual::Block`

Names are up to you.

Usage surfaces should preferentially demonstrate the new block mode.

Preserve coherent states for:

* normal
* warning
* exhausted
* stale
* refreshing
* error
* unknown/no quota

Do not use completion-green semantics incorrectly for generic task progress. Usage capacity and task completion remain different concepts.

# 7. Expand “Inspect changes” into two useful modes

The current Inspect Changes experience must remain available but be significantly expanded.

Build a proper change-inspection experience with TWO modes.

## 7.1 Current/compact mode

Preserve the current lightweight list-oriented experience.

Improve it so selecting/opening a changed file displays that individual file's diff.

Provide a preview mode conceptually representing either:

* standard `git diff`
* TUICR-style diff

Do NOT integrate the real `tuicr` executable in this goal.

This is still a deterministic preview application. Simulate realistic diff content using fixtures.

## 7.2 Advanced mode

Add an advanced change-inspection view with:

LEFT:

* hierarchical changed-file tree
* folders/files
* changed-state indicators
* keyboard navigation
* mouse navigation
* scrolling

RIGHT:

* selected file diff preview
* standard unified git-diff presentation
* optional TUICR-inspired presentation/view mode

The advanced view should feel like a proper source-control change viewer.

Provide a clear way to switch:

* compact ↔ advanced
* standard diff ↔ TUICR preview

Use existing reusable primitives such as tree, viewport, code rendering, splitters, scroll state, etc., extending them cleanly when necessary.

Do not build a disconnected miniature UI framework just for this screen.

For this goal, the TUICR mode is a visual/product preview only. No external process integration is required.

# 8. Fix Command Palette mouse/trackpad scrolling

The Command Palette currently does not behave correctly when scrolling with a mouse wheel or trackpad.

Fix it.

Do not merely change the scrollbar appearance.

The visible rows must actually move when wheel events are received.

Inspect the Picker's relationship between:

* `ScrollState`
* cursor
* `ensure_visible`
* rendering
* wheel events

Do not allow the render pass to immediately undo a legitimate wheel scroll.

Define intentional semantics for selection while wheel-scrolling.

A good UX is either:

* wheel scrolls the viewport while preserving selection until navigation resumes

or

* wheel moves the cursor/selection together with the viewport

Choose the behavior that fits the existing design system best and use it consistently.

# 9. Audit ALL scrollable surfaces

Treat mouse/trackpad scrolling as a system invariant.

Audit every scrollable interaction in `jackin-preview`, not just Command Palette.

At minimum verify:

* command palette / Picker
* lists
* tables
* help overlays
* Usage surfaces
* Accounts surfaces
* Workspace Manager
* Workspace Editor
* Settings
* Environments
* account/role selectors
* trees
* Inspect Changes file tree
* diff preview
* long dialogs
* Capsule scrollback
* any code/log viewport
* any screen with a scrollbar

Requirements:

* wheel/trackpad scrolling works when the pointer is over the scrollable viewport
* user should not need to click the viewport before wheel scrolling it
* wheel input applies to the topmost modal when a modal is open
* a modal must not accidentally scroll content behind it
* nested regions have deterministic scroll ownership
* scrollbar thumb updates with the viewport
* cursor/selection and viewport do not fight each other
* PageUp/PageDown and keyboard navigation continue to work
* vertical wheel works
* horizontal wheel works where horizontal scrolling is meaningful and already supported by the runtime
* scroll boundaries behave naturally
* no render-time state reset silently undoes wheel input

Centralize this behavior where possible.

Add regression tests specifically proving the Picker/Command Palette problem is fixed.

# 10. Replace useless Capsule identity strip with a real menu bar

Inside Capsule, remove this type of line entirely:

`▪ Jackin  inside the Construct  1 running  Capsule › ...  1 instance  ? help`

The customer considers this line useless.

Do not simply shorten it.

Replace it with a proper application menu bar similar to conventional desktop/TUI applications.

Desired Capsule hierarchy:

ROW 1:
Jackin application menu bar

NEXT ROW:
coding-agent tabs

BODY:
PTY/panes/content

BOTTOM CHROME:
redesigned status bar

FINAL ROW:
persistent context-aware HintBar

The menu bar should begin with the canonical Jackin logo, conceptually like an Apple/app menu, followed by traditional menu groups such as:

Jackin logo   File   Edit   View   Session   Help

Adjust the exact categories based on actual Jackin actions, but keep the familiar desktop-menu mental model.

The menus must be FUNCTIONAL in the preview.

Use existing commands rather than inventing meaningless menu items.

For example, appropriate actions may include:

File:

* New tab
* Split
* Export
* Close

Edit:

* Copy
* Paste
* Select / clear

View:

* Zoom
* Redraw
* Usage
* Inspect/context views

Session:

* Change agent/account where applicable
* Detach
* Close
* Exit

Help:

* Help / key reference

The Jackin-logo menu may expose application-level destinations such as Settings, Account & Usage Center, or product information if that fits the current routing model.

Support keyboard and mouse use.

Menu dropdowns should be anchored beneath menu labels and use the same reusable context/menu primitive as other anchored menus when sensible.

Do not reproduce the information removed from the old identity strip in another equally noisy sentence.

Information that still matters should live in the proper status bar or menu.

# 11. Hide coding agents that have no usable account

When opening a new Capsule tab or choosing an agent, do NOT show agents that have no configured account available for the current Workspace.

The current experience of listing an agent and saying `needs account` is not wanted.

An unconfigured coding agent should simply not appear as an available runtime option.

Shell remains available independently.

Resolution rules must account for:

* globally enabled accounts
* Workspace account activation
* Workspace exclusions
* account provider
* account lifecycle
* account availability

Do not show synthetic unavailable options just to fill the list.

If an account WAS deliberately configured/assigned but is temporarily invalid, stale, needs login, etc., you may show an intentional disabled/error state when that provides actionable information. The important requirement is that completely unconfigured agents do not appear as if they are usable.

# 12. Redesign account ownership and Workspace account assignment

This is a significant product-model improvement.

The Account & Usage Center should become the canonical place where coding-agent/provider accounts are REGISTERED and managed.

A Workspace should NOT duplicate account credential setup.

The mental model must become:

GLOBAL ACCOUNT REGISTRY
↓
registered accounts
↓
global defaults
↓
Workspace chooses which accounts are available
↓
Capsule/session chooses from Workspace-available accounts

## Global account management

Account & Usage Center owns:

* registering an account
* credential source
* validation
* rename
* enable/disable
* removal
* provider association
* safe identity metadata
* Usage
* global default account selection

Never duplicate secret entry across every Workspace.

Never expose secret values in the UI.

## Workspace account policy

Add a proper Workspace Accounts configuration experience.

It should behave conceptually like Role assignment: the Workspace sees accounts from the global registry and controls which ones are active.

A Workspace must be able to:

* inherit globally default accounts
* explicitly disable an inherited default
* enable additional accounts
* disable accounts for this Workspace without deleting them globally
* enable MULTIPLE accounts
* enable two accounts for the same provider
* see which entries are inherited/default versus explicitly enabled
* understand the effective set without reading internal precedence rules

Do not model Workspace configuration as only one `provider -> account` override anymore. Availability and preferred/default selection are separate concepts.

The existing provider override can evolve into a preference/default choice if useful, but it cannot remain the entire availability model.

## Multiple accounts in a container

A Workspace with two activated accounts must be able to launch a simulated instance/container with BOTH accounts available.

That means launch/runtime fixtures must no longer conceptually reduce the Workspace to only one account.

The instance/Capsule should know the effective account set available to it.

When creating a session:

* zero eligible accounts for an agent → that agent is not offered
* exactly one eligible account → use it automatically
* multiple eligible accounts → show the account picker
* Workspace-preferred/global-default account may be preselected when appropriate

The account picker must only display accounts that are effective for that Workspace.

Demonstrate this with fixtures where two accounts are enabled simultaneously and can be selected for different Capsule tabs/sessions.

## Workspace UI

Do not dump every account into a noisy screen.

Use:

* compact summaries
* search/filter if necessary
* clear inherited/explicit state
* enable/disable controls
* provider grouping where useful

Make the effective behavior immediately understandable.

## Auth cleanup

Review the existing Workspace `Auth` experience.

For CODING AGENT ACCOUNTS, replace credential configuration with the new global-account + Workspace-assignment model.

Do not leave two competing ways to configure the same coding-agent identity.

If an Auth concept is still legitimately required for something outside the account registry, keep only that distinct responsibility and label it clearly.

No legacy duplicate workflow should remain merely for compatibility with the old preview.

# 13. Make Environments scale to hundreds of Roles

The current Environments UX must not render a giant empty section for every available Role.

A Workspace may eventually have 100+ Roles.

Design for that now.

The main Environments surface should primarily show:

* Workspace/global environment entries
* Role scopes that ACTUALLY contain overrides/configuration

Do not expand every possible Role as a permanent section.

Provide an intentional interaction such as:

`Add role override…`

that opens a searchable Role picker.

After a Role has environment configuration, it can appear as a collapsible Role section.

Use compact summaries such as:

`Role overrides · 4 configured`

rather than listing 100 empty groups.

Apply the same scalability principle to Global Settings where Role-scoped environment configuration is supported.

Also inspect the Auth/account/scoped-config interfaces for the same anti-pattern and fix it where the identical scalability issue exists.

The registry may contain hundreds of Roles without making the default screen unreadable.

# Reusable component expectations

This feedback should strengthen the design system rather than make `jackin_preview` more bespoke.

Create or improve reusable components where appropriate, likely including concepts such as:

* Brand / JackinLogo
* StatusBar
* HintBar
* MenuBar
* ContextMenu / anchored menu
* progress/meter visual mode
* tab selected treatment
* scroll/wheel handling
* diff viewer composition if sufficiently generic

Exact names are your decision.

Do NOT put Jackin domain strings into a generic widget just to claim it is reusable.

Do NOT create abstractions that are used once and make the code harder to understand.

Use the existing primitives first.

# Interaction invariants

Everything new must work through both keyboard and mouse whenever the interaction makes sense.

For interactive controls, verify:

* keyboard focus
* keyboard activation
* mouse hover
* mouse activation
* disabled state
* active/selected state
* modal precedence
* Esc behavior
* resizing
* narrow terminal behavior

The terminal must never enter a state where the user cannot determine:

* what is selected
* what is focused
* what dialog/menu is active
* what keys are currently meaningful

That is the purpose of the persistent HintBar.

# Responsive requirements

Verify the finished app at several terminal sizes, including at least:

* 80×24
* approximately 120×36
* a wide desktop terminal

Also verify the minimum supported dimensions from the current application.

At narrow widths:

* status-bar items collapse by priority
* menus remain usable
* tabs use the existing overflow mechanism
* hints truncate intentionally
* dialog content remains reachable
* the diff inspector can degrade gracefully
* no important action becomes unreachable

Do not solve narrow layouts merely by clipping content.

# Regression requirements

Do not regress any existing major `jackin-preview` workflow.

In particular preserve:

* Construct intro/outro
* Workspace Manager
* create Workspace flow
* Workspace editing
* Settings
* Account & Usage Center
* Usage
* launch cockpit
* Capsule tabs/panes
* split/zoom/close
* command palette
* dialogs
* attach/detach/reconnect
* takeover
* dirty-exit handling
* mouse behavior
* deterministic scenarios/captures

# Testing strategy

Add unit and interaction tests for the new behavior.

At minimum test:

1. generic tabs:

   * active background
   * accent underline
   * no selected left gutter

2. Picker:

   * wheel down changes visible content
   * wheel up restores it
   * rendering does not reset wheel movement
   * selection remains coherent

3. progress/meter:

   * low/medium/high semantics
   * line mode
   * block mode
   * stale/error/exhausted states

4. Workspace accounts:

   * global default inherited
   * inherited default can be disabled
   * explicit additional account can be enabled
   * two accounts may be active together
   * two accounts from one provider can coexist
   * effective account resolution is deterministic

5. Capsule agent picker:

   * unconfigured agent absent
   * one account auto-resolves
   * multiple accounts open account picker
   * only Workspace-effective accounts appear

6. Environments:

   * registry with 100+ Roles does not create 100 empty sections
   * configured Role overrides remain visible
   * searchable add-role flow works

7. agent tab context menu:

   * rename
   * close
   * dismissal
   * mouse and keyboard path where supported

8. HintBar:

   * base screen hints
   * dialog hints
   * picker/menu hints
   * stable bottom position

9. Inspect Changes:

   * compact mode
   * advanced mode
   * tree selection
   * diff selection
   * standard/TUICR preview toggle
   * wheel scrolling in both tree and preview

# Visual verification

Do not trust tests alone.

Build and run `jackin-preview`.

Use the repository's existing capture tooling under `tools/` and the existing deterministic scenario system.

Capture representative before/after-quality states for at least:

* Capsule normal
* Capsule menu open
* Capsule tab context menu
* Capsule with multiple agent tabs
* redesigned status bar
* command palette with overflow
* Usage with low/medium/high block meters
* Workspace account assignment with multiple accounts
* new-tab account selection with two accounts
* Environments with a large Role registry
* Inspect Changes compact
* Inspect Changes advanced
* dialog with bottom HintBar

Inspect the actual rendered images, not only ANSI/text output.

Iterate when something looks cramped, inconsistent, noisy, or visually weaker than the current approved design.

# Engineering verification

Before completion, run the repository's applicable formatting, build, lint, and test commands.

At minimum ensure equivalent coverage of:

* `cargo fmt --check`
* `cargo test --all-targets`
* `cargo clippy --all-targets -- -D warnings`
* `cargo build --bin jackin-preview`

If existing repository instructions specify stronger commands, run those too.

Fix the code rather than suppressing warnings unnecessarily.

# Documentation

Update `DESIGN.md` when the final implementation establishes reusable rules for:

* canonical branding
* application menu bars
* persistent HintBar behavior
* active tabs
* status bars
* progress meter visual modes
* scroll ownership
* context menus
* scalable scoped configuration

Do not turn `DESIGN.md` into a changelog. Document the resulting system and principles.

Update README/help text only where user-facing instructions actually changed.

# Scope discipline

Do not rewrite unrelated working components.

Do not integrate real Jackin, real Git repositories, real provider APIs, or real TUICR.

`jackin-preview` remains deterministic and self-contained.

Do not add temporary compatibility layers between the old and new UI.

When a new component supersedes old duplicated rendering, migrate the relevant usages and remove the obsolete path.

Do not leave TODO placeholders for requirements in this goal.

# Definition of done

This goal is complete only when all of the following are true:

* Capsule has a distinctive redesigned status bar.
* Jackin branding is canonical and consistent everywhere.
* one context-aware HintBar remains pinned to the bottom.
* agent tabs have Rename/Close context menus.
* selected tabs use background + green underline and no selected left border.
* progress/Usage meters have semantic low/medium/high coloring and both line/block visual modes.
* Inspect Changes has compact and advanced views with realistic diff previews.
* Command Palette wheel/trackpad scrolling actually works.
* all scrollable surfaces have been audited and corrected.
* the useless Capsule identity sentence is gone.
* Capsule has a real menu bar above the agent tabs.
* unconfigured coding agents are absent from new-tab selection.
* accounts are globally registered and then activated/deactivated per Workspace.
* a Workspace/container can have multiple active accounts simultaneously.
* Workspace may disable inherited global defaults.
* session creation resolves zero/one/multiple account cases correctly.
* Environments does not explode into hundreds of empty Role sections.
* tests cover the new behavioral invariants.
* visual captures have been reviewed.
* formatting/lint/tests/build pass.
* `DESIGN.md` reflects the final reusable design rules.

Do not report completion merely because every checkbox has corresponding code.

Use the finished running application as the final judge.

The goal is to make this round of Jackin Preview feel substantially more polished than the current version while preserving everything that already made the current design successful.

