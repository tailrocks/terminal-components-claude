# TEST-SCOPE CHECK

## Purpose

This check stops changes that are not test changes.

It compares the worktree against a base ref. It fails on
any change outside the test scope. It checks paths and
file sections.

## Scope

These paths pass as whole files:

- `tests/*`
- `snapshots/tuiscotti/*`
- `baselines/tuiscotti-v1/*`
- `docs/testing/*`
- `scripts/test-scope-check.sh`
- `crates/*/tests/*`
- `crates/termrock-test-support/*` (test tooling crate:
  harness, digest, conformance driver, perf allocator)

These paths pass only in test sections:

- `src/*.rs`, `crates/*.rs`: only hunks inside
  `#[cfg(test)]` modules.
- `Cargo.toml`: only `[dev-dependencies]`, `*test*`, and
  `*tuiscotti*` sections. The prod dep graph must not change.
- `Cargo.lock`: only dev-only package blocks. The prod dep
  graph must not change.
- `.github/workflows/*.yml`: only inside jobs with `test`
  in the job id.

All other paths fail.

## Rules

- Empty lines do not count. Comment lines count.
- Renames count as delete plus add.
- New workflow files must hold only test jobs.
- Cargo checks fail closed when cargo cannot run.

## How to run the check

1. Go to the repo root.
2. Run `scripts/test-scope-check.sh`.
3. Read the `PASS` and `FAIL` lines.

To check against another ref, run
`scripts/test-scope-check.sh --base REF`.
The default base is tag `visual-baseline`, else `HEAD`.

## Exit codes

- `0`: all changes are test-only.
- `1`: scope violation.
- `2`: usage or tool error.

## Self test

Run `scripts/test-scope-check.sh --self-test`.
It builds a small repo. It checks pass cases and
fail cases. All cases must pass.

## Limits

- Out-of-line test modules (`#[cfg(test)] mod foo;`)
  do not count. Use inline test modules.
- `#[test]` functions must sit inside `#[cfg(test)]`
  modules. Bare `#[test]` functions fail.
- A package that is both a prod dep and a dev dep
  counts as a prod dep.
