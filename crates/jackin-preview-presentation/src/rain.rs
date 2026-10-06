//! Deterministic Jackin atmosphere and entry/exit timing.
//!
//! The renderer owns no product state.  It consumes semantic `Role`s through
//! the public facade when the shell paints it, while this module keeps the
//! exact virtual-frame contracts used by the preview and its tests.

use termrock::{Buffer, Color, Rect, Style, Surface, Theme};

use jackin_preview_domain::scenario::Motion;

/// Intro/outro cadence.  This is a product constant, not the runtime poll
/// interval; the virtual clock advances by the active route's cadence.
pub const TICK_MS: u64 = 33;
/// Seed used by all deterministic atmosphere fields.
pub const MOTION_SEED: u64 = 0x4A41_434B_494E_5E5E;
/// Number of two-tick glitch passes around the handoff.
pub const GLITCH_PASS_TICKS: u64 = 2;
/// Number of glitch passes in the intro and outro cadence.
pub const GLITCH_PASSES: u64 = 5;
/// Warp duration in 33 ms ticks.
pub const WARP_TICKS: u64 = 95;

const POOL: &[u8; 78] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz@#$%&*<>{}[]|/\\~";

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

/// Map a mixed value to a percentage without floating point.
pub const fn pct(value: u64) -> u64 {
    value % 100
}

/// Stable atmosphere glyph.
pub fn glyph(x: u64, y: u64, epoch: u64) -> char {
    POOL[(mix(x, y, epoch) % 78) as usize] as char
}

/// Semantic tone accepted by the compatibility painting helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// One of the five foreground ladder levels, ghost through primary.
    Ladder(u8),
    /// Accent trace.
    Accent,
}



pub fn style(theme: &Theme, tone: Tone, dim: u8) -> Option<Style> {
    match tone {
        Tone::Ladder(level) => {
            let eff = level.saturating_sub(dim);
            let fg = match eff {
                0 => theme.color.fg[4],
                1 => theme.color.fg[3],
                2 => theme.color.fg[2],
                3 => theme.color.fg[1],
                _ => theme.color.fg[0],
            };
            Some(Style::new().fg(fg).bg(theme.bg(Surface::Canvas)))
        }
        Tone::Accent => {
            if dim >= 3 {
                None
            } else if dim == 2 {
                Some(Style::new().fg(theme.color.fg[2]).bg(theme.bg(Surface::Canvas)))
            } else if dim == 1 {
                Some(Style::new().fg(theme.color.fg[3]).bg(theme.bg(Surface::Canvas)))
            } else {
                Some(Style::new().fg(theme.color.accent).bg(theme.bg(Surface::Canvas)))
            }
        }
    }
}

/// Fill a field with the theme canvas colour.
pub fn fill_canvas(buf: &mut Buffer, area: Rect, theme: &Theme) {
    let st = Style::new()
        .bg(theme.bg(Surface::Canvas))
        .fg(theme.color.fg[0]);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_symbol(" ");
                cell.set_style(st);
            }
        }
    }
}

/// Dim existing cells by replacing their foreground with a ladder step.
/// Caller-owned background and glyphs remain untouched.
pub fn dim_buffer(buf: &mut Buffer, area: Rect, steps: u8, theme: &Theme) {
    for row in area.rows() {
        for column in area.columns() {
            let x = column.x;
            let y = row.y;
            if let Some(cell) = buf.cell_mut((x, y)) {
                let fg = cell.fg;
                let level = theme
                    .color
                    .fg
                    .iter()
                    .position(|candidate| *candidate == fg)
                    .unwrap_or(4);
                let next = level.saturating_sub(usize::from(steps)).min(4);
                cell.set_fg(theme.color.fg.get(next).copied().unwrap_or(Color::Reset));
            }
        }
    }
}

const fn xorshift(seed: &mut u64) -> u64 {
    if *seed == 0 {
        *seed = 0xDEAD_BEEF_CAFE_1337;
    }
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Star {
    angle: f32,
    radius: f32,
    speed: f32,
}

/// One deterministic warp cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarpCell {
    /// Painted glyph.
    pub ch: char,
    /// Foreground ladder level, ghost through primary.
    pub level: u8,
    /// Whether the cell is an accent streak.
    pub accent: bool,
}

