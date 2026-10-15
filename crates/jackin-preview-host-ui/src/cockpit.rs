//! Launch cockpit state and controls.

use ratatui::layout::Rect;
use termrock::author::{FgStep, Modifier, Role, StylePatch, Ui};
use termrock::controls::{Brand, Button, Panel, PanelKind};
use termrock::layout::Track;
use termrock::navigation::{List, ListState};
use termrock::{
    Hint, HintKey, HintLayer, Id, ItemKey, Part, PropsList, PropsRow, PropsState, truncate, width,
};

use jackin_preview_presentation::rain::{HANDOFF_LEN, HandoffStage, handoff_stage};
use jackin_preview_sim::launch::{LaunchFailure, LaunchRun, Stage};
use jackin_preview_sim::world::World;

/// Cockpit root.
pub const ROOT: Id = Id::root("jackin.cockpit");
/// Stage list.
pub const STAGES: Id = ROOT.sub("stages");
/// Safe account/credential projection below the stage rail.
pub const ACCOUNT_LINE: Id = ROOT.sub("account-line");
/// Build log viewport.
pub const LOG: Id = ROOT.sub("log");
/// Cancel action.
pub const CANCEL: Id = ROOT.sub("cancel");
/// Retry action.
pub const RETRY: Id = ROOT.sub("retry");
/// Cockpit-to-Capsule transition surface.
pub const HANDOFF: Id = ROOT.sub("handoff");
/// Debug info props list.
pub const INFO_LAYER: Id = ROOT.sub("info");
pub const INFO_PROPS: Id = ROOT.sub("info-props");
/// Debug info close action.
pub const INFO_CLOSE: Id = ROOT.sub("info-close");

/// Safe account labels projected below a launch rail.
///
/// The cockpit receives display labels only.  It never stores credential
/// material or provider responses; the primary label may include its safe
/// source annotation while the remaining labels are plain account titles.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountLine {
    primary: Option<String>,
    additional: Vec<String>,
}

impl AccountLine {
    /// Build a line from the primary account and the other effective accounts.
    pub fn new<I, S>(primary: Option<S>, additional: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut line = Self {
            primary: None,
            additional: Vec::new(),
        };
        if let Some(primary) = primary {
            line.push_primary(primary);
        }
        for label in additional {
            line.push_additional(label);
        }
        line
    }

    /// Build a line from labels in display order.
    pub fn from_labels<I, S>(labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut labels = labels.into_iter();
        let primary = labels.next();
        Self::new(primary, labels)
    }

    /// Number of unique account labels in this line.
    pub fn len(&self) -> usize {
        usize::from(self.primary.is_some()) + self.additional.len()
    }

    /// Whether no account label is available.
    pub const fn is_empty(&self) -> bool {
        self.primary.is_none() && self.additional.is_empty()
    }

    /// Primary account label, including its optional safe source annotation.
    pub fn primary(&self) -> Option<&str> {
        self.primary.as_deref()
    }

    /// Additional effective-account labels in stable display order.
    pub fn additional(&self) -> impl Iterator<Item = &str> {
        self.additional.iter().map(String::as_str)
    }

    /// Render the account line without credential material.
    pub fn text(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }
        let count = self.len();
        let noun = if count == 1 { "account" } else { "accounts" };
        let labels = self
            .primary
            .iter()
            .chain(self.additional.iter())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" · ");
        Some(format!("{count} {noun} · {labels}"))
    }

    fn push_primary<S: Into<String>>(&mut self, label: S) {
        let label = label.into();
        if !label.is_empty() {
            self.primary = Some(label);
        }
    }

    fn push_additional<S: Into<String>>(&mut self, label: S) {
        let label = label.into();
        if !label.is_empty()
            && self.primary.as_deref() != Some(label.as_str())
            && !self.additional.iter().any(|existing| existing == &label)
        {
            self.additional.push(label);
        }
    }
}

/// Tick-driven cockpit-to-Capsule handoff.
///
/// Frame zero is the first visible cockpit-dim frame.  A caller advances this
/// state only for a product tick; bootstrap, input, and repaint passes must not
/// consume a handoff frame.  Completion is reported on the tick after frame
/// `HANDOFF_LEN - 1` was visible.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HandoffState {
    frame: u64,
    active: bool,
}

impl HandoffState {
    /// Start at the first handoff frame without consuming a tick.
    pub const fn start(&mut self) {
        self.frame = 0;
        self.active = true;
    }

