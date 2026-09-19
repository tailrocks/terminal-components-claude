# Next implementation goal — NOT AUTHORIZED FOR EXECUTION

This is the complete future `/goal` prompt. It is preparation documentation,
not permission to run. Current readiness is **NO-GO**; ledger
`.campaign/ledger.json` is `armed: false`; no task, worktree, receipt, push, or
merge may be created from this document.

## 1. Two-layer starting protocol

Layer 1 is the verified preparation payload, historical/tested only and never
the current final-tree binding:

- branch observed during the payload test: `refactor/holla-parity`;
- commit `14aa8ed0469219ff8f6570be7824e5ade39240cc`;
- tree `f3ef0f6badd01161bc24cdfef5161db55ba5d579`;
- parent `96c6c475b5d22193b7539565fa5b4612a88ea007`;
- tested run root
  `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-14aa8ed-lagrange`;
- tested verdict: **REJECTED**.

The Layer-1 proof qualification subchecks pass, but dispatcher verify exits
`1` with `RESULT FAIL`, preflight exits `1` at NO-GO, and calibration is
unresolved. These are historical/tested payload facts.

Layer 2 is the evidence-only docs package. Its parent is commit
`3b79d3403d52ededca087d62dccb9ad4474c105b` with tree
`d6b85e1feb8e89f25cae6db360b92f071a4c4f43`. Final sealing is external and
predeclared at:

`/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed`

The final verifier’s manifest from that run root must bind the actual clean
HEAD/tree, task contracts, qualified tools, oracle identities, and receipts.
This docs package does not self-attest final tree identity. Any subsequent
relevant edit invalidates the final sealing run and requires fresh external
verification.

