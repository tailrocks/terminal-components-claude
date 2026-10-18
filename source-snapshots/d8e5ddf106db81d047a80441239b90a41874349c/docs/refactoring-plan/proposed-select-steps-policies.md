# Proposed shared Select and Steps policies

Status: proposal history; final authority is ADJ-15/16 and the fixed TASK-018/019/020 contracts. The original ValueRequests proposal below is rejected: Select already owns its value and the existing Chose/Form bridge handles commits, so NavigationRequested and a second Form engine are unnecessary. Pins: main `7b27732a8c3c131760ec3438f641cb3c11343a42`, oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Preserve existing public defaults and accepted architectural history. One shared component implementation per family.

## Select proposal

### Authority and default

Keep existing `Select::new`: cursor/value separation, its extended navigation table, no navigation action, Popover `Dismiss::ALL`, unbound Tab/BackTab traversal and focus-out dismissal without opener restoration. Main architecture1834 and §29.8 explicitly accept these choices. `SelectState` remains the documented state-owned value exception, observed/set through existing `value()`/`set_value()`; no signature change to `update(cx,state,items)`.

Oracle select.rs70–128 differs: closed plain Up/Left or Down/Right derives the next index from selected value, immediately assigns it and returns Changed even at a clamped boundary, but SelectEvent::Changed only when the value differs. Closed k/j/Home/End/Page keys do nothing. Open Up/k and Down/j move cursor; Enter/Space commit and close; Esc restores cursor; other keys including Tab consume without moving focus. A preceding Ctrl/Alt-character guard ignores those characters; non-character modifiers are not universally restricted. Do not replace that with a broad “all modified keys ignored” rule.

### Exact additive API proposed

```rust
pub enum SelectNavigation { Cursor, ValueRequests }
pub enum SelectOpenKeys { FocusTraversal, ConsumeUnhandled }

// Existing defaults: Cursor and FocusTraversal.
Select::navigation(self, policy: SelectNavigation) -> Self
Select::open_keys(self, policy: SelectOpenKeys) -> Self

// Additive, carrying a real live key, never a synthetic index:
SelectAction::NavigationRequested(ItemKey)
```

`ValueRequests` selects the source-directional keyboard grammar above. While closed, shared navigation computes from the committed value's live index, updates the navigation cursor and emits NavigationRequested(key), including clamped repeats; it does NOT implicitly change `SelectState::value`. Open navigation still reports no value request. Existing Choose/pointer commit remains Chose; source callers compare the pre-update value and invoke their domain change callback only when the resulting key differs. A clamped source arrow still preserves Changed flow without a false domain callback. Disabled/readonly/inert input and reconciliation-only updates emit no request. Pointer Press must not borrow the keyboard request path.

Empty items have no live key and must never fabricate NavigationRequested/Chose. In the source policy, a closed directional key retains the oracle Changed flow without a value callback; opening an empty list then Enter/Space closes with Changed and no chosen key (a typed Closed is admissible). This source close behavior is distinct from main choose()'s current empty-list consumed early return; preserve the generic default separately. Option removal is reconciled by the existing live-key contract before any request, not by copying a stale old index.

`ConsumeUnhandled` is only an open, enabled policy. It publishes explicit focused-owner Tab/BackTab command bindings while the popup is open, consumes them in Select's own update, and withdraws them on close/disable. Runtime.rs832–839 already gives an explicitly published focused binding precedence over traversal; no Runtime, LayerSpec, focus trap or capture bypass is required. The Popover remains OVERLAY, not TRAPS_FOCUS. True external focus-out still dismisses without restoring the opener.

Use one typed shared binding/command table per admitted phase/policy; freeze the exact oracle code/modifier matrix, including Shift-only plain characters, Ctrl/Alt character rejection, and non-character modifier handling. Known modifier-bit combinations are finite; Tab/BackTab publication must cover the combinations that runtime otherwise traverses, not just one plain Tab example. Unhandled open keys consume through the same Select update fallback; no application raw-key interception, fake focus, second navigation function or model mutation in draw. Generic FocusTraversal continues to publish no Select Tab binding.

The ordinary source caller composition is `.navigation(ValueRequests).open_keys(ConsumeUnhandled)`. These are reusable interaction policies, not an oracle/legacy-named widget, style clone or per-application branch in the library.

### Form bridge and compile ownership

