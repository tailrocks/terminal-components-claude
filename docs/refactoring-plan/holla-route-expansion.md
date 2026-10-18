# Holla native tests and resized oracle transcripts

This contract resolves PARITY-FINAL-04. It applies to every Holla `ROUTE` seed, including routes nested inside HP, ACTIVITY, SCAN or SWEEP rows. Oracle authority remains `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Native test execution and resized capture are separate required products; neither substitutes for the other.

## Product A: unchanged source-native execution

Run every original Holla test with its original constructor dimensions, literal pointer coordinates, explicit resize events, fixture mutations, helper calls and assertions. Preserve the original test source and native test identity. Do not parameterize its constructor or edit an assertion to make this run pass. TASK-007/TASK-071 account for the complete source-qualified native test inventory, including all 143 previously measured binary tests and the owned PTY tests; TASK-003 records exact commands, source hashes and actual results. Every reached original assertion must pass. Conditional branches and helper assertions retain their original execution semantics; an unexecuted conditional branch is not reported as executed coverage.

Examples that remain literal in this product:

- `activities_survive_navigation_and_merged_logs_keep_identity` constructs 120×40 and checks row 38 for `attached`.
- `hard_cases_stay_legible_and_report_failed_discovery` starts at 100×30, checks row 28, explicitly resizes to 80×24 and checks row 22, returns to 100×30, and separately constructs a 160×50 instance.
- `mouse_selects_runs_and_opens_alternatives` starts at 120×40 and ends with the literal click `(3,38)` on the status path.

## Product B: oracle-derived per-size replay

Separately build action/checkpoint programs for 80×24, 100×30, 120×40 and 160×50. These are new oracle-derived capture programs, not claims that an unchanged native test passes at four substituted sizes. Preserve source fixture/domain mutations, action order, key modifiers, input payloads and tick boundaries. Preserve explicit resize commands from the source verbatim unless the scenario register itself specifies a different resize sequence. Record each constructor instance independently; materialize the constructor-size policy and its complete resulting checkpoints before candidate dispatch.

Capture complete terminal cells, cursor and semantic state after every materialized event/assertion boundary, including layouts where a native spatial claim no longer applies. All expected records come from the resized **oracle**, never from candidate geometry or failed candidate assertions. Source-native assertion success stays a separate mandatory receipt even when its spatial equivalent differs at another size.

### Protected assertion and coordinate map

TASK-003 must produce a complete `native-resize-map` product before its baseline can be accepted. The planner freezes the source-site inventory and the rules here; the accepted baseline materializes exact source spans, schemas, numeric positions, inserted reveal actions and checkpoint IDs. Its independent reviewer approves the entire map and rejects unexplained omissions. The candidate has no map-writing authority.

Every native assertion and pointer site receives a record containing source commit, file/blob hash, test/helper symbol, source span and ordinal, native constructor instance/size, native assertion or coordinate expression, original semantic owner, mapping category, all four size expansions, exact source rationale, expected observation schema, and resulting checkpoint IDs. Preserve both the original text and the mapped record. Require complete bidirectional source-site coverage; unknown, missing, duplicate or unmapped sites fail. The inventory scan is a starting set, not permission to omit additional sites found by the independently qualified Rust source traversal.

Permitted categories are deliberately narrow:

1. **Dimension-independent domain assertion.** Replay the original assertion unchanged against the same actual production domain state. Examples: argv/cwd/host, activity exit, retained output count, trust digest, filesystem effect, cleanup count, action identity and world ownership.
2. **Source-relative geometry assertion.** Replace only its coordinate expression with the exact owner geometry supplied by each size's oracle. Preserve the actual predicate and record both expressions. For the three status examples above, `App::draw_frame` places status at `area.bottom()-2`; row 38 at height40 and row28 at height30 therefore map to the same status row at each size. Assert the same `attached`/`unavailable` predicate at the oracle-owned status surface where that source item is displayed, with source-proven clipping/priority applicability if necessary. Never search the candidate frame for the string.
3. **Native-layout-specific assertion.** Preserve and execute it unchanged in Product A. For Product B, record the exact source layout guard and oracle-observed alternative rather than asserting an invalid universal layout. Examples: narrow Finder summary versus wide preview split, fixed scroll movement at120×24, and visibility of a clipped row. The alternate per-size complete frame and semantic state remain mandatory; this category does not remove a checkpoint or claim the native assertion was executed at a different size. If its native guard is still true, retain the predicate instead of exempting it.
4. **Pointer target remap.** Resolve the native pointer to its exact source semantic owner/item key and relative cell anchor, then resolve that same owner in each size's oracle before the action. Preserve Move/Down/Drag/Up/Secondary/DoubleClick ordering and use the frozen resulting literal coordinates against the candidate. Native `find()` calls are geometry-discovery inputs only during oracle preparation, never candidate retargeting. If the target needs reveal, use only its source-supported bounded scrolling/focus keys; record the additional oracle-derived reveal inputs and require every original action to remain accounted. A genuinely unavailable target requires explicit source-guard/absent-hit evidence and an independently approved applicability entry; do not invent a widget, silently omit the action, or fake a successful click.

Record mapped layout applicability as a product fact tied to the exact source guard, never as an executor-selected allowlist. A changed candidate layout cannot change that applicability. Full-frame equality remains exact and unmasked. A geometry mapping is not a tolerance or a layout normalization.

### Concrete mandatory mappings

| Native source site | Native proof | Resized proof |
| --- | --- | --- |
| `app_tests_flows.rs::activities_survive_navigation_and_merged_logs_keep_identity`, row38 assertion after monitor Enter | Run original120×40 assertion unchanged. | Observe the same attached-status item in `draw_status`; status rectangle comes from `draw_frame` at bottom−2. Record exact item visibility/priority guard and complete per-size frame. |
| `app_tests_flows.rs::hard_cases_stay_legible_and_report_failed_discovery`, row28 and later row22 assertions | Run original100×30→80×24→100×30 sequence unchanged, including the separate160×50 constructor. | Map status-row ownership through `draw_frame`; retain explicit resize events, source text predicates and full oracle frames at every resulting size. |
| `app_tests_flows.rs::mouse_selects_runs_and_opens_alternatives`, `(3,38)` click | Execute native120×40 literal click and original `Copied` assertion. | Resolve `STATUS_PATH` from oracle hits at each size; preserve its relative cell anchor and completed-click events. Assert exact clipboard/status effect and compare entire frames. |
| `app_tests_rows.rs::one_click_on_an_argument_field_starts_editing_at_the_pointer`, `ARG.child(0)` hit plus `(x+3,y)` | Keep original120×40 hit lookup, click and `/XUsers/alex/work/big` assertion. | Freeze the same field owner and relative text-cell anchor from each size's oracle; preserve the exact insertion/selection/cursor predicate, never candidate lookup. |

## Independent qualification and failure cases

TASK-070's independent preparation driver must exercise a fixture containing both a fixed native footer/status row and a literal native pointer. The positive run must prove unchanged native assertion execution **and** distinct correctly mapped per-size action/frame records. Negative cases must reject: a deleted/ignored native assertion; falsely reporting native execution after resizing its constructor; reusing row38 or `(3,38)` at a shorter height; mapping to the candidate's moved owner; a missing assertion-map row; changed predicate; missing reveal/original action; empty applicability in place of a required observation; and accepting a frame assembled by copying oracle cells instead of capturing the real process. After every rejection, the unchanged positive fixture must pass again.

The independent driver owns verdicts; neither the candidate adapter nor a self-authored assertion map may certify itself. The runner owner records the actual fixture/driver path, exact counts and measured results. Until that qualification passes, TASK-003 is not dispatch-ready. No terminal-components production change is authorized by this planning correction.

## Executed preparation qualification

The planning-owned representative fixture is `docs/refactoring-plan/evidence/runner-bootstrap-native.py`; its independent observer and rejection cases are in `runner-bootstrap-extensions.py`, under `runner-bootstrap-extensions-protocol.md`. The combined normal and optimized-Python driver runs are recorded in `runner-bootstrap-evidence.md`: both passed the real native-footer/pointer family within the46 extension observer cases, alongside the other qualification families. The native worker retains assertions enabled even when the outer driver runs with `-O`.

This proves the qualification rule on a real small fixture, not completion of a production Holla adapter or execution of the986 Holla source sites. TASK-070 must still qualify its submitted production executable against these protected groups; TASK-003 must then produce and independently seal the complete actual Holla native/resized map. No Holla baseline or candidate frame was accepted during this planning correction.
