# Retired host-bootstrap qualification evidence

> Historical, non-executable evidence. This synthetic host lifecycle was
> rejected for the refactoring campaign. Do not dispatch it, recreate its
> runtime layout, or treat its receipts as campaign authority.

## Why this record remains

The companion driver, observer, vectors, and synthetic fixture recorded a
bounded experiment about independent execution, protected inputs, and
observer-owned evidence. Those findings remain useful for security review.
They do not define the current campaign executor and cannot authorize a
production task.

The current model is [campaign-policy.md](../campaign-policy.md):
isolated implementer, verifier, and reviewer subagents operate on host-local
paths. Standalone latest taskfmt is used only for the per-task lint and verify
checks. The coordinator reviews evidence and integrates accepted commits.

## Preserved synthetic findings

- Candidate-written reports were not accepted as proof that a process ran.
- Protected inputs were snapshotted before candidate execution.
- Observer-owned process records, exit status, stdout/stderr, and output files
  were compared against the submitted result.
- Zero-worker, forged-result, replay, mutation-and-restore, and malformed
  protocol cases were rejection cases.
- The synthetic fixture had no application baseline, oracle receipt, or
  production integration authority.

The observer boundary and exact JSON schemas remain useful inputs to future
security review. They are not an instruction to build a host daemon, install a
host binary, or expose lifecycle operations.

## Historical scope

The old fixture modeled preparation, freezing, verification, sealing, and
integration as one host-owned lifecycle. That model conflicts with the current
rule that subagents own implementation, validation, and review while the
coordinator only schedules, reviews, and serially integrates.

The old record also mixed administrative progress generation with verification.
Current taskfmt policy deliberately has no progress or lifecycle authority:
the verifier subagent records its own evidence directory and runs only the
task-local lint and verify commands.

## Evidence limits

This fixture never proved a real production harness, application provenance,
visual parity, or a safe campaign integration. Its synthetic receipts,
campaign records, and acceptance JSON are historical test data. They must not
be copied into the ledger, used as predecessor receipts, or cited as a
successful task result.

The companion files remain frozen archival inputs for historical reproducibility
only. Any future requalification requires a separately authorized design that
follows the current no-container, subagent-only policy and latest-taskfmt
contract.
