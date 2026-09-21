//! Production-view session: `App::for_scenario` / `for_scenario_at` plus runtime draw.

use std::time::Duration;

use jackin_app::{App, Motion, Route, Scenario};
use junie_tui::{Axis, ColorLevel, Id, KeyCode, KeyModifiers, MouseKind, Rect, Theme};
use junie_tui_testing::Harness;

use crate::frame::{ObservedCell, ObservedCursor, ObservedFrame};

/// Atmosphere seed owned by the production rain renderer.
pub const MOTION_SEED: u64 = jackin_app::rain::MOTION_SEED;
/// Fixture epoch used by the production clock (`2026-09-03 09:14:00` UTC+7).
pub const EPOCH_SECS: i64 = 1_788_401_640;
/// Helper tick used by `apps/jackin-preview/tests/support.rs` `H::ticks`.
pub const HELPER_TICK_MS: u64 = 200;
/// Viewport at which `App::draw` currently overpaints selected routes.
///
/// The adapter still always calls production `App::draw`; it never invokes the
/// historical painter itself and never treats that overpaint as an expected
/// artifact.
pub const HISTORICAL_PAINT_SIZE: (u16, u16) = (120, 40);

/// Terminal size for a capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    /// Columns.
    pub width: u16,
    /// Rows.
    pub height: u16,
}

impl Viewport {
    /// Named size.
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Whether this size currently enables app-local historical overpaint.
    #[must_use]
    pub const fn is_historical_paint_size(self) -> bool {
        self.width == HISTORICAL_PAINT_SIZE.0 && self.height == HISTORICAL_PAINT_SIZE.1
    }
}

/// Color identity for a capture. `None` and `NoColor` both resolve to mono
/// through the public harness; they remain distinct identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureColor {
    /// 24-bit colour.
    TrueColor,
    /// 256-colour palette.
    Ansi256,
    /// 16 ANSI colours.
    Ansi16,
    /// Explicit `--color none` / mono.
    None,
    /// `NO_COLOR` without an explicit color flag.
    NoColor,
}

impl CaptureColor {
    /// Production theme downgrade used for this identity.
    #[must_use]
    pub const fn level(self) -> ColorLevel {
        match self {
            Self::TrueColor => ColorLevel::TrueColor,
            Self::Ansi256 => ColorLevel::Ansi256,
            Self::Ansi16 => ColorLevel::Ansi16,
            Self::None | Self::NoColor => ColorLevel::Mono,
        }
    }

    /// Stable identity label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TrueColor => "truecolor",
            Self::Ansi256 => "256",
            Self::Ansi16 => "16",
            Self::None => "none",
            Self::NoColor => "nocolor",
        }
    }

    /// JA-001 remaining colour identities after the truecolor first leaf.
    #[must_use]
    pub const fn ja001_remaining() -> [Self; 4] {
        [Self::Ansi256, Self::Ansi16, Self::None, Self::NoColor]
    }

    /// Full colour matrix used by JA-001/JA-067.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::TrueColor,
            Self::Ansi256,
            Self::Ansi16,
            Self::None,
            Self::NoColor,
        ]
    }
}

/// Direct production-view session over the public runtime harness.
#[derive(Debug)]
pub struct DirectSession {
    program: &'static str,
    scenario: Scenario,
    motion: Motion,
    construct_frame: u64,
    color: CaptureColor,
    harness: Harness<App>,
}

impl DirectSession {
    /// Construct `App::for_scenario(scenario, Paused)` and draw at 80×24 truecolor.
    #[must_use]
    pub fn ja001_first_leaf(scenario: Scenario) -> Self {
        Self::paused_frame0(
            scenario,
            Viewport::new(
                crate::JA001_FIRST_LEAF_WIDTH,
                crate::JA001_FIRST_LEAF_HEIGHT,
            ),
            CaptureColor::TrueColor,
        )
    }

    /// `fresh(world, Paused, 0); draw` through production `App::for_scenario`.
    #[must_use]
    pub fn paused_frame0(scenario: Scenario, viewport: Viewport, color: CaptureColor) -> Self {
        Self::fresh(
            crate::JA001_ID,
            scenario,
            Motion::Paused,
            0,
            viewport,
            color,
        )
    }

