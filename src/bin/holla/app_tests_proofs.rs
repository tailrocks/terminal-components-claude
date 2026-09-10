//! F23 proofs through the real App on a TestBackend: the executable
//! page/state inventory (F23a), batched-versus-separated event pumping and
//! resize (F23c), bounded input draining (F23d), rendering work and
//! fresh-layout equivalence (F23e), and the subsidiary input regressions
//! (F23h). Palette matrices in a fresh process live in `tests/holla_color.rs`;
//! the owned-PTY flood is `tests/input_flood.rs`.

use std::collections::VecDeque;
use std::time::Instant;

use ratatui::Terminal;
use ratatui::backend::{Backend, TestBackend};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::runtime::{DRAIN_BUDGET, drain_ready_inputs};
use junie_tui::theme::{ColorLevel, Theme};

use crate::app::{App, TabKind};
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::{activity, disk, files, finder, review, snapshot};

fn key(code: KeyCode) -> Input {
    Input::Key(Key {
        code,
        mods: KeyModifiers::NONE,
    })
}

fn key_mods(code: KeyCode, mods: KeyModifiers) -> Input {
    Input::Key(Key { code, mods })
}

fn typed(s: &str) -> Vec<Input> {
    s.chars().map(|c| key(KeyCode::Char(c))).collect()
}

fn with_theme(scenario: Scenario, frame: u64, w: u16, h: u16, theme: Theme) -> H {
    let app = App::for_scenario(scenario, Motion::Paused, frame, theme);
    let term = Terminal::new(TestBackend::new(w, h)).unwrap();
    let mut hh = H { app, term };
    hh.draw();
    hh
}

// ------------------------------------------------------------ F23a inventory

/// One reachable state: the route from the scenario's opening frame, the
/// control that must own focus there, and a semantic marker of the page.
struct State {
    name: &'static str,
    scenario: Scenario,
    frame: u64,
    route: Vec<Input>,
    focus: WidgetId,
    marker: &'static str,
    /// A modal is open (the finder underneath does not own the keyboard).
    modal: bool,
}

