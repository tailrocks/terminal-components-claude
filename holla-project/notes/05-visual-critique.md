# 05 · Visual critique of the Holla frames

Scope: 58 captures in `shots/` read as images (all four sizes of
`h_rust_dirty`, `h_docker_cleanup`, `h_disk_cleanup`, `h_upgrade_plan`,
`h_activities_multi`, `h_hard_cases`; both `_mono`; `h_first_use_80x24`,
`h_remote_host_100x30`; 28 `h_flow_*`/`h_p3_*` frames; `t_80`, `t_danger`,
`t_conn_prod`, `t_switcher`, `j_cockpit_running` for family comparison).
Colour claims below were checked against the `.html` captures (every span's
fg/bg), not eyeballed. Rules applied: DESIGN.md (Overview, Colors, Typography,
Layout, Shapes, State grammar, Do's and Don'ts), CONCEPT §6/§7/§16,
02-interaction-models §2 and §4.

Verdict in one line: the bones are right (finder rows in fixed columns, props
preview, gate pages + facts dialog, plan rail, tabs that run) and the mono
frames survive; what fails is discipline at the edges — the live-activity
green is missing where the spec puts it, safety tones are used as labels,
paths and states are repeated instead of resolved, and the 80/120-column
frames lose the one fact (destructive) that §16 says must never be lost.

## Blockers

**B1 · Spinners are not in primary.** `h_disk_cleanup_*` header row 6
(`⠋ scanning · 9 candidates so far`) and status bar (`⠋ scanning ~/work ·
66%`) draw the spinner in `#b3b3b3`; `h_activities_multi_*` status centre
`⠋ 4 running`, `h_flow_discovering` `⠋ discovering github · 8 of 9`,
`h_flow_logs`, `h_flow_btm_attached` likewise. In `h_activities_multi_120x40`
three green `⠋` sit in the tab strip (row 3) and a grey `⠋` sits in the status
bar (row 39) — the same glyph in two tones on one screen. DESIGN: the spinner
is "the one live-activity use of green"; the model's own sketch puts the green
`⠋` in the StatusBar centre. Fix: `tone = primary` for every spinner, no
exceptions.

**B2 · The destructive fact disappears with width.** `h_docker_cleanup_80x24`
and `_120x40` drop the `destructive` column entirely (present at 100×30 and
160×50); at 80×24 the summary line row 22 reads `→ docker stop acme-api-1
acme-worker-1 acme-scheduler-1 acme-db-1 acme-red…` — the tail
` · on devbox · destructive` is what got cut. The only surviving signal is the
`…` on `Enter Review…`. §16 requires "whether confirmation or trust is
required" at every size; 4.2 says type+scope never drop — risk must join
them. Fix: a one-cell risk glyph column that never drops, and the summary
line truncates the command in the middle, never the ` · scope · risk` tail.

**B3 · Preview props wrap paths mid-word and repeat them.**
`h_hard_cases_120x40` rows 15–21: `northwind-traders-platf` / `orm-migration/
services/i` / `nventory-reconciliation-` — the same path wrapped three times
(Runs, In, Scope). `h_flow_child_query` repeats `~/work/acme/apps/fronten` +
`d` six times (Does, In, Scope, Why, Changes, Gate). `h_docker_cleanup_120x40`
spends 22 wrapped rows on `Runs`, pushing In/Host/Scope/Why/Changes/Risk/Gate
below the fold — the lines that answer §7 are invisible without scrolling.
DESIGN: `truncate_middle` keeps the tail of identifiers; code previews are
capped at six lines with `… N more`. Fix: paths never wrap; `Scope` says
`here` when it equals `In`; `Runs` capped at six lines.

