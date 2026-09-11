# Historical authority re-audit

## Scope and method

Planning-only review, 2026-09-11. `PLANNING_GOAL.md` was read completely. Git independently resolved main to `7b27732a8c3c131760ec3438f641cb3c11343a42` and the oracle tag to `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Existing PASS reports and ledger joins were not treated as source coverage proof. The review examined the architecture/component matrices, decision authority, source ledgers and installed task obligations, then checked disputed clauses against Git source. No production source or task payload was changed by this review.

Independent edge enumeration used `git log --all --full-history --format=%H -- <paths>`, actual parents from each commit, and `git diff-tree --no-renames --raw <parent> <commit> -- <paths>`. The comparison key was the complete `(commit,parent,path,before_blob,after_blob)`, including deletion. The nine paths were COMPONENT_ARCHITECTURE, REFACTORING_STATE, REFACTORING_GOAL, IMPROVEMENTS_PLAN, GOAL, GOAL2, JUNIE_PROMPT3, JACKIN_GOAL, and HANDOFF_SLICE4_WAVE1 Markdown files. This is a reproducible source-derived universe, not an enumeration copied from the index.

## Coverage ledger

| Independent subject | Evidence inspected | Result |
|---|---|---|
| Required/related root-document Git edges | 251 actual edges, all parents and complete object IDs | Existing index had 248 exact matches, zero duplicates, three omissions below |
| Architecture matrix and decision authority | All 32 matrix rows; all eight decisions; current public `crates/tui/src/lib.rs` facade | Cross-file Props scope conflicts with one canonical obligation |
| Public API/gate requirements beyond matrix titles | Main architecture §§16.2–16.6, 29.9, 33.5, 39.5, 64, 73; curated runtime, collection, theme and author exports | No claim of whole-API completeness; named-gate absence alone was not reported as a defect |
| Recipe-rule reachability | Main architecture:7274; canonical HM24; task031 obligation | Accounted for: direct matching and inverted stale exemptions are owned, unlike simple state/part coverage |
| Covered TextArea override | Main architecture:6799–6815 and 6997–7003; EARLY-AMEND-044/HM08; task016:15 | Real exception lost in generic task prose |
| Held capture | A12, HL-73-CAPTURE, task010 source payload, main architecture §73 | Implemented preservation status corrupted in derived statement |
| Grid gesture | Main architecture:8310–8315; oracle `src/widgets/grid.rs`:1322–1344 | Accepted older product gesture conflicts with current oracle |
| Deferred paste/permission proposals | Complete oracle F01/F04 documents; oracle TablePro app:237–254 and Grid edit/click paths | Open proposal incorrectly bound as unconditional parity requirement |
| Nine historical ledgers | Canonical source-row links and targeted source clauses | Not an independent full reread of all 620 requirements; numerical self-joins do not establish semantic completeness |

## Findings

### HIST-R01 — deleted research lineage omitted from the supposedly complete index

Location: `docs/refactoring-plan/history-revision-index.tsv:1`; `history-ledger-reconciliation.md` complete-edge claim. Source-derived counterexample: the index omits all three JACKIN_GOAL.md edges:

| Commit | Parent | Before | After |
|---|---|---|---|
| d8f67460f70a410d24794eda0c04e12e2a2edcc9 | 74545e0fdb627cee24cbbab6b405c791c69363c8 | ABSENT | 5bed18785a8fef75855aa4b66a42d6e699ff4e7e |
| f0c262173c74459e21774783c7f4b0ff7e4fe8de | d8f67460f70a410d24794eda0c04e12e2a2edcc9 | 5bed18785a8fef75855aa4b66a42d6e699ff4e7e | a9fda52c26d0a071e320305f064323a996016902 |
| 1e6b2031d9594c01090e70d88a909f5630236ff9 | 8033144886f7737772bef697201c71f71422d8b0 | a9fda52c26d0a071e320305f064323a996016902 | ABSENT |

The rewritten document describes a research-only JACKIN_REFERENCE deliverable, requires separate shipped/planned/unknown evidence, and changes the earlier local/exact-SHA research rules to remote-only observed-reference rules. These are historical research instructions, not permission to redesign current Jackin or to replace the pinned oracle. The deletion must be dispositioned rather than silently treated as no history. Root cause: the completeness claim used a closed path list that omitted a differently named precursor; a join of that list cannot find its own missing path. Repair: add the actual edges and explicit research-only/superseded scope, then independently rediscover linked/deleted document paths when validating completeness. JUNIE_PROMPT1/2 and JACKIN_REFERENCE were also discovered as related paths; this audit does not claim their entire histories have been read or that every one adds an active requirement.

### HIST-R02 — scope-qualified source rule promoted to cross-file proof

Locations: `historical-obligations.tsv:48`, canonical A47, task065 source-obligations:6. Main architecture:2193 explicitly calls Props enforcement an intra-module source scan, not a cross-file call graph; :8577 repeats the limitation. EARLY-AMEND-042 and ADJ02 retain the same boundary. Yet A47 remaining work commands cross-file reach.

Counterexample: a private receiver-free constructor whose external helper lookup is explicitly recorded as outside the bounded scanner can satisfy the accepted bounded contract plus production phase-consistency proof, but fail the promoted whole-program reading. Conversely, an unresolved construction must not silently count as proven. Root cause: an evidence limitation was converted into an unconditional implementation obligation during consolidation. Repair: preserve the original shared-Props requirement; explicitly retain configured constant-ID scope, accepted exemptions, fail-closed parsing, private non-method constructor checks and honest unresolved-lookup reporting. Do not claim or require unadjudicated whole-program proof.

### HIST-R03 — covered-surface proof exception conflicts with generic painted-cell command

Locations: task016 `trusted/obligations.md:15` versus EARLY-AMEND-044 (`source-obligations.tsv:9`). Main architecture:6801–6814 and :6999–7003 explicitly permit the actual selected FIELD part's recorded resolution to change when composed TextArea painting leaves final digest unchanged; theme and sibling invariance remain mandatory. An optional patch_part selector is not an unfinished mandatory API.

Counterexample: the accepted covered FIELD case passes its actual recorded-resolution proof but cannot change a final cell already covered by child content without altering composition. Root cause: universal CP-COMMON prose erased source applicability and conflated resolution with visible override/slot substitution. Repair: source-qualified covered-surface resolution proof for this exception; actual painted-cell proof remains required for advertised visible overrides and installed replacement slots. Preserve both historical clauses; do not weaken all overrides to query-only proof. Component author owns task prose repair.

### HIST-R04 — implemented capture status becomes unresolved in propagation

Locations: `traceability-history.tsv:54`, task010 `trusted/source-obligations.tsv:13`. A12 and HL-73-CAPTURE already identify `715ee0777e20a09e0f373b07024076bdc742ef1d` and publication-boundary reconciliation as implemented, with fresh retention testing owed. The derived sentence ends `Source disposition: unresolved`.

Counterexample: a planner/executor consuming the installed sentence creates duplicate capture reconciliation rather than testing publication, dropped-frame, re-enable and feedback boundaries. Root cause: independently handwritten authority suffix diverged from the canonical authority field. Repair: derive suffix mechanically from current canonical authority; keep historical §73's gap as historical evidence, not present status. No fresh runtime pass is claimed by this read-only review.

### HIST-R05 — historical Grid gesture conflicts with immutable oracle

Locations: `history-late-obligations.tsv:23`, canonical HL-64-GESTURE, task022 source-obligations:30. Main architecture §64 says single click emits Moved and double click/Enter emit Activated. Oracle `src/widgets/grid.rs:1322–1344` computes whether the clicked cell was already current and, on one completed click, begins editing if editable or emits Activated if read-only. The current behavior must not be rewritten to fit the older conformance fixture.

Counterexample: one completed click on the current read-only cell cannot both emit Activated and emit only Moved. Root cause: historical accepted status had no separate current product-authority dimension. Repair: preserve quoted §64 text, supersede its universal Grid product gesture by exact current oracle trajectories, and retain the generic PointerGesture/identity/selection separation. Proof must distinguish initially current versus different cells, editable versus read-only, reorder and cancellation. Task022/031 own concrete proof; the matrix cannot choose one universal click response.

### HIST-R06 — deferred product changes attached to exact parity gates

Locations: canonical F01/F04; `traceability-history.tsv` F01/F04 edges; task017/023 F01 and task022/060 F04 payloads. Oracle F01:7 says Deferred, and :21–27 describes Picker paste falling through to a hidden query. Actual oracle TablePro `app.rs:237–254` special-cases Dialog and Filter only. Oracle F04:7 says Deferred; :21–28 records direct DataGrid begin-edit → editable=false → paste → commit still writing. Its heading also says current app transition is not established.

Counterexample: imposing exhaustive modal-first paste changes the observed TablePro Picker trajectory; imposing cancel/no-write on the direct permission-transition sequence changes current public-widget behavior. Absence of an established Holla editable-grid path does not prove absence of the public sequence or authorize its change. Root cause: retaining every improvement proposal was conflated with selecting every proposal for implementation; generic readonly language can reintroduce the same conflict through RG39.

Repair: retain proposal text with explicit deferred-not-selected disposition, no independent proposed-behavior PASS claim. Current application parity uses oracle behavior. The legacy direct-widget defect is archived evidence, not a requirement to restore unchecked mutation in modern Grid APIs: accepted main entry-point/readonly guards remain. Source search finds TablePro's `editable` assignments only at `tabs.rs:395` and `:2069`, both immediately after a new DataGrid; column readonly configuration at `:309` also constructs ColumnSpec. No Holla assignment or app input transition changing permissions during an active Grid edit was established. This is a bounded source finding, not an assertion that no conceivable indirect path exists. Any specific app compatibility exception requires a real source-qualified reachable trajectory. Any future selection of the deferred fix requires explicit authority, not a candidate-authored applicability waiver. RG39's ordinary readonly navigation/nonmutation remains required; do not turn broad historical wording into a mandate either to fix or reproduce an unmapped legacy direct-API defect.

## Source continuation

The initially unread JUNIE_PROMPT1/2 and JACKIN_REFERENCE histories are now completely read, together with their meaningful local DESIGN.md authority link. `history-source-continuation.md` records exact read blobs, all historical changes, clause-group dispositions, current source counterexamples and concrete existing task owners. Thirty newly discovered edges have been appended. Independent re-enumeration now finds281 actual/indexed edges across13 paths, zero missing/extra/duplicate identities. This closes the named follow-up source reads, not the whole620-clause semantic re-audit by arithmetic. Original research backend requirements, obsolete one-pass APIs, removed early exceptions and conflicting design prose are explicitly dispositioned; no new production feature or task payload is introduced.

Further linked-source continuation fully read Lane B/C, documentation findings, Slice6/7 plans and PLAN-FINDINGS through all twelve distinct historical edges, recorded separately in `history-inputs-read-index.tsv`. Q residuals and all four patches were also independently reread. The continuation report maps their complete clause groups to current owners and explicitly rejects historical permission to change Radio/Tab gestures, sort semantics, status expiry or fade collisions. Existing TP-0764999/5001ms checkpoints and HL-54-DIM parity qualification were checked as current defenses, not inferred from a coverage count.

## Repair verification boundary

Parent authorized narrow shared history-ledger/index/traceability repairs after this report. Task payload synchronization and validator changes remain parent-owned; component agents own task-top-clause changes. A repaired history index proves only exact edge accounting for the enumerated path universe. It does not certify that every source clause or public API requirement is semantically covered. All findings require source/current-disposition agreement before plan closure.

Post-repair focused verification: independent Git enumeration and the repaired index both contain 251 edges, with zero missing/extra/duplicate identities. The 196-row global, 620-row canonical, 60-row late and 56-row goal ledgers have uniform TSV schemas and retain their row counts. At verification, the concurrently maintained trace contained 1,094 rows; all targeted A12/A47/HL-73-CAPTURE/HL-64-GESTURE/F01/F04/RG39 authority suffixes matched the canonical authority, and every F01/F04 edge used R-004/AC-004/CHK-001 disposition rather than a proposed-behavior parity gate. No task payload synchronization or production test was run by this reviewer. Those are separate pending parent/component-owner actions, not a failure or PASS inferred from historical test counts.
