# Source-qualified Filter, chips and Inspect routing repairs

## Authority and boundary

The coordinator authorized repairs to TP-040/042 and JA-060, then explicitly extended the same-mechanism repair to TP-039/041. This is author work awaiting a separate root review, not self-approval. Surgical-patch required reproducing the wrong traces and changing only their owning planning contracts; verify-and-stop bounded diagnostics and lints. No Terminal Components production source, canonical AGENTS file, branch, commit or golden capture was changed.

Oracle authority is `02f5294bfdbf38004cc49130d0aff1d01f31434c`; main architecture authority remains `7b27732a8c3c131760ec3438f641cb3c11343a42`. Historical F08a/F09 dispositions belong to the history coordinator and remain deferred. These repairs preserve visible oracle defects rather than implementing those historical proposals.

## Findings and source-preserving repairs

### JTR-03 — generic replacement hid Filter submission

The original TP-040 fragment was `focus(Value);replace(10);focus(and);replace(20);click(Add filter)`. The documented TextInput replacement expands to Enter-if-idle, Ctrl+L, bracketed paste, Enter. Oracle `src/bin/tablepro/app.rs:1700–1715` maps a committed field Enter to Apply, not merely ending field editing. Thus lower replace(10) already creates a filter with value10 and empty value2 and closes the modal; the later upper selector has no target. TP-039's explicit Enter and TP-041/042's replace macros had the same premature-Apply problem, followed by button selectors against a closed modal. Cancellation in TP-039 could not restore the earlier value after the edit had already been applied.

The repair uses explicit Enter/Ctrl+L/paste-or-type/Tab before Add/Cancel/Update buttons. `CommittedTab` commits the field and traverses without Apply. Generic TextInput Enter behavior is not changed. TP-040 preserves the original lower replace(10) branch as a separate fresh-T variant with cp(lower_enter_applied_early), including its empty upper field. The invalid following selector is archived here rather than presented as an executable input. TP-039 now proves actual cancel retains1 and update changes it to2. TP-041 preserves source acceptance of nonempty `not-number`: it creates a filter and zero rows rather than inventing numeric validation; subsequent999999999 also leaves zero rows, and uppercase F clears filters/restores rows.

### JTR-04 — upper Filter paste is intentionally not routed

Even after fixing premature submission, oracle `app.rs:237–246` sends Filter paste only to an editing lower Value. When only value2 is editing, paste returns Consumed without insertion. TP-040 now captures upper_paste_ignored with lower10 unchanged, upper empty and upper still editing. Actual typed `2`, `0` then creates upper20; Tab and Add produce the range. The subsequent is-NULL operator has no Value focus stop and applies without a value. F08a does not authorize changing this routing. The reusable TextInput remains a correct input component; the source-qualified app delivery policy owns the asymmetry.

### JTR-05 — painted chip labels were mistaken for focus targets

The original TP-042 used `focus(chips lead);Enter`, `focus(first chip)` and `focus(clear all);Enter`. Oracle `src/widgets/chips.rs:76–145,245` has one group focus stop, separate pointer hit IDs for lead/chips, and uppercase X for ClearAll. There is no separate clear-all button or lead focus stop. Enter at the group activates/edits its current chip, not its lead. A diagnostic attempt to Tab from Grid to the registered chip group also failed: source Grid owns Tab navigation, so registration alone does not prove that traversal route.

TP-042 now records the actual focus inventory, then uses a completed lead click. Workbench `:1061–1080` focuses the chip group and handles Lead by toggling match_all; group cursor0 identifies the first chip. An explicit Enter opens its editor and Escape restores group focus. Space toggles, Delete removes and uppercase X clears the remaining filter. This preserves pointer coverage and source group keyboard behavior without inventing focus stops or a clear control. The borrowed reusable ChipBar remains the owner of these gestures; domain filters remain TablePro-owned.

### JTR-06 — Inspect discards the real copy event

Oracle `src/widgets/viewport.rs:1386–1389` emits Copy for nonempty selected text; DiffView forwards it at `src/widgets/diff.rs:286`. Compact-open and Advanced Diff Inspect routes discard that event at `src/bin/jackin_preview/screens/inspect.rs:250,343`. F09's proposed delivery is not current oracle behavior.

