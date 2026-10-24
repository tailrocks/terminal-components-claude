use termrock::author::PaintStyle;
use termrock::{
    Align, BlurPolicy, Button, CellRef, Color, Column, ColumnKey, Cx, Dialog, DialogAction,
    DialogState, EditIntent, FgStep, FieldError, FrameRead, GlyphRole, Grid, GridEditor, GridModel,
    GridState, Id, ItemKey, List, ListAction, ListState, Modifier, NavUnit, Part, Rect, Response,
    Role, RowUi, StateFlags, StylePatch, Tabs, TabsAction, TabsState, Ui, Variant, truncate_parts,
    width,
};

use super::{Page, PageUpdate, frame, paint_boundary_meta};

const TAB: Id = termrock::Id::root("showcase_ui::pages::settings::settings.tabs");
const MEMBERS: Id = termrock::Id::root("showcase_ui::pages::settings::settings.members");
const INVITE: Id = termrock::Id::root("showcase_ui::pages::settings::settings.invite");
const REMOVE: Id = termrock::Id::root("showcase_ui::pages::settings::settings.remove");
const MEMBER_GRID: Id = termrock::Id::root("showcase_ui::pages::settings::settings.member_grid");
const MEMBER_BACK: Id = termrock::Id::root("showcase_ui::pages::settings::settings.member_back");
const REMOVE_DIALOG: Id =
    termrock::Id::root("showcase_ui::pages::settings::settings.remove.dialog");
const TABS: &[&str] = &["General", "Members", "Security"];

#[derive(Clone, Debug, PartialEq, Eq)]
struct Member {
    id: u8,
    name: &'static str,
    edited_name: Option<String>,
    email: &'static str,
    role: MemberRole,
    last_active: &'static str,
    account_kind: AccountKind,
    lifecycle: MemberLifecycle,
}

