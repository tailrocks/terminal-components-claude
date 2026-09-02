/goal Continue directly from the completed Junie-inspired Ratatui component/design-system prototype.

The existing design system is APPROVED.

I love the current components exactly as they are.

DO NOT redesign them.
DO NOT reinterpret their visual language.
DO NOT change their palette, spacing, borders, hierarchy, focus language, hover treatment, selection treatment, editing treatment, scrolling behavior, or overall appearance.

Treat the current component showcase as the canonical visual specification.

Your mission now is to use those approved reusable components to design and implement a world-class terminal version of the CORE TablePro experience.

Reference:

https://tablepro.app

Source:

https://github.com/TableProApp/TablePro

Documentation:

https://docs.tablepro.app

The target is:

"If TablePro's core database workflow had originally been designed as a premium terminal application using our approved Junie-inspired design system, what would it look and feel like?"

BUILD THAT EXPERIENCE.

This phase is intentionally LIMITED to the following product areas:

1. Connection Experience
2. Database / Schema Explorer
3. SQL Editor
4. Autocomplete
5. Data Grid
6. Filtering & Sorting
7. Result Sets & Tabs
8. Table Structure
9. Query History
10. Quick Switcher
11. EXPLAIN / Query Plan
12. Safe Mode & Database Safety

Do not broaden scope into unrelated TablePro functionality.

---

# 1. PRESERVE THE APPROVED DESIGN

The current design already looks perfect.

Preserve:

- colors
- surfaces
- Junie-inspired green accent
- black / near-black hierarchy
- whites and grays
- spacing
- padding
- borders
- typography hierarchy
- hover
- focus
- selection
- editing
- disabled state
- error state
- scrollbars
- mouse interaction
- keyboard interaction
- visual density

Architectural refactoring is allowed.

Visual regression is not.

If the current prototype needs to be extracted into reusable components, do that without changing how the approved showcase looks.

The original component showcase must continue to work and remain the visual regression baseline.

---

# 2. MAKE THE COMPONENTS REUSABLE

Use the approved prototype as the foundation of a reusable Ratatui design system.

Separate:

DESIGN SYSTEM
- semantic design tokens
- reusable components
- focus handling
- mouse hit testing
- scrolling
- overlays
- forms
- tables
- editable controls
- interaction-state styling

TABLEPRO APPLICATION
- connections
- databases
- schemas
- tables
- SQL
- results
- filtering
- history
- execution plans
- safety state

Do not put TablePro-specific knowledge into generic components.

When TablePro requires a missing primitive, extend the component library.

Any new generic component must:

- follow the existing visual language exactly
- use existing semantic tokens
- support keyboard interaction
- support mouse interaction where appropriate
- support focus
- support hover
- support selection where appropriate
- support disabled/error/loading states where meaningful
- be added to the original component showcase

---

# 3. RESEARCH TABLEPRO DEEPLY FIRST

Before implementing these screens, deeply research the CURRENT TablePro application.

Use multiple subagents in parallel.

Study:

- official website
- documentation
- current source code
- screenshots
- navigation
- commands
- menus
- keyboard shortcuts
- data-grid behavior
- SQL editor behavior
- autocomplete behavior
- connection management
- schema navigation
- result tabs
- table structure
- query history
- quick switcher
- EXPLAIN
- Safe Mode
- dangerous-query handling

Do not design based on generic assumptions about database clients.

Understand how TablePro itself works.

Then translate those workflows into the best TERMINAL-NATIVE experience.

Do not literally reproduce the macOS layout using boxes.

---

# 4. OVERALL TABLEPRO INFORMATION ARCHITECTURE

Build a coherent application.

Do not create twelve disconnected showcase screens.

The screens and overlays must form one navigable application.

The core experience should revolve around a database workbench.

A likely conceptual structure is:

CONNECTION
    ↓
DATABASE / SCHEMA
    ↓
