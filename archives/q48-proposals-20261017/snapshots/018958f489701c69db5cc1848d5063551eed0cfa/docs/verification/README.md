# Verification

This directory is the canonical verification contract for the in-place
Termrock refactor. It describes how a future implementation is compared with
the frozen `visual-baseline` repository state. It does not add captures, bless
candidate output, or change an application, test, or snapshot.

The frozen visual and observable-interaction authority is commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, resolved by the annotated
`visual-baseline` tag. The approved artifact tree under [`../../baselines/tuiscotti-v1/`](../../baselines/tuiscotti-v1/)
and the trusted drivers under [`../../tests/visual_baseline/`](../../tests/visual_baseline/)
remain unchanged. `showcase`, `tablepro`, `jackin-preview`, and `holla` are
reference consumers and conformance fixtures.

## Document ownership

| Question | Canonical document |
| --- | --- |
| Eight proof gates and the evidence each gate requires | [`ownership-and-parity.md`](ownership-and-parity.md) |
| Exact rendered cells, pixels, geometry, colors, cursor and visual states | [`visual-parity.md`](visual-parity.md) |
| Event programs, pointer traces, focus/capture, actions and semantic observations | [`interaction-parity.md`](interaction-parity.md) |
| Oracle lanes, source/tool pins, artifacts, provenance and limitations | [`oracle-and-provenance.md`](oracle-and-provenance.md) |
| Coverage registry, negative mutations, fail-closed gates and acceptance | [`conformance.md`](conformance.md) |

Architecture and API contracts live under [`../architecture/`](../architecture/),
[`../api/`](../api/), [`../foundations/`](../foundations/) and
[`../components/`](../components/). Those documents own component behavior;
these verification documents own how that behavior is proved. The execution
plan under [`../implementation/`](../implementation/) references these
contracts and does not redefine them.

## Source-pack coverage

The imported capture metadata is retained under [`../reference/capture-plans/`](../reference/capture-plans/),
with its schema at [`../reference/capture-plan.schema.json`](../reference/capture-plan.schema.json).
The reconciled source pack contains 45 component/API plans and 222 required
case descriptions. Every plan is currently `planned_not_captured`; its
`expected_artifacts` field is null. A plan is a required-set declaration, not
an approval and not a passing test.

The source-pack foundation and family registries are linked from
[`../reference/foundations.json`](../reference/foundations.json) and
[`../reference/family-disposition.json`](../reference/family-disposition.json).
The coverage gate in [`conformance.md`](conformance.md) requires all of them
to remain accounted for.

## Non-negotiable rules

- Expected output is trusted, immutable input. Candidate code is read-only with
  respect to expected artifacts and can never create, update, or approve them.
- A missing, stale, corrupt, or incomplete expected artifact fails closed.
- Existing output and newly extracted output are classified separately from
  newly specified extension behavior; extension evidence cannot satisfy old
  parity requirements.
- Pure component captures and PTY captures are separate proof lanes. Neither
  replaces the other.
- A matching picture is insufficient when focus, capture, selected identity,
  cursor, draft/commit state, or typed action semantics differ.
- A one-cell, one-color, cursor, hit-target, action-target, timing, or
  continuation-cell mutation must fail the appropriate gate.

The implementation phase may add the harness and trusted capture bundle
described here. This documentation phase leaves the baseline applications,
snapshots, test sources, and runtime code untouched.
