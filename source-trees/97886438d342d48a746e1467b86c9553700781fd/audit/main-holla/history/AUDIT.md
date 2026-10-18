# Historical obligation audit — completed document semantics

Pinned source: c12cad8728755cd2d03eefdd8e02891143fca86d. Scope is both contract paths reachable from this commit, not Holla-only history (another owner).

## Exhaustiveness

All 484 reachable commits were enumerated with `git rev-list --parents`. Both paths were inspected in every commit and every parent tree, including root absence. Result: 976 parent/path rows, 68 changed parent/path edges, 64 unique document blobs, 105 rows whose child path is absent, zero deletion edges. Whole-tree `diff-tree --root -m -r --name-status --find-renames` cross-check found no rename touching either filename. No former path was discovered by `--follow`. Each changed edge has an individual unfiltered patch; every unique snapshot has its Git blob ID as filename. INDEX.md links all patches. 3 merge commits contribute 4 changed parent/path edges; unchanged merge-parent comparisons remain in tree-audit.json.

Mechanical and semantic coverage of both reachable document paths is complete: all68 changed parent-relative edges and64 unique snapshots reviewed by original creations plus every novel addition/deletion and necessary context, with exact repeats inherited by explicit induction. All68 patches exactly reconstruct authoritative child snapshots from parent bytes (`snapshot-reconstruction.json`). Repeated full-snapshot rereads are not claimed. `semantic-coverage-final.json` binds each edge to its reviewer report and hashes. Initial truncated reads were replaced by bounded complete reads. Independent coverage integrity audit passes (`late/coverage-integrity.md`): zero index gaps/overlaps, zero unseen parent snapshots, all endpoint Git hashes correct. Current gaps below are source-proven or explicitly historical/unverified; no builds or behavioral reproductions were executed by this audit. Broader user§3 also requires other owners' DESIGN/Holla-only and implementation mappings.

## Authority and semantic deltas already established

- e48137f creates reusable architecture goal; dce91d5 removes obsolete goal references; 2d81eec adds historical model routing; 25ea92b permits flexible research tooling. Current user supersedes routing and expands three applications to four.
- 2e45302 creates architecture §§1–15: caller-owned state, immutable draw, semantic actions, stable identity, runtime interaction, semantic themes, borrowed collections and domain separation. e8d053c adds tests, complete migration inventory, performance obligations and package staging.
- 95ab652 adds review corrections and exact API/ordering/testing requirements. 27bd918 adds backend/core/dependency/MSRV and Form/Grid contracts. 87ab93d fixes public author exports, ASCII borders and FieldKind.
- 587c53b records foundations defects F1–F26 plus layer sizing/measurement corrections. 4aabceb changes memo/performance methodology and ASCII/ANSI details. dc3e0fa adds fixture/overlay/readability obligations.
- 70dacec through 444ae3d repeatedly identify decorative gates: partial document scan, false/pending claims, nonexistent test names, explicit call lists, weak override checks, missing test mappings and package relocation blind spots. These are historical defect reports, not proof they remain unfixed.
- b60c828 supersedes initial rename-at-Slice-5 order with rename after Slice 7. Later mechanical global rename patches rewrite historical names to self-renames; those edits do not create a meaningful new architectural invariant.
- 3adb6ef adds semantic choice keys/AddRequested, caller-owned Radio value, keyed StatusBar hover, Grid request ownership, cached keyed Tree indexing, monotonic application time/status, exactly-once container closure traversal and SplitPane's two-slot exception.
- 14bca4a adds NavList choice versus enter-content distinction and stable-key incremental Steps behavior.
- a1759b2 adds §§60–72 covering Tree selected marker, single Grid schema/ragged cells, geometry-neutral mono text selection, TooSmall hierarchy, meaningful conformance, local readiness, isolated performance targets, pointer/form bridges, stable binding identity, reveal/press ownership, borrowed Picker performance, caller-owned viewport work probes, Select disclosure glyph and exact-target reference rendering.
- ecc1337 elaborates Bootstrap/Event/Tick/Settle, runtime bootstrap, persistent earliest exact deadlines, app-owned tick delta, monotonic dimming and fixed-buffer pure status; CLI color is a ceiling and capture matrix required before review.
- 0defae2 capture-hardening document amendment is reversed by 067869c; later ec3d760/e524cea merge corrected changes. Parent-relative patches preserve each reversal and merge edge.
- 15ecde1/0f01836 reconcile prose with implemented instrumentation. In particular proposed StyledBy row/component attribution is acknowledged absent; changing wording is not behavioral proof of the original ownership requirement.
- e49de3f, c936d51, 91f0296 and 3f26ab1 preserve dynamic form error masking and environment draft security. User explicitly preserves valid main-only safety improvements.
- 0369742 adds §73 disabled pointer absorption, common props and inert-safe registration; historical section includes explicit evidence gaps.
- 1537144 corrects conformance filters (target module paths have no conformance:: prefix), fixture forced state Option semantics and test inventories; explicitly leaves Choice mono in-run bracket unresolved §29.7. This is not permission to omit Choice.
- f01703a separates application ActionKey range 0x4000..=0x7fff from component custom range 0x8000..=0xffff.
- efada04 adds a historical header to REFACTORING_GOAL; it does not remove architectural obligations. 8514210 limits new scroll/Grid geometry exception to truecolor; unrelated mono or component drift remains unauthorized.

## Obligation map

Status legend: SOURCE = inspected source, HISTORY = historical contract/patch only, OPEN = acceptance not proven.

