# Jackin mono probe correction, 2026-10-20

The native `historical_failure_and_action_labels_remain_readable_in_mono` regression probed the wrong row for the capsule failure label. Clean archived commit `3600c0eeb0596ad61a5eef3bcb6ddab5a8020150` reproduces the same failure: the assertion expected `F` at `(103,8)`, where the real frozen application renders `o`.

Independent real CLI captures through the pinned Tuiscotti harness show `FAILED` beginning at `(103,7)` in both frame 0 and paused frame 40. The immutable baseline binary SHA-256 is `8fccb21faa2459d4b22e5e527f7c1cc29eff64cd1df025957aaed92da35c2a2e`. Capture provenance and canonical ANSI artifacts are retained at `/Users/donbeave/Projects/termrock-cli-evidence-20261020/jackin-mono-baseline/termrock-mono-real-baseline-20261020/{record.json,frozen40.ansi,frozen0.ansi}`. The frozen source is `4169783ae883788186d161af09ba9c481bd84e5c`; the Tuiscotti revision is `a1ea27f757d75d07b05499276cd932e9f8e067ad`.

The native assertion now uses the real baseline coordinate and additionally checks every glyph of `FAILED` and foreground/background readability in mono. The other existing probes remain. The focused Nextest test passed, and independent review verified both captured coordinates and the retained assertions. Product rendering, frozen snapshots, and the shared frozen comparison harness are unchanged by this test correction.

The remaining production capsule frame replay is an independent architecture violation. Correcting this assertion does not establish application parity or architecture completion.
