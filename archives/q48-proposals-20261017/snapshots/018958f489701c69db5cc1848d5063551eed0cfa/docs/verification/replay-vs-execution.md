# Replay vs execution qualification (FIX-011 / Q07)

Slice Q07 (ref stream S2) reacquires the replay-unknown cursor state from real
`visual-baseline` tag execution and qualifies which conformance expectations
are replay-derived versus execution-derived. R1 is the unknown-original-state
gap; R2 is the never-compared-observations gap.

## Verdict

- **R1 (cursor): closed by acquisition.** All 302 LEGACY roots reacquired from
  live tag PTY execution at two combos each (604 observations). Zero match the
  replay default. Per-case method records live in
  [`../../tests/conformance/cursor_provenance.json`](../../tests/conformance/cursor_provenance.json),
  gated by the `cursor_provenance` conformance test.
- **R1 (focus/motion): unobservable, documented as residual.** The
  tuisnap/tuiscotti `Frame` schema has no focus or motion fields (verified in
  both pins), so no execution run can reacquire them as state. Motion mode is
  recorded in argv where the suite pins it (`--motion paused --frame 40`);
  focus is implicit in settled cells only.
- **R2 (cursor): already closed by Q06 — verified, no gate change.**
  `compare_bundle_dirs` compares approved-vs-actual `.observations.json` over
  the stable subset (dims, cursor x/y/visible, cell counts, frame digest,
  tool identity) and compares the full cursor (plus style/blink) through
  decoded `.frame.json` `diff_cells`. The stable subset drops no semantic
  field: the dropped keys are bundle identity (`name`, `legacy_name`) and
  volatile provenance (`argv`, `created_unix`).
- **R2 (focus/motion): nothing to gate.** Approved and actual
  `.observations.json` carry no focus/motion keys (corpus-wide grep: zero
  hits), so there is no semantic field to extend the gate with short of
  inventing a schema and rewriting the frozen approved corpus — forbidden.

## Expectation provenance by class

| Expectation | Derived from | Authority | Residual risk |
| --- | --- | --- | --- |
| `.ansi` / `.txt` bytes (7,550 captures) | Replay: ANSI re-converted from legacy `snapshots/` | `legacy_replayed_conversion` (admission record) | Conversion fidelity, not live behavior; live PTY bytes could differ in cursor-addressing sequences the replay never emitted |
| `.frame.json` cells | Replay conversion | Same | Same as above; cell content matched 100% at admission per the admission summary |
| `.frame.json` cursor + `.observations.json` cursor | Replay **default** `(0,0,hidden)` for all 7,550 — contradicted by live execution on 604/604 sampled captures | Q07 acquisition record (this slice) | Live-vs-approved cursor comparison is now gated (Q06), so a candidate that reproduces true live cursor state will *fail* against replay-default approvals until an `ExtractedOracle` refresh is reviewed and blessed |
| `.png` / `.html` / `.ascii` + companions | Rendered/exported from replay frames | Same replay root | Inherits the cursor caveat wherever the renderer paints a cursor |
| Cursor style / blink | Live: uniformly `Block`, non-blinking (604/604) | Q07 acquisition record | None observed; still gated via frame diff |
| Focus state | No channel in any schema | None | Focus regressions are detectable only indirectly via cell diffs |
| Motion state | argv pin only (`--motion paused --frame 40`); stills freeze phase | Suite argv (recorded per case in the provenance record) | Live-motion divergence is out of scope for still-frame parity by design |

## Acquisition method (per case)

Method id: `live-pty-tag-execution`. For every `LEGACY:<root>` registry id:

1. Built the four tag binaries (`showcase`, `tablepro`, `jackin-preview`,
   `holla`) offline from a read-only `visual-baseline` archive copy
   (`rustc 1.98.1`, tuisnap `2d43458a`, Darwin arm64).
2. Ran the tag-era `visual_baseline` PTY suite unmodified except for a
   scratch-only probe that (a) filtered to one combo per sweep and
   (b) dumped the settled live `Frame` cursor instead of gating against
   `snapshots/`. Runner, needles, sends, settle (400 ms), timeouts, and env
   hygiene are the tag suite's verbatim.
3. Sweeps: `100x30/truecolor` (303 passed / 0 failed, 302 records) and
   `80x24/truecolor` (303 passed / 0 failed, 302 records). Two roots re-ran
   byte-identical (digest + cursor).

Record fields per root: `registry_id`, `legacy_root`, `target_root` (via the
snapshot migration map), `method`, `argv_class` (`run-once` with normalized
argv, or `live-session` for the 28 pointer/live flows whose argv is `[]`),
per-combo cursor + frame digest + verdict, size-stability flags, and the
replay default for contrast.

Headline: **0/604 live observations match the replay default**; 78 roots show
a visible live cursor at both sizes; visibility is size-stable 302/302 while
position is size-stable only 68/302.

## What was NOT done

- No approved baseline was read as mutable input and none was written; the
  `baselines/` tree and the `visual-baseline` tag are untouched
  (`visual-baseline^{commit}` still `4a79c0a2…`).
- No tag code was copied into the workspace. The external checkout
  (`/tmp/termrock-ref-4a79c0a2`, `chmod a-w`) served acquisition only; the
  writable build scratch (`/tmp/termrock-tag-build`) held the probe patch.
- No writer, admission, or gate code changed: Q06's gate already compares
  every semantic observations field, and focus/motion have no fields to add.
- The provenance record is a qualification input, not an approval: promoting
  any live value into expected output requires the `ExtractedOracle` lane
  (reviewed adapter + artifact binding), per
  [oracle-and-provenance](oracle-and-provenance.md).

## Reproduction

```sh
# 1. External read-only checkout (never inside the workspace):
mkdir -p /tmp/termrock-ref-4a79c0a2
git -C <worktree> archive visual-baseline | tar -x -C /tmp/termrock-ref-4a79c0a2
chmod -R a-w /tmp/termrock-ref-4a79c0a2

# 2. Writable build scratch (probe patch lives here, never committed):
mkdir -p /tmp/termrock-tag-build
tar -C /tmp/termrock-ref-4a79c0a2 -cf - Cargo.toml Cargo.lock src tests \
  | tar -C /tmp/termrock-tag-build -xf -
# add [workspace] isolation, apply the Q07 probe to
# tests/visual_baseline/support.rs (combo filter + cursor JSONL dump)

# 3. Offline build + sweep (one combo per run):
cd /tmp/termrock-tag-build
cargo build --offline --bins
cargo test --offline --no-run --test visual_baseline
Q07_COMBO='100x30/truecolor' Q07_DUMP=/tmp/q07-cursors.jsonl \
  ./target/debug/deps/visual_baseline-<hash> --ignored
```
