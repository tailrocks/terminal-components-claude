# Proposed repository copy plan (proposal record)

This file records the proposed path mapping; actual publication and review status are recorded separately.

VIS-04 allowed-path root: docs/implementation/visibility/evidence/deferred/**.

Proposed destination directory:

docs/implementation/visibility/evidence/deferred/harness-controls-claim-pin-b106-v2-20261009/

The proposed operation is to copy each staged file byte-for-byte to the same relative path under that destination, including README.md, COPY-PLAN.md, MANIFEST.json, runs/, reviews/, provenance/, and source_snapshot/. The proposed file set is limited to this one evidence subtree; it does not include queue, status, or source paths.

Before any publication operation, rehash every entry in MANIFEST.json and independently compute the manifest SHA-256. If the destination is already present, do not overwrite it. Root retains apply/index/commit authority; this proposal record does not authorize repository writes.
