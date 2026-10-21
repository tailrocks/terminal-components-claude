# Junie context menu danger-row profile

Date: 2026-10-21

Scope: Junie `ContextMenu` rows using `Family::MENU`, `Variant::DANGER`,
`Part::ROW` and its rendered label. Profile values use the existing
`Theme::define_variant_for_level` resolver. No application-specific flag,
widget painter, global quantizer change, palette rewrite, or public theme API
was added.

## Frozen source

Reference source: commit
`4169783ae883788186d161af09ba9c481bd84e5c`, tag object
`2718a6b8127a522a3d351d0b4bdb7542fc8da542`.

`src/widgets/menu.rs:686-708` builds a filled cursor row from focused, error,
disabled, hovered, and pressed state. `src/theme.rs:1410-1611` selects the
danger-highlight ladder, primary foreground, focused/fill modifiers, and
disabled surface fallback. The source ladder is:

| Capability | Rest | Hover | Press |
|---|---|---|---|
| TrueColor | `Rgb(122,42,42)` | `Rgb(140,48,48)` | `Rgb(90,31,31)` |
| ANSI256 | `Indexed(88)` | `Indexed(124)` | `Indexed(52)` |
| ANSI16 | `Red` | `Red` | `Red` |
| Mono | `DarkGray` | `DarkGray` | `DarkGray` |

The frozen text ink is primary for enabled danger cursor rows. Active enabled
rows retain focused weight; ANSI16 flat hover adds underline; Mono fill uses
reverse, flat hover adds underline, and pressed fill uses bold plus underline.
Mono disabled rows use dim ink and the normal popover surface. Where disabled
ink equals that surface, the frozen resolver falls back to muted ink.

## Change

`Theme::junie` adds capability profiles for the danger row's active, hover,
pressed, disabled, and active-disabled combinations. The profiles provide exact source
ladder colors and modifier precedence. More-specific hover/press/disabled
rules preserve their state over the active rest value. The disabled profile
returns to `CurrentSurface`, uses the frozen disabled-ink fallback, removes
active focus modifiers, and adds Mono dim. Inactive danger rows and normal menu
rows do not receive the active profile.

The profile uses existing provenance checks. Consumer token or recipe changes
detach the profile channel; global, overlay-scope, instance, and part-local
overrides still win. The test also verifies cache invalidation after replacing
the runtime theme.

An additional resolver check exposed that a disabled danger row could inherit
generic hover styling. The disabled rule now retains the frozen `DangerSoft`
ink and clears row fill and hover/press modifiers; the active-disabled rule
also removes those transient modifiers while restoring the frozen surface and
disabled ink. This keeps the disabled item inert at every capability level.

## Evidence

`crates/termrock-overlays/tests/context_menu_danger_profile.rs` drives the
public `ContextMenu` through a live `Harness`: Tab focus, keyboard Down, pointer
hover and press, and keyboard activation. It checks committed cells, selected
row state, and typed `ActionKey`; checks normal and inactive disabled rows;
checks disabled profile resolution and verifies disabled rows ignore hover and
press; exercises same-valued custom, different
custom, and Clear patches at global, scope, instance, and local precedence;
and checks theme replacement.

Before the profile, the real-widget regression failed on the frozen active
hover cell (`TrueColor` rest `Rgb(122,42,42)` versus expected hover
`Rgb(140,48,48)`). The disabled-hover resolver check also failed before the
disabled precedence patch because that row resolved to the generic blue
highlight. After both profile fixes, all six focused cases passed across all
four capabilities. No frozen artifacts or expected snapshots changed.

Checks on the implementation: `cargo nextest run -p termrock-theme
-p termrock-overlays --no-fail-fast` ran280 tests: 279 passed, the known
`capability_palettes::all_73_junie_slots_match_reference_capability_output`
failed, and no tests were skipped. `cargo clippy -p termrock-theme
-p termrock-overlays --all-targets --no-deps -- -D warnings` passed.
`cargo fmt --all -- --check` passed. The 73-slot failure remains intentionally
unblessed; this work did not update its expectations.

This record establishes only the cited Junie menu-row contract. It does not
clear the global 73-role inventory, prove application adoption, or establish
any application's complete visual or functional parity.
