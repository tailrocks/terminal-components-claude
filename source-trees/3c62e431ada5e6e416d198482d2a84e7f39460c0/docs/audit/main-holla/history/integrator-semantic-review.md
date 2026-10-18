# Normative amendments: fully reviewed patch ranges 22–34 and 54–67

These 27 parent-relative patches were reviewed through every added/deleted line with hunk headers, 215,628 characters in ten contiguous bounded reads. Necessary full context was also read for the earlier gate records. `snapshot-reconstruction.json` proves exact parent+patch reconstruction for all 68 history edges; unchanged snapshot text is inherited, not claimed repeatedly reread. This report complements earlier/later agent ranges.

## Revision dispositions

| Index / revision | Semantic disposition and surviving obligation |
|---|---|
|22 a3fe79b|Correct numeric/report claims, distinguish OWED from DONE. Require all documented numbered sections and checks to be within gate scope; missing Select coverage was historical and subsequently corrected.|
|23 2c848f9|Require registry-derived suite cases, exact declared/styled parts, exact thirteen named examples, signature/public visibility verification, complete public author sizing vocabulary, shifted chord lookup. Count>0 or name-only resolution insufficient.|
|24 2399c1a|Requires legacy per-test disposition before deletion, actual public facade boundaries and source-accurate DESIGN; review with preregistered rejection conditions and demonstrated red inputs. CLI color conflict later resolved §74 as ceiling.|
|25 1b580d7|Reject coincidental example count and assertions only in unexecuted mains. Specificity proof must distinguish specificity from last-write order. Seed token cascading explicitly undecided; do not infer cascade requirement or assume current no-cascade semantics correct without adjudication.|
|26 e6eaca4|247 legacy tests partitioned historically (76 library,33 Showcase,41 TablePro,67 Jackin,30 perf); require surviving mappings including secret-containment tests, clickable Brand, StatusBar geometry, Meter block, binding perf allocation guard. Counts are baseline evidence, not current required totals.|
|27 de817f2|Invariant R: documented SlotFn-addressable parts equal parts where slot changes actual cells under fixture sweep. Slots substitute content preserving interaction/geometry; busy spinner glyph Slot is inert while SlotFn replaces renderer. Preserve component/row attribution and forwarding through nested components. Retires obsolete missing Select claim.|
|28 3dc44c4|Records impossible original Slice-5 rename; no source deletion until consumers migrated. All 22 pages need component dependencies. Gate presence and exact test accounting mandatory; chronology later superseded by §47.|
|29 3646897|Style disabled-vs-hover conflict and complete suppression distinct. Marker probes needed when real patches are disjoint. Brand/StatusBar gaps were coverage gaps confirmed nondefects. Dynamic runtime suppression proposed, not then decided.|
|30 b60c828|Defers rename through Slice7, atomic removal of old bin when adding app, no unmigrated staging, single public lib/bin ownership. Frozen pre-refactor baselines stay at old paths; new app baselines separate. Exact names are multiset; missing expected app fails. Historical slice steps already passed, durable invariants survive.|
|31 444ae3d|Named-test scan expands §16.5; stale and missing deferrals fail. Cargo autodiscovery means removing bin stanza alone insufficient. Rule25/legacy allowlist replaces nonexistent apps/allow/dispatch.txt. Failures need actionable diagnostics.|
|32 4d26211|First-generation exemption consumed once even if original bless invalid; cannot launder keys via revert/base selection. Truecolor movement requires scoped approved item; first-generation tags cannot authorize movement. Live declared blockers block bless; push-main must use real preceding base, never HEAD self-diff. Readiness distinctions need value tests even on first generation.|
|33 3adb6ef|§§50–56 detailed below. Removes old unresolved radio/add/StatusBar claims and changes chip SELECTED to actual CHECKED.|
|34 14bca4a|§§57–59 detailed below.|
|54 0f01836|Corrects implementation prose without proving obligation fulfillment: doc-check syn/regex not rustdoc/signature/public proof; StyledBy attribution absent and target ownership still owed; PARTS exact component ownership survives. §72 supersedes production forced builders; historical hints hook removed.|
|55 82158df|Required HintLayer passed to HintBar; optional selection belongs resolve/DerivedHintBar. Wording correction, no behavior redesign.|
|56 e49de3f|Public FormState errors generic Invalid value; only Form's private current-owner-aware render path can show nonsensitive detailed errors. Reconcile dynamic hidden/inactive owner sensitivity.|
|57 0369742|§73 disabled selected top hit absorbs activating pointer without lower search; move/hover observable, presentation strips hover/press; wheel remains routed; active capture disabling expressly unresolved. Shared configured props across phases with gate limitations explicit. Inert/reference registration centralized.|
|58 c936d51|Sensitive constructors required before direct begin can copy secrets; Secret::expose crate-private; mask policy explicit. Zeroization best-effort safe Rust, not cryptographic guarantee.|
|59 91f0296|Hidden fields preserve draft only if identity/shape/sensitivity unchanged; transitions zeroize before visibility filtering. State errors private typed ErrorState, reveal Option; reconcile with current owner data.|
|60 3f26ab1|Owner must zeroize FormState on cancel/dismiss because Form does not own layer lifecycle. Removes FormState Reconcile claim. No clone/value bag secret boundary.|
|61 cd70747|Conformance registry corrected to actual 44 cases incl Probe, DerivedHintBar, Empty; old separate Chip case absent. Changes inventory evidence, does not authorize omitting capabilities.|
|62 43a147f|Adds Choice/Brand/Menu pressed geometry test names; Choice claim reversed next.|
|63 1537144|Corrects exact Cargo filters and fixture Option reference state, marks Choice unresolved, limits shared reserved-pad bracket gate, records TextArea covered FIELD resolved-style fallback. Hook patch_part optional future ordering hardening; does not substitute for user-mandated visible sentinel overrides.|
|64 2a0cf29|Correct grep syntax `Option<StateFlags>,` and bind report to tested source, not moving HEAD. Command correction only.|
|65 f01703a|Separate application ActionKey 0x4000..7fff from custom component 0x8000..ffff. Prevent cross-class collisions; no proof of within-class collision freedom.|
|66 efada04|Historical header preserves architecture while active execution changes. Current user controls main-base/four-app scope.|
|67 8514210|Truecolor scope only for full-height ScrollRegion/Grid transitive geometry; no unrelated mono/component drift authorized.|

