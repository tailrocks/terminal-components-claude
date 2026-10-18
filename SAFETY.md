# SAFETY

## Status

Retained safety rationales for unchanged unsafe operations. Removing Rust comments does not remove these obligations. This record is not a new proof that the incomplete product refactor is ready.

## Authority

Preserve existing validation and safety contracts. `main` supplies architectural direction; `design/interaction-polish` supplies product behavior. No unsafe operation or safety lint is changed by this documentation migration.

## Current evidence

Rationales below came from Rust comments at `d1122f69e6b9c2364aa6ede8174e7d35b8cfff39`. Comment removal and reviewed formatting preserve their operations. Original line links retain the exact source context; current source line numbers may differ.

### crates/holla/tests/holla_pty.rs, original line 67

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/holla/tests/holla_pty.rs#L67)

```text
SAFETY: valid out-pointers; the winsize outlives the call.
```

### crates/holla/tests/holla_pty.rs, original line 78

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/holla/tests/holla_pty.rs#L78)

```text
SAFETY: freshly created descriptors owned here.
```

### crates/holla/tests/holla_pty.rs, original line 103

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/holla/tests/holla_pty.rs#L103)

```text
SAFETY: only async-signal-safe calls run between fork and exec.
```

### crates/holla/tests/holla_pty.rs, original line 113

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/holla/tests/holla_pty.rs#L113)

```text
SAFETY: master is an owned open descriptor.
```

### crates/termrock-test-support/src/perf.rs, original line 65

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-test-support/src/perf.rs#L65)

```text
SAFETY: every method forwards to `System` unchanged after bumping a
thread-local counter; the counter access cannot allocate or panic, so the
`GlobalAlloc` contract of `System` is preserved verbatim.
```

### crates/termrock-test-support/src/perf.rs, original line 71

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-test-support/src/perf.rs#L71)

```text
SAFETY: same layout, forwarded to the system allocator.
```

### crates/termrock-test-support/src/perf.rs, original line 76

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-test-support/src/perf.rs#L76)

```text
SAFETY: `p` was returned by `System.alloc` with this layout.
```

### crates/termrock-test-support/src/perf.rs, original line 82

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-test-support/src/perf.rs#L82)

```text
SAFETY: same layout, forwarded to the system allocator.
```

### crates/termrock-test-support/src/perf.rs, original line 88

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-test-support/src/perf.rs#L88)

```text
SAFETY: `p` was returned by `System.alloc` with layout `l`.
```

### crates/termrock-visibility-tests/src/bin/capture-command.rs, original line 478

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/bin/capture-command.rs#L478)

```text
SAFETY: the signal set is writable storage initialized by libc.
```

### crates/termrock-visibility-tests/src/bin/capture-command.rs, original line 483

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/bin/capture-command.rs#L483)

```text
SAFETY: `action` contains a plain atomic-only handler and a valid mask.
```

### crates/termrock-visibility-tests/src/bin/fixture.rs, original line 565

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/bin/fixture.rs#L565)

```text
SAFETY: getpid has no preconditions; the validated SIGKILL targets this process.
```

### crates/termrock-visibility-tests/src/bin/fixture.rs, original line 567

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/bin/fixture.rs#L567)

```text
SAFETY: `pid` is this process and the parser permits only SIGKILL.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 651

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L651)

```text
SAFETY: fcntl only inspects and updates flags for this open pipe descriptor.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 656

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L656)

```text
SAFETY: `fd` remains open and the command accepts a bitmask of file status flags.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 707

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L707)

```text
SAFETY: `child` is this process's live, unreaped child and `info` is
writable storage for waitid's result. WNOWAIT preserves its PID.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 718

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L718)

```text
SAFETY: waitid initialized the siginfo result for this call.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 749

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L749)

```text
SAFETY: the child is still unreaped and owns this private process group.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 1039

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L1039)

```text
SAFETY: geteuid has no preconditions and returns the current process UID.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 1047

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L1047)

```text
SAFETY: `file` keeps this valid descriptor open for the duration of the call.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 1086

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L1086)

```text
SAFETY: `root_name` is a valid NUL-terminated path and flags request a
directory descriptor without following a symlink.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 1092

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L1092)

```text
SAFETY: `root_fd` is a newly opened descriptor now owned by this File.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 1107

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L1107)

```text
SAFETY: the parent descriptor is open and name is a single path
component. O_NOFOLLOW and O_DIRECTORY reject symlink/non-directory
ancestors at every step.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 1114

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L1114)

```text
SAFETY: `child_fd` is a newly opened descriptor now owned by this File.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 1128

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L1128)

```text
SAFETY: parent is a live directory descriptor and name is one component.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 1136

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L1136)

```text
SAFETY: parent is open; O_NOFOLLOW and O_DIRECTORY reject symlinks.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 1141

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L1141)

```text
SAFETY: `directory_fd` is now owned by this File.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 1164

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L1164)

```text
SAFETY: parent is an open checked directory; O_NOFOLLOW protects the
final lock-file component from symlink substitution.
```

### crates/termrock-visibility-tests/src/lib.rs, original line 1170

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/crates/termrock-visibility-tests/src/lib.rs#L1170)

```text
SAFETY: `file_fd` is now owned by this File.
```

### tools/refactor-proof/src/observer/client.rs, original line 177

[Original source](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69e6b9c2364aa6ede8174e7d35b8cfff39/tools/refactor-proof/src/observer/client.rs#L177)

```text
SAFETY: inherited anonymous pipe ends are owned by this host process for the fixture lifetime.
```

## Remaining work

Keep each rationale synchronized with its operation. Any future change to an unsafe operation requires a fresh safety review and applicable verification; this comment-only cleanup does not authorize such changes.

## Verification

Compare the original source context and current operation. Preserve compiler safety settings and existing tests. The retained text documents the prior justification; it does not substitute for inspecting changed code.
