# Direct inline amendment review: J, K/L, M

Planning-only. This closes the three early parent-delta gaps identified by `history-early.md`; it does not substitute an adjudication summary for the edited source.

| Exact source edge | Directly read scope | Complementary direct reading |
| --- | --- | --- |
| `95ab652983bf6c8e77c727bd31eeeaf93b0e7635` / `e2be0ced5cf12e78aab12f88f03ed7435c5a36b3` | Entire parent-relative `COMPONENT_ARCHITECTURE.md` diff before newly appended §21, including deleted/replaced prose, signatures, examples, test/perf tables and appendices A/B | `history-early.md` directly reads appended §21 J |
| `27bd918e3a8a0a7fdba14fb10643139340d6281f` / `f2d30b654e5c6bea154392a8be6e9f4b7d4dbcb1` | Entire parent-relative diff before appended §22, including inline §15.1 F1–F13, example 13, manifests and changes within §21 | `history-early.md` directly reads appended §§22–23 K/L |
| `87ab93d4d21b75ce975c3a5cfecd987a0b935d2f` / `d57ed9ff43a8f6845eaa298969826c9e14b8a122` | Entire parent-relative diff before appended §24, including examples, facade lists and resolution of earlier open items | `history-early.md` directly reads appended §24 M |

Read method: `git diff <parent> <commit> -- COMPONENT_ARCHITECTURE.md`, sequential line ranges. Truncated result ranges were re-read narrowly. The exact before/after blob IDs are in `history-revision-index.tsv`. The combined readings cover each entire edge, not just named decision headings.

## Material inline decisions and reconciliation

These rows map source edits to retained ledger clauses. A shared topic is not automatic equivalence: narrower proof requirements remain separately enforceable. Historical API spellings below are not instructions to undo later amendments.

