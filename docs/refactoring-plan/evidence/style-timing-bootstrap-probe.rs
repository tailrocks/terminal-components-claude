//! Testing-only observer, inserted only into a disposable pinned-main copy.
use std::cell::{Cell, RefCell};
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode { Disabled, Calibration, Measured }

#[derive(Clone, Debug, Default)]
pub struct Observation {
    pub witness: Vec<u8>,
    pub intervals: Vec<(u8, u128, u128, u32)>,
    pub reported_ns: u128,
    pub mode: u8,
    pub cal_source: u8,
    pub policy: u8,
}

#[derive(Debug)]
pub struct Probe {
    mode: Cell<Mode>,
    witness: RefCell<Vec<u8>>,
    intervals: RefCell<Vec<(u8, u128, u128, u32)>>,
    depth: Cell<u32>,
    epoch: Instant,
}

impl Default for Probe { fn default() -> Self { Self::new() } }
impl Probe {
    pub fn new() -> Self {
        Self { mode: Cell::new(Mode::Disabled), witness: RefCell::new(Vec::with_capacity(8192)),
            intervals: RefCell::new(Vec::with_capacity(8192)), depth: Cell::new(0), epoch: Instant::now() }
    }
    pub fn reset(&self, mode: Mode) {
        assert_eq!(self.depth.get(), 0, "observer scope leaked");
        self.mode.set(mode);
        self.witness.borrow_mut().clear();
        self.intervals.borrow_mut().clear();
    }
    pub fn mode(&self) -> Mode { self.mode.get() }
    pub fn visit(&self, id: u8) {
        // Independent membership witness is outside the timed entry interval.
        self.witness.borrow_mut().push(id);
    }
    pub fn enter(&self, id: u8) -> Guard<'_> {
        let depth = self.depth.get();
        self.depth.set(depth + 1);
        let start = if self.mode.get()==Mode::Disabled {0} else {self.epoch.elapsed().as_nanos()};
        let calibration_stop = if self.mode.get() == Mode::Calibration {
            self.epoch.elapsed().as_nanos()
        } else { start };
        Guard { probe: self, id, start, calibration_stop, depth }
    }
    pub fn observation(&self) -> Observation {
        let intervals = self.intervals.borrow().clone();
        let raw = intervals.iter().map(|(_,a,b,_)| b - a).sum();
        Observation { witness: self.witness.borrow().clone(), intervals,
            reported_ns: raw, mode: match self.mode.get() { Mode::Disabled=>0, Mode::Calibration=>1, Mode::Measured=>2 },
            cal_source: 1, policy: 1 }
    }
}

pub struct Guard<'p> { probe: &'p Probe, id: u8, start: u128, calibration_stop: u128, depth: u32 }
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        let stop = match self.probe.mode.get() {
            Mode::Disabled => self.start,
            Mode::Calibration => self.calibration_stop,
            Mode::Measured => self.probe.epoch.elapsed().as_nanos(),
        };
        self.probe.depth.set(self.depth);
        if self.probe.mode.get() != Mode::Disabled {
            self.probe.intervals.borrow_mut().push((self.id,self.start,stop,self.depth));
        }
    }
}
