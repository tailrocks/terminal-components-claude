---
schema: task/v5
id: TASK-901
title: "Transform the independent qualification payload"
kind: bugfix
---

# TASK-901 — Transform the independent qualification payload

## Goal

The candidate payload contains exactly `qualified\n` while the protected sentinel remains unchanged.

## Context

This is a planner-owned synthetic qualification task, never a terminal-components implementation task. An independent checker reads the payload bytes. The initial repository contains `pending\n`.

## Preconditions

- **P-001:** The host supplies a fresh synthetic Git repository and immutable checker.

## Scope

In scope:

- `src/payload.txt` and synthetic candidate worker probes under `src/`.

Out of scope:

- The protected sentinel, task contract, checker, host receipts, expected bytes, and repository refs.

## Requirements

- **R-001 (MUST):** Preserve the protected sentinel bytes.
- **R-002 (MUST):** Set the payload bytes to `qualified\n`.
- **R-003 (MUST NOT):** Modify protected verification inputs.
- **R-004 (MUST):** Execute the complete independent check set.
- **R-005 (MUST):** Pass the completion gate.

## Acceptance criteria

### AC-001 — Sentinel survives
```gherkin
Given the synthetic repository contains the protected sentinel
When the independent checker reads the sentinel
Then its bytes equal the immutable sentinel value
```

**Verification**

- **Type:** invariant
- **Covers:** `R-001`
- **Check:** `CHK-001`

### AC-002 — Payload transforms
```gherkin
Given the payload initially contains pending followed by a newline
When the candidate transformation completes
Then the payload contains qualified followed by a newline
```

**Verification**

- **Type:** scenario
- **Covers:** `R-002`
- **Check:** `CHK-002`

### AC-003 — Protected inputs survive
```gherkin
Given the checker and sentinel are protected inputs
When the scope gate examines the candidate changes
Then only paths under src have changed
```

**Verification**

- **Type:** invariant
- **Covers:** `R-003`
- **Check:** `CHK-003`

### AC-004 — Complete checks run
```gherkin
Given both the payload and sentinel are required outputs
When the independent checker validates the repository
Then both required byte comparisons pass
```

**Verification**

- **Type:** scenario
- **Covers:** `R-004`
- **Check:** `CHK-004`

### AC-005 — Completion gate passes

**Verification**

- **Type:** gate
- **Check:** `CHK-005`

## Fixed decisions

- **D-001:** Expected payload bytes originate in the planner fixture. Candidate output never updates them.
- **D-002:** This fixture tests verification and isolation; it contains no production implementation.

## Checklist

<!-- checklist:start -->
- [ ] **1** Prepare.
    - [ ] **1.1** Confirm the sentinel. (`R-001`, `AC-001`, `CHK-001`)
- [ ] **2** Transform.
    - [ ] **2.1** Set the payload. (`R-002`, `AC-002`, `CHK-002`)
    - [ ] **2.2** Preserve protected inputs. (`R-003`, `AC-003`, `CHK-003`)
    - [ ] **2.3** Check both outputs. (`R-004`, `AC-004`, `CHK-004`)
- [ ] **3** Verify.
    - [ ] **3.1** Run the completion gate. (`R-005`, `AC-005`, `CHK-005`)
<!-- checklist:end -->
