//! Boundary motion: the deterministic choreography of the Construct entry
//! and exit rituals, the cockpit atmosphere and the Capsule handoff. Every
//! frame is a pure function of `(tick, area, mode)`; there is no retained
//! simulation grid and no wall-clock randomness, so a resize simply
//! re-evaluates the same function at the new size.
//!
//! Tones are theme tokens only: the white ladder (primary → ghost), the
//! accent chain for signal trails, and the pill (on-accent on accent).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use junie_tui::theme::Theme;

use crate::scenario::Motion;

pub const TICK_MS: u64 = 33;

// ------------------------------------------------------------- randomness

pub const MOTION_SEED: u64 = 0x4A41_434B_494E_5E5E;

/// Stateless mixer over three keyed lanes: the only randomness source.
#[inline]
pub const fn mix(a: u64, b: u64, c: u64) -> u64 {
    let mut z = a.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ b.wrapping_mul(0xD1B5_4A32_D192_ED03)
        ^ c.wrapping_mul(0x2545_F491_4F6C_DD1D)
        ^ MOTION_SEED;
    z ^= z >> 30;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 27;
    z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    z
}

pub const fn pct(h: u64) -> u64 {
    h % 100
}

const POOL: &[u8; 78] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz@#$%&*<>{}[]|/\\~";

pub fn glyph(x: u64, y: u64, epoch: u64) -> char {
    POOL[(mix(x, y, epoch) % 78) as usize] as char
}

// ------------------------------------------------------------------ tones

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// 4 primary · 3 secondary · 2 muted · 1 faint · 0 ghost.
    Ladder(u8),
    Accent,
    /// On-accent text on the accent fill (the ` jackin❯ ` pill).
    Pill,
    PillChevron,
}

fn ladder_color(t: &Theme, i: u8) -> Color {
    match i {
        0 => t.text_ghost,
        1 => t.text_faint,
        2 => t.text_muted,
        3 => t.text_secondary,
        _ => t.text_primary,
    }
}

/// Style for a tone dimmed by `dim` steps; `None` when the cell is gone.
pub fn style(t: &Theme, tone: Tone, dim: u8) -> Option<Style> {
    match tone {
        Tone::Ladder(i) => {
            if dim > i {
                None
            } else {
                Some(Style::new().fg(ladder_color(t, i - dim)).bg(t.canvas))
            }
        }
        Tone::Accent => match dim {
            0 => Some(Style::new().fg(t.accent).bg(t.canvas)),
            1 => Some(Style::new().fg(t.accent_hover).bg(t.canvas)),
            2 => Some(Style::new().fg(t.accent_pressed).bg(t.canvas)),
            _ => None,
        },
        Tone::Pill => {
            if dim == 0 {
                // the one brand treatment: the same lockup the shell shows
                return Some(junie_tui::widgets::brand::Lockup::style(t));
            }
            let bg = match dim {
                1 | 2 => t.accent_bg,
                _ => return None,
            };
            Some(
                Style::new()
                    .fg(if dim == 0 {
                        t.text_on_accent
                    } else {
                        t.text_muted
                    })
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            )
        }
        Tone::PillChevron => {
            let bg = match dim {
                0 => t.accent,
                1 | 2 => t.accent_bg,
                _ => return None,
            };
            Some(
                Style::new()
                    .fg(if dim == 0 {
                        t.text_primary
                    } else {
                        t.text_secondary
                    })
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            )
        }
    }
}

fn put(buf: &mut Buffer, x: u16, y: u16, ch: char, st: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        let mut s = [0u8; 4];
        cell.set_symbol(ch.encode_utf8(&mut s));
        cell.set_style(st);
    }
}

pub fn fill_canvas(buf: &mut Buffer, area: Rect, t: &Theme) {
    junie_tui::ui::ctx::fill(buf, area, Style::new().bg(t.canvas).fg(t.text_primary));
}

