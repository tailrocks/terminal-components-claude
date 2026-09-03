# Visual changes ledger

**What this is.** The ledger `COMPONENT_ARCHITECTURE.md` §20.10 requires and `xtask bless-guard` (§16.3) reads. No baseline file (`crates/tui/tests/baselines/components.txt`, `apps/*/tests/baselines/*.txt`, `perf_baseline.txt` hit counts) may be regenerated without an entry here that names a §20.10 item **and** a capture path under `shots/`. Every entry classifies a difference as *intended* (matches the §20.10 item), *fix* (a demonstrated defect in the old output) or *regression* (must be fixed, never blessed).

**Order, fixed (review A14, §21 item 30): change → capture → classify → bless.**

1. **Change** — land the code change on the working tree.
2. **Capture** — `tools/capture.sh` / `xtask capture-matrix` writes the before/after captures into `shots/`; digest tests go red.
3. **Classify** — add or extend the entry under the matching §20.10 item below: capture paths, the affected tests/baseline lines, and the classification with its reason.
4. **Bless** — `BLESS=1 cargo test --workspace --test render --test visual` (or `PERF_BLESS=1` for hit counts). `xtask bless-guard` runs in CI on the committed tree and fails the commit if step 3 is missing.

A capture cannot exist before the change, so `bless-guard` never runs locally against an unchanged tree. No baseline is regenerated because a test failed; the classification comes first.

**Entry format** (one per affected surface):

```
- surface:   <app>/<page or component>/<state> @ <w>x<h> / <theme> / <color level>
- captures:  shots/<before>.png  →  shots/<after>.png
- tests:     <baseline file>:<line or name>, <test names>
- class:     intended | fix | regression
- reason:    <one sentence tying it to the §20.10 item>
```

---

## Item 1 — Mono legibility fallbacks (§11.4, §21 item 25)

captures / classification: `(pending — filled when the change lands)`

## Item 2 — Layer compositing order (§5 R7, §3.3 step 12)

captures / classification: `(pending — filled when the change lands)`

## Item 3 — `RadioGroup` separates cursor from value

captures / classification: `(pending — filled when the change lands)`

## Item 4 — `Picker` secondary action gains a mouse equivalent

captures / classification: `(pending — filled when the change lands)`

## Item 5 — `Dialog`'s `y`/`n` quick answers become an opt-in binding set

captures / classification: `(pending — filled when the change lands)`

## Item 6 — F10 / menu-bar drift fixes

captures / classification: `(pending — filled when the change lands)`

## Item 7 — Container / geometry defect fixes (a)–(j)

Items (b), (e) and (f) change pixels in the current baseline and are called out individually.

captures / classification: `(pending — filled when the change lands)`

## Item 8 — The backdrop excludes the footer row uniformly

captures / classification: `(pending — filled when the change lands)`

## Item 9 — `StatusBar` and `segments` merge

captures / classification: `(pending — filled when the change lands)`

## Item 10 — Hints are derived from component bindings

The diff of the old hand-written hint tables against the derived output is attached here, each drifted entry classified as fix or regression.

captures / classification: `(pending — filled when the change lands)`

## Item 11 — Surface inheritance replaces colour-equality `lift`

Any cell that changes under `junie` is enumerated with the token pair that collided.

captures / classification: `(pending — filled when the change lands)`

## Item 12 — The showcase visual baseline covers the sidebar and gains three axes

captures / classification: `(pending — filled when the change lands)`

## Item 13 — `Tabs`' strip window follows the logical first tab

captures / classification: `(pending — filled when the change lands)`

## Item 14 — New cell-exact baselines for TablePro and jackin

The first generation is produced on the pre-refactor tree (Appendix A, WP‑0 — still owed, not in `07cb2c9`), then regenerated once at the end of Slice 8.

captures / classification: `(pending — filled when the change lands)`

## Item 15 — Focus-ring composition changes in migrated screens (§21 item 33)

Per affected test: old reachable count, new reachable count, the `Harness::ring().reachable()` listing, and the reason — written **before** the expected value in the test is edited. Also the home for `frame_showcase_lists_120x40`'s classified hit-count growth (§16.6, P8).

captures / classification: `(pending — filled when the change lands)`
