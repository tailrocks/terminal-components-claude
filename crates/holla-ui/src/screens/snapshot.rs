use crate::sim::world::{SystemSnapshot, World};
use termrock::{
    Button, Constraints, Cx, FgStep, FrameRead, Hint, HintKey, HintLayer, Id, ItemKey, Meter,
    MeterVisual, NavList, NavListState, Panel, PanelKind, Part, Props, PropsRow, Rect, Response,
    Role, RowUi, Span, StylePatch, TextSection, TextViewport, Ui, Variant, ViewportAction,
    ViewportLine, ViewportState,
};

pub(crate) const PROCESSES: Id = Id::root("holla.snapshot.processes");
pub(crate) const RESOURCES_BODY: Id = Id::root("holla.snapshot.resources.body");
pub(crate) const MONITOR_BUTTON: Id = Id::root("holla.snapshot.resources.monitor");
const RESOURCES_PANEL: Id = Id::root("holla.snapshot.resources.panel");
const RESOURCE_FACT_PARTS: &[(Part, StylePatch)] = &[(
    Part::META,
    StylePatch::new().set_fg(Role::Fg(FgStep::Muted)),
)];
const RESOURCE_MIN_PANEL_HEIGHT: u16 = 5;

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

struct FactRow {
    label: String,
    value: String,
    tone: Role,
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
        if world.scenario == crate::scenario::Scenario::FirstUse && kind == Kind::Resources {
            let Some(system) = world.system.as_ref() else {
                return (Response::ignored(), None);
            };
            let lines = resource_process_lines(system);
            let spans = process_spans(&lines);
            let viewport_lines = viewport_lines(&spans);
            let mut viewport = TextViewport::new(RESOURCES_BODY).wrap(false).update(
                cx,
                &mut self.resources,
                &viewport_lines,
            );
            let event = match viewport.take_action() {
                Some(ViewportAction::Copy(text)) => Some(Event::Copy(text)),
                _ => None,
            };
            let mut response = viewport.erase();
            let monitor = monitor_available(world);
            let button = Button::new(MONITOR_BUTTON, "Open btm")
                .variant(Variant::SUBTLE)
                .disabled(!monitor)
                .update(cx);
            let event = if button.activated() && monitor {
                Some(Event::OpenMonitor)
            } else {
                event
            };
            response |= button.erase();
            return (response, event);
        }
        let rows = world.system.as_ref().map_or_else(Vec::new, process_rows);
        (
            process_nav_list()
                .update(cx, &mut self.processes, &rows)
                .erase(),
            None,
        )
    }

    pub(crate) fn hints(&self, world: &World, monitor_focused: bool) -> HintLayer {
        let monitor_focused = monitor_focused && monitor_available(world);
        let hints = if monitor_focused {
            vec![
                hint("Enter", "Run"),
                hint("Tab", "Next"),
                hint("Esc", "Back"),
            ]
        } else {
            let mut hints = vec![hint("↑↓", "Scroll"), hint("y", "Copy")];
            if monitor_available(world) {
                hints.push(hint("Tab", "Next actions"));
            }
            hints.push(hint("Esc", "Back"));
            hints
        };
        HintLayer {
            hints,
            ..HintLayer::default()
        }
    }
}