| Source | Inline decision or removed alternative | Ledger mapping and current disposition |
| --- | --- | --- |
| J §§3/17 | Frozen owner-indexed intent queue separates immutable iteration from mutable Cx services; paste arena lifetime is explicit | EARLY-010; retained. Empty queue costs zero probes; later O corrects nonempty probe arithmetic |
| J §§3/21 | Fifth focus request is bounded, not silently lost; no intent redelivery | EARLY-011; later §25 revises rollover details |
| J §§6/17 | Typed action cannot disappear through generic Response bitwise merge; merge exists only for unit action | EARLY-012; retained |
| J §§8/9 | Hit ordering is layer then registration, decorative regions deliver no ordinary input; capture disappears with closed layer | EARLY-008; lifecycle delivery still mandatory by §28 |
| J §§9/17 | Layer ID assigned at open; duplicate draw returns None plus diagnostic; unopened closure does not execute; draw call order cannot change z-order | EARLY-008/009; publication amendments further restrict authority |
| J §§7/17 | Inline debug identity labels, separate child-control Id versus owner/PartRef subregion; duplicate identity diagnosed without release panic | EARLY-004; later structural Eq correction retained |
| J §§12/17 | Data/model supplied per phase; key/row defaults use three impl blocks; transient string labels need not be static | EARLY-002/005; borrowing and inference proof retained |
| J §§12/17 | RowUi reserves trailing width; formatting writes directly; source generation/reconcile invalidation is explicit | EARLY-020/023 and global collection/perf rows; later grapheme painter supersedes borrowed-Line allocation sketch |
| J §§13/15 | Configured props built once by helper taking fields, not self; idless Field owns chrome while configured child owns interaction | EARLY-013 and RG customization/API rows; exact scanner obligation remains |
| J §§15/17 | Secret writes synthetic mask directly; no raw-value String or Clone/Eq/Serialize; Form must not return values bundle | EARLY-015 and security rows; later §25 corrects secret-state equality policy |
| J §§16/17 | Conformance uses real fixtures and capabilities, seven diagnostic classes, exact handled-binding correspondence and all tiny rects | EARLY-007/032/034; later readiness/reference rules supersede local forced-state API |
| J §§16/17 | Harness testing surface resolves actual family/variant and supports app lib test ownership; thin binary cannot replace a linkable test lib | EARLY-036 and architecture ownership rows; retained |
| J §§16/20 | Named unit/perf/trybuild tests have distinct discovery scopes; code examples compile verbatim; 15 documentation headings and no material TODOs | EARLY documentation/gate rows; doc-check omission of §§18–20 remains a defect, not accepted exclusion |
| J §§16/20 | App list style share remains ≤5%; zero-allocation cached hints; grid load formats once; clone benchmark deletion does not delete four-pane frame budget | EARLY-021/024/026; live style-share proof is remaining work, not covered by synthetic stand-in |
| J §§16/20 | Allocation debug/release ±1 requires stated optimizer rationale; hits growth >25% requires explanation; original baseline provenance preserved | EARLY-024/026; no permission to retain incorrect allocation assertions |
| J §§20/Appendix A | Historical visual changes require before/after classification and ring listings before expectation edits | EARLY visual rows; current immutable oracle supersedes permission to rebless |
| J Appendix B | Staging keeps original root tests; app lib is unpublished; no compatibility facade or duplicated source targets | EARLY-003 and middle staging rows; completed staging sequence is not new work |
| L §§10/11/Appendix B | Reuse ratatui geometry and eight-field border Set; three-case own Track distributes with zero allocations; no general layout solver/cache | EARLY-044 and architecture dependency/layout rows; retain architectural distinction |
| L §§11/17 | Vec replaces SmallVec in recipes, hints and sorted KeySet; contains must prove binary search, not merely result correctness | EARLY-022 plus theme/collection rows; unchanged-frame zero allocations remain |
| L §§16/Appendix B | Backend-free core, explicit own terminal-session restoration, typed terminal commands, one width function, exact normal dependency graph | EARLY-044/045; later own key enums plus optional backend dependency supersede direct backend key re-export |
| L §§16/Appendix B | Edition2024/MSRV1.88 must compile, not only appear in metadata; safe workspace lints and justified expect, never broad allow | EARLY quality rows; retain real toolchain/guard verification |
| L Appendix B | Runtime-produced extensible enums are non_exhaustive; authored token/decor records are exhaustive so added fields force downstream decisions | EARLY facade/theme rows; original J only-two-types rule superseded |
| L §19 | Semver comparison deferred until v0.1.0 release baseline, blocking from v0.1.1; no meaningless rewrite-era semver gate | Release sequencing only; does not waive current public API verification |
| K §§12/17 | Grid update takes shared GridModel; update_editable takes mutable GridEditor; read-only reason and cell actions belong to base model | EARLY-018; editable bool, GridCellActions and mandatory Editor bound explicitly rejected |
| K §15.1 | F1 props-data separation; F2 declaration Tab order; F3 hidden preserves draft/no registration; F4 pure token/width height | EARLY-014; all four are separate acceptance clauses |
| K §15.1 | F5 no returned values; F6 draw commits nothing; F7 declaration-first single action; F8 nested Select is layer-owned | EARLY-014/015; all four retained subject to later Select focus amendment |
| K §15.1 | F9 Form owns no layer/trap; F10 submit commits then validates visible fields then cross-field and reveals/focuses first error | EARLY-014/016; Form remains a component, not a draw helper |
| K §15.1 | F11 Enter arbitration respects editing/typing; F12 dirty only on committed change; F13 state redacts/zeroizes secrets | EARLY-015/016; retained, later secret-state type correction applies |
| K §16 | Real 15-field connection form <40 allocations/frame; application keyboard/mouse reachability, first invalid focus and absent password cells | EARLY Form and app proof matrices; generic toy Form cannot satisfy app workload proof |
| M §§10/12/Appendix B | Own Size and role-carrying Span keep names; own Span lives in text, not components; Frame root-only; foreign raw text qualified author::raw | EARLY-044; foreign-type facade completeness is its own mechanical gate |
| M §§11/17 | ASCII border is a plain foreign-type const, manual opt-in; Junie rounded/Paper plain unchanged; no automatic Unicode capability axis | EARLY-031; later full-glyph ASCII policy expands beyond border-only proof |
| M §§15/17 | Closed non-generic FieldKind holds default generic choice aliases; options supplied by FormData options/value_and_options under one borrow | EARLY-015/017; later keyed option semantics supersede positional sketch, not per-phase data invariant |
| M §§15/19 | Custom keyed/non-string choice uses Chooser plus caller-owned picker; generic FieldKind and dyn FieldControl are rejected | EARLY-017; later direct paths/private bridges decide present implementation; future trait widening remains optional |

No new implementation completion is inferred from these historical readings. Current code/proof status comes from architecture, proof and application audits. For atomic named tests, use the source-named gate inventory and current proof matrix as well as these semantic mappings; an umbrella EARLY row does not erase its constituent invariants.

## Final STATE merge-edge closure

The five remaining global STATE edge statuses were closed against actual blob pairs: `df83c5ed/a40daa59` adds the complete19-line boundary checkpoint (386 named checks,364 present22 deferred; fail-closed app roots; per-page production ownership;96-cell capture matrix). The three edges `8cfb810a/87bfbfda`, `87dae98d/34cf02ac`, and `e524ceaa/ec3d760d` share exact blobs `e956432e0116bba9b9e5b3e0b26dcaf5c2285bad`→`9e4ad165ef75cd4a7e54670773fcf966e24c4b6b`; full-content equality after only `tui-next`/`tui_next` replacement was mechanically verified. `7b27732a/ef405e22` adds193 lines; the entire original diff was read, retaining the historical September8 integration checkpoint, September9 contrary parity evidence/refusals and cleanup chronology. Current immutable September10 oracle remains controlling; earlier refusal to regress to September4 captures is not a new parity waiver.
