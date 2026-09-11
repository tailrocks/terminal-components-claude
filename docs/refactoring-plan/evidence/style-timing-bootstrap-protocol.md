# Actual style timing bootstrap protocol

Preparation-only qualification. Not a production instrumentation implementation;
not TASK067's product performance acceptance; not evidence that TASK073 is done.
TASK070 and TASK072 may consume this frozen qualifier before TASK073 implements
the production observer. No self-acceptance dependency on TASK073.

## Instrument correctness versus real measurement quality

The deterministic control/recovery corpus executes actual compiled Rust App,
all production resolution paths and complete frames, but uses explicitly
injected arithmetic clock readings. Its successes qualify observation/protocol
logic, never performance. Physical whole-frame boundaries are still recorded;
a separate effective-clock ledger binds arithmetic test durations and detects
forgery without confusing them with physical elapsed time.

Three mandatory actual Instant attestations run ordinary Lists,
256additional genuine binds inside the existing style resolver, and64repetitions
of existing paint-cell writes. Cost causality is proved only from these actual
intervals and whole-frame denominators. Every sample is retained.
The host computes quality: negative correction→`INVALID`; otherwise conservative
upper ratio>0.05→`REJECT`; otherwise `PASS`. Candidate attestation must match that
fresh actual verdict. Source validity never makes an invalid/noisy measurement
pass. No retry, sample discard or threshold/profile tuning occurs. Only this
quality function—not predicted wall time—is frozen before attestation dispatch.

A fixed diagnostic series showed one negative correction among12fresh Instant
replays. Source-positive controls therefore cannot be mislabeled as guaranteed
timing-positive runs. Arithmetic recoveries address that classification error;
mandatory real attestations keep actual measurement failures visible. TASK067
still requires controlled-host valid real-clock ≤5% evidence.

## Inputs and command

`python3 -B /task/trusted/style-timing-bootstrap/style-timing-bootstrap-driver.py --runner /work/tools/refactor-proof/bin/tc-proof`
qualifies the submitted `tc-proof architecture --context PATH` dispatcher.
`--self-test` instead supplies an independent external FD consumer plus malicious
consumers. Repeat self-test under `python3 -O -B`; Python assertions are not gates.

Input main is commit `7b27732a8c3c131760ec3438f641cb3c11343a42`, exact compressed
`main-source.tar.gz` SHA-256
`ab9568812e86e3c8e18c1d945886d4b2387b1e175ad61ba4d60750b5d6f98119`;
decompressed git archive SHA-256
`732e40d9cf82e98e3d6375aaeecee349e8cb774abc5094dbbff145555201633a`.
Existing main bootstrap validates archive identity and extraction. Every source
change occurs in a disposable `tc-architecture-main-style-*` source copy.
Compiler/toolchain bytes, lockfile, source inventory, flags and resulting binary
bytes are recorded before any candidate runs. Neither working production files
nor production branches are modified or committed.

## Actual workload and ownership

The denominator binary is the actual pinned main Showcase App at Lists, Junie,
120×40, driven by its existing production Harness. The caller observes exactly
one `Harness.draw()` with `Instant`; no synthetic frame, digest loop, multiplier,
probe-work subtraction or unrelated work is admitted into that interval.
State extraction and serialization of every cell (including attributes), cursor,
focus/hover, App/ListPage semantic state, fixed application clock, effects,
capture/layer/quit happen outside the interval.

The paired binary uses the same source workload. Temporary testing-only Ui
references carry a caller-owned `Probe`; a shorter Ui reborrow scopes it. No
probe global, TLS, semantic component state or cache-key field is introduced.
The registered leaf entries are `style`, `style_inherited`, `style_defaults`,
`paint_patch`, `style_patched`, `resolve`, `surface_style`, `bg`, plus exact
`CellUi::drop`, `StatusBar::item_style` and Grid `apply_style_delta` bind expressions.
Their existing
complete bodies, including cache lookup, accumulation and binding, are enclosed.
`with_part` is a wrapper, not an additional timed entry.

The RowUi entry is essential: the120-column shell leaves a94-column page,
selecting Lists' compact row path, which reaches12 final-cell binds. Status's
nonempty-delta bind and Grid's delta bind have zero calls in this closed frame;
separate untimed production-branch consumers exercise their empty/nonempty
paths. `dim_layer` and its private color-binding paint helpers are outside this
closed, no-layer fixture; a reached `dim_layer` records an uncovered-path ID and
fails, rather than being silently zero or timing the whole paint operation.

Each entry independently records its invocation ID outside the timed interval.
Disabled mode produces this untimed membership witness, not a timing numerator.
The denominator comes from the separate source-uninstrumented binary, not the
witness-bearing disabled frame. Calibration brackets an adjacent empty pair;
the actual resolver still executes outside that interval. Measured mode brackets
the actual resolver. Interval records include nesting depth; wrappers, omissions,
duplicate timing and foreign membership cannot silently count as valid work.

## Frozen schedule and arithmetic

Release compiler profile: opt-level3, debug0, assertions/overflow checks off,
LTO off, codegen-units16, unwind, incremental off, no stripping. All inherited
Cargo/Rust environment overrides are removed; actual RUSTC is pinned and wrappers
disabled. Applied settings and actual compiler artifact profile are recorded.
Resolve and directly invoke actual toolchain Cargo/Rustc binaries, not mutable
PATH launchers; validate their frozen paths, bytes and verbose versions before
and after compilation, including the testing-disabled check.
12 whole-frame warmups; nine batches; each batch order
disabled, calibration, measured. Both actual binaries warm before the protected
host sends draw commands. For every slot the host releases one control draw,
waits for its record, then releases one subject draw. This fixed cross-binary
order pairs real full-frame controls with all three modes. Application time does
not advance. Benchmark time is `std::time::Instant`.

