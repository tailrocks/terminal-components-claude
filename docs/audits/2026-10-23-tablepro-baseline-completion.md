# TablePro visual baseline completion: reviewed bless of the stale rows

Date: 2026-10-23. Branch: `parity/termrock-visual-baseline`. File:
`crates/tablepro-ui/tests/baselines/tablepro.txt`.

## Authorization

The independent review of
`docs/audits/2026-10-22-tablepro-digest-drift.md` returned **AUDIT
STANDS**: the reviewer verified both fixes (the perf repairs in product
source) and confirmed both dispositions. Disposition A authorizes a
reviewed bless of the remaining stale tablepro rows in one step, in the
precedent of `21ff14a951` ("fix(tablepro): re-bless the stale
quick-switcher frame digests"), which re-blessed the six quick-switcher
rows against the frozen oracle with the frame-cell evidence in its
commit body.

## Procedure

1. Enumerated every mismatched row by running the digest test with
   `BLESS=1` at the parity tip and diffing the regenerated baseline
   against the committed one (the harness aborts at the first mismatch,
   so a plain run shows only one row).
2. Determinism: the `BLESS=1` run was executed twice at the same tip;
   the two regenerated files are byte-identical (every got-digest is
   deterministic). The same double-run check was repeated at the final
   tip after rebasing.
3. Attribution: a commit walk from `3baa961531` (the last commit that
   touched the baseline) forward over every `crates/` commit to the tip,
   capturing the full digest set at each commit (`BLESS=1` run per
   commit). Each flipped row is attributed to the commit where its
   digest first moved; 0 rows ended the walk unexplained.
4. Only attributed rows were written to the baseline.

## Walk result (drift origin of the 2026-10-22 red set)

104 rows had drifted relative to the `3baa961531` baseline. First-move
attribution over the walk:

| Attributing commit | Subject | Rows flipped |
|---|---|---|
| `bd4130cbdd` | fix(tablepro): match narrow explain footers | 57 first moves |
| `4357e85ce4` | fix(tablepro): match 80-column query frames | 56 |
| `89e49b8c7c` | fix(tablepro): show wide explorer counts and the 160 editor seam | 64 |
| `1c17d8f262` | fix(theme): match baseline 256-color indexes | 25 |
| `72c280a6ad` | fix(theme): paint SQL diagnostics in baseline rose | 4 |
| `6cc8531bc0` | fix(tablepro): match 100-column query edges | 4 |
| `d1ac0c0dac` | fix(showcase): match the buttons audit chrome | 2 (transient; the value is superseded by `1c17d8f262` in the same walk) |

Every attributing commit is a recorded deliberate change: each body
cites the frozen-oracle stamp it implements, and each updates
`docs/architecture/termrock-migration-matrix.md`. `ecaa1bb96a`
(completion popup width) moved no tablepro digest. Steps after
`7e2597710a` (`d13154d886`, `de6b004f6c`, `ef23c3303c`) moved none.

Broken intermediate commits noted (same pattern as the audit's
`b12ced50e` note, not digest flips): `c3e2816db0` and `9efbf94bcd` do
not compile — `c3e2816db0` introduces duplicate `BLUE_INFO`,
`SYNTAX_STRING` and `SYNTAX_NUMBER` constants in
`crates/termrock-theme/src/builtin/junie.rs` (lines 25-27 and 47-49),
fixed by `72c280a6ad`. Their digest effect is therefore measured at the
next buildable commit, `72c280a6ad`.

## Mid-lane event

While this walk ran, `4fb24d8eae` ("test(tablepro): bless tablepro
visual baseline digests after theme and plate updates") landed the bless
of 96 of those 104 rows, and `2b9ab242d6` ("tablepro: drop set_surface
seeding, add safe-mode owner guard") then moved the eight
`tab-list-picker` rows. After rebasing on the tip, the residual red set
is exactly those eight rows, and this record blesses them.

## Rows blessed (before → after)

All eight attribute to `2b9ab242d6`: dropping `set_surface` seeding (it
no longer seeds twelve queries or forces `SafeFull`; the pickers build
their state through explicit `open()` builders) changes what the
`tab-list-picker` surface renders, and that commit landed after
`4fb24d8eae`'s bless without re-running it.

| Row | Before | After |
|---|---|---|
| `tab-list-picker 120 40 junie mono` | `641652a0aae2fd38` | `aa3feeb745cc0476` |
| `tab-list-picker 120 40 junie truecolor` | `af467dd0e23843e3` | `b7aea215516e85ad` |
| `tab-list-picker 120 40 paper mono` | `94911c1e5d5ae54a` | `3f906fb63249f82e` |
| `tab-list-picker 120 40 paper truecolor` | `fa600bb4e8dbe3d3` | `3c38fd7ca1fcb5ff` |
| `tab-list-picker 80 24 junie mono` | `5db82a252c5049c1` | `e3aa70d2534966c8` |
| `tab-list-picker 80 24 junie truecolor` | `3f7aa47005578084` | `95eab5a4441c4b1e` |
| `tab-list-picker 80 24 paper mono` | `ebecf1300ebbb6ef` | `60f30af9506d7080` |
| `tab-list-picker 80 24 paper truecolor` | `e3e7aff41217e98c` | `d7060a3f5e3389e8` |

Content evidence (frame captured from the failing run at the tip): the
`tab-list-picker 120 40 junie truecolor` frame shows the workbench with a
single `Query 1` tab and the `✓ safe` chip — the unseeded, explicitly
opened state — where the seeded frame carried the forced
twelve-query state. No row was left red: after this bless the digest
test passes in full, and a second `BLESS=1` run at the same HEAD
regenerates a byte-identical file.

Precedent: `21ff14a951` (six quick-switcher rows, frozen-oracle cell
evidence in the commit body). This record covers the remaining rows the
2026-10-22 audit left red.

## Unrelated red observed at this tip (not this lane's change)

`tablepro-ui::perf frame_tablepro_grid_500x14_120x40` fails at this tip
with `exceeded 100 allocs: 109`, on a clean checkout without this lane's
diff (the digest and docs files cannot affect allocation counts). This
is a re-regression of the 2026-10-22 audit's Finding 2 after
`8ff0a5f746` fixed it; it needs its own first-bad-commit hunt and fix in
product source. Recorded here so it is not silently dropped; the digest
rows this lane owns are green.
