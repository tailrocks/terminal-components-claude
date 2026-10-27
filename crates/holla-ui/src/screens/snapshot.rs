use crate::sim::world::{SystemProcess, SystemSnapshot, World};
use termrock::{
    Button, Cx, Id, ItemKey, Meter, NavList, NavListState, Props, Rect, Response, TextSection,
    TextViewport, Ui, Variant, ViewportAction, ViewportLine, ViewportState,
};

pub(crate) const PROCESSES: Id = Id::root("holla.snapshot.processes");
pub(crate) const RESOURCES_BODY: Id = Id::root("holla.snapshot.resources.body");
pub(crate) const MONITOR_BUTTON: Id = Id::root("holla.snapshot.resources.monitor");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Kind {
    #[default]
    Resources,
    Processes,
}

#[derive(Default)]
pub(crate) struct State {
    processes: NavListState,
    resources: ViewportState,
}

pub(crate) enum Event {
    Copy(String),
    OpenMonitor,
}

impl State {
    pub(crate) fn update(
        &mut self,
        world: &World,
        kind: Kind,
        cx: &mut Cx<'_>,
        active: bool,
    ) -> (Response<()>, Option<Event>) {
        if !active {
            return (Response::ignored(), None);
        }
        match kind {
            Kind::Resources => {
                let lines = process_lines(world.system.as_ref());
                let viewport_lines = plain_lines(&lines);
                let viewport = TextViewport::new(RESOURCES_BODY)
                    .wrap(false)
                    .reserve_gutter(false)
                    .update(cx, &mut self.resources, &viewport_lines);
                let copy = match viewport.action_ref() {
                    Some(ViewportAction::Copy(text)) => Some(Event::Copy(text.clone())),
                    _ => None,
                };
                let mut response = viewport.erase();
                let button = Button::new(MONITOR_BUTTON, "Open btm")
                    .variant(Variant::SUBTLE)
                    .disabled(!monitor_available(world))
                    .update(cx);
                let event = if button.activated() && monitor_available(world) {
                    Some(Event::OpenMonitor)
                } else {
                    copy
                };
                response |= button.erase();
                (response, event)
            }
            Kind::Processes => {
                let rows = process_rows(world.system.as_ref());
                (
                    process_nav_list()
                        .update(cx, &mut self.processes, &rows)
                        .erase(),
                    None,
                )
            }
        }
    }
}

pub(crate) fn draw(world: &World, kind: Kind, state: &State, ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let Some(system) = world.system.as_ref() else {
        TextSection::new("System snapshot")
            .subtitle("system inventory unavailable")
            .render(ui, area);
        return;
    };
    let subtitle = format!(
        "{} · {} · simulated live snapshot",
        world.host.name, system.os
    );
    let title = match kind {
        Kind::Resources => "System resources",
        Kind::Processes => "Processes",
    };
    TextSection::new(title).subtitle(&subtitle).render(
        ui,
        Rect::new(area.x, area.y, area.width, area.height.min(2)),
    );
    match kind {
        Kind::Resources => draw_resources(world, system, state, ui, area),
        Kind::Processes => {
            TextSection::new("Processes by CPU and memory").render(
                ui,
                Rect::new(area.x, area.y.saturating_add(3), area.width, 1),
            );
            process_nav_list().draw(
                ui,
                Rect::new(
                    area.x,
                    area.y.saturating_add(4),
                    area.width,
                    area.height.saturating_sub(4),
                ),
                &state.processes,
                &process_rows(Some(system)),
            );
        }
    }
}

