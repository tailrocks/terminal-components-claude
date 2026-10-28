# DCO remediation for the delivery branch

Status: **OPEN. The owner must run this; the coordinator did not.**
Branch: `parity/termrock-visual-baseline`.

## The problem

The DCO check on PR #38 is `ACTION_REQUIRED`. At the delivery tip,
**1630 commits** are in `main..parity/termrock-visual-baseline` and **1566**
of them carry no `Signed-off-by` trailer. The check does not look only at the
newest commits, so a partial re-sign does not clear it.

The delivery rules forbid the coordinator from force-pushing a shared
branch, and another session is committing to the same branch, so the
rewrite was not attempted from the coordinator.

## The verified procedure

The command shape was verified on a scratch clone of the delivery branch:
replaying three commits with

```sh
git config user.name "Alexey Zhokhov"
git config user.email "alexey@zhokhov.com"
git rebase --exec 'git commit --amend --no-edit -s' HEAD~3
```

produced three commits that each carry
`Signed-off-by: Alexey Zhokhov <alexey@zhokhov.com>`, with the author,
the author date and the message unchanged.

For the whole branch, in a quiet window, with no other writer:

```sh
# 1. work on a temporary branch so the delivery branch is never rewritten in place
git fetch origin
git checkout -B dco/sign-all origin/parity/termrock-visual-baseline
git rebase --exec 'git commit --amend --no-edit -s' origin/main

# 2. publish with an explicit lease, so a concurrent push is refused rather than lost
git push --force-with-lease=parity/termrock-visual-baseline:$(git rev-parse origin/parity/termrock-visual-baseline) \
       origin dco/sign-all:parity/termrock-visual-baseline

# 3. keep the old tip reachable for the duration of the review
git push origin dco/sign-all:refs/heads/wip/termrock-dco-signoff-backup
```

## Conditions

1. The branch must be quiet. Every push to it cancels the hosted pipeline, so
   the rewrite and the re-run have to be scheduled together.
2. `--force-with-lease` is mandatory. A plain `--force` could drop a commit
   another session pushed between the fetch and the push.
3. The pre-rewrite tip is kept on `wip/termrock-dco-signoff-backup` until
   the PR is merged, so the old history stays reachable.
4. The rewrite changes commit ids only. No tree, no message and no author
   date changes, so the content the reviews examined is the content that
   ships.
5. The hosted pipeline has to be re-run on the rewritten tip, because every
   commit id changed.

## Why the coordinator did not run it

- The delivery rules say: "Never force-push shared branches."
- Another session is committing to the same branch. A rewrite under it would
  race with that work, and the `wip/` branches that session is holding would
  need rebasing on top of it anyway.
- The rewrite is a delivery step, not a product change: it carries no
  behaviour and no reviewable content.
