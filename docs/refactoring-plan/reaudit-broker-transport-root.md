# Independent broker transport recheck

## Status

Historical evidence. Its dated findings and source pins describe the recorded review only.
## Authority

Current authority: main defines the intended split-crate architecture; design/interaction-polish defines UI/UX, colors, and behavior.
## Current evidence

Use only findings tied to the revisions and methods recorded in this document; they do not establish current implementation state.
## Remaining work

Preserve concrete requirements and unresolved items in docs/remaining-work.md; superseded procedural directions are not current instructions.
## Verification

Verification is limited to the recorded commands, revisions, and results. Re-run against current sources before closure.


The coordinator read the complete current broker-bootstrap-driver.py, including its transport controls, and independently executed the focused actual Rust parser/private-FD transport campaign with ordinary and optimized Python:

```
rtk proxy python3 -B docs/refactoring-plan/evidence/broker-bootstrap-driver.py --transport-self-test
rtk proxy python3 -O -B docs/refactoring-plan/evidence/broker-bootstrap-driver.py --transport-self-test
```

Both commands completed with exit0. Each compiled the actual observer with the repaired executable compiler-identity boundary and executed ten controls: coherent positive and fresh recovery accepted; always-pass negative, always-reject positive, zero execution, forged records, recomputed forged digest, Boolean-typed exit, malformed JSON and duplicate key rejected for their exact intended reasons. The independently observed result retained `submitted_checker_qualified=false` and an empty AST-premise digest map because this was the focused transport campaign, not a rerun of all45 AST premises.

This supplements the separate author's ordinary/optimized45-premise and10-control campaign. It does not qualify the future submitted architecture checker, the planned production singleton or terminal lifecycle behavior. TASK-072's mandatory `--runner` gate and TASK-009's real lifecycle/AST proof remain required. No production file or baseline was changed.
