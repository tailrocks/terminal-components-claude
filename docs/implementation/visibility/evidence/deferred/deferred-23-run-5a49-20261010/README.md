# VIS-04 deferred controls: exact 5a49 run evidence

This archive records one exact 23-test `termrock-conformance::control_states` run against source commit `5a49b6922b896460b2e9f0ed38cf9ce3d6aa088e`, tree `56f1a8483421ede75a959e14a8f97fcf39fc53f3`. The selected source archive had SHA-256 `9e3146671332ffc8c7e85adfd0b4ab9ba0e569c72564fb94dedde3bb796ae70e`; its file identity manifest is preserved here. The large source archive and source tree are intentionally not copied into this evidence package.

## Result

The exact list command exited 0 and matched 23 registered tests: 22 normal registrations and one ignored registration, BD-21 `w13_filter_wide_trail_cells_clear`. The ignored case was explicitly run with `--run-ignored all`; no expected-failure treatment was applied. The actual run produced 23 unique test starts, 22 passes, one failure, and zero incomplete outcomes. Nextest exited 100 and the wrapper exited 1. There was no signal, timeout, or output-cap event.

The real failure is `w13_filter_wide_trail_cells_clear` (`crates/termrock-conformance/tests/control_states.rs:6137`): the W13-05 assertion found an earlier-paint glyph remaining after repainting `旧日本`; observed cells included `旧, 界, 日, y, 本`. This is a failed assertion, not a harness failure, and is not reclassified as expected or ignored. The repository assertion was not changed.

## Earlier attempts

- The first native attempt exited 95 before tests started because Nextest required `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1`; zero tests executed.
- A subsequent wrapper attempt failed before spawning a child because it read `plan.tools.nextest` although the plan uses `plan.tools.cargo_nextest`; zero tests executed and no output root was created.
- The corrected list then verified the 23-name selection. The actual run used the exact same selection and re-used the list-built target.

The initial failure receipts, the pre-spawn observation, the list output and selection receipt, both run-plan revisions, runner revisions, and independent reviews are included below. Each file is hashed in `MANIFEST.json` and `SHA256SUMS`.

## Limits

This is a single selected-control run. It is not a full-suite, product-wide, or renderer qualification. The Rust source inputs were matched against the source manifest for 500 files before and after the run. Cargo operated with `CARGO_NET_OFFLINE=true`, but the Cargo home/cache was a mutable candidate and was not rehashed; no cache immutability or complete cache-closure claim is made. Direct-child close was recorded; there is no separate post-kill process-group disappearance poll in the receipt. The source manifest refers to files outside this archive and must not be read as evidence those source files are included here.

## Archive contents

- `provenance/`: request, source manifest, initial and retry run plans, and retry addendum.
- `selection/`: the exact 23 expected names.
- `attempts/00-initial-startup-failure/`: the exit-95 receipt and raw stderr.
- `attempts/01-runner-pre-spawn-failure/`: pre-spawn observation and the failing R2 runner source.
- `attempts/02-list/`: raw list output/stderr, executor receipt, and verified selection.
- `attempts/03-actual-run/`: corrected runner, raw Nextest JSONL/stderr, result receipt, and run summary.
- `reviews/`: independent retry-plan, runner-delta, and actual-result reviews.

No source, queue, or product files are included. No product assertion is altered by this evidence archive.
