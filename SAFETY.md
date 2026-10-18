# Safety rationale record

## Scope

This file records the rationale that accompanied unsafe operations at source revision [`d9e569b`](https://github.com/tailrocks/terminal-components-claude/tree/d9e569bf6fd33d90266db597c458442c729340ad). It is historical evidence, not a current safety certification or proof that the visual-refactor branch is ready. The linked source revision is the authority for the exact operation reviewed; inspect current code and tests before relying on this record.

The documentation/comment cleanup is not evidence that these operations were independently re-audited. It does not claim that any safety obligation below has been discharged.

## Recorded operation rationales

### Allocation counter wrapper

The bootstrap allocator wrapper preserves `System`'s pointer and layout contract and performs atomic counter increments before forwarding allocation, deallocation, and reallocation. The original explanations are in [`runner-bootstrap-performance.rs`](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/refactoring-plan/evidence/runner-bootstrap-performance.rs#L12) and the sealed copies for [task 070](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L12), [task 071](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L12), and [task 072](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L12).

### Runtime signal and terminal calls

The recorded runtime operations include installing a signal handler that performs an atomic store, restoring a default signal disposition and raising in ordinary context, checking whether descriptors refer to terminals, polling one descriptor without waiting, and querying/updating terminal attributes. The exact source context is [`src/runtime.rs` at the recorded revision](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/src/runtime.rs#L380).

### Terminal suspend PTY test

The test launches a helper around a PTY job, coordinates process groups and terminal foreground ownership, waits for the tracked job, and reads terminal attributes. The original safety explanations are in [`tests/terminal_suspend.rs` at the recorded revision](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L93).

The `pre_exec` closure runs after `fork` in a child of a potentially multithreaded process. It must remain limited to operations permitted in that context; `setpgid` is intended to satisfy that restriction. The parent-side `setpgid` call is intended to close the race between child setup and parent foreground handoff. These are obligations to review, not claims that every error or race case has been proved harmless.

The test initializes `termios` storage before `tcgetattr`, then compares its byte representation. This record does not prove that libc initializes padding bytes or that reading every byte, including padding, is valid. The POD description in the historical comment alone is not such proof; the byte-comparison approach requires a platform/layout-specific safety review.

### `xtask` environment mutation test

At the recorded revision, one test temporarily mutates `CI` to exercise refusal paths. Its comment relied on nextest process isolation. Running each test in a separate process does not prove that no thread or environment reader exists within that process, so process isolation alone is insufficient justification for concurrent environment mutation. Preserve this limitation when reviewing or changing the test.

## Verification and future changes

The links above identify the historical source locations; they do not report current verification results. For any change to an unsafe operation or its execution context, inspect the operation and all callers in the current source, establish the applicable platform/API guarantees, and add focused verification where feasible. Keep this record synchronized only with concrete reviewed evidence; do not infer safety from a passing test suite or from the presence of this file.
