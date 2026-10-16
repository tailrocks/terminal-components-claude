**Verdict: commit one file per scenario, a `.ansi` with a cursor trailer line.** Gate on that file only. Rebuild PNG and HTML from it when someone needs to review.

The PO hypothesis is half right. `.png` is redundant, but `.ansi` on its own is not enough. tuiscotti's `ansi_dump` (`tuiscotti-render/src/render/text.rs:46-70`) writes no cursor at all. Today the caret is checked only through the `.png` and `.html` gates, and 1,950 of the 7,550 scenarios (25.8%) have a visible cursor.

## Why each format stays or goes

| Format | Decision | Reason |
|---|---|---|
| `.ansi` + cursor trailer | **Keep** | Every cell exactly (text, fg/bg as indexed/RGB/default, every modifier, underline style and colour), grid size, and trailing blanks with background. The trailer adds the cursor as real VT sequences: `ESC[?25l` when hidden, else `ESC[?25h ESC[y;xH ESC[Ps q`. It is a lossless stand-in for the full frame on all 7,550 real scenarios. It can be read with `cat` and diffs line by line in PRs. |
| `.txt` | Drop | It is the `.ansi` with styles stripped and trailing blanks trimmed. No test pair changed `.txt` without also changing `.ansi`. It also loses grid width (24x5 and 25x5 give the same `.txt`). |
| `.png` | Drop from the commit, render on demand | It is a pure function of frame + profile + vendored fonts (`renderer.rs:306-344`). It catches nothing the cursor-trailer `.ansi` misses, and misses many things `.ansi` catches. It also churns with the tool: 0 of 7,550 committed PNGs match tuiscotti main pixel-for-pixel, though no app code changed. |
| `.html` | Drop from the commit, render on demand | It is the PNG in base64 plus the frame JSON (`svg.rs:251-266`), and makes up 80% of the store. Its byte gate also fails when only a hidden cursor's coordinates move (5,600 scenarios have one away from 0,0), which is a source of false failures. |
| `frame.json` | Rejected | It is the only fully lossless format, but averages 913 KB per scenario (6.89 GB of working tree). It can't be reviewed by eye, and it embeds absolute `argv` paths. |

## What each format captures

Measured with the probe for frame properties; non-frame rows come from the `Frame` struct and the replay test.

| Property | `.ansi` | `.txt` | `.png` | `.html` / `frame.json` | `.svg` | ascii | **proposed** |
|---|---|---|---|---|---|---|---|
| cell text | yes | yes | partial: tofu glyphs look alike (한/글, 😀/😃), NBSP looks like a space, hidden text not drawn | yes | yes | lossy | yes |
| wide / combining / emoji width | yes | content only | yes | yes | yes | lossy | yes |
| odd grapheme splits (leading zero-width char) | no | no | no | yes | no | yes | no (constructed test only; never occurs in the corpus) |
| fg/bg colour value | yes | no | yes | yes | yes | no | yes |
| indexed vs RGB vs default (same on screen) | yes | no | **no** | yes | no | no | yes |
| bold, dim, italic, reverse, strikethrough | yes | no | yes, but only on cells with a glyph | yes | yes | no | yes |
| style on blank cells | yes | no | **no** | yes | partial | no | yes |
| underline present / its colour | yes | no | yes | yes | yes / no | no | yes |
| underline style (double, curly, …) | yes | no | **no** (all drawn as one rule, `renderer/cell.rs:83-86`) | yes | yes | no | yes |
| underline colour on a non-underlined cell | yes | no | no | yes | yes | no | yes |
| blink | yes | no | **no** (frozen visible) | yes | no | no | yes |
| hidden: presence / content | yes / yes | – / yes | yes / **no** | yes | yes / no | – | yes / yes |
| cursor position and visibility | **no** | no | yes | yes | no | no | yes |
| cursor shape | **no** | no | yes | yes | no | no | yes |
| cursor blink | **no** | no | no | yes | no | no | yes |
| hidden cursor's coordinates | no | no | no | yes | no | no | no (deliberate: not visible) |
| style on a wide char's second cell | no | no | no | yes | no | no | no (0 occurrences in corpus) |
| grid size | yes | rows only | yes | yes | yes | rows | yes |
| trailing blanks, with or without background | yes | no | background only | yes | yes | no | yes |
| OSC 8 links, title, alt screen, mouse/keyboard/paste modes, scrollback, wrap flags | no | no | no | no | no | no | no |

