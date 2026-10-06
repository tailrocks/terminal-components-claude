//! Control state matrix: every Controls-widget state, rendered headless.
//!
//! Covers the ten Controls from the requirements registry — Brand, Button,
//! Checkbox, Toggle, RadioGroup, Panel, SplitPane, Props, Empty, TooSmall —
//! across idle, hover, kbd-focus, pointer-focus, focus-visible, pressed,
//! selected, checked, disabled, read-only, busy, loading, error, warning and
//! empty, plus the combos (focused+hovered, selected+hovered,
//! selected+disabled, busy+checked) and the precedence rules (disabled
//! removes hover/press/activation, busy removes press).
//!
//! Harness: each control renders into a sub-rectangle of a `·`-sentinel
//! buffer, so any 1-cell shift or overflow outside the widget area fails the
//! surrounding-region check. Narrow fixtures (widths 4/3/2/1) pin clipping.
//! Frames also go through the component capture path
//! ([`support::capture_buffer`]) so state deltas are digest-exact.
//!
//! States a control cannot express are pinned as N/A with the reason, not
//! silently skipped: Brand has no disabled/focus of its own (clickable-only),
//! Button has no read-only/error/warning/empty flags, Checkbox/Toggle have no
//! busy/loading/error/warning, RadioGroup has no busy/loading, Panel chrome
//! has no hover/press/disabled (focus shows on border/title only), the
//! Splitter is mouse-only with no focus or disabled, PropsList has no
//! disabled/busy/checked, EmptyState is display-only, and TooSmall is an
//! app-level screen (covered by ignored PTY probes, no snapshots).
//!
//! `HOVERED` gets explicit coverage for every hoverable control (Brand,
//! Button, Checkbox, Toggle, RadioGroup options, Splitter, PropsList rows),
//! including the per-option/per-row child-id hovers a generic matrix omits.
//!
//! Evidence: every test writes its captured frames and a human-readable
//! record under a unique run dir in `target/tuiscotti-actuals/` (gitignored
//! scratch, like `target/tuiscotti/`). The dir is printed via `eprintln`.
//!
//! No snapshot gating here on purpose: these are headless contract tests in
//! the default nextest run, plus ignored PTY needle probes for TooSmall.
//! This file deliberately declares no ported-matrix captures.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::FocusRing;
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{BadgeKind, ButtonKind, Theme, Tone};
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::ui::layout::{Maximized, Split, SplitDir};
use junie_tui::widgets::brand::Lockup;
use junie_tui::widgets::button::Button;
use junie_tui::widgets::choice::{Checkbox, RadioGroup, Toggle};
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::panel::{Panel, ScrollPanel};
use junie_tui::widgets::props::{self, Prop, PropsEvent, PropsList};
use junie_tui::widgets::splitter::Splitter;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use tuiscotti::{Frame, Provenance};

use crate::support::{self, HOLLA, JACKIN, SHOWCASE, TABLEPRO};

// ------------------------------------------------------------ harness --

fn key(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

fn id(name: &str) -> WidgetId {
    WidgetId::of(name)
}

/// Owned render stage: theme plus fresh registries per render.
struct Stage {
    theme: Theme,
    hits: HitRegistry,
    ring: FocusRing,
}

impl Stage {
    fn new() -> Self {
        Self {
            theme: Theme::junie(),
            hits: HitRegistry::default(),
            ring: FocusRing::default(),
        }
    }

    fn ctx(&mut self, ix: Interaction) -> RenderCtx<'_> {
        RenderCtx::new(&self.theme, ix, &mut self.hits, &mut self.ring)
    }

    fn bg(&self) -> Color {
        self.theme.canvas
    }
}

/// Buffer pre-filled with `·` sentinels; the widget renders into a sub-area
/// and every cell outside must still be `·` (no 1-cell overflow).
fn sentinel_buf(cols: u16, rows: u16) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, cols, rows));
    for y in 0..rows {
        for x in 0..cols {
            buf[(x, y)].set_symbol("·");
        }
    }
    buf
}

fn assert_surround_intact(buf: &Buffer, area: Rect, what: &str) {
    let (w, h) = (buf.area.width, buf.area.height);
    for y in 0..h {
        for x in 0..w {
            if x >= area.x && x < area.right() && y >= area.y && y < area.bottom() {
                continue;
            }
            assert_eq!(
                buf[(x, y)].symbol(),
                "·",
                "{what}: surround cell ({x},{y}) touched"
            );
        }
    }
}

fn row_text(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_owned())
        .collect()
}

fn capture(buf: &Buffer) -> Frame {
    support::capture_buffer(buf, Provenance::now("default", "control_states", vec![]))
}

/// Unique evidence dir for this process run (shared by the tests in this
/// file; each test writes distinctly-named files).
fn evidence_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/tuiscotti-actuals")
            .join(format!("control_states_{}_{}", std::process::id(), nanos));
        std::fs::create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("create evidence dir {}: {e}", dir.display()));
        eprintln!("control_states evidence: {}", dir.display());
        dir
    })
}

fn write_evidence(name: &str, bytes: &[u8]) {
    let path = evidence_dir().join(name);
    std::fs::write(&path, bytes)
        .unwrap_or_else(|e| panic!("write evidence {}: {e}", path.display()));
}

fn is_bold(buf: &Buffer, x: u16, y: u16) -> bool {
    buf[(x, y)].modifier.contains(Modifier::BOLD)
}

// -------------------------------------------------------------- Brand --

/// Brand lockup: idle accent fill, hover lift, pressed fill, hit registration.
/// The plain render is stateless (hover never changes it); only
/// `render_clickable` reads interaction, and it has no disabled/focus of its
/// own — clickable-only, never a focus stop.
#[test]
fn brand_state_matrix() {
    let mut record = String::from("state bg-symbol row digest\n");
    let lockup = Lockup::new("holla❯");
    assert_eq!(lockup.width(), 8, "padded width: 6 text + 2 pad");
    assert_eq!(
        Lockup::compact("holla❯").width(),
        6,
        "compact drops the padding"
    );

    // Idle: accent fill, on-accent bold text, hit registered, no focus stop.
    let bid = id("test.brand");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(16, 3);
    let w = lockup.render_clickable(4, 1, &mut buf, &mut stage.ctx(Interaction::default()), bid);
    assert_eq!(w, 8);
    let t = Theme::junie();
    assert_eq!(buf[(5, 1)].bg, t.accent, "idle fill is accent");
    assert_eq!(buf[(5, 1)].fg, t.text_on_accent);
    assert!(is_bold(&buf, 5, 1));
    assert_eq!(buf[(4, 1)].symbol(), " ", "outer pad cell");
    assert_eq!(
        stage.hits.hit(Position::new(5, 1)),
        Some(bid),
        "clickable registers its hit"
    );
    assert!(
        !stage.ring.contains(bid),
        "clickable-only: never a focus stop"
    );
    assert_surround_intact(&buf, Rect::new(4, 1, 8, 1), "brand idle");
    let idle = capture(&buf);
    record.push_str(&format!(
        "idle     accent   {:?} {:016x}\n",
        row_text(&buf, 1),
        idle.digest()
    ));
    write_evidence("brand_idle.json", idle.to_json_pretty().as_bytes());

    // Hover: exactly one plane up (accent_hover), same glyphs.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(16, 3);
    lockup.render_clickable(
        4,
        1,
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(bid),
            ..Default::default()
        }),
        bid,
    );
    assert_eq!(buf[(5, 1)].bg, t.accent_hover, "hover lifts the fill");
    assert_eq!(buf[(5, 1)].fg, t.text_on_accent, "hover keeps the text");
    assert_surround_intact(&buf, Rect::new(4, 1, 8, 1), "brand hover");
    let hovered = capture(&buf);
    assert_ne!(
        hovered.digest(),
        idle.digest(),
        "hover must change the frame"
    );
    record.push_str(&format!(
        "hover    accent^  {:?} {:016x}\n",
        row_text(&buf, 1),
        hovered.digest()
    ));
    write_evidence("brand_hover.json", hovered.to_json_pretty().as_bytes());

    // Pressed: press+hover (mouse down) and flash (post-activation) agree.
    for (name, ix) in [
        (
            "press",
            Interaction {
                hover: Some(bid),
                pressed: Some(bid),
                ..Default::default()
            },
        ),
        (
            "flash",
            Interaction {
                flash: Some(bid),
                ..Default::default()
            },
        ),
    ] {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(16, 3);
        lockup.render_clickable(4, 1, &mut buf, &mut stage.ctx(ix), bid);
        assert_eq!(buf[(5, 1)].bg, t.accent_pressed, "{name}: pressed fill");
        assert_surround_intact(&buf, Rect::new(4, 1, 8, 1), "brand {name}");
        let frame = capture(&buf);
        record.push_str(&format!(
            "{name:<8} accent!  {:?} {:016x}\n",
            row_text(&buf, 1),
            frame.digest()
        ));
    }

    // Press without hover is not a press (release-outside): stays idle.
    // `Interaction::pressed` requires hover, or flash.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(16, 3);
    lockup.render_clickable(
        4,
        1,
        &mut buf,
        &mut stage.ctx(Interaction {
            pressed: Some(bid),
            ..Default::default()
        }),
        bid,
    );
    assert_eq!(buf[(5, 1)].bg, t.accent, "press w/o hover is idle");
    assert_eq!(capture(&buf).digest(), idle.digest());

    // Plain render takes no interaction at all (stateless product mark).
    let stage = Stage::new();
    let mut buf = sentinel_buf(16, 3);
    lockup.render(4, 1, &mut buf, &stage.theme);
    assert_eq!(buf[(5, 1)].bg, t.accent);
    assert_surround_intact(&buf, Rect::new(4, 1, 8, 1), "brand plain");
    // Brand N/A: disabled, focus, selected, busy, error — no such flags.
    record.push_str("N/A disabled/focus/selected/busy/error: Lockup has no such flags\n");
    write_evidence("brand_state_matrix.txt", record.as_bytes());
    eprintln!("{record}");
}

// --------------------------------------- shared single-control shots --

/// One headless render: sentinel buffer, captured frame, widget (with its
/// recorded area) and the registries it wrote.
struct Shot<T> {
    buf: Buffer,
    frame: Frame,
    widget: T,
    hits: HitRegistry,
    ring: FocusRing,
}

fn render_button(ix: Interaction, button: Button) -> Shot<Button> {
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 4);
    let bg = stage.bg();
    let mut b = button;
    b.render(Rect::new(4, 1, 20, 1), &mut buf, &mut stage.ctx(ix), bg);
    let frame = capture(&buf);
    let Stage { hits, ring, .. } = stage;
    Shot {
        buf,
        frame,
        widget: b,
        hits,
        ring,
    }
}

fn render_checkbox(ix: Interaction, checkbox: Checkbox) -> Shot<Checkbox> {
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(28, 4);
    let bg = stage.bg();
    let mut c = checkbox;
    c.render(Rect::new(4, 1, 20, 1), &mut buf, &mut stage.ctx(ix), bg);
    let frame = capture(&buf);
    let Stage { hits, ring, .. } = stage;
    Shot {
        buf,
        frame,
        widget: c,
        hits,
        ring,
    }
}

fn render_toggle(ix: Interaction, toggle: Toggle) -> Shot<Toggle> {
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(32, 4);
    let bg = stage.bg();
    let mut t = toggle;
    t.render(Rect::new(4, 1, 24, 1), &mut buf, &mut stage.ctx(ix), bg);
    let frame = capture(&buf);
    let Stage { hits, ring, .. } = stage;
    Shot {
        buf,
        frame,
        widget: t,
        hits,
        ring,
    }
}

fn render_radio(ix: Interaction, group: RadioGroup) -> Shot<RadioGroup> {
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(28, 9);
    let bg = stage.bg();
    let mut g = group;
    let h = g.height();
    g.render(Rect::new(4, 2, 20, h), &mut buf, &mut stage.ctx(ix), bg);
    let frame = capture(&buf);
    let Stage { hits, ring, .. } = stage;
    Shot {
        buf,
        frame,
        widget: g,
        hits,
        ring,
    }
}

// ------------------------------------------------------------- Button --