/// A radial star/warp field.
#[derive(Debug, Clone, PartialEq)]
pub struct Starfield {
    seed: u64,
    stars: Vec<Star>,
    cols: u16,
    rows: u16,
    cells: Vec<Option<WarpCell>>,
    /// Number of frames painted.
    pub frame: u64,
}

fn edge_radius(angle: f32, cx: f32, cy: f32) -> f32 {
    let dx = (angle.cos() * 2.0).abs();
    let dy = angle.sin().abs();
    let rx = if dx > 1e-3 { cx / dx } else { f32::MAX };
    let ry = if dy > 1e-3 { cy / dy } else { f32::MAX };
    rx.min(ry).max(1.0)
}

impl Starfield {
    /// Build a seeded field for `cols × rows`.
    pub fn new(cols: u16, rows: u16, salt: u64) -> Self {
        use std::f32::consts::PI;
        let rows = rows.max(1);
        let mut seed: u64 = 0x9E37_79B9_7F4A_7C15 ^ salt;
        let (cx, cy) = (cols as f32 / 2.0, rows as f32 / 2.0);
        let n = (cols as usize * rows as usize / 4).clamp(80, 2400);
        let stars = (0..n)
            .map(|_| {
                let angle = (xorshift(&mut seed) % 36000) as f32 / 36000.0 * 2.0 * PI;
                Star {
                    angle,
                    radius: (xorshift(&mut seed) % 1000) as f32 / 1000.0
                        * edge_radius(angle, cx, cy),
                    speed: 0.5 + (xorshift(&mut seed) % 100) as f32 / 100.0,
                }
            })
            .collect();
        Self {
            seed,
            stars,
            cols,
            rows,
            cells: vec![None; cols as usize * rows as usize],
            frame: 0,
        }
    }

    /// Advance one frame.
    pub fn advance(&mut self, accelerating: bool, f: u64) {
        use std::f32::consts::PI;
        let (cols, rows) = (self.cols as usize, self.rows as usize);
        self.cells.iter_mut().for_each(|c| *c = None);
        let cx = cols as f32 / 2.0;
        let cy = rows as f32 / 2.0;
        let max_r = (cx / 2.0).hypot(cy).max(1.0);
        let t = f as f32 / WARP_TICKS as f32;
        let warp_factor = if accelerating {
            0.2 + t * t * 5.0
        } else {
            0.2 + (1.0 - t).powi(2) * 5.0
        };
        let entry_fade = (f as f32 / 8.0).min(1.0);
        for i in 0..self.stars.len() {
            let mut star = self.stars[i];
            let prev = star.radius;
            star.radius += star.speed * warp_factor;
            let (dx, dy) = (star.angle.cos() * 2.0, star.angle.sin());
            let head_x = cx + dx * star.radius;
            let head_y = cy + dy * star.radius;
            if head_x < 0.0 || head_x >= cols as f32 || head_y < 0.0 || head_y >= rows as f32 {
                star.angle = (xorshift(&mut self.seed) % 36000) as f32 / 36000.0 * 2.0 * PI;
                star.radius = (xorshift(&mut self.seed) % 60) as f32 / 100.0;
                star.speed = 0.5 + (xorshift(&mut self.seed) % 100) as f32 / 100.0;
                self.stars[i] = star;
                continue;
            }
            let steps = ((1.0 + warp_factor * 1.4) as usize).max(1);
            for s in 0..=steps {
                let rr = prev + (star.radius - prev) * (s as f32 / steps as f32);
                let x = (cx + dx * rr).round();
                let y = (cy + dy * rr).round();
                if x < 0.0 || y < 0.0 {
                    continue;
                }
                let (xu, yu) = (x as usize, y as usize);
                if xu >= cols || yu >= rows {
                    continue;
                }
                let frac = (rr / max_r).clamp(0.0, 1.0);
                let streak = frac > 0.66 && warp_factor > 2.5;
                let ch = if frac > 0.66 {
                    if streak { '─' } else { '*' }
                } else if frac > 0.33 {
                    '+'
                } else {
                    '·'
                };
                let bright = (frac * 0.7 + warp_factor / 5.2 * 0.3).clamp(0.0, 1.0) * entry_fade;
                let level = (bright * 4.999) as u8;
                self.cells[yu * cols + xu] = Some(WarpCell {
                    ch,
                    level,
                    accent: streak && s == steps && bright > 0.7,
                });
            }
            self.stars[i] = star;
        }
        self.frame = f + 1;
    }