OBJECT
    ↓
SQL / TABLE DATA
    ↓
RESULTS
    ↓
INSPECT / EDIT / EXPLAIN

Determine the final layout through research and experimentation.

A user must always understand:

- which connection is active
- which database is active
- which schema is active
- which object is open
- which tab is active
- which pane owns keyboard focus
- where the mouse is hovering
- whether a query is running
- whether data is filtered
- whether data is sorted
- whether there are unsaved changes
- what Safe Mode applies to the current connection

Avoid excessive permanent panels.

Use progressive disclosure.

---

# 5. CONNECTION EXPERIENCE

Build a polished connection-selection and connection-management screen.

Research TablePro's current connection model.

Represent relevant concepts such as:

- saved connections
- database type
- connection name
- host
- port
- database
- username
- connection state
- tags/groups where supported
- environment identity
- SSL
- SSH tunnel
- Safe Mode
- recent connections
- search
- test connection
- connecting
- connected
- authentication failure
- network failure
- reconnecting

Use realistic connections such as:

Local PostgreSQL
Development
Analytics
Production

Connection identity must remain visible enough that a user cannot easily confuse Production with Development.

Do not accomplish this by painting the entire application red.

Use the restrained design language already established.

Connection creation/editing should use progressive disclosure:

BASIC
- name
- engine
- host
- port
- database
- username

ADVANCED
- SSL
- SSH
- additional parameters
- safety level

Avoid presenting forty fields at once.

---

# 6. DATABASE / SCHEMA EXPLORER

Create a first-class database object explorer.

It should support a hierarchy such as:

Connection
└── Database
    └── Schema
        ├── Tables
        ├── Views
        ├── Functions
        └── Other supported objects

For tables, allow drilling into concepts such as:

Table
├── Columns
├── Indexes
├── Keys
├── Constraints
└── Triggers

where supported.

Interaction must include:

- expand
- collapse
- keyboard navigation
- mouse hover
- selection
- current object
- search/filter
- refresh
- loading
- error
- empty state
- long names
- huge schemas
- scrolling

Do not create a noisy ASCII tree.

Use the approved tree component and visual language.

Clearly distinguish:

HOVERED NODE
FOCUSED TREE
CURRENT NAVIGATION NODE
SELECTED/OPEN OBJECT

These are not necessarily the same thing.

---

# 7. SQL EDITOR

Create a professional terminal SQL editor.

This is one of the primary screens and must receive substantial design attention.

Support the important TablePro SQL workflows including where appropriate:

- syntax highlighting
- line numbers
- cursor
- text selection
- multiline editing
- horizontal scrolling
- vertical scrolling
- current statement
- statement-at-cursor execution
- execute all
- query cancellation
- multiple SQL tabs
- errors
- diagnostics
- query duration
- affected rows
- result association
- find/search
- formatting
- Vim mode if relevant to TablePro
- keyboard-first editing

The SQL editor must not look like a generic textarea with colored keywords.

It should feel like a real professional editor embedded in a database workbench.

A user must immediately understand:

- where the cursor is
- which statement will execute
- whether a query is currently running
- whether execution succeeded
- where an error occurred
- which result set belongs to which query

Quiet success.

Highly legible failure.

---

# 8. AUTOCOMPLETE

Implement a first-class autocomplete experience.

Research TablePro's actual autocomplete behavior.

Support relevant suggestions such as:

- schemas
- tables
- columns
- aliases
- SQL keywords
- functions
- database-specific items

The autocomplete overlay should support:

- filtering as the user types
- ranking
- highlighted current candidate
- keyboard navigation
- mouse hover
- mouse selection
- scrolling
- supplementary metadata where useful
- completion acceptance
- cancellation

It must feel visually native to the existing component library.

Do not render an unrelated gray rectangle over the editor.

If necessary, create a generic reusable:

CompletionPopup

and add it to the component showcase.

---