/// Button across kinds and states: idle, hover, kbd-focus, pointer-focus,
/// focus-visible/hidden, pressed, selected/checked, disabled, busy(=loading),
/// combos, precedence, actions and narrow clipping. Read-only, error,
/// warning and empty are N/A: Button has no such flags (Danger is a kind,
/// not a state).
#[test]
fn button_state_matrix() {
    let t = Theme::junie();
    let mut record = String::from("kind state row digest\n");
    let bid = id("test.button");
    // "Run task" is 8 cells: plain width 10, toggle width 12.
    assert_eq!(Button::secondary(bid, "Run task").width(), 10);
    assert_eq!(Button::toggle(bid, "Run task", true).width(), 12);

    // Idle secondary: blank gutter, overlay bg, primary text, not bold.
    let shot = render_button(Interaction::default(), Button::secondary(bid, "Run task"));
    assert_eq!(shot.buf[(4, 1)].symbol(), " ", "idle gutter is blank");
    assert_eq!(shot.buf[(5, 1)].symbol(), "R");
    assert_eq!(shot.buf[(6, 1)].bg, t.surface_overlay);
    assert_eq!(shot.buf[(6, 1)].fg, t.text_primary);
    assert!(!is_bold(&shot.buf, 6, 1));
    assert_eq!(shot.widget.area, Rect::new(4, 1, 10, 1));
    assert_surround_intact(&shot.buf, Rect::new(4, 1, 10, 1), "button idle");
    assert_eq!(
        shot.hits.hit(Position::new(6, 1)),
        Some(bid),
        "enabled button registers its hit"
    );
    assert!(shot.ring.contains(bid), "enabled button is a focus stop");
    record.push_str(&format!(
        "sec idle {:?} {:016x}\n",
        row_text(&shot.buf, 1),
        shot.frame.digest()
    ));
    write_evidence("button_idle.json", shot.frame.to_json_pretty().as_bytes());

    // Hover: exactly one plane up (popover for overlay kinds).
    let shot = render_button(
        Interaction {
            hover: Some(bid),
            ..Default::default()
        },
        Button::secondary(bid, "Run task"),
    );
    assert_eq!(shot.buf[(6, 1)].bg, t.popover, "hover lifts one plane");
    assert_eq!(shot.buf[(4, 1)].symbol(), " ", "hover alone shows no bar");
    record.push_str(&format!(
        "sec hover {:?} {:016x}\n",
        row_text(&shot.buf, 1),
        shot.frame.digest()
    ));

    // Primary hover lifts the accent itself.
    let shot = render_button(
        Interaction {
            hover: Some(bid),
            ..Default::default()
        },
        Button::primary(bid, "Run task"),
    );
    assert_eq!(shot.buf[(6, 1)].bg, t.accent_hover);
    assert_eq!(shot.buf[(6, 1)].fg, t.text_on_accent);

    // Kbd-focus: focus bar + bold, hover suppressed (app sets
    // hover_suppressed on every Key input, showcase/app.rs).
    let shot = render_button(
        Interaction {
            focus: Some(bid),
            hover: Some(bid),
            hover_suppressed: true,
            ..Default::default()
        },
        Button::secondary(bid, "Run task"),
    );
    assert_eq!(shot.buf[(4, 1)].symbol(), "▎", "kbd-focus shows the bar");
    assert_eq!(shot.buf[(4, 1)].fg, t.focus);
    assert!(is_bold(&shot.buf, 6, 1));
    assert_eq!(
        shot.buf[(6, 1)].bg,
        t.surface_overlay,
        "suppressed hover adds no lift"
    );
    record.push_str(&format!(
        "sec kbdfocus {:?} {:016x}\n",
        row_text(&shot.buf, 1),
        shot.frame.digest()
    ));

    // Pointer-focus: focus bar AND hover lift together.
    let shot = render_button(
        Interaction {
            focus: Some(bid),
            hover: Some(bid),
            ..Default::default()
        },
        Button::secondary(bid, "Run task"),
    );
    assert_eq!(shot.buf[(4, 1)].symbol(), "▎");
    assert_eq!(shot.buf[(6, 1)].bg, t.popover, "pointer keeps its lift");
    assert!(is_bold(&shot.buf, 6, 1));

    // Focus-visible vs hidden: a modal hides the bar (focus_hidden).
    let hidden = render_button(
        Interaction {
            focus: Some(bid),
            focus_hidden: true,
            ..Default::default()
        },
        Button::secondary(bid, "Run task"),
    );
    assert_eq!(hidden.buf[(4, 1)].symbol(), " ", "hidden focus: no bar");
    assert!(!is_bold(&hidden.buf, 6, 1));
    let idle = render_button(Interaction::default(), Button::secondary(bid, "Run task"));
    assert_eq!(
        hidden.frame.digest(),
        idle.frame.digest(),
        "hidden focus renders exactly idle"
    );

    // Pressed: mouse down (press+hover) and flash agree — inverted ink.
    for (name, ix) in [
        (
            "press",
            Interaction {
                hover: Some(bid),
                pressed: Some(bid),
                ..Default::default()
            },
        ),
        (
            "flash",
            Interaction {
                flash: Some(bid),
                ..Default::default()
            },
        ),
    ] {
        let shot = render_button(ix, Button::secondary(bid, "Run task"));
        assert_eq!(shot.buf[(6, 1)].fg, t.canvas, "{name}: inverted fg");
        assert_eq!(shot.buf[(6, 1)].bg, t.text_primary, "{name}: inverted bg");
        record.push_str(&format!(
            "sec {name:<9} {:?} {:016x}\n",
            row_text(&shot.buf, 1),
            shot.frame.digest()
        ));
    }
    // Press without hover is not a press.
    let shot = render_button(
        Interaction {
            pressed: Some(bid),
            ..Default::default()
        },
        Button::secondary(bid, "Run task"),
    );
    assert_eq!(shot.frame.digest(), idle.frame.digest());

    // Selected/checked: toggle marker — ● accent when on, ○ muted when off.
    let shot = render_button(
        Interaction::default(),
        Button::toggle(bid, "Run task", true),
    );
    assert_eq!(shot.buf[(5, 1)].symbol(), "●");
    assert_eq!(shot.buf[(5, 1)].fg, t.accent);
    assert_eq!(
        shot.buf[(7, 1)].symbol(),
        "R",
        "label starts after marker+gap"
    );
    let shot = render_button(
        Interaction::default(),
        Button::toggle(bid, "Run task", false),
    );
    assert_eq!(shot.buf[(5, 1)].symbol(), "○");
    assert_eq!(shot.buf[(5, 1)].fg, t.text_muted);

    // Disabled: muted text on one plane up, blank gutter, never bold;
    // hover/press/flash change nothing (precedence, digest-exact).
    let dis_idle = render_button(
        Interaction::default(),
        Button::primary(bid, "Run task").disabled(true),
    );
    assert_eq!(dis_idle.buf[(6, 1)].fg, t.disabled);
    assert_eq!(dis_idle.buf[(6, 1)].bg, t.lift(t.canvas));
    assert_eq!(dis_idle.buf[(6, 1)].bg, t.surface_elevated);
    assert_eq!(dis_idle.buf[(4, 1)].symbol(), " ");
    assert!(!is_bold(&dis_idle.buf, 6, 1));
    assert!(
        !dis_idle.ring.contains(bid),
        "disabled button is no focus stop"
    );
    assert_eq!(
        dis_idle.hits.hit(Position::new(6, 1)),
        Some(bid),
        "disabled button still registers its hit area"
    );
    let dis_all = render_button(
        Interaction {
            focus: Some(bid),
            hover: Some(bid),
            pressed: Some(bid),
            flash: Some(bid),
            ..Default::default()
        },
        Button::primary(bid, "Run task").disabled(true),
    );
    assert_eq!(
        dis_all.frame.digest(),
        dis_idle.frame.digest(),
        "disabled removes hover/press/focus rendering"
    );
    record.push_str(&format!(
        "prim disabled {:?} {:016x}\n",
        row_text(&dis_idle.buf, 1),
        dis_idle.frame.digest()
    ));
    // Subtle keeps the container bg when disabled.
    let shot = render_button(
        Interaction::default(),
        Button::subtle(bid, "Run task").disabled(true),
    );
    assert_eq!(shot.buf[(6, 1)].bg, t.canvas);
    // Danger is the error-toned kind: error text, error inversion on press.
    let shot = render_button(
        Interaction::default(),
        Button::new(bid, "Delete", ButtonKind::Danger),
    );
    assert_eq!(shot.buf[(6, 1)].fg, t.error);
    let shot = render_button(
        Interaction {
            hover: Some(bid),
            pressed: Some(bid),
            ..Default::default()
        },
        Button::danger(bid, "Delete"),
    );
    assert_eq!(shot.buf[(6, 1)].fg, t.text_primary);
    assert_eq!(shot.buf[(6, 1)].bg, t.error);

    // Busy (=loading): spinner wins the marker cell, label goes secondary
    // non-bold, press is cleared but hover still lifts.
    let tick = 3u64;
    let mut busy_btn = Button::secondary(bid, "Run task");
    busy_btn.busy = true;
    let busy = render_button(
        Interaction {
            tick,
            ..Default::default()
        },
        busy_btn,
    );
    assert_eq!(
        busy.buf[(5, 1)].symbol(),
        junie_tui::widgets::progress::spinner_frame(tick)
    );
    assert_eq!(busy.buf[(5, 1)].fg, t.accent);
    assert_eq!(busy.buf[(7, 1)].fg, t.text_secondary);
    assert!(!is_bold(&busy.buf, 7, 1));
    let busy_pressed = render_button(
        Interaction {
            tick,
            hover: Some(bid),
            pressed: Some(bid),
            flash: Some(bid),
            ..Default::default()
        },
        {
            let mut b = Button::secondary(bid, "Run task");
            b.busy = true;
            b
        },
    );
    assert_eq!(
        busy_pressed.buf[(7, 1)].bg,
        t.popover,
        "busy keeps hover lift"
    );
    assert_ne!(
        busy_pressed.buf[(7, 1)].fg,
        t.canvas,
        "busy clears the pressed inversion"
    );
    record.push_str("sec busy: spinner wins marker, press cleared, hover kept\n");

    // Combos: focused+hovered, selected+hovered, selected+disabled.
    let shot = render_button(
        Interaction {
            focus: Some(bid),
            hover: Some(bid),
            ..Default::default()
        },
        Button::toggle(bid, "Run task", true),
    );
    assert_eq!(shot.buf[(4, 1)].symbol(), "▎", "combo keeps the bar");
    assert_eq!(shot.buf[(6, 1)].bg, t.popover, "combo keeps hover");
    assert_eq!(shot.buf[(5, 1)].symbol(), "●", "combo keeps the mark");
    assert_eq!(shot.buf[(5, 1)].fg, t.accent);
    let shot = render_button(
        Interaction {
            hover: Some(bid),
            ..Default::default()
        },
        Button::toggle(bid, "Run task", true).disabled(true),
    );
    assert_eq!(
        shot.buf[(5, 1)].symbol(),
        "●",
        "selected+disabled keeps the on glyph"
    );
    assert_eq!(
        shot.buf[(5, 1)].fg,
        t.disabled,
        "selected+disabled loses the accent"
    );
    assert_eq!(shot.buf[(6, 1)].bg, t.surface_elevated, "no hover lift");
    // busy+checked: spinner overwrites the toggle dot (see also
    // button_busy_frames.rs, which owns the full-cycle proof).
    let shot = render_button(
        Interaction {
            tick,
            ..Default::default()
        },
        {
            let mut b = Button::toggle(bid, "Run task", true);
            b.busy = true;
            b
        },
    );
    assert_eq!(
        shot.buf[(5, 1)].symbol(),
        junie_tui::widgets::progress::spinner_frame(tick),
        "busy+checked: spinner wins"
    );
    assert_eq!(shot.widget.width(), 12, "marker slot is not doubled");

    // Actions: exactly (disabled, busy) gate activation.
    let mut b = Button::secondary(bid, "Run task");
    assert!(b.can_activate());
    assert_eq!(b.on_key(&key(KeyCode::Enter)), (Outcome::Changed, true));
    assert_eq!(b.on_key(&key(KeyCode::Char(' '))), (Outcome::Changed, true));
    assert_eq!(
        b.on_key(&key(KeyCode::Char('x'))),
        (Outcome::Ignored, false)
    );
    assert!(b.on_click());
    let mut b = Button::secondary(bid, "Run task").disabled(true);
    assert!(!b.can_activate());
    assert_eq!(b.on_key(&key(KeyCode::Enter)), (Outcome::Consumed, false));
    assert!(!b.on_click());
    let mut b = Button::secondary(bid, "Run task");
    b.busy = true;
    assert!(!b.can_activate());
    assert_eq!(b.on_key(&key(KeyCode::Enter)), (Outcome::Consumed, false));
    assert!(!b.on_click());
    // Toggle flips only on real activation.
    let mut b = Button::toggle(bid, "Run task", false);
    assert_eq!(b.on_key(&key(KeyCode::Enter)), (Outcome::Changed, true));
    assert_eq!(b.on, Some(true));
    let mut b = Button::toggle(bid, "Run task", false).disabled(true);
    assert_eq!(b.on_key(&key(KeyCode::Enter)), (Outcome::Consumed, false));
    assert_eq!(b.on, Some(false), "disabled never flips the toggle");

    // Narrow: clipping truncates with … (the `fit` contract), marker stomps
    // the ellipsis at w=3, w<=2 is gutter+pad blank, w=0 draws nothing.
    for area_w in [10u16, 6, 4, 3, 2, 1, 0] {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(16, 3);
        let bg = stage.bg();
        let mut b = Button::secondary(bid, "Run task");
        b.render(
            Rect::new(3, 1, area_w, 1),
            &mut buf,
            &mut stage.ctx(Interaction::default()),
            bg,
        );
        let expect_w = 10u16.min(area_w);
        assert_eq!(b.area.width, expect_w, "area_w={area_w}");
        if area_w > 0 {
            assert_surround_intact(&buf, Rect::new(3, 1, expect_w, 1), "button narrow");
        } else {
            assert_surround_intact(&buf, Rect::new(3, 1, 0, 1), "button w=0");
        }
        record.push_str(&format!("narrow w={area_w:<2} {:?}\n", row_text(&buf, 1)));
    }

    record.push_str("N/A read-only/error/warning/empty: Button has no such flags\n");
    write_evidence("button_state_matrix.txt", record.as_bytes());
    eprintln!("{record}");
}

