//! Production driver: `Runtime` → `App::update` / `App::draw`.
//!
//! No test-harness text search. Pointers use frozen cells or source `Id`.

use std::time::Duration;

use junie_tui::{
    Id, Input, Key, KeyCode, KeyModifiers, Moment, Mouse, MouseKind, Position, Rect, Runtime, Theme,
};
use ratatui_core::buffer::Buffer;
use showcase_app::App;

use crate::actions::{Action, ActionProgram, unfrozen_error};
use crate::clock::InstantClock;
use crate::color::{ColorSpec, TerminalSize};
use crate::error::AdapterError;
use crate::observe::{FrameObservation, HitObs};
use crate::pages::OraclePageId;

const SETTLE_BUDGET: usize = 16;

/// Headless production session with an Instant clock seam.
#[derive(Debug)]
pub struct ShowcaseDriver {
    runtime: Runtime<App>,
    buffer: Buffer,
    clock: InstantClock,
    size: TerminalSize,
    color: ColorSpec,
    page: OraclePageId,
}

impl ShowcaseDriver {
    /// Construct, bootstrap, and present a fresh production App for `page`.
    ///
    /// Theme and route follow the production CLI: `Theme::junie()` downgraded
    /// to the requested colour level, and `App::with_page` for the
    /// `PageId::from_name` route. Oracle-only pages report
    /// [`AdapterError::ProductionRouteAbsent`].
    pub fn open(
        page: OraclePageId,
        size: TerminalSize,
        color: ColorSpec,
    ) -> Result<Self, AdapterError> {
        let Some(production) = page.production() else {
            return Err(AdapterError::ProductionRouteAbsent { page });
        };
        let theme = Theme::junie().downgrade(color.level());
        let mut runtime = Runtime::new(App::with_page(production), theme);
        let _ = runtime.initialize();
        let mut driver = Self {
            runtime,
            buffer: Buffer::empty(size.rect()),
            clock: InstantClock::new(),
            size,
            color,
            page,
        };
        driver.present()?;
        Ok(driver)
    }

    /// Instant clock seam (logical Instant, not wall time, not text search).
    #[must_use]
    pub const fn clock(&self) -> InstantClock {
        self.clock
    }

    /// Runtime moment matching the Instant elapsed duration.
    #[must_use]
    pub fn runtime_now(&self) -> Moment {
        self.runtime.now()
    }

    /// Production App.
    #[must_use]
    pub const fn app(&self) -> &App {
        self.runtime.app()
    }

    /// Focused production control after the last present.
    #[must_use]
    pub const fn focus(&self) -> Option<Id> {
        self.runtime.focus()
    }

    /// Present frames and settle focus until input is ready.
    pub fn present(&mut self) -> Result<(), AdapterError> {
        for _ in 0..SETTLE_BUDGET {
            if self.runtime.needs_settle() {
                drop(self.runtime.settle());
            }
            self.buffer = Buffer::empty(self.size.rect());
            self.runtime
                .draw_buffer(self.size.rect(), &mut self.buffer)
                .commit_presented();
            if !self.runtime.needs_settle() && !self.runtime.needs_present() {
                return Ok(());
            }
        }
        Err(AdapterError::PresentDidNotSettle)
    }

    /// Handle one production input, then present.
    pub fn handle(&mut self, input: Input) -> Result<(), AdapterError> {
        if self.runtime.needs_present() || self.runtime.needs_settle() {
            self.present()?;
        }
        drop(
            self.runtime
                .handle(input)
                .map_err(|_| AdapterError::PendingInput)?,
        );
        self.present()
    }

    /// Key press.
    pub fn key(&mut self, key: Key) -> Result<(), AdapterError> {
        self.handle(Input::Key(key))
    }

    /// Unmodified key code.
    pub fn key_code(&mut self, code: KeyCode) -> Result<(), AdapterError> {
        self.key(Key {
            code,
            mods: KeyModifiers::NONE,
        })
    }

    /// Type one literal character (unmodified `Char` key).
    pub fn type_char(&mut self, value: char) -> Result<(), AdapterError> {
        self.key_code(KeyCode::Char(value))
    }

    /// Production Tick at unchanged Instant/Moment.
    pub fn tick(&mut self) -> Result<(), AdapterError> {
        self.handle(Input::Tick)
    }

    /// `n` production ticks.
    pub fn ticks(&mut self, count: usize) -> Result<(), AdapterError> {
        for _ in 0..count {
            self.tick()?;
        }
        Ok(())
    }

    /// Advance the Instant clock and matching runtime Moment, then present.
    /// Does not deliver Tick; domain ticks stay distinct from elapsed time.
    pub fn set_elapsed(&mut self, elapsed: Duration) -> Result<(), AdapterError> {
        self.clock.set_elapsed(elapsed);
        self.runtime
            .advance_to(self.clock.moment())
            .map_err(|_| AdapterError::ClockBackwards)?;
        for _ in 0..SETTLE_BUDGET {
            if !self.runtime.needs_settle() {
                break;
            }
            drop(self.runtime.settle());
        }
        self.present()
    }

