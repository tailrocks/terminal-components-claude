# Holla — design note (as built, P0–P6)

The build record: the chosen interaction model, what was rejected, and how
CONCEPT.md §13's principles landed in running code. The brainstorm and its
head-to-head live in `interaction-models.md`; session-by-session decisions
in `decisions.md`. This file is the map between them and the binary.

## Chosen model: Console

One persistent root surface with chrome (menu bar, activity strip, status
line, hint bar), an always-armed query, and full-window sub-surfaces (plan,
activity) that the strip and `0`/`Esc` ladder always lead back from.

Chosen over:

- **Lens** (transient summon-and-dismiss palette) — rejected: persistent
  activities and two-gate deliberation both demand the opposite posture;
  every deep flow becomes Console with worse chrome. Kept: its lightness —
  home is query-first, field armed in the first frame, Enter always means
  the top match.
- **Atlas** (spatial scope compass, columns per ring) — rejected: gates and
  plan DAGs need full width exactly when facts matter most; columns collapse
  on the narrow remote terminals where trust matters most; needs a new
  widget family before first render. Kept: scope as *place* — explicit scope
  tags on every nonlocal row, the breadcrumb, the `Ctrl+S` ring filter.

## How §13's principles landed

1. **Here first** — cwd is the primary ring; scope weight dominates the
   score before memory and urgency.
2. **Intent before syntax** — the query matches titles, reasons, commands,
   keywords, scope labels, and taught aliases (`cb` finds `cargo build`).
3. **One root experience** — Home; plans and activities are sub-surfaces of
   the same window, never separate modes.
4. **Useful before typing** — discovery seeds Suggested/Recent/Explore in
   the first frames (virtual clock; `--frame` seeks it deterministically).
5. **Search across domains** — one catalogue over mise, git, docker, disk,
   pg, ssh, gh, apt.
6. **Resources plus actions** — rows are both: Inspect (resource) and
   Restart / Update / Reclaim (actions) come from the same discovery.
7. **Primary action plus discoverable alternatives** — Enter runs the top
   match; `Ctrl+O` opens the alternatives menu (run/preview/copy/pin/alias/
   hide/reset, each with its why).
8. **Adaptive but predictable** — ranking = ring weight + pin (150) +
   alias (90) + usage + urgency (30), total and deterministic; a pin always
   beats alias+urgency combined.
9. **Explain every recommendation** — every row carries a reason; aliases
   annotate (`· alias gs`); ranking memory surfaces as facts.
10. **Keep scope visible** — explicit scope tag on every nonlocal row, ring
    filter beside the query, breadcrumb in the header.
11. **Feel immediate** — paused frames are byte-identical; every interaction
    is one keystroke from home.
12. **Make safety structural** — risk changes treatment, never availability:
    ReadOnly runs, Bounded previews, Broad gets the two-gate plan (review
    with exclusion recalculation, then a typed phrase bound to the target
    host). Untrusted task files gate per exact file. Production hosts get
    danger-toned confirms and a quit dialog that names the remote identity.
    Terminate revalidates before it kills.
13. **Coordinate instead of recreating** — handoffs are honest: btm owns
    live monitoring (snapshot + "btm takes over the screen"), pg_activity's
    lock tree is referenced, ssh connects are previews of the real command.
14. **Zero configuration, optional mastery** — useful from the first frame;
    mastery is pins, aliases, hides — all optional, all reversible
    (`Reset ranking`).
15. **Keyboard-first, not shortcut-secret** — every action is reachable by
    query + Enter; the hint bar shows the current surface's keys; `?` is the
    full reference; the mouse works everywhere but is never required.
16. **Local-first and privacy-conscious** — fixtures only; SSH identity
    files are filenames, contents never read; nothing leaves the process.

## Rejected along the way (build-time, not brainstorm)

- **`Tabs` widget for the activity strip** — document-oriented, no per-tab
  tone; the strip is manual cells with per-state tones and hit ids.
- **Digits as global activity shortcuts** — digits filter the query on home
  (P0 model holds); `1–9` jump only on activity pages; `Ctrl+A` is the
  chrome chord that works everywhere.
- **Viewing state in the World** — current activity and per-activity scroll
  live in the screen; the World stays fixture truth.
- **Hard-coded `main`** — every git repo resolves its own primary branch;
  the bulk plan checks out `trunk` for billing, `main` for legacy.
- **Silence on failure** — failed discovery renders as a blocked row with a
  warning tone (docker-cleanup, hard-cases), never an empty list.
- **Truncated policy lines** — dialog width is content-driven and set per
  dialog (`Dialog.width` is pub): pg tree 88, monitor 84, ssh 76.

## Boundaries held (§14)

Not a shell (queries filter, they don't execute), not an encyclopedia
(catalogue is derived from *this* context), not an autonomous daemon
(nothing runs without a gate), not a real executor (every effect is a
fixture mutation), not a specialist-TUI replacement (btm/pg_activity are
handoffs, not reimplementations), and ranking never defeats muscle memory
(pins and aliases are explicit user acts with a reset).
