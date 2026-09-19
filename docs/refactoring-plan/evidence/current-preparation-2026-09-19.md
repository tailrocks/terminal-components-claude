# Current preparation evidence — 2026-09-19

This is a provenance index for the preparation package. It is not a verifier
receipt, task acceptance, dispatch authorization, ledger-arm operation, or GO
decision. Raw evidence remains outside the checkout. Status is **NO-GO**.

## 1. Candidate and Git evidence

The final source payload used by the fresh evidence roots is:

```text
branch: refactor/holla-parity
commit: 74e4ec458e2d8b41257232900bdf511bfa335730
tree:   f14b129677ccc493de52cca25d85d14c0f413823
parent: 4a95fcdeedb8f7a3a132162e536ca28c2404b823
```

The four canonical docs are being reconciled as a documentation-only change
after that payload. Therefore the fresh roots below bind `74e4ec45` /
`f14b1296`, not the post-reconciliation documentation tree. A new external
seal must bind the post-commit HEAD/tree before acceptance. Any further
relevant source, documentation, contract, tool, oracle, or environment change
invalidates affected evidence.

The pre-repair inspection identity and preparation commits remain historical
provenance:

```text
e8c4950928b0ab6cc1268777dbed6f96cb0309ba
tree ea5edaaeddb3c5e5d5b5906a00c998d25375d3d4
fix(refactor-proof): satisfy strict Rust static gates

f6f94dc995f5b6451800d739174d7f23802a40c3
tree 61563f7b48d0fe7b8e5bdddae57072e96f6a6f12

8e783592afd0a2c2f08076858a386a091a35e712
tree e2867345caf658d339deaecd7f8fa13d04e7c008

bb574d84bf25ff9179b42e951fca52068f2ab623
tree 6e464830897f7c39014b12a79219d6cde8b56549
fix: harden preparation shell guards
```

No pre-documentation receipt survives the relevant documentation tree change.

## 2. Protected oracle import

Read-only Git checks:

```text
refs/tags/visual-baseline^{commit}
  4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
tag commit tree
  0b1f13431fdfd6060cf9f45a114afa5a99cc6c26
snapshots tree
  3f0261c32849e26feda24d87697de4a7ce6b8375
```

Exact external import:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-2026-09-19
```

Independent manifest:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/oracle-import-manifest-2026-09-19
```

Recorded facts:

```text
matrix keys: 7,550
ANSI: 7,550
plain text: 7,550
PNG: 7,550
HTML: 7,550
total artifacts: 30,200
symlinks in imported snapshot corpus: 0
```

The import was produced from the exact tag-derived Git tree and made
read-only. Its `sha256.manifest`, four key lists, tree manifest, and count
files are the raw inventory. Protected refs and expected artifacts were not
modified.

The complete snapshot-producing source identity is:

```text
commit: 89218626011f2f82c4e87c4dfd5868a4c5f3e284
tree:   6fccf997cd742071ebcff0e0a00e89404ef95ca8
snapshots tree: 3f0261c32849e26feda24d87697de4a7ce6b8375
```

The source control run is:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/baseline-control-892-2026-09-19
```

Its `logs/source-identity.txt` records the exact source and inventory. The
complete direct matrix summary is retained at:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/baseline-control-892-2026-09-19/logs/final-direct-matrix-summary-v2-2026-09-19.txt
SHA-256: 33034862cfb3d7ae2676bbe01269db5ac0c1af6d3cf6e61896f408c36eb14d3c
```

It records 302 cases, 298 passed, 4 failed, 2 skipped, 1 leaky, exit 100;
7,550 ANSI, plain-text, PNG, and HTML artifacts each; and 17 diff files. The
failed surfaces are Holla `task_input_cancelled` nocolor timing (`0 s`
versus frozen `1 s` at `80x24` and `120x40`), pointer
`tablepro_resize_workbench_grown`, TablePro `ack_gate`, and TablePro
`connections/form_advanced`. This proves full matrix execution coverage, not
a clean calibration. No output was normalized or blessed. Earlier exits `4`
and `94` remain incomplete control attempts, not calibration passes.

