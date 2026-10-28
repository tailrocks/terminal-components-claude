# Ordinary CLI terminal-profile partition

Status: focused feature checks pass; end-to-end paired product verification remains incomplete.

## Provenance and scope

- Existing feature branch before integration: `parity/ordinary-cli-partition-20261020` at `0d7f38bac1ce4420036d930e1f90ad05459732de`.
- Integrated PR #38 source before this feature: merge commit `073c635f7546741bb2e2271494201ccaf0823086`; its tree matches PR #38 tree `9f7940f6af0276e4816a67947e8416bd5c11fa5e` at integration time.
- Frozen references remain unchanged: baseline tag object `2718a6b8127a522a3d351d0b4bdb7542fc8da542`, source `4169783ae883788186d161af09ba9c481bd84e5c`, Tuiscotti `a1ea27f757d75d07b05499276cd932e9f8e067ad`.
- Unique useful feature ported: opt-in `--term-profile` (`xterm-256color`, `linux`, `dumb`, `unset`) for paired ordinary CLI scenarios, propagated consistently to the runner and spawned Tuiscotti child. An effective-environment record and digest are included in chunk/results evidence. Default behavior is retained: inherited runner `TERM`, pinned child `TERM=xterm-256color`.
- `tools/frozen_cli_pair.patch` forwards the opt-in profile after `.env_clear(true)`, without changing pinned environment values in default runs. The fixed grown-reference exception rejects all profile overrides because its review evidence is tied to the harness-default child TERM.
- No reference snapshots, approvals, expected frames, pins, or production app paths were changed. No unrelated branch work was ported.

## Checks

- Red-before-green on the target runner: the new feature tests failed as expected (31 tests: two missing-API errors and one missing harness-forwarding assertion).
- Green after implementation: `python3 tools/test_frozen_baseline.py` — 31 passed.
- `python3 -m py_compile tools/frozen_baseline.py tools/test_frozen_baseline.py` — passed (cache directed outside repository).
- `python3 tools/frozen_baseline.py --help` — passed.
- `git diff --check` — passed.
- Fixed-source extraction verified the tag, commit, Tuiscotti revision and source provenance. The frozen patch was dry-run and applied against that extracted source; it modified only `tests/visual_baseline/main.rs`, `scenarios/interact.rs`, `scenarios/mod.rs`, and `support.rs`. Snapshot tree digest was unchanged. Detailed preserved evidence: `/Users/donbeave/Projects/termrock-cli-evidence-20261020/branch-consolidation/ordinary-cli-profile-port/term-patch-check.json` and `.log`.

## Limitations

`prepare-live` did not complete: the required approved baseline `build.json` record was absent (and previous binary identity/rebuild evidence was not acceptable). No live paired PTY run under alternate TERM profiles or candidate-vs-baseline comparison is claimed. Do not create or bless a replacement build record to bypass this guard. Product parity remains unverified/incomplete until approved baseline binaries and the full paired checks are available.