    /// Current handoff frame.  Frame zero is the first visible frame.
    pub const fn frame(self) -> u64 {
        self.frame
    }

    /// Whether the transition is currently active.
    pub const fn is_active(self) -> bool {
        self.active
    }

    /// Whether the transition has consumed all product ticks.
    pub const fn is_complete(self) -> bool {
        !self.active && self.frame >= HANDOFF_LEN
    }

    /// Current cross-fade phase for painting.
    pub const fn stage(self) -> HandoffStage {
        if self.active || self.frame > 0 {
            handoff_stage(self.frame)
        } else {
            HandoffStage::Capsule
        }
    }

    /// Consume one product tick and report whether the route may switch.
    pub const fn advance(&mut self) -> bool {
        if !self.active {
            return false;
        }
        self.frame = self.frame.saturating_add(1);
        if self.frame >= HANDOFF_LEN {
            self.active = false;
            return true;
        }
        false
    }
}

/// Cockpit interaction state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CockpitState {
    /// Whether the launch log is visible.
    pub log_open: bool,
    /// Current launch-log scroll offset.
    pub log_scroll: u16,
    /// Safe account labels shown beneath the stage rail.
    pub account_line: AccountLine,
    /// Tick-owned handoff state.
    pub handoff: HandoffState,
}

/// Stage rail display row.
struct StageItem {
    glyph: &'static str,
    is_accent: bool,
    num: &'static str,
    name: &'static str,
    status: &'static str,
    is_current: bool,
}

/// Reusable Cockpit screen composition.
pub struct CockpitScreen;

impl CockpitScreen {
    fn paint_rain(
        ui: &mut Ui<'_>,
        area: Rect,
        exclude: &[Rect],
        t_local: u64,
        running: bool,
        frozen: bool,
    ) {
        jackin_preview_presentation::rain::paint_bound(ui, area, exclude, t_local, running, frozen);
    }

