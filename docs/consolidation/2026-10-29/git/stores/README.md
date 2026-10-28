# Related Git-state preservation groups

This directory contains five groups that received independent publication eligibility review. Their source repositories were not mutated. Each group preserves archived data and identity metadata; these files do not create Git branches in the destination repository. Use each group’s `archive-group-receipt.json` and `SHA256SUMS` for its exact input identities and files. `group-index.json` uses repository-relative paths for all operative archive references; its source aliases are explanatory metadata only.

The T01 group is explicitly raw fixture custody: its target pack is incomplete and the archive review makes no object-closure claim. The jm2recon group records an unborn `refs/heads/main`, 11 untracked source files, and no object database or index. The Termrock Cargo group contains independent Git object databases and checkout state; its receipts distinguish those aliases from the TCC common store.

The unrelated-looking package names are preserved because inventory established their repository and project provenance. The separate tc-claude `tmp_pack_1quA7Q` file is not included here; it lacks an index and its pack trailer does not match its bytes, so it remains a separate opaque-custody candidate awaiting its exact review packet.

The HIGH eligibility review is `eligibility-review.json` (SHA-256 `9b5b504a005fcf479f2a87774d1bff6d4b4e1d508a939ac8f6092d8568fcbf87`). Publication of these five groups does not complete the all-clone remote-custody or local-cleanup gate.