The verifier setting is inherited `gpt-5.6-luna/max`. Its verifier-owned proof
receipt is schema `tc-proof-native-build/v1`, binary SHA-256
`a3b7712ab7c3ea22940ffe915a2328d25767d35e5253e9df82776db0d3b70fcc`, bound to
the source candidate commit/tree. Qualified taskfmt is source
`/Users/donbeave/Projects/taskfmt/task-format`, revision
`afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, version `0.2.0`, binary
`/tmp/taskfmt-latest-install/bin/taskfmt`, SHA-256
`f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de`.

The protected oracle is tag ref object
`1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`, peeled commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, baseline tree
`0b1f13431fdfd6060cf9f45a114afa5a99cc6c26`, snapshot tree
`3f0261c32849e26feda24d87697de4a7ce6b8375`, unchanged inventory 7,550 keys
and 30,200 artifacts: 7,550 each ANSI, plain text, PNG, and HTML. The ledger
is `campaign-ledger/v1`, has four rows, and is `armed=false`.

The preserved calibration evidence is 302 selected, 301 passed, 1 failed, and
2 skipped. The
sole failure is `tablepro_connections_form_advanced_120x40_truecolor`, with
24/100 matching and 76/100 mismatching artifacts. Independent history
establishes a mixed frozen-oracle state. No oracle, snapshot, expected
artifact, threshold, timing, or fixture may be changed to hide it.

## 2. Mandatory startup rejection gate

At startup, before creating a task worktree or spawning an implementation
subagent:

1. Read `AGENTS.md`, this goal, the current readiness report, all current
   contracts, and the generated task graph.
2. Require the external final-sealing manifest from
   `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed`.
   That manifest, not this docs package, must report the actual live HEAD/tree,
   parent, branch, clean-worktree state, task contracts, tool identities,
   oracle identities, and receipts.
3. Reject immediately if the readiness report is **NO-GO**. This includes the
   current run. **NO-GO never authorizes execution.**
4. Reject immediately if evidence is stale, missing, self-attested by the docs
   package, candidate-generated, unqualified, rejected, blocked, unknown, or
   bound to the historical/tested Layer-1 payload when a final-tree binding is
   required. The `14aa…` payload and `verifier-14aa8ed-lagrange` run are
   historical evidence only.
   Reject evidence bound to any other commit, tree, parent, branch, task, run,
   oracle, taskfmt binary,
   proof binary, or configuration.
5. Reject immediately while the ledger is `armed=false`, has no current
   accepted dependency rows, or lacks the authorized workflow’s explicit arm.
6. Reject immediately unless calibration is complete and clean: all 302
   selected cases pass, zero fail, zero skip, and the full frozen inventory
   still has exactly 7,550 keys and 30,200 artifacts.
7. Reject immediately unless the independent verifier and reviewer both return
   accepted evidence for the exact final tree in the external manifest. The
   Layer-1 tested verdict is **REJECTED**, so startup must stop.
8. Reject any tag, snapshot, grouped-store, expected-artifact, baseline,
   ledger, task-contract, or generated-file mutation before execution.
9. Reject Docker, Podman, containers, images, mounts, firmlinks, and the old
   `/task`, `/work`, `/proof`, or `/run` namespaces. Rust tests use
   `cargo nextest`; never `cargo test`.

A future GO must be a newly sealed readiness result. It cannot be inferred
from passing subchecks in a rejected run.

## 3. Evidence invalidation and sealing

Apply these rules to every proof, task, calibration, and review result:

- Any change to source, tracked documentation, task manifests, proof code,
  scripts, fixtures, oracle identity, nextest/taskfmt configuration, tool
  binary, branch parent, ledger, or protected-ref identity invalidates all
  source-bound receipts. Rerun from a clean tree; do not edit a receipt.
- The final sealing manifest is valid only for the exact run root, HEAD/tree,
  task contracts, tools, oracle, and receipts it records. Any subsequent
  relevant edit invalidates the run, even when the edit is documentation-only;
  create a new external final-sealing run.
- A receipt must bind task/check/run IDs, source commit/tree/parent, scope base,
  oracle tag/manifest/snapshot tree, taskfmt revision/version/SHA, proof binary
  path/SHA, context index, observer capability, result paths, and exact raw
  stdout/stderr/exit tuples.
- The verifier owns an external `RUN_DIR` and target directory. Contexts,
  context index, preparation results, observer capability, comparator reports,
  and taskfmt logs are immutable for one run. Candidate outputs are untrusted
  until the reviewer accepts them.
- Every check gets exactly one context and expected result capability. Missing,
  extra, duplicate, stale, substituted, cross-run, symlinked, hardlinked,
  outside-root, mutated, empty, replayed, or unknown evidence fails closed.
- The observer must be verifier-owned, nonce/request bound, supervised, and
  validated by the native launcher. A worker-written result without the
  observer event is rejection evidence, not a pass.
- `REJECTED` or `BLOCKED` verifier/reviewer evidence never becomes a receipt.
  A passing taskfmt lint, a passing subcheck, a clean worktree, or a copied
  candidate output never substitutes for accepted task evidence.
- Calibration is sealed only after the full selected set passes with no skips,
  the complete 7,550-key/30,200-artifact inventory is present, and the exact
  frozen oracle is unchanged. Unknown or mixed results remain NO-GO.
- Never bless, regenerate, rewrite, filter, delete, threshold, or replace a
  frozen expected artifact. Candidate output stays outside the oracle.

## 4. Native roles and execution loop

All roles use isolated host-local macOS environments and
`gpt-5.6-luna/max`:

- **Coordinator:** validates readiness, immutable refs, dependency receipts,
  scope, DAG scheduling, and compare-and-swap serial integration. It does not
  implement task-owned product changes or replace subagent evidence.
- **Implementer:** reads one task contract, edits only declared task paths in
  one isolated worktree, runs focused checks, and commits with `git commit -s`
  plus `Co-authored-by: Codex <codex@openai.com>`.
- **Verifier:** freezes the committed candidate read-only, owns a separate
  external run/target directory, builds native `tc-proof`, materializes exact
  contexts/index/observer/result capabilities, runs only standalone taskfmt
  `lint` and `verify`, runs native behavior and affected-oracle checks, and
  records raw evidence plus the final manifest’s actual HEAD/tree. The docs
  package never supplies that identity.
- **Independent reviewer:** independently checks scope, ancestry, tool/source
  identities, proof provenance, behavior, visual results, architecture
  ownership, forbidden mutations, and all raw outcomes. It returns
  `VERIFIED`, `REJECTED`, or `BLOCKED`.

Required sequence:

```text
read contracts → clean/source/oracle/tool preflight
→ implement one scoped task in isolated worktree
→ signed commit with required trailer
→ verifier-owned native proof preparation
→ taskfmt lint and verify only
→ focused nextest/static/behavior checks
→ affected frozen replay
→ independent review
→ compare-and-swap serial integration
```

No taskfmt lifecycle, host, runtime, dispatch, monitor, or promotion command
may orchestrate this sequence.

## 5. Exact DAG waves

The generated graph has 73 tasks, 506 checks, 276 dependency edges, and depth
35. These are exact dependency layers; same-layer parallelism is allowed only
for independent tasks with disjoint writable paths. Integration remains serial.

~~~text
L01  001
L02  070
L03  071
L04  072
L05  002 003 004 005 007
L06  006
L07  008
L08  073
L09  009 011 012
L10  010 013 029
L11  014
L12  015 016 017 021
L13  018 022 026
L14  019 020
L15  023
L16  024
L17  025 027
L18  028
L19  030
L20  031
L21  032 040 051 058
L22  033 041 052 059
L23  034 042 053 060
L24  035 043 054 061
L25  036 044 055 062
L26  037 045 056 063
L27  038 046 057 064
L28  039 047
L29  048
L30  049
L31  050
L32  065 067
L33  066
L34  068
L35  069
~~~

Operational grouping: proof foundation `001→070→071→072`; oracle/application
capture `002–008` with generated closure `073`; shared component layers
`009–031`; Showcase `032–039`; Holla `040–050`; Jackin `051–057`; TablePro
`058–064`; closure `065–069`. Group labels never override the exact graph.

## 6. Complete product acceptance

Do not call a task, wave, or campaign complete until the exact integrated tree
has accepted verifier and reviewer evidence. Final acceptance requires all of
the following:

- all 73 task contracts lint and verify through the qualified standalone
  taskfmt, with native proof contexts/results and accepted receipts;
- full locked workspace `cargo nextest`, formatting/build/doc/API/static
  checks, documentation/link checks, native macOS platform checks, and clean
  generated-artifact scans;
- real component and application paths preserve ownership, public APIs,
  canonical cells/graphemes/continuations/styles, dimensions, cursor,
  focus/hover/hit testing, layers, scrolling, selection, layout, editing,
  cancellation, stale-result handling, and lifecycle semantics;
- all Showcase, Holla, Jackin, and TablePro routes, menus, dialogs, editors,
  drawers, overlays, reconnects, resize paths, and completion/cancellation
  flows pass through actual production code;
- PTY setup/cleanup, alternate-screen behavior, input delivery, Ctrl/Alt
  paths, resize, color/NO_COLOR modes, cursor/focus state, and settled-frame
  transitions are proven;
- strict existing application/component performance budgets pass with native
  measurements and no unexplained regression;
- duplicate-painter, compatibility-layer, ownership, and hidden-renderer
  scans pass. Hardcoded oracle frames and app-local repaint overlays fail even
  when pixels match;
- the exact protected suite compares all 7,550 matrix keys and all 30,200
  ANSI/plain/PNG/HTML artifacts 1:1 across five sizes, five color modes,
  every route/state/interaction checkpoint, and the complete PTY settled
  transition set;
- the final tree has correct ancestry, clean worktree, exact dependency and
  proof receipts, no unexpected generated/performance changes, unchanged
  protected refs/stores, and independent final verifier/reviewer approval.

One missing artifact, skipped case, unknown comparison, semantic mismatch,
behavior mismatch, provenance mismatch, performance failure, or unexplained
visual drift is a failed gate.

## 7. Baseline and platform protection

The `visual-baseline` tag/ref/release/branch, snapshot tree, grouped store,
fixtures, and expected artifacts are immutable policy inputs. Read them through
the exact peeled tag identity recorded above. Never move, retarget, recreate,
force-push, write, bless, or replace them. Never change fixtures, timing,
thresholds, inputs, or coverage to hide a mismatch. No Docker/Podman/container
runtime, mount, firmlink, shared writable store, or old container namespace is
permitted. GitHub CI/performance evidence is supplementary; it cannot replace
native macOS proof.

## 8. Final merge-readiness, without merge permission

The coordinator may report a future branch **merge-ready** only after the
external final-sealing manifest at `.../verifier-final-sealed` records the
actual final HEAD/tree and:

1. readiness changes to GO from a newly sealed, exact-tree report;
2. all preparation blockers and all 73 DAG tasks have accepted verifier and
   reviewer evidence;
3. full behavior, PTY/lifecycle, performance, API/static/docs/platform, and
   7,550/30,200 visual gates pass;
4. ancestry, compare-and-swap integration, protected refs, receipt provenance,
   and clean-tree checks pass; and
5. an independent final verifier/reviewer confirms the integrated tree.

“Merge-ready” is a status claim, not merge permission. This goal never grants
permission to merge, push, move refs, arm the ledger, or alter the baseline.
An authorized outer workflow or human must separately decide any merge.