# 9. DATA GRID

The Data Grid is a critical component.

It must support dense professional data while still looking elegant.

Design states for:

- headers
- row identity
- current cell
- selected cell
- selected row
- hovered cell
- focused grid
- sorted column
- filtered column
- edited cell
- dirty cell
- dirty row
- inserted row
- deleted row
- validation error
- NULL
- boolean
- numeric
- date/time
- JSON
- long text
- truncated values
- empty values
- read-only values
- primary keys
- loading
- empty result
- large result set

Preserve the critical interaction rule:

HOVER != FOCUS != CURRENT CELL != SELECTION != EDITING != DIRTY != ERROR

Do not solve this by assigning a different bright color to every state.

Use layered styling, typography, subtle surfaces, glyphs, borders, and semantic color sparingly.

---

# 10. DATA GRID NAVIGATION

The grid must feel excellent with keyboard and mouse.

Support:

- arrows
- Page Up / Page Down
- Home / End where appropriate
- horizontal scrolling
- vertical scrolling
- selecting cells
- selecting rows
- entering edit mode
- exiting edit mode
- mouse click
- mouse hover
- mouse wheel
- scrollbar interaction where appropriate

Wide tables are expected.

Do not squeeze all columns into the viewport.

Provide excellent horizontal-navigation behavior.

Users should always understand:

- current column
- current row
- scroll position
- whether additional columns exist off-screen

Render only what is visible where practical.

---

# 11. FILTERING & SORTING

Filtering and sorting should be tightly integrated with the Data Grid.

Support:

SORT
- ascending
- descending
- clear sort

FILTER
- add filter
- field
- operator
- value
- apply
- remove
- clear all

Use type-aware operators where useful.

Examples:

TEXT
- equals
- contains
- starts with

NUMBER
- =
- !=
- >
- >=
- <
- <=

NULL
- IS NULL
- IS NOT NULL

Do not permanently occupy substantial screen space with filter controls.

Use progressive disclosure through a compact bar, popup, overlay, or other polished terminal-native interaction.

However, when a dataset is filtered, that must be immediately visible.

When sorted, the active sort column and direction must be immediately visible.

---

# 12. RESULT SETS & TABS

Implement a coherent tab model.

Research TablePro's current tab semantics.

Represent relevant tab types such as:

- table/data tab
- SQL editor tab
- query result
- multiple result sets
- table structure
- EXPLAIN

Tabs must support:

- active state
- inactive state
- hover
- focus where meaningful
- loading
- error
- dirty
- close
- keyboard switching
- mouse switching
- many tabs

Do not allow twenty tabs to compress into unreadable fragments.

Design a terminal-native overflow strategy.

Possibilities include:

- scrollable tab strip
- overflow menu
- quick tab switcher
- compact contextual labels

Determine the best solution experimentally.

---

# 13. TABLE STRUCTURE

Create a polished Table Structure screen.

Research the real TablePro structure view.

Represent important table metadata such as:

COLUMNS
- name
- type
- nullable
- default
- primary key
- generated/identity where relevant

INDEXES
- name
- columns
- uniqueness
- type

KEYS / CONSTRAINTS
- primary keys
- foreign keys
- unique constraints
- checks where supported

OTHER
- relevant DDL/object metadata

Do not create four giant bordered tables merely because the information is tabular.

Determine the best hierarchy.

Table Structure should be easy to inspect quickly.

Where modifications are represented, clearly distinguish:

existing state
proposed state
pending change

Schema modifications can be dangerous.

Use the same safety language as the rest of the application.

---

# 14. QUERY HISTORY

Build a proper Query History experience.

Support relevant metadata:

- query text
- timestamp
- connection
- database
- execution duration
- row count
- success
- failure

Interaction should include:

- scrolling
- selection
- search
- fuzzy/full-text matching
- filtering by connection
- filtering by success/failure where useful
- reopen query
- copy query
- rerun query

