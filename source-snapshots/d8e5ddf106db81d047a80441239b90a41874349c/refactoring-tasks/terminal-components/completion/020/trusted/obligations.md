# TASK-020 trusted component obligations

This planner-owned input is frozen outside executor authority. Source IDs use namespaces; HIST:A01 and ARCH:A01 differ. The host resolves the exact catalog and scenario digests before execution. This document contains no candidate-selected expected pixels.

## Owned behavior and proof

Restore List current-row completed click/range selection, NavList Chose versus EnterContent and compact sections, Tree lazy branch/filter/replacement semantics, Steps skipped inspectability/frontier, Chip checked/close/add/overflow and Tabs active/hover/cursor/close/reorder geometry. Apply shared fades with current row protection.

Keep borrowed models and stable identities; Tree cache stores derived structure only and filtering projects matches plus ancestors without changing saved expansion. AddRequested has no fake item key; domain shell alone focuses content after EnterContent.

Run duplicate labels, external insert/delete/reorder, removed current/selected target, lazy update during input, filter clear restoration, disabled branch, full/narrow strip and first/last boundary. Prove 100k visible-only callbacks and incremental subtree/frontier work with saturation/revision invalidation. Reject pointer hover impersonating activation or stale index retargeting.

## Fixed re-audit contract — reusable chip data, lead and strip policies

ADJ-11 binds the actual oracle ChipBar uses in Showcase pages/chips.rs:47–113 and TablePro tabs.rs:406/528, workbench.rs:747–751/1075. Extend the existing borrowed ChipBar, not an app-local lead/clear renderer or alternate chip engine. Exact additive builders are `lead(self, label: &'a str) -> Self`, `checked(self, get: &'a dyn Fn(&T) -> bool) -> Self`, `removable(self, get: &'a dyn Fn(&T) -> bool) -> Self`, `error(self, get: &'a dyn Fn(&T) -> bool) -> Self`, `scrollable(self, enabled: bool) -> Self` and `reserve_add(self, enabled: bool) -> Self`. Defaults preserve existing API behavior: no lead, cached checked state, global closable, no per-item error, scrolling enabled and add reservation enabled. Providers are borrowed props, never stored/cloned in durable state or given a static/Clone/Display requirement. The same configured props feed both phases.

A checked provider is caller-controlled: it maps oracle chip.enabled (which means checked/toggled presentation, NOT interaction disabled), Space emits Toggled(key) without writing a second cached checked value, and completed body click emits Activated(key). Existing unconfigured Multi behavior remains supported. Per-item removable overrides global closable for exact close geometry/keyboard admission; error affects source-defined paint without disabling interaction. Readonly/configured/inherited-disabled guards still block mutation requests. The lead is a part of the same component, emits keyless ChipBarAction::LeadRequested, is clickable without another focus/cursor stop, and has no fabricated ItemKey. Add keyless ClearRequested for the clear-all command; keep AddRequested keyless. Explicit source preset is the ordinary builder composition with checked/removable/error providers, lead, scrollable(false), reserve_add(false), and the configured add label.

Bind source key semantics in the shared table: Left/Right/h/l move bounded cursor over items and optional add stop; Enter activates a chip or add; Space toggles only a chip; Delete/Backspace/plain x closes only a removable chip; + requests add and X requests clear-all. Modified character chords with Ctrl or Alt never perform the plain action; Shift-only uppercase X remains clear as in oracle chips.rs:89–135. Do not broaden a plain-char guard to Enter/Delete: test modified Enter/Delete against the exact source behavior. The no-stops early return remains authoritative, including +/X when both items and add are absent. Direct click on a live lead requests Lead even if no chip cursor exists. Runtime never exposes nonremovable/overflow-hidden close parts. Update conformance.rs:1678 exhaustive matches to preserve key extraction for existing item actions and return no key for all three keyless requests; TASK-018 precedes this shared test edit. The existing exhaustive Showcase match at apps/showcase/src/pages/chips.rs:269–274 also needs compile-only arms for the new variants; keep its original four mappings unchanged, use explicit request-status placeholders only until TASK-037 performs real domain migration, and retain that app's diagnostic mismatches under the immutable stage map. This is not application parity closure.

In clipped nonreserved mode, source geometry is exact: lead text is one leading space + label + one trailing space, then a one-cell gap; chip width is label width+3+(removable ? 2 : 0), its label starts x+1, close × is x+2+label width, and the next chip starts after one gap. A chip that does not fit wholly produces the clipped ellipsis and ends the strip; no pre-reserved trailing add column, partial chip, fabricated hidden hit or later add painting. After all chips, add width is label width+2 and it paints only if it fits. Lead/close hover styles, unchecked faint text and error precedence follow oracle chips.rs::render. In clipped nonreserved mode, the first-chip-overflow early return retains already painted visible chip/lead hits but omits the group focus-ring registration, exactly as the source; a subsequent fitting frame restores that single group registration. Generic default scrolling/reserved-add mode retains its ordinary one focus stop. Empty/tiny containment remains an architectural probe, not permission to invent an oracle baseline where raw source lead writes exceed the allocation. No offscreen/overflow-hidden hits are admitted. Preserve generic scroll/reserved-add mode as a separately tested supported policy, with the same writer/identity engine.

