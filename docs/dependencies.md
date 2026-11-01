# Testing dependency compatibility

Reviewed 2026-11-01 for the functional-testing addition to
`design/interaction-polish`. These notes concern the testing stack, not a
complete dependency freshness audit.

| Dependency | Pin | Reason / follow-up |
| --- | --- | --- |
| Existing `tuiscotti` | `a1ea27f757d75d07b05499276cd932e9f8e067ad` | Held below current main because the frozen visual harness depends on the removed live `Tui`/`Session`, `Observation`, and grouped-store APIs. Current main `d8d54155947f6303a41dfd6079b53c0abc16f29a` is tuiscotti 0.4.0 and is offline only. Migrate callers and verify reference compatibility deliberately; do not repin and mass-approve the frozen oracle. |
| New live driver, Rust crate `tui_test` | Registry `tuiscotti-tui-test = "=0.2.1"` | The user-requested fork supplies checked mouse input, native isolated launches, raw bytes, bounded waits, and owned snapshots needed by executable functional tests. This is a published dependency, with no vendoring or local patch. |
| Rust / Nextest | `1.99.1` / `0.9.148` | Existing repository pins retained. Both are used for the new suite. |
| Velnor Actions | `0.1.29` | Existing generator pin retained; generated functional CI is validated with the pinned actionlint, shellcheck and zizmor. No manual generated-file edits. |

The original instruction describing the newest tuiscotti as the PTY driver
predates its offline migration. The live fork and existing frozen visual
dependency coexist temporarily. The full rationale, source links, supported
test boundary, and migration sequence are in
[Functional testing](functional-testing.md).
