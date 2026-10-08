//! Launch cockpit state and controls.

use ratatui::layout::Rect;
use termrock::author::{FgStep, Modifier, Role, StylePatch, Ui};
use termrock::controls::{Brand, Button, Panel, PanelKind};
use termrock::layout::Track;
use termrock::navigation::{List, ListState};
use termrock::{Hint, HintKey, HintLayer, Id, Variant};

use jackin_preview_presentation::rain::{HANDOFF_LEN, HandoffStage, handoff_stage};
use jackin_preview_sim::launch::{LaunchFailure, Stage};
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
    /// Render the main Cockpit surface.
    pub fn draw(
        ui: &mut Ui<'_>,
        _area: Rect,
        _cockpit: &CockpitState,
        world: &World,
        role: &str,
        debug: bool,
    ) {
        let ws_name = world
            .workspaces
            .first()
            .map_or("payments-platform", |w| w.name.as_str());

        // 1. Centered jackin❯ brand
        let _ = Brand::new(ROOT.sub("brand"), "jackin❯").draw(ui, Rect::new(55, 3, 9, 1));

        // 2. Identity line 4: title
        let head = format!("Loading {role} into workspace {ws_name}");
        let ident1_state = ListState::default();
        List::new(ROOT.sub("ident-title"))
            .row(|_, row| {
                let p = StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .add(Modifier::BOLD);
                row.label_patched(&head, &p);
            })
            .draw(ui, Rect::new(33, 4, 54, 1), &ident1_state, &[()]);

        // 3. Identity line 5: agent / account
        let ident2_state = ListState::default();
        List::new(ROOT.sub("ident-agent"))
            .row(|_, row| {
                let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                row.label_patched(
                    "Claude Code · Anthropic / Claude · account Claude · Work (session choice)",
                    &p,
                );
            })
            .draw(ui, Rect::new(20, 5, 80, 1), &ident2_state, &[()]);

        // 4. Identity line 6: stage progress summary
        let ident3_state = ListState::default();
        List::new(ROOT.sub("ident-stage"))
            .row(|_, row| {
                let p = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                row.label_patched("stage 3 of 11 · Credentials · 2 done · 0 skipped", &p);
            })
            .draw(ui, Rect::new(33, 6, 52, 1), &ident3_state, &[()]);

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
        List::new(STAGES)
            .row(|item: &StageItem, row| {
                let mut cols = row.columns_with_gap(
                    &[
                        Track::Fixed(3),
                        Track::Fixed(3),
                        Track::Fixed(32),
                        Track::Fixed(6),
                    ],
                    0,
                );

                if item.is_accent {
                    let p_accent = StylePatch::new().set_fg(Role::Accent);
                    let glyph_str = format!(" {} ", item.glyph);
                    let text = if item.glyph.is_empty() {
                        "   "
                    } else {
                        glyph_str.as_str()
                    };
                    cols.cell(0).patch(&p_accent).text(text);
                } else {
                    cols.cell(0).text("   ");
                }

                if item.is_current {
                    let p_sec = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    cols.cell(1).patch(&p_sec).text(&format!("{} ", item.num));
                    let p_bold = StylePatch::new()
                        .set_fg(Role::Fg(FgStep::Primary))
                        .add(Modifier::BOLD);
                    cols.cell(2).patch(&p_bold).text(item.name);
                } else if item.is_accent {
                    let p_faint = StylePatch::new().set_fg(Role::BorderStrong);
                    cols.cell(1).patch(&p_faint).text(&format!("{} ", item.num));
                    let p_sec = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                    cols.cell(2).patch(&p_sec).text(item.name);
                    cols.cell(3).patch(&p_faint).text(item.status);
                } else {
                    let p_faint = StylePatch::new().set_fg(Role::BorderStrong);
                    cols.cell(1).patch(&p_faint).text(&format!("{} ", item.num));
                    let p_muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                    cols.cell(2).patch(&p_muted).text(item.name);
                    cols.cell(3).patch(&p_faint).text(item.status);
                }
            })
            .draw(ui, Rect::new(38, 8, 44, 11), &rail_state, &stages);

        // 6. Credentials projection line under stage rail
        let cred_state = ListState::default();
        List::new(ACCOUNT_LINE)
            .row(|_, row| {
                let p = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                row.label_patched(
                    "credentials  5 accounts · Claude · Work (1Password) · Claude · Personal · Codex · P…",
                    &p,
                );
            })
            .draw(ui, Rect::new(34, 20, 84, 1), &cred_state, &[()]);

        let quota_state = ListState::default();
        List::new(ACCOUNT_LINE.sub("quota"))
            .row(|_, row| {
                let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                row.label_patched("quota readable · session choice", &p);
            })
            .draw(ui, Rect::new(47, 21, 31, 1), &quota_state, &[()]);

        // 7. Bottom status line
        let status_state = ListState::default();
        List::new(ROOT.sub("status"))
            .row(|_, row| {
                let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                row.label_patched("⠋ Resolving credentials…", &p);
            })
            .draw(ui, Rect::new(1, 36, 24, 1), &status_state, &[()]);

        if debug {
            let debug_state = ListState::default();
            List::new(ROOT.sub("debug-chip"))
                .row(|_, row| {
                    let p = StylePatch::new().set_fg(Role::Warning);
                    row.label_patched(" run-202609030914-b5df ", &p);
                })
                .draw(ui, Rect::new(96, 36, 23, 1), &debug_state, &[()]);
        }
    }

    /// Render the container debug info modal.
    pub fn draw_info(ui: &mut Ui<'_>, _area: Rect, world: &World, role: &str, _debug: bool) {
        let modal_area = Rect::new(27, 14, 66, 11);
        let ws_name = world
            .workspaces
            .first()
            .map_or("payments-platform", |w| w.name.as_str());
        let target_val = format!("{role} into workspace {ws_name}");

        Panel::new(ROOT.sub("debug-info"))
            .kind(PanelKind::Framed)
            .title("Debug info")
            .meta("read-only")
            .draw(ui, modal_area, |ui, body| {
                let list_state = ListState::default();
                let props: [(&str, String, bool, bool); 5] = [
                    ("Target", target_val, true, true),
                    ("Role", role.to_owned(), false, false),
                    (
                        "Agent",
                        "Claude Code · account Claude · Work".to_owned(),
                        false,
                        false,
                    ),
                    ("Run id", "run-202609030914-b5df".to_owned(), false, false),
                    ("jackin", "0.6.4 · preview".to_owned(), false, false),
                ];

                List::new(ROOT.sub("info-props"))
                    .row(
                        |(label, val, has_focus, is_bold): &(&str, String, bool, bool), row| {
                            let mut cols = row.columns_with_gap(
                                &[Track::Fixed(2), Track::Fixed(6), Track::Flex(1)],
                                2,
                            );

                            if *has_focus {
                                let p_focus = StylePatch::new().set_fg(Role::Accent);
                                cols.cell(0).patch(&p_focus).text("▎");
                            } else {
                                cols.cell(0).text("  ");
                            }

                            let p_label = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                            cols.cell(1).patch(&p_label).text(label);

                            if *is_bold {
                                let p_val = StylePatch::new()
                                    .set_fg(Role::Fg(FgStep::Primary))
                                    .add(Modifier::BOLD);
                                cols.cell(2).patch(&p_val).text(val);
                            } else {
                                let p_val = StylePatch::new().set_fg(Role::Fg(FgStep::Primary));
                                cols.cell(2).patch(&p_val).text(val);
                            }
                        },
                    )
                    .draw(
                        ui,
                        Rect::new(body.x, body.y, body.width, 5),
                        &list_state,
                        &props,
                    );

                let close_area = Rect::new(
                    body.right().saturating_sub(9),
                    body.bottom().saturating_sub(1),
                    7,
                    1,
                );
                Button::new(ROOT.sub("info-close"), "Close").draw(ui, close_area);
            });
    }

    /// Render the cancel confirmation modal.
    pub fn draw_cancel_confirm(ui: &mut Ui<'_>, _area: Rect) {
        let modal_area = Rect::new(33, 15, 54, 11);

        Panel::new(ROOT.sub("cancel-confirm"))
            .kind(PanelKind::Framed)
            .draw(ui, modal_area, |ui, body| {
                let title_state = ListState::default();
                List::new(ROOT.sub("cancel-title"))
                    .row(|_, row| {
                        let p = StylePatch::new()
                            .set_fg(Role::Fg(FgStep::Primary))
                            .add(Modifier::BOLD);
                        row.label_patched("  Cancel the launch?", &p);
                    })
                    .draw(
                        ui,
                        Rect::new(body.x, body.y.saturating_add(1), body.width, 1),
                        &title_state,
                        &[()],
                    );

                let desc_state = ListState::default();
                let desc_lines = [
                    "  The pipeline stops at its current stage and the",
                    "  partially prepared instance is marked failed",
                    "  setup. Nothing is attached.",
                ];
                List::new(ROOT.sub("cancel-desc"))
                    .row(|line: &&str, row| {
                        let p = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
                        row.label_patched(line, &p);
                    })
                    .draw(
                        ui,
                        Rect::new(body.x, body.y.saturating_add(3), body.width, 3),
                        &desc_state,
                        &desc_lines,
                    );

                let cancel_btn_area = Rect::new(60, 23, 8, 1);
                let cancel_launch_area = Rect::new(69, 23, 15, 1);
                Button::new(CANCEL, "Cancel").draw(ui, cancel_btn_area);
                Button::new(ROOT.sub("cancel-launch"), "Cancel launch")
                    .variant(Variant::DANGER)
                    .draw(ui, cancel_launch_area);
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
    pub fn hints(log_open: bool, cancel_confirm_open: bool) -> HintLayer {
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
