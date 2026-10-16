Fresh captures of the current branch confirm the product owner's concern. Most screens use green for only the brand chip and the `▎` focus bar. Where colour does appear, it is mostly red and amber flooding whole lines rather than marking the state. Three things are wrong at the root:
- **Unused tokens:** the apps never use the `success` or `info` tokens, and there is no `Tone::Info` or `Tone::Active` to reach for.
- **Grey states:** live and running states, finished work and healthy quota are drawn in grey.
- **Whole-row alarms:** error and warning tones are applied to entire pre-formatted rows instead of to the cell that carries the state.

**How I measured.** The approved `.ansi` files for holla and jackin are stale (approved 2026-09-14). The theme changed on 2026-10-16, which added `text_faint #737373` and the blue cursor row. So I recaptured all 232 holla/jackin scenarios at 120x40 in truecolor and 256 with `VISUAL_TOOL=capture`. I parsed every cell for fg/bg tokens and rendered about 20 PNGs. Everything is in the scratchpad extract; nothing in the repo or the `visual-baseline` tag was touched.

Typical numbers:
- **Holla parity and finder screens:** 0.1–0.9% of text cells are chromatic, and one of those cells is the focus bar.
- **Jackin manager, editor and settings:** 0.3–3%.
- **Failure screens, where red floods the ink:** holla `git/merge` 36%, `brew/services_stop_failed` 44%, `docker/remove_failed` 30%; jackin `accounts/filter` 34%, of which 223 cells are amber and 144 red.

## (1) Semantic colour roles: additions and changes

Contrast below is the token against canvas `#000` / surface `#111` / card `#18181b` / raised `#27272a` / popover `#3f3f46`.

| Token | Hex | 256 | 16 | Mono cue | Contrast | Role |
|---|---|---|---|---|---|---|
| `info` (retune from `#8787ff`, which neither app uses) | `#6cb6ff` | 75 | LightBlue | glyph `→` / `↓↑` | 9.77 / 8.79 / 8.24 / 6.93 / 4.86 | Neutral facts that are "on": ahead/behind counts, refreshing, links, run ids, the low (healthy) meter run. It joins the blue family of `highlight #2f5aa8`, so blue reads as selection or information. |
| `link` (alias of `info` plus a modifier) | `#6cb6ff`, underlined on hover | 75 | LightBlue | underline | as `info` | URLs, `ledger/fx.rs:71`, copyable ids |
| `active` (alias of `accent` as a foreground, used for live state) | `#48e054` | 78 | LightGreen | `◉` / spinner, bold | 12.07 / 10.85 / 10.18 / 8.56 / 6.00 | Running, live or in-progress: the `◉` glyph, the word "running", "N running", spinners. Never used for "done". |
| `success` (exists; adopt it) | `#3fb950` | 71 | Green | `✓` | 8.27 / 7.43 / 6.97 / 5.86 / **4.11** | Done, ok, exit 0, current, succeeded, trashed or reclaimed. On popover planes use `accent`, or pin 256→77. |
| `error` (lift) | `#e44545` → `#f05d5d` | 167, and 203 on raised/popover | LightRed | `!` bold | 6.43 / 5.78 / 5.42 / 4.56 / 3.2 (today 4.42 on card and 3.71 on raised, both failing) | Failure; only the status cell plus `!` |
| `warning` (unchanged; pin it at 256) | `#f59e09` | 214, pinned in `apply_256` | Yellow | `▲` | 9.78 / 8.79 / 8.25 / 6.93 / 4.86 | Attention, dirty, pending risk |
| Meter levels (library) | Low = `info`, Medium = `warning`, High = `error` | 75 / 214 / 167 | LightBlue / Yellow / LightRed | `▲` / `!` suffix | — | Keeps the "a quota is not a completion, so never green" rule, but stops healthy meters being grey |
| Section headings | `text_faint` → `text_muted #808080` | 244 | Gray | — | 5.32 (faint is 4.43 on canvas, which fails 4.5:1) | "Suggested here", provider groups |
| `Tone` enum | add `Tone::Info` and `Tone::Active` | — | — | — | — | Today `Tone` cannot express either role (`theme.rs:1472-1482, 1514`) |

I recommend against brand or provider colours (Anthropic orange, OpenAI teal and so on). They would collide with the warning and success hues and add decoration rather than meaning. Providers stay neutral; their status carries the colour.

