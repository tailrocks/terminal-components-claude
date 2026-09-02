/goal You are a principal product designer, terminal UX specialist, and senior Rust/Ratatui engineer.

Your mission is to design and implement a world-class interactive TUI design-system prototype inspired by the visual language of:

https://junie.jetbrains.com

This is not a generic Ratatui demo and not another low-quality developer TUI.

The target quality bar is: if the Junie website had been designed natively for a modern terminal rather than a browser, what would it look and feel like?

## FIRST: RESEARCH THE REFERENCE

Before designing anything, thoroughly inspect the live Junie website and its available assets/styles.

Reverse-engineer its visual language:
- exact or near-exact color palette
- background and surface hierarchy
- green accent usage
- black / near-black surfaces
- white and gray typography hierarchy
- border treatment
- spacing rhythm
- visual density
- contrast
- emphasis
- selected/active states
- geometric/decorative patterns
- restrained use of color
- visual grouping
- information hierarchy
- overall personality

Inspect actual CSS/design tokens/computed styles where possible. Do not invent an arbitrary "green dark theme" and call it Junie-inspired.

Identify the principles that make the reference feel polished, then translate those principles into terminal-native design.

Do NOT blindly reproduce webpage layouts or JetBrains branding. Borrow the design language and experience, not logos or marketing content.

## OBJECTIVE

Build a runnable Rust + Ratatui "design system laboratory" that demonstrates what a future reusable component library should look and behave like.

The goal of this task is NOT to prematurely build and stabilize a complete public component-library API.

The goal is to create the interactive reference implementation/prototype from which that library can later be extracted.

The application itself is the design specification.

It must be polished enough that we can run it and immediately judge:
"Yes, this is the visual and interaction language we want for our future Ratatui applications."

Do not stop at mockups, Markdown descriptions, plans, or static screenshots.

BUILD THE RUNNABLE EXPERIENCE.

## DESIGN PHILOSOPHY

Translate Junie's visual character into a terminal-native medium.

Prefer:
- near-black backgrounds
- layered dark surfaces
- bright Junie-like green used deliberately as the primary accent
- crisp white primary text
- restrained gray secondary/muted text
- subtle borders
- generous negative space
- extremely clear hierarchy
- minimal chrome
- intentional alignment
- consistency
- high information legibility
- strong but tasteful focus states

Green is an accent, not paint to spread over the entire interface.

Do not create a "cyberpunk terminal."
Do not create an ncurses-looking admin panel.
Do not create a dashboard full of boxes.
Do not use Unicode decoration merely because it is available.
Do not compensate for poor hierarchy with excessive borders.
Do not make every component visually loud.

Simplicity, hierarchy, spacing, interaction feedback, and restraint are more important than ornament.

Use terminal-native techniques intelligently:
- truecolor
- foreground/background contrast
- text attributes
- Unicode where it improves the design
- rounded/light/heavy border variants where appropriate
- spacing
- padding
- alignment
- subtle surface changes
- glyphs/icons only when useful
- terminal mouse events
- keyboard focus
- cursor placement
- scrolling
- animation/ticks where appropriate

Typography cannot be controlled like the web, so reproduce hierarchy through weight, color, spacing, casing, alignment, and composition rather than pretending terminal fonts are web fonts.

## INTERACTION MODEL

This must feel like a real interactive application, not a screenshot gallery.

Explicitly distinguish:

- default
- hovered
- focused
- active/pressed
- selected
- disabled
- error
- editing

Hover and focus are NOT the same state.

Mouse-capable terminals must provide real hover feedback.

Keyboard-only operation must remain first-class.

Every interactive control must communicate:
- what can be interacted with
- what currently has keyboard focus
- what the mouse is currently hovering
- what is selected
- what is being edited
- where keyboard input will go
- where the cursor is
- what is disabled
- what has an error

Never rely exclusively on color to communicate important state.

Mouse interaction should include where appropriate:
- hover
- click
- selection
- scrollbar interaction
- wheel scrolling

