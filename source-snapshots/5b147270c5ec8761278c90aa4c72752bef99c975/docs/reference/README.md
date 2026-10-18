# Machine-readable Termrock references

This directory is the machine-readable companion to the canonical Termrock
documentation. It was imported from `termrock-library-spec/` and adapted for
the in-place refactor.

## Authority and destination

Execution is divided into two canonical stages:
1. **Stage A (Preparation on `termrock-refactor`):** baseline capture, verification, and CI qualification with `src/**` strictly read-only.
2. **Stage B (Implementation on `termrock-implementation`):** in-place Rust Termrock library refactor, created from the accepted Stage A commit.

The repository itself remains the implementation project. The target is:

```text
repository:     donbeave/terminal-components-claude
preparation:    termrock-refactor
implementation: termrock-implementation
mode:           in-place refactor
future identity: Termrock
```

There is no second repository or application migration destination. Narrative
contracts own the meaning of the data:

- [Architecture](../architecture/overview.md) owns cross-cutting ownership
  and consolidation decisions.
- [Public API](../api/public-api.md) and [types](../api/types.md) own the
  caller-facing API and type contracts.
- [Foundations](../foundations/README.md) own shared mechanisms.
- [Components](../components/README.md) own per-component behavior and parity
  cases.
- [Verification](../verification/oracle-and-provenance.md) owns oracle
  authority and comparison rules.

The JSON files below support inventory reconciliation, capture planning and
provenance. They do not introduce a competing prose authority. Tasks must link
to the canonical documents above instead of copying these records into new
contracts.

The component manifest records frozen source paths and proposed module names,
not a second workspace layout. Future implementation paths are owned by the
in-place architecture and task scopes; no `crates/termrock` destination is
imported from the superseded source-pack plan.

## Frozen provenance

Every visual or capture reference is anchored to the immutable baseline commit

```text
4a79c0a2d40fca46fc406b77157ce3b3f12ec16b
```

The annotated tag object recorded by the source pack is
`1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5`. The original source URLs and
source commits in `sources.lock.json` remain pinned for provenance; they are
not rewritten to a moving branch. The external functional-requirements pin is
metadata only and has `implementation_in_scope: false`.

## Files

| File | Content | Status and owner |
| --- | --- | --- |
| `components.json` | 45 component/API records, source references, phases, dependencies, state axes and parity cases | Machine inventory; component prose owns interpretation |
| `public-api.json` | 45 proposed public surface records and signatures | Machine API index; `docs/api/` owns the contract |
| `foundations.json` | 12 shared foundation records, APIs, sources and tests | Machine foundation index; `docs/foundations/` owns the contract |
| `family-disposition.json` | All 54 legacy-family dispositions and their Termrock destinations | Traceability index; consolidation is explained in architecture/foundation/component docs |
| `sources.lock.json` | Baseline, source paths, source pins and toolchain provenance | Provenance only |
| `validation.json` | Source-pack validation result and counts | Imported pack report; it does not claim Rust or implementation validation |
| `capture-plan.schema.json` | JSON Schema for component capture plans | Plan format |
| `capture-plans/*.json` | One planned capture contract per component | 45 plans; each is `planned_not_captured` and uses the frozen oracle |

The 12 foundation and 45 public-API JSON records are retained because they
carry complete machine-readable fields used for reconciliation. Their
canonical narrative owners remain the documents linked above. `validation.json`
is retained as a provenance snapshot of source-pack checks, separate from the
repository's future verification gates.

The F02 record links `Response<A>` to its canonical shape in
[`docs/api/types.md`](../api/types.md#typed-response) instead of repeating the
field declaration, keeping the machine inventory and prose on one type owner.

## Capture-plan rules

Capture plans preserve dimensions, applicable state axes, exact case IDs,
expected-artifact declarations, source references and authority lanes. They are
plans, not captured snapshots. Candidate output cannot create or bless its own
expected baseline; future comparisons must use the existing or independently
extracted baseline described by the verification contract.

The source-pack `reference/TYPES.md` is merged into
[`docs/api/types.md`](../api/types.md), so it is not duplicated here. The
packaging wrapper (`index.html`) and checksums file are not copied either;
they contain navigation/package metadata. The JSON files and canonical
repository documents carry the substantive requirements.

## Validation

All JSON in this directory must parse. Reconcile the inventories before changing
the plan:

```sh
find docs/reference -type f -name '*.json' -print0 |
  xargs -0 -n1 jq -e . >/dev/null
jq -e '.components | length == 45' docs/reference/components.json >/dev/null
jq -e 'length == 54' docs/reference/family-disposition.json >/dev/null
jq -e '.foundations | length == 12' docs/reference/foundations.json >/dev/null
find docs/reference/capture-plans -type f -name '*.json' | wc -l
```
