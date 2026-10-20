# Handoff

## TextSection extraction

- Branch: `parity/shared-components-20261020`
- Base: `74cc40322e11d7af59d2124308d8fd1036aa464b`
- State: reviewed extraction changes remain uncommitted and unpushed; parent
  coordinator owns checkpoint and integration.
- Item: finish integrating reusable `termrock_feedback::TextSection` for
  holla's Files, Review, Snapshot and the showcase's existing sidebar title.
- Owned paths: `crates/termrock-feedback/src/section.rs`,
  `crates/termrock-feedback/src/lib.rs`, `crates/termrock/src/lib.rs`,
  `crates/holla-ui/src/screens/{files.rs,review.rs,snapshot.rs}`,
  `crates/showcase-ui/src/app.rs`,
  `docs/audits/2026-10-20-text-section.md`.
- Checks: focused `cargo nextest run -p termrock-feedback section::tests`
  passed 4/4; `cargo check -p termrock-feedback -p holla-ui -p showcase-ui`
  passed; changed-file `rustfmt --check` and `git diff --check` passed.
  Earlier targeted application nextest run passed 328/329. The remaining
  `holla_visual_baseline` failure was reproduced with the original Holla screen
  files restored: `parity-discovery` digest expected
  `08e346d897de21b6`, got `491485d3b0a74eb1`. Repository-wide formatting still
  reports 16 unchanged visual-harness files.
- Parity status: incomplete. Paired tuiscotti scenario
  `holla::parity::files::120x40::truecolor` (`initialHome`) differs in 1,226
  cells. It launches HOME with `--scenario parity-files` and sends no inputs,
  so this is a Home/cataloguefixture frame, not the Files browser or evidence
  about `TextSection`/`screens/files.rs`. Cleanup-flow candidate capture is
  absent. Evidence: `/tmp/termrock-paired-proof-20261020/`.
- Next steps: wait for the coordinator's priority work on identical CLI tests
  across both branches. Then integrate this work onto the latest PR head,
  resolve conflicts only within owned paths, rerun focused nextest and checks,
  rerun paired Holla captures, and have the coordinator review and checkpoint
  the integrated result. Do not declare this extraction or full application
  parity complete until the paired captures pass.
