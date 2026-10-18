# SAFETY

## Status

Retained safety rationales for unchanged unsafe operations. Removing Rust comments does not remove these obligations. This record is not a new proof that the incomplete product refactor is ready.

## Authority

Preserve existing validation and safety contracts. `main` supplies architectural direction; `design/interaction-polish` supplies product behavior. No unsafe operation or safety lint is changed by this documentation migration.

## Current evidence

Rationales below came from Rust comments at `d9e569bf6fd33d90266db597c458442c729340ad`. Comment removal and reviewed formatting preserve their operations. Original line links retain the exact source context; current source line numbers may differ.

### docs/refactoring-plan/evidence/runner-bootstrap-performance.rs, original line 12

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/refactoring-plan/evidence/runner-bootstrap-performance.rs#L12)

```text
SAFETY: This wrapper preserves System's pointer/layout contract and performs
only allocation-free atomic increments before forwarding each allocation.
```

### docs/refactoring-plan/evidence/runner-bootstrap-performance.rs, original line 17

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/refactoring-plan/evidence/runner-bootstrap-performance.rs#L17)

```text
SAFETY: GlobalAlloc supplies a valid layout; System owns allocation.
```

### docs/refactoring-plan/evidence/runner-bootstrap-performance.rs, original line 21

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/docs/refactoring-plan/evidence/runner-bootstrap-performance.rs#L21)

```text
SAFETY: The caller provides the original System pointer and layout.
```

### refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 12

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L12)

```text
SAFETY: This wrapper preserves System's pointer/layout contract and performs
only allocation-free atomic increments before forwarding each allocation.
```

### refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 17

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L17)

```text
SAFETY: GlobalAlloc supplies a valid layout; System owns allocation.
```

### refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 21

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L21)

```text
SAFETY: The caller provides the original System pointer and layout.
```

### refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 12

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L12)

```text
SAFETY: This wrapper preserves System's pointer/layout contract and performs
only allocation-free atomic increments before forwarding each allocation.
```

### refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 17

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L17)

```text
SAFETY: GlobalAlloc supplies a valid layout; System owns allocation.
```

### refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 21

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L21)

```text
SAFETY: The caller provides the original System pointer and layout.
```

### refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 12

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L12)

```text
SAFETY: This wrapper preserves System's pointer/layout contract and performs
only allocation-free atomic increments before forwarding each allocation.
```

### refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 17

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L17)

```text
SAFETY: GlobalAlloc supplies a valid layout; System owns allocation.
```

### refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 21

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L21)

```text
SAFETY: The caller provides the original System pointer and layout.
```

### src/runtime.rs, original line 380

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/src/runtime.rs#L380)

```text
SAFETY: installing a handler that performs one atomic store.
```

### src/runtime.rs, original line 395

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/src/runtime.rs#L395)

```text
SAFETY: default disposition then raise; both are signal-safe libc
calls made from ordinary (non-handler) context.
```

### src/runtime.rs, original line 506

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/src/runtime.rs#L506)

```text
SAFETY: isatty only inspects a descriptor number.
```

### src/runtime.rs, original line 524

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/src/runtime.rs#L524)

```text
SAFETY: one valid pollfd, no wait.
```

### src/runtime.rs, original line 542

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/src/runtime.rs#L542)

```text
SAFETY: plain queries on an open descriptor; `tcsetattr` gets the
attributes `tcgetattr` just filled in.
```

### tests/terminal_suspend.rs, original line 93

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L93)

```text
SAFETY: plain libc calls on the helper's own process and descriptors.
tuiscotti already made it a session leader with the PTY as its
controlling terminal; repeating both is harmless and keeps the helper
a shell shape on its own.
```

### tests/terminal_suspend.rs, original line 103

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L103)

```text
SAFETY: ttyname returns a static buffer or null.
```

### tests/terminal_suspend.rs, original line 106

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L106)

```text
SAFETY: non-null, NUL-terminated per ttyname's contract.
```

### tests/terminal_suspend.rs, original line 120

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L120)

```text
SAFETY: setpgid is async-signal-safe.
```

### tests/terminal_suspend.rs, original line 131

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L131)

```text
SAFETY: both sides set the group so the foreground hand-off is not racy.
```

### tests/terminal_suspend.rs, original line 139

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L139)

```text
SAFETY: waiting on the tracked job.
```

### tests/terminal_suspend.rs, original line 146

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L146)

```text
SAFETY: the helper takes the terminal back like a shell.
```

### tests/terminal_suspend.rs, original line 157

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L157)

```text
SAFETY: foreground back to the job, then continue it.
```

### tests/terminal_suspend.rs, original line 250

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L250)

```text
SAFETY: zeroed storage of the platform layout, filled by libc.
```

### tests/terminal_suspend.rs, original line 276

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L276)

```text
SAFETY: a plain-old-data C struct viewed as its bytes.
```

### tests/terminal_suspend.rs, original line 367

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L367)

```text
SAFETY: signalling the job whose pid the helper reported.
```

### tests/terminal_suspend.rs, original line 471

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/tests/terminal_suspend.rs#L471)

```text
SAFETY: probing a pid that was never this process's child.
```

### xtask/src/main.rs, original line 1274

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d9e569bf6fd33d90266db597c458442c729340ad/xtask/src/main.rs#L1274)

```text
SAFETY: single-threaded test process under nextest.
```

## Remaining work

Keep each rationale synchronized with its operation. Any future change to an unsafe operation requires a fresh safety review and applicable verification; this comment-only cleanup does not authorize such changes.

## Verification

Compare the original source context and current operation. Preserve compiler safety settings and existing tests. The retained text documents the prior justification; it does not substitute for inspecting changed code.
