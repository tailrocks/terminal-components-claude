pub mod holla;
pub mod interact;
pub mod jackin;
pub mod showcase;
pub mod tablepro;

use std::sync::OnceLock;

use crate::support::{self, Case, Color, Session};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bin {
    Showcase,
    Tablepro,
    Jackin,
    Holla,
}

impl Bin {
    pub const ALL: [Bin; 4] = [Bin::Showcase, Bin::Tablepro, Bin::Jackin, Bin::Holla];

    pub const fn exe_name(self) -> &'static str {
        match self {
            Bin::Showcase => "showcase",
            Bin::Tablepro => "tablepro",
            Bin::Jackin => "jackin-preview",
            Bin::Holla => "holla",
        }
    }

    fn cargo_path(self) -> &'static str {
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|dir| dir.parent())
            .expect("crates/visual-harness is two directories under the workspace");
        let path = workspace.join("target/debug").join(self.exe_name());
        Box::leak(path.to_string_lossy().into_owned().into_boxed_str())
    }

    pub fn path(self) -> &'static str {
        static PATHS: OnceLock<[&'static str; 4]> = OnceLock::new();
        let paths = PATHS.get_or_init(|| {
            Bin::ALL.map(|bin| match std::env::var_os("VISUAL_BIN_DIR") {
                Some(dir) => {
                    let path = std::path::Path::new(&dir).join(bin.exe_name());
                    &*Box::leak(path.to_string_lossy().into_owned().into_boxed_str())
                }
                None => bin.cargo_path(),
            })
        });
        paths[self as usize]
    }
}

#[derive(Clone, Copy)]
pub enum Drive {
    Steps,

    Live(fn(&mut Session, &Case)),

    Resize(fn(&Case, u16, u16)),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Stored,

    Pending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pointer {
    Hover(&'static str),

    Press(&'static str),

    Click(&'static str),

    RightClick(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dismiss {
    Esc,

    Outside,
}

impl Dismiss {
    pub const ALL: [Dismiss; 2] = [Dismiss::Esc, Dismiss::Outside];

    pub const fn suffix(self) -> &'static str {
        match self {
            Dismiss::Esc => "-dismissed-esc",
            Dismiss::Outside => "-dismissed-outside",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Variant {
    pub cols: u16,
    pub rows: u16,
    pub color: Color,
    pub args: &'static [&'static str],
    pub ready: &'static str,
    pub steps: &'static [&'static str],
    pub timeout_ms: u64,
}

impl Variant {
    pub const fn new(
        cols: u16,
        rows: u16,
        color: Color,
        args: &'static [&'static str],
        ready: &'static str,
    ) -> Self {
        Self {
            cols,
            rows,
            color,
            args,
            ready,
            steps: &[],
            timeout_ms: support::TIMEOUT_MS,
        }
    }

    pub const fn sends(mut self, steps: &'static [&'static str]) -> Self {
        self.steps = steps;
        self
    }

    pub const fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }
}

#[derive(Clone, Copy)]
pub struct Root {
    pub root: &'static str,
    pub bin: Bin,

    pub args: &'static [&'static str],

    pub ready: &'static str,

    pub steps: &'static [&'static str],
    pub timeout_ms: u64,
    pub variants: &'static [Variant],

    pub compact: Option<(u16, &'static [&'static str])>,
    pub drive: Drive,

    pub from: Option<(u16, u16)>,
    pub status: Status,

    pub pointer: Option<Pointer>,

    pub dismiss: Option<Status>,

    pub min_size: (u16, u16),

    pub env: &'static [(&'static str, &'static str)],
}

