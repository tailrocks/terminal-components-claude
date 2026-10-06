# F11 · Conformance registry and public-surface checks

**Scope:** shared generic library/test infrastructure, not a product subsystem.  
**Legacy families:** C53, C54  
**Visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; interface spellings below are proposals.

## Source references

- [docs/refactoring-plan/component-parity.tsv](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/component-parity.tsv)
- [tests/visual_baseline/audit.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/tests/visual_baseline/audit.rs)
- [src/lib.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/lib.rs)
- [src/widgets/mod.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/mod.rs)

[Target architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) · [refactoring task catalog](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv)

## Proposed public surface

Rustdoc-style declaration notation. This is not a compiled implementation. Read with [PUBLIC-API.md](../PUBLIC-API.md) and the [shared type dictionary](../reference/TYPES.md).

```rust
// Test-only crate; not a dependency of termrock.
struct ComponentCase { component: &'static str, scenario: &'static str, authority: AuthorityLane, program: SealedProgram }
enum AuthorityLane { ExistingOracle, ExtractedOracle, Extension }
Harness::dispatch(&mut self, event: Input, at: Moment)
Harness::checkpoint(&mut self, name: &str) -> CanonicalObservation
compare_exact(expected: &CanonicalObservation, actual: &CanonicalObservation) -> Comparison
// CanonicalObservation includes cells, cursor, semantic trace, focus, capture and provenance.
```

## Contract

1. A registry is test data, not a production plugin system. The production library does not depend on tui-snap, PTYs, fonts, PNG/HTML libraries, discovery tools or filesystem manifests.
2. Every component has source references, proposed public signatures, applicable state cases and explicit non-applicability. Every source-family ledger row maps to at least one component/foundation/verification reference.
3. ExistingOracle means imported pinned approved output. ExtractedOracle means trusted capture from unchanged pinned source at a newly isolated state. Extension means newly specified robustness or adapter behavior. Never disguise Extension as old baseline parity.
4. Trusted oracle capture and approval are outside candidate authority. Missing required approval fails; disabled cases cannot silently reduce the denominator. Candidate has read-only expected artifacts.
5. Public API tests compile only against external consumer dependencies. No source-level include! into candidate internals, copied baseline painters, pub(crate) shortcuts or size-specific snapshot replicas.
6. Compile-fail tests cover read-only editing, illegal secret operations and private runtime access. Mutation tests verify painting and action trace gates actually fail; a successful Rust build alone is not parity.
7. This package specification and its manifests are not the implemented harness. Concrete Rust test drivers and sealed numeric capture programs are P0/P1 deliverables.

## Required proof

- all 54 legacy families dispositioned
- missing/changed expected artifact fails closed
- one cell color/cursor/target/action mutation fails
- unavailable scenario reports blocked rather than passes
- dead widget replaced by paint-over rejected
- all documented public examples compile after implementation

This foundation has no invented independent hover/pressed screenshot. Its visible effects are proved through the components and composed fixtures that use it. Nonvisual invariants have headless/state/compile-fail/protocol tests. Implementation and independent verification are not executed in this document pack.
