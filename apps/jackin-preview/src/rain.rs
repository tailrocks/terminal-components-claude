//! Deterministic Jackin atmosphere and entry/exit timing.
//!
//! The renderer owns no product state.  It consumes semantic `Role`s through
//! the public facade when the shell paints it, while this module keeps the
//! exact virtual-frame contracts used by the preview and its tests.

use junie_tui::{
    Buffer, Color, Family, FgStep, Part, Rect, Role, StateFlags, Style, StylePatch, Surface, Theme,
    Variant,
};

use crate::scenario::Motion;

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

const POOL: &[u8] = b" .,:;+=*#%@";

/// Stable integer mixer used for atmosphere placement.
pub const fn mix(mut a: u64, b: u64, c: u64) -> u64 {
    a ^= b.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    a = a.rotate_left(17).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    a ^= c.wrapping_mul(0x94D0_49BB_1331_11EB);
    a ^ (a >> 31)
}

/// Map a mixed value to a percentage without floating point.
pub const fn pct(value: u64) -> u64 {
    value % 100
}

/// Stable atmosphere glyph.
pub fn glyph(x: u64, y: u64, epoch: u64) -> char {
    POOL.get((mix(x, y, epoch) as usize) % POOL.len())
        .copied()
        .unwrap_or(b' ') as char
}

/// Semantic tone accepted by the compatibility painting helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// One of the five foreground ladder levels, ghost through primary.
    Ladder(u8),
    /// Accent trace.
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
}

/// Resolve a tone to a theme style.  Dim is index arithmetic over the
/// foreground ladder; no colour equality is used, so themes with distinct
/// accent/focus/success colours remain distinct.
pub fn style(theme: &Theme, tone: Tone, dim: u8) -> Option<Style> {
    let role = match tone {
        Tone::Ladder(level) => {
            let index = usize::from(level.saturating_sub(dim).min(4));
            Role::Fg(match index {
                0 => FgStep::Primary,
                1 => FgStep::Secondary,
                2 => FgStep::Muted,
                3 => FgStep::Faint,
                _ => FgStep::Ghost,
            })
        }
        Tone::Accent if dim >= 3 => return None,
        Tone::Accent if dim == 2 => Role::Fg(FgStep::Muted),
        Tone::Accent if dim == 1 => Role::Fg(FgStep::Faint),
        Tone::Accent => Role::Accent,
    };
    Some(role_style(theme, StylePatch::new().set_fg(role)))
}

/// Fill a field with the theme canvas colour.
pub fn fill_canvas(buf: &mut Buffer, area: Rect, theme: &Theme) {
    buf.set_style(
        area,
        role_style(
            theme,
            StylePatch::new().set_bg(Role::Surface(Surface::Canvas)),
        ),
    );
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

/// A bounded, reproducible star/warp field.  The implementation is integer
/// only: it is cheap enough for captures and byte-identical across machines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Starfield {
    seed: u64,
    cols: u16,
    rows: u16,
    cells: Vec<Option<WarpCell>>,
    /// Number of frames painted.
    pub frame: u64,
}

impl Starfield {
    /// Build a seeded field for `cols × rows`.
    pub fn new(cols: u16, rows: u16, salt: u64) -> Self {
        Self {
            seed: mix(MOTION_SEED ^ salt, u64::from(cols), u64::from(rows)),
            cols,
            rows,
            cells: vec![None; usize::from(cols) * usize::from(rows)],
            frame: 0,
        }
    }

    /// Advance and regenerate one field frame.
    pub fn advance(&mut self, accelerating: bool, frame: u64) {
        self.cells.fill(None);
        let cols = usize::from(self.cols);
        let rows = usize::from(self.rows);
        if cols == 0 || rows == 0 {
            self.frame = frame.saturating_add(1);
            return;
        }
        let count = (cols.saturating_mul(rows) / 8).clamp(8, 512);
        for index in 0..count {
            let seed = mix(self.seed, index as u64, frame);
            let x = (seed as usize) % cols;
            let y = ((seed >> 16) as usize) % rows;
            let level = (((seed >> 32) % 5) as u8).saturating_add(if accelerating { 0 } else { 1 });
            let level = level.min(4);
            let accent = accelerating && pct(seed >> 40) < 8;
            let cell_index = y.saturating_mul(cols).saturating_add(x);
            if let Some(cell) = self.cells.get_mut(cell_index) {
                *cell = Some(WarpCell {
                    ch: if accent {
                        '─'
                    } else {
                        glyph(x as u64, y as u64, frame)
                    },
                    level,
                    accent,
                });
            }
        }
        self.frame = frame.saturating_add(1);
        self.seed = mix(self.seed, frame, u64::from(accelerating));
    }

    /// Dimensions of the field.
    pub const fn size(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    /// Paint the last generated frame.
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

/// Paint deterministic atmosphere cells outside excluded rectangles.
pub fn paint_atmosphere(
    buf: &mut Buffer,
    area: Rect,
    exclude: &[Rect],
    frame: u64,
    running: bool,
    frozen: bool,
    theme: &Theme,
) {
    for column in area.columns() {
        let x = column.x;
        if pct(mix(u64::from(x), 11, 0)) >= 18 {
            continue;
        }
        let head = (frame + (mix(u64::from(x), 12, 0) % u64::from(area.height.max(1))))
            % u64::from(area.height.max(1));
        for row in area.rows() {
            let y = row.y;
            if exclude.iter().any(|rect| rect.contains((x, y).into())) {
                continue;
            }
            let distance = (u64::from(y.saturating_sub(area.y)) + u64::from(area.height) - head)
                % u64::from(area.height.max(1));
            if distance > 2 {
                continue;
            }
            let tone = if distance == 0 && running && !frozen && frame >= 15 {
                Tone::Accent
            } else {
                Tone::Ladder(1u8.saturating_sub(distance as u8))
            };
            if let Some(resolved) = style(theme, tone, 0) {
                buf.set_string(
                    x,
                    y,
                    glyph(u64::from(x), u64::from(y), frame >> 3).to_string(),
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
