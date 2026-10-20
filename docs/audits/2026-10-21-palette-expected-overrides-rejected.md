# Palette expected overrides rejected

## Decision

Removed the six ANSI 256 expected-value overrides added by `93b306503581d0c90665ca884af78000e3a29258` from `crates/termrock-theme/tests/capability_palettes.rs`. Kept the three pre-existing overrides (`on_accent`, `border_strong`, and `disabled_fg`). No production theme, palette, checksum, test case, or frozen artifact changed.

The removed values match the unchanged candidate palette, so they turn a failing assertion green without deriving the values from the frozen source. `highlight_danger_bg = 238` directly contradicts the qualified frozen error cursor-row fill value `88`. The other five values lack a universal frozen contract; several are contextual or extension tokens. Neither the removed values nor the former generic raw-color projection is accepted as a universal 73-token oracle.

The independently reviewed inventory remains 48 source-qualified entries and 25 contextual or ambiguous entries. Retain all 73 tokens in the inventory. Replace the invalid universal projection only after a documented, independently reviewed reference decision; test each qualified role through its real component and state.

## Evidence

Independent read-only review packet: `/Users/donbeave/Projects/termrock-cli-evidence-20261021/theme-six-overrides-review-93b306/review.json`, SHA-256 `34c22015f6dca2d9ffbf1cb862eaa8fab9b22a849e0bacd441ad3477c6cba158`.

The reviewer ran the predecessor assertion against the exact `93b306503` product and observed failure, then ran the six-override assertion against that same product and observed success. A separate source-qualified probe observed actual `Indexed(238)` where the frozen error fill requires `Indexed(88)`. The native Theme suite separately reported 139 passed and one unrelated modal-backdrop failure; that result does not establish palette parity.

References remain fixed: baseline source `4169783ae883788186d161af09ba9c481bd84e5c`, tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`, and tuiscotti `a1ea27f757d75d07b05499276cd932e9f8e067ad`.

## Worktree recovery record

During branch setup, a scoped stash command was used before the shared-worktree stash risk was recognized. `git stash pop` reported dropped object `8a8b5b5ca22235728602a0aa77d8eac0f81d2732`. Its two parents are `63f966679b0d564f3f8f361eb06d467992027644` and `a649f8c65f23fefa8aeb803412007249bd061d27`. It differs by one path from each parent. An atomic local safety ref, `refs/termrock-safety/stash-8a8b5b5c`, now protects the object from garbage collection. The shared stash reflog does not establish whether this object came from my command or another worktree actor. I did not inspect or publish its contents. The unrelated one-line `crates/jackin-preview-app/src/app.rs` worktree edit remains untouched and excluded from this change.

## Validation status

This increment removes the unsupported overrides, restores the earlier assertion, and leaves the known semantic mismatch visible. The resulting red assertion is an unresolved product defect, not a passing parity check. On current PR38 source, nextest reported the `Ansi256` mismatch: candidate values `accent_pressed=238`, `accent_tint=233`, `focus_ring=238`, `highlight_danger_bg=238`, `danger_tint=233`, and `syntax.match_bg=233`; the retained generic projection expects `71`, `22`, `71`, `95`, `52`, and `22`. The log SHA-256 is `c46e0da1374258f3aaa490735f622daa0694dbc4788a62b6a2975d628cdb71d4`. No palette implementation or frozen reference data was changed.
