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

## Superseded finding — b920 HOLD

The preceding decision describes commit `b9207254a133693687c7d4e97eea3d9d70fca9ea`
and is superseded. Independent review of that commit found that the manual
`StyleDefaults` seed did not preserve authored provenance: `define_variant`'
special-variant foreground was overwritten by the normal family recipe, and
`override_variant` selectors for the special variant were omitted. This is a
real shared-resolver defect, not an app or MenuBar-only issue. Evidence is in
`/private/tmp/menubar-b9207254-independent-evidence-20261021/architecture.json`
(SHA-256 `643d0077feaa8fa5d3cdd17ac8fcc42fcb4d685fc9a5585119fd3a46ec5c4f78`)
and `review.json`.

The corrective implementation captures built-in recipes separately from
author-authored family/variant recipes and selectors at the public API write
point. The additive generic `bind_fallback_variant` / `Ui::style_with_fallback_variant`
resolution order is built-in family and resolved default-variant recipes,
built-in component fallback and capability defaults, authored fallback variant
declarations/selectors/overlays, authored family and resolved default variant
declarations/selectors/overlays, then the local patch. The
ordinary resolver and MenuBar Bar scope continue through their existing path.
The fallback channels preserve `Slot::Clear`, modifier removals, and authored
values even when they equal a built-in value; no color equality inference or
post-render correction is used.

The unchanged independent consumer was run against b920 first: 2/4 passed and
the two special foreground/override probes failed as expected. The same four
unchanged probes run against the corrective source passed 4/4. Native theme and
overlay library coverage passed 180/180, including the new clear/removal
resolver regression and MenuBar behavior across supported color profiles.
Strict scoped Clippy and format checks passed. Full application-frame comparison
was not run in this scoped task; the result does not certify application-level
visual parity.

## Superseded finding — e18 HOLD

Independent review of `e18b33bf9c7c4e6a693aaf51a505ca6c66a238ae` found two
incomplete paths: the new fallback resolver omitted the established Mono-rule
and exact-profile restoration phase, and it read the built-in snapshot for
families that callers had mutated through public `Recipes::get_mut` or
`iter_mut`. The evidence and authoritative corrective plan are at
`/private/tmp/menubar-e18b33bf-independent-evidence-20261021/review.json` and
`correction-plan.json`.

The follow-up records raw mutable access as private per-family provenance,
retaining the public signatures and public `Recipe` fields. Such access is
conservatively a whole-family author declaration; fallback resolution reads
that family's live recipe. Typed Theme declarations use an internal tracked
accessor, and the built-in initialization boundary clears provenance markers.
Fallback resolution now also applies the same Mono rules and exact-profile
restoration used by ordinary resolution. Focused regressions cover live raw
family/default/special selectors, clone divergence, typed declarations before
and after raw access, iterator access, no-op other-part access, explicit clear,
ordinary-over-fallback precedence, and Mono behavior. The unchanged external
consumer's four probes pass. Theme, runtime and overlay library tests pass
446/446; strict scoped Clippy, formatting and diff checks pass. The result is
still a WIP pending independent review of raw ordering and broader application
comparison; it is not an acceptance claim.