Do not present this as a raw log.

Make it a useful navigation and recovery workflow.

Long SQL should remain readable.

Selecting a history entry should allow inspecting its full query without destroying list context.

---

# 15. QUICK SWITCHER

Create an excellent Quick Switcher.

This should be one of the fastest workflows in the application.

Optimize for:

shortcut
→ type
→ arrows
→ Enter

Search across relevant items such as:

- tables
- views
- schemas
- databases
- open tabs
- recent queries
- query history

Group and label results intelligently.

Show enough contextual metadata to distinguish ambiguous names.

For example:

orders
public · Production

orders
archive · Analytics

Do not overload each result with metadata.

Mouse interaction should also work.

If this requires a generic reusable searchable-picker component, create it and add it to the design-system showcase.

---

# 16. EXPLAIN / QUERY PLAN

Create a high-quality EXPLAIN experience.

Research TablePro's current implementation and supported engines.

Terminal presentation should emphasize what terminals are good at:

HIERARCHICAL STRUCTURED DATA.

Use an expandable execution-plan tree.

Represent useful information such as:

- operation/node type
- relation
- estimated cost
- actual time
- rows
- loops
- filters
- sort method
- join type
- indexes
- warnings or expensive operations

Allow:

- expand/collapse
- keyboard navigation
- mouse interaction
- inspection of node details
- scrolling
- opening raw EXPLAIN output

Use visual emphasis sparingly for expensive/problematic nodes.

Do NOT attempt to imitate a graphical flowchart with ugly ASCII arrows.

The execution-plan tree should look like it was designed specifically for the terminal.

---

# 17. SAFE MODE

Research TablePro's CURRENT Safe Mode implementation exactly.

Do not invent safety levels or behavior.

Represent the active connection's current safety level consistently.

The user should be able to understand whether the connection is:

- unrestricted
- protected
- read-only
- or whatever exact current TablePro levels exist

without constantly seeing intrusive warnings.

Safe Mode should appear as compact contextual information.

For example, the active connection identity may expose its safety state.

Exact representation should follow the approved design system.

Safe Mode is connection-specific.

Make this mental model obvious.

---

# 18. DATABASE SAFETY

Dangerous database actions require excellent UX.

Research TablePro's actual safeguards.

Create realistic flows around dangerous operations such as:

- UPDATE without WHERE
- DELETE without WHERE
- DROP TABLE
- DROP DATABASE
- TRUNCATE
- destructive schema changes
- read-only violations

Use severity proportionally.

Do not show dramatic confirmation for harmless queries.

When intervention is required, communicate:

ACTION
What operation is being attempted?

TARGET
Which connection/database/schema/table?

SCOPE
How broad is the effect?

RISK
What could happen?

REVERSIBILITY
Can it be undone?

REQUIRED CONFIRMATION
What must the user do to proceed?

Example conceptually:

Production
acme-prod / public.orders

DELETE without WHERE

This operation may delete every row in `orders`.

[Cancel] [Review Query] [Proceed]

Do not blindly use this exact wording or layout.

Research TablePro and design the best experience.

For highly destructive operations, stronger deliberate confirmation may be appropriate.

Avoid confirmation fatigue.

---

# 19. CONNECT THE SCREENS

These screens must compose into a continuous workflow.

A reviewer should be able to perform something similar to:

CONNECTIONS
↓
select Production PostgreSQL
↓
SCHEMA EXPLORER
↓
public
↓
orders
↓
DATA GRID
↓
sort by created_at
↓
add status = 'pending' filter
↓
inspect/edit row
↓
open TABLE STRUCTURE
↓
return to data
↓
open SQL EDITOR
↓
type query
↓
AUTOCOMPLETE
↓
execute
↓
RESULT TAB
↓
open EXPLAIN
↓
inspect execution tree
↓
QUICK SWITCHER
↓
jump to customers
↓
QUERY HISTORY
↓
reopen previous query
↓
attempt unsafe query
↓
SAFE MODE / DATABASE SAFETY workflow

