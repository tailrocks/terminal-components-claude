//! Settings route state and trust-tab presentation.

use std::collections::BTreeMap;

use jackin_preview_sim::world::TrustRow;
use ratatui::layout::Rect;
use termrock::author::{
    Family, FgStep, Modifier, PaintStyle, Part, Role, StateFlags, StyleDefaults, StylePatch,
    Surface, Ui, Variant,
};
use termrock::controls::Button;
use termrock::layout::Track;
use termrock::navigation::{List, ListState, Tabs, TabsState};
use termrock::{Hint, HintKey, HintLayer, Id, ItemKey, truncate, width};

/// Settings root.
pub const ROOT: Id = Id::root("jackin.settings");
/// Settings form namespace.
pub const FORM: Id = ROOT.sub("form");
/// Settings tab strip.
pub const TABS: Id = ROOT.sub("tabs");
/// Cancel action.
pub const CANCEL: Id = FORM.sub("cancel");
/// Save action.
pub const SAVE: Id = FORM.sub("save");
/// Trust body list.
pub const TRUST: Id = FORM.sub("trust");
/// General body focus target.
pub const GENERAL_BODY: Id = FORM.sub("general");
/// Mounts body focus target.
pub const MOUNTS_BODY: Id = FORM.sub("mounts");
/// Environments body focus target.
pub const ENV_BODY: Id = FORM.sub("env");
/// Agents body focus target.
pub const AGENTS_BODY: Id = FORM.sub("agents");

/// Settings tab names in tab order: General, Mounts, Environments, Agents,
/// Trust (tag `screens/settings.rs` `TAB_NAMES`).
pub const TAB_NAMES: [&str; 5] = ["General", "Mounts", "Environments", "Agents", "Trust"];

/// Which settings region owns the keyboard focus (tag focus model: `TABS`
/// shows the jump footer, the body shows per-tab hints, the Cancel/Save…
/// buttons show the choose footer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsFocus {
    /// The tab strip owns focus.
    #[default]
    Tabs,
    /// The active tab body owns focus.
    Body,
    /// The Cancel/Save… buttons own focus.
    Buttons,
}

/// Settings draft and save lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsState {
    /// Whether the settings draft has unsaved changes.
    pub dirty: bool,
    /// Number of save attempts made for the current draft.
    pub save_attempts: u8,
    /// Latest save error, if one occurred.
    pub save_error: Option<String>,
    /// Which settings region owns the keyboard focus.
    pub focus: SettingsFocus,
    /// Trust body cursor row.
    pub trust_cursor: usize,
    /// Pending trust edits by row index (index → pending trusted flag).
    /// Empty means the trust draft matches the world.
    pub trust_overrides: BTreeMap<usize, bool>,
}

