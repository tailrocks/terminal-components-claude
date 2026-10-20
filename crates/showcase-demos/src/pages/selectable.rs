use termrock::{
    Action, ActionKey, Color, Cx, Dialog, DialogAction, DialogState, Focusability, GlyphRole, Id,
    Intent, Key, KeyCode, KeyModifiers, Modifier, PaintStyle, Rect, Response, Surface, Ui, id,
    width, wrap,
};

use super::frame;
use super::{accent_tint_bg, color_u32};

const SECTION: Id = id!("selectable.sections");
const TABLE: Id = id!("selectable.members");
pub const DELETE_DIALOG: Id = id!("selectable.delete_confirm");

const DELETE_BODY: &str = "They lose access to the workspace right away. This can't be undone.";

const DIALOG_HINTS: &[(&str, &str)] = &[("←→", "Choose"), ("Enter", "Cancel"), ("Esc", "Cancel")];
const SELECTION_HINTS: &[(&str, &str)] = &[
    ("Space", "Select"),
    ("Shift+↑↓", "Extend"),
    ("Option+click", "Toggle row"),
    ("Ctrl+A", "Select all"),
    ("Enter", "Edit"),
    ("Esc", "Clear selection"),
];
const TABLE_HINTS: &[(&str, &str)] = &[
    ("Space", "Select"),
    ("Shift+↑↓", "Extend"),
    ("Ctrl+A", "Select all"),
    ("Enter", "Edit"),
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Model {
    cursor: usize,
    anchor: Option<usize>,
    selected: u16,
    dialog: bool,
    table_focus: bool,
    dialog_title: String,
    dialog_confirm: String,
    dialog_state: DialogState,
}

impl Model {
    pub fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let _ = cx.intents(SECTION);
        let mut changed = false;
        let mut ask = false;
        for intent in cx.intents(TABLE) {
            match intent {
                Intent::FocusIn { .. } => self.table_focus = true,
                Intent::FocusOut { .. } => self.table_focus = false,
                Intent::Key(key) if self.table_focus => {
                    if key.code == KeyCode::Char('d')
                        || key.code == KeyCode::Char('D')
                        || key.code == KeyCode::Delete
                    {
                        ask |= self.selected != 0;
                    } else {
                        changed |= self.apply_key(key);
                    }
                }
                _ => {}
            }
        }
        if ask {
            self.open_dialog(cx);
            changed = true;
        }
        if self.dialog {
            changed |= self.poll_dialog(cx);
        }
        if changed {
            Response::changed()
        } else {
            Response::ignored()
        }
    }

    fn open_dialog(&mut self, cx: &mut Cx<'_>) {
        let count = self.selected_count();
        let noun = if count == 1 { "member" } else { "members" };
        self.dialog_title = format!("Delete {count} {noun}?");
        self.dialog_confirm = format!("Delete {count} {noun}");
        let title = self.dialog_title.clone();
        let confirm = self.dialog_confirm.clone();
        Self::with_dialog(&title, &confirm, |dialog| {
            cx.open_layer(DELETE_DIALOG, dialog.layer(cx));
        });
        self.dialog = true;
    }

    fn poll_dialog(&mut self, cx: &mut Cx<'_>) -> bool {
        let title = self.dialog_title.clone();
        let confirm = self.dialog_confirm.clone();
        let action = Self::with_dialog(&title, &confirm, |dialog| {
            dialog.update(cx, &mut self.dialog_state)
        });
        let Some(action) = action.action_ref() else {
            return false;
        };
        if cx.is_open(DELETE_DIALOG) {
            cx.close_layer(DELETE_DIALOG, None);
        }
        self.dialog = false;
        if matches!(action, DialogAction::Action(key) if *key == ActionKey::CONFIRM) {
            self.clear_selection();
        }
        true
    }

    /// Builds the delete dialog and hands it to `body`. The action labels
    /// borrow the caller-owned `title`/`confirm` strings, so the dialog
    /// never outlives them.
    fn with_dialog<R>(title: &str, confirm: &str, body: impl FnOnce(Dialog<'_>) -> R) -> R {
        let actions = [
            Action::new(ActionKey::CANCEL, "Cancel"),
            Action::danger(ActionKey::CONFIRM, confirm),
        ];
        body(
            Dialog::new(DELETE_DIALOG)
                .title(title)
                .description(DELETE_BODY)
                .actions(&actions)
                .cancel(ActionKey::CANCEL),
        )
    }

    /// Renders the delete-confirmation dialog in the overlay layer. Called by
    /// the shell after the page paints, matching the dialogs page.
    pub fn draw_dialog(ui: &mut Ui<'_>, model: &Model) {
        if !model.dialog {
            return;
        }
        Model::with_dialog(&model.dialog_title, &model.dialog_confirm, |dialog| {
            ui.layer(DELETE_DIALOG, |ui, area| {
                dialog.draw(ui, area, &model.dialog_state, |_, _| {});
            });
        });
    }

    pub fn hints(&self) -> &'static [(&'static str, &'static str)] {
        if self.dialog {
            DIALOG_HINTS
        } else if self.selected != 0 {
            SELECTION_HINTS
        } else if self.table_focus {
            TABLE_HINTS
        } else {
            &[]
        }
    }

    fn clear_selection(&mut self) {
        self.selected = 0;
        self.anchor = None;
    }

    pub fn blur(&mut self) {
        self.table_focus = false;
    }

    pub const fn selected_count(&self) -> usize {
        self.selected.count_ones() as usize
    }

    fn apply_key(&mut self, key: Key) -> bool {
        if key.code == KeyCode::Char('a') && key.mods == KeyModifiers::CONTROL {
            self.select_all();
            return true;
        }
        if !key.mods.difference(KeyModifiers::SHIFT).is_empty() {
            return false;
        }
        match key.code {
            KeyCode::Down if key.shift() => self.extend(1),
            KeyCode::Up if key.shift() => self.extend(-1),
            KeyCode::Down => self.step(1),
            KeyCode::Up => self.step(-1),
            KeyCode::Char(' ') => self.toggle(),
            KeyCode::Esc => {
                let had = self.selected != 0;
                self.clear_selection();
                return had;
            }
            _ => return false,
        }
        true
    }

    fn select_all(&mut self) {
        let all = MEMBERS
            .iter()
            .enumerate()
            .filter(|(_, member)| !member.disabled)
            .fold(0_u16, |acc, (index, _)| acc | (1_u16 << index));
        if self.selected == all {
            self.clear_selection();
        } else {
            self.selected = all;
            self.anchor = None;
        }
    }

    fn step(&mut self, delta: i32) {
        let next = i32::try_from(self.cursor)
            .unwrap_or(0)
            .saturating_add(delta);
        let last = i32::try_from(MEMBERS.len().saturating_sub(1)).unwrap_or(0);
        self.cursor = usize::try_from(next.clamp(0, last)).unwrap_or(0);
    }

    fn extend(&mut self, delta: i32) {
        self.step(delta);
        let Some(anchor) = self.anchor else {
            return;
        };
        let (start, end) = if anchor <= self.cursor {
            (anchor, self.cursor)
        } else {
            (self.cursor, anchor)
        };
        self.selected = 0;
        for (index, member) in MEMBERS.iter().enumerate().take(end + 1).skip(start) {
            if !member.disabled {
                self.selected |= 1_u16 << index;
            }
        }
    }

    fn toggle(&mut self) {
        if MEMBERS
            .get(self.cursor)
            .is_some_and(|member| member.disabled)
        {
            return;
        }
        let bit = 1_u16 << self.cursor;
        if self.selected & bit == 0 {
            self.selected |= bit;
            self.anchor = Some(self.cursor);
        } else {
            self.selected &= !bit;
            if self.selected == 0 {
                self.anchor = None;
            }
        }
    }
}

