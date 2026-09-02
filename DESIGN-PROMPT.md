You are a senior product designer, design-systems engineer, and TUI/terminal UX specialist.

Your goal is to reverse-engineer the **existing TUI design implemented in this repository** and produce the canonical repository-root `DESIGN.md`.

The existing application is the source of truth.

Do not redesign it.

Your job is to discover the design system that already exists implicitly in the implementation and express it explicitly as a high-quality, reusable, machine-readable + human-readable design system.

# Critical requirement: follow the official DESIGN.md specification

`DESIGN.md` MUST conform to the current official **Google Labs / Stitch DESIGN.md open specification**.

Before generating the file:

1. Inspect the latest official `google-labs-code/design.md` specification.
2. Read the current `docs/spec.md`.
3. Inspect the current specification/rules exposed by `@google/design.md`.
4. Prefer the latest official specification over examples, blog posts, third-party interpretations, or older Stitch documentation.
5. Treat the official schema and linter as normative.

Where available, inspect the current specification with:

```bash
npx -y @google/design.md@latest spec --rules
```

If that command is unavailable or broken, read the latest official Google Labs DESIGN.md specification directly from its source repository instead.

The resulting `DESIGN.md` must be valid according to the current official specification.

Validate it before completing the task:

```bash
npx -y @google/design.md@latest lint DESIGN.md
```

Resolve all errors.

Also eliminate avoidable warnings. If a warning is genuinely unavoidable because of terminal-specific semantics, verify that the document represents the design truth correctly rather than distorting the design merely to silence the linter.

# DESIGN.md architecture

Follow the official two-layer model:

1. **YAML front matter** — exact, machine-readable design tokens.
2. **Markdown body** — semantic rationale explaining why and how those tokens and conventions are used.

Tokens are normative.

Prose explains intent, semantics, composition, hierarchy, constraints, and correct usage.

Do not replace exact tokens with vague prose when the implementation provides exact values.

Do not invent exact values when the implementation does not provide them.

# YAML front matter

Extract applicable tokens from the actual TUI implementation using the official schema, including where supported:

```yaml
---
version: alpha
name: ...
description: ...

colors:
  ...

typography:
  ...

rounded:
  ...

spacing:
  ...

components:
  ...
---
```

Use token references such as:

```yaml
"{colors.primary}"
```

where appropriate instead of duplicating literal values.

Use semantic token names based on purpose rather than arbitrary visual names whenever that reflects the implementation.

Examples:

* `primary`
* `secondary`
* `surface`
* `on-surface`
* `muted`
* `accent`
* `success`
* `warning`
* `error`

But derive the actual vocabulary from this application. Do not force these names if the application clearly uses a different semantic model.

# Terminal-specific interpretation

This is a TUI, not a web application.

Never invent web-design properties merely to make `DESIGN.md` look complete.

In particular:

* Do not invent application-controlled font families if the terminal emulator owns the font.
* Do not invent font sizes if the TUI does not control them.
* Do not invent rounded corner radii for character-cell borders.
* Do not invent CSS shadows.
* Do not translate terminal-cell spacing into fictional pixels.
* Do not claim hover behavior exists unless mouse/hover behavior actually exists.
* Do not describe responsive web breakpoints when the implementation actually responds to terminal rows/columns.
* Do not translate ANSI/terminal semantics into web concepts when doing so loses meaning.

The official DESIGN.md specification supports intentional omissions.

If a normative token category genuinely does not apply to this TUI, use the specification's `omitted` mechanism with a concise reason rather than fabricating values.

For example, if appropriate:

```yaml
omitted:
  - section: rounded
    reason: "The interface uses terminal cell border glyphs rather than geometric corner radii."
```

Use this only when the category truly does not apply.

A section's prose may still explain the terminal equivalent even when its machine-readable token category is intentionally omitted.

# Canonical Markdown structure

Use the official DESIGN.md section order exactly:

## Overview

## Colors

## Typography

## Layout

## Elevation & Depth

## Shapes

## Components

## Do's and Don'ts

Do not replace this structure with a custom design-document structure.

Do not create numerous competing `##` sections when the information can live naturally as `###` subsections beneath the canonical sections.

# Reverse-engineer before documenting

Perform a broad repository analysis before writing `DESIGN.md`.

Inspect:

* TUI application entry points
* rendering architecture
* screen/view definitions
* shared widgets
* theme/style modules
* color constants
* style helpers
* layout primitives
* borders
* symbols
* icons
* text modifiers
* focus styles
* selection styles
* tables
* lists
* trees
* editors
* forms
* overlays
* dialogs
* menus
* command palettes / quick switchers
* tabs
* status bars
* headers
* footers
* error presentation
* loading states
* empty states
* keyboard handling
* navigation
* focus management
* responsive terminal-size behavior
* snapshots
* golden tests
* fixtures
* demos/examples
* screenshots or visual verification tooling

Trace important design decisions to their actual implementation.

Do not infer a component's behavior merely from its name.

If the application can be run safely, inspect representative screens visually as well as through source code.

Use parallel subagents where useful to independently investigate:

