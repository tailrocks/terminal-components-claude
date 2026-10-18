# Provider poll attempt archive R1 (external proposal)

This proposal adds byte-checked copies under the accepted VIS-13 `reports/**` scope. Its current parent is candidate `67492afe9c6428c720a60e136f3941cbc91cd980` (tree `eea6cf5e25866fba6fa959419c71b46ed6ef70ca`); the accepted scope was recorded against `0b7545ff7e3fff40254fd7a84a80c2c8a7872f3f` (tree `48e074b5503c227422d51faaa8d8a143a295d9da`). The accepted prefix was absent at both trees. The existing R6 archive and earlier directories are unchanged. This proposal has not been committed or published.

The manifest lists each copied source path, byte count, source mode, and SHA-256. `SHA256SUMS` covers all payload files beneath this archive directory; it does not include itself, the manifest, or this README.

## Provider poll attempts

- **R4b:** the captured Nextest run reported 1 pass and 2 failures, with child exit 100 and wrapper exit 0. The capture plan is preserved with its original execution metadata. No independent R4b actual-result review receipt was located.
- **R5:** the captured Nextest run and Reader v2 report show 3 selected passes. The capture plan and Reader preparation remain unchanged with their original NOT_RUN/PREPARED fields; the actual capture and Reader receipts are separate files. An independent Reader v2 actual-result review is included.
- **Current 0b snapshot:** only the PR, run, jobs, and tool-version queries were made. The jobs API response is HTTP 200 with zero jobs. No artifacts or run-log query was captured, so no log HTTP status or provider failure cause is claimed.
- **Native ref readback:** preserved separately; it records the candidate/reference refs and immutable tag object/peeled commit.

The four-query provider snapshot below was captured while candidate tip was `0b7545ff7e3fff40254fd7a84a80c2c8a7872f3f`; it is not a claim about the later tip. The provider validator source was tested against the 0b base plus the accepted two-path overlay and was later integrated at candidate `67492afe9c6428c720a60e136f3941cbc91cd980`. These status-reporter tests are not product tests. The historical measured product pair remains candidate `1d797d41c8141fcbdc3f69d7f11eb8875ab54712` / reference `b274dd57f4dd078ade6e424d546d83efbd2e8526`, separate from current repository identities.

This archive contains raw evidence and provenance only. It adds no normalized current provider observation, source-facts update, STATUS update, or product qualification/readiness claim.