fn inventory() -> Vec<State> {
    let enter = key(KeyCode::Enter);
    let mut v = vec![];
    let mut add = |name, scenario, frame, route: Vec<Input>, focus, marker, modal| {
        v.push(State {
            name,
            scenario,
            frame,
            route,
            focus,
            marker,
            modal,
        });
    };
    add(
        "finder",
        Scenario::FirstUse,
        0,
        vec![],
        finder::FINDER,
        "Here",
        false,
    );
    add(
        "finder preview",
        Scenario::RustDirty,
        0,
        vec![key(KeyCode::Tab)],
        finder::PREVIEW,
        "Here",
        false,
    );
    add(
        "help modal",
        Scenario::FirstUse,
        0,
        vec![key(KeyCode::F(1))],
        finder::FINDER,
        "Key reference",
        true,
    );
    add(
        "activities picker",
        Scenario::ActivitiesMulti,
        0,
        vec![key_mods(KeyCode::Char('g'), KeyModifiers::CONTROL)],
        finder::FINDER,
        "Activities",
        true,
    );
    add(
        "alternatives menu",
        Scenario::RustDirty,
        0,
        vec![key_mods(KeyCode::Enter, KeyModifiers::ALT)],
        finder::FINDER,
        "alias",
        true,
    );
    let mut browse = typed("Browse ~/work/site");
    browse.push(enter.clone());
    add(
        "files browser",
        Scenario::ParityBrowser,
        40,
        browse,
        files::LIST,
        "Files ›",
        false,
    );
    let mut find = typed("Find files under home");
    find.push(enter.clone());
    add(
        "find files",
        Scenario::ParityFiles,
        40,
        find,
        files::LIST,
        "Files ›",
        false,
    );
    let mut overview = typed("Disk overview");
    overview.push(enter.clone());
    add(
        "disk overview",
        Scenario::ParityDiskNavigation,
        40,
        overview.clone(),
        disk::LIST,
        "Disk › Overview",
        false,
    );
    let mut tree = overview.clone();
    tree.push(enter.clone());
    add(
        "disk tree",
        Scenario::ParityDiskNavigation,
        40,
        tree.clone(),
        disk::TREE,
        "Disk › Usage",
        false,
    );
    let mut top = overview.clone();
    top.push(key(KeyCode::Char('t')));
    add(
        "top files",
        Scenario::ParityDiskNavigation,
        40,
        top,
        disk::LIST,
        "Disk › Top files",
        false,
    );
    let mut facts = tree.clone();
    facts.push(key(KeyCode::Char('p')));
    add(
        "disk facts drawer",
        Scenario::ParityDiskNavigation,
        40,
        facts,
        disk::DETAIL,
        "Disk › Usage",
        false,
    );
    let mut cleanup = typed("Review cleanup candidates");
    cleanup.push(enter.clone());
    add(
        "cleanup insight",
        Scenario::ParityInsights,
        40,
        cleanup,
        crate::screens::cleanup::LIST,
        "Cleanup",
        false,
    );
    let mut trust = typed("Deploy preview");
    trust.push(enter.clone());
    add(
        "trust review",
        Scenario::ParityCustomActions,
        40,
        trust,
        review::TRUST_CANCEL,
        "Trust",
        false,
    );
    let mut config = typed("Custom action configuration");
    config.push(enter.clone());
    add(
        "custom config",
        Scenario::ParityCustomActions,
        40,
        config,
        snapshot::BODY,
        "snapshot",
        false,
    );
    let mut args = typed("port");
    args.push(enter.clone());
    add(
        "arguments",
        Scenario::RustDirty,
        0,
        args.clone(),
        WidgetId::of("args.field").child(0),
        "Arguments",
        false,
    );
    let mut port = args.clone();
    port.extend(typed("5173"));
    // typing started the edit; Ctrl+S submits the facts page
    port.push(key_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    add(
        "port snapshot",
        Scenario::RustDirty,
        0,
        port,
        snapshot::BODY,
        "snapshot",
        false,
    );
    let mut gate = typed("restart payments");
    gate.push(enter.clone());
    add(
        "gate one",
        Scenario::RemoteHost,
        0,
        gate,
        review::GATE_CANCEL,
        "gate 1 of 2",
        false,
    );
    add(
        "activity tab",
        Scenario::ActivitiesMulti,
        0,
        vec![key_mods(KeyCode::Char('1'), KeyModifiers::ALT)],
        activity::VIEW,
        "output",
        false,
    );
    let mut services = typed("Homebrew services");
    services.push(enter.clone());
    add(
        "brew services",
        Scenario::ParityBrewServices,
        40,
        services,
        snapshot::BODY,
        "snapshot",
        false,
    );
    v
}

fn run_route(h: &mut H, route: &[Input]) {
    for i in route {
        h.app.handle(i.clone());
        h.draw();
    }
}

#[test]
fn inventory_reaches_every_state_with_its_focus_owner_at_every_size_and_palette() {
    let sizes = [(72u16, 20u16), (100, 30), (160, 50)];
    let levels = [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ];
    for state in inventory() {
        for (w, hgt) in sizes {
            for level in levels {
                let mut h =
                    with_theme(state.scenario, state.frame, w, hgt, Theme::for_level(level));
                h.ticks(3);
                run_route(&mut h, &state.route);
                let text = h.text();
                assert!(
                    text.contains(state.marker),
                    "{} at {w}x{hgt} {level:?}: marker {:?} missing\n{text}",
                    state.name,
                    state.marker
                );
                assert_eq!(
                    h.app.modals.is_empty(),
                    !state.modal,
                    "{} at {w}x{hgt}: modal presence",
                    state.name
                );
                if !state.modal {
                    assert_eq!(
                        h.app.focus.current(),
                        Some(state.focus),
                        "{} at {w}x{hgt} {level:?}: focus owner\n{text}",
                        state.name
                    );
                    // the owner is a registered hit target with an area
                    assert!(
                        h.app.hits.area_of(state.focus).is_some(),
                        "{} at {w}x{hgt}: focus owner has no visible target",
                        state.name
                    );
                    // keyboard traversal never leaves the page without focus
                    for _ in 0..3 {
                        h.key(KeyCode::Tab);
                        assert!(
                            h.app.focus.current().is_some(),
                            "{}: Tab lost focus",
                            state.name
                        );
                    }
                    for _ in 0..3 {
                        h.key(KeyCode::BackTab);
                        assert!(
                            h.app.focus.current().is_some(),
                            "{}: Shift+Tab lost focus",
                            state.name
                        );
                    }
                    // the ring is a cycle: Tab comes back to the owner
                    let mut back = false;
                    for _ in 0..16 {
                        h.key(KeyCode::Tab);
                        if h.app.focus.current() == Some(state.focus) {
                            back = true;
                            break;
                        }
                    }
                    assert!(
                        back,
                        "{} at {w}x{hgt}: Tab never returned to the owner",
                        state.name
                    );
                    // mouse: clicking the owner's area keeps or gives it focus
                    if let Some(area) = h.app.hits.area_of(state.focus) {
                        h.click(area.x, area.y);
                        assert!(
                            h.app.focus.current().is_some(),
                            "{}: click lost focus",
                            state.name
                        );
                    }
                }
                // a disabled control is never a traversal stop: every stop is hittable
                let start = h.app.focus.current();
                let mut seen = vec![];
                for _ in 0..12 {
                    h.key(KeyCode::Tab);
                    if let Some(f) = h.app.focus.current() {
                        if seen.contains(&f) {
                            break;
                        }
                        seen.push(f);
                        assert!(
                            h.app.hits.area_of(f).is_some() || !h.app.modals.is_empty(),
                            "{} at {w}x{hgt}: traversal stop {f:?} is not visible",
                            state.name
                        );
                    }
                }
                let _ = start;
            }
        }
    }
}

// ------------------------------------------------------------ F23c freshness

/// The runtime's pump: drain until a change, render, continue. A resize in
/// the batch resizes the backend the way the terminal would have.
fn pump(h: &mut H, inputs: Vec<Input>) {
    let mut q = VecDeque::from(inputs);
    loop {
        let mut resized = None;
        let changed = drain_ready_inputs(&mut h.app, || {
            let next = q.pop_front();
            if let Some(Input::Resize(w, hgt)) = &next {
                resized = Some((*w, *hgt));
            }
            Ok(next)
        })
        .unwrap();
        if let Some((w, hgt)) = resized {
            h.term.backend_mut().resize(w, hgt);
        }
        if changed {
            h.draw();
        }
        if q.is_empty() {
            break;
        }
    }
}

fn snapshot_state(h: &mut H) -> (String, Option<WidgetId>, usize, Option<Position>) {
    h.draw();
    let cursor = h.term.get_cursor_position().ok();
    (h.text(), h.app.focus.current(), h.app.hits.len(), cursor)
}

#[test]
fn batched_and_separated_event_sequences_agree_on_focus_hits_drafts_and_cursor() {
    // page → modal → resize → activation → paste → mouse, as one batch and
    // as separately rendered events, on the real App
    let sequence = |w: u16, h: u16| {
        let mut v = typed("port");
        v.push(key(KeyCode::Enter));
        v.push(Input::Resize(w, h));
        v.push(Input::Paste("5173".into()));
        v.push(key(KeyCode::Tab));
        v.push(key(KeyCode::F(1)));
        v.push(key(KeyCode::Esc));
        v.push(Input::Mouse(Mouse {
            kind: MouseKind::Move,
            pos: Position::new(3, 4),
        }));
        v
    };
    let mut batched = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    pump(&mut batched, sequence(100, 30));
    let a = snapshot_state(&mut batched);

    let mut separated = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    for i in sequence(100, 30) {
        if let Input::Resize(w, h) = i {
            separated.resize(w, h);
            continue;
        }
        separated.app.handle(i);
        separated.draw();
    }
    let b = snapshot_state(&mut separated);
    assert_eq!(a.0, b.0, "rendered text agrees");
    assert_eq!(a.1, b.1, "focus agrees");
    assert_eq!(a.2, b.2, "hit regions agree");
    assert_eq!(a.3, b.3, "cursor agrees");
    assert!(
        a.0.contains("5173"),
        "the pasted draft survived both routes: {}",
        a.0
    );
}

#[test]
fn resize_below_minimum_then_normal_then_wide_then_minimum_keeps_every_state_consistent() {
    for scenario in [
        Scenario::RustDirty,
        Scenario::ParityBrowser,
        Scenario::ParityDiskNavigation,
    ] {
        let mut h = H::new(scenario, Motion::Paused, 40, 120, 40);
        h.ticks(2);
        if scenario == Scenario::ParityBrowser {
            h.type_str("Browse ~/work/site");
            h.key(KeyCode::Enter);
        }
        if scenario == Scenario::ParityDiskNavigation {
            h.type_str("Disk overview");
            h.key(KeyCode::Enter);
            h.key(KeyCode::Enter);
        }
        let before = h.app.focus.current();
        for (w, hgt) in [
            (40u16, 10u16),
            (100, 30),
            (200, 60),
            (72, 20),
            (40, 10),
            (120, 40),
        ] {
            h.resize(w, hgt);
            let text = h.text();
            assert!(
                text.contains("holla❯") || w < 60,
                "{scenario:?} {w}x{hgt}: header\n{text}"
            );
            let focus = h.app.focus.current();
            assert!(focus.is_some(), "{scenario:?} {w}x{hgt}: focus lost");
            if let Some(f) = focus {
                assert!(
                    h.app.hits.area_of(f).is_some() || w < 60,
                    "{scenario:?} {w}x{hgt}: focused {f:?} has no visible area"
                );
            }
            // a key after the resize lands on the current owner, never on a
            // stale region
            h.key(KeyCode::Down);
            h.key(KeyCode::Up);
            // a text field's caret sits inside the field it belongs to
            if let Some(f) = h.app.focus.current()
                && f == files::QUERY
                && let (Some(area), Ok(c)) = (h.app.hits.area_of(f), h.term.get_cursor_position())
            {
                assert!(
                    c.y >= area.y && c.y < area.bottom(),
                    "{scenario:?} {w}x{hgt}: cursor {c:?} outside {area:?}"
                );
            }
        }
        assert_eq!(
            h.app.focus.current(),
            before,
            "{scenario:?}: the owner survived the resize ladder"
        );
    }
}

// ------------------------------------------------------------ F23d fairness

#[test]
fn a_finite_flood_of_ignored_input_never_starves_the_tick_check() {
    let mut h = H::new(Scenario::FirstUse, Motion::Reduced, 0, 100, 30);
    // an unbound chord is ignored by every owner: the worst case for draining
    let flood: Vec<Input> = (0..10_000)
        .map(|_| key_mods(KeyCode::Char('x'), KeyModifiers::ALT))
        .collect();
    let mut q = VecDeque::from(flood);
    let mut passes = 0;
    let mut dispatched_max = 0;
    let start = Instant::now();
    while !q.is_empty() {
        let before = q.len();
        let changed = drain_ready_inputs(&mut h.app, || Ok(q.pop_front())).unwrap();
        assert!(!changed, "ignored input never reports a change");
        let dispatched = before - q.len();
        dispatched_max = dispatched_max.max(dispatched);
        // the runtime reaches its tick check here
        h.app.handle(Input::Tick);
        passes += 1;
    }
    assert!(
        dispatched_max <= DRAIN_BUDGET,
        "one pass dispatched {dispatched_max} unchanged events; the budget is {DRAIN_BUDGET}"
    );
    assert!(
        passes >= 10_000 / DRAIN_BUDGET,
        "ticks interleaved the flood: {passes} passes"
    );
    let per_event = start.elapsed() / 10_000;
    eprintln!("F23d: 10000 ignored events in {passes} passes · {per_event:?} per dispatch");
    // a quit request queued behind a flood is honoured within one budget
    let mut q: VecDeque<Input> = (0..DRAIN_BUDGET * 3)
        .map(|_| key_mods(KeyCode::Char('x'), KeyModifiers::ALT))
        .collect();
    q.push_back(key(KeyCode::Esc));
    let mut passes = 0;
    while !h.app.quit {
        drain_ready_inputs(&mut h.app, || Ok(q.pop_front())).unwrap();
        passes += 1;
        assert!(passes <= 4, "quit waited {passes} passes");
    }
}

// ------------------------------------------------------------ F23e performance

fn activity_view(h: &mut H) -> &junie_tui::widgets::viewport::TextViewport {
    let i = h.app.active;
    h.app.tabs[i]
        .stack
        .last_mut()
        .and_then(|s| s.as_activity())
        .map(|a| a.view())
        .expect("an activity tab")
}

#[test]
fn a_burst_of_output_is_appended_not_rebuilt_and_equals_a_fresh_layout() {
    let mut h = H::new(Scenario::ParityExecutor, Motion::Reduced, 0, 120, 40);
    h.ticks(6);
    h.type_str("Emit a burst");
    h.key(KeyCode::Enter);
    assert!(matches!(h.tab_kind(), TabKind::Activity(_)), "{}", h.text());
    h.ticks(4);
    let emitted = |h: &H| {
        let TabKind::Activity(id) = h.tab_kind() else {
            unreachable!()
        };
        let a = h.app.world.activity(&id).unwrap();
        a.output.len() + a.dropped
    };
    let mid = activity_view(&mut h).work_counters();
    let mid_len = activity_view(&mut h).len();
    let mid_emitted = emitted(&h);
    assert!(mid_len > 0 && mid_len < 4500, "mid-burst: {mid_len} lines");
    let t = Instant::now();
    let mut frames = vec![];
    for _ in 0..24 {
        let f = Instant::now();
        h.ticks(1);
        frames.push(f.elapsed());
    }
    let total = t.elapsed();
    let end = activity_view(&mut h).work_counters();
    let n = activity_view(&mut h).len();
    assert!(n >= 4000, "the burst landed: {n} lines");
    let appended = emitted(&h) - mid_emitted;
    assert!(appended >= 2500, "the burst appended {appended} lines");
    assert_eq!(
        end.index_rebuilds - mid.index_rebuilds,
        0,
        "appends never rebuild the row index: {end:?} vs {mid:?}"
    );
    assert!(
        end.segmented_lines - mid.segmented_lines <= appended + 8,
        "each appended line is segmented once: {end:?} vs {mid:?}"
    );
    assert!(
        end.reflowed_lines - mid.reflowed_lines <= appended + 8,
        "retained lines are never reflowed by an append: {end:?} vs {mid:?}"
    );
    assert!(
        end.index_shifts > mid.index_shifts,
        "retention shifted the index instead of rebuilding it"
    );
    frames.sort();
    let median = frames[frames.len() / 2];
    let tail = frames[frames.len() - 1];
    eprintln!(
        "F23e burst: {n} lines · {total:?} for 24 ticks+draws · median {median:?} · tail {tail:?}"
    );
    // fresh-layout equivalence: a viewport built from scratch with the same
    // lines renders identically
    let incremental = h.text();
    let lines: Vec<_> = activity_view(&mut h).lines().cloned().collect();
    let mut fresh = junie_tui::widgets::viewport::TextViewport::new(activity::VIEW)
        .max_lines(crate::domain::activity::RETAIN_LINES)
        .wrap(true);
    fresh.set_lines(lines);
    let mut a = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, 118, 30));
    let mut b = a.clone();
    let theme = Theme::junie();
    let mut hits_a = junie_tui::core::hit::HitRegistry::default();
    let mut ring_a = junie_tui::core::focus::FocusRing::default();
    let mut hits_b = junie_tui::core::hit::HitRegistry::default();
    let mut ring_b = junie_tui::core::focus::FocusRing::default();
    let interaction = junie_tui::ui::ctx::Interaction::default();
    let mut ctx_a =
        junie_tui::ui::ctx::RenderCtx::new(&theme, interaction.clone(), &mut hits_a, &mut ring_a);
    let mut ctx_b =
        junie_tui::ui::ctx::RenderCtx::new(&theme, interaction, &mut hits_b, &mut ring_b);
    let mut incremental_view = activity_view(&mut h).clone();
    incremental_view.render(a.area, &mut a, &mut ctx_a, theme.canvas);
    fresh.render(b.area, &mut b, &mut ctx_b, theme.canvas);
    assert_eq!(a, b, "incremental and fresh layouts render the same cells");
    assert!(incremental.contains("burst line"));
}

