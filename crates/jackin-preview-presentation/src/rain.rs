use termrock::{
    Buffer, Color, Family, FgStep, Modifier, Part, Rect, Role, StateFlags, Style, StylePatch,
    Surface, Theme, Variant,
};

use jackin_preview_domain::scenario::Motion;

pub const TICK_MS: u64 = 33;
pub const MOTION_SEED: u64 = 0x4A41_434B_494E_5E5E;
pub const GLITCH_PASS_TICKS: u64 = 2;
pub const GLITCH_PASSES: u64 = 5;
pub const WARP_TICKS: u64 = 95;

const POOL: &[u8; 78] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz@#$%&*<>{}[]|/\\~";

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

pub const fn pct(value: u64) -> u64 {
    value % 100
}

pub fn glyph(x: u64, y: u64, epoch: u64) -> char {
    POOL.get((mix(x, y, epoch) % 78) as usize)
        .copied()
        .unwrap_or(b' ') as char
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Ladder(u8),
    Accent,
}

const RAIN_FAMILY: Family = Family::custom("jackin-preview.rain");
const RAIN_PART: Part = Part::custom("jackin-preview.rain.style");

fn role_style(theme: &Theme, patch: StylePatch) -> Style {
    theme
        .clone()
        .define_family(RAIN_FAMILY, |family| {
            family.part(RAIN_PART).base(patch);
        })
        .resolve(
            RAIN_FAMILY,
            Variant::DEFAULT,
            RAIN_PART,
            StateFlags::empty(),
            Surface::Canvas,
        )
        .style
        .into_style()
}

fn canvas(theme: &Theme) -> Color {
    theme.bg(Surface::Canvas)
}

fn ladder_color(theme: &Theme, level: u8) -> Color {
    let step = level.min(4);
    let index = 4usize.saturating_sub(usize::from(step));
    let resolved = theme.color.fg.get(index).copied().unwrap_or(Color::Reset);
    // Tag junie `text_muted` / `text_faint` are not the current FgStep ramp.
    match step {
        2 if resolved == Color::from_u32(0x80_80_80) => Color::from_u32(0x8a_8a_8a),
        1 if resolved == Color::from_u32(0x4d_4d_4d) => Color::from_u32(0x73_73_73),
        _ => resolved,
    }
}

pub fn style(theme: &Theme, tone: Tone, dim: u8) -> Option<Style> {
    let bg = canvas(theme);
    match tone {
        Tone::Ladder(level) => {
            if dim > level {
                None
            } else {
                Some(Style::new().fg(ladder_color(theme, level - dim)).bg(bg))
            }
        }
        Tone::Accent => match dim {
            0 => Some(Style::new().fg(theme.color.accent).bg(bg)),
            1 => Some(Style::new().fg(theme.color.accent_hover).bg(bg)),
            2 => Some(Style::new().fg(theme.color.accent_pressed).bg(bg)),
            _ => None,
        },
    }
}

pub(crate) fn put(buf: &mut Buffer, x: u16, y: u16, ch: char, style: Style) {
    if let Some(cell) = buf.cell_mut(termrock::Position::new(x, y)) {
        let mut bytes = [0u8; 4];
        cell.set_symbol(ch.encode_utf8(&mut bytes));
        cell.set_style(style);
    }
}

pub fn fill_canvas(buf: &mut Buffer, area: Rect, theme: &Theme) {
    buf.set_style(
        area,
        role_style(
            theme,
            StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(Surface::Canvas)),
        ),
    );
}

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