    /// Dimensions of the field.
    pub const fn size(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    /// Paint the last generated frame.
    pub fn paint(&self, buf: &mut Buffer, area: Rect, dim: u8, theme: &Theme) {
        for y in 0..self.rows.min(area.height) {
            for x in 0..self.cols.min(area.width) {
                let Some(c) = self.cells[y as usize * self.cols as usize + x as usize] else {
                    continue;
                };
                let tone = if c.accent {
                    Tone::Accent
                } else {
                    Tone::Ladder(c.level)
                };
                if c.level == 0 && !c.accent {
                    continue;
                }
                if let Some(resolved) = style(theme, tone, dim) {
                    put(
                        buf,
                        area.x.saturating_add(x),
                        area.y.saturating_add(y),
                        c.ch,
                        resolved,
                    );
                }
            }
        }
    }
}

fn put(buf: &mut Buffer, x: u16, y: u16, ch: char, st: Style) {
    if x < buf.area.right() && y < buf.area.bottom() {
        buf.set_string(x, y, ch.to_string(), st);
    }
}

const fn phrase_ticks(chars: u64, char_ms: u64, hold_ms: u64) -> u64 {
    (chars * char_ms + hold_ms).div_ceil(TICK_MS)
}

/// Phrase texts and original pacing.
pub const PHRASES: [(&str, u64, u64); 3] = [
    ("Stand up, operator…", 60, 950),
    ("Host stays outside…", 55, 950),
    ("Follow the green.", 50, 850),
];
/// Caption shown during the entry knock.
pub const CAPTION: &str = "Knock, knock, operator.";
/// Caption hold duration in milliseconds.
pub const CAPTION_HOLD_MS: u64 = 850;
/// First phrase duration in virtual ticks.
pub const P1_LEN: u64 = phrase_ticks(19, 60, 950);
/// Second phrase duration in virtual ticks.
pub const P2_LEN: u64 = phrase_ticks(19, 55, 950);
/// Third phrase duration in virtual ticks.
pub const P3_LEN: u64 = phrase_ticks(17, 50, 850);
/// Start of the knock phase.
pub const KNOCK_START: u64 = P1_LEN + P2_LEN + P3_LEN;
/// Duration of the knock phase.
pub const KNOCK_LEN: u64 = GLITCH_PASSES * GLITCH_PASS_TICKS + CAPTION_HOLD_MS.div_ceil(TICK_MS);
/// Start of the warp phase.
pub const WARP_START: u64 = KNOCK_START + KNOCK_LEN;
/// End of the intro ritual.
pub const INTRO_END: u64 = WARP_START + WARP_TICKS;
/// Reduced-motion intro hold duration.
pub const REDUCED_HOLD: u64 = 45;

/// Intro phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntroPhase {
    /// Entry phrases and the knock sequence.
    Phrases,
    /// Warp transition into the Construct.
    Warp,
    /// Entry ritual is complete.
    Done,
}

impl IntroPhase {
    /// Resolve phase at a virtual frame.
    pub const fn of(tick: u64) -> Self {
        if tick < WARP_START {
            Self::Phrases
        } else if tick < INTRO_END {
            Self::Warp
        } else {
            Self::Done
        }
    }
}

/// Deterministic intro state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntroState {
    /// Current virtual frame.
    pub tick: u64,
    /// Motion policy.
    pub mode: Motion,
}

impl IntroState {
    /// Construct at an exact frame.
    pub const fn new(mode: Motion, frame: u64) -> Self {
        Self { tick: frame, mode }
    }

    /// Current phase.
    pub const fn phase(&self) -> IntroPhase {
        match self.mode {
            Motion::Reduced if self.tick < REDUCED_HOLD => IntroPhase::Phrases,
            Motion::Reduced => IntroPhase::Done,
            _ => IntroPhase::of(self.tick),
        }
    }

    /// Whether the entry ritual finished.
    pub const fn is_done(&self) -> bool {
        matches!(self.phase(), IntroPhase::Done)
    }