    /// One truncated label on the canvas. The list is only as wide as the
    /// text, so cells past the ellipsis keep the row cleared behind it.
    fn paint_truncated(ui: &mut Ui<'_>, id: Id, x: u16, y: u16, text: &str, fg: Role) {
        let w = width(text) as u16;
        if w == 0 {
            return;
        }
        const CANVAS_ROW: [(Part, StylePatch); 1] = [(
            Part::CONTAINER,
            StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(termrock::Surface::Canvas)),
        )];
        let state = ListState::default();
        let patch = StylePatch::new().set_fg(fg);
        List::new(id)
            .bare(true)
            .patch_part(&CANVAS_ROW)
            .row(|_: &(), row| {
                row.label_patched(text, &patch);
            })
            .draw(ui, Rect::new(x, y, w, 1), &state, &[()]);
    }

    /// One centered identity line. Bare so the string starts at the centered
    /// column; a marker gutter would shift it three cells right.
    fn paint_centered_line(
        ui: &mut Ui<'_>,
        area: Rect,
        id: Id,
        y: u16,
        text: &str,
        patch: &StylePatch,
    ) {
        let n = width(text) as u16;
        let columns = area.width;
        if n == 0 || columns == 0 {
            return;
        }
        // Wider than the viewport: the tag truncates with an ellipsis at the
        // left edge instead of letting the screen clip the last glyph.
        let shown;
        let (x, w) = if n > columns {
            shown = truncate(text, columns);
            (area.x, columns)
        } else {
            shown = text.to_owned();
            (area.x + area.width.saturating_sub(n) / 2, n)
        };
        let state = ListState::default();
        List::new(id)
            .bare(true)
            .row(|_: &(), row| {
                row.label_patched(&shown, patch);
            })
            .draw(ui, Rect::new(x, y, w.max(1), 1), &state, &[()]);
    }

    /// Render the main Cockpit surface.
    pub fn draw(
        ui: &mut Ui<'_>,
        area: Rect,
        _cockpit: &CockpitState,
        world: &World,
        role: &str,
        debug: bool,
        run: Option<&LaunchRun>,
    ) {
        let ws_name = world
            .workspaces
            .first()
            .map_or("payments-platform", |w| w.name.as_str());
        // Display the role's short name. The stored key is `namespace/name`.
        let role = role.rsplit('/').next().unwrap_or(role);

        // Tag `cockpit.rs` centers the identity block on the live width and
        // paints the atmosphere behind it. The rail is 44 columns, clamped
        // when the viewport cannot hold that plus the side margin.
        let rail_w = 44u16.min(area.width.saturating_sub(4));
        let rail_x = area.x + area.width.saturating_sub(rail_w) / 2;
        let ident_y = area.y.saturating_add(1);
        let rail_h = 11u16.min(area.height.saturating_sub(9));
        let rail_y = ident_y.saturating_add(5);
        let tick = run.map(|r| r.tick).unwrap_or(0);
        let running = run.is_some_and(|r| !r.is_terminal());
        let frozen = run.is_some_and(|r| r.failure.is_some() || r.cancelled);
        let exclude = [
            Rect::new(
                rail_x.saturating_sub(2),
                ident_y,
                rail_w.saturating_add(4),
                rail_h.saturating_add(6),
            ),
            Rect::new(area.x, area.bottom().saturating_sub(3), area.width, 3),
        ];
        // Paint through `paint_str` so the page keeps the roles `dim_layer`
        // walks. `Ui::raw` would drop those roles and the modal would restyle
        // the whole frame as backdrop text.
        Self::paint_rain(ui, area, &exclude, tick, running, frozen);

        // 1. Centered jackin❯ brand. The wordmark rect is the 9-cell
        // " jackin❯ " run the tag centers; the glyph sits one cell in.
        let brand_w = 9u16;
        let brand_x = area.x + area.width.saturating_sub(brand_w) / 2;
        let _ = Brand::new(ROOT.sub("brand"), "jackin❯")
            .draw(ui, Rect::new(brand_x, ident_y, brand_w, 1));

        // 2–4. Identity lines. Bare lists: the tag centers the string itself,
        // with no marker gutter.
        let head = format!("Loading {role} into workspace {ws_name}");
        let title_patch = StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .add(Modifier::BOLD);
        Self::paint_centered_line(
            ui,
            area,
            ROOT.sub("ident-title"),
            ident_y.saturating_add(1),
            &head,
            &title_patch,
        );
        let agent_patch = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
        Self::paint_centered_line(
            ui,
            area,
            ROOT.sub("ident-agent"),
            ident_y.saturating_add(2),
            "Claude Code · Anthropic / Claude · account Claude · Work (session choice)",
            &agent_patch,
        );
        let stage_patch = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
        Self::paint_centered_line(
            ui,
            area,
            ROOT.sub("ident-stage"),
            ident_y.saturating_add(3),
            "stage 3 of 11 · Credentials · 2 done · 0 skipped",
            &stage_patch,
        );

        // 5. 11-stage launch rail
        let stages = [
            StageItem {
                glyph: "✓",
                is_accent: true,
                num: "01",
                name: "Identity",
                status: "1.4 s",
                is_current: false,
            },
            StageItem {
                glyph: "✓",
                is_accent: true,
                num: "02",
                name: "Role",
                status: "1.8 s",
                is_current: false,
            },
            StageItem {
                glyph: "⠋",
                is_accent: true,
                num: "03",
                name: "Credentials",
                status: "",
                is_current: true,
            },
            StageItem {
                glyph: "",
                is_accent: false,
                num: "04",
                name: "Construct",
                status: "queued",
                is_current: false,
            },
            StageItem {
                glyph: "",
                is_accent: false,
                num: "05",
                name: "Agent Binaries",
                status: "queued",
                is_current: false,
            },
            StageItem {
                glyph: "",
                is_accent: false,
                num: "06",
                name: "Derived Image",
                status: "queued",
                is_current: false,
            },
            StageItem {
                glyph: "",
                is_accent: false,
                num: "07",
                name: "Workspace",
                status: "queued",
                is_current: false,
            },
            StageItem {
                glyph: "",
                is_accent: false,
                num: "08",
                name: "Network",
                status: "queued",
                is_current: false,
            },
            StageItem {
                glyph: "",
                is_accent: false,
                num: "09",
                name: "Sidecar",
                status: "queued",
                is_current: false,
            },
            StageItem {
                glyph: "",
                is_accent: false,
                num: "10",
                name: "Capsule",
                status: "queued",
                is_current: false,
            },
            StageItem {
                glyph: "",
                is_accent: false,
                num: "11",
                name: "Hardline",
                status: "queued",
                is_current: false,
            },
        ];

        let rail_state = ListState::default();
        // Tag `StepRail` right-aligns the meta to `row.right() - 1`, and the
        // scrollbar (when the 11 rows exceed the viewport) narrows that row.
        // The list is bare and exactly `rail_w` wide so that scrollbar lands
        // on the rail's last column. A wider rect pushed it one cell past
        // the rail, and the modal kept that swapped edge.
        let has_sb = stages.len() > usize::from(rail_h);
        let row_w = rail_w - u16::from(has_sb);
        List::new(STAGES)
            .bare(true)
            .row(|item: &StageItem, row| {
                // Prefix is 6 columns (pad, glyph, number). The name track
                // stops where the meta begins, so the pad before a short
                // status stays the row fill.
                let mw = width(item.status) as u16;
                let name_w = if mw == 0 {
                    row_w.saturating_sub(6)
                } else {
                    row_w.saturating_sub(mw).saturating_sub(7)
                };
                let mut cols = row.columns_with_gap(
                    &[
                        Track::Fixed(1),
                        Track::Fixed(2),
                        Track::Fixed(3),
                        Track::Fixed(name_w),
                        Track::Fixed(mw),
                    ],
                    0,
                );
                let pad = StylePatch::new().set_fg(Role::CurrentSurface);
                cols.cell(0).patch(&pad).text(" ");
                if !item.glyph.is_empty() {
                    let p_accent = StylePatch::new().set_fg(Role::Accent);
                    cols.cell(1).patch(&p_accent).text(item.glyph);
                }

                if item.is_current {
                    let p_sec = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    cols.cell(2).patch(&p_sec).text(item.num);
                    let p_bold = StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Primary))
                        .add(Modifier::BOLD);
                    cols.cell(3).patch(&p_bold).text(item.name);
                } else if item.is_accent {
                    let p_faint = StylePatch::new().set_fg(Role::BorderStrong);
                    cols.cell(2).patch(&p_faint).text(item.num);
                    let p_sec = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    cols.cell(3).patch(&p_sec).text(item.name);
                    if !item.status.is_empty() {
                        cols.cell(4).patch(&p_faint).text(item.status);
                    }
                } else {
                    let p_faint = StylePatch::new().set_fg(Role::BorderStrong);
                    cols.cell(2).patch(&p_faint).text(item.num);
                    let p_muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                    cols.cell(3).patch(&p_muted).text(item.name);
                    if !item.status.is_empty() {
                        cols.cell(4).patch(&p_faint).text(item.status);
                    }
                }
            })
            .draw(
                ui,
                Rect::new(rail_x, rail_y, rail_w, rail_h),
                &rail_state,
                &stages,
            );

        // Bottom chrome row: two rows above the body bottom (tag `ay`).
        let chrome_y = area.bottom().saturating_sub(2);

        // 6. Credentials projection under the rail. The tag fills both rows
        // from the canvas first (so rain cannot show through), then truncates
        // each line to the live width. Short viewports drop the pair rather
        // than collide with the bottom chrome (`y + 1 < bottom - 3`).
        let cred_y = rail_y.saturating_add(rail_h).saturating_add(1);
        if cred_y.saturating_add(1) < area.bottom().saturating_sub(3)
            && let Some((line, detail)) = credential_projection(world)
        {
            let x = rail_x.saturating_sub(4);
            let wdt = area.right().saturating_sub(x.saturating_add(2));
            // A bare list fills its rect with the canvas before any label,
            // which clears rain on both rows without a preview paint call.
            const CANVAS_ROW: [(Part, StylePatch); 1] = [(
                Part::CONTAINER,
                StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .set_bg(Role::Surface(termrock::Surface::Canvas)),
            )];
            let clear_state = ListState::default();
            List::new(ACCOUNT_LINE.sub("clear"))
                .bare(true)
                .patch_part(&CANVAS_ROW)
                .row(|_: &(), row| {
                    row.label_patched("", &StylePatch::new());
                })
                .draw(
                    ui,
                    Rect::new(area.x, cred_y, area.width, 2),
                    &clear_state,
                    &[()],
                );
            if wdt > 0 {
                let shown = truncate(&line, wdt);
                Self::paint_truncated(ui, ACCOUNT_LINE, x, cred_y, &shown, Role::Fg(FgStep::Muted));
                let detail_w = wdt.saturating_sub(13);
                if detail_w > 0 {
                    let detail_shown = truncate(&detail, detail_w);
                    Self::paint_truncated(
                        ui,
                        ACCOUNT_LINE.sub("quota"),
                        x.saturating_add(13),
                        cred_y.saturating_add(1),
                        &detail_shown,
                        Role::Fg(FgStep::Secondary),
                    );
                }
            }
        }

        // 7. Bottom status line. A terminal failure replaces the live
        // activity row with the frozen failure chrome: the `! {stage}
        // failed` activity, the container chip, and the build-log
        // counter (tag bottom chrome).
        let failure = run.and_then(|run| run.failure.as_ref());
        if let (Some(run), Some(failure)) = (run, failure) {
            // `List` rows reserve a 3-cell marker gutter on the left and
            // one cell on the right; size rects so the chrome fits.
            let activity = format!("! {} failed", failure.stage.label());
            let activity_state = ListState::default();
            List::new(ROOT.sub("status"))
                .row(|_, row| {
                    let p = StylePatch::new().set_fg(Role::Danger);
                    row.label_patched(&activity, &p);
                })
                .draw(
                    ui,
                    Rect::new(
                        area.x.saturating_add(1),
                        chrome_y,
                        width(&activity) as u16 + 6,
                        1,
                    ),
                    &activity_state,
                    &[()],
                );
            let chip = format!(" {} ", run.container);
            let chip_w = width(&chip) as u16;
            let chip_state = ListState::default();
            List::new(ROOT.sub("status-container"))
                .row(|_, row| {
                    let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    row.label_patched(&chip, &p);
                })
                .draw(
                    ui,
                    Rect::new(
                        area.right().saturating_sub(chip_w + 5),
                        chrome_y,
                        chip_w + 4,
                        1,
                    ),
                    &chip_state,
                    &[()],
                );
            if run.build_lines_emitted > 0 {
                let lines = format!("{} lines · b build log", run.build_lines_emitted);
                let log_state = ListState::default();
                List::new(ROOT.sub("status-log"))
                    .row(|_, row| {
                        let p = StylePatch::new().set_fg(Role::Fg(FgStep::Faint));
                        row.label_patched(&lines, &p);
                    })
                    .draw(
                        ui,
                        Rect::new(
                            area.x.saturating_add(1),
                            chrome_y.saturating_add(1),
                            width(&lines) as u16 + 6,
                            1,
                        ),
                        &log_state,
                        &[()],
                    );
            }
        } else {
            let activity = "⠋ Resolving credentials…";
            let status_state = ListState::default();
            List::new(ROOT.sub("status"))
                .bare(true)
                .row(|_, row| {
                    let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    row.label_patched(activity, &p);
                })
                .draw(
                    ui,
                    Rect::new(
                        area.x.saturating_add(1),
                        chrome_y,
                        width(activity) as u16,
                        1,
                    ),
                    &status_state,
                    &[()],
                );
        }

        if debug {
            // Tag paints ` {run_id} ` ending one cell before the right edge,
            // with no marker gutter. The id itself is the `d` wait target.
            let chip = " run-202609030914-b5df ";
            let cw = width(chip) as u16;
            let debug_state = ListState::default();
            List::new(ROOT.sub("debug-chip"))
                .bare(true)
                .row(|_, row| {
                    let p = StylePatch::new().set_fg(Role::Warning);
                    row.label_patched(chip, &p);
                })
                .draw(
                    ui,
                    Rect::new(
                        area.right().saturating_sub(cw.saturating_add(1)),
                        chrome_y,
                        cw,
                        1,
                    ),
                    &debug_state,
                    &[()],
                );
        }
    }

    /// Row data for the debug info props, shared by update and draw so
    /// keys, order, and copyability agree (tag `cockpit.rs:395`).
    pub fn info_rows<'a>(
        role: &'a str,
        target: &'a str,
        container: Option<&'a str>,
        debug: bool,
        telemetry: &'a str,
    ) -> Vec<PropsRow<'a>> {
        let mut rows: Vec<PropsRow<'a>> = Vec::new();
        let mut key = 0u64;
        let mut next = || {
            let current = key;
            key += 1;
            ItemKey::num(current)
        };
        if let Some(container) = container {
            rows.push(PropsRow::new(next(), "Container", container).copyable());
        }
        rows.push(PropsRow::new(next(), "Target", target));
        rows.push(PropsRow::new(next(), "Role", role));
        rows.push(PropsRow::new(
            next(),
            "Agent",
            "Claude Code · account Claude · Work",
        ));
        rows.push(PropsRow::new(next(), "Run id", Self::RUN_ID).copyable());
        rows.push(PropsRow::new(next(), "jackin", "0.6.4 · preview"));
        if debug {
            rows.push(PropsRow::new(next(), "Telemetry", telemetry));
        }
        rows
    }

    /// The fixture run id voiced by the debug chrome.
    const RUN_ID: &'static str = "run-202609030914-b5df";

    /// Render the container debug info modal: the stock [`PropsList`]
    /// owns the cursor, gutter, copy hint, and scrolling; the caller owns
    /// the row data and the [`PropsState`].
    pub fn draw_info(
        ui: &mut Ui<'_>,
        area: Rect,
        world: &World,
        role: &str,
        debug: bool,
        props: &PropsState,
        container: Option<&str>,
    ) {
        let role = role.rsplit('/').next().unwrap_or(role);
        let ws_name = world
            .workspaces
            .first()
            .map_or("payments-platform", |w| w.name.as_str());
        let target_val = format!("{role} into workspace {ws_name}");
        let telemetry_val = format!("run {} -> otlp://collector.internal:4317", Self::RUN_ID);
        let rows = Self::info_rows(
            role,
            target_val.as_str(),
            container,
            debug,
            telemetry_val.as_str(),
        );
        let w = 66.min(area.width.saturating_sub(4));
        let h = 11.min(area.height.saturating_sub(2));
        let modal = Rect::new(
            area.x + area.width.saturating_sub(w) / 2,
            area.y + area.height.saturating_sub(h) / 2,
            w,
            h,
        );

        // The framed token insets by 3. The tag info dialog keeps one pad
        // cell beside the border, then the 2-cell focus gutter.
        // Focus supplies the strong border and the bold title. Clearing the
        // gutter leaves the title rule a plain `─`.
        const INFO_CHROME: [(Part, StylePatch); 2] = [
            (
                Part::CONTAINER,
                StylePatch::new().set_bg(Role::Surface(termrock::Surface::Elevated)),
            ),
            (
                Part::GUTTER,
                StylePatch {
                    glyph: termrock::Slot::Clear,
                    ..StylePatch::new()
                },
            ),
        ];
        // Labels drop the cursor-row bold. Values keep it.
        const INFO_ROWS: [(Part, StylePatch); 2] = [
            (
                Part::CONTAINER,
                StylePatch::new().set_bg(Role::Surface(termrock::Surface::Elevated)),
            ),
            (Part::META, StylePatch::new().remove(Modifier::BOLD)),
        ];
        Panel::new(ROOT.sub("debug-info"))
            .kind(PanelKind::Framed)
            .title("Debug info")
            .meta("read-only")
            .focused(true)
            .patch_part(&INFO_CHROME)
            .inner_inset(termrock::Insets {
                l: 2,
                t: 1,
                r: 1,
                b: 1,
            })
            .draw(ui, modal, |ui, body| {
                // Tag props start one cell left of the 3-column inset and
                // stop two cells before the right border, so the dimmed
                // plane stays visible beside the frame.
                let props_w = body.width.saturating_sub(2);
                PropsList::new(INFO_PROPS).patch_part(&INFO_ROWS).draw(
                    ui,
                    Rect::new(body.x, body.y, props_w, rows.len().min(9) as u16),
                    props,
                    &rows,
                );

                let close_area = Rect::new(
                    body.right().saturating_sub(9),
                    body.bottom().saturating_sub(1),
                    7,
                    1,
                );
                Button::new(INFO_CLOSE, "Close").draw(ui, close_area);
            });
    }

    /// Render the launch-failure modal: the frozen failure summary, its
    /// stage/run-id/next-step/container props, and the deterministic
    /// diagnostic detail. Escape acknowledges; see the app route handler.
    pub fn draw_failure(
        ui: &mut Ui<'_>,
        area: Rect,
        failure: &LaunchFailure,
        run_id: &str,
        role: &str,
        target: &str,
        container: &str,
    ) {
        let w = 70.min(area.width.saturating_sub(4)).max(20);
        let h = 22.min(area.height.saturating_sub(2)).max(5);
        let modal_area = Rect::new(
            area.x + area.width.saturating_sub(w) / 2,
            area.y + area.height.saturating_sub(h) / 2,
            w,
            h,
        );
        let title = match failure.stage {
            Stage::DerivedImage => "Docker build failed",
            Stage::Credentials => "Credential check failed",
            _ => "Launch failed",
        };
        Panel::new(ROOT.sub("failure"))
            .kind(PanelKind::Framed)
            .title(&format!("! {title}"))
            .meta("run id is the only copyable value")
            .draw(ui, modal_area, |ui, body| {
                let loading = format!("Loading {role} {target}");
                let intro = [failure.summary.as_str(), loading.as_str()];
                let intro_state = ListState::default();
                List::new(ROOT.sub("failure-intro"))
                    .row(|line: &&str, row| {
                        let p = StylePatch::new().set_fg(Role::Fg(FgStep::Primary));
                        row.label_patched(line, &p);
                    })
                    .draw(
                        ui,
                        Rect::new(body.x, body.y, body.width, 2.min(body.height)),
                        &intro_state,
                        &intro,
                    );
                let props: [(&str, String, bool); 4] = [
                    ("Stage", failure.stage.label().to_owned(), true),
                    ("Run id", run_id.to_owned(), false),
                    ("Next step", failure.next_step.clone(), false),
                    ("Container", container.to_owned(), false),
                ];
                let value_w = body.width.saturating_sub(13);
                let mut rows: Vec<(String, String, bool)> = Vec::new();
                for (label, value, is_error) in &props {
                    let wrapped = termrock::wrap(value, value_w.max(8));
                    for (i, line) in wrapped.iter().enumerate() {
                        let head = if i == 0 {
                            format!("  {label:<10}")
                        } else {
                            " ".repeat(12)
                        };
                        rows.push((head, line.clone(), *is_error));
                    }
                }
                let props_state = ListState::default();
                let props_y = body.y.saturating_add(3);
                let props_h = (rows.len() as u16).min(body.height.saturating_sub(4));
                List::new(ROOT.sub("failure-props"))
                    .row(|(head, value, is_error): &(String, String, bool), row| {
                        let mut cols = row.columns_with_gap(&[Track::Fixed(12), Track::Flex(1)], 1);
                        let p_label = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                        cols.cell(0).patch(&p_label).text(head);
                        let p_val = if *is_error {
                            StylePatch::new().set_fg(Role::Danger)
                        } else {
                            StylePatch::new().set_fg(Role::Fg(FgStep::Primary))
                        };
                        cols.cell(1).patch(&p_val).text(value);
                    })
                    .draw(
                        ui,
                        Rect::new(body.x, props_y, body.width, props_h),
                        &props_state,
                        &rows,
                    );
                let detail_y = props_y.saturating_add(props_h).saturating_add(1);
                let detail_w = body.width.saturating_sub(2);
                let detail: Vec<String> = failure
                    .detail
                    .iter()
                    .flat_map(|line| termrock::wrap(line, detail_w.max(8)))
                    .collect();
                let detail_state = ListState::default();
                let close_y = body.bottom().saturating_sub(1);
                let detail_h = close_y.saturating_sub(detail_y);
                List::new(ROOT.sub("failure-detail"))
                    .row(|line: &String, row| {
                        let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                        row.label_patched(&format!("  {line}"), &p);
                    })
                    .draw(
                        ui,
                        Rect::new(body.x, detail_y, body.width, detail_h),
                        &detail_state,
                        &detail,
                    );
                Button::new(ROOT.sub("failure-close"), "Close")
                    .draw(ui, Rect::new(body.right().saturating_sub(9), close_y, 7, 1));
            });
    }

    /// Cockpit hints for the bottom hint bar.
    pub fn hints(log_open: bool, cancel_confirm_open: bool, info_open: bool) -> HintLayer {
        // An open info modal (failure, debug info) replaces the screen
        // hints with the modal footer (tag `Modal::Info` footer hints).
        if info_open {
            return HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("↑↓"),
                        label: "Move",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Label("y"),
                        label: "Copy",
                        priority: 90,
                    },
                    Hint {
                        key: HintKey::Label("Esc"),
                        label: "Close",
                        priority: 80,
                    },
                ],
                badge: None,
                status: None,
                centered: true,
            };
        }
        if cancel_confirm_open {
            HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("← →"),
                        label: "Choose",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Label("Enter"),
                        label: "Confirm",
                        priority: 90,
                    },
                    Hint {
                        key: HintKey::Label("Esc"),
                        label: "Cancel",
                        priority: 80,
                    },
                    Hint {
                        key: HintKey::Label("y / n"),
                        label: "Quick answer",
                        priority: 70,
                    },
                ],
                badge: None,
                status: None,
                centered: true,
            }
        } else if log_open {
            HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("↑↓"),
                        label: "Scroll",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Label("End"),
                        label: "Follow",
                        priority: 90,
                    },
                    Hint {
                        key: HintKey::Label("Esc"),
                        label: "Close log",
                        priority: 80,
                    },
                ],
                badge: None,
                status: None,
                centered: true,
            }
        } else {
            HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("i"),
                        label: "Container info",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Label("c"),
                        label: "Cancel",
                        priority: 90,
                    },
                    Hint {
                        key: HintKey::Label("Ctrl+Q"),
                        label: "Quit",
                        priority: 80,
                    },
                    Hint {
                        key: HintKey::Label("Ctrl+C"),
                        label: "Abort",
                        priority: 70,
                    },
                ],
                badge: None,
                status: None,
                centered: true,
            }
        }
    }
}

