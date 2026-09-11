# Historical audit inputs and early state reconstruction

This evidence supplement covers the initial linked audits, review adjudications, and `REFACTORING_STATE.md` chronology through `70dacec1dd527218bdba394675938a9e9c4de285`. It is planning evidence, not a claim that current production code satisfies a historical check.

The pinned architecture source is `7b27732a8c3c131760ec3438f641cb3c11343a42`. The user-visible acceptance oracle supplied by the coordinator is `02f5294bfdbf38004cc49130d0aff1d01f31434c`. This reader did not independently resolve the tag. Historical permissions to change or bless visuals do not override the current planning goal's strict oracle parity.

## Reading method and honest coverage

Sources were inspected with `git show <commit>:<path>` and zero-context parent diffs. Prior `docs/audit/main-holla/history/*` semantic reports were not used as source proof. `git log --all -- <path>` supplied the revision inventory. A displayed heading is not a completed section read. Any truncated full-file call is treated as incomplete unless a subsequent bounded read covered the missing range.

The companion [obligation ledger](history-inputs-obligations.tsv) records the decisions established by these reads. `current_main_status=not independently tested` is deliberate: the architecture auditor must join these obligations to present source and tests. `task_ids=PENDING` prohibits an unsupported claim of an execution-ready graph. The [direct-read edge index](history-inputs-read-index.tsv) records exact parent and before/after blobs for this reader's 18 linked-input edges and 51 early STATE edges. For an immutable input, the main blob is the original blob. For an amended input, the full main body plus every parent delta reconstructs all meaningful historical versions; the index explicitly distinguishes that method from independently rereading the original blob.

### Directly inspected source blobs

All paths in this table are relative to the repository. Blob IDs are resolved at pinned main; historical edge completion is separate from reading that blob.

| Path | Blob at pinned main | Lines | Direct read coverage |
|---|---|---:|---|
| `docs/audit/api-audit.md` | `776b9927499b85633f2e6db5a1e4895374430a97` | 1312 | Complete bounded body and every edge read by the disjoint child reader; see child source record below. |
| `docs/audit/app-audit.md` | `2d7b8831135ae7dc308fcf0f01be52c2c7d9afc6` | 685 | This reader read 415–575; child read 1–414 and 576–685 plus original whole body and every later edge. |
| `docs/audit/architecture-research.md` | `a0c7d7700837b7bdea4e50ee63aadd211a139755` | 1094 | Complete bounded body read by child; original/main blob equality verified. |
| `docs/audit/domain-boundary-audit.md` | `3ac2d20ffc7719af92549b4fa982786c8c2dfc38` | 582 | Complete bounded body read; original blob identical. |
| `docs/audit/interaction-audit.md` | `193c27c8182266a090f7ae534e5f024c59f624ce` | 655 | Complete bounded body read; original blob identical. |
| `docs/audit/modern-api-audit.md` | `249c491a4e3017e301801e5744f24fc202fafe23` | 642 | Complete bounded body read, including reread of §2.3 width gap; original blob identical. |
| `docs/audit/performance-audit.md` | `9f547dae90bd36cca1040cb5992d912f6b61a1b7` | 576 | Complete bounded body and sole header-only edge read. |
| `docs/reviews/slice2-architecture-review.md` | `8539e65a0dfca9de47e7cc714853b09a38c0113f` | 821 | Complete bounded body and full name-scrub edge read by child; original `066f25ae244497445c8bbe2ae90fb619e5ffc233` equals scrub parent, scrub result equals main. |
| `docs/reviews/slice3-foundations-review.md` | `9d8f986c367e46d72226c063301d0bacdda23234` | 609 | 1–609 read completely in three bounded calls. |
| `docs/reviews/adjudication-k-form-grid.md` | `478535e9a6b2a13d49383e826a57960d5aee5f6f` | 574 | Complete in three bounded body reads; sole name-scrub edge read. |
| `docs/reviews/adjudication-m-small-items.md` | `43520978713f052cadaf1f249813fe8105642546` | 290 | Complete in two bounded body reads; original blob identical. |
| `docs/reviews/adjudication-n-layer-measure.md` | `dd51778c590073ea34a172420e2669961b4080b3` | 501 | Complete in three bounded body reads; original blob identical. |
| `docs/reviews/adjudication-o-foundations-followups.md` | `8e3670a64d97bdfc6f67abbdbe32533f09591a6e` | 430 | Complete in two bounded body reads; original blob identical. |
| `docs/reviews/adjudication-p-prototype-decisions.md` | `eb1c92f83b58b0d0a2d6108994a05f3fa8c56966` | 343 | Complete in two bounded body reads; original blob identical. |
| `docs/reviews/adjudication-q-residuals.md` | `422391112470eb5a627d952b5db2791a6db096cf` | 244 | Complete current body plus all four amendment patches; truncated middle of `15371443` separately reread. |

