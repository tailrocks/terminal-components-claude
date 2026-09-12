# Improvements plan: audit reference

This document preserves research, evidence inventories and verification narratives
moved from [IMPROVEMENTS_PLAN.md](PLAN.md). These records describe
the 2026-09-10 audit snapshots; their test results and completion claims are
historical, not evidence that implementation is complete or current checks pass.
The active plan owns scope, delivery order and acceptance criteria. The prior-fix
ledger and disposition map below remain regression requirements by reference.

The original wording is retained below, with relative links adjusted. References
to a current worktree, a pass, or conditional opportunities describe the audit at
that time. The retired proposals section records the later scope decision.

## Verified baseline and evidence boundaries

The [compiler-derived inventory](../audits/tui-audit-inventory.md) covers the existing
31 widget modules, primitives, interaction states, public API and four consumers:
Showcase, TablePro, Jackin Preview and Holla. Final inventory: **1,573 source-backed
exported entries**, including all 37 production `TextBuffer` methods; **572
derived contracts** counted separately. The earlier regex count of 849 was
incomplete and must not be reused. The final catalogue has **23 showcase pages**,
including the already-existing `DiffView`; no new widget was added.

The earlier library planning pass reran formatting, Clippy, tests and Rustdoc:
**328 tests pass across eight suites**. The 460 showcase baseline cases cover
23 pages × five sizes × four palette levels. Disk evidence contains 250 representative
capture sets and 51 interaction sets, each with ANSI/text/cursor/HTML/PNG files.
Those counts establish artifact presence, not exhaustive interaction or visual
correctness. Existing final captures were sampled again; selected failing flows
were independently recaptured in isolated terminals.

The previous audit's 100,000-operation Unicode stress run, before/after timing
and controlling-PTY restoration checks are historical diagnostic evidence, not
retained automated tests. Retained Enter coverage tests Ctrl+Enter, not every
modifier combination. Facts acknowledgement coverage types individual keys,
not a paste event. Editor wheel tests do not establish editor scrollbar-drag
coverage. The plan adds these missing proofs rather than repeating broader
claims. Unicode search uses Rust lowercase mapping, not full Unicode case folding
or canonical normalization.

PNG rasterization is an independent implementation with known grapheme-width
and font limitations. Buffer/cursor/copy assertions prove the application model;
emitted ANSI proves backend semantics; real terminal inspection proves the
terminal/font path. None substitutes for all three. Actual `NO_COLOR=1` remains
separate from selecting the library's monochrome palette.

## Previous findings: disposition map

The [original audit](../audits/tui-audit.md), [interaction report](../audits/tui-audit-interactions.md),
[editor report](../audits/tui-audit-editor.md), [inventory](../audits/tui-audit-inventory.md)
and [initial ecosystem research](../audits/tui-audit-research.md) are historical
records. Their original defect descriptions do not imply those fixes are still
missing. The new prior-fix verification ledger gives current symbols, retained
regressions and corrected claim scope for each row.

| Previous finding family | Current disposition / continuing obligation |
| --- | --- |
| TextBuffer UTF-8 selection panic, public offsets, grapheme word movement, atomic replacement, joined clusters, line-ending normalization | Fixed; preserve shared invariants across every editor. Retain the previously temporary stress proof as an automated deterministic test. |
| CodeEditor modified Enter, transformed search offsets, fuzzy Unicode matching, find paste/backspace, stale matches after `set_text` | Fixed; retain exact original-source grapheme mapping and add the missing input-modifier/real paste matrix. |
| CodeEditor tiny gutter/find footer, readonly focus, mid-edit read-only indentation | Fixed in CodeEditor; do not assume other editors inherit its guards. New grid transition finding is separate. |
| DataTable replacement edit state and source-row edit identity; invalid cell/header transitions in table/grid | Fixed for recorded flows (inventory API-1). Collection cursor/anchor issues below are distinct. |
| Input validation recovery, masked clicks, tiny allocations; TextArea horizontal editing | Fixed; retain keyboard, paste, Unicode, cursor and containment assertions. |
| TextArea/CodeEditor manual scroll preservation | Fixed for retained wheel tests; add explicit scrollbar-drag proof. |
| Terminal partial-startup restoration and queued Changed-event freshness | Fixed for demonstrated setup/dispatch paths; lifecycle coverage must state which exits/signals were actually exercised. |
| Forms minimum-size layout and sidebar reachability | Fixed; other showcase sections still fail reachability, described below. |
| Semantic focus gutters, monochrome reverse selection/disabled DIM, actual NO_COLOR backend resets | Fixed; retain palette and fresh-process emitted-ANSI tests. |
| Choice containment/empty geometry, footer status reservation, duplicate Tab hints | Fixed; add executable state/containment assertions before any new baseline approval. |
| First/second click, exact drag press anchor, Facts acknowledgement click routing | Fixed for named flows; app-level paste ownership is not covered by these fixes. |
| TablePro Ctrl+D collision | Fixed with Alt+D duplication alias; keep Data/Structure chord. Broader binding metadata remains a bounded architecture question. |
| Idle TextViewport cache thrash | Fixed; dirty append/replacement cost and public mutation invalidation are different remaining problems. |
| Diff showcase/catalogue contradiction, mode cache, narrow review fallback, Old/New headings | Fixed (inventory API-3/API-4); no second diff component. |
| Table/grid Unicode edit windows and click offsets; Diff grapheme emphasis/tab/scrollbar alignment; one-cell wrap overflow | Fixed for named primitives. Generic styled viewport span boundaries remain a separate defect. |
| Inventory API-2: collection invariants split between caller and widget | Promoted from architectural risk to reproduced supported-flow list panic and identity findings below. |
| Inventory API-5: duplicated shell pointer/focus ownership | Keep typed events and existing barriers. New modal routing failures justify owner-boundary work, not a universal Widget/Container rewrite. |
| Keyboard creation of viewport selections | Still absent; extend existing TextViewport and inherit through DiffView. |
| Real async workers/cancellation, Widget/Theme traits, configurable glyphs | Still conditional opportunities. Concrete consumer proof required before implementation. |