impl Member {
    fn display_name(&self) -> &str {
        self.edited_name.as_deref().unwrap_or(self.name)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemberRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl MemberRole {
    const fn label(self) -> &'static str {
        match self {
            Self::Owner => "Owner",
            Self::Admin => "Admin",
            Self::Member => "Member",
            Self::Viewer => "Viewer",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "Owner" => Some(Self::Owner),
            "Admin" => Some(Self::Admin),
            "Member" => Some(Self::Member),
            "Viewer" => Some(Self::Viewer),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AccountKind {
    Person,
    Service,
}

impl AccountKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Person => "Person",
            Self::Service => "Service account",
        }
    }

    const fn glyph(self) -> GlyphRole {
        match self {
            Self::Person => GlyphRole::Bullet,
            Self::Service => GlyphRole::FollowRef,
        }
    }

    const fn marker_role(self) -> Role {
        match self {
            Self::Person => Role::Info,
            Self::Service => Role::Accent,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemberLifecycle {
    Active,
    InvitationPending,
}

impl MemberLifecycle {
    const fn label(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::InvitationPending => "invitation pending",
        }
    }

    const fn glyph(self) -> GlyphRole {
        match self {
            Self::Active => GlyphRole::DotOn,
            Self::InvitationPending => GlyphRole::DotOff,
        }
    }

    const fn color(self) -> Color {
        match self {
            Self::Active => Color::Green,
            Self::InvitationPending => Color::Yellow,
        }
    }
}

const INITIAL_MEMBERS: &[Member] = &[
    Member {
        id: 1,
        name: "Mira Okafor",
        edited_name: None,
        email: "mira@acme.dev",
        role: MemberRole::Owner,
        last_active: "today",
        account_kind: AccountKind::Person,
        lifecycle: MemberLifecycle::Active,
    },
    Member {
        id: 2,
        name: "Jonas Weber",
        edited_name: None,
        email: "jonas@acme.dev",
        role: MemberRole::Admin,
        last_active: "2 h ago",
        account_kind: AccountKind::Person,
        lifecycle: MemberLifecycle::Active,
    },
    Member {
        id: 3,
        name: "Ana Costa",
        edited_name: None,
        email: "ana@acme.dev",
        role: MemberRole::Member,
        last_active: "yesterday",
        account_kind: AccountKind::Person,
        lifecycle: MemberLifecycle::Active,
    },
    Member {
        id: 4,
        name: "Kai Tanaka",
        edited_name: None,
        email: "kai@acme.dev",
        role: MemberRole::Member,
        last_active: "3 d ago",
        account_kind: AccountKind::Person,
        lifecycle: MemberLifecycle::Active,
    },
    Member {
        id: 5,
        name: "Sofia Rossi",
        edited_name: None,
        email: "sofia@acme.dev",
        role: MemberRole::Viewer,
        last_active: "never",
        account_kind: AccountKind::Person,
        lifecycle: MemberLifecycle::InvitationPending,
    },
    Member {
        id: 6,
        name: "deploy-bot",
        edited_name: None,
        email: "bot@acme.dev",
        role: MemberRole::Member,
        last_active: "1 m ago",
        account_kind: AccountKind::Service,
        lifecycle: MemberLifecycle::Active,
    },
];

const MEMBER_STATUS_MAX_WIDTH: u16 = 20;
const MEMBER_LABEL_AND_MARKER_MIN_WIDTH: u16 = 20;

fn member_key(member: &Member) -> ItemKey {
    ItemKey::num(u64::from(member.id))
}
fn member_row(member: &Member, row: &mut RowUi<'_>) {
    row.meta(member.email);

    let status_width = row
        .remaining_width()
        .saturating_sub(MEMBER_LABEL_AND_MARKER_MIN_WIDTH)
        .min(MEMBER_STATUS_MAX_WIDTH);
    if status_width > 0 {
        let mut status = row.part(Part::META, status_width);
        // DotOn and DotOff each reserve one terminal cell at the end.
        let text_width = status.available_width().saturating_sub(1);
        let (head, ellipsis) = truncate_parts(member.lifecycle.label(), text_width);
        status.text(head).text(ellipsis).suffix_style(
            member.lifecycle.glyph(),
            PaintStyle::new().fg(member.lifecycle.color()),
        );
    }

    let marker_patch = StylePatch::new().set_fg(member.account_kind.marker_role());
    row.marker_patched(member.account_kind.glyph(), &marker_patch);
    row.label(member.account_kind.label());
    row.label(" · ");
    row.label(member.display_name());
}
fn member_list()
-> List<'static, Member, impl Fn(&Member) -> ItemKey, impl Fn(&Member, &mut RowUi<'_>)> {
    List::new(MEMBERS).key(member_key).row(member_row)
}

const MEMBER_NAME_COLUMN: ColumnKey = ColumnKey::num(0);
const MEMBER_EMAIL_COLUMN: ColumnKey = ColumnKey::num(1);
const MEMBER_ROLE_COLUMN: ColumnKey = ColumnKey::num(2);
const MEMBER_LAST_ACTIVE_COLUMN: ColumnKey = ColumnKey::num(3);

const MEMBER_COLUMNS: [Column<'static>; 4] = [
    Column {
        key: MEMBER_NAME_COLUMN,
        title: "Name",
        subtitle: None,
        align: Align::Left,
        min_width: 18,
        max_width: 32,
        sortable: false,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
    Column {
        key: MEMBER_EMAIL_COLUMN,
        title: "Email",
        subtitle: None,
        align: Align::Left,
        min_width: 22,
        max_width: 32,
        sortable: false,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
    Column {
        key: MEMBER_ROLE_COLUMN,
        title: "Role",
        subtitle: None,
        align: Align::Left,
        min_width: 10,
        max_width: 12,
        sortable: false,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
    Column {
        key: MEMBER_LAST_ACTIVE_COLUMN,
        title: "Last active",
        subtitle: None,
        align: Align::Left,
        min_width: 12,
        max_width: 16,
        sortable: false,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
];

struct MemberGridModel<'a, M> {
    members: M,
    row_ids: &'a [u8],
    committed: Option<&'a mut bool>,
}

impl<M: AsRef<[Member]>> MemberGridModel<'_, M> {
    fn member(&self, row: usize) -> Option<&Member> {
        let id = *self.row_ids.get(row)?;
        self.members.as_ref().iter().find(|member| member.id == id)
    }
}

impl<M: AsRef<[Member]>> GridModel for MemberGridModel<'_, M> {
    fn row_count(&self) -> usize {
        self.row_ids.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        self.row_ids
            .get(row)
            .copied()
            .map_or(ItemKey::num(0), |id| ItemKey::num(u64::from(id)))
    }

    fn cell(&self, row: usize, column: usize) -> Option<CellRef<'_>> {
        let member = self.member(row)?;
        Some(match column {
            0 => CellRef::new(member.display_name()),
            1 => CellRef::new(member.email),
            2 => CellRef::new(member.role.label()),
            3 => CellRef::new(member.last_active).tone(Role::Fg(FgStep::Muted)),
            _ => return None,
        })
    }
}

impl<M: AsRef<[Member]> + AsMut<[Member]>> GridEditor for MemberGridModel<'_, M> {
    fn edit_intent(&self, row: usize, column: usize) -> EditIntent<'_> {
        let Some(member) = self.member(row) else {
            return EditIntent::Refuse {
                reason: "Unknown member row",
            };
        };
        if column == 0 {
            EditIntent::Inline {
                initial: member.display_name(),
            }
        } else if column == 2 {
            EditIntent::Inline {
                initial: member.role.label(),
            }
        } else {
            EditIntent::Refuse {
                reason: "Email and last active are read-only",
            }
        }
    }

