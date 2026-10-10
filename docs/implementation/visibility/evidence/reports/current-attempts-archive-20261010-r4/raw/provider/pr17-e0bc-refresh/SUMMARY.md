# PR17 exact-head Actions refresh

Read-only refresh via authenticated GitHub CLI REST GET calls. Raw HTTP status/headers/body are preserved in this directory and inventoried in `manifest.json`. No API writes, source edits, Cargo, builds, tests, workflow dispatches, reruns, or polling loop. Captured response dates range from `Fri, 09 Oct 2026 23:33:02 GMT` through `Fri, 09 Oct 2026 23:34:04 GMT` UTC.

## Pins and candidate head

- PR 17: `open`, draft `True`, mergeable state `dirty`; current head `e0bcfa26ec174241f09de17d04f3f45379be3452`, base `81a8bf15cd3042f80649e2b48fed479829518dbd`. Requested candidate pin matches current PR head.
- Exact candidate workflow `.github/workflows/ci.yml`: blob `672078dc6c964a768c05b53f73de8bb99b90bffa`, `537470` bytes. This is unchanged from the previous workflow/actionlint packet; its static `actionlint` 1.7.12 + ShellCheck 0.11.0 result was exit 0/no findings. See [previous summary](/private/tmp/termrock-vis11-pr17-actionlint-validation-luna-20261010.jgF6ur/SUMMARY.md), SHA-256 `7728cabd908d1a4606d3567a5c9e9b99ab59411007498069f8262f4662cd6752`.
- GitHub's [Actions limits](https://docs.github.com/en/actions/reference/limits) specify a 500 KB maximum for a workflow file to trigger a run; this 537,470-byte file exceeds that documented limit. This is a known workflow admission issue and a plausible explanation for zero jobs, not proof of this run's specific cause.

## Current exact-head checks and run

- Combined commit status: `pending`, `0` legacy contexts. Check-runs endpoint returned `1` check: DCO `success`. GitHub Actions check suite `102973892788` is `failure` with `latest_check_runs_count=0`.
- Latest exact-head workflow run: `38003496873` (`.github/workflows/ci.yml`, `push`, attempt `1`), completed `failure` at head `e0bcfa26ec174241f09de17d04f3f45379be3452`.
- Run jobs and attempt-1 jobs: `0` and `0`. Artifacts: `0`. Timing response has empty `billable`. Run logs archive: `HTTP/2.0 404 Not Found`. The Actions suite exposes zero check runs.
- Preserve the result as **completed failure with zero exposed jobs; startup/provider cause NOT EXPOSED**. The API evidence does not identify why GitHub failed before any job surfaced. Do not attribute the run failure to file size.

## Reference pin

- Reference commit `4b473a98a8641a9dae8dbf9c31c0c94b15465496`: combined status `pending` with zero legacy contexts; one DCO check, `action_required`. The DCO output reports `14` incorrectly signed-off commits. Exact-head Actions workflow runs: `0`. The workflow contents endpoint for this ref returned `HTTP/2.0 404 Not Found`, so no reference workflow blob was included in this refresh.

## Evidence inventory

`manifest.json` lists the exact GET route, HTTP status, response date, byte count, and SHA-256 for every raw response in this capture. The 404 responses are preserved as returned and are not interpreted as startup diagnostics.
