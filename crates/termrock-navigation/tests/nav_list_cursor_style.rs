use ratatui_core::layout::Rect;
use ratatui_core::style::Color;
use termrock_collections::DefaultRow;
use termrock_core::{Id, ItemKey, KeyCode, Part, PartRef, Response, StateFlags};
use termrock_navigation::{NavList, NavListAction, NavListState};
use termrock_runtime::{App, Cx, Focusability, FrameRead, LayerSpec, Ui};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, CursorStyle, GlyphRole, Role, Slot, StylePatch, Theme};

const ID: Id = Id::root("nav-list-cursor-style");
const MODAL: Id = Id::root("nav-list-cursor-style.modal");
const ITEMS: &[Entry] = &[
    Entry {
        key: "first",
        label: "界e\u{301}🙂",
    },
    Entry {
        key: "second",
        label: "Second",
    },
];
const GUTTER_SET: &[(Part, StylePatch)] = &[(
    Part::GUTTER,
    StylePatch::new().set_glyph(GlyphRole::Checked),
)];
const GUTTER_CLEAR: &[(Part, StylePatch)] = &[(
    Part::GUTTER,
    StylePatch {
        glyph: Slot::Clear,
        ..StylePatch::new()
    },
)];
const ROW_FILL: &[(Part, StylePatch)] = &[(
    Part::ROW,
    StylePatch::new().set_bg(Role::Custom(Color::Rgb(12, 34, 56))),
)];
const CONTAINER_FILL: &[(Part, StylePatch)] = &[(
    Part::CONTAINER,
    StylePatch::new().set_bg(Role::Custom(Color::Rgb(78, 90, 12))),
)];
const FILL_PATCHES: &[(Part, StylePatch)] = &[ROW_FILL[0], CONTAINER_FILL[0]];

#[derive(Clone, Copy)]
struct Entry {
    key: &'static str,
    label: &'static str,
}

fn key(entry: &Entry) -> ItemKey {
    ItemKey::text(entry.key)
}

struct Page {
    state: NavListState,
    cursor_style: CursorStyle,
    gutter: Option<&'static [(Part, StylePatch)]>,
    gutter_slot: bool,
    width: u16,
    covered: bool,
    modal_opened: bool,
    action: Option<NavListAction>,
}

impl Page {
    fn list(
        cursor_style: CursorStyle,
        gutter: Option<&'static [(Part, StylePatch)]>,
        gutter_slot: bool,
    ) -> NavList<'static, Entry, fn(&Entry) -> ItemKey, DefaultRow> {
        let list = NavList::new(ID)
            .key(key as fn(&Entry) -> ItemKey)
            .cursor_style(cursor_style);
        let list = if let Some(parts) = gutter {
            list.patch_part(parts)
        } else {
            list
        };
        if gutter_slot {
            list.slot(Part::GUTTER, &paint_gutter)
        } else {
            list
        }
    }
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label)
    }
}

fn paint_gutter(ui: &mut Ui<'_>, area: Rect) {
    let style = ui.surface_style();
    ui.paint_str(area, "!", style);
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.covered && !self.modal_opened {
            self.modal_opened = true;
            cx.focus(ID);
            cx.open_layer(MODAL, LayerSpec::modal(MODAL).initial_focus(MODAL));
        }
        let list = Self::list(self.cursor_style, self.gutter, self.gutter_slot);
        let response = list.update(cx, &mut self.state, ITEMS);
        self.action = response.action_ref().copied();
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Self::list(self.cursor_style, self.gutter, self.gutter_slot).draw(
            ui,
            Rect::new(2, 1, self.width, 2),
            &self.state,
            ITEMS,
        );
        if self.covered {
            ui.layer(MODAL, |ui, area| {
                let control = Rect::new(area.x.saturating_add(14), area.y, 8, 1);
                ui.register_control(MODAL, control, Focusability::Focusable);
                let style = ui.style(
                    termrock_runtime::Family::DIALOG,
                    termrock_runtime::Variant::DEFAULT,
                    Part::CONTAINER,
                    ui.state(MODAL),
                );
                ui.fill(control, style.style);
            });
        }
    }
}

fn harness(cursor_style: CursorStyle) -> Harness<Page> {
    Harness::new(
        Page {
            state: NavListState::new(),
            cursor_style,
            gutter: None,
            gutter_slot: false,
            width: 16,
            covered: false,
            modal_opened: false,
            action: None,
        },
        Theme::junie(),
        24,
        4,
    )
}

#[test]
fn a_modal_takes_focus_away_from_the_filled_background_row() {
    let mut h = harness(CursorStyle::Fill);
    h.app_mut().covered = true;
    let _ = h.tick();
    assert_eq!(h.focus(), Some(MODAL));
    assert_ne!(h.cell(2, 1).symbol(), "▎");
    assert_eq!(h.cell(2, 2).symbol(), " ");
    assert!(h.is_open(MODAL));
}

