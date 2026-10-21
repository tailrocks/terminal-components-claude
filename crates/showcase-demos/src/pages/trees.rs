use termrock::{
    Color, Cx, FgStep, FrameRead, GlyphRole, Id, ItemKey, Modifier, PaintStyle, Panel, PanelKind,
    Part, Rect, Role, RowUi, StateFlags, StylePatch, Tree, TreeAction, TreeNode, TreeState, Ui,
    width,
};

use showcase_data::{TREE, TREE_LABELS};

use super::{Page, PageUpdate, frame};

const PROJECT: Id = termrock::Id::root("showcase_ui::pages::trees::trees.project");
const TREE_GUTTER: &[(Part, StylePatch)] = &[(
    Part::GUTTER,
    StylePatch::new().set_glyph(GlyphRole::FocusBar),
)];
const PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::TITLE,
    StylePatch::new()
        .set_fg(Role::Fg(FgStep::Secondary))
        .remove(termrock::Modifier::BOLD),
)];

const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const TREE_H: u16 = 18;
const SELECTION_H: u16 = 10;
const COLUMN_GAP: u16 = 2;
const INDENT: u16 = 2;
const SCROLL_ROOM: u16 = 2;
// 72- and 80-column shells keep the column split that makes the left card 55 wide.
const CONTENT_MIN_W: u16 = 92;

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
const WARNING: u32 = 0xf59e09;
const DANGER: u32 = 0xff6b6b;

const FADE_OUTER: f32 = 0.55;
const FADE_INNER: f32 = 0.8;

const BLURB: &str = "Indent carries hierarchy; the focus bar never moves";

fn node_key(node: &TreeNode) -> ItemKey {
    node.key().unwrap_or(ItemKey::Num(0))
}

fn node_copy(node: &TreeNode) -> TreeNode {
    *node
}

fn node_label(node: &TreeNode) -> (&'static str, &'static str) {
    let index = match node.key() {
        Some(ItemKey::Num(key)) => key.saturating_sub(1) as usize,
        _ => usize::MAX,
    };
    TREE_LABELS.get(index).copied().unwrap_or(("unknown", ""))
}

fn node_row(node: &TreeNode, row: &mut RowUi<'_>) {
    let (label, meta) = node_label(node);
    row.label(label);
    if !node.has_children() && !meta.is_empty() {
        row.meta(meta);
    }
}

fn project_tree()
-> Tree<'static, TreeNode, impl Fn(&TreeNode) -> ItemKey, impl Fn(&TreeNode, &mut RowUi<'_>)> {
    Tree::new(PROJECT)
        .key(node_key)
        .node(&node_copy)
        .row(node_row)
        .patch_part(TREE_GUTTER)
}

fn project_panel(meta: &str) -> Panel<'_> {
    Panel::new(PROJECT)
        .kind(PanelKind::Card)
        .title("Project")
        .meta(meta)
        .patch_part(PANEL_PARTS)
}

struct FlatRow {
    key: ItemKey,
    depth: u16,
    label: &'static str,
    meta: &'static str,
    branch: bool,
    open: bool,
}

fn flatten(expanded: impl Fn(ItemKey) -> bool) -> Vec<FlatRow> {
    let mut rows = Vec::new();
    let mut hidden = None;
    for node in TREE {
        if hidden.is_some_and(|depth| node.depth() > depth) {
            continue;
        }
        hidden = None;
        let key = node_key(node);
        let open = node.has_children() && expanded(key);
        let (label, meta) = node_label(node);
        rows.push(FlatRow {
            key,
            depth: node.depth(),
            label,
            meta: if node.has_children() { "" } else { meta },
            branch: node.has_children(),
            open,
        });
        if node.has_children() && !open {
            hidden = Some(node.depth());
        }
    }
    rows
}

