# Picker authored-background contrast integration

Integration base: PR #38 `8e502d7fb9e40420b844964ec933bac497ebb2ae` (root FormDialog integration, fetched before this merge). Incoming WIP tip: `c932adb5ee29ef8ed9f90cc022c39cb9c0472e5d`. The incoming branch has three commits unique from merge base `bb3db3542933e1927376967c280fa920c92f8fdb`; it is integrated by a normal no-ff merge to retain branch history. The sole source conflict was the `termrock-navigation/src/item.rs` import hunk. Resolution takes the union of the current middle-parts/Unicode imports and the WIP theme/style imports plus its `picker_ink` helper; no behavior or API was changed by the resolution.

The correction uses resolved explicit-background provenance to scope generated Picker ICON/META contrast repair. It leaves inherited default colors unchanged and preserves explicitly authored foregrounds, including same-valued foregrounds. It does not approve the previous `cd1af58` change or update expected output.

Independent review decision: **CLEAR bounded authored-background contrast correction; inherited required gate gaps remain incomplete**, no new blocking findings. Exact independent packet and supporting outputs are retained under [independent-review](evidence/2026-10-21-picker-contrast-integration/independent-review/); its original `review.json` SHA-256 is `aad3119d5626be0654611d7469d4dae75d182ec1f93fe90b8bb68b704a13239e`. Review found default actual-profile Mono/Fill output identical across initial/down/resize checkpoints (8 cases, 24 checkpoints; dump SHA-256 `c91f80953226446a1748ce7a2790b6b6ac3d03f0cbfb24c9af8f8c30c953c6e3`). The owned-background probes passed; inherited default Mono/Fill white-on-DarkGray ICON/META contrast remains 4.00410696 and unchanged. The historical `constructor_defaults_match_the_pre_policy_full_cell_fixture` failure remains unwaived, as do three skipped native tests and other inherited parity gaps.

The independent review reports native navigation/runtime all-targets 646 pass, 1 fail, 3 skipped; strict scoped Clippy and format passed on the reviewed WIP. Current integration checks are recorded below after they run; reviewed-WIP results are not substituted for checks on this merged PR #38 head.

## Reference inventory and protected expectations

[Reference inventory delta](evidence/2026-10-21-picker-contrast-integration/reference-inventory-delta.json) structurally compares frozen commit `4169783ae883788186d161af09ba9c481bd84e5c` with the PR #38 application source inventory. It records source paths, ordered navigation entries, and missing/placeholder routes. Its candidate route-source inputs were checked unchanged from the earlier extraction commit `819ef8ef949dfa536d7fba9ef73a5d1f3e476b0b` through this integration base. It is not an executed screen/flow comparison, and no application parity conclusion is drawn.

[Held expectation manifest](evidence/2026-10-21-picker-contrast-integration/held-expectations.json) records all seven explicitly protected blobs: Showcase, Holla, Jackin and TablePro baselines; both Holla heading fixtures; and the Showcase page-heading assertion. Each current base blob equals its `52e1572caaf7d9effd0265bbfdf6edcd0e57533b` blob. The list resolves older matrix wording that referred to six or four-file subsets. These are preservation checks, not approval of their expected contents.

## Current integration gates

On the normal-merge result plus the import-union resolution, using exclusive `CARGO_TARGET_DIR=/private/tmp/pr38-picker-contrast-target-20261021`:

- `cargo nextest run -p termrock-navigation --lib`: 112 passed, zero skipped.
- `cargo nextest run -p termrock-navigation --test item_row_columns --no-fail-fast`: 14 passed, one failed, zero skipped. The sole failure is the unchanged `constructor_defaults_match_the_pre_policy_full_cell_fixture` at line 483; its expected value was not edited or waived.
- `cargo nextest run -p termrock-runtime --lib`: 261 passed, zero skipped.
- `cargo nextest run -p termrock --lib`: 11 passed, zero skipped.
- The public different-data consumer example `cargo check -p termrock --example picker_row_defaults` and `cargo run -p termrock --example picker_row_defaults` both passed.
- `cargo clippy -p termrock-runtime -p termrock-navigation -p termrock --all-targets -- -D warnings`, `cargo fmt --all --check`, and `git diff --check` passed.

Full logs are in [gates](evidence/2026-10-21-picker-contrast-integration/gates/). No application flows, visual inventories, or paired baseline comparisons were run here. Known historical fixture failures and other product parity gaps remain open; no tests or expectations were weakened to obtain a green result.