/// Dim an existing buffer region by `steps` ladder steps in token space
/// (the Capsule handoff cross-fade).
pub fn dim_buffer(buf: &mut Buffer, area: Rect, steps: u8, t: &Theme) {
    if steps == 0 {
        return;
    }
    let ladder = [
        t.text_ghost,
        t.text_faint,
        t.text_muted,
        t.text_secondary,
        t.text_primary,
    ];
    for pos in area.positions() {
        let Some(cell) = buf.cell_mut(pos) else {
            continue;
        };
        let st = cell.style();
        let fg = st.fg.unwrap_or(t.text_primary);
        let idx = ladder.iter().position(|c| *c == fg);
        let new_fg = match idx {
            Some(i) => {
                if (steps as usize) > i {
                    None
                } else {
                    Some(ladder[i - steps as usize])
                }
            }
            None if fg == t.accent || fg == t.success || fg == t.focus => match steps {
                1 => Some(t.accent_hover),
                2 => Some(t.accent_pressed),
                _ => None,
            },
            None if fg == t.error || fg == t.warning => {
                if steps >= 4 {
                    None
                } else {
                    Some(ladder[4 - steps as usize - 1])
                }
            }
            None => {
                if steps >= 3 {
                    None
                } else {
                    Some(t.text_faint)
                }
            }
        };
        let bg = st.bg.unwrap_or(t.canvas);
        let new_bg = if bg == t.canvas {
            t.canvas
        } else if bg == t.accent {
            if steps <= 2 { t.accent_bg } else { t.canvas }
        } else if steps >= 2 {
            t.canvas
        } else {
            t.surface
        };
        match new_fg {
            Some(c) => {
                let mut ns = Style::new().fg(c).bg(new_bg);
                if st.add_modifier.contains(Modifier::BOLD) {
                    ns = ns.add_modifier(Modifier::BOLD);
                }
                cell.set_style(ns);
            }
            None => {
                cell.set_symbol(" ");
                cell.set_style(Style::new().fg(new_bg).bg(new_bg));
            }
        }
    }
}

// --------------------------------------------------------------- glitch

/// A cell that resolves from noise into its final glyph.
#[derive(Debug, Clone, Copy)]
pub struct GlitchCell {
    pub x: u16,
    pub y: u16,
    pub target: char,
    pub tone: Tone,
}

/// Resolve tick for a cell: 2..=15.
fn resolve_at(x: u16, y: u16) -> u64 {
    2 + mix(x as u64, y as u64, 3) % 14
}

/// Glyph and tone at local tick `j` of a resolve that lasts `len` ticks.
fn resolve(cell: &GlitchCell, j: u64, tick: u64, len: u64) -> (char, Tone) {
    let r = resolve_at(cell.x, cell.y);
    let scramble_tone = match cell.tone {
        Tone::Pill | Tone::PillChevron => Tone::Pill,
        _ => Tone::Ladder(2),
    };
    if j < r || (j < len && pct(mix(cell.x as u64, cell.y as u64, tick)) < 8) {
        (glyph(cell.x as u64, cell.y as u64, tick), scramble_tone)
    } else {
        (cell.target, cell.tone)
    }
}

/// Paint a set of cells resolving over `len` ticks; blank cells inside the
/// bounding box show sparse noise while resolving. `dim` dims the result.
#[allow(clippy::too_many_arguments)]
fn paint_glitch(
    buf: &mut Buffer,
    cells: &[GlitchCell],
    bbox: Rect,
    j: u64,
    tick: u64,
    len: u64,
    dim: u8,
    t: &Theme,
) {
    for c in cells {
        let (ch, tone) = resolve(c, j, tick, len);
        if let Some(st) = style(t, tone, dim) {
            put(buf, c.x, c.y, ch, st);
        }
    }
    if j < len {
        for pos in bbox.positions() {
            if cells.iter().any(|c| c.x == pos.x && c.y == pos.y) {
                continue;
            }
            let r = resolve_at(pos.x, pos.y);
            if j < r
                && pct(mix(pos.x as u64, pos.y as u64, 5)) < 35
                && let Some(st) = style(t, Tone::Ladder(1), dim)
            {
                put(
                    buf,
                    pos.x,
                    pos.y,
                    glyph(pos.x as u64, pos.y as u64, tick),
                    st,
                );
            }
        }
    }
}

// ----------------------------------------------------------------- mark

/// The identity mark is the terminal pill, never large art: the current
/// product's brand rule, kept.
pub const PILL: &str = " jackin❯ ";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkVariant {
    /// The pill with the caption two rows below (roomy terminals).
    Full,
    /// The pill with the caption directly below.
    Compact,
}

