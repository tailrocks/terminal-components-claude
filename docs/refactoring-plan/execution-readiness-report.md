# Execution readiness report

**Verdict: GO.**

This is a preparation-qualified GO for **future DAG dispatch only**. It permits
the coordinator to perform the required live preflight and, if that preflight
passes, dispatch valid tasks in dependency order. It accepts no product task,
implementation result, verifier/reviewer task result, visual parity result,
snapshot update, or final campaign state. Retired `TASK-001` / `TASK-070`
remain excluded. Keep the campaign ledger `armed = false`; do not invent
receipts or arm it.

The disarmed ledger/preflight must bind and recheck the exact live commit/tree,
catalog/task-graph, frozen oracle, qualified taskfmt, proof-preparation,
verifier, and independent reviewer identities before any dispatch. The current
preparation evidence root is
`/private/tmp/campaign-proof-prep-20260922`; any candidate tree change
invalidates that evidence and requires fresh binding/recheck. This report
changes the candidate, so no embedded historical hash below is a current
candidate or proof seal.

This is the sole current readiness authority for `refactor/holla-parity`.
The catalog and historical qualification results remain structural or
provenance data; live exact-tree identities are accepted only when the
disarmed ledger/preflight rechecks them. This is not final product acceptance,
not permission to bless snapshots, and not a waiver of the final 7,550-key /
30,200-artifact candidate-vs-frozen-oracle gate.

## 1. Exact source and branch truth

For provenance, the clean coordinator candidate selected immediately before the
preceding documentation repair was:

```text
branch: refactor/holla-parity
commit: 02c02ffa28dc7f9083f0bdb957102844d965ce48
tree:   4e751fbc81a3c60c747a56454208e0b7391e2f6a
parent: 1fdc1cc24b4e88bb3c5994077b40eebadf3b694e
```

Those hashes are historical and are not current bindings. This report change
is docs-only and changes only this file. Its resulting commit/tree is a new
identity; future evidence must bind and recheck the exact live post-change
commit/tree, catalog/task-graph, oracle, taskfmt, proof-preparation, verifier,
and reviewer identities through the disarmed ledger/preflight. Any candidate
tree change invalidates the preparation evidence root named above.

No accepted task receipt is invented. Historical report hashes and prior
verdict identities are not current bindings. A historical review of the
known-good harness question did not edit the campaign repo, bless snapshots,
or authorize task execution:

```text
/var/folders/8p/h376l_nn3375kyj72czdq2x80000gn/T/grok-goal-c228471198a8/implementer/prep-go/GO-REVIEW.md
```

Later preparation commits after the prior documentation payload
`4abd4d7bac18d4d56b601b257e40de98422382a1` include `067068de`,
`25acf14f`, `3c249459`, `b7395789`, `a64c3529`, `76fa27f9`, `a22abb2f`,
and `4118c4f5`. They change policy, native proof, catalog remaps, or
proof tests. They are not product-acceptance receipts.

## 2. Immutable baseline and exact corpus

```text
tag peel:       4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
tag tree:       0b1f13431fdfd6060cf9f45a114afa5a99cc6c26
snapshots tree: 3f0261c32849e26feda24d87697de4a7ce6b8375
keys:           7,550
artifacts:      30,200 (ANSI/plain/PNG/HTML: 7,550 each)
```

Read-only import:
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-2026-09-19`

Manifest:
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-manifest-2026-09-19/sha256.manifest`

Manifest SHA-256:
`95e1f38220bd2fd09da44d3b98590543d1f03837b50e0069bf53bd1f73893637`.
The import was derived from the peeled tag, has zero symlinks, and was not
used as a candidate-generated baseline. Protected refs and artifacts are
unchanged.

At the assessed payload before this documentation rebinding,
`tests/visual_baseline/` and `.config/nextest.toml` exist. The candidate
branch still lacks `snapshots/` and `parity/evidence.tsv`; those remain
external protected-oracle inputs.

## 3. Catalog and executable DAG

The retained structural catalog summary contains 79 direct packages, 83
recursive contracts, 533 direct checks, 553 recursive checks, 282 dependency
edges, depth 33, 205 file-conflict pairs, 6 serialization locks, 1,174 source
obligations, and 3,256 traceability rows. Plan validation and graph derivation
both exit 0; graph output contains no status or acceptance field. The
disarmed ledger/preflight must bind and recheck the exact live catalog/task-
graph identity before dispatch.

Disposition:

- `TASK-001`, `TASK-070`: retired lifecycle/bootstrap contracts; fail
  closed; not dispatchable.
