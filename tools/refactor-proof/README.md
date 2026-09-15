# refactor-proof

TASK-001 deliverable: independently qualified `tc-proof` comparator and `tc-proof-host` core for product `qualified-harness`.

Command interface and ownership boundaries are defined in [`docs/refactoring-plan/proof-contract.md`](../../docs/refactoring-plan/proof-contract.md). This crate does not constitute qualification evidence until CHK-004/005/006/007 pass against the pinned bootstrap drivers.

Build:

```sh
cargo build -p refactor-proof
```

Installed entrypoints for qualification:

- `bin/tc-proof` — wrapper to the workspace-built comparator
- `bin/tc-proof-host` — wrapper to the workspace-built host