The distinct architecture/source oracle remains:

```text
commit: 02f5294bfdbf38004cc49130d0aff1d01f31434c
tree:   efa2b409b77077caf5c639f7f4c6154cbadbce5
```

Correction: `3570a2ed23444dddf1eddcdcc49b654b169038fe` is not the snapshot
source for this gate. It has tree `77d6a559536d6a1d733b52d3b78b5b49112315cb`
and only 28,580 artifacts / 7,145 keys. Its partial run is retained only as
negative provenance:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/baseline-control-3570-2026-09-19/logs/full-rtk.log
```

That log stops at 151/302 with missing baseline-approval failures.

## 3. Tool and preparation evidence

Qualified taskfmt identity:

```text
source: /Users/donbeave/Projects/taskfmt/task-format
revision: afd3b575dbcc7044620bec4b9493a74eca3e5ef2
version: 0.2.0
binary: /tmp/taskfmt-latest-install/bin/taskfmt
SHA-256: f9781ef8ad5909a8dc9f5902aafa177623310eb72cb1645a37de4567016664de
```

Only standalone taskfmt `lint` and `verify` are allowed. Taskfmt source Docker
integration was not run; its Docker-only test selection fails closed without
the explicit opt-in. No container or mount lifecycle was used.

Plan and catalog evidence:

```text
python3 -B docs/refactoring-plan/evidence/validate-plan.py --summary
  exit 0; error_count=0; 73 tasks; 1,174 source obligations;
  3,256 traceability rows; maximum dependency depth 35

task catalog
  73 packages; 506 checks (27 argv, 479 shell); 276 dependency edges;
  0 structural graph errors; 0 accepted production tasks

all standalone taskfmt lints
  73/73 passed in the source-payload qualification run
```

Proof and script evidence from the final source payload is:

| check | root and result |
| --- | --- |
| proof build | `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/proof-final-74e4ec45-2026-09-19`; receipt JSON binds commit `74e4ec458e2d8b41257232900bdf511bfa335730` and tree `f14b129677ccc493de52cca25d85d14c0f413823`; binary SHA-256 `88c5340476e1fbaa2e424d97db75b4c323735f4bf3bde7dea1d915b940d6c012`; receipt SHA-256 `98e6fe9d090391c9cb28b8cb6fb710ac5c94d2120a2bf39190b143fcb064a86f` |
| proof nextest | `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/proof-nextest-74e4ec45-2026-09-19`; identity binds the final payload; `cargo nextest: 28 passed (2 binaries, 21.527s)`; exit 0 |
| static | `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/static-74e4ec45-2026-09-19`; fmt, Clippy, and rustdoc each exit 0; identity binds the final payload |
| shell | `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/shell-74e4ec45-2026-09-19`; Bash syntax, ShellCheck, shfmt, preparation guards, proof-path guards, and dispatch authorization each exit 0; identity binds the final payload |
| catalog | `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/catalog-74e4ec45-2026-09-19`; plan exit 0, 73 packages linted with qualified taskfmt and zero failures, rebundle exit 0; identity binds the final payload |
| documentation | `/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/docs-74e4ec45-2026-09-19`; actionlint and Lychee each exit 0; identity binds the final payload |

These are raw source-payload observations, not an accepted preparation
receipt. The catalog still records 73 packages, 506 checks, 276 dependency
edges, maximum depth 35, and zero accepted production tasks. The
TASK-071/TASK-072 preparation attempt fails closed because dependency receipts
are absent; no task was accepted. Native proof remains integrity/control
evidence, not same-user hostile process isolation.

Superseded pre-documentation evidence remains preserved but is not current:

```text
/private/tmp/campaign-readonly-audit-20260919-refactor-proof-nextest.log
  historical pre-repair run: 27/27 passed
