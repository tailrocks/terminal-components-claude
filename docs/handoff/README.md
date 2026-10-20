# Migration handoff

Start with the current [Termrock migration and acceptance matrix](../architecture/termrock-migration-matrix.md). It records fixed source identities, application status, paired CLI coverage, architecture findings, unresolved differences, and next evidence. Treat every partial or missing result as incomplete work.

## Current handoff — 2026-10-20

The active priority is the CI-only workflow upgrade across `main`,
`parity/termrock-visual-baseline`, and `design/interaction-polish`. The
Termrock product refactor is paused while CI branches and required gates are
verified. See the matrix's [latest CI comparison](../architecture/termrock-migration-matrix.md#latest-ci-branch-comparison-2026-10-20)
for exact source tips and blockers.

Latest stable Velnor Actions is `0.1.29` at `e7b2f088715379c22be473fab896a43d50cc16fc`.
Parity tip `1d7b8eb291e37cb4806ce1369a8a0abab08a81ac` and design tip
`f2f6eb716c4a512be3dc9423742fa06e86879b32` already contain workflows generated
with the current 15-minute Rust job timeout. Parity generation, plan (47 crates,
44 jobs, 259 tasks) and Actionlint pass locally; remote run
[39445641197](https://github.com/tailrocks/terminal-components-claude/actions/runs/39445641197)
was in progress at observation and is not a green result. Design local checks
pass, but latest run
[39438273472](https://github.com/tailrocks/terminal-components-claude/actions/runs/39438273472)
failed Plan after five minutes and skipped two Rust jobs. Main tip
`a97e351df736b576904bc450619ca347ea64d65b` has not been upgraded; CI-only PR #39
is at `a6929aeba7479d0b3a746f69d412a1ab701b5242`. The required Plan job remains
blocked; run
[39436987238](https://github.com/tailrocks/terminal-components-claude/actions/runs/39436987238)
has Plan blocked and Required failed. Main PR #39 local checks pass with 43 jobs,
but remote required checks do not. Current Velnor main observation
`16c…` provides no Plan timeout override. Do not work around this by inventing
a config option. Fresh branch verifiers must provide exact gate results before
any status is called green.

The frozen product references remain unchanged: baseline tag object
`2718a6b8127a522a3d351d0b4bdb7542fc8da542`, baseline source
`4169783ae883788186d161af09ba9c481bd84e5c`, and tuiscotti
`a1ea27f757d75d07b05499276cd932e9f8e067ad`. All four application parity and
library acceptance states remain **incomplete**.

The refactor is paused at these published worker tips; resume by reviewing or
integrating the listed work, not by repeating its implementation:

| Worker ref | State at handoff | Next action |
| --- | --- | --- |
| `parity/runtime-fields-integration-20261020` — `ffc3b25efb4879a598e180d924daa5d37d721aab` | Runtime fields merged; review pending | Independent review, then integrate only after review clears. |
| `parity/shared-style-integration-20261020` — `e72dbb15369d541ee86095f6caaee3f7e4680828` | Shared-style review cleared | Keep the review receipt with the integration. |
| `parity/picker-query-component-20261020` — `fe158fb56be6bffb30309521b5b71f778f26f39f` | Picker review cleared; binding/runtime dependency still needs prepared commits `98900f3e11e4cd34835dec5902694ed260125aee` and `801e6c94892703bde5309e3daa31f37c3a727910` | Integrate the dependencies in order and re-run picker checks. |
| `parity/session-cursor-prepared-port-20261020` — `5ca3c4f205aed4719d6558f2bbe9829a6321bdae` | Session port prepared and cleared | Integrate after checking its dependency order. |
| `parity/note-reference-color-20261020` — `c0c0f148b54c3459ee157de19dd49ca84bb3b447` | Test-only reference change cleared | Preserve test-only boundary; integrate with its receipt. |
| `parity/composition-fallible-20261020` — `8cecb53b26470f7de6edfe6bbc95b25711cf1570` | Fallible composition facade cleared | Retain for the reviewed-layout integration chain. |
| `parity/composition-plan-final-20261020` — `6e81c0034989b11321e64cc2da0bb617c37b896c` | Final resolved layout tree; review pending | Obtain independent review before integration. |
| `parity/dialog-layout-contract-20261020` — `86226a2442f37621bbef91dd50b9b7bc3104659b` | Dialog work pending review, paired parity, and lifecycle checks | Complete the review and runtime lifecycle evidence. |
| `parity/theme-palette-baseline-20261020` — `737ecc836fc40d77b51d797bda9c3927b12eeee7` | Palette work in progress; 8/9 checks pass, mono remains red | Fix the mono failure and rerun the complete palette set. |
| `parity/binding-runtime-20261020` — `98900f3e11e4cd34835dec5902694ed260125aee` | Bounded binding identity correction independently cleared | Integration remains pending. |
| `parity/showcase-button-consumer-20261020` — `60138e17c6a16814039fd3cd84074b5ab0f4b895` | Bounded button change cleared; full Buttons comparison remains red by 2,982 cells | Preserve the red evidence; resolve the source difference at `0f181f05` and rerun the full Buttons suite. |

The parent also has a local reviewed-contract integration commit
`8462bf28aadabd76833e11395e6237e66468efe0`: native checks are 153 passed, but
paired product comparison is red. Preserve it unchanged at
`wip/termrock-reviewed-contract-integration-20261020`; do not integrate without
independent merged review and resolution of the red paired proof. This handoff is recorded on
`wip/ci-upgrade-handoff-20261020` and must receive independent review before
its documentation is integrated.