This should feel like one application.

Navigation context must persist.

---

# 20. SCREEN COMPOSITION

Do not make every feature a completely separate full-screen page.

Use appropriate combinations of:

- persistent regions
- panes
- overlays
- tabs
- inspectors
- dialogs
- command palette
- contextual panels

For example:

Schema Explorer may coexist with SQL Editor.

Results may appear below/alongside SQL.

EXPLAIN may be a dedicated result tab.

Autocomplete should be an editor overlay.

Quick Switcher should be a temporary global overlay.

Safety confirmation should be modal.

Query History may be a main workspace or overlay depending on the strongest UX.

Determine the final architecture through experimentation.

---

# 21. RESPONSIVE DESIGN

Test approximately:

80x24
100x30
120x40
160x50

The optimal experience should be around modern medium/large terminals, but smaller terminals must remain usable.

Do not simply squeeze panes.

At smaller widths, intelligently:

- collapse schema explorer
- focus one main pane
- move secondary information into overlays
- reduce secondary metadata
- allow pane maximization

At larger sizes:

- expose useful contextual information
- allow editor + results composition
- make good use of width

Never turn wide layouts into empty wasted space.

Never turn narrow layouts into clipped boxes.

---

# 22. REALISTIC DEMO DATABASE

Use a realistic deterministic demo database model.

For example:

acme_prod

schemas:

public
analytics
audit

tables:

customers
organizations
subscriptions
products
orders
order_items
payments
events

Create believable columns and values.

Include:

- UUIDs
- emails
- timestamps
- currencies
- NULL
- booleans
- JSON
- long text
- enums/statuses
- foreign keys

Use enough data to meaningfully exercise:

- scrolling
- sorting
- filtering
- wide tables
- query results
- autocomplete
- EXPLAIN
- history

Never use placeholder-quality `foo`, `bar`, `item1` data.

---

# 23. DO NOT BUILD DATABASE DRIVERS

This phase is about the complete TUI experience.

Do not spend the task implementing PostgreSQL/MySQL/etc drivers unless the current repository already has those integrations and they are trivial to reuse.

Use deterministic fixtures/state simulation where required.

But interaction must be REAL.

Users must actually be able to:

- navigate
- hover
- focus
- select
- edit
- type SQL
- trigger autocomplete
- execute simulated queries
- sort
- filter
- switch tabs
- navigate history
- use Quick Switcher
- inspect EXPLAIN
- trigger Safe Mode behavior
- encounter safety dialogs
- resize
- scroll

A static screen gallery is not acceptable.

---

# 24. PRIORITIZE ONE EXCELLENT CORE WORKBENCH

Do not implement all twelve areas shallowly.

Build in vertical slices.

## SLICE 1

Connection Experience
+
Schema Explorer
+
SQL Editor
+
Autocomplete
+
Result Tabs
+
Data Grid

Make this exceptional first.

RUN IT.
INTERACT WITH IT.
RENDER IT.
CRITIQUE IT.
FIX IT.

## SLICE 2

Filtering & Sorting
+
Table Structure
+
Query History
+
Quick Switcher

Again:

RUN.
INSPECT.
FIX.

## SLICE 3

EXPLAIN
+
Safe Mode
+
Database Safety

Then perform complete cross-screen integration.

Do not broaden until the previous slice meets the visual quality bar.

---

# 25. SUBAGENTS

Use multiple subagents aggressively.

At minimum delegate:

### TABLEPRO RESEARCH
Deeply map the exact functionality relevant to these twelve areas.

### TABLEPRO SOURCE REVIEW
Inspect source code for actual behavior, state and shortcuts.

### WORKBENCH INFORMATION ARCHITECTURE
Design the terminal-native relationship between explorer/editor/results/grid.

