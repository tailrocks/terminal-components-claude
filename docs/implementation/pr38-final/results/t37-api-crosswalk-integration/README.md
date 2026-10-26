# T37 API prose crosswalk integration

This unit composes three explicit cohorts into one 1,122-record crosswalk. Each output row preserves its source record and unchanged cohort record.

The producer validates source identity against accepted Git blob `2fee2c277d1213fc5fad5d81eb77479600182b69`. The output reports local integration status for both source inventories. Without a local copy, the producer uses the immutable blob and reports `PENDING_INTEGRATION`.

Run from the candidate checkout:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 docs/implementation/pr38-final/results/t37-shared/build_foundation_api_prose_crosswalk.py
```

Inputs use repository-relative paths. No plan root, temporary export, or other worktree is required. Any local source copy must match the same immutable Git blob byte-for-byte.

The producer validates all 1,122 zero-based source indices, stable IDs, source paths, line ranges, file hashes, and text hashes. It checks that indices and IDs are unique and cover the full inventory. Destinations must resolve inside this checkout.

The producer copies each cohort row without changing its attachment, disposition, required clauses, excerpt, reason, destination, or decision identifiers. Cohort-specific field names remain intact. The producer does not infer dispositions from keywords, source survival, or matching signatures.

Coordinator review bindings live in `coordinator-review-decisions.json`. Each accepted decision binds a reviewer and decision ID to an exact source commit, artifact path, and SHA-256. The producer verifies that Git blob against the current artifact. Worker review fields remain quoted source metadata and never set acceptance.

| Cohort | Records | Source commit | Imported source | Independent decision |
| --- | ---: | --- | --- | --- |
| Values A | 322 | `66728b43068c1ca54deba2a9cbd4d9a9973af182` | `9463cb8c36f0bc6e53af241334efb19bb8a2e5f1` | ACCEPT `T37-API-PROSE-01`, `/root/t37_components_review`, Sol high |
| Runtime B | 304 | `4890b3d9ae40323b07f8d112d0fcf038e22059c3` | `6e38a1d3be4b0dcfd47f793e7975299254e38d14` | ACCEPT `T37-API-PROSE-01`, `/root/t00_independent_review`, Sol high |
| Support C | 496 | `ee8a18a77f1a6e7340d0dfe81bbba16f4e87ddf9` | `f2657fe1cf0dcfb7baac271b5d93c523369798f4` | ACCEPT `T37-API-SUPPORT-EE8-ACCEPT`, `/root/t00_independent_review`, Sol high |

The full artifact and runbook hashes appear in the composed JSON and `agent-runbook.json`. Earlier cohort review fields remain unchanged inside recorded cohort metadata. Historical identity-crosswalk references are labeled as pre-composition references.

Hashes inside embedded cohort rows remain tied to their source commits. They are not current candidate target hashes.

Support C is independently accepted at `ee8a18a77f1a6e7340d0dfe81bbba16f4e87ddf9`. The composition review remains `REVIEW/PENDING`. Selected behavior cases: 0. Product behavior: `NOT VERIFIED`.
