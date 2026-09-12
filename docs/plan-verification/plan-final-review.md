# Independent final plan review

Scope: current `holla-fable` files only, September 10, 2026. This is a
read-only product review of `IMPROVEMENTS_PLAN.md` and its supporting reports.
No product, test, baseline, or plan edits were made by this reviewer. Findings
below distinguish completed evidence checks from future implementation proof.

**Final verdict:** approved as a research/implementation plan, not as completed
implementation. The owner addressed all review findings, and the completed
research, conditional-opportunity, dependency, acceptance and traceability
sections were reread. No unresolved factual issue was found in this bounded
review. Final document-link/diff checks remain the integration owner's gate.

## Review basis

Reviewed F01–F23, the previous-finding disposition map, baseline/method claims,
and supporting API, interaction, text, design, ecosystem, terminal and prior-fix
reports. The [prior-fix ledger](plan-prior-fixes-verification.md) supplies the
complete 17-row + six-group + nine-editor-row crosswalk and the fresh current
fmt/Clippy/328-test/Rustdoc gates. Those checks were not rerun redundantly for
this final document review; the product source remained unchanged.

Independent work in this review:

- Read the complete `/tmp/plan-terminal-suspend.py` diagnostic, traced its
  process/PTY/signal ownership, and compared the reported observation with F21.
  Did not duplicate the live suspension experiment.
- Freshly repeated selected-font mask comparison: the actual configured
  `JetBrainsMonoNerdFontMono-Regular.ttf` rendered 東, 京 and ☕ with the same
  `(9, 11)` bitmap as the missing-glyph sentinel; ASCII A differed.
- Freshly repeated `ansi2html.convert("東京", 4, 1)`: HTML adds two spaces
  after text already occupying four application cells.
- Inspected `/tmp/plan-terminal-overflow.png` and its source rendering path:
  the two-column `ABCD` diagnostic paints C and part of D into the right margin.
  This image came from the terminal reviewer; it was not regenerated here.

The independent public-API Grid/Table reproductions are already recorded as
C01–C04 in the prior-fix ledger. They prove cursor source drift, sorting against
stored rather than visible pending values, and accepted ragged-row sort panics
in both widgets. They are separate from the earlier fixed edit source identity.

## Findings and requested dispositions

| ID | Review finding | Required disposition / scope |
| --- | --- | --- |
| FR01 | The initial plan draft called the 250 base capture sets “default.” `audit_shots.sh` defaults to seven fixtures × five sizes × five modes = 175; the recorded 250 used ten explicitly selected fixtures. | **Resolved:** final wording says representative sets, and the acceptance gate names the seven-fixture default plus explicit ten-case recipe. This is wording/provenance accuracy, not a missing artifact or failure of the existing 301-set count. |
| FR02 | Terminal report TV3 adds two confirmed renderer defects beyond grapheme width and missing fonts: HTML padding counts scalars, and PNG drawing does not clip at declared columns. F22 initially named only width/font errors. | **Resolved:** F22 explicitly includes both reproductions, shared PNG/HTML reference geometry and no painting into padding. Merely fixing fonts or grapheme width would not remove those independent defects. |
| FR03 | The temporary suspension diagnostic's successful path supports F21, but its initial description overstated bounded cleanup. Startup report parsing was outside `try`, some `waitpid` calls have no deadline, error cleanup does not reap, and final cleanup signals numeric PIDs after normal reaping. | **Report correction accepted by terminal owner.** Treat this as a one-off observed-success diagnostic, not a safe retained harness. Before retaining it, guard startup, track live child ownership, impose a total deadline, reap all owned children, and never signal an already-reaped PID. No successful observation is invalidated by this qualification. |
| FR04 | F06 initially attributed independent corroboration only to source review. Current independent public-API execution also confirms cursor identity and pending-value ordering. | **Resolved:** F06 and traceability now name independent execution, including pending values. They do not claim corruption of source-keyed pending/check stores; those retain identity. |

No product fix is authorized by these review findings. FR01/FR02 concern plan
accuracy/completeness; FR03 concerns the safety requirements for future retained
verification. F21 and F23 now state the harness-hardening obligation explicitly.

## Suspension proof: exact independent review

