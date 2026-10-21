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

Focused validation on the implementation worktree: `cargo nextest run -p termrock-shell` passed 47/47 with zero skips; `cargo nextest run -p termrock --features testing --test slots` passed 10/10 with zero skips; `cargo clippy --all-targets -p termrock-shell -p termrock` passed; `cargo fmt --all --check` and `git diff --check` passed. Clippy emitted the existing `termrock-runtime/src/hit.rs:236` pedantic `too_many_arguments` warning. A stricter ad hoc `-D warnings` run fails on that unrelated warning and was not used as the project gate. Full application acceptance remains incomplete.