**B4 · Failure paints half the screen red and evicts the hints.**
`h_flow_upgrade_failed`: red on the header clause (`· 5 of 7 done · 1 blocked
· 1 failed`), the bar, row 06 (correct), row 08 `blocked · 06 fa…`, the
summary row 38 (`succeeded 5 · failed 1 · excluded 1 · skipped 0 · never
started 1`, all red) and a 70-cell red status sentence in the hint bar that
leaves only `↑↓ Step …` — `r` retry (4.11) is unreachable by discovery.
Blocked is red here and in `h_flow_upgrade_excluded` (`blocked · needs 0…`)
although 4.11 and the StepRail spec define blocked as faint `·`. DESIGN failed
state: `!` in the tab, `failed · …` status, the failed row — that is all.
Fix: red on `!`, the failed row and the `1 failed` clause only; summary and
status in text-secondary; status ≤ 30 cells (`Upgrade everything failed · 06
exit 1`); blocked = faint.

## Should fix

**S1 · Cursor row is over-marked.** Every finder frame draws the cursor row
as `▎` + `›` + accent-tint + bold *and* a second `▎` on the query row
(`h_rust_dirty_80x24.txt` rows 5 and 8). DESIGN: focus = `▎` + bold;
selected+focused = marker + bold + tint. TablePro's picker (`t_switcher`) uses
`▎` + bold on the cursor row and `▎` on the query — no `›`, no tint. Keep the
query `▎` (it is where typing goes), mark the cursor row with `›` + tint, drop
its `▎`. Same on the plan outline (`h_upgrade_plan_*` row 10: `▎[✓] 01
Preflight` on tint).

**S2 · Amber as identity and label.** `h_remote_host_*`, `h_flow_pg_blocking`,
`h_flow_system`, `h_flow_remote_gate1/2`: `◆ prod-eu-1 · production` in
`#f59e09` in the menu bar, plus `over SSH`, `privileged` in amber. TablePro
(`t_80`, `t_conn_prod`) draws `◆ production` in text-primary; the glyph is the
identity. Every `h_rust_dirty_*`/`h_monorepo_child_*` status bar paints the
whole `main ↓3 • 5` amber, and `h_hard_cases_160x50` paints a 70-cell
`feature/NWT-4821-…-invoices ↓14 ↑3 • 7 · rebase`. Amber belongs to `↓3 • 5`
and `rebase`, not to `main`. Also amber mode labels in the status centre:
`review · gate 1 of 2` (`h_flow_cleanup_gate1`, `h_p3_docker_gate1`), `trust
review · one exact file` (`h_flow_trust`), `attached` (`h_flow_btm_attached`)
— but `review · 8 steps · 2 branches parallel` is not amber. Decide once; the
DESIGN precedent (Safe Mode token) argues for none of them.

**S3 · Branch item drops all-or-none.** `h_hard_cases_80x24/100x30/120x40`
status bars show only the path — no branch, no `rebase` — while 120×40 has
~60 free cells; 160×50 shows the full 70-cell branch. Truncate the branch
name (`feature/NWT-4821-…-invoices ↓14 ↑3 • 7 · rebase`) before dropping it;
`rebase` is the fact a reader needs first.

**S4 · Red `destructive` tags at rest.** `h_docker_cleanup_100x30/160x50`
(rows 8, 12–14), `h_flow_scope_system` (3×), `h_flow_scope_children`,
`h_p3_docker_root`: four red words right-aligned on a results list are the
loudest thing on the screen, louder than focus. DESIGN: red is never used for
routine destructive affordances at rest beyond a danger label. Suggest: the
word in text-secondary, or a glyph column (`!`-free, e.g. a `•`-class marker
is already taken — use the word) and red only on the preview `Risk` line and
`Continue…`. Whatever the treatment, it must survive width (see B2).

**S5 · Wasted space and detached anchors.** `h_upgrade_plan_100x30`: 8-row
list, then 12 blank rows, then the consequence sentence at row 27;
`_160x50`: sentence at row 46. `h_flow_args`: `Cancel`/`Run` at row 38, 25
rows under the Command card. `h_disk_cleanup_120x40/160x50` and
`h_upgrade_plan_*` preview cards are full-height `#111111` planes for 5–12
lines (160×50: 37 rows of plane for 12 lines). `t_conn_prod`'s detail card
ends after its content. Put the sentence one blank row under the list; let
cards end after content; keep action rows near the thing they act on.

