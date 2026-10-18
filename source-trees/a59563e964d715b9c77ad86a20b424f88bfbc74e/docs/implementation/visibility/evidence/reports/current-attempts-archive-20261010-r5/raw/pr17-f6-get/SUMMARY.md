# PR17 current candidate CI status packet

Captured from `2026-10-10 00:49:35 UTC` through `2026-10-10 00:50:18 UTC`. Exact queried candidate SHA: `f6d26bf2f0e5ba5594b9c80c5e512dce4213e545`. This is a bounded, read-only snapshot for later status review; it does not track future head changes. Raw authenticated GitHub REST GET responses (including HTTP status/headers/body) and hashes are in `manifest.json`. No GitHub writes, reruns, builds, Cargo, tests, source edits, or long-history pagination.

## PR pins at capture

PR 17 was open and draft with mergeable state `dirty`. The head was `f6d26bf2f0e5ba5594b9c80c5e512dce4213e545` before and `f6d26bf2f0e5ba5594b9c80c5e512dce4213e545` after the request set; both match the exact queried SHA. Base: `81a8bf15cd3042f80649e2b48fed479829518dbd` (`main`). PR event endpoint page 1 returned `6` events and no next-page link.

## Exact-head workflow and checks

- Candidate workflow `.github/workflows/ci.yml`: GitHub blob `672078dc6c964a768c05b53f73de8bb99b90bffa`, `537470` bytes. This blob is unchanged, so reuse the prior static validation record: actionlint 1.7.12 + ShellCheck 0.11.0 passed with exit 0/no findings (`reused-actionlint-success.txt`, SHA-256 `0186c30d1b402bf338ebf6a638c2259c84f4f0b4e6e83b35800058aa16436671`). GitHub's [Actions limits](https://docs.github.com/en/actions/reference/limits) specify a 500 KB workflow-file limit for triggering a run. The measured workflow exceeds it; this is a known limit violation, not proof of this run's specific startup cause.
- Combined status: `pending`, `0` legacy contexts. DCO check run `success`. The GitHub Actions suite is `failure` with `latest_check_runs_count=0`.
- Latest exact-head run: `38007979466`, event `push`, attempt `1`, conclusion `failure`. Jobs `0`, attempt jobs `0`, artifacts `0`; timing billable is empty. The run-log archive response is `HTTP/2.0 404 Not Found`.
- Preserve as **completed failure with zero exposed jobs; startup/provider cause NOT EXPOSED**. Do not copy a cause from any other run or attribute this failure to workflow size.

## Reference and packet bounds

Reference pin `4b473a98a8641a9dae8dbf9c31c0c94b15465496` is recorded as unchanged per Root's remote check at `2026-10-10 00:46 UTC`; it was not re-fetched here. The exact-head runs, checks, check suites, jobs, artifacts, and PR events were requested as page 1 with `per_page=100`; every saved response had no `Link: rel=next`, so no continuation page was needed. This avoids scanning PR commit history.

`manifest.json` contains raw response request paths, response dates, HTTP statuses, byte counts, SHA-256 digests, and pagination headers.
