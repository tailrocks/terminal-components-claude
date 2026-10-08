# Initial visibility status

source-facts.json holds measured source identities and current CI observations.
status.py renders the root STATUS.md from that file and the accepted task records
in docs/implementation/visibility/tasks.json. It does not execute a product,
infer a result from a checklist, or consume a paired run receipt. A run receipt
is a record of execution results tied to an exact source pair and required test
set. Until a later accepted change adds validated run receipts, product result
rows must remain NOT_RUN.

The source facts were collected on 2026-10-08 through these read-only commands:

    git rev-parse HEAD
    gh api repos/tailrocks/terminal-components-claude/branches/termrock-implementation --jq .commit.sha
    gh api repos/tailrocks/terminal-components-claude/branches/visual-baseline --jq .commit.sha
    gh api repos/tailrocks/terminal-components-claude/git/ref/tags/visual-baseline --jq .object
    git rev-parse 'refs/tags/visual-baseline^{}'
    gh run view 37721476033 --repo tailrocks/terminal-components-claude --json headSha,conclusion,status,workflowName,createdAt,jobs,url
    gh api repos/tailrocks/terminal-components-claude/actions/runs/37721476033/jobs --jq .total_count
    gh api repos/tailrocks/terminal-components-claude/actions/runs/37721476033/artifacts --jq .total_count
    gh api repos/tailrocks/terminal-components-claude/check-runs/113129904697 --jq '{id, name, status, conclusion, head_sha, details_url, output: .output.summary}'
    mise current --quiet
    mise exec -- cargo nextest --version
    python3 --version

The remote candidate branch matched the local candidate HEAD. The remote
visual-baseline branch was recorded separately; its SHA was not compared with
local candidate HEAD. The visual-baseline annotated tag object and peeled commit
matched the frozen identities in the root instructions. CI run
[37721476033](https://github.com/tailrocks/terminal-components-claude/actions/runs/37721476033)
failed on the measured candidate SHA before any job or artifact was recorded.
Its run-page annotation states: “Workflow file exceeds the maximum allowed size
of 500 KB.” The DCO check is a separate repository gate. Neither observation is
a product test result.
The listed tool versions are available in this environment only; their presence
does not show that a product check ran.

## Generate and check

Run from the repository root with Python 3.9 or later:

    python3 tools/visibility/status.py --write
    python3 tools/visibility/status.py --check
    python3 -m unittest discover -s tools/visibility/tests -v

The `--role reference` option renders the same measured pair, result rows, and
accepted task records with `visual-baseline` as the local role. Keep the source
facts and accepted task records mirrored read-only on the reference branch. Its
work-queue links point back to the candidate branch. The default role is
`candidate`; `--write` writes the role selected for the current checkout.

Ready has its own result column. It is separate from build, launch, first frame,
input and interaction, exit, restoration, visual, and ownership results. If no
current run receipt is present, the product result is NOT_RUN; NOT_RUN is a
result status, not a run-receipt status.

--check is read-only and requires byte-for-byte agreement. The renderer has no
publication credentials, does not update the task records or work queue, and
does not write anything except STATUS.md when called with --write.

## Current limits

This first increment reports the measured source pair and an honest NOT_RUN
state. It does not implement the validated receipt reader, complete case
registry, freshness rejection for run receipts, trend calculation, component
checkpoint pages, or trusted publication. Those results stay unmeasured until
those mechanisms have their own accepted work and evidence.