The diagnostic allocates a fresh PTY, fixes its size to 80×24 and snapshots the
full slave termios. Its supervisor creates a new session, attaches that PTY
with `login_tty`, forks Showcase into a distinct process group, and makes that
group foreground before exec. The supervisor remains in the same session,
preventing the orphaned-process-group exception from invalidating the SIGTSTP
test. Showcase restores the default SIGTSTP/SIGTTOU dispositions before exec.

The outer diagnostic waits until canonical mode is off, sends SIGTSTP to the
exact owned application PID, receives a supervisor report based on
`waitpid(..., WUNTRACED)`, and samples termios after the supervisor takes
foreground ownership. It then asks the supervisor to return foreground to the
application and send SIGCONT, sends `q` through the PTY, and inspects final
termios after normal exit.

The terminal owner's reported successful result is internally consistent with
that code and current `TerminalSession` source:

| State | Canonical | Echo | Full termios equals original |
| --- | --- | --- | --- |
| Running application | false | false | false |
| Application confirmed stopped; supervisor foreground | false | false | false |
| Continued application then normal exit 0 | true | true | true |

F21 correctly describes **external SIGTSTP suspension**, not Ctrl+Z key
handling. The program does not touch the user's controlling terminal or signal
the user's process group. It demonstrates unusable inherited raw modes while
the application is stopped; it does not demonstrate broken normal teardown.

This review did not independently execute that PTY flow. Independent
corroboration is source/topology/result-trace review, not a second reproduction.
The experiment does not assert alternate-screen/paste/mouse/wrap reset traces,
resumed image correctness, resize while stopped, repeated suspension,
post-initialization I/O failure, or every platform/signal path. F21 lists these
as acceptance work, appropriately. SIGKILL and SIGSTOP cannot be caught for
pre-stop restoration. A lost output device cannot receive recovery escapes.

The temporary harness limitations in FR03 do not justify running broader
signals or live-terminal experiments. They are requirements for turning the
one-off diagnostic into reliable automated evidence.

## Prior findings: completeness cross-check

No original fixed group was lost from the current plan's disposition families
and F23 proof obligations:

| Previous report groups | Current plan destination |
| --- | --- |
| TextBuffer offsets/selection/word movement/atomic joining/newline normalization | Previous-findings first row; F23g retained deterministic proof; preserve invariants throughout F10–F14 |
| CodeEditor modified Enter, Unicode find/fuzzy, paste/Backspace, set_text refresh | Previous editor rows; F08 shared binding contract; F23h missing modifier and nonzero find-index assertions |
| CodeEditor clipping, read-only focus, dynamic read-only indentation | Previous editor rows; F04 explicitly separates Grid's independently implemented permission defect |
| DataTable replacement and edit source identity; table/grid invalid-cell/header transitions | Previous table row; F03/F05/F06/F07 address different collection/shape boundaries |
| TextInput required validation, masking, narrow cursor/click geometry; TextArea long-line editing | Previous input row; preserve tested geometry while F14 defines literal controls |
| Editor manual scroll and resize | Previous manual-scroll row; F23h adds missing editor pointer-scrollbar proof |
| Runtime startup guard, queued-event freshness | Previous runtime row; F21/F23b/F23c extend rather than reclassify those fixes as absent |
| Minimum Forms/sidebar | Previous Forms row; F16/F17 identify other sections and zero-area writes |
| Semantic gutters, reverse selection, disabled DIM, actual NO_COLOR backend | Previous monochrome row; F22/F23f separate renderer/backend/emulator proof |
| Choice containment, stale hidden hits, footer priority, duplicate Tab hints | Previous choice/footer row; F23a state/route assertions |
| First/second click, exact drag press, Facts acknowledgement mouse | Previous click/dialog row; F23h adds actual Facts paste, F01 handles separate app modal ownership |
| TablePro duplicate chord | Previous Ctrl+D row; F08 preserves alias and Data/Structure ownership |
| Idle viewport cache | Previous idle-cache row; F15 explicitly distinguishes repeated tail replacement and dirty work |
| Diff catalogue/page/mode cache/narrow fallback | Previous Diff row; no second component; F09 handles a different app copy transport defect |
| Cell-window Unicode mapping, Diff style/tab/scrollbar geometry, one-cell wrap | Previous final primitive row; F10 distinguishes arbitrary cross-style viewport input, F17 preserves zero-allocation ownership |
| API collection ownership, shell duplication, keyboard viewport selection, optional frameworks | Remaining disposition rows; F01–F07/F19 and conditional decisions, without assuming a global framework is required |

