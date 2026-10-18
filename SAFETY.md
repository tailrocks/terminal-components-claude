# Safety rationales

These records preserve the original justifications for unsafe operations. They are maintenance obligations, not a new safety audit or evidence of product readiness. Compiler safety settings and existing tests remain authoritative constraints.

## Original rationales

The excerpts below identify their exact source context at `c1a49af0f6ade36838c6330ce8a034e8d589ea79`. Inspect the current operation before relying on a rationale; source line numbers may change.

### docs/refactoring-plan/evidence/runner-bootstrap-performance.rs, original line 12

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/docs/refactoring-plan/evidence/runner-bootstrap-performance.rs#L12)

```text
SAFETY: This wrapper preserves System's pointer/layout contract and performs
only allocation-free atomic increments before forwarding each allocation.
```

### docs/refactoring-plan/evidence/runner-bootstrap-performance.rs, original line 17

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/docs/refactoring-plan/evidence/runner-bootstrap-performance.rs#L17)

```text
SAFETY: GlobalAlloc supplies a valid layout; System owns allocation.
```

### docs/refactoring-plan/evidence/runner-bootstrap-performance.rs, original line 21

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/docs/refactoring-plan/evidence/runner-bootstrap-performance.rs#L21)

```text
SAFETY: The caller provides the original System pointer and layout.
```

### refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 12

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L12)

```text
SAFETY: This wrapper preserves System's pointer/layout contract and performs
only allocation-free atomic increments before forwarding each allocation.
```

### refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 17

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L17)

```text
SAFETY: GlobalAlloc supplies a valid layout; System owns allocation.
```

### refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 21

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/refactoring-tasks/terminal-components/completion/070/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L21)

```text
SAFETY: The caller provides the original System pointer and layout.
```

### refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 12

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L12)

```text
SAFETY: This wrapper preserves System's pointer/layout contract and performs
only allocation-free atomic increments before forwarding each allocation.
```

### refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 17

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L17)

```text
SAFETY: GlobalAlloc supplies a valid layout; System owns allocation.
```

### refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 21

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/refactoring-tasks/terminal-components/completion/071/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L21)

```text
SAFETY: The caller provides the original System pointer and layout.
```

### refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 12

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L12)

```text
SAFETY: This wrapper preserves System's pointer/layout contract and performs
only allocation-free atomic increments before forwarding each allocation.
```

### refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 17

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L17)

```text
SAFETY: GlobalAlloc supplies a valid layout; System owns allocation.
```

### refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs, original line 21

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/refactoring-tasks/terminal-components/completion/072/trusted/runner-bootstrap/runner-bootstrap-performance.rs#L21)

```text
SAFETY: The caller provides the original System pointer and layout.
```

### src/runtime.rs, original line 375

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/src/runtime.rs#L375)

```text
SAFETY: installing a handler that performs one atomic store.
```

### src/runtime.rs, original line 390

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/src/runtime.rs#L390)

```text
SAFETY: default disposition then raise; both are signal-safe libc
calls made from ordinary (non-handler) context.
```

### src/runtime.rs, original line 501

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/src/runtime.rs#L501)

```text
SAFETY: isatty only inspects a descriptor number.
```

### src/runtime.rs, original line 519

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/src/runtime.rs#L519)

```text
SAFETY: one valid pollfd, no wait.
```

### src/runtime.rs, original line 537

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/src/runtime.rs#L537)

```text
SAFETY: plain queries on an open descriptor; `tcsetattr` gets the
attributes `tcgetattr` just filled in.
```

### tests/terminal_suspend.rs, original line 93

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/tests/terminal_suspend.rs#L93)

```text
SAFETY: plain libc calls on the helper's own process and descriptors.
tuiscotti already made it a session leader with the PTY as its
controlling terminal; repeating both is harmless and keeps the helper
a shell shape on its own.
```

### tests/terminal_suspend.rs, original line 103

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/tests/terminal_suspend.rs#L103)

```text
SAFETY: ttyname returns a static buffer or null.
```

### tests/terminal_suspend.rs, original line 106

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/tests/terminal_suspend.rs#L106)

```text
SAFETY: non-null, NUL-terminated per ttyname's contract.
```

### tests/terminal_suspend.rs, original line 120

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/tests/terminal_suspend.rs#L120)

```text
SAFETY: setpgid is async-signal-safe.
```

### tests/terminal_suspend.rs, original line 131

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/tests/terminal_suspend.rs#L131)

```text
SAFETY: both sides set the group so the foreground hand-off is not racy.
```

### tests/terminal_suspend.rs, original line 139

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/tests/terminal_suspend.rs#L139)

```text
SAFETY: waiting on the tracked job.
```

### tests/terminal_suspend.rs, original line 146

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/tests/terminal_suspend.rs#L146)

```text
SAFETY: the helper takes the terminal back like a shell.
```

### tests/terminal_suspend.rs, original line 157

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/tests/terminal_suspend.rs#L157)

```text
SAFETY: foreground back to the job, then continue it.
```

### tests/terminal_suspend.rs, original line 250

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/tests/terminal_suspend.rs#L250)

```text
SAFETY: zeroed storage of the platform layout, filled by libc.
```

### tests/terminal_suspend.rs, original line 276

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/tests/terminal_suspend.rs#L276)

```text
SAFETY: a plain-old-data C struct viewed as its bytes.
```

### tests/terminal_suspend.rs, original line 367

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/tests/terminal_suspend.rs#L367)

```text
SAFETY: signalling the job whose pid the helper reported.
```

### tests/terminal_suspend.rs, original line 471

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/tests/terminal_suspend.rs#L471)

```text
SAFETY: probing a pid that was never this process's child.
```

### xtask/src/main.rs, original line 1274

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/c1a49af0f6ade36838c6330ce8a034e8d589ea79/xtask/src/main.rs#L1274)

```text
SAFETY: single-threaded test process under nextest.
```

## Maintenance

Keep each rationale synchronized with its operation. Changes to unsafe operations require a fresh safety review and applicable verification. This documentation does not authorize changes to unsafe code or safety lints.