fn hint(key: &'static str, label: &'static str) -> Hint {
    Hint {
        key: HintKey::Label(key),
        label,
        priority: 50,
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
    if world.scenario == crate::scenario::Scenario::FirstUse && kind == Kind::Resources {
        draw_first_use_resources(world, state, ui, area);
        return;
    }
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

fn draw_first_use_resources(world: &World, state: &State, ui: &mut Ui<'_>, area: Rect) {
    let Some(system) = world.system.as_ref() else {
        TextSection::new("System resources")
            .subtitle("system inventory unavailable")
            .render(ui, area);
        return;
    };
    if area.is_empty() {
        return;
    }

    draw_resource_header(world, system, ui, area);
    let meter_rows = 2usize.saturating_add(usize::from(world.disk.is_some()));
    let meter_start = area.y.saturating_add(2);
    draw_resource_meters(world, system, ui, area, meter_start);

    let facts = resource_facts(world, system);
    let props_rows: Vec<_> = facts
        .iter()
        .enumerate()
        .map(|(index, row)| {
            PropsRow::new(ItemKey::index(index), &row.label, &row.value).tone(row.tone)
        })
        .collect();
    let props = Props::rich(&props_rows).patch_part(RESOURCE_FACT_PARTS);
    let props_y = meter_start
        .saturating_add(u16::try_from(meter_rows).unwrap_or(u16::MAX))
        .saturating_add(1);
    let follow_height = if monitor_available(world) { 2 } else { 0 };
    let follow_top = area.bottom().saturating_sub(follow_height);
    let props_width = area.width.saturating_sub(2);
    let facts_room = follow_top.saturating_sub(props_y);
    let measured = props
        .measure(ui, Constraints::loose(props_width, facts_room))
        .preferred
        .1;
    let facts_height =
        measured.min(facts_room.saturating_sub(RESOURCE_MIN_PANEL_HEIGHT.saturating_add(1)));
    let facts_area = Rect::new(area.x.saturating_add(1), props_y, props_width, facts_height);
    let used = props.draw(ui, facts_area).height;
    let panel_y = props_y.saturating_add(used).saturating_add(1);
    let panel_area = Rect::new(
        area.x,
        panel_y,
        area.width,
        follow_top.saturating_sub(panel_y),
    );
    draw_resource_processes(system, state, ui, panel_area);

    if follow_height > 0 {
        draw_monitor_action(ui, area, follow_top);
    }
}

fn draw_resource_header(world: &World, system: &SystemSnapshot, ui: &mut Ui<'_>, area: Rect) {
    let y = area.y;
    let title = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .add(termrock::Modifier::BOLD),
    );
    let subtitle = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
    let left = area.x.saturating_add(1);
    let meta = format!(
        "{} · {} · live · {}",
        world.host.name,
        system.os,
        world.clock.ago(world.clock.now_secs().saturating_sub(1))
    );
    let right = area.right().saturating_sub(1);
    let inner_width = right.saturating_sub(left);
    let title_width = termrock::width("System resources");
    ui.paint_str(
        Rect::new(left, y, inner_width.min(title_width), 1),
        "System resources",
        title,
    );
    let meta_width = termrock::width(&meta);
    let gap = 4u16;
    if title_width.saturating_add(gap).saturating_add(meta_width) <= inner_width {
        let meta_x = right.saturating_sub(meta_width);
        ui.paint_str(Rect::new(meta_x, y, meta_width, 1), &meta, subtitle);
    }
}

fn draw_resource_meters(
    world: &World,
    system: &SystemSnapshot,
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
) {
    let labels = ["CPU", "Memory", "Disk /"];
    let available = area.width.saturating_sub(2);
    let label_width = labels
        .iter()
        .map(|label| termrock::width(label))
        .max()
        .unwrap_or(0)
        .max(12)
        .min(available);
    let meter_x = area
        .x
        .saturating_add(1)
        .saturating_add(label_width)
        .saturating_add(1);
    let meter_width = area
        .right()
        .saturating_sub(meter_x)
        .saturating_sub(2)
        .min(34);
    let values = resource_meter_values(world, system);
    for (index, (label, ratio, value)) in values.iter().enumerate() {
        let Some(row_y) = y.checked_add(u16::try_from(index).unwrap_or(u16::MAX)) else {
            continue;
        };
        if row_y >= area.bottom() {
            break;
        }
        let style = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
        ui.paint_str(
            Rect::new(area.x.saturating_add(1), row_y, label_width, 1),
            label,
            style,
        );
        Meter::new(Id::root(match index {
            0 => "holla.snapshot.resources.cpu",
            1 => "holla.snapshot.resources.memory",
            _ => "holla.snapshot.resources.disk",
        }))
        .ratio(*ratio)
        .value(value)
        .visual(MeterVisual::Block)
        .draw(ui, Rect::new(meter_x, row_y, meter_width, 1));
    }
}

fn resource_meter_values(
    world: &World,
    system: &SystemSnapshot,
) -> Vec<(&'static str, f64, String)> {
    let memory_percent = truncated_percent(
        world.host.metrics.mem_used_mb,
        world.host.metrics.mem_total_mb,
    );
    let mut rows = vec![
        (
            "CPU",
            f64::from(system.cpu_pct) / 100.0,
            format!("{}%", system.cpu_pct),
        ),
        (
            "Memory",
            f64::from(memory_percent) / 100.0,
            format!("{memory_percent}%"),
        ),
    ];
    if let Some(disk) = world.disk.as_ref() {
        let percent = disk.used_percent().min(100);
        rows.push(("Disk /", f64::from(percent) / 100.0, format!("{percent}%")));
    }
    rows
}

fn resource_facts(world: &World, system: &SystemSnapshot) -> Vec<FactRow> {
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
        .join(" ");
    let memory = format!(
        "{:.1} of {:.0} GB · swap {:.1} GB",
        f64::from(world.host.metrics.mem_used_mb) / 1_024.0,
        f64::from(world.host.metrics.mem_total_mb) / 1_024.0,
        f64::from(system.swap_used_mb) / 1_024.0
    );
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
    let mut values = vec![
        FactRow {
            label: "Load".to_owned(),
            value: load,
            tone: Role::Fg(FgStep::Primary),
        },
        FactRow {
            label: "Cores".to_owned(),
            value: cores,
            tone: Role::Fg(FgStep::Muted),
        },
        FactRow {
            label: "Memory".to_owned(),
            value: memory,
            tone: Role::Fg(FgStep::Primary),
        },
        FactRow {
            label: "Pressure".to_owned(),
            value: pressure,
            tone: if system.pressure_minutes > 0 {
                Role::Warning
            } else {
                Role::Fg(FgStep::Muted)
            },
        },
    ];
    values.extend(system.disk_io.iter().map(|(device, value)| FactRow {
        label: "Disk I/O".to_owned(),
        value: format!("{device} · {value}"),
        tone: Role::Fg(FgStep::Muted),
    }));
    values.extend(system.network.iter().map(|(interface, value)| FactRow {
        label: "Network".to_owned(),
        value: format!("{interface} · {value}"),
        tone: Role::Fg(FgStep::Muted),
    }));
    if let Some(temp) = system.temp_c {
        values.push(FactRow {
            label: "Temperature".to_owned(),
            value: format!("{temp} °C"),
            tone: if temp > 80 {
                Role::Warning
            } else {
                Role::Fg(FgStep::Muted)
            },
        });
    }
    values
}

fn draw_resource_processes(system: &SystemSnapshot, state: &State, ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let lines = resource_process_lines(system);
    let spans = process_spans(&lines);
    let viewport_lines = viewport_lines(&spans);
    let meta = format!("{} lines · y copies", lines.len());
    let focused = ui
        .state(RESOURCES_BODY)
        .contains(termrock::StateFlags::FOCUSED);
    Panel::new(RESOURCES_PANEL)
        .kind(PanelKind::Framed)
        .title("Processes")
        .meta(&meta)
        .focused(focused)
        .draw(ui, area, |ui, inner| {
            TextViewport::new(RESOURCES_BODY).wrap(false).draw(
                ui,
                inner,
                &state.resources,
                &viewport_lines,
            );
        });
}

fn draw_monitor_action(ui: &mut Ui<'_>, area: Rect, y: u16) {
    if y >= area.bottom() {
        return;
    }
    let label_style = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint)));
    ui.paint_str(
        Rect::new(area.x.saturating_add(1), y, 5, 1),
        "Next",
        label_style,
    );
    Button::new(MONITOR_BUTTON, "Open btm")
        .variant(Variant::SUBTLE)
        .draw(
            ui,
            Rect::new(area.x.saturating_add(7), y, area.width.saturating_sub(8), 1),
        );
}

