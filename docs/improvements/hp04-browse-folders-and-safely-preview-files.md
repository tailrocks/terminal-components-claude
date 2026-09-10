# HP04 — Browse folders and safely preview files

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Build the Files page, exact-path jump, navigation, bounded previews and focus/scroll restoration on fixture directory/file data. Model stale responses, binary/control-rich content, special-file replacement and read failures.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real filesystem listing/reads, descriptor validation and OS race/nonblocking probes.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve the real browser and safe file preview, including obscure path navigation and asynchronous race protection. No equivalent general browser exists in the preview.

**Source evidence and mandatory scope:** [matrix HP04](../holla-parity-matrix.md#hp04--browse-folders-and-safely-preview-files) — `I-B01`, `I-B03`, `I-B04`, `I-B05`, `I-B07`, `I-B08`, `I-B10`, `I-B12`, `I-B14`, `I-B02`, `I-B06`, `I-B09`, `I-B11`, `I-B13`, `I-P01`, `I-P02`, `I-P03`, `I-P04`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Create a Files page inside Here: real directory semantics in simulated fixtures, directories first, type/size/mtime/hidden metadata, parent/child navigation and highlighted return target. Exact-path picker accepts relative/home/absolute paths and prioritizes exact existing paths over fuzzy suggestions. Preserve page/query/scroll state when returning to Here; make any context rebind explicit. File Enter previews; OS open remains an alternative. Directory command recommendations identify preview-only versus executable actions.

**Architecture / reusable components:** Use typed native path identity rather than lossy display text. Generation-tag listing/jump/preview results; discard stale successes and errors. Reuse Picker, TreeView where needed, Input, Splitter and TextViewport. Bounded preview adapter sanitizes content/title/path/links and validates the opened descriptor, not only pre-open metadata.

**Required deterministic fixture:** `parity-browser` — file/dir starting path, invalid path, hidden toggle, exact jump, missing/broken symlink, lossy-name collision, delayed old response, capped directory count, empty/large/binary/invalid UTF-8/control-rich file, FIFO/device replacement.

**Acceptance / automated verification:** Assert parent+highlight behavior, 50 jump suggestions and source labels, 2048-entry/40 ms lower-bound directory counts, 256 KiB/2000-line/4096-character preview bounds and safe UTF-8 cap. Assert no read of special opened descriptor, nonblocking race handling, sanitized terminal bytes, inline failures, focus/scroll retention and visible replacement for blind Ctrl-L editing. File reading never executes content. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture listing/preview focus, hidden entries, path error, partial count, binary/error/long-text preview and stale-result rejection journey.

**Dependencies:** HP03/HP15/HP23; F01/F05/F10–F14/F19/F20/F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