### DATA GRID SPECIALIST
Design navigation, editing, sorting, filtering, wide-data behavior and state language.

### SQL EDITOR SPECIALIST
Design editing, autocomplete, execution, errors and result association.

### SAFE MODE / SAFETY REVIEW
Research exact TablePro safeguards and propose accurate terminal interactions.

### COMPONENT GAP ANALYSIS
Identify reusable primitives missing from the current library.

### TERMINAL UX REVIEW
Inspect keyboard, mouse, focus, scrolling and responsive behavior.

### VISUAL REVIEW
Inspect actual rendered output independently and reject anything generic, noisy or inconsistent with the approved design.

The primary agent owns synthesis and consistency.

---

# 26. CRITICAL STATE RULE

Throughout the entire application:

HOVER
!=
FOCUS
!=
CURRENT
!=
SELECTED
!=
ACTIVE
!=
EDITING
!=
DIRTY
!=
ERROR

Design these as semantic application states.

Do not merely compose `Style` conditionals ad hoc inside render functions.

The visual treatment should remain restrained.

Users must understand state immediately without requiring a rainbow.

---

# 27. KEYBOARD-FIRST

The complete required workflow must work without a mouse.

Create coherent shortcuts for:

- moving between major panes
- explorer navigation
- editor focus
- result focus
- grid navigation
- execute query
- cancel query
- tab switching
- filter
- sort
- query history
- Quick Switcher
- EXPLAIN navigation
- dialogs
- confirmation
- cancellation

Research existing TablePro shortcuts first.

Preserve familiar semantics where appropriate.

Avoid terminal-level shortcut conflicts.

Provide contextual shortcut hints rather than a permanent shortcut manual.

---

# 28. MOUSE-EXCELLENT

Mouse-capable terminals must provide:

- hover
- click
- tab activation
- tree navigation
- cell navigation
- scrollbar/wheel scrolling
- filter interactions
- popup interaction
- Quick Switcher selection
- dialog buttons

Hover must provide useful feedback without becoming visually noisy.

Mouse and keyboard states must remain coherent when switching between input methods.

---

# 29. VISUAL REVIEW LOOP

For every major screen:

1. implement
2. run
3. render/capture
4. inspect actual output
5. test mouse
6. test keyboard
7. resize
8. critique
9. fix
10. repeat

Never judge visual quality purely from Rust code.

The previous phase established the design system through actual visual inspection.

Continue that discipline.

---

# 30. QUALITY BAR

Reject anything that resembles:

- generic Ratatui examples
- Midnight Commander
- htop
- generic database CLI
- ncurses administration UI
- cyberpunk terminal
- dashboard made from boxes

Avoid:

- borders everywhere
- green everywhere
- cramped tables
- unnecessary ASCII decoration
- noisy shortcut bars
- tiny unreadable metadata
- ambiguous focus
- ambiguous current cell
- ambiguous editing
- ambiguous sorting/filtering
- modal overuse

The approved Junie-inspired system remains the quality reference.

---

# 31. VERIFY HARD CASES

Test and visually inspect:

CONNECTIONS
- no connections
- connecting
- connection failed
- reconnecting
- Production Safe Mode

EXPLORER
- empty schema
- huge schema
- deeply nested objects
- very long names

SQL
- empty editor
- long SQL
- horizontal scrolling
- multiline selection
- autocomplete
- running
- success
- syntax error
- database error
- cancellation

GRID
- empty result
- many rows
- many columns
- horizontal scrolling
- NULL
- long values
- sorted
- filtered
- selected
- hovered
- editing
- dirty
- validation error

TABS
- one tab
- many tabs
- dirty tab
- loading tab
- error tab
- overflow

HISTORY
- empty
- large history
- long queries
- failed query

EXPLAIN
- simple plan
- deeply nested plan
- expensive node
- long metadata