The disjoint child reader's [API/app/research source record](history-inputs-api-app-research.md) and [HAR obligation ledger](history-inputs-api-app-research-obligations.tsv) provide exact edge/blob coverage and source-specific proposal dispositions. This is attributed delegation, not a claim that this reader personally reread those bodies. The reports also retain source defects: initial API publication was a fragment, the later inventory **prepended** 933 lines, positional tree paths were falsely called reorder-stable, and several source totals disagree with their own enumerations. Requirements are tracked by identity rather than inheriting those totals.

### Historical input edges requiring individual disposition

The following is the complete `--all` path history returned for the linked input set, excluding STATE. An indexed edge is not necessarily read. This index prevents the corrected current header from concealing an earlier proposal or omission.

| Path group | Revisions, oldest first | Material questions |
|---|---|---|
| API audit | `834aa58e76e463d1e446c386c78430eabd60132e`, `383e16fc4c302cfdbe05cb7381e06308240c1d03`, `ce5e99734ee4bb06ae803c9a7435d3fa1524a6ca`, `9df371d5fac393dd3fbbd95b1042ce128f95df6f` | Inventory completion, repaired document seam, historical-path/security correction. |
| App audit | `e81ca17b89cadc98c76dab492af85529960e3bdc`, `568e1ef65565998b19f3c3b15af7df91c2f098ee`, `9df371d5fac393dd3fbbd95b1042ce128f95df6f` | Original lower-bound app inventory versus later masking journey and historical-source labels. |
| Architecture research, domain, interaction | `e81ca17b89cadc98c76dab492af85529960e3bdc` | Original proposals are immutable blobs, but many prescriptions were later rejected or amended. |
| Performance | `3bb824b1c57b353402dd171b64f2526c4fafcc1e`, `2615fcf61c8bf19631c898155900a6de1cea979b` | Estimates versus measured WP-0; header correction. |
| Modern API | `ba8581318acd8c59d6e311e1e6464a39992a9073` | Binding rules accepted as L, then amended by foundations review and later architecture. |
| Slice 2 review | `4e15e97e069468f2ebfa805c35c7a31a89f05d83`, `d4715f8e453fa8078a7a51dc73c63931bd53af03` | Accepted J corrections; later agent-name scrub must not be mistaken for API revision. |
| Slice 3 review | `2e86bf61d5ab513db3a740130041a153a62877aa`, `d4715f8e453fa8078a7a51dc73c63931bd53af03` | F1–F26 and eight adjudications; later agent-name scrub. |
| K | `0100241f49d2be0b9700bea5bffc8117cd48527a`, `d4715f8e453fa8078a7a51dc73c63931bd53af03` | Header says proposed; STATE `f2d30b65` accepts K. Never infer current authority from header alone. |
| M | `fa8adb7bf323b2ad59a480ce32933bc0b9c22a1d` | Accepted at STATE `d57ed9ff`. |
| N | `d7faa2ce05ff42899f6608d5b87b1758c016f27d` | Accepted at STATE `04be43b0`. |
| O | `84993dfc2e3feec57b3c9c5c24a1a8685b5d1d67` | Accepted at STATE `eed81872`; doc and code land separately. |
| P | `bb92a657f4f6cce448cd2713c072a6f87a61be6b` | Review file lands in O code commit; P code and corrected premises come later. |
| Q | `e22ae190662fb97a6375578ff648b62775472736`, `52da837525ae88bf7e2e044f7c0d66e6d24b438d`, `c78c462a80a914ec22b75e28042d735d402a7beb`, `15371443467d1f8a069067d0cae106bd8636bfec`, `2a0cf299d02fae2adff2020f068ac8d5a00757e8` | Initial Q; applied/expanded interpretation; §§50–51 supersede add-key/value/hover questions; final evidence names/counts corrected. |

