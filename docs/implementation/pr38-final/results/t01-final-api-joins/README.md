# T01 final API joins

This supplement joins W44 declarations and extension points to accepted API records.
It records the derived `Default` implementation and the unresolved `InspectorBody` gap.

The builder uses fixed source and producer commits. It pins declaration files and manifests to the candidate.
The validator checks source hashes, joins, public paths, and reference closure.

W44 records retain seven type paths, associated items, and source-backed module and facade edges.
Extension records retain all 30 IDs and their path evidence.
The derived `Default` row has source evidence and no explicit implementation row in the accepted index.

`CompositionLeaf` has a private definition module. Its source owner remains separate from its public aliases.
`SlotFn` has a source-exact index row whose signature field includes trailing text. The supplement records that limit.
`InspectorBody` remains an unresolved T07 product gap. No retirement decision exists. The proposed later case has no allocated ID.

T01 evidence is structural. Functional case count is zero. Behavior remains `NOT VERIFIED`.

## Run

1. Run the builder from the repository root. It writes `api-join-supplement.json`.
2. Run the validator. It checks IDs, references, source hashes, and scoped gaps.
3. Record the output commit and artifact hash for aggregate integration.

```sh
python3 docs/implementation/pr38-final/results/t01-final-api-joins/build_join_supplement.py
python3 docs/implementation/pr38-final/results/t01-final-api-joins/validate_join_supplement.py
```

See [run-results.json](run-results.json) for command exits, counts, and artifact hash.