A configured `FieldKind::Select(LabelSelect::new(...).navigation(ValueRequests).open_keys(ConsumeUnhandled))` is already an explicit caller policy. TASK-019 extends the existing private `Select::update_in_form` bridge: seed state from the controlled usize, call the same update once, commit a NavigationRequested live ByIndex key back to that controlled value, and expose FormAction::Committed only for an actual value change. Suppress a same-value Chose/request outward action while retaining Changed flow; popup Opened/Closed retain their existing Form mapping. No second public Form boolean is needed: the configured Select prop is the sole policy owner. Default LabelSelect emits no request and preserves existing Form semantics. No public projection of private Form state or Secret is added.

TASK-018 owns select.rs plus narrow component/root re-exports for the two enum types and the additive action; conformance.rs1280–1281 must add the new key-carrying arm. Its existing form.rs exhaustive match1121–1125 needs compile-safe adaptation without implementing the later Form commit policy. TASK-019 must explicitly include select.rs for its bridge and depend on018. Main source search finds no other exhaustive external SelectAction match; Showcase chips.rs280 uses matches! and remains compiling, but real source callback consumption belongs037. No old test identities are removed: generic old expectations remain, source-policy fixtures are additional.

### Exact source callers and application owners

| Oracle caller | Control | Owner |
| --- | --- | --- |
| src/bin/showcase/pages/chips.rs53–61 | sort, page size, disabled single engine | TASK-037 |
| src/bin/holla/screens/review.rs943 | Choice-valued Args fields | TASK-043 |
| src/bin/jackin_preview/screens/editor.rs178 | dirty policy | TASK-052 |
| src/bin/jackin_preview/screens/config.rs1700,1721,1838 | schema/config/auth choice fields | TASK-052 |
| src/bin/jackin_preview/screens/accounts.rs532 | provider choice | TASK-053 |
| src/bin/tablepro/connections.rs143,165 | engine and group | TASK-058 |
| src/bin/tablepro/app.rs1408–1409,1692,2072 | filter column/operator and dependent operator rebuild | TASK-060 |

Confirmed Tab applicability: Holla app626–635 calls focused page first and returns consumed before637–644 fallback traversal; review1022–1053 dispatches focused Choice. Jackin shared FormDialog modals1073–1093 traverses only when no editing field/open Select, otherwise1127–1135 delegates to Select; direct Editor general_key1532–1536 precedes screen1695+ and App630–658 fallback. TablePro Filter1665–1698 traversal is gated by neither text editing nor column/op open. Connections573+ delegates engine622/group636 before parent fallback. Each listed app owner must still bind its exact intact oracle trajectory and outer global-chord precedence; no claim that every key bypasses app-global shortcuts.

Required negatives: default generic Tab still traverses; source open Tab/BackTab stays, closed Tab traverses; programmatic external focus-out dismisses; disabled/readonly/inert cannot trap; live options reorder/remove cannot retarget a stale key; closed value!=cursor starts from value; clamped callbacks stay zero; pointer Press emits no request; source open Home/Page keys consume but do not navigate; generic extended keys remain active. One shared binding table/command engine must explain both policies.

## Steps proposal

### Authority and why not a boolean capability switch

Main steps.rs1–11 explicitly chooses capability by constructor (§23 K2/G4), not a resurrected selectable flag. `Steps::new` is ClickOnly: Press/Click move cursor, DoubleClick activates, wheel works; `Steps::navigable` adds focus/bindings and single-click/Enter activation. Preserve both exactly.

Oracle StepRail defaults selectable=false and numbered=true (steps.rs72–90). That display mode has no control/row-hit/hover/focus action, but wheel and scrollbar remain live. Thus `.disabled(true)` is wrong: modern disabled ignores all input. Source selectable=true only moves cursor on completed row click and Up/Down/Home/End grammar; Enter/Space do not activate, and Page keys are not supported. Source flags and defaults cannot be implemented merely by changing new().

### Exact additive capability and presentation API proposed

```rust
// New entry points, same borrowed Steps type and painter:
Steps::passive(id)     // rows noninteractive, wheel/scrollbar live
Steps::inspectable(id) // focus + source directional navigation, Moved only

pub enum StepsPresentation { Compact, Rail }
Steps::presentation(self, value: StepsPresentation) -> Self
Steps::numbered(self, value: bool) -> Self
Steps::meta(self, get: &'a dyn Fn(&T) -> Option<&str>) -> Self
```