- `TASK-071`, `TASK-072`: qualification prerequisites; eligible for future
  graph-ordered dispatch after the live preflight; neither task is accepted.
- `TASK-002`–`TASK-069`: remaining valid implementation work; eligible only for
  future graph-ordered dispatch after accepted prerequisites; no task is
  accepted.
- `TASK-073`, `TASK-074`, `TASK-075`, `TASK-076`, `TASK-077`, `TASK-078`,
  `TASK-079`: remaining valid implementation work; eligible only for future
  graph-ordered dispatch after accepted prerequisites; no task is accepted.
- accepted production tasks: 0.
- current task acceptance receipts: none. Each dispatched task still requires
  independent verifier and reviewer evidence.

The graph is acyclic and machine-checkable. It encodes dependencies,
shared interfaces, file conflicts, migration boundaries, and verification
waves; it does not encode completion or authorization. Every task must
receive independent verifier and reviewer evidence after safe serial
integration.

## 4. Preparation qualification

Prior results below remain historical provenance where noted. Tool and
oracle identities do not turn mismatched results into current passes.
Current harness-trust evidence is listed after the historical record.

Qualified taskfmt:

```text
source:   /Users/donbeave/Projects/taskfmt/task-format
revision: afd3b575dbcc7044620bec4b9493a74eca3e5ef2
tree:     b7d90bd8adbe6c341a08fc485099ee8cf1584431
version:  0.2.0
binary:   /Users/donbeave/.cargo/bin/taskfmt
SHA-256:  f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
```

Historical exact-source lint evidence at payload `4abd4d7b` (not this
HEAD):

```text
run: /Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/taskfmt-lint-4abd4d7b-f978.log
result: 77 recursive contracts passed, exit 0 (73 numbered + 4 trusted fixtures)
log SHA-256: ae211906387b9161213e1fbb640fea14a62616d60a1391c7ba72046c8969ceef
```

The same-source fresh external release build produced hash
`e0b62abaf70490e714cb3ba4912e678522f1402b1258e1e9ffaf3260ec98754b`,
different from the qualified installed executable, so it was rejected
from the qualified tool identity rather than silently substituted.

The prior package-lint run at pushed code `247e47d5` remains historical:
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/taskfmt-lint-247-direct/all.log`;
73/73 passed, exit 0; log SHA-256
`afecad09ba800c74fc841fd4d830fd8932d80e84894bd18c5bf5660522134124`.
Lint validates contract format only; it does not accept implementation.

Historical native proof runs, invalid as current exact-tree seals:

| run | command/result |
| --- | --- |
| `final-37214cfb/refactor-proof` | `37214cfb`/`6445c996`; focused nextest 3/3 passed; log SHA-256 `0d649f4300204a5c9c9e2befe93f0cd88a6fa0ffcb0cf47d8beae0e79f8ef4c7` |
| `final-37214cfb/refactor-proof-full-serial` | `37214cfb`/`6445c996`; serial `cargo nextest run --locked -j 1 --package refactor-proof` 43/43 passed; log SHA-256 `77e6f97954eae5b88ef2d5725cdac8cab159381f127cd5172e3d2c646c7f71b4` |
| `preflight-current-clean` | historical clean run bound to `9346c104`/`00958932`; exit 1 with correct NO-GO refusal; result SHA-256 `ad1e78e8c2c5c14fd068befd42fa7774d115dfd83e91f41784ca2a22e56adf12` |
| `proof-full-bc4e5980`, `native-preparation-bc4e5980`, `adversarial-preparation-bc4e5980`, `taskfmt-lints-bc4e5980`, `final-checks-bc4e5980` | historical runs bound to superseded `bc4e5980`; provenance only |
| `proof-nextest-4abd4d7b` | pre-AP-repair payload `4abd4d7b` / tree `2dc5fdd7`; 45 passed, exit 0; log SHA-256 `6589e2d3aeb67bef443e61d50720f5b8f385b910079c8b71b89dd98cfda15252` |

Historical harness-trust evidence recorded by an independent review
(code is in HEAD ancestors `b7395789` AP-01–AP-05 native gates and
`a64c3529` frozen argv[0]; this is not a ledger receipt):

```text
rtk cargo nextest -p refactor-proof: 57/57 exit 0
  including visual-gate negatives and AP-01–AP-05
  proof-ap-repair/nextest.exit=0
  proof-ap-repair/visual_gate.exit=0
