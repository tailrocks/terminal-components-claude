# Independent Form validation adjudication review

2026-09-11. Planning-only, source-backed review of ADJ-12 and TASK-006/019/058/068 bindings. No production or canonical edits. Verify-and-stop limits this result to the stated contract.

## Authority and root condition

ADJ-12 is supported. Original `27bd918e3a8a0a7fdba14fb10643139340d6281f:COMPONENT_ARCHITECTURE.md:1477` requires every visible field in declaration order; the FormData comment:1411 separately admits cross-validation only after every per-field check passes. Main `7b27732a8c3c131760ec3438f641cb3c11343a42:COMPONENT_ARCHITECTURE.md:1702,1634` retains both. The first Form implementation `a1759b2a:crates/tui/src/components/form.rs:878–904` already returns at the first invalid field; current main:1175–1217 retains that early return. Implementation chronology is not an accepted amendment.

The root defect is a loop that uses “first failure” as permission to stop collecting observations. That conflates the one Invalid/focus result with the required complete ordered validation pass. A first-invalid-ID accumulator plus existing per-ID errors removes this enabling condition without adding a public policy or app-owned second engine.

## Source-visible counterexample

Exact oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c:src/bin/tablepro/connections.rs:246–250` evaluates name.validate and port.validate separately before combining booleans. Empty name and port0 therefore produce both “Required” and “Port must be 1–65535” (validators:89–105). Submit:689–711 commits active inputs, rejects the save and chooses name before port for focus. Render:1131 marks the Basic tab if either error exists;:1145–1153 draws both controls, so dropping the second error is observable. Input:177–185 stores each validation result. This is not inferred from a title or generic form claim.

The simultaneous-error requirement belongs to058's connection composition. The modern four-field architectural callback fixture is not an invented old Form widget.006's new explicit paragraph preserves those distinct lanes.

## Contract sufficiency

019 trusted obligations:11/21 and W-019-10 now require visible A/B/C plus hidden D; first, middle, last and simultaneous failures; all-visible-valid/cross-invalid; and full success. Required traces [A,B,C], [A,B,C,cross], [A,B,C,cross,submit] distinguish early return, duplicate/reordered/omitted validators, wrongly admitted or omitted cross-validation and premature submit. Active commit occurs once before clearing stale errors. All local failure slots survive; only the first invalid controls final reveal/focus/single Invalid. Hidden D never validates. Existing tests remain and gain discriminating coverage rather than being renamed as evidence.

Main already owns sensitivity-aware retention at form.rs:566, generic public error projection:546, and current-data private rendering/safe_error:1328–1356. The plan retains these paths for every collected error, including dynamic-sensitive cases; no raw secret detail may escape merely because a later field is now visited. Cross-validation is conditional, not an unconditional third step misread from F10's abbreviated list. Documentation owner068 reconciles that ambiguity without erasing original history.

006 binds the updated witness as immutable input, not a future019 receipt.019 owns the loop;058 owns source application error presentation;068 owns implementation-matching public documentation. All three ADJ-12 trace rows map to actual canonical R/AC/CHK arrays. No configurable fail-fast/collect-all API was introduced.

## Actual checks and limits

Pinned taskfmt lint with explicit task-format experiment.toml passes006/019/058/068: zero errors/warnings. On the readonly pinned-main worktree, `rtk cargo test --locked -p junie-tui --lib components::form::tests::submit_` passes4 tests (760 filtered). Read the actual `submit_validates_every_visible_field_then_focuses_the_first_error` at:2282–2296: its MatrixApp has one field and expects one callback. That passing test cannot prove the disputed total loop; this limitation is the exact reason for the new finite witness.

No corrected production implementation or future mutation suite was executed. Acceptance here is the source/ownership/proof contract, not completion of F10 implementation or entire planning goal.

## Byte snapshot

```tsv
path	lines	sha256
docs/refactoring-plan/architecture-adjudication.md	172	7ed0fb1a72057d5dd5f2d40a0444bc7e7d77a5d0fd16d71c9626d84076d723a7
docs/refactoring-plan/decision-ledger.tsv	13	5df9a02c382988c164bcd8d1b9943ca46dcc008704088746005f2522c8432291
docs/refactoring-plan/traceability-closure.tsv	30	9e4792cd8900b3143c7d7f986a229b370c98bb84fe709165065d56e3804dd41c
refactoring-tasks/terminal-components/completion/006/trusted/obligations.md	26	daa18778c7c66abfe032f283d8317b2d7406f19cc25a56b4d06477825547de8f
refactoring-tasks/terminal-components/completion/019/trusted/obligations.md	442	7421b92ed1a9cdc81b72e830c663f7eaaff7dd9386022d2658f81e262147f87f
refactoring-tasks/terminal-components/completion/058/trusted/obligations.md	201	46cef2f40ab7722496a650bdeedca1e89cf2fe374646d07466a2f27cf7e8e21c
refactoring-tasks/terminal-components/completion/068/trusted/obligations.md	37	f398dd48c22812ea3ef3648d39603118e7c62a935aa442283cf10dda01eda3f8
refactoring-tasks/terminal-components/completion/019/trusted/source-witnesses.md	29	925bbdf69a4c0b68705d88bfbf0844e0ee4436d6d33965548189388630a3b50f
```

