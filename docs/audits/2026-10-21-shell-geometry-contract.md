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

The second independent API review found that the prior return-type adapter still failed two unchanged legacy source probes: direct `AppShell::slots` explicit type assignment and a temporary `AppShellSlots` builder could not compile. The packet is preserved at `docs/audits/evidence/2026-10-21-shell-geometry/shell-geometry-api-review-e6a4829.json` (SHA-256 `4b29bedbaf89709a3052d44833c328c6b872981b786fcf5f821d5d47400f40fd`). Geometry findings were green (53/53); compatibility clearance remained withheld.

Compatibility is restored with one narrow extension to the existing runtime composition tree. `ContentSlot::owned` boxes a `CompositionLeaf` and routes it through the same measure, width remeasurement, resolved bounds, clipping, and draw path as borrowed leaves. The adapter allocates one box per owned leaf and drops it with its composition group. `AppShell::slots(...)` and `AppShellSlots::slots()` now both return `CompositionSlots<'a>` backed by an owned copy of the canonical `ShellSlots` view; the view has no component state. `ShellSlots::composition()` remains available for callers that hold the leaf, and `AppShell::resolved_slots(...)` is the additive API for consumers that explicitly need the leaf form.

The exact unchanged two-case compatibility consumer from the independent review is retained in `crates/termrock-shell/tests/api_legacy_compat.rs`; it checks explicit return type, `.spec(...)`, and temporary builder lifetime. `crates/termrock-shell/tests/composition_interop.rs` exercises live drawing and measurement, padding `.spec(...)`, direct `ContentSlot::nested(AppShell::slots(...), ...)`, borrowed domain data, owned/borrowed leaf bounds, cursor, focus, hit/hover, and exact drop count. It drives a runtime buffer; it is not a no-op type-only assertion.

```text
let group: CompositionSlots<'_> = AppShell::new().slots(&header, &body, &footer);
let nested = ContentSlot::nested(group.spec(Composition::new().gap(1)), policy);
```

The legacy direct nested call `ContentSlot::nested(AppShell::new().slots(...), policy)` and a temporary builder expression also remain valid. This uses the existing composition engine rather than duplicating it. `crates/termrock-shell/tests/composition_interop.rs` exercises the consumer API through the runtime.

After the owned-leaf adapter landed, `cargo nextest run -p termrock-shell --all-targets --no-fail-fast` passed 55/55 with zero skips, including both unchanged legacy API probes and all shell geometry tests. `cargo nextest run -p termrock-runtime --all-targets --no-fail-fast` passed 382 with 3 skipped; this broader runtime result is recorded as such, not a full pass. The focused `cargo nextest run -p termrock-runtime --test composition_consumer --no-fail-fast` passed 7/7, and `cargo nextest run -p termrock --features testing --test slots --no-fail-fast` passed 10/10. `cargo clippy --all-targets -p termrock-runtime -p termrock-shell -p termrock` exited 0 with the existing pedantic `too_many_arguments` warning at `crates/termrock-runtime/src/hit.rs:236`; formatting and `git diff --check` passed. The unchanged legacy compatibility tests are green locally; independent API/runtime review of the corrected exact follow-up is still required before integration. Product parity remains incomplete.