The last row is not in tuiscotti's `Frame` (`frame/model.rs:13-26`), so no snapshot format can hold it.

## Evidence

All work is under `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/formats/`. The probe crate is in `probe/` (`src/minimal.rs` is the encoder and inverse parser); outputs are `pairs.md`, `replay.md`, `corpus.log` and `hashes-*.txt`. tuiscotti is cloned at `a47c9aa`.

- **37 single-property frame pairs.**
  - Every change `.png` detected, `.ansi` also detected, except the four cursor pairs: show/hide, move, shape, and blink (`.png` misses blink too).
  - `.ansi` caught 14 changes the PNG missed: NBSP, two tofu glyph swaps, indexed vs RGB, default vs explicit fg and bg, fg on a blank, bold on a blank, double and curly underline, underline colour on a non-underlined cell, blink, hidden text changes, and cursor blink (via the trailer).
  - Every pair's encoded frames round-tripped through the proposed format, except the deliberately constructed odd-split pair.
- **11 raw-stream pairs replayed through tuiscotti's emulator** (OSC 8, OSC 0/2, 1049, 1000/1006, 2004, kitty `>1u`, soft vs hard wrap, scrollback):
  - Each pair produced an identical Frame, so every format was identical. Only the emulator's separate state record differed.
  - The cursor pairs (hide via `?25l`, bar shape via `6 q`) showed up only in `.png`, `.html` and the proposed format, never in plain `.ansi`.
- **Full corpus of 7,550 frames**, taken from the JSON embedded in the committed `.html` files:
  - tuiscotti main's `.ansi` and `.txt` are byte-identical to the committed files in all 7,550 cases, so the format itself does not churn.
  - The proposed format round-trips with identical cells and visible cursor in all 7,550.
  - A PNG rendered from the decoded file is byte-identical to one rendered from the full frame in all 7,550.
  - Wide-char second cells never carry their own style (0 of 7,550). Wide chars appear in 110 frames and multi-codepoint symbols in 225.
- **PNG determinism.** Two fresh renderers give identical bytes. PNG and HTML hashes for 755 real scenarios are identical on macOS arm64, Linux arm64 and Linux x86_64 (Docker `rust:1.98.1`).
- **Caret regression demo** (`gate-demo/`): moving the caret one cell in `holla/parity/brew-services/80x24/truecolor`, the proposed gate fails. It wrote expected and actual PNG/HTML plus a diff PNG (score 0.9988). Today's plain `.ansi` gate would have passed this.

## Sizes

| | Current (4 formats) | Proposed |
|---|---|---|
| Average per scenario | 1,749,325 B (ansi 9,429 + txt 2,752 + png 344,170 + html 1,392,974) | 9,438 B |
| Working tree, 7,550 scenarios | 13.2 GB (html 10.52 GB, png 2.60 GB, ansi 71.2 MB, txt 20.8 MB) | 71.25 MB (about 185x smaller) |
| Git pack | 7.00 GiB | 9.40 MiB on a fresh repo after `gc --aggressive` |

Sample scenarios (ansi / txt / png / html, then proposed):

| Scenario | ansi | txt | png | html | proposed |
|---|---|---|---|---|---|
| `showcase/pages/overview/120x40/truecolor` | 18,474 | 3,143 | 576,696 | 2,096,580 | 18,480 |
| `holla/parity/brew-services/80x24` | 5,736 | 799 | 145,008 | 714,246 | 5,753 (visible cursor) |
| `jackin/capsule/menu/160x50/256` | 20,199 | 9,784 | 1,027,921 | 3,296,894 | 20,217 |
| `tablepro/query/close_confirm/100x30/16` | 8,344 | 3,724 | 179,220 | 953,960 | 8,350 |

