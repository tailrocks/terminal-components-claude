# Independent atomic capture qualification

This preparation-owned corpus addresses BA-ART-01: independently valid representation files can belong to different production time states. It qualifies real PTY capture, decoded-frame/state coherence and representation origin on a small Rust source fixture. It is not a TC application, a reference baseline, a general rendering fidelity qualification, or an alternative to the existing runner/comparator/host suites.

## Protected inputs and build

Freeze `capture-atomicity-bootstrap-driver.py`, `capture-atomicity-bootstrap-producer.rs`, `capture-atomicity-bootstrap-decoder.rs` and this protocol. The driver imports the unchanged `architecture-bootstrap-main-driver.py`, `runner-bootstrap-driver.py` and `host-bootstrap-observer.py` from the same protected directory. The base Fixture additionally reads `runner-bootstrap-app.py` and `runner-bootstrap-worker.py` there. Copy all these dependencies exactly; do not substitute implementation-checkout helpers. The separate complete runner-bootstrap group, including its index/extensions dependencies, remains mandatory.

The operator supplies an external protected tui-snap Git object database at `/proof/bootstrap/tui-snap`, containing commit `883d03f19d890bbbf27468798db78b04e85297ac` and tree `dadbaa70facc317cfabb52f0374c1f3cdceb46a1`. Build source is extracted from that exact commit, never from mutable working-tree files, a moving branch or a candidate path. The source archive, lock, two fixture sources, compilation argv/results, actual resolved Cargo/Rustc bytes/version/path/toolchain and final executable digests are recorded. The decoder is an added binary only in a fresh private source copy; the pinned lock and all original source bytes remain unchanged. Builds are offline and locked. A missing prerequisite, compiler drift, source mutation, build failure or unsupported Darwin isolation fails preparation; none earns negative-test credit.

The source producer is compiled with actual Rustc and warnings denied. The decoder compiles through the pinned crate's actual `ansi::replay_raw`, `Frame::validate`, `Frame::text` and `render::ansi_dump`. No manually built Frame substitutes for PTY decoding. The fixture's small escaped-`pre` HTML uses the already decoded text; it proves common-generation derivation only. It does not claim production HTML styles, PNG fidelity, font coverage or raster acceptance.

## Actual source boundary

A real PTY supplies stdin/stdout, raw input and a 24×4 terminal. The fixture's Rust App owns value, generation and tick. Initial render and each explicit `+` action produce distinct A/B glyph/color, cursor position/visibility and semantic value/phase. A `+` advances the actual state and tick by the source-defined amount; no wall timer races the checkpoint. After completing all rendering bytes, the same source method appends a nonce-bound OSC packet with its actual semantic fields to the same stream, flushes and waits for the next input. This is the source-owned render/state barrier, not a self-selected artifact timestamp.

The independent observer withholds the next action until it has consumed that complete barrier. It records the cumulative raw stream, stream interval, input ordinal and exact extracted state. It invokes the actual pinned decoder once for that checkpoint and retains the resulting immutable validated Frame. Cells, cursor, text, normalized ANSI and fixture HTML derive from that one Frame. State derives from the same barrier. The observer separately checks narrow source premises—actual value/phase, cursor coordinates and complete 96-cell schema-3 dimensions—to catch a detached producer/decoder fixture. These premises are not replacement expected Frames.

Initial, first-update and second-update checkpoints all execute in one child; the producer then exits cleanly on `q`. Raw bytes include complete boundaries, not independently sampled representation captures. Each observed reference bundle binds checkpoint ordinal, action ordinal, source tree, tick, raw-byte digest and a generation digest over the actual raw bytes plus barrier state. The driver retains these observations privately before returning them to the submitted executable. Candidate labels or hashes never establish that authority.

## Capture interface

The submitted executable uses existing `tc-proof capture --context PATH --lane pty`, existing `tc-proof-runner-observe/v1` request and `tc-proof-runner-result/v1` response transport. It receives `capture_profile.schema = tc-capture-atomic-profile/v1` and `kind = atomic-generation`. The protected profile records exact source/compiler identities, dimensions, frozen numeric seed/actions and one required checkpoint (1 or 2); it contains no private case name or expected verdict. Source/tree/run/context identities remain bound through the existing protected environment and inherited capability-limited pipes. The candidate cannot choose an executable, decoder, path, environment or action sequence for observer execution.

