# BA-ART-01 atomic capture qualification proposal

This is a read-only seam proposal awaiting coordinator approval, not an implemented qualifier or passing receipt.

## Observed gap

The artifact audit found 17 archived capture sets with genuinely different time states across representations despite internally valid files and hashes. The existing runner worker records semantic JSON before/after frames and an extracted value, but has no decoded terminal cursor, raw stream boundary or generation-bound exported representations. The runner's complete observation replay check therefore does not establish this missing property. The comparator's 72-case corpus does reject rehashed cell/state changes against expected artifacts, but its current sidecar has no independently observed atomic-generation identity. Agreement with a previously mixed oracle set would not prove that set was captured atomically.

The enabling condition is that file integrity and checkpoint labels can authenticate individually valid members without establishing a shared production observation boundary. Adding another freely submitted generation string or hashing the mixed bundle does not repair it.

## Minimal independent seam

Use unique preparation-owned `capture-atomicity-bootstrap-*` files for driver, Rust producer, Rust decoder/observer and protocol. Reuse the frozen compiler identity and sandbox/FD transport helpers without changing their APIs. Require an exact offline tuisnap source/lock input, reviewed head `883d03f19d890bbbf27468798db78b04e85297ac`, not an installed executable inferred from PATH. TASK-070 already follows TASK-001, so the final capture dependency can be supplied by that accepted prerequisite; the preparation fixture still needs independently available pinned source and locked dependencies before its own qualification. No future TASK-002/003 baseline receipt is needed to define the test.

The producer is real executable source with one update state and one render path. Protected numeric input/tick actions alternate A/B. Each state differs in glyph, foreground, cursor position/visibility and a semantic counter/selection. Values and inert identity salts vary across fresh runs. Emit the terminal rendering and semantic checkpoint through a defined barrier: after the input/tick is processed and rendering completes, the producer publishes the semantic state and waits for the next explicit input. It must not update on wall time while the observer copies artifacts. The observer owns the PTY, input order and stream drain; it binds the completed raw stream interval to the acknowledged semantic checkpoint. A label without that causal boundary is insufficient.

The accepted decoder freezes one schema-3 Frame from that complete stream boundary. Validate it before use. Cells, cursor and every requested derived text/ANSI/HTML representation come from this one immutable Frame, not repeated reads of a live session. Semantic state comes from the same update/render barrier, not another later extraction. The observer privately records exact raw bytes/offsets, action ordinal, tick, source/tree/binary/tool identities and the resulting tuple before returning any data to the submitted runner.

The submitted runner receives a protected operation/profile and observed raw material through the existing capability-limited pipes. Its capture acceptance must validate the bundle against the independently decoded observation and semantic checkpoint. The private fixture oracle selects neither verdict nor expected cells from submitted reports. Candidate output retains exact canonical observation bytes; ordinary JSON equality cannot hide boolean/integer/float type changes.

This seam qualifies source-owned atomic observation and serialization on a deliberately small production fixture. It does not qualify all TC application extraction seams, all terminal exports, or PNG/font fidelity. Those retain their existing TASK-001 and baseline/application owner proofs. No handmade model dictionary may replace the actual decoder or production state extraction.

## Required discrimination corpus

Coherent A and B are independently observed positives, including a changed action/tick that legitimately reaches the alternate state. Reject every nonuniform combination of A/B cells, cursor and semantic state. Also reject text/ANSI/HTML derived from the other generation, a second live read after advancing the producer, stale action/tick, reordered/missing checkpoint and reuse from a different run. Recompute every submitted file/bundle hash in these negative cases and relabel all candidate generation fields consistently, so a hash-only or equality-of-labels checker still fails.

Preserve genuine observer digests during output-only byte/type forgery controls. Add whole-observation replay, zero-worker and always-pass adversaries. Each negative must follow a real successful producer/decoder invocation and fail for its intended atomicity cause; build/isolation failure is not credit. Every negative gets fresh coherent A and coherent B recovery, preventing an implementation from accepting only one phase or refusing the entire profile. Normal and optimized Python execution must agree. Independent review runs the final fixed bytes after author tests.

## Task and freeze integration

Make this an additional mandatory TASK-070 R-001/AC-001/CHK-004 capture corpus, preserved by R-003/AC-003/CHK-005. TASK-001/002/003 schema and artifact contracts require one host-observed frame/state checkpoint identity and forbid independent live recapture of derived members. TASK-072 extraction ownership remains necessary but cannot substitute for this atomic capture proof. The existing comparator corpus remains mandatory unchanged unless a separately reviewed schema extension requires new cases.

The coordinator owns the exact protected asset group and all source/destination bindings. New code must not modify shared helpers during active qualification runs. Before implementation, approve the exact operation/profile shape, error category, tuisnap input packaging and four new file paths. The proposal does not authorize rewriting the 17 historical capture sets, normalizing their changing cells, or accepting any existing archived member as a fresh oracle baseline.