## Early STATE chronology and authority changes

The original STATE at `2d81eec4358e88ef3a37c52466d612ba9e493238` was read completely. Zero-context patches for its first 50 path revisions were read in chronological order, followed by `70dacec1`. The aggregate patch stream comprised 1,255 lines. One bounded call covering stream lines 211–480 truncated a short section near the `23f48791`/`d24b3fca` transition; both patches were then separately reread completely, closing that gap. Exact edge hashes and blobs are in this reader's [direct-read edge index](history-inputs-read-index.tsv). The coordinator's broader [revision index](history-revision-index.tsv) is an inventory, not semantic proof.

1. `25ea92b0` relaxed research-tool wording. `d5e7075f` compressed STATE and removed previously listed accepted routing rules from the ledger; deletion did not itself adjudicate product architecture.
2. `596a4706` measured the pre-refactor baseline at `d5e7075f`: 198 tests and green fmt/clippy/build. `cefc4b8a` then contradicted “no pre-existing failures”: a seven-byte run ID panicked under `[..8]`, `Ctrl+B i` was advertised but rejected, F10 reopened the last menu, and tmux could not deliver a palette chord. Tool limitations and application defects must remain separate.
3. `001258b7` recorded the substantive research conflict: retained update/draw versus immediate-mode `show`. `a156054d` accepted A–I, explicitly rejected `show`, folded performance R1–R7 and §6.3, and separated family migration from application migration.
4. `a2ddd278` replaced performance estimates with WP-0 measurements (`07cb2c9`, historically movable `perf/baseline` tag). It found the viewport width/width-minus-one double relayout. `79efbf60` recorded 42 TablePro and 36 Jackin digests; these are historical baselines, not the current oracle.
5. `e2be0ced` accepted all Slice 2 corrections. It rejected renaming the legacy root package, instead temporarily naming the new crate `tui-next` until Slice 5. Phase-call data, derived `Ui::cache`, and post-update Esc were accepted amendments.
6. `f2d30b65` accepted K/L. `d57ed9ff` accepted M. `82467d32` explicitly marked several collection action/state names as builder declarations awaiting review. `d24b3fca` then accepted the foundations review's eight adjudications and F1–F26 obligations. A green initial implementation was not acceptance of its deviations.
7. `04be43b0` accepted N's explicit layer sizes and pure measurement path. `69fcdcad` recorded interrupted components and architecture edits as WIP. `1129ab1c` remeasured that state. `873787e3` recorded doc amendments at `587c53b`; `bf491867` recorded the F1–F26/N correction payload at `7899678`, while five conformance failures remained.
8. `eed81872` accepted O. `8b8e543f` recorded its doc-only half `4aabceb`; `1653fb13` recorded code `bb92a65`. Two-way memo, generation-wrap repair, whole ASCII glyph sets, DarkGray correction and amended performance budgets were distinct obligations.
9. `1653fb13` accepted P's mono/fixture/dismissal decisions. `beca6d55` recorded code `0f66160` and doc `dc3e0fa`, and corrected the earlier “fully green” claim's scope. It also corrected P's supposedly silent dismissal loss: an action-button owner emitted a diagnostic even though the dialog owner did not.
10. `014a07f6` declared Slice 3 closed at that measured checkpoint. `8377d635` accepted Q but immediately rejected its A4 “no marker callers yet” premise. `d7950b93`, `8ed714ba`, `939a7ead`, and `cc1c877e` recorded unfinished wave components, contradictory build observations, unresolved choice item channels, and missing status-item hover.
11. `2a9316c0` called the digest race fixed, then explicitly retained a possible cross-process lost update between read and rename. Its title is not proof of full process safety. `3d525eb6` remeasured a green build and 262 library tests rather than trusting prior E0502 notes.
12. `3c557f46`, `294e231c`, `c7269962`, `5e2d21ab`, `ddd1ddaf`, and `4535ff1a` separately tracked Tabs negative-control proof, `Slot` migration, fixture privacy, Q3 machinery, bracket geometry, TextArea ownership, and live ChipBar/RadioGroup defects. Checkpoint documents report dirty source work; their commit SHA is not automatically the implementation commit.
13. `d49145fe` recorded applied Q and `OVERLAY`/`TRAPS_FOCUS` separation, while full conformance remained red. `ad94d12a` corrected stale continuation text and an incorrectly shortened clippy command; it also exposed a conflicting proposed Select trap. `70dacec1` accepted §§31–32 and rejected several fresh completion claims: neutral mono remained missing, `RowUi::part` still collapsed Clear into Inherit, and machine-checked narrowing reasons could contain false prose.