#[test]
fn idle_ticks_rebuild_nothing_on_the_finder_and_the_disk_tree() {
    let mut h = H::new(Scenario::ParityDiskNavigation, Motion::Paused, 60, 160, 50);
    h.ticks(3);
    let finder_rebuilds = |h: &mut H| {
        h.app.tabs[0]
            .stack
            .first_mut()
            .and_then(|s| s.as_finder())
            .map(|f| f.rebuilds)
            .unwrap()
    };
    let before = finder_rebuilds(&mut h);
    let t = Instant::now();
    for _ in 0..30 {
        h.ticks(1);
    }
    let idle = t.elapsed() / 30;
    assert_eq!(
        finder_rebuilds(&mut h),
        before,
        "idle ticks never rebuild the finder"
    );
    h.type_str("Disk overview");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    h.ticks(2);
    let disk_rebuilds = |h: &mut H| {
        let i = h.app.active;
        h.app.tabs[i]
            .stack
            .last_mut()
            .and_then(|s| s.as_disk())
            .map(|d| d.rebuilds)
            .unwrap()
    };
    let done = disk_rebuilds(&mut h);
    for _ in 0..20 {
        h.ticks(1);
    }
    assert_eq!(
        disk_rebuilds(&mut h),
        done,
        "a complete scan never rebuilds on idle ticks"
    );
    h.key(KeyCode::Char('s'));
    assert_eq!(
        disk_rebuilds(&mut h),
        done + 1,
        "a sort change rebuilds exactly once"
    );
    eprintln!("F23e idle: {idle:?} per idle tick+draw at 160x50");
}