**S6 · State said twice, never as geometry.** `h_activities_multi_*` Running
rows: `api dev · running | activity | child | running · 3 s`,
`btm · detached | activity | host | detached · 3 s` — state in the label and
in the reason, no `⠋`/`✓` glyph in the row (the sketch has `⠋ 12 min`). In
mono the rows lose nothing only because there was nothing to lose.
`h_flow_logs` status centre repeats the header row 5 verbatim.
`h_flow_btm_attached` says `Ctrl+] detaches` three times (frame footer row 37,
hint, status sentence) and the header right reads `host · mbp · mbp`.

**S7 · Disk usage page (`h_disk_cleanup_*`, `h_flow_disk_80`).** Heading
`Largest under ~/work` lists `~/Library/Containers/…`, `~/Library/Caches`,
`~/.cargo` — not under `~/work`. Header says `67%`, status bar says `66%` in
the same frame. The Filesystems section duplicates the status-bar meter and at
80×24 pushes the checkbox sections below the fold (`h_flow_disk_80` shows 3 of
9 candidates; the actionable rows are the ones hidden). Faint (disabled-look)
rows `[ ] android › .gradle` carry amber `activity unknown` + red `in use by
gradle daemon …` — disabled and alarmed at once. Mode column mixes `./gradlew`,
`cargo`, `pnpm`, `Trash`, `permanent` — tool names are not a mode. 120×40
preview title `work` for row label `~/work`. `Space Select` is offered on
`Largest` rows that have no checkbox.

**S8 · Plan outline and its dialogs.** `h_upgrade_plan_*`: fourth column
(`sudo` / `optional` / `excluded`) has no header under `step needs lane`;
lane column mixes `·`, `a`, `b`, `join` while the preview says `trunk`.
`h_flow_upgrade_excluded` preview: `State excluded · excluded`.
`h_flow_upgrade_confirm`: `Host` label twice (rows 14 and 16); the dialog's
confirm button is a red danger `Start plan` while the page's button is the
green primary `Confirm plan` — one risk level per action (4.10: `Continue…`
only when a step is destructive). `h_flow_upgrade_running`,
`h_p3_docker_running`: `14%` sits alone on row 6 under a bar that spans row 5;
DESIGN progress is `label ━━━━──── 14%` on one row.

**S9 · Hints.** `h_upgrade_plan_*`: `Enter To confirm` next to `c Confirm` —
same verb, two keys, capital T. `Space Include / exclude` is 21 cells and at
80×24 (`h_flow_upgrade_80`) evicts `p Facts` — the only key that opens the
facts drawer is the one not shown. `h_p3_docker_done` status: `Clean Docker
completely finished · 7 succeeded · 0 failed · 0 excluded · 0 skipped · 0
never started` (four zero clauses, 78 cells, hints reduced to `↑↓ Step …`).
`h_flow_scope_children` status `Scope: the children scope`,
`h_flow_scope_system` `Scope: system scope on mbp` — say nothing.
`h_flow_btm_attached` hint bar carries a sentence (`Attached to btm · keys go
to the program · Ctrl+] detaches`); DESIGN: a hint is `key Action`.

**S10 · Truncation quality.** Summary line always cuts the tail
(`h_docker_cleanup_80x24` row 22, `h_hard_cases_80x24` row 22: the path wins,
` · here · read-only` loses). Reason cells cut mid-word: `since the las…`
(`h_rust_dirty_80x24`), `needs 0…` (`h_flow_upgrade_excluded`), `06 fa…`
(`h_flow_upgrade_failed`), `contin…` (`h_hard_cases_80x24`) — truncate at the
last ` · ` that fits. Tab crumb `Clean Docker co…ly` (`h_p3_docker_gate1`) —
truncate_middle inside a crumb is unreadable; drop middle crumbs
(`Here › … › Review`) first. Viewport lines clipped with no `…`
(`h_p3_docker_running` row 9 ends `acme-schedu`). Page title `… then continue
deliberate…` (`h_flow_cleanup_gate1` row 5): the subtitle clause should drop
whole. Duplicate labels become indistinguishable when truncated:
`Start development ecosys…` ×2 (`h_flow_child_query` rows 14–15, parent vs
sibling).

