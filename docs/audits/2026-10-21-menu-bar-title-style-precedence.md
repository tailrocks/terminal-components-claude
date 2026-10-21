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

## API contract limit — raw mutable recipe provenance

### Evidence and current call sites

At `crates/termrock-theme/src/recipe.rs:248-260`, `Recipe` exposes its
`default_variant`, family `parts`, and `variants`; each `PartRecipe` also
exposes style channels and state rules. `Recipes::get_mut` returns `&mut Recipe`
(`recipe.rs:509-513`) and `iter_mut` yields `&mut Recipe` for each family
(`recipe.rs:541-548`). `Theme::recipes` is public (`crates/termrock-theme/src/lib.rs:121`).
Those signatures expose no field-write event or selection of family, default,
variant, part, or channel. The tracked Theme builder methods at
`lib.rs:311-392` already record declaration provenance for `override_family`,
`override_variant`, `define_variant`, and `define_family`.

Current in-repository raw production use is limited to built-in construction:
`builtin/junie.rs:178-195` builds menu/list recipes and `builtin/paper.rs:68-74`
sets the built-in button default before `mark_builtin`. The broad `iter_mut` in
`builtin/mod.rs:890-895` is test-only. Other raw callers are tests/fixtures,
including `termrock-runtime/src/rowui.rs:1499-1502` and the public consumer
integration test `termrock-runtime/tests/rowui_glyph_contract.rs:11-14`.
Those consumer sites need a tracked public authoring equivalent before raw
mutation can be retired.

Independent review of 65b (`/private/tmp/menubar-65b2d0e5-independent-evidence-20261021/review.json`)
found that eager family provenance changes a no-op `get_mut`, a no-op `iter_mut`,
and an unrelated META edit: untouched built-in Title foreground moves above an
authored special-variant foreground. It also found that `Recipes::PartialEq`
omits the output-affecting raw marker, and a typed special foreground added
after raw access can lose to the raw ordinary-family tier. The same review
qualified the older raw-base background assertion as invalid; this is not a
product defect. Keep that qualification distinct from its five real failures.
The review artifact SHA-256 is
`5f7f65dc8fb638a8b3b0e3a569a93bf5eaa03b01bde77c4872944d87d81161a1`.

The unchanged standalone consumer at
`/private/tmp/menubar-fallback-consumer-20261021/tests/precedence.rs` uses the
tracked `define_variant`/override API and therefore does not exercise or depend
on the raw mutable accessors. It compiled and passed all four probes against
65b. A third-party consumer that follows the in-repository RowUi fixture and
mutates `Theme::recipes` directly would need a migration under the major API
boundary option; the staged option can first deprecate those calls while
shipping an equivalent public tracked author API.

### Why the raw contract cannot be inferred

For public `&mut Recipe` access, these operations have the same observable
access trace and final `Recipe`: (1) obtain the mutable reference and do
nothing; (2) assign a field its current value; and (3) edit a different field
while leaving the field under consideration unchanged. The resolver receives
only the final object and family-level access marker. Treating the family as
authored promotes untouched built-in fields and breaks no-op/unrelated-edit
invariants. Ignoring untouched values requires discovering which fields were
written; equality against a snapshot is explicitly disallowed and still cannot
distinguish a no-op from a same-value assignment. A plain `&mut` plus public
fields cannot provide both field-level authored intent and no-op stability.

### Available decisions and impact

1. **Tracked authoring API, staged migration (recommended).** Add/complete
   public declaration APIs that specify family versus variant, default
   selection, part, and channel/state intent; extend `RecipeEdit` for the full
   customization surface (including explicit channel clear/removal and
   low-level state-rule management needed by component authors). Migrate the
   in-repository runtime consumer test and any production caller, then
   deprecate raw mutable access. Keep ordinary
   legacy resolution behavior during a documented transition; fallback-aware
   components accept provenance-tracked declarations only. This preserves
   source compatibility initially, but users who mutate special fallback
   recipes directly must migrate before relying on fallback resolution.
2. **Major-version API boundary.** Make `Theme::recipes` read-only/private and
   remove public `get_mut`/`iter_mut` (and/or make recipe fields private),
   exposing only provenance-tracked authoring methods. This has the clearest
   semantics and lets all resolvers enforce the same contract, but breaks
   source compatibility for every external caller that reads through
   `Theme::recipes` or mutates a recipe, including the checked-in RowUi fixture.
3. **Legacy coarse/equality semantics.** Treat every mutable family access as
   an authored whole-family declaration, or compare against a captured
   baseline to infer changed values. Whole-family semantics are rejected by the
   independent no-op/other-part evidence. Equality-delta semantics are rejected
   by the project constraint and cannot preserve same-value write intent. Neither
   is a valid acceptance path without an explicit change to the contract.

No implementation can satisfy the no-op, unrelated-write, same-value-intent,
public-`&mut`, and no-equality constraints simultaneously. This audit records the
API boundary decision required before another code change; it does not silently
weaken those requirements or claim fallback parity. The 65b implementation is
superseded and remains unaccepted pending that decision.
