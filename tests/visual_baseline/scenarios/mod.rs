//! Scenario definitions as typed state tables (docs/visual-testing/plan.md
//! §B2). Each app module declares `const` tables of [`Root`] rows:
//!
//! | Table | Row | Expands to |
//! |---|---|---|
//! | `SURFACES` | one reachable screen (boot + keyboard steps) | `<root>/<cols>x<rows>/<color>` |
//! | `ELEMENTS` | an interactive element in one state (hover, pressed, disabled, selected+hover) | same |
//! | `OVERLAYS` | a popup, menu, dialog or picker, open or dismissed | same |
//! | `JOURNEYS` | one checkpoint of an ordered flow (keys, wheel, drag) | same |
//! | `RESIZES` | a surface resized mid-session from a start geometry | same (the combo size is the target) |
//!
//! [`MATRIX`] turns every row into one scenario per tier combo; [`all`]
//! is the single inventory the test harness, `store_integrity` and
//! `cargo xtask visual list` read. One nextest test per scenario, named
//! `<app>::<group>::<leaf>…::<cols>x<rows>::<color>` ([`Scenario::test_name`]).
//!
//! Existing scenarios keep their names; rows marked [`Root::pending`], and
//! every narrow-tier combo, are new coverage with no approval yet. They are
//! listed and runnable (`VISUAL_FULL=1`) but do not gate and need no store
//! file until re-approved.

pub mod holla;
pub mod interact;
pub mod jackin;
pub mod showcase;
pub mod tablepro;

use std::sync::OnceLock;

use crate::support::{self, Case, Color, Session};

/// The four binaries under test. The path is `VISUAL_BIN_DIR/<name>` when
/// that variable is set (plan §B1: the same harness on `main`, where the
/// binaries live in other crates), else the `CARGO_BIN_EXE_<name>` Cargo
/// built for this package.
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

    const fn cargo_path(self) -> &'static str {
        match self {
            Bin::Showcase => env!("CARGO_BIN_EXE_showcase"),
            Bin::Tablepro => env!("CARGO_BIN_EXE_tablepro"),
            Bin::Jackin => env!("CARGO_BIN_EXE_jackin-preview"),
            Bin::Holla => env!("CARGO_BIN_EXE_holla"),
        }
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

/// How a scenario is driven after boot + steps.
#[derive(Clone, Copy)]
pub enum Drive {
    /// Boot, steps, settle, gate.
    Steps,
    /// Boot and steps, then `interact` on the live session, settle, gate.
    Live(fn(&mut Session, &Case)),
    /// Full control over one resize capture: the case carries the start
    /// geometry, `(cols, rows)` is the target.
    Resize(fn(&Case, u16, u16)),
}

/// Approval status of a row's canonical combos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// The store holds an approval (legacy tuisnap or re-approved).
    Stored,
    /// New coverage: no approval yet, not gating until re-approved.
    Pending,
}

/// A pointer step after boot, steps and any live interaction. The target is
/// the first on-screen occurrence of the text, resolved at capture time;
/// the pointer lands two cells into it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pointer {
    /// Button-less motion: the hover state.
    Hover(&'static str),
    /// Primary button down with no release: the pressed state, held. Rows
    /// use it with `--motion paused` so no tick can clear it.
    Press(&'static str),
    /// Primary click (down + up), e.g. to open a menu.
    Click(&'static str),
    /// Secondary click: context menus.
    RightClick(&'static str),
}

/// How an overlay is dismissed for its `-dismissed-*` scenarios.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dismiss {
    /// `Esc`.
    Esc,
    /// A primary click outside the overlay, on the bottom-right cell.
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

/// Exact legacy declaration for one combo; it wins over the row's own
/// settings at that combo (old `baseline_case_with_variants!`).
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

/// One scenario root: everything but the size and colour.
#[derive(Clone, Copy)]
pub struct Root {
    /// `<app>/<group>/<leaf>[/<state>]`.
    pub root: &'static str,
    pub bin: Bin,
    /// argv after the binary, before `--color`.
    pub args: &'static [&'static str],
    /// Boot needle on the first full frame (`""`: non-blank + quiet).
    pub ready: &'static str,
    /// DSL steps after boot: key names, `type:<text>`, `sleep:<ms>`,
    /// `wait:<needle>`.
    pub steps: &'static [&'static str],
    pub timeout_ms: u64,
    pub variants: &'static [Variant],
    /// `(max_cols, steps)`: steps used instead at widths `<= max_cols`.
    pub compact: Option<(u16, &'static [&'static str])>,
    pub drive: Drive,
    /// Resize rows: the PTY geometry before the resize.
    pub from: Option<(u16, u16)>,
    pub status: Status,
    /// Pointer step after the steps and any live interaction.
    pub pointer: Option<Pointer>,
    /// Overlay rows: also capture `-dismissed-esc` and `-dismissed-outside`
    /// (both start from this row's open state), with this status.
    pub dismiss: Option<Status>,
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

    /// New coverage without an approval: listed and runnable, not gating.
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

    /// Also capture both dismissals of this overlay, pending approval.
    pub const fn dismiss_pending(mut self) -> Self {
        self.dismiss = Some(Status::Pending);
        self
    }

    /// The concrete capture for one combo: an exact [`Variant`] wins, then
    /// the compact steps at narrow widths, then the row itself. Resize rows
    /// start at [`Root::from`].
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
            },
        };
        if let Some((from_cols, from_rows)) = self.from {
            case.cols = from_cols;
            case.rows = from_rows;
        }
        case
    }
}