pub fn mark_variant(area: Rect) -> MarkVariant {
    if area.height >= 30 {
        MarkVariant::Full
    } else {
        MarkVariant::Compact
    }
}

fn center(area: Rect) -> (u16, u16) {
    (
        area.x + area.width / 2,
        area.y + (area.height / 2).saturating_sub(1),
    )
}

fn pill_cells(x0: u16, y: u16) -> Vec<GlitchCell> {
    PILL.chars()
        .enumerate()
        .map(|(i, ch)| GlitchCell {
            x: x0 + i as u16,
            y,
            target: ch,
            tone: if ch == '❯' {
                Tone::PillChevron
            } else {
                Tone::Pill
            },
        })
        .collect()
}

/// Cells of the identity mark and its bounding box.
pub fn mark_cells(area: Rect, variant: MarkVariant) -> (Vec<GlitchCell>, Rect) {
    let (cx, cy) = center(area);
    let x0 = cx.saturating_sub(4);
    let y = match variant {
        MarkVariant::Full => cy.saturating_sub(2),
        MarkVariant::Compact => cy.saturating_sub(1),
    };
    (
        pill_cells(x0, y),
        Rect::new(x0, y, PILL.chars().count() as u16, 1),
    )
}

fn caption_cells(area: Rect, text: &str, y: u16) -> (Vec<GlitchCell>, Rect) {
    let (cx, _) = center(area);
    let n = text.chars().count() as u16;
    let x0 = cx.saturating_sub(n / 2);
    let cells = text
        .chars()
        .enumerate()
        .map(|(i, ch)| GlitchCell {
            x: x0 + i as u16,
            y,
            target: ch,
            tone: Tone::Ladder(4),
        })
        .collect();
    (cells, Rect::new(x0, y, n, 1))
}

fn draw_text(buf: &mut Buffer, area: Rect, text: &str, y: u16, tone: Tone, t: &Theme) {
    let Some(st) = style(t, tone, 0) else { return };
    let (cx, _) = center(area);
    let n = text.chars().count() as u16;
    let x0 = cx.saturating_sub(n / 2);
    for (i, ch) in text.chars().enumerate() {
        put(buf, x0 + i as u16, y, ch, st);
    }
}

fn draw_hint(buf: &mut Buffer, area: Rect, key: &str, action: &str, t: &Theme) {
    let text = format!("{key} {action}");
    let n = text.chars().count() as u16;
    if area.width < n + 4 {
        return;
    }
    let x = area.right().saturating_sub(n + 2);
    let y = area.bottom().saturating_sub(1);
    let ks = Style::new()
        .fg(t.text_muted)
        .bg(t.canvas)
        .add_modifier(Modifier::BOLD);
    let as_ = Style::new().fg(t.text_faint).bg(t.canvas);
    for (i, ch) in key.chars().enumerate() {
        put(buf, x + i as u16, y, ch, ks);
    }
    for (i, ch) in action.chars().enumerate() {
        put(
            buf,
            x + key.chars().count() as u16 + 1 + i as u16,
            y,
            ch,
            as_,
        );
    }
}

// ----------------------------------------------------------------- rain

pub struct Curve {
    pub w: Vec<u32>,
    pub s: Vec<u32>,
}

impl Curve {
    fn build(len: usize, f: impl Fn(u32) -> u32) -> Self {
        let mut w = Vec::with_capacity(len);
        let mut s = Vec::with_capacity(len);
        let mut acc = 0u32;
        for k in 0..len as u32 {
            let v = f(k);
            acc += v;
            w.push(v);
            s.push(acc);
        }
        Self { w, s }
    }

    /// Intro: 0.25 → 4.0 rows per tick, accelerating.
    pub fn intro() -> Self {
        Self::build(110, |k| 250 + 3750 * k * k / (109 * 109))
    }

    /// Outro: 4.0 → 0.25 rows per tick, decelerating.
    pub fn outro() -> Self {
        Self::build(100, |k| 4000 - 3750 * k * k / (99 * 99))
    }
}

#[derive(Debug, Clone, Copy)]
struct ColumnParams {
    speed: u64,
    trail: u64,
    gap: u64,
    period: u64,
    phase: u64,
    signal: bool,
    order: u64,
}

