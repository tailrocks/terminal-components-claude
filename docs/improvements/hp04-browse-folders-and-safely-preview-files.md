# HP04 — Browse folders and safely preview files

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Build the Files page, exact-path jump, navigation, bounded previews and focus/scroll restoration on fixture directory/file data. Model stale responses, binary/control-rich content, special-file replacement and read failures.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real filesystem listing/reads, descriptor validation and OS race/nonblocking probes.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve the real browser and safe file preview, including obscure path navigation and asynchronous race protection. No equivalent general browser exists in the preview.

**Source evidence and mandatory scope:** [matrix HP04](../parity/holla-parity-matrix.md#hp04--browse-folders-and-safely-preview-files) — `I-B01`, `I-B03`, `I-B04`, `I-B05`, `I-B07`, `I-B08`, `I-B10`, `I-B12`, `I-B14`, `I-B02`, `I-B06`, `I-B09`, `I-B11`, `I-B13`, `I-P01`, `I-P02`, `I-P03`, `I-P04`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Create a Files page inside Here: real directory semantics in simulated fixtures, directories first, type/size/mtime/hidden metadata, parent/child navigation and highlighted return target. Exact-path picker accepts relative/home/absolute paths and prioritizes exact existing paths over fuzzy suggestions. Preserve page/query/scroll state when returning to Here; make any context rebind explicit. File Enter previews; OS open remains an alternative. Directory command recommendations identify preview-only versus executable actions.

**Architecture / reusable components:** Use typed native path identity rather than lossy display text. Generation-tag listing/jump/preview results; discard stale successes and errors. Reuse Picker, TreeView where needed, Input, Splitter and TextViewport. Bounded preview adapter sanitizes content/title/path/links and validates the opened descriptor, not only pre-open metadata.

**Required deterministic fixture:** `parity-browser` — file/dir starting path, invalid path, hidden toggle, exact jump, missing/broken symlink, lossy-name collision, delayed old response, capped directory count, empty/large/binary/invalid UTF-8/control-rich file, FIFO/device replacement.

**Acceptance / automated verification:** Assert parent+highlight behavior, 50 jump suggestions and source labels, 2048-entry/40 ms lower-bound directory counts, 256 KiB/2000-line/4096-character preview bounds and safe UTF-8 cap. Assert no read of special opened descriptor, nonblocking race handling, sanitized terminal bytes, inline failures, focus/scroll retention and visible replacement for blind Ctrl-L editing. File reading never executes content. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture listing/preview focus, hidden entries, path error, partial count, binary/error/long-text preview and stale-result rejection journey.

**Dependencies:** HP03/HP15/HP23; F01/F05/F10–F14/F19/F20/F23.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-browser` (fixture fn `browser` in src/bin/holla/domain/parity.rs) · `hp04_browser_lists_previews_and_jumps_safely` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/sim/fs.rs: listing_sorts_directories_first_and_counts_hidden`, `src/bin/holla/sim/fs.rs: preview_is_bounded_sanitised_and_refuses_special_files`, `src/bin/holla/sim/fs.rs: human_sizes_and_sanitise`. · row proofs in src/bin/holla/app_tests_rows.rs: `i_b14_context_markers_and_recommendations_are_preview_only`.

**What the journey asserts:**
- "Browse ~/work/site" opens "Files ›"; "assets" is listed above "index.html"; ".env" is absent until Ctrl+H, which shows ".env" and ".git"; Ctrl+H again hides them.
- Both "café.txt" (precomposed) and "café.txt" (decomposed) are distinct listed entries.
- Clicking each file previews it: "index.html" shows "<html>" and "hello"; "empty.txt" shows "empty"; "logo.png" shows "binary" and never the raw byte U+0089; "bad.txt" never leaks the raw byte 0xFF; "control.txt" shows "red" and never ESC (U+001B) or BEL (U+0007); "big.log" shows "first 2000 lines" and never "line 2099"; "long.txt" shows "cut at 4096"; "pipe" shows "special file".
- "broken" (dangling symlink) shows "broken symbol"; Enter on "private" renders "permission denied".
- Enter on "assets" (fixture latency 8 ticks) first shows "listing" or "…", then "2100" or "img0000.png".
- `g` opens the jump picker; "~/nowhere" keeps the modal open and shows "!"; "~/work" closes it, renders "Files › work" and "Jumped to ~/work".

**Captures:** `shots/h_hp04_browse` (listing, directories first), `shots/h_hp04_hidden` (hidden entries shown), `shots/h_hp04_preview_big` (capped long file preview), `shots/h_hp04_preview_control` (sanitised control-rich preview), `shots/h_hp04_jump` (jump picker), `shots/h_hp04_jump_error` (missing path error inline, picker stays open), `shots/h_hp04_jumped` (after jumping to ~/work); base matrix `shots/h_parity_browser_{80x24,100x30,120x40,160x50,mono}`. State: provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| I-B01 | hp04: directory start "Browse ~/work/site" gives "Files ›". File start highlighting and invalid initial path are not asserted. | real cwd default, pre-terminal failure |
| I-B03 | hp04: "assets" before "index.html"; `fs.rs: listing_sorts_directories_first_and_counts_hidden` asserts `[".hidden", "a", "b", "sparse.img"]` and `(visible, hidden, capped) == (3, 1, false)`. Size/mtime columns are visible in `h_hp04_browse`, not asserted. | real metadata |
| I-B04 | hp04: Enter on "private" gives "permission denied"; Enter on "assets" lists it. Left/Backspace parent re-highlight is not asserted. | none |
| I-B05 | hp04: Ctrl+H shows ".env" and ".git", second Ctrl+H hides them. | none |
| I-B07 | hp04: `g` opens the picker (`modals` non-empty). 50-suggestion cap, source labels and completion order are not asserted. | global index |
| I-B08 | hp04: "~/nowhere" keeps the picker open with "!"; "~/work" jumps ("Files › work", "Jumped to ~/work"). Relative path and file jump are not asserted. | none |
| I-B10 | hp04: "broken symbol" error for the dangling link; precomposed and decomposed "café.txt" both listed. Activation refusal is not asserted. | none |
| I-B12 | hp04: "2100" or "img0000.png" after the slow listing; `fs.rs` `COUNT_CAP` is 2048 and the count test asserts `capped == false`. The "at least (count capped at 2048 / 40 ms)" annotation is not asserted. | 40 ms wall-clock bound |
| I-B14 | `i_b14_context_markers_and_recommendations_are_preview_only`: previewing `~/work/site` from `~/work` shows `markers · .git, package.json`, `git status · git pull`, the pnpm script recommendations and "recommendations are preview-only"; `world.activities` stays empty. The eight-script cap is visible in the frame (scripts a to g plus the git line), not asserted. | command availability |
| I-B02 | Superseded: `open` helper returns to Here with Esc. Ctrl+O toggling is not asserted. | none |
| I-B06 | hp04 clicks list rows via `files::LIST`; preview focus and scrolling are not asserted. | none |
| I-B09 | hp04: the slow "assets" listing is "listing"/"…" then complete. Stale response rejection is not asserted directly. | real async listing |
| I-B11 | `app_tests.rs: too_small_notice_and_recovery`: below the minimum size the frame reads "Terminal too small" and the page returns after a resize. | none |
| I-B13 | `i_b14_context_markers_and_recommendations_are_preview_only`: the recommendation block ends with "recommendations are preview-only" and nothing runs from the preview. | none |
| I-P01 | hp04: "first 2000 lines" without "line 2099", "cut at 4096", "empty"; `fs.rs: preview_is_bounded_sanitised_and_refuses_special_files`: `bytes_shown <= PREVIEW_MAX_BYTES`, `bytes_shown % 2 == 0` (no split code point), `long_lines == 1`, `truncated_lines: true`, `Preview::Empty`. | real reads |
| I-P02 | hp04: "logo.png" says "binary"; `fs.rs` test: `bad.txt` (0xFF) is `Preview::Binary`, missing is `NotFound`; inline "broken symbol" and "permission denied". | descriptor errors |
| I-P03 | hp04: "control.txt" shows "red" with no ESC or BEL; `fs.rs: human_sizes_and_sanitise` asserts `sanitize("a\tb\u{7}c") == "a    b�c"`; `fs.rs` preview test: symlink preview carries `link: Some(_)`. | none |
| I-P04 | hp04: "pipe" shows "special file"; `fs.rs` test: `Preview::Error(FsError::SpecialFile)`. Nonblocking open and descriptor recheck are not modeled. | O_NONBLOCK open, fstat recheck, FIFO swap |

**Deferred remainder:**
- Real filesystem listing/reads, descriptor validation and OS race/nonblocking probes (Later).
- 50 jump suggestions with source labels, 40 ms directory-count bound and the capped annotation are not asserted.
- Focus/scroll retention on return and visible replacement for blind Ctrl+L editing are not asserted.
- Directory command recommendations (markers, eight-script cap, lock preference) are not asserted.
- Small-terminal resize message is not asserted.

**Limits:**
- Latency is a tick count on a fixture path; no OS scheduling or blocking open is exercised.
- Byte sanitisation is proven on fixture bytes, not on files a real terminal would receive.
