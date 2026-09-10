# Shared implementation contracts

Read [current scope](scope.md) first. This file retains the original cross-task
requirements and full-roadmap delivery/closure rules. Apply acceptance to the
selected slice; the broad roadmap does not expand the current goal into deferred
applications, production providers, persistence or CLI work.

The [tracker](../../IMPROVEMENTS_PLAN.md) owns progress. Original audit status and
counts are historical. References below to task bodies point to the linked task
files in that tracker. No task has been completed by reorganizing this document.
Source paths in retained contracts are repository-relative; their line numbers
describe the audit snapshot and must be checked against current code.

## Conditional scope and exclusions

Generic widget/theme/glyph, keyed-collection, container/modal and worker
framework proposals remain removed. Former O01/O02/O03/O06 entries are historical
context in the [audit reference](../improvements-plan-reference.md#retired-framework-proposals).
Concrete fixes retain their task contracts, with current versus later ownership
defined in scope.md. HP14/HP15 require task ownership, cancellation,
generation/revision matching, stale completion rejection, failure inspection and
shutdown. HP18 retains scanner ownership; HP22 retains cleanup/report ownership.
Their current implementations model these contracts in simulation. Real workers
and durable HP02/HP17/HP18 stores stay in their later operational slices.

O04, O05 and O07 remain conditional and unselected. Their absence does not excuse
a known defect or defer a concrete requirement within the current Holla scope.

No new command palette, file picker, preview viewer, Diff widget, notification/
toast family, task manager, retry card, paginator or generic badge is justified
by the inventory alone. Existing controls or composition already cover the
relevant flows, or the proposed capability lacks a demonstrated reusable
contract. This is a coverage decision, not a permanent prohibition against
future evidence. Reuse and extend existing components for selected Holla flows.

## Decision

Preserve every useful existing-Holla outcome within the new Here/context,
resource/action, preview, scope, activity and plan model. The external baseline
defines capability requirements; `holla-project/CONCEPT.md` defines the new
product model; `DESIGN.md` defines the Junie interaction and visual system.
This pass specifies deterministic product representations and future execution
contracts. It does not enable real commands in the preview.

Repair ownership boundaries first: modal input, destructive action eligibility,
collection identity, read-only transitions, and retained text. Then make existing
capabilities reachable and keyboard-complete. Preserve Junie's geometry-first
focus, restrained planes, sparse framing, contextual hints and semantic color.
No new widget, framework, dependency or palette is approved by this plan.

The initial audit's completed fixes remain protected regressions, not work to
repeat. The earlier library planning pass found additional defects; passing the existing suite does
not make those defects acceptable. Priorities express correctness and impact,
not effort or commercial value:

- **P1:** panic, leaked terminal state, unintended destructive action, lost/wrong-source copied text,
  or mutation violating the active interaction owner/read-only contract.
- **P2:** incorrect identity, text, interaction, layout, performance, reusable
  contract or required evidence; keyboard and documented-state coverage gaps.
- **P3:** optional integration/abstraction without a demonstrated current need.

Every item below distinguishes an executable reproduction, source-confirmed
absence, architectural explanation, and proposed design. A proposed fix is not
itself verified until its acceptance tests pass.

## Evidence boundaries and protected regressions

Audit counts, prior-finding dispositions, source snapshots and verification
history are in the [audit reference](../improvements-plan-reference.md).
Preserve the [prior-fix ledger](../plan-prior-fixes-verification.md) and
[disposition-map regression obligations](../improvements-plan-reference.md#previous-findings-disposition-map);
completed fixes are not work to repeat. F23 retains the missing proof requirements.
This plan's priorities govern duplicate findings, which count as one work item.

Historical passing tests and capture counts do not establish current correctness.
Buffer/cursor/copy assertions, emitted ANSI and actual terminal inspection prove
different layers. PNGs remain approximate for complex text until F22 passes;
actual NO_COLOR is separate from the monochrome palette. Unicode search uses
Rust lowercase mapping, not full case folding or canonical normalization.

## Dependency order and bounded delivery units

Each row is an independently reviewable implementation unit or coordinated
invariant change, not one giant rewrite. Work may run in parallel when ownership
does not overlap; any shared-file changes need one integration owner.

| Stage | Work / dependencies | Exit evidence |
| --- | --- | --- |
| 0 · freeze contracts and retain failures | Snapshot current inventory/source digest; turn accepted temporary repros into owning tests. Record decisions for draft revocation, replacement identity, span style precedence, tab/copy semantics and public-field migration. | Tests fail for the intended reason; prior suite still establishes baseline. No unrelated baseline updates or source normalization. |
| 1A · owner safety | F01/F02/F03/F04; F08a/b can share touched routes without merging their distinct tests. | No hidden paste, unrelated tab close, stale-anchor panic or forbidden grid commit through any listed path. Real app tests as well as units. |
| 1B · text identity | F10/F11/F12/F13 as coordinated document-mutation boundaries; settle F14 mapping compatibility before exposing changed cursor semantics. | Split-style text and selected source preserved; every ingestion path capped; cache follows content; retained reading/drag identity correct. |
| 1C · lifecycle | F21 with F23b in isolated PTYs; independent of document/widget work. | Shell usable while stopped; resume/quit modes correct; unsupported/fatal cases documented precisely. |
| 2 · existing API consistency | F05/F06/F07, F08c/d, F09, remaining F14 policy migration; depend on their specific owner/transaction decisions, not an all-library rewrite. | Stable targets, checked shape errors, pending-sort policy, chord/selection parity and delivered preview copy; current well-formed behavior preserved. |
| 3 · reachable design contract | F16/F17/F18 and executable F23a; can proceed beside nonoverlapping stage 1/2 work. | Every enabled control/reference state reachable at five sizes, bounded cells/hits/cursors, readable readonly/disabled distinction. |
| 4 · keyboard/output capability | F19 after F10–F14; F20 after selection/follow contract and two consumer compositions. | Keyboard/mouse exact-copy parity, find ownership and content invalidation, reusable docs/showcase states. |
| 5 · measured live rendering | F15 after content ownership; F22 capture fidelity and remaining F23 proofs run throughout and finish here. | Incremental-work counters and representative timing budget pass; terminal/font evidence honest; state/provenance manifests complete. |

Do not postpone an independently fixable P1 merely to complete a broad API design.
If a compatibility decision requires explicit approval, isolate the narrow safe
change and state the residual root cause. Never mark the root fixed while its
enabling unchecked path remains. Visibility/signature changes require an API
migration record: callers, old/new behavior, deprecation boundary, source examples,
compatibility tests and a scoped rollback preserving user data/drafts. A rollback
must not silently re-enable a known destructive path; disable that action if needed.

## Per-change acceptance gate

For each changed primitive or composition, add only relevant cases from this
matrix, and explicitly mark nonapplicable states rather than implying coverage:

- Default, hover, pressed, focused, selected, editing, disabled, read-only,
  error/recovery, busy/loading, empty and unavailable states; meaningful hints.
- Keyboard, mouse, drag, wheel/scrollbar, paste and programmatic mutation;
  modifier propagation, topmost owner, focus order/trap/restore and stale-hit rejection.
- 72×20, 80×24, 100×30, 120×40, 160×50; below-minimum recovery where applicable;
  nonzero-origin and zero-size sentinels; resize during edit/drag/selection.
- TrueColor, ANSI-256, ANSI-16, Mono and real NO_COLOR backend subprocesses;
  no meaning carried solely by color, no hidden-by-color glyphs.
- Emoji/ZWJ/variation selectors, CJK, combining marks, tabs/control policy,
  source/display mapping, exact copied payload, cursor and clipping invariants.
- Public API docs and inventory update, existing showcase demonstration,
  actual consumer integration, deterministic regression and inspected capture
  for a visual change. New public extraction requires demonstrated reuse.

Repository gates after integration:

```sh
rtk cargo fmt --check
rtk cargo clippy --all-targets -- -D warnings
rtk cargo test
rtk cargo doc --no-deps
rtk git diff --check
```

Regenerate only deliberate affected baselines after inspecting output. Capture
scripts default to seven fixtures, not the ten-case 250-set audit; the exact
ten-case reproduction is in the [verification ledger](../plan-prior-fixes-verification.md).
Use an isolated socket and a new explicit output directory. Retain provenance
and failed cases. Larger screenshot counts cannot replace state assertions.

## Existing Holla functional-parity audit

### Authority, scope and counting

The authoritative comparison is [the capability matrix](../holla-parity-matrix.md).
It is backed by [source inventories](../holla-parity-evidence.md),
[the preview inventory](../holla-preview-inventory.md), and
[independent verification](../holla-parity-verification.md).
Only matrix rows count as capabilities; evidence-report subrows, action variants,
tests and plan items are not extra counts. Each matrix row has exactly one
disposition and one owning HP item. Shared dependencies do not double-count it.

The comparison is pinned to the 2026-09-10 audit snapshots in the
[audit reference](../improvements-plan-reference.md#holla-audit-snapshots-and-counts).
The matrix and linked source inventories retain exact revisions and evidence.

“Covered” proves the complete stated interaction and outcome in the current
deterministic preview. It does not claim live external execution. A label,
command string, generic success animation, fixture resource, or conceptual
promise alone is insufficient. Partial rows name the missing outcome. Missing
means no usable representation of the stated capability. Internal safeguards
with user-visible consequences remain requirements; release plumbing and
documentation-only promises receive explicit exclusions.

### Three distinct work categories

- **A — Existing-Holla parity:** HP01–HP23 below. These preserve baseline user
  capabilities, including obscure CLI, filesystem and safety workflows. Each
  mapped Partial/Missing row must meet its item's acceptance contract.
- **B — New-Holla functionality:** existing context rings, explanatory relevance,
  exact aliases/pins/hide controls, PostgreSQL insights, SSH/remote identity,
  GitHub cloning, specialist handoffs, dependency-aware editable plans and
  persistent-in-session activity tabs originate in the new concept. Their
  current state is recorded in the preview inventory. They are not evidence
  of old-provider parity. Native apt/dnf/systemd-user and broader cross-session
  activity restoration are expansion where the baseline lacks them.
- **C — Design-system/library work:** retained F01–F23 and conditional O04/O05/O07.
  Existing widgets, focus/hit ownership, selection, retained text, lifecycle
  and capture contracts support A. No new generic widget, framework, palette
  or dependency is authorized by a parity row.

### Common implementation and evidence contract

Every HP item includes its mapped matrix rows' exact old behavior, source
locations, preview evidence and semantic differences by reference. Those rows
are mandatory acceptance cases, not illustrative examples. Implement all named
action variants and boundary cases before closing an item.

The implementation route has two explicit gates. **Representation gate:**
model discovery, inputs, outputs, errors and effects in deterministic fixtures;
make the full interaction reachable in the preview. **Operational gate:** in a
separately authorized production integration, connect typed adapters and prove
the same contract against isolated processes/filesystems on supported platforms.
The current preview retains its no-real-command/no-real-filesystem-mutation
guarantee. Finishing this planning task does not pass either future gate.

Architecture: use stable typed resource/action identities and explicit provenance,
effective cwd, host, argv, execution kind, risk, freshness and outcome. Resolve
these facts once for review and revalidate before mutation. Do not make command
display strings executable or infer success effects by an incomplete ID switch.
Discovery, simulation and later production adapters must produce the same
domain events. Keep app-specific policy in Holla; extract shared components only
when an actual second consumer needs the same contract.

Interaction/state acceptance for every UI item: keyboard-first reachability;
pointer selection/activation and wheel ownership through the hit registry;
one active focus owner; query input never interpreted as page hotkeys; modal
paste/keys/wheel cannot reach the background; cancel restores a still-valid
opener; resize/provider arrival cannot retarget selected or confirmed resources.
Use Here pages for decisions, previews and alternatives for resource operations,
activities for running work and plans for compound intent. Preserve DESIGN.md's
focus gutter, muted metadata, semantic danger, Cancel-default destructive gates,
contextual hints and monochrome legibility. Do not restore the old launcher layout.

Automated acceptance for every item: add the named deterministic scenario to
the existing Holla harness; assert discovery, selection, exact target/cwd/argv,
gate ownership, state transitions, failure and final world effects. Assert
absence of side effects after cancel, stale confirmation or invalid input.
Existing 53 passing Holla tests are a baseline, not new parity proof. Run
`rtk cargo test --bin holla`, then relevant library/CLI tests and the existing
per-change gates when implementation touches their owners. Adapter tests must
use controlled executables/temp trees and never live user cleanup targets.

Capture/visual acceptance for every UI item: extend existing `tools/holla_shots.sh`
and `tools/holla_flows.sh` composition, using an isolated terminal and output
directory. Capture decisive entry, review, running, failure and result states
at 80×24 and 120×40 in TrueColor and Mono; add 72×20, 100×30, 160×50, ANSI-16,
ANSI-256 and actual NO_COLOR for changed geometry/state grammar. Include a
keyboard journey and pointer journey for each new decision surface. Inspect
PNG/terminal rendering and retained text/cursor evidence; record provenance.
For CLI-only rows retain stdout/stderr/exit transcripts instead of inventing a
visual screen. Capture instructions below identify the additional decisive frame.

Safety defaults are shared requirements, not optional enhancements: no shell
interpolation of discovered text; explicit shell interpreter actions remain
possible after truthful review; no secrets in preview/history/output copies;
target-bound authorization; Trash first, permanent deletion explicit; no target mutation
on dry run (explicit audit-log writes remain allowed); no success claim without observed/simulated outcomes. Named legacy
defects in the matrix are not behaviors to reproduce.

### Conflict resolutions

| Legacy behavior / conflict | Preserved value and new equivalent | Required proof |
| --- | --- | --- |
| Executing a launcher row exits the launcher; generic confirmation precedes destructive actions. | Keep fast invocation and informed consent; return to Here while a named activity retains output. Use risk-appropriate Junie facts/typed gates. | HP14/HP15 preserve identity, cwd, output, prompt access, cancellation and final result through navigation. HP21 proves no gate bypass. |
| Scattered providers, callbacks and shell snippets permit preview/execution drift or swallowed failures. | Preserve every actual command outcome in typed action definitions and explicit plan steps; command preview derives from the executable specification. | HP05–HP17 compare displayed and executed argv and effects; dependent stages do not run after failed prerequisites. |
| Old group navigation/key hints differ from current Junie; browser recommendations are display-only. | Here mixed results retain discoverability; Files is a resource page, with Junie alternatives/focus grammar. Recommendation facts remain inspectable; execution from browser is new functionality. | HP01–HP04 prove each resource/action accessible without reproducing the old layout or treating labels as execution. |
| Global frecency and learned query choice differ from exact aliases/context ranking. | Preserve retained usage, query-choice learning and privacy controls; exact user aliases win, learned choices are contextual and explainable. | HP02 demonstrates restart, opt-out and reset; learned choice cannot defeat an exact alias or safety gate. |
| `run --yes` both confirms actions and persistently trusts project content. | Preserve unattended invocation, but make durable trust an explicit operation distinct from one-run authorization. | HP16/HP17 fixtures prove noninteractive execution, stable rejection exits and no silent persistent trust; document the deliberate CLI semantic change. |
| Some legacy deletions bypass descendant protections/process guards; dry run can stop Gradle and reports deletion wording. | Preserve selectable cleanup and previews, strengthen one shared authorization boundary, carry typed dry-run outcome through result/log layers. | HP20–HP22 prove protected descendants, process uncertainty, no dry-run daemon stop, and truthful estimates/results. |
| Legacy deletion quit screen can drop the worker's result receiver; legacy task output is an unbounded in-memory vector. | Retain cleanup ownership through quit decisions; distinguish task cancellation from an irreversible operation already committed. Apply the new library's bounded-retention contract without silently losing output identity. | HP15/HP22 prove no detached unknown mutation, no lost report, visible retained/truncated output and explicit final state. |
| New concept promises broad developer tooling while legacy macOS capabilities are concrete. | Keep Homebrew, services, Amp, Oh My Zsh and macOS artifact/platform behavior; Linux upgrade examples do not supersede them. | HP10/HP13/HP20/HP23 provide macOS and Linux fixtures with correct availability and equivalent outcomes. |

These are planned equivalences. None earns “Covered — redesigned/superseded”
until the current preview actually demonstrates the row's full value.

### Delivery order and closure

1. Freeze row contracts and deterministic fixture inputs. Resolve typed action,
   filesystem identity, trust and execution ownership first (HP01/HP14–HP18/HP21/HP22).
2. Complete Files and disk resource navigation (HP03/HP04/HP19), then full insight
   policy (HP20). Add operational providers HP05–HP13 against shared contracts;
   independent adapters can progress alongside resource work.
3. Complete durable ranking/custom configuration and CLI contracts
   (HP02/HP16/HP17), without postponing their required safety boundaries.
4. Run per-row semantic, state and platform gates (HP23 plus F23). Review current
   screenshots and actual terminal evidence before approving changed baselines.

Dependencies above express shared contracts and integration gates, not a mandate
for a single giant patch. HP11/HP12 share traversal; HP14/HP15 share task ownership;
HP16/HP17 share trust/CLI semantics; HP18–HP22 share immutable cleanup identity.
Define these interfaces jointly, then implement reviewable vertical slices.

To close a Partial/Missing row, record its fixture/test and inspected capture
or CLI transcript, exact target/effect assertions, failure/cancel cases and
platform scope. A successful old test or similarly named new action is not
closure. To close a provider item, every mapped action variant must pass.
No real execution is enabled by this plan or by finishing this audit.

The independent verification report records the final source-to-row sweep,
Covered-row challenges, corrections and residual limits. Future baseline
updates must pin both revisions, repeat that sweep and classify new capabilities;
otherwise the current parity claim does not automatically extend to newer code.

Historical verification results are retained in the
[audit reference](../improvements-plan-reference.md#holla-audit-completion-evidence).

## Stop conditions

Planning history and its completion criteria are retained in the
[audit reference](../improvements-plan-reference.md#historical-planning-completion-criteria).
This plan and its linked references retain finding dispositions, evidence,
uncertainty, dependencies, compatibility decisions and acceptance criteria.
All local evidence links must resolve.

Eventual implementation is complete only when every mapped HP capability passes
its representation gate (and operational gate for a production release), every
accepted F-item is fixed or
its stated coverage proof is satisfied, all relevant gates pass, no known
introduced regression remains, and conditional O04/O05/O07 retain explicit unclaimed
status. Retired framework proposals are not completion requirements. A proven
tool/platform limit must state missing evidence and the next required external
check; it is not a passing result. Do not equate completing
this plan, passing the old suite, or updating hashes with implementing the work.

## F08 shared context

Four related but independently testable items; none needs a new input widget.
Detailed evidence: [interaction verification](../plan-interaction-verification.md).


Key hints and matching remain separate data (`widgets/keyhint.rs::Hint`). Once
the matrix exposes duplicated actual bindings, a small shared descriptor may
drive both label and match. Preserve `HintBar` layers, typed events and owner
precedence; no universal command bus. Test advertised action reachability in the
state displaying its hint, not merely absence of duplicate labels.

## F23 shared context

**P2 · coverage gaps / verification architecture weaknesses.** Existing hashes
pass with F16/F17 present. They cover one focused frame per page, exclude sidebar,
and omit hardware cursor/underline-color metadata. Capture counts have no automatic
source/binary provenance. The [prior-fix ledger](../plan-prior-fixes-verification.md)
and independent design review establish these limits.


Risk: low/medium test maintenance; lifecycle harnesses need strict process/PTY
ownership and cleanup. Write contract assertions, not copies of implementation
logic. Keep platform-unavailable checks explicit with required external evidence;
do not silently treat skipped coverage as passed. Only update baselines after
contract tests and human/agent visual review approve the intentional change.