/private/tmp/tc-preflight-proof-current.sD2yr5/refactor-proof-nextest.log
  historical pre-final run: 27 passed
/private/tmp/campaign-preparation-guards-parent.log
  historical preparation-guard result
/private/tmp/review-f6-preparation.diff
  historical review diff for the shell/preparation repair
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/proof-preparation-final-2026-09-19
  superseded proof build; binary SHA-256 f85400a16dc131fe0f59bfc90b5ec22cadf97de02833a216ed81385bf70a1b44
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/taskfmt-path-repair-2026-09-19
  historical trust-path repair evidence
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/symlink-parent-repair-2026-09-19
  historical trust-path repair evidence
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/static-final-2026-09-19
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/shell-final-current-2026-09-19
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/docs-final-current-retry-2026-09-19
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/taskfmt-lints-final-retry-2026-09-19
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/boundary-final-current-2026-09-19
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/preflight-final-current-2026-09-19
```

The older boundary and preflight roots remain useful diagnostic evidence only:
the boundary check still exits 1 on missing `parity/evidence.tsv`, the exact
NO-GO report gate still exits 1, and the ignored ledger remains `armed=false`.

Independent checks retained from the current audit include:

```text
/tmp/tc-baseline-inventory-current.K4BNNv
/tmp/tc-taskfmt-lint-current.8gHWwI
/tmp/tc-boundary-current.LvN9KE
/tmp/tc-architecture-audit-current.xg4YPF
/tmp/tc-architecture-audit-current.EM05fP
/tmp/tc-architecture-serial-final.bY9i32
/tmp/tc-legacy-forced-audit.n9TrMr
/tmp/tc-git-diff-check-current.log
```

The older full-workspace nextest observation `3,359 passed, 1 failed, 6
skipped` is historical only. The current raw workspace result is `3,342
passed` and exit 0 at:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/workspace-nextest-sealed-final-2026-09-19
```

Durable log:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/workspace-nextest-sealed-final-2026-09-19/nextest.log
```

The run-root `identity.txt` binds this workspace result to the previous source
commit `4a95fcdeedb8f7a3a132162e536ca28c2404b823` and tree
`f98f4908506f55026a35ea4e2701b78c240d147c`. It ran before this docs-only
change; source behavior is unchanged by that edit, but this is raw evidence,
not a final-tree receipt and must not be relabeled as binding
`74e4ec458e2d8b41257232900bdf511bfa335730` / `f14b129677ccc493de52cca25d85d14c0f413823`.
No preparation receipt exists. Boundary validation still fails at missing
`parity/evidence.tsv`; native Linux is unavailable. The calibration, boundary,
proof-receipt, and Linux requirements remain open gates, not passes.

## 4. Trust-boundary review and repair

Bernoulli (`gpt-5.6-luna`, max reasoning effort) independently rejected the
prior tree `21f875318f61c30f55bf0a47b8daa4326d48540a7` after reproducing
taskfmt final/parent aliases and Rust verifier symlinked parent paths. Raw
review evidence:

```text
/tmp/campaign-review-evidence-21f87531.7EkYOO/
```

The taskfmt defect is repaired by `8e783592afd0a2c2f08076858a386a091a35e712`.
The Rust defect is repaired by `bb574d84bf25ff9179b42e951fca52068f2ab623`.
The Rust repair root below includes `final-identity.txt`, sealed nextest,
Clippy, and rustdoc results; all sealed exit files are zero:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/symlink-parent-repair-2026-09-19
```

This closes two preparation defects. It is not final verifier/reviewer
approval; no current-tree `VERIFIED` result exists.

## 5. Visual and behavioral calibration evidence