// ------------------------------------------------------------ F23h subsidiary

#[test]
fn facts_page_takes_an_actual_paste_into_the_editing_field_only() {
    let mut h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    h.type_str("port");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Arguments"));
    // the paste is one payload: no per-character key dispatch, exact text
    let o = h.app.handle(Input::Paste("51\t73\n".into()));
    h.draw();
    assert_eq!(o, Outcome::Changed);
    assert!(
        h.text().contains("5173"),
        "the field holds the sanitized payload: {}",
        h.text()
    );
    assert!(!h.text().contains("51\t73"));
    // no field is editing after Tab to a button: a paste is ignored, not misrouted
    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    let before = h.text();
    let o = h.app.handle(Input::Paste("junk".into()));
    h.draw();
    assert_ne!(o, Outcome::Changed);
    assert_eq!(h.text(), before);
}

#[test]
fn enter_with_modifiers_maps_to_exact_picker_and_finder_semantics() {
    // picker: plain Enter opens; Alt+Enter is the alternate choice; Ctrl and
    // Shift do not turn Enter into anything else
    let mut h = H::new(Scenario::ActivitiesMulti, Motion::Reduced, 0, 120, 40);
    h.ctrl(KeyCode::Char('g'));
    assert!(!h.app.modals.is_empty());
    h.app
        .handle(key_mods(KeyCode::Enter, KeyModifiers::CONTROL));
    h.draw();
    assert!(h.app.modals.is_empty(), "Ctrl+Enter chooses like Enter");
    assert!(matches!(h.tab_kind(), TabKind::Activity(_)));
    h.alt(KeyCode::Char('0'));
    h.ctrl(KeyCode::Char('g'));
    h.app.handle(key_mods(KeyCode::Enter, KeyModifiers::SHIFT));
    h.draw();
    assert!(h.app.modals.is_empty(), "Shift+Enter chooses like Enter");
    // finder: Alt+Enter opens alternatives; Shift+Enter runs like Enter
    h.alt(KeyCode::Char('0'));
    h.app.handle(key_mods(KeyCode::Enter, KeyModifiers::ALT));
    h.draw();
    assert!(
        !h.app.modals.is_empty(),
        "Alt+Enter opens the alternatives menu"
    );
    h.key(KeyCode::Esc);
    assert!(h.app.modals.is_empty());
}