const TITLE: &str = "Selectable table";
const BLURB: &str = "Select rows, edit one or many together, delete with a confirmation";
const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
// Page rect inside the 160×50 shell. Narrower shells stay stacked.
const WIDE_PAGE_W: u16 = 134;
const WIDE_PAGE_H: u16 = 46;

const MEMBERS_H: u16 = 15;
const EDITOR_H: u16 = 17;
const CARD_GAP: u16 = 1;
const SCROLL_ROOM: u16 = 2;
const COL_GAP: u16 = 2;
const TABLE_INSET: u16 = 3;
const TABLE_CHROME: u16 = 5;
const CHECK_W: u16 = 4;

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const ACCENT: u32 = 0x48e054;
const TAB_ACTIVE: u32 = 0x27272a;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;
const DISABLED: u32 = 0x4d4d4d;
const WARNING: u32 = 0xf59e09;
const FADE_OUTER: f32 = 0.55;

const CHECK_OFF: &str = "[ ]";
const CHECK_ON: &str = "[✓]";
const CHECK_MIXED: &str = "[–]";
const EMPTY_MARK: &str = "\u{2014}";
const HINT: &str = "Select rows in the table to edit them here. Space selects, \
Shift+\u{2191}\u{2193} extends, Ctrl+A selects all.";

struct Spec {
    title: &'static str,
    min: u16,
    flex: bool,
}

const SPECS: [Spec; 7] = [
    Spec {
        title: "Name",
        min: 15,
        flex: true,
    },
    Spec {
        title: "Role",
        min: 6,
        flex: false,
    },
    Spec {
        title: "Status",
        min: 9,
        flex: false,
    },
    Spec {
        title: "Region",
        min: 6,
        flex: false,
    },
    Spec {
        title: "On call",
        min: 7,
        flex: false,
    },
    Spec {
        title: "Email",
        min: 20,
        flex: true,
    },
    Spec {
        title: "Notes",
        min: 22,
        flex: true,
    },
];

#[derive(Clone, Copy)]
enum Status {
    Active,
    Invited,
    Suspended,
}

struct Member {
    name: &'static str,
    role: &'static str,
    status: Status,
    region: &'static str,
    on_call: bool,
    email: &'static str,
    notes: &'static str,
    disabled: bool,
}

const MEMBERS: [Member; 12] = [
    Member {
        name: "Dana Whitfield",
        role: "Owner",
        status: Status::Active,
        region: "EU",
        on_call: true,
        email: "dana@northwind.dev",
        notes: "Workspace owner",
        disabled: true,
    },
    Member {
        name: "Ari Moreno",
        role: "Admin",
        status: Status::Active,
        region: "US",
        on_call: true,
        email: "ari@northwind.dev",
        notes: "Runs the release train",
        disabled: false,
    },
    Member {
        name: "Cleo Barnes",
        role: "Editor",
        status: Status::Active,
        region: "EU",
        on_call: false,
        email: "cleo@northwind.dev",
        notes: "Docs and changelog",
        disabled: false,
    },
    Member {
        name: "Bo Lindqvist",
        role: "Editor",
        status: Status::Invited,
        region: "EU",
        on_call: false,
        email: "bo@northwind.dev",
        notes: "",
        disabled: false,
    },
    Member {
        name: "Eli Nakamura",
        role: "Viewer",
        status: Status::Active,
        region: "APAC",
        on_call: false,
        email: "eli@contractor.io",
        notes: "Contract ends in March",
        disabled: false,
    },
    Member {
        name: "Fay Okafor",
        role: "Editor",
        status: Status::Active,
        region: "US",
        on_call: true,
        email: "fay@northwind.dev",
        notes: "On call this week",
        disabled: false,
    },
    Member {
        name: "Gus Petrov",
        role: "Viewer",
        status: Status::Suspended,
        region: "EU",
        on_call: false,
        email: "gus@northwind.dev",
        notes: "Left the team",
        disabled: false,
    },
    Member {
        name: "Hana Sato",
        role: "Editor",
        status: Status::Active,
        region: "APAC",
        on_call: false,
        email: "hana@northwind.dev",
        notes: "",
        disabled: false,
    },
    Member {
        name: "Ivo Brandt",
        role: "Admin",
        status: Status::Active,
        region: "EU",
        on_call: true,
        email: "ivo@northwind.dev",
        notes: "Billing contact",
        disabled: false,
    },
    Member {
        name: "June Park",
        role: "Viewer",
        status: Status::Invited,
        region: "US",
        on_call: false,
        email: "june@northwind.dev",
        notes: "",
        disabled: false,
    },
    Member {
        name: "Kai Mensah",
        role: "Editor",
        status: Status::Active,
        region: "APAC",
        on_call: false,
        email: "kai@northwind.dev",
        notes: "Mobile app",
        disabled: false,
    },
    Member {
        name: "Lena Varga",
        role: "Viewer",
        status: Status::Active,
        region: "EU",
        on_call: false,
        email: "lena@northwind.dev",
        notes: "",
        disabled: false,
    },
];

