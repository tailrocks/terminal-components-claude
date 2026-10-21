# MenuBar title style precedence

## Finding

Commit `84d6477d7` began resolving title-scope styles through
`MENU_BAR_TITLE_VARIANT`. At the merge parent `7f819a779566559fd721949d1fd91fad29768374`,
`menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence`
failed in Title scope: the overlay's authored background (RGB 7, 8, 9) was
replaced by the built-in title fallback (RGB 28, 28, 31). The specialized
variant had accidentally become the resolver's effective variant, bypassing
the family-authored default variant and overlays targeting it.

## Decision

Keep `MENU_BAR_TITLE_VARIANT` as the MenuBar component's low-priority style
fallback, then resolve through `Family::MENU` and `Variant::DEFAULT` with the
shared `style_defaults` contract. This lets the family recipe choose its
public default variant and apply its overlays before the instance-local patch.
The regression test also authors an underline in `MENU_BAR_TITLE_VARIANT` and
checks that the customization survives in Title scope. Bar scope continues to
use the normal family resolver. No app workaround, resolver API change, or
expected-output change was made.

## Verification

Before the edit, the focused test failed only for Title-scope overlay background.
After the edit, `cargo nextest run --locked -p termrock-overlays --lib
--no-fail-fast` passed all 84 tests. Strict scoped Clippy, workspace format
check, and `git diff --check` passed.
