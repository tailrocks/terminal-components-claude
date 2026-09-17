# Historical refactoring planning progress

> **Superseded on 2026-09-18.** This historical progress ledger is retained
> for provenance only. It is not an execution prompt, receipt, or readiness
> approval.

Read [`execution-readiness-report.md`](execution-readiness-report.md), the sole
current readiness authority. Its current verdict is **NO-GO**.

Current policy is host-local and subagent-only:

- delegated implementer, verifier, and reviewer subagents own task work;
- no container, Docker, Podman, image, mount, firmlink, or container runtime;
- standalone latest taskfmt only performs per-task `lint` and `verify`;
- historical host lifecycle, dispatcher, promotion, and taskfmt lifecycle
  commands are retired and must not be replayed.

Use [`README.md`](README.md) for current navigation and
[`subagent-only-policy.md`](subagent-only-policy.md) for the binding execution
rules. Historical reports below this directory are evidence, not authority.
