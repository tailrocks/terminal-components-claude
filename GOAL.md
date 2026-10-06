/goal
# Finish Termrock through reusable components and exact visual-baseline parity

## 1. Active task

Finish the real Rust refactor across all 44 workspace crates, all four preview applications
(`showcase`, `jackin-preview`, `holla`, and `tablepro`), the required test coverage, and reproducible CI.
Execute exclusively on `termrock-implementation`.

Acceptance requires satisfying every condition conjunctively:

```text
exact default visual parity
AND observable interaction parity
AND reusable component ownership
AND simple, coherent public APIs
AND supported customization
AND one implementation of each shared mechanism
AND independently reviewed, executable evidence
```

Preserve the 44-member functionality-based workspace under `crates/`.
Preserve preview application flows, state transitions, CLI contracts, and visible content
using fixture data and simulated operations. Do not connect real external providers, credentials,
databases, Docker, or services.

## 2. Authorities

Work only on the existing `termrock-implementation` branch.
Do not reset/rebase the branch, merge `main`, force-push, or rewrite history.
Do not move, delete, recreate, or retarget the frozen `visual-baseline` tag or release.

| Reference | Purpose |
| --- | --- |
| Frozen `visual-baseline` tag (`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, object `1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`) | Immutable visual and observable interaction authority. |
| `origin/visual-baseline` branch | Test/scenario reference only after proving tag relation; never expected output. |
| `termrock-refactor` | Planning and preparation history. |
| `termrock-implementation` | Active candidate implementation branch. |
| `main` | Structural and API comparison only. Never visual authority. |

Research identities and branch-versus-tag comparison live in
[docs/implementation/findings-record.md](docs/implementation/findings-record.md).

## 3. Write scope

Authorized write scope:
- Rust source (`crates/**/src/**`, `Cargo.toml`, `Cargo.lock`)
- Integration, characterization, and conformance tests (`crates/**/tests/**`)
- Repository tools and xtasks (`crates/termrock-xtask/**`, `.velnor/**`)
- Generated CI configurations and workflows (`.github/**`)
- Documentation, specifications, recipes, and checklists (`docs/**`, `SPECIFICATION.md`, `CRATES.md`, `CHECKLIST.md`, `checklist.json`, `component-ownership.json`, `crate-map.json`, `GOAL.md`, `AGENTS.md`)
- Review receipts (`docs/implementation/review-receipts/**`)

Do not modify frozen baseline artifacts (`baselines/**`) or retarget the `visual-baseline` tag.
Do not stage or commit the `termrock-doc-guardrails/` input pack.

## 4. Canonical owners

| Contract | Owner |
| --- | --- |
| Agent entry rules | Root `AGENTS.md`. |
| Component-only composition | [docs/architecture/component-composition.md](docs/architecture/component-composition.md). |
| Public signatures and ownership | [docs/api/public-api.md](docs/api/public-api.md) and [types.md](docs/api/types.md). |
| Consumer examples | [docs/api/consumer-recipes.md](docs/api/consumer-recipes.md) and [example-catalog.json](docs/api/example-catalog.json). |
| Per-work and per-commit review | [docs/process/per-commit-review.md](docs/process/per-commit-review.md). |
| Visual and ownership verification | [docs/verification/ownership-and-parity.md](docs/verification/ownership-and-parity.md). |
| Individual component contracts | [docs/components/](docs/components/README.md) files. |
| Research record and findings | [docs/implementation/findings-record.md](docs/implementation/findings-record.md). |
| Remediation backlog | [docs/implementation/code-remediation-backlog.md](docs/implementation/code-remediation-backlog.md). |
| Current implementation status | `checklist.json` and derived `CHECKLIST.md`. |

## 5. Central rule: Component ownership

A preview screen is a composition of reusable Termrock components.
The component owns its pixels, geometry, interaction, measurement, and state rules.
The preview supplies domain data, layout choices, durable component state, and typed application actions.

Low-level painting belongs strictly inside reusable library components.
Preview code must not use raw cell writes, full-screen historical painters, scenario/size bypasses,
or painted-over components.

## 6. Execution discipline and per-commit review

Follow [docs/process/per-commit-review.md](docs/process/per-commit-review.md) for every work item and every proposed commit:

```text
approved small work plan
→ implement one coherent change
→ verify the exact candidate
→ independent review
→ commit immediately
→ push promptly
→ continue
```

Freeze the parent SHA, proposed Git tree, and SHA-256 of the full staged binary diff before review.
The reviewer must not author the submitted change.
Commits must strictly use sign-off: `Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>`.
All Rust test runs must use `cargo nextest`, never `cargo test`.
