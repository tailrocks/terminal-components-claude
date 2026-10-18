# Mechanical application-copy synchronization

2026-09-11. Bounded planning maintenance, not semantic scenario authoring or whole-plan acceptance. The surgical-patch skill kept the fix at explicit source-copy boundaries and required negative stale-copy proof before live synchronization.

## Root condition and scope

App scenario fields were copied into task obligations without an independent exact-copy guard. Source-correct action repairs could therefore leave old task wording or global source references in place. The fix is a fail-closed mechanical projection, not another inferred scenario ledger.

`evidence/sync-app-scenarios.py` reads the four authoritative scenario TSVs. It synchronizes only existing scenario sections in TASK-032–064 obligations, supporting both current bold `APP:` and plain field formats. It changes field values only: surrounding prose, headings, task membership, contribution boundaries, proof statuses and authored ownership remain untouched. Unknown/duplicate scenario sections, missing/duplicate/extra copied fields, invalid table identities/shapes and multiline values incompatible with the line-copy format fail before writes. It does not manufacture missing sections or full baseline appendices.

Baseline TASK-002–005 obligations contain source links, not copied scenario fields. Their existing source links and immutable host binding were retained; no huge generated appendix was added. TASK-003's contradictory generic paste wording was replaced with the already reviewed true-modal versus F10 distinction, including focused idle-field auto-begin, and a normative sixteen-branch source binding was added. Its trusted correction file is an exact copy of the authoritative Holla contract.

Shared projection is deliberately narrower than semantic authoring:

- Global shell contributions C02/C06/C07 come exactly from TASK-032's reviewed donor rows; every other Showcase/Holla row is preserved.
- `application-parity.tsv.reference_state` is exactly the scenario-file/ID reference plus its explicit source citation. No actions, root owners, component mappings, current defects, proof status or other authored fields are inferred or rewritten. The current pre-JT-handoff audit found only TP-071's source citation stale.
- TASK-003's correction file is copied byte-for-byte; its explicit reviewed binding must already exist or the script fails. The script does not invent or reinterpret the sixteen branches.

Flow audit/contribution semantic repairs stay with their authorized author/root; this script does not rewrite flow-audit prose or guess new contribution cuts. No history, assemble, global freeze or app producer source file is an output.

## Command behavior

Default invocation is read-only; exit0 means all selected copies match, exit1 reports drift, exit2 reports malformed input or other setup error. `--app` may repeat. `--scope shared` selects only the shared projections above; `packages` selects existing task scenario fields; `all` is the default. `--write` explicitly applies only those selected mechanical outputs. Every source/target read is compared again before the first write; a detected concurrent change aborts. This is not a multi-file transaction or replacement for coordinator serialization.

Examples:

```sh
python3 -B docs/refactoring-plan/evidence/sync-app-scenarios.py
python3 -B docs/refactoring-plan/evidence/sync-app-scenarios.py --scope packages
python3 -B docs/refactoring-plan/evidence/sync-app-scenarios.py --scope shared --app showcase --app holla --write
```

Do not run package writes while an author is changing those packages or their authoritative scenarios. The coordinator owns final freeze after source handoff.

## Actual validation and applied writes

Temporary `/tmp/app-copy-sync-audit.Uzr5BW/test_sync.py` invokes the real module and command entry point using an in-memory filesystem, with no live test writes. Normal and optimized-Python runs each pass **13 sequences**: controlled stale-copy rejection; complete projection/check/idempotent repeat; stale action in each of all four app formats with exact repair; missing/duplicate/extra field rejection before any write; concurrent source-change rejection before target write; three shell donor rows with unrelated Holla preservation; Holla sixteen-branch byte-copy drift; and source-reference-only matrix restoration with all authored fields preserved.

The full current read-only source-copy census covers **361 source scenario rows,763 copied sections,5334 copied fields**. Showcase/Holla package-only check passes with211 source rows,432 copied sections and3158 fields. The checker does not treat these structural counts as proof of action reachability or scenario semantics.

**Deliberate membership limit:** the checker validates fields of sections that exist; it has no independently authored expected section-ID manifest. A separate adversarial test deletes one entire TASK-033 scenario section while leaving others intact: the copy check accepts it. This is explicitly not omitted-scenario coverage. Frozen artifact membership and independent source-owner review must reject such deletion; task traceability alone cannot naively stand in for section membership because some preservation/contribution roles legitimately do not contain a full scenario copy. The test records this accepted counterexample rather than counting it as rejection proof.

Applied only the shared Showcase/Holla command. It changed global C02/C06/C07 and installed TASK-003's exact correction file. Read-only rerun and repeated `--write` both report zero changes. TASK-003 lint passes with zero errors/warnings using pinned taskfmt and explicit `/Users/donbeave/Projects/donbeave/task-format/experiment.toml`. The small TASK-003 prose/binding correction was applied separately, not generated by an inferred semantic transformation.

After the Jackin/TablePro author's explicit stop-ready handoff, the read-only check identified exactly five stale outputs: TASK-060/062/063/064 obligations and the global matrix source-reference projection. The scoped `--app jackin --app tablepro --write` synchronized exactly those five files (150 source rows,331 sections,2176 fields); repeated write changed nothing. The default all-app check now passes with361/763/5334. Both adversarial runs were repeated after this synchronization with the same13 passing sequences and explicit whole-section-deletion limitation. All four changed task packages lint with zero errors/warnings. Author-owned flow-audit and contribution copies were not rewritten by this pass; new policy prose outside copied scenario fields was preserved.

No002/004/005 appendix was added. Exact-copy completion does not independently approve the new Jackin/TablePro scenario semantics. No whole catalog, app parity, final source freeze or whole-goal acceptance is claimed.

## Snapshot

```tsv
path	lines	sha256
docs/refactoring-plan/evidence/sync-app-scenarios.py	193	879065fa6bf4e410ea057a65010bb1924455fd42ee99e9e77226a286d1104137
docs/refactoring-plan/shell-contributions.tsv	11	7de9e27736906daa9a1d7a8577558b6102c96367d7b8005bdf8469128dad53c1
refactoring-tasks/terminal-components/completion/003/trusted/obligations.md	23	e625a5e28b5d35d254cf65c5c35ecb4a510a1c179f7559152c4b27280c4c3ed4
refactoring-tasks/terminal-components/completion/003/trusted/holla-trace-corrections.md	57	cb64706e4134127e47047838db404c490af8a4f0eae0c63c791c61f79dc4e42f
/tmp/app-copy-sync-audit.Uzr5BW/test_sync.py	151	0b7379b6f892fd305f0d1d7e67ba833772d4125c1cb458fdb26b3d862563097f
docs/refactoring-plan/application-parity.tsv	362	ce08a2fd41a55b74394ee58deadf648bf93fbad72184df88501beff4ceb26bae
refactoring-tasks/terminal-components/completion/060/trusted/obligations.md	227	e7db3e5c15c529fa551ca1051bd78a4f1a19f57a39a4b297f097a55084bcabd2
refactoring-tasks/terminal-components/completion/062/trusted/obligations.md	153	5921bf5a015640e637ddfc39fd75a10adc18300c65e040905293ed27618dae3f
refactoring-tasks/terminal-components/completion/063/trusted/obligations.md	383	bb9d3b8d171c72ef4f76b1b9a3a09aeb5fe95bdbb0ad9e12d7e78a98f81e6895
refactoring-tasks/terminal-components/completion/064/trusted/obligations.md	833	e6efec468b5c37bd985d6b9d2385c98475fb2eae776792bf68bac364c9627d1c
```