Keyboard interaction should include:
- Tab / Shift+Tab focus traversal
- arrows where semantically appropriate
- Enter/Space activation
- Esc cancellation/back
- intuitive editing/navigation shortcuts

Provide concise contextual key hints where useful without turning the screen into a shortcut manual.

## COMPONENT SHOWCASE

Implement polished demonstrations for at least:

1. BUTTONS
   - default
   - hover
   - focus
   - pressed/active
   - selected/toggle
   - disabled
   - primary
   - secondary/subtle where appropriate

2. FORMS
   - labels
   - sections
   - validation
   - help text
   - required/optional fields
   - keyboard navigation
   - submission states

3. SINGLE-LINE INPUTS
   - default
   - hover
   - focused
   - disabled
   - error
   - populated
   - placeholder
   - cursor
   - selection where feasible

4. TEXT AREAS
   - default
   - hover
   - focused
   - disabled
   - error
   - cursor
   - multiline scrolling

5. PANELS
   - titled
   - untitled
   - focused/unfocused
   - scrollable content
   - nested content without visual clutter

6. SIDEBARS
   - navigation
   - current item
   - hover
   - focus
   - groups/sections
   - collapsed behavior if it improves the design

7. POPUPS / DIALOGS
   - modal visual hierarchy
   - background context
   - internal focus
   - selected action
   - keyboard + mouse interaction
   - confirmation and destructive examples where useful

8. TABLES
   - headers
   - selected row
   - hovered row/cell
   - keyboard navigation
   - column sorting
   - ascending/descending indicator
   - horizontal/vertical overflow when required
   - empty state

9. EDITABLE TABLES
   - cell focus
   - edit mode
   - cursor
   - commit
   - cancel
   - validation/error
   - visibly distinguish navigation from editing

10. LISTS
    - hover
    - focus
    - selected item
    - disabled item
    - scrolling
    - empty state
    - multiple selection if it improves the component model

11. TREES
    - parent/child hierarchy
    - expand/collapse
    - focus
    - hover
    - selection
    - scrolling
    - clear nesting without excessive ASCII noise

12. PROGRESS / LOADING
    - determinate progress bar
    - indeterminate loading
    - compact spinner/activity state
    - completed/error variants where appropriate

13. SCROLLING
    Treat scrolling as a first-class component/behavior.

    Demonstrate:
    - overflowing panels
    - long lists
    - long tables
    - text areas
    - mouse wheel
    - keyboard scrolling
    - visible scroll position
    - clear indication that additional content exists

Design a scrollbar that belongs to the visual system rather than using an arbitrary default-looking scrollbar.

## DEMO APPLICATION

Create a coherent application around these components rather than rendering unrelated widgets on one enormous page.

A strong structure could be:

- compact application/header area
- component navigation sidebar
- main demonstration canvas
- optional contextual/state information where genuinely useful
- contextual footer/help area

Allow navigating between component categories.

Each component page should expose meaningful interactive states so the user can experience the design rather than merely look at examples.

Include one or more composed screens showing several components working together, because isolated components can look good while producing a poor real application.

The demo should make it easy to evaluate:
- visual rhythm
- state consistency
- keyboard flow
- mouse interaction
- nested focus
- scrolling
- density
- hierarchy

## RESPONSIVE TERMINAL DESIGN

Do not assume one terminal size.

Design and test representative layouts such as:
- approximately 80x24
- approximately 120x40
- approximately 160x50

The experience should adapt intelligently rather than panic, overlap, truncate critical controls, or become unusable.

Define a sensible minimum supported size and show a polished reduced-size state when below it.

Truecolor should be the primary visual target.

Gracefully degrade when capabilities are unavailable; do not compromise the primary high-quality experience simply to target the lowest-common-denominator terminal.

## ARCHITECTURE

Although this task is a prototype, do not create disposable spaghetti code.

Separate at least conceptually:
- theme/design tokens
- interaction state
- component state
- rendering
- layout
- input/event routing
- mouse hit testing
- focus management
- scrolling
- demo/example data

Centralize design tokens.

Do not scatter raw RGB values and style decisions throughout rendering code.

