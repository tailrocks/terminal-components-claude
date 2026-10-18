# Protected task inputs for TASK-001

Product: `qualified-harness`.

Implement only tc-proof compare and the thin host core interfaces in proof-contract.md under tools/refactor-proof/. The comparator must pass all 72 independently authored vectors and 69 fresh positive recoveries. The host must pass every independently authored install/prepare/freeze/verify/seal-rejection/integrate case, including hostile workers and actual standalone taskfmt. A missing or unsupported isolation capability is a failing task, not an accepted limitation. Runner, accounting and architecture operations remain distinct producers TASK-070, TASK-071 and TASK-072; this task must not claim those untested operations are qualified.

Keep project orchestration outside junie-tui and tui-snap. Use existing tuisnap schema-3 capture and exact comparison capabilities, pinned standalone taskfmt, protected host authority, immutable file receipts and local integration compare-and-swap. Do not build a scheduler, monitor, receipt service or taskfmt schema extension. Candidate build/test code runs in separate workers without expected artifacts, credentials, network or trust-root access.

Qualify nonzero exit with printed DONE, omitted logs/checks, forged receipt, wrong producer, unintegrated prerequisite, mutable source/index/symlink/hard-link attacks, changed parent, refs/heads/main, missing expected files, incomplete test execution and temporary trust writes restored afterward. Every applicable rejection preserves independently inspected refs and trusted bytes and is followed by a fresh passing case.

Source requirements:

- [docs/refactoring-plan/proof-contract.md](/work/docs/refactoring-plan/proof-contract.md).
- [docs/refactoring-plan/evidence/proof-comparator-protocol.md](/work/docs/refactoring-plan/evidence/proof-comparator-protocol.md).
- [docs/refactoring-plan/evidence/host-bootstrap-protocol.md](/work/docs/refactoring-plan/evidence/host-bootstrap-protocol.md).

The host binds the source documents, canonical historical union and scenario files named above into the immutable campaign input manifest. Their exact scenario expansion algorithms and source-qualified tests are required membership, not a best-effort glob. Rejected/deferred source clauses retain their disposition. A task-local candidate manifest cannot override this document or the protected host context.