**S11 · Done and failed states do not land on the result.**
`h_p3_docker_done`: cursor and viewport stay on `01 Stop running containers`;
the payoff (`07 docker system df`, reclaimed GB) is not shown; no follow-ups
(CONCEPT §6.8: outcome, duration, affected scope, failures, follow-ups).
`h_flow_upgrade_failed`: viewport shows Preflight's output, not the failure;
4.11 says the frontier row is the failed one.

**S12 · Shell rhythm.** `h_flow_system` row 38 `Next  ▎Open btm  ▎Stop a
process…` sits directly on the status bar (row 39) — no blank row. The `→`
summary line in every finder frame also sits directly on the status bar
(`h_rust_dirty_80x24` rows 22–23); the elevated plane separates it, but §3 of
the model says `body · blank · StatusBar`.

**S13 · One fact, three labels.** `In` (preview) / `Effective cwd` (gate 1) /
`Runs in` (gate 2); `Not touched` (`h_flow_cleanup_gate1`) / `Not affected`
(`h_p3_docker_gate1`); scope readout `scope ‹ mbp ›` vs heading `System · mbp`
vs status `scope system · mbp` (`h_flow_scope_system`). Gate 2 for Docker has
no `Risk` line, Gate 2 for the remote restart does.

**S14 · Empty preview.** `h_flow_pg_blocking`: list empty state is right
(`No matches for "…"` + `Esc clears the query · Ctrl+↑ widens to the parent
scope`) but the preview card shows `Preview` / `Nothing selected` / `↑↓ moves
the cursor` — names a key that does nothing. Blank plane or nothing.

**S15 · Explore vocabulary.** `System` is a chip (row 18) and a row (`System ·
mbp | explore | host`) in the same section (`h_rust_dirty_*`,
`h_activities_multi_100x30+`); `Children · 3 projects` exists only as a row.
Two encodings of the same list.

**S16 · Args form command.** `h_flow_args`: action `Clone a GitHub
repository…`, live command `gh auth status --active --owner alex-dev
--protocol ssh --destination ~/work/holla`. Either a fixture bug or the wrong
preview; §6.7 says the resulting command is previewable — this one is not the
command that runs. Also `▎`/`*` fine; two blank rows after `owner` (rows
10–11) vs one elsewhere.

## Nits

- Copy: `1 tools missing` (`h_activities_multi_*`); `refresh debian metadata`
  / `clean docker completely` lower-case proper nouns (`h_upgrade_plan_120x40`
  Unlocks, `h_p3_docker_done` status centre); Alt+Enter lists mix case (`Fetch
  only · Push · pin · alias · why here?`); `Gate two gates · typed phrase with
  I UNDERSTAND · type I UNDERSTAND: …` says it twice (`h_docker_cleanup_160x50`);
  `Trash ~/Library/Logs › Logs`; `named volumes hold durable data; a generic…`
  uses `;` where the system uses ` · `; `following` as frame meta for an
  attached btm; `4 lines` over 3 rows + header (`h_flow_system`); `upgrade
  Debian packages…` subtitle starts lower-case under a sentence-case title.
- `h_flow_picker`: `▶` is not in the glyph table (running is `⠋`); the group
  label `activities` appears on the first row only; `Delete` stop and
  `Alt+Enter` restart (4.13) are not hinted.
- Bold on running rail rows 02/05 (`h_flow_upgrade_running`): bold means
  keyboard/heading; the rail spec bolds the failed row only.
- `[✓]` faint (required, locked) vs `[✓]` green (optional, checked) in
  `h_upgrade_plan_*` collapses to the same glyph in mono — no mono frame of
  the plan page exists to prove otherwise; use `[–]` or `[✓]`+faint label for
  locked.
- 100×30 has no preview (`h_rust_dirty_100x30`, 12 empty rows) although the
  model's 100×30 sketch shows one and the drawer rule says `<100`. Off by one
  somewhere.
