# T37 configuration guide correction

This follow-up preserves the accepted `ee2b0de1c76dc599c0f588201cb63e1845bcd8ac` records and corrects the maintained guide at `docs/contracts/build-configuration.md`. The current Velnor Actions repository configuration is `.velnor/config.toml`. The retained `.github-gen/velnor-workflow.toml` is historical material and the source of the R-43 note, not the current Velnor Actions input.

The guide preserves the note's fixed-input claims: D19 guard; R-43 marked blocked; one reusable unit workflow expanded per crate; 44 members exceeding the 5 MiB GitHub template ceiling (99 callers = 8.24 MiB); and regeneration when the generator chunks units or the workspace splits lanes. It states that these claims do not establish current generator behavior, size, or status.

The two generator logs enumerate 15 output paths each. The earlier accepted comparison contains per-run content snapshots for 12 selected operational paths. The other three paths—`.github/AGENTS.md`, `.github/CLAUDE.md`, and `.github/jactionlint.yaml`—are recorded here only by fixed-input-to-`ee2` Git entry identity. For `.github/CLAUDE.md`, that identity includes mode `120000` and the symlink target bytes. No per-run content snapshot is claimed for those three paths. Exact paths, modes, Git blobs, hashes, and prior snapshot references are in [generator-output-scope.json](generator-output-scope.json).

[guide-correction.json](guide-correction.json) binds the corrected guide hash and records the review basis. This change did not modify configuration, generated output, or build inputs and did not rerun Velnor, Cargo, or formatting checks. It adds no test or behavior claim.