struct Cell {
    text: &'static str,
    fg: u32,
}

struct Geom {
    cols_w: u16,
    has_sb: bool,
    has_bar: bool,
    body: u16,
    widths: Vec<u16>,
    shown: usize,
}

pub fn paint(ui: &mut Ui<'_>, area: Rect, model: &Model) {
    if area.width > 0 && area.height > 0 {
        ui.register_control(
            SECTION,
            Rect::new(area.x, area.y, 1, 1),
            Focusability::Focusable,
        );
        let table_y = area
            .y
            .saturating_add(1)
            .min(area.bottom().saturating_sub(1));
        ui.register_control(
            TABLE,
            Rect::new(area.x, table_y, 1, 1),
            Focusability::Focusable,
        );
    }
    ui.with_area(area, |ui| {
        frame(ui, area, TITLE, "", |ui, body| {
            paint_blurb(ui, body);
            paint_section_strip(
                ui,
                Rect {
                    height: 2.min(body.height),
                    ..body
                },
            );
            let top = body.y.saturating_add(3);
            if top >= body.bottom() {
                return;
            }
            let playground = Rect {
                y: top,
                height: body.bottom().saturating_sub(top),
                ..body
            };
            if wide_page(area) {
                paint_wide_playground(ui, playground, model);
            } else {
                paint_playground(ui, playground, model);
            }
        });
    });
}