- Harness artefacts: query `restart paymentsblocking` (`h_flow_pg_blocking`,
  two queries concatenated), every activity `3 s`, `truecolor · W×H` present
  in `h_rust_dirty_80x24` and absent in `h_hard_cases_80x24`, stale
  `Cancelled · nothing was executed` under the help dialog.
- `h_flow_alternatives`: the query `▎` and the row tint vanish while the menu
  is open; DESIGN says anchored popups leave focus with the owner. Acceptable
  as attention management; note it is a deliberate deviation.

## Axis summary

**1 · Hierarchy and density.** Keyboard destination: found in <1 s on every
finder and page frame because of the tint bar — but by over-marking (S1). At
80×24 the first result row is row 8 (menu, blank, tabs ×2, query, blank,
heading); TablePro's first data row is row 6. Headings (faint, sentence case,
blank row before) read well; rows are one cell high with 2-cell gaps; chrome
is quiet except the status bar's `truecolor · W×H`. Nothing is boxed twice.
Frames are used only for viewports (Sequence, merged logs, btm screen,
Definition, Processes) — all legitimate tab-body/viewport edges. Density fails
by emptiness, not clutter (S5).

**2 · One-hue discipline.** Green per frame, from the html: lockup fill,
active-tab `━`, query `▎`, cursor row `▎›` + tint, tab-strip `⠋`, primary
button (`Confirm plan`, `Run`, `Trust this file`), `EDIT` badge, required
`*`, checked `[✓]`, 100 % `━` + `✓` on the done plan. All allowed. Missing
green: every StatusBar/header spinner (B1). Red outside safety: the
`destructive` tag at rest (S4, arguable), `blocked` (B4), whole summary and
status sentences on failure (B4), the confirm dialog's `Start plan` (S8). Amber
outside safety: host identity, `over SSH`, `privileged`, branch names, mode
labels (S2). Blue: only the anchored menu cursor row (`h_flow_alternatives`)
— correct. Meters: red/amber by threshold (`/ 91%`, `Memory 88%`) are the
Quota meter's own tones — fine.

**3 · State legibility.** Focus vs selection: fine but redundant (S1). Hover:
not captured. Destructive: Gate 1 (Cancel focused, `Continue…` danger) and
Gate 2 (typed phrase, `Execute` enabled only when typed, EDIT badge) match
`t_danger` exactly — the strongest part of the set. Loading: `⠋` present but
grey (B1); partial: `9 candidates so far`, `discovering github · 8 of 9`
good. Failure: too loud (B4) and not focused (S11). Empty: list empty state
good, preview empty state wrong (S14), `nothing used here yet` inline is good.
Mono: `h_rust_dirty_mono`, `h_docker_cleanup_mono` survive — `▎`, bold, `›`,
`•`, `destructive` as text all carry; predicted failures are the locked
checkbox (nit) and any state that is colour-only in the status bar (branch
amber, `attached` amber — both survive as text).

**4 · Responsive.** 80×24: hints never cut mid-word; `…` marks the cut;
columns starve correctly (reason first) but the summary line truncates the
wrong end (S10) and the destructive column vanishes (B2); the disk page hides
its actionable section (S7); `p Facts` lost on the plan page (S9). 100×30:
no preview, 12 empty rows (nit). 120×40: preview column too narrow for
paths (B3); status bar drops git state with room to spare (S3). 160×50:
full-height empty planes (S5), list uses 60 % of the width and the rest is a
plane with 12 lines. Nothing overlaps or clips except viewport lines without
`…` (S10).

