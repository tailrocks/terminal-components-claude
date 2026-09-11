# Historical ledger reconciliation rules

The ledgers form one evidence set. Rows are never silently deleted because another row mentions the same component. `historical-obligations.tsv` is the compact global semantic index; supplemental ledgers retain independently enforceable source clauses, exact test obligations and supersession evidence.

`historical-obligations-canonical.tsv` is the lossless row-identity union: **620 unique IDs** from all nine source ledgers (196 global +54 later-prefix amendments +60 early +13 inline Form +44 HAR +66 HI +60 late +71 middle +56 goal). It normalizes the eleven required matrix columns and adds the exact original ledger line and relationship. Original source ledgers remain the detailed authority for additional rationale, named evidence and corrections; every substantive row is retained, including duplicate-equivalent and rejected/superseded rows. One trailing blank source record was excluded, not treated as an obligation. Current implementation status is copied when established; otherwise it explicitly says unverified and names the architecture/app matrices for the remaining join. No historical green count is promoted to current status.

`history-task-map.tsv` assigns every620 row to actual task IDs, including the separately owned comparator, runner, test-accounting, scanner and attribution bootstrap boundaries. `traceability-history.tsv` expands them to1084 exact HIST→task→requirement→acceptance→check edges with semantic roles. No unknown task, missing source, duplicate edge or blank field remained in the historian's integrity check. Existing task-bearing source ledgers and the canonical matrix carry matching task-ID sets. `trusted/source-obligations.tsv` in each package is the full-payload projection: namespace/source ID, all13 canonical columns and exact requirement/acceptance/check/role/disposition. All73 packages now contain this20-column asset; all1084 installed rows were compared field-for-field against the canonical source and trace edges with zero mismatches. A header-only HIST file means that package's source obligations come from its other APP/COMP/ARCH/decision assets, not that the package has no acceptance requirements. README binding and whole-catalog validation remain separate author/plan-validator checks; IDs or installation alone do not certify those checks.

Shell contribution qualification is not whole-application closure. A127/F23a additionally map to TASK-039/050/057/064, and RG43 includes TASK-057. TASK-032/040 semantic shell obligations use their actual `R-001/AC-001/CHK-006` contracts; HP16's existing preview CLI uses `AC-008/CHK-004` while its full live list/run/doctor proposal stays forbidden. TASK-067 observable perf obligations use its actual `R-001/AC-001/CHK-006` contract. No HIST edge targets TASK-006; other source namespaces own that baseline task's component corpus.

| Namespace | Role | Join rule |
| --- | --- | --- |
| A01–A130 | Accepted architecture and completion-proof semantic anchors | Historical revision points to exact cumulative source commit plus section; supplemental rows supply originating/amending source commits |
| F01–F23h | Concrete Improvements defect/proof proposals with per-row current authority | F01/F04 remain deferred and not selected; current oracle behavior is preserved. Other rows require their own accepted/deferred disposition. Do not confuse with Form F1–F13, which uses `INLINE-F*` |
| HP01–HP23 | Required simulated Holla product capability ownership | Real providers, persistence, deletion and full CLI execution are not silently added to refactoring |
| O01–O07 | Superseded generic/framework proposals or conditional product extensions | Rejected framework does not reject its concrete F/HP capability |
| P01–P03 | Historical interaction polish | Immutable current oracle decides observable behavior; no independent redesign permission |
| EARLY-001–060 | Direct original/adjudication A–Q semantics and early-prefix amendments | Adds retained and superseded subclauses; umbrella rows do not replace finer obligations |
| HM01–HM71 | Direct §§30–55 and corresponding state/review corrections | Includes finer PARTS/provenance/guard and legacy visual assertions absent from global summaries |
| HL-* | Direct §§56–75 and unnumbered late amendments | Includes per-part, capture, timing and proof refinements; source gaps explicitly remain work |
| RG01–RG56 | Original REFACTORING_GOAL capability/invariant obligations | Ancestor requirements continue unless a later accepted decision narrows/replaces them; old execution order is not current plan |
| HI-001–HI-066 and HAR-001–HAR-044 | Linked audit/review source and early STATE evidence | Proposals are not authority over accepted adjudications; preserved functional facts still require implementation/proof joins |
| INLINE-F1–INLINE-F13 | Individual original Form invariants | Each explicitly maps to global A rows; all13 retain their separate acceptance clauses |
| Later early-amendment ledger | Post-§29 edits to older source sections | Current decision wins over obsolete inline sketch; retain old row as superseded provenance |

## Explicit duplicate-equivalent groups

### Current authority corrections from the independent history re-audit

`reaudit-history.md` records the source counterexamples, not merely a failed join. Historical requirement text remains unchanged. A47 is bounded intra-module Props proof, including explicit exemptions and honest unresolved cross-file limits. A12 and HL-73-CAPTURE are implemented-retention verification, not unresolved policy. HL-64-GESTURE retains reusable gesture/identity separation while its universal Grid single-click-Moved behavior is superseded by current oracle same-cell activation/editing. EARLY-AMEND-044/HM08 retain the covered TextArea FIELD resolution exception; this does not weaken advertised visible override and slot substitution proof. F01/F04 are deferred-not-selected proposals and their task edges are disposition checks, not proposed-behavior parity checks. RG39's ordinary readonly invariant cannot silently select the separately deferred F04 mid-edit behavior change.