* colors/theme
* layout
* component vocabulary
* navigation and interaction
* terminal typography
* screen composition
* states
* symbols/borders
* responsive behavior
* visual snapshots/tests

Reconcile their findings against the source before writing the final file.

# ## Overview

Describe the overall visual identity and interaction philosophy.

Capture what actually makes this TUI feel like this TUI:

* density
* hierarchy
* restraint
* information presentation
* keyboard-first philosophy
* discoverability
* progressive disclosure
* contextual controls
* focus
* consistency
* safety
* feedback
* terminal-native character

Explain the design in sufficiently concrete visual language that another agent can recognize whether a new screen belongs to the same product.

Avoid empty adjectives such as:

* modern
* beautiful
* clean
* intuitive
* premium

unless followed by concrete characteristics that explain what creates that quality.

Extract principles from repeated implementation patterns rather than inventing aspirational principles.

# ## Colors

Extract the real palette and semantic color system.

For every important color explain:

* exact value where determinable
* semantic role
* normal usage
* prohibited/rare usage
* relationship to focus/selection/state
* foreground/background pairing

Capture states such as:

* normal
* muted
* focused
* selected
* active
* inactive
* disabled
* informational
* success
* warning
* destructive/error

Prefer semantic role over merely saying what the color looks like.

If the TUI supports multiple themes, document the semantic token model separately from theme-specific mappings.

Preserve terminal palette/ANSI semantics where they matter.

# ## Typography

Document the hierarchy the application creates within terminal typography.

The terminal emulator may control actual font family and font size, so concentrate on what the TUI itself controls:

* bold
* dim
* italic
* underline
* capitalization
* punctuation
* symbols
* spacing
* indentation
* alignment
* foreground/background contrast
* semantic emphasis

Describe the hierarchy for:

* screen titles
* section headings
* labels
* values
* metadata
* secondary information
* hints
* shortcuts
* warnings
* errors
* selected content
* disabled content

Do not invent font tokens that the application cannot control.

# ## Layout

Document the actual spatial system of the TUI.

Extract:

* terminal-cell spacing rhythm
* padding
* gaps
* separators
* major regions
* panel relationships
* alignment
* preferred proportions
* content density
* minimum viable width/height
* terminal dimension thresholds
* scrolling
* clipping
* truncation
* collapsing/hiding behavior
* narrow terminal behavior
* wide terminal behavior

Use machine-readable spacing tokens where they map honestly to the implementation.

Because the DESIGN.md spacing schema permits unitless numbers, use them where appropriate for genuine unitless terminal concepts rather than fabricating CSS dimensions.

Identify recurring screen composition patterns such as:

* sidebar + workspace
* master/detail
* editor + results
* table + contextual controls
* searchable list
* tabbed workspace
* modal workflow
* command palette
* drill-down navigation

Explain when each composition is appropriate.

# ## Elevation & Depth

Explain how this TUI establishes visual layering.

Terminal applications frequently use no physical shadows.

If this application uses a flat model, describe how hierarchy is instead communicated through things such as:

* borders
* contrast
* inverse backgrounds
* overlays
* dimming
* separators
* whitespace
* title treatment
* focus styling

Describe overlays, dialogs, menus, and layered UI precisely.

Never invent shadows.

# ## Shapes

Translate the application's terminal-native shape language accurately.

Document:

* border glyph style
* separator style
* corners
* boxes
* panel framing
* active/inactive borders
* dividers
* selected-row markers
* disclosure indicators
* tree connectors
* other recurring symbols

If CSS-style `rounded` tokens do not represent the TUI truthfully, intentionally omit them from YAML and explain the terminal-native shape language here.

# ## Components

This section should capture the reusable vocabulary of the actual application.

Identify canonical components such as applicable:

* panels
* tabs
* lists
* trees
* tables
* data grids
* text inputs
* search
* selectors
* command palette
* quick switcher
* menus
* editor
* status bar
* toolbar
* dialog
* confirmation dialog
* notifications
* badges
* keybinding hints
* empty states
* loading indicators
* error presentation

For every significant component describe:

### Purpose

What problem it solves.

### Anatomy

Its visual structure.

### Styling

How it uses the design system.

### States

Normal, focused, selected, disabled, loading, error, etc.

### Interaction

How the user operates it.

### Keyboard behavior

Keys and conventions that are part of the component.

### Sizing/layout

How it behaves with available terminal space.

### Composition

What it commonly appears with.

### Usage

When to use it.

### Avoid

When it should not be introduced.

Use machine-readable `components` tokens only where they map cleanly to the official DESIGN.md component-token schema.

Put richer TUI-specific component semantics in prose rather than abusing the token schema.

# Interaction grammar

Within the relevant canonical sections—primarily `Components` and `Do's and Don'ts`—extract the application's shared interaction grammar.

Document:

* global navigation
* local navigation
* focus transitions
* Enter behavior
* Escape behavior
* Tab / Shift-Tab
* arrow keys
* Vim-style navigation where present
* opening/closing overlays
* cancel behavior
* selection
* multi-selection
* search
* filtering
* sorting
* scrolling
* resizing
* editing
* copying
* command execution
* destructive confirmations

Clearly distinguish:

