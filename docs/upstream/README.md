# Upstream feedback prompts

Each file here is a self-contained prompt for an AI agent working in an
upstream repository. The product owner copies a file verbatim into that
agent's session. The files contain no conversation context and point at
this repository and branch (`design/interaction-polish`) for evidence.
Nobody opens issues or PRs upstream from here.

| File | Upstream | Our pin today | Asks | Status |
|---|---|---|---|---|
| [`velnor-new-feedback.md`](velnor-new-feedback.md) | [tailrocks/velnor-new](https://github.com/tailrocks/velnor-new) | `v0.1.8` (`.velnor/release-manifest.json`) | visual-test suites: named nextest profile, real `--partition` shards, schedule/dispatch triggers, artifact upload, timeouts, required membership, preserved workflows; opt-in `.github/CLAUDE.md` symlink | drafted, not yet forwarded |
| [`tuiscotti-feedback.md`](tuiscotti-feedback.md) | [tailrocks/tuiscotti](https://github.com/tailrocks/tuiscotti) | `rev = a47c9aaefb34e4c00026f99d8a8dd7ee5916b274` (`Cargo.toml`) | lone `f` key, exact dependency pins, `.ansi`+cursor artifact mode with CLI accept, non-unique locators, last screen in `WaitError`, render speed, no absolute paths, clock/pointer/docs | drafted, not yet forwarded |

When an upstream agent reports back:

1. Record the new version or rev and the date in the table.
2. Apply the consumer changes it lists. velnor: replace the manifest,
   edit `.velnor/config.toml`, run `velnor-actions generate`, update
   `docs/ci.md` and the `AGENTS.md` `.github/CLAUDE.md` exception.
   tuiscotti: bump the `rev`, delete the local gate and `ansi_cursor.rs`
   where upstream replaces them, and re-run the visual gate.
3. Mark the row done, or write a follow-up prompt here for anything it
   declined.