fn path_for(key: ItemKey) -> Option<String> {
    let mut stack = Vec::new();
    for (index, node) in TREE.iter().enumerate() {
        stack.truncate(usize::from(node.depth()));
        stack.push(TREE_LABELS.get(index).map(|(label, _)| *label)?);
        if node.key() == Some(key) {
            return Some(stack.join("/"));
        }
    }
    None
}

fn label_for(key: Option<ItemKey>) -> &'static str {
    key.and_then(|key| {
        TREE.iter()
            .position(|node| node.key() == Some(key))
            .and_then(|index| TREE_LABELS.get(index).map(|(label, _)| *label))
    })
    .unwrap_or("src")
}

struct ConnRow {
    depth: u16,
    branch: bool,
    open: bool,
    label: &'static str,
    meta: &'static str,
    glyph: Option<(&'static str, u32)>,
    label_fg: u32,
    note: bool,
}

fn connections() -> &'static [ConnRow] {
    &[
        ConnRow {
            depth: 0,
            branch: true,
            open: true,
            label: "production",
            meta: "",
            glyph: Some(("◆", DANGER)),
            label_fg: WHITE,
            note: false,
        },
        ConnRow {
            depth: 1,
            branch: true,
            open: false,
            label: "orders-db",
            meta: "postgres",
            glyph: None,
            label_fg: WHITE,
            note: false,
        },
        ConnRow {
            depth: 1,
            branch: true,
            open: false,
            label: "billing-db",
            meta: "postgres",
            glyph: None,
            label_fg: WHITE,
            note: false,
        },
        ConnRow {
            depth: 1,
            branch: false,
            open: false,
            label: "replica-eu",
            meta: "lag 4.2 s",
            glyph: None,
            label_fg: WARNING,
            note: false,
        },
        ConnRow {
            depth: 0,
            branch: true,
            open: true,
            label: "staging",
            meta: "",
            glyph: Some(("◇", WARNING)),
            label_fg: WHITE,
            note: false,
        },
        ConnRow {
            depth: 1,
            branch: true,
            open: false,
            label: "orders-db",
            meta: "postgres",
            glyph: None,
            label_fg: WHITE,
            note: false,
        },
        ConnRow {
            depth: 0,
            branch: true,
            open: true,
            label: "archive",
            meta: "",
            glyph: Some(("○", MUTED)),
            label_fg: WHITE,
            note: false,
        },
        ConnRow {
            depth: 1,
            branch: false,
            open: false,
            label: "No connections",
            meta: "",
            glyph: None,
            label_fg: MUTED,
            note: true,
        },
    ]
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

fn tab_width(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn paint_blurb(ui: &mut Ui<'_>, body: Rect) {
    let title_w = width("Trees");
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

fn columns(area: Rect) -> (Rect, Rect) {
    let left_w = (area.width.saturating_mul(3) / 5).max(30);
    if area.width < left_w.saturating_add(COLUMN_GAP).saturating_add(20) {
        let height = area.height / 2;
        return (
            Rect::new(area.x, area.y, area.width, height),
            Rect::new(
                area.x,
                area.y.saturating_add(height),
                area.width,
                area.height.saturating_sub(height),
            ),
        );
    }
    (
        Rect::new(area.x, area.y, left_w, area.height),
        Rect::new(
            area.x.saturating_add(left_w).saturating_add(COLUMN_GAP),
            area.y,
            area.width.saturating_sub(left_w).saturating_sub(COLUMN_GAP),
            area.height,
        ),
    )
}

fn clip_bottom(area: Rect, limit: u16) -> Rect {
    let bottom = area.bottom().min(limit);
    Rect {
        height: bottom.saturating_sub(area.y),
        ..area
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

fn paint_card_chrome(ui: &mut Ui<'_>, card: Rect, title: &str, meta: &str) {
    ui.fill(card, ink(WHITE, SURFACE));
    if card.height == 0 || card.width < 3 {
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

fn position_label(offset: usize, viewport: usize, total: usize) -> String {
    if viewport == 0 || total <= viewport {
        return String::new();
    }
    let end = offset.saturating_add(viewport).min(total);
    format!("{}–{} of {}", offset.saturating_add(1), end, total)
}

fn thumb_len(viewport: u16, content: u16) -> u16 {
    if content <= viewport || viewport == 0 {
        return viewport;
    }
    let len = u32::from(viewport).saturating_mul(u32::from(viewport)) / u32::from(content);
    u16::try_from(len).unwrap_or(u16::MAX).max(1).min(viewport)
}

fn paint_scrollbar(ui: &mut Ui<'_>, bar: Rect, content: usize, offset: usize) {
    let content_h = u16::try_from(content).unwrap_or(u16::MAX);
    let thumb = thumb_len(bar.height, content_h);
    let max_off = content.saturating_sub(usize::from(bar.height));
    let travel = usize::from(bar.height.saturating_sub(thumb));
    let origin = offset
        .saturating_mul(travel)
        .checked_div(max_off)
        .unwrap_or(0);
    let origin = u16::try_from(origin)
        .unwrap_or(0)
        .min(bar.height.saturating_sub(thumb));
    for row in 0..bar.height {
        let on_thumb = row >= origin && row < origin.saturating_add(thumb);
        ui.paint_str(
            Rect::new(bar.x, bar.y.saturating_add(row), 1, 1),
            if on_thumb { "┃" } else { "┆" },
            if on_thumb {
                ink(THUMB, SURFACE)
            } else {
                ink(TRACK, SURFACE)
            },
        );
    }
}

struct PaintedRow<'a> {
    depth: u16,
    branch: bool,
    open: bool,
    label: &'a str,
    meta: &'a str,
    glyph: Option<(&'static str, u32)>,
    label_fg: u32,
    note: bool,
    cursor: bool,
    focused: bool,
}

fn row_needs_meta(row: &PaintedRow<'_>, row_w: u16) -> bool {
    if row.meta.is_empty() {
        return true;
    }
    let glyph = u16::from(row.glyph.is_some()).saturating_mul(2);
    let need = 1u16
        .saturating_add(row.depth.saturating_mul(INDENT))
        .saturating_add(2)
        .saturating_add(glyph)
        .saturating_add(width(row.label))
        .saturating_add(2)
        .saturating_add(width(row.meta))
        .saturating_add(1);
    need <= row_w
}

fn fade_keep(
    index: usize,
    viewport: usize,
    total: usize,
    offset: usize,
    cursor: bool,
) -> Option<f32> {
    if cursor || viewport < 4 || total <= viewport {
        return None;
    }
    let depth = usize::from(viewport >= 12) + 1;
    let end = offset.saturating_add(viewport) < total;
    let start = offset > 0;
    if end && index + 1 == viewport {
        Some(FADE_OUTER)
    } else if end && depth == 2 && index + 2 == viewport {
        Some(FADE_INNER)
    } else if start && index == 0 {
        Some(FADE_OUTER)
    } else if start && depth == 2 && index == 1 {
        Some(FADE_INNER)
    } else {
        None
    }
}

fn paint_tree_row(
    ui: &mut Ui<'_>,
    area: Rect,
    row: &PaintedRow<'_>,
    show_meta: bool,
    fade: Option<f32>,
) {
    if area.is_empty() {
        return;
    }
    let keep = |color: u32| fade.map_or(color, |keep| mix(color, SURFACE, keep));
    ui.fill(area, ink(keep(WHITE), SURFACE));
    let gutter = Rect::new(area.x, area.y, 1, 1);
    if row.focused && row.cursor {
        ui.paint_str(
            gutter,
            "▎",
            ink(ACCENT, SURFACE).add_modifier(Modifier::BOLD),
        );
    } else {
        ui.fill(gutter, ink(SURFACE, SURFACE));
    }
    let fold_x = area
        .x
        .saturating_add(1)
        .saturating_add(row.depth.saturating_mul(INDENT));
    if row.branch {
        ui.paint_str(
            Rect::new(fold_x, area.y, 1, 1),
            if row.open { "▾" } else { "▸" },
            ink(keep(SECONDARY), SURFACE),
        );
    }
    let mut label_x = fold_x.saturating_add(2);
    if let Some((glyph, color)) = row.glyph {
        ui.paint_str(
            Rect::new(label_x, area.y, width(glyph), 1),
            glyph,
            ink(keep(color), SURFACE),
        );
        label_x = label_x.saturating_add(2);
    }
    let meta_w = if show_meta && !row.meta.is_empty() {
        width(row.meta)
    } else {
        0
    };
    let avail = area.right().saturating_sub(fold_x.saturating_add(2));
    let reserve = if meta_w > 0 {
        meta_w.saturating_add(2)
    } else {
        1
    };
    let fit_w = avail
        .saturating_sub(reserve)
        .saturating_sub(u16::from(row.glyph.is_some()).saturating_mul(2));
    let meta_x = area.right().saturating_sub(meta_w.saturating_add(1));
    if fit_w > 0 && label_x < area.right() {
        let mut style = ink(keep(row.label_fg), SURFACE);
        if row.note {
            style = style.remove_modifier(Modifier::BOLD);
        }
        ui.paint_str(Rect::new(label_x, area.y, fit_w, 1), row.label, style);
        let label_w = width(row.label).min(fit_w);
        let pad = fit_w.saturating_sub(label_w);
        if pad > 0 {
            ui.paint_str(
                Rect::new(label_x.saturating_add(label_w), area.y, pad, 1),
                &" ".repeat(usize::from(pad)),
                style,
            );
        }
    }
    if meta_w > 0 && meta_x < area.right() {
        ui.paint_str(
            Rect::new(meta_x, area.y, meta_w, 1),
            row.meta,
            ink(keep(MUTED), SURFACE),
        );
    }
}

fn paint_rows(ui: &mut Ui<'_>, inner: Rect, rows: &[PaintedRow<'_>], offset: usize) {
    if inner.is_empty() {
        return;
    }
    let total = rows.len();
    let viewport = usize::from(inner.height);
    let overflow = total > viewport;
    let row_w = inner.width.saturating_sub(u16::from(overflow));
    let show_meta = rows
        .get(offset..)
        .unwrap_or(rows)
        .iter()
        .take(viewport)
        .all(|row| row_needs_meta(row, row_w));
    for (index, row) in rows.iter().skip(offset).take(viewport).enumerate() {
        let Ok(y) = u16::try_from(index) else {
            break;
        };
        let fade = fade_keep(index, viewport, total, offset, row.cursor);
        paint_tree_row(
            ui,
            Rect::new(inner.x, inner.y.saturating_add(y), row_w, 1),
            row,
            show_meta,
            fade,
        );
    }
    if overflow {
        paint_scrollbar(
            ui,
            Rect::new(inner.right().saturating_sub(1), inner.y, 1, inner.height),
            total,
            offset,
        );
    }
}

fn paint_selection(ui: &mut Ui<'_>, card: Rect, state: &TreeState, rows: usize, open: usize) {
    paint_card_chrome(ui, card, "Selection", "");
    let inner = card_inner(card);
    if inner.is_empty() {
        return;
    }
    let primary = ink(WHITE, SURFACE);
    let muted = ink(MUTED, SURFACE);
    let faint = ink(FAINT, SURFACE);
    let secondary = ink(SECONDARY, SURFACE);
    let chosen = state.chosen().and_then(path_for);
    let mut lines: Vec<(&str, PaintStyle, Option<String>)> = Vec::new();
    if let Some(path) = chosen.as_deref() {
        lines.push((path, primary, None));
        lines.push((
            "",
            muted,
            Some(format!("depth {}", path.matches('/').count())),
        ));
    } else {
        lines.push(("Nothing selected", muted, None));
        lines.push(("Enter on a file selects it", faint, None));
    }
    lines.push(("", faint, None));
    let cursor = label_for(state.cursor());
    let facts = [
        ("cursor", cursor.to_owned()),
        ("visible", format!("{rows} rows")),
        ("open", format!("{open} folders")),
    ];
    for (index, (text, style, owned)) in lines.into_iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        if offset >= inner.height {
            break;
        }
        let shown = owned.as_deref().unwrap_or(text);
        if shown.is_empty() {
            continue;
        }
        ui.paint_str(
            Rect {
                y: inner.y.saturating_add(offset),
                height: 1,
                ..inner
            },
            shown,
            style,
        );
    }
    let base = 3u16;
    for (index, (key, value)) in facts.iter().enumerate() {
        let Ok(step) = u16::try_from(index) else {
            break;
        };
        let y = inner.y.saturating_add(base).saturating_add(step);
        if y >= inner.bottom() {
            break;
        }
        ui.paint_str(Rect::new(inner.x, y, inner.width, 1), key, faint);
        if inner.width > 8 {
            ui.paint_str(
                Rect::new(
                    inner.x.saturating_add(8),
                    y,
                    inner.width.saturating_sub(8),
                    1,
                ),
                value,
                secondary,
            );
        }
    }
}

fn readout(last: &str) -> [String; 3] {
    [
        "branch_activate Toggle · click Double · indent 2 · divider false · alt_click_recursive true · reserve_right 0 · show_meta true".to_owned(),
        "nav default · wrap false · type-ahead on · track_click Page · selected_marker None · * TreeCmd::ExpandAll · - TreeCmd::CollapseAll".to_owned(),
        format!(
            "last TreeAction: {last}   (TreeAction::Expand on a lazy node · TreeAction::Activate on a leaf, or a branch per branch_activate)"
        ),
    ]
}

fn wrap_lines(text: &str, columns: usize) -> Vec<String> {
    let columns = columns.max(1);
    let mut lines = Vec::new();
    for para in text.split('\n') {
        let body = para.trim_start_matches(' ');
        let lead = para.len().saturating_sub(body.len());
        let first = if lead.saturating_mul(2) <= columns {
            lead
        } else {
            0
        };
        let hang = hang_column(body, 3).unwrap_or(0);
        let mut cont = first.saturating_add(hang);
        if cont.saturating_mul(2) > columns {
            cont = first;
        }
        wrap_para(body, columns, first, cont, &mut lines);
    }
    lines
}

fn hang_column(text: &str, gap: usize) -> Option<usize> {
    let at = text.find(&" ".repeat(gap))?;
    let rest = text[at..].trim_start_matches(' ');
    if rest.is_empty() {
        return None;
    }
    Some(usize::from(width(
        &text[..text.len().saturating_sub(rest.len())],
    )))
}

fn wrap_para(body: &str, columns: usize, first: usize, cont: usize, lines: &mut Vec<String>) {
    let mut line = " ".repeat(first);
    let mut used = first;
    let mut base = first;
    for word in body.split(' ') {
        let word_w = usize::from(width(word));
        if used == base {
            push_word(&mut line, &mut used, word, word_w, columns);
        } else if used.saturating_add(1).saturating_add(word_w) <= columns {
            line.push(' ');
            line.push_str(word);
            used = used.saturating_add(1).saturating_add(word_w);
        } else {
            lines.push(std::mem::take(&mut line));
            line = " ".repeat(cont);
            used = cont;
            base = cont;
            push_word(&mut line, &mut used, word, word_w, columns);
        }
    }
    lines.push(line);
}

fn push_word(line: &mut String, used: &mut usize, word: &str, word_w: usize, columns: usize) {
    if *used + word_w <= columns {
        line.push_str(word);
        *used = used.saturating_add(word_w);
    }
}

fn paint_readout(ui: &mut Ui<'_>, area: Rect, last: &str) {
    let style = ink(FAINT, CANVAS);
    let mut y = area.y;
    for block in readout(last) {
        for line in wrap_lines(&block, usize::from(area.width)) {
            if y >= area.bottom() {
                return;
            }
            ui.paint_str(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                &line,
                style,
            );
            y = y.saturating_add(1);
        }
    }
}

fn readout_height(columns: u16) -> u16 {
    readout("—")
        .iter()
        .map(|block| {
            u16::try_from(wrap_lines(block, usize::from(columns)).len()).unwrap_or(u16::MAX)
        })
        .fold(0, u16::saturating_add)
}

fn content_height(width: u16) -> u16 {
    let conns_h = u16::try_from(connections().len())
        .unwrap_or(u16::MAX)
        .saturating_add(3);
    top_h(conns_h)
        .saturating_add(1)
        .saturating_add(readout_height(width))
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

fn track_thumb(viewport: u16, track: u16, content: u16) -> u16 {
    if content <= viewport || track == 0 {
        return track;
    }
    let len = u32::from(viewport).saturating_mul(u32::from(track)) / u32::from(content);
    u16::try_from(len).unwrap_or(u16::MAX).max(1).min(track)
}

fn paint_playground(ui: &mut Ui<'_>, area: Rect, page: &TreesPage) {
    if area.is_empty() {
        return;
    }
    let content_w = area.width.max(CONTENT_MIN_W);
    let content_h = content_height(content_w);
    let split = scroll_split(area, content_w, content_h);
    let content = Rect::new(area.x, area.y, content_w, content_h);
    ui.with_area(split.view, |ui| paint_playground_body(ui, content, page));
    paint_page_scrollbars(ui, area, &split, content_w, content_h);
}

fn paint_playground_body(ui: &mut Ui<'_>, area: Rect, page: &TreesPage) {
    if area.is_empty() {
        return;
    }
    let rows = flatten(|key| page.state.is_expanded(key));
    let (left, right) = columns(area);
    let project = clip_bottom(
        Rect {
            height: left.height.min(TREE_H),
            ..left
        },
        area.bottom(),
    );
    let selection_h = right.height.min(SELECTION_H);
    let selection = clip_bottom(
        Rect {
            height: selection_h,
            ..right
        },
        area.bottom(),
    );
    let inner = card_inner(project);
    if !inner.is_empty() {
        // Register the live tree so Tab, fold, and the focus bar stay on one control.
        // The card paint below replaces its pixels with the baseline geometry.
        project_tree().draw(ui, inner, &page.state, TREE);
    }
    let offset = page.state.scroll().offset();
    let label = position_label(offset, usize::from(inner.height), rows.len());
    paint_card_chrome(ui, project, "Project", &label);
    let focused = ui.state(PROJECT).contains(StateFlags::FOCUSED);
    let cursor = page.state.cursor();
    let painted: Vec<PaintedRow<'_>> = rows
        .iter()
        .map(|row| PaintedRow {
            depth: row.depth,
            branch: row.branch,
            open: row.open,
            label: row.label,
            meta: row.meta,
            glyph: None,
            label_fg: WHITE,
            note: false,
            cursor: cursor == Some(row.key),
            focused,
        })
        .collect();
    paint_rows(ui, card_inner(project), &painted, offset);

    paint_selection(
        ui,
        selection,
        &page.state,
        rows.len(),
        page.state.expanded().len_in(TREE.len()),
    );

    let conns = connections();
    let conns_h = u16::try_from(conns.len())
        .unwrap_or(u16::MAX)
        .saturating_add(3);
    let top = right.y.saturating_add(selection_h).saturating_add(1);
    if top.saturating_add(4) < right.bottom() {
        let height = project.bottom().saturating_sub(top).max(conns_h);
        let card = clip_bottom(Rect::new(right.x, top, right.width, height), area.bottom());
        paint_card_chrome(ui, card, "Connections", "→ loads a database");
        let painted: Vec<PaintedRow<'_>> = conns
            .iter()
            .map(|row| PaintedRow {
                depth: row.depth,
                branch: row.branch,
                open: row.open,
                label: row.label,
                meta: row.meta,
                glyph: row.glyph,
                label_fg: row.label_fg,
                note: row.note,
                cursor: false,
                focused: false,
            })
            .collect();
        paint_rows(ui, card_inner(card), &painted, 0);
    }

    let readout_y = area.y.saturating_add(top_h(conns_h)).saturating_add(1);
    if readout_y < area.bottom() {
        paint_readout(
            ui,
            Rect {
                y: readout_y,
                height: area.bottom().saturating_sub(readout_y),
                ..area
            },
            &page.last,
        );
    }
}

fn top_h(conns_h: u16) -> u16 {
    TREE_H.max(SELECTION_H.saturating_add(1).saturating_add(conns_h))
}

#[derive(Debug)]
pub struct TreesPage {
    state: TreeState,
    chosen: Option<ItemKey>,
    last: String,
}

impl TreesPage {
    pub fn new() -> Self {
        let mut state = TreeState::new();
        for key in [1_u64, 20, 26, 29, 30] {
            state.expand(ItemKey::Num(key));
        }
        Self {
            state,
            chosen: None,
            last: "—".to_owned(),
        }
    }
}

impl Default for TreesPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for TreesPage {
    fn title(&self) -> &'static str {
        "Trees"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let result = project_tree().update(cx, &mut self.state, TREE);
        if let Some(action) = result.action_ref() {
            self.last = match action {
                TreeAction::Expanded(_) => "branch expanded".to_owned(),
                TreeAction::Collapsed(_) => "branch collapsed".to_owned(),
                TreeAction::Chose(_) | TreeAction::Activated(_) => "file selected".to_owned(),
                TreeAction::Moved => "cursor moved".to_owned(),
            };
            if let TreeAction::Chose(key) | TreeAction::Activated(key) = action {
                self.chosen = Some(*key);
            }
        }
        let _ = project_panel(&position_label(
            self.state.scroll().offset(),
            self.state.scroll().viewport_len(),
            flatten(|key| self.state.is_expanded(key)).len(),
        ));
        result.erase().into()
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
                self,
            );
        });
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[
            ("↑ ↓", "Move"),
            ("← →", "Fold / unfold"),
            ("Enter", "Open"),
            ("*", "Expand all"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{TreesPage, wrap_lines};
    use crate::pages::Page;
    use termrock::{App, Color, Cx, Modifier, Rect, Response, Theme, Ui};
    use termrock_test_support::Harness;

    struct Host {
        page: TreesPage,
    }

    impl App for Host {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.page.update(cx).response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.page.draw(ui, Rect::new(0, 0, 94, 36));
        }
    }

    fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color::Rgb(r, g, b)
    }

    #[test]
    fn readout_wraps_like_the_baseline() {
        let lines = wrap_lines(
            "branch_activate Toggle · click Double · indent 2 · divider false · alt_click_recursive true · reserve_right 0 · show_meta true",
            94,
        );
        assert_eq!(
            lines,
            vec![
                "branch_activate Toggle · click Double · indent 2 · divider false · alt_click_recursive true ·".to_owned(),
                "reserve_right 0 · show_meta true".to_owned(),
            ]
        );
        let lines = wrap_lines(
            "nav default · wrap false · type-ahead on · track_click Page · selected_marker None · * TreeCmd::ExpandAll · - TreeCmd::CollapseAll",
            94,
        );
        assert_eq!(
            lines,
            vec![
                "nav default · wrap false · type-ahead on · track_click Page · selected_marker None · *".to_owned(),
                "TreeCmd::ExpandAll · - TreeCmd::CollapseAll".to_owned(),
            ]
        );
        let lines = wrap_lines(
            "last TreeAction: —   (TreeAction::Expand on a lazy node · TreeAction::Activate on a leaf, or a branch per branch_activate)",
            94,
        );
        assert_eq!(
            lines,
            vec![
                "last TreeAction: —   (TreeAction::Expand on a lazy node · TreeAction::Activate on a leaf, or a".to_owned(),
                "                     branch per branch_activate)".to_owned(),
            ]
        );
    }

    #[test]
    fn playground_matches_the_120_column_body() {
        let h = Harness::new(
            Host {
                page: TreesPage::new(),
            },
            Theme::junie(),
            94,
            36,
        );
        let row = |y| h.row(y).trim_end().to_owned();
        assert_eq!(
            row(0),
            "Trees  Indent carries hierarchy; the focus bar never moves"
        );
        assert!(row(2).starts_with(" Playground"), "{}", row(2));
        assert!(row(2).contains("the live widget"), "{}", row(2));
        assert!(row(3).starts_with("━━━━━━━━━━━━━"), "{}", row(3));
        assert!(row(5).contains("Project"), "{}", row(5));
        assert!(row(5).contains("1–15 of 16"), "{}", row(5));
        assert!(row(5).contains("Selection"), "{}", row(5));
        assert!(row(7).contains("▾ src"), "{}", row(7));
        assert!(row(11).contains("config.rs"), "{}", row(11));
        assert!(row(11).contains("1.9 KB"), "{}", row(11));
        assert!(row(21).contains("Cargo.toml"), "{}", row(21));
        assert!(row(16).contains("Connections"), "{}", row(16));
        assert!(row(16).contains("→ loads a database"), "{}", row(16));
        assert!(row(18).contains("◆ production"), "{}", row(18));
        assert!(row(21).contains("replica-eu"), "{}", row(21));
        assert!(row(25).contains("No connections"), "{}", row(25));
        assert!(row(7).contains("Nothing selected"), "{}", row(7));
        assert!(row(10).contains("cursor"), "{}", row(10));
        assert!(row(10).contains("src"), "{}", row(10));
        assert!(row(12).contains("5 folders"), "{}", row(12));
        assert!(
            row(28).starts_with("branch_activate Toggle · click Double"),
            "{}",
            row(28)
        );
        assert_eq!(h.cell(2, 7).symbol(), "▎");
        assert_eq!(h.cell(2, 7).fg, rgb(72, 224, 84));
        assert_eq!(h.cell(2, 8).fg, rgb(17, 17, 17));
        assert_eq!(h.cell(2, 8).bg, rgb(17, 17, 17));
        assert_eq!(h.cell(5, 7).fg, rgb(255, 255, 255));
        assert_eq!(h.cell(3, 7).fg, rgb(179, 179, 179));
        assert_eq!(h.cell(46, 11).fg, rgb(138, 138, 138));
        assert_eq!(h.cell(53, 7).symbol(), "┃");
        assert_eq!(h.cell(53, 7).fg, rgb(128, 128, 128));
        assert_eq!(h.cell(53, 21).symbol(), "┆");
        assert_eq!(h.cell(53, 21).fg, rgb(71, 71, 71));
        assert_eq!(h.cell(7, 21).fg, rgb(148, 148, 148));
        assert_eq!(h.cell(7, 20).fg, rgb(207, 207, 207));
        assert_eq!(h.cell(63, 18).fg, rgb(255, 107, 107));
        assert_eq!(h.cell(65, 21).fg, rgb(245, 158, 9));
        assert!(h.cell(0, 0).modifier.contains(Modifier::BOLD));
        assert_eq!(h.cell(1, 2).bg, rgb(39, 39, 42));
    }
}