#[test]
fn default_bar_is_unchanged_and_fill_suppresses_only_the_inherited_focus_gutter() {
    let mut bar = harness(CursorStyle::Bar);
    let mut fill = harness(CursorStyle::Fill);
    assert!(bar.tab_to(ID));
    assert!(fill.tab_to(ID));
    assert_eq!(bar.focus(), Some(ID));
    assert_eq!(fill.focus(), Some(ID));

    assert_eq!(bar.cell(2, 1).symbol(), "▎");
    assert_eq!(fill.cell(2, 1).symbol(), " ");
    assert_eq!(bar.cell(6, 1).symbol(), fill.cell(6, 1).symbol());
    assert_eq!(fill.cell(2, 2).symbol(), " ");
    assert_eq!(bar.cell(6, 2).symbol(), fill.cell(6, 2).symbol());
    assert_eq!(
        bar.area_of_part(ID, PartRef::item(Part::ROW, key(&ITEMS[0]))),
        fill.area_of_part(ID, PartRef::item(Part::ROW, key(&ITEMS[0])))
    );

    let _ = fill.key(KeyCode::Enter);
    assert_eq!(
        fill.app().action,
        Some(NavListAction::Chose(key(&ITEMS[0])))
    );
    assert!(fill.state_of(ID).contains(StateFlags::FOCUSED));
}

#[test]
fn narrow_fill_rows_keep_the_registered_area_and_clip_content_safely() {
    let mut h = harness(CursorStyle::Fill);
    h.app_mut().width = 5;
    assert!(h.tab_to(ID));
    assert_eq!(
        h.area_of_part(ID, PartRef::item(Part::ROW, key(&ITEMS[0]))),
        Some(Rect::new(2, 1, 5, 1))
    );
    assert_eq!(h.cell(2, 1).symbol(), " ");
    let _ = h.text();
}

#[test]
fn explicit_theme_glyph_wins_over_the_fill_gutter_suppression() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let page = Page {
            state: NavListState::new(),
            cursor_style: CursorStyle::Fill,
            gutter: None,
            gutter_slot: false,
            width: 16,
            covered: false,
            modal_opened: false,
            action: None,
        };
        let theme = Theme::junie().override_family(termrock_theme::Family::LIST, |edit| {
            edit.part(Part::GUTTER).glyph(GlyphRole::Checked);
        });
        let mut h = Harness::new(page, theme, 24, 4).with_color(level);
        assert!(h.tab_to(ID));
        assert_eq!(
            h.cell(2, 1).symbol(),
            Theme::junie().design.glyphs.get(GlyphRole::Checked)
        );
    }
}

#[test]
fn explicit_gutter_glyph_clear_and_slot_override_fill_defaults_at_every_profile() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let mut set = harness(CursorStyle::Fill);
        set.app_mut().gutter = Some(GUTTER_SET);
        set = set.with_color(level);
        assert!(set.tab_to(ID));
        assert_eq!(
            set.cell(2, 1).symbol(),
            Theme::junie().design.glyphs.get(GlyphRole::Checked)
        );

        let mut clear = harness(CursorStyle::Fill);
        clear.app_mut().gutter = Some(GUTTER_CLEAR);
        clear = clear.with_color(level);
        assert!(clear.tab_to(ID));
        assert_eq!(clear.cell(2, 1).symbol(), " ");

        let mut slot = harness(CursorStyle::Fill);
        slot.app_mut().gutter = Some(GUTTER_SET);
        slot.app_mut().gutter_slot = true;
        slot = slot.with_color(level);
        assert!(slot.tab_to(ID));
        assert_eq!(slot.cell(2, 1).symbol(), "!");
    }
}

#[test]
fn clear_glyph_is_a_real_authored_slot_value() {
    assert_eq!(GUTTER_CLEAR[0].1.glyph, Slot::Clear);
}

#[test]
fn container_part_override_has_precedence_over_row_fill_default_and_row_patch() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let mut combined = harness(CursorStyle::Fill);
        combined.app_mut().gutter = Some(FILL_PATCHES);
        combined = combined.with_color(level);
        let mut container = harness(CursorStyle::Fill);
        container.app_mut().gutter = Some(CONTAINER_FILL);
        container = container.with_color(level);
        let mut row = harness(CursorStyle::Fill);
        row.app_mut().gutter = Some(ROW_FILL);
        row = row.with_color(level);
        for h in [&mut combined, &mut container, &mut row] {
            assert!(h.tab_to(ID));
        }
        assert_eq!(
            combined.cell(17, 1).bg,
            container.cell(17, 1).bg,
            "{level:?}: authored CONTAINER overrides ROW and Fill defaults"
        );
        if level != ColorLevel::Mono {
            assert_ne!(combined.cell(17, 1).bg, row.cell(17, 1).bg);
        }
    }
}