The actual observation payload has schema `tc-capture-atomic-observation/v1`, three decoded reference bundles, three source/raw/decoder receipts, actual producer exit/stderr and the candidate artifact bundle list. References and receipts come from protected execution, not submitted JSON. Each bundle contains exact binding, six representation files and a sorted manifest of path, byte length, SHA-256 and generation. The one required submitted bundle must match the exact protected checkpoint in every representation after complete membership, provenance and manifest checks. Unknown or omitted representations, checkpoints and fields fail. Raw action/state receipts must retain their source order. The operator must not accept a mixed reference merely because candidate artifacts reproduce it.

Valid output is the existing passed result with exact canonical observed bytes. An atomicity violation requires nonzero exit, status `rejected`, category `EXECUTION`, one genuine observer digest and empty outputs. Missing actual execution is never an acceptable rejection. Bool/int/float JSON aliases must not survive canonical-byte equality. Candidate-produced digests cannot replace the observer's private event list.

## Discriminating corpus and recoveries

The corpus has coherent A and coherent B positives plus 20 negative cases:

- Six nonuniform A/B combinations of cells, cursor and semantic value/phase, plus a fully foreign tuple relabelled as the required checkpoint.
- Text, normalized ANSI or fixture HTML from the other real decoded generation; all three exports obtained from a later live read while the primary frame stays earlier.
- Wrong action ordinal, wrong tick, stale generation digest or raw-stream digest from another checkpoint.
- Missing, duplicate or reordered checkpoints.
- Semantic integer fields changed to booleans or floats.

Every mixed artifact is sourced from actual decoded A/B output. All file sizes, hashes and manifest generation labels are recomputed after mutation; semantic generation/action/tick labels can agree while actual value/phase belongs to another state. Thus accepting intact files, equal labels or a coherent but wrong checkpoint cannot pass. No negative fails solely through an incidental compile error. The private driver checks that each actual source execution still discriminates its intended case before judging the submitted runner.

Every negative is followed by two fresh processes/fixture roots, one coherent A and one coherent B, with fresh numeric values, nonces and run identities. There are 22 primary cases and 40 fresh recovery invocations. Preparation additionally rejects always-pass, zero-worker, hash-only, label-only and output-observation forgery adapters, including retained-digest boolean and float aliases. Normal and optimized Python runs must agree. These adapters test the acceptance boundary; they are not the future project runner.

## TASK-070 invocation and preservation

The mandatory CHK-004 and CHK-005 command runs the existing complete group070 driver first, then this additional corpus:

```sh
python3 -B /task/trusted/capture-atomicity-bootstrap/capture-atomicity-bootstrap-driver.py \
  --runner /work/tools/refactor-proof/bin/tc-proof \
  --base-driver /task/trusted/runner-bootstrap/runner-bootstrap-driver.py \
  --tuisnap-source /proof/bootstrap/tui-snap
```

CHK-004 maps R-001/AC-001; CHK-005 maps R-003/AC-003. Existing comparator CHK-008, architecture and close gates remain mandatory. The base-driver argument is protected host configuration, never supplied by an application/executor. The atomic command cannot report complete runner qualification after skipping the base group or its index/extensions. Base and submitted executable bytes must remain fixed through the chain.

Preparation command (no candidate required):

```sh
python3 -B docs/refactoring-plan/evidence/capture-atomicity-bootstrap-driver.py \
  --self-test --tuisnap-source /absolute/pinned/tui-snap
```

Run it again with `python3 -B -O`. Independent review of final sources, real PTY outcomes, type/transport attacks and coherent recoveries is required before asset freeze. The coordinator owns TASK-001/002/003 atomic receipt/schema clauses, public authority and exact freeze mappings. No historical artifact is rewritten, normalized, blessed or silently accepted by this qualification.
