# Git store relation and closure supplement r15

This immutable read-only supplement joins the 83 non-TCC common Git stores in census r14 to exact OID-set equivalence, physical object-tree hashes, refs, and local object availability. It compares full 40-character OIDs against the canonical TCC OID manifest and prior plan/Cargo/preimage/T01 groups. It also compares ref name/OID pairs with the frozen Downloads-TCC ref/reflog metadata, without claiming that reflog/ref metadata proves object bytes are physically present.

No source/Git data was changed. No raw reflog messages or config values are included. Use r14 `source-custody-requests.jsonl` for exact byte paths/hashes; this addendum supplies lineage and closure relations.