| ID | Original → valid amendment | Current implementation / gap | Concrete acceptance proof |
|---|---|---|---|
| H01 | Goal 11; architecture 3–5 → §55 closure traversal | Public update/draw split exists; immutable refs alone cannot prove absence of interior mutation/effects. OPEN | Repeated production draws preserve all durable state, clock, overlay state and effect counts; adversarial draw mutation fails |
| H02 | Goal 12; architecture 7 → §50/58/68 keyed actions | Stable item IDs and separate ActionKey ranges; app-level selection still needs inventory. OPEN | Insert/delete/sort/filter/reorder selected and focused items; identity remains logical; colliding display positions do not redirect |
| H03 | Goal 13; architecture 3.3/8 → §54/73 runtime ownership | SOURCE Showcase paints second sidebar over NavList registration at app.rs:1238 | Reference-coordinate click on every visible row, gaps, clipped rows at 80x24 and heights 30/31; one authoritative layout |
| H04 | Goal 13; §54 + ecc1337 exact deadlines | SOURCE buttons.rs:202 decrements busy_frames on every update; event-count duration violates elapsed time | Busy at 2199ms, exactly one completion at >=2200ms, 1000 events at fixed time do not finish; idle does |
| H05 | Goal 14; architecture 9 → §26/29.8/73 | Dialog overlays must preserve traps, popover focus-out dismissal, disabled absorption. OPEN | Nested modal barrier, background wheel/click/type blocked, close/restore focus, removed opener and zero-size layer |
| H06 | Goal 17; architecture 10/26 | SOURCE 8831a62 references missing overlay_chrome; prior inline implementation supplies full contract | Build plus dialogs with errors, tiny rects, elevated surface, borders/decor registration, clipped body/actions; no no-op helper |
| H07 | Goal 15; architecture 11 → §25/31/34 | Theme pipeline exists; actual color/state parity unproven | Exact Junie token/style/cell comparison in truecolor/256/16/mono and independent Paper coverage |
| H08 | Goal 15–16; §33/39/45/50 | SOURCE RowUi query recording has no StyledBy provenance; historical attribution proposal superseded in prose only | Sentinel family/variant/scope/instance/part/state overrides touch intended cells only; custom row parts separately attributed |
| H09 | Goal 18; §52/53/59/61 | Grid/Tree/Steps generic APIs; product capabilities need adapter proof | Borrowed 10k-item datasets, stable cached indices, ragged rows, custom renderer receives real data; binding alloc budgets |
| H10 | Goal 19; §23/67 + security corrections | Dynamic masking and draft protection are main-only obligations | Adversarial secret validation never leaks through text/cells/Debug/errors/logs/artifact metadata; save semantics remain |
| H11 | Goal 23 Scenario G; author §24/41 | Public author module exists; historical inaccessible sizing APIs were reported | Compile independent consumer without private imports/workspace feature unification; custom overlay, focus, capture, cursor, theme |
| H12 | Goal 21/22; §47 staging | Legacy root removed 7784719; four-app completion not implied | Exactly one binary each showcase/tablepro/jackin-preview/holla; no legacy facade or duplicate generic app controls |
| H13 | Goal 22 Showcase | Migration 4e07ea1, later facade 1378c31; restored/sidebar behavior 295c9d3 | All 22 pages, shell/inspector/footer/status/help/editing, all routes/input/viewport combinations |
| H14 | Goal 22 TablePro | Migration 5042a40; source size/stats do not prove preserved behavior | All reference workflows including completion, pending/null/default, undo/revert, validation, SQL preview and safety |
| H15 | Goal 22 Jackin | Recovery 444a8f4; 43a147f/f01703a later stable state corrections | Every scenario/journey, FirstUse default, aliases/help exits, explicit/env precedence, motion empty/0/1 |
| H16 | Current user expands Goal 22 to Holla | Separate owner auditing forward port | Eleven scenarios, 132 captures + missing color/state, ranking/context/activity/DAG/typed confirmations, simulations only |
| H17 | Goal 25; §28/38/39/64 | Historical zero-match filters and narrowed states could pass vacuously | Enumerate exact identities before execution; missing/zero-match required test fails; fixture forcing preserves props |
| H18 | Goal 25/26; §36/49/72/74 | First-generation hashes are drift pins, explicitly not approval of appearance | Actual immutable reference cells/cursor/images; before/after/diff hash-bound independent review; stale/missing capture fails |
| H19 | Goal 25.6; §20.9/25/27/66 | Historical figures corrected 6ec2917; no fresh measurements here | Same pinned workloads, isolated binaries, declared alloc/byte checks binding, monotonic elapsed metrics; failing pipeline exits nonzero |
| H20 | Goal 26/29; §40/42/48 | Historic checks omit declarations/new scope; current fix completeness OPEN | Public named inventory equals executable checks; test mutation missing name/recipe/reference/secret/hitbox/timing/style detected |

## Root-cause provenance

SOURCE: 8831a62 replaces inline complete dialog chrome with import/call absent from shared module; API extraction landed without its provider. Full patch saved as 8831a62-implementation.patch. No build result claimed here.

SOURCE: caaf792 adds busy_frames; 295c9d3 later changes it. Current source decrements on every update. Shared clock contract already existed (§54); app migration introduced an independent event counter, and tests failed to bind behavior to clock.

SOURCE: 295c9d3 introduces paint_sidebar. Current draw composes NavList registration then paints alternate geometry. Architectural enabling condition is independent painting allowed over registered component geometry. Behavioral coordinate reproduction belongs to application audit.

HISTORY: 7784719 removes root library without creating missing application equivalence proof; 4e07ea1/5042a40/444a8f4/1378c31 are relevant migration/recovery commits. Saved patches are investigation evidence, not comprehensive reviewed proof of every regression's introduction.

No targeted executable bisect was run. Current build failure would confound unrelated behavioral bisects; isolate that compile repair before comparing actual binary behavior.