Original reports sometimes assigned different priorities to the same defect.
This plan's definitions and current consequences govern execution; duplicates
are one work item, not independent defect counts.

## Extended primary-source research

Fetched live on 2026-09-10 through Firecrawl. This extends, rather than replaces,
the earlier Bubbles/Textual/fzf/Lazygit comparison. Local reproductions decide
defect status; another project's feature list is not proof of a missing widget.
Full source-to-consumer reasoning: [ecosystem report](../plan-verification/plan-ecosystem-research.md).

| Primary evidence | Useful contract / local decision |
| --- | --- |
| [Helix selection keymap](https://docs.helix-editor.com/keymap.html#select--extend-mode) | Explicit keyboard range extension distinguishes caret and anchor. Supports F19; do not import Helix's whole modal vocabulary. Its separate system-clipboard commands also reinforce truthful copy destinations. |
| [Zellij actions](https://zellij.dev/documentation/keybindings-possible-actions.html) and [keybinding modes](https://zellij.dev/documentation/keybindings) | Scroll/search/copy are distinct actions with ownership. Supports F19/F20 and input-mode tests, not external-editor launch or terminal emulation additions. |
| [prompt_toolkit bindings](https://python-prompt-toolkit.readthedocs.io/en/master/pages/advanced_topics/key_bindings.html#attaching-a-filter-condition) | Conditional binding groups make active ownership explicit. Supports F08d conformance before any shared descriptor. |
| [Yazi keymap](https://yazi-rs.github.io/docs/configuration/keymap/) | Separate manager/input/picker/confirm/help layers make precedence inspectable. Its task inspection confirms existing operation/output composition; no new task widget follows. |
| [rat-focus 2.1.1](https://docs.rs/rat-focus/2.1.1/rat_focus/) | Focus-set freshness and transition hooks are explicit. Existing FocusRing/HitRegistry already supply much of the mechanism; F01/F23 test owner transitions instead of replacing it. |
| [Urwid containers and dynamic ListBox](https://urwid.org/manual/widgets.html#dynamic-listbox-with-listwalker) | Insertions change integer-position meaning; opaque positions/lazy walkers separate identity and presentation. Supports F03/F05/F06 without imposing one identity type across domains. |
| [rat-widget 3.2.1](https://docs.rs/rat-widget/3.2.1/rat_widget/) and [rat-ftable 2.2.0](https://docs.rs/rat-ftable/2.2.0/rat_ftable/) | Visible-row drawing does not automatically bound copying or iterator traversal. Supports F15 measurement and existing paged/adapted data reuse, not a mandatory virtual-data trait. |
| [Kitty text-sizing protocol](https://sw.kovidgoyal.net/kitty/text-sizing-protocol/#fixing-the-character-width-issue-for-the-terminal-ecosystem) | Terminal/application Unicode width agreement is a coordination problem; optional explicit widths have a capability-detection protocol. Inference: F22 needs declared emulator/font evidence, and unconditional protocol emission is not a safe universal renderer fix. No current Junie failure on a named emulator was established by this research. |
| [Kitty OSC5522 clipboard extension](https://sw.kovidgoyal.net/kitty/clipboard/) | This extension specifies permission/error outcomes and separate read/write operations; do not attribute its status protocol to legacy OSC52. Inference: an eventual OS provider belongs at the application boundary with explicit destination/failure behavior. Current preview Copy events must remain simulated; F09 fixes delivery only. |

Earlier evidence remains relevant: [Bubbles Key/Help](https://github.com/charmbracelet/bubbles#key)
and [Textual binding declarations](https://textual.textualize.io/guide/input/#bindings)
motivate hint/action consistency; [Textual modal screens](https://textual.textualize.io/guide/screens/#modal-screens)
motivate complete input ownership. [fzf preview](https://github.com/junegunn/fzf#preview-window)
is already covered by Picker/Splitter/TextViewport composition. These links were
reviewed in the initial same-day research; the extended report adds the primary
sources above rather than claiming every historical page was fetched twice.

## Finding traceability and independent verification

All listed findings were checked by scoped subagents; cross-checks below name
which claims have a second executable reproduction versus independent source
review. The authoring agent's report remains the detailed evidence record.

| Source findings | Plan disposition | Independent check |
| --- | --- | --- |
| Prior main 17 rows, six additional groups, editor nine rows | Fixed or narrower proof scope; baseline table and F23 | Verification agent reran all 328 tests and made a per-symbol/test ledger; editor/interaction/design/API agents checked their respective prior families. |
| Inventory API-1/3/4 | Fixed; protect regressions/catalogue | API tests/inventory digest and prior-fix ledger. |
| Inventory API-2; PLAN-API-01/02/03 | F03/F02/F05 | API real-App/library probes; design independently replayed both app failures and tree insertion. |
| PLAN-API-04/05 | F06/F07 | Verification agent independently reproduced cursor drift, pending-value order and ragged Table/Grid panic. |
| PLAN-API-06; README hypotheses/API-5 | O02/O03/O06; narrow F01/F08/F09 | API and ecosystem source checks; no unproven Holla insertion bug promoted to fact. |
| INT01/02 | F01/F08a | Editor independent modal-paste App repro; main traced both-value omission against key/click paths. Upper-value test is rendered real dispatch with controlled setup, not a pure terminal replay. |
| INT03/04/05 | F08b/c/d | Design independently reproduced cluster Backspace, lost Shift range and Ctrl+S sorting; owner paste absence source-traced. |
| INT06; design D3; research R1 | F19 | Interaction key probes plus independent API/design/ecosystem source checks. |
| INT07; research R8 | F09 | Interaction and main complete copy-effect source trace; full selected Inspect App replay remains acceptance work. |
| TXT-01/03/04/05/06 | F10/F11/F12/F13/F04 | Interaction independently recompiled/reran text agent's public-API probes; main inspected ownership paths. |
| TXT-02/07/08 | F14/F15/O04 | Interaction independently checked tab geometry and repeated release workload; TSV behavior reproduced but matrix import remains optional. |
| Design D1/D2/D4/D5 | F16/F17/F23a/F18 | Design rendered/replayed; ecosystem independently inspected source and reran the still-passing baseline. |
| Research R2/R3/R4/R5/R6/R7 | F20/F08/F01/F03–F07/F11–F15; existing operation coverage | API cross-checked viewport find absence and identity recommendations; local probes independently ground architecture/performance claims. |
| Verification V01–V09 | F21–F23 and F15 | Fresh gate/artifact checks; terminal agent independently investigates lifecycle/rasterizer, with final review checking claim scope. |
| Terminal TV1/TV2/TV3/TV4/TV5 | F22/F21/F23f | Verification agent repeated width/font/HTML checks, inspected PNG overflow evidence and independently reviewed the isolated SIGTSTP probe topology/limitations. No universal terminal claim. |

Evidence documents:

- [API verification](../plan-verification/plan-api-verification.md)
- [Interaction verification](../plan-verification/plan-interaction-verification.md)
- [Text verification](../plan-verification/plan-text-verification.md)
- [Design verification](../plan-verification/plan-design-verification.md)
- [Extended ecosystem research](../plan-verification/plan-ecosystem-research.md)
- [Prior-fix ledger](../plan-verification/plan-prior-fixes-verification.md)
- [Terminal verification](../plan-verification/plan-terminal-verification.md)
- [Final cross-review](../plan-verification/plan-final-review.md)

Earlier library planning gate (historical evidence): all four cross-review findings resolved; no omitted finding
or unresolved factual issue. Main checked 34 local links across this plan and
eight supporting reports; all resolve. `git diff --check` passes. The 47-file
library SHA-256 remains
`40e412fd6bb7ff7700b90db054b0a4e5a5d535863e4eea4e55594f6bfb26202e`,
matching the earlier verified inventory. That earlier pass changed documentation
only, including a README link; its then-dirty implementation was its baseline.
The present parity pass starts from the clean holla-fable revision pinned above
and changes only this plan and its supporting parity documents.

## Holla audit snapshots and counts

The legacy snapshot is `tailrocks/holla` default branch `main`, commit
`fca7d0cc41e139014900e1876a4ab30f126cddda`, fetched on 2026-09-10 and checked
against the remote default HEAD. The new snapshot is the current local
`holla-fable` worktree at `215990ef2d682151bc0b16bd1697da3cbdbd7d9e`.
The named remote `holla-fable` tip was observed as `60090339617af590a9e62784ed65658b75a080ef`;
it was not substituted for this worktree. No other prototype branch or history
was read, compared or merged. References in the old goal to other historical
recipe URLs were not followed. Legacy dependencies were read only where their
pinned implementation determines reachable baseline behavior.

Current disposition totals: **226 classified rows** — **0 Covered — equivalent**, **3 Covered — redesigned/superseded**, **115 Partial**, **98 Missing**, **10 not applicable**, **0 deliberately rejected**. This is **216 implemented capability/workflow contracts** plus ten explicit boundary/exclusion rows. Counts exclude overlapping source-audit observations.

## Holla audit completion evidence

Current audit evidence: the independent verifier found **zero unclassified
meaningful legacy capabilities**, checked every Covered row and approved all
23 HP contracts. Two initial Covered claims were downgraded (ranking and full
command preview); headless streaming was downgraded from Partial to Missing.
The preview's **53 tests pass**; **99 focused legacy tests pass**. These runs
do not prove future parity. Primary review inspected existing cleanup-plan and
80×24 upgrade-plan PNGs; no new visual baseline was generated. All 23 prior
F-item bodies remain verbatim, local evidence links and pinned source locations
were checked, and `git diff --check` passes. This pass changes documentation
only; application code, dependencies, fixtures and captures remain unchanged.

## Historical planning completion criteria

Planning is complete when the independent parity verifier finds no unclassified
meaningful legacy capability, every Covered row has direct preview evidence,
every Partial/Missing row has its concrete HP contract, and this file covers
every prior/new finding disposition,
source evidence and uncertainty, dependencies, compatibility decisions and
acceptance; subagent cross-review has no unresolved factual issue; all local
evidence links resolve; product sources/baselines remain unchanged by this pass.

## Retired framework proposals

O01's generic worker framework, O02's widget/theme/glyph abstractions, O03's
generic keyed-collection abstraction and O06's container/modal manager are removed
from the active plan. They are not deferred implementation tasks or completion
requirements. Their original descriptions below are historical context only.

Concrete ownership and identity fixes remain in F01–F09. Holla task ownership,
cancellation, stale-result rejection and shutdown remain required by HP14/HP15,
with scanner ownership in HP18 and cleanup/report ownership in HP22. Persisted
state remains required by HP02/HP17/HP18. These contracts do not require a generic
framework. O04, O05 and O07 remain conditional in the active plan.

| Former ID / classification / priority | Original proposal (retired) |
| --- | --- |
| O01 · optional opportunity · P3: real async ownership | Existing apps simulate workers. No live-service stale-response defect proven. When integrating a real consumer, test cancellation on owner removal, generation/revision matching, stale completion rejection, failure inspection and shutdown. Then choose a shared worker boundary if two consumers need it. Medium/high lifecycle risk; no framework now. |
| O02 · optional opportunity · P3: Widget/Theme/glyph abstractions | README ideas are hypotheses; typed events and public semantic tokens already serve four apps. Require two blocked heterogeneous compositions for a Widget trait, a real second theme requirement, or tested alternate glyph-width/monochrome semantics. Medium/high public API risk. Existing page-level composition and theme resolvers remain default. |
| O03 · conditional architecture opportunity · P3: generic keyed collections | Holla restores numeric picker cursor after refresh, but a current in-modal insertion retargeting that selection was not reproduced. Reproduce an actual update flow first; F02 uses owner tab identity, F05 uses tree paths. Medium identity/compatibility risk. Only extract common reconciliation after two consumers share the same preserve/reset semantics. |
| O06 · conditional architecture opportunity · P3: generic Container/modal manager | Existing barriers, focus rings and typed outcomes already work. F01/F08/F09 establish narrow routing/effect gaps; fix them at those boundaries. Extract only policy shared by at least two real owners with parity tests. High routing risk if broadened; duplication alone is insufficient. |
