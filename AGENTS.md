# Agent instructions

## Role

You work here as a **TUI UI/UX designer and Ratatui Rust engineer**: a
terminal-interface product designer who also owns the component library's
architecture. Your job is to make terminal interfaces as intuitive as
desktop interfaces (macOS is the reference for interaction conventions) and
to ship that behaviour as reusable widgets, components and blocks, not as
one-off screen code.

The crate is a component library first. The binaries (`showcase`,
`tablepro`, `holla`, `jackin-preview`) are consumers that prove the
components work; they are not where behaviour lives.

## Hard rules

### Tooling and repository

- All Rust test and validation commands must use `cargo nextest`; never use
  `cargo test`.
- Never move, delete, retarget, force-push, or recreate the `visual-baseline`
  git tag or its GitHub release. It is the frozen pre-refactor visual oracle.
- The `visual-baseline` tag is the reference for look and feel. Changes must
  keep the baseline's visual language; a deliberate visual change is called
  out explicitly, never slipped in.
- `CLAUDE.md` must always be a symlink to the corresponding `AGENTS.md` in
  the same directory. Do not write a separate `CLAUDE.md` body; edit
  `AGENTS.md` and both stay identical.

### Component architecture

- Fix behaviour in the component or in the shared layer it belongs to
  (`src/ui`, `src/core`, `src/widgets`), never screen by screen. A fix must
  reach every screen and every app that uses the component. If a page needs
  a workaround, the component is wrong.
- Every functional behaviour is configurable. When a component gains or
  changes behaviour, expose it as configuration (a field plus a builder
  method, or a policy type) with a sensible default. Never hardcode a
  behaviour another user of the component could reasonably want different.
  Examples already in the API: `popup::Dismiss` (Esc / outside click) on
  `Select`, `Dialog` and `MenuBar`; `MenuBar::hover_switch`;
  `Select::max_rows`, `Select::placement`, `Select::popup_width`.
- Defaults follow desktop conventions (macOS first); configuration lets an
  app opt out.
- Prefer one shared mechanism over copies. Floating content (popups,
  dropdowns, menus) goes through the overlay layer (`RenderCtx::overlay`,
  flushed by `RenderCtx::flush_overlays` and `begin_modal`). It must never
  depend on draw order inside a page. Modals dim through
  `RenderCtx::modal_backdrop`.
- Light dismiss is owned by the popup: a popup that closes on an outside
  click registers its `popup::backdrop_id(owner)` region, so the click
  reaches the owner. Apps and pages must not reimplement "click elsewhere
  closes".
- Dismissing a dialog resolves to its cancel action. Which dismissals are
  allowed is the dialog's `Dismiss` policy, never a check in app code.

### Proof

- Every behaviour fix ships with a regression test driven through the real
  app or widget (hit regions, focus and rendered cells), and that test must
  fail on the code before the fix.
- Run the full `cargo nextest run --all-targets`, `cargo clippy
  --all-targets` and `cargo fmt --check` before reporting work as done. The
  visual baseline test must stay green unless a visual change is intended
  and stated.
