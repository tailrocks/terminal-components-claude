# Independent Jackin CLI review

Candidate: `aaf9630b9f8c828ff0fd488e90f11c5ee19563a3`, clean worktree. Reference: `794b095c196562d38f1b6f7ce379c128af2a023d`, `src/bin/jackin_preview/main.rs`. Scope: four committed Jackin parser/entrypoint/test files. No source edits.

## Result

Bounded CLI restoration passes independent review. Default FirstUse, NO_MOTION absent/empty/0 versus nonzero values, explicit motion precedence, frame parsing, scenario names, short aliases, help and preterminal error paths match reference contracts. Main additive paper theme and case-insensitive color aliases remain. Unknown args retain ignored behavior. Invalid values intentionally redacted (approved security exception), while error exit remains 2.

## Independent checks

- Stable locked all-features targeted run: 3 parser tests + 5 binary contract tests pass (`test.log`). Enumeration saved in `list.log`.
- Additional 10 real-process reference/candidate comparisons: help stdout byte-identical; help exits 0; four invalid-value paths exit 2; candidate never emits sentinel secret. Saved `independent-process.json`, including exact binary hashes.
- Source review traces parser pure inputs through Motion::resolve and App scenario construction. Only Run branch enters terminal. Help/error branches avoid terminal setup.
- Builder evidence separately reviewed: stable/MSRV runs, 18 terminal launch records, 42 PTY terminal-mode records, source provenance. Those executions belong to builder, not independent reruns. Their first-use traces prove scenario startup, not full visual fidelity.

## Outstanding, not waived

1. Session `theme.for_terminal()` still narrows explicit color; parser proof does not establish end-to-end explicit-color precedence.
2. Candidate intro Ctrl-C behavior differs from pinned reference. Builder terminal tests use q cleanup; this does not prove Ctrl-C parity.
3. Pinned main explicitly drains pending input before runtime startup. Candidate startup omits drain; source-proven retained gap, not introduced by parser patch. No independent queued-input reproducer in this review.
4. Candidate chrome/layout and stored baseline schema differ from pinned reference. Existing visual baseline test failure is not approved/blessed by CLI acceptance.

Current artifact conclusions are hash-bound; later runtime/color/lifecycle changes need their own focused checks.
