use termrock::{
    ActionKey, Button, Color, Cx, Dialog, DialogAction, DialogState, Id, Modifier, PaintStyle,
    Rect, Response, Ui, Variant, width,
};

use super::{Page, PageUpdate, frame};

const OPEN_CONFIRM: Id = termrock::Id::root("showcase_ui::pages::dialogs::dialogs.confirm.open");
const OPEN_PROMPT: Id = termrock::Id::root("showcase_ui::pages::dialogs::dialogs.prompt.open");
const OPEN_CHOICE: Id = termrock::Id::root("showcase_ui::pages::dialogs::dialogs.choice.open");
const OPEN_DELETE: Id = termrock::Id::root("showcase_ui::pages::dialogs::dialogs.delete.open");
const CONFIRM: Id = termrock::Id::root("showcase_ui::pages::dialogs::dialogs.confirm.layer");
const PROMPT: Id = termrock::Id::root("showcase_ui::pages::dialogs::dialogs.prompt.layer");
const DIALOG_LABEL_PATCH: termrock::StylePatch = termrock::StylePatch::new()
    .set_fg(termrock::Role::Accent)
    .add(termrock::Modifier::BOLD);
const DIALOG_PARTS: &[(termrock::Part, termrock::StylePatch)] =
    &[(termrock::Part::TITLE, DIALOG_LABEL_PATCH)];

const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const BLURB: &str = "Focus is trapped, the page dims, Esc always cancels";

const OPEN_H: u16 = 8;
const APPLIED_H: u16 = 13;
const CARD_GAP: u16 = 1;
const SLOT_MIN: u16 = 20;
const SLOT_GAP: u16 = 2;
const LABEL_W: u16 = 12;

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const ACCENT: u32 = 0x48e054;
const TAB_ACTIVE: u32 = 0x27272a;
const ON_ACCENT: u32 = 0x19191c;
const DANGER_FG: u32 = 0xff6b6b;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;

// 72- and 80-column shells keep the 120-column playground and scroll.
const CONTENT_MIN_W: u16 = 94;
const RESULTS_H: u16 = 8;
const SCROLL_ROOM: u16 = 2;

const TASK_LINE: &str = "Task: Migrate sessions table · Action(i) or Cancelled";

const PROPS: &[(&str, &str)] = &[
    ("Size", "width 54 · max height none · body scroll on"),
    ("Place", "Center · chrome 0+0 · margin 3x1"),
    ("Backdrop", "keeps 0 top · 1 bottom rows live"),
    (
        "Dismiss",
        "Esc · outside click · close button · typed token: Esc · close button · glyph ×",
    ),
    ("Keys", "mnemonics"),
    ("Nav", "desktop"),
    ("Fields", "Enter in token Submit · guard edited prompt on"),
    ("Edit entry", "Immediate (the prompt's field)"),
    (
        "Buttons",
        "End Wrap · cancel “Cancel” · token gates Drop table",
    ),
    ("Bodies", "Text · Input · Facts"),
];

#[derive(Clone, Copy)]
enum Face {
    Primary,
    Secondary,
    Danger,
}

struct VariantSpec {
    label: &'static str,
    caption: &'static str,
    face: Face,
    launcher: Option<Id>,
}

fn variants() -> [VariantSpec; 8] {
    [
        VariantSpec {
            label: "Confirm run",
            caption: "Text body",
            face: Face::Primary,
            launcher: Some(OPEN_CONFIRM),
        },
        VariantSpec {
            label: "Rename task…",
            caption: "Input body",
            face: Face::Secondary,
            launcher: Some(OPEN_PROMPT),
        },
        VariantSpec {
            label: "Unsaved changes…",
            caption: "three actions",
            face: Face::Secondary,
            launcher: Some(OPEN_CHOICE),
        },
        VariantSpec {
            label: "Delete branch…",
            caption: "▲ title status",
            face: Face::Danger,
            launcher: Some(OPEN_DELETE),
        },
        VariantSpec {
            label: "View config",
            caption: "Facts · Close only",
            face: Face::Secondary,
            launcher: None,
        },
        VariantSpec {
            label: "Drop table…",
            caption: "1 of 2 · 2 of 2",
            face: Face::Secondary,
            launcher: None,
        },
        VariantSpec {
            label: "Push failed…",
            caption: "Retry · Force push",
            face: Face::Secondary,
            launcher: None,
        },
        VariantSpec {
            label: "Release notes",
            caption: "body that scrolls",
            face: Face::Secondary,
            launcher: None,
        },
    ]
}

