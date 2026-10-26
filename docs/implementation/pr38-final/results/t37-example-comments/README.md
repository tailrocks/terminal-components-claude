# T37 example-comment correction

- Fixed source input: `4627a1c6f5c65a5f7527922051131080f65288cb`.
- T00 source comparison: `c817b99e9e43abbbf3519d56b2f5ee86924243d8`. The seven example files were byte-identical at both commits and combined audit `0e8e7a7cd1e11d0633309f5da4c1d5e7056dfd6c`.
- Removed 17 comment lines in 14 groups from seven preserved examples. Six leading blank separators were also removed. No non-comment token or literal changed.
- `removed-comments.jsonl` preserves each original comment with source SHA and line group. It is historical provenance, not active contract text.
- `per-file-deletion-map.jsonl` records input/output SHA-256 and exact residual-byte equality against the fixed input.
- `result.json` records the maintained guidance transfer, checks, model, and review status.
- `docs/contracts/facade-examples.md` is the maintained caller guide. It marks unobserved example claims as limits or open requirements.
- Rustfmt and whitespace checks passed. No tests, builds, or example executions were run. Behavior is `NOT VERIFIED`; independent Sol high review is pending.