Create semantic tokens such as concepts equivalent to:
- canvas
- surface
- elevated surface
- border subtle
- border strong
- text primary
- text secondary
- text muted
- accent
- accent hover
- accent selected
- focus
- disabled
- error
- warning/success if required

Derive their actual values from the Junie visual research.

Design the prototype so successful components could later be extracted into a reusable Ratatui library without rewriting the entire interaction model.

Use current stable Rust and modern Ratatui practices.

Avoid unnecessary dependencies.

## SUBAGENTS

Use multiple subagents aggressively and in parallel before converging on the implementation.

At minimum delegate independent work for:

1. VISUAL RESEARCH
   Analyze Junie's current website, CSS, assets, colors, spacing, hierarchy and interaction patterns.

2. TERMINAL UX RESEARCH
   Determine the strongest modern Ratatui patterns for focus, mouse hover/hit-testing, scrolling, editable controls, tables, trees, event routing and responsive layout.

3. COMPONENT/STATE DESIGN
   Build a complete interaction-state matrix and identify consistency problems before implementation.

4. VISUAL CRITIQUE
   Independently review rendered output/screenshots and identify anything that feels generic, cluttered, inconsistent, visually weak, or unlike the intended design language.

Parallelize research and independent review. The primary agent owns synthesis and final consistency.

Do not blindly combine conflicting subagent recommendations.

## ITERATIVE VISUAL REVIEW

Treat this as visual engineering.

Do not implement once and declare success.

Use this loop:

1. Research reference.
2. Establish design tokens.
3. Build first coherent screen.
4. Run the application.
5. Capture/inspect actual rendered terminal output where tooling allows.
6. Critique hierarchy, spacing, alignment, color, focus and density.
7. Correct weaknesses.
8. Add remaining components.
9. Run and inspect again.
10. Perform independent final visual/UX review.
11. Fix findings.
12. Verify the complete interactive experience.

Screenshots/rendered output are evidence, not the implementation itself.

Never judge visual quality solely by reading Rust source code.

## QUALITY BAR

Reject an implementation if it merely:
- compiles
- contains all requested widgets
- uses black and green
- looks like a normal Ratatui example
- surrounds everything with borders
- has inconsistent spacing
- lacks real hover/focus behavior
- confuses selected and focused states
- only works with a keyboard
- only works with a mouse
- becomes chaotic at smaller sizes
- contains decorative noise
- has no coherent design system

The result must feel intentionally designed.

A user should be able to understand the current interaction state within a fraction of a second.

The interface should have the same qualities that make the Junie reference compelling:
clarity, confidence, restraint, contrast, visual personality, and precision.

## VERIFICATION

Before finishing, prove at minimum:

- `cargo fmt --check`
- `cargo clippy` with warnings treated seriously
- tests pass
- application launches successfully
- keyboard navigation works
- mouse hover/click works in supported terminals
- focus traversal is deterministic
- scrolling works
- table sorting works both directions
- table editing works
- input/text-area editing works
- popup focus is correctly contained
- disabled components cannot activate
- error states are distinguishable
- resize handling works
- representative terminal sizes render coherently
- no panics during normal interaction
- component state visuals are consistent

Where practical, add tests for deterministic state transitions, focus navigation, hit-testing, sorting, scrolling and editing behavior.

## FINAL DELIVERABLE

Finish with a runnable, polished Ratatui design-system showcase/prototype.

Also leave concise documentation covering:
- extracted visual principles from Junie
- palette/design tokens
- component state model
- interaction conventions
- keyboard/mouse behavior
- how to run the showcase
- how this prototype should evolve into the eventual reusable component library

Do not spend the task producing an elaborate design document instead of the product.

The runnable TUI is the primary deliverable.

Do not ask the human to make ordinary implementation/design decisions that can be resolved through research, experimentation, comparison, or independent subagent review. When uncertain, investigate alternatives, choose the strongest solution, implement it, inspect the result, and continue.

The task is complete only when the experience has been implemented, run, interacted with, visually reviewed, corrected, and demonstrated — not merely when the code compiles.