Compile-positive borrowed-model/provider fixtures and exact full/narrow cells cover each provider true/false, checked+error, removable versus nonremovable siblings, body/close/lead/add pointer hit precedence, clamped cursor, empty items, modified chords, external reorder/removal and duplicate labels. Reject a keyless action carrying a sentinel index, separate app lead chrome, cached provider values, internal mutation of caller-controlled checked data, wrong add reservation or overflow registration. Showcase and TablePro owners consume LeadRequested by toggling their domain match_all and rebuilding borrowed lead, ClearRequested by clearing their domain list, and existing keyed events by live key lookup; Holla Activity uses removable=false for its source chips. ChipBar invokes the borrowed pure property callbacks but performs no external provider I/O or domain operation.

## Fixed ADJ-16 contract — typed Steps capabilities and Rail presentation

Add `Steps::passive(id)` and `Steps::inspectable(id)` returning the same borrowed Steps type. Internally use one typed four-way capability, not independent interactive/navigable/activate booleans. Preserve new's existing ClickOnly press/click Moved and DoubleClick Activated, and navigable's existing extended keyboard/single-click/Enter activation. Passive registers no row hits/focus/hover and emits no row actions; wheel/scrollbar remain live. Inspectable registers focus and source movement, emits existing Moved even for admitted clamped repeats, but never Activated on Enter/Space/DoubleClick; only completed click, not Press, moves cursor. is_navigable is true for navigable/inspectable. Disabled denies all input, unlike passive. Source inspectable guards: Up/k/Down/j/Home/g require plain (modifiers minus SHIFT empty); End/G source arm has no such guard; Page keys do nothing. Preserve exact source callback/flow and outside-release behavior.

Add public `StepsPresentation::{Compact,Rail}`; consuming `presentation(self, value: StepsPresentation) -> Self`, `numbered(self, value: bool) -> Self`, `meta(self, get: &'a dyn Fn(&T) -> Option<&str>) -> Self` and `variant(self, value: Variant) -> Self`. Keep all borrowed props in every type-changing builder. Internal Option<bool> and Option<Variant> distinguish explicit values from presentation defaults: Compact defaults unnumbered/DEFAULT; Rail numbered/RAIL; explicit numbered/variant wins regardless of call order. Some(meta) overrides all lifecycle defaults, including Some("") suppressing metadata; None uses Compact's current all-state labels or Rail's queued/skipped/blocked-only defaults. Accessor is pure borrowed property work, never domain I/O or retained data. Same single painter and row renderer handle both policies.

Append Variant::RAIL=8 in theme/recipe.rs after unchanged DEFAULT0,PRIMARY1,SECONDARY2,SUBTLE3,DANGER4,TOGGLE5,QUIET6,GHOST7; generated ALL/name/raw include it once. Existing Part::ROW_NUMBER=35 is appended to Steps::PARTS, not renumbered. Add narrow Family::STEPS recipe setup in theme/builtin/mod.rs: unchanged row_like DEFAULT and Rail-only variant rules. Other families and button variant lists remain untouched. Explicit variant(DEFAULT) with presentation(Rail) changes styling, not Rail geometry or capability. Use effective variant for every part, caller RowUi and attribution; never hardcoded concrete colors/role patches bypassing recipe precedence.

Rail rows exclude only a live scrollbar column. Gutter x, icon x+1, label base x+3; numbering writes one-based minimum-two-digit ordinal at x+3 and advances label by exactly3. Preserve paint order/clipping for 3/4/6-digit ordinals; no invented digit cap. Ordinal Running secondary, otherwise faint, removes bold. Glyphs queued/skipped/blocked blank; Running shared spinner; Done checked success; Failed ! error+bold. Labels queued muted, Running primary+bold, Done secondary, Skipped faint, Failed error+bold, Blocked secondary; focused/nonpressed adds bold. Rail does not add frontier ACTIVE emphasis. These are typed recipe-owned roles/state/glyph rules, not application drawing. All Inherit/Set/Clear/replacement and explicit overrides remain above defaults, with unchanged reserved geometry.

Metadata uses saturating avail=row.right−(label_x+1). It appears only mw>0 and avail>=mw+12; label budget avail−mw−2; meta right inset1. Meta Running secondary, Failed error, Blocked warning, otherwise faint, removes bold. Source hover/pressed row inversion, focused gutter, spinner frames and cursor fade protection remain source-qualified. Passive has no interactive flags but retains source cursor-row fade protection; hidden scrollbar reserves no column. Tiny raw-source containment exceptions are architectural lanes, not fabricated parity.

