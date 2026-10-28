# TCC common Git object payload archive

**Status:** all 97 payload chunks are published. The complete ref, reflog, stash, worktree, and index metadata set and independent final remote-custody verification are pending. This is not a complete repository backup or product acceptance result.

The archive contains the validated physical-object payload captured from the TCC shared object database at source HEAD `f629e9fabde2c9fb7f1ae4fd8f8d9a98cd07ba93`. `portable-manifest.json` records 97 ordered chunks. `SHA256SUMS` covers the published payload and metadata files in this archive tree.

The reconstructed pack has SHA-256 `3524526d4accf5bb689fb2ab5b4896728802c25a59c7fc5864756239d9133c85` and exact size 4,847,685,238 bytes. It contains 152,827 objects. The source physical inventory had 257,216 typed objects; the pack excludes only the exact f629 reachable closure of 104,389 objects.

## Restore prerequisites

Use a fresh object database and fetch the exact base commit `f629e9fabde2c9fb7f1ae4fd8f8d9a98cd07ba93` from `https://github.com/tailrocks/terminal-components-claude.git`. The base closure must match `inputs/f629-closure.oids` (SHA-256 `1568ba5671ec995483169332d4982e41993549e067a88ac3828274ebf7e7b8fe`). Concatenate `chunks/` in numeric sequence order, with no separators, to reconstruct the pack. The reconstructed pack must have SHA-256 `3524526d4accf5bb689fb2ab5b4896728802c25a59c7fc5864756239d9133c85` before import. Import and strict-check it against that base; do not import it into an unrelated or empty object database. `portable-manifest.json` gives each chunk’s repository-relative path, size, and SHA-256 and the verified object-set lists.

The PR38 source commit `c33708a5da3d38e985e4eacdbacd00dfb079f5d3` is a later commit on the existing PR branch and is separate from this pre-commit payload capture. Importing this pack alone does not reconstruct original refs or worktree state; their preservation metadata is a separate archive step.

## Evidence and limits

`evidence/pack-validation-summary-v2.json` records the exact-base import, exact typed-object union, and full strict fsck result. `evidence/eligibility-review.json` contains the independent HIGH publication-eligibility decision. Raw original command receipts remain unchanged under `evidence/`; absolute paths in those receipts are provenance metadata only. The operative chunk and input references in `portable-manifest.json` are repository-relative.

The prior f629 index is identified by captured SHA-256 `bcb9ad67052dd62216941d7a124dc03b3243348a060dcb4d6246c26762cca34f` and size 462,864 bytes, but those raw pre-publication index bytes were not retained in the frozen external inventory. The final metadata record will state this limit and distinguish it from the current c337 index.