store_integrity: exit 0
tablepro/query/results/160x50/nocolor: 4/4 SHA-identical
audit::* 5×5 matrices (all 4 apps): 250/250 exit 0
frozen HTML argv[0]:
  /Users/donbeave/Projects/terminal-components-claude/target/debug/{bin}
```

Evidence roots:

```text
/var/folders/8p/h376l_nn3375kyj72czdq2x80000gn/T/grok-goal-c228471198a8/implementer/proof-ap-repair/
/var/folders/8p/h376l_nn3375kyj72czdq2x80000gn/T/grok-goal-c228471198a8/implementer/proof-repair/known-good-store-integrity/
/var/folders/8p/h376l_nn3375kyj72czdq2x80000gn/T/grok-goal-c228471198a8/implementer/proof-repair/known-good-tablepro-key2/
/var/folders/8p/h376l_nn3375kyj72czdq2x80000gn/T/grok-goal-c228471198a8/implementer/known-good/audit-matrices/
```

The native protocol is non-circular: pin source/tools/contracts/oracle and
prerequisite receipts, materialize contexts, start the observer, execute
the declared operation, validate observed results, then review and seal
evidence. It rejects missing/extra/duplicate contexts, wrong
IDs/scope/tree/oracle/tool, mutations, links, stale/cross-run results,
wrong hashes/nonces, replay, truncation, incomplete close, missing
outputs, nonzero/signal/timeout, observer/taskfmt failure, invalid
dependencies, and trust-input mutation.

The native threat model is honest: same-user hostile processes are
outside the isolation claim. Hashes, regular-file checks, path checks,
read-only inputs, observer evidence, and independent exit observation
provide integrity and detection controls.

The independent launcher review at
`/private/tmp/tc-known-good-control-review-9346.oX9Hqi/` rejected an
earlier visual control for protected-target use. That failure mode is
closed by frozen argv[0] (`a64c3529`) and is not a current harness
blocker. It remains historical, not a qualification receipt.

The independent adversarial proof audit
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/adversarial-fe802/adversarial-proof-contract-audit-fe802.md`
(SHA-256 `e92d7f517c05992afefc0d80bc3b4aef5250b22cb17865ddfb136e12b4ab7f19`)
is historical. AP-01–AP-05 now have native gates in `b7395789` and are
included in the 57/57 proof run. No fabricated ledger receipt is
recorded for them.

## 5. Frozen calibration and parity (census, not dispatch timer)