impl Root {
    pub const fn new(
        root: &'static str,
        bin: Bin,
        args: &'static [&'static str],
        ready: &'static str,
    ) -> Self {
        Self {
            root,
            bin,
            args,
            ready,
            steps: &[],
            timeout_ms: support::TIMEOUT_MS,
            variants: &[],
            compact: None,
            drive: Drive::Steps,
            from: None,
            status: Status::Stored,
            pointer: None,
            dismiss: None,
            min_size: (0, 0),
            env: &[],
        }
    }

    pub const fn sends(mut self, steps: &'static [&'static str]) -> Self {
        self.steps = steps;
        self
    }

    pub const fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    pub const fn variants(mut self, variants: &'static [Variant]) -> Self {
        self.variants = variants;
        self
    }

    pub const fn live(mut self, interact: fn(&mut Session, &Case)) -> Self {
        self.drive = Drive::Live(interact);
        self
    }

    pub const fn compact(mut self, max_cols: u16, steps: &'static [&'static str]) -> Self {
        self.compact = Some((max_cols, steps));
        self
    }

    pub const fn resize(mut self, from: (u16, u16), capture: fn(&Case, u16, u16)) -> Self {
        self.from = Some(from);
        self.drive = Drive::Resize(capture);
        self
    }

    pub const fn pending(mut self) -> Self {
        self.status = Status::Pending;
        self
    }

    pub const fn hover(mut self, at: &'static str) -> Self {
        self.pointer = Some(Pointer::Hover(at));
        self
    }

    pub const fn press(mut self, at: &'static str) -> Self {
        self.pointer = Some(Pointer::Press(at));
        self
    }

    pub const fn click(mut self, at: &'static str) -> Self {
        self.pointer = Some(Pointer::Click(at));
        self
    }

    pub const fn right_click(mut self, at: &'static str) -> Self {
        self.pointer = Some(Pointer::RightClick(at));
        self
    }

    pub const fn min_size(mut self, cols: u16, rows: u16) -> Self {
        self.min_size = (cols, rows);
        self
    }

    pub const fn env(mut self, env: &'static [(&'static str, &'static str)]) -> Self {
        self.env = env;
        self
    }

    pub const fn dismiss_pending(mut self) -> Self {
        self.dismiss = Some(Status::Pending);
        self
    }

    pub fn case(&self, cols: u16, rows: u16, color: Color) -> Case {
        let name = format!("{}/{cols}x{rows}/{}", self.root, color.suffix());
        let mut case = match self
            .variants
            .iter()
            .find(|v| v.cols == cols && v.rows == rows && v.color == color)
        {
            Some(v) => Case {
                name: name.into(),
                bin: self.bin.path(),
                args: v.args,
                cols,
                rows,
                color,
                needle: v.ready,
                sends: v.steps,
                timeout_ms: v.timeout_ms,
                env: self.env,
            },
            None => Case {
                name: name.into(),
                bin: self.bin.path(),
                args: self.args,
                cols,
                rows,
                color,
                needle: self.ready,
                sends: match self.compact {
                    Some((max_cols, steps)) if cols <= max_cols => steps,
                    _ => self.steps,
                },
                timeout_ms: self.timeout_ms,
                env: self.env,
            },
        };
        if let Some((from_cols, from_rows)) = self.from {
            case.cols = from_cols;
            case.rows = from_rows;
        }
        case
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Surface,
    Element,
    Overlay,
    Journey,
    Resize,
}

impl Kind {
    pub const fn table(self) -> &'static str {
        match self {
            Kind::Surface => "SURFACES",
            Kind::Element => "ELEMENTS",
            Kind::Overlay => "OVERLAYS",
            Kind::Journey => "JOURNEYS",
            Kind::Resize => "RESIZES",
        }
    }
}

pub struct Tier {
    pub name: &'static str,
    pub sizes: &'static [(u16, u16)],
    pub colors: &'static [Color],

    pub kinds: &'static [Kind],

    pub pending: bool,
}

pub const ALL_KINDS: &[Kind] = &[
    Kind::Surface,
    Kind::Element,
    Kind::Overlay,
    Kind::Journey,
    Kind::Resize,
];

pub const MATRIX: &[Tier] = &[
    Tier {
        name: "full",
        sizes: &support::CANONICAL_SIZES,
        colors: &support::CANONICAL_COLORS,
        kinds: ALL_KINDS,
        pending: false,
    },
    Tier {
        name: "narrow",
        sizes: &[(40, 12), (60, 18)],
        colors: &[Color::Truecolor, Color::NoColorEnv],
        kinds: &[Kind::Surface, Kind::Overlay],
        pending: true,
    },
];