* global shortcuts
* screen-specific shortcuts
* contextual actions

Future screens should reuse this interaction grammar rather than inventing new controls for equivalent actions.

# UI state grammar

Document shared treatment of:

* initial state
* loading
* loaded data
* empty data
* no search matches
* partial data
* recoverable error
* fatal error
* disconnected state
* disabled action
* destructive operation
* background operation
* completion/success feedback

Extract reusable rules rather than creating a screen-by-screen manual.

# ## Do's and Don'ts

Turn the discovered design system into concrete implementation guardrails.

This section is especially important for future coding agents.

Include precise application-specific rules such as:

* Do inspect existing components before introducing a new one.
* Do reuse existing primitives and semantic tokens.
* Do preserve established navigation conventions.
* Do preserve established focus behavior.
* Do maintain the existing information-density strategy.
* Do compose screens from existing patterns.
* Do design loading, empty, failure, focus, and narrow-terminal states.
* Do preserve terminal readability and contrast.
* Do verify visual changes against representative existing screens.
* Do update `DESIGN.md` when intentionally introducing a genuinely reusable new design convention.

And corresponding prohibitions:

* Don't introduce one-off colors.
* Don't add arbitrary borders.
* Don't introduce unnecessary visual chrome.
* Don't create a new keyboard convention when one already exists.
* Don't use decorative symbols inconsistently.
* Don't invent one-off spacing.
* Don't duplicate an existing component.
* Don't optimize one screen at the expense of product-wide consistency.
* Don't introduce web/mobile conventions that conflict with terminal-native behavior.

Make these rules specific to what is discovered in this repository rather than leaving them generic.

Add a `### Agent implementation guardrails` subsection here containing the strongest rules future coding agents should follow when extending the TUI.

# Canonical vs. accidental implementation

Do not assume every implementation detail is intentional design.

Identify:

1. repeated/canonical patterns
2. deliberate exceptions
3. isolated inconsistencies
4. legacy/outlier behavior

`DESIGN.md` should describe the coherent canonical system.

Do not elevate an obvious isolated inconsistency into a design principle merely because it exists in source code.

When uncertain, prefer patterns demonstrated consistently across multiple mature screens/components.

# Writing quality

The document must work equally well for humans and AI coding agents.

Use:

* precise semantic terminology
* exact values where known
* concise rules
* functional explanations
* explicit constraints
* reusable patterns
* consistent naming

Prefer:

**what it is → why it exists → when to use it → how it behaves**

Avoid:

* source-code walkthroughs
* huge lists of file paths
* implementation trivia
* vague aesthetic adjectives
* duplicated guidance
* aspirational redesign proposals
* invented design tokens

The document should describe the **design system**, not merely catalogue the code.

# Source of truth hierarchy

When evidence conflicts, use:

1. Current rendered/observable application behavior
2. Current shared design/theme/component primitives
3. Repeated patterns across mature screens
4. Tests/snapshots/golden fixtures
5. Individual screen implementations
6. Existing prose/documentation

Do not allow stale prose to override current implementation.

# Validation

Before declaring completion:

1. Re-read the current official DESIGN.md specification.
2. Validate YAML front matter against its schema.
3. Check all token references.
4. Check canonical section ordering.
5. Ensure intentionally inapplicable token categories use the official omission mechanism where appropriate.
6. Check that colors/components correspond to source evidence.
7. Check that no fictional web-only properties were introduced.
8. Run:

```bash
npx -y @google/design.md@latest lint DESIGN.md
```

9. Fix all errors.
10. Resolve all avoidable warnings.
11. Re-read `DESIGN.md` as if you were an agent about to build a completely new screen.

Ask:

> Could I reproduce this application's existing visual language, layout grammar, component behavior, interaction model, and terminal-specific character using this document without rediscovering the whole design system from source?

If not, improve the document.

# Constraints

Do NOT redesign the application.

Do NOT modify the TUI merely to make the documentation cleaner.

Do NOT create a proposed future design system.

Do NOT manufacture tokens absent from the implementation.

Do NOT convert terminal concepts into fictional CSS concepts.

Do NOT document every implementation quirk as canonical.

Do NOT write a source-code architecture document.

Do NOT create another competing design specification.

`DESIGN.md` is the canonical output.

# Definition of Done

The goal is complete only when:

* repository-root `DESIGN.md` exists
* it follows the latest official Google Labs DESIGN.md specification
* YAML front matter contains accurate machine-readable tokens where applicable
* the canonical Markdown section order is respected
* terminal-inapplicable token categories are handled honestly
* visual identity is accurately captured
* semantic colors are documented
* terminal typography hierarchy is documented
* layout and spacing grammar are documented
* depth/layering behavior is documented
* terminal shape/border language is documented
* reusable components and their states are documented
* interaction conventions are captured
* important UI states are captured
* canonical patterns are distinguished from accidental inconsistencies
* future-agent design guardrails are explicit
* all claims are grounded in the existing implementation
* the official DESIGN.md linter reports no errors
* avoidable linter warnings have been resolved
* another coding agent could use this file as the authoritative design contract for implementing new TUI screens that look and behave indistinguishably from the existing product