    fn apply_cycle(&mut self, _row: usize, _column: usize) {}

    fn commit_cell(&mut self, row: usize, column: usize, text: &str) -> Result<(), FieldError> {
        let id = *self
            .row_ids
            .get(row)
            .ok_or_else(|| FieldError::new("Unknown member row"))?;
        let member = self
            .members
            .as_mut()
            .iter_mut()
            .find(|member| member.id == id)
            .ok_or_else(|| FieldError::new("Unknown member row"))?;
        if column == 0 {
            if text.trim().is_empty() {
                return Err(FieldError::new("Name cannot be blank"));
            }
            member.edited_name = Some(text.to_owned());
        } else if column == 2 {
            let Some(role) = MemberRole::parse(text) else {
                return Err(FieldError::new(
                    "Role must be Owner, Admin, Member, or Viewer",
                ));
            };
            member.role = role;
        } else {
            return Err(FieldError::new("Email and last active are read-only"));
        }
        if let Some(committed) = self.committed.as_deref_mut() {
            *committed = true;
        }
        Ok(())
    }

    fn is_editable(&self, _row: usize, column: usize) -> bool {
        matches!(column, 0 | 2)
    }
}

fn member_grid() -> Grid<'static> {
    Grid::new(MEMBER_GRID, &MEMBER_COLUMNS)
        .nav(NavUnit::Cell)
        .blur(BlurPolicy::Keep)
}

fn member_back_button() -> Button<'static> {
    Button::new(MEMBER_BACK, "Back to members").variant(Variant::SECONDARY)
}

fn invite_button() -> Button<'static> {
    Button::new(INVITE, "Invite member").variant(Variant::SECONDARY)
}

fn remove_button(has_members: bool) -> Button<'static> {
    Button::new(REMOVE, "Remove member")
        .variant(Variant::DANGER)
        .disabled(!has_members)
}

fn remove_dialog() -> Dialog<'static> {
    Dialog::destructive(
        REMOVE_DIALOG,
        "Remove member?",
        "This member will lose access to the workspace.",
    )
}

const CANVAS: u32 = 0x0000_0000;
const SURFACE: u32 = 0x0011_1111;
const WHITE: u32 = 0x00ff_ffff;
const SECONDARY: u32 = 0x00b3_b3b3;
const MUTED: u32 = 0x008a_8a8a;
const FAINT: u32 = 0x0073_7373;
const RULE: u32 = 0x003a_3a3a;
const ACCENT: u32 = 0x0048_e054;
const TAB_ACTIVE: u32 = 0x0027_272a;
const FIELD: u32 = 0x001e_1e22;
const BUTTON_INK: u32 = 0x0019_191c;

const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SETTINGS_TABS: [&str; 3] = ["General", "Members", "Environment"];
const SECTION_CAPTION: &str = "the live widget";
const DESC: &str = "Handles checkout, invoicing and refunds for the storefront.";
const LEFT_NEED: u16 = 24;
const CARD_H: u16 = 16;
const COLUMN_GAP: u16 = 4;
const NARROW_W: u16 = 59;
const NARROW_GUTTER: u16 = 2;
const NARROW_SCROLL_CONTENT: u16 = 25;
const COMPACT_W: u16 = 51;
// 11×11/24 = 5 thumb rows on the 72×20 bar.
const COMPACT_SCROLL_CONTENT: u16 = 24;
const SCROLL_THUMB: u32 = 0x0080_8080;
const SCROLL_TRACK: u32 = 0x0047_4747;

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

fn plain(fg: u32, bg: u32) -> PaintStyle {
    ink(fg, bg).remove_modifier(Modifier::all())
}

fn bold(fg: u32, bg: u32) -> PaintStyle {
    ink(fg, bg).add_modifier(Modifier::BOLD)
}

fn active_tab(tabs: &TabsState) -> usize {
    match tabs.active().or(tabs.cursor()) {
        Some(ItemKey::Index(index)) => index,
        _ => 0,
    }
}

