# Coordinator re-audit evidence

## Current boundary

This is ongoing planning/preparation verification. It does not certify completion, production refactoring, accepted application baselines or a working production proof harness. The complete re-audit remains open in [reaudit-plan.md](reaudit-plan.md). Repository tracked source remains unchanged; the original three planning artifact roots are still untracked.

## Rechecked external authorities

On 2026-09-11, fresh `git ls-remote` queries returned main `7b27732a8c3c131760ec3438f641cb3c11343a42`, Holla `2e2401393c47360741ebd321679de08982dca50a`, annotated tag `a643909d9a782adaf0aa1e3357710a5ed3f24443` and peeled oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Task-format remote main remains `52d9f1eb7721f409bc47beb9fced7997b5c13ede`. The GitHub PR query returns tui-snap PR #1 OPEN, non-draft, MERGEABLE, base main and head `883d03f19d890bbbf27468798db78b04e85297ac`. No external write was performed by these checks.

## Structural verifier correction

The historical trace carried a current canonical implementation decision but independently handwritten `Source disposition: unresolved` suffixes. `sync-history-stages.py` now derives the authority/provenance suffix from the canonical row, including authored remap targets; `validate-plan.py` rejects stale, missing, repeated, wrong-source or trailing-suffix substitutions. Semantic ownership text is preserved rather than inferred by the script.

The inventory validator also binds each canonical row to the actual source ledger filename and row. The two source ledgers already using the canonical schema require exact field projection; heterogeneous historical schemas retain explicit adjudicated mappings. This exposed the remaining unqualified HM46/HM47 revision references; the historian owns their correction. The checks do not pretend to establish semantic completeness of history.

[Independent review](reaudit-authority-binding.md) exercised the actual functions, complete virtual 620-source/1,094-edge/16-remap synchronization, idempotency, malformed suffixes and canonical field/provenance mutations under ordinary and optimized Python. These are bounded executable integrity checks, not a substitute for source-first review. Live synchronization remains deferred until the relevant authors finish their shared changes.

## ROOT-01 — a new paste helper initially preserved the wrong idle-input behavior

Source inspection found a contradiction in the first repaired TextInput helper. The oracle's `src/widgets/input.rs:237–243` checks disabled, calls `begin_edit`, inserts and live-validates, then returns Changed. It starts an enabled idle edit even for an empty paste. The first proposed TASK-016 helper copied main's existing editing-only guard and said never to begin editing. That would make the repaired API unable to reproduce an idle focused Args field receiving paste, with or without the Holla F10 menu. The corrected contract and real-source first-frame/idle/empty cases now have independent review in `reaudit-components-a-rereview.md`; no production helper has been implemented here.

This is a source-specific distinction, not a universal paste rule: oracle TextArea `:176–184` ignores idle paste; CodeEditor `:612–631` gives active find-query paste priority even when the document is read-only, otherwise requiring an editing writable document. The component author is repairing TASK-016 and the Holla author is extending idle/menu scenarios. The independent component reviewer additionally identified empty-paste validation/Changed semantics and viewport reconciliation requirements. [ADJ-09](architecture-adjudication.md#adj-09-deferred-product-fixes-do-not-redefine-the-immutable-oracle) records the shared authority. Closure requires independent checks of the final helper, including its first visible post-paste frame; a later close/focus restoration is not sufficient.

## Re-executed real oracle diagnostics

The coordinator reran the exact current Showcase disposable source harness with `cargo test --locked --bin showcase reaudit_` in `/tmp/showcase-reaudit.AWN3SD`: 14 focused tests passed, zero failed. They cover the reported edit-reentry, inaccessible geometry, absent scrollbar, incorrect default action, focus restoration, status Tick ownership, Members mutation and blocked domain Tick mechanisms. This is not execution of all 76 final Showcase scenarios.

The coordinator reran `cargo run --locked --quiet` in `/tmp/holla-reaudit.5pIf5v`: all recorded old-invalid/new-valid arguments, modal/menu paste, process-quit semantics, selection/copy and find checks passed at 80×24, 100×30, 120×40 and 160×50. The idle-paste extension was identified afterward, then repaired and independently rerun at all four sizes in `reaudit-app-repairs-rereview.md`. The direct harness does not certify PTY behavior or the full source SWEEP; the Holla author's separately measured PTY exit evidence has its own scope.

## ROOT-03 — dependency index drift removed at the projection owner

`assemble-plan.py` previously projected verify.toml writable scope but left dependency cells in task-index.tsv handwritten. Canonical task metadata changed for the shared painter, Form/ChipBar compatibility and dependency-owner repairs, leaving the index stale. It now derives ordered dependency IDs from each actual task.toml. `reaudit-assembly-rereview.md` independently verifies all 73 projections, unchanged prose/non-output files, normal and optimized runs, repeated no-change results, an injected duplicate/unknown stale index and the actual old two-line-removed mechanism. The dry-run utility still reports drift through its `changed` field rather than exit status; the final no-drift assertion must inspect that result or the independent validator.

## ROOT-04/05 — decision edges need actual workflow ownership

Independent review found the first new ADJ-09 TASK-043 edge overclaimed all sixteen Holla correction branches. It now binds the eight idle/editing overlay branches through that task's full comparator and its separately typed pre-submit/empty Args contributions through AC-008/CHK-006. TASK-048 closes successful raw-kind/parsed-fallback service descendants; TASK-042 retains preview/process-exit ownership outside ADJ-09's paste scope.

The first ADJ-10 source census correctly found the ChoiceDialog constructor at oracle `screens/capsule.rs:1093` but incorrectly called it a split and assigned TASK-055 based on its file. Reading `request_exit` and the JA-059 stage row proves it is the dirty-exit chooser, owned by TASK-056. The consumer contract, decision ledger and trace edge were corrected. Prelude `:164` remains TASK-051 and configuration `:1568` remains TASK-052. File ownership does not replace source workflow/stage ownership; independent rereview must include both.

## Current mechanical checkpoint — not a final seal

History normalization copied 620 canonical clauses through 1,094 historical edges and sixteen authored remaps, changing 29 files; its repeated run changed zero. Current assembly joins 3,183 edges over 1,162 sources. All 84 APP-FLOW sources and 336 protected edges were resynchronized from corrected primary-owner exemplars; repeat changed zero. App copy checking reports 361 source rows, 763 existing task sections and 5,334 exact copied fields with no drift. That narrow copier does not prove missing whole-section membership; its report documents a deleted-section counterexample rather than claiming completeness.

All 73 canonical packages lint with zero errors/warnings. The derived graph has depth 35, 24 equally deepest paths and zero unordered writable-scope overlaps. Inventory-only validation passes. Existing proof and runner frozen groups pass exact-byte checks (56 and 33 assets). Whole artifact validation remains failed until newly reviewed source-policy/timing and corrected flow assets are frozen; the previous 151-row manifest is not current acceptance. Tracked `git diff --exit-code` remains zero; no TC production edit, branch, commit or external write occurred.

## Remaining work

Finish source-history expansion, remaining finite component witness contracts, actual source-policy and timing qualification, independent repair rechecks, complete source/task/scenario synchronization and final current-artifact verification. Existing report counts and hashes belong to their recorded snapshots. No final manifest is regenerated until all required repairs and reviews are complete.