fn edge_radius(angle: f32, cx: f32, cy: f32) -> f32 {
    let dx = (angle.cos() * 2.0).abs();
    let dy = angle.sin().abs();
    let rx = if dx > 1e-3 { cx / dx } else { f32::MAX };
    let ry = if dy > 1e-3 { cy / dy } else { f32::MAX };
    rx.min(ry).max(1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarpCell {
    pub ch: char,
    pub level: u8,
    pub accent: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Starfield {
    seed: u64,
    stars: Vec<Star>,
    cols: u16,
    rows: u16,
    cells: Vec<Option<WarpCell>>,
    pub frame: u64,
}

impl Starfield {
    pub fn new(cols: u16, rows: u16, salt: u64) -> Self {
        let rows = rows.max(1);
        let mut seed: u64 = 0x9E37_79B9_7F4A_7C15 ^ salt;
        let (cx, cy) = (f32::from(cols) / 2.0, f32::from(rows) / 2.0);
        let count = (usize::from(cols).saturating_mul(usize::from(rows)) / 4).clamp(80, 2400);
        let mut stars = Vec::with_capacity(count);
        for _ in 0..count {
            let angle =
                (xorshift(&mut seed) % 36_000) as f32 / 36_000.0 * 2.0 * core::f32::consts::PI;
            stars.push(Star {
                angle,
                radius: (xorshift(&mut seed) % 1_000) as f32 / 1_000.0 * edge_radius(angle, cx, cy),
                speed: 0.5 + (xorshift(&mut seed) % 100) as f32 / 100.0,
            });
        }
        Self {
            seed,
            stars,
            cols,
            rows,
            cells: vec![None; usize::from(cols).saturating_mul(usize::from(rows))],
            frame: 0,
        }
    }

    pub fn advance(&mut self, accelerating: bool, frame: u64) {
        let cols = usize::from(self.cols);
        let rows = usize::from(self.rows);
        self.cells.fill(None);
        if cols == 0 || rows == 0 {
            self.frame = frame.saturating_add(1);
            return;
        }
        let cx = cols as f32 / 2.0;
        let cy = rows as f32 / 2.0;
        let max_r = (cx / 2.0).hypot(cy).max(1.0);
        let t = frame as f32 / WARP_TICKS as f32;
        let warp_factor = if accelerating {
            0.2 + t * t * 5.0
        } else {
            0.2 + (1.0 - t).powi(2) * 5.0
        };
        let entry_fade = (frame as f32 / 8.0).min(1.0);
        for index in 0..self.stars.len() {
            let Some(mut star) = self.stars.get(index).copied() else {
                continue;
            };
            let prev = star.radius;
            star.radius += star.speed * warp_factor;
            let dx = star.angle.cos() * 2.0;
            let dy = star.angle.sin();
            let head_x = cx + dx * star.radius;
            let head_y = cy + dy * star.radius;
            if head_x < 0.0 || head_x >= cols as f32 || head_y < 0.0 || head_y >= rows as f32 {
                star.angle = (xorshift(&mut self.seed) % 36_000) as f32 / 36_000.0
                    * 2.0
                    * core::f32::consts::PI;
                star.radius = (xorshift(&mut self.seed) % 60) as f32 / 100.0;
                star.speed = 0.5 + (xorshift(&mut self.seed) % 100) as f32 / 100.0;
                if let Some(slot) = self.stars.get_mut(index) {
                    *slot = star;
                }
                continue;
            }
            let steps = ((1.0 + warp_factor * 1.4) as usize).max(1);
            for step in 0..=steps {
                let rr = prev + (star.radius - prev) * (step as f32 / steps as f32);
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
                let cell_index = yu.saturating_mul(cols).saturating_add(xu);
                if let Some(cell) = self.cells.get_mut(cell_index) {
                    *cell = Some(WarpCell {
                        ch,
                        level,
                        accent: streak && step == steps && bright > 0.7,
                    });
                }
            }
            if let Some(slot) = self.stars.get_mut(index) {
                *slot = star;
            }
        }
        self.frame = frame.saturating_add(1);
    }

    pub const fn size(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    pub fn paint(&self, buf: &mut Buffer, area: Rect, dim: u8, theme: &Theme) {
        let cols = self.cols.min(area.width);
        let rows = self.rows.min(area.height);
        for y in 0..rows {
            for x in 0..cols {
                let cell_index = usize::from(y)
                    .saturating_mul(usize::from(self.cols))
                    .saturating_add(usize::from(x));
                let Some(cell) = self.cells.get(cell_index).copied().flatten() else {
                    continue;
                };
                if cell.level == 0 && !cell.accent {
                    continue;
                }
                let tone = if cell.accent {
                    Tone::Accent
                } else {
                    Tone::Ladder(cell.level)
                };
                if let Some(resolved) = style(theme, tone, dim) {
                    buf.set_string(
                        area.x.saturating_add(x),
                        area.y.saturating_add(y),
                        cell.ch.to_string(),
                        resolved,
                    );
                }
            }
        }
    }
}

const fn phrase_ticks(chars: u64, char_ms: u64, hold_ms: u64) -> u64 {
    (chars * char_ms + hold_ms).div_ceil(TICK_MS)
}

pub const PHRASES: [(&str, u64, u64); 3] = [
    ("Stand up, operator…", 60, 950),
    ("Host stays outside…", 55, 950),
    ("Follow the green.", 50, 850),
];
pub const CAPTION: &str = "Knock, knock, operator.";
pub const CAPTION_HOLD_MS: u64 = 850;
pub const P1_LEN: u64 = phrase_ticks(19, 60, 950);
pub const P2_LEN: u64 = phrase_ticks(19, 55, 950);
pub const P3_LEN: u64 = phrase_ticks(17, 50, 850);
pub const KNOCK_START: u64 = P1_LEN + P2_LEN + P3_LEN;
pub const KNOCK_LEN: u64 = GLITCH_PASSES * GLITCH_PASS_TICKS + CAPTION_HOLD_MS.div_ceil(TICK_MS);
pub const WARP_START: u64 = KNOCK_START + KNOCK_LEN;
pub const INTRO_END: u64 = WARP_START + WARP_TICKS;
pub const REDUCED_HOLD: u64 = 45;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntroPhase {
    Phrases,
    Warp,
    Done,
}

impl IntroPhase {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntroState {
    pub tick: u64,
    pub mode: Motion,
}

impl IntroState {
    pub const fn new(mode: Motion, frame: u64) -> Self {
        Self { tick: frame, mode }
    }

    pub const fn phase(&self) -> IntroPhase {
        match self.mode {
            Motion::Reduced if self.tick < REDUCED_HOLD => IntroPhase::Phrases,
            Motion::Reduced => IntroPhase::Done,
            _ => IntroPhase::of(self.tick),
        }
    }

    pub const fn is_done(&self) -> bool {
        matches!(self.phase(), IntroPhase::Done)
    }

    pub fn advance_tick(&mut self) -> bool {
        if self.mode == Motion::Paused || self.is_done() {
            return false;
        }
        self.tick = self.tick.saturating_add(1);
        true
    }

    pub fn skip(&mut self) {
        self.tick = match self.mode {
            Motion::Paused => INTRO_END,
            Motion::Reduced => REDUCED_HOLD,
            _ => match self.phase() {
                IntroPhase::Phrases => WARP_START,
                IntroPhase::Warp => INTRO_END,
                IntroPhase::Done => INTRO_END,
            },
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutroPhase {
    Warp,
    Caption,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutroState {
    pub tick: u64,
    pub elapsed_secs: Option<u64>,
    pub mode: Motion,
}

pub const OUT_WARP: u64 = WARP_TICKS;
pub const OUT_CAPTION: u64 = GLITCH_PASSES * GLITCH_PASS_TICKS + 2_400u64.div_ceil(TICK_MS);

impl OutroState {
    pub const fn new(mode: Motion, elapsed_secs: Option<u64>, frame: u64) -> Self {
        Self {
            tick: frame,
            elapsed_secs,
            mode,
        }
    }

    pub const fn end(&self) -> u64 {
        match self.mode {
            Motion::Reduced => REDUCED_HOLD,
            _ if self.elapsed_secs.is_some() => OUT_WARP + OUT_CAPTION,
            _ => OUT_WARP,
        }
    }

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

    pub const fn is_done(&self) -> bool {
        matches!(self.phase(), OutroPhase::Done)
    }

    pub fn advance_tick(&mut self) -> bool {
        if self.mode == Motion::Paused || self.is_done() {
            return false;
        }
        self.tick = self.tick.saturating_add(1);
        true
    }

    pub fn skip(&mut self) {
        self.tick = match self.phase() {
            OutroPhase::Warp => OUT_WARP,
            OutroPhase::Caption | OutroPhase::Done => self.end(),
        };
    }

    pub fn caption(&self) -> Option<String> {
        self.elapsed_secs.map(|secs| {
            format!(
                "You were in the Construct for {}",
                format_universe_duration(secs)
            )
        })
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandoffStage {
    CockpitDim(u8),
    Canvas,
    CapsuleDim(u8),
    Capsule,
}

pub const fn handoff_stage(frame: u64) -> HandoffStage {
    match frame {
        0..=3 => HandoffStage::CockpitDim((frame + 1) as u8),
        4..=5 => HandoffStage::Canvas,
        6..=10 => HandoffStage::CapsuleDim((10 - frame) as u8),
        _ => HandoffStage::Capsule,
    }
}

pub const HANDOFF_LEN: u64 = 12;

fn center(area: Rect) -> (u16, u16) {
    (
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2).saturating_sub(1),
    )
}

fn draw_text(buf: &mut Buffer, area: Rect, text: &str, y: u16, tone: Tone, theme: &Theme) {
    let n = text.chars().count() as u16;
    let x0 = area.x.saturating_add(area.width.saturating_sub(n) / 2);
    if let Some(resolved) = style(theme, tone, 0) {
        for (index, ch) in text.chars().enumerate() {
            put(buf, x0.saturating_add(index as u16), y, ch, resolved);
        }
    }
}

fn hint_ink(theme: &Theme, level: u8) -> Color {
    ladder_color(theme, level)
}

fn draw_hint(buf: &mut Buffer, area: Rect, key: &str, action: &str, theme: &Theme) {
    let text_len = key
        .chars()
        .count()
        .saturating_add(1)
        .saturating_add(action.chars().count()) as u16;
    if area.width < text_len.saturating_add(4) {
        return;
    }
    let x = area.right().saturating_sub(text_len.saturating_add(2));
    let y = area.bottom().saturating_sub(1);
    let clear = Style::new().bg(canvas(theme));
    let start = x.saturating_sub(1);
    for column in start..area.right() {
        put(buf, column, y, ' ', clear);
    }
    let key_style = Style::new()
        .fg(hint_ink(theme, 2))
        .bg(canvas(theme))
        .add_modifier(Modifier::BOLD);
    let action_style = Style::new().fg(hint_ink(theme, 1)).bg(canvas(theme));
    for (index, ch) in key.chars().enumerate() {
        put(buf, x.saturating_add(index as u16), y, ch, key_style);
    }
    let action_x = x
        .saturating_add(key.chars().count() as u16)
        .saturating_add(1);
    for (index, ch) in action.chars().enumerate() {
        put(
            buf,
            action_x.saturating_add(index as u16),
            y,
            ch,
            action_style,
        );
    }
}

fn draw_lockup(buf: &mut Buffer, area: Rect, text: &str, theme: &Theme) {
    let label_width = text.chars().count() as u16;
    let width = label_width.saturating_add(2);
    if area.height < 4 || area.width < width.saturating_add(2) {
        return;
    }
    let x = area.x.saturating_add(area.width.saturating_sub(width) / 2);
    let y = area.bottom().saturating_sub(2);
    let style = Style::new()
        .fg(theme.color.on_accent)
        .bg(theme.color.accent)
        .add_modifier(Modifier::BOLD);
    put(buf, x, y, ' ', style);
    for (index, ch) in text.chars().enumerate() {
        put(
            buf,
            x.saturating_add(1).saturating_add(index as u16),
            y,
            ch,
            style,
        );
    }
    put(
        buf,
        x.saturating_add(width).saturating_sub(1),
        y,
        ' ',
        style,
    );
}

fn draw_typed(buf: &mut Buffer, area: Rect, text: &str, y: u16, shown: usize, theme: &Theme) {
    let n = text.chars().count() as u16;
    let x0 = area.x.saturating_add(area.width.saturating_sub(n) / 2);
    if let Some(resolved) = style(theme, Tone::Ladder(4), 0) {
        for (index, ch) in text.chars().take(shown).enumerate() {
            put(buf, x0.saturating_add(index as u16), y, ch, resolved);
        }
    }
}

fn draw_glitched(
    buf: &mut Buffer,
    area: Rect,
    text: &str,
    y: u16,
    tick: u64,
    tone: Tone,
    theme: &Theme,
) {
    let n = text.chars().count() as u16;
    let x0 = area.x.saturating_add(area.width.saturating_sub(n) / 2);
    let Some(resolved) = style(theme, tone, 0) else {
        return;
    };
    let pass = tick / GLITCH_PASS_TICKS;
    for (index, ch) in text.chars().enumerate() {
        let x = x0.saturating_add(index as u16);
        let shown =
            if pass < GLITCH_PASSES && mix(u64::from(x), u64::from(y), pass).is_multiple_of(3) {
                glyph(u64::from(x), u64::from(y), pass)
            } else {
                ch
            };
        put(buf, x, y, shown, resolved);
    }
}

fn phrase_at(tick: u64) -> Option<(usize, usize)> {
    let mut start = 0u64;
    for (index, (text, char_ms, hold_ms)) in PHRASES.iter().enumerate() {
        let chars = text.chars().count() as u64;
        let len = phrase_ticks(chars, *char_ms, *hold_ms);
        if tick < start.saturating_add(len) {
            let k = tick.saturating_sub(start);
            let shown = ((k.saturating_mul(TICK_MS)) / char_ms).min(chars) as usize;
            return Some((index, shown));
        }
        start = start.saturating_add(len);
    }
    None
}

fn sync_field(
    field: &mut Option<Starfield>,
    area: Rect,
    salt: u64,
    accelerating: bool,
    frame: u64,
) {
    let fresh = !matches!(field, Some(existing) if existing.size() == (area.width, area.height));
    if fresh {
        *field = Some(Starfield::new(area.width, area.height, salt));
    }
    if let Some(existing) = field.as_mut() {
        if existing.frame > frame.saturating_add(1) {
            *existing = Starfield::new(area.width, area.height, salt);
        }
        while existing.frame <= frame {
            let step = existing.frame;
            existing.advance(accelerating, step);
        }
    }
}

const OUTRO_SALT: u64 = 0x5F5F_4F55_5452_4F5F;
const BRAND_MARK: &str = "jackin❯";

pub fn paint_intro(buf: &mut Buffer, area: Rect, state: &IntroState, theme: &Theme) {
    if area.is_empty() {
        return;
    }
    let (_, cy) = center(area);
    let cy = cy.saturating_add(1);
    match state.mode {
        Motion::Reduced => {
            fill_canvas(buf, area, theme);
            draw_text(buf, area, CAPTION, cy, Tone::Ladder(4), theme);
            draw_lockup(buf, area, BRAND_MARK, theme);
            draw_hint(buf, area, "Enter", "Continue", theme);
            return;
        }
        Motion::Full | Motion::Paused => {}
    }
    match IntroPhase::of(state.tick) {
        IntroPhase::Phrases => {
            fill_canvas(buf, area, theme);
            if state.tick < KNOCK_START {
                if let Some((index, shown)) = phrase_at(state.tick)
                    && let Some((text, _, _)) = PHRASES.get(index)
                {
                    draw_typed(buf, area, text, cy, shown, theme);
                }
            } else {
                draw_glitched(
                    buf,
                    area,
                    CAPTION,
                    cy,
                    state.tick.saturating_sub(KNOCK_START),
                    Tone::Ladder(4),
                    theme,
                );
            }
            draw_lockup(buf, area, BRAND_MARK, theme);
            draw_hint(buf, area, "Enter", "Skip", theme);
        }
        IntroPhase::Warp => {
            fill_canvas(buf, area, theme);
            let mut field = None;
            sync_field(
                &mut field,
                area,
                0,
                true,
                state.tick.saturating_sub(WARP_START),
            );
            if let Some(field) = &field {
                field.paint(buf, area, 0, theme);
            }
            draw_hint(buf, area, "Enter", "Skip", theme);
        }
        IntroPhase::Done => fill_canvas(buf, area, theme),
    }
}

pub fn paint_outro(buf: &mut Buffer, area: Rect, state: &OutroState, theme: &Theme) {
    if area.is_empty() {
        return;
    }
    let (_, cy) = center(area);
    let cy = cy.saturating_add(1);
    match state.phase() {
        OutroPhase::Warp => {
            fill_canvas(buf, area, theme);
            let mut field = None;
            sync_field(&mut field, area, OUTRO_SALT, false, state.tick);
            if let Some(field) = &field {
                let dim = if state.tick >= OUT_WARP.saturating_sub(12) {
                    ((state.tick.saturating_sub(OUT_WARP.saturating_sub(12))) / 4) as u8
                } else {
                    0
                };
                field.paint(buf, area, dim, theme);
            }
            draw_hint(buf, area, "Enter", "Skip", theme);
        }
        OutroPhase::Caption => {
            fill_canvas(buf, area, theme);
            match (state.mode, state.caption()) {
                (Motion::Reduced, Some(text)) => {
                    draw_text(buf, area, &text, cy, Tone::Ladder(4), theme)
                }
                (Motion::Reduced, None) => {}
                (_, Some(text)) => draw_glitched(
                    buf,
                    area,
                    &text,
                    cy,
                    state.tick.saturating_sub(OUT_WARP),
                    Tone::Ladder(4),
                    theme,
                ),
                (_, None) => {}
            }
            draw_lockup(buf, area, BRAND_MARK, theme);
            draw_hint(buf, area, "Enter", "Close", theme);
        }
        OutroPhase::Done => fill_canvas(buf, area, theme),
    }
}

pub fn paint_atmosphere(
    buf: &mut Buffer,
    area: Rect,
    exclude: &[Rect],
    frame: u64,
    running: bool,
    frozen: bool,
    theme: &Theme,
) {
    for x in area.left()..area.right() {
        if pct(mix(u64::from(x), 11, 0)) >= 18 {
            continue;
        }
        let mixed = mix(u64::from(x), 12, 0);
        let period_t = 2 + mixed % 2;
        let trail = 6 + (mixed >> 8) % 5;
        let gap = 6 + (mixed >> 16) % 19;
        let period = u64::from(area.height)
            .saturating_add(trail)
            .saturating_add(gap);
        if period == 0 {
            continue;
        }
        let phase = (mixed >> 24) % period;
        let signal = (mixed >> 40).is_multiple_of(10);
        let head = (frame / period_t + phase) % period;
        let head_y = head as i64 - gap as i64;
        for y in area.top()..area.bottom() {
            if exclude.iter().any(|rect| rect.contains((x, y).into())) {
                continue;
            }
            let age = head_y - i64::from(y.saturating_sub(area.y));
            if !(0..=3).contains(&age) {
                continue;
            }
            let tone = if age == 0 {
                if signal && running && !frozen && frame >= 15 {
                    Tone::Accent
                } else {
                    Tone::Ladder(1)
                }
            } else {
                Tone::Ladder(0)
            };
            let tone = if frame < 15 { Tone::Ladder(0) } else { tone };
            if let Some(resolved) = style(theme, tone, 0) {
                put(
                    buf,
                    x,
                    y,
                    glyph(u64::from(x), u64::from(y), frame >> 3),
                    resolved,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_canvas_paints_primary_foreground_on_the_canvas() {
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 2, 1);
        let mut buf = Buffer::empty(area);
        fill_canvas(&mut buf, area, &theme);
        let cell = buf.cell((0, 0)).expect("filled cell");
        assert_eq!(cell.fg, theme.color.fg[FgStep::Primary.index()]);
        assert_eq!(cell.bg, theme.bg(Surface::Canvas));
    }

    #[test]
    fn paused_intro_paints_the_padded_accent_lockup_and_tag_hint_ink() {
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 120, 40);
        let mut buf = Buffer::empty(area);
        paint_intro(&mut buf, area, &IntroState::new(Motion::Paused, 40), &theme);
        let pad = buf.cell((55, 38)).expect("lockup pad");
        let mark = buf.cell((56, 38)).expect("lockup mark");
        assert_eq!(pad.symbol(), " ");
        assert_eq!(pad.bg, theme.color.accent);
        assert_eq!(mark.symbol(), "j");
        assert_eq!(mark.fg, theme.color.on_accent);
        assert_eq!(mark.bg, theme.color.accent);
        assert!(mark.modifier.contains(Modifier::BOLD));
        let key = buf.cell((108, 39)).expect("hint key");
        let action = buf.cell((114, 39)).expect("hint action");
        assert_eq!(key.symbol(), "E");
        assert_eq!(key.fg, Color::from_u32(0x8a_8a_8a));
        assert!(key.modifier.contains(Modifier::BOLD));
        assert_eq!(action.symbol(), "S");
        assert_eq!(action.fg, Color::from_u32(0x73_73_73));
        assert!(!action.modifier.contains(Modifier::BOLD));
    }

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
    fn paused_outro_warp_uses_tag_muted_and_faint_ink() {
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 120, 40);
        let mut buf = Buffer::empty(area);
        paint_outro(
            &mut buf,
            area,
            &OutroState::new(Motion::Paused, Some(8_040), 40),
            &theme,
        );
        let mut muted = 0usize;
        let mut faint = 0usize;
        let mut legacy = 0usize;
        for y in 0..area.height {
            for x in 0..area.width {
                let cell = buf.cell((x, y)).expect("cell");
                if cell.fg == Color::from_u32(0x8a_8a_8a) {
                    muted += 1;
                } else if cell.fg == Color::from_u32(0x73_73_73) {
                    faint += 1;
                } else if cell.fg == Color::from_u32(0x80_80_80)
                    || cell.fg == Color::from_u32(0x4d_4d_4d)
                {
                    legacy += 1;
                }
            }
        }
        assert!(muted > 0, "tag muted ink missing");
        assert!(faint > 0, "tag faint ink missing");
        assert_eq!(legacy, 0, "current ramp leaked into the warp");
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