    /// Fresh world at an explicit motion/frame, then production draw.
    ///
    /// Frame 0 uses [`App::for_scenario`]; nonzero frames use
    /// [`App::for_scenario_at`].
    #[must_use]
    pub fn fresh(
        program: &'static str,
        scenario: Scenario,
        motion: Motion,
        frame: u64,
        viewport: Viewport,
        color: CaptureColor,
    ) -> Self {
        let app = if frame == 0 {
            App::for_scenario(scenario, motion)
        } else {
            App::for_scenario_at(scenario, motion, frame)
        };
        Self::from_app(program, app, scenario, motion, frame, viewport, color)
    }

    fn from_app(
        program: &'static str,
        app: App,
        scenario: Scenario,
        motion: Motion,
        construct_frame: u64,
        viewport: Viewport,
        color: CaptureColor,
    ) -> Self {
        let mut harness = Harness::new(app, Theme::junie(), viewport.width, viewport.height)
            .with_color(color.level());
        // Harness::new already presents once; an extra production draw is the
        // explicit `draw` in `fresh(...); draw` and is state-neutral when paused.
        harness.draw();
        Self {
            program,
            scenario,
            motion,
            construct_frame,
            color,
            harness,
        }
    }

    /// Advance `n` helper ticks with a production draw after each, matching `H::ticks`.
    pub fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            let _ = self.harness.advance(Duration::from_millis(HELPER_TICK_MS));
        }
    }

    /// Advance the production clock by an exact millisecond count, then draw.
    pub fn advance_ms(&mut self, ms: u64) {
        let _ = self.harness.advance(Duration::from_millis(ms));
    }

    /// One explicit production draw without dispatching input.
    pub fn draw(&mut self) {
        self.harness.draw();
    }

    /// Mutable production app projection for fixture-only state setup.
    ///
    /// Used only to install pinned oracle fixtures (1Password session state,
    /// arbiter claims, daemon attach) before the first captured event, never
    /// to mutate mid-sequence control flow.
    pub fn app_mut(&mut self) -> &mut App {
        self.harness.app_mut()
    }

    /// Area the resolver gave layer `id` in the last draw, if open.
    #[must_use]
    pub fn layer_area(&self, id: Id) -> Option<Rect> {
        self.harness.layer_area(id)
    }

    /// Control currently under the production hover, if any.
    #[must_use]
    pub fn hover(&self) -> Option<Id> {
        self.harness.hover()
    }

    /// Interaction style flags resolved for `id` in the last draw.
    #[must_use]
    pub fn state_of(&self, id: Id) -> junie_tui::StateFlags {
        self.harness.state_of(id)
    }

    /// Secondary (right) click at a production coordinate.
    pub fn secondary(&mut self, x: u16, y: u16) {
        let _ = self.harness.secondary(x, y);
    }

    /// First row containing `needle`, if painted.
    #[must_use]
    pub fn find_row(&self, needle: &str) -> Option<u16> {
        self.harness.find_row(needle)
    }

    /// Occurrences of `needle` across the last production frame.
    #[must_use]
    pub fn count(&self, needle: &str) -> usize {
        self.harness.count(needle)
    }

    /// Send one key through the production runtime.
    pub fn key(&mut self, code: KeyCode) {
        let _ = self.harness.key(code);
    }

    /// Send one key with modifiers through the production runtime.
    pub fn key_mod(&mut self, code: KeyCode, mods: KeyModifiers) {
        let _ = self.harness.key_mod(code, mods);
    }

    /// Send a control chord, matching `H::ctrl`.
    pub fn ctrl(&mut self, c: char) {
        let _ = self.harness.ctrl(c);
    }

    /// Type Unicode characters through the production runtime.
    pub fn type_str(&mut self, s: &str) {
        let _ = self.harness.type_str(s);
    }

    /// Click at a production coordinate.
    pub fn click(&mut self, x: u16, y: u16) {
        let _ = self.harness.click(x, y);
    }

    /// Click the centre of a registered control.
    pub fn click_id(&mut self, id: Id) {
        let _ = self.harness.click_id(id);
    }

    /// Whether a layer id is open.
    #[must_use]
    pub fn is_open(&self, id: Id) -> bool {
        self.harness.is_open(id)
    }

    /// Tab until `id` owns focus, matching `H::tab_to`.
    pub fn tab_to(&mut self, id: Id) -> bool {
        self.harness.tab_to(id)
    }

    /// Bracketed paste through the production runtime.
    pub fn paste(&mut self, s: &str) {
        let _ = self.harness.paste(s);
    }

    /// Double-click at a production coordinate.
    pub fn double_click(&mut self, x: u16, y: u16) {
        let _ = self.harness.double_click(x, y);
    }

    /// Last-draw area of a control, if registered.
    #[must_use]
    pub fn area_of(&self, id: Id) -> Option<Rect> {
        self.harness.area_of(id)
    }

    /// Wheel at a production coordinate.
    pub fn wheel(&mut self, axis: Axis, delta: i16, x: u16, y: u16) {
        let _ = self.harness.wheel(axis, delta, x, y);
    }

    /// Pointer event at a production coordinate.
    pub fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) {
        let _ = self.harness.mouse(kind, x, y);
    }

    /// Drag from one cell to another.
    pub fn drag(&mut self, from: (u16, u16), to: (u16, u16)) {
        let _ = self.harness.drag(from, to);
    }

    /// Resize the production test terminal.
    pub fn resize(&mut self, width: u16, height: u16) {
        let _ = self.harness.resize(width, height);
    }

    /// Find a painted label, matching `H::find`.
    #[must_use]
    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        self.harness.find(needle)
    }

    /// Immutable production app projection.
    #[must_use]
    pub fn app(&self) -> &App {
        self.harness.app()
    }

    /// Observe the last production frame without dispatching input.
    #[must_use]
    pub fn observe(&self, checkpoint: &str) -> ObservedFrame {
        let app = self.harness.app();
        let buffer = self.harness.buffer();
        let area = *buffer.area();
        let mut cells =
            Vec::with_capacity(usize::from(area.width).saturating_mul(usize::from(area.height)));
        for y in 0..area.height {
            for x in 0..area.width {
                cells.push(match buffer.cell((x, y)) {
                    Some(cell) => ObservedCell {
                        symbol: cell.symbol().to_owned(),
                        fg: format!("{:?}", cell.fg),
                        bg: format!("{:?}", cell.bg),
                        modifier: cell.modifier.bits(),
                    },
                    None => ObservedCell::missing(),
                });
            }
        }
        let cursor = self
            .harness
            .cursor()
            .map(|pos| ObservedCursor { x: pos.x, y: pos.y });
        ObservedFrame {
            identity: format!(
                "{id}/{world}/{motion}/{frame}/{width}x{height}/{color}/{checkpoint}",
                id = self.program,
                world = self.scenario.name(),
                motion = motion_name(self.motion),
                frame = self.construct_frame,
                width = area.width,
                height = area.height,
                color = self.color.label(),
                checkpoint = checkpoint,
            ),
            scenario: self.scenario.name().to_owned(),
            motion: motion_name(self.motion).to_owned(),
            construct_frame: self.construct_frame,
            app_frame: app.frame(),
            width: area.width,
            height: area.height,
            color: self.color.label().to_owned(),
            theme: "junie".to_owned(),
            route: route_name(app.route()).to_owned(),
            focus: self.harness.focus().map(|id| format!("{id:?}")),
            cursor,
            now_ms: app.world.clock.now_ms,
            now_secs: app.world.now_secs(),
            clock_running: app.world.clock.running,
            motion_seed: MOTION_SEED,
            epoch_secs: EPOCH_SECS,
            digest: self.harness.snapshot().digest(),
            text: self.harness.text(),
            cells,
            quit: app.should_quit() || self.harness.runtime().quit_requested(),
            selected_row: app.manager.selected_row().stable_key(),
        }
    }
}

/// Stable CLI motion name.
#[must_use]
pub const fn motion_name(motion: Motion) -> &'static str {
    match motion {
        Motion::Full => "full",
        Motion::Reduced => "reduced",
        Motion::Paused => "paused",
    }
}

/// Stable route name for semantic records.
#[must_use]
pub const fn route_name(route: Route) -> &'static str {
    match route {
        Route::Intro => "intro",
        Route::Manager => "manager",
        Route::Prelude => "prelude",
        Route::Editor => "editor",
        Route::Accounts => "accounts",
        Route::Usage => "usage",
        Route::Settings => "settings",
        Route::Launch => "launch",
        Route::Cockpit => "cockpit",
        Route::Handoff => "handoff",
        Route::Capsule => "capsule",
        Route::Outro => "outro",
    }
}

/// Color-level label matching `junie_tui_testing` digest keys.
#[must_use]
pub const fn color_label(color: ColorLevel) -> &'static str {
    match color {
        ColorLevel::TrueColor => "truecolor",
        ColorLevel::Ansi256 => "256",
        ColorLevel::Ansi16 => "16",
        ColorLevel::Mono => "mono",
        _ => "other",
    }
}