TASK-020 owns exact recipe/discriminant/re-export and component changes; existing 011/013/018 ancestry serializes these writers. Compile/public tests use non-Clone/non-Display models with borrowed row and meta closures, both builder orders, all four capabilities and both presentations. Existing Variant::ALL resolution test in measure.rs149 expands automatically and remains mandatory; completion_020 asserts exact original/new raw/name/ALL, parts census and real override effects. TASK-031 later joins complete family census, never a prerequisite receipt here. Only source apps Showcase terminal (036) and Jackin cockpit (054) use passive().presentation(Rail); inspectable source capability is direct component proof, not invented application behavior. No Form Steps bridge exists.

## Shared mandatory axes

CP-COMMON applies to every owned family: exact canonical cells, cursor and semantic state at 120×40/40×10 component allocations and each frozen application allocation; zero/one-cell and nonzero-origin clipping; Junie/Paper; truecolor/ANSI256/ANSI16/mono; every source-applicable base/focus/hover/pressed/current/selected/disabled/read-only/active/inactive/busy/loading/error/empty state. Non-applicability is source-qualified in the sealed baseline, not selected by the candidate. Each advertised visible override level and replacement slot must change intended painted cells without changing geometry/hits/focus or unrelated neighbors. The accepted covered-surface TextArea FIELD exception (main COMPONENT_ARCHITECTURE.md:6799–6815,6997–7003; HIST:EARLY-AMEND-044) instead requires exact recorded selected-part Resolved values while the composed painter's final digest stays unchanged; this is not an exemption for a visible override or replacement slot. Run real production update/draw and direct plus executable PTY trajectories; component-only states retain honest direct-lane applicability. Repeated draw preserves durable state, effects and callbacks. Keep borrowed non-static inputs, stable keys and measured visible-only work.

## Finite source-qualified witnesses

`trusted/source-witnesses.md` is normative under R-001/R-002/R-003. TASK-006 freezes every expanded case ID, exact source-qualified input/checkpoint and independent expected observation before dispatch; CHK-002/004 compare parity lanes, CHK-005 executes real positive/negative cases and CHK-006 proves architectural ownership. The finite rows supplement all assigned historical clauses and public part/slot census; they do not replace them with a representative subset. Missing source coverage is a failed precondition, not executor-selected non-applicability.

## Specific regression target

Add `crates/tui/tests/completion_020.rs` containing separately named positive and negative cases for every boundary above. Existing source-qualified targets remain active: `list_boundary`, `list_focused_click`, `tree_branch_policy`, `tree_state_replacement`, `nav_compact`, `nav_list_boundary`, `perf_collections`. Names denoting modules or source-discovered coverage are resolved by accepted TASK-007 inventory to actual package/target/test/profile identities; they are not invented Cargo target names. The protected runner executes those exact identities and the new target. A count, listing or matching source string is not execution proof.

## Family source contracts

### COMP:list

- family: list
- reference_implementation: O:src/widgets/list.rs
- main_implementation: M:crates/tui/src/components/list.rs
- architectural_target: Borrowed keyed List with RowUi and controlled semantic selection
- visual_status: missing fade
- interaction_status: unverified current-row completed click semantics
- api_refactor_status: implemented; parity verification required
- tests_available: M:list_boundary.rs;list_focused_click.rs;list_pointer_item.rs;list_row_extent.rs;list_row_renderer.rs
- tests_missing: CP-SCROLL-FADE protected cursor; insert/delete/sort range selection exact actions

### COMP:nav-list

- family: nav-list
- reference_implementation: O:Showcase shell/sidebars;Jackin/Holla custom sidebars
- main_implementation: M:crates/tui/src/components/nav_list.rs
- architectural_target: Single keyed NavList painter with sections compact mode and badges
- visual_status: missing protected-row fade
- interaction_status: unverified oracle focus versus navigation
- api_refactor_status: implemented; no duplicate old audit repair
- tests_available: M:nav_compact.rs;nav_list_boundary.rs;nav_list_scrollbar_visibility.rs;nav_list_scrolling.rs
- tests_missing: CP-SCROLL-FADE; current versus cursor; EnterContent; hover geometry narrow/full modes

### COMP:tree

- family: tree
- reference_implementation: O:src/widgets/tree.rs
- main_implementation: M:crates/tui/src/components/tree.rs
- architectural_target: Borrowed TreeNode source keyed expansion lazy loading and row renderer
- visual_status: missing fade
- interaction_status: lazy target and completed-click behavior needs oracle proof
- api_refactor_status: implemented; verify
- tests_available: M:tree_branch_policy.rs;tree_presentation.rs;tree_row_renderer.rs;tree_selected_click.rs;tree_state_replacement.rs
- tests_missing: Protected cursor fades; lazy insert/filter/replacement preserve target; exact branch policy