fn confirm_button() -> Button<'static> {
    Button::new(OPEN_CONFIRM, "Confirm run").variant(Variant::PRIMARY)
}

fn prompt_button() -> Button<'static> {
    Button::new(OPEN_PROMPT, "Rename task…").variant(Variant::SECONDARY)
}

fn choice_button() -> Button<'static> {
    Button::new(OPEN_CHOICE, "Three choices…").variant(Variant::SECONDARY)
}

fn delete_button() -> Button<'static> {
    Button::new(OPEN_DELETE, "Delete branch…").variant(Variant::DANGER)
}

fn launcher(id: Id) -> Button<'static> {
    match id {
        OPEN_CONFIRM => confirm_button(),
        OPEN_PROMPT => prompt_button(),
        OPEN_CHOICE => choice_button(),
        _ => delete_button(),
    }
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

fn tab_width(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn paint_blurb(ui: &mut Ui<'_>, body: Rect) {
    let title_w = width("Dialogs");
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
    ui.paint_str(
        Rect {
            y: rule_y,
            height: 1,
            ..area
        },
        &"─".repeat(usize::from(area.width)),
        ink(RULE, CANVAS),
    );
    let active = tab_width(SECTIONS[0]).min(area.width);
    ui.paint_str(
        Rect::new(area.x, rule_y, active, 1),
        &"━".repeat(usize::from(active)),
        ink(ACCENT, CANVAS),
    );
}

fn paint_card_chrome(ui: &mut Ui<'_>, card: Rect, title: &str, meta: &str) {
    ui.fill(card, ink(WHITE, SURFACE));
    if card.height == 0 || card.width < 4 {
        return;
    }
    let text_x = card.x.saturating_add(2);
    let text_w = card.width.saturating_sub(4);
    ui.paint_str(
        Rect::new(text_x, card.y, text_w, 1),
        title,
        ink(SECONDARY, SURFACE),
    );
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
        ui.paint_str(
            Rect::new(text_x, card.y.saturating_add(1), text_w, 1),
            &"─".repeat(usize::from(text_w)),
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

fn face_style(face: Face) -> (u32, u32, bool) {
    match face {
        Face::Primary => (ON_ACCENT, ACCENT, true),
        Face::Secondary => (WHITE, TAB_ACTIVE, false),
        Face::Danger => (DANGER_FG, TAB_ACTIVE, false),
    }
}

fn paint_face(ui: &mut Ui<'_>, origin: Rect, label: &str, face: Face) {
    let columns = width(label).saturating_add(2);
    let area = Rect {
        width: columns.min(origin.width),
        height: 1,
        ..origin
    };
    if area.is_empty() {
        return;
    }
    let (fg, bg, bold) = face_style(face);
    let mut text = ink(fg, bg);
    if bold {
        text = text.add_modifier(Modifier::BOLD);
    }
    ui.fill(area, text);
    // Cell style merges modifiers, so the gutter must subtract bold.
    ui.fill(
        Rect::new(area.x, area.y, 1, 1),
        ink(bg, bg).remove_modifier(Modifier::BOLD),
    );
    if area.width > 1 {
        ui.paint_str(
            Rect::new(
                area.x.saturating_add(1),
                area.y,
                area.width.saturating_sub(1),
                1,
            ),
            label,
            text,
        );
    }
}

fn columns_per(width: u16) -> usize {
    usize::from((width.saturating_sub(2) / 22).max(1)).min(4)
}

fn paint_open(ui: &mut Ui<'_>, card: Rect) {
    if card.is_empty() {
        return;
    }
    let inner = card_inner(card);
    let specs = variants();
    let per = columns_per(card.width).max(1);
    if !inner.is_empty() {
        for (row, chunk) in specs.chunks(per).enumerate() {
            let Ok(row) = u16::try_from(row) else {
                break;
            };
            let y = inner.y.saturating_add(row.saturating_mul(3));
            if y >= inner.bottom() {
                break;
            }
            let mut x = inner.x;
            for spec in chunk {
                let natural = width(spec.label).saturating_add(2);
                let slot = natural.max(SLOT_MIN);
                if x >= inner.right() {
                    break;
                }
                let room = inner.right().saturating_sub(x);
                if let Some(id) = spec.launcher {
                    launcher(id).draw(ui, Rect::new(x, y, natural.min(room), 1));
                }
                x = x.saturating_add(slot).saturating_add(SLOT_GAP);
            }
        }
    }
    paint_card_chrome(ui, card, "Open a dialog", "Enter · click");
    if inner.is_empty() {
        return;
    }
    for (row, chunk) in specs.chunks(per).enumerate() {
        let Ok(row) = u16::try_from(row) else {
            break;
        };
        let y = inner.y.saturating_add(row.saturating_mul(3));
        if y >= inner.bottom() {
            break;
        }
        let mut x = inner.x;
        for spec in chunk {
            let natural = width(spec.label).saturating_add(2);
            let slot = natural.max(SLOT_MIN);
            if x >= inner.right() {
                break;
            }
            let room = inner.right().saturating_sub(x);
            paint_face(ui, Rect::new(x, y, room, 1), spec.label, spec.face);
            let caption_y = y.saturating_add(1);
            if caption_y < inner.bottom() {
                ui.paint_str(
                    Rect::new(x, caption_y, slot.min(room), 1),
                    spec.caption,
                    ink(FAINT, SURFACE),
                );
            }
            x = x.saturating_add(slot).saturating_add(SLOT_GAP);
        }
    }
}

fn paint_props(ui: &mut Ui<'_>, inner: Rect) {
    for (index, (label, value)) in PROPS.iter().enumerate() {
        let Ok(index) = u16::try_from(index) else {
            break;
        };
        let y = inner.y.saturating_add(index);
        if y >= inner.bottom() {
            break;
        }
        ui.paint_str(
            Rect::new(inner.x, y, width(label).min(inner.width), 1),
            label,
            ink(MUTED, SURFACE),
        );
        if inner.width > LABEL_W {
            ui.paint_str(
                Rect::new(
                    inner.x.saturating_add(LABEL_W),
                    y,
                    inner.width.saturating_sub(LABEL_W),
                    1,
                ),
                value,
                ink(WHITE, SURFACE),
            );
        }
    }
}

fn paint_applied(ui: &mut Ui<'_>, card: Rect) {
    paint_card_chrome(ui, card, "Applied", "read back from the built dialog");
    let inner = card_inner(card);
    if !inner.is_empty() {
        paint_props(ui, inner);
    }
}

fn paint_results(ui: &mut Ui<'_>, card: Rect, result: &str) {
    paint_card_chrome(ui, card, "Results", "DialogResult");
    let inner = card_inner(card);
    if inner.is_empty() {
        return;
    }
    ui.paint_str(
        Rect { height: 1, ..inner },
        TASK_LINE,
        ink(SECONDARY, SURFACE),
    );
    if inner.height > 2 {
        ui.paint_str(
            Rect::new(inner.x, inner.y.saturating_add(2), inner.width, 1),
            result,
            ink(MUTED, SURFACE),
        );
    }
}

fn content_height(area_h: u16) -> u16 {
    let min = OPEN_H
        .saturating_add(CARD_GAP)
        .saturating_add(APPLIED_H)
        .saturating_add(CARD_GAP)
        .saturating_add(RESULTS_H);
    area_h.max(min)
}

struct ScrollSplit {
    view: Rect,
    vertical: bool,
    horizontal: bool,
}

fn scroll_split(area: Rect, content_w: u16, content_h: u16) -> ScrollSplit {
    let (mut vertical, mut horizontal) = (false, false);
    for _ in 0..3 {
        let view_w = area
            .width
            .saturating_sub(if vertical { SCROLL_ROOM } else { 0 });
        let view_h = area.height.saturating_sub(u16::from(horizontal));
        let next_v = content_h > view_h;
        let next_h = content_w > view_w;
        if (next_v, next_h) == (vertical, horizontal) {
            break;
        }
        vertical = next_v;
        horizontal = next_h;
    }
    let vertical = vertical && area.width > SCROLL_ROOM;
    let horizontal = horizontal && area.height > 1;
    let view = Rect {
        width: area
            .width
            .saturating_sub(if vertical { SCROLL_ROOM } else { 0 }),
        height: area.height.saturating_sub(u16::from(horizontal)),
        ..area
    };
    ScrollSplit {
        view,
        vertical,
        horizontal,
    }
}

fn track_thumb(viewport: u16, track: u16, content: u16) -> u16 {
    if content <= viewport || track == 0 {
        return track;
    }
    let len = u32::from(viewport).saturating_mul(u32::from(track)) / u32::from(content);
    u16::try_from(len).unwrap_or(u16::MAX).max(1).min(track)
}

fn paint_page_scrollbars(
    ui: &mut Ui<'_>,
    area: Rect,
    split: &ScrollSplit,
    content_w: u16,
    content_h: u16,
) {
    if split.vertical {
        let bar = Rect::new(area.right().saturating_sub(1), area.y, 1, split.view.height);
        let thumb = track_thumb(split.view.height, bar.height, content_h);
        for row in 0..bar.height {
            let on = row < thumb;
            ui.paint_str(
                Rect::new(bar.x, bar.y.saturating_add(row), 1, 1),
                if on { "┃" } else { "┆" },
                if on {
                    ink(THUMB, CANVAS)
                } else {
                    ink(TRACK, CANVAS)
                },
            );
        }
    }
    if split.horizontal {
        let width = area.width.saturating_sub(u16::from(split.vertical));
        let bar = Rect::new(area.x, area.bottom().saturating_sub(1), width, 1);
        let thumb = track_thumb(split.view.width, bar.width, content_w);
        for col in 0..bar.width {
            let on = col < thumb;
            ui.paint_str(
                Rect::new(bar.x.saturating_add(col), bar.y, 1, 1),
                if on { "━" } else { "┄" },
                if on {
                    ink(THUMB, CANVAS)
                } else {
                    ink(TRACK, CANVAS)
                },
            );
        }
    }
}

fn paint_playground(ui: &mut Ui<'_>, area: Rect, result: &str) {
    if area.is_empty() {
        return;
    }
    let content_w = area.width.max(CONTENT_MIN_W);
    let content_h = content_height(area.height);
    let split = scroll_split(area, content_w, content_h);
    let content = Rect::new(area.x, area.y, content_w, content_h);
    ui.with_area(split.view, |ui| paint_playground_body(ui, content, result));
    paint_page_scrollbars(ui, area, &split, content_w, content_h);
}

fn paint_playground_body(ui: &mut Ui<'_>, area: Rect, result: &str) {
    if area.is_empty() {
        return;
    }
    let mut y = area.y;
    let open_h = OPEN_H.min(area.bottom().saturating_sub(y));
    if open_h > 0 {
        paint_open(ui, Rect::new(area.x, y, area.width, open_h));
    }
    y = y.saturating_add(OPEN_H).saturating_add(CARD_GAP);
    if y >= area.bottom() {
        return;
    }
    let applied_h = APPLIED_H.min(area.bottom().saturating_sub(y));
    if applied_h > 0 {
        paint_applied(ui, Rect::new(area.x, y, area.width, applied_h));
    }
    y = y.saturating_add(APPLIED_H).saturating_add(CARD_GAP);
    if y >= area.bottom() {
        return;
    }
    paint_results(
        ui,
        Rect::new(area.x, y, area.width, area.bottom().saturating_sub(y)),
        result,
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OpenDialog {
    None,
    Confirm,
    Prompt,
}

#[derive(Debug)]
pub struct DialogsPage {
    open: OpenDialog,
    confirm_state: DialogState,
    prompt_state: DialogState,
    error: Option<String>,
    result: String,
}

impl DialogsPage {
    pub fn new() -> Self {
        Self {
            open: OpenDialog::None,
            confirm_state: DialogState::default(),
            prompt_state: DialogState::default(),
            error: None,
            result: String::from("Nothing yet"),
        }
    }

    fn confirm() -> Dialog<'static> {
        Dialog::confirm(
            CONFIRM,
            "Run task now?",
            "The task will be queued for the workspace.",
        )
        .patch_part(DIALOG_PARTS)
    }

    fn prompt(error: Option<&str>) -> Dialog<'_> {
        Dialog::prompt(PROMPT, "Rename task", "Task name")
            .error(error)
            .patch_part(DIALOG_PARTS)
    }

    fn close(&mut self, cx: &mut Cx<'_>, id: Id) {
        if cx.is_open(id) {
            cx.close_layer(id, None);
        }
        self.open = OpenDialog::None;
        if id == PROMPT {
            self.error = None;
        }
    }
}

impl Default for DialogsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for DialogsPage {
    fn title(&self) -> &'static str {
        "Dialogs"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let mut response = Response::ignored();
        let confirm_button = confirm_button().update(cx);
        if confirm_button.activated() && !cx.is_open(CONFIRM) {
            self.open = OpenDialog::Confirm;
            cx.open_layer(CONFIRM, Self::confirm().layer(cx));
        }
        response |= confirm_button.erase();
        let prompt_button = prompt_button().update(cx);
        if prompt_button.activated() && !cx.is_open(PROMPT) {
            self.open = OpenDialog::Prompt;
            self.error = None;
            cx.open_layer(PROMPT, Self::prompt(None).layer(cx));
        }
        response |= prompt_button.erase();
        let choice_button = choice_button().update(cx);
        if choice_button.activated() {
            self.result = String::from("Save selected");
        }
        response |= choice_button.erase();
        let delete_button = delete_button().update(cx);
        if delete_button.activated() {
            self.result = String::from("Cancelled");
        }
        response |= delete_button.erase();

        let action = Self::confirm().update(cx, &mut self.confirm_state);
        if self.open == OpenDialog::Confirm
            && let Some(action) = action.action_ref()
        {
            match action {
                DialogAction::Action(key) if *key == ActionKey::CONFIRM => {
                    self.result = String::from("Task started");
                    self.close(cx, CONFIRM);
                }
                DialogAction::Action(_) | DialogAction::Dismissed(_) => {
                    self.result = String::from("Cancelled");
                    self.close(cx, CONFIRM);
                }
            }
        }
        response |= action.erase();
        let action = Self::prompt(self.error.as_deref()).update(cx, &mut self.prompt_state);
        if self.open == OpenDialog::Prompt
            && let Some(action) = action.action_ref()
        {
            match action {
                DialogAction::Action(key) if *key == ActionKey::CONFIRM => {
                    let name = self.prompt_state.draft().trim();
                    if name.is_empty() {
                        self.error = Some(String::from("Name cannot be empty"));
                        self.result = String::from("Name cannot be empty");
                    } else {
                        self.result = format!("Task: {name}");
                        self.close(cx, PROMPT);
                    }
                }
                DialogAction::Action(_) | DialogAction::Dismissed(_) => {
                    self.result = String::from("Rename cancelled");
                    self.close(cx, PROMPT);
                }
            }
        }
        response |= action.erase();
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "", |ui, body| {
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
            paint_playground(
                ui,
                Rect {
                    y: top,
                    height: body.bottom().saturating_sub(top),
                    ..body
                },
                &self.result,
            );
        });
        ui.layer(CONFIRM, |ui, layer| {
            Self::confirm().draw(ui, layer, &self.confirm_state, |ui, body| {
                let _ = ui.paint_str(body, "Enter confirms · Esc cancels", ui.surface_style());
            });
        });
        ui.layer(PROMPT, |ui, layer| {
            Self::prompt(self.error.as_deref()).draw(ui, layer, &self.prompt_state, |ui, body| {
                let _ = ui.paint_str(body, "Type a name, then Enter", ui.surface_style());
            });
        });
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[("Enter", "Open"), ("d", "Delete branch")]
    }
}

#[cfg(test)]
mod tests {
    use super::{DialogsPage, Page};
    use termrock::{App, Color, Cx, Modifier, Response, Theme, Ui};
    use termrock_test_support::Harness;

    struct Host;

    impl App for Host {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            DialogsPage::new().draw(ui, ui.full());
        }
    }

    #[test]
    fn playground_matches_the_120_column_body() {
        let h = Harness::new(Host, Theme::junie(), 94, 36);
        let text = h.text();
        assert!(text.contains("Dialogs"), "{text}");
        assert!(
            text.contains("Focus is trapped, the page dims, Esc always cancels"),
            "{text}"
        );
        assert!(text.contains("Playground"), "{text}");
        assert!(text.contains("the live widget"), "{text}");
        assert!(text.contains("Open a dialog"), "{text}");
        assert!(text.contains("Enter · click"), "{text}");
        assert!(text.contains("Confirm run"), "{text}");
        assert!(text.contains("▲ title status"), "{text}");
        assert!(text.contains("body that scrolls"), "{text}");
        assert!(text.contains("read back from the built dialog"), "{text}");
        assert!(
            text.contains("width 54 · max height none · body scroll on"),
            "{text}"
        );
        assert!(text.contains("glyph ×"), "{text}");
        assert!(text.contains("cancel “Cancel”"), "{text}");
        assert!(text.contains("Text · Input · Facts"), "{text}");
        assert!(
            text.contains("Task: Migrate sessions table · Action(i) or Cancelled"),
            "{text}"
        );
        assert!(text.contains("Nothing yet"), "{text}");

        let (x, y) = h.find("Playground").expect("tab");
        assert_eq!(h.cell(x, y).bg, Color::Rgb(0x27, 0x27, 0x2a));
        assert!(h.cell(x, y).modifier.contains(Modifier::BOLD));
        assert_eq!(h.cell(x.saturating_sub(1), y + 1).symbol(), "━");

        let (cx, cy) = h.find("Confirm run").expect("primary");
        assert_eq!(h.cell(cx, cy).fg, Color::Rgb(0x19, 0x19, 0x1c));
        assert_eq!(h.cell(cx, cy).bg, Color::Rgb(0x48, 0xe0, 0x54));
        assert!(h.cell(cx, cy).modifier.contains(Modifier::BOLD));
        let gutter = h.cell(cx.saturating_sub(1), cy);
        assert_eq!(gutter.bg, Color::Rgb(0x48, 0xe0, 0x54));
        assert!(!gutter.modifier.contains(Modifier::BOLD));

        let (dx, dy) = h.find("Delete branch…").expect("danger");
        assert_eq!(h.cell(dx, dy).fg, Color::Rgb(0xff, 0x6b, 0x6b));
        assert_eq!(h.cell(dx, dy).bg, Color::Rgb(0x27, 0x27, 0x2a));

        let (sx, sy) = h.find("Size").expect("prop");
        assert_eq!(h.cell(sx, sy).fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(h.cell(sx + 12, sy).symbol(), "w");
    }

    #[test]
    fn narrow_shell_scrolls_the_wide_card_and_ends_the_blurb_on_ellipsis() {
        let h = Harness::new(Host, Theme::junie(), 51, 16);
        let text = h.text();
        assert!(text.contains("Esc alw…"), "{text}");
        assert!(!text.contains("Enter · click"), "{text}");
        assert!(text.contains("Confirm run"), "{text}");
        assert_eq!(h.cell(47, 7).symbol(), "U");
        assert_eq!(h.cell(48, 7).symbol(), "n");
        assert_eq!(h.cell(49, 7).bg, Color::Rgb(0, 0, 0));
        assert_eq!(h.cell(50, 5).symbol(), "┃");
        assert_eq!(h.cell(50, 5).fg, Color::Rgb(0x80, 0x80, 0x80));
        assert_eq!(h.cell(50, 8).symbol(), "┆");
        assert_eq!(h.cell(50, 8).fg, Color::Rgb(0x47, 0x47, 0x47));
        assert_eq!(h.cell(0, 15).symbol(), "━");
        assert_eq!(h.cell(26, 15).symbol(), "┄");
        assert_eq!(h.cell(50, 15).symbol(), " ");
    }
}