    /// Advance one virtual frame.
    pub fn advance_tick(&mut self) -> bool {
        if self.mode == Motion::Paused || self.is_done() {
            return false;
        }
        self.tick = self.tick.saturating_add(1);
        true
    }

    /// Skip phrases to warp, then warp to done.
    pub fn skip(&mut self) {
        self.tick = match self.mode {
            Motion::Reduced => REDUCED_HOLD,
            _ => match self.phase() {
                IntroPhase::Phrases => WARP_START,
                IntroPhase::Warp => INTRO_END,
                IntroPhase::Done => INTRO_END,
            },
        };
    }
}

/// Outro phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutroPhase {
    /// Warp transition out of the Construct.
    Warp,
    /// Exit caption is being held.
    Caption,
    /// Exit ritual is complete.
    Done,
}

/// Deterministic exit ritual state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutroState {
    /// Current virtual frame.
    pub tick: u64,
    /// Duration spent inside the Construct, if discovery supplied it.
    pub elapsed_secs: Option<u64>,
    /// Motion policy.
    pub mode: Motion,
}

/// Warp duration for the outro.
pub const OUT_WARP: u64 = WARP_TICKS;
/// Glitch reveal plus the original 2.4-second caption hold duration.
pub const OUT_CAPTION: u64 = GLITCH_PASSES * GLITCH_PASS_TICKS + 2_400u64.div_ceil(TICK_MS);

impl OutroState {
    /// Construct at an exact frame.
    pub const fn new(mode: Motion, elapsed_secs: Option<u64>, frame: u64) -> Self {
        Self {
            tick: frame,
            elapsed_secs,
            mode,
        }
    }

    /// End frame.
    pub const fn end(&self) -> u64 {
        match self.mode {
            Motion::Reduced => REDUCED_HOLD,
            _ if self.elapsed_secs.is_some() => OUT_WARP + OUT_CAPTION,
            _ => OUT_WARP,
        }
    }

    /// Current phase.
    pub const fn phase(&self) -> OutroPhase {
        match self.mode {
            Motion::Reduced => {
                if self.tick < REDUCED_HOLD {
                    OutroPhase::Caption
                } else {
                    OutroPhase::Done
                }
            }
            _ if self.tick < OUT_WARP => OutroPhase::Warp,
            _ if self.tick < self.end() => OutroPhase::Caption,
            _ => OutroPhase::Done,
        }
    }

    /// Whether the exit ritual finished.
    pub const fn is_done(&self) -> bool {
        matches!(self.phase(), OutroPhase::Done)
    }

    /// Advance one virtual frame.
    pub fn advance_tick(&mut self) -> bool {
        if self.mode == Motion::Paused || self.is_done() {
            return false;
        }
        self.tick = self.tick.saturating_add(1);
        true
    }

    /// Skip warp to caption, then caption to done.
    pub fn skip(&mut self) {
        self.tick = match self.phase() {
            OutroPhase::Warp => OUT_WARP,
            OutroPhase::Caption | OutroPhase::Done => self.end(),
        };
    }

    /// Product wording for the elapsed caption.
    pub fn caption(&self) -> Option<String> {
        self.elapsed_secs.map(|secs| {
            format!(
                "You were in the Construct for {}",
                format_universe_duration(secs)
            )
        })
    }
}

/// Format the two largest duration units, using the original wording.
pub fn format_universe_duration(secs: u64) -> String {
    fn unit(n: u64, name: &str) -> String {
        format!("{n} {name}{}", if n == 1 { "" } else { "s" })
    }
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if days > 0 {
        format!("{} {}", unit(days, "day"), unit(hours, "hour"))
    } else if hours > 0 {
        format!("{} {}", unit(hours, "hour"), unit(minutes, "minute"))
    } else if minutes > 0 {
        format!("{} {}", unit(minutes, "minute"), unit(seconds, "second"))
    } else {
        unit(seconds, "second")
    }
}

/// Handoff stage used to coordinate cockpit/capsule cross-fade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandoffStage {
    /// Dim the cockpit by the given ladder step.
    CockpitDim(u8),
    /// Paint only the shared canvas during the transition.
    Canvas,
    /// Dim the capsule by the given ladder step.
    CapsuleDim(u8),
    /// Capsule is fully revealed.
    Capsule,
}

