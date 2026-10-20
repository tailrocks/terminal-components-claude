# Button narrow Unicode label fit (2026-10-21)

This is a Button-only rendering correction. It does not approve application parity or the complete Button/theme contract.

## Frozen source contract

The frozen `visual-baseline` source commit is `4169783ae883788186d161af09ba9c481bd84e5c` (tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`). In `src/widgets/button.rs`, the Button renderer calls `crate::ui::text::fit(&text, (w as usize).saturating_sub(2))`. `src/ui/text.rs::fit` calls the shared `truncate` function and pads the fitted content to the requested cell width. `truncate` iterates Unicode grapheme clusters, budgets terminal-cell width for an ellipsis, and never writes a partial wide grapheme. Therefore a 5-cell Button with label `世界` has a 3-cell label slot and paints `世…`.

## Change and regressions

The refactored Button reserved the same label rectangle but passed the full label to `Ui::paint_str`, which clips without adding the baseline ellipsis. The real `termrock-test-support::Harness` regression at a 5-cell width failed before the fix: expected an ellipsis at cell x=3, observed a blank (`cargo nextest` run `d68533dc-24a2-40cc-9aa3-3404703e25e3`). The Button now uses the existing shared `termrock_text::truncate` operation for the default label renderer and fills the label rectangle with its resolved style before painting, preserving the baseline's padded label area. Explicit label slots still receive their original rectangle and own their painting.

Public Harness tests cover the five-cell `世界` label in TrueColor, ANSI256, ANSI16, and Mono; checked marker placement and truncated label; combining `e + U+0301`; the `👩‍💻` grapheme; empty labels; and zero-to-three-cell regions in every profile. The focused wide-label test passed (`518a2583-711e-471a-92f6-42746bfda359`), and the expanded Unicode/state test passed (`2a58713a-136e-4584-b6f5-de5d97b081e3`). The controls all-targets suite passed 123/123 (`cb727d09-cfd4-4831-973f-fe61e2a7df4c`); strict controls Clippy, targeted Rust formatting, and `git diff --check` passed.

## Remaining gates

This increment does not address the known Theme ANSI256 palette failure: `all_73_junie_slots_match_reference_capability_output` reports actual `Indexed(78)` versus expected `Indexed(77)` (unfiltered Theme run `9b77611d-eb33-4217-af7f-d31f2099e165` on WIP `57ca512b3796b04e518a73b5fdfe7391f312c6f3`; same mismatch was reproduced at the frozen worktree's pre-change base). The parent also reports an independent runtime natural-width measurement failure after its later merge; it is outside the files and contract addressed here. Neither is suppressed or waived by this Button test suite.

The final focused tests additionally check every visible cell in the 5-cell `世界` case (focus gutter, wide-grapheme continuation cell, ellipsis, trailing padding), checked marker placement under all four profiles, label-slot ownership, and zero-to-three-cell rendering. The slot test confirms custom label painting is not passed through the default truncator. Final controls all-targets passed 124/124 (run `c83dbd6f-397c-4d95-afd9-330550b892d0`); strict Clippy and targeted rustfmt passed on the same source. The initial RED remains run `d68533dc-24a2-40cc-9aa3-3404703e25e3`; the targeted GREEN run before expanded edge assertions is `518a2583-711e-471a-92f6-42746bfda359`.
