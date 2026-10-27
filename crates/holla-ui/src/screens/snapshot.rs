use crate::sim::world::{SystemSnapshot, World};
use termrock::{
    Cx, FgStep, Id, ItemKey, Meter, MeterVisual, NavList, NavListState, Rect, Response, Role,
    RowUi, StylePatch, TextSection, Ui,
};

pub(crate) const PROCESSES: Id = Id::root("holla.snapshot.processes");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Kind {
    #[default]
    Resources,
    Processes,
}

#[derive(Default)]
pub(crate) struct State {
    processes: NavListState,
}

impl State {
    pub(crate) fn update(&mut self, world: &World, cx: &mut Cx<'_>, active: bool) -> Response<()> {
        if !active {
            return Response::ignored();
        }
        let rows = world.system.as_ref().map_or_else(Vec::new, process_rows);
        process_nav_list()
            .update(cx, &mut self.processes, &rows)
            .erase()
    }
}

struct ProcessRow {
    key: String,
    label: String,
    cpu_pct: u32,
}

type ProcessKey = fn(&ProcessRow) -> ItemKey;
type ProcessRender = fn(&ProcessRow, &mut RowUi<'_>);
type ProcessList = NavList<'static, ProcessRow, ProcessKey, ProcessRender>;

pub(crate) fn draw(world: &World, kind: Kind, state: &State, ui: &mut Ui<'_>, area: Rect) {
    let Some(system) = world.system.as_ref() else {
        TextSection::new("System snapshot")
            .subtitle("system inventory unavailable")
            .render(ui, area);
        return;
    };
    let title = match kind {
        Kind::Resources => "System resources",
        Kind::Processes => "Process tree",
    };
    let subtitle = format!("{} · {}", world.host.name, system.os);
    TextSection::new(title).subtitle(&subtitle).render(ui, area);

    match kind {
        Kind::Resources => draw_resources(system, world, state, ui, area),
        Kind::Processes => draw_processes(system, state, ui, area),
    }
}

fn draw_resources(
    system: &SystemSnapshot,
    world: &World,
    state: &State,
    ui: &mut Ui<'_>,
    area: Rect,
) {
    let mut y = area.y.saturating_add(3);
    let end = area.bottom();
    draw_meter(
        ui,
        Id::root("holla.snapshot.cpu"),
        Rect::new(area.x.saturating_add(1), y, area.width.saturating_sub(2), 1),
        f64::from(system.cpu_pct) / 100.0,
        &format!("CPU {}%", system.cpu_pct),
    );
    y = y.saturating_add(1);
    draw_text(
        ui,
        area,
        y,
        &format!(
            "Cores  {}",
            system
                .cores
                .iter()
                .map(|core| format!("{core}%"))
                .collect::<Vec<_>>()
                .join(" ")
        ),
    );
    y = y.saturating_add(1);
    draw_text(
        ui,
        area,
        y,
        &format!(
            "Load  {:.2} {:.2} {:.2}",
            f64::from(world.host.metrics.load_x100.0) / 100.0,
            f64::from(world.host.metrics.load_x100.1) / 100.0,
            f64::from(world.host.metrics.load_x100.2) / 100.0
        ),
    );
    y = y.saturating_add(1);
    let memory_ratio = if world.host.metrics.mem_total_mb == 0 {
        0.0
    } else {
        f64::from(world.host.metrics.mem_used_mb) / f64::from(world.host.metrics.mem_total_mb)
    };
    draw_meter(
        ui,
        Id::root("holla.snapshot.memory"),
        Rect::new(area.x.saturating_add(1), y, area.width.saturating_sub(2), 1),
        memory_ratio,
        &format!(
            "Memory {:.1} of {:.0} GB · swap {:.1} GB",
            f64::from(world.host.metrics.mem_used_mb) / 1_024.0,
            f64::from(world.host.metrics.mem_total_mb) / 1_024.0,
            f64::from(system.swap_used_mb) / 1_024.0
        ),
    );
    y = y.saturating_add(1);
    let pressure = match system.pressure {
        Some((some, full)) => format!(
            "cpu some {some}% · full {full}% · {} min sustained",
            system.pressure_minutes
        ),
        None if system.pressure_minutes > 0 => format!(
            "{} min of sustained CPU pressure (no PSI on {})",
            system.pressure_minutes, system.os
        ),
        None => "none".to_owned(),
    };
    draw_text(ui, area, y, &format!("Pressure  {pressure}"));
    y = y.saturating_add(1);
    if let Some(disk) = &world.disk {
        let percent = rounded_percent(disk.used_bytes, disk.total_bytes);
        draw_meter(
            ui,
            Id::root("holla.snapshot.disk"),
            Rect::new(area.x.saturating_add(1), y, area.width.saturating_sub(2), 1),
            f64::from(percent) / 100.0,
            &format!("Disk /  {percent}%"),
        );
        y = y.saturating_add(1);
    }
    for (device, value) in &system.disk_io {
        draw_text(ui, area, y, &format!("Disk I/O  {device} · {value}"));
        y = y.saturating_add(1);
    }
    for (interface, value) in &system.network {
        draw_text(ui, area, y, &format!("Network  {interface} · {value}"));
        y = y.saturating_add(1);
    }
    if let Some(temp) = system.temp_c {
        draw_text(ui, area, y, &format!("Temperature  {temp} °C"));
        y = y.saturating_add(1);
    }
    draw_process_table(
        system,
        state,
        ui,
        Rect::new(area.x, y, area.width, end.saturating_sub(y)),
    );
}

fn draw_processes(system: &SystemSnapshot, state: &State, ui: &mut Ui<'_>, area: Rect) {
    draw_process_table(
        system,
        state,
        ui,
        Rect::new(
            area.x,
            area.y.saturating_add(3),
            area.width,
            area.height.saturating_sub(3),
        ),
    );
}

fn draw_process_table(system: &SystemSnapshot, state: &State, ui: &mut Ui<'_>, area: Rect) {
    if area.height == 0 {
        return;
    }
    draw_text(
        ui,
        area,
        area.y,
        &format!(
            "{:<8}{:<20}{:>6}{:>9}   {}",
            "pid", "name", "cpu", "mem", "state"
        ),
    );
    draw_process_list(
        system,
        state,
        ui,
        Rect::new(
            area.x,
            area.y.saturating_add(1),
            area.width,
            area.height.saturating_sub(1),
        ),
    );
}

fn draw_process_list(system: &SystemSnapshot, state: &State, ui: &mut Ui<'_>, area: Rect) {
    let rows = process_rows(system);
    process_nav_list().draw(ui, area, &state.processes, &rows);
}

fn process_rows(system: &SystemSnapshot) -> Vec<ProcessRow> {
    system
        .processes
        .iter()
        .map(|process| {
            let port = process
                .port
                .map_or_else(String::new, |port| format!(" · port {port}"));
            ProcessRow {
                key: process.pid.to_string(),
                label: format!(
                    "{:<8}{:<20}{:>5}%{:>7} MB   {}{port}",
                    process.pid, process.name, process.cpu_pct, process.mem_mb, process.state
                ),
                cpu_pct: process.cpu_pct,
            }
        })
        .collect()
}

fn process_nav_list() -> ProcessList {
    NavList::new(PROCESSES)
        .key(process_key as fn(&ProcessRow) -> ItemKey)
        .row(render_process_row as fn(&ProcessRow, &mut RowUi<'_>))
}

fn process_key(row: &ProcessRow) -> ItemKey {
    ItemKey::text(&row.key)
}

fn render_process_row(row: &ProcessRow, cells: &mut RowUi<'_>) {
    if row.cpu_pct >= 30 {
        cells.label_patched(&row.label, &StylePatch::new().set_fg(Role::Warning));
    } else {
        cells.label(&row.label);
    }
}

fn draw_meter(ui: &mut Ui<'_>, id: Id, area: Rect, ratio: f64, value: &str) {
    Meter::new(id)
        .ratio(ratio)
        .value(value)
        .visual(MeterVisual::Line)
        .draw(ui, area);
}

fn draw_text(ui: &mut Ui<'_>, area: Rect, y: u16, text: &str) {
    if y >= area.bottom() {
        return;
    }
    let style = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
    ui.paint_str(
        Rect::new(area.x.saturating_add(2), y, area.width.saturating_sub(4), 1),
        text,
        style,
    );
}

fn rounded_percent(used_bytes: u64, total_bytes: u64) -> u32 {
    if total_bytes == 0 {
        return 0;
    }
    let numerator = u128::from(used_bytes)
        .saturating_mul(100)
        .saturating_add(u128::from(total_bytes) / 2);
    let rounded = numerator / u128::from(total_bytes);
    u32::try_from(rounded.min(100)).unwrap_or(100)
}
