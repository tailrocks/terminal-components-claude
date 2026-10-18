# Findings from writing the §24 guides

Produced while writing `docs/guides/**`. Every item was **measured** — the agent built a throwaway crate outside the repo that depends on `crates/tui` under the final `junie_tui` name and compiled all 38 snippets — not inferred from reading. Each needs a decision or a fix.

## F1 — undeclared custom families get no mono fallbacks (defect)

`Recipes::apply_mono_fallbacks` (`crates/tui/src/theme/downgrade.rs:392`) iterates `self.iter_mut()`, i.e. `by_family` only, and never reaches `Recipes::neutral`. §11.2 says an undeclared `Family::custom("x")` renders through the neutral recipe rather than resolving to an empty style, and §16.2 case 9 is mandatory for every registered component — but at `ColorLevel::Mono` an undeclared family receives **zero of the 18 rules**.

Measured: `Theme::junie().downgrade(Mono).resolve(Family::custom("segmented"), DEFAULT, LABEL, FOCUSED, Canvas)` carries no `BOLD`; the same call after `define_family` does.

Consequence: goal §29's "component state remains readable without relying only on colour" fails for any downstream component that does not declare a family — which is the default path an author takes.

## F2 — the architecture's own Scenario G reference fails two conformance cases (defect)

`crates/tui/examples/12_author_component.rs` is the document's mechanical proof of goal §23 Scenario G. Registered against `conformance_suite!` it fails two of the twenty cases, measured:

- `local_override_does_not_mutate_the_theme` — "the instance patch had no effect": `Segmented` has no `.patch`/`.patch_part` builder, although §13 and the §13.2 rustdoc template both require an `## Overrides` line.
- `mono_states_are_distinguishable` — fails at the first pair: `Segmented` has no `.state_override`, so `Fixture::force` is invisible to it and every state renders identically; F1 means the theme cannot rescue it either.

A minimal corrected version (adding `.patch_part`, `.state_override`, self-painted `FocusBar`/`Chosen`/`PressLeft` affordances, and a `CONTAINER` fill so `PARTS.first()` is a part it actually paints) passes all 21 generated tests. So the fix is known and small; the point is that the reference example does not currently demonstrate what it claims.

## F3 — `PARTS` ordering is load-bearing and undocumented

`Fixture.patch` targets `C::PARTS.first()`, so a component whose first declared part is one it never paints fails case 10 with a misleading message. Not stated in §16.2.

## F4 — a rustdoc claim that does not match the theme

`Button`'s rustdoc reads as a claim that `Family::BUTTON.default_variant` is `SECONDARY` under both built-ins. Only Paper's is; Junie's is `Variant::DEFAULT`. The guide documents the measured behaviour.

## F5 — `define_family` with an empty edit silently loses the neutral styling

`get_mut` inserts a `Recipe::default()` with an empty `PartMap`, so `define_family(F, |_| {})` **replaces** the neutral recipe rather than merging into it. §11.2's "`define_family` replaces it" is literally accurate but reads as benign; in practice it is a footgun. Flagged in `theming.md` and `overrides.md`.

## F6 — `run()` performs no capability detection

`Theme::junie()` is constructed at `ColorLevel::TrueColor` and `run` passes it through unchanged. `ColorLevel::detect()` exists and nothing calls it, so a 16-colour or `NO_COLOR` terminal gets truecolor output unless the caller intervenes. Documented in the guides as an explicit caller obligation; whether that is the intended contract needs a decision.

## F7 — facade gaps

`theme::{Capability, Recipe, Recipes, RecipeEdit, PartRecipe, PartEdit, BorderSet, GlyphSet, SpaceTokens, SizeTokens, MotionTokens, MeterThresholds, MONO_RULES_PER_FAMILY, downgrade_color}` are not in the root facade's re-export list and are reachable only as `junie_tui::theme::*`; `StateRule` is in `author::` but not at the root. The guides use the `theme::` path rather than implying a root re-export. Appendix B.3/B.4 should either list them or state that `theme::` is the intended path.

## F8 — two binding-lookup idioms

`authoring.md` uses `Binding::lookup(BINDINGS, &k)` (chord `matches`); example 12 uses `.find(|b| b.chord == k.chord())` (exact equality). Both compile. If they are meant to differ semantically that is a separate question; if not, one should be the documented idiom.