### COMP:steps

- family: steps
- reference_implementation: O:src/widgets/steps.rs
- main_implementation: M:crates/tui/src/components/steps.rs
- architectural_target: Borrowed keyed Steps with display or navigable mode
- visual_status: missing fade
- interaction_status: unverified
- api_refactor_status: implemented; verify
- tests_available: M:steps.rs lifecycle/keyed/100k/frontier tests;conformance.rs
- tests_missing: Oracle lifecycle glyphs skip/fail/block; mouse versus keyboard navigation; fades

### COMP:chips

- family: chips
- reference_implementation: O:src/widgets/chips.rs
- main_implementation: M:crates/tui/src/components/chip.rs
- architectural_target: Keyed borrowed ChipBar; controlled checked state; row renderer
- visual_status: unverified
- interaction_status: unverified
- api_refactor_status: implemented; verify
- tests_available: M:chip.rs fourteen units;render_components.rs;conformance.rs
- tests_missing: Exact overflow/close/add/checked behavior under reorder and narrow strip

### COMP:tabs

- family: tabs
- reference_implementation: O:src/widgets/tabs.rs;Holla/Jackin entity strips
- main_implementation: M:crates/tui/src/components/tabs.rs
- architectural_target: Borrowed keyed Tabs plus reusable author composition for distinct entity strip
- visual_status: unverified per-item tone and width
- interaction_status: unverified close/reorder/tab focus restoration
- api_refactor_status: implemented generic Tabs; app entity compositions need migration
- tests_available: M:tabs.rs close/reorder/readiness/active-state units
- tests_missing: Oracle active/inactive/hover/pressed/close widths; dynamic tab removal and return focus

### ARCH:A04

- id: A04
- area: identity
- oracle_state: WidgetId positional children and locate
- main_state: Id ItemKey Part PartRef
- architectural_target: stable keys across filtering/reordering/removal; no reverse hit scans
- status: implemented;parity unproven
- remaining_obligation: prove item/focus/selection identity on all restored app models
- available_gates: grid_keyed_cursor;tree_state_replacement;tablepro row_identity;no_owns_or_locate_in_applications
- missing_proof: golden dynamic collection scenarios
- evidence: 7b27732:crates/tui/src/lib.rs:51;7b27732:COMPONENT_ARCHITECTURE.md:499

### ARCH:A17

- id: A17
- area: collections
- oracle_state: owned cloned items positional callbacks
- main_state: borrowed CollectionCore RowUi KeySet
- architectural_target: borrowed keyed O(visible) rendering selection reconciliation
- status: implemented
- remaining_obligation: keep reusable models during application migration
- available_gates: perf_collections;list_boundary;tree_row_renderer;picker_projection
- missing_proof: model-read and allocation probes for restored large data
- evidence: 7b27732:COMPONENT_ARCHITECTURE.md:1179;7b27732:crates/tui/src/collection/mod.rs:1

## Complete assigned historical clauses

Each clause below retains its exact source and disposition. Accepted requirements are implemented or preserved and tested; rejected/superseded/deferred alternatives are checked as non-goals with the recorded replacement. No stale historical test-name claim is evidence of current execution. When a global clause spans tasks, this task owns the component-side obligation; app/integration owners close their separately mapped sides.

### HIST:A06

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §7 / B,J,§25; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Stable namespaced ItemKey identity independent of visible index
- Disposition: accepted; current retain_and_verify
- Remaining proof: Reorder/insert/delete/filter/sort must not retarget selection/edit/close
- Gates: identity property tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:7; global semantic anchor; supplemental clauses retained

### HIST:A61

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §53; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Tree cached keyed incremental index contains derived identities, never owned text/model
- Disposition: accepted_rejects_full_scan; current retain_and_verify
- Remaining proof: Warm draw/update O(viewport); subtree toggle proportional
- Gates: tree_100k counters
- Origin: docs/refactoring-plan/historical-obligations.tsv:62; global semantic anchor; supplemental clauses retained

### HIST:A62

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §53; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Tree filtering includes matches/ancestors independently of saved expansion; clearing restores exact expansion
- Disposition: accepted; current retain_and_verify
- Remaining proof: Filter revision/reorder/selection and navigation tests
- Gates: tree query properties
- Origin: docs/refactoring-plan/historical-obligations.tsv:63; global semantic anchor; supplemental clauses retained