fn draw_resources(
    world: &World,
    system: &SystemSnapshot,
    state: &State,
    ui: &mut Ui<'_>,
    area: Rect,
) {
    let mut y = area.y.saturating_add(3);
    let width = area.width.saturating_sub(2);
    let cpu = system.cpu_pct.min(100);
    let memory_pct = percent(
        world.host.metrics.mem_used_mb,
        world.host.metrics.mem_total_mb,
    );
    let metrics = [
        ("CPU", f64::from(cpu) / 100.0, format!("{cpu}%")),
        (
            "Memory",
            f64::from(memory_pct) / 100.0,
            format!(
                "{:.1} / {:.0} GB · swap {:.1} GB",
                f64::from(world.host.metrics.mem_used_mb) / 1_024.0,
                f64::from(world.host.metrics.mem_total_mb) / 1_024.0,
                f64::from(system.swap_used_mb) / 1_024.0
            ),
        ),
    ];
    for (index, (label, ratio, value)) in metrics.iter().enumerate() {
        Meter::new(Id::root(match index {
            0 => "holla.snapshot.resources.cpu",
            _ => "holla.snapshot.resources.memory",
        }))
        .ratio(*ratio)
        .value(value)
        .draw(
            ui,
            Rect::new(area.x.saturating_add(8), y, width.saturating_sub(8), 1),
        );
        if width >= 10 {
            let style = ui.paint_patch(
                &termrock::StylePatch::new().set_fg(termrock::Role::Fg(termrock::FgStep::Muted)),
            );
            ui.paint_str(Rect::new(area.x, y, 8, 1), label, style);
        }
        y = y.saturating_add(1);
    }
    if let Some(disk) = &world.disk {
        let disk_pct = rounded_percent(disk.used_bytes, disk.total_bytes);
        Meter::new(Id::root("holla.snapshot.resources.disk"))
            .ratio(f64::from(disk_pct) / 100.0)
            .value(&format!(
                "{disk_pct}% · {} of {}",
                crate::domain::human_bytes(disk.used_bytes),
                crate::domain::human_bytes(disk.total_bytes)
            ))
            .draw(
                ui,
                Rect::new(area.x.saturating_add(8), y, width.saturating_sub(8), 1),
            );
        let style = ui.paint_patch(
            &termrock::StylePatch::new().set_fg(termrock::Role::Fg(termrock::FgStep::Muted)),
        );
        ui.paint_str(Rect::new(area.x, y, 8, 1), "Disk /", style);
        y = y.saturating_add(1);
    }
    let facts = resource_facts(world, system);
    let facts: Vec<_> = facts
        .iter()
        .map(|(label, value)| (label.as_str(), value.as_str()))
        .collect();
    let facts_height = u16::try_from(facts.len()).unwrap_or(u16::MAX);
    Props::new(&facts).draw(ui, Rect::new(area.x, y, area.width, facts_height));
    y = y.saturating_add(facts_height).saturating_add(1);
    let process_title = TextSection::new("Processes").subtitle("PID · name · CPU · memory · state");
    process_title.render(ui, Rect::new(area.x, y, area.width, 2));
    y = y.saturating_add(2);
    let button_height = 1;
    let button_y = area.bottom().saturating_sub(button_height);
    let process_area = Rect::new(area.x, y, area.width, button_y.saturating_sub(y));
    let lines = process_lines(Some(system));
    let viewport_lines = plain_lines(&lines);
    TextViewport::new(RESOURCES_BODY)
        .wrap(false)
        .reserve_gutter(false)
        .draw(ui, process_area, &state.resources, &viewport_lines);
    if monitor_available(world) {
        Button::new(MONITOR_BUTTON, "Open btm")
            .variant(Variant::SUBTLE)
            .draw(ui, Rect::new(area.x, button_y, area.width, 1));
    }
}

fn resource_facts(world: &World, system: &SystemSnapshot) -> Vec<(String, String)> {
    let load = format!(
        "{} {} {}",
        format_load(world.host.metrics.load_x100.0),
        format_load(world.host.metrics.load_x100.1),
        format_load(world.host.metrics.load_x100.2)
    );
    let cores = system
        .cores
        .iter()
        .map(|core| format!("{core}%"))
        .collect::<Vec<_>>()
        .join(" · ");
    let pressure = match system.pressure {
        Some((some, full)) => format!(
            "some {some}% · full {full}% · {} min",
            system.pressure_minutes
        ),
        None if system.pressure_minutes > 0 => format!("{} min sustained", system.pressure_minutes),
        None => "none".to_owned(),
    };
    let mut facts = vec![
        ("Load".to_owned(), load),
        ("Cores".to_owned(), cores),
        ("Pressure".to_owned(), pressure),
        ("Disk I/O".to_owned(), join_pairs(&system.disk_io)),
        ("Network".to_owned(), join_pairs(&system.network)),
    ];
    if let Some(temp) = system.temp_c {
        facts.push(("Temperature".to_owned(), format!("{temp} °C")));
    }
    facts
}

