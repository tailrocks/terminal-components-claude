# Visibility status renderer

`tools/visibility/status.py` renders the repository's source-pinned visibility observations from `source-facts.json`, task records, and queue helpers. It writes a root `STATUS.md`; generated output is an observation artifact, not current product authority or proof of readiness.

Run `python3 tools/visibility/status.py --help` for supported options. `--write` regenerates the root report, `--check` compares it with the report derived from current inputs, and `--role candidate|reference` selects the local presentation role. Do not regenerate the retired status report as part of routine project documentation work.

The black-box CLI contract is exercised by `crates/termrock-visibility-tests/tests/status.rs`; queue operations are covered in `queue_v2.rs`. Run the package tests with `cargo nextest run --manifest-path crates/termrock-visibility-tests/Cargo.toml --locked`. These tests validate report/schema and command behavior, not application parity.

The report links this page as the operational source note. Keep this path available because generated candidate/reference reports link to it. Product architecture, design acceptance, and remaining work are defined by the root [architecture](../../docs/architecture.md), [design](../../docs/design.md), and [plan](../../docs/plan.md).