### HIST:A63

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §60; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Tree chosen marker occupies reserved cell and follows stable key without new hit geometry
- Disposition: accepted; current compare_new_oracle
- Remaining proof: Set/Clear/Inherit and reorder visible output
- Gates: tree chosen frames
- Origin: docs/refactoring-plan/historical-obligations.tsv:64; global semantic anchor; supplemental clauses retained

### HIST:A64

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §50; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Chip checked-set uses CHECKED marker; AddRequested carries no fabricated item key
- Disposition: accepted; current retain_and_verify
- Remaining proof: Real-item collision and keyboard/mouse AddRequested
- Gates: Chip actions and parts
- Origin: docs/refactoring-plan/historical-obligations.tsv:65; global semantic anchor; supplemental clauses retained

### HIST:A67

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §57; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: NavList Chose distinct from EnterContent; shell alone focuses content
- Disposition: accepted; current retain_and_verify
- Remaining proof: Right/l versus Enter/Space/click and collapsed layouts
- Gates: sidebar_contract
- Origin: docs/refactoring-plan/historical-obligations.tsv:68; global semantic anchor; supplemental clauses retained

### HIST:A68

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §58; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Steps skipped remains read-only inspectable/navigable terminal state; stable action keys
- Disposition: accepted; current retain_and_verify
- Remaining proof: Whole-disabled differs from skipped; boundary no-op
- Gates: Steps lifecycle tests
- Origin: docs/refactoring-plan/historical-obligations.tsv:69; global semantic anchor; supplemental clauses retained

### HIST:A69

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §59; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Steps frontier derived/incremental with explicit invalidation and saturation recomputation
- Disposition: accepted; current retain_and_verify
- Remaining proof: Monotonic progression versus regress/reorder/reset counters
- Gates: Steps perf/accessor gates
- Origin: docs/refactoring-plan/historical-obligations.tsv:70; global semantic anchor; supplemental clauses retained

### HIST:A122

- Source: 7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md §53,§66; COMPONENT_ARCHITECTURE.md;docs/audit/main-holla/history
- Requirement: Tree render/count-independent work and toggle subtree bounds; isolate collection benchmark binary
- Disposition: accepted; current retain_and_verify
- Remaining proof: 100k render/flatten and 10k toggle names retained
- Gates: tree perf counters
- Origin: docs/refactoring-plan/historical-obligations.tsv:123; global semantic anchor; supplemental clauses retained

### HIST:F02

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f02-picker-action-eligibility-and-destructive-target-identity.md
- Requirement: One eligible-item resolver and stable destructive target mapping
- Disposition: current_holla_done_later_tablepro_open; current current_main_mapping_required
- Remaining proof: No empty/disabled/loading/stale fallback target; preserve filter reset policy
- Gates: Picker action and tab-close identity
- Origin: docs/refactoring-plan/historical-obligations.tsv:133; global semantic anchor; supplemental clauses retained

### HIST:F03

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f03-atomic-listbox-mutation.md
- Requirement: Collection mutation atomically reconciles items/checks/cursor/chosen/range anchor/scroll
- Disposition: later_open; current current_main_mapping_required
- Remaining proof: Actual Settings delete then Shift+Up plus insert/delete endpoint properties
- Gates: List and Settings identity
- Origin: docs/refactoring-plan/historical-obligations.tsv:134; global semantic anchor; supplemental clauses retained

### HIST:F05

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; IMPROVEMENTS_PLAN.md;docs/improvements/f05-preserve-treeview-target-across-lazy-insertion.md
- Requirement: Tree cursor/chosen target stable through lazy insertion
- Disposition: current_done; current current_main_mapping_required
- Remaining proof: Reorder/insertion cannot silently retarget Holla resources
- Gates: Tree identity
- Origin: docs/refactoring-plan/historical-obligations.tsv:136; global semantic anchor; supplemental clauses retained

### HIST:O03

- Source: 215990ef;abb480ac;c9661f6c;64add0ea;812bad84;02f5294b; docs/improvements-plan-reference.md
- Requirement: New generic keyed-collection framework proposal
- Disposition: retired; current current_main_mapping_required
- Remaining proof: Solve required identity through accepted main collections
- Gates: rejected design ledger
- Origin: docs/refactoring-plan/historical-obligations.tsv:190; global semantic anchor; supplemental clauses retained

### HIST:EARLY-AMEND-007

- Source: 6ec29171 §§16.1,29.7; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Regression proof exercises Clear geometry, keyed strip rendering, both cell boundaries, and actual ASCII glyph sets.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Clear reserves a blank cell; same buggy enumeration or non-overflowing fixture is insufficient; no-box-drawing is weaker than all-ASCII.
- Gates: Clear/Inherit differential; overflowing reorder; left/right out-of-area rejection; full typed GlyphSet ASCII scan.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:8; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-016

