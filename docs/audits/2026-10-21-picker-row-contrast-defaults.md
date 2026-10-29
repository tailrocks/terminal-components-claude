# Picker row contrast defaults

Base: `f46caa2ce68966d319ffbb4cf03428e8234368f3`.

The first implementation in `cd1af58378631bebadc64de7459b01060babb52c` is **superseded / HOLD**. Independent actual-widget decoded-cell comparison found nine changed cells in default Mono Fill (initial, down, resize): ICON, detail, and tag changed from White/DarkGray to Black/DarkGray. Source review found the cause: it inferred owner intent from resolved contrast and repaired defaults even when no owner-authored BG existed. The packet is `/Users/donbeave/Projects/termrock-cli-evidence-20261021/branch-consolidation/picker-contrast-cd1af58-independent-review-20261021/review.json` (SHA-256 `b224e58cdb4db6bfca40587d4ac9c21a3424a759add2d3d705ab3981d62a6ab3`); decoded source comparison is in the same directory. The earlier follow-up packet is `/private/tmp/picker-row-review-f46c-20261021-followup.json` (SHA-256 `601139db431c0d10651b413ecf42a45cfb89477ca2015f708fdeb5c9252bb2ff`).

Corrective policy: only repair generated picker ICON/META ink when the resolver reports an explicit authored BG (`ResolvedStyle.explicit.bg != Slot::Inherit`) and no authored FG. This uses resolved provenance, never equality guessing. The correction stays below owner FG, including same-valued FG. Built-in family/profile defaults remain unchanged. For an authored low-contrast BG without FG, generated ink now meets 4.5:1; an authored muted FG remains authoritative even when it is below 4.5:1. That explicit author choice is tested separately and recorded as an intentional override.

The real `Harness::with_color` + public `FilterList` regression checks default reference ink, actual focused non-searchable result-list owner, Bar and Fill, all four profiles, custom white BG with no FG, same-valued explicit black FG on white, and modifier preservation. The repair-disabled probe failed at 2.10:1 for the white-backed tag; the unscoped-repair probe failed because Mono Fill changed its reference white ink to black. The corrected test passed after restoring provenance scoping. `tests/item_row_columns.rs` also checks custom theme BG fallback and an explicit muted-FG positive control. The change from muted default ink to a contrast-safe fallback applies only to authored low-contrast BG without authored FG; explicit muted FG remains as authored. Historical fixture evidence remains unchanged and unwaived.

Corrective increment gates:

```text
cargo fmt --all --check                                             PASS
cargo nextest run -p termrock-navigation --lib                      PASS 112/112
cargo nextest run -p termrock-navigation --test item_row_columns \
  --no-fail-fast                                                     14/15; known historical fixture fails
cargo nextest run -p termrock-runtime --lib                         PASS 261/261
cargo clippy -p termrock-runtime -p termrock-navigation \
  --all-targets -- -D warnings                                       PASS
git diff --check                                                    PASS
```

The lone `item_row_columns::constructor_defaults_match_the_pre_policy_full_cell_fixture` failure is the already-recorded, unwaived historical fixture mismatch at line 483; it predates this corrective change and was not edited. All other tests in that binary pass, including the updated custom-background contract test.

The original cd1 test evidence is superseded because it directly wrote `theme.capability.color` and did not establish a focused result-list owner. Corrected current gate receipts and exact commit identity are recorded with the corrective WIP commit. This scoped work does not certify picker interaction parity, frozen baselines, application parity, or PR #38 acceptance.