pub(crate) fn copy_resource_body(world: &World) -> Option<Event> {
    if world.scenario != crate::scenario::Scenario::FirstUse {
        return None;
    }
    let system = world.system.as_ref()?;
    let text = resource_process_lines(system)
        .into_iter()
        .map(|(line, _)| line)
        .collect::<Vec<_>>()
        .join("\n");
    Some(Event::Copy(text))
}

fn resource_process_lines(system: &SystemSnapshot) -> Vec<(String, Role)> {
    let mut lines = Vec::with_capacity(system.processes.len().saturating_add(1));
    lines.push((
        format!(
            "{:<8}{:<20}{:>6}{:>9}   {}",
            "pid", "name", "cpu", "mem", "state"
        ),
        Role::Fg(FgStep::Muted),
    ));
    lines.extend(system.processes.iter().map(|process| {
        let port = process
            .port
            .map_or_else(String::new, |port| format!(" · port {port}"));
        (
            format!(
                "{:<8}{:<20}{:>5}%{:>7} MB   {}{port}",
                process.pid, process.name, process.cpu_pct, process.mem_mb, process.state
            ),
            if process.cpu_pct >= 30 {
                Role::Warning
            } else {
                Role::Fg(FgStep::Primary)
            },
        )
    }));
    lines
}

fn process_spans(lines: &[(String, Role)]) -> Vec<[Span<'_>; 1]> {
    lines
        .iter()
        .map(|(line, role)| [Span::new(line).role(*role)])
        .collect()
}

fn viewport_lines<'a>(spans: &'a [[Span<'a>; 1]]) -> Vec<ViewportLine<'a>> {
    spans
        .iter()
        .map(|line| ViewportLine::Spans(line.as_slice()))
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

fn truncated_percent(used: u32, total: u32) -> u32 {
    if total == 0 {
        return 0;
    }
    u32::try_from((u128::from(used).saturating_mul(100) / u128::from(total)).min(100))
        .unwrap_or(100)
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
            "Load  {} {} {}",
            format_load(world.host.metrics.load_x100.0),
            format_load(world.host.metrics.load_x100.1),
            format_load(world.host.metrics.load_x100.2)
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

fn format_load(value_x100: u32) -> String {
    let whole = value_x100 / 100;
    let fraction = value_x100 % 100;
    if fraction.is_multiple_of(10) {
        format!("{whole}.{}", fraction / 10)
    } else {
        format!("{whole}.{fraction:02}")
    }
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