fn rgb(hex: u32) -> Color {
    Color::Rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

fn ink(fg: u32, bg: u32) -> PaintStyle {
    PaintStyle::new().fg(rgb(fg)).bg(rgb(bg))
}

fn mix(fg: u32, bg: u32, keep: f32) -> u32 {
    let channel = |shift: u32| {
        let from = f32::from(((fg >> shift) & 0xff) as u8);
        let into = f32::from(((bg >> shift) & 0xff) as u8);
        (into + (from - into) * keep).round().clamp(0.0, 255.0) as u32
    };
    (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

fn faded(color: u32, bg: u32, keep: Option<f32>) -> u32 {
    keep.map_or(color, |keep| mix(color, bg, keep))
}

fn paint_at(ui: &mut Ui<'_>, x: u16, y: u16, columns: u16, text: &str, style: PaintStyle) {
    if columns == 0 {
        return;
    }
    ui.paint_str(Rect::new(x, y, columns, 1), text, style);
}

fn paint_repeat(ui: &mut Ui<'_>, x: u16, y: u16, columns: u16, glyph: &str, style: PaintStyle) {
    if columns == 0 {
        return;
    }
    paint_at(
        ui,
        x,
        y,
        columns,
        &glyph.repeat(usize::from(columns)),
        style,
    );
}

fn fit(text: &str, columns: u16) -> String {
    let shown = termrock::truncate(text, columns);
    let pad = usize::from(columns.saturating_sub(width(&shown)));
    format!("{shown}{}", " ".repeat(pad))
}

fn tab_width(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn paint_blurb(ui: &mut Ui<'_>, body: Rect) {
    let title_w = width(TITLE);
    if body.width <= title_w.saturating_add(4) {
        return;
    }
    let meta = Rect::new(
        body.x.saturating_add(title_w).saturating_add(2),
        body.y.saturating_sub(2),
        body.width.saturating_sub(title_w).saturating_sub(3),
        1,
    );
    let style = ink(MUTED, CANVAS);
    if width(BLURB) <= meta.width {
        ui.paint_str(meta, BLURB, style);
        return;
    }
    if meta.width == 0 {
        return;
    }
    let budget = Rect {
        width: meta.width.saturating_sub(1),
        ..meta
    };
    let used = ui.paint_str(budget, BLURB, style);
    ui.paint_str(
        Rect::new(meta.x.saturating_add(used), meta.y, 1, 1),
        "…",
        style,
    );
}

fn paint_section_strip(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let mut x = area.x;
    let mut tab_end = area.x;
    for (index, label) in SECTIONS.iter().enumerate() {
        let w = tab_width(label);
        if x.saturating_add(w) > area.right() {
            break;
        }
        let style = if index == 0 {
            ink(WHITE, TAB_ACTIVE).add_modifier(Modifier::BOLD)
        } else {
            ink(SECONDARY, CANVAS)
        };
        ui.fill(Rect::new(x, area.y, w, 1), style);
        ui.paint_str(
            Rect::new(x.saturating_add(1), area.y, w.saturating_sub(1), 1),
            label,
            style,
        );
        tab_end = x.saturating_add(w);
        x = tab_end.saturating_add(1);
    }
    if area.width >= SECTION_WIDE {
        let caption_width = width(SECTION_CAPTION);
        let caption_x = area.right().saturating_sub(caption_width.saturating_add(1));
        if caption_x > tab_end.saturating_add(2) {
            ui.paint_str(
                Rect::new(caption_x, area.y, caption_width, 1),
                SECTION_CAPTION,
                ink(FAINT, CANVAS),
            );
        }
    }
    if area.height < 2 {
        return;
    }
    let rule_y = area.y.saturating_add(1);
    paint_repeat(ui, area.x, rule_y, area.width, "─", ink(RULE, CANVAS));
    let active = tab_width(SECTIONS[0]).min(area.width);
    paint_repeat(ui, area.x, rule_y, active, "━", ink(ACCENT, CANVAS));
}

fn paint_card_chrome(ui: &mut Ui<'_>, card: Rect, title: &str, meta: &str) {
    paint_card_chrome_with_title(ui, card, title, meta, ink(SECONDARY, SURFACE));
}

fn paint_card_chrome_with_title(
    ui: &mut Ui<'_>,
    card: Rect,
    title: &str,
    meta: &str,
    title_style: PaintStyle,
) {
    ui.fill(card, ink(WHITE, SURFACE));
    if card.height == 0 || card.width < 4 {
        return;
    }
    let text_x = card.x.saturating_add(2);
    let text_w = card.width.saturating_sub(4);
    ui.paint_str(Rect::new(text_x, card.y, text_w, 1), title, title_style);
    if !meta.is_empty() {
        let meta_w = width(meta);
        if meta_w < text_w {
            ui.paint_str(
                Rect::new(
                    text_x.saturating_add(text_w.saturating_sub(meta_w)),
                    card.y,
                    meta_w,
                    1,
                ),
                meta,
                ink(FAINT, SURFACE),
            );
        }
    }
    if card.height > 1 && text_w > 0 {
        paint_repeat(
            ui,
            text_x,
            card.y.saturating_add(1),
            text_w,
            "─",
            ink(RULE, SURFACE),
        );
    }
}

fn card_inner(card: Rect) -> Rect {
    Rect {
        x: card.x.saturating_add(2),
        y: card.y.saturating_add(2),
        width: card.width.saturating_sub(4),
        height: card.height.saturating_sub(3),
    }
}

fn columns_fit(mins: &[u16], avail: u16) -> usize {
    let mut used = 0u16;
    let mut fit = 0usize;
    for &min in mins {
        let need = if fit == 0 {
            min
        } else {
            min.saturating_add(COL_GAP)
        };
        if used.saturating_add(need) > avail {
            break;
        }
        used = used.saturating_add(need);
        fit += 1;
    }
    fit
}

fn grow(specs: &[Spec], avail: u16) -> Vec<u16> {
    if specs.is_empty() {
        return Vec::new();
    }
    let gaps = COL_GAP.saturating_mul(u16::try_from(specs.len().saturating_sub(1)).unwrap_or(0));
    let budget = avail.saturating_sub(gaps);
    let mut widths: Vec<u16> = specs.iter().map(|spec| spec.min).collect();
    let fixed = specs
        .iter()
        .zip(widths.iter())
        .filter(|(spec, _)| !spec.flex)
        .map(|(_, width)| *width)
        .fold(0u16, u16::saturating_add);
    let mut room = budget.saturating_sub(fixed);
    let mut flex: Vec<usize> = specs
        .iter()
        .enumerate()
        .filter(|(_, spec)| spec.flex)
        .map(|(index, _)| index)
        .collect();
    if flex.is_empty() {
        if let Some(last) = widths.last_mut() {
            *last = last.saturating_add(room);
        }
        return widths;
    }
    while let Some(count) = u16::try_from(flex.len()).ok().filter(|count| *count > 0) {
        let share = room / count;
        let pin = flex.iter().copied().find(|index| widths[*index] > share);
        if let Some(index) = pin {
            room = room.saturating_sub(widths[index]);
            flex.retain(|item| *item != index);
            continue;
        }
        let mut remainder = room % count;
        for index in flex {
            widths[index] = share.saturating_add(u16::from(remainder > 0));
            remainder = remainder.saturating_sub(u16::from(remainder > 0));
        }
        break;
    }
    widths
}

fn table_geom(area: Rect, rows: usize) -> Option<Geom> {
    if area.height < 2 || area.width <= TABLE_CHROME.saturating_add(CHECK_W) {
        return None;
    }
    let mut has_sb = rows > usize::from(area.height.saturating_sub(1));
    let mut cols_w = area
        .width
        .saturating_sub(TABLE_CHROME)
        .saturating_sub(CHECK_W)
        .saturating_sub(u16::from(has_sb));
    let mins: Vec<u16> = SPECS.iter().map(|spec| spec.min).collect();
    let mut shown = columns_fit(&mins, cols_w);
    let has_bar = rows > 0 && area.height > 2 && shown < SPECS.len();
    let body = if has_bar {
        let body = area.height.saturating_sub(2);
        if !has_sb && rows > usize::from(body) {
            has_sb = true;
            cols_w = area
                .width
                .saturating_sub(TABLE_CHROME)
                .saturating_sub(CHECK_W)
                .saturating_sub(1);
            shown = columns_fit(&mins, cols_w);
        }
        body
    } else {
        area.height.saturating_sub(1)
    };
    Some(Geom {
        cols_w,
        has_sb,
        has_bar,
        body,
        widths: grow(&SPECS[..shown], cols_w),
        shown,
    })
}

fn thumb(offset: usize, viewport: usize, content: usize, track: usize) -> (usize, usize) {
    if viewport == 0 || track == 0 || content <= viewport {
        return (0, track);
    }
    let len = (viewport.saturating_mul(track) / content).max(1).min(track);
    let max_off = content.saturating_sub(viewport);
    let start = offset
        .saturating_mul(track.saturating_sub(len))
        .saturating_add(max_off / 2)
        / max_off.max(1);
    (start.min(track.saturating_sub(len)), len)
}

fn row_fade(index: usize, viewport: usize, content: usize) -> Option<f32> {
    if viewport < 4 || content <= viewport {
        return None;
    }
    let depth = usize::from(viewport >= 12) + 1;
    let down = viewport < content;
    if down && index + 1 == viewport {
        return Some(FADE_OUTER);
    }
    if down && depth == 2 && index + 2 == viewport {
        return Some(0.8);
    }
    None
}

fn paint_vertical_bar(ui: &mut Ui<'_>, area: Rect, rows: usize, viewport: u16, bg: u32) {
    if area.is_empty() {
        return;
    }
    let (start, len) = thumb(0, usize::from(viewport), rows, usize::from(area.height));
    for row in 0..area.height {
        let on = usize::from(row) >= start && usize::from(row) < start.saturating_add(len);
        paint_at(
            ui,
            area.x,
            area.y.saturating_add(row),
            1,
            if on { "┃" } else { "┆" },
            if on { ink(THUMB, bg) } else { ink(TRACK, bg) },
        );
    }
}

fn paint_horizontal_bar(ui: &mut Ui<'_>, area: Rect, shown: usize, columns: usize) {
    if area.is_empty() {
        return;
    }
    let (start, len) = thumb(0, shown, columns, usize::from(area.width));
    for col in 0..area.width {
        let on = usize::from(col) >= start && usize::from(col) < start.saturating_add(len);
        paint_at(
            ui,
            area.x.saturating_add(col),
            area.y,
            1,
            if on { "━" } else { "┄" },
            if on {
                ink(THUMB, SURFACE)
            } else {
                ink(TRACK, SURFACE)
            },
        );
    }
}

fn status_fg(status: Status) -> u32 {
    match status {
        Status::Active => WHITE,
        Status::Invited => SECONDARY,
        Status::Suspended => WARNING,
    }
}

fn status_text(status: Status) -> &'static str {
    match status {
        Status::Active => "Active",
        Status::Invited => "Invited",
        Status::Suspended => "Suspended",
    }
}

fn member_cells(member: &Member) -> [Cell; 7] {
    let text = if member.disabled { MUTED } else { WHITE };
    let call = if member.on_call {
        Cell {
            text: "Yes",
            fg: text,
        }
    } else {
        Cell {
            text: EMPTY_MARK,
            fg: if member.disabled { MUTED } else { FAINT },
        }
    };
    let notes = if member.notes.is_empty() {
        EMPTY_MARK
    } else {
        member.notes
    };
    [
        Cell {
            text: member.name,
            fg: text,
        },
        Cell {
            text: member.role,
            fg: text,
        },
        Cell {
            text: status_text(member.status),
            fg: if member.disabled {
                MUTED
            } else {
                status_fg(member.status)
            },
        },
        Cell {
            text: member.region,
            fg: text,
        },
        call,
        Cell {
            text: member.email,
            fg: text,
        },
        Cell {
            text: notes,
            fg: text,
        },
    ]
}

fn paint_header(ui: &mut Ui<'_>, area: Rect, geom: &Geom, any_selected: bool) {
    paint_at(
        ui,
        area.x.saturating_add(TABLE_INSET),
        area.y,
        3,
        if any_selected { CHECK_MIXED } else { CHECK_OFF },
        if any_selected {
            ink(ACCENT, SURFACE)
        } else {
            ink(MUTED, SURFACE)
        },
    );
    let mut x = area.x.saturating_add(TABLE_INSET).saturating_add(CHECK_W);
    for (spec, col_w) in SPECS.iter().zip(geom.widths.iter().copied()) {
        paint_at(
            ui,
            x,
            area.y,
            col_w,
            &fit(spec.title, col_w),
            ink(MUTED, SURFACE),
        );
        x = x.saturating_add(col_w).saturating_add(COL_GAP);
    }
    if geom.shown < SPECS.len() {
        paint_at(
            ui,
            area.x
                .saturating_add(TABLE_INSET)
                .saturating_add(CHECK_W)
                .saturating_add(geom.cols_w)
                .saturating_add(1),
            area.y,
            1,
            "…",
            ink(FAINT, SURFACE),
        );
    }
}

fn paint_member_row(
    ui: &mut Ui<'_>,
    area: Rect,
    member: &Member,
    geom: &Geom,
    keep: Option<f32>,
    model: &Model,
    index: usize,
) {
    if area.is_empty() {
        return;
    }
    let selected = model.selected & (1_u16 << index) != 0;
    let cursor = model.table_focus && model.cursor == index;
    if selected {
        // Accent selection fill (owner decision 2026-10-17, `InactiveSelection
        // ::Accent`): the chosen row keeps its accent fill and bold ink while
        // focus lives elsewhere in the same layer.
        ui.fill(
            area,
            ink(WHITE, accent_tint_bg(ui.theme_ref())).add_modifier(Modifier::BOLD),
        );
    } else if keep.is_some() {
        ui.fill(area, ink(faded(WHITE, SURFACE, keep), SURFACE));
    }
    let bg = if selected {
        accent_tint_bg(ui.theme_ref())
    } else {
        SURFACE
    };
    let row_ink = |fg: u32| ink(faded(fg, SURFACE, keep), bg).add_modifier(Modifier::BOLD);
    if cursor {
        ui.glyph(
            Rect::new(area.x, area.y, 1, 1),
            GlyphRole::FocusBar,
            ink(ACCENT, bg).add_modifier(Modifier::BOLD),
        );
    } else if !selected {
        paint_at(ui, area.x, area.y, 1, " ", ink(SURFACE, SURFACE));
    }
    let mark = if selected {
        row_ink(ACCENT)
    } else {
        let mut mark = ink(
            faded(
                if member.disabled { DISABLED } else { MUTED },
                SURFACE,
                keep,
            ),
            bg,
        );
        if member.disabled {
            mark = mark.add_modifier(Modifier::DIM);
        }
        mark
    };
    let check = if selected { CHECK_ON } else { CHECK_OFF };
    paint_at(
        ui,
        area.x.saturating_add(TABLE_INSET),
        area.y,
        3,
        check,
        mark,
    );
    let cells = member_cells(member);
    let mut x = area.x.saturating_add(TABLE_INSET).saturating_add(CHECK_W);
    for (col, col_w) in geom.widths.iter().copied().enumerate() {
        let Some(cell) = cells.get(col) else {
            break;
        };
        let style = if selected {
            row_ink(cell.fg)
        } else {
            ink(faded(cell.fg, SURFACE, keep), bg)
        };
        paint_at(ui, x, area.y, col_w, &fit(cell.text, col_w), style);
        x = x.saturating_add(col_w).saturating_add(COL_GAP);
    }
}

fn members_meta(end: usize, selected: usize) -> String {
    let range = if end < MEMBERS.len() && end > 0 {
        format!("1\u{2013}{end} of {}", MEMBERS.len())
    } else {
        String::new()
    };
    if selected == 0 {
        range
    } else if range.is_empty() {
        format!("{selected} selected")
    } else {
        format!("{selected} selected \u{b7} {range}")
    }
}

fn paint_members(ui: &mut Ui<'_>, card: Rect, model: &Model) {
    let inner = card_inner(card);
    let Some(geom) = table_geom(inner, MEMBERS.len()) else {
        paint_card_chrome(ui, card, "Members", "");
        return;
    };
    let end = usize::from(geom.body).min(MEMBERS.len());
    let meta = members_meta(end, model.selected_count());
    let mut title = ink(SECONDARY, SURFACE);
    if model.table_focus {
        // The focused table's card is the active panel: its title carries the
        // emphasis while selection keeps the accent (`InactiveSelection
        // ::Accent`), like a macOS selection in the active window.
        title = ink(WHITE, SURFACE).add_modifier(Modifier::BOLD);
    }
    paint_card_chrome_with_title(ui, card, "Members", &meta, title);
    paint_header(ui, Rect { height: 1, ..inner }, &geom, model.selected != 0);
    let row_w = inner.width.saturating_sub(u16::from(geom.has_sb));
    for (index, member) in MEMBERS.iter().enumerate().take(end) {
        paint_member_row(
            ui,
            Rect::new(
                inner.x,
                inner
                    .y
                    .saturating_add(1)
                    .saturating_add(u16::try_from(index).unwrap_or(u16::MAX)),
                row_w,
                1,
            ),
            member,
            &geom,
            row_fade(index, usize::from(geom.body), MEMBERS.len()),
            model,
            index,
        );
    }
    if geom.has_sb && geom.body > 0 {
        paint_vertical_bar(
            ui,
            Rect::new(
                inner.right().saturating_sub(1),
                inner.y.saturating_add(1),
                1,
                geom.body,
            ),
            MEMBERS.len(),
            geom.body,
            SURFACE,
        );
    }
    if geom.has_bar {
        let x = inner.x.saturating_add(TABLE_INSET).saturating_add(CHECK_W);
        let right = inner.right().saturating_sub(u16::from(geom.has_sb));
        paint_horizontal_bar(
            ui,
            Rect::new(
                x,
                inner.y.saturating_add(1).saturating_add(geom.body),
                right.saturating_sub(x),
                1,
            ),
            geom.shown,
            SPECS.len(),
        );
    }
}

fn paint_centered(ui: &mut Ui<'_>, area: Rect, y: u16, text: &str, style: PaintStyle) {
    if y < area.y || y >= area.bottom() || area.width == 0 {
        return;
    }
    let columns = width(text).min(area.width);
    let x = area
        .x
        .saturating_add(area.width.saturating_sub(columns) / 2);
    paint_at(ui, x, y, columns, text, style);
}

/// What the selected members agree on. `None` means the column is mixed
/// across the selection and the field shows `— mixed —`.
struct Summary {
    count: usize,
    role: Option<&'static str>,
    status: Option<&'static str>,
    region: Option<&'static str>,
    on_call: Option<bool>,
    notes: Option<&'static str>,
}

impl Summary {
    fn collect(model: &Model) -> Option<Summary> {
        let count = model.selected_count();
        if count == 0 {
            return None;
        }
        let mut summary: Option<Summary> = None;
        for (index, member) in MEMBERS.iter().enumerate() {
            if model.selected & (1_u16 << index) == 0 {
                continue;
            }
            let notes = if member.notes.is_empty() {
                None
            } else {
                Some(member.notes)
            };
            match &mut summary {
                None => {
                    summary = Some(Summary {
                        count,
                        role: Some(member.role),
                        status: Some(status_text(member.status)),
                        region: Some(member.region),
                        on_call: Some(member.on_call),
                        notes,
                    });
                }
                Some(shared) => {
                    unify(&mut shared.role, member.role);
                    unify(&mut shared.status, status_text(member.status));
                    unify(&mut shared.region, member.region);
                    unify(&mut shared.on_call, member.on_call);
                    // A member without notes makes the shared note mixed.
                    if let Some(note) = notes {
                        unify(&mut shared.notes, note);
                    } else {
                        shared.notes = None;
                    }
                }
            }
        }
        summary
    }
}

/// Narrows a shared value to `None` (mixed) once members disagree. `None`
/// here means "differs", so it never flips back to a value.
fn unify<T: PartialEq>(slot: &mut Option<T>, value: T) {
    if let Some(current) = slot
        && *current != value
    {
        *slot = None;
    }
}

const MIXED: &str = "\u{2014} mixed \u{2014}";

/// One bulk-edit field: a raised chip carrying the shared value (or the
/// mixed marker), with a dropdown arrow when the field is editable.
fn paint_field(ui: &mut Ui<'_>, area: Rect, value: &str, arrow: bool) {
    if area.is_empty() {
        return;
    }
    let field_bg = color_u32(ui.theme_ref().bg(Surface::Field));
    // The chip leads with one cell inked in its own background, then a
    // white-ink body; the value and the dropdown arrow overwrite their
    // cells on top.
    paint_at(ui, area.x, area.y, 1, " ", ink(field_bg, field_bg));
    if area.width > 1 {
        ui.fill(
            Rect::new(area.x.saturating_add(1), area.y, area.width - 1, 1),
            ink(WHITE, field_bg),
        );
    }
    paint_at(
        ui,
        area.x.saturating_add(2),
        area.y,
        area.width.saturating_sub(2),
        value,
        ink(MUTED, field_bg),
    );
    if arrow && area.width > 2 {
        paint_at(
            ui,
            area.x.saturating_add(area.width).saturating_sub(2),
            area.y,
            1,
            "\u{25be}",
            ink(SECONDARY, field_bg),
        );
    }
}

fn paint_editor(ui: &mut Ui<'_>, card: Rect, model: &Model) {
    let Some(summary) = Summary::collect(model) else {
        paint_card_chrome(ui, card, "Details", "");
        paint_editor_hint(ui, card);
        return;
    };
    let title = if summary.count == 1 {
        String::from("Edit 1 member")
    } else {
        format!("Edit {} members", summary.count)
    };
    paint_card_chrome(ui, card, &title, "");
    let inner = card_inner(card);
    if inner.height == 0 || inner.width == 0 || inner.width < 16 {
        return;
    }
    let applies = if summary.count == 1 {
        String::from("Applies to 1 member \u{b7} names and emails stay")
    } else {
        format!(
            "Applies to all {} \u{b7} names and emails stay",
            summary.count
        )
    };
    paint_at(
        ui,
        inner.x.saturating_add(2),
        inner.y,
        inner.width - 2,
        &applies,
        ink(FAINT, SURFACE),
    );
    // Two field columns with a two-column gap; the right chip absorbs the
    // odd column when the card width is uneven.
    let chip_w = inner.width.saturating_sub(2) / 2;
    let left = inner.x;
    let right = left.saturating_add(chip_w).saturating_add(2);
    let right_w = inner.width.saturating_sub(2).saturating_sub(chip_w);
    let label = ink(SECONDARY, SURFACE);
    paint_at(
        ui,
        left.saturating_add(2),
        inner.y.saturating_add(1),
        10,
        "Role",
        label,
    );
    paint_at(
        ui,
        right.saturating_add(2),
        inner.y.saturating_add(1),
        10,
        "Status",
        label,
    );
    let values_y = inner.y.saturating_add(2);
    paint_field(
        ui,
        Rect::new(left, values_y, chip_w, 1),
        summary.role.unwrap_or(MIXED),
        true,
    );
    paint_field(
        ui,
        Rect::new(right, values_y, right_w, 1),
        summary.status.unwrap_or(MIXED),
        true,
    );
    paint_at(
        ui,
        left.saturating_add(2),
        inner.y.saturating_add(4),
        10,
        "Region",
        label,
    );
    paint_field(
        ui,
        Rect::new(left, inner.y.saturating_add(5), chip_w, 1),
        summary.region.unwrap_or(MIXED),
        true,
    );
    let check_x = right.saturating_add(1);
    let (check, check_fg) = match summary.on_call {
        Some(true) => (CHECK_ON, ACCENT),
        Some(false) => (CHECK_OFF, MUTED),
        None => (CHECK_MIXED, ACCENT),
    };
    // One clearing cell keeps the card fill behind the checkbox inset.
    paint_at(
        ui,
        right,
        inner.y.saturating_add(5),
        1,
        " ",
        ink(SURFACE, SURFACE),
    );
    paint_at(
        ui,
        check_x,
        inner.y.saturating_add(5),
        3,
        check,
        ink(check_fg, SURFACE),
    );
    paint_at(
        ui,
        check_x.saturating_add(4),
        inner.y.saturating_add(5),
        7,
        "On call",
        ink(WHITE, SURFACE),
    );
    // The Notes label runs to the card edge, unlike the other labels.
    paint_at(
        ui,
        left.saturating_add(2),
        inner.y.saturating_add(7),
        inner.width.saturating_sub(2),
        &fit("Notes", inner.width.saturating_sub(2)),
        label,
    );
    // A lone member shows its own (possibly empty) note, not "mixed".
    let notes = if summary.count == 1 {
        summary.notes.unwrap_or(EMPTY_MARK)
    } else {
        summary.notes.unwrap_or(MIXED)
    };
    paint_field(
        ui,
        Rect::new(left, inner.y.saturating_add(8), inner.width, 1),
        notes,
        false,
    );
}

fn paint_editor_hint(ui: &mut Ui<'_>, card: Rect) {
    let inner = card_inner(card);
    if inner.height == 0 || inner.width == 0 {
        return;
    }
    let wrap_at = inner.width.saturating_sub(4).max(8);
    let lines = wrap(HINT, wrap_at);
    let extra = if lines.is_empty() {
        0
    } else {
        u16::try_from(lines.len())
            .unwrap_or(u16::MAX)
            .saturating_add(1)
    };
    let total = 1u16.saturating_add(extra);
    let y0 = inner
        .y
        .saturating_add(inner.height.saturating_sub(total) / 2);
    paint_centered(ui, inner, y0, "No member selected", ink(MUTED, SURFACE));
    for (index, line) in lines.iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        paint_centered(
            ui,
            inner,
            y0.saturating_add(2).saturating_add(offset),
            line,
            ink(FAINT, SURFACE),
        );
    }
}

fn wide_page(area: Rect) -> bool {
    area.width == WIDE_PAGE_W && area.height == WIDE_PAGE_H
}

fn content_height() -> u16 {
    MEMBERS_H.saturating_add(EDITOR_H).saturating_add(CARD_GAP)
}

fn paint_wide_playground(ui: &mut Ui<'_>, area: Rect, model: &Model) {
    if area.width <= CARD_GAP {
        return;
    }
    // Details keeps the right third. Members take the rest of the row.
    let right_w = area.width / 3;
    let left_w = area.width.saturating_sub(CARD_GAP).saturating_sub(right_w);
    if left_w == 0 || right_w == 0 {
        paint_playground(ui, area, model);
        return;
    }
    paint_members(ui, Rect::new(area.x, area.y, left_w, area.height), model);
    paint_editor(
        ui,
        Rect::new(
            area.x.saturating_add(left_w).saturating_add(CARD_GAP),
            area.y,
            right_w,
            EDITOR_H.min(area.height),
        ),
        model,
    );
}

fn paint_playground(ui: &mut Ui<'_>, area: Rect, model: &Model) {
    if area.is_empty() {
        return;
    }
    let content_h = content_height();
    let overflows = area.width > SCROLL_ROOM && content_h > area.height;
    let view = if overflows {
        Rect {
            width: area.width.saturating_sub(SCROLL_ROOM),
            ..area
        }
    } else {
        area
    };
    let members = Rect::new(
        view.x,
        view.y,
        view.width,
        MEMBERS_H.min(view.bottom().saturating_sub(view.y)),
    );
    if members.height > 0 {
        paint_members(ui, Rect::new(view.x, view.y, view.width, MEMBERS_H), model);
    }
    let editor_y = view.y.saturating_add(MEMBERS_H).saturating_add(CARD_GAP);
    if editor_y < view.bottom() {
        paint_editor(ui, Rect::new(view.x, editor_y, view.width, EDITOR_H), model);
    }
    if overflows {
        paint_vertical_bar(
            ui,
            Rect::new(area.right().saturating_sub(1), area.y, 1, area.height),
            usize::from(content_h),
            area.height,
            CANVAS,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, mods: KeyModifiers) -> Key {
        Key { code, mods }
    }

    #[test]
    fn down_space_and_shift_down_select_ari_then_neighbors() {
        let mut model = Model {
            table_focus: true,
            ..Model::default()
        };
        assert!(model.apply_key(key(KeyCode::Down, KeyModifiers::empty())));
        assert!(model.apply_key(key(KeyCode::Char(' '), KeyModifiers::empty())));
        assert_eq!(model.selected_count(), 1);
        assert_eq!(model.cursor, 1);
        assert!(model.apply_key(key(KeyCode::Down, KeyModifiers::SHIFT)));
        assert_eq!(model.selected_count(), 2);
        // Delete requests now flow through `update` (the dialog layer), not
        // through `apply_key`; the key itself is not a selection edit.
        assert!(!model.apply_key(key(KeyCode::Char('d'), KeyModifiers::empty())));
        assert!(!model.dialog);
        assert_eq!(model.selected_count(), 2);

        let mut bulk = Model {
            table_focus: true,
            ..Model::default()
        };
        assert!(bulk.apply_key(key(KeyCode::Down, KeyModifiers::empty())));
        assert!(bulk.apply_key(key(KeyCode::Char(' '), KeyModifiers::empty())));
        assert!(bulk.apply_key(key(KeyCode::Down, KeyModifiers::SHIFT)));
        assert!(bulk.apply_key(key(KeyCode::Down, KeyModifiers::SHIFT)));
        assert_eq!(bulk.selected_count(), 3);
        assert_eq!(bulk.cursor, 3);
    }

    #[test]
    fn space_on_the_disabled_owner_does_not_select() {
        let mut model = Model {
            table_focus: true,
            ..Model::default()
        };
        assert!(model.apply_key(key(KeyCode::Char(' '), KeyModifiers::empty())));
        assert_eq!(model.selected_count(), 0);
        assert!(!model.dialog);
    }

    #[test]
    fn ctrl_a_selects_all_enabled_and_toggles_back() {
        let mut model = Model {
            table_focus: true,
            ..Model::default()
        };
        let enabled = MEMBERS.iter().filter(|m| !m.disabled).count();
        assert!(model.apply_key(key(KeyCode::Char('a'), KeyModifiers::CONTROL)));
        assert_eq!(model.selected_count(), enabled);
        assert!(model.apply_key(key(KeyCode::Char('a'), KeyModifiers::CONTROL)));
        assert_eq!(model.selected_count(), 0);
    }

    #[test]
    fn esc_clears_the_selection() {
        let mut model = Model {
            table_focus: true,
            ..Model::default()
        };
        assert!(model.apply_key(key(KeyCode::Down, KeyModifiers::empty())));
        assert!(model.apply_key(key(KeyCode::Char(' '), KeyModifiers::empty())));
        assert_eq!(model.selected_count(), 1);
        assert!(model.apply_key(key(KeyCode::Esc, KeyModifiers::empty())));
        assert_eq!(model.selected_count(), 0);
        assert!(!model.apply_key(key(KeyCode::Esc, KeyModifiers::empty())));
    }

    #[test]
    fn summary_reports_shared_and_mixed_fields() {
        let mut model = Model {
            table_focus: true,
            ..Model::default()
        };
        assert!(Summary::collect(&model).is_none());
        // Cursor starts on the disabled owner; move to Ari first.
        assert!(model.apply_key(key(KeyCode::Down, KeyModifiers::empty())));
        assert!(model.apply_key(key(KeyCode::Char(' '), KeyModifiers::empty())));
        let one = Summary::collect(&model).expect("one member");
        assert_eq!(one.count, 1);
        assert_eq!(one.role, Some(MEMBERS[1].role));
        assert_eq!(one.status, Some(status_text(MEMBERS[1].status)));
        assert_eq!(one.notes, Some(MEMBERS[1].notes));
        // Extend to Cleo: same status, different role, no notes on Bo? no --
        // Cleo has notes; roles differ (Admin vs Editor).
        assert!(model.apply_key(key(KeyCode::Down, KeyModifiers::SHIFT)));
        let two = Summary::collect(&model).expect("two members");
        assert_eq!(two.count, 2);
        assert_eq!(two.role, None);
        assert_eq!(two.status, Some(status_text(MEMBERS[2].status)));
        assert_eq!(two.notes, None);
    }

    #[test]
    fn hints_follow_dialog_selection_and_focus() {
        let mut model = Model::default();
        assert_eq!(model.hints(), &[]);
        model.table_focus = true;
        assert_eq!(model.hints(), TABLE_HINTS);
        assert!(model.apply_key(key(KeyCode::Down, KeyModifiers::empty())));
        assert!(model.apply_key(key(KeyCode::Char(' '), KeyModifiers::empty())));
        assert_eq!(model.hints(), SELECTION_HINTS);
        model.dialog = true;
        assert_eq!(model.hints(), DIALOG_HINTS);
    }

    #[test]
    fn dialog_fields_track_the_selection_count() {
        let mut model = Model {
            table_focus: true,
            ..Model::default()
        };
        for _ in 0..3 {
            assert!(model.apply_key(key(KeyCode::Down, KeyModifiers::empty())));
            assert!(model.apply_key(key(KeyCode::Char(' '), KeyModifiers::empty())));
        }
        assert_eq!(model.selected_count(), 3);
        // `open_dialog` formats the title/confirm from the live selection.
        let count = model.selected_count();
        assert_eq!(format!("Delete {count} members?"), "Delete 3 members?");
        assert_eq!(format!("Delete {count} members"), "Delete 3 members");
    }
}
