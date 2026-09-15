# Jackin/TablePro branch-review interaction obligations

## Authority, expansion and evidence boundary

This normative source supplement resolves BD-JT-01/02/03 in `branch-diff-jackin-tablepro.md`. Oracle authority is `02f5294bfdbf38004cc49130d0aff1d01f31434c`; main `7b27732a8c3c131760ec3438f641cb3c11343a42` owns the retained keyed architecture. It supplements existing primary rows without replacing native tests, deleting assertions, changing full-parent ownership or enlarging the TablePro primary register.

TASK-005 must materialize every named branch below into distinct immutable action/checkpoint identities at 80×24, 100×30, 120×40 and 160×50, retaining each parent's palette and lane axes. Each input, draw and explicit tick batch has complete frame/cursor/state checkpoints. Source-selected labels, IDs and pointer coordinates are resolved once on the oracle and frozen; candidate label search, direct page replacement, fabricated response state and post-exit continuation are forbidden. Fixture preconditions are serialized before dispatch, never chosen by the candidate. Existing native tests retain their original dimensions and assertions; resized variants require the separate source-qualified mapping.

Quoted PASTE payloads use JSON escapes; decode each once into the indicated bytes, rather than typing literal backslashes.

These obligations are source-inspected planning branches for TASK-058, not executed or sealed baseline evidence. ADJ-09 TablePro Picker-under-modal paste in BD-JT-07 and the separately owned surface/command families in BD-JT-03 remain owned by TASK-059–063 supplements; this file closes only BD-JT-01/02 and the Connections/Form portions of BD-JT-03. No result from those sibling bindings qualifies the branches below.

## JT-CONNECTION-ACTIONS: separate control ownership — full owner TASK-058

`CONNECTIONS-PRELUDE` opens Workbench → Connections with the oracle fixture list visible, selects the Local PostgreSQL card, and records its tree key, details focus and initial frame. No modal is open. The card must expose four independently registered actions before any connect attempt.

1. **connections-edit-not-connect:** CONNECTIONS-PRELUDE; click(Edit). Require the Edit form opens with the selected connection loaded, connect is not invoked, and the Workbench route remains Connections with the same selected tree key. Clicking blank details space outside any action must not connect.
2. **connections-duplicate-isolated:** CONNECTIONS-PRELUDE; click(Duplicate). Require a new draft connection form opens with duplicated fields, the source row remains selected unchanged, and no connect/test job starts.
3. **connections-delete-confirms:** CONNECTIONS-PRELUDE; click(Delete); confirm. Require the typed confirmation dialog, exact removal of the selected row and no connect side effect when confirmation is cancelled.
4. **connections-connect-only-on-connect:** CONNECTIONS-PRELUDE; click(Connect). Require only the connect/test progression begins; Edit/Duplicate/Delete labels are not interchangeable proxies for this action.

Sources: oracle `connections.rs:488–549,772–788,1093–1114`; main counterexample `app.rs:1662–1664,2720–2862,3024–3036`. Retain keyed Tree composition and separate interaction/visual TreeState initialization, but remove coarse card-level click proxies. Shared Buttons/Form/Tree own paint and hit registration; do not restore widget-owned domain state. TP-004 and TP-006 provide existing proof owners; this supplement adds exact per-button and blank-space non-action witnesses.

## JT-CONVERSION: form policy and field restoration — full owner TASK-058

`NEW-CONNECTION` opens Connections → New and lands on Basic with PostgreSQL selected unless a branch states otherwise. Validation and conversion observations treat FormData and the shared validation collector as the public boundary; app domain policy must preserve each oracle branch without reintroducing state-owning TextInputs.

1. **connections-port-zero-rejected:** NEW-CONNECTION; focus(Port); replace(0); Ctrl+S. Require exact source rejection for port outside 1–65535, focus remains on Port, and no save/connect proceeds. Port 0 acceptance in main is wrong.
2. **connections-sqlite-blank-port-defaults:** NEW-CONNECTION; select(Engine,SQLite); leave Port blank; Ctrl+S; save succeeds. Require conversion to oracle default 5432 on save, not a conversion failure for blank port.
3. **connections-name-trim-and-ssh-empty:** NEW-CONNECTION; focus(Name); replace("  staging  "); enable(SSH); focus(SSH host); replace(""); Ctrl+S. Require trimmed name persistence and empty SSH host omitted from stored connection, matching oracle conversion rather than Some(empty).
4. **connections-advanced-fields-restored:** NEW-CONNECTION; Right (Advanced); enable(SSH). Require Clients group, SSH-user and Local-only fields visible with oracle disabled/enabled geometry; disabled SSH host remains visible but inactive rather than hidden.

Sources: oracle `connections.rs:89–105,213–250,621–632,689–735,1202–1208`; main deltas `connections.rs:80–85,253–309,396–400`. Retain Secret ownership and the separately adjudicated password-display disposition; this supplement does not authorize changing that frozen outcome. TP-008, TP-009 and TP-011 anchor port/choice/advanced proof; these branches bind the missing conversion and field-presence variants explicitly.

## JT-SURFACES: declared commands require handlers — contribution owners TASK-059–063, integration closure TASK-064

`QUERY-TAB-PRELUDE` opens Workbench → Query with a seeded tab selected and no modal layer. Surface enum seeds and key bindings alone are insufficient; each named command below must reach an executable handler and draw the corresponding source UI, not a generic QuickSwitcher-only frame.

1. **query-filter-opens-editor:** QUERY-TAB-PRELUDE; press(Filter chord). Require the Filter editor surface opens with enabled predicates editable, not a no-op or Help-only route change.
2. **query-preview-and-save-bindings:** QUERY-TAB-PRELUDE; execute(PREVIEW), then(SAVE), then(EXPLAIN). Require each command invokes its source handler with observable query text, plan/error/history state changes; absent handlers in main `app.rs:2913–2984` are wrong.
3. **query-completion-and-history-consumers:** QUERY-TAB-PRELUDE; invoke(COMPLETE) and open History with multiline/error records. Require completion uses the domain completion model and History shows full multiline SQL and failure detail bytes, not first-six-line truncation or label-only surfaces.

Sources: main `app.rs:152–165,337–432,1799–1856,2913–2984,3151–3159`; oracle tab/workbench flows referenced in BD-JT-03 read checkpoint. Retain keyed tab ownership and single TextInputState only as architecture; restoration must wire real handlers through reusable runtime controls owned by TASK-059/060/061/062/063. Parent scenarios stay intact; one marker frame per Surface cannot close these branches.

## Verification binding

R-001 binds all source outcomes to complete direct capture CHK-004 and applicable semantic capture CHK-006. R-002/AC-002/CHK-006 proves active configured shared controls and absence of inert calls, application-local text mutation or global routing bypass. R-003/AC-003/CHK-005 retains all prior native/size identities and complete-parent stage ownership. TASK-064 repeats the full TablePro union with zero unresolved entries from this supplement. No branch may qualify from its name, static output, substituted model or the source inspection recorded here.