The detailed calibration report
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/calibration-fe802/CALIBRATION-REPORT.md`
(SHA-256 `c1c8b3221cd51661eb36cd57a4c850ab3a093cb9a5647b16d39d88c33516d55e`)
is historical: it binds `fe802534`. Its all-302-fail findings (HTML
`argv[0]` remapped, one missing key, untrusted harness) are **not** the
current failure mode.

The known-good control (campaign suite + tag product bins + frozen
`CARGO_TARGET_DIR` argv[0]) proves the comparator:

- `store_integrity` exit 0.
- historically missing `tablepro/query/results/160x50/nocolor` 4/4
  SHA-identical.
- `audit::*` 250/250 exit 0 (all 4 apps, 5×5, ANSI/TXT/PNG/HTML).
- leftover 2,341-key byte compare: 0 ANSI/TXT/PNG/HTML diffs under
  frozen HTML path.
- in-progress full matrix at review time: 642 pass / 0 fail of 7,551
  (250 audit + holla concept/fade/flows) at
  `/var/folders/8p/h376l_nn3375kyj72czdq2x80000gn/T/grok-goal-c228471198a8/implementer/known-good/full-matrix/`.
  It had not reached `form_advanced`. When it does, exit will be
  nonzero. That is Class A/B, not a new harness defect.

Full 7,550-key tag recapture is a **census**, not a dispatch timer.
Requiring tag exit 0 before dispatch is a circular gate: peeled-tag
product cannot 1:1 the frozen oracle on
`tablepro/connections/form_advanced` without blessing snapshots or
rewriting frozen tag source — both forbidden.

Class A (`form_advanced` 72x20 / 80x24 color modes): TXT=EQ PNG=EQ;
ANSI/HTML differ on padding-cell space fg white vs RGB(38,38,38);
nocolor 4/4 EQ. Layout matches oracle (full-pane Edit connection).
Frozen HTML argv[0] matches oracle.

Class B (`form_advanced` 100x30+): all 15 combos TXT/PNG/ANSI/HTML
DIFF. Tag source splits Connections tree + form at width ≥ 80; oracle
is full-pane New connection. Snapshots remain the product oracle.
Campaign HEAD still splits; later TablePro tasks must reproduce the
**oracle full-pane** frames, not the later tag split.

Class A/B are **candidate/product obligations**. Do not bless. Do not
weaken comparison. Do not treat tag source as a second oracle. Halt
dispatch only on an unexplained Class C key. Holla
`flows/task/input_cancelled/120x40/256` (`failed · 1 s` vs `0 s`) is
the first watch item — classify, do not time-normalize.

A separate complete control from source
`89218626011f2f82c4e87c4dfd5868a4c5f3e284` ran 302 cases: 298 pass, 4
fail, 2 skipped, 1 leaky, exit 100. It remains historical provenance.

## 6. Product obligations, scripts, and platform state

The campaign product remains incomplete: consumer migration and
ownership cleanup, duplicate/compatibility renderer removal,
Holla/Showcase/Jackin/TablePro application integration,
focus/hover/input/scroll/resize/PTY and lifecycle behavior, exact
visual parity, performance/allocation budgets, API,
workspace/static/documentation, and platform gates remain
implementation obligations. They are not accepted by the preparation
suite.

Current macOS-native preparation checks are recorded under the external
run root. Fresh pre-documentation Lychee passed at
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/lychee-docs-247e47d5`
(exit 0, 748 inputs; log SHA-256
`2e3b6e8d8f1f9a9740000b37cb83cec558485df985dadcfd0642052b968c956c`) and
qualified actionlint 1.7.12 passed at
`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-20/actionlint-docs-247e47d5`
(exit 0; log SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`).
Historical Lychee/actionlint runs remain bound to `9346c104`/`00958932`.

Required Linux execution is **unavailable / unverifiable** in this
environment. That is a separate platform item. It does not expand this
limited preparation GO and is not claimed closed. CI is
supplementary and cannot substitute for native macOS/Linux evidence. No
Docker, Podman, container, image, mount, firmlink, namespace, or retired
lifecycle path was used by this campaign.

## 7. Blocker register and current status

| item | owner | status |
| --- | --- | --- |
| exact-tag 7,550 exit 0 as dispatch timer | — | **closed as circular**. Full recapture is census. Final 7,550/30,200 candidate vs frozen oracle remains required at the end, including Class A/B. |
| missing `tablepro/query/results/160x50/nocolor` | visual harness | **closed**: 4/4 SHA-identical. |
| HTML argv[0] remap | visual harness | **closed**: frozen path on both sides. |
| AP-01–AP-05 native gates | proof | **closed in code** (`b7395789`); 57/57 includes them. Not a fabricated receipt. |
| form_advanced Class A/B | later TablePro / product tasks | **open product obligation**. Snapshots stay the oracle. No bless. |
| unexplained Class C during census | coordinator | **watch**. First item: Holla `input_cancelled` 0s vs 1s. Classify; do not normalize. An unexplained Class C would block future execution. |
| Linux native evidence unavailable | platform owner | **open, separate**. Does not expand this limited preparation GO; final acceptance still requires the required platform evidence. |
| product migration/ownership/parity incomplete | implementation DAG | **open**. Future DAG dispatch is permitted under this preparation gate; no product task is accepted and final architecture/product gates remain required. |
| no invented preparation receipts | verifier/reviewer | **binding**. Ledger stays `armed=false`. Do not fabricate receipts. Task-owned verifier/reviewer evidence still required per task. |
| ledger must stay disarmed | coordinator | **binding** until a later explicit arming step. Stale `.campaign/ledger.json` must not authorize the current branch. |

The ignored ledger is schema-valid but stale: `.campaign/ledger.json` has
`armed=false`, `integration_head=a292cf860d87c93fc329d16d2310ca86ad370d3d`,
catalog tree `10d44afa017c8f6e68da8b78805477335081efb0`, no preparation
receipt, four blocked task rows, and receipt keys `task-001`/`task-070`.
Its SHA-256 is
`9c26f11628fda8a1feb7b248f7a53b05741bf6dd3eaea1d2746c008d5ed52b06`.
It must not authorize the current branch.

Prior reports recorded resolved defects including stale task graph
status metadata, stale taskfmt path/version enforcement, missing native
receipt binding, incomplete worker launch fixture provenance, unbounded
observer-provider teardown, noncanonical native target names, and
ambiguous readiness verdict parsing. Those reports are historical, not
current acceptance. This report does not claim a final-tree product seal.

This preparation-qualified GO is limited to future qualification and
graph-ordered DAG dispatch after the exact live preflight binding described
above. The catalog remains structural, no product task is accepted, and the
final candidate-versus-frozen-oracle gate still requires every key and all
30,200 artifacts.
