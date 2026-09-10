# Independent Holla parity verification

**Result: pass for analysis/planning. Zero unclassified meaningful legacy capabilities found.**

The final [matrix](holla-parity-matrix.md) has **226 unique classified rows**:
216 implemented capability/workflow contracts and ten explicit boundary rows.
Disposition counts independently checked: **0 Covered — equivalent**,
**3 Covered — redesigned/superseded**, **115 Partial**, **98 Missing**,
**10 Not applicable**, **0 Deliberately rejected**.
Every Partial/Missing row has a concrete HP01–HP23 implementation contract in
[IMPROVEMENTS_PLAN.md](../IMPROVEMENTS_PLAN.md#existing-holla-functional-parity-audit).
This verifies the plan, not implementation of those capabilities.

## Independence and comparison boundary

Verifier began its own legacy inspection before receiving the synthesized
matrix or reading the other agents' inventories. It read the task attachment,
legacy AGENTS.md, source tree, registered providers, commands, CLI parser,
README, PRODUCT and documentation. It examined the 55 Rust source files by
module responsibility and inspected relevant implementation branches and
embedded/integration test assertions. Function/test-name searches located
branches; names alone were not accepted as capability evidence.

Legacy source: `/tmp/holla-parity-baseline-20260910`, `tailrocks/holla` default
`main`, pinned `fca7d0cc41e139014900e1876a4ab30f126cddda`.
Preview source: only the current local `holla-fable` worktree,
`215990ef2d682151bc0b16bd1697da3cbdbd7d9e`. No other prototype branch or history
was inspected. The verifier made no product or fixture changes.

After the first matrix existed, the verifier compared its independent findings
against all rows and aliases, challenged each Covered row against actual
preview code/test bodies and retained capture text, then read all HP contracts
and conflict resolutions. Raw agent observation IDs were checked as aliases,
not added to the capability count.

## Independent source-to-contract sweep

| Independently inspected source surface | Canonical accounting |
| --- | --- |
| Provider registry, probe context, dynamic arrivals, action identity/collisions, groups, warnings and keyboard launcher | L01–L30 and explicit aliases; HP01/HP02/HP04/HP14/HP17 |
| All 15 built-in providers plus custom user providers; current Git, sibling batches/mirrors/hygiene, Cargo, native task sources, Compose/Docker, Brew services, Gradle, IDEA and every upgrade manager | OP01–OP58; OP59 aliases execution contracts; HP05–HP13 |
| Main parser and shared registry, text/JSON listing, exact-ID execution, exits, doctor, browse, help/version | E01–E10 and browse alias I-B01; HP04/HP16 |
| Global/project TOML parsing, argv validation, diagnostics, cwd, confirmation, whole-file hash trust and durable store | E11–E24; HP17 with explicit HP16 trust-semantic resolution |
| Task specifications, sequencing/parallelism, PTY/stdin/password prompts, output drain/tail/focus, errors/cancellation/reaping, terminal summary/session ownership | E25–E45 and L23; HP14/HP15/HP23 |
| Home file/folder indexing, ranking/highlights/ignore rules, cancellation, OS open/reveal, OSC52 copy and Analyze handoff | I-F01–I-F12; HP03 |
| Standalone/shared browser, exact paths/jumps, hidden toggle, parent highlight, native path identity, generation protection, directory recommendations and safe bounded previews | I-B01–I-B14, I-P01–I-P04; HP04 |
| Usage/query memory, recency decay, recent projection, bounded/atomic concurrent store, corruption and history opt-out | I-H01–I-H06; HP02 |
| Disk tree/walker, allocated/apparent accounting, hardlinks/symlinks, cancellation/error/cloud/platform semantics, persistent size cache, overview/custom root, folding/multiselection and Spotlight | LD001–LD021; HP18/HP19/HP22/HP23 |
| All 18 reachable insight categories, internal Docker pointer, artifact classifier, roots/platform/age/process policy, sizing/detail selection | LD022–LD047; HP20 and explicit LD036 exclusion |
| Shared deletion confirmation/modes/dry-run, protected paths/ancestors/leaf links, Trash retries, per-item results/logs and worker ownership | LD048–LD060; LD061 aliases OP50; HP21/HP22 |
| macOS/Linux differences, documented-only commands/features, unused component APIs and release infrastructure | Distributed platform rows plus X01–X07/L25/L26/LD036 exclusions; HP23 |

Specific obscure outcomes found independently and retained include `browse [path]`,
file-start parent/highlight, hidden Ctrl-L path editing translated into visible
navigation, preview FIFO/device race protection, distinct non-UTF8 path identity,
finder Analyze-file-parent behavior, Linux reveal-parent fallback, no-history
opt-out, query-memory precedence, service-cache TTL, Spotlight timeout/fallback,
all cleanup ages/roots and exact CLI rejection ordering.

The sweep did not find an implemented provider, command, meaningful workflow,
configuration surface, platform behavior or safety outcome lacking a canonical
row/disposition and, where incomplete, an HP contract. This conclusion comes
from source-to-row comparison, not from passing tests or a matching row count.

## Covered-row challenges and corrections

| First-pass claim | Independent evidence and final disposition |
| --- | --- |
| L09 equivalent ranking | Preview `domain/ranking.rs` adds a 900,000 explicit-pin boost, which can exceed stronger text buckets. Deterministic ordering alone does not prove the complete row. **Downgraded to Partial**, HP02. |
| L18 redesigned complete preview | `screens/finder.rs::preview_lines` caps commands and truncates path/command fields. `shots/h_flow_child_query.txt` visibly contains `mise run //apps/…nd:test`. A single-command drawer test does not prove full multiline inspection. **Downgraded to Partial**, HP01. |
| E42 Partial headless streaming | Activity output has no headless CLI execution/rendering contract. **Downgraded to Missing**, HP14/HP16. |
| L06 redesigned contextual grouping | `screens/finder.rs::rebuild` constructs Suggested/Recent/Explore and parent/active-work sections, skips heading rows for action dispatch, and keeps scope/type visible. `app_tests.rs::opening_without_typing_suggests_an_action_with_a_reason` and `shots/h_first_use_120x40.txt` substantiate the represented interaction. **Retained redesigned** for grouping; complete provider/discovery parity remains separate Partial/Missing rows. |
| L17 redesigned Esc behavior | `app_tests_flows.rs::esc_ladder_and_quit_rules` asserts query clear, scope reset, empty-root exit and running-work quit protection. Modal routing and gate tests substantiate cancellation priority. **Retained redesigned**. |
| L23 redesigned execution continuity | `app.rs::execute_item` routes execution into an identified retained activity; activity navigation/output tests and failure tests assert preserved output and result visibility. **Retained redesigned** for launcher-to-activity continuity. Actual PTY, terminal restoration and complete execution semantics remain E25–E44 obligations. |

No Covered-equivalent rows remain. The three retained redesigned claims are
limited to their stated interaction outcomes; they do not grant provider,
filesystem, persistent storage or real-process coverage.

Other corrected factual defects:

- L01 now states provider-specific prerequisites, including Node's unchecked
  runner availability and always-present entry routes.
- OP37's shell-pipe truncation was repaired. Its row now retains image-removal
  error suppression and all network/system/volume prune stages, with escaped
  Markdown pipes.
- The conflict table now correctly identifies legacy task output as an
  unbounded vector, distinguishing the new library's retention contract.

## Implementation-contract review

All 23 HP items were read. Each includes priority, preserved capability/current
gap, canonical source/preview rows, intended Junie interaction, architecture and
component reuse, deterministic scenario, acceptance/automated verification,
capture/visual verification and dependencies. Common requirements explicitly
apply keyboard/mouse/focus/state and safety gates to each mapped workflow.
Row references include exact old variants and boundary cases as mandatory
acceptance, not illustrative examples.

Conflict resolutions preserve useful outcomes while rejecting accidental bugs:
ordinary configured Git pull remains available alongside ff-only defaults;
origin/GitLab batch scope is explicit; native task adapters remain distinct from
mise fixtures; all upgrade managers remain independently reachable; typed
execution removes shell/preview drift and swallowed prerequisite failures;
durable trust receives an explicit approval operation; deletion gains protected
descendant closure, process uncertainty, truthful dry-run and retained worker
ownership. These planned equivalents are not prematurely classified Covered.

The plan separates A parity from B expansion and C library work, preserves
F01–F23, and requires both deterministic representation and later isolated
operational proof. It authorizes no application implementation in this audit.

## Evidence limits

This verifier's gate was read-only source, test-body, matrix, contract and capture
text inspection. The main/launcher audit additionally reports 99 focused legacy
tests (41 launcher plus 58 file/history/browser/preview/finder) and 53 preview
tests passing. Those runs cover selected behaviors, not all 216 implemented
contracts. No claim that the entire old product was runtime-tested is made.

The primary agent also inspected existing cleanup-plan and 80×24 upgrade-plan
PNGs. No new capture or visual baseline was generated by this verification.
Live OS, PTY, Trash and persistence parity remain future HP operational gates.
Future changes to either pinned source require a new comparison; this pass does
not certify later revisions.

## Primary integration checks

After independent review, the primary agent validated 766 local links/anchors,
226 pinned old-source locations, 226 unique matrix rows and all 23 HP contract
field sets. All 23 pre-existing F-item bodies match the starting revision
verbatim. The worktree diff contains exactly five documentation files and no
application, dependency, fixture or capture changes. `git diff --check` passes.
These structural checks supplement the independent semantic sweep; they do not
establish functionality by themselves.
