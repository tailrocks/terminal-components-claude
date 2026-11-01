# Refactoring task bundles retired (2026-11-01)

The per-task execution bundles under `refactoring-tasks/` (73 tasks,
575 files, ~68 MiB incl. a 61 MiB pre-refactor source tarball in task
072) and the generated planning corpus under `docs/refactoring-plan/`
(134 files of TSV/JSON ledgers, bootstrap drivers and captured logs)
were deleted. They were process debris, not documentation: all 73
tasks were still `pending`, their `writable_paths` referenced a
`crates/`/`apps/`/`tools/` layout that does not exist in this tree,
and nothing in code, tests or CI referenced them.

What was worth keeping is distilled into
[architecture](../architecture.md): the target crate structure, the
ownership contracts, the wave plan (B/C/S/H/J/T/X with task keys) and
the migration gates. The verifier model from the bundles — exact
comparison against a frozen oracle, writable/forbidden path
discipline, hash-manifested trusted inputs — survives as the design
principle behind the functional test estate in
[functional testing](../functional-testing.md).

Nothing else in the bundles was preservable: the per-task
`AGENTS.md` files were duplicated boilerplate, and the `trusted/`
TSVs were copies of the deleted planning ledgers. If the refactoring
programme is re-planned, it starts from
[architecture](../architecture.md), not from these bundles.