#[test]
fn output_scrollbar_press_and_drag_redraw_and_find_index_resets_on_a_new_query() {
    let mut h = H::new(Scenario::ParityExecutor, Motion::Reduced, 0, 120, 40);
    h.ticks(6);
    h.type_str("Emit a burst");
    h.key(KeyCode::Enter);
    h.ticks(40);
    assert!(activity_view(&mut h).len() >= 4000);
    let sb = junie_tui::widgets::scrollbar::id_for(activity::VIEW);
    let area = h
        .app
        .hits
        .area_of(sb)
        .expect("the output scrollbar is a hit target");
    let before = h.text();
    h.mouse(MouseKind::Down, area.x, area.y + 2);
    let pressed = h.text();
    assert_ne!(before, pressed, "a press on the scrollbar moves the view");
    h.mouse(MouseKind::Drag, area.x, area.y + area.height / 2);
    let dragged = h.text();
    assert_ne!(pressed, dragged, "a drag moves it again");
    h.mouse(MouseKind::Up, area.x, area.y + area.height / 2);
    assert!(
        !activity_view(&mut h).follow,
        "dragging away from the tail pauses follow"
    );
    // find: advance to a nonzero match, then a query with no such match resets
    h.key(KeyCode::Char('/'));
    h.type_str("line 44");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("3 of"), "{}", h.text());
    h.type_str("9");
    assert!(h.text().contains("find “line 449”"), "{}", h.text());
    assert!(
        h.text().contains("1 of"),
        "the index restarted at the first surviving match: {}",
        h.text()
    );
    h.key(KeyCode::Esc);
    assert!(activity_view(&mut h).marks().is_empty());
}
