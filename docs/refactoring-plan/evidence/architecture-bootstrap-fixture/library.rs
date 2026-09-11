//! Frozen miniature architecture subject, not a terminal-components API.
//! Integer cells are actual writes; the private driver compares the whole buffer.

#[derive(Clone, Copy)]
pub struct Props { pub id: u32, pub disabled: bool }
impl Props {
    pub fn new(id: u32) -> Self { Self { id, disabled: false } }
    pub fn disabled(mut self, value: bool) -> Self { self.disabled = value; self }
}

pub const PARTS: &[u32] = &[0, 1, 2];
pub const DOCUMENTED_SLOTS: &[u32] = &[1, 2];
pub const REGISTRY: &[u32] = &[7];
pub type SlotFn = fn(&mut Ui, usize);

pub struct Ui {
    pub cells: [u32; 4],
    pub writes: Vec<[u32; 3]>,
    pub resolutions: Vec<[u32; 3]>,
    pub draws: u32,
    pub updates: u32,
    pub owner: u32,
    pub poison: bool,
}
impl Ui {
    pub fn new(poison: bool) -> Self {
        Self { cells: [32; 4], writes: Vec::new(), resolutions: Vec::new(),
            draws: 0, updates: 0, owner: 1, poison }
    }
    pub fn resolve(&mut self, id: u32, part: u32) {
        self.resolutions.push([id, part, self.owner]);
    }
    pub fn paint(&mut self, index: usize, symbol: u32) {
        self.cells[index] = symbol;
        self.writes.push([index as u32, symbol, self.owner]);
    }
    pub fn row(&mut self, f: impl FnOnce(&mut Self)) {
        let previous = self.owner;
        self.owner = 2;
        f(self);
        self.owner = previous;
    }
}

pub struct Widget;
impl Widget {
    pub fn update(props: Props, value: &mut u32, ui: &mut Ui) {
        ui.updates += 1;
        if !props.disabled { *value += 1; }
    }
    pub fn draw(props: Props, value: u32, busy: bool, slot: Option<(u32, SlotFn)>, ui: &mut Ui) {
        ui.draws += 1;
        ui.resolve(props.id, 0);
        ui.paint(0, 91);
        ui.resolve(props.id, 1);
        ui.paint(1, 48 + value % 10 + if ui.poison { 16 } else { 0 });
        ui.resolve(props.id, 2);
        ui.paint(2, if busy { 42 } else { 43 });
        if let Some((part, render)) = slot { render(ui, part as usize); }
    }
}

/// Same-id child composition is owned; a caller row is not owned.
pub fn compose(props: Props, ui: &mut Ui) {
    ui.resolve(props.id, 1);
    ui.row(|ui| {
        ui.resolve(props.id, 99);
        ui.row(|ui| ui.resolve(props.id, 98));
        ui.resolve(props.id, 97);
    });
    ui.resolve(props.id, 0);
}
