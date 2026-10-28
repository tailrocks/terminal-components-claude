use crate::sim::world::World;
use termrock::{FgStep, Rect, Role, StylePatch, TextSection, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Overview,
    Resources,
}

pub(crate) fn draw(world: &World, ui: &mut Ui<'_>, area: Rect, kind: Kind) {
    if area.is_empty() {
        return;
    }
    let frame = format!("Frame {} · {}", world.now_ms().max(0), world.cwd);
    let (title, subtitle) = match kind {
        Kind::Overview => ("Snapshot", frame.as_str()),
        Kind::Resources => ("System resources", frame.as_str()),
    };
    TextSection::new(title).subtitle(subtitle).render(
        ui,
        Rect::new(area.x, area.y, area.width, area.height.min(2)),
    );
    if kind == Kind::Overview {
        return;
    }

    let Some(system) = &world.system else {
        paint_lines(ui, area, &["System snapshot unavailable".into()]);
        return;
    };
    let metrics = &world.host.metrics;
    let disk = world.disk.as_ref().map_or_else(
        || "unavailable".to_owned(),
        |disk| {
            format!(
                "{}% full · {} of {}",
                rounded_percent(disk.used_bytes, disk.total_bytes),
                crate::domain::human_bytes(disk.used_bytes),
                crate::domain::human_bytes(disk.total_bytes)
            )
        },
    );
    let memory = format!(
        "{:.1} / {:.1} GiB",
        f64::from(metrics.mem_used_mb) / 1_024.0,
        f64::from(metrics.mem_total_mb) / 1_024.0
    );
    let load = format!(
        "{} / {} / {}",
        load_value(metrics.load_x100.0),
        load_value(metrics.load_x100.1),
        load_value(metrics.load_x100.2)
    );
    let pressure = if system.pressure_minutes > 0 {
        format!("sustained for {} min", system.pressure_minutes)
    } else {
        "not active".to_owned()
    };
    let mut lines = vec![
        format!("Host: {}", world.host.identity()),
        format!("CPU: {}% · load {load}", system.cpu_percent),
        format!(
            "Memory: {memory} · swap {:.1} GiB",
            f64::from(system.swap_used_mb) / 1_024.0
        ),
        format!("Pressure: {pressure}"),
        format!("Disk: {disk}"),
    ];
    if system.network.is_empty() {
        lines.push("Network: unavailable".into());
    } else {
        lines.extend(
            system
                .network
                .iter()
                .map(|(name, detail)| format!("Network: {name} · {detail}")),
        );
    }
    lines.push(if system.btm_installed {
        "Monitor: btm available · simulated handoff".into()
    } else {
        "Monitor: btm unavailable".into()
    });
    paint_lines(ui, area, &lines);
}

fn paint_lines(ui: &mut Ui<'_>, area: Rect, lines: &[String]) {
    let style = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted)));
    for (index, line) in lines.iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        let Some(y) = area.y.checked_add(3).and_then(|y| y.checked_add(offset)) else {
            break;
        };
        if y >= area.bottom() {
            break;
        }
        ui.paint_str(
            Rect::new(area.x.saturating_add(1), y, area.width.saturating_sub(2), 1),
            line,
            style,
        );
    }
}

fn load_value(value: u32) -> String {
    format!("{}.{:02}", value / 100, value % 100)
}

fn rounded_percent(used: u64, total: u64) -> u32 {
    if total == 0 {
        return 0;
    }
    let value = u128::from(used)
        .saturating_mul(100)
        .saturating_add(u128::from(total) / 2)
        .checked_div(u128::from(total))
        .unwrap_or(0);
    u32::try_from(value).unwrap_or(u32::MAX)
}
