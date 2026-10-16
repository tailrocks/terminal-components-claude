# Upstream feedback prompts

Each file here is a self-contained prompt for an AI agent working in an
upstream repository. The product owner copies a file verbatim into that
agent's session. The files contain no conversation context and point at
this repository and branch (`design/interaction-polish`) for evidence.
Nobody opens issues or PRs upstream from here; we only review the PRs
the upstream agents open (see [Upstream PR loop](#upstream-pr-loop)).

| File | Upstream | Our pin today | Asks | Status |
|---|---|---|---|---|
| [`velnor-new-feedback.md`](velnor-new-feedback.md) | [tailrocks/velnor-new](https://github.com/tailrocks/velnor-new) | `v0.1.8` (`.velnor/release-manifest.json`) | visual-test suites: named nextest profile, real `--partition` shards, schedule/dispatch triggers, artifact upload, timeouts, required membership, preserved workflows; opt-in `.github/CLAUDE.md` symlink | forwarded; PR [#155](https://github.com/tailrocks/velnor-new/pull/155) open, §8 has no PR (see the tracker in the file) |
| [`tuiscotti-feedback.md`](tuiscotti-feedback.md) | [tailrocks/tuiscotti](https://github.com/tailrocks/tuiscotti) | `rev = a47c9aaefb34e4c00026f99d8a8dd7ee5916b274` (`Cargo.toml`) | lone `f` key, exact dependency pins, `.ansi`+cursor artifact mode with CLI accept, non-unique locators, last screen in `WaitError`, render speed, no absolute paths, clock/pointer/docs | forwarded; PR [#11](https://github.com/tailrocks/tuiscotti/pull/11) open (see the tracker in the file) |

When an upstream agent reports back:

1. Record the new version or rev and the date in the table.
2. Apply the consumer changes it lists. velnor: replace the manifest,
   edit `.velnor/config.toml`, run `velnor-actions generate`, update
   `docs/ci.md` and the `AGENTS.md` `.github/CLAUDE.md` exception.
   tuiscotti: bump the `rev`, delete the local gate and `ansi_cursor.rs`
   where upstream replaces them, and re-run the visual gate.
3. Mark the row done, or write a follow-up prompt here for anything it
   declined.

## Upstream PR loop

Separate agents in [tailrocks/velnor-new](https://github.com/tailrocks/velnor-new)
(CI generator) and [tailrocks/tuiscotti](https://github.com/tailrocks/tuiscotti)
(visual and PTY test harness) implement our feedback through GitHub PRs.
The product owner relays between us and them. This procedure is binding
for the main session.

### Triggers

- The owner sends a link to a PR, issue or commit in either repository:
  check that repository.
- "check velnor-new": check tailrocks/velnor-new.
- "check tuiscotti": check tailrocks/tuiscotti.
- "check upstream": check both.

### Procedure

The main session coordinates; subagents do the checking (one per
repository or per PR), as `AGENTS.md` "Way of working" requires.

1. **Find the work.** Run `gh pr list -R <repo> --state all --limit 20`
   and `gh issue list -R <repo> --state all --limit 20`. Match each open or
   recently updated PR or issue to a feedback item through the tracker at
   the top of the feedback file (`velnor-new-feedback.md`,
   `tuiscotti-feedback.md`). A feedback item with no PR gets the "open a
   PR" relay line.
2. **Read what changed.** For each PR whose head SHA differs from the
   tracker's "Last reviewed head", or that has new comments since that
   head: read the body, the commits, the CI checks (`gh pr checks`) and
   every reply in our review threads. A PR with the same head and no new
   comments gets the "nothing to tell" line and no further work.
3. **Verify against the item's acceptance criteria.** Integrate the PR
   head into a scratch copy of our repository (a worktree in the
   scratchpad, never our working branch) and run the parts of our gate
   that matter:
   - tuiscotti: point the dev-dependency at the PR head, run the visual
     gate and the `.ansi` round-trips, and check the lockfile;
   - velnor-new: a `velnor-actions generate --output-dir` preview from our
     `.velnor/config.toml`, the drift check against `.github/`, and
     actionlint on the output.

   Reproduce every blocker from the previous review. A blocker counts as
   fixed only when the reproduction no longer fails.
4. **Post the review** on the PR as a `COMMENT` review (the PR author and
   our reviewer are the same GitHub account, which cannot
   `REQUEST_CHANGES` or `APPROVE`). The first line states the verdict.
   Reply inside the existing threads; never resolve the author's threads
   ourselves.
5. **Update the tracker row** (state, last reviewed head, review link,
   verdict), then commit and push the docs as usual.
6. **When a PR is merged,** dispatch our follow-up: tuiscotti, bump the
   `rev =` pin to the new `main`; velnor-new, move to the new release
   (replace `.velnor/release-manifest.json`) and regenerate CI. In both
   cases delete the workarounds the feedback file lists under "Our
   follow-up after merge". The follow-up passes the full gate and is
   pushed like any other change.

### Relay reply to the owner

One line per PR, in this fixed vocabulary, so the owner can forward it
unchanged:

- `<repo> #<n> → tell agent: done, ready to merge.`
- `<repo> #<n> → tell agent: address review issues — check GitHub review <url> and the replies/comments.`
- `<repo> #<n> → tell agent: CI failing (<check>) — fix and push.`
- `<repo> #<n> → tell agent: rebase onto main (conflicts) and push.`
- `<repo> #<n> → nothing to tell: no new commits since last review (<sha>).`
- `<repo> → tell agent: open a PR for <feedback item §n>` (an item has no PR yet)
- `<repo> #<n> merged → no action for agent; our follow-up started (<what>).`

After the relay lines, at most 3 lines of why (blocker count, link).
Nothing else is needed to forward.

### Rules

- Review only what the owner asked to check.
- Never push to upstream repositories, and never merge upstream PRs
  ourselves.
- The verdict "ready to merge" requires all three: every blocker of the
  item verified fixed by reproduction, CI green, and the upstream docs and
  migration notes updated.