Existing constructors default Compact; their controls, activation table and cells remain unchanged. Capability is internally one typed four-way policy (existing Pointer/Navigable, additive Passive/Inspectable), not inconsistent navigable/interactive/activate booleans. `is_navigable()` returns true for Navigable and Inspectable. `StepsAction` remains Moved/Activated; no new action variants are required. Passive never emits either; Inspectable emits only Moved for an admitted source navigation/click operation, including clamped Changed repeats, and never Activated from Enter/Space/DoubleClick. Its row cursor changes on completed click, not Press; pointer press may still paint pressed affordance. The exact source key guards are retained: Up/k/Down/j/Home/g require plain; End/G's actual source arm lacks that guard. Modern extended Page navigation remains only in the unchanged Navigable constructor.

Effective numbering defaults to false for Compact and true for Rail unless `.numbered` was explicitly supplied; an internal Option<bool> gives order-independent configuration. The borrowed pure meta accessor does not enter durable state, allocate formatted strings or perform domain I/O. Some(nonempty) replaces lifecycle metadata, Some("") explicitly suppresses it, None selects the presentation default. Compact defaults to all current StepState::label() words; Rail defaults only queued/skipped/blocked. Labels remain the existing borrowed row renderer; no new owned Step model or Display bound on custom rows.

Rail is one semantic presentation in the same painter, not an alternate source renderer. Proposed role ownership is an append-only `Variant::RAIL = 8` (existing variants0–7 unchanged), used with Family::STEPS; Compact continues DEFAULT. This makes policy-owned role/mono differences auditable and independently overridable without changing global Compact recipes or hardcoding colors in component code. Add existing `Part::ROW_NUMBER` to Steps' advertised parts for ordinals; no new Part discriminant. Root may choose an equivalent typed recipe representation, but must not leave the role/variant owner unspecified.

Rail exact source paint: row excludes only the live scrollbar column; gutter x, state glyph x+1, label base x+3. Optional ordinal is one-based minimum-two-digit formatting at x+3, label advances a fixed3 cells; preserve source paint order and clipping for three-/four-digit ordinals rather than inventing a two-digit cap. Running ordinal secondary, others faint, remove bold. No extra frontier ACTIVE emphasis: oracle computes frontier but does not paint an additional frontier state.

Lifecycle glyphs: queued/skipped/blocked blank; Running actual shared spinner frame from runtime time; Done checked glyph/success; Failed !/error/bold. Label roles: queued muted; running primary+bold; done secondary; skipped faint; failed error+bold; blocked secondary; focused nonpressed label adds bold. All remain typed semantic roles and existing glyph slots, with explicit Inherit/Set/Clear/replacement behavior and identical reserved geometry.

Metadata: `avail = row.right - (label_x+1)` saturating; show only when mw>0 and avail>=mw+12; label budget avail−mw−2; right inset1. Running metadata secondary, Failed error, Blocked warning, others faint; all remove bold. Source source-qualified tiny containment remains separate from parity where raw oracle writes exceed allocation. The producer must make every role/slot/ordinal fixture executable; per-app ad hoc number/meta/glyph drawing is disallowed.

### Scope, callers and proof

TASK-020 owns steps.rs and its tests, plus bounded components/mod.rs and lib.rs public re-exports; the proposed RAIL discriminant/recipe requires explicit theme/recipe.rs and theme/builtin owner scope or an earlier declared producer. It must serialize after existing theme/public-export writers; TASK-011/013 precede the production repair already. Do not silently change Variant::ALL without extending complete variant/census proofs and preserving all original discriminants. TASK-031 closes the independent component/Overrides census.

Exactly two immutable source applications construct StepRail: Showcase terminal.rs97 (TASK-036) and Jackin cockpit.rs121 (TASK-054). Both use passive source behavior, default numbered rail, caller lifecycle and optional activity/duration meta. Their composition is `Steps::passive(id).presentation(Rail).step(...).meta(...)`, plus their existing stable keys/label renderer; no activation domain action is wired. Holla has no StepRail source caller. Inspectable is required to preserve the source component's selectable=true behavior in direct conformance, not falsely attributed to a nonexistent app branch. Form has no Steps field kind; no Form bridge is needed.

Required positive/negative cases: all six states; numbered true/false; ordinal indices0/8/9/98/99/998/999/99999; custom Some(text)/Some(empty)/None; metadata widths threshold−1/exact/+1; current≠frontier; all ten spinner frames; passive wheel/track live while rows/key/hover inert; disabled all input denied; generic new and navigable unchanged; inspectable movement includes Skipped/Blocked, no activation or Press mutation; visible-only callback/work and revision saturation. Source menu/Tab choices do not broaden Steps capability.