fn column_params(x: u16, rows: u16) -> ColumnParams {
    let m = mix(x as u64, 1, 0);
    let speed = 1 + m % 3;
    let trail = 8 + (m >> 8) % 13;
    let gap = 4 + (m >> 16) % 12;
    let period = rows as u64 + trail + gap;
    ColumnParams {
        speed,
        trail,
        gap,
        period,
        phase: (m >> 24) % period,
        signal: (m >> 40).is_multiple_of(8),
        order: (m >> 48) % 100,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RainSpec {
    /// Percent of columns active.
    pub density: u8,
    /// Cumulative distance in milli-rows.
    pub s_milli: u32,
    pub streak: bool,
    pub dim_steps: u8,
    pub epoch: u64,
}

fn rain_cell(spec: RainSpec, cp: ColumnParams, area: Rect, x: u16, y: u16) -> Option<(char, Tone)> {
    if cp.order >= spec.density as u64 {
        return None;
    }
    let head = (cp.phase + cp.speed * spec.s_milli as u64 / 1000) % cp.period;
    let head_y = head as i64 - cp.gap as i64;
    let age = head_y - (y - area.y) as i64;
    if age < 0 || age >= cp.trail as i64 {
        return None;
    }
    let ch = if spec.streak && (3..=9).contains(&age) {
        '│'
    } else {
        glyph(x as u64, y as u64, spec.epoch)
    };
    let signal_len = if cp.signal { 4 } else { 2 };
    let tone = match age {
        0 => Tone::Ladder(4),
        a if a <= signal_len => Tone::Accent,
        3..=5 => Tone::Ladder(3),
        6..=9 => Tone::Ladder(2),
        10..=14 => Tone::Ladder(1),
        _ => Tone::Ladder(0),
    };
    Some((ch, tone))
}

/// Paint column streams over `area`, skipping `exclude` rectangles and,
/// when given, the row band `band` (inclusive) where the underlying screen
/// stays visible.
pub fn paint_rain(
    buf: &mut Buffer,
    area: Rect,
    spec: RainSpec,
    exclude: &[Rect],
    band: Option<(u16, u16)>,
    t: &Theme,
) {
    for x in area.left()..area.right() {
        let cp = column_params(x, area.height);
        if cp.order >= spec.density as u64 {
            continue;
        }
        for y in area.top()..area.bottom() {
            if let Some((lo, hi)) = band
                && y >= lo
                && y <= hi
            {
                continue;
            }
            if exclude.iter().any(|r| r.contains((x, y).into())) {
                continue;
            }
            if let Some((ch, tone)) = rain_cell(spec, cp, area, x, y)
                && let Some(st) = style(t, tone, spec.dim_steps)
            {
                put(buf, x, y, ch, st);
            }
        }
    }
}

// ---------------------------------------------------------------- intro

pub const P_START: [u64; 3] = [0, 46, 92];
pub const P_LEN: [u64; 3] = [46, 46, 42];
pub const P_HOLD: [u64; 3] = [20, 20, 16];
pub const PHRASES: [&str; 3] = [
    "Stand up, operator…",
    "The host stays outside…",
    "Follow the green.",
];
pub const CAPTION: &str = "Knock, knock, operator.";
pub const MARK_START: u64 = 134;
pub const MARK_RESOLVE: u64 = 18;
pub const CAPTION_IN: u64 = 152;
pub const WARP_START: u64 = 206;
pub const INTRO_END: u64 = 316;
pub const REDUCED_HOLD: u64 = 45;
/// Reduced motion shows the resolved mark at this local tick.
const REDUCED_MARK_TICK: u64 = MARK_START + 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntroPhase {
    Phrases,
    Mark,
    Warp,
    Done,
}

impl IntroPhase {
    pub fn of(tick: u64) -> Self {
        if tick < MARK_START {
            IntroPhase::Phrases
        } else if tick < WARP_START {
            IntroPhase::Mark
        } else if tick < INTRO_END {
            IntroPhase::Warp
        } else {
            IntroPhase::Done
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntroState {
    pub tick: u64,
    pub mode: Motion,
}

impl IntroState {
    pub fn new(mode: Motion, frame: u64) -> Self {
        Self { tick: frame, mode }
    }

    pub fn phase(&self) -> IntroPhase {
        match self.mode {
            Motion::Reduced => {
                if self.tick < REDUCED_HOLD {
                    IntroPhase::Mark
                } else {
                    IntroPhase::Done
                }
            }
            _ => IntroPhase::of(self.tick),
        }
    }

    pub fn is_done(&self) -> bool {
        self.phase() == IntroPhase::Done
    }

    /// Advance one tick; returns whether anything moved.
    pub fn on_tick(&mut self) -> bool {
        if self.mode == Motion::Paused || self.is_done() {
            return false;
        }
        self.tick += 1;
        true
    }

    /// Enter/Esc: phrases jump to the warp, anything else finishes.
    pub fn skip(&mut self) {
        match self.mode {
            Motion::Reduced => self.tick = REDUCED_HOLD,
            _ => {
                self.tick = match IntroPhase::of(self.tick) {
                    IntroPhase::Phrases => WARP_START,
                    _ => INTRO_END,
                }
            }
        }
    }
}

fn phrase_tone(k: u64, hold: u64) -> Option<Tone> {
    if k < 12 {
        Some(Tone::Ladder((k / 3) as u8))
    } else if k < 12 + hold {
        Some(Tone::Ladder(4))
    } else if k < 12 + hold + 8 {
        Some(Tone::Ladder((3 - (k - 12 - hold) / 2) as u8))
    } else {
        None
    }
}

fn intro_density(k: u64) -> u8 {
    (if k < 30 {
        35 * k / 30
    } else if k < 80 {
        35 + 50 * (k - 30) / 50
    } else {
        85 * (109u64.saturating_sub(k)) / 30
    }) as u8
}

/// Render the intro at `state.tick`. `manager` paints the destination
/// screen (used during the collapse and at the end).
pub fn render_intro(
    buf: &mut Buffer,
    area: Rect,
    state: &IntroState,
    t: &Theme,
    curve: &Curve,
    manager: &mut dyn FnMut(&mut Buffer, Rect),
) {
    if area.is_empty() {
        return;
    }
    let (cx, cy) = center(area);
    let _ = cx;
    let variant = mark_variant(area);
    let tick = match state.mode {
        Motion::Reduced => REDUCED_MARK_TICK,
        _ => state.tick,
    };
    match IntroPhase::of(tick) {
        IntroPhase::Phrases => {
            fill_canvas(buf, area, t);
            for i in 0..3 {
                if tick >= P_START[i]
                    && tick < P_START[i] + P_LEN[i]
                    && let Some(tone) = phrase_tone(tick - P_START[i], P_HOLD[i])
                {
                    draw_text(buf, area, PHRASES[i], cy, tone, t);
                }
            }
            // bottom pill, the boundary marker of the host
            let py = area.bottom().saturating_sub(2);
            for c in pill_cells(cx.saturating_sub(4), py) {
                if let Some(st) = style(t, c.tone, 0) {
                    put(buf, c.x, c.y, c.target, st);
                }
            }
            draw_hint(buf, area, "Enter", "Skip", t);
        }
        IntroPhase::Mark => {
            fill_canvas(buf, area, t);
            let j = tick - MARK_START;
            let (cells, bbox) = mark_cells(area, variant);
            paint_glitch(buf, &cells, bbox, j, tick, MARK_RESOLVE, 0, t);
            if tick >= CAPTION_IN {
                let cj = tick - CAPTION_IN;
                let cap_y = match variant {
                    MarkVariant::Full => cy + 2,
                    MarkVariant::Compact => cy + 1,
                };
                let (mut cap, cbox) = caption_cells(area, CAPTION, cap_y);
                let ladder = (cj / 3).min(4) as u8;
                for c in &mut cap {
                    c.tone = Tone::Ladder(ladder);
                }
                paint_glitch(buf, &cap, cbox, cj, tick, 12, 0, t);
            }
            match state.mode {
                Motion::Reduced => draw_hint(buf, area, "Enter", "Continue", t),
                _ => draw_hint(buf, area, "Enter", "Skip", t),
            }
        }
        IntroPhase::Warp => {
            let k = tick - WARP_START;
            let density = intro_density(k);
            let spec = RainSpec {
                density,
                s_milli: curve.s[(k as usize).min(curve.s.len() - 1)],
                streak: curve.w[(k as usize).min(curve.w.len() - 1)] > 2000,
                dim_steps: if k < 12 { ((12 - k) / 3) as u8 } else { 0 },
                epoch: tick >> 2,
            };
            if k >= 80 {
                // collapse: the manager is revealed from the centre outwards
                manager(buf, area);
                let half = ((k - 80 + 1) * (area.height as u64 / 2 + 1) / 30) as u16;
                let lo = cy.saturating_sub(half);
                let hi = (cy + half).min(area.bottom().saturating_sub(1));
                // paint canvas outside the band
                for y in area.top()..area.bottom() {
                    if y < lo || y > hi {
                        fill_canvas(buf, Rect::new(area.x, y, area.width, 1), t);
                    }
                }
                paint_rain(buf, area, spec, &[], Some((lo, hi)), t);
            } else {
                fill_canvas(buf, area, t);
                let (cells, bbox) = mark_cells(area, variant);
                let halo = Rect::new(
                    bbox.x.saturating_sub(3),
                    bbox.y.saturating_sub(1),
                    bbox.width + 6,
                    bbox.height + 2,
                );
                let exclude: &[Rect] = if k < 30 {
                    std::slice::from_ref(&halo)
                } else {
                    &[]
                };
                paint_rain(buf, area, spec, exclude, None, t);
                if k < 30 {
                    // the mark dissolves cell by cell during ignition
                    let dim = if k >= 24 {
                        3
                    } else if k >= 16 {
                        2
                    } else if k >= 8 {
                        1
                    } else {
                        0
                    };
                    for c in &cells {
                        if mix(c.x as u64, c.y as u64, 7) % 30 >= k {
                            let tone_dim = match c.tone {
                                Tone::Pill | Tone::PillChevron => dim,
                                _ => 0,
                            };
                            if let Some(st) = style(t, c.tone, tone_dim) {
                                put(buf, c.x, c.y, c.target, st);
                            }
                        }
                    }
                }
            }
            draw_hint(buf, area, "Enter", "Skip", t);
        }
        IntroPhase::Done => manager(buf, area),
    }
}

// ---------------------------------------------------------------- outro

pub const OUT_WARP: u64 = 100;
pub const OUT_CAPTION: u64 = 90;
pub const OUTRO_EPOCH: u64 = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutroPhase {
    Warp,
    Caption,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutroState {
    pub tick: u64,
    pub elapsed_secs: Option<u64>,
    pub mode: Motion,
}

impl OutroState {
    pub fn new(mode: Motion, elapsed_secs: Option<u64>, frame: u64) -> Self {
        Self {
            tick: frame,
            elapsed_secs,
            mode,
        }
    }

    pub fn end(&self) -> u64 {
        match self.mode {
            Motion::Reduced => REDUCED_HOLD,
            _ => {
                if self.elapsed_secs.is_some() {
                    OUT_WARP + OUT_CAPTION
                } else {
                    OUT_WARP
                }
            }
        }
    }

    pub fn phase(&self) -> OutroPhase {
        match self.mode {
            Motion::Reduced => {
                if self.tick < REDUCED_HOLD {
                    OutroPhase::Caption
                } else {
                    OutroPhase::Done
                }
            }
            _ => {
                if self.tick < OUT_WARP {
                    OutroPhase::Warp
                } else if self.tick < self.end() {
                    OutroPhase::Caption
                } else {
                    OutroPhase::Done
                }
            }
        }
    }

    pub fn is_done(&self) -> bool {
        self.phase() == OutroPhase::Done
    }

    pub fn on_tick(&mut self) -> bool {
        if self.mode == Motion::Paused || self.is_done() {
            return false;
        }
        self.tick += 1;
        true
    }

    /// Enter/Esc: warp jumps to the caption; the caption finishes.
    pub fn skip(&mut self) {
        self.tick = match self.phase() {
            OutroPhase::Warp => OUT_WARP,
            _ => self.end(),
        };
    }

    pub fn caption(&self) -> Option<String> {
        self.elapsed_secs.map(|s| {
            format!(
                "You were in the Construct for {}",
                crate::clock::format_duration(s)
            )
        })
    }
}

fn outro_density(k: u64) -> u8 {
    (if k < 40 {
        85
    } else {
        85 * (99u64.saturating_sub(k)) / 60
    }) as u8
}

pub fn render_outro(
    buf: &mut Buffer,
    area: Rect,
    state: &OutroState,
    t: &Theme,
    curve: &Curve,
    origin: &mut dyn FnMut(&mut Buffer, Rect),
) {
    if area.is_empty() {
        return;
    }
    let (_, cy) = center(area);
    match state.phase() {
        OutroPhase::Warp => {
            let k = state.tick;
            let spec = RainSpec {
                density: outro_density(k),
                s_milli: curve.s[(k as usize).min(curve.s.len() - 1)],
                streak: curve.w[(k as usize).min(curve.w.len() - 1)] > 2000,
                dim_steps: if k >= 70 { (1 + (k - 70) / 8) as u8 } else { 0 },
                epoch: (OUTRO_EPOCH + k) >> 2,
            };
            if k < 10 {
                origin(buf, area);
                let half = ((k + 1) * (area.height as u64 / 2 + 1) / 10) as u16;
                let lo = cy.saturating_sub(half);
                let hi = (cy + half).min(area.bottom().saturating_sub(1));
                for y in lo..=hi {
                    fill_canvas(buf, Rect::new(area.x, y, area.width, 1), t);
                }
                // rain only inside the consumed band
                let above = Rect::new(area.x, area.y, area.width, lo.saturating_sub(area.y));
                let below = Rect::new(
                    area.x,
                    hi + 1,
                    area.width,
                    area.bottom().saturating_sub(hi + 1),
                );
                paint_rain(buf, area, spec, &[above, below], None, t);
            } else {
                fill_canvas(buf, area, t);
                paint_rain(buf, area, spec, &[], None, t);
            }
            draw_hint(buf, area, "Enter", "Skip", t);
        }
        OutroPhase::Caption => {
            fill_canvas(buf, area, t);
            match (state.mode, state.caption()) {
                (Motion::Reduced, Some(text)) => {
                    draw_text(buf, area, &text, cy, Tone::Ladder(4), t)
                }
                (Motion::Reduced, None) => {
                    draw_text(buf, area, PILL.trim(), cy, Tone::Ladder(1), t)
                }
                (_, Some(text)) => {
                    let c = state.tick - OUT_WARP;
                    let (mut cells, bbox) = caption_cells(area, &text, cy);
                    let tone = if c >= 78 {
                        Tone::Ladder((3u64.saturating_sub((c - 78) / 3)) as u8)
                    } else {
                        Tone::Ladder(4)
                    };
                    for cell in &mut cells {
                        cell.tone = tone;
                    }
                    paint_glitch(buf, &cells, bbox, c, state.tick, MARK_RESOLVE, 0, t);
                }
                (_, None) => {}
            }
            draw_hint(buf, area, "Enter", "Close", t);
        }
        OutroPhase::Done => fill_canvas(buf, area, t),
    }
}

// ----------------------------------------------------------- atmosphere

/// Restrained signal field behind the launch cockpit: ghost/faint bodies,
/// at most one accent head per column, frozen on failure.
pub fn paint_atmosphere(
    buf: &mut Buffer,
    area: Rect,
    exclude: &[Rect],
    t_local: u64,
    running: bool,
    frozen: bool,
    t: &Theme,
) {
    for x in area.left()..area.right() {
        if pct(mix(x as u64, 11, 0)) >= 18 {
            continue;
        }
        let m = mix(x as u64, 12, 0);
        let period_t = 2 + m % 2;
        let trail = 6 + (m >> 8) % 5;
        let gap = 6 + (m >> 16) % 19;
        let period = area.height as u64 + trail + gap;
        let phase = (m >> 24) % period;
        let signal = (m >> 40).is_multiple_of(10);
        let head = (t_local / period_t + phase) % period;
        let head_y = head as i64 - gap as i64;
        for y in area.top()..area.bottom() {
            if exclude.iter().any(|r| r.contains((x, y).into())) {
                continue;
            }
            let age = head_y - (y - area.y) as i64;
            if !(0..=3).contains(&age) {
                continue;
            }
            let tone = if age == 0 {
                if signal && running && !frozen && t_local >= 15 {
                    Tone::Accent
                } else {
                    Tone::Ladder(1)
                }
            } else {
                Tone::Ladder(0)
            };
            let tone = if t_local < 15 { Tone::Ladder(0) } else { tone };
            if let Some(st) = style(t, tone, 0) {
                put(buf, x, y, glyph(x as u64, y as u64, t_local >> 3), st);
            }
        }
    }
}

pub const HANDOFF_LEN: u64 = 12;

/// Dim steps for the cockpit (`Some`) or the Capsule (`None` = not yet)
/// at handoff tick `h`.
pub fn handoff_stage(h: u64) -> HandoffStage {
    match h {
        0..=3 => HandoffStage::CockpitDim((h + 1) as u8),
        4..=5 => HandoffStage::Canvas,
        6..=10 => HandoffStage::CapsuleDim((10 - h) as u8),
        _ => HandoffStage::Capsule,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandoffStage {
    CockpitDim(u8),
    Canvas,
    CapsuleDim(u8),
    Capsule,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_boundaries() {
        assert_eq!(IntroPhase::of(0), IntroPhase::Phrases);
        assert_eq!(IntroPhase::of(133), IntroPhase::Phrases);
        assert_eq!(IntroPhase::of(134), IntroPhase::Mark);
        assert_eq!(IntroPhase::of(205), IntroPhase::Mark);
        assert_eq!(IntroPhase::of(206), IntroPhase::Warp);
        assert_eq!(IntroPhase::of(315), IntroPhase::Warp);
        assert_eq!(IntroPhase::of(316), IntroPhase::Done);
    }

    #[test]
    fn skip_targets() {
        let mut s = IntroState::new(Motion::Full, 20);
        s.skip();
        assert_eq!(s.tick, WARP_START);
        s.tick = 150;
        s.skip();
        assert_eq!(s.tick, INTRO_END);
        let mut p = IntroState::new(Motion::Paused, 300);
        assert!(!p.on_tick());
        p.skip();
        assert!(p.is_done());
        let mut r = IntroState::new(Motion::Reduced, 0);
        assert_eq!(r.phase(), IntroPhase::Mark);
        r.skip();
        assert!(r.is_done());
        let mut o = OutroState::new(Motion::Full, Some(8040), 30);
        o.skip();
        assert_eq!(o.tick, OUT_WARP);
        assert_eq!(o.phase(), OutroPhase::Caption);
        o.skip();
        assert!(o.is_done());
        let mut n = OutroState::new(Motion::Full, None, 30);
        n.skip();
        assert!(n.is_done());
        assert_eq!(
            OutroState::new(Motion::Full, Some(8040), 0)
                .caption()
                .as_deref(),
            Some("You were in the Construct for 2 h 14 min")
        );
    }

    #[test]
    fn curves_and_density() {
        let c = Curve::intro();
        assert_eq!(c.w[0], 250);
        assert_eq!(c.w[109], 4000);
        assert_eq!(*c.s.last().unwrap(), 165_578);
        let o = Curve::outro();
        assert_eq!(o.w[0], 4000);
        assert_eq!(o.w[99], 250);
        assert_eq!(intro_density(0), 0);
        assert_eq!(intro_density(30), 35);
        assert_eq!(intro_density(109), 0);
        assert_eq!(outro_density(39), 85);
        assert_eq!(outro_density(99), 0);
        assert_eq!(phrase_tone(0, 20), Some(Tone::Ladder(0)));
        assert_eq!(phrase_tone(12, 20), Some(Tone::Ladder(4)));
        assert_eq!(phrase_tone(45, 20), None);
    }

    #[test]
    fn rain_is_deterministic_and_restrained() {
        let t = Theme::junie();
        let curve = Curve::intro();
        let area = Rect::new(0, 0, 80, 24);
        let mut a = Buffer::empty(area);
        let mut b = Buffer::empty(area);
        let st = IntroState::new(Motion::Paused, 282);
        let mut noop = |_: &mut Buffer, _: Rect| {};
        render_intro(&mut a, area, &st, &t, &curve, &mut noop);
        render_intro(&mut b, area, &st, &t, &curve, &mut noop);
        assert_eq!(a, b);
        // at most four accent cells per column
        for x in 0..80u16 {
            let n = (0..24u16)
                .filter(|&y| a[(x, y)].style().fg == Some(t.accent))
                .count();
            assert!(n <= 4, "column {x} has {n} accent cells");
        }
        assert_eq!(mark_variant(Rect::new(0, 0, 80, 24)), MarkVariant::Compact);
        assert_eq!(mark_variant(Rect::new(0, 0, 100, 30)), MarkVariant::Full);
    }
}