Key hints should stay `text_primary` bold. Blue keys in the k9s style would compete with the blue cursor row.

At 16 colours, ink on a fill (`highlight` is Blue) must stay `text_primary`. Using `info` LightBlue there would collide.

## (2) Ranked findings

| Sev | App / screen / state (capture) | Element | Today | Proposed | Change at |
|---|---|---|---|---|---|
| H | both; every page tab strip (holla `flows/*`, jackin `editor/*`, `settings/*`) | active tab underline while its page is active | `border_strong #4d4d4d`, 2.48:1, grey because `TabRule::Auto` is accent only when the strip has focus | `accent` whenever the tab's page is active | library `widgets/tabs.rs:104-115, 769-774` |
| H | jackin `usage/overview` | provider table rows | the whole row (name, count, window, status) in `error`/`warning`, as one formatted string with one tone | name `text_primary`; window % as a `Meter`; only the Status cell in `error`/`warning` with `!`/`▲`; "current" in `success` with `✓` | app `screens/usage.rs:398-420` (rebuild as a Table with per-cell tone) |
| H | jackin `accounts/filter`, `usage/overview`, `audit/accounts` | Health line | the whole sentence red, including "4 warnings · 2 stale · 1 refreshing" | "degraded" `error` bold; warnings and stale `warning`; refreshing `info`; counts `text_primary` | app `usage.rs:314-319`, `accounts.rs:1457` |
| H | jackin `usage/detail`, `accounts/detail` | Limits block meters at 21–54% | `text_secondary` grey fill | Low run `info` blue | library `widgets/progress.rs:274-278` |
| H | jackin `manager/tree_expanded`, `detail_drawer`, top bar | running instance `◉` and the word "running"; "2 running" in the bar and frame title | `Normal` white / `Muted` grey | `active` on the glyph and the word; count `text_secondary` with an `active` dot | app `manager.rs:220, 1177`; `app.rs:1027, 2769` |
| H | holla `brew/upgrade_plan`, batch toasts | "Upgrade Homebrew packages · 4 ok · 1 failed · 0 cancelled" | the whole line `error`; an all-ok batch is `Secondary` | segments: ok `success`, failed `error`, cancelled `muted`; an all-ok batch in `success` | app `holla/app.rs:548-553` |
| H | holla `git/merge`, `push_rejected`, `docker/remove_failed`, `trust/deploy_failed`, `brew/services_stop_failed`, `concept/launch-failure` | stderr body | 30–44% of ink bold red; the status repeated in header, status bar and toast | body `text_primary` with a red `!` gutter or first line; red only on the header status chip and the toast | app `holla/screens/activity.rs` (output line tone) |
| M | holla `activities/jump`, `docker/done`; jackin capsule tabs | finished-tab suffix `✓`, idle `○` | `text_muted` | `✓` `success`; `○` muted | library `tabs.rs:31, 726-735` (add a `done` state beside busy/error/dirty) |
| M | holla `activities/jump` | toast "api tests finished · exit 0"; `url http://localhost:5173/` | grey | exit 0 `success`; URL `link` | app `activity.rs` |
| M | holla `docker/done` | panel meta "succeeded"; "reclaimed 18.4 GB" | `text_faint` / plain | `success` | app `holla/screens/plan.rs:196` |
| M | holla `cleanup/history`, `cleanup/report` (0% chroma) | outcome column "trashed"; freed sizes | plain | `✓ trashed` in `success`; failures in `error` | app `holla/screens/cleanup.rs` (history table) |
| M | jackin `cockpit/debug` | run-id chip `run-2026…-b5df` | `warning` amber, though nothing is wrong | `link`/`info` | app `cockpit.rs:975` |
| M | holla status bar (all `parity/*`) | `↓2 ↑1 • 1` | all `warning` | `↓↑` in `info`; dirty `•` `warning`; conflicts `error` | app `holla/app.rs:3512-3526` |
| M | holla finder (`parity/*`, `flows/finder/*`) | section headings "Suggested here", "Recent here", "Explore" | `text_faint`, 4.43:1, fails AA | `text_muted` | app `holla/screens/mod.rs:472` |
| M | jackin `usage/*`, `accounts/*` left tree | provider group labels (Anthropic, OpenAI…) | `text_faint`, 4.43:1 | `text_muted` | app `usage.rs` / `accounts.rs` group rows |
| M | holla `cleanup/categories`, `artifacts`, `derived_data` vs jackin `editor/roles` | checked boxes | holla: always `accent`; jackin: white; library: accent only on focus | follow `t.chosen(focused)` everywhere | app `holla/screens/cleanup.rs:646`; library `choice.rs:141` |
| M | holla `brew/upgrade_plan` | step checkboxes | required steps grey, optional steps green, so green reads as "optional" | locked or required: faint plus a lock cue; toggleable: `chosen()` | app `holla/screens/plan.rs` |
| M | jackin `accounts/filter` Warnings block | each warning row | the whole row amber or red (account, window, reset time) | glyph and status phrase coloured; account `text_primary`; reset time `muted` | app `accounts.rs` (warnings list) |
| M | holla `disk/*` | filesystem meter at 66% | `warning`, because the 60% quota threshold is applied to a disk | disk thresholds 80/95 via `MeterTone::Level`; low run `info` | app `holla/screens/disk.rs:1669-1672` |
| M | jackin `capsule/zoom`, `audit/capsule` | status-bar Session meter at 76% | faint run (Stale), which hides the ▲ level that the Accounts screen shows for the same window | keep the level hue at stale and mark staleness with italic or the faint value only | library `progress.rs` (Stale branch); app `capsule.rs:1824-1836` |
| M | both, error on cards and raised planes (accounts detail on card, cleanup gates) | `error` text | 4.42:1 on card, 3.71:1 on raised | `#f05d5d`; pin 203 on raised in 256 | library `theme.rs` palette `RED` (~379), `apply_256:696` |
| M | both | `info` / `success` availability | `info` 0 cells in either app; `success` ≤5 cells per screen | add `Tone::Info` / `Tone::Active`; use `success` for done | library `theme.rs:1472-1482, 1514-1524` |
| M | 256-colour level | `warning` / `info` | not pinned; resolved by nearest-match | pin `warning=214`, `info=75` | library `theme.rs:666-714` |
| L | jackin `manager/tree_expanded` | amber `•` on c41e (dirty) | colour plus glyph, but unlabelled in the tree | add "dirty" in the meta or a tooltip | app `manager.rs` (~226) |
| L | jackin `cockpit/debug`, `launch-running` | stage list | good: `✓` success, accent spinner, queued muted | — keep as the reference pattern | — |
| L | jackin `editor/roles`, `settings/*` | "registry · trusted" | muted (default state) | keep it; colour only non-default states (already done for load error and not-in-registry) | — |
| L | holla `help/overlay`, jackin `manager/help_overlay`, `intro/f300`, `outro-last` (0% chroma) | reference and cinematic screens | grey | acceptable; at most `active` on the current-page section in help | — |
| L | 16 colours | `info` on a `highlight` (Blue) fill | would collide | ink on fills stays `text_primary` | library `apply_16` |

