# Git store census r14 (read-only)

This addendum inventories the 83 non-TCC common Git object stores represented by the frozen r13 marker inventory (85 total common dirs less the Projects and Downloads TCC stores). It hashes present object-store files, enumerates full OIDs from each unique pack-index/pack pair and validated loose objects, captures exact refs/stash OIDs, hashes raw reflog and index files without emitting reflog messages or config values, and records sanitized repository identity. Hooks were only inspected and hashed when non-template; none were executed.

No repository, source tree, cache, or Git metadata was changed. Output files are metadata only. The exact source paths, sizes, hashes, and archive requirements are in `source-custody-requests.jsonl`; raw refs/index/reflogs still need to be included by Integration after privacy screening.

The two TCC common stores were excluded because their complete stores have separate owners/inventories. Existing plan/preimage/T01/Cargo groups and canonical TCC OIDs were used as comparison baselines; exact matches are recorded per common directory. A no-origin marker is not a basis for exclusion.
