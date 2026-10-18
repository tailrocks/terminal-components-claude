# F11 · Test-only conformance registry

**Status:** canonical foundation contract; implementation is future work.

**Legacy families:** C53, C54.

**Scope:** the shape, ownership, and production-dependency boundary of the
future Termrock conformance registry. It is test infrastructure, not a
production plugin registry, component framework, or acceptance policy.

The conformance package will qualify the in-place Termrock refactor on
`termrock-implementation`. It uses the preserved applications and frozen baseline
without changing their source or expected output during registry operation.

## Contract ownership

The [verification contracts](../verification/README.md) own oracle authority,
provenance, comparison semantics, case coverage, and acceptance gates. The
[component documents](../components/README.md) own applicable component cases.
The [public API contract](../api/public-api.md) owns external-consumer rules.
This foundation owns how those cases are represented and kept out of the
production library.

## Registry shape

The registry contains stable case IDs, component/API ownership, fixture and
source references, sealed input programs, declared applicable axes, and an
authority-lane label. A not-applicable axis carries its reason. Manifests are
requirements; their presence or schema validity does not count as execution or
approval.

```rust
pub struct ComponentCase {
    pub id: CaseId,
    pub component: ComponentKey,
    pub fixture_revision: Revision,
    pub authority: AuthorityLane,
    pub program: SealedProgram,
    pub axes: Vec<ApplicableAxis>,
}

pub enum AuthorityLane {
    ExistingOracle,
    ExtractedOracle,
    Extension,
}

pub struct ConformanceRegistry { /* test-only validated case index */ }
```

The type names are target notation, not promised source symbols. Lane meaning,
provenance, sealing, and approval requirements are defined in
[oracle and provenance](../verification/oracle-and-provenance.md); this file
does not redefine those rules.

## Ownership and dependency boundary

- The registry owns case identity, references, applicable-axis declarations,
  and the sealed test-program index.
- Component contracts own each case's behavior and non-applicability reason.
- Verification owns expected observations, comparator rules, evidence lanes,
  acceptance receipts, and negative mutations.
- The caller/application owns domain fixtures and controlled values supplied
  to a case; the registry never performs real product actions.
- Capture and approval authority remains outside candidate implementation.

The production Termrock library must not depend on the registry, `tui-snap`,
PTY tooling, fonts, PNG/HTML libraries, source discovery, or filesystem
manifests. The in-place package shape is one public library crate plus one
nonpublished conformance crate in the same repository workspace; the
conformance package owns those tools. The frozen baseline harness may retain
its existing test-only dependency while it remains the oracle. Tests exercise
public exports; they do not introduce a production `Widget` trait, plugin
system, route registry, or dynamic component installer.

## Registry lifecycle

1. Validate unique case IDs, referenced component/foundation IDs, and the
   dependency closure before execution.
2. Load an explicitly selected case through its trusted lane and sealed
   program.
3. Pass observations to the verification-owned comparator and reporting
   policy.
4. Preserve every declared case and report missing or blocked evidence as a
   failure; candidate output cannot add expected results.

The exact observation fields and test lanes are owned by the
[conformance verification contract](../verification/conformance.md). This
foundation only defines the registry boundary and lifecycle.

## Required foundation checks

- The registry is unavailable to production callers and is absent from the
  Termrock library dependency graph.
- Duplicate case IDs, unresolved component references, malformed axes, and
  dependency cycles fail registry validation.
- A valid manifest with missing evidence remains unexecuted and cannot pass.
- Case selection cannot remove required rows or relabel an oracle lane.
- Reports preserve registry IDs and lane labels without copying secret input.
- The 45 component/API surfaces, 12 foundations, and 54 legacy families stay
  reconcilable against the machine-readable inventories in
  [`../reference/`](../reference/).

The current `tests/visual_baseline` files and `src/lib.rs` are migration
evidence. They are not modified by this documentation goal and do not define
the target registry API.