pub const SMOKE: &[((u16, u16), Color)] =
    &[((120, 40), Color::Truecolor), ((72, 20), Color::NoColorEnv)];

#[derive(Clone)]
pub struct Scenario {
    pub name: String,
    pub kind: Kind,
    pub tier: &'static str,
    pub root: &'static Root,
    pub cols: u16,
    pub rows: u16,
    pub color: Color,

    pub pending: bool,

    pub dismiss: Option<Dismiss>,
}

impl Scenario {
    pub fn test_name(&self) -> String {
        self.name.replace('/', "::")
    }

    pub fn run(&self) {
        support::clear_actual(&self.name);
        if !support::capture_enabled(&self.name) {
            return;
        }
        if let Drive::Resize(capture) = self.root.drive {
            let mut case = self.root.case(self.cols, self.rows, self.color);
            case.name = self.name.clone().into();
            capture(&case, self.cols, self.rows);
            return;
        }
        let mut session = self.drive_session(&self.name);
        support::settle_and_gate(&mut session, &self.name);
    }

    fn drive_session(&self, as_name: &str) -> Session {
        let mut case = self.root.case(self.cols, self.rows, self.color);
        case.name = as_name.to_string().into();
        let mut session = support::spawn_boot(&case);
        if let Drive::Live(interact) = self.root.drive {
            interact(&mut session, &case);
        }
        if let Some(pointer) = self.root.pointer {
            interact::pointer(&mut session, pointer);
        }
        if let Some(dismiss) = self.dismiss {
            interact::dismiss(&mut session, dismiss, self.cols, self.rows);
        }
        session
    }

    pub fn capture(&self, as_name: &str) -> Result<tuiscotti::Frame, String> {
        if let Drive::Resize(_) = self.root.drive {
            return Err(format!("`{}` is a resize row", self.name));
        }
        self.drive_session(as_name).wait_stable(support::SETTLE)
    }
}

pub fn tables() -> Vec<(&'static str, Kind, &'static [Root])> {
    let mut out = Vec::new();
    for (app, surfaces, elements, overlays, journeys, resizes) in [
        (
            "showcase",
            showcase::SURFACES,
            showcase::ELEMENTS,
            showcase::OVERLAYS,
            showcase::JOURNEYS,
            showcase::RESIZES,
        ),
        (
            "tablepro",
            tablepro::SURFACES,
            tablepro::ELEMENTS,
            tablepro::OVERLAYS,
            tablepro::JOURNEYS,
            tablepro::RESIZES,
        ),
        (
            "jackin",
            jackin::SURFACES,
            jackin::ELEMENTS,
            jackin::OVERLAYS,
            jackin::JOURNEYS,
            jackin::RESIZES,
        ),
        (
            "holla",
            holla::SURFACES,
            holla::ELEMENTS,
            holla::OVERLAYS,
            holla::JOURNEYS,
            holla::RESIZES,
        ),
    ] {
        out.push((app, Kind::Surface, surfaces));
        out.push((app, Kind::Element, elements));
        out.push((app, Kind::Overlay, overlays));
        out.push((app, Kind::Journey, journeys));
        out.push((app, Kind::Resize, resizes));
    }
    out
}

pub fn all() -> Vec<Scenario> {
    let mut out = Vec::new();
    for (_, kind, table) in tables() {
        for root in table {
            for tier in MATRIX {
                if !tier.kinds.contains(&kind) {
                    continue;
                }
                let mut phases = vec![(None, root.status)];
                if let Some(status) = root.dismiss {
                    phases.extend(Dismiss::ALL.map(|d| (Some(d), status)));
                }
                for &(dismiss, status) in &phases {
                    let suffix = dismiss.map_or("", Dismiss::suffix);
                    for &(cols, rows) in tier.sizes {
                        if cols < root.min_size.0 || rows < root.min_size.1 {
                            continue;
                        }
                        for &color in tier.colors {
                            out.push(Scenario {
                                name: format!(
                                    "{}{suffix}/{cols}x{rows}/{}",
                                    root.root,
                                    color.suffix()
                                ),
                                kind,
                                tier: tier.name,
                                root,
                                cols,
                                rows,
                                color,
                                pending: tier.pending || status == Status::Pending,
                                dismiss,
                            });
                        }
                    }
                }
            }
        }
    }
    out
}

