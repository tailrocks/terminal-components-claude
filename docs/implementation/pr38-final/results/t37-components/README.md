# T37 component cleanup result

- Input tree: `c817b99e9e43abbbf3519d56b2f5ee86924243d8`. Scoped source bytes match T00 product source `1d6ee92adf6ccca38eeee64b1371cba1497e8fcd`; each changed source and deleted test file is bound to its T00 SHA.
- Output source commit: pending the worker commit.
- Worker: `/root/t37_components`, `gpt-6-luna` max effort.
- Scope: the ten Termrock crate trees in `summary.json`; manifests and generated files are excluded.
- Removed: 161 T00 inventoried test and fixture source paths, 63 inline `cfg(test)` modules, 1869 explicit test function inventory rows, plus test-only helpers, fields, branches, counters, and imports in `removed-cfg-items.jsonl`.
- Raw Rustdoc: 722 groups / 1800 lines are preserved in `removed-rustdoc.jsonl` with source SHA and input line groups. This is historical provenance, not active API guidance.
- Token preservation: `source-token-map.jsonl` records per-file input/output hashes, test deletions, removed non-comment token counts and hashes, and surviving token/literal equality for 82 unaffected source files. Three viewport/facade files have the separately approved testing API deletion. `approved-testing-api-retirement.jsonl` lists every removed non-comment token against c817 input line groups; it records 33 opcodes, 716 tokens, and no insertions/replacements.
- Preserved: `TextViewport::selection_binding_chords`, facade runtime testing exports, `#[doc(hidden)]` metadata, and production lint/feature attributes. Retired `ViewportWorkProbe`/`ViewportWorkSnapshot` under independent Sol high decision T37-PROBE-02; exact token deltas are in `approved-testing-api-retirement.jsonl`. T00 lists no scoped unsafe sites or required source-comment directives.
- Manifest requests: listed in `summary.json`; those files are outside this worker's writable scope.
- Generated output: no generated file changed in this scope.
- Required viewport work bounds remain in API-RULES §10 and VALIDATION §7.1 for T17/T33. These contracts do not establish behavior on this T37 revision.
- Tests selected: 0. Runtime behavior is `NOT VERIFIED`. No Cargo tests or builds were run by this worker.
- Maintained API guidance: raw prose is not treated as a contract transfer. Details not represented in current contracts remain open for T01/T35 and the shared docs owner.

- Second follow-up from accepted `96fe6a2a`: after independent Sol high confirmation of T37-FOUNDATION-TESTING-01/02, the facade testing re-export for `ProjectedFrame`, `RenderModel`, and `StyledQuery` was removed with its now-empty cfg attribute. Runtime declarations are foundation-owned. The exact 21-token delta and hashes are in `approved-testing-frame-facade-retirement.json`.

- Third follow-up after accepted `f03e22c14`: independent Sol high decision T37-SELECTION-BINDING-01 retires only `TextViewport::selection_binding_chords` and its testing cfg. The 59-token deletion, f03/d402 input blob identity, and preserved registration/action definitions are recorded in `approved-selection-binding-retirement.json`.