Subsidiary editor E01–E09 are all covered by those families plus F23h; in
particular the easily missed E05 nonzero find-index reset is explicit there.
Old historical severity inconsistencies are resolved by the plan's stated
impact taxonomy, not counted as additional independent defects.

## Evidence boundaries that the plan preserves correctly

- **328 tests / 460 frames / 301 captures are different quantities.** One
  baseline test checks 23 pages × five sizes × four palettes. The 301 artifacts
  represent 250 matrix combinations plus 51 interaction states, not 301
  independently exhaustive scenarios. Hashes exclude sidebar and cursor/
  underline-color metadata; they do not traverse hidden controls.
- **Inventory counts name their method.** The API owner verifies the current
  library source digest against compiler-derived exported entries. The plan
  distinguishes 1,573 source-backed entries from 572 derived contracts and
  rejects the old incomplete regex count. This reviewer checked the reported
  categories/disposition, not reran the inventory extraction.
- **Historical versus fresh is explicit.** The 100,000-operation stress probe,
  old before/after timings and earlier PTY/burst runs are not called retained
  current regressions. Fresh fixed-only idle measurement does not establish a
  new old-versus-new speedup. F15 correctly states that only the first tail
  replacement changes content; repeated identical requests still cause work.
- **Panic probes are qualified by reachability.** F03/F02 have existing real-app
  flows. F07 is accepted malformed API input without a current app producer,
  and F04 is a public read-only transition without a proven current app trigger.
  Their APIs still need defined behavior; no synthetic current-app failure is
  invented to strengthen priority.
- **Source traces are not mislabeled live runs.** F09 explicitly says Inspect
  copy is verified by the complete dropped-event path, not a selected full-app
  replay. F23d calls flood fairness a source-supported risk, not a measured
  freeze. F20 is an absent proposed capability, not a broken promised search.
- **Terminal boundaries stay separate.** Actual NO_COLOR uses a real fresh
  backend process; Mono buffer tests are not equivalent. A tmux screen capture
  is interpreted output, not the application's original raw stream or a named
  direct-emulator screenshot. Missing glyphs and rasterizer geometry are not
  assigned to Rust widgets without matching evidence.
- **No scope expansion into external writes.** Inspect copy remains the
  preview clipboard; matrix import and real OS clipboard are product decisions,
  not automatic extensions of paste/copy fixes. The plan authorizes no product
  implementation, dependency installation or baseline blessing.

## Conditional decisions verified in the final appendix

The plan's research/dependency/ledger appendix was still being assembled when
this review began. The final appendix was subsequently reread; the following
source-report dispositions remain visible without being promoted to new
confirmed application defects:

- TXT-08: one-cell pasted TSV normalization versus an explicitly authorized
  matrix-import feature; no matrix-paste contract presently promised.
- PLAN-API-06: preserving numeric Picker cursor across asynchronous replacement
  might retarget it, but no current in-modal Holla insertion flow was proved.
  Picker query refresh intentionally resets to an eligible first result.
- Local sort of fully loaded data and fetch-more are separate contracts; the
  observed cursor/value ordering defects do not justify claiming globally
  sorted not-yet-loaded data.
- Real async worker cancellation/stale response ownership, dynamic Widget/
  Container/Theme traits, configurable glyphs, virtual data adapters, OS
  clipboard and optional terminal protocols require concrete consumer proof.
- Existing operation state remains durable/inspectable; external task-system
  examples do not imply a new task manager, toast or notification widget.

These map to O01–O07, the F06 fetch-more boundary, the extended research section
and the explicit component exclusions. The final Kitty clipboard row correctly
names **OSC5522** as an extension rather than assigning its status/permission
protocol to legacy OSC52. Research analogies remain separate from executable
local defect evidence.

This review finds no lost confirmed product finding in F01–F23. All four review
items have an explicit accepted disposition. Completion of this review does
**not** mean any planned product defect has been fixed, any temporary probe has
become a retained regression, or every terminal/platform has been verified.