    /// Frozen pointer event. Coordinates are caller-supplied, never searched.
    pub fn pointer(&mut self, kind: MouseKind, x: u16, y: u16) -> Result<(), AdapterError> {
        self.handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        }))
    }

    /// Click the published area of a source `Id` (not a text search).
    pub fn click_id(&mut self, id: Id) -> Result<(), AdapterError> {
        self.present()?;
        let Some(area) = self.runtime.area_of(id) else {
            return Err(AdapterError::UnaddressableId);
        };
        let (x, y) = centre(area);
        self.pointer(MouseKind::Down, x, y)?;
        self.pointer(MouseKind::Up, x, y)
    }

    /// Tab until `id` is focused, then Enter. Uses source identity, not text search.
    pub fn activate_id(&mut self, id: Id) -> Result<(), AdapterError> {
        self.present()?;
        for _ in 0..64 {
            if self.runtime.focus() == Some(id) {
                return self.key_code(KeyCode::Enter);
            }
            self.key_code(KeyCode::Tab)?;
        }
        Err(AdapterError::UnaddressableId)
    }

    /// Resize, then present the immediate resize frame.
    pub fn resize(&mut self, size: TerminalSize) -> Result<(), AdapterError> {
        self.size = size;
        self.buffer = Buffer::empty(size.rect());
        self.handle(Input::Resize(size.cols, size.rows))
    }

    /// Execute a flattened program. Unfrozen labels fail closed.
    pub fn execute(&mut self, program: &ActionProgram) -> Result<(), AdapterError> {
        for step in &program.steps {
            if let Some(error) = unfrozen_error(step) {
                return Err(error);
            }
            match step {
                Action::FreshDraw => self.present()?,
                Action::Tick => self.tick()?,
                Action::Ticks(count) => self.ticks(*count)?,
                Action::Time(elapsed) => self.set_elapsed(*elapsed)?,
                Action::TimeThenTick(elapsed) => {
                    self.set_elapsed(*elapsed)?;
                    self.tick()?;
                }
                Action::Resize(size) => self.resize(*size)?,
                Action::Key(key) => self.key(*key)?,
                Action::Pointer { kind, x, y } => self.dispatch_pointer(*kind, *x, *y)?,
                Action::Unfrozen { .. } => {}
            }
        }
        Ok(())
    }

    fn dispatch_pointer(
        &mut self,
        kind: crate::actions::PointerKind,
        x: u16,
        y: u16,
    ) -> Result<(), AdapterError> {
        use crate::actions::PointerKind;
        match kind {
            PointerKind::Hover => self.pointer(MouseKind::Move, x, y),
            PointerKind::Down => self.pointer(MouseKind::Down, x, y),
            PointerKind::Up => self.pointer(MouseKind::Up, x, y),
            PointerKind::Click => {
                self.pointer(MouseKind::Down, x, y)?;
                self.pointer(MouseKind::Up, x, y)
            }
            PointerKind::Wheel(delta) => {
                self.pointer(MouseKind::Wheel(junie_tui::Axis::V, delta), x, y)
            }
            PointerKind::WheelH(delta) => {
                self.pointer(MouseKind::Wheel(junie_tui::Axis::H, delta), x, y)
            }
        }
    }

    /// Complete observation of the last presented frame.
    pub fn observe(&self) -> Result<FrameObservation, AdapterError> {
        let production_page = Some(OraclePageId::from_production(self.runtime.app().page()));
        let hits = self
            .runtime
            .registry()
            .regions()
            .iter()
            .map(|region| HitObs {
                owner: region.owner,
                x: region.area.x,
                y: region.area.y,
                width: region.area.width,
                height: region.area.height,
                layer: region.layer,
            })
            .collect();
        FrameObservation::capture(
            self.page,
            production_page,
            self.size,
            self.color,
            &self.buffer,
            self.runtime.cursor(),
            self.runtime.focus(),
            self.runtime.hover(),
            self.runtime.app().quit() || self.runtime.quit_requested(),
            self.clock.elapsed().as_millis(),
            self.runtime.now().as_duration().as_millis(),
            hits,
        )
    }
}

fn centre(area: Rect) -> (u16, u16) {
    (
        area.x
            .saturating_add(area.width / 2)
            .min(area.right().saturating_sub(1)),
        area.y
            .saturating_add(area.height / 2)
            .min(area.bottom().saturating_sub(1)),
    )
}

/// Capture one expanded case. Diff and other missing routes are observations.
pub fn capture_case(
    page: OraclePageId,
    size: TerminalSize,
    color: ColorSpec,
    program: &ActionProgram,
) -> Result<FrameObservation, AdapterError> {
    match ShowcaseDriver::open(page, size, color) {
        Ok(mut driver) => {
            driver.execute(program)?;
            driver.observe()
        }
        Err(AdapterError::ProductionRouteAbsent { page }) => Ok(FrameObservation {
            oracle_page: page,
            production_page: None,
            size,
            color,
            cells: Vec::new(),
            cursor: None,
            focus: None,
            hover: None,
            quit: false,
            elapsed_ms: 0,
            moment_ms: 0,
            hits: Vec::new(),
            production_route_absent: true,
        }),
        Err(other) => Err(other),
    }
}
