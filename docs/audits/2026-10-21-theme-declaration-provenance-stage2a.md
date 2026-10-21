# Theme declaration provenance foundation, stage 2A

This increment gives typed theme edits channel-precise, private provenance while preserving current recipe resolution and public mutable recipe APIs. It does not install a fallback resolver and does not claim that raw mutable access is tracked.

## Implementation

`RecipeEdit` declarations are retained separately from populated builtin recipes. Records preserve family/variant/override/profile scope, explicit channel `Set` and `Clear`, modifier changes, state patches, glyph/size slots, and explicit default-variant choices. Empty part entries, empty state patches, and edits with no effective declaration are normalized away. Builtin Junie capability profiles stay in the existing profile store and are not misclassified as consumer edits. Builtin Mono rules and `ThemeBuilder::mono_rules` consumer rules retain distinct private origins; the public rules accessor and resolved behavior are unchanged.

Declaration metadata participates in `Recipes` clone and equality. Nonempty typed declarations advance the recipe revision used by the resolver cache. Empty edits do not change equality, revision, or Menu resolution. Raw `Recipes::get_mut` and `iter_mut` remain public and untracked. Their removal or replacement requires the next atomic stage to migrate callers and record a compatibility decision. No value-equality or `Role::Custom` inference is introduced.

## Evidence

Base revision before edits: `0abfee9e6882bfad5ee2549e761f89ebb2e5225c`.

At the base, `cargo nextest run -p termrock-theme all_73_junie_slots_match_reference_capability_output --no-capture` failed with the existing Ansi256 syntax-palette mismatch at `crates/termrock-theme/tests/capability_palettes.rs:92`. Nextest run ID: `5fc7db6e-5921-422e-af2f-8067fa80430c`; test binary SHA-256: `817ef504381e5dcad607dcfb1ec7840357eb83f1eddf8f7a0e027599e378a80e`.

After the change, `cargo nextest run -p termrock-theme --all-targets --no-fail-fast` ran 190 tests: 189 passed, zero skipped, and the same one test failed with the same Ansi256 syntax-palette mismatch. Latest final run ID on the merged WIP head: `1e3767c4-a564-4e81-afc7-b83ebb843def`. Earlier full run IDs: `7eff0f9e-268d-4138-a85d-bc7d1577500f` and `5968312b-989a-44a4-aa6d-24532960793e`; isolated failure reproduction run ID `ed6ad77d-2e87-47c0-ba25-e8839887a8e0`. Test binary SHA-256 on the prior 189-test build: `33cfbb822363d6ff200efc9b397ed727a9771d1e54b7098e958895cddeba72cc`. The focused declaration, normalization, alignment/state-patch retention, clone/equality, Mono-origin, empty-edit, and Menu-resolution probes passed.

`cargo fmt --all --check` passed. `cargo clippy -p termrock-theme --all-targets -- -D warnings` passed. No expected artifact or product frame changed. These results cover the Theme crate only and do not establish application parity, full workspace acceptance, or the requested future raw-mutation API migration.