- Source: 3adb6efe §§6,17 A7;a1759b2a §20.10 item23; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: ChipBar add is a collection-level AddRequested action with Part::NEW and add(label); no sentinel ItemKey.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Existing-item actions remain keyed; fabricated Activated(add-key) rejected.
- Gates: Keyboard/mouse AddRequested equivalence; reorder/overflow identity; no synthetic item enters checked/cursor data.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:17; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-017

- Source: a1759b2a §20.10 item23;3adb6efe §29.7; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: ChipBar checked membership uses canonical Checked in its reserved cell; component patches preserve caller META and owned geometry.
- Disposition: accepted; oracle output controls; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: CHECKED replaces synthesized SELECTED; automatic CONTAINER/LABEL patches do not overwrite caller-owned row styling.
- Gates: Set/Inherit/Clear marker; trailing META; close/pad/overflow positions; no CheckboxOn clipping or label truncation.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:18; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-AMEND-054

- Source: c3b51b96 §20.10 item35; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Empty List/Grid/FilterList/Picker regions inherit owning EMPTY style; NavList hover must lift only hovered row.
- Disposition: accepted reusable ownership; oracle output controls; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: EmptyState::draw_inherited and hovered-difference masks remove blank/gap/header tint bug class;historical20-key classification style-only.
- Gates: Canonical cells including blanks;empty full rect;hover header/gap/other-row invariance;exact text/geometry unchanged.
- Origin: docs/refactoring-plan/history-early-amendments-obligations.tsv:55; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-005

- Source: 2e453023 §§6,12;95ab6529 §21; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Reconcile cursor, selection, anchors and strip windows by stable item identity; generation/membership changes invalidate projections.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Equal lengths/endpoints cannot alone prove unchanged membership; rendering must consume keyed window too.
- Gates: Insert/remove/reorder/duplicate/disabled tests; overflowing multi-digit strip fixtures; unchanged-generation allocation bounds.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:6; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-020

- Source: 2e453023 §12.4; e8d053c9 §20.9; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: List/grid borrow data; tree projection is incremental/keyed; TextViewport caches source ranges/widths rather than owned grapheme strings.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Original TextViewport keep means behavior retained, storage rewritten.
- Gates: 100k items/nodes/lines with bounded visible-window allocations and identity-preserving edits.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:21; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-022

- Source: e8d053c9 §20.9 items13,15,16; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Select-all uses AllExcept; manager rows rebuild on world generation only; inert-below omits background registrations.
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: No full materialized100k key set or per-key manager rebuild.
- Gates: Select-all <100 allocations; manager key0/frame<60; modal hits <25percent of base workload.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:23; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-055

- Source: 2e453023 §12.5 J10–J13; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Shared RowDecor/change slots, keyed tabs, meter threshold helper and modal stack/result routing replace duplicates without moving product policy into library.
- Disposition: accepted_with_later_meter_policy; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Quota lifecycle remains application-owned; runtime feedback and typed actions replace ad hoc UI state.
- Gates: J10–J13 keyed row action, tab reorder, thresholdboundary, layerresult tests with pinned copy/cadence.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:56; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:EARLY-057

- Source: 70dacec1 §29.7;7b27732a §29.7 excerpt; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: RadioGroup caller value and ChipBar Add action were open in Q, later resolved§50; StatusBar hover primitive existed but lacked consumer, later resolved§51.
- Disposition: historical_open_items_superseded; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Preserve accepted clause with amendment: Do not revive fabricated ItemKey sentinel or claim hovered_part presence was implementation closure.
- Gates: Join middle-reader original§50–51; actual payloadless AddRequested/NEW and keyed hover consumer tests.
- Origin: docs/refactoring-plan/history-early-obligations.tsv:58; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-003

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:40-87;998-1004
- Requirement: Stable ownership and item identity replace reverse owns/locate scans
- Disposition: accepted amended; current unverified
- Remaining proof: Avoid positional Path and stale frame geometry
- Gates: Insert/remove/reorder plus click/close/edit identity; collision-kind negative control
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:4; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-009

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:258-349
- Requirement: Brand/chip/choice use shared presentation and roving/field mechanics
- Disposition: proposal homes; accepted broad dispositions; current unverified
- Remaining proof: Map each old family explicitly; overflowing chip bar remains reachable
- Gates: Full-width overflow; disabled/readonly distinctions; oracle marker/bracket geometry
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:10; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-016

- Source: locator_pin=7b27732a8c3c131760ec3438f641cb3c11343a42;historical_provenance_edges=383e16fc; docs/audit/api-audit.md:659-671;749-760;821-879
- Requirement: Separate generic progress from quota lifecycle; stable Tabs and keyboard SplitPane
- Disposition: accepted direction; exact homes owner governed; current unverified
- Remaining proof: Domain quota/tab/PTY policy stays app-owned; all catalog families mapped
- Gates: Keyed reorder/close; keyboard and drag resize; supported statuses and glyph parity
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:17; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HAR-024