Mode outputs must preserve full frame/state, allocation counts/bytes, style
cache hit/miss deltas and complete cache slot/generation/count contents. An untimed standalone actual Ui consumer exercises every
registered entry with built-in, empty custom and defined custom families,
inherited/default/local paths, a same-key miss then hit and `with_part` callbacks.
It also exercises actual filled/empty RowUi cells and Status/Grid delta branches,
compares every branch-consumer cell and resolved/cache output, and tests normal
and unwinding restoration of shorter probe reborrows.
This consumer is never part of the Showcase denominator.

The protected judge recomputes interval sums and paired corrected durations.
Negative corrected durations reject; they are not clamped. It reports all raw,
calibration, corrected and uninstrumented denominator samples. Conservative
interval is `[min(corrected)/max(denominator), max(corrected)/min(denominator)]`.
For an Instant attestation `strict_performance_pass` requires nonnegative
corrections and an upper bound at most0.05. Profiles `protocol-arithmetic` and
`strict-arithmetic` set only the distinct `strict_arithmetic_pass` scalar;
`strict_performance_pass` is always false for injected clocks. These profiles
qualify deterministic controls; `strict` refuses injected
clocks. Profile `attestation` reports the fresh actual quality verdict and
requires rejection for `INVALID` or `REJECT`. The actual expensive-binding source
is both an arithmetic over-budget control and a separate real Instant
attestation. Every source rejection and attestation is followed by a fresh
actual-Rust arithmetic positive. An injected-clock fixture submitted as real
strict acceptance must reject. None of these mechanism qualifications certifies
the future product.
Injected clock cases still execute actual source work but qualify arithmetic
only; they are labeled and never used as real performance samples.
Real strict acceptance rejects injected clocks regardless of a numerically
passing ratio. A measured ordinary-source strict positive failed the fixed5%
bound during preparation; that failure is preserved, not retried or cured by
changing the benchmark policy. No product certification is claimed.

## Adversarial corpus and recovery

Fourteen required source-copy rejection classes: omitted leaf; doubly timed wrapper; foreign
numerator; constant numerator; disabled hooks labeled measured; calibration
skipping the resolver; reordered invocation membership; changed actual cells;
probe allocation contamination; inflated denominator; foreign calibration;
negative correction; changed policy; threshold-straddling uncertainty. Each
must compile, execute the actual subject and fail for its frozen specific cause.
The omitted-leaf class is expanded to separate actual RowUi, Status and Grid
binding omissions. An over-budget arithmetic control tests strict acceptance,
and its actual expensive-binding source receives a separate real attestation.
An injected-clock-as-real-strict negative closes clock-kind substitution.
Unrelated compilation/launch failures are fixture failures, never successful
negative controls. Every rejected mutant is followed by a fresh positive
process with the same real workload.

Additional source-copy causal positives add real work inside the existing
resolver, or repeat the existing paint-cell writes without adding resolution.
Each has its own source-uninstrumented actual full-frame control. The first
must raise the measured numerator; the second must raise the denominator while
remaining below the expensive resolver's numerator. Neither changes final cells.

## Protected transport

Context profile schema `tc-style-timing-actual-profile/v1`, kind `style-timing`,
contains public source identity, census and frozen ordinary invocation sequence,
policy, acceptance mode and clock provenance, never
expected classifications. Exactly one five-field
`tc-proof-runner-observe/v1` request binds nonce, operation, commit and source
tree. The private observer runs both protected compiler-produced binaries and
retains the authoritative event in memory. Raw stdout/stderr from both processes
and both compiler receipts are returned in a
`tc-style-timing-actual-observation/v1` payload. The candidate independently
checks the public contract against these actual observations.

The standard `tc-proof-runner-result/v1` result must bind run/context and match
the observer event digest exactly. A positive publishes exactly the observation;
a negative exits nonzero, rejects as `ARCHITECTURE`, and publishes no outputs.
Always-pass, forged-observation, zero-observation, and genuine-digest output-only
integer-zero→boolean/float substitutions must be
rejected. A fresh same-kind external positive follows each transport attack.
Source/executable writes, observation provenance and private expected verdicts
remain outside candidate authority.

Full source is represented as the exact pinned archive plus exhaustive subject
and control byte-override directories and explicit deletion lists, never as a
reduced source selection. Both complete reconstructed path→SHA256 maps and each
compiler receipt's inventory digest are provided. The private source tree binds
the archive and all override bytes; protected context binds inventory/deletions.
The checker reconstructs exact compiled bytes in memory or its owned output
directory. This avoids thousands of duplicate filesystem entries per replay
without removing any source from inspection. Full source copies are still used
for actual compilation before dispatch.

## Freeze inventory

New files: this protocol, `style-timing-bootstrap-driver.py`,
`style-timing-bootstrap-probe.rs`, `style-timing-bootstrap-frame.rs`.
Required existing protected dependencies: `architecture-bootstrap-actual-driver.py`,
`architecture-bootstrap-main-driver.py`, `runner-bootstrap-driver.py`,
`host-bootstrap-observer.py`, `runner-bootstrap-app.py`,
`runner-bootstrap-worker.py`, exact `main-source.tar.gz`.
The root coordinator owns asset copying, manifest digests, TASK070/TASK072
bindings and independent acceptance; the author does not close its own findings.