**5 · Does it feel like Holla / the family.** Path and host: obvious (menu
bar right, status bar left; `acme › frontend · Node project · pnpm` is
excellent). Why a row appears: obvious on Suggested rows; not on scope pages
where 10 rows say `defined by …/mise.toml` (`h_flow_scope_children`) and 22
rows say `host` (`h_flow_scope_system`) — the column is present but carries
no information there. Scope on every row: yes. What Enter does: yes — the
hint verb changes (`Run` / `Open` / `Review…` / `Trust…`) and the `→` summary
line is the best line on the 80-column screens; it deserves better
truncation. Running work: tab spinners yes; Running rows no (S6); status bar
grey (B1). Family: same tabs, hint bar, props, dialogs, rail anatomy as
TablePro/jackin; holla spends one more chrome row (menu bar + status bar vs
TablePro's single identity strip) and is a shade louder (S1, S2, S4).

**6 · Copywriting.** Sentence case: yes everywhere except proper nouns
lower-cased by a slugging pass (nits). ` · ` joins: consistent; one `;`.
`…`: correct on buttons (`Review…`, `Continue…`, `Trust…`) and on truncation;
wrong on a page title (`Clone a GitHub repository… · arguments`). Past-tense
statuses: `Upgrade everything started`, `Cancelled · nothing was executed`,
`Clean Docker completely finished` — correct, but the finished/failed
sentences are too long (S9). Redundant: state twice per Running row (S6),
`Host` twice (S8), `excluded · excluded` (S8), `mbp · mbp` (S6), `Scope: the
children scope` (S9), the Gate line that repeats the phrase (nit), `System`
chip + row (S15), the `Kind required` prop under a title meta that already
says `required`.

## Order of work

1. B1 spinner tone (one token change, every frame).
2. B2 + S10 truncation policy for the summary line and reason cells; risk
   glyph column that never drops.
3. B3 props: `truncate_middle` on paths, `Runs` ≤ 6 lines, `Scope` = `here`
   when it equals `In`.
4. B4 + S11 failure/done treatment and status length.
5. S1, S2, S4 tone discipline (cursor row, amber identity, red tags).
6. S5 layout anchors; S7 disk page; S8/S9 plan page and hints.
7. Remaining should-fix copy and the nits.


## Response (after the fixes)

Applied: B1 (`StatusItem::busy`, spinner in primary everywhere), B2 + S4 +
S10 (risk column never drops, plain text-secondary; reasons and plan meta
truncate at ` · `; the summary line truncates the command in the middle;
breadcrumbs drop middle crumbs first), B3 (paths and commands
`truncate_middle`, `Runs` ≤ 6 with `… N more`, `Scope` says `here` when it
equals `In`), B4 + S11 (only the failed row and the `1 failed` clause are
red; blocked is faint; status ≤ 30 cells; the cursor lands on the failed or
last step; `Next` follow-ups on finished plans), S1 (cursor row is `›` +
tint), S2 (host identity text-primary; the git state after the branch is
the only warning status item; mode labels secondary; `attached` chip
secondary), S3 (branch and state are two items, the state outlives the
name), S5 (consequence line under the list, preview and step cards end
after content, argument buttons under the command card), S6 (Running rows
are `name · activity · scope · ⠋ running · 3 s`; activity status centre is
the elapsed time only; attach hint once), S7 (`Largest on /`; one rounded
percentage; Filesystems dropped below 22 rows; skipped candidates fully
quiet; `Space` only on candidates; card title is the path), S8 (`note`
header, `· trunk` lane text, `State` deduped, `Host` once, primary `Start
plan` for non-phrase plans), S9 (`Space Toggle`, no `Enter To confirm`,
short scope status, short attach status), S12 (one blank row above the
status bar), S13 (`In` everywhere, `Not touched`, `scope ‹ system ›`), S14
(empty preview names the key that helps), S15 (System is a chip only in
Explore), nits (plural `1 tool missing`, proper nouns kept in `needs`,
`⠋` in the picker, running rail rows not bold, capitalised menu labels,
gate line once, cleanup label `Trash ~/Library/Logs`, mono plan frame,
viewports wrap).

Kept, with reasons (D-23): the 110-column split, the outline's `▎`, the
warning tone on gate `Uncertainty`/`Privilege`. The `14% alone on row 6`
observation is a PNG artefact: the `.txt` shows one row; the box glyphs
draw at the baseline in the capture font.

Not addressed: `4 lines over 3 rows + header` (could not be located),
lower-case subtitles (intents are sentences reused as prose), the
harness-only artefacts (`3 s` everywhere in paused frames, a status that
never expires while the clock is paused).
