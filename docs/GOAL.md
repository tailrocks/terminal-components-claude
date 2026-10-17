# Goal: restart and continue the open work

Single entry point for any agent to restart and continue the work on branch
`design/interaction-polish`.

## Continue from a handoff

Work may have stopped mid-task. On every restart, before any new work,
the coordinator:

1. Reads [handoff/README.md](handoff/README.md): the table of every
   unfinished `wip/*` branch with its exact next steps, and the
   operational knowledge (push lock and lock relay, archive-first visual
   gate, cell attribution, `target-dev/`, known flakes, machine load).
2. Runs `git fetch origin && git ls-remote --heads origin 'wip/*'` and
   compares it with that table. A branch without a row gets one first.
3. Checks `"$(git rev-parse --git-common-dir)/push.lock"`: if its
   `owner` file is older than 50 minutes, the lock is stale and is
   removed with `rm -rf`.
4. Lands or resumes the WIP branches (rebase, finish, gate inside the
   lock, push to `design/interaction-polish`), one subagent per branch,
   each given [handoff/subagent-rules.md](handoff/subagent-rules.md) and
   the branch's resume note. Only then does it start new items.

**Stopping rule:** any coordinator (or agent) that stops with unfinished
work pushes it to `wip/<name>` on `origin` and lists it (tip, item,
state, exact next steps) in `docs/handoff/README.md` before it stops.
Then there is always a way to continue, even if the session is lost.

## How to restart

From a checkout of `design/interaction-polish` (`git pull`, `mise trust`,
`mise install`), start Claude Code and run `/goal` with the prompt below
(under 4000 characters).

```
Finish every item in docs/remaining-work.md on branch design/interaction-polish of /Users/donbeave/Projects/terminal-components-claude until its "done when" criteria hold.

You are the coordinator only: subagents do all implementation, review and audit. You plan, dispatch, read reports, relay results to me in plain language, forward new findings to the owning step, and keep docs/remaining-work.md and the plan's status table current.

First read AGENTS.md (binding) and docs/remaining-work.md, then the docs each item cites.

Continue from the handoff before any new work: read docs/handoff/README.md; run `git fetch origin && git ls-remote --heads origin 'wip/*'` and add a row for any branch it lacks; if push.lock's owner file is older than 50 minutes, `rm -rf` the lock; then land or resume every wip/* branch (its row's next steps).

If you stop with unfinished work, push it to `wip/<name>` and list it in docs/handoff/README.md first, so there is always a way to continue.

Every subagent prompt must require AGENTS.md, especially "Way of working", and docs/handoff/subagent-rules.md, plus:
- Its own worktree: `git fetch origin && git switch --detach origin/design/interaction-polish && git switch --no-track -C <branch>`, with `git status --porcelain` empty.
- Tools from mise.toml (`mise install`), plain cargo.
- Run only scripts it wrote. No `pkill -f` or `killall`.
- Push lock: `mkdir "$(git rev-parse --git-common-dir)/push.lock"` with an owner file inside. Remove a lock whose owner file is older than 50 minutes. Always `rm -rf` the lock after pushing or on failure.
- Inside the lock: fetch, rebase, check the diff stat shows only intended files, run the full gate on the exact HEAD, then `git push origin HEAD:design/interaction-polish`.
- If only waiting remains, end the run and report where to resume.
- If stopped unfinished, push to `wip/<name>` and report its tip and next steps.

When an agent stops on a held lock: run a background `until mkdir <lock>; do sleep 5; done`, write that agent's name in the owner file, then resume the agent and tell it the lock is its own.

Never touch the `visual-baseline` tag. Never post anything publicly without my OK, except reviews in the Upstream PR loop.

The owner decisions in docs/remaining-work.md are mine: ask me with your recommendation, and keep working on other items meanwhile. When I send a velnor-new or tuiscotti link, or say "check velnor-new", "check tuiscotti" or "check upstream", follow the Upstream PR loop in docs/upstream/README.md and answer with one relay line per PR.

Stop when every "done when" criterion holds, or when only owner decisions and upstream-blocked items remain; then list them for me.
```

## Where progress is tracked

- `docs/handoff/README.md`: unfinished `wip/*` branches with next steps,
  operational knowledge, scripts; read first on every restart.
- `docs/remaining-work.md`: authoritative list. Priority 0/0b, priority
  order NOW/NEXT/LATER, owner decisions D1-D11, all work items, "done when".
- `docs/architecture/shared-components-plan.md`: S-steps status table and
  outcome sections; section 7 has the showcase coverage SC steps.
- `docs/audits/design-review-2026-10/README.md`: all audits (responsive.md,
  visual-reapproval.md, states.md, colour, focus, keyboard, mouse) with
  status tables.
- `docs/visual-testing/README.md`, `plan.md`, `tuiscotti-migration.md`:
  visual gate, known flakes, oracle plan.
- `docs/upstream/README.md`, `velnor-new-feedback.md`,
  `tuiscotti-feedback.md`: upstream PR loop and trackers.
- `docs/dependencies.md`, `docs/ci.md`: dependency and CI policy.
- `AGENTS.md`: binding rules, including the 5-subagent cap, push lock,
  5-minute command rule, never wait, visual rules.
- `tests/showcase_coverage_allow.txt`, `tests/showcase_reach_allow.txt`,
  `tests/reach_allow.txt`: machine-checked remaining gaps; shrink-only lists.

## Coordinator conventions that make it work

- At most 5 subagents including helpers (3 when budget is tight).
- Each subagent prompt points to AGENTS.md "Way of working".
- Lock-relay pattern: coordinator claims a held push lock on behalf of the
  waiting agent, then resumes it.
- Take the push lock BEFORE gating.
- Visual gate runs from ONE nextest archive, at most 4 parallel partitions,
  each part under 4 minutes.
- Attribute every changed capture cell by script before `tuiscotti accept`;
  snapshot-only commits.
- Stop idle agents.
- When stopping mid-task, push the work to `wip/<name>`, list it in
  `docs/handoff/README.md` (tip, item, state, exact next steps), copy the
  resume notes to `docs/handoff/resume/`, and point the
  docs/remaining-work.md rows at the branch.
- On restart, land or resume the `wip/*` branches first (section
  "Continue from a handoff").

## Current state

The live status is the top of `docs/remaining-work.md`. Numbers are not
duplicated here.