fn tab_span(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn right_need() -> u16 {
    width("Auto-merge approved PRs")
        .saturating_add(10)
        .max(width("Protect main branch").saturating_add(10))
        .max(width("Visibility").saturating_add(2))
        .max(width("Internal").saturating_add(6))
}

fn left_width(inner: u16) -> Option<u16> {
    let right = right_need();
    let half = (inner / 2).saturating_sub(2);
    if half >= LEFT_NEED && inner.saturating_sub(half.saturating_add(COLUMN_GAP)) >= right {
        Some(half)
    } else if inner >= LEFT_NEED.saturating_add(COLUMN_GAP).saturating_add(right) {
        Some(inner.saturating_sub(COLUMN_GAP).saturating_sub(right))
    } else {
        None
    }
}

fn paint_at(ui: &mut Ui<'_>, area: Rect, x: u16, y: u16, text: &str, style: PaintStyle) {
    let x = area.x.saturating_add(x);
    if y >= area.bottom() || x >= area.right() || text.is_empty() {
        return;
    }
    ui.paint_str(
        Rect {
            x,
            y,
            width: area.right().saturating_sub(x),
            height: 1,
        },
        text,
        style,
    );
}

fn narrow_frame(body: Rect) -> bool {
    body.width <= NARROW_W && body.width > COMPACT_W
}

fn compact_frame(body: Rect) -> bool {
    body.width <= COMPACT_W
}

fn paint_member_summary(ui: &mut Ui<'_>, body: Rect, members: &[Member]) {
    let mut line = format!("{} members", members.len());
    for member in members {
        line.push_str("  ");
        line.push_str(member.display_name());
        line.push(' ');
        line.push_str(member.email);
    }
    paint_at(
        ui,
        body,
        0,
        body.y.saturating_add(2),
        &line,
        ink(WHITE, CANVAS),
    );
}

fn paint_tab_strip(ui: &mut Ui<'_>, area: Rect, labels: &[&str], active: usize, caption: bool) {
    if area.is_empty() {
        return;
    }
    ui.fill(Rect { height: 1, ..area }, plain(WHITE, CANVAS));
    let mut x = area.x;
    for (index, label) in labels.iter().enumerate() {
        let span = tab_span(label);
        if x.saturating_add(span) > area.right() {
            break;
        }
        let style = if index == active {
            bold(WHITE, TAB_ACTIVE)
        } else {
            ink(SECONDARY, CANVAS)
        };
        ui.fill(
            Rect {
                x,
                y: area.y,
                width: span,
                height: 1,
            },
            style,
        );
        ui.paint_str(
            Rect {
                x: x.saturating_add(1),
                y: area.y,
                width: span.saturating_sub(1),
                height: 1,
            },
            label,
            style,
        );
        x = x.saturating_add(span).saturating_add(1);
    }
    if caption && area.width >= 76 {
        let caption_w = width(SECTION_CAPTION);
        let caption_x = area.right().saturating_sub(caption_w.saturating_add(1));
        if caption_x > area.x {
            ui.paint_str(
                Rect {
                    x: caption_x,
                    y: area.y,
                    width: caption_w,
                    height: 1,
                },
                SECTION_CAPTION,
                ink(FAINT, CANVAS),
            );
        }
    }
    if area.height < 2 {
        return;
    }
    let rule_y = area.y.saturating_add(1);
    ui.paint_str(
        Rect {
            y: rule_y,
            height: 1,
            ..area
        },
        &"─".repeat(usize::from(area.width)),
        ink(RULE, CANVAS),
    );
    let active_w = labels
        .get(active)
        .map(|label| tab_span(label))
        .unwrap_or(0)
        .min(area.width);
    if active_w > 0 {
        ui.paint_str(
            Rect {
                x: area.x,
                y: rule_y,
                width: active_w,
                height: 1,
            },
            &"━".repeat(usize::from(active_w)),
            ink(ACCENT, CANVAS),
        );
    }
}

fn paint_general(ui: &mut Ui<'_>, body: Rect, active: usize) {
    let narrow = narrow_frame(body);
    let compact = compact_frame(body);
    if let Some(strip) = row_band(body, 0, 2) {
        paint_tab_strip(ui, strip, &SECTIONS, 0, true);
    }
    let content = if narrow || compact {
        Rect {
            width: body.width.saturating_sub(NARROW_GUTTER),
            ..body
        }
    } else {
        body
    };
    if let Some(strip) = row_band(content, 3, 2) {
        paint_tab_strip(ui, strip, &SETTINGS_TABS, active, false);
    }
    let Some(card) = row_band(content, 6, CARD_H.min(content.height.saturating_sub(6))) else {
        return;
    };
    paint_card(ui, card, narrow, compact);
    if narrow {
        paint_narrow_bar(ui, body);
    } else if compact {
        paint_scroll_gutter(ui, body, COMPACT_SCROLL_CONTENT);
    }
}

fn paint_narrow_bar(ui: &mut Ui<'_>, body: Rect) {
    paint_scroll_gutter(ui, body, NARROW_SCROLL_CONTENT);
}

fn paint_scroll_gutter(ui: &mut Ui<'_>, body: Rect, content_rows: u16) {
    let height = body.height.saturating_sub(3);
    let thumb = thumb_len(height, content_rows);
    let x = body.right().saturating_sub(1);
    let y0 = body.y.saturating_add(3);
    for row in 0..height {
        let on_thumb = row < thumb;
        ui.paint_str(
            Rect::new(x, y0.saturating_add(row), 1, 1),
            if on_thumb { "┃" } else { "┆" },
            plain(if on_thumb { SCROLL_THUMB } else { SCROLL_TRACK }, CANVAS),
        );
    }
}

fn thumb_len(viewport: u16, content: u16) -> u16 {
    if content <= viewport || viewport == 0 {
        return viewport;
    }
    let len = u32::from(viewport).saturating_mul(u32::from(viewport)) / u32::from(content);
    u16::try_from(len).unwrap_or(viewport).max(1).min(viewport)
}

fn row_band(body: Rect, row: u16, height: u16) -> Option<Rect> {
    if height == 0 || row >= body.height {
        return None;
    }
    Some(Rect {
        y: body.y.saturating_add(row),
        height: height.min(body.height.saturating_sub(row)),
        ..body
    })
}

fn paint_card(ui: &mut Ui<'_>, card: Rect, narrow: bool, compact: bool) {
    if card.width < 8 || card.height < 4 {
        return;
    }
    ui.fill(card, ink(WHITE, SURFACE));
    paint_at(ui, card, 2, card.y, "General", ink(SECONDARY, SURFACE));
    let rule_w = card.width.saturating_sub(4);
    if card.height > 1 && rule_w > 0 {
        ui.paint_str(
            Rect {
                x: card.x.saturating_add(2),
                y: card.y.saturating_add(1),
                width: rule_w,
                height: 1,
            },
            &"─".repeat(usize::from(rule_w)),
            ink(RULE, SURFACE),
        );
    }
    let content = Rect {
        x: card.x.saturating_add(2),
        y: card.y.saturating_add(2),
        width: card.width.saturating_sub(4),
        height: card.height.saturating_sub(3),
    };
    if content.is_empty() {
        return;
    }
    if let Some(left_w) = left_width(content.width) {
        paint_columns(ui, content, left_w);
    } else {
        paint_stacked(ui, content, !narrow && !compact);
        if narrow {
            paint_at(
                ui,
                card,
                4,
                card.bottom().saturating_sub(1),
                "Visibility",
                plain(SECONDARY, SURFACE),
            );
        } else if compact {
            paint_field(
                ui,
                Rect {
                    x: card.x.saturating_add(2),
                    y: card.bottom().saturating_sub(1),
                    width: card.width.saturating_sub(4),
                    height: 1,
                },
                "",
                false,
            );
        }
    }
}

fn paint_columns(ui: &mut Ui<'_>, content: Rect, left_w: u16) {
    let left = Rect {
        width: left_w,
        ..content
    };
    let right_x = left_w.saturating_add(COLUMN_GAP);
    let right = Rect {
        x: content.x.saturating_add(right_x),
        width: content.width.saturating_sub(right_x),
        ..content
    };
    paint_name_label(ui, left);
    paint_field(ui, row(left, 1), "payments-gateway", false);
    paint_at(
        ui,
        left,
        2,
        left.y.saturating_add(3),
        "Description",
        ink(SECONDARY, SURFACE),
    );
    let desc = Rect {
        y: left.y.saturating_add(4),
        height: 3.min(left.height.saturating_sub(4)),
        ..left
    };
    if desc.height > 0 {
        paint_field(ui, desc, DESC, true);
    }
    paint_at(ui, right, 2, right.y, "Visibility", ink(SECONDARY, SURFACE));
    for (index, (label, on)) in [("Private", true), ("Internal", false), ("Public", false)]
        .into_iter()
        .enumerate()
    {
        let y = right.y.saturating_add(1).saturating_add(index as u16);
        if y >= right.bottom() {
            break;
        }
        paint_mark_row(
            ui,
            right,
            y,
            if on { "(●)" } else { "( )" },
            if on { ACCENT } else { MUTED },
            label,
            None,
        );
    }
    paint_toggle(
        ui,
        right,
        right.y.saturating_add(5),
        false,
        "Auto-merge approved PRs",
    );
    paint_toggle(
        ui,
        right,
        right.y.saturating_add(6),
        true,
        "Protect main branch",
    );
    paint_save(ui, content);
}

fn paint_stacked(ui: &mut Ui<'_>, content: Rect, save: bool) {
    paint_name_label(ui, content);
    paint_field(ui, row(content, 1), "payments-gateway", false);
    paint_at(
        ui,
        content,
        2,
        content.y.saturating_add(3),
        "Description",
        ink(SECONDARY, SURFACE),
    );
    let desc = Rect {
        y: content.y.saturating_add(4),
        height: 3.min(content.height.saturating_sub(4)),
        ..content
    };
    if desc.height > 0 {
        paint_field(ui, desc, DESC, true);
    }
    if save {
        paint_save(ui, content);
    }
}

fn paint_name_label(ui: &mut Ui<'_>, area: Rect) {
    if area.height == 0 || area.width <= 2 {
        return;
    }
    let fit = usize::from(area.width.saturating_sub(2));
    let mut label = String::from("Project name *");
    let pad = fit.saturating_sub(usize::from(width(&label)));
    label.extend(std::iter::repeat_n(' ', pad));
    paint_at(ui, area, 2, area.y, &label, ink(SECONDARY, SURFACE));
    if area.width >= width("Project name").saturating_add(4) {
        paint_at(
            ui,
            area,
            2 + width("Project name") + 1,
            area.y,
            "*",
            ink(MUTED, SURFACE),
        );
    }
}

fn row(area: Rect, offset: u16) -> Rect {
    Rect {
        y: area.y.saturating_add(offset),
        height: u16::from(offset < area.height),
        ..area
    }
}

fn paint_field(ui: &mut Ui<'_>, area: Rect, text: &str, clip_tail: bool) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, ink(WHITE, FIELD));
    let gutter = ink(FIELD, FIELD);
    for y in area.y..area.bottom() {
        ui.paint_str(
            Rect {
                x: area.x,
                y,
                width: 1,
                height: 1,
            },
            " ",
            gutter,
        );
    }
    if area.width < 3 || area.height == 0 {
        return;
    }
    let inner_w = if clip_tail {
        area.width.saturating_sub(4)
    } else {
        area.width.saturating_sub(3)
    };
    let (shown, clipped) = clip_prefix(text, inner_w, clip_tail);
    paint_at(ui, area, 2, area.y, &shown, ink(WHITE, FIELD));
    if clipped && inner_w > 0 {
        paint_at(ui, area, 2 + inner_w - 1, area.y, "…", ink(MUTED, FIELD));
    }
}

fn clip_prefix(text: &str, cols: u16, clip_tail: bool) -> (String, bool) {
    if !clip_tail || width(text) <= cols {
        return (text.to_owned(), false);
    }
    let keep = usize::from(cols.saturating_sub(1));
    (text.chars().take(keep).collect(), true)
}

fn paint_mark_row(
    ui: &mut Ui<'_>,
    area: Rect,
    y: u16,
    mark: &str,
    mark_fg: u32,
    label: &str,
    state: Option<&str>,
) {
    if y < area.y || y >= area.bottom() {
        return;
    }
    let row = Rect {
        y,
        height: 1,
        ..area
    };
    paint_at(ui, row, 0, y, " ", ink(SURFACE, SURFACE));
    paint_at(ui, row, 1, y, mark, ink(mark_fg, SURFACE));
    paint_at(ui, row, 5, y, label, ink(WHITE, SURFACE));
    if let Some(state) = state {
        let offset = 6u16.saturating_add(width(label));
        if offset.saturating_add(3) < area.width {
            paint_at(ui, row, offset, y, state, ink(MUTED, SURFACE));
        }
    }
}

fn paint_toggle(ui: &mut Ui<'_>, area: Rect, y: u16, on: bool, label: &str) {
    let (mark, mark_fg, state) = if on {
        ("──●", ACCENT, "on")
    } else {
        ("○──", MUTED, "off")
    };
    paint_mark_row(ui, area, y, mark, mark_fg, label, Some(state));
}

fn paint_save(ui: &mut Ui<'_>, content: Rect) {
    if content.height == 0 {
        return;
    }
    let y = content.bottom().saturating_sub(1);
    let button_w = width("Save changes").saturating_add(2).min(content.width);
    if button_w == 0 {
        return;
    }
    let face = bold(BUTTON_INK, ACCENT);
    ui.fill(
        Rect {
            x: content.x,
            y,
            width: button_w,
            height: 1,
        },
        face,
    );
    ui.paint_str(
        Rect {
            x: content.x,
            y,
            width: 1,
            height: 1,
        },
        " ",
        plain(ACCENT, ACCENT),
    );
    paint_at(ui, content, 1, y, "Save changes", face);
    let note_x = button_w.saturating_add(2);
    paint_at(ui, content, note_x, y, "No changes", ink(FAINT, SURFACE));
}

#[derive(Debug)]
pub struct SettingsPage {
    tabs: TabsState,
    members: Vec<Member>,
    member_state: ListState,
    member_edit_open: bool,
    member_edit_id: Option<u8>,
    member_edit_order: Vec<u8>,
    member_grid_state: GridState,
    remove_state: DialogState,
    remove_open: bool,
    remove_target: Option<u8>,
    selected: usize,
    message: &'static str,
}

impl SettingsPage {
    pub fn new() -> Self {
        Self {
            tabs: TabsState::default(),
            members: INITIAL_MEMBERS.to_vec(),
            member_state: ListState::default(),
            member_edit_open: false,
            member_edit_id: None,
            member_edit_order: Vec::new(),
            member_grid_state: GridState::default(),
            remove_state: DialogState::default(),
            remove_open: false,
            remove_target: None,
            selected: 0,
            message: "workspace settings",
        }
    }

    fn selected_member(&self) -> Option<&Member> {
        self.members.get(self.selected)
    }

    fn open_member_editor(&mut self, id: u8, cx: &mut Cx<'_>) {
        self.member_edit_open = true;
        self.member_edit_id = Some(id);
        self.member_edit_order.clear();
        self.member_edit_order.push(id);
        self.member_edit_order.extend(
            self.members
                .iter()
                .filter(|member| member.id != id)
                .map(|member| member.id),
        );
        self.member_grid_state = GridState::default();
        self.message = "editing member";
        cx.focus(MEMBER_GRID);
    }

    fn close_member_editor(&mut self, cx: &mut Cx<'_>) {
        self.member_edit_open = false;
        self.member_grid_state = GridState::default();
        self.member_edit_order.clear();
        if let Some(id) = self.member_edit_id.take()
            && let Some(index) = self.members.iter().position(|member| member.id == id)
        {
            self.selected = index;
            if let Some(member) = self.members.get(index) {
                self.member_state.set_cursor(index, member_key(member));
            }
        } else if !self.members.is_empty() {
            self.selected = self.selected.min(self.members.len().saturating_sub(1));
            if let Some(member) = self.members.get(self.selected) {
                self.member_state
                    .set_cursor(self.selected, member_key(member));
            }
        }
        self.message = "members";
        cx.focus(MEMBERS);
    }
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for SettingsPage {
    fn title(&self) -> &'static str {
        "Settings"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        if self.member_edit_open {
            let mut result = Response::ignored();
            let grid = member_grid();
            let mut committed = false;
            let edited = {
                let mut model = MemberGridModel {
                    members: self.members.as_mut_slice(),
                    row_ids: &self.member_edit_order,
                    committed: Some(&mut committed),
                };
                grid.update_editable(cx, &mut self.member_grid_state, &mut model)
            };
            result |= edited.erase();
            if committed {
                self.message = "member updated";
            }

            let back = member_back_button().update(cx);
            if back.activated() {
                if self.member_grid_state.is_editing() {
                    self.message = "commit or cancel the active cell edit first";
                    cx.focus(grid.editor_id());
                } else {
                    self.close_member_editor(cx);
                }
            }
            result |= back.erase();
            return result.into();
        }

        let mut result = Response::ignored();
        // The editor mode may have just closed while Runtime still has its
        // focus-retirement notification queued for the Back owner.
        result |= member_back_button().update(cx).erase();
        let tabs = Tabs::new(TAB).update(cx, &mut self.tabs, TABS);
        if let Some(TabsAction::Activated(key)) = tabs.action_ref() {
            self.message = match key {
                ItemKey::Index(0) => "general settings",
                ItemKey::Index(1) => "members",
                ItemKey::Index(2) => "security",
                _ => "tab changed",
            };
        }
        result |= tabs.erase();
        let list = member_list().update(cx, &mut self.member_state, &self.members);
        let activated = match list.action_ref() {
            Some(ListAction::Activated(key)) => Some(*key),
            _ => None,
        };
        if let Some(ListAction::Moved | ListAction::Chose(_) | ListAction::Activated(_)) =
            list.action_ref()
            && let Some(key) = list.action_ref().and_then(|action| match action {
                ListAction::Moved => self.member_state.cursor(),
                ListAction::Chose(key) | ListAction::Activated(key) => Some(*key),
                _ => None,
            })
            && let Some(index) = self
                .members
                .iter()
                .position(|member| member_key(member) == key)
        {
            self.selected = index;
            self.message = "member selected";
        }
        result |= list.erase();
        if let Some(key) = activated
            && let Some(member) = self.members.iter().find(|member| member_key(member) == key)
        {
            let id = member.id;
            self.open_member_editor(id, cx);
        }
        let invite = invite_button().update(cx);
        if invite.activated() {
            self.message = "invite flow ready";
        }
        result |= invite.erase();
        let remove = remove_button(!self.members.is_empty()).update(cx);
        if remove.activated() && self.selected_member().is_some() && !cx.is_open(REMOVE_DIALOG) {
            self.remove_target = self.selected_member().map(|member| member.id);
            self.remove_open = true;
            cx.open_layer(REMOVE_DIALOG, remove_dialog().layer(cx));
        }
        result |= remove.erase();
        let dialog = remove_dialog().update(cx, &mut self.remove_state);
        if let Some(action) = dialog.action_ref() {
            match action {
                DialogAction::Action(key) if *key == termrock::ActionKey::CONFIRM => {
                    if let Some(target) = self.remove_target.take()
                        && let Some(index) =
                            self.members.iter().position(|member| member.id == target)
                    {
                        self.members.remove(index);
                        self.selected = index.min(self.members.len().saturating_sub(1));
                        if let Some(member) = self.members.get(self.selected) {
                            self.member_state
                                .set_cursor(self.selected, member_key(member));
                        }
                    }
                    self.message = "member removed";
                }
                DialogAction::Action(_) | DialogAction::Dismissed(_) => {
                    self.remove_target = None;
                    self.message = "remove cancelled";
                }
            }
            cx.close_layer(REMOVE_DIALOG, None);
            self.remove_open = false;
        }
        result |= dialog.erase();
        result.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            "Project settings",
            "Composed: tabs, form, editable table, list, dialogs",
            |ui, body| {
                if body.is_empty() {
                    return;
                }
                ui.with_area(body, |ui| {
                    if self.member_edit_open {
                        paint_at(
                            ui,
                            body,
                            0,
                            body.y,
                            "Member details · edit Name and Role; Email and Last active are read-only",
                            ink(SECONDARY, SURFACE),
                        );
                        // Keep the regular Grid band, moving Back to the last row
                        // only when a shortened frame clips its normal action row.
                        let back_row = if body.height > 16 {
                            16
                        } else {
                            body.height.saturating_sub(1)
                        };
                        let grid_row = if body.height > 5 { 3 } else { 1 };
                        let grid_height = back_row.saturating_sub(grid_row).saturating_sub(1);
                        if let Some(grid_area) = row_band(body, grid_row, grid_height) {
                            let model = MemberGridModel {
                                members: self.members.as_slice(),
                                row_ids: &self.member_edit_order,
                                committed: None,
                            };
                            member_grid().draw(
                                ui,
                                grid_area,
                                &self.member_grid_state,
                                &model,
                            );
                        }
                        if let Some(action_row) = row_band(body, back_row, 1) {
                            member_back_button().draw(ui, action_row);
                        }
                        return;
                    }

                    let member_tab = active_tab(&self.tabs) == 1;
                    if let Some(tabs_area) = row_band(body, 0, 2) {
                        let tabs = Tabs::new(TAB);
                        let previous = ui.layout(TAB).map(|facts| {
                            termrock::navigation::tabs::TabWindowReceipt {
                                viewport_len: facts.viewport_len,
                                content_len: facts.content_len,
                            }
                        });
                        let layout =
                            tabs.layout_default(tabs_area, ui.full(), &self.tabs, TABS, previous);
                        if tabs.draw_layout(ui, &layout).is_err() {
                            let _ = tabs.draw(ui, tabs_area, &self.tabs, TABS);
                        }
                    }
                    if member_tab {
                        if let Some(list_area) = row_band(body, 3, 12) {
                            member_list().draw(ui, list_area, &self.member_state, &self.members);
                        }
                        if let Some(action_row) = row_band(body, 16, 1) {
                            invite_button().draw(ui, action_row);
                            remove_button(!self.members.is_empty()).draw(ui, action_row);
                        }
                        paint_member_summary(ui, body, &self.members);
                    } else {
                        paint_general(ui, body, active_tab(&self.tabs));
                    }
                });
            },
        );
        paint_boundary_meta(
            ui,
            area,
            "Project settings",
            "Composed: tabs, form, editable table, list, dialogs",
        );
        ui.layer(REMOVE_DIALOG, |ui, layer| {
            remove_dialog().draw(ui, layer, &self.remove_state, |ui, body| {
                let _ = ui.paint_str(
                    body,
                    "Enter confirms · Esc keeps the member",
                    ui.surface_style(),
                );
            });
        });
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.member_edit_open
            && ui
                .state(MEMBER_GRID.part(Part::TEXT))
                .contains(StateFlags::EDITING)
        {
            &[
                ("Enter", "Commit cell"),
                ("Esc", "Cancel cell"),
                ("Tab", "Next"),
            ]
        } else if self.member_edit_open {
            &[("Enter", "Edit cell"), ("← →", "Cell"), ("Tab", "Back")]
        } else if ui.state(TAB).contains(StateFlags::FOCUSED) {
            &[("← →", "Switch tab"), ("1 2 3", "Jump")]
        } else if ui.state(MEMBERS).contains(StateFlags::FOCUSED) {
            &[("↑ ↓", "Select member"), ("Enter", "Edit member")]
        } else {
            &[("Enter", "Edit / activate"), ("Ctrl+S", "Save")]
        }
    }
}