- Source: e81ca17b; docs/audit/architecture-research.md:132-154;344-454
- Requirement: HashedId/debug registry and whole-key reconciliation sketches
- Disposition: amended/superseded; current unverified
- Remaining proof: Structural identity plus cheap change-aware reconciliation
- Gates: Same-index kind collision; reorder action; 100k versus1k dispatch/draw
- Origin: docs/refactoring-plan/history-inputs-api-app-research-obligations.tsv:25; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-020

- Source: e81ca17b; docs/audit/domain-boundary-audit.md2.9-2.11;4.2
- Requirement: Stable Tabs and SplitPane/Viewport reusable mechanics; dirty-close preview/pinning pane-tree/PTY semantics app-owned
- Disposition: accepted and amended; current not independently tested
- Remaining proof: Remove per-render tabs/viewport reconstruction; keep domain tab policies and pane simulation
- Gates: logical close after reorder; strip window; zoom/split/drag; caret override without clone
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:21; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-022

- Source: a156054d; docs/audit/performance-audit.md6 R1-R7
- Requirement: Borrow data, window drawing, cheap reconciliation, sorted rules, nonallocating hot paths
- Disposition: accepted and amended; current not independently tested
- Remaining proof: Join exact later thresholds; cheap generation heuristic cannot hide arbitrary in-place data changes
- Gates: 100k vs1k scaling; explicit invalidation; real action dispatch; zero allocation style path
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:23; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-023

- Source: a2ddd278; docs/audit/performance-audit.md6.3; STATE
- Requirement: Windowed incremental viewport and tree; remove full viewport clone; code edit-counter caches sorted span walks
- Disposition: accepted; current not independently tested
- Remaining proof: No buffer-size-scaled render/push or per-frame clone; semantic cache boundary retained
- Gates: 100k lines push/render; node-count-independent render; clone absence and byte limits
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:24; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HI-026

- Source: ba858131; docs/audit/modern-api-audit.md4; foundations2.5
- Requirement: bitflags adopted; library SmallVec containers rejected; sorted Vec KeySet
- Disposition: accepted amended: transitive backend-only SmallVec permitted; current not independently tested
- Remaining proof: Reject own SmallVec/direct crossterm; prove every permitted backend-internal path crosses ratatui-crossterm
- Gates: dependency graph path dominance; sorted-set mutation and binary-search probe test
- Origin: docs/refactoring-plan/history-inputs-obligations.tsv:27; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-57-ENTRY

- Source: 14bca4a3 §57; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Right/plain l emit EnterContent key; Enter/Space/click Chose key; shell owns focus handoff
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: LeaveForward; overloaded Chose; component cx.focus; modified l binding
- Gates: Input/action/focus oracle scenarios and stable-key remapping
- Origin: docs/refactoring-plan/history-late-obligations.tsv:8; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-57-PARTS

- Source: 14bca4a3 §57; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: NavList patches CONTAINER/GUTTER/MARKER/ICON/LABEL/HEADER/BADGE; slots GUTTER/MARKER/ICON/HEADER/BADGE; RowUi carries only CONTAINER/LABEL
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Generic row-part patch leakage; implicit BADGE row ownership
- Gates: Badge budget and slot replacement; full/collapsed grouping exact separators/text
- Origin: docs/refactoring-plan/history-late-obligations.tsv:9; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-58-LIFECYCLE

- Source: 14bca4a3 §58; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Skipped is READ_ONLY terminal lifecycle, remains reachable/activatable; only entire rail disabled suppresses
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Skipped-as-disabled; initial false Moved; wrapping
- Gates: Seed stable key; physical-row move clamp; boundary no mutation/repaint/action; keyed click/double-click
- Origin: docs/refactoring-plan/history-late-obligations.tsv:10; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-58-PARTS

- Source: 14bca4a3 §58; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Steps exact parts/slots; lifecycle META direct-owned; optional row META row-owned; scroll all overrides
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Forwarding arbitrary RowUi patches
- Gates: Owner isolation; lifecycle metadata space; track/thumb forwarding
- Origin: docs/refactoring-plan/history-late-obligations.tsv:11; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-59-CACHE

- Source: 14bca4a3 §59; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Private derived runtime frontier cache; StepsState monotonic revision; invalidate reconciles; saturation recomputes
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Semantic frontier stored in StepsState; whole scan unchanged draw
- Gates: steps_100k_rows_render accessor counts; forward advance; reset; completed cache; saturation
- Origin: docs/refactoring-plan/history-late-obligations.tsv:12; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-59-INVALIDATE