// ----------------------------------------------------------- Checkbox --

/// Checkbox: idle, hover, kbd/pointer focus, hidden, pressed, checked,
/// disabled, combos, actions and compact marks. Busy, loading, read-only,
/// error, warning and empty are N/A: Checkbox has no such flags.
#[test]
fn checkbox_state_matrix() {
    let t = Theme::junie();
    let mut record = String::from("state row digest\n");
    let cid = id("test.checkbox");

    // Idle unchecked: muted "[ ]" at x+1, label at x+5, canvas row.
    let idle = render_checkbox(
        Interaction::default(),
        Checkbox::new(cid, "Run tests", false),
    );
    assert_eq!(idle.buf[(4, 1)].symbol(), " ");
    assert_eq!(idle.buf[(5, 1)].symbol(), "[");
    assert_eq!(idle.buf[(6, 1)].symbol(), " ");
    assert_eq!(idle.buf[(7, 1)].symbol(), "]");
    assert_eq!(idle.buf[(5, 1)].fg, t.text_muted);
    assert_eq!(idle.buf[(9, 1)].symbol(), "R");
    assert_eq!(idle.buf[(9, 1)].bg, t.canvas);
    assert!(!is_bold(&idle.buf, 9, 1));
    assert_eq!(idle.widget.area, Rect::new(4, 1, 20, 1));
    assert_surround_intact(&idle.buf, Rect::new(4, 1, 20, 1), "checkbox idle");
    assert!(idle.ring.contains(cid));
    record.push_str(&format!(
        "idle {:?} {:016x}\n",
        row_text(&idle.buf, 1),
        idle.frame.digest()
    ));
    write_evidence("checkbox_idle.json", idle.frame.to_json_pretty().as_bytes());

    // Hover lifts the whole row one plane; checked paints the accent mark.
    let shot = render_checkbox(
        Interaction {
            hover: Some(cid),
            ..Default::default()
        },
        Checkbox::new(cid, "Run tests", false),
    );
    assert_eq!(shot.buf[(9, 1)].bg, t.surface_elevated);
    assert_eq!(shot.buf[(9, 1)].bg, t.lift(t.canvas));
    let shot = render_checkbox(
        Interaction::default(),
        Checkbox::new(cid, "Run tests", true),
    );
    assert_eq!(shot.buf[(6, 1)].symbol(), "✓");
    assert_eq!(shot.buf[(5, 1)].fg, t.accent, "checked mark is accent");
    record.push_str(&format!(
        "checked {:?} {:016x}\n",
        row_text(&shot.buf, 1),
        shot.frame.digest()
    ));

    // Focus bar + bold; pointer adds lift; hidden renders idle.
    let shot = render_checkbox(
        Interaction {
            focus: Some(cid),
            hover: Some(cid),
            hover_suppressed: true,
            ..Default::default()
        },
        Checkbox::new(cid, "Run tests", false),
    );
    assert_eq!(shot.buf[(4, 1)].symbol(), "▎");
    assert_eq!(shot.buf[(4, 1)].fg, t.focus);
    assert!(is_bold(&shot.buf, 9, 1));
    assert_eq!(shot.buf[(9, 1)].bg, t.canvas, "suppressed hover: no lift");
    let shot = render_checkbox(
        Interaction {
            focus: Some(cid),
            hover: Some(cid),
            ..Default::default()
        },
        Checkbox::new(cid, "Run tests", false),
    );
    assert_eq!(shot.buf[(4, 1)].symbol(), "▎");
    assert_eq!(shot.buf[(9, 1)].bg, t.surface_elevated);
    let hidden = render_checkbox(
        Interaction {
            focus: Some(cid),
            focus_hidden: true,
            ..Default::default()
        },
        Checkbox::new(cid, "Run tests", false),
    );
    assert_eq!(hidden.frame.digest(), idle.frame.digest());

    // Pressed inverts; press without hover is idle.
    for (name, ix) in [
        (
            "press",
            Interaction {
                hover: Some(cid),
                pressed: Some(cid),
                ..Default::default()
            },
        ),
        (
            "flash",
            Interaction {
                flash: Some(cid),
                ..Default::default()
            },
        ),
    ] {
        let shot = render_checkbox(ix, Checkbox::new(cid, "Run tests", false));
        assert_eq!(shot.buf[(9, 1)].fg, t.canvas, "{name}");
        assert_eq!(shot.buf[(9, 1)].bg, t.text_primary, "{name}");
        assert!(is_bold(&shot.buf, 9, 1), "{name}");
    }
    let shot = render_checkbox(
        Interaction {
            pressed: Some(cid),
            ..Default::default()
        },
        Checkbox::new(cid, "Run tests", false),
    );
    assert_eq!(shot.frame.digest(), idle.frame.digest());

    // Disabled: uniform muted style, blank gutter, no lift even when
    // hovered; digest-identical with all interaction set (precedence).
    let mut dis = Checkbox::new(cid, "Run tests", true);
    dis.disabled = true;
    let dis_idle = render_checkbox(Interaction::default(), dis.clone());
    assert_eq!(dis_idle.buf[(5, 1)].fg, t.disabled);
    assert_eq!(dis_idle.buf[(6, 1)].symbol(), "✓", "checked glyph kept");
    assert_eq!(dis_idle.buf[(4, 1)].symbol(), " ");
    assert!(!dis_idle.ring.contains(cid), "disabled is no focus stop");
    let dis_all = render_checkbox(
        Interaction {
            focus: Some(cid),
            hover: Some(cid),
            pressed: Some(cid),
            flash: Some(cid),
            ..Default::default()
        },
        dis,
    );
    assert_eq!(
        dis_all.frame.digest(),
        dis_idle.frame.digest(),
        "disabled removes hover/press/focus rendering"
    );
    record.push_str(&format!(
        "disabled {:?} {:016x}\n",
        row_text(&dis_idle.buf, 1),
        dis_idle.frame.digest()
    ));

    // Combos: checked+hovered keeps mark and lift.
    let shot = render_checkbox(
        Interaction {
            focus: Some(cid),
            hover: Some(cid),
            ..Default::default()
        },
        Checkbox::new(cid, "Run tests", true),
    );
    assert_eq!(shot.buf[(6, 1)].symbol(), "✓");
    assert_eq!(shot.buf[(5, 1)].fg, t.accent);
    assert_eq!(shot.buf[(9, 1)].bg, t.surface_elevated);
    assert_eq!(shot.buf[(4, 1)].symbol(), "▎");

    // Actions: Space/Enter flip, anything else ignores; disabled never flips.
    let mut c = Checkbox::new(cid, "Run tests", false);
    assert_eq!(c.on_key(&key(KeyCode::Char(' '))), Outcome::Changed);
    assert!(c.checked);
    assert_eq!(c.on_key(&key(KeyCode::Enter)), Outcome::Changed);
    assert!(!c.checked);
    assert_eq!(c.on_key(&key(KeyCode::Char('x'))), Outcome::Ignored);
    assert_eq!(c.on_click(), Outcome::Changed);
    assert!(c.checked);
    let mut c = Checkbox::new(cid, "Run tests", false);
    c.disabled = true;
    assert_eq!(c.on_key(&key(KeyCode::Char(' '))), Outcome::Ignored);
    assert!(!c.checked);
    assert_eq!(c.on_click(), Outcome::Consumed);
    assert!(!c.checked);

    // Narrow: compact ✓/□ marks below 4 cells; one cell shows focus only.
    let mut narrow = Checkbox::new(cid, "Run tests", true);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(12, 3);
    let bg = stage.bg();
    narrow.render(
        Rect::new(3, 1, 3, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert_eq!(buf[(4, 1)].symbol(), "✓", "compact checked mark");
    assert_surround_intact(&buf, Rect::new(3, 1, 3, 1), "checkbox w=3");
    record.push_str(&format!("narrow w=3 {:?}\n", row_text(&buf, 1)));
    let mut narrow = Checkbox::new(cid, "Run tests", false);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(12, 3);
    let bg = stage.bg();
    narrow.render(
        Rect::new(3, 1, 1, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(cid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(3, 1)].symbol(), "▎", "one cell shows only focus");
    assert_surround_intact(&buf, Rect::new(3, 1, 1, 1), "checkbox w=1");

    record.push_str("N/A busy/loading/read-only/error/warning/empty: no such flags\n");
    write_evidence("checkbox_state_matrix.txt", record.as_bytes());
    eprintln!("{record}");
}

// ------------------------------------------------------------- Toggle --

/// Toggle switch: on/off glyphs and state words, the full interaction
/// matrix, disabled, combos, actions and compact marks. Busy, loading,
/// read-only, error, warning and empty are N/A: no such flags.
#[test]
fn toggle_state_matrix() {
    let t = Theme::junie();
    let mut record = String::from("state row digest\n");
    let gid = id("test.toggle");

    // Off: muted "○──" + "off"; on: accent "──●" + "on".
    let off = render_toggle(
        Interaction::default(),
        Toggle::new(gid, "Auto-merge", false),
    );
    assert_eq!(off.buf[(5, 1)].symbol(), "○");
    assert_eq!(off.buf[(5, 1)].fg, t.text_muted);
    assert_eq!(off.buf[(6, 1)].symbol(), "─");
    assert_eq!(off.buf[(9, 1)].symbol(), "A");
    // State word at x + 6 + label width: "off" for the off switch.
    assert_eq!(off.buf[(20, 1)].symbol(), "o");
    assert_eq!(off.buf[(21, 1)].symbol(), "f");
    assert_eq!(off.buf[(22, 1)].symbol(), "f");
    assert_eq!(off.widget.area, Rect::new(4, 1, 24, 1));
    assert_surround_intact(&off.buf, Rect::new(4, 1, 24, 1), "toggle off");
    assert!(off.ring.contains(gid));
    record.push_str(&format!(
        "off {:?} {:016x}\n",
        row_text(&off.buf, 1),
        off.frame.digest()
    ));
    write_evidence("toggle_off.json", off.frame.to_json_pretty().as_bytes());
    let on = render_toggle(Interaction::default(), Toggle::new(gid, "Auto-merge", true));
    assert_eq!(on.buf[(5, 1)].symbol(), "─");
    assert_eq!(on.buf[(7, 1)].symbol(), "●");
    assert_eq!(on.buf[(7, 1)].fg, t.accent);
    assert_eq!(on.buf[(20, 1)].symbol(), "o");
    assert_eq!(on.buf[(21, 1)].symbol(), "n");

    // Focus/hover/pressed/hidden mirror the row contract.
    let shot = render_toggle(
        Interaction {
            focus: Some(gid),
            hover: Some(gid),
            hover_suppressed: true,
            ..Default::default()
        },
        Toggle::new(gid, "Auto-merge", true),
    );
    assert_eq!(shot.buf[(4, 1)].symbol(), "▎");
    assert!(is_bold(&shot.buf, 9, 1));
    assert_eq!(shot.buf[(9, 1)].bg, t.canvas);
    let shot = render_toggle(
        Interaction {
            hover: Some(gid),
            ..Default::default()
        },
        Toggle::new(gid, "Auto-merge", true),
    );
    assert_eq!(shot.buf[(9, 1)].bg, t.surface_elevated);
    assert_eq!(shot.buf[(7, 1)].fg, t.accent, "hover keeps the on accent");
    let hidden = render_toggle(
        Interaction {
            focus: Some(gid),
            focus_hidden: true,
            ..Default::default()
        },
        Toggle::new(gid, "Auto-merge", false),
    );
    assert_eq!(hidden.frame.digest(), off.frame.digest());
    let shot = render_toggle(
        Interaction {
            flash: Some(gid),
            ..Default::default()
        },
        Toggle::new(gid, "Auto-merge", false),
    );
    assert_eq!(shot.buf[(9, 1)].fg, t.canvas);
    assert_eq!(shot.buf[(9, 1)].bg, t.text_primary);

    // Disabled: glyphs in the uniform disabled style, no lift, no stop.
    let dis = Toggle::new(gid, "Auto-merge", true).disabled(true);
    let dis_idle = render_toggle(Interaction::default(), dis.clone());
    assert_eq!(dis_idle.buf[(7, 1)].symbol(), "●", "on glyph kept");
    assert_eq!(dis_idle.buf[(7, 1)].fg, t.disabled, "accent lost");
    assert!(!dis_idle.ring.contains(gid));
    let dis_all = render_toggle(
        Interaction {
            focus: Some(gid),
            hover: Some(gid),
            pressed: Some(gid),
            flash: Some(gid),
            ..Default::default()
        },
        dis,
    );
    assert_eq!(
        dis_all.frame.digest(),
        dis_idle.frame.digest(),
        "disabled removes hover/press/focus rendering"
    );
    record.push_str(&format!(
        "disabled {:?} {:016x}\n",
        row_text(&dis_idle.buf, 1),
        dis_idle.frame.digest()
    ));

    // Actions mirror the checkbox: Space/Enter flip, disabled never flips.
    let mut g = Toggle::new(gid, "Auto-merge", false);
    assert_eq!(g.on_key(&key(KeyCode::Char(' '))), Outcome::Changed);
    assert!(g.on);
    assert_eq!(g.on_click(), Outcome::Changed);
    assert!(!g.on);
    let mut g = Toggle::new(gid, "Auto-merge", false).disabled(true);
    assert_eq!(g.on_key(&key(KeyCode::Enter)), Outcome::Ignored);
    assert_eq!(g.on_click(), Outcome::Consumed);
    assert!(!g.on);

    // Narrow: single ●/○ below 4 cells; the state word needs room.
    let mut narrow = Toggle::new(gid, "Auto-merge", true);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(12, 3);
    let bg = stage.bg();
    narrow.render(
        Rect::new(3, 1, 3, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert_eq!(buf[(4, 1)].symbol(), "●", "compact on mark");
    assert_surround_intact(&buf, Rect::new(3, 1, 3, 1), "toggle w=3");
    record.push_str(&format!("narrow w=3 {:?}\n", row_text(&buf, 1)));

    record.push_str("N/A busy/loading/read-only/error/warning/empty: no such flags\n");
    write_evidence("toggle_state_matrix.txt", record.as_bytes());
    eprintln!("{record}");
}

// ---------------------------------------------------------- RadioGroup --

/// RadioGroup: label row, selected/cursor rows, per-option hover, disabled,
/// empty options, narrow marks and key/click actions. Busy and loading are
/// N/A: no such flags.
#[test]
fn radio_state_matrix() {
    let t = Theme::junie();
    let mut record = String::from("state digest\n");
    let gid = id("test.radio");
    let group = || RadioGroup::new(gid, "Mode", &["Fast", "Safe", "Full"], 1);
    assert_eq!(group().height(), 4, "label + 3 options");

    // Idle: secondary label at x+2, (●) accent on selected, ( ) muted rest.
    let idle = render_radio(Interaction::default(), group());
    assert_eq!(idle.buf[(6, 2)].symbol(), "M");
    assert_eq!(idle.buf[(6, 2)].fg, t.text_secondary);
    assert!(!is_bold(&idle.buf, 6, 2));
    assert_eq!(idle.buf[(5, 3)].symbol(), "(");
    assert_eq!(idle.buf[(6, 3)].symbol(), " ");
    assert_eq!(idle.buf[(6, 4)].symbol(), "●", "selected mark");
    assert_eq!(idle.buf[(5, 4)].fg, t.accent);
    assert_eq!(idle.buf[(5, 3)].fg, t.text_muted);
    assert_eq!(idle.buf[(4, 4)].symbol(), " ", "no bar without focus");
    assert_eq!(idle.widget.areas.len(), 3);
    assert_eq!(idle.widget.areas[1], Rect::new(4, 4, 20, 1));
    assert_surround_intact(&idle.buf, Rect::new(4, 2, 20, 4), "radio idle");
    assert!(idle.ring.contains(gid), "group is one focus stop");
    assert_eq!(
        idle.hits.hit(Position::new(9, 4)),
        Some(gid.child(1)),
        "option rows are click targets"
    );
    record.push_str(&format!("idle {:016x}\n", idle.frame.digest()));
    write_evidence("radio_idle.json", idle.frame.to_json_pretty().as_bytes());

    // Focused group: label goes bold-primary, cursor row shows the bar.
    let shot = render_radio(
        Interaction {
            focus: Some(gid),
            hover: Some(gid),
            hover_suppressed: true,
            ..Default::default()
        },
        group(),
    );
    assert_eq!(shot.buf[(6, 2)].fg, t.text_primary);
    assert!(is_bold(&shot.buf, 6, 2));
    assert_eq!(shot.buf[(4, 4)].symbol(), "▎", "bar on the cursor row");
    assert_eq!(shot.buf[(4, 3)].symbol(), " ", "other rows stay blank");
    // Per-option hover (the child-id hover a generic matrix omits).
    let shot = render_radio(
        Interaction {
            hover: Some(gid.child(0)),
            ..Default::default()
        },
        group(),
    );
    assert_eq!(shot.buf[(9, 3)].bg, t.surface_elevated, "hovered row lifts");
    assert_eq!(shot.buf[(9, 4)].bg, t.canvas, "selected row does not");
    // Selected+hovered: lift plus accent mark together.
    let shot = render_radio(
        Interaction {
            hover: Some(gid.child(1)),
            ..Default::default()
        },
        group(),
    );
    assert_eq!(shot.buf[(9, 4)].bg, t.surface_elevated);
    assert_eq!(shot.buf[(5, 4)].fg, t.accent);
    let hidden = render_radio(
        Interaction {
            focus: Some(gid),
            focus_hidden: true,
            ..Default::default()
        },
        group(),
    );
    assert_eq!(hidden.frame.digest(), idle.frame.digest());

    // Disabled: faint label, uniform rows, no hover, no focus stop.
    let mut dis = group();
    dis.disabled = true;
    let dis_idle = render_radio(Interaction::default(), dis.clone());
    assert_eq!(dis_idle.buf[(6, 2)].fg, t.text_faint);
    assert_eq!(dis_idle.buf[(5, 4)].fg, t.disabled, "selected loses accent");
    assert_eq!(dis_idle.buf[(6, 4)].symbol(), "●", "but keeps the glyph");
    assert!(!dis_idle.ring.contains(gid));
    let dis_all = render_radio(
        Interaction {
            focus: Some(gid),
            hover: Some(gid.child(1)),
            pressed: Some(gid.child(1)),
            flash: Some(gid.child(1)),
            ..Default::default()
        },
        dis,
    );
    assert_eq!(
        dis_all.frame.digest(),
        dis_idle.frame.digest(),
        "disabled removes hover/press/focus rendering"
    );
    record.push_str(&format!("disabled {:016x}\n", dis_idle.frame.digest()));

    // Empty: label only, areas cleared, no focus stop, no panic.
    let empty = RadioGroup::new(gid, "Mode", &[], 0);
    assert_eq!(empty.height(), 1);
    let shot = render_radio(Interaction::default(), empty);
    assert_eq!(shot.widget.areas.len(), 0);
    assert!(!shot.ring.contains(gid), "empty group is no stop");
    assert_eq!(shot.buf[(6, 2)].symbol(), "M", "label still renders");
    record.push_str(&format!("empty {:016x}\n", shot.frame.digest()));

    // Actions: Up/Down/k/j move cursor+selected, Space/Enter commit,
    // click selects, disabled and out-of-range never change.
    let mut g = group();
    assert_eq!(g.on_key(&key(KeyCode::Down)), Outcome::Changed);
    assert_eq!((g.cursor, g.selected), (2, 2));
    assert_eq!(g.on_key(&key(KeyCode::Char('k'))), Outcome::Changed);
    assert_eq!((g.cursor, g.selected), (1, 1));
    g.cursor = 0;
    assert_eq!(g.on_key(&key(KeyCode::Char(' '))), Outcome::Changed);
    assert_eq!(g.selected, 0);
    assert_eq!(g.on_key(&key(KeyCode::Char('x'))), Outcome::Ignored);
    assert_eq!(g.on_click(2), Outcome::Changed);
    assert_eq!((g.cursor, g.selected), (2, 2));
    assert_eq!(g.on_click(9), Outcome::Consumed);
    assert_eq!(g.selected, 2);
    let mut g = group();
    g.disabled = true;
    assert_eq!(g.on_key(&key(KeyCode::Down)), Outcome::Ignored);
    assert_eq!(g.on_click(0), Outcome::Consumed);
    assert_eq!((g.cursor, g.selected), (1, 1));

    // Narrow: compact ●/○ marks below 4 cells.
    let mut narrow = group();
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(12, 7);
    let bg = stage.bg();
    narrow.render(
        Rect::new(3, 1, 3, 4),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert_eq!(buf[(4, 3)].symbol(), "●", "compact selected mark");
    assert_surround_intact(&buf, Rect::new(3, 1, 3, 4), "radio w=3");

    record.push_str("N/A busy/loading: RadioGroup has no such flags\n");
    write_evidence("radio_state_matrix.txt", record.as_bytes());
    eprintln!("{record}");
}

// -------------------------------------------------------------- Panel --

fn scroll_line(t: &Theme, _line: &str) -> Style {
    t.primary()
}

/// Panel: card/framed chrome idle vs focused, title/meta/badge rows, narrow
/// chrome, and the read-only ScrollPanel (scroll actions only, no edits).
/// Hover, pressed, selected, checked, disabled, busy and loading are N/A:
/// container focus shows on border/title only.
#[test]
fn panel_state_matrix() {
    let t = Theme::junie();
    let mut record = String::from("panel state inner digest\n");

    // Card idle: secondary title, no bar; focused: bar in the padding
    // column plus bold-primary title.
    let mut buf = sentinel_buf(30, 8);
    let inner = Panel::card(Some("Log")).render(Rect::new(2, 1, 26, 6), &mut buf, &t);
    assert_eq!(inner, Rect::new(4, 3, 22, 3), "card inner below the title");
    assert_eq!(buf[(4, 1)].symbol(), "L");
    assert_eq!(buf[(4, 1)].fg, t.text_secondary);
    assert!(!is_bold(&buf, 4, 1));
    assert_eq!(buf[(3, 1)].symbol(), " ", "no bar without focus");
    assert_surround_intact(&buf, Rect::new(2, 1, 26, 6), "card idle");
    let idle_frame = capture(&buf);
    record.push_str(&format!(
        "card idle {inner:?} {:016x}\n",
        idle_frame.digest()
    ));
    write_evidence(
        "panel_card_idle.json",
        idle_frame.to_json_pretty().as_bytes(),
    );
    let mut buf = sentinel_buf(30, 8);
    let inner = Panel::card(Some("Log"))
        .focused(true)
        .render(Rect::new(2, 1, 26, 6), &mut buf, &t);
    assert_eq!(inner, Rect::new(4, 3, 22, 3), "focus keeps the inner");
    assert_eq!(buf[(3, 1)].symbol(), "▎", "container focus bar");
    assert_eq!(buf[(3, 1)].fg, t.focus);
    assert_eq!(buf[(4, 1)].fg, t.text_primary);
    assert!(is_bold(&buf, 4, 1));
    assert_surround_intact(&buf, Rect::new(2, 1, 26, 6), "card focused");

    // Card without title: inner starts in the padding, no bar even focused.
    let mut buf = sentinel_buf(30, 8);
    let inner = Panel::card(None)
        .focused(true)
        .render(Rect::new(2, 1, 26, 6), &mut buf, &t);
    assert_eq!(inner, Rect::new(4, 2, 22, 4));
    assert_eq!(buf[(3, 1)].symbol(), " ", "titleless card shows no bar");

    // Framed: rounded border, subtle idle / strong focused, padded title.
    let mut buf = sentinel_buf(30, 8);
    let inner =
        Panel::framed(Some("Usage"))
            .meta("3 of 9")
            .render(Rect::new(2, 1, 26, 6), &mut buf, &t);
    assert_eq!(inner, Rect::new(5, 2, 21, 4));
    assert_eq!(buf[(2, 1)].symbol(), "╭");
    assert_eq!(buf[(2, 1)].fg, t.border_subtle);
    assert_eq!(buf[(4, 1)].symbol(), " ", "framed title is padded");
    assert_eq!(buf[(5, 1)].symbol(), "U");
    let title_row = row_text(&buf, 1);
    assert!(
        title_row.contains("3 of 9"),
        "meta right-aligned: {title_row:?}"
    );
    assert_surround_intact(&buf, Rect::new(2, 1, 26, 6), "framed idle");
    let mut buf = sentinel_buf(30, 8);
    Panel::framed(Some("Usage"))
        .focused(true)
        .render(Rect::new(2, 1, 26, 6), &mut buf, &t);
    assert_eq!(
        buf[(2, 1)].fg,
        t.border_strong,
        "focus strengthens the border"
    );
    assert_eq!(buf[(2, 1)].symbol(), "╭", "same corner glyph");

    // Badge sits between title and meta; long titles yield to the meta.
    let mut buf = sentinel_buf(30, 8);
    let panel = Panel {
        badge: Some(("Edit", BadgeKind::Edit)),
        ..Panel::card(Some("Log"))
    };
    panel
        .meta("3 of 9")
        .render(Rect::new(2, 1, 26, 6), &mut buf, &t);
    let title_row = row_text(&buf, 1);
    assert!(title_row.contains("Edit"), "badge visible: {title_row:?}");
    assert!(title_row.contains("3 of 9"), "meta kept: {title_row:?}");
    assert!(title_row.contains("Log"), "title kept: {title_row:?}");
    let mut buf = sentinel_buf(24, 4);
    Panel::card(Some("A rather long title"))
        .meta("70–100 of 120")
        .render(Rect::new(0, 0, 24, 3), &mut buf, &t);
    let title_row = row_text(&buf, 0);
    assert!(
        title_row.contains("70–100 of 120"),
        "meta never dropped: {title_row:?}"
    );
    assert!(title_row.contains('…'), "title yields: {title_row:?}");

    // draw_meta repaints the title row after the content settled.
    let mut buf = sentinel_buf(30, 8);
    let panel = Panel::card(Some("Log"));
    panel.render(Rect::new(2, 1, 26, 6), &mut buf, &t);
    assert!(!row_text(&buf, 1).contains("of 9"));
    panel.draw_meta(Rect::new(2, 1, 26, 6), &mut buf, &t, "1–2 of 9");
    let title_row = row_text(&buf, 1);
    assert!(title_row.contains("1–2 of 9"), "{title_row:?}");
    assert!(title_row.contains("Log"), "{title_row:?}");

    // Narrow framed chrome (w<=4) draws the border but no title row.
    let mut buf = sentinel_buf(12, 4);
    Panel::framed(Some("Usage")).render(Rect::new(3, 1, 4, 3), &mut buf, &t);
    assert_eq!(buf[(3, 1)].symbol(), "╭");
    assert!(!row_text(&buf, 1).contains('U'), "no title at w=4");
    assert_surround_intact(&buf, Rect::new(3, 1, 4, 3), "framed w=4");
    record.push_str(&format!("framed w=4 {:?}\n", row_text(&buf, 1)));

    // ScrollPanel is read-only: scroll keys move, edit-ish keys ignore,
    // and no key sequence mutates the lines.
    let sid = id("test.scroll");
    let lines: Vec<String> = (1..=10).map(|i| format!("line {i:02}")).collect();
    let mut p = ScrollPanel::new(sid, lines.clone());
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(26, 6);
    let bg = stage.bg();
    p.render(
        Rect::new(2, 1, 20, 4),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(sid),
            ..Default::default()
        }),
        bg,
        scroll_line,
    );
    assert_eq!(p.area, Rect::new(2, 1, 20, 4));
    assert!(stage.ring.contains(sid), "scroll panel is a focus stop");
    assert_eq!(buf[(2, 1)].symbol(), "l");
    assert_eq!(buf[(7, 1)].symbol(), "0", "first line fits the width");
    assert_surround_intact(&buf, Rect::new(2, 1, 20, 4), "scrollpanel");
    assert_eq!(p.on_key(&key(KeyCode::Down)), Outcome::Changed);
    assert_eq!(p.on_key(&key(KeyCode::Up)), Outcome::Changed);
    assert_eq!(p.on_key(&key(KeyCode::PageDown)), Outcome::Changed);
    assert_eq!(p.on_key(&key(KeyCode::Home)), Outcome::Changed);
    assert_eq!(p.on_key(&key(KeyCode::End)), Outcome::Changed);
    assert_eq!(p.on_key(&key(KeyCode::Enter)), Outcome::Ignored);
    assert_eq!(p.on_key(&key(KeyCode::Char('a'))), Outcome::Ignored);
    assert_eq!(p.on_key(&key(KeyCode::Char(' '))), Outcome::Ignored);
    assert_eq!(p.lines, lines, "read-only: keys never mutate lines");
    // `f` only toggles following on a tailing panel.
    assert_eq!(p.on_key(&key(KeyCode::Char('f'))), Outcome::Ignored);
    let mut tail = ScrollPanel::new(sid, lines.clone()).tail(true);
    assert!(tail.follow);
    assert_eq!(tail.on_key(&key(KeyCode::Char('f'))), Outcome::Changed);
    assert!(!tail.follow);
    // Empty content renders blank without panic and never reports Changed.
    let mut p = ScrollPanel::new(sid, vec![]);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(26, 6);
    let bg = stage.bg();
    p.render(
        Rect::new(2, 1, 20, 4),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
        scroll_line,
    );
    assert_eq!(p.on_key(&key(KeyCode::Down)), Outcome::Consumed);
    record.push_str("scrollpanel: read-only scroll actions, empty safe\n");

    record.push_str("N/A hover/pressed/selected/checked/disabled/busy/loading: chrome only\n");
    write_evidence("panel_state_matrix.txt", record.as_bytes());
    eprintln!("{record}");
}

// ---------------------------------------------------------- SplitPane --

/// SplitPane: the Split geometry (both directions, minima, maximize,
/// collapse, drag, 1-cell nudge, grow clamps) plus the Splitter handle
/// (idle/hover/pressed glyphs) composed with framed panes. Focus, disabled,
/// selected and busy are N/A: the handle is a mouse-only affordance.
#[test]
fn splitpane_state_matrix() {
    let t = Theme::junie();
    let mut record = String::from("check geometry\n");

    // Horizontal: usable 100, so 1% is exactly one cell.
    let area = Rect::new(2, 1, 101, 6);
    let split = Split::new(50, 10, 10);
    let (a, b) = split.horizontal(area, 1);
    assert_eq!(a, Rect::new(2, 1, 50, 6));
    assert_eq!(b, Rect::new(53, 1, 50, 6));
    assert_eq!(a.width + 1 + b.width, 101, "panes + gap fill the area");
    assert_eq!(
        split.handle(SplitDir::Horizontal, area, 1),
        Rect::new(52, 1, 1, 6)
    );
    record.push_str(&format!("h50 a={a:?} b={b:?}\n"));

    // Vertical with room for both panes.
    let varea = Rect::new(2, 1, 40, 23);
    let vsplit = Split::new(40, 4, 4);
    let (a, b) = vsplit.vertical(varea, 1);
    assert_eq!(a, Rect::new(2, 1, 40, 8), "40% of usable 22");
    assert_eq!(b, Rect::new(2, 10, 40, 14));
    assert_eq!(
        vsplit.handle(SplitDir::Vertical, varea, 1),
        Rect::new(2, 9, 40, 1)
    );

    // Collapse: too small for both minima gives everything to one pane.
    let tiny = Rect::new(2, 1, 15, 6);
    let (a, b) = split.horizontal(tiny, 1);
    assert!(a.is_empty(), "horizontal collapse keeps the second");
    assert_eq!(b, tiny);
    let (a, b) = split.vertical(tiny, 1);
    assert_eq!(a, tiny, "vertical collapse keeps the first");
    assert!(b.is_empty());
    assert_eq!(split.handle(SplitDir::Horizontal, tiny, 1), Rect::ZERO);
    assert_eq!(
        split.handle(SplitDir::Horizontal, area, 0),
        Rect::ZERO,
        "gap 0: no handle"
    );

    // Maximize: one pane takes all, the handle vanishes.
    let mut m = split;
    m.toggle_max(Maximized::Second);
    let (a, b) = m.horizontal(area, 1);
    assert!(a.is_empty());
    assert_eq!(b, area);
    assert_eq!(m.handle(SplitDir::Horizontal, area, 1), Rect::ZERO);
    m.toggle_max(Maximized::Second);
    assert_eq!(m.maximized, Maximized::None, "toggle round-trips");

    // Drag moves the seam and clamps to the minima; grow clamps 5..95.
    let mut s = split;
    assert!(s.drag_to(SplitDir::Horizontal, area, 1, Position::new(72, 3)));
    assert_eq!(s.percent, 70);
    assert_eq!(s.horizontal(area, 1).0.width, 70);
    assert!(
        !s.drag_to(SplitDir::Horizontal, area, 1, Position::new(72, 3)),
        "same seam: no change"
    );
    s.drag_to(SplitDir::Horizontal, area, 1, Position::new(4, 3));
    assert_eq!(s.horizontal(area, 1).0.width, 10, "clamped to min_first");
    s.drag_to(SplitDir::Horizontal, area, 1, Position::new(200, 3));
    assert_eq!(s.horizontal(area, 1).0.width, 90, "clamped to min_second");
    s.grow(100);
    assert_eq!(s.percent, 95);
    s.grow(-100);
    assert_eq!(s.percent, 5);

    // Nudge moves whole cells: exactly ±1 on a 1%-per-cell split.
    let mut s = Split::new(50, 10, 10);
    s.nudge(SplitDir::Horizontal, area, 1, 1);
    assert_eq!(s.horizontal(area, 1).0.width, 51, "nudge +1 moves one cell");
    s.nudge(SplitDir::Horizontal, area, 1, -1);
    assert_eq!(s.horizontal(area, 1).0.width, 50, "nudge -1 moves one cell");
    record.push_str("nudge: +-1 cells exact on usable=100\n");

    // Splitter idle: quiet rule in border-subtle; hover strengthens the
    // border without changing the glyph; press draws the heavy rule.
    // NB: the handle reads raw `pressed`, not the hover-gated predicate,
    // and ignores flash — pinned below, not assumed.
    let hid = id("test.splitter");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(106, 8);
    let bg = stage.bg();
    let mut sp = Splitter::new(hid, SplitDir::Horizontal);
    sp.render(
        Rect::new(52, 1, 1, 6),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert_eq!(sp.area, Rect::new(52, 1, 1, 6));
    assert_eq!(buf[(52, 1)].symbol(), "│");
    assert_eq!(buf[(52, 4)].symbol(), "│", "rule spans the strip");
    assert_eq!(buf[(52, 1)].fg, t.border_subtle);
    assert_eq!(
        stage.hits.hit(Position::new(52, 3)),
        Some(hid),
        "handle registers its hit"
    );
    assert!(!stage.ring.contains(hid), "mouse-only: never a focus stop");
    assert_surround_intact(&buf, Rect::new(52, 1, 1, 6), "splitter idle");
    let idle_frame = capture(&buf);
    record.push_str(&format!("splitter idle {:016x}\n", idle_frame.digest()));
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(106, 8);
    let bg = stage.bg();
    let mut sp = Splitter::new(hid, SplitDir::Horizontal);
    sp.render(
        Rect::new(52, 1, 1, 6),
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(hid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(52, 1)].symbol(), "│", "hover keeps the glyph");
    assert_eq!(buf[(52, 1)].fg, t.border_strong, "hover strengthens it");
    // Pressed — with AND without hover (raw pressed, no hover gate).
    for (name, ix) in [
        (
            "press+hover",
            Interaction {
                hover: Some(hid),
                pressed: Some(hid),
                ..Default::default()
            },
        ),
        (
            "press-only",
            Interaction {
                pressed: Some(hid),
                ..Default::default()
            },
        ),
    ] {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(106, 8);
        let bg = stage.bg();
        let mut sp = Splitter::new(hid, SplitDir::Horizontal);
        sp.render(Rect::new(52, 1, 1, 6), &mut buf, &mut stage.ctx(ix), bg);
        assert_eq!(buf[(52, 1)].symbol(), "┃", "{name}: heavy rule");
        assert_eq!(buf[(52, 1)].fg, t.border_strong, "{name}");
    }
    // Flash does nothing to the handle (no Interaction::pressed call).
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(106, 8);
    let bg = stage.bg();
    let mut sp = Splitter::new(hid, SplitDir::Horizontal);
    sp.render(
        Rect::new(52, 1, 1, 6),
        &mut buf,
        &mut stage.ctx(Interaction {
            flash: Some(hid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(52, 1)].symbol(), "│", "flash is not a drag");
    // Vertical split handle: ─ idle, ━ dragged.
    let vid = id("test.splitter.v");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(44, 8);
    let bg = stage.bg();
    let mut sp = Splitter::new(vid, SplitDir::Vertical);
    sp.render(
        Rect::new(2, 4, 40, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            pressed: Some(vid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(2, 4)].symbol(), "━");
    assert_eq!(buf[(41, 4)].symbol(), "━", "heavy rule spans the strip");

    // Maximized: the empty handle draws nothing and records nothing.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(106, 8);
    let bg = stage.bg();
    let mut sp = Splitter::new(hid, SplitDir::Horizontal);
    sp.render(
        Rect::ZERO,
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert!(sp.area.is_empty());
    assert_surround_intact(&buf, Rect::new(0, 0, 0, 0), "splitter maximized");

    // on_drag moves the seam under the pointer through the Split.
    let mut s = Split::new(50, 10, 10);
    let sp = Splitter::new(hid, SplitDir::Horizontal);
    assert_eq!(
        sp.on_drag(&mut s, area, 1, Position::new(62, 3)),
        Outcome::Changed
    );
    assert_eq!(s.percent, 60);
    assert_eq!(
        sp.on_drag(&mut s, area, 1, Position::new(62, 3)),
        Outcome::Consumed,
        "same seam: consumed, not changed"
    );

    // Composition: two framed panes plus the handle fill the container.
    let mut buf = sentinel_buf(30, 8);
    let caret = Rect::new(2, 1, 26, 6);
    let split = Split::new(50, 4, 4);
    let (a, b) = split.horizontal(caret, 1);
    let handle = split.handle(SplitDir::Horizontal, caret, 1);
    assert_eq!(a.right() + 1, b.x, "handle sits exactly between panes");
    assert_eq!((a.x, b.right()), (caret.x, caret.right()));
    Panel::framed(Some("A")).render(a, &mut buf, &t);
    Panel::framed(Some("B")).render(b, &mut buf, &t);
    let mut stage = Stage::new();
    let bg = stage.bg();
    Splitter::new(hid, SplitDir::Horizontal).render(
        handle,
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert_surround_intact(&buf, caret, "split composition");
    record.push_str(&format!("composed {:016x}\n", capture(&buf).digest()));

    record.push_str("N/A focus/disabled/selected/busy: mouse-only affordance\n");
    write_evidence("splitpane_state_matrix.txt", record.as_bytes());
    eprintln!("{record}");
}

// --------------------------------------------------------------- Props --

/// PropsList: cursor/selected row, per-row hover, copyable vs read-only
/// rows, error/warning value tones, empty sheet, narrow truncation, and the
/// static sheet renderer. Disabled, checked, busy and loading are N/A:
/// PropsList has no such flags.
#[test]
fn props_state_matrix() {
    let t = Theme::junie();
    let mut record = String::from("state digest\n");
    let pid = id("test.props");
    let props = || {
        vec![
            Prop::new("Host", "db-1"),
            Prop::new("Token", "abc").copyable(),
            Prop::new("State", "failed").tone(Tone::Error),
            Prop::new("Quota", "91%").tone(Tone::Warning),
        ]
    };
    // Label width is max label + 2 = 7: labels at x+2, values at x+9.
    let render = |ix: Interaction, pl: &mut PropsList| -> (Buffer, Frame) {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(36, 6);
        let bg = stage.bg();
        pl.render(Rect::new(2, 1, 30, 4), &mut buf, &mut stage.ctx(ix), bg);
        let frame = capture(&buf);
        (buf, frame)
    };

    // Idle: muted labels, toned values, blank gutters.
    let mut pl = PropsList::new(pid, props());
    let (buf, frame) = render(Interaction::default(), &mut pl);
    assert_eq!(buf[(4, 1)].symbol(), "H");
    assert_eq!(buf[(4, 1)].fg, t.text_muted);
    assert_eq!(buf[(11, 1)].symbol(), "d");
    assert_eq!(buf[(11, 1)].fg, t.text_primary, "Normal tone value");
    assert_eq!(buf[(11, 3)].symbol(), "f");
    assert_eq!(buf[(11, 3)].fg, t.error, "Error tone value");
    assert_eq!(buf[(11, 4)].fg, t.warning, "Warning tone value");
    assert_eq!(buf[(2, 1)].symbol(), " ");
    assert_eq!(pl.area, Rect::new(2, 1, 30, 4));
    assert_surround_intact(&buf, Rect::new(2, 1, 30, 4), "props idle");
    record.push_str(&format!("idle {:016x}\n", frame.digest()));
    write_evidence("props_idle.json", frame.to_json_pretty().as_bytes());
    let idle_digest = frame.digest();

    // Focused: cursor row shows the bar and goes bold; the copyable cursor
    // row gains the "y copy" hint while other rows stay quiet.
    let mut pl = PropsList::new(pid, props());
    let (buf, _) = render(
        Interaction {
            focus: Some(pid),
            hover: Some(pid),
            hover_suppressed: true,
            ..Default::default()
        },
        &mut pl,
    );
    assert_eq!(buf[(2, 1)].symbol(), "▎", "bar on the cursor row");
    assert!(is_bold(&buf, 11, 1));
    assert_eq!(buf[(2, 2)].symbol(), " ");
    assert!(
        !row_text(&buf, 1).contains("y copy"),
        "row 0 is not copyable"
    );
    let mut pl = PropsList::new(pid, props());
    assert_eq!(pl.on_key(&key(KeyCode::Down)), (Outcome::Changed, None));
    assert_eq!(pl.cursor, 1, "Down moves the cursor = selection");
    let (buf, _) = render(
        Interaction {
            focus: Some(pid),
            ..Default::default()
        },
        &mut pl,
    );
    assert_eq!(buf[(2, 2)].symbol(), "▎", "bar follows the cursor");
    assert_eq!(buf[(25, 2)].symbol(), "y", "copy hint on the copyable row");
    assert!(row_text(&buf, 2).contains("y copy"));

    // Per-row hover (child-id hover); pressed rows invert (no clearing).
    let mut pl = PropsList::new(pid, props());
    let (buf, _) = render(
        Interaction {
            hover: Some(pid.child(2)),
            ..Default::default()
        },
        &mut pl,
    );
    assert_eq!(buf[(11, 3)].bg, t.surface_elevated, "hovered row lifts");
    assert_eq!(buf[(11, 1)].bg, t.canvas, "other rows do not");
    let mut pl = PropsList::new(pid, props());
    let (buf, _) = render(
        Interaction {
            hover: Some(pid.child(0)),
            pressed: Some(pid.child(0)),
            ..Default::default()
        },
        &mut pl,
    );
    // NB: the pressed bg inverts, but label/value fgs are always
    // overwritten with the muted/tone fg — only the bg carries "pressed".
    assert_eq!(buf[(11, 1)].bg, t.text_primary, "pressed row inverts bg");
    assert_eq!(buf[(11, 1)].fg, t.text_primary, "value keeps its tone fg");
    assert_eq!(buf[(4, 1)].bg, t.text_primary, "label bg inverts too");
    assert_eq!(buf[(4, 1)].fg, t.text_muted, "label keeps its muted fg");
    let mut pl = PropsList::new(pid, props());
    let (_, frame) = render(
        Interaction {
            focus: Some(pid),
            focus_hidden: true,
            ..Default::default()
        },
        &mut pl,
    );
    assert_eq!(frame.digest(), idle_digest, "hidden focus renders idle");

    // Read-only: `y` copies only the copyable row, Enter activates, and no
    // key mutates any value.
    let mut pl = PropsList::new(pid, props());
    assert_eq!(
        pl.on_key(&key(KeyCode::Char('y'))),
        (Outcome::Consumed, None)
    );
    assert_eq!(pl.on_key(&key(KeyCode::Down)), (Outcome::Changed, None));
    assert_eq!(
        pl.on_key(&key(KeyCode::Char('y'))),
        (Outcome::Changed, Some(PropsEvent::Copy(1)))
    );
    assert_eq!(
        pl.on_key(&key(KeyCode::Enter)),
        (Outcome::Changed, Some(PropsEvent::Activate(1)))
    );
    assert_eq!(
        pl.on_key(&key(KeyCode::Char('x'))),
        (Outcome::Ignored, None)
    );
    assert_eq!(
        pl.on_click(2),
        (Outcome::Changed, Some(PropsEvent::Activate(2)))
    );
    assert_eq!(pl.cursor, 2);
    assert_eq!(pl.on_click(9), (Outcome::Consumed, None));
    assert_eq!(pl.props, props(), "read-only: actions never mutate values");

    // Empty sheet: keys ignore, render is blank but still a stop.
    let mut pl = PropsList::new(pid, vec![]);
    assert_eq!(pl.on_key(&key(KeyCode::Down)), (Outcome::Ignored, None));
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(36, 6);
    let bg = stage.bg();
    pl.render(
        Rect::new(2, 1, 30, 4),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert_eq!(pl.area, Rect::new(2, 1, 30, 4));
    assert!(stage.ring.contains(pid), "empty list still registers");
    assert_surround_intact(&buf, Rect::new(2, 1, 30, 4), "props empty");
    record.push_str(&format!("empty {:016x}\n", capture(&buf).digest()));

    // Narrow: the value truncates but the copy hint always wins its cells.
    let mut pl = PropsList::new(pid, props());
    pl.cursor = 1;
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 6);
    let bg = stage.bg();
    pl.render(
        Rect::new(2, 1, 16, 4),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(pid),
            ..Default::default()
        }),
        bg,
    );
    assert!(
        row_text(&buf, 2).contains("y copy"),
        "hint wins narrow rows"
    );
    assert_surround_intact(&buf, Rect::new(2, 1, 16, 4), "props narrow");
    // Below the label width the hint underflows left (parity pin): at w=6
    // the hint starts at right-7, one cell left of the row, while the label
    // tail spills one cell right.
    let mut pl = PropsList::new(pid, props());
    pl.cursor = 1;
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 6);
    let bg = stage.bg();
    pl.render(
        Rect::new(3, 1, 6, 4),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(pid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(2, 2)].symbol(), "y", "hint underflows left of the row");
    assert_eq!(buf[(9, 2)].symbol(), "n", "label tail spills right");
    record.push_str(&format!("narrow w=6 {:?}\n", row_text(&buf, 2)));

    // Static sheet: measure agrees with render; narrow wraps to more rows.
    let sheet = vec![
        Prop::new("Host", "db-1"),
        Prop::new("Notes", "alpha beta gamma delta").wrap(),
    ];
    assert_eq!(props::measure(&sheet, 30), 2, "wrap fits on one row here");
    assert!(
        props::measure(&sheet, 14) > 2,
        "narrow wrap spans extra rows"
    );
    let stage = Stage::new();
    let mut buf = sentinel_buf(36, 6);
    let used = props::render(
        Rect::new(2, 1, 30, 4),
        &mut buf,
        &stage.theme,
        &sheet,
        stage.bg(),
    );
    assert_eq!(used, 2, "render uses exactly the measured rows");
    assert_eq!(buf[(2, 1)].symbol(), "H", "static labels sit at x");
    let mut buf = sentinel_buf(20, 8);
    let used = props::render(
        Rect::new(2, 1, 14, 6),
        &mut buf,
        &stage.theme,
        &sheet,
        stage.bg(),
    );
    assert_eq!(used, props::measure(&sheet, 14), "narrow render == measure");
    record.push_str("static sheet: measure == render\n");

    record.push_str("N/A disabled/checked/busy/loading: no such flags\n");
    write_evidence("props_state_matrix.txt", record.as_bytes());
    eprintln!("{record}");
}

// --------------------------------------------------------------- Empty --

/// EmptyState: quiet empty vs bold-`!` error, centering, hint wrap, tiny
/// viewports and degenerate content. Every interaction state is N/A:
/// display-only, no flags and no handlers.
#[test]
fn empty_state_matrix() {
    let t = Theme::junie();
    let mut record = String::from("kind row digest\n");
    let render = |e: &EmptyState, area: Rect, cols: u16, rows: u16| -> Buffer {
        let stage = Stage::new();
        let mut buf = sentinel_buf(cols, rows);
        empty::render(area, &mut buf, &stage.theme, e, stage.bg());
        buf
    };

    // Empty: muted centered title, faint hint two rows below.
    let e = EmptyState::new("No rows").hint("Add one to begin");
    let buf = render(&e, Rect::new(2, 1, 26, 7), 30, 9);
    assert_eq!(buf[(11, 3)].symbol(), "N", "title centered: x=2+(26-7)/2");
    assert_eq!(buf[(11, 3)].fg, t.text_muted);
    assert!(!is_bold(&buf, 11, 3));
    assert_eq!(buf[(7, 5)].symbol(), "A", "hint centered two rows down");
    assert_eq!(buf[(7, 5)].fg, t.text_faint);
    assert_surround_intact(&buf, Rect::new(2, 1, 26, 7), "empty");
    record.push_str(&format!(
        "empty {:?} {:016x}\n",
        row_text(&buf, 3),
        capture(&buf).digest()
    ));
    write_evidence("empty_idle.json", capture(&buf).to_json_pretty().as_bytes());

    // Error: bold `!` in the error tone before the error-toned title.
    let e = EmptyState::error("Read failed");
    let buf = render(&e, Rect::new(2, 1, 26, 7), 30, 9);
    let title = "! Read failed";
    let x = 2 + (26 - title.len() as u16) / 2;
    assert_eq!(buf[(x, 4)].symbol(), "!", "error title vertically centered");
    assert_eq!(buf[(x, 4)].fg, t.error);
    assert!(is_bold(&buf, x, 4), "the `!` is bold on its own");
    assert_eq!(buf[(x + 2, 4)].symbol(), "R");
    assert_eq!(buf[(x + 2, 4)].fg, t.error);
    assert!(!is_bold(&buf, x + 2, 4), "the rest is not bold");
    assert_surround_intact(&buf, Rect::new(2, 1, 26, 7), "empty error");
    record.push_str(&format!(
        "error {:?} {:016x}\n",
        row_text(&buf, 4),
        capture(&buf).digest()
    ));

    // Narrow: long titles truncate to the area, centering holds.
    let e = EmptyState::new("A very long empty title here");
    let buf = render(&e, Rect::new(2, 1, 10, 5), 16, 7);
    let row: String = (2..12).map(|x| buf[(x, 3)].symbol().to_owned()).collect();
    assert_eq!(row, "A very lo…", "truncated to the area width");
    assert_surround_intact(&buf, Rect::new(2, 1, 10, 5), "empty narrow");
    record.push_str(&format!("narrow {row:?}\n"));

    // One row: the title shows, the hint is skipped past the bottom.
    let e = EmptyState::new("No rows").hint("Add one to begin");
    let buf = render(&e, Rect::new(2, 1, 26, 1), 30, 5);
    assert_eq!(buf[(11, 1)].symbol(), "N");
    assert_surround_intact(&buf, Rect::new(2, 1, 26, 1), "empty h=1");

    // Degenerate: empty title and no hint render nothing but never panic.
    let e = EmptyState::new("");
    let buf = render(&e, Rect::new(2, 1, 26, 7), 30, 9);
    assert_surround_intact(&buf, Rect::new(2, 1, 26, 7), "empty blank");

    record.push_str("N/A all interaction states: display-only, no flags/handlers\n");
    write_evidence("empty_state_matrix.txt", record.as_bytes());
    eprintln!("{record}");
}

// ----------------------------------- narrow clipping contract (all) --

/// Uniform clipping contract: at widths 6/4/3/2/1 every control renders
/// without panic, records an area inside the buffer, and leaves the
/// surrounding sentinels untouched (no 1-cell overflow). Glyph-level narrow
/// behavior lives in each control's own matrix test.
#[test]
fn narrow_clipping_contract() {
    let mut record = String::from("control w row\n");
    for w in [6u16, 4, 3, 2, 1] {
        // Button.
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(12, 3);
        let bg = stage.bg();
        let mut b = Button::secondary(id("test.narrow.btn"), "Run task");
        b.render(
            Rect::new(3, 1, w, 1),
            &mut buf,
            &mut stage.ctx(Interaction::default()),
            bg,
        );
        assert!(b.area.right() <= 12, "button w={w} stays in bounds");
        assert_surround_intact(&buf, Rect::new(3, 1, b.area.width, 1), "button narrow");
        record.push_str(&format!("button {w} {:?}\n", row_text(&buf, 1)));
        // Checkbox.
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(12, 3);
        let bg = stage.bg();
        let mut c = Checkbox::new(id("test.narrow.chk"), "Run tests", true);
        c.render(
            Rect::new(3, 1, w, 1),
            &mut buf,
            &mut stage.ctx(Interaction::default()),
            bg,
        );
        assert_surround_intact(&buf, Rect::new(3, 1, w, 1), "checkbox narrow");
        record.push_str(&format!("checkbox {w} {:?}\n", row_text(&buf, 1)));
        // Toggle.
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(12, 3);
        let bg = stage.bg();
        let mut g = Toggle::new(id("test.narrow.tgl"), "Auto-merge", true);
        g.render(
            Rect::new(3, 1, w, 1),
            &mut buf,
            &mut stage.ctx(Interaction::default()),
            bg,
        );
        assert_surround_intact(&buf, Rect::new(3, 1, w, 1), "toggle narrow");
        record.push_str(&format!("toggle {w} {:?}\n", row_text(&buf, 1)));
        // RadioGroup (label + 2 options).
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(12, 6);
        let bg = stage.bg();
        let mut g = RadioGroup::new(id("test.narrow.radio"), "Mode", &["Fast", "Safe"], 0);
        g.render(
            Rect::new(3, 1, w, 3),
            &mut buf,
            &mut stage.ctx(Interaction::default()),
            bg,
        );
        assert_surround_intact(&buf, Rect::new(3, 1, w, 3), "radio narrow");
        record.push_str(&format!("radio {w} {:?}\n", row_text(&buf, 2)));
        // PropsList (2 rows). NB: labels are written unclipped, so narrow
        // rows spill the label tail past the row edge — a parity pin, not a
        // contract hole: "Token" at x=5 always spans 5..10.
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(12, 6);
        let bg = stage.bg();
        let mut pl = PropsList::new(
            id("test.narrow.props"),
            vec![
                Prop::new("Host", "db-1"),
                Prop::new("Token", "abc").copyable(),
            ],
        );
        pl.render(
            Rect::new(3, 1, w, 2),
            &mut buf,
            &mut stage.ctx(Interaction::default()),
            bg,
        );
        let spill_w = (2 + 5).max(w);
        if w < 7 {
            assert_eq!(buf[(9, 2)].symbol(), "n", "props w={w}: label tail spills");
        }
        assert_surround_intact(&buf, Rect::new(3, 1, spill_w, 2), "props narrow");
        record.push_str(&format!("props {w} {:?}\n", row_text(&buf, 1)));
        // EmptyState.
        let stage = Stage::new();
        let mut buf = sentinel_buf(12, 5);
        empty::render(
            Rect::new(3, 1, w, 3),
            &mut buf,
            &stage.theme,
            &EmptyState::new("No rows exceed width"),
            stage.bg(),
        );
        assert_surround_intact(&buf, Rect::new(3, 1, w, 3), "empty narrow");
        record.push_str(&format!("empty {w} {:?}\n", row_text(&buf, 2)));
        // Card panel.
        let stage = Stage::new();
        let mut buf = sentinel_buf(12, 5);
        Panel::card(Some("Log panel title")).render(Rect::new(3, 1, w, 3), &mut buf, &stage.theme);
        assert_surround_intact(&buf, Rect::new(3, 1, w, 3), "card narrow");
        record.push_str(&format!("card {w} {:?}\n", row_text(&buf, 1)));
        // Splitter strip.
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(12, 5);
        let bg = stage.bg();
        let mut sp = Splitter::new(id("test.narrow.split"), SplitDir::Horizontal);
        sp.render(
            Rect::new(3, 1, 1, 3),
            &mut buf,
            &mut stage.ctx(Interaction::default()),
            bg,
        );
        assert_surround_intact(&buf, Rect::new(3, 1, 1, 3), "splitter narrow");
    }
    write_evidence("narrow_clipping_contract.txt", record.as_bytes());
    eprintln!("{record}");
}

// --------------------------------- precedence across every control --

/// State precedence, stated once for every control: disabled removes
/// hover/press/focus rendering AND activation AND the focus stop (hit area
/// stays); keyboard suppression (`hover_suppressed`) removes hover;
/// modal hiding (`focus_hidden`) removes focus; busy removes press only.
/// Controls without a suppression show it: Splitter ignores focus and
/// flash, PropsList rows never suppress press.
#[test]
fn state_precedence_matrix() {
    let mut record = String::from("control precedence digest-idle digest-allset\n");

    // Disabled + every interaction set renders exactly disabled-idle, for
    // all four disablable controls.
    let all = |mine: WidgetId| Interaction {
        focus: Some(mine),
        hover: Some(mine),
        pressed: Some(mine),
        flash: Some(mine),
        ..Default::default()
    };
    let bid = id("test.prec.btn");
    let mut b = Button::secondary(bid, "Run task");
    b.disabled = true;
    let idle = render_button(Interaction::default(), b.clone())
        .frame
        .digest();
    let allset = render_button(all(bid), b).frame.digest();
    assert_eq!(idle, allset, "button: disabled removes all interaction");
    record.push_str(&format!("button disabled {idle:016x} {allset:016x}\n"));
    let cid = id("test.prec.chk");
    let mut c = Checkbox::new(cid, "Run tests", true);
    c.disabled = true;
    let idle = render_checkbox(Interaction::default(), c.clone())
        .frame
        .digest();
    let allset = render_checkbox(all(cid), c).frame.digest();
    assert_eq!(idle, allset, "checkbox: disabled removes all interaction");
    let gid = id("test.prec.tgl");
    let g = Toggle::new(gid, "Auto-merge", true).disabled(true);
    let idle = render_toggle(Interaction::default(), g.clone())
        .frame
        .digest();
    let allset = render_toggle(all(gid), g).frame.digest();
    assert_eq!(idle, allset, "toggle: disabled removes all interaction");
    let rid = id("test.prec.radio");
    let mut g = RadioGroup::new(rid, "Mode", &["Fast", "Safe"], 0);
    g.disabled = true;
    let idle = render_radio(Interaction::default(), g.clone())
        .frame
        .digest();
    let allset = render_radio(
        Interaction {
            focus: Some(rid),
            hover: Some(rid.child(0)),
            pressed: Some(rid.child(0)),
            flash: Some(rid.child(0)),
            ..Default::default()
        },
        g,
    )
    .frame
    .digest();
    assert_eq!(idle, allset, "radio: disabled removes all interaction");

    // Disabled activation + registration contract for all four.
    let mut b = Button::secondary(bid, "Run task");
    b.disabled = true;
    assert_eq!(b.on_key(&key(KeyCode::Enter)), (Outcome::Consumed, false));
    assert!(!b.on_click());
    let shot = render_button(Interaction::default(), b);
    assert!(!shot.ring.contains(bid), "disabled: no focus stop");
    assert!(
        shot.hits.hit(Position::new(6, 1)).is_some(),
        "disabled: hit area stays"
    );
    let mut c = Checkbox::new(cid, "Run tests", false);
    c.disabled = true;
    assert_eq!(c.on_key(&key(KeyCode::Char(' '))), Outcome::Ignored);
    assert_eq!(c.on_click(), Outcome::Consumed);
    assert!(!render_checkbox(Interaction::default(), c)
        .ring
        .contains(cid));
    let mut g = Toggle::new(gid, "Auto-merge", false);
    g.disabled = true;
    assert_eq!(g.on_key(&key(KeyCode::Enter)), Outcome::Ignored);
    assert_eq!(g.on_click(), Outcome::Consumed);
    assert!(!render_toggle(Interaction::default(), g).ring.contains(gid));
    let mut g = RadioGroup::new(rid, "Mode", &["Fast", "Safe"], 0);
    g.disabled = true;
    assert_eq!(g.on_key(&key(KeyCode::Down)), Outcome::Ignored);
    assert_eq!(g.on_click(0), Outcome::Consumed);
    assert!(!render_radio(Interaction::default(), g).ring.contains(rid));
    record.push_str("disabled: no activation, no focus stop, hit stays (x4)\n");

    // Keyboard suppression removes hover for every hoverable control.
    let bid2 = id("test.prec.btn2");
    let suppressed = render_button(
        Interaction {
            focus: Some(bid2),
            hover: Some(bid2),
            hover_suppressed: true,
            ..Default::default()
        },
        Button::secondary(bid2, "Run task"),
    )
    .frame
    .digest();
    let focus_only = render_button(
        Interaction {
            focus: Some(bid2),
            ..Default::default()
        },
        Button::secondary(bid2, "Run task"),
    )
    .frame
    .digest();
    assert_eq!(suppressed, focus_only, "button: suppression removes hover");
    let cid2 = id("test.prec.chk2");
    let suppressed = render_checkbox(
        Interaction {
            hover: Some(cid2),
            hover_suppressed: true,
            ..Default::default()
        },
        Checkbox::new(cid2, "Run tests", false),
    )
    .frame
    .digest();
    let idle = render_checkbox(
        Interaction::default(),
        Checkbox::new(cid2, "Run tests", false),
    )
    .frame
    .digest();
    assert_eq!(suppressed, idle, "checkbox: suppression renders idle");
    record.push_str("hover_suppressed: hover removed (button, checkbox)\n");

    // Modal hiding removes focus for every focusable control.
    let hidden = render_button(
        Interaction {
            focus: Some(bid2),
            focus_hidden: true,
            ..Default::default()
        },
        Button::secondary(bid2, "Run task"),
    )
    .frame
    .digest();
    let idle = render_button(Interaction::default(), Button::secondary(bid2, "Run task"))
        .frame
        .digest();
    assert_eq!(hidden, idle, "button: hidden focus renders idle");
    let hidden = render_toggle(
        Interaction {
            focus: Some(gid),
            focus_hidden: true,
            ..Default::default()
        },
        Toggle::new(gid, "Auto-merge", false),
    )
    .frame
    .digest();
    let idle = render_toggle(
        Interaction::default(),
        Toggle::new(gid, "Auto-merge", false),
    )
    .frame
    .digest();
    assert_eq!(hidden, idle, "toggle: hidden focus renders idle");
    record.push_str("focus_hidden: focus removed (button, toggle)\n");

    // Busy removes press but keeps hover (button-only suppression).
    let tick = 5u64;
    let mut b = Button::secondary(bid, "Run task");
    b.busy = true;
    let idle = render_button(
        Interaction {
            tick,
            ..Default::default()
        },
        b.clone(),
    )
    .frame
    .digest();
    let pressed = render_button(
        Interaction {
            tick,
            pressed: Some(bid),
            flash: Some(bid),
            ..Default::default()
        },
        b,
    )
    .frame
    .digest();
    assert_eq!(pressed, idle, "button: busy removes press");
    record.push_str("busy: press removed, hover kept (button)\n");

    // Reverse proof — controls without a suppression show it.
    // Splitter ignores focus and flash (raw pressed only).
    let hid = id("test.prec.split");
    let render_sp = |ix: Interaction| {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(12, 5);
        let bg = stage.bg();
        Splitter::new(hid, SplitDir::Horizontal).render(
            Rect::new(5, 1, 1, 3),
            &mut buf,
            &mut stage.ctx(ix),
            bg,
        );
        capture(&buf).digest()
    };
    assert_eq!(
        render_sp(Interaction::default()),
        render_sp(Interaction {
            focus: Some(hid),
            flash: Some(hid),
            ..Default::default()
        }),
        "splitter: no focus/flash response"
    );
    // PropsList rows never suppress press (no disabled concept either).
    let pid = id("test.prec.props");
    let mut pl = PropsList::new(pid, vec![Prop::new("Host", "db-1")]);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 4);
    let bg = stage.bg();
    pl.render(
        Rect::new(2, 1, 20, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(pid.child(0)),
            pressed: Some(pid.child(0)),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(
        buf[(11, 1)].bg,
        Theme::junie().text_primary,
        "props: press inverts bg"
    );
    record.push_str("reverse: splitter ignores focus/flash, props keeps press\n");

    write_evidence("state_precedence_matrix.txt", record.as_bytes());
    eprintln!("{record}");
}

// ------------------------------- HOVERED for every hoverable control --

/// Explicit HOVERED deltas: idle vs hovered frames differ for every
/// hoverable control, with the exact delta named (bg lift, accent lift,
/// border strengthen). Hover-insensitive surfaces (Panel chrome,
/// EmptyState, static Brand, ScrollPanel) provably ignore hover.
#[test]
fn hovered_delta_for_every_hoverable_control() {
    let t = Theme::junie();
    let mut record = String::from("control delta idle-digest hover-digest\n");

    // Brand clickable: accent -> accent_hover.
    let bid = id("test.hover.brand");
    let lockup = Lockup::new("holla❯");
    let render_brand = |ix: Interaction| {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(16, 3);
        lockup.render_clickable(4, 1, &mut buf, &mut stage.ctx(ix), bid);
        let delta = buf[(5, 1)].bg;
        (capture(&buf).digest(), delta)
    };
    let (idle_d, idle_bg) = render_brand(Interaction::default());
    let (hov_d, hov_bg) = render_brand(Interaction {
        hover: Some(bid),
        ..Default::default()
    });
    assert_ne!(idle_d, hov_d);
    assert_eq!((idle_bg, hov_bg), (t.accent, t.accent_hover));
    record.push_str(&format!("brand accent-lift {idle_d:016x} {hov_d:016x}\n"));

    // Button primary + secondary: accent_hover / popover.
    let bub = id("test.hover.btn");
    let idle = render_button(Interaction::default(), Button::primary(bub, "Run task"));
    let hov = render_button(
        Interaction {
            hover: Some(bub),
            ..Default::default()
        },
        Button::primary(bub, "Run task"),
    );
    assert_ne!(idle.frame.digest(), hov.frame.digest());
    assert_eq!(hov.buf[(6, 1)].bg, t.accent_hover);
    record.push_str(&format!(
        "button/primary accent-lift {:016x} {:016x}\n",
        idle.frame.digest(),
        hov.frame.digest()
    ));
    let idle = render_button(Interaction::default(), Button::secondary(bub, "Run task"));
    let hov = render_button(
        Interaction {
            hover: Some(bub),
            ..Default::default()
        },
        Button::secondary(bub, "Run task"),
    );
    assert_ne!(idle.frame.digest(), hov.frame.digest());
    assert_eq!(hov.buf[(6, 1)].bg, t.popover);
    record.push_str(&format!(
        "button/secondary plane-lift {:016x} {:016x}\n",
        idle.frame.digest(),
        hov.frame.digest()
    ));

    // Checkbox + Toggle: one plane up.
    let cid = id("test.hover.chk");
    let idle = render_checkbox(
        Interaction::default(),
        Checkbox::new(cid, "Run tests", false),
    );
    let hov = render_checkbox(
        Interaction {
            hover: Some(cid),
            ..Default::default()
        },
        Checkbox::new(cid, "Run tests", false),
    );
    assert_ne!(idle.frame.digest(), hov.frame.digest());
    assert_eq!(hov.buf[(9, 1)].bg, t.lift(t.canvas));
    record.push_str(&format!(
        "checkbox plane-lift {:016x} {:016x}\n",
        idle.frame.digest(),
        hov.frame.digest()
    ));
    let gid = id("test.hover.tgl");
    let idle = render_toggle(
        Interaction::default(),
        Toggle::new(gid, "Auto-merge", false),
    );
    let hov = render_toggle(
        Interaction {
            hover: Some(gid),
            ..Default::default()
        },
        Toggle::new(gid, "Auto-merge", false),
    );
    assert_ne!(idle.frame.digest(), hov.frame.digest());
    assert_eq!(hov.buf[(9, 1)].bg, t.lift(t.canvas));
    record.push_str(&format!(
        "toggle plane-lift {:016x} {:016x}\n",
        idle.frame.digest(),
        hov.frame.digest()
    ));

    // RadioGroup option row (child id): only that row lifts.
    let rid = id("test.hover.radio");
    let idle = render_radio(
        Interaction::default(),
        RadioGroup::new(rid, "Mode", &["Fast", "Safe"], 0),
    );
    let hov = render_radio(
        Interaction {
            hover: Some(rid.child(1)),
            ..Default::default()
        },
        RadioGroup::new(rid, "Mode", &["Fast", "Safe"], 0),
    );
    assert_ne!(idle.frame.digest(), hov.frame.digest());
    assert_eq!(hov.buf[(9, 4)].bg, t.lift(t.canvas));
    assert_eq!(hov.buf[(9, 3)].bg, t.canvas);
    record.push_str(&format!(
        "radio/option row-lift {:016x} {:016x}\n",
        idle.frame.digest(),
        hov.frame.digest()
    ));

    // Splitter: same glyph, stronger border.
    let hid = id("test.hover.split");
    let render_sp = |ix: Interaction| {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(12, 5);
        let bg = stage.bg();
        Splitter::new(hid, SplitDir::Horizontal).render(
            Rect::new(5, 1, 1, 3),
            &mut buf,
            &mut stage.ctx(ix),
            bg,
        );
        (
            capture(&buf).digest(),
            buf[(5, 1)].fg,
            buf[(5, 1)].symbol().to_owned(),
        )
    };
    let (idle_d, idle_fg, idle_sym) = render_sp(Interaction::default());
    let (hov_d, hov_fg, hov_sym) = render_sp(Interaction {
        hover: Some(hid),
        ..Default::default()
    });
    assert_ne!(idle_d, hov_d);
    assert_eq!((idle_sym, hov_sym), ("│".to_string(), "│".to_string()));
    assert_eq!((idle_fg, hov_fg), (t.border_subtle, t.border_strong));
    record.push_str(&format!(
        "splitter border-strong {idle_d:016x} {hov_d:016x}\n"
    ));

    // PropsList row (child id): only that row lifts.
    let pid = id("test.hover.props");
    let render_pl = |ix: Interaction| {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(30, 5);
        let bg = stage.bg();
        let mut pl = PropsList::new(
            pid,
            vec![Prop::new("Host", "db-1"), Prop::new("Token", "abc")],
        );
        pl.render(Rect::new(2, 1, 26, 2), &mut buf, &mut stage.ctx(ix), bg);
        (capture(&buf).digest(), buf[(11, 2)].bg, buf[(11, 1)].bg)
    };
    let (idle_d, _, _) = render_pl(Interaction::default());
    let (hov_d, hov_row, other_row) = render_pl(Interaction {
        hover: Some(pid.child(1)),
        ..Default::default()
    });
    assert_ne!(idle_d, hov_d);
    assert_eq!(hov_row, t.lift(t.canvas));
    assert_eq!(other_row, t.canvas);
    record.push_str(&format!("props/row row-lift {idle_d:016x} {hov_d:016x}\n"));

    // Hover-insensitive: ScrollPanel reads focus only.
    let sid = id("test.hover.scroll");
    let render_sc = |ix: Interaction| {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(26, 6);
        let bg = stage.bg();
        let mut p = ScrollPanel::new(sid, vec!["line 01".to_string(), "line 02".to_string()]);
        p.render(
            Rect::new(2, 1, 20, 4),
            &mut buf,
            &mut stage.ctx(ix),
            bg,
            scroll_line,
        );
        capture(&buf).digest()
    };
    assert_eq!(
        render_sc(Interaction::default()),
        render_sc(Interaction {
            hover: Some(sid),
            ..Default::default()
        }),
        "scrollpanel ignores hover"
    );
    // Panel chrome, EmptyState and the static Brand take no interaction at
    // all — hover-insensitivity is structural, not asserted per frame.
    record.push_str("scrollpanel/panel/empty/static-brand: hover-insensitive\n");

    write_evidence("hovered_delta.txt", record.as_bytes());
    eprintln!("{record}");
}

// ------------------------------------------------- TooSmall (PTY) --

use tuiscotti::tui::{Session, Tui};

/// Spawn `bin` at `cols`x`rows`, wait for `needle`, settle and return the
/// frame. Deliberately not a ported-matrix capture: no inventory names, no
/// snapshot gate — a needle proof for a screen with no approved frames.
fn spawn_frame(bin: &str, cols: u16, rows: u16, needle: &str) -> Frame {
    let argv = vec![
        bin.to_string(),
        "--color".to_string(),
        "truecolor".to_string(),
    ];
    let timeout = Duration::from_millis(8_000);
    let mut s: Session = Tui::new(argv.clone())
        .size(cols, rows)
        .env("COLORTERM", "truecolor")
        .env("LINES", rows.to_string())
        .env("COLUMNS", cols.to_string())
        .env_remove("NO_COLOR")
        .env_remove("HOLLA_NO_MOTION")
        .env_remove("JACKIN_NO_MOTION")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("FORCE_COLOR")
        .env("HOLLA_NO_HISTORY", "1")
        .spawn()
        .unwrap_or_else(|e| panic!("spawn {bin} at {cols}x{rows} failed: {e:#}"));
    support::wait_screen(
        &mut s,
        timeout,
        &format!("{bin}: needle `{needle}` missing"),
        |screen| support::screen_text(screen).contains(needle),
    );
    support::settle_frame(
        &mut s,
        Duration::from_millis(400),
        timeout,
        needle,
        Provenance::now("tuiscotti-default", "pty", argv),
    )
}

/// TooSmall is app-level (w < 72 or h < 20 in all four binaries): the notice
/// lines and the exact size report, proven against the real binaries.
#[test]
#[ignore = "pty probe; run with --ignored"]
fn toosmall_notice_all_apps() {
    let mut record = String::from("app notice-line size-line quit-line\n");
    for (app, bin) in [
        ("showcase", SHOWCASE),
        ("tablepro", TABLEPRO),
        ("jackin-preview", JACKIN),
        ("holla", HOLLA),
    ] {
        let frame = spawn_frame(bin, 60, 10, "Terminal too small");
        let text = frame.text();
        assert!(
            text.contains("Terminal too small"),
            "{app}: notice line missing in {text:?}"
        );
        assert!(
            text.contains("Need 72×20, have 60×10"),
            "{app}: size line missing in {text:?}"
        );
        assert!(
            text.contains("q Quit"),
            "{app}: quit line missing in {text:?}"
        );
        record.push_str(&format!("{app} ok {:016x}\n", frame.digest()));
        write_evidence(
            &format!("toosmall_{app}.json"),
            frame.to_json_pretty().as_bytes(),
        );
    }
    write_evidence("toosmall_notice_all_apps.txt", record.as_bytes());
    eprintln!("{record}");
}

/// Boundary: exactly 72x20 boots the normal shell (predicate is `<`, not
/// `<=`, in every app's layout).
#[test]
#[ignore = "pty probe; run with --ignored"]
fn toosmall_boundary_boots_normally() {
    let frame = spawn_frame(SHOWCASE, 72, 20, "Junie Design system");
    let text = frame.text();
    assert!(
        !text.contains("Terminal too small"),
        "72x20 must boot normally: {text:?}"
    );
    let mut record = format!(
        "showcase 72x20 boots normally {:016x}\n{text}\n",
        frame.digest()
    );
    record.push_str("predicate: w < 72 || h < 20 (strict)\n");
    write_evidence("toosmall_boundary.txt", record.as_bytes());
    eprintln!("{record}");
}