JA-060 adds separate fresh capsule-multi compact-open and advanced-Diff variants. Actual menu keys open Inspect and a file; advanced uses m/Tab. Home followed by Ctrl+Shift+Home makes a nonempty selection, then y returns Changed while World.clipboard, clipboard_gen, modal and full rendered buffer remain unchanged. TASK-056 explicitly preserves that app-side discard and forbids introducing a CustomModal clipboard API for this deferred proposal; reusable DiffView and other copy owners are not suppressed.

An initial diagnostic used Shift+End after Home. The source browse caret starts at the last visible line's end (`viewport.rs:1183–1198`), so this could select zero bytes and y returned Consumed. That failing probe was rejected, not counted as evidence of a discarded nonempty copy. Ctrl+Shift+Home reaches the intended nonempty branch; y returning Changed separately distinguishes it from the no-selection case.

## Actual source replay

Disposable harness `/tmp/jt-routing.fLfB8q` imports unchanged oracle production modules and uses actual App handlers/renders. TablePro's copied App has SHA-256 `809f136ec6eef63066ea58a081a54d396fbbb8345f601311fdc55bb1ea00f5f1`, equal to the pinned git blob bytes. `git diff` for all imported oracle shared library, Jackin, and TablePro connections/db/model/sql/tabs/workbench files is empty. TablePro directly establishes the connected Production fixture before normal orders navigation; this does not qualify a complete CLI/connection prefix or a protected seed.

Command: `rtk proxy cargo test --offline --manifest-path /tmp/jt-routing.fLfB8q/Cargo.toml --target-dir /private/tmp/tui-snap-audit.656lHG/oracle-clock/target routing:: -- --nocapture`.

Four TablePro tests and one Jackin test pass. Every TablePro repaired journey and the old lower-Enter counterexample cover80x24,100x30,120x40,160x50. Jackin covers both modes at all four sizes and additionally compares the complete TestBackend buffer before/after y. The final combined rerun includes the four-size old counterexample and buffer assertions. No all-color, PTY, animation-phase, clipboard-system-effect or whole-application parity certification is claimed.

Harness hashes: lib.rs `afcfc6d13ec8479ac26cf135dc4a07d4b625b33070c2a09e262528d42f8d7701`; jackin.rs `f3ad9f34c34b687a414a1d039d4ea543ecc2f11edd45698e3a08ed2f6507d0f3`; Cargo.toml `da7c03563b08ecfe30dde6b4be0514c2881fc184b2da19fae7131e31ebcbd3a8`; Cargo.lock `3160d8f08e6c7b516e0a0df6515e646aa61ba10f2ccbbb9be4af00a33e25cd5a`.

## Exact changed ownership and synchronization

Changed four TablePro scenario rows and JA-060, five stage rows, four semantic contributions and four whole-frame contributions. Added exact TP-040/042 checkpoint selections and the separate TP-040 seed restart. TASK-060 semantic keys now explicitly include modal/focus and Filter draft/editing/preview observations. Updated directly owned056/060/063/064 obligation copies and stage/contribution donors; TASK-060 primary APP-FLOW decision_requirement exemplars match the canonical semantic/frame text. The29 affected owned donor rows were compared exactly against canonical tables with zero drift, including a final rerun after source-path cleanup. Four taskfmt lints passed with zero warnings/errors. Root/closure must synchronize all other baseline/task/history/global projections from these canonical rows, not propagate the old traces.

Stable authored scenario hashes: tablepro-scenarios.tsv `b1eace66e823213048292fb4906fa000722488a184db647d674a9fdcf657a531`; jackin-scenarios.tsv `12561aebf9d6f046eda2bc6ee3218e63e35528e69d0b02eded61bcb71f765686`. These are authored checkpoints, not an independent acceptance seal. Separate root review and final coordinator synchronization remain required.

After this handoff the root explicitly authorized one final semantic-prefix correction: canonical traceability-history.tsv F08a/F09 and matching TASK-060/056 primary disposition text now state the preserved ignored-upper-paste/discarded-copy behavior. The old leading text had still asserted the historical proposed behavior despite its deferred suffix. Historical proposal titles and status remain untouched under historian ownership. The corrected prefix identifies TASK-060's TP-040 contribution plus TASK-063's complete parent, and TASK-056's actual JA-060 complete Inspect trajectory. Closure was notified to retain these authoritative prefixes during its final projection.