- Source: 14bca4a3 §59; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Caller invalidates retry/reset/interior reorder/regression; frontier() explicit O(n); draw O(viewport) with stamps
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: Claim every model access forbidden outside viewport; undetectable in-place edits auto-reconciled
- Gates: Same-length interior changes plus invalidate; deterministic counts outside PERF_STRICT
- Origin: docs/refactoring-plan/history-late-obligations.tsv:13; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HL-60-MARKER

- Source: a1759b2a §60; COMPONENT_ARCHITECTURE.md;REFACTORING_STATE.md where cited
- Requirement: Tree chosen ItemKey supplies SELECTED; reserved post-disclosure marker cell; slots honor Set/Clear/Inherit
- Disposition: accepted; current not independently verified in this source ledger; join architecture-matrix.tsv and {showcase,tablepro,jackin,holla}-scenarios.tsv; source disposition is not completion
- Remaining proof: Apply accepted contract; reject/supersede: ACTIVE/CHECKED substitute; fixture/row painter owns default chosen marker; extra pointer target
- Gates: Mono distinction, stable label coordinate, reorder, clear/replacement and region equality against oracle
- Origin: docs/refactoring-plan/history-late-obligations.tsv:14; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM44

- Source: GAP-10/11 3fa382cb; docs/audit/legacy-test-disposition.md
- Requirement: Tabs active underline/plane/no gutter and distinct hover/cursor/active weight/plane relationships remain observable
- Disposition: accepted_UI_invariant; current proof_required
- Remaining proof: Exact oracle visual grammar with keyed suffix through caller RowUi; no legacy TabItem revival
- Gates: active-only accent rule; no gutter; hover vs cursor weight; suffix after label
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:45; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM55

- Source: 3adb6efe §50;15371443; COMPONENT_ARCHITECTURE.md
- Requirement: ChipBar checked membership derives CHECKED; six PARTS; real item actions keyed; AddRequested payloadless and NEW has no fabricated key
- Disposition: accepted; current retain_verify
- Remaining proof: Checked/unchecked glyph, reserved pad, overflow/close geometry; add cannot collide with real identity
- Gates: mouse/keyboard add; item-key collision; live checked set; Set/Clear; exact CLOSE/OVERFLOW slot contract
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:56; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM60

- Source: 3adb6efe §53;session5 review correction; COMPONENT_ARCHITECTURE.md
- Requirement: Tree runtime cache stores only owned structural descriptors; warm update/draw O(viewport); incremental subtree expand/collapse; stable-key state; safe saturation and frame-gap eviction
- Disposition: accepted; current retain_verify
- Remaining proof: Cold/revision/invalidate rebuild once; small subtree counters; cache eviction and warmed reorder without stale identity
- Gates: 100k flatten/render; key_tree_toggle_10k; deterministic node-access counters; saturation/cache-clear
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:61; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:HM61

- Source: 3adb6efe §53 query followup; COMPONENT_ARCHITECTURE.md
- Requirement: Tree borrowed matcher+caller revision; strict matches-plus-ancestors projection; saved expansion untouched; unmatched children/branches excluded
- Disposition: accepted; current retain_verify
- Remaining proof: Filter forced ancestors cannot collapse away results; Left parent Right descendant; star/minus disabled; chosen hidden key retained
- Gates: query revision/rebuild once; clear query exact restoration; parent match excludes unmatched descendants
- Origin: docs/refactoring-plan/history-middle-obligations.tsv:62; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG19

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: External insertion/removal/reorder preserves logical focus/selection/edit target; disappearing owner reconciles predictably
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-identity; F02/F03/F05/F06
- Gates: Stable-key mutation/reorder/disappearance property tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:20; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG33

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Collections support borrowed domain content, custom rows/cells, metadata and relevant empty/loading/error states
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-collection; ItemSource/AsItem/row adapters
- Gates: Non-owned consumer fixtures and visible empty/loading states
- Origin: docs/refactoring-plan/history-other-obligations.tsv:34; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG34

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Collection cursor differs from chosen value; relevant single/multiple/range selection remains explicit
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-collection; Choice/Picker/Grid
- Gates: Independent cursor/value/check/anchor mutation tests
- Origin: docs/refactoring-plan/history-other-obligations.tsv:35; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md

### HIST:RG35

- Source: e48137f1:REFACTORING_GOAL.md;dce91d51;2d81eec4;25ea92b0;efada044;d4715f8e; REFACTORING_GOAL.md
- Requirement: Large/lazy collections avoid full-dataset cloning and avoidable full-tree scans on every event
- Disposition: accepted; current historical_contract_extracted_current_proof_owned_by_matrices
- Remaining proof: A-performance; tree/grid/viewport perf
- Gates: 100k/1m work counters and allocation assertions; dirty separate from idle
- Origin: docs/refactoring-plan/history-other-obligations.tsv:36; retained supplemental source clause; exact equivalent/refinement/supersession groups in history-ledger-reconciliation.md
