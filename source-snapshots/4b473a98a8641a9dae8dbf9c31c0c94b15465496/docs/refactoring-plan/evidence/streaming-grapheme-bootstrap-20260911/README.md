# Streaming grapheme feasibility qualifier

Planning-only prototype for branch finding BA-FMT and proposed ADJ-14; no TC production implementation or baseline change. The actual public consumer is pinned main List -> custom RowUi callback -> existing atomic-cluster label_fmt -> Scene. The original user Display is invoked exactly once per visible row callback. The adapter is not claimed to preserve production style-query cardinality: TASK-013's final shared painter resolves/applies the style once and TASK-015 migrates the real default Display path.

## Source and reproduction

Pinned main checkout used read-only: `/private/tmp/tc-architecture.L0vAdH/main`, HEAD `7b27732a8c3c131760ec3438f641cb3c11343a42`. Verified Git blobs: rowui `960a03bc2c2722eac2b32448642dc1e31575b6a9`; ui/mod `8cdb284353263620d62442ac201459f289179309`; runtime `bca132db3e0c75dcbe6b7325d4889ca390730a1c`. Dependencies ratatui-core 0.1.2 and unicode-segmentation 1.13.3 match main's lock. This fixture lock is independently recorded; do not mistake its remaining dependency resolution for the complete product lock.

Copy these authored fixture files into a fresh disposable directory, preserving the lock. On another host replace only the two manifest path dependencies with a read-only exact-pin checkout; do not edit that checkout. Run:

```sh
rtk proxy cargo test --manifest-path /tmp/stream-projection.ghBlfQ/Cargo.toml --offline --locked --release -- --nocapture --test-threads=1
rtk proxy cargo clippy --manifest-path /tmp/stream-projection.ghBlfQ/Cargo.toml --offline --locked --all-targets -- -D warnings
rtk proxy cargo fmt --manifest-path /tmp/stream-projection.ghBlfQ/Cargo.toml --check
```

Observed Rust 1.98 release: seven tests PASS; strict Clippy PASS; formatting PASS. The toolchain emitted a missing libLLVM warning while stripping debug info, but linked and executed the test binary successfully. Doctests: zero, because this is a private test fixture. MSRV/other-host execution is not claimed here.

## Concrete observations

- Exact Unicode cluster outputs at every UTF-8 split boundary for empty/ASCII/CJK/combining/ZWJ/RI/CRLF/Indic conjunct inputs and a 4096-mark cluster; empty writes do not delimit clusters. UnicodeSegmentation over the complete test input supplies the independent expected boundaries, not the streaming implementation.
- Scratch is inline 64 bytes plus an overflow String containing only the unfinished cluster and at most one scalar lookahead. Actual peak is at most largest encountered cluster bytes +4. Checked inline UTF-8 is bounded; overflow uses already-valid String::as_str, not repeated full-prefix validation.
- ASCII, CJK, e+acute and the 25-byte family ZWJ: scratch cold/warm/fresh-owner allocations all zero. “Warm” means the same caller-owned scratch capacity is retained.
- Retained capacity is bounded by the lifetime high-water largest cluster H for this same scratch/Runtime owner, not by a later short draw; live pending bytes remain bounded by the current unfinished cluster +4. New tests exercise long→short→long, fresh-owner cold allocation and actual List allocation resize20→8→20 on the same Scene, comparing every first resulting frame. Logical scratch content is empty between uses. The 256/4096-mark cold ceilings5/9 are now asserted, not merely printed.
- `a + 4096 combining acute + z`: peak 8194 bytes; scratch cold/fresh 9 allocation/reallocation events, cumulative requested bytes 16128, retained capacity 16128; warmed scratch 0 events/0 bytes. These are measured fixture values, not universal promises about every allocator or implementation.
- Actual List cells equal unsplit reference for e+acute, family ZWJ, RI pairs, CJK plus suffix, and 256 combining marks at widths 1/2/3/8/20 across two draws. At widths too narrow to invoke the row callback, no Display invocation is falsely claimed. Where invoked, the original Display is called once and all three write calls occur.
- Public width8/20 long-cluster case: scratch cold 5 / warm 0; the atomic painter+style scope still allocates once on each draw for the long cell symbol. A 25-byte family ZWJ likewise has one painted-cell allocation on each draw with zero scratch allocations. Do not report total warm zero by subtracting output storage silently.
- Once clipping ends projection, later formatter writes still execute their caller effects and return normally; a 4096-mark invisible suffix causes zero scratch allocations and no continued segmentation.
- A returned fmt::Error still flushes its successfully written prefix; unwind clears scratch without emitting a new partial cluster. Clearing uses the same safe best-effort fill/black_box/compiler-fence contract as TC Secret, not guaranteed erasure of allocator copies. Masking remains before projection; this does not authorize materializing a Secret via Display.
- Real isolated rejection: change Scratch::text's overflow `return self.heap.as_str()` to `return self.checked_text(self.heap.as_bytes())`. This compiles and the actual counted UTF-8 byte-work guard rejects “revalidated growing heap prefix.” `reject-quadratic.log` records exit101. The final source restores the correct branch; `positive.log` records the subsequent seven-test PASS.

## Production owner feasibility and limits

Main Ui directly borrows mutable FrameState (ui/mod.rs:318–326); Runtime owns one FrameState (runtime.rs:360/418). FrameState::reset clears output collections without replacing the object, while publication swaps individual registry/ring/binding fields rather than FrameState wholesale. Therefore a private FrameState text scratch field is a concrete per-Runtime reusable capacity owner, requiring TASK-013 ui/mod.rs scope, not a global/static or new public Runtime API.

The final implementation must use a scoped guard that takes/restores this private scratch around the shared painter, clears content on success/error/unwind, retains capacity only, omits content from Debug and never publishes scratch in FrameOut/state/effects. Public consumers receive no scratch parameter. A new Runtime is cold; independent runtimes cannot share capacity. Existing borrowed text allocation gates remain unchanged.

The prototype drives GraphemeCursor with an open-ended length and explicitly flushes the unfinished cluster at formatter completion; it never calls the cursor at a fake EOF. This is a bounded feasibility demonstration against the exact locked library, not a new upstream API guarantee or permission to copy unchecked arithmetic. Product proof must retain checked offsets, original-byte provenance, current clip/wide-shadow/tab/control semantics and first-byte style ownership. No full styled-line, runtime lifecycle, selection/copy, secret redaction or full application parity closure is claimed by these seven tests.
