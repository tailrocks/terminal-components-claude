# Visual change record

## Status

Historical visual classifications retained for reference. No baseline update is authorized by this document. There is no current `bless-guard` command in `crates/termrock-xtask/src`; old blessing, scratch-copy, and `git checkout` instructions are retired.

## Authority

The exact product target is the current `design/interaction-polish` branch: its UI, colors, and behavior. `main` supplies the split architecture. Current baseline files and capture tools are evidence only; they do not override the design target.

## Current evidence

The former ledger records classifications, captures, and test output from historical source revisions. Preserve those records as historical evidence at immutable commit [`d1122f69`](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/visual-changes.md). Paths, hashes, classifications, test names, and tool instructions in that archive do not describe current output unless reverified.

## Remaining work

For any future visual change, compare the current implementation with `design/interaction-polish`, identify affected surfaces and color levels, capture before/after evidence using currently available tools, and classify every observed difference as matching the design, correcting a demonstrated defect, or unresolved. Record evidence and unresolved differences here only after current source and artifact verification. Keep historical entries clearly identified as historical.

No baseline may be regenerated, copied, or updated as part of documentation cleanup. Any future baseline change requires an independently authorized task and review of current evidence; this ledger does not provide that authorization.

## Verification

Check current capture and baseline tooling in source before documenting a procedure. Compare the rendered current branch with `design/interaction-polish` across supported sizes and color modes. Never treat an old digest, screenshot, or successful test run as proof of current visual parity.
