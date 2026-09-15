# HAR-LOCATORS planning closure witness

Planning-only read-only verification, 2026-09-15. Confirms repaired immutable revision-range locators and HAR-020 accepted-disposition table; does not execute the 110 HI/HAR production owner proofs.

## Locator repair (HIHAR-01)

| Check | Result |
|---|---|
| `history-inputs-api-app-research-obligations.tsv` rows with `locator_pin=7b27732a…` | 22 |
| Impossible-span counterexample (834aa58e line 1073) | replaced by pinned-main locator + separate `historical_provenance_edges` |
| HAR-002 example | `locator_pin=7b27732a…;historical_provenance_edges=834aa58e` — body ranges name main blob, not creation fragment |

All 22 HAR body-locator rows inspected carry explicit `locator_pin` and provenance-edge separation per `reaudit-history-inputs.md` HIHAR-01 repair.

## HAR-020 accepted table (HIHAR-02)

| Artifact | Verified state |
|---|---|
| `history-inputs-api-app-research-obligations.tsv` HAR-020 | status `accepted dispositions`; `accepted23=7b27732a:COMPONENT_ARCHITECTURE.md:3927-3954` |
| `refactoring-tasks/.../065/trusted/source-obligations.tsv` HAR-020 | finite23 TASK-065 trusted table; #22 partial Cx services only |
| `historical-obligations.tsv` / canonical A47 adjacent rows | no remaining “proposals only” HAR-020 wording |

Proposal-table language survives only as historical provenance (`e81ca17b:324-350`), not as current disposition authority.

## Mechanical sync

| Command | Result |
|---|---|
| `sync-history-stages.py --write` | changed_files:0 |
| `sync-derived-obligations.py --write` | changed:[] |
| `assemble-plan.py --write` | changed:[] |
| `validate-plan.py --summary` | passed:true, error_count:0 |

## Boundary

110-row source/disposition review and future TASK-065/066 production proofs remain unexecuted. This witness closes planning-side locator/provenance and HAR-020 disposition specification only.

## Disposition

Planning-side HAR-LOCATORS is closed on evidence: `reaudit-history-inputs.md`, current locator/disposition bytes, and idempotent validator pass.
