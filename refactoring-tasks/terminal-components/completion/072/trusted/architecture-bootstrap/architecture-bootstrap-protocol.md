# Independent Rust architecture qualification

Planning owner: `/root/history_early`; independent reviewer: `/root/review_rust_qualification`. Neither is TASK072's implementer. These are qualification subjects, observers, and expected verdicts—not a terminal-components verifier implementation. No production source, public API, or baseline is changed. TASK073 is not a dependency; the `073 → 008 → 072` cycle is prohibited.

## Frozen check and authority

Reserved TASK072 AC-009 / CHK-011 invokes:

```sh
python3 /task/trusted/architecture-bootstrap/architecture-bootstrap-driver.py --runner /work/tools/refactor-proof/bin/tc-proof
```

The command runs both standalone Rust and actual terminal-components matrices. The planner freezes every file/digest before dispatch. The implementer cannot modify them or their expectations. The sibling `/task/trusted/runner-bootstrap/` package includes its driver, app, worker, and host observer because the inherited fixture constructs initial disposable Git authority. Final assembly and independent review must pass before the readiness flag is enabled. The candidate checker is later TASK072 acceptance, not a prerequisite for independent fixture preparation.

The architecture package contains:

- `architecture-bootstrap-driver.py`, `architecture-bootstrap-vectors.json`;
- `architecture-bootstrap-actual-driver.py`, `architecture-bootstrap-main-driver.py`;
- `architecture-bootstrap-state-vectors.json`, this protocol;
- `architecture-bootstrap-fixture/{app,library,main}.rs`;
- `architecture-bootstrap-{main,conformance,rain,nested-row}-probe.rs`;
- `main-source.tar.gz`.

The deterministic gzip SHA-256 is `ab9568812e86e3c8e18c1d945886d4b2387b1e175ad61ba4d60750b5d6f98119`. Decompression must yield tar SHA-256 `732e40d9cf82e98e3d6375aaeecee349e8cb774abc5094dbbff145555201633a`, the exact `git archive 7b27732a8c3c131760ec3438f641cb3c11343a42` bytes. Both hashes are checked before tar parsing, then seven material original Git blobs separately. The entire immutable source corpus is bound. Absolute/traversal paths are rejected; symlinks are not extracted/followed. Raw tar is not a checked-in or task-package asset.

## Finite expected verdicts

The standalone R001–R041 matrix has seven positives and thirty-four negatives. Its three actual Rust files implement a small contract model, **not** junie-tui. Thirty-eight subjects compile; three intentionally fail. Sixteen runtime mutants alter actual state/cell/write/query observations; fifteen compile-valid negatives require source analysis. Reachable real receiver cases include value/shared/mutable receivers, typed Self/references, Box, Rc, Arc and Pin. An explicit `this: &Self` data argument remains positive. Private free/associated helpers, dynamic/unconfigured constructions, test fixtures, trait projection and disjoint art are positive controls. Missing/empty roots, unreferenced malformed files, and malformed tails after test modules fail closed.

The actual-TC matrix contains twenty-five subjects:

| Subjects | Expected verdict and independently observed premise |
| --- | --- |
| Eleven original/perturbed Showcase subjects | Reject: other known app defects remain even if one overpaint is removed. Production update/draw produces two complete 120×40 canonical-cell checkpoints per subject. |
| Whole original conformance registry | Reject: all 44 registrations execute from the existing macro invocation; owned-resolution/declaration differences remain defects, not waivers. |
| Focused public Button consumer | Pass: exact original ButtonCase source slice, actual public library, generated one-case registry, frozen 7-state × 3-geometry corpus, exact owned PARTS union. |
| Eight focused Button/row variants | Reject owned extra, declared unreachable, missing registry invocation, ignored ICON SlotFn, measurement query, actual rustdoc slot drift, omitted state, or fabricated real RowUi owner. These contrast with the positive focused subject. |
| Actual nested RowUi/ColumnsUi | Pass; corrupting the existing retained outer owner after nested return rejects. No fictional public attribution flag. |
| Actual Jackin rain plus actual Button | Pass: rain changes outside cells while Button cells, registration, focus and two real activations remain unchanged. |
| Existing external-author component | Pass: actual Rust example tests execute rendering/override and reference behavior. |

Every rejection is followed by a fresh positive recovery. Together: 66 cases and 55 recovery executions. Neither always-pass nor always-reject qualifies. Exact immutable source variants are restored using complete file-inventory equality; previously injected files are removed only within observer-owned temporary trees. No project checkout or project commit occurs.

## Actual production causality and ownership

Changing Grid's actual MetricModel does not change overpainted original cells; removing compatibility painting exposes both model and perturbation. Dead/inert Grid variants retain the initial fixed picture but break production activation. Removing Brand/StatusBar or changing Brand text leaves original chrome cells unchanged; exposing library chrome restores Brand text causality. These are measured main negatives, never an accepted-defect allowlist.