## (3) Top 10 quick wins

1. Make the page tab strip underline `accent` whenever the tab's page is active (`tabs.rs:769-774`). This fixes the stated tab-strip constraint.
2. Add `Tone::Info` and `Tone::Active`, retune `info` to `#6cb6ff`, and pin 75 / 214 in `apply_256`.
3. Draw the low meter run in `info` instead of `text_secondary` (`progress.rs:274-278`). It colours every healthy quota in usage, accounts and capsule.
4. jackin running instances: show `◉` and "running" in `active` (`manager.rs:220`), and give the "N running" counts an `active` dot.
5. Use `success` for done states: tab `✓` suffix (library), exit 0, "succeeded", "ok", "current", "trashed". Each is one tone swap.
6. Usage overview table: colour only the Status cell and turn the window into a Meter (`usage.rs:398-420`). This removes the red wall.
7. Split the Health line and the holla batch toast into per-segment tones (`usage.rs:314`, `accounts.rs:1457`, `holla/app.rs:548`).
8. Failure output: body in `text_primary` with a red `!` gutter, keeping red for the status chip only.
9. Move headings and group labels from `text_faint` to `text_muted` (`holla/screens/mod.rs:472`, usage and accounts trees). This fixes the 4.43:1 AA failure.
10. Lift `error` to `#f05d5d` and recolour the run-id chip and the git `↓↑` counts from amber to `info`, leaving amber to mean attention.

Captures and render PNGs are in `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/cap/` and `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/colour-b/target/tuiscotti/render/`. The parsers are in `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/tools/` (`ansi.py`, `sum2.py`, `seg.py`).