## Keeping PNG/HTML as review artifacts

- Every check writes `target/tuiscotti/actual/<name>.ansi` and `.frame.json`.
- On a mismatch it also writes:
  - `expected.png` / `expected.html`, rebuilt from the approved file via `decode` → `Renderer::render_png` / `render_html`;
  - `actual.png` / `actual.html`;
  - a diff PNG from `tuiscotti_render::diff::compare_png(...).diff_png`.
- CI uploads `target/tuiscotti/` when the job fails.
- A small `render <name>` xtask can open any approved scenario as PNG/HTML locally.
- Pin the tuiscotti rev. A renderer upgrade then changes only these review images, never the baselines.

## tuiscotti support today

`GroupedStore` cannot gate on a subset:
- `check_with` (`grouped/check.rs:74-146`) reads all four approved files and fails as MissingApproval if any is missing (`check.rs:84-104`, `check_gates.rs:66-73`).
- It always renders and gates HTML and PNG (`check.rs:132-135`).
- `accept` copies all four (`mutate.rs:22-27`).
- The manifest and verdict hash all four (`seal.rs:60-75`, `110-135`).

The classic `Store` needs `.frame.json` plus `.png` (`snapshot/check.rs:219-240`). Neither stores the cursor in `.ansi`. Only the name listing already keys on `*.ansi` (`grouped/types.rs:168-215`).

**Do now: a local wrapper.** It needs no fork and is about 150 lines in `tests/visual_baseline/support.rs`, replacing `gate()` at line 493. It uses only public APIs: `render::ansi_dump`, `Renderer::{render_png, render_html}`, `diff::compare_png`, `Frame::to_json`, plus the encoder/decoder from `probe/src/minimal.rs`. Accept means copying the actual `.ansi`.

**Upstream PR to file, then switch to it:**
1. `tuiscotti-render/src/render/text.rs:46`: add `ansi_dump_with_cursor(&Frame)`, and add an inverse `formats::ansi::frame_from_ansi(&str) -> Result<Frame, FormatError>`.
2. `formats/ansi.rs:30` (`assert_normalized_sgr`): allow the cursor trailer at the end of the file.
3. `grouped/types.rs:71`: add `GroupedStore::with_artifacts(ArtifactSet::AnsiCursor)`.
4. `check.rs:84-104` and `check_gates.rs:66`: require and read only the configured files.
5. `check.rs:132-135`: in this mode, PNG/HTML are evidence only. On an `.ansi` mismatch, rebuild the expected frame with `frame_from_ansi` and write expected/actual PNG+HTML and the diff, reusing the diff-writing block at `check.rs:248-251`, without changing the status.
6. `mutate.rs:22-27` and `seal.rs:60-75,110-135`: copy and hash only the configured files.

## Functional comparison (hover, click, focus, scripted input)

- **Covered by the proposed format:** focus and hover highlight, selection (reverse or background), resize reflow, and the caret in text fields, including the hidden-cursor state (via the trailer).
  - Visible-cursor scenarios by group: holla/parity 575, holla/flows 575, holla/concept 175, showcase/flows 175, jackin/capsule 125, tablepro/query 100, …
- **Covered by no snapshot format:** terminal modes. The app's `src/runtime.rs:110-112,159-161` enables alternate screen, mouse capture and bracketed paste. It sets no title and emits no OSC 8 links. A broken mouse mode still shows up indirectly: hover/click screens change, and tuiscotti's mouse API fails with `ModeNotEnabled`.
  - `pointer.rs:44-50` `mouse_move` writes raw mouse bytes and bypasses that check, so for hover-by-move only the changed screen catches a regression.
- **Recommended addition:** one boot test per app asserting `Observation.state.modes` contains 1049, 1003/1006 and 2004. The live path does report modes and title; it reports hyperlinks and scrollback as Unsupported (`tui_shell/state.rs:116-130`). This is a test, not a stored file per scenario.