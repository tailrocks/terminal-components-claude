# Paired CLI launch receipts — 2026-10-21

This record covers the launch-identity and child-cleanup increment merged into the PR #38 branch. It does not admit a replacement baseline build or establish full application parity.

## Scope and decisions

- Delivery base: `parity/termrock-visual-baseline` at `df30dc013aa6d78df6f9eb58fc67f3f3d61be37f`.
- Frozen source remains commit `4169783ae883788186d161af09ba9c481bd84e5c`, tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`; Tuiscotti remains pinned at `a1ea27f757d75d07b05499276cd932e9f8e067ad`.
- Reviewed implementation commit: `d8fd898fc1b12f65f2072bb8bfe362b4cd20ea58` (based on `3fd30023b9983943c531d418977ab8016f5a9f22`); independent review cleared this exact commit. It is integrated by merge commit `fc616e0e351e874ff2805a3ab870a89fd2607b52`.
- The adapter now records each Tuiscotti child’s expected binary identity and OS-observed image, PID start identity, ordered argv/environment HMACs, and successful close/reap receipt. The receipt key is removed from the child environment. Missing launch or close receipts fail closed.
- macOS image observation retries the spawn-time exec transition but binds every retry to the original PID start identity and accepts only the registered expected executable. Linux retains its open `/proc/PID/exe` image probe and start-time checks. Unsupported systems remain fail closed.
- The frozen adapter restores per-case HOME cleanup and creation before assigning the directory as child cwd.
- The optional adapter smoke uses an exclusive, unique temporary binary directory. It proves HOME creation, receipt-key absence in the child, launch receipt creation, and successful close receipt creation, without modifying the caller’s binary directory.
- Baseline admission checks, frozen source/tool pins, approved expectations, and reference-decision guards are unchanged. New diagnostic builds remain unadmitted.

## Evidence

- `cargo fmt --manifest-path tools/frozen-cli-frame-support/Cargo.toml -- --check`: pass.
- `cargo nextest run --manifest-path tools/frozen-cli-frame-support/Cargo.toml --no-fail-fast`: 30 passed, 0 skipped.
- `python3 -m py_compile tools/frozen_baseline.py tools/test_frozen_baseline.py`: pass.
- `python3 -m unittest tools.test_frozen_baseline`: 41 passed.
- `git apply --check tools/frozen_cli_pair.patch` against the extracted frozen source: pass.
- Pinned frozen visual-baseline target compiled with `cargo nextest run --no-run --locked -p junie-tui --test visual_baseline`: pass.
- Optional patched-harness `paired_adapter_spawn_smoke --test-threads=1`: 1 passed. It confirmed the child omitted the receipt key and emitted both launch and close receipts; a sentinel executable in the supplied `VISUAL_BIN_DIR` remained unchanged.
- Independent negative control substituted the sealed baseline `showcase` where the candidate binary was expected. The harness rejected it, closed/reaped the child, and emitted no valid launch receipt.
- Independent review verified the Rust/Python canonical HMACs and closure-to-launch hash linkage on the exact implementation commit.

## Limits and remaining work

The full four-application paired capture/replay/candidate comparison has not passed twice, and no replacement baseline binary cohort has been admitted. Exact product parity and library acceptance remain incomplete. The reviewer did not inject a real Tuiscotti close failure; the error path omits completion evidence and the verifier rejects missing closure receipts, but teardown-failure injection remains additional coverage. Receipt smoke and negative-control checks are test-system evidence only, not application parity evidence.

`git diff --check` reports the required single-space context lines in the unified extension patch as whitespace. Those context lines are required for patch application and were retained; the patch applies cleanly to the frozen source.

## Task-branch cleanup

After integration and review, task-owned branch `wip/termrock-launch-receipts-20261021` at `d8fd898fc1b12f65f2072bb8bfe362b4cd20ea58` was deleted. Before deletion, its tip was verified as an ancestor of `parity/termrock-visual-baseline`; GitHub reported no pull request for that head (`gh pr list --state all --head ...` returned `[]`). The ordinary remote deletion succeeded without force. After deletion, `git ls-remote --heads origin` reported 86 branches, the PR #38 branch resolved to `e0ac3e260b8d9d944a10f080a528938f9dea05ed`, and the WIP ref was absent. This is task-branch cleanup evidence only; it does not imply that unrelated branches have been reviewed or removed.
