# Implementation quality and independent review

This contract owns cross-cutting implementation-quality gates and the review
protocol for the in-place Termrock refactor. It does not replace the detailed
API, component, visual, interaction, or oracle contracts linked below.

## Toolchain and package boundary

- Use one verified stable Rust toolchain pinned through the repository's
  `mise.toml` for local work and CI. The P0 package establishes and records
  this entry point; this documentation phase does not add or change runtime
  configuration.
- Keep the target package shape in [CRATES.md](../../CRATES.md): Termrock library
  crates with the `termrock` facade, several functionality crates per preview
  application, and nonpublished conformance crates. This supersedes the earlier
  one-public-library-crate limit. Do not split production code by widget.
  A further production crate split needs measured
  compile-time or dependency-isolation benefit.
- Keep terminal session support optional. PTY, `tui-snap`, raster, font,
  report, and case-manifest tools must remain outside the production library
  dependency graph. The frozen baseline harness may retain its existing
  test-only dependency while it remains the oracle.
- Keep tests in separate source files. Use `cargo nextest` for Rust validation;
  task commands use `rtk cargo nextest` inside the pinned toolchain environment.

## Blocking implementation findings

The following block task acceptance and release review:

- missing public documentation or broken local documentation links;
- unsafe production code, TODO production paths, or broad lint exemptions;
- unchecked value, index, or geometry arithmetic;
- production dependencies on conformance, PTY, snapshot, raster, font, or
  report tooling;
- private component imports or replacement painters in generic external
  consumers;
- any changed frozen application output, interaction, fixture, scenario, or
  approved snapshot.

Keep each production Rust source file at or below 400 lines, excluding
generated source. Split on cohesive responsibilities, not one-method traits or
one crate per widget. Record a reviewed exception before exceeding this limit.

## Performance evidence

The proposed review-lane CI target is two minutes on a declared runner. This is
a target, not a measured claim about the current repository. Report cold and
warm compile times, test-shard durations, runner details, and the full-workspace
gate duration. Improve the lane by sharding, prebuilding, or reusing immutable
artifacts; never skip required states, narrow the declared coverage, or relax a
parity gate to meet the target. The P7 performance task owns the measurements.

## Independent review packets

Run each review against the committed candidate and the canonical contracts.
Record the reviewed commit, toolchain, evidence digests, results, and unresolved
findings.

| Reviewer | Required inspection |
| --- | --- |
| API and consumer | Compare the [public API](../api/public-api.md), [author boundary](../api/authoring.md), [foundations](../foundations/README.md), and [component contracts](../components/README.md). Confirm caller-owned state, borrowed props, `update`/`draw`/`measure`, typed actions, stable keys, shared text/scroll/layer mechanisms, constrained parts, and secret exceptions. Compile a real external consumer using public exports only; reject abstractions without a demonstrated component need. |
| Visual and interaction | Compare exact frozen application output and event traces under the [visual](../verification/visual-parity.md) and [interaction](../verification/interaction-parity.md) contracts. Exercise Unicode, color modes, hover suppression, field-specific Escape/paste behavior, narrow geometry, protected fades, and every applicable animation phase. Candidate-only captures cannot approve expected output. |
| Coverage and adversarial | Reconcile every legacy family, component, advertised variant/part, applicable state, and transition. Mutate a cell/pixel, cursor, action target, duplicate activation, case membership, and expected artifact; each mutation must fail. Reject size-specific replicas, dead controls, paint-over workarounds, and secret leaks. |
| Simplicity and runtime | Inspect production dependency graphs, lifecycle cleanup, geometry freshness, headless operation, optional backend isolation, and measured compile/frame costs. Confirm conformance tooling is test-only and any production crate split has measured benefit. |

Reviewers use the canonical verification and component records for exact case
lists and acceptance semantics. A summary checklist must not become a second
behavior specification.

## Closure

P7 is complete only when all four reviews have evidence, all required parity
and negative gates pass, performance measurements include cold/warm/shard
results, public examples compile without private imports, and no blocking
finding remains. Product changes to Showcase, TablePro, Jackin Preview, or
Holla require a separately approved product change.