fn join_pairs(pairs: &[(String, String)]) -> String {
    pairs
        .iter()
        .map(|(label, value)| format!("{label} · {value}"))
        .collect::<Vec<_>>()
        .join(" · ")
}

struct ProcessRow {
    key: ItemKey,
    label: String,
    cpu_pct: u32,
    mem_mb: u32,
}

type ProcessKey = fn(&ProcessRow) -> ItemKey;
type ProcessRender = fn(&ProcessRow, &mut termrock::RowUi<'_>);
type ProcessList = NavList<'static, ProcessRow, ProcessKey, ProcessRender>;

fn process_rows(system: Option<&SystemSnapshot>) -> Vec<ProcessRow> {
    system.map_or_else(Vec::new, |system| {
        let mut rows: Vec<_> = system
            .processes
            .iter()
            .map(|process| ProcessRow {
                key: ItemKey::num(u64::from(process.pid)),
                label: process_label(process),
                cpu_pct: process.cpu_pct,
                mem_mb: process.mem_mb,
            })
            .collect();
        rows.sort_by_key(|row| std::cmp::Reverse((row.cpu_pct, row.mem_mb)));
        rows
    })
}

fn process_key(row: &ProcessRow) -> ItemKey {
    row.key
}

fn process_row(row: &ProcessRow, cells: &mut termrock::RowUi<'_>) {
    cells.label(&row.label);
}

fn process_nav_list() -> ProcessList {
    NavList::new(PROCESSES)
        .scrollable(true)
        .leave_at_boundary(true)
        .key(process_key as ProcessKey)
        .row(process_row as ProcessRender)
}

fn process_label(process: &SystemProcess) -> String {
    let port = process
        .port
        .map_or_else(String::new, |port| format!(" · port {port}"));
    format!(
        "{:<8} {:<22} {:>3}% CPU · {:>5} MB · {}{port}",
        process.pid, process.name, process.cpu_pct, process.mem_mb, process.state
    )
}

fn process_lines(system: Option<&SystemSnapshot>) -> Vec<String> {
    let mut lines = vec![format!(
        "{:<8} {:<22} {:>7} {:>10}   {}",
        "PID", "NAME", "CPU", "MEMORY", "STATE"
    )];
    if let Some(system) = system {
        let mut processes: Vec<_> = system.processes.iter().collect();
        processes.sort_by_key(|process| std::cmp::Reverse(process.cpu_pct));
        lines.extend(processes.into_iter().map(process_label));
    }
    lines
}

fn plain_lines(lines: &[String]) -> Vec<ViewportLine<'_>> {
    lines
        .iter()
        .map(|line| ViewportLine::from(line.as_str()))
        .collect()
}

fn monitor_available(world: &World) -> bool {
    world
        .system
        .as_ref()
        .is_some_and(|system| system.btm_installed)
        && crate::sim::catalogue::catalogue(world)
            .iter()
            .any(|action| action.id == "system.monitor")
}

fn percent(used: u32, total: u32) -> u32 {
    if total == 0 {
        return 0;
    }
    u32::try_from((u128::from(used).saturating_mul(100) / u128::from(total)).min(100))
        .unwrap_or(100)
}

fn rounded_percent(used: u64, total: u64) -> u32 {
    if total == 0 {
        return 0;
    }
    let numerator = u128::from(used).saturating_mul(100);
    let denominator = u128::from(total);
    u32::try_from(numerator.saturating_add(denominator / 2) / denominator).unwrap_or(u32::MAX)
}

fn format_load(value: u32) -> String {
    format!("{}.{:02}", value / 100, value % 100)
}