## Handoff: traps the final plan must avoid

Historical audit prescriptions are not all accepted API. Examples include immediate `show`, runtime-owned component trees, `GridCellActions`, optional backend dependency, SmallVec containers, CIE76 ANSI16, raw index identity, data borrowed by props, and row-label bracketing. The companion ledger distinguishes their accepted replacements.

Historical proof defects are recurring architecture evidence: identical test colors hid precedence reversal; tests enumerated only the same family set as buggy mono code; conformance capability flags asserted trapping for a popover; a name-containing narrowing reason claimed a state was unsupported while the component accepted its prop; digest writes claimed process safety while retaining a cross-process race. The future verification design must test the actual production invariant and include meaningful negative controls where these failures recur.

Current Q explicitly leaves Choice/Brand bracket questions open. It marks ChipBar `Activated(add_key)` superseded by `AddRequested` (§50), RadioGroup value wording settled by §§50.3–50.4, and status hover settled by §51. Its latest 934-test count is tied to source `f713ccb`, not pinned main proof. These later references must join the middle/late architecture ledger rather than reopening the historical proposals.

Additional obligations surfaced by full K/M/N/O/P reads:

- K's `Form` action/validation semantics are acceptance requirements, but its named trapped-Select test is superseded by Q's non-trapping popover rule. K's draft `SmallVec` fields are likewise superseded by L/foundations. M's positional `FieldKind` choices require caller index remapping on reordered options; the richer keyed path is `Chooser`, subject to later Form amendments.
- N's pure measurement path must not record styled parts or mutate the paint memo. Otherwise an invisible part can falsely pass conformance. `LayerSize::Fixed(0,0)` is empty, not a default sentinel; geometry mutation cannot change an open layer's modality policy. N does not decide its proposed `Ui::scroll_region` helper.
- O accepts a statistical cache-health floor, not a guaranteed hit rate under arbitrary key renumbering. Its deterministic hit/clear/wrap tests are the mechanism proof. The full manual ASCII table is scheduled for 4E, not indefinitely optional; whole typed line/scrollbar sets include fields unreachable by `GlyphRole`.
- O's pre-application style-cost substitution expires at Slice 5: the actual showcase frame's ≤5% share must be reinstated. The stand-in differential is 200−40 queries, requiring ×12.5 rather than ×10; the binding low-noise strict microbudget is ≤16 ns/query. Intent probes use a 480 difference for one update pass, not an absolute 500 count or a raw total-time ratio.
- P's split render targets and Buttons page/matrix split expire at Slice 5. While split, a gate naming only `render` omits the component matrix. Prompt and acknowledgement rows belong in Dialog's own measured-height formula. Forced-state propagation must make the whole owned subtree inert, not depend on a fixture manually forcing both halves.
- P's disabled mono fix explicitly rejects black-on-black even though a symbol/modifier-only test could pass. Capability requirements form a union. Accepted readiness props require real glyph painting, not a `PARTS` constant or a permanently narrowed test.
- Q's later changes correct test filters and fixture shape, explicitly preserve unresolved Choice/Brand questions, and attach the 934 count to tested source rather than “current HEAD.” The optional future `PARTS.first()` hook is not itself an open conformance gate in Q. Its documentation is still not current production proof.

## Source closure and remaining integration

All assigned linked input bodies and their material historical versions, including the child reader's Slice 2 review and name-scrub edge, and the 51 early STATE revisions are now reconstructed by direct body/patch reads as attributed above. There is no remaining source-read gap in this bounded assignment. Historical completeness does not finish the plan: join HI/HAR rows to the architecture readers' accepted/amended/rejected decision inventory, then to actual pinned source and real tests, and resolve every `PENDING` task mapping through the final graph. Broad rows deliberately group related invariants; a final graph must not drop a named subcondition when splitting them into independently verifiable work. Slice 2 review's CIE76 recommendation and own-SmallVec sketch are superseded by the foundations adjudications; its baseline-change allowance is not present permission.
