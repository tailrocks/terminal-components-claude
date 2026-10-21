# Shell geometry ownership and contract

Reviewed source: `b521748360a2a056d95c79090707ded595f53a6a`, unchanged for the shell geometry files at the implementation base `7c94b00331413350f37365c7a3a34151dc952e38`. The independent source review is preserved at `docs/audits/evidence/2026-10-21-shell-geometry/shell-api-architecture-review-b5217483.json` (SHA-256 `d5a76a06b76dd6b9ee8ac3825fd65cf3ccc385158748b9b00c3432955e5fb1fb`). It was source-only: no build, runtime probe, or product-parity approval.

## Decision

`termrock-shell` owns `Chrome`, `Regions`, `AppShell`, and its slot builder. `termrock` keeps its current facade paths and compatibility methods by re-exporting and adapting to those exact shell types. This follows the dependency direction already present in `Cargo.toml`: the facade depends on `termrock-shell`, while the shell does not depend on the facade. There must be one implementation of the public shell state and layout policy.

`Chrome::regions` is the single resolver for the header, body, and footer. It reserves header and footer first, adds a gap only beside a nonempty strip, and applies horizontal inset to the body only. Drawing and child hit/focus/cursor registration use those resolved rectangles through the existing `Ui` area and composition-leaf contracts. No app drawing or second renderer is introduced.

Page leaves fill the body but report no mandatory height. Their measurement remains pure. Header and footer leaves receive constraints bounded by their configured strip heights; shell rendering clips their registration and drawing to the same resolved strip areas. Small and zero-sized roots remain valid.

## Findings and scope

The independent review found duplicate facade and shell `Chrome`, `Regions`, and `AppShell` implementations; their `regions` and composition slots disagreed about inset and gaps. It also found that `PageLeaf` reported the entire root maximum as its minimum, which could consume the footer during column allocation. These are implementation defects and are addressed by the shared owner and shared resolved geometry.

The same review identified raw-key `KeyRouter` integration, unconditional activation of the runtime testing feature, and missing live Showcase/application adoption. Those findings are outside this geometry increment and remain open in the migration matrix. This change does not establish app parity, cursor fidelity, the shell input contract, or the production dependency boundary.

## Required proof

The public shell and page-leaf path now has real runtime control-registration coverage. Nonzero root origins, body-only inset, an absent header with a conditional gap, narrow and zero-sized roots, clipping, matching hit/focus/cursor bounds, and footer persistence with a filling page body are covered. Registered control bounds are compared with the shared resolver.

The first implementation was independently reviewed at commit `6b73822c361e3a253709ad71fe6c885693fd1135`. Review status was changes required. The immutable packet is `docs/audits/evidence/2026-10-21-shell-geometry/shell-geometry-api-review-6b73822.json` (SHA-256 `9fdc0ce9fc826184d4f0907fd72386bdd15636bfbb4fbf81609da22608b981b2`). Its measurement probes showed that the minimum omitted reserved strips and conditional gaps, allowing a nested shell body to be clipped despite feasible sibling shrinkage. Those probes are retained in `crates/termrock-shell/tests/api_review_measure.rs`; implementation follow-up adds minimum accounting for reserved header/footer rows and only the gaps adjacent to present strips, then clamps to the supplied constraints. Omitted strips add no gap, and body inset uses the resolved width consumed by the same Chrome resolver.

The shell return-type adaptation keeps `AppShellSlots::slots()` returning `CompositionSlots`, now wrapping one canonical shell leaf; its existing `.spec(...)` and `ContentSlot::nested(...)` composition patterns remain source-compatible. `AppShell::slots(...)` must return `ShellSlots` because it owns the borrowed header/body/footer references and cannot safely return a self-referential `CompositionSlots` without changing the runtime to support owned leaves. For callers that need explicit composition interop, keep the `ShellSlots` value alive and write:

```rust,ignore
let shell = AppShell::new().slots(&header, &body, &footer);
let group: CompositionSlots<'_> = shell.composition();
let nested = ContentSlot::nested(group.spec(Composition::new().gap(1)), policy);
```

This is a deliberate source-compatibility exception for explicit `CompositionSlots` annotations or direct `ContentSlot::nested(AppShell::slots(...), ...)` calls. It avoids creating a second geometry engine. `crates/termrock-shell/tests/composition_interop.rs` compiles the supported adapter and `AppShellSlots` legacy call shape as an external integration consumer.

The reviewer probes were run before the fix and failed (two failures); after reserving strips/gaps in the shell minimum and retaining the compatibility adapter, `cargo nextest run -p termrock-shell --no-fail-fast` passed 51/51 with zero skips. `cargo nextest run -p termrock --features testing --test slots` passed 10/10. `cargo clippy --all-targets -p termrock-shell -p termrock` exited 0 with the same existing pedantic `too_many_arguments` warning at `crates/termrock-runtime/src/hit.rs:236`; `cargo fmt --all --check` and `git diff --check` passed. The initial new omitted-strip assertion exposed a further defect (min 3 instead of 1); the implementation was corrected to ignore child minimums for strips that resolve to zero, and the whole scoped suite is green. Independent follow-up review of the corrected exact commit is still required before integration. Product parity remains incomplete.