impl SettingsState {
    /// Record a user edit and retain it across a failed save.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.save_error = None;
    }

    /// Begin a new draft lifecycle, clearing attempts from the prior draft.
    pub fn begin_draft(&mut self) {
        self.dirty = true;
        self.save_attempts = 0;
        self.save_error = None;
    }

    /// Whether the current draft can be discarded without confirmation.
    pub const fn is_clean(&self) -> bool {
        !self.dirty
    }

    /// Clear a displayed save error while retaining the draft.
    pub fn clear_error(&mut self) {
        self.save_error = None;
    }

    /// Effective trusted flag for one trust row: the pending override when
    /// present, otherwise the world value.
    pub fn trust_effective(&self, original: &[TrustRow], index: usize) -> bool {
        self.trust_overrides
            .get(&index)
            .copied()
            .unwrap_or_else(|| original.get(index).is_some_and(|row| row.trusted))
    }

    /// Pending trust edits (tag `change_count`, trust portion): overrides
    /// that differ from the world. Out-of-range overrides never count.
    pub fn trust_change_count(&self, original: &[TrustRow]) -> usize {
        self.trust_overrides
            .iter()
            .filter(|(index, pending)| {
                original
                    .get(**index)
                    .is_some_and(|row| row.trusted != **pending)
            })
            .count()
    }

    /// Whether the trust draft differs from the world.
    pub fn trust_dirty(&self, original: &[TrustRow]) -> bool {
        self.trust_change_count(original) > 0
    }

    /// Move the trust cursor by `delta`, clamped to the world rows.
    pub fn move_trust_cursor(&mut self, delta: i32, len: usize) {
        if len == 0 {
            self.trust_cursor = 0;
            return;
        }
        let next = self.trust_cursor as i32 + delta;
        self.trust_cursor = next.clamp(0, len as i32 - 1) as usize;
    }

    /// Flip the trusted flag at the cursor (tag `trust_key` space/enter).
    /// Toggling back to the world value drops the override. Returns the
    /// source and the new flag for the row status, or `None` when the
    /// cursor is out of range.
    pub fn toggle_trust(&mut self, original: &[TrustRow]) -> Option<(String, bool)> {
        let row = original.get(self.trust_cursor)?;
        let next = !self.trust_effective(original, self.trust_cursor);
        if next == row.trusted {
            self.trust_overrides.remove(&self.trust_cursor);
        } else {
            self.trust_overrides.insert(self.trust_cursor, next);
        }
        self.sync_dirty(original);
        Some((row.source.clone(), next))
    }

    /// Recompute the draft flag from the pending trust edits.
    pub fn sync_dirty(&mut self, original: &[TrustRow]) {
        self.dirty = self.trust_dirty(original);
        if self.dirty {
            self.save_error = None;
        }
    }

    /// Drop all pending trust edits (after a confirmed save).
    pub fn clear_trust(&mut self) {
        self.trust_overrides.clear();
        self.dirty = false;
        self.save_error = None;
    }

    /// Record one save attempt and return whether the draft should remain.
    pub fn attempt_save(&mut self, fails: bool) -> bool {
        self.save_attempts = self.save_attempts.saturating_add(1);
        if fails && self.save_attempts == 1 {
            self.dirty = true;
            self.save_error = Some("Settings error · host rejected the update".into());
            true
        } else {
            self.dirty = false;
            self.save_error = None;
            false
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UI Composition: SettingsScreen
// ─────────────────────────────────────────────────────────────────────────────

/// `n` rendered with the singular or plural noun (tag `plural`).
fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Resolve a canvas paint style (accounts palette pattern).
fn resolve_style(ui: &Ui<'_>, fg: Role, bold: bool) -> PaintStyle {
    let mut patch = StylePatch::new()
        .set_fg(fg)
        .set_bg(Role::Surface(Surface::Canvas));
    if bold {
        patch = patch.add(Modifier::BOLD);
    } else {
        patch = patch.remove(Modifier::BOLD);
    }
    ui.style_defaults(
        Family::PANEL,
        Variant::DEFAULT,
        Part::custom("jackin.settings.style"),
        StateFlags::empty(),
        StyleDefaults::new(patch),
        None,
    )
    .style
}

/// One trust row with its pending state resolved.
struct TrustItem {
    index: usize,
    source: String,
    kind: &'static str,
    trusted: bool,
    roles: usize,
    changed: bool,
    selected: bool,
}

/// Reusable global-settings screen composition (live Trust tab).
pub struct SettingsScreen;

impl SettingsScreen {
    /// Tab name for a 1-based settings tab (1 General … 5 Trust).
    pub fn tab_name(tab: usize) -> &'static str {
        TAB_NAMES[tab.saturating_sub(1).min(4)]
    }

    /// Header crumb for a 1-based settings tab (tag `crumb`).
    pub fn crumb(tab: usize) -> String {
        format!("Settings › global › {}", Self::tab_name(tab))
    }

    /// Footer hints for a 1-based settings tab and focus region (tag
    /// `hints`: tab strip, Cancel/Save… buttons, or the per-tab body).
    pub fn hints(tab: usize, focus: SettingsFocus) -> HintLayer {
        fn hint(key: &'static str, label: &'static str, priority: u8) -> Hint {
            Hint {
                key: HintKey::Label(key),
                label,
                priority,
            }
        }
        let tail = [
            hint("[ ]", "Switch tab", 70),
            hint("Ctrl+S", "Save", 60),
            hint("Esc", "Back", 50),
        ];
        let mut hints = match focus {
            SettingsFocus::Tabs => vec![
                hint("← →", "Tab", 100),
                hint("1–5", "Jump", 90),
                hint("Enter", "Body", 80),
            ],
            SettingsFocus::Buttons => vec![hint("← →", "Choose", 100), hint("Enter", "Run", 90)],
            SettingsFocus::Body => match tab {
                2 => vec![
                    hint("Enter", "Edit…", 100),
                    hint("r", "Read-only", 95),
                    hint("i", "Isolation", 90),
                    hint("o", "Open source", 85),
                    hint("d", "Remove", 80),
                    hint("s", "Scope…", 75),
                    hint("a", "Add mount…", 72),
                ],
                3 => vec![
                    hint("Enter", "Edit", 100),
                    hint("m", "Show", 95),
                    hint("p", "1Password…", 90),
                    hint("s", "Scope…", 85),
                    hint("d", "Remove…", 80),
                    hint("a", "Add…", 75),
                ],
                4 => vec![
                    hint("Space", "Cycle mode", 100),
                    hint("d", "Reset to sync", 90),
                    hint("c", "Manage accounts", 80),
                ],
                5 => vec![
                    hint("Space", "Toggle trust", 100),
                    hint("o", "Open source", 90),
                ],
                _ => vec![hint("Space", "Toggle", 100), hint("↑↓", "Move", 90)],
            },
        };
        hints.extend(tail);
        HintLayer {
            hints,
            badge: None,
            status: None,
            centered: true,
        }
    }

    /// Draw the live Trust tab: tab strip, role-source list, notes, and the
    /// Cancel/Save… actions (tag `render` + `render_trust`).
    pub fn draw_trust(
        ui: &mut Ui<'_>,
        settings: &SettingsState,
        world: &jackin_preview_sim::world::World,
    ) {
        let full = ui.full();
        let original = &world.global.trust;
        let dirty = settings.trust_dirty(original);

        // 1. Tab strip.
        let tabs_all = [0usize, 1, 2, 3, 4];
        let mut tab_state = TabsState::default();
        tab_state.set_active(4, ItemKey::num(4));
        Tabs::new(TABS)
            .key(|tab: &usize| ItemKey::num(*tab as u64))
            .row(|tab: &usize, row| {
                let name = TAB_NAMES[*tab];
                if dirty && *tab == 4 {
                    row.label(&format!("{name} •"));
                } else {
                    row.label(name);
                }
            })
            .draw(
                ui,
                Rect::new(2, 3, full.width.saturating_sub(4), 2),
                &tab_state,
                &tabs_all,
            );

        // 2. Title and trust counts.
        let title_style = resolve_style(ui, Role::Fg(FgStep::Secondary), true);
        ui.paint_str(Rect::new(4, 6, 12, 1), "Role sources", title_style);
        let effective: Vec<bool> = (0..original.len())
            .map(|i| settings.trust_effective(original, i))
            .collect();
        let trusted = effective.iter().filter(|t| **t).count();
        let meta = format!(
            "{} · {}",
            plural(trusted, "trusted", "trusted"),
            plural(original.len() - trusted, "untrusted", "untrusted")
        );
        let faint = resolve_style(ui, Role::Fg(FgStep::Faint), false);
        let meta_w = width(&meta) as u16;
        let meta_x = full
            .width
            .saturating_sub(2)
            .saturating_sub(meta_w)
            .saturating_sub(2);
        ui.paint_str(Rect::new(meta_x, 6, meta_w, 1), &meta, faint);

        // 3. Role-source rows (stock list; per-cell styles mirror the tag).
        let list_width = full.width.saturating_sub(4);
        let src_w = (list_width.saturating_sub(30).clamp(16, 40)) as usize;
        let items: Vec<TrustItem> = original
            .iter()
            .enumerate()
            .map(|(index, row)| TrustItem {
                index,
                // The frozen baseline keeps the 41-char acme source
                // untruncated with the kind column fixed, so the source may
                // overflow its padded width by one cell.
                source: format!(
                    "{:<src_w$}",
                    truncate(&row.source, src_w.saturating_add(1) as u16)
                ),
                kind: row.kind,
                trusted: effective[index],
                roles: row.roles,
                changed: original
                    .get(index)
                    .is_some_and(|o| o.trusted != effective[index]),
                selected: index == settings.trust_cursor,
            })
            .collect();
        let mut list_state = ListState::default();
        list_state.set_cursor(settings.trust_cursor, ItemKey::index(settings.trust_cursor));
        list_state.choose(Some(ItemKey::index(settings.trust_cursor)));
        List::new(TRUST)
            .key(|item: &TrustItem| ItemKey::index(item.index))
            .row(|item: &TrustItem, row| {
                let mut cols = row.columns_with_gap(
                    &[
                        Track::Fixed(1),
                        Track::Fixed(src_w as u16),
                        Track::Fixed(6),
                        Track::Fixed(13),
                        Track::Flex(1),
                    ],
                    1,
                );
                let p_warn = StylePatch::new().set_fg(Role::Warning);
                cols.cell(0)
                    .patch(&p_warn)
                    .text(if item.changed { "•" } else { " " });
                if item.selected {
                    let p_bold = StylePatch::new().add(Modifier::BOLD);
                    cols.cell(1).patch(&p_bold).text(&item.source);
                } else {
                    cols.cell(1).text(&item.source);
                }
                let p_muted = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
                cols.cell(2)
                    .patch(&p_muted)
                    .text(&format!(" {:<5}", item.kind));
                let mark = if item.trusted {
                    "[✓] trusted"
                } else {
                    "[ ] untrusted"
                };
                if item.trusted {
                    cols.cell(3).text(mark);
                } else {
                    cols.cell(3).patch(&p_warn).text(mark);
                }
                let p_faint = StylePatch::new().set_fg(Role::Fg(FgStep::Faint));
                cols.cell(4)
                    .patch(&p_faint)
                    .text(&format!(" {}", plural(item.roles, "role", "roles")));
            })
            .draw(
                ui,
                Rect::new(2, 7, list_width, (items.len() as u16).max(1)),
                &list_state,
                &items,
            );

        // 4. Untrusted-source notes.
        let note_y = 7 + items.len() as u16 + 1;
        if note_y + 1 < 37 {
            let note_w = full.width.saturating_sub(4);
            ui.paint_str(
                Rect::new(4, note_y, note_w, 1),
                &truncate(
                    "An untrusted source blocks + Load role until it is trusted here or",
                    note_w,
                ),
                faint,
            );
            ui.paint_str(
                Rect::new(4, note_y + 1, note_w, 1),
                &truncate("in the trust dialog that the load opens.", note_w),
                faint,
            );
        }

        // 5. Bottom actions.
        Button::new(CANCEL, "Cancel").draw(ui, Rect::new(97, 37, 8, 1));
        Button::new(SAVE, "Save…").draw(ui, Rect::new(108, 37, 8, 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_save_keeps_dirty_draft_and_retry_commits() {
        let mut state = SettingsState::default();
        state.begin_draft();
        assert!(state.attempt_save(true));
        assert!(!state.is_clean());
        assert!(state.save_error.is_some());

        assert!(!state.attempt_save(true));
        assert!(state.is_clean());
        assert!(state.save_error.is_none());
    }

    #[test]
    fn a_new_edit_clears_old_error_and_restarts_attempts() {
        let mut state = SettingsState::default();
        state.begin_draft();
        assert!(state.attempt_save(true));
        state.begin_draft();
        assert_eq!(state.save_attempts, 0);
        assert!(state.save_error.is_none());
        assert!(!state.is_clean());
    }

    fn trust_fixture() -> Vec<TrustRow> {
        vec![
            TrustRow {
                source: "github.com/chainargos/roles".into(),
                kind: "git",
                trusted: true,
                roles: 4,
            },
            TrustRow {
                source: "github.com/acme-labs/roles-experimental".into(),
                kind: "git",
                trusted: false,
                roles: 1,
            },
        ]
    }

    #[test]
    fn save_preview_flow_yields_two_row_targeted_changes() {
        let original = trust_fixture();
        let mut state = SettingsState::default();
        assert_eq!(state.trust_change_count(&original), 0);
        assert!(state.is_clean());

        let (source, trusted) = state.toggle_trust(&original).unwrap();
        assert_eq!(source, "github.com/chainargos/roles");
        assert!(!trusted);
        assert_eq!(state.trust_change_count(&original), 1);
        assert!(!state.is_clean());

        state.move_trust_cursor(1, original.len());
        let (source, trusted) = state.toggle_trust(&original).unwrap();
        assert_eq!(source, "github.com/acme-labs/roles-experimental");
        assert!(trusted);
        assert_eq!(state.trust_change_count(&original), 2);
        assert!(!state.is_clean());
    }

    #[test]
    fn toggling_back_to_world_value_drops_the_override() {
        let original = trust_fixture();
        let mut state = SettingsState::default();
        state.toggle_trust(&original);
        assert_eq!(state.trust_change_count(&original), 1);
        state.toggle_trust(&original);
        assert_eq!(state.trust_change_count(&original), 0);
        assert!(state.trust_overrides.is_empty());
        assert!(state.is_clean());
    }

    #[test]
    fn trust_cursor_clamps_and_toggle_out_of_range_is_none() {
        let original = trust_fixture();
        let mut state = SettingsState::default();
        state.move_trust_cursor(-5, original.len());
        assert_eq!(state.trust_cursor, 0);
        state.move_trust_cursor(99, original.len());
        assert_eq!(state.trust_cursor, original.len() - 1);
        state.trust_cursor = 99;
        assert!(state.toggle_trust(&original).is_none());
        assert_eq!(state.trust_change_count(&original), 0);
    }

    #[test]
    fn out_of_range_overrides_never_count() {
        let original = trust_fixture();
        let mut state = SettingsState::default();
        state.trust_overrides.insert(99, true);
        assert_eq!(state.trust_change_count(&original), 0);
        assert!(!state.trust_dirty(&original));
    }

    #[test]
    fn crumbs_name_every_tab() {
        assert_eq!(SettingsScreen::crumb(1), "Settings › global › General");
        assert_eq!(SettingsScreen::crumb(2), "Settings › global › Mounts");
        assert_eq!(SettingsScreen::crumb(3), "Settings › global › Environments");
        assert_eq!(SettingsScreen::crumb(4), "Settings › global › Agents");
        assert_eq!(SettingsScreen::crumb(5), "Settings › global › Trust");
    }

    #[test]
    fn hints_follow_focus_and_tab() {
        let tabs = SettingsScreen::hints(5, SettingsFocus::Tabs);
        let labels: Vec<&str> = tabs.hints.iter().map(|h| h.label).collect();
        assert!(labels.contains(&"Jump"));
        assert!(labels.contains(&"Body"));
        assert!(!labels.contains(&"Toggle trust"));

        let trust = SettingsScreen::hints(5, SettingsFocus::Body);
        let labels: Vec<&str> = trust.hints.iter().map(|h| h.label).collect();
        assert!(labels.contains(&"Toggle trust"));
        assert!(labels.contains(&"Open source"));
        assert!(labels.contains(&"Switch tab"));

        let agents = SettingsScreen::hints(4, SettingsFocus::Body);
        let labels: Vec<&str> = agents.hints.iter().map(|h| h.label).collect();
        assert!(labels.contains(&"Cycle mode"));

        let buttons = SettingsScreen::hints(5, SettingsFocus::Buttons);
        let labels: Vec<&str> = buttons.hints.iter().map(|h| h.label).collect();
        assert!(labels.contains(&"Choose"));
        assert!(labels.contains(&"Run"));
    }
}