SAFETY
- normal query
- dangerous query
- read-only violation
- destructive DDL
- production target

RESPONSIVENESS
- 80x24
- 100x30
- 120x40
- 160x50

---

# 32. TESTING

Add deterministic tests where practical for:

- focus navigation
- popup focus trapping
- hover hit testing
- explorer expansion
- tab switching
- tab overflow
- autocomplete navigation
- SQL execution state
- query cancellation
- result association
- grid navigation
- vertical scrolling
- horizontal scrolling
- sorting
- filtering
- history navigation
- Quick Switcher ranking
- EXPLAIN expansion
- Safe Mode state
- dangerous-query interception
- confirmation/cancellation
- resize behavior

Tests supplement visual review.

They do not replace it.

---

# 33. FINAL ACCEPTANCE FLOW

Before declaring completion, prove this exact class of end-to-end experience:

1. Launch the application.
2. View Connection Experience.
3. Select Production PostgreSQL.
4. See its Safe Mode state.
5. Connect.
6. Browse Database / Schema Explorer.
7. Open `public.orders`.
8. Browse its Data Grid.
9. Horizontally and vertically navigate.
10. Sort `created_at`.
11. Filter `status = pending`.
12. Inspect Table Structure.
13. Return to Data Grid.
14. Open SQL Editor.
15. Type a realistic query.
16. Trigger Autocomplete.
17. Select a completion.
18. Execute query.
19. Open resulting Result Set.
20. Switch between result tabs.
21. Open EXPLAIN.
22. Navigate Query Plan.
23. Open Query History.
24. Reopen a previous query.
25. Open Quick Switcher.
26. Jump to another table.
27. Return to SQL Editor.
28. Execute a dangerous Production query.
29. Trigger Safe Mode / Database Safety protection.
30. Clearly understand why the action is dangerous.
31. Cancel safely.

This flow must work with keyboard only.

Mouse interaction must also work naturally.

---

# 34. VERIFICATION

Before finishing prove:

- existing component showcase remains visually unchanged
- reusable library boundaries are clean
- all newly required generic components are added to the showcase
- TablePro prototype launches
- all twelve required product areas exist and are connected
- keyboard-only operation works
- mouse hover/click works
- focus is deterministic
- popup focus containment works
- scrolling works
- wide Data Grid works
- sorting works
- filtering works
- result tabs work
- tab overflow works
- Table Structure works
- Query History works
- Quick Switcher works
- autocomplete works
- EXPLAIN works
- Safe Mode is visible and accurate
- dangerous operations are intercepted correctly
- resizing works
- cargo fmt --check passes
- cargo clippy passes
- tests pass
- no normal interaction panics
- actual rendered output has been visually reviewed

---

# 35. DEFINITION OF DONE

This phase is complete only when there is a coherent, runnable, interactive TablePro TUI containing:

1. Connection Experience
2. Database / Schema Explorer
3. SQL Editor
4. Autocomplete
5. Data Grid
6. Filtering & Sorting
7. Result Sets & Tabs
8. Table Structure
9. Query History
10. Quick Switcher
11. EXPLAIN / Query Plan
12. Safe Mode & Database Safety

These must not be twelve unrelated mock screens.

They must form one polished database application.

They must use the approved Junie-inspired component system.

They must demonstrate real interaction.

They must have been run, resized, navigated, clicked, typed into, scrolled, visually reviewed, critiqued, and corrected.

Do not implement unrelated TablePro functionality during this phase unless a tiny supporting piece is absolutely necessary for one of the required workflows.

Do not finish merely because every screen exists.

Finish when the complete core workflow feels coherent and premium.

The quality target remains:

"This does not look like a desktop database client squeezed into a terminal.

This looks like what TablePro would have been if TablePro had originally been designed for a modern terminal."

Preserve what is already perfect.

Extend only what these workflows require.

Build the experience.

Run it.

Inspect it.

Fix it.

Prove it.