## Concrete component amendments

1. ChipBar: real item actions carry ItemKey; AddRequested is payloadless, Part::NEW unkeyed, no sentinel collision. Six parts CONTAINER/MARKER/LABEL/CLOSE/OVERFLOW/NEW; checked-set CHECKED owns reserved glyph. SlotFn support exactly CLOSE/OVERFLOW. MARKER glyph Set/Inherit/Clear works; no synthetic SELECTED. Component patch passes only CONTAINER/LABEL into caller RowUi; META/CELL/custom row parts stay row-owned; explicit label_patched wins.
2. Radio: immutable optional value ItemKey prop same in both phases; update only seeds missing cursor, established cursor does not track external value; draw uses live value for RadioOn; Chose asks caller to update value.
3. StatusBar: only keyed labels have addressable hover; live matching item alone hovered; keyboard suppresses until pointer move. Static unkeyed labels are unaddressable, not disabled. Reference forced hover affects labels but registers nothing.
4. Grid: domain adapter owns comparisons/display ordering; Sort(ColumnKey,SortDir) only request. Stable keys preserve cursor/range/check/edit across reorder. State exposes only owned properties. register_focus_only deliberate separate no-hit API, ClickOnly inert, area None, pointer diagnostic.
5. Tree: runtime derived keyed incremental index, no text/T ownership. Initial/source/query change scan once; steady update/draw O(viewport); toggle proportional subtree. Query projection includes only matches/ancestors independent of persisted expansion, never mutates expansion; clearing query restores it. Query revision caller-owned; accessor-count gates mandatory and all named perf tests survive.
6. Containers: Panel/Dialog body FnOnce returns bare R and executes once even zero/tiny, anchored inside original rect and clipped to empty under proper surface. Ui::layer alone returns Option. SplitPane sole two-rect single-closure exception: logical ordering, seam excluded/preserved, no public geometry bypass.
7. NavList: Right/plain l emits EnterContent; Enter/Space/click Chose; only shell performs content focus. Full headings and blank group separators; collapsed icon-only blank separator. Exact slot set GUTTER/MARKER/ICON/HEADER/BADGE; own patch CONTAINER/LABEL forwarding and own BADGE geometry.
8. Steps: skipped READ_ONLY stays navigable/activatable and terminal; only whole rail disabled blocks. Seed no Moved, boundaries consume without repaint/state write, stable actions. Own lifecycle META and delegated scrollbar overrides. Derived frontier runtime cached with stamp/key/state validation; monotonic advance proportional, regression/reorder/reset requires invalidate; saturation recomputes; accessor gates binding independent PERF_STRICT.
9. Runtime time: Bootstrap/Event/Tick/Settle with tick only initial pass; earliest exact persistent repaint deadline, no unsolicited idle ticks, product app chooses delta; no generic design token replacing product time. §54 addendum in late range further specifies.
10. Reference/inert: central suppression covers controls/focus/editor/parts/decor/scroll/bindings plus reference focus scopes/layers/layout/cursor; reference still paints/style-resolves and restores nested outer target. Disabled fresh-hit absorption must include overlapping lower control test and explicit active-capture policy.

## Unresolved decisions must not be silently weakened

- Choice mono bracket geometry; reserved padding approach must fit real Choice design and pass actual visible glyph/style tests.
- Dynamic disable during live capture: §73 expressly leaves unspecified; current user requires correct disabled/capture behavior, so define/test safe cancellation or retention semantics.
- Seed semantic-token cascade (syntax diagnostic/meter roles) remains undecided in this range.
- StyledQuery has no separate row provenance; PARTS ownership/override isolation requirement remains even though obsolete mechanism name is removed.
- doc-check is intentionally described as weaker than original proposal; public signatures/examples and testing boundary still require external compile proof. A prose correction does not close capability gap.
- TextArea FIELD resolved-style fallback verifies a queried covered surface, not necessarily visible user customization. Current user explicitly demands actual intended cells change; add suitable visible sentinel cases.

## Current-state acceptance discipline

Historical pass counts, code-present reports and signed old reviews are historical evidence only. Run exact tests on candidate, enumerate before execution, reject zero matches, bind output and image reviews to immutable binaries/source/artifact hashes. This range introduces no authorization to reset main to Holla or retain legacy component API.