/// The launch fixture's credential projection.
///
/// Workspace 1 hands the container every ready effective account. The session
/// starts on `acct-claude-work`, so that account keeps its source annotation
/// and the others stay titles. The line is truncated at paint time.
fn credential_projection(world: &World) -> Option<(String, String)> {
    let workspace = world
        .workspaces
        .iter()
        .find(|workspace| workspace.id == 1)?;
    let ids: Vec<String> = workspace
        .effective_accounts(&world.accounts)
        .into_iter()
        .filter(|entry| entry.usable.is_ready())
        .map(|entry| entry.id)
        .collect();
    const SELECTED: &str = "acct-claude-work";
    // Annotate the session account only when the workspace already hands it
    // to the container. A missing id must not be invented, and must not
    // drop the ready accounts.
    let selected_in = ids.iter().any(|id| id == SELECTED);
    let primary_id = if selected_in {
        SELECTED
    } else {
        ids.first().map(String::as_str)?
    };
    let primary = world.accounts.get(primary_id)?;
    let primary_label = if selected_in {
        format!("{} ({})", primary.title(), primary.source.origin_label())
    } else {
        primary.title()
    };
    let others = ids
        .iter()
        .filter(|id| id.as_str() != primary_id)
        .filter_map(|id| world.accounts.get(id).map(|account| account.title()))
        .collect::<Vec<_>>()
        .join(" · ");
    let count = ids.len();
    let noun = if count == 1 { "account" } else { "accounts" };
    let origin = if others.is_empty() {
        format!("{count} {noun} · {primary_label}")
    } else {
        format!("{count} {noun} · {primary_label} · {others}")
    };
    let level = primary
        .validation
        .level()
        .map(|level| level.label())
        .unwrap_or("quota readable");
    // Work is enabled beside the inherited default, so the tag calls the
    // choice a session pick rather than the resolver's level.
    Some((
        format!("credentials  {origin}"),
        format!("{level} · session choice"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_line_deduplicates_and_never_invents_material() {
        let line = AccountLine::new(
            Some("Claude · Work (1Password)"),
            ["Claude · Personal", "Claude · Personal", ""],
        );
        assert_eq!(line.len(), 2);
        assert_eq!(
            line.text().as_deref(),
            Some("2 accounts · Claude · Work (1Password) · Claude · Personal")
        );
        assert!(!line.text().is_some_and(|text| text.contains("valid-ant")));
    }

    #[test]
    fn handoff_starts_at_frame_zero_and_advances_only_when_called() {
        let mut handoff = HandoffState::default();
        assert!(!handoff.is_active());
        assert_eq!(handoff.stage(), HandoffStage::Capsule);
        handoff.start();
        assert_eq!(handoff.frame(), 0);
        assert_eq!(handoff.stage(), HandoffStage::CockpitDim(1));
        assert!(!handoff.advance());
        assert_eq!(handoff.frame(), 1);
        for _ in 1..HANDOFF_LEN - 1 {
            assert!(!handoff.advance());
        }
        assert_eq!(handoff.frame(), HANDOFF_LEN - 1);
        assert!(handoff.advance());
        assert!(handoff.is_complete());
        assert_eq!(handoff.frame(), HANDOFF_LEN);
    }
}