/// Which table a row came from.
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

/// Capture tiers (plan §C1). Every tier combo is one scenario; the tier is
/// selected with a nextest filter on the name suffix (`.config/nextest.toml`
/// profiles `smoke`, `full`, `narrow`).
pub struct Tier {
    pub name: &'static str,
    pub sizes: &'static [(u16, u16)],
    pub colors: &'static [Color],
    /// Which tables the tier expands.
    pub kinds: &'static [Kind],
    /// Combos only in this tier have no approval yet.
    pub pending: bool,
}

pub const ALL_KINDS: &[Kind] = &[
    Kind::Surface,
    Kind::Element,
    Kind::Overlay,
    Kind::Journey,
    Kind::Resize,
];

/// The capture matrix. `full` is the canonical 5 sizes × 5 colours every
/// existing scenario carries; `smoke` (`120x40/truecolor` +
/// `72x20/nocolor`) is a subset of it, so it needs no extra rows. `narrow`
/// adds 40x12 and 60x18 for surfaces and overlays, pending approval.
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

/// The smoke tier (every PR): two combos of the full tier, selected by the
/// nextest `smoke` profile. Kept here so `scenario_tables` can check that
/// every smoke combo exists in [`MATRIX`].
pub const SMOKE: &[((u16, u16), Color)] =
    &[((120, 40), Color::Truecolor), ((72, 20), Color::NoColorEnv)];

/// One generated scenario: a root at one size and colour.
#[derive(Clone)]
pub struct Scenario {
    /// Store name: `<root>/<cols>x<rows>/<color>`.
    pub name: String,
    pub kind: Kind,
    pub tier: &'static str,
    pub root: &'static Root,
    pub cols: u16,
    pub rows: u16,
    pub color: Color,
    /// No approval is expected in the store (pending row or pending tier).
    pub pending: bool,
    /// Overlay dismissal captured after the open state, if any.
    pub dismiss: Option<Dismiss>,
}

impl Scenario {
    /// nextest test name: the store name with `/` as `::`.
    pub fn test_name(&self) -> String {
        self.name.replace('/', "::")
    }

    /// Run this capture through the gate. A scenario without a tuiscotti
    /// approval is skipped unless `VISUAL_FULL=1` ([`support::capture_enabled`]).
    pub fn run(&self) {
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

    /// Boot, steps, live interaction, pointer and dismissal on a session
    /// spawned as `as_name` (which picks its isolated `HOME`).
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

    /// One settled capture without a gate, spawned as `as_name`. Resize rows
    /// drive their own capture and are not supported.
    pub fn capture(&self, as_name: &str) -> Result<tuiscotti::Frame, String> {
        if let Drive::Resize(_) = self.root.drive {
            return Err(format!("`{}` is a resize row", self.name));
        }
        self.drive_session(as_name).wait_stable(support::SETTLE)
    }
}

/// `(app, kind, table)` for every table of every app.
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

/// Every generated scenario, in table order.
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

/// Table invariants: unique roots, a valid `<app>/<group>/…` shape whose
/// app matches the table's module, variants inside the full matrix with at
/// most one declaration per combo.
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
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}