One wrapper consumes the existing conformance macro invocation for normal, clipped and zero-size cases. The separate state manifest fixes every original case's exact states; observed state/geometry sets must equal it, with duplicates rejected. Capability-implied states are also asserted in actual Rust. A changed state declaration cannot shorten expectations. This finite corpus does not claim current conformance already covers every eventual component behavior: measured gaps stay negative.

The private observer adds `track_caller` and logging at existing style-observation boundaries. Transparent row/column methods preserve real caller locations: library-default rows remain component-owned; frozen external callbacks are row-owned. Original ID, family, variant, part and Resolved values remain intact. Same-ID child composition counts; custom row/column parts do not. Real before/inner/after cells and existing owner fields prove nested carrier restoration; fake IDs or leaked inner owner fail. This frozen adapter is not TASK073's future production provenance implementation and adds no public API.

Button's slot set is parsed from actual public Overrides rustdoc with a fail-closed source-pinned grammar. Every actual Button part is attempted: the sentinel changes real cells precisely on GUTTER/ICON/MARKER/LABEL across ready/busy/loading. CONTAINER is not slot-addressable. Returned geometry and unrelated neighbors stay unchanged; live Harness checks compare all part-hit rectangles, ring/focus, click and Enter activation. Measurement cannot append a paint query. Actual mutations demonstrate ignored slots, false measurement proof and prose/paint disagreement.

## Context, transport and source identities

The interface remains exactly `tc-proof architecture --context <file>`. Protected extensions of `tc-proof-runner-context/v1` are not new CLI flags. Requests retain `tc-proof-runner-observe/v1` with exactly `schema`, `nonce`, `operation`, `source_commit`, `tree`; one pre-bound request is allowed. Result ABI remains `tc-proof-runner-result/v1`. Pass: exit zero, passed, null category, exact outputs `{"observations": [actual_event]}`. Reject: nonzero, rejected, ARCHITECTURE, empty outputs. Event digests match observer memory. Missing/forged/replayed observations or changed protected inputs fail.

`tc-architecture-rust-profile/v1` supplies standalone roots/hashes, phase/props/component symbols, registry/parts/slots symbols and fresh seed. `tc-architecture-actual-rust-profile/v1` supplies actual source directory, roots, entries, dependencies, full source hashes, main/archive authority and fixture kind/check scope. Roots may be explicit Rust files or directories; every claimed file and tail must parse. File-scoped positive external consumers do not waive checks on full production applications. Showcase claims its entire app source. The focused Button fixture is genuinely a standalone one-case consumer, not a claim that it registers all library components.

Conformance profiles carry immutable required state vectors from the frozen manifest. Subjects/kinds describe source roles, not verdicts; no vector ID, expected pass/reject value or mutation name is supplied. The candidate must use its production parser and architecture analysis, not match fixture spelling or return stored verdicts. Independent review of the built dispatcher/shared analysis path is mandatory during acceptance; a finite black-box suite alone cannot exclude hard-coding.

The generated fixture Git tree differs from main and includes the initial runner fixture plus exact TC subject. Its source_commit is that fixture's original oracle, not the immutable UI oracle. Separate main_commit/full archive/individual blob bindings establish TC provenance. Compiler source hashes and permitted frozen private instrumentation establish executed bytes. Do not conflate these identities.

## Protected execution and preparation checks

Before the first candidate process, every subject, executable, mutation and verdict is prepared/frozen. Cargo builds are locked/offline; compiler/Cargo paths, hashes, versions, lockfile and executable hashes are recorded. Intentional fixture faults use cap-lints=allow; project lint gates are unchanged. Missing cached dependencies or the Darwin observer fail closed. Candidate requests cannot choose argv/source/executable/configuration/expected output. Candidate writes only output; private authority, task ports, signals and network are inaccessible.

Actual binaries run under the independent Darwin observer. Full stdout/stderr, including real cell/query evidence, is returned as base64 in `tc-architecture-actual-rust-observation/v1`. Raw hashes remain event-bound. Replay compares verdict and semantic output; only libtest's anchored final-result wall-duration field and the numeric runtime thread ID in an anchored panic header are excluded from semantic equality. Failed-binary relocation directly measured that thread-ID difference. Test names, source locations, assertions, state, cells and all other bytes remain compared. Negative Rust premises require exit 101 and their independently fixed assertion diagnostic, not any panic. Raw events are never normalized or substituted with candidate logs.

Preparation commands:

```sh
python3 -B architecture-bootstrap-driver.py --self-test
python3 -B architecture-bootstrap-actual-driver.py --self-test
```

The actual rehearsal includes a real external FD bridge, positive recovery, always-pass rejection, forged events, zero-observer success, and relocated passing/failing binaries. The transport-only test double is not the production checker. No self-test emits a TASK072 completion receipt or certifies final application architecture/parity.