After a source-authority correction, previously installed row-equality counts describe the earlier snapshot only. Parent-owned synchronization must regenerate task payloads and derive each source-disposition suffix from the canonical authority field, then revalidate all affected edges. No hand-authored suffix may contradict the source row. Source-first history enumeration must include deleted/renamed related documents; the three JACKIN_GOAL.md edges now bind the research-only disposition in history-other-docs.md, without inventing an active implementation task for an obsolete research procedure.

These groups share a semantic contract. They may map to one task, but every cited row remains in that task's source obligations. A global row can include additional clauses beyond the duplicate group, so equivalence here does not permit dropping the rest of that row.

| Equivalent semantic clause | Rows |
| --- | --- |
| Choice direct per-phase route/private Form bridge; no mandatory trait widening | EARLY-017; HM71; HL-67-FORM; A83/A84 |
| Payloadless ChipBar AddRequested and real CHECKED | EARLY-057 resolved portion; HM55; A64 |
| Caller-owned RadioGroup value separate from cursor | EARLY-057 resolved portion; HM56; A65 |
| Keyed StatusBar hover only on live label | EARLY-057 resolved portion; HM45; A66 |
| Runtime bootstrap/update causes | HM62; HL-54-CAUSE; A97 |
| Persistent earliest repaint deadline | HM63; HL-54-CAUSE; A97 |
| Jackin simulation delta is product-owned, pause freezes it | HM64; HL-54-DOMAIN; A98 |
| Pure status projection into fixed caller-owned storage | HM66; HL-54-STATUS; A99 |
| Inactive Manager/Accounts inherent message fanout | HM67; HL-54-FANOUT; A100 |
| Total semantic dim and identity at zero | HM65; HL-54-DIM; A101 |
| Hidden scrollbar keeps wheel/reveal and releases thumb capture | EARLY-038; HL-SCROLL; A105 |
| Detailed Grid gutter/header extension with unchanged compact default | EARLY-058; HL-65-PARTS; A57/A59 plus extension clauses |
| Meter ReferenceLift narrow authored source policy | EARLY-030; HL-METER-POLICY; A37 |
| Authored capability palettes and sticky projection history | EARLY-029; HL-METER-PROJECTION; A38 |
| Full ASCII table, single-byte/width-one proof | EARLY-031; HM15; HL-71-ASCII continuation; A91 |
| No Ui::scroll_region convenience; direct owner composition | EARLY-037; HM13 |
| Frozen evidence remains immutable after producer removal | EARLY-052/056; HM18/HM47; HL-74-BASELINES; A113/A115 |
| Shared GridModel versus mutable GridEditor entry points | EARLY-018; A55 |
| Form values/options borrow under one access | EARLY-015; INLINE-F1/F5; A80/A83 |
| Draw cannot commit/validate/change semantic state | EARLY-001; INLINE-F6; A01/A02 |

## Refinement, not duplicate

- HM05 defines exact component-owned PARTS equality; HM06 requires conditional fixture sweep; HM07 requires registry-derived enumeration; HM09 requires truthful observation; A43/A44 summarize this family. All four obligations survive independently.
- HM08 demotes a specific `patch_part` hook to optional future ordering hardening. It does not revoke HM48's proof that every advertised SlotFn actually changes the appropriate painted cells.
- HM20/HM21/HM25/HM26/HM28/HM30 are distinct parser, dependency, whole-document scope, registry, executable-example and testing-crate proof requirements. A108/A109 cannot be treated as one name-existence test satisfying them all.
- HL-73-HIT and HL-73-EXCEPTIONS are accepted admission semantics. HL-73-CAPTURE retains a historical prose gap, but `715ee077` already implements publication-boundary cancellation. Its current disposition is retention verification, not a new policy/implementation task; direct-delivery inspection alone had missed the reconciliation boundary.
- HL-METER-PINS records current changed-preservation integers needing investigation. It is neither duplicate policy nor authority to bless them.
- INLINE-F1–F13 atomize EARLY-014/015/016. Declaration order, hidden retention, pure height, no-value actions, draw purity, first-action arbitration, layer independence, submit order, Enter arbitration, dirty semantics and secret clearing each require their own proof.
- Historical green test counts, first-generation hashes and merged status are evidence of what was claimed, not duplicate completion proof. HM70 and A128 explicitly retain later contrary evidence.

## Supersessions that must not become implementation tasks

Initial universal/fused component proposals; old generic worker/keyed collection/container framework proposals; Grid editable(bool)/GridCellActions/mandatory Editor bound; data-bearing choice constructors; generic or dyn-erased FieldKind; binary-only app packages; automatic Unicode capability detection; local forced-state APIs; immediate opener FocusIn before publication; synthetic perf stand-ins after real apps exist; changing reference pins/baselines to remove mismatches. Follow the accepted replacement, not both mutually exclusive historical versions.

Canonical task authoring must assign every remaining supplemental clause (including retained behavior needing regression proof), leave completed obligations as retention anchors, and label equivalent rows explicitly. This reconciliation is not a claim that a shared test name proves two workloads equivalent.