/// Resolve a handoff frame without consulting wall time.
pub const fn handoff_stage(frame: u64) -> HandoffStage {
    match frame {
        0..=3 => HandoffStage::CockpitDim((frame + 1) as u8),
        4..=5 => HandoffStage::Canvas,
        6..=10 => HandoffStage::CapsuleDim((10 - frame) as u8),
        _ => HandoffStage::Capsule,
    }
}

/// Number of handoff frames.
pub const HANDOFF_LEN: u64 = 12;

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

const OUTRO_SALT: u64 = 0x5F5F_4F55_5452_4F5F;

fn center(area: Rect) -> (u16, u16) {
    (
        area.x + area.width / 2,
        area.y + (area.height / 2).saturating_sub(1),
    )
}

fn draw_text(buf: &mut Buffer, area: Rect, text: &str, y: u16, tone: Tone, t: &Theme) {
    let n = text.chars().count() as u16;
    let x0 = area.x + area.width.saturating_sub(n) / 2;
    if let Some(st) = style(t, tone, 0) {
        for (i, ch) in text.chars().enumerate() {
            put(buf, x0 + i as u16, y, ch, st);
        }
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
    let empty_st = Style::new()
        .bg(t.bg(Surface::Canvas))
        .fg(t.color.fg[0]);
    for xx in x.saturating_sub(1)..area.right() {
        if let Some(cell) = buf.cell_mut((xx, y)) {
            cell.set_symbol(" ");
            cell.set_style(empty_st);
        }
    }
    let ks = Style::new()
        .fg(t.color.fg[2])
        .bg(t.bg(Surface::Canvas))
        .add_modifier(termrock::Modifier::BOLD);
    let as_ = Style::new()
        .fg(t.color.fg[3])
        .bg(t.bg(Surface::Canvas));
    buf.set_string(x, y, key, ks);
    buf.set_string(x + key.chars().count() as u16 + 1, y, action, as_);
}

fn draw_pill_bottom(buf: &mut Buffer, area: Rect, t: &Theme) {
    let w = 9;
    if area.height < 4 || area.width < w + 2 {
        return;
    }
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.bottom().saturating_sub(2);
    let st = Style::new()
        .fg(t.color.on_accent)
        .bg(t.color.accent)
        .add_modifier(termrock::Modifier::BOLD);
    buf.set_string(x, y, " jackin❯ ", st);
}

fn draw_typed(buf: &mut Buffer, area: Rect, text: &str, y: u16, shown: usize, t: &Theme) {
    let n = text.chars().count() as u16;
    let x0 = area.x + area.width.saturating_sub(n) / 2;
    if let Some(st) = style(t, Tone::Ladder(4), 0) {
        for (i, ch) in text.chars().take(shown).enumerate() {
            put(buf, x0 + i as u16, y, ch, st);
        }
    }
}

fn draw_glitched(buf: &mut Buffer, area: Rect, text: &str, y: u16, j: u64, tone: Tone, t: &Theme) {
    let n = text.chars().count() as u16;
    let x0 = area.x + area.width.saturating_sub(n) / 2;
    let Some(st) = style(t, tone, 0) else { return };
    let pass = j / GLITCH_PASS_TICKS;
    for (i, ch) in text.chars().enumerate() {
        let x = x0 + i as u16;
        let shown = if pass < GLITCH_PASSES && mix(x as u64, y as u64, pass) % 3 == 0 {
            glyph(x as u64, y as u64, pass)
        } else {
            ch
        };
        put(buf, x, y, shown, st);
    }
}

fn phrase_at(tick: u64) -> Option<(usize, usize)> {
    let mut start = 0;
    for (i, (text, char_ms, hold_ms)) in PHRASES.iter().enumerate() {
        let n = text.chars().count() as u64;
        let len = phrase_ticks(n, *char_ms, *hold_ms);
        if tick < start + len {
            let k = tick - start;
            let shown = ((k * TICK_MS) / char_ms).min(n) as usize;
            return Some((i, shown));
        }
        start += len;
    }
    None
}

/// Render the intro at `state.tick`.
pub fn render_intro(buf: &mut Buffer, area: Rect, state: &IntroState, t: &Theme) {
    if area.is_empty() {
        return;
    }
    let (_, cy) = center(area);
    let cy = cy + 1;
    match state.mode {
        Motion::Reduced => {
            fill_canvas(buf, area, t);
            draw_text(buf, area, CAPTION, cy, Tone::Ladder(4), t);
            draw_pill_bottom(buf, area, t);
            draw_hint(buf, area, "Enter", "Continue", t);
            return;
        }
        Motion::Full | Motion::Paused => {}
    }
    let tick = state.tick;
    match IntroPhase::of(tick) {
        IntroPhase::Phrases => {
            fill_canvas(buf, area, t);
            if tick < KNOCK_START {
                if let Some((i, shown)) = phrase_at(tick) {
                    draw_typed(buf, area, PHRASES[i].0, cy, shown, t);
                }
            } else {
                draw_glitched(
                    buf,
                    area,
                    CAPTION,
                    cy,
                    tick - KNOCK_START,
                    Tone::Ladder(4),
                    t,
                );
            }
            draw_pill_bottom(buf, area, t);
            draw_hint(buf, area, "Enter", "Skip", t);
        }
        IntroPhase::Warp => {
            fill_canvas(buf, area, t);
            let f = tick - WARP_START;
            let mut field = Starfield::new(area.width, area.height, 0);
            for step in 0..=f {
                field.advance(true, step);
            }
            field.paint(buf, area, 0, t);
            draw_hint(buf, area, "Enter", "Skip", t);
        }
        IntroPhase::Done => fill_canvas(buf, area, t),
    }
}

/// Render the outro at `state.tick`.
pub fn render_outro(buf: &mut Buffer, area: Rect, state: &OutroState, t: &Theme) {
    if area.is_empty() {
        return;
    }
    let (_, cy) = center(area);
    let cy = cy + 1;
    match state.phase() {
        OutroPhase::Warp => {
            fill_canvas(buf, area, t);
            let f = state.tick;
            let mut field = Starfield::new(area.width, area.height, OUTRO_SALT);
            for step in 0..=f {
                field.advance(false, step);
            }
            let dim = if f >= OUT_WARP - 12 {
                ((f - (OUT_WARP - 12)) / 4) as u8
            } else {
                0
            };
            field.paint(buf, area, dim, t);
            draw_hint(buf, area, "Enter", "Skip", t);
        }
        OutroPhase::Caption => {
            fill_canvas(buf, area, t);
            match (state.mode, state.caption()) {
                (Motion::Reduced, Some(text)) => {
                    draw_text(buf, area, &text, cy, Tone::Ladder(4), t)
                }
                (Motion::Reduced, None) => {}
                (_, Some(text)) => {
                    let j = state.tick - OUT_WARP;
                    draw_glitched(buf, area, &text, cy, j, Tone::Ladder(4), t);
                }
                (_, None) => {}
            }
            draw_pill_bottom(buf, area, t);
        }
        OutroPhase::Done => fill_canvas(buf, area, t),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intro_timeline_follows_original_pacing() {
        assert_eq!(P1_LEN, 64);
        let mut state = IntroState::new(Motion::Full, 10);
        assert_eq!(state.phase(), IntroPhase::Phrases);
        state.skip();
        assert_eq!(state.tick, WARP_START);
        state.skip();
        assert!(state.is_done());
    }

    #[test]
    fn outro_skips_and_caption_wording_stay_stable() {
        let mut state = OutroState::new(Motion::Full, Some(8_040), 5);
        assert_eq!(state.phase(), OutroPhase::Warp);
        state.skip();
        assert_eq!(state.phase(), OutroPhase::Caption);
        state.skip();
        assert!(state.is_done());
        assert_eq!(
            state.caption().as_deref(),
            Some("You were in the Construct for 2 hours 14 minutes")
        );
        assert_eq!(format_universe_duration(45), "45 seconds");
        assert_eq!(format_universe_duration(450), "7 minutes 30 seconds");
    }

    #[test]
    fn starfield_is_deterministic() {
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 80, 24);
        let mut a = Buffer::empty(area);
        let mut b = Buffer::empty(area);
        let mut sa = Starfield::new(area.width, area.height, 7);
        let mut sb = Starfield::new(area.width, area.height, 7);
        sa.advance(true, 42);
        sb.advance(true, 42);
        sa.paint(&mut a, area, 0, &theme);
        sb.paint(&mut b, area, 0, &theme);
        assert_eq!(a, b);
    }
}