The older run:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/calibration-frozen-connections/full-run-1
```

records 302 selected, 301 passed, 1 failed, 2 skipped, exit 100. It used an
invalid mixed/dirty source and shared target arrangement, so it cannot be
accepted. Its `metadata.txt`, `stderr.log`, `form-advanced-artifact-check.txt`,
and `mismatch-paths.tsv` remain raw evidence only.

The exact-tag control under:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/calibration-baseline-4a79c0a2
```

did not establish a complete clean replay. No output was blessed. The 892
source control did execute all 302 cases and produced 7,550 artifacts for each
of ANSI, plain text, PNG, and HTML, but the summary records four failed
surfaces, two skips, one leak, exit 100, and 17 diff files. It is complete
coverage evidence, not an accepted calibration. A deterministic clean rerun
and altered-output negative control remain blockers.

The final matrix must exercise exact ANSI/plain/PNG/HTML equality, all five
sizes (`72x20`, `80x24`, `100x30`, `120x40`, `160x50`), all five color modes,
all applications/fixtures/routes/checkpoints, and PTY setup/input/resize/
settled/exit/cleanup. It must prove content, grapheme/cell/style/cursor,
geometry, interaction, lifecycle, and behavior parity. It may not use copied
expected files, automatic blessing, broad tolerance, masks, or arbitrary
sleeps.

## 6. Rejected verifier/reviewer evidence

These roots are preserved but rejected or stale. They must never be treated as
current receipts:

```text
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed-proof-requal-b20ca5c6
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/reviewer-final-proof-requal-b20ca5c6
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-211c29ad-independent
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/verifier-final-sealed-8da4c2b8
/Users/donbeave/Projects/terminal-components-claude/.codex-runs/campaign-prep-2026-09-19/reviewer-final-b20ca5c6
/tmp/campaign-review-evidence-21f87531.7EkYOO/
```

They bind earlier candidates, report explicit rejection, or lack the required
complete current-tree evidence. Bernoulli's rejection was repaired through the
two trust-path commits, but it is not a final approval. No post-documentation
verifier or independent reviewer has returned `VERIFIED`.

## 7. Ledger and evidence freshness

The runtime ledger is `.campaign/ledger.json`, schema `campaign-ledger/v1`,
with four blocked historical rows and no accepted production rows. Its required
state is `armed=false`. It is ignored runtime state, not a tracked receipt.

The sealing protocol is intentionally two-phase: commit the exact preparation
payload first; then materialize external immutable verifier/reviewer manifests
that bind the resulting HEAD, tree, parent, branch, scope base, task contracts,
graph/catalog hashes, oracle, comparator, taskfmt, proof binary, environment,
results, and dependency ancestry. A receipt cannot hash a future commit that
contains itself. Any relevant change invalidates affected evidence. Ancestor
evidence never silently authorizes a changed tree.

## 8. Open blockers and disposition

| issue | state |
| --- | --- |
| preparation readiness | NO-GO |
| production dispatch | not performed |
| ledger | disarmed |
| accepted preparation receipt | none |
| protected baseline | unchanged |
| current product parity | unproven; known regressions remain |
| full oracle calibration | not accepted |
| 892 calibration result | 302 cases: 298 passed, 4 failed, 2 skipped, 1 leaky, exit 100; 7,550 ANSI/plain/PNG/HTML each; 30,200 total; 17 diff files |
| `parity/evidence.tsv` | missing; boundary gate fails |
| `TASK-071` / `TASK-072` dependency receipts | absent |
| native Linux | unavailable |
| taskfmt Docker integration | not run; fails closed |
| current independent verifier/reviewer | no `VERIFIED` result |
| final external seal for this documentation tree | not yet created |
| repaired trust paths | defects fixed; exact final-tree verifier/reviewer requalification pending |

The next implementation prompt is deliberately marked **NOT AUTHORIZED FOR
EXECUTION**. Startup must reject while the report is NO-GO, the ledger is
`armed=false`, dependency receipts are absent, native Linux is unavailable,
parity evidence is missing, or no accepted final-tree preparation receipt and
independent `VERIFIED` review exist.