pub fn check_tables() -> Result<(), String> {
    let mut problems = Vec::new();
    let mut roots = std::collections::BTreeSet::new();
    for (app, kind, table) in tables() {
        for row in table {
            if !roots.insert(row.root) {
                problems.push(format!("duplicate root `{}`", row.root));
            }
            if !row.root.starts_with(&format!("{app}/")) || row.root.matches('/').count() < 2 {
                problems.push(format!(
                    "{app}::{} row `{}` must be `{app}/<group>/<leaf>…`",
                    kind.table(),
                    row.root
                ));
            }
            if row.dismiss.is_some() && kind != Kind::Overlay {
                problems.push(format!("`{}`: only OVERLAYS rows are dismissed", row.root));
            }
            if kind == Kind::Resize && row.pointer.is_some() {
                problems.push(format!(
                    "`{}`: resize rows drive their own capture",
                    row.root
                ));
            }
            if let Some(Pointer::Press(_)) = row.pointer
                && !row.args.windows(2).any(|w| w == ["--motion", "paused"])
            {
                problems.push(format!(
                    "`{}`: a held press needs `--motion paused` so no tick clears it",
                    row.root
                ));
            }
            if (kind == Kind::Resize) != row.from.is_some() {
                problems.push(format!(
                    "`{}`: resize rows (and only they) carry a start geometry",
                    row.root
                ));
            }
            if row.min_size != (0, 0) && row.status != Status::Pending {
                problems.push(format!(
                    "`{}`: only pending rows may drop combos (min_size); a stored row keeps every size",
                    row.root
                ));
            }
            let mut combos = std::collections::BTreeSet::new();
            for v in row.variants {
                if !support::CANONICAL_SIZES.contains(&(v.cols, v.rows)) {
                    problems.push(format!(
                        "`{}` variant {}x{} outside the canonical sizes",
                        row.root, v.cols, v.rows
                    ));
                }
                if !combos.insert((v.cols, v.rows, v.color.suffix())) {
                    problems.push(format!(
                        "`{}` has two variants for {}x{}/{}",
                        row.root,
                        v.cols,
                        v.rows,
                        v.color.suffix()
                    ));
                }
            }
        }
    }
    for &(size, color) in SMOKE {
        if !MATRIX
            .iter()
            .any(|t| !t.pending && t.sizes.contains(&size) && t.colors.contains(&color))
        {
            problems.push(format!(
                "smoke combo {}x{}/{} is not in a gating tier",
                size.0,
                size.1,
                color.suffix()
            ));
        }
    }
    for tier in MATRIX {
        if tier.sizes.is_empty() || tier.colors.is_empty() || tier.kinds.is_empty() {
            problems.push(format!("tier `{}` expands to nothing", tier.name));
        }
    }
    let mut names = std::collections::BTreeSet::new();
    for scenario in all() {
        if let Err(e) = tuiscotti::grouped::validate_name(&scenario.name) {
            problems.push(e.to_string());
        }
        if !names.insert(scenario.name.clone()) {
            problems.push(format!("duplicate scenario `{}`", scenario.name));
        }

        let case = scenario
            .root
            .case(scenario.cols, scenario.rows, scenario.color);
        for step in case.sends {
            if let Some(ms) = step.strip_prefix("advance:") {
                if !ms.parse::<u64>().is_ok_and(|ms| ms > 0) {
                    problems.push(format!(
                        "`{}` step `{step}`: advance:<ms> takes a positive integer",
                        scenario.name
                    ));
                }
                continue;
            }
            if ["sleep:", "wait:", "type:"]
                .iter()
                .any(|p| step.starts_with(p))
            {
                continue;
            }
            if let Err(e) = tuiscotti::tui::parse_chord(step) {
                problems.push(format!("`{}` step `{step}`: {e}", scenario.name));
            }
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}
